// for deserializing the files custom deserialization is done,
// this is to use `serde::de::DeserializeSeed` to pass information on what locale or key we are currently at
// and give better information on what went wrong when an error is emitted.

/// Look for the configuration in the cargo manifest `Cargo.toml` at the root of the project and load the given locales.
///
/// It creates multiple types allowing to easily incorporate translations in you application such as:
///
/// - `Locale`: an enum representing the available locales of the application.
/// - `I18nKeys`: a struct representing the translation keys.
#[macro_export]
macro_rules! load_locales {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::load_locales!{$($tt)*}
    };
}

/// This is for a private use writing tests.
#[macro_export]
#[doc(hidden)]
macro_rules! declare_locales {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::declare_locales!{$($tt)*}
    };
}

/// Utility macro to easily put translation in your application.
///
/// Usage:
///
/// ```rust, ignore
/// use crate::i18n::*;
///
/// let i18n = use_i18n();
///
/// view! {
///     <p>{t!(i18n, $key)}</p>
///     <p>{t!(i18n, $key, $variable = $value, <$component> = |child| ... )}</p>
/// }
/// ```
///
/// # Notes
///
/// If your variable/component value is the same as the key, you remove the assignment, such that this:
///
/// ```rust, ignore
/// t!(i18n, $key, variable = variable, <component> = component, $other_key = $other_value, ..)
/// ```
///
/// can be shortened to:
///
/// ```rust, ignore
/// t!(i18n, $key, variable, <component>, $other_key = $other_value, ..)
/// ```
#[macro_export]
macro_rules! t {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::t!{$($tt)*}
    };
}

/// Just like the `t!` macro but instead of taking `I18nContext` as the first argument it takes the desired locale.
///
/// Usage:
///
/// ```rust, ignore
/// use crate::i18n::*;
///
/// view! {
///     <p>{td!(Locale::en, $key)}</p>
///     <p>{td!(Locale::fr, $key, $variable = $value, <$component> = |child| ... )}</p>
/// }
/// ```
///
/// This let you use a specific locale regardless of the current one.
#[macro_export]
macro_rules! td {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::td!{$($tt)*}
    };
}

/// Same as the `t!` macro but untracked.
#[macro_export]
macro_rules! tu {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::tu!{$($tt)*}
    };
}

/// Just like the `t!` macro but return a `Cow<'static, str>` instead of a view.
///
/// Usage:
///
/// ```rust, ignore
/// use crate::i18n::*;
///
/// let i18n = use_i18n(); // locale = "en"
///
/// // click_count = "You clicked {{ count }} times"
///
/// assert_eq!(
///     t_string!(i18n, click_count, count = 10),
///     "You clicked 10 times"
/// )
///
/// assert_eq!(
///     t_string!(i18n, click_count, count = "a lot of"),
///     "You clicked a lot of times"
/// )
/// ```
#[macro_export]
macro_rules! t_string {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::t_string!{$($tt)*}
    };
}

/// Just like the `t_string!` macro but takes the `Locale` as an argument instead of the context.
///
/// Usage:
///
/// ```rust, ignore
/// use crate::i18n::*;
///
/// // click_count = "You clicked {{ count }} times"
/// assert_eq!(
///     td_string!(Locale::en, click_count, count = 10),
///     "You clicked 10 times"
/// )
///
/// assert_eq!(
///     td_string!(Locale::en, click_count, count = "a lot of"),
///     "You clicked a lot of times"
/// )
/// ```
#[macro_export]
macro_rules! td_string {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::td_string!{$($tt)*}
    };
}

/// Same as the `t_string!` macro but untracked.
#[macro_export]
macro_rules! tu_string {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::tu_string!{$($tt)*}
    };
}

/// Just like the `t_string!` macro but return either a struct implementing `Display` or a `&'static str` instead of a `Cow<'static, str>`.
///
/// This is useful if you will print the value or use it in any formatting operation, as it will avoid a temporary `String`.
///
/// Usage:
///
/// ```rust, ignore
/// use crate::i18n::*;
///
/// let i18n = use_i18n(); // locale = "en"
///
/// // click_count = "You clicked {{ count }} times"
/// let t = t_display!(i18n, click_count, count = 10); // this only return the builder, no work has been done.
///
/// assert_eq!(format!("before {t} after"), "before You clicked 10 times after");
///
/// let t_str = t.to_string(); // can call `to_string` as the value impl `Display`
///
/// assert_eq!(t_str, "You clicked 10 times");
/// ```
#[macro_export]
macro_rules! t_display {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::t_display!{$($tt)*}
    };
}

/// Just like the `t_display!` macro but takes the `Locale` as an argument instead of the context.
///
/// This is useful if you will print the value or use it in any formatting operation, as it will avoid a temporary `String`.
///
/// Usage:
///
/// ```rust, ignore
/// use crate::i18n::*;
///
/// // click_count = "You clicked {{ count }} times"
/// let t = td_display!(Locale::en, click_count, count = 10); // this only return the builder, no work has been done.
///
/// assert_eq!(format!("before {t} after"), "before You clicked 10 times after");
///
/// let t_str = t.to_string(); // can call `to_string` as the value impl `Display`
///
/// assert_eq!(t_str, "You clicked 10 times");
/// ```
#[macro_export]
macro_rules! td_display {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::td_display!{$($tt)*}
    };
}

/// Same as the `t_display!` macro but untracked.
#[macro_export]
macro_rules! tu_display {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::tu_display!{$($tt)*}
    };
}

/// Like `use_i18n` but enable to scope the context:
///
/// Instead of
///
/// ```rust, ignore
/// let i18n = use_i18n();
/// t!(i18n, namespace.subkeys.value);
/// ```
///
/// You can do
///
/// ```rust, ignore
/// let i18n = use_i18n_scoped!(namespace);
/// t!(i18n, subkeys.value);
/// ```
///
/// Or
///
/// ```rust, ignore
/// let i18n = use_i18n_scoped!(namespace.subkeys);
/// t!(i18n, value);
/// ```
///
/// This macro is the equivalent to do
///
/// ```rust, ignore
/// let i18n = use_i18n();
/// let i18n = scope_i18n!(i18n, namespace.subkeys);
/// ```
#[macro_export]
macro_rules! use_i18n_scoped {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::use_i18n_scoped!{$($tt)*}
    };
}

/// Scope a context to the given keys
///
/// Instead of
///
/// ```rust, ignore
/// let i18n = use_i18n;
/// t!(i18n, namespace.subkeys.value);
/// ```
///
/// You can do
///
/// ```rust, ignore
/// let i18n = use_i18n();
/// let namespace_i18n = scope_i18n!(i18n, namespace);
///
/// t!(namespace_i18n, subkeys.value);
///
/// let subkeys_i18n = scope_i18n!(namespace_i18n, subkeys);
/// //  subkeys_i18n = scope_i18n!(i18n, namespace.subkeys);
///
/// t!(subkeys_i18n, value);
/// ```
#[macro_export]
macro_rules! scope_i18n {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::scope_i18n!{$($tt)*}
    };
}

/// Scope a locale to the given keys
///
/// Instead of
///
/// ```rust, ignore
/// let i18n = use_i18n();
/// t!(i18n, namespace.subkeys.value);
/// ```
///
/// You can do
///
/// ```rust, ignore
/// let i18n = use_i18n();
/// let namespace_i18n = scope_i18n!(i18n, namespace);
///
/// t!(namespace_i18n, subkeys.value);
///
/// let subkeys_i18n = scope_i18n!(namespace_i18n, subkeys);
/// //  subkeys_i18n = scope_i18n!(i18n, namespace.subkeys);
///
/// t!(subkeys_i18n, value);
/// ```
#[macro_export]
macro_rules! scope_locale {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::scope_locale!{$($tt)*}
    };
}

/// Format a given value with a given formatter and return:
///
/// ```rust, ignore
/// let i18n =  use_i18n();
/// let num = 100_000usize;
///
/// t_format!(i18n, num, formatter: number);
///
/// let list = || ["A", "B", "C"];
///
/// t_format!(i18n, list, formatter: list(list_type: and; list_style: wide));
/// ```
/// This function does exactly the same as if you had "{{ var, formatter_name(formatter_arg: value; ...) }}"
/// for a translation and do
///
/// ```rust,ignore
/// t!(i18n, key, var = ...)
/// ```
#[macro_export]
macro_rules! t_format {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::t_format!{$($tt)*}
    };
}

/// Same as the `t_format!` macro but takes the desired `Locale` as the first argument.
#[macro_export]
macro_rules! td_format {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::td_format!{$($tt)*}
    };
}

/// Same as the `t_format!` macro but untracked.
#[macro_export]
macro_rules! tu_format {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::tu_format!{$($tt)*}
    };
}

/// Format a given value with a given formatter and return a `String`:
///
/// ```rust, ignore
/// let i18n =  use_i18n();
/// let num = 100_000usize;
///
/// t_format_string!(i18n, num, formatter: number);
///
/// let list = || ["A", "B", "C"];
///
/// t_format_string!(i18n, list, formatter: list(list_type: and; list_style: wide));
/// ```
/// This function does exactly the same as if you had "{{ var, formatter_name(formatter_arg: value; ...) }}"
/// for a translation and do
///
/// ```rust,ignore
/// t_string!(i18n, key, var = ...)
/// ```
#[macro_export]
macro_rules! t_format_string {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::t_format_string!{$($tt)*}
    };
}

/// Same as the `t_format_string!` macro but takes the desired `Locale` as the first argument.
#[macro_export]
macro_rules! td_format_string {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::td_format_string!{$($tt)*}
    };
}

/// Same as the `t_format_string!` macro but untracked.
#[macro_export]
macro_rules! tu_format_string {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::tu_format_string!{$($tt)*}
    };
}

/// Format a given value with a given formatter and return a `impl Display`:
///
/// ```rust, ignore
/// let i18n = use_i18n();
/// let num = 100_000usize;
///
/// t_format_display!(i18n, num, formatter: number);
///
/// let list = || ["A", "B", "C"];
///
/// t_format_display!(i18n, list, formatter: list(list_type: and; list_style: wide));
/// ```
/// This function does exactly the same as if you had "{{ var, formatter_name(formatter_arg: value; ...) }}"
/// for a translation and do
///
/// ```rust,ignore
/// t_display!(i18n, key, var = ...)
/// ```
#[macro_export]
macro_rules! t_format_display {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::t_format_display!{$($tt)*}
    };
}

/// Same as the `t_format_display!` macro but takes the desired `Locale` as the first argument.
#[macro_export]
macro_rules! td_format_display {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::td_format_display!{$($tt)*}
    };
}

/// Same as the `t_format_display!` macro but untracked.
#[macro_export]
macro_rules! tu_format_display {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::tu_format_display!{$($tt)*}
    };
}

/// Match against the plural form of a given count:
///
/// ```rust, ignore
/// let i18n = use_i18n();
///
/// let form = t_plural! {
///     i18n,
///     count = || 0,
///     one => "one",
///     _ => "other"
/// };
///
/// Effect::new(|| {
///     let s = form();
///     log!("{}", s);
/// })
/// ```
///
/// This will print "one" with locale "fr" but "other" with locale "en".
///
/// Accepted forms are: `zero`, `one`, `two`, `few`, `many`, `other` and `_`.
///
/// This is for the cardinal form of plurals, for ordinal form see `t_plural_ordinal!`.
#[macro_export]
macro_rules! t_plural {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::t_plural!{$($tt)*}
    };
}

/// Same as the `t_plural!` macro but takes the desired `Locale` as the first argument.
/// Directly return the value instead of wrapping it in a closure.
#[macro_export]
macro_rules! td_plural {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::td_plural!{$($tt)*}
    };
}

/// Same as the `t_plural!` macro but untracked.
/// Directly return the value instead of wrapping it in a closure.
#[macro_export]
macro_rules! tu_plural {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::tu_plural!{$($tt)*}
    };
}

/// Match against the plural form of a given count:
///
/// ```rust, ignore
/// let i18n = use_i18n();
///
/// let form = t_plural! {
///     i18n,
///     count = || 2,
///     two => "two",
///     _ => "other"
/// };
///
/// Effect::new(|| {
///     let s = form();
///     log!("{}", s);
/// })
/// ```
///
/// This will print "other" with locale "fr" but "two" with locale "en".
///
/// Accepted forms are: `zero`, `one`, `two`, `few`, `many`, `other` and `_`.
///
/// This is for the ordinal form of plurals, for cardinal form see `t_plural!`.
#[macro_export]
macro_rules! t_plural_ordinal {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::t_plural_ordinal!{$($tt)*}
    };
}

/// Same as the `t_plural_ordinal!` macro but takes the desired `Locale` as the first argument.
/// Directly return the value instead of wrapping it in a closure.
#[macro_export]
macro_rules! td_plural_ordinal {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::td_plural_ordinal!{$($tt)*}
    };
}

/// Same as the `t_plural_ordinal!` macro but untracked.
/// Directly return the value instead of wrapping it in a closure.
#[macro_export]
macro_rules! tu_plural_ordinal {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::tu_plural_ordinal!{$($tt)*}
    };
}
