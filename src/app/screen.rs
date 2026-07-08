//! The `Screen` enum and its query methods.

/// The screen currently in the foreground.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Startup splash while servers are discovered.
    Splash,
    /// A test is running.
    Testing,
    /// Final results of a completed test.
    Results,
    /// Keyboard shortcut reference.
    Help,
    /// Read-only view of the active configuration.
    Settings,
    /// Server selection list.
    ServerSelect,
    /// Theme selection list.
    ThemeSelect,
    /// Past results and trends.
    Trends,
    /// A fatal test error.
    Error,
}

impl Screen {
    /// Whether this screen is an overlay that returns to a parent screen.
    #[must_use]
    pub fn is_overlay(self) -> bool {
        matches!(
            self,
            Self::Help | Self::Settings | Self::ServerSelect | Self::ThemeSelect | Self::Trends
        )
    }

    /// Whether this screen animates continuously and needs steady redraws.
    #[must_use]
    pub fn is_animated(self) -> bool {
        matches!(self, Self::Splash | Self::Testing)
    }
}
