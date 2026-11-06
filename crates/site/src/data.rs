//! The site's content.
//!
//! About is compiled in from `content/` (`static SITE`, by `build.rs`).
//! Projects, experience and skills live in the database: the server loads
//! them into a snapshot ([`set_site`]) and every page reads [`site`]. The
//! same snapshot rides along in each page as JSON ([`CATALOG_ID`]), and the
//! browser installs it before hydrating, so both sides render identical data
//! without a round trip. Until a snapshot is set (tests, the native room) the
//! compiled content stands in.

use std::sync::{Arc, RwLock};

use content::owned::Catalog;

include!(concat!(env!("OUT_DIR"), "/content.rs"));

/// Element id of the JSON snapshot in every page.
pub const CATALOG_ID: &str = "arthub-catalog";

struct Live {
    site: &'static content::Site,
    /// The snapshot as JSON, `</` escaped for embedding in a script tag.
    json: Arc<str>,
}

static LIVE: RwLock<Option<Live>> = RwLock::new(None);

pub fn site() -> &'static content::Site {
    LIVE.read()
        .ok()
        .and_then(|l| l.as_ref().map(|l| l.site))
        .unwrap_or(&SITE)
}

/// Install a new snapshot (the server after loading or editing content; the
/// browser at startup). Each call leaks one snapshot, by design.
pub fn set_site(catalog: Catalog) {
    let json: Arc<str> = serde_json::to_string(&catalog)
        .unwrap_or_else(|_| "null".into())
        .replace("</", "<\\/")
        .into();
    let site = catalog.leak(SITE.about);
    if let Ok(mut l) = LIVE.write() {
        *l = Some(Live { site, json });
    }
}

/// The current snapshot as JSON, for the page shell.
pub fn catalog_json() -> Arc<str> {
    LIVE.read()
        .ok()
        .and_then(|l| l.as_ref().map(|l| l.json.clone()))
        .unwrap_or_else(|| {
            serde_json::to_string(&Catalog::from_static(&SITE))
                .unwrap_or_else(|_| "null".into())
                .replace("</", "<\\/")
                .into()
        })
}

/// Browser: read the snapshot the server embedded, before hydrating.
#[cfg(feature = "hydrate")]
pub fn install_from_page() {
    let json = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(CATALOG_ID))
        .and_then(|e| e.text_content());
    if let Some(Ok(catalog)) = json.map(|j| serde_json::from_str::<Catalog>(&j)) {
        set_site(catalog);
    }
}
