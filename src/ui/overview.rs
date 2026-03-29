//! Overview - progress bar, exercise table, tree panel.
//!
//! Renders the main overview screen consisting of:
//! * A progress bar showing completed / total exercises.
//! * A scrollable exercise table (delegated to [`super::table`]).
//! * An optional tree panel showing the module/exercise hierarchy.
//! * A status bar with keybinding hints.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use super::table::{self, Column, TableData};
use crate::config::ProjectConfig;
use crate::exercise::{ExerciseStatus, Module};

// ---------------------------------------------------------------------------
// Status derivation
// ---------------------------------------------------------------------------

/// Derive an [`ExerciseStatus`] from persisted [`crate::config::ExerciseState`].
///
/// * `passed && solution_seen` → [`ExerciseStatus::Complete`]
/// * `passed` → [`ExerciseStatus::Partial`]
/// * otherwise → [`ExerciseStatus::Failing`]
pub fn derive_status(state: &crate::config::ExerciseState) -> ExerciseStatus {
  if state.passed && state.solution_seen {
    ExerciseStatus::Complete
  } else if state.passed {
    ExerciseStatus::Partial
  } else {
    ExerciseStatus::Failing
  }
}

// ---------------------------------------------------------------------------
// Public render entry-point
// ---------------------------------------------------------------------------

/// Render the full Overview screen.
///
/// `exercises` is a flat index of `(module_idx, exercise_idx)` pairs that
/// determines the display order and the meaning of `overview_cursor`.
#[allow(clippy::too_many_arguments)]
pub fn render(
  frame: &mut Frame,
  area: Rect,
  modules: &[Module],
  exercises: &[(usize, usize)],
  config: &ProjectConfig,
  overview_cursor: usize,
  show_tree: bool,
) {
  if area.height < 2 || area.width < 10 {
    return;
  }

  // Split into progress bar (3 lines) + table/tree region.
  let vertical = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(3), // progress bar
      Constraint::Min(1),    // exercise table (+ optional tree)
    ])
    .split(area);

  let progress_area = vertical[0];
  let content_area = vertical[1];

  // --- progress bar ---------------------------------------------------
  render_progress_bar(frame, progress_area, modules, exercises, config);

  // --- table + optional tree ------------------------------------------
  // The tree is a side panel - hide it only when the terminal is too narrow
  // to split meaningfully (< 80 columns), not based on height.
  let tree_visible = show_tree && area.width >= 80;

  if tree_visible {
    let horizontal = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
      .split(content_area);

    render_exercise_table(frame, horizontal[0], modules, exercises, config, overview_cursor);
    render_tree_panel(frame, horizontal[1], modules, exercises, config, overview_cursor);
  } else {
    render_exercise_table(frame, content_area, modules, exercises, config, overview_cursor);
  }
}

// ---------------------------------------------------------------------------
// Progress bar
// ---------------------------------------------------------------------------

fn render_progress_bar(frame: &mut Frame, area: Rect, modules: &[Module], exercises: &[(usize, usize)], config: &ProjectConfig) {
  let total = exercises.len();
  let completed = exercises
    .iter()
    .filter(|&&(mi, ei)| {
      if let Some(module) = modules.get(mi)
        && let Some(exercise) = module.exercises.get(ei)
      {
        let state = config.get_state(&exercise.relative_path);
        return derive_status(&state) == ExerciseStatus::Complete;
      }
      false
    })
    .count();

  // Bar width: area.width minus the label overhead.
  // Label format: "Progress: [====----]  12/42"
  let label_prefix = "Progress: [";
  let label_suffix_example = format!("]  {completed}/{total}");
  let overhead = label_prefix.len() + label_suffix_example.len();
  let bar_width = (area.width as usize).saturating_sub(overhead);

  let filled = if total == 0 { 0 } else { (bar_width * completed) / total };
  let empty = bar_width.saturating_sub(filled);

  let mut spans: Vec<Span<'_>> = Vec::new();
  spans.push(Span::styled(label_prefix.to_string(), Style::default().fg(Color::White)));
  spans.push(Span::styled("=".repeat(filled), Style::default().fg(Color::Green)));
  spans.push(Span::styled("-".repeat(empty), Style::default().fg(Color::DarkGray)));
  spans.push(Span::styled(format!("]  {completed}/{total}"), Style::default().fg(Color::White)));

  let line = Line::from(spans);
  let block = Block::default().borders(Borders::NONE);
  let paragraph = Paragraph::new(vec![line]).block(block).wrap(Wrap { trim: false });
  frame.render_widget(paragraph, area);
}

// ---------------------------------------------------------------------------
// Exercise table
// ---------------------------------------------------------------------------

fn render_exercise_table(frame: &mut Frame, area: Rect, modules: &[Module], exercises: &[(usize, usize)], config: &ProjectConfig, overview_cursor: usize) {
  let columns = vec![
    Column {
      header: "ID".to_string(),
      width: 20,
    },
    Column {
      header: "Name".to_string(),
      width: 30,
    },
    Column {
      header: "Language".to_string(),
      width: 12,
    },
    Column {
      header: "Difficulty".to_string(),
      width: 10,
    },
    Column {
      header: "Status".to_string(),
      width: 12,
    },
    Column {
      header: "Topics".to_string(),
      width: 30,
    },
  ];

  let rows: Vec<Vec<String>> = exercises
    .iter()
    .map(|&(mi, ei)| {
      let (id, name, lang_name, difficulty, status_text, topics) = if let Some(module) = modules.get(mi) {
        if let Some(ex) = module.exercises.get(ei) {
          let state = config.get_state(&ex.relative_path);
          let status = derive_status(&state);
          let stars = "*".repeat(ex.difficulty as usize);
          let status_str = format!("{} {}", status.symbol(), status.label());
          let topics_str = ex.topics.join(", ");
          (
            ex.id.clone(),
            ex.name.clone(),
            ex.language.display_name().to_string(),
            stars,
            status_str,
            topics_str,
          )
        } else {
          empty_row()
        }
      } else {
        empty_row()
      };
      vec![id, name, lang_name, difficulty, status_text, topics]
    })
    .collect();

  let data = TableData { columns, rows };

  let header_style = Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD);
  let highlight_style = Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD);

  let block = Block::default()
    .title(" Exercises ")
    .borders(Borders::ALL)
    .border_style(Style::default().fg(Color::DarkGray));

  let inner = block.inner(area);
  frame.render_widget(block, area);

  table::render_table(frame, inner, &data, overview_cursor, header_style, highlight_style);
}

/// Produce a placeholder row when a module/exercise index is out of bounds.
fn empty_row() -> (String, String, String, String, String, String) {
  (String::new(), String::new(), String::new(), String::new(), String::new(), String::new())
}

// ---------------------------------------------------------------------------
// Tree panel
// ---------------------------------------------------------------------------

fn render_tree_panel(frame: &mut Frame, area: Rect, modules: &[Module], exercises: &[(usize, usize)], config: &ProjectConfig, overview_cursor: usize) {
  // Determine which (module_idx, exercise_idx) is selected.
  let selected = exercises.get(overview_cursor).copied();

  let mut lines: Vec<Line<'_>> = Vec::new();

  for (mi, module) in modules.iter().enumerate() {
    let exercise_count = module.exercises.len();
    // Module header
    lines.push(Line::from(Span::styled(
      format!("  {}/", module.name),
      Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    for (ei, ex) in module.exercises.iter().enumerate() {
      let state = config.get_state(&ex.relative_path);
      let status = derive_status(&state);
      let is_last = ei + 1 == exercise_count;
      let connector = if is_last { "+--" } else { "|--" };

      let is_selected = selected == Some((mi, ei));

      let style = if is_selected {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
      } else {
        match status {
          ExerciseStatus::Complete => Style::default().fg(Color::Green),
          ExerciseStatus::Partial => Style::default().fg(Color::Yellow),
          ExerciseStatus::Failing => Style::default().fg(Color::Red),
        }
      };

      lines.push(Line::from(Span::styled(format!("  {connector} {} {}", status.symbol(), ex.name), style)));
    }

    // Blank line between modules.
    lines.push(Line::from(""));
  }

  let block = Block::default()
    .title(" Modules ")
    .borders(Borders::ALL)
    .border_style(Style::default().fg(Color::DarkGray));

  // Scroll the tree so the selected exercise stays visible.
  let inner_height = block.inner(area).height as usize;
  let selected_line = find_selected_line_in_tree(modules, exercises, overview_cursor);
  let scroll = if selected_line >= inner_height {
    (selected_line - inner_height + 1) as u16
  } else {
    0
  };

  let paragraph = Paragraph::new(lines).block(block).scroll((scroll, 0));
  frame.render_widget(paragraph, area);
}

/// Return the zero-based line index in the tree output that corresponds to
/// the exercise at `overview_cursor`.
fn find_selected_line_in_tree(modules: &[Module], exercises: &[(usize, usize)], overview_cursor: usize) -> usize {
  let selected = match exercises.get(overview_cursor) {
    Some(&pair) => pair,
    None => return 0,
  };

  let mut line: usize = 0;
  for (mi, module) in modules.iter().enumerate() {
    // Module header line.
    line += 1;
    for (ei, _) in module.exercises.iter().enumerate() {
      if (mi, ei) == selected {
        return line;
      }
      line += 1;
    }
    // Blank separator line after each module.
    line += 1;
  }
  0
}

// ---------------------------------------------------------------------------
// Status bar
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;
  use crate::config::ExerciseState;

  #[test]
  fn derive_status_failing() {
    let state = ExerciseState {
      best_score: 0.0,
      passed: false,
      solution_seen: false,
    };
    assert_eq!(derive_status(&state), ExerciseStatus::Failing);
  }

  #[test]
  fn derive_status_partial() {
    let state = ExerciseState {
      best_score: 1.0,
      passed: true,
      solution_seen: false,
    };
    assert_eq!(derive_status(&state), ExerciseStatus::Partial);
  }

  #[test]
  fn derive_status_complete() {
    let state = ExerciseState {
      best_score: 1.0,
      passed: true,
      solution_seen: true,
    };
    assert_eq!(derive_status(&state), ExerciseStatus::Complete);
  }

  #[test]
  fn derive_status_seen_but_not_passed_is_failing() {
    let state = ExerciseState {
      best_score: 0.3,
      passed: false,
      solution_seen: true,
    };
    assert_eq!(derive_status(&state), ExerciseStatus::Failing);
  }
}
