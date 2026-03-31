//! Configuration module for `lq.toml` config file reading/writing and repo path resolution.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::ConfigError;

/// Display settings for the TUI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
  /// Whether to show line numbers in code blocks.
  #[serde(default = "default_true")]
  pub line_numbers: bool,
  /// Whether to enable syntax highlighting in code blocks.
  #[serde(default = "default_true")]
  pub syntax_highlighting: bool,
}

/// Helper for serde default = true.
fn default_true() -> bool {
  true
}

impl Default for DisplayConfig {
  fn default() -> Self {
    Self {
      line_numbers: true,
      syntax_highlighting: true,
    }
  }
}

/// State tracking for a single exercise.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseState {
  /// The highest score achieved on this exercise.
  pub best_score: f64,
  /// Whether the exercise has been passed (sticky - never resets to false).
  pub passed: bool,
  /// Whether the reference solution has been viewed (sticky - never resets to false).
  pub solution_seen: bool,
}

impl Default for ExerciseState {
  fn default() -> Self {
    Self {
      best_score: 0.0,
      passed: false,
      solution_seen: false,
    }
  }
}

/// Configuration for the Ripes RISC-V simulator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipesConfig {
  /// Explicit path to the Ripes binary.
  ///
  /// When non-empty this takes priority over every other discovery mechanism
  /// (`$RIPES_PATH`, bundled walk-up, `$PATH`).  Populated automatically the
  /// first time a RISC-V exercise is verified and the binary is found via
  /// auto-discovery, so the resolved path is always visible and editable in
  /// `lq.toml`.
  #[serde(default)]
  pub bin: String,
  /// Command template used to invoke Ripes in CLI mode.
  ///
  /// `<file>` is substituted at runtime with the absolute path to the
  /// student's source file.  The first token may be a bare name (`ripes`),
  /// a relative path, or an absolute path; bare names are resolved via the
  /// bundled binary discovery logic before falling back to `$PATH`.
  pub cmd: String,
}

impl Default for RipesConfig {
  fn default() -> Self {
    Self {
      bin: String::new(),
      cmd: "ripes --mode cli -t asm --proc RV32_SS --json --regs --runinfo --src <file>".to_string(),
    }
  }
}

/// Configuration for the Rust toolchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustConfig {
  /// Command template used to compile the student source as a test binary.
  ///
  /// `<file>` is substituted with the absolute path to the student's source
  /// file; `<out>` is substituted with the path to the compiled test binary.
  pub cmd: String,
}

impl Default for RustConfig {
  fn default() -> Self {
    Self {
      cmd: "rustc --edition 2024 --test <file> -o <out>".to_string(),
    }
  }
}

/// Configuration for the Python toolchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonConfig {
  /// Command template used to run the student's Python tests.
  ///
  /// `<file>` is substituted with the absolute path to the student's source
  /// file.  The first token of the command is used as the Python interpreter
  /// for the `unittest` fallback when `pytest` is unavailable.
  pub cmd: String,
}

impl Default for PythonConfig {
  fn default() -> Self {
    Self {
      cmd: "python3 -m pytest <file> --tb=short -q".to_string(),
    }
  }
}

/// Configuration for the Go toolchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoConfig {
  /// Command template used to run the student's Go tests.
  ///
  /// The command is executed in the exercise directory.  The default
  /// `go test -v .` is sufficient for most setups.  No `<file>`
  /// substitution is performed since Go tests are addressed by package (`.`).
  pub cmd: String,
}

impl Default for GoConfig {
  fn default() -> Self {
    Self {
      cmd: "go test -v .".to_string(),
    }
  }
}

/// Top-level project configuration, persisted as `lq.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
  /// Relative path of the current exercise (e.g. `"01-basics/01-hello-world"`).
  pub current_exercise: Option<String>,
  /// Per-exercise state, keyed by relative path. Uses `BTreeMap` for sorted,
  /// deterministic TOML output.
  pub exercises: BTreeMap<String, ExerciseState>,
  /// Display settings for the TUI (line numbers, syntax highlighting).
  #[serde(default)]
  pub display: DisplayConfig,
  /// Rust toolchain settings.  Written to `lq.toml` on first save so the
  /// user can customise the command without recompiling.
  #[serde(default)]
  pub rust: RustConfig,
  /// Python toolchain settings.  Written to `lq.toml` on first save so the
  /// user can customise the command without recompiling.
  #[serde(default)]
  pub python: PythonConfig,
  /// Go toolchain settings.  Written to `lq.toml` on first save so the
  /// user can customise the command without recompiling.
  #[serde(default)]
  pub go: GoConfig,
  /// Ripes simulator settings.  Written to `lq.toml` on first save so the
  /// user can customise the command without recompiling.
  #[serde(default)]
  pub ripes: RipesConfig,
}

impl ProjectConfig {
  /// Load a `ProjectConfig` from the TOML file at `path`.
  ///
  /// Returns `Ok(Self::default())` if the file does not exist.
  /// Maps I/O errors to [`ConfigError::Read`] and parse errors to
  /// [`ConfigError::Parse`].
  pub fn load(path: &Path) -> Result<Self, ConfigError> {
    let contents = match fs::read_to_string(path) {
      Ok(s) => s,
      Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
        return Ok(Self::default());
      }
      Err(e) => {
        return Err(ConfigError::Read {
          path: path.to_path_buf(),
          source: e,
        });
      }
    };

    toml::from_str(&contents).map_err(|e| ConfigError::Parse {
      path: path.to_path_buf(),
      source: e,
    })
  }

  /// Serialize this config to TOML and write it to `path`.
  ///
  /// Maps serialization errors to [`ConfigError::Serialize`] and I/O errors
  /// to [`ConfigError::Write`].
  pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
    let contents = toml::to_string(self).map_err(|e| ConfigError::Serialize { source: e })?;

    fs::write(path, contents).map_err(|e| ConfigError::Write {
      path: path.to_path_buf(),
      source: e,
    })
  }

  /// Return the [`ExerciseState`] for the given exercise path, or a default
  /// state if no entry exists yet.
  pub fn get_state(&self, exercise_path: &str) -> ExerciseState {
    self.exercises.get(exercise_path).cloned().unwrap_or_default()
  }

  /// Update the score for an exercise.
  ///
  /// - `best_score` is only updated if `score` is strictly higher (monotonic increase).
  /// - `passed` is set to `true` when `score >= threshold` and is sticky (never reset).
  pub fn update_score(&mut self, exercise_path: &str, score: f64, threshold: f64) {
    let state = self.exercises.entry(exercise_path.to_owned()).or_default();

    if score > state.best_score {
      state.best_score = score;
    }

    if score >= threshold {
      state.passed = true;
    }
  }

  /// Mark the reference solution as seen for the given exercise.
  ///
  /// This is sticky - once set to `true` it is never reset.
  pub fn mark_solution_seen(&mut self, exercise_path: &str) {
    let state = self.exercises.entry(exercise_path.to_owned()).or_default();

    state.solution_seen = true;
  }

  /// Reset all exercise state and optionally set the current exercise to
  /// `first_exercise`.
  pub fn reset(&mut self, first_exercise: Option<&str>) {
    self.exercises.clear();
    self.current_exercise = first_exercise.map(String::from);
  }
}

/// Resolve the repository root path.
///
/// If `cli_repo` is `Some`, the provided path is canonicalized and returned.
/// Otherwise the current working directory is returned.
pub fn resolve_repo_path(cli_repo: Option<&Path>) -> PathBuf {
  match cli_repo {
    Some(p) => p.canonicalize().unwrap_or_else(|_| p.to_path_buf()),
    None => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
  }
}

/// Return the path to the `lq.toml` config file within the given repo root.
pub fn config_path(repo_root: &Path) -> PathBuf {
  repo_root.join("lq.toml")
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::io::Write;

  #[test]
  fn default_exercise_state() {
    let state = ExerciseState::default();
    assert_eq!(state.best_score, 0.0);
    assert!(!state.passed);
    assert!(!state.solution_seen);
  }

  #[test]
  fn default_project_config() {
    let cfg = ProjectConfig::default();
    assert!(cfg.current_exercise.is_none());
    assert!(cfg.exercises.is_empty());
  }

  #[test]
  fn load_missing_file_returns_default() {
    let path = Path::new("/tmp/lq_test_nonexistent_config.toml");
    let cfg = ProjectConfig::load(path).expect("should return default for missing file");
    assert!(cfg.current_exercise.is_none());
    assert!(cfg.exercises.is_empty());
  }

  #[test]
  fn save_and_load_roundtrip() {
    let dir = std::env::temp_dir().join("lq_test_roundtrip");
    let _ = fs::create_dir_all(&dir);
    let path = dir.join("lq.toml");

    let mut cfg = ProjectConfig {
      current_exercise: Some("01-basics/01-hello".to_owned()),
      ..Default::default()
    };
    cfg.update_score("01-basics/01-hello", 0.8, 0.7);
    cfg.mark_solution_seen("01-basics/01-hello");

    cfg.save(&path).expect("save should succeed");
    let loaded = ProjectConfig::load(&path).expect("load should succeed");

    assert_eq!(loaded.current_exercise.as_deref(), Some("01-basics/01-hello"));
    let state = loaded.get_state("01-basics/01-hello");
    assert_eq!(state.best_score, 0.8);
    assert!(state.passed);
    assert!(state.solution_seen);

    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn get_state_returns_default_for_unknown_exercise() {
    let cfg = ProjectConfig::default();
    let state = cfg.get_state("nonexistent/exercise");
    assert_eq!(state.best_score, 0.0);
    assert!(!state.passed);
    assert!(!state.solution_seen);
  }

  #[test]
  fn update_score_monotonic_increase() {
    let mut cfg = ProjectConfig::default();
    cfg.update_score("ex", 0.5, 0.7);
    assert_eq!(cfg.get_state("ex").best_score, 0.5);
    assert!(!cfg.get_state("ex").passed);

    // Lower score should not reduce best_score
    cfg.update_score("ex", 0.3, 0.7);
    assert_eq!(cfg.get_state("ex").best_score, 0.5);

    // Higher score updates, and crossing threshold sets passed
    cfg.update_score("ex", 0.9, 0.7);
    assert_eq!(cfg.get_state("ex").best_score, 0.9);
    assert!(cfg.get_state("ex").passed);
  }

  #[test]
  fn passed_is_sticky() {
    let mut cfg = ProjectConfig::default();
    cfg.update_score("ex", 1.0, 0.7);
    assert!(cfg.get_state("ex").passed);

    // Score below threshold should NOT reset passed
    cfg.update_score("ex", 0.1, 0.7);
    assert!(cfg.get_state("ex").passed);
  }

  #[test]
  fn mark_solution_seen_is_sticky() {
    let mut cfg = ProjectConfig::default();
    cfg.mark_solution_seen("ex");
    assert!(cfg.get_state("ex").solution_seen);
  }

  #[test]
  fn reset_clears_state() {
    let mut cfg = ProjectConfig {
      current_exercise: Some("old".to_owned()),
      ..Default::default()
    };
    cfg.update_score("ex1", 1.0, 0.5);
    cfg.update_score("ex2", 0.8, 0.5);

    cfg.reset(Some("first"));
    assert_eq!(cfg.current_exercise.as_deref(), Some("first"));
    assert!(cfg.exercises.is_empty());
  }

  #[test]
  fn reset_with_none() {
    let mut cfg = ProjectConfig::default();
    cfg.update_score("ex", 1.0, 0.5);
    cfg.reset(None);
    assert!(cfg.current_exercise.is_none());
    assert!(cfg.exercises.is_empty());
  }

  #[test]
  fn config_path_joins_correctly() {
    let root = Path::new("/some/repo");
    assert_eq!(config_path(root), PathBuf::from("/some/repo/lq.toml"));
  }

  #[test]
  fn resolve_repo_path_with_none_returns_cwd() {
    let result = resolve_repo_path(None);
    // Should return something (cwd or fallback), not panic
    assert!(!result.as_os_str().is_empty());
  }

  #[test]
  fn resolve_repo_path_with_some() {
    let dir = std::env::temp_dir();
    let result = resolve_repo_path(Some(&dir));
    // Canonicalized temp dir should exist
    assert!(result.exists());
  }

  #[test]
  fn load_invalid_toml_returns_parse_error() {
    let dir = std::env::temp_dir().join("lq_test_bad_toml");
    let _ = fs::create_dir_all(&dir);
    let path = dir.join("lq.toml");

    let mut f = fs::File::create(&path).expect("create file");
    f.write_all(b"this is [[[not valid toml").expect("write");
    drop(f);

    let result = ProjectConfig::load(&path);
    assert!(result.is_err());

    let _ = fs::remove_dir_all(&dir);
  }
}
