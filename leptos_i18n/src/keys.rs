use std::{fmt::Display, hash::Hash, marker::PhantomData};

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

pub type AnyKey<'a, L> = Key<AnyIntoViewArgs<'a, L>>;

pub struct AnyIntoViewArgs<'a, L: BaseLocale> {
    args: Box<dyn DynAnyIntoViewArgs<'a, Locale = L>>,
}

pub struct AnyDisplayArgs<'a, L: BaseLocale> {
    args: Box<dyn DynAnyDisplayArgs<'a, Locale = L>>,
}

pub trait ArgsBuilder: Copy + 'static {
    type Id: Send + Sync + Copy + Hash + Ord + 'static;
    type Builder: 'static;
    type Locale: BaseLocale;

    fn new() -> Self::Builder;
}

pub trait DisplayArgsBuilder: ArgsBuilder {
    type DisplayBuilder;

    fn new_display() -> Self::DisplayBuilder;
}

pub trait DowngradableArgBuilder: ArgsBuilder {
    type Downgraded: ArgsBuilder<Locale = Self::Locale, Builder = Self::Builder>;
    const ID: <Self::Downgraded as ArgsBuilder>::Id;
}

pub trait IntoViewArgsMarker<B>: ArgsBuilder {
    type Args: IntoViewArgs<Locale = Self::Locale, Id = Self::Id>;
    fn into_args(builder: B) -> Self::Args;
}

pub trait DisplayArgsMarker<B>: ArgsBuilder {
    type Args: DisplayArgs<Locale = Self::Locale, Id = Self::Id>;
    fn into_args(builder: B) -> Self::Args;
}

pub enum NoArgs {}

#[doc(hidden)]
pub trait ConstArgsMarker: ArgsBuilder {
    type ConstBuilder;
    type Builded;
    type Args: Args<Locale = Self::Locale, Id = Self::Id> + Copy + 'static;
    const THIS: Self::Args;
}

#[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
#[doc(hidden)]
pub trait IntoViewFuture: Future<Output: IntoView> + 'static {}

#[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
impl<F> IntoViewFuture for F where F: Future<Output: IntoView> + 'static {}

pub trait Args: Sized {
    type Locale: BaseLocale;
    type Id: Send + Sync + Copy + Hash + Ord + 'static;
}

pub trait IntoViewArgs: Args {
    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn render(self, id: Self::Id, locale: Self::Locale) -> impl IntoViewFuture;

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn render(self, id: Self::Id, locale: Self::Locale) -> impl IntoView;
}

pub trait DisplayArgs: Args {
    fn to_display(self, id: Self::Id, locale: Self::Locale) -> impl Display;
}

pub trait DowngradableArgs: IntoViewArgs {
    type Downgraded: IntoViewArgs<Locale = Self::Locale>;

    fn downgrade(this: Key<Self>) -> Key<Self::Downgraded>;
}

pub trait DowngradableDisplayArgs: DisplayArgs {
    type Downgraded: DisplayArgs<Locale = Self::Locale>;

    fn downgrade(this: Key<Self>) -> Key<Self::Downgraded>;
}

trait DynAnyIntoViewArgs<'a>: Send + Sync + 'a {
    type Locale: BaseLocale;

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn render(
        self: Box<Self>,
        locale: Self::Locale,
    ) -> core::pin::Pin<Box<dyn Future<Output = AnyView> + 'a>>;

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn render(self: Box<Self>, locale: Self::Locale) -> AnyView;

    fn clone(&self) -> Box<dyn DynAnyIntoViewArgs<'a, Locale = Self::Locale>>;
}

trait DynAnyDisplayArgs<'a>: Send + Sync + 'a {
    type Locale: BaseLocale;

    fn to_string(self: Box<Self>, locale: Self::Locale) -> String;

    fn clone(&self) -> Box<dyn DynAnyDisplayArgs<'a, Locale = Self::Locale>>;
}

#[derive(Clone, Copy)]
struct AnyArgsInner<A: Args> {
    id: A::Id,
    args: A,
}

impl<'a, A: IntoViewArgs + Send + Sync + 'a + Clone> DynAnyIntoViewArgs<'a> for AnyArgsInner<A> {
    type Locale = A::Locale;

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn render(
        self: Box<Self>,
        locale: Self::Locale,
    ) -> core::pin::Pin<Box<dyn Future<Output = AnyView> + 'a>> {
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

    fn clone(&self) -> Box<dyn DynAnyIntoViewArgs<'a, Locale = A::Locale>> {
        let this: Self = <AnyArgsInner<A> as Clone>::clone(self);
        Box::new(this)
    }
}

impl<'a, A: DisplayArgs + Clone + Send + Sync + 'a> DynAnyDisplayArgs<'a> for AnyArgsInner<A> {
    type Locale = A::Locale;

    fn to_string(self: Box<Self>, locale: Self::Locale) -> String {
        let AnyArgsInner { id, args } = *self;
        let as_display = A::to_display(args, id, locale);
        as_display.to_string()
    }

    fn clone(&self) -> Box<dyn DynAnyDisplayArgs<'a, Locale = A::Locale>> {
        let this: Self = <AnyArgsInner<A> as Clone>::clone(self);
        Box::new(this)
    }
}

impl<L: BaseLocale> Clone for AnyIntoViewArgs<'_, L> {
    fn clone(&self) -> Self {
        let args = DynAnyIntoViewArgs::clone(&*self.args);
        AnyIntoViewArgs { args }
    }
}

impl<L: BaseLocale> Clone for AnyDisplayArgs<'_, L> {
    fn clone(&self) -> Self {
        let args: Box<dyn DynAnyDisplayArgs<Locale = L>> = DynAnyDisplayArgs::clone(&*self.args);
        AnyDisplayArgs { args }
    }
}

impl<L: BaseLocale> Args for AnyIntoViewArgs<'_, L> {
    type Id = ();
    type Locale = L;
}

impl<L: BaseLocale> Args for AnyDisplayArgs<'_, L> {
    type Id = ();
    type Locale = L;
}

impl<L: BaseLocale> IntoViewArgs for AnyIntoViewArgs<'_, L> {
    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn render(self, _id: Self::Id, locale: Self::Locale) -> impl IntoViewFuture {
        DynAnyIntoViewArgs::render(self.args, locale)
    }

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn render(self, _id: Self::Id, locale: Self::Locale) -> impl IntoView {
        DynAnyIntoViewArgs::render(self.args, locale)
    }
}

impl<L: BaseLocale> DisplayArgs for AnyDisplayArgs<'_, L> {
    fn to_display(self, _id: Self::Id, locale: Self::Locale) -> impl Display {
        DynAnyDisplayArgs::to_string(self.args, locale)
    }
}

impl<L: BaseLocale> DowngradableArgs for AnyIntoViewArgs<'_, L> {
    type Downgraded = Self;
    fn downgrade(this: Key<Self>) -> Key<Self::Downgraded> {
        this
    }
}

impl<L: BaseLocale> DowngradableDisplayArgs for AnyDisplayArgs<'_, L> {
    type Downgraded = Self;
    fn downgrade(this: Key<Self>) -> Key<Self::Downgraded> {
        this
    }
}

impl<'a, L: BaseLocale> AnyIntoViewArgs<'a, L> {
    fn from_args<A>(args: A, id: A::Id) -> Self
    where
        A: IntoViewArgs<Locale = L> + Clone + Send + Sync + 'a,
    {
        let inner = AnyArgsInner { id, args };
        let boxed = Box::new(inner);
        AnyIntoViewArgs { args: boxed }
    }
}

impl<'a, L: BaseLocale> AnyDisplayArgs<'a, L> {
    fn from_args<A>(args: A, id: A::Id) -> Self
    where
        A: DisplayArgs<Locale = L> + Clone + Send + Sync + 'a,
    {
        let inner = AnyArgsInner { id, args };
        let boxed = Box::new(inner);
        AnyDisplayArgs { args: boxed }
    }
}

impl<B: ArgsBuilder> KeyBuilder<B> {
    #[doc(hidden)]
    pub fn build<F, A>(this: Self, f: F) -> Key<<B as IntoViewArgsMarker<A>>::Args>
    where
        F: FnOnce(B::Builder) -> A,
        B: IntoViewArgsMarker<A>,
    {
        let builded = f(B::new());
        let args = B::into_args(builded);
        Key { id: this.id, args }
    }

    #[doc(hidden)]
    pub fn build_display<F, A>(this: Self, f: F) -> Key<<B as DisplayArgsMarker<A>>::Args>
    where
        F: FnOnce(B::DisplayBuilder) -> A,
        B: DisplayArgsMarker<A> + DisplayArgsBuilder,
    {
        let builded = f(B::new_display());
        let args = B::into_args(builded);
        Key { id: this.id, args }
    }

    #[doc(hidden)]
    pub const fn const_build<F>(this: Self, _: &F) -> Key<<B as ConstArgsMarker>::Args>
    where
        F: FnOnce(B::ConstBuilder) -> B::Builded,
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
    #[doc(hidden)]
    pub fn render(this: Self, locale: A::Locale) -> impl IntoViewFuture
    where
        A: IntoViewArgs,
    {
        A::render(this.args, this.id, locale)
    }

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    #[doc(hidden)]
    pub fn render(this: Self, locale: A::Locale) -> impl IntoView
    where
        A: IntoViewArgs,
    {
        A::render(this.args, this.id, locale)
    }

    #[doc(hidden)]
    pub fn to_display(this: Self, locale: A::Locale) -> impl Display
    where
        A: DisplayArgs,
    {
        A::to_display(this.args, this.id, locale)
    }

    pub fn downgrade(self) -> Key<A::Downgraded>
    where
        A: DowngradableArgs,
    {
        A::downgrade(self)
    }

    pub fn downgrade_display(self) -> Key<A::Downgraded>
    where
        A: DowngradableDisplayArgs,
    {
        A::downgrade(self)
    }

    pub fn downgrade_any<'a>(self) -> Key<AnyIntoViewArgs<'a, A::Locale>>
    where
        A: IntoViewArgs + Clone + Send + Sync + 'a,
    {
        let any = AnyIntoViewArgs::from_args(self.args, self.id);
        Key { id: (), args: any }
    }

    pub fn downgrade_any_display<'a>(self) -> Key<AnyDisplayArgs<'a, A::Locale>>
    where
        A: DisplayArgs + Clone + Send + Sync + 'a,
    {
        let any = AnyDisplayArgs::from_args(self.args, self.id);
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
