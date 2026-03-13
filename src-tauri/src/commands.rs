use crate::pty::PtyManager;
use std::io::Read;
use std::sync::{Arc, Mutex};
use tauri::ipc::Channel;
use tauri::State;

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
