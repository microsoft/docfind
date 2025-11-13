#!/usr/bin/env bash
set -euo pipefail

# Version bumping script for Cargo workspace
# Usage: ./scripts/version.sh [major|minor|patch|<version>]

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to display usage
usage() {
    echo "Usage: $0 [major|minor|patch|<version>]"
    echo ""
    echo "Examples:"
    echo "  $0 patch       # 0.2.0 -> 0.2.1"
    echo "  $0 minor       # 0.2.0 -> 0.3.0"
    echo "  $0 major       # 0.2.0 -> 1.0.0"
    echo "  $0 1.5.2       # Set specific version"
    exit 1
}

# Function to extract current version from a Cargo.toml file
get_version() {
    local file="$1"
    grep '^version = ' "$file" | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# Function to validate semantic version format
is_valid_version() {
    local version="$1"
    if [[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        return 0
    else
        return 1
    fi
}

# Function to bump version
bump_version() {
    local current="$1"
    local bump_type="$2"
    
    IFS='.' read -r major minor patch <<< "$current"
    
    case "$bump_type" in
        major)
            echo "$((major + 1)).0.0"
            ;;
        minor)
            echo "${major}.$((minor + 1)).0"
            ;;
        patch)
            echo "${major}.${minor}.$((patch + 1))"
            ;;
        *)
            if is_valid_version "$bump_type"; then
                echo "$bump_type"
            else
                echo -e "${RED}Error: Invalid version format: $bump_type${NC}" >&2
                echo -e "${YELLOW}Version must be in format: X.Y.Z${NC}" >&2
                exit 1
            fi
            ;;
    esac
}

# Function to update version in a Cargo.toml file
update_cargo_toml() {
    local file="$1"
    local new_version="$2"
    
    # Use awk to replace only the first occurrence of version line (fully compatible)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        awk -v new_ver="$new_version" '/^version = / && !done { sub(/^version = ".*"/, "version = \"" new_ver "\""); done=1 } 1' "$file" > "$file.tmp" && mv "$file.tmp" "$file"
    else
        # GNU sed supports the 0,/pattern/ syntax
        sed -i "0,/^version = \".*\"/s//version = \"$new_version\"/" "$file"
    fi
    
    echo -e "${GREEN}✓${NC} Updated $(basename $(dirname "$file"))/$(basename "$file")"
}

# Main script
main() {
    if [ $# -eq 0 ]; then
        usage
    fi
    
    local bump_type="$1"
    
    # Find all Cargo.toml files with version fields
    CARGO_FILES=(
        "$PROJECT_ROOT/cli/Cargo.toml"
        "$PROJECT_ROOT/core/Cargo.toml"
        "$PROJECT_ROOT/wasm/Cargo.toml"
    )
    
    # Get current version from the first file (cli)
    CURRENT_VERSION=$(get_version "${CARGO_FILES[0]}")
    
    if [ -z "$CURRENT_VERSION" ]; then
        echo -e "${RED}Error: Could not determine current version${NC}"
        exit 1
    fi
    
    echo -e "Current version: ${YELLOW}$CURRENT_VERSION${NC}"
    
    # Calculate new version
    NEW_VERSION=$(bump_version "$CURRENT_VERSION" "$bump_type")
    
    echo -e "New version:     ${GREEN}$NEW_VERSION${NC}"
    echo ""
    
    # Confirm with user
    read -p "Update version to $NEW_VERSION? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Aborted"
        exit 0
    fi
    
    # Update all Cargo.toml files
    echo ""
    echo "Updating Cargo.toml files..."
    for file in "${CARGO_FILES[@]}"; do
        if [ -f "$file" ]; then
            update_cargo_toml "$file" "$NEW_VERSION"
        else
            echo -e "${YELLOW}Warning: File not found: $file${NC}"
        fi
    done
    
    echo ""
    echo -e "${GREEN}Version updated successfully!${NC}"
    echo ""
    
    echo ""
    echo -e "${YELLOW}Changes not committed. You can review and commit manually:${NC}"
    echo "  git add cli/Cargo.toml core/Cargo.toml wasm/Cargo.toml"
    echo "  git commit -m 'Bump version to $NEW_VERSION'"
    echo "  git tag -a v$NEW_VERSION -m 'Release version $NEW_VERSION'"
    echo "  git push && git push --tags"
}

main "$@"
