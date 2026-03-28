//! Top-level application state and TUI event loop.

use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{bail, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::prelude::*;

use crate::config::{self, ProjectConfig};
use crate::ui::markdown::PendingOsc8;
use crate::exercise::{discover_exercises, Exercise, ExerciseStatus, Module};
use crate::runner::{self, ExerciseWatcher, VerificationResult};
use crate::ui;

/// Which top-level view is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Focused on a single exercise with paged content.
    ExerciseView,
    /// Table of all exercises with optional tree panel.
    Overview,
    /// About page — project info and credits.
    About,
}

/// Pages within the Exercise View.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExercisePage {
    /// Theory / background reading.
    Theory,
    /// Task description.
    Task,
    /// Verification output.
    Output,
    /// Reference solution (gated).
    Solution,
}

impl ExercisePage {
    /// All pages in display order.
    const ALL: [ExercisePage; 4] = [
        ExercisePage::Theory,
        ExercisePage::Task,
        ExercisePage::Output,
        ExercisePage::Solution,
    ];

    /// Index of this page in the page list.
    pub(crate) fn index(self) -> usize {
        match self {
            ExercisePage::Theory => 0,
            ExercisePage::Task => 1,
            ExercisePage::Output => 2,
            ExercisePage::Solution => 3,
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
            ExercisePage::Output => "Output",
            ExercisePage::Solution => "Solution",
        }
    }
}

/// Main application state.
pub struct App {
    /// Discovered exercise modules.
    pub modules: Vec<Module>,
    /// Flat index mapping: `(module_idx, exercise_idx)` for each exercise.
    pub exercises: Vec<(usize, usize)>,
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
    /// Cursor position in the Overview table.
    pub overview_cursor: usize,
    /// Whether the tree panel is visible in the Overview.
    pub show_tree: bool,
    /// Whether the bottom status bar is expanded.
    pub show_menu: bool,
    /// Vertical scroll offset for markdown/text content.
    pub scroll_offset: usize,
    /// File watcher for the current exercise's source file.
    pub watcher: Option<ExerciseWatcher>,
    /// Whether the "unlock solution?" warning is awaiting a second `h` press.
    pub solution_unlock_pending: bool,
}

impl App {
    /// Create a new `App` by discovering exercises and loading config.
    ///
    /// # Errors
    ///
    /// Returns an error if no exercises are found or if the config cannot be
    /// loaded.
    pub fn new(repo_path: PathBuf) -> Result<Self> {
        let (modules, _errors) = discover_exercises(&repo_path);

        if modules.is_empty() {
            bail!("no exercises found in {}", repo_path.display());
        }

        // Build the flat index.
        let mut exercises = Vec::new();
        for (mi, module) in modules.iter().enumerate() {
            for ei in 0..module.exercises.len() {
                exercises.push((mi, ei));
            }
        }

        if exercises.is_empty() {
            bail!("no exercises found in {}", repo_path.display());
        }

        let cfg_path = config::config_path(&repo_path);
        let config = ProjectConfig::load(&cfg_path)?;

        // Resolve starting index from config.current_exercise.
        let current_index = config
            .current_exercise
            .as_deref()
            .and_then(|name| {
                exercises.iter().position(|&(mi, ei)| {
                    modules[mi].exercises[ei].relative_path == name
                })
            })
            .unwrap_or(0);

        let mut app = App {
            modules,
            exercises,
            config,
            config_path: cfg_path,
            current_index,
            view: View::ExerciseView,
            page: ExercisePage::Theory,
            hints_revealed: 0,
            last_result: None,
            overview_cursor: current_index,
            show_tree: true,
            show_menu: true,
            scroll_offset: 0,
            watcher: None,
            solution_unlock_pending: false,
        };

        app.setup_watcher();
        app.run_verify();
        app.save_config();

        Ok(app)
    }

    /// Get a reference to the exercise at `current_index`.
    pub fn current_exercise(&self) -> &Exercise {
        let (mi, ei) = self.exercises[self.current_index];
        &self.modules[mi].exercises[ei]
    }

    /// Get the exercise at a specific flat index.
    #[allow(dead_code)]
    pub fn exercise_at(&self, index: usize) -> &Exercise {
        let (mi, ei) = self.exercises[index];
        &self.modules[mi].exercises[ei]
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
        if state.passed && state.solution_seen {
            ExerciseStatus::Complete
        } else if state.passed {
            ExerciseStatus::Partial
        } else {
            ExerciseStatus::Failing
        }
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
        self.page = ExercisePage::Theory;
        self.scroll_offset = 0;
        self.setup_watcher();
        self.run_verify();
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

    /// Run verification on the current exercise and update state.
    fn run_verify(&mut self) {
        let exercise = self.current_exercise().clone();

        // Auto-populate ripes.bin the first time a RISC-V exercise is run so
        // the resolved path is written to lq.toml and becomes visible and
        // editable by the user.
        if exercise.language == crate::exercise::Language::Riscv
            && self.config.ripes.bin.is_empty()
        {
            if let Some(bin) = runner::find_ripes_binary() {
                self.config.ripes.bin = bin.to_string_lossy().to_string();
                self.save_config();
            }
        }

        let result = runner::verify(&exercise, &self.config);
        self.config
            .update_score(&exercise.relative_path, result.score, result.threshold);
        self.last_result = Some(result);
    }

    /// The main TUI event loop.
    ///
    /// # Errors
    ///
    /// Returns an error if terminal setup/teardown fails or on unrecoverable
    /// I/O errors.
    pub fn run(&mut self) -> Result<()> {
        // Enter alternate screen and enable raw mode.
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(std::io::stderr());
        let mut terminal = Terminal::new(backend)?;

        let result = self.event_loop(&mut terminal);

        // Always restore terminal state, even on error.
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen);

        result
    }

    /// Inner event loop, separated so cleanup always runs.
    fn event_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stderr>>) -> Result<()> {
        loop {
            let mut pending: Option<PendingOsc8> = None;
            let completed = terminal.draw(|frame| {
                pending = self.render(frame);
            })?;

            // Apply OSC 8 hyperlinks directly to the terminal after the frame
            // is flushed — bypasses ratatui's buffer diff width calculation.
            if let Some(ref p) = pending {
                let mut stderr = std::io::stderr();
                p.write_to(completed.buffer, &mut stderr)?;
                stderr.flush()?;
            }

            // Check for file-change events from the watcher.
            if let Some(ref watcher) = self.watcher {
                // Drain all pending events.
                let mut changed = false;
                while let Ok(()) = watcher.event_rx.try_recv() {
                    changed = true;
                }
                if changed {
                    self.run_verify();
                    self.save_config();
                }
            }

            // Poll for crossterm events with a 200ms timeout.
            if event::poll(Duration::from_millis(200))?
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && self.handle_key(key)
            {
                // Quit requested.
                self.save_config();
                return Ok(());
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
                false
            }

            KeyCode::Left => {
                self.handle_left();
                false
            }
            KeyCode::Right => {
                self.handle_right();
                false
            }
            KeyCode::Tab | KeyCode::Char('o') => {
                self.handle_tab();
                false
            }
            KeyCode::Char('k') => {
                self.handle_next();
                false
            }
            KeyCode::Char('j') => {
                self.handle_prev();
                false
            }
            KeyCode::Char('h') => {
                self.handle_hint();
                false
            }
            KeyCode::Up => {
                self.handle_scroll_up();
                false
            }
            KeyCode::Down => {
                self.handle_scroll_down();
                false
            }
            KeyCode::PageUp => {
                self.handle_page_up();
                false
            }
            KeyCode::PageDown => {
                self.handle_page_down();
                false
            }
            KeyCode::Enter => {
                self.handle_enter();
                false
            }
            KeyCode::Char('t') => {
                self.handle_toggle_tree();
                false
            }
            KeyCode::Char('e') => {
                self.open_in_editor();
                false
            }
            KeyCode::Char('a') => {
                self.handle_about();
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
            if self.can_view_solution() {
                last
            } else {
                last - 1
            }
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
                self.overview_cursor = self.current_index;
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

    /// Move to the next exercise. Blocked unless current exercise's
    /// `solution_seen` is true.
    fn handle_next(&mut self) {
        let exercise = self.current_exercise();
        let state = self.config.get_state(&exercise.relative_path);
        if !state.solution_seen {
            return;
        }
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

    /// Reveal the next hint, or — once all hints are shown — prompt the user
    /// to confirm unlocking the solution, then unlock it on a second press.
    fn handle_hint(&mut self) {
        // No-op when not in the Exercise View.
        if self.view != View::ExerciseView {
            return;
        }

        // On Theory or Task pages, switch to Output first; ignore on Solution.
        match self.page {
            ExercisePage::Theory | ExercisePage::Task => {
                self.page = ExercisePage::Output;
                self.scroll_offset = 0;
            }
            ExercisePage::Output => {}
            ExercisePage::Solution => return,
        }

        let total = self
            .current_exercise()
            .solution_data
            .as_ref()
            .map_or(0, |sd| sd.hints.len());

        if self.hints_revealed < total {
            // Still hints left — reveal the next one and clear any pending flag.
            self.hints_revealed += 1;
            self.solution_unlock_pending = false;
            self.scroll_to_hint_line(false, total);
        } else if !self.solution_unlock_pending {
            // All hints shown: first extra `h` → show warning.
            self.solution_unlock_pending = true;
            self.scroll_to_hint_line(true, total);
        } else {
            // Second extra `h` → actually unlock the solution and jump to it.
            self.solution_unlock_pending = false;
            self.mark_current_solution_seen();
            self.page = ExercisePage::Solution;
            self.scroll_offset = 0;
        }
    }

    /// Scroll the Output page so the newly revealed hint or unlock warning is
    /// visible, with two lines of context above it.
    ///
    /// Line layout produced by `build_output_lines` (when a result exists):
    /// ```text
    ///   0        progress bar
    ///   1        PASSED / FAILING
    ///   2        blank
    ///   3…3+N-1  runner output  (N = result.output.lines().count())
    ///   3+N      blank before hints
    ///   4+N+k    hint k  (k = 0-based index)
    /// ```
    /// When `is_warning` is true all `total_hints` hints are revealed and the
    /// `⚠` line sits at `4 + N + total_hints + 1` (after a blank separator).
    fn scroll_to_hint_line(&mut self, is_warning: bool, total_hints: usize) {
        let output_lines = self
            .last_result
            .as_ref()
            .map(|r| r.output.lines().count())
            .unwrap_or(0);

        let target = if is_warning {
            // blank at 4+N+total_hints, ⚠ at 4+N+total_hints+1
            5 + output_lines + total_hints
        } else {
            // hints_revealed already incremented; hint index is hints_revealed-1
            4 + output_lines + self.hints_revealed - 1
        };

        self.scroll_offset = target.saturating_sub(2);
    }

    /// Scroll up in the current view.
    fn handle_scroll_up(&mut self) {
        if self.view == View::Overview {
            if self.overview_cursor > 0 {
                self.overview_cursor -= 1;
            }
        } else {
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
        }
    }

    /// Scroll down in the current view.
    fn handle_scroll_down(&mut self) {
        if self.view == View::Overview {
            let max = self.exercises.len().saturating_sub(1);
            if self.overview_cursor < max {
                self.overview_cursor += 1;
            }
        } else {
            self.scroll_offset = self.scroll_offset.saturating_add(1);
        }
    }

    /// Page up — scroll content up by a larger amount.
    fn handle_page_up(&mut self) {
        if self.view == View::Overview {
            self.overview_cursor = self.overview_cursor.saturating_sub(10);
        } else {
            self.scroll_offset = self.scroll_offset.saturating_sub(10);
        }
    }

    /// Page down — scroll content down by a larger amount.
    fn handle_page_down(&mut self) {
        if self.view == View::Overview {
            let max = self.exercises.len().saturating_sub(1);
            self.overview_cursor = (self.overview_cursor + 10).min(max);
        } else {
            self.scroll_offset = self.scroll_offset.saturating_add(10);
        }
    }

    /// In Overview, jump to the exercise at the cursor and switch to
    /// ExerciseView.
    fn handle_enter(&mut self) {
        if self.view != View::Overview {
            return;
        }
        if self.overview_cursor < self.exercises.len() {
            self.switch_exercise(self.overview_cursor);
            self.view = View::ExerciseView;
        }
    }

    /// Toggle the tree panel (only in Overview view).
    fn handle_toggle_tree(&mut self) {
        if self.view == View::Overview {
            self.show_tree = !self.show_tree;
        }
    }

    /// Open the current exercise's source file in an editor.
    ///
    /// Resolution order:
    /// 1. `$VISUAL` — the user's preferred GUI editor (e.g. `code`, `zed`).
    ///    `$EDITOR` is intentionally skipped: terminal editors (vim, nano, …)
    ///    would conflict with the running TUI.
    /// 2. OS default text handler:
    ///    - macOS  : `open -t <file>` — always opens as text, even for unknown
    ///               extensions like `.asm` where plain `open` would fail.
    ///    - Linux  : `xdg-open <file>`
    ///    - Windows: `notepad <file>` — guaranteed to open any file as text.
    ///
    /// The process is spawned and forgotten — the TUI keeps running.
    fn open_in_editor(&self) {
        if self.view != View::ExerciseView {
            return;
        }
        let path = &self.current_exercise().source_path;

        // Prefer $VISUAL (GUI editor) over OS default.
        if let Ok(visual) = std::env::var("VISUAL") {
            if !visual.is_empty() {
                let _ = std::process::Command::new(&visual).arg(path).spawn();
                return;
            }
        }

        #[cfg(target_os = "macos")]
        let _ = std::process::Command::new("open")
            .args(["-t", &path.to_string_lossy().into_owned()])
            .spawn();

        #[cfg(target_os = "linux")]
        let _ = std::process::Command::new("xdg-open").arg(path).spawn();

        #[cfg(target_os = "windows")]
        let _ = std::process::Command::new("notepad").arg(path).spawn();
    }

    /// Check whether the Solution page is accessible for the current exercise.
    fn can_view_solution(&self) -> bool {
        let exercise = self.current_exercise();
        let state = self.config.get_state(&exercise.relative_path);
        state.solution_seen || state.passed
    }

    /// Mark the current exercise's solution as seen and persist.
    fn mark_current_solution_seen(&mut self) {
        let path = self.current_exercise().relative_path.clone();
        self.config.mark_solution_seen(&path);
        self.save_config();
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
                ui::overview::render(
                    frame,
                    content_area,
                    &self.modules,
                    &self.exercises,
                    &self.config,
                    self.overview_cursor,
                    self.show_tree,
                );
                None
            }
            View::About => Some(ui::about::render(frame, content_area, self.scroll_offset)),
        };

        let solution_accessible = {
            let ex = self.current_exercise();
            let state = self.config.get_state(&ex.relative_path);
            state.passed || state.solution_seen
        };

        if self.show_menu {
            ui::statusbar::render(
                frame,
                bar_area,
                self.view,
                self.page,
                self.show_tree,
                solution_accessible,
            );
        } else {
            ui::statusbar::render_collapsed(frame, bar_area);
        }

        pending
    }
}