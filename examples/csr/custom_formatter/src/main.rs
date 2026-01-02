#![deny(warnings)]

use std::fmt::Display;

pub mod app;
include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));

pub trait ToDisplayFn: 'static + Clone + Send + Sync {
    type Value: Display;
    fn to_value(&self) -> Self::Value;
}

impl<T: Display, F> ToDisplayFn for F
where
    F: Fn() -> T + 'static + Clone + Send + Sync,
{
    type Value = T;
    fn to_value(&self) -> Self::Value {
        self()
    }
}

fn main() {
    use app::App;
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| leptos::view! { <App/> });
}
