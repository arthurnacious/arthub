//! The dashboard (`/admin`): contact messages with email replies, and the
//! projects, experience and skills the public pages show.
//!
//! Sign-in is by magic link (the server's AuthModule sets [`SESSION_COOKIE`]);
//! every server function here checks that session first.

pub mod api;
pub mod pages;
pub mod types;

/// The dashboard's session cookie (set by the server's AuthModule).
pub const SESSION_COOKIE: &str = "arthub_admin";

/// The session id from a request's cookies.
#[cfg(feature = "ssr")]
pub fn session_from(headers: &http::HeaderMap) -> Option<String> {
    headers
        .get_all(http::header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(';'))
        .find_map(|kv| {
            let (k, v) = kv.trim().split_once('=')?;
            (k == SESSION_COOKIE && !v.is_empty()).then(|| v.to_string())
        })
}

/// What the dashboard's server functions need, provided by the server as
/// Leptos context.
#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct Backend {
    pub store: store::Store,
    pub mailer: std::sync::Arc<contact::mail::Mailer>,
    /// Where visitors' answers to your replies go (your inbox).
    pub reply_to: String,
}
