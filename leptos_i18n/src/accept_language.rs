//! The locales the visitor prefers: the `Accept-Language` request header on
//! the server, `navigator.languages` in the browser.

use default_struct_builder::DefaultBuilder;
use leptos::prelude::*;
use std::sync::Arc;

/// Options for reading the visitor's preferred locales.
#[derive(DefaultBuilder)]
pub struct UseLocalesOptions {
    /// Returns the raw value of the `Accept-Language` request header on the server.
    ///
    /// The `axum` and `actix` features provide a default that reads the current
    /// request; without them the default returns `None`.
    #[cfg_attr(not(feature = "ssr"), allow(dead_code))]
    ssr_lang_header_getter: Arc<dyn Fn() -> Option<String> + Send + Sync>,
}

impl Default for UseLocalesOptions {
    fn default() -> Self {
        Self {
            ssr_lang_header_getter: Arc::new(|| {
                #[cfg(feature = "ssr")]
                {
                    crate::server::request_header("accept-language")
                }
                #[cfg(not(feature = "ssr"))]
                {
                    None
                }
            }),
        }
    }
}

/// The visitor's preferred locales, most preferred first.
///
/// On the server the value is fixed for the request. In the browser it follows
/// `navigator.languages` and updates on the `languagechange` event.
pub(crate) fn accepted_locales(options: UseLocalesOptions) -> Signal<Vec<String>> {
    #[cfg(feature = "ssr")]
    {
        let UseLocalesOptions {
            ssr_lang_header_getter,
        } = options;
        Signal::stored(parse_accept_language(
            &ssr_lang_header_getter().unwrap_or_default(),
        ))
    }
    #[cfg(not(feature = "ssr"))]
    {
        let _ = options;
        let (locales, set_locales) = signal(navigator_languages());
        let handle = window_event_listener_untyped("languagechange", move |_| {
            set_locales.set(navigator_languages())
        });
        on_cleanup(move || handle.remove());
        locales.into()
    }
}

/// The language tags of an `Accept-Language` value in header order, without
/// their quality weights.
#[cfg(feature = "ssr")]
fn parse_accept_language(header: &str) -> Vec<String> {
    header
        .split(',')
        .map(|entry| {
            entry
                .split_once(';')
                .map_or(entry, |(tag, _weight)| tag)
                .trim()
                .to_owned()
        })
        .collect()
}

#[cfg(not(feature = "ssr"))]
fn navigator_languages() -> Vec<String> {
    window()
        .navigator()
        .languages()
        .iter()
        .filter_map(|language| language.as_string())
        .collect()
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    #[test]
    fn parse_accept_language_keeps_header_order_without_weights() {
        assert_eq!(
            parse_accept_language("fr-CH, fr;q=0.9, en;q=0.8, *;q=0.5"),
            ["fr-CH", "fr", "en", "*"]
        );
        assert_eq!(parse_accept_language(""), [""]);
    }
}
