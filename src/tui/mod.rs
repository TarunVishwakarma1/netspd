//! The presentation layer: terminal management, themes, animation,
//! widgets, screens and the frame renderer.
//!
//! Built exclusively on Ratatui and Crossterm. This layer reads
//! application state and draws it; it never talks to the network.

pub mod animation;
pub mod glyphs;
pub mod layout;
pub mod renderer;
pub mod screens;
pub mod terminal;
pub mod theme;
pub mod theme_registry;
pub mod wallpaper;
pub mod widgets;
