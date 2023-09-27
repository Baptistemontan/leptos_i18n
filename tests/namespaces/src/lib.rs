#![deny(warnings)]
leptos_i18n::load_locales!();
#[cfg(test)]
mod first_ns;
#[cfg(test)]
mod second_ns;

mod tests_includes {
    pub use super::{assert_eq_rendered, render_to_string};
    pub use crate::i18n::*;
    pub use leptos::*;
}

use leptos::{IntoView, Oco};
pub fn render_to_string<'a, T: 'a>(view: T) -> Oco<'a, str>
where
    T: IntoView,
{
    let rendered = view.into_view().render_to_string();
    let comment_removed = remove_html_comments(rendered);
    remove_hk(comment_removed)
}

fn remove_noise<'a>(s: Oco<'a, str>, start_delim: &str, end_delim: &str) -> Oco<'a, str> {
    if let Some((start, rest)) = s.split_once(start_delim) {
        let mut output_str = start.to_owned();
        let (_, mut s) = rest.split_once(end_delim).unwrap();
        while let Some((start, rest)) = s.split_once(start_delim) {
            output_str.push_str(start);
            let (_, rest) = rest.split_once(end_delim).unwrap();
            s = rest;
        }
        output_str.push_str(s);
        Oco::Owned(output_str)
    } else {
        s
    }
}

fn remove_html_comments(s: Oco<str>) -> Oco<str> {
    remove_noise(s, "<!--", "-->")
}

fn remove_hk(s: Oco<str>) -> Oco<str> {
    remove_noise(s, " data-hk=\"", "\"")
}

#[macro_export]
macro_rules! assert_eq_rendered {
    ($left:expr, $($right:tt)*) => {
        assert_eq!(render_to_string($left), $($right)*)
    };
}
