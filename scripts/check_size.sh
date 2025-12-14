#!/usr/bin/env bash
# Binary size checking script for AllFrame
# Usage: ./scripts/check_size.sh

set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Size limits (in MB) - Updated Dec 2025 for auth/resilience features
MINIMAL_LIMIT=4.0
DEFAULT_LIMIT=6.0
ALL_FEATURES_LIMIT=10.0

echo "🔍 AllFrame Binary Size Check"
echo "==============================="
echo ""

# Change to allframe-core directory
cd "$(dirname "$0")/../crates/allframe-core" || exit 1

# Function to get file size in MB
get_size_mb() {
    local file=$1
    if [[ -f "$file" ]]; then
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS
            local bytes=$(stat -f%z "$file")
        else
            # Linux
            local bytes=$(stat -c%s "$file")
        fi
        echo "scale=2; $bytes / 1048576" | bc
    else
        echo "0"
    fi
}

# Function to check size against limit
check_size() {
    local name=$1
    local size=$2
    local limit=$3

    if (( $(echo "$size > $limit" | bc -l) )); then
        echo -e "${RED}❌ $name: ${size}MB (exceeds ${limit}MB limit)${NC}"
        return 1
    else
        echo -e "${GREEN}✅ $name: ${size}MB (under ${limit}MB limit)${NC}"
        return 0
    fi
}

EXIT_CODE=0

echo "📦 Building configurations..."
echo ""

# Build minimal
echo "Building minimal (no features)..."
cargo build --release --no-default-features --quiet 2>&1 | grep -v "Compiling\|Finished" || true

# Build default
echo "Building default features..."
cargo build --release --quiet 2>&1 | grep -v "Compiling\|Finished" || true

# Build main features (excluding allsource which has external dependencies)
echo "Building main features..."
cargo build --release --features "di,openapi,router,cqrs,otel" --quiet 2>&1 | grep -v "Compiling\|Finished" || true

echo ""
echo "📊 Size Analysis"
echo "================"
echo ""

# Get sizes
MINIMAL_SIZE=$(get_size_mb "../../target/release/liballframe_core.rlib")
DEFAULT_SIZE=$(get_size_mb "../../target/release/liballframe_core.rlib")
ALL_SIZE=$(get_size_mb "../../target/release/liballframe_core.rlib")

# Check sizes
check_size "Minimal" "$MINIMAL_SIZE" "$MINIMAL_LIMIT" || EXIT_CODE=1
check_size "Default" "$DEFAULT_SIZE" "$DEFAULT_LIMIT" || EXIT_CODE=1
check_size "Main Features" "$ALL_SIZE" "$ALL_FEATURES_LIMIT" || EXIT_CODE=1

echo ""

if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}✅ All binary sizes within limits!${NC}"
else
    echo -e "${RED}❌ Some binaries exceed size limits${NC}"
    echo ""
    echo "Run ./scripts/analyze_size.sh for detailed analysis"
fi

exit $EXIT_CODE
