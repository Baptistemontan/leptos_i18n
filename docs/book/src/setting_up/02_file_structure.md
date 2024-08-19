# File Structure

Now that you have configured your locales, you can start writing your translations. This chapter covers where to put your files. We will cover how to write them in another section.

By default you must put your files in the `./locales` directory, and each file must be `%{locale}.json`:

```bash
./locales
├── en.json
└── fr.json
```

## Custom Directory

You can change the path to the directory containing the files with the `locales-dir` field in the configuration, for example

```toml
[package.metadata.leptos-i18n]
default = "en"
locales = ["en", "fr"]
locales-dir = "./path/to/mylocales
```

will look for

```bash
./path
└── to
    └── mylocales
        ├── en.json
        └── fr.json
```

## Other Formats

JSON being the default, you can change that by first removing the defaults features, and enabling the feature for the format you need:

```toml
# Cargo.toml

[dependencies]
leptos_i18n = {
    default-features = false,
    features = ["yaml_files"] # other default features: ["cookie"]
}
```

| Format         | Feature      |
| -------------- | ------------ |
| JSON (default) | `json_files` |
| YAML           | `yaml_files` |

Other formats may be supported later.
