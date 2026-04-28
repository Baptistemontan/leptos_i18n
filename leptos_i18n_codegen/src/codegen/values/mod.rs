use leptos_i18n_parser::{
    extraction::{Locales, Values},
    utils::{Key, KeyPath},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

mod components;
mod fmt;
mod into_view;

use crate::{
    CodegenOptions,
    codegen::{builders::infos::BuildersInfos, locales::strings_accessor_method_name},
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
    let formatter_ident = format_ident!("__formatter");
    let data_ident = format_ident!("__data");

    let defaults = values.defaults.compute();
    let non_defaulted_locales: Vec<_> = locales
        .locales
        .iter()
        .filter_map(|l| {
            let value = values.values.get(&l.name.key)?;
            Some((l, value))
        })
        .collect();

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

    let get_data_match_arms = non_defaulted_locales.iter().map(|(l, _)| {
        let loc = &*l.name.key.ident;
        let defaults = defaults.get(&l.name.key).map(|defaulted_locales| {
            defaulted_locales
                .iter()
                .map(|key| quote!(| #enum_ident::#key))
                .collect::<TokenStream>()
        });
        if cfg!(all(feature = "dynamic_load")) {
            let accessor_name = strings_accessor_method_name(&l.name);
            quote! {
                #enum_ident::#loc #defaults => super::#keys_ident::#accessor_name() #maybe_await
            }
        } else {
            quote! {
                #enum_ident::#loc #defaults => ()
            }
        }
    });

    let args_clone_impl = if builder_infos.fields.is_empty() {
        quote!(*self)
    } else {
        quote! {
            Self(core::clone::Clone::clone(&self.0), core::marker::PhantomData)
        }
    };

    let empty_marker = if builder_infos.fields.is_empty() {
        let match_arms = into_view::gen_const_values_match_arms(
            &non_defaulted_locales,
            &defaults,
            enum_ident,
            keys_ident,
        );
        quote! {
            impl __l_i18n_crate::keys::ConstArgsMarker for ArgsBuilder {
                type Args = Args<__l_i18n_crate::keys::NoArgs>;
                type Builded = BuildedArgs;
                type ConstBuilder = Builder<__l_i18n_crate::keys::NoArgs>;
            }

            impl __l_i18n_crate::keys::ConstArgs for Args<__l_i18n_crate::keys::NoArgs> {
                const THIS: Self = Args(BuildedArgs::__const_new(), core::marker::PhantomData);

                fn value(id: Id, locale: #enum_ident) -> __l_i18n_crate::keys::Literal {
                    Self::__const_value(Self::THIS, id, locale)
                }
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

    let render_body = into_view::gen_render_body(
        &non_defaulted_locales,
        &defaults,
        enum_ident,
        keys_ident,
        &locale_field,
    );

    let fmt_body = fmt::gen_fmt_body(
        &non_defaulted_locales,
        &defaults,
        enum_ident,
        keys_ident,
        &locale_field,
        &formatter_ident,
        &data_ident,
    );

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
                    Builder::__new()
                }
            }

            impl __l_i18n_crate::keys::DowngradableArgBuilder for ArgsBuilder {
                type Downgraded = __builders::#builder_name::ArgsBuilder;
                const ID: __builders::#builder_name::Id = __builders::#builder_name::Id::#variant_ident;
            }

            pub struct Args<__Marker, #generics>(BuildedArgs<#generics>, core::marker::PhantomData<__Marker>);

            impl<__Marker, #generics> core::clone::Clone for Args<__Marker, #generics> where BuildedArgs<#generics>: core::clone::Clone {
                fn clone(&self) -> Self {
                    #args_clone_impl
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
                #formatter_ident: &mut core::fmt::Formatter<'_>,
                #locale_field: #enum_ident,
                #data_ident: __l_i18n_crate::keys::DisplayData
            ) -> core::fmt::Result {
                use core::fmt::Write;
                let BuildedArgs #destructure = args;
                #fmt_body
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
