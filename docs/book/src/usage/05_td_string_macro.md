# The `td_string!` Macro

The `td_string!` macro is to use interpolations outside the context of rendering views, it let you give a different kind of values such that the resulting value implement `Display` (and thus inherit the `ToString` impl).

```rust
// click_count = "You clicked {{ count }} times"
assert_eq!(
    td_string!(Locale::en, click_count, count = 10).to_string(),
    "You clicked 10 times"
)
assert_eq!(
    td_string!(Locale::en, click_count, count = "a lot of").to_string(),
    "You clicked a lot of times"
)
```

### Expected values

Variables expect anything that implement `Display`.

If the key use plurals it expect the type of the count, if you set the type to `f32`, it expect a `f32`.

Components are a bit trickier, they expect this abomination:

```rust
Fn(&mut core::fmt::Formatter, &dyn Fn(&mut core::fmt::Formatter) -> core::fmt::Result) -> core::fmt::Result
```

which basically let you do this:

```rust
use core::fmt::{Formatter, Result};

fn render_b(f: &mut Formatter, child: &dyn Fn(&mut Formatter) -> Result) -> Result {
    write!(f, "<div>")?;
    child(f)?;
    write!(f, "</div>")
}

// hello_world = "Hello <b>World</b> !"
let hw = td_string!(Locale::en, hello_world, <b> = render_b);
println!("{}", hw); // print "Hello <div>World</div> !"
```

If you look closely, there is no `Clone` or `'static` bounds for any arguments, but they are captured by the value returned by the macro,
so the returned value as a lifetime bound to the "smallest" lifetime of the arguments.
