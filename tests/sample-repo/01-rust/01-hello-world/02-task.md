---
id          = "hello_world"
name        = "Hello, World!"
language    = "rust"
difficulty  = 1
description = "Write your first Rust function that returns a greeting string."
topics      = ["println", "main", "basic_syntax"]
---

# Hello, World!

## Objective

Make the program print **exactly** `Hello, World!` to standard output.

## Instructions

1. Open `main.rs` and find the `greeting()` function.
2. Replace the `todo!()` macro with an expression that returns the string `"Hello, World!"`.
3. The `main()` function already calls `greeting()` and prints the result — you only need to fix `greeting()`.

## Requirements

- `greeting()` must return a `&'static str` with the value `"Hello, World!"`.
- Do **not** modify the `main()` function or the test module.

## Running

```sh
# Run the program
cargo run

# Run the tests to verify your solution
cargo test
```

## Expected Output

```
Hello, World!
```
