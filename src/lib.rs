//! # tauri-plugin-key-intercept
//!
//! macOS-only Tauri plugin using CGEventTap FFI to intercept keyboard events
//! at hardware level BEFORE system shortcuts see them.
//!
//! ## Features
//! - Override system shortcuts (F5 dictation, F3 mission control, etc.)
//! - Raw CGEventTap FFI for maximum priority
//! - Proper event consumption (returns C NULL)
//! - Support for both standard and media key modes
//! - Developer utilities for keycode discovery

use std::sync::{Arc, Mutex};
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime, State,
};

pub use constants::*;
pub use models::*;

mod commands;
mod constants;
mod error;
mod models;

#[cfg(target_os = "macos")]
mod event_tap;
#[cfg(target_os = "macos")]
mod manager;

pub use error::{Error, Result};

/// Plugin state wrapper
pub struct KeyIntercept {
    #[cfg(target_os = "macos")]
    pub manager: Arc<Mutex<manager::HotkeyManager>>,

    #[cfg(not(target_os = "macos"))]
    _phantom: (),
}

/// Extension trait for ergonomic access to plugin APIs
pub trait KeyInterceptExt<R: Runtime> {
    fn key_intercept(&self) -> State<'_, KeyIntercept>;
}

impl<R: Runtime, T: Manager<R>> KeyInterceptExt<R> for T {
    fn key_intercept(&self) -> State<'_, KeyIntercept> {
        self.state::<KeyIntercept>()
    }
}

/// Initialize the plugin
pub fn init<R: Runtime>() -> TauriPlugin<R, ()> {
    Builder::<R, ()>::new("key-intercept")
        .invoke_handler(tauri::generate_handler![
            commands::register,
            commands::unregister,
            commands::is_registered,
            commands::get_keycode_table,
            commands::discover_keycode,
            commands::open_input_monitoring_settings,
            commands::check_permission,
        ])
        .setup(|app, _api| {
            #[cfg(target_os = "macos")]
            {
                println!("Initializing Key Intercept Plugin");
                let manager = manager::HotkeyManager::new(app.clone())?;
                app.manage(KeyIntercept {
                    manager: Arc::new(Mutex::new(manager)),
                });
                println!("Plugin initialized successfully");
            }

            #[cfg(not(target_os = "macos"))]
            {
                log::warn!("Key Intercept plugin only works on macOS");
                app.manage(KeyIntercept {});
            }

            Ok(())
        })
        .on_drop(|_app| {
            log::info!("Cleaning up Key Intercept Plugin");
        })
        .build()
}
