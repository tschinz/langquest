# Ownership in Rust

Rust uses a unique system called **ownership** to manage memory without a garbage collector.

## The Three Rules

Every value in Rust has exactly one owner, and these rules apply:

1. Each value has a variable that's its **owner**
2. There can only be **one owner** at a time
3. When the owner goes out of scope, the value is **dropped**

## Move Semantics

When you assign a value to another variable, ownership **moves**:

```rust
let s1 = String::from("hello");
let s2 = s1;  // s1 is now invalid — ownership moved to s2
```

After the move, using `s1` would cause a compile error.

## Borrowing

Instead of transferring ownership, you can **borrow** a value using references:

```rust
let s1 = String::from("hello");
let len = calculate_length(&s1);  // borrow s1
// s1 is still valid here
```

## Why Ownership Matters

This system prevents:
- Use-after-free bugs
- Double-free bugs
- Data races in concurrent code

All checked at **compile time** with zero runtime cost.