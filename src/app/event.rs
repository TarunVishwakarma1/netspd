//! The unified event type driving the application loop.

use crossterm::event::{KeyEvent, MouseEventKind};

use crate::engine::models::Server;
use crate::engine::EngineEvent;

/// Everything that can wake the application loop.
///
/// All input sources — keyboard, engine, timers, background discovery —
/// are normalized into this one type before being reduced into state.
#[derive(Debug)]
pub enum AppEvent {
    /// A key press from the terminal.
    Key(KeyEvent),
    /// A mouse action from the terminal.
    Mouse(MouseEventKind),
    /// The terminal was resized.
    Resize,
    /// A UI timer tick.
    Tick,
    /// An event from the running speed test.
    Engine(EngineEvent),
    /// Background server discovery finished.
    ServersLoaded(Result<Vec<Server>, String>),
    /// The client's public IP/ISP was discovered.
    ClientInfo(String),
}
