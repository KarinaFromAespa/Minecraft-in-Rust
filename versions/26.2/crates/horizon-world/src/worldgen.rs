//! Version-specific noise, density registry and terrain-router JSON decoding.
//! Complete terrain generation remains pending.
pub mod anchor;
pub mod aquifer;
pub mod bamboo;
pub mod biome;
pub mod biome_chunk;
mod biome_chunks;
pub mod biome_predicate;
pub mod biome_source;
pub mod biome_temperature;
pub mod biome_weather;
pub mod blending;
pub mod block_blob;
pub mod block_column;
pub mod block_feature;
pub mod block_pile;
pub mod block_predicate;
pub mod blue_ice;
pub mod bonus_chest;
pub mod bulk_section;
pub mod carver;
pub mod carving;
pub mod cave;
pub mod chorus_plant;
pub mod climate;
pub mod decoration;
pub mod decoration_order;
pub mod delta;
pub mod density;
pub mod disk;
pub mod dripstone;
pub mod end_gateway;
pub mod end_podium;
pub mod end_spike;
pub mod feature;
pub mod feature_catalog;
pub mod feature_graph;
pub mod feature_region;
pub mod feature_sort;
pub mod fluid_predicate;
pub mod fungus;
pub mod generator;
mod generator_blocks;
mod generator_decoration;
mod generator_structures;
pub(crate) use generator_structures::tag_json;
pub mod height_provider;
pub mod iceberg;
pub mod live;
mod long_set;
pub mod material_condition;
pub mod material_extensions;
pub mod material_stage;
pub mod material_surface;
pub mod native_generate;
pub mod ore_feature;
pub mod overrides;
pub mod placement;
pub mod projected;
pub mod replace_blobs;
pub mod root_system;
pub mod router;
pub mod rule_test;
pub mod sculk;
pub mod settings;
pub mod simple_block;
pub mod snow_freeze;
pub mod spawn_finder;
pub mod speleothem;
pub mod spike;
pub mod spring;
pub mod state_provider;
pub mod structure;
pub mod synthesize;
pub mod tags;
pub mod terrain;
pub mod tree;
pub(crate) mod tree_blocks;
pub mod underwater_magma;
pub mod value_provider;
pub mod vegetation;
pub mod vines;
pub mod weighted;
pub mod write_zone;
use horizon_core::noise::{NoiseBuildError, NormalNoise};
use serde_json::value::RawValue;
use std::collections::BTreeMap;

#[derive(Debug)]
pub enum DecodeError {
    Json(serde_json::Error),
    Field(&'static str),
    Noise(NoiseBuildError),
    Graph(horizon_core::density::graph::CompileError),
    Sample(horizon_core::density::SampleError),
    Registry(String),
}
impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(error) => error.fmt(f),
            Self::Field(field) => write!(f, "invalid world generation field: {field}"),
            Self::Noise(error) => error.fmt(f),
            Self::Graph(error) => error.fmt(f),
            Self::Sample(error) => error.fmt(f),
            Self::Registry(error) => f.write_str(error),
        }
    }
}
impl std::error::Error for DecodeError {}
impl From<serde_json::Error> for DecodeError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}
impl From<NoiseBuildError> for DecodeError {
    fn from(value: NoiseBuildError) -> Self {
        Self::Noise(value)
    }
}

impl From<horizon_core::density::graph::CompileError> for DecodeError {
    fn from(value: horizon_core::density::graph::CompileError) -> Self {
        Self::Graph(value)
    }
}

impl From<horizon_core::density::SampleError> for DecodeError {
    fn from(value: horizon_core::density::SampleError) -> Self {
        Self::Sample(value)
    }
}

type Object = BTreeMap<String, Box<RawValue>>;
fn optional<'a>(object: &'a Object, name: &str) -> Option<&'a RawValue> {
    object
        .get(name)
        .map(AsRef::as_ref)
        .filter(|raw| raw.get() != "null")
}
fn required<'a>(object: &'a Object, name: &'static str) -> Result<&'a RawValue, DecodeError> {
    optional(object, name).ok_or(DecodeError::Field(name))
}
fn number_text(raw: &RawValue) -> Result<&str, DecodeError> {
    let text = raw.get();
    if text.starts_with('-') || text.starts_with(|c: char| c.is_ascii_digit()) {
        Ok(text)
    } else {
        Err(DecodeError::Field("number"))
    }
}
fn double(raw: &RawValue) -> Result<f64, DecodeError> {
    number_text(raw)?
        .parse()
        .map_err(|_| DecodeError::Field("number"))
}

// Gson's lazily parsed numbers use BigDecimal.intValue for decimal/exponent
// forms: truncate toward zero, then retain the low 32 bits. Converting through
// f64 would both round large integers and saturate rather than wrap.
fn integer(raw: &RawValue) -> Result<i32, DecodeError> {
    let text = number_text(raw)?;
    let negative = text.starts_with('-');
    let text = text.strip_prefix('-').unwrap_or(text);
    let (mantissa, exponent) = match text.split_once(['e', 'E']) {
        Some((mantissa, exponent)) => (
            mantissa,
            exponent
                .parse::<i32>()
                .map_err(|_| DecodeError::Field("integer exponent"))? as i64,
        ),
        None => (text, 0),
    };
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let digits = whole.bytes().chain(fraction.bytes());
    let length = whole.len() as i64 + exponent;
    if length <= 0 {
        return Ok(0);
    }
    let available = whole.len() as i64 + fraction.len() as i64;
    let zeroes = (length - available).max(0);
    if zeroes >= 32 {
        return Ok(0);
    }
    let mut result = 0u32;
    for digit in digits.take(length as usize) {
        result = result
            .wrapping_mul(10)
            .wrapping_add(u32::from(digit - b'0'));
    }
    for _ in 0..zeroes {
        result = result.wrapping_mul(10);
    }
    Ok(if negative {
        result.wrapping_neg()
    } else {
        result
    } as i32)
}
fn int_range(raw: &RawValue, name: &'static str, low: i32, high: i32) -> Result<i32, DecodeError> {
    let value = integer(raw)?;
    if (low..=high).contains(&value) {
        Ok(value)
    } else {
        Err(DecodeError::Field(name))
    }
}
fn double_range(
    raw: &RawValue,
    name: &'static str,
    low: f64,
    high: f64,
) -> Result<f64, DecodeError> {
    let value = double(raw)?;
    if value.total_cmp(&low).is_ge() && value.total_cmp(&high).is_le() {
        Ok(value)
    } else {
        Err(DecodeError::Field(name))
    }
}

/// Decode Java Edition 26.2's normal-noise parameters from strict JSON:
/// `{"firstOctave": int, "amplitudes": [double, ...]}`. Numeric tokens retain
/// their decimal text; all rounding occurs at the target numeric width.
/// Unknown object fields are ignored, as in vanilla's codec. The codec is
/// plain `Codec.INT` / `Codec.DOUBLE.listOf()`: no range check on either
/// field, and an empty amplitudes list fails only at noise construction.
pub fn decode_noise(json: &str) -> Result<NormalNoise, DecodeError> {
    let object: Object = serde_json::from_str(json)?;
    let base = integer(required(&object, "firstOctave")?)?;
    let amplitudes: Vec<Box<RawValue>> =
        serde_json::from_str(required(&object, "amplitudes")?.get())?;
    let mut values = Vec::with_capacity(amplitudes.len());
    for raw in &amplitudes {
        values.push(double(raw)?);
    }
    Ok(NormalNoise::create_parity(base, &values)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decimal_integers_wrap_without_floating_point_conversion() {
        for (text, expected) in [
            ("4294967295", -1),
            ("4294967296.5", 0),
            ("-4294967297.9", -1),
            ("1.9999999999999999999", 1),
            ("12e1", 120),
            ("123e-1", 12),
            ("-123e-3", 0),
            ("123e99", 0),
        ] {
            let raw: Box<RawValue> = serde_json::from_str(text).unwrap();
            assert_eq!(integer(&raw).unwrap(), expected, "{text}");
        }
    }
    #[test]
    fn a_muted_octave_decodes_to_a_silent_noise() {
        let noise = decode_noise(r#"{"firstOctave":0,"amplitudes":[0.0]}"#).unwrap();
        assert!(noise.octaves().is_empty());
        assert_eq!(noise.normalization_factor(), 0.0);
        assert_eq!(noise.range(), [0.0, 0.0]);
    }
    #[test]
    fn first_octave_and_amplitudes_decode_and_unknown_fields_are_ignored() {
        let a = decode_noise(r#"{"firstOctave":-7,"amplitudes":[1.0]}"#).unwrap();
        let b = decode_noise(r#"{"firstOctave":-7,"amplitudes":[1.0],"unknown":42}"#).unwrap();
        assert_eq!(a.range(), b.range());
        assert_eq!(a.normalization_factor(), b.normalization_factor());
        assert_eq!(a.octaves().len(), b.octaves().len());
        assert!(decode_noise(r#"{"amplitudes":[1.0]}"#).is_err());
        assert!(decode_noise(r#"{"firstOctave":-7}"#).is_err());
        assert!(decode_noise(r#"{"firstOctave":-7,"amplitudes":[]}"#).is_err());
        assert!(decode_noise(r#"{"firstOctave":"-7","amplitudes":[1.0]}"#).is_err());
        assert!(decode_noise(r#"{"firstOctave":-7,"amplitudes":["1"]}"#).is_err());
        assert!(decode_noise(r#"{"firstOctave":-7,"amplitudes":1}"#).is_err());
    }
}
