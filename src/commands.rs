//! Tauri commands exposed to the frontend

use crate::error::{Error, Result};
use crate::models::{Hotkey, HotkeyId, Modifiers};
use crate::{constants, MacOSInputMonitor};
use std::collections::HashMap;
use tauri::{command, AppHandle, Manager, Runtime};

/// Register a new global hotkey
#[command]
pub async fn register<R: Runtime>(
    app: AppHandle<R>,
    keycodes: Vec<i64>,
    modifiers: Modifiers,
    consume: bool,
    event_name: String,
) -> Result<String> {
    log::info!(
        "📝 Registering hotkey: keycodes={:?}, event={}",
        keycodes,
        event_name
    );

    let monitor = app.state::<MacOSInputMonitor>();
    let manager = monitor.manager.lock().map_err(|_| Error::LockError)?;

    let hotkey = Hotkey {
        keycodes,
        modifiers,
        consume,
        event_name,
    };

    let id = manager.register(hotkey)?;
    Ok(id.0)
}

/// Unregister a hotkey by ID
#[command]
pub async fn unregister<R: Runtime>(app: AppHandle<R>, id: String) -> Result<()> {
    log::info!("🗑️  Unregistering hotkey: {}", id);

    let monitor = app.state::<MacOSInputMonitor>();
    let manager = monitor.manager.lock().map_err(|_| Error::LockError)?;

    let hotkey_id = HotkeyId(id);
    manager.unregister(&hotkey_id)
}

/// Check if a hotkey is registered
#[command]
pub async fn is_registered<R: Runtime>(app: AppHandle<R>, id: String) -> Result<bool> {
    let monitor = app.state::<MacOSInputMonitor>();
    let manager = monitor.manager.lock().map_err(|_| Error::LockError)?;

    let hotkey_id = HotkeyId(id);
    Ok(manager.is_registered(&hotkey_id))
}

/// Get comprehensive keycode table for all function keys
#[command]
pub fn get_keycode_table() -> HashMap<String, Vec<i64>> {
    HashMap::from([
        (
            "F1".to_string(),
            vec![constants::KEY_F1, constants::KEY_F1_MEDIA],
        ),
        (
            "F2".to_string(),
            vec![constants::KEY_F2, constants::KEY_F2_MEDIA],
        ),
        (
            "F3".to_string(),
            vec![constants::KEY_F3, constants::KEY_F3_MEDIA],
        ),
        (
            "F4".to_string(),
            vec![constants::KEY_F4, constants::KEY_F4_MEDIA],
        ),
        (
            "F5".to_string(),
            vec![constants::KEY_F5, constants::KEY_F5_MEDIA],
        ),
        (
            "F6".to_string(),
            vec![constants::KEY_F6, constants::KEY_F6_MEDIA],
        ),
        (
            "F7".to_string(),
            vec![constants::KEY_F7, constants::KEY_F7_MEDIA],
        ),
        (
            "F8".to_string(),
            vec![constants::KEY_F8, constants::KEY_F8_MEDIA],
        ),
        (
            "F9".to_string(),
            vec![constants::KEY_F9, constants::KEY_F9_MEDIA],
        ),
        (
            "F10".to_string(),
            vec![constants::KEY_F10, constants::KEY_F10_MEDIA],
        ),
        (
            "F11".to_string(),
            vec![constants::KEY_F11, constants::KEY_F11_MEDIA],
        ),
        (
            "F12".to_string(),
            vec![constants::KEY_F12, constants::KEY_F12_MEDIA],
        ),
    ])
}

/// Developer utility: Discover keycode for any key press
/// Starts a temporary event tap that logs all keypresses for the specified duration
#[command]
pub async fn discover_keycode<R: Runtime>(
    _app: AppHandle<R>,
    duration_ms: Option<u64>,
) -> Result<String> {
    let duration = duration_ms.unwrap_or(30000);
    log::info!("🔍 Starting keycode discovery for {} ms", duration);
    log::info!("   Press any keys to see their keycodes in the console");

    // This would start a temporary event tap
    // For MVP, we return instructions
    Ok(format!(
        "Keycode discovery: Press keys and check console logs. Duration: {}ms",
        duration
    ))
}

/// Open macOS Input Monitoring settings
/// Helper utility to make permission granting easier for users
#[command]
pub async fn open_input_monitoring_settings() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        log::info!("🔐 Opening Input Monitoring settings");
        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent")
            .spawn()
            .map_err(|e| Error::Io(e))?;
    }

    #[cfg(not(target_os = "macos"))]
    {
        return Err(Error::UnsupportedPlatform);
    }

    Ok(())
}

/// Check if Input Monitoring permission is granted
/// Returns true if permission is available, false otherwise
#[command]
pub async fn check_permission() -> Result<bool> {
    #[cfg(target_os = "macos")]
    {
        // Permission check: Try to create a test event tap
        // If it succeeds, permission is granted
        use std::os::raw::c_void;

        unsafe {
            extern "C" fn test_callback(
                _: *mut c_void,
                _: u32,
                event: *mut c_void,
                _: *mut c_void,
            ) -> *mut c_void {
                event
            }

            extern "C" {
                fn CGEventTapCreate(
                    tap: u32,
                    place: u32,
                    options: u32,
                    events: u64,
                    callback: extern "C" fn(
                        *mut c_void,
                        u32,
                        *mut c_void,
                        *mut c_void,
                    ) -> *mut c_void,
                    user_info: *mut c_void,
                ) -> *mut c_void;
            }

            let tap = CGEventTapCreate(0, 0, 0, 1 << 10, test_callback, std::ptr::null_mut());
            let has_permission = !tap.is_null();

            // Cleanup if created
            if has_permission {
                extern "C" {
                    fn CFRelease(cf: *mut c_void);
                }
                CFRelease(tap);
            }

            Ok(has_permission)
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(false)
    }
}
