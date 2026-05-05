use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use super::Key;
use super::KeyId;
use super::comp_time::{ConstArgs, ConstArgsMarker};
use super::display::{DisplayArgsBuilder, DisplayArgsMarker};
use super::view::IntoViewArgsMarker;
use crate::locale_traits::BaseLocale;

pub struct KeyBuilder<B: ArgsBuilder> {
    id: B::Id,
    _marker: PhantomData<B>,
}

pub trait ArgsBuilder: Copy + 'static {
    type Id: KeyId;
    type Builder: 'static;
    type Locale: BaseLocale;

    fn new() -> Self::Builder;
}

pub trait DowngradableArgBuilder: ArgsBuilder {
    type Downgraded: ArgsBuilder<Locale = Self::Locale, Builder = Self::Builder>;
    const ID: <Self::Downgraded as ArgsBuilder>::Id;
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

impl<B: ArgsBuilder> Debug for KeyBuilder<B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KeyBuilder")
            .field("key", &self.id.key())
            .finish()
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
