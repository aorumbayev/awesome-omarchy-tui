# AUR PKGBUILD Generator Scripts

This directory contains scripts for generating Arch User Repository (AUR) package files for the awesome-omarchy-tui project.

## Scripts

### `generate-aur-pkgbuild.sh`

A comprehensive script that generates compliant PKGBUILD files for both source and binary variants of awesome-omarchy-tui, following AUR submission guidelines.

#### Features

- **Dual Package Support**: Generates both source (`awesome-omarchy-tui`) and binary (`awesome-omarchy-tui-bin`) packages
- **AUR Compliance**: Follows all AUR submission guidelines and package standards
- **Automatic Version Detection**: Extracts version information from `Cargo.toml`
- **Dynamic Checksums**: Calculates SHA256 checksums for source and binary packages
- **Comprehensive Validation**: Includes PKGBUILD validation using makepkg
- **Flexible Output**: Configurable output directory and generation options
- **Error Handling**: Robust error handling and informative logging

#### Quick Start

```bash
# Generate both source and binary PKGBUILDs
./scripts/generate-aur-pkgbuild.sh

# Generate only source package
./scripts/generate-aur-pkgbuild.sh --source-only

# Generate to custom directory
./scripts/generate-aur-pkgbuild.sh --output ./my-aur-packages

# Use specific version
./scripts/generate-aur-pkgbuild.sh --version 1.0.0

# Generate and validate
./scripts/generate-aur-pkgbuild.sh --validate
```

#### Generated Files

The script generates the following files:

- `PKGBUILD-source`: Source package PKGBUILD for building from Rust source
- `PKGBUILD-bin`: Binary package PKGBUILD for pre-compiled binaries
- `.SRCINFO-source`: Source package metadata for AUR submission
- `.SRCINFO-bin`: Binary package metadata for AUR submission

#### AUR Package Details

**Source Package (`awesome-omarchy-tui`)**
- **Package Name**: `awesome-omarchy-tui`
- **Build Method**: Compiles from Rust source using cargo
- **Dependencies**: `rust`, `cargo`
- **Architecture**: `x86_64`
- **License**: `MIT`

**Binary Package (`awesome-omarchy-tui-bin`)**
- **Package Name**: `awesome-omarchy-tui-bin`
- **Build Method**: Installs pre-compiled binary from GitHub releases
- **Dependencies**: None (beyond system libraries)
- **Architecture**: `x86_64`
- **License**: `MIT`
- **Provides**: `awesome-omarchy-tui` (conflicts with source package)

#### Usage Examples

```bash
# Basic usage - generate both packages
./scripts/generate-aur-pkgbuild.sh

# Generate for testing without network calls
./scripts/generate-aur-pkgbuild.sh --no-checksums

# Generate source package only with validation
./scripts/generate-aur-pkgbuild.sh --source-only --validate

# Force overwrite existing files
./scripts/generate-aur-pkgbuild.sh --force

# Custom output directory
./scripts/generate-aur-pkgbuild.sh --output /path/to/aur-packages
```

#### Testing Generated PKGBUILDs

After generation, you can test the PKGBUILDs locally:

```bash
# Navigate to output directory
cd aur-pkgbuilds

# Test source package build
cp PKGBUILD-source PKGBUILD
makepkg -si --nocheck

# Or test binary package
cp PKGBUILD-bin PKGBUILD
makepkg -si --nocheck
```

#### AUR Submission Workflow

1. **Generate PKGBUILDs**: Run the script to generate current package files
2. **Review Files**: Check generated PKGBUILDs and .SRCINFO files
3. **Test Locally**: Build packages locally to verify functionality
4. **Submit to AUR**: Upload to AUR using git or web interface

```bash
# Example AUR submission workflow
./scripts/generate-aur-pkgbuild.sh --validate
cd aur-pkgbuilds

# Test source package
cp PKGBUILD-source PKGBUILD
cp .SRCINFO-source .SRCINFO
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "Update to version X.Y.Z"
git push
```

#### Integration with CI/CD

The script is designed to work with GitHub Actions and other CI/CD systems:

```yaml
- name: Generate AUR PKGBUILDs
  run: |
    ./scripts/generate-aur-pkgbuild.sh --validate
    
- name: Upload to AUR
  run: |
    # Custom logic for AUR submission
```

#### Dependencies

- **Required**: `bash`, `curl`, `sha256sum`
- **Optional**: `makepkg` (for validation and .SRCINFO generation)

#### Troubleshooting

**Common Issues:**

1. **Version extraction fails**: Ensure `Cargo.toml` exists and has proper version format
2. **Checksum calculation fails**: Check network connectivity and release availability
3. **Validation fails**: Install `base-devel` package group for makepkg

**Debug Mode:**

Use `bash -x` to run the script in debug mode:

```bash
bash -x ./scripts/generate-aur-pkgbuild.sh
```

#### Architecture Notes

The script currently supports `x86_64` architecture. Additional architectures can be added by:

1. Updating the `arch` array in PKGBUILDs
2. Adding checksum calculation for additional architectures
3. Updating the binary download URLs

#### License

This script is part of the awesome-omarchy-tui project and is licensed under the MIT License.