# OpenCode Rust

**Local AI coding assistant in the terminal.** CLI + TUI only — no Server, no SDK.

## Use cases

- **Chat with an Agent in the terminal** – Discuss design and implementation, review diffs, iterate on code.
- **One-off Q&A** – Ask a question from the shell or scripts; `opencode run "your question"`.
- **Local conversation history** – Sessions live under `.opencode` in your project; pick up where you left off.

## Two ways to use

- **TUI (main)** – `opencode tui` — Home (session list / new session), Chat (conversation + input), Config (Provider / API key) via **`C`**.
- **One-off** – `opencode run "your question"` — Single question, scriptable. (May be renamed to `ask` later.)

We do **not** provide an HTTP API as a main product. The `serve` subcommand is **experimental and planned for removal**; use TUI or `run` instead.

## Features

- **TUI-first** – Home, Chat, and Config modal; session list; syntax highlighting and virtual scrolling.
- **Session = chat history** – Persisted under `.opencode/sessions`; list refreshes from disk (StateSync; may be simplified later).
- **Built-in tool set** – File read/write, grep, search, edit, patch, etc.; no public Registry API, no dynamic loading.
- **Default AI provider** – **langchain-ai-rust 5.0.1**; OpenAI and Anthropic via config. Other providers may be marked experimental.
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
- **Session directory** – Sessions are stored under `.opencode/sessions` in the current directory. The Home screen session list is refreshed from this directory (StateSync; may be simplified to event-driven or manual refresh later).

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
# Start the TUI (main)
cargo run --bin opencode -- tui

# One-off Q&A (scriptable)
cargo run --bin opencode -- run "your question"
```

With an installed `opencode` binary:

```bash
opencode tui
opencode run "your question"
```

**Note:** `opencode serve` is **experimental and planned for removal**. Do not rely on it; use TUI or `run` instead. See [PROJECT_SCOPE.md](PROJECT_SCOPE.md).

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

- Core TUI, session management, and session list (StateSync) are in place.
- **langchain-ai-rust** is the default built-in provider (OpenAI and Anthropic via configuration).
- Session persistence and provider configuration (TUI dialog) work. Scope and convergence plans are in [PROJECT_SCOPE.md](PROJECT_SCOPE.md).

## Documentation

- `USAGE.md` – Usage guide and flow.
- `SETUP_API_KEY.md` – API key setup.
- `LANGCHAIN_FIXES.md` – Notes on langchain-ai-rust integration.

## License

MIT. See `LICENSE` in this repository.
