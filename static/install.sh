#!/bin/sh
# docfind installer script for Unix-like systems
# Usage: curl -fsSL https://microsoft.github.io/docfind/install.sh | sh

set -e

# Configuration
REPO="microsoft/docfind"
BINARY_NAME="docfind"
INSTALL_DIR="${DOCFIND_INSTALL_DIR:-$HOME/.local/bin}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
info() {
    printf "${GREEN}==>${NC} %s\n" "$1"
}

warn() {
    printf "${YELLOW}Warning:${NC} %s\n" "$1"
}

error() {
    printf "${RED}Error:${NC} %s\n" "$1" >&2
    exit 1
}

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"
    
    case "$OS" in
        Linux*)
            PLATFORM="unknown-linux-musl"
            ;;
        Darwin*)
            PLATFORM="apple-darwin"
            ;;
        *)
            error "Unsupported operating system: $OS"
            ;;
    esac
    
    case "$ARCH" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            error "Unsupported architecture: $ARCH"
            ;;
    esac
    
    TARGET="${ARCH}-${PLATFORM}"
    info "Detected platform: $TARGET"
}

# Get the current installed version
get_current_version() {
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        # Extract version from "docfind X.Y.Z" output
        CURRENT_VERSION=$("$BINARY_NAME" --version 2>/dev/null | sed -E 's/^[^ ]+ //')
        if [ -n "$CURRENT_VERSION" ]; then
            echo "$CURRENT_VERSION"
        fi
    fi
}

# Get the latest release version
get_latest_version() {
    info "Fetching latest release..."
    
    # Prepare auth header if GITHUB_TOKEN is set
    AUTH_HEADER=""
    if [ -n "$GITHUB_TOKEN" ]; then
        AUTH_HEADER="Authorization: Bearer $GITHUB_TOKEN"
    fi
    
    if command -v curl >/dev/null 2>&1; then
        if [ -n "$AUTH_HEADER" ]; then
            VERSION=$(curl -fsSL -H "$AUTH_HEADER" "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
        else
            VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
        fi
    elif command -v wget >/dev/null 2>&1; then
        if [ -n "$AUTH_HEADER" ]; then
            VERSION=$(wget -qO- --header="$AUTH_HEADER" "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
        else
            VERSION=$(wget -qO- "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
        fi
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
    
    if [ -z "$VERSION" ]; then
        error "Failed to fetch latest version"
    fi
    
    info "Latest version: $VERSION"
}

# Download and install binary
install_binary() {
    DOWNLOAD_URL="https://github.com/$REPO/releases/download/$VERSION/${BINARY_NAME}-${TARGET}.tar.gz"
    TEMP_FILE="/tmp/${BINARY_NAME}-${TARGET}.tar.gz"
    
    info "Downloading from $DOWNLOAD_URL..."
    
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$DOWNLOAD_URL" -o "$TEMP_FILE" || error "Download failed"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$DOWNLOAD_URL" -O "$TEMP_FILE" || error "Download failed"
    fi
    
    # Create install directory if it doesn't exist
    if [ ! -d "$INSTALL_DIR" ]; then
        info "Creating directory $INSTALL_DIR..."
        mkdir -p "$INSTALL_DIR" || error "Failed to create install directory"
    fi
    
    # Extract and install binary
    info "Extracting archive..."
    tar -xzf "$TEMP_FILE" -C "$INSTALL_DIR" || error "Failed to extract archive"
    
    info "Installing to $INSTALL_DIR/$BINARY_NAME..."
    chmod +x "$INSTALL_DIR/$BINARY_NAME" || error "Failed to make binary executable"
    
    # Clean up
    rm "$TEMP_FILE" 2>/dev/null || true
    
    info "Successfully installed $BINARY_NAME to $INSTALL_DIR"
}

# Check if install directory is in PATH
check_path() {
    case ":$PATH:" in
        *":$INSTALL_DIR:"*)
            return 0
            ;;
        *)
            return 1
            ;;
    esac
}

# Print post-install instructions
post_install() {
    echo ""
    info "Installation complete!"
    
    if ! check_path; then
        warn "$INSTALL_DIR is not in your PATH"
        echo ""
        echo "Add it to your PATH by adding this line to your shell profile:"
        echo "  ${GREEN}export PATH=\"\$PATH:$INSTALL_DIR\"${NC}"
        echo ""
        
        # Detect shell and provide specific instructions
        SHELL_NAME="$(basename "$SHELL")"
        case "$SHELL_NAME" in
            bash)
                echo "For bash, add it to ~/.bashrc or ~/.bash_profile"
                ;;
            zsh)
                echo "For zsh, add it to ~/.zshrc"
                ;;
            fish)
                echo "For fish, run: ${GREEN}fish_add_path $INSTALL_DIR${NC}"
                ;;
            *)
                echo "Add it to your shell's configuration file"
                ;;
        esac
        echo ""
        echo "Then reload your shell or run: ${GREEN}source ~/.${SHELL_NAME}rc${NC}"
    else
        echo "You can now use '${GREEN}$BINARY_NAME${NC}' from anywhere!"
    fi
    
    echo ""
    echo "Try it out:"
    echo "  ${GREEN}$BINARY_NAME --help${NC}"
}

# Main installation flow
main() {
    info "Installing $BINARY_NAME..."
    
    detect_platform
    get_latest_version
    
    # Check if already installed with the same version
    CURRENT_VERSION=$(get_current_version)
    if [ -n "$CURRENT_VERSION" ]; then
        info "Current version: $CURRENT_VERSION"
        # Strip 'v' prefix from VERSION if present for comparison
        LATEST_VERSION_NUM=$(echo "$VERSION" | sed 's/^v//')
        if [ "$CURRENT_VERSION" = "$LATEST_VERSION_NUM" ] || [ "$CURRENT_VERSION" = "$VERSION" ]; then
            info "$BINARY_NAME $CURRENT_VERSION is already installed (latest version)"
            echo ""
            echo "If you want to reinstall, please uninstall first:"
            echo "  ${GREEN}rm \$(which $BINARY_NAME)${NC}"
            exit 0
        fi
    fi
    
    install_binary
    post_install
}

main
