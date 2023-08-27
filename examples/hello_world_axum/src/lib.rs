#![deny(warnings)]

pub mod app;
#[cfg(feature = "ssr")]
pub mod fileserv;
pub mod i18n;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::App;
    use leptos::*;
    console_error_panic_hook::set_once();

    leptos::mount_to_body(move |cx| {
        view! { cx, <App/> }
    });
}
