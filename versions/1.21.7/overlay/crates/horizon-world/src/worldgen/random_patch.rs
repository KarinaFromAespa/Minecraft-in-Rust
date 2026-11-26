//! Vanilla 1.21.7 `RandomPatchFeature`: tries a nested placed feature
//! `tries` times at triangularly distributed offsets within
//! `xz_spread`/`y_spread` of the origin.
//!
//! In 1.21.7 `minecraft:flower` and `minecraft:random_patch` are the same
//! class (`Feature.FLOWER` and `Feature.RANDOM_PATCH` are both
//! `RandomPatchFeature` with `RandomPatchConfiguration`); 26.3 removed
//! both, so the Rust had no decode arm for them.
use super::{
    DecodeError, Object,
    feature::FeatureWorld,
    feature_graph::PlacedFeatureDefinition,
    int_range,
    placement::PlacementContext,
    required,
    tree::TreeResult,
    vegetation::{PlacedDecoder, PlacedPlacer},
};
use horizon_core::random::RandomSource;
#[derive(Clone, Debug)]
pub struct RandomPatchFeature {
    feature: PlacedFeatureDefinition,
    tries: i32,
    xz_spread: i32,
    y_spread: i32,
}
impl RandomPatchFeature {
    pub(crate) fn decode(
        json: &str,
        depth: usize,
        remaining: &mut usize,
        nested: &PlacedDecoder<'_>,
    ) -> Result<Self, DecodeError> {
        let object: Object = serde_json::from_str(json)?;
        Ok(Self {
            feature: nested(required(&object, "feature")?.get(), depth + 1, remaining)?,
            tries: int_range(required(&object, "tries")?, "random patch tries", 1, 3000)?,
            xz_spread: int_range(
                required(&object, "xz_spread")?,
                "random patch xz spread",
                0,
                16,
            )?,
            y_spread: int_range(
                required(&object, "y_spread")?,
                "random patch y spread",
                0,
                16,
            )?,
        })
    }
    pub(crate) fn place<W: FeatureWorld + PlacementContext, R: RandomSource>(
        &self,
        world: &mut W,
        random: &mut R,
        origin: [i32; 3],
        nested: &mut PlacedPlacer<'_, W, R>,
    ) -> TreeResult<bool, W::Error> {
        // Vanilla draws `nextInt(spread + 1) - nextInt(spread + 1)` per
        // axis, in x, y, z order — a triangular distribution.
        let xz = self.xz_spread + 1;
        let y = self.y_spread + 1;
        let mut placed = 0;
        for _ in 0..self.tries {
            let dx = random.next_int_bound(xz)? - random.next_int_bound(xz)?;
            let dy = random.next_int_bound(y)? - random.next_int_bound(y)?;
            let dz = random.next_int_bound(xz)? - random.next_int_bound(xz)?;
            let p = [
                origin[0].wrapping_add(dx),
                origin[1].wrapping_add(dy),
                origin[2].wrapping_add(dz),
            ];
            if nested(&self.feature, world, random, p)? {
                placed += 1;
            }
        }
        Ok(placed > 0)
    }
}
