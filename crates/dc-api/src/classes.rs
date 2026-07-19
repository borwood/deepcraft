//! Content-class registry: material classes as contracts (geology backbone,
//! docs/design/geology.md; second registry beside the command registry).
//!
//! Two define verbs, both ordinary [`crate::schema`] CommandSpecs (so every
//! MCP surface grows the tools automatically):
//!
//! - `dc:registry/define_content_class` — declares a class **and its
//!   parameter contract** (a list of [`ParamSpec`]).
//! - `dc:registry/define_class_member` — registers a member into a class,
//!   carrying `params: Vec<ParamEntry>` **validated at define time against
//!   the class's contract** ([`validate_params`]).
//!
//! **The Payload opening (needs ratification).** The `Payload` union stays a
//! closed enum, but the member-define payload's *contents* are open: an
//! ordered list of `(name, value)` pairs over a small closed value vocabulary
//! ([`ParamValue`]), whose legal shape is data (the registered class
//! contract) rather than a compile-time struct. This is the smallest honest
//! realization of S5's "schema-registered payloads" open path: new classes
//! need zero new Rust types or Payload variants, wire encoding stays
//! postcard-positional-safe (every field always serialized, no maps), and
//! validation is exactly as strong as a typed payload because it runs at
//! define time against a schema the registry owns.
//!
//! Namespace ownership is the S5 rule unchanged: defining `foo:*` (classes
//! or members) requires a `registry.define(foo)` grant. Members may join
//! classes in *other* namespaces — registering a material into
//! `dc:stratum/clastic-fine` from a plugin namespace is the point.
//!
//! The **bridge** to the typed geology model (dc-core
//! `materials::geology`): [`geology_param_specs`] is the contract the five
//! v1 geology classes declare, [`vanilla_geology_pack`] emits the vanilla
//! content as a recorded command batch ("vanilla is the first pack"),
//! generated *from* `dc_core::materials::geology::vanilla_members()` so the
//! two can never drift, and [`geology_set_from_defs`] compiles registered
//! defs back into a [`GeologySet`] (canonical order, normalized abundance —
//! the determinism rules live there).

use dc_core::materials::geology::{
    self, FormationWindow, GeoHabit, GeoMemberDef, GeologySet,
};
use dc_core::MaterialId;
use serde::{Deserialize, Serialize};

use crate::envelope::{ConsumerId, Tick};
use crate::payload::{DefineClassMember, DefineContentClass, Payload};

/// The kind (and bounds) one parameter of a class contract accepts.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ParamKind {
    /// A finite number in `[min, max]`.
    Number { min: f64, max: f64 },
    /// An ordered pair `(lo, hi)`, `lo <= hi`, both in `[min, max]`
    /// (formation windows).
    Range { min: f64, max: f64 },
    /// Free text.
    Text,
    /// Text naming a registered dc-core material.
    MaterialName,
    /// Text drawn from a closed option list.
    Choice { options: Vec<String> },
}

/// One parameter in a class contract.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ParamSpec {
    pub name: String,
    pub kind: ParamKind,
    pub required: bool,
}

/// One supplied parameter value (the closed value vocabulary).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ParamValue {
    Number(f64),
    Range(f64, f64),
    Text(String),
}

/// One `(name, value)` pair in a member definition.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ParamEntry {
    pub name: String,
    pub value: ParamValue,
}

/// A registered content class, as stored by the host.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ContentClassDef {
    pub name: String,
    pub doc: String,
    pub params: Vec<ParamSpec>,
    pub defined_tick: Tick,
    pub defined_by: ConsumerId,
}

/// A registered class member, as stored by the host (schema-validated).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ClassMemberDef {
    pub name: String,
    pub class: String,
    pub params: Vec<ParamEntry>,
    pub defined_tick: Tick,
    pub defined_by: ConsumerId,
}

/// Resolve a material name (`MaterialProps::name`) to its id.
pub fn material_by_name(name: &str) -> Option<MaterialId> {
    MaterialId::all().find(|m| m.props().name == name)
}

/// Validate a class contract itself (at `define_content_class` time).
pub fn validate_class_contract(params: &[ParamSpec]) -> Result<(), String> {
    for (i, p) in params.iter().enumerate() {
        if p.name.is_empty() {
            return Err("parameter names must be non-empty".into());
        }
        if params[..i].iter().any(|q| q.name == p.name) {
            return Err(format!("duplicate parameter `{}`", p.name));
        }
        match &p.kind {
            ParamKind::Number { min, max } | ParamKind::Range { min, max } => {
                if !(min.is_finite() && max.is_finite() && min <= max) {
                    return Err(format!("parameter `{}` has invalid bounds", p.name));
                }
            }
            ParamKind::Choice { options } => {
                if options.is_empty() {
                    return Err(format!("parameter `{}` has no options", p.name));
                }
            }
            ParamKind::Text | ParamKind::MaterialName => {}
        }
    }
    Ok(())
}

/// Validate supplied member params against a class contract (at
/// `define_class_member` time). Classes are contracts: unknown or duplicate
/// names, missing required params, kind mismatches, and out-of-bounds values
/// all reject the define.
pub fn validate_params(contract: &[ParamSpec], supplied: &[ParamEntry]) -> Result<(), String> {
    for (i, e) in supplied.iter().enumerate() {
        if supplied[..i].iter().any(|q| q.name == e.name) {
            return Err(format!("duplicate parameter `{}`", e.name));
        }
        let Some(spec) = contract.iter().find(|s| s.name == e.name) else {
            return Err(format!("parameter `{}` is not in the class contract", e.name));
        };
        match (&spec.kind, &e.value) {
            (ParamKind::Number { min, max }, ParamValue::Number(v)) => {
                if !(v.is_finite() && *v >= *min && *v <= *max) {
                    return Err(format!(
                        "parameter `{}` = {v} outside [{min}, {max}]",
                        e.name
                    ));
                }
            }
            (ParamKind::Range { min, max }, ParamValue::Range(lo, hi)) => {
                if !(lo.is_finite() && hi.is_finite() && lo <= hi && *lo >= *min && *hi <= *max) {
                    return Err(format!(
                        "parameter `{}` = ({lo}, {hi}) is not an ordered pair inside [{min}, {max}]",
                        e.name
                    ));
                }
            }
            (ParamKind::Text, ParamValue::Text(_)) => {}
            (ParamKind::MaterialName, ParamValue::Text(t)) => {
                if material_by_name(t).is_none() {
                    return Err(format!(
                        "parameter `{}`: `{t}` is not a registered material",
                        e.name
                    ));
                }
            }
            (ParamKind::Choice { options }, ParamValue::Text(t)) => {
                if !options.iter().any(|o| o == t) {
                    return Err(format!(
                        "parameter `{}`: `{t}` is not one of {options:?}",
                        e.name
                    ));
                }
            }
            _ => {
                return Err(format!(
                    "parameter `{}` has the wrong value kind for the contract",
                    e.name
                ));
            }
        }
    }
    for spec in contract.iter().filter(|s| s.required) {
        if !supplied.iter().any(|e| e.name == spec.name) {
            return Err(format!("required parameter `{}` missing", spec.name));
        }
    }
    Ok(())
}

// ------------------------------------------------- the geology contract --

fn num(name: &str, min: f64, max: f64) -> ParamSpec {
    ParamSpec {
        name: name.into(),
        kind: ParamKind::Number { min, max },
        required: true,
    }
}

fn range(name: &str, min: f64, max: f64) -> ParamSpec {
    ParamSpec {
        name: name.into(),
        kind: ParamKind::Range { min, max },
        required: true,
    }
}

/// The parameter contract every v1 geology class declares
/// (docs/design/geology.md: formation T/P window, abundance weight, habit,
/// hardness/erodibility — over the core context axes only).
pub fn geology_param_specs() -> Vec<ParamSpec> {
    vec![
        ParamSpec {
            name: "material".into(),
            kind: ParamKind::MaterialName,
            required: true,
        },
        range("temp_c", -80.0, 80.0),
        range("precip", 0.0, 1.0),
        range("depth_m", 0.0, 100_000.0),
        num("abundance", 1e-9, 1e9),
        ParamSpec {
            name: "habit".into(),
            kind: ParamKind::Choice {
                options: vec!["blanket".into(), "lens".into(), "grain".into()],
            },
            required: true,
        },
        num("hardness", 0.0, 1.0),
        num("erodibility", 0.0, 1.0),
    ]
}

/// Convert a typed geology member def into the define-command params.
pub fn member_params(def: &GeoMemberDef) -> Vec<ParamEntry> {
    let entry = |name: &str, value: ParamValue| ParamEntry {
        name: name.into(),
        value,
    };
    vec![
        entry("material", ParamValue::Text(def.material.props().name.into())),
        entry("temp_c", ParamValue::Range(def.window.temp_c.0, def.window.temp_c.1)),
        entry("precip", ParamValue::Range(def.window.precip.0, def.window.precip.1)),
        entry("depth_m", ParamValue::Range(def.window.depth_m.0, def.window.depth_m.1)),
        entry("abundance", ParamValue::Number(def.abundance)),
        entry("habit", ParamValue::Text(def.habit.as_str().into())),
        entry("hardness", ParamValue::Number(def.hardness)),
        entry("erodibility", ParamValue::Number(def.erodibility)),
    ]
}

/// The vanilla v1 geology content as a recorded command batch (content
/// packs are just registry command batches; this is the first one). The
/// member payloads are generated from the dc-core vanilla set, so pack and
/// typed model cannot drift.
pub fn vanilla_geology_pack() -> Vec<Payload> {
    let mut out = Vec::new();
    for class in geology::v1_classes() {
        out.push(Payload::DefineContentClass(DefineContentClass {
            name: class.to_string(),
            doc: format!("v1 geology class `{class}` (docs/design/geology.md)"),
            params: geology_param_specs(),
        }));
    }
    for def in geology::vanilla_members() {
        out.push(Payload::DefineClassMember(DefineClassMember {
            name: def.id.clone(),
            class: def.class.clone(),
            params: member_params(&def),
        }));
    }
    out
}

fn param<'a>(entries: &'a [ParamEntry], name: &str) -> Result<&'a ParamValue, String> {
    entries
        .iter()
        .find(|e| e.name == name)
        .map(|e| &e.value)
        .ok_or_else(|| format!("missing parameter `{name}`"))
}

fn param_range(entries: &[ParamEntry], name: &str) -> Result<(f64, f64), String> {
    match param(entries, name)? {
        ParamValue::Range(lo, hi) => Ok((*lo, *hi)),
        _ => Err(format!("parameter `{name}` is not a range")),
    }
}

fn param_number(entries: &[ParamEntry], name: &str) -> Result<f64, String> {
    match param(entries, name)? {
        ParamValue::Number(v) => Ok(*v),
        _ => Err(format!("parameter `{name}` is not a number")),
    }
}

fn param_text<'a>(entries: &'a [ParamEntry], name: &str) -> Result<&'a str, String> {
    match param(entries, name)? {
        ParamValue::Text(t) => Ok(t),
        _ => Err(format!("parameter `{name}` is not text")),
    }
}

/// Compile registered defs into the typed, canonically ordered
/// [`GeologySet`] the worldgen passes consume — the data-driven half of the
/// registry-table bridge. Only classes declaring the geology contract (all
/// [`geology_param_specs`] names present) participate; members of other
/// content classes are ignored here.
pub fn geology_set_from_defs<'a>(
    classes: impl Iterator<Item = &'a ContentClassDef>,
    members: impl Iterator<Item = &'a ClassMemberDef>,
) -> Result<GeologySet, String> {
    let contract_names: Vec<String> = geology_param_specs().into_iter().map(|s| s.name).collect();
    let mut builder = GeologySet::builder();
    let mut geo_classes: Vec<String> = Vec::new();
    for c in classes {
        let is_geo = contract_names
            .iter()
            .all(|n| c.params.iter().any(|p| p.name == *n));
        if !is_geo {
            continue;
        }
        builder
            .declare_class(&c.name)
            .map_err(|e| format!("class `{}`: {e}", c.name))?;
        geo_classes.push(c.name.clone());
    }
    for m in members {
        if !geo_classes.iter().any(|c| *c == m.class) {
            continue;
        }
        let material = param_text(&m.params, "material")
            .and_then(|t| material_by_name(t).ok_or_else(|| format!("unknown material `{t}`")))
            .map_err(|e| format!("member `{}`: {e}", m.name))?;
        let habit = param_text(&m.params, "habit")
            .and_then(|t| GeoHabit::from_str(t).ok_or_else(|| format!("unknown habit `{t}`")))
            .map_err(|e| format!("member `{}`: {e}", m.name))?;
        let wrap = |e: String| format!("member `{}`: {e}", m.name);
        let def = GeoMemberDef {
            id: m.name.clone(),
            class: m.class.clone(),
            material,
            window: FormationWindow {
                temp_c: param_range(&m.params, "temp_c").map_err(wrap)?,
                precip: param_range(&m.params, "precip").map_err(wrap)?,
                depth_m: param_range(&m.params, "depth_m").map_err(wrap)?,
            },
            abundance: param_number(&m.params, "abundance").map_err(wrap)?,
            habit,
            hardness: param_number(&m.params, "hardness").map_err(wrap)?,
            erodibility: param_number(&m.params, "erodibility").map_err(wrap)?,
        };
        builder
            .add_member(def)
            .map_err(|e| format!("member `{}`: {e}", m.name))?;
    }
    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geology_contract_validates_its_own_vanilla_members() {
        let contract = geology_param_specs();
        validate_class_contract(&contract).expect("contract is well-formed");
        for def in geology::vanilla_members() {
            validate_params(&contract, &member_params(&def))
                .unwrap_or_else(|e| panic!("vanilla member {} rejected: {e}", def.id));
        }
    }

    #[test]
    fn validate_params_enforces_the_contract() {
        let contract = geology_param_specs();
        let base = member_params(&geology::vanilla_members()[0]);
        // Unknown parameter.
        let mut bad = base.clone();
        bad.push(ParamEntry {
            name: "vein_angle".into(),
            value: ParamValue::Number(1.0),
        });
        assert!(validate_params(&contract, &bad).is_err());
        // Missing required parameter.
        let bad: Vec<ParamEntry> = base.iter().filter(|e| e.name != "abundance").cloned().collect();
        assert!(validate_params(&contract, &bad).is_err());
        // Kind mismatch.
        let mut bad = base.clone();
        bad.iter_mut().find(|e| e.name == "abundance").unwrap().value =
            ParamValue::Text("lots".into());
        assert!(validate_params(&contract, &bad).is_err());
        // Out of bounds.
        let mut bad = base.clone();
        bad.iter_mut().find(|e| e.name == "hardness").unwrap().value = ParamValue::Number(3.0);
        assert!(validate_params(&contract, &bad).is_err());
        // Unordered range.
        let mut bad = base.clone();
        bad.iter_mut().find(|e| e.name == "temp_c").unwrap().value = ParamValue::Range(30.0, -5.0);
        assert!(validate_params(&contract, &bad).is_err());
        // Unknown material.
        let mut bad = base.clone();
        bad.iter_mut().find(|e| e.name == "material").unwrap().value =
            ParamValue::Text("unobtainium".into());
        assert!(validate_params(&contract, &bad).is_err());
        // Bad habit choice.
        let mut bad = base;
        bad.iter_mut().find(|e| e.name == "habit").unwrap().value =
            ParamValue::Text("cloud".into());
        assert!(validate_params(&contract, &bad).is_err());
    }

    #[test]
    fn vanilla_pack_round_trips_into_the_typed_vanilla_set() {
        // Simulate what the host stores, then compile it back.
        let mut classes = Vec::new();
        let mut members = Vec::new();
        for p in vanilla_geology_pack() {
            match p {
                Payload::DefineContentClass(c) => classes.push(ContentClassDef {
                    name: c.name,
                    doc: c.doc,
                    params: c.params,
                    defined_tick: 1,
                    defined_by: crate::envelope::ConsumerId::new(
                        crate::envelope::ConsumerKind::Plugin,
                        "vanilla",
                    ),
                }),
                Payload::DefineClassMember(m) => members.push(ClassMemberDef {
                    name: m.name,
                    class: m.class,
                    params: m.params,
                    defined_tick: 1,
                    defined_by: crate::envelope::ConsumerId::new(
                        crate::envelope::ConsumerKind::Plugin,
                        "vanilla",
                    ),
                }),
                other => panic!("unexpected payload in pack: {other:?}"),
            }
        }
        let compiled = geology_set_from_defs(classes.iter(), members.iter())
            .expect("vanilla pack compiles");
        let typed = geology::vanilla();
        assert_eq!(compiled.members(), typed.members(), "pack ⇄ typed bridge must not drift");
    }
}
