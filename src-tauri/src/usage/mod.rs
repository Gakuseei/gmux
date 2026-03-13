use chrono::{DateTime, Utc};
use glob::glob;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct SessionUsage {
    pub session_id: String,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_read_tokens: u64,
    pub cache_write_tokens: u64,
    pub timestamp: String,
}

#[derive(Serialize)]
pub struct UsageData {
    pub total_input: u64,
    pub total_output: u64,
    pub total_cache_read: u64,
    pub total_cache_write: u64,
    pub sessions: Vec<SessionUsage>,
}

fn get_session_files() -> Vec<PathBuf> {
    let home = dirs::home_dir().unwrap_or_default();
    let pattern = home
        .join(".claude/projects/*/sessions/*.jsonl")
        .to_string_lossy()
        .to_string();

    glob(&pattern)
        .unwrap_or_else(|_| glob("").unwrap())
        .filter_map(|entry| entry.ok())
        .collect()
}

fn parse_session_file(path: &PathBuf, cutoff: DateTime<Utc>) -> Option<SessionUsage> {
    let metadata = fs::metadata(path).ok()?;
    let modified: DateTime<Utc> = metadata.modified().ok()?.into();

    if modified < cutoff {
        return None;
    }

    let session_id = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);

    let mut input_tokens: u64 = 0;
    let mut output_tokens: u64 = 0;
    let mut cache_read_tokens: u64 = 0;
    let mut cache_write_tokens: u64 = 0;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };

        let val: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        if let Some(usage) = val.get("usage") {
            input_tokens += usage
                .get("input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            output_tokens += usage
                .get("output_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            cache_read_tokens += usage
                .get("cache_read_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            cache_write_tokens += usage
                .get("cache_creation_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
        }
    }

    if input_tokens == 0 && output_tokens == 0 && cache_read_tokens == 0 && cache_write_tokens == 0
    {
        return None;
    }

    Some(SessionUsage {
        session_id,
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_write_tokens,
        timestamp: modified.to_rfc3339(),
    })
}

#[tauri::command]
pub fn get_usage_data(period: String) -> Result<UsageData, String> {
    let now = Utc::now();
    let cutoff = match period.as_str() {
        "today" => now - chrono::Duration::hours(24),
        "weekly" => now - chrono::Duration::days(7),
        "monthly" => now - chrono::Duration::days(30),
        _ => now - chrono::Duration::hours(24),
    };

    let files = get_session_files();
    let mut sessions: Vec<SessionUsage> = Vec::new();

    for path in &files {
        if let Some(session) = parse_session_file(path, cutoff) {
            sessions.push(session);
        }
    }

    sessions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let total_input: u64 = sessions.iter().map(|s| s.input_tokens).sum();
    let total_output: u64 = sessions.iter().map(|s| s.output_tokens).sum();
    let total_cache_read: u64 = sessions.iter().map(|s| s.cache_read_tokens).sum();
    let total_cache_write: u64 = sessions.iter().map(|s| s.cache_write_tokens).sum();

    Ok(UsageData {
        total_input,
        total_output,
        total_cache_read,
        total_cache_write,
        sessions,
    })
}
