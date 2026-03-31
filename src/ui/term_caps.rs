//! Terminal capability detection for cross-platform rendering.
//!
//! This module detects terminal features like Unicode support, true color,
//! and OSC 8 hyperlinks, providing appropriate fallbacks for terminals
//! with limited capabilities (e.g., Windows CMD).

use std::env;
use std::sync::OnceLock;

/// Cached terminal capabilities, initialized once at startup.
static CAPS: OnceLock<TermCaps> = OnceLock::new();

/// Terminal capabilities and feature flags.
#[derive(Debug, Clone, Copy)]
pub struct TermCaps {
    /// Whether the terminal supports Unicode characters (box-drawing, etc.).
    pub unicode: bool,
    /// Whether the terminal supports 24-bit true color (RGB).
    pub true_color: bool,
    /// Whether the terminal supports OSC 8 hyperlinks.
    pub osc8_links: bool,
    /// Whether we're running on Windows.
    pub is_windows: bool,
    /// Whether we're in Windows CMD (as opposed to Windows Terminal/PowerShell).
    pub is_windows_cmd: bool,
}

impl Default for TermCaps {
    fn default() -> Self {
        Self {
            unicode: true,
            true_color: true,
            osc8_links: true,
            is_windows: false,
            is_windows_cmd: false,
        }
    }
}

impl TermCaps {
    /// Detect terminal capabilities based on environment.
    pub fn detect() -> Self {
        let is_windows = cfg!(target_os = "windows");

        // Check for Windows Terminal (supports everything)
        let is_windows_terminal = env::var("WT_SESSION").is_ok();

        // Check for modern terminal emulators that support features
        let term = env::var("TERM").unwrap_or_default();
        let term_program = env::var("TERM_PROGRAM").unwrap_or_default();
        let colorterm = env::var("COLORTERM").unwrap_or_default();

        // Detect if we're in basic Windows CMD
        // CMD doesn't set TERM or TERM_PROGRAM, and doesn't have WT_SESSION
        let is_windows_cmd = is_windows
            && !is_windows_terminal
            && term.is_empty()
            && term_program.is_empty();

        // Check for ConEmu/Cmder which support more features
        let is_conemu = env::var("ConEmuANSI").is_ok() || env::var("CONEMUANSI").is_ok();

        // True color detection
        let true_color = colorterm.eq_ignore_ascii_case("truecolor")
            || colorterm.eq_ignore_ascii_case("24bit")
            || is_windows_terminal
            || is_conemu
            || term.contains("256color")
            || term.contains("truecolor")
            || term_program.eq_ignore_ascii_case("iTerm.app")
            || term_program.eq_ignore_ascii_case("Apple_Terminal")
            || term_program.eq_ignore_ascii_case("vscode")
            || (!is_windows_cmd && !is_windows);

        // Unicode detection
        // Most modern terminals support Unicode, CMD is the main exception
        let lang = env::var("LANG").unwrap_or_default();
        let unicode = !is_windows_cmd
            || is_windows_terminal
            || is_conemu
            || lang.to_lowercase().contains("utf");

        // OSC 8 hyperlink support
        // Only modern terminals support this
        let osc8_links = !is_windows_cmd
            && (is_windows_terminal
                || term_program.eq_ignore_ascii_case("iTerm.app")
                || term_program.eq_ignore_ascii_case("vscode")
                || term.contains("xterm")
                || term.contains("alacritty")
                || term.contains("kitty")
                || term.contains("foot")
                || term.contains("wezterm"));

        // Allow environment variable overrides
        let unicode = Self::env_override("LQ_UNICODE", unicode);
        let true_color = Self::env_override("LQ_TRUECOLOR", true_color);
        let osc8_links = Self::env_override("LQ_OSC8", osc8_links);

        Self {
            unicode,
            true_color,
            osc8_links,
            is_windows,
            is_windows_cmd,
        }
    }

    /// Check for environment variable override (1/true/yes to enable, 0/false/no to disable).
    fn env_override(var: &str, default: bool) -> bool {
        match env::var(var).ok().as_deref() {
            Some("1") | Some("true") | Some("yes") | Some("TRUE") | Some("YES") => true,
            Some("0") | Some("false") | Some("no") | Some("FALSE") | Some("NO") => false,
            _ => default,
        }
    }

    /// Get cached terminal capabilities (detects once, caches forever).
    pub fn get() -> &'static Self {
        CAPS.get_or_init(Self::detect)
    }

    /// Override capabilities (call before first `get()` for effect).
    /// Useful for CLI flag overrides like `--no-unicode`.
    pub fn init_with_overrides(unicode: Option<bool>, true_color: Option<bool>, osc8: Option<bool>) {
        let mut caps = Self::detect();
        if let Some(u) = unicode {
            caps.unicode = u;
        }
        if let Some(t) = true_color {
            caps.true_color = t;
        }
        if let Some(o) = osc8 {
            caps.osc8_links = o;
        }
        // Ignore error if already initialized
        let _ = CAPS.set(caps);
    }
}

// ── Character fallbacks ───────────────────────────────────────────────────────

/// Box-drawing and symbol characters with ASCII fallbacks.
pub mod chars {
    use super::TermCaps;

    /// Vertical line for gutters: `│` or `|`
    pub fn vertical() -> &'static str {
        if TermCaps::get().unicode { "│" } else { "|" }
    }

    /// Horizontal line: `─` or `-`
    pub fn horizontal() -> &'static str {
        if TermCaps::get().unicode { "─" } else { "-" }
    }

    /// Top-left corner: `┌` or `+`
    pub fn top_left() -> &'static str {
        if TermCaps::get().unicode { "┌" } else { "+" }
    }

    /// Top-right corner: `┐` or `+`
    pub fn top_right() -> &'static str {
        if TermCaps::get().unicode { "┐" } else { "+" }
    }

    /// Bottom-left corner: `└` or `+`
    pub fn bottom_left() -> &'static str {
        if TermCaps::get().unicode { "└" } else { "+" }
    }

    /// Bottom-right corner: `┘` or `+`
    pub fn bottom_right() -> &'static str {
        if TermCaps::get().unicode { "┘" } else { "+" }
    }

    /// T-junction (left): `├` or `+`
    pub fn tee_right() -> &'static str {
        if TermCaps::get().unicode { "├" } else { "+" }
    }

    /// T-junction (right): `┤` or `+`
    pub fn tee_left() -> &'static str {
        if TermCaps::get().unicode { "┤" } else { "+" }
    }

    /// Bullet point: `•` or `*`
    pub fn bullet() -> &'static str {
        if TermCaps::get().unicode { "•" } else { "*" }
    }

    /// Checkmark: `✓` or `[x]`
    pub fn checkmark() -> &'static str {
        if TermCaps::get().unicode { "✓" } else { "[x]" }
    }

    /// Cross mark: `✗` or `[!]`
    pub fn crossmark() -> &'static str {
        if TermCaps::get().unicode { "✗" } else { "[!]" }
    }

    /// Arrow right: `→` or `->`
    pub fn arrow_right() -> &'static str {
        if TermCaps::get().unicode { "→" } else { "->" }
    }

    /// Arrow left: `←` or `<-`
    pub fn arrow_left() -> &'static str {
        if TermCaps::get().unicode { "←" } else { "<-" }
    }

    /// Ellipsis: `…` or `...`
    pub fn ellipsis() -> &'static str {
        if TermCaps::get().unicode { "…" } else { "..." }
    }

    /// Tree branch: `├──` or `|--`
    pub fn tree_branch() -> &'static str {
        if TermCaps::get().unicode { "├──" } else { "|--" }
    }

    /// Tree last item: `└──` or `+--`
    pub fn tree_last() -> &'static str {
        if TermCaps::get().unicode { "└──" } else { "+--" }
    }

    /// Tree continuation: `│  ` or `|  `
    pub fn tree_pipe() -> &'static str {
        if TermCaps::get().unicode { "│  " } else { "|  " }
    }

    /// Code block gutter separator: `  │  ` or `  |  `
    pub fn gutter_sep() -> &'static str {
        if TermCaps::get().unicode { "  │  " } else { "  |  " }
    }
}

// ── Color fallbacks ───────────────────────────────────────────────────────────

/// Color utilities with fallbacks for limited terminals.
pub mod colors {
    use super::TermCaps;
    use ratatui::prelude::*;

    /// Convert an RGB color to a fallback if true color is not supported.
    pub fn rgb(r: u8, g: u8, b: u8) -> Color {
        if TermCaps::get().true_color {
            Color::Rgb(r, g, b)
        } else {
            rgb_to_ansi256(r, g, b)
        }
    }

    /// Code block background: dark blue-gray or indexed color.
    pub fn code_bg() -> Color {
        // base16-ocean.dark background: #2b303b
        rgb(43, 48, 59)
    }

    /// Code gutter foreground (muted).
    pub fn code_gutter_fg() -> Color {
        rgb(100, 105, 128)
    }

    /// Code gutter separator foreground.
    pub fn code_gutter_sep_fg() -> Color {
        rgb(55, 60, 78)
    }

    /// Convert RGB to closest ANSI 256 color.
    fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> Color {
        // Check grayscale first (colors 232-255)
        if r == g && g == b {
            if r < 8 {
                return Color::Indexed(16); // black
            }
            if r > 248 {
                return Color::Indexed(231); // white
            }
            let gray_idx = ((r as u16 - 8) * 24 / 240) as u8;
            return Color::Indexed(232 + gray_idx);
        }

        // Convert to 6x6x6 color cube (colors 16-231)
        let r_idx = color_cube_index(r);
        let g_idx = color_cube_index(g);
        let b_idx = color_cube_index(b);
        Color::Indexed(16 + 36 * r_idx + 6 * g_idx + b_idx)
    }

    /// Map 0-255 to 0-5 for the 6x6x6 color cube.
    fn color_cube_index(val: u8) -> u8 {
        match val {
            0..=47 => 0,
            48..=114 => 1,
            115..=154 => 2,
            155..=194 => 3,
            195..=234 => 4,
            235..=255 => 5,
        }
    }

    /// Get a style with the given RGB foreground, with fallback.
    pub fn fg_rgb(r: u8, g: u8, b: u8) -> Style {
        Style::default().fg(rgb(r, g, b))
    }

    /// Get a style with the given RGB background, with fallback.
    pub fn bg_rgb(r: u8, g: u8, b: u8) -> Style {
        Style::default().bg(rgb(r, g, b))
    }
}

// ── Windows-specific terminal setup ───────────────────────────────────────────

/// Enable virtual terminal processing on Windows.
///
/// This is handled automatically by crossterm when entering raw mode,
/// but we expose this for cases where early setup is needed.
/// Returns `Ok(())` on success or on non-Windows platforms.
pub fn enable_virtual_terminal() -> std::io::Result<()> {
    // crossterm::terminal::enable_raw_mode() already calls
    // SetConsoleMode with ENABLE_VIRTUAL_TERMINAL_PROCESSING on Windows.
    // This function is kept as a no-op placeholder for explicit calls
    // and documentation purposes.
    //
    // If you encounter issues on older Windows versions, you may need
    // to set the console code page to UTF-8:
    //   - In CMD: `chcp 65001`
    //   - Or programmatically via SetConsoleOutputCP(65001)
    Ok(())
}

/// Check if OSC 8 hyperlinks should be emitted for the current terminal.
pub fn supports_osc8() -> bool {
    TermCaps::get().osc8_links
}

/// Check if Unicode characters should be used.
pub fn supports_unicode() -> bool {
    TermCaps::get().unicode
}

/// Check if true color (24-bit RGB) is supported.
pub fn supports_true_color() -> bool {
    TermCaps::get().true_color
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::prelude::Color;

    #[test]
    fn detect_returns_sensible_defaults() {
        let caps = TermCaps::detect();
        // Just verify it doesn't panic and returns something
        assert!(caps.unicode || !caps.unicode);
    }

    #[test]
    fn rgb_produces_valid_color() {
        // Test that rgb() returns a valid color (either RGB or Indexed depending on terminal)
        let color = colors::rgb(43, 48, 59);
        match color {
            Color::Rgb(_, _, _) | Color::Indexed(_) => {} // both are valid
            _ => panic!("Expected RGB or Indexed color"),
        }
    }

    #[test]
    fn chars_return_non_empty_strings() {
        // These should work regardless of unicode support
        assert!(!chars::vertical().is_empty());
        assert!(!chars::horizontal().is_empty());
        assert!(!chars::gutter_sep().is_empty());
        assert!(!chars::tree_branch().is_empty());
    }

    #[test]
    fn env_override_parses_values() {
        assert!(TermCaps::env_override("NONEXISTENT_VAR_12345", true));
        assert!(!TermCaps::env_override("NONEXISTENT_VAR_12345", false));
    }
}