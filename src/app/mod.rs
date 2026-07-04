//! The application layer: state, events, actions and the runtime loop.
//!
//! Data flows one way: events (terminal, engine, timer) are reduced into
//! [`state::AppState`], and the renderer draws from that state. The UI
//! never mutates state directly.

pub mod action;
#[allow(clippy::module_inception)]
pub mod app;
pub mod cli;
pub mod controller;
pub mod event;
pub mod headless;
pub mod history;
pub mod serve;
pub mod share;
pub mod state;
