//! Statistics module for the `lq stats` subcommand.
//!
//! Reads the project config (`lq.toml`) and exercise tree to produce a
//! detailed progress report.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::Result;

use crate::config::{self, ProjectConfig};
use crate::exercise;

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
}

/// Aggregated statistics report.
#[derive(Debug, Default, PartialEq)]
pub struct Report {
  pub total_exercises: usize,
  pub completed: usize,
  pub solutions_seen: usize,
  pub hints_shown: usize,
  pub hints_max_sum: usize,
  pub hints_total_sum: usize,
  pub best_score_sum: f64,
  pub by_module: BTreeMap<String, PerModuleStats>,
}

/// Run the `stats` subcommand: load config, discover exercises, compute and
/// print statistics.
pub fn run(repo_path: &Path) -> Result<()> {
  let cfg_path = config::config_path(repo_path);
  let cfg = ProjectConfig::load(&cfg_path)?;
  let (_tree, all_exercises, _errors) = exercise::discover_exercises(repo_path);

  let report = compute(&cfg, &all_exercises);
  render(&report);

  Ok(())
}

/// Compute aggregated statistics from the project config and exercise list.
fn compute(cfg: &ProjectConfig, all_exercises: &[exercise::Exercise]) -> Report {
  let mut report = Report::default();

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

    // Parse hints_max (format: "{revealed}/{total}")
    if let Some((revealed_str, total_str)) = state.hints_max.split_once('/')
      && let (Ok(r), Ok(t)) = (revealed_str.parse::<usize>(), total_str.parse::<usize>())
    {
      report.hints_max_sum += r;
      report.hints_total_sum += t;
      stats.hints_max_sum += r;
      stats.hints_total_sum += t;
    }

    report.best_score_sum += state.best_score;
    stats.best_score_sum += state.best_score;
  }

  report
}

/// Render a statistics report to stdout.
fn render(report: &Report) {
  let total = report.total_exercises;

  println!("═══════════════════════════════════════");
  println!("         LangQuest Stats");
  println!("═══════════════════════════════════════");

  // Overall summary
  println!();
  println!("  Overall");
  println!("  ───────");
  println!("  Total exercises:      {total}");
  println!("  Completed (passed):   {} ({:.1}%)", report.completed, pct(report.completed, total));
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
      solution_data: None,
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
  fn parses_hints_max() {
    let mut cfg = ProjectConfig::default();

    // Manually set hints_max since record_hint_reveal also increments hints_shown
    let state = cfg.exercises.entry("ex".to_string()).or_default();
    state.hints_max = "3/5".to_string();

    let exercises = vec![make_exercise("ex", "01-rust")];
    let report = compute(&cfg, &exercises);

    assert_eq!(report.hints_max_sum, 3);
    assert_eq!(report.hints_total_sum, 5);
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
}
