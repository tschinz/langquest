---
title    = "Hello, World!"
hints    = [
    "The function signature already tells you the return type: &'static str",
    "A string literal like \"Hello, World!\" is already a &'static str",
    "In Rust, the last expression in a function (without a semicolon) is the tttreturn value",
    "You don't need the `return` keyword — just write the string literal as the final expression",
]
keywords = [
    "fn",
    "main",
    "println",
    "macro",
    "&'static str",
    "return value",
    "string literal",
]
---

The simplest Rust program starts with `fn main()`, which is the entry point for every executable. In this exercise, we extracted the greeting into its own function to make it testable.

`fn greeting() -> &'static str` declares a function that returns a string slice with a `'static` lifetime — meaning the string lives for the entire duration of the program. String literals in Rust are always `&'static str` because they are embedded directly in the compiled binary.

The solution is simply:

```
fn greeting() -> &'static str {
    "Hello, World!"
}
```

Notice there is no `return` keyword and no semicolon. In Rust, the last expression in a block becomes the block's value. Adding a semicolon would turn the expression into a statement that returns `()` (the unit type), which would cause a type mismatch.

The `println!` macro (note the `!` — it's a macro, not a regular function) handles formatted output to stdout. The `"{}"` is a format placeholder that will be replaced with the value of `greeting()`.
