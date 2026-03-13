#[tauri::command]
pub fn get_current_branch(path: String) -> Result<Option<String>, String> {
    let repo = git2::Repository::discover(&path).map_err(|e| e.to_string())?;
    let head = repo.head().map_err(|e| e.to_string())?;
    Ok(head.shorthand().map(|s| s.to_string()))
}
