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

    /// Clears the terminal and forces ratatui to repaint every cell next frame.
    ///
    /// Call this after a phase that produces heavy terminal output (e.g. the
    /// live testing screen's dial canvas and progress bar). Dropped or delayed
    /// escape sequences during that phase can leave the terminal's cursor
    /// position out of sync with ratatui's model, causing text to land in
    /// wrong cells on the next render.
    ///
    /// Avoids `ratatui::Terminal::clear()` because that method internally
    /// queries the cursor position (`ESC[6n`) to verify alignment. If the pty
    /// input buffer is still draining post-test output, the read times out and
    /// returns "The cursor position could not be read within a normal duration".
    /// Instead we send the clear sequence directly and then invalidate
    /// ratatui's diff buffer by resizing it to the current dimensions.
    pub fn clear(&mut self) -> io::Result<()> {
        // ESC[2J clears the screen; ESC[H homes the cursor — neither queries
        // the terminal for its current position.
        execute!(
            io::stdout(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All),
            crossterm::cursor::MoveTo(0, 0)
        )?;
        // Invalidate ratatui's internal diff buffer so the next draw() call
        // re-renders every cell from scratch.  resize() updates the buffer
        // dimensions without any terminal I/O.
        if let Ok(size) = self.terminal.size() {
            let _ = self.terminal.resize(size.into());
        }
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
