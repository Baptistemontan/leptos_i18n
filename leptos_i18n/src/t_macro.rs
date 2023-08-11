#[macro_export]
macro_rules! t {
    ($cx: ident, $locales:ident$(::$path:ident)*) => {{
        ::leptos_i18n::get_context::<$locales$(::$path)*>($cx).get_locale()
    }};
    ($cx: ident, $locales:ident$(::$path:ident)*, $key: ident) => {{
        let _context = ::leptos_i18n::get_context::<$locales$(::$path)*>($cx);
        move || {
            let _keys = _context.get_locale();
            _keys.$key
        }
    }};
    ($cx: ident, $locales:ident$(::$path:ident)*, $key: ident, $($variable:ident = $value:expr,)*) => {{
        let _context = ::leptos_i18n::get_context::<$locales$(::$path)*>($cx);
        move || {
            let _keys = _context.get_locale();
            let _key = _keys.$key;
            $(let _key = _key.$variable($value);)*
            _key
        }
    }};
}
