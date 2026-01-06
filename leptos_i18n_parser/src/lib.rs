#![forbid(unsafe_code)]
#![deny(warnings)]
#![allow(clippy::too_many_arguments)]

pub mod formatters;
pub mod parse_locales;
pub mod utils;
pub use parse_locales::error::Error;
