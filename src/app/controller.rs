//! Key decoding and action handling.
//!
//! The controller is the only place that interprets input. It mutates
//! state synchronously and returns a [`Command`] for anything that needs a
//! side effect (spawning a test, quitting), which the runtime executes.

use crossterm::event::{KeyCode, KeyEvent};

use super::action::{Action, Command};
use super::state::{AppState, Screen};

/// Decodes a key event into an [`Action`] for the current screen.
#[must_use]
pub fn map_key(screen: Screen, key: KeyEvent) -> Option<Action> {
    match key.code {
        KeyCode::Char('q') => Some(Action::Quit),
        KeyCode::Esc => {
            if screen.is_overlay() {
                Some(Action::Back)
            } else {
                Some(Action::Quit)
            }
        }
        KeyCode::Char('r') => Some(Action::Restart),
        KeyCode::Char('s') => Some(Action::ShowServers),
        KeyCode::Char('t') => Some(Action::ShowThemes),
        KeyCode::Char('c') => Some(Action::ShowSettings),
        KeyCode::Char('?') => Some(Action::ShowHelp),
        KeyCode::Up | KeyCode::Char('k') => Some(Action::MoveUp),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::MoveDown),
        KeyCode::Enter => Some(Action::Confirm),
        _ => None,
    }
}

/// Applies an action to the state and returns the requested side effect.
pub fn handle(state: &mut AppState, action: Action) -> Command {
    state.request_redraw();
    match action {
        Action::Quit => Command::Quit,
        Action::Restart => {
            if state.servers.is_empty() {
                Command::None
            } else {
                Command::StartTest
            }
        }
        Action::ShowHelp => {
            open_overlay(state, Screen::Help);
            Command::None
        }
        Action::ShowSettings => {
            open_overlay(state, Screen::Settings);
            Command::None
        }
        Action::ShowServers => {
            state.server_cursor = state
                .server_index
                .min(state.servers.len().saturating_sub(1));
            open_overlay(state, Screen::ServerSelect);
            Command::None
        }
        Action::ShowThemes => {
            state.theme_cursor = state.theme_index;
            open_overlay(state, Screen::ThemeSelect);
            Command::None
        }
        Action::Back => {
            if state.screen.is_overlay() {
                state.screen = state.return_to;
            }
            Command::None
        }
        Action::MoveUp => {
            move_cursor(state, -1);
            Command::None
        }
        Action::MoveDown => {
            move_cursor(state, 1);
            Command::None
        }
        Action::Confirm => confirm(state),
    }
}

/// Switches to an overlay screen, remembering where to return.
fn open_overlay(state: &mut AppState, overlay: Screen) {
    if state.screen == overlay {
        return;
    }
    if !state.screen.is_overlay() {
        state.return_to = state.screen;
    }
    state.screen = overlay;
}

/// Moves the list cursor on selection screens.
fn move_cursor(state: &mut AppState, delta: i64) {
    let (cursor, len) = match state.screen {
        Screen::ServerSelect => (&mut state.server_cursor, state.servers.len()),
        Screen::ThemeSelect => (&mut state.theme_cursor, state.theme_names.len()),
        _ => return,
    };
    if len == 0 {
        return;
    }
    let last = len - 1;
    *cursor = if delta < 0 {
        cursor.saturating_sub(1)
    } else {
        (*cursor + 1).min(last)
    };
}

/// Confirms the highlighted entry on selection screens.
fn confirm(state: &mut AppState) -> Command {
    match state.screen {
        Screen::ServerSelect => {
            if state.servers.is_empty() {
                return Command::None;
            }
            state.server_index = state.server_cursor.min(state.servers.len() - 1);
            state.screen = state.return_to;
            Command::StartTest
        }
        Screen::ThemeSelect => {
            if !state.theme_names.is_empty() {
                state.theme_index = state.theme_cursor.min(state.theme_names.len() - 1);
            }
            state.screen = state.return_to;
            Command::None
        }
        _ => Command::None,
    }
}
