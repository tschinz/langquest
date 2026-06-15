# Numeric Types in Rust

Rust provides several integer and floating-point types:

## Integers

| Type | Size | Signed? |
|------|------|---------|
| `i32` | 32-bit | Yes |
| `u32` | 32-bit | No |
| `i64` | 64-bit | Yes |
| `u64` | 64-bit | No |

## Floats

| Type | Size |
|------|------|
| `f32` | 32-bit |
| `f64` | 64-bit (default) |

## Arithmetic

Basic operators: `+`, `-`, `*`, `/`, `%` (remainder).

```rust
let sum = 10 + 20;
let product: i64 = 5 * 6;
let quotient = 7.0 / 3.0;
```
