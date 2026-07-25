//! **S17 keystone — the deep cell's working material inventory + the ratified
//! transformation-fact ledger.**
//!
//! Realizes `docs/design/material-behavior.md` §1 and its **"Commit semantics —
//! DECIDED 2026-07-24"**: the deep cell has no per-cell working inventory to
//! read-modify-write, so behaviors like weathering (bedrock → regolith) had
//! nothing to act on. This module builds the substrate the north star names — a
//! stack of `VoxelContents`-shaped spans indexed by depth — **and** the persistent
//! compiled artifact the ratified commit semantics require.
//!
//! **The ratified shape (DECIDED 2026-07-24).** The strata record is keyed by
//! depositional *environment* (`DepTag`), not material, so a material change
//! cannot rewrite it. Resolution:
//!
//! - A unit = its **immutable depositional base** (`DepTag` + thickness) **+ an
//!   appended list of [`Fact`]s** ([`FactLedger`]). Current composition =
//!   `derive(tag)` then **fold the facts** ([`compose_unit`]) — S-9 per unit.
//! - **In-place transformation → append a fact** to the existing unit.
//!   **Depositional arrival → a new unit** (`deposit_deep_history`, untouched).
//! - **Facts persist; the working inventory is transient** (S-2): facts are the
//!   stochastic, state-reading outcome — not re-derivable without re-running the
//!   compile — so they *are* the compiled artifact; the working inventory is
//!   compiler scratch, re-derived from `base + facts` each chapter.
//! - **[`commit_chapter`] = diff-and-append:** the working inventory vs the
//!   chapter-start derived state → the deltas ARE the facts → append. **Empty
//!   delta ⇒ no facts ⇒ record byte-identical** (the S17 identity default).
//!
//! **Provenance addresses the portion's lineage, not the cell** (DECIDED,
//! forward-looking): when contents *move*, the move is itself a fact that travels
//! with the material. Not built here; [`Fact`] is an enum so a `Move` variant
//! lands without disturbing the in-place shape — the representational room the
//! DECIDED asked to leave.
//!
//! **Granularity-agnostic on purpose** (§10): [`Granularity`] selects per-stratum
//! (one span per unit) or per-voxel (spans ≤ one collapse voxel), and [`InvCtx`]
//! indexes by span identically for either. Per-voxel *provenance* falls out as a
//! read over the per-unit `base + facts` (DECIDED), so the fact ledger itself is
//! per-unit — the commit path runs at per-stratum granularity.

use dc_core::{MaterialId, VOXEL_EIGHTHS, VoxelContents};

use super::lithology::litho_of_tag;
use super::recorder::{DeepStrata, DepUnit};

/// Floating tolerance below which a delta is treated as zero (no fact emitted /
/// no portion kept). Deep quantities are metres over eons; 1e-9 m is far below
/// any physical signal and above f64 round-trip noise.
const EPS: f64 = 1e-9;

/// A **finer-than-eighth fractional quantity at depth**, in metres — the deep
/// tier's native unit (`H` is metres; material-behavior.md §1 DECIDED). Eighths
/// appear **only at collapse** ([`quantize_to_eighths`]).
pub type FracM = f64;

/// The **form** a material-portion occupies volume in (material-behavior.md §2).
///
/// `Structure`/`Loose`/`PoreFill`/`Fluid` are the storable roles. `Void` is **not
/// a storable role** — it is the unoccupied complement (§2) — but it *is* a legal
/// **edge endpoint** (§3: edges to/from void change occupancy), so a [`Fact`] may
/// name it as a source or destination (dissolution is `… → Void`). Invariant: a
/// stored [`Portion`] never has `form == Void`; [`build_identity`] and the ctx
/// never create one.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum InvForm {
    /// Coherent, load-bearing framework (the `R`/structural stock).
    Structure,
    /// Granular, unreserved volume that obeys gravity (the `H`/regolith stock).
    Loose,
    /// Material held inside another's reserved-but-unfilled pore capacity.
    PoreFill,
    /// Liquid in pores + open space. **Accommodated, never stored by the identity
    /// default** — the water model derives it (§10, S-2).
    Fluid,
    /// The unoccupied complement — **edge endpoint only, never a stored portion**.
    Void,
}

/// A `(MaterialId, Form)` **portion** with a fractional metre quantity — the exact
/// read-modify-write target §4/§6 name.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Portion {
    pub material: MaterialId,
    pub form: InvForm,
    /// Finer-than-eighth quantity, metres. Never quantized until collapse.
    pub quantity_m: FracM,
}

/// The **cause of a transformation fact** — the responsible party (§1, DECIDED
/// 2026-07-24: *"`cause` is the responsible agent … or actor"*). The apply-time
/// edge log makes it free: each agent applies its own edge, so a chapter where
/// several agents drive one edge yields **one fact per agent** with distinct
/// causes (preserving "frost did 3, biotic did 2"), never a single blended fact.
///
/// Today the inhabited values are the **deeptime weathering agents**. The set is
/// closed-at-compile so adding an agent is a checked extension at every match.
///
/// FORWARD-NOTE (DECIDED 2026-07-24, not built): a present-tier **`Actor`** party
/// — `Actor(ActorId)` for a player/NPC that picks up and deposits material — lands
/// as a further variant, so the same ledger addresses both "frost-weathering did Y
/// at chapter Z" and "player P deposited it". The [`Fact::Move`] forward-note is
/// its structural companion (a move is itself a fact); adding `Actor` here does not
/// disturb the agent variants below.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Cause {
    /// Base **chemical** weathering (the `weathering` config rate; always present).
    Chemical,
    /// **Biotic** weathering — land plants/soil acids accelerate the attack.
    Biotic,
    /// **Frost** (periglacial) freeze–thaw shattering — acts on bare rock too, so
    /// it is *summed*, not multiplied (a product would zero it where biota is zero).
    Frost,
    /// **Dissolution** — carbonate/evaporite into solution (the karst agent,
    /// dormant until the carbonate milestone; §10). Named so the edge-to-`Void`
    /// dissolution fact carries an honest cause.
    Dissolution,
}

impl Cause {
    /// Short name for probe / provenance output.
    pub fn name(self) -> &'static str {
        match self {
            Cause::Chemical => "chemical",
            Cause::Biotic => "biotic",
            Cause::Frost => "frost",
            Cause::Dissolution => "dissolution",
        }
    }
}

/// A **transformation fact** — the ratified persistent compiled artifact (DECIDED
/// 2026-07-24). An enum so a portion-addressed `Move` variant can be added later
/// without disturbing the in-place shape (the room the DECIDED asked to leave).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Fact {
    /// **In-place transformation** on a unit: `fraction_m` metres of `from`
    /// `(material, form)` become `to` `(material, form)`, in tectonic `chapter`,
    /// **driven by `cause`** (the responsible agent). One shape carries all three
    /// §3 process classes:
    /// - **material change** (`from.0 != to.0`) — e.g. diagenesis;
    /// - **form-only change** (`from.0 == to.0`) — crumbling `Structure→PoreFill`;
    /// - **dissolution** (`to.1 == InvForm::Void`) — the portion leaves to the
    ///   complement (no sink portion is created).
    InPlace {
        chapter: u8,
        cause: Cause,
        from: (MaterialId, InvForm),
        to: (MaterialId, InvForm),
        fraction_m: FracM,
    },
    // FORWARD-NOTE (DECIDED 2026-07-24, not built): a
    // `Move { chapter, from_addr, to_addr, portion }` sibling — "provenance
    // addresses the material portion's lineage, not the cell": a move is itself a
    // fact and the portion's fact-history travels with it (transport in deeptime;
    // pickup/deposit in the present). Appending this variant relocates the
    // portion's address; the in-place shape above is untouched.
}

impl Fact {
    /// The tectonic chapter this fact was committed in.
    #[inline]
    pub fn chapter(&self) -> u8 {
        match self {
            Fact::InPlace { chapter, .. } => *chapter,
        }
    }

    /// The **responsible agent** (§1) — the party this transformation is attributed
    /// to. One fact per agent, so this is specific, not a blend.
    #[inline]
    pub fn cause(&self) -> Cause {
        match self {
            Fact::InPlace { cause, .. } => *cause,
        }
    }

    /// The `(from-form, to-form)` **edge** (§3) this fact rode.
    #[inline]
    pub fn edge(&self) -> (InvForm, InvForm) {
        match self {
            Fact::InPlace { from, to, .. } => (from.1, to.1),
        }
    }

    #[inline]
    pub fn from(&self) -> (MaterialId, InvForm) {
        match self {
            Fact::InPlace { from, .. } => *from,
        }
    }

    #[inline]
    pub fn to(&self) -> (MaterialId, InvForm) {
        match self {
            Fact::InPlace { to, .. } => *to,
        }
    }

    #[inline]
    pub fn fraction_m(&self) -> FracM {
        match self {
            Fact::InPlace { fraction_m, .. } => *fraction_m,
        }
    }
}

/// The **persistent fact ledger** (DECIDED 2026-07-24): per record unit, the
/// chapter-ordered list of transformation facts appended by chapter-commits.
/// Parallel to `DeepStrata.units` (indexed by unit position). The base
/// (`DepTag` + thickness) stays the immutable depositional record; **the facts are
/// the compiled deltas** and the only thing a chapter-commit writes.
///
/// **Where it hangs (reported design choice, not a silent divergence).** The
/// ledger is a *sidecar* keyed by unit index rather than a `facts` field grown
/// onto `DepUnit`. `DepUnit` is `Copy` and read across the just-merged
/// `collapse.rs` / `erosion.rs` / `biotic.rs`; growing a `Vec` onto it un-`Copy`s
/// it and churns files A1 merged this cycle. The sidecar keeps this spike's
/// write-set disjoint (only `inventory.rs`) and the base byte-identical. The
/// eventual home is a `RecordedUnit { base, facts }` on `DeepStrata`; that is the
/// integration step, filed as a plea in `docs/spikes/S17-*`.
///
/// **Layout: flat facts + a sparse CSR index** (journal/0100, ported from
/// [`flux.rs`](super::flux) — the in-tree precedent, not a second mechanism).
/// This used to be `Vec<Vec<Fact>>`, one inner `Vec` per slot. Measured on a
/// production world (`docs/spikes/S19-flow-record-cost-results.md` § 3b): **5.83 M
/// inner `Vec`s of which 98.8 % were EMPTY, and 89 % of the ledger's ~150 MiB heap
/// was empty `Vec` headers** — the record spent most of its residency on the
/// *absence* of facts. The facts themselves were 16.6 MiB.
///
/// The CSR shape pays for presence only:
/// - `facts` — every fact, grouped by slot in ascending slot order, and **within a
///   slot in exactly the append order the old inner `Vec` had** (that order is
///   observable: [`commit_chapter`] merges against the *earliest* match and
///   [`compose_unit`] folds in order).
/// - `rows` — one 8-byte `(slot, start)` entry per **non-empty** slot.
///
/// An empty ledger allocates **nothing at all**, which is what a never-weathered
/// cell now costs.
///
/// **Since journal/0102 this is the GEN-TIME ACCUMULATOR, not the resident shape.**
/// One of these per cell is still two `Vec` headers × 297,025 = 13.60 MiB paid
/// before a fact exists — the same defect one level up — so the *resident* record
/// is [`LedgerField`], one grid-wide record with the cell as a CSR row. This struct
/// keeps its mutable, append-and-merge nature because that is what the weathering
/// pass needs every epoch (an insert into a grid-wide array would memmove a million
/// facts); it is compacted into [`LedgerField`] once, at finalize.
#[derive(Clone, Default, PartialEq, Debug)]
pub struct FactLedger {
    /// Every fact, grouped by slot (ascending) and append-ordered within a slot.
    facts: Vec<Fact>,
    /// The CSR index — one row per **non-empty** slot, ascending by `slot`.
    rows: Vec<SlotRun>,
}

/// One **run of facts belonging to a single slot** — the CSR index entry. The run
/// starts at `start` in the flat fact array and ends where the next row starts (or
/// at the array's end for the last row).
///
/// Only slots that actually carry facts get a row. That is the whole point: a
/// per-slot `Vec` header costs 24 bytes *whether or not the slot has anything in
/// it*, and 98.8 % of slots have nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct SlotRun {
    /// The record-unit index (or the bedrock-seam sentinel) this run keys.
    slot: u32,
    /// Index of the run's first fact in `FactLedger::facts`.
    start: u32,
}

impl FactLedger {
    /// An empty ledger for a record (the identity default: every unit starts with
    /// zero facts, so `base + facts` == `base`).
    ///
    /// **No longer sized to the unit count** — under the CSR layout a slot with no
    /// facts has no row and no header, so there is nothing to pre-size. The
    /// argument is kept so callers do not churn, and because the *conceptual*
    /// contract ("this ledger belongs to this record") is unchanged.
    pub fn empty_for(_strata: &DeepStrata) -> Self {
        Self::default()
    }

    /// An empty ledger for a record **plus its bedrock `Structure` seam slot**
    /// (STUB #16, at index `units.len()` — the LAST slot). This is the ledger
    /// [`build_working`] and the weathering pass expect, so the `Structure→Loose`
    /// facts have somewhere to live.
    ///
    /// Like [`Self::empty_for`] this allocates nothing: the bedrock slot's row
    /// appears the moment a fact is committed to it, and costs nothing until then.
    pub fn empty_with_bedrock(_strata: &DeepStrata) -> Self {
        Self::default()
    }

    /// The bedrock seam's composed portions from `unit_count` (= the record's unit
    /// count; the bedrock slot is at that index). Reads `base + facts` for the
    /// bedrock (§E, the consumer fold): [`compose_bedrock`] of its facts.
    pub fn bedrock_composition(&self, unit_count: usize) -> Vec<Portion> {
        compose_bedrock(self.facts_for(unit_count))
    }

    /// **The weathering product** (metres of `Loose` [`BEDROCK_SEAM_MATERIAL`] the
    /// committed `Structure→Loose` facts produced) — the quantity the collapse
    /// expresses as a basal weathering-front band. `0.0` when the bedrock seam was
    /// never weathered (the identity default).
    pub fn weathering_product_m(&self, unit_count: usize) -> FracM {
        self.bedrock_composition(unit_count)
            .iter()
            .filter(|p| p.form == InvForm::Loose)
            .map(|p| p.quantity_m)
            .sum()
    }

    /// The end of row `k`'s run in `facts`.
    #[inline]
    fn row_end(&self, k: usize) -> usize {
        self.rows
            .get(k + 1)
            .map_or(self.facts.len(), |r| r.start as usize)
    }

    /// The facts appended to unit `i` (empty slice when none / out of range) — the
    /// CSR row lookup. Unchanged in signature, in contents and in **order** from
    /// the `Vec<Vec<Fact>>` era.
    #[inline]
    pub fn facts_for(&self, i: usize) -> &[Fact] {
        let Ok(key) = u32::try_from(i) else {
            return &[];
        };
        match self.rows.binary_search_by_key(&key, |r| r.slot) {
            Ok(k) => &self.facts[self.rows[k].start as usize..self.row_end(k)],
            Err(_) => &[],
        }
    }

    /// **Append a fact to `slot`, merging into the EARLIEST fact already there with
    /// the same `(chapter, cause, from, to)` key** — the exact semantics
    /// [`commit_chapter`] had against the per-slot `Vec`, preserved to the letter
    /// (see its docs for why the *earliest* match, and not the last, is the
    /// load-bearing choice).
    ///
    /// Opening a new slot, or appending to one that is not the last, shifts the
    /// tail of the flat array. That is an `O(facts after the slot)` memmove on a
    /// **per-cell** array of at most a few dozen facts, at **gen time** — the clock
    /// this project spends freely. Runtime pays only the smaller residency.
    fn append_merged(
        &mut self,
        slot: usize,
        chapter: u8,
        cause: Cause,
        from: (MaterialId, InvForm),
        to: (MaterialId, InvForm),
        fraction_m: FracM,
    ) {
        let key = u32::try_from(slot).expect("a slot index fits in u32");
        let new = Fact::InPlace {
            chapter,
            cause,
            from,
            to,
            fraction_m,
        };
        let (at, row_at) = match self.rows.binary_search_by_key(&key, |r| r.slot) {
            Ok(k) => {
                let (start, end) = (self.rows[k].start as usize, self.row_end(k));
                // Search this slot's run IN ORDER: the earliest match wins.
                for f in &mut self.facts[start..end] {
                    let Fact::InPlace {
                        chapter: c,
                        cause: ca,
                        from: fr,
                        to: t,
                        fraction_m: q,
                    } = f;
                    if *c == chapter && *ca == cause && *fr == from && *t == to {
                        *q += fraction_m;
                        return;
                    }
                }
                // No match: append at the END of this slot's run.
                (end, k + 1)
            }
            // A slot with no facts yet: its run opens where the next row starts.
            Err(k) => {
                let at = self
                    .rows
                    .get(k)
                    .map_or(self.facts.len(), |r| r.start as usize);
                self.rows.insert(
                    k,
                    SlotRun {
                        slot: key,
                        start: at as u32,
                    },
                );
                (at, k + 1)
            }
        };
        self.facts.insert(at, new);
        for r in &mut self.rows[row_at..] {
            r.start += 1;
        }
    }

    /// **Move the facts at `from_slot` onto `to_slot`, exact-sized** — the finalize
    /// step ([`finalize_ledgers`](super::weather_inventory::finalize_ledgers)): the
    /// in-loop accumulator keys its bedrock facts at a stable sentinel slot, and the
    /// resident consumer reads them at the final record's bedrock index.
    ///
    /// Every allocation the returned ledger owns is **exactly the size of its
    /// contents** — no `Vec` doubling slack. (Reclaiming that slack on `strata`
    /// recovered 33 % of all `DeepField` residency, so it is not a rounding error;
    /// and the compile is over, so growing room is pure waste.) Facts at other
    /// slots are dropped, matching the pre-CSR `finalize_ledgers` exactly — the
    /// accumulator only ever holds the sentinel slot.
    pub fn rekeyed(&self, from_slot: usize, to_slot: usize) -> Self {
        let facts = self.facts_for(from_slot);
        if facts.is_empty() {
            return Self::default();
        }
        Self {
            facts: facts.to_vec(),
            rows: vec![SlotRun {
                slot: u32::try_from(to_slot).expect("a slot index fits in u32"),
                start: 0,
            }],
        }
    }

    /// Release every byte of growth slack (the end-of-compile compaction).
    pub fn compact(&mut self) {
        self.facts.shrink_to_fit();
        self.rows.shrink_to_fit();
    }

    /// True when no fact has been committed anywhere — the identity-default state.
    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    /// Total facts across all units (a size metric).
    pub fn total_facts(&self) -> usize {
        self.facts.len()
    }

    /// How many slots actually carry a fact — the CSR row count. The sparsity
    /// numerator; every other slot is one the layout no longer pays for.
    pub fn slots_with_facts(&self) -> usize {
        self.rows.len()
    }

    /// The **payload** half of the footprint (bytes): the facts themselves.
    pub fn payload_bytes(&self) -> usize {
        self.facts.capacity() * std::mem::size_of::<Fact>()
    }

    /// The **index** half of the footprint (bytes): the CSR rows. `flux.rs`
    /// measured its index floor at 0.056× of total; this one is smaller still,
    /// because a row is paid only for a slot that has facts.
    pub fn index_bytes(&self) -> usize {
        self.rows.capacity() * std::mem::size_of::<SlotRun>()
    }

    /// Heap footprint (bytes): the flat fact array plus the CSR index.
    pub fn footprint_bytes(&self) -> usize {
        self.payload_bytes() + self.index_bytes()
    }
}

// ===========================================================================
// The grid-wide ledger record (journal/0102) — the cell as a CSR row.
// ===========================================================================

/// **One fact ledger for the WHOLE deep grid, with the CELL as a CSR row** — the
/// resident shape [`DeepField::ledgers`](super::field::DeepField::ledgers) carries.
///
/// **The defect it exists to kill.** journal/0100 collapsed the *inner* dimension
/// (`Vec<Vec<Fact>>` → flat facts + slot rows) and, in doing so, grew the per-cell
/// [`FactLedger`] struct from 24 to 48 bytes — two `Vec` headers. Across a
/// production grid of **297,025 cells** that is **13.60 MiB paid before a single
/// fact is stored**, and 75.8 % of those cells never weather at all. The
/// generalisation outlives this record and is the reason it is written down twice:
///
/// > *any per-cell OWNING CONTAINER in a 297 k-cell field costs a header per cell
/// > before it holds data.* The default for anything per-cell is **one grid-wide
/// > record with CSR rows**, never `Vec<Something>` per cell.
///
/// **The shape is [`flux.rs`](super::flux)'s, ported — not a second mechanism**
/// (A-4). [`FluxRecord`](super::flux::FluxRecord) is flat `entries` + a dense
/// `cell_start` offsets array; this is flat [`Fact`]s + the *slot* rows of
/// journal/0100 + a dense `cell_row_start` over those rows. Two levels of CSR, the
/// outer one dense (every cell exists and the offsets array *is* the addressing
/// scheme — 4 B/cell = 1.13 MiB) and the inner one sparse (a slot row is paid only
/// where facts exist).
///
/// A cell is read as a [`LedgerView`] — a borrowed window with exactly the read
/// surface [`FactLedger`] had. [`FactLedger`] itself survives as the **gen-time
/// accumulator**: the per-cell mutable scratch the weathering pass appends into
/// every epoch, which is compacted into this record once, at finalize.
#[derive(Clone, Default, PartialEq, Debug)]
pub struct LedgerField {
    /// Every fact in the world, grouped by cell (ascending), then by slot
    /// (ascending), then in append order within a slot.
    facts: Vec<Fact>,
    /// Every non-empty slot's run, in the same cell-major order. `start` is an
    /// **absolute** index into [`Self::facts`], so a row's end is the next row's
    /// start — uniformly, including across a cell boundary.
    rows: Vec<SlotRun>,
    /// `cell_row_start[i] .. cell_row_start[i + 1]` is cell `i`'s slice of
    /// [`Self::rows`]. Length `cells + 1`, or **empty when the record is empty**
    /// (the flag is off) — the S-5 identity default, which then costs nothing.
    cell_row_start: Vec<u32>,
}

impl LedgerField {
    /// **Compact the per-cell gen-time accumulators into the resident record**,
    /// re-keying each cell's `from_slot` run onto `to_slot(cell)`.
    ///
    /// This is [`FactLedger::rekeyed`] done once for the whole grid: the in-loop
    /// accumulator keys its bedrock facts at a stable sentinel slot (journal/0094)
    /// and the collapse consumer reads them at the final record's bedrock index.
    /// Facts at any other slot are dropped, exactly as the per-cell `rekeyed` did —
    /// the accumulator only ever holds the sentinel slot.
    ///
    /// Every array is **exact-sized**: the compile is over, growing room is waste.
    pub fn from_accumulators(
        accumulators: &[FactLedger],
        from_slot: usize,
        to_slot: impl Fn(usize) -> usize,
    ) -> Self {
        let cells = accumulators.len();
        if cells == 0 {
            return Self::default();
        }
        let runs = accumulators.iter().map(|a| a.facts_for(from_slot));
        let total_facts: usize = runs.clone().map(<[Fact]>::len).sum();
        let total_rows: usize = runs.filter(|r| !r.is_empty()).count();
        let mut facts = Vec::with_capacity(total_facts);
        let mut rows = Vec::with_capacity(total_rows);
        let mut cell_row_start = Vec::with_capacity(cells + 1);
        for (i, acc) in accumulators.iter().enumerate() {
            cell_row_start.push(u32::try_from(rows.len()).expect("a row index fits in u32"));
            let run = acc.facts_for(from_slot);
            if run.is_empty() {
                continue;
            }
            rows.push(SlotRun {
                slot: u32::try_from(to_slot(i)).expect("a slot index fits in u32"),
                start: u32::try_from(facts.len()).expect("a fact index fits in u32"),
            });
            facts.extend_from_slice(run);
        }
        cell_row_start.push(u32::try_from(rows.len()).expect("a row index fits in u32"));
        Self {
            facts,
            rows,
            cell_row_start,
        }
    }

    /// The number of **cells** the record covers (`0` when the flag is off).
    pub fn len(&self) -> usize {
        self.cell_row_start.len().saturating_sub(1)
    }

    /// True when the record covers no cells — the flag-off identity default.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// **Cell `i`'s ledger**, or `None` out of range. A borrowed window, not an
    /// owned struct — the whole point.
    pub fn get(&self, i: usize) -> Option<LedgerView<'_>> {
        let a = *self.cell_row_start.get(i)? as usize;
        let b = *self.cell_row_start.get(i + 1)? as usize;
        let end = self
            .rows
            .get(b)
            .map_or(self.facts.len(), |r| r.start as usize);
        Some(LedgerView {
            facts: &self.facts,
            rows: &self.rows[a..b],
            end: end as u32,
        })
    }

    /// Every cell's ledger, in cell order.
    pub fn iter(&self) -> impl Iterator<Item = LedgerView<'_>> {
        (0..self.len()).map(|i| self.get(i).expect("cell index is in range"))
    }

    /// Total facts across the whole grid.
    pub fn total_facts(&self) -> usize {
        self.facts.len()
    }

    /// Non-empty `(cell, slot)` runs across the whole grid — the sparsity
    /// numerator. Every other slot is one the layout no longer pays for.
    pub fn slots_with_facts(&self) -> usize {
        self.rows.len()
    }

    /// The **payload** half of the footprint (bytes): the facts themselves.
    pub fn payload_bytes(&self) -> usize {
        self.facts.capacity() * std::mem::size_of::<Fact>()
    }

    /// The **index** half of the footprint (bytes): the sparse slot rows plus the
    /// dense per-cell row offsets. The dense half is 4 B/cell — against the
    /// 48 B/cell of `Vec` headers it replaces.
    pub fn index_bytes(&self) -> usize {
        self.rows.capacity() * std::mem::size_of::<SlotRun>()
            + self.cell_row_start.capacity() * std::mem::size_of::<u32>()
    }

    /// Heap footprint (bytes). There is **no per-cell struct term** — that is the
    /// 13.60 MiB this shape removed.
    pub fn footprint_bytes(&self) -> usize {
        self.payload_bytes() + self.index_bytes()
    }
}

/// **A borrowed view of one cell's ledger** — the read surface [`FactLedger`] had,
/// over the grid-wide [`LedgerField`] instead of an owned per-cell struct.
///
/// `Copy` and two words plus a slice wide: it is created on demand at a call site
/// that used to hold `&FactLedger`, and costs nothing resident.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct LedgerView<'a> {
    /// The record's **whole** fact array — [`SlotRun::start`] is absolute.
    facts: &'a [Fact],
    /// This cell's slot rows, ascending by slot.
    rows: &'a [SlotRun],
    /// The absolute end of this cell's fact range (the next cell's first row's
    /// start, or the array's end).
    end: u32,
}

impl<'a> LedgerView<'a> {
    /// The end of row `k`'s run. The last row of a cell ends at the cell's end;
    /// every other row ends where the next begins.
    #[inline]
    fn row_end(&self, k: usize) -> usize {
        self.rows
            .get(k + 1)
            .map_or(self.end as usize, |r| r.start as usize)
    }

    /// The facts appended to unit `i` (empty slice when none / out of range) —
    /// identical in contents and in **order** to [`FactLedger::facts_for`].
    #[inline]
    pub fn facts_for(&self, i: usize) -> &'a [Fact] {
        let Ok(key) = u32::try_from(i) else {
            return &[];
        };
        match self.rows.binary_search_by_key(&key, |r| r.slot) {
            Ok(k) => &self.facts[self.rows[k].start as usize..self.row_end(k)],
            Err(_) => &[],
        }
    }

    /// The bedrock seam's composed portions — see [`FactLedger::bedrock_composition`].
    pub fn bedrock_composition(&self, unit_count: usize) -> Vec<Portion> {
        compose_bedrock(self.facts_for(unit_count))
    }

    /// **The weathering product** (metres of `Loose` [`BEDROCK_SEAM_MATERIAL`]) —
    /// the quantity the collapse expresses as a basal weathering-front band. See
    /// [`FactLedger::weathering_product_m`].
    pub fn weathering_product_m(&self, unit_count: usize) -> FracM {
        self.bedrock_composition(unit_count)
            .iter()
            .filter(|p| p.form == InvForm::Loose)
            .map(|p| p.quantity_m)
            .sum()
    }

    /// True when this cell carries no fact at all (the 75.8 % case).
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// Facts in this cell.
    pub fn total_facts(&self) -> usize {
        self.rows
            .first()
            .map_or(0, |r| self.end as usize - r.start as usize)
    }

    /// Slots in this cell that carry a fact.
    pub fn slots_with_facts(&self) -> usize {
        self.rows.len()
    }
}

/// The **depositional base composition** of a unit: exactly what `derive(DepTag)`
/// yields before any fact — one `Loose` portion of the unit's reference material
/// (`litho_of_tag(tag).reference_material()`, the same routing the collapse tier's
/// `deep_class` uses), at the unit's full thickness.
pub fn derive_base(unit: &DepUnit) -> Vec<Portion> {
    vec![Portion {
        material: litho_of_tag(unit.tag).reference_material(),
        form: InvForm::Loose,
        quantity_m: unit.thickness_m,
    }]
}

/// **Compose a unit's current composition** = `derive_base` then fold its facts
/// in chapter order (S-9 per unit). This is the provenance-read primitive: it
/// reconstructs "started as X, then chapter-Z weathering did Y" from `base +
/// facts`, and it is exactly the state a chapter's working inventory is built
/// from.
pub fn compose_unit(unit: &DepUnit, facts: &[Fact]) -> Vec<Portion> {
    let mut portions = derive_base(unit);
    for f in facts {
        apply_move(&mut portions, f.from(), f.to(), f.fraction_m());
    }
    portions
}

/// Apply one `(from) → (to)` move of `qty` metres to a portion multiset,
/// mass-conserving except when a side is `Void` (the complement). The shared
/// primitive behind fact-folding ([`compose_unit`]) and the live ctx
/// ([`InvCtx::apply_edge`]), so a committed fact re-folds to exactly the working
/// state it was diffed from. Returns the quantity actually moved.
fn apply_move(
    portions: &mut Vec<Portion>,
    from: (MaterialId, InvForm),
    to: (MaterialId, InvForm),
    qty: FracM,
) -> FracM {
    // Source: `Void` means "from the complement" (deposition) — unlimited; else
    // clamp to what the source portion holds so a move can never mint material.
    let avail = if from.1 == InvForm::Void {
        qty
    } else {
        portions
            .iter()
            .find(|p| p.material == from.0 && p.form == from.1)
            .map_or(0.0, |p| p.quantity_m)
    };
    let q = qty.min(avail).max(0.0);
    if q <= EPS {
        return 0.0;
    }
    if from.1 != InvForm::Void {
        for p in portions.iter_mut() {
            if p.material == from.0 && p.form == from.1 {
                p.quantity_m -= q;
            }
        }
        portions.retain(|p| p.quantity_m > EPS);
    }
    // Destination: `Void` means "to the complement" (dissolution) — no sink.
    if to.1 != InvForm::Void {
        match portions
            .iter_mut()
            .find(|p| p.material == to.0 && p.form == to.1)
        {
            Some(p) => p.quantity_m += q,
            None => portions.push(Portion {
                material: to.0,
                form: to.1,
                quantity_m: q,
            }),
        }
    }
    q
}

/// One **depth-span** of the working inventory: a `VoxelContents`-shaped mixture
/// (a [`Portion`] multiset) over a depth interval, carrying the provenance a
/// commit needs (which record unit it derives from).
#[derive(Clone, PartialEq, Debug)]
pub struct InvSpan {
    /// The record unit this span derives from (its index in `DeepStrata.units`) —
    /// the address a committed fact is appended to.
    pub unit_index: usize,
    /// The material×form multiset over this interval. Behaviors RMW this.
    pub portions: Vec<Portion>,
}

impl InvSpan {
    /// The span's thickness — the sum of its portion quantities (metres).
    #[inline]
    pub fn thickness_m(&self) -> FracM {
        self.portions.iter().map(|p| p.quantity_m).sum()
    }

    /// Rough heap footprint of this span's portion vector (bytes).
    #[inline]
    pub fn heap_bytes(&self) -> usize {
        self.portions.capacity() * std::mem::size_of::<Portion>()
    }
}

/// The vertical resolution of the working inventory (material-behavior.md §10 — a
/// measurement call, closed by `docs/spikes/S17` → per-stratum).
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Granularity {
    /// One span per record unit — cheaper, and span↔unit is 1:1 (exact commit).
    PerStratum,
    /// Spans ≤ one collapse voxel — a unit subdivided into slices, each tagged to
    /// its parent unit. Measured heavier; used only for the memory tradeoff.
    PerVoxel { voxel_m: f64 },
}

// ===========================================================================
// The bedrock Structure seam (stubs.md #16 — DOWN-AND-DIRTY, heir = genesis pass)
// ===========================================================================
//
// STUB #16 (docs/design/stubs.md), added 2026-07-24 for the first-real-behavior
// weathering slice. The working inventory derives every RECORD unit as `Loose`
// and holds bedrock only as the scalar `basement: MaterialId`. Weathering is
// `Structure → Loose`, so there was **no `Structure` source to weather from**.
// [`build_working`] therefore materializes a single flat basement `Structure`
// span at the BASE of the column, purely so the edge has a source.
//
// This is a stub on the **emplacement axis**: real bedrock is plutons, sills and
// province lithologies unroofed at real depths, not one flat basement material at
// a made-up thickness. **Heir: a genesis/emplacement pass** (ROADMAP 2026-07-24,
// "genesis passes model the honest genesis of rocks") that puts bedrock into the
// inventory as `Structure` with real per-column material identity and depth — at
// which point this stand-in is DELETED, not reimplemented.
//
// Blast: which material a weathering rind is made of in the deep tier (the loose
// product inherits the bedrock's identity); invisible until the weathering facts
// express through the collapse.

/// Provisional thickness (metres) of the materialized bedrock `Structure` seam —
/// **STUB #16**, chosen to outlast the accumulated `Structure→Loose` draw of a full
/// run (measured max **6.094 m of 50 m** at production scale, journal/0094).
///
/// **A-2 correction (spine-audit 2026-07-24):** this previously read "an
/// effectively-inexhaustible source over the one chapter this slice weathers" — that
/// justification **expired** when journal/0094 made weathering a per-epoch process
/// that ACCUMULATES over the whole run. The constant is therefore a live **ceiling on
/// saprolite depth worldwide**, not an inexhaustible source, and it will bite first
/// where the process runs strongest (thin cover, long subaerial residence, a cranked
/// `--erosion-budget`). The heir (a genesis/emplacement pass) supplies a real
/// per-column unroofing depth and retires it.
pub const BEDROCK_SEAM_THICKNESS_M: FracM = 50.0;

/// Provisional basement material of the bedrock seam — **STUB #16**. One flat
/// granite basement everywhere; the heir supplies per-column province lithology.
pub const BEDROCK_SEAM_MATERIAL: MaterialId = MaterialId::GRANITE;

/// One **applied edge**, logged as [`InvCtx`] runs it (§1 refined 2026-07-24:
/// *"the logged edges ARE the facts"*). Carries the cause and chapter the ctx was
/// scoped to, so a chapter with several agents driving one edge coalesces to **one
/// fact per agent** — not a single diffed fact that has lost per-agent provenance.
#[derive(Clone, Copy, PartialEq, Debug)]
struct LoggedEdge {
    /// The record-unit (or bedrock-seam) index the fact is appended to.
    unit_index: usize,
    chapter: u8,
    cause: Cause,
    from: (MaterialId, InvForm),
    to: (MaterialId, InvForm),
    fraction_m: FracM,
}

/// A deep cell's **mutable working material inventory** — the transient compiler
/// scratch of the ratified model, re-derived from `base + facts` each chapter.
#[derive(Clone, PartialEq, Debug)]
pub struct WorkingInventory {
    /// Spans bottom-up. `spans[0..units.len()]` mirror `DeepStrata.units`; a final
    /// bedrock `Structure` span (STUB #16) is appended at index `units.len()` by
    /// [`build_working`] so weathering has a source.
    pub spans: Vec<InvSpan>,
    /// The basement material below the record (derived; not committed — the record
    /// only ever held the `H` column).
    pub basement: MaterialId,
    /// The granularity this inventory was built at.
    pub granularity: Granularity,
    /// **Chapter-start snapshot** per span (the state `build_working` derived),
    /// against which [`diff_facts`] reconciles the drained log (a validation check
    /// now, not the fact source — §1 refined 2026-07-24). Empty for a
    /// [`build_identity`] inventory (which is not for committing) — that keeps the
    /// S17 memory-measurement footprint unchanged.
    baseline: Vec<Vec<Portion>>,
    /// **The applied-edge log** — the authoritative fact source (§1). Each
    /// [`InvCtx::apply_edge`] appends one entry; [`commit_chapter`] drains it.
    log: Vec<LoggedEdge>,
}

/// **The identity default** (base only, no facts, no baseline): build a working
/// inventory straight from a record where each span carries its unit's reference
/// material as `Loose`. The memory-measurement builder; equivalent to
/// [`build_working`] with an empty ledger, minus the commit baseline.
pub fn build_identity(strata: &DeepStrata, granularity: Granularity) -> WorkingInventory {
    let mut spans = Vec::new();
    for (ui, u) in strata.units.iter().enumerate() {
        let base = derive_base(u);
        match granularity {
            Granularity::PerStratum => spans.push(InvSpan {
                unit_index: ui,
                portions: base,
            }),
            Granularity::PerVoxel { voxel_m } => {
                let mat = base[0].material;
                let mut consumed = 0.0f64;
                while consumed < u.thickness_m {
                    let t = (u.thickness_m - consumed).min(voxel_m);
                    spans.push(InvSpan {
                        unit_index: ui,
                        portions: vec![Portion {
                            material: mat,
                            form: InvForm::Loose,
                            quantity_m: t,
                        }],
                    });
                    consumed += t;
                }
            }
        }
    }
    WorkingInventory {
        spans,
        basement: MaterialId::GRANITE,
        granularity,
        baseline: Vec::new(),
        log: Vec::new(),
    }
}

/// **Build the chapter's working inventory from `base + facts`** (the ratified
/// re-derive-each-chapter step, S-2), at per-stratum granularity, capturing the
/// baseline the commit reconciles against, **and materializing the bedrock
/// `Structure` seam (STUB #16)** at the base so `Structure→Loose` weathering has a
/// source. This is the inventory a behavior mutates; with an **empty** ledger the
/// record spans equal [`build_identity`]'s (the identity default) and the extra
/// bedrock span carries no facts, so nothing commits.
///
/// The bedrock span is at index `strata.units.len()` (the LAST span), and the
/// `ledger` is expected to carry a matching slot ([`FactLedger::empty_with_bedrock`]);
/// its base is [`compose_bedrock`] of the ledger's bedrock facts.
pub fn build_working(strata: &DeepStrata, ledger: &FactLedger) -> WorkingInventory {
    let mut spans = Vec::with_capacity(strata.units.len() + 1);
    let mut baseline = Vec::with_capacity(strata.units.len() + 1);
    for (ui, u) in strata.units.iter().enumerate() {
        let portions = compose_unit(u, ledger.facts_for(ui));
        baseline.push(portions.clone());
        spans.push(InvSpan {
            unit_index: ui,
            portions,
        });
    }
    // STUB #16: the bedrock Structure seam. Its base + facts compose the same way a
    // record unit does, so re-running a chapter is idempotent on it too.
    let bedrock_index = strata.units.len();
    let bedrock = compose_bedrock(ledger.facts_for(bedrock_index));
    baseline.push(bedrock.clone());
    spans.push(InvSpan {
        unit_index: bedrock_index,
        portions: bedrock,
    });
    WorkingInventory {
        spans,
        basement: BEDROCK_SEAM_MATERIAL,
        granularity: Granularity::PerStratum,
        baseline,
        log: Vec::new(),
    }
}

/// The bedrock seam's **base composition** (STUB #16): one `Structure` portion of
/// [`BEDROCK_SEAM_MATERIAL`] at [`BEDROCK_SEAM_THICKNESS_M`], before any fact.
pub fn derive_bedrock() -> Vec<Portion> {
    vec![Portion {
        material: BEDROCK_SEAM_MATERIAL,
        form: InvForm::Structure,
        quantity_m: BEDROCK_SEAM_THICKNESS_M,
    }]
}

/// Compose the bedrock seam's current composition = [`derive_bedrock`] then fold
/// its facts (the `Structure→Loose` weathering the pass committed). The `Loose`
/// portion this yields is the weathering product the collapse expresses.
pub fn compose_bedrock(facts: &[Fact]) -> Vec<Portion> {
    let mut portions = derive_bedrock();
    for f in facts {
        apply_move(&mut portions, f.from(), f.to(), f.fraction_m());
    }
    portions
}

/// **The chapter-commit — drain the applied-edge LOG** (§1 refined 2026-07-24:
/// *"the logged edges ARE the facts"*). Each edge the behavior applied logged
/// itself with its cause and chapter; this coalesces identical successive edges
/// (same unit · chapter · cause · edge endpoints) and appends **one [`Fact`] per
/// agent** to the target unit in `ledger`. Clears the log.
///
/// - **Empty log ⇒ no facts appended ⇒ the record is byte-identical** (the S17
///   identity default, now on the fact path — an unweathered inventory commits
///   nothing even though the bedrock seam exists).
/// - The strata record's `units` (the depositional base) are **never written** —
///   only the ledger grows (the ratified "base immutable, append facts").
/// - The **diff is not the source** any more: a multi-edge chapter has no unique
///   factorization, so a working-vs-baseline diff would lose per-agent provenance.
///   [`diff_facts`] survives only as a single-edge **validation check**
///   ([`reconciles_with_diff`]).
///
/// The ledger is grown to cover every unit index the log touches (including the
/// bedrock seam's `units.len()` slot).
pub fn commit_chapter(inv: &mut WorkingInventory, ledger: &mut FactLedger) {
    // Coalesce logged edges into one fact per `(chapter, cause, from, to)`.
    //
    // **A-4 fold (FLOW slice 1, journal/0096).** This lookup used to match only
    // `facts.last_mut()` — *consecutive* identical edges — and a second merger,
    // `weather_inventory::coalesce_facts`, then re-swept every slot to catch the
    // non-consecutive ones. Two mechanisms for one job. The generalized search is
    // the merge both wanted: it preserves first-occurrence order (the merge lands
    // on the *earliest* matching fact, exactly as a post-hoc sweep would), it is
    // idempotent, and the composed band `Σ fraction_m` is invariant under it — so
    // the post-hoc sweep it subsumes was deleted, not kept "just in case".
    //
    // Why it matters beyond tidiness: `weather_bedrock_epoch` fires three agents
    // per epoch in rotation (chem, biotic, frost, chem, …), so *no* two successive
    // edges on a slot ever matched, and last-only merging grew facts 3-per-firing
    // — hundreds per cell — until the second merger reaped them. One merger that
    // is right at the point of writing needs no reaper.
    // **The CSR port (journal/0100).** The merge-or-append above is now
    // [`FactLedger::append_merged`], which does the same search over the same
    // per-slot run — the run is a window into one flat array instead of its own
    // `Vec`. Nothing about *which* fact a merge lands on, or where a new fact is
    // appended, changed; only where the bytes live. The slot no longer has to
    // exist first (there is no per-slot header to `resize` into being) — a row is
    // created lazily by the first fact that needs it.
    for e in inv.log.drain(..) {
        ledger.append_merged(e.unit_index, e.chapter, e.cause, e.from, e.to, e.fraction_m);
    }
}

/// **Validation check** (§1): the drained log's net per-`(material, form)` delta on
/// a span must reconcile with a working-vs-baseline [`diff_facts`]. Exact for a
/// single edge (the diff recovers the net delta); a multi-edge chapter is where the
/// diff loses provenance the log keeps, so this is a test aid, not the commit path.
/// Returns the net-delta facts the diff produces for span `i`.
#[cfg(test)]
fn reconciles_with_diff(inv: &WorkingInventory, i: usize, chapter: u8) -> Vec<Fact> {
    diff_facts(&inv.baseline[i], &inv.spans[i].portions, chapter)
}

/// Diff a chapter-start portion multiset against the post-behavior one into a
/// deterministic set of [`Fact`]s. Net per-`(material, form)` deltas split into
/// sources (net loss) and sinks (net gain); sources pair to sinks greedily in
/// canonical `(material, form)` order. Leftover source ⇒ dissolution (`→ Void`);
/// leftover sink ⇒ deposition (`Void →`).
///
/// **Exact for a behavior whose net effect is a set of edges with distinct
/// endpoints** (the single-edge case this validation covers). The greedy pairing is
/// a deterministic *simplification* of the general minimal-move assignment when many
/// sources and sinks coexist in one chapter — which is exactly why the DIFF is no
/// longer the fact source (§1 refined 2026-07-24): it cannot recover per-agent
/// provenance a multi-edge chapter carries. The recovered facts are stamped
/// [`Cause::Chemical`] as a placeholder — the diff cannot know the cause, which is
/// the whole reason the log supersedes it.
#[cfg(test)]
fn diff_facts(before: &[Portion], after: &[Portion], chapter: u8) -> Vec<Fact> {
    // Net delta per key, in canonical order.
    let mut keys: Vec<(MaterialId, InvForm)> = Vec::new();
    for p in before.iter().chain(after) {
        let k = (p.material, p.form);
        if !keys.contains(&k) {
            keys.push(k);
        }
    }
    keys.sort_unstable();
    let qty = |set: &[Portion], k: (MaterialId, InvForm)| -> f64 {
        set.iter()
            .filter(|p| p.material == k.0 && p.form == k.1)
            .map(|p| p.quantity_m)
            .sum()
    };
    let mut sources: Vec<((MaterialId, InvForm), f64)> = Vec::new();
    let mut sinks: Vec<((MaterialId, InvForm), f64)> = Vec::new();
    for k in keys {
        let d = qty(after, k) - qty(before, k);
        if d < -EPS {
            sources.push((k, -d));
        } else if d > EPS {
            sinks.push((k, d));
        }
    }
    let mut facts = Vec::new();
    let (mut si, mut ki) = (0usize, 0usize);
    while si < sources.len() && ki < sinks.len() {
        let q = sources[si].1.min(sinks[ki].1);
        facts.push(Fact::InPlace {
            chapter,
            cause: Cause::Chemical,
            from: sources[si].0,
            to: sinks[ki].0,
            fraction_m: q,
        });
        sources[si].1 -= q;
        sinks[ki].1 -= q;
        if sources[si].1 <= EPS {
            si += 1;
        }
        if sinks[ki].1 <= EPS {
            ki += 1;
        }
    }
    // Leftover source ⇒ dissolution to the complement (Void).
    for (k, rem) in sources.iter().skip(si) {
        if *rem > EPS {
            facts.push(Fact::InPlace {
                chapter,
                cause: Cause::Chemical,
                from: *k,
                to: (k.0, InvForm::Void),
                fraction_m: *rem,
            });
        }
    }
    // Leftover sink ⇒ deposition from the complement (Void). In-place deposition
    // is unusual; handled for completeness and determinism.
    for (k, rem) in sinks.iter().skip(ki) {
        if *rem > EPS {
            facts.push(Fact::InPlace {
                chapter,
                cause: Cause::Chemical,
                from: (k.0, InvForm::Void),
                to: *k,
                fraction_m: *rem,
            });
        }
    }
    facts
}

impl WorkingInventory {
    /// A granularity-agnostic capability handle over this inventory (§7), **scoped
    /// to a `chapter` and a `cause`** — every edge it applies logs itself with that
    /// scope, so an agent gets its own facts (§1: one fact per agent). Open one ctx
    /// per agent per chapter; [`commit_chapter`] drains the accumulated log.
    #[inline]
    pub fn ctx_for(&mut self, chapter: u8, cause: Cause) -> InvCtx<'_> {
        InvCtx {
            inv: self,
            chapter,
            cause,
        }
    }

    /// Rough heap footprint of the resident inventory (bytes): the span vector plus
    /// each span's portion vector. Excludes the transient commit baseline.
    pub fn footprint_bytes(&self) -> usize {
        self.spans.capacity() * std::mem::size_of::<InvSpan>()
            + self.spans.iter().map(InvSpan::heap_bytes).sum::<usize>()
    }

    /// Total portions across all spans (identity default = one per span).
    pub fn portion_count(&self) -> usize {
        self.spans.iter().map(|s| s.portions.len()).sum()
    }

    /// **The derived surface regolith `H`** (material-behavior.md §13.6, Movement
    /// 2a): the `Loose` above the **topmost `Structure`** — a *positional* query,
    /// NOT a whole-column `Loose` sum. Buried loose / cave fill (Loose below the
    /// first Structure) is **excluded**, the distinction scalar `H` cannot make.
    ///
    /// The scan is top-to-bottom over the column. [`build_working`] lays the record
    /// `Loose` spans first (the mobile cover) and appends the basal bedrock
    /// `Structure` seam LAST, so the Vec order already places every surface-`Loose`
    /// portion **before** the only `Structure` — exactly the loose-then-structure
    /// order the positional rule needs. So over a production inventory this sums the
    /// whole record cover (= `grid.h`) and stops at the bedrock contact; the
    /// cave-exclusion only bites a column that puts `Loose` below a `Structure`
    /// (tested with an explicit column via [`surface_regolith_m`]).
    pub fn derived_regolith_m(&self) -> FracM {
        surface_regolith_m(self.spans.iter().flat_map(|s| s.portions.iter().copied()))
    }

    /// **The derived structural stock `R`** = `Σ Structure` over the whole column
    /// (material-behavior.md §13.6). NOTE (Movement 2a finding): in the two-plane
    /// erosion engine the scalar `R` plane is a bedrock-top **elevation datum**
    /// (signed), not a structural thickness, so `Σ Structure` is the *stock beneath
    /// the surface contact* and the scalar `R` elevation is recovered as
    /// `surf − H` ([`super::field::DeepField::derive_bedrock_at`]), not from this
    /// sum. This is the honest structural-stock query the fully-materialized column
    /// will grow into; today it reads the bedrock seam's (STUB #16) stock.
    pub fn derived_structure_stock_m(&self) -> FracM {
        structure_stock_m(self.spans.iter().flat_map(|s| s.portions.iter().copied()))
    }
}

/// **The positional surface-`H` rule** (material-behavior.md §13.6): given a
/// column's portions in **physical top-to-bottom order** (surface first, basement
/// last), sum the `Loose` down to — and stopping at — the **topmost `Structure`**.
/// Loose that lies *below* the first Structure (cave fill / buried regolith) is
/// **excluded** — the honest surface/subsurface distinction the scalar `H` plane
/// (a single whole-column thickness) structurally cannot represent.
pub fn surface_regolith_m<I: IntoIterator<Item = Portion>>(top_to_bottom: I) -> FracM {
    let mut h = 0.0;
    for p in top_to_bottom {
        match p.form {
            // The topmost Structure ends the surface regolith column; everything
            // below (including buried Loose) is subsurface, excluded from H.
            InvForm::Structure => break,
            InvForm::Loose => h += p.quantity_m,
            // PoreFill/Fluid above the first Structure are not regolith (not Loose)
            // and do not terminate the scan — only Structure marks the contact.
            InvForm::PoreFill | InvForm::Fluid | InvForm::Void => {}
        }
    }
    h
}

/// **`R = Σ Structure`** over a column's portions (order-independent). The
/// structural-stock half of the §13.6 derivation; see
/// [`WorkingInventory::derived_structure_stock_m`] for why the scalar `R`
/// *elevation* is recovered as `surf − H` rather than from this stock.
pub fn structure_stock_m<I: IntoIterator<Item = Portion>>(portions: I) -> FracM {
    portions
        .into_iter()
        .filter(|p| p.form == InvForm::Structure)
        .map(|p| p.quantity_m)
        .sum()
}

/// **The read-modify-write capability** a cellular behavior receives (§7). Indexes
/// by span (granularity-agnostic) and exposes the §3 form-transition / material-
/// change edge as one primitive. No real behavior runs in this spike; the
/// primitive is here and tested so the substrate demonstrably carries the §3/§4
/// RMW the ratified commit diffs into facts.
pub struct InvCtx<'a> {
    inv: &'a mut WorkingInventory,
    chapter: u8,
    cause: Cause,
}

impl InvCtx<'_> {
    /// The number of depth-spans (granularity-agnostic index space).
    #[inline]
    pub fn span_count(&self) -> usize {
        self.inv.spans.len()
    }

    /// Read the `(material, form)` fraction (metres) in span `span` — `0.0` when
    /// absent.
    pub fn fraction(&self, span: usize, material: MaterialId, form: InvForm) -> FracM {
        self.inv.spans[span]
            .portions
            .iter()
            .find(|p| p.material == material && p.form == form)
            .map_or(0.0, |p| p.quantity_m)
    }

    /// **Run a transformation edge** (§3): move `qty` metres from `(from)` to
    /// `(to)` `(material, form)` within span `span`. Material change, form-only
    /// change, dissolution (`to.1 == Void`) and deposition (`from.1 == Void`) all
    /// ride this one call — the exact vocabulary a committed [`Fact`] records.
    /// Mass-conserving except at a `Void` side; clamped so it cannot mint material.
    ///
    /// **Logs the edge it actually moved** with this ctx's scope (`chapter`,
    /// `cause`) — the fact source `commit_chapter` drains (§1). A zero-move edge
    /// (nothing available) logs nothing.
    pub fn apply_edge(
        &mut self,
        span: usize,
        from: (MaterialId, InvForm),
        to: (MaterialId, InvForm),
        qty: FracM,
    ) -> FracM {
        let unit_index = self.inv.spans[span].unit_index;
        let moved = apply_move(&mut self.inv.spans[span].portions, from, to, qty);
        if moved > EPS {
            self.inv.log.push(LoggedEdge {
                unit_index,
                chapter: self.chapter,
                cause: self.cause,
                from,
                to,
                fraction_m: moved,
            });
        }
        moved
    }

    /// A form-only edge (same material) — the common weathering/crumbling case.
    pub fn move_form(
        &mut self,
        span: usize,
        material: MaterialId,
        from: InvForm,
        to: InvForm,
        qty: FracM,
    ) -> FracM {
        self.apply_edge(span, (material, from), (material, to), qty)
    }
}

/// A unit's **provenance**: its immutable depositional base and the facts that
/// have transformed it (DECIDED: "started as X, weathering did Y at chapter Z" is
/// just reading `base + facts`). [`Self::compose`] folds them into the current
/// composition; [`Self::facts`] is the lineage read.
pub struct UnitProvenance<'a> {
    pub base: &'a DepUnit,
    pub facts: &'a [Fact],
}

impl<'a> UnitProvenance<'a> {
    /// Build the provenance view of unit `i` from a record + ledger.
    pub fn of(strata: &'a DeepStrata, ledger: &'a FactLedger, i: usize) -> Option<Self> {
        strata.units.get(i).map(|base| Self {
            base,
            facts: ledger.facts_for(i),
        })
    }

    /// Current composition = `derive(base) then fold(facts)`.
    pub fn compose(&self) -> Vec<Portion> {
        compose_unit(self.base, self.facts)
    }

    /// The lineage: the facts appended to this unit, chapter-ordered.
    pub fn facts(&self) -> &[Fact] {
        self.facts
    }
}

/// **Quantize a fractional metre quantity to eighths** for a collapse voxel of
/// height `voxel_m` — the *one* step where eighths appear (§1 DECIDED). Below this
/// call the deep tier is pure metres; above it, integer eighths.
#[inline]
pub fn quantize_to_eighths(quantity_m: FracM, voxel_m: f64) -> u8 {
    ((quantity_m / voxel_m) * f64::from(VOXEL_EIGHTHS))
        .round()
        .clamp(0.0, f64::from(VOXEL_EIGHTHS)) as u8
}

/// **Collapse the top `voxel_m` of the column into a present `VoxelContents`** — a
/// *demonstration* of the quantize-at-collapse step (not a replacement for the
/// shipped `ColumnFill`, untouched). Confirms the fractional representation closes
/// at exactly one step: the inventory holds sub-eighth metres, eighths are minted
/// here.
pub fn collapse_top_voxel(inv: &WorkingInventory, voxel_m: f64) -> VoxelContents {
    let mut remaining = voxel_m;
    let mut debris: Vec<MaterialId> = Vec::new();
    for span in inv.spans.iter().rev() {
        if remaining <= 0.0 {
            break;
        }
        for p in &span.portions {
            if remaining <= 0.0 {
                break;
            }
            let take = p.quantity_m.min(remaining);
            let eighths = quantize_to_eighths(take, voxel_m);
            for _ in 0..eighths {
                if debris.len() < VOXEL_EIGHTHS as usize {
                    debris.push(p.material);
                }
            }
            remaining -= take;
        }
    }
    VoxelContents::debris_only(&debris[..debris.len().min(VOXEL_EIGHTHS as usize)])
        .unwrap_or(VoxelContents::EMPTY)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deeptime::recorder::{Aridity, DepEnv, DepTag, EnergyBand};

    fn tag(env: DepEnv, energy: EnergyBand) -> DepTag {
        DepTag::mineral(env, Aridity::Humid, energy)
    }

    fn sample_record() -> DeepStrata {
        let mut s = DeepStrata::default();
        s.deposit(tag(DepEnv::Subsea, EnergyBand::Low), 4.3, 0);
        s.deposit(tag(DepEnv::Subaerial, EnergyBand::High), 2.7, 0);
        s.deposit(tag(DepEnv::Subaerial, EnergyBand::Low), 1.1, 0);
        s
    }

    #[test]
    fn identity_default_commit_appends_no_facts_and_leaves_the_record() {
        // The seam is free when empty: build from base+empty-ledger, run NO
        // behavior, commit → zero facts, and composition reproduces the base.
        let strata = sample_record();
        let mut ledger = FactLedger::empty_with_bedrock(&strata);
        let mut inv = build_working(&strata, &ledger);
        commit_chapter(&mut inv, &mut ledger);
        assert!(ledger.is_empty(), "identity default must append no facts");
        for (ui, u) in strata.units.iter().enumerate() {
            assert_eq!(
                compose_unit(u, ledger.facts_for(ui)),
                derive_base(u),
                "base + empty facts == base"
            );
        }
    }

    #[test]
    fn base_composition_matches_the_collapse_tier_routing() {
        let rec = sample_record();
        for u in &rec.units {
            let base = derive_base(u);
            assert_eq!(base.len(), 1);
            assert_eq!(base[0].material, litho_of_tag(u.tag).reference_material());
            assert_eq!(base[0].form, InvForm::Loose);
            assert_eq!(base[0].quantity_m, u.thickness_m);
        }
    }

    #[test]
    fn non_identity_a_known_edge_commits_a_fact_that_re_derives() {
        // The agreement test: a synthetic behavior applies a KNOWN material-change
        // edge; commit turns the delta into a fact; re-deriving base+facts returns
        // the changed composition, and the provenance read returns the fact.
        let strata = {
            let mut s = DeepStrata::default();
            s.deposit(tag(DepEnv::Subaerial, EnergyBand::High), 2.0, 3); // -> SANDSTONE (coarse)
            s
        };
        let mut ledger = FactLedger::empty_with_bedrock(&strata);
        let base_mat = litho_of_tag(strata.units[0].tag).reference_material();
        assert_eq!(base_mat, MaterialId::SANDSTONE);

        // Behavior: 0.5 m of SANDSTONE/Loose -> MUDSTONE/Loose (a material change),
        // attributed to a specific cause.
        let mut inv = build_working(&strata, &ledger);
        let moved = inv.ctx_for(4, Cause::Biotic).apply_edge(
            0,
            (MaterialId::SANDSTONE, InvForm::Loose),
            (MaterialId::MUDSTONE, InvForm::Loose),
            0.5,
        );
        assert_eq!(moved, 0.5);

        // The drained log reconciles with the diff for this single edge (§1
        // validation check): net delta recovered equals the logged fact's move.
        let recon = reconciles_with_diff(&inv, 0, 4);
        assert_eq!(recon.len(), 1);
        assert_eq!(recon[0].from(), (MaterialId::SANDSTONE, InvForm::Loose));
        assert_eq!(recon[0].to(), (MaterialId::MUDSTONE, InvForm::Loose));

        commit_chapter(&mut inv, &mut ledger);

        // One fact appended, with the expected shape — carrying its cause.
        assert_eq!(ledger.total_facts(), 1);
        let f = ledger.facts_for(0)[0];
        assert_eq!(f.chapter(), 4);
        assert_eq!(f.cause(), Cause::Biotic);
        assert_eq!(f.from(), (MaterialId::SANDSTONE, InvForm::Loose));
        assert_eq!(f.to(), (MaterialId::MUDSTONE, InvForm::Loose));
        assert!((f.fraction_m() - 0.5).abs() < 1e-12);
        assert_eq!(f.edge(), (InvForm::Loose, InvForm::Loose));

        // Re-derive base+facts → composition changed as expected.
        let comp = compose_unit(&strata.units[0], ledger.facts_for(0));
        let sand = comp
            .iter()
            .find(|p| p.material == MaterialId::SANDSTONE)
            .unwrap();
        let mud = comp
            .iter()
            .find(|p| p.material == MaterialId::MUDSTONE)
            .unwrap();
        assert!((sand.quantity_m - 1.5).abs() < 1e-12);
        assert!((mud.quantity_m - 0.5).abs() < 1e-12);

        // Provenance read returns the same fact and composition.
        let prov = UnitProvenance::of(&strata, &ledger, 0).unwrap();
        assert_eq!(prov.facts().len(), 1);
        assert_eq!(prov.compose(), comp);
    }

    #[test]
    fn dissolution_edge_commits_a_to_void_fact() {
        // A dissolution behavior removes material to the complement; the fact's
        // destination form is Void, and re-derivation shrinks the column.
        let strata = {
            let mut s = DeepStrata::default();
            s.deposit(tag(DepEnv::Subaerial, EnergyBand::High), 2.0, 0);
            s
        };
        let mut ledger = FactLedger::empty_with_bedrock(&strata);
        let mat = litho_of_tag(strata.units[0].tag).reference_material();
        let mut inv = build_working(&strata, &ledger);
        inv.ctx_for(1, Cause::Dissolution).apply_edge(
            0,
            (mat, InvForm::Loose),
            (mat, InvForm::Void),
            0.75,
        );
        commit_chapter(&mut inv, &mut ledger);
        let f = ledger.facts_for(0)[0];
        assert_eq!(f.to().1, InvForm::Void);
        assert_eq!(f.cause(), Cause::Dissolution);
        let comp = compose_unit(&strata.units[0], ledger.facts_for(0));
        let total: f64 = comp.iter().map(|p| p.quantity_m).sum();
        assert!(
            (total - 1.25).abs() < 1e-12,
            "dissolution shrinks the column"
        );
    }

    #[test]
    fn move_form_is_a_mass_neutral_read_modify_write_edge() {
        let rec = sample_record();
        let ledger = FactLedger::empty_with_bedrock(&rec);
        let mut inv = build_working(&rec, &ledger);
        inv.spans[0].portions.push(Portion {
            material: MaterialId::GRANITE,
            form: InvForm::Structure,
            quantity_m: 1.0,
        });
        let before = inv.spans[0].thickness_m();
        inv.ctx_for(0, Cause::Chemical).move_form(
            0,
            MaterialId::GRANITE,
            InvForm::Structure,
            InvForm::Loose,
            0.4,
        );
        assert_eq!(
            inv.spans[0].thickness_m(),
            before,
            "form change is mass-neutral"
        );
    }

    #[test]
    fn substrate_accommodates_fluid_without_building_it() {
        let rec = sample_record();
        let ledger = FactLedger::empty_with_bedrock(&rec);
        let mut inv = build_working(&rec, &ledger);
        assert_eq!(
            inv.spans
                .iter()
                .flat_map(|s| &s.portions)
                .filter(|p| p.form == InvForm::Fluid)
                .count(),
            0,
            "identity default builds no fluid"
        );
        let mat = inv.spans[0].portions[0].material;
        inv.ctx_for(0, Cause::Chemical)
            .move_form(0, mat, InvForm::Loose, InvForm::Fluid, 0.1);
        assert!(
            inv.ctx_for(0, Cause::Chemical)
                .fraction(0, mat, InvForm::Fluid)
                > 0.0
        );
    }

    #[test]
    fn eighths_appear_only_at_the_quantize_step() {
        let mut s = DeepStrata::default();
        s.deposit(tag(DepEnv::Subaerial, EnergyBand::High), 0.37, 0);
        let inv = build_identity(&s, Granularity::PerStratum);
        assert_eq!(inv.spans[0].portions[0].quantity_m, 0.37);
        assert_eq!(quantize_to_eighths(0.37, 0.9), 3);
        assert_eq!(collapse_top_voxel(&inv, 0.9).solid_eighths(), 3);
    }

    #[test]
    fn derived_regolith_is_the_whole_loose_cover_over_a_production_shaped_inventory() {
        // build_working lays record Loose spans then the basal bedrock Structure
        // seam, so the derived surface H = Σ record Loose (= H the scalar plane
        // tracks), and the structural stock = the bedrock seam thickness.
        let rec = sample_record();
        let ledger = FactLedger::empty_with_bedrock(&rec);
        let inv = build_working(&rec, &ledger);
        let cover: f64 = rec.units.iter().map(|u| u.thickness_m).sum();
        assert!((inv.derived_regolith_m() - cover).abs() < 1e-12);
        assert!((inv.derived_structure_stock_m() - BEDROCK_SEAM_THICKNESS_M).abs() < 1e-12);
    }

    #[test]
    fn buried_loose_below_a_structure_is_excluded_from_surface_h() {
        // The positional/cave rule (material-behavior.md §13.6): a column with a
        // Structure roof over buried Loose (cave fill) — the derived surface H is
        // ONLY the loose above the roof, though the whole-column Loose sum (what a
        // scalar H plane would report) is larger.
        let m = MaterialId::GRANITE;
        let s = MaterialId::SANDSTONE;
        let column = [
            Portion {
                material: m,
                form: InvForm::Loose,
                quantity_m: 2.0,
            }, // surface regolith
            Portion {
                material: m,
                form: InvForm::Structure,
                quantity_m: 5.0,
            }, // roof
            Portion {
                material: s,
                form: InvForm::Loose,
                quantity_m: 3.0,
            }, // buried / cave fill
        ];
        assert!((surface_regolith_m(column) - 2.0).abs() < 1e-12);
        // The naive whole-column loose sum (the scalar plane's blind answer) is 5.0.
        let whole: f64 = column
            .iter()
            .filter(|p| p.form == InvForm::Loose)
            .map(|p| p.quantity_m)
            .sum();
        assert!((whole - 5.0).abs() < 1e-12);
        // Structure stock is order-independent.
        assert!((structure_stock_m(column) - 5.0).abs() < 1e-12);
    }

    // --- The CSR layout (journal/0100) ---------------------------------------

    #[test]
    fn fact_order_within_a_slot_is_preserved_across_interleaved_slots() {
        // **The order trap.** `commit_chapter` merges an edge into the EARLIEST
        // matching fact in its slot, and `compose_unit` folds facts in order — so
        // per-slot order is observable, and a flat array that groups slots must
        // preserve it exactly. Here two slots interleave in the log and one cause
        // repeats non-consecutively; the merge must still land on the first
        // chemical fact of that slot, and the biotic fact must stay behind it.
        let strata = sample_record();
        let mut ledger = FactLedger::empty_with_bedrock(&strata);
        let mut inv = build_working(&strata, &ledger);
        let m0 = inv.spans[0].portions[0].material;
        let m1 = inv.spans[1].portions[0].material;
        let f = InvForm::Fluid;
        let l = InvForm::Loose;
        inv.ctx_for(0, Cause::Chemical).move_form(0, m0, l, f, 0.1);
        inv.ctx_for(0, Cause::Chemical).move_form(1, m1, l, f, 0.2);
        inv.ctx_for(0, Cause::Biotic).move_form(0, m0, l, f, 0.3);
        inv.ctx_for(0, Cause::Frost).move_form(1, m1, l, f, 0.4);
        inv.ctx_for(0, Cause::Chemical).move_form(0, m0, l, f, 0.5);
        commit_chapter(&mut inv, &mut ledger);

        let s0 = ledger.facts_for(0);
        assert_eq!(s0.len(), 2, "one fact per cause on slot 0");
        assert_eq!(
            s0[0].cause(),
            Cause::Chemical,
            "first-committed cause first"
        );
        assert!(
            (s0[0].fraction_m() - 0.6).abs() < 1e-12,
            "the later chemical edge merged into the EARLIEST chemical fact"
        );
        assert_eq!(s0[1].cause(), Cause::Biotic);
        assert!((s0[1].fraction_m() - 0.3).abs() < 1e-12);

        let s1 = ledger.facts_for(1);
        assert_eq!(s1.len(), 2, "slot 1's run is untouched by slot 0's growth");
        assert_eq!(s1[0].cause(), Cause::Chemical);
        assert!((s1[0].fraction_m() - 0.2).abs() < 1e-12);
        assert_eq!(s1[1].cause(), Cause::Frost);
        assert!((s1[1].fraction_m() - 0.4).abs() < 1e-12);

        // Untouched slots read empty, in range or out of it.
        assert!(ledger.facts_for(2).is_empty());
        assert!(ledger.facts_for(9_999).is_empty());
        assert_eq!(ledger.total_facts(), 4);
        assert_eq!(ledger.slots_with_facts(), 2);
    }

    #[test]
    fn the_ledger_costs_its_facts_not_its_slots() {
        // **The measured defect this layout exists to kill** (journal/0100, S19 §3b):
        // `Vec<Vec<Fact>>` paid a 24-byte header per slot whether or not the slot
        // carried a fact, and 98.8 % of production slots carry none — 89 % of the
        // ledger's heap was the absence of facts. A BOUND, not a snapshot: the cost
        // of a ledger must be a function of its FACTS, not of its record's depth.
        let mut deep = DeepStrata::default();
        for i in 0..10_000 {
            let energy = if i % 2 == 0 {
                EnergyBand::High
            } else {
                EnergyBand::Low
            };
            deep.deposit(tag(DepEnv::Subaerial, energy), 0.5, 0);
        }
        assert_eq!(deep.units.len(), 10_000, "alternating tags never merge");
        let old_shape = (deep.units.len() + 1) * std::mem::size_of::<Vec<Fact>>();

        // An unweathered cell — the 75.8 % case — allocates NOTHING.
        let mut ledger = FactLedger::empty_with_bedrock(&deep);
        assert!(ledger.is_empty());
        assert_eq!(
            ledger.footprint_bytes(),
            0,
            "an unweathered ledger costs zero bytes ({old_shape} B under the old shape)"
        );

        // One weathered bedrock seam: three agent facts and ONE index row.
        let mut inv = build_working(&deep, &ledger);
        let bedrock = inv.spans.len() - 1;
        for &cause in &[Cause::Chemical, Cause::Biotic, Cause::Frost] {
            inv.ctx_for(0, cause).move_form(
                bedrock,
                BEDROCK_SEAM_MATERIAL,
                InvForm::Structure,
                InvForm::Loose,
                0.25,
            );
        }
        commit_chapter(&mut inv, &mut ledger);
        ledger.compact();
        assert_eq!(ledger.total_facts(), 3);
        assert_eq!(ledger.slots_with_facts(), 1);
        assert_eq!(ledger.facts_for(deep.units.len()).len(), 3);
        // The bound: facts + one row, and nothing that scales with 10,000 slots.
        let expect = 3 * std::mem::size_of::<Fact>() + std::mem::size_of::<u64>();
        assert_eq!(ledger.payload_bytes(), 3 * std::mem::size_of::<Fact>());
        assert!(
            ledger.footprint_bytes() <= expect,
            "exact-sized: {} B > {expect} B",
            ledger.footprint_bytes()
        );
        assert!(
            ledger.footprint_bytes() * 1_000 < old_shape,
            "cost must not scale with slot count: {} B vs {old_shape} B of old headers",
            ledger.footprint_bytes()
        );
    }

    #[test]
    fn rekeying_moves_a_run_and_leaves_it_exact_sized() {
        // The finalize step: the accumulator keys bedrock at the stable sentinel
        // slot 0, the consumer reads it at the final record's bedrock index. The
        // facts — and their ORDER — must survive the move untouched, and the
        // result must carry no growth slack.
        let empty = DeepStrata::default();
        let mut acc = FactLedger::empty_with_bedrock(&empty);
        let mut inv = build_working(&empty, &acc);
        for &cause in &[Cause::Chemical, Cause::Biotic, Cause::Frost] {
            inv.ctx_for(0, cause).move_form(
                0,
                BEDROCK_SEAM_MATERIAL,
                InvForm::Structure,
                InvForm::Loose,
                0.25,
            );
        }
        commit_chapter(&mut inv, &mut acc);
        let before: Vec<Fact> = acc.facts_for(0).to_vec();
        assert_eq!(before.len(), 3);

        let moved = acc.rekeyed(0, 7);
        assert!(
            moved.facts_for(0).is_empty(),
            "the sentinel slot is vacated"
        );
        assert_eq!(moved.facts_for(7), &before[..], "same facts, same order");
        assert_eq!(
            moved.footprint_bytes(),
            3 * std::mem::size_of::<Fact>() + std::mem::size_of::<u64>(),
            "exact-sized, no doubling slack"
        );
        // An empty accumulator re-keys to a free ledger.
        let none = FactLedger::default().rekeyed(0, 7);
        assert!(none.is_empty());
        assert_eq!(none.footprint_bytes(), 0);
    }

    // --- The grid-wide record (journal/0102) ---------------------------------

    /// Build a bedrock-only accumulator carrying `causes.len()` facts at slot 0.
    fn accumulator_with(causes: &[Cause], qty: FracM) -> FactLedger {
        let empty = DeepStrata::default();
        let mut acc = FactLedger::empty_with_bedrock(&empty);
        let mut inv = build_working(&empty, &acc);
        for &cause in causes {
            inv.ctx_for(0, cause).move_form(
                0,
                BEDROCK_SEAM_MATERIAL,
                InvForm::Structure,
                InvForm::Loose,
                qty,
            );
        }
        commit_chapter(&mut inv, &mut acc);
        acc
    }

    #[test]
    fn the_grid_record_reproduces_every_cell_fact_for_fact_and_in_order() {
        // **The byte-identity claim in its most direct form.** A grid of cells, most
        // of them empty (the production shape), each re-keyed onto a DIFFERENT slot —
        // and every cell's facts, their order and their slot must survive the
        // flattening exactly as the per-cell `rekeyed` produced them.
        let accs = vec![
            accumulator_with(&[], 0.0),
            accumulator_with(&[Cause::Chemical, Cause::Biotic, Cause::Frost], 0.25),
            accumulator_with(&[], 0.0),
            accumulator_with(&[Cause::Frost, Cause::Chemical], 0.5),
            accumulator_with(&[], 0.0),
        ];
        // Distinct per-cell bedrock slots, exactly as records of different depths give.
        let to_slot = |i: usize| i * 3 + 1;
        let field = LedgerField::from_accumulators(&accs, 0, to_slot);
        assert_eq!(field.len(), accs.len());

        for (i, acc) in accs.iter().enumerate() {
            let want = acc.rekeyed(0, to_slot(i));
            let got = field.get(i).expect("cell in range");
            assert_eq!(
                got.facts_for(to_slot(i)),
                want.facts_for(to_slot(i)),
                "cell {i}: same facts, same ORDER"
            );
            assert_eq!(
                got.total_facts(),
                want.total_facts(),
                "cell {i}: fact count"
            );
            assert_eq!(got.slots_with_facts(), want.slots_with_facts());
            assert_eq!(got.is_empty(), want.is_empty());
            // The sentinel slot is vacated and no other slot answers.
            assert!(got.facts_for(0).is_empty() || to_slot(i) == 0);
            assert!(got.facts_for(9_999).is_empty());
        }
        assert_eq!(field.total_facts(), 5, "3 + 2 facts across the grid");
        assert_eq!(field.slots_with_facts(), 2, "two cells carry a row");
        assert!(field.get(accs.len()).is_none(), "out of range");
    }

    #[test]
    fn a_cells_run_ends_at_its_own_boundary_not_the_next_cells() {
        // The CSR trap one level up: `start` is absolute and a row's end is the NEXT
        // row's start, so a cell whose last row is followed by another CELL's row must
        // still stop at its own boundary. Two adjacent non-empty cells is the case
        // that catches an off-by-one here.
        let accs = vec![
            accumulator_with(&[Cause::Chemical], 0.25),
            accumulator_with(&[Cause::Biotic, Cause::Frost], 0.5),
        ];
        let field = LedgerField::from_accumulators(&accs, 0, |_| 7);
        let c0 = field.get(0).unwrap();
        let c1 = field.get(1).unwrap();
        assert_eq!(c0.facts_for(7).len(), 1, "cell 0 does not swallow cell 1");
        assert_eq!(c0.facts_for(7)[0].cause(), Cause::Chemical);
        assert_eq!(c1.facts_for(7).len(), 2);
        assert_eq!(c1.facts_for(7)[0].cause(), Cause::Biotic);
        assert_eq!(c1.facts_for(7)[1].cause(), Cause::Frost);
        assert_eq!(field.total_facts(), 3);
    }

    #[test]
    fn the_record_costs_its_facts_not_its_cells() {
        // **The measured defect journal/0102 exists to kill**, stated as a BOUND. The
        // per-cell owning container cost 48 B × 297,025 = 13.60 MiB before a single
        // fact was stored, in a field where 75.8 % of cells never weather. A
        // grid-wide record must cost its facts plus 4 B/cell of dense offsets, and
        // NOTHING that scales with the per-cell struct.
        const CELLS: usize = 100_000;
        let mut accs = vec![FactLedger::default(); CELLS];
        accs[7] = accumulator_with(&[Cause::Chemical, Cause::Biotic, Cause::Frost], 0.25);
        let field = LedgerField::from_accumulators(&accs, 0, |_| 3);

        let old_shape = CELLS * std::mem::size_of::<FactLedger>();
        let want = 3 * std::mem::size_of::<Fact>()          // the facts
            + std::mem::size_of::<u64>()                    // one SlotRun
            + (CELLS + 1) * std::mem::size_of::<u32>(); // the dense cell offsets
        assert_eq!(field.total_facts(), 3);
        assert_eq!(field.slots_with_facts(), 1);
        assert_eq!(field.payload_bytes(), 3 * std::mem::size_of::<Fact>());
        assert_eq!(
            field.footprint_bytes(),
            want,
            "exact-sized: facts + one row + 4 B/cell, and nothing else \
             ({old_shape} B of per-cell structs under the old shape)"
        );
        // The dense half is the only per-cell term, and it is 12x smaller.
        assert!(
            field.footprint_bytes() * 4 < old_shape,
            "{} B is not a fraction of {old_shape} B",
            field.footprint_bytes()
        );
        // An empty grid costs literally nothing (the flag-off identity default).
        let off = LedgerField::default();
        assert!(off.is_empty());
        assert_eq!(off.len(), 0);
        assert_eq!(off.footprint_bytes(), 0);
        assert!(off.get(0).is_none());
    }

    #[test]
    fn sizes_are_pinned() {
        assert_eq!(std::mem::size_of::<Portion>(), 16);
        assert!(std::mem::size_of::<Fact>() <= 24);
        // The CSR index row: two u32s. Residency is sacred and this one is paid
        // per non-empty slot across millions of cells.
        assert_eq!(std::mem::size_of::<SlotRun>(), 8);
        let rec = sample_record();
        let ps = build_identity(&rec, Granularity::PerStratum);
        let pv = build_identity(&rec, Granularity::PerVoxel { voxel_m: 0.9 });
        assert!(pv.footprint_bytes() >= ps.footprint_bytes());
    }
}
