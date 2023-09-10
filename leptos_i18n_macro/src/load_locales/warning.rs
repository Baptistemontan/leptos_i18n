use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use super::key::{Key, KeyPath};
use std::{cell::RefCell, fmt::Display, rc::Rc};

#[derive(Debug)]
pub enum Warning {
    MissingKey { locale: Rc<Key>, key_path: KeyPath },
    SurplusKey { locale: Rc<Key>, key_path: KeyPath },
}

thread_local! {
    pub static WARNINGS: RefCell<Vec<Warning>> = const { RefCell::new(Vec::new()) };
}

pub fn emit_warning(warning: Warning) {
    WARNINGS.with_borrow_mut(|warnings| warnings.push(warning));
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
        }
    }
}

impl Warning {
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
}

fn generate_warnings_inner(warnings: &[Warning]) -> TokenStream {
    let warning_fns = warnings.iter().enumerate().map(|(i, w)| w.to_fn(i));

    let fn_calls = (0..warnings.len()).map(|i| {
        let fn_name = format_ident!("w{}", i);
        quote!(#fn_name();)
    });

    quote! {
        #[allow(unused)]
        fn emit_warnings() {
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
    WARNINGS.with_borrow(|ws| {
        if ws.is_empty() {
            None
        } else {
            Some(generate_warnings_inner(ws))
        }
    })
}
