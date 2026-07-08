//! Application configuration: schema and loading.

mod loader;
mod settings;

pub use loader::{load, load_from, save};
pub use settings::{EngineSection, ServerEntry, Settings, WallpaperKind, WallpaperSection};
