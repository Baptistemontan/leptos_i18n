pub(crate) mod load_locales;
pub(crate) mod t_macro;

// for deserializing the files custom deserialization is done,
// this is to use serde::de::DeserializeSeed to pass information on what locale or key we are currently at
// and give better information on what went wrong when an error is emitted.

#[proc_macro]
pub fn load_locales(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match load_locales::load_locales(None::<String>) {
        Ok(ts) => ts.into(),
        Err(err) => err.into(),
    }
}

#[proc_macro]
pub fn t(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    t_macro::t_macro(tokens)
}
