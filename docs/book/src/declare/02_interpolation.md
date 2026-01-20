# Interpolation

## Interpolate Values

There may be situations where you must interpolate a value inside your translations, for example, a dynamic number.
You could declare 2 translations and use them with that number, but this is not an elegant solution.

To declare a value that will be interpolated in your translations, simply give it a name surrounded by `{{ }}`:

```json
{
  "click_count": "You clicked {{ count }} times"
}
```

## Interpolate Components

There may also be situations where you want to wrap part of your translation into a component, for example, to highlight it.

You can declare a component with HTML-like syntax:

```json
{
  "highlight_me": "highlight <b>me</b>"
}
```

Or self-closed components:

```json
{
  "with_break": "some line <br /> some other line"
}
```

## Use both

You can mix both interpolated values and interpolated components:

```json
{
  "click_count": "You clicked <b>{{ count }}</b> times"
}
```

## Components attributes

You can pass attributes to the components:

```json
{
  "highlight_me": "highlight <b id=\"john\">me</b>"
}
```

The values the attributes accept are:

- strings
- booleans
- numbers (signed, unsigned, floats),
- variables

The syntax for variables:

```json
{
  "with_break": "some line <br id={{ id }} /> some other line"
}
```

## Values Names.

Values names must follow the same rules as [keys](./01_key_value.md#keys).
