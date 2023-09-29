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

#[cfg(not(any(
    feature = "ssr",
    feature = "hydrate",
    all(feature = "csr", feature = "cookie")
)))]
#[inline]
pub fn fetch_locale<T: Locale>() -> T {
    Default::default()
}

#[cfg(all(feature = "csr", feature = "cookie"))]
pub fn fetch_locale<T: Locale>() -> T {
    fn inner<T: Locale>() -> Option<T> {
        let document = super::get_html_document()?;
        let cookies = document.cookie().ok()?;
        cookies.split(';').find_map(|cookie| {
            let (key, value) = cookie.split_once('=')?;
            if key.trim() == super::COOKIE_PREFERED_LANG {
                T::from_str(value)
            } else {
                None
            }
        })
    }
    inner().unwrap_or_default()
}
