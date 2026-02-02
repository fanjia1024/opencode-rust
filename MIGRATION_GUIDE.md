# Migration Guide: TypeScript to Rust

This guide helps developers migrate from the TypeScript/Bun version of OpenCode to the Rust implementation.

## Key Differences

### Architecture

**TypeScript Version:**
- Uses Bun runtime
- TypeScript with Zod for validation
- SolidJS for TUI
- ai-sdk for AI providers

**Rust Version:**
- Uses Tokio async runtime
- Rust with serde + schemars for validation
- ratatui for TUI
- langchain-rust for AI providers (planned)

### Configuration

The Rust version maintains partial compatibility with the TypeScript configuration format. Most configuration files can be migrated with minimal changes.

### CLI Commands

Most CLI commands remain the same:
- `opencode tui` - Start TUI interface
- `opencode run <command>` - Run a command
- `opencode serve` - Start HTTP server

### Tools

All tools from the TypeScript version are available in Rust:
- File operations: read, write, edit, multiedit, patch
- Search: grep, glob, codesearch
- Execution: bash, batch
- Web: webfetch, websearch
- Other: question, task, todo

## Migration Steps

1. **Install Rust**: Ensure Rust 1.70+ is installed
2. **Build**: `cargo build --release`
3. **Test**: `cargo test --workspace`
4. **Run**: `cargo run --bin opencode -- tui`

## API Changes

### Tool Definition

**TypeScript:**
```typescript
export const ReadTool = Tool.define("read", {
  description: "...",
  parameters: z.object({...}),
  async execute(params, ctx) {...}
});
```

**Rust:**
```rust
#[async_trait::async_trait]
impl Tool for ReadTool {
    fn id(&self) -> &str { "read" }
    fn description(&self) -> &str { "..." }
    fn parameters(&self) -> &'static JsonSchema { ... }
    async fn execute(&self, args: Value, ctx: &ToolContext) -> Result<ToolResult> { ... }
}
```

### Agent Definition

**TypeScript:**
```typescript
export const BuildAgent = {
  name: "build",
  mode: "primary",
  async process(ctx, input) {...}
};
```

**Rust:**
```rust
#[async_trait]
impl Agent for BuildAgent {
    async fn process(&self, ctx: &Context, input: &str, session: &mut Session) -> Result<()> { ... }
    fn name(&self) -> &str { "build" }
    fn mode(&self) -> AgentMode { AgentMode::Build }
}
```

## Performance Improvements

Expected improvements in Rust version:
- **Startup time**: 60%+ faster
- **Memory usage**: 40%+ reduction
- **Tool execution**: 30%+ faster

## Compatibility

- ✅ Configuration files (partial)
- ✅ CLI commands
- ✅ Tool interfaces
- ⚠️ Session format (may need migration)
- ❌ Plugin system (to be implemented)

## Getting Help

- Check the [README.md](README.md) for basic usage
- See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines
- Review [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md) for architecture details
