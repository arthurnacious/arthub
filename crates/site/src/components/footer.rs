use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::header::NAV;
use crate::config;

/// Year of the build (set by build.rs), for the copyright line.
const YEAR: &str = env!("ARTHUB_BUILD_YEAR");

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="site-footer">
            <div class="site-footer__cta">
                <p class="mono site-footer__kicker">"Got a project in mind?"</p>
                <A href="/contact" attr:class="site-footer__talk display">
                    "Let's talk"<span class="site-footer__arrow" aria-hidden="true">"→"</span>
                </A>
            </div>
            <div class="site-footer__grid">
                <nav aria-label="Footer">
                    <h2 class="mono site-footer__head">"Pages"</h2>
                    <ul class="site-footer__links">
                        <li><A href="/">"Home"</A></li>
                        {NAV.iter().map(|(_, label, href)| view! {
                            <li><A href=*href>{*label}</A></li>
                        }).collect_view()}
                    </ul>
                </nav>
                <div>
                    <h2 class="mono site-footer__head">"Play"</h2>
                    <ul class="site-footer__links">
                        <li><A href="/room">"The game room"</A></li>
                        <li><A href="/room?spot=pool">"Shoot some pool"</A></li>
                        <li><A href="/room?spot=arcade">"Play the arcade"</A></li>
                        <li><A href="/room?spot=jukebox">"Pick a record"</A></li>
                    </ul>
                </div>
                <div>
                    <h2 class="mono site-footer__head">"Colophon"</h2>
                    <p class="site-footer__note">
                        "Built in Rust, end to end: Leptos on the page, Bevy in the room, compiled to WebAssembly. Racked by hand."
                    </p>
                </div>
            </div>
            <div class="site-footer__bottom mono">
                <span>"© "{YEAR}" "{config::NAME}</span>
                <span>{config::ROLE}</span>
                <a href="#main" class="site-footer__top">"Back to top ↑"</a>
            </div>
        </footer>
    }
}
