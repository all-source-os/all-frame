#!/usr/bin/env bash
#
# Update documentation statistics from actual code
#
# Usage: ./scripts/update_stats.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "📊 Updating AllFrame documentation statistics..."
echo ""

# Get test count
echo "🧪 Counting tests..."
cd "${PROJECT_ROOT}/crates/allframe-core"
TEST_COUNT=$(cargo test --all-features 2>&1 | grep -E "test result:" | awk '{print $4}' | head -1)

if [ -z "$TEST_COUNT" ]; then
    echo "❌ Failed to get test count from cargo test"
    exit 1
fi

echo "   Total tests: ${TEST_COUNT}"
echo ""

# Get code statistics using tokei (if available)
if command -v tokei &> /dev/null; then
    echo "📝 Counting lines of code..."
    TOKEI_OUTPUT=$(tokei --output json "${PROJECT_ROOT}/crates/allframe-core/src")
    # Parse JSON to get Rust lines
    RUST_LINES=$(echo "$TOKEI_OUTPUT" | grep -o '"code":[0-9]*' | head -1 | cut -d':' -f2)
    echo "   Rust code lines: ${RUST_LINES}"
    echo ""
else
    echo "⚠️  tokei not found - skipping LOC count"
    echo "   Install with: cargo install tokei"
    RUST_LINES="~5,835"
fi

# Count files
echo "📁 Counting files..."
RUST_FILES=$(find "${PROJECT_ROOT}/crates/allframe-core/src" -name "*.rs" | wc -l | tr -d ' ')
DOC_FILES=$(find "${PROJECT_ROOT}/docs" -name "*.md" | wc -l | tr -d ' ')

echo "   Rust files: ${RUST_FILES}"
echo "   Doc files: ${DOC_FILES}"
echo ""

# Summary
echo "✅ Statistics Summary:"
echo "   • Total tests: ${TEST_COUNT}"
echo "   • Rust lines: ${RUST_LINES}"
echo "   • Rust files: ${RUST_FILES}"
echo "   • Doc files: ${DOC_FILES}"
echo ""
echo "💡 Next: Update PROJECT_STATUS.md with these values"
echo ""
echo "Example:"
echo "  Tests: ${TEST_COUNT} (was: 99)"
echo "  Lines: ${RUST_LINES} (was: ~5,835)"
echo "  Files: ${RUST_FILES} (was: ~46)"
