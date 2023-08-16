// use quote::quote;

// pub enum Error {}

// pub type Result<T, E = Error> = std::result::Result<T, E>;

// impl ToString for Error {
//     fn to_string(&self) -> String {
//         todo!()
//     }
// }

// impl From<Error> for proc_macro::TokenStream {
//     fn from(value: Error) -> Self {
//         let error = value.to_string();
//         quote!(compile_error!(#error)).into()
//     }
// }
