# The debt walkthrough — live working notes (2026-08-05)

> **⚠ THIS IS A LIVE WORKING DOC, WRITTEN DURING THE CONVERSATION, NOT A FINISHED AUDIT.**
> The user's direction: *"we're going to write down everything as we go, then at wrap you'll
> read it and check for consistency, then we'll make sure it's somewhere that makes sense for
> next session to pick it up."* **So: unreconciled, possibly internally inconsistent until the
> wrap pass, and NOT yet filed where it belongs.** Reconciliation + placement are owed at wrap.
>
> **Provenance is marked per item.** ✅ = user ruling · ⬦ = integrator analysis, unratified ·
> ⚠ = open/unknown · 📌 = verified at source this session.

## 0. The frame — two rulings that govern everything below

**✅ NEXT SESSION IS DEEPEST-FIRST, AND THERE IS NO MID-ARC (user, 2026-08-05).**
> *"In next session we do deepest first. **There is no mid-arc.** Time and time again we
> entrench and forget. We have to accept it is **debt-cleanup time** without losing track of
> everything it's meant to buy us downstream — items currently open or planned."*

Both halves bind: do the foundational work first, **and** keep the downstream payoff visible so
the cleanup does not become its own end. *That second clause is why this document exists.*

**✅ THE CORRELATION KERNEL IS EXTRACTED TO `dc-core` (user, 2026-08-05).** Same ruling as E4-1's
venue (U-3, *"the engine owns primitives… we're not going to entrench a primitive in worldgen,
whose heir is the pack"*).
- 📌 **The subtraction test passes at source:** `deeptime/correlate.rs` contains **no
  `MaterialId` and no `GeologySet`**. Its vocabulary is `Bed`, `Borehole`, `EpochPartition`,
  `ColumnInterval`, `bilinear_weights` — epochs, thicknesses, weights. Subtract the whole pack
  and the module is untouched.
- ⬦ **Do it before S2 lands on it**, since every slice built on top enlarges the extraction.

**✅ AND A PREMISE CORRECTION THAT REACHES MOST COST ARGUMENTS (user, 2026-08-05):**
> *"There is **no world data to migrate period** because there are no saved worlds. Nothing
> persists yet. The goldens are the only thing materially different for us, and they matter only
> for inspecting our outputs, calibrating instruments, measuring results of changes… **they will
> be unrecognizable in the final pack.** The world is never replayed currently: it is
> regenerated new."*

**So "we'd have to migrate the record" is not an argument that exists right now.** Schema changes
to `DepUnit`, the ledger, the flux record cost a **golden re-capture** (scratch instruments) and
whatever packed-layout arithmetic they force — nothing else. *The integrator built a
three-way cost ranking on the opposite premise and it collapsed; recorded so it is not rebuilt.*

## 1. The inventory — everything surfaced or carried, so nothing drops

| # | item | state |
|---|---|---|
| 1 | **The closed enums** — `DeepAxis`, `FlowCause`, `Cause` | walked, § 2 below |
| 2 | **Correlation extraction → `dc-core`** | ✅ RULED |
| 3 | **The composition discard** — argmax at deposition (D-9) + bulk face archive (D-7) | next |
| 4 | **Structural attitude** — dip/strike/fold/fault; stored vs re-derived fork | queued |
| 5 | **Weathering altitude** — basement-only → general surface process | queued |
| 6 | **The three weathering authorities** — reconciliation unowned | queued |
| 7 | **Top-of-stack placement** — ratified or unexamined? | queued |
| 8 | **Move C removal** — replacement + when the world goes smooth | queued |
| 9 | **The voxelization contract** — shares→eighths; the slots question | queued |
| 10 | **The refinement firewall** — write path unverified for the planes | queued |
| 11 | **Two never-taken measurements** — mixture count; identity-loss fraction | queued |
| 12 | **Carried from GEO-4** — E4-2's fork; P2's M re-pick | queued |
| 13 | **The materials/term-space pass** — regions + metric | queued |
| 14 | **Housekeeping** — ROADMAP at 2.2×, `spines.md` over threshold | queued |

## 2. ITEM 1 — the closed enums ✅ WALKED

### 2.1 The principle is RATIFIED, not new — and this session nearly re-derived it

📌 `ARCHITECTURE.md` § *What this indicts today* (**DECIDED 2026-07-26**, corrections #65):
> *"A highly customizable voxel-based crafting-game **generator** — plugin based, each world able
> to be its own unique sim composed from declared plugins built on engine primitives, whose
> **first plugin pack** is the earth-like world generator we are building. **The engine is not
> 'our simulation with a mod API bolted on.' The default pack is one pack.**"*
>
> *"`DeepAxis` is a **closed enum inside `dc-worldgen` whose variants are the names of our own
> pipeline stages**… The engine knows the default pack's pass roster by name. **That is the
> violation**, and it is not incidental: it is the *consequence* of choosing to **derive** pass
> order from declarations rather than **accept** it as data."*

The user's own statement of it, 2026-08-05, is that doctrine restated:
> *"The problem with hard coded enums: **they are pack opinions living in the engine.** The
> engine **has no concept of earth science processes**. Alongside the engine, we are building an
> earth sciences sim pack. **We must not foreclose or inject weird opinions into other future
> mods / plugin packs.**"*

### 2.2 📌 THE ENUMERATION IS INCOMPLETE — the finding of this item

**The ratified indictment names `DeepAxis` and ONLY `DeepAxis`.** Two more exist and appear in
no indictment anywhere:

| enum | names | where | width |
|---|---|---|---|
| `DeepAxis` (`runner.rs`) | our own pipeline stages | compile-time | 18 variants |
| `FlowCause` (`flux.rs:337-352`) | transport regimes — Fluvial, Eolian, Glacial, Gravity, Marine, Hydrothermal, Dissolution | **in every `DepUnit`, in the merge key** | 3 bits, **+`MOVER_NONE` = full** |
| `Cause` (`inventory.rs:395-407`) | weathering agents — Chemical, Biotic, Frost, Dissolution | the fact ledger's `Fact` | 4 variants |

📌 **`Dissolution` appears in BOTH `FlowCause` and `Cause`, owned by neither.**

⬦ *The corpus predicted this class: its knowledge notebook records **"no enumeration is ever
checked for completeness"** as a standing gap. This is a live instance of it, inside the ratified
statement of the principle.*

### 2.3 ✅ The sharpened rule (user framing, integrator wording)

> **The engine's vocabulary must be STRUCTURAL, never NOMINAL.** *There is a mover. There is a
> cause. There is an axis.* What they are **called** is pack-declared.

⬦ **The test — "would two good packs disagree" made vivid: WOULD A NON-EARTH PACK INHERIT THIS?**
A moon with no liquid water, a gas giant, a fantasy world — do they get `Fluvial` and `Frost`
compiled into the engine they build on? `Fluvial`, `Eolian`, `Glacial`, `Frost`, `Chemical` are
**Earth concepts**. It cuts all three enums identically, and it catches `Dissolution`-in-both as
Earth chemistry modelled twice inside the engine.

### 2.4 ⬦ The replacement shape is ALREADY IN USE — no new machinery

An open vocabulary cannot live in a 3-bit enum — and does not need to. **It becomes a compact id
into a pack-declared registry, exactly as `MaterialId` already works.** The "the enum is full"
problem was never a storage problem; it was full **because it was an enum**.

### 2.5 Dispositions

- ⬦ **All three are ONE item**, not three ranked by cost. The principle is ratified; **the
  enumeration should be completed** — an amendment to `ARCHITECTURE.md`'s indictment. No code.
- ⬦ **`DeepAxis` stays entangled with E7.** Authored order removes its *reason to exist*; doing
  E6 alone fixes the symptom and leaves the mechanism that regenerates it.
- ⬦ **`Cause` and `FlowCause` are entangled with nothing** and are the cheap half. The fact
  ledger is additionally **empty in every shipped world** (`weather_inventory` default off).
- 📌 **CORRECTION MADE HERE: `FlowCause` does NOT block the ice agent.** `Glacial = 2` already
  exists; ice needs a **producer**, not a slot. `FlowCause` blocks *third-party packs adding
  movers we never enumerated* — a **plugin-agnosticism** blocker, not a next-feature one.
- ⚠ **OPEN, cheap to decide, and it changes item 3's storage: are MOVER and CAUSE one
  vocabulary?** An agent that *moves* material and an agent that *transforms* it in place may be
  one declarable thing seen twice — `Dissolution` sits in both because it genuinely does both.

**What next session should produce on this item:** an amendment completing the indictment's
enumeration, plus one design call on mover/cause unification. **Neither needs code.**

---

*(Items 3–14 append below as they are walked.)*

## 3. ITEM 3 — the composition discard (D-9 / D-7) ⚠ WALKED, ONE QUESTION OPEN

### 3.1 📌 IT WAS DESIGNED, COSTED AND RULED — this is not an unnoticed oversight

The integrator framed D-9 as a defect nobody had reasoned about. **The design pass reasoned
about it explicitly and the user ruled on it.**

📌 `docs/audits/2026-08-01-members-into-history-design.md` § 3.1 (Option A, the one that
shipped) states the consequence in as many words:
> *"a unit is still **one identity**. **A bed that is 60 % sandstone / 40 % siltstone records
> as sandstone.** The composition the record-terms slice wants (ruling 1) is a **face**
> quantity, not a unit quantity — see Option B."*

📌 **Option B — a share vector per unit — was priced and rejected on arithmetic:** +42.23 MiB at
7 shares, +84.47 MiB at 14, **+168.94 MiB at 26**, against a whole-field residency of
**108.55 MiB**. The doc's own verdict: *"the option the arithmetic is hostile to."*

📌 **And ruling 3 (user, 2026-08-02) chose FACE:** composition rides a **sparse CSR sidecar on
the flux record**; **`DepUnit` stays single-species**. Stated rationale: *directional provenance
for the refinement operators.*

⬦ **So D-9's real contribution is not "nobody noticed." It is "the reasons may now be stale."**

### 3.2 ⬦ Two things have changed since that ruling

**(a) The costing was DENSE, and sparse later beat dense.** Option B was priced as a dense share
vector per unit. **P11 slice 2 then shipped CSR planes at 0.70× the dense they replaced**, and
slice 3 shipped the packed 8 B unit. A *sparse* per-unit distribution — most units really are
one species — was never costed. **The arithmetic that killed Option B may no longer hold, and
re-costing it is cheap.**

**(b) The refinement ruling (2026-08-05, § 0) makes the unit's single identity a CEILING ON
APPEARANCE, not just on provenance.** If refinement may express only what deeptime persisted,
then a bed recorded as one species **can never be expressed as a mixed bed** — the refiner has
nothing to read. Under the pre-2026-08-05 framing that was a provenance limitation; now it is a
limit on what the world can look like.

### 3.3 ⚠ THE OPEN QUESTION — put to the user, not resolved here

**Does ruling 3's FACE sidecar actually give refinement what the honesty ruling now requires?**

⬦ The integrator's doubt, stated as doubt: a face records **what crossed a boundary**; a bed is
**what settled in a cell**. Face composition plus the settling arithmetic *might* let a refiner
reconstruct a deposited mixture, or might only ever answer *which direction material came from*
— which is what ruling 3's own rationale says it is for. **If the second, then FACE and UNIT are
answering different questions and choosing one did not dispose of the other.**

**Not resolvable from the desk.** What would settle it: trace whether the face record plus `dh`
is sufficient to reconstruct the arriving mixture at a cell, or whether the argmax destroys
information the faces never held.

### 3.4 Owed measurements (both still untaken)

- **The identity-loss fraction** — what share of recorded metres is relabelled by the argmax.
  Sum non-winning mass at each deposit against total recorded metres. Cheap, headless.
- **A re-cost of a SPARSE per-unit distribution**, against the post-slice-3 baseline.

⬦ *Both should precede any remedy. corrections **#57** is the worked case where the argmax
produced a wrong world and is the strongest existing argument on this item.*

### 3.5 ✅ USER RULING, 2026-08-05: **"any share of recorded metres getting relabelled is too much."**

Settles the argmax question **without a measurement**. The identity-loss fraction is demoted from
a *decision* input to a **sizing** input — still worth taking (it tells us how sparse a real
distribution is, hence what it costs), but not to decide whether.

### 3.6 📌 WHAT A FACE ENTRY ACTUALLY HOLDS (the user asked; verified at `flux.rs:372-421`)

16 B, grouped per cell (CSR), ordered by `(chapter, face)`:

| field | what it is |
|---|---|
| `magnitude` | flux across the face, **summed over the chapter** |
| `load` | suspended load — **metres of column thickness, BULK ONLY** |
| `fluid` | always `WATER` |
| `chapter` | **the tectonic chapter — NOT the epoch** |
| `face` | which face |
| `form` | free/bound — always `Free` |
| `cause` | the mover — **always `Fluvial`** (nothing else has a producer) |

### 3.7 📌 THREE GAPS against what the user says refinement wants

The user's statement of the requirement (2026-08-05):
> *"if face says this much x during this epoch entered this side, then refinement wants to know
> that when it draws a bed and some of the bed was deposited from alluvial on north face under
> whatever drainage field and elevation at that time."*

1. **Composition.** `load` is bulk. Known; ruling 3's sidecar is the ruled fix, **unbuilt**.
2. **Time resolution — 25× too coarse.** A face aggregates a whole **chapter** (~25 epochs); units
   carry the **exact epoch**. *"During this epoch"* is unanswerable. **This is D-6's asymmetry in
   a THIRD store — the corpus now keeps time at three different resolutions** (unit: epoch ·
   fact: chapter · face: chapter).
3. 🔴 **THE PALEO-FIELDS ARE NOT PERSISTED AT ALL — and this appears on no list anywhere.**
   Drainage is exported as **the last routing only** (`recv`/`area`/`lake` at the final sea
   stand); elevation likewise survives only as the final surface. So *"under whatever drainage
   field and elevation at that time"* **has no store to read**. A refiner cannot know the
   drainage configuration when a bed was laid — only the one at the end of time.
   ⬦ *Unlike the others this was never a discard; it was never kept. New finding, 2026-08-05.*

### 3.8 ⬦ CONSEQUENCE: items 1 and 3 CONVERGE into one record-schema pass

Opening the mover vocabulary (item 1) and stopping the identity discard (item 3) touch the **same
struct, the same packed layout, and the same merge key**. With no persistence to migrate (§ 0),
one pass is cheaper than either was priced at separately. **Do not schedule them apart.**

⬦ **And a distribution may make the MERGE simpler, not harder** — flagged, not designed. Units
merge today when keys match, which forces species *into* the key. Distributions **accumulate**:
mass-weighted addition, no identity conflict, no argmax, possibly no species in the key at all.
Whether the same holds for the mover (also a plurality verdict in the key) is **unknown**.

### 3.9 ⚠ Still open on this item

- **Does the FACE sidecar give refinement what the honesty ruling requires, or only directional
  provenance?** A face records what **crossed a boundary**; a bed is what **settled in a cell**.
  If they answer different questions, ruling 3 remains correct and simply does not cover the bed.
  **Bounded source question, not a design pass.**
- **What the face record is INTENDED to become** after the slated work subsumes it — the user
  noted they do not know, and neither does this document. Read `flow.md`'s continuations and the
  fluvial record-terms slice before designing anything that reads faces.
- **Whether paleo-drainage / paleo-elevation should be persisted, derived, or neither** (§ 3.7.3).
  Untouched by any existing plan.
