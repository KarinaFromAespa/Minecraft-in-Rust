//! `minecraft:flower`, `minecraft:no_bonemeal_flower` and
//! `minecraft:random_patch`: one nested placed feature tried at random
//! offsets around the origin (1.21.10's `RandomPatchFeature`; all three
//! registry entries are the same class, their configs' inner features
//! differ). 26.x dropped the class and rewrote the flower features as
//! `minecraft:simple_block`, so the 26.x tree never needed it.
use super::{
    DecodeError, Object,
    feature::FeatureWorld,
    feature_graph::PlacedFeatureDefinition,
    int_range,
    placement::PlacementContext,
    required,
    tree::{TreeResult, kind},
    vegetation::{PlacedDecoder, PlacedPlacer},
};
use horizon_core::random::RandomSource;

#[derive(Clone, Debug)]
pub struct RandomPatchFeature {
    feature: Box<PlacedFeatureDefinition>,
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
        match kind(&object)?.as_str() {
            "minecraft:flower" | "minecraft:no_bonemeal_flower" | "minecraft:random_patch" => {}
            _ => return Err(DecodeError::Field("random patch type")),
        }
        let int = |name: &'static str, low: i32, high: i32| {
            int_range(required(&object, name)?, name, low, high)
        };
        Ok(Self {
            feature: Box::new(nested(
                required(&object, "feature")?.get(),
                depth + 1,
                remaining,
            )?),
            tries: int("tries", 0, 4096)?,
            xz_spread: int("xz_spread", 0, 256)?,
            y_spread: int("y_spread", 0, 256)?,
        })
    }
    pub(crate) fn place<W: FeatureWorld + PlacementContext, R: RandomSource>(
        &self,
        world: &mut W,
        random: &mut R,
        origin: [i32; 3],
        nested: &mut PlacedPlacer<'_, W, R>,
    ) -> TreeResult<bool, W::Error> {
        let xz = self.xz_spread + 1;
        let y = self.y_spread + 1;
        let mut count = 0;
        for _ in 0..self.tries {
            let dx = random.next_int_bound(xz)? - random.next_int_bound(xz)?;
            let dy = random.next_int_bound(y)? - random.next_int_bound(y)?;
            let dz = random.next_int_bound(xz)? - random.next_int_bound(xz)?;
            let pos = [
                origin[0].wrapping_add(dx),
                origin[1].wrapping_add(dy),
                origin[2].wrapping_add(dz),
            ];
            if nested(&self.feature, world, random, pos)? {
                count += 1;
            }
        }
        Ok(count > 0)
    }
}
