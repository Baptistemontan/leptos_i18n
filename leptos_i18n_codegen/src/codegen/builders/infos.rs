use std::collections::{BTreeMap, HashSet};

use leptos_i18n_parser::{
    extraction::{Builder, BuilderId, Builders, CompInfos, InterpolationKeys, VarInfos},
    formatters::VarBound,
    utils::{Key, KeyPath},
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};

use crate::{codegen::docs::gen_fields_docs, utils::EitherIter};

pub enum VarOrComp {
    Var {
        bounds: Vec<VarBound>,
        plural: bool,
    },
    Comp {
        into_view: syn::Ident,
        self_closed: Option<bool>,
    },
}

pub struct Field {
    pub key: Key,
    pub generic: syn::Ident,
    pub var_or_comp: VarOrComp,
}

pub struct BuilderInfos {
    pub id_variants: BTreeMap<KeyPath, syn::Ident>,
    pub docs: String,
    pub name: Key,
    pub fields: Vec<Field>,
}

pub struct BuildersInfos {
    pub markers_field: syn::Ident,
    pub infos: BTreeMap<BuilderId, BuilderInfos>,
}

impl BuildersInfos {
    pub fn new(builders: &Builders, markers_field: syn::Ident, gen_docs: bool) -> Self {
        let infos = builders
            .builders
            .iter()
            .map(|(id, builder)| {
                let infos = BuilderInfos::new(builder, gen_docs);
                (id.clone(), infos)
            })
            .collect();
        BuildersInfos {
            infos,
            markers_field,
        }
    }
}

impl BuilderInfos {
    fn gen_variants(keypath: &KeyPath, variants: &mut HashSet<String>) -> syn::Ident {
        use core::fmt::Write;
        let mut buff = String::new();
        if let Some(ns) = &keypath.namespace {
            write!(&mut buff, "{}_", &ns.ident).unwrap();
        }

        for key in &keypath.path {
            write!(&mut buff, "{}_", &key.ident).unwrap();
        }

        while variants.contains(&buff) {
            buff.push('_');
        }

        let ident = syn::Ident::new(&buff, Span::call_site());

        variants.insert(buff);

        ident
    }

    pub fn new(builder: &Builder, gen_docs: bool) -> Self {
        let mut variants = HashSet::new();
        let id_variants = builder
            .used_by
            .iter()
            .map(|keypath| {
                let ident = Self::gen_variants(keypath, &mut variants);
                (keypath.clone(), ident)
            })
            .collect();

        let fields = Self::make_fields(&builder.keys);

        let docs = if gen_docs {
            let mut docs = String::new();
            gen_fields_docs(&mut docs, &fields).unwrap();
            docs
        } else {
            String::new()
        };

        BuilderInfos {
            name: builder.name.clone(),
            id_variants,
            fields,
            docs,
        }
    }

    fn make_fields(keys: &InterpolationKeys) -> Vec<Field> {
        let vars = keys
            .vars
            .iter()
            .map(|(key, infos)| Field::new_var(key.clone(), infos));

        let comps = keys
            .components
            .iter()
            .map(|(key, infos)| Field::new_comp(key.clone(), infos));

        vars.chain(comps).collect()
    }

    pub fn bounded_generics(&self) -> TokenStream {
        let bounded_generics = self.fields.iter().flat_map(Field::as_bounded_generic);
        quote! {
            #(#bounded_generics,)*
        }
    }

    pub fn bounded_fmt_generics(&self) -> TokenStream {
        let bounded_generics = self.fields.iter().flat_map(Field::as_bounded_fmt_generic);
        quote! {
            #(#bounded_generics,)*
        }
    }

    pub fn generics(&self) -> TokenStream {
        let generics = self.fields.iter().flat_map(Field::as_generics);
        quote! {
            #(#generics,)*
        }
    }

    pub fn struct_fields(&self, markers_field: &syn::Ident) -> TokenStream {
        let fields = self.fields.iter().map(Field::as_struct_field);
        let into_view_markers = self.fields.iter().flat_map(Field::as_into_view_marker);
        quote! {
            {
                pub #markers_field: core::marker::PhantomData<(#(#into_view_markers,)*)>,
                #(
                    pub #fields,
                )*
            }
        }
    }

    pub fn destructure(&self, markers_field: &syn::Ident) -> TokenStream {
        let fields = self.fields.iter().map(|f| &*f.key.ident);
        quote! {
            {
                #markers_field: _,
                #(#fields,)*
            }
        }
    }
}

impl Field {
    fn new_comp(key: Key, infos: &CompInfos) -> Self {
        let into_view = format_ident!("__into_view_{}__", key);
        let var_or_comp = VarOrComp::Comp {
            into_view,
            self_closed: infos.self_closed,
        };
        let generic = format_ident!("__{}__", key);
        Field {
            key,
            var_or_comp,
            generic,
        }
    }

    fn new_var(key: Key, infos: &VarInfos) -> Self {
        let bounds = infos.bounds.iter().cloned().collect::<Vec<_>>();
        let var_or_comp = VarOrComp::Var {
            bounds,
            plural: infos.plural,
        };
        let generic = format_ident!("__{}__", key);
        Field {
            key,
            var_or_comp,
            generic,
        }
    }

    pub fn as_generics(&self) -> impl Iterator<Item = &syn::Ident> {
        let generic = std::iter::once(&self.generic);
        match &self.var_or_comp {
            VarOrComp::Var { .. } => EitherIter::Iter1(generic),
            VarOrComp::Comp { into_view, .. } => {
                EitherIter::Iter2(generic.chain(std::iter::once(into_view)))
            }
        }
    }

    pub fn as_bounded_generic(&self) -> impl Iterator<Item = TokenStream> {
        self.var_or_comp.as_bounded_generic(&self.generic)
    }

    pub fn as_bounded_fmt_generic(&self) -> impl Iterator<Item = TokenStream> {
        self.var_or_comp.as_bounded_fmt_generic(&self.generic)
    }

    pub fn as_struct_field(&self) -> TokenStream {
        let Self {
            key,
            generic,
            var_or_comp: _,
        } = self;
        quote!(#key: #generic)
    }

    pub fn as_into_view_marker(&self) -> Option<&syn::Ident> {
        self.var_or_comp.as_into_view_marker()
    }
}

impl VarOrComp {
    pub fn get_bounded_var_generics(
        generic: &syn::Ident,
        bounds: &[VarBound],
        plural: bool,
    ) -> TokenStream {
        let bounds = bounds.iter().map(VarBound::view_bounds);
        let plural_bound =
            plural.then(|| quote!(__l_i18n_crate::__private::InterpolatePluralCount));
        let bounds = bounds.chain(plural_bound);

        quote!(#generic: 'static + ::core::clone::Clone #(+ #bounds)*)
    }

    pub fn get_bounded_fmt_var_generics(
        generic: &syn::Ident,
        bounds: &[VarBound],
        plural: bool,
    ) -> TokenStream {
        if plural {
            let bounds = bounds.iter().map(VarBound::fmt_bounds);
            quote!(#generic: #(#bounds +)* Clone + Into<__l_i18n_crate::reexports::icu::plurals::PluralOperands>)
        } else {
            let bounds = bounds.iter().map(VarBound::fmt_bounds);
            quote!(#generic: #(#bounds +)*)
        }
    }

    pub fn get_bounded_comp_generics(
        generic: &syn::Ident,
        into_view: &syn::Ident,
        self_closed: Option<bool>,
    ) -> [TokenStream; 2] {
        [
            match self_closed {
                Some(true) => {
                    quote!(#generic: __l_i18n_crate::__private::InterpolateCompSelfClosed<#into_view>)
                }
                Some(false) => {
                    quote!(#generic: __l_i18n_crate::__private::InterpolateComp<#into_view>)
                }
                None => quote!(#generic: __l_i18n_crate::__private::InterpolateDummy<#into_view>),
            },
            quote!(#into_view: __l_i18n_crate::__private::AttributesArgMarker),
        ]
    }

    pub fn get_bounded_fmt_comp_generics(
        generic: &syn::Ident,
        into_view: &syn::Ident,
        _self_closed: Option<bool>,
    ) -> [TokenStream; 2] {
        [
            quote!(#generic: __l_i18n_crate::display::DisplayComponent<#into_view>),
            quote!(#into_view),
        ]
    }

    pub fn as_bounded_generic(&self, generic: &syn::Ident) -> impl Iterator<Item = TokenStream> {
        match &self {
            VarOrComp::Var { bounds, plural } => {
                let ts = Self::get_bounded_var_generics(generic, bounds, *plural);
                EitherIter::Iter1(std::iter::once(ts))
            }
            VarOrComp::Comp {
                into_view,
                self_closed,
            } => {
                let ts = Self::get_bounded_comp_generics(generic, into_view, *self_closed);
                EitherIter::Iter2(ts.into_iter())
            }
        }
    }

    pub fn as_bounded_fmt_generic(
        &self,
        generic: &syn::Ident,
    ) -> impl Iterator<Item = TokenStream> {
        match &self {
            VarOrComp::Var { bounds, plural } => {
                let ts = Self::get_bounded_fmt_var_generics(generic, bounds, *plural);
                EitherIter::Iter1(std::iter::once(ts))
            }
            VarOrComp::Comp {
                into_view,
                self_closed,
            } => {
                let ts = Self::get_bounded_fmt_comp_generics(generic, into_view, *self_closed);
                EitherIter::Iter2(ts.into_iter())
            }
        }
    }

    pub fn as_into_view_marker(&self) -> Option<&syn::Ident> {
        match self {
            VarOrComp::Var {
                bounds: _,
                plural: _,
            } => None,
            VarOrComp::Comp {
                into_view,
                self_closed: _,
            } => Some(into_view),
        }
    }
}
