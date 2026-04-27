use std::{any::Any, hash::Hash, marker::PhantomData};

use leptos::{
    IntoView,
    prelude::{AnyView, IntoAny},
};

use crate::locale_traits::BaseLocale;

// TODO: manual impl of Debug to print key path
#[derive(Debug)]
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

pub struct AnyArgs<L: BaseLocale> {
    args: Box<dyn DynAnyArgs<Locale = L>>,
}

pub trait ArgsBuilder: Copy + 'static {
    type Id: Send + Sync + Copy + Hash + Ord + 'static;
    type Builder: 'static;
    type Locale: BaseLocale;

    fn new() -> Self::Builder;
}

pub trait DowngradableArgBuilder: ArgsBuilder {
    type Downgraded: ArgsBuilder<Locale = Self::Locale, Builder = Self::Builder>;
    const ID: <Self::Downgraded as ArgsBuilder>::Id;
}

pub trait ArgsMarker<B>: ArgsBuilder {
    type Args: Args<Locale = Self::Locale, Id = Self::Id>;
    fn into_args(builder: B) -> Self::Args;
}

pub enum NoArgs {}

#[doc(hidden)]
#[diagnostic::on_unimplemented(message = "TODO")]
pub trait ConstArgsMarker: ArgsMarker<NoArgs, Args: Copy + 'static> {
    const THIS: Self::Args;
}

#[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
#[doc(hidden)]
pub trait IntoViewFuture: Future<Output: IntoView> + 'static {}

impl<F> IntoViewFuture for F where F: Future<Output: IntoView> + 'static {}

pub trait Args: Clone + 'static {
    type Locale: BaseLocale;
    type Id: Send + Sync + Copy + Hash + Ord + 'static;

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn render(self, id: Self::Id, locale: Self::Locale) -> impl IntoViewFuture;

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn render(self, id: Self::Id, locale: Self::Locale) -> impl IntoView;
}

pub trait DowngradableArgs: Args {
    type Downgraded: Args<Locale = Self::Locale>;

    fn downgrade(this: Key<Self>) -> Key<Self::Downgraded>;
}

trait DynAnyArgs: Send + Sync + Any + 'static {
    type Locale: BaseLocale;

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn render(
        self: Box<Self>,
        locale: Self::Locale,
    ) -> core::pin::Pin<Box<dyn Future<Output = AnyView> + 'static>>;

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn render(self: Box<Self>, locale: Self::Locale) -> AnyView;

    fn clone(&self) -> Box<dyn DynAnyArgs<Locale = Self::Locale>>;
}

#[derive(Clone, Copy)]
struct AnyArgsInner<A: Args> {
    id: A::Id,
    args: A,
}

impl<A: Args + Send + Sync> DynAnyArgs for AnyArgsInner<A> {
    type Locale = A::Locale;

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn render(
        self: Box<Self>,
        locale: Self::Locale,
    ) -> core::pin::Pin<Box<dyn Future<Output = AnyView> + 'static>> {
        let fut = async move {
            let AnyArgsInner { id, args } = *self;
            let view = args.render(id, locale).await;
            view.into_any()
        };
        Box::pin(fut)
    }

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
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

impl<L: BaseLocale> Clone for AnyArgs<L> {
    fn clone(&self) -> Self {
        let args = DynAnyArgs::clone(&*self.args);
        AnyArgs { args }
    }
}

impl<L: BaseLocale> Args for AnyArgs<L> {
    type Id = ();
    type Locale = L;

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn render(self, _id: Self::Id, locale: Self::Locale) -> impl IntoViewFuture {
        DynAnyArgs::render(self.args, locale)
    }

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn render(self, _id: Self::Id, locale: Self::Locale) -> impl IntoView {
        DynAnyArgs::render(self.args, locale)
    }
}

impl<L: BaseLocale> DowngradableArgs for AnyArgs<L> {
    type Downgraded = Self;
    fn downgrade(this: Key<Self>) -> Key<Self::Downgraded> {
        this
    }
}

impl<L: BaseLocale> AnyArgs<L> {
    fn from_args<A>(args: A, id: A::Id) -> AnyArgs<L>
    where
        A: Args<Locale = L> + Send + Sync,
    {
        let inner = AnyArgsInner { id, args };
        let boxed = Box::new(inner);
        AnyArgs { args: boxed }
    }
}

impl<B: ArgsBuilder> KeyBuilder<B> {
    #[doc(hidden)]
    pub fn build<F, A>(this: Self, f: F) -> Key<<B as ArgsMarker<A>>::Args>
    where
        F: FnOnce(B::Builder) -> A,
        B: ArgsMarker<A>,
    {
        let builded = f(B::new());
        let args = B::into_args(builded);
        Key { id: this.id, args }
    }

    #[doc(hidden)]
    pub const fn const_build<F, A>(this: Self, _: &F) -> Key<<B as ArgsMarker<NoArgs>>::Args>
    where
        F: FnOnce(B::Builder) -> A,
        B: ArgsMarker<A>,
        B: ConstArgsMarker,
    {
        Key {
            id: this.id,
            args: B::THIS,
        }
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
    pub const fn downgrade(self) -> KeyBuilder<B::Downgraded> {
        KeyBuilder {
            id: B::ID,
            _marker: PhantomData,
        }
    }
}

impl<A: Args> Key<A> {
    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    pub fn render(self, locale: A::Locale) -> impl IntoView {
        A::render(self.args, self.id, locale)
    }

    pub fn downgrade(self) -> Key<A::Downgraded>
    where
        A: DowngradableArgs,
    {
        A::downgrade(self)
    }

    pub fn downgrade_any(self) -> Key<AnyArgs<A::Locale>>
    where
        A: Send + Sync,
    {
        let any = AnyArgs::from_args(self.args, self.id);
        Key { id: (), args: any }
    }

    #[doc(hidden)]
    pub const fn const_into_args_and_id(this: Self) -> (A, A::Id)
    where
        Self: Copy,
    {
        let Self { id, args } = this;
        (args, id)
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

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Literal {
    String(&'static str),
    Signed(i64),
    Unsigned(u64),
    Float(f64),
    Bool(bool),
}

impl Literal {
    pub const fn str(self) -> Option<&'static str> {
        if let Literal::String(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub const fn signed(self) -> Option<i64> {
        if let Literal::Signed(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub const fn unsigned(self) -> Option<u64> {
        if let Literal::Unsigned(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub const fn float(self) -> Option<f64> {
        if let Literal::Float(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub const fn bool(self) -> Option<bool> {
        if let Literal::Bool(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl<B: ArgsBuilder> Clone for KeyBuilder<B> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<B: ArgsBuilder> Copy for KeyBuilder<B> {}

impl<B: ArgsBuilder> PartialEq for KeyBuilder<B> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<B: ArgsBuilder> Eq for KeyBuilder<B> {}

impl<B: ArgsBuilder> PartialOrd for KeyBuilder<B> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<B: ArgsBuilder> Ord for KeyBuilder<B> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl<B: ArgsBuilder> Hash for KeyBuilder<B> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
