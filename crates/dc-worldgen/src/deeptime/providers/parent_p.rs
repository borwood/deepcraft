//! **Slot `parent_p`** — *how much phosphorus is in this cell's parent
//! material?*
//!
//! - Owing system: **materials** (the heir is parent-material petrology; the
//!   *consumer* is ecology, which is exactly the distinction the grouping
//!   records — slots are filed under who will **answer**, not who asks).
//! - Granularity: **pass-level**, materialized once per run at
//!   [`BioticSim::new`](crate::deeptime::biotic::BioticSim::new).
//! - Identity: [`identity_parent_p`] — `1.0` everywhere.
//!
//! The slot's question, heir and identity are documented on the field itself
//! ([`Providers::parent_p`](super::Providers::parent_p)). This slot exists in
//! the first slice specifically to make the hot-loop rule concrete: **a provider
//! must never be called inside a hot loop to answer a question that does not
//! change inside that loop.**

/// The cell whose parent material is being characterized, at the one moment the
/// biotic layer asks: initialization, before any epoch has run.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParentCell {
    /// Row-major index into the deep grid.
    pub index: usize,
    /// Grid column.
    pub gx: usize,
    /// Grid row.
    pub gy: usize,
}

/// **Identity for [`Providers::parent_p`](super::Providers::parent_p)**: `1.0`
/// everywhere — a uniform, maximally phosphorus-rich parent material. This is
/// the true identity: the pre-seam code seeded every cell's rock-P pool from one
/// constant (`biotic::P_ROCK_INIT`) and capped rejuvenation at the same
/// constant.
pub fn identity_parent_p(_cell: ParentCell) -> f64 {
    1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_identity_parent_p_is_uniform_one() {
        for index in [0usize, 1, 4_242] {
            let c = ParentCell {
                index,
                gx: index % 64,
                gy: index / 64,
            };
            assert_eq!(identity_parent_p(c), 1.0);
        }
    }
}
