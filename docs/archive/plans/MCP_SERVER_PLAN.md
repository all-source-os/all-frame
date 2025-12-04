# AllFrame MCP Server Implementation Plan

## Overview

Implement a **Native MCP (Model Context Protocol) Server** that automatically exposes AllFrame Router handlers as LLM-callable tools. This enables Claude Desktop and other MCP clients to discover and invoke API endpoints through natural language.

## Goals

1. **Zero-Config Auto-Exposure**: Automatically convert Router handlers into MCP Tools
2. **Protocol-Agnostic**: Work with REST, GraphQL, and gRPC adapters
3. **Schema-Driven**: Leverage existing OpenAPI schemas for tool definitions
4. **Production-Ready**: <100ms latency, comprehensive test coverage
5. **Standards-Compliant**: Implement full MCP specification using official Rust SDK

## Architecture

### Components

```
┌─────────────────────────────────────────────────────────┐
│                    Claude Desktop                        │
│                  (MCP Client)                            │
└────────────────────┬────────────────────────────────────┘
                     │ JSON-RPC 2.0 over stdio
                     │
┌────────────────────▼────────────────────────────────────┐
│              AllFrame MCP Server                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Tool Discovery Engine                          │   │
│  │  - Introspect Router                            │   │
│  │  - Generate JSON Schemas                        │   │
│  │  - Map protocols to tools                       │   │
│  └─────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────┐   │
│  │  MCP Protocol Handler (rmcp SDK)                │   │
│  │  - tools/list                                   │   │
│  │  - tools/call                                   │   │
│  │  - Server metadata                              │   │
│  └─────────────────────────────────────────────────┘   │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│              AllFrame Router                             │
│  - REST Adapter                                          │
│  - GraphQL Adapter                                       │
│  - gRPC Adapter                                          │
└──────────────────────────────────────────────────────────┘
```

### Core Types

```rust
// MCP Tool generated from Router
pub struct McpTool {
    pub name: String,           // e.g., "get_user"
    pub description: String,    // From handler docs
    pub input_schema: JsonSchema, // Derived from OpenAPI
}

// MCP Server wrapping Router
pub struct McpServer {
    router: Arc<Router>,
    tools: Vec<McpTool>,
}

impl McpServer {
    pub fn new(router: Router) -> Self;
    pub async fn serve_stdio() -> Result<()>;
    pub async fn list_tools() -> Vec<McpTool>;
    pub async fn call_tool(&self, name: &str, args: Value) -> Result<Value>;
}
```

## Implementation Phases

### Phase 1: Core MCP Server (v0.5.0)
**Goal**: Basic MCP server exposing Router handlers as tools

**Features**:
- [x] Research MCP specification ✅
- [ ] Add `rmcp` dependency with `server` feature
- [ ] Implement `McpServer` struct wrapping `Router`
- [ ] Auto-generate tools from registered handlers
- [ ] Implement `tools/list` RPC method
- [ ] Implement `tools/call` RPC method with Router integration
- [ ] Stdio transport for Claude Desktop
- [ ] Basic error handling and validation

**Tests**: ~15 tests
- Server initialization
- Tool discovery from Router
- Tool listing
- Tool invocation
- Error cases (unknown tool, invalid args)
- Stdio transport

**Example Usage**:
```rust
let mut router = Router::new();
router.register("get_user", get_user_handler);
router.register("create_user", create_user_handler);

let mcp_server = McpServer::new(router);
mcp_server.serve_stdio().await?; // Start MCP server

// Claude can now call:
// - get_user(id: "123")
// - create_user(name: "Alice", email: "alice@example.com")
```

### Phase 2: Schema Generation & Validation (v0.5.1)
**Goal**: Rich tool schemas from OpenAPI specs

**Features**:
- [ ] Integration with `OpenApiGenerator`
- [ ] Convert OpenAPI schemas to JSON Schema (draft 2020-12)
- [ ] Input validation using schemas
- [ ] Output validation
- [ ] Parameter type conversion (string → int, etc.)
- [ ] Enum support
- [ ] Nested object support

**Tests**: ~12 tests
- Schema generation from OpenAPI
- Input validation (valid/invalid)
- Type coercion
- Enum validation
- Required/optional parameters

### Phase 3: Protocol Adapter Integration (v0.5.2)
**Goal**: Expose REST/GraphQL/gRPC routes as distinct tools

**Features**:
- [ ] REST adapter tool generation (GET /users → "list_users")
- [ ] GraphQL adapter tool generation (query user → "query_user")
- [ ] gRPC adapter tool generation (UserService.GetUser → "grpc_get_user")
- [ ] Tool naming conventions per protocol
- [ ] Protocol-specific parameter mapping
- [ ] Response format normalization

**Tests**: ~18 tests
- REST route → tool mapping
- GraphQL operation → tool mapping
- gRPC method → tool mapping
- Parameter extraction
- Response formatting

### Phase 4: Advanced Features (v0.5.3)
**Goal**: Production-ready MCP server

**Features**:
- [ ] HTTP transport (alternative to stdio)
- [ ] SSE (Server-Sent Events) support
- [ ] Tool metadata (tags, versioning)
- [ ] Rate limiting per tool
- [ ] Request/response logging
- [ ] Telemetry integration (OpenTelemetry)
- [ ] Configuration file support
- [ ] CLI: `allframe mcp serve`

**Tests**: ~15 tests
- HTTP transport
- SSE streaming
- Rate limiting
- Logging
- Telemetry
- Configuration

## Total Test Count: ~60 tests

## Dependencies

```toml
[dependencies]
# MCP Protocol
rmcp = { version = "0.8", features = ["server"] }
rmcp-macros = "0.8"

# JSON Schema
schemars = "0.8"  # For JSON Schema generation
jsonschema = "0.18"  # For validation

# Existing
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

## File Structure

```
crates/allframe-core/src/
├── mcp/
│   ├── mod.rs           # Public API
│   ├── server.rs        # McpServer implementation
│   ├── tools.rs         # Tool discovery and generation
│   ├── schema.rs        # Schema generation from OpenAPI
│   ├── transport.rs     # Stdio/HTTP transports
│   └── error.rs         # MCP-specific errors
├── mcp.rs               # Re-exports
└── lib.rs               # Add `pub mod mcp;`

examples/
├── mcp_server.rs        # Basic MCP server example
├── mcp_with_schemas.rs  # Schema validation example
└── mcp_multi_protocol.rs # REST+GraphQL+gRPC tools

docs/
├── guides/
│   └── MCP_SERVER.md    # Complete MCP server guide
└── phases/
    └── MCP_SERVER_COMPLETE.md  # Implementation summary
```

## Success Criteria

1. **Zero Config**: `McpServer::new(router).serve_stdio().await` just works
2. **Auto-Discovery**: All Router handlers automatically become MCP tools
3. **Schema-Driven**: Tool schemas derived from OpenAPI specs
4. **Fast**: <100ms tool invocation latency
5. **Complete**: 60+ tests, 100% coverage
6. **Standards-Compliant**: Full MCP spec compliance using `rmcp` SDK
7. **Production-Ready**: Error handling, logging, telemetry

## Claude Desktop Integration

**Config**: `~/Library/Application Support/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "allframe-api": {
      "command": "/path/to/allframe",
      "args": ["mcp", "serve"],
      "env": {
        "ALLFRAME_CONFIG": "/path/to/config.toml"
      }
    }
  }
}
```

**Usage in Claude**:
```
User: "Show me user 123"
Claude: [calls get_user tool with id: "123"]
Result: { "id": 123, "name": "Alice", ... }

User: "Create a new user named Bob with email bob@example.com"
Claude: [calls create_user tool with name: "Bob", email: "bob@example.com"]
Result: { "id": 456, "name": "Bob", ... }
```

## Next Steps After MCP Server

Once MCP server is complete, proceed to:
- **v0.6**: `allframe forge` CLI for LLM-powered code generation
- Use MCP server to enable LLMs to scaffold new projects
- Code generation using Claude API with AllFrame-specific templates

## References

- [MCP Specification](https://modelcontextprotocol.io)
- [rmcp Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [AllSource MCP Server](https://github.com/all-source-os/chronos-monorepo/tree/main/apps/mcp-server-elixir)
- [Anthropic MCP Announcement](https://www.anthropic.com/news/model-context-protocol)

---

**Status**: 📋 Planning
**Assignee**: Claude Code
**Est. Completion**: 4 phases, ~60 tests
