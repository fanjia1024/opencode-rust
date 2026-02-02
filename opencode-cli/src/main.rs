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
    /// Manage sessions
    Sessions {
        #[command(subcommand)]
        subcommand: SessionCommands,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        subcommand: ConfigCommands,
    },
}

#[derive(Subcommand)]
enum SessionCommands {
    /// List all sessions
    List,
    /// Show a specific session
    Show {
        /// Session ID to show
        session_id: String,
    },
    /// Delete a specific session
    Delete {
        /// Session ID to delete
        session_id: String,
    },
}

#[derive(Subcommand)]
enum ConfigCommands {
    /// Show current configuration
    Show,
    /// Reset configuration to defaults
    Reset,
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
        Commands::Sessions { subcommand } => {
            match subcommand {
                SessionCommands::List => {
                    commands::sessions::list_sessions().await
                }
                SessionCommands::Show { session_id } => {
                    commands::sessions::show_session(&session_id).await
                }
                SessionCommands::Delete { session_id } => {
                    commands::sessions::delete_session(&session_id).await
                }
            }
        }
        Commands::Config { subcommand } => {
            match subcommand {
                ConfigCommands::Show => {
                    commands::config::show_config().await
                }
                ConfigCommands::Reset => {
                    commands::config::reset_config().await
                }
            }
        }
    }
}
