# OpenCode Rust

**A local AI coding assistant in your terminal.**
CLI + desktop app — **no server, no SDK, no agent framework**.

OpenCode Rust is a lightweight, local-first client for chatting with an AI agent while you design, review, and iterate on code. Use the **desktop app** (Tauri) for interactive sessions or the **CLI** for one-off questions and scripting.

---

## What this project is

- **Terminal + desktop**
  - **Desktop app** (Tauri + Vue 3) for interactive conversations
  - Scriptable **CLI** for one-off questions
- **Local-first**
  - **Desktop app**：配置与会话均保存在当前工作区目录下 `.opencode/`（`config.json`、`sessions/`），不依赖启动参数
  - **CLI**：使用全局配置目录（如 `~/.config/opencode/`）
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

# 运行桌面应用
opencode app
# 或先构建 opencode-app 后：cargo run -p opencode-app（开发模式需在 opencode-app 目录执行 npm install && npm run dev + cargo tauri dev）
```

**不启用 AI Provider 的构建**（仅核心 + CLI，无网络调用）：

```bash
cargo build --workspace --no-default-features
```

### Quick start

1. **设置 API Key**  
   `export OPENAI_API_KEY="your-key"` 或 `OPENCODE_OPENAI_API_KEY`。
2. **启动桌面应用**  
   `opencode app`（需先 `cargo build -p opencode-app`；或进入 `opencode-app` 目录执行 `npm install` 后 `cargo tauri dev` 开发运行）。
3. **基本流程**  
   在应用中新建会话、输入消息发送；或使用 CLI：`opencode run "your question"`。详见 [USAGE.md](USAGE.md)。

---

## 使用说明

### 1. 桌面应用（主要使用方式）

```bash
opencode app
```

需先构建桌面应用：`cargo build -p opencode-app`。开发时可在 `opencode-app` 目录执行 `npm install` 后运行 `cargo tauri dev`。

- **Home**：会话列表，新建会话
- **Session**：对话 + 输入框 + Agent 日志
- **Settings**：查看 Provider、切换 Agent（build / plan / general）
- **Help**：简要说明

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

### 5. 桌面应用测试方案

本地开发或排查问题时，可按以下流程验证桌面应用与 Agent：

| 步骤 | 操作 | 说明 |
|------|------|------|
| 1. 构建 | `cargo build -p opencode-cli` 与 `cargo build -p opencode-app` | Debug 构建便于看日志与 panic |
| 2. 日志级别 | `export RUST_LOG=debug` | 可选：`opencode_provider=debug,opencode_cli=debug` 只看部分模块 |
| 3. 启动方式 A | `RUST_LOG=debug opencode app` | 使用已构建的二进制，日志写入 `logs/opencode.log`，debug 时同时输出到终端 |
| 3. 启动方式 B | `cd opencode-app && npm install && RUST_LOG=debug cargo tauri dev` | Tauri 开发模式：前端热更 + 终端直接看后端/Agent 日志 |
| 4. 看日志 | 项目根目录 `logs/opencode.log` | 含 list_sessions、send_message、provider 调用、deep_agent 等 |
| 5. 单独测 Provider | `RUST_LOG=debug opencode run "Hello"` | 不经过 App UI，验证 API key 与简单生成；带工具的 Agent 需在 App 内发消息并配合日志验证 |

桌面应用配置与会话保存在**当前工作区**的 `.opencode/` 下；未指定工作区时使用启动时当前目录。完整调试步骤与常见异常对照见 [USAGE.md 中的「本地调试 App 与 Agent」](USAGE.md#本地调试-app-与-agent)。

---

## 编译脚本说明

| 脚本                       | 说明                                                                         |
| -------------------------- | ---------------------------------------------------------------------------- |
| `scripts/build.sh`         | 开发构建 `cargo build --workspace`，产出 `target/debug/opencode`             |
| `scripts/build-release.sh` | 发布构建 `cargo build --workspace --release`，产出 `target/release/opencode` |

脚本检查项目根目录与 Rust 后直接执行构建，使用 Cargo 默认配置。使用前请赋予执行权限：`chmod +x scripts/build.sh scripts/build-release.sh`。

---

## Core design principles

### CLI + desktop app

OpenCode Rust is a **local client** (desktop app + CLI), not a service. There is no stable HTTP API.

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
├── opencode-cli/       # CLI (opencode run/sessions/config/init/app)
├── opencode-app/       # Desktop app (Tauri + Vue 3); run via opencode app
├── scripts/            # Build scripts
└── tests/              # Integration tests
```

---

## Troubleshooting

- **No API key** — 设置 `OPENAI_API_KEY` 或 `OPENCODE_OPENAI_API_KEY`。
- **Feature / langchain not enabled** — 若曾使用 `--no-default-features` 构建，请用 `cargo build --workspace` 或 `cargo build --workspace --features langchain`。
- **Error initializing provider** — 检查 API key 与网络是否可达。
- **聊天无响应 / Processing 一直转** — 可设置 `RUST_LOG=debug` 后重新运行应用，根据终端或 `logs/opencode.log` 查看是 provider 初始化失败、网络超时还是响应未回写。
- **API 调用不应走代理** — 若系统设置了 `HTTP_PROXY`/`HTTPS_PROXY`，可在运行前设置 `NO_PROXY=api.openai.com`（或你的 API 主机）再启动应用。
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

If you are interested in contributing, please read **[PROJECT_SCOPE.md](PROJECT_SCOPE.md)** first. PRs that expand scope beyond CLI and the desktop app will likely be declined.
