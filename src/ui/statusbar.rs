//! Context-aware bottom status bar.
//!
//! Two visual states:
//!
//! * **Expanded** (2 rows) – row 1 shows the active top-level view + sub-view
//!   tabs; row 2 shows keyboard shortcut hints.
//! * **Collapsed** (1 row) – shows only a small `m menu` prompt.
//!
//! Toggled by pressing `m` in the app.

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use super::term_caps::chars;
use crate::app::{ExercisePage, View};

// ── Heights ───────────────────────────────────────────────────────────────────

/// Terminal lines consumed when the bar is fully expanded.
pub const EXPANDED_HEIGHT: u16 = 2;

/// Terminal lines consumed when the bar is collapsed.
pub const COLLAPSED_HEIGHT: u16 = 1;

// ── Colour palette ────────────────────────────────────────────────────────────

/// Background for the tabs row.
const BG_TABS: Color = Color::Rgb(22, 22, 36);

/// Background for the hints row (slightly darker than tabs).
const BG_HINTS: Color = Color::Rgb(15, 15, 26);

/// Foreground for the active tab (consistent with inline-code orange).
const ACTIVE_FG: Color = Color::Rgb(255, 165, 0);

/// Foreground for an inactive top-level view tab.
const INACTIVE_TOP_FG: Color = Color::DarkGray;

/// Foreground for an inactive sub-view tab.
const INACTIVE_SUB_FG: Color = Color::White;

/// Foreground for key-badge text (consistent with link light-blue).
const KEY_FG: Color = Color::Rgb(130, 205, 255);

/// Foreground for hint descriptions.
const DESC_FG: Color = Color::Rgb(160, 160, 175);

/// Foreground for separator decorations.
const SEP_FG: Color = Color::Rgb(55, 55, 72);

// ── Public API ────────────────────────────────────────────────────────────────

/// Render the fully expanded 2-row status bar.
///
/// `area` must be at least 2 lines tall; if it is only 1 line the collapsed
/// bar is rendered instead.
pub fn render(frame: &mut Frame, area: Rect, view: View, page: ExercisePage, show_tree: bool, solution_accessible: bool) {
  if area.height < EXPANDED_HEIGHT {
    render_collapsed(frame, area);
    return;
  }

  let rows = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(1), Constraint::Length(1)])
    .split(area);

  render_tabs_row(frame, rows[0], view, page, show_tree, solution_accessible);
  render_hints_row(frame, rows[1], view);
}

/// Render the collapsed 1-row indicator.
pub fn render_collapsed(frame: &mut Frame, area: Rect) {
  let line = Line::from(vec![
    Span::styled(" m ", Style::default().fg(Color::Rgb(22, 22, 36)).bg(KEY_FG).add_modifier(Modifier::BOLD)),
    Span::styled(" menu", Style::default().fg(DESC_FG)),
  ]);
  frame.render_widget(Paragraph::new(line).style(Style::default().bg(BG_HINTS)).alignment(Alignment::Right), area);
}

// ── Row 1: view + sub-view tabs ───────────────────────────────────────────────

fn render_tabs_row(frame: &mut Frame, area: Rect, view: View, page: ExercisePage, show_tree: bool, solution_accessible: bool) {
  // Use term_caps for cross-platform separator characters
  let use_unicode = chars::vertical() == "│";
  let slash_sep: &'static str = if use_unicode { "  ╱  " } else { "  /  " };
  let vert_sep: &'static str = if use_unicode { "     │     " } else { "     |     " };

  let mut spans: Vec<Span<'static>> = vec![
    Span::raw("  "),
    top_tab("Exercise", view == View::ExerciseView),
    dim_sep(slash_sep),
    top_tab("Overview", view == View::Overview),
    dim_sep(slash_sep),
    top_tab("About", view == View::About),
  ];

  // ── Vertical rule (hidden on the About page which has no sub-tabs) ─────────
  if view != View::About {
    spans.push(dim_sep(vert_sep));
  }

  // ── Sub-view tabs (context-sensitive) ─────────────────────────────────────
  match view {
    View::ExerciseView => {
      const PAGES: &[ExercisePage] = &[ExercisePage::Theory, ExercisePage::Task, ExercisePage::Output, ExercisePage::Solution];
      for (i, p) in PAGES.iter().enumerate() {
        if i > 0 {
          spans.push(dim_sep("  ·  "));
        }
        let label = p.label();
        let is_solution = *p == ExercisePage::Solution;
        if is_solution && !solution_accessible {
          // Locked: show dimmed label with lock prefix.
          let active = *p == page;
          let style = if active {
            Style::default().fg(ACTIVE_FG).add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
          } else {
            Style::default().fg(SEP_FG)
          };
          spans.push(Span::styled(format!("🔒 {label}"), style));
        } else {
          spans.push(sub_tab(label, *p == page));
        }
      }
    }

    View::Overview => {
      spans.push(sub_tab("Table", !show_tree));
      spans.push(dim_sep("  ·  "));
      spans.push(sub_tab("Tree", show_tree));
    }

    View::About => {}
  }

  frame.render_widget(Paragraph::new(Line::from(spans)).style(Style::default().bg(BG_TABS)), area);
}

// ── Row 2: keyboard hint badges ───────────────────────────────────────────────

fn render_hints_row(frame: &mut Frame, area: Rect, view: View) {
  // Use term_caps for cross-platform arrow characters
  let (left_right, up_down) = if chars::vertical() == "│" {
    ("← →", "↑ ↓")
  } else {
    ("<- ->", "^ v")
  };

  // Build hints dynamically to support cross-platform arrow characters
  let hints: Vec<(&str, &str)> = match view {
    View::ExerciseView => vec![
      (left_right, "page"),
      (up_down, "scroll"),
      ("j / k", "exercise"),
      ("e", "edit"),
      ("h", "hint"),
      ("o", "overview"),
      ("a", "about"),
      ("q", "quit"),
      ("m", "menu"),
    ],
    View::Overview => vec![
      (up_down, "navigate"),
      ("Enter", "open"),
      ("t", "tree"),
      ("o", "exercise"),
      ("a", "about"),
      ("q", "quit"),
      ("m", "menu"),
    ],
    View::About => vec![(up_down, "scroll"), ("a", "back"), ("q", "quit"), ("m", "menu")],
  };

  let mut spans: Vec<Span<'static>> = Vec::new();
  spans.push(Span::raw("  "));

  for (i, (key, desc)) in hints.iter().enumerate() {
    if i > 0 {
      spans.push(Span::raw("    "));
    }
    // Key badge: coloured foreground on dark background, bold.
    spans.push(Span::styled(
      format!(" {key} "),
      Style::default().fg(KEY_FG).bg(Color::Rgb(35, 35, 52)).add_modifier(Modifier::BOLD),
    ));
    // Description text.
    spans.push(Span::styled(format!(" {desc}"), Style::default().fg(DESC_FG)));
  }

  frame.render_widget(Paragraph::new(Line::from(spans)).style(Style::default().bg(BG_HINTS)), area);
}

// ── Span helpers ──────────────────────────────────────────────────────────────

/// Top-level view tab (Exercise / Overview).
fn top_tab(label: &'static str, active: bool) -> Span<'static> {
  if active {
    Span::styled(label, Style::default().fg(ACTIVE_FG).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
  } else {
    Span::styled(label, Style::default().fg(INACTIVE_TOP_FG))
  }
}

/// Sub-view tab (Theory/Task/Output/Solution or Table/Tree).
fn sub_tab(label: &'static str, active: bool) -> Span<'static> {
  if active {
    Span::styled(label, Style::default().fg(ACTIVE_FG).add_modifier(Modifier::BOLD | Modifier::UNDERLINED))
  } else {
    Span::styled(label, Style::default().fg(INACTIVE_SUB_FG))
  }
}

/// Dim decorative separator between tabs.
fn dim_sep(s: &'static str) -> Span<'static> {
  Span::styled(s, Style::default().fg(SEP_FG))
}
