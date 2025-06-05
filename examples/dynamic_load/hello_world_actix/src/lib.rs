#![deny(warnings)]

pub mod app;

include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::App;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
