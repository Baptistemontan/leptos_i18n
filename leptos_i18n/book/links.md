Link definitions between chapters, for mdbook.

A chapter that links to another one uses reference-style links ([text][label]) and ends with an
HTML comment containing an mdbook include of this file (see the end of usage/07_router.md).
rustdoc ignores that comment, while mdbook replaces the include with this file. This file closes
the comment right below and reopens it at its end, so the definitions in between are live for
mdbook, while this text stays hidden. Do not write the comment closing marker or an include
directive in this text.

The paths are relative to a chapter in a sub-folder (e.g. usage/), so only those chapters can
include this file. The rustdoc side of these links is in rustdoc_links.md: keep both in sync.
-->

[key_value_keys]: ../declare/01_key_value.md#keys
[router_caveat]: ../usage/07_router.md#caveat
[locale_resolution]: ../infos/01_locale_resol.md

<!--
