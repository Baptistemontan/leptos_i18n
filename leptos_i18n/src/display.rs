//! This module contain some helpers to format component using the `td_string!` macro.

use std::fmt;

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

/// This struct is made to be used with the `td_string!` macro when interpolating a component
///
/// ```rust,ignore
/// /* key = "highlight <b>me</b>" */
/// let t = td_string!(locale, key, <b> = DisplayComp("div"));
/// assert_eq!(t.to_string(), "highlight <div>me</div>");
/// ```
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct DisplayComp<'a>(pub &'a str);

impl<'a> DisplayComp<'a> {
    #[inline]
    /// Create a new `DisplayComp`
    pub fn new(component_name: &'a str) -> Self {
        DisplayComp(component_name)
    }
}

impl DisplayComponent for DisplayComp<'_> {
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, children: T) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        write!(f, "<{}>", self.0)?;
        children(f)?;
        write!(f, "</{}>", self.0)
    }
}
