# Downloads the installers bundled into the ONYKS Chalcedon setup into src-tauri/redist.
# Run from the repository root before `pnpm tauri build`.
param(
    [string]$SvnUrl = 'https://sourceforge.net/projects/tortoisesvn/files/latest/download',
    [string]$TailscaleUrl = 'https://pkgs.tailscale.com/stable/tailscale-setup-latest-amd64.msi',
    [string]$PsqlOdbcUrl = 'https://ftp.postgresql.org/pub/odbc/releases/REL-18_00_0005/psqlodbc_x64.msi',
    [string]$PythonVersion = '3.12.10'
)

$ErrorActionPreference = 'Stop'
$target = Join-Path $PSScriptRoot '..\src-tauri\redist'
New-Item -ItemType Directory -Force $target | Out-Null

function Get-Installer($url, $name) {
    $path = Join-Path $target $name
    Write-Host "Downloading $name from $url"
    # curl.exe (shipped with Windows 10+): SourceForge answers 403 to browser-like
    # User-Agents such as the one Invoke-WebRequest sends, but serves curl's.
    & curl.exe --fail --location --silent --show-error --output $path $url
    if ($LASTEXITCODE -ne 0) { throw "Download of $name failed (curl exit code $LASTEXITCODE)" }
    if ((Get-Item $path).Length -lt 1MB) { throw "$name looks too small - download failed?" }
}

Get-Installer $SvnUrl 'svn.msi'
Get-Installer $TailscaleUrl 'tailscale.msi'
Get-Installer $PsqlOdbcUrl 'psqlodbc_x64.msi'
# Embeddable Python for the Convert tab (the converter uses only the standard library).
$python = Join-Path $target 'python'
$zip = Join-Path $target 'python-embed.zip'
Write-Host "Downloading Python $PythonVersion (embeddable)"
& curl.exe --fail --location --silent --show-error --output $zip "https://www.python.org/ftp/python/$PythonVersion/python-$PythonVersion-embed-amd64.zip"
if ($LASTEXITCODE -ne 0) { throw "Download of Python failed (curl exit code $LASTEXITCODE)" }
Remove-Item -Recurse -Force $python -ErrorAction SilentlyContinue
Expand-Archive -Path $zip -DestinationPath $python
Remove-Item $zip
# The embeddable Python ignores the script's folder; let it see the converter, installed at ..\..\converter.
$pth = Get-ChildItem $python -Filter 'python*._pth' | Select-Object -First 1
Add-Content -Path $pth.FullName -Value '../../converter'
Write-Host 'Done.'
