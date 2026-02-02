# Project scope

## Project goal

**OpenCode Rust** is a **local CLI / TUI Agent client**. It focuses on high-quality interaction between you, the Agent, and your local system.

- We do **not** build an HTTP Server as a main product.
- We do **not** build an Agent SDK for third-party integration.
- We do **not** build a Workflow / DAG engine.

---

## In scope

- **CLI**
  - `opencode tui` – main product (TUI).
  - One-off Q&A: `opencode run "your question"` (may be renamed to `ask` later).
- **TUI**
  - **Home** – Session list and new session.
  - **Chat** – Conversation and input.
  - **Config (modal)** – Provider, API key, optional tool toggles.
- **Session**
  - Chat history plus minimal state; persisted under `.opencode/sessions`.
- **Provider**
  - One default provider; others may be marked experimental.
- **Tools**
  - Built-in tool set for the TUI/CLI. No public Registry API, no dynamic loading.

---

## Out of scope

- HTTP API / `serve` as a main product (experimental only; planned removal or long-term freeze).
- Acting as an Agent SDK for third-party integration.
- Generic Workflow / DAG orchestration.
- Multi-tenant, remote sessions, or cloud sync.

---

## Contributions and PRs

- We do **not** accept PRs that expand the above scope (e.g. adding a long-term supported HTTP server, or a public Tool Registry API).
- Experimental features must be clearly marked in docs and in `--help`.

---

## Kept for now but planned convergence

- **`serve`** – Experimental; will be frozen and marked “planned removal”. Prefer TUI or `run` for CLI usage.
- **StateSync polling** – May be replaced by event-driven or manual refresh later.
- **Some tools** – See “Convergence checklist” below.

---

## Convergence checklist (for later implementation)

Use this list when simplifying the codebase. Do not treat it as immediate code changes.

### CLI subcommands

| Item        | Recommendation                    |
| ----------- | --------------------------------- |
| `tui`       | Keep; main product                |
| `run "..."` | Keep; consider renaming to `ask`  |
| `serve`     | Freeze; mark “planned removal”    |

### TUI

| Item                      | Recommendation                                      |
| ------------------------- | ---------------------------------------------------- |
| Three screens (Home/Chat/Config modal) | Treat as main shape; other screens/dialogs as reserved |
| StateSync 5s polling      | Replace with event-driven or manual refresh later   |
| Session list              | Keep; do not use Session as workflow orchestration  |

### Session model (`opencode-core/src/session.rs`)

- Session = **chat history + minimal state** only.
- Orchestration (calling Provider, dispatching Tools) lives in the CLI runner / TUI controller, not in Session.
- Later: reduce Session’s role to a pure data structure + load/save; no lifecycle or I/O scheduling in Session.

### Tools (`opencode-tools/src/tools/`)

- **Principle**: Prefer tools that only make sense in TUI/CLI; mark the rest as removable or experimental.
- **Keep candidates**: read, write, grep, ls, bash (restricted), codesearch, edit, patch (strong terminal use).
- **Consider cutting or merging**: batch, multiedit, task, todo, question (weaker link to “terminal chat”); webfetch/websearch if placeholder-only, mark experimental.
- **Registry**: Do not expose publicly; no dynamic loading; built-in list only.
