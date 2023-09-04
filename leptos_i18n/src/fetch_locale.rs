use crate::Locales;

#[cfg(feature = "ssr")]
#[inline]
pub fn fetch_locale<T: Locales>() -> T::Variants {
    crate::server::fetch_locale_server_side::<T>()
}

#[cfg(feature = "hydrate")]
pub fn fetch_locale<T: Locales>() -> T::Variants {
    use crate::LocaleVariant;
    leptos::document()
        .document_element()
        .and_then(|el| el.get_attribute("lang"))
        .and_then(|lang| <T::Variants as LocaleVariant>::from_str(&lang))
        .unwrap_or_default()
}

#[cfg(not(any(feature = "ssr", feature = "hydrate")))]
#[inline]
pub fn fetch_locale<T: Locales>() -> T::Variants {
    Default::default()
}
