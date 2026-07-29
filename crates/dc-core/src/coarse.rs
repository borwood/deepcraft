//! **`CoarseField<T>` — the sim→expression boundary type (spines § S-4
//! "Ratified end-state").**
//!
//! A coarse-resolution field (the 460 m deep-time sim, an octree node stratum)
//! hands a value to a fine consumer (per-voxel collapse, the far mesher). The
//! project's characteristic S-4 defect is the consumer performing a **raw
//! per-cell read** and then either thresholding it or painting it across the
//! fine span — a 460 m *square* reaching the eye. This type makes that raw read
//! **inexpressible** on the public surface: the only fine-scale accessors are
//! the two S-4-legal moves, so a downstream square cannot be written
//! (the S-6 / `Option<fn>` pattern — structural, not disciplinary).
//!
//! ## The two moves, extracted from two shape-teachers
//!
//! - **Move A — [`CoarseField::sample`]** (journal/0072, site A1): the cause is
//!   *interpolable*. Bilinearly blend the surrounding cells at a fine world
//!   position, and let a downstream threshold fire on the blended value — the
//!   boundary then follows the cause's contour, not the grid. Gated on
//!   [`Interpolable`], whose [`Interpolable::blend`] must be **exact at
//!   identities** (see the trait docs — A1's hardest lesson).
//! - **Move B — [`CoarseField::sample_dithered`]** (journal/0073, site B1): the
//!   cause is *categorical / non-interpolable* (a strata unit list). You may not
//!   blend it, but you may **dither which cell wins the contact**. The payload is
//!   a [`ShareVec`] (metres per class), never a stored winner — *"store the
//!   winner" is the plurality bug frozen into a type* — and the fine read
//!   inverse-CDF-samples it, the same machinery as `fill::allocate`.
//!
//! ## What each teacher paid for, encoded here
//!
//! - **The anchored blend** ([`Interpolable::blend`]): A1's naïve dot
//!   `Σ w·s` was *close* but sub-ULP off at the identities, and that drift broke
//!   the byte-identity off-switch. The anchor construction `s[d] + Σ w·(s − s[d])`
//!   is bit-exact there. The generic trait-level test [`tests`] pins it so no
//!   future implementor can ship the naïve dot.
//! - **The source axis** ([`DitherSource`]): B1's wrong turn. Unbiasedness is a
//!   property of the *draw* (inverse-CDF over an addressed uniform); the *source*
//!   of that uniform — white noise vs a coherent/interpolated field — is a
//!   **separate axis** the API must expose, because a per-position white-noise
//!   draw is only correct for a consumer reading at the field's own resolution.
//!   A coarse consumer point-sampling white noise aliases it (B1 measured: the
//!   far mesh doubled). So a coarse consumer calls [`CoarseField::summarize`]
//!   instead and dithers at its *own* resolution — near/far agreement becomes
//!   statistical by design (the octree node contract, octree-substrate.md).
//! - **The cake law** (user, 2026-07-22): a continuous *source* over
//!   *nearest-sampled* shares still guillotines minority phases at cell
//!   perimeters. [`CoarseField::sample_dithered`] therefore performs a seeded,
//!   bilinearly-weighted **membership dither of the source cell** before drawing
//!   within that cell's shares — so minority phases interfinger across the
//!   perimeter rather than dying on the grid line.
//!
//! ## Entropy is caller-owned
//!
//! dc-core is headless and holds no RNG. [`DitherSource`] is the seam through
//! which the caller supplies the addressed uniform (dc-worldgen wraps its own
//! `draw_f64` / `interp_select_draw`), so there is **one** draw machinery and no
//! fork, and the SOURCE axis is literally the impl the caller passes.
//!
//! ## Convergence with the octree node payload (A-4: one type)
//!
//! The octree substrate's node stratum reads a finer scale either as an
//! interpolable reduction (move A, deterministic agreement) or a categorical one
//! that must be statistically unbiased at the contact (move B, seeded). That is
//! this same two-move vocabulary; the node payload's fine-scale sampling adopts
//! `CoarseField<T>` in a follow-on (octree-substrate.md § 3). Building two
//! sampling contracts would be A-4 committed on the day the octree pass named
//! A-4 as its shape.

/// **Move A — an interpolable coarse value.** Implemented for causes you may
/// *blend*: the boundary a downstream threshold draws follows `Self`'s contour
/// rather than the cell grid.
///
/// ## The one law: [`blend`](Interpolable::blend) must be EXACT at identities
///
/// A blend that has to round-trip an identity must be *written* to be exact
/// there, not merely close (journal/0072, A1's hardest lesson). Two identities
/// are load-bearing, because a downstream off-switch relies on them being
/// bit-exact:
///
/// - **one-hot weights** (`weights = [.., 1.0, ..]`) → the sample at that index,
///   bit-for-bit. This is why a share vector that is 100 % one class blends a
///   susceptibility table to exactly that class's rate — *argmax is the limiting
///   case of the blend*, not a separate path.
/// - **uniform samples** (`samples = [v, v, …, v]`) → `v`, bit-for-bit, *even
///   when the weights do not sum to exactly 1.0*. A neutralised coupling (every
///   table entry equal) must be a perfect no-op, or a "coupling off" switch
///   silently drifts.
///
/// The naïve dot `Σ wᵢ·sᵢ` fails the second identity sub-ULP when the weights
/// carry normalisation error (`acc / w` does not sum to exactly 1.0). The cure
/// is to **anchor at the max-weight sample** `d`:
///
/// ```text
/// blend(w, s) = s[d] + Σᵢ wᵢ·(sᵢ − s[d]),   d = argmax(w)
/// ```
///
/// When all `sᵢ` are equal every difference is `0`, so the result is `s[d]`
/// exactly; when `w` is one-hot at `d`, every other term is `0`, so the result
/// is `s[d]` exactly. The between is the same convex combination the dot gives.
pub trait Interpolable: Copy {
    /// Weighted blend of `samples` by `weights` (equal length), **anchored at
    /// the max-weight sample** so it is exact at the two identities in the trait
    /// docs. `weights` need not sum to exactly `1.0`.
    fn blend(weights: &[f64], samples: &[Self]) -> Self;
}

/// Index of the strict first-maximum of `weights` — the blend anchor, and the
/// argmax "verdict" a categorical read derives (never stores).
#[inline]
fn argmax(weights: &[f64]) -> Option<usize> {
    let mut best: Option<usize> = None;
    let mut best_v = f64::NEG_INFINITY;
    for (i, &w) in weights.iter().enumerate() {
        if w > best_v {
            best_v = w;
            best = Some(i);
        }
    }
    best
}

impl Interpolable for f64 {
    /// The anchored blend for scalars — bit-for-bit the erosion susceptibility
    /// blend (`lithology::blend_susceptibility`): `weights` are the window
    /// shares, `samples` the per-class rate table.
    #[inline]
    fn blend(weights: &[f64], samples: &[Self]) -> Self {
        debug_assert_eq!(weights.len(), samples.len());
        let d = argmax(weights).unwrap_or(0);
        let anchor = samples.get(d).copied().unwrap_or(0.0);
        weights
            .iter()
            .zip(samples.iter())
            .fold(anchor, |acc, (&w, &s)| acc + w * (s - anchor))
    }
}

/// **A share vector over `N` categories** — the quantity BOTH moves read.
///
/// The shares are metres-per-class (or any additive weight); their argmax is the
/// verdict, their inverse-CDF draw is the dithered membership, their sum is the
/// coarse summary. It is deliberately **not** "the winning class": storing the
/// winner is the plurality bug S-4 forbids, frozen into a type (journal/0073).
///
/// - **Move A:** `ShareVec` is [`Interpolable`] (blend two share vectors,
///   component-wise anchored), so a `CoarseField<ShareVec<N>>` can be
///   `sample`d and the consumer reduces the blended shares (e.g.
///   [`ShareVec::blend_table`]).
/// - **Move B:** [`ShareVec::draw`] inverse-CDF-samples a category index, the
///   same shape as `fill::allocate` / `collapse::draw_class`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShareVec<const N: usize> {
    shares: [f64; N],
}

impl<const N: usize> ShareVec<N> {
    /// A share vector from raw per-class weights. The caller owns their meaning
    /// (metres, fractions); the type only requires them non-negative for the
    /// draw to be a valid CDF (`debug_assert`ed).
    #[inline]
    pub fn from_shares(shares: [f64; N]) -> Self {
        debug_assert!(shares.iter().all(|s| *s >= 0.0), "shares must be ≥ 0");
        ShareVec { shares }
    }

    /// The all-zero vector — the additive identity for [`ShareVec::add`]
    /// / [`CoarseField::summarize`].
    #[inline]
    pub fn zero() -> Self {
        ShareVec { shares: [0.0; N] }
    }

    /// The raw per-class shares. Read-only; the field itself is private so a
    /// `ShareVec` cannot be mutated into an inconsistent CDF after construction.
    #[inline]
    pub fn shares(&self) -> &[f64; N] {
        &self.shares
    }

    /// Total weight (the CDF normaliser). `0.0` for an empty vector.
    #[inline]
    pub fn total(&self) -> f64 {
        self.shares.iter().sum()
    }

    /// **The verdict — argmax of the shares** (strict first-max). A *derivation*
    /// of the quantity, never a separately stored field (spines § S-3): the
    /// square this type forbids is exactly "store the argmax and paint it".
    /// `None` for an all-zero vector.
    #[inline]
    pub fn argmax(&self) -> Option<usize> {
        if self.total() <= 0.0 {
            return None;
        }
        argmax(&self.shares)
    }

    /// **Reduce the shares against a per-class numeric table** by the anchored
    /// blend — move A's numeric consumption (a category whose only downstream use
    /// is a scalar, e.g. an erosion rate). Bit-for-bit
    /// `lithology::blend_susceptibility(shares, tab)`; argmax is the limiting
    /// case (a one-hot window → that class's table entry exactly).
    #[inline]
    pub fn blend_table(&self, tab: &[f64; N]) -> f64 {
        f64::blend(&self.shares, tab)
    }

    /// **Move B — inverse-CDF draw of a category index** from a uniform
    /// `u ∈ [0, 1)`. `P(class i) = shareᵢ / total` exactly for a uniform `u`
    /// (the unbiasedness `fill::allocate` rests on). `None` for an empty vector.
    ///
    /// Unbiasedness is a property of *this draw*; the SOURCE of `u` (white noise
    /// vs a coherent field) is [`DitherSource`]'s axis, not this function's.
    #[inline]
    pub fn draw(&self, u: f64) -> Option<usize> {
        let total = self.total();
        if total <= 0.0 {
            return None;
        }
        let mut cum = 0.0f64;
        let mut last = None;
        for (i, &s) in self.shares.iter().enumerate() {
            if s <= 0.0 {
                continue;
            }
            last = Some(i);
            cum += s / total;
            if u < cum {
                return Some(i);
            }
        }
        // A floating-point residue at the very top of the unit interval lands on
        // the last class with positive share.
        last
    }

    /// Component-wise sum — the summarize reduction (child→parent: a coarse
    /// cell's shares are the sum of the finer shares it covers, metres adding as
    /// metres). Named `plus` rather than `add` so it is not mistaken for
    /// `std::ops::Add` (this is a domain reduction, not operator overloading).
    #[inline]
    pub fn plus(self, other: Self) -> Self {
        let mut shares = self.shares;
        for (a, b) in shares.iter_mut().zip(other.shares.iter()) {
            *a += *b;
        }
        ShareVec { shares }
    }

    /// This vector scaled by `k` — used to area-weight cells in
    /// [`CoarseField::summarize`].
    #[inline]
    fn scale(self, k: f64) -> Self {
        let mut shares = self.shares;
        for a in shares.iter_mut() {
            *a *= k;
        }
        ShareVec { shares }
    }
}

impl<const N: usize> Interpolable for ShareVec<N> {
    /// Component-wise anchored blend. The anchor index is chosen once from the
    /// weights (the same `d` for every component), so the two identities hold for
    /// the whole vector: uniform corners → that vector bit-for-bit; one-hot
    /// weight → that corner bit-for-bit.
    #[inline]
    fn blend(weights: &[f64], samples: &[Self]) -> Self {
        debug_assert_eq!(weights.len(), samples.len());
        let d = argmax(weights).unwrap_or(0);
        let anchor = samples.get(d).copied().unwrap_or_else(Self::zero);
        let mut out = anchor.shares;
        for (c, o) in out.iter_mut().enumerate() {
            let a = anchor.shares[c];
            for (&w, s) in weights.iter().zip(samples.iter()) {
                *o += w * (s.shares[c] - a);
            }
        }
        ShareVec { shares: out }
    }
}

/// **The addressed-uniform source — caller-owned entropy** (dc-core is
/// headless). The *implementation* IS the SOURCE axis move B must expose:
///
/// - a **white-noise** impl (`draw_f64(seed, salt, wx, wz)`) is unbiased at the
///   field's own resolution but **aliases** under a coarse point-sampler
///   (journal/0073: the far mesh doubled);
/// - a **coherent** impl (a bilinearly-interpolated corner-hash field, à la
///   `interp_select_draw`) has a low-frequency content a coarse consumer can
///   point-sample, at the cost of a small toward-50/50 bias (the dice-sum CDF
///   distortion, spines § 4 carve-out 1).
///
/// A coarse consumer that wants neither cost calls [`CoarseField::summarize`]
/// and dithers at its own resolution. The type does not pick for you; it makes
/// the choice a visible argument.
pub trait DitherSource {
    /// A uniform in `[0, 1)` addressed by a world column and a salt.
    /// Deterministic: no wall clock, no ambient entropy (the caller's `salt`
    /// plays the role `SALT_GEO_*` do in the generator).
    fn uniform(&self, wx: i64, wz: i64, salt: u64) -> f64;
}

/// The cell-space registration a [`CoarseField`] carries — how a world position
/// maps to fractional cell coordinates. Cell centres sit at integer coordinates,
/// so `(0.0, 0.0)` is cell `(0, 0)`'s centre and a fractional coordinate of
/// `i + 0.5` is the boundary between cells `i` and `i + 1`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Registration {
    /// World position of cell `(0, 0)`'s centre.
    pub origin_x: f64,
    pub origin_z: f64,
    /// Width of one cell in world units (isotropic).
    pub cell_size: f64,
}

impl Registration {
    /// A registration with cell `(0, 0)` centred at `(origin_x, origin_z)`.
    pub fn new(origin_x: f64, origin_z: f64, cell_size: f64) -> Self {
        Registration {
            origin_x,
            origin_z,
            cell_size,
        }
    }

    /// Fractional cell coordinates of a world position.
    #[inline]
    pub fn cell_coords(&self, wx: f64, wz: f64) -> (f64, f64) {
        (
            (wx - self.origin_x) / self.cell_size,
            (wz - self.origin_z) / self.cell_size,
        )
    }
}

/// A world voxel column address — the fine read takes this and nothing else
/// (journal/0073: near/far agree exactly only because both address the draw by
/// world position alone).
pub type WorldVoxel = (i64, i64);

/// **A coarse-resolution field with a fine-scale sampling contract.**
///
/// Construct it explicitly from the coarse producer (the sim); the **only** fine
/// reads are the two S-4-legal moves. There is deliberately no `at_cell(ix, iy)`
/// and no [`std::ops::Index`] impl on the public surface, so a fine consumer
/// physically cannot ask for a cell's raw verdict and paint it across a span —
/// the square is inexpressible (see the crate-level compile-fail proof in
/// [`tests`]). The one escape hatch, [`CoarseField::_cell_honest`], is named to
/// be greppable and documented sim-internal-only (the residue's jurisdiction
/// line — a cell-scale verdict the sim takes *about its own cell* is cell-honest
/// by design; the principle governs the EXPRESSION boundary).
#[derive(Clone, Debug)]
pub struct CoarseField<T> {
    data: Vec<T>,
    w: usize,
    h: usize,
    reg: Registration,
}

impl<T: Copy> CoarseField<T> {
    /// Build a field from a coarse producer's row-major `w × h` cell values and
    /// its registration. Explicit construction is the point: there is no `Index`
    /// through which a fine consumer could later smuggle a raw read.
    pub fn from_cells(w: usize, h: usize, reg: Registration, data: Vec<T>) -> Self {
        assert_eq!(data.len(), w * h, "cell data must be exactly w × h");
        CoarseField { data, w, h, reg }
    }

    /// A uniform field — every cell the same value. The registration is a unit
    /// grid at the origin (enough for the law-tests; producers pass a real one).
    pub fn uniform(w: usize, h: usize, value: T) -> Self {
        CoarseField {
            data: vec![value; w * h],
            w,
            h,
            reg: Registration::new(0.0, 0.0, 1.0),
        }
    }

    /// Width in cells.
    pub fn width(&self) -> usize {
        self.w
    }
    /// Height in cells.
    pub fn height(&self) -> usize {
        self.h
    }

    /// **Sim-internal raw cell read — NOT an expression accessor.** Named with a
    /// leading underscore and this warning so it is greppable and never reached
    /// for by reflex on the expression side. Legitimate only where the reader IS
    /// the sim taking a verdict about its own cell (residue (a): cell-honest by
    /// design). Any expression-side consumer that calls this is writing the
    /// square this type exists to forbid.
    #[inline]
    pub fn _cell_honest(&self, ix: usize, iy: usize) -> Option<&T> {
        self.data.get(iy * self.w + ix)
    }

    /// Clamp a base cell index to the valid range (edge extension for the
    /// bilinear stencil, so a sample at the field's rim reads the rim cell).
    #[inline]
    fn clamp_ij(&self, i: i64, j: i64) -> usize {
        let ci = i.clamp(0, self.w as i64 - 1) as usize;
        let cj = j.clamp(0, self.h as i64 - 1) as usize;
        cj * self.w + ci
    }

    /// The bilinear stencil at a world position: the four cell indices and their
    /// weights `[w00, w10, w01, w11]`, edge-clamped. `None` if the field is empty.
    fn stencil(&self, pos: WorldVoxel) -> Option<([usize; 4], [f64; 4])> {
        if self.data.is_empty() {
            return None;
        }
        let (cx, cz) = self.reg.cell_coords(pos.0 as f64, pos.1 as f64);
        // Cell centres at integer coords: the stencil base is floor of the
        // centre-relative coordinate.
        let i0 = cx.floor() as i64;
        let j0 = cz.floor() as i64;
        let fx = cx - i0 as f64;
        let fz = cz - j0 as f64;
        let idx = [
            self.clamp_ij(i0, j0),
            self.clamp_ij(i0 + 1, j0),
            self.clamp_ij(i0, j0 + 1),
            self.clamp_ij(i0 + 1, j0 + 1),
        ];
        let w = [
            (1.0 - fx) * (1.0 - fz),
            fx * (1.0 - fz),
            (1.0 - fx) * fz,
            fx * fz,
        ];
        Some((idx, w))
    }
}

impl<T: Interpolable> CoarseField<T> {
    /// **Move A — threshold LATE on an interpolated cause.** Bilinearly blend the
    /// four surrounding cells at a fine world position (edge-clamped). A
    /// downstream threshold on the result follows `T`'s contour, not the 460 m
    /// grid. `None` only if the field is empty.
    ///
    /// Exact at identities by [`Interpolable::blend`]: a uniform neighbourhood
    /// samples to that value bit-for-bit, so a byte-identity off-switch survives.
    pub fn sample(&self, pos: WorldVoxel) -> Option<T> {
        let (idx, w) = self.stencil(pos)?;
        let samples = [
            self.data[idx[0]],
            self.data[idx[1]],
            self.data[idx[2]],
            self.data[idx[3]],
        ];
        Some(T::blend(&w, &samples))
    }
}

impl<const N: usize> CoarseField<ShareVec<N>> {
    /// **Move B — dither MEMBERSHIP for a categorical cause**, with the cake-law
    /// boundary treatment. Returns a category index, or `None` if the addressed
    /// cell is empty.
    ///
    /// Two seeded draws, both addressed by world position alone (so a shared
    /// kernel feeding near and far inherits the identical answer):
    ///
    /// 1. **Membership dither of the SOURCE CELL** (the cake law). The four
    ///    surrounding cells carry bilinear weights; one uniform draw
    ///    (`salt_membership`) inverse-CDF-selects *which* cell's shares to draw
    ///    from. Near a boundary the weights are ~½/½, so minority phases from
    ///    each cell surface stochastically across the perimeter — the fence
    ///    dissolves into interfingering. At a cell **centre** one weight is
    ///    exactly 1, so it reduces to the containing cell's own shares.
    ///
    ///    **⚠ THIS IS A CELL-WIDE BLEND, NOT A PERIMETER TREATMENT, AND THE
    ///    SENTENCE ABOVE HAS BEEN MISREAD ONCE ALREADY** (journal/0124, the first
    ///    adoption). "One weight ≈ 1 away from a boundary" is true *pointwise* and
    ///    badly misleading *in aggregate* — the reduction holds only in a small
    ///    neighbourhood of the centre. Integrated over a cell, the weight on the
    ///    home cell is
    ///
    ///    ```text
    ///    E[w_home] = 4·(∫₀^½ (1 − x) dx)²  =  4·(3/8)²  =  9/16  =  0.5625
    ///    ```
    ///
    ///    so **~44 % of fine samples read a NEIGHBOURING cell's shares, everywhere
    ///    in the field**, not just near a frontier. That is the intended
    ///    behaviour — a categorical field that varies continuously in *expectation*
    ///    is the whole point — but a consumer sizing its effect, or comparing this
    ///    field against a nearest-cell reader, must budget for 9/16 and not for
    ///    "≈ 1". The first adopter budgeted for "≈ 1", and a near/far agreement
    ///    test was the only thing that noticed.
    /// 2. **Class draw within that cell** (`salt_class`): [`ShareVec::draw`],
    ///    inverse-CDF over the chosen cell's shares.
    ///
    /// The SOURCE axis (white noise vs coherent) is `src`'s impl — a coarse
    /// consumer that would alias a white-noise source calls [`Self::summarize`]
    /// instead.
    pub fn sample_dithered(
        &self,
        pos: WorldVoxel,
        src: &(impl DitherSource + ?Sized),
        salt_membership: u64,
        salt_class: u64,
    ) -> Option<usize> {
        let (idx, w) = self.stencil(pos)?;
        // 1. Cake law: inverse-CDF over the bilinear membership weights.
        let total_w: f64 = w.iter().sum();
        let um = src.uniform(pos.0, pos.1, salt_membership) * total_w;
        let mut cum = 0.0f64;
        let mut chosen = idx[0];
        for (&wk, &ik) in w.iter().zip(idx.iter()) {
            cum += wk;
            chosen = ik;
            if um < cum {
                break;
            }
        }
        // 2. Draw a class within the chosen source cell's shares.
        let uc = src.uniform(pos.0, pos.1, salt_class);
        self.data[chosen].draw(uc)
    }

    /// **The coarse read — sum the shares over a world region.** A coarse
    /// consumer point-sampling [`Self::sample_dithered`] aliases a white-noise
    /// source (journal/0073); instead it asks for *the mix over its own cell*
    /// and dithers (or renders dominant) at its own resolution. Near/far
    /// agreement then becomes statistical by design (the octree contract).
    ///
    /// The region is a world-voxel AABB `[(x0, z0), (x1, z1))`; each overlapping
    /// cell contributes its shares weighted by the overlap **area**, so the
    /// summary is the area-weighted mean composition (the expectation a fine
    /// dither over the region converges to). `None` if the field is empty or the
    /// region has no area.
    pub fn summarize(&self, x0: i64, z0: i64, x1: i64, z1: i64) -> Option<ShareVec<N>> {
        if self.data.is_empty() || x1 <= x0 || z1 <= z0 {
            return None;
        }
        // The region in fractional cell coordinates.
        let (cx0, cz0) = self.reg.cell_coords(x0 as f64, z0 as f64);
        let (cx1, cz1) = self.reg.cell_coords(x1 as f64, z1 as f64);
        // Cell i covers cell-coordinate [i - 0.5, i + 0.5) (centres at integers);
        // it overlaps [cx0, cx1) when i > cx0 - 0.5 and i < cx1 + 0.5. Bound the
        // loop generously — the overlap-area computation zeroes any cell that
        // does not actually intersect, so a slightly-wide range is harmless.
        let i_lo = (cx0 - 0.5).floor() as i64;
        let i_hi = (cx1 + 0.5).ceil() as i64;
        let j_lo = (cz0 - 0.5).floor() as i64;
        let j_hi = (cz1 + 0.5).ceil() as i64;
        let mut acc = ShareVec::<N>::zero();
        let mut area = 0.0f64;
        for j in j_lo..j_hi {
            let cj = j.clamp(0, self.h as i64 - 1) as usize;
            let cell_z0 = j as f64 - 0.5;
            let cell_z1 = j as f64 + 0.5;
            let oz = (cell_z1.min(cz1) - cell_z0.max(cz0)).max(0.0);
            if oz <= 0.0 {
                continue;
            }
            for i in i_lo..i_hi {
                let ci = i.clamp(0, self.w as i64 - 1) as usize;
                let cell_x0 = i as f64 - 0.5;
                let cell_x1 = i as f64 + 0.5;
                let ox = (cell_x1.min(cx1) - cell_x0.max(cx0)).max(0.0);
                if ox <= 0.0 {
                    continue;
                }
                let a = ox * oz;
                acc = acc.plus(self.data[cj * self.w + ci].scale(a));
                area += a;
            }
        }
        if area <= 0.0 {
            return None;
        }
        Some(acc.scale(1.0 / area))
    }
}

/// **The structural proof — the raw per-cell read is inexpressible.**
///
/// There is no `Index` impl and no `at_cell` on the public surface, so a fine
/// consumer cannot fetch a cell's verdict and paint it across a span. This
/// doc-test FAILS TO COMPILE, which is the S-6-style proof (the square is a
/// *type error*, not a discipline):
///
/// ```compile_fail
/// use dc_core::coarse::{CoarseField, ShareVec};
/// let f = CoarseField::<ShareVec<2>>::uniform(4, 4, ShareVec::from_shares([1.0, 0.0]));
/// // Neither of these exists on the public API — a raw per-cell read cannot be
/// // written, so the 460 m square cannot reach the eye:
/// let _sq = f.at_cell(0, 0);
/// let _sq2 = f[(0usize, 0usize)];
/// ```
///
/// (The sim-internal `_cell_honest` escape hatch is the one deliberate,
/// greppable exception — see [`CoarseField::_cell_honest`].)
#[allow(dead_code)]
fn _raw_read_is_inexpressible() {}

#[cfg(test)]
mod tests {
    use super::*;

    // ── A white-noise DitherSource and a coherent one, for the move-B tests ──
    // These are TEST scaffolding (a local splitmix), not the production RNG:
    // the type is source-agnostic by design (dc-core is headless).

    fn splitmix(mut z: u64) -> u64 {
        z = z.wrapping_add(0x9E37_79B9_7F4A_7C15);
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    fn hash01(parts: &[u64]) -> f64 {
        let mut h = 0x243F_6A88_85A3_08D3u64;
        for &p in parts {
            h = splitmix(h ^ p);
        }
        (h >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// White noise: a fresh coin per world position. Correct at the field's own
    /// resolution, aliases under coarse point-sampling.
    struct WhiteNoise {
        seed: u64,
    }
    impl DitherSource for WhiteNoise {
        fn uniform(&self, wx: i64, wz: i64, salt: u64) -> f64 {
            hash01(&[self.seed, salt, wx as u64, wz as u64])
        }
    }

    /// Coherent: a bilinear interpolation of per-cell corner hashes (à la
    /// `interp_select_draw`) over a `stride`-voxel cell, so the field has
    /// low-frequency content a coarse consumer can point-sample.
    struct Coherent {
        seed: u64,
        stride: i64,
    }
    impl DitherSource for Coherent {
        fn uniform(&self, wx: i64, wz: i64, salt: u64) -> f64 {
            let cx = wx.div_euclid(self.stride);
            let cz = wz.div_euclid(self.stride);
            let fx = (wx.rem_euclid(self.stride) as f64 + 0.5) / self.stride as f64;
            let fz = (wz.rem_euclid(self.stride) as f64 + 0.5) / self.stride as f64;
            let c =
                |dx: i64, dz: i64| hash01(&[self.seed, salt, (cx + dx) as u64, (cz + dz) as u64]);
            let a = c(0, 0) * (1.0 - fx) + c(1, 0) * fx;
            let b = c(0, 1) * (1.0 - fx) + c(1, 1) * fx;
            (a * (1.0 - fz) + b * fz).clamp(0.0, 1.0 - f64::EPSILON)
        }
    }

    // ─────────────────────── Interpolable law tests ───────────────────────
    // GENERIC over the trait, so no future implementor can ship the naïve dot.

    fn assert_one_hot_is_exact<T: Interpolable + PartialEq + std::fmt::Debug>(samples: &[T]) {
        for d in 0..samples.len() {
            let mut w = vec![0.0; samples.len()];
            w[d] = 1.0;
            assert_eq!(
                T::blend(&w, samples),
                samples[d],
                "one-hot weight at {d} must return that sample bit-for-bit"
            );
        }
    }

    fn assert_uniform_samples_exact<T: Interpolable + PartialEq + std::fmt::Debug>(
        v: T,
        n: usize,
        // weights that deliberately do NOT sum to 1.0 (normalisation drift)
        weights: &[f64],
    ) {
        let samples = vec![v; n];
        assert_eq!(
            T::blend(weights, &samples),
            v,
            "uniform samples must blend to that value bit-for-bit, even with weight drift"
        );
    }

    #[test]
    fn blend_is_exact_at_one_hot_for_f64() {
        assert_one_hot_is_exact(&[1.0f64, 2.5, -3.0, 8.125]);
    }

    #[test]
    fn blend_is_exact_at_uniform_samples_for_f64() {
        // Weights summing to 0.999_999_9 — the sub-ULP drift the naïve dot fails.
        let w = [0.3, 0.2999999, 0.4];
        assert_uniform_samples_exact(7.5f64, 3, &w);
        assert_uniform_samples_exact(0.0f64, 3, &w);
    }

    #[test]
    fn blend_is_exact_at_one_hot_for_sharevec() {
        let a = ShareVec::from_shares([1.0, 0.0, 0.0]);
        let b = ShareVec::from_shares([0.25, 0.5, 0.25]);
        let c = ShareVec::from_shares([0.0, 0.1, 0.9]);
        assert_one_hot_is_exact(&[a, b, c]);
    }

    #[test]
    fn blend_is_exact_at_uniform_samples_for_sharevec() {
        let v = ShareVec::from_shares([0.55, 0.45, 0.0]);
        let w = [0.3, 0.2999999, 0.4];
        assert_uniform_samples_exact(v, 3, &w);
    }

    #[test]
    fn the_naive_dot_would_drift_here_but_the_anchor_does_not() {
        // Direct demonstration of the identity the anchor construction saves and
        // the naïve dot loses: uniform table, weights that don't sum to 1.0.
        let tab = [2.0f64; 4];
        let w = [0.25, 0.25, 0.25, 0.2499999];
        let naive: f64 = w.iter().zip(tab.iter()).map(|(&x, &t)| x * t).sum();
        assert_ne!(
            naive, 2.0,
            "the naïve dot drifts (this is A1's broken off-switch)"
        );
        assert_eq!(f64::blend(&w, &tab), 2.0, "the anchored blend does not");
    }

    #[test]
    fn argmax_is_the_limiting_case_of_blend_table() {
        // A one-hot share vector reduces a table to exactly that class's entry —
        // argmax ∘ shares, recovered as the degenerate blend.
        let tab = [1.5f64, 9.0, 0.25];
        for d in 0..3 {
            let mut s = [0.0; 3];
            s[d] = 1.0;
            let sv = ShareVec::from_shares(s);
            assert_eq!(sv.blend_table(&tab), tab[d]);
            assert_eq!(sv.argmax(), Some(d));
        }
    }

    // ─────────────────────── CoarseField::sample (move A) ───────────────────

    #[test]
    fn sample_is_exact_over_a_uniform_field() {
        // The byte-identity off-switch: a uniform field samples to that value
        // everywhere, bit-for-bit (edge cells included).
        let v = ShareVec::from_shares([0.55, 0.45, 0.0]);
        let f = CoarseField::from_cells(4, 4, Registration::new(0.0, 0.0, 460.0), vec![v; 16]);
        for x in [-1000i64, 0, 137, 460, 900, 4000] {
            for z in [-1000i64, 0, 250, 920, 3000] {
                assert_eq!(
                    f.sample((x, z)),
                    Some(v),
                    "uniform field must sample exactly at ({x},{z})"
                );
            }
        }
    }

    #[test]
    fn sample_interpolates_between_cells() {
        // A two-cell gradient in x: at the midpoint between cell centres the
        // sample is the mean, and it is monotone across — the contour, not a step.
        let a = 0.0f64;
        let b = 10.0f64;
        let reg = Registration::new(0.0, 0.0, 100.0);
        let f = CoarseField::from_cells(2, 1, reg, vec![a, b]);
        // Cell 0 centre at world x=0, cell 1 centre at world x=100.
        assert_eq!(f.sample((0, 0)), Some(0.0));
        assert_eq!(f.sample((100, 0)), Some(10.0));
        let mid = f.sample((50, 0)).unwrap();
        assert!(
            (mid - 5.0).abs() < 1e-9,
            "midpoint should be the mean, got {mid}"
        );
        // Monotone: no step anywhere across the boundary.
        let mut prev = f.sample((0, 0)).unwrap();
        for x in 1..=100 {
            let v = f.sample((x, 0)).unwrap();
            assert!(
                v >= prev - 1e-12,
                "sample must not step down across the contour"
            );
            prev = v;
        }
    }

    // ──────────────── ShareVec::draw unbiasedness (the fill.rs proof) ────────

    #[test]
    fn the_draw_is_unbiased_over_the_uniform() {
        // Same proof shape as fill::allocation_is_unbiased_over_the_draw:
        // averaged over the draw, P(class i) = share_i / total to 1e-2.
        //
        // **The three cases are `collapse::draw_class`'s, ported here when that
        // hand-rolled twin retired into this function (journal/0124, member #0).**
        // The retiring test asserted exactly this bound over exactly these
        // vectors, and two of them say something the original single case did
        // not: a THREE-class split (a CDF with an interior step, not just a
        // boundary), and an **un-normalized** vector whose shares are metres
        // rather than fractions — which is what a `ShareVec` built from a
        // record's top-window thicknesses actually holds. `draw` computes its own
        // `total`, so the metres case is the one that proves the normalisation is
        // the type's job and not the caller's.
        const M: usize = 20_000;
        let cases = [
            ShareVec::from_shares([0.55, 0.45, 0.0, 0.0]),
            ShareVec::from_shares([0.50, 0.30, 0.20, 0.0]),
            // metres, summing to 0.90 — the top-of-record window, un-normalized
            ShareVec::from_shares([0.09, 0.81, 0.0, 0.0]),
        ];
        for sv in cases {
            let mut count = [0u64; 4];
            for k in 0..M {
                let u = (k as f64 + 0.5) / M as f64;
                if let Some(i) = sv.draw(u) {
                    count[i] += 1;
                }
            }
            let total = sv.total();
            for (i, (&cnt, &share)) in count.iter().zip(sv.shares().iter()).enumerate() {
                let got = cnt as f64 / M as f64;
                let want = share / total;
                assert!(
                    (got - want).abs() < 1e-2,
                    "class {i}: drew {got}, share {want}"
                );
                if share <= 0.0 {
                    assert_eq!(cnt, 0, "a zero-share class is never drawn");
                }
            }
        }
    }

    // ─────────────── Cake law: minority crosses a synthetic boundary ─────────

    #[test]
    fn cake_law_a_minority_phase_interfingers_across_a_cell_boundary() {
        // Two adjacent cells, DIFFERENT plurality winners, each with a minority:
        //   cell A: 70% class 0, 30% class 1
        //   cell B: 30% class 0, 70% class 1
        // Under a plurality verdict the boundary is a fence: all class 0 up to the
        // grid line, all class 1 past it — class 1 NEVER appears in A's interior,
        // class 0 NEVER in B's. Under the cake-law dither, both minorities appear
        // on BOTH sides of the perimeter, with the fraction shifting across it.
        let a = ShareVec::from_shares([0.7, 0.3]);
        let b = ShareVec::from_shares([0.3, 0.7]);
        let reg = Registration::new(0.0, 0.0, 100.0); // cells 100 voxels wide
        let f = CoarseField::from_cells(2, 1, reg, vec![a, b]);
        let src = Coherent {
            seed: 99,
            stride: 8,
        };

        // Sample a strip straddling the boundary (world x in [40, 60), around the
        // cell-centre-to-cell-centre span 0..100; the perimeter is at x=50).
        let sample_band = |lo: i64, hi: i64| {
            let mut c = [0u64; 2];
            for x in lo..hi {
                for z in 0..64 {
                    if let Some(i) = f.sample_dithered((x, z), &src, 0xA, 0xB) {
                        c[i] += 1;
                    }
                }
            }
            c
        };
        // Just inside A (x < 50): class 1 (A's minority) must appear, not zero.
        let left = sample_band(35, 50);
        // Just inside B (x >= 50): class 0 (B's minority) must appear, not zero.
        let right = sample_band(50, 65);
        assert!(
            left[1] > 0,
            "A's minority (class 1) must interfinger, got 0 — a fence"
        );
        assert!(
            right[0] > 0,
            "B's minority (class 0) must interfinger, got 0 — a fence"
        );
        // And the fraction shifts across the perimeter: class 1 is rarer on the A
        // side than on the B side (a gradient, not a step, not a uniform mush).
        let f_left = left[1] as f64 / (left[0] + left[1]) as f64;
        let f_right = right[1] as f64 / (right[0] + right[1]) as f64;
        assert!(
            f_right > f_left,
            "class-1 fraction must rise across the boundary: A-side {f_left:.3}, B-side {f_right:.3}"
        );
    }

    // ──────── summarize-then-dither ≈ fine-dither statistics (coarse read) ───

    #[test]
    fn summarize_then_coarse_dither_matches_fine_dither_within_tolerance() {
        // A field with a smoothly varying composition; a coarse consumer that
        // summarizes a region and dithers once per coarse cell must recover the
        // same class histogram (in expectation) as a fine consumer dithering
        // every voxel — the octree "agreement is statistical" contract.
        let mut cells = Vec::new();
        let (w, h) = (8usize, 8usize);
        for j in 0..h {
            for i in 0..w {
                // composition sweeps across the field
                let t = (i + j) as f64 / (w + h) as f64;
                cells.push(ShareVec::from_shares([1.0 - t, t]));
            }
        }
        let cell = 32i64;
        let reg = Registration::new(0.0, 0.0, cell as f64);
        let f = CoarseField::from_cells(w, h, reg, cells);
        let src = WhiteNoise { seed: 7 };

        // Fine dither over the whole field's world extent, at voxel resolution.
        let (x0, z0) = (-16i64, -16i64);
        let (x1, z1) = (x0 + (w as i64) * cell, z0 + (h as i64) * cell);
        let mut fine = [0u64; 2];
        for x in x0..x1 {
            for z in z0..z1 {
                if let Some(i) = f.sample_dithered((x, z), &src, 1, 2) {
                    fine[i] += 1;
                }
            }
        }
        // Coarse: summarize the whole region, then weight by its own class shares
        // (the coarse consumer renders composition, not a point draw).
        let s = f.summarize(x0, z0, x1, z1).unwrap();
        let n: f64 = (fine[0] + fine[1]) as f64;
        let fine_frac1 = fine[1] as f64 / n;
        let coarse_frac1 = s.shares()[1] / s.total();
        assert!(
            (fine_frac1 - coarse_frac1).abs() < 2e-2,
            "coarse summary {coarse_frac1:.4} must match fine dither {fine_frac1:.4} within 2e-2"
        );
    }

    /// **The SOURCE axis, measured — and a correction to the teachers' framing.**
    ///
    /// Spines § 4 carve-out 1 and journal/0073 describe the coherent
    /// (interpolated-uniform) source's cost as a *"toward-50/50 bias"* that
    /// *"flattens mixes"*. Measured through the inverse-CDF draw, the aggregate
    /// bias is the **opposite sign**: a middle-heavy `u` makes `F(s) > s` for the
    /// class straddling the cumulative-½ point, so in a 2-class cell the
    /// **majority is amplified and the minority under-represented** — the mix is
    /// pushed *away* from 50/50, not toward it. White noise is unbiased
    /// (`F(s) = s`). This is a LOUD PLEA finding (see journal/0075): the coherent
    /// source *sharpens* minority phases rather than flattening them, which
    /// compounds the cake observation rather than easing it.
    #[test]
    fn white_noise_is_unbiased_but_coherent_amplifies_the_majority() {
        let sv = ShareVec::from_shares([0.6, 0.4]);
        let reg = Registration::new(0.0, 0.0, 1.0); // 1-voxel cells: native res
        let f = CoarseField::from_cells(1, 1, reg, vec![sv]);
        let measure = |src: &dyn DitherSource| -> f64 {
            let mut c = [0u64; 2];
            for x in 0..300i64 {
                for z in 0..300i64 {
                    if let Some(i) = f.sample_dithered((x, z), src, 5, 6) {
                        c[i] += 1;
                    }
                }
            }
            c[0] as f64 / (c[0] + c[1]) as f64
        };
        let white = measure(&WhiteNoise { seed: 3 });
        let coherent = measure(&Coherent { seed: 3, stride: 4 });
        assert!(
            (white - 0.6).abs() < 2e-2,
            "white noise must be unbiased at native res, got {white}"
        );
        // The majority (share 0.6) is rendered ABOVE 0.6 by the coherent source —
        // the dice-sum CDF distortion, in the sharpening direction.
        assert!(
            coherent > 0.63,
            "coherent source should amplify the 0.6 majority past 0.63, got {coherent}"
        );
    }

    #[test]
    fn cell_honest_is_the_only_raw_read_and_it_is_named() {
        // The escape hatch exists (residue jurisdiction) but is greppable and
        // returns by-ref, never a paint-across primitive. This test documents
        // that the raw read is reachable ONLY through the underscored name.
        let v = ShareVec::from_shares([1.0, 0.0]);
        let f = CoarseField::from_cells(2, 1, Registration::new(0.0, 0.0, 1.0), vec![v, v]);
        assert_eq!(f._cell_honest(0, 0), Some(&v));
        assert_eq!(f._cell_honest(9, 9), None);
    }
}
