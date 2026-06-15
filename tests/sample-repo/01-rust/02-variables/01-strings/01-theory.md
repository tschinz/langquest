# Strings in Rust

Rust has two main string types:

## `&str` (String Slice)

An immutable reference to a string stored elsewhere. String literals like `"hello"` are `&str`:

```rust
let greeting: &str = "Hello";
```

## `String` (Owned String)

A growable, heap-allocated string:

```rust
let mut name = String::from("Alice");
name.push_str(" Bob");
```

## Concatenation

Use `+` or `format!()` to combine strings:

```rust
let result = format!("{}-{}", "foo", "bar");
```
