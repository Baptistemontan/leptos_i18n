# The `i18n Ally` VS Code Extension

The [`i18n Ally`](https://marketplace.visualstudio.com/items?itemName=lokalise.i18n-ally) extension includes many features for managing, structuring, and automating translations, with the most notable being an overlay on translation keys in the code, displaying the corresponding translations.

This is very helpful, and this section is a guide for a minimal setup to make this extension work with `Leptos i18n`.

## Custom Framework Setup

For obvious reasons, this lib is not supported by `i18n Ally` (one day maybe?), but the developers of that extension have provided [a way](https://github.com/lokalise/i18n-ally/wiki/Custom-Framework) to make it work with custom frameworks.

You will need to first create a file in your `.vscode` folder named `i18n-ally-custom-framework.yml` and put this in it:

```yaml
languageIds:
  - rust

usageMatchRegex:
  - "[^\\w\\d]t!\\(\\s*[\\w.:]*,\\s*([\\w.]*)"
  - "[^\\w\\d]td!\\(\\s*[\\w.:]*,\\s*([\\w.]*)"
  - "[^\\w\\d]td_string!\\(\\s*[\\w.:]*,\\s*([\\w.]*)"
  - "[^\\w\\d]td_display!\\(\\s*[\\w.:]*,\\s*([\\w.]*)"

monopoly: true
```

`languageIds` is the language you are using in your project. I'm no expert, but this is probably for a VSC API to know what files to check.

`usageMatchRegex` is the regex to use to find the translations keys; the above regex patterns are, in order: for `t!`, `td!`, `td_string!` and `td_display!`. If you don't use all translations macros, you can remove/comment out the regex for that macro. These regexes may not be perfect, and I am no expert, so there may be better or faster alternatives. If you encounter a problem with them, feel free to open an issue or discussion on GitHub.

`monopoly` disables all supported frameworks; if you use any other frameworks supported by the extension in your project, set it to `false`.

## Common Settings

There are multiple settings for the extension that you can set in `.vscode/settings.json`; those are all optional. Here is a non-exhaustive list with their defaults in parentheses:

- `i18n-ally.keystyle` (auto): This option can be `flat` (`"a.b.c": "..."`) or `nested` (`"a": { "b": { "c": "..." } }`). This is irrelevant if you don’t use subkeys, but if you do, set it to `"nested"` as this is the style that this library supports.

- `i18n-ally.localesPaths`  (auto): This is the path to your locales; it can be a path or a list of paths. By default, it is set to `"locales"`, but if you use a custom locales path or a cargo workspace, you will have to supply the path here.

- `i18n-ally.namespace` (false): Set this to `true` if you use namespaces. If you use namespaces with `i18n Ally`, I have not figured out (maybe you will?) how to make the `namespace::key` syntax work for the macros, so just use `namespace.key`.

- `i18n-ally.sourceLanguage` (en): The primary language of the project; I suggest setting this to your default locale.

- `i18n-ally.displayLanguage` (en): The locale that the overlay uses.

You can find other settings that may interest you in the [official documentation](https://github.com/lokalise/i18n-ally/wiki/Configurations), with more information about the settings mentioned above, along with their default values.

## Other Features

This extension offers other interesting features. I suggest you take a look at their  [wiki](https://github.com/lokalise/i18n-ally/wiki) for more information.
