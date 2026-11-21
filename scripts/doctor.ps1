[CmdletBinding()]
param(
    [Alias("skip-smoke")]
    [switch]$SkipSmoke
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$failures = [System.Collections.Generic.List[string]]::new()
$warnings = [System.Collections.Generic.List[string]]::new()

function Add-Failure {
    param([Parameter(Mandatory)][string]$Message)
    $failures.Add($Message)
    Write-Host "FAIL: $Message" -ForegroundColor Red
}

function Add-Warning {
    param([Parameter(Mandatory)][string]$Message)
    $warnings.Add($Message)
    Write-Host "WARN: $Message" -ForegroundColor Yellow
}

function Require-Command {
    param(
        [Parameter(Mandatory)][string]$Name,
        [Parameter(Mandatory)][string]$Remediation
    )

    $command = Get-Command $Name -ErrorAction SilentlyContinue
    if ($null -eq $command) {
        Add-Failure "$Name is missing. $Remediation"
        return $false
    }
    return $true
}

function Invoke-SmokeCheck {
    param(
        [Parameter(Mandatory)][string]$Label,
        [Parameter(Mandatory)][string]$Command,
        [Parameter(Mandatory)][string[]]$Arguments
    )

    Write-Host "check: $Label"
    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        Add-Failure "$Label failed with exit code $LASTEXITCODE. Re-run the command above after correcting its first error."
    }
}

function Test-BuildTools {
    $vswhereCandidates = @(
        (Join-Path ${env:ProgramFiles(x86)} "Microsoft Visual Studio\Installer\vswhere.exe"),
        (Join-Path $env:ProgramFiles "Microsoft Visual Studio\Installer\vswhere.exe")
    )

    foreach ($candidate in $vswhereCandidates) {
        if (-not (Test-Path -LiteralPath $candidate -PathType Leaf)) {
            continue
        }
        $installation = & $candidate -products "*" -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
        if ($LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace(($installation -join ""))) {
            Write-Host "ok: Visual Studio C++ Build Tools at $($installation | Select-Object -First 1)"
            return
        }
    }

    Add-Failure "Visual Studio C++ Build Tools with the VCTools workload are missing. Run scripts/bootstrap.ps1."
}

function Test-WebView2 {
    $roots = @(
        "HKLM:\SOFTWARE\Microsoft\EdgeUpdate\Clients",
        "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients",
        "HKCU:\SOFTWARE\Microsoft\EdgeUpdate\Clients"
    )

    foreach ($root in $roots) {
        if (-not (Test-Path -LiteralPath $root)) {
            continue
        }
        foreach ($client in Get-ChildItem -LiteralPath $root -ErrorAction SilentlyContinue) {
            $properties = Get-ItemProperty -LiteralPath $client.PSPath -ErrorAction SilentlyContinue
            if ($null -ne $properties -and "$($properties.name)" -match "WebView2") {
                Write-Host "ok: WebView2 Runtime $($properties.pv)"
                return
            }
        }
    }

    Add-Warning "WebView2 Runtime was not found in the Edge Update registry. Install Microsoft.EdgeWebView2Runtime before running the desktop app."
}

Push-Location $repoRoot
try {
    Write-Host "Imaguncula doctor"
    Write-Host "repository: $repoRoot"

    $expectedNode = (Get-Content -LiteralPath ".node-version" -Raw).Trim()
    $expectedJust = (Get-Content -LiteralPath ".just-version" -Raw).Trim()
    $package = Get-Content -LiteralPath "package.json" -Raw | ConvertFrom-Json
    $expectedPnpm = "$($package.packageManager)".Split("@")[-1]
    $toolchainText = Get-Content -LiteralPath "rust-toolchain.toml" -Raw
    $toolchainMatch = [regex]::Match($toolchainText, 'channel\s*=\s*"([^"]+)"')
    if (-not $toolchainMatch.Success) {
        Add-Failure "rust-toolchain.toml does not declare a channel."
        $expectedRust = "unknown"
    } else {
        $expectedRust = $toolchainMatch.Groups[1].Value
    }

    $hasNode = Require-Command "node" "Run scripts/bootstrap.ps1."
    $hasPnpm = Require-Command "pnpm" "Run scripts/bootstrap.ps1."
    $hasRustc = Require-Command "rustc" "Run scripts/bootstrap.ps1."
    $hasCargo = Require-Command "cargo" "Run scripts/bootstrap.ps1."
    $hasRustup = Require-Command "rustup" "Run scripts/bootstrap.ps1."
    $hasJust = Require-Command "just" "Run scripts/bootstrap.ps1."

    if ($hasNode) {
        $actualNode = (& node --version).Trim().TrimStart("v")
        if ($actualNode -ne $expectedNode) {
            Add-Failure "Node.js $actualNode is active; expected $expectedNode from .node-version."
        } else {
            Write-Host "ok: Node.js $actualNode"
        }
    }
    if ($hasPnpm) {
        $actualPnpm = (& pnpm --version).Trim()
        if ($actualPnpm -ne $expectedPnpm) {
            Add-Failure "pnpm $actualPnpm is active; expected $expectedPnpm from package.json."
        } else {
            Write-Host "ok: pnpm $actualPnpm"
        }
    }
    if ($hasRustc -and $expectedRust -ne "unknown") {
        $actualRust = ((& rustc --version).Trim() -split " ")[1]
        if ($actualRust -ne $expectedRust) {
            Add-Failure "Rust $actualRust is active; expected $expectedRust from rust-toolchain.toml."
        } else {
            Write-Host "ok: Rust $actualRust"
        }
    }
    if ($hasCargo) {
        Write-Host "ok: $((& cargo --version).Trim())"
    }
    if ($hasRustup) {
        $components = & rustup component list --toolchain $expectedRust --installed
        foreach ($component in @("clippy", "rustfmt")) {
            if (-not ($components -match "^$component-")) {
                Add-Failure "Rust component $component is missing for toolchain $expectedRust."
            }
        }
    }
    if ($hasJust) {
        $actualJust = ((& just --version).Trim() -split " ")[-1]
        if ($actualJust -ne $expectedJust) {
            Add-Failure "just $actualJust is active; expected $expectedJust from .just-version."
        } else {
            Write-Host "ok: just $actualJust"
        }
    }

    Test-BuildTools
    Test-WebView2

    if (-not $SkipSmoke -and $failures.Count -eq 0) {
        Invoke-SmokeCheck "locked frontend dependency install" "pnpm" @("install", "--frozen-lockfile")
        Invoke-SmokeCheck "Rust workspace check" "cargo" @("check", "--workspace", "--all-targets")
        Invoke-SmokeCheck "Svelte and TypeScript check" "pnpm" @("check")
    } elseif ($SkipSmoke) {
        Write-Host "skip: dependency and compile smoke checks"
    }

    if ($failures.Count -gt 0) {
        Write-Host "doctor failed: $($failures.Count) blocking issue(s), $($warnings.Count) warning(s)" -ForegroundColor Red
        exit 1
    }

    Write-Host "doctor passed: $($warnings.Count) warning(s)"
} finally {
    Pop-Location
}
