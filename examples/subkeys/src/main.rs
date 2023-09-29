#![deny(warnings)]

pub mod app;
leptos_i18n::load_locales!();

use app::App;

fn main() {
    leptos::mount_to_body(|| leptos::view! { <App /> })
}
