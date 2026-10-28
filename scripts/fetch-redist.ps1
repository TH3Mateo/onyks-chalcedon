# Downloads the installers bundled into the ONYKS Chalcedon setup into src-tauri/redist.
# Run from the repository root before `pnpm tauri build`.
param(
    [string]$SvnUrl = 'https://sourceforge.net/projects/tortoisesvn/files/latest/download',
    [string]$TailscaleUrl = 'https://pkgs.tailscale.com/stable/tailscale-setup-latest-amd64.msi',
    [string]$PsqlOdbcUrl = 'https://ftp.postgresql.org/pub/odbc/releases/REL-18_00_0005/psqlodbc_x64.msi'
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
Write-Host 'Done.'
