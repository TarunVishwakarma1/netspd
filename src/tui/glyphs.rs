//! Glyph sets: rich Unicode by default, plain ASCII with `--ascii`.
//!
//! Widgets take their characters from here instead of hardcoding them,
//! so terminals and fonts without braille or block-drawing support get a
//! clean fallback rather than tofu boxes. The active set is chosen once
//! at startup and read everywhere, like a theme.

use std::sync::OnceLock;

use ratatui::symbols::border;

/// One coherent set of UI characters.
#[derive(Debug, Clone, Copy)]
pub struct Glyphs {
    /// Spinner animation frames.
    pub spinner: &'static [&'static str],
    /// Progress bar fill character.
    pub bar_fill: &'static str,
    /// Progress bar sub-cell fills, from thinnest to full.
    pub bar_partials: &'static [&'static str],
    /// Progress bar empty track character.
    pub bar_track: &'static str,
    /// Download direction marker.
    pub down: &'static str,
    /// Upload direction marker.
    pub up: &'static str,
    /// Latency marker.
    pub latency: &'static str,
    /// Success/completion marker.
    pub check: &'static str,
    /// Repeat/auto-restart marker.
    pub repeat: &'static str,
    /// List cursor marker.
    pub cursor: &'static str,
    /// Active list entry marker.
    pub active: &'static str,
    /// Border character set for cards and popups.
    pub border: border::Set<'static>,
    /// Whether braille/block art (dial, big digits, sparklines) is
    /// available.
    pub fancy: bool,
}

/// The default Unicode set.
pub const UNICODE: Glyphs = Glyphs {
    spinner: &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"],
    bar_fill: "█",
    bar_partials: &["▏", "▎", "▍", "▌", "▋", "▊", "▉", "█"],
    bar_track: "╌",
    down: "↓",
    up: "↑",
    latency: "⇄",
    check: "✓",
    repeat: "⟳",
    cursor: "▸ ",
    active: " ●",
    border: border::ROUNDED,
    fancy: true,
};

/// The plain ASCII fallback for `--ascii`.
pub const ASCII: Glyphs = Glyphs {
    spinner: &["|", "/", "-", "\\"],
    bar_fill: "#",
    bar_partials: &["#"],
    bar_track: "-",
    down: "v",
    up: "^",
    latency: "<>",
    check: "OK",
    repeat: "~",
    cursor: "> ",
    active: " *",
    border: border::Set {
        top_left: "+",
        top_right: "+",
        bottom_left: "+",
        bottom_right: "+",
        vertical_left: "|",
        vertical_right: "|",
        horizontal_top: "-",
        horizontal_bottom: "-",
    },
    fancy: false,
};

static ACTIVE: OnceLock<&'static Glyphs> = OnceLock::new();

/// Selects the glyph set for the whole session. Call once at startup;
/// later calls are ignored.
pub fn select(ascii: bool) {
    let _ = ACTIVE.set(if ascii { &ASCII } else { &UNICODE });
}

/// The active glyph set (Unicode until [`select`] is called).
#[must_use]
pub fn current() -> &'static Glyphs {
    ACTIVE.get().copied().unwrap_or(&UNICODE)
}

impl Glyphs {
    /// The spinner frame for a UI tick.
    #[must_use]
    pub fn spinner_frame(&self, tick: u64) -> &'static str {
        self.spinner[(tick as usize) % self.spinner.len()]
    }
}
