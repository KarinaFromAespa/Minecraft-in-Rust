//! Vanilla 1.21.7 `DesertWellFeature`: a sandstone well sunk into sand, with
//! a water cross, a sand cross, a slabbed rim, four pillars, a slabbed
//! roof, and two suspicious-sand blocks seeded with the desert-well
//! archaeology loot table.
//!
//! The 1.21.7 class is `added` in the 26.3 -> 1.21.7 diff; the Rust had no
//! decode arm for it, so the 1.21.7 `desert_well` configured feature failed
//! to decode.
use super::{
    DecodeError,
    block_feature::FeatureBlockBindings,
    feature::FeatureWorld,
    placement::{PlacementContext, PlacementQueryError},
};
use crate::registry::BlockStateId;
use horizon_core::{
    position::BlockPos,
    random::{InvalidBound, RandomSource},
};
use std::{collections::BTreeSet, sync::Arc};

fn offset(p: [i32; 3], d: [i32; 3]) -> [i32; 3] {
    std::array::from_fn(|i| p[i].wrapping_add(d[i]))
}

/// Vanilla `Direction.Plane.HORIZONTAL`: north, east, south, west.
const HORIZONTAL: [[i32; 3]; 4] = [[0, 0, -1], [1, 0, 0], [0, 0, 1], [-1, 0, 0]];

#[derive(Debug)]
pub enum DesertWellError<E> {
    World(E),
    Query(PlacementQueryError),
    Random(InvalidBound),
}
impl<E: std::fmt::Display> std::fmt::Display for DesertWellError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::World(e) => e.fmt(f),
            Self::Query(e) => e.fmt(f),
            Self::Random(e) => e.fmt(f),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for DesertWellError<E> {}
impl<E> From<InvalidBound> for DesertWellError<E> {
    fn from(error: InvalidBound) -> Self {
        Self::Random(error)
    }
}
impl<E> From<PlacementQueryError> for DesertWellError<E> {
    fn from(error: PlacementQueryError) -> Self {
        Self::Query(error)
    }
}

#[derive(Clone, Debug)]
pub struct DesertWellFeature {
    air: [BlockStateId; 3],
    sand_states: Arc<BTreeSet<BlockStateId>>,
    sand: BlockStateId,
    sand_slab: BlockStateId,
    sandstone: BlockStateId,
    water: BlockStateId,
    sus_sand: BlockStateId,
}
impl DesertWellFeature {
    pub fn new(bindings: &FeatureBlockBindings<'_>) -> Result<Self, DecodeError> {
        let registry = bindings.registry();
        let get = |name| {
            registry
                .default_state(name)
                .ok_or_else(|| DecodeError::Registry(format!("unknown desert well block {name}")))
        };
        Ok(Self {
            air: bindings.air,
            sand_states: bindings
                .blocks
                .get("minecraft:sand")
                .cloned()
                .ok_or(DecodeError::Field("desert well sand identity"))?,
            sand: get("minecraft:sand")?,
            sand_slab: get("minecraft:sandstone_slab")?,
            sandstone: get("minecraft:sandstone")?,
            water: get("minecraft:water")?,
            sus_sand: get("minecraft:suspicious_sand")?,
        })
    }
    pub fn place<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        random: &mut impl RandomSource,
        origin: [i32; 3],
    ) -> Result<bool, DesertWellError<W::Error>> {
        let mut origin = offset(origin, [0, 1, 0]);
        let min_y = world.min_y()?;
        while self.is_empty(world, origin)? && origin[1] > min_y.wrapping_add(2) {
            origin = offset(origin, [0, -1, 0]);
        }
        if !self
            .sand_states
            .contains(&world.block(origin).map_err(DesertWellError::World)?)
        {
            return Ok(false);
        }
        for ox in -2..=2 {
            for oz in -2..=2 {
                if self.is_empty(world, [origin[0] + ox, origin[1] - 1, origin[2] + oz])?
                    && self.is_empty(world, [origin[0] + ox, origin[1] - 2, origin[2] + oz])?
                {
                    return Ok(false);
                }
            }
        }
        for oy in -2..=0 {
            for ox in -2..=2 {
                for oz in -2..=2 {
                    self.set(
                        world,
                        [origin[0] + ox, origin[1] + oy, origin[2] + oz],
                        self.sandstone,
                    )?;
                }
            }
        }
        self.set(world, origin, self.water)?;
        for d in HORIZONTAL {
            self.set(world, offset(origin, d), self.water)?;
        }
        let sand_center = offset(origin, [0, -1, 0]);
        self.set(world, sand_center, self.sand)?;
        for d in HORIZONTAL {
            self.set(world, offset(sand_center, d), self.sand)?;
        }
        for ox in -2..=2 {
            for oz in -2..=2 {
                if ox == -2 || ox == 2 || oz == -2 || oz == 2 {
                    self.set(
                        world,
                        [origin[0] + ox, origin[1] + 1, origin[2] + oz],
                        self.sandstone,
                    )?;
                }
            }
        }
        self.set(
            world,
            [origin[0] + 2, origin[1] + 1, origin[2]],
            self.sand_slab,
        )?;
        self.set(
            world,
            [origin[0] - 2, origin[1] + 1, origin[2]],
            self.sand_slab,
        )?;
        self.set(
            world,
            [origin[0], origin[1] + 1, origin[2] + 2],
            self.sand_slab,
        )?;
        self.set(
            world,
            [origin[0], origin[1] + 1, origin[2] - 2],
            self.sand_slab,
        )?;
        for ox in -1..=1 {
            for oz in -1..=1 {
                let state = if ox == 0 && oz == 0 {
                    self.sandstone
                } else {
                    self.sand_slab
                };
                self.set(
                    world,
                    [origin[0] + ox, origin[1] + 4, origin[2] + oz],
                    state,
                )?;
            }
        }
        for oy in 1..=3 {
            for (dx, dz) in [(-1, -1), (-1, 1), (1, -1), (1, 1)] {
                self.set(
                    world,
                    [origin[0] + dx, origin[1] + oy, origin[2] + dz],
                    self.sandstone,
                )?;
            }
        }
        // Util.getRandom over [centre, east, south, west, north], twice.
        let water_positions = [
            origin,
            offset(origin, [1, 0, 0]),
            offset(origin, [0, 0, 1]),
            offset(origin, [-1, 0, 0]),
            offset(origin, [0, 0, -1]),
        ];
        for depth in [1, 2] {
            let pick = water_positions[random.next_int_bound(5)? as usize];
            let pos = [pick[0], pick[1] - depth, pick[2]];
            world
                .set_block(pos, self.sus_sand, 3)
                .map_err(DesertWellError::World)?;
            // Vanilla seeds the brushable block with `pos.asLong()` and
            // draws nothing from the feature random for it.
            let seed = BlockPos::new(pos[0], pos[1], pos[2]).pack();
            world.set_block_entity_loot_table_seeded(
                pos,
                "minecraft:archaeology/desert_well",
                seed,
            )?;
        }
        Ok(true)
    }
    fn is_empty<W: FeatureWorld>(
        &self,
        world: &mut W,
        pos: [i32; 3],
    ) -> Result<bool, DesertWellError<W::Error>> {
        Ok(self
            .air
            .contains(&world.block(pos).map_err(DesertWellError::World)?))
    }
    fn set<W: FeatureWorld>(
        &self,
        world: &mut W,
        pos: [i32; 3],
        state: BlockStateId,
    ) -> Result<(), DesertWellError<W::Error>> {
        world
            .set_block(pos, state, 2)
            .map(|_| ())
            .map_err(DesertWellError::World)
    }
}
