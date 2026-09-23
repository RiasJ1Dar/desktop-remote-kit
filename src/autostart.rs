//! Launch the app when the user signs in.
//!
//! **Windows:** `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
//! **Linux / macOS:** not implemented here — use a `.desktop` autostart file
//! or LaunchAgent (see host app docs). This module returns [`Error::Unsupported`].

use crate::AppId;
use std::path::Path;
use thiserror::Error;

/// Autostart errors.
#[derive(Debug, Error)]
pub enum Error {
    /// This platform has no built-in implementation in the kit.
    #[error("autostart is not implemented on this platform; configure it in the host app")]
    Unsupported,
    /// Missing or invalid executable path.
    #[error("executable path is invalid")]
    InvalidPath,
    /// OS or registry call failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Whether the app is registered to start at sign-in.
pub fn is_enabled(app: &AppId) -> Result<bool, Error> {
    #[cfg(windows)]
    {
        windows::is_enabled(&app.id)
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        Err(Error::Unsupported)
    }
}

/// Register `exe` to start at sign-in (current user).
///
/// `exe` should be the absolute path to the host binary. Optional `args` are
/// appended as a single command line on Windows.
pub fn enable(app: &AppId, exe: &Path, args: &str) -> Result<(), Error> {
    if !exe.is_absolute() {
        return Err(Error::InvalidPath);
    }
    #[cfg(windows)]
    {
        windows::enable(&app.id, exe, args)
    }
    #[cfg(not(windows))]
    {
        let _ = (app, args);
        Err(Error::Unsupported)
    }
}

/// Remove the sign-in registration.
pub fn disable(app: &AppId) -> Result<(), Error> {
    #[cfg(windows)]
    {
        windows::disable(&app.id)
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        Err(Error::Unsupported)
    }
}

#[cfg(windows)]
mod windows {
    use super::Error;
    use std::path::Path;
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

    fn open_run(write: bool) -> std::io::Result<RegKey> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if write {
            Ok(hkcu.create_subkey(RUN_KEY)?.0)
        } else {
            Ok(hkcu.open_subkey(RUN_KEY)?)
        }
    }

    pub fn is_enabled(id: &str) -> Result<bool, Error> {
        let key = open_run(false)?;
        match key.get_value::<String, _>(id) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    pub fn enable(id: &str, exe: &Path, args: &str) -> Result<(), Error> {
        let key = open_run(true)?;
        let exe_s = exe.to_string_lossy();
        let value = if args.trim().is_empty() {
            format!("\"{exe_s}\"")
        } else {
            format!("\"{exe_s}\" {args}")
        };
        key.set_value(id, &value)?;
        Ok(())
    }

    pub fn disable(id: &str) -> Result<(), Error> {
        let key = open_run(true)?;
        match key.delete_value(id) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
