//! This module contain some helpers to format component using the `td_string!` macro.

use std::{
    fmt::{self, Debug, Display},
    marker::PhantomData,
};

/// Function that takes a formatter to format things like children or attributes
pub type DynDisplayFn<'a> = &'a dyn Fn(&mut fmt::Formatter<'_>) -> fmt::Result;

/// Attributes, an array of `DynDisplayFn`
#[derive(Clone, Copy)]
pub struct Attributes<'a>(pub &'a [DynDisplayFn<'a>]);

impl Debug for Attributes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Attributes").finish()
    }
}

impl Display for Attributes<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for attr in self.0 {
            attr(f)?;
        }
        Ok(())
    }
}

/// A struct that represent a children to format with components
#[derive(Clone, Copy)]
pub struct Children<'a>(pub DynDisplayFn<'a>);

impl Debug for Children<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Children").finish()
    }
}

impl Display for Children<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0(f)
    }
}

#[doc(hidden)]
pub struct WithAttributes<M>(PhantomData<M>);
#[doc(hidden)]
pub struct WithoutAttributes<M>(PhantomData<M>);
#[doc(hidden)]
pub struct WithChildren<M>(PhantomData<M>);
#[doc(hidden)]
pub struct WithoutChildren;

#[doc(hidden)]
pub struct ChildrenFn;
#[doc(hidden)]
pub struct DisplayChildren;

/// This trait is used when interpolating component with the `td_string!` macro
pub trait DisplayComponent<M> {
    /// Takes as an input a formatter and a function to format the component with children and attributes
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, children: T, attrs: Attributes) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result;

    /// Format a self-closing component (no children)
    fn fmt_self_closing(&self, f: &mut fmt::Formatter<'_>, attrs: Attributes) -> fmt::Result;
}

impl<F> DisplayComponent<WithAttributes<WithChildren<DisplayChildren>>> for F
where
    F: Fn(&mut fmt::Formatter<'_>, Children, Attributes) -> fmt::Result,
{
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, children: T, attrs: Attributes) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        self(f, Children(&children), attrs)
    }

    fn fmt_self_closing(&self, f: &mut fmt::Formatter<'_>, attrs: Attributes) -> fmt::Result {
        self(f, Children(&|_| Ok(())), attrs)
    }
}

impl<F> DisplayComponent<WithoutAttributes<WithChildren<DisplayChildren>>> for F
where
    F: Fn(&mut fmt::Formatter<'_>, Children) -> fmt::Result,
{
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, children: T, _attrs: Attributes) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        self(f, Children(&children))
    }

    fn fmt_self_closing(&self, f: &mut fmt::Formatter<'_>, _attrs: Attributes) -> fmt::Result {
        self(f, Children(&|_| Ok(())))
    }
}

impl<F> DisplayComponent<WithAttributes<WithChildren<ChildrenFn>>> for F
where
    F: Fn(&mut fmt::Formatter<'_>, DynDisplayFn, Attributes) -> fmt::Result,
{
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, children: T, attrs: Attributes) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        self(f, &children, attrs)
    }

    fn fmt_self_closing(&self, f: &mut fmt::Formatter<'_>, attrs: Attributes) -> fmt::Result {
        self(f, &|_| Ok(()), attrs)
    }
}

impl<F> DisplayComponent<WithoutAttributes<WithChildren<ChildrenFn>>> for F
where
    F: Fn(&mut fmt::Formatter<'_>, DynDisplayFn) -> fmt::Result,
{
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, children: T, _attrs: Attributes) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        self(f, &children)
    }

    fn fmt_self_closing(&self, f: &mut fmt::Formatter<'_>, _attrs: Attributes) -> fmt::Result {
        self(f, &|_| Ok(()))
    }
}

impl<F> DisplayComponent<WithAttributes<WithoutChildren>> for F
where
    F: Fn(&mut fmt::Formatter<'_>, Attributes) -> fmt::Result,
{
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, _children: T, attrs: Attributes) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        self(f, attrs)
    }

    fn fmt_self_closing(&self, f: &mut fmt::Formatter<'_>, attrs: Attributes) -> fmt::Result {
        self(f, attrs)
    }
}

impl<F> DisplayComponent<WithoutAttributes<WithoutChildren>> for F
where
    F: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
{
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, _children: T, _attrs: Attributes) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        self(f)
    }

    fn fmt_self_closing(&self, f: &mut fmt::Formatter<'_>, _attrs: Attributes) -> fmt::Result {
        self(f)
    }
}

impl DisplayComponent<()> for str {
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, children: T, attrs: Attributes) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        write!(f, "<{self}{attrs}>")?;
        children(f)?;
        write!(f, "</{self}>")
    }

    fn fmt_self_closing(&self, f: &mut fmt::Formatter<'_>, attrs: Attributes) -> fmt::Result {
        write!(f, "<{self}{attrs} />")
    }
}

impl DisplayComponent<()> for String {
    #[inline]
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, children: T, attrs: Attributes) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        DisplayComponent::fmt(self.as_str(), f, children, attrs)
    }

    #[inline]
    fn fmt_self_closing(&self, f: &mut fmt::Formatter<'_>, attrs: Attributes) -> fmt::Result {
        self.as_str().fmt_self_closing(f, attrs)
    }
}

impl DisplayComponent<()> for &str {
    #[inline]
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, children: T, attrs: Attributes) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        <str as DisplayComponent<()>>::fmt(self, f, children, attrs)
    }

    #[inline]
    fn fmt_self_closing(&self, f: &mut fmt::Formatter<'_>, attrs: Attributes) -> fmt::Result {
        <str as DisplayComponent<()>>::fmt_self_closing(self, f, attrs)
    }
}

impl DisplayComponent<()> for &String {
    #[inline]
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, children: T, attrs: Attributes) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        DisplayComponent::fmt(self.as_str(), f, children, attrs)
    }

    #[inline]
    fn fmt_self_closing(&self, f: &mut fmt::Formatter<'_>, attrs: Attributes) -> fmt::Result {
        self.as_str().fmt_self_closing(f, attrs)
    }
}

/// This struct is made to be used with the `t_string!` macro when interpolating a component
///
#[cfg_attr(feature = "dynamic_load", doc = "```rust, ignore")]
#[cfg_attr(not(feature = "dynamic_load"), doc = "```rust")]
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
    fn write_attrs(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (attr_name, attr) in self.attrs {
            write!(f, " {attr_name}=\"{attr}\"")?;
        }
        Ok(())
    }
}

impl DisplayComponent<()> for DisplayComp<'_> {
    fn fmt<T>(&self, f: &mut fmt::Formatter<'_>, children: T, attrs: Attributes) -> fmt::Result
    where
        T: Fn(&mut fmt::Formatter<'_>) -> fmt::Result,
    {
        write!(f, "<{}{attrs}", self.comp_name)?;
        self.write_attrs(f)?;
        write!(f, ">")?;
        children(f)?;
        write!(f, "</{}>", self.comp_name)
    }

    fn fmt_self_closing(&self, f: &mut fmt::Formatter<'_>, attrs: Attributes) -> fmt::Result {
        write!(f, "<{}{attrs}", self.comp_name)?;
        self.write_attrs(f)?;
        write!(f, " />")
    }
}
