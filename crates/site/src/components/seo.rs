use leptos::prelude::*;
use leptos_meta::{Link, Meta, Title};
use leptos_router::hooks::use_location;

use crate::config;

/// The public origin, e.g. `https://arthub.dev`, for absolute URLs in meta
/// tags. On the server it comes from `SITE_URL`; in the browser it's simply
/// where the page was loaded from (the same thing in production).
pub fn origin() -> String {
    #[cfg(feature = "ssr")]
    let o = std::env::var("SITE_URL").unwrap_or_else(|_| "http://127.0.0.1:3000".into());
    #[cfg(not(feature = "ssr"))]
    let o = web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .unwrap_or_default();
    o.trim_end_matches('/').to_string()
}

/// Title, description, canonical URL, Open Graph and Twitter card for a page.
/// `image` is a site path (`/media/…`); it defaults to the room still.
#[component]
pub fn Seo(
    /// Page title; empty for the home page ("ARTHUB").
    #[prop(into)]
    title: String,
    #[prop(into)] description: String,
    #[prop(optional, into)] image: Option<String>,
    /// `website` by default; case studies are `article`s.
    #[prop(optional)]
    kind: Option<&'static str>,
    /// Keep the page out of search results (404, unfinished pages).
    #[prop(optional)]
    noindex: bool,
) -> impl IntoView {
    let origin = origin();
    let url = format!("{origin}{}", use_location().pathname.get_untracked());
    let image = format!("{origin}{}", image.as_deref().unwrap_or("/og.jpg"));
    let full_title = if title.is_empty() {
        format!("ARTHUB — {}", config::NAME)
    } else {
        format!("{title} — ARTHUB")
    };
    view! {
        <Title text=title/>
        <Meta name="description" content=description.clone()/>
        <Link rel="canonical" href=url.clone()/>
        <Meta property="og:site_name" content="ARTHUB"/>
        <Meta property="og:type" content=kind.unwrap_or("website")/>
        <Meta property="og:title" content=full_title.clone()/>
        <Meta property="og:description" content=description.clone()/>
        <Meta property="og:url" content=url/>
        <Meta property="og:image" content=image.clone()/>
        <Meta name="twitter:card" content="summary_large_image"/>
        <Meta name="twitter:title" content=full_title/>
        <Meta name="twitter:description" content=description/>
        <Meta name="twitter:image" content=image/>
        {noindex.then(|| view! { <Meta name="robots" content="noindex"/> })}
    }
    // Erased: thirteen meta tags in every page's view type otherwise push the
    // server's debug info past what the linker can address.
    .into_any()
}
