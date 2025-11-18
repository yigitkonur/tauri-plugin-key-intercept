/**
 * macOS Virtual Key Code Constants
 * 
 * macOS has TWO modes for function keys:
 * - Standard Function Keys Mode: F5 = 96
 * - Media Keys Mode (MacBook default): F5 = 176
 * 
 * Use the array exports (e.g., F5) to support both modes!
 */

// Standard Function Key Mode Keycodes
export const F1_STANDARD = 122;
export const F2_STANDARD = 120;
export const F3_STANDARD = 99;
export const F4_STANDARD = 118;
export const F5_STANDARD = 96;
export const F6_STANDARD = 97;
export const F7_STANDARD = 98;
export const F8_STANDARD = 100;
export const F9_STANDARD = 101;
export const F10_STANDARD = 109;
export const F11_STANDARD = 103;
export const F12_STANDARD = 111;

// Media Keys Mode Keycodes (MacBook Default)
export const F1_MEDIA = 145;   // Brightness down
export const F2_MEDIA = 144;   // Brightness up
export const F3_MEDIA = 160;   // Mission Control/Exposé
export const F4_MEDIA = 131;   // Launchpad
export const F5_MEDIA = 176;   // Keyboard brightness down
export const F6_MEDIA = 177;   // Keyboard brightness up
export const F7_MEDIA = 180;   // Rewind
export const F8_MEDIA = 179;   // Play/Pause
export const F9_MEDIA = 178;   // Fast forward
export const F10_MEDIA = 173;  // Mute
export const F11_MEDIA = 174;  // Volume down
export const F12_MEDIA = 175;  // Volume up

// Convenience Arrays (Recommended - works in both modes!)
export const F1 = [F1_STANDARD, F1_MEDIA];
export const F2 = [F2_STANDARD, F2_MEDIA];
export const F3 = [F3_STANDARD, F3_MEDIA];
export const F4 = [F4_STANDARD, F4_MEDIA];
export const F5 = [F5_STANDARD, F5_MEDIA];
export const F6 = [F6_STANDARD, F6_MEDIA];
export const F7 = [F7_STANDARD, F7_MEDIA];
export const F8 = [F8_STANDARD, F8_MEDIA];
export const F9 = [F9_STANDARD, F9_MEDIA];
export const F10 = [F10_STANDARD, F10_MEDIA];
export const F11 = [F11_STANDARD, F11_MEDIA];
export const F12 = [F12_STANDARD, F12_MEDIA];

/**
 * Helper to get keycodes for a function key by number
 * @param keyNumber - Function key number (1-12)
 * @returns Array of keycodes for that key (standard + media mode)
 */
export function getFunctionKey(keyNumber: number): number[] {
  const keys: Record<number, number[]> = {
    1: F1, 2: F2, 3: F3, 4: F4,
    5: F5, 6: F6, 7: F7, 8: F8,
    9: F9, 10: F10, 11: F11, 12: F12
  };
  return keys[keyNumber] || [];
}

