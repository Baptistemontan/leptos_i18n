use std::{collections::BTreeMap, fmt::Display, rc::Rc};

use leptos_i18n_codegen::CodegenOptions;
use leptos_i18n_parser::{
    error::Diagnostics,
    extraction::extract_locales,
    formatters::Formatters,
    options::{Config, LocaleName},
    parsing::{
        ParseContext, RawLocale, RawLocalesOrNamespaces, RawValue, RawValueOrSubkeys, RawValues,
    },
    utils::{Key, KeyPath, Loc, Location},
};
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    Ident, LitStr, Token, parse::ParseBuffer, parse_macro_input, punctuated::Punctuated,
    token::Comma,
};

pub fn declare_locales(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ParsedInput {
        cfg,
        values,
        crate_path,
        interpolate_display,
    } = parse_macro_input!(tokens as ParsedInput);

    let parsed_locales = extract_locales(values, cfg, None).unwrap();

    let codegen_options = CodegenOptions::new()
        .crate_path(Some(crate_path))
        .interpolate_display(interpolate_display);

    let result = leptos_i18n_codegen::gen_code(&parsed_locales, codegen_options);

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
    cfg: Config,
    values: RawLocalesOrNamespaces,
    interpolate_display: bool,
}

fn emit_err<A, T: ToTokens, U: Display>(tokens: T, message: U) -> syn::Result<A> {
    Err(syn::Error::new_spanned(tokens, message))
}

fn make_locale_name_key(lit_str: LitStr) -> syn::Result<LocaleName> {
    let value = lit_str.value();
    let Some(key) = Key::new(&value) else {
        return Err(syn::Error::new_spanned(lit_str, "invalid key"));
    };

    match value.parse() {
        Ok(loc_id) => Ok(LocaleName {
            key,
            loc_id: Rc::new(loc_id),
        }),
        Err(err) => Err(syn::Error::new_spanned(lit_str, err)),
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
    loc: &Location,
    formatters: &Formatters,
) -> syn::Result<Option<RawValue>> {
    if !input.peek(LitStr) {
        return Ok(None);
    }
    let lit_str = input.parse::<LitStr>()?;
    let value = lit_str.value();

    let diag = Diagnostics::new();

    let ctx = ParseContext {
        loc: Loc {
            key_path: &loc.key_path,
            locale: &loc.locale,
        },
        formatters,
        diag: &diag,
        parse_fns: RawValue::DEFAULT_FNS,
    };

    match RawValue::parse(&ctx, &value) {
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
        Err(()) => {
            let errors = diag.errors();
            let err = errors.first().unwrap();
            emit_err(lit_str, err)
        }
    }
}

fn parse_map_values(
    input: syn::parse::ParseStream,
    loc: &mut Location,
    formatters: &Formatters,
) -> syn::Result<Option<RawValues>> {
    fn inner(input: syn::parse::ParseStream) -> syn::Result<ParseBuffer> {
        let content;
        syn::braced!(content in input);
        Ok(content)
    }
    let Ok(content) = inner(input) else {
        return Ok(None);
    };

    let values = parse_block_inner(content, loc, formatters)?;

    Ok(Some(values))
}

fn parse_values(
    input: syn::parse::ParseStream,
    loc: &mut Location,
    formatters: &Formatters,
) -> syn::Result<(Key, RawValueOrSubkeys)> {
    let ident: Ident = input.parse()?;
    input.parse::<Token![:]>()?;
    let key = Key::from_ident(ident);
    let mut loc = loc.push_key(key.clone());
    if let Some(parsed_value) = parse_str_value(input, &loc, formatters)? {
        return Ok((key, RawValueOrSubkeys::Value(parsed_value)));
    }
    if let Some(subkeys) = parse_map_values(input, &mut loc, formatters)? {
        return Ok((key, RawValueOrSubkeys::Subkeys(subkeys)));
    }

    Err(input.error("Invalid input"))
}

fn parse_block_inner(
    content: ParseBuffer,
    loc: &mut Location,
    formatters: &Formatters,
) -> syn::Result<RawValues> {
    let mut values = BTreeMap::new();
    while !content.is_empty() {
        let (key, value) = parse_values(&content, loc, formatters)?;
        values.insert(key, value);
        if !content.is_empty() {
            content.parse::<Comma>()?;
        }
    }
    Ok(RawValues { values })
}

fn parse_block(
    input: syn::parse::ParseStream,
    loc: &mut Location,
    formatters: &Formatters,
) -> syn::Result<RawValues> {
    let content;
    syn::braced!(content in input);
    parse_block_inner(content, loc, formatters)
}

fn parse_locale(
    input: syn::parse::ParseStream,
    locale_name: LocaleName,
    formatters: &Formatters,
) -> syn::Result<RawLocale> {
    let loc_name_ident: Ident = input.parse()?;
    if loc_name_ident != *locale_name.key.ident {
        return emit_err(loc_name_ident, "unknown locale.");
    }

    input.parse::<Token![:]>()?;

    let mut location = Location {
        key_path: KeyPath::new(None),
        locale: locale_name,
    };

    let values = parse_block(input, &mut location, formatters)?;

    if !input.is_empty() {
        input.parse::<Comma>()?;
    }

    Ok(RawLocale {
        name: location.locale,
        values,
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

        let default = make_locale_name_key(def_loc)?;

        // locales: ["defaultloc", ...]
        let loc_ident: Ident = input.parse()?;
        if loc_ident != "locales" {
            return emit_err(loc_ident, "not locales");
        }
        input.parse::<Token![:]>()?;
        let mut locales_iter = parse_array::<LitStr>(input)?.into_iter();
        match locales_iter.next() {
            None => return emit_err(loc_ident, "missing locales."),
            Some(l) if Key::new(&l.value()).as_ref() != Some(&default.key) => {
                return emit_err(l, "first locale should be the same as the default.");
            }
            _ => {}
        }
        let locales_key = std::iter::once(Ok(default.clone()))
            .chain(locales_iter.map(make_locale_name_key))
            .collect::<syn::Result<Vec<_>>>()?;
        input.parse::<Token![,]>()?;

        // loc: { .. }
        let formatters = Formatters::new();

        let locales = locales_key
            .iter()
            .cloned()
            .map(|k| parse_locale(input, k, &formatters))
            .collect::<syn::Result<Vec<_>>>()?;

        if !input.is_empty() {
            return Err(input.error("expected end of stream."));
        }

        let crate_path = crate_path
            .unwrap_or_else(|| syn::Path::from(syn::Ident::new("leptos_i18n", Span::call_site())));

        let mut config = Config::new(&default.key.name).unwrap();
        config.locales = locales_key;
        Ok(ParsedInput {
            cfg: config,
            values: RawLocalesOrNamespaces::Locales(locales),
            crate_path,
            interpolate_display,
        })
    }
}
