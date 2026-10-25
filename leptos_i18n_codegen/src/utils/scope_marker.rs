use proc_macro2::TokenStream;
use quote::quote;

pub fn key_to_scope_marker(key: &str, crate_path: &syn::Path) -> TokenStream {
    let iter = key.chars();
    quote!((#(#crate_path::scopes::markers::Marker<#iter>,)*))
}
