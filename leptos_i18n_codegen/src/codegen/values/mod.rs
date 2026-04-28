use leptos_i18n_parser::{
    extraction::{Literal, Locales, Value, Values},
    parsing::Variable,
    utils::{Key, KeyPath},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

mod components;

use crate::{
    CodegenOptions,
    codegen::{builders::infos::BuildersInfos, locales::strings_accessor_method_name},
    utils::{EitherIter, EitherOfWrapper, fit_in_leptos_tuple},
};

pub fn gen_values_modules_and_accessors(
    key: &Key,
    values: &Values,
    keys_ident: &syn::Ident,
    enum_ident: &syn::Ident,
    locales: &Locales,
    builders: &BuildersInfos,
    path: &KeyPath,
    options: &CodegenOptions,
) -> TokenStream {
    let builder_infos = builders
        .infos
        .get(&values.builder_id)
        .expect("invalid builder id");

    let docs = if options.gen_docs {
        let docs = &builder_infos.docs;
        quote! {
            #[doc = #docs]
        }
    } else {
        quote! {}
    };

    let markers_field = &builders.markers_field;

    let builder_name = &builder_infos.name;
    let variant_ident = builder_infos
        .id_variants
        .get(path)
        .expect("to contain a variant for this key");

    let bounded_generics = builder_infos.bounded_generics();
    let bounded_fmt_generics = builder_infos.bounded_fmt_generics();
    let generics = builder_infos.generics();
    let destructure = &builder_infos.destructure(markers_field);

    let locale_field = format_ident!("__locale");

    let (render_fn_out_type, maybe_async, maybe_await) =
        if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
            (
                quote!(impl __l_i18n_crate::keys::IntoViewFuture),
                quote!(async),
                quote!(.await),
            )
        } else {
            (
                quote!(impl __l_i18n_crate::reexports::leptos::IntoView),
                quote!(),
                quote!(),
            )
        };

    let get_data_match_arms = locales.locales.iter().map(|l| {
        let loc = &*l.name.key.ident;
        if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
            let accessor_name = strings_accessor_method_name(&l.name);
            quote! {
                #enum_ident::#loc => super::#keys_ident::#accessor_name().await
            }
        } else {
            quote! {
                #enum_ident::#loc => ()
            }
        }
    });

    let empty_marker = if builder_infos.fields.is_empty() {
        let match_arms = gen_const_values_match_arms(values, enum_ident, locales, keys_ident);
        quote! {
            impl __l_i18n_crate::keys::ConstArgsMarker for ArgsBuilder {
                type Args = Args<__l_i18n_crate::keys::NoArgs>;
                type Builded = BuildedArgs;
                type ConstBuilder = Builder<__l_i18n_crate::keys::NoArgs>;
                const THIS: Self::Args = Args(BuildedArgs::__const_new(), core::marker::PhantomData);
            }

            impl Args<__l_i18n_crate::keys::NoArgs> {
                #[doc(hidden)]
                pub const fn __const_value(self, _: Id, locale: #enum_ident) -> __l_i18n_crate::keys::Literal {
                    __const_value(locale)
                }
            }

            impl<#generics> __l_i18n_crate::keys::IntoViewArgs for Args<__l_i18n_crate::keys::NoArgs, #generics>
                where Args<__IntoViewMarker, #generics>: __l_i18n_crate::keys::IntoViewArgs,
            {
                fn render(self, id: Self::Id, locale: Self::Locale) -> #render_fn_out_type {
                    <Args<__IntoViewMarker, #generics> as __l_i18n_crate::keys::IntoViewArgs>::render(Args(
                        self.0,
                        core::marker::PhantomData
                    ), id, locale)
                }
            }

            impl<#bounded_fmt_generics> __l_i18n_crate::keys::DisplayArgs for Args<__l_i18n_crate::keys::NoArgs, #generics> {
                type Data = __l_i18n_crate::keys::DisplayData;

                #maybe_async fn get_data(&self, _id: Self::Id, locale: Self::Locale) -> Self::Data {
                    __get_data(locale) #maybe_await
                }

                fn fmt(
                    &self,
                    formatter: &mut core::fmt::Formatter<'_>,
                    _id: Self::Id,
                    locale: Self::Locale,
                    data: &Self::Data,
                ) -> core::fmt::Result {
                    __fmt(&self.0, formatter, locale, *data)
                }
            }

            #[doc(hidden)]
            pub const fn __const_value(locale: #enum_ident) -> __l_i18n_crate::keys::Literal {
                match locale {
                    #(
                        #match_arms,
                    )*
                }
            }
        }
    } else {
        quote! {}
    };

    let render_body = gen_render_body(values, locales, enum_ident, keys_ident, &locale_field);

    quote! {
        #docs
        pub mod #key {
            #[allow(unused)]
            use super::{#enum_ident, __l_i18n_crate, __builders};
            pub use __builders::#builder_name::{Builder, __IntoViewMarker, __DisplayMarker};

            pub type BuildedArgs<#generics> = __builders::#builder_name::BuildedArgs<#generics>;

            pub type Id = ();

            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct ArgsBuilder;

            impl __l_i18n_crate::keys::ArgsBuilder for ArgsBuilder {
                type Id = Id;
                type Builder = Builder<__IntoViewMarker>;
                type Locale = #enum_ident;

                fn new() -> Self::Builder {
                    Builder::new()
                }
            }

            impl __l_i18n_crate::keys::DowngradableArgBuilder for ArgsBuilder {
                type Downgraded = __builders::#builder_name::ArgsBuilder;
                const ID: __builders::#builder_name::Id = __builders::#builder_name::Id::#variant_ident;
            }

            pub struct Args<__Marker, #generics>(BuildedArgs<#generics>, core::marker::PhantomData<__Marker>);

            impl<__Marker, #generics> core::clone::Clone for Args<__Marker, #generics> where BuildedArgs<#generics>: core::clone::Clone {
                fn clone(&self) -> Self {
                    Self(core::clone::Clone::clone(&self.0), core::marker::PhantomData)
                }
            }

            impl<__Marker, #generics> core::marker::Copy for Args<__Marker, #generics> where BuildedArgs<#generics>: core::marker::Copy {}

            impl<#bounded_generics> __l_i18n_crate::keys::IntoViewArgsMarker<BuildedArgs<#generics>> for ArgsBuilder {
                type Args = Args<__IntoViewMarker, #generics>;

                fn into_args(builder: BuildedArgs<#generics>) -> Self::Args {
                    Args(builder, core::marker::PhantomData)
                }
            }

            impl<__Marker, #generics> __l_i18n_crate::keys::Args for Args<__Marker, #generics> {
                type Locale = #enum_ident;
                type Id = Id;
            }

            impl<#bounded_generics> __l_i18n_crate::keys::IntoViewArgs for Args<__IntoViewMarker, #generics> {
                fn render(self, _: (), locale: Self::Locale) -> #render_fn_out_type {
                    let Self(builder, _) = self;
                    __render(builder, locale)
                }
            }

            impl<#bounded_fmt_generics> __l_i18n_crate::keys::DisplayArgs for Args<__DisplayMarker, #generics> {
                type Data = __l_i18n_crate::keys::DisplayData;

                #maybe_async fn get_data(&self, _id: Self::Id, locale: Self::Locale) -> Self::Data {
                    __get_data(locale) #maybe_await
                }

                fn fmt(
                    &self,
                    formatter: &mut core::fmt::Formatter<'_>,
                    _id: Self::Id,
                    locale: Self::Locale,
                    data: &Self::Data,
                ) -> core::fmt::Result {
                    __fmt(&self.0, formatter, locale, *data)
                }
            }

            impl<__Marker, #generics> __l_i18n_crate::keys::DowngradableArgs for Args<__Marker, #generics>
                where
                __builders::#builder_name::Args<__Marker, #generics>: __l_i18n_crate::keys::IntoViewArgs<
                    Locale = #enum_ident,
                    Id =  __builders::#builder_name::Id,
                >,
                Self: __l_i18n_crate::keys::IntoViewArgs<Locale = #enum_ident, Id = Id>
            {
                type Downgraded = __builders::#builder_name::Args<__Marker, #generics>;

                fn downgrade(this: __l_i18n_crate::keys::Key<Self>) -> __l_i18n_crate::keys::Key<Self::Downgraded> {
                    let (Self(args, _), ()) = __l_i18n_crate::keys::Key::into_args_and_id(this);
                    let args = __builders::#builder_name::Args(args, core::marker::PhantomData);
                    __l_i18n_crate::keys::Key::from_args_and_id(args, __builders::#builder_name::Id::#variant_ident)
                }
            }

            impl<__Marker, #generics> __l_i18n_crate::keys::DowngradableDisplayArgs for Args<__Marker, #generics>
                where
                __builders::#builder_name::Args<__Marker, #generics>: __l_i18n_crate::keys::DisplayArgs<
                    Locale = #enum_ident,
                    Id =  __builders::#builder_name::Id,
                    Data = Self::Data
                >,
                Self: __l_i18n_crate::keys::DisplayArgs<Locale = #enum_ident, Id = Id>
            {
                type Downgraded = __builders::#builder_name::Args<__Marker, #generics>;

                fn downgrade(this: __l_i18n_crate::keys::Key<Self>) -> __l_i18n_crate::keys::Key<Self::Downgraded> {
                    let (Self(args, _), ()) = __l_i18n_crate::keys::Key::into_args_and_id(this);
                    let args = __builders::#builder_name::Args(args, core::marker::PhantomData);
                    __l_i18n_crate::keys::Key::from_args_and_id(args, __builders::#builder_name::Id::#variant_ident)
                }
            }

            #[doc(hidden)]
            pub #maybe_async fn __render<#bounded_generics>(args: BuildedArgs<#generics>, #locale_field: #enum_ident) -> impl __l_i18n_crate::reexports::leptos::IntoView {
                let BuildedArgs #destructure = args;
                #render_body
            }

            #[doc(hidden)]
            pub #maybe_async fn __get_data(locale: #enum_ident) -> __l_i18n_crate::keys::DisplayData {
                match locale {
                    #(
                        #get_data_match_arms,
                    )*
                }
            }

            #[doc(hidden)]
            pub fn __fmt<#bounded_fmt_generics>(
                args: &BuildedArgs<#generics>,
                __formatter: &mut core::fmt::Formatter<'_>,
                #locale_field: #enum_ident,
                __data: __l_i18n_crate::keys::DisplayData
            ) -> core::fmt::Result {
                use core::fmt::Write;
                let BuildedArgs #destructure = args;
                todo!()
            }

            #empty_marker
        }

        impl #keys_ident {
            #docs
            pub const fn #key(self) -> __l_i18n_crate::keys::KeyBuilder<#key::ArgsBuilder> {
                __l_i18n_crate::keys::KeyBuilder::from_id(())
            }
        }
    }
}

fn gen_render_body(
    values: &Values,
    locales: &Locales,
    enum_ident: &syn::Ident,
    keys_ident: &syn::Ident,
    locale_field: &syn::Ident,
) -> TokenStream {
    let either_of = EitherOfWrapper::new(locales.locales.len());
    let render_match_arms = locales.locales.iter().enumerate().map(|(i, l)| {
        let loc = &*l.name.key.ident;
        let accessor_name = strings_accessor_method_name(&l.name);
        let translations_ident = if cfg!(feature = "dynamic_load") {
            format_ident!("__i18n_translations__")
        } else {
            format_ident!("__I18N_TRANSLATIONS__")
        };
        let string_count = l.strings.len();
        // TODO: check defaulting
        let value = values.values.get(&l.name.key).expect("a value for this locale");
        let render_value = gen_render_value(value, &translations_ident, string_count, locale_field);
        let render_value = either_of.wrap(i, render_value);
        if cfg!(feature = "dynamic_load") {
            let maybe_await = if cfg!(not(feature = "ssr")) {
                quote!(.await)
            } else {
                quote!()
            };
            quote! {
                #enum_ident::#loc => {
                    let #translations_ident = super::#keys_ident::#accessor_name() #maybe_await;
                    #render_value
                }
            }
        } else {
            quote! {
                #enum_ident::#loc => {
                    const #translations_ident: &[&str; #string_count] = super::#keys_ident::#accessor_name();
                    #render_value
                }
            }
        }
    });

    quote! {
        match #locale_field {
            #(
                #render_match_arms,
            )*
        }
    }
}

pub fn gen_render_value(
    value: &Value,
    translations_ident: &syn::Ident,
    strings_count: usize,
    locale_field: &syn::Ident,
) -> TokenStream {
    let mut tokens = Vec::new();
    flatten_value(
        value,
        &mut tokens,
        translations_ident,
        strings_count,
        locale_field,
    );
    match tokens.as_mut_slice() {
        [] => quote!(""),
        [value] => core::mem::take(value),
        values => fit_in_leptos_tuple(values),
    }
}

fn flatten_value(
    value: &Value,
    tokens: &mut Vec<TokenStream>,
    translations_ident: &syn::Ident,
    strings_count: usize,
    locale_field: &syn::Ident,
) {
    match value {
        Value::Literal(lit) => {
            let ts = gen_render_lit(lit, translations_ident, strings_count);
            tokens.push(ts);
        }
        Value::Variable(Variable { key, bound }) => {
            let ts = bound.var_to_view(&key.ident, locale_field);
            tokens.push(quote! {{
                let #key = core::clone::Clone::clone(&#key);
                #ts
            }});
        }
        Value::Component(component) => {
            let ts = components::render_component(
                component,
                translations_ident,
                strings_count,
                locale_field,
            );
            tokens.push(ts);
        }
        Value::Bloc(values) => {
            for value in values {
                flatten_value(
                    value,
                    tokens,
                    translations_ident,
                    strings_count,
                    locale_field,
                );
            }
        }
        Value::Plurals(plurals) => todo!(),
    }
}

pub fn gen_string_access(
    index: usize,
    translations_ident: &syn::Ident,
    strings_count: usize,
) -> TokenStream {
    let str_access = quote! {
        __l_i18n_crate::__private::index_translations::<#strings_count, #index>(#translations_ident)
    };
    if cfg!(feature = "dynamic_load") {
        str_access
    } else {
        quote! {
            const {
                #str_access
            }
        }
    }
}

pub fn gen_render_lit(
    lit: &Literal,
    translations_ident: &syn::Ident,
    strings_count: usize,
) -> TokenStream {
    match lit {
        Literal::String(index) => {
            let str_access = quote! {
                __l_i18n_crate::__private::index_translations::<#strings_count, #index>(#translations_ident)
            };
            if cfg!(feature = "dynamic_load") {
                str_access
            } else {
                quote! {
                    const {
                        #str_access
                    }
                }
            }
        }
        Literal::Signed(v) => quote!(#v),
        Literal::Unsigned(v) => quote!(#v),
        Literal::Float(v) => quote!(#v),
        Literal::Bool(v) => quote!(#v),
    }
}

fn gen_const_values_match_arms(
    values: &Values,
    enum_ident: &syn::Ident,
    locales: &Locales,
    keys_ident: &syn::Ident,
) -> impl Iterator<Item = TokenStream> {
    //TODO: check defaulting
    values.values.iter().map(move |(locale, value)| {
        let Value::Literal(lit) = value else { todo!() };

        let value = match lit {
            Literal::String(index) => {
                let loc = locales
                    .locales
                    .iter()
                    .find(|l| l.name.key == *locale)
                    .expect("to find the locale for this value");
                if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
                    let value = &*loc.strings[*index];
                    quote!(__l_i18n_crate::keys::Literal::String(#value))
                } else {
                    let string_count = loc.strings.len();
                    let string_accessor = strings_accessor_method_name(&loc.name);
                    let string_accessor = if cfg!(all(feature = "dynamic_load", feature = "ssr")) {
                        format_ident!("{}_no_register", string_accessor)
                    } else {
                        string_accessor
                    };
                    quote!(
                        const {
                            __l_i18n_crate::keys::Literal::String(
                                __l_i18n_crate::__private::index_translations::<#string_count, #index>(
                                    super::#keys_ident::#string_accessor()
                                )
                            )
                        }
                    )
                }
            }
            Literal::Signed(n) => quote!(__l_i18n_crate::keys::Literal::Signed(#n)),
            Literal::Unsigned(n) => quote!(__l_i18n_crate::keys::Literal::Unsigned(#n)),
            Literal::Float(n) => quote!(__l_i18n_crate::keys::Literal::Float(#n)),
            Literal::Bool(n) => quote!(__l_i18n_crate::keys::Literal::Bool(#n)),
        };

        quote! {
            #enum_ident::#locale => #value
        }
    })
}

pub fn captured_keys(value: &Value) -> impl Iterator<Item = &syn::Ident> {
    match value {
        Value::Literal(_) => EitherIter::Iter1(EitherIter::Iter1(core::iter::empty())),
        Value::Variable(variable) => {
            EitherIter::Iter1(EitherIter::Iter2(core::iter::once(&*variable.key.ident)))
        }
        Value::Component(component) => match &component.inner {
            None => EitherIter::Iter1(EitherIter::Iter1(core::iter::empty())),
            Some(inner) => captured_keys(inner),
        },
        Value::Bloc(values) => {
            // we have to collect the iter for bloc and plurals or we get recursive types
            let iter = values.iter().flat_map(captured_keys).collect::<Vec<_>>();
            EitherIter::Iter2(iter.into_iter())
        }
        Value::Plurals(plurals) => {
            let iter = plurals
                .forms
                .iter_forms()
                .flat_map(|(_, inner)| captured_keys(inner))
                .collect::<Vec<_>>();
            EitherIter::Iter2(iter.into_iter())
        }
    }
}
