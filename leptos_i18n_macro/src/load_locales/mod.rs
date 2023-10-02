use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, Not},
    path::PathBuf,
    rc::Rc,
};

pub mod cfg_file;
pub mod error;
pub mod interpolate;
pub mod key;
pub mod locale;
pub mod parsed_value;
pub mod plural;
pub mod warning;

use cfg_file::ConfigFile;
use error::{Error, Result};
use interpolate::{create_empty_type, Interpolation};
use key::Key;
use locale::{Locale, LocaleValue};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::load_locales::parsed_value::ParsedValue;

use self::{
    locale::{BuildersKeys, BuildersKeysInner, LocalesOrNamespaces, Namespace},
    warning::generate_warnings,
};

pub fn load_locales() -> Result<TokenStream> {
    let mut cargo_manifest_dir: PathBuf = std::env::var("CARGO_MANIFEST_DIR")
        .map_err(Error::CargoDirEnvNotPresent)?
        .into();

    let cfg_file = ConfigFile::new(&mut cargo_manifest_dir)?;
    let mut locales = LocalesOrNamespaces::new(&mut cargo_manifest_dir, &cfg_file)?;

    ParsedValue::resolve_foreign_keys(&locales, &cfg_file.default)?;

    let keys = Locale::check_locales(&mut locales)?;

    let locale_type = create_locale_type(keys, &cfg_file);
    let locale_enum = create_locales_enum(&cfg_file);

    let warnings = generate_warnings();

    let macros_reexport = if cfg!(feature = "interpolate_display") {
        quote!(
            pub use leptos_i18n::{t, td, td_string};
        )
    } else {
        quote!(
            pub use leptos_i18n::{t, td};
        )
    };

    Ok(quote! {
        pub mod i18n {
            #locale_enum

            #locale_type

            #[inline]
            pub fn use_i18n() -> leptos_i18n::I18nContext<Locale> {
                leptos_i18n::use_i18n_context()
            }

            #[inline]
            pub fn provide_i18n_context() -> leptos_i18n::I18nContext<Locale> {
                leptos_i18n::provide_i18n_context()
            }

            #macros_reexport

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
        .map(|(variant, locale)| quote!(Locale::#variant => #locale))
        .collect::<Vec<_>>();

    let from_str_match_arms = locales
        .iter()
        .map(|key| (&key.ident, &key.name))
        .map(|(variant, locale)| quote!(#locale => Some(Locale::#variant)))
        .collect::<Vec<_>>();

    let derives = if cfg!(feature = "serde") {
        quote!(#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)])
    } else {
        quote!(#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)])
    };

    quote! {
        #derives
        #[allow(non_camel_case_types)]
        pub enum Locale {
            #(#locales,)*
        }

        impl Default for Locale {
            fn default() -> Self {
                Locale::#default
            }
        }

        impl leptos_i18n::Locale for Locale {
            type Keys = I18nKeys;

            fn as_str(self) -> &'static str {
                match self {
                    #(#as_str_match_arms,)*
                }
            }
            fn from_str(s: &str) -> Option<Self> {
                match s.trim() {
                    #(#from_str_match_arms,)*
                    _ => None
                }
            }
        }
    }
}

struct Subkeys<'a> {
    original_key: &'a syn::Ident,
    key: syn::Ident,
    mod_key: syn::Ident,
    locales: &'a [Locale],
    keys: &'a BuildersKeysInner,
}

impl<'a> Subkeys<'a> {
    pub fn new(key: &'a Key, locales: &'a [Locale], keys: &'a BuildersKeysInner) -> Self {
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

fn get_default_match(
    default_locale: &Key,
    top_locales: &HashSet<&Key>,
    locales: &[Locale],
) -> TokenStream {
    let current_keys = locales
        .iter()
        .map(|locale| &*locale.top_locale_name)
        .collect();
    let missing_keys = top_locales.difference(&current_keys);
    quote!(Locale::#default_locale #(| Locale::#missing_keys)*)
}

fn create_locale_type_inner(
    default_locale: &Key,
    type_ident: &syn::Ident,
    top_locales: &HashSet<&Key>,
    locales: &[Locale],
    keys: &HashMap<Rc<Key>, LocaleValue>,
    is_namespace: bool,
) -> TokenStream {
    let default_match = get_default_match(default_locale, top_locales, locales);

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
        let subkey_impl = create_locale_type_inner(
            default_locale,
            &sk.key,
            top_locales,
            sk.locales,
            &sk.keys.0,
            true,
        );
        quote! {
            pub mod #subkey_mod_ident {
                use super::Locale;

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
            quote!(#original_key: subkeys::#mod_ident::#key::new(_locale))
        })
        .collect::<Vec<_>>();

    let subkeys_module = subkeys.is_empty().not().then(move || {
        quote! {
            #[doc(hidden)]
            pub mod subkeys {
                use super::Locale;

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
                Some((key, Interpolation::new(key, keys, locales, &default_match)))
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
            quote!(#key: builders::#ident::new(_locale))
        })
        .collect();

    let default_locale = locales.first().unwrap();

    let new_match_arms = locales.iter().enumerate().map(|(i, locale)| {
        let filled_string_fields =
            string_keys
                .iter()
                .filter_map(|&key| match locale.keys.get(key) {
                    Some(ParsedValue::String(str_value)) => Some(quote!(#key: #str_value)),
                    _ => {
                        let str_value = default_locale
                            .keys
                            .get(key)
                            .and_then(ParsedValue::is_string)?;
                        Some(quote!(#key: #str_value))
                    }
                });

        let ident = &locale.top_locale_name;
        let pattern = (i != 0).then(|| quote!(Locale::#ident));
        let pattern = pattern.as_ref().unwrap_or(&default_match);
        quote! {
            #pattern => #type_ident {
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
                use super::Locale;

                #empty_type

                #(
                    #builder_impls
                )*
            }
        }
    });

    let (from_locale, const_values) = if !is_namespace {
        let from_locale_match_arms = top_locales
            .iter()
            .map(|locale| quote!(Locale::#locale => &Self::#locale));

        let from_locale = quote! {
            impl leptos_i18n::LocaleKeys for #type_ident {
                type Locale = Locale;
                fn from_locale(_locale: Locale) -> &'static Self {
                    match _locale {
                        #(
                            #from_locale_match_arms,
                        )*
                    }
                }
            }
        };

        let const_values = top_locales
            .iter()
            .map(|locale| quote!(pub const #locale: Self = Self::new(Locale::#locale);));

        let const_values = quote! {
            #(
                #[allow(non_upper_case_globals)]
                #const_values
            )*
        };

        (Some(from_locale), Some(const_values))
    } else {
        (None, None)
    };

    quote! {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[allow(non_camel_case_types, non_snake_case)]
        pub struct #type_ident {
            #(#string_fields,)*
            #(#builder_fields,)*
            #(#subkeys_fields,)*
        }

        impl #type_ident {

            #const_values

            pub const fn new(_locale: Locale) -> Self {
                match _locale {
                    #(
                        #new_match_arms,
                    )*
                }
            }
        }

        #from_locale

        #builder_module

        #subkeys_module
    }
}

fn create_namespace_mod_ident(namespace_ident: &syn::Ident) -> syn::Ident {
    format_ident!("ns_{}", namespace_ident)
}

fn create_namespaces_types(
    default_locale: &Key,
    i18n_keys_ident: &syn::Ident,
    namespaces: &[Namespace],
    top_locales: &HashSet<&Key>,
    keys: &HashMap<Rc<Key>, BuildersKeysInner>,
) -> TokenStream {
    let namespaces_ts = namespaces.iter().map(|namespace| {
        let namespace_ident = &namespace.key.ident;
        let namespace_module_ident = create_namespace_mod_ident(namespace_ident);
        let keys = keys.get(&namespace.key).unwrap();
        let type_impl = create_locale_type_inner(
            default_locale,
            namespace_ident,
            top_locales,
            &namespace.locales,
            &keys.0,
            true,
        );
        quote! {
            pub mod #namespace_module_ident {
                use super::Locale;

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
        quote!(#key: namespaces::#namespace_module_ident::#key::new(_locale))
    });

    let locales = &namespaces.iter().next().unwrap().locales;

    let const_values = locales.iter().map(|locale| {
        let locale_ident = &locale.name;
        quote!(pub const #locale_ident: Self = Self::new(Locale::#locale_ident);)
    });

    let from_locale_match_arms = locales.iter().map(|locale| {
        let locale_ident = &locale.name;
        quote!(Locale::#locale_ident => &Self::#locale_ident)
    });

    quote! {
        pub mod namespaces {
            use super::Locale;

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

            pub const fn new(_locale: Locale) -> Self {
                Self {
                    #(
                        #namespaces_fields_new,
                    )*
                }
            }
        }

        impl leptos_i18n::LocaleKeys for #i18n_keys_ident {
            type Locale = Locale;
            fn from_locale(_locale: Locale) -> &'static Self {
                match _locale {
                    #(
                        #from_locale_match_arms,
                    )*
                }
            }
        }
    }
}

fn create_locale_type(keys: BuildersKeys, cfg_file: &ConfigFile) -> TokenStream {
    let top_locales = cfg_file.locales.iter().map(Deref::deref).collect();
    let default_locale = cfg_file.default.as_ref();

    let i18n_keys_ident = format_ident!("I18nKeys");
    match keys {
        BuildersKeys::NameSpaces { namespaces, keys } => create_namespaces_types(
            default_locale,
            &i18n_keys_ident,
            namespaces,
            &top_locales,
            &keys,
        ),
        BuildersKeys::Locales { locales, keys } => create_locale_type_inner(
            default_locale,
            &i18n_keys_ident,
            &top_locales,
            locales,
            &keys.0,
            false,
        ),
    }
}
