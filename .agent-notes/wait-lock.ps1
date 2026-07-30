# Wait until the shared agent build lock is free or provably stale.
# Emits one line and exits. Scratch helper, not part of the build.
$lk = 'B:\repos\borwood\deepcraft\target\.agent-build.lock'
for ($i = 0; $i -lt 160; $i++) {
    if (-not (Test-Path $lk)) { Write-Output 'LOCK-FREE: file gone'; exit 0 }
    $n = (Get-Process cargo, rustc -ErrorAction SilentlyContinue | Measure-Object).Count
    $age = ((Get-Date) - (Get-Item $lk).LastWriteTime).TotalMinutes
    if ($n -eq 0 -and $age -gt 40) {
        Write-Output ('LOCK-STALE: age {0:N1} min, no live compiler' -f $age)
        exit 0
    }
    Start-Sleep -Seconds 20
}
Write-Output 'LOCK-STILL-HELD after ~53 min'
