#!/usr/bin/env python3
"""PROTOTYPE — the stand-in marker control (corrections #97, ROADMAP § Sequenced).

NOT WIRED INTO ANYTHING. Shipped by the 2026-08-04 scoping audit
(`docs/audits/2026-08-04-stand-in-marker-control-scoping.md`) as the measured
artifact behind its recommendation. Wiring it into a gate or a hook is a
separate, user-owned call.

WHAT IT CHECKS
--------------
CLAUDE.md read-first item 6: a deliberate loose end is annotated **in code AND**
listed in a locus (`stubs.md` / `spines.md` § 3 / ROADMAP Owed). Nothing checked
the second half. This checks it for the one marker form that is mechanically
separable from prose:

    a `STAND-IN` token inside a DOC comment (`///` or `//!`) in a .rs file
    must carry a RESOLVABLE locus pointer inside its own comment block.

"Resolvable" means a reader can walk it: `stubs.md #40`, `stubs.md § 34`,
`stubs.md (B7-a, \\`the-slug\\`)`, `spines.md § 3`, `ROADMAP §`. The bare
filename is deliberately NOT enough — the marker that shipped
`\\`swing_gain\\`` said *"Owed a `stubs.md` entry"*, which a substring check
reads as satisfied and a reader cannot walk.

WHY ONLY `STAND-IN`, AND NOT `heir`
-----------------------------------
Measured 2026-08-04 over 261 .rs files. `\\bheirs?\\b` has 213 sites; hand
classification puts ~47 % of them at genuine markers and the rest at prose,
assertion strings and already-landed heirs. A `heir`-based check alarms on 34
sites of which ~1 is a real unfiled loose end — 97 % noise, and a noisy gate
gets worked around. `STAND-IN` in a doc comment has **14 sites and zero prose
false positives**, today and across the sigil's entire history.

WHAT IT CANNOT SEE — state this whenever the output is quoted
-------------------------------------------------------------
It checks markers that were WRITTEN. It cannot find a simplification nobody
annotated, and it does not read `stubs.md` to check the entry actually exists
or still describes the code. It is a back-pointer check, not a stub audit.

Usage:  python scripts/standin_locus_check.py [repo_root]
Exit:   0 clean, 1 if any marker lacks a resolvable pointer.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

MARKER = re.compile(r"STAND-IN")
DOC_COMMENT = re.compile(r"^\s*(///|//!)")
ANY_COMMENT = re.compile(r"^\s*(///|//!|//|\*)")

# A pointer a reader can WALK = the LOCUS named, plus a HANDLE that identifies the
# entry inside it. The bare filename does not qualify: the `swing_gain` marker said
# *"Owed a `stubs.md` entry"*, which a substring check reads as satisfied and a
# reader cannot walk. The two halves are checked separately rather than by
# proximity, because a module-header marker legitimately names the locus once and
# then lists five slugs over the following fifteen lines (`dc-api/.../limits.rs`).
ORDINAL_NEAR_LOCUS = re.compile(  # `stubs.md` #40 · stubs.md § 34 · stub #16 · stubs.md B7-a
    r"(?:stubs\.md|\bstubs?\b)\W{0,14}(?:#\s*\d+|§\s*\d+|[A-Z]\d+-[a-z]\b)", re.I
)
LOCUS_NAMED = re.compile(r"stubs\.md|spines\.md|ROADMAP", re.I)
# A stubs.md slug is always a hyphenated sentence — at least three words. Matching
# any backticked lowercase token instead let `` `clearance` `` satisfy the check on
# the ONE marker in the tree that says its entry is still *owed* (`swing_gain`).
SLUG = re.compile(r"`[a-z]+(?:-[a-z]+){2,}`")  # `the-fold-sense-is-declared-...`
OTHER_LOCUS = re.compile(r"spines\.md\s*§\s*3|ROADMAP\s*§", re.I)


def has_resolvable_pointer(block: str) -> bool:
    """A pointer is walkable if it names the locus AND identifies the entry."""
    if ORDINAL_NEAR_LOCUS.search(block):
        return True
    if LOCUS_NAMED.search(block) and SLUG.search(block):
        return True
    return bool(OTHER_LOCUS.search(block))

SEARCH_DIRS = ("crates", "plugins", "tools")


def comment_block(lines: list[str], i: int) -> tuple[int, int]:
    """The contiguous comment block containing line `i`."""
    lo = i
    while lo > 0 and ANY_COMMENT.match(lines[lo - 1]):
        lo -= 1
    hi = i
    while hi + 1 < len(lines) and ANY_COMMENT.match(lines[hi + 1]):
        hi += 1
    return lo, hi


def check(root: Path) -> list[tuple[str, int, str]]:
    alarms: list[tuple[str, int, str]] = []
    total = 0
    for d in SEARCH_DIRS:
        for path in sorted((root / d).rglob("*.rs")):
            try:
                lines = path.read_text(encoding="utf-8").splitlines()
            except OSError:
                continue
            for i, line in enumerate(lines):
                # Only DECLARATIONS: the sigil in a doc comment. An ordinary `//`
                # line mentioning another site's marker is a reference, not a
                # loose end, and is the only false positive this check has ever
                # produced (`dc-client/src/body.rs`, "comment's STAND-IN marker").
                if not (MARKER.search(line) and DOC_COMMENT.match(line)):
                    continue
                total += 1
                lo, hi = comment_block(lines, i)
                if not has_resolvable_pointer("\n".join(lines[lo : hi + 1])):
                    alarms.append(
                        (path.relative_to(root).as_posix(), i + 1, line.strip())
                    )
    print(f"stand-in markers (doc-comment `STAND-IN`): {total}")
    return alarms


def main() -> int:
    # A control that CRASHES on some terminals is a control people stop trusting.
    # Marker text routinely contains `⚠`, `—`, `§`; a Windows console defaulting to
    # cp1252 raises UnicodeEncodeError mid-report and the run dies AFTER printing a
    # partial finding — which reads exactly like a broken tool rather than a real
    # hit. Found on this script's first real run, 2026-08-04. Degrade the glyph,
    # never the verdict.
    for stream in (sys.stdout, sys.stderr):
        try:
            stream.reconfigure(encoding="utf-8", errors="replace")
        except (AttributeError, ValueError):  # not a reconfigurable TextIO
            pass

    root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parent.parent
    alarms = check(root)
    if not alarms:
        print("all markers carry a resolvable locus pointer.")
        return 0
    print(f"\n{len(alarms)} marker(s) with NO resolvable locus pointer:\n")
    for f, ln, text in alarms:
        print(f"  {f}:{ln}")
        print(f"      {text}")
    print(
        "\nEach needs EITHER a locus entry (stubs.md / spines.md § 3 / ROADMAP Owed)\n"
        "OR — if one already exists — the pointer written back into the code.\n"
        "A one-directional pointer is not a pointer (CLAUDE.md read-first item 5)."
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
