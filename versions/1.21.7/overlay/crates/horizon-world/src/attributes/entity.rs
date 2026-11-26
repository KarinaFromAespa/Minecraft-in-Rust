//! Registered entity defaults and lazy attribute instances. Cross-attribute bulk
//! mutation/serialization order is deliberately outside this per-attribute API.
use super::{Definition, Instance, Observer};
use crate::{DATA_VERSION, compression::invalid};
use serde_json::Value;
use std::{collections::BTreeMap, convert::Infallible, io, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AttributeId(usize);
impl AttributeId {
    pub(super) fn index(self) -> usize {
        self.0
    }
    /// The attribute's id in the `minecraft:attribute` registry: the catalog
    /// export lists `BuiltInRegistries.ATTRIBUTE` in order.
    pub fn registry_id(self) -> i32 {
        self.0 as i32
    }
}
#[derive(Debug)]
pub struct RegisteredAttribute {
    pub name: String,
    pub definition: Definition,
    pub syncable: bool,
}
#[derive(Debug)]
pub struct Catalog {
    attributes: Vec<RegisteredAttribute>,
    ids: BTreeMap<String, AttributeId>,
    defaults: BTreeMap<String, Arc<[Option<f64>]>>,
}
fn string(v: &Value) -> io::Result<&str> {
    v.as_str()
        .ok_or_else(|| invalid("expected attribute string"))
}
fn number(v: &Value) -> io::Result<f64> {
    let s = string(v)?;
    if s.len() != 16 {
        return Err(invalid("expected 16-digit attribute double bits"));
    }
    Ok(f64::from_bits(
        u64::from_str_radix(s, 16).map_err(|e| invalid(e.to_string()))?,
    ))
}
impl Catalog {
    pub(super) fn default_bases(&self, entity: &str) -> Option<Arc<[Option<f64>]>> {
        self.defaults.get(entity).cloned()
    }
    /// Imports a version-checked export of registered definitions and actual
    /// DefaultAttributes suppliers. Defaults contain base values, not instances.
    pub fn from_json(v: &Value) -> io::Result<Self> {
        if v["data_version"].as_i64() != Some(i64::from(DATA_VERSION)) {
            return Err(invalid("attribute catalog data version mismatch"));
        }
        let mut attributes = Vec::new();
        let mut ids = BTreeMap::new();
        for a in v["attributes"]
            .as_array()
            .ok_or_else(|| invalid("missing attributes"))?
        {
            let name = string(&a["id"])?;
            // Identifier normalization is shared with modifier IDs.
            let canonical = super::Modifier::new(name, 0.0, super::Operation::AddValue)?;
            if canonical.id() != name
                || ids
                    .insert(name.into(), AttributeId(attributes.len()))
                    .is_some()
            {
                return Err(invalid("noncanonical or duplicate attribute identifier"));
            }
            let default = number(&a["default"])?;
            let definition = if a.get("min").is_some() {
                Definition::ranged(default, number(&a["min"])?, number(&a["max"])?)?
            } else {
                Definition::basic(default)
            };
            attributes.push(RegisteredAttribute {
                name: name.into(),
                definition,
                syncable: a["syncable"]
                    .as_bool()
                    .ok_or_else(|| invalid("missing attribute sync flag"))?,
            });
        }
        let mut defaults = BTreeMap::new();
        // 1.21.7 predates the three 26.x physics attributes: vanilla 1.21.7's
        // attribute registry has no entries for them, but the movement code
        // reads all three for every living entity (and the crash is a fatal
        // "missing movement attribute"). Synthesize them with the 26.2-verified
        // registry values so a 1.21.7 catalog behaves like a 26.x one; a
        // catalog that already lists an attribute is untouched.
        const SYNTHESIZED_ATTRIBUTES: [(&str, f64, f64, f64); 3] = [
            ("minecraft:air_drag_modifier", 1.0, 0.0, 2048.0),
            ("minecraft:bounciness", 0.0, 0.0, 1.0),
            ("minecraft:friction_modifier", 1.0, 0.0, 2048.0),
        ];
        let mut synthesized = Vec::new();
        for (name, default, min, max) in SYNTHESIZED_ATTRIBUTES {
            if ids.contains_key(name) {
                continue;
            }
            let id = AttributeId(attributes.len());
            ids.insert(name.into(), id);
            attributes.push(RegisteredAttribute {
                name: name.into(),
                definition: Definition::ranged(default, min, max)?,
                syncable: true,
            });
            synthesized.push((id, default));
        }
        for (entity, values) in v["entities"]
            .as_object()
            .ok_or_else(|| invalid("missing entity defaults"))?
        {
            let mut bases = vec![None; attributes.len()];
            for (name, base) in values
                .as_object()
                .ok_or_else(|| invalid("invalid entity supplier"))?
            {
                let id = ids
                    .get(name)
                    .ok_or_else(|| invalid("unknown supplier attribute"))?;
                bases[id.0] = Some(number(base)?);
            }
            // A pre-26.x supplier lists none of the synthesized attributes;
            // give every entity the registry default, as all 93 of 26.2's
            // suppliers do (player bases there are exactly these defaults).
            for (id, default) in &synthesized {
                if bases[id.0].is_none() {
                    bases[id.0] = Some(*default);
                }
            }
            defaults.insert(entity.clone(), Arc::from(bases));
        }
        Ok(Self {
            attributes,
            ids,
            defaults,
        })
    }
    /// The entity type's supplier base of an attribute, if the type has it.
    pub fn default_base(&self, entity: &str, id: AttributeId) -> Option<f64> {
        self.defaults.get(entity).and_then(|b| b[id.0])
    }
    pub fn id(&self, name: &str) -> Option<AttributeId> {
        self.ids.get(name).copied()
    }
    pub fn attribute(&self, id: AttributeId) -> &RegisteredAttribute {
        &self.attributes[id.0]
    }
    pub fn attributes(&self) -> impl Iterator<Item = (AttributeId, &RegisteredAttribute)> {
        self.attributes
            .iter()
            .enumerate()
            .map(|(i, a)| (AttributeId(i), a))
    }
    pub fn entities(&self) -> impl Iterator<Item = &str> {
        self.defaults.keys().map(String::as_str)
    }
    pub fn create(self: &Arc<Self>, entity: &str) -> Option<EntityAttributes> {
        let bases = self.defaults.get(entity)?.clone();
        let n = bases.len();
        Some(EntityAttributes {
            catalog: self.clone(),
            bases,
            instances: vec![None; n],
            updates: vec![false; n],
            sync: vec![false; n],
        })
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LookupError {
    MissingAttribute,
    MissingModifier,
    InstanceMissingModifier,
}
/// Membership in the native update/sync sets, with no JVM identity-hash order
/// promise. Clearing a flag acknowledges that particular pending update.
pub struct DirtyObserver<'a> {
    update: &'a mut bool,
    sync: &'a mut bool,
    syncable: bool,
}
impl Observer for DirtyObserver<'_> {
    type Error = Infallible;
    fn dirty(&mut self, _: &mut Instance) -> Result<(), Infallible> {
        *self.update = true;
        if self.syncable {
            *self.sync = true;
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct EntityAttributes {
    catalog: Arc<Catalog>,
    bases: Arc<[Option<f64>]>,
    instances: Vec<Option<Box<Instance>>>,
    updates: Vec<bool>,
    sync: Vec<bool>,
}
impl EntityAttributes {
    pub fn catalog(&self) -> &Arc<Catalog> {
        &self.catalog
    }
    pub fn has(&self, id: AttributeId) -> bool {
        self.bases[id.0].is_some()
    }
    pub fn instance(&self, id: AttributeId) -> Option<&Instance> {
        self.instances[id.0].as_deref()
    }
    /// Merely querying a default value must not instantiate or dirty it.
    pub fn value(&mut self, id: AttributeId) -> Result<f64, LookupError> {
        if let Some(a) = self.instances[id.0].as_deref_mut() {
            return Ok(a.value());
        }
        let base = self.bases[id.0].ok_or(LookupError::MissingAttribute)?;
        Ok(self.catalog.attribute(id).definition.sanitize(base))
    }
    pub fn base_value(&self, id: AttributeId) -> Result<f64, LookupError> {
        self.instances[id.0]
            .as_deref()
            .map(Instance::base_value)
            .or(self.bases[id.0])
            .ok_or(LookupError::MissingAttribute)
    }
    pub fn has_modifier(&self, id: AttributeId, modifier: &str) -> bool {
        self.instance(id)
            .and_then(|a| a.modifier(modifier))
            .is_some()
    }
    pub fn modifier_value(&self, id: AttributeId, modifier: &str) -> Result<f64, LookupError> {
        if let Some(a) = self.instance(id) {
            return a
                .modifier(modifier)
                .map(super::Modifier::amount)
                .ok_or(LookupError::InstanceMissingModifier);
        }
        if self.has(id) {
            Err(LookupError::MissingModifier)
        } else {
            Err(LookupError::MissingAttribute)
        }
    }
    /// Native getInstance copies the supplier on first access and immediately
    /// schedules an update, plus a sync when the definition is client-syncable.
    pub fn ensure(&mut self, id: AttributeId) -> bool {
        if self.instances[id.0].is_some() {
            return true;
        }
        let Some(base) = self.bases[id.0] else {
            return false;
        };
        let definition = self.catalog.attribute(id);
        let mut instance = Instance::new(definition.definition);
        let mut observer = DirtyObserver {
            update: &mut self.updates[id.0],
            sync: &mut self.sync[id.0],
            syncable: definition.syncable,
        };
        // Registered default suppliers have no modifiers. replaceFrom always
        // invalidates even when the supplier base equals the registered default.
        let mut source = Instance::new(definition.definition);
        source.base = base;
        instance.replace_from(&source, &mut observer).unwrap();
        self.instances[id.0] = Some(Box::new(instance));
        true
    }
    /// Mutations must use the supplied observer to preserve dirty memberships.
    pub fn edit<R>(
        &mut self,
        id: AttributeId,
        f: impl FnOnce(&mut Instance, &mut DirtyObserver<'_>) -> R,
    ) -> Option<R> {
        if !self.ensure(id) {
            return None;
        }
        let mut observer = DirtyObserver {
            update: &mut self.updates[id.0],
            sync: &mut self.sync[id.0],
            syncable: self.catalog.attribute(id).syncable,
        };
        Some(f(
            self.instances[id.0].as_deref_mut().unwrap(),
            &mut observer,
        ))
    }
    /// Reset uses the entity supplier's base, without creating a missing instance.
    pub fn reset_base(&mut self, id: AttributeId) -> bool {
        let Some(base) = self.bases[id.0] else {
            return false;
        };
        if self.instances[id.0].is_some() {
            self.edit(id, |a, o| a.set_base_value(base, o).unwrap());
        }
        true
    }
    pub fn needs_update(&self, id: AttributeId) -> bool {
        self.updates[id.0]
    }
    pub fn needs_sync(&self, id: AttributeId) -> bool {
        self.sync[id.0]
    }
    pub fn clear_update(&mut self, id: AttributeId) {
        self.updates[id.0] = false;
    }
    pub fn clear_sync(&mut self, id: AttributeId) {
        self.sync[id.0] = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn catalog_json(attributes: Value, entities: Value) -> Value {
        serde_json::json!({
            "data_version": crate::DATA_VERSION,
            "attributes": attributes,
            "entities": entities,
        })
    }

    fn ranged_attribute(id: &str, default: &str) -> Value {
        serde_json::json!({
            "id": id,
            "default": default,
            "syncable": true,
            "min": "0000000000000000",
            "max": "40a0000000000000",
        })
    }

    #[test]
    fn synthesizes_missing_physics_attributes() {
        // A 1.21.7-shaped catalog: no air_drag_modifier, bounciness or
        // friction_modifier anywhere, player supplier lists movement_speed.
        let v = catalog_json(
            serde_json::json!([ranged_attribute(
                "minecraft:movement_speed",
                "3fb999999999999a"
            )]),
            serde_json::json!({
                "minecraft:player": {
                    "minecraft:movement_speed": "3fb999999999999a",
                },
            }),
        );
        let catalog = Catalog::from_json(&v).expect("catalog loads");
        for (name, expected) in [
            ("minecraft:air_drag_modifier", 1.0),
            ("minecraft:bounciness", 0.0),
            ("minecraft:friction_modifier", 1.0),
        ] {
            let id = catalog.id(name).expect("synthesized attribute present");
            assert_eq!(
                catalog.default_base("minecraft:player", id),
                Some(expected),
                "{name} supplier base"
            );
        }
        // The attributes that were there are untouched.
        let speed = catalog.id("minecraft:movement_speed").unwrap();
        assert_eq!(
            catalog.default_base("minecraft:player", speed),
            Some(0.1)
        );
    }

    #[test]
    fn keeps_catalog_attributes_unsynthesized() {
        // A catalog that already lists air_drag_modifier keeps it exactly:
        // no duplicate entry, and a supplier that omits it stays omitted
        // (only synthesized attributes get a default base).
        let v = catalog_json(
            serde_json::json!([
                ranged_attribute("minecraft:movement_speed", "3fb999999999999a"),
                ranged_attribute("minecraft:air_drag_modifier", "4000000000000000"),
            ]),
            serde_json::json!({
                "minecraft:player": {
                    "minecraft:movement_speed": "3fb999999999999a",
                },
            }),
        );
        let catalog = Catalog::from_json(&v).expect("catalog loads");
        // 2 listed + bounciness and friction_modifier synthesized; the
        // listed air_drag_modifier is not duplicated (that would be 5).
        assert_eq!(catalog.attributes().count(), 4);
        let air = catalog.id("minecraft:air_drag_modifier").unwrap();
        assert_eq!(catalog.default_base("minecraft:player", air), None);
    }
}
