//! About page - content is driven by the embedded `about.md` file.
//!
//! The file is compiled into the binary via [`include_str!`], parsed with the
//! standard markdown renderer, and displayed identically to the Theory / Task
//! pages.  Links are made clickable through OSC 8 terminal hyperlink sequences
//! applied *after* the ratatui frame is flushed (see [`PendingOsc8`]).
//!
//! The content is rendered in a fixed-width column (≤ 72 columns) that is
//! centred horizontally in the available area.  When the content fits in the
//! viewport it is also centred vertically.

use ratatui::prelude::*;
use ratatui::widgets::{Paragraph, Wrap};

use crate::ui::markdown::{PendingOsc8, parse_markdown_with_links};

/// Markdown source embedded at compile time.
const ABOUT_MD: &str = include_str!("about.md");

/// Maximum content column width used for comfortable reading and to keep the
/// ASCII art intact.
const MAX_CONTENT_WIDTH: u16 = 72;

/// Render the About page and return the OSC 8 link data to be applied after
/// [`Terminal::draw`] returns.
///
/// The content is placed in a column of at most [`MAX_CONTENT_WIDTH`] columns,
/// centred horizontally.  When the rendered content fits inside `area` without
/// scrolling it is also centred vertically.
///
/// `scroll` mirrors `App::scroll_offset` so the content can be paged when
/// the terminal is very short.
pub fn render(frame: &mut Frame, area: Rect, scroll: usize) -> PendingOsc8 {
  // ── Content width ─────────────────────────────────────────────────────
  // Cap at MAX_CONTENT_WIDTH; never exceed the available area.
  let content_width = area.width.min(MAX_CONTENT_WIDTH);

  // ── Parse at the clamped width ────────────────────────────────────────
  let (lines, links) = parse_markdown_with_links(ABOUT_MD, content_width);

  // ── Visual height ─────────────────────────────────────────────────────
  // Account for lines that wrap when computing the total number of terminal
  // rows the content will occupy.  Blank lines (width == 0) still take one
  // row.
  let visual_height: u16 = lines
    .iter()
    .map(|line| {
      let w = line.width() as u16;
      if w == 0 { 1 } else { w.div_ceil(content_width) }
    })
    .sum();

  // ── Horizontal centring ───────────────────────────────────────────────
  let h_margin = area.width.saturating_sub(content_width) / 2;

  // ── Vertical centring ─────────────────────────────────────────────────
  // Only centre when the content fits in the viewport without scrolling.
  let v_margin = if visual_height <= area.height && scroll == 0 {
    area.height.saturating_sub(visual_height) / 2
  } else {
    0
  };

  // ── Build the centred rect ────────────────────────────────────────────
  let content_area = Rect {
    x: area.x + h_margin,
    y: area.y + v_margin,
    width: content_width,
    // Do not extend beyond the bottom of `area`.
    height: area.height.saturating_sub(v_margin),
  };

  let para = Paragraph::new(lines).scroll((scroll as u16, 0)).wrap(Wrap { trim: false });

  frame.render_widget(para, content_area);

  PendingOsc8 {
    area: content_area,
    scroll,
    links,
  }
}
