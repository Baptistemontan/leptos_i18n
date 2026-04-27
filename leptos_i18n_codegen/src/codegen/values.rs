use leptos_i18n_parser::{
    extraction::{Literal, Locales, Value, Values},
    utils::{Key, KeyPath},
};
use proc_macro2::TokenStream;
use quote::quote;

use crate::{CodegenOptions, codegen::builders::infos::BuildersInfos};

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
    let generics = builder_infos.generics();
    let destructure = &builder_infos.destructure(markers_field);

    let empty_marker = if builder_infos.fields.is_empty() {
        let match_arms = gen_const_values_match_arms(values, enum_ident, locales);
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
                pub const fn __const_value(self, _: Id, locale: #enum_ident) -> __l_i18n_crate::keys::Literal {
                    __const_value(locale)
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

    quote! {
        #docs
        pub mod #key {
            #[allow(unused)]
            use super::{#enum_ident, __l_i18n_crate, __builders};
            pub type Builder = __builders::#builder_name::Builder;
            pub type BuildedArgs<#generics> = __builders::#builder_name::BuildedArgs<#generics>;

            pub type Id = ();

            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
            pub struct ArgsBuilder;

            impl __l_i18n_crate::keys::ArgsBuilder for ArgsBuilder {
                type Id = Id;
                type Builder = Builder;
                type Locale = #enum_ident;

                fn new() -> Self::Builder {
                    Builder::new()
                }
            }

            impl __l_i18n_crate::keys::DowngradableArgBuilder for ArgsBuilder {
                type Downgraded = __builders::#builder_name::ArgsBuilder;
                const ID: __builders::#builder_name::Id = __builders::#builder_name::Id::#variant_ident;
            }


            pub struct Args<#generics>(BuildedArgs<#generics>);

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

                fn render(self, _: (), locale: Self::Locale) -> impl __l_i18n_crate::reexports::leptos::IntoView {
                    let Self(builder) = self;
                    __render(builder, locale)
                }
            }

            impl<#bounded_generics> __l_i18n_crate::keys::DowngradableArgs for Args<#generics> {
                type Downgraded = __builders::#builder_name::Args<#generics>;

                fn downgrade(this: __l_i18n_crate::keys::Key<Self>) -> __l_i18n_crate::keys::Key<Self::Downgraded> {
                    let (Self(args), ()) = __l_i18n_crate::keys::Key::into_args_and_id(this);
                    let args = __builders::#builder_name::Args(args);
                    __l_i18n_crate::keys::Key::from_args_and_id(args, __builders::#builder_name::Id::#variant_ident)
                }
            }

            #[doc(hidden)]
            pub fn __render< #bounded_generics >(args: BuildedArgs< #generics >, locale: #enum_ident) -> impl __l_i18n_crate::reexports::leptos::IntoView {
                let BuildedArgs #destructure = args;
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

fn gen_const_values_match_arms(
    values: &Values,
    enum_ident: &syn::Ident,
    locales: &Locales,
) -> impl Iterator<Item = TokenStream> {
    //TODO: check defaulting
    values.values.iter().map(move |(locale, value)| {
        let Value::Literal(lit) = value else { todo!() };

        let value = match lit {
            Literal::String(index) => {
                // TODO: use direct string only on client dyn_load, use the static strs otherwise.
                let loc = locales
                    .locales
                    .iter()
                    .find(|l| l.name.key == *locale)
                    .expect("to find the locale for this value");
                let value = &*loc.strings[*index];
                quote!(__l_i18n_crate::keys::Literal::String(#value))
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
