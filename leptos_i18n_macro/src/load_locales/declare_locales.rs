use std::{collections::BTreeMap, fmt::Display, rc::Rc};

use crate::load_locales::ranges::{RangeParseBuffer, Ranges, UntypedRangesInner};
use crate::utils::key::{Key, KeyPath, CACHED_VAR_COUNT_KEY};

use super::{
    cfg_file::ConfigFile,
    load_locales_inner,
    locale::{Locale, LocalesOrNamespaces},
    parsed_value::ParsedValue,
    ranges::{Range, RangeNumber, RangesInner, TypeOrRange},
};
use proc_macro2::Span;
use quote::ToTokens;
use syn::{
    parse::ParseBuffer, parse_macro_input, punctuated::Punctuated, token::Comma, Ident, Lit,
    LitStr, Token,
};

pub fn declare_locales(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ParsedInput {
        cfg_file,
        mut locales,
        crate_path,
    } = parse_macro_input!(tokens as ParsedInput);
    let result = load_locales_inner(&crate_path, &cfg_file, &mut locales);
    match result {
        Ok(ts) => ts.into(),
        Err(err) => err.into(),
    }
}

pub struct ParsedInput {
    crate_path: syn::Path,
    cfg_file: ConfigFile,
    locales: LocalesOrNamespaces,
}

fn emit_err<A, T: ToTokens, U: Display>(tokens: T, message: U) -> syn::Result<A> {
    Err(syn::Error::new_spanned(tokens, message))
}

fn make_key(lit_str: LitStr) -> syn::Result<Rc<Key>> {
    let value = lit_str.value();
    if let Some(k) = Key::new(&value) {
        Ok(Rc::new(k))
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
    locale: &Rc<Key>,
) -> syn::Result<Option<ParsedValue>> {
    if !input.peek(LitStr) {
        return Ok(None);
    }
    let lit_str = input.parse::<LitStr>()?;
    let value = lit_str.value();
    ParsedValue::new(&value, key_path, locale)
        .map(Some)
        .map_err(|_| syn::Error::new_spanned(lit_str, "unknown formatter."))
}

fn parse_map_values(
    input: syn::parse::ParseStream,
    name: &Rc<Key>,
    key_path: &mut KeyPath,
    locale: &Rc<Key>,
) -> syn::Result<Option<ParsedValue>> {
    fn inner(input: syn::parse::ParseStream) -> syn::Result<ParseBuffer> {
        let content;
        syn::braced!(content in input);
        Ok(content)
    }
    let Ok(content) = inner(input) else {
        return Ok(None);
    };

    let keys = parse_block_inner(content, key_path, locale)?;

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
    pub locale: &'a Rc<Key>,
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
) -> syn::Result<(Range<T>, ParsedValue)> {
    let content;
    syn::bracketed!(content in input);

    let Some(parsed_value) = parse_str_value(&content, seed.key_path, seed.locale)? else {
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
    while !content.is_empty() {
        let pair = parse_range_pair(content, &mut seed)?;
        ranges.push(pair);
        if !content.is_empty() {
            content.parse::<Comma>()?;
        }
    }
    Ok(())
}

fn parse_range_type(content: &ParseBuffer, seed: &mut ParseRangeSeed) -> syn::Result<TypeOrRange> {
    if content.peek(LitStr) {
        let lit_str = content.parse::<LitStr>()?;
        let s = lit_str.value();
        return TypeOrRange::from_str(&s)
            .ok_or_else(|| syn::Error::new_spanned(lit_str, "invalid range type."));
    }

    let range = parse_range_pair(content, seed)?;

    Ok(TypeOrRange::Range(range))
}

fn parse_ranges(
    input: syn::parse::ParseStream,
    mut seed: ParseRangeSeed,
) -> syn::Result<Option<ParsedValue>> {
    fn inner(input: syn::parse::ParseStream) -> syn::Result<ParseBuffer> {
        let content;
        syn::bracketed!(content in input);
        Ok(content)
    }
    let Ok(content) = inner(input) else {
        return Ok(None);
    };

    let mut ranges = match parse_range_type(&content, &mut seed)? {
        TypeOrRange::Type(range_type) => Ranges::from_type(range_type),
        TypeOrRange::Range(range) => Ranges {
            inner: UntypedRangesInner::I32(vec![range]),
            count_key: CACHED_VAR_COUNT_KEY.with(Clone::clone),
        },
    };

    ranges.deserialize_inner(RangeParseBuffer(content), seed)?;

    Ok(Some(ParsedValue::Ranges(ranges)))
}

fn parse_values(
    input: syn::parse::ParseStream,
    key_path: &mut KeyPath,
    locale: &Rc<Key>,
) -> syn::Result<(Rc<Key>, ParsedValue)> {
    let ident: Ident = input.parse()?;
    let key = Rc::new(Key::from_ident(ident));
    key_path.push_key(key.clone());
    input.parse::<Token![:]>()?;
    if let Some(parsed_value) = parse_str_value(input, key_path, locale)? {
        key_path.pop_key();
        return Ok((key, parsed_value));
    }
    if let Some(parsed_value) = parse_map_values(input, &key, key_path, locale)? {
        key_path.pop_key();
        return Ok((key, parsed_value));
    }

    let seed = ParseRangeSeed { key_path, locale };

    if let Some(parsed_value) = parse_ranges(input, seed)? {
        key_path.pop_key();
        return Ok((key, parsed_value));
    }

    Err(input.error("Invalid input"))
}

fn parse_block_inner(
    content: ParseBuffer,
    key_path: &mut KeyPath,
    locale: &Rc<Key>,
) -> syn::Result<BTreeMap<Rc<Key>, ParsedValue>> {
    let mut values = BTreeMap::new();
    while !content.is_empty() {
        let (key, value) = parse_values(&content, key_path, locale)?;
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
    locale: &Rc<Key>,
) -> syn::Result<BTreeMap<Rc<Key>, ParsedValue>> {
    let content;
    syn::braced!(content in input);
    parse_block_inner(content, key_path, locale)
}

fn parse_locale(input: syn::parse::ParseStream, locale_key: Rc<Key>) -> syn::Result<Locale> {
    let loc_name_ident: Ident = input.parse()?;
    if loc_name_ident != locale_key.ident {
        return emit_err(loc_name_ident, "unknown locale.");
    }

    input.parse::<Token![:]>()?;

    let mut key_path = KeyPath::new(None);

    let keys = parse_block(input, &mut key_path, &locale_key)?;

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

        // default: "defaultloc",
        let def_ident: Ident = if crate_path.is_none() {
            ident
        } else {
            input.parse()?
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
            Some(l) if Key::new(&l.value()).as_ref() != Some(&*default) => {
                return emit_err(l, "first locale should be the same as the default.")
            }
            _ => {}
        }
        let locales_key = std::iter::once(Ok(default.clone()))
            .chain(locales_iter.map(make_key))
            .collect::<syn::Result<Vec<_>>>()?;
        input.parse::<Token![,]>()?;

        // loc: { .. }

        let locales = locales_key
            .iter()
            .cloned()
            .map(|k| parse_locale(input, k))
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
            },
            locales: LocalesOrNamespaces::Locales(locales),
            crate_path,
        })
    }
}
