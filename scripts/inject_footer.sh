#!/usr/bin/env bash
#
# Inject consistent footer into documentation files
#
# Usage: ./scripts/inject_footer.sh [--dry-run]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DOCS_DIR="${PROJECT_ROOT}/docs"
FOOTER_TEMPLATE="${DOCS_DIR}/_templates/FOOTER.md"

DRY_RUN=false
if [ "${1:-}" = "--dry-run" ]; then
    DRY_RUN=true
    echo "🔍 DRY RUN MODE - No files will be modified"
    echo ""
fi

echo "📝 Injecting consistent footers into documentation..."
echo ""

if [ ! -f "$FOOTER_TEMPLATE" ]; then
    echo "❌ Footer template not found: ${FOOTER_TEMPLATE}"
    exit 1
fi

FOOTER=$(cat "$FOOTER_TEMPLATE")
UPDATED_FILES=0

# Find all markdown files (excluding templates and archive)
while IFS= read -r -d '' file; do
    RELATIVE_FILE="${file#${PROJECT_ROOT}/}"

    # Skip template files and archived docs
    if [[ "$file" =~ /_templates/ ]] || [[ "$file" =~ /archive/ ]]; then
        continue
    fi

    # Check if file already has the standard footer
    if grep -q "AllFrame. One frame. Infinite transformations." "$file"; then
        # Check if it's the current footer
        if ! diff -q <(tail -n 6 "$file") <(echo "$FOOTER") > /dev/null 2>&1; then
            echo "📝 Updating footer in: ${RELATIVE_FILE}"

            if [ "$DRY_RUN" = false ]; then
                # Remove old footer and add new one
                # Find the last occurrence of the footer separator
                LINE_NUM=$(grep -n "^---$" "$file" | tail -1 | cut -d':' -f1)

                if [ -n "$LINE_NUM" ]; then
                    # Remove everything after the last separator
                    head -n $((LINE_NUM - 1)) "$file" > "${file}.tmp"
                    echo "" >> "${file}.tmp"
                    echo "$FOOTER" >> "${file}.tmp"
                    mv "${file}.tmp" "$file"
                    UPDATED_FILES=$((UPDATED_FILES + 1))
                fi
            else
                UPDATED_FILES=$((UPDATED_FILES + 1))
            fi
        fi
    else
        echo "➕ Adding footer to: ${RELATIVE_FILE}"

        if [ "$DRY_RUN" = false ]; then
            echo "" >> "$file"
            echo "$FOOTER" >> "$file"
            UPDATED_FILES=$((UPDATED_FILES + 1))
        else
            UPDATED_FILES=$((UPDATED_FILES + 1))
        fi
    fi
done < <(find "$DOCS_DIR" -name "*.md" -print0)

echo ""
echo "📊 Summary:"
echo "   • Files updated: ${UPDATED_FILES}"

if [ "$DRY_RUN" = true ]; then
    echo ""
    echo "💡 Run without --dry-run to apply changes"
else
    echo ""
    echo "✅ Footers updated successfully!"
fi
