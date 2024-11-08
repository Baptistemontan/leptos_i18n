//! This module contain some helpers to format component using the `td_string!` macro.

use std::fmt;

// use leptos::Attribute;

/// This trait is used when interpolating component with the `td_string!` macro
pub trait DisplayComponent {
    /// Takes as an input a formatter and a function to format the component children
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, children: T) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result;
}

impl<F> DisplayComponent for F
where
    F: Fn(&mut fmt::Formatter<'_>, &dyn Fn(&mut fmt::Formatter<'_>) -> fmt::Result) -> fmt::Result,
{
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, children: T) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        self(f, &children)
    }
}

impl DisplayComponent for &str {
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, children: T) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        write!(f, "<{}>", self)?;
        children(f)?;
        write!(f, "</{}>", self)
    }
}

impl DisplayComponent for String {
    #[inline]
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, children: T) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        self.as_str().fmt(f, children)
    }
}

/// This struct is made to be used with the `t_string!` macro when interpolating a component
///
/// ```rust
/// #   leptos_i18n::declare_locales! {
/// #       path: leptos_i18n,
/// #       interpolate_display,
/// #       default: "en",
/// #       locales: ["en"],
/// #       en: {
/// #           key: "highlight <b>me</b>",
/// #       },
/// #   };
/// # use i18n::*;
/// use leptos_i18n::display::DisplayComp;
/// // key = "highlight <b>me</b>"
/// let t = td_string!(Locale::en, key, <b> = DisplayComp::new("div", &[("id", "my_div")]));
/// assert_eq!(t.to_string(), "highlight <div id=\"my_div\">me</div>");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct DisplayComp<'a> {
    comp_name: &'a str,
    attrs: &'a [(&'a str, &'a str)],
}

impl<'a> DisplayComp<'a> {
    #[inline]
    /// Create a new `DisplayComp`
    pub fn new(comp_name: &'a str, attrs: &'a [(&'a str, &'a str)]) -> Self {
        Self { comp_name, attrs }
    }
}

impl DisplayComponent for DisplayComp<'_> {
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, children: T) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        write!(f, "<{}", self.comp_name)?;
        for (attr_name, attr) in self.attrs {
            write!(f, " {}=\"{}\"", attr_name, attr)?;
        }
        f.write_str(">")?;
        children(f)?;
        write!(f, "</{}>", self.comp_name)
    }
}
