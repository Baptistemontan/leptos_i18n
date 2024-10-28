# The `td_string!` Macro

The `td_string!` macro is to use interpolations outside the context of rendering views, it let you give a different kind of values and return a `&'static str` or a `String` depending on the value of the key.
If the value is a plain string or a boolean it returns a `&'static str`, if it's an interpolations or a number it returns a `String`.

This requires the `interpolate_display` feature to be enabled to work with interpolations.

It enable you to do this:

```rust
// click_count = "You clicked {{ count }} times"
assert_eq!(
    td_string!(Locale::en, click_count, count = 10),
    "You clicked 10 times"
)
assert_eq!(
    td_string!(Locale::en, click_count, count = "a lot of"),
    "You clicked a lot of times"
)
```

### Expected values

Variables expect anything that implement `Display`.

If the key use ranges it expect the type of the count, if you set the type to `f32`, it expect a `f32`.

Components expect a value that implement `leptos_i18::display::DisplayComponent`, you can find some type made to help the formatting in the `display` module,
such as `DisplayComp`.

`String` and `&str` implement this trait such that

```rust
// hello_world = "Hello <b>World</b> !"

let hw = td_string(Locale::en, hello_world, <b> = "span");
assert_eq!(hw, "Hello <span>World</span> !");
```

The `DisplayComp` struct let you pass leptos attributes:

```rust
let attrs = [("id", leptos::Attribute::String("my_id".into()))];
let b = DisplayComp::new("div", &attrs);
let hw = td_string!(Locale::en, hello_world, <b>);
assert_eq!(hw, "Hello <div id=\"my_id\">World</div> !");
```

If you want finer control over the formatting, you can create your own types implementing the `DisplayComponent` trait, or you can pass this abomination of a function:

```rust
Fn(&mut core::fmt::Formatter, &dyn Fn(&mut core::fmt::Formatter) -> core::fmt::Result) -> core::fmt::Result
```

which basically let you do this:

```rust
use core::fmt::{Formatter, Result};

fn render_b(f: &mut Formatter, child: &dyn Fn(&mut Formatter) -> Result) -> Result {
    write!(f, "<div id=\"some_id\">")?;
    child(f)?; // format the children
    write!(f, "</div>")
}

// hello_world = "Hello <b>World</b> !"
let hw = td_string!(Locale::en, hello_world, <b> = render_b);
assert_eq!(hw, "Hello <div id=\"some_id\">World</div> !");
```

If you look closely, there is no `Clone` or `'static` bounds for any arguments, but they are captured by the value returned by the macro,
so the returned value as a lifetime bound to the "smallest" lifetime of the arguments.

# The `td_display!` Macro

Just like the `td_string!` macro but return either a struct implementing `Display` or a `&'static str` instead of a `Cow<'static, str>`.

This is useful if you will print the value or use it in any formatting operation, as it will avoid a temporary `String`.

```rust
use crate::i18n::Locale;
use leptos_i18n::td_display;

// click_count = "You clicked {{ count }} times"
let t = td_display!(Locale::en, click_count, count = 10); // this only return the builder, no work has been done.
assert_eq!(format!("before {t} after"), "before You clicked 10 times after");

let t_str = t.to_string(); // can call `to_string` as the value impl `Display`
assert_eq!(t_str, "You clicked 10 times");
```

# `t_string` and `t_display`

They also exist, `td_string` was used here for easier demonstration. Remember that `t_string` access a signal reactively.
