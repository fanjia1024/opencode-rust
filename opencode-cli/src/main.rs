mod commands;
mod config;
mod session_store;
mod tui;

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "opencode")]
#[command(about = "The open source AI coding agent")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the TUI interface
    Tui,
    /// Run a command
    Run {
        /// The command to run
        command: String,
    },
    /// (Deprecated) HTTP server – planned removal; use TUI or run for CLI
    #[clap(hide = true)]
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Tui => {
            commands::tui::run_tui().await
        }
        Commands::Run { command } => {
            commands::run::run_command(&command).await
        }
        Commands::Serve { port } => {
            commands::serve::serve(port).await
        }
    }
}
