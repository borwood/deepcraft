#!/usr/bin/env python3
"""PreToolUse hook: the build-slot mutex, as a MECHANISM instead of a memo.

WHY (2026-08-02, ROADMAP § Observed "THE BUILD-SLOT MUTEX IS DEAD AS A
MECHANISM"). On 2026-08-01 the advisory `.agent-build.lock` failed in both
directions in one evening, between two sessions that both knew the doctrine: a
full test gate was externally terminated at 58/90 suites while its lock sat in
place, and the other session's cleanup deleted the lock unread and
force-killed cargo/rustc ownership-blind — its own words: *"I deleted a mutex
without reading it back, which is the exact failure CLAUDE.md names."*
CLAUDE.md § Gates already holds the general lesson: do not answer "the gate
cannot see X" with a rule asking people to remember X. So the rule became a
hook.

WHAT IT ENFORCES — CLAUDE.md § Build rules, verbatim: **"One cargo invocation
at a time across ALL agents/sessions"** (parallel heavy builds have hung this
machine). Any Bash/PowerShell command that invokes cargo is DENIED while:

  1. any `cargo.exe` / `rustc.exe` process is alive (whosever it is —
     including this session's own background gate; the rule is one cargo
     TOTAL, not one per session); or
  2. `.agent-build.lock` is fresher than SPAWN_GRACE_S and stamped by a
     DIFFERENT session — this covers the window after another session's hook
     approved a cargo command but before its process shows up in tasklist.

Otherwise the hook stays silent (exit 0, no decision) so the normal
permission flow proceeds unchanged — it never auto-APPROVES anything — and it
stamps the lock with this session's id, a timestamp and the command, so the
next session's hook (and any human) can see who holds the slot.

SESSIONS NO LONGER MANAGE THE LOCK BY HAND. Do not write it, do not delete
it, do not re-read it before cargo calls — the hook stamps it and the hook
reads it. A stale lock (older than SPAWN_GRACE_S, no processes) blocks
nothing. If you must clear a wedged build, stop the specific PID you have
diagnosed — never `Get-Process cargo | Stop-Process` unscoped.

Fail-open by design: if tasklist errors or the lock is unreadable garbage,
the hook does not brick the machine's builds — the residual risk is the old
advisory world, which is strictly no worse.
"""

import json
import os
import re
import subprocess
import sys
import time

REPO = os.environ.get("CLAUDE_PROJECT_DIR") or r"B:\repos\borwood\deepcraft"
LOCK = os.path.join(REPO, ".agent-build.lock")

# How long a foreign lock stamp blocks the slot while no process is visible
# yet. Long enough to cover approval->spawn latency; short enough that a
# cancelled permission prompt self-heals quickly.
SPAWN_GRACE_S = 180

# Match cargo as an INVOKED PROGRAM, not as a substring: (position) at the
# start of the command, after a command separator (; & | ( or newline), or as
# the tail of a path (…\cargo.exe / …/cargo); AND (shape) followed by a
# subcommand-looking word, a flag, a toolchain +, or end-of-command. Two false
# positives taught this shape in the hook's first hours (2026-08-02): a grep
# whose PATTERN contained "cargo …", then a commit message whose line wrap put
# a quoted "cargo" mention at start-of-line. Residual misses are accepted and
# fail-open (`ENV=1 cargo build`, an unlisted future subcommand — add it to
# SUBCOMMANDS); a residual false positive remains for prose lines that START
# with a real invocation shape ("cargo test is slow") — reword and retry.
SUBCOMMANDS = (
    "add|bench|b|build|check|c|clean|clippy|doc|fix|fmt|install|metadata|"
    "nextest|remove|run|r|test|t|tree|update|version"
)
CARGO_RE = re.compile(
    r"(?:^[\"']?|[;&|(\n]\s*[\"']?|[\\/])cargo(\.exe)?[\"']?"
    rf"(\s+(?:[+-]|(?:{SUBCOMMANDS})\b)|$)",
    re.IGNORECASE,
)


def build_processes():
    """Names+PIDs of live cargo/rustc processes, or [] (fail-open on error)."""
    procs = []
    for image in ("cargo.exe", "rustc.exe"):
        try:
            out = subprocess.run(
                ["tasklist", "/FI", f"IMAGENAME eq {image}", "/FO", "CSV", "/NH"],
                capture_output=True, text=True, timeout=10,
            ).stdout
        except Exception:
            continue
        for line in out.splitlines():
            if line.startswith(f'"{image}"'):
                parts = line.split('","')
                pid = parts[1] if len(parts) > 1 else "?"
                procs.append(f"{image} pid {pid}")
    return procs


def deny(reason: str) -> int:
    json.dump(
        {
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": reason,
            }
        },
        sys.stdout,
    )
    return 0


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except Exception:
        return 0
    cmd = (payload.get("tool_input") or {}).get("command") or ""
    if not CARGO_RE.search(cmd):
        return 0
    session = payload.get("session_id") or "unknown-session"

    procs = build_processes()
    if procs:
        return deny(
            "BUILD SLOT TAKEN — live build processes: "
            + ", ".join(procs)
            + ". One cargo invocation at a time across ALL sessions (CLAUDE.md "
            "§ Build rules; parallel builds have hung this machine) — this "
            "includes your own background gate. Wait for them to exit and "
            "retry. Do NOT Stop-Process ownership-blind; if a build is truly "
            "wedged, diagnose and stop the specific PID."
        )

    if os.path.isfile(LOCK):
        try:
            with open(LOCK, encoding="utf-8", errors="replace") as fh:
                raw = fh.read()
            info = json.loads(raw)
            holder = str(info.get("session", "unknown"))
            age = time.time() - float(info.get("ts", 0))
        except Exception:
            holder, age = "unparsable (pre-hook format)", time.time() - os.path.getmtime(LOCK)
        if holder != session and age < SPAWN_GRACE_S:
            return deny(
                f"BUILD SLOT CLAIMED {int(age)}s ago by another session "
                f"({holder}) whose cargo has not spawned yet. Wait "
                f"~{int(SPAWN_GRACE_S - age)}s and retry; if no cargo process "
                "ever appears, the claim expires on its own."
            )

    # Slot is free: stamp it for the other session's hook to see, and stay
    # silent so the normal permission flow decides the command itself.
    try:
        with open(LOCK, "w", encoding="utf-8") as fh:
            json.dump(
                {"session": session, "ts": time.time(), "cmd": cmd[:120]}, fh
            )
    except Exception:
        pass
    return 0


if __name__ == "__main__":
    sys.exit(main())
