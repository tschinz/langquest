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
use super::term_caps::chars;
use crate::config::ProjectConfig;
use crate::exercise::{Exercise, ExerciseStatus, TreeNode};

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
/// `modules` is the tree of groups and exercises (for the tree panel).
/// `exercises` is a flat list of all exercises (for the table and cursor).
#[allow(clippy::too_many_arguments)]
pub fn render(frame: &mut Frame, area: Rect, modules: &[TreeNode], exercises: &[Exercise], config: &ProjectConfig, overview_cursor: usize, show_tree: bool) {
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

fn render_progress_bar(frame: &mut Frame, area: Rect, _modules: &[TreeNode], exercises: &[Exercise], config: &ProjectConfig) {
  let total = exercises.len();
  let completed = exercises
    .iter()
    .filter(|ex| {
      let state = config.get_state(&ex.relative_path);
      derive_status(&state) == ExerciseStatus::Complete
    })
    .count();

  // Bar width: area.width minus the label overhead.
  // Label format: "Progress: [====----]  12/42"
  let label_prefix = "Progress: [";
  let label_suffix_example = format!("]  {completed}/{total}");
  let overhead = label_prefix.len() + label_suffix_example.len();
  let bar_width = (area.width as usize).saturating_sub(overhead);

  let filled = (bar_width * completed).checked_div(total).unwrap_or(0);
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

fn render_exercise_table(frame: &mut Frame, area: Rect, _modules: &[TreeNode], exercises: &[Exercise], config: &ProjectConfig, overview_cursor: usize) {
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
    .map(|ex| {
      let state = config.get_state(&ex.relative_path);
      let status = derive_status(&state);
      let stars = "*".repeat(ex.difficulty as usize);
      let status_str = format!("{} {}", status.symbol(), status.label());
      let topics_str = ex.topics.join(", ");
      vec![
        ex.id.clone(),
        ex.name.clone(),
        ex.language.display_name().to_string(),
        stars,
        status_str,
        topics_str,
      ]
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

// ---------------------------------------------------------------------------
// Tree panel
// ---------------------------------------------------------------------------

fn render_tree_panel(frame: &mut Frame, area: Rect, nodes: &[TreeNode], exercises: &[Exercise], config: &ProjectConfig, overview_cursor: usize) {
  let selected_path = exercises.get(overview_cursor).map(|ex| ex.relative_path.as_str());

  let mut lines: Vec<Line<'_>> = Vec::new();

  for node in nodes {
    if let Some(ex) = &node.exercise {
      // Top-level exercise (unusual but handle gracefully)
      let state = config.get_state(&ex.relative_path);
      let status = derive_status(&state);
      let is_selected = selected_path == Some(ex.relative_path.as_str());
      let style = if is_selected {
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
      } else {
        match status {
          ExerciseStatus::Complete => Style::default().fg(Color::Green),
          ExerciseStatus::Partial => Style::default().fg(Color::Yellow),
          ExerciseStatus::Failing => Style::default().fg(Color::Red),
        }
      };
      lines.push(Line::from(Span::styled(format!("  {} {}", status.symbol(), ex.name), style)));
    } else {
      // Top-level group header (no connector, matching old behaviour)
      lines.push(Line::from(Span::styled(
        format!("  {}/", node.name),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
      )));

      // Render children recursively
      let child_count = node.children.len();
      for (i, child) in node.children.iter().enumerate() {
        render_tree_node(child, config, selected_path, "    ", i + 1 == child_count, &mut lines);
      }

      // Blank line between top-level groups (matching old behaviour)
      lines.push(Line::from(""));
    }
  }

  let block = Block::default()
    .title(" Modules ")
    .borders(Borders::ALL)
    .border_style(Style::default().fg(Color::DarkGray));

  // Scroll the tree so the selected exercise stays visible.
  let inner_height = block.inner(area).height as usize;
  let selected_line = find_selected_line_in_tree(nodes, exercises, overview_cursor);
  let scroll = if selected_line >= inner_height {
    (selected_line - inner_height + 1) as u16
  } else {
    0
  };

  let paragraph = Paragraph::new(lines).block(block).scroll((scroll, 0));
  frame.render_widget(paragraph, area);
}

/// Recursively render a tree node and its children.
fn render_tree_node(node: &TreeNode, config: &ProjectConfig, selected_path: Option<&str>, prefix: &str, is_last: bool, lines: &mut Vec<Line<'_>>) {
  let connector = if is_last { chars::tree_last() } else { chars::tree_branch() };

  if let Some(ex) = &node.exercise {
    let state = config.get_state(&ex.relative_path);
    let status = derive_status(&state);
    let is_selected = selected_path == Some(ex.relative_path.as_str());

    let style = if is_selected {
      Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
    } else {
      match status {
        ExerciseStatus::Complete => Style::default().fg(Color::Green),
        ExerciseStatus::Partial => Style::default().fg(Color::Yellow),
        ExerciseStatus::Failing => Style::default().fg(Color::Red),
      }
    };

    lines.push(Line::from(Span::styled(format!("{prefix}{connector} {} {}", status.symbol(), ex.name), style)));
  } else {
    // Group header
    lines.push(Line::from(Span::styled(
      format!("{prefix}{connector} {}/", node.name),
      Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    )));

    // Children
    let child_prefix = format!("{prefix}{}", if is_last { "  " } else { "│ " });
    let child_count = node.children.len();
    for (i, child) in node.children.iter().enumerate() {
      render_tree_node(child, config, selected_path, &child_prefix, i + 1 == child_count, lines);
    }
  }
}

/// Return the zero-based line index in the tree output that corresponds to
/// the exercise at `overview_cursor`.
fn find_selected_line_in_tree(nodes: &[TreeNode], exercises: &[Exercise], overview_cursor: usize) -> usize {
  let selected_path = exercises.get(overview_cursor).map(|ex| ex.relative_path.as_str());
  let mut line: usize = 0;
  for node in nodes {
    if node.is_group() {
      // Top-level group header
      line += 1;
      // Children (recursive) — return early if found
      if find_selected_line_in_tree_children(&node.children, selected_path, &mut line) {
        return line.saturating_sub(1);
      }
      // Blank line after top-level group
      line += 1;
    } else if let Some(ex) = &node.exercise {
      // Top-level exercise
      line += 1;
      if selected_path == Some(ex.relative_path.as_str()) {
        return line.saturating_sub(1);
      }
    }
  }
  line.saturating_sub(1)
}

/// Recursively traverse child tree nodes, counting rendered lines,
/// until we find the exercise that matches `selected_path`.
fn find_selected_line_in_tree_children(nodes: &[TreeNode], selected_path: Option<&str>, line: &mut usize) -> bool {
  for node in nodes {
    if let Some(ex) = &node.exercise {
      *line += 1;
      if selected_path == Some(ex.relative_path.as_str()) {
        return true;
      }
    } else {
      // Group header
      *line += 1;
      // Children
      if find_selected_line_in_tree_children(&node.children, selected_path, line) {
        return true;
      }
    }
  }
  false
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
