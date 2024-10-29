# Formatters

For interpolation, every variables (other than `count` for ranges) are expected to be of type `impl IntoView + Clone + 'static`.

But some values have different ways to be represented based on the locale:

- Number
- Date
- Time
- List

You can specify the kind of value you are going to supply like this:

```json
{
  "key": "{{ var, formatter }}"
}
```

Some of the formatters can take arguments to better suits the format you need:

```json
{
  "key": "{{ var, formatter(arg_name: value; arg_name2: value; ...) }}"
}
```

If an argument has a default value, not supplying that argument will make that arg take the default value.

Here is all the formatters:

## Number

```json
{
  "number_formatter": "{{ num, number }}"
}
```

Will format the number based on the locale.
This make the variable needed to be `impl leptos_i18n::formatting::NumberFormatterInputFn`, which is auto implemented for `impl Fn() -> T + Clone + 'static where T: leptos_i18n::formatting::IntoFixedDecimal`.
`IntoFixedDecimal` is a trait to turn a value into a `fixed_decimal::FixedDecimal` which is a type used by `icu` to format numbers. That trait is currently implemented for:

- FixedDecimal
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

> \* Is implemented for convenience, but uses [`FixedDecimal::try_from_f64`](https://docs.rs/fixed_decimal/latest/fixed_decimal/struct.FixedDecimal.html#method.try_from_f64) with the floating precision, you may want to use your own.

The formatter itself does'nt provide formatting options such as maximum significant digits, but those can be customize through `FixedDecimal` before being passed to the formatter.

Enable the "format_nums" feature to use the number formatter.

### Arguments

There are no arguments for this formatter at the moment.

### Example

```rust
use crate::i18n::*;

let i18n = use_i18n();

let num = move || 100_000;

t!(i18n, number_formatter, num);
```

## Date

```json
{
  "date_formatter": "{{ date_var, date }}"
}
```

Will format the date based on the locale.
This make the variable needed to be `impl leptos_i18n::formatting::DateFormatterInputFn`, which is auto implemented for `impl Fn() -> T + Clone + 'static where T: leptos_i18n::formatting::IntoIcuDate`.
`IntoIcuDate` is a trait to turn a value into a `impl icu::datetime::input::DateInput` which is a trait used by `icu` to format dates. The `IntoIcuDate` trait is currently implemented for `T: DateInput<Calendar = AnyCalendar>`.
You can use `icu::datetime::{Date, DateTime}`, or implement that trait for anything you want.

Enable the "format_datetime" feature to use the date formatter.

### Arguments

There is one argument at the moment for the date formatter: `date_length`, which is based on [`icu::datetime::options::length::Date`](https://docs.rs/icu/latest/icu/datetime/options/length/enum.Date.html), that can take 4 values:

- full
- long
- medium (default)
- short

```json
{
  "short_date_formatter": "{{ date_var, date(date_length: short) }}"
}
```

### Example

```rust
use crate::i18n::*;
use leptos_i18n::reexports::icu::calendar::Date;

let i18n = use_i18n();

let date_var = move || Date::try_new_iso_date(1970, 1, 2).unwrap().to_any();

t!(i18n, date_formatter, date_var);
```

## Time

```json
{
  "time_formatter": "{{ time_var, time }}"
}
```

Will format the time based on the locale.
This make the variable needed to be `impl leptos_i18n::formatting::TimeFormatterInputFn`, which is auto implemented for `impl Fn() -> T + Clone + 'static where T: leptos_i18n::formatting::IntoIcuTime`.
`IntoIcuTime` is a trait to turn a value into a `impl icu::datetime::input::TimeInput` which is a trait used by `icu` to format time. The `IntoIcuTime` trait is currently implemented for `T: IsoTimeInput`.
You can use `icu::datetime::{Time, DateTime}`, or implement that trait for anything you want.

Enable the "format_datetime" feature to use the time formatter.

### Arguments

There is one argument at the moment for the time formatter: `time_length`, which is based on [`icu::datetime::options::length::Time`](https://docs.rs/icu/latest/icu/datetime/options/length/enum.Time.html), that can take 4 values:

- full
- long
- medium
- short (default)

```json
{
  "full_time_formatter": "{{ time_var, time(time_length: full) }}"
}
```

### Example

```rust
use crate::i18n::*;
use leptos_i18n::reexports::icu::calendar::Date;

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
This make the variable needed to be `impl leptos_i18n::formatting::DateTimeFormatterInputFn`, which is auto implemented for `impl Fn() -> T + Clone + 'static where T: leptos_i18n::formatting::IntoIcuDateTime`.
`IntoIcuDateTime` is a trait to turn a value into a `impl icu::datetime::input::DateTimeInput` which is a trait used by `icu` to format datetimes. The `IntoIcuDateTime` trait is currently implemented for `T: DateTimeInput<Calendar = AnyCalendar>`.
You can use `icu::datetime::DateTime`, or implement that trait for anything you want.

Enable the "format_datetime" feature to use the datetime formatter.

### Arguments

There is two arguments at the moment for the datetime formatter: `date_length` and `time_length` that behave exactly the same at the one above.

```json
{
  "short_date_long_time_formatter": "{{ datetime_var, datetime(date_length: short; time_length: full) }}"
}
```

### Example

```rust
use crate::i18n::*;
use leptos_i18n::reexports::icu::calendar::DateTime;

let i18n = use_i18n();

let datetime_var = move || {
    let date = Date::try_new_iso_date(1970, 1, 2).unwrap().to_any();
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
This make the variable needed to be `impl leptos_i18n::formatting::ListFormatterInputFn`, which is auto implemented for `impl Fn() -> T + Clone + 'static where T: leptos_i18n::formatting::WriteableList`.
`WriteableList` is a trait to turn a value into a `impl Iterator<Item = impl writeable::Writeable>`.

Enable the "format_list" feature to use the list formatter.

### Arguments

There is two arguments at the moment for the list formatter: `list_type` and `list_length`.

`list_type` takes 3 possible values:

- and
- or
- unit (Default)

`list_length` takes 3 possible values:

- wide (default)
- short
- narrow

See [`Intl.ListFormat`](https://developer.mozilla.org/fr/docs/Web/JavaScript/Reference/Global_Objects/Intl/ListFormat) documentation. `icu` is used to do the formatting but I found the Mozilla doc to have more details.

```json
{
  "short_and_list_formatter": "{{ list_var, list(list_type: and; list_length: short) }}"
}
```

### Example

```rust
use crate::i18n::*;

let i18n = use_i18n();

let list_var = move || ["A", "B", "C"];

t!(i18n, list_formatter, list_var);
```
