# Cargo: The Very Basics

## What is Cargo?

When you write Rust, you don't manage the project by hand. **Cargo** is the tool that does all the work for you: it figures out what dependencies to download, builds them, compiles your code, runs it, runs your tests and more.

## What does a Cargo project look like?

When you run `cargo new my_project`, Cargo creates following folder-structure:

```
my_project/
├── Cargo.toml      <-- the "recipe" of the project (a text file)
└── src/
    └── main.rs     <-- the actual Rust code
```

This is the typical rust project structure, you can also change it to a certain degree by setting options in `Cargo.toml`.

## The manifest: `Cargo.toml`

`Cargo.toml` is a text file in the **TOML** format. TOML files are made of sections and `key = value` pairs. Here is an example:

```toml
[package]              # a section: describes the project itself
name = "excercise"     # its name
version = "0.1.0"      # its version (most often in sem-ver)
edition = "2021"       # which 'version-family' to used

[[bin]]                # a section: one executable
name = "excercise"     # the name of the produced program
path = "src/main.rs"       # which file the entrypoint to the source code is in
```

The **edition** (`2024`, `2021`, `2018`, ...) specifies which 'version-family' the code is written in. New editions bring breaking changes and new features, so setting this to the correct value is necessary for the code to compile successfully.

## The most important commands

| Command         | What it does                                              |
|-----------------|-----------------------------------------------------------|
| `cargo new`     | Creates a fresh project.                                  |
| `cargo build`   | Compiles the project into a binary.                       |
| `cargo run`     | Builds (if needed) and runs the project.                  |
| `cargo test`    | Builds in test mode and runs all `#[test]` functions.     |

