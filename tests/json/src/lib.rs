#![cfg(test)]
#![deny(warnings)]

leptos_i18n::load_locales!();

mod defaulted;
mod foreign;
mod plurals;
mod scoped;
mod subkeys;
mod tests;
