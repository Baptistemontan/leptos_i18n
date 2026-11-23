# Custom File Format

The translations files can be written in JSON, JSON5, YAML or TOML (see `FileFormat`). If you want to use another format, you can give your own parser with `ParseOptions::with_custom_parser`.

## The `Parser` Trait

A parser implements the `Parser` trait, found in `leptos_i18n_build::options::parser`:

```rust,ignore
pub trait Parser: 'static {
    fn deserialize(&self, file: &str, path: Option<&Path>, seed: Seed) -> Result<Value, SerdeError>;

    fn file_extensions(&self) -> &'static [&'static str];
}
```

- `deserialize` gets the content of a translation file, and the path of that file if it was read from disk (`None` when the translations are given as strings, see `TranslationsInfos::parse_with_files_as_string`). It is mostly useful for error messages.
- `seed` is a `serde::de::DeserializeSeed`: give it a `serde` deserializer of your format, and it reads the translations from it, returning the `Value` to give back.
- `file_extensions` are the extensions of the files, tried in order when looking for `locales/{locale}.{ext}`.

Errors are returned as a `SerdeError`, use `SerdeError::custom` to build one from any error.

## Example

Here is a parser for [RON](https://github.com/ron-rs/ron), using the `ron` crate:

```rust,ignore
use leptos_i18n_build::options::parser::{Parser, Seed, SerdeError, Value};
use serde::de::DeserializeSeed;
use std::path::Path;

struct RonParser;

impl Parser for RonParser {
    fn deserialize(&self, file: &str, _path: Option<&Path>, seed: Seed) -> Result<Value, SerdeError> {
        let mut deserializer = ron::Deserializer::from_str(file).map_err(SerdeError::custom)?;
        seed.deserialize(&mut deserializer).map_err(SerdeError::custom)
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["ron"]
    }
}
```

And in the build script:

```rust,ignore
use leptos_i18n_build::options::ParseOptions;

let options = ParseOptions::default().with_custom_parser(RonParser);
let cfg = Config::new("en")?.add_locale("fr")?.parse_options(options);
```

The translations are then read from `locales/en.ron` and `locales/fr.ron`.

## Requirements

The translations are a map of keys to values, read through `serde`, so the format must be able to describe:

- a map with string keys at the root (the keys must be valid Rust identifiers, as with any other format),
- strings, numbers (signed, unsigned and floats), booleans, and null (for explicit defaults),
- nested maps for subkeys.

The values are read with `deserialize_any`, so the format must be self-describing, like the built-in ones.

`ParseOptions::with_custom_parser(parser)` is a shortcut for `ParseOptions::file_format(FileFormat::Custom(Arc::new(parser)))`.
