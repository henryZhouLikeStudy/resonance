//! Path resolution for application data.
//!
//! Resonance supports a portable mode: when a `PORTABLE` marker file sits next
//! to the running executable, every byte of application data is stored in a
//! `data` directory beside the executable so the whole application travels
//! with the folder. Installed mode keeps using the platform app data directory
//! provided by Tauri.
//!
//! All backend code that needs the data directory must go through
//! [`app_data_dir`] / [`ensure_app_data_dir`] (and [`store_file_path`] for the
//! on-disk store) instead of calling `app.path().app_data_dir()` directly, so
//! the portable/installed decision stays in exactly one place.

use std::path::PathBuf;

use tauri::{AppHandle, Manager};

/// Marker file that switches Resonance into portable mode when it sits next to
/// the executable. The release pipeline ships this file inside the portable
/// ZIP, next to `Resonance.exe`.
pub const PORTABLE_MARKER_FILE: &str = "PORTABLE";

/// Name of the directory (beside the executable) that holds all application
/// data in portable mode.
pub const PORTABLE_DATA_DIR: &str = "data";

/// Whether this run is in portable mode, i.e. a [`PORTABLE_MARKER_FILE`] marker
/// file sits next to the running executable.
pub fn is_portable() -> bool {
    portable_data_dir().is_some()
}

/// The portable data directory (`<exe-dir>/data`), or `None` when no
/// [`PORTABLE_MARKER_FILE`] marker sits next to the running executable.
pub fn portable_data_dir() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    if exe_dir.join(PORTABLE_MARKER_FILE).is_file() {
        Some(exe_dir.join(PORTABLE_DATA_DIR))
    } else {
        None
    }
}

/// The single source of truth for where application data lives.
///
/// - Portable mode: `<exe-dir>/data`, so data travels with the executable.
/// - Installed mode: the platform app data dir provided by Tauri.
pub fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    if let Some(dir) = portable_data_dir() {
        return Ok(dir);
    }
    app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {e}"))
}

/// Like [`app_data_dir`], but also creates the directory and hardens its
/// permissions (owner-only on Unix) so callers can write into it immediately.
pub fn ensure_app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app_data_dir(app)?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create app data dir {}: {e}", dir.display()))?;
    crate::commands::fs_secure::restrict_dir(&dir);
    Ok(dir)
}
