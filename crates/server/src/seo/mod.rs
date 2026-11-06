//! SeoModule: `robots.txt` and `sitemap.xml`, built from the same content as
//! the pages. Absolute URLs use `SITE_URL` when set, otherwise the request's
//! host (fine behind a proxy that forwards `Host`).

use axum::http::{HeaderMap, header};
use axum::response::IntoResponse;
use nestforge::{HttpException, Inject, controller, module, routes};

use crate::content::{ContentModule, ContentRepository};
use crate::core::{AppConfig, RawResponse};

/// Indexable pages that aren't generated from content. The lab is left out
/// until it has something in it (it's `noindex` for now).
const PAGES: &[&str] = &[
    "/",
    "/work",
    "/about",
    "/skills",
    "/experience",
    "/contact",
    "/room",
];

fn base(cfg: &AppConfig, headers: &HeaderMap) -> String {
    crate::core::public_origin(cfg, headers)
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[controller("")]
#[derive(Default)]
pub struct SeoController;

#[routes]
impl SeoController {
    #[nestforge::get("/robots.txt")]
    async fn robots(
        cfg: Inject<AppConfig>,
        headers: HeaderMap,
    ) -> Result<RawResponse, HttpException> {
        let body = format!(
            "User-agent: *\nDisallow: /api/\nDisallow: /lab\n\nSitemap: {}/sitemap.xml\n",
            base(&cfg, &headers)
        );
        Ok(RawResponse(
            ([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], body).into_response(),
        ))
    }

    #[nestforge::get("/sitemap.xml")]
    async fn sitemap(
        cfg: Inject<AppConfig>,
        repo: Inject<ContentRepository>,
        headers: HeaderMap,
    ) -> Result<RawResponse, HttpException> {
        let base = base(&cfg, &headers);
        let paths = PAGES.iter().map(|p| p.to_string()).chain(
            repo.site()
                .projects
                .iter()
                .map(|p| format!("/work/{}", p.slug)),
        );
        let mut xml = String::from(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n",
        );
        for p in paths {
            xml.push_str(&format!(
                "  <url><loc>{}</loc></url>\n",
                xml_escape(&format!("{base}{p}"))
            ));
        }
        xml.push_str("</urlset>\n");
        Ok(RawResponse(
            (
                [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
                xml,
            )
                .into_response(),
        ))
    }
}

#[module(controllers = [SeoController], imports = [ContentModule])]
pub struct SeoModule;
