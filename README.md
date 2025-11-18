# tauri-plugin-macos-input-monitor

> **macOS-only Tauri plugin using CGEventTap FFI to intercept keyboard events at hardware level, enabling override of system shortcuts.**

[![Crates.io](https://img.shields.io/crates/v/tauri-plugin-macos-input-monitor.svg)](https://crates.io/crates/tauri-plugin-macos-input-monitor)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

## 🚀 Quick Start (2 Minutes)

**Want to see it work immediately?**

```bash
git clone https://github.com/yigitkonur/tauri-plugin-macos-input-monitor
cd tauri-plugin-macos-input-monitor/examples/vanilla
pnpm install
pnpm tauri dev
```

Press **F5** - dictation blocked! ✅  
(Grant Input Monitoring permission when prompted, then restart)

![Example App Demo](docs/images/example-app-demo.png)

**What you're seeing:** The example app demonstrates overriding macOS F5 dictation shortcut - a perfect real-world use case. Normally, pressing F5 triggers the "Enable Dictation?" popup. With this plugin, F5 is intercepted at hardware level BEFORE macOS sees it, giving your app full control. The green indicator confirms Input Monitoring permission is granted, and the event log shows real-time F5 detection with timestamps.

---

## ⚠️ Critical: Dev Mode vs Release Mode

| Mode | Status | Launch Method |
|------|--------|---------------|
| **Dev** (`pnpm tauri dev`) | ✅ Works perfectly | Automatic |
| **Release** (without Developer ID) | ⚠️ Needs special launch | Direct binary only |

**Dev mode always works.** Release builds require special handling - see [Launch Services Limitation](#️-important-launch-services-limitation-release-builds) below.

---

## Why This Plugin Exists

### The Problem

macOS system shortcuts (F5 dictation, F3 mission control, etc.) cannot be overridden by standard keyboard APIs. The popular `tauri-plugin-global-shortcut` uses `RegisterEventHotKey`, which operates at **application level** with low priority. System shortcuts always win.

**Developers have no way to:**
- Override F5 to prevent dictation popup
- Intercept F3 before mission control
- Use function keys for custom app actions

### The Solution

This plugin uses **raw CGEventTap FFI** with `HeadInsertEventTap` to intercept keyboard events at the **hardware level**, BEFORE macOS system handlers see them.

```
Keyboard Hardware
       ↓
CGEventTap (HID, HeadInsert) ← Plugin intercepts HERE  
       ↓
System Shortcuts (dictation, etc.) ← Never receives event!
       ↓
Application Shortcuts
       ↓
Window receives event
```

## Installation

**Cargo.toml:**
```toml
[dependencies]
tauri-plugin-macos-input-monitor = "0.1"
```

**src-tauri/src/lib.rs:**
```rust
use tauri_plugin_macos_input_monitor::{MacOSInputMonitorExt, Modifiers};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri-plugin-macos-input-monitor::init())
        .setup(|app| {
            // Override F5 dictation shortcut
            let hotkey = tauri_plugin_macos_input_monitor::Hotkey {
                keycodes: vec![96, 176], // F5 in both keyboard modes
                modifiers: Modifiers::empty(),
                consume: true, // Block system from seeing it
                event_name: "f5-pressed".to_string(),
            };
            
            let manager = app.macos_input_monitor().manager.lock().unwrap();
            let id = manager.register(hotkey)?;
            println!("F5 hotkey registered: {}", id.0);
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Required Permissions

**⚠️ Critical:** Your app MUST have **Input Monitoring** permission:

1. Build and run your app
2. macOS will prompt for Input Monitoring permission
3. Grant permission in: **System Settings → Privacy & Security → Input Monitoring**
4. **Restart the app** after granting permission

Without this permission, CGEventTap creation fails silently.

### ⚠️ Important: Launch Services Limitation (Release Builds)

**Dev mode works perfectly** (`pnpm tauri dev`) ✅  
**Release builds have a limitation** when launched via `open` command ❌

**The Issue:**

macOS Launch Services requires **Developer ID signing** for apps using CGEventTap with Input Monitoring. Without it:
- ✅ Direct binary execution works: `/YourApp.app/Contents/MacOS/binary`
- ❌ Launch Services fails: `open YourApp.app`

**Why:** macOS enforces stricter security for Input Monitoring when apps are launched through Launch Services vs direct execution. Read more: [macOS Privacy Permissions Guide](https://gannonlawlor.com/posts/macos_privacy_permissions/)

**Solutions:**

**For Local Development/Testing:**
```bash
# Option 1: Run binary directly
/Applications/YourApp.app/Contents/MacOS/your-binary

# Option 2: Create double-clickable launcher
echo '#!/bin/bash\n/Applications/YourApp.app/Contents/MacOS/your-binary' > Launch.command
chmod +x Launch.command
# Double-click Launch.command from Finder!
```

**For Production Distribution:**
```bash
# Requires Apple Developer account ($99/year)
codesign --deep --force --sign "Developer ID Application: Your Name (TEAM_ID)" \
  --options=runtime YourApp.app

# Then open works properly
```

**Note:** This is a macOS security feature, not a plugin bug. The CGEventTap implementation is production-ready and works flawlessly when launched correctly.

## API Reference

### Hotkey Struct

```rust
pub struct Hotkey {
    /// Keycodes to match (support multiple for same key in different modes)
    pub keycodes: Vec<i64>,
    /// Required modifier keys
    pub modifiers: Modifiers,
    /// Block event from reaching system (true = override system shortcuts)
    pub consume: bool,
    /// Tauri event name to emit when triggered
    pub event_name: String,
}
```

### Modifiers

```rust
pub struct Modifiers {
    pub command: bool,  // Cmd/⌘ key
    pub option: bool,   // Option/Alt key
    pub control: bool,  // Control key
    pub shift: bool,    // Shift key
}
```

**Helpers:**
```rust
Modifiers::empty()    // No modifiers
Modifiers::command()  // Cmd only
```

### Commands

```rust
// Register hotkey
let id = manager.register(hotkey)?;

// Unregister hotkey
manager.unregister(&id)?;

// Check if registered
let is_active = manager.is_registered(&id);
```

### Getting Keycode Table

```rust
use tauri_plugin_macos_input_monitor::get_function_key_codes;

// Get both keycodes for F5
let f5_codes = get_function_key_codes(5); // [96, 176]
```

## Critical Technical Details

### The NULL Return Secret

**Why Rust wrappers fail:**
```rust
// core-graphics crate wrapper
return None;  // Translates to Rust Option::None, NOT C NULL!
// Result: Event still dispatched to system ❌
```

**Why raw FFI works:**
```rust
extern "C" fn callback(...) -> CGEventRef {
    return std::ptr::null_mut();  // Actual C NULL pointer
}
// Result: Event consumed, system never sees it ✅
```

### macOS Function Key Keycodes

macOS has **TWO modes** for function keys (configured in System Settings):

**Standard Function Keys Mode:**
- F5 = keycode `96`

**Media Keys Mode (MacBook default):**
- F5 = keycode `176` (keyboard brightness down)

**Solution:** Register BOTH keycodes!

```rust
keycodes: vec![96, 176]  // Works in both modes
```

### Complete Keycode Reference

| Key | Standard Mode | Media Keys Mode | Media Function |
|-----|--------------|-----------------|----------------|
| F1  | 122          | 145             | Brightness Down |
| F2  | 120          | 144             | Brightness Up |
| F3  | 99           | 160             | Mission Control |
| F4  | 118          | 131             | Launchpad |
| F5  | 96           | 176             | KB Brightness Down |
| F6  | 97           | 177             | KB Brightness Up |
| F7  | 98           | 180             | Rewind |
| F8  | 100          | 179             | Play/Pause |
| F9  | 101          | 178             | Fast Forward |
| F10 | 109          | 173             | Mute |
| F11 | 103          | 174             | Volume Down |
| F12 | 111          | 175             | Volume Up |

### Flag Filtering Pattern

macOS includes **internal flags** that should be ignored when matching hotkeys:

```rust
// Internal macOS flags (IGNORE these):
const SECONDARY_FN_FLAG: u64 = 0x800000;  // Fn key indicator
const NON_COALESCED_FLAG: u64 = 0x100;    // Internal event flag
const CAPS_LOCK_FLAG: u64 = 0x10000;      // Caps Lock state
const NUM_PAD_FLAG: u64 = 0x200000;       // Numeric keypad

// User-intentional modifiers (CHECK these):
const CMD_FLAG: u64 = 0x100000;
const OPT_FLAG: u64 = 0x80000;
const CTRL_FLAG: u64 = 0x40000;
const SHIFT_FLAG: u64 = 0x20000;
```

**The plugin automatically strips internal flags** when matching, so you only specify intentional modifiers.

## How It Works

### CGEventTap Configuration

```rust
CGEventTapCreate(
    CG_HID_EVENT_TAP,              // Hardware/System-wide tap
    CG_HEAD_INSERT_EVENT_TAP,      // Highest priority placement
    CG_EVENT_TAP_OPTION_DEFAULT,   // Active filter (can modify)
    event_mask,                     // KeyDown events
    callback,                       // Our extern "C" callback
    null                            // No user info
)
```

**Key points:**
- `HID` location = hardware level (lowest in stack)
- `HeadInsertEventTap` = inserted at head of event queue (highest priority)
- `Default` option = active filter (can return NULL to consume)

### Event Consumption

When `consume: true`:
```rust
// In callback
if hotkey_matches {
    return std::ptr::null_mut();  // C NULL - event consumed!
}
```

macOS receives NULL and **stops event dispatch completely**. System never sees the keypress!

## Examples

### Override F5 Dictation

```rust
let hotkey = Hotkey {
    keycodes: vec![96, 176],  // F5 both modes
    modifiers: Modifiers::empty(),
    consume: true,  // Block dictation
    event_name: "f5-pressed".to_string(),
};
manager.register(hotkey)?;

// Listen for events
app.listen("f5-pressed", |event| {
    println!("F5 pressed! Dictation blocked.");
});
```

### Override F3 Mission Control

```rust
let hotkey = Hotkey {
    keycodes: vec![99, 160],  // F3 both modes
    modifiers: Modifiers::empty(),
    consume: true,
    event_name: "f3-pressed".to_string(),
};
```

### Cmd+F5 (with modifiers)

```rust
let hotkey = Hotkey {
    keycodes: vec![96, 176],
    modifiers: Modifiers::command(),
    consume: false,  // Don't block, just monitor
    event_name: "cmd-f5-pressed".to_string(),
};
```

## Troubleshooting

### F5 Still Opens Dictation

**Check Input Monitoring permission:**
1. System Settings → Privacy & Security → Input Monitoring
2. Ensure your app is listed and enabled
3. **Restart the app** after granting permission

**Check console logs:**
```
✅ CGEventTap created successfully  ← Should see this
🎯 Hotkey matched! keycode: 176    ← When pressing F5
🚫 Consuming event (returning NULL) ← Event blocked
```

If you don't see "CGEventTap created", permission is missing.

### Wrong Keycode for My Mac

Different Mac models use different keycodes! Use the discovery utility:

```rust
// TODO: Implement keycode discovery command
```

Or check console logs when pressing keys - keycode is logged for all events.

### Event Not Consumed

**Verify `consume: true`:**
```rust
consume: true,  // Must be true to block system
```

**Check you're returning NULL:**
- Plugin uses raw FFI that returns `std::ptr::null_mut()`
- This is THE critical difference from other solutions

## Comparison with tauri-plugin-global-shortcut

| Feature | global-shortcut | macos-input-monitor |
|---------|----------------|---------------------|
| **API Used** | RegisterEventHotKey | CGEventTap FFI |
| **Priority** | Application level | Hardware level |
| **Override System Shortcuts** | ❌ No | ✅ Yes |
| **Cross-platform** | ✅ Yes (Win/Mac/Linux) | ❌ macOS only |
| **Event Consumption** | Limited | ✅ Full (C NULL) |
| **Use Case** | App-level shortcuts | System override |

**When to use global-shortcut:**
- Cross-platform apps
- Regular app shortcuts (Cmd+S, Cmd+Q)
- Don't need to override system

**When to use macos-input-monitor:**
- Need to override macOS system shortcuts
- F5, F3, or other keys with system bindings
- macOS-specific apps only

## Technical Deep Dive

### The Journey to Success

This plugin is the result of extensive research and experimentation. Here's what we learned:

**Attempt 1:** Use `tauri-plugin-global-shortcut`
- ❌ Can't override system shortcuts
- Uses `RegisterEventHotKey` (app-level API)

**Attempt 2:** Use `core-graphics` Rust crate
- ❌ Event still reached system
- `Option<CGEvent>` return doesn't translate to C NULL

**Attempt 3:** Raw CGEventTap FFI ✅
- Returns actual `std::ptr::null_mut()`
- Direct C API calls
- **This works!**

### Send + Sync Safety

The plugin is designed to be thread-safe:

```rust
unsafe impl Send for EventTap {}
unsafe impl Sync for EventTap {}
```

**Why this is safe:**
- Actual `CFMachPortRef` lives in dedicated thread
- State protected by `Arc<Mutex<>>`
- Never share raw pointers across threads
- AppHandle is Send + Sync

### Flag Extraction Algorithm

```rust
pub fn extract_user_modifiers(raw_flags: u64) -> u64 {
    // Bitwise AND with user modifier mask
    raw_flags & (CMD_FLAG | OPT_FLAG | CTRL_FLAG | SHIFT_FLAG)
}
```

This strips `SecondaryFn`, `NonCoalesced`, `CapsLock` - internal macOS flags that appear in raw event data but aren't user-intentional modifiers.

## Contributing

Contributions welcome! This plugin solves a real problem for macOS Tauri developers.

**Areas for improvement:**
- Interactive keycode discovery tool
- TypeScript API bindings
- More comprehensive keycode database
- Swift Package integration
- Accessibility permission helpers

## License

MIT OR Apache-2.0

## Credits

Built by Yigit Konur based on:
- EventTapper Swift library architecture
- Official Tauri plugin development guide  
- Extensive CGEventTap API experimentation

**Special thanks to:**
- Tauri team for the plugin system
- EventTapper authors for Swift reference implementation
- macOS CoreGraphics documentation

## Related Resources

- [Tauri Plugin Development Guide](https://tauri.app/develop/plugins)
- [CGEventTap Apple Documentation](https://developer.apple.com/documentation/coregraphics/cgeventtap)
- [EventTapper Swift Library](https://github.com/usagimaru/EventTapper)
- [BetterTouchTool](https://folivora.ai) - Uses same technique
