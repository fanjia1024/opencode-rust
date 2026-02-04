# v0.2 Scope Lock — Code Deletion & Refactor Checklist

This checklist maps the current codebase to the v0.2 roadmap.
Anything listed here is required to keep the project aligned with README.md.

---

## 🪓 MUST DELETE (or fully deprecate)

### 1. HTTP Server Path (serve)

**Why**

* README: "CLI + TUI only — no Server"
* Any server code implies a platform / API mindset

**Action**

* [ ] Mark `opencode serve` as deprecated in CLI
* [ ] Hide it from default help output
* [ ] Add runtime warning: *"serve will be removed in v0.3"*
* [ ] Stop adding any new features or fixes to server code

**Target (v0.3)**

* Full removal

**Concrete mapping**

| Files / code | Action |
|--------------|--------|
| `opencode-cli/src/main.rs` — enum `Commands::Serve`, doc comment already says "planned removal" | Deprecate: hide `Serve` from default `--help` (e.g. clap `#[command(hide = true)]` or `hide_subcommands`), add runtime `eprintln!`/tracing warning "serve will be removed in v0.3" when invoked. Do not delete yet; full removal in v0.3. |
| `opencode-cli/src/commands/serve.rs` — stub `serve(port)` | Keep; add warning when invoked. |
| `opencode-cli/src/commands/mod.rs` — `pub mod serve` | No change. |

---

### 2. Agent / Framework-oriented documentation

**Files**

* `API.md`
* `AGENT_INTEGRATION.md`
* Any docs implying SDK / framework usage

**Why**

* README explicitly rejects Agent SDK / framework positioning
* These docs attract the wrong contributors

**Action**

* [ ] Delete, or
* [ ] Add large banner: *"Historical / not a supported direction"*

**Concrete mapping**

| Files / code | Action |
|--------------|--------|
| `API.md` — OpenCode Rust API Documentation, Agent/Tool/Provider traits | Delete, or add a large top banner: "Historical / not a supported direction. This project is CLI + TUI only; see README and PROJECT_SCOPE.md." |
| `AGENT_INTEGRATION.md` — Agent 集成完成报告, AgentManager, Langchain | Same as above. |

---

### 3. Overbuilt Tool Infrastructure APIs

**Symptoms**

* Public tool registry
* Generic execution abstraction
* Tool lifecycle hooks

**Why**

* Tools are *internal UX helpers*, not an extensibility point

**Action**

* [ ] Remove public-facing tool registry APIs
* [ ] Make tool registration private to `opencode-cli`
* [ ] Delete any code supporting dynamic loading / discovery

**Concrete mapping**

| Files / code | Action |
|--------------|--------|
| `opencode-tools/src/lib.rs` — `pub use registry::ToolRegistry` | Remove or narrow public API: make `ToolRegistry` and `register_all_tools` internal to the crate or to opencode-cli only (e.g. `pub(crate)` + keep one internal entry point for CLI). |
| `opencode-tools/src/registry.rs` — public `ToolRegistry`, `register`, `get`, `list`, `execute` | Same; no dynamic loading code found; document "no plugin interface". |
| `opencode-tools/src/tools/mod.rs` — `pub fn register_all_tools` | Same. Tests in `tests/integration/` use `ToolRegistry` and `register_all_tools` — keep tests but use internal API. |

---

## 🔧 MUST CHANGE (architecture correction)

### 4. Session Model → Pure Data

**Current smell**

* Session knows about:
  * Providers
  * Tools
  * Execution / orchestration

**Target**

```rust
struct Session {
    id: SessionId,
    messages: Vec<Message>,
    created_at,
    updated_at,
}
```

**Action**

* [ ] Move provider orchestration out of `Session`
* [ ] Move tool invocation logic into CLI/TUI controllers
* [ ] Ensure session serialization is stable and minimal

**Rule**

> Session should be loadable without initializing a provider.

**Concrete mapping**

| Files / code | Action |
|--------------|--------|
| `opencode-core/src/session.rs` — `Session` already has only `id`, `project_id`, `directory`, `title`, `messages`, `created_at`, `updated_at`; no provider/tool fields | Keep `Session` as-is (already data-only). |
| `opencode-core/src/session_state.rs` — `SessionStateMachine` (Idle, Processing, WaitingForTool, etc.) is orchestration state, not persisted with Session | Optionally move `SessionStateMachine` to opencode-cli as runtime-only state, or leave in core but document that Session is loadable without initializing a provider. |
| `opencode-core/src/agent.rs` — `Agent::process(ctx, input, session, provider, tools)` mutates `Session` (e.g. add_message); orchestration lives in Agent/CLI | Ensure session serialization stays minimal (already only Session fields in session.json). |

---

### 5. StateSync Polling (TUI)

**Current**

* Periodic refresh (polling `.opencode/sessions`)

**Why this must change**

* Polling implies multi-writer / service mindset
* TUI is the only writer in normal usage

**Action**

* [ ] Replace polling with:
  * Manual refresh (keybind), or
  * Event-driven refresh (if trivial)
* [ ] Remove background timers from core UI loop

**Concrete mapping**

| Files / code | Action |
|--------------|--------|
| `opencode-cli/src/tui/sync.rs` — `StateSync` with `sync_interval: Duration::from_secs(5)`, `sync_if_needed()` | Replace polling: (a) Remove `sync_if_needed()` from the main loop; (b) Trigger sync on entering Home screen and/or on a keybind (e.g. `r` for refresh); (c) Optionally keep `StateSync::sync()` for one-shot use. Remove or greatly increase `sync_interval` so there is no background timer. |
| `opencode-cli/src/tui/app.rs` — main loop calls `self.state_sync.sync_if_needed().await` every frame (~line 156) | Remove that call; trigger sync on Home enter or keybind. |

---

### 6. Provider Abstraction Tightening

**Current**

* Multiple providers treated as equals
* Provider-specific config leaks into UI logic

**Target**

* One **default path**
* Others clearly marked experimental

**Action**

* [ ] Centralize provider selection
* [ ] Prevent provider-specific logic in TUI widgets
* [ ] Simplify config UI to reflect "one happy path"

**Concrete mapping**

| Files / code | Action |
|--------------|--------|
| `opencode-cli/src/tui/screens/dialogs/provider.rs` — Provider dialog | Centralize "one default provider path" in config and CLI; mark non-default providers experimental in docs and help. Audit TUI provider dialog for provider-specific branches. |
| `opencode-cli/src/config.rs` | Same. |
| `opencode-provider/` — multiple adapters | Keep UI to a single happy path (e.g. OpenAI/Anthropic via langchain). |

---

## ⚠️ FREEZE (no new work in v0.2)

### 7. Tool Expansion

**Rule**

* No new tools unless they:
  * Improve terminal UX
  * Are directly used in chat

**Action**

* [ ] Audit existing tools
* [ ] Delete tools that:
  * Feel "platform-like"
  * Exist only to demonstrate extensibility

**Scope**

No new tools; audit per [TECH_DEBT.md](TECH_DEBT.md); remove tools that are platform-like or only for extensibility demos.

---

### 8. Provider Feature Expansion

**Freeze**

* Streaming variants
* Advanced routing
* Provider-specific optimizations

**Reason**

* Product clarity > provider cleverness

**Scope**

No streaming variants, advanced routing, or provider-specific optimizations.

---

### 9. Documentation Proliferation

**Current**

* Many overlapping status / progress docs

**Action**

* [ ] Freeze creation of new meta-docs
* [ ] Consolidate later (v0.3)

**Scope**

No new meta-docs (e.g. COMPLETION_REPORT, PROGRESS, PLAN_VS_ACTUAL); consolidate in v0.3.

---

## 🧭 Sanity Check Rule (v0.2 Gate)

Before merging any PR, ask:

> Does this make OpenCode Rust a **better terminal companion**
> or a **more powerful platform**?

If it's the second — it's out of scope.

---

## v0.2 Exit Criteria

You should be able to say, honestly:

* `opencode tui` feels intentional and calm
* Sessions are boring data
* Tools feel invisible, not impressive
* The codebase no longer looks like an Agent framework

---

## Recommended commit order

Suggested sequence so each commit is small and reviewable:

1. **Docs + CLI serve (opencode-cli + root)**  
   Deprecate `serve`: hide from help, runtime warning; optionally add banner or delete API.md / AGENT_INTEGRATION.md.  
   Crate: opencode-cli (main.rs, commands/serve.rs), root (docs).  
   Rationale: Fast, visible, no architectural risk.

2. **TUI StateSync (opencode-cli only)**  
   Replace polling with manual/event-driven refresh; remove `sync_if_needed()` from loop; add keybind or "sync on Home enter".  
   Files: opencode-cli/src/tui/app.rs, opencode-cli/src/tui/sync.rs.  
   Rationale: Single crate, clear before/after.

3. **Session / core (opencode-core)**  
   Confirm Session is data-only; optionally move SessionStateMachine to CLI or document; no new Session responsibilities.  
   Files: opencode-core/src/session.rs, session_state.rs, lib.rs (exports).  
   Rationale: Aligns with "sessions are boring data".

4. **Tool API visibility (opencode-tools + tests)**  
   Make ToolRegistry/register_all_tools internal or single entry point; update tests to use that path.  
   Files: opencode-tools/src/lib.rs, registry.rs, tools/mod.rs; tests/integration/*.rs.  
   Rationale: Prevents "framework" use without breaking CLI.

5. **Provider UX (opencode-cli TUI + config)**  
   One default path in config; mark others experimental; simplify provider dialog if needed.  
   Rationale: Polish after structure is locked.

Start with **opencode-cli + root** (serve deprecation + docs), then **opencode-cli** again (StateSync), then **opencode-core**, then **opencode-tools**. That order minimizes cross-crate changes and delivers README alignment quickly.

---

## 一个**非常重要的现实判断**

你现在的代码**不是"写坏了"**，而是：

> **写得太成功了，已经长成了你不想维护的东西**

v0.2 的目标不是"补功能"，
而是：**把已经存在的能力，压回一个终端产品该有的形状**。
