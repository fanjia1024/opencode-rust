# Technical debt and future removals

This list summarizes items that are **kept for now but planned for removal or simplification**. Use it when prioritizing convergence work. See [PROJECT_SCOPE.md](PROJECT_SCOPE.md) for full scope and convergence checklist.

---

## Planned removal or freeze

| Item | Action | Notes |
|------|--------|--------|
| **`serve` subcommand** | Remove or long-term freeze | Experimental; planned removal. Prefer TUI or `run` for usage. |
| **StateSync 5s polling** | Replace | Replace with event-driven or manual refresh for session list. |

---

## Tools: candidates for cutting or merging

**Principle:** Prefer tools that only make sense in TUI/CLI; mark or remove the rest.

| Category | Tools | Recommendation |
|----------|--------|----------------|
| **Keep** | read, write, grep, ls, bash (restricted), codesearch, edit, patch | Strong terminal use; keep. |
| **Consider cutting or merging** | batch, multiedit, task, todo, question | Weaker link to “terminal chat”; evaluate for removal or merge. |
| **Experimental / placeholder** | webfetch, websearch | If placeholder-only, mark experimental or remove. |

**Registry:** Do not expose publicly; no dynamic loading; built-in list only.

---

## Session model

- Session = **chat history + minimal state** only.
- Orchestration (calling Provider, dispatching Tools) lives in the CLI runner / TUI controller, not in Session.
- **Later:** Reduce Session’s role to a pure data structure + load/save; no lifecycle or I/O scheduling in Session.

---

## Reference

- Full convergence checklist: [PROJECT_SCOPE.md](PROJECT_SCOPE.md) → “Convergence checklist”.
- Roadmap for sequencing: [ROADMAP.md](ROADMAP.md).
