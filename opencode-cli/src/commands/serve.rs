use anyhow::Result;

pub async fn serve(port: u16) -> Result<()> {
    eprintln!("WARNING: The 'serve' command is deprecated and will be removed in a future version.");
    eprintln!("Please use 'opencode tui' for interactive sessions or 'opencode run' for one-off questions.");
    println!("Starting HTTP server on port {} (deprecated)", port);
    Ok(())
}
