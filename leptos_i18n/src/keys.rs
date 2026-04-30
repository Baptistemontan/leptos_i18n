use std::{
    fmt::{Debug, Display},
    hash::Hash,
    marker::PhantomData,
    sync::Arc,
};

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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayKey<A: DisplayArgs> {
    id: A::Id,
    locale: A::Locale,
    args: A,
    data: A::Data,
}

pub type AnyKey<L> = Key<AnyIntoViewArgs<L>>;

pub struct AnyIntoViewArgs<L: BaseLocale> {
    args: Box<dyn DynAnyIntoViewArgs<Locale = L>>,
}

pub struct AnyDisplayArgs<'a, L: BaseLocale, Data = DisplayData> {
    args: Arc<dyn DynAnyDisplayArgs<'a, Locale = L, Data = Data>>,
}

#[doc(hidden)]
#[cfg(not(feature = "dynamic_load"))]
pub type DisplayData = ();

#[doc(hidden)]
#[cfg(all(feature = "dynamic_load", feature = "ssr"))]
pub type DisplayData = &'static [&'static str];

#[doc(hidden)]
#[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
pub type DisplayData = &'static [Box<str>];

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

pub trait ConstArgs: Args + Copy + 'static {
    const THIS: Self;

    fn value(id: Self::Id, locale: Self::Locale) -> Literal;
}

#[doc(hidden)]
pub trait ConstArgsMarker: ArgsBuilder {
    type Builded;
    type Args: ConstArgs<Locale = Self::Locale, Id = Self::Id>;
}

#[doc(hidden)]
#[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
pub trait IntoViewFuture: Future<Output: IntoView + Clone + 'static> + 'static {}

#[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
impl<F> IntoViewFuture for F where F: Future<Output: IntoView + Clone + 'static> + 'static {}

pub trait Args: Sized {
    type Locale: BaseLocale;
    type Id: Send + Sync + Copy + Hash + Ord + 'static;
}

pub trait IntoViewArgs: Args {
    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn render(self, id: Self::Id, locale: Self::Locale) -> impl IntoViewFuture;

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn render(self, id: Self::Id, locale: Self::Locale) -> impl IntoView + Clone + 'static;
}

pub trait DisplayArgs: Args {
    type Data;

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn get_data(&self, id: Self::Id, locale: Self::Locale) -> Self::Data;

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn get_data(&self, id: Self::Id, locale: Self::Locale) -> impl Future<Output = Self::Data>;

    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
        id: Self::Id,
        locale: Self::Locale,
        data: &Self::Data,
    ) -> core::fmt::Result;
}

pub trait DowngradableArgs: IntoViewArgs {
    type Downgraded: IntoViewArgs<Locale = Self::Locale>;

    fn downgrade(this: Key<Self>) -> Key<Self::Downgraded>;
}

pub trait DowngradableDisplayArgs: DisplayArgs {
    type Downgraded: DisplayArgs<Locale = Self::Locale, Data = Self::Data>;

    fn downgrade(this: Key<Self>) -> Key<Self::Downgraded>;
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

trait DynAnyDisplayArgs<'a>: Send + Sync + 'a {
    type Locale: BaseLocale;
    type Data;

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn get_data(&self, locale: Self::Locale) -> Self::Data;

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn get_data<'b>(
        &'b self,
        locale: Self::Locale,
    ) -> core::pin::Pin<Box<dyn Future<Output = Self::Data> + 'b>>;

    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
        locale: Self::Locale,
        data: &Self::Data,
    ) -> core::fmt::Result;
}

#[derive(Clone, Copy)]
struct AnyArgsInner<A: Args, Data = ()> {
    id: A::Id,
    args: A,
    data_marker: PhantomData<Data>,
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
            data_marker: PhantomData,
        } = *self;
        let view = args.render(id, locale);
        ClonableAnyView(Box::new(view))
    }

    fn clone(&self) -> Box<dyn DynAnyIntoViewArgs<Locale = A::Locale>> {
        let this: Self = <AnyArgsInner<A> as Clone>::clone(self);
        Box::new(this)
    }
}

impl<'a, A: DisplayArgs + Clone + Send + Sync + 'a> DynAnyDisplayArgs<'a>
    for AnyArgsInner<A, A::Data>
where
    A: DisplayArgs + Clone + Send + Sync + 'a,
    A::Data: Send + Sync + 'a,
{
    type Locale = A::Locale;
    type Data = A::Data;

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn get_data(&self, locale: Self::Locale) -> Self::Data {
        A::get_data(&self.args, self.id, locale)
    }

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn get_data<'b>(
        &'b self,
        locale: Self::Locale,
    ) -> core::pin::Pin<Box<dyn Future<Output = Self::Data> + 'b>> {
        Box::pin(A::get_data(&self.args, self.id, locale))
    }

    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
        locale: Self::Locale,
        data: &Self::Data,
    ) -> core::fmt::Result {
        A::fmt(&self.args, formatter, self.id, locale, data)
    }
}

impl<L: BaseLocale> Clone for AnyIntoViewArgs<L> {
    fn clone(&self) -> Self {
        let args = DynAnyIntoViewArgs::clone(&*self.args);
        AnyIntoViewArgs { args }
    }
}

impl<L: BaseLocale, Data> Clone for AnyDisplayArgs<'_, L, Data> {
    fn clone(&self) -> Self {
        AnyDisplayArgs {
            args: self.args.clone(),
        }
    }
}

impl<L: BaseLocale> Args for AnyIntoViewArgs<L> {
    type Id = ();
    type Locale = L;
}

impl<L: BaseLocale, Data> Args for AnyDisplayArgs<'_, L, Data> {
    type Id = ();
    type Locale = L;
}

trait DynClonableAnyView: 'static + Send {
    fn as_any_view(&self) -> AnyView;

    fn clone(&self) -> Box<dyn DynClonableAnyView>;
}

struct ClonableAnyView(Box<dyn DynClonableAnyView>);

impl Clone for ClonableAnyView {
    fn clone(&self) -> Self {
        let inner = DynClonableAnyView::clone(&*self.0);
        Self(inner)
    }
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

impl<'a, L, Data> DisplayArgs for AnyDisplayArgs<'a, L, Data>
where
    L: BaseLocale,
    Data: 'a,
{
    type Data = Data;

    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    fn get_data(&self, _id: Self::Id, locale: Self::Locale) -> Self::Data {
        DynAnyDisplayArgs::get_data(&*self.args, locale)
    }

    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    fn get_data(&self, _id: Self::Id, locale: Self::Locale) -> impl Future<Output = Self::Data> {
        DynAnyDisplayArgs::get_data(&*self.args, locale)
    }

    fn fmt(
        &self,
        formatter: &mut core::fmt::Formatter<'_>,
        _id: Self::Id,
        locale: Self::Locale,
        data: &Self::Data,
    ) -> core::fmt::Result {
        DynAnyDisplayArgs::fmt(&*self.args, formatter, locale, data)
    }
}

impl<L: BaseLocale> DowngradableArgs for AnyIntoViewArgs<L> {
    type Downgraded = Self;
    fn downgrade(this: Key<Self>) -> Key<Self::Downgraded> {
        this
    }
}

impl<'a, L, Data> DowngradableDisplayArgs for AnyDisplayArgs<'a, L, Data>
where
    L: BaseLocale,
    Data: 'a,
{
    type Downgraded = Self;
    fn downgrade(this: Key<Self>) -> Key<Self::Downgraded> {
        this
    }
}

impl<L: BaseLocale> AnyIntoViewArgs<L> {
    fn from_args<A>(args: A, id: A::Id) -> Self
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

impl<'a, L, Data> AnyDisplayArgs<'a, L, Data>
where
    L: BaseLocale,
    Data: 'a + Send + Sync,
{
    fn from_args<A>(args: A, id: A::Id) -> Self
    where
        A: DisplayArgs<Locale = L, Data = Data> + Clone + Send + Sync + 'a,
    {
        let inner = AnyArgsInner {
            id,
            args,
            data_marker: PhantomData,
        };
        let boxed = Arc::new(inner);
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
        F: FnOnce(B::Builder) -> B::Builded,
        B: ConstArgsMarker,
    {
        Key {
            id: this.id,
            args: <B::Args as ConstArgs>::THIS,
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
    #[doc(hidden)]
    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    pub fn render(this: Self, locale: A::Locale) -> impl IntoViewFuture
    where
        A: IntoViewArgs,
    {
        A::render(this.args, this.id, locale)
    }

    #[doc(hidden)]
    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    pub fn render(this: Self, locale: A::Locale) -> impl IntoView + Clone + 'static
    where
        A: IntoViewArgs,
    {
        A::render(this.args, this.id, locale)
    }

    #[doc(hidden)]
    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    pub fn to_display(this: Self, locale: A::Locale) -> DisplayKey<A>
    where
        A: DisplayArgs,
    {
        let data = A::get_data(&this.args, this.id, locale);
        DisplayKey {
            locale,
            id: this.id,
            args: this.args,
            data,
        }
    }

    #[doc(hidden)]
    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    pub async fn to_display(this: Self, locale: A::Locale) -> DisplayKey<A>
    where
        A: DisplayArgs,
    {
        let data = A::get_data(&this.args, this.id, locale).await;
        DisplayKey {
            locale,
            id: this.id,
            args: this.args,
            data,
        }
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

    pub fn downgrade_any(self) -> Key<AnyIntoViewArgs<A::Locale>>
    where
        A: IntoViewArgs + Clone + Send + Sync + 'static,
    {
        let any = AnyIntoViewArgs::from_args(self.args, self.id);
        Key { id: (), args: any }
    }

    pub fn downgrade_any_display<'a>(self) -> Key<AnyDisplayArgs<'a, A::Locale, A::Data>>
    where
        A: DisplayArgs + Clone + Send + Sync + 'a,
        A::Data: Send + Sync + 'a,
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

impl<A: DisplayArgs> Debug for DisplayKey<A> {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<A: DisplayArgs> Display for DisplayKey<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        A::fmt(&self.args, f, self.id, self.locale, &self.data)
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum NoRecurse {}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Literal<M = &'static [Literal<NoRecurse>]>
where
    M: Copy + 'static,
{
    String(&'static str),
    Signed(i64),
    Unsigned(u64),
    Float(f64),
    Bool(bool),
    Multiple(M),
}

impl<M: Copy + 'static> Literal<M> {
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

    pub const fn multiple(self) -> Option<M> {
        if let Literal::Multiple(v) = self {
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
