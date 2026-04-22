#![forbid(unsafe_code)]
#![deny(warnings)]
#![allow(clippy::too_many_arguments, clippy::result_unit_err)]

pub mod error;
mod extractor;
pub mod formatters;
mod parser;
pub mod utils;

pub use extractor::extract_locales;
pub use parser::parse_locales_raw;

pub mod parsing {
    pub use super::parser::{RawParsedLocales, parse_locales_raw};
}

pub mod extraction {
    pub use super::extractor::{ParsedLocales, defaults::DefaultedLocales, extract_locales};
}
