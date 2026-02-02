# OpenCode Rust API Documentation

## Core Modules

### opencode-core

Core abstractions and interfaces for the OpenCode system.

#### Agent

```rust
pub trait Agent: Send + Sync {
    async fn process(&self, ctx: &Context, input: &str, session: &mut Session) -> Result<()>;
    fn name(&self) -> &str;
    fn mode(&self) -> AgentMode;
}
```

#### Tool

```rust
#[async_trait]
pub trait Tool: Send + Sync {
    fn id(&self) -> &str;
    fn description(&self) -> &str;
    fn parameters(&self) -> &'static JsonSchema;
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult>;
}
```

#### Provider

```rust
#[async_trait]
pub trait Provider: Send + Sync {
    async fn generate(&self, request: GenerateRequest) -> Result<GenerateResponse>;
    async fn stream(&self, request: GenerateRequest) -> Result<Box<dyn Stream<Item = Result<Chunk>> + Send + Unpin>>;
    fn models(&self) -> &[ModelInfo];
}
```

#### Session

```rust
pub struct Session {
    pub id: String,
    pub project_id: String,
    pub directory: String,
    pub title: String,
    pub messages: VecDeque<Message>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

#### Permission

```rust
pub struct PermissionManager {
    rules: HashMap<String, PermissionAction>,
    matchers: Vec<(GlobMatcher, PermissionAction)>,
}
```

## Provider Module

### OpenAI Provider

```rust
let provider = OpenAIProvider::new(
    api_key,
    Some(base_url)
)?;
```

### Anthropic Provider

```rust
let provider = AnthropicProvider::new(
    api_key,
    Some(base_url)
)?;
```

## Tools Module

### Tool Registry

```rust
let mut registry = ToolRegistry::new();
register_all_tools(&mut registry);

let result = registry.execute("read", args, &ctx).await?;
```

### Available Tools

- `read`: Read file contents
- `write`: Write file contents
- `edit`: Edit file by replacing lines
- `multiedit`: Apply multiple edits
- `patch`: Apply unified diff patch
- `ls`: List directory contents
- `grep`: Search for patterns
- `glob`: Match file patterns
- `bash`: Execute shell commands
- `batch`: Execute multiple tools
- `webfetch`: Fetch URL content
- `websearch`: Search the web
- `codesearch`: Search code context
- `question`: Ask user questions
- `task`: Manage tasks
- `todo`: Manage todo items

## CLI Module

### Commands

```rust
opencode tui          // Start TUI
opencode run <cmd>    // Run command
opencode serve        // Start HTTP server
```

### TUI Components

- `App`: Main application
- `HomeScreen`: Home interface
- `SessionScreen`: Session interface
- `Dialog`: Dialog system
- `Theme`: Theme management
- `Keybind`: Keyboard shortcuts

## Examples

### Using a Tool

```rust
use opencode_tools::ToolRegistry;
use opencode_tools::tools::register_all_tools;

let mut registry = ToolRegistry::new();
register_all_tools(&mut registry);

let ctx = ToolContext {
    session_id: "session-1".to_string(),
    message_id: "msg-1".to_string(),
    agent: "build".to_string(),
    call_id: None,
};

let args = serde_json::json!({
    "path": "file.txt"
});

let result = registry.execute("read", args, &ctx).await?;
println!("{}", result.output);
```

### Using an Agent

```rust
use opencode_core::agent::{BuildAgent, Context};
use opencode_core::session::Session;

let agent = BuildAgent::new();
let ctx = Context {
    session_id: "session-1".to_string(),
    message_id: "msg-1".to_string(),
    agent: "build".to_string(),
};
let mut session = Session::new(...);

agent.process(&ctx, "Hello", &mut session).await?;
```

### Using a Provider

```rust
use opencode_provider::{OpenAIProvider, Provider};

let provider = OpenAIProvider::new(api_key, None)?;
let request = GenerateRequest {
    messages: vec![...],
    model: Some("gpt-4o-mini".to_string()),
    temperature: Some(0.7),
    max_tokens: Some(4096),
};

let response = provider.generate(request).await?;
println!("{}", response.content);
```
