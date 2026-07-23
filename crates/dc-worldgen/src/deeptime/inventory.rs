//! **S17 — the deep cell's mutable working material inventory (spike).**
//!
//! This module de-risks `docs/design/material-behavior.md` §1: today a deep cell
//! stores `R`/`H` heights plus an append-only strata record (its *history*), so a
//! behavior like "weathering consumes bedrock, produces regolith" has **nothing
//! per-cell to read-modify-write**. S16 (`weather_behavior.rs`) had to thin-adapt
//! one over the height planes; this module builds the real substrate the north
//! star names: a **stack of `VoxelContents`-shaped spans indexed by depth**, that
//!
//! - behaviors **read-modify-write** per `(MaterialId, Form)` fraction (§4/§6);
//! - a **chapter boundary commits back into the strata record** — the
//!   deeptime-as-compiler step (§1, §8);
//! - the present voxel still **derives** from record + inventory (§1).
//!
//! **Two roles, cleanly separated** (spines S-2, S-9):
//! - the **strata record stays the temporal authority** (append-only history —
//!   the committed facts);
//! - the **working inventory is the mutable current composition** (fluid state a
//!   behavior mutates freely) — *store only what derivation cannot predict*.
//!
//! The whole point, as an instrument (the deep-sim-flags pattern): wired with an
//! **identity default** (build from the record, run *no* behavior, commit back),
//! the record round-trips **byte-identically** — so the seam is provably free when
//! empty. Where it strains is the finding, not a defect.
//!
//! **Granularity-agnostic on purpose** (§10): a [`Granularity`] selects
//! per-stratum (one span per record unit) or per-voxel (spans no thicker than a
//! collapse voxel), and [`InvCtx`] — the capability a behavior holds — indexes by
//! span identically for either, so the shape is not accidentally deeptime-only.

use dc_core::{MaterialId, VOXEL_EIGHTHS, VoxelContents};

use super::lithology::litho_of_tag;
use super::recorder::{DeepStrata, DepTag, DepUnit};

/// A **finer-than-eighth fractional quantity at depth**, in metres — the deep
/// tier's native unit (`H` is metres). This is the ratified representation
/// (material-behavior.md §1 DECIDED 2026-07-23: deep spans carry sub-eighth
/// fractional quantities; *"fractions come only from the ledger"*). Eighths
/// appear **only at collapse** ([`quantize_to_eighths`]).
pub type FracM = f64;

/// The **form** a material-portion occupies volume in — the deep-tier view of the
/// machine's closed form set (material-behavior.md §2). `Void` is deliberately
/// **absent**: it is the unoccupied complement, never a stored role (§2, §10
/// RESOLVED). `Fluid` is present so the substrate *accommodates* bound water, but
/// the identity default never populates it and this spike builds no water (§10 —
/// bound water is derived per the water model, S-2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum InvForm {
    /// Coherent, load-bearing framework (the `R`/structural stock).
    Structure,
    /// Granular, unreserved volume that obeys gravity (the `H`/regolith stock).
    Loose,
    /// Material held inside another's reserved-but-unfilled pore capacity.
    PoreFill,
    /// Liquid in pores + open space. **Accommodated, never stored by the identity
    /// default** — the substrate has a slot for it; the water model derives it.
    Fluid,
}

/// A `(MaterialId, Form)` **portion** with a fractional metre quantity — the
/// exact read-modify-write target §4/§6 name ("a behavior's RMW targets a
/// `(MaterialId, Form)` fraction drawn from the multiset").
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Portion {
    pub material: MaterialId,
    pub form: InvForm,
    /// Finer-than-eighth quantity, metres. Never quantized until collapse.
    pub quantity_m: FracM,
}

/// One **depth-span** of the working inventory: a `VoxelContents`-shaped mixture
/// (a multiset of [`Portion`]s) over a depth interval. The working analogue of a
/// [`DepUnit`] (per-stratum) or a collapse voxel (per-voxel).
///
/// Carries the **provenance** the chapter-commit needs to fold back into the
/// record: the depositional tag/chapter/unconformity of the source unit. (The
/// record is tagged by *depositional environment*, not material — `litho_of_tag`
/// is many-to-one — so a faithful commit must preserve the source tag, not
/// re-derive it from the material.)
#[derive(Clone, PartialEq, Debug)]
pub struct InvSpan {
    /// The depositional tag of the record unit this span derives from.
    pub tag: DepTag,
    /// The tectonic chapter of the source unit.
    pub chapter: u8,
    /// Whether this span opens on an erosional surface (an unconformity).
    pub unconformity: bool,
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

/// The vertical resolution of the working inventory (material-behavior.md §10 —
/// *a measurement call*, closed by `docs/spikes/S17`). Kept as an explicit choice
/// so the substrate is not accidentally locked to one tier.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Granularity {
    /// One span per record unit — variable thickness, follows the tag changes.
    /// **Cheaper** (span count = unit count) and span↔unit is 1:1, so the commit
    /// is exact (no float re-accumulation). The recommended shape (S17 results).
    PerStratum,
    /// Spans no thicker than a collapse voxel — a unit is subdivided into
    /// `ceil(thickness / voxel_m)` slices, each still tagged to its parent unit.
    /// Costs more spans (and re-merges at commit under float addition).
    PerVoxel { voxel_m: f64 },
}

/// A deep cell's **mutable working material inventory**: a stack of spans indexed
/// by depth (bottom-up, matching `DeepStrata.units`), plus the basement material
/// below the recorded column.
#[derive(Clone, PartialEq, Debug)]
pub struct WorkingInventory {
    /// Spans bottom-up: `spans[0]` is the deepest recorded stratum, the last span
    /// ends at the surface. Below is unrecorded basement (`R`).
    pub spans: Vec<InvSpan>,
    /// The basement material below the record. Derived (at real collapse from the
    /// tectonic province); here the reference basement, and **not committed** to
    /// the record — the record only ever held the `H` column, so the basement is
    /// out of the byte-identity round-trip.
    pub basement: MaterialId,
    /// The granularity this inventory was built at.
    pub granularity: Granularity,
}

/// **The identity default** (the deep-sim-flags pattern): build a working
/// inventory from a strata record where every span carries exactly the material
/// its source unit's tag resolves to (`litho_of_tag(...).reference_material()` —
/// the *same* routing the collapse tier's `deep_class` uses, so the material is
/// the one the world already builds), all in the `Loose` form (the record mirrors
/// the loose `H` cover). No behavior transforms anything.
///
/// Committed straight back with [`commit_chapter`], this reproduces the record
/// byte-identically at [`Granularity::PerStratum`] — the proof the seam is free
/// when empty.
pub fn build_identity(strata: &DeepStrata, granularity: Granularity) -> WorkingInventory {
    let mut spans = Vec::new();
    for u in &strata.units {
        let material = litho_of_tag(u.tag).reference_material();
        match granularity {
            Granularity::PerStratum => spans.push(InvSpan {
                tag: u.tag,
                chapter: u.chapter,
                unconformity: u.unconformity,
                portions: vec![Portion {
                    material,
                    form: InvForm::Loose,
                    quantity_m: u.thickness_m,
                }],
            }),
            Granularity::PerVoxel { voxel_m } => {
                // Subdivide the unit into <= voxel_m slices. Only the first slice
                // inherits the unit's unconformity flag; interior slices are
                // conformable *within* the unit, so the commit re-merges them.
                let mut consumed = 0.0f64;
                let mut first = true;
                while consumed < u.thickness_m {
                    let t = (u.thickness_m - consumed).min(voxel_m);
                    spans.push(InvSpan {
                        tag: u.tag,
                        chapter: u.chapter,
                        unconformity: u.unconformity && first,
                        portions: vec![Portion {
                            material,
                            form: InvForm::Loose,
                            quantity_m: t,
                        }],
                    });
                    consumed += t;
                    first = false;
                }
            }
        }
    }
    WorkingInventory {
        spans,
        basement: MaterialId::GRANITE,
        granularity,
    }
}

/// **The chapter-commit — the deeptime-as-compiler step** (§1, §8). Fold the
/// working inventory's spans back into the strata record: reconstruct `units`
/// from the spans, run-length-merging consecutive spans that share tag+chapter
/// and do not open an unconformity — *exactly* the recorder's own merge rule
/// (`DeepStrata::deposit`), so per-voxel subdivisions of one unit re-coalesce.
///
/// With the identity default (no behavior ran) the rebuilt units equal the input
/// units, so the whole `DeepStrata` is byte-identical (the private `stripped`
/// flag and `strips` counter are left untouched, and `units` is public).
///
/// A real behavior would have changed portion quantities/forms/materials before
/// this call; the commit is where those working deltas become committed history.
pub fn commit_chapter(inv: &WorkingInventory, strata: &mut DeepStrata) {
    let mut units: Vec<DepUnit> = Vec::new();
    for span in &inv.spans {
        let t = span.thickness_m();
        if t <= 0.0 {
            continue;
        }
        if let Some(last) = units.last_mut()
            && last.tag == span.tag
            && last.chapter == span.chapter
            && !span.unconformity
        {
            last.thickness_m += t;
            continue;
        }
        units.push(DepUnit {
            tag: span.tag,
            thickness_m: t,
            unconformity: span.unconformity,
            chapter: span.chapter,
        });
    }
    strata.units = units;
}

impl WorkingInventory {
    /// A granularity-agnostic capability handle over this inventory (§7): the
    /// surface a cellular behavior read-modify-writes through.
    #[inline]
    pub fn ctx(&mut self) -> InvCtx<'_> {
        InvCtx { inv: self }
    }

    /// Rough heap footprint of the whole inventory (bytes): the span vector plus
    /// every span's portion vector. The "what carrying this per cell would cost"
    /// number the granularity measurement reports.
    pub fn footprint_bytes(&self) -> usize {
        self.spans.capacity() * std::mem::size_of::<InvSpan>()
            + self.spans.iter().map(InvSpan::heap_bytes).sum::<usize>()
    }

    /// Total portions across all spans (a size metric; identity default = one per
    /// span).
    pub fn portion_count(&self) -> usize {
        self.spans.iter().map(|s| s.portions.len()).sum()
    }
}

/// **The read-modify-write capability** a cellular behavior receives (§7 — "`ctx`
/// is a capability, not a god-object"). Wraps a mutable inventory; a behavior
/// reads `(material, form)` fractions and requests **form-transition edges**
/// (§3), identically whether the underlying spans are per-stratum or per-voxel.
///
/// No behavior runs in this spike (the identity default), but the primitive is
/// here — and tested — so the substrate demonstrably carries the §3 edges and the
/// §4 `(MaterialId, Form)`-fraction RMW the north star's cellular passes need.
pub struct InvCtx<'a> {
    inv: &'a mut WorkingInventory,
}

impl InvCtx<'_> {
    /// The number of depth-spans (granularity-agnostic index space).
    #[inline]
    pub fn span_count(&self) -> usize {
        self.inv.spans.len()
    }

    /// Read the `(material, form)` fraction (metres) in span `span` — `0.0` when
    /// absent. The pure read side §6 opens with.
    pub fn fraction(&self, span: usize, material: MaterialId, form: InvForm) -> FracM {
        self.inv.spans[span]
            .portions
            .iter()
            .find(|p| p.material == material && p.form == form)
            .map_or(0.0, |p| p.quantity_m)
    }

    /// **Run a form-transition edge** (§3): move `qty` metres of `material` from
    /// `from` to `to` within span `span`, mass-neutral. This is the read side +
    /// the write side of the §6 shape in one primitive — the move every cellular
    /// behavior (weathering `Structure→Loose`, cementation `Loose→Structure`, …)
    /// is built from. Clamps to the available quantity so an over-large request
    /// cannot mint material.
    pub fn move_form(
        &mut self,
        span: usize,
        material: MaterialId,
        from: InvForm,
        to: InvForm,
        qty: FracM,
    ) {
        let portions = &mut self.inv.spans[span].portions;
        let avail = portions
            .iter()
            .find(|p| p.material == material && p.form == from)
            .map_or(0.0, |p| p.quantity_m);
        let q = qty.min(avail).max(0.0);
        if q <= 0.0 {
            return;
        }
        for p in portions.iter_mut() {
            if p.material == material && p.form == from {
                p.quantity_m -= q;
            }
        }
        portions.retain(|p| p.quantity_m > 0.0);
        match portions
            .iter_mut()
            .find(|p| p.material == material && p.form == to)
        {
            Some(p) => p.quantity_m += q,
            None => self.inv.spans[span].portions.push(Portion {
                material,
                form: to,
                quantity_m: q,
            }),
        }
    }
}

/// **Quantize a fractional metre quantity to eighths** for a collapse voxel of
/// height `voxel_m` — the *one* step where eighths appear (§1 DECIDED: deep spans
/// are sub-eighth fractional; they quantize to eighths **only at collapse**).
/// Below this call the deep tier is pure metres; above it, integer eighths.
#[inline]
pub fn quantize_to_eighths(quantity_m: FracM, voxel_m: f64) -> u8 {
    ((quantity_m / voxel_m) * f64::from(VOXEL_EIGHTHS))
        .round()
        .clamp(0.0, f64::from(VOXEL_EIGHTHS)) as u8
}

/// **Collapse the top `voxel_m` of the column into a present `VoxelContents`** — a
/// *demonstration* of the quantize-at-collapse step (not a replacement for the
/// shipped `ColumnFill`, which the spike does not touch). Walks spans from the
/// surface down, accumulating up to `voxel_m` of material, and quantizes each
/// `Loose` portion's metres to debris eighths.
///
/// This confirms the fractional representation closes at exactly one step: the
/// inventory holds sub-eighth metre quantities, and eighths are minted **here**,
/// nowhere earlier.
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
    // Loose portions become debris (the `Loose` form is the debris role, §2).
    // A demonstration only, so cap at a legal open voxel.
    VoxelContents::debris_only(&debris[..debris.len().min(VOXEL_EIGHTHS as usize)])
        .unwrap_or(VoxelContents::EMPTY)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deeptime::recorder::{Aridity, DepEnv, EnergyBand};

    fn tag(env: DepEnv, energy: EnergyBand) -> DepTag {
        DepTag::mineral(env, Aridity::Humid, energy)
    }

    /// A hand-built record with three units of distinct thicknesses and tags —
    /// the fixture the round-trip proofs run on.
    fn sample_record() -> DeepStrata {
        let mut s = DeepStrata::default();
        s.deposit(tag(DepEnv::Subsea, EnergyBand::Low), 4.3, 0);
        s.deposit(tag(DepEnv::Subaerial, EnergyBand::High), 2.7, 0);
        s.deposit(tag(DepEnv::Subaerial, EnergyBand::Low), 1.1, 0);
        s
    }

    #[test]
    fn identity_default_per_stratum_round_trips_byte_identically() {
        // The core proof: record -> inventory (identity, no behavior) -> commit
        // reproduces the record BYTE-IDENTICALLY. The seam is free when empty.
        let original = sample_record();
        let inv = build_identity(&original, Granularity::PerStratum);
        let mut committed = original.clone();
        commit_chapter(&inv, &mut committed);
        assert_eq!(
            committed, original,
            "identity-default per-stratum commit must reproduce the record byte-identically"
        );
        // One span per unit, one portion per span (the identity default shape).
        assert_eq!(inv.spans.len(), original.units.len());
        assert_eq!(inv.portion_count(), original.units.len());
    }

    #[test]
    fn per_stratum_span_material_matches_the_collapse_tier_routing() {
        // The span material is the SAME one the collapse tier's deep_class routes
        // (both go through litho_of_tag) — the inventory builds the rock the world
        // already builds, not a parallel guess.
        let rec = sample_record();
        let inv = build_identity(&rec, Granularity::PerStratum);
        for (span, u) in inv.spans.iter().zip(&rec.units) {
            let expected = litho_of_tag(u.tag).reference_material();
            assert_eq!(span.portions.len(), 1);
            assert_eq!(span.portions[0].material, expected);
            assert_eq!(span.portions[0].form, InvForm::Loose);
        }
    }

    #[test]
    fn per_voxel_round_trips_within_floating_point_and_preserves_tags() {
        // Per-voxel subdivides units into <= voxel_m slices; the commit re-merges
        // them. Thickness re-accumulates under float addition, so the round-trip
        // is exact-up-to-fp (a second strike against per-voxel — see S17 results),
        // but tags/chapters/unit COUNT are preserved exactly.
        let original = sample_record();
        let inv = build_identity(&original, Granularity::PerVoxel { voxel_m: 0.9 });
        assert!(
            inv.spans.len() > original.units.len(),
            "per-voxel must have more spans than units"
        );
        let mut committed = original.clone();
        commit_chapter(&inv, &mut committed);
        assert_eq!(
            committed.units.len(),
            original.units.len(),
            "re-merged unit count must match"
        );
        for (a, b) in committed.units.iter().zip(&original.units) {
            assert_eq!(a.tag, b.tag);
            assert_eq!(a.chapter, b.chapter);
            assert_eq!(a.unconformity, b.unconformity);
            assert!(
                (a.thickness_m - b.thickness_m).abs() < 1e-9,
                "thickness must re-accumulate to within fp tolerance"
            );
        }
    }

    #[test]
    fn move_form_is_a_mass_neutral_read_modify_write_edge() {
        // The §3 form-transition edge / §6 RMW primitive: a weathering-shaped
        // Structure->Loose move conserves mass within the span.
        let rec = sample_record();
        let mut inv = build_identity(&rec, Granularity::PerStratum);
        // Seed a structural portion so there is something to weather.
        inv.spans[0].portions.push(Portion {
            material: MaterialId::GRANITE,
            form: InvForm::Structure,
            quantity_m: 1.0,
        });
        let before = inv.spans[0].thickness_m();
        let mut ctx = inv.ctx();
        ctx.move_form(
            0,
            MaterialId::GRANITE,
            InvForm::Structure,
            InvForm::Loose,
            0.4,
        );
        assert_eq!(
            inv.spans[0].thickness_m(),
            before,
            "form change is mass-neutral within the span"
        );
        assert!(
            (inv.ctx()
                .fraction(0, MaterialId::GRANITE, InvForm::Structure)
                - 0.6)
                .abs()
                < 1e-12
        );
        assert!((inv.ctx().fraction(0, MaterialId::GRANITE, InvForm::Loose) - 0.4).abs() < 1e-12);
    }

    #[test]
    fn substrate_accommodates_fluid_without_the_identity_default_building_it() {
        // §10 open question #4: the substrate ACCOMMODATES a fluid form (move_form
        // can target it) but the identity default never populates one, and this
        // spike builds no water.
        let rec = sample_record();
        let mut inv = build_identity(&rec, Granularity::PerStratum);
        assert_eq!(
            inv.spans
                .iter()
                .flat_map(|s| &s.portions)
                .filter(|p| p.form == InvForm::Fluid)
                .count(),
            0,
            "identity default builds no fluid"
        );
        // But the slot exists: a move into Fluid is legal (accommodation).
        let mat = inv.spans[0].portions[0].material;
        let mut ctx = inv.ctx();
        ctx.move_form(0, mat, InvForm::Loose, InvForm::Fluid, 0.1);
        assert!(inv.ctx().fraction(0, mat, InvForm::Fluid) > 0.0);
    }

    #[test]
    fn eighths_appear_only_at_the_quantize_step() {
        // §10 open question #3 (fractional representation): the deep tier holds
        // sub-eighth metre quantities; eighths are minted ONLY at collapse.
        let mut s = DeepStrata::default();
        s.deposit(tag(DepEnv::Subaerial, EnergyBand::High), 0.37, 0); // sub-eighth of a 0.9 m voxel
        let inv = build_identity(&s, Granularity::PerStratum);
        // Below the quantize step: a genuine fraction, no rounding.
        assert_eq!(inv.spans[0].portions[0].quantity_m, 0.37);
        // The quantize step: 0.37 / 0.9 * 8 = 3.29 -> 3 eighths.
        assert_eq!(quantize_to_eighths(0.37, 0.9), 3);
        let vc = collapse_top_voxel(&inv, 0.9);
        assert_eq!(vc.solid_eighths(), 3);
    }

    #[test]
    fn footprint_and_sizes_are_reported() {
        // Not a falsifier — pins the type sizes the S17 memory measurement rests
        // on, so a later struct change that moves them fails loudly here.
        assert_eq!(std::mem::size_of::<Portion>(), 16);
        let rec = sample_record();
        let per_stratum = build_identity(&rec, Granularity::PerStratum);
        let per_voxel = build_identity(&rec, Granularity::PerVoxel { voxel_m: 0.9 });
        assert!(per_voxel.footprint_bytes() > per_stratum.footprint_bytes());
    }
}
