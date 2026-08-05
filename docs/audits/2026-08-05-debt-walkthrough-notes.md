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
