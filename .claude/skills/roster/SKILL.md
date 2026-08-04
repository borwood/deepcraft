---
name: roster
description: Add, modify, or retire a material in the DEEPSIM DEFAULT PACK's roster (the GeologySet members and their materials) — the term-space philosophy, the definition a member owes, the derived-relations rule, and the proxy discipline. Use whenever anyone proposes a new rock/sediment/organic material, questions whether an existing member should exist, or reaches for a hand-authored material relationship. NOT for engine material machinery (MaterialId type, mixtures, packing, contents) — that is dc-core primitive work; this skill is pack content only.
---

# roster — adding to the sim pack's material roster

**Scope, first, because the user drew it by name (2026-08-04):** this skill governs the
**sim pack's roster** — the `GeologySet` members (`crates/dc-core/src/materials/geology.rs
::vanilla_members`), the materials they deposit, and their term definitions. It does
**not** govern engine material machinery (the `MaterialId` type, the mixture/eighths
model, packing, contents) — deeply related, different owner. If your change is to the
*capability* rather than the *content*, you are in dc-core primitive territory and this
skill only tells you to say so loudly.

## 0. SWEEP BEFORE USE — this skill is self-updating, by design

The user's directive at creation: *"sweep for new sketches and rulings on this domain…
update this skill if any new information bears on it, check with user."* So, before
relying on anything below:

1. Grep the corpus for rulings newer than this file's last update (below) on:
   `materials.md` DECIDED entries · `ideas.md` (term space, axis, roster, mineral,
   continuum) · ROADMAP § Sequenced/Observed (P10 grain, P11 members, FS-A) · the
   newest journals and design audits touching materials.
2. If anything bears on this skill, **update this file in the same session and check
   with the user** — a stale procedure doc is worse than none (the corpus's
   one-directional-pointer lesson).
3. Record the sweep watermark here when you update: **last swept/updated 2026-08-04**
   (creation; the ruling it carries was ratified that day).

## 1. The philosophy (RATIFIED 2026-08-04, user — `materials.md` § DECIDED 2026-08-04)

**A material name is a pack-authored label over a REGION of a continuous term space.**
The record stores the name; expression may move within the region and blend between
regions; the space between names is MIXTURE, not nameless matter. In the user's words:
*"materials are a label for a point in a smooth field… we have axes that materials can
be related on, and the pack can derive relationships from these axes, and with these
opinions it will drive the refinement/presentational rules for how these materials can
share space together."*

Why this is the honest shape (the reasoning, compressed — full form in the materials.md
entry and the 2026-08-04 geo-session conversation):

- **Petrology says so.** Clastic rock names are *texture labels on a conserved mineral
  stock* (Wentworth grain-size fences); mechanical weathering re-sorts that stock
  without changing it — sandstone → sand is one substance changing state. Igneous names
  are *regions* on continuous composition diagrams (QAPF), and granite-vs-rhyolite is a
  texture axis, not a composition one. Only chemical edges (feldspar → clay, calcite
  dissolving) genuinely change identity. It is continua most of the way down, with
  useful names painted on.
- **The loose half already made this move.** U1 (2026-08-02): grain is an AXIS on the
  loose form, not a species per size. The lithified clastics are the same symmetry —
  mudstone/siltstone/sandstone are the lithified φ ladder. FS-A's literature-cited
  release spectra already encode it (each rock's products are the same minerals at
  different φ).
- **S-3 applied to petrology.** A rock name is a SUMMARY of composition + state. The
  summary must derive from the authority (the terms), never stand beside it as a
  parallel truth.
- **The roster is bring-up-era and owes nobody its current shape.** The user: *"i'm not
  married to a single material in the roster yet."* Siltstone and conglomerate carry
  literal `(roster proof)` comments; coal is a proxy for life that does not exist yet.
  Existence is not standing.

**Engine/pack line:** the *capability* is engine (term schemas as declarable data,
per-term blend/conservation, the mixture channel, term-keyed refinement kernels); the
*opinion* is pack (which axes exist, every label, every derived relation, this
philosophy). When unsure which side a change is on, run the subtraction test: subtract
the pack — whatever remains is engine, and it must be featureless and conservation-true.

## 2. The procedure — adding or modifying a member

1. **Write the term definition FIRST.** Which declared axes place this material, and
   what region does it occupy? Day-one term sources: the FS-A release spectrum
   (`release_vanilla.rs` — literature-cited, add one for a new member), the property
   sheet (grain size, density, hardness/erodibility), organic/carbonate fraction if its
   axis exists. **A member you cannot honestly define in terms is a PROXY** — it may
   still ship (proxies are legal bring-up), but it is *marked* as one, with its heir
   named (the coal precedent: heir = life/organics, `materials.md` § transformation
   axes heir chain).
2. **Derive, never hand-pair.** Any relationship this member has to others (continuum
   membership, weathering products, settle ordering) must be computed from its terms —
   the `settle_energy` precedent: *"from properties instead of a hand table."* A hand
   list of related materials is a fitted taxonomy and gets rejected at review.
3. **The class is a SELECTION contract, nothing more.** Formation window, abundance,
   habit — where and how often this member is deposited. Class co-membership implies
   NOTHING about material relations (the A-CLEAN bar: no class view survives storage or
   physics). Do not encode a relationship by class placement.
4. **Cite the literature.** Formation windows, spectra, hardness — a number with a
   published source is evidence; one chosen until it looks right is not (CLAUDE.md § a
   closed system cannot detect its own scale error). FS-A's eight cited tables are the
   bar.
5. **Check the mechanical invariants** (they are tested; run the dc-core suite):
   canonical ordering means registration order never changes worlds; abundance
   normalizes within class (adding a member diversifies, never inflates);
   `vanilla_members_have_distinct_materials` pins the material→member inversion.
6. **Retiring a member:** confirm nothing in the record names its `MaterialId` on the
   worlds we keep (the record stores materials now — P11), or plan the golden
   re-capture; check FS-A tables and any term data that references it; a proxy being
   replaced by its heir (coal → real organics) is the intended lifecycle, not a
   deprecation.
7. **Say what changed where it is asked.** materials.md if the definition philosophy
   moved; ROADMAP if it unblocks/blocks anything; the release tables' own comments; and
   this skill (§ 0) if the ruling landscape moved.

## 3. Pointers

- `docs/design/materials.md` § DECIDED 2026-08-04 (the ruling), § DECIDED 2026-07-22
  (block IS material; transformation axes), § the U1 revisit note.
- `crates/dc-core/src/materials/geology.rs` (the set + `settle_energy`),
  `release_vanilla.rs` (FS-A tables), the props sheet.
- `docs/audits/2026-08-03-stratigraphic-correlation-design.md` § 3 + header (P-1:
  M-C with the derived predicate — the first consumer of the derived-relations rule).
- `docs/design/material-genesis-notebook.md` § 2 (term-keyed edges),
  `docs/design/refinement.md` § 4 (operators read terms, never names).
- The 2026-08-04 geo-session conversation (journal/0152) — the borehole story, the
  petrology walkthrough, and the ratification exchange this skill compresses.
