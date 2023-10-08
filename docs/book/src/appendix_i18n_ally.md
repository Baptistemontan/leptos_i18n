# The `i18n Ally` VS Code extension

The [`i18n Ally`](https://marketplace.visualstudio.com/items?itemName=lokalise.i18n-ally) extension is an extension
that have a bunch of features for managing, structuring and even automate translations, with the most notable one being an overlay over translations keys
in the code displaying the corresponding translations.

This is very helpfull, and this section is a guide for a minimal setup to make this extension work with `Leptos i18n`.

## Custom framework setup

For obvious reason this lib is not supported by `i18n Ally` (one day maybe ?), but the awesome people working on that extension
gave us [a way](https://github.com/lokalise/i18n-ally/wiki/Custom-Framework) to make it work with custom frameworks.

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

`languageIds` is the language you are using in your project, I'm no expert but this is probably for a VSC api to know what files to check.

`usageMatchRegex` is the regex to use to find the translations keys, the above regex are for, in order, `t!`, `td!`, `td_string!` and `td_display!`. If you don't use all translations macro you can remove/comment out the regex for that macro. Those regex are not perfect, and I'm no expert so there maybe is some better/faster ones, and if you encounter a problem with them feel free to open an issue/discussion on github about it.

`monopoly` is to disable all supported frameworks, if you use any other frameworks supported by the extension in your project set it to `false`.

## Common settings

There is multiple settings for the extension that you can set in `.vscode/settings.json`, those are all optionnal, here is non exhaustive list with their (default):

- `i18n-ally.keystyle` (auto): this one can be `flat` (`"a.b.c": "..."`) or `nested` (`"a": { "b": { "c": "..." } }`), this is irrelevant to you if you don't use subkeys, but if you do, set it to `"nested"` as this is the style that this lib support.

- `i18n-ally.localesPaths` (auto): this is the path to your locales, it can be a path or a list of path, by default set it to `"locales"`, but if you either have a custom locales path or use a cargo workspaces you will have to supply the path here.

- `i18n-ally.namespace` (false): this is is if you use namespaces, set it to `true` then. If you use namespaces with `i18n Ally` I have not figured out (maybe you will ?) how to make the `namespace::key` syntax work for the macros, so just use `namespace.key`.

- `i18n-ally.sourceLanguage` (en): The primary key of the project, so I suggest putting the default locale for the value.

- `i18n-ally.displayLanguage` (en): The locale that the overlay use.

You can find other settings that could interest you in the [official doc](https://github.com/lokalise/i18n-ally/wiki/Configurations), with more informations about the settings mentionned above, with their default value.

## Other features

This extension offer some other interesting features that could interest you, I would suggest you to take a look at their [wiki](https://github.com/lokalise/i18n-ally/wiki) for mor informations.
