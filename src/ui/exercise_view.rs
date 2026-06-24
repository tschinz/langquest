//! Exercise View - paged display (Theory / Task / Output / Solution).

use std::fs;

// ---------------------------------------------------------------------------
// Scroll-percent helpers
// ---------------------------------------------------------------------------

/// Compute how far through the content we are as a 0–100 integer.
///
/// Returns `None` when the content fits entirely in the viewport (no scrolling
/// is possible), so callers can omit the indicator in that case.
///
/// * `scroll_offset` – current top row (in logical lines / visual rows)
/// * `total_lines`   – total number of lines in the rendered content
/// * `content_height` – visible rows available for content (border already subtracted)
fn scroll_percent(scroll_offset: usize, total_lines: usize, content_height: u16) -> Option<usize> {
  let ch = content_height as usize;
  if total_lines <= ch {
    return None;
  }
  let max_scroll = total_lines.saturating_sub(ch);
  let pct = (scroll_offset.min(max_scroll) * 100) / max_scroll;
  Some(pct)
}

/// Build the block title string, appending `[N%]` when the content scrolls.
fn scroll_title(label: &str, scroll_offset: usize, total_lines: usize, area_height: u16) -> String {
  // TOP border consumes one row; the rest is content.
  let content_height = area_height.saturating_sub(1);
  match scroll_percent(scroll_offset, total_lines, content_height) {
    Some(pct) => format!("{label} [{pct}%]"),
    None => label.to_owned(),
  }
}

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

// ── Top bar colours (match bottom status bar palette) ─────────────────────────
const TOPBAR_BG: Color = Color::Rgb(22, 22, 36);
const TOPBAR_NAME_FG: Color = Color::White;
const TOPBAR_PATH_FG: Color = Color::Rgb(55, 55, 72);
const TOPBAR_SEP_FG: Color = Color::Rgb(55, 55, 72);

use crate::ui::cache::{CachedContent, ContentType, RenderCache};
use crate::ui::markdown::{LinkSpan, PendingOsc8, parse_markdown_with_links};
use crate::ui::term_caps::chars;

use crate::app::{App, ExercisePage};
use crate::config::ExerciseState;
use crate::exercise::Exercise;
use crate::runner::VerificationResult;

// ---------------------------------------------------------------------------
// Bundled parameters to avoid too-many-arguments clippy lint
// ---------------------------------------------------------------------------

/// Groups the rendering parameters that the exercise view needs, avoiding
/// clippy's `too_many_arguments` lint on the internal helpers.
struct ViewParams<'a> {
  exercise: &'a Exercise,
  page: ExercisePage,
  hints_revealed: usize,
  solution_unlock_pending: bool,
  last_result: Option<&'a VerificationResult>,
  /// Output from running the exercise's `main()` (empty if unavailable).
  last_main_output: &'a str,
  config_state: ExerciseState,
  scroll_offset: usize,
}

// ---------------------------------------------------------------------------
// Public entry point (matches call-site in App::render)
// ---------------------------------------------------------------------------

/// Render the exercise view into the given frame.
pub fn render(app: &mut App, frame: &mut Frame, area: Rect) -> Option<PendingOsc8> {
  let exercise = app.current_exercise().clone();
  let page = app.page;
  let hints_revealed = app.hints_revealed;
  let solution_unlock_pending = app.solution_unlock_pending;
  let last_result = app.last_result.as_ref();
  let last_main_output = app.last_main_output.as_str();
  let config_state = app.config.get_state(&exercise.relative_path);
  let scroll_offset = app.scroll_offset;

  let params = ViewParams {
    exercise: &exercise,
    page,
    hints_revealed,
    solution_unlock_pending,
    last_result,
    last_main_output,
    config_state,
    scroll_offset,
  };

  let (pending, content_height, viewport_height) = render_exercise(frame, area, &params, &mut app.render_cache);

  // Update app with content dimensions for scroll limiting
  app.content_height = content_height;
  app.viewport_height = viewport_height;

  pending
}

// ---------------------------------------------------------------------------
// Core rendering
// ---------------------------------------------------------------------------

/// Render the exercise view with bundled parameters.
/// Returns (pending_osc8, content_height, viewport_height) for scroll limiting.
fn render_exercise(frame: &mut Frame, area: Rect, params: &ViewParams<'_>, cache: &mut RenderCache) -> (Option<PendingOsc8>, usize, usize) {
  let rows = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(1), Constraint::Min(1)])
    .split(area);

  render_topbar(frame, rows[0], params.exercise);
  render_content(frame, rows[1], params, cache)
}

/// Render the 1-line top bar showing the exercise path and name.
fn render_topbar(frame: &mut Frame, area: Rect, exercise: &Exercise) {
  // Use term_caps for cross-platform separator character
  let sep = format!("     {}     ", chars::vertical());
  let spans = vec![
    Span::raw("  "),
    Span::styled(exercise.relative_path.clone(), Style::default().fg(TOPBAR_PATH_FG)),
    Span::styled(sep, Style::default().fg(TOPBAR_SEP_FG)),
    Span::styled(exercise.name.clone(), Style::default().fg(TOPBAR_NAME_FG).add_modifier(Modifier::BOLD)),
  ];

  frame.render_widget(Paragraph::new(Line::from(spans)).style(Style::default().bg(TOPBAR_BG)), area);
}

// ---------------------------------------------------------------------------
// Content dispatcher
// ---------------------------------------------------------------------------

/// Returns (pending_osc8, content_height, viewport_height) for scroll limiting.
fn render_content(frame: &mut Frame, area: Rect, params: &ViewParams<'_>, cache: &mut RenderCache) -> (Option<PendingOsc8>, usize, usize) {
  // Viewport height is the area height minus the border (1 line for top border)
  let viewport_height = area.height.saturating_sub(1) as usize;

  match params.page {
    ExercisePage::Theory => {
      let (pending, content_height) = render_theory(frame, area, params.exercise, params.scroll_offset, cache);
      (pending, content_height, viewport_height)
    }
    ExercisePage::Task => {
      let (pending, content_height) = render_task(frame, area, params.exercise, params.scroll_offset, cache);
      (pending, content_height, viewport_height)
    }
    ExercisePage::Debug => {
      let content_height = render_debug(frame, area, params.last_main_output, params.scroll_offset);
      (None, content_height, viewport_height)
    }
    ExercisePage::Output => {
      let content_height = render_output(
        frame,
        area,
        params.exercise,
        params.hints_revealed,
        params.solution_unlock_pending,
        params.last_result,
        params.scroll_offset,
      );
      (None, content_height, viewport_height)
    }
    ExercisePage::Solution => {
      let (pending, content_height) = render_solution_page(frame, area, params.exercise, &params.config_state, params.scroll_offset, cache);
      (pending, content_height, viewport_height)
    }
  }
}

// ---------------------------------------------------------------------------
// Page 1: Theory
// ---------------------------------------------------------------------------

/// Returns (pending_osc8, content_line_count) for scroll limiting.
fn render_theory(frame: &mut Frame, area: Rect, exercise: &Exercise, scroll_offset: usize, cache: &mut RenderCache) -> (Option<PendingOsc8>, usize) {
  let cache_key = RenderCache::make_key(&exercise.relative_path, ContentType::Theory, area.width);
  let (lines, links) = if let Some(cached) = cache.get_cached(&cache_key) {
    (cached.lines.clone(), cached.links.clone())
  } else {
    let content = match exercise.theory_path {
      Some(ref path) => match fs::read_to_string(path) {
        Ok(text) => text.replace("\r\n", "\n"),
        Err(e) => format!("Error reading theory file: {e}"),
      },
      None => "No theory available.".to_string(),
    };
    let (lines, links) = parse_markdown_with_links(&content, area.width);
    cache.insert(cache_key, CachedContent::new(lines.clone(), links.clone(), &content));
    (lines, links)
  };
  let content_height = lines.len();
  let title = scroll_title("Theory", scroll_offset, content_height, area.height);

  let block = Block::default().borders(Borders::TOP).title(title);
  let inner = block.inner(area);
  let paragraph = Paragraph::new(lines).block(block).scroll((scroll_offset as u16, 0)).wrap(Wrap { trim: false });

  frame.render_widget(paragraph, area);
  (
    Some(PendingOsc8 {
      area: inner,
      scroll: scroll_offset,
      links,
    }),
    content_height,
  )
}

// ---------------------------------------------------------------------------
// Page 2: Task
// ---------------------------------------------------------------------------

/// Returns (pending_osc8, content_line_count) for scroll limiting.
fn render_task(frame: &mut Frame, area: Rect, exercise: &Exercise, scroll_offset: usize, cache: &mut RenderCache) -> (Option<PendingOsc8>, usize) {
  let cache_key = RenderCache::make_key(&exercise.relative_path, ContentType::Task, area.width);
  let (lines, links) = if let Some(cached) = cache.get_cached(&cache_key) {
    (cached.lines.clone(), cached.links.clone())
  } else {
    let content = match fs::read_to_string(&exercise.task_path) {
      Ok(text) => {
        let stripped = strip_frontmatter(&text);
        stripped.replace("\r\n", "\n")
      }
      Err(e) => format!("Error reading task file: {e}"),
    };
    let (lines, links) = parse_markdown_with_links(&content, area.width);
    cache.insert(cache_key, CachedContent::new(lines.clone(), links.clone(), &content));
    (lines, links)
  };
  let content_height = lines.len();
  let title = scroll_title("Task", scroll_offset, content_height, area.height);

  let block = Block::default().borders(Borders::TOP).title(title);
  let inner = block.inner(area);
  let paragraph = Paragraph::new(lines).block(block).scroll((scroll_offset as u16, 0)).wrap(Wrap { trim: false });

  frame.render_widget(paragraph, area);
  (
    Some(PendingOsc8 {
      area: inner,
      scroll: scroll_offset,
      links,
    }),
    content_height,
  )
}

// ---------------------------------------------------------------------------
// Page 3: Debug (main() output)
// ---------------------------------------------------------------------------

/// Render the output of the exercise's `main()` in the dedicated "Debug" page.
///
/// Shows a placeholder message when no main output is available (the source
/// doesn't have a `fn main` or can't be compiled as a regular binary).
fn render_debug(frame: &mut Frame, area: Rect, main_output: &str, scroll_offset: usize) -> usize {
  let lines: Vec<Line> = if main_output.is_empty() {
    vec![Line::from(Span::styled(
      "No main() output — the exercise either doesn't have a fn main or it could not be compiled as a regular binary.",
      Style::default().fg(Color::DarkGray),
    ))]
  } else {
    main_output.lines().map(|line| Line::from(Span::raw(line.to_string()))).collect()
  };

  let content_height = lines.len();
  let title = scroll_title("Debug", scroll_offset, content_height, area.height);

  let paragraph = Paragraph::new(lines)
    .block(Block::default().borders(Borders::TOP).title(title))
    .scroll((scroll_offset as u16, 0))
    .wrap(Wrap { trim: false });

  frame.render_widget(paragraph, area);
  content_height
}

// ---------------------------------------------------------------------------
// Page 4: Output (test results)
// ---------------------------------------------------------------------------

fn render_output(
  frame: &mut Frame,
  area: Rect,
  exercise: &Exercise,
  hints_revealed: usize,
  solution_unlock_pending: bool,
  last_result: Option<&VerificationResult>,
  scroll_offset: usize,
) -> usize {
  let lines = match last_result {
    Some(result) => build_output_lines(exercise, hints_revealed, solution_unlock_pending, result, area.width as usize),
    None => vec![Line::from("No verification result yet. Save your file to trigger verification.")],
  };

  let content_height = lines.len();
  let title = scroll_title("Output", scroll_offset, content_height, area.height);

  let paragraph = Paragraph::new(lines)
    .block(Block::default().borders(Borders::TOP).title(title))
    .scroll((scroll_offset as u16, 0))
    .wrap(Wrap { trim: false });

  frame.render_widget(paragraph, area);
  content_height
}

/// Strip markdown code-fence lines (```, ```rust, ```python, etc.) from
/// the start and end of a hint string, since they are useless in plain-text
/// output and just clutter the display.
pub(crate) fn strip_code_fences(text: &str) -> String {
  let mut lines: Vec<&str> = text.lines().collect();
  // Strip leading fence lines
  while let Some(first) = lines.first() {
    let trimmed = first.trim();
    if trimmed.starts_with("```") {
      lines.remove(0);
    } else {
      break;
    }
  }
  // Strip trailing fence lines
  while let Some(last) = lines.last() {
    let trimmed = last.trim();
    if trimmed.starts_with("```") {
      lines.pop();
    } else {
      break;
    }
  }
  lines.join("\n")
}

/// Word-wrap a single line at `max_width` characters, returning multiple
/// lines. This prevents ratatui from wrapping continuation lines without
/// the proper indent.
pub(crate) fn wrap_line(text: &str, max_width: usize) -> Vec<String> {
  if text.len() <= max_width || max_width < 10 {
    return vec![text.to_string()];
  }
  let mut result = Vec::new();
  let mut remaining = text;
  while !remaining.is_empty() {
    if remaining.len() <= max_width {
      result.push(remaining.to_string());
      break;
    }
    // Find the last whitespace before max_width to break at
    let break_at = remaining[..max_width].rfind(|c: char| c.is_whitespace()).unwrap_or(max_width);
    if break_at == 0 {
      // No whitespace found, hard-break at max_width
      result.push(remaining[..max_width].to_string());
      remaining = &remaining[max_width..];
    } else {
      result.push(remaining[..break_at].to_string());
      remaining = remaining[break_at..].trim_start();
    }
  }
  result
}

fn build_output_lines<'a>(
  exercise: &'a Exercise,
  hints_revealed: usize,
  solution_unlock_pending: bool,
  result: &'a VerificationResult,
  area_width: usize,
) -> Vec<Line<'a>> {
  // Hint lines are indented by 4 spaces; subtract that plus a safety margin.
  let hint_width = area_width.saturating_sub(6);
  let mut lines: Vec<Line<'a>> = Vec::new();

  // Progress bar
  let bar = result.progress_bar(30);
  lines.push(Line::from(bar));

  // Status line
  if result.score >= result.threshold {
    lines.push(Line::from(Span::styled(
      "PASSED ✓",
      Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
    )));
  } else {
    lines.push(Line::from(Span::styled(
      "FAILING ✗",
      Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
    )));
  }

  // Blank line
  lines.push(Line::from(""));

  // Runner output — pre-wrapped so lines.len() reflects visual row count
  for line in result.output.lines() {
    for wrapped in wrap_line(line, area_width) {
      lines.push(Line::from(wrapped));
    }
  }

  // Blank line before hints
  lines.push(Line::from(""));

  // Hints section
  if let Some(ref solution_data) = exercise.solution_data {
    let total_hints = solution_data.hints.len();
    let reveal_count = hints_revealed.min(total_hints);

    // Show regular hints — split by newlines so code blocks display properly.
    for i in 0..reveal_count {
      let raw_hint = solution_data.hints.get(i).cloned().unwrap_or_default();
      let hint_text = strip_code_fences(&raw_hint);
      // Header line: [HINT N/M]
      lines.push(Line::from(Span::styled(
        format!("[HINT {}/{}]", i + 1, total_hints),
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
      )));
      // Content lines: indented with 4 spaces
      for line_text in hint_text.lines() {
        for wrapped in wrap_line(line_text, hint_width) {
          lines.push(Line::from(Span::styled(format!("    {}", wrapped), Style::default().fg(Color::Yellow))));
        }
      }
    }

    if reveal_count < total_hints {
      lines.push(Line::from(""));
      lines.push(Line::from(Span::styled("Press 'h' to reveal next hint", Style::default().fg(Color::Cyan))));
    } else if solution_unlock_pending {
      lines.push(Line::from(""));
      lines.push(Line::from(Span::styled(
        "⚠  The solution will be unlocked. Are you really sure?",
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
      )));
      lines.push(Line::from(Span::styled(
        "   Press 'h' again to confirm, or any other key to cancel.",
        Style::default().fg(Color::Yellow),
      )));
    } else {
      lines.push(Line::from(""));
      lines.push(Line::from(Span::styled(
        "No more hints. Press 'h' to unlock the solution.",
        Style::default().fg(Color::Cyan),
      )));
    }
  }

  lines
}

// ---------------------------------------------------------------------------
// Page 4: Solution
// ---------------------------------------------------------------------------

/// Returns (pending_osc8, content_line_count) for scroll limiting.
fn render_solution_page(
  frame: &mut Frame,
  area: Rect,
  exercise: &Exercise,
  config_state: &ExerciseState,
  scroll_offset: usize,
  cache: &mut RenderCache,
) -> (Option<PendingOsc8>, usize) {
  let solution_accessible = config_state.passed || config_state.solution_seen;

  if !solution_accessible {
    let paragraph = Paragraph::new("Complete the exercise to unlock the solution.")
      .block(Block::default().borders(Borders::TOP).title("Solution"))
      .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
    return (None, 1);
  }

  // Try cache first — invalidated by file watcher or exercise switch, not per-frame
  let cache_key = RenderCache::make_key(&exercise.relative_path, ContentType::Solution, area.width);
  if let Some(cached) = cache.get_cached(&cache_key) {
    let content_height = cached.lines.len();
    let title = scroll_title("Solution", scroll_offset, content_height, area.height);
    let block = Block::default().borders(Borders::TOP).title(title);
    let inner = block.inner(area);
    let paragraph = Paragraph::new(cached.lines.clone())
      .block(block)
      .scroll((scroll_offset as u16, 0))
      .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
    return (
      Some(PendingOsc8 {
        area: inner,
        scroll: scroll_offset,
        links: cached.links.clone(),
      }),
      content_height,
    );
  }

  // Cache miss - build content
  let mut lines: Vec<Line<'_>> = Vec::new();
  let mut solution_links: Vec<LinkSpan> = Vec::new();
  let mut content_for_hash = String::new();

  // Source code from solution file
  if let Some(ref solution_path) = exercise.solution_source {
    match fs::read_to_string(solution_path) {
      Ok(source) => {
        content_for_hash.push_str(&source);
        let source = source.replace("\r\n", "\n");
        // Use term_caps for cross-platform separator characters
        let sep_char = chars::horizontal();
        lines.push(Line::from(Span::styled(
          format!("{0}{0} Reference Solution {0}{0}", sep_char),
          Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )));
        lines.push(Line::from(""));
        let token = exercise.language.syntax_token();
        lines.extend(crate::ui::markdown::highlight_code_block(&source, token));
      }
      Err(e) => {
        lines.push(Line::from(format!("Error reading solution source: {e}")));
      }
    }
  }

  // Explanation from solution_data
  if let Some(ref sd) = exercise.solution_data
    && !sd.explanation.is_empty()
  {
    content_for_hash.push_str(&sd.explanation);
    solution_links = append_explanation(&mut lines, sd, area.width);
  }

  if lines.is_empty() {
    lines.push(Line::from("No solution content available."));
  }

  // Store in cache
  let content_height = lines.len();
  cache.insert(cache_key, CachedContent::new(lines.clone(), solution_links.clone(), &content_for_hash));

  let title = scroll_title("Solution", scroll_offset, content_height, area.height);

  let block = Block::default().borders(Borders::TOP).title(title);
  let inner = block.inner(area);
  let paragraph = Paragraph::new(lines).block(block).scroll((scroll_offset as u16, 0)).wrap(Wrap { trim: false });

  frame.render_widget(paragraph, area);

  (
    Some(PendingOsc8 {
      area: inner,
      scroll: scroll_offset,
      links: solution_links,
    }),
    content_height,
  )
}

/// Append the explanation section from `SolutionData` into `lines`.
fn append_explanation(lines: &mut Vec<Line<'static>>, solution_data: &crate::exercise::SolutionData, width: u16) -> Vec<LinkSpan> {
  if !lines.is_empty() {
    lines.push(Line::from(""));
    // Use term_caps for cross-platform separator characters
    let sep_char = chars::horizontal();
    lines.push(Line::from(Span::styled(sep_char.repeat(24), Style::default().add_modifier(Modifier::DIM))));
    lines.push(Line::from(""));
  }
  lines.push(Line::from(Span::styled(
    "Explanation",
    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
  )));
  lines.push(Line::from(""));
  // Record where markdown content begins so link line indices can be offset.
  let md_line_offset = lines.len();
  let (md_lines, mut links) = parse_markdown_with_links(&solution_data.explanation, width);
  lines.extend(md_lines);
  // Shift each link's line_idx to account for the header lines above.
  for link in &mut links {
    link.line_idx += md_line_offset;
  }
  links
}

// ---------------------------------------------------------------------------
// OSC 8 terminal hyperlinks
// ---------------------------------------------------------------------------

// Overwrite the ratatui buffer cells that correspond to rendered hyperlinks
// with OSC 8 escape sequences, making them clickable in terminals that
// support the protocol (iTerm2, kitty, WezTerm, GNOME Terminal ≥ 3.26, …).

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Strip `---` delimited TOML frontmatter from the beginning of content.
///
/// If the content starts with `---\n`, everything up to and including the next
/// `---\n` line (or `---` at EOF) is removed.  If no closing delimiter is
/// found the content is returned unchanged.
fn strip_frontmatter(content: &str) -> &str {
  // Strip optional BOM then check for opening ---
  let trimmed = content.trim_start_matches('\u{feff}');
  if !trimmed.starts_with("---") {
    return content;
  }

  // After the opening "---", require a newline.
  let after_dashes = &trimmed[3..];
  let after_opening = if let Some(rest) = after_dashes.strip_prefix("\r\n") {
    rest
  } else if let Some(rest) = after_dashes.strip_prefix('\n') {
    rest
  } else {
    // "---" followed by other chars or nothing - not valid frontmatter.
    return content;
  };

  // Search for the closing "---" on its own line.
  let mut byte_offset = 0;
  for line in after_opening.lines() {
    let line_end = byte_offset + line.len();
    if line.trim() == "---" {
      let rest = &after_opening[line_end..];
      // Skip the newline after the closing ---
      if let Some(stripped) = rest.strip_prefix("\r\n") {
        return stripped;
      } else if let Some(stripped) = rest.strip_prefix('\n') {
        return stripped;
      }
      return rest;
    }
    // Advance past the line content plus the newline character(s).
    byte_offset = line_end;
    if after_opening.get(byte_offset..byte_offset + 2) == Some("\r\n") {
      byte_offset += 2;
    } else if after_opening.get(byte_offset..byte_offset + 1) == Some("\n") {
      byte_offset += 1;
    }
  }

  // No closing delimiter found - return content as-is.
  content
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn scroll_percent_no_scroll_needed() {
    // content fits - no indicator
    assert_eq!(scroll_percent(0, 10, 20), None);
    assert_eq!(scroll_percent(0, 20, 20), None);
  }

  #[test]
  fn scroll_percent_at_top() {
    assert_eq!(scroll_percent(0, 40, 20), Some(0));
  }

  #[test]
  fn scroll_percent_at_bottom() {
    assert_eq!(scroll_percent(20, 40, 20), Some(100));
  }

  #[test]
  fn scroll_percent_midpoint() {
    assert_eq!(scroll_percent(10, 40, 20), Some(50));
  }

  #[test]
  fn scroll_percent_clamped_past_max() {
    // scroll_offset beyond max_scroll is clamped to 100 %
    assert_eq!(scroll_percent(999, 40, 20), Some(100));
  }

  #[test]
  fn scroll_title_no_overflow_returns_bare_label() {
    assert_eq!(scroll_title("Theory", 0, 5, 20), "Theory");
  }

  #[test]
  fn scroll_title_overflow_appends_percent() {
    // area_height=21, content_height=20, total=40 → same as midpoint above
    assert_eq!(scroll_title("Theory", 10, 40, 21), "Theory [50%]");
  }

  #[test]
  fn scroll_title_at_top_shows_zero() {
    assert_eq!(scroll_title("Output", 0, 40, 21), "Output [0%]");
  }

  #[test]
  fn scroll_title_at_bottom_shows_hundred() {
    assert_eq!(scroll_title("Solution", 20, 40, 21), "Solution [100%]");
  }

  #[test]
  fn strip_frontmatter_basic() {
    let input = "---\ntitle: Hello\n---\nBody text here";
    assert_eq!(strip_frontmatter(input), "Body text here");
  }

  #[test]
  fn strip_frontmatter_no_frontmatter() {
    let input = "Just some text\nwith no frontmatter";
    assert_eq!(strip_frontmatter(input), input);
  }

  #[test]
  fn strip_frontmatter_empty() {
    assert_eq!(strip_frontmatter(""), "");
  }

  #[test]
  fn strip_frontmatter_only_opening() {
    let input = "---\ntitle: Hello\nno closing";
    assert_eq!(strip_frontmatter(input), input);
  }

  #[test]
  fn strip_frontmatter_with_bom() {
    let input = "\u{feff}---\nkey: val\n---\nContent";
    assert_eq!(strip_frontmatter(input), "Content");
  }

  #[test]
  fn strip_frontmatter_multiline_body() {
    let input = "---\nid: test\nname: Test\n---\nLine 1\nLine 2\nLine 3";
    assert_eq!(strip_frontmatter(input), "Line 1\nLine 2\nLine 3");
  }

  #[test]
  fn page_index_values() {
    assert_eq!(ExercisePage::Theory.index(), 0);
    assert_eq!(ExercisePage::Task.index(), 1);
    assert_eq!(ExercisePage::Debug.index(), 2);
    assert_eq!(ExercisePage::Output.index(), 3);
    assert_eq!(ExercisePage::Solution.index(), 4);
  }

  #[test]
  fn page_from_index_roundtrip() {
    assert_eq!(ExercisePage::from_index(0), ExercisePage::Theory);
    assert_eq!(ExercisePage::from_index(1), ExercisePage::Task);
    assert_eq!(ExercisePage::from_index(2), ExercisePage::Debug);
    assert_eq!(ExercisePage::from_index(3), ExercisePage::Output);
    assert_eq!(ExercisePage::from_index(4), ExercisePage::Solution);
    // Wraps around
    assert_eq!(ExercisePage::from_index(5), ExercisePage::Theory);
  }

  #[test]
  fn page_label_values() {
    assert_eq!(ExercisePage::Theory.label(), "Theory");
    assert_eq!(ExercisePage::Task.label(), "Task");
    assert_eq!(ExercisePage::Debug.label(), "Debug");
    assert_eq!(ExercisePage::Output.label(), "Output");
    assert_eq!(ExercisePage::Solution.label(), "Solution");
  }
}
