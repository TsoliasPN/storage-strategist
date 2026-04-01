use crate::model::ScanHistory;
use anyhow::Result;
use std::fs;
use std::path::PathBuf;

const CACHE_DIR_NAME: &str = "storage-strategist-cache";
const HISTORY_FILE_NAME: &str = "history.json";

fn history_cache_dir() -> PathBuf {
    std::env::temp_dir().join(CACHE_DIR_NAME)
}

fn history_file_path() -> PathBuf {
    history_cache_dir().join(HISTORY_FILE_NAME)
}

pub fn load_history() -> Result<ScanHistory> {
    let path = history_file_path();
    if !path.exists() {
        return Ok(ScanHistory::default());
    }

    let payload = fs::read_to_string(path)?;
    let history: ScanHistory = serde_json::from_str(&payload)?;
    Ok(history)
}

pub fn save_history(history: &ScanHistory) -> Result<()> {
    let path = history_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let payload = serde_json::to_string_pretty(history)?;
    fs::write(path, payload)?;
    Ok(())
}
