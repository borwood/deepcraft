# Gate stage 1: clean the crates I touched AND the crates the siblings touch,
# then fmt + clippy. Tee everything; grep the FILE, never this console.
$ErrorActionPreference = 'Continue'
$env:CARGO_TARGET_DIR = 'B:\repos\borwood\deepcraft\target'
$env:CARGO_BUILD_JOBS = '4'
$cargo = "$env:USERPROFILE\.cargo\bin\cargo.exe"
$log = '.agent-notes/gate1.log'
try {
    & $cargo clean -p dc-api -p dc-client -p dc-core -p dc-worldgen --release 2>&1 |
        Tee-Object -FilePath $log
    & $cargo fmt --all --check 2>&1 | Tee-Object -FilePath $log -Append
    "FMT EXIT=$LASTEXITCODE" | Tee-Object -FilePath $log -Append
    & $cargo clippy --workspace --all-targets --release -- -D warnings 2>&1 |
        Tee-Object -FilePath $log -Append
    "CLIPPY EXIT=$LASTEXITCODE" | Tee-Object -FilePath $log -Append
} finally {
    Remove-Item 'B:\repos\borwood\deepcraft\target\.agent-build.lock' -ErrorAction SilentlyContinue
    'LOCK RELEASED'
}
