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
        .plugin(tauri_plugin_notification::init())
        .manage(Arc::new(Mutex::new(pty::PtyManager::new())))
        .invoke_handler(tauri::generate_handler![
            commands::create_pty,
            commands::write_pty,
            commands::resize_pty,
            commands::kill_pty,
            commands::spawn_batch,
            commands::get_default_shell,
            config::save_app_state,
            config::load_app_state,
            config::save_scrollback,
            config::load_scrollback,
            config::save_settings,
            config::load_settings,
            git::get_current_branch,
            git::get_branches,
            git::switch_branch,
            git::get_git_status,
            git::get_file_diff,
            git::stage_file,
            git::unstage_file,
            git::revert_file,
            usage::get_usage_data,
            commands::get_system_info,
            commands::check_cli_exists,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
