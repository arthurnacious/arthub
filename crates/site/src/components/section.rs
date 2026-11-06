use leptos::prelude::*;

/// The `01 / WORK` label plus thin rule used to open every section.
#[component]
pub fn SectionLabel(num: &'static str, text: &'static str) -> impl IntoView {
    view! {
        <div class="section-label mono">
            <span>{num}</span>
            <span class="section-label__slash">"/"</span>
            <span>{text}</span>
        </div>
    }
}

/// Shared top-of-page block: label, oversized title, short lede.
#[component]
pub fn PageHeader(
    num: &'static str,
    label: &'static str,
    #[prop(into)] title: TextProp,
    #[prop(optional)] lede: Option<&'static str>,
) -> impl IntoView {
    view! {
        <header class="page-header">
            <SectionLabel num text=label/>
            <h1 class="display page-header__title">{move || title.get().to_string()}</h1>
            {lede.map(|l| view! { <p class="lede">{l}</p> })}
        </header>
    }
}
