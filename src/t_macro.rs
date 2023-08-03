#[macro_export]
macro_rules! t {
    ($cx: ident, $key: expr) => {
        ::leptos_i18n::translate($cx, $key, None)
    };
    ($cx: ident, $key: expr, $default:expr) => {
        ::leptos_i18n::translate($cx, $key, Some($default))
    };
}
