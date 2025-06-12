# Inheritance

This crate support the extension of locales, which allow to override keys of a locale. 
For example you have setup the "en" locale, and now you want to setup the "en-US" locale, but only a few keys need to be overriden,
so what you would want is that locale "en-US" use its keys when present, but "en" keys when absent.
This is what inheritance allow.

# Implicit Inheritance

The default behavior is that a locale will automically extend a more "general" locale if it exists, 
for example with the following locales:

 - fr-FR-u-ca-buddhist
 - fr-u-ca-buddhist
 - fr-FR
 - fr
 - en-Latn-US-Valencia
 - en-Latn-US-Valencia-u-ca-buddhist
 - en-Latn-US-u-ca-buddhist
 - en-Valencia
 - en-Latn
 - en-US
 - en (default)

the resulting default inheritance tree will be:

en
├── en-US
│   ├── en-Latn-US-Valencia
│   ├── en-Latn-US-Valencia-u-ca-buddhist
│   └── en-Latn-US-u-ca-buddhist
├── en-Valencia
├── en-Latn
└── fr
    ├── fr-FR
    │   └── fr-FR-u-ca-buddhist 
    └── fr-u-ca-buddhist

If you look closely, you would think that "en-Latn-US-Valencia-u-ca-buddhist" should extend "en-Latn-US-Valencia",
while this make sense, we decided to keep the default behavior simple and do a match only on the language and region,
making `lang[-region][-*]` only match with `lang[-region]`, scripts/variants/extensions are ignored.
If you need more control, you can manually specify the inheritance of some locales:

# Explicit Inheritance

The `inherits` config options under the `[package.metadata.leptos-i18n]` can allow you to manually override inheritance hierarchy for your locales:

```toml
[package.metadata.leptos-i18n]
inherits = { 
    en-Latn-US-Valencia-u-ca-buddhist = "en-Latn-US-Valencia" 
}
```

This will make "en-Latn-US-Valencia-u-ca-buddhist" inherit the values of "en-Latn-US-Valencia",
Updating the inheritance tree to look like:

en
├── en-US
│   ├── en-Latn-US-Valencia
│   │   └── en-Latn-US-Valencia-u-ca-buddhist
│   └── en-Latn-US-u-ca-buddhist
├── en-Valencia
├── en-Latn
└── fr
    ├── fr-FR
    │   └── fr-FR-u-ca-buddhist 
    └── fr-u-ca-buddhist

## Missing key warnings

if locale A extend locale B, missing key warnings will not be emitted for locale A.

While technically "fr" extends "en", it considered a defaulting rather than an extension,
So if some keys are present in "en" but not in "fr" a warning will be emitted.
In this case, "fr" is the only locale that can emit warning.

Explicitly setting the inheritance to the default locale is also a way to suppress missing key warnings for a given locale:

```toml
[package.metadata.leptos-i18n]
default = "en"
locales = ["en", "fr", "it"]
inherits = { it = "en" }
```

While the above is technically already the default inheritance behavior, warnings will not be emitted for the "it" locale, but will be emitted for the "fr" locale.

## Extends the default locale

The default locale can not inherit.

This is not allowed and will error:

```toml
[package.metadata.leptos-i18n]
default = "en"
locales = ["en", "fr"]
inherits = { en = "fr" }
```
