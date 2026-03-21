#![deny(warnings)]

pub mod app;
include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));

fn main() {
    use app::App;
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| leptos::view! { <App /> })
}
