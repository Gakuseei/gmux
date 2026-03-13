use std::fs;
use std::path::PathBuf;

const UUID_PATTERN: &str = r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$";

fn validate_terminal_id(terminal_id: &str) -> Result<(), String> {
    let re = regex::Regex::new(UUID_PATTERN).map_err(|e| e.to_string())?;
    if !re.is_match(terminal_id) {
        return Err("Invalid terminal ID format".to_string());
    }
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
    fs::write(&path, data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_app_state() -> Result<Option<String>, String> {
    let path = get_config_dir().join("state.json");
    if !path.exists() {
        return Ok(None);
    }
    fs::read_to_string(&path)
        .map(Some)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_scrollback(terminal_id: String, content: String) -> Result<(), String> {
    validate_terminal_id(&terminal_id)?;
    let path = get_scrollback_dir().join(format!("{}.txt", terminal_id));
    fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_scrollback(terminal_id: String) -> Result<Option<String>, String> {
    validate_terminal_id(&terminal_id)?;
    let path = get_scrollback_dir().join(format!("{}.txt", terminal_id));
    if !path.exists() {
        return Ok(None);
    }
    fs::read_to_string(&path)
        .map(Some)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_settings(data: String) -> Result<(), String> {
    let path = get_config_dir().join("settings.json");
    fs::write(&path, data).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_settings() -> Result<Option<String>, String> {
    let path = get_config_dir().join("settings.json");
    if !path.exists() {
        return Ok(None);
    }
    fs::read_to_string(&path)
        .map(Some)
        .map_err(|e| e.to_string())
}
