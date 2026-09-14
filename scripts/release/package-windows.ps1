# Builds the v0.1 Windows release archive: the native binary plus the shared
# install note, wrapped in db-pro-v<version>-windows-x86_64.zip.
#
# Usage: package-windows.ps1 -Version <version> -BinaryPath <exe> -OutputDir <dir>
#
# Contract (goal-3 sections 8 and 10, V01-06 step 2):
#
#   db-pro-native.exe
#   README-INSTALL.txt
#
# The artifact is a portable zip: no .msi/NSIS installer is part of the v0.1
# contract, and the archive is unsigned. Files are added at the archive root
# (no directory entries), which also avoids the Windows PowerShell 5.1
# Compress-Archive backslash-separator issue for nested paths.
#
# Write-Host output goes to the log; the absolute path of the created archive
# is the only object written to the success output stream, so callers can
# capture it with $archive = & package-windows.ps1 ...
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Version,
    [Parameter(Mandatory = $true)][string]$BinaryPath,
    [Parameter(Mandatory = $true)][string]$OutputDir
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

if ($Version -notmatch '^[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.]+)?$') {
    throw "invalid version '$Version'"
}
if (-not (Test-Path -LiteralPath $BinaryPath -PathType Leaf)) {
    throw "binary not found: $BinaryPath"
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$installNote = Join-Path $scriptDir 'README-INSTALL.txt'
if (-not (Test-Path -LiteralPath $installNote -PathType Leaf)) {
    throw "install note not found: $installNote"
}

$staging = Join-Path ([System.IO.Path]::GetTempPath()) ('db-pro-package-' + [System.Guid]::NewGuid().ToString('N'))
try {
    New-Item -ItemType Directory -Path $staging -Force | Out-Null
    $stagedBinary = Join-Path $staging 'db-pro-native.exe'
    $stagedNote = Join-Path $staging 'README-INSTALL.txt'
    Copy-Item -LiteralPath $BinaryPath -Destination $stagedBinary -Force
    Copy-Item -LiteralPath $installNote -Destination $stagedNote -Force

    if (-not (Test-Path -LiteralPath $OutputDir -PathType Container)) {
        New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
    }
    $outputDirFull = (Resolve-Path -LiteralPath $OutputDir).Path
    $archive = Join-Path $outputDirFull "db-pro-v$Version-windows-x86_64.zip"
    if (Test-Path -LiteralPath $archive -PathType Leaf) {
        Remove-Item -LiteralPath $archive -Force
    }

    Compress-Archive -LiteralPath @($stagedBinary, $stagedNote) -DestinationPath $archive -CompressionLevel Optimal

    # The archive must contain exactly the contract layout and nothing else: no
    # LICENSE (the project has no license decision yet), no user state, no
    # build output.
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $allowed = @('db-pro-native.exe', 'README-INSTALL.txt')
    $zip = [System.IO.Compression.ZipFile]::OpenRead($archive)
    try {
        foreach ($entry in $zip.Entries) {
            if ($allowed -notcontains $entry.FullName) {
                throw "unexpected archive entry: $($entry.FullName)"
            }
            Write-Host ("  {0} ({1} bytes)" -f $entry.FullName, $entry.Length)
        }
        foreach ($required in $allowed) {
            if (-not ($zip.Entries | Where-Object { $_.FullName -eq $required })) {
                throw "archive entry is missing: $required"
            }
        }
    }
    finally {
        $zip.Dispose()
    }

    $size = (Get-Item -LiteralPath $archive).Length
    $hash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    Write-Host "Archive: $archive"
    Write-Host "Archive: $size bytes"
    Write-Host "SHA256:  $hash"

    Write-Output $archive
}
finally {
    if (Test-Path -LiteralPath $staging) {
        Remove-Item -LiteralPath $staging -Recurse -Force
    }
}
