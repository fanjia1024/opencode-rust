# API Key 设置指南

## 问题

看到错误："Error: No API key configured. Please set OPENAI_API_KEY environment variable."

## 解决方案

### 方法 1：临时设置（当前终端会话）

```bash
export OPENAI_API_KEY="your-api-key-here"
cargo run --bin opencode --features langchain -- tui
```

### 方法 2：永久设置（推荐）

#### macOS/Linux

添加到 `~/.zshrc` 或 `~/.bashrc`：

```bash
echo 'export OPENAI_API_KEY="your-api-key-here"' >> ~/.zshrc
source ~/.zshrc
```

#### 验证设置

```bash
echo $OPENAI_API_KEY
```

### 方法 3：使用 .env 文件（如果支持）

创建 `.env` 文件在项目根目录：

```
OPENAI_API_KEY=your-api-key-here
```

## 获取 API Key

1. 访问 https://platform.openai.com/api-keys
2. 登录或注册账户
3. 创建新的 API key
4. 复制 key（只显示一次，请妥善保存）

## 验证

设置完成后，重新运行：

```bash
cargo run --bin opencode --features langchain -- tui
```

应该不再看到 "No API key configured" 错误。

## 安全提示

⚠️ **重要**：
- 不要将 API key 提交到 Git 仓库
- 不要分享你的 API key
- 定期轮换 API key
- 使用环境变量而不是硬编码
