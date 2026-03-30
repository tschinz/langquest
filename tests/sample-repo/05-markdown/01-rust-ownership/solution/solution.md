---
title    = "Rust Ownership"
hints    = [
    "Think about how Rust manages memory without a garbage collector.",
    "Each value in Rust has exactly one variable that owns it.",
    "When the owner goes out of scope, the value is dropped automatically.",
    "Keywords to mention - ownership, owner, scope, drop, move, borrow",
]
keywords = ["ownership", "owner", "scope", "drop", "move", "borrow"]
---

## Explanation

Rust uses **ownership** to manage memory safely without a garbage collector.

The three ownership rules are:

1. Each value has exactly one **owner** (a variable)
2. There can only be one owner at a time
3. When the owner goes out of **scope**, the value is **dropped**

```rust
fn main() {
    let s = String::from("hello");  // s owns the String
    let t = s;                       // ownership moves to t, s is invalid
    // println!("{}", s);            // ERROR: s no longer owns the value
    println!("{}", t);               // OK: t is the owner
}   // t goes out of scope, String is dropped
```

**Key concepts:**
- **Ownership** — the system that tracks which variable owns a value
- **Move** — transferring ownership from one variable to another
- **Drop** — automatic cleanup when the owner goes out of scope
- **Borrow** — temporarily accessing a value without taking ownership
