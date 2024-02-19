use std::{
    collections::HashSet,
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
pub mod tracking;
pub mod warning;

use cfg_file::ConfigFile;
use error::{Error, Result};
use interpolate::{create_empty_type, Interpolation};
use key::Key;
use locale::{Locale, LocaleValue};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::load_locales::parsed_value::ParsedValue;

use self::{
    locale::{BuildersKeys, BuildersKeysInner, LocalesOrNamespaces, Namespace},
    warning::generate_warnings,
};

/// Steps:
///
/// 1: Locate and parse the manifest (`ConfigFile::new`)
/// 2: parse each locales/namespaces files (`LocalesOrNamespaces::new`)
/// 3: Resolve foreign keys (`ParsedValue::resolve_foreign_keys`)
/// 4: check the locales: (`Locale::check_locales`)
/// 4.1: get interpolations keys of the default, meaning all variables/components/plurals of the default locale (`Locale::make_builder_keys`)
/// 4.2: in the process reduce all values and check for default in the default locale
/// 4.3: then merge all other locales in the default locale keys, reducing all values in the process (`Locale::merge`)
/// 4.4: discard any surplus key and emit a warning
/// 5: generate code (and warnings)
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

    let file_tracking = tracking::generate_file_tracking();

    let mut macros_reexport = vec![quote!(t), quote!(td)];
    if cfg!(feature = "interpolate_display") {
        macros_reexport.extend([
            quote!(t_string),
            quote!(t_display),
            quote!(td_string),
            quote!(td_display),
        ]);
    }

    let island_or_component = if cfg!(feature = "experimental-islands") {
        macros_reexport.push(quote!(ti));
        quote!(island)
    } else {
        quote!(component)
    };

    let macros_reexport = quote!(pub use leptos_i18n::{#(#macros_reexport,)*};);

    Ok(quote! {
        pub mod i18n {
            #file_tracking

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

            mod provider {
                #[leptos::#island_or_component]
                pub fn i18n_context_provider(children: leptos::Children) -> impl leptos::IntoView {
                    super::provide_i18n_context();
                    children()
                }
            }

            #[allow(non_snake_case)]
            pub use provider::i18n_context_provider as I18nContextProvider;

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

#[allow(clippy::too_many_arguments)]
fn create_locale_type_inner(
    default_locale: &Key,
    type_ident: &Ident,
    top_locales: &HashSet<&Key>,
    locales: &[Locale],
    keys: &[(Rc<Key>, LocaleValue)],
    is_namespace: bool,
    parent_ident: Option<&Ident>,
    original_name: &Ident,
) -> TokenStream {
    let default_match = get_default_match(default_locale, top_locales, locales);

    let string_keys = keys
        .iter()
        .filter(|(_, value)| matches!(value, LocaleValue::Value(None)))
        .map(|(key, _)| key)
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
            Some(type_ident),
            sk.original_key,
        );
        quote! {
            pub mod #subkey_mod_ident {
                use super::Locale;

                #subkey_impl
            }
        }
    });

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
            LocaleValue::Value(Some(keys)) => Some((
                key,
                Interpolation::new(key, type_ident, keys, locales, &default_match),
            )),
        })
        .collect::<Vec<_>>();

    let default_locale = locales.first().unwrap();

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

    let translation_types = locales.iter().map(|locale| {
        let (fields, constructors): (Vec<_>, Vec<_>) = keys
            .iter()
            .filter_map(|(key, _)| {
                let locale_value = keys
                    .iter()
                    .find_map(|(k, value)| (k == key).then_some(value));
                let ts = match locale.get(key).zip(locale_value)? {
                    (ParsedValue::String(s), LocaleValue::Value(None)) => {
                        let len = s.len();
                        let field = quote!(pub #key: leptos_i18n::__private::SizedString<#len>);
                        let constructor =
                            quote!(#key: leptos_i18n::__private::SizedString::new(#s));
                        (field, constructor)
                    }
                    (ParsedValue::Subkeys(_), _) => {
                        let mod_ident = format_ident!("sk_{}", key.ident);
                        let type_key =
                            format_ident!("{}_subkeys_{}", key.ident, locale.top_locale_name.ident);
                        let field = quote!(pub #key: subkeys::#mod_ident::#type_key);
                        let constructor = quote!(#key: subkeys::#mod_ident::#type_key::new());
                        (field, constructor)
                    }
                    (ParsedValue::Default, _) => {
                        // defaulted builder
                        let builder_name = format_ident!(
                            "{}_builder_{}",
                            key.ident,
                            default_locale.top_locale_name.ident
                        );
                        let field = quote!(pub #key: builders::#builder_name);
                        let constructor = quote!(#key: builders::#builder_name::new());
                        (field, constructor)
                    }
                    _ => {
                        // builder
                        let builder_name =
                            format_ident!("{}_builder_{}", key.ident, locale.top_locale_name.ident);
                        let field = quote!(pub #key: builders::#builder_name);
                        let constructor = quote!(#key: builders::#builder_name::new());
                        (field, constructor)
                    }
                };
                Some(ts)
            })
            .unzip();

        let translation_type_name =
            format_ident!("{}_{}", type_ident, locale.top_locale_name.ident);

        let get_method = match parent_ident {
            Some(parent_ident) => {
                let parent_ident =
                    format_ident!("{}_{}", parent_ident, locale.top_locale_name.ident);
                quote! {
                    pub fn get() -> &'static Self {
                        &super::super::#parent_ident::get().#original_name
                    }
                }
            }
            None => {
                quote! {
                    const THIS: Self = Self::new();
                    pub fn get() -> &'static Self {
                        &Self::THIS
                    }
                }
            }
        };

        quote! {
            #[allow(non_camel_case_types, non_snake_case)]
            pub struct #translation_type_name {
                #(
                    #fields,
                )*
            }

            impl #translation_type_name {
                #get_method

                pub const fn new() -> Self {
                    #translation_type_name {
                        #(
                            #constructors,
                        )*
                    }
                }
            }
        }
    });

    let from_locale = if !is_namespace {
        let from_locale = quote! {
            impl leptos_i18n::LocaleKeys for #type_ident {
                type Locale = Locale;
                fn from_locale(locale: Locale) -> Self {
                    #type_ident(locale)
                }
            }
        };
        Some(from_locale)
    } else {
        None
    };

    let string_accessors = string_keys
        .iter()
        .map(|key| create_string_accessor(type_ident, key, locales));

    let builder_accessors = builders
        .iter()
        .map(|(key, builder)| create_builder_accessor(key, builder));

    let subkeys_accessors = subkeys.iter().map(create_subkeys_accessor);

    quote! {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[allow(non_camel_case_types, non_snake_case)]
        pub struct #type_ident(Locale);

        impl #type_ident {
            #(
                #string_accessors
            )*

            #(
                #builder_accessors
            )*

            #(
                #subkeys_accessors
            )*

            pub fn new(locale: Locale) -> Self {
                #type_ident(locale)
            }
        }


        #(
            #translation_types
        )*

        #from_locale

        #builder_module

        #subkeys_module
    }
}

fn create_string_accessor(type_ident: &Ident, key: &Key, locales: &[Locale]) -> TokenStream {
    let translation_type = locales
        .iter()
        .map(|locale| format_ident!("{}_{}", type_ident, locale.top_locale_name.ident));
    let locales = locales.iter().map(|locale| &locale.top_locale_name.ident);
    quote! {
        #[allow(non_snake_case)]
        pub fn #key(self) -> &'static str {
            match self.0 {
                #(Locale::#locales => &#translation_type::get().#key,)*
            }
        }
    }
}

fn create_builder_accessor(key: &Key, builder: &Interpolation) -> TokenStream {
    let builder_ident = &builder.ident;
    let generics = &builder.default_generic_ident;
    quote! {
        #[allow(non_snake_case)]
        pub fn #key(self) -> builders::#generics {
            builders::#builder_ident::new(self.0)
        }
    }
}

fn create_subkeys_accessor(sk: &Subkeys) -> TokenStream {
    let original_key = &sk.original_key;
    let key = &sk.key;
    let mod_ident = &sk.mod_key;
    quote! {
        #[allow(non_snake_case)]
        pub fn #original_key(self) -> subkeys::#mod_ident::#key {
            subkeys::#mod_ident::#key::new(self.0)
        }
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
    keys: &[(Rc<Key>, BuildersKeysInner)],
) -> TokenStream {
    let namespaces_ts = namespaces.iter().map(|namespace| {
        let namespace_ident = &namespace.key.ident;
        let namespace_module_ident = create_namespace_mod_ident(namespace_ident);
        let keys = keys
            .iter()
            .find_map(|(key, value)| (key == &namespace.key).then_some(value))
            .unwrap();
        let type_impl = create_locale_type_inner(
            default_locale,
            namespace_ident,
            top_locales,
            &namespace.locales,
            &keys.0,
            true,
            None,
            namespace_ident,
        );
        quote! {
            pub mod #namespace_module_ident {
                use super::Locale;

                #type_impl
            }
        }
    });

    let translations_accessors = namespaces.iter().map(|namespace| {
        let key = &namespace.key;
        let namespace_module_ident = create_namespace_mod_ident(&key.ident);
        quote! {
            #[allow(non_snake_case)]
            pub fn #key(self) -> namespaces::#namespace_module_ident::#key {
                namespaces::#namespace_module_ident::#key::new(self.0)
            }
        }
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
        pub struct #i18n_keys_ident(Locale);

        impl #i18n_keys_ident {
            pub const fn new(locale: Locale) -> #i18n_keys_ident {
                #i18n_keys_ident(locale)
            }

            #(
                #translations_accessors
            )*
        }

        impl leptos_i18n::LocaleKeys for #i18n_keys_ident {
            type Locale = Locale;
            fn from_locale(locale: Locale) -> Self {
                Self::new(locale)
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
            None,
            &i18n_keys_ident,
        ),
    }
}
