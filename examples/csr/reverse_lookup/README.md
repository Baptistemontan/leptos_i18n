# Reverse Lookup Example

This example demonstrates how to use `translation_map_builder` (behind the `reverse_lookup` feature) to translate dynamic strings received at runtime from a server you don't control.

It fetches a string from an external API (httpbin.org) and uses locale JSON files to build a map from the default locale's values to the current locale's values.

## How to run

Simply use `trunk` to run it:

```bash
trunk serve --open
```
