use std::collections::{BTreeMap, HashSet};

use leptos_i18n_parser::{
    extraction::{Builder, BuilderId, Builders},
    utils::{Key, KeyPath},
};
use proc_macro2::{Span, TokenStream};
use quote::quote;

use crate::utils::EitherOfWrapper;

pub struct BuilderInfos {
    pub id_variants: BTreeMap<KeyPath, syn::Ident>,
    pub generics: TokenStream,
    pub bounded_generics: TokenStream,
    pub empty_generics: TokenStream,
    pub empty_fields: TokenStream,
    pub fields: TokenStream,
    pub destructured: TokenStream,
    pub is_empty: bool,
    pub name: Key,
}

pub struct BuildersInfos {
    pub infos: BTreeMap<BuilderId, BuilderInfos>,
}

pub fn gen_builder_module(
    builders: &Builders,
    enum_ident: &syn::Ident,
) -> (TokenStream, BuildersInfos) {
    let infos = builders
        .builders
        .iter()
        .map(|(id, builder)| {
            let infos = gen_builder_info(builder);
            (id.clone(), infos)
        })
        .collect();

    let builder_infos = BuildersInfos { infos };

    let ts = gen_module(&builder_infos, enum_ident);

    (ts, builder_infos)
}

fn gen_module(infos: &BuildersInfos, enum_ident: &syn::Ident) -> TokenStream {
    let inner_modules = infos
        .infos
        .values()
        .map(|infos| gen_inner_module(infos, enum_ident));

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

fn gen_inner_module(infos: &BuilderInfos, enum_ident: &syn::Ident) -> TokenStream {
    let mod_key = &*infos.name.ident;

    let variants = infos.id_variants.values();
    let bounded_generics = &infos.bounded_generics;
    let fields = &infos.fields;
    let empty_generics = &infos.empty_generics;
    let empty_fields = &infos.empty_fields;
    let generics = &infos.generics;

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

    let empty_marker = if infos.is_empty {
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
                const THIS: Args = Args(BuildedArgs {});
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
        }
    } else {
        quote! {}
    };

    quote! {
        pub mod #mod_key {
            #[allow(unused)]
            use super::{#enum_ident, __l_i18n_crate};

            pub type Builder = BuildedArgs #empty_generics;

            #[doc(hidden)]
            #[derive(Clone, Copy)]
            pub struct BuildedArgs #bounded_generics {
                #fields
            }

            impl Builder {
                pub fn new() -> Self {
                    Builder {
                        #empty_fields
                    }
                }
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
                    Builder::new()
                }
            }

            #[derive(Clone, Copy)]
            pub struct Args #bounded_generics (pub BuildedArgs #generics);

            impl #bounded_generics __l_i18n_crate::keys::ArgsMarker<BuildedArgs #generics> for ArgsBuilder {
                type Args = Args #generics;

                fn into_args(builder: BuildedArgs #generics) -> Self::Args {
                    Args(builder)
                }
            }

            impl #bounded_generics __l_i18n_crate::keys::Args for Args {
                type Locale = #enum_ident;
                type Id = Id;
                type Downgraded = __l_i18n_crate::keys::AnyArgs<#enum_ident>;

                fn downgrade(this: __l_i18n_crate::keys::Key<Self>) -> __l_i18n_crate::keys::Key<Self::Downgraded> {
                    __l_i18n_crate::keys::Key::downgrade_any(this)
                }

                fn render(self, id: Self::Id, locale: Self::Locale) -> impl __l_i18n_crate::reexports::leptos::IntoView {
                    let Self(builder) = self;
                    match id {
                        #(
                            #render_match_arms,
                        )*
                    }
                }
            }

            #empty_marker
        }
    }
}

fn generate_variant_ident(keypath: &KeyPath, variants: &mut HashSet<String>) -> syn::Ident {
    use core::fmt::Write;
    let mut buff = String::new();
    if let Some(ns) = &keypath.namespace {
        write!(&mut buff, "{}_", &ns.ident).unwrap();
    }

    for key in &keypath.path {
        write!(&mut buff, "{}_", &key.ident).unwrap();
    }

    while variants.contains(&buff) {
        buff.push('_');
    }

    let ident = syn::Ident::new(&buff, Span::call_site());

    variants.insert(buff);

    ident
}

fn gen_builder_info(builder: &Builder) -> BuilderInfos {
    let mut variants = HashSet::new();
    let id_variants = builder
        .used_by
        .iter()
        .map(|keypath| {
            let ident = generate_variant_ident(keypath, &mut variants);
            (keypath.clone(), ident)
        })
        .collect();

    let is_empty = builder.keys.components.is_empty() && builder.keys.vars.is_empty();

    let generics = quote! {};
    let bounded_generics = quote! {};
    let empty_generics = quote! {};
    let empty_fields = quote! {};
    let fields = quote! {};
    let destructured = quote! {
        {}
    };

    BuilderInfos {
        name: builder.name.clone(),
        id_variants,
        generics,
        bounded_generics,
        empty_generics,
        empty_fields,
        fields,
        destructured,
        is_empty,
    }
}
