# Hello, World!

Every programmer's journey begins with a simple greeting. In Rust, this tradition
is no different — but even this tiny program introduces several important concepts.

## The `main` Function

Every Rust program starts execution in the `main` function:

```rust
fn main() {
    // your code here
}
```
- `fn` declares a function.
- `main` is a special name — it is the **entry point** of the program.
- The curly braces `{}` define the **body** of the function.

## The `println!` Macro

To print text to the console, Rust provides the `println!` macro:

```rust
println!("Hello, World!");
```

A few things to note:

- The `!` after `println` means it is a **macro**, not a regular function.
  Macros can do things that functions cannot — like accepting a variable number
  of arguments.
- The text inside the double quotes is a **string literal**.
- Each statement ends with a **semicolon** `;`.

## Format Strings

`println!` supports placeholders using `{}`:

```rust
let name = "Rust";
println!("Hello, {}!", name);
// Output: Hello, Rust!
```

You can also use numbered or named placeholders:

```rust
println!("{0} is {1} and {1} is {0}", "up", "down");
println!("{language} is fun!", language = "Rust");
```

## Functions That Return Values

In Rust, functions can return values. The return type is specified after `->`:

```rust
fn greeting() -> &'static str {
    "Hello, World!"
}
```

Notice there is **no semicolon** on the last expression — this makes it the
function's **return value**. This is called an *expression-based return*. You
could also write `return "Hello, World!";` explicitly, but idiomatic Rust
prefers the expression form.

## Putting It Together

A complete program that uses a helper function:

```rust
fn greeting() -> &'static str {
    "Hello, World!"
}

fn main() {
    println!("{}", greeting());
}
```

This pattern — separating logic into small functions — makes code easier to
**test** and **reuse**.
