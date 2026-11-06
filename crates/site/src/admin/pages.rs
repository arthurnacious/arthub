//! Dashboard pages. The shell checks the session once; each page loads its
//! data through the server functions in `api` and refreshes after changes.

use content::owned::{Credit, Image, Link, Media, Project, Role, Skill, SkillGroup};
use leptos::prelude::*;
use leptos_router::components::{A, Outlet, Redirect};
use leptos_router::hooks::{use_navigate, use_params_map, use_query_map};

use super::api::*;
use super::types::{Kind, MessageRow, Upload, when};
use crate::components::seo::Seo;

// ---- sign in -----------------------------------------------------------------

#[component]
pub fn AdminLogin() -> impl IntoView {
    let q = use_query_map();
    let note = move || {
        let q = q.read();
        if q.get("sent").is_some() {
            Some((
                "ok",
                "If that's the admin address, a sign-in link is on its way. It works once, for 15 minutes.",
            ))
        } else {
            q.get("error").map(|e| match e.as_str() {
                "expired" => (
                    "error",
                    "That link has expired or was already used. Ask for a new one.",
                ),
                "rate" => (
                    "error",
                    "Too many sign-in requests. Try again in a few minutes.",
                ),
                "send" => (
                    "error",
                    "Couldn't send the email. Check the server's email settings.",
                ),
                _ => ("error", "Something went wrong. Try again."),
            })
        }
    };
    view! {
        <Seo title="Sign in" description="Dashboard sign-in." noindex=true/>
        <section class="admin-login">
            <A href="/" attr:class="admin-login__mark display">"ARTHUB"</A>
            <h1 class="display admin-login__title">"Dashboard"</h1>
            <p class="admin-muted">"Enter your email and we'll send you a sign-in link."</p>
            {move || note().map(|(kind, text)| view! { <p class=format!("admin-note admin-note--{kind}") role="status">{text}</p> })}
            <form method="post" action="/api/admin/login" class="admin-login__form">
                <label class="admin-field">
                    <span class="mono">"Email"</span>
                    <input type="email" name="email" required autocomplete="email" autofocus/>
                </label>
                <button type="submit" class="btn btn--accent">"Send me a link"</button>
            </form>
        </section>
    }
}

// ---- shell -------------------------------------------------------------------

/// Signed-in layout: sidebar and the page. Signed out: off to sign-in.
#[component]
pub fn AdminShell() -> impl IntoView {
    let me = Resource::new(|| (), |_| admin_me());
    view! {
        <Seo title="Dashboard" description="Dashboard." noindex=true/>
        <Suspense fallback=|| view! { <p class="admin-loading mono">"Loading…"</p> }>
            {move || me.get().map(|r| match r {
                Ok(true) => view! { <AdminLayout/> }.into_any(),
                _ => view! { <Redirect path="/admin/login"/> }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn AdminLayout() -> impl IntoView {
    // Refreshed whenever a page changes something that affects it.
    let unread = Resource::new(|| (), |_| new_count());
    provide_context(UnreadCount(unread));
    let link = |href: &'static str, label: &'static str| {
        view! { <li><A href=href attr:class="admin-nav__link">{label}</A></li> }
    };
    view! {
        <div class="admin">
            <aside class="admin__side">
                <A href="/" attr:class="admin__mark display">"ARTHUB"</A>
                <nav aria-label="Dashboard">
                    <ul class="admin-nav">
                        <li>
                            <A href="/admin/messages" attr:class="admin-nav__link">
                                "Messages"
                                <Suspense>
                                    {move || unread.get().and_then(|r| r.ok()).filter(|n| *n > 0).map(|n| view! {
                                        <span class="admin-badge mono">{n}</span>
                                    })}
                                </Suspense>
                            </A>
                        </li>
                        {link("/admin/projects", "Projects")}
                        {link("/admin/experience", "Experience")}
                        {link("/admin/skills", "Skills")}
                    </ul>
                </nav>
                <div class="admin__side-foot">
                    <A href="/" attr:class="admin-nav__link admin-muted">"View site ↗"</A>
                    <form method="post" action="/api/admin/logout">
                        <button type="submit" class="admin-link">"Sign out"</button>
                    </form>
                </div>
            </aside>
            <div class="admin__main">
                <Outlet/>
            </div>
        </div>
    }
}

#[derive(Clone, Copy)]
struct UnreadCount(Resource<Result<i64, ServerFnError>>);

fn refresh_unread() {
    if let Some(UnreadCount(r)) = use_context::<UnreadCount>() {
        r.refetch();
    }
}

fn err_text(e: &ServerFnError) -> String {
    match e {
        ServerFnError::ServerError(m) => m.clone(),
        other => other.to_string(),
    }
}

#[component]
fn ErrorNote(#[prop(into)] error: Signal<Option<String>>) -> impl IntoView {
    move || {
        error
            .get()
            .map(|e| view! { <p class="admin-note admin-note--error" role="alert">{e}</p> })
    }
}

// ---- messages ----------------------------------------------------------------

const FILTERS: [(&str, &str); 4] = [
    ("open", "Inbox"),
    ("new", "Unread"),
    ("replied", "Replied"),
    ("archived", "Archived"),
];

#[component]
pub fn AdminMessages() -> impl IntoView {
    let q = use_query_map();
    let filter = move || q.read().get("f").unwrap_or_else(|| "open".into());
    let list = Resource::new(filter, list_messages);
    view! {
        <header class="admin-head">
            <h1 class="display admin-head__title">"Messages"</h1>
        </header>
        <div class="admin-tabs" role="tablist">
            {FILTERS.iter().map(|(id, label)| {
                let href = format!("/admin/messages?f={id}");
                view! {
                    <A href=href attr:class="chip" attr:aria-pressed=move || (filter() == *id).to_string()>{*label}</A>
                }
            }).collect_view()}
        </div>
        <Transition fallback=|| view! { <p class="admin-loading mono">"Loading…"</p> }>
            {move || list.get().map(|r| match r {
                Err(e) => view! { <p class="admin-note admin-note--error">{err_text(&e)}</p> }.into_any(),
                Ok(rows) if rows.is_empty() => view! {
                    <p class="admin-empty">"Nothing here. Enquiries from the contact form will show up in the inbox."</p>
                }.into_any(),
                Ok(rows) => view! {
                    <ol class="admin-list">
                        {rows.into_iter().map(|m| view! { <MessageItem m/> }).collect_view()}
                    </ol>
                }.into_any(),
            })}
        </Transition>
    }
}

#[component]
fn MessageItem(m: MessageRow) -> impl IntoView {
    let preview: String = m.body.chars().take(140).collect();
    let class = format!("admin-msg admin-msg--{}", m.status);
    view! {
        <li>
            <A href=format!("/admin/messages/{}", m.id) attr:class=class>
                <span class="admin-msg__who">
                    <strong>{m.name.clone()}</strong>
                    <span class="admin-muted">{m.email.clone()}</span>
                </span>
                <span class="mono admin-msg__meta">
                    {m.project_type.clone()}" · "{m.budget.clone()}
                </span>
                <span class="admin-msg__preview">{preview}</span>
                <span class="mono admin-msg__when">
                    {when(m.created_at)}
                    {(m.replies > 0).then(|| format!(" · {} repl{}", m.replies, if m.replies == 1 { "y" } else { "ies" }))}
                </span>
                <span class=format!("mono admin-status admin-status--{}", m.status)>{m.status.clone()}</span>
            </A>
        </li>
    }
}

#[component]
pub fn AdminThread() -> impl IntoView {
    let params = use_params_map();
    let id = move || {
        params
            .read()
            .get("id")
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0)
    };
    let data = Resource::new(id, get_thread);
    let draft = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);
    let navigate = use_navigate();

    let send = Action::new_local(move |(id, body): &(i64, String)| {
        let (id, body) = (*id, body.clone());
        async move { reply(id, body).await }
    });
    Effect::new(move |_| {
        if let Some(r) = send.value().get() {
            match r {
                Ok(()) => {
                    draft.set(String::new());
                    error.set(None);
                }
                Err(e) => error.set(Some(err_text(&e))),
            }
            data.refetch();
            refresh_unread();
        }
    });
    let status = Action::new_local(move |(id, s): &(i64, String)| {
        let (id, s) = (*id, s.clone());
        async move { set_status(id, s).await }
    });
    Effect::new(move |_| {
        if status.value().get().is_some() {
            data.refetch();
            refresh_unread();
        }
    });
    let remove = Action::new_local(move |id: &i64| {
        let id = *id;
        async move { delete_message(id).await }
    });
    Effect::new(move |_| {
        if let Some(Ok(())) = remove.value().get() {
            refresh_unread();
            navigate("/admin/messages", Default::default());
        }
    });

    view! {
        <A href="/admin/messages" attr:class="admin-back mono">"← Messages"</A>
        <Transition fallback=|| view! { <p class="admin-loading mono">"Loading…"</p> }>
            {move || data.get().map(|r| match r {
                Err(e) => view! { <p class="admin-note admin-note--error">{err_text(&e)}</p> }.into_any(),
                Ok(t) => {
                    let m = t.message.clone();
                    let mid = m.id;
                    let archived = m.status == "archived";
                    view! {
                        <header class="admin-head">
                            <h1 class="display admin-head__title">{m.name.clone()}</h1>
                            <div class="admin-head__actions">
                                <a class="btn btn--small" href=format!("mailto:{}", m.email)>{m.email.clone()}</a>
                                <button class="btn btn--small" on:click=move |_| {
                                    status.dispatch((mid, if archived { "read".into() } else { "archived".into() }));
                                }>{if archived { "Unarchive" } else { "Archive" }}</button>
                                <button class="btn btn--small admin-danger" on:click=move |_| {
                                    let ok = web_sys::window()
                                        .and_then(|w| w.confirm_with_message("Delete this conversation for good?").ok())
                                        .unwrap_or(false);
                                    if ok {
                                        remove.dispatch(mid);
                                    }
                                }>"Delete"</button>
                            </div>
                        </header>
                        <ol class="admin-thread">
                            <li class="admin-bubble admin-bubble--them">
                                <p class="mono admin-bubble__meta">
                                    {when(m.created_at)}" · "{m.project_type.clone()}" · "{m.budget.clone()}
                                </p>
                                <p class="admin-bubble__body">{m.body.clone()}</p>
                            </li>
                            {t.replies.into_iter().map(|r| {
                                let failed = r.error.is_some();
                                view! {
                                <li class="admin-bubble admin-bubble--me" class:admin-bubble--failed=failed>
                                    <p class="mono admin-bubble__meta">
                                        {when(r.created_at)}
                                        {r.error.as_ref().map(|_| " · not sent")}
                                    </p>
                                    <p class="admin-bubble__body">{r.body}</p>
                                    {r.error.map(|e| view! { <p class="mono admin-bubble__error">{e}</p> })}
                                </li>
                            }}).collect_view()}
                        </ol>
                        <form class="admin-compose" on:submit=move |ev| {
                            ev.prevent_default();
                            send.dispatch((mid, draft.get_untracked()));
                        }>
                            <label class="admin-field">
                                <span class="mono">{format!("Reply to {} (sent by email)", m.name)}</span>
                                <textarea rows="6" prop:value=move || draft.get()
                                    on:input=move |e| draft.set(event_target_value(&e))
                                    placeholder="Write your reply…"></textarea>
                            </label>
                            <ErrorNote error=error/>
                            <button type="submit" class="btn btn--accent" disabled=move || send.pending().get() || draft.with(|d| d.trim().is_empty())>
                                {move || if send.pending().get() { "Sending…" } else { "Send reply" }}
                            </button>
                        </form>
                    }.into_any()
                }
            })}
        </Transition>
    }
}

// ---- reorder / delete controls -------------------------------------------------

/// Up, down and delete for a row of a collection. Calls `done` after a change.
#[component]
fn RowControls(kind: Kind, id: i64, first: bool, last: bool, done: Callback<()>) -> impl IntoView {
    let mv = Action::new_local(move |up: &bool| {
        let up = *up;
        async move { move_item(kind, id, up).await }
    });
    let del = Action::new_local(move |_: &()| async move { delete_item(kind, id).await });
    Effect::new(move |_| {
        if mv.value().get().is_some() || del.value().get().is_some() {
            done.run(());
        }
    });
    view! {
        <span class="admin-row-controls">
            <button class="admin-icon" title="Move up" aria-label="Move up" disabled=first on:click=move |_| { mv.dispatch(true); }>"↑"</button>
            <button class="admin-icon" title="Move down" aria-label="Move down" disabled=last on:click=move |_| { mv.dispatch(false); }>"↓"</button>
            <button class="admin-icon admin-danger" title="Delete" aria-label="Delete" on:click=move |_| {
                let ok = web_sys::window()
                    .and_then(|w| w.confirm_with_message("Delete this for good?").ok())
                    .unwrap_or(false);
                if ok {
                    del.dispatch(());
                }
            }>"✕"</button>
        </span>
    }
}

// ---- projects ------------------------------------------------------------------

#[component]
pub fn AdminProjects() -> impl IntoView {
    let list = Resource::new(|| (), |_| list_projects());
    let done = Callback::new(move |_| list.refetch());
    view! {
        <header class="admin-head">
            <h1 class="display admin-head__title">"Projects"</h1>
            <div class="admin-head__actions">
                <A href="/admin/projects/new" attr:class="btn btn--accent btn--small">"New project"</A>
            </div>
        </header>
        <p class="admin-muted">"Shown newest first, in this order. Featured projects go on the home page and the arcade cabinets."</p>
        <Transition fallback=|| view! { <p class="admin-loading mono">"Loading…"</p> }>
            {move || list.get().map(|r| match r {
                Err(e) => view! { <p class="admin-note admin-note--error">{err_text(&e)}</p> }.into_any(),
                Ok(rows) => {
                    let n = rows.len();
                    view! {
                        <ol class="admin-list">
                            {rows.into_iter().enumerate().map(|(i, row)| {
                                let p = row.data;
                                view! {
                                    <li class="admin-item">
                                        <img class="admin-item__thumb" src=p.cover.src.clone() alt="" loading="lazy"/>
                                        <A href=format!("/admin/projects/{}", row.id) attr:class="admin-item__main">
                                            <strong>{p.title.clone()}</strong>
                                            <span class="mono admin-muted">
                                                {p.year}" · /work/"{p.slug.clone()}
                                                {p.featured.then_some(" · featured")}
                                                {p.example.then_some(" · example")}
                                            </span>
                                        </A>
                                        <RowControls kind=Kind::Projects id=row.id first={i == 0} last={i + 1 == n} done/>
                                    </li>
                                }
                            }).collect_view()}
                        </ol>
                    }.into_any()
                }
            })}
        </Transition>
    }
}

fn blank_project() -> Project {
    Project {
        slug: String::new(),
        title: String::new(),
        year: 2026,
        client: String::new(),
        role: String::new(),
        categories: vec![],
        tools: vec![],
        featured: false,
        accent: [228, 223, 209],
        cover: Image {
            src: String::new(),
            width: 0,
            height: 0,
            alt: String::new(),
        },
        cover_asset: String::new(),
        preview_video: None,
        summary: String::new(),
        body_md: String::new(),
        body_html: String::new(),
        gallery: vec![],
        links: vec![],
        credits: vec![],
        example: false,
    }
}

fn slugify(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

fn split_list(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}

fn pairs(s: &str) -> Vec<(String, String)> {
    s.lines()
        .filter_map(|l| {
            let (a, b) = l.split_once('|')?;
            let (a, b) = (a.trim(), b.trim());
            (!a.is_empty() && !b.is_empty()).then(|| (a.to_string(), b.to_string()))
        })
        .collect()
}

fn hex(c: [u8; 3]) -> String {
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

fn parse_hex(s: &str) -> Option<[u8; 3]> {
    let s = s.trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let p = |i: usize| u8::from_str_radix(&s[i..i + 2], 16).ok();
    Some([p(0)?, p(2)?, p(4)?])
}

/// Upload a file through the dashboard's upload route.
async fn upload(file: web_sys::File) -> Result<Upload, String> {
    #[cfg(feature = "hydrate")]
    {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        let form = web_sys::FormData::new().map_err(|_| "form".to_string())?;
        form.append_with_blob_and_filename("file", &file, &file.name())
            .map_err(|_| "form".to_string())?;
        let init = web_sys::RequestInit::new();
        init.set_method("POST");
        init.set_body(&form);
        init.set_credentials(web_sys::RequestCredentials::SameOrigin);
        let req = web_sys::Request::new_with_str_and_init("/api/admin/upload", &init)
            .map_err(|_| "request".to_string())?;
        let window = web_sys::window().ok_or("no window")?;
        let res: web_sys::Response = JsFuture::from(window.fetch_with_request(&req))
            .await
            .map_err(|_| "Upload failed: network error.".to_string())?
            .unchecked_into();
        let text = JsFuture::from(res.text().map_err(|_| "response".to_string())?)
            .await
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();
        if !res.ok() {
            return Err(if text.is_empty() {
                format!("Upload failed ({}).", res.status())
            } else {
                text
            });
        }
        serde_json::from_str(&text).map_err(|e| format!("Upload: {e}"))
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = file;
        Err("Uploads happen in the browser.".into())
    }
}

/// The first file chosen in an `<input type=file>`.
fn chosen_file(ev: &leptos::ev::Event) -> Option<web_sys::File> {
    use wasm_bindgen::JsCast;
    let input: web_sys::HtmlInputElement = ev.target()?.dyn_into().ok()?;
    input.files()?.get(0)
}

#[component]
pub fn AdminProjectEdit() -> impl IntoView {
    let params = use_params_map();
    let id = move || params.read().get("id").and_then(|s| s.parse::<i64>().ok());
    let loaded = Resource::new(id, |id| async move {
        match id {
            Some(id) => get_project(id).await.map(|p| (Some(id), p)),
            None => Ok((None, blank_project())),
        }
    });
    view! {
        <A href="/admin/projects" attr:class="admin-back mono">"← Projects"</A>
        <Suspense fallback=|| view! { <p class="admin-loading mono">"Loading…"</p> }>
            {move || loaded.get().map(|r| match r {
                Err(e) => view! { <p class="admin-note admin-note--error">{err_text(&e)}</p> }.into_any(),
                Ok((id, p)) => view! { <ProjectForm id project=p/> }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn ProjectForm(id: Option<i64>, project: Project) -> impl IntoView {
    let p = RwSignal::new(project);
    let row_id = RwSignal::new(id);
    // Free-text views of the list fields, parsed on save.
    let categories = RwSignal::new(p.with_untracked(|p| p.categories.join(", ")));
    let tools = RwSignal::new(p.with_untracked(|p| p.tools.join(", ")));
    let links = RwSignal::new(p.with_untracked(|p| {
        p.links
            .iter()
            .map(|l| format!("{} | {}", l.label, l.url))
            .collect::<Vec<_>>()
            .join("\n")
    }));
    let credits = RwSignal::new(p.with_untracked(|p| {
        p.credits
            .iter()
            .map(|c| format!("{} | {}", c.role, c.name))
            .collect::<Vec<_>>()
            .join("\n")
    }));
    let slug_touched = RwSignal::new(id.is_some());
    let error = RwSignal::new(None::<String>);
    let saved = RwSignal::new(false);
    let busy = RwSignal::new(false);

    let save = Action::new_local(move |(id, project): &(Option<i64>, Project)| {
        let (id, project) = (*id, project.clone());
        async move { save_project(id, project).await }
    });
    Effect::new(move |_| {
        if let Some(r) = save.value().get() {
            match r {
                Ok(new_id) => {
                    error.set(None);
                    saved.set(true);
                    if row_id.get_untracked().is_none() {
                        row_id.set(Some(new_id));
                        let _ = web_sys::window().map(|w| {
                            w.history().ok().map(|h| {
                                h.replace_state_with_url(
                                    &wasm_bindgen::JsValue::NULL,
                                    "",
                                    Some(&format!("/admin/projects/{new_id}")),
                                )
                            })
                        });
                    }
                }
                Err(e) => {
                    saved.set(false);
                    error.set(Some(err_text(&e)));
                }
            }
        }
    });

    let field =
        move |label: &'static str, get: fn(&Project) -> String, set: fn(&mut Project, String)| {
            view! {
                <label class="admin-field">
                    <span class="mono">{label}</span>
                    <input type="text" prop:value=move || p.with(get) on:input=move |e| {
                        let v = event_target_value(&e);
                        p.update(|p| set(p, v));
                        saved.set(false);
                    }/>
                </label>
            }
        };

    let on_cover = move |ev: leptos::ev::Event| {
        let Some(file) = chosen_file(&ev) else { return };
        busy.set(true);
        leptos::task::spawn_local(async move {
            match upload(file).await {
                Ok(u) if !u.video => p.update(|p| {
                    p.cover.src = u.src;
                    p.cover.width = u.width;
                    p.cover.height = u.height;
                    p.cover_asset = u.asset;
                }),
                Ok(_) => error.set(Some("The cover must be an image.".into())),
                Err(e) => error.set(Some(e)),
            }
            busy.set(false);
        });
    };
    let on_video = move |ev: leptos::ev::Event| {
        let Some(file) = chosen_file(&ev) else { return };
        busy.set(true);
        leptos::task::spawn_local(async move {
            match upload(file).await {
                Ok(u) if u.video => p.update(|p| p.preview_video = Some(u.src)),
                Ok(_) => error.set(Some("That's not a video.".into())),
                Err(e) => error.set(Some(e)),
            }
            busy.set(false);
        });
    };
    let on_gallery = move |ev: leptos::ev::Event| {
        let Some(file) = chosen_file(&ev) else { return };
        busy.set(true);
        leptos::task::spawn_local(async move {
            match upload(file).await {
                Ok(u) => p.update(|p| {
                    p.gallery.push(Media {
                        video: u.video,
                        src: u.src,
                        width: u.width,
                        height: u.height,
                        alt: String::new(),
                        caption: String::new(),
                    })
                }),
                Err(e) => error.set(Some(e)),
            }
            busy.set(false);
        });
    };

    let submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let mut out = p.get_untracked();
        out.categories = split_list(&categories.get_untracked());
        out.tools = split_list(&tools.get_untracked());
        out.links = pairs(&links.get_untracked())
            .into_iter()
            .map(|(label, url)| Link { label, url })
            .collect();
        out.credits = pairs(&credits.get_untracked())
            .into_iter()
            .map(|(role, name)| Credit { role, name })
            .collect();
        let problems = check_project(&out);
        if !problems.is_empty() {
            error.set(Some(problems.join(" ")));
            return;
        }
        save.dispatch((row_id.get_untracked(), out));
    };

    view! {
        <header class="admin-head">
            <h1 class="display admin-head__title">{move || {
                let t = p.with(|p| p.title.clone());
                if t.is_empty() { "New project".to_string() } else { t }
            }}</h1>
            <div class="admin-head__actions">
                {move || row_id.get().and(Some(p.with(|p| p.slug.clone())))
                    .filter(|s| !s.is_empty())
                    .map(|s| view! { <a class="btn btn--small" href=format!("/work/{s}") target="_blank">"View ↗"</a> })}
            </div>
        </header>
        <form class="admin-form" on:submit=submit>
            <div class="admin-grid">
                <label class="admin-field">
                    <span class="mono">"Title"</span>
                    <input type="text" required prop:value=move || p.with(|p| p.title.clone()) on:input=move |e| {
                        let v = event_target_value(&e);
                        p.update(|p| {
                            if !slug_touched.get_untracked() {
                                p.slug = slugify(&v);
                            }
                            p.title = v;
                        });
                        saved.set(false);
                    }/>
                </label>
                <label class="admin-field">
                    <span class="mono">"Slug (the /work/… address)"</span>
                    <input type="text" required prop:value=move || p.with(|p| p.slug.clone()) on:input=move |e| {
                        slug_touched.set(true);
                        let v = event_target_value(&e);
                        p.update(|p| p.slug = v);
                        saved.set(false);
                    }/>
                </label>
                <label class="admin-field">
                    <span class="mono">"Year"</span>
                    <input type="number" min="1990" max="2100" prop:value=move || p.with(|p| p.year.to_string()) on:input=move |e| {
                        if let Ok(y) = event_target_value(&e).parse() {
                            p.update(|p| p.year = y);
                            saved.set(false);
                        }
                    }/>
                </label>
                {field("Your role", |p| p.role.clone(), |p, v| p.role = v)}
                {field("Client", |p| p.client.clone(), |p, v| p.client = v)}
                <label class="admin-field">
                    <span class="mono">"Categories (comma-separated)"</span>
                    <input type="text" prop:value=move || categories.get() on:input=move |e| { categories.set(event_target_value(&e)); saved.set(false); }/>
                </label>
                <label class="admin-field">
                    <span class="mono">"Tools (comma-separated)"</span>
                    <input type="text" prop:value=move || tools.get() on:input=move |e| { tools.set(event_target_value(&e)); saved.set(false); }/>
                </label>
                <label class="admin-field">
                    <span class="mono">"Accent (arcade glow)"</span>
                    <input type="color" prop:value=move || p.with(|p| hex(p.accent)) on:input=move |e| {
                        if let Some(c) = parse_hex(&event_target_value(&e)) {
                            p.update(|p| p.accent = c);
                            saved.set(false);
                        }
                    }/>
                </label>
            </div>
            <div class="admin-checks">
                <label><input type="checkbox" prop:checked=move || p.with(|p| p.featured) on:change=move |e| { let v = event_target_checked(&e); p.update(|p| p.featured = v); saved.set(false); }/>" Featured (home page and arcade)"</label>
                <label><input type="checkbox" prop:checked=move || p.with(|p| p.example) on:change=move |e| { let v = event_target_checked(&e); p.update(|p| p.example = v); saved.set(false); }/>" Example (shows an “Example” badge)"</label>
            </div>
            <label class="admin-field">
                <span class="mono">"Summary (one or two sentences)"</span>
                <textarea rows="3" prop:value=move || p.with(|p| p.summary.clone()) on:input=move |e| { let v = event_target_value(&e); p.update(|p| p.summary = v); saved.set(false); }></textarea>
            </label>

            <fieldset class="admin-set">
                <legend class="mono">"Cover image"</legend>
                <div class="admin-media">
                    {move || p.with(|p| (!p.cover.src.is_empty()).then(|| view! {
                        <img class="admin-media__preview" src=p.cover.src.clone() alt=""/>
                    }))}
                    <div class="admin-media__fields">
                        <label class="btn btn--small admin-upload">
                            {move || if p.with(|p| p.cover.src.is_empty()) { "Upload cover" } else { "Replace cover" }}
                            <input type="file" accept="image/jpeg,image/png,image/webp" on:change=on_cover/>
                        </label>
                        {field("Alt text (what the image shows)", |p| p.cover.alt.clone(), |p, v| p.cover.alt = v)}
                    </div>
                </div>
            </fieldset>

            <fieldset class="admin-set">
                <legend class="mono">"Preview video (plays on hover)"</legend>
                <div class="admin-media__fields">
                    {move || p.with(|p| p.preview_video.clone()).map(|v| view! {
                        <p class="mono admin-muted">{v}" "
                            <button type="button" class="admin-link" on:click=move |_| { p.update(|p| p.preview_video = None); saved.set(false); }>"Remove"</button>
                        </p>
                    })}
                    <label class="btn btn--small admin-upload">
                        "Upload video"
                        <input type="file" accept="video/mp4,video/webm" on:change=on_video/>
                    </label>
                </div>
            </fieldset>

            <label class="admin-field">
                <span class="mono">"Case study (Markdown)"</span>
                <textarea rows="14" class="admin-code" prop:value=move || p.with(|p| p.body_md.clone()) on:input=move |e| { let v = event_target_value(&e); p.update(|p| p.body_md = v); saved.set(false); }></textarea>
            </label>

            <fieldset class="admin-set">
                <legend class="mono">"Gallery"</legend>
                <ol class="admin-gallery">
                    {move || p.with(|p| p.gallery.clone()).into_iter().enumerate().map(|(i, m)| view! {
                        <li class="admin-gallery__item">
                            {if m.video {
                                view! { <video src=m.src.clone() muted=true class="admin-media__preview"></video> }.into_any()
                            } else {
                                view! { <img src=m.src.clone() alt="" class="admin-media__preview"/> }.into_any()
                            }}
                            <input type="text" placeholder="Alt text" prop:value=m.alt.clone() on:input=move |e| { let v = event_target_value(&e); p.update(|p| p.gallery[i].alt = v); saved.set(false); }/>
                            <input type="text" placeholder="Caption" prop:value=m.caption.clone() on:input=move |e| { let v = event_target_value(&e); p.update(|p| p.gallery[i].caption = v); saved.set(false); }/>
                            <button type="button" class="admin-icon admin-danger" aria-label="Remove" on:click=move |_| { p.update(|p| { p.gallery.remove(i); }); saved.set(false); }>"✕"</button>
                        </li>
                    }).collect_view()}
                </ol>
                <label class="btn btn--small admin-upload">
                    "Add image or video"
                    <input type="file" accept="image/jpeg,image/png,image/webp,video/mp4,video/webm" on:change=on_gallery/>
                </label>
            </fieldset>

            <div class="admin-grid">
                <label class="admin-field">
                    <span class="mono">"Links (one per line: Label | URL)"</span>
                    <textarea rows="4" class="admin-code" prop:value=move || links.get() on:input=move |e| { links.set(event_target_value(&e)); saved.set(false); }></textarea>
                </label>
                <label class="admin-field">
                    <span class="mono">"Credits (one per line: Role | Name)"</span>
                    <textarea rows="4" class="admin-code" prop:value=move || credits.get() on:input=move |e| { credits.set(event_target_value(&e)); saved.set(false); }></textarea>
                </label>
            </div>

            <ErrorNote error=error/>
            <div class="admin-actions">
                <button type="submit" class="btn btn--accent" disabled=move || save.pending().get() || busy.get()>
                    {move || if busy.get() { "Uploading…" } else if save.pending().get() { "Saving…" } else { "Save project" }}
                </button>
                {move || saved.get().then(|| view! { <span class="mono admin-ok" role="status">"Saved. It's live."</span> })}
            </div>
        </form>
    }
}

// ---- experience ----------------------------------------------------------------

const KINDS: [(&str, &str); 4] = [
    ("job", "Role"),
    ("freelance", "Freelance"),
    ("education", "Education"),
    ("milestone", "Milestone"),
];

fn blank_role() -> Role {
    Role {
        title: String::new(),
        org: String::new(),
        kind: "job".into(),
        start: String::new(),
        end: String::new(),
        location: String::new(),
        summary: String::new(),
        placeholder: false,
    }
}

#[component]
pub fn AdminExperience() -> impl IntoView {
    let list = Resource::new(|| (), |_| list_roles());
    let done = Callback::new(move |_| list.refetch());
    let adding = RwSignal::new(false);
    view! {
        <header class="admin-head">
            <h1 class="display admin-head__title">"Experience"</h1>
            <div class="admin-head__actions">
                <button class="btn btn--accent btn--small" on:click=move |_| adding.set(true)>"Add a role"</button>
            </div>
        </header>
        <p class="admin-muted">"Shown on the Experience page in this order."</p>
        <Show when=move || adding.get()>
            <RoleForm id=None role=blank_role() done=Callback::new(move |_| { adding.set(false); list.refetch(); })/>
        </Show>
        <Transition fallback=|| view! { <p class="admin-loading mono">"Loading…"</p> }>
            {move || list.get().map(|r| match r {
                Err(e) => view! { <p class="admin-note admin-note--error">{err_text(&e)}</p> }.into_any(),
                Ok(rows) => {
                    let n = rows.len();
                    view! {
                        <ol class="admin-list">
                            {rows.into_iter().enumerate().map(|(i, row)| view! {
                                <li class="admin-card">
                                    <div class="admin-card__head">
                                        <strong>{row.data.title.clone()}" · "{row.data.org.clone()}</strong>
                                        <RowControls kind=Kind::Roles id=row.id first={i == 0} last={i + 1 == n} done/>
                                    </div>
                                    <details>
                                        <summary class="mono">"Edit"</summary>
                                        <RoleForm id=Some(row.id) role=row.data done/>
                                    </details>
                                </li>
                            }).collect_view()}
                        </ol>
                    }.into_any()
                }
            })}
        </Transition>
    }
}

#[component]
fn RoleForm(id: Option<i64>, role: Role, done: Callback<()>) -> impl IntoView {
    let r = RwSignal::new(role);
    let error = RwSignal::new(None::<String>);
    let save = Action::new_local(move |role: &Role| {
        let role = role.clone();
        async move { save_role(id, role).await }
    });
    Effect::new(move |_| {
        if let Some(res) = save.value().get() {
            match res {
                Ok(_) => {
                    error.set(None);
                    done.run(());
                }
                Err(e) => error.set(Some(err_text(&e))),
            }
        }
    });
    let text = move |label: &'static str, get: fn(&Role) -> String, set: fn(&mut Role, String)| {
        view! {
            <label class="admin-field">
                <span class="mono">{label}</span>
                <input type="text" prop:value=move || r.with(get) on:input=move |e| { let v = event_target_value(&e); r.update(|x| set(x, v)); }/>
            </label>
        }
    };
    view! {
        <form class="admin-form admin-form--inline" on:submit=move |ev| { ev.prevent_default(); save.dispatch(r.get_untracked()); }>
            <div class="admin-grid">
                {text("Title", |x| x.title.clone(), |x, v| x.title = v)}
                {text("Organisation", |x| x.org.clone(), |x, v| x.org = v)}
                <label class="admin-field">
                    <span class="mono">"Kind"</span>
                    <select on:change=move |e| { let v = event_target_value(&e); r.update(|x| x.kind = v); }>
                        {KINDS.iter().map(|(id, label)| view! {
                            <option value=*id selected=move || r.with(|x| x.kind == *id)>{*label}</option>
                        }).collect_view()}
                    </select>
                </label>
                {text("Location", |x| x.location.clone(), |x, v| x.location = v)}
                {text("Start (e.g. 2023 or Mar 2023)", |x| x.start.clone(), |x, v| x.start = v)}
                {text("End (empty = now)", |x| x.end.clone(), |x, v| x.end = v)}
            </div>
            <label class="admin-field">
                <span class="mono">"Summary"</span>
                <textarea rows="3" prop:value=move || r.with(|x| x.summary.clone()) on:input=move |e| { let v = event_target_value(&e); r.update(|x| x.summary = v); }></textarea>
            </label>
            <ErrorNote error=error/>
            <button type="submit" class="btn btn--accent btn--small" disabled=move || save.pending().get()>
                {move || if save.pending().get() { "Saving…" } else if id.is_some() { "Save role" } else { "Add role" }}
            </button>
        </form>
    }
}

// ---- skills ----------------------------------------------------------------------

const LEVELS: [(&str, &str); 3] = [
    ("daily", "Daily driver"),
    ("confident", "Confident"),
    ("learning", "Learning"),
];

#[component]
pub fn AdminSkills() -> impl IntoView {
    let list = Resource::new(|| (), |_| list_skill_groups());
    let done = Callback::new(move |_| list.refetch());
    let adding = RwSignal::new(false);
    view! {
        <header class="admin-head">
            <h1 class="display admin-head__title">"Skills"</h1>
            <div class="admin-head__actions">
                <button class="btn btn--accent btn--small" on:click=move |_| adding.set(true)>"Add a group"</button>
            </div>
        </header>
        <p class="admin-muted">"Groups of skills, tallied on the Skills page: daily = 5 marks, confident = 3, learning = 1."</p>
        <Show when=move || adding.get()>
            <GroupForm id=None group=SkillGroup { name: String::new(), blurb: String::new(), skills: vec![], placeholder: false }
                done=Callback::new(move |_| { adding.set(false); list.refetch(); })/>
        </Show>
        <Transition fallback=|| view! { <p class="admin-loading mono">"Loading…"</p> }>
            {move || list.get().map(|r| match r {
                Err(e) => view! { <p class="admin-note admin-note--error">{err_text(&e)}</p> }.into_any(),
                Ok(rows) => {
                    let n = rows.len();
                    view! {
                        <ol class="admin-list">
                            {rows.into_iter().enumerate().map(|(i, row)| view! {
                                <li class="admin-card">
                                    <div class="admin-card__head">
                                        <strong>{row.data.name.clone()}</strong>
                                        <span class="mono admin-muted">{format!("{} skills", row.data.skills.len())}</span>
                                        <RowControls kind=Kind::SkillGroups id=row.id first={i == 0} last={i + 1 == n} done/>
                                    </div>
                                    <details>
                                        <summary class="mono">"Edit"</summary>
                                        <GroupForm id=Some(row.id) group=row.data done/>
                                    </details>
                                </li>
                            }).collect_view()}
                        </ol>
                    }.into_any()
                }
            })}
        </Transition>
    }
}

#[component]
fn GroupForm(id: Option<i64>, group: SkillGroup, done: Callback<()>) -> impl IntoView {
    let g = RwSignal::new(group);
    let error = RwSignal::new(None::<String>);
    let save = Action::new_local(move |group: &SkillGroup| {
        let group = group.clone();
        async move { save_skill_group(id, group).await }
    });
    Effect::new(move |_| {
        if let Some(res) = save.value().get() {
            match res {
                Ok(_) => {
                    error.set(None);
                    done.run(());
                }
                Err(e) => error.set(Some(err_text(&e))),
            }
        }
    });
    view! {
        <form class="admin-form admin-form--inline" on:submit=move |ev| {
            ev.prevent_default();
            let mut out = g.get_untracked();
            out.skills.retain(|s| !s.name.trim().is_empty());
            save.dispatch(out);
        }>
            <div class="admin-grid">
                <label class="admin-field">
                    <span class="mono">"Group name"</span>
                    <input type="text" prop:value=move || g.with(|x| x.name.clone()) on:input=move |e| { let v = event_target_value(&e); g.update(|x| x.name = v); }/>
                </label>
                <label class="admin-field">
                    <span class="mono">"Blurb"</span>
                    <input type="text" prop:value=move || g.with(|x| x.blurb.clone()) on:input=move |e| { let v = event_target_value(&e); g.update(|x| x.blurb = v); }/>
                </label>
            </div>
            <ol class="admin-skills">
                {move || g.with(|x| x.skills.clone()).into_iter().enumerate().map(|(i, s)| view! {
                    <li class="admin-skill">
                        <input type="text" placeholder="Skill" prop:value=s.name.clone() on:input=move |e| { let v = event_target_value(&e); g.update(|x| x.skills[i].name = v); }/>
                        <select on:change=move |e| { let v = event_target_value(&e); g.update(|x| x.skills[i].level = v); }>
                            {LEVELS.iter().map(|(id, label)| {
                                let sel = s.level == *id;
                                view! { <option value=*id selected=sel>{*label}</option> }
                            }).collect_view()}
                        </select>
                        <input type="text" placeholder="Note (optional)" prop:value=s.note.clone() on:input=move |e| { let v = event_target_value(&e); g.update(|x| x.skills[i].note = v); }/>
                        <button type="button" class="admin-icon admin-danger" aria-label="Remove skill" on:click=move |_| g.update(|x| { x.skills.remove(i); })>"✕"</button>
                    </li>
                }).collect_view()}
            </ol>
            <button type="button" class="btn btn--small" on:click=move |_| g.update(|x| x.skills.push(Skill { name: String::new(), level: "confident".into(), note: String::new() }))>"Add a skill"</button>
            <ErrorNote error=error/>
            <button type="submit" class="btn btn--accent btn--small" disabled=move || save.pending().get()>
                {move || if save.pending().get() { "Saving…" } else if id.is_some() { "Save group" } else { "Add group" }}
            </button>
        </form>
    }
}
