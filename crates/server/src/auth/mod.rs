//! AuthModule: dashboard sign-in by magic link.
//!
//! * `POST /api/admin/login` (form `email`): if it's `ADMIN_EMAIL`, email a
//!   single-use link valid for 15 minutes. Always answers the same way, so
//!   the form doesn't reveal who the admin is. Rate-limited per IP.
//! * `GET /api/admin/auth?token=…`: spend the token, start a 30-day session
//!   (HttpOnly cookie), go to `/admin`.
//! * `POST /api/admin/logout`: end the session.
//!
//! Tokens and session ids are random and stored only as SHA-256 hashes. The
//! dashboard's server functions check the same cookie (`site::admin`).

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use axum::Form;
use axum::extract::{ConnectInfo, FromRequest, Query, Request};
use axum::http::{HeaderMap, HeaderValue, header};
use axum::response::{IntoResponse, Redirect};
use contact::mail::Outgoing;
use nestforge::{Container, Inject, Provider, controller, module, routes};
use serde::Deserialize;
use store::secret;

use crate::contact::{ContactModule, ContactService};
use crate::core::{AppConfig, Db, RawResponse, public_origin};

pub use site::admin::SESSION_COOKIE;

const LINK_TTL: i64 = 15 * 60;
const SESSION_TTL: i64 = 30 * 24 * 3600;
/// Sign-in requests per IP per window.
const LIMIT: usize = 5;
const WINDOW: Duration = Duration::from_secs(15 * 60);

#[derive(Default)]
pub struct AuthService {
    attempts: Mutex<HashMap<IpAddr, Vec<Instant>>>,
}

impl AuthService {
    /// Record an attempt; false if this IP is over the limit.
    fn allow(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let mut map = self.attempts.lock().unwrap_or_else(|e| e.into_inner());
        map.retain(|_, v| {
            v.retain(|t| now.duration_since(*t) < WINDOW);
            !v.is_empty()
        });
        let v = map.entry(ip).or_default();
        if v.len() >= LIMIT {
            return false;
        }
        v.push(now);
        true
    }
}

#[derive(Deserialize)]
struct LoginForm {
    #[serde(default)]
    email: String,
}

#[derive(Deserialize)]
struct AuthQuery {
    #[serde(default)]
    token: String,
}

fn client_ip(cfg: &AppConfig, headers: &HeaderMap, peer: Option<IpAddr>) -> IpAddr {
    if cfg.trust_proxy
        && let Some(ip) = headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.split(',').next())
            .and_then(|v| v.trim().parse().ok())
    {
        return ip;
    }
    peer.unwrap_or(IpAddr::from([0, 0, 0, 0]))
}

fn session_cookie(value: &str, max_age: i64, secure: bool) -> HeaderValue {
    let secure = if secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "{SESSION_COOKIE}={value}; Path=/; Max-Age={max_age}; HttpOnly; SameSite=Lax{secure}"
    ))
    .expect("cookie is ASCII")
}

fn redirect_with_cookie(to: &str, cookie: HeaderValue) -> RawResponse {
    let mut res = Redirect::to(to).into_response();
    res.headers_mut().append(header::SET_COOKIE, cookie);
    RawResponse(res)
}

#[controller("/api/admin")]
#[derive(Default)]
pub struct AuthController;

#[routes]
impl AuthController {
    #[nestforge::post("/login")]
    async fn login(
        auth: Inject<AuthService>,
        cfg: Inject<AppConfig>,
        db: Inject<Db>,
        contact: Inject<ContactService>,
        req: Request,
    ) -> RawResponse {
        let headers = req.headers().clone();
        let peer = req
            .extensions()
            .get::<ConnectInfo<SocketAddr>>()
            .map(|c| c.0.ip());
        let ip = client_ip(&cfg, &headers, peer);
        if !auth.allow(ip) {
            tracing::warn!("admin: sign-in rate limit hit for {ip}");
            return RawResponse(Redirect::to("/admin/login?error=rate").into_response());
        }
        let email = Form::<LoginForm>::from_request(req, &())
            .await
            .map(|f| f.0.email.trim().to_ascii_lowercase())
            .unwrap_or_default();
        let is_admin = cfg
            .admin_email
            .as_deref()
            .is_some_and(|a| !a.is_empty() && a == email);
        let sent_page = || RawResponse(Redirect::to("/admin/login?sent=1").into_response());
        if !is_admin {
            tracing::info!("admin: sign-in requested for a non-admin address from {ip}");
            return sent_page();
        }
        let token = secret::new_token();
        let sent = async {
            db.0.add_login_token(&secret::hash(&token), store::now() + LINK_TTL)
                .await
                .map_err(|e| e.to_string())?;
            let link = format!("{}/api/admin/auth?token={token}", public_origin(&cfg, &headers));
            contact
                .mailer
                .send_mail(&Outgoing {
                    to: email.clone(),
                    subject: "Sign in to the Arthub dashboard".into(),
                    text: format!(
                        "Use this link to sign in. It works once and expires in 15 minutes.\n\n{link}\n\n\
                         If you didn't ask for it, ignore this email."
                    ),
                    ..Outgoing::default()
                })
                .await
        }
        .await;
        if let Err(e) = sent {
            tracing::error!("admin: couldn't send the sign-in link: {e}");
            return RawResponse(Redirect::to("/admin/login?error=send").into_response());
        }
        tracing::info!("admin: sign-in link sent");
        sent_page()
    }

    #[nestforge::get("/auth")]
    async fn auth(cfg: Inject<AppConfig>, db: Inject<Db>, q: Query<AuthQuery>) -> RawResponse {
        let ok = !q.token.is_empty()
            && db
                .0
                .take_login_token(&secret::hash(&q.token))
                .await
                .unwrap_or(false);
        if !ok {
            return RawResponse(Redirect::to("/admin/login?error=expired").into_response());
        }
        let session = secret::new_token();
        if let Err(e) =
            db.0.add_session(&secret::hash(&session), store::now() + SESSION_TTL)
                .await
        {
            tracing::error!("admin: couldn't start a session: {e}");
            return RawResponse(Redirect::to("/admin/login?error=server").into_response());
        }
        let _ = db.0.prune().await;
        tracing::info!("admin: signed in");
        redirect_with_cookie(
            "/admin",
            session_cookie(&session, SESSION_TTL, cfg.production),
        )
    }

    #[nestforge::post("/logout")]
    async fn logout(cfg: Inject<AppConfig>, db: Inject<Db>, headers: HeaderMap) -> RawResponse {
        if let Some(s) = site::admin::session_from(&headers) {
            let _ = db.0.end_session(&secret::hash(&s)).await;
        }
        redirect_with_cookie("/admin/login", session_cookie("", 0, cfg.production))
    }
}

#[module(
    controllers = [AuthController],
    imports = [ContactModule],
    providers = [Provider::factory(|_: &Container| Ok(AuthService::default()))],
    exports = [AuthService]
)]
pub struct AuthModule;
