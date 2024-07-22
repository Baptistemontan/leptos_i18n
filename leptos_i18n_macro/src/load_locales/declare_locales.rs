use std::{collections::HashMap, fmt::Display, rc::Rc};

use crate::load_locales::key::Key;

use super::{
    cfg_file::ConfigFile,
    key::KeyPath,
    load_locales_inner,
    locale::{Locale, LocalesOrNamespaces},
    parsed_value::ParsedValue,
};
use quote::ToTokens;
use syn::{parse_macro_input, punctuated::Punctuated, token::Comma, Ident, LitStr, Token};

pub fn declare_locales(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ParsedInput {
        cfg_file,
        mut locales,
    } = parse_macro_input!(tokens as ParsedInput);
    let result = load_locales_inner(&cfg_file, &mut locales);
    match result {
        Ok(ts) => ts.into(),
        Err(err) => err.into(),
    }
}

pub struct ParsedInput {
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

fn parse_values(
    input: syn::parse::ParseStream,
    key_path: &mut KeyPath,
    locale: &Rc<Key>,
) -> syn::Result<(Rc<Key>, ParsedValue)> {
    let ident: Ident = input.parse()?;
    let key = Rc::new(Key::from_ident(ident));
    key_path.push_key(key);
    input.parse::<Token![:]>()?;
    let value = input.parse::<LitStr>()?.value();
    let parsed_value = ParsedValue::new(&value, key_path, locale);
    let key = key_path.pop_key().unwrap();
    Ok((key, parsed_value))
}

fn parse_block(
    input: syn::parse::ParseStream,
    key_path: &mut KeyPath,
    locale: &Rc<Key>,
) -> syn::Result<HashMap<Rc<Key>, ParsedValue>> {
    let content;
    syn::braced!(content in input);
    let mut values = HashMap::new();
    while !content.is_empty() {
        let (key, value) = parse_values(&content, key_path, locale)?;
        values.insert(key, value);
        if !content.is_empty() {
            content.parse::<Comma>()?;
        }
    }
    Ok(values)
}

fn parse_locale(input: syn::parse::ParseStream, locale_key: Rc<Key>) -> syn::Result<Locale> {
    let loc_name_ident: Ident = input.parse()?;
    if loc_name_ident != locale_key.name {
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
    })
}

impl syn::parse::Parse for ParsedInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // default: "defaultloc",
        let def_ident: Ident = input.parse()?;
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

        Ok(ParsedInput {
            cfg_file: ConfigFile {
                default,
                locales: locales_key,
                name_spaces: None,
                locales_dir: "".into(),
            },
            locales: LocalesOrNamespaces::Locales(locales),
        })
    }
}
