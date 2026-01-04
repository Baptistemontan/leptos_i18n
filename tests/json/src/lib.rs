#![deny(warnings)]
#![cfg(test)]

include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));

mod components;
mod defaulted;
mod foreign;
mod formatting;
mod plurals;
mod scoped;
mod subkeys;
mod t_format;
mod t_plural;
mod tests;
