# TUI Provider 和 API Key 配置实现完成

## 实现总结

已成功实现完整的 TUI Provider 和 API Key 配置功能，用户可以通过界面设置和保存配置，配置立即生效。

## 实现的功能

### 1. ProviderDialog 组件增强 ✅

**文件**: `opencode-cli/src/tui/screens/dialogs/provider.rs`

- ✅ 输入状态管理（SelectProvider, InputApiKey, InputBaseUrl）
- ✅ API key 掩码显示（非编辑时显示为 `•`）
- ✅ Tab 键切换输入字段
- ✅ 字符输入和删除功能
- ✅ 输入验证（API key 不能为空）
- ✅ 错误提示显示
- ✅ 保存和取消操作

**关键特性**:
- 编辑 API key 时显示明文，非编辑时显示掩码
- 高亮显示当前编辑的字段
- 清晰的帮助提示

### 2. DialogState 枚举扩展 ✅

**文件**: `opencode-cli/src/tui/state.rs`

- ✅ 添加 `Provider` 变体到 `DialogState` 枚举

### 3. 配置管理模块 ✅

**文件**: `opencode-cli/src/config.rs` (新建)

- ✅ 封装 `opencode_core::config::Config` 的使用
- ✅ `load_config()` 和 `save_config()` 方法
- ✅ `get_provider_config()` 和 `get_default_provider()` 方法
- ✅ `set_provider_config()` 方法保存配置
- ✅ 配置文件读写错误处理

### 4. App 集成 ✅

**文件**: `opencode-cli/src/tui/app.rs`

- ✅ 添加 `config` 字段（`RefCell<AppConfig>`）
- ✅ 在 `App::new()` 中加载配置
- ✅ 添加 `provider_dialog` 字段（`RefCell<Option<ProviderDialog>>`）
- ✅ 在 `ui()` 方法中渲染 Provider 对话框
- ✅ 在 `handle_key()` 中添加快捷键 `C` 打开配置对话框
- ✅ 实现对话框的键盘事件处理
- ✅ `open_provider_dialog()` 方法
- ✅ `save_provider_config()` 方法
- ✅ 保存配置后更新内存中的配置

### 5. 消息处理逻辑更新 ✅

**文件**: `opencode-cli/src/tui/app.rs` - `process_message_async` 方法

- ✅ 优先从配置读取 API key，环境变量作为后备
- ✅ 从配置读取 provider 类型
- ✅ 根据配置的 provider 类型初始化相应的 adapter
- ✅ 支持 OpenAI 和 Anthropic provider

## 使用方法

### 打开配置对话框

在任何屏幕按 `C` 键打开 Provider 配置对话框。

### 配置步骤

1. **选择 Provider**（默认选中）:
   - 使用 `Up`/`Down` 键选择 provider（openai 或 anthropic）

2. **输入 API Key**:
   - 按 `Tab` 切换到 API Key 字段
   - 输入 API key（编辑时显示明文）

3. **输入 Base URL**（可选）:
   - 按 `Tab` 切换到 Base URL 字段
   - 输入自定义 base URL（如果需要）

4. **保存配置**:
   - 按 `Enter` 保存配置
   - 配置会立即生效，无需重启

5. **取消**:
   - 按 `Esc` 取消并关闭对话框

### 快捷键

- `C`: 打开 Provider 配置对话框（全局快捷键）
- `Tab`: 在对话框内切换输入字段
- `Enter`: 保存配置并关闭对话框
- `Esc`: 取消并关闭对话框
- `Up`/`Down`: 在 provider 选择列表中导航

## 配置存储

### 存储位置

- **macOS**: `~/Library/Application Support/opencode/config.json`
- **Linux**: `~/.config/opencode/config.json`
- **Windows**: `%APPDATA%/opencode/config.json`

### 配置格式

```json
{
  "providers": [
    {
      "id": "default",
      "provider_type": "openai",
      "api_key": "sk-...",
      "base_url": null
    }
  ],
  "agents": [],
  "storage": {
    "session_dir": "...",
    "config_dir": "..."
  }
}
```

## 配置优先级

1. **TUI 配置文件中保存的配置**（最高优先级）
2. 环境变量 `OPENAI_API_KEY` 或 `OPENCODE_OPENAI_API_KEY`
3. 如果都没有，显示错误提示："Press 'C' to configure provider and API key."

## 立即生效机制

- 保存配置后，立即更新 `App` 中的 `config` 字段
- 后续的 `process_message_async` 调用使用新配置
- 无需重启应用

## 测试验证

### 测试步骤

1. **打开配置对话框**:
   - 运行应用: `cargo run --bin opencode --features langchain -- tui`
   - 按 `C` 键

2. **输入配置**:
   - 选择 provider（openai）
   - 切换到 API Key 字段（Tab）
   - 输入 API key
   - 按 Enter 保存

3. **验证立即生效**:
   - 创建新会话（按 `n`）
   - 输入消息并发送
   - 应该看到 Assistant 的真实响应（使用配置的 API key）

4. **验证持久化**:
   - 退出应用
   - 重新启动
   - 配置应该仍然存在

5. **验证配置优先级**:
   - 设置环境变量 `OPENAI_API_KEY`
   - 在配置对话框中设置不同的 API key
   - 应该使用配置文件中的 key（更高优先级）

## 技术细节

### 数据流

```
用户按 C 键
  ↓
open_provider_dialog()
  ↓
显示 ProviderDialog（从配置加载初始值）
  ↓
用户输入配置
  ↓
按 Enter 保存
  ↓
save_provider_config()
  ↓
保存到配置文件 (config.json)
  ↓
更新 App.config
  ↓
下次 process_message_async 使用新配置
```

### 关键实现点

1. **RefCell 用于内部可变性**: `config` 和 `provider_dialog` 使用 `RefCell` 允许在 `&self` 上下文中修改

2. **配置克隆**: `AppConfig` 实现 `Clone`，允许在 async 任务中使用

3. **错误处理**: 配置加载失败时使用默认配置，不阻塞应用启动

4. **掩码显示**: API key 在非编辑状态下显示为掩码，保护隐私

## 文件修改清单

1. ✅ `opencode-cli/src/tui/screens/dialogs/provider.rs` - 完善输入处理和掩码显示
2. ✅ `opencode-cli/src/tui/state.rs` - 扩展 `DialogState` 枚举
3. ✅ `opencode-cli/src/config.rs` - 新建配置管理模块
4. ✅ `opencode-cli/src/tui/app.rs` - 集成配置对话框和修改消息处理逻辑
5. ✅ `opencode-cli/src/main.rs` - 添加 config 模块

## 后续改进建议

1. **多 Provider 支持**: 支持配置多个 provider 并切换
2. **配置验证**: 在保存前验证 API key 格式
3. **配置导入/导出**: 支持导入/导出配置文件
4. **配置加密**: 对 API key 进行加密存储
5. **Provider 测试**: 保存配置前测试 provider 连接

## 完成状态

✅ 所有计划的功能已实现
✅ 编译通过
✅ 配置立即生效
✅ 配置持久化
✅ 用户友好的界面

现在用户可以通过 TUI 界面方便地配置 Provider 和 API Key，无需手动编辑配置文件或设置环境变量。
