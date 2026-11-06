//! Delivering enquiries. Configured entirely from the environment:
//!
//! | variable         | meaning                                                   |
//! |------------------|-----------------------------------------------------------|
//! | `CONTACT_TO`     | where enquiries go (your inbox)                           |
//! | `CONTACT_FROM`   | sender address, on a domain your provider lets you send as |
//! | `EMAIL_PROVIDER` | `smtp`, `resend`, `postmark` or `log` (optional, inferred) |
//! | `SMTP_HOST`      | SMTP server; `SMTP_PORT` (587 STARTTLS, 465 TLS), `SMTP_USER`, `SMTP_PASS` |
//! | `EMAIL_API_KEY`  | Resend API key, or Postmark server token                  |
//!
//! With no provider configured, development logs each email to the console;
//! production refuses to start.

use lettre::message::Mailbox;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};

use crate::{Submission, compose};

#[derive(Clone, Debug, PartialEq)]
pub enum Transport {
    Smtp {
        host: String,
        port: u16,
        user: String,
        pass: String,
    },
    Resend {
        key: String,
    },
    Postmark {
        key: String,
    },
    /// Development: print the email instead of sending it.
    Log,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MailConfig {
    pub to: String,
    pub from: String,
    pub transport: Transport,
}

impl MailConfig {
    /// Read the environment. In production every required variable must be
    /// set; the error lists all that are missing.
    pub fn from_env(production: bool) -> Result<Self, String> {
        Self::from_lookup(production, |k| {
            std::env::var(k).ok().filter(|v| !v.trim().is_empty())
        })
    }

    pub fn from_lookup(
        production: bool,
        get: impl Fn(&str) -> Option<String>,
    ) -> Result<Self, String> {
        let mut missing: Vec<&str> = Vec::new();
        let mut need = |k: &'static str| {
            get(k).unwrap_or_else(|| {
                missing.push(k);
                String::new()
            })
        };
        let provider = get("EMAIL_PROVIDER")
            .map(|p| p.to_ascii_lowercase())
            .unwrap_or_else(|| {
                if get("SMTP_HOST").is_some() {
                    "smtp".into()
                } else if get("EMAIL_API_KEY").is_some() {
                    "resend".into()
                } else {
                    "log".into()
                }
            });
        let transport = match provider.as_str() {
            "smtp" => Transport::Smtp {
                host: need("SMTP_HOST"),
                port: get("SMTP_PORT").and_then(|p| p.parse().ok()).unwrap_or(587),
                user: need("SMTP_USER"),
                pass: need("SMTP_PASS"),
            },
            "resend" => Transport::Resend {
                key: need("EMAIL_API_KEY"),
            },
            "postmark" => Transport::Postmark {
                key: need("EMAIL_API_KEY"),
            },
            "log" if !production => Transport::Log,
            "log" => {
                return Err("no email provider configured: set SMTP_HOST/SMTP_USER/SMTP_PASS, or EMAIL_API_KEY \
                            (and EMAIL_PROVIDER=resend|postmark), plus CONTACT_TO and CONTACT_FROM. See .env.example."
                    .into());
            }
            other => {
                return Err(format!(
                    "EMAIL_PROVIDER={other} isn't one of smtp, resend, postmark, log"
                ));
            }
        };
        let (to, from) = if production || transport != Transport::Log {
            (need("CONTACT_TO"), need("CONTACT_FROM"))
        } else {
            (
                get("CONTACT_TO").unwrap_or_else(|| "you@localhost".into()),
                get("CONTACT_FROM").unwrap_or_else(|| "arthub@localhost".into()),
            )
        };
        if !missing.is_empty() {
            return Err(format!(
                "missing environment variables: {} (see .env.example)",
                missing.join(", ")
            ));
        }
        for (k, v) in [("CONTACT_TO", &to), ("CONTACT_FROM", &from)] {
            if v.parse::<Mailbox>().is_err() {
                return Err(format!("{k}={v:?} isn't a valid email address"));
            }
        }
        Ok(Self {
            to,
            from,
            transport,
        })
    }

    pub fn describe(&self) -> String {
        match &self.transport {
            Transport::Smtp { host, port, .. } => format!("SMTP {host}:{port} → {}", self.to),
            Transport::Resend { .. } => format!("Resend API → {}", self.to),
            Transport::Postmark { .. } => format!("Postmark API → {}", self.to),
            Transport::Log => "console log (development: no email provider configured)".into(),
        }
    }
}

pub struct Mailer {
    cfg: MailConfig,
    smtp: Option<AsyncSmtpTransport<Tokio1Executor>>,
    http: Option<reqwest::Client>,
}

impl Mailer {
    pub fn new(cfg: MailConfig) -> Result<Self, String> {
        // One crypto provider for every TLS client in the process.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let smtp = match &cfg.transport {
            Transport::Smtp {
                host,
                port,
                user,
                pass,
            } => {
                let builder = if *port == 465 {
                    AsyncSmtpTransport::<Tokio1Executor>::relay(host)
                } else {
                    AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host)
                }
                .map_err(|e| format!("SMTP host {host}: {e}"))?;
                Some(
                    builder
                        .port(*port)
                        .credentials(Credentials::new(user.clone(), pass.clone()))
                        .build(),
                )
            }
            _ => None,
        };
        let http = matches!(
            cfg.transport,
            Transport::Resend { .. } | Transport::Postmark { .. }
        )
        .then(|| {
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()
        })
        .transpose()
        .map_err(|e| format!("HTTP client: {e}"))?;
        Ok(Self { cfg, smtp, http })
    }

    pub fn config(&self) -> &MailConfig {
        &self.cfg
    }

    /// Send a validated submission to you. Reply-To is the visitor.
    pub async fn send(&self, s: &Submission) -> Result<(), String> {
        let (subject, text) = compose(s);
        self.send_mail(&Outgoing {
            to: self.cfg.to.clone(),
            subject,
            text,
            reply_to: Some(format!(
                "{} <{}>",
                s.name.replace(['<', '>', '"'], ""),
                s.email
            )),
            ..Outgoing::default()
        })
        .await
    }

    /// Send any email from `CONTACT_FROM`.
    pub async fn send_mail(&self, m: &Outgoing) -> Result<(), String> {
        match &self.cfg.transport {
            Transport::Log => {
                tracing::info!(
                    "\n──── email (not sent: no email provider configured) ────\n\
                     To: {}\nFrom: {}\nReply-To: {}\nSubject: {}\nMessage-ID: {}\nIn-Reply-To: {}\n\n{}\n────────────────────────────────────────",
                    m.to,
                    self.cfg.from,
                    m.reply_to.as_deref().unwrap_or("-"),
                    m.subject,
                    m.message_id.as_deref().unwrap_or("-"),
                    m.in_reply_to.as_deref().unwrap_or("-"),
                    m.text
                );
                Ok(())
            }
            Transport::Smtp { .. } => {
                let mut b = Message::builder()
                    .from(
                        self.cfg
                            .from
                            .parse()
                            .map_err(|e| format!("CONTACT_FROM: {e}"))?,
                    )
                    .to(m.to.parse().map_err(|e| format!("to: {e}"))?)
                    .subject(m.subject.clone())
                    .header(ContentType::TEXT_PLAIN);
                if let Some(r) = &m.reply_to {
                    b = b.reply_to(r.parse::<Mailbox>().map_err(|e| format!("reply-to: {e}"))?);
                }
                if let Some(id) = &m.message_id {
                    b = b.message_id(Some(id.clone()));
                }
                if let Some(id) = &m.in_reply_to {
                    b = b.in_reply_to(id.clone());
                }
                if !m.references.is_empty() {
                    b = b.references(m.references.join(" "));
                }
                let msg = b
                    .body(m.text.clone())
                    .map_err(|e| format!("building email: {e}"))?;
                self.smtp
                    .as_ref()
                    .expect("smtp transport")
                    .send(msg)
                    .await
                    .map(|_| ())
                    .map_err(|e| format!("SMTP: {e}"))
            }
            Transport::Resend { key } => {
                let mut body = serde_json::json!({
                    "from": self.cfg.from,
                    "to": [m.to],
                    "subject": m.subject,
                    "text": m.text,
                });
                if let Some(r) = &m.reply_to {
                    body["reply_to"] = serde_json::json!(r);
                }
                let headers = m.thread_headers();
                if !headers.is_empty() {
                    body["headers"] = serde_json::Value::Object(
                        headers
                            .into_iter()
                            .map(|(k, v)| (k.to_string(), serde_json::json!(v)))
                            .collect(),
                    );
                }
                let res = self
                    .http
                    .as_ref()
                    .expect("http client")
                    .post("https://api.resend.com/emails")
                    .bearer_auth(key)
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| format!("Resend: {e}"))?;
                check(res, "Resend").await
            }
            Transport::Postmark { key } => {
                let mut body = serde_json::json!({
                    "From": self.cfg.from,
                    "To": m.to,
                    "Subject": m.subject,
                    "TextBody": m.text,
                    "MessageStream": "outbound",
                });
                if let Some(r) = &m.reply_to {
                    body["ReplyTo"] = serde_json::json!(r);
                }
                let headers = m.thread_headers();
                if !headers.is_empty() {
                    body["Headers"] = serde_json::json!(
                        headers
                            .into_iter()
                            .map(|(k, v)| serde_json::json!({ "Name": k, "Value": v }))
                            .collect::<Vec<_>>()
                    );
                }
                let res = self
                    .http
                    .as_ref()
                    .expect("http client")
                    .post("https://api.postmarkapp.com/email")
                    .header("X-Postmark-Server-Token", key)
                    .header("Accept", "application/json")
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| format!("Postmark: {e}"))?;
                check(res, "Postmark").await
            }
        }
    }
}

/// An email to send. Threading headers are optional: a reply in a
/// conversation sets its own `message_id`, `in_reply_to` the previous one,
/// and `references` the whole chain, so mail clients show one thread.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outgoing {
    pub to: String,
    pub subject: String,
    pub text: String,
    pub reply_to: Option<String>,
    pub message_id: Option<String>,
    pub in_reply_to: Option<String>,
    pub references: Vec<String>,
}

impl Outgoing {
    fn thread_headers(&self) -> Vec<(&'static str, String)> {
        let mut h = Vec::new();
        if let Some(id) = &self.message_id {
            h.push(("Message-ID", id.clone()));
        }
        if let Some(id) = &self.in_reply_to {
            h.push(("In-Reply-To", id.clone()));
        }
        if !self.references.is_empty() {
            h.push(("References", self.references.join(" ")));
        }
        h
    }
}

async fn check(res: reqwest::Response, who: &str) -> Result<(), String> {
    let status = res.status();
    if status.is_success() {
        Ok(())
    } else {
        let text = res.text().await.unwrap_or_default();
        Err(format!(
            "{who} returned {status}: {}",
            text.chars().take(300).collect::<String>()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn cfg(production: bool, vars: &[(&str, &str)]) -> Result<MailConfig, String> {
        let map: HashMap<String, String> = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        MailConfig::from_lookup(production, |k| map.get(k).cloned())
    }

    #[test]
    fn development_without_credentials_logs() {
        let c = cfg(false, &[]).unwrap();
        assert_eq!(c.transport, Transport::Log);
    }

    #[test]
    fn production_without_credentials_refuses() {
        let e = cfg(true, &[]).unwrap_err();
        assert!(e.contains("no email provider"), "{e}");
    }

    #[test]
    fn production_lists_every_missing_variable() {
        let e = cfg(true, &[("SMTP_HOST", "smtp.example.com")]).unwrap_err();
        for k in ["SMTP_USER", "SMTP_PASS", "CONTACT_TO", "CONTACT_FROM"] {
            assert!(e.contains(k), "{k} in {e}");
        }
    }

    #[test]
    fn provider_is_inferred() {
        let smtp = cfg(
            true,
            &[
                ("SMTP_HOST", "h"),
                ("SMTP_USER", "u"),
                ("SMTP_PASS", "p"),
                ("CONTACT_TO", "me@x.co"),
                ("CONTACT_FROM", "site@x.co"),
            ],
        )
        .unwrap();
        assert!(matches!(smtp.transport, Transport::Smtp { port: 587, .. }));
        let resend = cfg(
            true,
            &[
                ("EMAIL_API_KEY", "k"),
                ("CONTACT_TO", "me@x.co"),
                ("CONTACT_FROM", "site@x.co"),
            ],
        )
        .unwrap();
        assert_eq!(resend.transport, Transport::Resend { key: "k".into() });
        let postmark = cfg(
            true,
            &[
                ("EMAIL_PROVIDER", "postmark"),
                ("EMAIL_API_KEY", "k"),
                ("CONTACT_TO", "me@x.co"),
                ("CONTACT_FROM", "site@x.co"),
            ],
        )
        .unwrap();
        assert_eq!(postmark.transport, Transport::Postmark { key: "k".into() });
    }

    #[test]
    fn rejects_bad_addresses_and_providers() {
        assert!(
            cfg(
                true,
                &[
                    ("EMAIL_API_KEY", "k"),
                    ("CONTACT_TO", "nope"),
                    ("CONTACT_FROM", "site@x.co")
                ]
            )
            .unwrap_err()
            .contains("CONTACT_TO")
        );
        assert!(
            cfg(false, &[("EMAIL_PROVIDER", "pigeon")])
                .unwrap_err()
                .contains("pigeon")
        );
    }

    #[tokio::test]
    async fn log_transport_sends_nothing_and_succeeds() {
        let m = Mailer::new(cfg(false, &[]).unwrap()).unwrap();
        let s = Submission {
            name: "A".into(),
            email: "a@b.co".into(),
            project_type: crate::PROJECT_TYPES[0].into(),
            budget: crate::BUDGETS[0].into(),
            message: "Hello hello".into(),
        };
        assert!(m.send(&s).await.is_ok());
    }
}
