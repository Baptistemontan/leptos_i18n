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
    pub use super::parser::locale::{
        RawLocale, RawLocalesOrNamespaces, RawValueOrSubkeys, RawValues,
    };
    pub use super::parser::raw_value::RawValue;
    pub use super::parser::raw_value::component::Component;
    pub use super::parser::raw_value::variable::Variable;
    pub use super::parser::{ParseContext, RawParsedLocales, parse_locales_raw};
}

pub mod extraction {
    pub use super::extractor::defaults::DefaultedLocales;
    pub use super::extractor::values::attributes::{Attribute, AttributeValue, Attributes};
    pub use super::extractor::values::plurals::{PluralForm, PluralForms, PluralRuleType, Plurals};
    pub use super::extractor::values::{Keys, Literal, Value, Values, ValuesOrSubkeys};
    pub use super::extractor::{
        Builder, BuilderId, Builders, CompInfos, InterpolationKeys, Locale, Locales,
        LocalesOrNamespaces, Namespace, ParsedLocales, VarInfos, extract_locales,
    };
}
