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
                super::super::keys::#(#keys ::)* __render(builder, locale)
            };
            let wrapped = either_of.wrap(i, ts);
            quote! {
                Id::#variant => #wrapped
            }
        });

    let empty_marker = if infos.fields.is_empty() {
        let const_value_match_arms = infos.id_variants.iter().map(|(path, variant)| {
            let keys = iter_path_keys(path);
            quote! {
                Id::#variant => super::super::keys::#(#keys ::)* __const_value(locale)
            }
        });

        quote! {
            impl __l_i18n_crate::keys::ArgsMarker<__l_i18n_crate::keys::NoArgs> for ArgsBuilder {
                type Args = Args;

                fn into_args(builder: __l_i18n_crate::keys::NoArgs) -> Self::Args {
                    match builder {}
                }
            }

            impl __l_i18n_crate::keys::ConstArgsMarker for ArgsBuilder {
                const THIS: Args = Args(BuildedArgs::__const_new());
            }

            impl Args {
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
        }
    } else {
        quote! {}
    };

    let builder_impl = builder::gen_builder(infos, markers_field);

    let relevant_clone_generics = infos.fields.iter().map(|f| &f.generic);
    let relevant_copy_generics = infos.fields.iter().map(|f| &f.generic);

    let relevant_clone_fields = infos.fields.iter().map(|f| &*f.key.ident);

    let render_fn_out_type = if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
        quote!(impl __l_i18n_crate::keys::IntoViewFuture)
    } else {
        quote!(impl __l_i18n_crate::reexports::leptos::IntoView)
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
                    Self {
                        #markers_field: core::marker::PhantomData,
                        #(
                            #relevant_clone_fields: core::clone::Clone::clone(&self.#relevant_clone_fields),
                        )*
                    }
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
                type Builder = Builder;
                type Locale = #enum_ident;

                fn new() -> Self::Builder {
                    todo!()
                }
            }


            pub struct Args<#generics>(pub BuildedArgs<#generics>);

            impl<#generics> core::clone::Clone for Args<#generics> where BuildedArgs<#generics>: core::clone::Clone {
                fn clone(&self) -> Self {
                    Self(core::clone::Clone::clone(&self.0))
                }
            }

            impl<#generics> core::marker::Copy for Args<#generics> where BuildedArgs<#generics>: core::marker::Copy {}


            impl<#bounded_generics> __l_i18n_crate::keys::ArgsMarker<BuildedArgs<#generics>> for ArgsBuilder {
                type Args = Args<#generics>;

                fn into_args(builder: BuildedArgs<#generics>) -> Self::Args {
                    Args(builder)
                }
            }

            impl<#bounded_generics> __l_i18n_crate::keys::Args for Args<#generics> {
                type Locale = #enum_ident;
                type Id = Id;

                fn render(self, id: Self::Id, locale: Self::Locale) -> #render_fn_out_type {
                    let Self(builder) = self;
                    match id {
                        #(
                            #render_match_arms,
                        )*
                    }
                }
            }

            impl<#bounded_generics> __l_i18n_crate::keys::DowngradableArgs for Args<#generics>
                where Self: Send + Sync
            {
                type Downgraded = __l_i18n_crate::keys::AnyArgs<#enum_ident>;

                fn downgrade(this: __l_i18n_crate::keys::Key<Self>) -> __l_i18n_crate::keys::Key<Self::Downgraded> {
                    __l_i18n_crate::keys::Key::downgrade_any(this)
                }
            }

            #empty_marker
        }
    }
}
