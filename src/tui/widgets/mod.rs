//! Reusable widgets, one per file.
//!
//! Widgets are plain render functions: they take a frame, an area, a theme
//! and the data they display. None of them reads global state, which keeps
//! every widget previewable and testable in isolation.

pub mod completion;
pub mod dial_gauge;
pub mod digits;
pub mod download_card;
pub mod error_popup;
pub mod footer;
pub mod header;
pub mod help_popup;
pub mod ping_card;
pub mod progress_bar;
pub mod speed_gauge;
pub mod spinner;
pub mod status_bar;
pub mod transfer_card;
pub mod upload_card;
