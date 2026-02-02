# OpenCode Rust

**A local AI coding assistant in your terminal.**
CLI + TUI only — **no server, no SDK, no agent framework**.

OpenCode Rust is a lightweight, local-first terminal client for chatting with an AI agent while you design, review, and iterate on code.

---

## What this project is

* **A terminal-first product**
  * Interactive **TUI** for ongoing conversations
  * Scriptable **CLI** for one-off questions
* **Local-first**
  * Sessions live in your project under `.opencode/sessions`
  * No background services, no daemon
* **Human-in-the-loop**
  * You drive the conversation
  * The agent assists — it does not autonomously run workflows

---

## What this project is NOT

* No HTTP server / SaaS backend
* No Agent SDK or framework
* No workflow engine
* No long-running autonomous agents
* No plugin marketplace or dynamic tool loading

If you are looking for an agent platform or orchestration framework, this is **not** the right project.

---

## Primary use cases

* **Chat with an AI agent in the terminal**
  * Discuss architecture and design
  * Review code and diffs
  * Refine implementation ideas
* **One-off Q&A from the shell**
  * Ask a single question from scripts or CI
* **Persistent local context**
  * Pick up conversations where you left off
  * Sessions are plain data, stored on disk

---

## How you use it

### 1. TUI (main experience)

```bash
opencode tui
```

* Home: session list / create new session
* Chat: conversation + input
* Config: provider & API key (press `C`)

This is the primary way OpenCode Rust is meant to be used.

**Keybindings:** `n` new session, `C` config, `Esc` back to Home, `q` quit. See [USAGE.md](USAGE.md) for full keybindings.

### 2. One-off CLI questions

```bash
opencode run "Explain this Rust lifetime error"
```

* Single question
* No interactive session
* Designed to be scriptable

*(The `run` command may be renamed to `ask` in the future.)*

---

## Getting started

### Requirements

* Rust toolchain (1.70+ recommended)
* Network access for API calls when using the AI provider

### Installation and build

```bash
git clone https://github.com/fanjia1024/opencode-rust.git
cd opencode-rust
cargo build --workspace
```

To build **without** the AI provider:

```bash
cargo build --workspace --no-default-features
```

### Quick start

1. **Set API key** — `export OPENAI_API_KEY="your-key"` (or `OPENCODE_OPENAI_API_KEY`). You can also set provider and key in the TUI with `C`.
2. **Run the TUI** — `cargo run --bin opencode -- tui` (or `opencode tui` if installed).
3. **Basic flow** — From Home, press `n` to create a session; in Chat, type and press Enter to send. See [USAGE.md](USAGE.md) for details.

---

## Core design principles

### CLI + TUI only

OpenCode Rust is a **terminal application**, not a service.
There is no stable HTTP API.

> The `serve` subcommand is experimental and planned for removal.

### Sessions are just chat history

A session is:

* An ID
* A list of messages
* Timestamps

Sessions do **not** manage tools, providers, or execution logic.

### Built-in tools, not a platform

Tools exist only to improve the terminal coding experience:

* File read / write
* Search / grep
* Patch / edit helpers

There is:

* No public tool registry
* No dynamic loading
* No plugin API

### Opinionated, minimal providers

* Default provider: `langchain-ai-rust` (OpenAI / Anthropic)
* Other providers may exist but are **explicitly experimental**

---

## Project structure

```
opencode-rust/
├── opencode-core/      # Core data models and abstractions
├── opencode-provider/  # AI provider integrations
├── opencode-tools/     # Built-in tool set (internal)
└── opencode-cli/       # CLI and TUI (the product)
```

---

## Troubleshooting

* **No API key** — Set `OPENAI_API_KEY` or `OPENCODE_OPENAI_API_KEY`, or press `C` in the TUI to configure the provider.
* **Feature / langchain not enabled** — If you built with `--no-default-features`, use `cargo build --workspace` or `cargo build --workspace --features langchain`.
* **Error initializing provider** — Check API key and network reachability to the provider.

See [SETUP_API_KEY.md](SETUP_API_KEY.md) and [USAGE.md](USAGE.md) for more.

---

## Status

* Core TUI experience implemented
* Local session persistence
* Provider configuration via TUI
* Some components are intentionally overbuilt and will be simplified
* Scope is actively converging — see [PROJECT_SCOPE.md](PROJECT_SCOPE.md)

---

## Documentation

* [USAGE.md](USAGE.md) — Usage guide and flow
* [SETUP_API_KEY.md](SETUP_API_KEY.md) — API key setup
* [PROJECT_SCOPE.md](PROJECT_SCOPE.md) — Scope and contributions
* [TECH_DEBT.md](TECH_DEBT.md) — Future removals and convergence list
* [ROADMAP.md](ROADMAP.md) — v0.2 / v0.3 roadmap
* [LANGCHAIN_FIXES.md](LANGCHAIN_FIXES.md) — Notes on langchain-ai-rust integration

---

## License

MIT License. See [LICENSE](LICENSE) in this repository.

---

If you are interested in contributing, please read **[PROJECT_SCOPE.md](PROJECT_SCOPE.md)** first.
PRs that expand scope beyond CLI/TUI will likely be declined.
