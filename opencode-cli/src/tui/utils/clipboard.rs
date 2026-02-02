use opencode_core::error::Result;

pub fn copy_to_clipboard(text: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
                Ok(())
            })
            .map_err(|e| opencode_core::error::Error::Unknown(format!("Failed to copy to clipboard: {}", e)))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xclip")
            .arg("-selection")
            .arg("clipboard")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
                Ok(())
            })
            .map_err(|e| opencode_core::error::Error::Unknown(format!("Failed to copy to clipboard: {}", e)))?;
    }
    
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("clip")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child.stdin.as_mut().unwrap().write_all(text.as_bytes())?;
                Ok(())
            })
            .map_err(|e| opencode_core::error::Error::Unknown(format!("Failed to copy to clipboard: {}", e)))?;
    }
    
    Ok(())
}
