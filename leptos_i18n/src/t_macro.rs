// #[macro_export]
// macro_rules! t {
//     ($cx: ident, $key: expr) => {
//         move || ::leptos_i18n::translate($cx, $key, None)
//     };
//     ($cx: ident, $key: expr, $default:expr) => {
//         move || ::leptos_i18n::translate($cx, $key, Some($default))
//     };
// }

#[macro_export]
macro_rules! t {
    ($cx: ident, $locales: path, $key: ident) => {
        move || {
            let _keys = t!($cx, $locales);
            _keys.$key
        }
    };
    ($cx: ident, $locales: path) => {{
        let _context = ::leptos_i18n::get_context::<$locales>($cx);
        _context.get_locale($cx)
    }};
}
