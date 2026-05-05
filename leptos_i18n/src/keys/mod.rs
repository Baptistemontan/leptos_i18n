use std::fmt::Debug;
use std::hash::Hash;

pub mod args;
pub mod builder;
pub mod comp_time;
pub mod display;
pub mod view;

use args::{AnyArgsId, Args};
use display::{AnyDisplayArgs, DisplayArgs, DisplayKey, DowngradableDisplayArgs};
use view::{AnyIntoViewArgs, DowngradableArgs, IntoViewArgs};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key<A: Args> {
    id: A::Id,
    args: A,
}

pub type AnyKey<L> = Key<AnyIntoViewArgs<L>>;

pub trait KeyId: Send + Sync + Copy + Hash + Ord + 'static {
    fn key(self) -> &'static str;
}

impl<A: Args> Key<A> {
    #[doc(hidden)]
    #[cfg(all(feature = "dynamic_load", not(feature = "ssr")))]
    pub fn render(this: Self, locale: A::Locale) -> impl view::IntoViewFuture
    where
        A: IntoViewArgs,
    {
        A::render(this.args, this.id, locale)
    }

    #[doc(hidden)]
    #[cfg(not(all(feature = "dynamic_load", not(feature = "ssr"))))]
    pub fn render(this: Self, locale: A::Locale) -> impl leptos::IntoView + Clone + 'static
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
        let key_id = self.id.key();
        let any = AnyIntoViewArgs::from_args(self.args, self.id);
        Key {
            id: AnyArgsId(key_id),
            args: any,
        }
    }

    pub fn downgrade_any_display<'a>(self) -> Key<AnyDisplayArgs<'a, A::Locale, A::Data>>
    where
        A: DisplayArgs + Clone + Send + Sync + 'a,
        A::Data: Send + Sync + 'a,
    {
        let key_id = self.id.key();
        let any = AnyDisplayArgs::from_args(self.args, self.id);
        Key {
            id: AnyArgsId(key_id),
            args: any,
        }
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

impl<A: Args + Debug> Debug for Key<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Key")
            .field("key", &self.id.key())
            .field("args", &self.args)
            .finish()
    }
}
