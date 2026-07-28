#!/usr/bin/env python3
"""SessionStart hook: tell the session which corpus sweeps are DUE, and why.

WHY THIS EXISTS (user-directed, 2026-07-28: *"sweeps should probably run first
thing"*). Measured the same day, and it is the finding that produced this file:

    This repo has four corpus controls -- spine-audit, doc-topology, the staleness
    sweep, and the filesize hook. THREE OF THEM SHARE ONE TRIGGER: the main
    session remembering to invoke them. That trigger is the resource under the
    most pressure, and the evidence is unambiguous -- over eleven active days,
    spine-audit left ZERO artifacts despite its own description saying "run
    periodically, a few times a day on active days"; doc-topology ran ONCE, the
    day it was created; the staleness sweep ran twice and its own skill file
    admits "today's sweep had to be requested -- that is the gap."

    The filesize hook is the only control that does not depend on memory, and it
    is the only one that fires reliably. That is not a coincidence, and it is the
    entire argument for this file.

CLAUDE.md Sec Gates already holds the governing rule, earned when the probe advisory
failed in one day: **"do not answer 'the gate cannot see X' with a rule asking
people to remember X."** Packaging a sweep as a skill does NOT satisfy that rule --
a skill still waits to be invoked. This hook does: it computes the delta from each
recorded watermark and states what is due, unprompted, at the one moment a session
can still act on it.

WHAT IT DOES NOT DO. It does not dispatch agents -- an agent cannot self-dispatch
and the integrator owns that call. It reports. The judgement of whether to spend
the tokens stays with the session and the user, which is also why it prints the
reason rather than just the verdict.

ADOPTION: the watermarks are written BY each sweep as part of producing its audit
doc, never as a separate step. Measured law (2026-07-28): a convention survives
when it is inseparable from an act the author must perform anyway, and dies when it
asks them to restate something in a second notation -- JUSTIFIED-BY had a documented
convention, a stated validator and a promised sweep, and got 3 uses, 0 in crates/.
"""

import json
import os
import subprocess
import sys

STALE_AFTER_COMMITS = 25  # a session's worth of work; tune from experience, not taste
WATERMARK_REL = "docs/audits/.sweep-watermarks.json"


def git(repo, *args):
    try:
        out = subprocess.run(
            ["git", "-C", repo, *args],
            capture_output=True, text=True, timeout=15,
        )
        return out.stdout.strip() if out.returncode == 0 else ""
    except Exception:
        return ""


def changed(repo, since, *paths):
    """Files matching `paths` changed since `since`. Empty string if unknown."""
    if not since:
        return ""
    return git(repo, "diff", "--name-only", f"{since}..HEAD", "--", *paths)


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except Exception:
        payload = {}

    repo = payload.get("cwd") or os.environ.get("CLAUDE_PROJECT_DIR") or "."
    wm_path = os.path.join(repo, WATERMARK_REL)

    try:
        with open(wm_path, "r", encoding="utf-8") as fh:
            wm = json.load(fh)
    except Exception:
        return 0  # no watermark file -> say nothing rather than nag

    head = git(repo, "rev-parse", "--short", "HEAD")
    if not head:
        return 0

    due = []

    for name, key, ref_paths, ref_label in (
        ("staleness-sweep", "staleness", None, None),
        ("spine-audit", "spine-audit", ("docs/spines.md",), "docs/spines.md"),
        ("doc-topology", "doc-topology", None, None),
    ):
        mark = wm.get(key)

        if not mark:
            due.append(f"  - **{name}** — DUE: no watermark recorded (never run, or reset).")
            continue

        log = git(repo, "log", "--oneline", f"{mark}..HEAD")
        n = len([ln for ln in log.splitlines() if ln.strip()])
        if n == 0:
            continue  # nothing has happened; genuinely not due

        reasons = [f"{n} commit(s) since `{mark}`"]
        full = n >= STALE_AFTER_COMMITS

        # A sweep's incremental mode is valid only while its REFERENCE side is
        # unchanged. If the reference moved, every prior verdict was made against
        # a different rule -> full re-run.
        if ref_paths:
            if changed(repo, mark, *ref_paths):
                full = True
                reasons.append(f"**{ref_label} itself changed → FULL re-run**")

        # A recalibration CAN invalidate a whole cohort of prior observations at
        # once, and no amount of per-entry reading finds that (journal/0111).
        #
        # ⚠ But only if it is ENABLED in the config those observations were made
        # under. Falsified 2026-07-28, the day this shipped: `calibrated_rates` was
        # built, measured and left OFF -- production asserts it false
        # (walk_tour_0115.rs:150) -- so the pre-0111 cohort was never artifactual,
        # and a full-corpus read of all 129 Observed entries found exactly ONE
        # sensitive entry against a brief that predicted many.
        #
        # So this flags for ATTENTION; it does not assert the cohort is void. The
        # sweep must check the flag's default before re-reading anything.
        if key == "staleness":
            body = git(repo, "log", f"{mark}..HEAD", "--format=%s%n%b")
            if body and any(w in body.lower() for w in ("recalibrat", "calibrat")):
                full = True
                reasons.append(
                    "**a recalibration landed → FULL re-run** — but FIRST check whether it is "
                    "ENABLED by default; one left off-by-default voids nothing (journal/0111, "
                    "`calibrated_rates`)"
                )

        if full and "FULL" not in " ".join(reasons):
            reasons.append(f"**≥{STALE_AFTER_COMMITS} commits → FULL re-run**")

        mode = "FULL" if full else "incremental"
        due.append(f"  - **{name}** — DUE ({mode}): " + "; ".join(reasons))

    if not due:
        return 0

    context = (
        "CORPUS SWEEPS DUE (SessionStart). Run these FIRST THING — findings are only "
        "actionable if they arrive with a session left to act on them (user, 2026-07-28).\n"
        + "\n".join(due)
        + "\n  Each is a background agent, model opus, isolation worktree, read-only except its "
        "own audit doc. Delta for an incremental run is `git log <watermark>..HEAD`.\n"
        "  The sweep writes its watermark in the same commit as its audit — never as a "
        "separate step.\n"
        "  ⚠ This hook REPORTS; it does not dispatch. Whether to spend the tokens is the "
        "session's call and the user's — say what is due and what it would cost before launching."
    )

    json.dump(
        {
            "hookSpecificOutput": {
                "hookEventName": "SessionStart",
                "additionalContext": context,
            }
        },
        sys.stdout,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
