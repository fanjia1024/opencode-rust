use anyhow::Result;

pub async fn serve(port: u16) -> Result<()> {
    println!("Starting HTTP server on port {}", port);
    Ok(())
}
