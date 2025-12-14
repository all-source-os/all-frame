# MCP Distribution Model - Library vs Binary

**Date**: 2025-12-04
**Status**: Clarified ✅

---

## Executive Summary

`allframe-mcp` is distributed as a **library crate**, NOT as a pre-compiled binary. This document clarifies the distribution model and corrects outdated references to binary distribution.

---

## Distribution Model

### ✅ Current: Library Distribution

**Package**: `allframe-mcp` on crates.io
**Type**: Library crate
**Usage**: Embedded in user applications

```toml
# User's Cargo.toml
[dependencies]
allframe-core = "0.1"
allframe-mcp = "0.1"  # Library dependency
```

```rust
// User's application code
use allframe_core::router::Router;
use allframe_mcp::McpServer;

#[tokio::main]
async fn main() {
    let mut router = Router::new();
    router.register("get_user", get_user_handler);

    // Create MCP server from router
    let mcp = McpServer::new(router);

    // User can implement their own server logic
    // serve_stdio(), serve_http(), etc.
}
```

### ❌ Previous Misconception: Binary Distribution

**What was planned** (now outdated):
- Pre-compiled binaries: `allframe-mcp-linux-x86_64`, etc.
- GitHub Releases with platform-specific executables
- MCP Registry submission with binary installation

**Why this was incorrect**:
1. `allframe-mcp/Cargo.toml` has no `[[bin]]` section
2. `allframe-mcp/src/` only has `lib.rs`, no `main.rs`
3. MCP server is designed as a library component, not a standalone application

---

## Architecture Rationale

### Why Library Distribution?

**1. Flexibility**
- Users can embed MCP server in their own applications
- Full control over server lifecycle and configuration
- Can integrate with existing authentication, logging, metrics

**2. Zero-Bloat Guarantee**
- Library only compiled when explicitly added as dependency
- No forced binary installation
- Users choose their own runtime environment

**3. Composability**
- Can be combined with other AllFrame features
- Works with any async runtime (tokio, async-std)
- Supports custom transport layers (stdio, HTTP, WebSocket)

**4. Deployment Options**
- Docker containers with user's application
- Serverless functions (AWS Lambda, etc.)
- Standalone microservices
- Embedded in larger applications

---

## Usage Patterns

### Pattern 1: Standalone MCP Server Binary (User-Created)

Users can create their own binary wrapper:

```rust
// User creates: src/main.rs
use allframe_core::router::Router;
use allframe_mcp::McpServer;

#[tokio::main]
async fn main() {
    let mut router = Router::new();

    // Register handlers from config, database, etc.
    router.register("get_user", get_user_handler);
    router.register("create_order", create_order_handler);

    let mcp = McpServer::new(router);

    // Implement stdio transport
    // (User's responsibility)
    mcp.serve_stdio().await;
}
```

```toml
# User's Cargo.toml
[package]
name = "my-mcp-server"
version = "0.1.0"

[[bin]]
name = "my-mcp-server"
path = "src/main.rs"

[dependencies]
allframe-core = "0.1"
allframe-mcp = "0.1"
tokio = { version = "1.48", features = ["full"] }
```

### Pattern 2: Embedded in Web Application

```rust
// User's web server
use axum::{Router as AxumRouter, routing::post};
use allframe_core::router::Router;
use allframe_mcp::McpServer;

#[tokio::main]
async fn main() {
    // AllFrame router for business logic
    let mut af_router = Router::new();
    af_router.register("get_user", get_user_handler);

    // MCP server exposes AllFrame handlers
    let mcp = McpServer::new(af_router);

    // Axum web server with MCP endpoint
    let app = AxumRouter::new()
        .route("/api/users", get(get_users))
        .route("/mcp/tools", post(mcp_tools_handler))
        .route("/mcp/call", post(mcp_call_handler));

    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
```

### Pattern 3: Serverless Function

```rust
// AWS Lambda handler
use lambda_runtime::{service_fn, LambdaEvent, Error};
use allframe_core::router::Router;
use allframe_mcp::McpServer;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut router = Router::new();
    router.register("process", process_handler);

    let mcp = McpServer::new(router);

    lambda_runtime::run(service_fn(|event| handle(event, &mcp))).await
}
```

---

## Distribution Channels

### Primary: crates.io

**URL**: https://crates.io/crates/allframe-mcp
**Installation**: `cargo add allframe-mcp`
**Documentation**: https://docs.rs/allframe-mcp
**Cost**: $0 (free hosting)

### Secondary: GitHub

**Repository**: https://github.com/all-source-os/all-frame
**Path**: `crates/allframe-mcp/`
**Source Access**: Direct Git dependency

```toml
[dependencies]
allframe-mcp = { git = "https://github.com/all-source-os/all-frame" }
```

### NOT: MCP Registry with Binaries

**Status**: Not applicable
**Reason**: MCP Registry is for standalone binary servers, not libraries

If users want to submit their own binary wrappers to MCP Registry, that's their choice.

---

## Documentation Updates Required

### ✅ Completed

1. Created this document (MCP_DISTRIBUTION_MODEL.md)
2. Archived LAUNCH_CHECKLIST.md (contains outdated binary references)

### 📋 Still Needed

1. Create `crates/allframe-mcp/README.md` with:
   - Clear library usage examples
   - Pattern examples (standalone, embedded, serverless)
   - No mention of pre-compiled binaries

2. Update `docs/guides/MCP_ZERO_BLOAT_STRATEGY.md`:
   - Clarify library distribution model
   - Remove any binary distribution references

3. Update root `README.md`:
   - Add allframe-mcp installation instructions
   - Link to usage patterns

---

## CI/CD Implications

### No Binary Build Workflow Needed

**Status**: Correct ✅
**Reason**: Library crates don't need binary builds

Current CI workflows are correct:
- `compatibility-matrix.yml` - Tests library compilation
- `binary-size.yml` - Tests allframe-core binary size only
- No release workflow needed for binaries

### Future: crates.io Publishing

When ready to publish `allframe-mcp` to crates.io:

```bash
# One-time setup
cargo login <api-token>

# Publish
cd crates/allframe-mcp
cargo publish

# Verify
https://crates.io/crates/allframe-mcp
https://docs.rs/allframe-mcp
```

**GitHub Actions** (optional):
```yaml
# .github/workflows/publish.yml
name: Publish to crates.io

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable

      - name: Publish allframe-core
        run: |
          cd crates/allframe-core
          cargo publish --token ${{ secrets.CARGO_TOKEN }}

      - name: Publish allframe-mcp
        run: |
          cd crates/allframe-mcp
          cargo publish --token ${{ secrets.CARGO_TOKEN }}
```

---

## User Communication

### Correct Messaging

✅ "Install via cargo: `cargo add allframe-mcp`"
✅ "Embed MCP server in your application"
✅ "Create your own binary wrapper if needed"
✅ "Deploy as library in Docker, Lambda, etc."

### Incorrect Messaging

❌ "Download pre-compiled MCP binary"
❌ "Install from MCP Registry"
❌ "GitHub Releases contain allframe-mcp binaries"
❌ "Platform-specific executables available"

---

## Comparison: Library vs Binary Distribution

| Aspect | Library (Current ✅) | Binary (Outdated ❌) |
|--------|---------------------|---------------------|
| **Distribution** | crates.io | GitHub Releases |
| **Installation** | `cargo add` | Download executable |
| **Flexibility** | Full control | Limited to binary features |
| **Integration** | Embedded in apps | Standalone process |
| **Deployment** | Docker, Lambda, etc. | VM, container only |
| **Zero-Bloat** | 100% (opt-in) | N/A (always installed) |
| **Maintenance** | Cargo ecosystem | Manual version management |
| **Updates** | `cargo update` | Re-download binary |
| **Customization** | Full access | Config files only |

---

## Example: User Creates MCP Binary

If a user wants a standalone MCP binary server:

```bash
# User creates new project
cargo new my-allframe-mcp --bin
cd my-allframe-mcp

# Add dependencies
cargo add allframe allframe-mcp tokio --features full
```

```rust
// src/main.rs
use allframe_core::router::Router;
use allframe_mcp::McpServer;
use std::io::{stdin, stdout, BufRead, Write};

#[tokio::main]
async fn main() {
    // Build router from config
    let mut router = Router::new();
    router.register("echo", |input: String| async move { input });

    // Create MCP server
    let mcp = McpServer::new(router);

    // Stdio transport (MCP protocol)
    let stdin = stdin();
    let mut stdout = stdout();

    for line in stdin.lock().lines() {
        let line = line.unwrap();

        // Parse MCP request
        let request: serde_json::Value = serde_json::from_str(&line).unwrap();

        // Handle request
        let response = match request["method"].as_str() {
            Some("tools/list") => {
                let tools = mcp.list_tools().await;
                serde_json::json!({ "tools": tools })
            }
            Some("tools/call") => {
                let name = request["params"]["name"].as_str().unwrap();
                let args = &request["params"]["arguments"];
                let result = mcp.call_tool(name, args.clone()).await;
                serde_json::json!({ "result": result })
            }
            _ => serde_json::json!({ "error": "Unknown method" })
        };

        // Write response
        writeln!(stdout, "{}", serde_json::to_string(&response).unwrap()).unwrap();
        stdout.flush().unwrap();
    }
}
```

```bash
# Build binary
cargo build --release

# Binary available at:
# target/release/my-allframe-mcp
```

**This is the user's responsibility**, not ours.

---

## Questions & Answers

**Q: Why not provide a reference binary implementation?**
A: Different users have different needs (stdio, HTTP, WebSocket, auth, etc.). A library gives maximum flexibility.

**Q: Will this make it harder to use?**
A: No. Advanced users prefer libraries. Beginners can use examples or community binaries.

**Q: What about MCP Registry?**
A: Users can create their own binaries and submit to MCP Registry if desired.

**Q: Should we provide an example binary?**
A: Yes, as an `examples/mcp_stdio_server.rs` in the allframe-mcp crate for reference.

**Q: Will docs.rs automatically build docs?**
A: Yes, when published to crates.io, docs.rs builds documentation automatically (free).

---

## Next Steps

1. ✅ Document the library distribution model (this document)
2. 📋 Create `crates/allframe-mcp/README.md` with usage examples
3. 📋 Add example binary: `examples/mcp_stdio_server.rs`
4. 📋 Update root README.md with installation instructions
5. 📋 When ready: Publish to crates.io

---

## Related Documentation

- `/docs/archive/LAUNCH_CHECKLIST.md` - Outdated binary distribution plan
- `/docs/guides/MCP_ZERO_BLOAT_STRATEGY.md` - Zero-bloat architecture
- `/docs/phases/MCP_ZERO_BLOAT_COMPLETE.md` - Separate crate implementation
- `/crates/allframe-mcp/Cargo.toml` - Library crate configuration

---

**Status**: ✅ Distribution model clarified
**Decision**: Library distribution via crates.io (NOT binaries)
**Owner**: @all-source-os
**Last Updated**: 2025-12-04
