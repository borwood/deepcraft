#!/usr/bin/env python3
"""One-off ROADMAP archiving helper — written 2026-07-29 for the 129 -> 85 pass.

RESCUED FROM SCRATCHPAD AT WRAP, VERBATIM AND UNPARAMETERISED. The ROOT below is
hardcoded to a worktree that no longer exists; edit it before use. Kept because
ROADMAP archiving is a RECURRING chore -- the file-size hook fires on every edit
(5,176 lines against a 2,500 threshold for class REGISTRY) and the sanctioned fix
is archiving resolved entries, by STATUS never by age. This does the mechanical
half: locate the Observed section, split it into top-level bullet blocks, and move
whole blocks verbatim to ROADMAP-history.md.

It does NOT decide what to archive. That judgement stays with a reader who checks
each entry at source -- an audit finding is a hypothesis, not an authority.
"""
import io, sys, re

ROOT = r"B:/repos/borwood/deepcraft/.claude/worktrees/agent-a71a27b14865de17c/"
RM = ROOT + "ROADMAP.md"
HI = ROOT + "ROADMAP-history.md"

lines = io.open(RM, encoding="utf-8").read().split("\n")

# locate Observed section
start = next(i for i, l in enumerate(lines) if l.startswith("## Observed"))
end = next(i for i, l in enumerate(lines) if i > start and l.startswith("## NEXT SESSION"))

# top-level bullet starts
starts = [i for i in range(start, end) if lines[i].startswith("- ")]

def block(idx):
    """return (s,e) half-open line range for the top-level bullet at position idx in starts"""
    s = starts[idx]
    e = starts[idx + 1] if idx + 1 < len(starts) else end
    # trim trailing blanks / the --- separator
    while e > s and (lines[e - 1].strip() == "" or lines[e - 1].strip() == "---"):
        e -= 1
    return s, e

def find(anchor):
    hits = []
    for k in range(len(starts)):
        s, e = block(k)
        body = "\n".join(lines[s:e])
        if anchor in body:
            hits.append(k)
    if len(hits) != 1:
        raise SystemExit("ANCHOR %r -> %d hits %s" % (anchor, len(hits), hits))
    return hits[0]

STRUCK = [
 ("THE SHIPPED WORLD HAS ZERO COAL",
  "The (a)-(d) world-content call is **withdrawn**. Made against an old implementation and the "
  "loop was never closed as the project moved on. *Its sibling entry — \"the only test defending "
  "'a player can find and dig a coal seam' runs on a world no player can open\" — was NOT struck "
  "and stays live in `ROADMAP.md` \u00a7 Observed.*"),
 ("Loose materials do not exist in the world yet",
  "Stale. Its own premise was already half-falsified (journal/0055 falsified *\"worldgen does not "
  "yet emit sub-8 loose voxels\"* world-wide, `ROADMAP.md` \u00a7 Observed \"HOLES IN THE GROUND\"), "
  "and the report as written describes an implementation the project has moved past."),
 ("coal renders as pure black in the",
  "Stale. Already annotated un-walkable on the shipped world (zero coal, corrections #51) and now "
  "withdrawn outright. The **lighting/tonemap** question it raised — no floor under the dark end of "
  "the lit path — is not carried by this entry; it belongs to PBR-2's shadow work, which owns it."),
 ("Material identity is illegible under splat blending",
  "Stale. A user sketch (*\"not the only possible answer, just a thought\"*) filed against the "
  "2026-07-22 render shape; withdrawn."),
 ("Olivine reads as exceedingly common and surface-visible (user field",
  "Stale. Filed 2026-07-20 against the then-current accessory roster; withdrawn."),
 ("Walk 0059 \u2014 the holes and the chunk patches are gone",
  "Stale. **Its two YES answers are load-bearing and are preserved here**: *no sky-holes* at two "
  "partial-rich stations (journal/0057 confirmed by eye) and *no 28.8 m chunk patches* "
  "(journal/0058 confirmed) \u2014 this entry is what discharges the two **UNWALKED** tags still "
  "carried live in `ROADMAP.md` \u00a7 Observed, which now cross-reference it here. Assets `0059-*`."),
 ("The bare-cell fallback: a walker stood on paint over nothing",
  "Stale. Coordinates preserved below (deep cell (488, 278), world metres (106 938, 9 953)) and the "
  "falsified wind-banding hypothesis with them."),
 ("filed at the FF2a/0024 ratification",
  "**The user's own reasoning, and it is the stronger kill:** *grass and dirt are not generated in "
  "the current shape of the default plugin pack, so the observation itself is stale.* Not merely "
  "that the named mechanism retired \u2014 the thing observed is not produced by the world any more.\n"
  "  **\u26a0 The S6 audit's recommended re-shoot of `0024-fb-ne.png`'s framing is CANCELLED** by this "
  "ruling. Do not re-derive it: there is nothing at that vantage to re-photograph. Assets "
  "`0024-after-ne.png` / `0024-fb-ne.png` kept for the record."),
]

RIP = [
 "lagged terrain read is undeclared**~~",
 "OWED (small, as diagnosed)",
 "A TIE-BREAK IS DECIDING PHYSICS AGAIN",
 "OWED / next residency lever \u2014 a per-cell OWNING CONTAINER",
 "DONE 2026-07-25 \u2014 shipped, see",
 "the front's voxel-tier mass error is NOISE",
 "`pore_rider_share`'s offset is NOT disjoint from",
 "the query can now say `UNRECORDED`",
 "FREE WIN TAKEN: 54.02 MiB reclaimed",
 "Dev slice \u2014 look-at-voxel contents inspector \u2014 SHIPPED 2026-07-24",
 "Deep-cell-square surface-material frontiers checker the far field",
 "systematically under-expresses",
 "Caves \u2194 hydrology integration thread captured",
 "UPDATE 2026-07-21 (second occurrence)",
 "CONFIRMED AT HORIZON 6, 2026-07-21",
 "RESOLVED IN THE SAME ENTRY",
 "FIXED 2026-07-21 (journal/0051): eviction landed",
 "DIAGNOSED 2026-07-21 (journal/0050): the leak is host-RAM",
 "Sub-km relief / roughness decay: MEASURED",
 "`climate_at` half-cell offset: CONFIRMED BUG, FIXED",
 "The far field cuts off at 1.2 km",
 "Material placement rules are climate mocks: DECIDED 2026-07-21",
 "Console v1 field report: FIXED same day",
 "lit before/after is invalid because the sun moves",
 "thin bright seams between far",
 "the far LOD sheet is buried under the near field",
 "clear pixel gaps between far-field tiles \u2014 RESOLVED",
 "Walk 17 (journal/0022 \u00a7 walk 17",
 "Walk 16 (journal/0021 \u00a7 walk 16)",
 "blocking regression",
 "Instrument fix: pose replies echo the voxel coordinate \u2014 DONE",
 "Walk 13 (journal/0018)",
 "Walk 5 (journal/0005, character surface)",
 "~2/3 of far-mesh triangles were sealed cave surfaces",
 "`client_player_pose_set` outside the one door: RATIFIED",
 "Console follow-ups: RATIFIED same day",
 "The dominance flip quantizes smooth gradients",
]

# extra notes keyed by anchor, for RIP entries needing a caution
RIP_NOTE = {
 "FREE WIN TAKEN: 54.02 MiB reclaimed":
  "\u26a0 **Dated denominators inside \u2014 do not propagate.** This entry's *\"today's total is 108.55 "
  "MiB\"* and its `S19` note are 2026-07-25 figures taken **before** journal/0102's ledger collapse "
  "moved `DeepField` residency again the same afternoon. The archived text is the dated record; the "
  "numbers were **not** re-measured by this archive pass. Its closing claim that the `FactLedger` "
  "`Vec<Vec<Fact>>` sibling defect *\"is NOT fixed by this and remains\"* was **superseded hours "
  "later** by journal/0102 (archived above).",
 "A TIE-BREAK IS DECIDING PHYSICS AGAIN":
  "S6 finding **F4**, verified 2026-07-29: both halves carry \u2705 inline (option (a) shipped "
  "journal/0104; head's under-declaration shipped journal/0107) and only the kept-for-the-record "
  "diagnoses remained.",
}

picked = []
for a, why in STRUCK:
    picked.append((find(a), "STRUCK", why))
for a in RIP:
    picked.append((find(a), "RIP", RIP_NOTE.get(a)))

seen = {}
for k, kind, why in picked:
    if k in seen:
        raise SystemExit("DUPLICATE block %d" % k)
    seen[k] = 1

print("struck=%d rip=%d total=%d of %d bullets" % (
    len(STRUCK), len(RIP), len(picked), len(starts)))

# ---- build history text ----
def render(kind, why, s, e):
    out = []
    if kind == "STRUCK":
        out.append("**STRUCK (user, 2026-07-29)** \u2014 " + why)
    else:
        out.append("**ARCHIVED 2026-07-29 \u2014 resolved in place** (the entry's own body already "
                   "declared it closed; moved by status, not by age).")
        if why:
            out.append("")
            out.append(why)
    out.append("")
    out.extend(lines[s:e])
    out.append("")
    return out

hist = []
hist.append("## Observed \u2014 archived")
hist.append("")
hist.append("**Archived from `ROADMAP.md` \u00a7 Observed on 2026-07-29**, by STATUS, not by age \u2014 the")
hist.append("same rule as \u00a7 Shipped above. Sourced from the complete classification in")
hist.append("[`docs/audits/baseline-2026-07-28/S6-roadmap-observed.md`](docs/audits/baseline-2026-07-28/S6-roadmap-observed.md),")
hist.append("with every finding re-verified at source before the move.")
hist.append("")
hist.append("**Entries are reproduced VERBATIM.** Recorded camera poses, world coordinates and asset")
hist.append("filenames travel with them (corrections #48 \u2014 *a prose landmark is not a pose*); nothing")
hist.append("was summarised away and nothing was deleted. Each entry keeps its **journal number**, which")
hist.append("is the stable pointer.")
hist.append("")
hist.append("### Struck by the user, 2026-07-29 \u2014 eight field reports")
hist.append("")
hist.append("The user reviewed the open field reports on 2026-07-29 and struck these eight. Their stated")
hist.append("general reason, which is the frame for all eight:")
hist.append("")
hist.append("> *\"plenty of these are going to be stale and were made in the context of old")
hist.append("> implementations and we didn't close the loop as the project moved on.\"*")
hist.append("")
hist.append("These are **user field reports** and the audit trail matters, so they are struck here rather")
hist.append("than deleted. A struck report is **not** a resolved defect \u2014 it is an observation the user")
hist.append("has ruled no longer describes the world. Do not re-derive them.")
hist.append("")
for k, kind, why in picked:
    if kind != "STRUCK":
        continue
    s, e = block(k)
    hist.extend(render(kind, why, s, e))
hist.append("### Resolved in place \u2014 37 entries whose own bodies already said DONE")
hist.append("")
hist.append("The S6 baseline sweep found **37 of 129** \u00a7 Observed entries (29 %) already declaring")
hist.append("themselves \u2705 DONE / RESOLVED / FIXED in their own text, kept live \"for the record\". They are")
hist.append("moved here whole. This is the archive-by-status rule applied mechanically: **no judgement")
hist.append("call, no information loss.** Where an archived entry carries a *measurement*, that")
hist.append("measurement is a **dated record** \u2014 it was not re-measured by this pass and must not be")
hist.append("propagated without checking (\u00a7 Read first item 5, immutable body / mutable header).")
hist.append("")
for k, kind, why in picked:
    if kind != "RIP":
        continue
    s, e = block(k)
    hist.extend(render(kind, why, s, e))

# ---- splice into history ----
h = io.open(HI, encoding="utf-8").read().split("\n")
mark = next(i for i, l in enumerate(h) if l.startswith("# Superseded close blocks"))
h2 = h[:mark] + hist + h[mark:]

# update the file's own intro list (2 parts -> 3)
txt = "\n".join(h2)
old = """2. **Superseded close blocks** \u2014 each was consumed by the one after it."""
new = """2. **Observed \u2014 archived** \u2014 \u00a7 Observed entries that stopped requiring a live read: user field
   reports the user has **struck**, and entries whose own bodies already declared them closed.
3. **Superseded close blocks** \u2014 each was consumed by the one after it."""
assert old in txt
txt = txt.replace(old, new, 1)
txt = txt.replace(
    "whose narrative is already carried elsewhere:",
    "whose narrative is already carried elsewhere:", 1)
txt = txt.replace(
    "This file holds the two parts that are least often needed live and",
    "This file holds the parts that are least often needed live and", 1)
io.open(HI, "w", encoding="utf-8", newline="\n").write(txt)

# ---- remove from ROADMAP ----
drop = set()
for k, kind, why in picked:
    s, e = block(k)
    # also swallow following blank lines up to the next bullet
    ee = e
    nxt = starts[k + 1] if k + 1 < len(starts) else end
    while ee < nxt and lines[ee].strip() == "":
        ee += 1
    for i in range(s, ee):
        drop.add(i)

kept = [l for i, l in enumerate(lines) if i not in drop]
io.open(RM, "w", encoding="utf-8", newline="\n").write("\n".join(kept))
print("removed %d lines from ROADMAP.md" % len(drop))
