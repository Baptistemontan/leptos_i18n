use proc_macro2::TokenStream;
use quote::quote;
use std::{
    any::{Any, TypeId},
    cmp::Ordering,
    collections::HashMap,
    fmt::{Debug, Display},
    rc::Rc,
};
use syn::{Ident, Token, punctuated::Punctuated, spanned::Spanned};

use crate::{
    parse_locales::{
        error::{Diagnostics, Error},
        parsed_value::Context,
    },
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

    pub fn parse(&self, ctx: &Context, name: &str, args: &[(&str, Option<&str>)]) -> VarBounds {
        let Some(formatter) = self.formatters.get(name) else {
            ctx.diag.emit_error(Error::UnknownFormatter {
                name: name.to_string(),
                locale: ctx.locale.clone(),
                key_path: ctx.key_path.clone(),
            });
            return VarBounds::Dummy;
        };

        formatter.parse(ctx, args)
    }

    #[doc(hidden)]
    pub fn parse_from_tt(
        &self,
        formatter_name: syn::Ident,
        args: Option<Punctuated<(Ident, Option<Ident>), Token![;]>>,
    ) -> syn::Result<VarBounds> {
        let name = formatter_name.to_string();
        let Some(f) = self.formatters.get(&*name) else {
            return Err(syn::Error::new(formatter_name.span(), "unknown formatter"));
        };
        f.parse_from_tt(formatter_name.span(), args)
    }
}

impl Debug for Formatters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_set().entries(self.formatters.keys()).finish()
    }
}

pub trait Formatter: 'static {
    const DISABLED: Option<&str> = None;
    const NAME: &str;

    type Builder;
    type Field<'a>;
    type ToTokens: FormatterToTokens;
    type ParseError: Display;

    fn parse_with_diagnostics(
        &self,
        locale: &Key,
        key_path: &KeyPath,
        args: &[(&str, Option<&str>)],
        diag: &Diagnostics,
    ) -> Option<Self::ToTokens> {
        if let Some(formatter_err) = Self::DISABLED {
            diag.emit_error(Error::DisabledFormatter {
                locale: locale.clone(),
                key_path: key_path.clone(),
                formatter_err,
            });
        }
        let mut builder = self.builder();
        let mut errored = false;
        for (arg_name, arg) in args {
            let field = match self.parse_arg_name(arg_name) {
                Ok(field) => field,
                Err(err) => {
                    diag.emit_error(Error::InvalidFormatterArgName {
                        locale: locale.clone(),
                        key_path: key_path.clone(),
                        name: arg_name.to_string(),
                        err: err.to_string(),
                    });
                    errored = true;
                    continue;
                }
            };
            if let Err(err) = self.parse_arg(&mut builder, field, *arg) {
                diag.emit_error(Error::InvalidFormatterArg {
                    locale: locale.clone(),
                    key_path: key_path.clone(),
                    arg_name: arg_name.to_string(),
                    arg: arg.map(str::to_string),
                    err: err.to_string(),
                });
            }
        }
        if errored {
            None
        } else {
            match self.build(builder) {
                Ok(tt) => Some(tt),
                Err(err) => {
                    diag.emit_error(Error::InvalidFormatter {
                        locale: locale.clone(),
                        key_path: key_path.clone(),
                        err: err.to_string(),
                    });
                    None
                }
            }
        }
    }

    fn builder(&self) -> Self::Builder;

    fn parse_arg_name<'a>(
        &self,
        argument_name: &'a str,
    ) -> Result<Self::Field<'a>, Self::ParseError>;

    fn parse_arg(
        &self,
        builder: &mut Self::Builder,
        field: Self::Field<'_>,
        arg: Option<&str>,
    ) -> Result<(), Self::ParseError>;

    fn build(&self, builder: Self::Builder) -> Result<Self::ToTokens, Self::ParseError>;
}

trait DynFormatter {
    fn parse(&self, ctx: &Context, args: &[(&str, Option<&str>)]) -> VarBounds;

    fn parse_from_tt(
        &self,
        formatter_span: proc_macro2::Span,
        args: Option<Punctuated<(Ident, Option<Ident>), Token![;]>>,
    ) -> syn::Result<VarBounds>;
}

impl<T: Formatter + ?Sized> DynFormatter for T {
    fn parse(&self, ctx: &Context, args: &[(&str, Option<&str>)]) -> VarBounds {
        match T::parse_with_diagnostics(self, ctx.locale, ctx.key_path, args, ctx.diag) {
            Some(f) => VarBounds::Formatted {
                formatter_name: T::NAME,
                to_tokens: Rc::new(f),
            },
            None => VarBounds::Dummy,
        }
    }

    fn parse_from_tt(
        &self,
        formatter_span: proc_macro2::Span,
        args: Option<Punctuated<(Ident, Option<Ident>), Token![;]>>,
    ) -> syn::Result<VarBounds> {
        if let Some(formatter_err) = Self::DISABLED {
            return Err(syn::Error::new(formatter_span, formatter_err));
        }
        let mut builder = self.builder();
        if let Some(args) = args {
            for (arg_name, arg) in args {
                let arg_name_str = arg_name.to_string();
                let field = match self.parse_arg_name(&arg_name_str) {
                    Ok(field) => field,
                    Err(err) => {
                        return Err(syn::Error::new(arg_name.span(), err));
                    }
                };
                let arg_str = arg.as_ref().map(|i| i.to_string());
                if let Err(err) = self.parse_arg(&mut builder, field, arg_str.as_deref()) {
                    return Err(syn::Error::new(
                        arg.as_ref().unwrap_or(&arg_name).span(),
                        err,
                    ));
                }
            }
        }
        match self.build(builder) {
            Ok(f) => Ok(VarBounds::Formatted {
                formatter_name: "",
                to_tokens: Rc::new(f),
            }),
            Err(err) => Err(syn::Error::new(formatter_span.span(), err)),
        }
    }
}

pub trait FormatterToTokens: Any {
    fn to_view(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream;
    fn view_bounds(&self) -> TokenStream;

    fn to_fmt(&self, key: &Key, locale_field: &Key) -> TokenStream;
    fn fmt_bounds(&self) -> TokenStream;

    #[doc(hidden)]
    fn is(&self, type_id: TypeId) -> bool {
        self.type_id() == type_id
    }
    #[doc(hidden)]
    fn to_impl_display(&self, _key: &syn::Ident, _locale_field: &syn::Ident) -> TokenStream {
        // internals for the t_format! macro, custom formatters aren't possible there.
        unimplemented!()
    }
}

#[derive(Default, Clone)]
pub enum VarBounds {
    /// NOT A FORMATTER, this formatter will emit no bound, this is for dummy code to reduce errors
    Dummy,
    #[default]
    None,
    AttributeValue,
    Formatted {
        formatter_name: &'static str,
        to_tokens: Rc<dyn FormatterToTokens>,
    },
}

impl VarBounds {
    pub fn is<T: Any>(&self) -> bool {
        match self {
            Self::Formatted { to_tokens, .. } => to_tokens.is(TypeId::of::<T>()),
            _ => false,
        }
    }

    pub fn view_bounds(&self) -> TokenStream {
        match self {
            Self::None => quote!(l_i18n_crate::__private::InterpolateVar),
            Self::AttributeValue => quote!(l_i18n_crate::reexports::leptos::attr::AttributeValue),
            Self::Dummy => quote!(l_i18n_crate::__private::AnyBound),
            Self::Formatted { to_tokens, .. } => to_tokens.view_bounds(),
        }
    }
    pub fn fmt_bounds(&self) -> TokenStream {
        match self {
            Self::None | Self::AttributeValue => quote!(::std::fmt::Display),
            Self::Dummy => quote!(l_i18n_crate::__private::AnyBound),
            Self::Formatted { to_tokens, .. } => to_tokens.fmt_bounds(),
        }
    }

    pub fn var_to_view(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        match self {
            Self::AttributeValue => {
                unreachable!("attributes values should be rendered by the component renderer.")
            }
            Self::None => {
                quote!(#key)
            }
            Self::Dummy => {
                quote!({ let _ = #key; core::unimplemented!("Dummy formatter, parsing of a formatter must have failed.") })
            }
            Self::Formatted { to_tokens, .. } => to_tokens.to_view(key, locale_field),
        }
    }
    pub fn var_fmt(&self, key: &Key, locale_field: &Key) -> TokenStream {
        match self {
            Self::AttributeValue => {
                unreachable!("attributes values should be rendered by the component renderer.")
            }
            Self::None => {
                quote!(core::fmt::Display::fmt(#key, __formatter))
            }
            Self::Dummy => {
                quote!({ let _ = #key; core::unimplemented!("Dummy formatter, parsing of a formatter must have failed.") })
            }
            Self::Formatted { to_tokens, .. } => to_tokens.to_fmt(key, locale_field),
        }
    }
    pub fn var_to_impl_display(self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        match self {
            Self::AttributeValue => {
                unreachable!("attributes values should be rendered by the component renderer.")
            }
            Self::None => unreachable!(
                "var_to_display function should not have been called on a variable with no formatter."
            ),
            Self::Dummy => {
                quote!({ let _ = #key; core::unimplemented!("Dummy formatter, parsing of a formatter must have failed.") })
            }
            Self::Formatted { to_tokens, .. } => to_tokens.to_impl_display(key, locale_field),
        }
    }
}

impl Eq for VarBounds {}

impl Ord for VarBounds {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (VarBounds::Dummy, VarBounds::Dummy) => Ordering::Equal,
            (VarBounds::Dummy, VarBounds::None) => Ordering::Less,
            (VarBounds::Dummy, VarBounds::Formatted { .. }) => Ordering::Less,
            (VarBounds::None, VarBounds::Dummy) => Ordering::Greater,
            (VarBounds::None, VarBounds::None) => Ordering::Equal,
            (VarBounds::None, VarBounds::Formatted { .. }) => Ordering::Less,
            (VarBounds::Formatted { .. }, VarBounds::Dummy) => Ordering::Greater,
            (VarBounds::Formatted { .. }, VarBounds::None) => Ordering::Greater,
            (
                VarBounds::Formatted {
                    formatter_name: self_name,
                    ..
                },
                VarBounds::Formatted {
                    formatter_name: other_name,
                    ..
                },
            ) => self_name.cmp(other_name),
            (VarBounds::AttributeValue, VarBounds::AttributeValue) => Ordering::Equal,
            (_, VarBounds::AttributeValue) => Ordering::Less,
            (VarBounds::AttributeValue, _) => Ordering::Greater,
        }
    }
}

impl PartialOrd for VarBounds {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Debug for VarBounds {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            VarBounds::Dummy => f.write_str("Dummy"),
            VarBounds::AttributeValue => f.write_str("AttributeValue"),
            VarBounds::None => f.write_str("None"),
            VarBounds::Formatted { formatter_name, .. } => f
                .debug_struct("Formatted")
                .field("formatter_name", &formatter_name)
                .finish(),
        }
    }
}

impl PartialEq for VarBounds {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Display for DuplicateFormatterErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Duplicate formatter: {:?}", self.name)
    }
}

macro_rules! impl_from_arg {
    ($($arg_name:literal => $value:expr,)*) => {
        pub fn from_arg(arg: Option<&str>) -> Result<Self, &'static str> {
            match arg {
                $(
                    Some($arg_name) => Ok($value),
                )*
                Some(_) => Err("unknown argument value"),
                None => Err("missing value for argument"),
            }
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
    ($t: ident, $name: literal, $builder_name: ident, $f:ident  $( ( $($arg_name:ident => $s: ident),* $(,)?) )?, $feature: literal, $err: literal) => {

        pub struct $builder_name {
            $($($arg_name: Option<$s>),*)?
        }

        impl Formatter for $t {
            const DISABLED: Option<&str> = const {
                if cfg!(not(feature = $feature)) {
                    Some($err)
                } else {
                    None
                }
            };

            const NAME: &str = $name;
            type Builder = $builder_name;
            type Field<'a> = &'a str;
            type ParseError = std::borrow::Cow<'static, str>;
            type ToTokens = $f;

            fn builder(&self) -> Self::Builder {
                $builder_name {
                    $($($arg_name: Option::<$s>::None),*)?
                }
            }

            fn parse_arg_name<'a>(&self, arg_name: &'a str) -> Result<Self::Field<'a>, Self::ParseError> {
                match arg_name {
                    $(
                        $(stringify!($arg_name) => Ok(arg_name)),*
                    )?,
                    _ => Err(std::borrow::Cow::Borrowed("unknown argument name")),
                }
            }

            #[allow(unused_variables)]
            fn parse_arg(&self, builder: &mut Self::Builder, field: Self::Field<'_>, arg: Option<&str>) -> Result<(), Self::ParseError> {
                match field {
                    $(
                        $(stringify!($arg_name) => {
                            let value = $s::from_arg(arg)?;
                            if builder.$arg_name.replace(value).is_some() {
                                Err(std::borrow::Cow::Borrowed("duplicate argument"))
                            } else {
                                Ok(())
                            }
                        }),*
                    )?
                    _ => unreachable!()
                }
            }

            fn build(&self, builder: Self::Builder) -> Result<Self::ToTokens, Self::ParseError> {
                let $builder_name {
                    $($($arg_name),*)?
                } = builder;
                Ok($f $( ( $( $arg_name.unwrap_or_default() ),*  ) )?)
            }
        }
    };
}

pub(crate) use impl_formatter;
pub(crate) use impl_from_arg;
pub(crate) use impl_to_tokens;
