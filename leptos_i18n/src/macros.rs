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
///s
/// ```rust, no_run
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {
/// #           key: "",
/// #           interpolate_key: "<component>{{ variable }}</component>",
/// #       },
/// #   };
/// #
/// # use i18n::*;
/// # use leptos::prelude::*;
/// # let var = "";
/// # let comp = |_: leptos::children::ChildrenFn| {};
/// let i18n = use_i18n();
/// # let _ =
/// view! {
///     <p>{t!(i18n, key)}</p>
///     <p>{t!(i18n, interpolate_key, variable = var, <component> = comp)}</p>
/// }
/// # ;
/// ```
///
/// # Notes
///
/// If your variable/component value is the same as the key, you remove the assignment, such that this:
///
/// ```rust, no_run
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {
/// #           key: "<component>{{ variable }}</component>",
/// #       },
/// #   };
/// #
/// # use i18n::*;
/// # use leptos::prelude::*;
/// # let var = "";
/// # let comp = |_: leptos::children::ChildrenFn| {};
/// # let i18n = use_i18n();
/// # let _ =  
/// t!(i18n, key, variable = var, <component> = comp)
/// # ;
/// ```
///
/// can be shortened to:
///
/// ```rust, no_run
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {
/// #           key: "<component>{{ variable }}</component>",
/// #       },
/// #   };
/// #
/// # use i18n::*;
/// # use leptos::prelude::*;
/// # let variable = "";
/// # let component = |_: leptos::children::ChildrenFn| {};
/// # let i18n = use_i18n();
/// # let _ =  
/// t!(i18n, key, variable, <component>)
/// # ;
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
#[cfg_attr(feature = "dynamic_load", doc = "```rust, no_run")]
#[cfg_attr(not(feature = "dynamic_load"), doc = "```rust")]
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en", "fr"],
/// #       en: {
/// #           key: "",
/// #           interpolate_key: "<component>{{ variable }}</component>",
/// #       },
/// #       fr: {
/// #           key: "",
/// #           interpolate_key: "",
/// #       }
/// #   };
/// # use i18n::*;
/// # use leptos::prelude::*;
/// # let _ =
/// view! {
///     <p>{td!(Locale::en, key)}</p>
///     <p>{td!(Locale::fr, interpolate_key, variable = "some value", <component> = |child| { /* ... */} )}</p>
/// }
/// # ;
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

/// Just like the `t!` macro but return a `&'static str` or a `String` instead of a view.
///
/// Usage:
///
#[cfg_attr(feature = "dynamic_load", doc = "```rust, ignore")]
#[cfg_attr(not(feature = "dynamic_load"), doc = "```rust, no_run")]
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       interpolate_display,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {
/// #           click_count: "You clicked {{ count }} times",
/// #       },
/// #   };
/// # use i18n::*;
/// let i18n = use_i18n();
///
/// // click_count = "You clicked {{ count }} times"
///
/// assert_eq!(
///     t_string!(i18n, click_count, count = 10),
///     "You clicked 10 times"
/// );
///
/// assert_eq!(
///     t_string!(i18n, click_count, count = "a lot of"),
///     "You clicked a lot of times"
/// );
/// ```
///
/// If you want to avoid a temporary `String` to format in a buffer, you can use `t_display!` which return the raw builder which implement `Display`.
/// In fact, `t_string!(args)` internally is `t_display!(args).to_string()` (when using interpolation, else it just returns a `&'static str`).
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
#[cfg_attr(feature = "dynamic_load", doc = "```rust, ignore")]
#[cfg_attr(not(feature = "dynamic_load"), doc = "```rust")]
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       interpolate_display,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {
/// #           click_count: "You clicked {{ count }} times",
/// #       },
/// #   };
/// # use i18n::*;
/// // click_count = "You clicked {{ count }} times"
///
/// assert_eq!(
///     td_string!(Locale::en, click_count, count = 10),
///     "You clicked 10 times"
/// );
///
/// assert_eq!(
///     td_string!(Locale::en, click_count, count = "a lot of"),
///     "You clicked a lot of times"
/// );
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

/// Just like the `t_string!` macro but return either a struct implementing `Display` or a `&'static str` instead.
///
/// This is useful if you will print the value or use it in any formatting operation, as it will avoid a temporary `String`.
///
/// Usage:
///
#[cfg_attr(feature = "dynamic_load", doc = "```rust, ignore")]
#[cfg_attr(not(feature = "dynamic_load"), doc = "```rust, no_run")]
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       interpolate_display,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {
/// #           click_count: "You clicked {{ count }} times",
/// #       },
/// #   };
/// # use i18n::*;
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
///
/// Note that this is only usefull with interpolations, as with plain strings `t_display!` and `t_string!` both just returns the inner `&'static str`.
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
#[cfg_attr(feature = "dynamic_load", doc = "```rust, ignore")]
#[cfg_attr(not(feature = "dynamic_load"), doc = "```rust")]
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       interpolate_display,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {
/// #           click_count: "You clicked {{ count }} times",
/// #       },
/// #   };
/// # use i18n::*;
/// // click_count = "You clicked {{ count }} times"
///
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
/// ```rust, no_run
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {
/// #           namespace: {
/// #               subkeys: {
/// #                   value: "",
/// #               },
/// #           },
/// #       },
/// #   };
/// # use i18n::*;
/// let i18n = use_i18n();
/// t!(i18n, namespace.subkeys.value);
/// ```
///
/// You can do
///
/// ```rust, no_run
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {
/// #           namespace: {
/// #               subkeys: {
/// #                   value: "",
/// #               },
/// #           },
/// #       },
/// #   };
/// # use i18n::*;
/// let i18n = use_i18n_scoped!(namespace);
/// t!(i18n, subkeys.value);
/// ```
///
/// Or
///
/// ```rust, no_run
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {
/// #           namespace: {
/// #               subkeys: {
/// #                   value: "",
/// #               },
/// #           },
/// #       },
/// #   };
/// # use i18n::*;
/// let i18n = use_i18n_scoped!(namespace.subkeys);
/// t!(i18n, value);
/// ```
///
/// This macro is the equivalent to do
///
/// ```rust, no_run
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {
/// #           namespace: {
/// #               subkeys: {
/// #                   value: "",
/// #               },
/// #           },
/// #       },
/// #   };
/// # use i18n::*;
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
/// ```rust, no_run
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {
/// #           namespace: {
/// #               subkeys: {
/// #                   value: "",
/// #               },
/// #           },
/// #       },
/// #   };
/// # use i18n::*;
/// let i18n = use_i18n();
/// t!(i18n, namespace.subkeys.value);
/// ```
///
/// You can do
///
/// ```rust, no_run
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {
/// #           namespace: {
/// #               subkeys: {
/// #                   value: "",
/// #               },
/// #           },
/// #       },
/// #   };
/// # use i18n::*;
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
#[cfg_attr(feature = "dynamic_load", doc = "```rust, ignore")]
#[cfg_attr(not(feature = "dynamic_load"), doc = "```rust")]
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {
/// #           namespace: {
/// #               subkeys: {
/// #                   value: "",
/// #               },
/// #           },
/// #       },
/// #   };
/// # use i18n::*;
/// let locale = Locale::en;
/// td!(locale, namespace.subkeys.value);
/// ```
///
/// You can do
///
#[cfg_attr(feature = "dynamic_load", doc = "```rust, ignore")]
#[cfg_attr(not(feature = "dynamic_load"), doc = "```rust")]
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {
/// #           namespace: {
/// #               subkeys: {
/// #                   value: "",
/// #               },
/// #           },
/// #       },
/// #   };
/// # use i18n::*;
/// let locale = Locale::en;
/// let namespace_locale = scope_locale!(locale, namespace);
///
/// td!(namespace_locale, subkeys.value);
///
/// let subkeys_locale = scope_locale!(namespace_locale, subkeys);
/// //  subkeys_locale = scope_locale!(locale, namespace.subkeys);
///
/// td!(subkeys_locale, value);
/// ```
#[macro_export]
macro_rules! scope_locale {
    ($($tt:tt)*) => {
        $crate::__private::macros_reexport::scope_locale!{$($tt)*}
    };
}

/// Format a given value with a given formatter and return a `impl IntoView`.
///
#[cfg_attr(
    all(feature = "format_list", feature = "format_nums"),
    doc = "```rust, no_run"
)]
#[cfg_attr(
    not(all(feature = "format_list", feature = "format_nums")),
    doc = "```rust, ignore"
)]
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {},
/// #   };
/// # use i18n::*;
/// use leptos_i18n::t_format;
///
/// let i18n = use_i18n();
/// let num = || 100_000usize;
/// t_format!(i18n, num, formatter: number);
/// let list = || ["A", "B", "C"];
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
///
#[cfg_attr(all(feature = "format_list", feature = "format_nums"), doc = "```rust")]
#[cfg_attr(
    not(all(feature = "format_list", feature = "format_nums")),
    doc = "```rust, ignore"
)]
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {},
/// #   };
/// # use i18n::*;
/// use leptos_i18n::td_format;
///
/// let num = || 100_000usize;
/// td_format!(Locale::en, num, formatter: number);
/// let list = || ["A", "B", "C"];
/// td_format!(Locale::en, list, formatter: list(list_type: and; list_style: wide));
/// ```
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

/// Format a given value with a given formatter and return a `String`.
///
#[cfg_attr(
    all(feature = "format_list", feature = "format_nums"),
    doc = "```rust, no_run"
)]
#[cfg_attr(
    not(all(feature = "format_list", feature = "format_nums")),
    doc = "```rust, ignore"
)]
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {},
/// #   };
/// # use i18n::*;
/// use leptos_i18n::t_format_string;
///
/// let i18n = use_i18n();
/// let num = 100_000usize;
///
/// t_format_string!(i18n, num, formatter: number);
///
/// let list = ["A", "B", "C"];
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
///
#[cfg_attr(all(feature = "format_list", feature = "format_nums"), doc = "```rust")]
#[cfg_attr(
    not(all(feature = "format_list", feature = "format_nums")),
    doc = "```rust, ignore"
)]
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en", "fr"],
/// #       en: {},
/// #       fr: {},
/// #   };
/// # use i18n::*;
/// use leptos_i18n::td_format_string;
///
/// let num = 100_000usize;
///
/// let formated_num = td_format_string!(Locale::en, num, formatter: number);
/// assert_eq!(formated_num, "100,000");
/// let formated_num = td_format_string!(Locale::fr, num, formatter: number);
/// assert_eq!(formated_num, "100\u{202f}000");
///
/// let list = ["A", "B", "C"];
///
/// let formated_list = td_format_string!(Locale::en, list, formatter: list(list_type: and; list_style: wide));
/// assert_eq!(formated_list, "A, B, and C");
/// let formated_list = td_format_string!(Locale::fr, list, formatter: list(list_type: and; list_style: wide));
/// assert_eq!(formated_list, "A, B et C");
/// ```
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
#[cfg_attr(
    all(feature = "format_list", feature = "format_nums"),
    doc = "```rust, no_run"
)]
#[cfg_attr(
    not(all(feature = "format_list", feature = "format_nums")),
    doc = "```rust, ignore"
)]
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en", "fr"],
/// #       en: {},
/// #       fr: {},
/// #   };
/// # use i18n::*;
/// use leptos_i18n::t_format_display;
///
/// let i18n = use_i18n();
/// let num = 100_000usize;
///
/// t_format_display!(i18n, num, formatter: number);
///
/// let list = ["A", "B", "C"];
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
///
#[cfg_attr(all(feature = "format_list", feature = "format_nums"), doc = "```rust")]
#[cfg_attr(
    not(all(feature = "format_list", feature = "format_nums")),
    doc = "```rust, ignore"
)]
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en", "fr"],
/// #       en: {},
/// #       fr: {},
/// #   };
/// # use i18n::*;
/// use leptos_i18n::td_format_display;
///
/// let num = 100_000usize;
///
/// let num_formatter = td_format_display!(Locale::en, num, formatter: number);
/// assert_eq!(format!("number: {}.", num_formatter), "number: 100,000.");
/// let num_formatter = td_format_display!(Locale::fr, num, formatter: number);
/// assert_eq!(format!("nombre: {}.", num_formatter), "nombre: 100\u{202f}000.");
///
/// let list = ["A", "B", "C"];
///
/// let list_formatter = td_format_display!(Locale::en, list, formatter: list(list_type: and; list_style: wide));
/// assert_eq!(format!("values: {}.", list_formatter), "values: A, B, and C.");
/// let list_formatter = td_format_display!(Locale::fr, list, formatter: list(list_type: and; list_style: wide));
/// assert_eq!(format!("valeurs: {}.", list_formatter), "valeurs: A, B et C.");
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
#[cfg_attr(feature = "plurals", doc = "```rust, no_run")]
#[cfg_attr(not(feature = "plurals"), doc = "```rust, ignore")]
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en", "fr"],
/// #       en: {},
/// #       fr: {},
/// #   };
/// # use i18n::*;
/// # use leptos::logging::log;
/// # use leptos::prelude::Effect;
/// use leptos_i18n::t_plural;
///
/// let i18n = use_i18n();
///
/// let form = t_plural! {
///     i18n,
///     count = || 0,
///     one => "one",
///     _ => "other"
/// };
///
/// Effect::new(move || {
///     let s = form();
///     log!("{}", s);
/// });
/// ```
///
/// This will log "one" with locale "fr" but "other" with locale "en".
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
///
#[cfg_attr(feature = "plurals", doc = "```rust")]
#[cfg_attr(not(feature = "plurals"), doc = "```rust, ignore")]
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en", "fr"],
/// #       en: {},
/// #       fr: {},
/// #   };
/// # use i18n::*;
/// # use leptos::logging::log;
/// # use leptos::prelude::Effect;
/// use leptos_i18n::td_plural;
///
/// let form_en = td_plural! {
///     Locale::en,
///     count = || 0,
///     one => "one",
///     _ => "other"
/// };
///
/// assert_eq!(form_en, "other");
///
/// let form_fr = td_plural! {
///     Locale::fr,
///     count = || 0,
///     one => "one",
///     _ => "other"
/// };
///
/// assert_eq!(form_fr, "one");
/// ```
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
#[cfg_attr(feature = "plurals", doc = "```rust, no_run")]
#[cfg_attr(not(feature = "plurals"), doc = "```rust, ignore")]
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en", "fr"],
/// #       en: {},
/// #       fr: {},
/// #   };
/// # use i18n::*;
/// # use leptos::logging::log;
/// # use leptos::prelude::Effect;
/// use leptos_i18n::t_plural_ordinal;
///
/// let i18n = use_i18n();
///
/// let form = t_plural_ordinal! {
///     i18n,
///     count = || 2,
///     two => "two",
///     _ => "other"
/// };
///
/// Effect::new(move || {
///     let s = form();
///     log!("{}", s);
/// });
/// ```
///
/// This will log "other" with locale "fr" but "two" with locale "en".
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
///
#[cfg_attr(feature = "plurals", doc = "```rust")]
#[cfg_attr(not(feature = "plurals"), doc = "```rust, ignore")]
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       default: "en",
/// #       locales: ["en", "fr"],
/// #       en: {},
/// #       fr: {},
/// #   };
/// # use i18n::*;
/// # use leptos::logging::log;
/// # use leptos::prelude::Effect;
/// use leptos_i18n::td_plural_ordinal;
///
/// let form_en = td_plural_ordinal! {
///     Locale::en,
///     count = || 2,
///     two => "two",
///     _ => "other"
/// };
///
/// assert_eq!(form_en, "two");
///
/// let form_fr = td_plural_ordinal! {
///     Locale::fr,
///     count = || 2,
///     two => "two",
///     _ => "other"
/// };
///
/// assert_eq!(form_fr, "other");
/// ```
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

/// Create a route segment that is possible to define based on a locale.
///
/// ```rust, ignore
/// <Route path=i18n_path!(Locale, |locale| td_string(locale, path_name)) view=.. />
/// ```
#[macro_export]
macro_rules! i18n_path {
    ($t:ty, $func:expr) => {{
        leptos_i18n::__private::make_i18n_segment::<$t, _>($func)
    }};
}
