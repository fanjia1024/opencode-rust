//! OpenCode CLI library: session store, config, message processor, and commands for use by the CLI binary and Tauri app.

pub mod commands;
pub mod config;
pub mod message_processor;
pub mod session_store;

pub use config::{AppConfig, ProviderInfo, ProviderListItem};
pub use message_processor::{process_message_async, LogEntry, LogLevel, SessionUpdate};
pub use session_store::{load_session, save_session};
