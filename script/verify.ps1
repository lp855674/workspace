$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

Write-Host "=== verify: 1/2 check-db-boundary.ps1 ==="
powershell -ExecutionPolicy Bypass -File "$repoRoot\script\check-db-boundary.ps1"
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

Write-Host "=== verify: 2/2 cargo check ==="
cargo check
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

Write-Host "=== verify passed ==="
