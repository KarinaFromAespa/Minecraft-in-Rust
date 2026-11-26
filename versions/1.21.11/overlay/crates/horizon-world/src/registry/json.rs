//! Java 26.3 compact/full block-state JSON codecs over a pinned registry.
use super::{BlockRegistry, BlockStateId};
use crate::compression::invalid;
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    io,
};

/// Property kinds accompany the imported state registry. Integer properties
/// parse Java decimal strings; enum and boolean properties use exact names.
pub struct BlockStateJsonCodec<'a> {
    registry: &'a BlockRegistry,
    integers: BTreeSet<(String, String)>,
    counts: BTreeMap<String, usize>,
}
impl<'a> BlockStateJsonCodec<'a> {
    pub fn new(
        registry: &'a BlockRegistry,
        integers: BTreeSet<(String, String)>,
    ) -> io::Result<Self> {
        for (block, property) in &integers {
            let state = registry
                .default_state(block)
                .and_then(|id| registry.state(id));
            if !state.is_some_and(|state| state.properties.contains_key(property)) {
                return Err(invalid("integer property is absent from block registry"));
            }
        }
        let mut counts = BTreeMap::new();
        for state in &registry.states {
            *counts.entry(state.name.clone()).or_insert(0) += 1;
        }
        Ok(Self {
            registry,
            integers,
            counts,
        })
    }
    /// `full` selects the object form. In 1.21.11 `BlockState.CODEC` is the
    /// object form with the NBT field names only (`Name`/`Properties`) —
    /// 26.3's `FULL_CODEC`; a bare block-id string or the `id`/`properties`
    /// names decode-error in 1.21.11. `full=false` is 26.3's compact `CODEC`
    /// (string or object with either naming), kept for the 26.3-model
    /// callers and the block-state-codec harness.
    /// Invalid/missing properties keep their defaults, as vanilla's codec does.
    pub fn decode(&self, json: &str, full: bool) -> io::Result<BlockStateId> {
        let value: Value = serde_json::from_str(json).map_err(|e| invalid(e.to_string()))?;
        let name = if !full && value.is_string() {
            value.as_str()
        } else if full {
            // 1.21.11 object form: `Name` only, never `id`.
            value.get("Name").and_then(Value::as_str)
        } else {
            value
                .get("id")
                .or_else(|| value.get("Name"))
                .and_then(Value::as_str)
        }
        .ok_or_else(|| invalid("missing block identifier"))?;
        let (namespace, path) = name.split_once(':').unwrap_or(("minecraft", name));
        let name = format!(
            "{}:{path}",
            if namespace.is_empty() {
                "minecraft"
            } else {
                namespace
            }
        );
        let default = self
            .registry
            .default_state(&name)
            .ok_or_else(|| invalid("unknown block identifier"))?;
        let mut state = self.registry.state(default).unwrap().clone();
        let properties_value = if full {
            // 1.21.11 object form: `Properties` only, never `properties`.
            value.get("Properties")
        } else {
            value.get("properties").or_else(|| value.get("Properties"))
        };
        if let Some(properties) = properties_value.and_then(Value::as_object) {
            for (key, raw) in properties {
                if !state.properties.contains_key(key) {
                    continue;
                }
                let Some(value) = raw.as_str() else {
                    continue;
                };
                let candidate = if self.integers.contains(&(name.clone(), key.clone())) {
                    let Some(value) = java_integer(value) else {
                        continue;
                    };
                    value.to_string()
                } else {
                    value.to_owned()
                };
                let previous = state.properties.insert(key.clone(), candidate).unwrap();
                if self.registry.id(&state).is_none() {
                    state.properties.insert(key.clone(), previous);
                }
            }
        }
        self.registry
            .id(&state)
            .ok_or_else(|| invalid("block state is absent from registry"))
    }
    pub fn encode(&self, id: BlockStateId, full: bool) -> io::Result<String> {
        let state = self
            .registry
            .state(id)
            .ok_or_else(|| invalid("unknown block state ID"))?;
        let value = if !full && self.registry.default_state(&state.name) == Some(id) {
            Value::String(state.name.clone())
        } else if self.counts[&state.name] == 1 {
            serde_json::json!({"id":state.name})
        } else {
            serde_json::json!({"id":state.name,"properties":state.properties})
        };
        serde_json::to_string(&value).map_err(|e| invalid(e.to_string()))
    }
}

pub(crate) use horizon_core::text::parse_java_integer as java_integer;
