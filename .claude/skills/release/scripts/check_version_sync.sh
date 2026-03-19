#!/usr/bin/env bash
# Verify all version strings in root Cargo.toml are in sync.
# Usage: bash check_version_sync.sh [expected_version]
# If expected_version is omitted, uses the [workspace.package] version.

set -euo pipefail

CARGO="Cargo.toml"

if [[ ! -f "$CARGO" ]]; then
  echo "ERROR: Run from repo root (Cargo.toml not found)" >&2
  exit 1
fi

# Extract workspace.package version
WS_VERSION=$(grep -A2 '^\[workspace\.package\]' "$CARGO" | grep '^version' | head -1 | sed 's/.*"\(.*\)".*/\1/')
EXPECTED="${1:-$WS_VERSION}"

echo "Expected version: $EXPECTED"

ERRORS=0

# Check [package] version
PKG_VERSION=$(grep -A5 '^\[package\]' "$CARGO" | grep '^version' | head -1 | sed 's/.*"\(.*\)".*/\1/')
if [[ "$PKG_VERSION" != "$EXPECTED" ]]; then
  echo "MISMATCH: [package] version = \"$PKG_VERSION\" (expected \"$EXPECTED\")"
  ERRORS=$((ERRORS + 1))
else
  echo "OK: [package] version"
fi

# Check workspace.package version
if [[ "$WS_VERSION" != "$EXPECTED" ]]; then
  echo "MISMATCH: [workspace.package] version = \"$WS_VERSION\" (expected \"$EXPECTED\")"
  ERRORS=$((ERRORS + 1))
else
  echo "OK: [workspace.package] version"
fi

# Check workspace dependency versions
for CRATE in allframe-core allframe-forge allframe-macros allframe-tauri; do
  DEP_VERSION=$(grep "^${CRATE}" "$CARGO" | head -1 | sed 's/.*version = "\([^"]*\)".*/\1/')
  if [[ "$DEP_VERSION" != "$EXPECTED" ]]; then
    echo "MISMATCH: [workspace.dependencies] $CRATE version = \"$DEP_VERSION\" (expected \"$EXPECTED\")"
    ERRORS=$((ERRORS + 1))
  else
    echo "OK: [workspace.dependencies] $CRATE"
  fi
done

if [[ $ERRORS -gt 0 ]]; then
  echo ""
  echo "FAILED: $ERRORS version mismatches found"
  exit 1
else
  echo ""
  echo "ALL VERSIONS IN SYNC: $EXPECTED"
fi
