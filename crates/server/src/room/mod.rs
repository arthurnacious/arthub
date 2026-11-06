//! RoomModule: the game room's JSON API and its asset serving.
//!
//! * `GET /api/room/featured`: the featured projects the arcade cabinets show.
//! * `/room/bundle/*`: the room's WASM + JS, served as the pre-compressed
//!   `.br`/`.gz` files `scripts/build-room.sh` writes (several MB, so this
//!   matters), with `Content-Type: application/wasm` for the module.

use axum::Router;
use nestforge::serde_json::json;
use nestforge::{Container, HttpException, Inject, controller, module, routes};
use tower_http::services::ServeDir;

use crate::content::{ContentModule, ContentRepository};
use crate::core::JsonBody;

#[controller("/api/room")]
#[derive(Default)]
pub struct RoomController;

#[routes]
impl RoomController {
    #[nestforge::get("/featured")]
    async fn featured(repo: Inject<ContentRepository>) -> Result<JsonBody, HttpException> {
        Ok(JsonBody(json!(
            repo.site()
                .featured()
                .map(|p| json!({
                    "slug": p.slug,
                    "title": p.title,
                    "accent": format!("#{:02x}{:02x}{:02x}", p.accent[0], p.accent[1], p.accent[2]),
                    "cover": p.cover_asset,
                    "url": format!("/work/{}", p.slug),
                }))
                .collect::<Vec<_>>()
        )))
    }
}

/// Static serving for the room bundle under the built site root (merged into
/// the app by `app::build`).
pub fn bundle_router(site_root: &str) -> Router<Container> {
    let dir = std::path::Path::new(site_root).join("room/bundle");
    Router::new().nest_service(
        "/room/bundle",
        ServeDir::new(dir).precompressed_br().precompressed_gzip(),
    )
}

#[module(controllers = [RoomController], imports = [ContentModule])]
pub struct RoomModule;
