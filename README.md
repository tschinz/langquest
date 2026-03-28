![GitHub Repo stars](https://img.shields.io/github/stars/tschinz/lq)
![GitHub Release](https://img.shields.io/github/v/release/tschinz/lq)
![](https://tianji.zahno.dev/telemetry/clnzoxcy10001vy2ohi4obbi0/cmn9yy8dy1cc9sjrz8b6ejk3v.gif)

<div align="center">
  <img src="img/md-pdf.svg" alt="md-pdf logo" width="150">
</div>

# lq — LangQuest

A terminal-based programming exercise tool. Work through hands-on exercises in Rust, Go, Python, RISC-V assembly, and Markdown — with real-time feedback, progress tracking, and a built-in hint system, all inside your terminal.

---

## Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Exercise Repository Layout](#exercise-repository-layout)
  - [Directory Structure](#directory-structure)
  - [Task Frontmatter](#task-frontmatter-02-taskmd)
  - [Student Source Files](#student-source-files)
  - [Solution Files](#solution-files-solutionsolutionmd)
- [Verification](#verification)
  - [Rust](#rust)
  - [Go](#go)
  - [Python](#python)
  - [Assembly (RISC-V)](#assembly-risc-v)
  - [Markdown / Conceptual](#markdown--conceptual)
- [TUI Reference](#tui-reference)
  - [Exercise View](#exercise-view)
  - [Overview](#overview)
  - [Keybindings](#keybindings)
  - [Hints & Solution Unlock](#hints--solution-unlock)
- [Progress Tracking](#progress-tracking)
- [CLI Reference](#cli-reference)
- [Creating Exercises](#creating-exercises)
- [Dependencies](#dependencies)

---

## Features

- **Multi-language** — Rust, Go, Python, RISC-V assembly, and
  Markdown/conceptual question exercises.
- **Live verification** — saves trigger an immediate re-run; results stream into
  the Output page without leaving the TUI.
- **Paged exercise view** — Theory → Task → Output → Solution, navigated with
  arrow keys. Scroll position shown as a `[N%]` indicator in each page header.
- **Progressive hints** — reveal one hint at a time with `h`. After all hints
  are exhausted, a second confirmation unlocks the full solution.
- **Solution page** — syntax-highlighted reference code plus a prose
  explanation. Gated until the exercise is passed *or* explicitly unlocked
  through the hint flow.
- **Overview with tree panel** — scrollable exercise table and a module/exercise
  tree panel, both reflecting live progress. Toggle the tree with `t`.
- **Persistent progress** — a single `lq.toml` at the repo root records every
  score, pass status, and solution-seen flag. Best scores never decrease.

---

## Installation

**Prerequisites:** Rust toolchain (edition 2024, Rust ≥ 1.85).

```sh
git clone <this-repo>
cd lq
cargo install --path .
```

Or run directly without installing:

```sh
cargo run -- --repo /path/to/exercises
```

---

## Quick Start

```sh
# Point lq at an exercise repository and launch the TUI
lq --repo /path/to/exercise-repo

# Or cd into the exercise repo first
cd /path/to/exercise-repo
lq

# Check progress without launching the TUI
lq status

# Reset all progress (asks for confirmation)
lq --reset
```

Once inside the TUI:

| What you want | Key |
|---|---|
| Switch between Theory / Task / Output / Solution | `←` `→` |
| Reveal a hint | `h` |
| Toggle the Overview | `Tab` |
| Navigate exercises in Overview | `↑` `↓` then `Enter` |
| Quit | `q` |

---

## Exercise Repository Layout

`lq` operates on a **separate exercise repository** — a plain directory tree
that you create and maintain. It discovers this repo (in priority order):

1. `--repo <path>` CLI argument
2. Current working directory

### Directory Structure

```
<repo>/
├── lq.toml                            ← created by lq on first run
├── 01-rust-basics/
│   ├── 01-hello-world/
│   │   ├── 01-theory.md               ← background reading (optional)
│   │   ├── 02-task.md                 ← task description + TOML frontmatter
│   │   ├── main.rs                    ← student edits this file
│   │   └── solution/
│   │       ├── main.rs                ← reference solution
│   │       └── solution.md            ← hints, keywords, explanation
│   └── 02-variables/
│       └── ...
├── 02-control-flow/
│   └── ...
└── 08-concepts/
    └── 01-rust-memory/
        ├── 02-task.md
        ├── main.md                    ← Markdown/conceptual exercise
        └── solution/
            └── solution.md
```

**Naming rules:**
- Module and exercise directories must be prefixed with a zero-padded two-digit
  number (`01-`, `02-`, …). Directories without this prefix are ignored.
- Titles use lowercase kebab-case: `01-hello-world`, `02-variables`.
- Each exercise directory contains **exactly one** student source file named
  `main.*`. The extension determines the language.
- The `solution/` subdirectory is read-only; `lq` never modifies it.

### Task Frontmatter (`02-task.md`)

Every exercise requires a `02-task.md` with a TOML frontmatter block. This is
the single source of truth for the Overview table.

```markdown
---
id          = "01_hello_world"
name        = "Hello, World!"
language    = "rust"
difficulty  = 2
description = "Implement a function that returns a greeting string."
topics      = ["functions", "string_literals", "return_values"]
---

# Hello, World!

Your task is to implement the `greeting()` function so that it returns the
string `"Hello, World!"` exactly.
```

| Field | Type | Required | Description |
|---|---|---|---|
| `id` | string | ✓ | Unique snake_case identifier; used as the key in `lq.toml` |
| `name` | string | ✓ | Display name shown in the exercise table |
| `language` | string | ✓ | `rust`, `go`, `python`, `riscv`, or `text` |
| `difficulty` | integer 1–5 | ✓ | Rendered as `*` stars in the table |
| `description` | string | ✓ | One-line summary |
| `topics` | array of strings | ✓ | Tag list shown in the Topics column |

Missing or malformed frontmatter causes the exercise to be skipped with a
clear error message — `lq` never silently uses defaults.

### Student Source Files

Source files are pre-populated with stubs and `// TODO` comments marking what
the student must implement. There is no special completion marker — correctness
is determined entirely by the verification runner on every save.

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

### Solution Files (`solution/solution.md`)

Each solution directory contains `solution.md`, a Markdown file with TOML
frontmatter. The frontmatter holds hints and keywords; the Markdown body is
the prose explanation shown on the Solution page.

```markdown
---
title    = "Variables and Mutability"
hints    = [
    "Remember that variables are immutable by default in Rust.",
    "Use the `mut` keyword to allow reassignment.",
    "The compiler error message tells you exactly which binding needs `mut`.",
]
keywords = ["mut", "let", "immutable", "shadow"]
---

Rust variables are **immutable by default**. This is a deliberate design
choice — you must explicitly opt into mutability by adding `mut` after `let`.

```rust
let x = 5;       // immutable — cannot reassign
let mut y = 5;   // mutable — can reassign
y = 10;          // ok
```
```

| Field | Description |
|---|---|
| `title` | Display name for the solution |
| `hints` | Ordered list of hints, revealed one at a time with `h` |
| `keywords` | Regex patterns used to score Markdown/conceptual exercises |
| body | Prose explanation rendered on the Solution page |

---

## Verification

`lq` watches the student source file with `notify` and re-runs verification on
every save. Verification output streams into the **Output page** (capped at 200
lines). No manual trigger is needed.

### Scoring

Each language produces a score in `0.0..=1.0`. The Solution page unlocks once
`score >= threshold`.

| Language | Threshold | Pass condition |
|---|---|---|
| Rust | 1.0 | All `#[test]` functions pass |
| Go | 1.0 | All `TestXxx` functions pass |
| Python | 1.0 | All unittest/pytest tests pass |
| RISC-V assembly | 0.8 | ≥ 80 % of `EXPECT_*` directives satisfied |
| Markdown | 0.75 | ≥ 75 % of keywords matched in the answer |

The best-ever score is persisted in `lq.toml` and never decreases across
sessions. Reaching the threshold sets `passed = true` permanently.

### Rust

Uses `#[test]` functions defined directly inside `main.rs`. No `Cargo.toml` is
needed per exercise.

```sh
rustc --edition 2024 --test main.rs -o .lq_test && ./.lq_test
```

Score = `tests_passed / tests_total`. A compile error gives score `0.0` and
shows the full compiler output.

### Go

Uses `TestXxx` functions defined in `main_test.go` (same package). Each Go
exercise directory contains a `go.mod` — no workspace setup is needed.

```sh
go test -v .
```

Score = `tests_passed / tests_total`. A build error gives score `0.0` and
shows the full compiler output. The `-v` flag is always passed so that both
passing and failing test names appear in the Output page.

### Python

Uses `unittest` or `pytest` test classes defined inside `main.py`.

```sh
python3 -m pytest main.py --tb=short -q
```

`pytest` is preferred for richer output; `python3 main.py` is used as a
fallback if `pytest` is unavailable. Score = `tests_passed / tests_total`.

### Assembly (RISC-V)

Expected behaviour is declared via structured directives in comments at the top
of `main.asm`:

```asm
; EXPECT_EXIT: 0
; EXPECT_STDOUT: Hello, World!
; EXPECT_STDOUT: Done
```

| Directive | Description |
|---|---|
| `EXPECT_EXIT: N` | Process must exit with code N (default: 0) |
| `EXPECT_STDOUT: S` | stdout must contain S as a substring (repeatable) |

```sh
riscv64-linux-gnu-as main.asm -o .lq_main.o && riscv64-linux-gnu-ld .lq_main.o -o .lq_main && ./.lq_main
```

Score = `satisfied_directives / total_directives`. Assemble or link errors give
score `0.0`.

### Markdown / Conceptual

Used for question-and-answer exercises where the student writes free text.
No execution; verification is purely regex-based.

Student file (`main.md`):

```markdown
# Question: What keyword makes a Rust variable mutable?

<!-- Write your answer below -->
You use the `mut` keyword: `let mut x = 5;`
```

`lq` loads the `keywords` array from `solution/solution.md`, compiles each
entry as a case-insensitive regex, and checks whether it matches anywhere in
`main.md`. Score = `matched / total_keywords`. Unmatched keywords are listed
in the Output page as gap indicators.

---

## TUI Reference

### Exercise View

The default view. Each exercise is presented as four pages cycled with `←` / `→`.

| Page | Content | Always accessible |
|---|---|---|
| 1 — Theory | Rendered `01-theory.md` | Yes |
| 2 — Task | Rendered `02-task.md` (frontmatter stripped) | Yes |
| 3 — Output | Live verification output + hints | Yes |
| 4 — Solution | Syntax-highlighted reference code + explanation | After pass or explicit unlock |

When a page's content is taller than the terminal, its header shows a scroll
percentage: `Output [42%]`. Scrolling is done with `↑` / `↓`.

### Overview

Press `Tab` to switch between the Exercise View and the Overview.

```
Progress: [=======================-----------]  14/42

 ID                   Name                  Language   Difficulty  Status      Topics
 ─────────────────────────────────────────────────────────────────────────────────────
 hello_world          Hello, World!         Rust       *           [*] Done    basic_syntax
 variables            Variables             Rust       **          [~] Partial mutability
▶ functions           Functions             Rust       **          [x] Failing functions
 ...

                        │  01-rust-basics/
                        │  |-- [*] Hello, World!
                        │  |-- [~] Variables
                        │  +-- [x] Functions
                        │
                        │  02-control-flow/
                        │  |-- [x] If / Else
                        │  +-- [x] Loops
```

**Progress bar** — `[===---]  completed/total` across all exercises.

**Exercise table** — fixed-width columns sourced from `02-task.md` frontmatter.
The cursor row is highlighted and kept in sync with the tree panel.

| Column | Source | Notes |
|---|---|---|
| ID | `id` frontmatter field | |
| Name | `name` frontmatter field | |
| Language | `language` frontmatter field | |
| Difficulty | `difficulty` frontmatter field | Rendered as `*` stars |
| Status | Derived from `lq.toml` at runtime | |
| Topics | `topics` frontmatter array | Joined with `, ` |

**Tree panel** — shows the module hierarchy with status symbols. Highlights the
same exercise as the table cursor. Toggle visibility with `t`. Hidden
automatically if the terminal is narrower than 80 columns.

**Status symbols:**

| Symbol | Meaning |
|---|---|
| `[*]` | Complete — passed and solution seen |
| `[~]` | Partial — passed but solution not yet seen, or unlocked manually |
| `[x]` | Failing — not yet passed |

### Keybindings

#### Exercise View

| Key | Action |
|---|---|
| `←` / `→` | Previous / next page |
| `↑` / `↓` | Scroll content up / down |
| `h` | Reveal next hint (switches to Output page first if on Theory or Task) |
| `j` / `k` | Previous / next exercise |
| `Tab` or `o` | Switch to Overview |
| `q` / `Esc` | Quit |
| `m` | Toggle status bar |

#### Overview

| Key | Action |
|---|---|
| `↑` / `↓` | Move cursor through exercise table |
| `Enter` | Jump to selected exercise and open Exercise View |
| `t` | Toggle tree panel |
| `Tab` or `o` | Switch to Exercise View |
| `q` / `Esc` | Quit |
| `m` | Toggle status bar |

### Hints & Solution Unlock

Hints are revealed one at a time from the Output page (or from Theory/Task —
`h` switches to Output automatically):

1. Each `h` press reveals the next hint: `[HINT 2/3] Use the mut keyword.`
2. After the last hint is shown, pressing `h` again displays a warning:
   `⚠ The solution will be unlocked. Are you really sure?`
3. Pressing `h` once more confirms and jumps directly to the Solution page.
4. Pressing any other key at the warning cancels without unlocking.

The hint index resets when navigating to a different exercise.

---

## Progress Tracking

All progress lives in **`lq.toml`** at the root of the exercise repository.
`lq` creates this file on first run and is the only file it ever writes inside
the repo. Exercise source and solution files are never modified.

```toml
current_exercise = "02-control-flow/03-loops"

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

- `best_score` only ever increases — a lower-scoring run never overwrites a
  higher value.
- `passed` is set to `true` the first time `score >= threshold` and is never
  reset to `false`.
- `solution_seen` is set to `true` on the first visit to the Solution page and
  is never reset.
- Advancing to the next exercise with `k` is blocked until `solution_seen =
  true` (either by passing or by unlocking via hints).

`lq.toml` can be committed to the exercise repo to preserve progress across
machines, or added to `.gitignore` for single-student use — the choice belongs
to the repo maintainer.

---

## CLI Reference

```
lq [OPTIONS] [SUBCOMMAND]

OPTIONS:
    --repo <PATH>    Path to exercise repository root
                     Default: current working directory
    --reset          Wipe all progress and start fresh (prompts for confirmation)
    -h, --help
    -V, --version

SUBCOMMANDS:
    status           Print current exercise and overall progress (non-TUI)
```

### `lq status`

Prints a summary to stdout without launching the TUI:

```
Current exercise: 02-control-flow/03-loops
14/42 exercises completed
```

### `lq --reset`

Wipes all per-exercise state in `lq.toml` and resets `current_exercise` to
the first exercise. Asks for explicit confirmation:

```
[!] This will delete all progress in lq.toml. This cannot be undone.
    Type "yes" to confirm, or anything else to cancel: _
```

Only the exact string `yes` (case-sensitive) confirms. After a successful
reset, re-run `lq` to start fresh.

---

## Creating Exercises

A minimal exercise requires three files:

```
<NN>-<module>/<NN>-<exercise>/
├── 02-task.md          ← required, with TOML frontmatter
├── main.rs             ← (or main.py / main.asm / main.md)
└── solution/
    ├── main.rs         ← reference solution
    └── solution.md     ← hints and explanation
```

`01-theory.md` is optional but recommended for non-trivial topics.

**Minimal `02-task.md`:**

```markdown
---
id          = "my_exercise"
name        = "My Exercise"
language    = "rust"
difficulty  = 1
description = "A short one-line description."
topics      = ["topic_a", "topic_b"]
---

Task description in Markdown.
```

**Minimal `solution/solution.md`:**

```markdown
---
title    = "My Exercise"
hints    = [
    "First hint shown to the student.",
    "Second hint shown after the first.",
]
keywords = []
---

Explanation of the solution approach shown after the student unlocks the solution.
```

---

## Dependencies

| Crate | Purpose |
|---|---|
| `ratatui` | TUI rendering framework |
| `crossterm` | Cross-platform terminal backend |
| `clap` | CLI argument parsing |
| `notify` | File-system watcher for live re-verification |
| `syntect` | Syntax highlighting on the Solution page |
| `pulldown-cmark` | Markdown rendering in Theory, Task, and Solution pages |
| `toml` + `serde` | `lq.toml` and frontmatter parsing |
| `regex` | Keyword matching for Markdown exercises |
| `anyhow` | Error propagation (application layer) |
| `thiserror` | Typed domain errors (library layer) |

---

## License

MIT
