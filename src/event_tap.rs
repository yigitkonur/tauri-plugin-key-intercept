//! Low-level CGEventTap Implementation for macOS
//!
//! This module provides raw FFI bindings to macOS CoreGraphics CGEventTap API.
//! It intercepts keyboard events at the hardware level BEFORE system shortcuts see them.

use crate::constants::*;
use crate::error::Error;
use crate::models::{Hotkey, HotkeyId, KeypressEvent};
use std::collections::HashMap;
use std::os::raw::c_void;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, AppHandle, Runtime};

// CGEvent opaque types for FFI
type CGEventRef = *mut c_void;
type CGEventTapProxy = *mut c_void;
type CFMachPortRef = *mut c_void;
type CFRunLoopSourceRef = *mut c_void;
type CFAllocatorRef = *mut c_void;

// Raw CoreGraphics C API bindings
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        events_of_interest: u64,
        callback: extern "C" fn(CGEventTapProxy, u32, CGEventRef, *mut c_void) -> CGEventRef,
        user_info: *mut c_void,
    ) -> CFMachPortRef;

    fn CGEventTapEnable(tap: CFMachPortRef, enable: bool);
    fn CGEventGetIntegerValueField(event: CGEventRef, field: u32) -> i64;
    fn CGEventGetFlags(event: CGEventRef) -> u64;
    fn CFMachPortCreateRunLoopSource(
        allocator: CFAllocatorRef,
        port: CFMachPortRef,
        order: i32,
    ) -> CFRunLoopSourceRef;
}

use core_foundation::base::TCFType;
use core_foundation::runloop::{kCFRunLoopCommonModes, CFRunLoop, CFRunLoopSource};

// Global state for callback access
static mut EVENT_TAP_STATE: Option<Arc<Mutex<EventTapState>>> = None;
static mut APP_HANDLE: *const () = std::ptr::null();

struct EventTapState {
    hotkeys: HashMap<HotkeyId, Hotkey>,
}

/// CGEventTap wrapper managing low-level keyboard interception
pub struct EventTap {
    state: Arc<Mutex<EventTapState>>,
}

// SAFETY: EventTap is Send + Sync because:
// - The actual CFMachPortRef tap lives in a dedicated thread
// - State is protected by Arc<Mutex<>>  
// - We never share raw pointers across threads
unsafe impl Send for EventTap {}
unsafe impl Sync for EventTap {}

impl EventTap {
    /// Create new event tap with hardware-level interception  
    pub fn new<R: Runtime>(app_handle: AppHandle<R>) -> Result<Self, Error> {
        // Write to file immediately to confirm we reach this point
        let _ = std::fs::write("/tmp/f5-listener-init.txt", "EventTap::new() called");
        
        let state = Arc::new(Mutex::new(EventTapState {
            hotkeys: HashMap::new(),
        }));

        unsafe {
            EVENT_TAP_STATE = Some(state.clone());
            // Store AppHandle as raw pointer (unsafe but necessary for C callback)
            let app_ptr = Box::into_raw(Box::new(app_handle));
            APP_HANDLE = app_ptr as *const ();
        }

        let _ = std::fs::write("/tmp/f5-listener-init.txt", "About to spawn thread");

        // Spawn dedicated thread for event tap (needs own run loop)
        std::thread::spawn(move || unsafe {
            let _ = std::fs::write("/tmp/f5-listener-init.txt", "Thread started!");
            
            #[cfg(debug_assertions)]
            {
                println!("\n========================================");
                println!("🚀 STARTING CGEventTap - RAW FFI MODE");
                println!("========================================");
                println!("Configuration:");
                println!("  • Location: HID (Hardware/System-wide)");
                println!("  • Placement: HeadInsertEventTap (Highest priority)");
                println!("  • Returns: NULL to consume events");
                println!("========================================\n");
            }
            #[cfg(not(debug_assertions))]
            {
                println!("🚀 Starting CGEventTap...");
            }

            // Create event mask for key down events only
            let event_mask: u64 = 1 << CG_EVENT_KEY_DOWN;

            // Create the event tap using raw C API
            let tap = CGEventTapCreate(
                CG_HID_EVENT_TAP,
                CG_HEAD_INSERT_EVENT_TAP,
                CG_EVENT_TAP_OPTION_DEFAULT,
                event_mask,
                event_callback,
                std::ptr::null_mut(),
            );

            if tap.is_null() {
                eprintln!("❌ CRITICAL: Failed to create CGEventTap!");
                eprintln!("   This means Input Monitoring permission is NOT granted");
                eprintln!("   Grant permission in: System Settings → Privacy & Security → Input Monitoring");
                eprintln!("   Then RESTART this app");
                
                // Write to file for debugging release builds
                let _ = std::fs::write("/tmp/f5-listener-error.txt", 
                    "CGEventTap creation FAILED - Input Monitoring permission not granted or app needs restart");
                return;
            }
            
            // Write success status
            let _ = std::fs::write("/tmp/f5-listener-status.txt", 
                "CGEventTap created successfully - permission granted");

            #[cfg(debug_assertions)]
            println!("✅ Step 1: CGEventTap created");

            // Create run loop source
            let run_loop_source = CFMachPortCreateRunLoopSource(std::ptr::null_mut(), tap, 0);

            if run_loop_source.is_null() {
                eprintln!("❌ Failed to create run loop source");
                return;
            }

            // Get current run loop and add source
            let run_loop = CFRunLoop::get_current();
            let source = CFRunLoopSource::wrap_under_create_rule(run_loop_source as *mut _);
            run_loop.add_source(&source, kCFRunLoopCommonModes);

            // Enable the tap
            CGEventTapEnable(tap, true);
            
            println!("✅ Event tap ready - listening for hotkeys");

            // Run the loop (blocks forever)
            CFRunLoop::run_current();
        });

        // Return event tap (actual tap runs on dedicated thread)
        Ok(Self { state })
    }

    /// Register a new hotkey
    pub fn register(&mut self, hotkey: Hotkey) -> Result<HotkeyId, Error> {
        let id = HotkeyId::new();
        let mut state = self.state.lock().map_err(|_| Error::LockError)?;
        state.hotkeys.insert(id.clone(), hotkey);
        log::info!("✅ Registered hotkey: {}", id.0);
        Ok(id)
    }

    /// Unregister a hotkey
    pub fn unregister(&mut self, id: &HotkeyId) -> Result<(), Error> {
        let mut state = self.state.lock().map_err(|_| Error::LockError)?;
        state.hotkeys.remove(id).ok_or(Error::HotkeyNotFound)?;
        log::info!("✅ Unregistered hotkey: {}", id.0);
        Ok(())
    }

    /// Check if hotkey is registered
    pub fn is_registered(&self, id: &HotkeyId) -> bool {
        self.state
            .lock()
            .map(|state| state.hotkeys.contains_key(id))
            .unwrap_or(false)
    }
}

// Raw C callback - called by CoreGraphics for every keyboard event
extern "C" fn event_callback(
    _proxy: CGEventTapProxy,
    event_type: u32,
    event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    unsafe {
        // Only process key down events
        if event_type != CG_EVENT_KEY_DOWN {
            return event;
        }

        let keycode = CGEventGetIntegerValueField(event, CG_KEYBOARD_EVENT_KEYCODE);
        let raw_flags = CGEventGetFlags(event);

        // Extract user-intentional modifiers only (strip internal macOS flags)
        let user_modifiers = extract_user_modifiers(raw_flags);

        // Check all registered hotkeys
        let state_ptr = std::ptr::addr_of!(EVENT_TAP_STATE);
        if let Some(state_arc) = &*state_ptr {
            if let Ok(state) = state_arc.lock() {
                for (_id, hotkey) in &state.hotkeys {
                    // Check if keycode matches any of the registered keycodes
                    if hotkey.keycodes.contains(&keycode) {
                        // Check if modifiers match
                        let expected_modifiers = hotkey.get_modifier_flags();
                        
                        if user_modifiers == expected_modifiers {
                            #[cfg(debug_assertions)]
                            {
                                println!("🎯 Hotkey matched! keycode: {}, modifiers: 0x{:x}", keycode, user_modifiers);
                                println!("   Event: {}", hotkey.event_name);
                            }

                            // Emit event to frontend via AppHandle
                            let event_name = hotkey.event_name.clone();
                            if !APP_HANDLE.is_null() {
                                // SAFETY: APP_HANDLE points to a valid AppHandle stored during init
                                let app_handle = &*(APP_HANDLE as *const AppHandle);
                                let handle = app_handle.clone();
                                let event_data = KeypressEvent {
                                    keycode,
                                    raw_flags,
                                    user_modifiers,
                                };
                                let event_name_for_spawn = event_name.clone();
                                
                                tauri::async_runtime::spawn(async move {
                                    let _ = handle.emit(&event_name_for_spawn, event_data);
                                });
                            }

                            // If consume is true, return NULL to block system
                            if hotkey.consume {
                                #[cfg(debug_assertions)]
                                println!("   🚫 Consumed\n");
                                
                                return std::ptr::null_mut();
                            }
                        }
                    }
                }
            }
        }

        // Pass through if no match or consume=false
        event
    }
}

impl Drop for EventTap {
    fn drop(&mut self) {
        log::info!("🧹 Cleaning up EventTap");
        unsafe {
            EVENT_TAP_STATE = None;
        }
    }
}

