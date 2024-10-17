use std::ops::Bound;

use leptos_i18n_parser::parse_locales::{
    locale::InterpolOrLit,
    ranges::{Range, RangeNumber, Ranges, UntypedRangesInner},
};
use leptos_i18n_parser::{
    parse_locales::parsed_value::ParsedValue,
    utils::{Key, KeyPath},
};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};

use crate::{load_locales::parsed_value, utils::EitherOfWrapper};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum RangeType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
}

impl Default for RangeType {
    fn default() -> Self {
        Self::I32
    }
}

impl From<leptos_i18n_parser::parse_locales::ranges::RangeType> for RangeType {
    fn from(value: leptos_i18n_parser::parse_locales::ranges::RangeType) -> Self {
        match value {
            leptos_i18n_parser::parse_locales::ranges::RangeType::I8 => RangeType::I8,
            leptos_i18n_parser::parse_locales::ranges::RangeType::I16 => RangeType::I16,
            leptos_i18n_parser::parse_locales::ranges::RangeType::I32 => RangeType::I32,
            leptos_i18n_parser::parse_locales::ranges::RangeType::I64 => RangeType::I64,
            leptos_i18n_parser::parse_locales::ranges::RangeType::U8 => RangeType::U8,
            leptos_i18n_parser::parse_locales::ranges::RangeType::U16 => RangeType::U16,
            leptos_i18n_parser::parse_locales::ranges::RangeType::U32 => RangeType::U32,
            leptos_i18n_parser::parse_locales::ranges::RangeType::U64 => RangeType::U64,
            leptos_i18n_parser::parse_locales::ranges::RangeType::F32 => RangeType::F32,
            leptos_i18n_parser::parse_locales::ranges::RangeType::F64 => RangeType::F64,
        }
    }
}

impl ToTokens for RangeType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let range_type = match self {
            RangeType::I8 => quote!(i8),
            RangeType::I16 => quote!(i16),
            RangeType::I32 => quote!(i32),
            RangeType::I64 => quote!(i64),
            RangeType::U8 => quote!(u8),
            RangeType::U16 => quote!(u16),
            RangeType::U32 => quote!(u32),
            RangeType::U64 => quote!(u64),
            RangeType::F32 => quote!(f32),
            RangeType::F64 => quote!(f64),
        };
        tokens.extend(range_type)
    }
}

fn to_tokens_integers_string<T: RangeNumber>(
    ranges: &[(Range<T>, ParsedValue)],
    count_key: &Key,
    strings_count: usize,
) -> TokenStream {
    let match_arms = ranges.iter().map(|(range, value)| {
        let value = parsed_value::as_string_impl(value, strings_count);
        let range = range_to_token_stream(range);
        quote!(#range => #value)
    });

    quote! {
        {
            match *#count_key {
                #(
                    #match_arms,
                )*
            }
        }
    }
}

fn to_tokens_floats_string<T: RangeNumber>(
    ranges: &[(Range<T>, ParsedValue)],
    count_key: &Key,
    strings_count: usize,
) -> TokenStream {
    let mut ifs = ranges.iter().map(|(range, value)| {
        let value = parsed_value::as_string_impl(value, strings_count);
        match range_to_condition(range) {
            None => quote!({ #value }),
            Some(condition) => quote!(if #condition { #value }),
        }
    });
    let first = ifs.next();
    let ifs = quote! {
        #first
        #(else #ifs)*
    };

    quote! {
        {
            let plural_count = *#count_key;
            #ifs
        }
    }
}

fn to_tokens_integers<T: RangeNumber>(
    ranges: &[(Range<T>, ParsedValue)],
    count_key: &Key,
    strings_count: usize,
) -> TokenStream {
    let either_of = EitherOfWrapper::new(ranges.len());
    let match_arms = ranges.iter().enumerate().map(|(i, (range, value))| {
        let ts = parsed_value::to_token_stream(value, strings_count);
        let ts = either_of.wrap(i, ts);
        let range = range_to_token_stream(range);
        quote!(#range => { #ts })
    });

    let mut captured_values =
        InterpolOrLit::Lit(leptos_i18n_parser::parse_locales::locale::LiteralType::String);
    let mut key_path = KeyPath::new(None);

    for (_, value) in ranges {
        value
            .get_keys_inner(&mut key_path, &mut captured_values, false)
            .unwrap();
    }

    let captured_values = captured_values.is_interpol().map(|keys| {
        let keys = keys
            .iter_keys()
            .map(|key| quote!(let #key = core::clone::Clone::clone(&#key);));
        quote!(#(#keys)*)
    });
    let match_statement = quote! {
        {
            match #count_key() {
                #(
                    #match_arms,
                )*
            }
        }
    };

    quote! {
        {
            #captured_values
            move || #match_statement
        }
    }
}

fn to_tokens_floats<T: RangeNumber>(
    ranges: &[(Range<T>, ParsedValue)],
    count_key: &Key,
    strings_count: usize,
) -> TokenStream {
    let either_of = EitherOfWrapper::new(ranges.len());
    let mut ifs = ranges.iter().enumerate().map(|(i, (range, value))| {
        let ts = parsed_value::to_token_stream(value, strings_count);
        let ts = either_of.wrap(i, ts);
        match range_to_condition(range) {
            None => quote!({ #ts }),
            Some(condition) => quote!(if #condition { #ts }),
        }
    });
    let first = ifs.next();
    let ifs = quote! {
        #first
        #(else #ifs)*
    };

    let mut captured_values =
        InterpolOrLit::Lit(leptos_i18n_parser::parse_locales::locale::LiteralType::String);
    let mut key_path = KeyPath::new(None);

    for (_, value) in ranges {
        value
            .get_keys_inner(&mut key_path, &mut captured_values, false)
            .unwrap();
    }

    let captured_values = captured_values.is_interpol().map(|keys| {
        let keys = keys
            .iter_keys()
            .map(|key| quote!(let #key = core::clone::Clone::clone(&#key);));
        quote!(#(#keys)*)
    });

    quote! {
        {
            #captured_values
            move || {
                let plural_count = #count_key();
                #ifs
            }
        }
    }
}

pub fn to_token_stream(this: &Ranges, strings_count: usize) -> TokenStream {
    match &this.inner {
        UntypedRangesInner::I8(ranges) => {
            to_tokens_integers(ranges, &this.count_key, strings_count)
        }
        UntypedRangesInner::I16(ranges) => {
            to_tokens_integers(ranges, &this.count_key, strings_count)
        }
        UntypedRangesInner::I32(ranges) => {
            to_tokens_integers(ranges, &this.count_key, strings_count)
        }
        UntypedRangesInner::I64(ranges) => {
            to_tokens_integers(ranges, &this.count_key, strings_count)
        }
        UntypedRangesInner::U8(ranges) => {
            to_tokens_integers(ranges, &this.count_key, strings_count)
        }
        UntypedRangesInner::U16(ranges) => {
            to_tokens_integers(ranges, &this.count_key, strings_count)
        }
        UntypedRangesInner::U32(ranges) => {
            to_tokens_integers(ranges, &this.count_key, strings_count)
        }
        UntypedRangesInner::U64(ranges) => {
            to_tokens_integers(ranges, &this.count_key, strings_count)
        }
        UntypedRangesInner::F32(ranges) => to_tokens_floats(ranges, &this.count_key, strings_count),
        UntypedRangesInner::F64(ranges) => to_tokens_floats(ranges, &this.count_key, strings_count),
    }
}

pub fn as_string_impl(this: &Ranges, strings_count: usize) -> TokenStream {
    match &this.inner {
        UntypedRangesInner::I8(ranges) => {
            to_tokens_integers_string(ranges, &this.count_key, strings_count)
        }
        UntypedRangesInner::I16(ranges) => {
            to_tokens_integers_string(ranges, &this.count_key, strings_count)
        }
        UntypedRangesInner::I32(ranges) => {
            to_tokens_integers_string(ranges, &this.count_key, strings_count)
        }
        UntypedRangesInner::I64(ranges) => {
            to_tokens_integers_string(ranges, &this.count_key, strings_count)
        }
        UntypedRangesInner::U8(ranges) => {
            to_tokens_integers_string(ranges, &this.count_key, strings_count)
        }
        UntypedRangesInner::U16(ranges) => {
            to_tokens_integers_string(ranges, &this.count_key, strings_count)
        }
        UntypedRangesInner::U32(ranges) => {
            to_tokens_integers_string(ranges, &this.count_key, strings_count)
        }
        UntypedRangesInner::U64(ranges) => {
            to_tokens_integers_string(ranges, &this.count_key, strings_count)
        }
        UntypedRangesInner::F32(ranges) => {
            to_tokens_floats_string(ranges, &this.count_key, strings_count)
        }
        UntypedRangesInner::F64(ranges) => {
            to_tokens_floats_string(ranges, &this.count_key, strings_count)
        }
    }
}

fn range_to_condition<T: RangeNumber>(range: &Range<T>) -> Option<TokenStream> {
    match range {
        Range::Exact(exact) => Some(quote!(plural_count == #exact)),
        Range::Bounds { .. } => {
            let ts = range_to_token_stream(range);
            Some(quote!(core::ops::RangeBounds::contains(&(#ts), &plural_count)))
        }
        Range::Multiple(conditions) => {
            let mut conditions = conditions.iter().filter_map(range_to_condition);
            let first = conditions.next();
            Some(quote!(#first #(|| #conditions)*))
        }
        Range::Fallback => None,
    }
}

fn range_to_token_stream<T: RangeNumber>(range: &Range<T>) -> proc_macro2::TokenStream {
    match range {
        Range::Exact(num) => quote!(#num),
        Range::Bounds {
            start,
            end: Bound::Included(end),
        } => {
            quote!(#start..=#end)
        }
        Range::Bounds {
            start,
            end: Bound::Unbounded,
        } => {
            quote!(#start..)
        }
        Range::Bounds {
            start,
            end: Bound::Excluded(end),
        } => {
            quote!(#start..#end)
        }
        Range::Fallback => quote!(_),
        Range::Multiple(matchs) => {
            let mut matchs = matchs.iter().map(range_to_token_stream);
            if let Some(first) = matchs.next() {
                quote!(#first #(| #matchs)*)
            } else {
                quote!()
            }
        }
    }
}
