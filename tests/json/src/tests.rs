use crate::i18n::*;
use leptos::attr::any_attribute::AnyAttribute;
use tests_common::*;

#[test]
fn click_to_change_lang() {
    let en = td!(Locale::en, click_to_change_lang);
    assert_eq_rendered!(en, "Click to change language");
    let fr = td!(Locale::fr, click_to_change_lang);
    assert_eq_rendered!(fr, "Cliquez pour changez de langue");
}

#[test]
fn click_count() {
    for count in -5..5 {
        let en = td!(Locale::en, click_count, count);
        assert_eq_rendered!(en, format!("You clicked {} times", count));
        let fr = td!(Locale::fr, click_count, count);
        assert_eq_rendered!(fr, format!("Vous avez cliqué {} fois", count));
    }

    let count = "whatever impl into view";
    let en = td!(Locale::en, click_count, count);
    assert_eq_rendered!(en, format!("You clicked {} times", count));
    let fr = td!(Locale::fr, click_count, count);
    assert_eq_rendered!(fr, format!("Vous avez cliqué {} fois", count));

    let count = view! { <p>"even a view!"</p> };
    let en = td!(Locale::en, click_count, count = count.clone());
    assert_eq_rendered!(en, "You clicked <p>even a view!</p> times");
    let fr = td!(Locale::fr, click_count, count);
    assert_eq_rendered!(fr, "Vous avez cliqué <p>even a view!</p> fois");
}

#[test]
fn click_count_string() {
    for count in -5..5 {
        let en = td_string!(Locale::en, click_count, count);
        assert_eq!(en, format!("You clicked {count} times"));
        let fr = td_string!(Locale::fr, click_count, count);
        assert_eq!(fr, format!("Vous avez cliqué {count} fois"));
    }

    let en = td_string!(Locale::en, click_count, count = "a lot of");
    assert_eq!(en, "You clicked a lot of times");
    let fr = td_string!(Locale::fr, click_count, count = "beaucoups de");
    assert_eq!(fr, "Vous avez cliqué beaucoups de fois");
}

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
fn same_lit_type() {
    let en = td!(Locale::en, same_lit_type);
    assert_eq_rendered!(en, "true");
    let fr = td!(Locale::fr, same_lit_type);
    assert_eq_rendered!(fr, "false");
}

#[test]
fn mixed_lit_type() {
    let en = td!(Locale::en, mixed_lit_type);
    assert_eq_rendered!(en, "59.89");
    let fr = td!(Locale::fr, mixed_lit_type);
    assert_eq_rendered!(fr, "true");
}

#[test]
fn test() {
    let div = |children: ChildrenFn, attrs: Vec<AnyAttribute>| {
        leptos::view! { <div {..attrs}>{children()}</div>}
    };
    let en = td!(Locale::en, comp_with_attrs, <div> = div);
    assert_eq_rendered!(en, "<div id=\"en\">test</div>");
    let fr = td!(Locale::fr, comp_with_attrs, <div> = div);
    assert_eq_rendered!(fr, "<div id=\"fr\">test</div>");
}
