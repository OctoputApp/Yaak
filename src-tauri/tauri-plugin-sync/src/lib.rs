use tauri::{generate_handler, plugin::{Builder, TauriPlugin}, Runtime};

use commands::*;
pub use models::*;

mod commands;
mod error;
mod models;
mod sync;
mod diff;
mod queries;

pub use error::{Error, Result};

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("sync")
        .invoke_handler(generate_handler![commits, commit, changes])
        .build()
}
