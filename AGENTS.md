# OpenCode Rust - Developer Guide

## Project Overview

OpenCode Rust is a lightweight, local-first terminal client for chatting with an AI coding assistant. The project is built as a Rust workspace consisting of four main crates that work together to provide a complete CLI and TUI experience without requiring any server infrastructure.

The architecture follows a clear separation of concerns across the workspace. `opencode-core` contains the fundamental domain logic including the agent system, session state management, caching, configuration, and permission handling. `opencode-provider` implements the interface layers for connecting to various LLM providers like Anthropic, with adapters for different protocols including LangChain integration. `opencode-tools` provides a registry system for exposing functionality to agents, while `opencode-cli` delivers the user-facing terminal interface with both interactive TUI and scriptable CLI modes.

This project deliberately avoids agent frameworks, SDKs, and workflow engines. Instead, it provides a human-in-the-loop experience where the user drives the conversation and the agent assists without autonomous operation. Sessions are stored locally in `.opencode/sessions` within the user's project, and configuration follows platform-specific conventions in standard config directories.

## Directory Structure

The workspace root contains several metadata and documentation files alongside the core project directories. The `opencode-core/` crate houses the primary application logic with modules for agent management (`agent.rs`, `agent_manager.rs`), system infrastructure (`cache.rs`, `config.rs`, `session_state.rs`), and core abstractions (`tool.rs`, `ids.rs`, `permission.rs`). The `opencode/` subdirectory within core contains session-related implementations.

The `opencode-cli/` crate provides the entry point through `main.rs` and organizes functionality into the TUI layer (`tui/`), session persistence (`session_store.rs`), and CLI commands. Provider implementations live in `opencode-provider/` with adapters for different LLM services (`anthropic.rs`, `langchain_adapter.rs`, `provider_adapter.rs`) and the trait definitions that enable provider flexibility.

Supporting crates include `opencode-tools/` for the tool registry and `tests/` for integration testing. Build and automation scripts reside in `scripts/`, while documentation files in the root provide historical context about implementation decisions, scope, and roadmap.

## Build

The project uses standard Cargo commands for building. Execute `cargo build` from the workspace root to compile all crates in debug mode. For release builds optimized for distribution, run `cargo build --release` which places binaries in the `target/release/` directory. The provided `scripts/build.sh` and `scripts/build-release.sh` scripts offer convenience wrappers for common build operations.

All crates are part of the same workspace, so Cargo automatically resolves dependencies across `opencode-core`, `opencode-cli`, `opencode-provider`, and `opencode-tools` without requiring separate compilation steps. Dependencies are managed through the root `Cargo.toml` workspace manifest and individual crate manifests.

## Test

The project maintains both unit tests within crate source files and integration tests in the dedicated `tests/` directory. Run the full test suite with `cargo test` from the workspace root. This executes tests across all workspace members including the core library, CLI application, provider implementations, and integration scenarios.

Integration tests are organized by functionality with files like `agent_test.rs`, `cache_test.rs`, `tool_test.rs`, and `basic_test.rs` covering major system components. New features should include corresponding integration tests that verify behavior across crate boundaries.

## Conventions

Code organization follows Rust best practices with clear module boundaries. The project uses `thiserror` for error definitions and `anyhow` for ergonomic error handling. Configuration management relies on platform-appropriate config directories, falling back to platform-specific defaults (AppData on Windows, Application Support on macOS, XDG config on Linux).

The agent system emphasizes explicit permission handling rather than autonomous action. Agents must request operations and receive user approval before executing tools or making provider calls. This design philosophy should guide any additions to the codebase—always prefer human-in-the-loop patterns over background automation.

Commits should follow conventional commit format for automated changelog generation. Code review should verify that new functionality respects the local-first, terminal-only principles and does not introduce unnecessary dependencies or framework abstractions. When adding provider support, implement the established trait patterns in `opencode-provider/` rather than introducing provider-specific logic elsewhere in the codebase.
