use crate::{scopes::ScopedLocale, ConstScope, I18nContext, Locale, Scope};

#[doc(hidden)]
pub const fn scope_ctx_util<L: Locale, OS: Scope<L>, NS: Scope<L>>(
    ctx: I18nContext<L, OS>,
    map_fn: fn(OS) -> NS,
) -> I18nContext<L, NS> {
    let old_scope = ConstScope::<L, OS>::new();
    let new_scope = old_scope.map(map_fn);
    ctx.scope(new_scope)
}

#[doc(hidden)]
pub fn scope_locale_util<BL: Locale, L: Locale<BL>, NS: Scope<BL>>(
    locale: L,
    map_fn: fn(<L as Locale<BL>>::Keys) -> NS,
) -> ScopedLocale<BL, NS> {
    let _ = map_fn;
    ScopedLocale::new(locale.to_base_locale())
}
