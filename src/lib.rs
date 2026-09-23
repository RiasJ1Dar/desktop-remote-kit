//! Kit for self-contained desktop apps: Windows autostart, optional GitHub
//! update check, optional Telegram remote, optional system tray.
//!
//! Enable features: `update` (default), `telegram`, `tray`, or `full`.

#![deny(missing_docs)]

pub mod autostart;

#[cfg(feature = "update")]
pub mod update;

#[cfg(feature = "telegram")]
pub mod telegram;

#[cfg(feature = "tray")]
pub mod tray;

/// Identity of the host application (used for Run-key name, User-Agent, etc.).
#[derive(Debug, Clone)]
pub struct AppId {
    /// Short id, e.g. `TwitchDropFarm` — used as Windows Run value name.
    pub id: String,
    /// Human-readable name for menus and User-Agent.
    pub name: String,
    /// Semver or release tag without `v`, e.g. `1.2.0`.
    pub version: String,
}

impl AppId {
    /// Create a new application identity.
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
        }
    }
}
