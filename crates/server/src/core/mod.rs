//! CoreModule: configuration from the environment, logging, and error shape.
//! Everything else injects [`AppConfig`] instead of reading env vars itself.

use nestforge::{Container, HttpException, Provider, controller, module, routes};

/// Process-wide settings, read once at startup.
#[derive(Clone, Debug)]
pub struct AppConfig {
    /// `LEPTOS_ENV=PROD`: secure cookies, and missing email config is fatal.
    pub production: bool,
    /// Behind a proxy, trust `X-Forwarded-For` for client IPs (`TRUST_PROXY=1`).
    pub trust_proxy: bool,
    /// Public origin for absolute URLs (`SITE_URL`, e.g. `https://arthub.dev`).
    pub site_url: Option<String>,
    /// Where the database and uploads live (`DATA_DIR`, default `data`).
    pub data_dir: std::path::PathBuf,
    /// Who may sign in to the dashboard (`ADMIN_EMAIL`). Unset: nobody.
    pub admin_email: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.is_empty());
        Self {
            production: var("LEPTOS_ENV").is_some_and(|v| {
                v.eq_ignore_ascii_case("prod") || v.eq_ignore_ascii_case("production")
            }),
            trust_proxy: var("TRUST_PROXY")
                .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true")),
            site_url: var("SITE_URL"),
            data_dir: var("DATA_DIR").unwrap_or_else(|| "data".into()).into(),
            admin_email: var("ADMIN_EMAIL").map(|e| e.trim().to_ascii_lowercase()),
        }
    }
}

/// The site's public origin: `SITE_URL`, else the request's host (https in
/// production).
pub fn public_origin(cfg: &AppConfig, headers: &axum::http::HeaderMap) -> String {
    if let Some(url) = &cfg.site_url {
        return url.trim_end_matches('/').to_string();
    }
    let host = headers
        .get(axum::http::header::HOST)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("127.0.0.1:3000");
    let scheme = if cfg.production { "https" } else { "http" };
    format!("{scheme}://{host}")
}

/// The database, registered into the container by `app::build`.
pub struct Db(pub store::Store);

/// Open (creating) `DATA_DIR/arthub.db`, seed an empty one from the compiled
/// `content/`, and publish its content to the pages.
pub async fn open_store(cfg: &AppConfig) -> anyhow::Result<store::Store> {
    std::fs::create_dir_all(&cfg.data_dir)?;
    let path = cfg.data_dir.join("arthub.db");
    let store = store::Store::open(&path.to_string_lossy()).await?;
    let seed = content::owned::Catalog::from_static(&site::data::SITE);
    if store.seed_if_empty(&seed).await? {
        tracing::info!("database: seeded {} from content/", path.display());
    }
    publish(&store).await?;
    Ok(store)
}

/// Reload the content snapshot every page reads (after an edit).
pub async fn publish(store: &store::Store) -> anyhow::Result<()> {
    site::data::set_site(store.catalog().await?);
    Ok(())
}

/// Structured logs to stdout; `RUST_LOG` overrides the default filter.
pub fn init_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,server=info,contact=info".into()),
        )
        .try_init();
}

/// Liveness probe for hosts and load balancers.
#[controller("/healthz")]
#[derive(Default)]
pub struct HealthController;

#[routes]
impl HealthController {
    #[nestforge::get("/")]
    async fn health() -> Result<String, HttpException> {
        Ok("ok".into())
    }
}

#[module(
    controllers = [HealthController],
    providers = [Provider::factory(|_: &Container| Ok(AppConfig::from_env()))],
    exports = [AppConfig],
    global = true
)]
pub struct CoreModule;

/// A JSON response for NestForge routes. Its route macro wants an OpenAPI
/// schema for every return type and has no derive, so we describe ours here.
pub struct JsonBody(pub nestforge::serde_json::Value);

impl axum::response::IntoResponse for JsonBody {
    fn into_response(self) -> axum::response::Response {
        axum::Json(self.0).into_response()
    }
}

impl nestforge::OpenApiSchema for JsonBody {
    fn schema() -> nestforge::serde_json::Value {
        nestforge::serde_json::json!({ "type": "object" })
    }
}

/// A response we build ourselves (redirects, status-coded JSON, cookies).
pub struct RawResponse(pub axum::response::Response);

impl axum::response::IntoResponse for RawResponse {
    fn into_response(self) -> axum::response::Response {
        self.0
    }
}

impl nestforge::OpenApiSchema for RawResponse {
    fn schema() -> nestforge::serde_json::Value {
        nestforge::serde_json::json!({})
    }
}
