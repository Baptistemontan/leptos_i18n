use std::marker::PhantomData;

use super::KeyId;
use crate::locale_traits::BaseLocale;

pub trait Args: Sized {
    type Locale: BaseLocale;
    type Id: KeyId;
}

#[derive(Clone, Copy)]
pub(crate) struct AnyArgsInner<A: Args, Data = ()> {
    pub id: A::Id,
    pub args: A,
    pub data_marker: PhantomData<Data>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnyArgsId(pub &'static str);

impl KeyId for AnyArgsId {
    fn key(self) -> &'static str {
        self.0
    }
}
