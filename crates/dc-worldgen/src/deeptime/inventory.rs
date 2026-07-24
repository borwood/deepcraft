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
#[derive(Clone, Default, PartialEq, Debug)]
pub struct FactLedger {
    /// `facts[i]` = the facts appended to `units[i]`, chapter-ordered.
    pub facts: Vec<Vec<Fact>>,
}

impl FactLedger {
    /// An empty ledger sized to a record's unit count (the identity default: every
    /// unit starts with zero facts, so `base + facts` == `base`).
    pub fn empty_for(strata: &DeepStrata) -> Self {
        Self {
            facts: vec![Vec::new(); strata.units.len()],
        }
    }

    /// An empty ledger sized to a record's unit count **plus one slot for the
    /// bedrock `Structure` seam** (STUB #16, at index `units.len()` — the LAST
    /// slot). This is the ledger [`build_working`] and the weathering pass expect,
    /// so the `Structure→Loose` facts have somewhere to live.
    pub fn empty_with_bedrock(strata: &DeepStrata) -> Self {
        Self {
            facts: vec![Vec::new(); strata.units.len() + 1],
        }
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

    /// The facts appended to unit `i` (empty slice when none / out of range).
    #[inline]
    pub fn facts_for(&self, i: usize) -> &[Fact] {
        self.facts.get(i).map_or(&[], Vec::as_slice)
    }

    /// True when no fact has been committed anywhere — the identity-default state.
    pub fn is_empty(&self) -> bool {
        self.facts.iter().all(Vec::is_empty)
    }

    /// Total facts across all units (a size metric).
    pub fn total_facts(&self) -> usize {
        self.facts.iter().map(Vec::len).sum()
    }

    /// Rough heap footprint (bytes): the outer vector plus every inner fact vector.
    pub fn footprint_bytes(&self) -> usize {
        self.facts.capacity() * std::mem::size_of::<Vec<Fact>>()
            + self
                .facts
                .iter()
                .map(|v| v.capacity() * std::mem::size_of::<Fact>())
                .sum::<usize>()
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
/// **STUB #16**, a made-up depth chosen only to be an effectively-inexhaustible
/// `Structure→Loose` source over the one chapter this slice weathers. The heir
/// (a genesis/emplacement pass) supplies a real per-column unroofing depth.
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
    // Coalesce identical successive logged edges into one fact each.
    for e in inv.log.drain(..) {
        let slot = e.unit_index;
        if ledger.facts.len() <= slot {
            ledger.facts.resize(slot + 1, Vec::new());
        }
        let facts = &mut ledger.facts[slot];
        if let Some(Fact::InPlace {
            chapter,
            cause,
            from,
            to,
            fraction_m,
        }) = facts.last_mut()
            && *chapter == e.chapter
            && *cause == e.cause
            && *from == e.from
            && *to == e.to
        {
            *fraction_m += e.fraction_m;
            continue;
        }
        facts.push(Fact::InPlace {
            chapter: e.chapter,
            cause: e.cause,
            from: e.from,
            to: e.to,
            fraction_m: e.fraction_m,
        });
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

    #[test]
    fn sizes_are_pinned() {
        assert_eq!(std::mem::size_of::<Portion>(), 16);
        assert!(std::mem::size_of::<Fact>() <= 24);
        let rec = sample_record();
        let ps = build_identity(&rec, Granularity::PerStratum);
        let pv = build_identity(&rec, Granularity::PerVoxel { voxel_m: 0.9 });
        assert!(pv.footprint_bytes() >= ps.footprint_bytes());
    }
}
