# OpenCode Rust Implementation

Rust implementation of OpenCode for better performance and resource efficiency. It ships with **langchain-ai-rust** as the default AI provider, so you get working LLM integration out of the box.

## Features

- **TUI** – Terminal UI with syntax highlighting, virtual scrolling, Home and Session screens, and a session list synced from disk (StateSync).
- **Default AI Provider** – Built on **langchain-ai-rust 5.0.1**, with support for OpenAI and Anthropic (via configuration).
- **Tool system** – 16+ tools for code, files, web search, and more.
- **Session management** – Sessions are persisted (save/load) and the Home screen list refreshes from the session directory.
- **Provider abstraction** – Pluggable providers; configure API key and provider type in TUI or via config.
- **Caching** – LRU cache for performance.

## Requirements

- Rust toolchain (1.70+ recommended)
- Network access for API calls when using the AI provider

## Installation and build

Clone and build the workspace (default build includes langchain-ai-rust):

```bash
git clone https://github.com/fanjia1024/opencode-rust.git
cd opencode-rust
cargo build --workspace
```

To build **without** the AI provider (no langchain-ai-rust):

```bash
cargo build --workspace --no-default-features
```

## Quick start

### 1. Set API key

Set one of these environment variables:

```bash
export OPENAI_API_KEY="your-openai-api-key"
# or
export OPENCODE_OPENAI_API_KEY="your-openai-api-key"
```

You can also configure the provider later in the TUI by pressing **`C`** (see [Configuration](#configuration)).

### 2. Run the TUI

```bash
cargo run --bin opencode -- tui
```

### 3. Basic flow

1. **Home** – Press **`n`** to create a new session (or use the session list if you have saved sessions).
2. **Session** – Type your message in the input area and press **Enter** to send.
3. The assistant reply is shown in the conversation; responses use the configured provider (e.g. OpenAI via langchain-ai-rust).

## Configuration

- **Provider** – The app reads the default provider from config or environment. In the TUI, press **`C`** to open the Provider dialog and set provider type (e.g. OpenAI, Anthropic), API key, and optional base URL. Config is stored under the platform config directory (e.g. `~/.config/opencode` on Linux/macOS).
- **Session directory** – Sessions are stored under the configured `session_dir` (e.g. under the app data directory). The Home screen session list is refreshed from this directory about every 5 seconds (StateSync).

## TUI usage

### Home screen

- **`n`** – Create a new session.
- **`q`** – Quit the application.
- The session list shows persisted sessions from disk; it updates automatically.

### Session screen

- **`Esc`** – Return to Home.
- **`↑` / `↓`** – Scroll messages.
- Type in the input box; **Enter** sends the message, **Backspace** deletes a character.
- **`q`** – Quit.

### Global

- **`C`** – Open the Provider configuration dialog (API key, provider type, base URL).

## CLI commands

After building:

```bash
# Start the TUI
cargo run --bin opencode -- tui

# Run a single command
cargo run --bin opencode -- run "your command"

# Start the HTTP server (default port 8080)
cargo run --bin opencode -- serve --port 8080
```

With an installed `opencode` binary:

```bash
opencode tui
opencode run "your command"
opencode serve --port 8080
```

## Project structure

```
opencode-rust/
├── opencode-core/          # Core abstractions and interfaces
├── opencode-provider/      # AI provider (langchain-ai-rust 5.0.1)
├── opencode-tools/         # Tool system
└── opencode-cli/           # CLI and TUI
```

## Troubleshooting

- **No API key configured** – Set `OPENAI_API_KEY` or `OPENCODE_OPENAI_API_KEY`, or press **`C`** in the TUI to configure the provider.
- **Feature / langchain not enabled** – If you built with `--no-default-features`, the AI provider is disabled. Use a normal build (`cargo build --workspace`) or enable the feature explicitly: `cargo build --workspace --features langchain`.
- **Error initializing provider** – Check that your API key is valid and that the machine can reach the provider (e.g. OpenAI) over the network.

For more detail, see `SETUP_API_KEY.md` and `USAGE.md`.

## Status

- Core TUI, session management, and StateSync-based session list are in place.
- **langchain-ai-rust** is the default built-in provider (OpenAI and Anthropic via configuration).
- Session persistence and provider configuration (including TUI dialog) work with the current codebase.

## Documentation

- `USAGE.md` – Usage guide and flow.
- `SETUP_API_KEY.md` – API key setup.
- `LANGCHAIN_FIXES.md` – Notes on langchain-ai-rust integration.

## License

MIT. See `LICENSE` in this repository.
