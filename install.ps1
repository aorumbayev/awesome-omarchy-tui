# awsomarchy Installer for Windows
# This script installs awsomarchy on your Windows system

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
    exit 0
}

# Function to print errors
function Write-Error-Exit {
    param([string]$Message)
    Write-Host "Error: $Message" -ForegroundColor Red
    exit 1
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
        Write-Error-Exit "Failed to create directory $InstallDir. $($_.Exception.Message)"
    }
}

# Check if the directory is writable
try {
    $testFile = Join-Path -Path $InstallDir -ChildPath "write_test"
    New-Item -Path $testFile -ItemType File -Force | Out-Null
    Remove-Item -Path $testFile -Force
}
catch {
    Write-Error-Exit "Cannot write to $InstallDir. Please ensure the directory exists and you have write permissions."
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
            Write-Error-Exit "Failed to determine latest version"
        }

        # Remove 'v' prefix if present
        $Version = $Version -replace '^v', ''
    }
    catch {
        Write-Error-Exit "Failed to fetch latest version information. $($_.Exception.Message)"
    }
}

Write-Info "Installing awsomarchy $Version for $arch-$os..."

# Construct package name and download URL
$binaryName = "awsomarchy"
$pkgName = "$binaryName-$arch-$os.zip"
$downloadUrl = "https://github.com/aorumbayev/awesome-omarchy-tui/releases/download/v$Version/$pkgName"

# Create temp directory
$tempDir = [System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid().ToString()
New-Item -Path $tempDir -ItemType Directory -Force | Out-Null

try {
    # Download the package
    Write-Info "Downloading from $downloadUrl..."
    $downloadPath = Join-Path -Path $tempDir -ChildPath $pkgName

    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $downloadPath
    }
    catch {
        Write-Error-Exit "Download failed. Check URL or network. $($_.Exception.Message)"
    }

    # Extract the archive
    Write-Info "Extracting archive..."
    try {
        Expand-Archive -Path $downloadPath -DestinationPath $tempDir -Force
    }
    catch {
        Write-Error-Exit "Failed to extract archive. $($_.Exception.Message)"
    }

    # Check if binary exists after extraction
    $exeName = "$binaryName.exe"
    $extractedExe = Join-Path -Path $tempDir -ChildPath $exeName

    if (-not (Test-Path -Path $extractedExe)) {
        Write-Error-Exit "Binary '$exeName' not found in the archive."
    }

    # Install the binary
    Write-Info "Installing $exeName to $InstallDir..."

    try {
        Copy-Item -Path $extractedExe -Destination $InstallDir -Force
    }
    catch {
        Write-Error-Exit "Failed to copy binary to installation directory. $($_.Exception.Message)"
    }

    # Add to PATH if needed
    $exePath = Join-Path -Path $InstallDir -ChildPath $exeName
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")

    if ($userPath -notlike "*$InstallDir*") {
        Write-Info "Adding $InstallDir to your PATH..."

        try {
            [Environment]::SetEnvironmentVariable("Path", "$userPath;$InstallDir", "User")
            # Update current session PATH
            $env:Path = "$env:Path;$InstallDir"
        }
        catch {
            Write-Warning "Failed to update PATH environment variable. You may need to add $InstallDir to your PATH manually."
        }
    }

    Write-Success "$binaryName $Version has been installed to $InstallDir\$exeName"
    Write-Success "Run '$binaryName' to get started browsing the awesome-omarchy repository!"

}
finally {
    # Clean up
    if (Test-Path -Path $tempDir) {
        Remove-Item -Path $tempDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}