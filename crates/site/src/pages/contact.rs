//! `/contact`: the one thing on the site that must never fail.
//!
//! Server-rendered as a normal `<form method="post" action="/api/contact">`
//! with HTML constraints, so it works before (or without) the WASM bundle.
//! Once hydrated, submission goes through `fetch`: same rules checked
//! client-side first, inline errors, a busy button, and the result in place.
//! A plain POST lands back here with `?sent=1` or `?error=<code>`.

use contact::{
    BUDGETS, Field, HONEYPOT, MESSAGE_MAX, MESSAGE_MIN, NAME_MAX, PROJECT_TYPES, Submission,
};
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::components::section::PageHeader;
use crate::components::seo::Seo;
use crate::data::site;

#[derive(Clone, Debug, PartialEq)]
enum Status {
    Idle,
    Sending,
    Sent,
    /// Error code from the server (`send`, `rate`, `fast`, `invalid`, `network`).
    Failed(String),
}

fn failure_text(code: &str) -> &'static str {
    match code {
        "fast" => "That was quick! Give it a few seconds, then send again.",
        "rate" => "That's a lot of messages in an hour. Try again later, or just email me.",
        "invalid" => "Something in the form needs fixing. Check the fields and try again.",
        _ => "Couldn't send that just now. My fault, not yours.",
    }
}

#[component]
pub fn Contact() -> impl IntoView {
    let about = site().about;
    let query = use_query_map();
    // A no-JS POST comes back with the outcome in the URL.
    let initial = query.with_untracked(|q| {
        if q.get("sent").is_some() {
            Status::Sent
        } else if let Some(code) = q.get("error") {
            Status::Failed(code)
        } else {
            Status::Idle
        }
    });
    let status = RwSignal::new(initial);

    let name = RwSignal::new(String::new());
    let email = RwSignal::new(String::new());
    let project = RwSignal::new(String::new());
    let budget = RwSignal::new(String::new());
    let message = RwSignal::new(String::new());
    let honeypot = RwSignal::new(String::new());
    let errors = RwSignal::new(Vec::<(Field, String)>::new());
    let attempted = RwSignal::new(false);
    // Once hydrated we validate ourselves (inline, in the site's voice), so
    // switch off the browser's bubbles. Before that they're the safety net.
    let hydrated = RwSignal::new(false);
    Effect::new(move |_| hydrated.set(true));

    let current = move || Submission {
        name: name.get_untracked(),
        email: email.get_untracked(),
        project_type: project.get_untracked(),
        budget: budget.get_untracked(),
        message: message.get_untracked(),
    };
    // After the first attempt, re-check as they type so errors clear.
    let revalidate = move || {
        if attempted.get_untracked() {
            errors.set(match contact::validate(&current()) {
                Ok(_) => vec![],
                Err(e) => e.into_iter().map(|(f, m)| (f, m.to_string())).collect(),
            });
        }
    };
    let error_for = move |f: Field| {
        move || errors.with(|e| e.iter().find(|(x, _)| *x == f).map(|(_, m)| m.clone()))
    };

    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| client::fetch_token());

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        // Hydrated: handle it here. (Without WASM, the browser posts the form.)
        ev.prevent_default();
        if status.get_untracked() == Status::Sending {
            return;
        }
        attempted.set(true);
        match contact::validate(&current()) {
            Err(e) => {
                let first = e.first().map(|(f, _)| f.key());
                errors.set(e.into_iter().map(|(f, m)| (f, m.to_string())).collect());
                #[cfg(target_arch = "wasm32")]
                if let Some(id) = first {
                    client::focus(&format!("contact-{id}"));
                }
                let _ = first;
            }
            Ok(valid) => {
                errors.set(vec![]);
                status.set(Status::Sending);
                #[cfg(target_arch = "wasm32")]
                leptos::task::spawn_local(async move {
                    match client::post(&valid, &honeypot.get_untracked()).await {
                        Ok(()) => status.set(Status::Sent),
                        Err(client::Failure::Fields(f)) => {
                            errors.set(f);
                            status.set(Status::Idle);
                        }
                        Err(client::Failure::Code(c)) => status.set(Status::Failed(c)),
                    }
                });
                let _ = valid;
            }
        }
    };

    let reset = move |_| {
        for s in [name, email, project, budget, message] {
            s.set(String::new());
        }
        attempted.set(false);
        status.set(Status::Idle);
    };

    let field_class = move |f: Field| {
        move || {
            if error_for(f)().is_some() {
                "field field--error"
            } else {
                "field"
            }
        }
    };

    view! {
        <Seo title="Contact" description="Got a project? Tell me about it."/>
        <PageHeader num="06" label="Contact" title="Rack 'em up." lede="Got a project? Let's play a round."/>

        <div class="contact" id="contact-form">
            <aside class="contact__aside">
                <p class="mono">"Prefer email?"</p>
                <a class="contact__email" href=format!("mailto:{}", about.email)>{about.email}</a>
                <p class="mono contact__reply">"I reply within " {about.reply_time} "."</p>
                {about.placeholder.then(|| view! { <span class="badge mono">"Placeholder email"</span> })}
            </aside>

            <Show when=move || status.get() == Status::Sent fallback=move || view! {
                <form class="contact__form" method="post" action="/api/contact" on:submit=on_submit novalidate=move || hydrated.get()>
                    {move || match status.get() {
                        Status::Failed(code) => Some(view! {
                            <div class="contact__alert" role="alert">
                                <p>{failure_text(&code)}</p>
                                <p class="mono">"Email me directly: "<a href=format!("mailto:{}", about.email)>{about.email}</a></p>
                            </div>
                        }),
                        _ => None,
                    }}

                    <div class=field_class(Field::Name)>
                        <label for="contact-name">"Your name"</label>
                        <input id="contact-name" name="name" type="text" autocomplete="name" required maxlength=NAME_MAX
                            prop:value=name on:input=move |e| { name.set(event_target_value(&e)); revalidate(); }
                            aria-invalid=move || error_for(Field::Name)().is_some().to_string()
                            aria-describedby="contact-name-error"/>
                        <p class="field__error" id="contact-name-error">{error_for(Field::Name)}</p>
                    </div>

                    <div class=field_class(Field::Email)>
                        <label for="contact-email">"Email"</label>
                        <input id="contact-email" name="email" type="email" autocomplete="email" required maxlength="254"
                            prop:value=email on:input=move |e| { email.set(event_target_value(&e)); revalidate(); }
                            aria-invalid=move || error_for(Field::Email)().is_some().to_string()
                            aria-describedby="contact-email-error"/>
                        <p class="field__error" id="contact-email-error">{error_for(Field::Email)}</p>
                    </div>

                    <div class="field-row">
                        <div class=field_class(Field::ProjectType)>
                            <label for="contact-project_type">"What are you after?"</label>
                            <select id="contact-project_type" name="project_type" required
                                on:change=move |e| { project.set(event_target_value(&e)); revalidate(); }
                                aria-invalid=move || error_for(Field::ProjectType)().is_some().to_string()
                                aria-describedby="contact-project_type-error">
                                <option value="" selected=move || project.get().is_empty()>"Choose one"</option>
                                {PROJECT_TYPES.iter().map(|p| view! {
                                    <option value=*p selected=move || project.get() == *p>{*p}</option>
                                }).collect_view()}
                            </select>
                            <p class="field__error" id="contact-project_type-error">{error_for(Field::ProjectType)}</p>
                        </div>
                        <div class=field_class(Field::Budget)>
                            <label for="contact-budget">"Budget"</label>
                            <select id="contact-budget" name="budget" required
                                on:change=move |e| { budget.set(event_target_value(&e)); revalidate(); }
                                aria-invalid=move || error_for(Field::Budget)().is_some().to_string()
                                aria-describedby="contact-budget-error">
                                <option value="" selected=move || budget.get().is_empty()>"Choose one"</option>
                                {BUDGETS.iter().map(|b| view! {
                                    <option value=*b selected=move || budget.get() == *b>{*b}</option>
                                }).collect_view()}
                            </select>
                            <p class="field__error" id="contact-budget-error">{error_for(Field::Budget)}</p>
                        </div>
                    </div>

                    <div class=field_class(Field::Message)>
                        <label for="contact-message">"Tell me about it"</label>
                        <textarea id="contact-message" name="message" rows="7" required minlength=MESSAGE_MIN maxlength=MESSAGE_MAX
                            prop:value=message on:input=move |e| { message.set(event_target_value(&e)); revalidate(); }
                            aria-invalid=move || error_for(Field::Message)().is_some().to_string()
                            aria-describedby="contact-message-error contact-message-count"></textarea>
                        <p class="field__error" id="contact-message-error">{error_for(Field::Message)}</p>
                        <p class="mono field__count" id="contact-message-count">
                            {move || message.with(|m| m.trim().chars().count())} " / " {MESSAGE_MAX}
                        </p>
                    </div>

                    // Honeypot: hidden from people (and screen readers), irresistible to bots.
                    <div class="hp" aria-hidden="true">
                        <label>"Leave this empty"
                            <input name=HONEYPOT type="text" tabindex="-1" autocomplete="off"
                                prop:value=honeypot on:input=move |e| honeypot.set(event_target_value(&e))/>
                        </label>
                    </div>

                    <button class="btn btn--accent contact__send" type="submit"
                        disabled=move || status.get() == Status::Sending
                        aria-busy=move || (status.get() == Status::Sending).to_string()>
                        {move || if status.get() == Status::Sending { "Lining up the shot…" } else { "Send it" }}
                    </button>
                </form>
            }>
                <div class="contact__sent" role="status">
                    <p class="display display--md">"Shot sunk."</p>
                    <p class="lede">"Message received. I'll get back to you within " {about.reply_time} "."</p>
                    <button class="btn" type="button" on:click=reset>"Send another"</button>
                </div>
            </Show>
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
mod client {
    use contact::{Field, Submission};
    use js_sys::{Reflect, encode_uri_component};
    use wasm_bindgen::{JsCast, JsValue};
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{Headers, RequestCredentials, RequestInit, Response};

    pub enum Failure {
        Fields(Vec<(Field, String)>),
        Code(String),
    }

    pub fn focus(id: &str) {
        if let Some(el) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.get_element_by_id(id))
            .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
        {
            let _ = el.focus();
        }
    }

    /// Ask the server for a form token (sets the time-on-form cookie).
    pub fn fetch_token() {
        if let Some(w) = web_sys::window() {
            let init = RequestInit::new();
            init.set_credentials(RequestCredentials::SameOrigin);
            let _ = w.fetch_with_str_and_init("/api/contact/token", &init);
        }
    }

    fn get(v: &JsValue, k: &str) -> JsValue {
        Reflect::get(v, &k.into()).unwrap_or(JsValue::UNDEFINED)
    }

    pub async fn post(s: &Submission, honeypot: &str) -> Result<(), Failure> {
        let enc = |v: &str| String::from(encode_uri_component(v));
        let body = [
            ("name", s.name.as_str()),
            ("email", s.email.as_str()),
            ("project_type", s.project_type.as_str()),
            ("budget", s.budget.as_str()),
            ("message", s.message.as_str()),
            (contact::HONEYPOT, honeypot),
        ]
        .iter()
        .map(|(k, v)| format!("{k}={}", enc(v)))
        .collect::<Vec<_>>()
        .join("&");

        let headers = Headers::new().map_err(|_| Failure::Code("network".into()))?;
        let _ = headers.set("Content-Type", "application/x-www-form-urlencoded");
        let _ = headers.set("Accept", "application/json");
        let init = RequestInit::new();
        init.set_method("POST");
        init.set_headers(&headers);
        init.set_body(&JsValue::from_str(&body));
        init.set_credentials(RequestCredentials::SameOrigin);

        let window = web_sys::window().ok_or_else(|| Failure::Code("network".into()))?;
        let res: Response = JsFuture::from(window.fetch_with_str_and_init("/api/contact", &init))
            .await
            .map_err(|_| Failure::Code("network".into()))?
            .unchecked_into();
        let json = match res.json() {
            Ok(p) => JsFuture::from(p).await.unwrap_or(JsValue::NULL),
            Err(_) => JsValue::NULL,
        };
        if get(&json, "ok").as_bool() == Some(true) {
            return Ok(());
        }
        let errs = get(&json, "errors");
        if errs.is_object() {
            let fields: Vec<(Field, String)> = Field::ALL
                .iter()
                .filter_map(|f| get(&errs, f.key()).as_string().map(|m| (*f, m)))
                .collect();
            if !fields.is_empty() {
                return Err(Failure::Fields(fields));
            }
        }
        Err(Failure::Code(
            get(&json, "error")
                .as_string()
                .unwrap_or_else(|| "send".into()),
        ))
    }
}
