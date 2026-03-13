use crate::pty::PtyManager;
use futures::future::join_all;
use std::io::Read;
use std::process::Command;
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;
use tauri::State;

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
    let mut manager = state.lock().map_err(|e| format!("lock poisoned: {e}"))?;
    let (id, reader) = manager
        .spawn(&shell, &cwd, cols, rows, vec![])
        .map_err(|e: anyhow::Error| e.to_string())?;
    drop(manager);

    spawn_reader_thread(reader, on_event);

    Ok(id)
}

fn spawn_reader_thread(mut reader: Box<dyn Read + Send>, on_event: Channel<TerminalEvent>) {
    tauri::async_runtime::spawn_blocking(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    on_event.send(TerminalEvent::Exit { code: None }).ok();
                    break;
                }
                Ok(n) => {
                    on_event
                        .send(TerminalEvent::Output {
                            data: buf[..n].to_vec(),
                        })
                        .ok();
                }
                Err(_) => break,
            }
        }
    });
}

#[tauri::command]
pub fn write_pty(
    id: String,
    data: String,
    state: State<'_, Arc<Mutex<PtyManager>>>,
) -> Result<(), String> {
    let manager = state.lock().map_err(|e| format!("lock poisoned: {e}"))?;
    manager
        .write(&id, data.as_bytes())
        .map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub fn resize_pty(
    id: String,
    rows: u16,
    cols: u16,
    state: State<'_, Arc<Mutex<PtyManager>>>,
) -> Result<(), String> {
    let manager = state.lock().map_err(|e| format!("lock poisoned: {e}"))?;
    manager
        .resize(&id, rows, cols)
        .map_err(|e: anyhow::Error| e.to_string())
}

#[tauri::command]
pub fn kill_pty(
    id: String,
    state: State<'_, Arc<Mutex<PtyManager>>>,
) -> Result<(), String> {
    let mut manager = state.lock().map_err(|e| format!("lock poisoned: {e}"))?;
    manager
        .kill(&id)
        .map_err(|e: anyhow::Error| e.to_string())
}

#[derive(serde::Deserialize)]
pub struct SpawnRequest {
    pub shell: String,
    pub cwd: String,
    pub command: Option<String>,
    pub cols: u16,
    pub rows: u16,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchTerminalEvent {
    pub index: usize,
    pub event: TerminalEvent,
}

fn spawn_batch_reader_thread(
    mut reader: Box<dyn Read + Send>,
    index: usize,
    on_event: Channel<BatchTerminalEvent>,
) {
    tauri::async_runtime::spawn_blocking(move || {
        let mut buf = [0u8; 4096];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    on_event
                        .send(BatchTerminalEvent {
                            index,
                            event: TerminalEvent::Exit { code: None },
                        })
                        .ok();
                    break;
                }
                Ok(n) => {
                    on_event
                        .send(BatchTerminalEvent {
                            index,
                            event: TerminalEvent::Output {
                                data: buf[..n].to_vec(),
                            },
                        })
                        .ok();
                }
                Err(_) => break,
            }
        }
    });
}

#[tauri::command]
pub async fn spawn_batch(
    requests: Vec<SpawnRequest>,
    on_event: Channel<BatchTerminalEvent>,
    state: State<'_, Arc<Mutex<PtyManager>>>,
) -> Result<Vec<String>, String> {
    let manager = Arc::clone(&state);

    let tasks: Vec<_> = requests
        .into_iter()
        .enumerate()
        .map(|(index, req)| {
            let mgr = Arc::clone(&manager);
            let channel = on_event.clone();
            tokio::spawn(async move {
                let (id, reader) = {
                    let mut locked = mgr.lock().map_err(|e| format!("lock poisoned: {e}"))?;
                    locked
                        .spawn(&req.shell, &req.cwd, req.cols, req.rows, vec![])
                        .map_err(|e: anyhow::Error| e.to_string())?
                };

                spawn_batch_reader_thread(reader, index, channel);

                if let Some(cmd) = req.command {
                    let mgr_write = Arc::clone(&mgr);
                    let pty_id = id.clone();
                    tokio::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        let locked = mgr_write.lock().ok();
                        if let Some(m) = locked {
                            let payload = format!("{cmd}\r");
                            let _ = m.write(&pty_id, payload.as_bytes());
                        }
                    });
                }

                Ok::<String, String>(id)
            })
        })
        .collect();

    let results = join_all(tasks).await;

    results
        .into_iter()
        .map(|r| r.map_err(|e| format!("task join error: {e}"))?)
        .collect()
}
