# README change summary (merged version)

Short diff-style summary of the README update: what was removed, added, and kept.

---

## Removed (from previous README)

* **Section titles and structure:** "Use cases", "Two ways to use", "Features", "Configuration", "TUI usage" (Home screen / Session screen / Global as separate subsections), "CLI commands" as its own section.
* **Feature list bullets:** TUI-first (syntax highlighting, virtual scrolling), Session = chat history (StateSync mention), Caching (LRU).
* **Configuration subsection:** Full paragraph on Provider config path and Session directory / StateSync.
* **TUI usage detail:** Full keybinding tables (Home: n, q; Session: Esc, ↑/↓, Enter, Backspace, q; Global: C).
* **CLI commands block:** Duplicate "After building" vs "With installed binary" code blocks (replaced by single Quick start flow).
* **Project structure descriptions:** "Core abstractions and interfaces", "AI provider (langchain-ai-rust 5.0.1)", "Tool system" (replaced by proposed descriptions).
* **Status bullets:** "Core TUI, session management, and session list (StateSync)", "langchain-ai-rust is the default...", "Session persistence and provider configuration...".
* **Version in README:** "langchain-ai-rust 5.0.1" (no version in README per plan).

---

## Added (from proposed README + plan)

* **Positioning block:** "A local AI coding assistant in your terminal" + "no server, no SDK, no agent framework" + one-line product description.
* **What this project is:** Terminal-first, Local-first, Human-in-the-loop (with sub-bullets).
* **What this project is NOT:** Five explicit "No" bullets + one-line "not the right project" warning.
* **Primary use cases:** Chat, One-off Q&A, Persistent local context (with sub-bullets).
* **How you use it:** TUI (main) and One-off CLI, with short descriptions and note on `run` → `ask`.
* **Core design principles:** CLI + TUI only, Sessions are just chat history, Built-in tools not a platform, Opinionated minimal providers (each with short explanation).
* **Getting started:** Single section containing Requirements, Installation and build, Quick start (3 steps).
* **Contributing line:** "Read PROJECT_SCOPE.md first; PRs that expand scope will likely be declined."

---

## Kept (merged from previous README)

* **Requirements:** Rust 1.70+, network for API.
* **Installation and build:** Clone repo, `cargo build --workspace`, optional `--no-default-features`.
* **Quick start:** Set API key (env vars + TUI `C`), run TUI command, basic flow (Home → session → Chat); link to USAGE.md for details.
* **TUI keybindings:** Compact line under "How you use it → TUI" (n, C, Esc, q) + link to USAGE.md for full keybindings.
* **Troubleshooting:** Three bullets (no API key, feature/langchain not enabled, error initializing provider) + link to SETUP_API_KEY.md and USAGE.md.
* **Documentation:** USAGE.md, SETUP_API_KEY.md, PROJECT_SCOPE.md, LANGCHAIN_FIXES.md.
* **Project structure tree:** Same four crates; descriptions updated to "Core data models and abstractions", "AI provider integrations", "Built-in tool set (internal)", "CLI and TUI (the product)".
* **Status:** Short bullets (core TUI, session persistence, provider config, overbuilt/simplify, scope converging) + link to PROJECT_SCOPE.md.
* **License:** MIT + link to LICENSE.

---

## Structural change

| Before | After |
|--------|--------|
| Lead with use cases + two ways to use | Lead with what this is / is NOT, then use cases, then how you use it |
| Features list | Replaced by "Core design principles" |
| Long Configuration + TUI usage sections | Folded into Getting started + one keybinding line |
| Separate CLI commands section | Folded into "How you use it" and Quick start |
| Status as implementation notes | Status as short checklist + scope note |

This file can be removed after the merge is settled; it is for review and history only.
