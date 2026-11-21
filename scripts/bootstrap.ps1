[CmdletBinding()]
param(
    [Alias("dry-run")]
    [switch]$DryRun,
    [switch]$Yes
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$expectedNode = (Get-Content -LiteralPath (Join-Path $repoRoot ".node-version") -Raw).Trim()
$expectedJust = (Get-Content -LiteralPath (Join-Path $repoRoot ".just-version") -Raw).Trim()
$package = Get-Content -LiteralPath (Join-Path $repoRoot "package.json") -Raw | ConvertFrom-Json
$expectedPnpm = "$($package.packageManager)".Split("@")[-1]
$toolchainText = Get-Content -LiteralPath (Join-Path $repoRoot "rust-toolchain.toml") -Raw
$expectedRust = [regex]::Match($toolchainText, 'channel\s*=\s*"([^"]+)"').Groups[1].Value

function Refresh-ProcessPath {
    $machinePath = [Environment]::GetEnvironmentVariable("Path", "Machine")
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $extraPaths = @(
        (Join-Path $env:USERPROFILE ".cargo\bin"),
        (Join-Path $env:APPDATA "npm")
    )
    $env:Path = (@($machinePath, $userPath) + $extraPaths | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }) -join ";"
}

function Invoke-Step {
    param(
        [Parameter(Mandatory)][string]$Description,
        [Parameter(Mandatory)][string]$Command,
        [Parameter(Mandatory)][string[]]$Arguments
    )

    Write-Host "step: $Description"
    Write-Host "  $Command $($Arguments -join ' ')"
    if ($DryRun) {
        return
    }

    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Description failed with exit code $LASTEXITCODE. Correct the reported error, then re-run this script."
    }
}

function Test-CommandVersion {
    param(
        [Parameter(Mandatory)][string]$Command,
        [Parameter(Mandatory)][string]$Expected,
        [Parameter(Mandatory)][scriptblock]$ReadVersion
    )

    if ($null -eq (Get-Command $Command -ErrorAction SilentlyContinue)) {
        return $false
    }
    return (& $ReadVersion) -eq $Expected
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
            return $true
        }
    }
    return $false
}

function Test-WingetPackage {
    param([Parameter(Mandatory)][string]$Id)

    if ($null -eq (Get-Command "winget" -ErrorAction SilentlyContinue)) {
        return $false
    }
    & winget list --exact --id $Id --accept-source-agreements *> $null
    return $LASTEXITCODE -eq 0
}

Push-Location $repoRoot
try {
    Write-Host "Imaguncula Windows bootstrap"
    Write-Host "repository: $repoRoot"
    Write-Host "mode: $(if ($DryRun) { 'dry run' } elseif ($Yes) { 'unattended' } else { 'interactive' })"
    Write-Host "pinned tools: Node $expectedNode; pnpm $expectedPnpm; Rust $expectedRust; just $expectedJust"
    Write-Host "system changes may install Node.js, Visual Studio C++ Build Tools, and WebView2 through winget."
    Write-Host "user changes may install Rust under .cargo, pnpm through npm, and just through cargo."

    if ($DryRun) {
        Write-Host "dry-run plan:"
        Write-Host "  verify winget and install missing Windows system prerequisites"
        Write-Host "  install or select the pinned Node, pnpm, Rust, and just versions"
        Write-Host "  run scripts/doctor.ps1, including locked install and compile checks"
        Write-Host "dry run complete; no commands were executed"
        exit 0
    }

    if (-not $Yes) {
        $answer = Read-Host "Continue with the listed installations? [y/N]"
        if ($answer -notmatch '^(?i:y|yes)$') {
            Write-Host "bootstrap cancelled"
            exit 2
        }
    }

    Refresh-ProcessPath
    $nodeReady = Test-CommandVersion "node" $expectedNode { (& node --version).Trim().TrimStart("v") }
    $buildToolsReady = Test-BuildTools
    $webViewReady = Test-WingetPackage "Microsoft.EdgeWebView2Runtime"
    if (-not $nodeReady -or -not $buildToolsReady -or -not $webViewReady) {
        if ($null -eq (Get-Command "winget" -ErrorAction SilentlyContinue)) {
            throw "winget is required for missing Windows system prerequisites. Install Microsoft App Installer, then re-run this script."
        }
    }

    if (-not $nodeReady) {
        # The versioned package id tracks the pinned major; the unversioned
        # LTS id moves to the next major line and drops older versions.
        $nodeMajor = ($expectedNode -split "\.")[0]
        Invoke-Step "install Node.js $expectedNode" "winget" @(
            "install", "--exact", "--id", "OpenJS.NodeJS.$nodeMajor", "--version", $expectedNode,
            "--accept-package-agreements", "--accept-source-agreements", "--silent", "--force"
        )
        Refresh-ProcessPath
    } else {
        Write-Host "skip: Node.js $expectedNode is already active"
    }

    if (-not $buildToolsReady) {
        Invoke-Step "install Visual Studio 2022 C++ Build Tools (may request elevation)" "winget" @(
            "install", "--exact", "--id", "Microsoft.VisualStudio.2022.BuildTools",
            "--accept-package-agreements", "--accept-source-agreements", "--silent",
            "--override", "--wait --quiet --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
        )
    } else {
        Write-Host "skip: Visual Studio C++ Build Tools are installed"
    }

    if (-not $webViewReady) {
        Invoke-Step "install Microsoft Edge WebView2 Runtime" "winget" @(
            "install", "--exact", "--id", "Microsoft.EdgeWebView2Runtime",
            "--accept-package-agreements", "--accept-source-agreements", "--silent"
        )
    } else {
        Write-Host "skip: WebView2 Runtime is installed"
    }

    if ($null -eq (Get-Command "rustup" -ErrorAction SilentlyContinue)) {
        Invoke-Step "install rustup" "winget" @(
            "install", "--exact", "--id", "Rustlang.Rustup",
            "--accept-package-agreements", "--accept-source-agreements", "--silent"
        )
        Refresh-ProcessPath
    } else {
        Write-Host "skip: rustup is installed"
    }
    Invoke-Step "install pinned Rust toolchain and components" "rustup" @(
        "toolchain", "install", $expectedRust, "--profile", "minimal", "--component", "clippy", "--component", "rustfmt"
    )

    $pnpmReady = Test-CommandVersion "pnpm" $expectedPnpm { (& pnpm --version).Trim() }
    if (-not $pnpmReady) {
        Invoke-Step "install pnpm $expectedPnpm for the active Node.js installation" "npm" @(
            "install", "--global", "pnpm@$expectedPnpm"
        )
        Refresh-ProcessPath
    } else {
        Write-Host "skip: pnpm $expectedPnpm is already active"
    }

    $justReady = Test-CommandVersion "just" $expectedJust { ((& just --version).Trim() -split " ")[-1] }
    if (-not $justReady) {
        Invoke-Step "install just $expectedJust in the Cargo user bin directory" "cargo" @(
            "install", "just", "--locked", "--version", $expectedJust
        )
        Refresh-ProcessPath
    } else {
        Write-Host "skip: just $expectedJust is already active"
    }

    & (Join-Path $PSScriptRoot "doctor.ps1")
    if ($LASTEXITCODE -ne 0) {
        throw "doctor verification failed. Correct its first failure, then re-run this script."
    }

    Write-Host "ready: open a new PowerShell window if this script installed tools, then run just run" -ForegroundColor Green
} finally {
    Pop-Location
}
