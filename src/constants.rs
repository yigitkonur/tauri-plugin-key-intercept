//! macOS Virtual Key Code Constants and Modifier Flags
//!
//! This module provides comprehensive keycode mappings for macOS keyboard events.
//! macOS has TWO modes for function keys, resulting in different keycodes:
//! - Standard Function Keys Mode: F5 = 96
//! - Media Keys Mode (default on MacBooks): F5 = 176
//!
//! # Example
//! ```rust
//! use tauri_plugin_macos_input_monitor::*;
//!
//! // Use both keycodes to support all Mac configurations
//! let f5_codes = vec![KEY_F5, KEY_F5_MEDIA];
//! ```

/// F1 keycode in standard function keys mode
pub const KEY_F1: i64 = 122;
/// F2 keycode in standard function keys mode
pub const KEY_F2: i64 = 120;
/// F3 keycode in standard function keys mode
pub const KEY_F3: i64 = 99;
/// F4 keycode in standard function keys mode
pub const KEY_F4: i64 = 118;
/// F5 keycode in standard function keys mode
pub const KEY_F5: i64 = 96;
/// F6 keycode in standard function keys mode
pub const KEY_F6: i64 = 97;
/// F7 keycode in standard function keys mode
pub const KEY_F7: i64 = 98;
/// F8 keycode in standard function keys mode
pub const KEY_F8: i64 = 100;
/// F9 keycode in standard function keys mode
pub const KEY_F9: i64 = 101;
/// F10 keycode in standard function keys mode
pub const KEY_F10: i64 = 109;
/// F11 keycode in standard function keys mode
pub const KEY_F11: i64 = 103;
/// F12 keycode in standard function keys mode
pub const KEY_F12: i64 = 111;

/// F1 keycode in media keys mode (Brightness down)
pub const KEY_F1_MEDIA: i64 = 145; // Brightness down
pub const KEY_F2_MEDIA: i64 = 144; // Brightness up
pub const KEY_F3_MEDIA: i64 = 160; // Mission Control/Exposé
pub const KEY_F4_MEDIA: i64 = 131; // Launchpad
pub const KEY_F5_MEDIA: i64 = 176; // Keyboard brightness down
pub const KEY_F6_MEDIA: i64 = 177; // Keyboard brightness up
pub const KEY_F7_MEDIA: i64 = 180; // Rewind
pub const KEY_F8_MEDIA: i64 = 179; // Play/Pause
pub const KEY_F9_MEDIA: i64 = 178; // Fast forward
pub const KEY_F10_MEDIA: i64 = 173; // Mute
pub const KEY_F11_MEDIA: i64 = 174; // Volume down
pub const KEY_F12_MEDIA: i64 = 175; // Volume up

// Modifier Flag Bit Masks (as hex values in event flags)
pub const CMD_FLAG: u64 = 0x100000; // Command/Super key
pub const OPT_FLAG: u64 = 0x80000; // Option/Alt key
pub const CTRL_FLAG: u64 = 0x40000; // Control key
pub const SHIFT_FLAG: u64 = 0x20000; // Shift key

// Internal macOS flags (should be IGNORED when matching hotkeys)
pub const SECONDARY_FN_FLAG: u64 = 0x800000; // Fn key indicator
pub const CAPS_LOCK_FLAG: u64 = 0x10000; // Caps Lock
pub const NUM_PAD_FLAG: u64 = 0x200000; // Numeric keypad
pub const NON_COALESCED_FLAG: u64 = 0x100; // Internal event flag

// CGEventTap C API Constants
pub const CG_HID_EVENT_TAP: u32 = 0;
pub const CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
pub const CG_EVENT_TAP_OPTION_DEFAULT: u32 = 0;
pub const CG_EVENT_KEY_DOWN: u32 = 10;
pub const CG_EVENT_KEY_UP: u32 = 11;
pub const CG_EVENT_FLAGS_CHANGED: u32 = 12;
pub const CG_KEYBOARD_EVENT_KEYCODE: u32 = 9;

/// Helper to get both keycodes for a function key
pub fn get_function_key_codes(key_number: u8) -> Vec<i64> {
    match key_number {
        1 => vec![KEY_F1, KEY_F1_MEDIA],
        2 => vec![KEY_F2, KEY_F2_MEDIA],
        3 => vec![KEY_F3, KEY_F3_MEDIA],
        4 => vec![KEY_F4, KEY_F4_MEDIA],
        5 => vec![KEY_F5, KEY_F5_MEDIA],
        6 => vec![KEY_F6, KEY_F6_MEDIA],
        7 => vec![KEY_F7, KEY_F7_MEDIA],
        8 => vec![KEY_F8, KEY_F8_MEDIA],
        9 => vec![KEY_F9, KEY_F9_MEDIA],
        10 => vec![KEY_F10, KEY_F10_MEDIA],
        11 => vec![KEY_F11, KEY_F11_MEDIA],
        12 => vec![KEY_F12, KEY_F12_MEDIA],
        _ => vec![],
    }
}

/// Extract only user-intentional modifiers from raw event flags
pub fn extract_user_modifiers(raw_flags: u64) -> u64 {
    raw_flags & (CMD_FLAG | OPT_FLAG | CTRL_FLAG | SHIFT_FLAG)
}
