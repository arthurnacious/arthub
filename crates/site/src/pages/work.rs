//! `/work` (index with filters and grid/list views) and `/work/:slug`.

use content::{MediaKind, Project};
use leptos::prelude::*;
use leptos_router::{
    NavigateOptions,
    components::A,
    hooks::{use_navigate, use_params_map, use_query_map},
};

use crate::components::section::{PageHeader, SectionLabel};
use crate::components::seo::Seo;
use crate::data::site;
use crate::pages::NotFound;

thread_local! {
    /// A shared do-nothing rejection handler for `play()` (browser only in
    /// practice: on the server there is never a video to play).
    static IGNORE: wasm_bindgen::closure::Closure<dyn FnMut(wasm_bindgen::JsValue)> =
        wasm_bindgen::closure::Closure::new(|_| {});
}

/// Cover image that swaps to the preview video (or first gallery image) on
/// hover. Videos only load when first hovered.
#[component]
pub fn ProjectMedia(project: &'static Project, #[prop(optional)] eager: bool) -> impl IntoView {
    let video = NodeRef::<leptos::html::Video>::new();
    let alt_img = project
        .gallery
        .iter()
        .find(|m| m.kind == MediaKind::Image)
        .map(|m| m.src);
    let play = move |_| {
        if let Some(v) = video.get() {
            if v.preload() == "none" {
                v.set_preload("auto");
            }
            // Navigating away mid-play rejects the promise; that's fine.
            if let Ok(p) = v.play() {
                IGNORE.with(|f| {
                    let _ = p.catch(f);
                });
            }
        }
    };
    let stop = move |_| {
        if let Some(v) = video.get() {
            let _ = v.pause();
        }
    };
    view! {
        <div class="project-media" on:mouseenter=play on:mouseleave=stop
            style:aspect-ratio=format!("{} / {}", project.cover.width, project.cover.height)>
            <img src=project.cover.src alt=project.cover.alt width=project.cover.width height=project.cover.height
                loading=if eager { "eager" } else { "lazy" } decoding="async"/>
            {match (project.preview_video, alt_img) {
                (Some(src), _) => view! {
                    <video class="project-media__hover" node_ref=video src=src muted=true loop=true playsinline=true
                        preload="none" aria-hidden="true"></video>
                }.into_any(),
                (None, Some(src)) => view! {
                    <img class="project-media__hover" src=src alt="" loading="lazy" decoding="async"/>
                }.into_any(),
                _ => ().into_any(),
            }}
        </div>
    }
}

fn badges(p: &'static Project) -> impl IntoView {
    p.example
        .then(|| view! { <span class="badge mono">"Example"</span> })
}

/// A project as a card (grid view, home page).
#[component]
pub fn ProjectCard(project: &'static Project, index: usize) -> impl IntoView {
    view! {
        <A href=format!("/work/{}", project.slug) attr:class="project-card" attr:data-cursor="view">
            <ProjectMedia project/>
            <div class="project-card__meta">
                <span class="mono project-card__num">{format!("{:02}", index + 1)}</span>
                <h3 class="project-card__title">{project.title}{badges(project)}</h3>
                <span class="mono">{project.year}" · "{project.role}</span>
                <ul class="tags mono">{project.categories.iter().map(|c| view! { <li>{*c}</li> }).collect_view()}</ul>
            </div>
        </A>
    }
}

const VIEWS: [&str; 2] = ["grid", "list"];

#[component]
pub fn WorkIndex() -> impl IntoView {
    let query = use_query_map();
    let navigate = use_navigate();
    let cat = move || query.read().get("cat");
    let view_mode = move || {
        query
            .read()
            .get("view")
            .filter(|v| VIEWS.contains(&v.as_str()))
            .unwrap_or_else(|| "grid".into())
    };

    // Filters and view live in the URL, so they survive reloads and share.
    let set = move |cat: Option<String>, mode: String| {
        let mut qs = Vec::new();
        if let Some(c) = cat {
            qs.push(format!("cat={c}"));
        }
        if mode != "grid" {
            qs.push(format!("view={mode}"));
        }
        let url = if qs.is_empty() {
            "/work".to_string()
        } else {
            format!("/work?{}", qs.join("&"))
        };
        navigate(
            &url,
            NavigateOptions {
                replace: true,
                scroll: false,
                ..Default::default()
            },
        );
    };

    let shown = move || {
        let c = cat();
        site()
            .projects
            .iter()
            .filter(|p| {
                c.as_ref()
                    .is_none_or(|c| p.categories.iter().any(|pc| pc.eq_ignore_ascii_case(c)))
            })
            .collect::<Vec<_>>()
    };

    // List view: a preview that follows the cursor.
    let float = NodeRef::<leptos::html::Img>::new();
    let hovered = RwSignal::new(None::<&'static str>);
    let follow = move |e: leptos::ev::MouseEvent| {
        if let Some(img) = float.get() {
            let _ = web_sys::HtmlElement::style(&img).set_property(
                "transform",
                &format!(
                    "translate3d({}px, {}px, 0)",
                    e.client_x() + 24,
                    e.client_y() - 90
                ),
            );
        }
    };

    let total = site().projects.len();
    let set_cat = set.clone();
    let set_view = set;

    view! {
        <Seo title="Work" description="Selected projects: web, systems and real-time 3D."/>
        <PageHeader num="01" label="Work" title="Work" lede="Everything I've shipped, sorted newest first."/>

        <div class="work-controls">
            <div class="work-filters" role="group" aria-label="Filter by category">
                <button class="chip mono" aria-pressed=move || cat().is_none().to_string()
                    on:click={ let set = set_cat.clone(); move |_| set(None, view_mode()) }>
                    "All " <span>{total}</span>
                </button>
                {site().categories().into_iter().map(|c| {
                    let n = site().projects.iter().filter(|p| p.categories.contains(&c)).count();
                    let key = c.to_ascii_lowercase();
                    let k2 = key.clone();
                    let set = set_cat.clone();
                    view! {
                        <button class="chip mono"
                            aria-pressed=move || cat().is_some_and(|x| x.eq_ignore_ascii_case(&k2)).to_string()
                            on:click=move |_| set(Some(key.clone()), view_mode())>
                            {c} " " <span>{n}</span>
                        </button>
                    }
                }).collect_view()}
            </div>
            <div class="work-views" role="group" aria-label="Layout">
                {VIEWS.into_iter().map(|m| {
                    let set = set_view.clone();
                    view! {
                        <button class="chip mono" aria-pressed=move || (view_mode() == m).to_string()
                            on:click=move |_| set(cat(), m.to_string())>{m}</button>
                    }
                }).collect_view()}
            </div>
        </div>

        <p class="visually-hidden" aria-live="polite">{move || format!("{} projects shown", shown().len())}</p>

        {move || if view_mode() == "list" {
            view! {
                <ol class="work-list" on:mousemove=follow on:mouseleave=move |_| hovered.set(None)>
                    <For each=shown key={|p| p.slug} children={move |p| view! {
                        <li class="reveal">
                            <A href=format!("/work/{}", p.slug) attr:class="work-row" attr:data-cursor="view"
                                on:mouseenter=move |_| hovered.set(Some(p.cover.src))>
                                <span class="mono">{p.year}</span>
                                <span class="work-row__title">{p.title}{badges(p)}</span>
                                <span class="mono work-row__role">{p.role}</span>
                                <span class="mono work-row__cats">{p.categories.join(" / ")}</span>
                                <span class="work-row__arrow" aria-hidden="true">"↗"</span>
                            </A>
                        </li>
                    }}/>
                </ol>
                <img class="work-float" node_ref=float alt="" aria-hidden="true"
                    class:work-float--on=move || hovered.get().is_some()
                    src=move || hovered.get().unwrap_or_default()/>
            }.into_any()
        } else {
            view! {
                <div class="work-grid">
                    <For each={move || shown().into_iter().enumerate().collect::<Vec<_>>()} key={|(_, p)| p.slug}
                        children={move |(i, p)| view! { <div class="reveal"><ProjectCard project=p index=i/></div> }}/>
                </div>
            }.into_any()
        }}
        <Show when=move || shown().is_empty()>
            <p class="lede">"Nothing in that category yet."</p>
        </Show>
    }
}

#[component]
pub fn CaseStudy() -> impl IntoView {
    let params = use_params_map();
    let project = move || params.read().get("slug").and_then(|s| site().project(&s));

    move || match project() {
        None => view! { <NotFound/> }.into_any(),
        Some(p) => view! { <CaseStudyView project=p/> }.into_any(),
    }
}

#[component]
fn CaseStudyView(project: &'static Project) -> impl IntoView {
    let p = project;
    let next = site().next_after(p.slug).filter(|n| n.slug != p.slug);

    let meta: Vec<(&str, String)> = [
        ("Client", p.client.to_string()),
        ("Year", p.year.to_string()),
        ("Role", p.role.to_string()),
        ("Tools", p.tools.join(", ")),
        (
            "Team",
            p.credits
                .iter()
                .map(|c| c.name)
                .collect::<Vec<_>>()
                .join(", "),
        ),
    ]
    .into_iter()
    .filter(|(_, v)| !v.is_empty())
    .collect();

    view! {
        <Seo title=p.title description=p.summary image=p.cover.src kind="article"/>

        <article class="case">
            <header class="case__head">
                <SectionLabel num="01" text="Case study"/>
                <h1 class="display case__title">{p.title}</h1>
                {badges(p)}
                <p class="lede case__summary">{p.summary}</p>
                <dl class="case__meta mono">
                    {meta.into_iter().map(|(k, v)| view! { <div><dt>{k}</dt><dd>{v}</dd></div> }).collect_view()}
                </dl>
            </header>

            <figure class="case__hero" style=format!("--ratio: {}", p.cover.width as f32 / p.cover.height.max(1) as f32)>
                {match p.preview_video {
                    Some(v) => view! {
                        <video src=v poster=p.cover.src muted=true loop=true autoplay=true playsinline=true
                            aria-label=p.cover.alt></video>
                    }.into_any(),
                    None => view! {
                        <img src=p.cover.src alt=p.cover.alt width=p.cover.width height=p.cover.height decoding="async"/>
                    }.into_any(),
                }}
            </figure>

            <div class="case__body">
                <div class="prose" inner_html=p.body_html></div>
                <aside class="case__aside mono">
                    {(!p.links.is_empty()).then(|| view! {
                        <section>
                            <h2>"Links"</h2>
                            <ul>{p.links.iter().map(|l| view! {
                                <li><a href=l.url target="_blank" rel="noopener">{l.label}" ↗"</a></li>
                            }).collect_view()}</ul>
                        </section>
                    })}
                    {(!p.credits.is_empty()).then(|| view! {
                        <section>
                            <h2>"Credits"</h2>
                            <ul>{p.credits.iter().map(|c| view! {
                                <li><span>{c.role}</span>" "{c.name}</li>
                            }).collect_view()}</ul>
                        </section>
                    })}
                    <section>
                        <h2>"Categories"</h2>
                        <ul class="tags">{p.categories.iter().map(|c| view! { <li>{*c}</li> }).collect_view()}</ul>
                    </section>
                </aside>
            </div>

            {(!p.gallery.is_empty()).then(|| view! {
                <section class="case__gallery" aria-label="Gallery">
                    <SectionLabel num="02" text="Gallery"/>
                    <div class="gallery">
                        {p.gallery.iter().map(|m| view! {
                            <figure class="gallery__item" class:gallery__item--tall={m.height > m.width}>
                                {match m.kind {
                                    MediaKind::Video => view! {
                                        <video src=m.src controls=true muted=true playsinline=true preload="metadata"
                                            aria-label=m.alt></video>
                                    }.into_any(),
                                    MediaKind::Image => view! {
                                        <img src=m.src alt=m.alt width=m.width height=m.height loading="lazy" decoding="async"/>
                                    }.into_any(),
                                }}
                                {(!m.caption.is_empty()).then(|| view! { <figcaption class="mono">{m.caption}</figcaption> })}
                            </figure>
                        }).collect_view()}
                    </div>
                </section>
            })}

            {next.map(|n| view! {
                <A href=format!("/work/{}", n.slug) attr:class="next-project" attr:data-cursor="view">
                    <span class="mono">"Next project →"</span>
                    <span class="display next-project__title">{n.title}</span>
                    <img src=n.cover.src alt="" loading="lazy" decoding="async" width=n.cover.width height=n.cover.height/>
                </A>
            })}
        </article>
    }
}
