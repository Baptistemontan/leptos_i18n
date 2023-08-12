use std::collections::HashSet;

use quote::{quote, ToTokens};

use crate::key::Key;

pub enum ValueKind<'a> {
    String(&'a str),
    Variable(Key<'a>),
    Component { key: Key<'a>, inner: Box<Self> },
    Bloc(Vec<Self>),
}

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub enum InterpolateKeyKind<'a, 'b> {
    Variable(&'b Key<'a>),
    Component(&'b Key<'a>),
}

impl<'a> ValueKind<'a> {
    pub fn get_keys_inner<'b>(&'b self, keys: &mut Option<HashSet<InterpolateKeyKind<'a, 'b>>>) {
        match self {
            ValueKind::String(_) => {}
            ValueKind::Variable(key) => {
                keys.get_or_insert_with(HashSet::new)
                    .insert(InterpolateKeyKind::Variable(key));
            }
            ValueKind::Component { key, inner } => {
                keys.get_or_insert_with(HashSet::new)
                    .insert(InterpolateKeyKind::Component(key));
                inner.get_keys_inner(keys);
            }
            ValueKind::Bloc(values) => {
                for value in values {
                    value.get_keys_inner(keys)
                }
            }
        }
    }

    pub fn get_keys<'b>(&'b self) -> Option<HashSet<InterpolateKeyKind<'a, 'b>>> {
        let mut keys = None;
        self.get_keys_inner(&mut keys);
        keys
    }

    pub fn is_string(&self) -> Option<&'a str> {
        match self {
            ValueKind::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn new(value: &'a str) -> Self {
        // look for component
        if let Some(component) = Self::find_component(value) {
            return component;
        }
        // else look for variables
        if let Some(variable) = Self::find_variable(value) {
            return variable;
        }

        // else it's just a string
        ValueKind::String(value)
    }

    fn find_variable(value: &'a str) -> Option<Self> {
        let (before, rest) = value.split_once("{{")?;
        let (ident, after) = rest.split_once("}}")?;

        let ident = Key::try_new(ident)?;

        let before = Self::new(before);
        let after = Self::new(after);
        let this = ValueKind::Variable(ident);

        Some(ValueKind::Bloc(vec![before, this, after]))
    }

    fn find_component(value: &'a str) -> Option<Self> {
        let (before, key, after) = Self::find_opening_tag(value)?;

        let (beetween, after) = Self::find_closing_tag(after, &key)?;

        let before = ValueKind::new(before);
        let beetween = ValueKind::new(beetween);
        let after = ValueKind::new(after);

        let this = ValueKind::Component {
            key,
            inner: beetween.into(),
        };

        Some(ValueKind::Bloc(vec![before, this, after]))
    }

    fn find_closing_tag(value: &'a str, key: &Key) -> Option<(&'a str, &'a str)> {
        let mut indices = None;
        let mut depth = 0;
        for i in value.match_indices('<').map(|x| x.0) {
            let rest = &value[i + 1..];
            if let Some((ident, _)) = rest.split_once('>') {
                if let Some(closing_tag) = ident.trim_start().strip_prefix('/') {
                    if depth == 0 && closing_tag.trim() == key.name {
                        let end_i = i + ident.len() + 2;
                        indices = Some((i, end_i))
                    }
                } else if ident.trim() == key.name {
                    depth += 1;
                }
            }
        }

        let (start, end) = indices?;

        let before = &value[..start];
        let after = &value[end..];

        Some((before, after))
    }

    fn find_opening_tag(value: &'a str) -> Option<(&'a str, Key<'a>, &'a str)> {
        let (before, rest) = value.split_once('<')?;
        let (ident, after) = rest.split_once('>')?;

        let ident = Key::try_new(ident)?;

        Some((before, ident, after))
    }
}

impl<'a, 'b> InterpolateKeyKind<'a, 'b> {
    pub fn as_key(self) -> &'b Key<'a> {
        match self {
            InterpolateKeyKind::Variable(key) | InterpolateKeyKind::Component(key) => key,
        }
    }
}

impl<'a, 'b> ToTokens for InterpolateKeyKind<'a, 'b> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            InterpolateKeyKind::Variable(key) | InterpolateKeyKind::Component(key) => {
                key.to_tokens(tokens)
            }
        }
    }
}

impl<'a> ToTokens for ValueKind<'a> {
    fn to_token_stream(&self) -> proc_macro2::TokenStream {
        match self {
            ValueKind::String("") => quote!(),
            ValueKind::String(s) => quote!(__leptos__::IntoView::into_view(#s, cx),),
            ValueKind::Variable(key) => {
                quote!(__leptos__::IntoView::into_view(core::clone::Clone::clone(&#key), cx),)
            }
            ValueKind::Bloc(values) => quote!(#(#values)*),
            ValueKind::Component { key, inner } => {
                let captured_keys = inner.get_keys().map(|keys| {
                    let keys = keys
                        .into_iter()
                        .map(|key| quote!(let #key = core::clone::Clone::clone(&#key);));
                    quote!(#(#keys)*)
                });

                let f = quote!({
                    #captured_keys
                    move |cx| Into::into(__leptos__::CollectView::collect_view([#inner], cx))
                });
                let boxed_fn = quote!(Box::new(#f));
                quote!(__leptos__::IntoView::into_view(core::clone::Clone::clone(&#key)(cx, #boxed_fn), cx),)
            }
        }
    }

    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        Self::to_token_stream(self).to_tokens(tokens)
    }
}
