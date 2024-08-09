use core::fmt;

use fixed_decimal::{FixedDecimal, FloatPrecision};
use icu::decimal::FixedDecimalFormatter;
use leptos::IntoView;

use crate::Locale;

// Format nums
macro_rules! impl_into_fixed_decimal {
    ($ty:ty) => {
        impl IntoFixedDecimal for $ty {
            fn to_fixed_decimal(self) -> FixedDecimal {
                Into::into(self)
            }
        }
    };
    ($ty:ty, $($tt:tt)*) => {
        impl_into_fixed_decimal!($ty);
        impl_into_fixed_decimal!($($tt)*);
    }
}

pub trait IntoFixedDecimal: Clone {
    fn to_fixed_decimal(self) -> FixedDecimal;
}

// T: Into<FixedDecimal>
impl_into_fixed_decimal!(
    usize,
    u8,
    u16,
    u32,
    u64,
    u128,
    isize,
    i8,
    i16,
    i32,
    i64,
    i128,
    FixedDecimal
);

impl IntoFixedDecimal for f32 {
    fn to_fixed_decimal(self) -> FixedDecimal {
        FixedDecimal::try_from_f64(Into::into(self), FloatPrecision::Floating).unwrap()
    }
}

impl IntoFixedDecimal for f64 {
    fn to_fixed_decimal(self) -> FixedDecimal {
        FixedDecimal::try_from_f64(self, FloatPrecision::Floating).unwrap()
    }
}

pub trait FormattedNumber: Clone + 'static {
    fn to_fixed_decimal(&self) -> FixedDecimal;
}

impl<T: IntoFixedDecimal, F: Fn() -> T + Clone + 'static> FormattedNumber for F {
    fn to_fixed_decimal(&self) -> FixedDecimal {
        IntoFixedDecimal::to_fixed_decimal(self())
    }
}

pub fn format_number_to_string<L: Locale>(
    locale: L,
    number: impl FormattedNumber,
) -> impl IntoView {
    let formatter =
        FixedDecimalFormatter::try_new(&locale.as_langid().into(), Default::default()).unwrap();

    move || {
        let value = number.to_fixed_decimal();
        formatter.format_to_string(&value)
    }
}

pub fn format_number_to_formatter<L: Locale>(
    f: &mut fmt::Formatter<'_>,
    locale: L,
    number: impl IntoFixedDecimal,
) -> fmt::Result {
    let num_formatter =
        FixedDecimalFormatter::try_new(&locale.as_langid().into(), Default::default()).unwrap();
    let fixed_dec = number.to_fixed_decimal();
    let formatted_num = num_formatter.format(&fixed_dec);
    std::fmt::Display::fmt(&formatted_num, f)
}
