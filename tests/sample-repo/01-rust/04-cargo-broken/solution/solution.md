---
title    = "Meet Cargo"
hints    = [
    "Look at the **Debug** page: Cargo fails before the compiler even runs.",
    "Two things are broken in `Cargo.toml`: the `edition` must be a real one and the `path` in `[[bin]]` must point at the file that actually contains the code.",
    "```toml\n[package]\nname = \"excercise\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[[bin]]\nname = \"excercise\"\npath = \"main.rs\"\n```",
]
keywords = ["Cargo", "build"]
---

## Explanation

The Rust code in this exercise needed **zero changes** - both bugs were in `Cargo.toml`:

1. **`edition = "2025"`**: there is no such edition. The fix is a real, current edition, e.g. `edition = "2024"`.

2. **`path = "source/main.rs"`**: the `[[bin]]` target told Cargo to look for the source in a folder that does not exist, so it couldn't find the program to compile.
