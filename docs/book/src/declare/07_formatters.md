# Formatters

For interpolation, every variable (other than `count` for plurals) is expected to be of type `impl IntoView + Clone + 'static`.

However, some values can be represented differently depending on the locale:

- Number
- Currency
- Date
- Time
- List

You can specify the kind of value you are going to supply like this:

```json
{
  "key": "{{ var, formatter }}"
}
```

Some of the formatters can take arguments to better suit your formatting needs:

```json
{
  "key": "{{ var, formatter(arg_name: value; arg_name2: value; ...) }}"
}
```

If an argument has a default value, not supplying that argument will make that arg take the default value.

Here are all the formatters:

## Number

```json
{
  "number_formatter": "{{ num, number }}"
}
```

Will format the number based on the locale.
This means the variable must be `impl leptos_i18n::formatting::NumberFormatterInputFn`, which is automatically implemented for `impl Fn() -> T + Clone + 'static where T: leptos_i18n::formatting::IntoFixedDecimal`.
`IntoFixedDecimal` is a trait to turn a value into a `fixed_decimal::Decimal`, which is a type used by `icu` to format numbers. That trait is currently implemented for:

- Decimal
- UnsignedDecimal
- Unsigned
- usize
- u8
- u16
- u32
- u64
- u128
- isize
- i8
- i16
- i32
- i64
- i128
- f32 \*
- f64 \*

> \* Is implemented for convenience, but uses [`Decimal::try_from_f64`](https://docs.rs/fixed_decimal/latest/fixed_decimal/type.Decimal.html#method.try_from_f64) with the floating precision; you may want to use your own.

The formatter itself doesn’t provide formatting options such as maximum significant digits, but those can be customized through `Decimal` before being passed to the formatter.

Enable the "format_nums" feature to use the number formatter.

### Arguments

There is one argument at the moment for the number formatter: `grouping_strategy`, which is based on [`icu::decimal::options::GroupingStrategy`](https://docs.rs/icu/2.0.0/icu/decimal/options/enum.GroupingStrategy.html), that can take 4 values:

- auto (default)
- never
- always
- min2

### Example

```rust,ignore
use crate::i18n::*;

let i18n = use_i18n();

let num = move || 100_000;

t!(i18n, number_formatter, num);
```

## Currency (Experimental)

```json
{
  "currency_formatter": "{{ num, currency }}"
}
```

Will format the currency based on the locale.
The variable should be the same as [number](#number).

Enable the "format_currency" feature to use the currency formatter.

### Arguments

There are two arguments at the moment for the currency formatter: `width` and `currency_code`, which are based on [`icu::experimental::dimension::currency::options::Width`](https://docs.rs/icu/2.0.0/icu/experimental/dimension/currency/options/enum.Width.html) and [`icu::experimental::dimension::currency::CurrencyCode`](https://docs.rs/icu/2.0.0/icu/experimental/dimension/currency/struct.CurrencyCode.html).

`width` values:

- short (default)
- narrow

`currency_code` value should be a [currency code](https://www.iban.com/currency-codes), such as USD or EUR. USD is the default value.

### Example

```rust,ignore
use crate::i18n::*;

let i18n = use_i18n();

let num = move || 100_000;

t!(i18n, currency_formatter, num);
```

## Date

```json
{
  "date_formatter": "{{ date_var, date }}"
}
```

Will format the date based on the locale.
This means the variable must be `impl leptos_i18n::formatting::DateFormatterInputFn`, which is automatically implemented for `impl Fn() -> T + Clone + 'static where T: leptos_i18n::formatting::IntoIcuDate`.
`IntoIcuDate` is a trait to turn a value into a `impl icu::datetime::input::Date`, which is a trait used by `icu` to format dates. The `IntoIcuDate` trait is currently implemented for `T: ConvertCalendar<Converted<'a> = Date<Ref<'a, AnyCalendar>>>`.
You can use `icu::datetime::input::{Date, DateTime}`, or implement that trait for anything you want.

Enable the "format_datetime" feature to use the date formatter.

### Arguments

`length`, which is based on [`icu::datetime::options::Length`](https://docs.rs/icu/2.0.0/icu/datetime/options/enum.Length.html), that can take 3 values:

- long
- medium (default)
- short

`alignment`, which is based on [`icu::datetime::options::Alignment`](https://docs.rs/icu/2.0.0/icu/datetime/options/enum.Alignment.html), that can take 2 values:

- auto (default)
- column

`time_precision`, which is based on [`icu::datetime::options::TimePrecision`](https://docs.rs/icu/2.0.0/icu/datetime/options/enum.TimePrecision.html), that can take 13 values:

- hour
- minute
- second (default)
- subsecond_s1,
- subsecond_s2,
- subsecond_s3,
- subsecond_s4,
- subsecond_s5,
- subsecond_s6,
- subsecond_s7,
- subsecond_s8,
- subsecond_s9,
- minute_optional,

`year_style`, which is based on [`icu::datetime::options::YearStyle`](https://docs.rs/icu/2.0.0/icu/datetime/options/enum.YearStyle.html), that can take 3 values:

- auto
- full
- with_era

```json
{
  "short_date_formatter": "{{ date_var, date(length: short) }}"
}
```

### Example

```rust,ignore
use crate::i18n::*;
use leptos_i18n::reexports::icu::datetime::input::Date;

let i18n = use_i18n();

let date_var = move || Date::try_new_iso(1970, 1, 2).unwrap().to_any();

t!(i18n, date_formatter, date_var);
```

## Time

```json
{
  "time_formatter": "{{ time_var, time }}"
}
```

Will format the time based on the locale.
This means the variable must be `impl leptos_i18n::formatting::TimeFormatterInputFn`, which is automatically implemented for `impl Fn() -> T + Clone + 'static where T: leptos_i18n::formatting::IntoIcuTime`.
`IntoIcuTime` is a trait to turn a value into a `impl icu::datetime::input::Time`, which is a trait used by `icu` to format time. The `IntoIcuTime` trait is currently implemented for `T: ConvertCalendar<Converted<'a> = Time> + InFixedCalendar<()> + AllInputMarkers<fieldsets::T>`.
You can use `icu::datetime::input::{Time, DateTime}`, or implement that trait for anything you want.

Enable the "format_datetime" feature to use the time formatter.

### Arguments

`length`, which is based on [`icu::datetime::options::Length`](https://docs.rs/icu/2.0.0/icu/datetime/options/enum.Length.html), that can take 3 values:

- long
- medium (default)
- short

`alignment`, which is based on [`icu::datetime::options::Alignment`](https://docs.rs/icu/2.0.0/icu/datetime/options/enum.Alignment.html), that can take 2 values:

- auto (default)
- column

`time_precision`, which is based on [`icu::datetime::options::TimePrecision`](https://docs.rs/icu/2.0.0/icu/datetime/options/enum.TimePrecision.html), that can take 13 values:

- hour
- minute
- second (default)
- subsecond_s1,
- subsecond_s2,
- subsecond_s3,
- subsecond_s4,
- subsecond_s5,
- subsecond_s6,
- subsecond_s7,
- subsecond_s8,
- subsecond_s9,
- minute_optional,

```json
{
  "full_time_formatter": "{{ time_var, time(length: long) }}"
}
```

### Example

```rust,ignore
use crate::i18n::*;
use leptos_i18n::reexports::icu::datetime::input::Time;

let i18n = use_i18n();

let time_var = move || Time::try_new(14, 34, 28, 0).unwrap();

t!(i18n, time_formatter, time_var);
```

## DateTime

```json
{
  "datetime_formatter": "{{ datetime_var, datetime }}"
}
```

Will format the datetime based on the locale.
This means the variable must be `impl leptos_i18n::formatting::DateTimeFormatterInputFn`, which is automatically implemented for `impl Fn() -> T + Clone + 'static where T: leptos_i18n::formatting::IntoIcuDateTime`.
`IntoIcuDateTime` is a trait to turn a value into a `impl icu::datetime::input::DateTime` which is a trait used by `icu` to format datetimes. The `IntoIcuDateTime` trait is currently implemented for `T: ConvertCalendar<Converted<'a> = DateTime<Ref<'a, AnyCalendar>>>`.
You can use `icu::datetime::input::DateTime`, or implement that trait for anything you want.

Enable the "format_datetime" feature to use the datetime formatter.

### Arguments

There are four arguments at the moment for the datetime formatter: `length`, `alignment`, `time_precision` and `year_style`, which behave exactly the same as the ones above.

```json
{
  "short_date_long_time_formatter": "{{ datetime_var, datetime(length: short; time_precision: minute) }}"
}
```

### Example

```rust,ignore
use crate::i18n::*;
use leptos_i18n::reexports::icu::datetime::input::{Date, DateTime, Time};

let i18n = use_i18n();

let datetime_var = move || {
    let date = Date::try_new_iso(1970, 1, 2).unwrap().to_any();
    let time = Time::try_new(14, 34, 28, 0).unwrap();
    DateTime::new(date, time)
};

t!(i18n, datetime_formatter, datetime_var);
```

## List

```json
{
  "list_formatter": "{{ list_var, list }}"
}
```

Will format the list based on the locale.
This means the variable must be `impl leptos_i18n::formatting::ListFormatterInputFn`, which is automatically implemented for `impl Fn() -> T + Clone + 'static where T: leptos_i18n::formatting::WriteableList`.
`WriteableList` is a trait to turn a value into an `impl Iterator<Item = impl writeable::Writeable>`.

Enable the "format_list" feature to use the list formatter.

### Arguments

There are two arguments at the moment for the list formatter: `list_type` and `list_length`.

`list_type` takes 3 possible values:

- and
- or
- unit (default)

`list_length` takes 3 possible values:

- wide (default)
- short
- narrow

See the [`Intl.ListFormat`](https://developer.mozilla.org/fr/docs/Web/JavaScript/Reference/Global_Objects/Intl/ListFormat) documentation. `icu` is used to do the formatting, but I found the Mozilla doc to have more details.

```json
{
  "short_and_list_formatter": "{{ list_var, list(list_type: and; list_length: short) }}"
}
```

### Example

```rust,ignore
use crate::i18n::*;

let i18n = use_i18n();

let list_var = move || ["A", "B", "C"];

t!(i18n, list_formatter, list_var);
```

## Notes

Formatters _cannot_ be used inside component attributes, this is **_NOT_** allowed:

```json
{
  "highlight_me": "highlight <b id={{ id, number }}>me</b>"
}
```
