//! Pages still waiting on their own phase (lab, contact) and the 404.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::{section::PageHeader, seo::Seo};

#[component]
pub fn Lab() -> impl IntoView {
    view! {
        <Seo title="Lab" description="Side projects, shader tests and experiments." noindex=true/>
        <PageHeader num="05" label="Lab" title="Lab"
            lede="Side projects, shader tests and things that probably shouldn't work."/>
        <div class="placeholder mono" data-cursor="drag">"Masonry wall · phase 2"</div>
    }
}

#[component]
pub fn NotFound() -> impl IntoView {
    #[cfg(feature = "ssr")]
    if let Some(resp) = use_context::<leptos_axum::ResponseOptions>() {
        resp.set_status(http::StatusCode::NOT_FOUND);
    }
    view! {
        <Seo title="Scratch" description="Page not found." noindex=true/>
        <PageHeader num="404" label="Not Found" title="Scratch."
            lede="That shot missed every pocket."/>
        <A href="/" attr:class="btn">"Back to the table"</A>
    }
}
