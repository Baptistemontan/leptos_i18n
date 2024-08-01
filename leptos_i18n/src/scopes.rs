use std::marker::PhantomData;

use crate::{I18nContext, Locale, LocaleKeys};

/// Represent a scope in a locale.
pub trait Scope<L: Locale> {
    /// The keys of the scopes
    type Keys: LocaleKeys<Locale = L>;
}

impl<K: LocaleKeys> Scope<K::Locale> for K {
    type Keys = K;
}

/// A struct that act as a marker for a scope, can be constructed as a constant and can be used to scope a context or a locale.
pub struct ConstScope<L: Locale, S: Scope<L> = <L as Locale>::Keys>(PhantomData<(L, S)>);

impl<L: Locale, S: Scope<L>> Default for ConstScope<L, S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<L: Locale, S: Scope<L>> Clone for ConstScope<L, S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<L: Locale, S: Scope<L>> Copy for ConstScope<L, S> {}

impl<L: Locale, S: Scope<L>> ConstScope<L, S> {
    /// Create a marker for a scope
    pub const fn new() -> Self {
        ConstScope(PhantomData)
    }

    /// This function is a helper for type resolution in macros.
    ///
    /// You can use it but it's meant to be used inside `use_i18n_scoped!` and `scope_i18n`.
    pub const fn new_from_ctx(_: I18nContext<L, S>) -> Self {
        Self::new()
    }

    #[doc(hidden)]
    pub const fn map<NS: Scope<L>>(self, _: fn(&S::Keys) -> &NS::Keys) -> ConstScope<L, NS> {
        ConstScope(PhantomData)
    }
}
