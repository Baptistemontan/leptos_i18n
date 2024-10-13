This document contain a cleaned up version of the expanded `load_locales!` macro.

_note_: this document is purely for curiosity, don't expect it to be kept in sync with every changes, it may be out of date or even never updated.
This macro is where most of the changes are made, either new features or improvement, keeping it in sync every time is time consumming, so for now I try to at least update it for new major releases.

`Cargo.toml`:

```toml
[package.metadata.leptos-i18n]
default = "en"
locales = ["en", "fr"]
```

`/locales/en.json`:

```json
{
  "hello_world": "Hello World!",
  "range": [
    "u32",
    ["Zero", 0],
    ["One", 1],
    ["2..=5", "2..=5"],
    ["{{ count }}"]
  ],
  "plural_one": "one item",
  "plural_other": "{{ count }} items",
  "some_subkeys": {
    "subkey_1": "This is subkey 1"
  },
  "key_present_only_in_default": "english default"
}
```

`/locales/fr.json`:

```json
{
  "hello_world": "Bonjour le monde!",
  "range": "<b>interpolate</b>",
  "plural_one": "un truc",
  "plural_other": "{{ count }} trucs",
  "some_subkeys": {
    "subkey_1": "Sous clé numéro 1"
  }
}
```

Expected expanded code of the `load_locales!` macro :

```rust
pub mod i18n {
    // the reason this exist is to easily swap the crate path,
    // there is an internal macro to declare translations without the need for configuration/translation files,
    // and if we want to use it inside the crate the path `leptos_i18n` is not present at root, so it must be swapped with `crate`.
    // Maybe one day I'll add a parameter to `load_locales` to give a custom path for the crate, or/and make the internal macro public.
    use leptos_i18n as l_i18n_crate;

    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    #[allow(non_camel_case_types)]
    pub enum Locale {
        en,
        fr,
    }

    impl Default for Locale {
        fn default() -> Self {
            Locale::en
        }
    }

    impl l_i18n_crate::reexports::serde::Serialize for Locale {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: l_i18n_crate::reexports::serde::Serializer,
        {
            l_i18n_crate::reexports::serde::Serialize::serialize(
                l_i18n_crate::Locale::as_str(*self),
                serializer,
            )
        }
    }

    impl<'de> l_i18n_crate::reexports::serde::Deserialize<'de> for Locale {
        fn deserialize<D>(deserializer: D) -> Result<Locale, D::Error>
        where
            D: l_i18n_crate::reexports::serde::de::Deserializer<'de>,
        {
            l_i18n_crate::reexports::serde::de::Deserializer::deserialize_str(
                deserializer,
                l_i18n_crate::__private::LocaleVisitor::<Locale>::new(),
            )
        }
    }

    impl l_i18n_crate::Locale for Locale {

        type Keys = I18nKeys;

        fn as_str(self) -> &'static str {
            let s = match self {
                Locale::en => "en",
                Locale::fr => "fr",
            };
            l_i18n_crate::__private::intern(s)
        }

        fn as_icu_locale(self) -> &'static l_i18n_crate::reexports::icu::locid::Locale {
            const EN_LANGID: &'static l_i18n_crate::reexports::icu::locid::Locale = &l_i18n_crate::reexports::icu::locid::locale!("en");
            const FR_LANGID: &'static l_i18n_crate::reexports::icu::locid::Locale = &l_i18n_crate::reexports::icu::locid::locale!("fr");
            match self {
                Locale::en => EN_LANGID,
                Locale::fr => FR_LANGID,
            }
        }

        fn get_all() -> &'static [Self] {
            &[Locale::en, Locale::fr]
        }

        fn to_base_locale(self) -> Self {
            self
        }

        fn from_base_locale(locale: Self) -> Self {
            locale
        }
    }

    impl core::str::FromStr for Locale {
        type Err = ();
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s.trim() {
                "en" => Ok(Locale::en),
                "fr" => Ok(Locale::fr),
                _ => Err(()),
            }
        }
    }

    impl core::convert::AsRef<l_i18n_crate::reexports::icu::locid::LanguageIdentifier> for Locale {
        fn as_ref(&self) -> &l_i18n_crate::reexports::icu::locid::LanguageIdentifier {
            l_i18n_crate::Locale::as_langid(*self)
        }
    }

    impl core::convert::AsRef<l_i18n_crate::reexports::icu::locid::Locale> for Locale {
        fn as_ref(&self) -> &l_i18n_crate::reexports::icu::locid::Locale {
            l_i18n_crate::Locale::as_icu_locale(*self)
        }
    }

    impl core::convert::AsRef<str> for Locale {
        fn as_ref(&self) -> &str {
            l_i18n_crate::Locale::as_str(*self)
        }
    }

    impl core::convert::AsRef<Self> for Locale {
        fn as_ref(&self) -> &Self {
            self
        }
    }

    impl core::fmt::Display for Locale {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            core::fmt::Display::fmt(l_i18n_crate::Locale::as_str(*self), f)
        }
    }

    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    #[allow(non_camel_case_types, non_snake_case)]
    pub struct I18nKeys {
        pub hello_world: &'static str,
        pub key_present_only_in_default: &'static str,
        pub range: builders::range_builder_dummy,
        pub plural: builders::plural_builder_dummy,
        pub some_subkeys: subkeys::sk_some_subkeys::some_subkeys_subkeys,
    }

    impl I18nKeys {
        #[allow(non_upper_case_globals)]
        pub const en: Self = Self::new(Locale::en);
        #[allow(non_upper_case_globals)]
        pub const fr: Self = Self::new(Locale::fr);

        pub const fn new(_locale: Locale) -> Self {
            match _locale {
                Locale::en => {
                    I18nKeys {
                        hello_world: "Hello World!",
                        key_present_only_in_default: "english default",
                        range: builders::range_builder_dummy::new(_locale),
                        plural: builders::plural_builder_dummy::new(_locale),
                        some_subkeys: subkeys::sk_some_subkeys::some_subkeys_subkeys::new(
                            _locale,
                        ),
                    }
                }
                Locale::fr => {
                    I18nKeys {
                        hello_world: "Bonjour le monde!",
                        key_present_only_in_default: "english default",
                        range: builders::range_builder_dummy::new(_locale),
                        plural: builders::plural_builder_dummy::new(_locale),
                        some_subkeys: subkeys::sk_some_subkeys::some_subkeys_subkeys::new(
                            _locale,
                        ),
                    }
                }
            }
        }
    }

    impl l_i18n_crate::LocaleKeys for I18nKeys {
        type Locale = Locale;
        fn from_locale(_locale: Locale) -> &'static Self {
            match _locale {
                Locale::en => &Self::en,
                Locale::fr => &Self::fr,
            }
        }
    }

    #[doc(hidden)]
    pub mod builders {
        use super::{Locale, l_i18n_crate};

        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        #[allow(non_camel_case_types, non_snake_case)]
        pub struct range_builder_dummy {
            _locale: Locale,
        }

        #[derive(l_i18n_crate::reexports::typed_builder::TypedBuilder)]
        #[builder(crate_module_path = l_i18n_crate::reexports::typed_builder)]
        #[allow(non_camel_case_types, non_snake_case)]
        pub struct range_builder<__comp_b__, __into_view_comp_b__, __var_count__> {
            _locale: Locale,
            _into_views_marker: core::marker::PhantomData<(__into_view_comp_b__,)>,
            comp_b: __comp_b__,
            var_count: __var_count__,
        }

        #[allow(non_camel_case_types)]
        impl<
            __comp_b__: l_i18n_crate::__private::InterpolateComp<__into_view_comp_b__>,
            __into_view_comp_b__: l_i18n_crate::reexports::leptos::IntoView,
            __var_count__: 'static + ::core::clone::Clone
                + l_i18n_crate::__private::InterpolateVar
                + l_i18n_crate::__private::InterpolateRangeCount<u32>,
        > l_i18n_crate::reexports::leptos::IntoView
        for range_builder<__comp_b__, __into_view_comp_b__, __var_count__> {
            fn into_view(self) -> l_i18n_crate::reexports::leptos::View {
                let Self { comp_b, var_count, _locale, .. } = self;
                match _locale {
                    Locale::fr => {
                        l_i18n_crate::reexports::leptos::IntoView::into_view(
                            core::clone::Clone::clone(
                                &comp_b,
                            )(
                                l_i18n_crate::reexports::leptos::ToChildren::to_children({
                                    move || Into::into(
                                        l_i18n_crate::reexports::leptos::IntoView::into_view(
                                            "interpolate",
                                        ),
                                    )
                                }),
                            ),
                        )
                    }
                    Locale::en => {
                        l_i18n_crate::reexports::leptos::IntoView::into_view({
                            let var_count = core::clone::Clone::clone(&var_count);
                            move || {
                                match var_count() {
                                    0u32 => {
                                        l_i18n_crate::reexports::leptos::IntoView::into_view("Zero")
                                    }
                                    1u32 => {
                                        l_i18n_crate::reexports::leptos::IntoView::into_view("One")
                                    }
                                    2u32..=5u32 => {
                                        l_i18n_crate::reexports::leptos::IntoView::into_view(
                                            "2..=5",
                                        )
                                    }
                                    _ => {
                                        let var_count = core::clone::Clone::clone(&var_count);
                                        l_i18n_crate::reexports::leptos::IntoView::into_view(
                                            var_count,
                                        )
                                    }
                                }
                            }
                        })
                    }
                }
            }
        }

        #[allow(non_camel_case_types)]
        impl<__comp_b__, __into_view_comp_b__, __var_count__> core::fmt::Debug
        for range_builder<__comp_b__, __into_view_comp_b__, __var_count__> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("range_builder").finish()
            }
        }

        impl range_builder_dummy {
            pub const fn new(_locale: Locale) -> Self {
                Self { _locale }
            }

            pub fn builder<
                __comp_b__: l_i18n_crate::__private::InterpolateComp<
                        __into_view_comp_b__,
                    >,
                __into_view_comp_b__: l_i18n_crate::reexports::leptos::IntoView,
                __var_count__: 'static + ::core::clone::Clone
                    + l_i18n_crate::__private::InterpolateVar
                    + l_i18n_crate::__private::InterpolateRangeCount<u32>,
            >(
                self,
            ) -> range_builderBuilder<
                __comp_b__,
                __into_view_comp_b__,
                __var_count__,
                (
                    (Locale,),
                    (core::marker::PhantomData<(__into_view_comp_b__,)>,),
                    (),
                    (),
                ),
            > {
                range_builder::builder()
                    ._locale(self._locale)
                    ._into_views_marker(core::marker::PhantomData)
            }
        }

        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        #[allow(non_camel_case_types, non_snake_case)]
        pub struct plural_builder_dummy {
            _locale: Locale,
        }

        #[derive(l_i18n_crate::reexports::typed_builder::TypedBuilder)]
        #[builder(crate_module_path = l_i18n_crate::reexports::typed_builder)]
        #[allow(non_camel_case_types, non_snake_case)]
        pub struct plural_builder<__var_count__> {
            _locale: Locale,
            _into_views_marker: core::marker::PhantomData<()>,
            var_count: __var_count__,
        }

        #[allow(non_camel_case_types)]
        impl<
            __var_count__: 'static + ::core::clone::Clone
                + l_i18n_crate::__private::InterpolateVar
                + l_i18n_crate::__private::InterpolatePluralCount,
        > l_i18n_crate::reexports::leptos::IntoView for plural_builder<__var_count__> {
            fn into_view(self) -> l_i18n_crate::reexports::leptos::View {
                let Self { var_count, _locale, .. } = self;
                match _locale {
                    Locale::fr => {
                        l_i18n_crate::reexports::leptos::IntoView::into_view({
                            let var_count = core::clone::Clone::clone(&var_count);
                            let _plural_rules = l_i18n_crate::__private::get_plural_rules(
                                _locale,
                                l_i18n_crate::reexports::icu::plurals::PluralRuleType::Cardinal,
                            );
                            move || {
                                match _plural_rules.category_for(var_count()) {
                                    l_i18n_crate::reexports::icu::plurals::PluralCategory::One => {
                                        l_i18n_crate::reexports::leptos::IntoView::into_view(
                                            "un truc",
                                        )
                                    }
                                    _ => {
                                        l_i18n_crate::reexports::leptos::CollectView::collect_view([
                                            {
                                                let var_count = core::clone::Clone::clone(&var_count);
                                                l_i18n_crate::reexports::leptos::IntoView::into_view(
                                                    var_count,
                                                )
                                            },
                                            l_i18n_crate::reexports::leptos::IntoView::into_view(
                                                " trucs",
                                            ),
                                        ])
                                    }
                                }
                            }
                        })
                    }
                    Locale::en => {
                        l_i18n_crate::reexports::leptos::IntoView::into_view({
                            let var_count = core::clone::Clone::clone(&var_count);
                            let _plural_rules = l_i18n_crate::__private::get_plural_rules(
                                _locale,
                                l_i18n_crate::reexports::icu::plurals::PluralRuleType::Cardinal,
                            );
                            move || {
                                match _plural_rules.category_for(var_count()) {
                                    l_i18n_crate::reexports::icu::plurals::PluralCategory::One => {
                                        l_i18n_crate::reexports::leptos::IntoView::into_view(
                                            "one item",
                                        )
                                    }
                                    _ => {
                                        l_i18n_crate::reexports::leptos::CollectView::collect_view([
                                            {
                                                let var_count = core::clone::Clone::clone(&var_count);
                                                l_i18n_crate::reexports::leptos::IntoView::into_view(
                                                    var_count,
                                                )
                                            },
                                            l_i18n_crate::reexports::leptos::IntoView::into_view(
                                                " items",
                                            ),
                                        ])
                                    }
                                }
                            }
                        })
                    }
                }
            }
        }

        #[allow(non_camel_case_types)]
        impl<__var_count__> core::fmt::Debug for plural_builder<__var_count__> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct("plural_builder").finish()
            }
        }

        impl plural_builder_dummy {
            pub const fn new(_locale: Locale) -> Self {
                Self { _locale }
            }
            pub fn builder<
                __var_count__: 'static + ::core::clone::Clone
                    + l_i18n_crate::__private::InterpolateVar
                    + l_i18n_crate::__private::InterpolatePluralCount,
            >(
                self,
            ) -> plural_builderBuilder<
                __var_count__,
                ((Locale,), (core::marker::PhantomData<()>,), ()),
            > {
                plural_builder::builder()
                    ._locale(self._locale)
                    ._into_views_marker(core::marker::PhantomData)
            }
        }
    }

    #[doc(hidden)]
    pub mod subkeys {
        use super::{Locale, l_i18n_crate};

        pub mod sk_some_subkeys {
            use super::{Locale, l_i18n_crate};

            #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
            #[allow(non_camel_case_types, non_snake_case)]
            pub struct some_subkeys_subkeys {
                pub subkey_1: &'static str,
            }

            impl some_subkeys_subkeys {
                pub const fn new(_locale: Locale) -> Self {
                    match _locale {
                        Locale::en => {
                            some_subkeys_subkeys {
                                subkey_1: "This is subkey 1",
                            }
                        }
                        Locale::fr => {
                            some_subkeys_subkeys {
                                subkey_1: "Sous clé numéro 1",
                            }
                        }
                    }
                }
            }

            impl l_i18n_crate::LocaleKeys for some_subkeys_subkeys {
                type Locale = Locale;
                fn from_locale(_locale: Locale) -> &'static Self {
                    &<super::super::I18nKeys as l_i18n_crate::LocaleKeys>::from_locale(
                            _locale,
                        )
                        .some_subkeys
                }
            }
        }
    }

    #[inline]
    pub fn use_i18n() -> l_i18n_crate::I18nContext<Locale> {
        l_i18n_crate::use_i18n_context()
    }

    #[deprecated(
        note = "It is now preferred to use the <I18nContextProvider> component"
    )]
    pub fn provide_i18n_context() -> l_i18n_crate::I18nContext<Locale> {
        l_i18n_crate::context::provide_i18n_context_with_options_inner(None, None, None)
    }

    mod providers {
        use super::{l_i18n_crate, Locale};
        use l_i18n_crate::reexports::leptos;
        use leptos::{IntoView, Children, Signal};
        use std::borrow::Cow;
        use l_i18n_crate::context::CookieOptions;

        quote! {
            #[l_i18n_crate::reexports::leptos::component]
            #[allow(non_snake_case)]
            pub fn I18nContextProvider(
                /// If the "lang" attribute should be set on the root `<html>` element. (default to true)
                #[prop(optional)]
                set_lang_attr_on_html: Option<bool>,
                /// Enable the use of a cookie to save the choosen locale (default to true).
                /// Does nothing without the "cookie" feature
                #[prop(optional)]
                enable_cookie: Option<bool>,
                /// Specify a name for the cookie, default to the library default.
                #[prop(optional, into)]
                cookie_name: Option<Cow<'static, str>>,
                /// Options for the cookie, see `leptos_use::UseCookieOptions`.
                #[prop(optional)]
                cookie_options: Option<CookieOptions<Locale>>,
                children: Children
            ) -> impl IntoView {
                l_i18n_crate::context::provide_i18n_context_component_inner::<Locale>(
                    set_lang_attr_on_html,
                    enable_cookie,
                    cookie_name,
                    cookie_options,
                    children
                )
            }

            #[l_i18n_crate::reexports::leptos::component]
            #[allow(non_snake_case)]
            pub fn I18nSubContextProvider(
                children: Children,
                /// The initial locale for this subcontext.
                /// Default to the locale set in the cookie if set and some,
                /// if not use the parent context locale.
                /// if no parent context, use the default locale.
                #[prop(optional, into)]
                initial_locale: Option<Signal<Locale>>,
                /// If set save the locale in a cookie of the given name (does nothing without the `cookie` feature).
                #[prop(optional, into)]
                cookie_name: Option<Cow<'static, str>>,
                /// Options for the cookie, see `leptos_use::UseCookieOptions`.
                #[prop(optional)]
                cookie_options: Option<CookieOptions<Locale>>,
            ) -> impl IntoView {
                l_i18n_crate::context::i18n_sub_context_provider_inner::<Locale>(
                    children,
                    initial_locale,
                    cookie_name,
                    cookie_options
                )
            }
        }
    }


    mod routing {
        use super::{l_i18n_crate, Locale};
        use l_i18n_crate::reexports::leptos_router;
        use l_i18n_crate::reexports::leptos;
        use leptos::{IntoView, Children};
        use leptos_router::{Loader, Method, TrailingSlash, SsrMode};
        #[l_i18n_crate::reexports::leptos::component(transparent)]
        #[allow(non_snake_case)]
        pub fn I18nRoute<E, F>(
            /// The base path of this application.
            /// If you setup your i18n route such that the path is `/foo/:locale/bar`,
            /// the expected base path is `/foo/`.
            /// Defaults to `"/"``.
            #[prop(default = "/")]
            base_path: &'static str,
            /// The view that should be shown when this route is matched. This can be any function
            /// that returns a type that implements [`IntoView`] (like `|| view! { <p>"Show this"</p> })`
            /// or `|| view! { <MyComponent/>` } or even, for a component with no props, `MyComponent`).
            /// If you use nested routes you can just set it to `view=Outlet`
            view: F,
            /// The mode that this route prefers during server-side rendering. Defaults to out-of-order streaming.
            #[prop(optional)]
            ssr: leptos_router::SsrMode,
            /// The HTTP methods that this route can handle (defaults to only `GET`).
            #[prop(default = &[Method::Get])]
            methods: &'static [Method],
            /// A data-loading function that will be called when the route is matched. Its results can be
            /// accessed with [`use_route_data`](crate::use_route_data).
            #[prop(optional, into)]
            data: Option<Loader>,
            /// How this route should handle trailing slashes in its path.
            /// Overrides any setting applied to [`crate::components::Router`].
            /// Serves as a default for any inner Routes.
            #[prop(optional)]
            trailing_slash: Option<TrailingSlash>,
            /// `children` may be empty or include nested routes.
            #[prop(optional)]
            children: Option<Children>,
        ) -> impl IntoView
            where E: IntoView,
            F: Fn() -> E + 'static
        {
            l_i18n_crate::__private::i18n_routing::<Locale, E, F>(base_path, children, ssr, methods, data, trailing_slash, view)
        }
    }

    pub use providers::{I18nContextProvider, I18nSubContextProvider};
    pub use routing::I18nRoute;
    pub use l_i18n_crate::Locale as I18nLocaleTrait;
    pub use leptos_i18n::{t, td, tu, use_i18n_scoped, scope_i18n, scope_locale};

    #[allow(unused)]
    fn warnings() {
        #[deprecated(
            note = "Missing key \"key_present_only_in_default\" in locale \"fr\""
        )]
        fn w0() {
            unimplemented!()
        }
        w0();
    }
}
```
