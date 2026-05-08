use std::marker::PhantomData;

use super::KeyId;
use crate::locale_traits::BaseLocale;

pub trait Args: Sized {
    type Locale: BaseLocale;
    type Id: KeyId;
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FailedArgs<L, I> {
    pub _marker: PhantomData<(L, I)>,
    pub failed_builder: super::builder::FailedBuilder,
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

impl<L: BaseLocale, I: KeyId> Args for FailedArgs<L, I> {
    type Locale = L;
    type Id = I;
}

impl<L: BaseLocale, I: KeyId> super::view::IntoViewArgs for FailedArgs<L, I> {
    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn render(self, _: Self::Id, _: Self::Locale) -> impl IntoViewFuture {
        async {}
    }

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn render(self, _: Self::Id, _: Self::Locale) -> impl leptos::IntoView + Clone + 'static {}
}

impl<L: BaseLocale, I: KeyId> super::display::DisplayArgs for FailedArgs<L, I> {
    type Data = super::display::DisplayData;

    fn get_data(&self, _: Self::Id, _: Self::Locale) -> Self::Data {
        match self.failed_builder {}
    }

    fn fmt(
        &self,
        _: &mut core::fmt::Formatter<'_>,
        _: Self::Id,
        _: Self::Locale,
        _: &Self::Data,
    ) -> core::fmt::Result {
        match self.failed_builder {}
    }
}

impl<L: BaseLocale, I: KeyId> super::DowngradableArgs for FailedArgs<L, I> {
    type Downgraded = Self;
    fn downgrade(this: super::Key<Self>) -> super::Key<Self::Downgraded> {
        this
    }
}

impl<L: BaseLocale, I: KeyId> super::display::DowngradableDisplayArgs for FailedArgs<L, I> {
    type Downgraded = Self;
    fn downgrade(this: super::Key<Self>) -> super::Key<Self::Downgraded> {
        this
    }
}
