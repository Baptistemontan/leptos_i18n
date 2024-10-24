# ICU4X Datagen

This library use ICU4X as a backend for formatters and plurals, and the default baked data provider can take quite a lot of space as it contains informations for _every possible locale_. So if you use only a few this is a complete waste.

## Disable compiled data

The first step to remove those excess informations is to disable the default data provider, it is activated by the `"icu_compiled_data"` feature that is enabled by default. So turn off default features or remove this feature.

## Custom provider

Great we lost a lot of size, but now instead of having too much informations we have 0 informations. You will now need to bring your own data provider. For that you will need multiple things.

## 1. Datagen

First generate the informations, you can use [`icu_datagen`](https://docs.rs/icu_datagen/latest/icu_datagen/) for that, either as a CLI of with a build.rs (we will come back to it later).

## 2. Load

Then you need to load those informations, this is as simple as

```rust
include!(concat!(env!("OUT_DIR"), "/baked_data/mod.rs"));

pub struct MyDataProvider;
impl_data_provider!(MyDataProvider);
```

This is explained in the `icu_datagen` doc

## 3. Supply to leptos_i18n the provider

You now just need to tell `leptos_i18n` what provider to use, for that you first need to impl `IcuDataProvider` for you provider, you can do it manually as it is straight forward, but the lib comes with a derive macro:

```rust
include!(concat!(env!("OUT_DIR"), "/baked_data/mod.rs"));

#[derive(leptos_i18n::custom_provider::IcuDataProvider)]
pub struct MyDataProvider;
impl_data_provider!(MyDataProvider);
```

And then pass it to the `set_icu_data_provider` function when the program start,
so for CSR apps in the main function:

```rust
fn main() {
    leptos_i18n::custom_provider::set_icu_data_provider(MyDataProvider);
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| leptos::view! { <App /> })
}
```

and for SSR apps in both on hydrate and on server startup:

```rust
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    leptos_i18n::custom_provider::set_icu_data_provider(MyDataProvider);
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
```

```rust
// example for actix
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    leptos_i18n::custom_provider::set_icu_data_provider(MyDataProvider);
    // ..
}
```

## Build.rs datagen

The doc for ICU4X datagen can be quite intimidating, but it is actually quite straight forward. Your build.rs can look like this:

```rust
use icu_datagen::baked_exporter::*;
use icu_datagen::prelude::*;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("baked_data");

    let exporter = BakedExporter::new(mod_directory, Default::default()).unwrap();

    DatagenDriver::new()
        // Keys needed for plurals
        .with_keys(icu_datagen::keys(&[
            "plurals/cardinal@1",
            "plurals/ordinal@1",
        ]))
        // Used locales, no fallback needed
        .with_locales_no_fallback([langid!("en"), langid!("fr")], Default::default())
        .export(&DatagenProvider::new_latest_tested(), exporter)
        .unwrap();
}
```

Here we are generating the informations for locales `"en"` and `"fr"`, with the data needed for plurals.

## Using `leptos_i18n_build` crate

You can use the `leptos_i18n_build` crate that contains utils for the datagen.
The problem with above `build.rs` is that it can go out of sync with your translations,
when all informations are already in the translations.

```toml
# Cargo.toml
[build-dependencies]
leptos_i18n_build = "0.5.0-gamma"
```

```rust
use leptos_i18n_build::TranslationsInfos;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=Cargo.toml");

    let mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("baked_data");

    let translations_infos = TranslationsInfos::parse().unwrap();

    translations_infos.rerun_if_locales_changed();

    translations_infos.generate_data(mod_directory).unwrap();
}
```

This will parse the config and the translations and generate the data for you using the informations gained when parsing the translations.
This will trigger a rerun if the config or translations changed and be kept in sync.
If your code use plurals, it will build with informations for plurals. If it use a formatter it will build with the informations for that formatter.

If you use more data someway, like for example using `t*_format!` with a formatter not used in the translations, there is functions to either supply additionnal options or keys:

```rust
use leptos_i18n_build::Options;

translations_infos.generate_data_with_options(mod_directory, [Options::FormatDateTime]).unwrap();
```

This will inject the ICU `DataKey`s needed for the `date`, `time` and `datetime` formatters.

```rust
use leptos_i18n_build::Options;

translations_infos.generate_data_with_data_keys(
    mod_directory,
    icu_datagen::keys(&["plurals/cardinal@1", "plurals/ordinal@1"])
).unwrap();
```

This will inject the keys for cardinal and ordinal plurals.

If you need both, `Options` can be turned into the need keys:

```rust
use leptos_i18n_build::Options;

let mut keys = icu_datagen::keys(&["plurals/cardinal@1", "plurals/ordinal@1"])
let keys.extend(Options::FormatDateTime.into_data_keys())

// keys now contains the `DataKey`s needed for plurals and for the `time`, `date` and `datetime` formatters.

translations_infos.generate_data_with_data_keys(mod_directory, keys).unwrap();
```

## Is it worth the trouble ?

YES. With `opt-level = "z"` and `lto = true`, the plurals example is at 394ko (at the time of writing), now by just providing a custom provider tailored to the used locales ("en" and "fr"), it shrinks down to 248ko! It almost cutted in half the binary size!
I highly suggest to take the time to implement this.

## Example

You can take a look at the `counter_icu_datagen` example, this is a copy of the `counter_plurals` example but with a custom provider.
