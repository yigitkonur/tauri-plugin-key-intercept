Tauri v2 plugin that intercepts global keyboard events at the hardware level on macOS. uses `CGEventTap` FFI to steal keypresses before the OS sees them — so you can override system shortcuts like F5 (dictation) without asking nicely.

```rust
app.plugin(tauri_plugin_macos_input_monitor::init())
```

[![crates.io](https://img.shields.io/crates/v/tauri-plugin-key-intercept.svg?style=flat-square)](https://crates.io/crates/tauri-plugin-key-intercept)
[![rust](https://img.shields.io/badge/rust-1.77.2+-93450a.svg?style=flat-square)](https://www.rust-lang.org/)
[![license](https://img.shields.io/badge/license-MIT_|_Apache--2.0-grey.svg?style=flat-square)](https://opensource.org/licenses/MIT)

---

## how it works

inserts a `CG_HEAD_INSERT_EVENT_TAP` at `CG_HID_EVENT_TAP` position — highest priority in the macOS event chain, before any other listener. when a registered hotkey fires, the C callback returns `NULL` to consume the event (OS never sees it) and emits a Tauri event to your frontend with the keycode and modifier flags.

same technique used by BetterTouchTool and similar tools. runs on a dedicated thread with its own `CFRunLoop`, fully idle at 0.0% CPU between events.

## what it does

- **hardware-level event tap** — `CGEventTap` FFI, not `NSEvent` global monitors
- **consume or passthrough** — per-hotkey choice to block the OS or let the event propagate
- **both keycode modes** — register standard and media keycodes simultaneously (e.g. F5 = `[96, 176]`)
- **modifier filtering** — strips internal macOS flags (fn, caps lock, numpad), matches only intentional modifiers (cmd, opt, ctrl, shift)
- **runtime permission check** — probes `CGEventTapCreate` to verify Input Monitoring access without side effects
- **deep-link to settings** — opens System Settings directly at Privacy > Input Monitoring
- **cross-platform safe** — compiles on all platforms, returns `UnsupportedPlatform` on non-macOS

## install

### Rust side

add to your `src-tauri/Cargo.toml`:

```toml
[dependencies]
tauri-plugin-key-intercept = "0.1"
```

register the plugin:

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_macos_input_monitor::init())
    .run(tauri::generate_context!())
    .expect("failed to run app");
```

### frontend side

```bash
npm install @yigitkonur/plugin-macos-input-monitor
```

### capabilities

add to `src-tauri/capabilities/default.json`:

```json
{
  "permissions": [
    "macos-input-monitor:default",
    "macos-input-monitor:allow-check-permission",
    "macos-input-monitor:allow-open-input-monitoring-settings"
  ]
}
```

the default permission set grants `register`, `unregister`, `is-registered`, and `get-keycode-table`. permission check and settings opener must be added explicitly.

## usage

### from TypeScript

```typescript
import { Hotkey, F5, checkPermission, openInputMonitoringSettings } from '@yigitkonur/plugin-macos-input-monitor';

// check Input Monitoring permission
const hasAccess = await checkPermission();
if (!hasAccess) {
  await openInputMonitoringSettings();
  // user must restart the app after granting
}

// register a hotkey
const hotkey = new Hotkey({ keycodes: F5, consume: true }, 'f5-pressed');
const id = await hotkey.register();

// listen for it
const unlisten = await hotkey.onTriggered((event) => {
  console.log('keycode:', event.keycode, 'modifiers:', event.userModifiers);
});

// cleanup
await hotkey.unregister();
unlisten();
```

### from Rust (host app setup)

```rust
use tauri_plugin_macos_input_monitor::{MacOSInputMonitorExt, Modifiers, Hotkey};

let hotkey = Hotkey {
    keycodes: vec![96, 176],  // F5 standard + media
    modifiers: Modifiers::empty(),
    consume: true,
    event_name: "f5-pressed".to_string(),
};

app.macos_input_monitor()
    .manager.lock().unwrap()
    .register(hotkey)?;
```

### keycode table

```typescript
import { getKeycodeTable } from '@yigitkonur/plugin-macos-input-monitor';

const table = await getKeycodeTable();
// { "F1": [122, 145], "F2": [120, 144], ..., "F12": [111, 175] }
```

each key returns `[standard_keycode, media_keycode]`. pass both when registering to catch the key regardless of the "use F1-F12 as standard function keys" toggle in System Settings.

## IPC commands

| command | description |
|:---|:---|
| `register` | register a global hotkey. returns a UUID |
| `unregister` | remove a hotkey by its UUID |
| `is_registered` | check if a hotkey ID is active |
| `get_keycode_table` | returns F1-F12 keycode map (standard + media) |
| `check_permission` | probes Input Monitoring access at runtime |
| `open_input_monitoring_settings` | deep-links to System Settings > Privacy > Input Monitoring |
| `discover_keycode` | developer utility (currently a stub — logs instructions) |

## event payload

when a registered hotkey fires, the Tauri event carries:

```typescript
interface KeypressEvent {
  keycode: number;      // the virtual keycode that matched
  rawFlags: number;     // full macOS modifier bitmask
  userModifiers: number; // cleaned — only cmd/opt/ctrl/shift
}
```

## permissions

requires **Input Monitoring** in System Settings > Privacy & Security. without it, `CGEventTapCreate` returns `NULL` and the plugin silently does nothing. the app must be restarted after granting — macOS TCC changes don't take effect in a running process.

for unsigned/ad-hoc builds: the binary works when launched from terminal but may fail when double-clicked (Launch Services restriction). a paid Apple Developer ID certificate resolves this.

## project structure

```
src/
  lib.rs          — plugin entry point, init(), state, extension trait
  commands.rs     — 7 Tauri command handlers
  models.rs       — IPC types (Hotkey, Modifiers, KeypressEvent, etc.)
  event_tap.rs    — CGEventTap FFI, dedicated thread, C callback
  manager.rs      — thread-safe HotkeyManager wrapper
  constants.rs    — keycodes, modifier flags, CoreGraphics constants
  error.rs        — error types with thiserror
guest-js/
  index.ts        — TypeScript API (Hotkey class, standalone functions)
  keycodes.ts     — F1-F12 keycode constants
```

## running the example

```bash
cd examples/vanilla
pnpm install
pnpm tauri dev
```

demo app registers F5 as a global hotkey, shows a counter and event log. demonstrates permission checking and the settings deep-link.

## license

MIT / Apache-2.0
