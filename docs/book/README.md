This book is a documentation on how to setup and use the `leptos_i18` lib.

The chapters live in [`leptos_i18n/book`](../../leptos_i18n/book), inside the crate, so they are published with it and also rendered as rustdoc under `leptos_i18n::book` (see `book.toml`).

Links between chapters are reference-style links, defined once for mdbook in `leptos_i18n/book/links.md` and once for rustdoc in `leptos_i18n/book/rustdoc_links.md`; `links.md` explains how a chapter uses them.

It is built using `mdbook`. You can view a local copy by installing `mdbook`

```bash
cargo install mdbook
```

and run the book with

```
mdbook serve
```

It should be available at `http://localhost:3000`.
