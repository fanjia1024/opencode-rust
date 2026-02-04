# OpenCode Rust - Developer Guide

## Project Overview

OpenCode Rust is a lightweight, local-first client for chatting with an AI coding assistant. The project is a Rust workspace with five main crates: `opencode-core`, `opencode-provider`, `opencode-tools`, `opencode-cli`, and `opencode-app`. The **desktop app** (opencode-app, Tauri + Vue 3) provides the interactive UI; the **CLI** (opencode-cli) provides scriptable commands and launches the app via `opencode app`.

`opencode-core` contains the agent system, session state, caching, configuration, and permission handling. `opencode-provider` implements LLM provider adapters (including LangChain). `opencode-tools` provides the tool registry. `opencode-cli` exposes the library (session store, config, message processor, commands) and the `opencode` binary (run, sessions, config, init, app). `opencode-app` is the Tauri 2 desktop application (Vue 3 frontend, Rust backend invoking opencode-cli and core/provider/tools).

This project deliberately avoids agent frameworks, SDKs, and workflow engines. Instead, it provides a human-in-the-loop experience where the user drives the conversation and the agent assists without autonomous operation. Sessions are stored locally in `.opencode/sessions` within the user's project, and configuration follows platform-specific conventions in standard config directories.

## Directory Structure

The workspace root contains metadata and documentation alongside the crates. The `opencode-core/` crate holds agent management (`agent.rs`, `agent_manager.rs`), system infrastructure (`cache.rs`, `config.rs`, `session_state.rs`), and core abstractions (`tool.rs`, `ids.rs`, `permission.rs`). Session types live in `opencode-core/src/session/` (message, role, session).

The `opencode-cli/` crate provides the `opencode` binary and library: session persistence (`session_store.rs`), config, message processor, and CLI commands. The `opencode-app/` directory contains the Tauri desktop app (`src-tauri/` for Rust, `src/` for Vue 3 frontend). Provider implementations live in `opencode-provider/` with adapters for different LLM services (`anthropic.rs`, `langchain_adapter.rs`, `provider_adapter.rs`) and the trait definitions that enable provider flexibility.

Supporting crates include `opencode-tools/` for the tool registry and `tests/` for integration testing. Build and automation scripts reside in `scripts/`, while documentation files in the root provide historical context about implementation decisions, scope, and roadmap.

## Build

From the workspace root:

- **CLI only**: `cargo build -p opencode-cli` → `target/debug/opencode` or `target/release/opencode`.
- **Desktop app**: `cargo build -p opencode-app` → `target/debug/opencode-app` or `target/release/opencode-app`. The app frontend (Vue 3) is built by Tauri when running `cargo tauri dev` or `cargo tauri build` from the `opencode-app/` directory (requires `npm install` first).
- **All**: `cargo build --workspace` compiles every workspace member, including `opencode-app/src-tauri`.

Use `scripts/build.sh` and `scripts/build-release.sh` for convenience. To run the desktop app in development, run `npm install` and `cargo tauri dev` inside `opencode-app/`.

All crates are part of the same workspace, so Cargo automatically resolves dependencies across `opencode-core`, `opencode-cli`, `opencode-provider`, `opencode-tools`, and `opencode-app/src-tauri` without requiring separate compilation steps. Dependencies are managed through the root `Cargo.toml` workspace manifest and individual crate manifests.

## Test

The project maintains both unit tests within crate source files and integration tests in the dedicated `tests/` directory. Run the full test suite with `cargo test` from the workspace root. This executes tests across all workspace members including the core library, CLI application, provider implementations, and integration scenarios.

Integration tests are organized by functionality with files like `agent_test.rs`, `cache_test.rs`, `tool_test.rs`, and `basic_test.rs` covering major system components. New features should include corresponding integration tests that verify behavior across crate boundaries.

## Conventions

Code organization follows Rust best practices with clear module boundaries. The project uses `thiserror` for error definitions and `anyhow` for ergonomic error handling. Configuration management relies on platform-appropriate config directories, falling back to platform-specific defaults (AppData on Windows, Application Support on macOS, XDG config on Linux).

The agent system emphasizes explicit permission handling rather than autonomous action. Agents must request operations and receive user approval before executing tools or making provider calls. This design philosophy should guide any additions to the codebase—always prefer human-in-the-loop patterns over background automation.

Commits should follow conventional commit format for automated changelog generation. Code review should verify that new functionality respects the local-first, CLI + desktop app scope and does not introduce unnecessary dependencies or framework abstractions. When adding provider support, implement the established trait patterns in `opencode-provider/` rather than introducing provider-specific logic elsewhere in the codebase.
