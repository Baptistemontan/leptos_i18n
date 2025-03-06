# Extanding a locale

The `inherits` config options under the `[package.metadata.leptos-i18n]` can allow you to describe inheritance hierarchy for your locales:

```toml
[package.metadata.leptos-i18n]
default = "en"
locales = ["en", "fr", "fr-CA"]
inherits = { fr-CA = "fr" }
```

This will default any missing keys in "fr-CA" to the value in "fr".

The "general" default for missing keys will still be the default locale, so if a key is missing in both "fr" and "fr-CA", the key will use the value in "en".

## Recursive inheritance

You can have recursive inheritances, this is allowed and works as expected:

```toml
[package.metadata.leptos-i18n]
default = "en"
locales = ["en", "fr", "fr-CA", "fr-FR"]
inherits = { fr-CA = "fr-FR", fr-FR = "fr" }
```

> note: cyclic inheritance is also valid but I don't see the use, it's only supported because if we didn't detected cycles we could have an endless loop when resolving what default to use, if a cycle is encountered on a missing key the default locale is used.

## Missing key warnings

if locale A extend locale B, missing key warnings will not be emitted for locale A.

Explicitly setting the inheritance to the default locale is also a way to suppress missing key warnings for a given locale:

```toml
[package.metadata.leptos-i18n]
default = "en"
locales = ["en", "fr", "it"]
inherits = { it = "en" }
```

While the above is technically already the default behavior, missing warnings will not be emitted for the "it" locale, but will be emitted for the "fr" locale.

## Extends the default locale

The default locale can not inherit.

This is not allowed and will error:

```toml
[package.metadata.leptos-i18n]
default = "en"
locales = ["en", "fr"]
inherits = { en = "fr" }
```
