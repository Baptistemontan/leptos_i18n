use super::{IntoFixedDecimal, NumberFormatterInputFn};
use crate::Locale;
use core::fmt::{self, Display};
use icu_experimental::dimension::currency::{CurrencyCode, options::Width as CurrencyWidth};
use leptos::IntoView;

use serde::{Deserialize, Serialize};
use writeable::Writeable;

// TODO: this struct should be removed in version ICU4x v2
// Reference: https://docs.rs/icu_experimental/0.1.0/icu_experimental/dimension/currency/options/enum.Width.html
// Issue: https://github.com/unicode-org/icu4x/pull/6100
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

impl From<CurrencyWidth> for Width {
    fn from(value: CurrencyWidth) -> Self {
        match value {
            CurrencyWidth::Short => Self::Short,
            CurrencyWidth::Narrow => Self::Narrow,
            _ => unimplemented!(),
        }
    }
}

#[doc(hidden)]
pub fn format_currency_to_view<L: Locale>(
    locale: L,
    number: impl NumberFormatterInputFn,
    width: CurrencyWidth,
    currency_code: CurrencyCode,
) -> impl IntoView + Clone {
    let currency_formatter = super::get_currency_formatter(locale, width);

    move || {
        let fixed_dec = number.to_fixed_decimal();
        let currency = currency_formatter.format_fixed_decimal(&fixed_dec, currency_code);
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
    width: CurrencyWidth,
    currency_code: CurrencyCode,
) -> fmt::Result {
    let currency_formatter = super::get_currency_formatter(locale, width);
    let fixed_dec = number.to_fixed_decimal();
    let formatted_currency = currency_formatter.format_fixed_decimal(&fixed_dec, currency_code);
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
    width: CurrencyWidth,
    currency_code: CurrencyCode,
) -> impl Display {
    let currency_formatter = super::get_currency_formatter(locale, width);
    let fixed_dec = number.to_fixed_decimal();
    let currency = currency_formatter.format_fixed_decimal(&fixed_dec, currency_code);
    let mut formatted_currency = String::new();
    currency.write_to(&mut formatted_currency).unwrap();
    formatted_currency
}
