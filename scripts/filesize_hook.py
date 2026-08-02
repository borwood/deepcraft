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


CONVENTIONS — DECIDED 2026-07-28 (user). These are no longer provisional.
=========================================================================

**⚠ SCOPE CORRECTED 2026-08-01 (user; corrections #85): the liveness axis is a
DOCUMENTS convention.** Every measurement below is a `.md` measurement, and the
thresholds comment further down always said so — *"code splits on ordinary module
boundaries"* — while `remedy()` fell through and emitted the docs doctrine at
SOURCE files for four days. **Source files split BY CONCERN, on module
boundaries**: the compiler and the gate enforce cross-file consistency, so the
claim-near-refutation hazard that motivates everything below has no code
analogue. The section that follows governs `.md` only.

**THE SPLIT AXIS IS LIVENESS, NEVER TOPIC.** Every split moves out the *cold*
half — content that is still true and still cited but is no longer read to do
today's work. Both real conversions in this repo already did exactly that, and
neither was by topic:

    ROADMAP.md  -> ROADMAP-history.md    split by STATUS ("still read live?"),
                                          explicitly NOT by age
    ...notebook -> ...-evidence.md       split by READ PATTERN ("read once,
                                          cited often" vs the live argument)

**Why topic-splitting is disallowed, from this repo's own measurements.**
Contradiction here is produced by ADDITION, not replacement — design docs delete
only 2-4 % of what they add. The expensive failures were all a claim sitting near
its own refutation: two sentences apart (corrections #65), forty lines (#58), two
subsections (#53), 400 lines (journal/0119). Splitting a *live* doc by topic takes
two claims that were forty lines apart and puts them in separate files, where only
the `doc-topology` sweep can reach them — and five of that sweep's top eight
findings were unsuspected by construction. **That trades the cheapest of the three
docs-ops failures (VOLUME, "third in value") for the most expensive one
(TOPOLOGY, "the one that cost an architecture").**

Stated at its honest strength: co-location did *not* prevent those contradictions
— the evidence is explicit that access was never the problem. The claim is the
weaker, sufficient one: **topic-splitting costs the one condition under which a
reader could notice, and buys only line count.** Liveness-splitting has no such
cost, because the cold half **has stopped accreting** — you cannot author a new
contradiction into a file nobody writes to.

**THE THRESHOLD APPLIES TO THE HOT FILE ONLY.** An archive or evidence file is
exempt by designation: nobody reads it whole, by design, and it is not growing.
(Before this, the hook flagged `ROADMAP-history.md` for being precisely what it
was built to be — crying wolf on a file doing its job, with no correct action
available.)

**THREE CLASSES OF .md, because they have different READ PATTERNS:**

  NARRATIVE  journal entries, audits, spike results. Written once, read whole or
             not at all, never revised. EXEMPT. Splitting one is actively harmful
             — a narrative cut in half is two things nobody can follow.
             (Measured: 120 entries, median 177 lines, max 536.)

  REGISTRY   corrections.md, stubs.md, spines.md, ROADMAP.md. Entry-addressable
             and append-only: you look up #66, you do not read the file.
             High threshold; the split is ARCHIVING RESOLVED ENTRIES, as
             ROADMAP-history.md already does.

  ARGUMENT   docs/design/*.md, docs/*.md, skills. Read in sections to understand a
             system, and actively revised. The only class where the threshold
             really bites, and the only class where topology failures happen.

**WHAT IS DELIBERATELY NOT BUILT:** no taxonomy registry, no frontmatter marking a
file's class, no validator. Two conversions is below this project's own bar —
`stubs.md:22` and session-workflow § Seam-first #6 both forbid designing the
general mechanism before several real conversions have taught the shape. Class is
derived from path below, which is enough until the next two or three splits.

**HONEST LIMIT:** volume is the *third* most valuable of the three docs-ops
failures, and the ROADMAP archive "would not have prevented journal/0119." This
hook buys agent context efficiency. It is not a correctness fix, and the
still-open corpus-addressability thread may subsume part of it — a file that is
addressable by section may not need to be small.
"""

import json
import os
import sys

# Source thresholds. Source is tightest because code has the most separable
# concerns and is the most expensive to read irrelevantly. (Note: the liveness
# rule above is about DOCUMENTS; code splits on ordinary module boundaries.)
THRESHOLDS = {".rs": 700, ".md": 1000, ".toml": 400, ".py": 500, ".ps1": 400}
DEFAULT_THRESHOLD = 800  # applies to any extension not listed above

# --- .md document classes (DECIDED 2026-07-28) -------------------------------
# Matched against the forward-slash-normalised path, first hit wins.

# NARRATIVE — written once, read whole, never revised. Splitting is harmful.
NARRATIVE_PATTERNS = ("/journal/0", "/docs/audits/", "/docs/spikes/")

# ARCHIVE / EVIDENCE — the designated COLD half of an already-performed split.
# Exempt: it is not read whole and it is not accreting.
ARCHIVE_PATTERNS = ("-history.md", "-evidence.md", "/archive/")

# REGISTRY — entry-addressable, append-only, looked up by ordinal.
REGISTRY_PATTERNS = (
    "/roadmap.md",
    "/corrections.md",
    "/spines.md",
    "/stubs.md",
)
REGISTRY_THRESHOLD = 2500

# Non-.md exemptions.
EXEMPT_SUBSTRINGS = ("Cargo.lock", "/target/", "\\target\\")


def classify_md(norm_lower: str):
    """Return (class_name, threshold) or (class_name, None) if exempt."""
    if any(p in norm_lower for p in NARRATIVE_PATTERNS):
        return "NARRATIVE", None
    if any(p in norm_lower for p in ARCHIVE_PATTERNS):
        return "ARCHIVE", None
    if any(p in norm_lower for p in REGISTRY_PATTERNS):
        return "REGISTRY", REGISTRY_THRESHOLD
    return "ARGUMENT", THRESHOLDS[".md"]


def remedy(cls: str) -> str:
    if cls == "SOURCE":
        return (
            "THIS IS SOURCE CODE: split it BY CONCERN, on ordinary module boundaries — the "
            "compiler and the test gate enforce cross-file consistency, so the docs-only "
            "claim-near-refutation hazard that forbids topic-splitting .md files has no "
            "analogue here (user ruling 2026-08-01, corrections #85; the liveness axis was "
            "always scoped to DOCUMENTS). Prefer splits that leave each module one concern "
            "whose consumers import it by name. Propose the extraction to the user rather "
            "than doing it silently mid-task."
        )
    if cls == "REGISTRY":
        return (
            "THIS IS A REGISTRY (entry-addressable, append-only — read by ordinal, not read "
            "whole). The split for a registry is ARCHIVING RESOLVED ENTRIES into a designated "
            "cold file, exactly as ROADMAP-history.md already does; entries keep their stable "
            "number as the pointer. Archive by STATUS, never by age — a two-week-old live entry "
            "may be the most important thing on the board."
        )
    return (
        "THE SPLIT AXIS IS LIVENESS, NEVER TOPIC (DECIDED 2026-07-28). ASK NOW: has this file "
        "accumulated a COLD HALF — content still true and still cited, but no longer read to do "
        "today's work (shipped/resolved entries, evidence tables, dated measurement records)? If "
        "so, move THAT out and leave the live argument whole. Do NOT split by topic: this corpus "
        "produces contradictions by ADDITION (design docs delete 2-4% of what they add), and the "
        "expensive failures were all a claim sitting near its own refutation — two sentences "
        "apart, forty lines, two subsections. Topic-splitting converts an in-file contradiction "
        "into a cross-file one, trading the cheapest docs-ops failure (volume) for the most "
        "expensive (topology). Propose the extraction to the user rather than doing it silently "
        "mid-task."
    )


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

    if ext == ".md":
        cls, threshold = classify_md(norm.lower())
        if threshold is None:
            return 0  # NARRATIVE and ARCHIVE are exempt by class
    else:
        cls, threshold = "SOURCE", THRESHOLDS.get(ext, DEFAULT_THRESHOLD)

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
        f"FILE SIZE — {name} is {lines} lines against the {threshold}-line threshold "
        f"for class {cls} ({over:.1f}x).\n"
        "A file is a unit of context. Past this, an agent must either grep it (and see "
        "only what it already knew to look for) or read it whole (and burn context on "
        "irrelevance). Both are lossy, and this repo has the receipts: DeepField::chapters "
        "sat unlisted through three spine audits, and the ROADMAP is grepped rather than "
        "read.\n"
        f"{remedy(cls)}\n"
        "Adoption: immediate for new files, gradual for old work when touched. Conventions "
        "DECIDED 2026-07-28, scope corrected 2026-08-01 — see scripts/filesize_hook.py's "
        "module docstring for the .md liveness axis, the three .md classes, and why "
        "topic-splitting of DOCS (only) is disallowed."
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
