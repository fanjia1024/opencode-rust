# OpenCode Rust - 使用与配置指南

本指南涵盖：快速开始、API Key 设置、使用流程、功能验证与故障排除。

---

## 快速开始

### 1. 设置 API Key

见下文 [API Key 设置](#api-key-设置)。

### 2. 运行应用

```bash
# 开发运行（默认已启用 langchain）
cargo run --bin opencode -- tui
```

或安装后：

```bash
opencode tui
```

### 3. 使用流程

1. **创建会话**：在 Home 页按 `n`
2. **输入消息**：在输入框中输入问题
3. **发送消息**：按 `Enter`
4. **查看响应**：Assistant 通过配置的 Provider（如 OpenAI/Anthropic）返回回复

也可在 TUI 中按 `C` 打开配置对话框，选择 Provider、填写 API Key 与可选 Base URL，无需事先设置环境变量。

---

## API Key 设置

### 常见错误

若看到：**"Error: No API key configured. Please set OPENAI_API_KEY environment variable."**，表示需要配置 API Key。

### 方法 1：临时设置（当前终端会话）

```bash
export OPENAI_API_KEY="your-api-key-here"
# 或
export OPENCODE_OPENAI_API_KEY="your-api-key-here"

cargo run --bin opencode -- tui
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
- **Anthropic**：在对应控制台创建 API key，并在 TUI 配置中选择 Anthropic 与对应 key。

### 安全提示

- 不要将 API key 提交到 Git 或分享给他人。
- 建议定期轮换 key，并使用环境变量而非硬编码。

---

## 功能验证

### 验证 Provider 是否工作

1. 运行应用后创建一个新会话（按 `n`）。
2. 输入测试消息，例如："Hello, how are you?"
3. 按 Enter 发送。
4. 应看到 Assistant 的真实响应（而非占位符或报错）。

### 常见错误处理

| 错误 | 原因 | 解决 |
|------|------|------|
| **No API key configured** | 未设置环境变量或未在 TUI 中配置 | 设置 `OPENAI_API_KEY` 或 `OPENCODE_OPENAI_API_KEY`，或在 TUI 按 `C` 配置 |
| **Error initializing provider** | API key 无效或网络问题 | 检查 key 是否正确、网络是否可达 |
| **langchain-rust feature not enabled** | 使用 `--no-default-features` 构建 | 使用 `cargo build --workspace` 或 `cargo build --workspace --features langchain` 后运行 |

---

## 故障排除

### 没有 Assistant 响应 / Processing 一直转

1. **检查 API Key**：`env | grep OPENAI` 或 `env | grep OPENCODE`；或在 TUI 按 `C` 确认配置已保存。
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
NO_PROXY=api.openai.com cargo run --bin opencode -- tui
```

将 `api.openai.com` 替换为你的 API 主机。

---

## 技术说明（简要）

- **Provider**：默认集成 langchain-ai-rust，支持 OpenAI、Anthropic；消息经 Agent 与 Provider 处理后再回写 UI。
- **会话**：会话列表与历史保存在项目下 `.opencode/sessions`，配置在标准配置目录（如 `~/.config/opencode/` 或 `~/Library/Application Support/opencode/`）。

更多架构与贡献边界见 [README.md](README.md)、[PROJECT_SCOPE.md](PROJECT_SCOPE.md)。
