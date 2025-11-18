/**
 * tauri-plugin-macos-input-monitor TypeScript API
 * 
 * macOS-only plugin for intercepting keyboard events at hardware level
 * to override system shortcuts like F5 dictation, F3 mission control, etc.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

/**
 * Modifier key configuration
 */
export interface Modifiers {
  command?: boolean;
  option?: boolean;
  control?: boolean;
  shift?: boolean;
}

/**
 * Hotkey configuration
 */
export interface HotkeyConfig {
  /** Keycodes to match (e.g., [96, 176] for F5 in both modes) */
  keycodes: number[];
  /** Required modifier keys */
  modifiers?: Modifiers;
  /** Block event from reaching system (true = override system shortcuts) */
  consume?: boolean;
}

/**
 * Keypress event data
 */
export interface KeypressEvent {
  keycode: number;
  rawFlags: number;
  userModifiers: number;
}

/**
 * Hotkey class for easy hotkey management
 */
export class Hotkey {
  private id?: string;
  private eventName: string;
  
  constructor(
    private config: HotkeyConfig,
    eventName: string
  ) {
    this.eventName = eventName;
  }
  
  /**
   * Register this hotkey
   * @returns Promise<string> - Hotkey ID for later unregistration
   */
  async register(): Promise<string> {
    this.id = await invoke<string>('plugin:macos-input-monitor|register', {
      keycodes: this.config.keycodes,
      modifiers: this.config.modifiers || {},
      consume: this.config.consume !== false, // default true
      eventName: this.eventName
    });
    return this.id;
  }
  
  /**
   * Unregister this hotkey
   */
  async unregister(): Promise<void> {
    if (this.id) {
      await invoke('plugin:macos-input-monitor|unregister', { id: this.id });
      this.id = undefined;
    }
  }
  
  /**
   * Listen for this hotkey being triggered
   * @param handler - Callback function
   * @returns Promise<UnlistenFn> - Function to unlisten
   */
  onTriggered(handler: (event: KeypressEvent) => void): Promise<UnlistenFn> {
    return listen<KeypressEvent>(this.eventName, (event) => {
      handler(event.payload);
    });
  }
  
  /**
   * Check if this hotkey is currently registered
   */
  async isRegistered(): Promise<boolean> {
    if (!this.id) return false;
    return await invoke<boolean>('plugin:macos-input-monitor|is_registered', {
      id: this.id
    });
  }
}

/**
 * Get keycode table for all function keys (F1-F12)
 * Returns both standard and media mode keycodes
 */
export async function getKeycodeTable(): Promise<Record<string, number[]>> {
  return await invoke<Record<string, number[]>>('plugin:macos-input-monitor|get_keycode_table');
}

/**
 * Open Input Monitoring settings in System Preferences
 * Helpful for guiding users to grant permission
 */
export async function openInputMonitoringSettings(): Promise<void> {
  await invoke('plugin:macos-input-monitor|open_input_monitoring_settings');
}

/**
 * Check if Input Monitoring permission is granted
 * @returns Promise<boolean> - true if permission granted
 */
export async function checkPermission(): Promise<boolean> {
  return await invoke<boolean>('plugin:macos-input-monitor|check_permission');
}

/**
 * Discover keycodes for keys you press
 * Useful for finding keycodes on different Mac models
 */
export async function discoverKeycode(durationMs?: number): Promise<string> {
  return await invoke<string>('plugin:macos-input-monitor|discover_keycode', {
    durationMs: durationMs || 30000
  });
}

// Re-export keycode constants
export * from './keycodes';

