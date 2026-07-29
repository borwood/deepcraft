#!/usr/bin/env python3
"""ROADMAP archiving pass 2 -- In flight + Sequenced + Observed, 2026-07-29.

WHAT THIS IS. The successor to `roadmap_archive.py`, which did the first pass of the day
(37 self-declared-resolved Observed entries + 8 user strikes) and was written unparameterised
against a worktree that no longer exists. This one is parameterised: set ROOT below.

WHAT IT CONSUMES. `docs/audits/2026-07-29-roadmap-classification.md` -- a full read of every
top-level entry in the three live sections, verified at source. That audit is the authority
for WHY each item moves; this file is only the mover. **An audit finding is a hypothesis, not
an authority: the classification was re-verified at source before it was written here, and
should be spot-checked again before this is run.**

WHAT IT DOES NOT DO.
  - It does not decide anything. Every anchor below came from the audit.
  - It does not touch STRIKE-CANDIDATES. Five items are user-owned (two 2026-07-21 appearance
    ratifications, two field reports resting on a premise the user struck this morning, and the
    ores fork). They stay on the live board until the user rules. See audit section 5.
  - It does not create new entries. Two moves carry an OBLIGATION the human running this must
    discharge in the same commit -- see OBLIGATIONS below.

THREE OPERATION KINDS.
  MOVE   -- a whole top-level bullet block, located by a unique anchor, moved verbatim.
  RANGE  -- an inclusive line range, located by a start and end anchor, scoped either to a
            single bullet block (a SHRINK-IN-PLACE: the live head stays, a closed sub-thread
            moves) or to a section (for the un-bulleted prose in In flight).
  FIXUP  -- a unique in-place string replacement in ROADMAP.md, applied AFTER removal, to
            repair a pointer whose target just moved. A one-directional pointer is not a
            pointer; a pointer into a hole is worse.

OBLIGATIONS ON THE HUMAN RUNNING THIS -- both are in the moved text's banner, and both must be
discharged in the SAME COMMIT or the move loses something live:
  1. The octree-substrate move takes four unowned follow-ons with it: persistence + dirty-rail
     (far edits), synthesized sub-surface strata, partial-coverage composition, deep-span
     greedy merge. Re-file them as one live line in section Sequenced.
  2. The APPEARANCE WALKS OWED move takes one live residue with it: the poke-through geometry
     check on the lit pass (low priority). Re-file or accept its loss deliberately.

USAGE.
    python scripts/roadmap_archive2.py --check    # validate anchors, print ranges, write nothing
    python scripts/roadmap_archive2.py            # apply
"""
import io
import sys

# ---------------------------------------------------------------- configuration

ROOT = "B:/repos/borwood/deepcraft/"
RM = ROOT + "ROADMAP.md"
HI = ROOT + "ROADMAP-history.md"

DATE = "2026-07-29"
AUDIT = "docs/audits/2026-07-29-roadmap-classification.md"

# section name -> (line-prefix that opens it, line-prefix that closes it)
SECTIONS = {
    "In flight": ("## In flight", "## Sequenced"),
    "Sequenced": ("## Sequenced", "## Observed"),
    "Observed": ("## Observed", "## NEXT SESSION"),
}

# ---------------------------------------------------------------- the operations
#
# Each op is a dict:
#   kind    "move"  -> whole bullet block containing `anchor`
#           "range" -> inclusive [start .. end], scoped by `block` (a bullet anchor) or by
#                      `section` alone
#   sect    which section to search
#   cls     "MECHANICAL" | "POINTER" | "SHRINK"
#   why     the banner written above the moved text in ROADMAP-history.md
#
# Anchors are checked for uniqueness inside their scope before anything is written.

OPS = [
    # ============================================================ In flight
    dict(kind="range", sect="In flight", cls="MECHANICAL",
         start="**Session 3 shipped, all gates green on merged main:**",
         end="corrections filed (#11",
         why="A shipped ledger for journals 0023-0030. Every entry it names is in "
             "`ROADMAP-history.md` \u00a7 Shipped with its journal number as the stable pointer."),

    dict(kind="range", sect="In flight", cls="MECHANICAL",
         start="**Session 5 shipped (2026-07-21, journal/0039",
         end="(ores.md R1",
         why="A shipped ledger for journals 0039-0049. Same disposition as the session-3 "
             "ledger above."),

    dict(kind="range", sect="In flight", cls="MECHANICAL",
         start="**(SUPERSEDED by the 2026-07-22 close at the end of this file.)**",
         end="2026-07-21, journal/0065: it survives.**",
         why="**The entry declares its own supersession in its first six words.** A "
             "2026-07-21 close block that survived the "
             + DATE + " close-block archive only because it is not written under a "
             "`## NEXT SESSION` heading. Its walk directive was discharged the next day "
             "(journal/0057-0059) and its item (5) already carries `DONE 2026-07-21`."),

    dict(kind="range", sect="In flight", cls="MECHANICAL",
         start="**Wide horizons: the blocker is GONE and now PROVEN at 6**",
         end="watch there is `far_tiles` at fill, not slope.",
         why="Self-declared resolved (journal/0065, 2026-07-21). **It is also the refutation "
             "of the `GPU DeviceLost crash under a teleport storm at --horizon 6` entry "
             "archived below** -- the two are moved in the same commit so the claim does not "
             "outlive its refutation, which is the failure this whole pass exists to stop. "
             "*Its measurements are a dated record and were not re-taken.*"),

    dict(kind="range", sect="In flight", cls="MECHANICAL",
         start="*(Superseded by a session-close block; kept for the record.",
         end="see the question you are asking.",
         why="Self-declared superseded, with a `RE-SEQUENCED 2026-07-21` list whose items all "
             "carry their own `SHIPPED`/`DONE`/`MEASURED` stamps, and a `Read first next "
             "session` directive four close blocks stale."),

    dict(kind="move", sect="In flight", cls="POINTER",
         anchor="The octree substrate \u2014 DESIGN PASS OPENED, D1\u2013D3 DECIDED",
         why="**Shipped and ratified in its own body:** *\"MERGED TO MAIN + LOOK RATIFIED "
             "AS-BUILT (2026-07-22, integrator merge; user, from the 0070 screenshot set: "
             "'visually indistinguishable from the previous version, for me. which is "
             "good!')\"*. FF2b-minimal landed as journal/0070.\n"
             "\n"
             "  \u26a0 **FOUR FOLLOW-ONS TRAVEL WITH THIS ENTRY AND ARE OWNED NOWHERE ELSE** "
             "-- persistence + dirty-rail (far edits), synthesized sub-surface strata "
             "(the `surface_sample` summarization-half home), partial-coverage composition, "
             "and deep-span greedy merge. They were live when this moved. **Re-file them on "
             "the live board**; the archive is not a home for an unowned obligation."),

    dict(kind="move", sect="In flight", cls="POINTER",
         anchor="**Tectonics architecture RATIFIED 2026-07-20** \u2014 all of U1\u2013U8",
         why="A pointer to a spike that has had its own \u00a7 Sequenced entry since 2026-07-20 "
             "(*\"Tectonics SPIKE (per tectonics.md \u00a7 SPIKE, architecture ratified "
             "2026-07-20)\"*), and to a dispatch gate -- *\"behind the eolian agent's "
             "landing\"* -- discharged 2026-07-21."),

    dict(kind="move", sect="In flight", cls="MECHANICAL",
         anchor="Tectonic design pass \u2014 DRAFT LANDED (historical entry)",
         why="**Self-labelled *\"(historical entry)\"* in its own header.** "
             "`docs/design/tectonics.md` shipped 2026-07-20 and now carries a supersession "
             "banner of its own (added " + DATE + ", baseline sweep S2/F2)."),

    dict(kind="range", sect="In flight", cls="POINTER",
         start="**Zonal circulation profile** \u2014 dispatched same moment (write-set",
         end="measured shift for the user's eye.",
         why="Shipped 2026-07-20 as journal/0037. This board said so itself, one section "
             "over, for nine days -- the duplicate in \u00a7 Sequenced is archived below."),

    # ============================================================ Sequenced
    dict(kind="range", sect="Sequenced", cls="SHRINK",
         block="ALL FIVE USER DECISIONS RULED 2026-07-28 — journal/0120.**",
         start="*Original framing, kept: five calls surfaced 2026-07-28",
         end="walked from an end that holds no link.",
         why="**The five rulings and their reasoning.** The entry's own head says *\"Nothing "
             "here is owed\"*; all five carry `\u2705 RULED` and each shipped its record "
             "elsewhere (CLAUDE.md read-first item 5, `worldgen.md`'s ON-HOLD banner, "
             "`north-star.md` \u00a7 Materials, `spines.md` \u00a7 3's third exit, banners on "
             "`S10-results.md` and `S2-results.md`). **The live board keeps the head and the "
             "*Also surfaced, NOT user-owned* tail**, which still holds two open obligations."),

    dict(kind="range", sect="Sequenced", cls="SHRINK",
         block="THE BASELINE SWEEP'S FINDINGS",
         start="**\u2705 ALL FOUR USER CALLS RULED 2026-07-28**",
         end="Deviations 2 calls *\"EXPLICITLY NOT THE MODEL\"*.",
         why="**The four ruled user calls, and the five `HIGHEST BLAST RADIUS` items -- all "
             "five verified applied at source on " + DATE + ":** `spines.md` \u00a7 S-6 "
             "rewritten to the ratified authored-order shape \u00b7 the erosion-axis losing-side "
             "banner \u00b7 `tectonics.md`'s supersession banner \u00b7 the seam count corrected to "
             "*\"31 LIVE, of 34 inventoried\"* \u00b7 ROADMAP's first bullet struck "
             "(*\"THE TIERING IS RETIRED\"*). **The live board keeps the head and the "
             "STRUCTURAL / BULK / `flow_cost_probe` / NOT-COVERED-BY-THIS-BASELINE bullets** "
             "-- `flow_cost_probe` is still the one probe the gate cannot see fail."),

    dict(kind="move", sect="Sequenced", cls="MECHANICAL",
         anchor="SWEEPS RUN FIRST THING, INCREMENTALLY, AND THE HARNESS SAYS WHICH ARE DUE",
         why="`\u2705 DECIDED 2026-07-28` and both halves shipped: the `staleness-sweep` skill "
             "and `scripts/sweep_due_hook.py`, both present in the tree. Kept live only for "
             "its reasoning, which is preserved here whole -- including the honest limit that "
             "*\"a green sweep must never read as 'the corpus is sound'\"*."),

    dict(kind="range", sect="Sequenced", cls="SHRINK",
         block="DOC-TOPOLOGY RESIDUALS \u2014 the 19 findings not actioned 2026-07-26",
         start="**\u2705 DONE 2026-07-28 \u2014 the dangling cross-file pointers.**",
         end="not the rows.*",
         why="Three consecutive `\u2705 DONE 2026-07-28` sub-bullets: the 44 dangling pointers "
             "at 43 sites, `flow.md:715`'s RATIFIED stamp, and the `spines.md`/`stubs.md` "
             "scheduling heirs. **The live board keeps the residuals** -- the ABI-spike "
             "contest, the coal evidence base (which needs a measurement, not an edit), and "
             "the next sweep's spine."),

    dict(kind="move", sect="Sequenced", cls="MECHANICAL",
         anchor="\u2705 DONE 2026-07-26 \u2014 THE DOC-TOPOLOGY SWEEP + THE ROADMAP ARCHIVE",
         why="`\u2705 DONE`; both shipments verified present (`.claude/skills/doc-topology/"
             "SKILL.md`, `ROADMAP-history.md`). Kept for the three-way VOLUME / TOPOLOGY / "
             "AUTHORITY diagnosis, which is preserved here -- including its own honest limit, "
             "*\"the archive buys less than it looks like it does\"*, a judgement this second "
             "archiving pass confirms."),

    dict(kind="range", sect="Sequenced", cls="SHRINK",
         block="THE PASS ARCHITECTURE \u2014 AUTHORED ORDER, OPEN VOCABULARY",
         start="*Record of how the slice was framed before it shipped",
         end="`stubs.md` \u00a7 30 carries the stand-in with RATE named as its heir.",
         why="**The pre-ship framing of the RATE slice**, which shipped " + DATE + " as "
             "journal/0123. Its own opening says it is a *\"record of how the slice was framed "
             "**before it shipped**\"*. **The arc itself is LIVE and stays** -- (1) authored "
             "ORDER and (2) the open vocabulary are owed, the continuation slot (b)-(f) is "
             "open, and the `\u2705 RATE IS NOT EXPANDED` user ruling that follows this range "
             "stays on the live board because it is a standing constraint on the S-10 spine."),

    dict(kind="move", sect="Sequenced", cls="MECHANICAL",
         anchor="\u2705 DONE 2026-07-28 \u2014 THE BOOTSTRAP HISTORY CONTENT IS REMOVED",
         why="`\u2705 DONE`, merged and gate-verified (85 binaries / 808 passed), "
             "`corrections #66`, narrative in journal/0121. **Checked before moving: its one "
             "live residual -- `approx_resident_bytes` is non-monotone in extent -- is "
             "duplicated in the surviving tail of the five-rulings entry, so it stays on the "
             "live board.** Moving both copies would have dropped an observation."),

    dict(kind="move", sect="Sequenced", cls="MECHANICAL",
         anchor="REMOVE THE BOOTSTRAP HISTORY CONTENT \u2014 polities, sites, ruins, the history pass",
         why="The *\"Original entry, preserved\"* half of the `\u2705 DONE` removal above. "
             "Verified at source " + DATE + ": `grep -rn \"ruin_posts\\|pregen/history\" "
             "crates/ --include=*.rs` returns nothing."),

    dict(kind="move", sect="Sequenced", cls="POINTER",
         anchor="THE HILLSLOPE CONVEYOR CHECKERBOARDED THE REGOLITH ABOVE 1\u00d7 \u2014 stubs #29",
         why="**Its own header reads `\u2705 FIXED`, and `docs/design/stubs.md` \u00a7 29 agrees:** "
             "*\"\u26a0 DISCHARGED " + DATE + " (journal/0122) -- the operator is fixed, and one "
             "inference below is falsified (corrections #72).\"* The live successor is "
             "`ROADMAP.md` \u00a7 Sequenced *\"THE HILLSLOPE OPERATOR IS FIXED -- SHIPPED "
             + DATE + "\"*, which now points here rather than *\"below\"*.\n"
             "\n"
             "  \u26a0 **READ corrections #72 BEFORE CITING THE (b) BLOCK BELOW.** *\"It is NOT "
             "a time-step limit\"* is **wrong**: it was a stability limit, and journal/0116's "
             "4\u00d7 refinement was ~25\u00d7 short of reaching the bound. Every measurement in "
             "the entry stands; that one inference does not. The `iso_rate` finding in (c) "
             "-- *\"do not touch `iso_rate`, it is the only grid-scale low-pass in the "
             "solve\"* -- is unaffected and is still worth reading."),

    dict(kind="move", sect="Sequenced", cls="POINTER",
         anchor="THE TRANSPORT OPERATOR HAS A CEILING \u2014 stubs #27",
         why="**Its headline mechanism is falsified.** `docs/design/stubs.md` \u00a7 27, " + DATE
             + ": *\"\u26a0 THE CONVEYOR IS GONE " + DATE + " (journal/0122) -- and the "
             "calibration fitted on top of it does not survive. The cap was a symptom of the "
             "same defect as \u00a7 29.\"* The entry still asserted *\"the pass is a "
             "one-cell-per-epoch conveyor\"* as a live finding. **Its surviving contribution "
             "-- journal/0114's six measured 200-epoch worlds -- is carried forward as INPUT "
             "by the live `RE-PICK EROSION_CALIBRATION AGAINST THE FIXED OPERATOR` entry**, "
             "whose own ladder inverts this one's headline (cover now *thins* with the "
             "multiplier where it used to thicken)."),

    dict(kind="range", sect="Sequenced", cls="SHRINK",
         block="REQUIRED CHORE \u2014 FILE SIZE IS A CORRECTNESS PROBLEM",
         start="**\u2705 CONVENTIONS DECIDED 2026-07-28 (user)**",
         end="that lands.",
         why="`\u2705 CONVENTIONS DECIDED`, and **the entry itself names where the record now "
             "lives**: *\"shipped in `scripts/filesize_hook.py`, whose module docstring is now "
             "the record (the hook is the only mechanically-enforced corpus control, so the "
             "convention lives where it is enforced)\"*. **The live board keeps the chore's "
             "open half** -- immediate for new files, gradual refactor of old work when "
             "touched. *This archiving pass is that chore, executed on its largest subject.*"),

    dict(kind="range", sect="Sequenced", cls="SHRINK",
         block="Finish the draw-domain conversion: the residual hand-rolled sites",
         start="(corrections #64, a read-only trace taken",
         end="`GOLDEN_SURFACE` / `GOLDEN_RECORD` are **not** downstream (deep-time field only).",
         why="**The 2026-07-26 consumer trace, which the entry itself labels a dated record** "
             "(*\"Read the trace below as the 2026-07-26 record it is\"*). Every consumer it "
             "traces -- `pregen/history.rs`, `Pregen.sites`, `collapse.rs::ruin_posts`, the "
             "`Block::Wood` emission -- was deleted 2026-07-28 (journal/0121). **The live "
             "board keeps (a), (b) and (c)**, which are now byte-identical housekeeping."),

    dict(kind="range", sect="Sequenced", cls="MECHANICAL",
         start="*(The `production_* \u2192 golden_*` rename that stood here",
         end="**shipped 2026-07-26**; see `ROADMAP-history.md` \u00a7 Shipped.)*",
         why="Self-declared shipped, and its Shipped entry is the first one in this file."),

    dict(kind="range", sect="Sequenced", cls="SHRINK",
         block="FLOW IS ONE PROCESS \u2014 flux on FACES, facts on STRATA",
         start="~~**\U0001f534 OWED BY MFD, and it is the nearest-term thing on this arc",
         end="not a later critique.*",
         why="Already struck in place on " + DATE + " (baseline sweep S5/F4): *\"\u2705 SHIPPED "
             "2026-07-26 -- both terms are stale.\"* Hybrid-`p` is the shipped default and the "
             "`k_bedrock`/`k_transport` recalibration rode the joint calibration. **The FLOW "
             "arc stays live** -- (b'), (c), (d), (e) are unbuilt and its three riders are "
             "still in the slot."),

    dict(kind="move", sect="Sequenced", cls="POINTER",
         anchor="APPEARANCE WALKS OWED** (tracking, user:",
         why="**Nothing in it is owed.** (1) `\u2705 DONE -- walk-confirmed 2026-07-24`; "
             "(2) `\u2705 DONE -- walk-confirmed & ACCEPTED 2026-07-25`; (3) tour-mapped to a "
             "**null** -- the shipped world has zero coal, *\"do not spend a walk on it\"* "
             "(corrections #51); (4) `\u2705 DONE -- WALK-CONFIRMED & PASSED 2026-07-25`; "
             "(5) `NEVER OWED -- ANSWERED AT THE DESK`.\n"
             "\n"
             "  \u26a0 **ONE RESIDUE TRAVELS WITH THIS ENTRY:** *\"the poke-through geometry "
             "check on the lit pass -- low priority\"*, left over from walk (1). It was live "
             "when this moved. The next owed appearance walk should open a fresh tracker "
             "rather than resurrect this one -- an empty tracker is not a live entry, but a "
             "residue inside an archive is a lost obligation."),

    dict(kind="move", sect="Sequenced", cls="MECHANICAL",
         anchor="`FactLedger` IS 89 % EMPTY HEADERS",
         why="`\u2705 DONE 2026-07-25` (journal/0100), with the measured result in its own head: "
             "`DeepField` 311.02 \u2192 179.12 MiB, ledger heap 155.01 \u2192 16.31 MiB, world "
             "byte-identical. Kept *\"as shaped, for the record\"*. **Its correction is worth "
             "carrying forward: triangularity is a budgeting tool for projecting an UNBUILT "
             "record, never a sizing rule for a BUILT one.** *The MiB figures are a dated "
             "record; residency has moved since and they were not re-measured.*"),

    dict(kind="move", sect="Sequenced", cls="MECHANICAL",
         anchor="THE WEATHERING FRONT NEEDS A PROFILE, NOT A SLAB",
         why="`\u2705 SHIPPED 2026-07-25 (journal/0099)`, kept for its reasoning. The flag-ON "
             "walk it earned was item (4) of APPEARANCE WALKS OWED, also archived here, also "
             "`\u2705 DONE` -- user at the station: *\"Success on the gradation! ... Our world "
             "just got far deeper and more interesting to look at, just with this.\"*"),

    dict(kind="range", sect="Sequenced", cls="SHRINK",
         block="THE HONEST IDENTITY SURFACE \u2014 retire the stored `Block` summary",
         start="**~~TIER BOUNDARIES DERIVE FROM THE LOD LADDER~~ \u2014 SUPERSEDED SAME DAY",
         end="still owed from journal/0091.",
         why="**Three consecutive blocks the entry itself marks superseded** -- "
             "`SUPERSEDED SAME DAY`, `DISSOLVED 2026-07-25 with the tiers themselves`, and "
             "`PRESERVED FOR THE RENDER-SIDE QUERY` -- all retired by the user's UNTIERED "
             "decision stated fifty lines above them (*\"if this is a world query then why "
             "tiered at all when we can inspect chunk and read the voxel?\"*). **The arc stays "
             "live**: its continuation slot (storage/wire migration, the runtime edit-fact "
             "overlay, far-span `Block`\u2192material, legacy-S1 retire) is unbuilt."),

    dict(kind="move", sect="Sequenced", cls="POINTER",
         anchor="Collapse-cache `evict()` is unreachable from far-field-only sampling",
         why="**Verified false at source " + DATE + ".** `crates/dc-worldgen/src/collapse.rs` "
             "calls `self.evict()` at `:451`, `:711`, `:997`, `:1012`, `:1022`, `:1033` and "
             "`:1041` -- reachable from every sampling path, not only `generate_chunk`. Fixed "
             "by journal/0052, whose \u00a7 In flight entry records it by name: *\"Also landed: "
             "the journal/0050 collapse-cache `evict()` gap (now reachable from "
             "`coarse_surface`, `column_record`, `surface_elev_m`, `lattice_point`, "
             "`surface_chunk_y`)\"*. **A claim and its own refutation, ~2,500 lines apart, in "
             "the one file every session opens.**"),

    dict(kind="move", sect="Sequenced", cls="MECHANICAL",
         anchor="`--horizon <km>`, default provably unchanged, measured to 10 km.",
         why="Self-declared SHIPPED 2026-07-21, journal/0042."),

    dict(kind="move", sect="Sequenced", cls="MECHANICAL",
         anchor="Wave-magnitude retune \u2014 STRUCK 2026-07-21 (user): no retune.",
         why="**Struck by the user in its own header**, with the reasoning quoted: *\"that "
             "whole mechanism changes after water machinery. that would be a bandaid, against "
             "our standing rule against bandaids. can revisit later.\"* The confirmed null "
             "(0.68 m) rides as-built until the fetch model replaces the constant -- which is "
             "the live littoral-heir line in \u00a7 In flight."),

    dict(kind="move", sect="Sequenced", cls="MECHANICAL",
         anchor="**Zonal circulation profile: SHIPPED** 2026-07-20, journal/0037",
         why="Self-declared SHIPPED. The duplicate of this entry in \u00a7 In flight is archived "
             "above; the two sat in different sections of one file for nine days."),

    dict(kind="move", sect="Sequenced", cls="MECHANICAL",
         anchor="**Deep-config flag plumbing: SHIPPED** 2026-07-20, journal/0039",
         why="Self-declared SHIPPED. Its four launch flags are still the door every dev "
             "override rides through (`DeepOverrides` on top of `production_config`)."),

    dict(kind="move", sect="Sequenced", cls="POINTER",
         anchor="Erosion-supply calibration** (from the S12 spike's new finding,",
         why="**Its stated deliverable was done, and it is the largest measurement on the "
             "board.** The entry asks to *\"compare model denudation against real orogen "
             "rates\"*; journal/0111 did exactly that with `examples/denudation_probe.rs` and "
             "its cited literature table, finding the shipped world denudes at 0.0110 m/Myr -- "
             "**9\u00d7 slower than the slowest landscape ever measured on Earth.** The live "
             "successors are `CALIBRATE THE DEEP-TIME CLOCK` and `RE-PICK "
             "EROSION_CALIBRATION`, both of which carry this entry's method rule (against a "
             "published band, never against a look) as their binding constraint."),

    dict(kind="move", sect="Sequenced", cls="MECHANICAL",
         anchor="Tectonic uplift-plane redesign \u2014 DESIGN PASS (superseded \u2014 done)",
         why="**Self-labelled *\"(superseded -- done)\"*.** The direction it names shipped as "
             "the U8 tectonic flip (journal/0044)."),

    dict(kind="range", sect="Sequenced", cls="POINTER",
         start="*(**FF2a \u2014 voxel-language far field: SHIPPED** 2026-07-19, journal/0023",
         end="edit-tracked LOD store stay drop-in).",
         why="FF2a shipped (journal/0023). The FF2b paragraph attached to it is superseded "
             "twice over by the octree-substrate entry archived above: *\"this supersedes "
             "FF2b's earlier pairing with the caves/underground water thread for the minimal "
             "slice\"*, and `FF2b-minimal LANDED` (journal/0070). **The two follow-ons it "
             "names -- async meshing and a persistent edit-tracked LOD store -- survive in "
             "\u00a7 Observed's Voxy-vs-Distant-Horizons entry, which stays live.**"),

    dict(kind="range", sect="Sequenced", cls="MECHANICAL",
         start="*(**Erodibility coupling \u2014 lithology-aware erosion: SHIPPED**",
         end="agent-specific resistance so karst/glacial/littoral stay implementable.", extra=1,
         why="Self-declared SHIPPED 2026-07-20 (journal/0029). The user decisions it points "
             "at stay on the live board immediately below where this stood."),

    dict(kind="range", sect="Sequenced", cls="SHRINK",
         start="1. ~~**Flipping `production_config`'s `erodibility` to true**~~ **RATIFIED AND",
         end="makes item 2 below the live question, not a footnote.",
         why="Struck in place: *\"RATIFIED AND DONE 2026-07-20 (user: 'flip it, i want to "
             "see')\"*. **The live board keeps items 2 and 3, the " + DATE + " losing-side "
             "banner, and journal/0079's record** -- item 2 is an explicit open user call "
             "(*\"whether the axis reopens, and on what terms, is the USER'S call\"*)."),

    dict(kind="range", sect="Sequenced", cls="SHRINK",
         start="Original charter (for the record):",
         end="feedback stability clamped; off-by-default and byte-identical.",
         why="**Self-labelled *\"(for the record)\"***, describing a gap closed 2026-07-20: "
             "`erosion.rs` incising bedrock with a single global `k_bedrock`. Erosion has "
             "been lithology-aware since journal/0029."),

    dict(kind="range", sect="Sequenced", cls="SHRINK",
         start="1. ~~**The world-creation ritual grows 15 s \u2192 25 s (+66 %)**~~ \u2014 **RATIFIED",
         end="changed for every world created from here.",
         why="S10 ratification calls 1 and 2, both struck and done. **Call 1 carries a "
             "correction worth keeping**: the 25 s figure came from the spike harness's "
             "*scalar* driver and production takes the parallel path -- the real ritual is "
             "13.79 s (corrections #12; `S10-results.md` now carries the banner). **Calls 3, "
             "4 and 5 stay live** on the board."),

    dict(kind="range", sect="Sequenced", cls="MECHANICAL",
         start="*(**Collapse-tier organic materials + the production flip: SHIPPED**",
         end="measurement-backed:)*",
         why="Self-declared SHIPPED 2026-07-20 (journal/0026)."),

    dict(kind="range", sect="Sequenced", cls="POINTER",
         start="**Charcoal as an inclusion, not a band** (journal/0026, measured)",
         end="invariant work.",
         why="**Both of its stated blockers are false at source, verified " + DATE + ".** It "
             "says *\"no charcoal material was shipped\"* and that `deep_class` routes a "
             "charcoal-tagged unit to its mineral host. But "
             "`crates/dc-core/src/materials/mod.rs:133` defines `CHARCOAL`, "
             "`crates/dc-worldgen/src/geology.rs:430` routes "
             "`Biofacies::Charcoal => CLASS_ORGANIC_CHARCOAL`, and "
             "`crates/dc-worldgen/src/fill.rs:419` makes it loose-formed. Shipped by "
             "journal/0063. **\u00a7 Observed already carries this same verification** (S6 "
             "finding F3) on the entry that stays live for its wider point about a conclusion "
             "outliving its premise."),

    dict(kind="range", sect="Sequenced", cls="POINTER",
         start="**Water-model design pass** (ratified 2026-07-19, user; field-notebook",
         end="before any code.",
         why="Superseded by `docs/design/flow.md` (RATIFIED 2026-07-25) and part-delivered by "
             "`dc:field/head` (journal/0098). **Its surviving half is stated in full in the "
             "entry that stood directly above it, which stays live**: *\"what this pass still "
             "owes is the PRESENT/RUNTIME tier -- visible and flowing water, ponds and "
             "sub-resolution water, speleogenesis, and the free-water body-graph coupling. "
             "Caves ride FLOW continuation (c).\"* Two entries, one thread, the newer one "
             "already complete."),

    # ============================================================ Observed
    dict(kind="move", sect="Observed", cls="MECHANICAL",
         anchor="Two declaration defects on the new `dc:deep/weather_inventory` pass",
         why="`\u2705 BOTH FIXED -- verified at source " + DATE + "` (S6 finding F2). (a) "
             "shipped as the honest fix -- `BioMod` moved into the within-epoch `reads` "
             "roster; (b) fixed by **deletion**, which is the honest disposal -- `Exposed` is "
             "no longer declared, and `runner.rs:1454-1456` asserts it stays undeclared."),

    dict(kind="move", sect="Observed", cls="POINTER",
         anchor="`world_get_contents` reports `dc:air` and `has_contents: true` over",
         why="**Its single `OWED` shipped**: *\"a tier flag that can say UNRECORDED as a "
             "first-class answer\"* is `Identity::Unrecorded` (journal/0101), and CLAUDE.md "
             "\u00a7 Agent walks now teaches the fixed behaviour -- *\"`has_contents` is now a "
             "PER-VOXEL fact and is trustworthy (fixed 2026-07-25, journal/0101; it used to "
             "be answered per-CHUNK -- corrections #49)\"*. **The diagnosis is preserved "
             "whole** because its mechanism -- a per-voxel question answered with a per-chunk "
             "presence test -- is the *\"a summary is not an authority\"* shape caught in the "
             "query surface, and because its sibling *bare-cell fallback* entry is a "
             "different, genuinely-open defect."),

    dict(kind="move", sect="Observed", cls="MECHANICAL",
         anchor="HOLES IN THE GROUND: FIXED** 2026-07-21, journal/0057",
         why="`FIXED` in its own body (occupancy-aware culling, seven tests by name) and its "
             "`UNWALKED` tag discharged in place: `\u2705 WALKED AND CONFIRMED` (S6 finding F1, "
             "applied " + DATE + ")."),

    dict(kind="move", sect="Observed", cls="MECHANICAL",
         anchor="HOLES IN THE GROUND \u2014 partial voxels are missing side faces",
         why="Its own header ends `(FIXED -- see above)`, and the *above* is archived with it. "
             "**Its wider lesson is why it is preserved rather than deleted**: journal/0010 "
             "shipped partial-height rendering dormant and predicted it would *\"light up for "
             "free\"*; it lit up and did not work, because the assumption it rested on lived "
             "in a doc comment nobody re-read when the world changed underneath it."),

    dict(kind="move", sect="Observed", cls="MECHANICAL",
         anchor="\"Surface material is quantized per chunk\": FIXED** 2026-07-21,",
         why="`FIXED` (journal/0058; chunk footprints expressing more than one surface member "
             "went 0/169 \u2192 147/169) and its `Unwalked` tag discharged in place: "
             "`\u2705 WALKED AND CONFIRMED` (S6 finding F1)."),

    dict(kind="move", sect="Observed", cls="POINTER",
         anchor="GPU DeviceLost crash under a teleport storm at `--horizon 6`",
         why="**Refuted by the wide-horizons measurement archived above** (journal/0065): "
             "re-measured at horizon 6 in both regimes, alternated 6/3/6/3 against machine "
             "drift, plus one unbroken 700-jump / 23-minute horizon-6 session -- *\"All runs "
             "exited 0, no `DeviceLost`, no panic, no `ERROR`.\"* The root cause was host-RAM "
             "exhaustion from an unbounded chunk store, fixed by journal/0051. **Its secondary "
             "defect also shipped**: *\"a DeviceLost should not cascade into unwrap panics\"* "
             "\u2192 journal/0054's honest exit codes, which retired the *\"exit codes lie about "
             "GPU crashes\"* warning in CLAUDE.md."),

    dict(kind="move", sect="Observed", cls="POINTER",
         anchor="Stub inventory filed** (`docs/design/stubs.md`, 2026-07-21, read-only audit",
         why="Its *\"one genuine discovery\"* is already `\u2705 VOID` in place -- the owed "
             "comment was on `collapse.rs::ruin_posts`, deleted 2026-07-28. Its residual -- "
             "*\"the `field.rs` doc-comment claims the collapse tier reads exhum/t_crust when "
             "nothing does\"* -- is owned by the live \u00a7 Sequenced entry `METAMORPHISM -- the "
             "grade axis`, which names the same two planes and retires stubs #4. "
             "**`docs/design/stubs.md` itself is the live inventory and is untouched by this "
             "move.**"),

    dict(kind="move", sect="Observed", cls="POINTER",
         anchor="`--fullbright` is blind to geometry, and that cost a walk its conclusion",
         why="**Both of its filed proposals shipped as journal/0031** -- crease/silhouette "
             "edges under a separate `--edges` flag (`crates/dc-client/src/edgepass.rs`, "
             "`shaders/edges.wgsl`), and fullbright no longer applying distance fog. Its (b) "
             "-- sun determinism -- was confirmed in the entry itself. **The doctrine it "
             "earned is read-first material in CLAUDE.md \u00a7 Agent walks** (*\"Pick the "
             "control that can SEE your question\"*), which is the durable form of this "
             "finding."),

    dict(kind="move", sect="Observed", cls="POINTER",
         anchor="INSTRUMENT: `--fullbright` is BLIND TO SHAPE",
         why="Same finding, filed twice on 2026-07-20. Its *\"proposed fix, filed not built\"* "
             "is `--edges`, shipped journal/0031, and the lit-for-shape / fullbright-for-"
             "material rule is in CLAUDE.md."),

    dict(kind="move", sect="Observed", cls="POINTER",
         anchor="INSTRUMENT: `--fullbright` does not disable distance fog",
         why="Fixed by journal/0031; CLAUDE.md \u00a7 Agent walks states it -- *\"Fullbright also "
             "no longer applies distance fog (0031), so long-vista silhouettes are "
             "readable.\"*"),

    dict(kind="move", sect="Observed", cls="POINTER",
         anchor="No pooling/reuse of chunk or far-tile GPU resources",
         why="**Answered, and the answer is *deliberately not built*.** The live \u00a7 Observed "
             "entry `Mesh-buffer pooling: measured, deliberately NOT built (2026-07-21, "
             "journal/0051 -- the user asked for pooling; this is the numbered answer)` is the "
             "reply to this exact user question, and it is decisive on the mechanism: Bevy's "
             "`Mesh::insert_attribute` takes ownership, so pooled scratch buffers would have "
             "to be copied in -- zero copies per attribute becomes one. **That entry stays "
             "live**, including its surviving residual idea (reuse `MeshData`'s buffers "
             "*inside* `mesh_chunk`). The two entries never referenced each other."),

    dict(kind="move", sect="Observed", cls="POINTER",
         anchor="Walk 7 loose ends (journal/0008): **unloaded-neighbour and far-mesh",
         why="Its subject -- the S1 phantom old world visible below the real terrain -- is "
             "gone under the worldgen authority (journal/0022), as the far-mesh entry says in "
             "its own text: *\"the phantom old world ~1 km down is gone and there is a "
             "horizon.\"* **That entry stays live** for its unrelated open half, the "
             "`TerrainGen` seal (`\u2705 VERIFIED STILL OPEN " + DATE + "`)."),

    dict(kind="move", sect="Observed", cls="MECHANICAL",
         anchor="The embedded HostWorld never evicts chunks",
         why="Struck in place `\u2705 IT EVICTS -- verified at source " + DATE + "` "
             "(`crates/dc-api/src/host.rs:524`, a bounded LRU with hysteresis in which edited "
             "entries are protected). It was kept live that morning **only** so that archiving "
             "its refutation would not leave the claim standing alone; both are now in this "
             "file together, which is the correct end state."),

    dict(kind="move", sect="Observed", cls="MECHANICAL",
         anchor="Site cap 240 (u8 RegionId)",
         why="Struck in place `\u2705 THE SUBJECT IS GONE -- verified " + DATE + "`. A scale "
             "constraint on content removed 2026-07-28 (journal/0121). **S2's ledger-scale "
             "question itself is a different, live entry** -- the \U0001f514 TRIGGERED "
             "S2-checkpoint-facts item, which stays."),

    dict(kind="move", sect="Observed", cls="MECHANICAL",
         anchor="We cannot see where runtime goes \u2014 the perf observability gap",
         why="Struck in place `\u2705 ANSWERED AS POSED -- verified at source " + DATE + "` "
             "(`crates/dc-client/src/perf.rs:26,40,68-69`). **Its live residual is its own "
             "entry and stays**: *\"the perf instrument can't show the frame-thread "
             "envelope\"* -- per-thread-role attribution is what is actually missing. The two "
             "sat ~1,000 lines apart never referencing each other, and **a complaint about an "
             "instrument is not evidence the instrument is absent.**"),

    dict(kind="move", sect="Observed", cls="POINTER",
         anchor="Chunk gen time is now noticeable in vertical streaming",
         why="The suspect it sharpened to -- `LOAD_BUDGET_PER_FRAME = 8`, generated "
             "synchronously on the main schedule -- was addressed by the 0083/0084 offload. "
             "**Its outcome is the live \u00a7 Observed entry `Perf: throughput ceiling at "
             "terminal velocity`**: *\"the drop reaches the choking point later ... but at "
             "terminal velocity it still chokes, about as hard.\"* Two entries, one thread, no "
             "cross-reference. The ceiling, not the onset, is the live question."),

    dict(kind="move", sect="Observed", cls="MECHANICAL",
         anchor="The `history.rs` reject-don't-crash skip is SILENT",
         why="Struck in place `\u2705 THE FILE IS DELETED -- verified " + DATE + "`. **The "
             "pack-degradation doctrine it invoked is explicitly untouched and still binds** "
             "(API.md: degradation must be LOUD) -- only this instance of it is void."),
]

# In-place repairs applied to ROADMAP.md AFTER removal. Each `old` must occur exactly once.
FIXUPS = [
    # the hillslope-operator-fixed entry points "below" at the conveyor entry, three times
    ("The blocker\n  below is discharged; it is kept live only because",
     "The blocker\n  \u2014 **archived " + DATE + " to [`ROADMAP-history.md`](ROADMAP-history.md)** \u2014 "
     "is discharged; it is kept live only because"),
    ("the (b) block below is WRONG and #63 (ii) with it.",
     "the (b) block of the archived conveyor entry (`ROADMAP-history.md`) is WRONG and "
     "#63 (ii) with it.",),
    ("limit\" from below without reading #72.**",
     "limit\" from that archived entry without reading #72.**"),
    # the deep-time clock entry points at both archived stubs-#27/#29 entries
    ("blocked on stubs #29 above.",
     "blocked on stubs #29 \u2014 **discharged " + DATE + " by journal/0122; that entry is now in "
     "`ROADMAP-history.md`**.",),
    # the five-rulings head, after its body moved
    ("below; the context is kept because the reasoning is what makes each ruling legible.**",
     "below; the context is kept because the reasoning is what makes each ruling legible.**\n"
     "  **\u2b07 The five rulings and their reasoning moved to "
     "[`ROADMAP-history.md`](ROADMAP-history.md) on " + DATE + " \u2014 nothing there is owed. "
     "What stays live here is the *not user-owned* tail below.**"),
    # the baseline-sweep head, after its applied half moved
    ("  UNAPPLIED** (2026-07-28, `docs/audits/baseline-2026-07-28/`",
     "  UNAPPLIED**, *and now largely APPLIED \u2014 the four user calls and all five "
     "highest-blast-radius\n  items were ruled or applied and moved to "
     "[`ROADMAP-history.md`](ROADMAP-history.md) on " + DATE + "* (2026-07-28, "
     "`docs/audits/baseline-2026-07-28/`"),
]

# ---------------------------------------------------------------- machinery

lines = io.open(RM, encoding="utf-8").read().split("\n")


def section_bounds(name):
    opener, closer = SECTIONS[name]
    s = next(i for i, l in enumerate(lines) if l.startswith(opener))
    e = next(i for i, l in enumerate(lines) if i > s and l.startswith(closer))
    return s, e


def bullet_blocks(name):
    """[(start, end_exclusive)] for every top-level bullet in a section, blanks trimmed."""
    s, e = section_bounds(name)
    starts = [i for i in range(s, e) if lines[i].startswith("- ")]
    out = []
    for k, st in enumerate(starts):
        en = starts[k + 1] if k + 1 < len(starts) else e
        while en > st and lines[en - 1].strip() in ("", "---"):
            en -= 1
        out.append((st, en))
    return out


def find_block(name, anchor):
    hits = [(s, e) for (s, e) in bullet_blocks(name) if anchor in "\n".join(lines[s:e])]
    if len(hits) != 1:
        raise SystemExit("BLOCK ANCHOR %r in %s -> %d hits %s" % (anchor, name, len(hits), hits))
    return hits[0]


def find_line(lo, hi, anchor, what):
    hits = [i for i in range(lo, hi) if anchor in lines[i]]
    if len(hits) != 1:
        raise SystemExit("%s ANCHOR %r -> %d hits (lines %s)"
                         % (what, anchor, len(hits), [h + 1 for h in hits]))
    return hits[0]


def resolve(op):
    """-> (start, end_exclusive)"""
    if op["kind"] == "move":
        return find_block(op["sect"], op["anchor"])
    if "block" in op:
        lo, hi = find_block(op["sect"], op["block"])
    else:
        lo, hi = section_bounds(op["sect"])
    s = find_line(lo, hi, op["start"], "START")
    e = find_line(s, hi, op["end"], "END")
    # `extra` extends the range past the end anchor, for the case where the true last
    # line of a block is not itself uniquely addressable.
    return s, e + 1 + op.get("extra", 0)


resolved = []
for op in OPS:
    s, e = resolve(op)
    resolved.append((s, e, op))

resolved.sort(key=lambda t: t[0])

# no overlaps
for i in range(1, len(resolved)):
    if resolved[i][0] < resolved[i - 1][1]:
        raise SystemExit("OVERLAP: %s and %s"
                         % (resolved[i - 1][2].get("anchor", resolved[i - 1][2].get("start")),
                            resolved[i][2].get("anchor", resolved[i][2].get("start"))))

# every fixup target must be unique in the *surviving* text
survivor_idx = set(range(len(lines)))
for s, e, _ in resolved:
    survivor_idx -= set(range(s, e))
survivor_text = "\n".join(lines[i] for i in sorted(survivor_idx))
for old, new in FIXUPS:
    n = survivor_text.count(old)
    if n != 1:
        raise SystemExit("FIXUP %r -> %d hits in surviving text" % (old[:60], n))

total = sum(e - s for s, e, _ in resolved)
by_cls = {}
for s, e, op in resolved:
    by_cls.setdefault(op["cls"], []).append(e - s)

print("ROADMAP.md is %d lines" % len(lines))
for cls in ("MECHANICAL", "POINTER", "SHRINK"):
    v = by_cls.get(cls, [])
    print("  %-11s %2d ops, %4d lines" % (cls, len(v), sum(v)))
print("  %-11s %2d ops, %4d lines" % ("TOTAL", len(resolved), total))
print("  %d fixups, all unique" % len(FIXUPS))
print("  projected board after removal: ~%d lines" % (len(lines) - total))

if "--check" in sys.argv:
    for s, e, op in resolved:
        print("  %5d-%-5d %-11s %s" % (s + 1, e, op["cls"],
                                       (op.get("anchor") or op.get("start"))[:72]))
    print("\n--check: nothing written.")
    raise SystemExit(0)

# ---------------------------------------------------------------- build history text

CLS_LEAD = {
    "MECHANICAL": "**ARCHIVED " + DATE + " \u2014 the entry's own body already declared it "
                  "closed** (moved by status, not by age; reproduced verbatim).",
    "POINTER": "**ARCHIVED " + DATE + " \u2014 resolved by later work this entry did not know "
               "about** (moved by status, not by age; reproduced verbatim).",
    "SHRINK": "**ARCHIVED " + DATE + " \u2014 a closed sub-thread of an entry that is STILL "
              "LIVE on the board.** Only this passage moved; its parent entry stays in "
              "`ROADMAP.md` (reproduced verbatim).",
}

hist = []
hist.append("## In flight \u00b7 Sequenced \u00b7 Observed \u2014 archived " + DATE + " (pass 2)")
hist.append("")
hist.append("**Archived from `ROADMAP.md` on " + DATE + ", by STATUS, not by age** \u2014 the same rule")
hist.append("as the sections above. This is the **second** pass of that day: the first moved 45")
hist.append("\u00a7 Observed entries and four superseded close blocks; this one reads \u00a7 In flight and")
hist.append("\u00a7 Sequenced, **which had never been swept for archivability at all.**")
hist.append("")
hist.append("Sourced from the complete classification in")
hist.append("[`" + AUDIT + "`](" + AUDIT + "), with every")
hist.append("finding re-verified at source before the move. *An audit finding is a hypothesis, not")
hist.append("an authority.*")
hist.append("")
hist.append("**Entries are reproduced VERBATIM.** Recorded poses, world coordinates, measured")
hist.append("numbers and asset filenames travel with them; nothing was summarised away and nothing")
hist.append("was deleted. **Where an archived entry carries a measurement, that measurement is a")
hist.append("dated record** \u2014 it was not re-measured by this pass and must not be propagated")
hist.append("without checking (read-first item 5: immutable body, mutable header).")
hist.append("")
hist.append("**Nothing user-owned moved.** Five strike-candidates \u2014 two 2026-07-21 appearance")
hist.append("ratifications, two field reports resting on a premise the user struck this morning,")
hist.append("and the ores fork \u2014 stayed on the live board awaiting the user's word. They are")
hist.append("listed in \u00a7 5 of the classification audit.")
hist.append("")
hist.append("**Why this pass was worth its cost, stated as the finding rather than the tidy:**")
hist.append("\u00a7 Observed's measured failure was *closure-in-the-wrong-place* \u2014 a claim and its own")
hist.append("refutation coexisting in one artifact. \u00a7 Sequenced and \u00a7 In flight have the same")
hist.append("shape and it is worse, because the two halves sit in **different sections**: the")
hist.append("collapse-cache `evict()` claim and its refutation stood ~2,500 lines apart for eight")
hist.append("days; the erosion-supply calibration asked for a comparison another entry 1,300 lines")
hist.append("above it had already made; the GPU-pooling question and *\"this is the numbered")
hist.append("answer\"* never named each other. **The rule is unchanged and it is not about volume:**")
hist.append("discharge the old entry in the same commit as the new fact.")
hist.append("")

for cls in ("MECHANICAL", "POINTER", "SHRINK"):
    picked = [(s, e, op) for (s, e, op) in resolved if op["cls"] == cls]
    if not picked:
        continue
    title = {
        "MECHANICAL": "### Resolved in place \u2014 entries whose own bodies already said DONE",
        "POINTER": "### Resolved by later work the entry did not know about",
        "SHRINK": "### Closed sub-threads lifted out of entries that are STILL LIVE",
    }[cls]
    hist.append(title)
    hist.append("")
    if cls == "SHRINK":
        hist.append("**Every passage below was cut from an entry that remains on the live board.**")
        hist.append("The parent entry's head, its open questions and its reasoning stayed; what moved")
        hist.append("is a sub-thread the entry itself had already marked shipped, struck or")
        hist.append("superseded. Each banner names the parent so the cut is walkable from this end.")
        hist.append("")
    for s, e, op in picked:
        hist.append(CLS_LEAD[cls])
        hist.append("")
        if cls == "SHRINK":
            parent = op.get("block") or ("\u00a7 " + op["sect"] + " (un-bulleted prose)")
            hist.append("**Parent entry (still live in `ROADMAP.md`):** " + parent)
            hist.append("")
        hist.append(op["why"])
        hist.append("")
        hist.extend(lines[s:e])
        hist.append("")

# ---------------------------------------------------------------- splice into history

h = io.open(HI, encoding="utf-8").read().split("\n")
mark = next(i for i, l in enumerate(h) if l.startswith("# Superseded close blocks"))
h2 = h[:mark] + hist + h[mark:]
txt = "\n".join(h2)

old_toc = """3. **Superseded close blocks** \u2014 each was consumed by the one after it."""
new_toc = """3. **In flight \u00b7 Sequenced \u00b7 Observed \u2014 archived """ + DATE + """ (pass 2)** \u2014 the first
   sweep of \u00a7 In flight and \u00a7 Sequenced for archivability, plus the \u00a7 Observed entries the
   " + DATE + " morning pass annotated in place rather than moving.
4. **Superseded close blocks** \u2014 each was consumed by the one after it."""
new_toc = new_toc.replace('" + DATE + "', DATE)
if old_toc not in txt:
    raise SystemExit("history intro list not in the expected shape; fix by hand")
txt = txt.replace(old_toc, new_toc, 1)
io.open(HI, "w", encoding="utf-8", newline="\n").write(txt)

# ---------------------------------------------------------------- rewrite ROADMAP

drop = set()
for s, e, _ in resolved:
    drop |= set(range(s, e))
kept = [l for i, l in enumerate(lines) if i not in drop]

# collapse any run of 3+ blank lines left behind down to one
out = []
blanks = 0
for l in kept:
    if l.strip() == "":
        blanks += 1
        if blanks > 2:
            continue
    else:
        blanks = 0
    out.append(l)

body = "\n".join(out)
for old, new in FIXUPS:
    assert body.count(old) == 1, old[:60]
    body = body.replace(old, new, 1)

io.open(RM, "w", encoding="utf-8", newline="\n").write(body)
print("removed %d lines; ROADMAP.md is now %d lines" % (total, len(body.split("\n"))))
print("\nOBLIGATIONS \u2014 discharge in this same commit:")
print("  1. re-file the octree substrate's four follow-ons as a live \u00a7 Sequenced line")
print("  2. re-file (or deliberately drop) APPEARANCE WALKS OWED's poke-through residue")
print("  3. the five STRIKE-CANDIDATES are still on the board and still owed to the user")
