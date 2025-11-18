# Release Build Guide

## Dev Mode vs Release Mode Differences

### Why Release Build Behaves Differently

**macOS treats dev and release as DIFFERENT apps:**

| Aspect | Dev Mode | Release Mode |
|--------|----------|--------------|
| **Binary Location** | `target/debug/transcribe-realtime` | `target/release/transcribe-realtime` |
| **Code Signing** | None | Ad-hoc signed |
| **Bundle ID** | Same | Same but different binary |
| **Permission Scope** | Per-binary | Per-binary |
| **Console Output** | Terminal | Console.app only |

### Critical: Permission is Per-Binary!

**The Problem:**
- You granted Input Monitoring to the **dev build**
- The **release build** is a different binary
- macOS requires separate permission grant!

**The Solution:**
1. Launch release app: `F5 Global Listener.app`
2. macOS will prompt for Input Monitoring (again!)
3. Grant permission
4. **Restart the release app**
5. Now it works!

## Testing Release Build

### Step 1: Fresh Install

```bash
# Remove any old version
rm -rf /Applications/F5\ Global\ Listener.app

# Copy fresh release build
cp -r src-tauri/target/release/bundle/macos/F5\ Global\ Listener.app /Applications/

# Launch
open /Applications/F5\ Global\ Listener.app
```

### Step 2: Check UI Indicators

The new enhanced UI shows:

**Permission Status:**
- 🟢 Green = Input Monitoring granted
- 🔴 Red = Not granted (click "Open System Settings")

**Plugin Status:**
- 🟢 Green = Ready to intercept
- 🟡 Yellow = Waiting for permission
- 🔴 Red = Failed to initialize

### Step 3: Verify Functionality

**When working correctly:**
- Press F5 → Counter increments
- Event log shows: "F5 pressed! Count: X"
- Green "✅ Working Correctly!" box appears
- **Dictation does NOT appear**

**If not working:**
- Click "Test Permission" button
- If red, click "Open System Settings"
- Grant permission to "F5 Global Listener"
- **Restart the app** (important!)

## Common Issues

### Issue 1: Counter Doesn't Increment

**Symptom:** UI shows 0, never increases

**Cause:** Input Monitoring permission not granted to release build

**Fix:**
1. Check permission indicator (should be green)
2. If red, grant permission
3. **Must restart app** after granting

### Issue 2: Can't See Console Logs

**Symptom:** No `println!` output in terminal

**Cause:** Release builds log to macOS Console.app, not terminal

**Fix:**
```bash
# View logs in real-time
log stream --predicate 'process == "transcribe-realtime"' --level debug

# Or open Console.app and filter for "transcribe-realtime"
```

### Issue 3: Permission Already Granted But Still Doesn't Work

**Symptom:** System Settings shows permission granted, but app doesn't work

**Possible causes:**
1. Permission granted to **old binary** (rebuild creates new one)
2. App not restarted after permission grant
3. Wrong app in System Settings list

**Fix:**
1. Remove ALL "F5 Global Listener" entries from System Settings
2. Restart Mac (clears permission cache)
3. Launch app fresh
4. Grant permission when prompted
5. Restart app

## Debugging Release Builds

### Enable Debug Logging

The plugin uses `#[cfg(debug_assertions)]` for verbose logs. To see them in release:

**Temporarily disable the flag in event_tap.rs:**
```rust
// Change from:
#[cfg(debug_assertions)]
println!("🎯 Hotkey matched!");

// To:
println!("🎯 Hotkey matched!");  // Always log
```

Then rebuild: `pnpm tauri build`

### Check macOS Console

```bash
# Stream logs in real-time
log stream --predicate 'subsystem == "com.apple.CoreGraphics"' --level debug

# Look for CGEventTap messages
```

### Verify CGEventTap Created

Look for these in Console.app or log stream:
```
✅ CGEventTap created successfully
✅ Event tap ready - listening for hotkeys
```

If you see:
```
❌ Failed to create CGEventTap
```

Then permission is definitely missing.

## Production Deployment

For distribution to users:

### Option 1: Proper Code Signing (Recommended)

```bash
# Set signing identity
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"

# Build with signing
pnpm tauri build

# Notarize (for distribution)
xcrun notarytool submit path/to/app.dmg \
  --apple-id "your@email.com" \
  --team-id "TEAM_ID" \
  --password "app-specific-password" \
  --wait
```

### Option 2: Ad-hoc Signing (Local Use Only)

Current builds use ad-hoc signing. This is fine for:
- Personal use
- Internal distribution
- Testing

NOT suitable for:
- App Store
- Public distribution
- Enterprise deployment

## Quick Checklist

Before saying "it doesn't work":

- [ ] Launched the **release build** (not dev)
- [ ] Checked permission indicator in UI (green = granted)
- [ ] Granted Input Monitoring to **this specific app**
- [ ] **Restarted app** after granting permission
- [ ] Pressed F5 and watched UI counter
- [ ] Checked event log in the UI
- [ ] If still failing, checked macOS Console.app logs

If all above are true and it still doesn't work, the issue is likely:
- macOS permission cache (restart Mac)
- Multiple app versions confusion (clean install)
- System integrity issue (rare)

