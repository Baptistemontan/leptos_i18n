use crate::Locale;

#[inline]
pub fn fetch_locale<T: Locale>(current_cookie: Option<T>) -> T {
    if cfg!(feature = "ssr") {
        fetch_locale_ssr(current_cookie)
    } else if cfg!(feature = "hydrate") {
        fetch_locale_hydrate(current_cookie)
    } else if cfg!(feature = "csr") {
        fetch_locale_csr(current_cookie)
    } else {
        current_cookie.unwrap_or_default()
    }
}

// ssr fetch
fn fetch_locale_ssr<T: Locale>(current_cookie: Option<T>) -> T {
    crate::server::fetch_locale_server_side(current_cookie)
}

// hydrate fetch
fn fetch_locale_hydrate<T: Locale>(current_cookie: Option<T>) -> T {
    leptos::document()
        .document_element()
        .and_then(|el| el.get_attribute("lang"))
        .and_then(|lang| T::from_str(&lang).ok())
        .unwrap_or(current_cookie.unwrap_or_default())
}

// csr fetch
fn fetch_locale_csr<T: Locale>(current_cookie: Option<T>) -> T {
    if let Some(lang) = current_cookie {
        return lang;
    }

    let accepted_langs = leptos::window()
        .navigator()
        .languages()
        .into_iter()
        .filter_map(|js_val| js_val.as_string())
        .collect::<Vec<_>>();

    T::find_locale(&accepted_langs)
}
