use leptos_i18n_parser::formatters::VarBound;
use proc_macro2::TokenStream;
use quote::quote;

use crate::codegen::builders::infos::{BuilderInfos, Field, VarOrComp};

pub fn gen_builder(infos: &BuilderInfos, markers_field: &syn::Ident) -> TokenStream {
    let empty = {
        let iter = infos.fields.iter().map(|_| quote!(()));

        quote!((#(#iter,)*))
    };

    let methods = gen_fields_methods(&infos.fields);

    let generics = infos.generics();

    let destructure = infos.fields.iter().map(|field| {
        let key = &*field.key.ident;
        match field.var_or_comp {
            VarOrComp::Var { .. } => quote!((#key,)),
            VarOrComp::Comp { .. } => quote!(((#key, _),)),
        }
    });

    let constructed = infos.fields.iter().map(|field| &*field.key.ident);

    let build_right_generics = infos.fields.iter().map(|field| {
        let generic = &field.generic;
        match &field.var_or_comp {
            VarOrComp::Var { .. } => quote!((#generic,)),
            VarOrComp::Comp { into_view, .. } => {
                quote!(((#generic, core::marker::PhantomData<#into_view>),))
            }
        }
    });

    quote! {
        pub struct Builder<__Marker, Fields = #empty>(Fields, core::marker::PhantomData<__Marker>);

        impl<__Marker> Builder<__Marker> {
            pub const fn __new() -> Self {
                Builder(#empty, core::marker::PhantomData)
            }
        }

        #(
            #methods
        )*


        impl<__Marker, #generics> Builder<__Marker, (#(#build_right_generics,)*)> {
            pub fn build(self) -> BuildedArgs<#generics> {
                let (#(#destructure,)*) = self.0;
                BuildedArgs {
                    #markers_field: core::marker::PhantomData,
                    #(#constructed,)*
                }
            }
        }

    }
}

fn gen_fields_methods(fields: &[Field]) -> impl Iterator<Item = TokenStream> {
    iter_fields(fields).map(|(before, field, after)| gen_methods(field, before, after))
}

fn gen_var_methods(
    field: &Field,
    bounds: &[VarBound],
    plural: bool,
    before: &[Field],
    after: &[Field],
) -> TokenStream {
    let key = &field.key;
    let key_generic = &field.generic;

    let var_name = key
        .name
        .strip_prefix("var_")
        .expect("variable keys must start with var_");

    let repeated_message = format!("Repeated variable {var_name}");
    let missing_message = format!("Missing variable {var_name}");

    let bounded_generics = VarOrComp::get_bounded_var_generics(&field.generic, bounds, plural);

    let constructed_out_generics = quote! {
        #key_generic
    };

    let constructed = quote! {
        #key
    };

    gen_methods_inner(
        field,
        before,
        after,
        &constructed,
        &constructed_out_generics,
        &bounded_generics,
        &repeated_message,
        &missing_message,
    )
}

fn gen_comp_methods(
    field: &Field,
    into_view: &syn::Ident,
    self_closed: bool,
    before: &[Field],
    after: &[Field],
) -> TokenStream {
    let key = &field.key;
    let key_generic = &field.generic;

    let var_name = key
        .name
        .strip_prefix("comp_")
        .expect("components keys must start with var_");

    let repeated_message = format!("Repeated component {var_name}");
    let missing_message = format!("Missing component {var_name}");

    let [bounded_generics, into_view_bounded_generics] =
        VarOrComp::get_bounded_comp_generics(&field.generic, into_view, self_closed);
    let bounded_generics = quote! {
        #bounded_generics, #into_view_bounded_generics
    };

    let constructed_out_generics = quote! {
        (#key_generic, core::marker::PhantomData<#into_view>)
    };

    let constructed = quote! {
        (#key, core::marker::PhantomData)
    };

    gen_methods_inner(
        field,
        before,
        after,
        &constructed,
        &constructed_out_generics,
        &bounded_generics,
        &repeated_message,
        &missing_message,
    )
}

fn gen_methods_inner(
    field: &Field,
    before: &[Field],
    after: &[Field],
    constructed: &TokenStream,
    constructed_out_generics: &TokenStream,
    bounded_generics: &TokenStream,
    repeated_message: &str,
    missing_message: &str,
) -> TokenStream {
    let before_generics = before.iter().map(|field| &field.generic);
    let after_generics = after.iter().map(|field| &field.generic);
    let generics = {
        let iter = before_generics.clone().chain(after_generics.clone());
        quote!(#(#iter,)*)
    };

    let key = &field.key;
    let key_generic = &field.generic;

    let destructured_before = before.iter().map(|f| &*f.key.ident);
    let destructured_after = after.iter().map(|f| &*f.key.ident);

    let destructured = {
        let (destructured_before, destructured_after) =
            (destructured_before.clone(), destructured_after.clone());

        quote! {
            let (#(#destructured_before,)* (), #(#destructured_after,)*) = self.0;
        }
    };

    let destructured_dup = quote! {
        let (#(#destructured_before,)* (_,), #(#destructured_after,)*) = self.0;
    };

    let construction = {
        let iter = before
            .iter()
            .chain(Some(field))
            .chain(after)
            .map(|f| &*f.key.ident);

        quote! {
            Builder((#(#iter,)*), core::marker::PhantomData)
        }
    };

    let right_generics = {
        let (before_generics, after_generics) = (before_generics.clone(), after_generics.clone());

        quote! {
            ( #(#before_generics,)* (), #(#after_generics,)*)
        }
    };

    let dup_right_generics = {
        let (before_generics, after_generics) = (before_generics.clone(), after_generics.clone());

        quote! {
            ( #(#before_generics,)* (__Dup__,), #(#after_generics,)*)
        }
    };

    let build_right_generics = {
        let (before_generics, after_generics) = (before_generics.clone(), after_generics.clone());

        quote! {
            ( #((#before_generics,),)* (), #(#after_generics,)*)
        }
    };

    let constructed_out_generics = {
        quote! {
            ( #(#before_generics,)* (#constructed_out_generics,), #(#after_generics,)*)
        }
    };

    quote! {
        impl<#generics> Builder<__IntoViewMarker, #right_generics> {
            pub fn #key<#bounded_generics>(self, #key: #key_generic) -> Builder<__IntoViewMarker, #constructed_out_generics> {
                let #key = (#constructed,);
                #destructured
                #construction
            }
        }

        impl<__Dup__, #generics> Builder<__IntoViewMarker, #dup_right_generics> {
            #[deprecated(note = #repeated_message)]
            pub fn #key<#bounded_generics>(self, #key: #key_generic) -> Builder<__IntoViewMarker, #constructed_out_generics> {
                let #key = (#constructed,);
                #destructured_dup
                #construction
            }
        }

        impl<__Marker, #generics> Builder<__Marker, #build_right_generics> {
            #[deprecated(note = #missing_message)]
            pub fn build(self) -> ! {
                panic!()
            }
        }

    }
}

fn gen_methods(field: &Field, before: &[Field], after: &[Field]) -> TokenStream {
    match &field.var_or_comp {
        VarOrComp::Var { bounds, plural } => gen_var_methods(field, bounds, *plural, before, after),
        VarOrComp::Comp {
            into_view,
            self_closed,
        } => gen_comp_methods(field, into_view, *self_closed, before, after),
    }
}

fn iter_fields(fields: &[Field]) -> impl Iterator<Item = (&[Field], &Field, &[Field])> {
    fields.iter().enumerate().map(|(i, field)| {
        let before = &fields[..i];
        let after = &fields[i + 1..];
        (before, field, after)
    })
}
