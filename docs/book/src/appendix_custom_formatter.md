# Custom Formatter

If you find the formatters provided by the library lacking some features, you can either:

- Open an issue; we will gladly look into your needs and see if we can accommodate you
- Make your own formatter

This chapter is about how to do the later.

Buckle up, we are going to dig into the internals of the parser/codegen.

## Formatter Trait

The name is misleading, this is'nt the formatter itself but the parser, the trait look like this:

```rust
pub trait Formatter {
    const NAME: &str;

    type Builder;
    type Field<'a>;
    type ToTokens: FormatterToTokens;
    type ParseError: Display;

    fn builder(&self) -> Self::Builder;

    fn parse_field<'a>(&self, field_name: &'a str) -> Result<Self::Field<'a>, Self::ParseError>;

    fn parse_arg(
        &self,
        builder: &mut Self::Builder,
        field: Self::Field<'_>,
        arg: Option<&str>,
    ) -> Result<(), Self::ParseError>;

    fn build(&self, builder: Self::Builder) -> Result<Self::ToTokens, Self::ParseError>;
}
```

When the translations parser finds a formatter syntax (`{{ var, formatter(arg_name: value; no_value_arg)}}`)
it will look for a formatter with a matching name, this is the name you put to `NAME`.

It will then create the builder with the `builder` function.

Then it will ask your parser to validate each arguments, if any,
to do that it will first validate the argument name with `parse_field`.

When the argument is validated it will ask to parse the value (or the lack of), and update the builder with it
by calling `parse_arg`.

The two last steps are repeated for each arguments until no arguments are left to be parsed.

It will then call `build` with the builder to validate it and get the `Self::ToTokens`.

## The `FormatterToTokens` Trait

This is where we touch the internals, this is where you inject what is done with the value, and what the value should be:

```rust
pub trait FormatterToTokens: Any {
    fn to_view(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream;
    fn view_bounds(&self) -> TokenStream;

    fn to_fmt(&self, key: &Key, locale_field: &Key) -> TokenStream;
    fn fmt_bounds(&self) -> TokenStream;
}
```

The `to_*` functions are how the values are used, `key` is the ident of the value, and `locale_field` is the ident of a value of type `i18n::Locale`.

the `*_bounds` functions are how you restrict the type of the values being formatted, those functions must return some traits to bound the value,
for example `Copy + ToString`.

## Implementation Example

Let's implement a simple formatter, it can pad left or right, such that we can have:

```json
{
  "custom_formatter": "{{ var, padding(len: 14) }}",
  "right_padded": "{{ var, padding(len: 23; dir: right) }}"
}
```

first the data representation:

```rust
/// The parser
struct PaddingFormatterParser;

// The builder
#[derive(Debug, Clone, Copy, Default)]
struct PaddingFormatterBuilder {
    direction: Option<PadDirection>,
    total_len: Option<u32>,
}

// The direction to pad
#[derive(Debug, Clone, Copy, Default)]
enum PadDirection {
    #[default]
    Left,
    Right,
}

// The actual formatter
#[derive(Debug, Clone, Copy, Default)]
struct PaddingFormatter {
    direction: PadDirection,
    total_len: u32,
}
```

Now let's implement the `Formatter` trait:

```rust
impl Formatter for PaddingFormatterParser {
    const NAME: &str = "padding";

    type Builder = PaddingFormatterBuilder;
    // using a 'static str for the error for easier example, but this as a drawback of no dynamic errors,
    // for example we can't explain why the len parsing failed, we can only say it failed.
    type ParseError = &'static str;
    // Same here, easier to use a simple str, but an enum would be more suited.
    type Field<'a> = &'a str;
    type ToTokens = PaddingFormatter;

    fn builder(&self) -> Self::Builder {
        PaddingFormatterBuilder::default()
    }

    fn parse_arg_name<'a>(&self, arg_name: &'a str) -> Result<Self::Field<'a>, Self::ParseError> {
        // if the name match one we are expecting, good
        // if not, return an error.
        match arg_name {
            "dir" | "len" => Ok(arg_name),
            // The error message might look simple, but the translations parser already take care
            // of reporting the position, locale, and arg_name of where this error occurs.
            _ => Err("unknown argument name"),
        }
    }

    fn parse_arg(
        &self,
        builder: &mut Self::Builder,
        field: &mut Self::Field<'_>,
        arg: Option<&str>,
    ) -> Result<(), Self::ParseError> {
        // we don't have marker arguments here, so we can unwrap the value
        let Some(arg) = arg else {
            return Err("missing value"); // Same as above, position, arg_name and arg value are reported with the error.
        };
        match field {
            "dir" => {
                // parse the value
                let dir = match arg {
                    "left" => PadDirection::Left,
                    "right" => PadDirection::Right,
                    _ => return Err("unknown value"),
                };
                // check for duplicates
                if builder.direction.replace(dir).is_some() {
                    Err("duplicate argument")
                } else {
                    Ok(())
                }
            }
            "len" => {
                // parse the len
                let Ok(len) = arg.parse() else {
                    return Err("Invalid value"); // due to using 'static str, we can't report the actual error.
                };
                // check duplicates
                if builder.total_len.replace(len).is_some() {
                    Err("duplicate argument")
                } else {
                    Ok(())
                }
            }
            // We already validated the field, we can't get random values here.
            // Wouldn't need this if we used an enum for the fields.
            _ => unreachable!(),
        }
    }

    fn build(&self, builder: Self::Builder) -> Result<Self::ToTokens, Self::ParseError> {
        // check if all required fields are present, and default any missing optional fields.
        match builder {
            PaddingFormatterBuilder {
                direction,
                total_len: Some(total_len),
            } => Ok(PaddingFormatter {
                direction: direction.unwrap_or_default(),
                total_len,
            }),
            PaddingFormatterBuilder {
                direction: _,
                total_len: None,
            } => Err("missing \"len\" argument"),
        }
    }
}
```

Now the `FormatterToTokens` impl:

```rust
impl FormatterToTokens for PaddingFormatter {
    fn fmt_bounds(&self) -> TokenStream {
        quote!(core::fmt::Display)
    }
    fn to_fmt(&self, key: &Key, locale_field: &Key) -> TokenStream {
        // the locale is irrelevant here, we can ignore it.
        let _ = locale_field;

        let fmt_string = match self.direction {
            PadDirection::Left => {
                format!("{{:<{}}}", self.total_len)
            }
            PadDirection::Right => {
                format!("{{:>{}}}", self.total_len)
            }
        };
        // In the codegen for the to string implementation,
        // there will be a `&mut core::fmt::Formatter<'_>` to sink to under the ident `__formatter`
        // and it is expected to return `core::fmt::Result`.
        quote! {
            core::write!(__formatter, #fmt_string, #key)
        }
    }

    // use strings for simplicity
    fn view_bounds(&self) -> TokenStream {
        quote!(core::fmt::Display)
    }

    fn to_view(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {
        // the locale is irrelevant here, we can ignore it.
        let _ = locale_field;

        let fmt_string = match self.direction {
            PadDirection::Left => {
                format!("{{:<{}}}", self.total_len)
            }
            PadDirection::Right => {
                format!("{{:>{}}}", self.total_len)
            }
        };
        // In the codegen for the to into_view implementation,
        // it is expected to return something that implement `IntoView` and is 'static
        quote! {
            std::format!(#fmt_string, #key)
        }
    }
}
```

Now we can do

```rust
let i18n = use_i18n();

t_string!(i18n, custom_formatter, var = 14)
```

But this is still incomplete, this won't compile:

```rust
let i18n = use_i18n();
let value = Signal::new(14);

t!(i18n, custom_formatter, var = || value.get())
```

We expect `var` to be `impl display`, so we will get errors looking like this:

```
error[E0277]: `{closure@src\app.rs:37:47: 37:49}` doesn't implement `std::fmt::Display`
   --> src\app.rs:37:14
    |
 37 |             {t!(i18n, custom_formatter, var = || value.get())}
    |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^--^^^^^^^^
    |              |                                |
    |              |                                consider calling this closure
    |              the trait `std::fmt::Display` is not implemented for closure `{closure@src\app.rs:37:47: 37:49}`
    |
note: required by a bound in `custom_formatter_builder_dummy::builder`
   --> C:\Users\Baptiste\Documents\Code\Rust\leptos_i18n\examples\csr\custom_formatter\target\wasm32-unknown-unknown\debug\build\custom_formatter-dcd7a2e200ee4dc1\out/i18n/mod.rs:403:63
    |
402 |             pub fn builder<
    |                    ------- required by a bound in this associated function
403 |                 __var_var__: 'static + ::core::clone::Clone + core::fmt::Display,
    |                                                               ^^^^^^^^^^^^^^^^^^ required by this bound in `custom_formatter_builder_dummy::builder`
    = note: this error originates in the macro `$crate::__private::macros_reexport::t` which comes from the expansion of the macro `t` (in Nightly builds, run with -Z macro-backtrace for more info)
```

We don't have reactivity, yet.

What we want is to have a bound that accept any functions that take no parameters and return something that implement `Display`,
unfortunately we can't directly address that inside the `fmt_bounds` function...

One way to do that, is to have a custom trait that describe this behavior:

```rust
pub trait ToDisplayFn: 'static + Clone + Send + Sync { // Need to Clone + Send + Sync + 'static
    type Value: Display;
    fn to_value(&self) -> Self::Value;
}

impl<T: Display, F> ToDisplayFn for F
where
    F: Fn() -> T + 'static + Clone + Send + Sync,
{
    type Value = T;
    fn to_value(&self) -> Self::Value {
        self()
    }
}
```

The generated i18n module is created at the crate level, it is not it's own package, so it can access traits, functions, types from the crate,
so we can bound like this:

```rust
fn fmt_bounds(&self) -> TokenStream {
    quote!(crate::ToDisplayFn) // doesn't need to be at the root
}
```

And then emit this code instead:

```rust
fn to_view(&self, key: &syn::Ident, locale_field: &syn::Ident) -> TokenStream {

        // ...

        quote! {
            move || {
                std::format!(#fmt_string, crate::ToDisplayFn::to_value(&#key))
            }
        }
    }
```

And now the above code with the signal will compile fine and be reactive!

## Inject the Formatter

In your `build.rs` file, simply pass your formatter to the `ParseOptions`:

```rust
let options = ParseOptions::new().add_formatter(PaddingFormatterParser);
let cfg = cfg.parse_options(options);
let translations_infos = TranslationsInfos::parse(cfg).unwrap();
```

> Note that `add_formatter` will panic if a formatter with the same name is already present. It also comes already loaded with all built-in formatters.

## Notes

### Disabled Formatters

You can disable a formatter by setting the `DISABLED` constant in the `Formatter` trait (defaults to `None`) with an error message:

```rust
impl FormatterToTokens for PaddingFormatter {
    const DISABLED: Option<&str> = const {
        if SOME_CONDITIONS {
            None
        } else {
            Some("padding formatter is disabled")
        }
    };

    // ...
}
```

This is used internally to still have the builtin formatters active but able to emit diagnostics on how to enable them.

### Continue on Error

When driving the parser, if an error occurs on an argument name parsing, it will stop for that argument and emit an error but continue to the next argument.
If it fails to parse the value for an argument, it will also emit an error and continue to the next argument.
This is done to provide as much diagnostic information as possible and not report only a single error when multiple could be present.
If any error is emitted, it will not call the final `build` method.

### Diagnostics

The parsing is done by the already defined function `parse_with_diagnostics` in the `Formatter` trait, this is the one in charge of driving the parser and report errors,
but it treats every problem as errors, you may want to have warnings or multiple errors, have custom errors, you can overwrite the `parse_with_diagnostics` for that:

```rust
fn parse_with_diagnostics(
        &self,
        locale: &Key, // Name of the current locale being parsed
        key_path: &KeyPath, // path to the current key
        args: &[(&str, Option<&str>)], // args for the formatter
        diag: &Diagnostics, // Diagnostics for error and warnings
    ) -> Option<Self::ToTokens> {
        todo!()
    }
```
You can now have fine-grained control over the parsing and the diagnostics emitted.

If `None` is returned, no error will be emitted and a "dummy" formatter will be set that accepts any value and does nothing, so the error must comes from the diagnostics.
