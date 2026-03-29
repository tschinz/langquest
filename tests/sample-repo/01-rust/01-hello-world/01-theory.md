# Functions in Rust

In Rust, functions are declared with `fn` and can return values.

## Return Types

A function's return type is specified after `->`:

```rust
fn get_number() -> i32 {
    42
}
```

## Expression-Based Returns

The **last expression** in a function (without a semicolon) becomes its return value:

```rust
fn double(x: i32) -> i32 {
    x * 2  // no semicolon = return value
}
```

Adding a semicolon turns it into a statement returning `()` (unit type).

## String Slices

The type `&'static str` is a string slice with a static lifetime — string literals are always this type:

```rust
fn get_text() -> &'static str {
    "some text"
}
```
