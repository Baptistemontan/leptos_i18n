#![cfg_attr(feature = "nightly", feature(fn_traits))]
#![cfg_attr(feature = "nightly", feature(unboxed_closures))]
#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![deny(warnings)]
//! # About Leptos i18n
//!
//! Leptos i18n is library to help with translations in a Leptos application
//!
//! It loads the translations at compile time and provides checks on translation keys, interpolation keys and the selected locale.
//!
//!
//! Explore our [Examples](https://github.com/Baptistemontan/leptos_i18n/tree/master/examples) to see it in action.
//!
//! # Learning by Example
//!
//! If you want to see what Leptos i18n is capable of, check out
//! the [examples](https://github.com/Baptistemontan/leptos_i18n/tree/master/examples):
//! - [`hello_world_actix`](https://github.com/Baptistemontan/leptos_i18n/tree/master/examples/hello_world_actix) is a simple example
//!     to showcase the syntax and file structure to easily incorporate translations in you application using the actix backend
//! - [`hello_world_axum`](https://github.com/Baptistemontan/leptos_i18n/tree/master/examples/hello_world_axum) is like the actix hello world example
//!     but uses axum as the backend, it showcases that the code you will write with this library will be the same using actix or axum as a backend.
//! - [`counter`](https://github.com/Baptistemontan/leptos_i18n/tree/master/examples/counter) is the classic
//!     counter example, showing how you can interpolate values in the translations and switch locale without full reload.
//! - [`counter_plurals`](https://github.com/Baptistemontan/leptos_i18n/tree/master/examples/counter_plurals) is like the `counter` example
//!     but show how you can use plurals to display different texts based on a count.
//! - [`namespaces`](https://github.com/Baptistemontan/leptos_i18n/tree/master/examples/namespaces) show how to break down your translations
//!     in multiple namespaces to avoid big files
//!
//! Details on how to run each example can be found in its README.
//!
//! # In depth documentation
//!
//! You can find the [book](https://github.com/Baptistemontan/leptos_i18n/tree/master/docs/book) on the github repo.
//!
//! # Feature Flags
//! - `cookie` (*Default*): Enable this feature to set a cookie on the client to remember the last locale set.
//! - `json_files` (*Default*): Enable this feature if you use JSON files for declaring your locales.
//! - `hydrate`: Enable this feature when building for the client with hydratation.
//! - `actix`: Enable this feature when building for the server with actix as the backend (can't be enabled with the `axum` feature).
//! - `axum`: Enable this feature when building for the server with axum as the backend (can't be enabled with the `actix` feature).
//! - `csr`: Enable this feature when building for the client with CSR.
//! - `serde`: Enabling this feature implement `serde::Serialize` and `serde::Deserialize` for the locale enum.
//! - `debug_interpolations`: Enable the macros to generate code to emit a warning if a key is supplied twice in interpolations and a better compilation error when a key is missing.
//! - `suppress_key_warnings`: Disable the warning emission of the `load_locales!()` macro when some keys are missing or ignored.
//! - `yaml_files`: Enable this feature if you use YAML files for declaring your locales (can't be used with `json_files`).
//! - `nightly`: On `nightly` Rust, enables the function-call syntax on the i18n context to get/set the locale.
//!
//! # A Simple Counter
//!
//! `Cargo.toml`:
//!
//! ```toml
//! [package.metadata.leptos-i18n]
//! default = "en"
//! locales = ["en", "fr"]
//! ```
//!
//! `./locales/en.json`:
//!
//! ```json
//! {
//!     "click_to_change_lang": "Click to change language",
//!     "click_count": "You clicked {{ count }} times",
//!     "click_to_inc": "Click to increment the counter"
//! }
//! ```
//!
//! `./locales/fr.json`:
//!
//! ```json
//! {
//!     "click_to_change_lang": "Cliquez pour changez de langue",
//!     "click_count": "Vous avez cliqué {{ count }} fois",
//!     "click_to_inc": "Cliquez pour incrémenter le compteur"
//! }
//! ```
//!
//!
//! ```rust,ignore
//! leptos_i18n::load_locales!();
//! use i18n::*; // `i18n` module created by the macro above
//! use leptos::*;
//!
//! #[component]
//! pub fn App() -> impl IntoView {
//!     leptos_meta::provide_meta_context();
//!
//!     let i18n = provide_i18n_context();
//!
//!     let on_switch = move |_| {
//!         let new_lang = match i18n.get_locale() {
//!             Locale::en => Locale::fr,
//!             Locale::fr => Locale::en,
//!         };
//!         i18n.set_locale(new_lang);
//!     };
//!
//!     view! {
//!         <button on:click=on_switch>{t!(i18n, click_to_change_lang)}</button>
//!         <Counter />
//!     }
//! }
//!
//! #[component]
//! fn Counter() -> impl IntoView {
//!     let i18n = use_i18n();
//!
//!     let (counter, set_counter) = create_signal( 0);
//!
//!     let inc = move |_| set_counter.update(|count| *count += 1);
//!
//!     let count = move || counter.get();
//!
//!     view! {
//!         <p>{t!(i18n, click_count, count)}</p>
//!         // Equivalent to:
//!         // <p>{t!(i18n, click_count, count = count)}</p>
//!         // Could also be wrote:
//!         // <p>{t!(i18n, click_count, count = move || counter.get())}</p>
//!         <button on:click=inc>{t!(i18n, click_to_inc)}</button>
//!     }
//! }
//! ```

mod context;
mod fetch_locale;
mod locale_traits;
#[cfg(feature = "ssr")]
mod server;

pub use locale_traits::*;

pub use context::{provide_i18n_context, use_i18n_context, I18nContext};

pub use leptos_i18n_macro::{load_locales, t, td};

#[cfg(feature = "interpolate_display")]
pub use leptos_i18n_macro::td_string;

#[doc(hidden)]
pub mod __private {
    pub use super::locale_traits::BuildStr;
}

#[cfg(all(
    feature = "cookie",
    any(feature = "ssr", feature = "hydrate", feature = "csr")
))]
pub(crate) const COOKIE_PREFERED_LANG: &str = "i18n_pref_locale";

#[cfg(all(feature = "cookie", any(feature = "hydrate", feature = "csr")))]
pub(crate) fn get_html_document() -> Option<web_sys::HtmlDocument> {
    use wasm_bindgen::JsCast;
    leptos::document().dyn_into::<web_sys::HtmlDocument>().ok()
}
