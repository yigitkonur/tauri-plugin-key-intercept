//! Shared data models for plugin IPC

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for registered hotkeys
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct HotkeyId(pub String);

impl HotkeyId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl fmt::Display for HotkeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Modifier key configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Modifiers {
    #[serde(default)]
    pub command: bool,
    #[serde(default)]
    pub option: bool,
    #[serde(default)]
    pub control: bool,
    #[serde(default)]
    pub shift: bool,
}

impl Default for Modifiers {
    fn default() -> Self {
        Self::empty()
    }
}

impl Modifiers {
    pub fn empty() -> Self {
        Self {
            command: false,
            option: false,
            control: false,
            shift: false,
        }
    }

    pub fn command() -> Self {
        Self {
            command: true,
            ..Self::empty()
        }
    }
}

/// Hotkey definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hotkey {
    /// One or more keycodes to match (e.g., [96, 176] for F5 in both modes)
    pub keycodes: Vec<i64>,
    /// Required modifier keys
    pub modifiers: Modifiers,
    /// Whether to consume the event (block system from seeing it)
    pub consume: bool,
    /// Event name to emit when hotkey is triggered
    pub event_name: String,
}

impl Hotkey {
    /// Convert modifiers to flag bitmask
    pub fn get_modifier_flags(&self) -> u64 {
        use crate::constants::*;
        let mut flags = 0u64;
        if self.modifiers.command {
            flags |= CMD_FLAG;
        }
        if self.modifiers.option {
            flags |= OPT_FLAG;
        }
        if self.modifiers.control {
            flags |= CTRL_FLAG;
        }
        if self.modifiers.shift {
            flags |= SHIFT_FLAG;
        }
        flags
    }
}

/// Event data sent when a hotkey is triggered
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeypressEvent {
    pub keycode: i64,
    pub raw_flags: u64,
    pub user_modifiers: u64,
}

/// Keycode discovery event (for developer utility)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeycodeDiscovery {
    pub keycode: i64,
    pub raw_flags: u64,
    pub user_modifiers: u64,
    pub key_name: Option<String>,
}
