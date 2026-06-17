---
title    = "Basic Arithmetic"
hints    = [
    "To build up a total you need to update a value on each iteration. Is a plain `let` binding enough, or do you need something extra?",
    "Add `mut` to make the binding mutable: `let mut sum = 0.0;`. Then loop over the slice with `for &r in readings` to get each `f64` value.",
    "```rust\nfn zero_pad(n: u32) -> String {\n    format!(\"...\")\n}\n```",
    "```rust\nfn zero_pad(n: u32) -> String {\n    format!(\"{n:08}\")\n}\n```",
]
keywords = []
---

For `add` and `multiply`, use the `+` and `*` operators directly.

For `average`, add both numbers and divide by 2. Cast the sum to `f64` with `as f64` before dividing to avoid integer division truncation: `(a + b) as f64 / 2.0`.
