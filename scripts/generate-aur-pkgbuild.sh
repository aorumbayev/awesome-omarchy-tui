#!/bin/bash

# ============================================================================
# AUR PKGBUILD Generator Script for awesome-omarchy-tui
# ============================================================================
# 
# This script generates compliant PKGBUILD files for both source and binary
# variants of awesome-omarchy-tui, following AUR submission guidelines.
#
# Author: Altynbek Orumbayev <aorumbayev@pm.me>
# Repository: https://github.com/aorumbayev/awesome-omarchy-tui
# License: MIT
# 
# Usage:
#   ./generate-aur-pkgbuild.sh [OPTIONS]
#
# Options:
#   -v, --version VERSION    Override version (default: extract from Cargo.toml)
#   -o, --output DIR         Output directory (default: ./aur-pkgbuilds)
#   -s, --source-only        Generate only source PKGBUILD
#   -b, --binary-only        Generate only binary PKGBUILD  
#   -f, --force              Force overwrite existing files
#   --validate               Validate generated PKGBUILDs (requires makepkg)
#   --no-checksums           Skip checksum calculation (for testing)
#   -h, --help              Show this help message
#
# Examples:
#   ./generate-aur-pkgbuild.sh                    # Generate both variants
#   ./generate-aur-pkgbuild.sh -v 1.0.0          # Use specific version
#   ./generate-aur-pkgbuild.sh -s -o /tmp/aur    # Source only to /tmp/aur
#   ./generate-aur-pkgbuild.sh --validate        # Generate and validate
#
# Generated Files:
#   - PKGBUILD-source: Source package (awesome-omarchy-tui)
#   - PKGBUILD-bin: Binary package (awesome-omarchy-tui-bin)  
#   - .SRCINFO-source: Source metadata
#   - .SRCINFO-bin: Binary metadata
#
# ============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DEFAULT_OUTPUT_DIR="$PROJECT_ROOT/aur-pkgbuilds"

REPO_URL="https://github.com/aorumbayev/awesome-omarchy-tui"
BINARY_NAME="awsomarchy"
SOURCE_PKGNAME="awesome-omarchy-tui"
BINARY_PKGNAME="awesome-omarchy-tui-bin"
MAINTAINER="Altynbek Orumbayev <aorumbayev@pm.me>"

VERSION=""
OUTPUT_DIR="$DEFAULT_OUTPUT_DIR"
GENERATE_SOURCE=true
GENERATE_BINARY=true
FORCE_OVERWRITE=false
VALIDATE_PKGBUILDS=false
CALCULATE_CHECKSUMS=true

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly CYAN='\033[0;36m'
readonly NC='\033[0m'

# Logging functions
log_info() {
    echo -e "${BLUE}INFO:${NC} $*" >&2
}

log_success() {
    echo -e "${GREEN}✓${NC} $*" >&2
}

log_warning() {
    echo -e "${YELLOW}WARNING:${NC} $*" >&2
}

log_error() {
    echo -e "${RED}ERROR:${NC} $*" >&2
}

log_step() {
    echo -e "$*..." >&2
}

# Help message
show_help() {
    cat << 'EOF'
AUR PKGBUILD Generator for awesome-omarchy-tui

This script generates compliant PKGBUILD files for both source and binary
variants of awesome-omarchy-tui, following AUR submission guidelines.

USAGE:
    generate-aur-pkgbuild.sh [OPTIONS]

OPTIONS:
    -v, --version VERSION    Override version (default: extract from Cargo.toml)
    -o, --output DIR         Output directory (default: ./aur-pkgbuilds)
    -s, --source-only        Generate only source PKGBUILD
    -b, --binary-only        Generate only binary PKGBUILD
    -f, --force              Force overwrite existing files
    --validate               Validate generated PKGBUILDs (requires makepkg)
    --no-checksums           Skip checksum calculation (for testing)
    -h, --help               Show this help message

EXAMPLES:
    generate-aur-pkgbuild.sh                    Generate both variants
    generate-aur-pkgbuild.sh -v 1.0.0          Use specific version
    generate-aur-pkgbuild.sh -s -o /tmp/aur    Source only to /tmp/aur
    generate-aur-pkgbuild.sh --validate        Generate and validate

GENERATED FILES:
    PKGBUILD-source         Source package (awesome-omarchy-tui)
    PKGBUILD-bin           Binary package (awesome-omarchy-tui-bin)
    .SRCINFO-source        Source metadata
    .SRCINFO-bin           Binary metadata

AUR COMPLIANCE FEATURES:
    ✓ Follows AUR package guidelines and naming conventions
    ✓ Proper metadata including pkgname, pkgver, pkgrel, pkgdesc, arch, license
    ✓ Automatic version extraction from Cargo.toml
    ✓ Dynamic checksum calculation for source and binary packages
    ✓ Support for both x86_64 architectures
    ✓ Proper maintainer information and comments
    ✓ Generated .SRCINFO files for AUR submission

ARCHITECTURE SUPPORT:
    x86_64                  Full support for both source and binary packages

For more information, see: https://wiki.archlinux.org/title/AUR_submission_guidelines
EOF
}

# Parse command line arguments
parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            -v|--version)
                VERSION="$2"
                shift 2
                ;;
            -o|--output)
                OUTPUT_DIR="$2"
                shift 2
                ;;
            -s|--source-only)
                GENERATE_SOURCE=true
                GENERATE_BINARY=false
                shift
                ;;
            -b|--binary-only)
                GENERATE_SOURCE=false
                GENERATE_BINARY=true
                shift
                ;;
            -f|--force)
                FORCE_OVERWRITE=true
                shift
                ;;
            --validate)
                VALIDATE_PKGBUILDS=true
                shift
                ;;
            --no-checksums)
                CALCULATE_CHECKSUMS=false
                shift
                ;;
            -h|--help)
                show_help
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                log_info "Use --help to see available options"
                exit 1
                ;;
        esac
    done
}

check_dependencies() {
    log_step "Checking dependencies"
    
    local missing_deps=()
    
    for cmd in curl sha256sum; do
        if ! command -v "$cmd" &> /dev/null; then
            missing_deps+=("$cmd")
        fi
    done
    
    # Check for optional validation tools
    if [[ "$VALIDATE_PKGBUILDS" == true ]] && ! command -v makepkg &> /dev/null; then
        log_error "makepkg is required for validation but not found"
        log_info "Install base-devel group: pacman -S base-devel"
        exit 1
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        exit 1
    fi
}

extract_version() {
    log_step "Extracting version information"
    
    if [[ -n "$VERSION" ]]; then
        log_info "Using provided version: $VERSION"
        return
    fi
    
    local cargo_toml="$PROJECT_ROOT/Cargo.toml"
    if [[ ! -f "$cargo_toml" ]]; then
        log_error "Cargo.toml not found at: $cargo_toml"
        exit 1
    fi
    
    VERSION=$(grep '^version = ' "$cargo_toml" | head -n1 | sed 's/version = "\(.*\)"/\1/')
    
    if [[ -z "$VERSION" ]]; then
        log_error "Could not extract version from Cargo.toml"
        exit 1
    fi
    
    # Validate version format (semantic versioning)
    if [[ ! "$VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([+-][a-zA-Z0-9.-]+)?$ ]]; then
        log_error "Invalid version format: $VERSION"
        log_info "Expected semantic versioning format (e.g., 1.0.0, 1.0.0-beta.1)"
        exit 1
    fi
    
    log_info "Version: $VERSION"
}

extract_metadata() {
    log_step "Extracting package metadata"
    
    local cargo_toml="$PROJECT_ROOT/Cargo.toml"
    
    DESCRIPTION=$(grep '^description = ' "$cargo_toml" | head -n1 | sed 's/description = "\(.*\)"/\1/')
    AUTHORS=$(grep '^authors = ' "$cargo_toml" | head -n1 | sed 's/authors = \[\(.*\)\]/\1/' | sed 's/"//g')
    LICENSE=$(grep '^license = ' "$cargo_toml" | head -n1 | sed 's/license = "\(.*\)"/\1/')
    REPOSITORY=$(grep '^repository = ' "$cargo_toml" | head -n1 | sed 's/repository = "\(.*\)"/\1/')
    
    # Extract keywords and categories for additional context
    KEYWORDS=$(grep '^keywords = ' "$cargo_toml" | head -n1 | sed 's/keywords = \[\(.*\)\]/\1/' | sed 's/"//g' | tr ',' ' ')
    CATEGORIES=$(grep '^categories = ' "$cargo_toml" | head -n1 | sed 's/categories = \[\(.*\)\]/\1/' | sed 's/"//g' | tr ',' ' ')
}

calculate_source_checksum() {
    if [[ "$CALCULATE_CHECKSUMS" != true ]]; then
        echo "SKIP"
        return
    fi
    
    log_step "Calculating source checksum"
    
    local tarball_url="https://github.com/aorumbayev/awesome-omarchy-tui/archive/refs/tags/v${VERSION}.tar.gz"
    
    local checksum
    if ! checksum=$(curl -fsSL "$tarball_url" | sha256sum | cut -d' ' -f1); then
        log_error "Failed to download or calculate checksum for source tarball"
        return 1
    fi
    
    if [[ -z "$checksum" ]] || [[ ${#checksum} -ne 64 ]]; then
        log_error "Invalid SHA256 checksum calculated: $checksum"
        return 1
    fi
    
    echo "$checksum"
}

calculate_binary_checksum() {
    local arch="$1"
    
    if [[ "$CALCULATE_CHECKSUMS" != true ]]; then
        echo "SKIP"
        return
    fi
    
    log_step "Calculating binary checksum ($arch)"
    
    local binary_url="https://github.com/aorumbayev/awesome-omarchy-tui/releases/download/v${VERSION}/${BINARY_NAME}-${arch}-unknown-linux-gnu.tar.gz"
    
    local checksum
    if ! checksum=$(curl -fsSL "$binary_url" | sha256sum | cut -d' ' -f1); then
        log_warning "Failed to download or calculate checksum for binary: $arch"
        log_warning "This might be expected if the release doesn't exist yet"
        echo "PLACEHOLDER_$(echo "$arch" | tr '[:lower:]' '[:upper:]')_CHECKSUM"
        return
    fi
    
    if [[ -z "$checksum" ]] || [[ ${#checksum} -ne 64 ]]; then
        log_warning "Invalid SHA256 checksum calculated for $arch: $checksum"
        echo "PLACEHOLDER_$(echo "$arch" | tr '[:lower:]' '[:upper:]')_CHECKSUM"
        return
    fi
    
    echo "$checksum"
}

generate_source_pkgbuild() {
    log_step "Generating source PKGBUILD"
    
    local output_file="$OUTPUT_DIR/PKGBUILD-source"
    local source_checksum
    
    source_checksum=$(calculate_source_checksum)
    
    cat > "$output_file" << EOF
# Maintainer: $MAINTAINER
# 
# This is the source package for awesome-omarchy-tui, which builds the application 
# from source using the Rust toolchain. For a binary package (pre-compiled), 
# see awesome-omarchy-tui-bin.

pkgname=$SOURCE_PKGNAME
pkgver=$VERSION
pkgrel=1
pkgdesc="$DESCRIPTION"
arch=('x86_64')
url="$REPOSITORY"
license=('$LICENSE')
makedepends=('rust' 'cargo')
source=("\${pkgname}-\${pkgver}.tar.gz::https://github.com/aorumbayev/awesome-omarchy-tui/archive/refs/tags/v\${pkgver}.tar.gz")
sha256sums=('$source_checksum')

prepare() {
    cd "\${srcdir}/\${pkgname}-\${pkgver}"
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target "\$CARCH-unknown-linux-gnu"
}

build() {
    cd "\${srcdir}/\${pkgname}-\${pkgver}"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release --no-default-features
}

check() {
    cd "\${srcdir}/\${pkgname}-\${pkgver}"
    export RUSTUP_TOOLCHAIN=stable
    cargo test --frozen --release --no-default-features
}

package() {
    cd "\${srcdir}/\${pkgname}-\${pkgver}"
    install -Dm0755 -t "\$pkgdir/usr/bin/" "target/release/$BINARY_NAME"
    install -Dm0644 LICENSE "\$pkgdir/usr/share/licenses/\$pkgname/LICENSE"
    install -Dm0644 README.md "\$pkgdir/usr/share/doc/\$pkgname/README.md"
}
EOF
    
    log_success "Source PKGBUILD: $output_file"
}

generate_binary_pkgbuild() {
    log_step "Generating binary PKGBUILD"
    
    local output_file="$OUTPUT_DIR/PKGBUILD-bin"
    local x86_64_checksum
    
    x86_64_checksum=$(calculate_binary_checksum "x86_64")
    
    cat > "$output_file" << EOF
# Maintainer: $MAINTAINER
#
# This is the binary package for awesome-omarchy-tui, which provides pre-compiled
# binaries for faster installation. For building from source, see awesome-omarchy-tui.

pkgname=$BINARY_PKGNAME
pkgver=$VERSION
pkgrel=1
pkgdesc="$DESCRIPTION (binary package)"
arch=('x86_64')
url="$REPOSITORY"
license=('$LICENSE')
provides=('$SOURCE_PKGNAME')
conflicts=('$SOURCE_PKGNAME')
source_x86_64=("\${pkgname%-bin}-\${pkgver}-x86_64.tar.gz::https://github.com/aorumbayev/awesome-omarchy-tui/releases/download/v\${pkgver}/$BINARY_NAME-x86_64-unknown-linux-gnu.tar.gz")
sha256sums_x86_64=('$x86_64_checksum')

package() {
    # Install binary
    install -Dm0755 "\$srcdir/$BINARY_NAME" "\$pkgdir/usr/bin/$BINARY_NAME"
    
    # Install documentation and license files required by AUR guidelines
    install -Dm0644 "\$srcdir/LICENSE" "\$pkgdir/usr/share/licenses/\$pkgname/LICENSE"
    install -Dm0644 "\$srcdir/README.md" "\$pkgdir/usr/share/doc/\$pkgname/README.md"
}
EOF
    
    log_success "Binary PKGBUILD: $output_file"
}

generate_srcinfo() {
    local pkgbuild_type="$1"  # "source" or "bin"
    local pkgbuild_file="$OUTPUT_DIR/PKGBUILD-$pkgbuild_type"
    local srcinfo_file="$OUTPUT_DIR/.SRCINFO-$pkgbuild_type"
    
    log_step "Generating .SRCINFO ($pkgbuild_type)"
    
    if [[ ! -f "$pkgbuild_file" ]]; then
        log_error "PKGBUILD file not found: $pkgbuild_file"
        return 1
    fi
    
    # Check if makepkg is available for proper .SRCINFO generation
    if command -v makepkg &> /dev/null; then
        local temp_dir
        temp_dir=$(mktemp -d)
        cp "$pkgbuild_file" "$temp_dir/PKGBUILD"
        
        (
            cd "$temp_dir"
            makepkg --printsrcinfo
        ) > "$srcinfo_file"
        
        rm -rf "$temp_dir"
        
        log_success ".SRCINFO ($pkgbuild_type): $srcinfo_file"
        return
    fi
    
    # Fallback: Generate .SRCINFO manually
    log_info "makepkg not available, generating .SRCINFO manually"
    
    if [[ "$pkgbuild_type" == "source" ]]; then
        generate_source_srcinfo_manual "$srcinfo_file"
    else
        generate_binary_srcinfo_manual "$srcinfo_file"
    fi
    
    log_success ".SRCINFO ($pkgbuild_type): $srcinfo_file"
}

generate_source_srcinfo_manual() {
    local output_file="$1"
    local source_checksum
    
    source_checksum=$(calculate_source_checksum)
    
    cat > "$output_file" << EOF
pkgbase = $SOURCE_PKGNAME
	pkgdesc = $DESCRIPTION
	pkgver = $VERSION
	pkgrel = 1
	url = $REPOSITORY
	arch = x86_64
	license = $LICENSE
	makedepends = rust
	makedepends = cargo
	source = $SOURCE_PKGNAME-$VERSION.tar.gz::https://github.com/aorumbayev/awesome-omarchy-tui/archive/refs/tags/v$VERSION.tar.gz
	sha256sums = $source_checksum

pkgname = $SOURCE_PKGNAME
EOF
}

generate_binary_srcinfo_manual() {
    local output_file="$1"
    local x86_64_checksum
    
    x86_64_checksum=$(calculate_binary_checksum "x86_64")
    
    cat > "$output_file" << EOF
pkgbase = $BINARY_PKGNAME
	pkgdesc = $DESCRIPTION (binary package)
	pkgver = $VERSION
	pkgrel = 1
	url = $REPOSITORY
	arch = x86_64
	license = $LICENSE
	provides = $SOURCE_PKGNAME
	conflicts = $SOURCE_PKGNAME
	source_x86_64 = $BINARY_PKGNAME-$VERSION-x86_64.tar.gz::https://github.com/aorumbayev/awesome-omarchy-tui/releases/download/v$VERSION/$BINARY_NAME-x86_64-unknown-linux-gnu.tar.gz
	sha256sums_x86_64 = $x86_64_checksum

pkgname = $BINARY_PKGNAME
EOF
}

validate_pkgbuilds() {
    log_step "Validating PKGBUILDs"
    
    if ! command -v makepkg &> /dev/null; then
        log_warning "makepkg not available, skipping validation"
        return
    fi
    
    local validation_failed=false
    
    if [[ "$GENERATE_SOURCE" == true ]]; then
        if validate_single_pkgbuild "$OUTPUT_DIR/PKGBUILD-source"; then
            log_success "Source PKGBUILD validation passed"
        else
            log_error "Source PKGBUILD validation failed"
            validation_failed=true
        fi
    fi
    
    if [[ "$GENERATE_BINARY" == true ]]; then
        if validate_single_pkgbuild "$OUTPUT_DIR/PKGBUILD-bin"; then
            log_success "Binary PKGBUILD validation passed"
        else
            log_error "Binary PKGBUILD validation failed"
            validation_failed=true
        fi
    fi
    
    if [[ "$validation_failed" == true ]]; then
        log_error "PKGBUILD validation failed"
        exit 1
    fi
}

validate_single_pkgbuild() {
    local pkgbuild_file="$1"
    local temp_dir
    temp_dir=$(mktemp -d)
    
    cp "$pkgbuild_file" "$temp_dir/PKGBUILD"
    
    (
        cd "$temp_dir"
        # Run makepkg in check mode without building
        if makepkg --printsrcinfo > /dev/null 2>&1; then
            exit 0
        else
            exit 1
        fi
    )
    
    local result=$?
    rm -rf "$temp_dir"
    return $result
}

setup_output_directory() {
    log_step "Setting up output directory"
    
    if [[ -d "$OUTPUT_DIR" ]]; then
        if [[ "$FORCE_OVERWRITE" != true ]]; then
            log_error "Output directory already exists: $OUTPUT_DIR"
            log_info "Use --force to overwrite existing files"
            exit 1
        fi
        rm -rf "$OUTPUT_DIR"
    fi
    
    mkdir -p "$OUTPUT_DIR"
    log_info "Output directory: $OUTPUT_DIR"
}

generate_summary() {
    cat << EOF

AUR PKGBUILD Generation Complete
================================

Package Information:
  • Source Package: $SOURCE_PKGNAME
  • Binary Package: $BINARY_PKGNAME  
  • Version: $VERSION
  • License: $LICENSE

Generated Files:
EOF

    if [[ "$GENERATE_SOURCE" == true ]]; then
        echo "  • PKGBUILD-source     (Source package PKGBUILD)"
        echo "  • .SRCINFO-source     (Source package metadata)"
    fi
    
    if [[ "$GENERATE_BINARY" == true ]]; then
        echo "  • PKGBUILD-bin        (Binary package PKGBUILD)"
        echo "  • .SRCINFO-bin        (Binary package metadata)"
    fi
    
    cat << EOF

Output Directory: $OUTPUT_DIR

Next Steps:
  1. Review generated PKGBUILD files
  2. Test build locally: makepkg -si --nocheck
  3. Submit to AUR: https://aur.archlinux.org/submit/

EOF
}

main() {
    log_info "Starting AUR PKGBUILD generation for awesome-omarchy-tui"
    
    parse_arguments "$@"
    check_dependencies
    extract_version
    extract_metadata
    setup_output_directory
    
    if [[ "$GENERATE_SOURCE" == true ]]; then
        generate_source_pkgbuild
        generate_srcinfo "source"
    fi
    
    if [[ "$GENERATE_BINARY" == true ]]; then
        generate_binary_pkgbuild
        generate_srcinfo "bin"
    fi
    
    if [[ "$VALIDATE_PKGBUILDS" == true ]]; then
        validate_pkgbuilds
    fi
    
    generate_summary
    
    log_success "PKGBUILD generation completed successfully"
}

main "$@"