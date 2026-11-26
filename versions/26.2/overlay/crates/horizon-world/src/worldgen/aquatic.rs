//! 26.2 water-vegetation features: kelp, seagrass and sea pickle.
//!
//! The 26.2 classes are `added` in the 26.3 -> 26.2 diff (26.3 replaced
//! them with record features); the Rust had no decode arm for any of them,
//! so every 26.2 configured feature of these types failed to decode.
use super::{
    DecodeError, Object,
    block_feature::FeatureBlockBindings,
    feature::FeatureWorld,
    number_text,
    placement::{PlacementContext, PlacementQueryError},
    required,
    value_provider::IntProvider,
};
use crate::{heightmap::HeightmapType, registry::BlockStateId};
use horizon_core::random::{InvalidBound, RandomSource};
use std::{collections::BTreeSet, sync::Arc};

fn offset(p: [i32; 3], d: [i32; 3]) -> [i32; 3] {
    std::array::from_fn(|i| p[i].wrapping_add(d[i]))
}

/// Vanilla `Mth.nextInt(random, min, max)`: inclusive bounds, drawn as
/// `nextInt(max - min + 1) + min` with wrapping int arithmetic.
pub(crate) fn next_int_range(
    random: &mut impl RandomSource,
    min: i32,
    max: i32,
) -> Result<i32, InvalidBound> {
    Ok(random.next_int_bound(max.wrapping_sub(min).wrapping_add(1))? + min)
}

#[derive(Debug)]
pub enum AquaticError<E> {
    World(E),
    Query(PlacementQueryError),
    Random(InvalidBound),
}
impl<E: std::fmt::Display> std::fmt::Display for AquaticError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::World(e) => e.fmt(f),
            Self::Query(e) => e.fmt(f),
            Self::Random(e) => e.fmt(f),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for AquaticError<E> {}
impl<E> From<InvalidBound> for AquaticError<E> {
    fn from(error: InvalidBound) -> Self {
        Self::Random(error)
    }
}
impl<E> From<PlacementQueryError> for AquaticError<E> {
    fn from(error: PlacementQueryError) -> Self {
        Self::Query(error)
    }
}

fn block_set(
    bindings: &FeatureBlockBindings<'_>,
    name: &str,
) -> Result<Arc<BTreeSet<BlockStateId>>, DecodeError> {
    bindings
        .blocks
        .get(name)
        .cloned()
        .ok_or_else(|| DecodeError::Field("aquatic block identity"))
}

/// Vanilla 26.2 `KelpFeature`: a column of kelp plant topped with a kelp
/// head (age 20-23) at the ocean-floor height, with early termination that
/// re-caps the block below when the column is cut short.
#[derive(Clone, Debug)]
pub struct KelpFeature {
    water: Arc<BTreeSet<BlockStateId>>,
    plant: BlockStateId,
    top: [BlockStateId; 4],
    top_default: BlockStateId,
    kelp: Arc<BTreeSet<BlockStateId>>,
}
impl KelpFeature {
    pub fn new(bindings: &FeatureBlockBindings<'_>) -> Result<Self, DecodeError> {
        let registry = bindings.registry();
        let get = |name| {
            registry
                .default_state(name)
                .ok_or_else(|| DecodeError::Registry(format!("unknown kelp block {name}")))
        };
        let plant = get("minecraft:kelp_plant")?;
        let top_default = get("minecraft:kelp")?;
        let mut top = [top_default; 4];
        for (i, id) in top.iter_mut().enumerate() {
            let mut state = registry.state(top_default).unwrap().clone();
            state.properties.insert("age".into(), (20 + i).to_string());
            *id = registry.id(&state).ok_or(DecodeError::Field("kelp age"))?;
        }
        Ok(Self {
            water: block_set(bindings, "minecraft:water")?,
            plant,
            top,
            top_default,
            kelp: block_set(bindings, "minecraft:kelp")?,
        })
    }
    pub fn place<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        random: &mut impl RandomSource,
        origin: [i32; 3],
    ) -> Result<bool, AquaticError<W::Error>> {
        let mut pos = [
            origin[0],
            world.height(HeightmapType::OceanFloor, origin[0], origin[2])?,
            origin[2],
        ];
        if !self.is_water(world, pos)? {
            return Ok(false);
        }
        let height = 1 + random.next_int_bound(10)?;
        let mut placed = 0;
        for h in 0..=height {
            if self.is_water(world, pos)?
                && self.is_water(world, offset(pos, [0, 1, 0]))?
                && world.block_survives(self.plant, pos)?
            {
                if h == height {
                    let state = self.top[random.next_int_bound(4)? as usize];
                    world
                        .set_block(pos, state, 2)
                        .map_err(AquaticError::World)?;
                    placed += 1;
                } else {
                    world
                        .set_block(pos, self.plant, 2)
                        .map_err(AquaticError::World)?;
                }
            } else if h > 0 {
                let below = offset(pos, [0, -1, 0]);
                if world.block_survives(self.top_default, below)?
                    && !self.kelp.contains(
                        &world
                            .block(offset(below, [0, -1, 0]))
                            .map_err(AquaticError::World)?,
                    )
                {
                    let state = self.top[random.next_int_bound(4)? as usize];
                    world
                        .set_block(below, state, 2)
                        .map_err(AquaticError::World)?;
                    placed += 1;
                }
                break;
            }
            pos = offset(pos, [0, 1, 0]);
        }
        Ok(placed > 0)
    }
    fn is_water<W: FeatureWorld>(
        &self,
        world: &mut W,
        pos: [i32; 3],
    ) -> Result<bool, AquaticError<W::Error>> {
        Ok(self
            .water
            .contains(&world.block(pos).map_err(AquaticError::World)?))
    }
}

/// Vanilla 26.2 `SeagrassFeature`: one seagrass (or a tall pair) at a
/// random offset from the origin, at the ocean-floor height.
#[derive(Clone, Debug)]
pub struct SeagrassFeature {
    water: Arc<BTreeSet<BlockStateId>>,
    probability: f32,
    seagrass: BlockStateId,
    tall_lower: BlockStateId,
    tall_upper: BlockStateId,
}
impl SeagrassFeature {
    pub fn from_json(json: &str, bindings: &FeatureBlockBindings<'_>) -> Result<Self, DecodeError> {
        let object: Object = serde_json::from_str(json)?;
        let probability = number_text(required(&object, "probability")?)?
            .parse::<f64>()
            .map_err(|_| DecodeError::Field("seagrass probability"))?
            as f32;
        if !(0.0..=1.0).contains(&probability) || probability.is_sign_negative() {
            return Err(DecodeError::Field("seagrass probability"));
        }
        let registry = bindings.registry();
        let get = |name| {
            registry
                .default_state(name)
                .ok_or_else(|| DecodeError::Registry(format!("unknown seagrass block {name}")))
        };
        let seagrass = get("minecraft:seagrass")?;
        let tall_lower = get("minecraft:tall_seagrass")?;
        let mut upper = registry.state(tall_lower).unwrap().clone();
        upper.properties.insert("half".into(), "upper".into());
        let tall_upper = registry
            .id(&upper)
            .ok_or(DecodeError::Field("tall seagrass upper"))?;
        Ok(Self {
            water: block_set(bindings, "minecraft:water")?,
            probability,
            seagrass,
            tall_lower,
            tall_upper,
        })
    }
    pub fn place<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        random: &mut impl RandomSource,
        origin: [i32; 3],
    ) -> Result<bool, AquaticError<W::Error>> {
        let x = origin[0] + random.next_int_bound(8)? - random.next_int_bound(8)?;
        let z = origin[2] + random.next_int_bound(8)? - random.next_int_bound(8)?;
        let pos = [x, world.height(HeightmapType::OceanFloor, x, z)?, z];
        let mut placed = false;
        if self.is_water(world, pos)? {
            let tall = random.next_double() < self.probability as f64;
            let state = if tall { self.tall_lower } else { self.seagrass };
            if world.block_survives(state, pos)? {
                if tall {
                    let above = offset(pos, [0, 1, 0]);
                    if self.is_water(world, above)? {
                        world
                            .set_block(pos, self.tall_lower, 2)
                            .map_err(AquaticError::World)?;
                        world
                            .set_block(above, self.tall_upper, 2)
                            .map_err(AquaticError::World)?;
                    }
                } else {
                    world
                        .set_block(pos, self.seagrass, 2)
                        .map_err(AquaticError::World)?;
                }
                // Vanilla sets placedAny even when a tall plant's upper
                // block is not water and nothing was written.
                placed = true;
            }
        }
        Ok(placed)
    }
    fn is_water<W: FeatureWorld>(
        &self,
        world: &mut W,
        pos: [i32; 3],
    ) -> Result<bool, AquaticError<W::Error>> {
        Ok(self
            .water
            .contains(&world.block(pos).map_err(AquaticError::World)?))
    }
}

/// Vanilla 26.2 `SeaPickleFeature`: `count` attempts, each a sea pickle
/// with 1-4 pickles at a random offset, at the ocean-floor height.
#[derive(Clone, Debug)]
pub struct SeaPickleFeature {
    water: Arc<BTreeSet<BlockStateId>>,
    count: IntProvider,
    pickles: [BlockStateId; 4],
}
impl SeaPickleFeature {
    pub fn from_json(json: &str, bindings: &FeatureBlockBindings<'_>) -> Result<Self, DecodeError> {
        Self::decode(json, bindings, 0, &mut 65536)
    }
    pub(crate) fn decode(
        json: &str,
        bindings: &FeatureBlockBindings<'_>,
        depth: usize,
        remaining: &mut usize,
    ) -> Result<Self, DecodeError> {
        if depth > 128 || *remaining == 0 {
            return Err(DecodeError::Field("sea pickle depth or work"));
        }
        *remaining -= 1;
        let object: Object = serde_json::from_str(json)?;
        let count = IntProvider::decode(required(&object, "count")?, depth + 1, remaining)?;
        let registry = bindings.registry();
        let default = registry
            .default_state("minecraft:sea_pickle")
            .ok_or_else(|| DecodeError::Registry("unknown sea pickle block".into()))?;
        let mut pickles = [default; 4];
        for (i, id) in pickles.iter_mut().enumerate() {
            let mut state = registry.state(default).unwrap().clone();
            state
                .properties
                .insert("pickles".into(), (1 + i).to_string());
            *id = registry
                .id(&state)
                .ok_or(DecodeError::Field("sea pickle count"))?;
        }
        Ok(Self {
            water: block_set(bindings, "minecraft:water")?,
            count,
            pickles,
        })
    }
    pub fn place<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        random: &mut impl RandomSource,
        origin: [i32; 3],
    ) -> Result<bool, AquaticError<W::Error>> {
        let count = self.count.sample(random)?;
        let mut placed = 0;
        for _ in 0..count {
            let x = origin[0] + random.next_int_bound(8)? - random.next_int_bound(8)?;
            let z = origin[2] + random.next_int_bound(8)? - random.next_int_bound(8)?;
            let pos = [x, world.height(HeightmapType::OceanFloor, x, z)?, z];
            let state = self.pickles[random.next_int_bound(4)? as usize];
            if self.is_water(world, pos)? && world.block_survives(state, pos)? {
                world
                    .set_block(pos, state, 2)
                    .map_err(AquaticError::World)?;
                placed += 1;
            }
        }
        Ok(placed > 0)
    }
    fn is_water<W: FeatureWorld>(
        &self,
        world: &mut W,
        pos: [i32; 3],
    ) -> Result<bool, AquaticError<W::Error>> {
        Ok(self
            .water
            .contains(&world.block(pos).map_err(AquaticError::World)?))
    }
}
