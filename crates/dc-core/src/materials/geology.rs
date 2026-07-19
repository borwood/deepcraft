//! Geology content classes — classes-as-contracts, deterministic selection
//! (docs/design/geology.md, backbone ratified 2026-07-18).
//!
//! A **class** (e.g. `dc:stratum/clastic-fine`) is a contract: registering a
//! member into it means supplying the class parameter sheet — a formation
//! window over the core context axes (temperature, precipitation, depth), an
//! abundance weight, a habit, and hardness/erodibility. This module is the
//! *typed* model those parameters compile into; the define-time schema
//! validation and namespace ownership live in dc-api's registry surface (the
//! S5 machinery), which bridges into this model.
//!
//! **Determinism rules (non-negotiable, tested):**
//!
//! - Members are canonically ordered by namespaced id — registration order
//!   never changes worlds. [`GeologySet`] keeps every member list sorted and
//!   assigns global member indices by that order alone.
//! - Abundance is normalized within a class: adding a member *diversifies*
//!   its class share, it never inflates it. Selection weights divide by the
//!   class abundance sum.
//! - Selection = fitness(formation-condition distance) × normalized abundance
//!   × seed. This crate owns no entropy (project rule: all entropy flows from
//!   caller seeds), so [`GeoClass::select`] takes the caller's uniform draw
//!   `u ∈ [0, 1)` — worldgen supplies it via addressed hashing.

use std::collections::BTreeMap;

use super::{MaterialId, MaterialProps};

/// v1 class roster (DECIDED 2026-07-18: minimal-but-complete) plus the 3d
/// accessory-inclusion class (docs/design/geology.md § roster/inclusions).
pub const CLASS_CLASTIC_FINE: &str = "dc:stratum/clastic-fine";
pub const CLASS_CLASTIC_COARSE: &str = "dc:stratum/clastic-coarse";
pub const CLASS_IGNEOUS_INTRUSIVE: &str = "dc:stratum/igneous-intrusive";
pub const CLASS_IGNEOUS_EXTRUSIVE: &str = "dc:stratum/igneous-extrusive";
pub const CLASS_ORE_PLACER: &str = "dc:ore/placer";
/// Accessory minerals that ride the pore slots of a host igneous rock
/// (olivine in basalt/gabbro) — the inclusion-as-pore-partial representation
/// (DECIDED 2026-07-19). Province/depth-driven like its igneous host; it
/// never reads the weather.
pub const CLASS_ACCESSORY_MAFIC: &str = "dc:accessory/mafic";

/// The vanilla classes, in canonical (sorted) order.
pub fn v1_classes() -> [&'static str; 6] {
    let mut c = [
        CLASS_CLASTIC_FINE,
        CLASS_CLASTIC_COARSE,
        CLASS_IGNEOUS_INTRUSIVE,
        CLASS_IGNEOUS_EXTRUSIVE,
        CLASS_ORE_PLACER,
        CLASS_ACCESSORY_MAFIC,
    ];
    c.sort_unstable();
    c
}

/// Whether a class binds its member fitness to the **surface weather** of the
/// deposition epoch. Clastic sediment and surficial placers do (year-zero
/// climate is ratified-correct for the veneer, geology.md § formation
/// context); igneous and its accessories do **not** — their fitness is
/// province/depth-driven, so a granite never reads the weather. dc-api's
/// class contract omits the `temp_c`/`precip` params for the latter, and the
/// worldgen igneous windows leave those axes unbounded, so weather cannot
/// enter their selection.
pub fn class_reads_climate(class: &str) -> bool {
    !matches!(
        class,
        CLASS_IGNEOUS_INTRUSIVE | CLASS_IGNEOUS_EXTRUSIVE | CLASS_ACCESSORY_MAFIC
    )
}

/// An inclusive window on one context axis plus the whole formation-condition
/// window a member forms under. Outside the window fitness decays smoothly
/// (never a hard zero), so a class always has *some* answer where any member
/// is remotely plausible, and boundaries in context space do not produce
/// discontinuous world content.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct FormationWindow {
    /// Formation temperature window, °C (climate-at-deposition axis).
    pub temp_c: (f64, f64),
    /// Formation precipitation window, normalized 0..1.
    pub precip: (f64, f64),
    /// Emplacement depth window below the surface, meters.
    pub depth_m: (f64, f64),
}

impl FormationWindow {
    /// The everything-goes window (fitness 1 everywhere).
    pub const ANY: FormationWindow = FormationWindow {
        temp_c: (f64::NEG_INFINITY, f64::INFINITY),
        precip: (f64::NEG_INFINITY, f64::INFINITY),
        depth_m: (f64::NEG_INFINITY, f64::INFINITY),
    };

    /// A province/depth-driven igneous window: the `temp_c`/`precip` axes are
    /// left unbounded (fitness 1 regardless of weather), so an igneous member
    /// **never reads the surface weather** — only its emplacement `depth_m`
    /// (and the pass's province gate) steer selection (geology.md § formation
    /// context, the 2026-07-19 shim correction).
    pub const fn igneous(depth_m: (f64, f64)) -> FormationWindow {
        FormationWindow {
            temp_c: (f64::NEG_INFINITY, f64::INFINITY),
            precip: (f64::NEG_INFINITY, f64::INFINITY),
            depth_m,
        }
    }

    fn axes_valid(&self) -> bool {
        let ok = |(lo, hi): (f64, f64)| !lo.is_nan() && !hi.is_nan() && lo <= hi;
        ok(self.temp_c) && ok(self.precip) && ok(self.depth_m)
    }
}

/// The queryable core axes a formation window binds to (core-axes-only for
/// v1; plugin-published axes are a later contract).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct FormationContext {
    pub temp_c: f64,
    pub precip: f64,
    pub depth_m: f64,
}

/// Occurrence habit of a class member.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GeoHabit {
    /// Laterally continuous stratum.
    Blanket,
    /// Bounded body (pluton, lens).
    Lens,
    /// Disseminated grains inside a host stratum (placer ore).
    Grain,
}

impl GeoHabit {
    pub fn as_str(self) -> &'static str {
        match self {
            GeoHabit::Blanket => "blanket",
            GeoHabit::Lens => "lens",
            GeoHabit::Grain => "grain",
        }
    }

    pub fn parse(s: &str) -> Option<GeoHabit> {
        Some(match s {
            "blanket" => GeoHabit::Blanket,
            "lens" => GeoHabit::Lens,
            "grain" => GeoHabit::Grain,
            _ => None?,
        })
    }
}

/// One registered class member: the class parameter sheet, filled in.
#[derive(Clone, PartialEq, Debug)]
pub struct GeoMemberDef {
    /// Namespaced member id, e.g. `dc:geo/mudstone`. The canonical sort key.
    pub id: String,
    /// The class this member implements, e.g. `dc:stratum/clastic-fine`.
    pub class: String,
    /// The granular material this member deposits as.
    pub material: MaterialId,
    pub window: FormationWindow,
    /// Relative abundance weight (> 0); normalized within the class.
    pub abundance: f64,
    pub habit: GeoHabit,
    /// 0..1, resistance to mechanical breakdown.
    pub hardness: f64,
    /// 0..1, susceptibility to erosion (hardness→erodibility coupling is an
    /// open design question; both are carried so passes can bind to either).
    pub erodibility: f64,
}

/// Errors registering members into a [`GeologySet`].
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum GeologyError {
    #[error("member id `{0}` is not of the form namespace:path")]
    BadId(String),
    #[error("duplicate member id `{0}`")]
    DuplicateMember(String),
    #[error("unknown class `{class}` for member `{member}`")]
    UnknownClass { class: String, member: String },
    #[error("duplicate class `{0}`")]
    DuplicateClass(String),
    #[error("member `{member}` has invalid parameters: {reason}")]
    BadParams { member: String, reason: String },
}

/// Stable index of a member across the whole set, assigned by canonical
/// (id-sorted) order — never by registration order. Written into strata
/// records, so its stability under registration permutation is exactly the
/// registration-order-independence guarantee.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct GeoMemberIdx(pub u16);

/// A compiled, immutable view of one class: members in canonical order with
/// the class abundance sum precomputed.
#[derive(Clone, Debug)]
pub struct GeoClass {
    id: String,
    /// Global member indices, in canonical (member-id) order.
    members: Vec<GeoMemberIdx>,
    abundance_sum: f64,
}

impl GeoClass {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn members(&self) -> &[GeoMemberIdx] {
        &self.members
    }
}

/// The registered geology content: classes and their members, compiled to
/// canonical order. Build with [`GeologySetBuilder`]; the built set is
/// immutable, so member indices cannot shift under later registration.
#[derive(Clone, Debug, Default)]
pub struct GeologySet {
    /// All members, sorted by id; index = [`GeoMemberIdx`].
    members: Vec<GeoMemberDef>,
    classes: BTreeMap<String, GeoClass>,
}

/// Builder: declare classes, register members in any order.
#[derive(Clone, Debug, Default)]
pub struct GeologySetBuilder {
    classes: Vec<String>,
    members: Vec<GeoMemberDef>,
}

impl GeologySetBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Declare a class id. Classes are contracts; a member naming an
    /// undeclared class is rejected at build.
    pub fn declare_class(&mut self, id: &str) -> Result<&mut Self, GeologyError> {
        if !valid_namespaced(id) {
            return Err(GeologyError::BadId(id.to_string()));
        }
        if self.classes.iter().any(|c| c == id) {
            return Err(GeologyError::DuplicateClass(id.to_string()));
        }
        self.classes.push(id.to_string());
        Ok(self)
    }

    /// Register a member (any order; canonical order is imposed at build).
    pub fn add_member(&mut self, def: GeoMemberDef) -> Result<&mut Self, GeologyError> {
        if !valid_namespaced(&def.id) {
            return Err(GeologyError::BadId(def.id));
        }
        if self.members.iter().any(|m| m.id == def.id) {
            return Err(GeologyError::DuplicateMember(def.id));
        }
        if !self.classes.contains(&def.class) {
            return Err(GeologyError::UnknownClass {
                class: def.class,
                member: def.id,
            });
        }
        let bad = |reason: &str| GeologyError::BadParams {
            member: def.id.clone(),
            reason: reason.to_string(),
        };
        if !(def.abundance.is_finite() && def.abundance > 0.0) {
            return Err(bad("abundance must be finite and > 0"));
        }
        if !(0.0..=1.0).contains(&def.hardness) || !(0.0..=1.0).contains(&def.erodibility) {
            return Err(bad("hardness/erodibility must be in [0, 1]"));
        }
        if !def.window.axes_valid() {
            return Err(bad("formation window axes must satisfy min <= max"));
        }
        self.members.push(def);
        Ok(self)
    }

    /// Compile to the immutable, canonically ordered set.
    pub fn build(mut self) -> GeologySet {
        // Canonical order: by namespaced id, never registration order.
        self.members.sort_by(|a, b| a.id.cmp(&b.id));
        let mut classes: BTreeMap<String, GeoClass> = self
            .classes
            .into_iter()
            .map(|id| {
                (
                    id.clone(),
                    GeoClass {
                        id,
                        members: Vec::new(),
                        abundance_sum: 0.0,
                    },
                )
            })
            .collect();
        for (i, m) in self.members.iter().enumerate() {
            let class = classes.get_mut(&m.class).expect("validated at add");
            class.members.push(GeoMemberIdx(i as u16));
            class.abundance_sum += m.abundance;
        }
        GeologySet {
            members: self.members,
            classes,
        }
    }
}

impl GeologySet {
    pub fn builder() -> GeologySetBuilder {
        GeologySetBuilder::new()
    }

    /// All members in canonical (id) order.
    pub fn members(&self) -> &[GeoMemberDef] {
        &self.members
    }

    pub fn member(&self, idx: GeoMemberIdx) -> &GeoMemberDef {
        &self.members[idx.0 as usize]
    }

    pub fn member_index(&self, id: &str) -> Option<GeoMemberIdx> {
        self.members
            .binary_search_by(|m| m.id.as_str().cmp(id))
            .ok()
            .map(|i| GeoMemberIdx(i as u16))
    }

    pub fn class(&self, id: &str) -> Option<&GeoClass> {
        self.classes.get(id)
    }

    /// Classes in canonical (id) order.
    pub fn classes(&self) -> impl Iterator<Item = &GeoClass> {
        self.classes.values()
    }

    /// Deterministic member selection: fitness(formation-condition distance)
    /// × normalized abundance × the caller's seed draw `u ∈ [0, 1)`.
    ///
    /// Iteration is in canonical member order and weights are divided by the
    /// class abundance sum, so (a) registration order cannot change the
    /// answer and (b) adding a member redistributes its class's share rather
    /// than inflating it. Returns `None` only when the class is empty or
    /// every member's fitness underflows to zero.
    pub fn select(
        &self,
        class: &str,
        ctx: &FormationContext,
        u: f64,
    ) -> Option<(GeoMemberIdx, &GeoMemberDef)> {
        let class = self.classes.get(class)?;
        if class.members.is_empty() || class.abundance_sum <= 0.0 {
            return None;
        }
        let mut total = 0.0f64;
        let mut weights = Vec::with_capacity(class.members.len());
        for &idx in &class.members {
            let m = self.member(idx);
            let w = fitness(&m.window, ctx) * (m.abundance / class.abundance_sum);
            weights.push(w);
            total += w;
        }
        if total <= 0.0 {
            return None;
        }
        let threshold = u.clamp(0.0, 1.0 - f64::EPSILON) * total;
        let mut acc = 0.0f64;
        for (&idx, &w) in class.members.iter().zip(&weights) {
            acc += w;
            if threshold < acc {
                return Some((idx, self.member(idx)));
            }
        }
        // Float slack: the last member with nonzero weight takes the tail.
        let last = *class
            .members
            .iter()
            .zip(&weights)
            .filter(|&(_, &w)| w > 0.0)
            .map(|(idx, _)| idx)
            .next_back()?;
        Some((last, self.member(last)))
    }
}

/// Fitness of a context against a formation window: 1 inside, smooth
/// exponential falloff outside (soft distance in units of the window's own
/// width, floored so degenerate windows still decay finitely). Pure IEEE
/// arithmetic on canonical inputs — bit-stable across platforms.
pub fn fitness(window: &FormationWindow, ctx: &FormationContext) -> f64 {
    axis_fitness(window.temp_c, ctx.temp_c, 4.0)
        * axis_fitness(window.precip, ctx.precip, 0.08)
        * axis_fitness(window.depth_m, ctx.depth_m, 20.0)
}

fn axis_fitness((lo, hi): (f64, f64), x: f64, min_soft: f64) -> f64 {
    if x >= lo && x <= hi {
        return 1.0;
    }
    let soft = (0.25 * (hi - lo)).max(min_soft);
    let d = if x < lo { lo - x } else { x - hi };
    if !soft.is_finite() {
        return 1.0; // an unbounded axis accepts everything
    }
    (-d / soft).exp()
}

fn valid_namespaced(id: &str) -> bool {
    matches!(id.split_once(':'), Some((ns, path)) if !ns.is_empty() && !path.is_empty())
}

/// Settle-energy threshold for graded (energy-decay) settling, derived from
/// the property sheet: grains settle out of a flow when its energy drops
/// below `sqrt(grain_size_mm × specific_gravity)`. This reproduces the S8
/// alluvial ordering (gravel > sand > silt > clay) from properties instead of
/// a hand table, and gives the placer mechanism for free: a dense ore grain's
/// threshold lands in the coarse band despite its small size, so it settles
/// with the gravel — which is what a placer *is*.
pub fn settle_energy(props: &MaterialProps) -> f64 {
    (f64::from(props.grain_size_mm) * f64::from(props.density_kg_m3) / 1000.0).sqrt()
}

/// The vanilla v1 geology content (docs/design/geology.md § v1 content):
/// clastic fine + coarse, igneous intrusive + extrusive, one placer ore.
/// This is data, not code — dc-api's vanilla content pack is generated from
/// this set so the two can never drift (the documented bridge from
/// registry-table entries to slice-1 class defs).
pub fn vanilla() -> GeologySet {
    let mut b = GeologySet::builder();
    for class in v1_classes() {
        b.declare_class(class).expect("vanilla classes are valid");
    }
    let members = vanilla_members();
    for m in members {
        b.add_member(m).expect("vanilla members are valid");
    }
    b.build()
}

/// The vanilla member defs (pre-canonicalization; any order is fine).
pub fn vanilla_members() -> Vec<GeoMemberDef> {
    vec![
        GeoMemberDef {
            id: "dc:geo/mudstone".into(),
            class: CLASS_CLASTIC_FINE.into(),
            material: MaterialId::MUDSTONE,
            window: FormationWindow {
                temp_c: (-5.0, 35.0),
                precip: (0.15, 1.0),
                depth_m: (0.0, 80.0),
            },
            abundance: 1.0,
            habit: GeoHabit::Blanket,
            hardness: 0.35,
            erodibility: 0.7,
        },
        GeoMemberDef {
            id: "dc:geo/sandstone".into(),
            class: CLASS_CLASTIC_COARSE.into(),
            material: MaterialId::SANDSTONE,
            window: FormationWindow {
                temp_c: (-10.0, 40.0),
                precip: (0.05, 1.0),
                depth_m: (0.0, 100.0),
            },
            abundance: 1.0,
            habit: GeoHabit::Blanket,
            hardness: 0.5,
            erodibility: 0.5,
        },
        // --- igneous: province/depth-driven, weather-blind (unbounded
        // temp_c/precip via FormationWindow::igneous). ---
        GeoMemberDef {
            id: "dc:geo/granite".into(),
            class: CLASS_IGNEOUS_INTRUSIVE.into(),
            material: MaterialId::GRANITE,
            window: FormationWindow::igneous((120.0, 40_000.0)),
            abundance: 1.0,
            habit: GeoHabit::Lens,
            hardness: 0.9,
            erodibility: 0.15,
        },
        GeoMemberDef {
            // A second intrusive (roster proof): a shallower-seated pluton, so
            // depth — not weather — differentiates it from granite.
            id: "dc:geo/diorite".into(),
            class: CLASS_IGNEOUS_INTRUSIVE.into(),
            material: MaterialId::DIORITE,
            window: FormationWindow::igneous((90.0, 8_000.0)),
            abundance: 0.6,
            habit: GeoHabit::Lens,
            hardness: 0.88,
            erodibility: 0.16,
        },
        GeoMemberDef {
            id: "dc:geo/basalt".into(),
            class: CLASS_IGNEOUS_EXTRUSIVE.into(),
            material: MaterialId::BASALT,
            window: FormationWindow::igneous((0.0, 60.0)),
            abundance: 1.0,
            habit: GeoHabit::Blanket,
            hardness: 0.85,
            erodibility: 0.2,
        },
        GeoMemberDef {
            // A second extrusive (roster proof): intermediate lava.
            id: "dc:geo/andesite".into(),
            class: CLASS_IGNEOUS_EXTRUSIVE.into(),
            material: MaterialId::ANDESITE,
            window: FormationWindow::igneous((0.0, 40.0)),
            abundance: 0.7,
            habit: GeoHabit::Blanket,
            hardness: 0.82,
            erodibility: 0.22,
        },
        GeoMemberDef {
            id: "dc:geo/gold-dust".into(),
            class: CLASS_ORE_PLACER.into(),
            material: MaterialId::GOLD_DUST,
            window: FormationWindow {
                temp_c: (-40.0, 50.0),
                precip: (0.0, 1.0),
                depth_m: (0.0, 30.0),
            },
            abundance: 1.0,
            habit: GeoHabit::Grain,
            hardness: 0.4,
            erodibility: 0.3,
        },
        // --- second fine + coarse clastic members (roster proof). ---
        GeoMemberDef {
            id: "dc:geo/siltstone".into(),
            class: CLASS_CLASTIC_FINE.into(),
            material: MaterialId::SILTSTONE,
            window: FormationWindow {
                temp_c: (-5.0, 30.0),
                precip: (0.10, 0.9),
                depth_m: (0.0, 80.0),
            },
            abundance: 0.7,
            habit: GeoHabit::Blanket,
            hardness: 0.32,
            erodibility: 0.72,
        },
        GeoMemberDef {
            id: "dc:geo/conglomerate".into(),
            class: CLASS_CLASTIC_COARSE.into(),
            material: MaterialId::CONGLOMERATE,
            window: FormationWindow {
                temp_c: (-10.0, 40.0),
                precip: (0.05, 1.0),
                depth_m: (0.0, 100.0),
            },
            abundance: 0.6,
            habit: GeoHabit::Blanket,
            hardness: 0.55,
            erodibility: 0.45,
        },
        // --- accessory: rides the host igneous rock's pores (province/depth
        // -driven, weather-blind). ---
        GeoMemberDef {
            id: "dc:geo/olivine".into(),
            class: CLASS_ACCESSORY_MAFIC.into(),
            material: MaterialId::OLIVINE,
            window: FormationWindow::igneous((0.0, 40_000.0)),
            abundance: 1.0,
            habit: GeoHabit::Grain,
            hardness: 0.7,
            erodibility: 0.25,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(temp_c: f64, precip: f64, depth_m: f64) -> FormationContext {
        FormationContext {
            temp_c,
            precip,
            depth_m,
        }
    }

    fn test_member(id: &str, class: &str, abundance: f64) -> GeoMemberDef {
        GeoMemberDef {
            id: id.into(),
            class: class.into(),
            material: MaterialId::SILT,
            window: FormationWindow::ANY,
            abundance,
            habit: GeoHabit::Blanket,
            hardness: 0.5,
            erodibility: 0.5,
        }
    }

    #[test]
    fn vanilla_builds_and_selects_each_class() {
        let set = vanilla();
        assert_eq!(set.members().len(), 10);
        // Warm wet surface: fine clastic answers (mudstone or siltstone).
        let (_, m) = set
            .select(CLASS_CLASTIC_FINE, &ctx(18.0, 0.6, 3.0), 0.1)
            .expect("fine clastic member");
        assert!(matches!(
            m.material,
            MaterialId::MUDSTONE | MaterialId::SILTSTONE
        ));
        // Deep intrusive: an igneous member answers regardless of the (unused)
        // weather axes.
        let (_, m) = set
            .select(CLASS_IGNEOUS_INTRUSIVE, &ctx(10.0, 0.3, 400.0), 0.1)
            .expect("intrusive member");
        assert!(matches!(
            m.material,
            MaterialId::GRANITE | MaterialId::DIORITE
        ));
        // The accessory class answers under province/depth context.
        let (_, m) = set
            .select(CLASS_ACCESSORY_MAFIC, &ctx(0.0, 0.0, 50.0), 0.5)
            .expect("accessory member");
        assert_eq!(m.material, MaterialId::OLIVINE);
        assert!(
            set.select("dc:stratum/nope", &ctx(0.0, 0.0, 0.0), 0.5)
                .is_none()
        );
    }

    #[test]
    fn igneous_selection_never_reads_the_weather() {
        // The formation-context shim correction (geology.md 2026-07-19): an
        // igneous member's fitness is depth-driven only — swinging temp/precip
        // across their whole range cannot change the selection at a fixed depth.
        let set = vanilla();
        for class in [CLASS_IGNEOUS_INTRUSIVE, CLASS_IGNEOUS_EXTRUSIVE] {
            for depth in [5.0, 60.0, 300.0, 2_000.0] {
                for &u in &[0.05, 0.35, 0.65, 0.95] {
                    let hot_wet = set.select(class, &ctx(45.0, 1.0, depth), u).map(|(i, _)| i);
                    let cold_dry = set
                        .select(class, &ctx(-40.0, 0.0, depth), u)
                        .map(|(i, _)| i);
                    assert_eq!(
                        hot_wet, cold_dry,
                        "igneous class {class} at depth {depth} read the weather"
                    );
                }
            }
        }
    }

    #[test]
    fn member_indices_are_registration_order_independent() {
        let members = vanilla_members();
        let build = |order: &[usize], classes: &[&str]| {
            let mut b = GeologySet::builder();
            for c in classes {
                b.declare_class(c).unwrap();
            }
            for &i in order {
                b.add_member(members[i].clone()).unwrap();
            }
            b.build()
        };
        // Classes declared in a different order too.
        let a = build(
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            &[
                CLASS_CLASTIC_FINE,
                CLASS_CLASTIC_COARSE,
                CLASS_IGNEOUS_INTRUSIVE,
                CLASS_IGNEOUS_EXTRUSIVE,
                CLASS_ORE_PLACER,
                CLASS_ACCESSORY_MAFIC,
            ],
        );
        let z = build(
            &[9, 7, 5, 3, 1, 8, 6, 4, 2, 0],
            &[
                CLASS_ACCESSORY_MAFIC,
                CLASS_ORE_PLACER,
                CLASS_IGNEOUS_EXTRUSIVE,
                CLASS_IGNEOUS_INTRUSIVE,
                CLASS_CLASTIC_COARSE,
                CLASS_CLASTIC_FINE,
            ],
        );
        assert_eq!(a.members(), z.members(), "canonical order must win");
        for m in a.members() {
            assert_eq!(a.member_index(&m.id), z.member_index(&m.id));
        }
        // And selection agrees everywhere we probe.
        for k in 0..32 {
            let c = ctx(
                -10.0 + f64::from(k) * 2.0,
                f64::from(k) / 32.0,
                f64::from(k) * 12.0,
            );
            let u = f64::from(k) / 32.0;
            for class in v1_classes() {
                assert_eq!(
                    a.select(class, &c, u).map(|(i, _)| i),
                    z.select(class, &c, u).map(|(i, _)| i),
                );
            }
        }
    }

    #[test]
    fn abundance_is_normalized_within_class() {
        // One member: it takes the whole class share at any abundance.
        let mut b = GeologySet::builder();
        b.declare_class("t:class/a").unwrap();
        b.add_member(test_member("t:m/solo", "t:class/a", 7.0))
            .unwrap();
        let solo = b.build();
        // Adding a second member redistributes, never inflates: with equal
        // fitness the shares follow relative abundance and sum to 1.
        let mut b = GeologySet::builder();
        b.declare_class("t:class/a").unwrap();
        b.add_member(test_member("t:m/solo", "t:class/a", 7.0))
            .unwrap();
        b.add_member(test_member("t:m/new", "t:class/a", 21.0))
            .unwrap();
        let duo = b.build();

        let c = ctx(10.0, 0.5, 5.0);
        let n = 4096;
        let mut solo_hits = 0usize;
        let mut duo_solo_hits = 0usize;
        for k in 0..n {
            let u = (k as f64 + 0.5) / n as f64;
            if solo.select("t:class/a", &c, u).is_some() {
                solo_hits += 1;
            }
            let (_, m) = duo.select("t:class/a", &c, u).expect("duo selects");
            if m.id == "t:m/solo" {
                duo_solo_hits += 1;
            }
        }
        // The class always answers (its total share is unchanged)…
        assert_eq!(solo_hits, n);
        // …but the old member's share dropped to abundance/(sum) = 1/4.
        let share = duo_solo_hits as f64 / n as f64;
        assert!(
            (share - 0.25).abs() < 0.01,
            "expected ~0.25 share for the diluted member, got {share}"
        );
    }

    #[test]
    fn selection_is_deterministic_and_seed_sensitive() {
        let set = vanilla();
        let c = ctx(15.0, 0.4, 10.0);
        let a = set.select(CLASS_CLASTIC_FINE, &c, 0.37).map(|(i, _)| i);
        let b = set.select(CLASS_CLASTIC_FINE, &c, 0.37).map(|(i, _)| i);
        assert_eq!(a, b);
        // With a multi-member class, different u picks different members.
        let mut builder = GeologySet::builder();
        builder.declare_class("t:class/a").unwrap();
        builder
            .add_member(test_member("t:m/a", "t:class/a", 1.0))
            .unwrap();
        builder
            .add_member(test_member("t:m/b", "t:class/a", 1.0))
            .unwrap();
        let set = builder.build();
        let lo = set.select("t:class/a", &c, 0.1).unwrap().1.id.clone();
        let hi = set.select("t:class/a", &c, 0.9).unwrap().1.id.clone();
        assert_ne!(lo, hi, "seed draw must steer selection");
    }

    #[test]
    fn fitness_prefers_the_window_and_decays_outside() {
        let w = FormationWindow {
            temp_c: (0.0, 20.0),
            precip: (0.2, 0.8),
            depth_m: (0.0, 50.0),
        };
        assert_eq!(fitness(&w, &ctx(10.0, 0.5, 25.0)), 1.0);
        let near = fitness(&w, &ctx(25.0, 0.5, 25.0));
        let far = fitness(&w, &ctx(40.0, 0.5, 25.0));
        assert!(near < 1.0 && far < near && far > 0.0);
        // Unbounded windows accept everything.
        assert_eq!(fitness(&FormationWindow::ANY, &ctx(1e6, -5.0, 1e9)), 1.0);
    }

    #[test]
    fn builder_rejects_bad_defs() {
        let mut b = GeologySet::builder();
        assert_eq!(
            b.declare_class("no-namespace").unwrap_err(),
            GeologyError::BadId("no-namespace".into())
        );
        b.declare_class("t:class/a").unwrap();
        assert!(matches!(
            b.declare_class("t:class/a").unwrap_err(),
            GeologyError::DuplicateClass(_)
        ));
        assert!(matches!(
            b.add_member(test_member("t:m/a", "t:class/missing", 1.0))
                .unwrap_err(),
            GeologyError::UnknownClass { .. }
        ));
        assert!(matches!(
            b.add_member(test_member("t:m/a", "t:class/a", 0.0))
                .unwrap_err(),
            GeologyError::BadParams { .. }
        ));
        b.add_member(test_member("t:m/a", "t:class/a", 1.0))
            .unwrap();
        assert!(matches!(
            b.add_member(test_member("t:m/a", "t:class/a", 1.0))
                .unwrap_err(),
            GeologyError::DuplicateMember(_)
        ));
        let mut bad_window = test_member("t:m/w", "t:class/a", 1.0);
        bad_window.window.temp_c = (10.0, -10.0);
        assert!(matches!(
            b.add_member(bad_window).unwrap_err(),
            GeologyError::BadParams { .. }
        ));
    }

    #[test]
    fn settle_energy_orders_grains_and_places_the_ore_with_the_gravel() {
        let e = |m: MaterialId| settle_energy(m.props());
        // The S8 alluvial ordering, from properties.
        assert!(e(MaterialId::GRAVEL) > e(MaterialId::SAND));
        assert!(e(MaterialId::SAND) > e(MaterialId::SILT));
        assert!(e(MaterialId::SILT) > e(MaterialId::CLAY));
        // The placer anomaly: gold-dust is far finer than gravel but settles
        // between sand and gravel — density puts it in the coarse band.
        assert!(e(MaterialId::GOLD_DUST) > e(MaterialId::SAND));
        assert!(e(MaterialId::GOLD_DUST) < e(MaterialId::GRAVEL));
        assert!(
            MaterialId::GOLD_DUST.props().grain_size_mm < MaterialId::GRAVEL.props().grain_size_mm
        );
    }
}
