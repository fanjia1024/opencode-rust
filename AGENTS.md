<

# AGENTS.md

## Project Overview

OpenCode Rust is a local AI coding assistant built as a terminal-first application. It provides both an interactive TUI for ongoing conversations and a scriptable CLI for one-off queries. The application is designed around human-in-the-loop principles—you drive the conversation while the agent assists, without autonomous workflow execution.

The project consists of four main crates within a Rust workspace:

- **opencode-core**: The central library containing agent management, session state, caching, configuration, permission handling, and tool registration.
- **opencode-provider**: Adapters for various AI providers (Anthropic, LangChain, custom implementations), message handling, and provider abstraction.
- **opencode-tools**: A registry of built-in tools that agents can invoke during conversations.
- **opencode-cli**: The command-line interface and terminal user interface implementation.

## Directory Structure

```
opencode-core/src/        # Core library: agent, session, cache, config, tools
opencode-provider/src/    # Provider adapters and AI service integrations
opencode-tools/src/       # Built-in tool definitions and registry
opencode-cli/src/         # CLI entry point and TUI implementation
tests/integration/        # Integration tests for core functionality
scripts/                  # Build and release automation scripts
Cargo.toml                # Workspace manifest
```

## Build

Build the entire workspace from the root directory:

```bash
cargo build
```

For release builds with optimizations:

```bash
cargo build --release
```

Or use the provided build scripts:

```bash
./scripts/build.sh        # Development build
./scripts/build-release.sh # Optimized release build
```

Individual crates can also be built separately:

```bash
cargo build -p opencode-core
cargo build -p opencode-cli
```

## Test

Run the full test suite from the workspace root:

```bash
cargo test
```

Run tests for a specific crate:

```bash
cargo test -p opencode-core
cargo test -p opencode-provider
```

Run integration tests specifically:

```bash
cargo test --test integration
```

Check code coverage and linting:

```bash
cargo clippy
cargo fmt --check
```

## Conventions

### Code Organization

- Each crate maintains its own `src/lib.rs` as the public API entry point
- Internal modules are organized by functionality (e.g., `agent.rs`, `session_state.rs`, `cache.rs`)
- Public APIs are clearly marked in `lib.rs` exports
- Tests reside alongside implementation files (`tests.rs`) or in dedicated integration test directories

### Error Handling

- Use the `error.rs` module for domain-specific error types
- Errors should be descriptive and include context for debugging
- Propagate errors using the `?` operator where appropriate
- Consider using `thiserror` for deriving error types

### Configuration

- Configuration follows platform-specific standards (`.config/opencode/` on Linux/macOS, AppData on Windows)
- Session data stored in `.opencode/sessions/` within each project
- Use the `config.rs` module for loading and managing settings

### Tool Development

- Tools are registered through `opencode-tools/src/tools/registry.rs`
- Implement the `Tool` trait from the core library
- Tools should have clear input/output contracts
- Permissions are managed through the permission system in `opencode-core/src/permission.rs`

### Provider Integration

- New providers should implement the provider trait in `opencode-provider/src/trait_.rs`
- Use the adapter pattern for provider-specific implementations
- Message formats follow the structure defined in `opencode-provider/src/message.rs`
- Caching is handled through `CachedProvider` for performance

### Session Management

- Sessions are stateful and persisted to disk
- Use `session_state.rs` for tracking conversation context
- Session storage uses the standard session store pattern in `opencode-cli/src/session_store.rs`

### Contributing

- Follow Rust idioms and the project's existing patterns
- Run `cargo fmt` before committing
- Add tests for new functionality
- Update documentation as needed
- See `CONTRIBUTING.md` for detailed guidelines