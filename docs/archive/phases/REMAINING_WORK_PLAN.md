# AllFrame Remaining Work Plan

**Date**: 2025-12-01
**Status**: Planning Complete
**Priority**: Sequential execution (6.3 → 6.4 → 6.5 → v0.5 → v0.6)

---

## Overview

After completing **Track A (Scalar Integration)** and **Track B (Binary Size Monitoring)**, the following major features remain from the original vision:

| Phase | Feature | Status | Priority | Est. Duration |
|-------|---------|--------|----------|---------------|
| **6.3** | GraphQL Documentation (GraphiQL) | 📋 Next | P0 | 2 weeks |
| **6.4** | gRPC Documentation (Explorer) | 📋 Planned | P1 | 2 weeks |
| **6.5** | Contract Testing | 📋 Planned | P1 | 2 weeks |
| **v0.5** | Native MCP Server | 📋 Planned | P2 | 3 weeks |
| **v0.6** | LLM Code Generation (forge) | 📋 Planned | P2 | 4 weeks |

**Total Estimated Time**: ~13 weeks (Q1 2026)

---

## Phase 6.3: GraphQL Documentation (GraphiQL)

### Goal
Beautiful, interactive GraphQL API documentation with GraphiQL playground

### Status
📋 **READY TO START** (All prerequisites complete)

### Prerequisites
- ✅ Router Core (Phase 6.1) complete
- ✅ Scalar Integration (Phase 6.2) complete - provides pattern to follow
- ✅ OpenAPI generation working - similar approach for GraphQL schema

### Deliverables

#### 1. GraphQL Schema Generation
```rust
pub struct GraphQLSchemaGenerator {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

impl GraphQLSchemaGenerator {
    pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self;
    pub fn with_description(self, desc: impl Into<String>) -> Self;
    pub fn generate(&self, router: &Router) -> String; // SDL format
}
```

#### 2. GraphiQL UI Integration
```rust
pub struct GraphiQLConfig {
    pub endpoint_url: String,
    pub subscription_url: Option<String>,
    pub theme: GraphiQLTheme, // Light, Dark
    pub enable_explorer: bool,
    pub enable_history: bool,
    pub headers: HashMap<String, String>,
}

pub fn graphiql_html(
    config: &GraphiQLConfig,
    title: &str,
    schema_sdl: &str,
) -> String;
```

#### 3. Schema Introspection API
- Automatic introspection query support
- Type resolution
- Field documentation
- Deprecation notices

#### 4. Interactive Features
- Query auto-completion
- Schema documentation sidebar
- Query history persistence
- Subscription testing UI
- Variable editor with JSON validation

### Implementation Plan

#### Week 1: Core Infrastructure
**Days 1-2**: Schema Generation
- Create `GraphQLSchemaGenerator` struct
- Implement SDL (Schema Definition Language) generation
- Route metadata → GraphQL types mapping
- Write 15-20 tests for schema generation

**Days 3-4**: GraphiQL Configuration
- Create `GraphiQLConfig` struct with builder pattern
- Add theme support (Light/Dark)
- Implement HTML generation with embedded schema
- CDN integration (following Scalar pattern)

**Day 5**: Documentation & Testing
- Write comprehensive documentation guide
- Add integration tests
- Create example project

#### Week 2: Advanced Features & Polish
**Days 6-7**: Interactive Features
- Query auto-completion support
- Schema explorer sidebar
- Variable editor
- Query history

**Days 8-9**: Subscription Support
- WebSocket configuration
- Subscription testing UI
- Connection status indicators

**Day 10**: Final Polish
- Example project (`examples/graphql_docs.rs`)
- Update README
- Update PROJECT_STATUS.md
- Announcement document

### Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| Schema Generation | 100% automatic | No manual schema writing |
| GraphiQL Integration | Working playground | Interactive queries work |
| Subscription Support | WebSocket enabled | Real-time updates work |
| Bundle Size | <100KB | CDN-hosted GraphiQL |
| Test Coverage | 100% | TDD enforced |
| Documentation | Comprehensive | 400+ lines guide |

### Dependencies
- `async-graphql` crate (already in dependencies)
- GraphiQL CDN (jsdelivr or unpkg)
- WebSocket support (already have via tokio)

### Risks & Mitigation
1. **Risk**: GraphQL schema complexity
   - **Mitigation**: Start with simple types, add complexity incrementally

2. **Risk**: Subscription testing complexity
   - **Mitigation**: Optional feature, can defer if needed

3. **Risk**: Integration with existing async-graphql
   - **Mitigation**: Build on top of existing patterns, don't replace

---

## Phase 6.4: gRPC Documentation (Explorer)

### Goal
First-class gRPC documentation with interactive service explorer

### Status
📋 **PLANNED** (Starts after 6.3)

### Deliverables
1. gRPC reflection API implementation
2. Service explorer UI (custom or integrate existing tool)
3. Proto file generation from Rust types
4. Request builder with syntax highlighting
5. Stream testing (server/client/bidirectional)

### Timeline
2 weeks (starts after Phase 6.3)

### Key Technologies
- `tonic` reflection API (already using tonic)
- Custom UI or integrate BloomRPC/grpcurl UI
- Proto generation from Rust structs

---

## Phase 6.5: Contract Testing

### Goal
Effortless API contract testing for all protocols

### Status
📋 **PLANNED** (Starts after 6.4)

### Deliverables
1. **Contract Test Generators**
   ```rust
   #[test]
   async fn test_create_user_contract() {
       let contract = Router::generate_contract_test("POST /users");
       contract.run().await.expect("Contract test failed");
   }
   ```

2. **Schema Validation**
   - Request/response validation against specs
   - OpenAPI schema validation
   - GraphQL query validation
   - gRPC message validation

3. **Mock Server Generation**
   - Generate mock servers from specs
   - Realistic fake data generation
   - Error scenario testing

4. **Test Reporting**
   - Coverage reports (% of endpoints tested)
   - Schema drift detection
   - Breaking change detection
   - CI/CD integration

### Timeline
2 weeks (starts after Phase 6.4)

### Key Technologies
- `schemars` for JSON Schema validation
- `fake` crate for test data generation
- Custom mock server infrastructure

---

## v0.5: Native MCP Server

### Goal
LLMs can discover and call your API as tools via Model Context Protocol

### Status
📋 **PLANNED** (Starts after Phase 6.5)

### What is MCP?
Model Context Protocol - Anthropic's standard for LLM-tool integration
- Allows Claude/GPT to call your API endpoints as tools
- Automatic function discovery from OpenAPI
- Type-safe parameter passing
- Result handling

### Deliverables

#### 1. MCP Server Implementation
```rust
pub struct McpServer {
    router: Router,
    tools: Vec<Tool>,
}

impl McpServer {
    pub fn from_router(router: Router) -> Self;
    pub async fn serve(&self, addr: SocketAddr) -> Result<()>;
}
```

#### 2. Automatic Tool Generation
- Convert OpenAPI endpoints → MCP tools
- Parameter schemas from JSON Schema
- Description from route metadata
- Example generation from doc comments

#### 3. Tool Execution
- Request validation
- Async execution
- Error handling
- Result serialization

#### 4. Configuration
```rust
pub struct McpConfig {
    pub server_name: String,
    pub version: String,
    pub description: String,
    pub max_tokens: usize,
    pub timeout: Duration,
}
```

### Implementation Plan

#### Week 1: Core MCP Protocol
- MCP protocol implementation (JSON-RPC over stdio/HTTP)
- Tool discovery endpoint
- Tool execution endpoint
- Schema conversion (OpenAPI → MCP format)

#### Week 2: Integration & Features
- Router integration
- Configuration options
- Error handling
- Logging/tracing

#### Week 3: Polish & Documentation
- Example projects
- MCP client testing
- Documentation guide
- Claude Desktop integration guide

### Success Metrics
- LLMs can discover all API endpoints
- LLMs can call endpoints with correct parameters
- Error messages are LLM-friendly
- Documentation includes integration guide

### References
- [MCP Specification](https://modelcontextprotocol.io)
- [Anthropic MCP Documentation](https://docs.anthropic.com/mcp)

---

## v0.6: LLM-Powered Code Generation (`allframe forge`)

### Goal
AI-powered code generation for AllFrame projects

### Status
📋 **PLANNED** (Starts after v0.5)

### Deliverables

#### 1. `allframe forge` CLI
```bash
# Generate a new endpoint
allframe forge endpoint "Create user with email validation"

# Generate CQRS command
allframe forge command "UpdateUserProfile with avatar upload"

# Generate projection
allframe forge projection "UserAnalytics from UserEvents"

# Generate saga
allframe forge saga "PaymentProcessing with refund compensation"
```

#### 2. Code Generation Engine
```rust
pub struct ForgeEngine {
    llm_client: LlmClient,
    templates: TemplateRegistry,
    validators: Vec<Validator>,
}

impl ForgeEngine {
    pub async fn generate_endpoint(&self, prompt: &str) -> Result<GeneratedCode>;
    pub async fn generate_command(&self, prompt: &str) -> Result<GeneratedCode>;
    pub async fn generate_projection(&self, prompt: &str) -> Result<GeneratedCode>;
    pub async fn generate_saga(&self, prompt: &str) -> Result<GeneratedCode>;
}
```

#### 3. Features
- **Prompt → Code**: Natural language → Rust code
- **Test Generation**: Automatic test generation (TDD)
- **Documentation**: Auto-generated doc comments
- **Validation**: Compile checks before committing
- **Interactive Mode**: Iterative refinement
- **Template System**: Customizable patterns

#### 4. LLM Integration
- Support multiple providers (Anthropic, OpenAI, local models)
- Streaming responses
- Token budget management
- Context window optimization

### Implementation Plan

#### Week 1: CLI & Infrastructure
- Forge CLI scaffolding
- Configuration system
- Template engine
- File management

#### Week 2: LLM Integration
- LLM client implementation
- Prompt engineering
- Response parsing
- Error recovery

#### Week 3: Code Generators
- Endpoint generator
- Command generator
- Projection generator
- Saga generator

#### Week 4: Polish & Release
- Test generation
- Documentation generation
- Interactive mode
- Examples & guides

### Success Metrics
- 90%+ code generation accuracy
- Generated code compiles
- Generated tests pass
- 80%+ time savings for common tasks

### Key Technologies
- Claude API (Anthropic)
- OpenAI API (GPT-4)
- Local LLM support (llama.cpp)
- `handlebars` for templates
- `tree-sitter` for code parsing

---

## Execution Strategy

### Priority Order
1. **Phase 6.3** (GraphQL) - Completes API docs triad
2. **Phase 6.4** (gRPC) - Completes API docs triad
3. **Phase 6.5** (Contract Testing) - Critical for quality
4. **v0.5** (MCP Server) - High value, moderate complexity
5. **v0.6** (Forge CLI) - High complexity, highest value

### Parallel Work Opportunities
- Documentation can be written in parallel with implementation
- Examples can be developed alongside features
- Testing infrastructure improvements can happen continuously

### Risk Management
- Each phase is independently valuable (can ship incrementally)
- No blocking dependencies between phases (except 6.3 → 6.4 → 6.5)
- Can adjust timeline based on complexity discovered

### Quality Gates
- ✅ 100% test coverage (TDD enforced)
- ✅ Comprehensive documentation
- ✅ Working examples for each feature
- ✅ Zero breaking changes
- ✅ Performance targets met

---

## Timeline Summary

| Quarter | Phases | Key Milestones |
|---------|--------|----------------|
| **Q1 2026** | 6.3, 6.4, 6.5 | Complete API docs + contract testing |
| **Q2 2026** | v0.5 | MCP server integration |
| **Q3 2026** | v0.6 | Forge CLI release |
| **Q4 2026** | 1.0 | Production-ready 1.0 release |

---

## Next Immediate Step

🎯 **START: Phase 6.3 (GraphQL Documentation)**

**First Task**: Create failing test for GraphQL schema generation
**Estimated Start Date**: 2025-12-01
**Estimated Completion**: 2025-12-15 (2 weeks)

---

**AllFrame. One frame. Infinite transformations.**
*Building the future of Rust web frameworks, one test at a time.* 🦀
