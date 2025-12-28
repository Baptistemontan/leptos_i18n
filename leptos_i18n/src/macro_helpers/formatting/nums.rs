use core::fmt::{self, Display};

use fixed_decimal::{Decimal, FloatPrecision, Sign, UnsignedDecimal};
use icu_decimal::options::GroupingStrategy;
use leptos::IntoView;

use crate::Locale;

// Format nums
macro_rules! impl_into_fixed_decimal {
    ($ty:ty) => {
        impl IntoFixedDecimal for $ty {
            fn to_fixed_decimal(self) -> Decimal {
                Into::into(self)
            }
        }
    };
    ($ty:ty, $($tt:tt)*) => {
        impl_into_fixed_decimal!($ty);
        impl_into_fixed_decimal!($($tt)*);
    }
}

/// Marker trait for types that can be turned into a `fixed_decimal::Decimal`.
pub trait IntoFixedDecimal: Clone {
    /// Consume self to produce a `Decimal`.
    fn to_fixed_decimal(self) -> Decimal;
}

// T: Into<Decimal>
impl_into_fixed_decimal!(
    usize, u8, u16, u32, u64, u128, isize, i8, i16, i32, i64, i128, Decimal
);

impl IntoFixedDecimal for f32 {
    fn to_fixed_decimal(self) -> Decimal {
        Decimal::try_from_f64(Into::into(self), FloatPrecision::RoundTrip)
            .expect("A Decimal from a f32")
    }
}

impl IntoFixedDecimal for f64 {
    fn to_fixed_decimal(self) -> Decimal {
        Decimal::try_from_f64(self, FloatPrecision::RoundTrip).expect("A Decimal from a f64")
    }
}

impl IntoFixedDecimal for UnsignedDecimal {
    fn to_fixed_decimal(self) -> Decimal {
        Decimal::new(Sign::None, self)
    }
}

/// Marker trait for types that produce a `Decimal`.
pub trait NumberFormatterInputFn: Clone + Send + Sync + 'static {
    /// Produce a `Decimal`.
    fn to_fixed_decimal(&self) -> Decimal;
}

impl<T: IntoFixedDecimal, F: Fn() -> T + Clone + Send + Sync + 'static> NumberFormatterInputFn
    for F
{
    fn to_fixed_decimal(&self) -> Decimal {
        IntoFixedDecimal::to_fixed_decimal(self())
    }
}

#[doc(hidden)]
pub fn format_number_to_view<L: Locale>(
    locale: L,
    number: impl NumberFormatterInputFn,
    grouping_strategy: GroupingStrategy,
) -> impl IntoView + Clone {
    let num_formatter = super::get_num_formatter(locale, grouping_strategy);

    move || {
        let value = number.to_fixed_decimal();
        num_formatter.format(&value).to_string()
    }
}

#[doc(hidden)]
pub fn format_number_to_formatter<L: Locale>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    number: impl IntoFixedDecimal,
    grouping_strategy: GroupingStrategy,
) -> fmt::Result {
    let num_formatter = super::get_num_formatter(locale, grouping_strategy);
    let fixed_dec = number.to_fixed_decimal();
    let formatted_num = num_formatter.format(&fixed_dec);
    Display::fmt(&formatted_num, f)
}

/// This function is a lie.
/// The only reason it exist is for the `format` macros.
/// It does NOT return a `impl Display` struct with no allocation like the other
/// This directly return a `String` of the formatted num, because borrow issues.
#[doc(hidden)]
pub fn format_number_to_display<L: Locale>(
    locale: L,
    number: impl IntoFixedDecimal,
    grouping_strategy: GroupingStrategy,
) -> impl Display {
    let num_formatter = super::get_num_formatter(locale, grouping_strategy);
    let fixed_dec = number.to_fixed_decimal();
    num_formatter.format(&fixed_dec).to_string()
}
