#![deny(warnings)]
leptos_i18n::load_locales!();

#[cfg(test)]
mod first_ns;
#[cfg(test)]
mod second_ns;

#[cfg(scoped)]
mod second_ns;
