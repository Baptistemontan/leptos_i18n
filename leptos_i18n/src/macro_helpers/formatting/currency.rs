use super::{IntoFixedDecimal, NumberFormatterInputFn};
use crate::Locale;
use core::fmt::{self, Display};
use icu_experimental::dimension::currency::CurrencyType;
use leptos::IntoView;

use serde::{Deserialize, Serialize};
use writeable::Writeable;

/// Width of the currency symbol.
///
/// ICU4X (since `icu_experimental` 0.7) no longer has a width option; the width
/// instead picks the constructor (`try_new_symbol` / `try_new_symbol_narrow`).
#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
#[doc(hidden)]
pub enum Width {
    #[default]
    #[serde(rename = "short")]
    Short,

    #[serde(rename = "narrow")]
    Narrow,
}

#[doc(hidden)]
pub fn format_currency_to_view<L: Locale>(
    locale: L,
    number: impl NumberFormatterInputFn,
    width: Width,
    currency_code: CurrencyType,
) -> impl IntoView + Clone {
    let currency_formatter = super::get_currency_formatter(locale, width, currency_code);

    move || {
        let fixed_dec = number.to_fixed_decimal();
        let currency = currency_formatter.format_fixed_decimal(&fixed_dec);
        let mut formatted_currency = String::new();
        currency.write_to(&mut formatted_currency).unwrap();
        formatted_currency
    }
}

#[doc(hidden)]
pub fn format_currency_to_formatter<L: Locale>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    number: impl IntoFixedDecimal,
    width: Width,
    currency_code: CurrencyType,
) -> fmt::Result {
    let currency_formatter = super::get_currency_formatter(locale, width, currency_code);
    let fixed_dec = number.to_fixed_decimal();
    let formatted_currency = currency_formatter.format_fixed_decimal(&fixed_dec);
    formatted_currency.write_to(f)
}

/// This function is a lie.
/// The only reason it exist is for the `format` macros.
/// It does NOT return a `impl Display` struct with no allocation like the other
/// This directly return a `String` of the formatted num, because borrow issues.
#[doc(hidden)]
pub fn format_currency_to_display<L: Locale>(
    locale: L,
    number: impl IntoFixedDecimal,
    width: Width,
    currency_code: CurrencyType,
) -> impl Display {
    let currency_formatter = super::get_currency_formatter(locale, width, currency_code);
    let fixed_dec = number.to_fixed_decimal();
    let currency = currency_formatter.format_fixed_decimal(&fixed_dec);
    let mut formatted_currency = String::new();
    currency.write_to(&mut formatted_currency).unwrap();
    formatted_currency
}
