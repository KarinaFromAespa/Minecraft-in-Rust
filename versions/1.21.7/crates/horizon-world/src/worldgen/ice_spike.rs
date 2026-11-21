//! Vanilla 1.21.7 `IceSpikeFeature`: a packed-ice spike rising from snow,
//! with a mirrored root below the origin and packed-ice base columns
//! sinking toward y=50.
//!
//! The 1.21.7 class is `added` in the 26.3 -> 1.21.7 diff (26.3 removed
//! it); the Rust had no decode arm for it, so the 1.21.7 `ice_spike`
//! configured feature failed to decode.
use super::{
    DecodeError,
    block_feature::FeatureBlockBindings,
    block_predicate::PredicateBlockResolver,
    feature::FeatureWorld,
    placement::{PlacementContext, PlacementQueryError},
};
use crate::registry::BlockStateId;
use horizon_core::random::{InvalidBound, RandomSource};
use std::{collections::BTreeSet, sync::Arc};
#[derive(Clone, Debug)]
pub struct IceSpikeFeature {
    packed_ice: BlockStateId,
    air: [BlockStateId; 3],
    snow: Arc<BTreeSet<BlockStateId>>,
    replaceable: Arc<BTreeSet<BlockStateId>>,
    base_replaceable: Arc<BTreeSet<BlockStateId>>,
}
#[derive(Debug)]
pub enum IceSpikeError<E> {
    Random(InvalidBound),
    Query(PlacementQueryError),
    World(E),
    WorkLimit,
}
impl<E: std::fmt::Display> std::fmt::Display for IceSpikeError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Random(e) => e.fmt(f),
            Self::Query(e) => e.fmt(f),
            Self::World(e) => e.fmt(f),
            Self::WorkLimit => f.write_str("ice spike work limit exceeded"),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for IceSpikeError<E> {}
impl IceSpikeFeature {
    pub fn new(bindings: &FeatureBlockBindings<'_>) -> Result<Self, DecodeError> {
        let packed_ice = bindings
            .registry()
            .default_state("minecraft:packed_ice")
            .ok_or(DecodeError::Field("packed ice block"))?;
        let snow = bindings.predicate_block_set("\"minecraft:snow_block\"")?;
        let ice = bindings.predicate_block_set("\"minecraft:ice\"")?;
        let dirt = bindings.predicate_block_set("\"#minecraft:dirt\"")?;
        let packed = bindings.predicate_block_set("\"minecraft:packed_ice\"")?;
        let mut replaceable: BTreeSet<BlockStateId> = bindings.air.iter().copied().collect();
        replaceable.extend(dirt.iter().copied());
        replaceable.extend(snow.iter().copied());
        replaceable.extend(ice.iter().copied());
        let mut base_replaceable = replaceable.clone();
        base_replaceable.extend(packed.iter().copied());
        Ok(Self {
            packed_ice,
            air: bindings.air,
            snow,
            replaceable: Arc::new(replaceable),
            base_replaceable: Arc::new(base_replaceable),
        })
    }
    fn try_set<W: FeatureWorld>(
        &self,
        world: &mut W,
        position: [i32; 3],
        replaceable: &BTreeSet<BlockStateId>,
    ) -> Result<(), IceSpikeError<W::Error>> {
        let state = world.block(position).map_err(IceSpikeError::World)?;
        if replaceable.contains(&state) {
            world
                .set_block(position, self.packed_ice, 3)
                .map_err(IceSpikeError::World)?;
        }
        Ok(())
    }
    pub fn place<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        random: &mut impl RandomSource,
        mut origin: [i32; 3],
    ) -> Result<bool, IceSpikeError<W::Error>> {
        let mut remaining = 1_048_576;
        let min_y = world.min_y().map_err(IceSpikeError::Query)?;
        while self
            .air
            .contains(&world.block(origin).map_err(IceSpikeError::World)?)
            && origin[1] > min_y.wrapping_add(2)
        {
            spend(&mut remaining)?;
            origin[1] = origin[1].wrapping_sub(1);
        }
        if !self
            .snow
            .contains(&world.block(origin).map_err(IceSpikeError::World)?)
        {
            return Ok(false);
        }
        origin[1] =
            origin[1].wrapping_add(random.next_int_bound(4).map_err(IceSpikeError::Random)?);
        let height = random.next_int_bound(4).map_err(IceSpikeError::Random)? + 7;
        let radius = height / 4 + random.next_int_bound(2).map_err(IceSpikeError::Random)?;
        if radius > 1 && random.next_int_bound(60).map_err(IceSpikeError::Random)? == 0 {
            origin[1] = origin[1]
                .wrapping_add(10 + random.next_int_bound(30).map_err(IceSpikeError::Random)?);
        }
        for i in 0..height {
            let f = (1.0f32 - i as f32 / height as f32) * radius as f32;
            let r = f.ceil() as i32;
            for x in -r..=r {
                let fx = x.abs() as f32 - 0.25;
                for z in -r..=r {
                    let fz = z.abs() as f32 - 0.25;
                    if (x != 0 || z != 0) && fx * fx + fz * fz > f * f {
                        continue;
                    }
                    if (x == -r || x == r || z == -r || z == r) && random.next_float() > 0.75 {
                        continue;
                    }
                    spend(&mut remaining)?;
                    self.try_set(
                        world,
                        [origin[0] + x, origin[1] + i, origin[2] + z],
                        &self.replaceable,
                    )?;
                    if i != 0 && r > 1 {
                        self.try_set(
                            world,
                            [origin[0] + x, origin[1] - i, origin[2] + z],
                            &self.replaceable,
                        )?;
                    }
                }
            }
        }
        let r2 = (radius - 1).clamp(0, 1);
        for x in -r2..=r2 {
            for z in -r2..=r2 {
                let mut p = [origin[0] + x, origin[1] - 1, origin[2] + z];
                let mut n = 50;
                if x.abs() == 1 && z.abs() == 1 {
                    n = random.next_int_bound(5).map_err(IceSpikeError::Random)?;
                }
                while p[1] > 50 {
                    spend(&mut remaining)?;
                    self.try_set(world, p, &self.base_replaceable)?;
                    p[1] -= 1;
                    n -= 1;
                    if n <= 0 {
                        p[1] -= 1 + random.next_int_bound(5).map_err(IceSpikeError::Random)?;
                        n = random.next_int_bound(5).map_err(IceSpikeError::Random)?;
                    }
                }
            }
        }
        Ok(true)
    }
}
fn spend<E>(remaining: &mut usize) -> Result<(), IceSpikeError<E>> {
    if *remaining == 0 {
        return Err(IceSpikeError::WorkLimit);
    }
    *remaining -= 1;
    Ok(())
}
