leptos_i18n::load_locales!();

pub fn i18n_context(cx: leptos::Scope) -> leptos_i18n::I18nContext<Locales> {
    leptos_i18n::get_context(cx)
}
