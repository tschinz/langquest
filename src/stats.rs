//! Statistics module for the `lq stats` subcommand.
//!
//! Reads the project config (`lq.toml`) and exercise tree to produce a
//! detailed progress report.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use anyhow::bail;
use serde::Serialize;

use crate::config::{self, ProjectConfig};
use crate::exercise;

/// Filename of the machine-readable evaluation export written by `lq -s`.
const RESULTS_FILE: &str = "results.toml";

/// Per-module statistics accumulator.
#[derive(Debug, Default, PartialEq)]
pub struct PerModuleStats {
  pub total: usize,
  pub completed: usize,
  pub solutions_seen: usize,
  pub hints_shown: usize,
  pub hints_max_sum: usize,
  pub hints_total_sum: usize,
  pub best_score_sum: f64,
  /// Sum of unit tests passed (at each exercise's best score).
  pub tests_passed_sum: usize,
  /// Sum of total unit tests across exercises.
  pub tests_total_sum: usize,
}

/// Aggregated statistics report.
#[derive(Debug, Default, PartialEq)]
pub struct Report {
  /// GitHub identity the progress is bound to, decrypted from the tamper-proof
  /// progress file. Lets a teacher confirm the progress actually belongs to the
  /// student whose repo this is (a swapped file shows a different owner).
  /// `None` when progress has never been bound (no online launch yet).
  pub owner: Option<crate::identity::GithubIdentity>,
  /// Relative path of the current exercise, if one is set.
  pub current_exercise: Option<String>,
  pub total_exercises: usize,
  pub completed: usize,
  pub solutions_seen: usize,
  pub hints_shown: usize,
  pub hints_max_sum: usize,
  pub hints_total_sum: usize,
  pub best_score_sum: f64,
  /// Sum of unit tests passed (at each exercise's best score).
  pub tests_passed_sum: usize,
  /// Sum of total unit tests across all exercises.
  pub tests_total_sum: usize,
  pub by_module: BTreeMap<String, PerModuleStats>,
}

// ---------------------------------------------------------------------------
// Machine-readable results export (results.toml)
// ---------------------------------------------------------------------------

/// Full machine-readable evaluation, serialized to `results.toml`.
#[derive(Debug, Serialize, PartialEq)]
struct Results {
  meta: Meta,
  student: StudentInfo,
  summary: Summary,
  /// Per top-level module aggregates, keyed by module name.
  modules: BTreeMap<String, ModuleResult>,
  /// Flat per-exercise results (serialized as `[[exercises]]`).
  exercises: Vec<ExerciseResult>,
}

/// Provenance metadata for the export.
#[derive(Debug, Serialize, PartialEq)]
struct Meta {
  /// Unix timestamp (seconds) when the export was generated.
  generated_at: u64,
  /// Version of the `lq` binary that produced the file.
  lq_version: String,
}

/// The GitHub identity the progress is bound to.
#[derive(Debug, Serialize, PartialEq)]
struct StudentInfo {
  /// Whether progress has ever been bound to a GitHub account.
  verified: bool,
  /// GitHub login (empty when unverified).
  login: String,
  /// Immutable numeric GitHub id (0 when unverified).
  github_id: u64,
}

/// Overall totals across all exercises.
#[derive(Debug, Serialize, PartialEq)]
struct Summary {
  total_exercises: usize,
  completed: usize,
  /// Unit tests passed across all exercises (at each exercise's best score).
  tests_passed: usize,
  /// Total unit tests across all exercises.
  tests_total: usize,
  solutions_seen: usize,
  /// Cumulative hint presses (only counted while unsolved).
  hints_shown: usize,
  /// Sum of the furthest hint level reached per exercise.
  hints_explored: usize,
  /// Sum of the total hints available across exercises.
  hints_total: usize,
  /// Mean of per-exercise best scores in `[0.0, 1.0]`.
  average_best_score: f64,
}

/// Per-module aggregate.
#[derive(Debug, Serialize, PartialEq)]
struct ModuleResult {
  total: usize,
  completed: usize,
  tests_passed: usize,
  tests_total: usize,
  solutions_seen: usize,
  hints_shown: usize,
  average_best_score: f64,
}

/// One exercise's full evaluation record.
#[derive(Debug, Serialize, PartialEq)]
struct ExerciseResult {
  path: String,
  name: String,
  language: String,
  difficulty: u8,
  passed: bool,
  best_score: f64,
  /// Unit tests passed at the best score.
  tests_passed: usize,
  /// Total unit tests for this exercise.
  tests_total: usize,
  solution_seen: bool,
  /// Cumulative hint presses recorded for this exercise.
  hints_shown: usize,
  /// Furthest hint level reached.
  hints_revealed: usize,
  /// Total hints available for this exercise.
  hints_total: usize,
}

/// Build the machine-readable [`Results`] from config and the exercise list.
fn build_results(cfg: &ProjectConfig, all_exercises: &[exercise::Exercise]) -> Results {
  let report = compute(cfg, all_exercises);

  let exercises = all_exercises
    .iter()
    .map(|ex| {
      let state = cfg.get_state(&ex.relative_path);
      let hints_total = ex.solution_data.as_ref().map(|s| s.hints.len()).unwrap_or(0);
      let hints_revealed = state.hints_max.split_once('/').and_then(|(r, _)| r.parse::<usize>().ok()).unwrap_or(0);
      ExerciseResult {
        path: ex.relative_path.clone(),
        name: ex.name.clone(),
        language: ex.language.code().to_string(),
        difficulty: ex.difficulty,
        passed: state.passed,
        best_score: state.best_score,
        tests_passed: state.best_tests_passed,
        tests_total: if ex.test_count > 0 { ex.test_count } else { state.tests_total },
        solution_seen: state.solution_seen,
        hints_shown: state.hints_shown,
        hints_revealed,
        hints_total,
      }
    })
    .collect();

  let modules = report
    .by_module
    .iter()
    .map(|(name, m)| {
      (
        name.clone(),
        ModuleResult {
          total: m.total,
          completed: m.completed,
          tests_passed: m.tests_passed_sum,
          tests_total: m.tests_total_sum,
          solutions_seen: m.solutions_seen,
          hints_shown: m.hints_shown,
          average_best_score: if m.total > 0 { m.best_score_sum / m.total as f64 } else { 0.0 },
        },
      )
    })
    .collect();

  let (verified, login, github_id) = match &report.owner {
    Some(o) => (true, o.login.clone(), o.id),
    None => (false, String::new(), 0),
  };

  Results {
    meta: Meta {
      generated_at: crate::identity::unix_now(),
      lq_version: env!("CARGO_PKG_VERSION").to_string(),
    },
    student: StudentInfo { verified, login, github_id },
    summary: Summary {
      total_exercises: report.total_exercises,
      completed: report.completed,
      tests_passed: report.tests_passed_sum,
      tests_total: report.tests_total_sum,
      solutions_seen: report.solutions_seen,
      hints_shown: report.hints_shown,
      hints_explored: report.hints_max_sum,
      hints_total: report.hints_total_sum,
      average_best_score: if report.total_exercises > 0 {
        report.best_score_sum / report.total_exercises as f64
      } else {
        0.0
      },
    },
    modules,
    exercises,
  }
}

/// Serialize [`Results`] to TOML and write it to `path`.
fn write_results(path: &Path, cfg: &ProjectConfig, all_exercises: &[exercise::Exercise]) -> Result<()> {
  let results = build_results(cfg, all_exercises);
  let toml = toml::to_string_pretty(&results)?;
  fs::write(path, toml)?;
  Ok(())
}

/// Run the `stats` subcommand: load config, discover exercises, compute and
/// print statistics, and write a machine-readable `results.toml`.
pub fn run(repo_path: &Path) -> Result<()> {
  let cfg_path = config::config_path(repo_path);
  let cfg = ProjectConfig::load(&cfg_path)?;

  // Reading is open (teachers/anyone may inspect progress); the identity gate is
  // enforced on the write paths (TUI / reset). The bound owner is decrypted from
  // the tamper-proof blob and shown below so a swapped file reveals its true
  // owner.
  let (_tree, all_exercises, _errors) = exercise::discover_exercises(repo_path);

  if all_exercises.is_empty() {
    bail!("no exercises found in {}", repo_path.display());
  }

  let report = compute(&cfg, &all_exercises);
  render(&report);

  // Also emit a machine-readable evaluation for grading/automation.
  let results_path = repo_path.join(RESULTS_FILE);
  write_results(&results_path, &cfg, &all_exercises)?;
  println!("\n  Results written to {}", results_path.display());

  Ok(())
}

/// Compute aggregated statistics from the project config and exercise list.
fn compute(cfg: &ProjectConfig, all_exercises: &[exercise::Exercise]) -> Report {
  let mut report = Report {
    owner: cfg.owner.clone(),
    current_exercise: cfg.current_exercise.clone(),
    ..Default::default()
  };

  for ex in all_exercises {
    let state = cfg.get_state(&ex.relative_path);

    // Use the top-level directory as the module key so that
    // sub-modules (e.g. "01-rust/02-variables") are grouped under
    // their parent module.
    let module = ex.module_name.split_once('/').map(|(first, _)| first).unwrap_or(&ex.module_name).to_string();

    let stats = report.by_module.entry(module).or_default();
    stats.total += 1;
    report.total_exercises += 1;

    if state.passed {
      report.completed += 1;
      stats.completed += 1;
    }
    if state.solution_seen {
      report.solutions_seen += 1;
      stats.solutions_seen += 1;
    }

    report.hints_shown += state.hints_shown;
    stats.hints_shown += state.hints_shown;

    // Total available hints come from the exercise definition (`solution.md`),
    // so the denominator is correct even before the exercise has ever been
    // opened (i.e. when there is no persisted progress yet). The furthest hint
    // level *reached* comes from the numerator of the persisted `hints_max`.
    let hints_total = ex.solution_data.as_ref().map(|s| s.hints.len()).unwrap_or(0);
    let hints_revealed = state
      .hints_max
      .split_once('/')
      .and_then(|(revealed, _)| revealed.parse::<usize>().ok())
      .unwrap_or(0);

    report.hints_total_sum += hints_total;
    report.hints_max_sum += hints_revealed;
    stats.hints_total_sum += hints_total;
    stats.hints_max_sum += hints_revealed;

    report.best_score_sum += state.best_score;
    stats.best_score_sum += state.best_score;

    // Unit-test counts (partial-credit signal). The total comes from the
    // exercise's statically-counted tests so it is known before verification;
    // fall back to the last recorded runtime total if the static count is 0.
    // `best_tests_passed` is the pass count at the best score.
    let tests_total = if ex.test_count > 0 { ex.test_count } else { state.tests_total };
    report.tests_passed_sum += state.best_tests_passed;
    report.tests_total_sum += tests_total;
    stats.tests_passed_sum += state.best_tests_passed;
    stats.tests_total_sum += tests_total;
  }

  report
}

/// Render a statistics report to stdout.
fn render(report: &Report) {
  let total = report.total_exercises;

  println!("═══════════════════════════════════════");
  println!("         LangQuest Stats");
  println!("═══════════════════════════════════════");

  // Owner identity — decrypted from the progress file so a teacher can verify
  // the progress belongs to the expected student.
  println!();
  match &report.owner {
    Some(o) => println!("  Owner: {} (GitHub #{})", o.login, o.id),
    None => println!("  Owner: (unverified — no GitHub identity bound yet)"),
  }
  match &report.current_exercise {
    Some(name) => println!("  Current exercise: {name}"),
    None => println!("  Current exercise: (none set)"),
  }

  // Overall summary
  println!();
  println!("  Overall");
  println!("  ───────");
  println!("  Total exercises:      {total}");
  println!("  Completed (passed):   {} ({:.1}%)", report.completed, pct(report.completed, total));
  println!(
    "  Unit tests passed:    {}/{} ({:.1}%)",
    report.tests_passed_sum,
    report.tests_total_sum,
    pct(report.tests_passed_sum, report.tests_total_sum)
  );
  println!("  Solutions seen:       {} ({:.1}%)", report.solutions_seen, pct(report.solutions_seen, total));
  println!("  Hints revealed:       {} total presses", report.hints_shown);
  println!("  Hints explored:       {}", fmt_hints(report.hints_max_sum, report.hints_total_sum));

  if total > 0 {
    println!("  Average best score:   {:.1}%", report.best_score_sum / total as f64 * 100.0);
  }

  // Per-module breakdown
  if report.by_module.len() > 1 {
    println!();
    println!("  By Module");
    println!("  ──────────");
    for (mod_path, s) in &report.by_module {
      println!();
      println!("    {mod_path}");
      println!("      Exercises:          {}", s.total);
      println!("      Completed:          {} ({:.1}%)", s.completed, pct(s.completed, s.total));
      println!(
        "      Unit tests passed:  {}/{} ({:.1}%)",
        s.tests_passed_sum,
        s.tests_total_sum,
        pct(s.tests_passed_sum, s.tests_total_sum)
      );
      println!("      Solutions seen:     {} ({:.1}%)", s.solutions_seen, pct(s.solutions_seen, s.total));
      println!("      Hints revealed:     {} total presses", s.hints_shown);
      println!("      Hints explored:     {}", fmt_hints(s.hints_max_sum, s.hints_total_sum));
      if s.total > 0 {
        println!("      Average best score: {:.1}%", s.best_score_sum / s.total as f64 * 100.0);
      }
    }
  }

  println!();
  println!("═══════════════════════════════════════");
}

fn pct(part: usize, total: usize) -> f64 {
  if total == 0 { 0.0 } else { part as f64 / total as f64 * 100.0 }
}

fn fmt_hints(revealed: usize, total: usize) -> String {
  if total == 0 {
    "0/0".to_string()
  } else {
    let pct = revealed as f64 / total as f64 * 100.0;
    format!("{revealed}/{total} ({pct:.1}%)")
  }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;
  use std::path::PathBuf;

  fn make_exercise(relative_path: &str, module_name: &str) -> exercise::Exercise {
    make_exercise_hints(relative_path, module_name, 0)
  }

  /// Build a test exercise whose `solution.md` defines `n_hints` hints.
  fn make_exercise_hints(relative_path: &str, module_name: &str, n_hints: usize) -> exercise::Exercise {
    let solution_data = (n_hints > 0).then(|| exercise::SolutionData {
      title: "Test".to_string(),
      hints: vec!["hint".to_string(); n_hints],
      keywords: vec![],
      explanation: String::new(),
    });
    exercise::Exercise {
      id: "test".to_string(),
      name: "Test".to_string(),
      language: exercise::Language::Rust,
      difficulty: 1,
      description: "".to_string(),
      topics: vec![],
      module_name: module_name.to_string(),
      relative_path: relative_path.to_string(),
      dir: PathBuf::new(),
      theory_path: None,
      task_path: PathBuf::new(),
      source_path: PathBuf::new(),
      solution_source: None,
      solution_data,
      test_count: 0,
    }
  }

  #[test]
  fn empty_config_and_no_exercises() {
    let cfg = ProjectConfig::default();
    let exercises = vec![];
    let report = compute(&cfg, &exercises);

    assert_eq!(report.total_exercises, 0);
    assert_eq!(report.completed, 0);
    assert_eq!(report.solutions_seen, 0);
    assert_eq!(report.hints_shown, 0);
    assert_eq!(report.hints_max_sum, 0);
    assert_eq!(report.hints_total_sum, 0);
    assert_eq!(report.best_score_sum, 0.0);
    assert!(report.by_module.is_empty());
  }

  #[test]
  fn single_exercise_no_state() {
    let cfg = ProjectConfig::default();
    let exercises = vec![make_exercise("01-rust/01-hello", "01-rust")];
    let report = compute(&cfg, &exercises);

    assert_eq!(report.total_exercises, 1);
    assert_eq!(report.completed, 0);
    assert_eq!(report.solutions_seen, 0);
    assert_eq!(report.hints_shown, 0);
    assert_eq!(report.best_score_sum, 0.0);

    let m = report.by_module.get("01-rust").unwrap();
    assert_eq!(m.total, 1);
    assert_eq!(m.completed, 0);
    assert_eq!(m.solutions_seen, 0);
    assert_eq!(m.hints_shown, 0);
  }

  #[test]
  fn tracks_completed_and_solutions() {
    let mut cfg = ProjectConfig::default();
    cfg.update_score("01-rust/01-hello", 0.9, 0.7);
    cfg.mark_solution_seen("01-rust/01-hello");

    let exercises = vec![make_exercise("01-rust/01-hello", "01-rust")];
    let report = compute(&cfg, &exercises);

    assert_eq!(report.completed, 1);
    assert_eq!(report.solutions_seen, 1);
    assert!(report.best_score_sum > 0.0);

    let m = report.by_module.get("01-rust").unwrap();
    assert_eq!(m.completed, 1);
    assert_eq!(m.solutions_seen, 1);
    assert!(m.best_score_sum > 0.0);
  }

  #[test]
  fn accumulates_hints_shown() {
    let mut cfg = ProjectConfig::default();

    // Simulate hint presses across sessions
    cfg.record_hint_reveal("01-rust/01-hello", 1, 5);
    cfg.record_hint_reveal("01-rust/01-hello", 2, 5);
    cfg.record_hint_reveal("01-rust/02-variables", 1, 3);

    let exercises = vec![make_exercise("01-rust/01-hello", "01-rust"), make_exercise("01-rust/02-variables", "01-rust")];
    let report = compute(&cfg, &exercises);

    assert_eq!(report.hints_shown, 3); // 2 + 1

    let m = report.by_module.get("01-rust").unwrap();
    assert_eq!(m.hints_shown, 3);
  }

  #[test]
  fn revealed_from_progress_total_from_definition() {
    let mut cfg = ProjectConfig::default();

    // Furthest level reached is persisted in progress ("3/…"); the total comes
    // from the exercise's own 5 hints, not from the progress string.
    let state = cfg.exercises.entry("ex".to_string()).or_default();
    state.hints_max = "3/5".to_string();

    let exercises = vec![make_exercise_hints("ex", "01-rust", 5)];
    let report = compute(&cfg, &exercises);

    assert_eq!(report.hints_max_sum, 3);
    assert_eq!(report.hints_total_sum, 5);
  }

  #[test]
  fn total_shown_even_without_any_progress() {
    // The whole point of the fix: a never-opened repo (no progress at all)
    // still reports the correct hint total from the exercise definitions.
    let cfg = ProjectConfig::default();
    let exercises = vec![make_exercise_hints("ex", "01-rust", 4)];
    let report = compute(&cfg, &exercises);

    assert_eq!(report.hints_max_sum, 0);
    assert_eq!(report.hints_total_sum, 4);
  }

  #[test]
  fn ignores_unparseable_hints_max() {
    let mut cfg = ProjectConfig::default();
    let state = cfg.exercises.entry("ex".to_string()).or_default();
    state.hints_max = "not-valid".to_string();

    let exercises = vec![make_exercise("ex", "01-rust")];
    let report = compute(&cfg, &exercises);

    assert_eq!(report.hints_max_sum, 0);
    assert_eq!(report.hints_total_sum, 0);
  }

  #[test]
  fn groups_sub_modules_under_top_level() {
    let cfg = ProjectConfig::default();
    let exercises = vec![
      make_exercise("01-rust/01-hello", "01-rust"),
      make_exercise("01-rust/02-variables/01-strings", "01-rust/02-variables"),
      make_exercise("01-rust/02-variables/02-numbers", "01-rust/02-variables"),
      make_exercise("02-python/01-hello", "02-python"),
    ];
    let report = compute(&cfg, &exercises);

    assert_eq!(report.by_module.len(), 2);

    let rust = report.by_module.get("01-rust").unwrap();
    assert_eq!(rust.total, 3); // 1 + 2

    let python = report.by_module.get("02-python").unwrap();
    assert_eq!(python.total, 1);
  }

  #[test]
  fn multiple_modules_accumulate_independently() {
    let mut cfg = ProjectConfig::default();
    cfg.update_score("01-rust/01-hello", 0.9, 0.7);
    cfg.mark_solution_seen("01-rust/01-hello");
    cfg.record_hint_reveal("01-rust/01-hello", 1, 5);
    cfg.record_hint_reveal("01-rust/01-hello", 2, 5);
    cfg.record_hint_reveal("01-rust/01-hello", 3, 5);

    cfg.update_score("02-python/01-hello", 0.4, 0.7);
    cfg.record_hint_reveal("02-python/01-hello", 1, 3);

    let exercises = vec![make_exercise("01-rust/01-hello", "01-rust"), make_exercise("02-python/01-hello", "02-python")];
    let report = compute(&cfg, &exercises);

    assert_eq!(report.total_exercises, 2);
    assert_eq!(report.completed, 1);
    assert_eq!(report.solutions_seen, 1);
    assert_eq!(report.hints_shown, 4); // 3 + 1

    let rust = report.by_module.get("01-rust").unwrap();
    assert_eq!(rust.total, 1);
    assert_eq!(rust.completed, 1);
    assert_eq!(rust.solutions_seen, 1);
    assert_eq!(rust.hints_shown, 3);
    assert_eq!(rust.best_score_sum, 0.9);

    let python = report.by_module.get("02-python").unwrap();
    assert_eq!(python.total, 1);
    assert_eq!(python.completed, 0);
    assert_eq!(python.solutions_seen, 0);
    assert_eq!(python.hints_shown, 1);
    assert_eq!(python.best_score_sum, 0.4);
  }

  #[test]
  fn build_results_captures_per_exercise_and_summary() {
    let mut cfg = ProjectConfig {
      owner: Some(crate::identity::GithubIdentity {
        id: 7,
        login: "alice".to_string(),
      }),
      ..Default::default()
    };
    cfg.record_verification("01-rust/01-hello", 1.0, 0.7, 4, 4); // passed, 4/4 tests
    cfg.record_hint_reveal("01-rust/01-hello", 2, 5); // furthest level 2, one press

    let exercises = vec![make_exercise_hints("01-rust/01-hello", "01-rust", 5)];
    let results = build_results(&cfg, &exercises);

    assert!(results.student.verified);
    assert_eq!(results.student.login, "alice");
    assert_eq!(results.student.github_id, 7);

    assert_eq!(results.summary.total_exercises, 1);
    assert_eq!(results.summary.completed, 1);
    assert_eq!(results.summary.tests_passed, 4);
    assert_eq!(results.summary.tests_total, 4);
    assert_eq!(results.summary.hints_total, 5);
    let m = results.modules.get("01-rust").unwrap();
    assert_eq!(m.completed, 1);
    assert_eq!(m.tests_passed, 4);
    assert_eq!(m.tests_total, 4);

    assert_eq!(results.exercises.len(), 1);
    let e = &results.exercises[0];
    assert_eq!(e.path, "01-rust/01-hello");
    assert_eq!(e.language, "rust");
    assert!(e.passed);
    assert_eq!(e.best_score, 1.0);
    assert_eq!(e.tests_passed, 4);
    assert_eq!(e.tests_total, 4);
    assert_eq!(e.hints_total, 5);
    assert_eq!(e.hints_revealed, 2);
    assert_eq!(e.hints_shown, 1);

    // Must serialize to valid TOML.
    assert!(toml::to_string_pretty(&results).is_ok());
  }

  #[test]
  fn tests_total_uses_static_count_before_verification() {
    // An exercise with 4 statically-counted tests and NO progress must report
    // its total (0/4), not 0/0.
    let cfg = ProjectConfig::default();
    let mut ex = make_exercise("03-riscv/01-regs", "03-riscv");
    ex.test_count = 4;

    let report = compute(&cfg, std::slice::from_ref(&ex));
    assert_eq!(report.tests_passed_sum, 0);
    assert_eq!(report.tests_total_sum, 4);

    let results = build_results(&cfg, std::slice::from_ref(&ex));
    assert_eq!(results.exercises[0].tests_total, 4);
    assert_eq!(results.exercises[0].tests_passed, 0);
    assert_eq!(results.summary.tests_total, 4);
  }

  #[test]
  fn build_results_marks_unverified_without_owner() {
    let results = build_results(&ProjectConfig::default(), &[]);
    assert!(!results.student.verified);
    assert_eq!(results.student.github_id, 0);
    assert!(results.student.login.is_empty());
  }

  #[test]
  fn fmt_hints_zero() {
    assert_eq!(fmt_hints(0, 0), "0/0");
    assert_eq!(fmt_hints(5, 0), "0/0");
  }

  #[test]
  fn fmt_hints_with_data() {
    assert_eq!(fmt_hints(3, 5), "3/5 (60.0%)");
    assert_eq!(fmt_hints(0, 5), "0/5 (0.0%)");
    assert_eq!(fmt_hints(5, 5), "5/5 (100.0%)");
  }

  #[test]
  fn pct_zero_total() {
    assert_eq!(pct(0, 0), 0.0);
    assert_eq!(pct(10, 0), 0.0);
  }

  #[test]
  fn pct_normal() {
    assert!((pct(3, 4) - 75.0).abs() < f64::EPSILON);
    assert!((pct(0, 5) - 0.0).abs() < f64::EPSILON);
    assert!((pct(5, 5) - 100.0).abs() < f64::EPSILON);
  }

  #[test]
  fn report_carries_bound_owner_for_grading() {
    // A teacher reads a student's stats; the owner must surface so a swapped
    // file (bound to a different account) is detectable.
    let mut cfg = ProjectConfig {
      owner: Some(crate::identity::GithubIdentity {
        id: 4242,
        login: "student-alice".to_string(),
      }),
      ..Default::default()
    };
    cfg.update_score("01-rust/01-hello", 1.0, 0.7);

    let exercises = vec![make_exercise("01-rust/01-hello", "01-rust")];
    let report = compute(&cfg, &exercises);

    let owner = report.owner.expect("owner should be reported");
    assert_eq!(owner.id, 4242);
    assert_eq!(owner.login, "student-alice");
  }

  #[test]
  fn report_owner_is_none_when_unbound() {
    let cfg = ProjectConfig::default();
    let report = compute(&cfg, &[]);
    assert!(report.owner.is_none());
  }
}
