# Phase 6.2: Scalar Integration - Implementation Plan

**Status**: 📋 READY FOR IMPLEMENTATION
**Created**: 2025-11-27
**Target**: 2 weeks (Dual Track with Binary Size Monitoring)
**Priority**: P0

---

## Executive Summary

Integrate **Scalar** (modern OpenAPI UI) to provide beautiful, interactive REST API documentation. Scalar is lighter (<50KB vs 100KB+ for Swagger UI), faster, and has better UX with dark mode by default.

**Prerequisites**: ✅ All met (Phase 6.1 complete)
- ✅ OpenAPI 3.1 generation working
- ✅ Route metadata extraction complete
- ✅ JSON Schema generation ready
- ✅ Documentation serving infrastructure ready

---

## Goals

### Must Have (P0)

1. ✅ **Scalar UI Integration**
   - Embed Scalar in documentation endpoint
   - Serve at `/docs` route
   - Load OpenAPI spec dynamically

2. ✅ **Interactive Testing**
   - "Try It" functionality working
   - Request/response preview
   - Authentication support (API keys)

3. ✅ **Modern UX**
   - Dark mode by default
   - Mobile-friendly responsive design
   - Fast load times (<1s)

4. ✅ **Bundle Size**
   - <50KB JavaScript bundle
   - CDN delivery option
   - Optional self-hosted

### Should Have (P1)

1. **Customization**
   - Theme customization (colors, fonts)
   - Logo/branding support
   - Custom CSS injection

2. **Advanced Features**
   - Request history
   - Environment variables
   - Code generation (cURL, JS, Python)

### Nice to Have (P2)

1. **Offline Mode**
   - Download Scalar assets locally
   - Work without internet
   - Self-contained binary

---

## Technical Architecture

### 1. Scalar Integration Options

#### Option A: CDN (Recommended for MVP)
**Pros**:
- Zero bundle size impact
- Always up-to-date
- Simple implementation
**Cons**:
- Requires internet
- External dependency

**Implementation**:
```html
<script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
```

#### Option B: Self-Hosted
**Pros**:
- No external dependencies
- Works offline
- Full control
**Cons**:
- Larger binary size (~50KB)
- Need to update manually

**Recommendation**: Start with Option A (CDN), add Option B in P1

---

### 2. Route Handler Design

```rust
// src/router/scalar.rs

pub struct ScalarConfig {
    pub spec_url: String,           // URL to OpenAPI spec
    pub theme: ScalarTheme,         // Dark, Light, Auto
    pub show_sidebar: bool,
    pub layout: ScalarLayout,        // Classic, Modern
    pub custom_css: Option<String>,
}

impl Default for ScalarConfig {
    fn default() -> Self {
        Self {
            spec_url: "/docs/openapi.json".to_string(),
            theme: ScalarTheme::Dark,
            show_sidebar: true,
            layout: ScalarLayout::Modern,
            custom_css: None,
        }
    }
}

pub fn scalar_html(config: &ScalarConfig, openapi_spec: &str) -> String {
    // Generate HTML with embedded Scalar
}
```

---

### 3. Router Integration

```rust
// Add to Router methods

impl Router {
    /// Serve Scalar documentation UI
    pub fn scalar_docs(&self, config: ScalarConfig) -> String {
        let spec = self.to_openapi(&config.title, &config.version);
        let spec_json = serde_json::to_string(&spec).unwrap();

        scalar::scalar_html(&config, &spec_json)
    }

    /// Convenience method with defaults
    pub fn scalar(&self, title: &str, version: &str) -> String {
        let config = ScalarConfig {
            spec_url: "/docs/openapi.json".to_string(),
            ..Default::default()
        };
        self.scalar_docs(config)
    }
}
```

---

### 4. HTML Template

```html
<!DOCTYPE html>
<html>
<head>
    <title>{{title}} - API Documentation</title>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style>
        body { margin: 0; padding: 0; }
    </style>
</head>
<body>
    <script
        id="api-reference"
        data-url="{{spec_url}}"
        data-configuration='{{configuration}}'
    ></script>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
</body>
</html>
```

**Configuration JSON**:
```json
{
  "theme": "dark",
  "layout": "modern",
  "showSidebar": true,
  "hideModels": false,
  "hideDownloadButton": false
}
```

---

## Implementation Tasks

### Task 1: Scalar Module Setup (Day 1)

**Goal**: Create Scalar integration module

**Deliverables**:
1. Create `src/router/scalar.rs`
2. `ScalarConfig` struct
3. `ScalarTheme` enum
4. `ScalarLayout` enum
5. HTML template generation
6. 5+ tests

**Tests**:
- ✅ ScalarConfig creation
- ✅ Default theme is dark
- ✅ HTML contains Scalar script tag
- ✅ Configuration JSON is valid
- ✅ Custom CSS injection works

**Acceptance**:
- Module compiles
- All tests pass
- HTML validates

---

### Task 2: Router Integration (Day 2)

**Goal**: Add Scalar methods to Router

**Deliverables**:
1. `Router::scalar_docs()` method
2. `Router::scalar()` convenience method
3. Integration with existing OpenAPI generation
4. 5+ tests

**Tests**:
- ✅ Scalar HTML generated correctly
- ✅ OpenAPI spec embedded
- ✅ Default config works
- ✅ Custom config works
- ✅ Multiple routes documented

**Acceptance**:
- Can generate Scalar HTML
- OpenAPI spec is valid
- All tests pass

---

### Task 3: CDN Integration (Day 3)

**Goal**: Integrate Scalar via CDN

**Deliverables**:
1. CDN URL configuration
2. Version pinning (security)
3. Fallback handling
4. SRI (Subresource Integrity) hashes
5. 3+ tests

**Tests**:
- ✅ CDN URL is correct
- ✅ SRI hash present
- ✅ Version is pinned

**Acceptance**:
- CDN loads reliably
- SRI verification works
- Fallback handles offline

---

### Task 4: Interactive Features (Days 4-5)

**Goal**: Enable "Try It" functionality

**Deliverables**:
1. CORS configuration helpers
2. Authentication config (API keys, Bearer)
3. Request/response handling
4. Error display
5. 5+ tests

**Tests**:
- ✅ CORS headers configured
- ✅ Auth tokens injected
- ✅ Requests work
- ✅ Responses displayed
- ✅ Errors shown

**Acceptance**:
- Can test endpoints
- Auth works
- Errors display nicely

---

### Task 5: Theme Customization (Days 6-7)

**Goal**: Allow theme customization

**Deliverables**:
1. Theme selection (Dark, Light, Auto)
2. Custom color configuration
3. Logo/branding support
4. Custom CSS injection
5. 5+ tests

**Tests**:
- ✅ Dark theme works
- ✅ Light theme works
- ✅ Auto theme detects system
- ✅ Custom colors apply
- ✅ Custom CSS loads

**Acceptance**:
- Themes work
- Customization applies
- Branding displays

---

### Task 6: Example Integration (Days 8-9)

**Goal**: Create working example

**Deliverables**:
1. Example project in `examples/scalar_docs/`
2. Full REST API example
3. Multiple routes documented
4. "Try It" working
5. README with usage

**Tests**:
- ✅ Example compiles
- ✅ Server starts
- ✅ Docs load
- ✅ "Try It" works

**Acceptance**:
- Example runs
- Docs are beautiful
- Interactive testing works

---

### Task 7: Documentation (Day 10)

**Goal**: Document Scalar integration

**Deliverables**:
1. Usage guide in docs
2. Configuration reference
3. Migration guide (from basic docs)
4. Best practices
5. Troubleshooting

**Sections**:
- Quick start
- Configuration
- Customization
- Examples
- FAQ

**Acceptance**:
- Docs complete
- Examples tested
- Links verified

---

## Code Examples

### Basic Usage

```rust
use allframe::prelude::*;

#[tokio::main]
async fn main() {
    let mut router = Router::new();

    // Register routes
    router.get("/users", || async { "User list" });
    router.post("/users", || async { "User created" });
    router.get("/users/{id}", || async { "User details" });

    // Generate Scalar docs
    let docs_html = router.scalar("My API", "1.0.0");

    // Serve in your HTTP framework (Axum example):
    // app.route("/docs", get(|| async { Html(docs_html) }));
}
```

---

### Advanced Usage

```rust
use allframe::router::{Router, ScalarConfig, ScalarTheme};

let mut router = Router::new();
router.get("/users", || async { "Users" });

let config = ScalarConfig {
    spec_url: "/api/openapi.json".to_string(),
    theme: ScalarTheme::Auto,
    show_sidebar: true,
    custom_css: Some("body { font-family: 'Inter'; }".to_string()),
    ..Default::default()
};

let docs_html = router.scalar_docs(config);
```

---

### With Authentication

```rust
use allframe::router::{Router, ScalarConfig, AuthConfig};

let config = ScalarConfig {
    auth: Some(AuthConfig::ApiKey {
        header: "X-API-Key".to_string(),
        placeholder: "Enter your API key".to_string(),
    }),
    ..Default::default()
};

let docs_html = router.scalar_docs(config);
```

---

## Success Metrics

### Performance

| Metric | Target | Hard Limit |
|--------|--------|------------|
| Bundle size | <50KB | 75KB |
| Load time | <1s | 2s |
| Time to interactive | <1.5s | 3s |

### Quality

| Metric | Target |
|--------|--------|
| Test coverage | 100% |
| Breaking changes | 0 |
| Examples | 3+ |
| Documentation | Complete |

### User Experience

| Metric | Target |
|--------|--------|
| Dark mode | Default ✅ |
| Mobile friendly | Yes ✅ |
| "Try It" works | Yes ✅ |
| Beautiful UI | Yes ✅ |

---

## Timeline

| Days | Tasks | Deliverable |
|------|-------|-------------|
| 1 | Scalar module setup | Module compiles |
| 2 | Router integration | Methods working |
| 3 | CDN integration | Scalar loads |
| 4-5 | Interactive features | "Try It" works |
| 6-7 | Theme customization | Themes working |
| 8-9 | Example integration | Example runs |
| 10 | Documentation | Docs complete |

**Total**: 10 working days (2 weeks)

---

## Dependencies

### External

- Scalar CDN: `https://cdn.jsdelivr.net/npm/@scalar/api-reference`
- Version: Latest stable (will pin during implementation)

### Internal

- ✅ Phase 6.1 complete (OpenAPI generation)
- ✅ Router core (route metadata)
- ✅ JSON Schema (type generation)

---

## Risk Mitigation

### Risk 1: CDN Unavailable

**Mitigation**: Add fallback to self-hosted version
**Priority**: P1 (add after MVP)

### Risk 2: Scalar Breaking Changes

**Mitigation**: Pin specific version, test before upgrading
**Priority**: P0 (implement during setup)

### Risk 3: Bundle Size Exceeds Target

**Mitigation**: Use tree-shaking, lazy loading
**Priority**: P1 (optimize if needed)

### Risk 4: CORS Issues in "Try It"

**Mitigation**: Provide CORS helpers, document configuration
**Priority**: P0 (include in Task 4)

---

## Deliverables Checklist

- [ ] `src/router/scalar.rs` module
- [ ] `ScalarConfig` struct
- [ ] `Router::scalar()` method
- [ ] CDN integration with SRI
- [ ] "Try It" functionality
- [ ] Theme customization
- [ ] Example in `examples/scalar_docs/`
- [ ] Documentation
- [ ] 30+ tests (100% coverage)
- [ ] Zero breaking changes

---

## Next Steps

### Immediate (Today)

1. Create `src/router/scalar.rs`
2. Implement `ScalarConfig`
3. Write failing tests (TDD)
4. Implement HTML generation

### This Week (Days 1-5)

1. Complete Tasks 1-4
2. Get basic Scalar working
3. Enable "Try It" functionality

### Next Week (Days 6-10)

1. Complete Tasks 5-7
2. Polish examples
3. Complete documentation

---

**AllFrame. One frame. Infinite transformations.**
*Built with TDD, from day zero.* 🦀
