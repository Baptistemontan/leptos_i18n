use std::{cell::RefCell, collections::HashMap, ops::Not, rc::Rc};

pub mod cfg_file;
pub mod error;
pub mod interpolate;
pub mod key;
pub mod locale;
pub mod parsed_value;
pub mod plural;
pub mod warning;

use cfg_file::ConfigFile;
use error::Result;
use interpolate::{create_empty_type, Interpolation};
use key::Key;
use locale::{Locale, LocaleValue};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use self::{
    locale::{BuildersKeys, BuildersKeysInner, LocalesOrNamespaces, Namespace},
    warning::generate_warnings,
};

pub fn load_locales() -> Result<TokenStream> {
    let cfg_file = ConfigFile::new()?;
    let locales = LocalesOrNamespaces::new(&cfg_file)?;

    let keys = Locale::check_locales(locales)?;

    let locale_type = create_locale_type(keys);
    let locale_variants = create_locales_enum(&cfg_file);
    let locales = create_locales_type(&cfg_file);

    let warnings = generate_warnings();

    Ok(quote! {
        pub mod i18n {
            #locales

            #locale_variants

            #locale_type

            #[inline]
            pub fn use_i18n() -> leptos_i18n::I18nContext<Locales> {
                leptos_i18n::use_i18n_context()
            }

            #[inline]
            pub fn provide_i18n_context() -> leptos_i18n::I18nContext<Locales> {
                leptos_i18n::provide_i18n_context()
            }

            pub use leptos_i18n::{t, td};

            #warnings
        }
    })
}

fn create_locales_enum(cfg_file: &ConfigFile) -> TokenStream {
    let ConfigFile {
        default, locales, ..
    } = cfg_file;

    let as_str_match_arms = locales
        .iter()
        .map(|key| (&key.ident, &key.name))
        .map(|(variant, locale)| quote!(LocaleEnum::#variant => #locale))
        .collect::<Vec<_>>();

    let from_str_match_arms = locales
        .iter()
        .map(|key| (&key.ident, &key.name))
        .map(|(variant, locale)| quote!(#locale => Some(LocaleEnum::#variant)))
        .collect::<Vec<_>>();

    let derives = if cfg!(feature = "serde") {
        quote!(#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)])
    } else {
        quote!(#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)])
    };

    quote! {
        #derives
        #[allow(non_camel_case_types)]
        pub enum LocaleEnum {
            #(#locales,)*
        }

        impl Default for LocaleEnum {
            fn default() -> Self {
                LocaleEnum::#default
            }
        }

        impl leptos_i18n::LocaleVariant for LocaleEnum {
            type Keys = I18nKeys;

            fn as_str(self) -> &'static str {
                match self {
                    #(#as_str_match_arms,)*
                }
            }
            fn from_str(s: &str) -> Option<Self> {
                match s {
                    #(#from_str_match_arms,)*
                    _ => None
                }
            }
        }
    }
}

fn create_locales_type(_cfg_file: &ConfigFile) -> TokenStream {
    quote! {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        pub struct Locales;

        impl leptos_i18n::Locales for Locales {
            type Variants = LocaleEnum;
            type LocaleKeys = I18nKeys;
        }
    }
}

struct Subkeys<'a> {
    original_key: &'a syn::Ident,
    key: syn::Ident,
    mod_key: syn::Ident,
    locales: &'a [Rc<RefCell<Locale>>],
    keys: &'a BuildersKeysInner,
}

impl<'a> Subkeys<'a> {
    pub fn new(
        key: &'a Key,
        locales: &'a [Rc<RefCell<Locale>>],
        keys: &'a BuildersKeysInner,
    ) -> Self {
        let original_key = &key.ident;
        let mod_key = format_ident!("sk_{}", key.ident);
        let key = format_ident!("{}_subkeys", key.ident);
        Subkeys {
            original_key,
            key,
            mod_key,
            locales,
            keys,
        }
    }
}

fn create_locale_type_inner(
    type_ident: &syn::Ident,
    top_locales: &[Rc<RefCell<Locale>>],
    locales: &[Rc<RefCell<Locale>>],
    keys: &HashMap<Rc<Key>, LocaleValue>,
    is_namespace: bool,
) -> TokenStream {
    let string_keys = keys
        .iter()
        .filter(|(_, value)| matches!(value, LocaleValue::Value(None)))
        .map(|(key, _)| key)
        .collect::<Vec<_>>();

    let string_fields = string_keys
        .iter()
        .map(|key| quote!(pub #key: &'static str))
        .collect::<Vec<_>>();

    let subkeys = keys
        .iter()
        .filter_map(|(key, value)| match value {
            LocaleValue::Subkeys { locales, keys } => Some(Subkeys::new(key, locales, keys)),
            _ => None,
        })
        .collect::<Vec<_>>();

    let subkeys_ts = subkeys.iter().map(|sk| {
        let subkey_mod_ident = &sk.mod_key;
        let subkey_impl =
            create_locale_type_inner(&sk.key, top_locales, sk.locales, &sk.keys.0, true);
        quote! {
            pub mod #subkey_mod_ident {
                use super::LocaleEnum;

                #subkey_impl
            }
        }
    });

    let subkeys_fields = subkeys.iter().map(|sk| {
        let original_key = &sk.original_key;
        let key = &sk.key;
        let mod_ident = &sk.mod_key;
        quote!(pub #original_key: subkeys::#mod_ident::#key)
    });

    let subkeys_field_new = subkeys
        .iter()
        .map(|sk| {
            let original_key = &sk.original_key;
            let key = &sk.key;
            let mod_ident = &sk.mod_key;
            quote!(#original_key: subkeys::#mod_ident::#key::new(_variant))
        })
        .collect::<Vec<_>>();

    let subkeys_module = subkeys.is_empty().not().then(move || {
        quote! {
            #[doc(hidden)]
            pub mod subkeys {
                use super::LocaleEnum;

                #(
                    #subkeys_ts
                )*
            }
        }
    });

    let builders = keys
        .iter()
        .filter_map(|(key, value)| match value {
            LocaleValue::Value(None) | LocaleValue::Subkeys { .. } => None,
            LocaleValue::Value(Some(keys)) => {
                Some((key, Interpolation::new(key, keys, top_locales, locales)))
            }
        })
        .collect::<Vec<_>>();

    let builder_fields = builders.iter().map(|(key, inter)| {
        let inter_ident = &inter.default_generic_ident;
        quote!(pub #key: builders::#inter_ident)
    });

    let init_builder_fields: Vec<TokenStream> = builders
        .iter()
        .map(|(key, inter)| {
            let ident = &inter.ident;
            quote!(#key: builders::#ident::new(_variant))
        })
        .collect();

    let new_match_arms = top_locales.iter().zip(locales).map(|(top_locale, locale)| {
        let locale_ref = locale.borrow();
        let filled_string_fields = locale_ref
            .keys
            .iter()
            .filter(|(key, _)| {
                keys.get(*key)
                    .is_some_and(|value| matches!(value, LocaleValue::Value(None)))
            })
            .filter_map(|(key, value)| {
                let str_value = value.is_string()?;
                Some(quote!(#key: #str_value))
            });

        let ident = &top_locale.borrow().name.ident;
        quote! {
            LocaleEnum::#ident => #type_ident {
                #(#filled_string_fields,)*
                #(#init_builder_fields,)*
                #(#subkeys_field_new,)*
            }
        }
    });

    let builder_impls = builders.iter().map(|(_, inter)| &inter.imp);

    let builder_module = builders.is_empty().not().then(move || {
        let empty_type = create_empty_type();
        quote! {
            #[doc(hidden)]
            pub mod builders {
                use super::LocaleEnum;

                #empty_type

                #(
                    #builder_impls
                )*
            }
        }
    });

    let (from_variant, const_values) = if !is_namespace {
        let from_variant_match_arms = top_locales.iter().map(|locale| {
            let ident = &locale.borrow().name.ident;
            quote!(LocaleEnum::#ident => &Self::#ident)
        });

        let from_variant = quote! {
            impl leptos_i18n::LocaleKeys for #type_ident {
                type Variants = LocaleEnum;
                fn from_variant(_variant: LocaleEnum) -> &'static Self {
                    match _variant {
                        #(
                            #from_variant_match_arms,
                        )*
                    }
                }
            }
        };

        let const_values = top_locales.iter().map(|locale| {
            let ident = &locale.borrow().name.ident;
            quote!(pub const #ident: Self = Self::new(LocaleEnum::#ident);)
        });

        let const_values = quote! {
            #(
                #[allow(non_upper_case_globals)]
                #const_values
            )*
        };

        (Some(from_variant), Some(const_values))
    } else {
        (None, None)
    };

    quote! {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[allow(non_camel_case_types)]
        pub struct #type_ident {
            #(#string_fields,)*
            #(#builder_fields,)*
            #(#subkeys_fields,)*
        }

        impl #type_ident {

            #const_values

            pub const fn new(_variant: LocaleEnum) -> Self {
                match _variant {
                    #(
                        #new_match_arms,
                    )*
                }
            }
        }

        #from_variant

        #builder_module

        #subkeys_module
    }
}

fn create_namespace_mod_ident(namespace_ident: &syn::Ident) -> syn::Ident {
    format_ident!("ns_{}", namespace_ident)
}

fn create_namespaces_types(
    i18n_keys_ident: &syn::Ident,
    namespaces: &[Namespace],
    keys: &HashMap<Rc<Key>, BuildersKeysInner>,
) -> TokenStream {
    let namespaces_ts = namespaces.iter().map(|namespace| {
        let namespace_ident = &namespace.key.ident;
        let namespace_module_ident = create_namespace_mod_ident(namespace_ident);
        let keys = keys.get(&namespace.key).unwrap();
        let type_impl = create_locale_type_inner(
            namespace_ident,
            &namespace.locales,
            &namespace.locales,
            &keys.0,
            true,
        );
        quote! {
            pub mod #namespace_module_ident {
                use super::LocaleEnum;

                #type_impl
            }
        }
    });

    let namespaces_fields = namespaces.iter().map(|namespace| {
        let key = &namespace.key;
        let namespace_module_ident = create_namespace_mod_ident(&key.ident);
        quote!(pub #key: namespaces::#namespace_module_ident::#key)
    });

    let namespaces_fields_new = namespaces.iter().map(|namespace| {
        let key = &namespace.key;
        let namespace_module_ident = create_namespace_mod_ident(&key.ident);
        quote!(#key: namespaces::#namespace_module_ident::#key::new(_variant))
    });

    let locales = &namespaces.iter().next().unwrap().locales;

    let const_values = locales.iter().map(|locale| {
        let locale_ident = &locale.borrow().name;
        quote!(pub const #locale_ident: Self = Self::new(LocaleEnum::#locale_ident);)
    });

    let from_variant_match_arms = locales.iter().map(|locale| {
        let locale_ident = &locale.borrow().name;
        quote!(LocaleEnum::#locale_ident => &Self::#locale_ident)
    });

    quote! {
        pub mod namespaces {
            use super::LocaleEnum;

            #(
                #namespaces_ts
            )*

        }

        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[allow(non_snake_case)]
        pub struct #i18n_keys_ident {
            #(#namespaces_fields,)*
        }

        impl #i18n_keys_ident {
            #(
                #[allow(non_upper_case_globals)]
                #const_values
            )*

            pub const fn new(_variant: LocaleEnum) -> Self {
                Self {
                    #(
                        #namespaces_fields_new,
                    )*
                }
            }
        }

        impl leptos_i18n::LocaleKeys for #i18n_keys_ident {
            type Variants = LocaleEnum;
            fn from_variant(_variant: LocaleEnum) -> &'static Self {
                match _variant {
                    #(
                        #from_variant_match_arms,
                    )*
                }
            }
        }
    }
}

fn create_locale_type(keys: BuildersKeys) -> TokenStream {
    let i18n_keys_ident = format_ident!("I18nKeys");
    match keys {
        BuildersKeys::NameSpaces { namespaces, keys } => {
            create_namespaces_types(&i18n_keys_ident, &namespaces, &keys)
        }
        BuildersKeys::Locales { locales, keys } => {
            create_locale_type_inner(&i18n_keys_ident, &locales, &locales, &keys.0, false)
        }
    }
}
