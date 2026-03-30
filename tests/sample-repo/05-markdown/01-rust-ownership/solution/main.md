# Question: What is ownership in Rust?

Ownership is Rust's memory management system. Each value has exactly one owner, and when the owner goes out of scope, the value is dropped.

The three ownership rules are:
1. Each value has an owner
2. There can only be one owner at a time
3. When the owner goes out of scope, the value is dropped

You can transfer ownership using `move` semantics, or borrow a reference using `&` for shared access or `&mut` for mutable access.