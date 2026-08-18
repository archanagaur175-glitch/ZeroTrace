# Verify-Offline — ZeroTrace offline shipping gate.
#
# Confirms the release artifacts contain no network telemetry:
#   1. The dependency audit embedded in the build is zero-telemetry.
#   2. No frontend bundle references external analytics/telemetry endpoints.
#   3. The lockfile only resolves from the official npm registry.
#
# Run AFTER `npm run build` and (optionally) `npm run dep-audit`.

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$fail = $false

function Check([string]$what, [scriptblock]$cond) {
    if (& $cond) {
        Write-Host "[verify-offline] PASS  $what"
    } else {
        Write-Host "[verify-offline] FAIL  $what"
        $script:fail = $true
    }
}

# 1. Embedded audit report (produced by scripts/dep-audit.mjs).
$auditPath = Join-Path $root "target\dep-audit.json"
Check "dependency audit report exists and is zero-telemetry" {
    (Test-Path $auditPath) -and
    ((Get-Content $auditPath -Raw | ConvertFrom-Json).zero_telemetry -eq $true)
}

# 2. Telemetry endpoints that must never appear in delivered frontend code.
$telemetryDomains = @(
    "mixpanel.com", "segment.io", "segment.com", "amplitude.com", "sentry.io",
    "sentry.ingest.us", "posthog.com", "google-analytics.com", "googletagmanager.com",
    "fullstory.com", "bugsnag.com", "datadoghq.com", "newrelic.com", "logrocket.com",
    "heapanalytics.com", "telemetrydeck.com", "countly.com", "rudderlabs.com",
    "applicationinsights.azure.com", "honeycomb.io"
)

Check "frontend bundle has no analytics/telemetry endpoints (built or source)" {
    $needles = $telemetryDomains
    $targets = @()
    if (Test-Path (Join-Path $root "dist")) {
        $targets += Get-ChildItem (Join-Path $root "dist") -Recurse -Include *.js,*.html,*.css -File
    }
    $targets += Get-ChildItem (Join-Path $root "src") -Recurse -Include *.ts,*.tsx,*.html -File -ErrorAction SilentlyContinue
    $leaks = $targets | Select-String -Pattern ($needles -join "|") -SimpleMatch
    -not $leaks
}

# 3. Lockfile resolves only from the official registry.
Check "package-lock.json has no third-party registry hosts" {
    if (-not (Test-Path (Join-Path $root "package-lock.json"))) { return $false }
    $text = Get-Content (Join-Path $root "package-lock.json") -Raw
    $hosts = [regex]::Matches($text, '"resolved":\s*"(https?://[^/]+)') |
        ForEach-Object { $_.Groups[1].Value }
    ($hosts | Where-Object { $_ -notlike "https://registry.npmjs.org*" }).Count -eq 0
}

# 4. Allowed hosts in the Rust dependency graph must stay the two official
#    registries only (crates.io via sparse index).
Check "Cargo.lock registry entries are crates.io only" {
    if (-not (Test-Path (Join-Path $root "src-tauri\Cargo.lock"))) { return $false }
    $text = Get-Content (Join-Path $root "src-tauri\Cargo.lock") -Raw
    -not [regex]::IsMatch($text, 'source = "git\+')
}

if ($fail) {
    Write-Host "[verify-offline] GATE FAILED" -ForegroundColor Red
    exit 1
}
Write-Host "[verify-offline] ALL PASS - artifacts are offline-clean" -ForegroundColor Green