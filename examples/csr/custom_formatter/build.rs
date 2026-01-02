use leptos_i18n_build::{
    formatter::{Formatter, FormatterToTokens, Key},
    ParseOptions, TranslationsInfos,
};
use proc_macro2::TokenStream;
use quote::quote;
use std::path::PathBuf;

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");

    let i18n_mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");

    let options = ParseOptions::new()
        .interpolate_display(true)
        .add_formatter(PaddingFormatterParser);

    let translations_infos = TranslationsInfos::parse(options).unwrap();

    translations_infos.emit_diagnostics();

    translations_infos.rerun_if_locales_changed();

    translations_infos
        .generate_i18n_module(i18n_mod_directory)
        .unwrap();
}

struct PaddingFormatterParser;

#[derive(Debug, Clone, Copy, Default)]
struct PaddingFormatterBuilder {
    direction: Option<PadDirection>,
    total_len: Option<u32>,
}

#[derive(Debug, Clone, Copy, Default)]
enum PadDirection {
    #[default]
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, Default)]
struct PaddingFormatter {
    direction: PadDirection,
    total_len: u32,
}

impl Formatter for PaddingFormatterParser {
    const NAME: &str = "padding";

    type Builder = PaddingFormatterBuilder;
    type ParseError = &'static str;
    type Field<'a> = &'a str;
    type ToTokens = PaddingFormatter;

    fn builder(&self) -> Self::Builder {
        PaddingFormatterBuilder::default()
    }

    fn parse_arg_name<'a>(&self, arg_name: &'a str) -> Result<Self::Field<'a>, Self::ParseError> {
        match arg_name {
            "dir" | "len" => Ok(arg_name),
            _ => Err("unknown argument name"),
        }
    }

    fn parse_arg(
        &self,
        builder: &mut Self::Builder,
        field: Self::Field<'_>,
        arg: Option<&str>,
    ) -> Result<(), Self::ParseError> {
        // panic!("field: {}, value: {:?}", field, arg);
        let Some(arg) = arg else {
            return Err("missing value");
        };
        match field {
            "dir" => {
                let dir = match arg {
                    "left" => PadDirection::Left,
                    "right" => PadDirection::Right,
                    _ => return Err("unknown value"),
                };
                if builder.direction.replace(dir).is_some() {
                    Err("duplicate argument")
                } else {
                    Ok(())
                }
            }
            "len" => {
                let Ok(len) = arg.parse() else {
                    return Err("Invalid value");
                };
                if builder.total_len.replace(len).is_some() {
                    Err("duplicate argument")
                } else {
                    Ok(())
                }
            }
            _ => unreachable!(),
        }
    }

    fn build(&self, builder: Self::Builder) -> Result<Self::ToTokens, Self::ParseError> {
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
            } => Err("missing len argument"),
        }
    }
}

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
        quote!(crate::ToDisplayFn)
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
            move || {
                std::format!(#fmt_string, crate::ToDisplayFn::to_value(&#key))
            }
        }
    }
}
