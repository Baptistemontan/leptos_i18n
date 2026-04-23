#![forbid(unsafe_code)]
#![deny(warnings)]
#![allow(clippy::too_many_arguments, clippy::result_unit_err)]

pub mod error;
mod extractor;
pub mod formatters;
mod parser;
pub mod utils;

pub mod options {
    pub use super::parser::options::{Config, FileFormat, LocaleName, ParseOptions, parser};
}

pub mod parsing {
    pub use super::parser::{RawParsedLocales, parse_locales_raw};
}

pub mod extraction {
    pub use super::extractor::defaults::DefaultedLocales;
    pub use super::extractor::values::{Keys, Value, Values, ValuesOrSubkeys};
    pub use super::extractor::{Locales, LocalesOrNamespaces, ParsedLocales, extract_locales};
}
