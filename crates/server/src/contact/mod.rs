//! ContactModule: `POST /api/contact` (the contact form), `GET
//! /api/contact/token` (time-on-form token), validation, spam checks and
//! delivery, all through the injected [`ContactService`].
//!
//! Two kinds of client:
//! * the hydrated form sends `Accept: application/json` and gets JSON back
//!   (`{ok}` or `{ok: false, errors: {field: message}, error}`);
//! * a plain HTML form POST (no JS/WASM yet) gets a 303 back to `/contact`
//!   with `?sent=1` or `?error=<code>`, which the page renders server-side.
//!
//! Order of checks: honeypot (silently "succeed"), time-on-form token, rate
//! limit, validation, then delivery.

use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use axum::extract::{ConnectInfo, FromRequest, Request, State};
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Form, Json};
use contact::mail::{MailConfig, Mailer};
use contact::spam::{Gate, Guard, TOKEN_COOKIE};
use contact::{HONEYPOT, Submission, validate};
use nestforge::{Container, Inject, Provider, controller, module, routes};
use serde_json::json;

use crate::core::{AppConfig, RawResponse};

pub struct ContactService {
    pub guard: Guard,
    pub mailer: Arc<Mailer>,
    /// Behind a reverse proxy, take the client IP from X-Forwarded-For.
    pub trust_proxy: bool,
    /// Mark the token cookie `Secure` (production, served over HTTPS).
    pub secure_cookie: bool,
    /// Emails actually handed to the mailer (for tests and logs).
    pub sent: AtomicUsize,
    /// Enquiries are kept for the dashboard once `app::build` sets this.
    pub store: std::sync::OnceLock<store::Store>,
}

impl ContactService {
    /// Build from config and the environment's email settings. In
    /// production a missing provider is an error (the app refuses to start);
    /// in development enquiries are logged instead.
    pub fn from_config(cfg: &AppConfig) -> anyhow::Result<Self> {
        let mail = match MailConfig::from_env(cfg.production) {
            Ok(m) => m,
            Err(e) if cfg.production => anyhow::bail!("contact form: {e}"),
            Err(e) => {
                tracing::warn!("contact form: {e}; logging enquiries to the console instead");
                MailConfig::from_lookup(false, |_| None).map_err(anyhow::Error::msg)?
            }
        };
        tracing::info!("contact form delivers via {}", mail.describe());
        Ok(Self {
            guard: Guard::default(),
            mailer: Arc::new(Mailer::new(mail).map_err(anyhow::Error::msg)?),
            trust_proxy: cfg.trust_proxy,
            secure_cookie: cfg.production,
            sent: AtomicUsize::new(0),
            store: std::sync::OnceLock::new(),
        })
    }
}

#[controller("/api/contact")]
#[derive(Default)]
pub struct ContactController;

#[routes]
impl ContactController {
    #[nestforge::post("/")]
    async fn submit(svc: Inject<ContactService>, req: Request) -> RawResponse {
        RawResponse(submit(&svc, req).await)
    }

    #[nestforge::get("/token")]
    async fn token(svc: Inject<ContactService>) -> RawResponse {
        RawResponse(token(&svc))
    }
}

#[module(
    controllers = [ContactController],
    providers = [Provider::factory(|c: &Container| {
        let cfg = c.resolve::<AppConfig>()?;
        ContactService::from_config(&cfg)
    })],
    exports = [ContactService]
)]
pub struct ContactModule;

/// Middleware: whenever the contact page itself is served (a full page load,
/// including no-JS visitors), hand out a form token cookie.
pub async fn token_on_contact_page(
    State(st): State<Arc<ContactService>>,
    req: Request,
    next: Next,
) -> Response {
    let is_page = req.method() == Method::GET && req.uri().path() == "/contact";
    let mut res = next.run(req).await;
    if is_page
        && res.status().is_success()
        && let Ok(v) = cookie(&st)
    {
        res.headers_mut().append(header::SET_COOKIE, v);
    }
    res
}

fn cookie(st: &ContactService) -> Result<HeaderValue, header::InvalidHeaderValue> {
    let t = st.guard.tokens.issue(Instant::now());
    let secure = if st.secure_cookie { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "{TOKEN_COOKIE}={t}; Path=/; Max-Age=21600; HttpOnly; SameSite=Lax{secure}"
    ))
}

/// The hydrated form calls this on mount (client-side navigation never
/// requests the page from the server, so the middleware wouldn't run).
fn token(st: &ContactService) -> Response {
    match cookie(st) {
        Ok(v) => (
            StatusCode::NO_CONTENT,
            [
                (header::SET_COOKIE, v),
                (header::CACHE_CONTROL, HeaderValue::from_static("no-store")),
            ],
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

fn client_ip(headers: &HeaderMap, peer: Option<IpAddr>, trust_proxy: bool) -> IpAddr {
    if trust_proxy
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

fn token_from(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .find_map(|kv| {
            let (k, v) = kv.trim().split_once('=')?;
            (k == TOKEN_COOKIE).then(|| v.to_string())
        })
}

/// Shape a failure for whichever client asked.
fn fail(json: bool, status: StatusCode, code: &str, message: &str) -> Response {
    if json {
        (
            status,
            Json(json!({ "ok": false, "error": code, "message": message })),
        )
            .into_response()
    } else {
        Redirect::to(&format!("/contact?error={code}#contact-form")).into_response()
    }
}

fn ok(json: bool) -> Response {
    if json {
        Json(json!({ "ok": true })).into_response()
    } else {
        Redirect::to("/contact?sent=1#contact-form").into_response()
    }
}

async fn submit(st: &ContactService, req: Request) -> Response {
    let headers = req.headers().clone();
    let peer = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0.ip());
    let json = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.contains("application/json"));
    let ip = client_ip(&headers, peer, st.trust_proxy);

    let Ok(Form(pairs)) = Form::<Vec<(String, String)>>::from_request(req, &()).await else {
        return fail(
            json,
            StatusCode::BAD_REQUEST,
            "invalid",
            "That form didn't come through properly.",
        );
    };
    let get = |k: &str| {
        pairs
            .iter()
            .find(|(n, _)| n == k)
            .map(|(_, v)| v.as_str())
            .unwrap_or("")
    };
    let now = Instant::now();

    match st
        .guard
        .check(get(HONEYPOT), token_from(&headers).as_deref(), ip, now)
    {
        Gate::Open => {}
        Gate::Honeypot => {
            tracing::info!("contact: honeypot filled from {ip}; dropped");
            return ok(json);
        }
        Gate::TooFast => {
            tracing::info!("contact: submitted too fast from {ip}");
            return fail(
                json,
                StatusCode::TOO_MANY_REQUESTS,
                "fast",
                "That was quick! Give it a few seconds and send again.",
            );
        }
        Gate::RateLimited => {
            tracing::warn!("contact: rate limit hit for {ip}");
            return fail(
                json,
                StatusCode::TOO_MANY_REQUESTS,
                "rate",
                "That's a lot of messages in an hour. Try again later, or email me directly.",
            );
        }
    }

    let submission = Submission::from_pairs(pairs.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    let valid = match validate(&submission) {
        Ok(v) => v,
        Err(errors) => {
            if json {
                let map: serde_json::Map<String, serde_json::Value> = errors
                    .into_iter()
                    .map(|(f, m)| (f.key().to_string(), json!(m)))
                    .collect();
                return (
                    StatusCode::UNPROCESSABLE_ENTITY,
                    Json(json!({ "ok": false, "error": "invalid", "errors": map })),
                )
                    .into_response();
            }
            return fail(false, StatusCode::UNPROCESSABLE_ENTITY, "invalid", "");
        }
    };

    st.guard.limiter.record(ip, now);
    // Kept first, so a mail outage doesn't lose it: the dashboard has it.
    let saved = match st.store.get() {
        Some(db) => match db
            .add_message(
                &valid.name,
                &valid.email,
                &valid.project_type,
                &valid.budget,
                &valid.message,
            )
            .await
        {
            Ok(_) => true,
            Err(e) => {
                tracing::error!("contact: couldn't save the enquiry: {e}");
                false
            }
        },
        None => false,
    };
    match st.mailer.send(&valid).await {
        Ok(()) => {
            st.sent.fetch_add(1, Ordering::Relaxed);
            tracing::info!(
                "contact: enquiry from {} <{}> delivered",
                valid.name,
                valid.email
            );
            ok(json)
        }
        Err(e) if saved => {
            tracing::error!(
                "contact: notification email failed ({e}); the enquiry is in the dashboard"
            );
            ok(json)
        }
        Err(e) => {
            tracing::error!("contact: delivery failed: {e}");
            fail(
                json,
                StatusCode::BAD_GATEWAY,
                "send",
                "Couldn't send that just now.",
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::CoreModule;
    use axum::body::{Body, to_bytes};
    use nestforge::{NestForgeFactory, module};
    use std::time::Duration;
    use tower::ServiceExt;

    #[module(imports = [CoreModule, ContactModule])]
    struct TestModule;

    /// A real NestForge app (Core + Contact) and the service instance it uses.
    struct Harness {
        app: axum::Router,
        st: Arc<ContactService>,
    }

    fn harness() -> Harness {
        let f = NestForgeFactory::<TestModule>::create().expect("app");
        let st = f.container().resolve::<ContactService>().expect("service");
        Harness {
            app: f.into_router(),
            st,
        }
    }

    const GOOD: &str = "name=Thandi&email=thandi%40example.co.za&project_type=Website&budget=Not+sure+yet&message=Let%27s+build+something+good.&website=";

    fn post_req(body: &str, json: bool, ip: [u8; 4], cookie: Option<&str>) -> Request {
        let mut b = Request::builder()
            .method("POST")
            .uri("/api/contact")
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
        if json {
            b = b.header(header::ACCEPT, "application/json");
        }
        if let Some(c) = cookie {
            b = b.header(header::COOKIE, format!("other=1; {TOKEN_COOKIE}={c}"));
        }
        let mut req = b.body(Body::from(body.to_string())).unwrap();
        req.extensions_mut()
            .insert(ConnectInfo(SocketAddr::from((ip, 4000))));
        req
    }

    async fn call(h: &Harness, req: Request) -> (StatusCode, String, Option<String>) {
        let res = h.app.clone().oneshot(req).await.unwrap();
        let status = res.status();
        let location = res
            .headers()
            .get(header::LOCATION)
            .map(|v| v.to_str().unwrap().to_string());
        let body =
            String::from_utf8(to_bytes(res.into_body(), 1 << 20).await.unwrap().to_vec()).unwrap();
        (status, body, location)
    }

    #[tokio::test]
    async fn valid_json_submission_is_sent() {
        let h = harness();
        let st = &h.st;
        let (status, body, _) = call(&h, post_req(GOOD, true, [1, 1, 1, 1], None)).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert!(body.contains("\"ok\":true"));
        assert_eq!(st.sent.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn plain_form_post_redirects_back() {
        let h = harness();
        let (status, _, loc) = call(&h, post_req(GOOD, false, [1, 1, 1, 2], None)).await;
        assert_eq!(status, StatusCode::SEE_OTHER);
        assert_eq!(loc.as_deref(), Some("/contact?sent=1#contact-form"));
        let bad = GOOD.replace("Website", "Crypto");
        let (_, _, loc) = call(&h, post_req(&bad, false, [1, 1, 1, 2], None)).await;
        assert_eq!(loc.as_deref(), Some("/contact?error=invalid#contact-form"));
    }

    #[tokio::test]
    async fn invalid_fields_come_back_individually() {
        let h = harness();
        let st = &h.st;
        let (status, body, _) = call(
            &h,
            post_req(
                "name=&email=nope&project_type=Website&budget=Not+sure+yet&message=hi",
                true,
                [1, 1, 1, 3],
                None,
            ),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        let v: serde_json::Value = serde_json::from_str(&body).unwrap();
        for f in ["name", "email", "message"] {
            assert!(v["errors"][f].is_string(), "{f} in {body}");
        }
        assert!(v["errors"]["project_type"].is_null());
        assert_eq!(st.sent.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn honeypot_pretends_success_and_sends_nothing() {
        let h = harness();
        let st = &h.st;
        let body = GOOD.replace("website=", "website=http%3A%2F%2Fspam.example");
        let (status, text, _) = call(&h, post_req(&body, true, [1, 1, 1, 4], None)).await;
        assert_eq!(status, StatusCode::OK);
        assert!(text.contains("\"ok\":true"));
        assert_eq!(st.sent.load(Ordering::Relaxed), 0);
    }

    #[tokio::test]
    async fn too_fast_is_rejected() {
        let h = harness();
        let st = &h.st;
        let token = st.guard.tokens.issue(Instant::now());
        let (status, body, _) = call(&h, post_req(GOOD, true, [1, 1, 1, 5], Some(&token))).await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert!(body.contains("\"fast\""));
        // An older token is fine.
        let old = st
            .guard
            .tokens
            .issue(Instant::now() - Duration::from_secs(10));
        let (status, _, _) = call(&h, post_req(GOOD, true, [1, 1, 1, 5], Some(&old))).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn sixth_submission_in_an_hour_is_rate_limited() {
        let h = harness();
        let st = &h.st;
        for i in 0..5 {
            let (status, _, _) = call(&h, post_req(GOOD, true, [9, 9, 9, 9], None)).await;
            assert_eq!(status, StatusCode::OK, "submission {i}");
        }
        let (status, body, _) = call(&h, post_req(GOOD, true, [9, 9, 9, 9], None)).await;
        assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
        assert!(body.contains("\"rate\""));
        assert_eq!(st.sent.load(Ordering::Relaxed), 5);
        // Someone else is fine.
        let (status, _, _) = call(&h, post_req(GOOD, true, [8, 8, 8, 8], None)).await;
        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn token_endpoint_sets_cookie() {
        let h = harness();
        let res = h
            .app
            .clone()
            .oneshot(
                Request::get("/api/contact/token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::NO_CONTENT);
        let c = res
            .headers()
            .get(header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap();
        assert!(
            c.starts_with(&format!("{TOKEN_COOKIE}="))
                && c.contains("HttpOnly")
                && c.contains("SameSite=Lax")
        );
    }

    #[test]
    fn forwarded_ip_only_when_trusted() {
        let mut h = HeaderMap::new();
        h.insert(
            "x-forwarded-for",
            HeaderValue::from_static("203.0.113.7, 10.0.0.1"),
        );
        let peer = Some(IpAddr::from([10, 0, 0, 1]));
        assert_eq!(client_ip(&h, peer, true), IpAddr::from([203, 0, 113, 7]));
        assert_eq!(client_ip(&h, peer, false), IpAddr::from([10, 0, 0, 1]));
    }
}
