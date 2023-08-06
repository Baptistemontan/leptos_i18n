mod load_locales;

#[proc_macro]
pub fn load_locales(_tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match load_locales::load_locales_inner(None::<String>) {
        Ok(ts) => ts.into(),
        Err(err) => err.into(),
    }
}
