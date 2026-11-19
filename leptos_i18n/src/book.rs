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

/// Declares a documentation-only module for a chapter of the book.
///
/// `book/rustdoc_links.md` is put in front of the chapter: it defines the reference-style links
/// between chapters for rustdoc. A chapter may also include `book/links.md` (the same links, for
/// mdbook), but the first definition of a label wins, so rustdoc uses its own.
macro_rules! chapter {
    ($name:ident, $path:literal $(, { $($sub_chapters:tt)* })?) => {
        #[doc = include_str!("../book/rustdoc_links.md")]
        #[doc = include_str!(concat!("../book/", $path))]
        pub mod $name {
            $($($sub_chapters)*)?
        }
    };
}

chapter!(ch01_introduction, "01_introduction.md");

chapter!(ch02_getting_started, "02_getting_started.md");

chapter!(ch03_setting_up, "setting_up/README.md", {
    chapter!(ch01_configuration, "setting_up/01_configuration.md");
    chapter!(ch02_file_structure, "setting_up/02_file_structure.md");
    chapter!(ch03_namespaces, "setting_up/03_namespaces.md");
    chapter!(ch04_inheritance, "setting_up/04_inheritance.md");
});

chapter!(ch04_declare, "declare/README.md", {
    chapter!(ch01_key_value, "declare/01_key_value.md");
    chapter!(ch02_interpolation, "declare/02_interpolation.md");
    chapter!(ch03_plurals, "declare/03_plurals.md");
    chapter!(ch04_subkeys, "declare/04_subkeys.md");
    chapter!(ch05_foreign_keys, "declare/05_foreign_keys.md");
    chapter!(ch06_mix_kinds, "declare/06_mix_kinds.md");
    chapter!(ch07_formatters, "declare/07_formatters.md");
});

chapter!(ch05_usage, "usage/README.md", {
    chapter!(ch02_context, "usage/02_context.md");
    chapter!(ch03_subcontext, "usage/03_subcontext.md");
    chapter!(ch04_t_macro, "usage/04_t_macro.md");
    chapter!(ch05_td_macro, "usage/05_td_macro.md");
    chapter!(ch06_td_string_macro, "usage/06_td_string_macro.md");
    chapter!(ch07_router, "usage/07_router.md");
    chapter!(ch08_scoping, "usage/08_scoping.md");
    chapter!(ch09_t_format, "usage/09_t_format.md");
    chapter!(ch10_t_plural, "usage/10_t_plural.md");
    chapter!(ch11_const_access, "usage/11_const_access.md");
    chapter!(ch12_server_functions, "usage/12_serverfn.md");
});

chapter!(ch06_infos, "infos/README.md", {
    chapter!(ch01_locale_resolution, "infos/01_locale_resol.md");
});

chapter!(ch07_reduce_size, "reduce_size/README.md", {
    chapter!(ch01_datagen, "reduce_size/01_datagen.md");
    chapter!(ch02_dynamic_load, "reduce_size/02_dynamic_load.md");
});

chapter!(ch08_features, "06_features.md");

chapter!(ch09_appendix_i18n_ally, "appendix_i18n_ally.md");

chapter!(
    ch10_appendix_custom_formatter,
    "appendix_custom_formatter.md"
);
