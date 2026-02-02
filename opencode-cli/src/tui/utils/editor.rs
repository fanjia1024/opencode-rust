use opencode_core::error::Result;
use std::process::Command;
use tokio::process::Command as AsyncCommand;

pub async fn open_editor(path: &str) -> Result<()> {
    let editor = std::env::var("EDITOR")
        .unwrap_or_else(|_| {
            if cfg!(target_os = "windows") {
                "notepad".to_string()
            } else {
                "vi".to_string()
            }
        });

    AsyncCommand::new(&editor)
        .arg(path)
        .spawn()
        .map_err(|e| opencode_core::error::Error::Unknown(format!("Failed to open editor: {}", e)))?;

    Ok(())
}

pub fn get_editor() -> String {
    std::env::var("EDITOR").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            "notepad".to_string()
        } else {
            "vi".to_string()
        }
    })
}
