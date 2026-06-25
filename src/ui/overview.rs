//! Overview - progress bar, exercise tree panel.
//!
//! Renders the main overview screen consisting of:
//! * A progress bar showing completed / total exercises.
//! * A scrollable tree panel showing the module/exercise hierarchy
//!   with Language, Difficulty and Topics inline on each exercise.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

use std::collections::HashSet;

use super::term_caps::chars;
use crate::app::LineKind;
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

/// Compute the aggregate [`ExerciseStatus`] for all exercises under `node`.
fn group_status(node: &TreeNode, config: &ProjectConfig) -> ExerciseStatus {
  let mut total = 0usize;
  let mut passed = 0usize;
  let mut seen = 0usize;
  count_group_exercises(node, config, &mut total, &mut passed, &mut seen);
  if total == 0 {
    return ExerciseStatus::Failing;
  }
  if seen == total {
    ExerciseStatus::Complete
  } else if passed > 0 {
    ExerciseStatus::Partial
  } else {
    ExerciseStatus::Failing
  }
}

/// Recursively count exercises and their states under `node`.
fn count_group_exercises(node: &TreeNode, config: &ProjectConfig, total: &mut usize, passed: &mut usize, seen: &mut usize) {
  for child in &node.children {
    if let Some(ex) = &child.exercise {
      *total += 1;
      let state = config.get_state(&ex.relative_path);
      if state.passed {
        *passed += 1;
      }
      if state.solution_seen {
        *seen += 1;
      }
    } else {
      count_group_exercises(child, config, total, passed, seen);
    }
  }
}

// ---------------------------------------------------------------------------
// Public render entry-point
// ---------------------------------------------------------------------------

/// Render the full Overview screen.
///
/// `modules` is the tree of groups and exercises.
/// `exercises` is the flat exercise list (for cursor navigation).
#[allow(clippy::too_many_arguments)]
pub fn render(
  frame: &mut Frame,
  area: Rect,
  modules: &[TreeNode],
  exercises: &[Exercise],
  config: &ProjectConfig,
  mut overview_cursor: usize,
  collapsed_groups: &HashSet<String>,
) -> (usize, Vec<LineKind>) {
  if area.height < 2 || area.width < 10 {
    return (0, Vec::new());
  }

  // Split into progress bar (3 lines) + tree region.
  let vertical = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(3), Constraint::Min(1)])
    .split(area);

  let progress_area = vertical[0];
  let content_area = vertical[1];

  // --- progress bar ---------------------------------------------------
  render_progress_bar(frame, progress_area, modules, exercises, config);

  // --- full-width tree panel ------------------------------------------
  let line_kinds = render_tree_panel(frame, content_area, modules, exercises, config, &mut overview_cursor, collapsed_groups);
  (overview_cursor, line_kinds)
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

  let pct = completed.checked_mul(100).and_then(|n| n.checked_div(total)).unwrap_or(0);

  // Build a compact progress bar: "57% [***xx~*xx]"
  // Each symbol is individually colored: * = green, ~ = yellow, x = dark gray.
  let mut spans: Vec<Span<'_>> = Vec::new();
  spans.push(Span::styled(format!("{}% [", pct), Style::default().fg(Color::White)));

  for exercise in exercises {
    let state = config.get_state(&exercise.relative_path);
    let status = derive_status(&state);

    let (ch, style) = match status {
      ExerciseStatus::Complete => ("*", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
      ExerciseStatus::Partial => ("~", Style::default().fg(Color::Yellow)),
      ExerciseStatus::Failing => ("x", Style::default().fg(Color::Red)),
    };

    spans.push(Span::styled(ch.to_string(), style));
  }

  spans.push(Span::styled("]", Style::default().fg(Color::White)));
  spans.push(Span::styled(format!("   Done: {completed}/{total}"), Style::default().fg(Color::White)));

  let lines = vec![Line::from(spans)];
  let block = Block::default().borders(Borders::NONE);
  let paragraph = Paragraph::new(lines).block(block).alignment(Alignment::Center).wrap(Wrap { trim: false });
  frame.render_widget(paragraph, area);
}

// ---------------------------------------------------------------------------
// Tree panel
// ---------------------------------------------------------------------------

/// Approximate visual width of a string in terminal columns.
/// Works correctly for ASCII (1 byte = 1 column) and box-drawing Unicode
/// (multi-byte but 1 column). May be inaccurate for CJK/emoji (unlikely
/// in exercise names).
fn visual_width(s: &str) -> usize {
  s.chars().count()
}

/// Format an exercise line with the tree on the left (padded to a fixed
/// width) and aligned metadata columns (Language, Difficulty, Topics)
/// on the right.
#[allow(clippy::too_many_arguments)]
fn build_exercise_line(
  prefix: &str,
  connector: &str,
  ex: &Exercise,
  symbol: &str,
  stars: &str,
  topics: &str,
  tree_width: usize,
  lang_width: usize,
  diff_width: usize,
  topics_width: usize,
) -> String {
  let tree_part = format!("{}{} {} {}", prefix, connector, symbol, ex.name);
  // Pad by visual width (not byte length) since box-drawing chars are
  // multi-byte but single-column in the terminal.
  let tree_visual = visual_width(&tree_part);
  let padding = tree_width.saturating_sub(tree_visual);
  let tree_padded = format!("{}{}", tree_part, " ".repeat(padding));
  let lang_padded = format!("{:<width$}", ex.language.display_name(), width = lang_width);
  let diff_padded = format!("{:<width$}", stars, width = diff_width);
  let topics_truncated: String = topics.chars().take(topics_width).collect();
  format!("{}   {}   {}   {}", tree_padded, lang_padded, diff_padded, topics_truncated)
}

fn render_tree_panel(
  frame: &mut Frame,
  area: Rect,
  nodes: &[TreeNode],
  exercises: &[Exercise],
  config: &ProjectConfig,
  overview_cursor: &mut usize,
  collapsed_groups: &HashSet<String>,
) -> Vec<LineKind> {
  // Column widths.
  let lang_width = 12usize;
  let diff_width = 10usize;
  let tree_width = 44usize;
  let block_inner = (area.width.saturating_sub(2)) as usize;
  // Separators: tree|lang + lang|diff + diff|topics = 3 each
  let sep_total = 3 + 3 + 3;
  let topics_width = block_inner.saturating_sub(tree_width + lang_width + diff_width + sep_total);

  let has_header = topics_width > 4;

  // Determine which exercise (if any) is under the cursor tree-line.
  let selected_path = exercise_path_at_line(nodes, *overview_cursor, collapsed_groups, has_header);

  let mut lines: Vec<Line<'_>> = Vec::new();
  let mut line_kinds: Vec<LineKind> = Vec::new();

  // Column headers.
  if topics_width > 4 {
    let lang_padded = format!("{:<width$}", "Language", width = lang_width);
    let diff_padded = format!("{:<width$}", "Difficulty", width = diff_width);
    let hdr = format!(
      "{:tree_width$}   {}   {}   Topics",
      "Exercise",
      lang_padded,
      diff_padded,
      tree_width = tree_width,
    );
    lines.push(Line::from(Span::styled(hdr, Style::default().fg(Color::DarkGray))));
    line_kinds.push(LineKind::Blank);
  }

  for node in nodes {
    if let Some(ex) = &node.exercise {
      // Top-level exercise (unusual but handle gracefully)
      let state = config.get_state(&ex.relative_path);
      let status = derive_status(&state);
      let is_selected = selected_path.as_deref() == Some(ex.relative_path.as_str());
      let style = if is_selected {
        Style::default().fg(Color::Rgb(255, 0, 255)).add_modifier(Modifier::BOLD)
      } else {
        match status {
          ExerciseStatus::Complete => Style::default().fg(Color::Green),
          ExerciseStatus::Partial => Style::default().fg(Color::Yellow),
          ExerciseStatus::Failing => Style::default().fg(Color::Red),
        }
      };
      let stars = "*".repeat(ex.difficulty as usize);
      let topics_str = ex.topics.join(", ");
      let line_text = build_exercise_line(
        "  ",
        "",
        ex,
        status.symbol(),
        &stars,
        &topics_str,
        tree_width,
        lang_width,
        diff_width,
        topics_width,
      );
      let ex_idx = exercises
        .iter()
        .position(|e| e.relative_path == ex.relative_path)
        .expect("exercise must be in flat list");
      lines.push(Line::from(Span::styled(line_text, style)));
      line_kinds.push(LineKind::Exercise(ex_idx));
    } else {
      // Top-level group header (no connector)
      let is_collapsed = collapsed_groups.contains(&node.path);
      let icon = if is_collapsed { " ▸" } else { " ▾" };
      let is_cursor_here = lines.len() == *overview_cursor;
      let style = if is_cursor_here {
        Style::default().fg(Color::Rgb(255, 0, 255)).add_modifier(Modifier::BOLD)
      } else {
        Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
      };
      let gs = group_status(node, config);
      lines.push(Line::from(Span::styled(format!("  {} {}/{}", gs.symbol(), node.name, icon), style)));
      line_kinds.push(LineKind::Group(node.path.clone()));

      // Render children recursively (only if not collapsed)
      if !is_collapsed {
        let child_count = node.children.len();
        for (i, child) in node.children.iter().enumerate() {
          render_tree_node(
            child,
            config,
            selected_path.as_deref(),
            "    ",
            i + 1 == child_count,
            &mut lines,
            tree_width,
            lang_width,
            diff_width,
            topics_width,
            collapsed_groups,
            exercises,
            overview_cursor,
            &mut line_kinds,
          );
        }
      }

      // Blank line between top-level groups
      lines.push(Line::from(""));
      line_kinds.push(LineKind::Blank);
    }
  }

  // Clamp cursor in case groups were collapsed/expanded since last frame
  let max_idx = line_kinds.len().saturating_sub(1);
  *overview_cursor = (*overview_cursor).min(max_idx);

  let block = Block::default()
    .title(" Modules ")
    .borders(Borders::ALL)
    .border_style(Style::default().fg(Color::DarkGray));

  let inner_height = block.inner(area).height as usize;
  const MARGIN: usize = 3;
  let max_scroll = lines.len().saturating_sub(inner_height);
  let target = (*overview_cursor + MARGIN + 1).saturating_sub(inner_height);
  let scroll = target.min(max_scroll) as u16;

  let paragraph = Paragraph::new(lines).block(block).scroll((scroll, 0));
  frame.render_widget(paragraph, area);
  line_kinds
}

/// Recursively render a tree node and its children.
#[allow(clippy::too_many_arguments)]
fn render_tree_node(
  node: &TreeNode,
  config: &ProjectConfig,
  selected_path: Option<&str>,
  prefix: &str,
  is_last: bool,
  lines: &mut Vec<Line<'_>>,
  tree_width: usize,
  lang_width: usize,
  diff_width: usize,
  topics_width: usize,
  collapsed_groups: &HashSet<String>,
  exercises: &[Exercise],
  overview_cursor: &mut usize,
  line_kinds: &mut Vec<LineKind>,
) {
  let connector = if is_last { chars::tree_last() } else { chars::tree_branch() };

  if let Some(ex) = &node.exercise {
    let state = config.get_state(&ex.relative_path);
    let status = derive_status(&state);
    let is_selected = selected_path == Some(ex.relative_path.as_str());

    let style = if is_selected {
      Style::default().fg(Color::Rgb(255, 0, 255)).add_modifier(Modifier::BOLD)
    } else {
      match status {
        ExerciseStatus::Complete => Style::default().fg(Color::Green),
        ExerciseStatus::Partial => Style::default().fg(Color::Yellow),
        ExerciseStatus::Failing => Style::default().fg(Color::Red),
      }
    };

    let stars = "*".repeat(ex.difficulty as usize);
    let topics_str = ex.topics.join(", ");
    let line_text = build_exercise_line(
      prefix,
      connector,
      ex,
      status.symbol(),
      &stars,
      &topics_str,
      tree_width,
      lang_width,
      diff_width,
      topics_width,
    );
    let ex_idx = exercises
      .iter()
      .position(|e| e.relative_path == ex.relative_path)
      .expect("exercise must be in flat list");
    lines.push(Line::from(Span::styled(line_text, style)));
    line_kinds.push(LineKind::Exercise(ex_idx));
  } else {
    // Group header
    let is_collapsed = collapsed_groups.contains(&node.path);
    let icon = if is_collapsed { " ▸" } else { " ▾" };
    let is_cursor_here = lines.len() == *overview_cursor;
    let style = if is_cursor_here {
      Style::default().fg(Color::Rgb(255, 0, 255)).add_modifier(Modifier::BOLD)
    } else {
      Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD)
    };
    let gs = group_status(node, config);
    lines.push(Line::from(Span::styled(
      format!("{}{} {} {}/{}", prefix, connector, gs.symbol(), node.name, icon),
      style,
    )));
    line_kinds.push(LineKind::Group(node.path.clone()));

    // Children (only if not collapsed)
    if !is_collapsed {
      let child_prefix = format!("{prefix}{}", if is_last { "  " } else { "│ " });
      let child_count = node.children.len();
      for (i, child) in node.children.iter().enumerate() {
        render_tree_node(
          child,
          config,
          selected_path,
          &child_prefix,
          i + 1 == child_count,
          lines,
          tree_width,
          lang_width,
          diff_width,
          topics_width,
          collapsed_groups,
          exercises,
          overview_cursor,
          line_kinds,
        );
      }
    }
  }
}

/// Given a tree and a tree line index, return the relative path of the
/// exercise at that line (or `None` if the line is a group header, blank,
/// or out of bounds).
fn exercise_path_at_line(nodes: &[TreeNode], target: usize, collapsed_groups: &HashSet<String>, has_header: bool) -> Option<String> {
  let mut line = if has_header { 1 } else { 0 };
  walk_nodes_for_path(nodes, target, &mut line, collapsed_groups, true)
}

/// Recursive helper for [`exercise_path_at_line`].
///
/// `top_level` controls whether blank separators (added by
/// `render_tree_panel` between top-level groups) are counted.
fn walk_nodes_for_path(nodes: &[TreeNode], target: usize, line: &mut usize, collapsed_groups: &HashSet<String>, top_level: bool) -> Option<String> {
  for node in nodes {
    if let Some(ex) = &node.exercise {
      if *line == target {
        return Some(ex.relative_path.clone());
      }
      *line += 1;
    } else {
      // Group header
      if *line == target {
        return None;
      }
      *line += 1;
      // Children (only if not collapsed)
      if !collapsed_groups.contains(&node.path)
        && let Some(path) = walk_nodes_for_path(
          &node.children,
          target,
          line,
          collapsed_groups,
          false, // never top_level — no blank separators between nested groups
        )
      {
        return Some(path);
      }
      // Top-level groups have a blank separator line after them (matching
      // render_tree_panel). Nested groups do not.
      if top_level {
        *line += 1;
      }
    }
  }
  None
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
      hints_shown: 0,
      hints_max: String::new(),
    };
    assert_eq!(derive_status(&state), ExerciseStatus::Failing);
  }

  #[test]
  fn derive_status_partial() {
    let state = ExerciseState {
      best_score: 1.0,
      passed: true,
      solution_seen: false,
      hints_shown: 0,
      hints_max: String::new(),
    };
    assert_eq!(derive_status(&state), ExerciseStatus::Partial);
  }

  #[test]
  fn derive_status_complete() {
    let state = ExerciseState {
      best_score: 1.0,
      passed: true,
      solution_seen: true,
      hints_shown: 0,
      hints_max: String::new(),
    };
    assert_eq!(derive_status(&state), ExerciseStatus::Complete);
  }

  #[test]
  fn derive_status_seen_but_not_passed_is_failing() {
    let state = ExerciseState {
      best_score: 0.3,
      passed: false,
      solution_seen: true,
      hints_shown: 0,
      hints_max: String::new(),
    };
    assert_eq!(derive_status(&state), ExerciseStatus::Failing);
  }
}
