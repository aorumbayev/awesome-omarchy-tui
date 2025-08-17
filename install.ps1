# awsomarchy Installer for Windows
# This script installs awsomarchy on your Windows system with SHA256 verification

# Parameters
param(
    [string]$Version = "",
    [string]$InstallDir = "$env:LOCALAPPDATA\Programs\awsomarchy",
    [switch]$Help
)

# Help information
if ($Help) {
    Write-Host "Usage: .\install.ps1 [-Version <version>] [-InstallDir <directory>] [-Help]"
    Write-Host "Install awsomarchy on your Windows system."
    Write-Host ""
    Write-Host "Options:"
    Write-Host "  -Version <version>     Specify version to install (defaults to latest)"
    Write-Host "  -InstallDir <dir>      Installation directory (default: $env:LOCALAPPDATA\Programs\awsomarchy)"
    Write-Host "  -Help                  Display this help and exit"
    Write-Host ""
    Write-Host "Installation Directory Selection:"
    Write-Host "  The default directory is a user-local location that doesn't require administrator rights:"
    Write-Host "  $env:LOCALAPPDATA\Programs\awsomarchy"
    Write-Host ""
    Write-Host "  If you prefer a system-wide installation, run PowerShell as Administrator and specify:"
    Write-Host "  -InstallDir `"C:\Program Files\awsomarchy`""
    Write-Host ""
    Write-Host "Security Features:"
    Write-Host "  - SHA256 integrity verification of downloaded binaries"
    Write-Host "  - Secure HTTPS downloads from GitHub releases"
    Write-Host "  - Pre-download availability verification"
    Write-Host ""
    Write-Host "Examples:"
    Write-Host "  .\install.ps1                                    # Install latest version to user directory"
    Write-Host "  .\install.ps1 -Version 1.0.0                   # Install specific version"
    Write-Host "  .\install.ps1 -InstallDir `"C:\tools\awsomarchy`"  # Install to custom directory"
    Write-Host ""
    Write-Host "Troubleshooting:"
    Write-Host "  If you encounter execution policy issues, run:"
    Write-Host "  Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser"
    exit 0
}

# Function to print errors with enhanced Windows-specific guidance
function Write-Error-Exit {
    param([string]$Message, [string]$Category = "General")
    
    Write-Host "Error: $Message" -ForegroundColor Red
    
    # Provide Windows-specific troubleshooting guidance based on error category
    switch ($Category) {
        "Network" {
            Write-Host ""
            Write-Host "Network Troubleshooting:" -ForegroundColor Yellow
            Write-Host "  - Check your internet connection"
            Write-Host "  - Verify you can access github.com in your browser"
            Write-Host "  - If behind a corporate firewall, contact your IT administrator"
            Write-Host "  - Try again in a few minutes if GitHub is experiencing issues"
        }
        "Permission" {
            Write-Host ""
            Write-Host "Permission Troubleshooting:" -ForegroundColor Yellow
            Write-Host "  - Run PowerShell as Administrator for system-wide installation"
            Write-Host "  - Or choose a user-writable directory: -InstallDir `"$env:USERPROFILE\tools\awsomarchy`""
            Write-Host "  - Check if antivirus software is blocking the installation"
            Write-Host "  - Ensure the directory path exists and is accessible"
        }
        "Security" {
            Write-Host ""
            Write-Host "Security Error:" -ForegroundColor Red
            Write-Host "  - Binary integrity verification failed"
            Write-Host "  - This could indicate file corruption or a security issue"
            Write-Host "  - DO NOT ignore this error"
            Write-Host "  - Try downloading again or contact the maintainers"
        }
        "Availability" {
            Write-Host ""
            Write-Host "Release Availability:" -ForegroundColor Yellow
            Write-Host "  - The release was just published and binaries may still be uploading"
            Write-Host "  - Please wait a few minutes and try again"
            Write-Host "  - Check https://github.com/aorumbayev/awesome-omarchy-tui/releases for status"
        }
    }
    
    exit 1
}

# Function to convert binary URL to SHA256 file URL
function Get-Sha256Url {
    param([string]$BinaryUrl)
    
    # Replace .zip extension with .sha256
    $sha256Url = $BinaryUrl -replace '\.zip$', '.sha256'
    return $sha256Url
}

# Function to test binary availability before download
function Test-BinaryAvailability {
    param([string]$BinaryUrl, [string]$Sha256Url)
    
    try {
        # Check binary availability with HEAD request
        Write-Info "Verifying binary and hash file availability..."
        
        $binaryResponse = Invoke-WebRequest -Uri $BinaryUrl -Method Head -UseBasicParsing
        $contentLength = $binaryResponse.Headers.'Content-Length'
        if ($contentLength -is [array]) { $contentLength = $contentLength[0] }
        $binarySize = [math]::Round([int]$contentLength / 1MB, 1)
        Write-Success "✅ Binary verified ($binarySize MB)"
        
        # Check SHA256 file availability
        try {
            $sha256Response = Invoke-WebRequest -Uri $Sha256Url -Method Head -UseBasicParsing
            Write-Success "✅ SHA256 hash file verified - integrity validation will be performed"
            return $true
        }
        catch {
            Write-Warning "⚠️  SHA256 file not available - installation will proceed without integrity verification"
            Write-Warning "   Consider using a release that includes SHA256 files for enhanced security"
            return $false
        }
    }
    catch {
        if ($_.Exception.Response.StatusCode -eq 404) {
            Write-Error-Exit "Binary not found at $BinaryUrl. The release may still be uploading." "Availability"
        }
        else {
            Write-Error-Exit "Failed to verify binary availability: $($_.Exception.Message)" "Network"
        }
    }
}

# Function to download and verify SHA256 hash
function Get-ExpectedHash {
    param([string]$Sha256Url)
    
    try {
        Write-Info "🔍 Downloading SHA256 hash file..."
        $sha256Content = Invoke-WebRequest -Uri $Sha256Url -UseBasicParsing | Select-Object -ExpandProperty Content
        
        # Parse hash from content (format: "hash *filename" or "hash  filename")
        $hashLine = $sha256Content.Trim()
        $hash = ($hashLine -split '\s+')[0]
        
        # Validate hash format (64 hex characters)
        if ($hash -match '^[a-fA-F0-9]{64}$') {
            Write-Success "✅ SHA256 hash retrieved: $($hash.Substring(0,16))..."
            return $hash.ToLower()
        }
        else {
            throw "Invalid SHA256 hash format: $hash"
        }
    }
    catch {
        Write-Error-Exit "Failed to download or parse SHA256 hash: $($_.Exception.Message)" "Network"
    }
}

# Function to verify file SHA256 hash
function Verify-FileSha256 {
    param([string]$FilePath, [string]$ExpectedHash)
    
    try {
        Write-Info "🔐 Verifying file integrity..."
        Write-Info "   Expected: $($ExpectedHash.Substring(0,16))..."
        
        $computedHash = (Get-FileHash -Path $FilePath -Algorithm SHA256).Hash.ToLower()
        Write-Info "   Computed: $($computedHash.Substring(0,16))..."
        
        if ($computedHash -eq $ExpectedHash) {
            Write-Success "✅ File integrity verified successfully!"
            return $true
        }
        else {
            Write-Error-Exit "SHA256 hash mismatch! Expected: $ExpectedHash, Got: $computedHash" "Security"
            return $false
        }
    }
    catch {
        Write-Error-Exit "Failed to compute file hash: $($_.Exception.Message)" "General"
    }
}

# Function to print messages
function Write-Info {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Blue
}

function Write-Success {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Green
}

# Create installation directory if it doesn't exist
if (-not (Test-Path -Path $InstallDir)) {
    Write-Info "Creating installation directory: $InstallDir"
    try {
        New-Item -Path $InstallDir -ItemType Directory -Force | Out-Null
    }
    catch {
        Write-Error-Exit "Failed to create directory $InstallDir. $($_.Exception.Message)" "Permission"
    }
}

# Check if the directory is writable
try {
    $testFile = Join-Path -Path $InstallDir -ChildPath "write_test"
    New-Item -Path $testFile -ItemType File -Force | Out-Null
    Remove-Item -Path $testFile -Force
}
catch {
    Write-Error-Exit "Cannot write to $InstallDir. Please ensure the directory exists and you have write permissions." "Permission"
}

# Detect architecture (only supporting x86_64 for Windows for now)
$arch = "x86_64"

# Set OS for Windows
$os = "pc-windows-msvc"

# Get latest version from GitHub if not specified
if (-not $Version) {
    Write-Info "Determining latest version..."
    try {
        $releaseData = Invoke-RestMethod -Uri "https://api.github.com/repos/aorumbayev/awesome-omarchy-tui/releases/latest"
        $Version = $releaseData.tag_name

        if (-not $Version) {
            Write-Error-Exit "Failed to determine latest version" "Network"
        }
    }
    catch {
        Write-Error-Exit "Failed to fetch latest version information. $($_.Exception.Message)" "Network"
    }
}

# Remove 'v' prefix if present (normalize version for both API and parameter input)
$Version = $Version -replace '^v', ''

Write-Info "Installing awsomarchy $Version for $arch-$os..."

# Construct package name and download URLs
$binaryName = "awsomarchy"
$pkgName = "$binaryName-$arch-$os.zip"
$downloadUrl = "https://github.com/aorumbayev/awesome-omarchy-tui/releases/download/v$Version/$pkgName"
$sha256Url = Get-Sha256Url -BinaryUrl $downloadUrl

# Pre-download verification
$hasSha256 = Test-BinaryAvailability -BinaryUrl $downloadUrl -Sha256Url $sha256Url

# Get expected hash if SHA256 file is available
$expectedHash = $null
if ($hasSha256) {
    $expectedHash = Get-ExpectedHash -Sha256Url $sha256Url
}

# Create temp directory
$tempDir = [System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid().ToString()
New-Item -Path $tempDir -ItemType Directory -Force | Out-Null

try {
    # Download the package
    Write-Info "⬇️  Downloading from $downloadUrl..."
    $downloadPath = Join-Path -Path $tempDir -ChildPath $pkgName

    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $downloadPath
    }
    catch {
        Write-Error-Exit "Download failed. Check URL or network. $($_.Exception.Message)" "Network"
    }

    # Verify SHA256 hash if available
    if ($expectedHash) {
        if (-not (Verify-FileSha256 -FilePath $downloadPath -ExpectedHash $expectedHash)) {
            # Error handling is done inside Verify-FileSha256
            return
        }
        Write-Success "🔐 Hash verification passed - proceeding with installation..."
    }
    else {
        Write-Warning "⚠️  Installing without SHA256 verification (hash file not available)"
    }

    # Extract the archive
    Write-Info "📦 Extracting archive..."
    try {
        Expand-Archive -Path $downloadPath -DestinationPath $tempDir -Force
    }
    catch {
        Write-Error-Exit "Failed to extract archive. $($_.Exception.Message)" "General"
    }

    # Check if binary exists after extraction
    $exeName = "$binaryName.exe"
    $extractedExe = Join-Path -Path $tempDir -ChildPath $exeName

    if (-not (Test-Path -Path $extractedExe)) {
        Write-Error-Exit "Binary '$exeName' not found in the archive." "General"
    }

    # Install the binary
    Write-Info "📋 Installing $exeName to $InstallDir..."

    try {
        Copy-Item -Path $extractedExe -Destination $InstallDir -Force
    }
    catch {
        Write-Error-Exit "Failed to copy binary to installation directory. $($_.Exception.Message)" "Permission"
    }

    # Add to PATH if needed
    $exePath = Join-Path -Path $InstallDir -ChildPath $exeName
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")

    if ($userPath -notlike "*$InstallDir*") {
        Write-Info "🛤️  Adding $InstallDir to your PATH..."

        try {
            [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
            # Update current session PATH
            $env:Path = "$env:Path;$InstallDir"
        }
        catch {
            Write-Warning "Failed to update PATH environment variable. You may need to add $InstallDir to your PATH manually."
        }
    }

    Write-Success "✅ $binaryName $Version has been installed to $InstallDir\$exeName"
    if ($expectedHash) {
        Write-Success "🔐 Installation completed with verified integrity!"
    }
    Write-Success "🚀 Run '$binaryName' to get started browsing the awesome-omarchy repository!"

}
finally {
    # Clean up
    if (Test-Path -Path $tempDir) {
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}