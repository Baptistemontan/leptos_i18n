# The `td_string!` Macro

The `td_string!` macro is used for interpolations outside the context of rendering views. It lets you provide different kinds of values and returns either a `&'static str` or a `String` depending on the value of the key.
If the value is a plain string or boolean, it returns a `&'static str`. If it's an interpolation or a number, it returns a `String`.

This requires the `interpolate_display` feature to be enabled to work with interpolations.

It enables you to do this:

```rust,ignore
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

If the key uses plurals, it expects the type of the count. If you set the type to `f32`, it expects a `f32`.

Components expect a value that implements `leptos_i18::display::DisplayComponent<M>`. You can find some types made to help with formatting in the `display` module,
such as `DisplayComp`. (`M` is a marker for `Fn` trait shenanigans, if you implement the trait yourself you can set it to `()`.)

`str`, `String`, and references to them implement this trait such that

```rust,ignore
// hello_world = "Hello <b>World</b> !"

let hw = td_string!(Locale::en, hello_world, <b> = "span");
assert_eq!(hw, "Hello <span>World</span> !");
```

The `DisplayComp` struct lets you pass attributes:

```rust,ignore
let attrs = [("id", "my_id")];
let b = DisplayComp::new("div", &attrs);
let hw = td_string!(Locale::en, hello_world, <b>);
assert_eq!(hw, "Hello <div id=\"my_id\">World</div> !");
```

If you want finer control over the formatting, you can create your own types implementing the `DisplayComponent` trait, or you can pass this abomination of a function:

```rust,ignore
Fn(&mut core::fmt::Formatter, leptos_i18n::display::Children, leptos_i18n::display::Attributes) -> core::fmt::Result
```

which basically lets you do this:

```rust,ignore
use core::fmt::{Formatter, Result};
use leptos_i18n::display::{Attributes, Children};

fn render_b(f: &mut Formatter, child: Children, attrs: Attributes) -> Result {
    write!(f, "<div{attrs} id=\"some_id\">{child}</div>")
}

// hello_world = "Hello <b foo={{ foo }}>World</b> !"
let hw = td_string!(Locale::en, hello_world, foo = "bar", <b> = render_b);
assert_eq!(hw, "Hello <div foo=\"bar\" id=\"some_id\">World</div> !");
```

> _note_: values for attributes must implement the `leptos_i18n::display::AttributeValue` trait, already implemented for numbers (u\*, i\*, f\* and NonZero<N>), str, bools and `Option<impl AttributeValue>`

If you look closely, there are no `Clone` or `'static` bounds for any arguments, but they are captured by the value returned by the macro,
so the returned value has a lifetime bound to the "smallest" lifetime of the arguments.

Components with children can accept `Fn(&mut Formatter, Children, Attributes)` or `Fn(&mut Formatter, Children)`,
and self-closing components can accept `Fn(&mut Formatter, Attributes)` or `Fn(&mut Formatter)`.

# The `td_display!` Macro

Just like the `td_string!` macro but returns either a struct implementing `Display` or a `&'static str` instead of a `Cow<'static, str>`.

This is useful if you will print the value or use it in any formatting operation, as it will avoid a temporary `String`.

```rust,ignore
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
