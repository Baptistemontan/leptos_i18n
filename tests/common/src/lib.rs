#![deny(warnings)]

pub use leptos::*;

pub fn render_to_string<'a, T: 'a>(view: T) -> Oco<'a, str>
where
    T: IntoView,
{
    let rendered = view.into_view().render_to_string();
    let comment_removed = remove_html_comments(rendered);
    let hk_removed = remove_hk(comment_removed);
    decode_special_chars(hk_removed)
}

fn remove_noise<'a>(s: Oco<'a, str>, start_delim: &str, end_delim: &str) -> Oco<'a, str> {
    let Some((start, rest)) = s.split_once(start_delim) else {
        return s;
    };
    let mut output_str = start.to_owned();
    let (_, mut s) = rest.split_once(end_delim).unwrap();
    while let Some((start, rest)) = s.split_once(start_delim) {
        output_str.push_str(start);
        let (_, rest) = rest.split_once(end_delim).unwrap();
        s = rest;
    }
    output_str.push_str(s);
    Oco::Owned(output_str)
}

fn remove_html_comments(s: Oco<str>) -> Oco<str> {
    remove_noise(s, "<!--", "-->")
}

fn remove_hk(s: Oco<str>) -> Oco<str> {
    remove_noise(s, " data-hk=\"", "\"")
}

fn split_html_special_char(s: &str) -> Option<(&str, char, &str)> {
    let (before, rest) = s.split_once("&#x")?;
    let (code, after) = rest.split_once(';')?;
    let code = u32::from_str_radix(code, 16).ok()?;
    let ch = char::from_u32(code)?;

    Some((before, ch, after))
}

fn decode_special_chars(s: Oco<str>) -> Oco<str> {
    let Some((before, ch, mut s)) = split_html_special_char(&s) else {
        return s;
    };
    let mut output_str = before.to_owned();
    output_str.push(ch);
    while let Some((before, ch, rest)) = split_html_special_char(s) {
        output_str.push_str(before);
        output_str.push(ch);
        s = rest;
    }
    output_str.push_str(s);
    Oco::Owned(output_str)
}

#[macro_export]
macro_rules! assert_eq_rendered {
    ($left:expr, $($right:tt)*) => {
        assert_eq!(render_to_string($left), $($right)*)
    };
}
