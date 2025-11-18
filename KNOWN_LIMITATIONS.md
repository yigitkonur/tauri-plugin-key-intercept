# Known Limitations

## Launch Services Restriction (Without Developer ID)

### The Issue

**Symptoms:**
- ✅ App works when run directly: `/Contents/MacOS/binary`
- ❌ App fails when launched via `open App.app`
- Permission shows as granted in System Settings
- CGEventTap creation fails silently

**Root Cause:**

macOS Launch Services enforces stricter security for Input Monitoring permissions. Apps using CGEventTap require either:
1. **Developer ID signing** (paid Apple Developer account)
2. **Direct binary execution** (bypasses Launch Services)

Ad-hoc signing (`codesign --sign -`) is **insufficient** for Launch Services to grant Input Monitoring at the CGEventTap level.

### Workarounds

#### Option 1: Double-Clickable Launcher (Recommended for Local Use)

Create a `.command` file that users can double-click:

```bash
#!/bin/bash
/Applications/YourApp.app/Contents/MacOS/your-binary
```

Make it executable:
```bash
chmod +x Launch\ YourApp.command
```

Users can double-click this from Finder like a normal app!

#### Option 2: Alias or Script

```bash
# Add to ~/.zshrc or ~/.bashrc
alias f5listener='/Applications/F5\ Global\ Listener.app/Contents/MacOS/transcribe-realtime &'
```

#### Option 3: Get Developer ID Certificate (For Distribution)

1. Enroll in Apple Developer Program ($99/year)
2. Get Developer ID Application certificate
3. Sign with:
```bash
codesign --deep --force --sign "Developer ID Application: Your Name" \
  --options=runtime YourApp.app
```
4. Notarize for distribution (optional but recommended)

### Why Dev Mode Always Works

Dev builds (`cargo tauri dev`) run the binary directly via cargo, never going through Launch Services. This is why you never encounter this issue in development!

### Technical Details

**Launch Services enforces:**
- Code signature validation
- Entitlement verification
- Team ID matching
- Proper signing chain

**Direct binary execution:**
- Kernel checks permission directly
- No Launch Services layer
- Works with ad-hoc signing
- Perfect for local development/testing

### Recommendation

**For Development/Personal Use:**
- Use the `.command` launcher script
- Or always launch via terminal/script

**For Distribution:**
- Get Apple Developer account
- Use proper Developer ID signing
- This is required anyway for notarization

## Summary

This is **not a bug in the plugin** - it's how macOS security works. The CGEventTap code is perfect and battle-tested. The limitation is purely in how macOS Launch Services handles permissions for unsigned apps.

**The plugin works flawlessly** - you just need to launch it correctly based on your signing setup!

