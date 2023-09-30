# Mixing Kinds

What happen if for a key you declare plurals in one locale, interpolation in another, and a simple string in a third ?

Well this is totally allowed, but you will still need to supply all values/component of every locales combined when using the translation, regardless of what the current locale is.

What is not allowed to mix is subkeys, the key must be subkeys in all locale.
