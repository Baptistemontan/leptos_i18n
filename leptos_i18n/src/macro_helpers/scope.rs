use crate::{I18nContext, Locale, Scope, scopes::ScopedLocale};

#[doc(hidden)]
pub const fn scope_ctx_util<OS: Scope, NS: Scope<BaseLocale = OS::BaseLocale>>(
    ctx: I18nContext<OS>,
    _: fn(OS::Keys) -> NS,
) -> I18nContext<NS> {
    ctx.scope()
}

#[doc(hidden)]
pub fn scope_locale_util<L: Locale, S: Scope<BaseLocale = L::BaseLocale>>(
    locale: L,
    _: fn(<L as Scope>::Keys) -> S,
) -> ScopedLocale<S> {
    locale.scope()
}

#[doc(hidden)]
pub const fn get_keys_const<S: Scope>() -> S::Keys {
    <S::Keys as crate::scopes::Keys>::THIS
}

#[doc(hidden)]
pub const fn check_is_key<B: crate::keys::ArgsBuilder>(
    key: crate::keys::KeyBuilder<B>,
) -> crate::keys::KeyBuilder<B> {
    key
}
