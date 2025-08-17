#!/usr/bin/env bash
set -e

# awsomarchy Installer
# This script installs awsomarchy on your system interactively or unattended

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
      echo "Install awsomarchy on your system."
      echo
      echo "Options:"
      echo "  --version VERSION    Specify version to install (defaults to latest)"
      echo "  --dir DIR            Installation directory (default: auto-detect)"
      echo "  --help               Display this help and exit"
      echo
      echo "Installation Directory Selection:"
      echo "  The installer automatically finds a writable directory in this order:"
      echo "  1. /usr/local/bin (system-wide, requires sudo)"
      echo "  2. \$HOME/.local/bin (user-local, recommended)"
      echo "  3. \$HOME/bin (user-local, alternative)"
      echo
      echo "Examples:"
      echo "  $0                           # Interactive installation with auto-detection"
      echo "  $0 --version 1.0.0          # Install specific version"
      echo "  $0 --dir \$HOME/.local/bin    # Install to specific directory"
      echo "  sudo $0                     # System-wide installation"
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

# Function to check if a directory is writable
is_writable() {
    [ -w "$1" ]
}

# Function to find the best writable installation directory
find_writable_install_dir() {
    local preferred_dirs=("/usr/local/bin" "$HOME/.local/bin" "$HOME/bin")
    
    # If user specified a directory via --dir, try that first
    if [ "$INSTALL_DIR" != "/usr/local/bin" ]; then
        preferred_dirs=("$INSTALL_DIR" "${preferred_dirs[@]}")
    fi
    
    for dir in "${preferred_dirs[@]}"; do
        # Create directory if it doesn't exist
        if mkdir -p "$dir" 2>/dev/null && is_writable "$dir"; then
            echo "$dir"
            return 0
        fi
    done
    
    # If nothing worked, return empty string
    echo ""
    return 1
}

# Determine installation directory (works in both interactive and unattended modes)
CHOSEN_DIR=$(find_writable_install_dir)
if [ -z "$CHOSEN_DIR" ]; then
    if [ "$UNATTENDED" = true ]; then
        error_exit "Cannot find a writable installation directory. Tried:
  - /usr/local/bin (requires sudo)
  - \$HOME/.local/bin ($HOME/.local/bin)
  - \$HOME/bin ($HOME/bin)

To install to a specific directory, use: $0 --dir /path/to/directory
For user-local installation, try: $0 --dir \$HOME/.local/bin
To install system-wide, run: sudo $0"
    else
        error_exit "Cannot find a writable installation directory. Options:
  1. Run with sudo for system-wide installation: sudo $0
  2. Install to user directory: $0 --dir \$HOME/.local/bin
  3. Create and use custom directory: $0 --dir /path/to/directory

Tried these directories:
  - /usr/local/bin (requires sudo)
  - \$HOME/.local/bin ($HOME/.local/bin)
  - \$HOME/bin ($HOME/bin)"
    fi
else
    INSTALL_DIR="$CHOSEN_DIR"
fi

# Handle PATH updates for both interactive and unattended modes
if [ "$INSTALL_DIR" != "/usr/local/bin" ]; then
    # Only handle PATH for user directories
    PATH_CMD="export PATH=\"$INSTALL_DIR:\$PATH\""
    SHELL_CONFIG=""
    
    # Determine shell config file
    if [ -n "$BASH_VERSION" ]; then
        SHELL_CONFIG="$HOME/.bashrc"
    elif [ -n "$ZSH_VERSION" ]; then
        SHELL_CONFIG="$HOME/.zshrc"
    elif [ -n "$BASH" ]; then
        SHELL_CONFIG="$HOME/.bashrc"
    elif [ -n "$ZSH_NAME" ]; then
        SHELL_CONFIG="$HOME/.zshrc"
    fi
    
    # Check if PATH update is needed
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]] && [ -n "$SHELL_CONFIG" ]; then
        if [ "$UNATTENDED" = true ]; then
            print_msg "Adding $INSTALL_DIR to PATH in $SHELL_CONFIG"
            echo -e "\n# Added by awsomarchy installer\n$PATH_CMD" >> "$SHELL_CONFIG"
            print_msg "Restart your shell or run: source $SHELL_CONFIG"
        else
            print_msg "Adding $INSTALL_DIR to your PATH in $SHELL_CONFIG"
            echo -e "\n# Added by awsomarchy installer\n$PATH_CMD" >> "$SHELL_CONFIG"
            print_msg "Please run 'source $SHELL_CONFIG' or restart your shell."
        fi
        export PATH="$INSTALL_DIR:$PATH" # Add to current session
    fi
fi

# Function to construct SHA256 URL from binary URL
construct_sha256_url() {
    local binary_url="$1"
    # Replace .tar.gz extension with .sha256
    echo "${binary_url%.tar.gz}.sha256"
}

# Function to check if SHA256 tools are available
check_sha256_tool() {
    if command -v shasum >/dev/null 2>&1; then
        echo "shasum"
    elif command -v sha256sum >/dev/null 2>&1; then
        echo "sha256sum"
    else
        echo ""
    fi
}

# Function to compute SHA256 hash of a file
compute_file_sha256() {
    local file_path="$1"
    local tool=$(check_sha256_tool)
    
    case $tool in
        "shasum")
            shasum -a 256 "$file_path" | cut -d' ' -f1
            ;;
        "sha256sum")
            sha256sum "$file_path" | cut -d' ' -f1
            ;;
        "")
            echo ""
            ;;
    esac
}

# Function to download and parse SHA256 hash file
download_sha256_hash() {
    local sha256_url="$1"
    local temp_file=$(mktemp)
    
    # Download SHA256 file
    if curl -fsSL -o "$temp_file" "$sha256_url" 2>/dev/null; then
        # Parse hash from content (first field, separated by whitespace)
        local hash_line=$(head -n 1 "$temp_file")
        local hash=$(echo "$hash_line" | cut -d' ' -f1)
        
        # Clean up temp file
        rm -f "$temp_file"
        
        # Validate hash format (64 hex characters)
        if echo "$hash" | grep -q '^[a-fA-F0-9]\{64\}$'; then
            echo "$hash"
        else
            echo ""
        fi
    else
        # Clean up temp file on failure
        rm -f "$temp_file"
        echo ""
    fi
}

# Function to verify file SHA256 hash
verify_file_sha256() {
    local file_path="$1"
    local expected_hash="$2"
    
    # Check if SHA256 tool is available
    local tool=$(check_sha256_tool)
    if [ -z "$tool" ]; then
        print_msg "⚠️  SHA256 verification skipped - no hash computation tool available"
        return 0  # Allow installation to proceed
    fi
    
    print_msg "🔐 Verifying file integrity..."
    print_msg "   Expected: ${expected_hash:0:16}..."
    
    # Compute file hash
    local computed_hash=$(compute_file_sha256 "$file_path")
    if [ -z "$computed_hash" ]; then
        print_msg "⚠️  SHA256 computation failed - proceeding without verification"
        return 0  # Allow installation to proceed
    fi
    
    print_msg "   Computed: ${computed_hash:0:16}..."
    
    # Compare hashes (case-insensitive)
    local expected_lower=$(echo "$expected_hash" | tr '[:upper:]' '[:lower:]')
    local computed_lower=$(echo "$computed_hash" | tr '[:upper:]' '[:lower:]')
    
    if [ "$computed_lower" = "$expected_lower" ]; then
        print_success "✅ File integrity verified successfully!"
        return 0
    else
        error_exit "❌ SHA256 verification failed!
Expected: $expected_hash
Computed: $computed_hash
This indicates file corruption or a security issue. Aborting installation."
    fi
}

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

print_msg "Installing awsomarchy $VERSION for $ARCH-$OS..."

# Construct package name and download URL
BINARY_NAME="awsomarchy"
PKG_NAME="${BINARY_NAME}-${ARCH}-${OS}.tar.gz"
DOWNLOAD_URL="https://github.com/aorumbayev/awesome-omarchy-tui/releases/download/v$VERSION/$PKG_NAME"
SHA256_URL=$(construct_sha256_url "$DOWNLOAD_URL")

# Check binary and SHA256 file availability
print_msg "🔍 Verifying binary and hash file availability..."
if curl -fsSL --head "$DOWNLOAD_URL" >/dev/null 2>&1; then
    # Get content length for binary size display
    binary_size=$(curl -fsSL --head "$DOWNLOAD_URL" 2>/dev/null | grep -i content-length | cut -d' ' -f2 | tr -d '\r')
    if [ -n "$binary_size" ]; then
        size_mb=$(echo "scale=1; $binary_size/1024/1024" | bc 2>/dev/null || echo "unknown")
        print_success "✅ Binary verified (${size_mb} MB)"
    else
        print_success "✅ Binary verified"
    fi
    
    # Check SHA256 file availability
    expected_hash=""
    if curl -fsSL --head "$SHA256_URL" >/dev/null 2>&1; then
        print_success "✅ SHA256 hash file verified - integrity validation will be performed"
        print_msg "🔍 Downloading SHA256 hash file..."
        expected_hash=$(download_sha256_hash "$SHA256_URL")
        
        if [ -n "$expected_hash" ]; then
            print_success "✅ SHA256 hash retrieved: ${expected_hash:0:16}..."
        else
            print_msg "⚠️  SHA256 file format invalid - proceeding without verification"
            expected_hash=""
        fi
    else
        print_msg "⚠️  SHA256 file not available - proceeding without integrity verification"
        print_msg "   Consider using a release that includes SHA256 files for enhanced security"
    fi
else
    error_exit "Binary not found at $DOWNLOAD_URL. The release may still be uploading."
fi

# Create temp directory
TMP_DIR=$(mktemp -d)
trap 'rm -rf -- "$TMP_DIR"' EXIT
cd "$TMP_DIR"

# Download and extract
print_msg "⬇️  Downloading from $DOWNLOAD_URL..."
curl -fsSL -o "$PKG_NAME" "$DOWNLOAD_URL" || error_exit "Download failed. Check URL or network."

# Verify SHA256 hash if available
if [ -n "$expected_hash" ]; then
    verify_file_sha256 "$PKG_NAME" "$expected_hash"
    print_success "🔐 Hash verification passed - proceeding with installation..."
else
    print_msg "⚠️  Installing without SHA256 verification (hash file not available)"
fi

print_msg "📦 Extracting archive..."
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
if [ -n "$expected_hash" ]; then
    print_success "🔐 Installation completed with verified integrity!"
fi
print_success "🚀 Run '${BINARY_NAME}' to get started browsing the awesome-omarchy repository!"