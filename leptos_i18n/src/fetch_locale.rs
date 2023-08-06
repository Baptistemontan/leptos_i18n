use leptos::*;

use crate::Locales;

#[cfg(feature = "ssr")]
pub fn fetch_locale<T: Locales>(cx: Scope) -> T::Variants {
    crate::server::fetch_locale_server_side::<T>(cx)
}

#[cfg(feature = "hydrate")]
pub fn fetch_locale<T: Locales>(cx: Scope) -> T::Variants {
    use crate::LocaleVariant;
    let _ = cx;
    document()
        .document_element()
        .and_then(|el| el.get_attribute("lang"))
        .and_then(|lang| <T::Variants as LocaleVariant>::from_str(&lang))
        .unwrap_or_default()
}

#[cfg(not(any(feature = "ssr", feature = "hydrate")))]
pub fn fetch_locale<T: Locales>(cx: Scope) -> T::Variants {
    let _ = cx;
    Default::default()
}
