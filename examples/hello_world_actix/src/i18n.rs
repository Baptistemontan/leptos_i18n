leptos_i18n::load_locales!();

#[macro_export]
macro_rules! t {
    ($cx: ident) => {
        ::leptos_i18n::t!($cx, $crate::i18n::Locales)
    };
    ($cx: ident, $key: ident) => {
        ::leptos_i18n::t!($cx, $crate::i18n::Locales, $key)
    };
    ($cx: ident, $key: ident, $($variable:ident = $value:expr,)*) => {
        ::leptos_i18n::t!($cx, $crate::i18n::Locales, $key, $($variable = $value,)*)
    };
}
