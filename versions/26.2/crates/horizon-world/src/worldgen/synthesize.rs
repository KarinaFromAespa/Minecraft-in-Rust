//! Density functions vanilla 26.2 derives from the noise router instead of
//! naming in data, synthesized as graph nodes at pipeline load.
//!
//! 26.3 moved these into data (the aquifer config object, the ore-vein
//! material rules, the debug functions); 26.2 computes them from the router
//! fields: the aquifer's deep-dark exclusion from erosion/depth, the ore
//! veins from the vein fields, and the debug line's peaks-and-valleys from
//! ridges. The vein synthesis follows Mojang's own 26.3 translation of
//! `OreVeinifier` into density functions (the 26.3 `ore_vein/*` functions),
//! with the mask read off the router's `vein_ridged` field.
use super::{
    DecodeError,
    density::DensityRegistry,
    router::{NoiseRouter, RouterField},
};
use horizon_core::{
    density::{
        TilingMode,
        graph::{FunctionId, GraphBinary, GraphNode, GraphUnary},
    },
    geometry::Axis,
};

fn constant(registry: &mut DensityRegistry, value: f32) -> Result<FunctionId, DecodeError> {
    registry.insert(GraphNode::Constant(value))
}
fn binary(
    registry: &mut DensityRegistry,
    kind: GraphBinary,
    left: FunctionId,
    right: FunctionId,
) -> Result<FunctionId, DecodeError> {
    registry.insert(GraphNode::Binary { kind, left, right })
}
fn unary(
    registry: &mut DensityRegistry,
    kind: GraphUnary,
    input: FunctionId,
) -> Result<FunctionId, DecodeError> {
    registry.insert(GraphNode::Unary { kind, input })
}
fn choice(
    registry: &mut DensityRegistry,
    input: FunctionId,
    min: f32,
    max: f32,
    inside: FunctionId,
    outside: FunctionId,
) -> Result<FunctionId, DecodeError> {
    registry.insert(GraphNode::Choice {
        input,
        min,
        max,
        inside,
        outside,
    })
}
fn clamp(
    registry: &mut DensityRegistry,
    input: FunctionId,
    min: f32,
    max: f32,
) -> Result<FunctionId, DecodeError> {
    registry.insert(GraphNode::Clamp { input, min, max })
}

/// The `minecraft:y` identity gradient, from the registry when the bundle
/// defines it (both vanilla versions do) or synthesized for synthetic
/// settings that do not.
pub(crate) fn y_function(registry: &mut DensityRegistry) -> Result<FunctionId, DecodeError> {
    if let Some(id) = registry.reference("minecraft:y") {
        return Ok(id);
    }
    registry.insert(GraphNode::Gradient {
        axis: Axis::Y,
        tiling: TilingMode::ClampToEdge,
        from: -4064,
        to: 4062,
        low: -4064.0,
        high: 4062.0,
    })
}

/// The aquifer's deep-dark exclusion: `min(-0.225 - erosion, max(depth - 0.9, 0))`,
/// positive exactly where `OverworldBiomeBuilder.isDeepDarkRegion`
/// (erosion < -0.225 and depth > 0.9) holds. 26.3's aquifer config names
/// this function directly; 26.2's aquifer takes it from the router.
pub(crate) fn aquifer_exclusion(
    router: &NoiseRouter,
    registry: &mut DensityRegistry,
) -> Result<FunctionId, DecodeError> {
    let erosion = router.function(RouterField::Erosion);
    let depth = router.function(RouterField::Depth);
    let minus_threshold = constant(registry, -0.225)?;
    let below = binary(registry, GraphBinary::Subtract, minus_threshold, erosion)?;
    let threshold = constant(registry, 0.9)?;
    let above = binary(registry, GraphBinary::Subtract, depth, threshold)?;
    let zero = constant(registry, 0.0)?;
    let above = binary(registry, GraphBinary::Maximum, above, zero)?;
    binary(registry, GraphBinary::Minimum, below, above)
}

/// `TerrainProvider.peaksAndValleys`: `-(||x| - 0.6666667| - 0.33333334) * 3`,
/// sampled over the ridges field for the debug line.
pub(crate) fn peaks_and_valleys(
    router: &NoiseRouter,
    registry: &mut DensityRegistry,
) -> Result<FunctionId, DecodeError> {
    let ridges = router.function(RouterField::Ridges);
    let abs = unary(registry, GraphUnary::Abs, ridges)?;
    let first = constant(registry, 0.6666667)?;
    let shifted = binary(registry, GraphBinary::Subtract, abs, first)?;
    let abs = unary(registry, GraphUnary::Abs, shifted)?;
    let second = constant(registry, 0.33333334)?;
    let shifted = binary(registry, GraphBinary::Subtract, abs, second)?;
    let negated = unary(registry, GraphUnary::Negate, shifted)?;
    let three = constant(registry, 3.0)?;
    binary(registry, GraphBinary::Multiply, negated, three)
}

/// One of 26.2's two `OreVeinifier$VeinType`s, with its blocks and y range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum VeinKind {
    Copper,
    Iron,
}
impl VeinKind {
    pub(crate) const ALL: [Self; 2] = [Self::Copper, Self::Iron];
    fn min_y(self) -> i32 {
        match self {
            Self::Copper => 0,
            Self::Iron => -60,
        }
    }
    fn max_y(self) -> i32 {
        match self {
            Self::Copper => 50,
            Self::Iron => -8,
        }
    }
    /// Inclusive y bounds the vein's density is positive over.
    pub(crate) fn positive_y(self) -> [i32; 2] {
        [self.min_y(), self.max_y() - 1]
    }
    /// (ore, raw ore, filler) block names.
    pub(crate) fn blocks(self) -> [&'static str; 3] {
        match self {
            Self::Copper => [
                "minecraft:copper_ore",
                "minecraft:raw_copper_block",
                "minecraft:granite",
            ],
            Self::Iron => [
                "minecraft:deepslate_iron_ore",
                "minecraft:raw_iron_block",
                "minecraft:tuff",
            ],
        }
    }
}

/// The three density roots of one ore vein, in `MaterialRule::OreVein` order.
#[derive(Clone, Copy, Debug)]
pub(crate) struct VeinRoots {
    pub density: FunctionId,
    pub richness: FunctionId,
    pub filler_gap: FunctionId,
}

/// The three density functions of one 26.2 ore vein, following Mojang's 26.3
/// `ore_vein/{copper,iron}_density`, `richness` and `gap` functions, with
/// the router's vein fields standing in for 26.3's named functions:
///
/// - density: y in the vein's range, the mask non-negative, and
///   `(signed toggle - 0.4) + edge` non-negative, where the edge roundoff is
///   `clamp(min(maxY - y, y - minY), 0, 20) * 0.01 - 0.2`; 0.7 inside, -1
///   outside. The mask is `-vein_ridged`: 26.2's `vein_ridged` is
///   `-0.08 + max(|interpA|, |interpB|)`, so its negation is exactly 26.3's
///   `0.08 - max(...)` (IEEE negation is exact). 26.3's mask also gates on
///   `|toggle| >= 0.4`, which the density gate already implies
///   (toggle - 0.4 + edge >= 0 with edge <= 0 forces toggle >= 0.4), so it
///   is dropped. The y-range and out-of-range differences between 26.2's
///   vein fields and 26.3's named functions sit outside the vein's y range
///   and are unobservable through the density gate.
/// - richness: `clamp(|toggle|, 0.4, 0.6) - 0.3`, the `clampedMap` of
///   `OreVeinifier`'s veininess from [0.4, 0.6] to [0.1, 0.3].
/// - filler gap: `-0.3 - vein_gap`, negative exactly where vanilla places ore.
pub(crate) fn vein_roots(
    router: &NoiseRouter,
    registry: &mut DensityRegistry,
    kind: VeinKind,
) -> Result<VeinRoots, DecodeError> {
    let toggle = router.function(RouterField::VeinToggle);
    let ridged = router.function(RouterField::VeinRidged);
    let gap = router.function(RouterField::VeinGap);
    let y = y_function(registry)?;
    let signed = match kind {
        VeinKind::Copper => toggle,
        VeinKind::Iron => unary(registry, GraphUnary::Negate, toggle)?,
    };
    let min_y = kind.min_y();
    let max_y = kind.max_y();
    let max_y_const = constant(registry, max_y as f32)?;
    let dist_top = binary(registry, GraphBinary::Subtract, max_y_const, y)?;
    let min_y_const = constant(registry, min_y as f32)?;
    let dist_bottom = binary(registry, GraphBinary::Subtract, y, min_y_const)?;
    let dist = binary(registry, GraphBinary::Minimum, dist_top, dist_bottom)?;
    let clamped = clamp(registry, dist, 0.0, 20.0)?;
    let hundredth = constant(registry, 0.01)?;
    let scaled = binary(registry, GraphBinary::Multiply, clamped, hundredth)?;
    let fifth = constant(registry, 0.2)?;
    let edge = binary(registry, GraphBinary::Subtract, scaled, fifth)?;
    let threshold = constant(registry, 0.4)?;
    let over = binary(registry, GraphBinary::Subtract, signed, threshold)?;
    let gate = binary(registry, GraphBinary::Add, over, edge)?;
    let inside = constant(registry, 0.7)?;
    let outside = constant(registry, -1.0)?;
    let inner = choice(registry, gate, 0.0, 1.0e6, inside, outside)?;
    let mask = unary(registry, GraphUnary::Negate, ridged)?;
    let mid = choice(registry, mask, 0.0, 1.0e6, inner, outside)?;
    let density = choice(registry, y, min_y as f32, max_y as f32, mid, outside)?;
    let abs_toggle = unary(registry, GraphUnary::Abs, toggle)?;
    let clamped = clamp(registry, abs_toggle, 0.4, 0.6)?;
    let base = constant(registry, 0.3)?;
    let richness = binary(registry, GraphBinary::Subtract, clamped, base)?;
    let minus_threshold = constant(registry, -0.3)?;
    let filler_gap = binary(registry, GraphBinary::Subtract, minus_threshold, gap)?;
    Ok(VeinRoots {
        density,
        richness,
        filler_gap,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_core::density::{SamplerContext, graph::DensityCompiler};
    use std::collections::BTreeMap;

    /// A router over constant densities, for sampling the synthesized roots.
    fn constant_router() -> (NoiseRouter, DensityRegistry) {
        let densities = BTreeMap::from([
            ("minecraft:erosion".into(), "-0.5".into()),
            ("minecraft:depth".into(), "1.0".into()),
            ("minecraft:ridges".into(), "0.5".into()),
            ("minecraft:toggle".into(), "0.5".into()),
            ("minecraft:ridged".into(), "-0.05".into()),
            ("minecraft:gap".into(), "0.0".into()),
            ("minecraft:zero".into(), "0.0".into()),
            (
                "minecraft:y".into(),
                r#"{"type":"y_clamped_gradient","from_y":-4064,"to_y":4062,"from_value":-4064,"to_value":4062}"#.into(),
            ),
        ]);
        let mut registry = DensityRegistry::from_json(&BTreeMap::new(), &densities).unwrap();
        let fields = BTreeMap::from([
            ("temperature", "minecraft:zero"),
            ("vegetation", "minecraft:zero"),
            ("continents", "minecraft:zero"),
            ("erosion", "minecraft:erosion"),
            ("depth", "minecraft:depth"),
            ("ridges", "minecraft:ridges"),
            ("barrier", "minecraft:zero"),
            ("fluid_level_floodedness", "minecraft:zero"),
            ("fluid_level_spread", "minecraft:zero"),
            ("lava", "minecraft:zero"),
            ("preliminary_surface_level", "minecraft:zero"),
            ("final_density", "minecraft:zero"),
            ("vein_toggle", "minecraft:toggle"),
            ("vein_ridged", "minecraft:ridged"),
            ("vein_gap", "minecraft:gap"),
        ]);
        let json = serde_json::to_string(&fields).unwrap();
        let router = NoiseRouter::from_json(&json, &mut registry).unwrap();
        (router, registry)
    }

    fn sample(registry: &DensityRegistry, id: FunctionId, at: [i32; 3]) -> f32 {
        let mut compiler = DensityCompiler::new(registry.graph().clone());
        compiler
            .get_sampler(id)
            .unwrap()
            .sample_value(&mut SamplerContext::default(), at)
            .unwrap()
    }

    #[test]
    fn exclusion_is_positive_only_in_the_deep_dark() {
        let (router, mut registry) = constant_router();
        // min(-0.225 - (-0.5), max(1.0 - 0.9, 0)) = min(0.275, 0.1);
        // 1.0 - 0.9 rounds to 0.100000024 in f32.
        let id = aquifer_exclusion(&router, &mut registry).unwrap();
        let value = sample(&registry, id, [0; 3]);
        assert!((value - 0.1).abs() < 1e-6, "{value}");
    }

    #[test]
    fn peaks_and_valleys_follows_the_vanilla_formula() {
        let (router, mut registry) = constant_router();
        // -(| |0.5| - 0.6666667 | - 0.33333334) * 3 = 0.5.
        let id = peaks_and_valleys(&router, &mut registry).unwrap();
        let value = sample(&registry, id, [0; 3]);
        assert!((value - 0.5).abs() < 1e-6, "{value}");
    }

    #[test]
    fn vein_roots_follow_the_ore_veinifier() {
        let (router, mut registry) = constant_router();
        let copper = vein_roots(&router, &mut registry, VeinKind::Copper).unwrap();
        let iron = vein_roots(&router, &mut registry, VeinKind::Iron).unwrap();
        // Copper at y 25, toggle 0.5: mask -(-0.05) = 0.05, edge
        // clamp(min(25, 25), 0, 20) * 0.01 - 0.2 = 0, gate 0.1 -> 0.7.
        assert_eq!(sample(&registry, copper.density, [0, 25, 0]), 0.7);
        // Outside the copper y range the density is -1.
        assert_eq!(sample(&registry, copper.density, [0, 60, 0]), -1.0);
        // Iron needs a negative toggle: with +0.5 the gate fails.
        assert_eq!(sample(&registry, iron.density, [0, -30, 0]), -1.0);
        // Richness: clamp(|0.5|, 0.4, 0.6) - 0.3 = 0.2 (0.5 - 0.3 rounds to
        // 0.19999999 in f32); gap: -0.3 - 0.
        let richness = sample(&registry, copper.richness, [0, 25, 0]);
        assert!((richness - 0.2).abs() < 1e-6, "{richness}");
        assert_eq!(sample(&registry, copper.filler_gap, [0, 25, 0]), -0.3);
    }
}
