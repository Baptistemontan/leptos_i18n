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

impl<'a> DisplayComponent for &'a str {
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

// /// This struct is made to be used with the `td_string!` macro when interpolating a component
// ///
// /// ```rust,ignore
// /// /* key = "highlight <b>me</b>" */
// /// let t = td_string!(locale, key, <b> = DisplayComp("div"));
// /// assert_eq!(t.to_string(), "highlight <div>me</div>");
// /// ```
// #[derive(Debug, Clone, Copy)]
// pub struct DisplayComp<'a> {
//     comp_name: &'a str,
//     attrs: &'a [(&'static str, Attribute)],
// }

// impl<'a> DisplayComp<'a> {
//     #[inline]
//     /// Create a new `DisplayComp`
//     pub fn new(comp_name: &'a str, attrs: &'a [(&'static str, Attribute)]) -> Self {
//         Self { comp_name, attrs }
//     }
// }

// impl DisplayComponent for DisplayComp<'_> {
//     fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, children: T) -> fmt::Result
//     where
//         T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
//     {
//         write!(f, "<{}", self.comp_name)?;
//         for (attr_name, attr) in self.attrs {
//             let value = attr.as_value_string(attr_name);
//             write!(f, " {}", value)?;
//         }
//         f.write_str(">")?;
//         children(f)?;
//         write!(f, "</{}>", self.comp_name)
//     }
// }
