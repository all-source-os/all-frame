#!/usr/bin/env bash
#
# Check for broken links in documentation
#
# Usage: ./scripts/check_links.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DOCS_DIR="${PROJECT_ROOT}/docs"

echo "🔗 Checking AllFrame documentation links..."
echo ""

BROKEN_LINKS=0
CHECKED_FILES=0

# Find all markdown files
while IFS= read -r -d '' file; do
    CHECKED_FILES=$((CHECKED_FILES + 1))
    RELATIVE_FILE="${file#${PROJECT_ROOT}/}"

    # Extract markdown links: [text](url)
    grep -oE '\[([^]]+)\]\(([^)]+)\)' "$file" || true | while IFS= read -r link; do
        # Extract URL from [text](url)
        URL=$(echo "$link" | sed -n 's/.*](\([^)]*\)).*/\1/p')

        # Skip external URLs (http/https)
        if [[ "$URL" =~ ^https?:// ]]; then
            continue
        fi

        # Skip anchors
        if [[ "$URL" =~ ^# ]]; then
            continue
        fi

        # Resolve relative path
        FILE_DIR="$(dirname "$file")"
        TARGET_PATH="$(cd "$FILE_DIR" && realpath -m "$URL" 2>/dev/null || echo "")"

        if [ -z "$TARGET_PATH" ]; then
            echo "⚠️  Invalid path in ${RELATIVE_FILE}:"
            echo "   Link: ${link}"
            echo "   URL: ${URL}"
            BROKEN_LINKS=$((BROKEN_LINKS + 1))
            continue
        fi

        # Check if target exists
        if [ ! -e "$TARGET_PATH" ]; then
            # Check if it's an anchor link (file#anchor)
            if [[ "$URL" =~ ^([^#]+)#.* ]]; then
                BASE_PATH="${BASH_REMATCH[1]}"
                TARGET_PATH="$(cd "$FILE_DIR" && realpath -m "$BASE_PATH" 2>/dev/null || echo "")"
            fi

            if [ ! -e "$TARGET_PATH" ]; then
                echo "❌ Broken link in ${RELATIVE_FILE}:"
                echo "   Link: ${link}"
                echo "   Target: ${URL}"
                echo "   Resolved: ${TARGET_PATH}"
                BROKEN_LINKS=$((BROKEN_LINKS + 1))
            fi
        fi
    done
done < <(find "$DOCS_DIR" -name "*.md" -print0)

echo ""
echo "📊 Summary:"
echo "   • Files checked: ${CHECKED_FILES}"
echo "   • Broken links: ${BROKEN_LINKS}"
echo ""

if [ "$BROKEN_LINKS" -eq 0 ]; then
    echo "✅ All links are valid!"
    exit 0
else
    echo "❌ Found ${BROKEN_LINKS} broken link(s)"
    exit 1
fi
