#!/usr/bin/env bash
set -e

# awsmomarchy Installer
# This script installs awsmomarchy on your system interactively or unattended

# Colors (only used in interactive mode)
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Default values
VERSION=""
INSTALL_DIR="/usr/local/bin"
UNATTENDED=false

# Parse command-line arguments (unattended mode when args are provided)
while [[ $# -gt 0 ]]; do
  UNATTENDED=true
  case $1 in
    --version)
      VERSION="$2"
      shift 2
      ;;
    --dir)
      INSTALL_DIR="$2"
      shift 2
      ;;
    --help)
      echo "Usage: $0 [OPTIONS]"
      echo "Install awsmomarchy on your system."
      echo
      echo "Options:"
      echo "  --version VERSION    Specify version to install (defaults to latest)"
      echo "  --dir DIR            Installation directory (default: /usr/local/bin)"
      echo "  --help               Display this help and exit"
      echo
      echo "When run without options, the installer runs in interactive mode."
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      echo "Use --help for usage information."
      exit 1
      ;;
  esac
done

# Function to print errors
error_exit() {
    if [ "$UNATTENDED" = true ]; then
        echo "Error: $1" >&2
    else
        echo -e "${RED}Error: $1${NC}" >&2
    fi
    exit 1
}

# Function to print messages
print_msg() {
    if [ "$UNATTENDED" = true ]; then
        echo "$1"
    else
        echo -e "${BLUE}$1${NC}"
    fi
}

print_success() {
    if [ "$UNATTENDED" = true ]; then
        echo "$1"
    else
        echo -e "${GREEN}$1${NC}"
    fi
}

# Writable function to check if a directory is writable
is_writable() {
    [ -w "$1" ]
}

# Determine installation directory (in interactive mode)
if [ "$UNATTENDED" = false ]; then
    if ! is_writable "$INSTALL_DIR" && [ -d "$HOME/.local/bin" ]; then
        INSTALL_DIR="$HOME/.local/bin"
    fi

    # Create if it doesn't exist and check writability again
    mkdir -p "$INSTALL_DIR"
    if ! is_writable "$INSTALL_DIR"; then
        error_exit "Cannot write to $INSTALL_DIR. Please ensure the directory exists and you have permissions, or run with sudo. Alternatively, use --dir to specify a writable path."
    fi

    # Add to PATH if needed (using a more robust check)
    PATH_CMD="export PATH=\"$INSTALL_DIR:\$PATH\""
    SHELL_CONFIG=""
    if [ -n "$BASH_VERSION" ]; then
        SHELL_CONFIG="$HOME/.bashrc"
    elif [ -n "$ZSH_VERSION" ]; then
        SHELL_CONFIG="$HOME/.zshrc"
    fi

    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]] && [ -n "$SHELL_CONFIG" ]; then
        print_msg "Adding $INSTALL_DIR to your PATH in $SHELL_CONFIG"
        echo -e "\n# Added by awsmomarchy Installer\n$PATH_CMD" >> "$SHELL_CONFIG"
        print_msg "Please run 'source $SHELL_CONFIG' or restart your shell."
        export PATH="$INSTALL_DIR:$PATH" # Add to current session
    fi
else
    # In unattended mode, just ensure the directory exists
    mkdir -p "$INSTALL_DIR"
fi

# Detect architecture
ARCH=$(uname -m)
case $ARCH in
    x86_64)
        ARCH="x86_64"
        ;;
    aarch64|arm64)
        ARCH="aarch64"
        ;;
    *)
        error_exit "Unsupported architecture: $ARCH"
        ;;
esac

# Detect OS
OS=$(uname | tr '[:upper:]' '[:lower:]')
case $OS in
    darwin)
        OS="apple-darwin"
        ;;
    linux)
        OS="unknown-linux-gnu"
        ;;
    *)
        error_exit "Unsupported OS: $OS"
        ;;
esac

# Get latest version from GitHub if not specified
if [ -z "$VERSION" ]; then
    print_msg "Determining latest version..."
    VERSION=$(curl -s "https://api.github.com/repos/aorumbayev/awesome-omarchy-tui/releases/latest" | grep -o '"tag_name": "[^"]*' | cut -d'"' -f4)

    if [ -z "$VERSION" ]; then
        error_exit "Failed to determine latest version"
    fi
fi
VERSION=${VERSION#v}

print_msg "Installing awsmomarchy $VERSION for $ARCH-$OS..."

# Construct package name and download URL
BINARY_NAME="awsmomarchy"
PKG_NAME="${BINARY_NAME}-${ARCH}-${OS}.tar.gz"
DOWNLOAD_URL="https://github.com/aorumbayev/awesome-omarchy-tui/releases/download/v$VERSION/$PKG_NAME"

# Create temp directory
TMP_DIR=$(mktemp -d)
trap 'rm -rf -- "$TMP_DIR"' EXIT
cd "$TMP_DIR"

# Download and extract
print_msg "Downloading from $DOWNLOAD_URL..."
curl -fsSL -o "$PKG_NAME" "$DOWNLOAD_URL" || error_exit "Download failed. Check URL or network."

print_msg "Extracting archive..."
tar -xzf "$PKG_NAME" || error_exit "Failed to extract archive."

# Check if binary exists after extraction
if [ ! -f "$BINARY_NAME" ]; then
    error_exit "Binary '$BINARY_NAME' not found in the archive."
fi

# Mac OS specific quarantine removal
if [[ "$OS" == *"darwin"* ]]; then
    print_msg "Removing quarantine attribute (macOS)..."
    xattr -d com.apple.quarantine "$BINARY_NAME" 2>/dev/null || true
fi

# Install the binary
print_msg "Installing ${BINARY_NAME} to ${INSTALL_DIR}..."
chmod +x "$BINARY_NAME"
mv "$BINARY_NAME" "$INSTALL_DIR/"

# Clean up is handled by trap
cd - > /dev/null

print_success "${BINARY_NAME} ${VERSION} has been installed to ${INSTALL_DIR}/${BINARY_NAME}"
print_success "Run '${BINARY_NAME}' to get started browsing the awesome-omarchy repository!"