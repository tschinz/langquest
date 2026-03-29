#![deny(clippy::all)]

use std::io::{BufRead, Write};
use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use lq::{app, config, exercise};

/// CLI definition for the `lq` binary.
#[derive(Parser)]
#[command(name = "lq", version, about = "LangQuest — interactive programming exercises")]
struct Cli {
  /// Path to exercise repository root
  #[arg(long)]
  repo: Option<PathBuf>,

  /// Wipe all progress in lq.toml and start fresh
  #[arg(long)]
  reset: bool,

  #[command(subcommand)]
  command: Option<Command>,
}

/// Subcommands available in `lq`.
#[derive(Subcommand)]
enum Command {
  /// Print current exercise and overall progress
  Status,
}

fn main() -> Result<()> {
  let cli = Cli::parse();

  if cli.reset {
    return handle_reset(cli.repo);
  }

  match cli.command {
    Some(Command::Status) => handle_status(cli.repo),
    None => handle_default(cli.repo),
  }
}

/// Handle the `--reset` flag: wipe all progress after user confirmation.
fn handle_reset(repo: Option<PathBuf>) -> Result<()> {
  let repo_path = config::resolve_repo_path(repo.as_deref());

  eprintln!("[!] This will delete all progress in lq.toml. This cannot be undone.");
  eprint!("    Type \"yes\" to confirm, or anything else to cancel: ");
  std::io::stderr().flush()?;

  let mut input = String::new();
  std::io::stdin().lock().read_line(&mut input)?;

  if input.trim() != "yes" {
    eprintln!("Cancelled.");
    return Ok(());
  }

  let cfg_path = config::config_path(&repo_path);
  let mut cfg = config::ProjectConfig::load(&cfg_path)?;
  let (modules, _errors) = exercise::discover_exercises(&repo_path);

  let first_exercise = modules.first().and_then(|m| m.exercises.first()).map(|e| e.relative_path.as_str());

  cfg.reset(first_exercise);
  cfg.save(&cfg_path)?;

  Ok(())
}

/// Handle the `status` subcommand: print current exercise and progress.
fn handle_status(repo: Option<PathBuf>) -> Result<()> {
  let repo_path = config::resolve_repo_path(repo.as_deref());
  let cfg_path = config::config_path(&repo_path);
  let cfg = config::ProjectConfig::load(&cfg_path)?;
  let (modules, _errors) = exercise::discover_exercises(&repo_path);

  match &cfg.current_exercise {
    Some(name) => println!("Current exercise: {name}"),
    None => println!("No current exercise set."),
  }

  let total: usize = modules.iter().map(|m| m.exercises.len()).sum();
  let completed = modules
    .iter()
    .flat_map(|m| &m.exercises)
    .filter(|e| cfg.get_state(&e.relative_path).passed)
    .count();

  println!("{completed}/{total} exercises completed");

  Ok(())
}

/// Default handler (no subcommand, no `--reset`): launch the TUI.
fn handle_default(repo: Option<PathBuf>) -> Result<()> {
  let repo_path = config::resolve_repo_path(repo.as_deref());
  let mut application = app::App::new(repo_path)?;
  application.run()
}
