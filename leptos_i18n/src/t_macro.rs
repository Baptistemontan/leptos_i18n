#[macro_export]
macro_rules! t {
    ($i18n: ident, $key: ident) => {{
        move || {
            let _keys = ::leptos_i18n::I18nContext::get_keys($i18n);
            _keys.$key
        }
    }};
    ($i18n: ident, $key: ident, $($variable:ident = $value:expr,)*) => {{
        move || {
            let _keys = ::leptos_i18n::I18nContext::get_keys($i18n);
            let _key = _keys.$key;
            $(let _key = _key.$variable($value);)*
            _key
        }
    }};
}
