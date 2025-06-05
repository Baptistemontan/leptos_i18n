#![deny(warnings)]
include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));

#[cfg(test)]
mod ranges;

#[cfg(test)]
mod subkeys;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod defaulted;
