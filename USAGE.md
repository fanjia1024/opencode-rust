# OpenCode Rust - 使用指南

## Agent 集成完成 ✅

langchain-rust provider 集成已完成，现在 Assistant 可以返回真实的 AI 响应！

## 快速开始

### 1. 设置 API Key

```bash
export OPENAI_API_KEY="your-openai-api-key-here"
```

或者：

```bash
export OPENCODE_OPENAI_API_KEY="your-openai-api-key-here"
```

### 2. 运行应用

```bash
# 使用 langchain-rust 功能
cargo run --bin opencode --features langchain -- tui
```

### 3. 使用流程

1. **创建会话**：按 `n` 键
2. **输入消息**：在输入框中输入你的问题
3. **发送消息**：按 `Enter` 键
4. **查看响应**：Assistant 会通过 langchain-rust 调用 OpenAI API 并显示响应

## 功能验证

### 验证 Provider 是否工作

1. 运行应用后，创建一个新会话
2. 输入测试消息，例如："Hello, how are you?"
3. 按 Enter 发送
4. 应该看到 Assistant 的真实响应（而不是占位符）

### 常见错误处理

#### 错误：No API key configured
- **原因**：未设置环境变量
- **解决**：设置 `OPENAI_API_KEY` 环境变量

#### 错误：Error initializing provider
- **原因**：Provider 初始化失败（可能是 API key 无效或网络问题）
- **解决**：检查 API key 是否正确，检查网络连接

#### 错误：langchain-rust feature not enabled
- **原因**：未使用 `--features langchain` 编译
- **解决**：使用 `cargo run --bin opencode --features langchain -- tui` 运行

## 技术实现

### Provider 流程

1. **用户输入** → TUI 接收
2. **消息发送** → 通过 mpsc 通道发送到异步任务
3. **Provider 初始化** → 从环境变量读取 API key，创建 LangChainAdapter
4. **Agent 处理** → AgentManager 使用 Provider 处理消息
5. **响应返回** → 通过 mpsc 通道返回响应到 UI
6. **UI 更新** → 显示 Assistant 响应

### 当前实现状态

- ✅ Langchain-Rust 集成
- ✅ OpenAI Provider 支持
- ✅ 异步消息处理
- ✅ UI 响应更新
- ⚠️ 会话持久化（待实现）
- ⚠️ 工具集成（待实现）
- ⚠️ 流式响应（待实现）

## 下一步改进

1. **会话持久化**：保存会话历史到文件
2. **工具集成**：集成实际的工具（read, write, grep 等）
3. **流式响应**：实现流式输出，提供更好的用户体验
4. **Provider 选择**：支持切换不同的 AI provider
5. **错误重试**：添加自动重试机制

## 测试建议

1. **基本对话**：测试简单的问答
2. **多轮对话**：测试上下文理解
3. **错误处理**：测试 API key 错误、网络错误等
4. **性能测试**：测试响应时间和资源使用
