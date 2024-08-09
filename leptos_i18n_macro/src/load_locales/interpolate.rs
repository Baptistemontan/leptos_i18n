use std::collections::HashSet;
use std::rc::Rc;

use proc_macro2::{Span, TokenStream};
use quote::format_ident;
use quote::quote;

use super::parsed_value::Formatter;
use super::parsed_value::InterpolationKeys;
use super::plural::PluralType;
use super::{
    key::{Key, KeyPath},
    locale::Locale,
    parsed_value::ParsedValue,
};

enum EitherIter<A, B> {
    Iter1(A),
    Iter2(B),
}

impl<T, A: Iterator<Item = T>, B: Iterator<Item = T>> Iterator for EitherIter<A, B> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EitherIter::Iter1(iter) => iter.next(),
            EitherIter::Iter2(iter) => iter.next(),
        }
    }
}

pub struct Interpolation {
    pub ident: syn::Ident,
    pub imp: TokenStream,
}

enum VarOrComp<'a> {
    Var {
        formatters: &'a HashSet<Formatter>,
        plural: Option<PluralType>,
    },
    Comp {
        into_view: syn::Ident,
    },
}

struct Field<'a> {
    key: Rc<Key>,
    generic: syn::Ident,
    var_or_comp: VarOrComp<'a>,
}

impl<'a> Field<'a> {
    fn get_var_generics(
        generic: &syn::Ident,
        formatters: &HashSet<Formatter>,
        plural: Option<PluralType>,
    ) -> TokenStream {
        //! FIXME
        let _ = formatters;
        match plural {
            Some(plural_type) => {
                quote!(#generic: l_i18n_crate::__private::InterpolateCount<#plural_type>)
            }
            None => quote!(#generic: ::std::fmt::Display),
        }
    }

    fn get_string_var_generics(
        generic: &syn::Ident,
        formatters: &HashSet<Formatter>,
        plural: Option<PluralType>,
    ) -> TokenStream {
        //! FIXME
        let _ = formatters;
        match plural {
            Some(plural_type) => {
                quote!(#generic: l_i18n_crate::__private::InterpolateCount<#plural_type>)
            }
            None => quote!(#generic: l_i18n_crate::__private::InterpolateVar),
        }
    }

    pub fn as_bounded_generic(&self) -> impl Iterator<Item = TokenStream> {
        let generic = &self.generic;
        match &self.var_or_comp {
            VarOrComp::Var { formatters, plural } => {
                let ts = Self::get_var_generics(generic, formatters, *plural);
                EitherIter::Iter1(std::iter::once(ts))
            }
            VarOrComp::Comp { into_view } => {
                let ts = [
                    quote!(#into_view: l_i18n_crate::__private::leptos::IntoView),
                    quote!(#generic: l_i18n_crate::__private::InterpolateComp<#into_view>),
                ];
                EitherIter::Iter2(ts.into_iter())
            }
        }
    }

    pub fn as_string_bounded_generic(&self) -> TokenStream {
        let generic = &self.generic;
        match &self.var_or_comp {
            VarOrComp::Var { formatters, plural } => {
                Self::get_string_var_generics(generic, formatters, *plural)
            }
            VarOrComp::Comp { .. } => {
                quote!(#generic: l_i18n_crate::display::DisplayComponent)
            }
        }
    }

    pub fn as_generic(&self) -> &syn::Ident {
        &self.generic
    }

    pub fn as_struct_field(&self) -> TokenStream {
        let Field { key, generic, .. } = self;
        quote!(#key: #generic)
    }

    pub fn as_into_view_generic(&self) -> Option<&syn::Ident> {
        match &self.var_or_comp {
            VarOrComp::Var { .. } => None,
            VarOrComp::Comp { into_view } => Some(into_view),
        }
    }
}

impl Interpolation {
    fn make_fields(keys: &InterpolationKeys) -> Vec<Field> {
        let vars = keys.iter_vars().map(|(key, infos)| {
            let var_or_comp = VarOrComp::Var {
                formatters: &infos.formatters,
                plural: infos.plural_count,
            };
            let generic = format_ident!("__{}__", key.ident);
            Field {
                key,
                var_or_comp,
                generic,
            }
        });

        let comps = keys.iter_comps().map(|key| {
            let into_view = format_ident!("__into_view_{}__", key.ident);
            let var_or_comp = VarOrComp::Comp { into_view };
            let generic = format_ident!("__{}__", key.ident);
            Field {
                key,
                var_or_comp,
                generic,
            }
        });

        let mut fields: Vec<_> = vars.chain(comps).collect();

        fields.sort_by(|a, b| a.key.cmp(&b.key));

        fields
    }

    pub fn new(
        key: &Key,
        enum_ident: &syn::Ident,
        keys: &InterpolationKeys,
        locales: &[Locale],
        default_match: &TokenStream,
        key_path: &KeyPath,
    ) -> Self {
        let builder_name = format!("{}_builder", key.name);

        let ident = syn::Ident::new(&builder_name, Span::call_site());

        let dummy_ident = format_ident!("{}_dummy", ident);

        let locale_field = Key::new("_locale").unwrap();
        let into_view_field = Key::new("_into_views_marker").unwrap();

        let typed_builder_name = format_ident!("{}Builder", ident);

        let fields = Self::make_fields(keys);

        let type_def = Self::create_types(
            &ident,
            &dummy_ident,
            enum_ident,
            &locale_field,
            &into_view_field,
            &fields,
        );

        let dummy_impl = Self::dummy_impl(
            &ident,
            &dummy_ident,
            enum_ident,
            &typed_builder_name,
            &locale_field,
            &into_view_field,
            &fields,
        );

        let into_view_impl = Self::into_view_impl(
            key,
            &ident,
            enum_ident,
            &locale_field,
            &fields,
            locales,
            default_match,
            key_path,
        );

        let debug_impl = Self::debug_impl(&builder_name, &ident, &fields);

        let (display_impl, builder_display) = if cfg!(feature = "interpolate_display") {
            let display_impl = Self::display_impl(
                key,
                &ident,
                enum_ident,
                &locale_field,
                &fields,
                locales,
                default_match,
            );
            let builder_display =
                Self::builder_string_build_fns(enum_ident, &typed_builder_name, &fields);
            (display_impl, builder_display)
        } else {
            (quote!(), quote!())
        };

        let imp = quote! {
            #type_def

            #into_view_impl

            #debug_impl

            #display_impl

            #builder_display

            #dummy_impl
        };

        Self {
            imp,
            ident: dummy_ident,
        }
    }

    fn builder_string_build_fns(
        enum_ident: &syn::Ident,
        typed_builder_name: &syn::Ident,
        fields: &[Field],
    ) -> TokenStream {
        let left_generics = fields.iter().map(Field::as_string_bounded_generic);

        let right_generics = fields.iter().map(Field::as_generic);
        let marker = right_generics.clone();

        let into_views = fields
            .iter()
            .filter_map(Field::as_into_view_generic)
            .map(|_| quote!(()));

        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#left_generics,)*> #typed_builder_name<#(#right_generics,)* ((#enum_ident,), (core::marker::PhantomData<(#(#into_views,)*)>,), #((#marker,),)*)> {
                #[inline]
                pub fn build_display(self) -> impl std::fmt::Display {
                    self.build()
                }

                #[inline]
                pub fn build_string(self) -> std::borrow::Cow<'static, str> {
                    std::borrow::Cow::Owned(self.build().to_string())
                }
            }
        }
    }

    fn display_builder_fn(
        ident: &syn::Ident,
        enum_ident: &syn::Ident,
        typed_builder_name: &syn::Ident,
        locale_field: &Key,
        into_view_field: &Key,
        fields: &[Field],
    ) -> TokenStream {
        let left_generics = fields.iter().map(Field::as_string_bounded_generic);
        let right_generics = fields.iter().map(Field::as_generic);
        let builder_marker = fields.iter().map(|_| quote!(()));
        let into_views = fields
            .iter()
            .filter_map(Field::as_into_view_generic)
            .map(|_| quote!(()));

        quote! {
            pub fn display_builder<#(#left_generics,)*>(self) -> #typed_builder_name<#(#right_generics,)* ((#enum_ident,), (core::marker::PhantomData<(#(#into_views,)*)>,), #(#builder_marker,)*)> {
                #ident::builder().#locale_field(self.#locale_field).#into_view_field(core::marker::PhantomData)
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn dummy_impl(
        ident: &syn::Ident,
        dummy_ident: &syn::Ident,
        enum_ident: &syn::Ident,
        typed_builder_name: &syn::Ident,
        locale_field: &Key,
        into_view_field: &Key,
        fields: &[Field],
    ) -> TokenStream {
        let left_generics = fields.iter().flat_map(Field::as_bounded_generic);

        let right_generics = fields.iter().map(Field::as_generic);

        let empty_builder_marker = fields.iter().map(|_| quote!(()));

        let display_builder_fn = if cfg!(feature = "interpolate_display") {
            Self::display_builder_fn(
                ident,
                enum_ident,
                typed_builder_name,
                locale_field,
                into_view_field,
                fields,
            )
        } else {
            quote!()
        };

        let into_views = fields.iter().filter_map(Field::as_into_view_generic);

        quote! {
            impl #dummy_ident {
                pub const fn new(#locale_field: #enum_ident) -> Self {
                    Self {
                        #locale_field
                    }
                }

                pub fn builder<#(#left_generics,)*>(self) -> #typed_builder_name<#(#right_generics,)* ((#enum_ident,), (core::marker::PhantomData<(#(#into_views,)*)>,), #(#empty_builder_marker,)*)> {
                    #ident::builder().#locale_field(self.#locale_field).#into_view_field(core::marker::PhantomData)
                }

                #display_builder_fn
            }
        }
    }

    fn create_types(
        ident: &syn::Ident,
        dummy_ident: &syn::Ident,
        enum_ident: &syn::Ident,
        locale_field: &Key,
        into_view_field: &Key,
        fields: &[Field],
    ) -> TokenStream {
        let generics = fields.iter().map(Field::as_generic);

        let into_views = fields.iter().filter_map(Field::as_into_view_generic);

        let fields = fields.iter().map(Field::as_struct_field);

        let into_views_marker = quote! {
            #into_view_field: core::marker::PhantomData<(#(#into_views,)*)>
        };

        quote! {
            #[allow(non_camel_case_types, non_snake_case)]
            #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
            pub struct #dummy_ident {
                #locale_field: #enum_ident
            }

            #[allow(non_camel_case_types, non_snake_case)]
            #[derive(l_i18n_crate::__private::typed_builder::TypedBuilder)]
            #[builder(crate_module_path = l_i18n_crate::typed_builder)]
            pub struct #ident<#(#generics,)*> {
                #locale_field: #enum_ident,
                #into_views_marker,
                #(#fields,)*
            }
        }
    }

    fn debug_impl(builder_name: &str, ident: &syn::Ident, fields: &[Field]) -> TokenStream {
        let left_generics = fields.iter().map(Field::as_string_bounded_generic);

        let right_generics = fields.iter().map(Field::as_generic);

        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#left_generics,)*> core::fmt::Debug for #ident<#(#right_generics,)*> {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.debug_struct(#builder_name).finish()
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn display_impl(
        key: &Key,
        ident: &syn::Ident,
        enum_ident: &syn::Ident,
        locale_field: &Key,
        fields: &[Field],
        locales: &[Locale],
        default_match: &TokenStream,
    ) -> TokenStream {
        let left_generics = fields.iter().map(Field::as_string_bounded_generic);
        let right_generics = fields.iter().map(Field::as_generic);

        let fields_key = fields.iter().map(|f| &*f.key);

        let destructure = quote!(let Self { #(#fields_key,)* #locale_field, .. } = self;);

        let locales_impls =
            Self::create_locale_string_impl(key, enum_ident, locales, default_match);

        quote! {
            #[allow(non_camel_case_types)]
            impl<#(#left_generics,)*> ::core::fmt::Display for #ident<#(#right_generics,)*> {
                fn fmt(&self, __formatter: &mut ::core::fmt::Formatter<'_>) -> core::fmt::Result {
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

    #[allow(clippy::too_many_arguments)]
    fn into_view_impl(
        key: &Key,
        ident: &syn::Ident,
        enum_ident: &syn::Ident,
        locale_field: &Key,
        fields: &[Field],
        locales: &[Locale],
        default_match: &TokenStream,
        key_path: &KeyPath,
    ) -> TokenStream {
        let left_generics = fields.iter().flat_map(Field::as_bounded_generic);

        let right_generics = fields.iter().map(Field::as_generic);

        if cfg!(feature = "show_keys_only") {
            let key = key_path.to_string_with_key(key);
            return quote! {
                #[allow(non_camel_case_types)]
                impl<#(#left_generics,)*> leptos::IntoView for #ident<#(#right_generics,)*> {
                    fn into_view(self) -> leptos::View {
                        let _ = self;
                        leptos::IntoView::into_view(#key)
                    }
                }
            };
        }

        let fields_key = fields.iter().map(|f| &*f.key);

        let destructure = quote!(let Self { #(#fields_key,)* #locale_field, .. } = self;);

        let locales_impls = Self::create_locale_impl(key, enum_ident, locales, default_match);

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

    fn create_locale_impl<'a>(
        key: &'a Key,
        enum_ident: &'a syn::Ident,
        locales: &'a [Locale],
        default_match: &TokenStream,
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
                        default_match.extend(quote!(| #enum_ident::#locale_key));
                        return None;
                    }
                    Some(value) => value,
                };

                let ts = match i == 0 {
                    true => quote!(#default_match => { #value }),
                    false => quote!(#enum_ident::#locale_key => { #value }),
                };
                Some(ts)
            })
    }

    fn create_locale_string_impl<'a>(
        key: &'a Key,
        enum_ident: &'a syn::Ident,
        locales: &'a [Locale],
        default_match: &TokenStream,
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
                        default_match.extend(quote!(| #enum_ident::#locale_key));
                        return None;
                    }
                    Some(value) => value,
                };

                let value = value.as_string_impl();

                let ts = match i == 0 {
                    true => quote!(#default_match => { #value }),
                    false => quote!(#enum_ident::#locale_key => { #value }),
                };
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
