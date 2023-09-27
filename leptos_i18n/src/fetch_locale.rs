use crate::Locale;

#[cfg(feature = "ssr")]
#[inline]
pub fn fetch_locale<T: Locale>() -> T {
    crate::server::fetch_locale_server_side::<T>()
}

#[cfg(feature = "hydrate")]
pub fn fetch_locale<T: Locale>() -> T {
    leptos::document()
        .document_element()
        .and_then(|el| el.get_attribute("lang"))
        .and_then(|lang| T::from_str(&lang))
        .unwrap_or_default()
}

#[cfg(not(any(feature = "ssr", feature = "hydrate")))]
#[inline]
pub fn fetch_locale<T: Locale>() -> T {
    Default::default()
}
