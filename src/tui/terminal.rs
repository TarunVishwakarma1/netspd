//! Terminal lifecycle: raw mode, alternate screen and guaranteed restore.

use std::io::{self, Stdout};

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{cursor, execute};
use ratatui::backend::CrosstermBackend;
use ratatui::{Frame, Terminal};

/// An RAII guard around the terminal.
///
/// Construction enters raw mode and the alternate screen; dropping the
/// guard restores the user's terminal even when the application exits
/// through an error path.
pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Tui {
    /// Enters raw mode and the alternate screen.
    pub fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            EnableMouseCapture,
            cursor::Hide
        )?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        Ok(Self { terminal })
    }

    /// Draws one frame.
    pub fn draw(&mut self, render: impl FnOnce(&mut Frame)) -> io::Result<()> {
        self.terminal.draw(render)?;
        Ok(())
    }

    /// Restores the terminal to its normal state.
    ///
    /// Safe to call multiple times; also invoked on drop.
    pub fn restore() -> io::Result<()> {
        disable_raw_mode()?;
        execute!(
            io::stdout(),
            DisableMouseCapture,
            LeaveAlternateScreen,
            cursor::Show
        )?;
        Ok(())
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        // Best effort: a failure to restore here cannot be meaningfully
        // handled, and must not panic during unwinding.
        let _ = Self::restore();
    }
}

/// Chains a terminal restore in front of the default panic hook, so a
/// panic inside a dependency never leaves the terminal in raw mode.
pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = Tui::restore();
        default_hook(info);
    }));
}
