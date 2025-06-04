# Load The Translations

To load the translations we need codegen, and for that you can use the `leptos_i18n_build` package, the one we referenced from the beginning of this book but never talked about.

You use it with a `build.rs` file to generate the code to properly use you translations:

```rust,ignore
// build.rs

use leptos_i18n_build::TranslationsInfos;
use std::path::PathBuf;

fn main() {
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=Cargo.toml");

  // where to generate the translations
  let i18n_mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");

  let translations_infos = TranslationsInfos::parse(Default::default()).unwrap();

  // emit "cargo:rerun-if-changed" for every translations files
  translations_infos.rerun_if_locales_changed();

  // codegen
  translations_infos
    .generate_i18n_module(i18n_mod_directory)
    .unwrap();
}
```

## The `i18n` module

You can then import the generated code with:

```rust, ignore
include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));
```

This will include a module called `i18n`. This module contains everything you need to use your translations.

```rust, ignore
include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));
use i18n::*;
``` 

## Options

`TranslationsInfos::parse` take some options as an argument, for now we use the default but you can import the `Options` struct to tell th codegen and parser what to expect and produce, here we change the file format to `yaml`:

```rust, ignore
use leptos_i18n_build::{FileFormat, Options, TranslationsInfos};
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    let i18n_mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");

    let options = Options::default().file_format(FileFormat::Yaml);

    let translations_infos = TranslationsInfos::parse(options).unwrap();

    translations_infos.rerun_if_locales_changed();

    translations_infos
        .generate_i18n_module(i18n_mod_directory)
        .unwrap();
}
```

There are other options:
- `suppress_key_warnings`: remove warnings emitted by missing keys or surplus keys
- `interpolate_display`: generates extra code for each interpolation to allow rendering them as a string instead of a `View`
- `show_keys_only`: This feature makes every translation to only display its corresponding key, this is useful to track untranslated strings in your application.

example:

```rust, ignore
let options = Options::default()
  .file_format(FileFormat::Json5)
  .suppress_key_warnings(true)
  .interpolate_display(true)
  .show_keys_only(true);
``` 

  








