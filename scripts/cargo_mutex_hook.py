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
     approved a cargo command but before its process shows up in tasklist; or
  3. the slot is free but `.agent-build.queue` says another session was
     denied FIRST and is still retrying (entries refresh on each retry and
     expire after QUEUE_TTL_S). Added 2026-08-03 (greenlit fingerprint):
     without it, a contended slot was won by whichever session's retry poll
     landed first, and a slower-polling session could starve indefinitely.
     First-come order, recorded at first denial. `.agent-build.queue` is
     hook-managed exactly like the lock: never write, delete, or reason from
     it by hand.

Otherwise the hook stays silent (exit 0, no decision) so the normal
permission flow proceeds unchanged — it never auto-APPROVES anything — pops
this session from the queue head if it was there, and stamps the lock with
this session's id, a timestamp and the command, so the next session's hook
(and any human) can see who holds the slot.

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
QUEUE = os.path.join(REPO, ".agent-build.queue")

# How long a foreign lock stamp blocks the slot while no process is visible
# yet. Long enough to cover approval->spawn latency; short enough that a
# cancelled permission prompt self-heals quickly.
SPAWN_GRACE_S = 180

# How long a queue entry stays valid without being refreshed by a retry.
# Denied sessions retry every 60-120 s by doctrine; 600 s tolerates a slow
# loop while evicting sessions that stopped retrying (killed, wrapped, gone).
# Added 2026-08-03 (greenlit fingerprint): before the queue, contended
# afternoons were retry RACES — whichever session's retry landed first took
# the slot, and a session could starve behind two faster-polling siblings.
# First-come order, recorded at first denial, fixes that. Fail-open like
# everything else here: unreadable queue = no queue.
QUEUE_TTL_S = 600

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


def read_queue():
    """The live queue: [{'session', 'ts'}, ...] with stale entries evicted.
    Fail-open: any error reads as an empty queue."""
    try:
        with open(QUEUE, encoding="utf-8") as fh:
            entries = json.load(fh)
        now = time.time()
        return [
            e for e in entries
            if isinstance(e, dict) and now - float(e.get("ts", 0)) < QUEUE_TTL_S
        ]
    except Exception:
        return []


def write_queue(entries) -> None:
    try:
        if entries:
            with open(QUEUE, "w", encoding="utf-8") as fh:
                json.dump(entries, fh)
        elif os.path.isfile(QUEUE):
            os.remove(QUEUE)
    except Exception:
        pass


def enqueue(session: str) -> int:
    """Register (or refresh) this session's place; return its 1-based position."""
    entries = read_queue()
    for i, e in enumerate(entries):
        if e.get("session") == session:
            e["ts"] = time.time()
            write_queue(entries)
            return i + 1
    entries.append({"session": session, "ts": time.time()})
    write_queue(entries)
    return len(entries)


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
        pos = enqueue(session)
        return deny(
            "BUILD SLOT TAKEN — live build processes: "
            + ", ".join(procs)
            + f". You are QUEUED at position {pos} (first-come; your place "
            "refreshes on every retry and expires after 10 min without one). "
            "One cargo invocation at a time across ALL sessions (CLAUDE.md "
            "§ Build rules; parallel builds have hung this machine) — this "
            "includes your own background gate. Wait 60-120 s and retry. "
            "Do NOT Stop-Process ownership-blind; if a build is truly "
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
            pos = enqueue(session)
            return deny(
                f"BUILD SLOT CLAIMED {int(age)}s ago by another session "
                f"({holder}) whose cargo has not spawned yet. You are QUEUED "
                f"at position {pos}. Wait ~{int(SPAWN_GRACE_S - age)}s and "
                "retry; if no cargo process ever appears, the claim expires "
                "on its own."
            )

    # Slot is free. Honor the queue: whoever was denied first goes first.
    entries = read_queue()
    if entries:
        head = str(entries[0].get("session", ""))
        if head != session:
            pos = enqueue(session)
            return deny(
                f"BUILD SLOT FREE but QUEUED — session {head} was denied "
                f"first and holds the head of the queue; you are position "
                f"{pos}. Wait 60-120 s and retry; a head that stops retrying "
                "is evicted after 10 min and the queue advances."
            )
        # This session is the head: pop itself and take the slot.
        write_queue(entries[1:])

    # Stamp the slot for the other sessions' hooks to see, and stay
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
