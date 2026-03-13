mod commands;
mod config;
mod git;
mod pty;
mod usage;

use std::sync::{Arc, Mutex};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Arc::new(Mutex::new(pty::PtyManager::new())))
        .invoke_handler(tauri::generate_handler![
            commands::create_pty,
            commands::write_pty,
            commands::resize_pty,
            commands::kill_pty,
            commands::spawn_batch,
            config::save_app_state,
            config::load_app_state,
            config::save_scrollback,
            config::load_scrollback,
            config::save_settings,
            config::load_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
