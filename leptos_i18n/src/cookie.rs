//! The cookie that remembers the visitor's locale across page loads.

use default_struct_builder::DefaultBuilder;
use leptos::prelude::*;
use std::sync::Arc;

use crate::Locale;

/// The [`SameSite`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie#samesitesamesite-value)
/// attribute of the locale cookie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SameSite {
    /// The cookie is only sent with requests from the cookie's own site.
    Strict,
    /// The cookie is also sent when the visitor navigates to the site from elsewhere.
    Lax,
    /// The cookie is sent with every request, which browsers only allow with `Secure`.
    None,
}

impl SameSite {
    const fn as_str(self) -> &'static str {
        match self {
            SameSite::Strict => "Strict",
            SameSite::Lax => "Lax",
            SameSite::None => "None",
        }
    }
}

/// Options for the locale cookie.
///
/// By default the cookie lasts for the browser session and carries no
/// `Domain`, `Path`, `SameSite`, `Secure` or `HttpOnly` attribute.
#[derive(DefaultBuilder)]
pub struct CookieOptions {
    /// The [`Max-Age`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie#max-agenumber)
    /// attribute in seconds. Default: `None`, the cookie lasts for the browser session.
    #[builder(into)]
    max_age: Option<i64>,
    /// Whether to set the [`HttpOnly`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie#httponly)
    /// attribute. Default: `false`. A browser cannot read or write such a cookie,
    /// so the locale then only reaches the client through the server-rendered page.
    http_only: bool,
    /// Whether to set the [`Secure`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie#secure)
    /// attribute. Default: `false`.
    secure: bool,
    /// The [`Domain`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie#domaindomain-value)
    /// attribute. Default: `None`, the cookie applies to the current host only.
    #[builder(into)]
    domain: Option<String>,
    /// The [`Path`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie#pathpath-value)
    /// attribute. Default: `None`, the browser derives it from the current URL.
    #[builder(into)]
    path: Option<String>,
    /// The [`SameSite`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Set-Cookie#samesitesamesite-value)
    /// attribute. Default: `None`, the attribute is not set.
    #[builder(into)]
    same_site: Option<SameSite>,
    /// Returns the raw value of the `Cookie` request header on the server.
    ///
    /// The `axum` and `actix` features provide a default that reads the current
    /// request; without them the default returns `None`.
    #[cfg_attr(not(feature = "ssr"), allow(dead_code))]
    ssr_cookies_header_getter: Arc<dyn Fn() -> Option<String> + Send + Sync>,
    /// Appends a `Set-Cookie` header with the given value to the response on the server.
    ///
    /// The `axum` and `actix` features provide a default that writes to the
    /// current response; without them the default does nothing.
    #[cfg_attr(not(feature = "ssr"), allow(dead_code))]
    ssr_set_cookie: Arc<dyn Fn(&str) + Send + Sync>,
}

impl Default for CookieOptions {
    fn default() -> Self {
        Self {
            max_age: None,
            http_only: false,
            secure: false,
            domain: None,
            path: None,
            same_site: None,
            ssr_cookies_header_getter: Arc::new(|| {
                #[cfg(feature = "ssr")]
                {
                    crate::server::request_header("cookie")
                }
                #[cfg(not(feature = "ssr"))]
                {
                    None
                }
            }),
            ssr_set_cookie: Arc::new(|header| {
                #[cfg(feature = "ssr")]
                {
                    crate::server::append_set_cookie(header);
                }
                #[cfg(not(feature = "ssr"))]
                {
                    let _ = header;
                }
            }),
        }
    }
}

/// The locale stored in the cookie `name`, and a writer that keeps the cookie
/// in sync with the locale written to it.
///
/// On the server the cookie the request carried is sent back through a
/// `Set-Cookie` response header, so a `max_age` counts from the last render;
/// the locale resolved for a first visit is stored by the browser after
/// hydration. In the browser the cookie is written through `document.cookie`
/// after every change.
pub(crate) fn use_locale_cookie<L: Locale>(
    name: &str,
    options: CookieOptions,
) -> (Signal<Option<L>>, WriteSignal<Option<L>>) {
    let name = name.to_owned();
    let initial = read_cookie(&name, &options).and_then(|value| L::from_str(&value).ok());

    #[cfg(feature = "ssr")]
    if let Some(locale) = &initial {
        write_cookie(&name, Some(locale.as_str()), &options);
    }

    let (cookie, set_cookie) = signal(initial);

    #[cfg(not(feature = "ssr"))]
    Effect::new(move |_| {
        let value = cookie.get().map(|locale| locale.as_str());
        if read_cookie(&name, &options).as_deref() != value {
            write_cookie(&name, value, &options);
        }
    });

    (cookie.into(), set_cookie)
}

/// The raw value of the cookie `name`, from the request on the server and
/// from `document.cookie` in the browser.
fn read_cookie(name: &str, options: &CookieOptions) -> Option<String> {
    #[cfg(feature = "ssr")]
    let header = (options.ssr_cookies_header_getter)();
    #[cfg(not(feature = "ssr"))]
    let header = {
        let _ = options;
        html_document().cookie().ok()
    };
    cookie_value(&header?, name)
}

/// The value of the cookie `name` in a `Cookie` header or `document.cookie` string.
fn cookie_value(header: &str, name: &str) -> Option<String> {
    header
        .split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(key, _value)| *key == name)
        .map(|(_key, value)| value.to_owned())
}

fn write_cookie(name: &str, value: Option<&str>, options: &CookieOptions) {
    let header = set_cookie_header(name, value, options);
    #[cfg(feature = "ssr")]
    (options.ssr_set_cookie)(&header);
    #[cfg(not(feature = "ssr"))]
    {
        let _ = html_document().set_cookie(&header);
    }
}

/// The `Set-Cookie` value that stores `value` in the cookie `name`, or
/// expires the cookie when `value` is `None`.
fn set_cookie_header(name: &str, value: Option<&str>, options: &CookieOptions) -> String {
    let mut header = format!("{name}={}", value.unwrap_or_default());
    match (value, options.max_age) {
        (None, _) => header.push_str("; Max-Age=0"),
        (Some(_), Some(max_age)) => header.push_str(&format!("; Max-Age={max_age}")),
        (Some(_), None) => {}
    }
    if let Some(domain) = &options.domain {
        header.push_str(&format!("; Domain={domain}"));
    }
    if let Some(path) = &options.path {
        header.push_str(&format!("; Path={path}"));
    }
    if let Some(same_site) = options.same_site {
        header.push_str(&format!("; SameSite={}", same_site.as_str()));
    }
    if options.secure {
        header.push_str("; Secure");
    }
    if options.http_only {
        header.push_str("; HttpOnly");
    }
    header
}

#[cfg(not(feature = "ssr"))]
fn html_document() -> web_sys::HtmlDocument {
    use wasm_bindgen::JsCast;
    document().unchecked_into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_cookie_header_carries_every_configured_attribute() {
        let options = CookieOptions::default()
            .max_age(3600)
            .domain("example.com")
            .path("/")
            .same_site(SameSite::Lax)
            .secure(true)
            .http_only(true);
        assert_eq!(
            set_cookie_header("locale", Some("fr"), &options),
            "locale=fr; Max-Age=3600; Domain=example.com; Path=/; SameSite=Lax; Secure; HttpOnly"
        );
    }

    #[test]
    fn set_cookie_header_expires_a_removed_cookie() {
        let options = CookieOptions::default().max_age(3600).path("/");
        assert_eq!(
            set_cookie_header("locale", None, &options),
            "locale=; Max-Age=0; Path=/"
        );
    }

    #[test]
    fn cookie_value_finds_the_named_pair_among_others() {
        let header = " theme=dark; locale=fr ;token=a=b";
        assert_eq!(cookie_value(header, "locale").as_deref(), Some("fr"));
        assert_eq!(cookie_value(header, "token").as_deref(), Some("a=b"));
        assert_eq!(cookie_value(header, "missing"), None);
        assert_eq!(cookie_value("", "locale"), None);
    }
}
