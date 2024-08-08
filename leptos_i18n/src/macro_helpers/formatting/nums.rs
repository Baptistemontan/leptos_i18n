use core::fmt;

use fixed_decimal::{FixedDecimal, FloatPrecision};
use icu::decimal::FixedDecimalFormatter;
use leptos::{Signal, SignalGet, SignalWith};

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
        FixedDecimal::try_from_f64(self.into(), FloatPrecision::Floating).unwrap()
    }
}

impl IntoFixedDecimal for f64 {
    fn to_fixed_decimal(self) -> FixedDecimal {
        FixedDecimal::try_from_f64(self, FloatPrecision::Floating).unwrap()
    }
}

pub type FixedDecimalSignal = Signal<FixedDecimal>;

pub trait IntoFixedDecimalSignal {
    fn to_signal(self) -> FixedDecimalSignal;
}

impl<T: IntoFixedDecimal> IntoFixedDecimalSignal for T {
    fn to_signal(self) -> FixedDecimalSignal {
        let v = self.to_fixed_decimal();
        Signal::derive(move || v.clone())
    }
}

impl<T: IntoFixedDecimal> IntoFixedDecimalSignal for Signal<T> {
    fn to_signal(self) -> FixedDecimalSignal {
        Signal::derive(move || self.get().to_fixed_decimal())
    }
}

pub fn check_into_fixed_decimal(var: impl IntoFixedDecimalSignal) -> FixedDecimalSignal {
    var.to_signal()
}

pub fn check_into_fixed_decimal_string(var: impl IntoFixedDecimal) -> FixedDecimal {
    var.to_fixed_decimal()
}

pub fn format_number_to_string<L: Locale>(
    locale: L,
    number: Signal<FixedDecimal>,
) -> Signal<String> {
    let formatter =
        FixedDecimalFormatter::try_new(&locale.as_langid().into(), Default::default()).unwrap();

    Signal::derive(move || number.with(|value| formatter.format_to_string(value)))
}

pub fn format_number_to_formatter<L: Locale>(
    locale: L,
    number: &FixedDecimal,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    std::fmt::Display::fmt(
        &FixedDecimalFormatter::try_new(&locale.as_langid().into(), Default::default())
            .unwrap()
            .format(number),
        f,
    )
}
