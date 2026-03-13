use crate::pty::PtyManager;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;
use tauri::State;

const SHELL_METACHARACTERS: &[char] = &[
    '|', '&', ';', '$', '`', '(', ')', '{', '}', '[', ']', '<', '>', '!', '#', '*', '?', '~',
    '"', '\'', '\\', '\n', '\r',
];

fn validate_shell_path(shell: &str) -> Result<(), String> {
    if !shell.starts_with('/') {
        return Err("Shell path must be absolute".to_string());
    }
    if !Path::new(shell).exists() {
        return Err(format!("Shell not found: {shell}"));
    }
    Ok(())
}

fn validate_cli_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("Command name must not be empty".to_string());
    }
    if name.contains(char::is_whitespace) {
        return Err("Command name must not contain whitespace".to_string());
    }
    if name.contains('/') || name.contains('\\') {
        return Err("Command name must not contain path separators".to_string());
    }
    if name.contains(SHELL_METACHARACTERS) {
        return Err("Command name contains invalid characters".to_string());
    }
    Ok(())
}

#[tauri::command]
pub fn get_default_shell() -> String {
    std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string())
}

#[derive(serde::Serialize)]
pub struct SystemInfo {
    pub os: String,
    pub arch: String,
    pub hostname: String,
}

#[tauri::command]
pub fn get_system_info() -> Result<SystemInfo, String> {
    let hostname = Command::new("hostname")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    Ok(SystemInfo {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        hostname,
    })
}

#[tauri::command]
pub fn check_cli_exists(command: String) -> Result<bool, String> {
    validate_cli_name(&command)?;
    Command::new("which")
        .arg(&command)
        .output()
        .map(|o| o.status.success())
        .map_err(|e| e.to_string())
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum TerminalEvent {
    Output { data: Vec<u8> },
    Exit { code: Option<u32> },
}

#[tauri::command]
pub fn create_pty(
    shell: String,
    cwd: String,
    cols: u16,
    rows: u16,
    on_event: Channel<TerminalEvent>,
    state: State<'_, Arc<Mutex<PtyManager>>>,
) -> Result<String, String> {
    validate_shell_path(&shell)?;
    let mut manager = state.lock().map_err(|e| format!("lock poisoned: {e}"))?;
    let env_vars = vec![
        ("POWERLEVEL9K_DISABLE_CONFIGURATION_WIZARD".to_string(), "true".to_string()),
    ];
    let (id, reader) = manager
        .spawn(&shell, &cwd, cols, rows, env_vars)
        .map_err(|e: anyhow::Error| e.to_string())?;
    drop(manager);

    spawn_reader_thread(reader, on_event);

    Ok(id)
}

const BACKPRESSURE_HIGH_WATERMARK: usize = 512 * 1024;
const BACKPRESSURE_LOW_WATERMARK: usize = 128 * 1024;
const BACKPRESSURE_SLEEP: std::time::Duration = std::time::Duration::from_millis(1);

static PENDING_BYTES: AtomicUsize = AtomicUsize::new(0);

fn spawn_reader_thread(mut reader: Box<dyn Read + Send>, on_event: Channel<TerminalEvent>) {
    tauri::async_runtime::spawn_blocking(move || {
        let mut buf = [0u8; 65536];
        loop {
            while PENDING_BYTES.load(Ordering::Relaxed) > BACKPRESSURE_HIGH_WATERMARK {
                std::thread::sleep(BACKPRESSURE_SLEEP);
                if PENDING_BYTES.load(Ordering::Relaxed) <= BACKPRESSURE_LOW_WATERMARK {
                    break;
                }
            }

            match reader.read(&mut buf) {
                Ok(0) => {
                    let _ = on_event.send(TerminalEvent::Exit { code: None });
                    break;
                }
                Ok(n) => {
                    PENDING_BYTES.fetch_add(n, Ordering::Relaxed);
                    if on_event
                        .send(TerminalEvent::Output {
                            data: buf[..n].to_vec(),
                        })
                        .is_err()
                    {
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("PTY reader error: {e}");
                    let _ = on_event.send(TerminalEvent::Exit { code: None });
                    break;
                }
            }
        }
    });
}

#[tauri::command]
pub fn ack_terminal_data(bytes: usize) {
    PENDING_BYTES
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            Some(current.saturating_sub(bytes))
        })
        .ok();
}

#[tauri::command]
pub fn write_pty(
    id: String,
    data: String,
    state: State<'_, Arc<Mutex<PtyManager>>>,
) -> Result<(), String> {
    let instance = {
        let manager = state.lock().map_err(|e| format!("lock poisoned: {e}"))?;
        manager.get(&id).map_err(|e: anyhow::Error| e.to_string())?
    };
    let mut pty = instance.lock().map_err(|e| format!("pty lock poisoned: {e}"))?;
    pty.write(data.as_bytes())
        .map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub fn resize_pty(
    id: String,
    rows: u16,
    cols: u16,
    state: State<'_, Arc<Mutex<PtyManager>>>,
) -> Result<(), String> {
    let instance = {
        let manager = state.lock().map_err(|e| format!("lock poisoned: {e}"))?;
        manager.get(&id).map_err(|e: anyhow::Error| e.to_string())?
    };
    let mut pty = instance.lock().map_err(|e| format!("pty lock poisoned: {e}"))?;
    pty.resize(rows, cols)
        .map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub fn kill_pty(
    id: String,
    state: State<'_, Arc<Mutex<PtyManager>>>,
) -> Result<(), String> {
    let instance = {
        let manager = state.lock().map_err(|e| format!("lock poisoned: {e}"))?;
        manager.get(&id).map_err(|e: anyhow::Error| e.to_string())?
    };
    {
        let mut pty = instance.lock().map_err(|e| format!("pty lock poisoned: {e}"))?;
        pty.kill().map_err(|e: anyhow::Error| e.to_string())?;
    }
    let mut manager = state.lock().map_err(|e| format!("lock poisoned: {e}"))?;
    manager.remove(&id);
    Ok(())
}

