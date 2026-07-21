use std::{fmt::Display, ops::Bound};

use leptos_i18n_parser::{
    parse_locales::{
        locale::InterpolOrLit,
        parsed_value::ParsedValue,
        ranges::{Range, RangeNumber, Ranges, UntypedRangesInner},
    },
    utils::{Key, KeyPath, UnwrapAt},
};
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

use crate::{load_locales::parsed_value, utils::EitherOfWrapper};

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
pub enum RangeType {
    I8,
    I16,
    #[default]
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
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

/// Integers specific operations, needed to check that a set of ranges covers the whole domain of
/// the type. `RangeNumber` alone doesn't expose the bounds nor a way to step through values.
trait IntegerRangeNumber: RangeNumber + Ord + Display {
    const MIN: Self;
    const MAX: Self;

    fn checked_pred(self) -> Option<Self>;
    fn checked_succ(self) -> Option<Self>;
}

macro_rules! impl_integer_range_number {
    ($($num_type:ty)*) => {
        $(
            impl IntegerRangeNumber for $num_type {
                const MIN: Self = <$num_type>::MIN;
                const MAX: Self = <$num_type>::MAX;

                fn checked_pred(self) -> Option<Self> {
                    self.checked_sub(1)
                }

                fn checked_succ(self) -> Option<Self> {
                    self.checked_add(1)
                }
            }
        )*
    };
}

impl_integer_range_number!(i8 i16 i32 i64 u8 u16 u32 u64);

/// Push the values matched by `range` as inclusive intervals.
fn push_covered_intervals<T: IntegerRangeNumber>(range: &Range<T>, covered: &mut Vec<(T, T)>) {
    match range {
        Range::Exact(value) => covered.push((*value, *value)),
        Range::Bounds { start, end } => {
            let start = start.unwrap_or(T::MIN);
            let end = match end {
                Bound::Included(end) => Some(*end),
                // `x..MIN` matches nothing.
                Bound::Excluded(end) => end.checked_pred(),
                Bound::Unbounded => Some(T::MAX),
            };
            if let Some(end) = end
                && start <= end
            {
                covered.push((start, end));
            }
        }
        Range::Multiple(ranges) => {
            for range in ranges {
                push_covered_intervals(range, covered);
            }
        }
        Range::Fallback => covered.push((T::MIN, T::MAX)),
    }
}

/// Compute the inclusive intervals of `T` left unmatched by `ranges`.
///
/// Only a fallback is mandatory for floats (`RangeType::should_have_fallback`), so integer ranges
/// can leave holes in the domain of the count. The generated `match` would then be
/// non-exhaustive, failing the user build with a `E0004` pointing inside generated code, so the
/// holes are computed here to report them properly.
fn uncovered_intervals<T: IntegerRangeNumber>(ranges: &[(Range<T>, ParsedValue)]) -> Vec<(T, T)> {
    let mut covered = Vec::with_capacity(ranges.len());

    for (range, _) in ranges {
        push_covered_intervals(range, &mut covered);
    }

    covered.sort_unstable_by_key(|(start, _)| *start);

    let mut uncovered = Vec::new();
    // smallest value not covered by the intervals seen so far, `None` means the domain is
    // exhausted.
    let mut cursor = Some(T::MIN);

    for (start, end) in covered {
        let Some(current) = cursor else {
            break;
        };
        if start > current {
            // `start > current >= T::MIN` so `start` has a predecessor.
            uncovered.push((
                current,
                start.checked_pred().unwrap_at("uncovered_intervals_1"),
            ));
        }
        if end >= current {
            cursor = end.checked_succ();
        }
    }

    if let Some(current) = cursor {
        uncovered.push((current, T::MAX));
    }

    uncovered
}

/// A `_` arm reporting the values not covered by `ranges`, if any.
fn missing_ranges_arm<T: IntegerRangeNumber>(
    ranges: &[(Range<T>, ParsedValue)],
) -> Option<TokenStream> {
    let uncovered = uncovered_intervals(ranges);

    if uncovered.is_empty() {
        return None;
    }

    let missing = uncovered
        .iter()
        .map(|(start, end)| {
            if start == end {
                format!("{start}")
            } else {
                format!("{start}..={end}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ");

    let msg = format!(
        "ranges don't cover every possible value of `{}`, missing: {missing}. Add a \"_\" fallback range or widen the existing ones.",
        T::TYPE
    );

    Some(quote!(_ => core::compile_error!(#msg)))
}

fn to_tokens_integers_string<T: IntegerRangeNumber>(
    ranges: &[(Range<T>, ParsedValue)],
    count_key: &Key,
    strings_count: usize,
) -> TokenStream {
    let match_arms = ranges.iter().map(|(range, value)| {
        let value = parsed_value::as_string_impl(value, strings_count);
        let range = range_to_token_stream(range);
        quote!(#range => #value)
    });

    let missing_ranges_arm = missing_ranges_arm(ranges);

    quote! {
        {
            match *#count_key {
                #(
                    #match_arms,
                )*
                #missing_ranges_arm
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

fn to_tokens_integers<T: IntegerRangeNumber>(
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
            .unwrap_at("ranges::to_tokens_integers_1");
    }

    let captured_values = captured_values.is_interpol().map(|keys| {
        let keys = keys
            .iter_keys()
            .map(|key| quote!(let #key = core::clone::Clone::clone(&#key);));
        quote!(#(#keys)*)
    });
    let missing_ranges_arm = missing_ranges_arm(ranges);

    let match_statement = quote! {
        {
            match #count_key() {
                #(
                    #match_arms,
                )*
                #missing_ranges_arm
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
            .unwrap_at("ranges::to_tokens_floats_1");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn ranges<T: IntegerRangeNumber>(
        ranges: impl IntoIterator<Item = &'static str>,
    ) -> Vec<(Range<T>, ParsedValue)> {
        ranges
            .into_iter()
            .map(|range| (Range::new(range).unwrap(), ParsedValue::Default))
            .collect()
    }

    #[test]
    fn full_domain_is_covered() {
        assert!(uncovered_intervals(&ranges::<i32>(["..0", "0", "1.."])).is_empty());
        assert!(uncovered_intervals(&ranges::<u8>(["..=127", "128.."])).is_empty());
        assert!(uncovered_intervals(&ranges::<i8>(["_"])).is_empty());
        assert!(uncovered_intervals(&ranges::<u8>(["0..=255"])).is_empty());
        // overlapping ranges
        assert!(uncovered_intervals(&ranges::<u8>(["..=200", "100.."])).is_empty());
        // out of order ranges
        assert!(uncovered_intervals(&ranges::<u8>(["128..", "..=127"])).is_empty());
        // "|" separated ranges
        assert!(uncovered_intervals(&ranges::<u8>(["0..=127 | 128..=255"])).is_empty());
    }

    #[test]
    fn holes_are_found() {
        assert_eq!(
            uncovered_intervals(&ranges::<u8>(["0", "1..=3"])),
            [(4, 255)]
        );
        assert_eq!(
            uncovered_intervals(&ranges::<i8>(["0", "1..=3"])),
            [(-128, -1), (4, 127)]
        );
        assert_eq!(uncovered_intervals(&ranges::<u8>(["1.."])), [(0, 0)]);
        assert_eq!(uncovered_intervals(&ranges::<u8>([])), [(0, 255)]);
        // exclusive end
        assert_eq!(uncovered_intervals(&ranges::<u8>(["..255"])), [(255, 255)]);
    }
}
