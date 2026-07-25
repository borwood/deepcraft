# gate.ps1 — the merge gate, run as STAGES.
#
# WHY STAGES (earned 2026-07-25): the full gate exceeded a single 10-minute tool call and was
# KILLED mid-`test`. fmt and clippy had passed and no test had failed, yet the run produced
# NO VERDICT AT ALL — the worst possible shape. Both causes are permanent: `cargo clean -p`
# across dc-worldgen + dc-api + dc-client forces a Bevy rebuild, and the suite has grown past
# ~870 s in dc-worldgen alone plus the newly-gated probes (journal/0103).
#
# A killed stage is now unambiguous: you know exactly which one lacks a verdict, and `test`
# re-runs in a fraction of the time because `check` already built everything.
#
#   ./scripts/gate.ps1 -Stage check -Clean dc-worldgen,dc-api   # pays the rebuild
#   ./scripts/gate.ps1 -Stage test                              # fast; artifacts exist
#
# A TIMEOUT IS NOT A RED GATE — and it is equally not a green. Check how far it got, then
# finish the missing stage.
param(
    [ValidateSet('check','test')] [string]$Stage = 'check',
    # Crates to `cargo clean -p` first. Clean the crates YOU changed AND any a live sibling
    # changed — a sibling's stale artifact poisons yours even in a crate you never touched
    # (corrections #21/#27/#34).
    [string[]]$Clean = @()
)

$ErrorActionPreference = 'Continue'
$repo = Split-Path $PSScriptRoot -Parent
$env:CARGO_TARGET_DIR = Join-Path $repo 'target'
$env:CARGO_BUILD_JOBS = '4'
$cargo = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'

Write-Host "=== live builds (should be empty before a gate) ==="
Get-Process cargo, rustc, dc-client -ErrorAction SilentlyContinue |
    Select-Object Name, Id | Format-Table -AutoSize

if ($Stage -eq 'check') {
    foreach ($c in $Clean) {
        Write-Host "=== clean -p $c ==="
        & $cargo clean -p $c --release
    }
    Write-Host '=== fmt ==='
    & $cargo fmt --all --check
    $fmt = $LASTEXITCODE
    Write-Host "fmt exit=$fmt"

    Write-Host '=== clippy ==='
    & $cargo clippy --workspace --all-targets --release -- -D warnings
    $clippy = $LASTEXITCODE
    Write-Host "clippy exit=$clippy"

    Write-Host "=== CHECK STAGE COMPLETE (fmt=$fmt clippy=$clippy) ==="
    if ($fmt -ne 0 -or $clippy -ne 0) { exit 1 }
    exit 0
}

Write-Host '=== test ==='
& $cargo test --workspace --release
$test = $LASTEXITCODE
Write-Host "test exit=$test"
Write-Host '=== TEST STAGE COMPLETE ==='
# Verify by NAME/COUNT, never by `test result: ok` alone — a false green flatters
# (corrections #27). And confirm `Compiling <crate>` names MAIN's path, not a worktree's.
exit $test
