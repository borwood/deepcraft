//! **Flux on faces** — the flow record (`docs/design/flow.md` § 2, RATIFIED
//! 2026-07-25; FLOW slice 1, the *recording* half).
//!
//! This module replaces the **output representation** of the drainage solve. The
//! solve itself is untouched (priority-flood → D8 route → accumulate area is good
//! numerics, flow.md § 7); what changes is what it *leaves behind*.
//!
//! ## Why the receiver tree had to go
//!
//! `recv: Vec<i32>` is **one out-edge per cell** — D8-with-one-receiver is a
//! spanning tree by construction. A tree can represent convergence. It
//! **structurally cannot represent divergence**: no distributaries, no braiding,
//! no anabranch, no alluvial fan, no delta. *The Mississippi delta cannot exist in
//! a receiver tree.* And it was exported as "the **last** routing" — we ran the
//! process two hundred times and kept the final frame.
//!
//! ## What replaces it
//!
//! **Directed flux on faces, accumulated per tectonic CHAPTER.** A face carries
//! the § 1.3 atom: *at (cell, stratum-slot), flux crossed face F with magnitude M,
//! in form P (free/bound), carrying load L of fluid f, under cause C, at chapter
//! K.*
//!
//! ### Where the divergence comes from (and why it is honest)
//!
//! Within one epoch the solve still hands us one receiver per cell. But a chapter
//! is **25 epochs** (200 iterations / 8 chapters), and the terrain moves under the
//! flow: uplift, incision and deposition all re-shape the filled surface, so a
//! cell's steepest-descent receiver **switches** between epochs. Accumulating the
//! epoch fluxes into per-chapter face totals therefore records, honestly:
//!
//! > during chapter K, this much water left the cell eastward **and** this much
//! > left it southward.
//!
//! That is not a fabrication laid on top of a tree — it is *avulsion*, the actual
//! physical origin of braid plains, alluvial fans and distributary networks, read
//! at the time resolution the record keeps. A receiver tree cannot say it at any
//! resolution; a face record says it for free.
//!
//! ### Faces are 3D (flow.md § 2.3)
//!
//! [`FaceKey`] spans all three families, because lateral-only forecloses most of
//! the hydrosphere:
//!
//! | family | variants | today |
//! |---|---|---|
//! | **lateral** (cell ↔ cell) | [`FaceKey::Nw`] … [`FaceKey::Se`] | **populated** — the routed discharge |
//! | **vertical** (slot ↔ slot in a column) | [`FaceKey::Down`], [`FaceKey::Up`] | **populated since continuation (a)** — the head field's exchange |
//! | **boundary** | [`FaceKey::Ocean`], [`FaceKey::BaseLevel`], [`FaceKey::Atmosphere`] | **populated** (see below) |
//!
//! **The vertical faces were slice 1's honest empty, and continuation (a) filled
//! them.** That solve was purely surface routing — no infiltration, no percolation,
//! no Darcy flux — so there was no number to write and fabricating one would have
//! been worse than a zero (A-1). What writes them now is the **head field**
//! ([`super::head`], `dc:field/head`): infiltration where the water table stands
//! below the ground, **artesian rise** where a confined potential stands above it.
//! Their remaining heir is the **free↔bound form edge** (continuation (c)), which
//! turns the exchange into an occupancy transition with void intervals.
//!
//! **Magnitudes are in one currency across all three families**: one unit is one
//! cell-epoch of the drainage solve's seeded source. That holds for the vertical
//! faces without a conversion constant because gravity drainage through the vadose
//! zone runs at a **unit hydraulic gradient** and the property sheet's
//! `permeability` is already a relative 0..1 number — so no mode has to be carried
//! in the record to say what a magnitude means (flow.md § 11.3).
//!
//! The boundary faces split by *what the flow left through*:
//! - [`FaceKey::Ocean`] — the cell stood at or below the current sea stand.
//! - [`FaceKey::BaseLevel`] — the domain border (the model's base level).
//! - [`FaceKey::Atmosphere`] — an interior closed-basin terminus: flow reached a
//!   filled depression with no lower outlet and left the model. Physically the
//!   only way mass leaves an endorheic basin is **evaporation**, and flow.md
//!   § 2.3 names the top face as exactly that sink, so that is where it is
//!   recorded. The atmospheric *source* half (precipitation) is deliberately
//!   **not** recorded: this solve seeds one unit of area per cell per epoch, a
//!   value that is exactly derivable, and S-2 says store only what derivation
//!   cannot predict. A real precip-weighted source arrives with the head field.
//!
//! ### Directed half-faces, and why that is the seamless primitive
//!
//! A face is **shared** — cell A's east face *is* cell B's west face — so the
//! record stores each *directed* half-face exactly once, on the cell it leaves.
//! Cell B's in-flux through that face is then a **query over A's entry**, not a
//! second copy that could disagree ([`FluxRecord::in_faces`]). Refinement built on
//! face data therefore agrees from both sides **by construction**: seamlessness
//! stops being an achievement and becomes something you cannot violate.
//!
//! Keeping both directed halves separately (rather than a signed net) also means a
//! chapter in which flow reversed records **both** crossings — a partial down
//! payment on flow.md § 5's "gross energy separate from net flux" limit.
//!
//! ### The slot-pairing rule — stated, because flow.md § 2.2 does not
//!
//! "A face is shared by construction" holds at the **cell-pair** level and stops
//! there. Adjacent columns do **not** have aligned slot indices — § 1.2 already
//! forbids correlating by slot index, because surfaces are diachronous (one
//! shoreline deposit is one slot and many ages). So a rule is needed and the
//! design document is silent. **This slice pairs lateral faces by CHAPTER**:
//!
//! > cell A's out-face `F` in chapter `K` is the same physical face as cell B's
//! > in-face `opposite(F)` in chapter `K`.
//!
//! It is the rule the free/surface regime can justify: chapter is a *time*
//! surface, shared globally, and the flow being paired is contemporaneous surface
//! flow across a shared cell boundary. Each column then binds that chapter to its
//! own slot independently via [`slot_for_chapter`] — which is exactly the § 1.2
//! discipline ("correlate by chapter, never by slot index") rather than a
//! violation of it, and is why the slot is derived per column instead of stored.
//!
//! **The residual is loud and unbuilt.** The *bound* regime almost certainly needs
//! a different pairing — an aquifer's flux crosses a cell boundary at a
//! **paleo-elevation**, not at "the same chapter's stratum", and regional
//! groundwater genuinely crosses surface divides (§ 2.4). No entry carries
//! `FlowForm::Bound` (the head field's vertical exchange is recorded on vertical
//! faces, form `Free`, and vertical faces pair within a column — no cross-column
//! choice arises), so the question is not yet forced; **it is a design decision the
//! user owns**, filed with continuation (c) (the free/bound edge), and this rule
//! must not be assumed to extend there by default.
//!
//! **RATIFIED 2026-07-29 (flow.md § 11.5 banner, the three-mode confinement rule):**
//! this module's chapter pairing IS mode 1, blessed; modes 2 (elevation/head) and
//! 3 (void connectivity) arrive with continuation (c). The paragraph above
//! previously opened with "Slice 1 records no bound flux at all" — literally true
//! of the `form` tag but misleading once continuation (a) populated the vertical
//! faces; reworded 2026-07-29 (doc-topology F3) with the conclusion unchanged.
//!
//! ## What is NOT stored, on purpose
//!
//! The **stratum slot** of the atom. `DepUnit::chapter` already stamps every
//! recorded unit and units never merge across a chapter boundary, so the slot is
//! *derivable* — [`slot_for_chapter`]. Deriving is not merely cheaper, it is more
//! honest: when erosion has since stripped that chapter's unit the derivation
//! answers `None` (the unconformity ate the record), where a stored index would
//! have silently gone stale and pointed at someone else's stratum.

use super::recorder::DeepStrata;

/// Number of lateral (cell ↔ cell) faces — the D8 neighbourhood. Indices `0..8`
/// of [`FaceKey`] mirror `erosion::NEIGH8` exactly.
pub const LATERAL_FACES: usize = 8;
/// Total face slots per cell in the accumulator's dense scratch.
pub const FACE_SLOTS: usize = 13;

/// **A face of a deep cell** — the thing flux crosses (flow.md § 2.3). Three
/// families in one vocabulary: lateral (cell ↔ cell), vertical (slot ↔ slot
/// within the column), boundary (atmosphere / ocean / base level).
///
/// The lateral variants are ordered to match `erosion::NEIGH8`
/// (`(-1,-1), (0,-1), (1,-1), (-1,0), (1,0), (-1,1), (0,1), (1,1)`), so a
/// direction index and a `FaceKey` are the same integer.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum FaceKey {
    /// Lateral: −x, −y.
    Nw = 0,
    /// Lateral: −y.
    N = 1,
    /// Lateral: +x, −y.
    Ne = 2,
    /// Lateral: −x.
    W = 3,
    /// Lateral: +x.
    E = 4,
    /// Lateral: −x, +y.
    Sw = 5,
    /// Lateral: +y.
    S = 6,
    /// Lateral: +x, +y.
    Se = 7,
    /// **Vertical**: this slot → the slot below it — **infiltration /
    /// percolation**. Populated by the head field (continuation (a)): recharge
    /// runs at the vadose zone's unit gravity gradient, so its rate is the
    /// column's vertical relative permeability, and it is **rejected** where the
    /// water table already stands at the ground (saturation-excess). Karst
    /// capture rides the same face once dissolution switches on (continuation
    /// (c)).
    Down = 8,
    /// **Vertical**: this slot → the slot above it — **artesian rise**, a
    /// spring's last step. Populated by the head field (continuation (a)) wherever
    /// a confining bed lets the potential stand *above* the local ground, which is
    /// exactly what the unconfined `H = y + sat` proxy could not express.
    /// Capillary rise is a further term (an unsaturated-flow seam, not built).
    Up = 9,
    /// **Boundary**: the top face, exchanging with the atmosphere. Populated in
    /// slice 1 only as an **evaporative sink** at closed-basin termini; the
    /// precipitation *source* is exactly derivable under this solve and is
    /// therefore not stored (S-2). See the module docs.
    Atmosphere = 10,
    /// **Boundary**: seaward — the cell stood at or below the sea stand, so the
    /// flow entered the ocean (the persistent base level).
    Ocean = 11,
    /// **Boundary**: seaward — the domain border, the model's base level.
    BaseLevel = 12,
}

impl FaceKey {
    /// The lateral face for a D8 direction index (`0..8`), or `None` past it.
    #[inline]
    pub fn lateral(dir: usize) -> Option<Self> {
        const LATERAL: [FaceKey; LATERAL_FACES] = [
            FaceKey::Nw,
            FaceKey::N,
            FaceKey::Ne,
            FaceKey::W,
            FaceKey::E,
            FaceKey::Sw,
            FaceKey::S,
            FaceKey::Se,
        ];
        LATERAL.get(dir).copied()
    }

    /// Reconstruct a face from its stored code.
    #[inline]
    pub fn from_code(code: u8) -> Option<Self> {
        const ALL: [FaceKey; FACE_SLOTS] = [
            FaceKey::Nw,
            FaceKey::N,
            FaceKey::Ne,
            FaceKey::W,
            FaceKey::E,
            FaceKey::Sw,
            FaceKey::S,
            FaceKey::Se,
            FaceKey::Down,
            FaceKey::Up,
            FaceKey::Atmosphere,
            FaceKey::Ocean,
            FaceKey::BaseLevel,
        ];
        ALL.get(code as usize).copied()
    }

    /// The stored code (also the dense-scratch slot index).
    #[inline]
    pub fn code(self) -> u8 {
        self as u8
    }

    /// Whether this is a lateral (cell ↔ cell) face.
    #[inline]
    pub fn is_lateral(self) -> bool {
        (self as u8) < LATERAL_FACES as u8
    }

    /// Whether this is a vertical (slot ↔ slot) face.
    #[inline]
    pub fn is_vertical(self) -> bool {
        matches!(self, FaceKey::Down | FaceKey::Up)
    }

    /// Whether this is a boundary face (atmosphere / ocean / base level).
    #[inline]
    pub fn is_boundary(self) -> bool {
        matches!(
            self,
            FaceKey::Atmosphere | FaceKey::Ocean | FaceKey::BaseLevel
        )
    }

    /// The `(dx, dy)` cell step of a lateral face (`None` for the others).
    #[inline]
    pub fn delta(self) -> Option<(i32, i32)> {
        const D: [(i32, i32); LATERAL_FACES] = [
            (-1, -1),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ];
        D.get(self as usize).copied()
    }

    /// **The shared-face involution**: the same physical face named from the
    /// other cell. `A.east` and `B.west` are one face — this is what makes a
    /// refinement built on face data agree from both sides by construction.
    /// `None` for vertical and boundary faces (whose opposite is a different
    /// slot or the world outside, not a sibling cell).
    #[inline]
    pub fn opposite(self) -> Option<Self> {
        if !self.is_lateral() {
            return None;
        }
        // NEIGH8 is antisymmetric about its centre: index 7 - i is -(delta i).
        FaceKey::lateral(LATERAL_FACES - 1 - self as usize)
    }

    /// Short code for printouts.
    pub fn label(self) -> &'static str {
        match self {
            FaceKey::Nw => "NW",
            FaceKey::N => "N",
            FaceKey::Ne => "NE",
            FaceKey::W => "W",
            FaceKey::E => "E",
            FaceKey::Sw => "SW",
            FaceKey::S => "S",
            FaceKey::Se => "SE",
            FaceKey::Down => "v",
            FaceKey::Up => "^",
            FaceKey::Atmosphere => "atm",
            FaceKey::Ocean => "sea",
            FaceKey::BaseLevel => "base",
        }
    }
}

/// **The occupancy of the flowing fluid** (flow.md § 1.1): free = fluid in open
/// space, bound = fluid in pore space. Not two systems — one edge on the form
/// transition graph (S-8), the fluid's analogue of `Structure→Loose`.
///
/// Slice 1 records only [`FlowForm::Free`]: the solve routes surface water.
/// [`FlowForm::Bound`] is the named seam — its heir is continuation (c), the
/// free/bound edge with void intervals, which is also what makes a spring a
/// *derived outlet condition* rather than an authored feature.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum FlowForm {
    /// Fluid occupying open space (a channel, a lake, a cave conduit).
    Free = 0,
    /// Fluid occupying pore space (groundwater, an aquifer, a saturated regolith).
    Bound = 1,
}

/// **The mover** (flow.md § 7): `Cause` promoted from "which weathering agent" to
/// "which transport regime drove this flux". Regimes differ in four numbers and a
/// field — viscosity, density, competence, resistance, and the field they follow.
///
/// Slice 1 records only [`FlowCause::Fluvial`]. The rest are carried so the
/// representation does not foreclose them; their heirs are Movement 2b
/// (material-aware transport) and continuation (d) (fluid identity).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[repr(u8)]
pub enum FlowCause {
    /// Water down a head gradient — the only mover slice 1 records.
    Fluvial = 0,
    /// Wind. Silt–sand competence only; climbs gradients water cannot.
    Eolian = 1,
    /// Ice. Indiscriminate competence — a solid whose viscosity is ~10¹³.
    Glacial = 2,
    /// Mass wasting down a slope, cohesion-limited.
    Gravity = 3,
    /// Littoral / marine currents, longshore drift, contourites.
    Marine = 4,
    /// Thermal + head driven circulation (vein and ore fluids, magma).
    Hydrothermal = 5,
    /// Solute transport — the karst mover.
    Dissolution = 6,
}

/// **The identity of the flowing fluid** (flow.md § 2.5). Carrying this is what
/// keeps lava tubes, brine, CO₂ and ice from being foreclosed: *form* is
/// occupancy, *fluid* is the material, *rheology* is that material's properties.
///
/// **A named seam** (heir: continuation (d), "fluid identity — lava/ice/brine on
/// the same atom, own competence curves"). Slice 1 has exactly one fluid and no
/// registry to point at — `dc_core::MaterialId` is the *granular* material
/// registry and water is not in it — so this is a small dense index with one
/// inhabitant. It is deliberately **not** a bool and not absent: the field exists
/// so the day a second fluid flows, nothing about the record's shape has to move.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct FluidId(pub u16);

impl FluidId {
    /// Liquid water — the only fluid slice 1 records.
    pub const WATER: FluidId = FluidId(0);
}

/// **One recorded flux atom** (flow.md § 1.3), minus the stratum slot, which is
/// derived rather than stored (see [`slot_for_chapter`] and the module docs).
///
/// Sixteen bytes, laid out so the two `f32` payloads pack the tags into the tail
/// padding. The residency of the whole record is `entries × 16 + (cells + 1) × 4`
/// — measured, reported, and the input to flow.md § 9.1's open cost question.
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(C)]
pub struct FluxEntry {
    /// **M** — flux magnitude across the face, summed over the chapter's epochs,
    /// in the solve's own units: contributing cell-area per epoch (the same
    /// quantity the old `area` export called discharge), integrated. It is a
    /// *chapter integral*, not a rate: that is what a mass budget wants.
    ///
    /// **One currency across all three face families.** The vertical faces carry
    /// the head field's exchange in the same unit — one cell-epoch of the seeded
    /// source — because vadose recharge runs at a unit hydraulic gradient through a
    /// relative permeability. See the module docs.
    pub magnitude: f32,
    /// **L** — the suspended load that crossed this face, summed over the
    /// chapter's epochs (metres of column thickness, the transport pass's own
    /// currency). Zero on boundary faces: at a sink the solve settles the whole
    /// load into the cell, so nothing crosses out. Zero on **vertical** faces too,
    /// and honestly so: the bound phase carries **solute**, not suspended clastic
    /// load, and dissolution is dormant until continuation (c) — a fabricated
    /// number here would be exactly the A-1 failure the vertical zeros avoided.
    ///
    /// **Bulk only.** The *composition* of the load — which materials, in what
    /// proportion, the thing a placer streak in channel gravel is made of — is a
    /// named seam. Its heir was **Movement 2b, material-aware transport**, which
    /// shipped 2026-07-26 *without landing this*: the per-species split exists in
    /// the solve (`erosion.rs::split_by_shares` feeds a `face_total` that
    /// deliberately collapses species) and is discarded. Heir re-named 2026-07-29:
    /// **the fluvial record-terms slice** (refinement member #1, records-first
    /// ruling). An honest bulk number now; a fabricated composition never.
    pub load: f32,
    /// **f** — the fluid. Always [`FluidId::WATER`] in slice 1.
    pub fluid: FluidId,
    /// **K** — the tectonic chapter, the same stamp `DepUnit::chapter` carries.
    /// This is the whole point of the slice: the flow record keeps *every*
    /// chapter, not the last epoch's frame.
    pub chapter: u8,
    /// **F** — which face the flux crossed.
    pub face: FaceKey,
    /// **P** — free or bound. Always [`FlowForm::Free`] in slice 1.
    pub form: FlowForm,
    /// **C** — the mover. Always [`FlowCause::Fluvial`] in slice 1.
    pub cause: FlowCause,
}

/// **The per-cell face-flux record** — the deep-time flow archive.
///
/// Entries are grouped by cell (a CSR-style index over a flat entry array), and
/// within a cell ordered by `(chapter, face)`. Only faces that actually carried
/// flux are stored; a cell/chapter with no crossing has no entry.
#[derive(Clone, Default, Debug)]
pub struct FluxRecord {
    /// Deep grid width (cells per side). `0` when the record is empty.
    pub w: usize,
    /// Number of tectonic chapters the run spanned (`1` off the tectonic path).
    pub chapters: u8,
    entries: Vec<FluxEntry>,
    /// `cell_start[i] .. cell_start[i + 1]` is cell `i`'s slice. Length `n + 1`,
    /// or empty when the record is empty.
    cell_start: Vec<u32>,
    /// **The simultaneous-divergence counters** (flow.md § 2.6). See
    /// [`SimulDivergence`] — 24 bytes, and the only thing in this struct that is
    /// not a recorded fact.
    pub simultaneous: SimulDivergence,
}

/// **The simultaneous-divergence instrument** (flow.md § 2.6, FLOW continuation
/// (b)) — counters, deliberately **not** facts, carried on the record because the
/// question they answer is not answerable from the record.
///
/// The record aggregates a chapter's 25 epochs into one entry per face, so
/// `out_face_count(cell, chapter) >= 2` is **temporal** divergence: the cell's
/// flow left by two faces *at some point during* the chapter, which is avulsion
/// and which the single-receiver solve already produced. Whether two faces
/// carried flux **in the same epoch** — a delta with two channels flowing at once
/// — is a property of the *solve*, and it is erased by the aggregation before any
/// reader sees the archive. So it is counted as it happens, once, here.
///
/// These are 24 bytes on a ~40 MiB record and they buy the one number that
/// distinguishes MFD's claim from its predecessor's. They are **not** part of the
/// § 1.3 atom, no consumer may treat them as one, and dropping the aggregation
/// window to one epoch would make them redundant rather than wrong.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct SimulDivergence {
    /// `(cell, epoch)` pairs whose flux left through **two or more lateral faces
    /// within that single epoch**. Structurally **zero** under single-receiver
    /// routing, at any cadence and any aggregation window.
    pub cell_epochs: u64,
    /// Distinct `(cell, chapter)` pairs that had **at least one epoch** of
    /// simultaneous divergence — the like-for-like comparand to the record's own
    /// `census().divergent`, which counts the temporal kind over the same keys.
    pub cell_chapters: u64,
    /// The most lateral out-faces any one cell used in a single epoch.
    pub max_out_faces: u32,
}

impl FluxRecord {
    /// True when nothing was recorded (the flag was off).
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty() && self.cell_start.is_empty()
    }

    /// Total recorded entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Every entry, flat (grouped by cell).
    pub fn entries(&self) -> &[FluxEntry] {
        &self.entries
    }

    /// Cell count (`w²`), or `0` when empty.
    pub fn cells(&self) -> usize {
        self.cell_start.len().saturating_sub(1)
    }

    /// **The directed half-faces leaving cell `i`**, all chapters.
    pub fn entries_for(&self, i: usize) -> &[FluxEntry] {
        let (Some(&a), Some(&b)) = (self.cell_start.get(i), self.cell_start.get(i + 1)) else {
            return &[];
        };
        &self.entries[a as usize..b as usize]
    }

    /// The out-faces cell `i` used in `chapter`. **Two or more is DIVERGENCE** —
    /// the thing a receiver tree cannot express at any resolution.
    pub fn out_faces(&self, i: usize, chapter: u8) -> impl Iterator<Item = &FluxEntry> {
        self.entries_for(i)
            .iter()
            .filter(move |e| e.chapter == chapter && e.magnitude > 0.0)
    }

    /// How many distinct faces flux left cell `i` through during `chapter`.
    pub fn out_face_count(&self, i: usize, chapter: u8) -> usize {
        self.out_faces(i, chapter).count()
    }

    /// **The in-faces of cell `i` in `chapter` — a QUERY over the neighbours'
    /// stored half-faces, never a second copy.** A face is shared: cell A's east
    /// face *is* cell B's west face, so the in-flux of B is exactly the out-flux
    /// A recorded. Two or more is convergence (a confluence).
    ///
    /// **Pairs by CHAPTER, never by slot index** — see the module docs § "the
    /// slot-pairing rule". Each column resolves the chapter to its own slot with
    /// [`slot_for_chapter`]; correlating the two columns by slot index would be
    /// the diachronous-surface error flow.md § 1.2 forbids.
    ///
    /// Yields `(neighbour cell, that neighbour's out-entry)`.
    pub fn in_faces(&self, i: usize, chapter: u8) -> Vec<(usize, FluxEntry)> {
        let w = self.w;
        if w == 0 || i >= self.cells() {
            return Vec::new();
        }
        let (gx, gy) = ((i % w) as i32, (i / w) as i32);
        let mut out = Vec::new();
        for dir in 0..LATERAL_FACES {
            let face = FaceKey::lateral(dir).expect("lateral");
            let (dx, dy) = face.delta().expect("lateral has a delta");
            let (nx, ny) = (gx + dx, gy + dy);
            if nx < 0 || ny < 0 || nx as usize >= w || ny as usize >= w {
                continue;
            }
            let j = ny as usize * w + nx as usize;
            // The neighbour's face pointing back at us is the involution.
            let back = face.opposite().expect("lateral has an opposite");
            for e in self.entries_for(j) {
                if e.chapter == chapter && e.face == back && e.magnitude > 0.0 {
                    out.push((j, *e));
                }
            }
        }
        out
    }

    /// How many distinct faces flux entered cell `i` through during `chapter`.
    pub fn in_face_count(&self, i: usize, chapter: u8) -> usize {
        self.in_faces(i, chapter).len()
    }

    /// Sweep the whole record for the two numbers the slice exists to produce.
    pub fn census(&self) -> FluxCensus {
        let mut c = FluxCensus {
            chapters: self.chapters,
            entries: self.entries.len(),
            ..FluxCensus::default()
        };
        let n = self.cells();
        c.cells = n;
        for e in &self.entries {
            if e.face.is_lateral() {
                c.lateral += 1;
            } else if e.face.is_vertical() {
                c.vertical += 1;
            } else {
                match e.face {
                    FaceKey::Ocean => c.boundary_ocean += 1,
                    FaceKey::BaseLevel => c.boundary_base += 1,
                    _ => c.boundary_atmosphere += 1,
                }
            }
        }
        for i in 0..n {
            for k in 0..self.chapters {
                let outs = self.out_face_count(i, k);
                if outs >= 2 {
                    c.divergent += 1;
                }
                c.max_out_faces = c.max_out_faces.max(outs);
                let ins = self.in_face_count(i, k);
                if ins >= 2 {
                    c.convergent += 1;
                }
                c.max_in_faces = c.max_in_faces.max(ins);
            }
        }
        c
    }

    /// **The measured resident footprint** (bytes) — flow.md § 9.1's open cost
    /// question, answered with a number rather than an estimate.
    pub fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.entries.len() * std::mem::size_of::<FluxEntry>()
            + self.cell_start.len() * std::mem::size_of::<u32>()
    }
}

/// The two load-bearing counts of FLOW slice 1, plus the shape of the record.
#[derive(Clone, Copy, Default, Debug)]
pub struct FluxCensus {
    pub cells: usize,
    pub chapters: u8,
    pub entries: usize,
    /// `(cell, chapter)` pairs with **≥ 2 outgoing faces** carrying flux.
    /// **This is the number that proves the primitive changed** — a receiver
    /// tree's count is identically zero, by construction, forever.
    pub divergent: usize,
    /// `(cell, chapter)` pairs with ≥ 2 incoming faces carrying flux.
    pub convergent: usize,
    pub max_out_faces: usize,
    pub max_in_faces: usize,
    pub lateral: usize,
    pub vertical: usize,
    pub boundary_ocean: usize,
    pub boundary_base: usize,
    pub boundary_atmosphere: usize,
}

impl FluxCensus {
    /// **The measured face sparsity** — recorded entries as a fraction of the
    /// dense `cells × chapters × FACE_SLOTS` rectangle a naive layout would
    /// allocate. This is the number the cost model turns on: the S19 projection
    /// puts the record at 1× today's whole `DeepField` around 4.3 % and 4× around
    /// 17.9 %, so "how sparse is the solve, really" is the whole question.
    pub fn face_sparsity(&self) -> f64 {
        let dense = self.cells * self.chapters as usize * FACE_SLOTS;
        if dense == 0 {
            return 0.0;
        }
        self.entries as f64 / dense as f64
    }

    /// The same fraction against only the **lateral** rectangle (`cells ×
    /// chapters × 8`) — the comparison against a D8-shaped dense allocation.
    pub fn lateral_sparsity(&self) -> f64 {
        let dense = self.cells * self.chapters as usize * LATERAL_FACES;
        if dense == 0 {
            return 0.0;
        }
        self.entries as f64 / dense as f64
    }

    /// Mean recorded out-faces per `(cell, chapter)` — the honest "how many
    /// entries does a cell actually cost" number. The distribution behind it is
    /// strongly bimodal (land vs marine), so this mean is a budget figure, never
    /// a sizing rule for a per-cell allocation (S19 finding 3).
    pub fn entries_per_cell_chapter(&self) -> f64 {
        let d = self.cells * self.chapters as usize;
        if d == 0 {
            return 0.0;
        }
        self.entries as f64 / d as f64
    }
}

/// **The stratum slot chapter `K`'s flow facts bind to** (flow.md § 1.2 — in a
/// stratigraphic record, depth IS time).
///
/// Derived, never stored (S-2). `DepUnit::chapter` stamps every unit and the
/// recorder refuses to merge across a chapter boundary, so the slot is the index
/// of the last unit stamped `K`.
///
/// `None` has a meaning and it is the honest one: **that chapter's unit is gone**
/// — erosion stripped it and the contact above is an unconformity. A stored index
/// would have gone stale and pointed at a stranger's stratum; the derivation
/// cannot. Callers wanting the bedrock seam use `strata.units.len()`, the same
/// sentinel slot the [`FactLedger`](super::inventory::FactLedger) keys bedrock at.
pub fn slot_for_chapter(strata: &DeepStrata, chapter: u8) -> Option<usize> {
    strata.units.iter().rposition(|u| u.chapter == chapter)
}

// ---------------------------------------------------------------------------
// The gen-time accumulator.

/// Gen-time accumulator for [`FluxRecord`]: a dense per-chapter scratch that the
/// `dc:deep/flow_record` pass adds into every epoch, flushed to sparse entries at
/// each chapter boundary.
///
/// Inactive (`is_active() == false`) when `DeepConfig::flow_record` is off — then
/// every method is a no-op and [`FluxAccum::finish`] yields an empty record, so
/// the world is byte-identical either way (the S-5 identity default).
pub struct FluxAccum {
    w: usize,
    n: usize,
    active: bool,
    cur_chapter: u8,
    max_chapter: u8,
    /// Dense `n × FACE_SLOTS` chapter scratch: magnitude and load.
    mag: Vec<f32>,
    load: Vec<f32>,
    /// Committed entries, chapter-major; sorted into cell-major by `finish`.
    pending_cell: Vec<u32>,
    pending: Vec<FluxEntry>,
    /// The simultaneous-divergence instrument (see [`SimulDivergence`]), plus the
    /// per-chapter "has this cell already diverged simultaneously?" bitset that
    /// turns a per-epoch event into a distinct `(cell, chapter)` count. One bit per
    /// cell — 36 KB at production scale — cleared at each chapter flush.
    simul: SimulDivergence,
    simul_seen: Vec<u64>,
}

impl FluxAccum {
    /// An inactive accumulator (the flag is off).
    pub fn inactive() -> Self {
        Self {
            w: 0,
            n: 0,
            active: false,
            cur_chapter: 0,
            max_chapter: 0,
            mag: Vec::new(),
            load: Vec::new(),
            pending_cell: Vec::new(),
            pending: Vec::new(),
            simul: SimulDivergence::default(),
            simul_seen: Vec::new(),
        }
    }

    /// An active accumulator over a `w × w` deep grid.
    pub fn new(w: usize) -> Self {
        let n = w * w;
        Self {
            w,
            n,
            active: true,
            cur_chapter: 0,
            max_chapter: 0,
            mag: vec![0.0; n * FACE_SLOTS],
            load: vec![0.0; n * FACE_SLOTS],
            pending_cell: Vec::new(),
            pending: Vec::new(),
            simul: SimulDivergence::default(),
            simul_seen: vec![0u64; n.div_ceil(64)],
        }
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Flush the current chapter's dense scratch into sparse entries and zero it.
    fn flush_chapter(&mut self) {
        if !self.active {
            return;
        }
        let chapter = self.cur_chapter;
        for i in 0..self.n {
            let base = i * FACE_SLOTS;
            for slot in 0..FACE_SLOTS {
                let m = self.mag[base + slot];
                if m <= 0.0 {
                    continue;
                }
                let face = FaceKey::from_code(slot as u8).expect("slot is a face code");
                self.pending_cell.push(i as u32);
                self.pending.push(FluxEntry {
                    magnitude: m,
                    load: self.load[base + slot],
                    fluid: FluidId::WATER,
                    chapter,
                    face,
                    // Slice 1: surface water down a gravitational head, and
                    // nothing else. Both are seams with named heirs (see the
                    // `FlowForm` / `FlowCause` docs) — carried, not assumed away.
                    form: FlowForm::Free,
                    cause: FlowCause::Fluvial,
                });
            }
        }
        self.mag.iter_mut().for_each(|v| *v = 0.0);
        self.load.iter_mut().for_each(|v| *v = 0.0);
        self.simul_seen.iter_mut().for_each(|v| *v = 0);
    }

    /// Add one epoch's routed discharge to the current chapter's faces.
    ///
    /// `out_area` / `out_face_load` are `n × LATERAL_FACES` planes written by the
    /// drainage solve itself ([`super::erosion::Erosion::out_face_area`]) — **what
    /// the solve actually moved across each face**, not a re-derivation from
    /// weights. That is the § 3 discipline: one arithmetic, so the flux record and
    /// the mass budget cannot drift apart. `area` and `routed_surf` supply the sink
    /// case, where the discharge left the model through a boundary face and no
    /// lateral face carries it. Crossing a chapter boundary flushes the previous
    /// chapter first.
    ///
    /// A cell with **two or more non-zero lateral out-faces in this one call** is
    /// *simultaneous* divergence (flow.md § 2.6) and is counted into
    /// [`SimulDivergence`] here, because the per-chapter aggregation below is about
    /// to erase the distinction forever.
    #[allow(clippy::too_many_arguments)]
    pub fn add_epoch(
        &mut self,
        chapter: u8,
        out_area: &[f32],
        out_face_load: &[f32],
        area: &[f64],
        routed_surf: &[f64],
        sea_level: f64,
    ) {
        if !self.active || out_area.len() != self.n * LATERAL_FACES {
            return;
        }
        if chapter != self.cur_chapter {
            self.flush_chapter();
            self.cur_chapter = chapter;
        }
        self.max_chapter = self.max_chapter.max(chapter);
        let w = self.w;
        for i in 0..self.n {
            let base = i * LATERAL_FACES;
            let mut out_faces = 0u32;
            for d in 0..LATERAL_FACES {
                let q = out_area[base + d];
                if q <= 0.0 {
                    continue;
                }
                out_faces += 1;
                let idx = i * FACE_SLOTS + d;
                self.mag[idx] += q;
                self.load[idx] += out_face_load[base + d];
            }
            if out_faces >= 2 {
                self.simul.cell_epochs += 1;
                self.simul.max_out_faces = self.simul.max_out_faces.max(out_faces);
                let (word, bit) = (i / 64, 1u64 << (i % 64));
                if self.simul_seen[word] & bit == 0 {
                    self.simul_seen[word] |= bit;
                    self.simul.cell_chapters += 1;
                }
            }
            if out_faces > 0 {
                continue;
            }
            // A sink: no lateral face carried this cell's discharge, so it left
            // the model through a boundary. Which one, mirroring `route_cell`'s
            // own test order: sea stand first, then the domain border, then an
            // interior closed basin.
            let q = area[i] as f32;
            if q <= 0.0 {
                continue;
            }
            let face = if routed_surf[i] <= sea_level {
                FaceKey::Ocean
            } else if is_border(i, w) {
                FaceKey::BaseLevel
            } else {
                // Endorheic terminus: the solve's flow stops here, and the
                // only physical exit from a closed basin is evaporation.
                FaceKey::Atmosphere
            };
            // The transport pass settles the whole suspended load into a
            // sink cell, so nothing crosses the boundary face with it.
            self.mag[i * FACE_SLOTS + face.code() as usize] += q;
        }
    }

    /// **Add one epoch's VERTICAL (slot ↔ slot) exchange** — FLOW continuation
    /// (a)'s consumer of the head field, and the term that fills the faces slice 1
    /// left honestly zero.
    ///
    /// `exchange` is the head field's signed per-cell rate
    /// ([`super::head::vertical_exchange`]): **positive = down** (infiltration /
    /// percolation), **negative = up** (artesian rise, a spring's last step).
    /// Magnitudes are in **the same currency as the lateral faces** — one unit is
    /// one cell-epoch of the drainage solve's seeded source, because gravity
    /// drainage runs at a unit hydraulic gradient and the property sheet's
    /// `permeability` is already a relative 0..1 number. No mode flag is therefore
    /// needed to say what a vertical magnitude means (flow.md § 11.3).
    ///
    /// **Which slot.** The entry binds to the chapter's slot exactly as every other
    /// entry does ([`slot_for_chapter`]) — and that is not a convenience, it is the
    /// right answer: during chapter `K`, chapter `K`'s unit *was* the contemporaneous
    /// land surface, so the flux crossing its face is precisely the recharge (or the
    /// discharge) of that moment. Vertical faces pair **within** a column, so the
    /// lateral slot-pairing rule (§ 2.2 / § 11.5) is not touched and the conduit
    /// third mode assigned to continuation (c) is not pre-empted.
    ///
    /// **Call it AFTER [`Self::add_epoch`] in the same epoch** — that call has
    /// already flushed any chapter boundary, so this one adds into the open chapter.
    /// A no-op when the head field is off (`exchange` empty), which is what keeps
    /// the record byte-identical to slice 1's under the flag.
    pub fn add_epoch_vertical(&mut self, chapter: u8, exchange: &[f32]) {
        if !self.active || exchange.len() != self.n {
            return;
        }
        if chapter != self.cur_chapter {
            self.flush_chapter();
            self.cur_chapter = chapter;
        }
        self.max_chapter = self.max_chapter.max(chapter);
        for (i, &q) in exchange.iter().enumerate() {
            if q == 0.0 {
                continue;
            }
            let face = if q > 0.0 { FaceKey::Down } else { FaceKey::Up };
            self.mag[i * FACE_SLOTS + face.code() as usize] += q.abs();
        }
    }

    /// Close the last chapter and sort the pending entries into the cell-major
    /// CSR the [`FluxRecord`] serves reads from.
    pub fn finish(mut self) -> FluxRecord {
        if !self.active {
            return FluxRecord::default();
        }
        self.flush_chapter();
        let n = self.n;
        let mut counts = vec![0u32; n + 1];
        for &c in &self.pending_cell {
            counts[c as usize + 1] += 1;
        }
        for i in 0..n {
            counts[i + 1] += counts[i];
        }
        let cell_start = counts.clone();
        let mut cursor = counts;
        let mut entries = vec![
            FluxEntry {
                magnitude: 0.0,
                load: 0.0,
                fluid: FluidId::WATER,
                chapter: 0,
                face: FaceKey::Nw,
                form: FlowForm::Free,
                cause: FlowCause::Fluvial,
            };
            self.pending.len()
        ];
        // Stable counting sort: the pending order is chapter-major then cell then
        // face, so within a cell the result stays ordered by (chapter, face).
        for (e, &c) in self.pending.iter().zip(&self.pending_cell) {
            let at = &mut cursor[c as usize];
            entries[*at as usize] = *e;
            *at += 1;
        }
        FluxRecord {
            w: self.w,
            chapters: self.max_chapter + 1,
            entries,
            cell_start,
            simultaneous: self.simul,
        }
    }
}

/// Border test, matching `erosion::is_border`.
#[inline]
fn is_border(i: usize, w: usize) -> bool {
    let (gx, gy) = (i % w, i / w);
    gx == 0 || gy == 0 || gx == w - 1 || gy == w - 1
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The record is 16 bytes an entry. Residency is sacred; a silent widening
    /// here multiplies by millions.
    #[test]
    fn a_flux_entry_is_sixteen_bytes() {
        assert_eq!(std::mem::size_of::<FluxEntry>(), 16);
    }

    /// `A.east` and `B.west` must be one face, or refinement can disagree across
    /// a cell boundary and the seamlessness guarantee is a wish.
    #[test]
    fn the_lateral_faces_are_an_involution() {
        for d in 0..LATERAL_FACES {
            let f = FaceKey::lateral(d).expect("lateral");
            let o = f.opposite().expect("lateral has an opposite");
            assert_eq!(o.opposite(), Some(f));
            let (dx, dy) = f.delta().expect("delta");
            assert_eq!(o.delta(), Some((-dx, -dy)));
        }
        assert_eq!(FaceKey::Down.opposite(), None);
        assert_eq!(FaceKey::Ocean.opposite(), None);
    }

    #[test]
    fn face_codes_round_trip() {
        for c in 0..FACE_SLOTS as u8 {
            let f = FaceKey::from_code(c).expect("code is a face");
            assert_eq!(f.code(), c);
        }
        assert_eq!(FaceKey::from_code(FACE_SLOTS as u8), None);
    }
}
