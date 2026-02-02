use anyhow::Result;
use opencode_core::session::Session;
use std::path::Path;

/// Saves a session to disk. Creates parent directory if needed.
/// Path should be the full path to session.json.
pub fn save_session(path: &Path, session: &Session) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let data = serde_json::to_string_pretty(session)?;
    std::fs::write(path, data)?;
    Ok(())
}

/// Loads a session from disk.
/// Path should be the full path to session.json.
pub fn load_session(path: &Path) -> Result<Session> {
    let data = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&data)?)
}
