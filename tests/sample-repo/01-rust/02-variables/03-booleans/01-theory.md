# Booleans and Logic in Rust

The `bool` type has two values: `true` and `false`.

## Comparison Operators

| Operator | Meaning |
|----------|---------|
| `==` | Equal to |
| `!=` | Not equal to |
| `<` | Less than |
| `>` | Greater than |
| `<=` | Less than or equal |
| `>=` | Greater than or equal |

## Logical Operators

- `&&` — logical AND
- `||` — logical OR
- `!` — logical NOT

```rust
let is_adult = age >= 18;
let can_drive = is_adult && has_license;
let is_invalid = !is_adult;
```
