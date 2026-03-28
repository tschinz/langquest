//! Exercise and module data structures, frontmatter parsing, and repo scanning.
//!
//! This module is responsible for discovering exercises in a repository,
//! parsing their metadata from `02-task.md` frontmatter, loading solution
//! data from `solution/solution.md`, and presenting them as structured
//! [`Exercise`] and [`Module`] types.

use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::ExerciseError;

// ---------------------------------------------------------------------------
// Language
// ---------------------------------------------------------------------------

/// Programming language (or content type) of an exercise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Rust (`main.rs`)
    Rust,
    /// RISC-V assembly (`main.asm`)
    Riscv,
    /// Python (`main.py`)
    Python,
    /// Go (`main.go`)
    Go,
    /// Markdown / plain-text questions (`main.md`)
    Text,
}

impl Language {
    /// Attempt to derive a [`Language`] from a file extension string.
    ///
    /// This is a *helper* — the canonical language comes from frontmatter,
    /// not from the file extension.
    #[allow(dead_code)]
    pub fn from_extension(ext: &str) -> Option<Language> {
        match ext {
            "rs" => Some(Language::Rust),
            "asm" | "s" | "S" => Some(Language::Riscv), // ambiguous; prefer frontmatter
            "py" => Some(Language::Python),
            "go" => Some(Language::Go),
            "md" => Some(Language::Text),
            _ => None,
        }
    }

    /// File extension used for student source files of this language.
    #[allow(dead_code)]
    pub fn source_extension(&self) -> &'static str {
        match self {
            Language::Rust => "rs",
            Language::Riscv => "asm",
            Language::Python => "py",
            Language::Go => "go",
            Language::Text => "md",
        }
    }

    /// Score threshold required to consider an exercise *passed*.
    pub fn threshold(&self) -> f64 {
        match self {
            Language::Rust => 1.0,
            Language::Python => 1.0,
            Language::Go => 1.0,
            Language::Riscv => 0.8,
            Language::Text => 0.75,
        }
    }

    /// Human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::Riscv => "RISC-V",
            Language::Python => "Python",
            Language::Go => "Go",
            Language::Text => "Text",
        }
    }

    /// Syntect language token used for syntax highlighting.
    ///
    /// Passed to [`highlight_code_block`](crate::ui::markdown::highlight_code_block)
    /// when rendering the solution source file. An empty string means plain
    /// text (no highlighting).
    pub fn syntax_token(&self) -> &'static str {
        match self {
            Language::Rust => "rust",
            Language::Python => "python",
            Language::Go => "go",
            Language::Riscv => "asm",
            Language::Text => "",
        }
    }

    /// Parse a language identifier string (case-insensitive) into a [`Language`].
    fn parse(s: &str) -> Option<Language> {
        match s.to_ascii_lowercase().as_str() {
            "rust" => Some(Language::Rust),
            "riscv" => Some(Language::Riscv),
            "python" => Some(Language::Python),
            "go" => Some(Language::Go),
            "text" => Some(Language::Text),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// ExerciseStatus
// ---------------------------------------------------------------------------

/// Current completion status of an exercise, derived from persisted state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExerciseStatus {
    /// Below threshold — keep working.
    Failing,
    /// Threshold reached or previously passed, but solution not yet viewed.
    Partial,
    /// Passed **and** solution viewed — fully complete.
    Complete,
}

impl ExerciseStatus {
    /// Bracket symbol shown in the overview table.
    pub fn symbol(&self) -> &'static str {
        match self {
            ExerciseStatus::Failing => "[x]",
            ExerciseStatus::Partial => "[~]",
            ExerciseStatus::Complete => "[*]",
        }
    }

    /// Short human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            ExerciseStatus::Failing => "Failing",
            ExerciseStatus::Partial => "Partial",
            ExerciseStatus::Complete => "Done",
        }
    }
}

// ---------------------------------------------------------------------------
// SolutionData
// ---------------------------------------------------------------------------

/// Metadata parsed from `solution/solution.md` (TOML frontmatter + Markdown body).
#[derive(Debug, Clone)]
pub struct SolutionData {
    /// Display title of the solution.
    #[allow(dead_code)]
    pub title: String,
    /// Ordered list of hint strings revealed one at a time.
    pub hints: Vec<String>,
    /// Keywords / regex patterns used for text-exercise verification.
    pub keywords: Vec<String>,
    /// Prose explanation shown on the solution page.
    pub explanation: String,
}

// ---------------------------------------------------------------------------
// Exercise
// ---------------------------------------------------------------------------

/// A single exercise within a module directory.
#[derive(Debug, Clone)]
pub struct Exercise {
    /// Unique identifier from frontmatter (snake_case).
    pub id: String,
    /// Human-readable display name from frontmatter.
    pub name: String,
    /// Target language from frontmatter.
    pub language: Language,
    /// Difficulty rating 1–5 from frontmatter.
    pub difficulty: u8,
    /// One-line description from frontmatter.
    #[allow(dead_code)]
    pub description: String,
    /// Topic tags from frontmatter.
    pub topics: Vec<String>,
    /// Parent module directory name (e.g. `"01-basics"`).
    #[allow(dead_code)]
    pub module_name: String,
    /// Relative path used as a key in `lq.toml` (e.g. `"01-basics/01-hello-world"`).
    pub relative_path: String,
    /// Absolute path to the exercise directory.
    pub dir: PathBuf,
    /// Path to `01-theory.md`, if present.
    pub theory_path: Option<PathBuf>,
    /// Path to `02-task.md`.
    pub task_path: PathBuf,
    /// Path to the student source file (`main.*`).
    pub source_path: PathBuf,
    /// Path to `solution/main.*`, if present.
    pub solution_source: Option<PathBuf>,
    /// Parsed contents of `solution/solution.md`, if present.
    pub solution_data: Option<SolutionData>,
}

// ---------------------------------------------------------------------------
// Module
// ---------------------------------------------------------------------------

/// A group of exercises under a numbered module directory.
#[derive(Debug, Clone)]
pub struct Module {
    /// Module directory name (e.g. `"01-basics"`).
    pub name: String,
    /// Exercises contained in this module, sorted by numeric prefix.
    pub exercises: Vec<Exercise>,
}

// ---------------------------------------------------------------------------
// Frontmatter parsing helpers
// ---------------------------------------------------------------------------

/// Raw deserialization target for `02-task.md` TOML frontmatter.
///
/// All fields are optional so we can produce precise missing-field errors
/// rather than relying on serde's generic messages.
#[derive(Deserialize)]
struct RawFrontmatter {
    id: Option<String>,
    name: Option<String>,
    language: Option<String>,
    difficulty: Option<u8>,
    description: Option<String>,
    topics: Option<Vec<String>>,
}

/// Split a markdown document into its TOML frontmatter and body.
///
/// Frontmatter is delimited by `---` on its own line at the very start of the
/// file. Returns `Some((toml_str, body))` on success.
pub fn parse_frontmatter(content: &str) -> Option<(&str, &str)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }

    // Skip the opening `---` line.
    let after_open = &trimmed[3..];
    let after_open = after_open.strip_prefix('\n').or_else(|| after_open.strip_prefix("\r\n"))?;

    // Find the closing `---`.
    let close_idx = find_closing_delimiter(after_open)?;
    let toml_str = &after_open[..close_idx];

    // Body starts after the closing `---` line.
    let rest = &after_open[close_idx + 3..];
    let body = rest
        .strip_prefix('\n')
        .or_else(|| rest.strip_prefix("\r\n"))
        .unwrap_or(rest);

    Some((toml_str, body))
}

/// Find the byte offset of a `---` line that closes the frontmatter block.
fn find_closing_delimiter(s: &str) -> Option<usize> {
    let mut search_from = 0;
    while search_from < s.len() {
        let remaining = &s[search_from..];
        let pos = remaining.find("---")?;
        let abs = search_from + pos;

        // Must be at the start of a line.
        let at_line_start = abs == 0 || s.as_bytes()[abs - 1] == b'\n';
        if at_line_start {
            return Some(abs);
        }
        search_from = abs + 3;
    }
    None
}

/// Validated frontmatter fields extracted from `02-task.md`.
struct ValidatedFrontmatter {
    id: String,
    name: String,
    language: Language,
    difficulty: u8,
    description: String,
    topics: Vec<String>,
}

/// Validate and convert [`RawFrontmatter`] into the fields needed by [`Exercise`].
fn validate_frontmatter(
    raw: RawFrontmatter,
    path: &Path,
) -> Result<ValidatedFrontmatter, ExerciseError> {
    let id = raw.id.ok_or_else(|| ExerciseError::MissingField {
        field: "id",
        path: path.to_path_buf(),
    })?;

    let name = raw.name.ok_or_else(|| ExerciseError::MissingField {
        field: "name",
        path: path.to_path_buf(),
    })?;

    let language_str = raw.language.ok_or_else(|| ExerciseError::MissingField {
        field: "language",
        path: path.to_path_buf(),
    })?;

    let language = Language::parse(&language_str).ok_or_else(|| ExerciseError::InvalidField {
        field: "language",
        path: path.to_path_buf(),
        reason: format!("unknown language `{language_str}`"),
    })?;

    let difficulty = raw.difficulty.ok_or_else(|| ExerciseError::MissingField {
        field: "difficulty",
        path: path.to_path_buf(),
    })?;

    if !(1..=5).contains(&difficulty) {
        return Err(ExerciseError::InvalidField {
            field: "difficulty",
            path: path.to_path_buf(),
            reason: format!("expected 1–5, got {difficulty}"),
        });
    }

    let description = raw.description.ok_or_else(|| ExerciseError::MissingField {
        field: "description",
        path: path.to_path_buf(),
    })?;

    let topics = raw.topics.ok_or_else(|| ExerciseError::MissingField {
        field: "topics",
        path: path.to_path_buf(),
    })?;

    Ok(ValidatedFrontmatter {
        id,
        name,
        language,
        difficulty,
        description,
        topics,
    })
}

// ---------------------------------------------------------------------------
// Exercise loading
// ---------------------------------------------------------------------------

/// Load a single exercise from its directory.
///
/// Reads `02-task.md` for frontmatter, locates the student source file,
/// and optionally loads theory, solution source, and solution metadata.
pub fn load_exercise(exercise_dir: &Path, module_name: &str) -> Result<Exercise, ExerciseError> {
    let task_path = exercise_dir.join("02-task.md");

    // -- Read and parse 02-task.md ------------------------------------------
    let task_content = fs::read_to_string(&task_path).map_err(|e| ExerciseError::FileRead {
        path: task_path.clone(),
        source: e,
    })?;

    let (toml_str, _body) =
        parse_frontmatter(&task_content).ok_or_else(|| ExerciseError::MissingFrontmatter {
            path: task_path.clone(),
        })?;

    let raw: RawFrontmatter =
        toml::from_str(toml_str).map_err(|e| ExerciseError::FrontmatterParse {
            path: task_path.clone(),
            source: e,
        })?;

    let fm = validate_frontmatter(raw, &task_path)?;

    // -- Locate student source file (main.*) --------------------------------
    let source_path = find_student_source(exercise_dir)?;

    // -- Optional: 01-theory.md ---------------------------------------------
    let theory_candidate = exercise_dir.join("01-theory.md");
    let theory_path = if theory_candidate.is_file() {
        Some(theory_candidate)
    } else {
        None
    };

    // -- Optional: solution/solution.md ------------------------------------
    let solution_data = load_solution_data(exercise_dir)?;

    // -- Optional: solution/main.* ------------------------------------------
    let solution_source = find_solution_source(exercise_dir);

    // -- Build relative path ------------------------------------------------
    let exercise_dir_name = exercise_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let relative_path = format!("{module_name}/{exercise_dir_name}");

    Ok(Exercise {
        id: fm.id,
        name: fm.name,
        language: fm.language,
        difficulty: fm.difficulty,
        description: fm.description,
        topics: fm.topics,
        module_name: module_name.to_owned(),
        relative_path,
        dir: exercise_dir.to_path_buf(),
        theory_path,
        task_path,
        source_path,
        solution_source,
        solution_data,
    })
}

/// Find the student source file (`main.*` not inside `solution/`).
fn find_student_source(exercise_dir: &Path) -> Result<PathBuf, ExerciseError> {
    let entries = fs::read_dir(exercise_dir).map_err(|e| ExerciseError::FileRead {
        path: exercise_dir.to_path_buf(),
        source: e,
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| ExerciseError::FileRead {
            path: exercise_dir.to_path_buf(),
            source: e,
        })?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let file_name = match path.file_stem().and_then(|s| s.to_str()) {
            Some(n) => n,
            None => continue,
        };

        if file_name == "main" {
            // Make sure it isn't inside `solution/` — should not be since we
            // are iterating the exercise dir itself, but guard anyway.
            if let Some(parent) = path.parent()
                && parent == exercise_dir
            {
                return Ok(path);
            }
        }
    }

    Err(ExerciseError::NoSourceFile {
        path: exercise_dir.to_path_buf(),
    })
}

/// Raw deserialization target for `solution/solution.md` TOML frontmatter.
#[derive(Deserialize)]
struct RawSolutionFrontmatter {
    #[serde(default)]
    title: String,
    #[serde(default)]
    hints: Vec<String>,
    #[serde(default)]
    keywords: Vec<String>,
}

/// Load and parse `solution/solution.md` if it exists.
///
/// The file uses the same TOML frontmatter convention as `02-task.md`.
/// The `title`, `hints`, and `keywords` fields live in the frontmatter block;
/// the explanation is the Markdown body that follows the closing `---`.
fn load_solution_data(exercise_dir: &Path) -> Result<Option<SolutionData>, ExerciseError> {
    let solution_md = exercise_dir.join("solution").join("solution.md");

    if !solution_md.is_file() {
        return Ok(None);
    }

    let content = fs::read_to_string(&solution_md).map_err(|e| ExerciseError::FileRead {
        path: solution_md.clone(),
        source: e,
    })?;

    let (toml_str, body) =
        parse_frontmatter(&content).ok_or_else(|| ExerciseError::MissingFrontmatter {
            path: solution_md.clone(),
        })?;

    let raw: RawSolutionFrontmatter =
        toml::from_str(toml_str).map_err(|e| ExerciseError::SolutionParse {
            path: solution_md,
            source: e,
        })?;

    Ok(Some(SolutionData {
        title: raw.title,
        hints: raw.hints,
        keywords: raw.keywords,
        explanation: body.trim_end().to_owned(),
    }))
}

/// Find `solution/main.*` if it exists.
fn find_solution_source(exercise_dir: &Path) -> Option<PathBuf> {
    let solution_dir = exercise_dir.join("solution");

    if !solution_dir.is_dir() {
        return None;
    }

    let entries = fs::read_dir(&solution_dir).ok()?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
            && stem == "main"
        {
            return Some(path);
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Repo discovery
// ---------------------------------------------------------------------------

/// Discover all modules and exercises under `repo_root`.
///
/// Scans for directories matching the `<NN>-<title>` naming convention,
/// treating top-level matches as modules and their children as exercises.
///
/// Returns a tuple of `(modules, errors)`. Malformed exercises are skipped
/// and their errors collected so the TUI can present partial results.
pub fn discover_exercises(repo_root: &Path) -> (Vec<Module>, Vec<(PathBuf, ExerciseError)>) {
    let dir_pattern = match Regex::new(r"^\d{2}-.+") {
        Ok(re) => re,
        Err(_) => return (Vec::new(), Vec::new()),
    };

    let mut modules = Vec::new();
    let mut errors: Vec<(PathBuf, ExerciseError)> = Vec::new();

    // -- Collect module directories -----------------------------------------
    let mut module_dirs = match sorted_numbered_dirs(repo_root, &dir_pattern) {
        Ok(dirs) => dirs,
        Err(e) => {
            errors.push((repo_root.to_path_buf(), e));
            return (modules, errors);
        }
    };
    module_dirs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

    for module_dir in &module_dirs {
        let module_name = match module_dir.file_name().and_then(|n| n.to_str()) {
            Some(n) => n.to_owned(),
            None => continue,
        };

        // -- Collect exercise directories inside this module ----------------
        let mut exercise_dirs = match sorted_numbered_dirs(module_dir, &dir_pattern) {
            Ok(dirs) => dirs,
            Err(e) => {
                errors.push((module_dir.clone(), e));
                continue;
            }
        };
        exercise_dirs.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        let mut exercises = Vec::new();

        for ex_dir in &exercise_dirs {
            match load_exercise(ex_dir, &module_name) {
                Ok(exercise) => exercises.push(exercise),
                Err(e) => errors.push((ex_dir.clone(), e)),
            }
        }

        // Only include the module if it has at least one valid exercise.
        if !exercises.is_empty() {
            modules.push(Module {
                name: module_name,
                exercises,
            });
        }
    }

    (modules, errors)
}

/// List subdirectories of `parent` whose names match `pattern`, returned
/// in a `Vec` for the caller to sort.
fn sorted_numbered_dirs(parent: &Path, pattern: &Regex) -> Result<Vec<PathBuf>, ExerciseError> {
    let entries = fs::read_dir(parent).map_err(|e| ExerciseError::FileRead {
        path: parent.to_path_buf(),
        source: e,
    })?;

    let mut dirs = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| ExerciseError::FileRead {
            path: parent.to_path_buf(),
            source: e,
        })?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let dir_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) => n,
            None => continue,
        };

        if pattern.is_match(dir_name) {
            dirs.push(path);
        }
    }

    Ok(dirs)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_frontmatter_valid() {
        let input = "---\nid = \"hello\"\n---\nBody text here.";
        let result = parse_frontmatter(input);
        assert!(result.is_some());
        let (toml_str, body) = result.unwrap();
        assert_eq!(toml_str, "id = \"hello\"\n");
        assert_eq!(body, "Body text here.");
    }

    #[test]
    fn test_parse_frontmatter_missing() {
        let input = "No frontmatter here.";
        assert!(parse_frontmatter(input).is_none());
    }

    #[test]
    fn test_language_parse_roundtrip() {
        for (s, expected) in [
            ("rust", Language::Rust),
            ("RISCV", Language::Riscv),
            ("Python", Language::Python),
            ("text", Language::Text),
        ] {
            assert_eq!(Language::parse(s), Some(expected));
        }
        assert_eq!(Language::parse("unknown"), None);
    }

    #[test]
    fn test_language_threshold() {
        assert!((Language::Rust.threshold() - 1.0).abs() < f64::EPSILON);
        assert!((Language::Riscv.threshold() - 0.8).abs() < f64::EPSILON);
        assert!((Language::Text.threshold() - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn test_exercise_status_symbols() {
        assert_eq!(ExerciseStatus::Failing.symbol(), "[x]");
        assert_eq!(ExerciseStatus::Partial.symbol(), "[~]");
        assert_eq!(ExerciseStatus::Complete.symbol(), "[*]");
    }

    #[test]
    fn test_exercise_status_labels() {
        assert_eq!(ExerciseStatus::Failing.label(), "Failing");
        assert_eq!(ExerciseStatus::Partial.label(), "Partial");
        assert_eq!(ExerciseStatus::Complete.label(), "Done");
    }

    #[test]
    fn test_load_exercise_full() {
        let tmp = std::env::temp_dir().join("lq_test_load_exercise_full");
        let _ = fs::remove_dir_all(&tmp);

        let ex_dir = tmp.join("01-basics").join("01-hello");
        let sol_dir = ex_dir.join("solution");
        fs::create_dir_all(&sol_dir).unwrap();

        // 02-task.md
        fs::write(
            ex_dir.join("02-task.md"),
            r#"---
id          = "hello"
name        = "Hello World"
language    = "rust"
difficulty  = 1
description = "Print hello."
topics      = ["basics"]
---
Do the thing.
"#,
        )
        .unwrap();

        // main.rs
        fs::write(ex_dir.join("main.rs"), "fn main() {}").unwrap();

        // 01-theory.md
        fs::write(ex_dir.join("01-theory.md"), "# Theory").unwrap();

        // solution/main.rs
        fs::write(sol_dir.join("main.rs"), "fn main() { println!(\"hi\"); }").unwrap();

        // solution/solution.md
        fs::write(
            sol_dir.join("solution.md"),
            "---\ntitle    = \"Hello World\"\nhints    = [\n    \"Try println!\",\n]\nkeywords = []\n---\n\nJust print it.\n",
        )
        .unwrap();

        let exercise = load_exercise(&ex_dir, "01-basics").unwrap();
        assert_eq!(exercise.id, "hello");
        assert_eq!(exercise.name, "Hello World");
        assert_eq!(exercise.language, Language::Rust);
        assert_eq!(exercise.difficulty, 1);
        assert_eq!(exercise.relative_path, "01-basics/01-hello");
        assert!(exercise.theory_path.is_some());
        assert!(exercise.solution_source.is_some());
        assert!(exercise.solution_data.is_some());
        assert_eq!(exercise.solution_data.as_ref().unwrap().title, "Hello World");

        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_discover_exercises_collects_errors() {
        let tmp = std::env::temp_dir().join("lq_test_discover_errors");
        let _ = fs::remove_dir_all(&tmp);

        let mod_dir = tmp.join("01-mod");
        let good_dir = mod_dir.join("01-good");
        let bad_dir = mod_dir.join("02-bad");
        fs::create_dir_all(&good_dir).unwrap();
        fs::create_dir_all(&bad_dir).unwrap();

        // Good exercise
        fs::write(
            good_dir.join("02-task.md"),
            "---\nid=\"g\"\nname=\"G\"\nlanguage=\"rust\"\ndifficulty=1\ndescription=\"d\"\ntopics=[]\n---\n",
        )
        .unwrap();
        fs::write(good_dir.join("main.rs"), "").unwrap();

        // Bad exercise — no 02-task.md
        fs::write(bad_dir.join("main.rs"), "").unwrap();

        let (modules, errors) = discover_exercises(&tmp);
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].exercises.len(), 1);
        assert_eq!(modules[0].exercises[0].id, "g");
        assert!(!errors.is_empty());

        let _ = fs::remove_dir_all(&tmp);
    }
}