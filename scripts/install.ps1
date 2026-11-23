<#
.SYNOPSIS
    GitVerse Cross-Platform One-Line Installer for Windows (PowerShell)

.DESCRIPTION
    Installs, updates, or reinstalls the GitVerse desktop client on Windows.

.PARAMETER Version
    Specific release version tag to install (e.g. v1.0.0).

.PARAMETER Force
    Bypass interactive confirmation prompts.

.PARAMETER Reinstall
    Force reinstallation even if the latest version is already installed.

.PARAMETER Uninstall
    Remove GitVerse from the system.

.EXAMPLE
    irm https://raw.githubusercontent.com/farshidrezaei/gitverse/main/scripts/install.ps1 | iex
    install.ps1 -Force
    install.ps1 -Version v1.0.0
    install.ps1 -Uninstall
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory = $false)]
    [string]$Version = "",

    [Parameter(Mandatory = $false)]
    [switch]$Force,

    [Parameter(Mandatory = $false)]
    [switch]$Reinstall,

    [Parameter(Mandatory = $false)]
    [switch]$Uninstall,

    [Parameter(Mandatory = $false)]
    [switch]$Help
)

$ErrorActionPreference = "Stop"

$RepositoryOwner = "farshidrezaei"
$RepositoryName = "gitverse"
$GitHubRepository = "$RepositoryOwner/$RepositoryName"

function Show-Banner {
    Write-Host ""
    Write-Host "   ____ _ _   _    __               " -ForegroundColor Cyan
    Write-Host "  / ___(_) |_| |  / /__ _ __ ___  ___ " -ForegroundColor Cyan
    Write-Host " | |  _| | __| | / / _ \ '__/ __|/ _ \" -ForegroundColor Cyan
    Write-Host " | |_| | | |_| |/ /  __/ |  \__ \  __/" -ForegroundColor Cyan
    Write-Host "  \____|_|\__|_/_/ \___|_|  |___/\___|" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "GitVerse Desktop Client Installer for Windows" -ForegroundColor White
    Write-Host "Fast, visual, local-first Git client" -ForegroundColor Gray
    Write-Host ""
}

function Show-Help {
    Write-Host "GitVerse Windows Installer"
    Write-Host ""
    Write-Host "Usage:"
    Write-Host "  install.ps1 [-Version <tag>] [-Force] [-Reinstall] [-Uninstall]"
    Write-Host ""
    Write-Host "Parameters:"
    Write-Host "  -Version <tag>   Install a specific version tag (e.g., v1.0.0)"
    Write-Host "  -Force           Bypass interactive confirmation prompts"
    Write-Host "  -Reinstall       Reinstall GitVerse even if latest version is already present"
    Write-Host "  -Uninstall       Remove GitVerse desktop application"
    Write-Host "  -Help            Show this help documentation"
    Write-Host ""
    Write-Host "One-liner:"
    Write-Host '  irm https://raw.githubusercontent.com/farshidrezaei/gitverse/main/scripts/install.ps1 | iex'
}

function Write-InfoLog([string]$messageText) {
    Write-Host "==> " -ForegroundColor Blue -NoNewline
    Write-Host $messageText
}

function Write-SuccessLog([string]$messageText) {
    Write-Host "==> " -ForegroundColor Green -NoNewline
    Write-Host $messageText
}

function Write-WarningLog([string]$messageText) {
    Write-Host "Warning: " -ForegroundColor Yellow -NoNewline
    Write-Host $messageText
}

function Write-ErrorLog([string]$messageText) {
    Write-Host "Error: " -ForegroundColor Red -NoNewline
    Write-Host $messageText
}

function Clean-VersionString([string]$rawVersion) {
    if ($rawVersion.StartsWith("v")) {
        return $rawVersion.Substring(1)
    }
    return $rawVersion
}

function Compare-ApplicationVersions([string]$installedVersion, [string]$targetVersion) {
    $cleanInstalled = Clean-VersionString $installedVersion
    $cleanTarget = Clean-VersionString $targetVersion

    try {
        $parsedInstalled = [System.Version]::Parse($cleanInstalled)
        $parsedTarget = [System.Version]::Parse($cleanTarget)

        if ($parsedInstalled -lt $parsedTarget) {
            return "older"
        }
        elseif ($parsedInstalled -gt $parsedTarget) {
            return "newer"
        }
        else {
            return "equal"
        }
    }
    catch {
        if ($cleanInstalled -eq $cleanTarget) {
            return "equal"
        }
        return "older"
    }
}

function Get-InstalledGitVerseVersion {
    # 1. Check default local app data folder
    $standardPath = Join-Path $env:LOCALAPPDATA "Programs\GitVerse\GitVerse.exe"
    if (Test-Path $standardPath) {
        $fileVersion = (Get-Item $standardPath).VersionInfo.ProductVersion
        if (-not [string]::IsNullOrWhiteSpace($fileVersion)) {
            return $fileVersion.Trim()
        }
    }

    # 2. Check registry uninstall keys (64-bit and 32-bit views)
    $registryPaths = @(
        "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*",
        "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*",
        "HKLM:\Software\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\*"
    )

    foreach ($registryPath in $registryPaths) {
        try {
            $installedItems = Get-ItemProperty $registryPath -ErrorAction SilentlyContinue
            foreach ($item in $installedItems) {
                if ($item.DisplayName -match "GitVerse" -and $item.DisplayVersion) {
                    return $item.DisplayVersion.Trim()
                }
            }
        }
        catch {
            # Continue checking next path
        }
    }

    # 3. Check CLI command in PATH
    $cliCommand = Get-Command "gitverse" -ErrorAction SilentlyContinue
    if ($cliCommand) {
        try {
            $cliVersion = & gitverse --version 2>$null
            if ($cliVersion -match '[0-9]+\.[0-9]+\.[0-9]+') {
                return $matches[0]
            }
        }
        catch {}
    }

    return $null
}

function Prompt-UserChoice([string]$promptMessage, [string]$defaultChoice = "N") {
    if ($Force) {
        return $true
    }

    $formattedPrompt = "$promptMessage "
    $userResponse = Read-Host -Prompt $formattedPrompt
    if ([string]::IsNullOrWhiteSpace($userResponse)) {
        $userResponse = $defaultChoice
    }

    return ($userResponse -match '^[Yy]')
}

function Fetch-GitHubReleaseData([string]$requestedTag) {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

    $apiUri = if ([string]::IsNullOrWhiteSpace($requestedTag)) {
        "https://api.github.com/repos/$GitHubRepository/releases/latest"
    } else {
        if (-not $requestedTag.StartsWith("v")) {
            $requestedTag = "v$requestedTag"
        }
        "https://api.github.com/repos/$GitHubRepository/releases/tags/$requestedTag"
    }

    $requestHeaders = @{
        "User-Agent" = "GitVerse-PowerShell-Installer"
        "Accept"     = "application/vnd.github+json"
    }

    if ($env:GITHUB_TOKEN) {
        $requestHeaders["Authorization"] = "Bearer $env:GITHUB_TOKEN"
    }

    try {
        $responseObject = Invoke-RestMethod -Uri $apiUri -Headers $requestHeaders -Method Get
        return $responseObject
    }
    catch {
        Write-ErrorLog "Failed to retrieve release metadata from GitHub ($apiUri): $_"
        exit 1
    }
}

function Uninstall-GitVerseInstallation {
    Write-InfoLog "Searching for GitVerse uninstaller..."

    $uninstallKeys = @(
        "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*",
        "HKLM:\Software\Microsoft\Windows\CurrentVersion\Uninstall\*"
    )

    $hasUninstalled = $false
    foreach ($keyPath in $uninstallKeys) {
        try {
            $installedItems = Get-ItemProperty $keyPath -ErrorAction SilentlyContinue
            foreach ($item in $installedItems) {
                if ($item.DisplayName -match "GitVerse" -and $item.UninstallString) {
                    Write-InfoLog "Running uninstaller: $($item.UninstallString)"
                    Start-Process -FilePath $item.UninstallString -ArgumentList "/S" -Wait
                    $hasUninstalled = $true
                    break
                }
            }
        }
        catch {}
    }

    # Also clean up local program files if left behind
    $installDirectory = Join-Path $env:LOCALAPPDATA "Programs\GitVerse"
    if (Test-Path $installDirectory) {
        Remove-Item -Path $installDirectory -Recurse -Force -ErrorAction SilentlyContinue
        $hasUninstalled = $true
    }

    if ($hasUninstalled) {
        Write-SuccessLog "GitVerse uninstalled successfully."
    }
    else {
        Write-WarningLog "No existing GitVerse installation found to remove."
    }
}

function Main {
    if ($Help) {
        Show-Help
        return
    }

    Show-Banner

    if ($Uninstall) {
        Uninstall-GitVerseInstallation
        return
    }

    # 1. Check architecture
    $architecture = if ([Environment]::Is64BitOperatingSystem) { "x64" } else { "x86" }
    Write-InfoLog "Detected Windows architecture: $architecture"

    # 2. Fetch release data
    Write-InfoLog "Querying GitHub release information..."
    $releaseData = Fetch-GitHubReleaseData $Version

    $targetTag = $releaseData.tag_name
    $targetVersion = Clean-VersionString $targetTag
    Write-InfoLog "Target release version: v$targetVersion"

    # 3. Check existing local installation
    $installedVersion = Get-InstalledGitVerseVersion

    if ($installedVersion) {
        $versionComparison = Compare-ApplicationVersions $installedVersion $targetVersion

        if ($versionComparison -eq "equal") {
            Write-Host ""
            Write-Host "GitVerse is already installed and up to date (v$installedVersion)." -ForegroundColor Green

            if ($Reinstall) {
                Write-InfoLog "Reinstalling GitVerse v$targetVersion as requested..."
            }
            else {
                $shouldReinstall = Prompt-UserChoice "Would you like to reinstall GitVerse v$targetVersion? [y/N]:" "N"
                if (-not $shouldReinstall) {
                    Write-InfoLog "Installation aborted by user. Exiting."
                    return
                }
                Write-InfoLog "Reinstalling GitVerse v$targetVersion..."
            }
        }
        elseif ($versionComparison -eq "older") {
            Write-Host ""
            Write-Host "Update Available!" -ForegroundColor Yellow
            Write-Host "Installed version: v$installedVersion"
            Write-Host "Latest version:    v$targetVersion"
            Write-Host ""

            $shouldUpdate = Prompt-UserChoice "Would you like to update GitVerse to v$targetVersion now? [Y/n]:" "Y"
            if (-not $shouldUpdate) {
                Write-InfoLog "Update cancelled by user. Exiting."
                return
            }
            Write-InfoLog "Updating GitVerse from v$installedVersion to v$targetVersion..."
        }
        else {
            Write-Host ""
            Write-WarningLog "Installed version (v$installedVersion) is newer than target release (v$targetVersion)."
            $shouldDowngrade = Prompt-UserChoice "Do you want to downgrade to v$targetVersion? [y/N]:" "N"
            if (-not $shouldDowngrade) {
                Write-InfoLog "Operation cancelled by user. Exiting."
                return
            }
        }
    }
    else {
        Write-InfoLog "No existing GitVerse installation detected."
        Write-InfoLog "Installing GitVerse v$targetVersion..."
    }

    # 4. Find Windows installer asset (.exe preferred, or .msi), ignoring signature files
    $asset = $releaseData.assets | Where-Object { 
        ($_.name -like "*setup.exe" -or $_.name -like "*x64*.exe") -and -not $_.name.EndsWith(".sig") 
    } | Select-Object -First 1

    if (-not $asset) {
        $asset = $releaseData.assets | Where-Object { 
            $_.name -like "*.msi" -and -not $_.name.EndsWith(".sig") 
        } | Select-Object -First 1
    }

    if (-not $asset) {
        Write-ErrorLog "Could not find a compatible Windows installer asset in release v$targetVersion."
        exit 1
    }

    $downloadUrl = $asset.browser_download_url
    $installerFileName = $asset.name
    $temporaryDirectory = [System.IO.Path]::GetTempPath()
    $localInstallerPath = Join-Path $temporaryDirectory $installerFileName

    Write-InfoLog "Downloading $installerFileName..."
    Write-Host "URL: $downloadUrl" -ForegroundColor Gray

    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $localInstallerPath -UseBasicParsing
    }
    catch {
        Write-ErrorLog "Failed to download installer: $_"
        exit 1
    }

    Write-InfoLog "Running installer..."

    try {
        if ($installerFileName.EndsWith(".exe")) {
            # Run NSIS installer silently
            $installerProcess = Start-Process -FilePath $localInstallerPath -ArgumentList "/S" -PassThru -Wait
            if ($installerProcess.ExitCode -ne 0) {
                Write-WarningLog "Installer process exited with code: $($installerProcess.ExitCode)"
            }
        }
        elseif ($installerFileName.EndsWith(".msi")) {
            $msiArguments = "/i `"$localInstallerPath`" /qn /norestart"
            $installerProcess = Start-Process -FilePath "msiexec.exe" -ArgumentList $msiArguments -PassThru -Wait
            if ($installerProcess.ExitCode -ne 0) {
                Write-WarningLog "msiexec exited with code: $($installerProcess.ExitCode)"
            }
        }
    }
    finally {
        # Clean up temporary installer
        Remove-Item -Path $localInstallerPath -Force -ErrorAction SilentlyContinue
    }

    # Ensure PATH contains install directory
    $defaultInstallPath = Join-Path $env:LOCALAPPDATA "Programs\GitVerse"
    $currentUserPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($currentUserPath -notlike "*$defaultInstallPath*") {
        try {
            [Environment]::SetEnvironmentVariable("Path", "$currentUserPath;$defaultInstallPath", "User")
            Write-InfoLog "Added $defaultInstallPath to user PATH environment variable."
        }
        catch {
            Write-WarningLog "Could not update user PATH automatically. You can manually add $defaultInstallPath to PATH."
        }
    }

    Write-Host ""
    Write-SuccessLog "GitVerse v$targetVersion installed successfully!"
    Write-Host "  • Location: $defaultInstallPath\GitVerse.exe" -ForegroundColor Gray
    Write-Host ""
    Write-Host "Star GitVerse on GitHub: https://github.com/$GitHubRepository" -ForegroundColor Cyan
}

Main
