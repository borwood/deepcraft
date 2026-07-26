#!/bin/sh
# Poll the shared build mutex until it is free (or the holder's compilers are gone).
i=0
while [ $i -lt 200 ]; do
  L=$(cat "B:/repos/borwood/deepcraft/target/.agent-build.lock" 2>/dev/null)
  P=$(powershell -NoProfile -Command "(Get-Process cargo,rustc -ErrorAction SilentlyContinue | Measure-Object).Count" 2>/dev/null | tr -d '\r')
  if [ -z "$L" ]; then
    echo "FREE (lock removed) after $((i*15))s"
    exit 0
  fi
  if [ "$P" = "0" ]; then
    echo "NO LIVE COMPILER after $((i*15))s; lock still reads: $L"
    exit 0
  fi
  i=$((i+1))
  sleep 15
done
echo "STILL HELD after 50 min: $L"
