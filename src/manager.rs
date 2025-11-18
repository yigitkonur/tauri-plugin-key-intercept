//! Hotkey Manager - Thread-safe registry for managing multiple hotkeys

use crate::error::Error;
use crate::event_tap::EventTap;
use crate::models::{Hotkey, HotkeyId};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Runtime};

/// Main manager for registering and managing hotkeys
pub struct HotkeyManager {
    event_tap: Arc<Mutex<EventTap>>,
}

impl HotkeyManager {
    /// Create new hotkey manager
    pub fn new<R: Runtime>(app_handle: AppHandle<R>) -> Result<Self, Error> {
        #[cfg(target_os = "macos")]
        {
            let _ = std::fs::write("/tmp/f5-manager-init.txt", "HotkeyManager::new() called");
            let event_tap = EventTap::new(app_handle)?;
            let _ = std::fs::write("/tmp/f5-manager-init.txt", "EventTap::new() returned successfully");
            Ok(Self {
                event_tap: Arc::new(Mutex::new(event_tap)),
            })
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(Error::UnsupportedPlatform)
        }
    }

    /// Register a new hotkey
    pub fn register(&self, hotkey: Hotkey) -> Result<HotkeyId, Error> {
        #[cfg(target_os = "macos")]
        {
            let mut tap = self.event_tap.lock().map_err(|_| Error::LockError)?;
            tap.register(hotkey)
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(Error::UnsupportedPlatform)
        }
    }

    /// Unregister a hotkey
    pub fn unregister(&self, id: &HotkeyId) -> Result<(), Error> {
        #[cfg(target_os = "macos")]
        {
            let mut tap = self.event_tap.lock().map_err(|_| Error::LockError)?;
            tap.unregister(id)
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(Error::UnsupportedPlatform)
        }
    }

    /// Check if a hotkey is registered
    pub fn is_registered(&self, id: &HotkeyId) -> bool {
        #[cfg(target_os = "macos")]
        {
            self.event_tap
                .lock()
                .map(|tap| tap.is_registered(id))
                .unwrap_or(false)
        }

        #[cfg(not(target_os = "macos"))]
        {
            false
        }
    }

    /// Cleanup (called on drop)
    pub fn cleanup(&mut self) {
        log::info!("🧹 Cleaning up HotkeyManager");
    }
}

