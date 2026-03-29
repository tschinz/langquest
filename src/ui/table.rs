//! Reusable fixed-width table renderer.
//!
//! Provides [`TableData`] (columns + rows) and [`render_table`] which draws a
//! header row, a `─` separator, and scrollable data rows inside any
//! [`ratatui::layout::Rect`].

use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

/// A single column definition.
pub struct Column {
  /// Text shown in the header row.
  pub header: String,
  /// Fixed display width (in terminal columns).
  pub width: u16,
}

/// Column definitions together with row data, ready to be rendered.
pub struct TableData {
  /// Ordered column definitions.
  pub columns: Vec<Column>,
  /// Row data - each inner `Vec` must have the same length as `columns`.
  pub rows: Vec<Vec<String>>,
}

/// Truncate `text` so that its display length is at most `max_width`.
///
/// If the text is longer it is trimmed to `max_width - 2` characters and `..`
/// is appended.  When `max_width < 3` the text is simply truncated without a
/// suffix (there is not enough room for the dots to be useful).
fn truncate(text: &str, max_width: u16) -> String {
  let max = max_width as usize;
  if text.len() <= max {
    return text.to_string();
  }
  if max < 3 {
    return text.chars().take(max).collect();
  }
  let mut s: String = text.chars().take(max - 2).collect();
  s.push_str("..");
  s
}

/// Build a single [`Line`] from `cells` using the widths defined in `columns`.
///
/// Each cell is truncated and then right-padded to exactly `column.width`
/// characters so that columns stay aligned.
fn build_row_line<'a>(columns: &[Column], cells: &[String], style: Style) -> Line<'a> {
  let mut spans: Vec<Span<'a>> = Vec::with_capacity(columns.len());
  for (i, col) in columns.iter().enumerate() {
    let raw = cells.get(i).map_or("", String::as_str);
    let truncated = truncate(raw, col.width);
    let padded = format!("{:<width$}", truncated, width = col.width as usize);
    spans.push(Span::styled(padded, style));
    // One-space gap between columns (except after the last one).
    if i + 1 < columns.len() {
      spans.push(Span::raw(" "));
    }
  }
  Line::from(spans)
}

/// Render a fixed-width table into `area`.
///
/// *   The first line is the **header** row styled with `header_style`.
/// *   The second line is a `─` separator spanning the full width.
/// *   Remaining lines are data rows; the row at position `cursor` is drawn
///     with `highlight_style`, all others with `Style::default()`.
/// *   If the data rows do not fit, the view scrolls so that the cursor row
///     remains visible.
pub fn render_table(frame: &mut Frame, area: Rect, data: &TableData, cursor: usize, header_style: Style, highlight_style: Style) {
  if area.height == 0 || area.width == 0 {
    return;
  }

  let mut lines: Vec<Line<'_>> = Vec::new();

  // --- header ---------------------------------------------------------
  let header_cells: Vec<String> = data.columns.iter().map(|c| c.header.clone()).collect();
  lines.push(build_row_line(&data.columns, &header_cells, header_style));

  // --- separator ------------------------------------------------------
  let sep: String = "─".repeat(area.width as usize);
  lines.push(Line::from(Span::styled(sep, Style::default().fg(Color::DarkGray))));

  // The number of lines already consumed by header + separator.
  let chrome_lines: u16 = 2;

  // How many data rows can actually be displayed.
  let visible_rows = area.height.saturating_sub(chrome_lines) as usize;

  if visible_rows == 0 {
    // Only enough room for the header and separator.
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
    return;
  }

  // --- scrolling -------------------------------------------------------
  let total_rows = data.rows.len();
  let scroll_offset = if total_rows <= visible_rows || cursor < visible_rows / 2 {
    0
  } else if cursor + visible_rows / 2 >= total_rows {
    total_rows.saturating_sub(visible_rows)
  } else {
    cursor.saturating_sub(visible_rows / 2)
  };

  let end = (scroll_offset + visible_rows).min(total_rows);

  // --- data rows -------------------------------------------------------
  for (idx, row) in data.rows.iter().enumerate().skip(scroll_offset).take(end - scroll_offset) {
    let style = if idx == cursor { highlight_style } else { Style::default() };
    lines.push(build_row_line(&data.columns, row, style));
  }

  let paragraph = Paragraph::new(lines);
  frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn truncate_short_text_unchanged() {
    assert_eq!(truncate("hi", 10), "hi");
  }

  #[test]
  fn truncate_exact_width() {
    assert_eq!(truncate("abcde", 5), "abcde");
  }

  #[test]
  fn truncate_long_text_adds_dots() {
    assert_eq!(truncate("abcdefgh", 6), "abcd..");
  }

  #[test]
  fn truncate_very_narrow() {
    assert_eq!(truncate("abcdefgh", 2), "ab");
  }
}
