This document contain what the `load_locales!` macro should expand to based on the given locales files, the only relevant feature flag enabled is `serde`, the relevant _not_ enabled feature flags are `debug_interpolations`, `suppress_key_warnings` and `nightly`.

None of the comments are part of the outputed code, they are here to explain the choices made that lead to this code.

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
  "plural": [
    "u32",
    ["Zero", 0],
    ["One", 1],
    ["2..=5", "2..=5"],
    ["{{ count }}"]
  ],
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
  "plural": "<b>interpolate</b>",
  "some_subkeys": {
    "subkey_1": "Sous clé numéro 1"
  }
}
```

Expected expanded code of the `load_locales!` macro :

```rust
// originally directly outputed the code, now output all code in it's own module. Changed that in `v0.2` beta
pub mod i18n {
    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    #[allow(non_camel_case_types)]
    pub enum Locale {
        en,
        fr
    }

    impl Default for Locale {
        fn default() -> Self {
            Locale::en
        }
    }

    impl leptos_i18n::Locale for Locale {
        type Keys = I18nKeys;

        fn as_str(self) -> &'static str {
            match self {
                Locale::en => "en",
                Locale::fr => "fr",
            }
        }
        fn from_str(s: &str) -> Option<Self> {
            match s {
                "en" => Some(Locale::en),
                "fr" => Some(Locale::fr),
                _ => None
            }
        }
    }

    #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
    #[allow(non_camel_case_types, non_snake_case)]
    pub struct I18nKeys {
        pub hello_world: &'static str,
        pub key_present_only_in_default: &'static str,
        pub plural: builders::plural_builder<builders::EmptyInterpolateValue, builders::EmptyInterpolateValue>,
        pub some_subkeys: subkeys::sk_some_subkeys::some_subkeys_subkeys,
    }

    impl I18nKeys {
        // Cool thing about making it all compile time is that you can even make a constante value at compile time,
        // one of the problem is that with all the pointers for each translations the type can grow quite big,
        // so instead of re-creating the type eache time you want to access the values you return a static ref to a value
        // created at compile time.
        #[allow(non_upper_case_globals)]
        pub const en: Self = Self::new(Locale::en);
        #[allow(non_upper_case_globals)]
        pub const fr: Self = Self::new(Locale::fr);

        pub const fn new(_locale: Locale) -> Self {
            match _locale {
                Locale::en => I18nKeys {
                    hello_world: "Hello World!",
                    key_present_only_in_default: "english default",
                    plural: builders::plural_builder::new(_locale),
                    some_subkeys: subkeys::sk_some_subkeys::some_subkeys_subkeys::new(_locale),
                },
                Locale::fr => I18nKeys {
                    hello_world: "Bonjour le monde!",
                    // keys present in default but not in another locale is defaulted to the default locale value
                    key_present_only_in_default: "english default",
                    plural: builders::plural_builder::new(_locale),
                    some_subkeys: subkeys::sk_some_subkeys::some_subkeys_subkeys::new(_locale),
                }
            }
        }
    }

    impl leptos_i18n::LocaleKeys for I18nKeys {
        type Locale = Locale;
        fn from_locale(_locale: Locale) -> &'static Self {
            match _locale {
                Locale::en => &Self::en,
                Locale::fr => &Self::fr,
            }
        }
    }

    // Builders type have there own module
    #[doc(hidden)]
    pub mod builders {
        use super::Locale;

        // this type is a marker for an empty field
        // as a ZST this makes the empty builder the same size as Locale
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub struct EmptyInterpolateValue;

        #[allow(non_camel_case_types, non_snake_case)]
        #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub struct plural_builder<__var_count, __comp_b> {
            _locale: Locale,
            var_count: __var_count,
            comp_b: __comp_b
        }

        impl plural_builder<EmptyInterpolateValue, EmptyInterpolateValue> {
            pub const fn new(_locale: Locale) -> Self {
                Self {
                    _locale,
                    var_count: EmptyInterpolateValue,
                    comp_b: EmptyInterpolateValue
                }
            }
        }

        #[allow(non_camel_case_types)]
        impl<
            __var_count: Fn() -> u32 + core::clone::Clone + 'static,
            __comp_b: Fn(leptos::ChildrenFn) -> leptos::View + core::clone::Clone + 'static
        > leptos::IntoView for plural_builder<__var_count, __comp_b> {
            fn into_view(self) -> leptos::View {
                let Self { _locale, var_count, comp_b } = self;
                match _locale {
                    Locale::en => {
                        leptos::IntoView::into_view(
                            {
                                let var_count = core::clone::Clone::clone(&var_count);
                                move || match var_count() {
                                    0u32 => leptos::IntoView::into_view("Zero"),
                                    1u32 => leptos::IntoView::into_view("One"),
                                    2u32..=5u32 => leptos::IntoView::into_view("2..=5"),
                                    _ => leptos::IntoView::into_view(core::clone::Clone::clone(&var_count))
                                }
                            },
                        )
                        // one thing I want to revisit is the amount of clones,
                        // it's not obvious here but every variable/components/ect could be used multiple times
                        // and without the clones the function would be `FnOnce`, which can't be turned into a `View`
                        // The block return a function because `var_count` could be a wrapper for a signal, needing reactivity.
                    },
                    Locale::fr => {
                        leptos::IntoView::into_view(core::clone::Clone::clone(&comp_b)(
                            leptos::ToChildren::to_children({
                                move || Into::into(leptos::IntoView::into_view("interpolate"))
                            })
                        ))
                    }
                }
            }
        }

        #[allow(non_camel_case_types)]
        impl<__var_count, __comp_b> plural_builder<__var_count, __comp_b> {
            #[inline]
            pub fn var_count<__T>(self, var_count: __T) -> plural_builder<impl Fn() -> u32 + core::clone::Clone + 'static, __comp_b>
                where __T: Fn() -> u32 + core::clone::Clone + 'static
            {
                let Self { _locale, comp_b, .. } = self;
                plural_builder { _locale, var_count, comp_b }
            }
        }

        #[allow(non_camel_case_types)]
        impl<__var_count, __comp_b> plural_builder<__var_count, __comp_b> {
            #[inline]
            pub fn comp_b<__O, __T>(self, comp_b: __T) -> plural_builder<__var_count, impl Fn(leptos::ChildrenFn) ->  leptos::View + core::clone::Clone + 'static>
            where
                __O: leptos::IntoView,
                __T: Fn(leptos::ChildrenFn) -> __O + core::clone::Clone + 'static
            {
                let Self { _locale, var_count, .. } = self;
                let comp_b = move |children| leptos::IntoView::into_view(comp_b(children));
                plural_builder { _locale, var_count, comp_b }
            }
        }

        // The build function is pointless work wise, as it just return itself
        // This code is to gate uncomplete builders,
        // if a key is missing you'll get a `builder function does not exist ...` type of error, instead of the obscure `IntoView is not implemented on super_weird_generics_whatever`. Not a lot better in itself, but from what I've seen the `IntoView` error span the whole `view!` macro, but the build function error span only the `t!` macro, which is a lot more helpfull.
        // This also allow to generate variants of this function that can serves as better error feedback with the `debug_interpolations` feature.
        #[allow(non_camel_case_types)]
        impl<
            __var_count: Fn() -> u32 + core::clone::Clone + 'static,
            __comp_b: Fn(leptos::ChildrenFn) -> leptos::View + core::clone::Clone + 'static
        > plural_builder<__var_count, __comp_b> {
            #[inline]
            pub fn build(self) -> Self {
                self
            }
        }
    }

    // Subkeys have there own modules
    #[doc(hidden)]
    pub mod subkeys {
        use super::Locale;

        // and each subkeys have the own modules
        // this is because it's the same function that is called to make the subkeys type that the one that make the `I18nKeys` type,
        // so if this has some builders, or some subkeys, it will create a `builders`/`subkeys` module.
        pub mod sk_some_subkeys {
            use super::Locale;

            #[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
            #[allow(non_camel_case_types, non_snake_case)]
            pub struct some_subkeys_subkeys {
                pub subkey_1: &'static str,
            }

            impl some_subkeys_subkeys {
                pub const fn new(_locale: Locale) -> Self {
                    match _locale {
                        Locale::en => Self {
                            subkey_1: "This is subkey 1",
                        },
                        Locale::fr => Self {
                            subkey_1: "Sous clé numéro 1",
                        }
                    }
                }
            }
        }
    }

    // create wrapper function to avoid needing to type the `Locale` type every time.
    // also shorten the function name.
    #[inline]
    pub fn use_i18n() -> leptos_i18n::I18nContext<Locale> {
        leptos_i18n::use_i18n_context()
    }

    // same here
    #[inline]
    pub fn provide_i18n_context() -> leptos_i18n::I18nContext<Locale> {
        leptos_i18n::provide_i18n_context()
    }

    mod provider {
        // #[leptos::island] if the `experimental-islands` feature is enabled
        #[leptos::component]
        pub fn I18nContextProvider(children: leptos::Children) -> impl leptos::IntoView {
            super::provide_i18n_context();
            children()
        }
    }
    pub use provider::I18nContextProvider; // this is to avoid bloat with the generated struct of the component

    // re-export `t!` and `td!` to just need to do `use i18n::*` and basically import everything you need.
    pub use leptos_i18n::{t, td};

    // `Diagnostic` is a nightly feature, so this is a trick to output custom warning messages:
    // calling depreacted functions with a custom note.
    #[allow(unused)]
    fn warnings() {
        #[deprecated(note = "Missing key \"key_present_only_in_default\" in locale \"fr\"")]
        fn w0() {
            unimplemented!()
        }
        w0();
    }
}
```
