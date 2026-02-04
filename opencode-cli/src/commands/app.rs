//! Launch the OpenCode desktop app (opencode-app). Use `opencode app` to start the Tauri UI.

use anyhow::{Context as AnyhowContext, Result};
use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Find the opencode-app binary: same directory as opencode binary, or target/debug / target/release.
fn find_opencode_app_binary() -> Option<PathBuf> {
    if let Ok(exe) = env::current_exe() {
        let dir = exe.parent()?;
        let name = if cfg!(windows) { "opencode-app.exe" } else { "opencode-app" };
        let same_dir = dir.join(name);
        if same_dir.exists() {
            return Some(same_dir);
        }
    }
    // Fallback: workspace target dir (when running via cargo run)
    let cwd = env::current_dir().ok()?;
    for subdir in ["target/release", "target/debug"] {
        let path = cwd.join(subdir).join(if cfg!(windows) { "opencode-app.exe" } else { "opencode-app" });
        if path.exists() {
            return Some(path);
        }
    }
    None
}

pub fn run_app() -> Result<()> {
    let binary = find_opencode_app_binary().with_context(|| {
        "opencode-app binary not found. Build it with: cargo build -p opencode-app"
    })?;
    let status = Command::new(&binary)
        .status()
        .with_context(|| format!("Failed to run {}", binary.display()))?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}
