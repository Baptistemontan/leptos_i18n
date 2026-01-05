use crate::i18n::*;
use leptos::attr::any_attribute::AnyAttribute;
use leptos_i18n::display::{Attributes, Children};
use std::fmt::Formatter;
use tests_common::*;

#[test]
fn interpolate_variable_and_comp() {
    let en = td!(Locale::en, interpolate_variable_and_comp, <b> = <span id="test"/>, count = 12);
    assert_eq_rendered!(en, "<span id=\"test\">12</span>");
    let fr = td!(Locale::fr, interpolate_variable_and_comp, <b> = <span/>, count = 34);
    assert_eq_rendered!(fr, "<span>34</span>");
}

#[test]
fn interpolate_variable_and_comp_self_closed() {
    let en = td!(Locale::en, interpolate_variable_and_comp_self_closed, <br/> = <br/>);
    assert_eq_rendered!(en, "hello<br>world");
    let fr = td!(Locale::fr, interpolate_variable_and_comp_self_closed, <br/> = <br/>);
    assert_eq_rendered!(fr, "hello<br>world");
}

#[test]
fn interpolate_variable_and_comp_self_closed_as_string() {
    let en = td_string!(Locale::en, interpolate_variable_and_comp_self_closed, <br/> = "br");
    assert_eq!(en, "hello<br />world");
    let fr = td_string!(Locale::fr, interpolate_variable_and_comp_self_closed, <br/> = "div");
    assert_eq!(fr, "hello<div />world");
}

#[test]
fn non_copy_arg() {
    fn check_impl_fn<T>(_: &impl Fn() -> T) {}

    let count = String::from("count");
    let en = td!(Locale::en, interpolate_variable_and_comp, <b> = <span id="test"/>, count);
    check_impl_fn(&en);
    assert_eq_rendered!(en, "<span id=\"test\">count</span>");

    let count = String::from("count");
    let fr = td!(Locale::fr, interpolate_variable_and_comp, <b> = <span/>, count);
    check_impl_fn(&fr);
    assert_eq_rendered!(fr, "<span>count</span>");
}

#[test]
fn test_comp_with_attributes_with_function() {
    // with function
    let div = |children: ChildrenFn, attrs: Vec<AnyAttribute>| {
        leptos::view! { <div {..attrs}>{children()}</div>}
    };
    let en = td!(Locale::en, comp_with_attrs, <div> = div, id = "en");
    assert_eq_rendered!(en, "<div id=\"en\" foo=\"bar\">test</div>");
    let fr = td!(Locale::fr, comp_with_attrs, <div> = div, id = "fr");
    assert_eq_rendered!(fr, "<div id=\"fr\" bool true_bool num=\"17\">test</div>");
    let fr = td!(Locale::fr, comp_with_attrs, <div> = div, id = || "foo bar");
    assert_eq_rendered!(
        fr,
        "<div id=\"foo bar\" bool true_bool num=\"17\">test</div>"
    );
}

#[test]
fn test_comp_with_attributes_with_direct_comp() {
    let en = td!(Locale::en, comp_with_attrs, <div> = <div />, id = "en");
    assert_eq_rendered!(en, "<div id=\"en\" foo=\"bar\">test</div>");
    let fr = td!(Locale::fr, comp_with_attrs, <div> = <div />, id = "foo bar");
    assert_eq_rendered!(
        fr,
        "<div id=\"foo bar\" bool true_bool num=\"17\">test</div>"
    );
}

#[test]
fn test_comp_with_attributes_as_string() {
    let en = td_string!(Locale::en, comp_with_attrs, <div> = "div", id = "en");
    assert_eq!(en, "<div id=\"en\" foo=\"bar\">test</div>");
    let fr = td_string!(Locale::fr, comp_with_attrs, <div> = "div", id = "foo_bar");
    assert_eq!(
        fr,
        "<div id=\"foo_bar\" bool true_bool num=\"17\">test</div>"
    );
}

#[test]
fn test_comp_with_attributes_variable_to_string() {
    let en = td_string!(Locale::en, comp_with_attrs, <div> = "div", id = true);
    assert_eq!(en, "<div id foo=\"bar\">test</div>");

    let en = td_string!(Locale::en, comp_with_attrs, <div> = "div", id = false);
    assert_eq!(en, "<div foo=\"bar\">test</div>");

    let en = td_string!(Locale::en, comp_with_attrs, <div> = "div", id = 56);
    assert_eq!(en, "<div id=\"56\" foo=\"bar\">test</div>");

    let en = td_string!(Locale::en, comp_with_attrs, <div> = "div", id = 56.78);
    assert_eq!(en, "<div id=\"56.78\" foo=\"bar\">test</div>");

    let en = td_string!(Locale::en, comp_with_attrs, <div> = "div", id = -56.78);
    assert_eq!(en, "<div id=\"-56.78\" foo=\"bar\">test</div>");

    let en = td_string!(Locale::en, comp_with_attrs, <div> = "div", id = Some("test"));
    assert_eq!(en, "<div id=\"test\" foo=\"bar\">test</div>");

    let en = td_string!(Locale::en, comp_with_attrs, <div> = "div", id = Some(true));
    assert_eq!(en, "<div id foo=\"bar\">test</div>");

    let en = td_string!(Locale::en, comp_with_attrs, <div> = "div", id = Some(false));
    assert_eq!(en, "<div foo=\"bar\">test</div>");

    let en = td_string!(Locale::en, comp_with_attrs, <div> = "div", id = Some(56.78));
    assert_eq!(en, "<div id=\"56.78\" foo=\"bar\">test</div>");

    let en = td_string!(Locale::en, comp_with_attrs, <div> = "div", id = Option::<u8>::None);
    assert_eq!(en, "<div foo=\"bar\">test</div>");
}

#[test]
fn test_comp_with_attributes_as_string_with_fn() {
    let div = |f: &mut Formatter, children: leptos_i18n::display::DynDisplayFn| {
        write!(f, "<div>")?;
        children(f)?;
        write!(f, "</div>")
    };
    let en = td_string!(Locale::en, comp_with_attrs, <div>, id = "en");
    assert_eq!(en, "<div>test</div>");
    let div = |f: &mut Formatter, children: Children, attrs: Attributes| {
        write!(f, "<div{attrs}>{children}</div>")
    };
    let fr = td_string!(Locale::fr, comp_with_attrs, <div>, id = "foo_bar");
    assert_eq!(
        fr,
        "<div id=\"foo_bar\" bool true_bool num=\"17\">test</div>"
    );
}

#[test]
fn test_comp_with_attributes_self_closed() {
    let en = td!(Locale::en, comp_with_attrs_self_closed, <br/> = <br />, id = "en");
    assert_eq_rendered!(en, "before<br id=\"test\">after");
    let fr = td!(Locale::fr, comp_with_attrs_self_closed, <br/> = <br />, id = "foo bar");
    assert_eq_rendered!(fr, "before<br id=\"foo bar\">after");
}

#[test]
fn test_comp_with_attributes_self_closed_as_string() {
    let en = td_string!(Locale::en, comp_with_attrs_self_closed, <br/> = "br", id = "en");
    assert_eq!(en, "before<br id=\"test\" />after");
    let fr = td_string!(Locale::fr, comp_with_attrs_self_closed, <br/> = "br", id = "foo bar");
    assert_eq!(fr, "before<br id=\"foo bar\" />after");
}

#[test]
fn test_comp_with_attributes_self_closed_as_string_with_fn() {
    let br = |f: &mut Formatter| write!(f, "<br />");
    let en = td_string!(Locale::en, comp_with_attrs_self_closed, <br/>, id = "en");
    assert_eq!(en, "before<br />after");
    let br =
        |f: &mut Formatter, attrs: leptos_i18n::display::Attributes| write!(f, "<br{attrs} />");
    let fr = td_string!(Locale::fr, comp_with_attrs_self_closed, <br/>, id = "foo bar");
    assert_eq!(fr, "before<br id=\"foo bar\" />after");
}

#[test]
fn test_comp_with_escaped_str_attribute() {
    let en = td!(Locale::en, comp_with_escaped_str_attrs, <div> = <div />, foo = "");
    assert_eq_rendered!(en, "<div foo=\"\\&quot;bar\">test</div>");

    let fr = td!(Locale::en, comp_with_escaped_str_attrs, <div> = <div />, foo = "\"bar");
    assert_eq_rendered!(fr, "<div foo=\"\\&quot;bar\">test</div>");
}

#[test]
fn test_comp_with_escaped_str_attribute_as_str() {
    let en = td_string!(Locale::en, comp_with_escaped_str_attrs, <div> = "div", foo = "");
    assert_eq!(en, "<div foo=\"\\\"bar\">test</div>");

    let fr = td_string!(Locale::en, comp_with_escaped_str_attrs, <div> = "div", foo = "\"bar");
    assert_eq!(fr, "<div foo=\"\\\"bar\">test</div>");
}
