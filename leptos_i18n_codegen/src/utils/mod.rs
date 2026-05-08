use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};

#[derive(Debug, Clone)]
pub enum EitherOfWrapper {
    Single,
    Duo,
    Multiple(syn::Ident),
    Nested(Box<Self>),
}

impl EitherOfWrapper {
    #[track_caller]
    pub fn new(size: usize) -> EitherOfWrapper {
        match size {
            0 => {
                unreachable!("EitherOfWrapper requires at least one element.")
            }
            1 => EitherOfWrapper::Single,
            2 => EitherOfWrapper::Duo,
            3..=16 => EitherOfWrapper::Multiple(format_ident!("EitherOf{}", size)),
            17.. => EitherOfWrapper::Nested(Box::new(Self::new(size - 15))),
        }
    }

    pub fn wrap<T: ToTokens>(&self, i: usize, ts: T) -> TokenStream {
        const LETTERS: [char; 16] = [
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P',
        ];
        match self {
            EitherOfWrapper::Single => ts.into_token_stream(),
            EitherOfWrapper::Duo if i == 0 => {
                quote!(__l_i18n_crate::reexports::leptos::either::Either::Left(#ts))
            }
            EitherOfWrapper::Duo => {
                quote!(__l_i18n_crate::reexports::leptos::either::Either::Right(#ts))
            }
            EitherOfWrapper::Multiple(ident) => {
                let variant = format_ident!("{}", LETTERS[i]);
                quote!(__l_i18n_crate::reexports::leptos::either::#ident::#variant(#ts))
            }
            EitherOfWrapper::Nested(last) => match i {
                0..=14 => {
                    let variant = format_ident!("{}", LETTERS[i]);
                    quote!(__l_i18n_crate::reexports::leptos::either::EitherOf16::#variant(#ts))
                }
                15.. => {
                    let variant = format_ident!("{}", LETTERS[15]);
                    let ts = last.wrap(i - 15, ts);
                    quote!(__l_i18n_crate::reexports::leptos::either::EitherOf16::#variant(#ts))
                }
            },
        }
    }
}

pub fn fit_in_leptos_tuple(values: &[TokenStream]) -> TokenStream {
    const TUPLE_MAX_SIZE: usize = 26;
    let values_len = values.len();
    if values_len <= TUPLE_MAX_SIZE {
        quote!((#(#values,)*))
    } else {
        let chunk_size = values_len.div_ceil(TUPLE_MAX_SIZE);
        let values = values.chunks(chunk_size).map(fit_in_leptos_tuple);
        quote!((#(#values,)*))
    }
}

#[derive(Clone)]
pub enum EitherIter<A, B> {
    Iter1(A),
    Iter2(B),
}

impl<T, A: Iterator<Item = T>, B: Iterator<Item = T>> Iterator for EitherIter<A, B> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            EitherIter::Iter1(iter) => iter.next(),
            EitherIter::Iter2(iter) => iter.next(),
        }
    }
}

impl<T, A, B> ExactSizeIterator for EitherIter<A, B>
where
    A: ExactSizeIterator<Item = T>,
    B: ExactSizeIterator<Item = T>,
    Self: Iterator<Item = T>,
{
    fn len(&self) -> usize {
        match self {
            EitherIter::Iter1(iter) => ExactSizeIterator::len(iter),
            EitherIter::Iter2(iter) => ExactSizeIterator::len(iter),
        }
    }
}
