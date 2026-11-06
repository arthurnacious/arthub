use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{ParentRoute, Route, Router, Routes},
    hooks::use_location,
    path,
};

use crate::admin::pages::{
    AdminExperience, AdminLogin, AdminMessages, AdminProjectEdit, AdminProjects, AdminShell,
    AdminSkills, AdminThread,
};
use crate::components::{
    cursor::{Cursor, CursorOverride},
    footer::Footer,
    header::Header,
};
use crate::pages::*;
use crate::room::RoomBridge;

/// The HTML document rendered on the server. Fonts are preloaded here so the
/// oversized headlines don't reflow when the display face arrives.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <meta name="theme-color" content="#0a0a0a"/>
                // Lets CSS hold back effects that need the app (the hero's
                // ASCII portrait decodes in rather than flashing).
                <script inner_html="document.documentElement.classList.add('js')"></script>
                <link rel="icon" href="/favicon.svg" r#type="image/svg+xml"/>
                <link rel="preload" href="/fonts/archivo-var.woff2" r#as="font" r#type="font/woff2" crossorigin=""/>
                <link rel="preload" href="/fonts/jetbrains-mono-var.woff2" r#as="font" r#type="font/woff2" crossorigin=""/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
                // The content snapshot the browser hydrates with (see data.rs).
                <script type="application/json" id=crate::data::CATALOG_ID inner_html=crate::data::catalog_json().to_string()></script>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    provide_context(CursorOverride(RwSignal::new(None)));

    view! {
        <Stylesheet id="leptos" href="/pkg/arthub.css"/>
        <Title formatter=|t: String| if t.is_empty() { "ARTHUB".into() } else { format!("{t} — ARTHUB") }/>

        <a class="skip-link" href="#main">"Skip to content"</a>
        <div class="grain" aria-hidden="true"></div>
        <Cursor/>

        <Router>
            <RoomBridge/>
            <PublicOnly><Header/></PublicOnly>
            <main id="main" tabindex="-1">
                <Routes fallback=NotFound>
                    <Route path=StaticSegment("") view=Home/>
                    <Route path=path!("/work") view=WorkIndex/>
                    <Route path=path!("/work/:slug") view=CaseStudy/>
                    <Route path=path!("/about") view=About/>
                    <Route path=path!("/skills") view=Skills/>
                    <Route path=path!("/experience") view=Experience/>
                    <Route path=path!("/lab") view=Lab/>
                    <Route path=path!("/contact") view=Contact/>
                    <Route path=path!("/room") view=Room/>
                    <Route path=path!("/admin/login") view=AdminLogin/>
                    <ParentRoute path=path!("/admin") view=AdminShell>
                        <Route path=path!("") view=AdminMessages/>
                        <Route path=path!("messages") view=AdminMessages/>
                        <Route path=path!("messages/:id") view=AdminThread/>
                        <Route path=path!("projects") view=AdminProjects/>
                        <Route path=path!("projects/new") view=AdminProjectEdit/>
                        <Route path=path!("projects/:id") view=AdminProjectEdit/>
                        <Route path=path!("experience") view=AdminExperience/>
                        <Route path=path!("skills") view=AdminSkills/>
                    </ParentRoute>
                </Routes>
            </main>
            <PublicOnly><Footer/></PublicOnly>
        </Router>
    }
}

/// The public header and footer; the dashboard has its own chrome.
#[component]
fn PublicOnly(children: ChildrenFn) -> impl IntoView {
    let location = use_location();
    let public = move || !location.pathname.read().starts_with("/admin");
    view! { <Show when=public>{children()}</Show> }
}
