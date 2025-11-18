//! # tauri-plugin-macos-input-monitor
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

use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime, State,
};
use std::sync::{Arc, Mutex};

pub use models::*;
pub use constants::*;

mod commands;
mod error;
mod models;
mod constants;

#[cfg(target_os = "macos")]
mod event_tap;
#[cfg(target_os = "macos")]
mod manager;

pub use error::{Error, Result};

/// Plugin state wrapper
pub struct MacOSInputMonitor {
    #[cfg(target_os = "macos")]
    pub manager: Arc<Mutex<manager::HotkeyManager>>,
    
    #[cfg(not(target_os = "macos"))]
    _phantom: (),
}

/// Extension trait for ergonomic access to plugin APIs
pub trait MacOSInputMonitorExt<R: Runtime> {
  fn macos_input_monitor(&self) -> State<'_, MacOSInputMonitor>;
}

impl<R: Runtime, T: Manager<R>> MacOSInputMonitorExt<R> for T {
  fn macos_input_monitor(&self) -> State<'_, MacOSInputMonitor> {
    self.state::<MacOSInputMonitor>()
  }
}

/// Initialize the plugin
pub fn init<R: Runtime>() -> TauriPlugin<R, ()> {
  Builder::<R, ()>::new("macos-input-monitor")
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
        println!("🚀 Initializing macOS Input Monitor Plugin");
        let manager = manager::HotkeyManager::new(app.clone())?;
        app.manage(MacOSInputMonitor {
          manager: Arc::new(Mutex::new(manager)),
        });
        println!("✅ Plugin initialized successfully");
      }
      
      #[cfg(not(target_os = "macos"))]
      {
        log::warn!("⚠️  macOS Input Monitor plugin only works on macOS");
        app.manage(MacOSInputMonitor {});
      }
      
      Ok(())
    })
    .on_drop(|_app| {
      log::info!("🧹 Cleaning up macOS Input Monitor Plugin");
    })
    .build()
}
