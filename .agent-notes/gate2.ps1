# Gate stage 2: the workspace test run, plus the retargeting report printed.
$ErrorActionPreference = 'Continue'
$env:CARGO_TARGET_DIR = 'B:\repos\borwood\deepcraft\target'
$env:CARGO_BUILD_JOBS = '4'
$cargo = "$env:USERPROFILE\.cargo\bin\cargo.exe"
$log = '.agent-notes/gate2.log'
try {
    & $cargo test --workspace --release 2>&1 | Tee-Object -FilePath $log
    "TEST EXIT=$LASTEXITCODE" | Tee-Object -FilePath $log -Append
    # The measurement, printed (same binary, --nocapture).
    & $cargo test --release -p dc-client --bin dc-client -- --nocapture --exact `
        body::tests::retargeting_across_proportions_is_measured 2>&1 |
        Tee-Object -FilePath '.agent-notes/retarget-report.log'
    "REPORT EXIT=$LASTEXITCODE" | Tee-Object -FilePath '.agent-notes/retarget-report.log' -Append
} finally {
    Remove-Item 'B:\repos\borwood\deepcraft\target\.agent-build.lock' -ErrorAction SilentlyContinue
    'LOCK RELEASED'
}
