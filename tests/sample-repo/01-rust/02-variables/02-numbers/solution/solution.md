---
title    = "Basic Arithmetic"
hints    = [
    "Use `+` for addition and `*` for multiplication",
    "To compute an average, add both numbers then divide by 2",
    "Cast to `f64` before dividing to avoid integer truncation",
]
keywords = []
---

For `add` and `multiply`, use the `+` and `*` operators directly.

For `average`, add both numbers and divide by 2. Cast the sum to `f64` with `as f64` before dividing to avoid integer division truncation: `(a + b) as f64 / 2.0`.
