use leptos_i18n_parser::{
    extraction::Locales,
    options::LocaleName,
    utils::{Key, KeyPath},
};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{
    CodegenOptions,
    codegen::{builders::infos::BuildersInfos, keys::gen_subkeys_impls},
};

pub fn strings_accessor_method_name(locale: &LocaleName) -> syn::Ident {
    format_ident!("__get_{}_translations__", &*locale.key.ident)
}

pub fn gen_locales(
    locales: &Locales,
    keys_ident: &syn::Ident,
    enum_ident: &syn::Ident,
    translation_unit_enum_ident: &syn::Ident,
    namespace: Option<&Key>,
    builders: &BuildersInfos,
    options: &CodegenOptions,
) -> TokenStream {
    let string_holders = gen_strings_holders(
        locales,
        keys_ident,
        enum_ident,
        translation_unit_enum_ident,
        namespace,
        options,
    );

    let string_accessors = locales.locales.iter().map(|locale| {
        let accessor_ident = strings_accessor_method_name(&locale.name);
        let strings_count = locale.strings.len();
        let string_holder = format_ident!("{}_{}", keys_ident, &*locale.name.key.ident);
        if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
            quote! {
                pub async fn #accessor_ident() -> &'static [Box<str>; #strings_count] {
                    #string_holder::get_translations().await
                }
            }
        } else if cfg!(all(feature = "dynamic_load", feature = "ssr")) {
            quote! {
                pub fn #accessor_ident() -> &'static [&'static str; #strings_count] {
                    #string_holder::get_translations()
                }
            }
        } else {
            quote! {
                pub const fn #accessor_ident() -> &'static [&'static str; #strings_count] {
                    #string_holder::get_translations()
                }
            }
        }
    });

    let i18n_request_translations_fn =
        gen_request_translations_fns(locales, keys_ident, enum_ident);

    let init_translations = if cfg!(all(feature = "dynamic_load", feature = "hydrate")) {
        if cfg!(feature = "ssr") {
            quote! {
                #[doc(hidden)]
                pub fn __init_translations__(_locale: #enum_ident, _: (), _values: Vec<Box<str>>) {
                    panic!("Tried to compile with both \"ssr\" and \"hydrate\" features enabled.")
                }
            }
        } else {
            let match_arms = locales.locales.iter().map(|locale| {
                let locale_name = &locale.name;
                let string_holder = get_string_holder_ident(keys_ident, locale_name);
                let locale_key = &locale_name.key;
                quote! {
                    #enum_ident::#locale_key => <#string_holder as __l_i18n_crate::__private::fetch_translations::TranslationUnit>::init_translations(values)
                }
            });
            quote! {
                #[doc(hidden)]
                pub fn __init_translations__(locale: #enum_ident, _: (), values: Vec<Box<str>>) {
                    match locale {
                        #(
                            #match_arms,
                        )*
                    }
                }
            }
        }
    } else {
        quote!()
    };

    let mut path = KeyPath::new(namespace.cloned());

    let keys_impls = gen_subkeys_impls(
        &locales.keys,
        keys_ident,
        enum_ident,
        locales,
        builders,
        &mut path,
        options,
    );

    quote! {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[allow(non_camel_case_types, non_snake_case)]
        #[doc(hidden)]
        pub struct #keys_ident;

        #[allow(dead_code)]
        #[doc(hidden)]
        pub type #translation_unit_enum_ident = ();

        #(#string_holders)*


        impl __l_i18n_crate::scopes::Keys for #keys_ident {
            type BaseLocale = #enum_ident;
            const THIS: Self = #keys_ident;
        }


        impl #keys_ident {
            #(
                #[allow(non_snake_case)]
                #string_accessors
            )*

            #i18n_request_translations_fn

            #init_translations
        }

        #(#keys_impls)*
    }
}

fn get_string_holder_ident(keys_ident: &syn::Ident, locale_name: &LocaleName) -> syn::Ident {
    format_ident!("{}_{}", keys_ident, &*locale_name.key.ident)
}

fn gen_strings_holders<'a>(
    locales: &'a Locales,
    keys_ident: &'a syn::Ident,
    enum_ident: &'a syn::Ident,
    translation_unit_enum_ident: &'a syn::Ident,
    namespace: Option<&'a Key>,
    options: &'a CodegenOptions,
) -> impl Iterator<Item = TokenStream> + use<'a> {
    locales
        .locales
            .iter()
            .map(move |locale| {
                let locale_name = &locale.name;
                let locale_key = &locale_name.key;
                let strings_count = locale.strings.len();
                let strings = &*locale.strings;
                let struct_name = get_string_holder_ident(keys_ident, locale_name);

                let get_fn = if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
                    quote! {
                        pub async fn get_translations() -> &'static [Box<str>; #strings_count] {
                            <Self as __l_i18n_crate::__private::fetch_translations::TranslationUnit>::request_strings().await
                        }
                    }
                } else if cfg!(all(feature = "dynamic_load", feature = "ssr")) {
                    quote! {
                        pub fn get_translations() -> &'static [&'static str; #strings_count] {
                            <Self as __l_i18n_crate::__private::fetch_translations::TranslationUnit>::register();
                            <Self as __l_i18n_crate::__private::fetch_translations::TranslationUnit>::STRINGS
                        }
                    }

                } else {
                    quote! {
                        pub const fn get_translations() -> &'static [&'static str; #strings_count] {
                            <Self as __l_i18n_crate::__private::fetch_translations::TranslationUnit>::STRINGS
                        }
                    }
                };

                let request_translations = if cfg!(all(feature = "dynamic_load", feature = "csr")) {
                    let uri = options.translations_uri.as_ref().expect("Missing URI"); // Already check before
                    // trigger with rustc 1.85, still in nightly tho
                    // #[allow(clippy::literal_string_with_formatting_args)]
                    let endpoint = uri.replace("{locale}", &locale.name.key.name).replace("{namespace}", namespace.map(|k| &*k.name).unwrap_or(""));
                    quote! {
                        pub async fn __i18n_request_translations__() -> Result<__l_i18n_crate::__private::fetch_translations::LocaleServerFnOutput, __l_i18n_crate::reexports::leptos::server_fn::ServerFnError> {
                            use __l_i18n_crate::reexports::leptos::server_fn;

                            #[__l_i18n_crate::reexports::leptos::server(endpoint = #endpoint, prefix = "", input = __l_i18n_crate::reexports::leptos::server_fn::codec::GetUrl, output = __l_i18n_crate::reexports::leptos::server_fn::codec::Json)]
                            pub async fn i18n_request_translations_inner() -> Result<__l_i18n_crate::__private::fetch_translations::LocaleServerFnOutput, server_fn::ServerFnError>;

                            i18n_request_translations_inner().await
                        }
                    }
                } else {
                    quote!()
                };

                let id = if let Some(ns) = namespace {
                    quote!(const ID: super::super::#translation_unit_enum_ident = super::super::#translation_unit_enum_ident::#ns)
                } else {
                    quote!(const ID: () = ())
                };

                let get_string = if cfg!(not(all(feature = "dynamic_load", not(feature = "ssr")))) {
                    quote!{
                        const STRINGS: &[&str; #strings_count] = &[#(#strings,)*];
                    }
                } else {
                    quote! {
                        fn get_strings_lock() -> &'static __l_i18n_crate::__private::fetch_translations::OnceCell<Box<Self::Strings>> {
                            Self::__get_strings_lock()
                        }
                    }
                };

                let string_type = if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
                    quote!([Box<str>; #strings_count])
                } else {
                    quote!([&'static str; #strings_count])
                };

                let translation_unit_impl = quote! {
                    impl __l_i18n_crate::__private::fetch_translations::TranslationUnit for #struct_name {
                        type Locale = #enum_ident;
                        const LOCALE: #enum_ident = #enum_ident::#locale_key;
                        #id;
                        type Strings = #string_type;
                        #get_string
                    }
                };

                let get_strings_lock_fn = if cfg!(all(feature = "dynamic_load", not(feature = "ssr"))) {
                    quote! {
                        fn __get_strings_lock() -> &'static __l_i18n_crate::__private::fetch_translations::OnceCell<Box<[Box<str>; #strings_count]>> {
                            static STRINGS_LOCK: __l_i18n_crate::__private::fetch_translations::OnceCell<Box<[Box<str>; #strings_count]>> = __l_i18n_crate::__private::fetch_translations::OnceCell::new();
                            &STRINGS_LOCK
                        }
                    }
                } else {
                    quote! {}
                };

                quote! {
                    #[allow(non_camel_case_types)]
                    pub struct #struct_name;

                    impl #struct_name {
                        #get_fn

                        #request_translations

                        #get_strings_lock_fn
                    }

                    #translation_unit_impl
                }
            })
}

fn gen_request_translations_fns(
    locales: &Locales,
    keys_ident: &syn::Ident,
    enum_ident: &syn::Ident,
) -> TokenStream {
    let match_arms = locales.locales.iter().map(|locale| {
        let locale_name = &locale.name;
        let string_holder = get_string_holder_ident(keys_ident, locale_name);
        let locale_key = &locale_name.key;
        if cfg!(all(feature = "dynamic_load", feature = "csr")) {
            quote! {
                #enum_ident::#locale_key => #string_holder::__i18n_request_translations__().await
            }
        } else {
            quote! {
                #enum_ident::#locale_key => #string_holder::get_translations()
            }
        }
    });
    let match_stmt = if cfg!(all(
        feature = "dynamic_load",
        not(any(feature = "ssr", feature = "csr"))
    )) {
        quote! {
            unreachable!(
                "This function should not have been called on the client!"
            )
        }
    } else {
        quote! {
            match _locale {
                #(
                    #match_arms,
                )*
            }
        }
    };
    if cfg!(all(feature = "dynamic_load", feature = "csr")) {
        quote! {
            #[doc(hidden)]
            pub async fn __i18n_request_translations__(_locale: #enum_ident, _: ()) -> Result<__l_i18n_crate::__private::fetch_translations::LocaleServerFnOutput, __l_i18n_crate::reexports::leptos::server_fn::ServerFnError> {
                #match_stmt
            }
        }
    } else {
        quote! {
            #[doc(hidden)]
            pub fn __i18n_request_translations__(_locale: #enum_ident, _: ()) -> &'static [&'static str] {
                #match_stmt
            }
        }
    }
}
