![GitHub Repo stars](https://img.shields.io/github/stars/tschinz/langquest)
![GitHub Release](https://img.shields.io/github/v/release/tschinz/langquest)
![](https://tianji.zahno.dev/telemetry/clnzoxcy10001vy2ohi4obbi0/cmn9yy8dy1cc9sjrz8b6ejk3v.gif)

<div align="center">
  <img src="img/lq.svg" alt="LangQuest logo" width="150">
</div>

# lq — LangQuest

A terminal-based, interactive programming exercise runner. Inspired by [Rustlings](https://github.com/rust-lang/rustlings) and [100 Exercises to Learn Rust](https://rust-exercises.com/), LangQuest extends the concept to multiple languages — work through hands-on exercises in **Rust**, **Go**, **Python**, **RISC-V assembly**, and **Markdown** with real-time feedback, progress tracking, and a built-in hint system.

---

## Table of Contents

- [Features](#features)
- [Installation](#installation)
  - [Installing lq](#installing-lq)
  - [Exercise Toolchains](#exercise-toolchains)
- [Getting Started](#getting-started)
  - [Creating Your Exercise Repository](#creating-your-exercise-repository)
  - [Launching lq](#launching-lq)
  - [Configuration File (lq.toml)](#configuration-file-lqtoml)
- [Creating Your Own Exercises](#creating-your-own-exercises)
  - [File Structure](#file-structure)
  - [Exercise Contents](#exercise-contents)
- [CLI Reference](#cli-reference)
- [Dependencies](#dependencies)
- [License](#license)

---

## Features

- **Multi-language support** — Rust, Go, Python, RISC-V assembly, and Markdown/conceptual exercises
- **Live verification** — file saves trigger immediate re-runs; results stream into the TUI without leaving the editor
- **Paged exercise view** — Theory → Task → Output → Solution, navigated with arrow keys
- **Progressive hints** — reveal hints one at a time; after all hints, optionally unlock the full solution
- **Syntax-highlighted solutions** — reference code and prose explanations, gated until pass or explicit unlock
- **Overview with tree panel** — scrollable exercise table and module/exercise tree with live progress
- **Persistent progress** — `lq.toml` at the repo root tracks scores, pass status, and solution visibility

---

## Installation

### Installing lq

**Prerequisites:** Rust toolchain (edition 2024, Rust ≥ 1.85)

```sh
# Clone and install
git clone https://github.com/tschinz/langquest.git
cd langquest
cargo install --path .

# Or run directly without installing
cargo run -- --repo /path/to/exercises

# of via crates.io
cargo install langquest
lq --repo /path/to/exercises
```

### Exercise Toolchains

Depending on which languages your exercises use, install the corresponding toolchains:

| Language | Installation |
|----------|--------------|
| **Rust** | Install via [rustup](https://rustup.rs/) |
| **Python** | Install Python 3.x and pytest: `pip install pytest` |
| **Go** | Install from [go.dev](https://go.dev/dl/) or via package manager |
| **RISC-V** | GNU toolchain (`apt install gcc-riscv64-linux-gnu`) or [Ripes](https://github.com/mortbopet/Ripes) simulator |
| **Markdown** | No additional tools required — verification is regex-based |

---

## Getting Started

### Creating Your Exercise Repository

Create a new directory for your exercises. The structure follows a simple **modules → exercises** hierarchy:

```
my-exercises/
├── lq.toml                      ← auto-created by lq on first run
├── 01-basics/                   ← module (prefixed with NN-)
│   ├── 01-hello-world/          ← exercise (prefixed with NN-)
│   │   ├── 01-theory.md
│   │   ├── 02-task.md
│   │   ├── main.rs
│   │   └── solution/
│   │       ├── main.rs
│   │       └── solution.md
│   └── 02-variables/
│       └── ...
└── 02-control-flow/
    └── ...
```

**Naming conventions:**
- Module and exercise directories must be prefixed with a two-digit number (`01-`, `02-`, …)
- Use lowercase kebab-case: `01-hello-world`, `02-variables`
- Directories without the numeric prefix are ignored

### Launching lq

```sh
# Point lq at your exercise repository
lq --repo /path/to/my-exercises

# Or cd into the repo first
cd /path/to/my-exercises
lq
```

### Configuration File (lq.toml)

`lq` creates and manages `lq.toml` at the root of your exercise repository. This file tracks all progress:

```toml
current_exercise = "01-basics/02-variables"

[exercises."01-basics/01-hello-world"]
best_score    = 1.0
passed        = true
solution_seen = true

[exercises."01-basics/02-variables"]
best_score    = 0.6
passed        = false
solution_seen = false
```

**Persistence rules:**
- `best_score` only increases — lower scores never overwrite higher ones
- `passed` becomes `true` when `score >= threshold` and never resets
- `solution_seen` becomes `true` on first Solution page visit and never resets

You can commit `lq.toml` to share progress across machines, or add it to `.gitignore` for single-user use.

---

## Creating Your Own Exercises

### File Structure

Each exercise lives in its own directory within a module:

```
<NN>-<module>/
└── <NN>-<exercise>/
    ├── 01-theory.md           ← optional background reading
    ├── 02-task.md             ← required task description with frontmatter
    ├── main.<ext>             ← student source file (rs, go, py, md, asm)
    └── solution/
        ├── main.<ext>         ← reference solution
        └── solution.md        ← hints and explanation
```

### Exercise Contents

#### 01-theory.md (Optional)

Background reading rendered on the Theory page. Plain Markdown, no special requirements.

#### 02-task.md (Required)

The task description with **required TOML frontmatter**:

```markdown
---
id          = "hello_world"
name        = "Hello, World!"
language    = "rust"
difficulty  = 2
description = "Implement a function that returns a greeting string."
topics      = ["functions", "strings", "return_values"]
---

# Hello, World!

Your task is to implement the `greeting()` function so that it returns
the string `"Hello, World!"` exactly.
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `id` | string | ✓ | Unique snake_case identifier (key in `lq.toml`) |
| `name` | string | ✓ | Display name in the exercise table |
| `language` | string | ✓ | `rust`, `go`, `python`, `riscv`, or `text` |
| `difficulty` | integer 1–5 | ✓ | Shown as stars in the Overview |
| `description` | string | ✓ | One-line summary |
| `topics` | array | ✓ | Tags shown in the Topics column |

#### Student Source File (main.*)

The file extension determines the language and verification method:

**Rust** (`main.rs`) — Uses `#[test]` functions with `// TODO` markers:

```rust
// TODO: implement the add function
fn add(a: i32, b: i32) -> i32 {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_positive() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add(-1, 1), 0);
    }
}
```

**Go** (`main.go` + `main_test.go`) — Uses `TestXxx` functions:

```go
// TODO: implement the Add function
func Add(a, b int) int {
    return 0
}
```

**Python** (`main.py`) — Uses unittest or pytest:

```python
# TODO: implement the add function
def add(a: int, b: int) -> int:
    pass

def test_add_positive():
    assert add(2, 3) == 5

def test_add_negative():
    assert add(-1, 1) == 0
```

**RISC-V Assembly** (`main.asm`) — Uses `EXPECT_*` directives:

```asm
; EXPECT_EXIT: 0
; EXPECT_STDOUT: Hello, World!

; TODO: implement the program
.global _start
_start:
    ; your code here
```

| Directive | Description |
|-----------|-------------|
| `EXPECT_EXIT: N` | Process must exit with code N |
| `EXPECT_STDOUT: S` | stdout must contain S as substring |

**Markdown** (`main.md`) — Free-text answers matched against keywords:

```markdown
# Question: What keyword makes a Rust variable mutable?

<!-- Write your answer below -->

```

#### Solution Folder

**solution/main.*** — The complete reference solution:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_positive() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add(-1, 1), 0);
    }
}
```

**solution/solution.md** — Hints, keywords, and explanation with frontmatter:

```markdown
---
title    = "Adding Numbers"
hints    = [
    "The function should return the sum of both parameters.",
    "Use the + operator to add two integers.",
    "Rust returns the last expression without a semicolon.",
]
keywords = ["mut", "let", "i32"]
---

## Explanation

To add two numbers in Rust, simply use the `+` operator. The function
returns the last expression automatically when there's no semicolon.

The `keywords` array is used for Markdown/conceptual exercises to score
free-text answers via regex matching.
```

| Field | Description |
|-------|-------------|
| `title` | Display name for the solution |
| `hints` | Ordered list revealed one at a time with `h` |
| `keywords` | Regex patterns for scoring Markdown exercises |
| body | Prose explanation shown on the Solution page |

---

## CLI Reference

```
LangQuest — interactive programming exercises

Usage: lq [OPTIONS] [COMMAND]

Commands:
  status  Print current exercise and overall progress
  help    Print this message or the help of the given subcommand(s)

Options:
      --repo <REPO>  Path to exercise repository root
      --reset        Wipe all progress in lq.toml and start fresh
  -h, --help         Print help
  -V, --version      Print version
```

**Examples:**

```sh
# Launch TUI with exercise repository
lq --repo /path/to/exercises

# Check progress without launching TUI
lq status

# Reset all progress (prompts for confirmation)
lq --reset
```

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` | TUI rendering framework |
| `crossterm` | Cross-platform terminal backend |
| `clap` | CLI argument parsing |
| `notify` | File-system watcher for live verification |
| `syntect` | Syntax highlighting on the Solution page |
| `pulldown-cmark` | Markdown rendering |
| `toml` + `serde` | Configuration and frontmatter parsing |
| `regex` | Keyword matching for Markdown exercises |
| `anyhow` | Error propagation |
| `thiserror` | Typed domain errors |

---

## License

This project is licensed under the MIT License — see the [LICENSE](LICENSE) file for details.
