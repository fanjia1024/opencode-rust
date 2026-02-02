# Roadmap

This roadmap is derived directly from the current project scope defined in README.md.

The goal is **scope convergence**, not feature expansion.

---

## v0.2 — Scope Lock & Simplification

**Theme:** Make CLI + TUI the only first-class product
**Goal:** Reduce conceptual and architectural complexity

---

### 1. CLI scope tightening

* [ ] Clearly mark `serve` as **deprecated**

  * Hide from default help output
  * Add runtime warning when used
* [ ] Keep only two supported entry points:

  * `opencode tui`
  * `opencode run "question"` (may be renamed later)

**Non-goals**

* No HTTP API stabilization
* No background service mode

---

### 2. Session model simplification

* [ ] Redefine `Session` as **pure data**

  * ID
  * Message list
  * Timestamps
* [ ] Remove session responsibility for:

  * Tool lifecycle
  * Provider orchestration
  * IO scheduling
* [ ] Ensure session serialization format is stable and minimal

**Outcome**

* Sessions are portable, inspectable, and future-proof
* UI logic moves out of the data model

---

### 3. TUI behavior cleanup

* [ ] Treat TUI as the **primary product**
* [ ] Simplify Home → Chat → Config flow
* [ ] Reduce or remove periodic StateSync polling

  * Prefer manual refresh or event-based updates
* [ ] Improve visual clarity over adding new features

**Non-goals**

* No multi-pane dashboards
* No workflow or task execution UI

---

### 4. Tool system contraction

* [ ] Audit all existing tools
* [ ] Remove tools that are not:

  * Terminal-oriented
  * Directly useful in a coding chat
* [ ] Make tool set explicitly **internal**

  * No public registry API
  * No plugin interface

**Rule of thumb**

> If a tool does not make sense without a TUI, it probably does not belong here.

---

### 5. Documentation alignment

* [ ] Ensure README, PROJECT_SCOPE.md, and CLI help are consistent
* [ ] Remove or de-emphasize framework-like language
* [ ] Clearly communicate what PRs will be declined

---

## v0.3 — Product Polish & Stability

**Theme:** Make the terminal experience feel intentional
**Goal:** Improve usability without expanding scope

---

### 1. CLI UX polish

* [ ] Consider renaming `run` → `ask` (breaking change if needed)
* [ ] Improve error messages for:

  * Missing API keys
  * Provider initialization failures
* [ ] Ensure CLI output is script-friendly

---

### 2. TUI interaction refinement

* [ ] Better keyboard navigation and discoverability
* [ ] Clearer focus handling (input vs scroll)
* [ ] Improve long conversation rendering performance
* [ ] Reduce visual noise; prefer calm defaults

---

### 3. Provider experience stabilization

* [ ] Keep a single **default provider path**
* [ ] Clearly mark other providers as experimental
* [ ] Improve provider configuration UX in TUI
* [ ] Avoid provider-specific logic leaking into UI code

**Non-goals**

* No provider plugin marketplace
* No auto-selection or routing logic

---

### 4. Persistence & safety

* [ ] Ensure session data is never corrupted on crash
* [ ] Improve graceful shutdown behavior
* [ ] Validate session files before loading

---

### 5. Contribution boundaries

* [ ] Strengthen CONTRIBUTING.md with scope rules
* [ ] Add guidance for acceptable refactors vs feature PRs
* [ ] Explicitly discourage:

  * Server features
  * Agent autonomy
  * Workflow engines

---

## Explicitly out of scope (v0.x)

These items are intentionally postponed or rejected:

* HTTP API as a supported interface
* Autonomous agents or background workers
* Workflow definitions or task graphs
* Plugin ecosystems
* Cloud sync or SaaS features

---

## Guiding principle

> Every change must make OpenCode Rust a **better terminal companion**,
> not a more powerful platform.

If a feature increases architectural ambition more than user clarity,
it is probably out of scope.
