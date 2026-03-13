use std::fs;
use std::path::{Path, PathBuf};

fn atomic_write(path: &Path, data: &[u8]) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
    fs::write(&tmp, data).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())
}

fn validate_terminal_id(terminal_id: &str) -> Result<(), String> {
    uuid::Uuid::parse_str(terminal_id).map_err(|_| "Invalid terminal ID format".to_string())?;
    Ok(())
}

fn get_config_dir() -> PathBuf {
    let mut dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    dir.push("gmux");
    fs::create_dir_all(&dir).ok();
    dir
}

fn get_scrollback_dir() -> PathBuf {
    let dir = get_config_dir().join("scrollback");
    fs::create_dir_all(&dir).ok();
    dir
}

#[tauri::command]
pub fn save_app_state(data: String) -> Result<(), String> {
    let path = get_config_dir().join("state.json");
    atomic_write(&path, data.as_bytes())
}

#[tauri::command]
pub fn load_app_state() -> Result<Option<String>, String> {
    let path = get_config_dir().join("state.json");
    match fs::read_to_string(&path) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn save_scrollback(terminal_id: String, content: String) -> Result<(), String> {
    validate_terminal_id(&terminal_id)?;
    let path = get_scrollback_dir().join(format!("{}.txt", terminal_id));
    atomic_write(&path, content.as_bytes())
}

#[tauri::command]
pub fn load_scrollback(terminal_id: String) -> Result<Option<String>, String> {
    validate_terminal_id(&terminal_id)?;
    let path = get_scrollback_dir().join(format!("{}.txt", terminal_id));
    match fs::read_to_string(&path) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn save_settings(data: String) -> Result<(), String> {
    let path = get_config_dir().join("settings.json");
    atomic_write(&path, data.as_bytes())
}

#[tauri::command]
pub fn load_settings() -> Result<Option<String>, String> {
    let path = get_config_dir().join("settings.json");
    match fs::read_to_string(&path) {
        Ok(s) => Ok(Some(s)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}
