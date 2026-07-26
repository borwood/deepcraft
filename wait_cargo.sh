#!/bin/sh
# Emit one line when no cargo process remains (the workspace gate has finished).
while true; do
  n=$(powershell -NoProfile -Command "(Get-Process cargo -ErrorAction SilentlyContinue | Measure-Object).Count" | tr -d '\r')
  if [ "$n" = "0" ]; then
    echo "CARGO IDLE - workspace gate finished"
    exit 0
  fi
  sleep 15
done
