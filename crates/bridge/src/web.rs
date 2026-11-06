//! Browser transport: `CustomEvent`s on `window` carrying flat string objects.

use js_sys::{Object, Reflect};
use wasm_bindgen::{JsCast, JsValue, prelude::Closure};
use web_sys::{CustomEvent, CustomEventInit};

use crate::{COMMAND, Command, ROOM_EVENT, RoomEvent};

fn dispatch(name: &str, kind: &str, fields: &[(&str, String)]) {
    let detail = Object::new();
    let _ = Reflect::set(&detail, &"type".into(), &kind.into());
    for (k, v) in fields {
        let _ = Reflect::set(&detail, &(*k).into(), &v.as_str().into());
    }
    let init = CustomEventInit::new();
    init.set_detail(&detail);
    if let (Some(window), Ok(ev)) = (
        web_sys::window(),
        CustomEvent::new_with_event_init_dict(name, &init),
    ) {
        let _ = window.dispatch_event(&ev);
    }
}

fn field(detail: &JsValue, name: &str) -> Option<String> {
    Reflect::get(detail, &name.into()).ok()?.as_string()
}

/// Listen on `window` for `name`, handing each event's detail to `f`. The
/// listener lives for the rest of the page (both apps are page-lifetime).
fn listen(name: &str, mut f: impl FnMut(JsValue) + 'static) {
    let cb = Closure::<dyn FnMut(web_sys::Event)>::new(move |e: web_sys::Event| {
        if let Some(ce) = e.dyn_ref::<CustomEvent>() {
            f(ce.detail());
        }
    });
    if let Some(window) = web_sys::window() {
        let _ = window.add_event_listener_with_callback(name, cb.as_ref().unchecked_ref());
    }
    cb.forget();
}

pub fn send_command(cmd: &Command) {
    let (kind, value) = cmd.encode();
    dispatch(COMMAND, kind, &[("value", value)]);
}

pub fn send_room_event(ev: &RoomEvent) {
    let (kind, fields) = ev.encode();
    dispatch(ROOM_EVENT, kind, &fields);
}

pub fn on_command(mut f: impl FnMut(Command) + 'static) {
    listen(COMMAND, move |d| {
        let kind = field(&d, "type").unwrap_or_default();
        let value = field(&d, "value").unwrap_or_default();
        if let Some(c) = Command::decode(&kind, &value) {
            f(c);
        }
    });
}

pub fn on_room_event(mut f: impl FnMut(RoomEvent) + 'static) {
    listen(ROOM_EVENT, move |d| {
        let kind = field(&d, "type").unwrap_or_default();
        if let Some(e) = RoomEvent::decode(&kind, |n| field(&d, n)) {
            f(e);
        }
    });
}
