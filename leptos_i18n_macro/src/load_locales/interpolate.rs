use std::{borrow::Cow, collections::HashSet};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};

use super::{
    key::Key,
    locale::Locale,
    parsed_value::{InterpolateKey, ParsedValue},
};

#[cfg(feature = "debug_interpolations")]
const MAX_KEY_GENERATE_BUILD_DEBUG: usize = 4;

pub struct Interpolation {
    pub ident: syn::Ident,
    pub default_generic_ident: TokenStream,
    pub imp: TokenStream,
}

struct Field<'a> {
    generic: syn::Ident,
    name: String,
    kind: &'a InterpolateKey,
    #[cfg(feature = "debug_interpolations")]
    real_name: &'a str,
}

impl Interpolation {
    pub fn new(
        key: &Key,
        parent_ident: &Ident,
        keys_set: &HashSet<InterpolateKey>,
        locales: &[Locale],
        default_match: &TokenStream,
    ) -> Self {
        let builder_name = format!("{}_builder", key.name);

        let ident = syn::Ident::new(&builder_name, Span::call_site());

        let locale_field = Key::new("_locale").unwrap();

        let fields = keys_set
            .iter()
            .map(|kind| {
                #[cfg(feature = "debug_interpolations")]
                let real_name = kind.get_real_name();
                let key = kind
                    .as_key()
                    .map(|key| key.name.as_str())
                    .unwrap_or("var_count");
                let name = format!("__{}", key);
                let generic = syn::Ident::new(&name, Span::call_site());
                Field {
                    generic,
                    name,
                    kind,
                    #[cfg(feature = "debug_interpolations")]
                    real_name,
                }
            })
            .collect::<Vec<_>>();

        let type_def = Self::create_type(&ident, &fields);
        let translations_types = Self::create_translation_types(key, parent_ident, locales, &ident);
        let builder_impl = Self::builder_impl(&ident, &locale_field, &fields);
        let into_view_impl =
            Self::into_view_impl(key, &ident, &locale_field, &fields, locales, default_match);
        let debug_impl = Self::debug_impl(&builder_name, &ident, &fields);

        #[cfg(feature = "interpolate_display")]
        let display_impl =
            Self::display_impl(key, &ident, &locale_field, &fields, locales, default_match);
        #[cfg(not(feature = "interpolate_display"))]
        let display_impl = quote!();

        let new_impl = Self::new_impl(&ident, &locale_field, &fields);
        let default_generics = fields
            .iter()
            .map(|_| quote!(builders::EmptyInterpolateValue));
        let default_generic_ident = quote!(#ident<#(#default_generics,)*>);

        let imp = quote! {
            #type_def

            #(
                #translations_types
            )*

            #new_impl

            #into_view_impl

            #debug_impl

            #display_impl

            #builder_impl
        };

        Self {
            imp,
            ident,
            default_generic_ident,
        }
    }

    fn new_impl(ident: &syn::Ident, locale_field: &Key, fields: &[Field]) -> TokenStream {
        let generics = fields.iter().map(|_| quote!(EmptyInterpolateValue));

        let fields = fields.iter().map(|field| {
            let field_key = field.kind;
            quote!(#field_key: EmptyInterpolateValue)
        });

        quote! {
            impl #ident<#(#generics,)*> {
                pub const fn new(#locale_field: Locale) -> Self {
                    Self {
                        #(#fields,)*
                        #locale_field
                    }
                }
            }
        }
    }

    fn split_at<T>(slice: &[T], i: usize) -> (&[T], &T, &[T]) {
        let (left, rest) = slice.split_at(i);
        let (mid, right) = rest.split_first().unwrap();
        (left, mid, right)
    }
    fn genenerate_set_fns(ident: &syn::Ident, locale_field: &Key, fields: &[Field]) -> TokenStream {
        (0..fields.len())
            .map(|i| Self::split_at(fields, i))
            .map(|(left_fields, field, right_fields)| {
                Self::create_field_set_fn(ident, locale_field, left_fields, right_fields, field)
            })
            .collect()
    }

    #[cfg(feature = "debug_interpolations")]
    fn generate_build_fns(ident: &syn::Ident, fields: &[Field], locale_field: &Key) -> TokenStream {
        if fields.len() > MAX_KEY_GENERATE_BUILD_DEBUG {
            return Self::generate_success_build_fn(ident, fields);
        }
        let failing_builds = Self::generate_all_failing_build_fn(ident, fields, locale_field);
        let success_build = Self::generate_success_build_fn(ident, fields);

        quote! {
            #(#failing_builds)*

            #success_build
        }
    }

    #[cfg(feature = "interpolate_display")]
    fn generate_string_build(ident: &syn::Ident, fields: &[Field]) -> TokenStream {
        let left_generics_string = fields.iter().filter_map(|field| {
            let ident = &field.generic;
            let generic = field.kind.get_string_generic().ok()?;
            Some(quote!(#ident: #generic))
        });

        let right_generics_string = fields.iter().map(|field| match field.kind {
            InterpolateKey::Count(t) => quote!(#t),
            _ => {
                let ident = &field.generic;
                quote!(#ident)
            }
        });

        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#left_generics_string,)*> #ident<#(#right_generics_string,)*> {
                #[inline]
                pub fn build_display(self) -> Self {
                    self
                }

                pub fn build_string(self) -> std::borrow::Cow<'static, str> {
                    self.to_string().into()
                }
            }
        }
    }

    fn generate_success_build_fn(ident: &syn::Ident, fields: &[Field]) -> TokenStream {
        let left_generics = fields.iter().map(|field| {
            let ident = &field.generic;
            let generic = field.kind.get_generic();
            quote!(#ident: #generic)
        });

        let right_generics = fields.iter().map(|field| {
            let ident = &field.generic;
            quote!(#ident)
        });

        #[cfg(feature = "interpolate_display")]
        let string_build = Self::generate_string_build(ident, fields);

        #[cfg(not(feature = "interpolate_display"))]
        let string_build = quote!();

        quote! {

            #[allow(non_camel_case_types)]
            impl<#(#left_generics,)*> #ident<#(#right_generics,)*> {
                #[inline]
                pub fn build(self) -> Self {
                    self
                }
            }

            #string_build

        }
    }

    fn builder_impl(ident: &syn::Ident, locale_field: &Key, fields: &[Field]) -> TokenStream {
        let set_fns = Self::genenerate_set_fns(ident, locale_field, fields);

        #[cfg(not(feature = "debug_interpolations"))]
        let build_fns = Self::generate_success_build_fn(ident, fields);
        #[cfg(feature = "debug_interpolations")]
        let build_fns = Self::generate_build_fns(ident, fields, locale_field);

        quote! {
            #set_fns
            #build_fns
        }
    }

    fn create_type(ident: &syn::Ident, fields: &[Field]) -> TokenStream {
        let generics = fields.iter().map(|field| &field.generic);
        let fields = fields.iter().map(|field| {
            let key = field.kind;
            let generic = &field.generic;
            quote!(#key: #generic)
        });

        quote! {
            #[allow(non_camel_case_types, non_snake_case)]
            #[derive(Clone, Copy, Hash, PartialEq, Eq)]
            pub struct #ident<#(#generics,)*> {
                _locale: Locale,
                #(#fields,)*
            }
        }
    }

    fn generate_generics<'a, F: 'a, T: Clone + 'a>(
        left_fields: &'a [Field],
        field_generic: Option<T>,
        right_fields: &'a [Field],
        other_field_map_fn: F,
    ) -> impl Iterator<Item = T> + 'a + Clone
    where
        F: FnMut(&'a Field) -> T + Copy,
    {
        left_fields
            .iter()
            .map(other_field_map_fn)
            .chain(field_generic)
            .chain(right_fields.iter().map(other_field_map_fn))
    }

    #[cfg(feature = "debug_interpolations")]
    fn generate_default_constructed(
        ident: &syn::Ident,
        fields: &[Field],
        locale_field: &Key,
    ) -> TokenStream {
        let populated_fields = fields
            .iter()
            .map(|field| {
                let field_ident = field.kind;
                let default = field.kind.get_default();
                quote!(#field_ident: #default)
            })
            .chain(std::iter::once(
                quote!(#locale_field: core::default::Default::default()),
            ));

        quote! {
            #ident {
                #(#populated_fields,)*
            }
        }
    }

    #[cfg(feature = "debug_interpolations")]
    fn generate_all_failing_build_fn<'a>(
        ident: &'a syn::Ident,
        fields: &'a [Field],
        locale_field: &Key,
    ) -> impl Iterator<Item = TokenStream> + 'a {
        let default_constructed = Self::generate_default_constructed(ident, fields, locale_field);
        let output_generics = fields.iter().map(|field| {
            let generic = field.kind.get_generic();
            quote!(impl #generic)
        });
        let output = quote!(#ident<#(#output_generics,)*>);
        let max = 1u64 << fields.len();
        (0..max - 1).map(move |states| {
            let fields_iter = fields.iter().enumerate().map(|(i, field)| {
                let state = (states >> i & 1) == 1;
                (state, field)
            });
            Self::generate_failing_build_fn(ident, &default_constructed, &output, fields_iter)
        })
    }

    #[cfg(feature = "debug_interpolations")]
    fn generate_failing_build_fn<'a, I>(
        self_ident: &syn::Ident,
        default_constructed: &TokenStream,
        output: &TokenStream,
        fields: I,
    ) -> TokenStream
    where
        I: Iterator<Item = (bool, &'a Field<'a>)> + Clone,
    {
        let right_generics = fields.clone().map(|(set, field)| {
            if set {
                quote::ToTokens::to_token_stream(&field.generic)
            } else {
                quote!(EmptyInterpolateValue)
            }
        });
        let left_generics = fields.clone().filter(|(set, _)| *set).map(|(_, field)| {
            let ident = &field.generic;
            let generic_bound = field.kind.get_generic();
            quote!(#ident: #generic_bound)
        });

        let missing_fields = fields
            .filter_map(|(set, field)| (!set).then_some(field))
            .map(|field| match field.kind {
                InterpolateKey::Count(_) | InterpolateKey::Variable(_) => field.real_name.into(),
                InterpolateKey::Component(_) => format!("<{}>", field.real_name).into(),
            })
            .collect::<Vec<Cow<_>>>();

        let error_message = format!("\nMissing interpolations keys: {:?}", missing_fields);

        quote! {
            impl<#(#left_generics,)*> #self_ident<#(#right_generics,)*> {
                #[deprecated(note = #error_message)]
                pub fn build(self) -> #output {
                    panic!(#error_message);
                    #[allow(unreachable_code)]
                    #default_constructed
                }
            }
        }
    }

    fn create_field_set_fn(
        ident: &syn::Ident,
        locale_field: &Key,
        left_fields: &[Field],
        right_fields: &[Field],
        field: &Field,
    ) -> TokenStream {
        let quoted_gen = |field: &Field| {
            let generic = &field.generic;
            quote!(#generic)
        };
        let output_field_generic = field.kind.get_generic();
        let output_generics = Self::generate_generics(
            left_fields,
            Some(quote!(impl #output_field_generic)),
            right_fields,
            quoted_gen,
        );

        #[cfg(feature = "interpolate_display")]
        let output_field_generic_string = match field.kind.get_string_generic() {
            Ok(bounds) => quote!(impl #bounds),
            Err(t) => quote!(#t),
        };
        #[cfg(feature = "interpolate_display")]
        let output_generics_string = Self::generate_generics(
            left_fields,
            Some(output_field_generic_string.clone()),
            right_fields,
            quoted_gen,
        );
        let other_fields = Self::generate_generics(left_fields, None, right_fields, |field| {
            if let Some(key) = field.kind.as_key() {
                quote!(#key)
            } else {
                quote!(var_count)
            }
        })
        .chain(Some(quote!(#locale_field)));

        let kind = field.kind;

        let destructure = {
            let other_fields = other_fields.clone();
            quote!(let Self { #(#other_fields,)* .. } = self;)
        };
        let restructure = quote!(#ident { #(#other_fields,)* #kind });

        let fns = match kind {
            InterpolateKey::Variable(key) => (
                quote! {
                    #[inline]
                    pub fn #key<__T>(self, #key: __T) -> #ident<#(#output_generics,)*>
                        where __T: leptos::IntoView + core::clone::Clone + 'static
                    {
                        #destructure
                        #restructure
                    }
                },
                #[cfg(feature = "interpolate_display")]
                {
                    let string_key = format_ident!("{}_string", key.ident);
                    quote! {
                        #[inline]
                        pub fn #string_key(self, #key: #output_field_generic_string) -> #ident<#(#output_generics_string,)*>
                        {
                            #destructure
                            #restructure
                        }
                    }
                },
            ),
            InterpolateKey::Component(key) => (
                quote! {
                    #[inline]
                    pub fn #key<__O, __T>(self, #key: __T) -> #ident<#(#output_generics,)*>
                    where
                        __O: leptos::IntoView,
                        __T: Fn(leptos::ChildrenFn) -> __O + core::clone::Clone + 'static
                    {
                        #destructure
                        let #key = move |children| leptos::IntoView::into_view(#key(children));
                        #restructure
                    }
                },
                #[cfg(feature = "interpolate_display")]
                {
                    let string_key = format_ident!("{}_string", key.ident);
                    quote! {
                        #[inline]
                        pub fn #string_key(self, #key: #output_field_generic_string) -> #ident<#(#output_generics_string,)*>
                        {
                            #destructure
                            #restructure
                        }
                    }
                },
            ),
            InterpolateKey::Count(plural_type) => (
                quote! {
                    #[inline]
                    pub fn var_count<__T>(self, var_count: __T) -> #ident<#(#output_generics,)*>
                        where __T: Fn() -> #plural_type + core::clone::Clone + 'static,
                    {
                        #destructure
                        #restructure
                    }
                },
                #[cfg(feature = "interpolate_display")]
                quote! {
                    #[inline]
                    pub fn var_count_string(self, var_count: #plural_type) -> #ident<#(#output_generics_string,)*>
                    {
                        #destructure
                        #restructure
                    }
                },
            ),
        };

        #[cfg(feature = "interpolate_display")]
        let (set_function, set_str_function) = fns;
        #[cfg(not(feature = "interpolate_display"))]
        let (set_function, set_str_function) = (fns.0, quote!());

        if cfg!(feature = "debug_interpolations") {
            let left_generics_empty =
                Self::generate_generics(left_fields, None, right_fields, |field| &field.generic);
            let left_generics_already_set = Self::generate_generics(
                left_fields,
                Some({
                    let field_gen = &field.generic;
                    quote!(#field_gen: #output_field_generic)
                }),
                right_fields,
                quoted_gen,
            );
            let right_generics_empty = Self::generate_generics(
                left_fields,
                Some(quote!(EmptyInterpolateValue)),
                right_fields,
                quoted_gen,
            );
            let right_generics_already_set =
                Self::generate_generics(left_fields, Some(&field.generic), right_fields, |field| {
                    &field.generic
                });

            let compile_warning = match field.kind {
                InterpolateKey::Count(_) => "variable `count` is already set".to_string(),
                InterpolateKey::Variable(_) => format!("variable `{}` is already set", field.name),
                InterpolateKey::Component(_) => {
                    format!("component `{}` is already set", field.name)
                }
            };

            quote! {
                #[allow(non_camel_case_types)]
                impl<#(#left_generics_empty,)*> #ident<#(#right_generics_empty,)*> {
                    #set_function
                }
                #[allow(non_camel_case_types)]
                impl<#(#left_generics_already_set,)*> #ident<#(#right_generics_already_set,)*> {
                    #[deprecated(note = #compile_warning)]
                    #set_function
                }
            }
        } else {
            let left_generics =
                Self::generate_generics(left_fields, Some(&field.generic), right_fields, |field| {
                    &field.generic
                });
            let right_generics = left_generics.clone();

            quote! {
                #[allow(non_camel_case_types)]
                impl<#(#left_generics,)*> #ident<#(#right_generics,)*> {
                    #set_function

                    #set_str_function
                }
            }
        }
    }

    fn debug_impl(builder_name: &str, ident: &syn::Ident, fields: &[Field]) -> TokenStream {
        let left_generics = fields.iter().map(|field| &field.generic);

        let right_generics = left_generics.clone();

        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#left_generics,)*> core::fmt::Debug for #ident<#(#right_generics,)*> {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.debug_struct(#builder_name).finish()
                }
            }
        }
    }

    #[cfg(feature = "interpolate_display")]
    fn display_impl(
        key: &Key,
        ident: &syn::Ident,
        locale_field: &Key,
        fields: &[Field],
        locales: &[Locale],
        default_match: &TokenStream,
    ) -> TokenStream {
        let left_generics = fields.iter().filter_map(|field| {
            let ident = &field.generic;
            let generic = field.kind.get_string_generic().ok()?;
            Some(quote!(#ident: #generic))
        });

        let right_generics = fields.iter().map(|field| match field.kind {
            InterpolateKey::Count(t) => quote!(#t),
            _ => {
                let ident = &field.generic;
                quote!(#ident)
            }
        });

        let fields_key = fields.iter().map(|f| f.kind);

        let destructure = quote!(let Self { #(#fields_key,)* #locale_field } = self;);

        let locales_impls = Self::create_locale_string_impl(key, locales, default_match, ident);

        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#left_generics,)*> core::fmt::Display for #ident<#(#right_generics,)*> {
                fn fmt(&self, __formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    #destructure
                    match #locale_field {
                        #(
                            #locales_impls,
                        )*
                    }
                }
            }
        }
    }

    fn into_view_impl(
        key: &Key,
        ident: &syn::Ident,
        locale_field: &Key,
        fields: &[Field],
        locales: &[Locale],
        default_match: &TokenStream,
    ) -> TokenStream {
        let left_generics = fields.iter().map(|field| {
            let ident = &field.generic;
            let generic = field.kind.get_generic();
            quote!(#ident: #generic)
        });

        let right_generics = fields.iter().map(|field| {
            let ident = &field.generic;
            quote!(#ident)
        });

        let fields_key = fields.iter().map(|f| f.kind);

        let destructure = quote!(let Self { #(#fields_key,)* #locale_field } = self;);

        let locales_impls = Self::create_locale_impl(key, locales, default_match, ident);

        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#left_generics,)*> leptos::IntoView for #ident<#(#right_generics,)*> {
                fn into_view(self) -> leptos::View {
                    #destructure
                    match #locale_field {
                        #(
                            #locales_impls,
                        )*
                    }
                }
            }
        }
    }

    fn create_translation_types<'a>(
        key: &'a Key,
        parent_ident: &'a Ident,
        locales: &'a [Locale],
        ident: &'a syn::Ident,
    ) -> impl Iterator<Item = TokenStream> + 'a {
        locales.iter().filter_map(move |locale| {
            let value = locale.keys.get(key)?;
            if matches!(value, ParsedValue::Default) {
                return None;
            }
            let strings = value.get_strings();
            let top_locale_name = &*locale.top_locale_name;
            let translation_ident = format_ident!("{}_{}", ident, top_locale_name.ident);
            let parent_ident = format_ident!("{}_{}", parent_ident, top_locale_name.ident);
            let lengths = strings.iter().map(|s| s.len());
            let lengths_clone = lengths.clone();
            let ts = quote! {
                #[allow(non_camel_case_types, non_snake_case)]
                pub struct #translation_ident(
                    #(
                        leptos_i18n::__private::SizedString<#lengths>,
                    )*
                );

                impl #translation_ident {
                    pub fn get() -> &'static Self {
                        &super::#parent_ident::get().#key
                    }

                    pub const fn new() -> Self {
                        #translation_ident(
                            #(
                                leptos_i18n::__private::SizedString::new(#strings),
                            )*
                        )
                    }
                }

                impl leptos_i18n::__private::ParseTranslation for #translation_ident {
                    fn parse(buff: &mut &str) -> Option<Self> {
                        Some(#translation_ident(
                            #(
                                <leptos_i18n::__private::SizedString<#lengths_clone> as leptos_i18n::__private::ParseTranslation>::parse(buff)?,
                            )*
                        ))
                    }
                }
            };
            Some(ts)
        })
    }

    fn create_locale_impl<'a>(
        key: &'a Key,
        locales: &'a [Locale],
        default_match: &TokenStream,
        ident: &'a syn::Ident,
    ) -> impl Iterator<Item = TokenStream> + 'a {
        let mut default_match = default_match.clone();
        locales
            .iter()
            .enumerate()
            .rev()
            .filter_map(move |(i, locale)| {
                let locale_key = &locale.top_locale_name;

                let value = match locale.keys.get(key) {
                    None | Some(ParsedValue::Default) => {
                        default_match.extend(quote!(| Locale::#locale_key));
                        return None;
                    }
                    Some(value) => value,
                };

                let translations_ident = format_ident!("{}_{}", ident, locale_key.ident);

                let matc = match i == 0 {
                    true => Cow::Borrowed(&default_match),
                    false => Cow::Owned(quote!(Locale::#locale_key)),
                };
                let ts = quote!(#matc => {
                    let __translations = #translations_ident::get();
                    #value
                });
                Some(ts)
            })
    }

    #[cfg(feature = "interpolate_display")]
    fn create_locale_string_impl<'a>(
        key: &'a Key,
        locales: &'a [Locale],
        default_match: &TokenStream,
        ident: &'a syn::Ident,
    ) -> impl Iterator<Item = TokenStream> + 'a {
        let mut default_match = default_match.clone();
        locales
            .iter()
            .enumerate()
            .rev()
            .filter_map(move |(i, locale)| {
                let locale_key = &locale.top_locale_name;

                let value = match locale.keys.get(key) {
                    None | Some(ParsedValue::Default) => {
                        default_match.extend(quote!(| Locale::#locale_key));
                        return None;
                    }
                    Some(value) => value,
                };

                let translations_ident = format_ident!("{}_{}", ident, locale_key.ident);

                let value = value.as_string_impl();

                let matc = match i == 0 {
                    true => Cow::Borrowed(&default_match),
                    false => Cow::Owned(quote!(Locale::#locale_key)),
                };
                let ts = quote!(#matc => {
                    let __translations = #translations_ident::get();
                    #value
                });
                Some(ts)
            })
    }
}

pub fn create_empty_type() -> TokenStream {
    quote! {
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub struct EmptyInterpolateValue;
    }
}
