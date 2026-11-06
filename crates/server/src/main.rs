//! Arthub server: NestForge modules (see `app.rs`) hosting the Leptos SSR app.

// Leptos view types nest deeply; the app's route tree needs more headroom.
#![recursion_limit = "512"]

use std::net::SocketAddr;

use leptos::prelude::*;

mod admin;
mod app;
mod auth;
mod contact;
mod content;
mod core;
mod room;
mod seo;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Local development reads secrets from `.env` (never committed).
    let _ = dotenvy::dotenv();
    core::init_logging();

    let conf = get_configuration(None).expect("leptos config (run via cargo-leptos)");
    let options = conf.leptos_options;
    let addr = options.site_addr;

    // Panics with the reason if a module can't start (e.g. production with no
    // email provider): a misconfigured server must not quietly drop enquiries.
    let cfg = core::AppConfig::from_env();
    let store = core::open_store(&cfg)
        .await
        .unwrap_or_else(|e| panic!("startup failed: {e:#}"));
    let app = app::build(options, store).unwrap_or_else(|e| panic!("startup failed: {e:#}"));

    // Served here rather than with NestForge's `listen()`, which binds to
    // 127.0.0.1 only and doesn't pass client addresses (the contact form's
    // per-IP rate limit needs them).
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("arthub listening on http://{addr}");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
