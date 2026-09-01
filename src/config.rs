//! Configuration module for `lq.toml` config file reading/writing and repo path resolution.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::ConfigError;
use crate::identity::GithubIdentity;

/// Embedded key used to seal the on-disk progress file.
const PROGRESS_STR: &str = env!("PROGRESS_KEY");
const PROGRESS_KEY: [u8; 32] = unsafe { *PROGRESS_STR.as_ptr().cast::<[u8; 32]>() }; // ok due to build.rs checking key size | safe "tryinto" error as not yet stable on const traits

/// Filename of the encrypted progress file, stored alongside `lq.toml`.
pub const PROGRESS_FILE: &str = ".lq.progress";

/// State tracking for a single exercise.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseState {
  /// The highest score achieved on this exercise.
  pub best_score: f64,
  /// Whether the exercise has been passed (sticky - never resets to false).
  pub passed: bool,
  /// Whether the reference solution has been viewed (sticky - never resets to false).
  pub solution_seen: bool,
  /// Cumulative number of hint reveals across all sessions.
  /// Each time the user presses 'h' this counter goes up by 1,
  /// even if the same hint level is reached again in a later session.
  #[serde(default)]
  pub hints_shown: usize,
  /// Furthest hint level reached, stored as "hint_level/total" (e.g. "3/5").
  /// The first component is the highest hint index the user has ever revealed;
  /// the second is the total number of hints for this exercise.
  #[serde(default, skip_serializing_if = "String::is_empty")]
  pub hints_max: String,
  /// Amount of times the student save the file and a regrading occured.
  #[serde(default)]
  pub times_saved: u32,
  /// Total number of unit tests / checks in this exercise, from the most recent
  /// verification (0 until first verified).
  #[serde(default)]
  pub tests_total: usize,
  /// Number of tests / checks that passed at the [`best_score`](Self::best_score).
  /// Enables partial-credit grading ("almost finished" exercises).
  #[serde(default)]
  pub best_tests_passed: usize,
}

impl Default for ExerciseState {
  fn default() -> Self {
    Self {
      best_score: 0.0,
      passed: false,
      solution_seen: false,
      hints_shown: 0,
      hints_max: String::new(),
      times_saved: 0,
      tests_total: 0,
      best_tests_passed: 0,
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
      cmd: "ripes --mode cli -t asm --proc RV32_SS --json --isaexts M --cycles --regs --runinfo --timeout 5000 --src <file>".to_string(),
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

  /// Command used to build the cargo project
  pub cmd_cargo: String,
}

impl Default for RustConfig {
  fn default() -> Self {
    Self {
      cmd: "rustc --edition 2024 --test <file> -o <out>".to_string(),
      cmd_cargo: "cargo test --no-fail-fast --no-run --message-format=json".to_string(),
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

/// Configuration for the C++ toolchain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CppConfig {
  /// Command template used to compile the student's C++ source files.
  ///
  /// `<files>` is substituted with the list of `.cpp` files in the exercise
  /// directory; `<out>` is substituted with the path to the compiled test
  /// binary.  Catch2 flags are resolved automatically via `pkg-config` at
  /// runtime and appended to the command.
  pub cmd: String,
}

impl Default for CppConfig {
  fn default() -> Self {
    Self {
      cmd: "g++ -std=c++20 <files> -o <out>".to_string(),
    }
  }
}

/// Configuration for PlantUML rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlantumlConfig {
  /// Explicit path to a pre-existing PlantUML jar (or launcher script).
  ///
  /// When non-empty this is used as the `<plantuml>` substitution, letting a
  /// course require a specific installed PlantUML. When empty, the
  /// `PLANTUML_JAR` environment variable is used. Mirrors
  /// [`RipesConfig::bin`].
  #[serde(default)]
  pub bin: String,
  /// Command template used to render a `.puml` file to PNG on save.
  ///
  /// `<plantuml>` is substituted with [`bin`](Self::bin) when set, otherwise the
  /// `PLANTUML_JAR` environment variable; `<file>` with the student's diagram
  /// source. The `java` launcher must be on `PATH` (Oracle Java JDK 21).
  pub cmd: String,
}

impl Default for PlantumlConfig {
  fn default() -> Self {
    Self {
      bin: String::new(),
      cmd: "java -jar <plantuml> -tpng <file>".to_string(),
    }
  }
}

/// Configuration for the external editor / IDE used by the `e` shortcut and to
/// preview rendered PlantUML diagrams.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeConfig {
  /// Path to the IDE launcher (e.g. `zed` or `code`).
  ///
  /// Auto-populated on first launch with a platform-appropriate default when a
  /// known IDE is discovered on the system; edit to point at your preferred
  /// editor. When empty (nothing found), `lq` falls back to the OS default
  /// handler. Mirrors [`RipesConfig::bin`].
  #[serde(default)]
  pub bin: String,
  /// Command template used to open a file. `<ide>` is substituted with
  /// [`bin`](Self::bin) and `<file>` with the file to open.
  pub cmd: String,
}

impl Default for IdeConfig {
  fn default() -> Self {
    Self {
      bin: String::new(),
      cmd: "<ide> <file>".to_string(),
    }
  }
}

/// Plaintext, hand-editable toolchain commands, persisted as `lq.toml`.
///
/// These are safe for students to see and tweak; they contain no progress.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct CommandsFile {
  #[serde(default)]
  rust: RustConfig,
  #[serde(default)]
  python: PythonConfig,
  #[serde(default)]
  go: GoConfig,
  #[serde(default)]
  cpp: CppConfig,
  #[serde(default)]
  plantuml: PlantumlConfig,
  #[serde(default)]
  ide: IdeConfig,
  #[serde(default)]
  ripes: RipesConfig,
}

/// Cheat-sensitive progress, serialized then AEAD-encrypted into
/// [`PROGRESS_FILE`]. Never written in plaintext.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ProgressFile {
  /// GitHub identity this progress is bound to (set on first online launch).
  #[serde(default)]
  owner: Option<GithubIdentity>,
  current_exercise: Option<String>,
  #[serde(default)]
  exercises: BTreeMap<String, ExerciseState>,
}

/// Top-level project configuration.
///
/// In memory this aggregates both the plaintext toolchain commands (persisted
/// to `lq.toml`) and the encrypted progress (persisted to [`PROGRESS_FILE`]).
/// [`load`](ProjectConfig::load) / [`save`](ProjectConfig::save) transparently
/// split across the two files.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConfig {
  /// GitHub identity the progress is bound to. Populated by
  /// [`crate::identity::authorize`] on first successful verification.
  #[serde(default)]
  pub owner: Option<GithubIdentity>,
  /// Relative path of the current exercise (e.g. `"01-basics/01-hello-world"`).
  pub current_exercise: Option<String>,
  /// Per-exercise state, keyed by relative path. Uses `BTreeMap` for sorted,
  /// deterministic output.
  pub exercises: BTreeMap<String, ExerciseState>,
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
  /// C++ toolchain settings.  Written to `lq.toml` on first save so the
  /// user can customise the command without recompiling.
  #[serde(default)]
  pub cpp: CppConfig,
  /// PlantUML rendering settings.  Written to `lq.toml` on first save so the
  /// user can customise the command without recompiling.
  #[serde(default)]
  pub plantuml: PlantumlConfig,
  /// Editor/IDE settings for the `e` shortcut and PlantUML preview.
  #[serde(default)]
  pub ide: IdeConfig,
  /// Ripes simulator settings.  Written to `lq.toml` on first save so the
  /// user can customise the command without recompiling.
  #[serde(default)]
  pub ripes: RipesConfig,
  /// Grade mode: the encrypted progress file is read-only and is therefore skipped by serde.
  #[serde(skip)]
  pub grade_mode: bool,
}

impl ProjectConfig {
  /// Load a `ProjectConfig`, reading plaintext commands from the `lq.toml`
  /// file at `path` and the encrypted progress from the sibling
  /// [`PROGRESS_FILE`].
  ///
  /// Missing files yield defaults. TOML parse errors map to
  /// [`ConfigError::Parse`]; a tampered/corrupt progress file maps to
  /// [`ConfigError::Decrypt`]. Identity binding is enforced separately by
  /// [`crate::identity::authorize`], not here.
  pub fn load(path: &Path) -> Result<Self, ConfigError> {
    let commands = Self::load_commands(path)?;
    let progress = Self::load_progress(&progress_path(path))?;

    Ok(ProjectConfig {
      owner: progress.owner,
      current_exercise: progress.current_exercise,
      exercises: progress.exercises,
      rust: commands.rust,
      python: commands.python,
      go: commands.go,
      cpp: commands.cpp,
      plantuml: commands.plantuml,
      ide: commands.ide,
      ripes: commands.ripes,
      grade_mode: false,
    })
  }

  /// Like [`load`](Self::load) but tolerant of a corrupt/tampered progress
  /// file: on a decrypt or parse failure the progress falls back to default
  /// instead of erroring.
  ///
  /// Used by `--reset`, which wipes progress anyway, so a student whose
  /// progress file got corrupted is never locked out of recovering.
  pub fn load_lenient(path: &Path) -> Result<Self, ConfigError> {
    let commands = Self::load_commands(path)?;
    let progress = Self::load_progress(&progress_path(path)).unwrap_or_default();

    Ok(ProjectConfig {
      owner: progress.owner,
      current_exercise: progress.current_exercise,
      exercises: progress.exercises,
      rust: commands.rust,
      python: commands.python,
      go: commands.go,
      cpp: commands.cpp,
      plantuml: commands.plantuml,
      ide: commands.ide,
      ripes: commands.ripes,
      grade_mode: false,
    })
  }

  /// Read and parse the plaintext `lq.toml` commands file.
  fn load_commands(path: &Path) -> Result<CommandsFile, ConfigError> {
    let contents = match fs::read_to_string(path) {
      Ok(s) => s,
      Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(CommandsFile::default()),
      Err(e) => {
        return Err(ConfigError::Read {
          path: path.to_path_buf(),
          source: e,
        });
      }
    };

    toml::from_str(&contents).map_err(|e| ConfigError::Parse {
      path: path.to_path_buf(),
      source: Box::new(e),
    })
  }

  /// Read, decrypt, and parse the encrypted progress file.
  fn load_progress(path: &Path) -> Result<ProgressFile, ConfigError> {
    let data = match fs::read(path) {
      Ok(d) => d,
      Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(ProgressFile::default()),
      Err(e) => {
        return Err(ConfigError::Read {
          path: path.to_path_buf(),
          source: e,
        });
      }
    };

    let plain = crate::crypto::open(&PROGRESS_KEY, &data).map_err(|e| ConfigError::Decrypt {
      path: path.to_path_buf(),
      source: e,
    })?;

    serde_json::from_slice(&plain).map_err(|e| ConfigError::ProgressParse {
      path: path.to_path_buf(),
      source: e,
    })
  }

  /// Write plaintext commands to `lq.toml` (`path`) and the encrypted progress
  /// to the sibling [`PROGRESS_FILE`].
  ///
  /// Maps serialization errors to [`ConfigError::Serialize`] and I/O errors to
  /// [`ConfigError::Write`].
  pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
    // Grade mode is ro, just return
    if self.grade_mode {
      return Ok(());
    }

    // Plaintext commands -> lq.toml
    let commands = CommandsFile {
      rust: self.rust.clone(),
      python: self.python.clone(),
      go: self.go.clone(),
      cpp: self.cpp.clone(),
      plantuml: self.plantuml.clone(),
      ide: self.ide.clone(),
      ripes: self.ripes.clone(),
    };
    let contents = toml::to_string(&commands).map_err(|e| ConfigError::Serialize { source: e })?;
    fs::write(path, contents).map_err(|e| ConfigError::Write {
      path: path.to_path_buf(),
      source: e,
    })?;

    // Encrypted progress -> .lq.progress
    let progress = ProgressFile {
      owner: self.owner.clone(),
      current_exercise: self.current_exercise.clone(),
      exercises: self.exercises.clone(),
    };
    let plain = serde_json::to_vec(&progress).expect("progress serialises");
    let sealed = crate::crypto::seal(&PROGRESS_KEY, &plain);
    let ppath = progress_path(path);
    fs::write(&ppath, sealed).map_err(|e| ConfigError::Write { path: ppath, source: e })
  }

  /// Return the [`ExerciseState`] for the given exercise path, or a default
  /// state if no entry exists yet.
  pub fn get_state(&self, exercise_path: &str) -> ExerciseState {
    self.exercises.get(exercise_path).cloned().unwrap_or_default()
  }

  /// Update the score for an exercise (without test-count details).
  ///
  /// - `best_score` is only updated if `score` is strictly higher (monotonic increase).
  /// - `passed` is set to `true` when `score >= threshold` and is sticky (never reset).
  pub fn update_score(&mut self, exercise_path: &str, score: f64, threshold: f64) {
    self.record_verification(exercise_path, score, threshold, 0, 0);
  }

  /// Record a full verification result, including unit-test counts.
  ///
  /// Behaves like [`update_score`](Self::update_score) for `best_score` /
  /// `passed`, and additionally:
  /// - stores `tests_total` whenever it is known (`> 0`), since it is a fixed
  ///   property of the exercise;
  /// - records `best_tests_passed` alongside a new best score, so the persisted
  ///   pass count always corresponds to the best score (partial-credit grading).
  pub fn record_verification(&mut self, exercise_path: &str, score: f64, threshold: f64, tests_passed: usize, tests_total: usize) {
    let state = self.exercises.entry(exercise_path.to_owned()).or_default();

    if score > state.best_score {
      state.best_score = score;
      state.best_tests_passed = tests_passed;
    }

    // The total is a stable property of the exercise; keep it current whenever
    // a verification actually ran (avoids clobbering a known total with 0 from
    // the count-less `update_score` path).
    if tests_total > 0 {
      state.tests_total = tests_total;
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

  /// Mark the solution as seen only if the exercise has **not** been passed.
  ///
  /// Viewing the reference solution after already passing all tests is "free":
  /// a student who has solved the exercise may study the solution without being
  /// recorded as having needed it. Returns `true` if the flag was newly set.
  pub fn mark_solution_seen_if_unpassed(&mut self, exercise_path: &str) -> bool {
    if self.get_state(exercise_path).passed {
      return false;
    }
    self.mark_solution_seen(exercise_path);
    true
  }

  /// Ensure `hints_max` is initialised to `"0/{hints_total}"` when it is
  /// still empty (e.g. for a freshly discovered exercise).  No-op once the
  /// field already has a value.
  pub fn init_hints_max(&mut self, exercise_path: &str, hints_total: usize) {
    let state = self.exercises.entry(exercise_path.to_owned()).or_default();
    if state.hints_max.is_empty() {
      state.hints_max = format!("0/{}", hints_total);
    }
  }

  /// Record that the user revealed one more hint for an exercise.
  ///
  /// - `hints_shown` is incremented by 1 (cumulative across sessions).
  /// - `hints_max` is updated to `"{hints_revealed}/{hints_total}"` whenever
  ///   `hints_revealed` exceeds the previously stored numerator.
  pub fn record_hint_reveal(&mut self, exercise_path: &str, hints_revealed: usize, hints_total: usize) {
    let state = self.exercises.entry(exercise_path.to_owned()).or_default();

    state.hints_shown += 1;

    // Initialise to "0/<total>" if still empty (shouldn't normally happen
    // because App::new pre-initialises all exercises, but be defensive).
    if state.hints_max.is_empty() {
      state.hints_max = format!("0/{}", hints_total);
    }

    let current_max = state.hints_max.split('/').next().and_then(|s| s.parse::<usize>().ok()).unwrap_or(0);
    if hints_revealed > current_max {
      state.hints_max = format!("{}/{}", hints_revealed, hints_total);
    }
  }

  /// Increment the save counter for an exercise.
  pub fn record_save(&mut self, exercise_path: &str) {
    self.exercises.entry(exercise_path.to_owned()).or_default().times_saved += 1;
  }

  /// Record a hint reveal, but only if the exercise has **not** been passed.
  ///
  /// Once an exercise is solved, revealing hints for study is "free" and does
  /// not increase its hint counters. Returns `true` if the reveal was recorded.
  pub fn record_hint_reveal_if_unpassed(&mut self, exercise_path: &str, hints_revealed: usize, hints_total: usize) -> bool {
    if self.get_state(exercise_path).passed {
      return false;
    }
    self.record_hint_reveal(exercise_path, hints_revealed, hints_total);
    true
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

/// Return the path to the encrypted progress file, given the `lq.toml` path.
///
/// The progress file lives in the same directory as `lq.toml`.
pub fn progress_path(config_path: &Path) -> PathBuf {
  config_path.parent().unwrap_or_else(|| Path::new(".")).join(PROGRESS_FILE)
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
    assert_eq!(state.hints_shown, 0);
    assert!(state.hints_max.is_empty());
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
    // Simulate 3 hint reveals in first session, 2 more in a second session
    cfg.record_hint_reveal("01-basics/01-hello", 1, 5);
    cfg.record_hint_reveal("01-basics/01-hello", 2, 5);
    cfg.record_hint_reveal("01-basics/01-hello", 3, 5);

    cfg.save(&path).expect("save should succeed");
    let loaded = ProjectConfig::load(&path).expect("load should succeed");

    assert_eq!(loaded.current_exercise.as_deref(), Some("01-basics/01-hello"));
    let state = loaded.get_state("01-basics/01-hello");
    assert_eq!(state.best_score, 0.8);
    assert!(state.passed);
    assert!(state.solution_seen);
    // Cumulative counter: 3 reveal events across sessions
    assert_eq!(state.hints_shown, 3);
    // Furthest level reached: 3 out of 5
    assert_eq!(state.hints_max, "3/5");

    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn grade_mode_never_writes_progress_file() {
    let dir = std::env::temp_dir().join("lq_test_grade_save");
    let _ = fs::create_dir_all(&dir);
    let path = dir.join("lq.toml");
    fs::write(&path, "# original\n").unwrap();

    let mut cfg = ProjectConfig::default();
    cfg.update_score("01-basics/01-hello", 1.0, 0.7);
    cfg.grade_mode = true;
    cfg.save(&path).unwrap();

    // Neither the plaintext lq.toml nor the encrypted progress file should be modified
    // (or created in this case).
    assert_eq!(fs::read_to_string(&path).unwrap(), "# original\n");
    assert!(!progress_path(&path).exists());

    let _ = fs::remove_dir_all(&dir);
  }

  #[test]
  fn get_state_returns_default_for_unknown_exercise() {
    let cfg = ProjectConfig::default();
    let state = cfg.get_state("nonexistent/exercise");
    assert_eq!(state.best_score, 0.0);
    assert!(!state.passed);
    assert!(!state.solution_seen);
    assert_eq!(state.hints_shown, 0);
    assert!(state.hints_max.is_empty());
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
  fn record_verification_tracks_test_counts_with_best_score() {
    let mut cfg = ProjectConfig::default();

    // First run: 2/4 tests, below threshold.
    cfg.record_verification("ex", 0.5, 0.7, 2, 4);
    let s = cfg.get_state("ex");
    assert_eq!(s.best_score, 0.5);
    assert_eq!(s.best_tests_passed, 2);
    assert_eq!(s.tests_total, 4);
    assert!(!s.passed);

    // Regression run: lower score must not change best_score or its pass count,
    // but the (stable) total is still refreshed.
    cfg.record_verification("ex", 0.25, 0.7, 1, 4);
    let s = cfg.get_state("ex");
    assert_eq!(s.best_score, 0.5);
    assert_eq!(s.best_tests_passed, 2);
    assert_eq!(s.tests_total, 4);

    // New best: all tests pass and threshold crossed.
    cfg.record_verification("ex", 1.0, 0.7, 4, 4);
    let s = cfg.get_state("ex");
    assert_eq!(s.best_score, 1.0);
    assert_eq!(s.best_tests_passed, 4);
    assert!(s.passed);
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
  fn mark_solution_seen_if_unpassed_records_only_before_passing() {
    let mut cfg = ProjectConfig::default();

    // Not yet passed: viewing the solution is recorded (student needed help).
    assert!(cfg.mark_solution_seen_if_unpassed("ex"));
    assert!(cfg.get_state("ex").solution_seen);

    // Once passed, a fresh exercise's post-pass view is NOT recorded.
    cfg.update_score("passed-ex", 1.0, 0.7);
    assert!(!cfg.mark_solution_seen_if_unpassed("passed-ex"));
    assert!(!cfg.get_state("passed-ex").solution_seen);
  }

  #[test]
  fn default_times_saved_0() {
    let cfg = ProjectConfig::default();
    assert_eq!(cfg.get_state("ex").times_saved, 0);
  }

  #[test]
  fn record_save_normal_case() {
    let mut cfg = ProjectConfig::default();
    cfg.record_save("ex");
    assert_eq!(cfg.get_state("ex").times_saved, 1);
  }

  #[test]
  fn record_hint_reveal_if_unpassed_stops_counting_after_pass() {
    let mut cfg = ProjectConfig::default();

    // Before passing: reveals are counted.
    assert!(cfg.record_hint_reveal_if_unpassed("ex", 1, 5));
    assert!(cfg.record_hint_reveal_if_unpassed("ex", 2, 5));
    assert_eq!(cfg.get_state("ex").hints_shown, 2);
    assert_eq!(cfg.get_state("ex").hints_max, "2/5");

    // After passing: further reveals do not increase the counters.
    cfg.update_score("ex", 1.0, 0.7);
    assert!(!cfg.record_hint_reveal_if_unpassed("ex", 3, 5));
    assert_eq!(cfg.get_state("ex").hints_shown, 2);
    assert_eq!(cfg.get_state("ex").hints_max, "2/5");
  }

  #[test]
  fn record_hint_reveal_cumulative_and_max() {
    let mut cfg = ProjectConfig::default();

    // Session 1: reveal 3 hints sequentially
    cfg.record_hint_reveal("ex", 1, 5);
    cfg.record_hint_reveal("ex", 2, 5);
    cfg.record_hint_reveal("ex", 3, 5);

    let state = cfg.get_state("ex");
    assert_eq!(state.hints_shown, 3); // 3 reveal events
    assert_eq!(state.hints_max, "3/5"); // furthest level = 3

    // Session 2: user comes back and reveals hints again (levels 1, 2, 3 again)
    cfg.record_hint_reveal("ex", 1, 5);
    cfg.record_hint_reveal("ex", 2, 5);

    let state = cfg.get_state("ex");
    assert_eq!(state.hints_shown, 5); // 3 + 2 = 5 reveal events
    assert_eq!(state.hints_max, "3/5"); // furthest level unchanged (never exceeded 3)

    // Session 3: user goes further and reaches level 5
    cfg.record_hint_reveal("ex", 4, 5);
    cfg.record_hint_reveal("ex", 5, 5);

    let state = cfg.get_state("ex");
    assert_eq!(state.hints_shown, 7); // 5 + 2 = 7 reveal events
    assert_eq!(state.hints_max, "5/5"); // furthest level now 5
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
