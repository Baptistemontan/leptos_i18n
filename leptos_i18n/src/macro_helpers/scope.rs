use crate::{I18nContext, Locale, Scope, scopes::ScopedLocale};

#[doc(hidden)]
pub const fn scope_ctx_util<L: Locale, OS: Scope<L>, NS: Scope<L>>(
    ctx: I18nContext<L, OS>,
    _: fn(OS) -> NS,
) -> I18nContext<L, NS> {
    ctx.scope()
}

#[doc(hidden)]
pub fn scope_locale_util<BL: Locale, L: Locale<BL>, NS: Scope<BL>>(
    locale: L,
    _: fn(<L as Locale<BL>>::Keys) -> NS,
) -> ScopedLocale<BL, NS> {
    locale.scope()
}
