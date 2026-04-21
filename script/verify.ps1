$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

Write-Host "=== verify: 1/4 check-db-boundary.ps1 ==="
powershell -ExecutionPolicy Bypass -File "$repoRoot\script\check-db-boundary.ps1"
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

Write-Host "=== verify: 2/4 check-log-style.ps1 ==="
powershell -ExecutionPolicy Bypass -File "$repoRoot\script\check-log-style.ps1"
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

Write-Host "=== verify: 3/4 check-log-fields.ps1 ==="
powershell -ExecutionPolicy Bypass -File "$repoRoot\script\check-log-fields.ps1"
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

Write-Host "=== verify: 4/5 check-doc-drift.ps1 ==="
powershell -ExecutionPolicy Bypass -File "$repoRoot\script\check-doc-drift.ps1"
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

Write-Host "=== verify: 5/5 cargo check ==="
cargo check
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

Write-Host "=== verify passed ==="
