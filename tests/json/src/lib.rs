#![deny(warnings)]

include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));

#[cfg(test)]
mod defaulted;
#[cfg(test)]
mod foreign;
#[cfg(test)]
mod formatting;
#[cfg(test)]
mod plurals;
#[cfg(test)]
mod scoped;
#[cfg(test)]
mod subkeys;
#[cfg(test)]
mod tests;

#[cfg(test)]
mod t_format;
#[cfg(test)]
mod t_plural;
