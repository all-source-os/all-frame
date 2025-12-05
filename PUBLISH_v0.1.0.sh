#!/bin/bash
#
# AllFrame v0.1.0 Publishing Script
#
# This script publishes all AllFrame crates to crates.io in the correct dependency order.
#
# IMPORTANT:
# 1. Ensure you're logged in: cargo login <token>
# 2. Ensure working directory is clean: git status
# 3. Review PUBLISHING_CHECKLIST.md before running
#
# Usage:
#   ./PUBLISH_v0.1.0.sh         # Dry run (safe)
#   ./PUBLISH_v0.1.0.sh --real  # Actually publish (dangerous!)

set -e  # Exit on error

REAL_PUBLISH=false
if [ "$1" == "--real" ]; then
    REAL_PUBLISH=true
    echo "⚠️  REAL PUBLISH MODE - This will actually upload to crates.io!"
    read -p "Are you sure? (yes/no): " confirm
    if [ "$confirm" != "yes" ]; then
        echo "Aborted."
        exit 1
    fi
else
    echo "🧪 DRY RUN MODE - No actual publishing will occur"
    echo "To actually publish, run: ./PUBLISH_v0.1.0.sh --real"
    echo ""
fi

# Track published crates
PUBLISHED=()

# Function to publish a crate
publish_crate() {
    local crate_name=$1
    local crate_path=$2
    local wait_time=${3:-120}  # Default 120s wait for indexing

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "📦 Publishing: $crate_name"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    cd "$crate_path"

    if [ "$REAL_PUBLISH" = true ]; then
        # Real publish
        cargo publish
        PUBLISHED+=("$crate_name")

        echo "✅ Published $crate_name"
        echo "⏳ Waiting ${wait_time}s for crates.io to index..."
        sleep "$wait_time"
    else
        # Dry run
        cargo publish --dry-run --allow-dirty
        echo "✅ Dry run successful for $crate_name"
    fi

    cd - > /dev/null
}

# Start publishing
echo "🚀 Starting AllFrame v0.1.0 publication process..."
echo ""

# Step 1: Publish allframe-macros (no dependencies)
publish_crate "allframe-macros" "crates/allframe-macros" 120

# Step 2: Publish allframe-core (depends on allframe-macros)
publish_crate "allframe-core" "crates/allframe-core" 120

# Step 3: Publish allframe-forge (independent CLI tool)
publish_crate "allframe-forge" "crates/allframe-forge" 120

# Step 4: Update allframe-mcp dependency (only in real mode)
if [ "$REAL_PUBLISH" = true ]; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🔧 Updating allframe-mcp dependencies"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    # Update Cargo.toml to use published allframe-core
    sed -i.bak 's|allframe-core = { path = "../allframe-core" }|allframe-core = "0.1.0"|g' crates/allframe-mcp/Cargo.toml

    # Verify it builds
    echo "Testing allframe-mcp builds with published allframe-core..."
    cd crates/allframe-mcp
    cargo build
    cargo test --lib
    cd - > /dev/null

    echo "✅ allframe-mcp updated and tested"
fi

# Step 5: Publish allframe-mcp (depends on allframe-core)
publish_crate "allframe-mcp" "crates/allframe-mcp" 120

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Publication Complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ "$REAL_PUBLISH" = true ]; then
    echo ""
    echo "Published crates:"
    for crate in "${PUBLISHED[@]}"; do
        echo "  ✅ $crate v0.1.0"
    done

    echo ""
    echo "Next steps:"
    echo "  1. Verify packages on crates.io:"
    echo "     - https://crates.io/crates/allframe-macros"
    echo "     - https://crates.io/crates/allframe-core"
    echo "     - https://crates.io/crates/allframe-forge"
    echo "     - https://crates.io/crates/allframe-mcp"
    echo ""
    echo "  2. Tag the release:"
    echo "     git tag -a v0.1.0 -m \"Release v0.1.0\""
    echo "     git push origin v0.1.0"
    echo ""
    echo "  3. Wait 5-10 minutes for docs.rs to build documentation"
    echo ""
    echo "  4. Test installation in a fresh project"
    echo ""
    echo "  5. Create announcement and share on social media"
else
    echo ""
    echo "🧪 This was a DRY RUN - no packages were published"
    echo ""
    echo "All crates passed dry-run validation!"
    echo "To actually publish, run:"
    echo "  ./PUBLISH_v0.1.0.sh --real"
fi

echo ""
echo "📚 See PUBLISHING_CHECKLIST.md for detailed post-publishing steps"
