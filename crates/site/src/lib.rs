// Leptos view types nest deeply; the release (wasm) build needs headroom.
#![recursion_limit = "512"]

pub mod admin;
pub mod app;
pub mod components;
pub mod config;
pub mod data;
pub mod pages;
pub mod room;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    data::install_from_page();
    leptos::mount::hydrate_body(app::App);
}
