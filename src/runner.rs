//! Exercise verification runners and file-change watcher.
//!
//! This module dispatches exercises to language-specific runners (Rust, RISC-V
//! assembly, Python, Markdown/text) and provides an [`ExerciseWatcher`] that
//! signals file changes via an `mpsc` channel so the application layer can
//! re-run [`verify`] at its own pace.
//!
//! # Design invariants
//!
//! * **No panics** — every runner catches errors and returns
//!   [`VerificationResult`] with `score == 0.0` and a human-readable `output`.
//! * All child processes are run via [`std::process::Command::output`], which
//!   captures both stdout and stderr.  A 30-second timeout is not enforced at
//!   the process level today; callers that need a bound should wrap the call in
//!   a thread with a deadline.
//! * Temporary artefacts (`.lq_test`, `.lq_main.o`, `.lq_main`) are cleaned up
//!   on a best-effort basis — cleanup failures are silently ignored.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc;

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use regex::RegexBuilder;

use crate::config::{GoConfig, ProjectConfig, PythonConfig, RipesConfig, RustConfig};
use crate::exercise::{Exercise, Language};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default maximum number of output lines retained by [`cap_output`].
const MAX_OUTPUT_LINES: usize = 200;

// ---------------------------------------------------------------------------
// VerificationResult
// ---------------------------------------------------------------------------

/// Outcome of verifying a single exercise.
#[derive(Debug, Clone)]
pub struct VerificationResult {
  /// Fractional score in the range `0.0..=1.0`.
  pub score: f64,
  /// Number of tests / checks that passed.
  pub passed: usize,
  /// Total number of tests / checks.
  pub total: usize,
  /// Combined stdout + stderr output, capped to the last [`MAX_OUTPUT_LINES`] lines.
  pub output: String,
  /// Language-specific pass threshold for this exercise.
  pub threshold: f64,
}

impl VerificationResult {
  /// Build a zero-score result carrying only a diagnostic message.
  fn zero(output: String, threshold: f64) -> Self {
    Self {
      score: 0.0,
      passed: 0,
      total: 0,
      output,
      threshold,
    }
  }

  /// Render a fixed-width ASCII progress bar with score summary.
  ///
  /// `width` controls the number of fill characters inside the brackets.
  ///
  /// # Example
  ///
  /// ```text
  /// Score: [========----]  4/5 checks  (threshold: 80%)
  /// ```
  pub fn progress_bar(&self, width: usize) -> String {
    let filled = if self.total > 0 { (self.score * width as f64).round() as usize } else { 0 }.min(width);
    let empty = width.saturating_sub(filled);
    let threshold_pct = (self.threshold * 100.0).round() as usize;
    format!(
      "Score: [{}{}]  {}/{} checks  (threshold: {}%)",
      "=".repeat(filled),
      "-".repeat(empty),
      self.passed,
      self.total,
      threshold_pct,
    )
  }
}

// ---------------------------------------------------------------------------
// Output helpers
// ---------------------------------------------------------------------------

/// Keep only the last `max_lines` lines of `raw`.
///
/// If `raw` already has fewer than `max_lines` lines the string is returned
/// unchanged (though re-allocated).
pub fn cap_output(raw: &str, max_lines: usize) -> String {
  let lines: Vec<&str> = raw.lines().collect();
  if lines.len() <= max_lines {
    raw.to_string()
  } else {
    lines[lines.len() - max_lines..].join("\n")
  }
}

/// Combine the stdout and stderr byte buffers of a finished process into a
/// single UTF-8 string (lossy).
fn combined_output(output: &std::process::Output) -> String {
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);
  format!("{stdout}{stderr}")
}

// ---------------------------------------------------------------------------
// Top-level dispatch
// ---------------------------------------------------------------------------

/// Verify an exercise by dispatching to the appropriate language runner.
///
/// Never panics.  On any error (compile failure, missing tool, timeout) the
/// returned result has `score == 0.0` and the error description in `output`.
pub fn verify(exercise: &Exercise, config: &ProjectConfig) -> VerificationResult {
  match exercise.language {
    Language::Rust => verify_rust(exercise, &config.rust),
    Language::Riscv => verify_riscv(exercise, &config.ripes),
    Language::Python => verify_python(exercise, &config.python),
    Language::Go => verify_go(exercise, &config.go),
    Language::Text => verify_markdown(exercise),
  }
}

// ---------------------------------------------------------------------------
// Rust runner
// ---------------------------------------------------------------------------

/// Parse the `[rust] cmd` template and return `(binary, args)`.
///
/// Substitutes `<file>` with the absolute source path and `<out>` with the
/// path for the compiled test binary.
fn build_rust_command(cfg: &RustConfig, src_path: &Path, out_path: &Path) -> Result<(PathBuf, Vec<String>), String> {
  let tokens: Vec<&str> = cfg.cmd.split_whitespace().collect();
  if tokens.is_empty() {
    return Err("rust.cmd is empty in lq.toml".to_string());
  }
  let binary = PathBuf::from(tokens[0]);
  let args: Vec<String> = tokens[1..]
    .iter()
    .map(|t| match *t {
      "<file>" => src_path.to_string_lossy().to_string(),
      "<out>" => out_path.to_string_lossy().to_string(),
      other => other.to_string(),
    })
    .collect();
  Ok((binary, args))
}

/// Compile the student source as a Rust test binary and run it.
///
/// Score is `passed / (passed + failed)` based on lines matching
/// `test … … ok` and `test … … FAILED` in the test-harness output.
fn verify_rust(exercise: &Exercise, rust_cfg: &RustConfig) -> VerificationResult {
  let threshold = exercise.language.threshold();
  let test_bin = exercise.dir.join(".lq_test");

  // --- compile --------------------------------------------------------
  let (compile_bin, compile_args) = match build_rust_command(rust_cfg, &exercise.source_path, &test_bin) {
    Ok(b) => b,
    Err(e) => return VerificationResult::zero(e, threshold),
  };

  let compile = Command::new(&compile_bin).args(&compile_args).current_dir(&exercise.dir).output();

  let compile = match compile {
    Ok(o) => o,
    Err(e) => {
      return VerificationResult::zero(format!("Failed to run {}: {e}", compile_bin.display()), threshold);
    }
  };

  if !compile.status.success() {
    let out = combined_output(&compile);
    return VerificationResult::zero(cap_output(&out, MAX_OUTPUT_LINES), threshold);
  }

  // --- run tests ------------------------------------------------------
  let run = Command::new(&test_bin).current_dir(&exercise.dir).output();

  // best-effort cleanup regardless of run outcome
  let _ = fs::remove_file(&test_bin);

  let run = match run {
    Ok(o) => o,
    Err(e) => {
      return VerificationResult::zero(format!("Failed to execute test binary: {e}"), threshold);
    }
  };

  let out = combined_output(&run);

  // --- parse test results ---------------------------------------------
  let ok_re = match regex::Regex::new(r"test .+ \.\.\. ok") {
    Ok(r) => r,
    Err(e) => {
      return VerificationResult::zero(format!("Internal regex error: {e}"), threshold);
    }
  };
  let fail_re = match regex::Regex::new(r"test .+ \.\.\. FAILED") {
    Ok(r) => r,
    Err(e) => {
      return VerificationResult::zero(format!("Internal regex error: {e}"), threshold);
    }
  };

  let passed = ok_re.find_iter(&out).count();
  let failed = fail_re.find_iter(&out).count();
  let total = passed + failed;
  let score = if total > 0 { passed as f64 / total as f64 } else { 0.0 };

  VerificationResult {
    score,
    passed,
    total,
    output: cap_output(&out, MAX_OUTPUT_LINES),
    threshold,
  }
}

// ---------------------------------------------------------------------------
// RISC-V assembly runner  (riscv64-linux-gnu-as + riscv64-linux-gnu-ld)
// ---------------------------------------------------------------------------

/// Directives parsed from assembly source comment lines.
struct AsmDirectives {
  /// `; EXPECT_REG: <name> <value>` — register name → expected 32-bit value.
  expected_regs: Vec<(String, i64)>,
}

// ---------------------------------------------------------------------------
// RISC-V directive parsing helpers
// ---------------------------------------------------------------------------

/// Parse a register value that may be decimal (`42`, `-1`) or hexadecimal
/// (`0xFF`, `0x0000 0001`).  Spaces inside a hex literal are stripped so that
/// formatted hex like `0x0000 0001` is accepted.
fn parse_reg_value(s: &str) -> Option<i64> {
  let s = s.trim();
  let lower = s.to_lowercase();
  if lower.starts_with("0x") {
    // Strip all internal whitespace so "0x0000 0001" → "0x00000001"
    let clean: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    let hex = &clean[2..]; // strip "0x"
    // Parse as u64 first to handle full 32-bit unsigned values, then cast
    u64::from_str_radix(hex, 16).ok().map(|v| v as i64)
  } else {
    s.parse::<i64>().ok()
  }
}

/// Scan the source text for `; EXPECT_REG: <name> <value>` directives.
///
/// Each directive names one RISC-V register (`x0`–`x31` or ABI aliases) and
/// its expected value after the program finishes.  Values may be decimal or
/// hexadecimal (optionally with spaces inside the hex literal).
///
/// # Examples
///
/// ```asm
/// ; EXPECT_REG: x18 1
/// ; EXPECT_REG: x26 0x22
/// ; EXPECT_REG: x5  0x0000 0022
/// ```
fn parse_asm_directives(source: &str) -> AsmDirectives {
  let mut expected_regs: Vec<(String, i64)> = Vec::new();

  for line in source.lines() {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("# EXPECT_REG:").or_else(|| trimmed.strip_prefix("; EXPECT_REG:")) {
      let rest = rest.trim();
      // First whitespace-separated token is the register name; the
      // remainder (which may contain spaces for formatted hex) is the
      // value.
      if let Some(space) = rest.find(|c: char| c.is_whitespace()) {
        let reg = rest[..space].trim().to_string();
        let val_raw = rest[space + 1..].trim();
        // Strip any inline comment that follows the value.
        // Both `#` and `;` are treated as comment starters.
        // This handles lines like: `# EXPECT_REG: x5 10    # t0 = 10`
        let val_str = val_raw
          .find('#')
          .map(|i| val_raw[..i].trim())
          .or_else(|| val_raw.find(';').map(|i| val_raw[..i].trim()))
          .unwrap_or(val_raw);
        if let Some(val) = parse_reg_value(val_str) {
          expected_regs.push((reg, val));
        }
      }
    }
  }

  AsmDirectives { expected_regs }
}

// ---------------------------------------------------------------------------
// Ripes binary discovery
// ---------------------------------------------------------------------------

/// Attempt to locate the bundled Ripes binary relative to `base` (typically
/// the directory containing the `lq` executable).
///
/// Expected layout:
/// ```text
/// <base>/ripes/macos/<Name>.app/Contents/MacOS/Ripes   (macOS)
/// <base>/ripes/linux/<Name>.AppImage                   (Linux)
/// <base>/ripes/win/Ripes.exe                           (Windows)
/// ```
fn find_bundled_ripes(base: &Path) -> Option<PathBuf> {
  let ripes_dir = base.join("ripes");

  #[cfg(target_os = "macos")]
  {
    let macos_dir = ripes_dir.join("macos");
    if let Ok(entries) = fs::read_dir(&macos_dir) {
      for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().ends_with(".app") {
          let bin = entry.path().join("Contents/MacOS/Ripes");
          if bin.exists() {
            return Some(bin);
          }
        }
      }
    }
  }

  #[cfg(target_os = "linux")]
  {
    let linux_dir = ripes_dir.join("linux");
    if let Ok(entries) = fs::read_dir(&linux_dir) {
      for entry in entries.flatten() {
        let name = entry.file_name();
        let n = name.to_string_lossy();
        if n.to_lowercase().contains("ripes") && n.ends_with(".AppImage") {
          return Some(entry.path());
        }
      }
    }
  }

  #[cfg(target_os = "windows")]
  {
    let win_exe = ripes_dir.join("win").join("Ripes.exe");
    if win_exe.exists() {
      return Some(win_exe);
    }
  }

  None
}

/// Resolve the Ripes binary path using the following priority:
///
/// 1. `$RIPES_PATH` environment variable (absolute path to the binary).
/// 2. Bundled binary next to the `lq` executable (`<exe_dir>/ripes/…`).
///
/// Returns `None` if neither location yields an existing file; the caller
/// falls back to letting the OS resolve `ripes` via `$PATH`.
pub(crate) fn find_ripes_binary() -> Option<PathBuf> {
  // 1. Environment override
  if let Ok(p) = std::env::var("RIPES_PATH") {
    let p = PathBuf::from(p);
    if p.exists() {
      return Some(p);
    }
  }

  // 2. Walk up from the lq executable directory.
  //    Covers both the installed layout (<exe_dir>/ripes/…) and the
  //    development layout where the exe is buried inside
  //    target/debug/ — walking up two levels reaches the repo root
  //    where ripes/ lives.
  if let Ok(exe) = std::env::current_exe() {
    let mut dir = exe.parent().map(Path::to_path_buf);
    let mut depth = 0usize;
    while let Some(d) = dir {
      if let Some(bin) = find_bundled_ripes(&d) {
        return Some(bin);
      }
      depth += 1;
      if depth >= 4 {
        break;
      }
      dir = d.parent().map(Path::to_path_buf);
    }
  }

  // 3. CWD fallback — covers `cargo run` invoked from the repo root.
  if let Ok(cwd) = std::env::current_dir()
    && let Some(bin) = find_bundled_ripes(&cwd)
  {
    return Some(bin);
  }

  None
}

// ---------------------------------------------------------------------------
// Ripes command builder
// ---------------------------------------------------------------------------

/// Split `ripes_cfg.cmd` on whitespace, substitute `<file>` with
/// `source_file`, and resolve the binary using the following priority:
///
/// 1. `ripes_cfg.bin` — explicit path from `lq.toml` (highest priority).
/// 2. First token of `cmd` if it already looks like a path.
/// 3. Auto-discovery (`find_ripes_binary`).
/// 4. Fall back to the bare token name and let the OS resolve via `$PATH`.
///
/// Returns `(binary_path, args)`.
fn build_ripes_command(ripes_cfg: &RipesConfig, source_file: &str) -> Result<(PathBuf, Vec<String>), String> {
  let tokens: Vec<&str> = ripes_cfg.cmd.split_whitespace().collect();
  if tokens.is_empty() {
    return Err("ripes.cmd is empty in lq.toml".to_string());
  }

  let program_token = tokens[0];

  // Resolve the binary.
  let binary = if !ripes_cfg.bin.is_empty() {
    // 1. Explicit path set in lq.toml — highest priority.
    PathBuf::from(&ripes_cfg.bin)
  } else if program_token.contains(std::path::MAIN_SEPARATOR) || program_token.contains('/') {
    // 2. The cmd token already looks like a path.
    PathBuf::from(program_token)
  } else {
    // 3. Bare name — try discovery first, fall back to letting the OS find it.
    find_ripes_binary().unwrap_or_else(|| PathBuf::from(program_token))
  };

  let args: Vec<String> = tokens[1..]
    .iter()
    .map(|t| if *t == "<file>" { source_file.to_string() } else { t.to_string() })
    .collect();

  Ok((binary, args))
}

// ---------------------------------------------------------------------------
// Ripes JSON parsing
// ---------------------------------------------------------------------------

/// Extract the `registers` map from the Ripes JSON output.
///
/// The expected structure is:
/// ```json
/// { "registers": { "x0": 0, "x18": 1, … } }
/// ```
fn parse_ripes_registers(json_str: &str) -> Result<HashMap<String, i64>, String> {
  // Ripes may print "Program exited with code: N\n" before the JSON object
  // when an exit ecall is executed.  Find the first '{' to skip any such
  // prefix lines.
  let json_start = json_str.find('{').ok_or_else(|| "no JSON object found in Ripes output".to_string())?;
  let json_slice = &json_str[json_start..];

  let root: serde_json::Value = serde_json::from_str(json_slice).map_err(|e| format!("JSON parse error: {e}"))?;

  let regs = root.get("registers").ok_or_else(|| "no 'registers' key in Ripes JSON output".to_string())?;

  let obj = regs.as_object().ok_or_else(|| "'registers' is not a JSON object".to_string())?;

  let mut map = HashMap::with_capacity(obj.len());
  for (k, v) in obj {
    // Values are always integers; try signed first, then unsigned (for
    // large 32-bit values like 0xFFFF_FFFF that fit u32 but not i32).
    if let Some(n) = v.as_i64() {
      map.insert(k.clone(), n);
    } else if let Some(n) = v.as_u64() {
      map.insert(k.clone(), n as i64);
    }
  }
  Ok(map)
}

// ---------------------------------------------------------------------------
// 32-bit register equality
// ---------------------------------------------------------------------------

/// Compare two register values as 32-bit quantities (RV32 semantics).
///
/// This means `0xFFFF_FFFF` and `-1` are considered equal, matching Ripes'
/// unsigned output against signed directive values (and vice-versa).
#[inline]
fn regs_equal(actual: i64, expected: i64) -> bool {
  (actual as u32) == (expected as u32)
}

// ---------------------------------------------------------------------------
// RISC-V runner (Ripes)
// ---------------------------------------------------------------------------

/// Simulate the student's RISC-V assembly with Ripes and check every
/// `; EXPECT_REG:` directive found in the source.
///
/// Score is `satisfied / total_directives`.  If no directives are present the
/// score is `0.0` and the output explains how to add them.
fn verify_riscv(exercise: &Exercise, ripes_cfg: &RipesConfig) -> VerificationResult {
  let threshold = exercise.language.threshold();

  // --- read source for directives ------------------------------------
  let source = match fs::read_to_string(&exercise.source_path) {
    Ok(s) => s,
    Err(e) => {
      return VerificationResult::zero(format!("Failed to read source file: {e}"), threshold);
    }
  };
  let directives = parse_asm_directives(&source);

  if directives.expected_regs.is_empty() {
    return VerificationResult::zero(
      "No EXPECT_REG directives found.\n\
             Add one or more lines like the following to the top of your .asm file:\n\
             \n\
             # EXPECT_REG: x18 1\n\
             # EXPECT_REG: x26 0x22\n\
             # EXPECT_REG: x5  0x0000 0022"
        .to_string(),
      threshold,
    );
  }

  // --- build and run Ripes command -----------------------------------
  let source_str = exercise.source_path.to_string_lossy().to_string();
  let (binary, args) = match build_ripes_command(ripes_cfg, &source_str) {
    Ok(v) => v,
    Err(e) => return VerificationResult::zero(e, threshold),
  };

  let output = match Command::new(&binary).args(&args).output() {
    Ok(o) => o,
    Err(e) => {
      return VerificationResult::zero(
        format!(
          "Failed to launch Ripes ({}):\n  {e}\n\n\
                     Checked locations:\n\
                     • $RIPES_PATH environment variable\n\
                     • <exe_dir>/ripes/macos/*.app/Contents/MacOS/Ripes\n\
                     • <exe_dir>/ripes/linux/*.AppImage\n\
                     • <exe_dir>/ripes/win/Ripes.exe\n\
                     • $PATH\n\n\
                     Set $RIPES_PATH or update [ripes] cmd in lq.toml.",
          binary.display()
        ),
        threshold,
      );
    }
  };

  let stdout = String::from_utf8_lossy(&output.stdout).to_string();
  let stderr = String::from_utf8_lossy(&output.stderr).to_string();

  if !output.status.success() {
    let combined = format!("{stdout}{stderr}");
    return VerificationResult::zero(format!("Ripes exited with an error:\n{}", cap_output(&combined, MAX_OUTPUT_LINES)), threshold);
  }

  // --- parse registers from JSON -------------------------------------
  let registers = match parse_ripes_registers(&stdout) {
    Ok(r) => r,
    Err(e) => {
      return VerificationResult::zero(
        format!("Could not parse Ripes output: {e}\n\nRaw output (first 50 lines):\n{}", cap_output(&stdout, 50)),
        threshold,
      );
    }
  };

  // --- check EXPECT_REG directives -----------------------------------
  let total = directives.expected_regs.len();
  let mut satisfied: usize = 0;
  let mut report: Vec<String> = Vec::new();

  for (reg, expected) in &directives.expected_regs {
    match registers.get(reg.as_str()) {
      Some(&actual) if regs_equal(actual, *expected) => {
        satisfied += 1;
        report.push(format!("  ✓ {reg} = {expected} (0x{:08x})", *expected as u32));
      }
      Some(&actual) => {
        report.push(format!(
          "  ✗ {reg}: expected {expected} (0x{:08x}), got {actual} (0x{:08x})",
          *expected as u32, actual as u32
        ));
      }
      None => {
        report.push(format!(
          "  ✗ {reg}: expected {expected} (0x{:08x}), register not found in output",
          *expected as u32
        ));
      }
    }
  }

  if !stderr.is_empty() {
    report.push(format!("\n--- stderr ---\n{}", stderr.trim_end()));
  }

  let score = satisfied as f64 / total as f64;

  VerificationResult {
    score,
    passed: satisfied,
    total,
    output: cap_output(&report.join("\n"), MAX_OUTPUT_LINES),
    threshold,
  }
}

// ---------------------------------------------------------------------------
// Python runner
// ---------------------------------------------------------------------------

/// Parse the `[python] cmd` template and return `(binary, args)`.
///
/// Substitutes `<file>` with the absolute path to the student's source file.
fn build_python_command(cfg: &PythonConfig, src_path: &Path) -> Result<(PathBuf, Vec<String>), String> {
  let tokens: Vec<&str> = cfg.cmd.split_whitespace().collect();
  if tokens.is_empty() {
    return Err("python.cmd is empty in lq.toml".to_string());
  }
  let binary = PathBuf::from(tokens[0]);
  let args: Vec<String> = tokens[1..]
    .iter()
    .map(|t| {
      if *t == "<file>" {
        src_path.to_string_lossy().to_string()
      } else {
        t.to_string()
      }
    })
    .collect();
  Ok((binary, args))
}

/// Run the configured Python test command and score it.
///
/// Falls back to running the script directly with the configured interpreter
/// if `pytest` is absent (i.e. "No module named pytest" appears in output).
fn verify_python(exercise: &Exercise, python_cfg: &PythonConfig) -> VerificationResult {
  let threshold = exercise.language.threshold();

  let (pytest_bin, pytest_args) = match build_python_command(python_cfg, &exercise.source_path) {
    Ok(b) => b,
    Err(e) => return VerificationResult::zero(e, threshold),
  };

  // Try pytest first ---------------------------------------------------
  let pytest = Command::new(&pytest_bin).args(&pytest_args).current_dir(&exercise.dir).output();

  match pytest {
    Ok(output) => {
      let combined = combined_output(&output);

      // If pytest module itself is absent, fall back.
      if combined.contains("No module named pytest") {
        return verify_python_fallback(exercise, &pytest_bin);
      }

      parse_pytest_output(&combined, threshold)
    }
    // interpreter binary not found at all
    Err(e) => VerificationResult::zero(format!("Failed to run {}: {e}", pytest_bin.display()), threshold),
  }
}

/// Fallback: run the script directly with the configured interpreter and parse
/// unittest output.
fn verify_python_fallback(exercise: &Exercise, python_bin: &Path) -> VerificationResult {
  let threshold = exercise.language.threshold();

  let run = Command::new(python_bin).arg(&exercise.source_path).current_dir(&exercise.dir).output();

  match run {
    Ok(output) => {
      let combined = combined_output(&output);
      parse_unittest_output(&combined, threshold)
    }
    Err(e) => VerificationResult::zero(format!("Failed to run {}: {e}", python_bin.display()), threshold),
  }
}

// ---------------------------------------------------------------------------
// Go runner
// ---------------------------------------------------------------------------

/// Parse the `[go] cmd` template and return `(binary, args)`.
///
/// No `<file>` substitution is performed; Go tests are addressed by package
/// (`.`), so the command runs directly in the exercise directory.
fn build_go_command(cfg: &GoConfig) -> Result<(PathBuf, Vec<String>), String> {
  let tokens: Vec<&str> = cfg.cmd.split_whitespace().collect();
  if tokens.is_empty() {
    return Err("go.cmd is empty in lq.toml".to_string());
  }
  let binary = PathBuf::from(tokens[0]);
  let args: Vec<String> = tokens[1..].iter().map(|t| t.to_string()).collect();
  Ok((binary, args))
}

/// Run the configured Go test command in the exercise directory and score it.
///
/// Score is `passed / (passed + failed)` based on `--- PASS:` and
/// `--- FAIL:` lines in the verbose test output.
fn verify_go(exercise: &Exercise, go_cfg: &GoConfig) -> VerificationResult {
  let threshold = exercise.language.threshold();

  let (go_bin, go_args) = match build_go_command(go_cfg) {
    Ok(b) => b,
    Err(e) => return VerificationResult::zero(e, threshold),
  };

  let output = Command::new(&go_bin).args(&go_args).current_dir(&exercise.dir).output();

  match output {
    Err(e) => VerificationResult::zero(
      format!(
        "Failed to run '{}': {e}\n\
                 Make sure Go is installed and available in $PATH.",
        go_bin.display()
      ),
      threshold,
    ),
    Ok(out) => {
      let combined = combined_output(&out);
      parse_go_test_output(&combined, threshold)
    }
  }
}

/// Parse `go test -v` output, counting `--- PASS:` and `--- FAIL:` lines.
///
/// Example output fragment:
/// ```text
/// --- PASS: TestGreeting (0.00s)
/// --- FAIL: TestGreetingStartsWithHello (0.00s)
/// ```
fn parse_go_test_output(output: &str, threshold: f64) -> VerificationResult {
  let pass_re = match regex::Regex::new(r"--- PASS: \S+") {
    Ok(r) => r,
    Err(e) => {
      return VerificationResult::zero(format!("Internal regex error: {e}"), threshold);
    }
  };
  let fail_re = match regex::Regex::new(r"--- FAIL: \S+") {
    Ok(r) => r,
    Err(e) => {
      return VerificationResult::zero(format!("Internal regex error: {e}"), threshold);
    }
  };

  let passed = pass_re.find_iter(output).count();
  let failed = fail_re.find_iter(output).count();
  let total = passed + failed;
  let score = if total > 0 { passed as f64 / total as f64 } else { 0.0 };

  VerificationResult {
    score,
    passed,
    total,
    output: cap_output(output, MAX_OUTPUT_LINES),
    threshold,
  }
}

/// Parse a pytest summary line such as `2 passed, 1 failed in 0.03s`.
fn parse_pytest_output(output: &str, threshold: f64) -> VerificationResult {
  let passed_re = match regex::Regex::new(r"(\d+) passed") {
    Ok(r) => r,
    Err(e) => {
      return VerificationResult::zero(format!("Regex error: {e}"), threshold);
    }
  };
  let failed_re = match regex::Regex::new(r"(\d+) failed") {
    Ok(r) => r,
    Err(e) => {
      return VerificationResult::zero(format!("Regex error: {e}"), threshold);
    }
  };

  let passed: usize = passed_re
    .captures(output)
    .and_then(|c| c.get(1))
    .and_then(|m| m.as_str().parse().ok())
    .unwrap_or(0);

  let failed: usize = failed_re
    .captures(output)
    .and_then(|c| c.get(1))
    .and_then(|m| m.as_str().parse().ok())
    .unwrap_or(0);

  let total = passed + failed;
  let score = if total > 0 { passed as f64 / total as f64 } else { 0.0 };

  VerificationResult {
    score,
    passed,
    total,
    output: cap_output(output, MAX_OUTPUT_LINES),
    threshold,
  }
}

/// Parse Python `unittest` output (`Ran N tests`, `OK` / `FAILED (failures=N)`).
fn parse_unittest_output(output: &str, threshold: f64) -> VerificationResult {
  let ran_re = match regex::Regex::new(r"Ran (\d+) test") {
    Ok(r) => r,
    Err(e) => {
      return VerificationResult::zero(format!("Regex error: {e}"), threshold);
    }
  };
  let failures_re = match regex::Regex::new(r"FAILED \(failures=(\d+)") {
    Ok(r) => r,
    Err(e) => {
      return VerificationResult::zero(format!("Regex error: {e}"), threshold);
    }
  };

  let total: usize = ran_re
    .captures(output)
    .and_then(|c| c.get(1))
    .and_then(|m| m.as_str().parse().ok())
    .unwrap_or(0);

  if total == 0 {
    return VerificationResult {
      score: 0.0,
      passed: 0,
      total: 0,
      output: cap_output(output, MAX_OUTPUT_LINES),
      threshold,
    };
  }

  let failed: usize = failures_re
    .captures(output)
    .and_then(|c| c.get(1))
    .and_then(|m| m.as_str().parse().ok())
    .unwrap_or(0);

  let passed = total.saturating_sub(failed);
  let score = passed as f64 / total as f64;

  VerificationResult {
    score,
    passed,
    total,
    output: cap_output(output, MAX_OUTPUT_LINES),
    threshold,
  }
}

// ---------------------------------------------------------------------------
// Markdown / text runner
// ---------------------------------------------------------------------------

/// Check the student's markdown against keyword patterns from the solution
/// data, using case-insensitive regex matching.
///
/// Score is `matched / total_keywords`.  Unmatched keywords are listed in
/// the output as gap indicators.
fn verify_markdown(exercise: &Exercise) -> VerificationResult {
  let threshold = exercise.language.threshold();

  let solution_data = match &exercise.solution_data {
    Some(data) => data,
    None => {
      return VerificationResult::zero("No solution data found for keyword verification.".to_string(), threshold);
    }
  };

  let keywords = &solution_data.keywords;
  if keywords.is_empty() {
    return VerificationResult::zero("No keywords defined in solution data.".to_string(), threshold);
  }

  let content = match fs::read_to_string(&exercise.source_path) {
    Ok(s) => s,
    Err(e) => {
      return VerificationResult::zero(format!("Failed to read source file: {e}"), threshold);
    }
  };

  let mut matched: usize = 0;
  let mut unmatched: Vec<&str> = Vec::new();

  for kw in keywords {
    let found = match RegexBuilder::new(kw).case_insensitive(true).build() {
      Ok(re) => re.is_match(&content),
      // If the keyword is not a valid regex pattern, fall back to a
      // plain case-insensitive substring search.
      Err(_) => content.to_lowercase().contains(&kw.to_lowercase()),
    };

    if found {
      matched += 1;
    } else {
      unmatched.push(kw.as_str());
    }
  }

  let total = keywords.len();
  let score = matched as f64 / total as f64;

  let mut output = format!("{matched}/{total} keywords matched.");
  if !unmatched.is_empty() {
    output.push_str("\n\nMissing concepts:");
    for kw in &unmatched {
      output.push_str(&format!("\n  • {kw}"));
    }
  }

  VerificationResult {
    score,
    passed: matched,
    total,
    output,
    threshold,
  }
}

// ---------------------------------------------------------------------------
// ExerciseWatcher
// ---------------------------------------------------------------------------

/// Watches an exercise source file for changes and sends a `()` signal
/// through [`event_rx`](Self::event_rx) each time a create or modify event
/// is detected.
///
/// The application layer is responsible for calling [`verify`] when a signal
/// arrives — the watcher itself performs no verification work.
pub struct ExerciseWatcher {
  /// Held to keep the underlying OS watcher alive.  Dropped when the
  /// struct is dropped, which stops watching.
  _watcher: RecommendedWatcher,
  /// Receives `()` each time the watched file is created or modified.
  pub event_rx: mpsc::Receiver<()>,
}

impl ExerciseWatcher {
  /// Begin watching `source_path` (a single file) for create / modify
  /// events.
  ///
  /// # Errors
  ///
  /// Returns an error if the underlying OS watcher cannot be created or if
  /// the path cannot be watched (e.g. it does not exist).
  pub fn new(source_path: &Path) -> anyhow::Result<Self> {
    let (tx, rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(
      move |res: Result<notify::Event, notify::Error>| {
        if let Ok(event) = res
          && matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_))
        {
          // Ignore send errors — the receiver may have been
          // dropped if the app is shutting down.
          let _ = tx.send(());
        }
      },
      Config::default(),
    )?;

    watcher.watch(source_path, RecursiveMode::NonRecursive)?;

    Ok(Self {
      _watcher: watcher,
      event_rx: rx,
    })
  }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;

  // -- cap_output ------------------------------------------------------

  #[test]
  fn cap_output_short_input_unchanged() {
    let input = "line1\nline2\nline3";
    assert_eq!(cap_output(input, 5), input.to_string());
  }

  #[test]
  fn cap_output_trims_to_last_n_lines() {
    let input = "a\nb\nc\nd\ne";
    let result = cap_output(input, 3);
    assert_eq!(result, "c\nd\ne");
  }

  #[test]
  fn cap_output_empty_string() {
    assert_eq!(cap_output("", 10), "".to_string());
  }

  #[test]
  fn cap_output_exact_boundary() {
    let input = "1\n2\n3";
    assert_eq!(cap_output(input, 3), input.to_string());
  }

  // -- VerificationResult::zero ----------------------------------------

  #[test]
  fn zero_result_has_correct_defaults() {
    let r = VerificationResult::zero("oops".into(), 0.8);
    assert_eq!(r.score, 0.0);
    assert_eq!(r.passed, 0);
    assert_eq!(r.total, 0);
    assert_eq!(r.output, "oops");
    assert_eq!(r.threshold, 0.8);
  }

  // -- progress_bar ----------------------------------------------------

  #[test]
  fn progress_bar_full_score() {
    let r = VerificationResult {
      score: 1.0,
      passed: 5,
      total: 5,
      output: String::new(),
      threshold: 1.0,
    };
    let bar = r.progress_bar(10);
    assert!(bar.contains("[==========]"));
    assert!(bar.contains("5/5 checks"));
    assert!(bar.contains("threshold: 100%"));
  }

  #[test]
  fn progress_bar_zero_score() {
    let r = VerificationResult {
      score: 0.0,
      passed: 0,
      total: 4,
      output: String::new(),
      threshold: 0.75,
    };
    let bar = r.progress_bar(8);
    assert!(bar.contains("[--------]"));
    assert!(bar.contains("0/4 checks"));
    assert!(bar.contains("threshold: 75%"));
  }

  #[test]
  fn progress_bar_partial_score() {
    let r = VerificationResult {
      score: 0.5,
      passed: 2,
      total: 4,
      output: String::new(),
      threshold: 0.8,
    };
    let bar = r.progress_bar(10);
    // 0.5 * 10 = 5.0 → 5 filled
    assert!(bar.contains("[=====-----]"));
    assert!(bar.contains("2/4 checks"));
    assert!(bar.contains("threshold: 80%"));
  }

  #[test]
  fn progress_bar_no_checks() {
    let r = VerificationResult {
      score: 0.0,
      passed: 0,
      total: 0,
      output: String::new(),
      threshold: 1.0,
    };
    let bar = r.progress_bar(6);
    assert!(bar.contains("[------]"));
    assert!(bar.contains("0/0 checks"));
  }

  // -- parse_asm_directives --------------------------------------------

  #[test]
  fn parse_asm_directives_empty() {
    let d = parse_asm_directives("section .text\n  addi x0, x0, 0\n");
    assert!(d.expected_regs.is_empty());
  }

  #[test]
  fn parse_asm_directives_decimal() {
    let src = "\
; EXPECT_REG: x18 1
; EXPECT_REG: x19 2
addi s2, s0, 1
";
    let d = parse_asm_directives(src);
    assert_eq!(d.expected_regs, vec![("x18".to_string(), 1), ("x19".to_string(), 2),]);
  }

  #[test]
  fn parse_asm_directives_inline_hash_comment() {
    // Values followed by an inline `# comment` must still parse correctly.
    let src = "\
# EXPECT_REG: x5  10    # t0 = 10
# EXPECT_REG: x6  32    # t1 = 32
# EXPECT_REG: x7  42    # t2 = t0 + t1
";
    let d = parse_asm_directives(src);
    assert_eq!(d.expected_regs, vec![("x5".to_string(), 10), ("x6".to_string(), 32), ("x7".to_string(), 42),]);
  }

  #[test]
  fn parse_asm_directives_inline_semicolon_comment() {
    // Values followed by an inline `; comment` must still parse correctly.
    let src = "; EXPECT_REG: x18 55   ; sum of 1..10\n";
    let d = parse_asm_directives(src);
    assert_eq!(d.expected_regs, vec![("x18".to_string(), 55)]);
  }

  #[test]
  fn parse_asm_directives_hex_inline_comment() {
    // Hex value with inline comment — formatted hex space variant.
    let src = "# EXPECT_REG: x26 0x0000 0022    # 34 decimal\n";
    let d = parse_asm_directives(src);
    assert_eq!(d.expected_regs, vec![("x26".to_string(), 0x22)]);
  }

  #[test]
  fn parse_asm_directives_hex_compact() {
    let src = "; EXPECT_REG: x26 0x22\n";
    let d = parse_asm_directives(src);
    assert_eq!(d.expected_regs, vec![("x26".to_string(), 0x22)]);
  }

  #[test]
  fn parse_asm_directives_hex_with_spaces() {
    // "0x0000 0001" — formatted hex with an internal space
    let src = "; EXPECT_REG: x18 0x0000 0001\n";
    let d = parse_asm_directives(src);
    assert_eq!(d.expected_regs, vec![("x18".to_string(), 1)]);
  }

  #[test]
  fn parse_asm_directives_negative() {
    let src = "; EXPECT_REG: x5 -1\n";
    let d = parse_asm_directives(src);
    assert_eq!(d.expected_regs, vec![("x5".to_string(), -1)]);
  }

  #[test]
  fn parse_reg_value_decimal() {
    assert_eq!(parse_reg_value("42"), Some(42));
    assert_eq!(parse_reg_value("-1"), Some(-1));
    assert_eq!(parse_reg_value("0"), Some(0));
  }

  #[test]
  fn parse_reg_value_hex() {
    assert_eq!(parse_reg_value("0x0"), Some(0));
    assert_eq!(parse_reg_value("0xFF"), Some(255));
    assert_eq!(parse_reg_value("0xFFFFFFFF"), Some(0xFFFF_FFFF_u64 as i64));
    assert_eq!(parse_reg_value("0x0000 0022"), Some(0x22));
    assert_eq!(parse_reg_value("0x0000 0001"), Some(1));
  }

  #[test]
  fn regs_equal_handles_32bit_wrapping() {
    // 0xFFFF_FFFF and -1 are the same 32-bit value
    assert!(regs_equal(0xFFFF_FFFF_u64 as i64, -1));
    assert!(regs_equal(-1, 0xFFFF_FFFF_u64 as i64));
    assert!(regs_equal(1, 1));
    assert!(!regs_equal(1, 2));
  }

  // -- parse_pytest_output ---------------------------------------------

  #[test]
  fn parse_pytest_all_passed() {
    let out = "3 passed in 0.02s\n";
    let r = parse_pytest_output(out, 1.0);
    assert_eq!(r.passed, 3);
    assert_eq!(r.total, 3);
    assert!((r.score - 1.0).abs() < f64::EPSILON);
  }

  #[test]
  fn parse_pytest_mixed() {
    let out = "2 passed, 1 failed in 0.05s\n";
    let r = parse_pytest_output(out, 1.0);
    assert_eq!(r.passed, 2);
    assert_eq!(r.total, 3);
    assert!((r.score - 2.0 / 3.0).abs() < 1e-9);
  }

  #[test]
  fn parse_pytest_no_match() {
    let out = "some random output\n";
    let r = parse_pytest_output(out, 1.0);
    assert_eq!(r.total, 0);
    assert_eq!(r.score, 0.0);
  }

  // -- parse_unittest_output -------------------------------------------

  #[test]
  fn parse_unittest_all_ok() {
    let out = "Ran 4 tests in 0.001s\n\nOK\n";
    let r = parse_unittest_output(out, 1.0);
    assert_eq!(r.passed, 4);
    assert_eq!(r.total, 4);
    assert!((r.score - 1.0).abs() < f64::EPSILON);
  }

  #[test]
  fn parse_unittest_some_failures() {
    let out = "Ran 5 tests in 0.002s\n\nFAILED (failures=2)\n";
    let r = parse_unittest_output(out, 1.0);
    assert_eq!(r.passed, 3);
    assert_eq!(r.total, 5);
    assert!((r.score - 0.6).abs() < 1e-9);
  }

  #[test]
  fn parse_unittest_no_tests() {
    let out = "nothing here\n";
    let r = parse_unittest_output(out, 1.0);
    assert_eq!(r.total, 0);
    assert_eq!(r.score, 0.0);
  }

  // -- combined_output -------------------------------------------------

  #[test]
  fn combined_output_merges_streams() {
    // We can only test the helper with a real Output struct from a
    // command — use a trivial echo.
    let out = Command::new("sh").args(["-c", "echo hello; echo err >&2"]).output();
    if let Ok(o) = out {
      let combined = combined_output(&o);
      assert!(combined.contains("hello"));
      assert!(combined.contains("err"));
    }
  }
}
