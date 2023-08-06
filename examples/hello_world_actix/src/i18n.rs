use leptos::*;

leptos_i18n::load_locales!();

#[macro_export]
macro_rules! t {
    ($cx: ident) => {
        ::leptos_i18n::t!($cx, $crate::i18n::Locales)
    };
    ($cx: ident, $key: ident) => {
        move || t!($cx).$key
    };
}

#[server(FetchLocale, "/api")]
pub async fn fetch_locale(cx: Scope) -> Result<LocaleEnum, ServerFnError> {
    leptos_i18n::server::fetch_locale::<Locales>(cx).await
}
