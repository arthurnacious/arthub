use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::{
    ascii::AsciiPortrait,
    marquee::Marquee,
    seo::{Seo, origin},
};
use crate::config;
use crate::data::site;
use crate::pages::ProjectCard;

/// How many featured projects the home masonry shows.
const FEATURED: usize = 4;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        <Seo title="" description=format!("{} — {}. {}", config::NAME, config::ROLE, config::TAGLINE)/>
        <script type="application/ld+json" inner_html=person_json()></script>
        // Who/what/where, the line in big type, the two ways in, and the
        // ASCII portrait. The room itself is a click away.
        <section class="hero" aria-labelledby="hero-title">
            <div class="hero__meta mono">
                <span>{config::NAME}</span>
                <span>{config::ROLE}</span>
                <span>{site().about.location}</span>
            </div>
            <div class="hero__main">
                <div class="hero__copy">
                    <h1 id="hero-title" class="display hero__title">{headline()}</h1>
                    <p class="hero__disciplines mono">{config::DISCIPLINES.join(" · ")}</p>
                    <div class="hero__ctas">
                        <A href="/room" attr:class="btn btn--accent">"Enter the game room"</A>
                        <A href="/work" attr:class="btn">"See my work"</A>
                    </div>
                </div>
                <figure class="hero__portrait">
                    <AsciiPortrait label=format!("Portrait of {} in ASCII characters", config::NAME)/>
                    <figcaption class="mono">{format!("{} — {}", config::NAME, site().about.location)}</figcaption>
                </figure>
            </div>
        </section>

        <Marquee items=config::DISCIPLINES/>

        <section class="home-section" aria-labelledby="work-title">
            <div class="home-section__head">
                <h2 id="work-title" class="display home-section__title">"Work worth a rematch."</h2>
                <A href="/work" attr:class="link-arrow mono">{format!("All work ({})", site().projects.len())}</A>
            </div>
            <ol class="featured">
                {site().featured().take(FEATURED).enumerate().map(|(i, p)| view! {
                    <li><ProjectCard project=p index=i/></li>
                }).collect_view()}
            </ol>
        </section>
    }
}

/// The tagline as the headline, its second half dimmed ("I build fast
/// things for the web / and the metal underneath it."), each word rising in.
fn headline() -> impl IntoView {
    let (lead, rest) = match config::TAGLINE.split_once(" and ") {
        Some((a, b)) => (a.to_string(), format!("and {b}")),
        None => (config::TAGLINE.to_string(), String::new()),
    };
    let mut i = 0;
    let mut words = |text: &str, class: &'static str| {
        text.split_whitespace()
            .map(|w| {
                let v =
                    view! { <span class=class style=format!("--i: {i}")>{w.to_string()}</span>" " };
                i += 1;
                v
            })
            .collect_view()
    };
    let lead = words(&lead, "hero__word");
    let rest = words(&rest, "hero__word hero__word--dim");
    view! { {lead}{rest} }
}

/// schema.org `Person`, so search engines can tie the site to its owner.
fn person_json() -> String {
    let esc = |v: &str| {
        v.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('<', "\\u003c")
    };
    format!(
        r#"{{"@context":"https://schema.org","@type":"Person","name":"{}","jobTitle":"{}","description":"{}","url":"{}/"}}"#,
        esc(config::NAME),
        esc(config::ROLE),
        esc(config::TAGLINE),
        esc(&origin()),
    )
}
