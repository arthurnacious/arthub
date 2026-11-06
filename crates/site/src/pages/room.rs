use bridge::{Command, Hotspot, Mode, Quality};
use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_query_map};

use crate::components::cursor::CursorOverride;
use crate::components::seo::Seo;
use crate::room::{Load, RoomLoader, RoomSlot, use_room};

/// Clickable areas over the static room image (1600×1000, overview camera),
/// as % of its width/height. Projected from the scene's object positions.
pub const FALLBACK_SPOTS: &[(&str, &str, f32, f32)] = &[
    ("Pool table — about me", "/about", 50.0, 63.4),
    ("Bar — contact", "/contact", 93.0, 46.0),
    ("Arcade — all work", "/work", 72.0, 34.1),
];

/// Objects the overview camera can't see, linked underneath the image.
const OFFSCREEN: &[(&str, &str)] = &[
    ("Chalkboard — skills", "/skills"),
    ("Dartboard — lab", "/lab"),
    ("Exit", "/"),
];

/// The room as a picture with links on it: no WebGL2, or reduced motion.
#[component]
pub fn RoomFallback() -> impl IntoView {
    view! {
        <div class="room-fallback">
            <img src="/room/fallback.jpg" alt="The ARTHUB game room: a dark lounge with a pool table under a hanging lamp, a row of arcade cabinets, a neon ARTHUB sign, a jukebox and a bar." />
            <ul class="room-fallback__spots">
                {FALLBACK_SPOTS.iter().map(|(label, href, x, y)| view! {
                    <li style:left=format!("{x}%") style:top=format!("{y}%")>
                        <A href=*href attr:class="room-fallback__spot">
                            <span class="mono">{*label}</span>
                        </A>
                    </li>
                }).collect_view()}
            </ul>
            <ul class="room-fallback__more">
                {crate::data::site().featured().map(|p| view! {
                    <li><A href=format!("/work/{}", p.slug) attr:class="btn btn--small">{p.title}</A></li>
                }).collect_view()}
                {OFFSCREEN.iter().map(|(label, href)| view! {
                    <li><A href=*href attr:class="btn btn--small">{*label}</A></li>
                }).collect_view()}
            </ul>
        </div>
    }
}

#[component]
pub fn Room() -> impl IntoView {
    let state = use_room();
    let query = use_query_map();

    // The URL decides the viewpoint: deep links and Back both land here.
    Effect::new(move |_| {
        let h = query
            .read()
            .get("spot")
            .and_then(|s| Hotspot::from_id(&s))
            .unwrap_or(Hotspot::Overview);
        state.send(Command::FocusHotspot(h));
    });

    #[cfg(target_arch = "wasm32")]
    {
        use leptos::ev;
        let root = || {
            web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .document_element()
                .unwrap()
        };
        let _ = root().class_list().add_1("room-open");
        let keys = window_event_listener(ev::keydown, move |e| {
            if e.key() != "Escape" {
                return;
            }
            if state.showreel.get_untracked() {
                state.showreel.set(false);
            } else if state.pool_active.get_untracked() {
                state.send(Command::PoolExit);
            } else {
                state.send(Command::FocusHotspot(Hotspot::Overview));
            }
        });
        on_cleanup(move || {
            keys.remove();
            let _ = root().class_list().remove_1("room-open");
        });
    }

    let unsupported = move || state.supported.get() == Some(false);

    view! {
        <Seo title="Game Room" description="A Rust/WASM game lounge: shoot pool, play the arcade, and find the work on the walls."/>
        <h1 class="visually-hidden">"The ARTHUB game room"</h1>
        <div class="room-page">
            <Show when=unsupported fallback=move || view! {
                <RoomSlot mode=Mode::Full/>
                <RoomLoader/>
                <RoomHud/>
            }>
                <RoomFallback/>
                <div class="room-hud__top">
                    <span class="mono">"Static view — your browser or motion settings skip the 3D room"</span>
                    <A href="/" attr:class="btn btn--small">"Exit"</A>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn RoomHud() -> impl IntoView {
    let state = use_room();
    let tip = NodeRef::<leptos::html::Div>::new();

    // Tie the site cursor to what's under it in 3D.
    if let Some(CursorOverride(cursor)) = use_context::<CursorOverride>() {
        Effect::new(move |_| {
            cursor.set(if state.pool_active.get() {
                Some("shoot")
            } else if state.hover.get().is_some() {
                Some("link")
            } else {
                None
            });
        });
        on_cleanup(move || cursor.set(None));
    }

    #[cfg(target_arch = "wasm32")]
    {
        use leptos::ev;
        let follow = window_event_listener(ev::pointermove, move |e| {
            if let Some(t) = tip.get_untracked() {
                let _ = web_sys::HtmlElement::style(&t).set_property(
                    "transform",
                    &format!(
                        "translate3d({}px, {}px, 0)",
                        e.client_x() + 22,
                        e.client_y() + 18
                    ),
                );
            }
        });
        on_cleanup(move || follow.remove());
    }

    let ready = move || state.load.get() == Load::Ready;

    view! {
        <div class="room-hud" class:room-hud--ready=ready>
            <div class="room-hud__top">
                <span class="room-hud__mark">"ARTHUB"<span class="mono">" / Game Room"</span></span>
                <div class="room-hud__controls">
                    <button class="btn btn--small" aria-pressed=move || (!state.muted.get()).to_string()
                        on:click=move |_| {
                            let m = !state.muted.get_untracked();
                            state.muted.set(m);
                            state.send(Command::Mute(m));
                        }>
                        {move || if state.muted.get() { "Sound off" } else { "Sound on" }}
                    </button>
                    <div class="room-hud__quality" role="group" aria-label="Graphics quality">
                        {Quality::ALL.into_iter().map(|q| view! {
                            <button class="mono" aria-pressed=move || (state.quality.get() == q).to_string()
                                on:click=move |_| { state.quality.set(q); state.send(Command::SetQuality(q)); }>
                                {q.id()}
                            </button>
                        }).collect_view()}
                    </div>
                    <button class="btn btn--small" on:click=move |_| state.send(Command::ResetView)>"Reset view"</button>
                    <A href="/" attr:class="btn btn--small btn--accent">"Exit"</A>
                </div>
            </div>

            <nav class="room-hud__spots" aria-label="Room hotspots">
                <ol>
                    {Hotspot::ALL.into_iter().enumerate().map(|(i, h)| view! {
                        <li>
                            <button aria-current=move || (state.hotspot.get() == h).then_some("location")
                                on:click=move |_| state.send(Command::FocusHotspot(h))>
                                <span class="mono">{format!("{:02}", i + 1)}</span>
                                {h.label()}
                            </button>
                        </li>
                    }).collect_view()}
                </ol>
            </nav>

            <Show when=move || state.pool_active.get()>
                <section class="room-hud__pool" aria-label="Pool">
                    <p class="room-hud__score display">
                        {move || state.score.get().0}
                        <span class="mono">" balls potted · " {move || state.score.get().1} " shots"</span>
                    </p>
                    <div class="room-hud__power" aria-hidden="true">
                        <div style:transform=move || format!("scaleX({})", state.power.get())></div>
                    </div>
                    <p class="mono room-hud__hint">"Drag back from the cue ball. Let go to shoot."</p>
                    <div class="room-hud__row">
                        <button class="btn btn--small" on:click=move |_| state.send(Command::PoolReset)>"Re-rack"</button>
                        <button class="btn btn--small" on:click=move |_| state.send(Command::PoolExit)>"Leave table"</button>
                    </div>
                </section>
            </Show>

            <p class="room-hud__help mono" aria-hidden="true">
                {move || if state.pool_active.get() {
                    "Drag from the cue ball to shoot · drag elsewhere to look · Esc leaves"
                } else {
                    "Click things · double-click to focus · R resets · Esc steps back"
                }}
            </p>

            // One-time hint; CSS picks the pointer or touch wording.
            <div class="room-hint" class:room-hint--gone=move || !state.hint.get() aria-hidden="true">
                <p class="mono room-hint__pointer">"Drag to look around · Right-drag to move · Scroll to zoom"</p>
                <p class="mono room-hint__touch">"Drag to look · Two fingers to move and zoom"</p>
            </div>

            <div class="room-tip mono" node_ref=tip aria-hidden="true" class:room-tip--on=move || state.hover.get().is_some()>
                {move || state.hover.get().unwrap_or_default()}
            </div>
            <div class="visually-hidden" aria-live="polite">
                {move || format!("Viewing {}", state.hotspot.get().label())}
            </div>
        </div>

        <Show when=move || state.showreel.get()>
            <div class="showreel" role="dialog" aria-modal="true" aria-label="Showreel">
                <div class="showreel__frame">
                    <p class="display display--md">"Showreel"</p>
                    <p class="mono">"Drop a video at /room/showreel.mp4 and it plays here."</p>
                    <video src="/room/showreel.mp4" controls autoplay muted playsinline></video>
                    <button class="btn" on:click=move |_| state.showreel.set(false)>"Close"</button>
                </div>
            </div>
        </Show>
    }
}
