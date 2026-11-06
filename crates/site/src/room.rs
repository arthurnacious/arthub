//! Hosting the Bevy game room inside the Leptos site.
//!
//! * One canvas for the whole page. It is created on first use and moved
//!   between "slots" (the hero preview, the /room page) as routes change, so
//!   Bevy, which can only start once per page, keeps running across navigation.
//!   With no slot on screen it is parked off-screen and the room is paused.
//! * The room bundle (`/room/bundle/game_room_bg.wasm`, several MB) loads only
//!   when a slot asks for it, with real byte-level progress.
//! * [`RoomState`] mirrors what the room reports over the bridge.

use bridge::{Command, Hotspot, Mode, Quality};
use leptos::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Load {
    Idle,
    /// Download progress, 0..=1 (0 while the size is unknown).
    Loading(f32),
    /// Downloaded and started; waiting for the first frames.
    Starting,
    Ready,
    Failed(String),
}

/// Everything the DOM needs to know about the room. Lives in context.
#[derive(Clone, Copy)]
pub struct RoomState {
    pub load: RwSignal<Load>,
    /// `None` until checked on the client; `false` = show the static fallback.
    pub supported: RwSignal<Option<bool>>,
    pub hotspot: RwSignal<Hotspot>,
    pub hover: RwSignal<Option<String>>,
    pub muted: RwSignal<bool>,
    pub music: RwSignal<bool>,
    pub quality: RwSignal<Quality>,
    pub pool_active: RwSignal<bool>,
    pub score: RwSignal<(u32, u32)>,
    pub power: RwSignal<f32>,
    pub showreel: RwSignal<bool>,
    /// Show the one-time controls hint (until the visitor first moves the camera).
    pub hint: RwSignal<bool>,
    pending: StoredValue<Vec<Command>, LocalStorage>,
}

impl RoomState {
    fn new() -> Self {
        Self {
            load: RwSignal::new(Load::Idle),
            supported: RwSignal::new(None),
            hotspot: RwSignal::new(Hotspot::Overview),
            hover: RwSignal::new(None),
            muted: RwSignal::new(true),
            music: RwSignal::new(false),
            quality: RwSignal::new(Quality::Medium),
            pool_active: RwSignal::new(false),
            score: RwSignal::new((0, 0)),
            power: RwSignal::new(0.0),
            showreel: RwSignal::new(false),
            hint: RwSignal::new(true),
            pending: StoredValue::new_local(Vec::new()),
        }
    }

    /// Send a command now, or queue it until the room is listening.
    pub fn send(&self, cmd: Command) {
        if self.load.get_untracked() == Load::Ready {
            dispatch(&cmd);
        } else {
            self.pending.update_value(|p| p.push(cmd));
        }
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn flush(&self) {
        for c in self
            .pending
            .try_update_value(std::mem::take)
            .unwrap_or_default()
        {
            dispatch(&c);
        }
    }
}

fn dispatch(cmd: &Command) {
    #[cfg(target_arch = "wasm32")]
    bridge::web::send_command(cmd);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = cmd;
}

pub fn use_room() -> RoomState {
    expect_context::<RoomState>()
}

/// Provide room state and wire the room → site bridge. Must sit inside the
/// `<Router>` so room clicks can navigate.
#[component]
pub fn RoomBridge() -> impl IntoView {
    let state = RoomState::new();
    provide_context(state);

    #[cfg(target_arch = "wasm32")]
    {
        let navigate = leptos_router::hooks::use_navigate();
        Effect::new(move |done: Option<()>| {
            if done.is_some() {
                return;
            }
            state.supported.set(Some(client::supported()));
            client::install_visibility(state);
            let navigate = navigate.clone();
            bridge::web::on_room_event(move |ev| client::on_event(state, &navigate, ev));
        });
    }
}

/// Mount point for the shared canvas.
#[component]
pub fn RoomSlot(mode: Mode) -> impl IntoView {
    let el = NodeRef::<leptos::html::Div>::new();
    let state = use_room();

    #[cfg(target_arch = "wasm32")]
    {
        // Keep our own handle: the NodeRef may already be cleared at cleanup.
        let mounted = StoredValue::new_local(None::<web_sys::Element>);
        Effect::new(move |done: Option<bool>| {
            if done == Some(true) {
                return true;
            }
            let (Some(slot), Some(true)) = (el.get(), state.supported.get()) else {
                return false;
            };
            let slot: web_sys::Element = slot.into();
            mounted.set_value(Some(slot.clone()));
            client::mount(state, slot, mode);
            true
        });
        on_cleanup(move || {
            if let Some(slot) = mounted.try_get_value().flatten() {
                client::unmount(state, &slot);
            }
        });
    }
    let _ = (mode, state);

    view! { <div class="room-slot" node_ref=el></div> }
}

/// Byte-level progress, styled like the site loader.
#[component]
pub fn RoomLoader() -> impl IntoView {
    let state = use_room();
    let pct = move || match state.load.get() {
        Load::Loading(p) => format!("{:02}", (p * 100.0).floor() as u32),
        Load::Starting => "99".into(),
        _ => "00".into(),
    };
    let label = move || match state.load.get() {
        Load::Starting => "Racking the balls".to_string(),
        Load::Failed(e) => format!("Couldn't load the room: {e}"),
        _ => "Loading the game room".to_string(),
    };
    view! {
        <Show when=move || state.load.get() != Load::Ready>
            <div class="room-loader" role="status" aria-live="polite">
                <span class="room-loader__pct display">{pct}<span class="room-loader__unit">"%"</span></span>
                <span class="mono">{label}</span>
                <div class="room-loader__bar"><div class="room-loader__fill" style:transform=move || {
                    let p = match state.load.get() { Load::Loading(p) => p, Load::Starting | Load::Ready => 1.0, _ => 0.0 };
                    format!("scaleX({p})")
                }></div></div>
            </div>
        </Show>
    }
}

#[cfg(target_arch = "wasm32")]
mod client {
    use std::cell::RefCell;

    use bridge::{Command, Hotspot, Mode, Quality, RoomEvent};
    use leptos::prelude::*;
    use leptos::task::spawn_local;
    use leptos_router::NavigateOptions;
    use wasm_bindgen::prelude::*;
    use web_sys::{Element, HtmlCanvasElement, IntersectionObserver, IntersectionObserverEntry};

    use super::{Load, RoomState};

    #[wasm_bindgen(inline_js = r#"
        // Browsers start audio suspended until a gesture. The room creates its
        // AudioContext on boot, so remember every context and resume them all
        // on the next pointer/key press.
        function patchAudio() {
            if (window.__arthubAudio || !window.AudioContext) return;
            const all = window.__arthubAudio = [];
            const Base = window.AudioContext;
            window.AudioContext = class extends Base { constructor(...a) { super(...a); all.push(this); } };
            const resume = () => all.forEach(c => c.state !== 'running' && c.resume());
            for (const ev of ['pointerdown', 'keydown', 'touchend']) document.addEventListener(ev, resume, true);
        }
        export async function load_room(base, on_progress) {
            patchAudio();
            let total = 0;
            try { total = (await (await fetch(base + 'manifest.json')).json()).wasm_bytes || 0; } catch (_) {}
            const res = await fetch(base + 'game_room_bg.wasm');
            if (!res.ok) throw new Error('HTTP ' + res.status);
            total = total || +res.headers.get('content-length') || 0;
            const reader = res.body.getReader();
            const chunks = []; let got = 0;
            for (;;) {
                const { done, value } = await reader.read();
                if (done) break;
                chunks.push(value); got += value.length;
                on_progress(total ? Math.min(got / total, 0.99) : 0);
            }
            const bytes = new Uint8Array(got); let o = 0;
            for (const c of chunks) { bytes.set(c, o); o += c.length; }
            const mod = await import(base + 'game_room.js');
            await mod.default({ module_or_path: bytes });
            window.__arthubRoom = mod; // debug handle (e.g. camera_pose())
            return mod;
        }
        export function start_room(mod, mode, featured) {
            try { mod.start(mode, featured); }
            catch (e) { if (!String(e).includes('Using exceptions for control flow')) throw e; }
        }
    "#)]
    extern "C" {
        #[wasm_bindgen(catch)]
        async fn load_room(
            base: &str,
            on_progress: &Closure<dyn FnMut(f64)>,
        ) -> Result<JsValue, JsValue>;
        #[wasm_bindgen(catch)]
        fn start_room(module: &JsValue, mode: &str, featured: &str) -> Result<(), JsValue>;
    }

    thread_local! {
        static CANVAS: RefCell<Option<HtmlCanvasElement>> = const { RefCell::new(None) };
        static OBSERVER: RefCell<Option<IntersectionObserver>> = const { RefCell::new(None) };
        static SLOT_VISIBLE: RefCell<bool> = const { RefCell::new(false) };
        /// The canvas is in the full-screen room: it owns wheel, pinch and
        /// right-click. In the hero preview the page keeps them.
        static OWNS_GESTURES: RefCell<bool> = const { RefCell::new(false) };
    }

    fn document() -> web_sys::Document {
        web_sys::window().unwrap().document().unwrap()
    }

    pub fn supported() -> bool {
        let w = web_sys::window().unwrap();
        let reduced = w
            .match_media("(prefers-reduced-motion: reduce)")
            .ok()
            .flatten()
            .is_some_and(|m| m.matches());
        let webgl2 = document()
            .create_element("canvas")
            .ok()
            .and_then(|c| c.dyn_into::<HtmlCanvasElement>().ok())
            .and_then(|c| c.get_context("webgl2").ok().flatten())
            .is_some();
        webgl2 && !reduced
    }

    fn canvas() -> HtmlCanvasElement {
        CANVAS.with(|c| {
            c.borrow_mut()
                .get_or_insert_with(|| {
                    let el: HtmlCanvasElement = document()
                        .create_element("canvas")
                        .unwrap()
                        .unchecked_into();
                    el.set_id(bridge::CANVAS_ID);
                    let _ = el.set_attribute("aria-label", "Interactive 3D game room");
                    let _ = el.set_attribute("data-cursor", "room");
                    guard_gestures(&el);
                    el
                })
                .clone()
        })
    }

    fn park() {
        let doc = document();
        let park = doc.get_element_by_id("room-park").unwrap_or_else(|| {
            let p = doc.create_element("div").unwrap();
            p.set_id("room-park");
            let _ = p.set_attribute("aria-hidden", "true");
            let _ = doc.body().unwrap().append_child(&p);
            p
        });
        let _ = park.append_child(&canvas());
    }

    pub fn mount(state: RoomState, slot: Element, mode: Mode) {
        let _ = slot.append_child(&canvas());
        OWNS_GESTURES.with(|o| *o.borrow_mut() = mode == Mode::Full);
        if mode == Mode::Full && hint_seen() {
            state.hint.set(false);
        }
        state.send(Command::SetMode(mode));
        // The full room takes the keyboard (arrows orbit, R resets…).
        set_focusable();
        if mode == Mode::Full {
            let _ = canvas().focus();
        }
        // Visibility drives both the lazy load and pause/resume.
        observe(state, &slot, mode);
    }

    /// Over the room canvas, the wheel zooms the camera and pinch zooms the
    /// camera, not the page; right-drag pans instead of opening a menu.
    /// Listeners are non-passive so they may cancel the browser's default.
    fn guard_gestures(el: &HtmlCanvasElement) {
        let opts = web_sys::AddEventListenerOptions::new();
        opts.set_passive(false);
        let stop = |always: bool| {
            Closure::<dyn FnMut(web_sys::Event)>::new(move |e: web_sys::Event| {
                if always || OWNS_GESTURES.with(|o| *o.borrow()) {
                    e.prevent_default();
                }
            })
        };
        for (name, always) in [
            ("wheel", false),
            ("gesturestart", false),
            ("gesturechange", false),
            ("touchmove", false),
            ("contextmenu", true),
        ] {
            let cb = stop(always);
            let _ = el.add_event_listener_with_callback_and_add_event_listener_options(
                name,
                cb.as_ref().unchecked_ref(),
                &opts,
            );
            cb.forget();
        }
    }

    const HINT_KEY: &str = "arthub-room-hint-seen";

    fn hint_seen() -> bool {
        // Storage can be unavailable (private mode, blocked): then just show it.
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item(HINT_KEY).ok().flatten())
            .is_some()
    }

    fn remember_hint_seen() {
        if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
            let _ = s.set_item(HINT_KEY, "1");
        }
    }

    /// Only the full-screen room is a tab stop. As the hero backdrop the
    /// canvas is decoration (the "Enter the game room" button is the way in).
    fn set_focusable() {
        let full = OWNS_GESTURES.with(|o| *o.borrow());
        let c = canvas();
        let _ = c.set_attribute("tabindex", if full { "0" } else { "-1" });
        if !full && document().active_element().as_ref() == Some(c.as_ref()) {
            let _ = c.blur();
        }
    }

    pub fn unmount(state: RoomState, slot: &Element) {
        OWNS_GESTURES.with(|o| *o.borrow_mut() = false);
        set_focusable();
        OBSERVER.with(|o| {
            if let Some(o) = o.borrow_mut().take() {
                o.disconnect();
            }
        });
        let c = canvas();
        // A newer slot may already have claimed the canvas.
        if c.parent_element().as_ref() == Some(slot) {
            park();
            SLOT_VISIBLE.with(|v| *v.borrow_mut() = false);
            state.send(Command::Pause);
        }
    }

    /// Pause while the slot is scrolled out of view.
    fn observe(state: RoomState, slot: &Element, mode: Mode) {
        let cb = Closure::<dyn FnMut(js_sys::Array)>::new(move |entries: js_sys::Array| {
            let visible = entries
                .iter()
                .filter_map(|e| e.dyn_into::<IntersectionObserverEntry>().ok())
                .any(|e| e.is_intersecting());
            SLOT_VISIBLE.with(|v| *v.borrow_mut() = visible);
            if visible && state.load.get_untracked() == Load::Idle {
                load(state, mode);
            }
            state.send(if visible {
                Command::Resume
            } else {
                Command::Pause
            });
        });
        if let Ok(o) = IntersectionObserver::new(cb.as_ref().unchecked_ref()) {
            o.observe(slot);
            OBSERVER.with(|slot| {
                if let Some(old) = slot.borrow_mut().replace(o) {
                    old.disconnect();
                }
            });
        }
        cb.forget();
    }

    pub fn install_visibility(state: RoomState) {
        let doc = document();
        let cb = Closure::<dyn FnMut()>::new(move || {
            let hidden = document().hidden();
            let visible_slot = SLOT_VISIBLE.with(|v| *v.borrow());
            if hidden {
                state.send(Command::Pause);
            } else if visible_slot {
                state.send(Command::Resume);
            }
        });
        let _ =
            doc.add_event_listener_with_callback("visibilitychange", cb.as_ref().unchecked_ref());
        cb.forget();
    }

    fn load(state: RoomState, mode: Mode) {
        state.load.set(Load::Loading(0.0));
        // Small screens start on the low tier; the watchdog takes it from there.
        let coarse = web_sys::window()
            .unwrap()
            .match_media("(pointer: coarse)")
            .ok()
            .flatten()
            .is_some_and(|m| m.matches());
        if coarse {
            state.send(Command::SetQuality(Quality::Low));
            state.quality.set(Quality::Low);
        }
        spawn_local(async move {
            let progress = Closure::<dyn FnMut(f64)>::new(move |p: f64| {
                state.load.set(Load::Loading(p as f32))
            });
            match load_room("/room/bundle/", &progress).await {
                Ok(module) => {
                    state.load.set(Load::Starting);
                    if let Err(e) = start_room(&module, mode.id(), &featured_for_room()) {
                        state.load.set(Load::Failed(format!("{e:?}")));
                    }
                }
                Err(e) => {
                    let msg = e.as_string().unwrap_or_else(|| {
                        js_sys::Reflect::get(&e, &"message".into())
                            .ok()
                            .and_then(|m| m.as_string())
                            .unwrap_or_else(|| "network error".into())
                    });
                    state.load.set(Load::Failed(msg));
                }
            }
            drop(progress);
        });
    }

    /// Featured projects from `content/`, in the room's wire format.
    fn featured_for_room() -> String {
        let list: Vec<bridge::FeaturedProject> = crate::data::site()
            .featured()
            .map(|p| bridge::FeaturedProject {
                slug: p.slug.into(),
                title: p.title.into(),
                color: p.accent,
                cover: Some(p.cover_asset.into()),
            })
            .collect();
        bridge::encode_featured(&list)
    }

    pub fn on_event(state: RoomState, navigate: &impl Fn(&str, NavigateOptions), ev: RoomEvent) {
        match ev {
            RoomEvent::Ready => {
                // Winit has just given the canvas tabindex="0"; undo that
                // unless it's the full room.
                set_focusable();
                state.load.set(Load::Ready);
                state.flush();
            }
            RoomEvent::Navigate { path } => navigate(&path, NavigateOptions::default()),
            RoomEvent::Exit => navigate("/", NavigateOptions::default()),
            RoomEvent::Hotspot(h) => {
                state.hotspot.set(h);
                sync_url(navigate, h);
            }
            RoomEvent::Hover { label } => state.hover.set(label),
            RoomEvent::ShowreelOpen => state.showreel.set(true),
            RoomEvent::Music { playing } => {
                state.music.set(playing);
                if playing {
                    state.muted.set(false);
                }
            }
            RoomEvent::PoolActive(a) => {
                state.pool_active.set(a);
                if !a {
                    state.power.set(0.0);
                }
            }
            RoomEvent::PoolScore { potted, shots } => state.score.set((potted, shots)),
            RoomEvent::PoolPower(p) => state.power.set(p),
            RoomEvent::Quality(q) => state.quality.set(q),
            RoomEvent::Interacted => {
                state.hint.set(false);
                remember_hint_seen();
            }
        }
    }

    /// Keep `/room?spot=…` in step with the camera. Leaving the overview pushes
    /// a history entry (so Back returns to it); moving between hotspots replaces.
    fn sync_url(navigate: &impl Fn(&str, NavigateOptions), h: Hotspot) {
        let loc = web_sys::window().unwrap().location();
        if loc.pathname().ok().as_deref() != Some("/room") {
            return;
        }
        let search = loc.search().unwrap_or_default();
        let current = search
            .trim_start_matches('?')
            .split('&')
            .find_map(|kv| kv.strip_prefix("spot="))
            .and_then(Hotspot::from_id)
            .unwrap_or(Hotspot::Overview);
        if current == h {
            return;
        }
        let url = if h == Hotspot::Overview {
            "/room".to_string()
        } else {
            format!("/room?spot={}", h.id())
        };
        let replace = current != Hotspot::Overview || h == Hotspot::Overview;
        navigate(
            &url,
            NavigateOptions {
                replace,
                scroll: false,
                ..Default::default()
            },
        );
    }
}
