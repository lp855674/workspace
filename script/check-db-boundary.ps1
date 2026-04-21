# DB-02: Only crates/db and crates/console may use sqlx; no SQL in other crates.
$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $RepoRoot

Write-Host "=== 1. Check cargo tree -i sqlx: only db and console may directly depend on sqlx ==="
$tree = cargo tree -i sqlx 2>$null
if (-not ($tree | Select-String "db v")) {
    Write-Host "ERROR: sqlx dependency tree does not show db as a direct dependent"
    cargo tree -i sqlx
    exit 1
}
if (-not ($tree | Select-String "console v")) {
    Write-Host "ERROR: sqlx dependency tree does not show console as a direct dependent"
    cargo tree -i sqlx
    exit 1
}
Write-Host "  OK: db and console are direct sqlx consumers"

Write-Host "=== 2. Scan crates (except db, console) for sqlx:: or sqlx usage ==="
$crates = Get-ChildItem -Path crates -Directory | Where-Object { $_.Name -ne "db" -and $_.Name -ne "console" }
$violations = @()
foreach ($c in $crates) {
    $m = Get-ChildItem -Path $c.FullName -Recurse -Filter "*.rs" | Select-String -Pattern "sqlx::|use sqlx|from sqlx"
    if ($m) { $violations += $m }
}
if ($violations.Count -gt 0) {
    Write-Host "ERROR: sqlx usage found outside crates/db and crates/console:"
    $violations | ForEach-Object { Write-Host $_ }
    exit 1
}
Write-Host "  OK: no sqlx usage outside db and console crates"

Write-Host "=== 3. Scan for SQL in crates other than db and console ==="
$sqlViolations = @()
foreach ($c in $crates) {
    $m = Get-ChildItem -Path $c.FullName -Recurse -Filter "*.rs" | Select-String -Pattern "sqlx::query|sqlx::query_as"
    if ($m) { $sqlViolations += $m }
}
if ($sqlViolations.Count -gt 0) {
    Write-Host "ERROR: SQL queries found outside crates/db and crates/console:"
    $sqlViolations | ForEach-Object { Write-Host $_ }
    exit 1
}
Write-Host "  OK: no SQL outside db and console crates"

Write-Host "=== DB boundary check passed ==="
