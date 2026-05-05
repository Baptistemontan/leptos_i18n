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

    let (maybe_async, maybe_await) = if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
        (quote!(async), quote!(.await))
    } else {
        (quote!(), quote!())
    };

    let id_key_match_arms = infos.id_variants.iter().map(|(path, variant)| {
        let keys = iter_path_keys(path);
        quote! {
            Id::#variant => {
                use super::super::keys::#(#keys ::)* Id as __Id;
                <__Id as __l_i18n_crate::keys::KeyId>::key(__Id)
            }
        }
    });

    let render_match_arms = infos
        .id_variants
        .iter()
        .enumerate()
        .map(|(i, (path, variant))| {
            let keys = iter_path_keys(path);
            let ts = quote! {
                super::super::keys::#(#keys ::)* __render(args, locale) #maybe_await
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

    let get_data_match_arms = infos.id_variants.iter().map(|(path, variant)| {
        let keys = iter_path_keys(path);
        quote! {
            Id::#variant => super::super::keys::#(#keys ::)* __get_data(locale) #maybe_await
        }
    });

    let render_fn_out_type = if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
        quote!(impl __l_i18n_crate::keys::IntoViewFuture)
    } else {
        quote!(impl __l_i18n_crate::reexports::leptos::IntoView + core::clone::Clone + 'static)
    };

    let empty_marker = if infos.fields.is_empty() {
        let const_value_match_arms = infos.id_variants.iter().map(|(path, variant)| {
            let keys = iter_path_keys(path);
            quote! {
                Id::#variant => super::super::keys::#(#keys ::)* __const_value_as_lit(locale)
            }
        });

        quote! {
            impl __l_i18n_crate::keys::comp_time::ConstArgsMarker for ArgsBuilder {
                type Args = Args<__l_i18n_crate::keys::comp_time::NoArgs>;
                type Builded = BuildedArgs;
                type Value = __l_i18n_crate::keys::comp_time::Literal;
            }

            impl __l_i18n_crate::keys::comp_time::ConstArgs for Args<__l_i18n_crate::keys::comp_time::NoArgs> {
                const THIS: Self = Args(BuildedArgs::__const_new(), core::marker::PhantomData);
                type Value = __l_i18n_crate::keys::comp_time::Literal;

                fn value(id: Id, locale: #enum_ident) -> __l_i18n_crate::keys::comp_time::Literal {
                    Self::__const_value(Self::THIS, id, locale)
                }
            }

            impl Args<__l_i18n_crate::keys::comp_time::NoArgs> {
                #[doc(hidden)]
                pub const fn __const_value(self, id: Id, locale: #enum_ident) -> __l_i18n_crate::keys::comp_time::Literal {
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

            impl<#generics> __l_i18n_crate::keys::view::IntoViewArgs for Args<__l_i18n_crate::keys::comp_time::NoArgs, #generics>
                where Args<__IntoViewMarker, #generics>: __l_i18n_crate::keys::view::IntoViewArgs,
            {
                fn render(self, id: Self::Id, locale: Self::Locale) -> #render_fn_out_type {
                    <Args<__IntoViewMarker, #generics> as __l_i18n_crate::keys::view::IntoViewArgs>::render(Args(
                        self.0,
                        core::marker::PhantomData
                    ), id, locale)
                }
            }

            impl<#bounded_fmt_generics> __l_i18n_crate::keys::display::DisplayArgs for Args<__l_i18n_crate::keys::comp_time::NoArgs, #generics> {
                type Data = __l_i18n_crate::keys::display::DisplayData;

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
                    __fmt(&self.0, formatter, id, locale, data)
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

    let builded_args_clone_impl = if infos.fields.is_empty() {
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

    let args_clone_impl = if infos.fields.is_empty() {
        quote!(*self)
    } else {
        quote! {
            Self(core::clone::Clone::clone(&self.0), core::marker::PhantomData)
        }
    };

    let render_match = if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
        quote!(async move {
            match id {
                #(
                    #render_match_arms,
                )*
            }
        })
    } else {
        quote!({
                match id {
                #(
                    #render_match_arms,
                )*
            }
        })
    };

    quote! {
        pub mod #mod_key {
            #[allow(unused)]
            use super::{#enum_ident, __l_i18n_crate};

            #[doc(hidden)]
            pub enum __IntoViewMarker {}

            #[doc(hidden)]
            pub enum __DisplayMarker {}

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
                    #builded_args_clone_impl
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

            impl __l_i18n_crate::keys::KeyId for Id {
                fn key(self) -> &'static str {
                    match self {
                        #(
                            #id_key_match_arms,
                        )*
                    }
                }
            }

            impl __l_i18n_crate::keys::builder::ArgsBuilder for ArgsBuilder {
                type Id = Id;
                type Builder = Builder<__IntoViewMarker>;
                type Locale = #enum_ident;

                fn new() -> Self::Builder {
                    Builder::__new()
                }
            }

            impl __l_i18n_crate::keys::display::DisplayArgsBuilder for ArgsBuilder {
                type DisplayBuilder = Builder<__DisplayMarker>;

                fn new_display() -> Self::DisplayBuilder {
                    Builder::__new()
                }
            }

            pub struct Args<__Marker, #generics>(pub BuildedArgs<#generics>, pub core::marker::PhantomData<__Marker>);

            impl<__Marker, #generics> core::clone::Clone for Args<__Marker, #generics> where BuildedArgs<#generics>: core::clone::Clone {
                fn clone(&self) -> Self {
                    #args_clone_impl
                }
            }

            impl<__Marker, #generics> core::marker::Copy for Args<__Marker, #generics> where BuildedArgs<#generics>: core::marker::Copy {}


            impl<#bounded_generics> __l_i18n_crate::keys::view::IntoViewArgsMarker<BuildedArgs<#generics>> for ArgsBuilder {
                type Args = Args<__IntoViewMarker, #generics>;

                fn into_args(builder: BuildedArgs<#generics>) -> Self::Args {
                    Args(builder, core::marker::PhantomData)
                }
            }

            impl<#bounded_fmt_generics> __l_i18n_crate::keys::display::DisplayArgsMarker<BuildedArgs<#generics>> for ArgsBuilder {
                type Args = Args<__DisplayMarker, #generics>;

                fn into_args(builder: BuildedArgs<#generics>) -> Self::Args {
                    Args(builder, core::marker::PhantomData)
                }
            }

            impl<__Marker, #generics> __l_i18n_crate::keys::args::Args for Args<__Marker, #generics> {
                type Locale = #enum_ident;
                type Id = Id;
            }

            impl<#bounded_generics> __l_i18n_crate::keys::view::IntoViewArgs for Args<__IntoViewMarker, #generics> {
                fn render(self, id: Self::Id, locale: Self::Locale) -> #render_fn_out_type {
                    let Self(args, _) = self;
                    #render_match
                }
            }

            impl<#bounded_fmt_generics> __l_i18n_crate::keys::display::DisplayArgs for Args<__DisplayMarker, #generics> {
                type Data = __l_i18n_crate::keys::display::DisplayData;

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
                    __fmt(&self.0, formatter, id, locale, data)
                }
            }

            impl<__Marker, #generics> __l_i18n_crate::keys::view::DowngradableArgs for Args<__Marker, #generics>
                where Self: 'static + Clone + Send + Sync + __l_i18n_crate::keys::view::IntoViewArgs<Locale = #enum_ident, Id = Id>
            {
                type Downgraded = __l_i18n_crate::keys::view::AnyIntoViewArgs<#enum_ident>;

                fn downgrade(this: __l_i18n_crate::keys::Key<Self>) -> __l_i18n_crate::keys::Key<Self::Downgraded> {
                    __l_i18n_crate::keys::Key::downgrade_any(this)
                }
            }

            impl<__Marker, #generics> __l_i18n_crate::keys::display::DowngradableDisplayArgs for Args<__Marker, #generics>
                where Self: 'static + Clone + Send + Sync + __l_i18n_crate::keys::display::DisplayArgs<Locale = #enum_ident, Id = Id>,
                    Self::Data: Send + Sync + 'static
            {
                type Downgraded = __l_i18n_crate::keys::display::AnyDisplayArgs<'static, #enum_ident, Self::Data>;

                fn downgrade(this: __l_i18n_crate::keys::Key<Self>) -> __l_i18n_crate::keys::Key<Self::Downgraded> {
                    __l_i18n_crate::keys::Key::downgrade_any_display(this)
                }
            }

            #[doc(hidden)]
            pub #maybe_async fn __get_data(id: Id, locale: #enum_ident) -> __l_i18n_crate::keys::display::DisplayData {
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
                data: &__l_i18n_crate::keys::display::DisplayData
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
