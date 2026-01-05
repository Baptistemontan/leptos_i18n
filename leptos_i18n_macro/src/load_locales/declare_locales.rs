use std::{collections::BTreeMap, fmt::Display};

use leptos_i18n_parser::{
    formatters::Formatters,
    parse_locales::{
        ForeignKeysPaths, ParsedLocales,
        cfg_file::ConfigFile,
        error::Diagnostics,
        locale::{Locale, LocalesOrNamespaces},
        make_builder_keys,
        options::ParseOptions,
        parsed_value::{Context, ParsedValue},
        ranges::{
            ParseRanges, Range, RangeNumber, Ranges, RangesInner, TypeOrRange, UntypedRangesInner,
        },
    },
    utils::{Key, KeyPath},
};
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    Ident, Lit, LitStr, Token, parse::ParseBuffer, parse_macro_input, punctuated::Punctuated,
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

    let options = ParseOptions::default().interpolate_display(interpolate_display);

    let builder_keys =
        make_builder_keys(locales, &cfg_file, foreign_keys_paths, &diag, &options).unwrap();

    let parsed_locales = ParsedLocales {
        cfg_file,
        builder_keys,
        diag,
        tracked_files: None,
        options,
    };

    let result = leptos_i18n_codegen::gen_code(&parsed_locales, Some(&crate_path), true, None);
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
    key_path: &mut KeyPath,
    locale: &Key,
    formatters: &Formatters,
    foreign_keys_paths: &ForeignKeysPaths,
) -> syn::Result<Option<ParsedValue>> {
    if !input.peek(LitStr) {
        return Ok(None);
    }
    let lit_str = input.parse::<LitStr>()?;
    let value = lit_str.value();

    let diag = Diagnostics::new();

    let ctx = Context {
        locale,
        key_path,
        foreign_keys_paths,
        formatters,
        diag: &diag,
        parse_fns: ParsedValue::DEFAULT_FNS,
    };

    match ParsedValue::new(&ctx, &value) {
        Ok(pv) => {
            if let Some(err) = diag.errors().first() {
                return Err(syn::Error::new_spanned(lit_str, err.to_string()));
            }
            if let Some(warn) = diag.warnings().first() {
                // TODO: warn instead of error
                return Err(syn::Error::new_spanned(lit_str, warn.to_string()));
            }
            Ok(Some(pv))
        }
        Err(err) => Err(syn::Error::new_spanned(lit_str, err.to_string())),
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

pub struct ParseRangeSeed<'a> {
    pub key_path: &'a mut KeyPath,
    pub locale: &'a Key,
    pub formatters: &'a Formatters,
    pub foreign_keys_paths: &'a ForeignKeysPaths,
}

fn parse_range_count<T: RangeNumber>(input: &ParseBuffer) -> syn::Result<Range<T>> {
    let lit = input.parse::<Lit>()?;
    let range = match lit {
        Lit::Str(slit) => {
            let s = slit.value();
            match Range::new(&s) {
                Ok(p) => p,
                Err(_) => return emit_err(slit, "invalid range count."),
            }
        }
        Lit::Int(intlit) => {
            let n = intlit
                .base10_digits()
                .parse()
                .map_err(|_| syn::Error::new(intlit.span(), "invalid int"))?;
            Range::Exact(n)
        }
        Lit::Float(floatlit) => {
            let n = floatlit
                .base10_digits()
                .parse()
                .map_err(|_| syn::Error::new(floatlit.span(), "invalid float"))?;
            Range::Exact(n)
        }
        _ => return emit_err(lit, "invalid litteral."),
    };
    Ok(range)
}

fn parse_range_pair<T: RangeNumber>(
    input: &ParseBuffer,
    seed: &mut ParseRangeSeed,
    foreign_keys_paths: &ForeignKeysPaths,
) -> syn::Result<(Range<T>, ParsedValue)> {
    let content;
    syn::bracketed!(content in input);

    let Some(parsed_value) = parse_str_value(
        &content,
        seed.key_path,
        seed.locale,
        seed.formatters,
        foreign_keys_paths,
    )?
    else {
        return Err(content.error("only strings are accepted here."));
    };

    if content.is_empty() {
        return Ok((Range::Fallback, parsed_value));
    }
    content.parse::<Comma>()?;

    let mut counts = content
        .parse_terminated(parse_range_count::<T>, Comma)?
        .into_iter();

    match (counts.next(), counts.next()) {
        (None, _) => Ok((Range::Fallback, parsed_value)),
        (Some(count), None) => Ok((count, parsed_value)),
        (Some(a), Some(b)) => Ok((
            Range::Multiple([a, b].into_iter().chain(counts).collect()),
            parsed_value,
        )),
    }
}

pub fn parse_range_pairs<T: RangeNumber>(
    content: &ParseBuffer,
    ranges: &mut RangesInner<T>,
    mut seed: ParseRangeSeed,
) -> syn::Result<()> {
    let foreign_keys_paths = seed.foreign_keys_paths;
    while !content.is_empty() {
        let pair = parse_range_pair(content, &mut seed, foreign_keys_paths)?;
        ranges.push(pair);
        if !content.is_empty() {
            content.parse::<Comma>()?;
        }
    }
    Ok(())
}

fn parse_range_type(
    content: &ParseBuffer,
    seed: &mut ParseRangeSeed,
    foreign_keys_paths: &ForeignKeysPaths,
) -> syn::Result<TypeOrRange> {
    if content.peek(LitStr) {
        let lit_str = content.parse::<LitStr>()?;
        let s = lit_str.value();
        return TypeOrRange::from_string(&s)
            .ok_or_else(|| syn::Error::new_spanned(lit_str, "invalid range type."));
    }

    let range = parse_range_pair(content, seed, foreign_keys_paths)?;

    Ok(TypeOrRange::Range(range))
}

fn parse_ranges(
    input: syn::parse::ParseStream,
    mut seed: ParseRangeSeed,
    foreign_keys_paths: &ForeignKeysPaths,
) -> syn::Result<Option<ParsedValue>> {
    fn inner(input: syn::parse::ParseStream) -> syn::Result<ParseBuffer> {
        let content;
        syn::bracketed!(content in input);
        Ok(content)
    }
    let Ok(content) = inner(input) else {
        return Ok(None);
    };

    let mut ranges = match parse_range_type(&content, &mut seed, foreign_keys_paths)? {
        TypeOrRange::Type(range_type) => Ranges::from_type(range_type),
        TypeOrRange::Range(range) => Ranges {
            inner: UntypedRangesInner::I32(vec![range]),
            count_key: Key::count(),
        },
    };

    ranges.deserialize_inner(RangeParseBuffer(content), seed)?;

    Ok(Some(ParsedValue::Ranges(ranges)))
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
    if let Some(parsed_value) = parse_str_value(
        input,
        &mut pushed_key,
        locale,
        formatters,
        foreign_keys_paths,
    )? {
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

    let seed = ParseRangeSeed {
        key_path: &mut pushed_key,
        locale,
        formatters,
        foreign_keys_paths,
    };

    if let Some(parsed_value) = parse_ranges(input, seed, foreign_keys_paths)? {
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

pub struct RangeParseBuffer<'de>(ParseBuffer<'de>);

impl<'a, 'de> ParseRanges<'a, 'de> for RangeParseBuffer<'de> {
    type Result<O>
        = syn::Result<O>
    where
        O: 'de + 'a;

    type Seed = super::declare_locales::ParseRangeSeed<'a>;

    fn deserialize_all_pairs<T: RangeNumber>(
        self,
        ranges: &mut RangesInner<T>,
        seed: Self::Seed,
    ) -> Self::Result<()> {
        parse_range_pairs(&self.0, ranges, seed)
    }
}
