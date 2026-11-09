//! # The Leptos i18n book
//!
//! The full guide on how to set up and use `leptos_i18n`: configuration, translation file syntax
//! (interpolation, plurals, subkeys, formatters, ...), the translation macros, and more.
//!
//! It is also available online at <https://baptistemontan.github.io/leptos_i18n>.
//!
//! Each chapter is a documentation-only module below, prefixed with its chapter number so they
//! are listed in reading order (sub-chapters keep the number of their markdown file).
//!
//! The markdown sources are in the `book` directory at the root of this crate; when reading the
//! crate source, they can be read directly (start with `book/SUMMARY.md` for the table of contents).
//!
//! Links between chapters point to the markdown files and only resolve in the online book.

#[doc = include_str!("../book/01_introduction.md")]
pub mod ch01_introduction {}

#[doc = include_str!("../book/02_getting_started.md")]
pub mod ch02_getting_started {}

#[doc = include_str!("../book/setting_up/README.md")]
pub mod ch03_setting_up {
    #[doc = include_str!("../book/setting_up/01_configuration.md")]
    pub mod ch01_configuration {}

    #[doc = include_str!("../book/setting_up/02_file_structure.md")]
    pub mod ch02_file_structure {}

    #[doc = include_str!("../book/setting_up/03_namespaces.md")]
    pub mod ch03_namespaces {}

    #[doc = include_str!("../book/setting_up/04_inheritance.md")]
    pub mod ch04_inheritance {}
}

#[doc = include_str!("../book/declare/README.md")]
pub mod ch04_declare {
    #[doc = include_str!("../book/declare/01_key_value.md")]
    pub mod ch01_key_value {}

    #[doc = include_str!("../book/declare/02_interpolation.md")]
    pub mod ch02_interpolation {}

    #[doc = include_str!("../book/declare/03_plurals.md")]
    pub mod ch03_plurals {}

    #[doc = include_str!("../book/declare/04_subkeys.md")]
    pub mod ch04_subkeys {}

    #[doc = include_str!("../book/declare/05_foreign_keys.md")]
    pub mod ch05_foreign_keys {}

    #[doc = include_str!("../book/declare/06_mix_kinds.md")]
    pub mod ch06_mix_kinds {}

    #[doc = include_str!("../book/declare/07_formatters.md")]
    pub mod ch07_formatters {}
}

#[doc = include_str!("../book/usage/README.md")]
pub mod ch05_usage {
    #[doc = include_str!("../book/usage/02_context.md")]
    pub mod ch02_context {}

    #[doc = include_str!("../book/usage/03_subcontext.md")]
    pub mod ch03_subcontext {}

    #[doc = include_str!("../book/usage/04_t_macro.md")]
    pub mod ch04_t_macro {}

    #[doc = include_str!("../book/usage/05_td_macro.md")]
    pub mod ch05_td_macro {}

    #[doc = include_str!("../book/usage/06_td_string_macro.md")]
    pub mod ch06_td_string_macro {}

    #[doc = include_str!("../book/usage/07_router.md")]
    pub mod ch07_router {}

    #[doc = include_str!("../book/usage/08_scoping.md")]
    pub mod ch08_scoping {}

    #[doc = include_str!("../book/usage/09_t_format.md")]
    pub mod ch09_t_format {}

    #[doc = include_str!("../book/usage/10_t_plural.md")]
    pub mod ch10_t_plural {}

    #[doc = include_str!("../book/usage/11_const_access.md")]
    pub mod ch11_const_access {}

    #[doc = include_str!("../book/usage/12_serverfn.md")]
    pub mod ch12_server_functions {}
}

#[doc = include_str!("../book/infos/README.md")]
pub mod ch06_infos {
    #[doc = include_str!("../book/infos/01_locale_resol.md")]
    pub mod ch01_locale_resolution {}
}

#[doc = include_str!("../book/reduce_size/README.md")]
pub mod ch07_reduce_size {
    #[doc = include_str!("../book/reduce_size/01_datagen.md")]
    pub mod ch01_datagen {}

    #[doc = include_str!("../book/reduce_size/02_dynamic_load.md")]
    pub mod ch02_dynamic_load {}
}

#[doc = include_str!("../book/06_features.md")]
pub mod ch08_features {}

#[doc = include_str!("../book/appendix_i18n_ally.md")]
pub mod ch09_appendix_i18n_ally {}

#[doc = include_str!("../book/appendix_custom_formatter.md")]
pub mod ch10_appendix_custom_formatter {}
