use leptos::prelude::*;

/// Infinite horizontal strip. The track holds the items twice and slides by
/// exactly half its width, so the loop is seamless. Pure CSS: it costs nothing
/// on the main thread and stops under `prefers-reduced-motion`.
#[component]
pub fn Marquee(items: &'static [&'static str]) -> impl IntoView {
    let run = |hidden: bool| {
        view! {
            <ul class="marquee__group" aria-hidden=hidden.then_some("true")>
                {items.iter().map(|i| view! { <li>{*i}</li> }).collect_view()}
            </ul>
        }
    };
    view! {
        <div class="marquee">
            <div class="marquee__track">{run(false)}{run(true)}</div>
        </div>
    }
}
