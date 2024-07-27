use crate::Locale;

#[inline]
pub fn fetch_locale<T: Locale>() -> T {
    if cfg!(feature = "ssr") {
        fetch_locale_ssr()
    } else if cfg!(feature = "hydrate") {
        fetch_locale_hydrate()
    } else if cfg!(feature = "csr") {
        fetch_locale_csr()
    } else {
        Default::default()
    }
}

// ssr fetch
fn fetch_locale_ssr<T: Locale>() -> T {
    crate::server::fetch_locale_server_side::<T>()
}

// hydrate fetch
fn fetch_locale_hydrate<T: Locale>() -> T {
    leptos::document()
        .document_element()
        .and_then(|el| el.get_attribute("lang"))
        .and_then(|lang| T::from_str(&lang))
        .unwrap_or_default()
}

// csr fetch
fn fetch_locale_csr<T: Locale>() -> T {
    fn get_lang_cookie<T: Locale>() -> Option<T> {
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
    if cfg!(feature = "cookie") {
        get_lang_cookie().unwrap_or_default()
    } else {
        Default::default()
    }
}
