use leptos_i18n_parser::{
    extraction::Builders,
    utils::{Key, KeyPath},
};
use proc_macro2::TokenStream;
use quote::quote;

mod builder;
pub mod infos;

use infos::{BuilderInfos, BuildersInfos};

use crate::utils::EitherOfWrapper;

pub fn gen_builder_module(
    builders: &Builders,
    enum_ident: &syn::Ident,
    markers_field: syn::Ident,
    gen_docs: bool,
) -> (TokenStream, BuildersInfos) {
    let infos = BuildersInfos::new(builders, markers_field, gen_docs);

    let ts = gen_module(&infos, enum_ident);

    (ts, infos)
}

fn gen_module(infos: &BuildersInfos, enum_ident: &syn::Ident) -> TokenStream {
    let markers_field = &infos.markers_field;
    let inner_modules = infos
        .infos
        .values()
        .map(|infos| gen_inner_module(infos, enum_ident, markers_field));

    quote! {
        #[doc(hidden)]
        pub mod __builders {
            #[allow(unused)]
            use super::{#enum_ident, __l_i18n_crate};
            #(#inner_modules)*
        }
    }
}

fn iter_path_keys(path: &KeyPath) -> impl Iterator<Item = &Key> {
    let iter = path.namespace.as_ref().into_iter();
    iter.chain(path.path.iter())
}

fn gen_inner_module(
    infos: &BuilderInfos,
    enum_ident: &syn::Ident,
    markers_field: &syn::Ident,
) -> TokenStream {
    let mod_key = &*infos.name.ident;
    let variants = infos.id_variants.values();
    let bounded_generics = infos.bounded_generics();
    let bounded_fmt_generics = infos.bounded_fmt_generics();
    let generics = infos.generics();
    let struct_fields = infos.struct_fields(markers_field);

    let either_of = EitherOfWrapper::new(infos.id_variants.len());

    let render_match_arms = infos
        .id_variants
        .iter()
        .enumerate()
        .map(|(i, (path, variant))| {
            let keys = iter_path_keys(path);
            let ts = quote! {
                super::super::keys::#(#keys ::)* __render(args, locale)
            };
            let wrapped = either_of.wrap(i, ts);
            quote! {
                Id::#variant => #wrapped
            }
        });

    let fmt_match_arms = infos.id_variants.iter().map(|(path, variant)| {
        let keys = iter_path_keys(path);
        quote! {
            Id::#variant => super::super::keys::#(#keys ::)* __fmt(args, formatter, locale, data)
        }
    });

    let (maybe_async, maybe_await) = if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
        (quote!(async), quote!(.await))
    } else {
        (quote!(), quote!())
    };

    let get_data_match_arms = infos.id_variants.iter().map(|(path, variant)| {
        let keys = iter_path_keys(path);
        quote! {
            Id::#variant => super::super::keys::#(#keys ::)* __get_data(locale) #maybe_await
        }
    });

    let render_fn_out_type = if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
        quote!(impl __l_i18n_crate::keys::IntoViewFuture)
    } else {
        quote!(impl __l_i18n_crate::reexports::leptos::IntoView)
    };

    let empty_marker = if infos.fields.is_empty() {
        let const_value_match_arms = infos.id_variants.iter().map(|(path, variant)| {
            let keys = iter_path_keys(path);
            quote! {
                Id::#variant => super::super::keys::#(#keys ::)* __const_value(locale)
            }
        });

        quote! {
            impl __l_i18n_crate::keys::ConstArgsMarker for ArgsBuilder {
                type Args = Args<__l_i18n_crate::keys::NoArgs>;
                type Builded = BuildedArgs;
                type ConstBuilder = ();
                const THIS: Self::Args = Args(BuildedArgs::__const_new(), core::marker::PhantomData);
            }

            impl Args<__l_i18n_crate::keys::NoArgs> {
                #[doc(hidden)]
                pub const fn __const_value(self, id: Id, locale: #enum_ident) -> __l_i18n_crate::keys::Literal {
                    match id {
                        #(
                            #const_value_match_arms,
                        )*
                    }
                }
            }

            impl BuildedArgs {
                #[doc(hidden)]
                pub const fn __const_new() -> Self {
                    BuildedArgs { #markers_field: core::marker::PhantomData }
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

                #maybe_async fn get_data(&self, id: Self::Id, locale: Self::Locale) -> Self::Data {
                    __get_data(id, locale) #maybe_await
                }

                fn fmt(
                    &self,
                    formatter: &mut core::fmt::Formatter<'_>,
                    id: Self::Id,
                    locale: Self::Locale,
                    data: &Self::Data,
                ) -> core::fmt::Result {
                    __fmt(&self.0, formatter, id, locale, *data)
                }
            }
        }
    } else {
        quote! {}
    };

    let builder_impl = builder::gen_builder(infos, markers_field);

    let relevant_clone_generics = infos.fields.iter().map(|f| &f.generic);
    let relevant_copy_generics = infos.fields.iter().map(|f| &f.generic);

    let relevant_clone_fields = infos.fields.iter().map(|f| &*f.key.ident);

    let clone_impl = if infos.fields.is_empty() {
        quote!(*self)
    } else {
        quote! {
            Self {
                #markers_field: core::marker::PhantomData,
                #(
                    #relevant_clone_fields: core::clone::Clone::clone(&self.#relevant_clone_fields),
                )*
            }
        }
    };

    quote! {
        pub mod #mod_key {
            #[allow(unused)]
            use super::{#enum_ident, __l_i18n_crate};

            #builder_impl

            #[doc(hidden)]
            pub struct BuildedArgs<#generics> #struct_fields

            impl<#generics> core::clone::Clone for BuildedArgs<#generics>
                where (): core::clone::Clone,
                    #(
                        #relevant_clone_generics: core::clone::Clone,
                    )*
            {
                fn clone(&self) -> Self {
                    #clone_impl
                }
            }

            impl<#generics> core::marker::Copy for BuildedArgs<#generics>
                where (): core::marker::Copy,
                    #(
                        #relevant_copy_generics: core::marker::Copy,
                    )*
            {
            }

            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct ArgsBuilder;

            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub enum Id {
                #(
                    #variants,
                )*
            }

            impl __l_i18n_crate::keys::ArgsBuilder for ArgsBuilder {
                type Id = Id;
                type Builder = Builder<__IntoViewMarker>;
                type Locale = #enum_ident;

                fn new() -> Self::Builder {
                    todo!()
                }
            }

            #[doc(hidden)]
            pub enum __IntoViewMarker {}

            #[doc(hidden)]
            pub enum __DisplayMarker {}

            pub struct Args<__Marker, #generics>(pub BuildedArgs<#generics>, pub core::marker::PhantomData<__Marker>);

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
                fn render(self, id: Self::Id, locale: Self::Locale) -> #render_fn_out_type {
                    let Self(args, _) = self;
                    match id {
                        #(
                            #render_match_arms,
                        )*
                    }
                }
            }

            impl<#bounded_fmt_generics> __l_i18n_crate::keys::DisplayArgs for Args<__DisplayMarker, #generics> {
                type Data = __l_i18n_crate::keys::DisplayData;

                #maybe_async fn get_data(&self, id: Self::Id, locale: Self::Locale) -> Self::Data {
                    __get_data(id, locale) #maybe_await
                }

                fn fmt(
                    &self,
                    formatter: &mut core::fmt::Formatter<'_>,
                    id: Self::Id,
                    locale: Self::Locale,
                    data: &Self::Data,
                ) -> core::fmt::Result {
                    __fmt(&self.0, formatter, id, locale, *data)
                }
            }

            impl<__Marker, #generics> __l_i18n_crate::keys::DowngradableArgs for Args<__Marker, #generics>
                where Self: 'static + Clone + Send + Sync + __l_i18n_crate::keys::IntoViewArgs<Locale = #enum_ident, Id = Id>
            {
                type Downgraded = __l_i18n_crate::keys::AnyIntoViewArgs<#enum_ident>;

                fn downgrade(this: __l_i18n_crate::keys::Key<Self>) -> __l_i18n_crate::keys::Key<Self::Downgraded> {
                    __l_i18n_crate::keys::Key::downgrade_any(this)
                }
            }

            impl<__Marker, #generics> __l_i18n_crate::keys::DowngradableDisplayArgs for Args<__Marker, #generics>
                where Self: 'static + Clone + Send + Sync + __l_i18n_crate::keys::DisplayArgs<Locale = #enum_ident, Id = Id>,
                    Self::Data: Send + Sync + 'static
            {
                type Downgraded = __l_i18n_crate::keys::AnyDisplayArgs<'static, #enum_ident, Self::Data>;

                fn downgrade(this: __l_i18n_crate::keys::Key<Self>) -> __l_i18n_crate::keys::Key<Self::Downgraded> {
                    __l_i18n_crate::keys::Key::downgrade_any_display(this)
                }
            }

            #[doc(hidden)]
            pub #maybe_async fn __get_data(id: Id, locale: #enum_ident) -> __l_i18n_crate::keys::DisplayData {
                match id {
                    #(
                        #get_data_match_arms,
                    )*
                }
            }

            #[doc(hidden)]
            pub fn __fmt<#bounded_fmt_generics>(
                args: &BuildedArgs<#generics>,
                formatter: &mut core::fmt::Formatter<'_>,
                id: Id,
                locale: #enum_ident,
                data: __l_i18n_crate::keys::DisplayData
            ) -> core::fmt::Result {
                match id {
                    #(
                        #fmt_match_arms,
                    )*
                }
            }

            #empty_marker
        }
    }
}
