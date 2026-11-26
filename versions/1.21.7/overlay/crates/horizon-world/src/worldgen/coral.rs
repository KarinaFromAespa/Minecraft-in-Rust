//! Vanilla 1.21.7 coral features: `coral_tree`, `coral_claw` and
//! `coral_mushroom` (CoralTreeFeature, CoralClawFeature and
//! CoralMushroomFeature over the shared CoralFeature base).
//!
//! 26.3 replaced these with a single coral feature that takes a nested
//! placed feature; 1.21.7's configs are empty and the coral block is
//! picked at random from the `minecraft:coral_blocks` tag at placement,
//! with the coral, sea pickle and wall-coral decorations placed inline
//! by `CoralFeature.placeCoralBlock`.
use super::{
    DecodeError, Object,
    block_feature::FeatureBlockBindings,
    feature::FeatureWorld,
    placement::PlacementContext,
    tree::{TreeResult, kind, read, write},
    tree_blocks::{self as tb, TreeBlocks},
};
use crate::registry::BlockStateId;
use horizon_core::{geometry::Direction, random::RandomSource};
use std::{collections::BTreeSet, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CoralKind {
    Tree,
    Claw,
    Mushroom,
}

#[derive(Clone, Debug)]
pub struct LegacyCoralFeature {
    kind: CoralKind,
    coral_blocks: Arc<[BlockStateId]>,
    corals: Arc<[BlockStateId]>,
    wall_corals: Arc<[BlockStateId]>,
    coral_states: Arc<BTreeSet<BlockStateId>>,
    water: BlockStateId,
    sea_pickle: BlockStateId,
    blocks: Arc<TreeBlocks>,
}
impl LegacyCoralFeature {
    pub(crate) fn decode(
        json: &str,
        bindings: &FeatureBlockBindings<'_>,
    ) -> Result<Self, DecodeError> {
        let object: Object = serde_json::from_str(json)?;
        let kind = match kind(&object)?.as_str() {
            "minecraft:coral_tree" => CoralKind::Tree,
            "minecraft:coral_claw" => CoralKind::Claw,
            "minecraft:coral_mushroom" => CoralKind::Mushroom,
            _ => return Err(DecodeError::Field("coral feature type")),
        };
        let registry = bindings.registry();
        let tag_states = |tag: &str| {
            let members = bindings
                .tags
                .members(tag)
                .ok_or_else(|| DecodeError::Registry(format!("unknown block tag {tag}")))?;
            if members.is_empty() {
                return Err(DecodeError::Registry(format!("empty block tag {tag}")));
            }
            members
                .iter()
                .map(|name| {
                    registry
                        .default_state(name)
                        .ok_or_else(|| DecodeError::Registry(format!("unknown block {name}")))
                })
                .collect::<Result<Arc<[_]>, _>>()
        };
        Ok(Self {
            kind,
            coral_blocks: tag_states("minecraft:coral_blocks")?,
            corals: tag_states("minecraft:corals")?,
            wall_corals: tag_states("minecraft:wall_corals")?,
            coral_states: bindings
                .tags
                .get("minecraft:corals")
                .ok_or_else(|| DecodeError::Registry("unknown block tag minecraft:corals".into()))?
                .clone(),
            water: registry
                .default_state("minecraft:water")
                .ok_or(DecodeError::Field("water block"))?,
            sea_pickle: registry
                .default_state("minecraft:sea_pickle")
                .ok_or(DecodeError::Field("sea pickle block"))?,
            blocks: bindings.tree_blocks()?,
        })
    }
    fn pick(
        &self,
        states: &[BlockStateId],
        random: &mut impl RandomSource,
    ) -> Result<BlockStateId, horizon_core::random::InvalidBound> {
        // Registry.getRandomElementOf: one bounded draw over the tag's
        // members in tag order.
        Ok(states[random.next_int_bound(states.len() as i32)? as usize])
    }
    fn is_water(&self, state: BlockStateId) -> bool {
        self.blocks.same_block(state, self.water)
    }
    /// `CoralFeature.placeCoralBlock`: set the coral block where the
    /// current block is water or coral and the block above is water,
    /// then the coral/sea-pickle/wall-coral decorations.
    fn place_coral_block<W: FeatureWorld + PlacementContext, R: RandomSource>(
        &self,
        world: &mut W,
        random: &mut R,
        pos: [i32; 3],
        state: BlockStateId,
    ) -> TreeResult<bool, W::Error> {
        let current = read(world, pos)?;
        if !self.is_water(current) && !self.coral_states.contains(&current) {
            return Ok(false);
        }
        let above = tb::offset(pos, 0, 1, 0);
        if !self.is_water(read(world, above)?) {
            return Ok(false);
        }
        write(world, pos, state, 3)?;
        if random.next_float() < 0.25 {
            let coral = self.pick(&self.corals, random)?;
            write(world, above, coral, 2)?;
        } else if random.next_float() < 0.05 {
            let count = random.next_int_bound(4)? + 1;
            let pickle = self
                .blocks
                .with(self.sea_pickle, "pickles", &count.to_string())
                .unwrap_or(self.sea_pickle);
            write(world, above, pickle, 2)?;
        }
        for direction in tb::HORIZONTAL {
            if random.next_float() < 0.2 {
                let neighbor = tb::relative(pos, direction, 1);
                if self.is_water(read(world, neighbor)?) {
                    let wall = self.pick(&self.wall_corals, random)?;
                    let wall = self
                        .blocks
                        .with(wall, "facing", tb::direction_name(direction))
                        .unwrap_or(wall);
                    write(world, neighbor, wall, 2)?;
                }
            }
        }
        Ok(true)
    }
    pub(crate) fn place<W: FeatureWorld + PlacementContext, R: RandomSource>(
        &self,
        world: &mut W,
        random: &mut R,
        origin: [i32; 3],
    ) -> TreeResult<bool, W::Error> {
        let state = self.pick(&self.coral_blocks, random)?;
        match self.kind {
            CoralKind::Tree => self.place_tree(world, random, origin, state),
            CoralKind::Claw => self.place_claw(world, random, origin, state),
            CoralKind::Mushroom => self.place_mushroom(world, random, origin, state),
        }
    }
    fn place_tree<W: FeatureWorld + PlacementContext, R: RandomSource>(
        &self,
        world: &mut W,
        random: &mut R,
        origin: [i32; 3],
        state: BlockStateId,
    ) -> TreeResult<bool, W::Error> {
        let mut pos = origin;
        let trunk = random.next_int_bound(3)? + 1;
        for _ in 0..trunk {
            if !self.place_coral_block(world, random, pos, state)? {
                return Ok(true);
            }
            pos = tb::offset(pos, 0, 1, 0);
        }
        let top = pos;
        let branches = random.next_int_bound(3)? + 2;
        let mut directions = tb::HORIZONTAL;
        tb::shuffle(&mut directions, random)?;
        for &direction in directions.iter().take(branches as usize) {
            let mut pos = tb::relative(top, direction, 1);
            let height = random.next_int_bound(5)? + 2;
            let mut segment = 0;
            let mut j = 0;
            while j < height && self.place_coral_block(world, random, pos, state)? {
                segment += 1;
                pos = tb::offset(pos, 0, 1, 0);
                if j == 0 || (segment >= 2 && random.next_float() < 0.25) {
                    pos = tb::relative(pos, direction, 1);
                    segment = 0;
                }
                j += 1;
            }
        }
        Ok(true)
    }
    fn place_claw<W: FeatureWorld + PlacementContext, R: RandomSource>(
        &self,
        world: &mut W,
        random: &mut R,
        origin: [i32; 3],
        state: BlockStateId,
    ) -> TreeResult<bool, W::Error> {
        if !self.place_coral_block(world, random, origin, state)? {
            return Ok(false);
        }
        let claw = tb::HORIZONTAL[random.next_int_bound(4)? as usize];
        let branches = random.next_int_bound(2)? + 2;
        let mut possible = [
            claw,
            tb::clockwise(claw),
            super::vegetation::counter_clockwise(claw),
        ];
        tb::shuffle(&mut possible, random)?;
        for &branch in possible.iter().take(branches as usize) {
            let mut pos = origin;
            let sideways = random.next_int_bound(2)? + 1;
            pos = tb::relative(pos, branch, 1);
            let (segment, inway) = if branch == claw {
                (claw, random.next_int_bound(3)? + 2)
            } else {
                pos = tb::offset(pos, 0, 1, 0);
                let options = [branch, Direction::Up];
                let segment = options[random.next_int_bound(2)? as usize];
                (segment, random.next_int_bound(3)? + 3)
            };
            let mut i = 0;
            while i < sideways && self.place_coral_block(world, random, pos, state)? {
                pos = tb::relative(pos, segment, 1);
                i += 1;
            }
            pos = tb::relative(pos, segment.opposite(), 1);
            pos = tb::offset(pos, 0, 1, 0);
            for _ in 0..inway {
                pos = tb::relative(pos, claw, 1);
                if !self.place_coral_block(world, random, pos, state)? {
                    break;
                }
                if random.next_float() < 0.25 {
                    pos = tb::offset(pos, 0, 1, 0);
                }
            }
        }
        Ok(true)
    }
    fn place_mushroom<W: FeatureWorld + PlacementContext, R: RandomSource>(
        &self,
        world: &mut W,
        random: &mut R,
        origin: [i32; 3],
        state: BlockStateId,
    ) -> TreeResult<bool, W::Error> {
        let y_size = random.next_int_bound(3)? + 3;
        let x_size = random.next_int_bound(3)? + 3;
        let z_size = random.next_int_bound(3)? + 3;
        let y_down = random.next_int_bound(3)? + 1;
        for x in 0..=x_size {
            for y in 0..=y_size {
                for z in 0..=z_size {
                    let at_edge = |v: i32, size: i32| v == 0 || v == size;
                    let edges = at_edge(x, x_size) as u8
                        + at_edge(y, y_size) as u8
                        + at_edge(z, z_size) as u8;
                    // Face cells only: exactly one coordinate at a bound
                    // (edges and the interior are skipped).
                    if edges != 1 {
                        continue;
                    }
                    if random.next_float() < 0.1 {
                        continue;
                    }
                    let pos = tb::offset(origin, x, y - y_down, z);
                    self.place_coral_block(world, random, pos, state)?;
                }
            }
        }
        Ok(true)
    }
}
