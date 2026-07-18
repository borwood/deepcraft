//! Block identity. A handful of hard-coded ids for the S1 spike; the real
//! data-driven registry (content packs) arrives with S5.

use serde::{Deserialize, Serialize};

/// Block id. `u16`-sized on purpose: S3's palette compression and the eventual
/// data-driven registry both want a small fixed-width id, and 256 ids (u8) is
/// too tight a ceiling for a game whose content is user-authored.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default, Serialize, Deserialize)]
#[repr(u16)]
pub enum Block {
    #[default]
    Air = 0,
    Stone = 1,
    Dirt = 2,
    Grass = 3,
    Wood = 4,
}

impl Block {
    /// Does this block participate in collision and face culling?
    #[inline]
    pub const fn is_solid(self) -> bool {
        !matches!(self, Block::Air)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn air_is_not_solid_everything_else_is() {
        assert!(!Block::Air.is_solid());
        for b in [Block::Stone, Block::Dirt, Block::Grass, Block::Wood] {
            assert!(b.is_solid());
        }
    }

    #[test]
    fn block_is_two_bytes() {
        assert_eq!(std::mem::size_of::<Block>(), 2);
    }
}
