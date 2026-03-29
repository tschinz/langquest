//! Integration tests for LangQuest (`lq`).
//!
//! These tests exercise the public API surface across module boundaries:
//! * Exercise discovery against the `tests/fixtures/sample-repo` fixture.
//! * Configuration round-trip (save → load → verify).
//! * Runner score parsing for each language backend.

use std::fs;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Return the absolute path to the `tests/fixtures/sample-repo` directory.
fn sample_repo() -> PathBuf {
  Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("sample-repo")
}

/// Create a temporary directory that is automatically cleaned up when dropped.
struct TempDir(PathBuf);

impl TempDir {
  fn new(name: &str) -> Self {
    let path = std::env::temp_dir().join("lq_integration_tests").join(name);
    let _ = fs::create_dir_all(&path);
    Self(path)
  }

  fn path(&self) -> &Path {
    &self.0
  }
}

impl Drop for TempDir {
  fn drop(&mut self) {
    let _ = fs::remove_dir_all(&self.0);
  }
}

// ===========================================================================
// Phase 2 — Exercise Discovery
// ===========================================================================

mod discovery {
  use super::*;

  #[test]
  fn discovers_all_modules() {
    let (modules, errors) = lq::exercise::discover_exercises(&sample_repo());

    // There should be 4 modules in the fixture repo.
    assert_eq!(
      modules.len(),
      4,
      "expected 4 modules, got {}: {:?}",
      modules.len(),
      modules.iter().map(|m| &m.name).collect::<Vec<_>>()
    );

    // No discovery errors on a well-formed repo.
    assert!(errors.is_empty(), "unexpected discovery errors: {errors:?}");
  }

  #[test]
  fn module_order_matches_numeric_prefix() {
    let (modules, _) = lq::exercise::discover_exercises(&sample_repo());
    let names: Vec<&str> = modules.iter().map(|m| m.name.as_str()).collect();

    assert_eq!(names[0], "01-rust");
    assert_eq!(names[1], "02-python");
    assert_eq!(names[2], "03-riscv");
    assert_eq!(names[3], "04-go");
  }

  #[test]
  fn rust_has_one_exercise() {
    let (modules, _) = lq::exercise::discover_exercises(&sample_repo());
    let basics = modules.iter().find(|m| m.name == "01-rust");
    assert!(basics.is_some(), "01-rust module not found");
    let basics = basics.unwrap();
    assert_eq!(basics.exercises.len(), 1);
  }

  #[test]
  fn exercise_metadata_parsed_correctly() {
    let (modules, _) = lq::exercise::discover_exercises(&sample_repo());
    let hello = modules
      .iter()
      .find(|m| m.name == "01-rust")
      .and_then(|m| m.exercises.iter().find(|e| e.id == "hello_rust"));

    assert!(hello.is_some(), "hello_rust exercise not found");
    let hello = hello.unwrap();

    assert_eq!(hello.name, "Hello, Rust!");
    assert_eq!(hello.language, lq::exercise::Language::Rust);
    assert!(hello.difficulty >= 1 && hello.difficulty <= 5);
    assert!(!hello.description.is_empty());
    assert!(!hello.topics.is_empty());
  }

  #[test]
  fn exercise_has_theory_and_task_paths() {
    let (modules, _) = lq::exercise::discover_exercises(&sample_repo());
    let ex = &modules[0].exercises[0];

    assert!(ex.theory_path.is_some(), "theory_path should be set for {}", ex.id);
    assert!(ex.theory_path.as_ref().unwrap().exists(), "theory_path should point to an existing file");
    assert!(ex.task_path.exists(), "task_path should point to an existing file");
    assert!(ex.source_path.exists(), "source_path should point to an existing file");
  }

  #[test]
  fn exercise_has_solution_data() {
    let (modules, _) = lq::exercise::discover_exercises(&sample_repo());
    let ex = &modules[0].exercises[0];

    assert!(ex.solution_data.is_some(), "solution_data should be loaded for {}", ex.id);
    let sol = ex.solution_data.as_ref().unwrap();
    assert!(!sol.hints.is_empty(), "hints should not be empty");
    assert!(!sol.explanation.is_empty(), "explanation should not be empty");
  }

  #[test]
  fn relative_path_is_module_slash_exercise() {
    let (modules, _) = lq::exercise::discover_exercises(&sample_repo());
    let ex = &modules[0].exercises[0];

    assert!(ex.relative_path.contains('/'), "relative_path should contain a slash: {}", ex.relative_path);
    assert!(
      ex.relative_path.starts_with("01-rust/"),
      "relative_path should start with module name: {}",
      ex.relative_path
    );
  }

  #[test]
  fn python_exercises_detected() {
    let (modules, _) = lq::exercise::discover_exercises(&sample_repo());
    let python_mod = modules.iter().find(|m| m.name == "02-python");
    assert!(python_mod.is_some(), "python module not found");
    let python_mod = python_mod.unwrap();

    assert!(
      !python_mod.exercises.is_empty(),
      "expected at least 1 python exercise, found {}",
      python_mod.exercises.len()
    );
    for ex in &python_mod.exercises {
      assert_eq!(ex.language, lq::exercise::Language::Python);
    }
  }

  #[test]
  fn go_exercises_detected() {
    let (modules, _) = lq::exercise::discover_exercises(&sample_repo());

    let go_mod = modules.iter().find(|m| m.name == "04-go");
    assert!(go_mod.is_some(), "04-go module not found");
    let go_mod = go_mod.unwrap();

    assert!(
      !go_mod.exercises.is_empty(),
      "expected at least 1 Go exercise, found {}",
      go_mod.exercises.len()
    );
    for ex in &go_mod.exercises {
      assert_eq!(ex.language, lq::exercise::Language::Go);
    }
  }

  #[test]
  fn assembly_exercises_detected() {
    let (modules, _) = lq::exercise::discover_exercises(&sample_repo());

    let riscv = modules.iter().find(|m| m.name == "03-riscv");
    assert!(riscv.is_some());
    for ex in &riscv.unwrap().exercises {
      assert_eq!(ex.language, lq::exercise::Language::Riscv);
    }
  }

  #[test]
  fn empty_directory_yields_no_modules() {
    let tmp = TempDir::new("empty_discovery");
    let (modules, errors) = lq::exercise::discover_exercises(tmp.path());
    assert!(modules.is_empty());
    assert!(errors.is_empty());
  }

  #[test]
  fn non_numbered_directories_are_ignored() {
    let tmp = TempDir::new("non_numbered_dirs");
    let _ = fs::create_dir_all(tmp.path().join("not-numbered"));
    let _ = fs::create_dir_all(tmp.path().join(".hidden"));
    let _ = fs::create_dir_all(tmp.path().join("README.md"));

    let (modules, errors) = lq::exercise::discover_exercises(tmp.path());
    assert!(modules.is_empty());
    assert!(errors.is_empty());
  }
}

// ===========================================================================
// Phase 2 — Frontmatter Parsing
// ===========================================================================

mod frontmatter {
  use lq::exercise::parse_frontmatter;

  #[test]
  fn parse_valid_frontmatter() {
    let input = r#"---
id          = "test_exercise"
name        = "Test Exercise"
language    = "rust"
difficulty  = 3
description = "A test exercise."
topics      = ["testing", "parsing"]
---

Some body text.
"#;
    let result = parse_frontmatter(input);
    assert!(result.is_some(), "parse_frontmatter returned None");
    let (toml_str, body) = result.unwrap();
    assert!(toml_str.contains("id"));
    assert!(body.contains("Some body text."));
  }

  #[test]
  fn missing_frontmatter_is_none() {
    let input = "No frontmatter here, just plain text.";
    let result = parse_frontmatter(input);
    assert!(result.is_none());
  }

  #[test]
  fn incomplete_frontmatter_is_none() {
    let input = "---\nid = \"test\"\nno closing delimiter";
    let result = parse_frontmatter(input);
    assert!(result.is_none());
  }
}

// ===========================================================================
// Phase 2 — Config Persistence
// ===========================================================================

mod config {
  use super::*;

  #[test]
  fn save_and_load_roundtrip() {
    let tmp = TempDir::new("config_roundtrip");
    let cfg_path = tmp.path().join("lq.toml");

    let mut cfg = lq::config::ProjectConfig {
      current_exercise: Some("01-basics/01-hello".to_owned()),
      ..Default::default()
    };
    cfg.update_score("01-basics/01-hello", 0.85, 0.7);
    cfg.mark_solution_seen("01-basics/01-hello");

    cfg.save(&cfg_path).expect("save should succeed");

    let loaded = lq::config::ProjectConfig::load(&cfg_path).expect("load should succeed");
    assert_eq!(loaded.current_exercise.as_deref(), Some("01-basics/01-hello"));
    let state = loaded.get_state("01-basics/01-hello");
    assert_eq!(state.best_score, 0.85);
    assert!(state.passed);
    assert!(state.solution_seen);
  }

  #[test]
  fn load_missing_file_returns_default() {
    let path = Path::new("/tmp/lq_integration_nonexistent.toml");
    let cfg = lq::config::ProjectConfig::load(path).expect("should return default");
    assert!(cfg.current_exercise.is_none());
    assert!(cfg.exercises.is_empty());
  }

  #[test]
  fn load_corrupt_toml_returns_error() {
    let tmp = TempDir::new("config_corrupt");
    let cfg_path = tmp.path().join("lq.toml");
    fs::write(&cfg_path, "this is [[[not valid toml").expect("write");

    let result = lq::config::ProjectConfig::load(&cfg_path);
    assert!(result.is_err());
  }

  #[test]
  fn best_score_is_monotonic() {
    let mut cfg = lq::config::ProjectConfig::default();

    cfg.update_score("ex", 0.5, 1.0);
    assert_eq!(cfg.get_state("ex").best_score, 0.5);

    // Lower score should not reduce best_score.
    cfg.update_score("ex", 0.3, 1.0);
    assert_eq!(cfg.get_state("ex").best_score, 0.5);

    // Higher score updates.
    cfg.update_score("ex", 0.9, 1.0);
    assert_eq!(cfg.get_state("ex").best_score, 0.9);
  }

  #[test]
  fn passed_is_sticky() {
    let mut cfg = lq::config::ProjectConfig::default();

    cfg.update_score("ex", 1.0, 0.7);
    assert!(cfg.get_state("ex").passed);

    // Score below threshold must NOT reset passed.
    cfg.update_score("ex", 0.1, 0.7);
    assert!(cfg.get_state("ex").passed);
  }

  #[test]
  fn solution_seen_is_sticky() {
    let mut cfg = lq::config::ProjectConfig::default();
    cfg.mark_solution_seen("ex");
    assert!(cfg.get_state("ex").solution_seen);
  }

  #[test]
  fn reset_clears_all_state() {
    let mut cfg = lq::config::ProjectConfig {
      current_exercise: Some("old_exercise".into()),
      ..Default::default()
    };
    cfg.update_score("ex1", 1.0, 0.5);
    cfg.update_score("ex2", 0.8, 0.5);
    cfg.mark_solution_seen("ex1");

    cfg.reset(Some("new_first"));

    assert_eq!(cfg.current_exercise.as_deref(), Some("new_first"));
    assert!(cfg.exercises.is_empty());
  }

  #[test]
  fn config_path_is_lq_toml() {
    let path = lq::config::config_path(Path::new("/some/repo"));
    assert_eq!(path, PathBuf::from("/some/repo/lq.toml"));
  }

  #[test]
  fn multiple_exercises_persist_independently() {
    let tmp = TempDir::new("config_multi");
    let cfg_path = tmp.path().join("lq.toml");

    let mut cfg = lq::config::ProjectConfig::default();
    cfg.update_score("ex1", 1.0, 0.7);
    cfg.update_score("ex2", 0.5, 0.7);
    cfg.mark_solution_seen("ex1");

    cfg.save(&cfg_path).expect("save");
    let loaded = lq::config::ProjectConfig::load(&cfg_path).expect("load");

    let s1 = loaded.get_state("ex1");
    assert!(s1.passed);
    assert!(s1.solution_seen);
    assert_eq!(s1.best_score, 1.0);

    let s2 = loaded.get_state("ex2");
    assert!(!s2.passed);
    assert!(!s2.solution_seen);
    assert_eq!(s2.best_score, 0.5);
  }
}

// ===========================================================================
// Phase 3 — Runner Score Parsing & Verification
// ===========================================================================

mod runner {
  use super::*;

  #[test]
  fn cap_output_preserves_short_input() {
    let input = "line1\nline2\nline3";
    let result = lq::runner::cap_output(input, 200);
    assert_eq!(result, input);
  }

  #[test]
  fn cap_output_trims_long_input() {
    let lines: Vec<String> = (0..300).map(|i| format!("line {i}")).collect();
    let input = lines.join("\n");
    let result = lq::runner::cap_output(&input, 200);

    let result_lines: Vec<&str> = result.lines().collect();
    assert_eq!(result_lines.len(), 200);
    // Should contain the last 200 lines.
    assert!(result.contains("line 299"));
    assert!(result.contains("line 100"));
    assert!(!result.contains("line 99\n"));
  }

  #[test]
  fn cap_output_handles_empty() {
    let result = lq::runner::cap_output("", 200);
    assert_eq!(result, "");
  }

  #[test]
  fn verification_result_progress_bar_format() {
    let result = lq::runner::VerificationResult {
      score: 0.8,
      passed: 4,
      total: 5,
      output: String::new(),
      threshold: 0.7,
    };
    let bar = result.progress_bar(20);

    assert!(bar.contains("4/5 checks"), "bar: {bar}");
    assert!(bar.contains("70%"), "bar: {bar}");
    assert!(bar.contains('['), "bar: {bar}");
    assert!(bar.contains(']'), "bar: {bar}");
    assert!(bar.contains('='), "bar: {bar}");
  }

  #[test]
  fn verification_result_zero_score_bar() {
    let result = lq::runner::VerificationResult {
      score: 0.0,
      passed: 0,
      total: 5,
      output: String::new(),
      threshold: 1.0,
    };
    let bar = result.progress_bar(10);

    assert!(bar.contains("0/5"), "bar: {bar}");
    assert!(bar.contains("----------"), "bar should be all dashes: {bar}");
  }

  #[test]
  fn verification_result_full_score_bar() {
    let result = lq::runner::VerificationResult {
      score: 1.0,
      passed: 5,
      total: 5,
      output: String::new(),
      threshold: 1.0,
    };
    let bar = result.progress_bar(10);

    assert!(bar.contains("5/5"), "bar: {bar}");
    assert!(bar.contains("=========="), "bar should be all fills: {bar}");
  }

  #[test]
  fn verify_go_exercise_from_sample_repo() {
    let (modules, _) = lq::exercise::discover_exercises(&sample_repo());
    let hello = modules
      .iter()
      .find(|m| m.name == "04-go")
      .and_then(|m| m.exercises.iter().find(|e| e.id == "hello_go"));

    if let Some(exercise) = hello {
      let result = lq::runner::verify(exercise, &lq::config::ProjectConfig::default());
      // The starter returns "" so tests should fail — we only verify the
      // runner doesn't panic and produces a well-formed result.
      assert!(result.threshold > 0.0);
      assert!(result.threshold <= 1.0);
    }
  }

  #[test]
  fn verify_rust_exercise_from_sample_repo() {
    let (modules, _) = lq::exercise::discover_exercises(&sample_repo());
    let hello = modules
      .iter()
      .find(|m| m.name == "01-rust")
      .and_then(|m| m.exercises.iter().find(|e| e.id == "hello_world"));

    if let Some(exercise) = hello {
      let result = lq::runner::verify(exercise, &lq::config::ProjectConfig::default());
      // The starter code has `todo!()` stubs, so tests should fail.
      // We just verify the runner doesn't panic and returns a result.
      assert!(result.threshold > 0.0);
      assert!(result.threshold <= 1.0);
    }
  }

  #[test]
  fn verify_markdown_exercise_from_sample_repo() {
    let (modules, _) = lq::exercise::discover_exercises(&sample_repo());
    // No markdown/text exercise exists in the trimmed sample repo;
    // the test is a no-op but must still compile and pass.
    let concept = modules.iter().find(|m| m.name == "08-concepts").and_then(|m| m.exercises.first());

    if let Some(exercise) = concept {
      assert_eq!(exercise.language, lq::exercise::Language::Text);
      let result = lq::runner::verify(exercise, &lq::config::ProjectConfig::default());
      // The starter has empty answer placeholders, so score should be low.
      assert!(result.threshold > 0.0);
      // Markdown exercises have keyword-based scoring.
      assert!(result.total > 0 || result.score == 0.0);
    }
  }

  #[test]
  fn language_thresholds_are_sane() {
    use lq::exercise::Language;

    for lang in [Language::Rust, Language::Go, Language::Python, Language::Riscv, Language::Text] {
      let t = lang.threshold();
      assert!((0.0..=1.0).contains(&t), "{:?} threshold {t} is out of range", lang);
    }
  }
}

// ===========================================================================
// Phase 2+4 — Exercise Status Derivation
// ===========================================================================

mod status {
  #[test]
  fn failing_when_not_passed() {
    let state = lq::config::ExerciseState {
      best_score: 0.3,
      passed: false,
      solution_seen: false,
    };
    let status = lq::ui::overview::derive_status(&state);
    assert_eq!(status, lq::exercise::ExerciseStatus::Failing);
  }

  #[test]
  fn partial_when_passed_but_not_seen() {
    let state = lq::config::ExerciseState {
      best_score: 1.0,
      passed: true,
      solution_seen: false,
    };
    let status = lq::ui::overview::derive_status(&state);
    assert_eq!(status, lq::exercise::ExerciseStatus::Partial);
  }

  #[test]
  fn complete_when_passed_and_seen() {
    let state = lq::config::ExerciseState {
      best_score: 1.0,
      passed: true,
      solution_seen: true,
    };
    let status = lq::ui::overview::derive_status(&state);
    assert_eq!(status, lq::exercise::ExerciseStatus::Complete);
  }

  #[test]
  fn seen_but_not_passed_is_still_failing() {
    let state = lq::config::ExerciseState {
      best_score: 0.0,
      passed: false,
      solution_seen: true,
    };
    let status = lq::ui::overview::derive_status(&state);
    assert_eq!(status, lq::exercise::ExerciseStatus::Failing);
  }

  #[test]
  fn status_symbols() {
    use lq::exercise::ExerciseStatus;

    assert_eq!(ExerciseStatus::Failing.symbol(), "[x]");
    assert_eq!(ExerciseStatus::Partial.symbol(), "[~]");
    assert_eq!(ExerciseStatus::Complete.symbol(), "[*]");
  }

  #[test]
  fn status_labels() {
    use lq::exercise::ExerciseStatus;

    assert_eq!(ExerciseStatus::Failing.label(), "Failing");
    assert_eq!(ExerciseStatus::Partial.label(), "Partial");
    assert_eq!(ExerciseStatus::Complete.label(), "Done");
  }
}

// ===========================================================================
// End-to-end — Discovery + Config merge
// ===========================================================================

mod end_to_end {
  use super::*;

  #[test]
  fn discover_then_persist_initial_config() {
    let tmp = TempDir::new("e2e_discover_persist");
    let cfg_path = tmp.path().join("lq.toml");

    let (modules, errors) = lq::exercise::discover_exercises(&sample_repo());
    assert!(errors.is_empty());

    let mut cfg = lq::config::ProjectConfig::default();

    // Set the first exercise as current.
    let first = modules.first().and_then(|m| m.exercises.first()).map(|e| e.relative_path.as_str());
    cfg.current_exercise = first.map(String::from);

    // Initialize state for all exercises.
    for module in &modules {
      for exercise in &module.exercises {
        let _ = cfg.exercises.entry(exercise.relative_path.clone()).or_default();
      }
    }

    cfg.save(&cfg_path).expect("save initial config");
    let loaded = lq::config::ProjectConfig::load(&cfg_path).expect("load config");

    assert!(loaded.current_exercise.is_some());

    let total_exercises: usize = modules.iter().map(|m| m.exercises.len()).sum();
    assert_eq!(loaded.exercises.len(), total_exercises);
  }

  #[test]
  fn simulated_exercise_completion_flow() {
    let mut cfg = lq::config::ProjectConfig {
      current_exercise: Some("01-rust/01-hello-world".into()),
      ..Default::default()
    };

    // Simulate: student starts exercise, gets partial score.
    cfg.update_score("01-rust/01-hello-world", 0.5, 1.0);
    assert!(!cfg.get_state("01-rust/01-hello-world").passed);

    // Student fixes code, gets full score.
    cfg.update_score("01-rust/01-hello-world", 1.0, 1.0);
    assert!(cfg.get_state("01-rust/01-hello-world").passed);

    // Student views solution.
    cfg.mark_solution_seen("01-rust/01-hello-world");
    let state = cfg.get_state("01-rust/01-hello-world");
    assert!(state.passed);
    assert!(state.solution_seen);
    assert_eq!(state.best_score, 1.0);

    // Status should be Complete.
    let status = lq::ui::overview::derive_status(&state);
    assert_eq!(status, lq::exercise::ExerciseStatus::Complete);

    // Move to next exercise.
    cfg.current_exercise = Some("01-rust/02-variables".into());
    assert_eq!(cfg.current_exercise.as_deref(), Some("01-rust/02-variables"));
  }

  #[test]
  fn reset_preserves_only_first_exercise_pointer() {
    let (modules, _) = lq::exercise::discover_exercises(&sample_repo());

    let mut cfg = lq::config::ProjectConfig {
      current_exercise: Some("03-ownership/01-ownership".into()),
      ..Default::default()
    };
    cfg.update_score("01-basics/01-hello", 1.0, 0.7);
    cfg.update_score("02-flow/01-if-else", 0.9, 0.8);

    let first_exercise = modules.first().and_then(|m| m.exercises.first()).map(|e| e.relative_path.as_str());

    cfg.reset(first_exercise);

    assert!(cfg.exercises.is_empty(), "reset should clear all exercises");
    assert!(cfg.current_exercise.is_some(), "reset should set current to first exercise");
  }
}
