# The `td_string!` Macro

The `td_string!` macro is to use interpolations outside the context of rendering views. It lets you give a different kind of values and return a `&'static str` or a `String` depending on the value of the key.
If the value is a plain string or a boolean, it returns a `&'static str`. If it's an interpolation or a number, it returns a `String`.

This requires the `interpolate_display` feature to be enabled to work with interpolations.

It enables you to do this:

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

Variables expect anything that implements `Display`.

If the key uses ranges, it expects the type of the count. If you set the type to `f32`, it expects a `f32`.

Components expect a value that implements `leptos_i18::display::DisplayComponent`. You can find some types made to help with formatting in the `display` module,
such as `DisplayComp`.

`String` and `&str` implement this trait such that

```rust
// hello_world = "Hello <b>World</b> !"

let hw = td_string(Locale::en, hello_world, <b> = "span");
assert_eq!(hw, "Hello <span>World</span> !");
```

The `DisplayComp` struct lets you pass leptos attributes:

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

which basically lets you do this:

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

If you look closely, there are no `Clone` or `'static` bounds for any arguments, but they are captured by the value returned by the macro,
so the returned value has a lifetime bound to the "smallest" lifetime of the arguments.

# The `td_display!` Macro

Just like the `td_string!` macro but returns either a struct implementing `Display` or a `&'static str` instead of a `Cow<'static, str>`.

This is useful if you will print the value or use it in any formatting operation, as it will avoid a temporary `String`.

```rust
use crate::i18n::Locale;
use leptos_i18n::td_display;

// click_count = "You clicked {{ count }} times"
let t = td_display!(Locale::en, click_count, count = 10); // this only returns the builder, no work has been done.
assert_eq!(format!("before {t} after"), "before You clicked 10 times after");

let t_str = t.to_string(); // can call `to_string` as the value implements `Display`
assert_eq!(t_str, "You clicked 10 times");
```

# `t_string`, `t_display`, `tu_string` and `tu_display`

They also exist, `td_string` was used here for easier demonstration. Remember that `t_string` accesses a signal reactively.
