#!/usr/bin/env bash
# Detailed binary size analysis script for AllFrame
# Usage: ./scripts/analyze_size.sh [configuration]
# Configurations: minimal, default, all (default: all)

set -e

CONFIG="${1:-all}"

echo "🔬 AllFrame Binary Size Analysis"
echo "=================================="
echo ""

# Change to allframe-core directory
cd "$(dirname "$0")/../crates/allframe-core" || exit 1

# Check if cargo-bloat is installed
if ! command -v cargo-bloat &> /dev/null; then
    echo "❌ cargo-bloat is not installed"
    echo "Install it with: cargo install cargo-bloat"
    exit 1
fi

analyze_config() {
    local name=$1
    local features=$2

    echo "📊 Analyzing: $name"
    echo "-------------------"
    echo ""

    echo "Top 15 crate dependencies by size:"
    echo ""

    if [ "$features" == "none" ]; then
        cargo bloat --release --no-default-features --crates -n 15 2>&1 || echo "Analysis failed"
    elif [ "$features" == "default" ]; then
        cargo bloat --release --crates -n 15 2>&1 || echo "Analysis failed"
    else
        cargo bloat --release --all-features --crates -n 15 2>&1 || echo "Analysis failed"
    fi

    echo ""
    echo "Top 15 functions by size:"
    echo ""

    if [ "$features" == "none" ]; then
        cargo bloat --release --no-default-features -n 15 2>&1 || echo "Analysis failed"
    elif [ "$features" == "default" ]; then
        cargo bloat --release -n 15 2>&1 || echo "Analysis failed"
    else
        cargo bloat --release --all-features -n 15 2>&1 || echo "Analysis failed"
    fi

    echo ""
    echo "=================================="
    echo ""
}

case "$CONFIG" in
    minimal)
        analyze_config "Minimal (no features)" "none"
        ;;
    default)
        analyze_config "Default features" "default"
        ;;
    all)
        analyze_config "All features" "all"
        ;;
    full)
        analyze_config "Minimal (no features)" "none"
        analyze_config "Default features" "default"
        analyze_config "All features" "all"
        ;;
    *)
        echo "Unknown configuration: $CONFIG"
        echo "Usage: $0 [minimal|default|all|full]"
        exit 1
        ;;
esac

echo "✅ Analysis complete"
