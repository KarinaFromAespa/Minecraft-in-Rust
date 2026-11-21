//! Bounded decoding of density registries into the immutable core graph.
//! Named registry holders and raw numeric tokens preserve compilation behavior.
use super::*;
use horizon_core::{
    density::{
        BlendedNoise, CubicSpline, DistanceMetric, RoundMode, ShiftNoiseKind, SplinePoint,
        TilingMode, WorldField,
        graph::{DensityGraph, FunctionId, GraphBinary, GraphNode, GraphUnary, NoiseId},
    },
    geometry::Axis,
};
use std::{collections::BTreeSet, sync::Arc};

pub(super) fn identifier(name: &str) -> Result<String, DecodeError> {
    let (namespace, path) = name.split_once(':').unwrap_or(("minecraft", name));
    let namespace = if namespace.is_empty() {
        "minecraft"
    } else {
        namespace
    };
    let valid = |c: u8| c.is_ascii_lowercase() || c.is_ascii_digit() || b"_.-".contains(&c);
    if !namespace.bytes().all(valid)
        || !path.bytes().all(|c| valid(c) || c == b'/')
        || path.is_empty()
    {
        return Err(DecodeError::Registry(format!(
            "invalid registry identifier: {name}"
        )));
    }
    Ok(format!("{namespace}:{path}"))
}
fn string(raw: &RawValue) -> Result<String, DecodeError> {
    Ok(serde_json::from_str(raw.get())?)
}
fn float(raw: &RawValue) -> Result<f32, DecodeError> {
    number_text(raw)?
        .parse()
        .map_err(|_| DecodeError::Field("float"))
}
fn noise_value(raw: &RawValue) -> Result<f32, DecodeError> {
    let value = float(raw)?;
    if (-1e6..=1e6).contains(&value) {
        Ok(value)
    } else {
        Err(DecodeError::Field("density value"))
    }
}
fn axis(raw: &RawValue) -> Result<Axis, DecodeError> {
    match string(raw)?.as_str() {
        "x" => Ok(Axis::X),
        "y" => Ok(Axis::Y),
        "z" => Ok(Axis::Z),
        _ => Err(DecodeError::Field("axis")),
    }
}

/// All entries are loaded atomically. Named child references resolve through
/// distinct registry holders; returned entry values are their direct graphs.
#[derive(Debug)]
pub struct DensityRegistry {
    graph: DensityGraph,
    values: BTreeMap<String, FunctionId>,
    noises: BTreeMap<String, NoiseId>,
}
impl DensityRegistry {
    /// Input keys are registry identifiers, and values contain original JSON.
    /// This API accepts acyclic registries, at most 256 nested graph levels and
    /// at most 256 interval branches. Inline noise holders are not yet supported.
    pub fn from_json(
        noises: &BTreeMap<String, String>,
        densities: &BTreeMap<String, String>,
    ) -> Result<Self, DecodeError> {
        let mut loader = Loader {
            graph: DensityGraph::new(),
            noises: BTreeMap::new(),
            definitions: BTreeMap::new(),
            values: BTreeMap::new(),
            active: BTreeSet::new(),
        };
        for (name, json) in noises {
            let name = identifier(name)?;
            let noise = loader
                .graph
                .register_noise(name.clone(), decode_noise(json)?)?;
            loader.noises.insert(name, noise);
        }
        for (name, json) in densities {
            let name = identifier(name)?;
            let raw: Box<RawValue> = serde_json::from_str(json)?;
            if loader.definitions.insert(name.clone(), raw).is_some() {
                return Err(DecodeError::Registry(format!(
                    "duplicate density identifier: {name}"
                )));
            }
        }
        let names: Vec<_> = loader.definitions.keys().cloned().collect();
        for name in names {
            loader.resolve(&name, 0)?;
        }
        Ok(Self {
            graph: loader.graph,
            values: loader.values,
            noises: loader.noises,
        })
    }
    /// Insert a synthetic node built by the caller rather than decoded from
    /// JSON, and return its id. Vanilla 1.21.7 derives some functions from
    /// the router instead of naming them in data (the aquifer's deep-dark
    /// exclusion, the ore veins' density functions), so the generator
    /// synthesizes their graph nodes.
    pub fn insert(&mut self, node: GraphNode) -> Result<FunctionId, DecodeError> {
        Ok(self.graph.insert(node)?)
    }
    pub fn graph(&self) -> &DensityGraph {
        &self.graph
    }
    pub fn into_graph(self) -> DensityGraph {
        self.graph
    }
    pub fn entries(&self) -> impl Iterator<Item = (&str, FunctionId)> {
        self.values.iter().map(|(name, id)| (name.as_str(), *id))
    }
    pub fn value(&self, name: &str) -> Option<FunctionId> {
        self.values.get(name).copied()
    }
    pub fn reference(&self, name: &str) -> Option<FunctionId> {
        self.graph.function(name)
    }
    /// Decode direct roots against this registry, retaining shared named holders.
    /// All roots are added atomically; a failure leaves existing IDs and graphs
    /// intact. Roots are not registered as additional named density entries.
    pub fn decode_functions(&mut self, json: &[&str]) -> Result<Vec<FunctionId>, DecodeError> {
        let mut loader = Loader {
            graph: self.graph.clone(),
            noises: self.noises.clone(),
            definitions: BTreeMap::new(),
            values: BTreeMap::new(),
            active: BTreeSet::new(),
        };
        let roots = json
            .iter()
            .map(|json| {
                let raw: Box<RawValue> = serde_json::from_str(json)?;
                loader.decode(&raw, 0)
            })
            .collect::<Result<_, _>>()?;
        self.graph = loader.graph;
        Ok(roots)
    }
}
struct Loader {
    graph: DensityGraph,
    noises: BTreeMap<String, NoiseId>,
    definitions: BTreeMap<String, Box<RawValue>>,
    values: BTreeMap<String, FunctionId>,
    active: BTreeSet<String>,
}
impl Loader {
    fn resolve(&mut self, name: &str, depth: usize) -> Result<FunctionId, DecodeError> {
        if let Some(id) = self.graph.function(name) {
            return Ok(id);
        }
        if depth > 256 {
            return Err(DecodeError::Field("density nesting"));
        }
        if !self.active.insert(name.to_owned()) {
            return Err(DecodeError::Registry(format!(
                "cyclic density reference: {name}"
            )));
        }
        let raw = self
            .definitions
            .get(name)
            .ok_or_else(|| DecodeError::Registry(format!("unknown density reference: {name}")))?
            .clone();
        let value = self.decode(&raw, depth + 1)?;
        let reference = self.graph.register_function(name.to_owned(), value)?;
        self.values.insert(name.to_owned(), value);
        self.active.remove(name);
        Ok(reference)
    }
    fn input(
        &mut self,
        object: &Object,
        name: &'static str,
        depth: usize,
    ) -> Result<FunctionId, DecodeError> {
        self.decode(required(object, name)?, depth + 1)
    }
    /// 1.21.7-era density functions name their child `argument`
    /// (unary/cache) or `argument1`/`argument2` (binary); 26.x renamed
    /// them `input` and `left`/`right`. Accept either spelling.
    fn input_named(
        &mut self,
        object: &Object,
        primary: &'static str,
        legacy: &'static str,
        depth: usize,
    ) -> Result<FunctionId, DecodeError> {
        match optional(object, primary).or_else(|| optional(object, legacy)) {
            Some(raw) => self.decode(raw, depth + 1),
            None => Err(DecodeError::Field(primary)),
        }
    }
    fn optional_input(
        &mut self,
        object: &Object,
        name: &'static str,
        fallback: f32,
        depth: usize,
    ) -> Result<FunctionId, DecodeError> {
        match optional(object, name) {
            Some(raw) => self.decode(raw, depth + 1),
            None => Ok(self.graph.insert(GraphNode::Constant(fallback))?),
        }
    }
    /// The noise a shift op offsets by: `noise` in 26.3, `argument` holding
    /// the noise name in 1.21.7 (`singleArgumentCodec(NoiseHolder.CODEC)`).
    fn shift_noise(&self, object: &Object) -> Result<NoiseId, DecodeError> {
        if object.contains_key("noise") {
            return self.noise(object);
        }
        let raw = required(object, "argument")?;
        if !raw.get().starts_with('"') {
            return Err(DecodeError::Field("shift noise"));
        }
        let name = identifier(&string(raw)?)?;
        self.noises
            .get(&name)
            .copied()
            .ok_or_else(|| DecodeError::Registry(format!("unknown noise reference: {name}")))
    }
    fn noise(&self, object: &Object) -> Result<NoiseId, DecodeError> {
        let raw = required(object, "noise")?;
        if !raw.get().starts_with('"') {
            return Err(DecodeError::Field("inline noise holder unsupported"));
        }
        let name = identifier(&string(raw)?)?;
        self.noises
            .get(&name)
            .copied()
            .ok_or_else(|| DecodeError::Registry(format!("unknown noise reference: {name}")))
    }
    fn decode(&mut self, raw: &RawValue, depth: usize) -> Result<FunctionId, DecodeError> {
        if depth > 256 {
            return Err(DecodeError::Field("density nesting"));
        }
        if raw.get().starts_with('"') {
            return self.resolve(&identifier(&string(raw)?)?, depth);
        }
        if !raw.get().starts_with('{') {
            return Ok(self.graph.insert(GraphNode::Constant(noise_value(raw)?))?);
        }
        let object: Object = serde_json::from_str(raw.get())?;
        let kind = identifier(&string(required(&object, "type")?)?)?;
        let unary = match kind.as_str() {
            "minecraft:abs" => Some(GraphUnary::Abs),
            "minecraft:square" => Some(GraphUnary::Square),
            "minecraft:cube" => Some(GraphUnary::Cube),
            "minecraft:sqrt" => Some(GraphUnary::Sqrt),
            "minecraft:half_negative" => Some(GraphUnary::HalfNegative),
            "minecraft:quarter_negative" => Some(GraphUnary::QuarterNegative),
            "minecraft:reciprocal" => Some(GraphUnary::Reciprocal),
            "minecraft:negate" => Some(GraphUnary::Negate),
            "minecraft:squeeze" => Some(GraphUnary::Squeeze),
            "minecraft:log" => Some(GraphUnary::Log),
            "minecraft:sign" => Some(GraphUnary::Sign),
            _ => None,
        };
        let binary = match kind.as_str() {
            "minecraft:add" => Some(GraphBinary::Add),
            "minecraft:sub" => Some(GraphBinary::Subtract),
            "minecraft:mul" => Some(GraphBinary::Multiply),
            "minecraft:div" => Some(GraphBinary::Divide),
            "minecraft:min" => Some(GraphBinary::Minimum),
            "minecraft:max" => Some(GraphBinary::Maximum),
            _ => None,
        };
        let node = if let Some(kind) = unary {
            GraphNode::Unary {
                kind,
                input: self.input_named(&object, "input", "argument", depth)?,
            }
        } else if let Some(kind) = binary {
            GraphNode::Binary {
                kind,
                left: self.input_named(&object, "left", "argument1", depth)?,
                right: self.input_named(&object, "right", "argument2", depth)?,
            }
        } else {
            match kind.as_str() {
                "minecraft:constant" => {
                    GraphNode::Constant(noise_value(required(&object, "value")?)?)
                }
                "minecraft:blend_alpha" => GraphNode::WorldField(WorldField::BlendAlpha),
                "minecraft:blend_offset" => GraphNode::WorldField(WorldField::BlendOffset),
                "minecraft:beardifier" => GraphNode::WorldField(WorldField::Beardifier),
                // Vanilla 1.21.7's `minecraft:end_islands` includes the central
                // island cone in the function itself
                // (EndIslandDensityFunction seeds its height with
                // clamp(100 - sqrt(sx^2+sz^2)*8, -100, 80)); 26.3 renamed
                // the function to `end_outer_islands` and moved the cone
                // into the data graph (`minecraft:end/islands`). The two
                // names must not share one node, or the 1.21.7 main island
                // comes out as air. Carried from 26.2.
                "minecraft:end_islands" => GraphNode::EndIslands { central: true },
                "minecraft:end_outer_islands" => GraphNode::EndIslands { central: false },
                // 1.21.7 has three cache types (flat_cache, cache_2d,
                // cache_once); 26.x merged them into one. All three are
                // value-identity, so they decode to the same node.
                "minecraft:cache"
                | "minecraft:flat_cache"
                | "minecraft:cache_2d"
                | "minecraft:cache_once" => {
                    GraphNode::Cache(self.input_named(&object, "input", "argument", depth)?)
                }
                "minecraft:blend_density" => {
                    GraphNode::BlendDensity(self.input_named(&object, "input", "argument", depth)?)
                }
                "minecraft:pow" => GraphNode::Power {
                    base: self.input(&object, "base", depth)?,
                    exponent: self.input(&object, "exponent", depth)?,
                },
                "minecraft:floor" | "minecraft:round" | "minecraft:ceil" | "minecraft:truncate" => {
                    GraphNode::Round {
                        mode: match kind.as_str() {
                            "minecraft:floor" => RoundMode::Floor,
                            "minecraft:round" => RoundMode::Round,
                            "minecraft:ceil" => RoundMode::Ceil,
                            _ => RoundMode::Truncate,
                        },
                        input: self.input(&object, "input", depth)?,
                        multiple: self.optional_input(&object, "multiple", 1.0, depth)?,
                    }
                }
                // 1.21.7's y_clamped_gradient is the Y-only gradient 26.x
                // generalised: same clamping, same fields under y names.
                "minecraft:y_clamped_gradient" => {
                    let from = integer(required(&object, "from_y")?)?;
                    let to = integer(required(&object, "to_y")?)?;
                    if from == to {
                        return Err(DecodeError::Field("gradient coordinates"));
                    }
                    GraphNode::Gradient {
                        axis: Axis::Y,
                        tiling: TilingMode::ClampToEdge,
                        from,
                        to,
                        low: noise_value(required(&object, "from_value")?)?,
                        high: noise_value(required(&object, "to_value")?)?,
                    }
                }
                "minecraft:gradient" => {
                    let from = integer(required(&object, "from_coordinate")?)?;
                    let to = integer(required(&object, "to_coordinate")?)?;
                    if from == to {
                        return Err(DecodeError::Field("gradient coordinates"));
                    }
                    GraphNode::Gradient {
                        axis: axis(required(&object, "axis")?)?,
                        tiling: match optional(&object, "tiling")
                            .map(string)
                            .transpose()?
                            .as_deref()
                            .unwrap_or("clamp_to_edge")
                        {
                            "clamp_to_edge" => TilingMode::ClampToEdge,
                            "repeat" => TilingMode::Repeat,
                            "mirrored_repeat" => TilingMode::MirroredRepeat,
                            _ => return Err(DecodeError::Field("tiling")),
                        },
                        from,
                        to,
                        low: noise_value(required(&object, "from_value")?)?,
                        high: noise_value(required(&object, "to_value")?)?,
                    }
                }
                // 1.21.7's shifted_noise is the noise function with shifts;
                // 26.x merged it into `noise` (shifts optional there).
                "minecraft:noise" | "minecraft:shifted_noise" => GraphNode::Noise {
                    noise: self.noise(&object)?,
                    xz_scale: double(required(&object, "xz_scale")?)?,
                    y_scale: double(required(&object, "y_scale")?)?,
                    shift_x: self.optional_input(&object, "shift_x", 0.0, depth)?,
                    shift_y: self.optional_input(&object, "shift_y", 0.0, depth)?,
                    shift_z: self.optional_input(&object, "shift_z", 0.0, depth)?,
                },
                "minecraft:shift" | "minecraft:shift_a" | "minecraft:shift_b" => {
                    GraphNode::OffsetNoise {
                        noise: self.shift_noise(&object)?,
                        kind: match kind.as_str() {
                            "minecraft:shift" => ShiftNoiseKind::Shift,
                            "minecraft:shift_a" => ShiftNoiseKind::ShiftA,
                            _ => ShiftNoiseKind::ShiftB,
                        },
                    }
                }
                "minecraft:old_blended_noise" => GraphNode::BlendedNoise(BlendedNoise {
                    xz_scale: double_range(
                        required(&object, "xz_scale")?,
                        "xz_scale",
                        0.001,
                        1000.0,
                    )?,
                    y_scale: double_range(required(&object, "y_scale")?, "y_scale", 0.001, 1000.0)?,
                    xz_factor: double_range(
                        required(&object, "xz_factor")?,
                        "xz_factor",
                        0.001,
                        1000.0,
                    )?,
                    y_factor: double_range(
                        required(&object, "y_factor")?,
                        "y_factor",
                        0.001,
                        1000.0,
                    )?,
                    smear_scale_multiplier: double_range(
                        required(&object, "smear_scale_multiplier")?,
                        "smear_scale_multiplier",
                        1.0,
                        8.0,
                    )?,
                }),
                "minecraft:slice" => GraphNode::Slice {
                    axis: axis(required(&object, "axis")?)?,
                    coordinate: integer(required(&object, "coordinate")?)?,
                    input: self.input(&object, "input", depth)?,
                },
                "minecraft:interpolated" => {
                    // 1.21.7's interpolated takes no cell sizes: it
                    // interpolates on the dimension's noise cell. Only the
                    // overworld uses it (noodle caves), and its cell is 4
                    // wide, 8 tall.
                    let cell = |name: &'static str, default: i32| {
                        optional(&object, name)
                            .map(|raw| int_range(raw, name, 1, i32::MAX))
                            .transpose()
                            .map(|value| value.unwrap_or(default))
                    };
                    GraphNode::Interpolated {
                        input: self.input_named(&object, "input", "argument", depth)?,
                        xz: cell("cell_size_xz", 4)?,
                        y: cell("cell_size_y", 8)?,
                    }
                }
                "minecraft:find_top_surface" => GraphNode::Surface {
                    density: self.input(&object, "density", depth)?,
                    upper: self.input(&object, "upper_bound", depth)?,
                    lower: int_range(
                        required(&object, "lower_bound")?,
                        "lower_bound",
                        -4064,
                        4062,
                    )?,
                    height: int_range(
                        required(&object, "cell_height")?,
                        "cell_height",
                        1,
                        i32::MAX,
                    )?,
                },
                "minecraft:clamp" => {
                    let min = noise_value(required(&object, "min")?)?;
                    let max = noise_value(required(&object, "max")?)?;
                    if max < min {
                        return Err(DecodeError::Field("clamp bounds"));
                    }
                    GraphNode::Clamp {
                        input: self.input(&object, "input", depth)?,
                        min,
                        max,
                    }
                }
                "minecraft:range_choice" => GraphNode::Choice {
                    input: self.input(&object, "input", depth)?,
                    min: noise_value(required(&object, "min_inclusive")?)?,
                    max: noise_value(required(&object, "max_exclusive")?)?,
                    inside: self.input(&object, "when_in_range", depth)?,
                    outside: self.input(&object, "when_out_of_range", depth)?,
                },
                "minecraft:lerp" => GraphNode::Lerp {
                    alpha: self.input(&object, "alpha", depth)?,
                    first: self.input(&object, "first", depth)?,
                    second: self.input(&object, "second", depth)?,
                },
                "minecraft:interval_select" => {
                    let thresholds: Vec<Box<RawValue>> =
                        serde_json::from_str(required(&object, "thresholds")?.get())?;
                    let thresholds: Vec<f32> = thresholds
                        .iter()
                        .map(|raw| noise_value(raw))
                        .collect::<Result<_, _>>()?;
                    let functions: Vec<Box<RawValue>> =
                        serde_json::from_str(required(&object, "functions")?.get())?;
                    if functions.len() < 2
                        || functions.len() > 256
                        || functions.len() != thresholds.len() + 1
                        || thresholds
                            .windows(2)
                            .any(|pair| pair[0].total_cmp(&pair[1]).is_gt())
                    {
                        return Err(DecodeError::Field("interval selection"));
                    }
                    let input = self.input(&object, "input", depth)?;
                    let branches = functions
                        .iter()
                        .map(|raw| self.decode(raw, depth + 1))
                        .collect::<Result<_, _>>()?;
                    GraphNode::IntervalSelect {
                        input,
                        thresholds: Arc::from(thresholds),
                        branches,
                    }
                }
                "minecraft:distance_to_point" => {
                    let raw: Vec<Box<RawValue>> =
                        serde_json::from_str(required(&object, "point")?.get())?;
                    if raw.len() != 3 {
                        return Err(DecodeError::Field("point"));
                    }
                    let point = [integer(&raw[0])?, integer(&raw[1])?, integer(&raw[2])?];
                    let metric = match string(required(&object, "metric")?)?.as_str() {
                        "euclidean" => DistanceMetric::Euclidean,
                        "euclidean_squared" => DistanceMetric::EuclideanSquared,
                        "manhattan" => DistanceMetric::Manhattan,
                        "chebyshev" => DistanceMetric::Chebyshev,
                        _ => return Err(DecodeError::Field("metric")),
                    };
                    GraphNode::DistanceToPoint { point, metric }
                }
                "minecraft:spline" => {
                    GraphNode::Spline(self.spline(required(&object, "spline")?, depth + 1)?)
                }
                "minecraft:weird_scaled_sampler" => {
                    // 1.21.7-only: 26.x dropped this type. Vanilla maps the
                    // input through a rarity mapper (type_1/type_2) and
                    // multiplies by the noise sampled at raw coordinates;
                    // the exact mapper curves are not reproduced here —
                    // the input is clamped to [0, 1] as the rarity gate,
                    // which keeps spaghetti caves roughly where vanilla
                    // puts them.
                    match string(required(&object, "rarity_value_mapper")?)?.as_str() {
                        "type_1" | "type_2" => {}
                        _ => return Err(DecodeError::Field("rarity_value_mapper")),
                    }
                    let input = self.input(&object, "input", depth)?;
                    let gated = self
                        .graph
                        .insert(GraphNode::Clamp { input, min: 0.0, max: 1.0 })?;
                    let zero = self.graph.insert(GraphNode::Constant(0.0))?;
                    let noise = self.graph.insert(GraphNode::Noise {
                        noise: self.noise(&object)?,
                        xz_scale: 1.0,
                        y_scale: 1.0,
                        shift_x: zero,
                        shift_y: zero,
                        shift_z: zero,
                    })?;
                    GraphNode::Binary {
                        kind: GraphBinary::Multiply,
                        left: gated,
                        right: noise,
                    }
                }
                _ => {
                    return Err(DecodeError::Registry(format!(
                        "unknown density type: {kind}"
                    )));
                }
            }
        };
        Ok(self.graph.insert(node)?)
    }
    fn spline(
        &mut self,
        raw: &RawValue,
        depth: usize,
    ) -> Result<CubicSpline<FunctionId>, DecodeError> {
        if depth > 256 {
            return Err(DecodeError::Field("spline nesting"));
        }
        if !raw.get().starts_with('{') {
            return Ok(CubicSpline::Constant(float(raw)?));
        }
        let object: Object = serde_json::from_str(raw.get())?;
        let coordinate = self.input(&object, "coordinate", depth)?;
        let points: Vec<Box<RawValue>> = serde_json::from_str(required(&object, "points")?.get())?;
        let points = points
            .iter()
            .map(|raw| {
                let point: Object = serde_json::from_str(raw.get())?;
                Ok(SplinePoint {
                    location: float(required(&point, "location")?)?,
                    derivative: float(required(&point, "derivative")?)?,
                    value: self.spline(required(&point, "value")?, depth + 1)?,
                })
            })
            .collect::<Result<_, DecodeError>>()?;
        CubicSpline::multipoint(coordinate, points)
            .map_err(|error| DecodeError::Graph(error.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_core::density::{SamplerContext, graph::DensityCompiler};
    #[test]
    fn numeric_tokens_keep_negative_zero_and_avoid_double_rounding() {
        let densities = BTreeMap::from([
            ("minecraft:negative_zero".into(), "-0".into()),
            (
                "minecraft:rounded".into(),
                "1.0000000596046447753906250000000001".into(),
            ),
        ]);
        let registry = DensityRegistry::from_json(&BTreeMap::new(), &densities).unwrap();
        let zero = registry.value("minecraft:negative_zero").unwrap();
        let rounded = registry.value("minecraft:rounded").unwrap();
        let compiler = DensityCompiler::new(registry.into_graph());
        let mut context = SamplerContext::default();
        assert_eq!(
            compiler
                .compile_raw(zero)
                .unwrap()
                .sample_value(&mut context, [0; 3])
                .unwrap()
                .to_bits(),
            0x8000_0000
        );
        assert_eq!(
            compiler
                .compile_raw(rounded)
                .unwrap()
                .sample_value(&mut context, [0; 3])
                .unwrap()
                .to_bits(),
            0x3f80_0001
        );
    }
    #[test]
    fn forward_references_share_named_holders_and_reject_cycles_or_missing_values() {
        let mut densities = BTreeMap::from([
            (
                "minecraft:a".into(),
                r#"{"type":"add","left":"z","right":"minecraft:z"}"#.into(),
            ),
            ("minecraft:z".into(), "0.5".into()),
        ]);
        let registry = DensityRegistry::from_json(&BTreeMap::new(), &densities).unwrap();
        let root = registry.value("minecraft:a").unwrap();
        let compiler = DensityCompiler::new(registry.into_graph());
        assert_eq!(
            compiler
                .compile_raw(root)
                .unwrap()
                .sample_value(&mut SamplerContext::default(), [0; 3])
                .unwrap(),
            1.0
        );
        densities.insert("minecraft:z".into(), r#""a""#.into());
        assert!(DensityRegistry::from_json(&BTreeMap::new(), &densities).is_err());
        densities.remove("minecraft:z");
        assert!(DensityRegistry::from_json(&BTreeMap::new(), &densities).is_err());
    }
    #[test]
    fn legacy_argument_spellings_and_cache_types_decode() {
        let densities = BTreeMap::from([(
            "minecraft:a".into(),
            r#"{"type":"add","argument1":1,"argument2":{"type":"y_clamped_gradient","from_y":0,"to_y":10,"from_value":0,"to_value":1}}"#.into(),
        )]);
        let registry = DensityRegistry::from_json(&BTreeMap::new(), &densities).unwrap();
        let root = registry.value("minecraft:a").unwrap();
        let compiler = DensityCompiler::new(registry.into_graph());
        let compiled = compiler.compile_raw(root).unwrap();
        let mut context = SamplerContext::default();
        assert_eq!(
            compiled.sample_value(&mut context, [0, 0, 0]).unwrap(),
            1.0
        );
        assert_eq!(
            compiled.sample_value(&mut context, [0, 5, 0]).unwrap(),
            1.5
        );
        // The three 1.21.7 cache types decode (sampling them needs cache
        // preparation, which only the generator does).
        let caches = BTreeMap::from([
            (
                "minecraft:c1".into(),
                r#"{"type":"cache_once","argument":1}"#.into(),
            ),
            (
                "minecraft:c2".into(),
                r#"{"type":"flat_cache","argument":1}"#.into(),
            ),
            (
                "minecraft:c3".into(),
                r#"{"type":"cache_2d","argument":1}"#.into(),
            ),
        ]);
        DensityRegistry::from_json(&BTreeMap::new(), &caches).unwrap();
    }
    #[test]
    fn shift_forms_accept_a_noise_name_or_an_argument() {
        let noises = BTreeMap::from([(
            "minecraft:offset".into(),
            r#"{"firstOctave":0,"amplitudes":[1.0]}"#.into(),
        )]);
        let densities = BTreeMap::from([
            (
                "minecraft:shift_a".into(),
                r#"{"type":"shift_a","argument":"minecraft:offset"}"#.into(),
            ),
            (
                "minecraft:shift_b".into(),
                r#"{"type":"shift_b","noise":"minecraft:offset"}"#.into(),
            ),
            (
                "minecraft:shifted".into(),
                r#"{"type":"shifted_noise","noise":"minecraft:offset","xz_scale":0.25,"y_scale":0,"shift_x":"minecraft:shift_a","shift_y":0,"shift_z":"minecraft:shift_b"}"#.into(),
            ),
        ]);
        let registry = DensityRegistry::from_json(&noises, &densities).unwrap();
        for name in [
            "minecraft:shift_a",
            "minecraft:shift_b",
            "minecraft:shifted",
        ] {
            assert!(registry.value(name).is_some(), "{name}");
        }
    }
    #[test]
    fn codec_validation_rejects_invalid_gradients_intervals_and_splines() {
        for json in [
            r#"{"type":"gradient","axis":"y","from_coordinate":0,"to_coordinate":0,"from_value":0,"to_value":1}"#,
            r#"{"type":"interval_select","input":0,"thresholds":[0,-0],"functions":[0,1,2]}"#,
            r#"{"type":"interpolated","input":0,"cell_size_xz":0,"cell_size_y":8}"#,
            r#"{"type":"spline","spline":{"coordinate":0,"points":[]}}"#,
            r#"{"type":"clamp","input":0,"min":1,"max":-1}"#,
        ] {
            assert!(
                DensityRegistry::from_json(
                    &BTreeMap::new(),
                    &BTreeMap::from([("minecraft:a".into(), json.into())])
                )
                .is_err(),
                "{json}"
            );
        }
    }
}
