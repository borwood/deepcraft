//! **Slot `wave_energy`** — *how hard does the sea work at this cell?*
//!
//! - Owing system: **hydrology** (the heir is fetch over open water × the wind
//!   field).
//! - Granularity: **value-level**, per shore cell per epoch.
//! - Identity: [`identity_wave_energy`] — the configured global rate.
//!
//! The slot's question, heir and identity are documented on the field itself
//! ([`Providers::wave_energy`](field@super::Providers::wave_energy)). Note that this
//! payload is the slice's known design error, kept deliberately: journal/0060
//! records that the named heir needs *fetch*, which no per-cell payload can
//! carry, so this slot will convert to pass-level when its heir lands.
//! **Granularity follows the heir, not the call site.**

/// The cell the littoral agent is about to attack, as much of it as a provider
/// is allowed to see: no `&DeepGrid`, because a provider captures nothing and
/// borrows nothing that would make it non-trivially registrable.
///
/// `base_rate` is [`DeepConfig::wave_erosion`](crate::deeptime::grid::DeepConfig::wave_erosion)
/// — the global constant the identity provider hands straight back.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaveCell {
    /// Row-major index into the deep grid.
    pub index: usize,
    /// Grid column.
    pub gx: usize,
    /// Grid row.
    pub gy: usize,
    /// The configured global littoral rate (m/epoch at the waterline).
    pub base_rate: f64,
}

/// **Identity for [`Providers::wave_energy`](field@super::Providers::wave_energy)**:
/// the configured global rate, handed back unchanged — the world has one wave
/// climate everywhere.
///
/// Note the pre-existing off-switch this preserves: the littoral agent returns
/// early when the *configured* rate is `<= 0.0`, which is the byte-identity
/// escape `tests/full_agents.rs` already leans on. The provider is consulted per
/// cell only after that gate, so `wave_erosion: 0.0` still means "no littoral
/// term at all", provider or no provider.
pub fn identity_wave_energy(cell: WaveCell) -> f64 {
    cell.base_rate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_identity_wave_energy_is_the_configured_rate() {
        for base_rate in [0.0, 0.05, 3.25] {
            let c = WaveCell {
                index: 0,
                gx: 0,
                gy: 0,
                base_rate,
            };
            assert_eq!(identity_wave_energy(c).to_bits(), base_rate.to_bits());
        }
    }
}
