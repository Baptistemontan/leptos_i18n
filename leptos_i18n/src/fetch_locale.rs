use crate::Locale;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "ssr", not(any(feature = "hydrate", all(feature="csr", feature="cookie")))))] {
        #[inline]
        pub fn fetch_locale<T: Locale>() -> T {
            crate::server::fetch_locale_server_side::<T>()
        }
    } else if #[cfg(all(feature = "hydrate", not(any(all(feature="csr", feature="cookie"), feature = "ssr"))))] {
        pub fn fetch_locale<T: Locale>() -> T {
            leptos::document()
                .document_element()
                .and_then(|el| el.get_attribute("lang"))
                .and_then(|lang| T::from_str(&lang))
                .unwrap_or_default()
        }
    } else if #[cfg(all(all(feature="csr", feature="cookie"), not(any(feature = "ssr", feature = "hydrate"))))] {
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
    } else {
        #[inline]
        pub fn fetch_locale<T: Locale>() -> T {
            Default::default()
        }
    }
}
