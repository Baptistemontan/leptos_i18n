# Locale Inheritance

Locale inheritance allows you to create specialized locales that build upon more general ones, reducing duplication and making maintenance easier. Instead of defining every key for each locale, you can override only the keys that differ while inheriting the rest from a parent locale.

## What is Locale Inheritance?

Imagine you have an English locale (`en`) with all your application's text. Now you want to create an American English locale (`en-US`) that uses most of the same text but changes a few specific terms (like "colour" to "color").

With inheritance, the `en-US` locale will:

- Use its own keys when they exist
- Fall back to the `en` locale's keys when they don't exist

This means you only need to define the differences in `en-US`, not duplicate everything.

## How Inheritance Works

There are two types of inheritance in this crate:

### 1. Implicit Inheritance (Automatic)

The crate automatically creates inheritance relationships based on locale structure. It follows a simple rule: more specific locales inherit from more general ones.

**Matching Pattern**: `language[-region][-anything-else]` inherits from `language[-region]`

#### Example Inheritance Tree

Given these locales:

- `en` (default)
- `en-US`
- `en-Latn`
- `en-Valencia`
- `en-Latn-US-Valencia`
- `en-Latn-US-Valencia-u-ca-buddhist`
- `en-Latn-US-u-ca-buddhist`
- `fr`
- `fr-FR`
- `fr-FR-u-ca-buddhist`
- `fr-u-ca-buddhist`

The automatic inheritance tree becomes:

```
en (default)
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
```

**Important Note**: Scripts, variants, and extensions are ignored in automatic matching. For example, `en-Latn-US-Valencia-u-ca-buddhist` inherits from `en-US` (not `en-Latn-US-Valencia`) because the system only considers the language (`en`) and region (`US`) parts.

### 2. Explicit Inheritance (Manual)

When automatic inheritance is not sufficient, you can manually specify inheritance relationships using the `inherits` configuration.

#### Configuration

Add inheritance rules with the config builder:

```rust
let cfg = cfg.extend_locale("child-locale", "parent-locale")?;
```

#### Example

To make `en-Latn-US-Valencia-u-ca-buddhist` inherit from `en-Latn-US-Valencia` instead of `en-US`:

```rust
let cfg = cfg.extend_locale("en-Latn-US-Valencia-u-ca-buddhist", "en-Latn-US-Valencia")?;
```

This changes the inheritance tree to:

```
en (default)
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
```

## Missing Key Warnings

The inheritance system affects how missing key warnings are handled.

### When Warnings Are Suppressed

- **Child locales**: If locale A inherits from locale B, no missing key warnings are emitted for locale A
- **Reason**: Missing keys are expected to be provided by the parent locale

### When Warnings Are Emitted

- **Root locales**: Locales that don't inherit from others (except the default) will show warnings for missing keys
- **Example**: In the tree above, `fr` will show warnings for keys present in `en` but missing in `fr`

### Suppressing Warnings

You can suppress missing key warnings for a locale by explicitly setting it to inherit from the default locale:

```rust
let cfg = Config::new("en")?
    .add_locale("fr")?
    .add_locale("it")?
    .extend_locale("it", "en")?;
```

**Result**:

- `it` locale: No missing key warnings (explicitly inherits from `en`)
- `fr` locale: Will show missing key warnings (doesn't explicitly inherit)

## Important Rules and Limitations

### Default Locale Cannot Inherit

The default locale is the root of the inheritance tree and cannot inherit from other locales.

**This will cause an error**:

```rust
let cfg = Config::new("en")?
    .add_locale("fr")?
    .extend_locale("en", "fr")?; // ❌ Error: default locale cannot inherit
```

### Inheritance vs. Defaulting

There's a distinction between:

- **Inheritance**: Explicit parent-child relationships between related locales
- **Defaulting**: Falling back to the default locale when no other option exists

For example, while `fr` technically falls back to `en` (the default), this is considered defaulting, not inheritance. Therefore, `fr` can still generate missing key warnings.

This inheritance system provides flexibility while maintaining simplicity for common use cases.
