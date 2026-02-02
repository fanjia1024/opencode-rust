use opencode_core::error::Result;

pub fn get_terminal_size() -> Result<(u16, u16)> {
    use crossterm::terminal;
    let (cols, rows) = terminal::size()
        .map_err(|e| opencode_core::error::Error::Unknown(format!("Failed to get terminal size: {}", e)))?;
    Ok((cols, rows))
}

pub fn detect_background_color() -> Result<String> {
    Ok("dark".to_string())
}
