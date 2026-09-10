---
id          = "intro_cargo"
name        = "Meet Cargo"
language    = "rust"
difficulty  = 1
description = "Learn how Cargo builds a project by fixing a broken Cargo.toml."
topics      = ["cargo", "toml", "manifest", "build", "editions"]
source      = "Cargo.toml"
---

# Meet Cargo

The source code in this exercise is correct. The problem is in `Cargo.toml`: the manifest contains incorrect values, so cargo can't build the project.

## Your Task

1. `Cargo.toml` declares an `edition` that does not exist. Replace it with the newest real edition.
2. The `[[bin]]` target points at a source file that does not exist. point it to the correct file.
3. Do **not** change `main.rs`.

## Expected Result

Both tests pass:

- `course_language()` equals `"Rust"`.
- `course_edition()` equals `"2024"`.
