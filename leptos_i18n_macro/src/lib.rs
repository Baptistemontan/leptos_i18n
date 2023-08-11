pub(crate) mod cfg_file;
pub(crate) mod error;
pub(crate) mod interpolate;
pub(crate) mod key;
pub(crate) mod load_locales;
pub(crate) mod locale;
pub(crate) mod value_kind;

#[proc_macro]
pub fn load_locales(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match load_locales::load_locales_inner(None::<String>) {
        Ok(ts) => ts.into(),
        Err(err) => err.into(),
    }
}
