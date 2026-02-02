# OpenCode Rust Implementation

This is the Rust implementation of OpenCode, migrating from TypeScript/Bun to Rust for better performance and resource efficiency.

## Project Structure

```
opencode-rust/
├── opencode-core/          # Core abstractions and interfaces
├── opencode-provider/      # AI Provider implementations (based on langchain-rust)
├── opencode-tools/         # Tool system
└── opencode-cli/           # CLI and TUI application
```

## Building

### Without langchain-rust (default)
```bash
cargo build --workspace
```

### With langchain-rust support
```bash
cargo build --workspace --features langchain
```

## Running

### TUI Interface
```bash
# Without langchain-rust
cargo run --bin opencode -- tui

# With langchain-rust
cargo run --bin opencode --features langchain -- tui
```

### TUI Controls

**Home Screen:**
- **`n`** - Create a new session
- **`q`** - Quit the application

**Session Screen:**
- **`Esc`** - Return to home screen
- **`↑/↓`** - Scroll messages
- **Type text** - Enter your message in the input box
- **`Enter`** - Send message
- **`Backspace`** - Delete character
- **`q`** - Quit the application

### CLI Commands
```bash
# Run a command
cargo run --bin opencode -- run "your command"

# Start HTTP server
cargo run --bin opencode -- serve --port 8080
```

## Features

- ✅ **TUI Interface** - Terminal User Interface with syntax highlighting and virtual scrolling
- ✅ **Langchain-Rust Integration** - AI provider support via langchain-rust
- ✅ **Tool System** - 16+ tools for code manipulation, file operations, web search, etc.
- ✅ **Session Management** - Persistent session state with message history
- ✅ **Provider Abstraction** - Support for multiple AI providers (OpenAI, Anthropic, etc.)
- ✅ **Caching** - LRU cache for performance optimization

## Status

✅ **Core functionality complete** - The project has successfully migrated to Rust with:
- Complete TUI implementation
- Langchain-rust integration
- Tool system with 16+ tools
- Session management
- Provider abstraction layer
- Performance optimizations

See `LANGCHAIN_FIXES.md` for details on langchain-rust integration fixes.
