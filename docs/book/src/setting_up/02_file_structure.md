# File Structure

Now that you have configured your locales, you can start writing your translations. This chapter covers where to put your files. We will cover how to write them in another section.

By default, you must put your files in the `./locales` directory, and each file must be `%{locale}.json`:

```bash
./locales
├── en.json
└── fr.json
```

## Custom Directory

You can change the path to the directory containing the files with the `locales-dir` field in the configuration, for example:

```toml
[package.metadata.leptos-i18n]
default = "en"
locales = ["en", "fr"]
locales-dir = "./path/to/mylocales”
```

will look for:

```bash
./path
└── to
    └── mylocales
        ├── en.json
        └── fr.json
```

## Other Formats

JSON is the default format, but other format are supported, we will see how to change that later, here is a list of supported formats:

| Format         |
| -------------- |
| JSON (default) |
| JSON5          |
| YAML           |

Other formats may be supported later.
