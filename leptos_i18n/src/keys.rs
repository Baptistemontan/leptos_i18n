use std::{any::Any, hash::Hash, marker::PhantomData};

use leptos::{
    IntoView,
    prelude::{AnyView, IntoAny},
};

use crate::Locale;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KeyBuilder<B: ArgsBuilder> {
    id: B::Id,
    _marker: PhantomData<B>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key<A: Args> {
    id: A::Id,
    args: A,
}

pub type AnyKey<L> = Key<AnyArgs<L>>;

pub struct AnyArgs<L: Locale> {
    args: Box<dyn DynAnyArgs<Locale = L>>,
}

pub trait ArgsBuilder: 'static {
    type Id: Send + Sync + Copy + Hash + Ord + 'static;
    type Builder: 'static;
    type Locale: Locale;

    fn new() -> Self::Builder;
}

pub trait DowngradableArgBuilder: ArgsBuilder {
    type Downgraded: ArgsBuilder<Locale = Self::Locale, Builder = Self::Builder>;

    fn map_id(id: Self::Id) -> <Self::Downgraded as ArgsBuilder>::Id;
}

pub trait ArgsMarker<B>: ArgsBuilder {
    type Args: Args<Locale = Self::Locale, Id = Self::Id>;
    fn into_args(builder: B) -> Self::Args;
}

pub trait Args: Clone + Send + Sync + 'static {
    type Locale: Locale;
    type Id: Send + Sync + Copy + Hash + Ord + 'static;
    type Downgraded: Args<Locale = Self::Locale>;

    fn downgrade(this: Key<Self>) -> Key<Self::Downgraded>;

    fn render(self, id: Self::Id, locale: Self::Locale) -> impl IntoView;
}

trait DynAnyArgs: Send + Sync + Any + 'static {
    type Locale: Locale;

    fn render(self: Box<Self>, locale: Self::Locale) -> AnyView;

    fn clone(&self) -> Box<dyn DynAnyArgs<Locale = Self::Locale>>;
}

#[derive(Clone, Copy)]
struct AnyArgsInner<A: Args> {
    id: A::Id,
    args: A,
}

impl<A: Args> DynAnyArgs for AnyArgsInner<A> {
    type Locale = A::Locale;
    fn render(self: Box<Self>, locale: Self::Locale) -> AnyView {
        let AnyArgsInner { id, args } = *self;
        let view = args.render(id, locale);
        view.into_any()
    }

    fn clone(&self) -> Box<dyn DynAnyArgs<Locale = A::Locale>> {
        let this: Self = <AnyArgsInner<A> as Clone>::clone(self);
        Box::new(this)
    }
}

impl<L: Locale> Clone for AnyArgs<L> {
    fn clone(&self) -> Self {
        let args = DynAnyArgs::clone(&*self.args);
        AnyArgs { args }
    }
}

impl<L: Locale> Args for AnyArgs<L> {
    type Downgraded = Self;
    type Id = ();
    type Locale = L;

    fn downgrade(this: Key<Self>) -> Key<Self::Downgraded> {
        this
    }

    fn render(self, _id: Self::Id, locale: Self::Locale) -> impl IntoView {
        DynAnyArgs::render(self.args, locale)
    }
}

impl<L: Locale> AnyArgs<L> {
    fn from_args<A>(args: A, id: A::Id) -> AnyArgs<L>
    where
        A: Args<Locale = L>,
    {
        let inner = AnyArgsInner { id, args };
        let boxed = Box::new(inner);
        AnyArgs { args: boxed }
    }
}

impl<B: ArgsBuilder> KeyBuilder<B> {
    pub fn build<F, A>(self, f: F) -> Key<<B as ArgsMarker<A>>::Args>
    where
        F: FnOnce(B::Builder) -> A,
        B: ArgsMarker<A>,
    {
        let builded = f(B::new());
        let args = B::into_args(builded);
        Key { id: self.id, args }
    }

    #[doc(hidden)]
    pub const fn from_id(id: B::Id) -> Self {
        KeyBuilder {
            id,
            _marker: PhantomData,
        }
    }

    #[doc(hidden)]
    pub const fn into_id(this: Self) -> B::Id {
        this.id
    }
}

impl<B: DowngradableArgBuilder> KeyBuilder<B> {
    pub fn downgrade(self) -> KeyBuilder<B::Downgraded> {
        let id = B::map_id(self.id);
        KeyBuilder {
            id,
            _marker: PhantomData,
        }
    }
}

impl<A: Args> Key<A> {
    pub fn render(self, locale: A::Locale) -> impl IntoView {
        A::render(self.args, self.id, locale)
    }

    pub fn downgrade(self) -> Key<A::Downgraded> {
        A::downgrade(self)
    }

    pub fn downgrade_any(self) -> Key<AnyArgs<A::Locale>> {
        let any = AnyArgs::from_args(self.args, self.id);
        Key { id: (), args: any }
    }

    #[doc(hidden)]
    pub fn from_args_and_id(args: A, id: A::Id) -> Self {
        Self { id, args }
    }

    #[doc(hidden)]
    pub fn into_args_and_id(this: Self) -> (A, A::Id) {
        let Self { id, args } = this;
        (args, id)
    }
}
