use proc_macro2::TokenStream;
use quote::quote;
use std::{
    any::{Any, TypeId},
    cmp::Ordering,
    collections::HashMap,
    fmt::{Debug, Display},
    rc::Rc,
};

use crate::{
    parse_locales::error::{Diagnostics, Error},
    utils::{Key, KeyPath},
};

pub mod currency;
pub mod datetime;
pub mod list;
pub mod nums;

#[derive(Debug)]
pub struct DuplicateFormatterErr {
    name: &'static str,
}

pub struct Formatters {
    formatters: HashMap<&'static str, Rc<dyn DynFormatter>>,
}

impl Default for Formatters {
    fn default() -> Self {
        Self::new()
    }
}

impl Formatters {
    pub fn new_empty() -> Self {
        Self {
            formatters: HashMap::new(),
        }
    }

    pub fn insert_formatter<F: Formatter>(
        &mut self,
        formatter: F,
    ) -> Result<&mut Self, DuplicateFormatterErr> {
        if self
            .formatters
            .insert(F::NAME, Rc::new(formatter))
            .is_some()
        {
            Err(DuplicateFormatterErr { name: F::NAME })
        } else {
            Ok(self)
        }
    }

    fn new_populated() -> Result<Self, DuplicateFormatterErr> {
        let mut this = Self::new_empty();
        this.insert_formatter(currency::CurrencyFormatterParser)?
            .insert_formatter(nums::NumberFormatterParser)?
            .insert_formatter(list::ListFormatterParser)?
            .insert_formatter(datetime::DateTimeFormatterParser)?
            .insert_formatter(datetime::DateFormatterParser)?
            .insert_formatter(datetime::TimeFormatterParser)?;
        Ok(this)
    }

    pub fn new() -> Self {
        Self::new_populated().unwrap()
    }

    pub fn parse(
        &self,
        locale: &Key,
        key_path: &KeyPath,
        name: &str,
        args: &[(&str, Option<&str>)],
        diag: &Diagnostics,
    ) -> ValueFormatter {
        let Some(formatter) = self.formatters.get(name) else {
            diag.emit_error(Error::UnknownFormatter {
                name: name.to_string(),
                locale: locale.clone(),
                key_path: key_path.clone(),
            });
            return ValueFormatter::Dummy;
        };

        formatter.parse(locale, key_path, args, diag)
    }

    #[doc(hidden)]
    pub fn parse_from_tt(
        &self,
        name: &str,
        args: &[(syn::Ident, Option<syn::Ident>)],
    ) -> Option<Result<ValueFormatter, &'static str>> {
        let f = self.formatters.get(name)?;
        Some(f.parse_from_tt(args))
    }
}

impl Debug for Formatters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.formatters.keys()).finish()
    }
}

pub trait Formatter: 'static {
    const NAME: &str;

    type ToTokens: FormatterToTokens;
    type ParseError: ToString;

    fn parse_with_diagnostics(
        &self,
        locale: &Key,
        key_path: &KeyPath,
        args: &[(&str, Option<&str>)],
        diag: &Diagnostics,
    ) -> Option<Self::ToTokens> {
        match Self::parse_args(self, args) {
            Ok(v) => Some(v),
            Err(err) => {
                diag.emit_error(Error::custom(locale.clone(), key_path.clone(), err));
                None
            }
        }
    }

    fn parse_args(&self, args: &[(&str, Option<&str>)])
        -> Result<Self::ToTokens, Self::ParseError>;

    #[doc(hidden)]
    fn parse_from_tt<'a, S>(&self, _args: &[(S, Option<S>)]) -> Result<Self::ToTokens, &'static str>
    where
        S: PartialEq + PartialEq<&'a str>,
    {
        unimplemented!()
    }
}

trait DynFormatter {
    fn parse(
        &self,
        locale: &Key,
        key_path: &KeyPath,
        args: &[(&str, Option<&str>)],
        diag: &Diagnostics,
    ) -> ValueFormatter;

    fn parse_from_tt(
        &self,
        args: &[(syn::Ident, Option<syn::Ident>)],
    ) -> Result<ValueFormatter, &'static str>;
}

impl<T: Formatter + ?Sized> DynFormatter for T {
    fn parse(
        &self,
        locale: &Key,
        key_path: &KeyPath,
        args: &[(&str, Option<&str>)],
        diag: &Diagnostics,
    ) -> ValueFormatter {
        match T::parse_with_diagnostics(self, locale, key_path, args, diag) {
            Some(f) => ValueFormatter::Formatted {
                formatter_name: T::NAME,
                to_tokens: Rc::new(f),
            },
            None => ValueFormatter::Dummy,
        }
    }

    fn parse_from_tt(
        &self,
        args: &[(syn::Ident, Option<syn::Ident>)],
    ) -> Result<ValueFormatter, &'static str> {
        let f = T::parse_from_tt(self, args)?;
        Ok(ValueFormatter::Formatted {
            formatter_name: T::NAME,
            to_tokens: Rc::new(f),
        })
    }
}

pub trait FormatterToTokens: Any {
    fn to_view(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream;
    fn to_display(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream;
    fn to_fmt(&self, key: &Key, locale_field: &Key) -> TokenStream;
    fn view_bounds(&self) -> TokenStream;
    fn display_bounds(&self) -> TokenStream;
    fn is(&self, type_id: TypeId) -> bool {
        self.type_id() == type_id
    }
}

#[derive(Default, Clone)]
pub enum ValueFormatter {
    /// NOT A FORMATTER, this formatter will emit no bound, this is for dummy code to reduce errors
    Dummy,
    #[default]
    None,
    Formatted {
        formatter_name: &'static str,
        to_tokens: Rc<dyn FormatterToTokens>,
    },
}

impl ValueFormatter {
    pub fn is<T: Any>(&self) -> bool {
        match self {
            Self::Formatted { to_tokens, .. } => to_tokens.is(TypeId::of::<T>()),
            _ => false,
        }
    }

    pub fn to_bound(&self) -> TokenStream {
        match self {
            Self::None => quote!(l_i18n_crate::__private::InterpolateVar),
            Self::Dummy => quote!(l_i18n_crate::__private::AnyBound),
            Self::Formatted { to_tokens, .. } => to_tokens.view_bounds(),
        }
    }
    pub fn to_string_bound(&self) -> TokenStream {
        match self {
            Self::None => quote!(::std::fmt::Display),
            Self::Dummy => quote!(l_i18n_crate::__private::AnyBound),
            Self::Formatted { to_tokens, .. } => to_tokens.display_bounds(),
        }
    }

    pub fn var_to_view(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        match self {
            Self::None => {
                quote!(#key)
            }
            Self::Dummy => unreachable!(
                "var_to_view function should not have been called on a dummy formatter"
            ),
            Self::Formatted { to_tokens, .. } => to_tokens.to_view(key, locale_field),
        }
    }
    pub fn var_fmt(&self, key: &Key, locale_field: &Key) -> TokenStream {
        match self {
            Self::None => {
                quote!(core::fmt::Display::fmt(#key, __formatter))
            }
            Self::Dummy => {
                unreachable!("var_fmt function should not have been called on a dummy formatter")
            }
            Self::Formatted { to_tokens, .. } => to_tokens.to_fmt(key, locale_field),
        }
    }
    pub fn var_to_display(self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        match self {
            Self::None => unreachable!(
                "var_to_display function should not have been called on a variable with no formatter."
            ),
            Self::Dummy => {
                unreachable!(
                    "var_to_display function should not have been called on a dummy formatter"
                )
            }
            Self::Formatted { to_tokens, .. } => to_tokens.to_display(key, locale_field)
        }
    }
}

impl Eq for ValueFormatter {}

impl Ord for ValueFormatter {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (ValueFormatter::Dummy, ValueFormatter::Dummy) => Ordering::Equal,
            (ValueFormatter::Dummy, ValueFormatter::None) => Ordering::Less,
            (ValueFormatter::Dummy, ValueFormatter::Formatted { .. }) => Ordering::Less,
            (ValueFormatter::None, ValueFormatter::Dummy) => Ordering::Greater,
            (ValueFormatter::None, ValueFormatter::None) => Ordering::Equal,
            (ValueFormatter::None, ValueFormatter::Formatted { .. }) => Ordering::Less,
            (ValueFormatter::Formatted { .. }, ValueFormatter::Dummy) => Ordering::Greater,
            (ValueFormatter::Formatted { .. }, ValueFormatter::None) => Ordering::Greater,
            (
                ValueFormatter::Formatted {
                    formatter_name: self_name,
                    ..
                },
                ValueFormatter::Formatted {
                    formatter_name: other_name,
                    ..
                },
            ) => self_name.cmp(other_name),
        }
    }
}

impl PartialOrd for ValueFormatter {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Debug for ValueFormatter {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            ValueFormatter::Dummy => f.write_str("Dummy"),
            ValueFormatter::None => f.write_str("None"),
            ValueFormatter::Formatted { formatter_name, .. } => f
                .debug_struct("Formatted")
                .field("formatter_name", &formatter_name)
                .finish(),
        }
    }
}

impl PartialEq for ValueFormatter {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Dummy, Self::Dummy) => true,
            (Self::None, Self::None) => true,
            (
                Self::Formatted {
                    formatter_name: self_name,
                    ..
                },
                Self::Formatted {
                    formatter_name: other_name,
                    ..
                },
            ) => self_name == other_name,
            _ => false,
        }
    }
}

impl Display for DuplicateFormatterErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Duplicate formatter: {:?}", self.name)
    }
}

pub(crate) fn from_args_helper<'a, T: Default, S: PartialEq + PartialEq<&'a str>>(
    args: &[(S, Option<S>)],
    name: &'a str,
    f: impl Fn(Option<&S>) -> Option<T>,
) -> T {
    for (arg_name, value) in args {
        if arg_name != &name {
            continue;
        }
        if let Some(v) = f(value.as_ref()) {
            return v;
        }
    }
    Default::default()
}

macro_rules! impl_from_args {
    ($name:literal, $($arg_name:literal => $value:expr,)*) => {
        pub fn from_args<'a, S: PartialEq + PartialEq<&'a str>>(args: &[(S, Option<S>)]) -> Self {
        $crate::formatters::from_args_helper(args, $name, |arg| {
            $(
                if arg.is_some_and(|arg| arg == &$arg_name) {
                    Some($value)
                } else
            )*
            {
                None
            }
        })
    }
    }
}

macro_rules! impl_to_tokens {
    (
        $type_name:ident,
        $path_prefix:expr,
        { $($variant:ident $(( $($inner:ident),+ ))?),+ $(,)? }
    ) => {
        impl ToTokens for $type_name {
            fn to_token_stream(&self) -> TokenStream {
                match self {
                    $(
                        $type_name::$variant $(( $($inner),+ ))? => {
                            quote!($path_prefix::$variant $(( $(#$inner),+ ))?)
                        },
                    )+
                }
            }

            fn to_tokens(&self, tokens: &mut TokenStream) {
                let ts = Self::to_token_stream(self);
                tokens.extend(ts);
            }
        }
    };
}

macro_rules! impl_formatter {
    ($t: ident, $name: literal, $f:ident  $( ( $($s: ident),* $(,)?) )?, $feature: literal, $err: literal) => {
        impl Formatter for $t {
            const NAME: &str = $name;
            type ParseError = std::convert::Infallible;
            type ToTokens = $f;
            fn parse_with_diagnostics(
                &self,
                locale: &Key,
                key_path: &KeyPath,
                args: &[(&str, Option<&str>)],
                diag: &Diagnostics,
            ) -> Option<Self::ToTokens> {
                if cfg!(not(feature = $feature)) {
                    diag.emit_error(Error::DisabledFormatter {
                        locale: locale.clone(),
                        key_path: key_path.clone(),
                        formatter_err: $err,
                    });
                    return None;
                }

                Self::parse_args(self, args).ok()
            }
            fn parse_args(
                &self,
                args: &[(&str, Option<&str>)],
            ) -> Result<Self::ToTokens, Self::ParseError> {
                Ok($f
                    $( (
                        $( $s::from_args(args) ),*
                    ) )?
                )
            }
        }
    };
}

pub(crate) use impl_formatter;
pub(crate) use impl_from_args;
pub(crate) use impl_to_tokens;
