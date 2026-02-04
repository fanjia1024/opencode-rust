mod commands;
mod config;
mod session_store;
mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

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
    /// Create or refresh AGENTS.md in the current directory
    Init {
        /// Overwrite existing AGENTS.md with newly generated content
        #[arg(long, default_value_t = false)]
        refresh: bool,
    },
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
    // Ensure logs directory exists (for both tracing and tklog)
    let _ = std::fs::create_dir_all("logs");

    // File logging to ./logs/opencode.log for debugging (e.g. /init flow)
    tklog::LOG
        .set_console(false)
        .set_level(tklog::LEVEL::Info)
        .set_cutmode_by_size("logs/opencode.log", 1 << 20, 5, false);

    let cli = Cli::parse();

    // Tracing: for Tui, file-only so stdout is not mixed with the TUI; for other commands, stdout + file
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    let _guard = match &cli.command {
        Commands::Tui => {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("logs/opencode.log")?;
            let (file_writer, guard) = tracing_appender::non_blocking(file);
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt::Layer::new().with_writer(file_writer))
                .init();
            guard
        }
        _ => {
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("logs/opencode.log")?;
            let (file_writer, guard) = tracing_appender::non_blocking(file);
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt::Layer::new().with_writer(std::io::stdout))
                .with(fmt::Layer::new().with_writer(file_writer))
                .init();
            guard
        }
    };

    match cli.command {
        Commands::Tui => commands::tui::run_tui().await,
        Commands::Init { refresh } => commands::init::run_init(refresh).await,
        Commands::Run { command } => commands::run::run_command(&command).await,
        Commands::Serve { port } => commands::serve::serve(port).await,
        Commands::Sessions { subcommand } => match subcommand {
            SessionCommands::List => commands::sessions::list_sessions().await,
            SessionCommands::Show { session_id } => {
                commands::sessions::show_session(&session_id).await
            }
            SessionCommands::Delete { session_id } => {
                commands::sessions::delete_session(&session_id).await
            }
        },
        Commands::Config { subcommand } => match subcommand {
            ConfigCommands::Show => commands::config::show_config().await,
            ConfigCommands::Reset => commands::config::reset_config().await,
        },
    }
}
