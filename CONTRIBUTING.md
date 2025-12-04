# Contributing to AllFrame

Thank you for your interest in contributing to AllFrame! This document provides guidelines and workflows for contributing to the project.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Dependency Management](#dependency-management)
- [Testing Requirements](#testing-requirements)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Style Guide](#style-guide)

---

## Code of Conduct

AllFrame follows a professional, inclusive, and respectful community standard. We expect all contributors to:

- Be respectful and constructive in discussions
- Focus on technical merit and project goals
- Welcome newcomers and help them learn
- Give and receive feedback gracefully

---

## Getting Started

### Prerequisites

- **Rust**: 1.86.0 or higher (see `rust-version` in `Cargo.toml`)
- **Git**: For version control
- **cargo-make** (optional): For task automation

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/all-source-os/all-frame.git
cd all-frame

# Build the project
cargo build --all

# Run tests
cargo test --all

# Optional: Install cargo-make
cargo install cargo-make
cargo make test
```

---

## Development Workflow

### 1. Create a Feature Branch

```bash
# For new features
git checkout -b feature/your-feature-name

# For bug fixes
git checkout -b fix/issue-description

# For documentation
git checkout -b docs/what-you-are-documenting
```

### 2. Make Your Changes

- Write tests first (TDD approach)
- Implement the feature/fix
- Ensure all tests pass
- Update documentation

### 3. Commit Your Changes

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
# Feature
git commit -m "feat: add GraphQL subscription support"

# Bug fix
git commit -m "fix: resolve router panic on empty path"

# Documentation
git commit -m "docs: add examples for DI container usage"

# Dependency update
git commit -m "deps: update tokio to 1.48"

# Breaking change
git commit -m "feat!: change router API for better ergonomics

BREAKING CHANGE: Router::new() now requires a RouterConfig parameter"
```

**Commit Message Format:**

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks
- `deps`: Dependency updates
- `ci`: CI/CD changes

### 4. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a PR on GitHub.

---

## Dependency Management

**IMPORTANT**: AllFrame has strict dependency management policies. Please read this section carefully before updating dependencies.

### Policy

We use **pinned dependency versions** for stability and reproducibility. See `docs/DEPENDENCY_MANAGEMENT.md` for complete details.

### Quick Rules

✅ **DO:**
- Pin to exact minor versions: `tokio = "1.48"`
- Document why specific versions are chosen
- Create upgrade plan before updating
- Test thoroughly after updates
- Update `docs/upgrade-plans/` with your changes

❌ **DON'T:**
- Use version ranges: `tokio = "1"` ❌
- Use wildcards: `tokio = "*"` ❌
- Update dependencies during feature development
- Skip testing after updates

### Updating Dependencies

If you need to update a dependency:

1. **Read the policy**: `docs/DEPENDENCY_MANAGEMENT.md`

2. **Create an upgrade plan**:
   ```bash
   cp docs/upgrade-plans/TEMPLATE.md docs/upgrade-plans/$(date +%Y-%m-%d)-your-update.md
   # Fill out the template
   ```

3. **Update Cargo.toml**:
   ```toml
   # Add update date and reason
   # === Your Category ===
   # Updated: YYYY-MM-DD - Reason for update
   your-crate = "X.Y"
   ```

4. **Update lockfile**:
   ```bash
   cargo update -p your-crate
   ```

5. **Test thoroughly**:
   ```bash
   cargo test --all
   cargo clippy --all
   cargo build -p allframe-core --all-features
   ```

6. **Document**:
   - Complete upgrade plan
   - Create summary doc if major update
   - Update comments in Cargo.toml

7. **Separate PR**:
   - Dependency updates must be in separate PRs from feature work
   - Use commit type `deps:`

**Example PR:**
```
Title: deps: Update tokio to 1.48 for performance improvements

Body:
Updates tokio from 1.35 to 1.48 for performance improvements and bug fixes.

See docs/upgrade-plans/2025-12-04-december-updates.md for details.

- All 291+ tests passing
- No breaking changes
- CI verified
```

---

## Testing Requirements

### Test-Driven Development (TDD)

AllFrame is built using TDD. **All new features must have tests written first.**

### Test Hierarchy

1. **Unit Tests**: Test individual components
2. **Integration Tests**: Test component interactions
3. **Doc Tests**: Ensure documentation examples work
4. **Property Tests**: Use proptest for invariants

### Running Tests

```bash
# All tests
cargo test --all

# Specific package
cargo test -p allframe-core

# Specific feature
cargo test -p allframe-core --features="router"

# With output
cargo test -- --nocapture

# Single test
cargo test test_router_creation
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_name() {
        // Arrange
        let input = setup_test_data();

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected);
    }
}
```

### Test Coverage

We aim for:
- **Core functionality**: 90%+ coverage
- **Feature modules**: 80%+ coverage
- **Examples**: All examples must compile and run

---

## Documentation

### Types of Documentation

1. **Code Documentation**: Inline comments and doc comments
2. **API Documentation**: `///` doc comments for public APIs
3. **User Guides**: Markdown docs in `docs/`
4. **Examples**: Runnable examples in `examples/`

### Writing Doc Comments

```rust
/// Brief description of the function.
///
/// More detailed explanation of what the function does,
/// how it works, and when to use it.
///
/// # Arguments
///
/// * `param1` - Description of parameter
/// * `param2` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Examples
///
/// ```
/// use allframe_core::Router;
///
/// let router = Router::new();
/// router.register("hello", || async { "Hello!" });
/// ```
///
/// # Panics
///
/// When the function might panic (if applicable)
///
/// # Errors
///
/// When the function might return an error (if applicable)
pub fn function_name(param1: Type1, param2: Type2) -> ReturnType {
    // Implementation
}
```

### Documentation Files

- `README.md`: Project overview and quick start
- `docs/`: Detailed guides and technical docs
- `CHANGELOG.md`: Version history and changes
- `CONTRIBUTING.md`: This file

---

## Pull Request Process

### Before Submitting

- [ ] All tests pass (`cargo test --all`)
- [ ] No clippy warnings (`cargo clippy --all -- -D warnings`)
- [ ] Code is formatted (`cargo fmt --all`)
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (if applicable)
- [ ] Commit messages follow conventional commits
- [ ] PR description explains changes clearly

### PR Template

```markdown
## Description
Brief description of what this PR does

## Motivation
Why is this change needed?

## Changes
- List of specific changes made
- Be detailed but concise

## Testing
How was this tested?
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Checklist
- [ ] Tests pass locally
- [ ] Documentation updated
- [ ] CHANGELOG.md updated
- [ ] No new warnings
```

### Review Process

1. **Automated Checks**: CI must pass
   - All tests pass
   - Clippy checks pass
   - Format check passes
   - Compatibility matrix passes

2. **Code Review**: At least one approval required
   - Maintainer reviews code quality
   - Architecture review for larger changes
   - Security review if applicable

3. **Merge**: Squash and merge (typically)
   - Keep history clean
   - Meaningful commit messages

---

## Style Guide

### Rust Code Style

We follow standard Rust conventions:

- **Formatting**: Use `rustfmt` (run `cargo fmt`)
- **Linting**: Use `clippy` (run `cargo clippy`)
- **Naming**:
  - `snake_case` for functions and variables
  - `PascalCase` for types and traits
  - `SCREAMING_SNAKE_CASE` for constants

### Code Organization

```rust
// Imports grouped and sorted
use std::collections::HashMap;
use std::sync::Arc;

use external_crate::Something;

use crate::internal::Module;

// Constants
const MAX_RETRIES: u32 = 3;

// Type definitions
pub struct MyStruct {
    field1: Type1,
    field2: Type2,
}

// Implementations
impl MyStruct {
    pub fn new() -> Self {
        // ...
    }

    pub fn method(&self) -> Result<(), Error> {
        // ...
    }
}

// Tests at the end
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // ...
    }
}
```

### Error Handling

```rust
// Use thiserror for error types
#[derive(Debug, thiserror::Error)]
pub enum MyError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

// Return Result types
pub fn fallible_operation() -> Result<Output, MyError> {
    // ...
}
```

### Async Code

```rust
// Use async-trait for trait async methods
#[async_trait]
pub trait MyTrait {
    async fn async_method(&self) -> Result<(), Error>;
}

// Prefer async/await over manual futures
pub async fn my_function() {
    let result = some_async_call().await;
    // ...
}
```

---

## Feature Development Checklist

When adding a new feature:

- [ ] **Write tests first** (TDD)
- [ ] Implement the feature
- [ ] Add documentation
- [ ] Add examples (if user-facing)
- [ ] Update README.md (if needed)
- [ ] Update CHANGELOG.md
- [ ] Ensure backward compatibility (or document breaking changes)
- [ ] Performance considerations addressed
- [ ] Security implications reviewed
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Code formatted

---

## Bug Fix Checklist

When fixing a bug:

- [ ] **Write failing test** that reproduces the bug
- [ ] Implement the fix
- [ ] Ensure test now passes
- [ ] Add regression test (if not covered)
- [ ] Update documentation (if bug was in docs)
- [ ] Update CHANGELOG.md
- [ ] All tests pass

---

## Release Process

(For maintainers)

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`
4. Push tag: `git push origin vX.Y.Z`
5. CI publishes to crates.io
6. Create GitHub release with changelog

---

## Getting Help

### Resources

- **Documentation**: `docs/`
- **Examples**: `examples/`
- **Issues**: [GitHub Issues](https://github.com/all-source-os/all-frame/issues)
- **Discussions**: [GitHub Discussions](https://github.com/all-source-os/all-frame/discussions)

### Questions

- Check existing documentation first
- Search existing issues
- Create a new issue with "question" label
- Be specific and provide context

---

## Recognition

Contributors are recognized in:
- Git commit history
- CHANGELOG.md (for significant contributions)
- README.md (for major contributors)

---

## License

By contributing to AllFrame, you agree that your contributions will be licensed under the same license as the project (MIT OR Apache-2.0).

---

Thank you for contributing to AllFrame! 🚀
