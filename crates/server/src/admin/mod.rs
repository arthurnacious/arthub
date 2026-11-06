//! AdminModule: the dashboard's file uploads. The dashboard itself is Leptos
//! pages plus server functions (`site::admin`); uploads are a plain multipart
//! route because server functions don't stream files.
//!
//! * `POST /api/admin/upload` (multipart `file`, signed in): an image (jpg,
//!   png, webp, gif) or video (mp4, webm), saved to `DATA_DIR/uploads/` and
//!   served at `/media/uploads/…`. Answers with its URL and pixel size.

use axum::extract::{FromRequest, Multipart, Request};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use nestforge::serde_json::json;
use nestforge::{Inject, controller, module, routes};
use store::secret;

use crate::core::{AppConfig, Db, JsonBody, RawResponse};

/// Biggest upload accepted (a short preview video fits).
pub const MAX_UPLOAD: usize = 60 * 1024 * 1024;

const IMAGES: [&str; 5] = ["jpg", "jpeg", "png", "webp", "gif"];
const VIDEOS: [&str; 2] = ["mp4", "webm"];

pub async fn signed_in(db: &Db, headers: &HeaderMap) -> bool {
    match site::admin::session_from(headers) {
        Some(s) => db.0.session_valid(&secret::hash(&s)).await.unwrap_or(false),
        None => false,
    }
}

fn refuse(status: StatusCode, msg: &str) -> RawResponse {
    RawResponse((status, msg.to_string()).into_response())
}

#[controller("/api/admin")]
#[derive(Default)]
pub struct AdminController;

#[routes]
impl AdminController {
    #[nestforge::post("/upload")]
    async fn upload(cfg: Inject<AppConfig>, db: Inject<Db>, req: Request) -> RawResponse {
        if !signed_in(&db, req.headers()).await {
            return refuse(StatusCode::UNAUTHORIZED, "Signed out. Sign in again.");
        }
        let Ok(mut form) = Multipart::from_request(req, &()).await else {
            return refuse(StatusCode::BAD_REQUEST, "Expected a file upload.");
        };
        while let Ok(Some(field)) = form.next_field().await {
            if field.name() != Some("file") {
                continue;
            }
            let ext = field
                .file_name()
                .and_then(|n| n.rsplit_once('.'))
                .map(|(_, e)| e.to_ascii_lowercase())
                .unwrap_or_default();
            let video = VIDEOS.contains(&ext.as_str());
            if !video && !IMAGES.contains(&ext.as_str()) {
                return refuse(
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    "Use a JPG, PNG, WebP or GIF image, or an MP4 or WebM video.",
                );
            }
            let bytes = match field.bytes().await {
                Ok(b) => b,
                Err(_) => {
                    return refuse(
                        StatusCode::PAYLOAD_TOO_LARGE,
                        "That file is too big (60 MB at most).",
                    );
                }
            };
            let (width, height) = if video {
                (0, 0)
            } else {
                match imagesize::blob_size(&bytes) {
                    Ok(s) => (s.width as u32, s.height as u32),
                    Err(_) => {
                        return refuse(
                            StatusCode::UNPROCESSABLE_ENTITY,
                            "That image couldn't be read.",
                        );
                    }
                }
            };
            let dir = cfg.data_dir.join("uploads");
            let name = format!("{}.{ext}", &secret::new_token()[..20]);
            let saved = async {
                tokio::fs::create_dir_all(&dir).await?;
                tokio::fs::write(dir.join(&name), &bytes).await
            }
            .await;
            if let Err(e) = saved {
                tracing::error!("admin: couldn't save an upload: {e}");
                return refuse(StatusCode::INTERNAL_SERVER_ERROR, "Couldn't save the file.");
            }
            tracing::info!("admin: uploaded {name} ({} KB)", bytes.len() / 1024);
            let src = format!("/media/uploads/{name}");
            return RawResponse(
                JsonBody(json!({
                    "asset": src.trim_start_matches('/'),
                    "src": src,
                    "width": width,
                    "height": height,
                    "video": video,
                }))
                .into_response(),
            );
        }
        refuse(StatusCode::BAD_REQUEST, "No file in that upload.")
    }
}

#[module(controllers = [AdminController])]
pub struct AdminModule;
