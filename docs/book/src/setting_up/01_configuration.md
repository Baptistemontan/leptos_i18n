# Load The Translations

To load the translations we need codegen, and for that you can use the `leptos_i18n_build` package.

You use it with a `build.rs` file to generate the code to properly use you translations:

```rust,ignore
// build.rs

use leptos_i18n_build::{TranslationsInfos, Config};
use std::path::PathBuf;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
  println!("cargo::rerun-if-changed=build.rs");
  println!("cargo::rerun-if-changed=Cargo.toml");

  // where to generate the translations
  let i18n_mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");

  let cfg = Config::new("en")?.add_locale("fr")?; // "en" is the default locale, "fr" is another locale.

  let translations_infos = TranslationsInfos::parse(cfg)?;

  // emit the errors and warnings found during parsing
  translations_infos.emit_diagnostics();

  // emit "cargo::rerun-if-changed" for every translation file
  translations_infos.rerun_if_locales_changed();

  // codegen
  translations_infos.generate_i18n_module(i18n_mod_directory)?;

  Ok(())
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

# Configuration

This crate is basically entirely based around the code generated in a build script. We will cover it in a later chapter, but for now just know that it looks at your translation files and generates code for them.

To load those translations it first needs to know what to look for, so you need to declare what locales you are supporting and which one is the default.
To do that you use the `Config` builder:

To declare `en` and `fr` as locales, with `en` being the default you would write:

```rust
let cfg = Config::new("en")?.add_locale("fr")?;
```

There are more optional values you can supply:

- `add_namespace`: This is to split your translations into multiple files, we will cover it in a later chapter
- `locales_path`: This is to have a custom path to the directory containing the locales files, it defaults to `"./locales"`.
- `translations_uri`: Used in a CSR application with the `dynamic_load` feature, more information in a later chapter.
- `extend_locale`: "Allows you to describe the inheritance structure for locales, covered in a later chapter.
- `parse_options`: Parsing options, covered in the next segment

Once this configuration is done, you can start writing your translations.

## Parsing Options

`Config` can take some options as an argument, for now we use the default but you can import the `ParseOptions` struct to tell the parser what to expect and produce, here we change the file format to `yaml`:

```rust, ignore
use leptos_i18n_build::{FileFormat, ParseOptions, TranslationsInfos};
use std::path::PathBuf;
use std::error::Error;

fn main() -> Resul<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");

    let i18n_mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");

    let options = ParseOptions::default().file_format(FileFormat::Yaml);

    let cfg = Config::new("en")?.add_locale("fr")?.parse_options(options);

    let translations_infos = TranslationsInfos::parse(cfg)?;

    translations_infos.emit_diagnostics();

    translations_infos.rerun_if_locales_changed();

    translations_infos.generate_i18n_module(i18n_mod_directory)?;

    Ok(())
}
```

There are other options:

- `suppress_key_warnings`: remove warnings emitted by missing keys or surplus keys
- `interpolate_display`: generates extra code for each interpolation to allow rendering them as a string instead of a `View`
- `show_keys_only`: This feature makes every translation display only its corresponding key; this is useful for tracking untranslated strings in your application.

example:

```rust, ignore
let options = ParseOptions::default()
  .file_format(FileFormat::Json5)
  .suppress_key_warnings(true)
  .interpolate_display(true)
  .show_keys_only(true);
```

There is also a way to inject your own formatter, this needs its own chapter, which you can find in an appendix.

## Codegen Options

`TranslationsInfos::generate_i18n_module_with_options` can take a `CodegenOptions` argument that let you:

- Add some top level attributes for the generated module
- Customize the name of the generated file

example:

```rust, ignore
use leptos_i18n_build::CodegenOptions;

let attributes = "#![allow(missing_docs)]".parse()?;

let options = CodegenOptions::default()
  .top_level_attributes(Some(attributes))
  .module_file_name("i18n.rs"); // "mod.rs" by default

translations_infos.generate_i18n_module_with_options(options)?;
```
