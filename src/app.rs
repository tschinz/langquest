//! Top-level application state and TUI event loop.

use std::collections::HashSet;
use std::io::BufWriter;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::Duration;

use anyhow::{Result, bail};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind};
use ratatui::prelude::*;

use crate::config::{self, ProjectConfig};
use crate::exercise::{Exercise, ExerciseStatus, TreeNode, discover_exercises};
use crate::runner::{self, ExerciseWatcher, VerificationResult};
use crate::ui;
use crate::ui::cache::RenderCache;
use crate::ui::exercise_view::{strip_code_fences, wrap_line};
use crate::ui::markdown::PendingOsc8;

/// What a tree line represents — used to map cursor position to action.
#[derive(Debug, Clone)]
pub enum LineKind {
  /// An exercise line, with its index in the flat exercises list.
  Exercise(usize),
  /// A group-header line, with the group's relative path.
  Group(String),
  /// A separator / blank line.
  Blank,
}

/// Which top-level view is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
  /// Focused on a single exercise with paged content.
  ExerciseView,
  /// Table of all exercises with optional tree panel.
  Overview,
  /// About page - project info and credits.
  About,
}

/// Pages within the Exercise View.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExercisePage {
  /// Theory / background reading.
  Theory,
  /// Task description.
  Task,
  /// Output of the exercise's `main()` for debugging (only appears when
  /// the source has a `fn main` that compiles and runs successfully).
  Debug,
  /// Verification output (test results).
  Output,
  /// Reference solution (gated).
  Solution,
}

impl ExercisePage {
  /// All pages in display order.
  const ALL: [ExercisePage; 5] = [
    ExercisePage::Theory,
    ExercisePage::Task,
    ExercisePage::Debug,
    ExercisePage::Output,
    ExercisePage::Solution,
  ];

  /// Index of this page in the page list.
  pub(crate) fn index(self) -> usize {
    match self {
      ExercisePage::Theory => 0,
      ExercisePage::Task => 1,
      ExercisePage::Debug => 2,
      ExercisePage::Output => 3,
      ExercisePage::Solution => 4,
    }
  }

  /// Create a page from its index (wrapping).
  pub(crate) fn from_index(idx: usize) -> Self {
    ExercisePage::ALL[idx % ExercisePage::ALL.len()]
  }

  /// Human-readable label for this page.
  pub(crate) fn label(self) -> &'static str {
    match self {
      ExercisePage::Theory => "Theory",
      ExercisePage::Task => "Task",
      ExercisePage::Debug => "Debug",
      ExercisePage::Output => "Output",
      ExercisePage::Solution => "Solution",
    }
  }
}

/// Background verification completion message.
struct VerifyMessage {
  request_id: u64,
  exercise_path: String,
  result: VerificationResult,
  main_output: String,
}

/// Main application state.
pub struct App {
  /// Tree of discovered groups and exercises.
  pub tree: Vec<TreeNode>,
  /// Flat list of all exercises (depth-first order, for linear navigation).
  pub exercises: Vec<Exercise>,
  /// Persisted project configuration.
  pub config: ProjectConfig,
  /// Path to the `lq.toml` config file.
  pub config_path: PathBuf,
  /// Index into the flat exercise list for the current exercise.
  pub current_index: usize,
  /// Which top-level view is displayed.
  pub view: View,
  /// Current page within the Exercise View.
  pub page: ExercisePage,
  /// Number of hints revealed so far for the current exercise.
  pub hints_revealed: usize,
  /// Most recent verification result.
  pub last_result: Option<VerificationResult>,
  /// Captured stdout/stderr from running the exercise's `main()` as a
  /// regular binary. Displayed in the "Debug" page. Empty when the source
  /// doesn't have a `fn main` or can't be compiled as a regular binary.
  pub last_main_output: String,
  /// Cursor position in the Overview table (tree line index).
  pub overview_cursor: usize,
  /// For each tree line, what it represents (exercise, group, or blank).
  /// Used by `handle_enter` to know whether to open an exercise or toggle a group.
  pub tree_line_kinds: Vec<LineKind>,
  /// Groups (by relative path) whose children are hidden in the tree.
  pub collapsed_groups: HashSet<String>,
  /// Whether the bottom status bar is expanded.
  pub show_menu: bool,
  /// Vertical scroll offset for markdown/text content.
  pub scroll_offset: usize,
  /// Last known content height (in lines) for scroll limiting.
  pub content_height: usize,
  /// Last known viewport height (in lines) for scroll limiting.
  pub viewport_height: usize,
  /// File watcher for the current exercise's source file.
  pub watcher: Option<ExerciseWatcher>,
  /// Whether the "unlock solution?" warning is awaiting a second `h` press.
  pub solution_unlock_pending: bool,
  /// Cache for parsed markdown content to avoid re-parsing every frame.
  pub render_cache: RenderCache,
  /// Track if a redraw is needed (dirty flag for optimization).
  pub needs_redraw: bool,
  /// Last terminal width to detect resize.
  last_width: u16,
  /// Whether verification is queued/running for the current exercise.
  pub verifying: bool,
  /// Monotonic request id for background verification jobs.
  verify_request_id: u64,
  /// Shared generation used by worker threads to cancel stale requests.
  verify_generation: Arc<AtomicU64>,
  /// Request id of the verification result we still care about.
  active_verify_request: Option<u64>,
  /// Sender side used to report verification completion from worker threads.
  verify_result_tx: mpsc::Sender<VerifyMessage>,
  /// Receiver side polled by the TUI loop for completed verification results.
  verify_result_rx: mpsc::Receiver<VerifyMessage>,
}

impl App {
  /// Create a new `App` by discovering exercises and loading config.
  ///
  /// # Errors
  ///
  /// Returns an error if no exercises are found or if the config cannot be
  /// loaded.
  pub fn new(repo_path: PathBuf) -> Result<Self> {
    let (tree, all_exercises, _errors) = discover_exercises(&repo_path);

    if all_exercises.is_empty() {
      bail!("no exercises found in {}", repo_path.display());
    }

    let cfg_path = config::config_path(&repo_path);
    let mut config = ProjectConfig::load(&cfg_path)?;

    // Enforce GitHub-identity binding (online, with offline attestation
    // fallback) before exposing any progress. Records the bound owner.
    let owner = crate::identity::authorize(&repo_path, config.owner.clone()).map_err(|reason| anyhow::anyhow!("progress locked: {reason}"))?;
    config.owner = Some(owner);

    // Resolve starting index from config.current_exercise.
    let current_index = config
      .current_exercise
      .as_deref()
      .and_then(|name| all_exercises.iter().position(|ex| ex.relative_path == name))
      .unwrap_or(0);

    let (verify_result_tx, verify_result_rx) = mpsc::channel();
    let verify_generation = Arc::new(AtomicU64::new(0));

    let mut app = App {
      tree,
      exercises: all_exercises,
      config,
      config_path: cfg_path,
      current_index,
      view: View::Overview,
      page: ExercisePage::Theory,
      hints_revealed: 0,
      last_result: None,
      last_main_output: String::new(),
      overview_cursor: 1,
      tree_line_kinds: Vec::new(),
      collapsed_groups: HashSet::new(),
      show_menu: true,
      scroll_offset: 0,
      content_height: 0,
      viewport_height: 0,
      watcher: None,
      solution_unlock_pending: false,
      render_cache: RenderCache::new(),
      needs_redraw: true,
      last_width: 0,
      verifying: false,
      verify_request_id: 0,
      verify_generation,
      active_verify_request: None,
      verify_result_tx,
      verify_result_rx,
    };

    // Pre-initialise hints_max for exercises with solution data so the TOML
    // shows "0/<total>" from the start rather than an empty string or "0/0".
    for exercise in &app.exercises {
      if let Some(ref sd) = exercise.solution_data {
        let total = sd.hints.len();
        if total > 0 {
          app.config.init_hints_max(&exercise.relative_path, total);
        }
      }
    }

    app.setup_watcher();
    app.queue_verify();
    app.save_config();

    Ok(app)
  }

  /// Get a reference to the exercise at `current_index`.
  pub fn current_exercise(&self) -> &Exercise {
    &self.exercises[self.current_index]
  }

  /// Get the exercise at a specific flat index.
  #[allow(dead_code)]
  pub fn exercise_at(&self, index: usize) -> &Exercise {
    &self.exercises[index]
  }

  /// Derive the current exercise's status from persisted config state.
  #[allow(dead_code)]
  pub fn current_status(&self) -> ExerciseStatus {
    self.status_at(self.current_index)
  }

  /// Derive the exercise status for any flat index.
  #[allow(dead_code)]
  pub fn status_at(&self, index: usize) -> ExerciseStatus {
    let exercise = self.exercise_at(index);
    let state = self.config.get_state(&exercise.relative_path);
    if state.passed { ExerciseStatus::Complete } else { ExerciseStatus::Failing }
  }

  /// Switch to a new exercise by index, updating all related state.
  fn switch_exercise(&mut self, new_index: usize) {
    if new_index >= self.exercises.len() {
      return;
    }
    self.current_index = new_index;
    let exercise = self.current_exercise();
    self.config.current_exercise = Some(exercise.relative_path.clone());
    self.hints_revealed = 0;
    self.solution_unlock_pending = false;
    self.last_main_output.clear();
    self.page = ExercisePage::Theory;
    self.scroll_offset = 0;
    self.setup_watcher();
    self.queue_verify();
    self.save_config();
  }

  /// Save config to disk, ignoring errors (best-effort).
  fn save_config(&self) {
    let _ = self.config.save(&self.config_path);
  }

  /// Create an `ExerciseWatcher` for the current exercise's source file.
  fn setup_watcher(&mut self) {
    let source = self.current_exercise().source_path.clone();
    self.watcher = ExerciseWatcher::new(&source).ok();
  }

  /// Queue verification in a worker thread so the UI remains responsive.
  ///
  /// The latest request always supersedes previous in-flight requests.
  fn queue_verify(&mut self) {
    let exercise = self.current_exercise().clone();

    // Auto-populate ripes.bin the first time a RISC-V exercise is run so
    // the resolved path is written to lq.toml and becomes visible and
    // editable by the user.
    if exercise.language == crate::exercise::Language::Riscv
      && self.config.ripes.bin.is_empty()
      && let Some(bin) = runner::find_ripes_binary()
    {
      self.config.ripes.bin = bin.to_string_lossy().to_string();
      self.save_config();
    }

    self.verify_request_id = self.verify_request_id.wrapping_add(1);
    let request_id = self.verify_request_id;
    self.verify_generation.store(request_id, Ordering::Relaxed);
    self.active_verify_request = Some(request_id);

    let config = self.config.clone();
    let tx = self.verify_result_tx.clone();
    let verify_generation = Arc::clone(&self.verify_generation);

    std::thread::spawn(move || {
      let cancel = runner::VerifyCancel::new(verify_generation, request_id);
      let result = runner::verify(&exercise, &config, &cancel);
      // Capture output from main() for the "Debug" page.
      let main_output = if exercise.language == crate::exercise::Language::Rust {
        runner::rust_run_main(&exercise, &config.rust, &cancel)
      } else {
        String::new()
      };

      let _ = tx.send(VerifyMessage {
        request_id,
        exercise_path: exercise.relative_path.clone(),
        result,
        main_output,
      });
    });

    self.verifying = true;
    self.needs_redraw = true;
  }

  /// Drain completed verification messages and apply only the latest request.
  fn poll_verify_results(&mut self) {
    let mut applied_latest = false;

    while let Ok(msg) = self.verify_result_rx.try_recv() {
      if Some(msg.request_id) != self.active_verify_request {
        // Superseded result: treat as cancelled from the UI perspective.
        continue;
      }

      self.config.update_score(&msg.exercise_path, msg.result.score, msg.result.threshold);
      self.last_result = Some(msg.result);
      self.last_main_output = msg.main_output;
      self.active_verify_request = None;
      self.verifying = false;
      applied_latest = true;
    }

    if applied_latest {
      self.save_config();
      self.needs_redraw = true;
    }
  }

  /// The main TUI event loop.
  ///
  /// # Errors
  ///
  /// Returns an error if terminal setup/teardown fails or on unrecoverable
  /// I/O errors.
  pub fn run(&mut self) -> Result<()> {
    // Enter alternate screen, enable raw mode, and capture mouse events.
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
      std::io::stderr(),
      crossterm::terminal::EnterAlternateScreen,
      crossterm::event::EnableMouseCapture,
    )?;

    let backend = CrosstermBackend::new(BufWriter::new(std::io::stderr()));
    let mut terminal = Terminal::new(backend)?;

    let result = self.event_loop(&mut terminal);

    // Always restore terminal state, even on error.
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(
      std::io::stderr(),
      crossterm::terminal::LeaveAlternateScreen,
      crossterm::event::DisableMouseCapture,
    );

    result
  }

  /// Inner event loop, separated so cleanup always runs.
  fn event_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<BufWriter<std::io::Stderr>>>) -> Result<()> {
    loop {
      // Check for terminal resize
      let size = terminal.size()?;
      if size.width != self.last_width {
        self.last_width = size.width;
        self.render_cache.clear(); // Width changed, invalidate cache
        self.needs_redraw = true;
      }

      // Only redraw when needed
      if self.needs_redraw {
        self.needs_redraw = false;
        let mut pending: Option<PendingOsc8> = None;
        let completed = terminal.draw(|frame| {
          pending = self.render(frame);
        })?;

        // Apply OSC 8 hyperlinks directly to the terminal after the frame
        // is flushed - bypasses ratatui's buffer diff width calculation.
        if let Some(ref p) = pending {
          // Use a separate BufWriter for OSC 8 sequences to avoid a double
          // mutable borrow of `terminal` (completed.buffer also borrows it).
          let mut w = BufWriter::new(std::io::stderr());
          p.write_to(completed.buffer, &mut w)?;
          // BufWriter flushes on drop
        }
      }

      self.poll_verify_results();

      // Check for file-change events from the watcher.
      if let Some(ref watcher) = self.watcher {
        // Drain all pending events.
        let mut changed = false;
        while let Ok(()) = watcher.event_rx.try_recv() {
          changed = true;
        }
        if changed {
          // Invalidate render cache so the next frame re-reads and re-highlights
          // the modified source file instead of serving stale cached lines.
          let exercise_path = self.current_exercise().relative_path.clone();
          self.render_cache.invalidate_exercise(&exercise_path);
          self.queue_verify();
        }
      }

      // Poll for crossterm events with a 200ms timeout.
      if event::poll(Duration::from_millis(200))? {
        match event::read()? {
          Event::Key(key) if key.kind == KeyEventKind::Press && self.handle_key(key) => {
            // Quit requested.
            self.save_config();
            return Ok(());
          }
          Event::Mouse(mouse) => {
            self.handle_mouse(mouse);
          }
          _ => {}
        }

        // Drain any additional events that arrived while we were handling the
        // first one.  This batches rapid scroll / navigation input so we only
        // render once at the final position instead of once per keypress.
        while event::poll(Duration::from_millis(0))? {
          match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press && self.handle_key(key) => {
              self.save_config();
              return Ok(());
            }
            Event::Mouse(_) => {
              // Skip mouse events in the drain loop — they arrive in
              // bursts and processing them all would cause multi-line
              // jumps per scroll tick.
            }
            _ => {}
          }
        }
      }
    }
  }

  /// Handle a key event. Returns `true` if the app should quit.
  fn handle_key(&mut self, key: KeyEvent) -> bool {
    // Ctrl+C always quits.
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
      return true;
    }

    // If the unlock warning is pending and the user pressed anything other
    // than `h`, cancel the warning.
    if self.solution_unlock_pending && key.code != KeyCode::Char('h') {
      self.solution_unlock_pending = false;
    }

    match key.code {
      KeyCode::Char('q') | KeyCode::Esc => true,

      KeyCode::Char('m') => {
        self.show_menu = !self.show_menu;
        self.needs_redraw = true;
        false
      }

      KeyCode::Left => {
        self.handle_left();
        self.needs_redraw = true;
        false
      }
      KeyCode::Right => {
        self.handle_right();
        self.needs_redraw = true;
        false
      }
      KeyCode::Tab => {
        self.handle_tab();
        self.needs_redraw = true;
        false
      }
      KeyCode::Char('o') => {
        // 'o' only works from ExerciseView to go to Overview
        if self.view == View::ExerciseView {
          self.view = View::Overview;
          self.overview_cursor = 0;
          self.scroll_offset = 0;
          self.needs_redraw = true;
        }
        false
      }
      KeyCode::Char('k') => {
        self.handle_next();
        self.needs_redraw = true;
        false
      }
      KeyCode::Char('j') => {
        self.handle_prev();
        self.needs_redraw = true;
        false
      }
      KeyCode::Char('h') => {
        self.handle_hint();
        self.needs_redraw = true;
        false
      }
      KeyCode::Up => {
        self.handle_scroll_up();
        self.needs_redraw = true;
        false
      }
      KeyCode::Down => {
        self.handle_scroll_down();
        self.needs_redraw = true;
        false
      }
      KeyCode::PageUp => {
        self.handle_page_up();
        self.needs_redraw = true;
        false
      }
      KeyCode::PageDown => {
        self.handle_page_down();
        self.needs_redraw = true;
        false
      }
      KeyCode::Enter => {
        self.handle_enter();
        self.needs_redraw = true;
        false
      }
      KeyCode::Char('e') => {
        self.open_in_editor();
        false
      }
      KeyCode::Char('a') => {
        self.handle_about();
        self.needs_redraw = true;
        false
      }
      KeyCode::Char('z') => {
        self.toggle_collapse_all();
        self.needs_redraw = true;
        false
      }
      _ => false,
    }
  }

  /// Navigate to the previous page (wrapping) within Exercise View.
  fn handle_left(&mut self) {
    if self.view != View::ExerciseView {
      return;
    }
    let idx = self.page.index();
    let new_idx = if idx == 0 {
      // Wrap: but check if Solution page is accessible.
      let last = ExercisePage::ALL.len() - 1;
      if self.can_view_solution() { last } else { last - 1 }
    } else {
      idx - 1
    };
    self.page = ExercisePage::from_index(new_idx);
    self.scroll_offset = 0;

    // If we landed on Solution, mark it seen.
    if self.page == ExercisePage::Solution {
      self.mark_current_solution_seen();
    }
  }

  /// Navigate to the next page within Exercise View.
  /// Solution page is gated behind `solution_seen` or `passed`.
  fn handle_right(&mut self) {
    if self.view != View::ExerciseView {
      return;
    }
    let idx = self.page.index();
    let next_idx = idx + 1;

    if next_idx >= ExercisePage::ALL.len() {
      // Wrap to first page.
      self.page = ExercisePage::from_index(0);
      self.scroll_offset = 0;
      return;
    }

    let next_page = ExercisePage::from_index(next_idx);

    // Gate the Solution page.
    if next_page == ExercisePage::Solution && !self.can_view_solution() {
      // Wrap to first page instead.
      self.page = ExercisePage::from_index(0);
      self.scroll_offset = 0;
      return;
    }

    self.page = next_page;
    self.scroll_offset = 0;

    // If we landed on Solution, mark it seen.
    if self.page == ExercisePage::Solution {
      self.mark_current_solution_seen();
    }
  }

  /// Toggle between ExerciseView and Overview.
  fn handle_tab(&mut self) {
    match self.view {
      View::ExerciseView => {
        self.view = View::Overview;
        self.overview_cursor = 0;
        self.scroll_offset = 0;
      }
      View::Overview => {
        self.view = View::ExerciseView;
        self.scroll_offset = 0;
      }
      View::About => {
        self.view = View::Overview;
        self.scroll_offset = 0;
      }
    }
  }

  /// Toggle the About page. Opens from any view; closes back to Overview.
  fn handle_about(&mut self) {
    if self.view == View::About {
      self.view = View::Overview;
    } else {
      self.view = View::About;
    }
    self.scroll_offset = 0;
  }

  /// Move to the next exercise (no blocking).
  fn handle_next(&mut self) {
    let new_index = self.current_index + 1;
    if new_index < self.exercises.len() {
      self.switch_exercise(new_index);
    }
  }

  /// Move to the previous exercise (no blocking).
  fn handle_prev(&mut self) {
    if self.current_index > 0 {
      self.switch_exercise(self.current_index - 1);
    }
  }

  /// Reveal the next hint, or - once all hints are shown - prompt the user
  /// to confirm unlocking the solution, then unlock it on a second press.
  fn handle_hint(&mut self) {
    // No-op when not in the Exercise View.
    if self.view != View::ExerciseView {
      return;
    }

    // On Theory, Task, or Debug pages, switch to Output first; ignore on Solution.
    match self.page {
      ExercisePage::Theory | ExercisePage::Task | ExercisePage::Debug => {
        self.page = ExercisePage::Output;
        self.scroll_offset = 0;
      }
      ExercisePage::Output => {}
      ExercisePage::Solution => return,
    }

    let exercise = self.current_exercise();
    let total = exercise.solution_data.as_ref().map_or(0, |sd| sd.hints.len());

    if self.hints_revealed < total {
      // Still hints left - reveal the next one and clear any pending flag.
      self.hints_revealed += 1;
      self.solution_unlock_pending = false;

      // Persist hint progress (cumulative counter + furthest level reached),
      // but only while the exercise is unsolved: once passed, revealing hints
      // for study is free and does not count. The hint still displays either way.
      let path = self.current_exercise().relative_path.clone();
      if self.config.record_hint_reveal_if_unpassed(&path, self.hints_revealed, total) {
        self.save_config();
      }

      self.scroll_to_hint_line(false);
    } else if !self.solution_unlock_pending {
      // All hints shown: first extra `h` → show warning.
      self.solution_unlock_pending = true;
      self.scroll_to_hint_line(true);
    } else {
      // Second extra `h` → actually unlock the solution and jump to it.
      self.solution_unlock_pending = false;
      self.mark_current_solution_seen();
      self.page = ExercisePage::Solution;
      self.scroll_offset = 0;
    }
  }

  /// Scroll the Output page so the newly revealed hint or unlock warning is
  /// visible at the very bottom of the viewport.
  ///
  /// Estimates the total content height after the reveal, then sets the
  /// scroll offset so the last line sits at the bottom of the viewport.
  fn scroll_to_hint_line(&mut self, is_warning: bool) {
    let area_width = self.last_width as usize;
    let hint_width = self.last_width.saturating_sub(6) as usize;

    // Pre-wrapped output line count (matches build_output_lines rendering)
    let output_lines: usize = self
      .last_result
      .as_ref()
      .map(|r| r.output.lines().map(|line| wrap_line(line, area_width).len()).sum())
      .unwrap_or(0);

    // Preamble: progress (1) + status (1) + blank (1) + N output + blank before hints (1)
    let base = 4 + output_lines;

    // Sum rendered lines for all hints currently visible.
    let revealed = self.hints_revealed;
    let hint_lines: usize = self
      .current_exercise()
      .solution_data
      .as_ref()
      .map(|sd| {
        let mut t = 0;
        for i in 0..revealed.min(sd.hints.len()) {
          t += 1; // header
          t += Self::hint_content_lines(&sd.hints[i], hint_width);
        }
        t
      })
      .unwrap_or(0);

    // Trailer after the last hint: blank + message
    let trailer = if is_warning {
      3 // blank + "⚠" message + instruction
    } else {
      2 // blank + prompt ("Press 'h'…" or "No more hints…")
    };

    let total_lines = base + hint_lines + trailer;
    let vh = self.viewport_height.max(1);
    self.scroll_offset = total_lines.saturating_sub(vh);
  }

  /// Number of rendered lines a single hint's body produces (after stripping
  /// code fences and word-wrapping at `hint_width`).  Does *not* include the
  /// header line.
  fn hint_content_lines(raw: &str, hint_width: usize) -> usize {
    let text = strip_code_fences(raw);
    if text.is_empty() {
      return 0;
    }
    let mut count = 0;
    for line in text.lines() {
      count += wrap_line(line, hint_width).len();
    }
    count
  }

  /// Handle mouse events (scrolling).
  fn handle_mouse(&mut self, mouse: crossterm::event::MouseEvent) {
    match mouse.kind {
      MouseEventKind::ScrollDown => {
        self.handle_scroll_down();
        self.needs_redraw = true;
      }
      MouseEventKind::ScrollUp => {
        self.handle_scroll_up();
        self.needs_redraw = true;
      }
      _ => {}
    }
  }

  /// Scroll up in the current view.
  fn handle_scroll_up(&mut self) {
    if self.view == View::Overview {
      if self.overview_cursor > 0 {
        self.overview_cursor -= 1;
        // Skip blank separator lines so the cursor lands on the nearest
        // group header or exercise.
        while self.overview_cursor > 0 && self.tree_line_kinds.get(self.overview_cursor).is_some_and(|k| matches!(k, LineKind::Blank)) {
          self.overview_cursor -= 1;
        }
      }
    } else {
      self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }
  }

  /// Scroll down in the current view.
  fn handle_scroll_down(&mut self) {
    if self.view == View::Overview {
      let max = self.content_height.saturating_sub(1);
      if self.overview_cursor < max {
        self.overview_cursor += 1;
        // Skip blank separator lines.
        while self.overview_cursor < max && self.tree_line_kinds.get(self.overview_cursor).is_some_and(|k| matches!(k, LineKind::Blank)) {
          self.overview_cursor += 1;
        }
      }
    } else {
      let max_scroll = self.content_height.saturating_sub(1);
      if self.scroll_offset < max_scroll {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
      }
    }
  }

  /// Page up - scroll content up by a larger amount.
  fn handle_page_up(&mut self) {
    if self.view == View::Overview {
      self.overview_cursor = self.overview_cursor.saturating_sub(10);
    } else {
      self.scroll_offset = self.scroll_offset.saturating_sub(10);
    }
  }

  /// Page down - scroll content down by a larger amount.
  fn handle_page_down(&mut self) {
    if self.view == View::Overview {
      let max = self.content_height.saturating_sub(1);
      self.overview_cursor = (self.overview_cursor + 10).min(max);
    } else {
      let max_scroll = self.content_height.saturating_sub(1);
      self.scroll_offset = (self.scroll_offset + 10).min(max_scroll);
    }
  }

  /// In Overview, jump to the exercise at the cursor and switch to
  /// ExerciseView, or toggle a group if the cursor is on a header.
  fn handle_enter(&mut self) {
    if self.view != View::Overview {
      return;
    }
    match self.tree_line_kinds.get(self.overview_cursor) {
      Some(LineKind::Exercise(ex_idx)) => {
        self.switch_exercise(*ex_idx);
        self.view = View::ExerciseView;
      }
      Some(LineKind::Group(path)) => {
        if !self.collapsed_groups.insert(path.clone()) {
          self.collapsed_groups.remove(path);
        }
        self.needs_redraw = true;
      }
      _ => {} // Blank lines — do nothing
    }
  }

  /// Collect all group paths from the tree recursively.
  fn collect_group_paths(&self, nodes: &[TreeNode]) -> HashSet<String> {
    let mut paths = HashSet::new();
    for node in nodes {
      if node.exercise.is_none() {
        paths.insert(node.path.clone());
        paths.extend(self.collect_group_paths(&node.children));
      }
    }
    paths
  }

  /// Toggle all groups between collapsed and expanded.
  /// Works from both Overview and ExerciseView.
  fn toggle_collapse_all(&mut self) {
    let all_groups = self.collect_group_paths(&self.tree);
    if self.collapsed_groups.len() == all_groups.len() {
      // All collapsed → expand all
      self.collapsed_groups.clear();
    } else {
      // Some or none collapsed → collapse all
      self.collapsed_groups = all_groups;
    }
  }

  /// Open the current exercise's source file in an editor.
  ///
  /// Resolution order:
  /// 1. `$VISUAL` - the user's preferred GUI editor (e.g. `code`, `zed`).
  ///    `$EDITOR` is intentionally skipped: terminal editors (vim, nano, …)
  ///    would conflict with the running TUI.
  /// 2. OS default text handler:
  ///    - macOS  : `open -t <file>` - always opens as text, even for unknown
  ///      extensions like `.asm` where plain `open` would fail.
  ///    - Linux  : `xdg-open <file>`
  ///    - Windows: `explorer <file>` - guaranteed to open any file as text.
  ///
  /// The process is spawned and forgotten - the TUI keeps running.
  fn open_in_editor(&self) {
    if self.view != View::ExerciseView {
      return;
    }
    let path = &self.current_exercise().source_path;

    // Prefer $VISUAL (GUI editor) over OS default.
    if let Ok(visual) = std::env::var("VISUAL")
      && !visual.is_empty()
    {
      let _ = std::process::Command::new(&visual).arg(path).spawn();
      return;
    }

    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").args(["-t", &path.to_string_lossy()]).spawn();

    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(path).spawn();

    #[cfg(target_os = "windows")]
    let _ = std::process::Command::new("explorer").arg(path).spawn();
  }

  /// Check whether the Solution page is accessible for the current exercise.
  fn can_view_solution(&self) -> bool {
    let exercise = self.current_exercise();
    let state = self.config.get_state(&exercise.relative_path);
    state.solution_seen || state.passed
  }

  /// Mark the current exercise's solution as seen and persist.
  ///
  /// No-op once the exercise is passed: a student who has already solved it can
  /// view the reference solution without it counting against them.
  fn mark_current_solution_seen(&mut self) {
    let path = self.current_exercise().relative_path.clone();
    if self.config.mark_solution_seen_if_unpassed(&path) {
      self.save_config();
    }
  }

  /// Dispatch rendering to the appropriate UI module based on the current
  /// view.
  pub fn render(&mut self, frame: &mut Frame) -> Option<PendingOsc8> {
    let full_area = frame.area();

    let menu_height = if self.show_menu {
      ui::statusbar::EXPANDED_HEIGHT
    } else {
      ui::statusbar::COLLAPSED_HEIGHT
    };

    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Min(1), Constraint::Length(menu_height)])
      .split(full_area);

    let content_area = chunks[0];
    let bar_area = chunks[1];

    let pending = match self.view {
      View::ExerciseView => ui::exercise_view::render(self, frame, content_area),
      View::Overview => {
        // Overview uses overview_cursor for navigation, not scroll_offset
        let (_tree_line_count, line_kinds) = ui::overview::render(
          frame,
          content_area,
          &self.tree,
          &self.exercises,
          &self.config,
          self.overview_cursor,
          &self.collapsed_groups,
        );
        // Cap cursor to the last non-blank line so trailing separator
        // lines aren't selectable, and clamp it in case collapsing/
        // expanding the tree changed the number of selectable lines.
        let last_non_blank = line_kinds.iter().rposition(|k| !matches!(k, LineKind::Blank));
        self.content_height = last_non_blank.map_or(0, |i| i + 1);
        self.overview_cursor = self.overview_cursor.min(self.content_height.saturating_sub(1));
        self.tree_line_kinds = line_kinds;
        self.viewport_height = content_area.height as usize;
        None
      }
      View::About => {
        let (pending, content_height, viewport_height) = ui::about::render(frame, content_area, self.scroll_offset);
        self.content_height = content_height;
        self.viewport_height = viewport_height;
        Some(pending)
      }
    };

    let solution_accessible = {
      let ex = self.current_exercise();
      let state = self.config.get_state(&ex.relative_path);
      state.passed || state.solution_seen
    };

    if self.show_menu {
      ui::statusbar::render(frame, bar_area, self.view, self.page, solution_accessible);
    } else {
      ui::statusbar::render_collapsed(frame, bar_area);
    }

    pending
  }
}
