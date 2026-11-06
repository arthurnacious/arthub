//! `/about`, `/skills` and `/experience`, straight from `content/`.

use content::Level;
use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::section::{PageHeader, SectionLabel};
use crate::components::seo::Seo;
use crate::data::site;

fn placeholder(on: bool) -> impl IntoView {
    on.then(
        || view! { <span class="badge mono" title="Replace this in content/">"Placeholder"</span> },
    )
}

/// Chalk tally marks: up to four strokes, the fifth slashes the gate.
#[component]
fn Tally(n: u8, label: &'static str) -> impl IntoView {
    view! {
        <span class="tally" role="img" aria-label=label>
            {(0..n.min(4)).map(|_| view! { <i></i> }).collect_view()}
            {(n >= 5).then(|| view! { <i class="tally__slash"></i> })}
        </span>
    }
}

#[component]
pub fn About() -> impl IntoView {
    let a = site().about;
    view! {
        <Seo title="About" description=a.intro/>
        <PageHeader num="02" label="About" title="About" lede=a.intro/>
        {placeholder(a.placeholder)}

        <div class="about">
            {a.portrait.map(|p| view! {
                <figure class="about__portrait">
                    <img src=p.src alt=p.alt width=p.width height=p.height decoding="async"/>
                    <figcaption class="mono">{a.name}" · "{a.location}</figcaption>
                </figure>
            })}
            <div class="about__text">
                <p class="mono about__role">{a.role}</p>
                <div class="prose" inner_html=a.body_html></div>
            </div>
        </div>

        {(!a.stats.is_empty()).then(|| view! {
            <ul class="stats" aria-label="In numbers">
                {a.stats.iter().map(|s| view! {
                    <li><span class="display stats__value">{s.value}</span><span class="mono">{s.label}</span></li>
                }).collect_view()}
            </ul>
        })}

        {(!a.currently.is_empty()).then(|| view! {
            <section class="currently">
                <SectionLabel num="03" text="Currently"/>
                <dl>
                    {a.currently.iter().map(|c| view! {
                        <div><dt class="mono">{c.label}</dt><dd>{c.text}</dd></div>
                    }).collect_view()}
                </dl>
            </section>
        })}

        <div class="page-cta">
            <A href="/experience" attr:class="btn">"Experience"</A>
            <A href="/skills" attr:class="btn">"Skills"</A>
        </div>
    }
}

#[component]
pub fn Skills() -> impl IntoView {
    let levels = [Level::Daily, Level::Confident, Level::Learning];
    view! {
        <Seo title="Skills" description="Languages, tools and platforms, tallied on the chalkboard."/>
        <PageHeader num="03" label="Skills" title="Skills"
            lede="Tallied on the chalkboard. No percentage bars, promise."/>

        <ul class="tally-legend mono" aria-label="Legend">
            {levels.into_iter().map(|l| view! {
                <li><Tally n=l.tallies() label=l.label()/>{l.label()}</li>
            }).collect_view()}
        </ul>

        <div class="skill-groups">
            {site().skills.iter().map(|g| view! {
                <section class="skill-group">
                    <h2 class="skill-group__name">{g.name}{placeholder(g.placeholder)}</h2>
                    {(!g.blurb.is_empty()).then(|| view! { <p class="skill-group__blurb">{g.blurb}</p> })}
                    <ul>
                        {g.skills.iter().map(|s| view! {
                            <li class="skill">
                                <span class="skill__name">{s.name}</span>
                                <Tally n=s.level.tallies() label=s.level.label()/>
                                <span class="mono skill__level">{s.level.label()}</span>
                                {(!s.note.is_empty()).then(|| view! { <span class="mono skill__note">{s.note}</span> })}
                            </li>
                        }).collect_view()}
                    </ul>
                </section>
            }).collect_view()}
        </div>
    }
}

#[component]
pub fn Experience() -> impl IntoView {
    let e = site().experience;
    view! {
        <Seo title="Experience" description="Roles, freelance work and education."/>
        <PageHeader num="04" label="Experience" title="Experience" lede="Where I've played."/>

        {e.cv.map(|cv| view! {
            <a class="btn btn--accent cv-button" href=cv download="">"Download CV (PDF)"</a>
        })}

        <ol class="timeline">
            {e.roles.iter().map(|r| view! {
                <li class="timeline__item">
                    <span class="mono timeline__when">
                        {r.start}" — "{if r.end.is_empty() { "Now" } else { r.end }}
                    </span>
                    <div class="timeline__what">
                        <h2 class="timeline__title">{r.title}{placeholder(r.placeholder)}</h2>
                        <p class="mono timeline__org">
                            {r.org}
                            {(!r.location.is_empty()).then(|| view! { " · "{r.location} })}
                            " · "{r.kind.label()}
                        </p>
                        <p>{r.summary}</p>
                    </div>
                </li>
            }).collect_view()}
        </ol>
    }
}
