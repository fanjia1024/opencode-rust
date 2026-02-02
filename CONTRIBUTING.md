# Contributing to OpenCode Rust

Thank you for your interest in contributing to OpenCode Rust!

## Development Setup

1. Clone the repository
2. Install Rust (stable or latest)
3. Build the workspace:
   ```bash
   cargo build --workspace
   ```
4. Run tests:
   ```bash
   cargo test --workspace
   ```

## Code Style

- Follow Rust standard formatting: `cargo fmt`
- Run clippy: `cargo clippy --workspace`
- Ensure all tests pass before submitting

## Project Structure

- `opencode-core/`: Core abstractions and traits
- `opencode-provider/`: AI provider implementations
- `opencode-tools/`: Tool system
- `opencode-cli/`: CLI and TUI application

## Submitting Changes

1. Create a feature branch
2. Make your changes
3. Add tests if applicable
4. Ensure all tests pass
5. Submit a pull request
