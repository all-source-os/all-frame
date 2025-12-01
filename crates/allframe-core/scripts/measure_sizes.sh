#!/usr/bin/env bash
# Measure binary sizes for all feature configurations

set -e

echo "# Binary Size Baseline Measurements"
echo ""
echo "Date: $(date)"
echo "Rust version: $(rustc --version)"
echo ""

# Minimal (no features)
echo "## 1. Minimal Configuration (no features)"
echo ""
cargo bloat --release --example minimal --no-default-features --crates -n 10 2>/dev/null || true
SIZE_MINIMAL=$(stat -f%z ../target/release/examples/minimal 2>/dev/null || stat -c%s ../target/release/examples/minimal 2>/dev/null || echo "N/A")
echo ""
echo "Binary size: $SIZE_MINIMAL bytes ($(echo "scale=2; $SIZE_MINIMAL/1024/1024" | bc) MB)"
echo ""

# Default features (di, openapi, router)
echo "## 2. Default Features (di, openapi, router)"
echo ""
cargo bloat --release --example default_features --features="di,openapi,router" --crates -n 10 2>/dev/null || true
SIZE_DEFAULT=$(stat -f%z ../target/release/examples/default_features 2>/dev/null || stat -c%s ../target/release/examples/default_features 2>/dev/null || echo "N/A")
echo ""
echo "Binary size: $SIZE_DEFAULT bytes ($(echo "scale=2; $SIZE_DEFAULT/1024/1024" | bc) MB)"
echo ""

# CQRS features
echo "## 3. CQRS Configuration (di, openapi, cqrs)"
echo ""
cargo bloat --release --example all_features --features="di,openapi,cqrs" --crates -n 10 2>/dev/null || true
SIZE_CQRS=$(stat -f%z ../target/release/examples/all_features 2>/dev/null || stat -c%s ../target/release/examples/all_features 2>/dev/null || echo "N/A")
echo ""
echo "Binary size: $SIZE_CQRS bytes ($(echo "scale=2; $SIZE_CQRS/1024/1024" | bc) MB)"
echo ""

# All features
echo "## 4. All Features"
echo ""
cargo bloat --release --example all_features --all-features --crates -n 10 2>/dev/null || true
SIZE_ALL=$(stat -f%z ../target/release/examples/all_features 2>/dev/null || stat -c%s ../target/release/examples/all_features 2>/dev/null || echo "N/A")
echo ""
echo "Binary size: $SIZE_ALL bytes ($(echo "scale=2; $SIZE_ALL/1024/1024" | bc) MB)"
echo ""

# Summary table
echo "## Summary"
echo ""
echo "| Configuration | Size (bytes) | Size (MB) | vs Target | Status |"
echo "|---------------|--------------|-----------|-----------|--------|"
echo "| Minimal | $SIZE_MINIMAL | $(echo "scale=2; $SIZE_MINIMAL/1024/1024" | bc) | 2 MB | TBD |"
echo "| Default | $SIZE_DEFAULT | $(echo "scale=2; $SIZE_DEFAULT/1024/1024" | bc) | 4 MB | TBD |"
echo "| CQRS | $SIZE_CQRS | $(echo "scale=2; $SIZE_CQRS/1024/1024" | bc) | 5 MB | TBD |"
echo "| All Features | $SIZE_ALL | $(echo "scale=2; $SIZE_ALL/1024/1024" | bc) | 8 MB | TBD |"
echo ""
