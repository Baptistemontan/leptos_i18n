# Namespaces

Translation files can grow quite rapidly and become very large. Avoiding key collisions can be difficult without the use of long names.
To avoid this situation, you can declare namespaces in the configuration:

```rust
let cfg = Config::new("en")?.add_locale("fr")?.add_namespaces(["common", "home"]);
```

Then your file structure must look like this in the `/locales` directory:

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
