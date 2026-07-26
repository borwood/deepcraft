#!/usr/bin/env python3
"""PostToolUse hook: flag files that have outgrown an agent's ability to read them.

WHY THIS EXISTS (user-directed, 2026-07-26). In an AI-native workspace a file is a
**unit of context**. Past a threshold an agent has only two options and *both are
lossy*: grep it (and see only what it already knew to look for) or read it whole
(and burn context on irrelevance, crowding out the files it actually needed).

The failure is on the record. `DeepField::chapters` sat unlisted through **three**
spine audits while sweeps added rows for its immediate neighbours in the same
struct — its own doc comment said "read by nothing" the whole time. And the
2026-07-25 close block named the same defect for the board: *"the ROADMAP is ~5,600
lines and no session reads it end-to-end. It is grepped."* This hook generalises
that finding from one document to the repo.

WHY A HOOK AND NOT A RULE. CLAUDE.md § Gates already carries the lesson: *"do not
answer 'the gate cannot see X' with a rule asking people to remember X."* That was
tried for probes and failed in one day. A reminder the harness issues is a
mechanism; a line in a doc is not.

ADOPTION (user's terms): **immediate for new files; gradual refactor of old work
when it is touched.** So firing on an existing oversized file is not noise — it is
the trigger doing its job. Output goes to the model as `additionalContext`, not to
the user as a `systemMessage`, so it guides without nagging.

⚠ THE THRESHOLDS BELOW ARE PROVISIONAL. The conventions themselves are an open
question (ROADMAP § Sequenced, "FILE SIZE IS A CORRECTNESS PROBLEM"): what the
limits should be per file type, what "separate concerns" means for each, and the
split conventions. These are a starting point to be *set*, not guessed at forever.
"""

import json
import os
import sys

# Provisional. Source is tightest because code has the most separable concerns and
# is the most expensive to read irrelevantly.
THRESHOLDS = {".rs": 700, ".md": 1000, ".toml": 400, ".py": 500, ".ps1": 400}
DEFAULT_THRESHOLD = 800  # applies to any extension not listed above

# Journal entries are linear narrative, written once to be read start-to-finish by
# a human, and append-only. Length there is not a context defect. Same for lock and
# generated files.
EXEMPT_SUBSTRINGS = ("/journal/0", "\\journal\\0", "Cargo.lock", "/target/", "\\target\\")


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except Exception:
        return 0

    tool_input = payload.get("tool_input") or {}
    tool_response = payload.get("tool_response") or {}
    path = tool_input.get("file_path") or tool_response.get("filePath")
    if not path or not os.path.isfile(path):
        return 0

    norm = path.replace("\\", "/")
    if any(s.replace("\\", "/") in norm for s in EXEMPT_SUBSTRINGS):
        return 0

    ext = os.path.splitext(path)[1].lower()
    threshold = THRESHOLDS.get(ext, DEFAULT_THRESHOLD)

    try:
        with open(path, "r", encoding="utf-8", errors="replace") as fh:
            lines = sum(1 for _ in fh)
    except Exception:
        return 0

    if lines <= threshold:
        return 0

    name = os.path.basename(path)
    over = lines / threshold
    context = (
        f"FILE SIZE — {name} is {lines} lines against a provisional {threshold}-line "
        f"threshold for {ext or 'this type'} ({over:.1f}x).\n"
        "A file is a unit of context. Past this, an agent must either grep it (and see "
        "only what it already knew to look for) or read it whole (and burn context on "
        "irrelevance). Both are lossy, and this repo has the receipts: DeepField::chapters "
        "sat unlisted through three spine audits, and the ROADMAP is grepped rather than "
        "read.\n"
        "ASK NOW: is what you just wrote a SEPARABLE CONCERN that belongs in its own file? "
        "New files should land under the threshold. For an existing oversized file this is "
        "the gradual-refactor trigger (adoption is: immediate for new work, gradual for old "
        "when touched) — if the concern you touched is cleanly extractable, propose the "
        "extraction to the user rather than doing it silently mid-task.\n"
        "Thresholds and split conventions are still OPEN — see ROADMAP § Sequenced, "
        '"FILE SIZE IS A CORRECTNESS PROBLEM IN AN AI-NATIVE WORKSPACE".'
    )

    json.dump(
        {
            "hookSpecificOutput": {
                "hookEventName": "PostToolUse",
                "additionalContext": context,
            }
        },
        sys.stdout,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
