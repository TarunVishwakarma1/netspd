//! netspd — a beautiful, modern, minimalistic network speed testing
//! terminal application.
//!
//! The crate is layered strictly:
//!
//! - [`tui`] (presentation) draws [`app`] state with Ratatui;
//! - [`app`] (application) reduces events into state and runs the loop;
//! - [`engine`] (domain) measures ping, download and upload through a
//!   provider abstraction, with no knowledge of any UI;
//! - [`config`], [`errors`] and [`utils`] support all layers.
//!
//! The engine is reusable on its own: a CLI, GUI or REST API can drive
//! [`engine::Engine`] and consume [`engine::EngineEvent`]s without pulling
//! in any terminal code.

pub mod app;
pub mod config;
pub mod engine;
pub mod errors;
pub mod tui;
pub mod utils;
