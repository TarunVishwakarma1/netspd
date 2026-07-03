//! Application configuration: schema and loading.

mod loader;
mod settings;

pub use loader::{load, load_from};
pub use settings::{EngineSection, ServerEntry, Settings};
