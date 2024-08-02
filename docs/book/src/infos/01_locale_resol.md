# Locale Resolution

This library handles the detection of what locale to use for you, but it can be done in a multiple of ways.

Here is the list of detection methods, sorted in priorities:

1. A locale prefix is present in the URL pathname when using `I18nRoute` (e.g. `/en/about`)
1. A cookie is present that contains a previously detected locale
1. A locale can be matched based on the [`Accept-Language` header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Accept-Language) in SSR
1. A locale can be matched base on the [`navigator.languages` API](https://developer.mozilla.org/en-US/docs/Web/API/Navigator/languages) in CSR
1. As a last resort, the default locale is used.

In SSR it is always the server that resolves what locale to use, the client do not tries to compute a locale when loading, the only locale changes that can happen is by explicitly setting in in the context.
