use proc_macro2::TokenStream;
#[cfg(not(feature = "nightly"))]
use quote::{format_ident, quote};

use super::key::{Key, KeyPath};
use std::{cell::RefCell, fmt::Display, rc::Rc};

#[derive(Debug)]
pub enum Warning {
    MissingKey {
        locale: Rc<Key>,
        key_path: KeyPath,
    },
    SurplusKey {
        locale: Rc<Key>,
        key_path: KeyPath,
    },
    #[cfg(feature = "nightly")]
    NonUnicodePath {
        locale: Rc<Key>,
        namespace: Option<Rc<Key>>,
        path: std::path::PathBuf,
    },
}

thread_local! {
    pub static WARNINGS: RefCell<Vec<Warning>> = const { RefCell::new(Vec::new()) };
}

pub fn emit_warning(warning: Warning) {
    if !cfg!(feature = "suppress_key_warnings") {
        WARNINGS.with(|warnings| warnings.borrow_mut().push(warning));
    }
}

impl Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Warning::MissingKey { locale, key_path } => {
                write!(f, "Missing key {} in locale {:?}", key_path, locale)
            }
            Warning::SurplusKey { locale, key_path } => write!(
                f,
                "Key {} is present in locale {:?} but not in default locale, it is ignored",
                key_path, locale
            ),
            #[cfg(feature = "nightly")]
            Warning::NonUnicodePath { locale, namespace: None, path } => write!(f, "File path for locale {:?} is not valid Unicode, can't add it to proc macro depedencies. Path: {:?}", locale, path),
            #[cfg(feature = "nightly")]
            Warning::NonUnicodePath { locale, namespace: Some(ns), path } => write!(f, "File path for locale {:?} in namespace {:?} is not valid Unicode, can't add it to proc macro depedencies. Path: {:?}", locale, ns, path),
        }
    }
}

impl Warning {
    #[cfg(not(feature = "nightly"))]
    fn to_fn(&self, index: usize) -> TokenStream {
        let msg = self.to_string();
        let fn_name = format_ident!("w{}", index);
        quote! {
            #[deprecated(note = #msg)]
            fn #fn_name() {
                unimplemented!()
            }
        }
    }

    #[cfg(feature = "nightly")]
    fn emit(&self) {
        use proc_macro::{Diagnostic, Span};

        Diagnostic::spanned(
            Span::call_site(),
            proc_macro::Level::Warning,
            self.to_string(),
        )
        .emit();
    }
}

#[cfg(not(feature = "nightly"))]
fn generate_warnings_inner(warnings: &[Warning]) -> TokenStream {
    let warning_fns = warnings.iter().enumerate().map(|(i, w)| w.to_fn(i));

    let fn_calls = (0..warnings.len()).map(|i| {
        let fn_name = format_ident!("w{}", i);
        quote!(#fn_name();)
    });

    quote! {
        #[allow(unused)]
        fn warnings() {
            #(
                #warning_fns
            )*

            #(
                #fn_calls
            )*
        }
    }
}

#[cfg(not(feature = "nightly"))]
pub fn generate_warnings() -> Option<TokenStream> {
    WARNINGS.with(|cell| {
        let ws = cell.borrow();
        if ws.is_empty() {
            None
        } else {
            Some(generate_warnings_inner(&ws))
        }
    })
}

#[cfg(feature = "nightly")]
pub fn generate_warnings() -> Option<TokenStream> {
    WARNINGS.with(|ws| {
        for warning in ws.borrow().iter() {
            warning.emit();
        }
        None
    })
}
