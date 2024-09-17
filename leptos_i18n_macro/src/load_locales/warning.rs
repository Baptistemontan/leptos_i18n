use proc_macro::Span;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::utils::key::{Key, KeyPath};
use std::{cell::RefCell, fmt::Display, rc::Rc};

use super::plurals::{PluralForm, PluralRuleType};

#[derive(Debug)]
struct SpannedWarning {
    #[allow(unused)]
    span: Span,
    warning: Warning,
}

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
    UnusedCategory {
        locale: Rc<Key>,
        key_path: KeyPath,
        category: PluralForm,
        rule_type: PluralRuleType,
    },
    NonUnicodePath {
        locale: Rc<Key>,
        namespace: Option<Rc<Key>>,
        path: std::path::PathBuf,
    },
}

thread_local! {
    pub static WARNINGS: RefCell<Vec<SpannedWarning>> = const { RefCell::new(Vec::new()) };
}

pub fn emit_warning(warning: Warning, span: Option<Span>) {
    if !cfg!(feature = "suppress_key_warnings") {
        let span = span.unwrap_or_else(Span::call_site);
        let spanned_warning = SpannedWarning { span, warning };
        WARNINGS.with(|warnings| warnings.borrow_mut().push(spanned_warning));
    }
}

impl Display for Warning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Warning::MissingKey { locale, key_path } => {
                write!(f, "Missing key \"{}\" in locale {:?}", key_path, locale)
            }
            Warning::SurplusKey { locale, key_path } => write!(
                f,
                "Key \"{}\" is present in locale {:?} but not in default locale, it is ignored",
                key_path, locale
            ),
            Warning::UnusedCategory { locale, key_path, category, rule_type } => {
                write!(f, "at key \"{}\", locale {:?} does not use {} category {}, it is still kept but is useless.", key_path, locale, rule_type, category)
            },
            Warning::NonUnicodePath { locale, namespace: None, path } => write!(f, "File path for locale {:?} is not valid Unicode, can't add it to proc macro depedencies. Path: {:?}", locale, path),
            Warning::NonUnicodePath { locale, namespace: Some(ns), path } => write!(f, "File path for locale {:?} in namespace {:?} is not valid Unicode, can't add it to proc macro depedencies. Path: {:?}", locale, ns, path),
        }
    }
}

impl SpannedWarning {
    fn to_fn(&self, index: usize) -> TokenStream {
        let msg = self.warning.to_string();
        let fn_name = format_ident!("w{}", index);
        quote! {
            #[deprecated(note = #msg)]
            fn #fn_name() {
                unimplemented!()
            }
        }
    }

    fn emit(&self) {
        #[cfg(feature = "nightly")]
        {
            use proc_macro::Diagnostic;

            Diagnostic::spanned(
                self.span,
                proc_macro::Level::Warning,
                self.warning.to_string(),
            )
            .emit();
        }
    }
}

fn generate_warnings_inner(warnings: &[SpannedWarning]) -> TokenStream {
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

pub fn generate_warnings() -> Option<TokenStream> {
    WARNINGS.with(|cell| {
        let ws = cell.borrow();
        if cfg!(not(feature = "nightly")) {
            if ws.is_empty() {
                None
            } else {
                Some(generate_warnings_inner(&ws))
            }
        } else {
            for warning in ws.iter() {
                warning.emit();
            }
            None
        }
    })
}
