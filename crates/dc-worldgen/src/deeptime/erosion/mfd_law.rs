//! **The hybrid-`p` convergence law** — [`MfdParams`], the spatially varying
//! exponent that decides how much of a cell's discharge stays confined
//! (journal/0113, `docs/design/flow.md` § 2.6.2). The kernel it parameterises is
//! `super::mfd`.
//!
//! Partition (north star): **pass/content logic — plugin side by destination.**

use super::mfd::MFD_MIN_WEIGHT;

/// **The hybrid-`p` law — a spatially varying convergence exponent**
/// (FLOW continuation (b'), journal/0113, `docs/design/flow.md` § 2.6.2).
///
/// journal/0109 shipped **one** exponent for the whole world, and that is
/// physically wrong in a specific way: it applies **hillslope sheet-flow behaviour
/// inside channels.** Real water spreads where it is unchannelised and stays in
/// its banks once it is not, and a uniform `p` cannot say both. The measured cost
/// was the peak catchment collapsing **1,245 → 84 cells**: dispersing at *every*
/// cell compounds down the chain, so a trunk river never accumulates.
///
/// **The discriminator is channelisation, and the standard index for it is
/// Montgomery & Dietrich (1988, 1992)'s `χ = A · S²`** — the drainage-area × slope
/// product whose exceedance marks a channel head on a real landscape. `A` is the
/// cell's drainage area in cells (**lagged one epoch** — see [`Erosion::route`])
/// and `S` the steepest downslope gradient on the free-surface potential. The
/// exponent ramps log-linearly in `χ` from [`Self::p_hill`] to [`Self::p_chan`]
/// between [`Self::chi_lo`] and [`Self::chi_hi`].
///
/// **Why `A·S²` and not `A` alone, which is the obvious choice.** An area-only law
/// destroys what the slice before this one bought. Deltas, alluvial-fan tops and
/// braid plains are exactly the places with the *largest* `A`, so an area-only law
/// would make them the most convergent ground on the world and concurrent
/// distributaries would vanish. `A·S²` puts them back on the dispersive side for
/// the physically correct reason: **a delta is where a channel loses its
/// confinement.** Three regimes fall out of one law:
///
/// | regime | `A` | `S` | `χ` | `p` | behaviour |
/// |---|---|---|---|---|---|
/// | hillslope / interfluve | small | any | low | `p_hill` | sheet flow, spreads |
/// | trunk river, gorge, incised valley | large | moderate–high | high | `p_chan` | stays in its banks |
/// | delta top, fan, coastal plain | large | ≈0 | low | `p_hill` | splits — distributaries |
///
/// **Honest limit at this tier.** At 460 m a cell contains an entire
/// hillslope-and-channel system, so this is not "is this cell a channel" but a
/// **sub-grid parameterisation of how much of the cell's discharge is confined**.
/// The thresholds are therefore calibrated against *this world's* own `χ`
/// distribution (`examples/hybrid_p_probe.rs` prints it), never lifted from a
/// field study at 10 m.
///
/// **⚠ STUB #26 — [`Self::chi_lo`] / [`Self::chi_hi`] are fitted to ONE world.**
/// The index is cited and general; the two thresholds are a stand-in until either
/// the joint supply+transport calibration gives `χ` a physical scale (stub #24) or
/// the index is re-expressed dimensionlessly. See `docs/design/stubs.md` § 26.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MfdParams {
    /// The convergence exponent where flow is **unchannelised**. Quinn (1991) /
    /// Freeman (1991)'s dispersive limit is `1.0`.
    ///
    /// Holmgren's calibrated `4–6` band — journal/0109's uniform default — is the
    /// *compromise* a single-exponent scheme is forced into, because one number
    /// has to serve hillslope and channel alike. Once `p` varies, the endpoints
    /// should be the endpoints and not the compromise.
    pub p_hill: f64,
    /// The convergence exponent at the top of the ramp — the last value before
    /// the cell is treated as fully channelised. At `p = 16` a neighbour at 90 %
    /// of the maximum slope keeps 19 % of the steepest direction's weight and one
    /// at 70 % keeps 0.3 %, which the representational floor then drops.
    ///
    /// `p_chan == p_hill` is **uniform `p`** — journal/0109's solve, recovered
    /// exactly, with no ramp and no switch evaluated.
    pub p_chan: f64,
    /// `χ` at or below which the exponent is [`Self::p_hill`].
    pub chi_lo: f64,
    /// **`χ` at or above which the cell routes SINGLE-RECEIVER, exactly** — the
    /// hard switch, and it is a measured necessity rather than a stylistic choice.
    ///
    /// A smooth exponent cannot make a channel confined on terrain this smooth.
    /// The partition's weights go as `(Sₖ/S_max)^p`, so a neighbour at 90 % of the
    /// steepest slope still keeps 19 % at `p = 16` and 8 % at `p = 24`; suppressing
    /// it below the representational floor needs `p > 44`, and a neighbour at 95 %
    /// needs `p > 90`. **Measured on the shipped world: `p = 16` everywhere lifts
    /// the peak catchment only 84 → 145 cells against D8's 1,175** — the trunk
    /// still bleeds a fifth of its discharge at every hop, and a fifth per hop
    /// down a fifty-hop chain is everything.
    ///
    /// So above `chi_hi` the solve routes the whole discharge down the steepest
    /// slope: **once flow is channelised it is confined, and confined flow takes
    /// one path.** The ramp below it is what keeps the transition continuous, so
    /// the switch fires at `p = p_chan` rather than out of a dispersive state.
    pub chi_hi: f64,
    /// [`MFD_MIN_WEIGHT`] as a knob — the representational floor on a share.
    pub min_weight: f64,
}

impl MfdParams {
    /// journal/0109's solve: one exponent everywhere. A named constructor because
    /// it is the control every hybrid measurement is quoted against — and because
    /// it is a **pinned identity path**: `partition_cell` skips the `S/S_max`
    /// normalisation for a uniform law so the arithmetic is bit-for-bit 0109's,
    /// which is what keeps the scalar-load and anonymous-creep goldens reachable.
    ///
    /// The price of that pin is that a uniform law inherits 0109's exponent-range
    /// limit: raw `S^p` on the gentlest gradients this world carries underflows
    /// somewhere past `p ≈ 40`, and a partition that underflows to zero reports a
    /// draining cell as a sink. Uniform `p` is a *control*, not a shipping mode;
    /// the ramp is where large exponents live and it is normalised.
    #[must_use]
    pub fn uniform(p: f64) -> Self {
        Self {
            p_hill: p,
            p_chan: p,
            chi_lo: 1.0,
            chi_hi: 1.0,
            min_weight: MFD_MIN_WEIGHT,
        }
    }

    /// Is this the uniform-`p` solve (no ramp)?
    #[inline]
    #[must_use]
    pub fn is_uniform(&self) -> bool {
        self.p_hill == self.p_chan
    }

    /// Is the flow at channelisation index `chi` confined — i.e. does this cell
    /// route **single-receiver**? Always false for a uniform law.
    #[inline]
    #[must_use]
    pub fn is_channel(&self, chi: f64) -> bool {
        !self.is_uniform() && chi >= self.chi_hi
    }

    /// **The exponent at channelisation index `chi`.** `p_hill` at or below
    /// `chi_lo`, `p_chan` at or above `chi_hi`, log-linear between — monotone
    /// non-decreasing in `chi` by construction, which is the invariant the gate
    /// pins (a law wired backwards would disperse channels and concentrate
    /// hillslopes, and no absolute count could see it). Above `chi_hi` the
    /// exponent stops being consulted at all: see [`Self::is_channel`].
    ///
    /// The ramp is **rounded to an integer**, and that is a cost decision stated
    /// rather than hidden: a fractional exponent forces `powf` on
    /// `8 × cells × epochs` ≈ half a billion directions per production run, where
    /// an integer goes through `powi` — a handful of multiplies. At a 460 m tier
    /// where the exponent is a coarse sub-grid dial, the difference between
    /// `p = 7` and `p = 7.3` is false precision; the difference in gen time is
    /// not. Uniform mode does **not** round, so a probe may still sweep `p = 1.5`.
    #[inline]
    #[must_use]
    pub fn exponent_at(&self, chi: f64) -> f64 {
        if self.is_uniform() || chi <= self.chi_lo {
            return self.p_hill;
        }
        if chi >= self.chi_hi {
            return self.p_chan;
        }
        let t = (chi / self.chi_lo).ln() / (self.chi_hi / self.chi_lo).ln();
        (self.p_hill + (self.p_chan - self.p_hill) * t).round()
    }
}

impl Default for MfdParams {
    /// The shipped hybrid law — see [`DeepConfig::mfd_exponent_channel`] and
    /// siblings for where each number comes from. `chi_lo`/`chi_hi` are calibrated
    /// against the shipped world's own `χ` distribution (journal/0113).
    fn default() -> Self {
        Self {
            p_hill: 1.0,
            p_chan: 16.0,
            chi_lo: 3.0e-2,
            chi_hi: 1.2e-1,
            min_weight: MFD_MIN_WEIGHT,
        }
    }
}
