# AllFrame Forge CLI - LLM-Powered Code Generation

**Date**: December 2025
**Status**: 📋 Planning
**Version**: v0.6.0

## Overview

`allframe forge` is a CLI tool for LLM-powered code generation, leveraging the latest Claude Opus 4.5 API features (November 2025) to scaffold AllFrame projects, generate handlers, and write boilerplate code.

## Latest LLM Landscape (December 2025)

### Top Models for Code Generation

Based on recent benchmarks:

1. **GPT-5** (OpenAI) - 74.9% SWE-bench Verified, 88% Aider Polyglot
2. **Gemini 2.5 Pro** (Google) - Tops WebDev Arena leaderboard
3. **Claude 3.7 Sonnet** (Anthropic) - 200K token context window
4. **Qwen3-Coder** (Open-source) - 256K+ context, 100+ languages
5. **DeepSeek R1** (Open-source) - Competitive at fraction of cost
6. **Codestral 25.01** (Mistral) - 2× faster generation

### Claude Opus 4.5 New Features (Nov 2025)

**Key API Enhancements for Code Generation**:

1. **Programmatic Tool Calling (Beta)** - Call tools from within code execution
2. **Tool Search (Beta)** - Dynamic tool discovery from large catalogs
3. **Effort Parameter (Beta)** - Tune speed vs. thoroughness per-call
4. **Client-Side Compaction** - Auto-manage context via summarization
5. **200K Token Context** - Handle large multi-file projects
6. **Improved Vision & Coding** - Step-change improvements

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│             allframe forge <command>                     │
│                  (CLI Entry Point)                       │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│              Forge Command Router                        │
│  - forge new <project>                                   │
│  - forge handler <name>                                  │
│  - forge endpoint <method> <path>                        │
│  - forge adapter <protocol>                              │
│  - forge docs                                            │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│              LLM Integration Layer                       │
│  ┌─────────────────────────────────────────────────┐   │
│  │  Claude API Client (Opus 4.5)                   │   │
│  │  - Programmatic tool calling                    │   │
│  │  - Context management (200K tokens)             │   │
│  │  - Effort parameter tuning                      │   │
│  └─────────────────────────────────────────────────┘   │
└────────────────────┬────────────────────────────────────┘
                     │
┌────────────────────▼────────────────────────────────────┐
│              Template Engine                             │
│  - AllFrame project templates                            │
│  - Handler templates (REST, GraphQL, gRPC)               │
│  - Test templates (unit, integration)                    │
│  - Documentation templates                               │
└──────────────────────────────────────────────────────────┘
```

## Core Commands

### 1. `forge new <project-name>`

**Description**: Scaffold a new AllFrame project with LLM assistance

**Interactive Flow**:
```bash
$ allframe forge new my-api

🤖 AllFrame Forge - LLM-Powered Project Generator

📋 What kind of API are you building?
   1. REST API
   2. GraphQL API
   3. gRPC Service
   4. Multi-Protocol Gateway
   → 4

📋 What features do you need?
   [✓] CQRS + Event Sourcing
   [✓] OpenAPI Documentation
   [ ] Authentication
   [✓] PostgreSQL
   [ ] Redis Cache

🔧 Generating project with Claude Opus 4.5...
   ✅ Created project structure
   ✅ Generated handlers (5)
   ✅ Generated tests (15)
   ✅ Generated documentation

🎉 Project 'my-api' created successfully!

Next steps:
   cd my-api
   cargo test
   cargo run
```

**Features**:
- Interactive Q&A powered by Claude
- Project structure generation
- Initial handler scaffolding
- Test generation
- Documentation generation
- Git initialization

### 2. `forge handler <name>`

**Description**: Generate a new handler with tests and documentation

**Example**:
```bash
$ allframe forge handler create_user

🤖 Generating handler 'create_user'...

📋 What does this handler do?
→ Creates a new user with email validation and password hashing

📋 What are the input parameters?
→ email: String (required), password: String (required), name: String (optional)

📋 What should it return?
→ User object with id, email, name, created_at

🔧 Generating with Claude Opus 4.5...
   ✅ Generated src/handlers/create_user.rs
   ✅ Generated tests/integration/create_user_test.rs
   ✅ Updated router configuration
   ✅ Generated OpenAPI documentation

✨ Handler 'create_user' ready!
```

### 3. `forge endpoint <method> <path>`

**Description**: Generate a REST endpoint with full implementation

**Example**:
```bash
$ allframe forge endpoint POST /users

🤖 Generating POST /users endpoint...
   ✅ Generated REST route
   ✅ Generated handler
   ✅ Generated request/response types
   ✅ Generated validation
   ✅ Generated tests
   ✅ Updated OpenAPI spec
```

### 4. `forge adapter <protocol>`

**Description**: Add protocol support (GraphQL, gRPC)

**Example**:
```bash
$ allframe forge adapter graphql

🤖 Adding GraphQL adapter...
   ✅ Generated GraphQL schema
   ✅ Generated resolvers
   ✅ Generated subscriptions
   ✅ Updated router configuration
   ✅ Added GraphiQL playground
```

### 5. `forge docs`

**Description**: Generate comprehensive project documentation

**Example**:
```bash
$ allframe forge docs

🤖 Generating documentation...
   ✅ Generated API reference
   ✅ Generated architecture diagrams
   ✅ Generated usage examples
   ✅ Generated deployment guide
   ✅ Generated CONTRIBUTING.md
```

## Implementation Phases

### Phase 1: Core CLI Infrastructure (v0.6.0)

**Goal**: Basic CLI with Claude API integration

**Features**:
- [ ] CLI framework (clap)
- [ ] Claude API client (Opus 4.5)
- [ ] API key management (env vars, config file)
- [ ] Context management (200K token window)
- [ ] Error handling and retries
- [ ] Progress indicators
- [ ] Configuration file support

**Tests**: ~20 tests
- CLI argument parsing
- API client initialization
- Request/response handling
- Error cases
- Configuration loading

**Dependencies**:
```toml
clap = "4.5"              # CLI framework
reqwest = "0.12"          # HTTP client
serde = "1.0"
serde_json = "1.0"
tokio = "1.0"
anyhow = "1.0"            # Error handling
indicatif = "0.17"        # Progress bars
```

### Phase 2: Project Scaffolding (v0.6.1)

**Goal**: `forge new` command with templates

**Features**:
- [ ] Interactive project wizard
- [ ] Template system
- [ ] Project structure generation
- [ ] Dependency management
- [ ] Git initialization
- [ ] LLM-powered customization

**Tests**: ~25 tests
- Template rendering
- File generation
- Directory structure
- Git initialization
- Custom templates

### Phase 3: Handler Generation (v0.6.2)

**Goal**: `forge handler` command

**Features**:
- [ ] Handler template generation
- [ ] Test generation (unit + integration)
- [ ] Router configuration updates
- [ ] OpenAPI spec updates
- [ ] Type generation from OpenAPI
- [ ] Validation code generation

**Tests**: ~30 tests
- Handler generation
- Test generation
- Router updates
- Type generation
- Validation

### Phase 4: Protocol Adapters (v0.6.3)

**Goal**: `forge endpoint` and `forge adapter` commands

**Features**:
- [ ] REST endpoint generation
- [ ] GraphQL schema generation
- [ ] gRPC proto generation
- [ ] Adapter configuration
- [ ] Multi-protocol support

**Tests**: ~25 tests

### Phase 5: Documentation Generation (v0.6.4)

**Goal**: `forge docs` command

**Features**:
- [ ] API reference generation
- [ ] Architecture diagrams
- [ ] Usage examples
- [ ] Deployment guides
- [ ] Contributing guidelines

**Tests**: ~15 tests

**Total: ~115 tests across 5 phases**

## Claude API Integration

### Authentication

```rust
// Via environment variable
export ANTHROPIC_API_KEY="sk-ant-..."

// Or via config file
~/.allframe/config.toml:
[forge]
anthropic_api_key = "sk-ant-..."
model = "claude-opus-4-5-20251120"  # Latest
```

### Request Structure

```rust
use reqwest::Client;

async fn generate_code(prompt: &str) -> Result<String> {
    let client = Client::new();

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2025-11-01")  // Latest
        .json(&json!({
            "model": "claude-opus-4-5-20251120",
            "max_tokens": 8192,
            "effort": "high",  // New: Tune reasoning depth
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "tools": tools,  // Programmatic tool calling
        }))
        .send()
        .await?;

    // Parse response...
}
```

### Prompt Engineering

**Template Structure**:
```
You are an expert Rust developer using the AllFrame framework.

Context:
- AllFrame version: 0.6.0
- Features enabled: {features}
- Project structure: {structure}

Task: {task_description}

Requirements:
1. Follow AllFrame best practices
2. Write TDD-style tests first
3. Use Clean Architecture patterns
4. Include comprehensive documentation
5. Handle errors properly

Output format:
```rust
// Generated code here
```

Please generate the code.
```

## File Structure

```
crates/allframe-cli/
├── Cargo.toml
├── src/
│   ├── main.rs           # CLI entry point
│   ├── commands/
│   │   ├── mod.rs
│   │   ├── new.rs        # forge new
│   │   ├── handler.rs    # forge handler
│   │   ├── endpoint.rs   # forge endpoint
│   │   ├── adapter.rs    # forge adapter
│   │   └── docs.rs       # forge docs
│   ├── llm/
│   │   ├── mod.rs
│   │   ├── client.rs     # Claude API client
│   │   ├── prompts.rs    # Prompt templates
│   │   └── context.rs    # Context management
│   ├── templates/
│   │   ├── mod.rs
│   │   ├── project.rs    # Project templates
│   │   ├── handler.rs    # Handler templates
│   │   └── test.rs       # Test templates
│   └── config.rs         # Configuration
└── tests/
    └── integration/      # CLI integration tests

templates/
├── project/
│   ├── rest/
│   ├── graphql/
│   └── grpc/
├── handlers/
└── tests/
```

## Success Criteria

- [ ] Zero-config: Works with just API key
- [ ] Fast: < 5s for handler generation
- [ ] Quality: Generated code passes all tests
- [ ] Complete: Includes tests + docs
- [ ] Flexible: Supports custom templates
- [ ] Interactive: Clear user feedback
- [ ] Smart: Learns from project context

## Competitive Analysis

| Tool | Language | LLM | Features | Open Source |
|------|----------|-----|----------|-------------|
| **AllFrame Forge** | Rust | Claude 4.5 | Framework-specific | ✅ |
| Aider | Python | Multi-model | Generic | ✅ |
| Claude Code | TypeScript | Claude | Generic | ❌ |
| GitHub Copilot | Multi | GPT-4 | IDE plugin | ❌ |
| Cursor | Multi | Multi-model | IDE | ❌ |

**Unique Value**: Framework-aware code generation with AllFrame-specific patterns

## Cost Estimation

**Claude Opus 4.5 Pricing** (as of Nov 2025):
- Input: $15 per million tokens
- Output: $75 per million tokens

**Estimated Costs per Operation**:
- `forge new`: ~$0.15 (10K input, 20K output)
- `forge handler`: ~$0.05 (5K input, 5K output)
- `forge endpoint`: ~$0.03 (3K input, 3K output)
- `forge adapter`: ~$0.08 (8K input, 8K output)
- `forge docs`: ~$0.10 (10K input, 10K output)

**Monthly Budget for Active Development**: ~$50-100

## Security Considerations

1. **API Key Storage**: Use secure storage (keychain, env vars)
2. **Code Review**: Always review generated code
3. **Secrets Detection**: Warn if API key in generated files
4. **Rate Limiting**: Respect Anthropic rate limits
5. **Offline Mode**: Cache templates for offline use

## Next Steps

1. ✅ Research latest LLM landscape (December 2025)
2. ✅ Design `allframe forge` architecture
3. [ ] Create Phase 1 implementation plan
4. [ ] Set up `allframe-cli` crate
5. [ ] Implement Claude API client
6. [ ] Create project templates
7. [ ] Build `forge new` command
8. [ ] Write comprehensive tests

## References

### Latest LLM Research (December 2025)
- [Best LLMs for Coding 2025](https://www.leanware.co/insights/best-llms-for-coding)
- [Top Code Generation LLMs 2025](https://www.gocodeo.com/post/top-code-generation-llms-in-2025-which-models-are-best-for-developers)
- [Comparing Top 7 LLMs for Coding](https://www.marktechpost.com/2025/11/04/comparing-the-top-7-large-language-models-llms-systems-for-coding-in-2025/)
- [23 Best LLMs December 2025](https://backlinko.com/list-of-llms)

### Claude API
- [Anthropic Release Notes December 2025](https://releasebot.io/updates/anthropic)
- [Claude Opus 4.5 API Guide](https://www.cometapi.com/how-to-use-claude-opus-4-5-api/)

### CLI Tools
- [Agentic CLI Tools Compared](https://research.aimultiple.com/agentic-cli/)
- [Simon Willison's LLM CLI](https://github.com/simonw/llm)

---

**Status**: 📋 Planning Complete
**Next Phase**: Phase 1 Implementation
**Timeline**: Q1 2026

*Generated for AllFrame v0.6.0 - December 2025*
