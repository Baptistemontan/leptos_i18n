#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![deny(warnings)]
//! This crate contain anything related to routing for the `leptos_i18n` crate.

mod components;
mod routing;

pub use components::I18nRoute;

/// Create a localized path (one or more static segments) based on a locale.
///
/// ```rust, ignore
/// <Route path=i18n_path!(Locale, routes.some_path) view=.. />
/// ```
#[macro_export]
macro_rules! i18n_path {
    ($scope:ty, $first_key:ident $(.$keys:ident)*) => {
        const {
            $crate::__private::make_i18n_path(
                $crate::__private::leptos_i18n::key!(scope = $scope, $first_key $(.$keys)*)
            )
        }
    };
}

#[doc(hidden)]
pub mod __private {
    pub use crate::routing::make_i18n_path;
    pub use leptos_i18n;
}
