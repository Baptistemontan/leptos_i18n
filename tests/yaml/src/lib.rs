#![deny(warnings)]
leptos_i18n::load_locales!();

#[cfg(test)]
mod ranges;

#[cfg(test)]
mod subkeys;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod defaulted;
