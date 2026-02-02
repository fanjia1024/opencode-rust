use anyhow::Result;
use crate::tui::App;

pub async fn run_tui() -> Result<()> {
    let mut app = App::new();
    app.run().await?;
    Ok(())
}
