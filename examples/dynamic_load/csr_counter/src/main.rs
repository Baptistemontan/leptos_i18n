#![deny(warnings)]

pub mod app;
leptos_i18n::load_locales!();

fn main() {
    use app::App;
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| leptos::view! { <App /> })
}
