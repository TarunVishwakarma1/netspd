//! User intents and the side effects they request.

/// A high-level user intent, decoded from raw key events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Exit the application.
    Quit,
    /// Restart the speed test.
    Restart,
    /// Open the help screen.
    ShowHelp,
    /// Open the settings screen.
    ShowSettings,
    /// Open the server selection screen.
    ShowServers,
    /// Open the theme selection screen.
    ShowThemes,
    /// Open the result trends screen.
    ShowTrends,
    /// Copy the last result to the clipboard.
    Share,
    /// Leave the current overlay screen.
    Back,
    /// Move the selection cursor up.
    MoveUp,
    /// Move the selection cursor down.
    MoveDown,
    /// Adjust the focused value down / cycle the filter left.
    MoveLeft,
    /// Adjust the focused value up / cycle the filter right.
    MoveRight,
    /// Confirm the current selection.
    Confirm,
    /// Write the current settings to the config file.
    SaveConfig,
}

/// A side effect the controller asks the runtime to perform.
///
/// The controller itself only mutates state; anything that spawns tasks or
/// touches the outside world is returned as a command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    /// Nothing to do.
    None,
    /// Terminate the application loop.
    Quit,
    /// Cancel any running test and start a new one.
    StartTest,
    /// Load stored results for the trends screen.
    LoadTrends,
    /// Copy the last result to the clipboard.
    Share,
    /// Persist settings to the config file.
    SaveConfig,
    /// Rebuild the engine for the newly selected provider and rediscover
    /// servers.
    ReloadProvider,
}
