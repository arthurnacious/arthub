//! The application: NestForge modules for the API, with the Leptos SSR app
//! (pages, server functions, static files) mounted on the same Axum router.
//!
//! NestForge owns `/api/*` (except server functions), `/healthz` and
//! `/room/bundle/*`; Leptos owns every page and the static file fallback.

use axum::Router;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use nestforge::{NestForgeFactory, module};
use site::app::{App, shell};

use crate::admin::AdminModule;
use crate::auth::AuthModule;
use crate::contact::{ContactModule, ContactService, token_on_contact_page};
use crate::content::ContentModule;
use crate::core::CoreModule;
use crate::room::{RoomModule, bundle_router};
use crate::seo::SeoModule;

#[module(imports = [CoreModule, ContentModule, ContactModule, RoomModule, SeoModule, AuthModule, AdminModule])]
pub struct AppModule;

/// Build the whole app. Fails (and so does startup) if a module can't be
/// constructed, e.g. a production server with no email provider configured.
pub fn build(options: LeptosOptions, store: store::Store) -> anyhow::Result<Router> {
    let factory = NestForgeFactory::<AppModule>::create()?;
    let container = factory.container().clone();
    container.register(crate::core::Db(store.clone()))?;
    let contact = container.resolve::<ContactService>()?;
    let _ = contact.store.set(store.clone());
    let cfg = container.resolve::<crate::core::AppConfig>()?;

    // What the dashboard's server functions run against.
    let backend = site::admin::Backend {
        store,
        mailer: contact.mailer.clone(),
        reply_to: contact.mailer.config().to.clone(),
    };
    let with_backend = {
        let backend = backend.clone();
        move || provide_context(backend.clone())
    };

    let uploads = tower_http::services::ServeDir::new(cfg.data_dir.join("uploads"));
    let api = factory
        .merge_router(bundle_router(&options.site_root))
        .merge_router(Router::new().nest_service("/media/uploads", uploads))
        .into_router()
        .layer(axum::extract::DefaultBodyLimit::max(
            crate::admin::MAX_UPLOAD,
        ));

    let routes = generate_route_list(App);
    let pages: Router = Router::new()
        // Leptos server functions (the dashboard's).
        .route(
            "/api/{*fn_name}",
            axum::routing::post({
                let with_backend = with_backend.clone();
                move |req: axum::extract::Request| {
                    leptos_axum::handle_server_fns_with_context(with_backend.clone(), req)
                }
            }),
        )
        .leptos_routes_with_context(&options, routes, with_backend, {
            let options = options.clone();
            move || shell(options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(options);

    Ok(api
        .merge(pages)
        // The contact page hands out a time-on-form token on every full page
        // load (it's a Leptos route, so this wraps the whole app).
        .layer(axum::middleware::from_fn_with_state(
            contact,
            token_on_contact_page,
        )))
}

#[cfg(test)]
mod tests {
    //! Every module's routes, through the fully assembled app (NestForge
    //! modules + the Leptos SSR router), as a browser would hit them.

    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode, header};
    use nestforge::serde_json::Value;
    use tower::ServiceExt;

    use super::*;

    /// A throwaway site root holding a fake room bundle (plain + Brotli).
    fn site_root() -> String {
        let root = std::env::temp_dir().join(format!("arthub-app-test-{}", std::process::id()));
        let bundle = root.join("room/bundle");
        std::fs::create_dir_all(&bundle).unwrap();
        std::fs::write(bundle.join("game_room_bg.wasm"), b"\0asm\x01\0\0\0plain").unwrap();
        std::fs::write(bundle.join("game_room_bg.wasm.br"), b"pretend-brotli").unwrap();
        root.to_string_lossy().into_owned()
    }

    /// A fresh in-memory database, seeded from `content/` like a first run.
    async fn store() -> store::Store {
        let s = store::Store::open(":memory:").await.unwrap();
        s.seed_if_empty(&content::owned::Catalog::from_static(&site::data::SITE))
            .await
            .unwrap();
        s
    }

    async fn app() -> Router {
        let options = LeptosOptions::builder()
            .output_name("arthub")
            .site_root(site_root())
            .build();
        build(options, store().await).expect("app builds")
    }

    async fn send(req: Request<Body>) -> (StatusCode, axum::http::HeaderMap, String) {
        let res = app().await.oneshot(req).await.unwrap();
        let (status, headers) = (res.status(), res.headers().clone());
        let body = String::from_utf8_lossy(&to_bytes(res.into_body(), 1 << 22).await.unwrap())
            .into_owned();
        (status, headers, body)
    }

    async fn get(uri: &str) -> (StatusCode, axum::http::HeaderMap, String) {
        send(Request::get(uri).body(Body::empty()).unwrap()).await
    }

    async fn json(uri: &str) -> Value {
        let (status, _, body) = get(uri).await;
        assert_eq!(status, StatusCode::OK, "{uri}: {body}");
        nestforge::serde_json::from_str(&body).expect("json")
    }

    // ---- CoreModule
    #[tokio::test]
    async fn core_health() {
        let (status, _, body) = get("/healthz").await;
        assert_eq!((status, body.as_str()), (StatusCode::OK, "ok"));
    }

    // ---- ContentModule
    #[tokio::test]
    async fn content_projects_and_detail() {
        let list = json("/api/content/projects").await;
        let slugs: Vec<_> = list
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["slug"].as_str().unwrap().to_string())
            .collect();
        assert!(slugs.contains(&"neon-city".to_string()), "{slugs:?}");
        let one = json("/api/content/projects/neon-city").await;
        assert_eq!(one["title"], "Neon City");
        assert!(one["body_html"].as_str().unwrap().contains("<h2>"));
        assert!(one["gallery"].as_array().is_some_and(|g| !g.is_empty()));
    }

    #[tokio::test]
    async fn content_unknown_project_is_404() {
        let (status, _, body) = get("/api/content/projects/nope").await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    }

    #[tokio::test]
    async fn content_skills_experience_about() {
        assert!(
            json("/api/content/skills")
                .await
                .as_array()
                .is_some_and(|g| !g.is_empty())
        );
        assert!(
            json("/api/content/experience").await["roles"]
                .as_array()
                .is_some_and(|r| !r.is_empty())
        );
        assert!(json("/api/content/about").await["name"].is_string());
    }

    // ---- RoomModule
    #[tokio::test]
    async fn room_featured() {
        let f = json("/api/room/featured").await;
        let first = &f.as_array().unwrap()[0];
        assert!(first["cover"].as_str().unwrap().starts_with("media/work/"));
        assert!(first["url"].as_str().unwrap().starts_with("/work/"));
    }

    #[tokio::test]
    async fn room_bundle_is_wasm_and_precompressed() {
        let (status, h, _) = send(
            Request::get("/room/bundle/game_room_bg.wasm")
                .header(header::ACCEPT_ENCODING, "br")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(h[header::CONTENT_TYPE], "application/wasm");
        assert_eq!(h[header::CONTENT_ENCODING], "br");
        let (_, h, body) = get("/room/bundle/game_room_bg.wasm").await;
        assert_eq!(h[header::CONTENT_TYPE], "application/wasm");
        assert!(h.get(header::CONTENT_ENCODING).is_none());
        assert!(body.ends_with("plain"));
    }

    // ---- ContactModule (full app; the module's own tests cover the rules)
    #[tokio::test]
    async fn contact_submit_and_token() {
        let (status, _, body) = send(
            Request::post("/api/contact")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::ACCEPT, "application/json")
                .body(Body::from("name=A&email=a%40example.com&project_type=Website&budget=Not+sure+yet&message=Hello+there+friend&website="))
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let (status, h, _) = get("/api/contact/token").await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        assert!(
            h[header::SET_COOKIE]
                .to_str()
                .unwrap()
                .starts_with("arthub_ct=")
        );
    }

    #[tokio::test]
    async fn contact_submission_is_kept_for_the_dashboard() {
        let db = store().await;
        let options = LeptosOptions::builder()
            .output_name("arthub")
            .site_root(site_root())
            .build();
        let app = build(options, db.clone()).unwrap();
        let res = app
            .oneshot(
                Request::post("/api/contact")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header(header::ACCEPT, "application/json")
                    .body(Body::from("name=Thabo&email=t%40example.com&project_type=Website&budget=Not+sure+yet&message=Hello+there+friend&website="))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let msgs = db.messages(None).await.unwrap();
        assert_eq!(msgs.len(), 1);
        assert_eq!(
            (msgs[0].name.as_str(), msgs[0].status.as_str()),
            ("Thabo", "new")
        );
    }

    #[tokio::test]
    async fn contact_page_hands_out_a_token() {
        let (status, h, body) = get("/contact").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("action=\"/api/contact\""));
        assert!(
            h[header::SET_COOKIE]
                .to_str()
                .unwrap()
                .starts_with("arthub_ct=")
        );
    }

    // ---- Leptos pages, unchanged
    #[tokio::test]
    async fn pages_render_server_side() {
        let (status, _, body) = get("/").await;
        assert_eq!(status, StatusCode::OK);
        assert!(body.contains("<title>ARTHUB</title>"));
        let (_, _, body) = get("/work/neon-city").await;
        assert!(body.contains("<title>Neon City — ARTHUB</title>"));
        let (status, _, _) = get("/definitely-not-a-page").await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    // ---- SeoModule + page meta
    #[tokio::test]
    async fn seo_robots_sitemap_and_meta() {
        let (status, h, body) = get("/robots.txt").await;
        assert_eq!(status, StatusCode::OK);
        assert!(
            h[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .starts_with("text/plain")
        );
        assert!(body.contains("Sitemap: http") && body.ends_with("/sitemap.xml\n"));

        let (status, h, body) = get("/sitemap.xml").await;
        assert_eq!(status, StatusCode::OK);
        assert!(h[header::CONTENT_TYPE].to_str().unwrap().contains("xml"));
        assert!(body.contains("/work/neon-city</loc>"));
        assert!(body.contains("/about</loc>"));
        assert!(!body.contains("/lab<"));

        let (_, _, body) = get("/").await;
        assert!(body.contains("rel=\"canonical\""));
        assert!(body.contains("og:image") && body.contains("/og.jpg"));
        assert!(body.contains("application/ld+json"));
        let (_, _, body) = get("/work/neon-city").await;
        assert!(body.contains("content=\"article\""));
        let (_, _, body) = get("/definitely-not-a-page").await;
        assert!(body.contains("noindex"));
    }

    // ---- AuthModule + AdminModule + the dashboard's server functions
    #[tokio::test]
    async fn dashboard_sign_in_server_functions_and_uploads() {
        use leptos::server_fn::ServerFn;

        let db = store().await;
        let options = LeptosOptions::builder()
            .output_name("arthub")
            .site_root(site_root())
            .build();
        let app = build(options, db.clone()).unwrap();
        let call = |req: Request<Body>| {
            let app = app.clone();
            async move {
                let res = app.oneshot(req).await.unwrap();
                let (status, headers) = (res.status(), res.headers().clone());
                let body = to_bytes(res.into_body(), 1 << 22).await.unwrap();
                (status, headers, String::from_utf8_lossy(&body).into_owned())
            }
        };
        let list = <site::admin::api::ListMessages as ServerFn>::PATH;
        let fn_call = |cookie: Option<&str>| {
            let mut r = Request::post(list)
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::ACCEPT, "application/json");
            if let Some(c) = cookie {
                r = r.header(header::COOKIE, c);
            }
            r.body(Body::from("filter=open")).unwrap()
        };

        // Signed out: refused.
        let (status, _, body) = call(fn_call(None)).await;
        assert!(!status.is_success(), "{status} {body}");

        // A login link works once and starts a session.
        let token = store::secret::new_token();
        db.add_login_token(&store::secret::hash(&token), store::now() + 600)
            .await
            .unwrap();
        let (status, h, _) = call(
            Request::get(format!("/api/admin/auth?token={token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(status, StatusCode::SEE_OTHER);
        assert_eq!(h[header::LOCATION], "/admin");
        let set = h[header::SET_COOKIE].to_str().unwrap().to_string();
        assert!(
            set.contains("HttpOnly") && set.starts_with("arthub_admin="),
            "{set}"
        );
        let cookie = set.split(';').next().unwrap().to_string();
        let (_, h, _) = call(
            Request::get(format!("/api/admin/auth?token={token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(h[header::LOCATION], "/admin/login?error=expired");

        // Signed in: the server function answers.
        db.add_message("Ann", "ann@example.com", "Website", "R10k", "Hi")
            .await
            .unwrap();
        let (status, _, body) = call(fn_call(Some(&cookie))).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        assert!(body.contains("ann@example.com"), "{body}");

        // Uploads need the session and a real image.
        let png: &[u8] = &[
            0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 0x0d, b'I', b'H', b'D', b'R',
            0, 0, 0, 3, 0, 0, 0, 2, 8, 6, 0, 0, 0,
        ];
        let upload = |cookie: Option<&str>| {
            let mut body = Vec::new();
            body.extend_from_slice(b"--XX\r\nContent-Disposition: form-data; name=\"file\"; filename=\"a.png\"\r\nContent-Type: image/png\r\n\r\n");
            body.extend_from_slice(png);
            body.extend_from_slice(b"\r\n--XX--\r\n");
            let mut r = Request::post("/api/admin/upload")
                .header(header::CONTENT_TYPE, "multipart/form-data; boundary=XX");
            if let Some(c) = cookie {
                r = r.header(header::COOKIE, c);
            }
            r.body(Body::from(body)).unwrap()
        };
        let (status, _, _) = call(upload(None)).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        let (status, _, body) = call(upload(Some(&cookie))).await;
        assert_eq!(status, StatusCode::OK, "{body}");
        let v: Value = nestforge::serde_json::from_str(&body).unwrap();
        assert_eq!(
            (v["width"].as_u64(), v["height"].as_u64()),
            (Some(3), Some(2))
        );
        let src = v["src"].as_str().unwrap().to_string();
        let (status, _, _) = call(Request::get(&src).body(Body::empty()).unwrap()).await;
        assert_eq!(status, StatusCode::OK, "{src} is served");
        let _ = std::fs::remove_file(
            std::path::Path::new("data/uploads").join(src.trim_start_matches("/media/uploads/")),
        );

        // Signing out ends it.
        let (_, h, _) = call(
            Request::post("/api/admin/logout")
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(h[header::LOCATION], "/admin/login");
        let (status, _, _) = call(fn_call(Some(&cookie))).await;
        assert!(!status.is_success());
    }

    // ---- Every module constructs
    #[tokio::test]
    async fn all_modules_construct() {
        let f = NestForgeFactory::<AppModule>::create().expect("modules");
        assert!(
            f.container()
                .resolve::<crate::content::ContentRepository>()
                .is_ok()
        );
        assert!(f.container().resolve::<ContactService>().is_ok());
    }
}
