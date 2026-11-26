//! `minecraft:ice_spike`: the packed-ice spikes of the ice spikes biome
//! (1.21.11's `IceSpikeFeature`). 26.x generalized the class into the
//! config-driven `minecraft:spike` (state and predicates in the config);
//! 1.21.11 hardcodes packed ice on a snow-block footing.
use super::{
    DecodeError, Object,
    block_feature::FeatureBlockBindings,
    feature::FeatureWorld,
    feature_graph::FeatureExecutionError,
    placement::{PlacementContext, PlacementQueryError},
    tree::{TreeResult, kind, read, write},
    tree_blocks::{self as tb, TreeBlocks},
};
use crate::registry::BlockStateId;
use horizon_core::random::RandomSource;
use std::sync::Arc;

fn query<E>(error: PlacementQueryError) -> FeatureExecutionError<E> {
    FeatureExecutionError::Query(error)
}
/// `Mth.ceil(float)`: the float widens to double exactly, then ceils.
fn ceil(v: f32) -> i32 {
    f64::from(v).ceil() as i32
}

#[derive(Clone, Debug)]
pub struct IceSpikeFeature {
    snow_block: BlockStateId,
    ice: BlockStateId,
    packed_ice: BlockStateId,
    blocks: Arc<TreeBlocks>,
}
impl IceSpikeFeature {
    pub(crate) fn decode(
        json: &str,
        bindings: &FeatureBlockBindings<'_>,
    ) -> Result<Self, DecodeError> {
        let object: Object = serde_json::from_str(json)?;
        if kind(&object)? != "minecraft:ice_spike" {
            return Err(DecodeError::Field("ice spike type"));
        }
        let registry = bindings.registry();
        let get = |name: &str| {
            registry
                .default_state(name)
                .ok_or_else(|| DecodeError::Registry(format!("missing ice spike block {name}")))
        };
        Ok(Self {
            snow_block: get("minecraft:snow_block")?,
            ice: get("minecraft:ice")?,
            packed_ice: get("minecraft:packed_ice")?,
            blocks: bindings.tree_blocks()?,
        })
    }
    fn is(&self, a: BlockStateId, b: BlockStateId) -> bool {
        self.blocks.same_block(a, b)
    }
    /// `Feature.isDirt`: the block is in the dirt tag.
    fn is_dirt<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        pos: [i32; 3],
    ) -> TreeResult<bool, W::Error> {
        world.block_matches_tag(pos, "minecraft:dirt").map_err(query)
    }
    /// The cone's replaceable set: air, dirt, snow block, ice.
    fn replaceable<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        pos: [i32; 3],
    ) -> TreeResult<bool, W::Error> {
        let state = read(world, pos)?;
        if self.blocks.is_air(state)
            || self.is(state, self.snow_block)
            || self.is(state, self.ice)
        {
            return Ok(true);
        }
        self.is_dirt(world, pos)
    }
    /// The skirt's replaceable set: the cone's, plus packed ice.
    fn skirt_replaceable<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        pos: [i32; 3],
    ) -> TreeResult<bool, W::Error> {
        let state = read(world, pos)?;
        if self.blocks.is_air(state)
            || self.is(state, self.snow_block)
            || self.is(state, self.ice)
            || self.is(state, self.packed_ice)
        {
            return Ok(true);
        }
        self.is_dirt(world, pos)
    }
    pub(crate) fn place<W: FeatureWorld + PlacementContext, R: RandomSource>(
        &self,
        world: &mut W,
        random: &mut R,
        origin: [i32; 3],
    ) -> TreeResult<bool, W::Error> {
        let min_y = world.min_y().map_err(query)?;
        let mut pos = origin;
        while self.blocks.is_air(read(world, pos)?) && pos[1] > min_y + 2 {
            pos = tb::offset(pos, 0, -1, 0);
        }
        if !self.is(read(world, pos)?, self.snow_block) {
            return Ok(false);
        }
        pos = tb::offset(pos, 0, random.next_int_bound(4)?, 0);
        let height = random.next_int_bound(4)? + 7;
        let radius = height / 4 + random.next_int_bound(2)?;
        if radius > 1 && random.next_int_bound(60)? == 0 {
            pos = tb::offset(pos, 0, 10 + random.next_int_bound(30)?, 0);
        }
        for layer in 0..height {
            let f = (1.0 - layer as f32 / height as f32) * radius as f32;
            let r = ceil(f);
            for dx in -r..=r {
                let ax = dx.abs() as f32 - 0.25;
                for dz in -r..=r {
                    let az = dz.abs() as f32 - 0.25;
                    let inside = (dx == 0 && dz == 0) || ax * ax + az * az <= f * f;
                    // Rim cells alone draw the random float (short-circuit).
                    let rim = dx == -r || dx == r || dz == -r || dz == r;
                    if inside && (!rim || random.next_float() <= 0.75) {
                        let p = tb::offset(pos, dx, layer, dz);
                        if self.replaceable(world, p)? {
                            write(world, p, self.packed_ice, 3)?;
                        }
                        if layer != 0 && r > 1 {
                            let p = tb::offset(pos, dx, -layer, dz);
                            if self.replaceable(world, p)? {
                                write(world, p, self.packed_ice, 3)?;
                            }
                        }
                    }
                }
            }
        }
        // The base skirt: packed ice down to y = 50 in runs, dropping
        // past the occasional gap.
        let s = (radius - 1).clamp(0, 1);
        for dx in -s..=s {
            for dz in -s..=s {
                let mut p = tb::offset(pos, dx, -1, dz);
                let mut run = if dx.abs() == 1 && dz.abs() == 1 {
                    random.next_int_bound(5)?
                } else {
                    50
                };
                while p[1] > 50 {
                    if !self.skirt_replaceable(world, p)? {
                        break;
                    }
                    write(world, p, self.packed_ice, 3)?;
                    p = tb::offset(p, 0, -1, 0);
                    run -= 1;
                    if run <= 0 {
                        p = tb::offset(p, 0, -(random.next_int_bound(5)? + 1), 0);
                        run = random.next_int_bound(5)?;
                    }
                }
            }
        }
        Ok(true)
    }
}
