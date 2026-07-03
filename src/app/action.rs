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
    /// Leave the current overlay screen.
    Back,
    /// Move the selection cursor up.
    MoveUp,
    /// Move the selection cursor down.
    MoveDown,
    /// Confirm the current selection.
    Confirm,
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
}
