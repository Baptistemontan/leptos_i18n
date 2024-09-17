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
//! # Learning by examples
//!
//! If you want to see what Leptos i18n is capable of, check out
//! the [examples](https://github.com/Baptistemontan/leptos_i18n/tree/master/examples).
//!
//! Details on how to run each example can be found in its README.
//!
//! # In depth documentation
//!
//! You can find the [book](https://baptistemontan.github.io/leptos_i18n) on the github repo.
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
//! ### Rust code:
//!
//! ```rust,ignore
//! leptos_i18n::load_locales!();
//! use i18n::*; // `i18n` module created by the macro above
//! use leptos::prelude::*;
//!
//! #[component]
//! pub fn App() -> impl IntoView {
//!     leptos_meta::provide_meta_context();
//!
//!     view! {
//!         <I18nContextProvider>
//!             <Counter />
//!             <SwitchLang />
//!         </I18nContextProvider>
//!     }
//! }
//!
//! #[component]
//! fn SwitchLang() -> impl IntoView {
//!     let i18n = use_i18n();
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
//!     }
//! }
//!
//! #[component]
//! fn Counter() -> impl IntoView {
//!     let i18n = use_i18n();
//!
//!     let (counter, set_counter) = signal( 0);
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

pub mod context;
mod fetch_locale;
mod langid;
mod locale_traits;
mod macro_helpers;
mod routing;
mod scopes;
mod static_lock;

pub mod display;

pub use macro_helpers::formatting;

pub use locale_traits::{Locale, LocaleKeys};

pub use context::{use_i18n_context, I18nContext};

#[allow(deprecated)]
pub use context::provide_i18n_context;

pub use leptos_i18n_macro::{
    load_locales, scope_i18n, scope_locale, t, t_display, t_string, td, td_display, td_string, tu,
    tu_display, tu_string, use_i18n_scoped,
};
pub use scopes::{ConstScope, Scope};

#[doc(hidden)]
pub mod __private {
    pub use crate::formatting::get_plural_rules;
    pub use crate::macro_helpers::*;
    pub use crate::routing::{i18n_routing, BaseRoute, I18nNestedRoute};
    pub use crate::static_lock::*;
    pub use icu::locid;
    pub use leptos_i18n_macro::declare_locales;
}

/// Reexports of backend libraries, mostly about formatting.
pub mod reexports {
    pub use fixed_decimal;
    pub use icu;
    pub use leptos;
    pub use leptos_router;
    pub use serde;
    pub use typed_builder;
    pub use wasm_bindgen;
}

/// Utility macro for using reactive translations in a non reactive component when using islands.
///
/// ```rust, ignore
/// use crate::i18n::*;
///
/// #[component]
/// fn App() -> impl IntoView {
///     view! {
///         <I18nContextProvider>
///             <p>{ti!(HelloWorld, hello_world)}</p> // <- using `t!` here wouldn't work
///         </I18nContextProvider>
///     }
/// }
/// ```
///
/// The code above would not work because the component is only rendered on the server and never runs on the client, so using `t!` would make the translation unreactive.
///
/// `ti!` wrapp the call to `t!` in an isolated island, making it run on the client.
///
/// The drawbacks are that this macro is really simple, so it don't add args to the island, making it impossible to use variable in your translation.
/// I mean ACTUAL variables, it is totally ok to use literals or refer to global variable, as long as you are not trying to capture outer variables.
///
/// ```rust, ignore
/// ti!(SayName, say_name, name = "John"); // totally OK
///
/// static MY_NUM: usize = 0;
/// ti!(Counter, counter, count = MY_NUM); // totally OK
///
/// let foo: String = get_my_string();
/// ti!(RenderMyStruct, render_my_struct, <bar> = |children| view! {
///     <div>
///         <p>{foo}</p>
///         {children}
///     </div>
/// }); // NOT OK -> tries to capture outer scope.
/// ```
///
/// Also note that this macro does NOT take the context as the first argument, instead it takes the name for the generated island.
///
/// If you need to pass variable args, you will have to make yourself an island that take those args.
#[cfg(feature = "experimental-islands")]
#[macro_export]
macro_rules! ti {
    ($island_name: ident, $($tt:tt)*) => {
        {
            mod inner {
                use super::*;
                $crate::make_i18n_island!($island_name, $($tt)*);
            }

            || view! { <inner::$island_name /> }
        }
    };
}

/// Utility Macro to generate an island for a translation key.
///
/// One of the limitation of `ti!` is that if you use the same key multiple time, you must still give a unique name and creating duplicate code.
///
/// This macro mitigate that by creating the island and then you can use it multiple time.
///
/// ```rust, ignore
/// use crate::i18n::*;
///
/// #[component]
/// fn App() -> impl IntoView {
///     view! {
///         <I18nContextProvider>
///             <p>
///                 {ti!(HelloWorld, hello_world)}
///                 {ti!(HelloWorld, hello_world)}
///                 {ti!(HelloWorld, hello_world)}
///                 {ti!(HelloWorld, hello_world)}
///             </p>
///         </I18nContextProvider>
///     }
/// }
/// ```
///
/// The code above won't compile as the `HelloWorld` island is created multiple time, and `wasm_bindgen` don't like duplicate symbols.
///
/// Do this instead:
///
/// ```rust, ignore
/// use crate::i18n::*;
///
/// leptos_i18n::make_i18n_island(HelloWorld, hello_world);
///
/// #[component]
/// fn App() -> impl IntoView {
///     view! {
///         <I18nContextProvider>
///             <p>
///                 <HelloWorld />
///                 <HelloWorld />
///                 <HelloWorld />
///                 <HelloWorld />
///             </p>
///         </I18nContextProvider>
///     }
/// }
/// ```
#[cfg(feature = "experimental-islands")]
#[macro_export]
macro_rules! make_i18n_island {
    ($island_name: ident, $($tt:tt)*) => {
        #[island]
        pub fn $island_name() -> impl IntoView {
            t!(use_i18n(), $($tt)*)
        }
    };
}
