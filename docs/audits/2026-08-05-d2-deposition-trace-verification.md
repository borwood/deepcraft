# D2 deposition trace — independent source verification

Base commit: `dd575b652deb828f9e724ff0381190fbb5658406` (merge with `main`: already
up to date; `git log -1 --format=%H main` == HEAD at start).
Date: 2026-08-05. Method: static source trace only (see § *What I ran*).

Independent trace of two mechanical questions about the deep-time loop, produced
from source without reading any prior analysis of the same questions.

---

## Q1 — the fate of weathered mass

### 1.1 What the weathering phase mutates, exactly

The height-tier phase is `Erosion::weather`
(`crates/dc-worldgen/src/deeptime/erosion/weathering.rs:512-602`). It delegates
every per-cell write to `weather_behavior::weather_one_cell`
(`weathering.rs:566-576` parallel / `:586-596` scalar).

The **only** state written per cell is in `WeatherApply::apply`
(`crates/dc-worldgen/src/deeptime/weather_behavior.rs:237-253`):

```rust
let q = rate * self.dt;
*self.r -= q;
*self.h += q;
*self.dh += q;
```

Bound at `weather_behavior.rs:328` to `grid.r[i]`, `grid.h[i]`, `self.dh[i]`.
So the complete write-set is:

| field | file:line | effect |
|---|---|---|
| `grid.r[i]` | `weather_behavior.rs:245` | bedrock stock −q |
| `grid.h[i]` | `weather_behavior.rs:246` | regolith stock +q |
| `Erosion::dh[i]` | `weather_behavior.rs:247` | recorder's net-ΔH accumulator +q |

Plus four **run-total scalars on `TransportLedger`**, all gated and all
read-only-by-probes:

- `ledger.weather_taper_sum`, `ledger.weather_mod_sum`,
  `ledger.weather_cell_epochs` — `weathering.rs:543-545`, gated on `self.denude`;
- `ledger.weathered_m` — `weathering.rs:600`, gated on `self.sorted`, computed as
  `Σgrid.h` after − `Σgrid.h` before (`weathering.rs:518-522`).

These four are consumed **only** by `examples/denudation_probe.rs` and
`examples/facies_probe.rs` (grep over `crates/`: no other reader). They are
gen-time instruments, not world state.

Nothing else is written. In particular the phase does not touch the strata
record, the fact ledger, or the flux record.

### 1.2 Phase ordering, and whether anything zeroes the accumulator

`Erosion::step` (`crates/dc-worldgen/src/deeptime/erosion/mod.rs:704-761`), the
single-epoch driver:

```
self.expose(grid, cfg);            // :725
self.periglacial(grid, cfg);       // :726
self.build_surface(grid);          // :727
self.flood();                      // :728
self.route();                      // :729
self.accumulate_area();            // :730
if cfg.tectonic_history { self.snapshot_bedrock(grid); }   // :731-733
self.transport(grid, cfg);         // :734
self.weather(grid, cfg);           // :735
self.diffuse(grid, cfg, 1.0);      // :736
... exhumation/isostasy ...        // :742-747
if cfg.record { self.record(grid, mem); }                  // :748-750
if cfg.full_agents { self.wind(...); self.wave(...); }     // :756-759
```

The production path is the pass runner, and it topo-sorts to the same order.
`crates/dc-worldgen/src/deeptime/runner.rs:1185-1220`, `PRODUCTION_ORDER`,
asserted by `order_reproduces_the_erosion_step_phase_order` (`runner.rs:1222-1229`):

```
"dc:deep/transport", "dc:deep/flow_record", "dc:deep/weather",
"dc:deep/diffuse", "dc:deep/isostasy", "dc:deep/deposition", ...
```

**`dh` is zeroed exactly once per epoch, at the top of `transport`** —
`crates/dc-worldgen/src/deeptime/erosion/transport.rs:538`:

```rust
self.dh.iter_mut().for_each(|d| *d = 0.0);
```

A grep for `dh` across `crates/dc-worldgen/src/deeptime/` returns exactly four
writers — `transport.rs` (`:314` `-= ent`, `:431/:437/:499/:575/:661` `+=`),
`weathering.rs` via `weather_behavior`, `creep_kernel.rs:374` (`+= netdiff`), and
the reader `record.rs:212/242/340/350/370/374` — and exactly one zeroing site
(`transport.rs:538`).

Since `transport` runs **before** `weather` and `record` runs **after**, **the
weathered `q` survives the epoch and reaches the recorder.** Verified.

### 1.3 Where the mass ends up

Three destinations, and only three:

1. **`grid.h` — the regolith height plane.** Durable world state; `H` is the
   loose stock the strata record mirrors (`weather_behavior.rs:18-22, 235`).
2. **The strata record**, via `dh` → `Erosion::record`
   (`crates/dc-worldgen/src/deeptime/erosion/record.rs:194-377`) →
   `record_cell` (`record.rs:74-90`) → `DeepStrata::deposit_moved`
   (`crates/dc-worldgen/src/deeptime/recorder.rs:1009-1054`). Only if the cell's
   **net** `dh` for the epoch is positive: `record.rs:81-89` returns on
   `|dh| < 1e-9` and calls `s.erode(-dh)` when negative. Weathered metres are
   summed into that one scalar and are not separable from transport/creep
   contributions at this point.
3. **Nowhere else.** It does **not** reach:
   - the fact ledger (`inventory.rs` / `FactLedger`) — the height-tier pass has
     no ledger handle at all; the `dc:deep/weather` pass declares
     `writes: &[DeltaH, Weathered]` (`runner.rs:876-883`), no `Saprolite`;
   - the flux record (`flux.rs`) — `flux::add_epoch` is fed only from
     `erosion.out_face_area()/out_face_load()/area()/routed_surface()`
     (`runner.rs:524-540`), all transport/drainage outputs. Grep for `weather` in
     `deeptime/flux.rs` returns one doc comment (`flux.rs:328`) and no code path.

### 1.4 What is recorded about its origin

**Material identity.** `record.rs:322-326` (`deposit_at`) computes the tag from
the *current environment* and calls `species_at` (`record.rs:233-321`).
`species_at` builds `carried` from `arriving_material`
(`record.rs:134-181`), which sums **only** the two identity-carrying movers —
fluvial `dep_sp` and hillslope `creep_sp`. Weathered mass is in **neither**; the
module's own comment names it (`record.rs:99-102`):

> What is left in `dh` — bedrock weathered to regolith in place, wind and wave,
> the biotic layer — never rode any mover, and is the *incumbent*.

`arriving_material` sets `best_m = dh - carried` (`record.rs:162`) and only a
**strict** `>` displaces it (`:165-168`). So:

- **If the movers win**, the weathered metres are recorded under the *transported*
  material's `MaterialId` (`record.rs:300` returns `(m, mover)`). The
  in-place-made rock is silently absorbed into a river deposit's identity.
- **If the incumbent (weathered/wind/wave/biotic remainder) wins**, `carried` is
  `None` and the identity comes from `record.rs:307`:
  `(lithology::litho_of_tag(tag), MOVER_NONE)` — then a **fitness draw**
  `mem.surface(i, temp_c, precip[i], draw_class, dep_tags::TRANSPORT, 0)`
  (`record.rs:310-320`). `litho_of_tag`
  (`crates/dc-worldgen/src/deeptime/lithology.rs:460-474`) is a pure function of
  the *depositional environment* (biota facies, subsea/subaerial, energy band).

  **So the identity of in-place weathered regolith is derived from the
  environment it sits in, not from the bedrock it came from.** There is no code
  path from `grid.r`'s material (the basement) to the recorded species on this
  tier. The parent link is absent by construction: `R` is basement everywhere
  and carries no per-cell material (`weather_behavior.rs:26-35`, the S16
  diagnostic, which states this explicitly as the known strain).

**Position in the stack.** `deposit_moved` either merges into the top unit when
the merge key matches (`recorder.rs:1029-1037`) or `self.units.push(unit)`
(`recorder.rs:1052`). Units run bottom→top (`lithology.rs:555`, `window_walk`
iterates `units.iter().rev()` to walk down from the surface). So the weathered
metres land at the **top** of the loose stack — above everything, in the same
slot a transported deposit would occupy. Physically the conversion happens at
the *base* of the regolith (a saprolite front at the bedrock contact); the record
places it at the surface. Verified from code; flagged as a fidelity gap, not a
bug in the traced sense.

**Produced-in-place vs transported-in.** Yes, one axis exists: the **mover byte**.
`MOVER_NONE = 7` (`recorder.rs:243`) is written for the incumbent case
(`record.rs:307`), against `FlowCause::Fluvial = 0` / `Gravity = 3`
(`crates/dc-worldgen/src/deeptime/flux.rs:337-352`) for the mover cases. It is
stored in `DepUnit` bits 16–18 (`recorder.rs:522 MOVER_SHIFT`, `:458` the layout
table) and it is **in the merge key** (`recorder.rs:543`), so an in-place unit and
a transported unit of the same rock under the same environment are two units.

The caveat is sharp: the mover byte says *"nothing outvoted the remainder here"*,
not *"this metre was made here"*. When a mover wins, the weathered fraction of the
same `dh` is recorded under that mover's byte. The distinction is a per-unit
plurality verdict, not a per-metre provenance.

### 1.5 Is the conversion event itself recorded anywhere?

**On the height tier (the shipped default): no.** Nothing durable says
*"granite became regolith"*. `R` falls, `H` rises, `dh` accumulates, a unit is
appended with an environment-derived identity. The only trace of the event as an
event is `ledger.weathered_m` (`weathering.rs:600`) — a single run-total f64 read
only by two probe examples, never persisted.

**On the inventory tier (off by default): yes, explicitly.**
`weather_inventory::weather_bedrock_epoch`
(`crates/dc-worldgen/src/deeptime/weather_inventory.rs:291-330`) emits, per agent,
a `Fact` on the `Structure → Loose` edge of `BEDROCK_SEAM_MATERIAL`, carrying the
`Cause` (`Chemical` / `Biotic` / `Frost` — `weather_inventory.rs:125`). The unit
test pins the shape: `f.from() == (BEDROCK_SEAM_MATERIAL, InvForm::Structure)`,
`f.to() == (BEDROCK_SEAM_MATERIAL, InvForm::Loose)`
(`weather_inventory.rs:484-487`). That is a literal, durable "X became Y, by
agent A" record — in the `FactLedger` sidecar, never in the strata record.

### 1.6 How many weathering systems, and the flag

**Two**, and they are the same physical process at two tiers.

| | `dc:deep/weather` | `dc:deep/weather_inventory` |
|---|---|---|
| code | `erosion/weathering.rs` + `weather_behavior.rs` | `weather_inventory.rs` |
| pass decl | `runner.rs:876-883` | `runner.rs:996-1012` |
| rate form | **product** `base × ((biotic × weatherability) × frost) × taper` (`weather_behavior.rs:205-209`) | **sum over agents** `taper × Σ_a (driver_a × susc_{m,a})` (`weather_inventory.rs:201-206`) |
| writes | `grid.r`, `grid.h`, `dh` (+`Weathered` axis) | **only** the `FactLedger` sidecar (`Saprolite` axis) |
| flag | none — always present | `DeepConfig::weather_inventory` |
| **default** | on | **`false`** — `crates/dc-worldgen/src/deeptime/grid.rs:590` |

The split is stated as doctrine at `weather_inventory.rs:341-343`: *"writes only
the ledger sidecar — never `r`/`h`/the record … this pass owns material
composition, the height-tier `dc:deep/weather` pass owns the `R`/`H` budget, and
they do not touch each other."* And they are deliberately **not** numerically
consistent — `weather_inventory.rs:16-20` says the sum form is *"deliberately NOT
byte-identical to S16's product"*.

Consequence, stated plainly: **in the shipped default configuration the only
durable record of a weathering event is a height change and an
environment-named unit at the top of the stack. The pass that records the
conversion as a fact is off.**

### Q1 — one sentence

*Weathered mass moves from `R` to `H`, is summed into the epoch's single scalar
`dh` alongside transport and creep, and — if the cell's net `dh` is positive —
is committed to the top of the strata stack inside one unit whose material
identity comes either from whatever mover outvoted it or from a fitness draw on
the depositional environment, so the mass survives in full while every trace of
its bedrock parentage is lost.*

---

## Q2 — composition at deposition

### 2.1 How many units, and what identity

**Exactly one unit at most, carrying exactly one `MaterialId`.**

`Erosion::record` calls `record_cell` once per cell
(`record.rs:340`, `:350`, `:370`, `:374`). `record_cell`
(`record.rs:74-90`) makes **one** call on a net gain:

```rust
if dh > 0.0 {
    let (tag, species, mover) = deposit();
    s.deposit_moved(tag, dh, chapter, epoch, species, mover);   // record.rs:86
}
```

`deposit_moved` (`recorder.rs:1009-1054`) takes a **scalar `d: f64`** and a
**single `species: MaterialId`**, and its outcomes are:

- sub-quantum ⇒ rides in `self.carry`, **zero** units (`recorder.rs:1022-1025`);
- merge into the existing top when `key_bits()` match ⇒ **zero** new units
  (`recorder.rs:1029-1037`);
- otherwise `self.units.push(unit)` ⇒ **one** unit (`recorder.rs:1052`).

`DepUnit` has no multi-material representation: species is a single 6-bit field
at `SPECIES_SHIFT = 10` (`recorder.rs:521`).

### 2.2 Where the per-species information goes

Transport maintains `dep_sp` — a CSR `SpeciesPlane` of what it set down per
species per cell (`erosion/mod.rs:388`) — and creep maintains `creep_sp`
(`erosion/mod.rs:404`). The recorder reads both (`record.rs:213-214`) and hands
their rows to `arriving_material` (`record.rs:241-248`).

**The collapse is `arriving_material` (`record.rs:134-181`).** It accumulates the
union of the two rows into a dense `mix` (`:149-159`), then takes an **argmax**
against the un-carried remainder:

```rust
let mut best_m = dh - carried;              // record.rs:162
let mut best: Option<usize> = None;
for (k, &m) in mix.iter().take(axis.len()).enumerate() {
    if m > best_m { best_m = m; best = Some(k); }   // record.rs:164-169
}
...
best.map(|k| { ...; (axis.material(k), mover) })    // record.rs:173-180
```

It returns `Option<(MaterialId, u8)>` — **one** material and **one** mover byte.
Everything else in `mix` is discarded when the function returns.

The load-bearing call is `record.rs:86`:

```rust
s.deposit_moved(tag, dh, chapter, epoch, species, mover);
```

`dh` is the **scalar total**; `species` is the **argmax winner**.

### 2.3 What is lost and what is preserved

**Preserved, exactly:**
- **Total mass/thickness.** `dh` is the full net gain, all species and all
  non-mover sources summed. Nothing is dropped; sub-quantum residue rides in
  `DeepStrata::carry` (`recorder.rs:1021-1027`) rather than being truncated.
- **The winning material's identity**, at member grade (`axis.material(k)`,
  `record.rs:179`).
- **The winner's dominant mover** — fluvial vs gravity, `record.rs:173-178`.
- The environmental tag, chapter, and deposition epoch (`record.rs:86`).

**Lost, precisely:**
- **The identity of every non-winning species.** If a cell gains 0.6 m sandstone
  and 0.4 m mudstone, the unit records **1.0 m of sandstone**. The mudstone's
  *mass* is preserved (it is inside `dh`); its *identity* is gone.
- **The winner's own proportion.** `best_m`/`mix[k]` are local variables; the
  ratio "sandstone was 60 % of this metre" is never stored.
- **The split between transported and in-place metres.** `carried` and
  `dh - carried` (`record.rs:162`) are computed and discarded. `mix_dep`
  (`record.rs:147`) exists solely to pick the mover byte and is dropped.
- **Grain grade.** Every unit is written with `GRAIN_UNSET`
  (`recorder.rs:557-561`); the release-spectrum grade split computed by the
  inventory pass is handed to `grain_write_seam`
  (`weather_inventory.rs:115-120`), which deliberately writes nothing.

The code names this itself, at `record.rs:126-128` (STUB #25 note) and
`record.rs:113-133` (the tie rule); the argmax is a stated design choice, not an
oversight. The doc comment's own framing — *"the argmax of the arriving
mixture"* — matches the code exactly; I found no comment/code disagreement in
this file.

### 2.4 Can one epoch-gain at one cell produce >1 unit, or a multi-material unit?

**No, on both counts, on this path.**

- `record` calls `record_cell` once per cell per epoch; `record_cell` makes at
  most one `deposit_moved` call; `deposit_moved` pushes at most one unit.
- `DepUnit` structurally cannot carry two materials (one 6-bit species field,
  `recorder.rs:521`).

Two adjacent facts that could be mistaken for exceptions, and are not:

1. **`wind` and `wave` run after `record`** (`erosion/mod.rs:756-759`) and
   self-record, so a cell *can* end an epoch with more than one new unit. But
   those are separate phases writing their own `dh`-independent deposits with
   their own movers (`FlowCause::Eolian` / `Marine`) — not a second unit from the
   recorder's gain. Same for the biotic layer's `overprint_top`
   (`recorder.rs:1074+`), which alters the existing top rather than stacking.
2. **The merge key** can *prevent* a unit (merge into top, `recorder.rs:1035`) but
   never multiply one.

### Q2 — one sentence

*The per-species composition tracked by transport and creep is collapsed at the
moment of recording into a single argmax winner — one `MaterialId`, one mover
byte — and committed as at most one unit carrying the full scalar thickness, so
all the mass is preserved and every material identity except the plurality
winner's is discarded.*

---

## Verified vs inferred

**Verified from source (every link read):**
- The weathering write-set (`r`, `h`, `dh`, four ledger scalars) and its sole
  application site.
- `dh` is zeroed only at `transport.rs:538`, and `transport` precedes `weather`
  precedes `record` in both `Erosion::step` and the runner's topo order.
- The weathered mass reaches the strata record via `dh`, and reaches neither the
  `FactLedger` nor the flux record on the height-tier path.
- The identity of an incumbent-won deposit is `litho_of_tag` + fitness draw, with
  no reference to bedrock.
- `MOVER_NONE` is stored and is in the merge key.
- Two weathering systems; `weather_inventory` default `false` at `grid.rs:590`.
- The argmax collapse in `arriving_material` and the single scalar + single
  `MaterialId` signature of `deposit_moved`.
- One unit maximum per cell per epoch from the recorder path.

**Inferred (reasoning, not a read of every consumer):**
- *"`ledger.weathered_m` and friends are never persisted."* Established by a grep
  over `crates/` showing only `examples/denudation_probe.rs` and
  `examples/facies_probe.rs` as readers. I did not read the serialization path to
  prove `TransportLedger` is not itself written to disk; the grep is strong but it
  is a grep.
- *"Weathering never reaches the flux record."* Established by reading
  `flow_record_pass` (`runner.rs:524-540`) and grepping `flux.rs` for `weather`.
  I did not read all of `flux.rs` (995 lines).
- *"`sorted` (material-aware transport) is on in production."*
  `DeepConfig::material_transport` and `material_creep` both default `true`
  (`grid.rs:603`, `:606`) and `mod.rs:262/273` wire them. I did not verify that
  the shipped pregen config leaves them at default.

---

## What I could not determine

1. **Whether the physical placement of weathered regolith at the *top* of the
   stack (rather than at the saprolite front below) is a ratified simplification
   or an unexamined consequence.** The code is unambiguous
   (`recorder.rs:1052`); the *intent* is not stated in any comment I read.
   Settled by: a ruling in `docs/design/` or `ARCHITECTURE.md`, or the user.
2. **What fraction of recorded units are actually incumbent-won** (i.e. how often
   weathered mass gets recorded under a transported rock's name). The
   `identity_audit` instrument (`erosion/mod.rs:440-460`, `record.rs:328-365`)
   measures exactly the adjacent question but buckets by provenance class, not by
   weathered-vs-other. Settled by: a run with `identity_audit` on plus a
   dedicated tally of `dh - carried` against `dh`.
3. **Whether `Erosion::step`'s ordering is ever used in production**, or only by
   tests/profiling harnesses. The doc at `mod.rs:694-697` says a caller driving
   `step` owns the deposition clock and that the production loop is
   `DeepSchedule::run`, which implies `step` is not the production path — but I
   did not trace every caller. It does not affect the answer, because both orders
   agree (`runner.rs:1222-1229`).

---

## What I ran

Nothing. The static chain has no unconfirmed link on either question: the write
site, the single zeroing site, the phase order (asserted by an existing test at
`runner.rs:1222`), and the single-scalar/single-`MaterialId` recorder signature
are all directly readable, and no arithmetic depends on runtime values. A build
would have confirmed only what the ordering test already asserts.
