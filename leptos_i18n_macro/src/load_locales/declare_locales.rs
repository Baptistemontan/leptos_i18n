use std::{collections::BTreeMap, fmt::Display};

use leptos_i18n_parser::{
    formatters::Formatters,
    parse_locales::{
        ForeignKeysPaths, ParsedLocales,
        cfg_file::ConfigFile,
        error::Diagnostics,
        locale::{Locale, LocalesOrNamespaces},
        make_builder_keys,
        options::{Config, ParseOptions},
        parsed_value::ParsedValue,
    },
    utils::{Key, KeyPath, Loc, ParseContext},
};
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    Ident, LitStr, Token, parse::ParseBuffer, parse_macro_input, punctuated::Punctuated,
    token::Comma,
};

pub fn declare_locales(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ParsedInput {
        cfg_file,
        locales,
        crate_path,
        foreign_keys_paths,
        interpolate_display,
    } = parse_macro_input!(tokens as ParsedInput);
    let diag = Diagnostics::new();

    let mut cfg: Config = cfg_file.into();

    cfg.options = ParseOptions::default().interpolate_display(interpolate_display);

    let builder_keys = make_builder_keys(locales, &cfg, foreign_keys_paths, &diag).unwrap();

    let parsed_locales = ParsedLocales {
        cfg,
        builder_keys,
        diag,
        tracked_files: None,
    };

    let result =
        leptos_i18n_codegen::gen_code(&parsed_locales, Some(&crate_path), true, None, true);
    match result {
        Ok(ts) => ts.into(),
        Err(err) => {
            let err = err.to_string();
            quote::quote!(compile_error!(#err);).into()
        }
    }
}

pub struct ParsedInput {
    crate_path: syn::Path,
    cfg_file: ConfigFile,
    locales: LocalesOrNamespaces,
    foreign_keys_paths: ForeignKeysPaths,
    interpolate_display: bool,
}

fn emit_err<A, T: ToTokens, U: Display>(tokens: T, message: U) -> syn::Result<A> {
    Err(syn::Error::new_spanned(tokens, message))
}

fn make_key(lit_str: LitStr) -> syn::Result<Key> {
    let value = lit_str.value();
    if let Some(k) = Key::new(&value) {
        Ok(k)
    } else {
        Err(syn::Error::new_spanned(lit_str, "invalid key"))
    }
}

fn parse_array<T: syn::parse::Parse>(
    input: syn::parse::ParseStream,
) -> syn::Result<Punctuated<T, Comma>> {
    let content;
    syn::bracketed!(content in input);
    content.parse_terminated(T::parse, Comma)
}

fn parse_str_value(
    input: syn::parse::ParseStream,
    loc: &Loc,
    formatters: &Formatters,
    foreign_keys_paths: &ForeignKeysPaths,
) -> syn::Result<Option<ParsedValue>> {
    if !input.peek(LitStr) {
        return Ok(None);
    }
    let lit_str = input.parse::<LitStr>()?;
    let value = lit_str.value();

    let diag = Diagnostics::new();

    let ctx = ParseContext {
        loc: *loc,
        foreign_keys_paths,
        formatters,
        diag: &diag,
        parse_fns: ParsedValue::DEFAULT_FNS,
    };

    match ParsedValue::new(&ctx, &value) {
        Ok(pv) => {
            if let Some(err) = diag.errors().first() {
                return emit_err(lit_str, err);
            }
            if let Some(warn) = diag.warnings().first() {
                // TODO: warn instead of error
                return emit_err(lit_str, warn);
            }
            Ok(Some(pv))
        }
        Err(err) => emit_err(lit_str, err),
    }
}

fn parse_map_values(
    input: syn::parse::ParseStream,
    name: &Key,
    key_path: &mut KeyPath,
    locale: &Key,
    formatters: &Formatters,
    foreign_keys_paths: &ForeignKeysPaths,
) -> syn::Result<Option<ParsedValue>> {
    fn inner(input: syn::parse::ParseStream) -> syn::Result<ParseBuffer> {
        let content;
        syn::braced!(content in input);
        Ok(content)
    }
    let Ok(content) = inner(input) else {
        return Ok(None);
    };

    let keys = parse_block_inner(content, key_path, locale, formatters, foreign_keys_paths)?;

    Ok(Some(ParsedValue::Subkeys(Some(Locale {
        top_locale_name: locale.clone(),
        name: name.clone(),
        keys,
        strings: vec![],
        top_locale_string_count: 0,
    }))))
}

fn parse_values(
    input: syn::parse::ParseStream,
    key_path: &mut KeyPath,
    locale: &Key,
    formatters: &Formatters,
    foreign_keys_paths: &ForeignKeysPaths,
) -> syn::Result<(Key, ParsedValue)> {
    let ident: Ident = input.parse()?;
    let key = Key::from_ident(ident);
    let mut pushed_key = key_path.push_key(key.clone());
    input.parse::<Token![:]>()?;
    let loc = Loc {
        locale,
        key_path: &pushed_key,
    };
    if let Some(parsed_value) = parse_str_value(input, &loc, formatters, foreign_keys_paths)? {
        return Ok((key, parsed_value));
    }
    if let Some(parsed_value) = parse_map_values(
        input,
        &key,
        &mut pushed_key,
        locale,
        formatters,
        foreign_keys_paths,
    )? {
        return Ok((key, parsed_value));
    }

    Err(input.error("Invalid input"))
}

fn parse_block_inner(
    content: ParseBuffer,
    key_path: &mut KeyPath,
    locale: &Key,
    formatters: &Formatters,
    foreign_keys_paths: &ForeignKeysPaths,
) -> syn::Result<BTreeMap<Key, ParsedValue>> {
    let mut values = BTreeMap::new();
    while !content.is_empty() {
        let (key, value) =
            parse_values(&content, key_path, locale, formatters, foreign_keys_paths)?;
        values.insert(key, value);
        if !content.is_empty() {
            content.parse::<Comma>()?;
        }
    }
    Ok(values)
}

fn parse_block(
    input: syn::parse::ParseStream,
    key_path: &mut KeyPath,
    locale: &Key,
    formatters: &Formatters,
    foreign_keys_paths: &ForeignKeysPaths,
) -> syn::Result<BTreeMap<Key, ParsedValue>> {
    let content;
    syn::braced!(content in input);
    parse_block_inner(content, key_path, locale, formatters, foreign_keys_paths)
}

fn parse_locale(
    input: syn::parse::ParseStream,
    locale_key: Key,
    formatters: &Formatters,
    foreign_keys_paths: &ForeignKeysPaths,
) -> syn::Result<Locale> {
    let loc_name_ident: Ident = input.parse()?;
    if loc_name_ident != *locale_key.ident {
        return emit_err(loc_name_ident, "unknown locale.");
    }

    input.parse::<Token![:]>()?;

    let mut key_path = KeyPath::new(None);

    let keys = parse_block(
        input,
        &mut key_path,
        &locale_key,
        formatters,
        foreign_keys_paths,
    )?;

    if !input.is_empty() {
        input.parse::<Comma>()?;
    }

    Ok(Locale {
        top_locale_name: locale_key.clone(),
        name: locale_key,
        keys,
        strings: vec![],
        top_locale_string_count: 0,
    })
}

impl syn::parse::Parse for ParsedInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let crate_path = if ident == "path" {
            input.parse::<Token![:]>()?;
            let path = input.parse::<syn::Path>()?;
            input.parse::<Token![,]>()?;
            Some(path)
        } else {
            None
        };
        let ident: Ident = if crate_path.is_none() {
            ident
        } else {
            input.parse()?
        };

        let interpolate_display = ident == "interpolate_display";

        // default: "defaultloc",
        let def_ident: Ident = if interpolate_display {
            input.parse::<Token![,]>()?;
            input.parse()?
        } else {
            ident
        };
        if def_ident != "default" {
            return emit_err(def_ident, "not default");
        }
        input.parse::<Token![:]>()?;
        let def_loc = input.parse::<LitStr>()?;
        input.parse::<Token![,]>()?;

        let default = make_key(def_loc)?;

        // locales: ["defaultloc", ...]
        let loc_ident: Ident = input.parse()?;
        if loc_ident != "locales" {
            return emit_err(loc_ident, "not locales");
        }
        input.parse::<Token![:]>()?;
        let mut locales_iter = parse_array::<LitStr>(input)?.into_iter();
        match locales_iter.next() {
            None => return emit_err(loc_ident, "missing locales."),
            Some(l) if Key::new(&l.value()).as_ref() != Some(&default) => {
                return emit_err(l, "first locale should be the same as the default.");
            }
            _ => {}
        }
        let locales_key = std::iter::once(Ok(default.clone()))
            .chain(locales_iter.map(make_key))
            .collect::<syn::Result<Vec<_>>>()?;
        input.parse::<Token![,]>()?;

        // loc: { .. }

        let foreign_keys_paths = ForeignKeysPaths::new();
        let formatters = Formatters::new();

        let locales = locales_key
            .iter()
            .cloned()
            .map(|k| parse_locale(input, k, &formatters, &foreign_keys_paths))
            .collect::<syn::Result<Vec<_>>>()?;

        if !input.is_empty() {
            return Err(input.error("expected end of stream."));
        }

        let crate_path = crate_path
            .unwrap_or_else(|| syn::Path::from(syn::Ident::new("leptos_i18n", Span::call_site())));

        Ok(ParsedInput {
            cfg_file: ConfigFile {
                default,
                locales: locales_key,
                name_spaces: None,
                locales_dir: "".into(),
                translations_uri: None,
                extensions: Default::default(),
            },
            locales: LocalesOrNamespaces::Locales(locales),
            crate_path,
            foreign_keys_paths,
            interpolate_display,
        })
    }
}
