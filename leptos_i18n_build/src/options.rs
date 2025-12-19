//! Codegen options

use std::path::Path;

use proc_macro2::TokenStream;

/// Options for the leptos_i18n codegen
#[derive(Clone)]
#[non_exhaustive]
pub struct CodegenOptions<'a> {
    /// Attributes for the generated module,
    /// usefull to supress some warnings with for exemple `#![allow(missing_docs)]`
    pub top_level_attributes: Option<TokenStream>,
    /// Allow to customize the name of generated .rs file,
    /// "mod.rs" by default.
    pub module_file_name: &'a Path,
}

#[allow(clippy::derivable_impls)]
impl<'a> Default for CodegenOptions<'a> {
    fn default() -> Self {
        CodegenOptions::new()
    }
}

const DEFAULT_FILE_NAME: &str = "mod.rs";

impl<'a> CodegenOptions<'a> {
    /// Create the default Options for the codegen
    pub fn new() -> Self {
        CodegenOptions {
            top_level_attributes: None,
            module_file_name: DEFAULT_FILE_NAME.as_ref(),
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
    pub fn module_file_name(self, module_file_name: &'a impl AsRef<Path>) -> Self {
        Self {
            module_file_name: module_file_name.as_ref(),
            ..self
        }
    }
}
