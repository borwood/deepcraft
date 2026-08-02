#!/usr/bin/env python
"""Pre-commit guard: reject a commit that introduces a duplicate ordinal in any
append-only numbered inventory.

WHY (ratified by the user 2026-08-02, after the FIFTH collision in 24 hours):
two parallel sessions share this checkout, and "check the inventory's tail at
merge" races the sibling's unpushed/unmerged work — journal 0135, corrections
#85/#86, stub #34, and journal 0138 all collided in one day, each caught only
by a later sweep or a lucky `ls`. Per the project's own doctrine (CLAUDE.md
§ Gates): do not answer a systematic failure with a rule asking people to
remember — make it a mechanism. This hook makes the SECOND committer fail
loudly at the moment the collision is created, whichever session it is.

Checked inventories (the staged state, `git ls-files -s` / staged blobs):
  - journal/NNNN-*.md          -> duplicate 4-digit prefixes (assets excluded:
                                  many assets legitimately share an entry's
                                  number)
  - journal/corrections.md     -> duplicate `## N.` headers
  - docs/design/stubs.md       -> duplicate `### N.` headers

Slug-only / PENDING-headed artifacts pass untouched — that protocol
(session-workflow § reserve-at-dispatch) is unchanged; this guard fires only
when an ordinal is actually taken twice.
"""

import re
import subprocess
import sys
from collections import Counter

FAIL = 0


def staged_blob(path: str) -> str:
    """The staged content of *path* ('' if not present)."""
    try:
        return subprocess.run(
            ["git", "show", f":{path}"],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            check=True,
        ).stdout
    except subprocess.CalledProcessError:
        return ""


def complain(msg: str) -> None:
    global FAIL
    FAIL = 1
    print(f"ORDINAL COLLISION: {msg}", file=sys.stderr)


def check_journal() -> None:
    out = subprocess.run(
        ["git", "ls-files", "--cached", "--", "journal/*.md"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout.splitlines()
    nums = Counter()
    names: dict[str, list[str]] = {}
    for path in out:
        name = path.rsplit("/", 1)[-1]
        m = re.match(r"^(\d{4})-", name)
        if m:
            nums[m.group(1)] += 1
            names.setdefault(m.group(1), []).append(name)
    for n, count in nums.items():
        if count > 1:
            complain(f"journal ordinal {n} is claimed by {count} files: {names[n]}")


def check_headers(path: str, pattern: str, label: str) -> None:
    text = staged_blob(path)
    if not text:
        return
    nums = Counter(re.findall(pattern, text, flags=re.MULTILINE))
    for n, count in nums.items():
        if count > 1:
            complain(f"{label} #{n} appears {count} times in {path}")


def main() -> int:
    check_journal()
    check_headers("journal/corrections.md", r"^## (\d+)\.", "correction")
    check_headers("docs/design/stubs.md", r"^### (\d+)\.", "stub")
    if FAIL:
        print(
            "\nA numbered inventory has two entries with one ordinal. The other\n"
            "claimant is probably a parallel session's work already in the tree.\n"
            "Fix: renumber YOUR entry to the next free ordinal (the later claimant\n"
            "renumbers - the 3cab931/c546d19/462d903 precedent), re-point your own\n"
            "citations, and commit again. Do not delete the sibling's entry.",
            file=sys.stderr,
        )
    return FAIL


if __name__ == "__main__":
    sys.exit(main())
