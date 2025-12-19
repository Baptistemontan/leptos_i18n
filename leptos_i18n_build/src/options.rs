//! Codegen options

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use proc_macro2::TokenStream;

/// Options for the leptos_i18n codegen
#[derive(Clone)]
#[non_exhaustive]
pub struct CodegenOptions {
    /// Attributes for the generated module,
    /// usefull to supress some warnings with for exemple `#![allow(missing_docs)]`
    pub top_level_attributes: Option<TokenStream>,
    /// Allow to customize the name of generated .rs file,
    /// "mod.rs" by default.
    pub module_file_name: Cow<'static, Path>,
}

#[allow(clippy::derivable_impls)]
impl Default for CodegenOptions {
    fn default() -> Self {
        CodegenOptions::new()
    }
}

impl CodegenOptions {
    /// Create the default Options for the codegen
    pub fn new() -> Self {
        CodegenOptions {
            top_level_attributes: None,
            module_file_name: "mod.rs".to_cow_path(),
        }
    }

    /// Attributes for the generated module,
    /// usefull to supress some warnings with for exemple `#![allow(missing_docs)]`
    ///
    /// # Example
    ///
    /// ```
    /// # use leptos_i18n_build::options::CodegenOptions;
    /// let attributes = "#![allow(missing_docs)]".parse().unwrap();
    /// let options = CodegenOptions::new().top_level_attributes(Some(attributes));
    /// ```
    pub fn top_level_attributes(self, top_level_attributes: Option<TokenStream>) -> Self {
        Self {
            top_level_attributes,
            ..self
        }
    }

    /// Allow to customize the name of generated .rs file,
    /// "mod.rs" by default.
    pub fn module_file_name(self, module_file_name: impl ToPath) -> Self {
        Self {
            module_file_name: module_file_name.to_cow_path(),
            ..self
        }
    }
}

/// Helper trait to create a `Cow<'static, Path>`.
pub trait ToPath {
    /// Convert Self to a `Cow<'static, Path>`
    fn to_cow_path(self) -> Cow<'static, Path>;
}

impl<T: AsRef<Path>> ToPath for &'static T {
    fn to_cow_path(self) -> Cow<'static, Path> {
        Cow::Borrowed(self.as_ref())
    }
}

impl ToPath for PathBuf {
    fn to_cow_path(self) -> Cow<'static, Path> {
        Cow::Owned(self)
    }
}

impl ToPath for String {
    fn to_cow_path(self) -> Cow<'static, Path> {
        Cow::Owned(self.into())
    }
}
