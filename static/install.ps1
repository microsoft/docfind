# docfind installer script for Windows
# Usage: irm https://microsoft.github.io/docfind/install.ps1 | iex

$ErrorActionPreference = 'Stop'

# Configuration
$Repo = "microsoft/docfind"
$BinaryName = "docfind"
$InstallDir = if ($env:DOCFIND_INSTALL_DIR) { $env:DOCFIND_INSTALL_DIR } else { "$env:USERPROFILE\.docfind\bin" }

# Helper functions
function Write-Info {
    param([string]$Message)
    Write-Host "==> " -ForegroundColor Green -NoNewline
    Write-Host $Message
}

function Write-Warn {
    param([string]$Message)
    Write-Host "Warning: " -ForegroundColor Yellow -NoNewline
    Write-Host $Message
}

function Write-Error-Custom {
    param([string]$Message)
    Write-Host "Error: " -ForegroundColor Red -NoNewline
    Write-Host $Message
    exit 1
}

# Detect architecture
function Get-Architecture {
    $arch = $env:PROCESSOR_ARCHITECTURE
    switch ($arch) {
        "AMD64" { return "x86_64" }
        "ARM64" { return "aarch64" }
        default { Write-Error-Custom "Unsupported architecture: $arch" }
    }
}

# Get the current installed version
function Get-CurrentVersion {
    try {
        # Check if docfind is in PATH and can be executed
        $currentVersionOutput = & $BinaryName --version 2>&1
        if ($LASTEXITCODE -eq 0 -and $currentVersionOutput) {
            # Extract version from "docfind X.Y.Z" output
            $versionMatch = $currentVersionOutput -match "^$BinaryName\s+(.+)$"
            if ($versionMatch -and $Matches[1]) {
                return $Matches[1].Trim()
            }
        }
    }
    catch {
        # Binary not found or not executable
    }
    return $null
}

# Get the latest release version
function Get-LatestVersion {
    Write-Info "Fetching latest release..."
    
    try {
        # Prepare headers for authentication if GITHUB_TOKEN is set
        $headers = @{}
        if ($env:GITHUB_TOKEN) {
            $headers["Authorization"] = "Bearer $env:GITHUB_TOKEN"
        }
        
        $response = if ($headers.Count -gt 0) {
            Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -Headers $headers
        } else {
            Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
        }
        
        $version = $response.tag_name
        
        if (-not $version) {
            Write-Error-Custom "Failed to fetch latest version"
        }
        
        Write-Info "Latest version: $version"
        return $version
    }
    catch {
        Write-Error-Custom "Failed to fetch release information: $_"
    }
}

# Download and install binary
function Install-Binary {
    param(
        [string]$Version,
        [string]$Target
    )
    
    $fileName = "${BinaryName}-${Target}.zip"
    $downloadUrl = "https://github.com/$Repo/releases/download/$Version/$fileName"
    $tempFile = Join-Path $env:TEMP $fileName
    $tempExtractDir = Join-Path $env:TEMP "docfind-extract"
    
    Write-Info "Downloading from $downloadUrl..."
    
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -UseBasicParsing
    }
    catch {
        Write-Error-Custom "Download failed: $_"
    }
    
    # Create install directory if it doesn't exist
    if (-not (Test-Path $InstallDir)) {
        Write-Info "Creating directory $InstallDir..."
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }
    
    # Extract archive
    Write-Info "Extracting archive..."
    try {
        # Clean up temp extract directory if it exists
        if (Test-Path $tempExtractDir) {
            Remove-Item -Path $tempExtractDir -Recurse -Force
        }
        New-Item -ItemType Directory -Path $tempExtractDir -Force | Out-Null
        
        Expand-Archive -Path $tempFile -DestinationPath $tempExtractDir -Force
    }
    catch {
        Write-Error-Custom "Failed to extract archive: $_"
    }
    
    # Install binary
    $destination = Join-Path $InstallDir "${BinaryName}.exe"
    $extractedBinary = Join-Path $tempExtractDir "${BinaryName}.exe"
    Write-Info "Installing to $destination..."
    
    try {
        Move-Item -Path $extractedBinary -Destination $destination -Force
    }
    catch {
        Write-Error-Custom "Failed to install binary: $_"
    }
    
    # Clean up
    try {
        Remove-Item -Path $tempFile -Force -ErrorAction SilentlyContinue
        Remove-Item -Path $tempExtractDir -Recurse -Force -ErrorAction SilentlyContinue
    }
    catch {
        # Ignore cleanup errors
    }
    
    Write-Info "Successfully installed $BinaryName to $InstallDir"
}

# Check if install directory is in PATH
function Test-InPath {
    param([string]$Directory)
    
    $pathDirs = $env:PATH -split ';'
    return $pathDirs -contains $Directory
}

# Add directory to PATH
function Add-ToPath {
    param([string]$Directory)
    
    Write-Info "Adding $Directory to your PATH..."
    
    try {
        # Get current user PATH
        $currentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
        
        if ($currentPath -notlike "*$Directory*") {
            $newPath = if ($currentPath) { "$currentPath;$Directory" } else { $Directory }
            [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
            
            # Update current session PATH
            $env:PATH = "$env:PATH;$Directory"
            
            Write-Info "Added $Directory to PATH"
            return $true
        }
        else {
            Write-Info "$Directory is already in PATH"
            return $false
        }
    }
    catch {
        Write-Warn "Failed to add to PATH automatically: $_"
        return $false
    }
}

# Print post-install instructions
function Show-PostInstall {
    param([bool]$PathUpdated)
    
    Write-Host ""
    Write-Info "Installation complete!"
    Write-Host ""
    
    if ($PathUpdated) {
        Write-Host "The installation directory has been added to your PATH."
        Write-Host "You may need to restart your terminal for the changes to take effect."
        Write-Host ""
        Write-Host "In a new terminal, you can run:" -ForegroundColor Cyan
        Write-Host "  $BinaryName --help" -ForegroundColor Green
    }
    else {
        if (-not (Test-InPath $InstallDir)) {
            Write-Warn "$InstallDir is not in your PATH"
            Write-Host ""
            Write-Host "To add it permanently, run this in an elevated PowerShell:" -ForegroundColor Cyan
            Write-Host "  [Environment]::SetEnvironmentVariable('PATH', `$env:PATH + ';$InstallDir', 'User')" -ForegroundColor Green
            Write-Host ""
            Write-Host "Or add it to your current session:" -ForegroundColor Cyan
            Write-Host "  `$env:PATH += ';$InstallDir'" -ForegroundColor Green
            Write-Host ""
        }
        else {
            Write-Host "You can now use '$BinaryName' from anywhere!" -ForegroundColor Cyan
            Write-Host ""
            Write-Host "Try it out:" -ForegroundColor Cyan
            Write-Host "  $BinaryName --help" -ForegroundColor Green
        }
    }
}

# Main installation flow
function Main {
    Write-Info "Installing $BinaryName..."
    
    $arch = Get-Architecture
    $target = "${arch}-pc-windows-msvc"
    Write-Info "Detected platform: $target"
    
    $version = Get-LatestVersion
    
    # Check if already installed with the same version
    $currentVersion = Get-CurrentVersion
    if ($currentVersion) {
        Write-Info "Current version: $currentVersion"
        # Strip 'v' prefix from version if present for comparison
        $latestVersionNum = $version -replace '^v', ''
        if ($currentVersion -eq $latestVersionNum -or $currentVersion -eq $version) {
            Write-Info "$BinaryName $currentVersion is already installed (latest version)"
            Write-Host ""
            Write-Host "If you want to reinstall, please uninstall first:" -ForegroundColor Cyan
            Write-Host "  Remove-Item (Get-Command $BinaryName).Path" -ForegroundColor Green
            exit 0
        }
    }
    
    Install-Binary -Version $version -Target $target
    
    $pathUpdated = $false
    if (-not (Test-InPath $InstallDir)) {
        $pathUpdated = Add-ToPath -Directory $InstallDir
    }
    
    Show-PostInstall -PathUpdated $pathUpdated
}

# Run the installer
try {
    Main
}
catch {
    Write-Error-Custom "Installation failed: $_"
}
