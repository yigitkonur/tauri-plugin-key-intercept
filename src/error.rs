use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to create CGEventTap. Ensure Input Monitoring permission is granted in System Settings → Privacy & Security → Input Monitoring")]
    EventTapCreationFailed,

    #[error("Hotkey not found")]
    HotkeyNotFound,

    #[error("Failed to acquire lock on event tap state")]
    LockError,

    #[error("This plugin only works on macOS")]
    UnsupportedPlatform,

    #[error("Invalid keycode: {0}")]
    InvalidKeycode(i64),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Tauri(#[from] tauri::Error),

    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
