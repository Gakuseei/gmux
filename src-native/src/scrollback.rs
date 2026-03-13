use std::fs;
use std::path::PathBuf;

use anyhow::Context;

fn scrollback_dir() -> anyhow::Result<PathBuf> {
    let config_dir = dirs::config_dir().context("no config directory")?;
    Ok(config_dir.join("gmux").join("scrollback"))
}

fn validate_id(terminal_id: &str) -> anyhow::Result<()> {
    uuid::Uuid::parse_str(terminal_id)
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!("invalid terminal id: {e}"))
}

pub fn save_scrollback(terminal_id: &str, content: &str) -> anyhow::Result<()> {
    validate_id(terminal_id)?;
    let dir = scrollback_dir()?;
    fs::create_dir_all(&dir)?;
    let target = dir.join(format!("{terminal_id}.txt"));
    let tmp = dir.join(format!("{terminal_id}.txt.tmp"));
    fs::write(&tmp, content)?;
    fs::rename(&tmp, &target)?;
    Ok(())
}

pub fn load_scrollback(terminal_id: &str) -> anyhow::Result<Option<String>> {
    validate_id(terminal_id)?;
    let path = scrollback_dir()?.join(format!("{terminal_id}.txt"));
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        Ok(Some(content))
    } else {
        Ok(None)
    }
}

pub fn delete_scrollback(terminal_id: &str) -> anyhow::Result<()> {
    validate_id(terminal_id)?;
    let path = scrollback_dir()?.join(format!("{terminal_id}.txt"));
    if path.exists() {
        fs::remove_file(&path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load_roundtrip() {
        let id = uuid::Uuid::new_v4().to_string();
        let content = "$ echo hello\nhello\n$ ";
        save_scrollback(&id, content).unwrap();
        let loaded = load_scrollback(&id).unwrap();
        assert_eq!(loaded, Some(content.to_string()));
        delete_scrollback(&id).unwrap();
    }

    #[test]
    fn load_nonexistent_returns_none() {
        let id = uuid::Uuid::new_v4().to_string();
        let loaded = load_scrollback(&id).unwrap();
        assert_eq!(loaded, None);
    }

    #[test]
    fn invalid_id_rejected() {
        let result = save_scrollback("../../../etc/passwd", "bad");
        assert!(result.is_err());
    }
}
