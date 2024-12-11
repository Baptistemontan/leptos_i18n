#![deny(missing_docs)]
#![forbid(unsafe_code)]
#![deny(warnings)]
//! edzdz
//!
//!
//!
//!
//!

mod components;
mod routing;

pub use components::I18nRoute;

/// Create a route segment that is possible to define based on a locale.
///
/// ```rust, ignore
/// <Route path=i18n_path!(Locale, |locale| td_string(locale, path_name)) view=.. />
/// ```
#[macro_export]
macro_rules! i18n_path {
    ($t:ty, $func:expr) => {{
        $crate::__private::make_i18n_segment::<$t, _>($func)
    }};
}

#[doc(hidden)]
pub mod __private {
    pub use crate::routing::make_i18n_segment;
}