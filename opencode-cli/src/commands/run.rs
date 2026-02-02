use anyhow::Result;

pub async fn run_command(command: &str) -> Result<()> {
    println!("Running command: {}", command);
    Ok(())
}
