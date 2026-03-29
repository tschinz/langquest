---
title    = "Hello, World!"
hints    = [
    "The return type is &'static str - a string literal works perfectly.",
    "In Rust, the last expression without a semicolon is returned.",
    "No need for the return keyword - just write the string.",
]
keywords = []
---

## Explanation

In Rust, functions return the last expression when there's no semicolon. String literals are `&'static str` by default.

```rust
// Returns a static string slice - the greeting lives in the binary
fn greeting() -> &'static str {
    "Hello, World!"  // No semicolon = this is the return value
}

fn main() {
    println!("{}", greeting());  // {} is a format placeholder
}
```

**Key concepts:**
- `&'static str` - a string slice that lives for the entire program
- Expression-based return - no `return` keyword needed
- `println!` macro - the `!` indicates it's a macro, not a function
