//! 26.2 configured-carver configurations: the shared base (`CarverConfiguration`,
//! itself a probability feature configuration) plus the cave and canyon
//! extensions. 26.2 nests each registry entry's fields under `config`
//! (`ConfiguredWorldCarver`); 26.3 flattened the entries and dropped the
//! `configured_` prefix from the registries.
use crate::worldgen::{
    DecodeError, Object, anchor::VerticalAnchor, density::identifier,
    height_provider::HeightProvider, number_text, optional, required,
    value_provider::FloatProvider,
};

/// A block holder set as written in data: a `#tag`, a single block id, or a
/// list of block ids. Resolved against the block tags and registry at load;
/// the carving apply phase uses the dimension's replaceable set (all vanilla
/// carvers in a dimension share one), so a data-pack carver with a custom set
/// in a vanilla dimension applies the dimension's set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BlockSetRef {
    Tag(String),
    Blocks(Vec<String>),
}
impl BlockSetRef {
    pub fn from_json(raw: &serde_json::value::RawValue) -> Result<Self, DecodeError> {
        if let Ok(name) = serde_json::from_str::<String>(raw.get()) {
            if let Some(tag) = name.strip_prefix('#') {
                return Ok(Self::Tag(identifier(tag)?));
            }
            return Ok(Self::Blocks(vec![identifier(&name)?]));
        }
        if raw.get().trim_start().starts_with('[') {
            let names: Vec<String> = serde_json::from_str(raw.get())?;
            // An empty list decodes to an empty set in vanilla
            // (HolderSetCodec accepts it); it simply replaces nothing.
            return names
                .into_iter()
                .map(|name| identifier(&name))
                .collect::<Result<Vec<_>, _>>()
                .map(Self::Blocks);
        }
        Err(DecodeError::Field("replaceable"))
    }
}

/// `CarverDebugSettings`: the debug-mode flag and the four replacement states.
/// Debug mode is off for every vanilla configuration (no `debug_mode` field,
/// and the shared debug flag is false), so the states are validated but never
/// applied: the two-phase carving pipeline cannot know which carver claimed a
/// mask bit, and each configuration carries its own states.
#[derive(Clone, Debug)]
pub struct CarverDebugSettings {
    pub debug_mode: bool,
}
impl CarverDebugSettings {
    pub fn from_json(raw: &serde_json::value::RawValue) -> Result<Self, DecodeError> {
        let object: Object = serde_json::from_str(raw.get())?;
        let debug_mode = optional(&object, "debug_mode")
            .map(|raw| serde_json::from_str::<bool>(raw.get()))
            .transpose()
            .map_err(|_| DecodeError::Field("debug_mode"))?
            .unwrap_or(false);
        // Each state is a block id or a `{Name, Properties}` object.
        for name in ["air_state", "water_state", "lava_state", "barrier_state"] {
            if let Some(raw) = optional(&object, name) {
                if serde_json::from_str::<String>(raw.get()).is_err() {
                    let state: Object = serde_json::from_str(raw.get())?;
                    let raw = required(&state, "Name")?;
                    let name = serde_json::from_str::<String>(raw.get())
                        .map_err(|_| DecodeError::Field("debug state"))?;
                    identifier(&name)?;
                }
            }
        }
        Ok(Self { debug_mode })
    }
}

/// `CarverConfiguration`: probability, height, y scale, lava level, debug
/// settings and the replaceable block set, shared by cave, canyon and nether
/// carvers.
#[derive(Clone, Debug)]
pub struct CarverConfiguration {
    pub probability: f32,
    pub y: HeightProvider,
    pub y_scale: FloatProvider,
    pub lava_level: VerticalAnchor,
    pub debug_settings: CarverDebugSettings,
    pub replaceable: BlockSetRef,
}
impl CarverConfiguration {
    pub fn from_json(object: &Object) -> Result<Self, DecodeError> {
        let probability: f32 = number_text(required(object, "probability")?)?
            .parse()
            .map_err(|_| DecodeError::Field("probability"))?;
        if !(0.0..=1.0).contains(&probability) || probability.is_sign_negative() {
            return Err(DecodeError::Field("probability"));
        }
        Ok(Self {
            probability,
            y: HeightProvider::from_json(required(object, "y")?.get())?,
            y_scale: FloatProvider::from_json(required(object, "yScale")?.get())?,
            lava_level: VerticalAnchor::from_json(required(object, "lava_level")?.get())?,
            debug_settings: match optional(object, "debug_settings") {
                Some(raw) => CarverDebugSettings::from_json(raw)?,
                None => CarverDebugSettings { debug_mode: false },
            },
            replaceable: BlockSetRef::from_json(required(object, "replaceable")?)?,
        })
    }
}

/// `CaveCarverConfiguration`: the base plus the radius multipliers and the
/// floor level. Also the nether carver's configuration.
#[derive(Clone, Debug)]
pub struct CaveCarverConfiguration {
    pub base: CarverConfiguration,
    pub horizontal_radius_multiplier: FloatProvider,
    pub vertical_radius_multiplier: FloatProvider,
    pub floor_level: FloatProvider,
}
impl CaveCarverConfiguration {
    pub fn from_json(object: &Object) -> Result<Self, DecodeError> {
        let floor_level = FloatProvider::from_json(required(object, "floor_level")?.get())?;
        if floor_level.bounds()[0] < -1. || floor_level.bounds()[1] > 1. {
            return Err(DecodeError::Field("floor_level"));
        }
        Ok(Self {
            base: CarverConfiguration::from_json(object)?,
            horizontal_radius_multiplier: FloatProvider::from_json(
                required(object, "horizontal_radius_multiplier")?.get(),
            )?,
            vertical_radius_multiplier: FloatProvider::from_json(
                required(object, "vertical_radius_multiplier")?.get(),
            )?,
            floor_level,
        })
    }
}

/// `CanyonCarverConfiguration.CanyonShapeConfiguration`: the canyon shape
/// without the y scale, which is a top-level configuration field in 26.2
/// (26.3 moved it into the shape).
#[derive(Clone, Debug)]
pub struct CanyonShapeConfiguration {
    pub distance_factor: FloatProvider,
    pub thickness: FloatProvider,
    pub width_smoothness: i32,
    pub horizontal_radius_factor: FloatProvider,
    pub vertical_radius_default_factor: f32,
    pub vertical_radius_center_factor: f32,
}
impl CanyonShapeConfiguration {
    pub fn from_json(raw: &serde_json::value::RawValue) -> Result<Self, DecodeError> {
        let object: Object = serde_json::from_str(raw.get())?;
        let provider = |name| FloatProvider::from_json(required(&object, name)?.get());
        Ok(Self {
            distance_factor: provider("distance_factor")?,
            thickness: provider("thickness")?,
            width_smoothness: crate::worldgen::int_range(
                required(&object, "width_smoothness")?,
                "width_smoothness",
                1,
                i32::MAX,
            )?,
            horizontal_radius_factor: provider("horizontal_radius_factor")?,
            vertical_radius_default_factor: scalar(&object, "vertical_radius_default_factor")?,
            vertical_radius_center_factor: scalar(&object, "vertical_radius_center_factor")?,
        })
    }
}

/// `CanyonCarverConfiguration`: the base plus the vertical rotation and the
/// shape.
#[derive(Clone, Debug)]
pub struct CanyonCarverConfiguration {
    pub base: CarverConfiguration,
    pub vertical_rotation: FloatProvider,
    pub shape: CanyonShapeConfiguration,
}
impl CanyonCarverConfiguration {
    pub fn from_json(object: &Object) -> Result<Self, DecodeError> {
        Ok(Self {
            base: CarverConfiguration::from_json(object)?,
            vertical_rotation: FloatProvider::from_json(
                required(object, "vertical_rotation")?.get(),
            )?,
            shape: CanyonShapeConfiguration::from_json(required(object, "shape")?)?,
        })
    }
}

fn scalar(object: &Object, name: &'static str) -> Result<f32, DecodeError> {
    number_text(required(object, name)?)?
        .parse()
        .map_err(|_| DecodeError::Field(name))
}

/// The `config` object of a configured-carver registry entry
/// (`{type, config}`), or the entry itself for the flat form the carver
/// fixtures send.
pub fn config_object(json: &str) -> Result<Object, DecodeError> {
    let object: Object = serde_json::from_str(json)?;
    match optional(&object, "config") {
        Some(raw) => Ok(serde_json::from_str(raw.get())?),
        None => Ok(object),
    }
}
