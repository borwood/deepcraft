//! Capability tokens — deny by default, scoped, attenuable.
//!
//! v0 trust model (spike-honest): a token is plain serde data carried in the
//! envelope, so *within one process* nothing stops a consumer fabricating one.
//! Enforcement is real at the trust boundaries: the WASM host and the MCP
//! server both hold the consumer's *installed* token and reject any envelope
//! whose claimed token is not covered by it ([`CapabilityToken::covers_token`]
//! — the same subset relation attenuation uses). The production design is a
//! host-issued opaque handle; see S5 results § proposed revisions.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::payload::{DefineItem, Payload, Volume};

/// One scoped grant. `None` volumes mean "anywhere".
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Grant {
    WorldRead {
        volume: Option<Volume>,
    },
    WorldWrite {
        volume: Option<Volume>,
    },
    EntitySpawn,
    RegistryDefine {
        namespace: String,
    },
    EventsSubscribe,
    /// Control of a character's body **and** access to its senses — the
    /// controller verbs (`set_move_intent`/`set_look`/`jump`) and the
    /// diegetic queries (`pose`/`sense_raycast`/`sense_surroundings`) all
    /// require this. `None` = any character (the broad dev/parent form);
    /// `Some(name)` = exactly that character (the attenuated session form).
    CharacterControl {
        character: Option<String>,
    },
}

impl Grant {
    /// Is `narrower` equal-or-narrower than `self`? (The attenuation partial
    /// order; also what boundary hosts use to check claimed vs installed.)
    pub fn covers_grant(&self, narrower: &Grant) -> bool {
        match (self, narrower) {
            (Grant::WorldRead { volume: a }, Grant::WorldRead { volume: b })
            | (Grant::WorldWrite { volume: a }, Grant::WorldWrite { volume: b }) => match (a, b) {
                (None, _) => true,
                (Some(_), None) => false,
                (Some(a), Some(b)) => a.contains_volume(b),
            },
            (Grant::EntitySpawn, Grant::EntitySpawn) => true,
            (Grant::RegistryDefine { namespace: a }, Grant::RegistryDefine { namespace: b }) => {
                a == b
            }
            (Grant::EventsSubscribe, Grant::EventsSubscribe) => true,
            (
                Grant::CharacterControl { character: a },
                Grant::CharacterControl { character: b },
            ) => match (a, b) {
                (None, _) => true,
                (Some(_), None) => false,
                (Some(a), Some(b)) => a == b,
            },
            _ => false,
        }
    }
}

/// What a specific payload needs from a token to be applied.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Requirement {
    /// Read every voxel in this box.
    WorldRead(Volume),
    /// Read with no bound (e.g. an unbounded entity query).
    WorldReadAnywhere,
    /// Write every voxel in this box.
    WorldWrite(Volume),
    EntitySpawn,
    RegistryDefine(String),
    EventsSubscribe,
    /// Control/senses of this specific character.
    CharacterControl(String),
}

impl std::fmt::Display for Requirement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Requirement::WorldRead(v) => write!(
                f,
                "world.read({},{},{})..({},{},{})",
                v.min.x, v.min.y, v.min.z, v.max.x, v.max.y, v.max.z
            ),
            Requirement::WorldReadAnywhere => write!(f, "world.read(anywhere)"),
            Requirement::WorldWrite(v) => write!(
                f,
                "world.write({},{},{})..({},{},{})",
                v.min.x, v.min.y, v.min.z, v.max.x, v.max.y, v.max.z
            ),
            Requirement::EntitySpawn => write!(f, "entity.spawn"),
            Requirement::RegistryDefine(ns) => write!(f, "registry.define({ns})"),
            Requirement::EventsSubscribe => write!(f, "events.subscribe"),
            Requirement::CharacterControl(name) => write!(f, "character.control({name})"),
        }
    }
}

/// The requirement a payload implies, derived from its concrete scope.
/// Malformed payloads (an item name with no namespace) fail here with the
/// string that becomes the reject reason.
pub fn requirement_for(payload: &Payload) -> Result<Requirement, String> {
    Ok(match payload {
        Payload::SetBlock(p) => Requirement::WorldWrite(Volume::point(p.pos)),
        Payload::GetBlock(p) => Requirement::WorldRead(Volume::point(p.pos)),
        Payload::Fill(p) => Requirement::WorldWrite(Volume::new(p.min, p.max)),
        Payload::ScanRegion(p) => Requirement::WorldRead(Volume::new(p.min, p.max)),
        Payload::EntitySpawn(_) => Requirement::EntitySpawn,
        Payload::EntityQuery(p) => match p.volume {
            Some(v) => Requirement::WorldRead(v),
            None => Requirement::WorldReadAnywhere,
        },
        Payload::DefineItem(p) => Requirement::RegistryDefine(item_namespace(p)?),
        Payload::EventsSubscribe(_) => Requirement::EventsSubscribe,
        Payload::EventsPoll(_) => Requirement::EventsSubscribe,
        // Spawning a character is a dev-grant act (the character surface's
        // attach flow spawns with the surface's parent token, not the
        // session's); everything a character *does or senses* requires
        // control of exactly that character.
        Payload::SpawnCharacter(_) => Requirement::EntitySpawn,
        Payload::SetMoveIntent(p) => Requirement::CharacterControl(p.character.clone()),
        Payload::SetLook(p) => Requirement::CharacterControl(p.character.clone()),
        Payload::Jump(p) => Requirement::CharacterControl(p.character.clone()),
        Payload::CharacterPose(p) => Requirement::CharacterControl(p.character.clone()),
        Payload::SenseRaycast(p) => Requirement::CharacterControl(p.character.clone()),
        Payload::SenseSurroundings(p) => Requirement::CharacterControl(p.character.clone()),
    })
}

/// Parse the namespace out of a `namespace:path` item name.
pub fn item_namespace(item: &DefineItem) -> Result<String, String> {
    match item.name.split_once(':') {
        Some((ns, path)) if !ns.is_empty() && !path.is_empty() => Ok(ns.to_string()),
        _ => Err(format!(
            "item name `{}` is not of the form namespace:path",
            item.name
        )),
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
pub enum AttenuationError {
    #[error("requested grant is not covered by the parent token: {requested}")]
    NotCovered { requested: String },
}

/// A set of grants. Deny by default: anything not covered by a grant is
/// rejected at apply time.
#[derive(Clone, PartialEq, Eq, Debug, Default, Serialize, Deserialize)]
pub struct CapabilityToken {
    pub grants: Vec<Grant>,
}

impl CapabilityToken {
    pub fn new(grants: Vec<Grant>) -> Self {
        Self { grants }
    }

    /// An empty token (nothing allowed).
    pub fn none() -> Self {
        Self::default()
    }

    /// Does this token satisfy a payload's requirement?
    pub fn covers(&self, req: &Requirement) -> bool {
        self.grants.iter().any(|g| match (g, req) {
            (Grant::WorldRead { volume }, Requirement::WorldRead(need)) => {
                volume.is_none_or(|v| v.contains_volume(need))
            }
            (Grant::WorldRead { volume }, Requirement::WorldReadAnywhere) => volume.is_none(),
            (Grant::WorldWrite { volume }, Requirement::WorldWrite(need)) => {
                volume.is_none_or(|v| v.contains_volume(need))
            }
            (Grant::EntitySpawn, Requirement::EntitySpawn) => true,
            (Grant::RegistryDefine { namespace }, Requirement::RegistryDefine(need)) => {
                namespace == need
            }
            (Grant::EventsSubscribe, Requirement::EventsSubscribe) => true,
            (Grant::CharacterControl { character }, Requirement::CharacterControl(need)) => {
                character.as_ref().is_none_or(|c| c == need)
            }
            _ => false,
        })
    }

    /// Is every grant in `other` covered by some grant in `self`?
    /// (Attenuation validity; also the boundary-host check of claimed tokens.)
    pub fn covers_token(&self, other: &CapabilityToken) -> bool {
        other
            .grants
            .iter()
            .all(|n| self.grants.iter().any(|g| g.covers_grant(n)))
    }

    /// Derive a strictly-scoped child token. Every requested grant must be
    /// equal-or-narrower than one this token holds — a holder can never mint
    /// authority it does not have.
    pub fn attenuate(&self, grants: Vec<Grant>) -> Result<CapabilityToken, AttenuationError> {
        for g in &grants {
            if !self.grants.iter().any(|parent| parent.covers_grant(g)) {
                return Err(AttenuationError::NotCovered {
                    requested: format!("{g:?}"),
                });
            }
        }
        Ok(CapabilityToken { grants })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::payload::Vec3i;

    fn vol(a: (i64, i64, i64), b: (i64, i64, i64)) -> Volume {
        Volume::new(Vec3i::new(a.0, a.1, a.2), Vec3i::new(b.0, b.1, b.2))
    }

    #[test]
    fn empty_token_denies_everything() {
        let t = CapabilityToken::none();
        assert!(!t.covers(&Requirement::WorldRead(vol((0, 0, 0), (0, 0, 0)))));
        assert!(!t.covers(&Requirement::EntitySpawn));
        assert!(!t.covers(&Requirement::RegistryDefine("foo".into())));
        assert!(!t.covers(&Requirement::EventsSubscribe));
    }

    #[test]
    fn volume_scoped_write_covers_inside_only() {
        let t = CapabilityToken::new(vec![Grant::WorldWrite {
            volume: Some(vol((0, 0, 0), (15, 15, 15))),
        }]);
        assert!(t.covers(&Requirement::WorldWrite(vol((0, 0, 0), (15, 15, 15)))));
        assert!(t.covers(&Requirement::WorldWrite(vol((3, 3, 3), (4, 4, 4)))));
        assert!(!t.covers(&Requirement::WorldWrite(vol((3, 3, 3), (16, 4, 4)))));
        assert!(!t.covers(&Requirement::WorldWrite(vol((-1, 0, 0), (0, 0, 0)))));
        // Write does not imply read.
        assert!(!t.covers(&Requirement::WorldRead(vol((1, 1, 1), (1, 1, 1)))));
    }

    #[test]
    fn unbounded_read_covers_anywhere_bounded_does_not() {
        let unbounded = CapabilityToken::new(vec![Grant::WorldRead { volume: None }]);
        let bounded = CapabilityToken::new(vec![Grant::WorldRead {
            volume: Some(vol((0, 0, 0), (7, 7, 7))),
        }]);
        assert!(unbounded.covers(&Requirement::WorldReadAnywhere));
        assert!(!bounded.covers(&Requirement::WorldReadAnywhere));
    }

    #[test]
    fn namespace_grant_is_exact() {
        let t = CapabilityToken::new(vec![Grant::RegistryDefine {
            namespace: "foo".into(),
        }]);
        assert!(t.covers(&Requirement::RegistryDefine("foo".into())));
        assert!(!t.covers(&Requirement::RegistryDefine("bar".into())));
        assert!(!t.covers(&Requirement::RegistryDefine("foobar".into())));
    }

    #[test]
    fn attenuation_narrows_but_never_widens() {
        let parent = CapabilityToken::new(vec![
            Grant::WorldWrite {
                volume: Some(vol((0, 0, 0), (31, 31, 31))),
            },
            Grant::RegistryDefine {
                namespace: "foo".into(),
            },
        ]);
        // Narrower volume: ok.
        let child = parent
            .attenuate(vec![Grant::WorldWrite {
                volume: Some(vol((0, 0, 0), (7, 7, 7))),
            }])
            .expect("narrower volume attenuates");
        assert!(child.covers(&Requirement::WorldWrite(vol((1, 1, 1), (7, 7, 7)))));
        assert!(!child.covers(&Requirement::WorldWrite(vol((8, 0, 0), (8, 0, 0)))));
        // Wider volume: refused.
        assert!(
            parent
                .attenuate(vec![Grant::WorldWrite {
                    volume: Some(vol((0, 0, 0), (32, 31, 31))),
                }])
                .is_err()
        );
        // Unbounded from bounded: refused.
        assert!(
            parent
                .attenuate(vec![Grant::WorldWrite { volume: None }])
                .is_err()
        );
        // Different namespace: refused.
        assert!(
            parent
                .attenuate(vec![Grant::RegistryDefine {
                    namespace: "bar".into()
                }])
                .is_err()
        );
        // A grant family the parent lacks entirely: refused.
        assert!(parent.attenuate(vec![Grant::EntitySpawn]).is_err());
    }

    #[test]
    fn covers_token_is_the_boundary_check() {
        let installed = CapabilityToken::new(vec![Grant::WorldWrite {
            volume: Some(vol((0, 0, 0), (15, 15, 15))),
        }]);
        let claimed_ok = CapabilityToken::new(vec![Grant::WorldWrite {
            volume: Some(vol((2, 2, 2), (3, 3, 3))),
        }]);
        let claimed_forged = CapabilityToken::new(vec![Grant::WorldWrite { volume: None }]);
        assert!(installed.covers_token(&claimed_ok));
        assert!(!installed.covers_token(&claimed_forged));
    }

    #[test]
    fn character_control_scopes_to_one_name_and_never_widens() {
        let any = CapabilityToken::new(vec![Grant::CharacterControl { character: None }]);
        let scout = CapabilityToken::new(vec![Grant::CharacterControl {
            character: Some("scout".into()),
        }]);
        // Coverage: the broad grant covers any name; the scoped one exactly its own.
        assert!(any.covers(&Requirement::CharacterControl("scout".into())));
        assert!(any.covers(&Requirement::CharacterControl("other".into())));
        assert!(scout.covers(&Requirement::CharacterControl("scout".into())));
        assert!(!scout.covers(&Requirement::CharacterControl("other".into())));
        // Character control implies nothing else (deny by default).
        assert!(!scout.covers(&Requirement::WorldWrite(vol((0, 0, 0), (0, 0, 0)))));
        assert!(!scout.covers(&Requirement::WorldRead(vol((0, 0, 0), (0, 0, 0)))));
        assert!(!scout.covers(&Requirement::EntitySpawn));
        assert!(!scout.covers(&Requirement::EventsSubscribe));
        // Attenuation: any -> one narrows; one -> other/any refused.
        assert!(
            any.attenuate(vec![Grant::CharacterControl {
                character: Some("scout".into())
            }])
            .is_ok()
        );
        assert!(
            scout
                .attenuate(vec![Grant::CharacterControl {
                    character: Some("other".into())
                }])
                .is_err()
        );
        assert!(
            scout
                .attenuate(vec![Grant::CharacterControl { character: None }])
                .is_err()
        );
        // A token without the family cannot mint it at all.
        assert!(
            CapabilityToken::none()
                .attenuate(vec![Grant::CharacterControl {
                    character: Some("scout".into())
                }])
                .is_err()
        );
    }

    #[test]
    fn requirement_for_rejects_malformed_item_names() {
        for bad in ["no_namespace", ":path", "ns:", ""] {
            let p = Payload::DefineItem(DefineItem {
                name: bad.into(),
                display_name: "x".into(),
                description: None,
            });
            assert!(requirement_for(&p).is_err(), "{bad:?} should be malformed");
        }
    }
}
