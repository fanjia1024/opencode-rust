# OpenCode Rust - 使用与配置指南

本指南涵盖：快速开始、API Key 设置、使用流程、功能验证与故障排除。

---

## 快速开始

### 1. 设置 API Key

见下文 [API Key 设置](#api-key-设置)。

### 2. 运行应用

**桌面应用（推荐，交互式会话）：**

```bash
# 先构建桌面应用
cargo build -p opencode-app

# 启动（需与 opencode 同目录或 PATH 中有 opencode-app）
opencode app
```

开发模式：进入 `opencode-app` 目录，执行 `npm install` 后运行 `cargo tauri dev`。

**CLI 一次性问答：**

```bash
opencode run "your question"
```

### 3. 使用流程（桌面应用）

1. **创建会话**：在 Home 页点击「New session」
2. **输入消息**：在会话页输入框输入问题
3. **发送**：点击 Send 或按 Enter
4. **查看响应**：消息区与 Agent 日志会更新

### 桌面应用配置（工作区）

桌面应用中，**Provider、模型和 Agent 仅通过 Settings 界面配置**，不能通过启动参数或环境变量指定。保存后，配置会写入**当前工作区目录**下的 `.opencode` 目录：

- 配置文件：`<工作区>/.opencode/config.json`（含 Provider、默认 Agent 等）
- 会话目录：`<工作区>/.opencode/sessions/`

工作区默认为应用启动时的当前目录；若曾通过应用内设置过工作区路径，下次启动会恢复该路径。在 Settings 页可查看当前「Config and sessions for」对应的路径。

**CLI 命令**（如 `opencode run`）仍使用全局配置目录（如 `~/.config/opencode`）和环境变量，与桌面应用的工作区配置相互独立。

---

## API Key 设置

### 常见错误

若看到：**"Error: No API key configured. Please set OPENAI_API_KEY environment variable."**，表示需要配置 API Key。

### 方法 1：临时设置（当前终端会话）

```bash
export OPENAI_API_KEY="your-api-key-here"
# 或
export OPENCODE_OPENAI_API_KEY="your-api-key-here"

opencode app
```

### 方法 2：永久设置（推荐）

**macOS / Linux**：写入 `~/.zshrc` 或 `~/.bashrc`：

```bash
echo 'export OPENAI_API_KEY="your-api-key-here"' >> ~/.zshrc
source ~/.zshrc
```

**验证**：

```bash
echo $OPENAI_API_KEY
```

### 方法 3：使用 .env 文件（若项目支持）

在项目根目录创建 `.env`：

```
OPENAI_API_KEY=your-api-key-here
```

### 获取 API Key

- **OpenAI**：访问 https://platform.openai.com/api-keys，登录后创建并复制 key（仅显示一次，请妥善保存）。
- **Anthropic**：在对应控制台创建 API key；桌面应用 Settings 或环境变量中配置。

### 安全提示

- 不要将 API key 提交到 Git 或分享给他人。
- 建议定期轮换 key，并使用环境变量而非硬编码。

---

## 功能验证

### 验证 Provider 是否工作

1. 运行桌面应用（`opencode app`）或开发模式（`cd opencode-app && npm install && cargo tauri dev`）。
2. 在 Home 页点击「New session」创建会话，进入会话页。
3. 在输入框输入测试消息（如 "Hello, how are you?"），点击 Send 或按 Enter。
4. 应看到 Assistant 的回复与（可选）Agent 日志更新。

### 常见错误处理

| 错误 | 原因 | 解决 |
|------|------|------|
| **No API key configured** | 未设置环境变量 | 设置 `OPENAI_API_KEY` 或 `OPENCODE_OPENAI_API_KEY` |
| **Error initializing provider** | API key 无效或网络问题 | 检查 key 是否正确、网络是否可达 |
| **langchain-rust feature not enabled** | 使用 `--no-default-features` 构建 | 使用 `cargo build --workspace` 或 `cargo build --workspace --features langchain` 后运行 |

---

## 故障排除

### 没有 Assistant 响应 / Processing 一直转

1. **检查 API Key**：`env | grep OPENAI` 或 `env | grep OPENCODE`；桌面应用可在 Settings 查看已配置的 Provider。
2. **确认启用 Provider**：默认构建已包含 langchain；若曾用 `--no-default-features`，需重新 `cargo build --workspace`。
3. **查看日志**：设置 `RUST_LOG=debug` 或 `RUST_LOG=opencode_cli=debug,opencode_provider=debug` 后重新运行，根据终端或 `logs/opencode.log` 判断是初始化失败、网络超时还是响应未回写。

### 编译错误

```bash
cargo clean
cargo build --workspace
```

若使用 langchain：`cargo build --workspace --features langchain`。

### API 调用不希望走代理

若系统设置了 `HTTP_PROXY`/`HTTPS_PROXY` 但希望直连 API（如 `api.openai.com` 或自定义 base URL），可在运行前设置：

```bash
NO_PROXY=api.openai.com opencode app
```

将 `api.openai.com` 替换为你的 API 主机。

---

## 本地调试 App 与 Agent

在本地排查桌面应用命令与 Agent 执行异常时，可按以下步骤进行。

### 1. 构建（Debug）

```bash
# 在项目根目录
cargo build -p opencode-cli
cargo build -p opencode-app
```

Debug 构建会保留日志与 panic 信息，便于排查。

### 2. 开启日志级别

运行前设置环境变量，将后端与 provider 的日志打到终端（开发模式）和日志文件：

```bash
export RUST_LOG=debug
# 或只关心部分模块：
# export RUST_LOG=opencode_provider=debug,opencode_cli=debug
```

桌面应用会将日志同时写入 **`logs/opencode.log`**（与 CLI 共用），开发模式下还会输出到终端 stderr。

### 3. 运行桌面应用

**方式 A：通过 CLI 启动已构建的二进制**

```bash
# 需先有 target/debug/opencode 和 target/debug/opencode-app
RUST_LOG=debug opencode app
```

**方式 B：Tauri 开发模式（前端热更 + 看终端日志）**

```bash
cd opencode-app
npm install
RUST_LOG=debug cargo tauri dev
```

会启动 Vite 前端与 Tauri 窗口，终端中可看到后端与 agent 的日志。

### 4. 查看日志

- **文件**：项目根目录下 `logs/opencode.log`，包含 list_sessions、send_message、provider 调用、deep_agent 等。
- **终端**：以 `cargo tauri dev` 或 `RUST_LOG=debug opencode app` 运行时，debug 构建会同时把日志打到 stderr。

### 5. 单独测试 Agent（不经过 App UI）

不打开桌面应用时，可用 CLI 一次性问答验证 provider 与简单生成是否正常：

```bash
export OPENAI_API_KEY="your-key"
RUST_LOG=debug opencode run "Hello, reply in one sentence"
```

该命令走 CLI 的 `run` 流程（直接调用 provider 生成），与桌面应用内 `send_message` 使用的 **message_processor（含 deep_agent、工具调用）** 不是同一条路径。若需验证 **带工具的 agent 与 send_message**，必须在桌面应用中发起对话，并配合上述日志查看。

### 6. 常见异常对照

| 现象 | 建议排查 |
|------|----------|
| 点击 Send 无响应 | 看 `logs/opencode.log` 是否有 `process_message_async started`、provider 初始化错误或 `run_deep_agent_turn` 报错。 |
| 报错 No API key | 确认环境变量 `OPENAI_API_KEY` 或 `OPENCODE_OPENAI_API_KEY` 已设置；或将来在 App Settings 中配置。 |
| 报错 Unsupported provider type | 检查默认 provider 的 `provider_type`（如 openai / anthropic），与代码中 match 分支一致。 |
| Max iterations reached | 长对话触发了 deep_agent 的迭代上限。可在项目 `.opencode/config.json`（或应用配置）中设置 `"max_agent_iterations": 25` 等提高上限；或压缩历史。 |

---

## 技术说明（简要）

- **Provider**：默认集成 langchain-ai-rust，支持 OpenAI、Anthropic；消息经 Agent 与 Provider 处理后再回写 UI。
- **会话**：会话列表与历史保存在项目下 `.opencode/sessions`，配置在标准配置目录（如 `~/.config/opencode/` 或 `~/Library/Application Support/opencode/`）。

更多架构与贡献边界见 [README.md](README.md)、[PROJECT_SCOPE.md](PROJECT_SCOPE.md)。
