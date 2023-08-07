leptos_i18n::load_locales!();

#[macro_export]
macro_rules! t {
    ($cx: ident) => {
        ::leptos_i18n::t!($cx, $crate::i18n::Locales)
    };
    ($cx: ident, $key: ident) => {{
        ::leptos_i18n::t!($cx, $crate::i18n::Locales, $key)
    }};
}
