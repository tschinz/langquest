#![deny(clippy::all)]

use std::io::{BufRead, Write};
use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use lq::{app, config, exercise, stats};

/// CLI definition for the `lq` binary.
#[derive(Parser)]
#[command(name = "lq", version, about = "LangQuest - interactive programming exercises")]
struct Cli {
  /// Path to exercise repository root
  #[arg(long, global = true)]
  repo: Option<PathBuf>,

  /// Wipe all progress in lq.toml and start fresh
  #[arg(long)]
  reset: bool,

  /// Display detailed statistics about exercise progress
  #[arg(short = 's', long)]
  stats: bool,

  #[command(subcommand)]
  command: Option<Command>,
}

/// Subcommands available in `lq`.
#[derive(Subcommand)]
enum Command {
  /// Print current exercise and overall progress
  Status,
  /// Encrypt every `solution/` file in place (teacher → student repo, for CI)
  SealSolutions,
}

fn main() -> Result<()> {
  let cli = Cli::parse();

  if cli.reset {
    return handle_reset(cli.repo);
  }

  if cli.stats {
    return handle_stats(cli.repo);
  }

  match cli.command {
    Some(Command::Status) => handle_status(cli.repo),
    Some(Command::SealSolutions) => handle_seal_solutions(cli.repo),
    None => handle_default(cli.repo),
  }
}

/// Handle `seal-solutions`: encrypt every file under each `solution/` directory
/// in the repo, in place. There is deliberately no CLI command to reverse this
/// (students must not be able to bulk-decrypt solutions); the teacher's source
/// repo is the plaintext of record.
fn handle_seal_solutions(repo: Option<PathBuf>) -> Result<()> {
  let repo_path = config::resolve_repo_path(repo.as_deref());
  let count = lq::solutions::seal_solutions_in(&repo_path)?;
  println!("Sealed {count} solution file(s) in {}", repo_path.display());
  Ok(())
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
  // Tolerate a corrupt progress file here: reset wipes it anyway, so a student
  // with a damaged file must still be able to recover.
  let mut cfg = config::ProjectConfig::load_lenient(&cfg_path)?;

  // Only the bound owner may reset their progress.
  let owner = lq::identity::authorize(&repo_path, cfg.owner.clone()).map_err(|reason| anyhow::anyhow!("progress locked: {reason}"))?;
  cfg.owner = Some(owner);

  let (_tree, all_exercises, _errors) = exercise::discover_exercises(&repo_path);

  let first_exercise = all_exercises.first().map(|e| e.relative_path.as_str());

  cfg.reset(first_exercise);
  cfg.save(&cfg_path)?;

  Ok(())
}

/// Handle the `status` subcommand.
///
/// `status` and `-s`/`--stats` show the same complete report; `status` is a
/// convenient alias.
fn handle_status(repo: Option<PathBuf>) -> Result<()> {
  handle_stats(repo)
}

/// Handle the `stats` subcommand / `-s` flag: print the complete status report
/// and write the machine-readable `results.toml`.
fn handle_stats(repo: Option<PathBuf>) -> Result<()> {
  let repo_path = config::resolve_repo_path(repo.as_deref());
  stats::run(&repo_path)
}

/// Default handler (no subcommand, no `--reset`): launch the TUI.
fn handle_default(repo: Option<PathBuf>) -> Result<()> {
  let repo_path = config::resolve_repo_path(repo.as_deref());
  eprintln!("   Repository: {}", repo_path.display());

  eprint!("   Loading exercises…");
  std::io::stderr().flush()?;
  let mut application = app::App::new(repo_path)?;

  let total = application.exercises.len();
  eprintln!(" found {total} exercise(s) across {} module(s).", application.tree.len());
  eprintln!("   Entering TUI (press q to quit)…");

  application.run()?;

  eprintln!("   Goodbye!");
  Ok(())
}
