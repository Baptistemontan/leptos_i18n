# Namespaces

Translations files can grow quite rapidly and become very big, and avoiding key collisions can be hard without avoiding long names.
To avoid this situation, you can declare namespaces in the configuration:

```toml
[package.metadata.leptos-i18n]
default = "en"
locales = ["en", "fr"]
namespaces = ["common", "home"]
```

Then your file structures must look like this in the `/locales` directory:

```bash
./locales
├── en
│   ├── common.json
│   └── home.json
└── fr
    ├── common.json
    └── home.json
```

You can now make smaller files, with one for each section of the website, for example.
This also allows the `common` namespace to use keys that the `home` namespace also uses, without colliding.
