use std::marker::PhantomData;

use leptos::IntoView;
use leptos::prelude::{AnyView, IntoAny};

use crate::locale_traits::BaseLocale;

use super::Key;
use super::args::{AnyArgsId, AnyArgsInner, Args};
use super::builder::ArgsBuilder;

#[doc(hidden)]
#[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
pub trait IntoViewFuture: Future<Output: IntoView + Clone + 'static> + 'static {}

#[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
impl<F> IntoViewFuture for F where F: Future<Output: IntoView + Clone + 'static> + 'static {}

pub trait IntoViewArgsMarker<B>: ArgsBuilder {
    type Args: IntoViewArgs<Locale = Self::Locale, Id = Self::Id>;
    fn into_args(builder: B) -> Self::Args;
}

pub trait IntoViewArgs: Args {
    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn render(self, id: Self::Id, locale: Self::Locale) -> impl IntoViewFuture;

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn render(self, id: Self::Id, locale: Self::Locale) -> impl IntoView + Clone + 'static;
}

pub trait DowngradableArgs: IntoViewArgs {
    type Downgraded: IntoViewArgs<Locale = Self::Locale>;

    fn downgrade(this: Key<Self>) -> Key<Self::Downgraded>;
}

pub struct AnyIntoViewArgs<L: BaseLocale> {
    args: Box<dyn DynAnyIntoViewArgs<Locale = L>>,
}

impl<L: BaseLocale> Clone for AnyIntoViewArgs<L> {
    fn clone(&self) -> Self {
        let args = DynAnyIntoViewArgs::clone(&*self.args);
        AnyIntoViewArgs { args }
    }
}

impl<L: BaseLocale> Args for AnyIntoViewArgs<L> {
    type Id = AnyArgsId;
    type Locale = L;
}

impl<L: BaseLocale> IntoViewArgs for AnyIntoViewArgs<L> {
    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn render(self, _id: Self::Id, locale: Self::Locale) -> impl IntoViewFuture {
        async move {
            let clonable_view = DynAnyIntoViewArgs::render(self.args, locale).await;
            move || {
                let view = &clonable_view;
                DynClonableAnyView::as_any_view(&*view.0)
            }
        }
    }

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn render(self, _id: Self::Id, locale: Self::Locale) -> impl IntoView + Clone + 'static {
        let clonable_view = DynAnyIntoViewArgs::render(self.args, locale);
        move || {
            let view = &clonable_view;
            DynClonableAnyView::as_any_view(&*view.0)
        }
    }
}

impl<L: BaseLocale> DowngradableArgs for AnyIntoViewArgs<L> {
    type Downgraded = Self;
    fn downgrade(this: Key<Self>) -> Key<Self::Downgraded> {
        this
    }
}
impl<L: BaseLocale> AnyIntoViewArgs<L> {
    pub fn from_args<A>(args: A, id: A::Id) -> Self
    where
        A: IntoViewArgs<Locale = L> + Clone + Send + Sync + 'static,
    {
        let inner = AnyArgsInner {
            id,
            args,
            data_marker: PhantomData,
        };
        let boxed = Box::new(inner);
        AnyIntoViewArgs { args: boxed }
    }
}

trait DynAnyIntoViewArgs: Send + Sync + 'static {
    type Locale: BaseLocale;

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn render(
        self: Box<Self>,
        locale: Self::Locale,
    ) -> core::pin::Pin<Box<dyn Future<Output = ClonableAnyView> + 'static>>;

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn render(self: Box<Self>, locale: Self::Locale) -> ClonableAnyView;

    fn clone(&self) -> Box<dyn DynAnyIntoViewArgs<Locale = Self::Locale>>;
}

impl<A: IntoViewArgs + Send + Sync + Clone + 'static> DynAnyIntoViewArgs for AnyArgsInner<A> {
    type Locale = A::Locale;

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn render(
        self: Box<Self>,
        locale: Self::Locale,
    ) -> core::pin::Pin<Box<dyn Future<Output = ClonableAnyView> + 'static>> {
        let fut = async move {
            let AnyArgsInner {
                id,
                args,
                data_marker: PhantomData,
            } = *self;
            let view = args.render(id, locale).await;
            ClonableAnyView(Box::new(view))
        };
        Box::pin(fut)
    }

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn render(self: Box<Self>, locale: Self::Locale) -> ClonableAnyView {
        let AnyArgsInner {
            id,
            args,
            data_marker: _,
        } = *self;
        let view = args.render(id, locale);
        ClonableAnyView(Box::new(view))
    }

    fn clone(&self) -> Box<dyn DynAnyIntoViewArgs<Locale = A::Locale>> {
        let this: Self = <AnyArgsInner<A> as Clone>::clone(self);
        Box::new(this)
    }
}

trait DynClonableAnyView: 'static + Send {
    fn as_any_view(&self) -> AnyView;

    fn clone(&self) -> Box<dyn DynClonableAnyView>;
}

impl<T> DynClonableAnyView for T
where
    T: IntoView + Clone + Send + 'static,
{
    fn as_any_view(&self) -> AnyView {
        IntoAny::into_any(self.clone())
    }

    fn clone(&self) -> Box<dyn DynClonableAnyView> {
        Box::new(self.clone())
    }
}

struct ClonableAnyView(Box<dyn DynClonableAnyView>);

impl Clone for ClonableAnyView {
    fn clone(&self) -> Self {
        let inner = DynClonableAnyView::clone(&*self.0);
        Self(inner)
    }
}
