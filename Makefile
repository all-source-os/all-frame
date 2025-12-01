# AllFrame - Quality Gates & Development Tasks
#
# This Makefile provides consistent commands for linting, formatting,
# testing, and building AllFrame.

.PHONY: help lint lint-check lint-sort format format-sort test test-all build clean install-tools

# Default target
help:
	@echo "AllFrame Development Commands"
	@echo ""
	@echo "Quality Gates:"
	@echo "  make lint          - Check formatting and run clippy (CI mode)"
	@echo "  make lint-check    - Check code formatting without modifying files"
	@echo "  make lint-sort     - Check if Cargo.toml dependencies are sorted"
	@echo "  make format        - Format all Rust code"
	@echo "  make format-sort   - Sort Cargo.toml dependencies"
	@echo ""
	@echo "Testing:"
	@echo "  make test          - Run all tests with main features"
	@echo "  make test-all      - Run all tests with all features"
	@echo "  make test-minimal  - Run tests with no features"
	@echo ""
	@echo "Building:"
	@echo "  make build         - Build in debug mode"
	@echo "  make build-release - Build in release mode"
	@echo "  make clean         - Clean build artifacts"
	@echo ""
	@echo "Tools:"
	@echo "  make install-tools - Install required development tools"
	@echo ""
	@echo "CI/CD:"
	@echo "  make ci            - Run all CI checks (lint + test + build)"

# Quality Gates

# Run both format check and clippy with -D warnings (fail on warnings)
# Note: Using main features instead of --all-features to avoid allsource dependency issues
lint: lint-check
	@echo "Running clippy..."
	@cd crates/allframe-core && cargo clippy --features "di,openapi,router,cqrs,otel,mcp" -- -D warnings

# Check formatting without modifying files
lint-check:
	@echo "Checking code formatting..."
	@cd crates/allframe-core && cargo fmt --check

# Check if Cargo.toml dependencies are sorted
lint-sort:
	@echo "Checking Cargo.toml sorting..."
	@cd crates/allframe-core && cargo sort --check

# Format all Rust code
format:
	@echo "Formatting Rust code..."
	@cd crates/allframe-core && cargo fmt

# Sort Cargo.toml dependencies
format-sort:
	@echo "Sorting Cargo.toml dependencies..."
	@cd crates/allframe-core && cargo sort -w

# Testing

# Run tests with main features (di, openapi, router, cqrs)
test:
	@echo "Running tests with main features..."
	@cd crates/allframe-core && cargo test --features "di,openapi,router,cqrs"

# Run tests with all features
test-all:
	@echo "Running tests with all features..."
	@cd crates/allframe-core && cargo test --all-features

# Run tests with minimal features
test-minimal:
	@echo "Running tests with no features..."
	@cd crates/allframe-core && cargo test --no-default-features

# Building

# Build in debug mode
build:
	@echo "Building in debug mode..."
	@cd crates/allframe-core && cargo build

# Build in release mode
build-release:
	@echo "Building in release mode..."
	@cd crates/allframe-core && cargo build --release

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cd crates/allframe-core && cargo clean

# Tools

# Install required development tools
install-tools:
	@echo "Installing development tools..."
	@cargo install cargo-sort --quiet || true
	@cargo install cargo-bloat --quiet || true
	@echo "Tools installed!"

# CI/CD

# Run all CI checks
ci: lint lint-sort test build
	@echo ""
	@echo "✅ All CI checks passed!"
