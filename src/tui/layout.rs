//! Responsive layout helpers.

use ratatui::layout::Rect;

/// Coarse terminal-size classes the screens adapt to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Breakpoint {
    /// Small terminals (~80x24): stacked cards, tight spacing.
    Compact,
    /// Medium terminals (~100x30): side-by-side cards.
    Normal,
    /// Large terminals (120x40 and beyond): generous whitespace.
    Wide,
}

/// Classifies a terminal area into a [`Breakpoint`].
#[must_use]
pub fn breakpoint(area: Rect) -> Breakpoint {
    if area.width >= 120 && area.height >= 36 {
        Breakpoint::Wide
    } else if area.width >= 96 && area.height >= 28 {
        Breakpoint::Normal
    } else {
        Breakpoint::Compact
    }
}

/// Centers a `width` x `height` box inside `area`, clamping to fit.
#[must_use]
pub fn centered(area: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}

/// Shrinks an area by a horizontal and vertical margin on each side.
#[must_use]
pub fn padded(area: Rect, horizontal: u16, vertical: u16) -> Rect {
    let dx = horizontal.min(area.width / 2);
    let dy = vertical.min(area.height / 2);
    Rect {
        x: area.x + dx,
        y: area.y + dy,
        width: area.width - dx * 2,
        height: area.height - dy * 2,
    }
}
