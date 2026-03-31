//! Markdown → ratatui `Line` renderer powered by pulldown-cmark.
//!
//! The main entry-point is [`parse_markdown`], which converts a Markdown
//! string into a `Vec<Line<'static>>` that can be fed directly to a ratatui
//! [`Paragraph`](ratatui::widgets::Paragraph).
//!
//! Code block rendering can be customised via [`CodeBlockOptions`].

use std::io;
use std::sync::LazyLock;

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::buffer::Buffer;
use ratatui::prelude::*;
use syntect::easy::HighlightLines;
use syntect::highlighting::{FontStyle, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use super::term_caps::{chars, colors, supports_osc8};

// ── Code block options ────────────────────────────────────────────────────────

/// Options for rendering code blocks in markdown.
#[derive(Debug, Clone, Copy)]
pub struct CodeBlockOptions {
  /// Whether to show line numbers in the gutter.
  pub line_numbers: bool,
  /// Whether to apply syntax highlighting.
  pub syntax_highlighting: bool,
}

impl Default for CodeBlockOptions {
  fn default() -> Self {
    Self {
      line_numbers: true,
      syntax_highlighting: true,
    }
  }
}

// ── Syntax highlighting ───────────────────────────────────────────────────────

/// Syntect syntax definitions, loaded once at first use.
static SYNTAX_SET: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);

/// Syntect colour themes, loaded once at first use.
static THEME_SET: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

/// The syntect theme applied to all code blocks.
const CODE_THEME: &str = "base16-ocean.dark";

/// Background colour for code blocks - matches the base16-ocean.dark theme
/// background (`#2b303b`) so syntax colours sit on their intended canvas.
/// Uses term_caps for cross-platform color support.
fn code_bg() -> Color {
  colors::code_bg()
}

/// Render `code` with full syntax highlighting.
///
/// `lang` is the language token from the opening fence (e.g. `"rust"`,
/// `"python"`, `""`). Unknown languages fall back to the plain-text grammar.
///
/// Use `opts` to control line numbers and syntax highlighting.
pub(crate) fn highlight_code_block(code: &str, lang: &str, width: u16, opts: CodeBlockOptions) -> Vec<Line<'static>> {
  // If syntax highlighting is disabled, use plain rendering (no background)
  if !opts.syntax_highlighting {
    return plain_code_lines(code, width, opts.line_numbers, None);
  }

  let ps = &*SYNTAX_SET;
  let ts = &*THEME_SET;
  let bg = Some(code_bg());

  let syntax = if lang.is_empty() {
    ps.find_syntax_plain_text()
  } else {
    ps.find_syntax_by_token(lang).unwrap_or_else(|| ps.find_syntax_plain_text())
  };

  let theme = match ts.themes.get(CODE_THEME) {
    Some(t) => t,
    None => return plain_code_lines(code, width, opts.line_numbers, bg),
  };

  // Collect source lines up-front so we know the total count for number width.
  let mut source_lines: Vec<&str> = LinesWithEndings::from(code).collect();
  // Drop the trailing empty entry produced by a final '\n'.
  while source_lines.last().map(|l| l.trim_end_matches(['\n', '\r']).trim().is_empty()).unwrap_or(false) {
    source_lines.pop();
  }
  if source_lines.is_empty() {
    return Vec::new();
  }

  let num_width = source_lines.len().to_string().len().max(1);
  let mut h = HighlightLines::new(syntax, theme);
  let mut content = Vec::with_capacity(source_lines.len());

  for (i, source_line) in source_lines.iter().enumerate() {
    match h.highlight_line(source_line, ps) {
      Ok(ranges) => {
        let spans: Vec<Span<'static>> = ranges
          .into_iter()
          .filter(|(_, text)| !text.is_empty())
          .map(|(style, text)| {
            let fg = colors::rgb(style.foreground.r, style.foreground.g, style.foreground.b);
            let mut s = Style::default().fg(fg);
            if let Some(bg_color) = bg {
              s = s.bg(bg_color);
            }
            if style.font_style.contains(FontStyle::BOLD) {
              s = s.add_modifier(Modifier::BOLD);
            }
            if style.font_style.contains(FontStyle::ITALIC) {
              s = s.add_modifier(Modifier::ITALIC);
            }
            if style.font_style.contains(FontStyle::UNDERLINE) {
              s = s.add_modifier(Modifier::UNDERLINED);
            }
            Span::styled(text.trim_end_matches('\n').to_string(), s)
          })
          .collect();
        content.push(code_line(spans, i + 1, num_width, width, bg, opts.line_numbers));
      }
      Err(_) => {
        let mut style = Style::default().fg(Color::Yellow);
        if let Some(bg_color) = bg {
          style = style.bg(bg_color);
        }
        content.push(code_line(
          vec![Span::styled(
            source_line.trim_end_matches('\n').to_string(),
            style,
          )],
          i + 1,
          num_width,
          width,
          bg,
          opts.line_numbers,
        ));
      }
    }
  }
  content
}

/// Fallback: plain yellow lines when syntect is unavailable or highlighting disabled.
fn plain_code_lines(code: &str, width: u16, show_line_numbers: bool, bg: Option<Color>) -> Vec<Line<'static>> {
  let src: Vec<&str> = code.lines().collect();
  if src.is_empty() {
    return Vec::new();
  }
  let num_width = src.len().to_string().len().max(1);
  let mut lines = Vec::with_capacity(src.len());
  for (i, l) in src.iter().enumerate() {
    let mut style = Style::default().fg(Color::Yellow);
    if let Some(bg_color) = bg {
      style = style.bg(bg_color);
    }
    lines.push(code_line(
      vec![Span::styled(l.to_string(), style)],
      i + 1,
      num_width,
      width,
      bg,
      show_line_numbers,
    ));
  }
  lines
}

/// Build a single code-block line with optional line number gutter.
///
/// Layout (optionally on background color):
/// ```text
///  <1 char pad> [<line_num right-aligned> <gutter sep>] <code spans> <trailing pad>
/// ```
/// The trailing pad span explicitly fills the row to `width` so the background
/// covers the full terminal width regardless of ratatui's line-style behaviour.
/// When `bg` is `None`, no background is applied (for plain/disabled highlighting).
fn code_line(spans: Vec<Span<'static>>, line_num: usize, num_width: usize, width: u16, bg: Option<Color>, show_line_numbers: bool) -> Line<'static> {
  let code_chars: usize = spans.iter().map(|s| s.content.chars().count()).sum();

  let mut all = Vec::with_capacity(spans.len() + 4);

  // Helper to apply optional background
  let with_bg = |style: Style| -> Style {
    match bg {
      Some(bg_color) => style.bg(bg_color),
      None => style,
    }
  };

  let prefix_len = if show_line_numbers {
    // Get the gutter separator (Unicode or ASCII depending on terminal)
    let gutter_sep = chars::gutter_sep();
    // Fixed prefix length: 1 (left pad) + num_width (number) + gutter_sep length
    let prefix_len = 1 + num_width + gutter_sep.chars().count();

    // 1-char left padding.
    all.push(Span::styled(" ", with_bg(Style::default())));
    // Line number, right-aligned and muted.
    all.push(Span::styled(
      format!("{:>num_width$}", line_num),
      with_bg(Style::default().fg(colors::code_gutter_fg())),
    ));
    // Gutter separator (uses term_caps for cross-platform char).
    all.push(Span::styled(gutter_sep.to_string(), with_bg(Style::default().fg(colors::code_gutter_sep_fg()))));

    prefix_len
  } else {
    // Just 1-char left padding when line numbers are disabled
    all.push(Span::styled(" ", with_bg(Style::default())));
    1
  };

  // Code content.
  all.extend(spans);

  // Trailing spaces to fill the rest of the row (only when background is enabled).
  let used = prefix_len + code_chars;
  let trailing = (width as usize).saturating_sub(used);
  if trailing > 0 && bg.is_some() {
    all.push(Span::styled(" ".repeat(trailing), with_bg(Style::default())));
  }

  let mut line = Line::from(all);
  if let Some(bg_color) = bg {
    line.style = Style::default().bg(bg_color);
  }
  line
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Convert a Markdown string into a vector of styled ratatui [`Line`]s.
///
/// The returned lines can be passed directly to
/// `Paragraph::new(lines).scroll(…).wrap(…)`.
/// `width` is the available render width in terminal columns; it is used to
/// pad code-block lines with trailing spaces so the background fills the full
/// row.  Pass `0` when the width is unknown (e.g. in unit tests).
/// Position of a rendered hyperlink within the output line vector.
///
/// `line_idx` is a zero-based index into the `Vec<Line>` returned by
/// [`parse_markdown_with_links`].  `col_start`/`col_end` are display-column
/// offsets within that line (measured in Unicode scalar values, which is a
/// good-enough approximation for OSC 8 cell overwriting).
#[derive(Debug, Clone)]
pub struct LinkSpan {
  /// Index of the logical line that contains this link.
  pub line_idx: usize,
  /// First column of the link anchor text (inclusive).
  pub col_start: usize,
  /// One-past-the-last column of the link anchor text (exclusive).
  pub col_end: usize,
  /// The destination URL.
  pub url: String,
}

/// Bundle of positioning data needed to apply OSC 8 hyperlinks to the
/// terminal **after** ratatui has flushed a frame.
///
/// Create one instance per rendered markdown region and call
/// [`PendingOsc8::write_to`] once [`Terminal::draw`] returns.
#[derive(Debug)]
pub struct PendingOsc8 {
  /// The screen area the content was rendered into (border rows excluded).
  pub area: Rect,
  /// The vertical scroll offset (in logical lines) that was active.
  pub scroll: usize,
  /// Link positions recorded during markdown parsing.
  pub links: Vec<LinkSpan>,
}

impl PendingOsc8 {
  /// Write OSC 8 hyperlink sequences directly to `out` (typically stderr).
  ///
  /// Each link's text is re-printed - with its original colour read back
  /// from the completed ratatui [`Buffer`] - wrapped between an OSC 8
  /// opening (`ESC ] 8 ;; URL BEL`) and closing (`ESC ] 8 ;; BEL`)
  /// sequence.  Because we write the sequences directly to the terminal
  /// *after* the buffer diff, ratatui's width calculation never sees the
  /// embedded URL and therefore never skips subsequent cells.
  ///
  /// **Must be called after [`Terminal::draw`] returns.**  Callers are
  /// responsible for flushing `out` after all pending sets are written.
  ///
  /// On terminals that don't support OSC 8 (e.g., Windows CMD), this
  /// function returns early without writing any escape sequences.
  pub fn write_to<W: io::Write>(&self, buf: &Buffer, out: &mut W) -> io::Result<()> {
    use crossterm::{
      cursor::{Hide, MoveTo, RestorePosition, SavePosition, Show},
      queue,
      style::{Attribute, Print, ResetColor, SetAttribute, SetForegroundColor},
    };

    // Skip OSC 8 on terminals that don't support it (e.g., Windows CMD)
    if !supports_osc8() {
      return Ok(());
    }

    if self.links.is_empty() {
      return Ok(());
    }

    queue!(out, Hide, SavePosition)?;

    for link in &self.links {
      if link.line_idx < self.scroll {
        continue;
      }
      let row = link.line_idx - self.scroll;
      if row >= self.area.height as usize {
        break;
      }
      let screen_y = self.area.y + row as u16;
      if link.col_start >= self.area.width as usize {
        continue;
      }
      let col_end = link.col_end.min(self.area.width as usize);

      // Position cursor and emit the OSC 8 opening sequence.
      queue!(out, MoveTo(self.area.x + link.col_start as u16, screen_y))?;
      write!(out, "\x1B]8;;{}\x07", link.url)?;

      // Re-print each character with its original colour from the buffer.
      let mut prev_fg: Option<Color> = None;
      let mut prev_bold = false;
      for col in link.col_start..col_end {
        let cell = &buf[(self.area.x + col as u16, screen_y)];
        let is_bold = cell.modifier.contains(Modifier::BOLD);

        if prev_fg != Some(cell.fg) {
          match cell.fg {
            Color::Reset => queue!(out, ResetColor)?,
            fg => queue!(out, SetForegroundColor(ratatui_to_crossterm_color(fg)))?,
          }
          prev_fg = Some(cell.fg);
        }
        if is_bold != prev_bold {
          if is_bold {
            queue!(out, SetAttribute(Attribute::Bold))?;
          } else {
            queue!(out, SetAttribute(Attribute::NoBold))?;
          }
          prev_bold = is_bold;
        }
        queue!(out, Print(cell.symbol()))?;
      }

      // Emit OSC 8 closing and reset styling.
      write!(out, "\x1B]8;;\x07")?;
      queue!(out, ResetColor, SetAttribute(Attribute::Reset))?;
    }

    queue!(out, RestorePosition, Show)?;
    Ok(())
  }
}

/// Convert a ratatui [`Color`] to its crossterm equivalent.
fn ratatui_to_crossterm_color(c: Color) -> crossterm::style::Color {
  use crossterm::style::Color as CC;
  match c {
    Color::Reset => CC::Reset,
    Color::Black => CC::Black,
    Color::Red => CC::DarkRed,
    Color::Green => CC::DarkGreen,
    Color::Yellow => CC::DarkYellow,
    Color::Blue => CC::DarkBlue,
    Color::Magenta => CC::DarkMagenta,
    Color::Cyan => CC::DarkCyan,
    Color::Gray => CC::Grey,
    Color::DarkGray => CC::DarkGrey,
    Color::LightRed => CC::Red,
    Color::LightGreen => CC::Green,
    Color::LightYellow => CC::Yellow,
    Color::LightBlue => CC::Blue,
    Color::LightMagenta => CC::Magenta,
    Color::LightCyan => CC::Cyan,
    Color::White => CC::White,
    Color::Rgb(r, g, b) => CC::Rgb { r, g, b },
    Color::Indexed(i) => CC::AnsiValue(i),
  }
}

/// Like [`parse_markdown`] but also returns every hyperlink found in the
/// document together with its rendered position, so that callers can apply
/// OSC 8 terminal hyperlink escape sequences to the ratatui buffer.
pub fn parse_markdown_with_links(input: &str, width: u16) -> (Vec<Line<'static>>, Vec<LinkSpan>) {
  parse_markdown_with_links_opts(input, width, CodeBlockOptions::default())
}

/// Like [`parse_markdown_with_links`] but with configurable code block options.
pub fn parse_markdown_with_links_opts(input: &str, width: u16, code_opts: CodeBlockOptions) -> (Vec<Line<'static>>, Vec<LinkSpan>) {
  let options = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS | Options::ENABLE_TABLES;
  let parser = Parser::new_ext(input, options);
  let mut renderer = Renderer { width, code_opts, ..Default::default() };
  for event in parser {
    renderer.process(event);
  }
  renderer.finish()
}

pub fn parse_markdown(input: &str, width: u16) -> Vec<Line<'static>> {
  parse_markdown_with_links(input, width).0
}

// ── Internal renderer ─────────────────────────────────────────────────────────

#[derive(Default)]
struct Renderer {
  /// Completed lines ready to be returned.
  lines: Vec<Line<'static>>,
  /// Spans accumulating for the line currently being built.
  current_spans: Vec<Span<'static>>,
  /// Bold nesting depth (also incremented inside headings).
  bold: u32,
  /// Italic nesting depth.
  italic: u32,
  /// Strikethrough nesting depth.
  strikethrough: u32,
  /// Foreground colour override while rendering a heading.
  heading_fg: Option<Color>,
  /// Whether we're inside a fenced/indented code block.
  in_code_block: bool,
  /// Language token from the opening fence (empty string for indented blocks).
  code_lang: String,
  /// Accumulated raw text for the current code block (may arrive in multiple
  /// `Text` events when the block is inside a list item).
  code_text: String,
  /// List stack: `None` = unordered bullet, `Some(n)` = ordered starting at `n`.
  list_stack: Vec<Option<u64>>,
  /// Running ordinal counter for each ordered list level.
  item_numbers: Vec<u64>,
  /// Block-quote nesting depth.
  blockquote_depth: u32,
  /// Whether a blank separator line should precede the next block element.
  pending_blank: bool,
  /// Whether we are currently inside a link (anchor text).
  in_link: bool,
  /// Destination URL of the link currently being rendered.
  link_url: String,
  /// Display-column offset where the current link's anchor text starts.
  link_col_start: usize,
  /// Collected link positions for the entire document.
  links: Vec<LinkSpan>,
  /// Available render width passed through to code-block helpers.
  width: u16,
  /// Code block rendering options.
  code_opts: CodeBlockOptions,
}

impl Renderer {
  // ── Style helpers ─────────────────────────────────────────────────────────

  /// Compute the `Style` for the next text span based on current context.
  fn current_style(&self) -> Style {
    let mut style = Style::default();
    if let Some(fg) = self.heading_fg {
      // Heading colour always wins.
      style = style.fg(fg);
    } else if self.in_code_block {
      style = style.fg(Color::Yellow); // fallback; syntect normally handles this
    } else if self.in_link {
      style = style.fg(Color::Rgb(130, 205, 255)); // light blue
    } else if self.bold > 0 || self.italic > 0 {
      style = style.fg(Color::Rgb(188, 120, 255)); // vivid purple
    }
    if self.bold > 0 {
      style = style.add_modifier(Modifier::BOLD);
    }
    if self.italic > 0 {
      style = style.add_modifier(Modifier::ITALIC);
    }
    if self.strikethrough > 0 {
      style = style.add_modifier(Modifier::CROSSED_OUT);
    }
    style
  }

  // ── Line management ───────────────────────────────────────────────────────

  /// Move `current_spans` into a finished [`Line`].
  fn flush_line(&mut self) {
    let spans = std::mem::take(&mut self.current_spans);
    self.lines.push(Line::from(spans));
  }

  /// Prepare for a new block-level element:
  /// 1. Flush any pending inline spans.
  /// 2. If `pending_blank` is set, emit a blank separator line (unless the
  ///    most-recently emitted line is already blank).
  fn start_block(&mut self) {
    if !self.current_spans.is_empty() {
      self.flush_line();
    }
    if self.pending_blank {
      self.pending_blank = false;
      let last_is_blank = self.lines.last().map(|l| l.spans.is_empty()).unwrap_or(true);
      if !last_is_blank {
        self.lines.push(Line::default());
      }
    }
  }

  // ── List / blockquote helpers ─────────────────────────────────────────────

  /// Indentation string proportional to the current list nesting level.
  fn list_indent(&self) -> String {
    "  ".repeat(self.list_stack.len().saturating_sub(1))
  }

  /// Blockquote `│ ` (or `| ` on limited terminals) leader for the current nesting depth.
  fn blockquote_prefix(&self) -> String {
    if self.blockquote_depth == 0 {
      String::new()
    } else {
      format!("{} ", chars::vertical()).repeat(self.blockquote_depth as usize)
    }
  }

  // ── Event dispatch ────────────────────────────────────────────────────────

  fn process(&mut self, event: Event<'_>) {
    match event {
      Event::Start(tag) => self.start_tag(tag),
      Event::End(tag) => self.end_tag(tag),

      // ── Text content ──────────────────────────────────────────────────
      Event::Text(text) => {
        let text = text.into_string();
        if self.in_code_block {
          // Accumulate all text chunks - pulldown-cmark may split a
          // code block into multiple Text events (e.g. inside list
          // items).  We render once at End(CodeBlock).
          self.code_text.push_str(&text);
        } else {
          // Emit the blockquote leader at the start of a fresh line.
          let prefix = self.blockquote_prefix();
          if !prefix.is_empty() && self.current_spans.is_empty() {
            self.current_spans.push(Span::styled(prefix, Style::default().fg(Color::DarkGray)));
          }
          self.current_spans.push(Span::styled(text, self.current_style()));
        }
      }

      // ── Inline code ───────────────────────────────────────────────────
      Event::Code(code) => {
        self.current_spans.push(Span::styled(
          format!("`{}`", code.as_ref()),
          Style::default().fg(Color::Rgb(255, 165, 0)).add_modifier(Modifier::BOLD),
        ));
      }

      // ── Raw HTML (show dimmed; we cannot render it properly) ───────────
      Event::Html(html) | Event::InlineHtml(html) => {
        self.current_spans.push(Span::styled(html.into_string(), Style::default().fg(Color::DarkGray)));
      }

      // ── Soft break: flush the current line so the author's line breaks
      // are preserved.  This differs from HTML reflow but is the right
      // behaviour for a terminal renderer where wrapping is handled by
      // the Paragraph widget, not the content author.
      Event::SoftBreak => {
        if !self.in_code_block && !self.current_spans.is_empty() {
          self.flush_line();
        }
      }

      // ── Hard break: explicit newline ──────────────────────────────────
      Event::HardBreak => {
        self.flush_line();
      }

      // ── Thematic break (---) ──────────────────────────────────────────
      Event::Rule => {
        self.start_block();
        self.lines.push(Line::from(Span::styled("─".repeat(60), Style::default().fg(Color::DarkGray))));
        self.pending_blank = true;
      }

      // ── Task-list checkbox ────────────────────────────────────────────
      Event::TaskListMarker(checked) => {
        let (marker, color) = if checked { ("[x] ", Color::Green) } else { ("[ ] ", Color::DarkGray) };
        self.current_spans.push(Span::styled(marker.to_string(), Style::default().fg(color)));
      }

      _ => {}
    }
  }

  fn start_tag(&mut self, tag: Tag<'_>) {
    match tag {
      // ── Headings ──────────────────────────────────────────────────────
      Tag::Heading { level, .. } => {
        self.start_block();
        let (fg, prefix): (Color, &'static str) = match level {
          HeadingLevel::H1 => (Color::Rgb(147, 226, 255), "# "),     // lightest sky-blue
          HeadingLevel::H2 => (Color::Rgb(79, 195, 247), "## "),     // sky blue
          HeadingLevel::H3 => (Color::Rgb(56, 152, 235), "### "),    // azure blue
          HeadingLevel::H4 => (Color::Rgb(100, 130, 220), "#### "),  // cornflower blue
          HeadingLevel::H5 => (Color::Rgb(120, 110, 210), "##### "), // periwinkle
          HeadingLevel::H6 => (Color::Rgb(140, 90, 200), "###### "), // indigo
        };
        self.heading_fg = Some(fg);
        self.bold += 1;
        // Push the hashed prefix in the heading colour.
        self
          .current_spans
          .push(Span::styled(prefix, Style::default().fg(fg).add_modifier(Modifier::BOLD)));
      }

      // ── Paragraphs ────────────────────────────────────────────────────
      Tag::Paragraph => {
        self.start_block();
      }

      // ── Block quotes ──────────────────────────────────────────────────
      Tag::BlockQuote(_) => {
        self.start_block();
        self.blockquote_depth += 1;
      }

      // ── Fenced / indented code blocks ─────────────────────────────────
      Tag::CodeBlock(kind) => {
        // Emit the preceding blank separator (if any) before the top
        // code_pad_line, then buffer text until End(CodeBlock) so that
        // multiple Text events (e.g. inside list items) are joined into
        // a single highlighted block.
        self.start_block();
        self.in_code_block = true;
        self.code_lang = match kind {
          CodeBlockKind::Fenced(lang) => lang.to_string(),
          CodeBlockKind::Indented => String::new(),
        };
        self.code_text.clear();
      }

      // ── Lists ─────────────────────────────────────────────────────────
      Tag::List(first) => {
        self.start_block();
        self.list_stack.push(first);
        self.item_numbers.push(first.unwrap_or(1));
      }

      Tag::Item => {
        self.start_block();
        let indent = self.list_indent();
        let is_ordered = self.list_stack.last().and_then(|v| *v).is_some();
        let prefix = if is_ordered {
          let n = self.item_numbers.last_mut().expect("item_numbers always in sync with list_stack");
          let label = format!("{indent}{}. ", n);
          *n += 1;
          label
        } else {
          format!("{indent}• ")
        };
        self.current_spans.push(Span::styled(prefix, Style::default().fg(Color::White)));
      }

      // ── Inline formatting ─────────────────────────────────────────────
      Tag::Strong => self.bold += 1,
      Tag::Emphasis => self.italic += 1,
      Tag::Strikethrough => self.strikethrough += 1,

      // Links: render anchor text in light blue; record position + URL
      // so the caller can apply OSC 8 hyperlink sequences.
      Tag::Link { dest_url, .. } => {
        self.in_link = true;
        self.link_url = dest_url.into_string();
        // Column start = total display width of spans already on this line.
        self.link_col_start = self.current_spans.iter().map(|s| s.content.chars().count()).sum();
      }

      // Images can't be rendered as pixels; show a dim placeholder.
      Tag::Image { .. } => {
        self
          .current_spans
          .push(Span::styled("[image]".to_string(), Style::default().fg(Color::DarkGray)));
      }

      _ => {}
    }
  }

  fn end_tag(&mut self, tag: TagEnd) {
    match tag {
      // ── Headings ──────────────────────────────────────────────────────
      TagEnd::Heading(_) => {
        self.bold -= 1;
        self.heading_fg = None;
        self.flush_line();
        self.pending_blank = true;
      }

      // ── Paragraphs ────────────────────────────────────────────────────
      TagEnd::Paragraph => {
        self.flush_line();
        self.pending_blank = true;
      }

      // ── Block quotes ──────────────────────────────────────────────────
      TagEnd::BlockQuote(_) => {
        if !self.current_spans.is_empty() {
          self.flush_line();
        }
        self.blockquote_depth = self.blockquote_depth.saturating_sub(1);
        self.pending_blank = true;
      }

      // ── Code blocks ───────────────────────────────────────────────────
      TagEnd::CodeBlock => {
        // Render the accumulated text in one shot so that multiple
        // Text events (e.g. inside list items) produce a single block
        // with a shared line-number gutter.
        self.lines.extend(highlight_code_block(&self.code_text, &self.code_lang, self.width, self.code_opts));
        self.code_text.clear();
        self.in_code_block = false;
        self.code_lang.clear();
        // Place the blank separator *after* the bottom code_pad_line so
        // that the next block element is visually separated from the
        // code block without the blank appearing before the top pad.
        self.pending_blank = true;
      }

      // ── Lists ─────────────────────────────────────────────────────────
      TagEnd::List(_) => {
        self.list_stack.pop();
        self.item_numbers.pop();
        self.pending_blank = true;
      }

      TagEnd::Item => {
        // Flush only if there are spans that weren't already flushed by
        // a child `Paragraph` end (tight list items have no Paragraph
        // wrapper, so their text lives directly in current_spans here).
        if !self.current_spans.is_empty() {
          self.flush_line();
        }
      }

      // ── Inline formatting ─────────────────────────────────────────────
      TagEnd::Strong => self.bold = self.bold.saturating_sub(1),
      TagEnd::Emphasis => self.italic = self.italic.saturating_sub(1),
      TagEnd::Strikethrough => self.strikethrough = self.strikethrough.saturating_sub(1),

      TagEnd::Link => {
        self.in_link = false;
        if !self.link_url.is_empty() {
          let col_end: usize = self.current_spans.iter().map(|s| s.content.chars().count()).sum();
          self.links.push(LinkSpan {
            line_idx: self.lines.len(),
            col_start: self.link_col_start,
            col_end,
            url: std::mem::take(&mut self.link_url),
          });
          self.link_col_start = 0;
        }
      }
      TagEnd::Image => {}

      _ => {}
    }
  }

  // ── Finalise ──────────────────────────────────────────────────────────────

  fn finish(mut self) -> (Vec<Line<'static>>, Vec<LinkSpan>) {
    if !self.current_spans.is_empty() {
      self.flush_line();
    }
    // Drop trailing blank lines so the widget doesn't show empty scroll space.
    while self.lines.last().map(|l| l.spans.is_empty()).unwrap_or(false) {
      self.lines.pop();
    }
    (self.lines, self.links)
  }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
  use super::*;

  fn text_of(line: &Line<'_>) -> String {
    line.spans.iter().map(|s| s.content.as_ref()).collect()
  }

  #[test]
  fn h1_has_prefix_and_bold() {
    let lines = parse_markdown("# Hello world", 0);
    assert_eq!(lines.len(), 1);
    let full = text_of(&lines[0]);
    assert!(full.contains("# "), "should contain '# ' prefix");
    assert!(full.contains("Hello world"));
    let bold = lines[0].spans.iter().any(|s| s.style.add_modifier.contains(Modifier::BOLD));
    assert!(bold, "heading text should be bold");
  }

  #[test]
  fn h2_h3_different_prefix() {
    let lines = parse_markdown("## Second\n### Third", 0);
    let texts: Vec<_> = lines.iter().map(text_of).collect();
    assert!(texts.iter().any(|t| t.starts_with("## ")));
    assert!(texts.iter().any(|t| t.starts_with("### ")));
  }

  #[test]
  fn bold_span_carries_modifier() {
    let lines = parse_markdown("Plain **bold** text", 0);
    assert_eq!(lines.len(), 1);
    let bold_span = lines[0].spans.iter().find(|s| s.content.contains("bold")).expect("bold span");
    assert!(bold_span.style.add_modifier.contains(Modifier::BOLD));
  }

  #[test]
  fn italic_span_carries_modifier() {
    let lines = parse_markdown("*italic*", 0);
    let span = lines[0].spans.iter().find(|s| s.content.contains("italic")).expect("italic span");
    assert!(span.style.add_modifier.contains(Modifier::ITALIC));
  }

  #[test]
  fn inline_code_orange_with_backticks() {
    let lines = parse_markdown("Call `foo()` now", 0);
    let span = lines[0].spans.iter().find(|s| s.content.contains("foo()")).expect("code span");
    assert_eq!(span.style.fg, Some(Color::Rgb(255, 165, 0)));
    assert!(span.content.contains('`'));
  }

  #[test]
  fn bold_text_is_purple() {
    let lines = parse_markdown("Plain **important** word", 0);
    let span = lines[0].spans.iter().find(|s| s.content.contains("important")).expect("bold span");
    assert_eq!(span.style.fg, Some(Color::Rgb(188, 120, 255)));
    assert!(span.style.add_modifier.contains(Modifier::BOLD));
  }

  #[test]
  fn italic_text_is_purple() {
    let lines = parse_markdown("Plain *soft* word", 0);
    let span = lines[0].spans.iter().find(|s| s.content.contains("soft")).expect("italic span");
    assert_eq!(span.style.fg, Some(Color::Rgb(188, 120, 255)));
    assert!(span.style.add_modifier.contains(Modifier::ITALIC));
  }

  #[test]
  fn link_text_is_light_blue() {
    let lines = parse_markdown("[Rust book](https://doc.rust-lang.org/book/)", 0);
    let span = lines[0].spans.iter().find(|s| s.content.contains("Rust book")).expect("link span");
    assert_eq!(span.style.fg, Some(Color::Rgb(130, 205, 255)));
  }

  #[test]
  fn fenced_code_block_is_highlighted() {
    let lines = parse_markdown("```rust\nlet x = 1;\n```", 0);
    assert!(!lines.is_empty(), "highlighted code block should produce lines");
    let code_line = lines.iter().find(|l| text_of(l).contains("let")).expect("a line containing 'let'");
    // syntect assigns RGB colours; just verify at least one span has a fg.
    assert!(
      code_line.spans.iter().any(|s| s.style.fg.is_some()),
      "code block spans should carry foreground colours from syntect"
    );
  }

  #[test]
  fn unordered_list_bullet_prefix() {
    let lines = parse_markdown("- Alpha\n- Beta", 0);
    let texts: Vec<_> = lines.iter().map(text_of).collect();
    assert!(texts.iter().any(|t| t.contains("• ") && t.contains("Alpha")));
    assert!(texts.iter().any(|t| t.contains("• ") && t.contains("Beta")));
  }

  #[test]
  fn ordered_list_numbers() {
    let lines = parse_markdown("1. First\n2. Second", 0);
    let texts: Vec<_> = lines.iter().map(text_of).collect();
    assert!(texts.iter().any(|t| t.contains("1.") && t.contains("First")));
    assert!(texts.iter().any(|t| t.contains("2.") && t.contains("Second")));
  }

  #[test]
  fn blockquote_leader() {
    let lines = parse_markdown("> A quoted line", 0);
    let texts: Vec<_> = lines.iter().map(text_of).collect();
    assert!(texts.iter().any(|t| t.contains("│")), "blockquote should show │ leader");
  }

  #[test]
  fn no_trailing_blank_lines() {
    let lines = parse_markdown("Hello\n\n", 0);
    assert!(!lines.is_empty());
    assert!(!lines.last().unwrap().spans.is_empty(), "trailing blank lines should be stripped");
  }

  #[test]
  fn separator_rule_sixty_dashes() {
    let lines = parse_markdown("---", 0);
    assert!(
      lines.iter().any(|l| text_of(l).contains("─")),
      "thematic break should render as a line of box-drawing dashes"
    );
  }

  #[test]
  fn strikethrough_modifier() {
    let lines = parse_markdown("~~gone~~", 0);
    let span = lines[0].spans.iter().find(|s| s.content.contains("gone")).expect("strikethrough span");
    assert!(span.style.add_modifier.contains(Modifier::CROSSED_OUT));
  }
}

#[cfg(test)]
mod code_block_layout_tests {
  use super::*;

  /// Returns true if the line carries the code_bg() background colour.
  fn has_code_bg(l: &Line<'_>) -> bool {
    let bg = code_bg();
    l.style.bg == Some(bg) || l.spans.iter().any(|s| s.style.bg == Some(bg))
  }

  /// Default code options for tests.
  fn test_opts() -> CodeBlockOptions {
    CodeBlockOptions::default()
  }

  /// Find all (index, has_code_bg) pairs for a rendered markdown string.
  fn bg_pattern(md: &str) -> Vec<bool> {
    parse_markdown(md, 80).iter().map(has_code_bg).collect()
  }

  #[test]
  fn blank_separator_after_last_code_line() {
    // Structure: text → blank → content... → blank
    let p = bg_pattern("Some intro\n\n```rust\nlet x = 1;\n```\n\nAfter.");
    // Find the last CODE_BG line and verify a non-CODE_BG blank follows it.
    let last_code = p.iter().rposition(|&b| b).expect("at least one code line");
    assert!(last_code + 1 < p.len(), "blank separator should follow last code line");
    assert!(!p[last_code + 1], "line after last code line must be blank (no CODE_BG)");
  }

  #[test]
  fn blank_separator_between_adjacent_code_blocks() {
    let p = bg_pattern("```rust\nfn a() {}\n```\n\n```rust\nfn b() {}\n```");
    // Expect: CODE … CODE  false  CODE … CODE
    // There must be at least one non-CODE_BG line sandwiched between two
    // CODE_BG groups.
    let first_code = p.iter().position(|&b| b).expect("code lines");
    let last_code = p.iter().rposition(|&b| b).expect("code lines");
    let has_gap = p[first_code..=last_code].iter().any(|&b| !b);
    assert!(has_gap, "blank separator must exist between the two blocks");
  }

  #[test]
  fn list_item_code_block_merged_into_one() {
    // When pulldown-cmark emits multiple Text events for lines in a list-item
    // code block, they must be merged into a single highlighted block - not
    // rendered as separate blocks each with their own top/bottom pads.
    let md = "1. Description:\n\n   ```rust\n   let x = 1;\n   let y = 2;\n   let z = 3;\n   ```\n\n2. Next item";
    let lines = parse_markdown(md, 80);

    // There should be exactly one contiguous run of CODE_BG lines.
    let mut runs = 0u32;
    let mut prev = false;
    for l in &lines {
      let cur = has_code_bg(l);
      if cur && !prev {
        runs += 1;
      }
      prev = cur;
    }
    assert_eq!(runs, 1, "all code lines should form a single CODE_BG run");

    // The code block should contain lines numbered 1, 2, 3.
    let code_lines: Vec<_> = lines
      .iter()
      .filter(|l| has_code_bg(l))
      .filter(|l| {
        let t: String = l.spans.iter().map(|s| s.content.as_ref()).collect();
        t.contains("│")
      })
      .collect();
    assert_eq!(code_lines.len(), 3, "should have exactly 3 content lines");
    let texts: Vec<String> = code_lines.iter().map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect()).collect();
    assert!(texts[0].contains(" 1 "), "first line numbered 1");
    assert!(texts[1].contains(" 2 "), "second line numbered 2");
    assert!(texts[2].contains(" 3 "), "third line numbered 3");
  }
}
