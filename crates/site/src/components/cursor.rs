//! Custom cursor: a dot that tracks the pointer exactly and a ring that trails
//! it. Elements opt into states with `data-cursor="view|drag|shoot|link"`;
//! plain links and buttons get `link` automatically.
//!
//! Only fine pointers (mouse, trackpad) get the custom cursor. On touch it never
//! starts, and the native cursor is hidden only after it has started (via the
//! `has-cursor` class), so a failed hydration never leaves you cursorless.

use leptos::html;
use leptos::prelude::*;

/// Lets non-DOM content (the 3D room) drive the cursor state.
#[derive(Clone, Copy)]
pub struct CursorOverride(pub RwSignal<Option<&'static str>>);

#[component]
pub fn Cursor() -> impl IntoView {
    let dot = NodeRef::<html::Div>::new();
    let ring = NodeRef::<html::Div>::new();
    let state = RwSignal::new("default");
    let over = use_context::<CursorOverride>().map(|c| c.0);
    let shown = move || over.and_then(|o| o.get()).unwrap_or_else(|| state.get());

    Effect::new(move |started: Option<bool>| {
        if started == Some(true) {
            return true;
        }
        match (dot.get(), ring.get()) {
            (Some(d), Some(r)) => {
                imp::start(d.into(), r.into(), state);
                true
            }
            _ => false,
        }
    });

    let label = move || match shown() {
        "view" => "VIEW",
        "drag" => "DRAG",
        "shoot" => "SHOOT",
        _ => "",
    };

    view! {
        <div class="cursor" aria-hidden="true" data-state=shown>
            <div class="cursor__dot" node_ref=dot></div>
            <div class="cursor__ring" node_ref=ring>
                <span class="cursor__label mono">{label}</span>
            </div>
        </div>
    }
}

mod imp {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    use leptos::prelude::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::prelude::Closure;
    use web_sys::{HtmlElement, PointerEvent};

    type Frame = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

    /// Fraction of the remaining distance the ring covers each frame.
    const TRAIL: f64 = 0.18;

    pub fn start(dot: HtmlElement, ring: HtmlElement, state: RwSignal<&'static str>) {
        let window = web_sys::window().expect("window");
        let matches = |q: &str| {
            window
                .match_media(q)
                .ok()
                .flatten()
                .is_some_and(|m| m.matches())
        };
        if !matches("(pointer: fine)") {
            return;
        }
        let ease = if matches("(prefers-reduced-motion: reduce)") {
            1.0
        } else {
            TRAIL
        };

        let document = window.document().expect("document");
        let root = document.document_element().expect("<html>");
        let _ = root.class_list().add_1("has-cursor");

        let target = Rc::new(Cell::new((-100.0, -100.0)));
        let current = Rc::new(Cell::new((-100.0, -100.0)));
        let running = Rc::new(Cell::new(false));

        // Ring follow loop. It only runs while the ring is catching up, so an
        // idle cursor costs zero frames.
        let tick: Frame = Rc::new(RefCell::new(None));
        {
            let (tick2, target, current, running) = (
                tick.clone(),
                target.clone(),
                current.clone(),
                running.clone(),
            );
            *tick.borrow_mut() = Some(Closure::new(move || {
                let (tx, ty) = target.get();
                let (cx, cy) = current.get();
                let (nx, ny) = (cx + (tx - cx) * ease, cy + (ty - cy) * ease);
                current.set((nx, ny));
                set_pos(&ring, nx, ny);
                if (tx - nx).abs() + (ty - ny).abs() > 0.1 {
                    request_frame(&tick2);
                } else {
                    running.set(false);
                }
            }));
        }

        let on_move = Closure::<dyn FnMut(PointerEvent)>::new(move |e: PointerEvent| {
            let p = (e.client_x() as f64, e.client_y() as f64);
            target.set(p);
            set_pos(&dot, p.0, p.1);
            if !running.replace(true) {
                request_frame(&tick);
            }
        });

        let on_over = Closure::<dyn FnMut(PointerEvent)>::new(move |e: PointerEvent| {
            let next = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                .map(|el| cursor_state(&el))
                .unwrap_or("default");
            if state.get_untracked() != next {
                state.set(next);
            }
        });

        let pressed = |on: bool| {
            let root = root.clone();
            Closure::<dyn FnMut(PointerEvent)>::new(move |_| {
                let _ = root.class_list().toggle_with_force("cursor-pressed", on);
            })
        };

        for (ev, cb) in [
            ("pointermove", on_move),
            ("pointerover", on_over),
            ("pointerdown", pressed(true)),
            ("pointerup", pressed(false)),
        ] {
            let _ = document.add_event_listener_with_callback(ev, cb.as_ref().unchecked_ref());
            // The cursor lives as long as the page, so its listeners do too.
            cb.forget();
        }
    }

    fn cursor_state(el: &web_sys::Element) -> &'static str {
        if let Ok(Some(hit)) = el.closest("[data-cursor]") {
            return match hit.get_attribute("data-cursor").as_deref() {
                Some("view") => "view",
                Some("drag") => "drag",
                Some("shoot") => "shoot",
                // The 3D room reports its own hover state via CursorOverride.
                Some("room") => "default",
                _ => "link",
            };
        }
        match el.closest("a, button, [role=button], label, select") {
            Ok(Some(_)) => "link",
            _ => "default",
        }
    }

    fn set_pos(el: &HtmlElement, x: f64, y: f64) {
        let _ = el
            .style()
            .set_property("transform", &format!("translate3d({x}px, {y}px, 0)"));
    }

    fn request_frame(cb: &Frame) {
        if let Some(cb) = cb.borrow().as_ref() {
            let _ = web_sys::window()
                .expect("window")
                .request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }
}
