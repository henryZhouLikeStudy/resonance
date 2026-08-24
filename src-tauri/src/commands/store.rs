use serde_json::Value;
use std::path::PathBuf;
use tauri::AppHandle;
use tauri_plugin_store::StoreExt;

use super::fs_secure::restrict_file;

pub(crate) const STORE_FILE: &str = "resonance-store.json";

/// Absolute path of the on-disk store file (settings, history, cookie jar,
/// workspace tabs, ...).
///
/// Portable mode: `<exe-dir>/data/resonance-store.json`, so the data travels
/// with the executable. Installed mode: the Tauri app data dir. The directory
/// is created on demand so the store plugin can write into it immediately.
pub(crate) fn store_file_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(crate::paths::ensure_app_data_dir(app)?.join(STORE_FILE))
}

/// Best-effort restriction of the on-disk store file to owner-only access.
/// The store plugin writes it with the process umask (typically world-readable),
/// so it holds request history, the cookie jar, and any plaintext-fallback
/// secrets — tighten it after every save.
pub(crate) fn restrict_store_file(app: &AppHandle) {
    if let Ok(path) = store_file_path(app) {
        restrict_file(&path);
    }
}

fn get_default_for_key(key: &str) -> Value {
    match key {
        "collections" => serde_json::json!([]),
        "environments" => serde_json::json!([]),
        "activeEnvironmentId" => Value::Null,
        "requestHistory" => serde_json::json!([]),
        "cookieJar" => serde_json::json!([]),
        "workspaceTabs" => serde_json::json!([]),
        "activeWorkspaceTabId" => Value::Null,
        "theme" => serde_json::json!("system"),
        "accentColor" => serde_json::json!("blue"),
        "proxySettings" => {
            serde_json::to_value(super::proxy::ProxySettings::default()).unwrap_or(Value::Null)
        }
        "mockServerSettings" => serde_json::json!({
            "port": 3001,
            "delay": 0,
            "enabled": false
        }),
        "clientCertificates" => serde_json::json!({ "items": [] }),
        "secretValues" => serde_json::json!({}),
        "secretIndex" => serde_json::json!({}),
        "settings" => serde_json::json!({
            "httpVersion": "auto",
            "timeout": 30000,
            "theme": "system",
            "accentColor": "blue",
            "language": "zh-CN"
        }),
        _ if key.ends_with("Scripts") => serde_json::json!({}),
        _ if key.ends_with("Variables") => serde_json::json!([]),
        _ => Value::Null,
    }
}

#[tauri::command]
pub async fn store_get(app: AppHandle, key: String) -> Result<Value, String> {
    let store = app.store(store_file_path(&app)?).map_err(|e| e.to_string())?;

    let value = store.get(&key).unwrap_or(Value::Null);

    if value.is_null() {
        Ok(get_default_for_key(&key))
    } else {
        Ok(value)
    }
}

#[tauri::command]
pub async fn store_set(app: AppHandle, key: String, value: Value) -> Result<(), String> {
    let store = app.store(store_file_path(&app)?).map_err(|e| e.to_string())?;

    store.set(key, value);
    store.save().map_err(|e| e.to_string())?;
    restrict_store_file(&app);

    Ok(())
}

#[tauri::command]
pub async fn settings_get(app: AppHandle) -> Result<Value, String> {
    let result = store_get(app, "settings".to_string()).await?;

    if result.is_null() {
        Ok(serde_json::json!({
            "httpVersion": "auto",
            "timeout": 30000,
            "theme": "system",
            "accentColor": "blue",
            "language": "zh-CN"
        }))
    } else {
        Ok(result)
    }
}

#[tauri::command]
pub async fn settings_set(app: AppHandle, settings: Value) -> Result<(), String> {
    store_set(app, "settings".to_string(), settings).await
}
