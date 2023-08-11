use std::{
    collections::{HashMap, HashSet},
    fs::File,
    ops::Not,
    path::Path,
};

use crate::{
    cfg_file::{ConfigFile, RawConfigFile},
    error::{Error, InterpolateVariableNotMatching, Result},
    interpolate::{create_empty_type, Interpolation},
    key::Key,
    locale::{Locale, LocaleValue, RawLocale},
    value_kind::ValueKind,
};
use proc_macro2::TokenStream;
use quote::quote;

pub fn load_locales_inner(cfg_file_path: Option<impl AsRef<Path>>) -> Result<TokenStream> {
    let raw_cfg_file = RawConfigFile::new(cfg_file_path)?;
    let cfg_file = ConfigFile::new(&raw_cfg_file)?;

    let mut raw_locales: Vec<RawLocale> = Vec::with_capacity(cfg_file.locales.len());

    for locale in &cfg_file.locales {
        let path = format!("./locales/{}.json", locale.name);
        raw_locales.push(RawLocale::new(path, locale.name)?);
    }

    let locales = raw_locales
        .iter()
        .map(Locale::new)
        .collect::<Result<Vec<_>>>()?;

    let keys = Locale::check_locales(&locales)?;

    // let (keys, kinds) = check_keys(&cfg_file, &raw_locales)?;

    let locale_type = create_locale_type(&locales, &keys);
    let locale_variants = create_locales_enum(&cfg_file);
    let locales = create_locales_type(&cfg_file);

    Ok(quote! {
        use ::leptos as __leptos__;

        #locales

        #locale_variants

        #locale_type
    })
}

fn create_locales_enum(cfg_file: &ConfigFile) -> TokenStream {
    let ConfigFile { default, locales } = cfg_file;

    let as_str_match_arms = locales
        .iter()
        .map(|key| (&key.ident, key.name))
        .map(|(variant, locale)| quote!(LocaleEnum::#variant => #locale))
        .collect::<Vec<_>>();

    let from_str_match_arms = locales
        .iter()
        .map(|key| (&key.ident, key.name))
        .map(|(variant, locale)| quote!(#locale => Some(LocaleEnum::#variant)))
        .collect::<Vec<_>>();

    quote! {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
        #[allow(non_camel_case_types)]
        pub enum LocaleEnum {
            #(#locales,)*
        }

        impl Default for LocaleEnum {
            fn default() -> Self {
                LocaleEnum::#default
            }
        }

        impl ::leptos_i18n::LocaleVariant for LocaleEnum {
            fn as_str(&self) -> &'static str {
                match *self {
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

        impl ::leptos_i18n::Locales for Locales {
            type Variants = LocaleEnum;
            type LocaleKeys = I18nKeys;
        }
    }
}

fn create_locale_type(locales: &[Locale], keys: &HashMap<&Key, LocaleValue>) -> TokenStream {
    let string_keys = keys
        .iter()
        .filter(|(_, value)| matches!(value, LocaleValue::String))
        .map(|(key, _)| *key)
        .collect::<Vec<_>>();

    let string_fields = string_keys
        .iter()
        .copied()
        .map(|key| quote!(pub #key: &'static str))
        .collect::<Vec<_>>();

    let interpolations = keys
        .iter()
        .filter_map(|(key, value)| match value {
            LocaleValue::String => None,
            LocaleValue::Interpolate(keys) => Some((*key, Interpolation::new(key, keys, locales))),
        })
        .collect::<Vec<_>>();

    let interpolation_fields = interpolations.iter().map(|(key, inter)| {
        let inter_ident = &inter.default_generic_ident;
        quote!(pub #key: #inter_ident)
    });

    let filled_interpolate_fields: Vec<TokenStream> = interpolations
        .iter()
        .map(|(key, inter)| {
            let ident = &inter.ident;
            quote!(#key: #ident::new(_variant))
        })
        .collect();

    let from_variant_match_arms = locales.iter().map(|locale| {
        let filled_string_fields = locale.keys.iter().filter_map(|(key, value)| {
            let str_value = value.is_string()?;
            Some(quote!(#key: #str_value))
        });

        let ident = &locale.name.ident;
        quote! {
            LocaleEnum::#ident => I18nKeys {
                #(#filled_string_fields,)*
                #(#filled_interpolate_fields,)*
            }
        }
    });

    let interpolation_impls = interpolations.iter().map(|(_, inter)| &inter.imp);

    let empty_type = interpolations.is_empty().not().then(create_empty_type);

    quote! {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[allow(non_snake_case)]
        pub struct I18nKeys {
            #(#string_fields,)*
            #(#interpolation_fields,)*
        }

        impl ::leptos_i18n::LocaleKeys for I18nKeys {
            type Locales = Locales;
            fn from_variant(_variant: LocaleEnum) -> Self {
                match _variant {
                    #(
                        #from_variant_match_arms,
                    )*
                }
            }
        }

        #empty_type

        #(
            #interpolation_impls
        )*
    }
}

// type CheckResult<'a> = (
//     Vec<(&'a str, syn::Ident)>,
//     Vec<HashMap<&'a str, ValueKind<'a>>>,
// );

// fn check_keys<'a>(
//     cfg_file: &ConfigFile,
//     raw_locales: &'a [HashMap<String, String>],
// ) -> Result<CheckResult<'a>> {
//     // locales is non empty, as default need to exist and locales to contain default, so len >= 1
//     let first_locale = raw_locales.first().unwrap();
//     let first_locale_name = cfg_file.locales.first().unwrap();

//     let keys: HashSet<&str> = first_locale.keys().map(String::as_str).collect();

//     // this check if all locales have the same keys, no more no less.
//     for (raw_locale, locale_name) in raw_locales.iter().zip(&cfg_file.locales).skip(1) {
//         let mut count = 0;
//         let mut missing_keys = Vec::new();
//         // check if all keys in first locale are in this locale
//         for key in raw_locale.keys() {
//             count += 1;
//             if !keys.contains(key.as_str()) {
//                 missing_keys.push(key.to_owned());
//             }
//         }
//         if !missing_keys.is_empty() {
//             // missing key in first locale
//             return Err(Error::MissingKeysInLocale {
//                 keys: missing_keys,
//                 locale: first_locale_name.to_owned(),
//             });
//         }
//         // missing key in current locale
//         if count < keys.len() {
//             missing_keys.reserve(keys.len() - count);
//             for key in keys {
//                 if !raw_locale.contains_key(key) {
//                     missing_keys.push(key.to_owned());
//                 }
//             }
//             return Err(Error::MissingKeysInLocale {
//                 keys: missing_keys,
//                 locale: locale_name.to_owned(),
//             });
//         }
//     }

//     let kinds = raw_locales
//         .iter()
//         .map(|locale| {
//             locale
//                 .iter()
//                 .map(|(key, value)| parse_value(value).map(|kind| (key.as_str(), kind)))
//                 .collect::<Result<HashMap<_, _>>>()
//         })
//         .collect::<Result<Vec<_>>>()?;

//     let keys = keys
//         .iter()
//         .map(|key| new_ident(key).map(|ident| (*key, ident)))
//         .collect::<Result<_>>()?;

//     Ok((keys, kinds))
// }

// fn check_if_key_valid_for_field_name(key: &str) -> bool {
//     syn::parse_str::<syn::Ident>(key).is_ok()
// }

// fn get_locale<T: AsRef<Path>>(path: T, locale: &str) -> Result<HashMap<String, String>> {
//     let locale_file =
//         File::open(path).map_err(|err| Error::LocaleFileNotFound(locale.to_string(), err))?;

//     serde_json::from_reader(locale_file)
//         .map_err(|err| Error::LocaleFileDeser(locale.to_string(), err))
// }

// fn new_ident<T: AsRef<str>>(ident_str: T) -> Result<syn::Ident> {
//     syn::parse_str::<syn::Ident>(ident_str.as_ref())
//         .map_err(|_| Error::InvalidKey(ident_str.as_ref().to_string()))
// }

// fn compare_kind(
//     key: &str,
//     locale1: &str,
//     kind1: &ValueKind,
//     locale2: &str,
//     kind2: &ValueKind,
// ) -> Result<()> {
//     match (kind1, kind2) {
//         (ValueKind::String { .. }, ValueKind::String { .. }) => Ok(()),
//         (ValueKind::String { .. }, ValueKind::Interpolate { .. }) => {
//             Err(Error::MismatchLocaleKeyKind {
//                 key: key.to_string(),
//                 locale_str: locale1.to_string(),
//                 locale_inter: locale2.to_string(),
//             })
//         }
//         (ValueKind::Interpolate { .. }, ValueKind::String { .. }) => {
//             Err(Error::MismatchLocaleKeyKind {
//                 key: key.to_string(),
//                 locale_str: locale2.to_string(),
//                 locale_inter: locale1.to_string(),
//             })
//         }
//         (ValueKind::Interpolate { keys: k1, .. }, ValueKind::Interpolate { keys: k2, .. }) => {
//             fn to_set<'a>(keys: &[(&'a str, syn::Ident)]) -> HashSet<&'a str> {
//                 keys.iter().map(|(key, _)| key).copied().collect()
//             }
//             let k1 = to_set(k1);
//             let k2 = to_set(k2);

//             if k1 != k2 {
//                 let keys1 = k1.into_iter().map(String::from).collect();
//                 let keys2 = k2.into_iter().map(String::from).collect();
//                 return Err(Error::InterpolateVariableNotMatching(
//                     InterpolateVariableNotMatching {
//                         key: key.to_string(),
//                         locale1: locale1.to_string(),
//                         locale2: locale2.to_string(),
//                         keys1,
//                         keys2,
//                     }
//                     .into(),
//                 ));
//             }

//             Ok(())
//         }
//     }
// }

// enum ValueKind<'a> {
//     String(&'a str),
//     Interpolate {
//         keys: Vec<(&'a str, syn::Ident)>,
//         format_string: String,
//     },
// }

// fn parse_value<'a>(value: &'a str) -> Result<ValueKind<'a>> {
//     let Some((first, rest)) = value.split_once("{{") else {
//         return Ok(ValueKind::String(value));
//     };

//     let Some((ident, rest)) = rest.split_once("}}") else {
//         return Ok(ValueKind::String(value));
//     };

//     let (keys, format_string) = parse_rest(first, ident, rest)?;

//     Ok(ValueKind::Interpolate {
//         keys,
//         format_string,
//     })
// }

// fn parse_rest<'a>(
//     first: &'a str,
//     ident: &'a str,
//     rest: &'a str,
// ) -> Result<(Vec<(&'a str, syn::Ident)>, String)> {
//     fn replace(s: &str) -> String {
//         s.replace('{', "{{").replace('}', "}}")
//     }

//     fn make_ident(s: &str) -> Result<(&str, syn::Ident)> {
//         let s = s.trim();
//         let ident = new_ident(s)?;
//         Ok((s, ident))
//     }

//     let mut format_string = replace(first);
//     let mut keys = vec![make_ident(ident)?];

//     let mut current_rest = rest;

//     loop {
//         if let Some((first, rest)) = current_rest.split_once("{{") {
//             format_string.push_str(&replace(first));
//             if let Some((ident, rest)) = rest.split_once("}}") {
//                 keys.push(make_ident(ident)?);
//                 format_string.push_str("{}");
//                 current_rest = rest;
//             } else {
//                 format_string.push_str(&replace(rest));
//                 break;
//             }
//         } else {
//             format_string.push_str(&replace(current_rest));
//             break;
//         }
//     }

//     Ok((keys, format_string))
// }
