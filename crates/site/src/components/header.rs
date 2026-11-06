use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_location;

pub const NAV: &[(&str, &str, &str)] = &[
    ("01", "Work", "/work"),
    ("02", "About", "/about"),
    ("03", "Skills", "/skills"),
    ("04", "Experience", "/experience"),
    ("05", "Lab", "/lab"),
    ("06", "Contact", "/contact"),
];

/// The site header. Wide screens get the nav inline; below 860px it folds
/// into a Menu button that opens a full-screen menu (closed by Escape, by
/// picking a link, or by any navigation).
#[component]
pub fn Header() -> impl IntoView {
    let open = RwSignal::new(false);
    let toggle = NodeRef::<leptos::html::Button>::new();

    // Any route change closes it (covers links, back/forward).
    let location = use_location();
    Effect::new(move |_| {
        location.pathname.track();
        open.set(false);
    });

    #[cfg(feature = "hydrate")]
    {
        // Lock the page behind the menu; on close, focus returns to the toggle.
        Effect::new(move |was: Option<bool>| {
            let now = open.get();
            if let Some(root) = document().document_element() {
                let _ = root.class_list().toggle_with_force("menu-open", now);
            }
            // Keyboard focus stays on the toggle; Tab goes on into the menu
            // and nothing behind it can take focus while it's open.
            for sel in ["main", ".site-footer", ".skip-link"] {
                if let Ok(Some(el)) = document().query_selector(sel) {
                    let _ = el.toggle_attribute_with_force("inert", now);
                }
            }
            if !now
                && was == Some(true)
                && let Some(b) = toggle.get()
            {
                let _ = b.focus();
            }
            now
        });
        let esc = window_event_listener(leptos::ev::keydown, move |e| {
            if e.key() == "Escape" && open.get_untracked() {
                open.set(false);
            }
        });
        on_cleanup(move || esc.remove());
    }

    view! {
        <header class="site-header">
            <A href="/" attr:class="site-header__mark" attr:aria-label="Arthub, home">"ARTHUB"</A>
            <nav class="site-header__nav" aria-label="Primary">
                <ul class="site-nav">
                    {NAV.iter().map(|(n, label, href)| view! {
                        <li>
                            <A href=*href attr:class="site-nav__link">
                                <span class="mono site-nav__num">{*n}</span>
                                {*label}
                            </A>
                        </li>
                    }).collect_view()}
                </ul>
            </nav>
            <A href="/room" attr:class="btn btn--small site-header__room" attr:data-cursor="link">"Game Room"</A>
            <button type="button" class="site-header__toggle mono" node_ref=toggle
                aria-controls="site-menu" aria-expanded=move || open.get().to_string()
                on:click=move |_| open.update(|o| *o = !*o)>
                <span>{move || if open.get() { "Close" } else { "Menu" }}</span>
                <span class="site-header__burger" aria-hidden="true"><i></i><i></i></span>
            </button>
        </header>
        // Outside the header: its backdrop blur would trap a fixed overlay.
        <div id="site-menu" class="site-menu" class:site-menu--open=move || open.get()
            inert=move || !open.get() aria-hidden=move || (!open.get()).to_string()>
            <nav aria-label="Menu">
                <ol class="site-menu__list">
                    {NAV.iter().enumerate().map(|(i, (n, label, href))| view! {
                        <li style=format!("--i: {i}")>
                            <A href=*href attr:class="site-menu__link display">
                                <span class="mono site-menu__num">{*n}</span>
                                {*label}
                            </A>
                        </li>
                    }).collect_view()}
                </ol>
            </nav>
            <A href="/room" attr:class="btn btn--accent site-menu__room">"Enter the game room"</A>
        </div>
    }
}
