#![deny(warnings)]

pub mod app;
leptos_i18n::load_locales!();

include!(concat!(env!("OUT_DIR"), "/baked_data/mod.rs"));

#[derive(leptos_i18n::custom_provider::IcuDataProvider)]
pub struct BakedProvider;
impl_data_provider!(BakedProvider);

fn main() {
    use app::App;
    leptos_i18n::custom_provider::set_icu_data_provider(BakedProvider);
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| leptos::view! { <App /> })
}
