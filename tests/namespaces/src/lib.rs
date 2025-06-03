#![deny(warnings)]
include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));

#[cfg(test)]
mod first_ns;
#[cfg(test)]
mod scoped;
#[cfg(test)]
mod second_ns;
