use std::{
    collections::{HashMap, HashSet},
    ops::{Deref, Not},
    path::PathBuf,
    rc::Rc,
};

pub mod cfg_file;
pub mod declare_locales;
pub mod error;
pub mod interpolate;
pub mod locale;
pub mod parsed_value;
pub mod ranges;
pub mod tracking;
pub mod warning;

pub mod plurals;

use crate::utils::key::{Key, KeyPath};
use cfg_file::ConfigFile;
use error::{Error, Result};
use interpolate::Interpolation;
use locale::{Locale, LocaleValue};
use parsed_value::InterpolOrLit;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};

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
/// 4.1: get interpolations keys of the default, meaning all variables/components/ranges of the default locale (`Locale::make_builder_keys`)
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

    let crate_path = syn::Path::from(syn::Ident::new("leptos_i18n", Span::call_site()));

    load_locales_inner(&crate_path, &cfg_file, &mut locales)
}

fn load_locales_inner(
    crate_path: &syn::Path,
    cfg_file: &ConfigFile,
    locales: &mut LocalesOrNamespaces,
) -> Result<TokenStream> {
    locales.merge_plurals()?;

    ParsedValue::resolve_foreign_keys(locales, &cfg_file.default)?;

    let keys = Locale::check_locales(locales)?;

    let enum_ident = syn::Ident::new("Locale", Span::call_site());
    let keys_ident = syn::Ident::new("I18nKeys", Span::call_site());

    let locale_type = create_locale_type(keys, cfg_file, &keys_ident, &enum_ident);
    let locale_enum = create_locales_enum(
        &enum_ident,
        &keys_ident,
        &cfg_file.default,
        &cfg_file.locales,
    );

    let warnings = generate_warnings();

    let file_tracking = tracking::generate_file_tracking();

    let mut macros_reexport = vec![
        quote!(t),
        quote!(td),
        quote!(tu),
        quote!(use_i18n_scoped),
        quote!(scope_i18n),
        quote!(scope_locale),
    ];
    if cfg!(feature = "interpolate_display") {
        macros_reexport.extend([
            quote!(t_string),
            quote!(tu_string),
            quote!(t_display),
            quote!(tu_display),
            quote!(td_string),
            quote!(td_display),
        ]);
    }

    let provider = if cfg!(feature = "experimental-islands") {
        macros_reexport.push(quote!(ti));
        quote! {
            #[leptos::island]
            #[allow(non_snake_case)]
            pub fn I18nContextProvider(
                #[prop(optional)]
                set_lang_attr_on_html: bool,
                #[prop(optional)]
                enable_cookie: Option<bool>,
                #[prop(optional)]
                cookie_name: Option<&'static str>,
                children: leptos::Children
            ) -> impl leptos::IntoView {
                l_i18n_crate::context::provide_i18n_context_component_inner::<#enum_ident>(
                    set_lang_attr_on_html,
                    enable_cookie,
                    cookie_name,
                    None,
                    children
                )
            }
        }
    } else {
        quote! {
            #[leptos::component]
            #[allow(non_snake_case)]
            pub fn I18nContextProvider(
                #[prop(optional)]
                set_lang_attr_on_html: bool,
                #[prop(optional)]
                enable_cookie: Option<bool>,
                #[prop(optional)]
                cookie_name: Option<&'static str>,
                #[prop(optional)]
                cookie_options: Option<l_i18n_crate::context::CookieOptions<#enum_ident>>,
                children: leptos::Children
            ) -> impl leptos::IntoView {
                l_i18n_crate::context::provide_i18n_context_component_inner::<#enum_ident>(
                    set_lang_attr_on_html,
                    enable_cookie,
                    cookie_name,
                    cookie_options,
                    children
                )
            }
        }
    };

    let macros_reexport = quote!(pub use #crate_path::{#(#macros_reexport,)*};);

    Ok(quote! {
        pub mod i18n {
            use #crate_path as l_i18n_crate;

            #file_tracking

            #locale_enum

            #locale_type

            #[inline]
            pub fn use_i18n() -> l_i18n_crate::I18nContext<#enum_ident> {
                l_i18n_crate::use_i18n_context()
            }

            #[deprecated(
                note = "It is now preferred to use the <I18nContextProvider> component"
            )]
            pub fn provide_i18n_context() -> l_i18n_crate::I18nContext<#enum_ident> {
                l_i18n_crate::context::provide_i18n_context_with_options_inner(None, None, None)
            }

            mod provider {
                use super::{l_i18n_crate, #enum_ident};

                #provider
            }

            mod routing {
                use super::{l_i18n_crate, #enum_ident};
                use l_i18n_crate::__private as l_i18n_crate_priv;
                use l_i18n_crate_priv::leptos_router;
                #[leptos::component(transparent)]
                #[allow(non_snake_case)]
                pub fn I18nRoute<E, F>(
                    /// The base path of this application.
                    /// If you setup your i18n route such that the path is `/foo/:locale/bar`,
                    /// the expected base path is `/foo/`.
                    /// Defaults to `"/"``.
                    #[prop(default = "/")]
                    base_path: &'static str,
                    /// The view that should be shown when this route is matched. This can be any function
                    /// that returns a type that implements [`IntoView`] (like `|| view! { <p>"Show this"</p> })`
                    /// or `|| view! { <MyComponent/>` } or even, for a component with no props, `MyComponent`).
                    /// If you use nested routes you can just set it to `view=Outlet`
                    view: F,
                    /// The mode that this route prefers during server-side rendering. Defaults to out-of-order streaming.
                    #[prop(optional)]
                    ssr: leptos_router::SsrMode,
                    /// The HTTP methods that this route can handle (defaults to only `GET`).
                    #[prop(default = &[leptos_router::Method::Get])]
                    methods: &'static [leptos_router::Method],
                    /// A data-loading function that will be called when the route is matched. Its results can be
                    /// accessed with [`use_route_data`](crate::use_route_data).
                    #[prop(optional, into)]
                    data: Option<leptos_router::Loader>,
                    /// How this route should handle trailing slashes in its path.
                    /// Overrides any setting applied to [`crate::components::Router`].
                    /// Serves as a default for any inner Routes.
                    #[prop(optional)]
                    trailing_slash: Option<leptos_router::TrailingSlash>,
                    /// `children` may be empty or include nested routes.
                    #[prop(optional)]
                    children: Option<leptos::Children>,
                ) -> impl leptos::IntoView
                    where E: leptos::IntoView,
                    F: Fn() -> E + 'static
                {
                    l_i18n_crate_priv::i18n_routing::<#enum_ident, E, F>(base_path, children, ssr, methods, data, trailing_slash, view)
                }
            }

            pub use provider::I18nContextProvider;
            pub use routing::I18nRoute;

            #macros_reexport

            #warnings
        }
    })
}

fn create_locales_enum(
    enum_ident: &syn::Ident,
    keys_ident: &syn::Ident,
    default: &Key,
    locales: &[Rc<Key>],
) -> TokenStream {
    let as_str_match_arms = locales
        .iter()
        .map(|key| (&key.ident, &key.name))
        .map(|(variant, locale)| quote!(#enum_ident::#variant => #locale))
        .collect::<Vec<_>>();

    let from_str_match_arms = locales
        .iter()
        .map(|key| (&key.ident, &key.name))
        .map(|(variant, locale)| quote!(#locale => Ok(#enum_ident::#variant)))
        .collect::<Vec<_>>();

    let constant_names_ident = locales
        .iter()
        .map(|key| {
            (
                key,
                format_ident!("{}_LANGID", key.name.to_uppercase().replace('-', "_")),
            )
        })
        .collect::<Vec<_>>();

    let const_icu_locales = constant_names_ident
        .iter()
        .map(|(key, ident)| {
            let locale = &key.name;
            quote!(const #ident: &'static l_i18n_crate::__private::locid::Locale = &l_i18n_crate::__private::locid::locale!(#locale);)
        })
        .collect::<Vec<_>>();

    let as_icu_locale_match_arms = constant_names_ident
        .iter()
        .map(|(variant, constant)| quote!(#enum_ident::#variant => #constant))
        .collect::<Vec<_>>();

    quote! {
        use l_i18n_crate::reexports::serde as serde;

        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[allow(non_camel_case_types)]
        pub enum #enum_ident {
            #(#locales,)*
        }

        impl Default for #enum_ident {
            fn default() -> Self {
                #enum_ident::#default
            }
        }

        impl l_i18n_crate::reexports::serde::Serialize for #enum_ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: l_i18n_crate::reexports::serde::Serializer,
            {
                l_i18n_crate::reexports::serde::Serialize::serialize(l_i18n_crate::Locale::as_str(*self), serializer)
            }
        }

        impl<'de> l_i18n_crate::reexports::serde::Deserialize<'de> for #enum_ident {
            fn deserialize<D>(deserializer: D) -> Result<#enum_ident, D::Error>
            where
                D: l_i18n_crate::reexports::serde::de::Deserializer<'de>,
            {
                l_i18n_crate::reexports::serde::de::Deserializer::deserialize_str(deserializer, l_i18n_crate::__private::LocaleVisitor::<#enum_ident>::new())
            }
        }

        impl l_i18n_crate::Locale for #enum_ident {
            type Keys = #keys_ident;

            fn as_str(self) -> &'static str {
                let s = match self {
                    #(#as_str_match_arms,)*
                };
                l_i18n_crate::reexports::wasm_bindgen::intern(s)
            }

            fn as_icu_locale(self) -> &'static l_i18n_crate::__private::locid::Locale {
                #(
                    #const_icu_locales;
                )*
                match self {
                    #(#as_icu_locale_match_arms,)*
                }
            }

            fn get_all() -> &'static [Self] {
                &[#(#enum_ident::#locales,)*]
            }

            fn to_base_locale(self) -> Self {
                self
            }

            fn from_base_locale(locale: Self) -> Self {
                locale
            }
        }

        impl core::str::FromStr for #enum_ident {
            type Err = ();

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.trim() {
                    #(#from_str_match_arms,)*
                    _ => Err(())
                }
            }
        }

        impl core::convert::AsRef<l_i18n_crate::__private::locid::LanguageIdentifier> for #enum_ident {
            fn as_ref(&self) -> &l_i18n_crate::__private::locid::LanguageIdentifier {
                l_i18n_crate::Locale::as_langid(*self)
            }
        }

        impl core::convert::AsRef<l_i18n_crate::__private::locid::Locale> for #enum_ident {
            fn as_ref(&self) -> &l_i18n_crate::__private::locid::Locale {
                l_i18n_crate::Locale::as_icu_locale(*self)
            }
        }

        impl core::convert::AsRef<str> for #enum_ident {
            fn as_ref(&self) -> &str {
                l_i18n_crate::Locale::as_str(*self)
            }
        }

        impl core::fmt::Display for #enum_ident {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Display::fmt(l_i18n_crate::Locale::as_str(*self), f)
            }
        }
    }
}

struct Subkeys<'a> {
    original_key: Rc<Key>,
    key: syn::Ident,
    mod_key: syn::Ident,
    locales: &'a [Locale],
    keys: &'a BuildersKeysInner,
}

impl<'a> Subkeys<'a> {
    pub fn new(key: Rc<Key>, locales: &'a [Locale], keys: &'a BuildersKeysInner) -> Self {
        let mod_key = format_ident!("sk_{}", key.ident);
        let new_key = format_ident!("{}_subkeys", key.ident);
        Subkeys {
            original_key: key,
            key: new_key,
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
    enum_ident: &syn::Ident,
) -> TokenStream {
    let current_keys = locales
        .iter()
        .map(|locale| &*locale.top_locale_name)
        .collect();
    let missing_keys = top_locales.difference(&current_keys);
    quote!(#enum_ident::#default_locale #(| #enum_ident::#missing_keys)*)
}

#[allow(clippy::too_many_arguments)]
fn create_locale_type_inner(
    default_locale: &Key,
    type_ident: &syn::Ident,
    parent_ident: Option<&syn::Ident>,
    enum_ident: &syn::Ident,
    top_locales: &HashSet<&Key>,
    locales: &[Locale],
    keys: &HashMap<Rc<Key>, LocaleValue>,
    key_path: &mut KeyPath,
    original_ident: &syn::Ident,
) -> TokenStream {
    let default_match = get_default_match(default_locale, top_locales, locales, enum_ident);

    let literal_keys = keys
        .iter()
        .filter_map(|(key, value)| match value {
            LocaleValue::Value(InterpolOrLit::Lit(t)) => Some((key.clone(), t)),
            _ => None,
        })
        .collect::<Vec<_>>();

    let literal_fields = literal_keys
        .iter()
        .map(|(key, literal_type)| {
            if cfg!(feature = "show_keys_only") {
                quote!(pub #key: &'static str)
            } else {
                quote!(pub #key: #literal_type)
            }
        })
        .collect::<Vec<_>>();

    let subkeys = keys
        .iter()
        .filter_map(|(key, value)| match value {
            LocaleValue::Subkeys { locales, keys } => {
                Some(Subkeys::new(key.clone(), locales, keys))
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    let subkeys_ts = subkeys.iter().map(|sk| {
        let subkey_mod_ident = &sk.mod_key;
        key_path.push_key(sk.original_key.clone());
        let subkey_impl = create_locale_type_inner(
            default_locale,
            &sk.key,
            Some(type_ident),
            enum_ident,
            top_locales,
            sk.locales,
            &sk.keys.0,
            key_path,
            &sk.original_key.ident,
        );
        key_path.pop_key();
        quote! {
            pub mod #subkey_mod_ident {
                use super::{#enum_ident, l_i18n_crate};

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
                use super::{#enum_ident, l_i18n_crate};

                #(
                    #subkeys_ts
                )*
            }
        }
    });

    let builders = keys
        .iter()
        .filter_map(|(key, value)| match value {
            LocaleValue::Value(InterpolOrLit::Interpol(keys)) => Some((
                key,
                Interpolation::new(key, enum_ident, keys, locales, &default_match, key_path),
            )),
            _ => None,
        })
        .collect::<Vec<_>>();

    let builder_fields = builders.iter().map(|(key, inter)| {
        let inter_ident = &inter.ident;
        quote!(pub #key: builders::#inter_ident)
    });

    let init_builder_fields: Vec<TokenStream> = builders
        .iter()
        .map(|(key, inter)| {
            let ident = &inter.ident;
            quote!(#key: builders::#ident::new(_locale))
        })
        .collect();

    let default_locale = locales
        .first()
        .expect("There should be at least one Locale");

    let new_match_arms = locales.iter().enumerate().map(|(i, locale)| {
        let filled_lit_fields = literal_keys.iter().filter_map(|(key, _)| {
            if cfg!(feature = "show_keys_only") {
                let key_str = key_path.to_string_with_key(key);
                return Some(quote!(#key: #key_str));
            }
            match locale.keys.get(key) {
                Some(ParsedValue::Literal(lit)) => Some(quote!(#key: #lit)),
                _ => {
                    let lit = default_locale
                        .keys
                        .get(key)
                        .and_then(ParsedValue::is_literal)?;
                    Some(quote!(#key: #lit))
                }
            }
        });

        let ident = &locale.top_locale_name;
        let pattern = (i != 0).then(|| quote!(#enum_ident::#ident));
        let pattern = pattern.as_ref().unwrap_or(&default_match);
        quote! {
            #pattern => #type_ident {
                #(#filled_lit_fields,)*
                #(#init_builder_fields,)*
                #(#subkeys_field_new,)*
            }
        }
    });

    let builder_impls = builders.iter().map(|(_, inter)| &inter.imp);

    let builder_module = builders.is_empty().not().then(move || {
        quote! {
            #[doc(hidden)]
            pub mod builders {
                use super::{#enum_ident, l_i18n_crate};

                #(
                    #builder_impls
                )*
            }
        }
    });

    let locale_keys_impl = if let Some(parent_ident) = parent_ident {
        quote! {
            impl l_i18n_crate::LocaleKeys for #type_ident {
                type Locale = #enum_ident;
                fn from_locale(_locale: #enum_ident) -> &'static Self {
                    &<super::super::#parent_ident as l_i18n_crate::LocaleKeys>::from_locale(_locale).#original_ident
                }
            }
        }
    } else {
        let from_locale_match_arms = top_locales
            .iter()
            .map(|locale| quote!(#enum_ident::#locale => &Self::#locale));
        quote! {
            impl l_i18n_crate::LocaleKeys for #type_ident {
                type Locale = #enum_ident;
                fn from_locale(_locale: #enum_ident) -> &'static Self {
                    match _locale {
                        #(
                            #from_locale_match_arms,
                        )*
                    }
                }
            }
        }
    };

    let const_values = if parent_ident.is_none() {
        let const_values = top_locales
            .iter()
            .map(|locale| quote!(pub const #locale: Self = Self::new(#enum_ident::#locale);));

        let const_values = quote! {
            #(
                #[allow(non_upper_case_globals)]
                #const_values
            )*
        };

        Some(const_values)
    } else {
        None
    };

    quote! {
        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[allow(non_camel_case_types, non_snake_case)]
        pub struct #type_ident {
            #(#literal_fields,)*
            #(#builder_fields,)*
            #(#subkeys_fields,)*
        }

        impl #type_ident {

            #const_values

            pub const fn new(_locale: #enum_ident) -> Self {
                match _locale {
                    #(
                        #new_match_arms,
                    )*
                }
            }
        }

        #locale_keys_impl

        #builder_module

        #subkeys_module

    }
}

fn create_namespace_mod_ident(namespace_ident: &syn::Ident) -> syn::Ident {
    format_ident!("ns_{}", namespace_ident)
}

fn create_namespaces_types(
    default_locale: &Key,
    keys_ident: &syn::Ident,
    enum_ident: &syn::Ident,
    namespaces: &[Namespace],
    top_locales: &HashSet<&Key>,
    keys: &HashMap<Rc<Key>, BuildersKeysInner>,
) -> TokenStream {
    let namespaces = namespaces
        .iter()
        .map(|ns| {
            let namespace_module_ident = create_namespace_mod_ident(&ns.key.ident);
            (ns, namespace_module_ident)
        })
        .collect::<Vec<_>>();

    let namespaces_ts = namespaces
        .iter()
        .map(|(namespace, namespace_module_ident)| {
            let keys = keys
                .get(&namespace.key)
                .expect("There should be a namspace of that name.");
            let mut key_path = KeyPath::new(Some(namespace.key.clone()));
            let type_impl = create_locale_type_inner(
                default_locale,
                &namespace.key.ident,
                Some(keys_ident),
                enum_ident,
                top_locales,
                &namespace.locales,
                &keys.0,
                &mut key_path,
                &namespace.key.ident,
            );

            quote! {
                pub mod #namespace_module_ident {
                    use super::{#enum_ident, l_i18n_crate};

                    #type_impl
                }
            }
        });

    let namespaces_fields = namespaces
        .iter()
        .map(|(namespace, namespace_module_ident)| {
            let key = &namespace.key;
            quote!(pub #key: namespaces::#namespace_module_ident::#key)
        });

    let namespaces_fields_new = namespaces
        .iter()
        .map(|(namespace, namespace_module_ident)| {
            let key = &namespace.key;
            quote!(#key: namespaces::#namespace_module_ident::#key::new(_locale))
        });

    let locales = &namespaces
        .first()
        .expect("There should be at least one namespace.")
        .0
        .locales;

    let const_values = locales.iter().map(|locale| {
        let locale_ident = &locale.name;
        quote!(pub const #locale_ident: Self = Self::new(#enum_ident::#locale_ident);)
    });

    let from_locale_match_arms = locales.iter().map(|locale| {
        let locale_ident = &locale.name;
        quote!(#enum_ident::#locale_ident => &Self::#locale_ident)
    });

    quote! {
        #[doc(hidden)]
        pub mod namespaces {
            use super::{#enum_ident, l_i18n_crate};

            #(
                #namespaces_ts
            )*

        }

        #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
        #[allow(non_snake_case)]
        pub struct #keys_ident {
            #(#namespaces_fields,)*
        }

        impl #keys_ident {
            #(
                #[allow(non_upper_case_globals)]
                #const_values
            )*

            pub const fn new(_locale: #enum_ident) -> Self {
                Self {
                    #(
                        #namespaces_fields_new,
                    )*
                }
            }
        }

        impl l_i18n_crate::LocaleKeys for #keys_ident {
            type Locale = #enum_ident;
            fn from_locale(_locale: #enum_ident) -> &'static Self {
                match _locale {
                    #(
                        #from_locale_match_arms,
                    )*
                }
            }
        }
    }
}

fn create_locale_type(
    keys: BuildersKeys,
    cfg_file: &ConfigFile,
    keys_ident: &syn::Ident,
    enum_ident: &syn::Ident,
) -> TokenStream {
    let top_locales = cfg_file.locales.iter().map(Deref::deref).collect();
    let default_locale = cfg_file.default.as_ref();
    match keys {
        BuildersKeys::NameSpaces { namespaces, keys } => create_namespaces_types(
            default_locale,
            keys_ident,
            enum_ident,
            namespaces,
            &top_locales,
            &keys,
        ),
        BuildersKeys::Locales { locales, keys } => create_locale_type_inner(
            default_locale,
            keys_ident,
            None,
            enum_ident,
            &top_locales,
            locales,
            &keys.0,
            &mut KeyPath::new(None),
            keys_ident,
        ),
    }
}
