# Mixing Kinds

What happens if for a key you declare plurals in one locale, interpolation in another, and a simple string in a third ?

Well, this is totally allowed, but you will still need to supply all values/components of every locale combined when using the translation, regardless of what the current locale is.

What is not allowed to mix are subkeys. If a key has subkeys in one locale, the key must have subkeys in all locales.
