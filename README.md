# OpenCode Rust

**A local AI coding assistant in your terminal.**
CLI + TUI only — **no server, no SDK, no agent framework**.

OpenCode Rust is a lightweight, local-first terminal client for chatting with an AI agent while you design, review, and iterate on code.

---

## What this project is

- **A terminal-first product**
  - Interactive **TUI** for ongoing conversations
  - Scriptable **CLI** for one-off questions
- **Local-first**
  - Sessions live in your project under `.opencode/sessions`
  - Config stored in standard config dir (e.g. `~/.config/opencode/` or `~/Library/Application Support/opencode/`)
  - No background services, no daemon
- **Human-in-the-loop**
  - You drive the conversation
  - The agent assists — it does not autonomously run workflows

---

## What this project is NOT

- No HTTP server / SaaS backend
- No Agent SDK or framework
- No workflow engine
- No long-running autonomous agents
- No plugin marketplace or dynamic tool loading

If you are looking for an agent platform or orchestration framework, this is **not** the right project.

---

## Primary use cases

- **Chat with an AI agent in the terminal**
  - Discuss architecture and design
  - Review code and diffs
  - Refine implementation ideas
- **One-off Q&A from the shell**
  - Ask a single question from scripts or CI
- **Persistent local context**
  - Pick up conversations where you left off
  - Sessions are plain data, stored on disk

---

## Getting started

### Requirements

- **Rust** 1.70+ (install from [rustup.rs](https://rustup.rs))
- Network access for API calls when using the AI provider

### Build & install

**方式一：使用编译脚本（推荐）**

```bash
# 克隆仓库
git clone https://github.com/fanjia1024/opencode-rust.git
cd opencode-rust

# 开发构建（含 langchain provider）
./scripts/build.sh

# 或发布构建（优化体积与性能）
./scripts/build-release.sh
```

构建产物：`target/debug/opencode` 或 `target/release/opencode`。安装到 PATH 可执行：

```bash
cp target/release/opencode ~/.local/bin/   # 或 /usr/local/bin
```

**方式二：直接使用 Cargo**

```bash
git clone https://github.com/fanjia1024/opencode-rust.git
cd opencode-rust

# 全工作区构建（默认启用 langchain）
cargo build --workspace

# 运行 TUI
cargo run --bin opencode -- tui
```

**不启用 AI Provider 的构建**（仅核心 + CLI，无网络调用）：

```bash
cargo build --workspace --no-default-features
```

### Quick start

1. **设置 API Key**  
   `export OPENAI_API_KEY="your-key"` 或 `OPENCODE_OPENAI_API_KEY`；也可在 TUI 中按 `C` 打开配置对话框设置 Provider 与 API Key。
2. **启动 TUI**  
   `cargo run --bin opencode -- tui`，或安装后执行 `opencode tui`。
3. **基本流程**  
   Home 页按 `n` 创建会话；进入会话后输入内容按 Enter 发送。详见 [USAGE.md](USAGE.md)。

---

## 使用说明

### 1. TUI（主要使用方式）

```bash
opencode tui
```

- **Home**：会话列表 / 创建新会话
- **Chat**：对话 + 输入框
- **配置**：按 `C` 打开 Provider 与 API Key 配置

**快捷键：**

| 按键      | 说明                                       |
| --------- | ------------------------------------------ |
| `q`       | 退出应用                                   |
| `n`       | 新建会话（在 Home 页）                     |
| `C`       | 打开 Provider / API Key 配置（全局）       |
| `Esc`     | 返回上一级 / 关闭对话框；在会话内返回 Home |
| `Tab`     | 在配置对话框内切换输入项                   |
| `Enter`   | 确认 / 提交；在会话内发送消息              |
| `↑` / `↓` | 在列表中导航；在会话内滚动消息             |

**配置对话框**（按 `C` 后）：选择 Provider（OpenAI / Anthropic）、输入 API Key、可选 Base URL，按 Enter 保存，Esc 取消。配置写入本地 config，无需重启即生效。

### 2. 一次性 CLI 问答

```bash
opencode run "Explain this Rust lifetime error"
```

- 单次提问，无交互会话，适合脚本或 CI。

### 3. 会话管理（CLI）

```bash
opencode sessions list              # 列出所有会话
opencode sessions show <session_id> # 查看指定会话
opencode sessions delete <session_id> # 删除指定会话
```

### 4. 配置管理（CLI）

```bash
opencode config show   # 显示当前配置
opencode config reset  # 恢复默认配置
```

---

## 编译脚本说明

| 脚本                       | 说明                                                                         |
| -------------------------- | ---------------------------------------------------------------------------- |
| `scripts/build.sh`         | 开发构建 `cargo build --workspace`，产出 `target/debug/opencode`             |
| `scripts/build-release.sh` | 发布构建 `cargo build --workspace --release`，产出 `target/release/opencode` |

脚本检查项目根目录与 Rust 后直接执行构建，使用 Cargo 默认配置。使用前请赋予执行权限：`chmod +x scripts/build.sh scripts/build-release.sh`。

---

## Core design principles

### CLI + TUI only

OpenCode Rust is a **terminal application**, not a service. There is no stable HTTP API.

> The `serve` subcommand is experimental and planned for removal.

### Sessions are just chat history

A session is: an ID, a list of messages, timestamps. Sessions do **not** manage tools, providers, or execution logic.

### Built-in tools, not a platform

Tools exist only to improve the terminal coding experience (e.g. file read/write, search, patch). There is no public tool registry, no dynamic loading, no plugin API.

### Opinionated, minimal providers

- Default provider integration: **langchain-ai-rust** (OpenAI / Anthropic)
- Other providers may exist but are **explicitly experimental**

---

## Project structure

```
opencode-rust/
├── opencode-core/      # Core data models and abstractions
├── opencode-provider/  # AI provider integrations
├── opencode-tools/     # Built-in tool set (internal)
├── opencode-cli/       # CLI and TUI (the product)
├── scripts/            # Build scripts
└── tests/              # Integration tests
```

---

## Troubleshooting

- **No API key** — 设置 `OPENAI_API_KEY` 或 `OPENCODE_OPENAI_API_KEY`，或在 TUI 中按 `C` 配置。
- **Feature / langchain not enabled** — 若曾使用 `--no-default-features` 构建，请用 `cargo build --workspace` 或 `cargo build --workspace --features langchain`。
- **Error initializing provider** — 检查 API key 与网络是否可达。
- **聊天无响应 / Processing 一直转** — 可设置 `RUST_LOG=debug` 或 `RUST_LOG=opencode_cli=debug,opencode_provider=debug` 后重新运行 `cargo run --bin opencode -- tui`，根据日志查看是 provider 初始化失败、网络超时还是响应未回写到 UI。
- **API 调用不应走代理** — 若系统设置了 `HTTP_PROXY`/`HTTPS_PROXY`，请求会经代理发出。若希望直连你配置的 API 地址（如 `api.openai.com` 或自定义 base URL 如 `mgallery.haier.net`），可在运行前设置 `NO_PROXY` 包含该主机，例如：`NO_PROXY=mgallery.haier.net cargo run --bin opencode -- tui` 或 `NO_PROXY=api.openai.com cargo run --bin opencode -- tui`。
- **下载超时 / 镜像慢** — 若全局 `~/.cargo/config.toml` 里配置了 crates 镜像（如 rsproxy），直接运行 `cargo build` / `cargo run` 会使用该镜像，可能超时。任选其一：**临时不用镜像** — 编辑 `~/.cargo/config.toml`，注释或删除 `[source.crates-io]` 下的 `replace-with = "rsproxy-sparse"` 以及对应的 `[source.rsproxy-sparse]` 整块，保存后再执行 cargo；**使用代理** — 配置 `HTTP_PROXY` / `HTTPS_PROXY` 后再执行 cargo。

详见 [USAGE.md](USAGE.md)（含 API Key 配置与故障排除）。

---

## Documentation

常用文档（根目录）：

- [USAGE.md](USAGE.md) — 使用与配置（快速开始、API Key、故障排除）
- [PROJECT_SCOPE.md](PROJECT_SCOPE.md) — 项目范围与贡献边界
- [CONTRIBUTING.md](CONTRIBUTING.md) — 贡献流程与开发设置
- [AGENTS.md](AGENTS.md) — Cursor/Agent 规则与项目约定
- [API.md](API.md) — 内部 API 概览
- [ROADMAP.md](ROADMAP.md) — v0.2 / v0.3 路线图
- [TECH_DEBT.md](TECH_DEBT.md) — 技术债务与计划中的移除项
- [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) — TypeScript → Rust 迁移

API Key 配置详见 [USAGE.md](USAGE.md)；若需简短入口可参考 [SETUP_API_KEY.md](SETUP_API_KEY.md)。

完整文档索引见 [docs/README.md](docs/README.md)。历史实现与完成报告见 [docs/archive/](docs/archive/README.md)。

---

## License

MIT License. See [LICENSE](LICENSE) in this repository.

---

If you are interested in contributing, please read **[PROJECT_SCOPE.md](PROJECT_SCOPE.md)** first. PRs that expand scope beyond CLI/TUI will likely be declined.
