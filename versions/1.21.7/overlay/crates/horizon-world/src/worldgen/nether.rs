//! 1.21.7 nether features: basalt columns, basalt pillars, glowstone blobs,
//! nether forest vegetation, twisting vines and weeping vines.
//!
//! The 1.21.7 classes are `added` in the 26.3 -> 1.21.7 diff (26.3 replaced
//! them with record features or removed them); the Rust had no decode arm
//! for any of them, so every 1.21.7 configured feature of these types failed
//! to decode.
use super::{
    DecodeError, Object,
    aquatic::next_int_range,
    block_feature::FeatureBlockBindings,
    feature::FeatureWorld,
    int_range,
    placement::{PlacementContext, PlacementQueryError},
    required,
    state_provider::{BlockStateProvider, StateProviderError},
    value_provider::IntProvider,
};
use crate::registry::BlockStateId;
use horizon_core::random::{InvalidBound, RandomSource};
use std::{collections::BTreeSet, sync::Arc};

fn offset(p: [i32; 3], d: [i32; 3]) -> [i32; 3] {
    std::array::from_fn(|i| p[i].wrapping_add(d[i]))
}

/// Vanilla `Direction.values()` order: down, up, north, south, west, east.
const NEIGHBORS: [[i32; 3]; 6] = [
    [0, -1, 0],
    [0, 1, 0],
    [0, 0, -1],
    [0, 0, 1],
    [-1, 0, 0],
    [1, 0, 0],
];

/// Vanilla `BlockPos.distManhattan`, with Java's wrapping int arithmetic.
fn manhattan(a: [i32; 3], b: [i32; 3]) -> i32 {
    (0..3).fold(0i32, |sum, i| {
        sum.wrapping_add(a[i].wrapping_sub(b[i]).wrapping_abs())
    })
}

#[derive(Debug)]
pub enum NetherFeatureError<E> {
    World(E),
    Query(PlacementQueryError),
    Random(InvalidBound),
    Provider(StateProviderError<E>),
}
impl<E: std::fmt::Display> std::fmt::Display for NetherFeatureError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::World(e) => e.fmt(f),
            Self::Query(e) => e.fmt(f),
            Self::Random(e) => e.fmt(f),
            Self::Provider(e) => e.fmt(f),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for NetherFeatureError<E> {}
impl<E> From<InvalidBound> for NetherFeatureError<E> {
    fn from(error: InvalidBound) -> Self {
        Self::Random(error)
    }
}
impl<E> From<PlacementQueryError> for NetherFeatureError<E> {
    fn from(error: PlacementQueryError) -> Self {
        Self::Query(error)
    }
}
impl<E> From<StateProviderError<E>> for NetherFeatureError<E> {
    fn from(error: StateProviderError<E>) -> Self {
        Self::Provider(error)
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
        .ok_or_else(|| DecodeError::Field("nether block identity"))
}

fn union(
    bindings: &FeatureBlockBindings<'_>,
    names: &[&str],
) -> Result<Arc<BTreeSet<BlockStateId>>, DecodeError> {
    let mut set = BTreeSet::new();
    for name in names {
        set.extend(block_set(bindings, name)?.iter().copied());
    }
    Ok(Arc::new(set))
}

fn is_empty<W: FeatureWorld>(
    world: &mut W,
    air: &[BlockStateId; 3],
    pos: [i32; 3],
) -> Result<bool, NetherFeatureError<W::Error>> {
    Ok(air.contains(&world.block(pos).map_err(NetherFeatureError::World)?))
}

fn set<W: FeatureWorld>(
    world: &mut W,
    pos: [i32; 3],
    state: BlockStateId,
) -> Result<(), NetherFeatureError<W::Error>> {
    set_flags(world, pos, state, 2)
}

/// Vanilla `Feature.setBlock` writes with flag 3; the features that call
/// `level.setBlock` directly pass flag 2.
fn set_flags<W: FeatureWorld>(
    world: &mut W,
    pos: [i32; 3],
    state: BlockStateId,
    flags: i32,
) -> Result<(), NetherFeatureError<W::Error>> {
    world
        .set_block(pos, state, flags)
        .map(|_| ())
        .map_err(NetherFeatureError::World)
}

/// Vanilla 1.21.7 `BasaltColumnsFeature`: clustered or scattered basalt
/// columns on a lava-ocean floor. The config `reach` provider is sampled
/// once per qualifying cluster position, not once per feature.
#[derive(Clone, Debug)]
pub struct BasaltColumnsFeature {
    height: IntProvider,
    reach: IntProvider,
    air: [BlockStateId; 3],
    basalt: BlockStateId,
    basalt_states: Arc<BTreeSet<BlockStateId>>,
    lava: Arc<BTreeSet<BlockStateId>>,
    cannot_place_on: Arc<BTreeSet<BlockStateId>>,
}
impl BasaltColumnsFeature {
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
            return Err(DecodeError::Field("basalt columns depth or work"));
        }
        *remaining -= 1;
        let object: Object = serde_json::from_str(json)?;
        let height = IntProvider::decode(required(&object, "height")?, depth + 1, remaining)?;
        let reach = IntProvider::decode(required(&object, "reach")?, depth + 1, remaining)?;
        let registry = bindings.registry();
        let basalt = registry
            .default_state("minecraft:basalt")
            .ok_or_else(|| DecodeError::Registry("unknown basalt block".into()))?;
        Ok(Self {
            height,
            reach,
            air: bindings.air,
            basalt,
            basalt_states: block_set(bindings, "minecraft:basalt")?,
            lava: block_set(bindings, "minecraft:lava")?,
            cannot_place_on: union(
                bindings,
                &[
                    "minecraft:lava",
                    "minecraft:bedrock",
                    "minecraft:magma_block",
                    "minecraft:soul_sand",
                    "minecraft:nether_bricks",
                    "minecraft:nether_brick_fence",
                    "minecraft:nether_brick_stairs",
                    "minecraft:nether_wart",
                    "minecraft:chest",
                    "minecraft:spawner",
                ],
            )?,
        })
    }
    pub fn place<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        random: &mut impl RandomSource,
        origin: [i32; 3],
    ) -> Result<bool, NetherFeatureError<W::Error>> {
        // Vanilla reads the chunk generator's sea level
        // (context.chunkGenerator().getSeaLevel()), not the world's.
        let sea_level = world.generation().sea_level;
        if !self.can_place_at(world, sea_level, origin)? {
            return Ok(false);
        }
        let column_height = self.height.sample(random)?;
        let clustered = random.next_float() < 0.9;
        let spread = column_height.min(if clustered { 5 } else { 8 });
        let count = if clustered { 50 } else { 15 };
        let mut placed = false;
        for _ in 0..count {
            // BlockPos.randomBetweenClosed draws x, y, z per position; the
            // y draw still happens even though min == max == origin y.
            let pos = [
                next_int_range(random, origin[0] - spread, origin[0] + spread)?,
                next_int_range(random, origin[1], origin[1])?,
                next_int_range(random, origin[2] - spread, origin[2] + spread)?,
            ];
            let blocks_to_place = column_height.wrapping_sub(manhattan(pos, origin));
            if blocks_to_place >= 0 {
                let reach = self.reach.sample(random)?;
                placed |= self.place_column(world, sea_level, pos, blocks_to_place, reach)?;
            }
        }
        Ok(placed)
    }
    fn place_column<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        sea_level: i32,
        center: [i32; 3],
        column_height: i32,
        reach: i32,
    ) -> Result<bool, NetherFeatureError<W::Error>> {
        let mut placed = false;
        // i64 bounds: the same square as vanilla's betweenClosed, without
        // panicking on extreme centres.
        let (x0, x1) = (
            center[0] as i64 - reach as i64,
            center[0] as i64 + reach as i64,
        );
        let (z0, z1) = (
            center[2] as i64 - reach as i64,
            center[2] as i64 + reach as i64,
        );
        // BlockPos.betweenClosed walks x fastest, then y, then z: z is the
        // outer loop, x the inner one.
        for z in z0..=z1 {
            for x in x0..=x1 {
                let pos = [x as i32, center[1], z as i32];
                let step_limit = manhattan(pos, center);
                let column_pos = if self.is_air_or_lava_ocean(world, sea_level, pos)? {
                    self.find_surface(world, sea_level, pos, step_limit)?
                } else {
                    self.find_air(world, pos, step_limit)?
                };
                if let Some(column_pos) = column_pos {
                    let mut blocks_y = column_height - step_limit / 2;
                    let mut cursor = column_pos;
                    while blocks_y >= 0 {
                        if self.is_air_or_lava_ocean(world, sea_level, cursor)? {
                            // BasaltColumnsFeature calls Feature.setBlock,
                            // which writes with flag 3.
                            set_flags(world, cursor, self.basalt, 3)?;
                            cursor = offset(cursor, [0, 1, 0]);
                            placed = true;
                        } else if self
                            .basalt_states
                            .contains(&world.block(cursor).map_err(NetherFeatureError::World)?)
                        {
                            cursor = offset(cursor, [0, 1, 0]);
                        } else {
                            break;
                        }
                        blocks_y -= 1;
                    }
                }
            }
        }
        Ok(placed)
    }
    fn find_surface<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        sea_level: i32,
        start: [i32; 3],
        mut limit: i32,
    ) -> Result<Option<[i32; 3]>, NetherFeatureError<W::Error>> {
        let mut cursor = start;
        let min_y = world.min_y()?;
        while cursor[1] > min_y.wrapping_add(1) && limit > 0 {
            limit -= 1;
            if self.can_place_at(world, sea_level, cursor)? {
                return Ok(Some(cursor));
            }
            cursor = offset(cursor, [0, -1, 0]);
        }
        Ok(None)
    }
    fn find_air<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        start: [i32; 3],
        mut limit: i32,
    ) -> Result<Option<[i32; 3]>, NetherFeatureError<W::Error>> {
        let mut cursor = start;
        let max_y = world.max_y()?;
        while cursor[1] <= max_y && limit > 0 {
            limit -= 1;
            let state = world.block(cursor).map_err(NetherFeatureError::World)?;
            if self.cannot_place_on.contains(&state) {
                return Ok(None);
            }
            if self.air.contains(&state) {
                return Ok(Some(cursor));
            }
            cursor = offset(cursor, [0, 1, 0]);
        }
        Ok(None)
    }
    fn can_place_at<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        sea_level: i32,
        pos: [i32; 3],
    ) -> Result<bool, NetherFeatureError<W::Error>> {
        if !self.is_air_or_lava_ocean(world, sea_level, pos)? {
            return Ok(false);
        }
        let below = world
            .block(offset(pos, [0, -1, 0]))
            .map_err(NetherFeatureError::World)?;
        Ok(!self.air.contains(&below) && !self.cannot_place_on.contains(&below))
    }
    fn is_air_or_lava_ocean<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        sea_level: i32,
        pos: [i32; 3],
    ) -> Result<bool, NetherFeatureError<W::Error>> {
        let state = world.block(pos).map_err(NetherFeatureError::World)?;
        Ok(self.air.contains(&state) || (self.lava.contains(&state) && pos[1] <= sea_level))
    }
}

/// Vanilla 1.21.7 `BasaltPillarFeature`: a basalt pillar descending from a
/// ceiling, with random side hang-offs and a spread base.
#[derive(Clone, Debug)]
pub struct BasaltPillarFeature {
    air: [BlockStateId; 3],
    basalt: BlockStateId,
}
impl BasaltPillarFeature {
    pub fn new(bindings: &FeatureBlockBindings<'_>) -> Result<Self, DecodeError> {
        Ok(Self {
            air: bindings.air,
            basalt: bindings
                .registry()
                .default_state("minecraft:basalt")
                .ok_or_else(|| DecodeError::Registry("unknown basalt block".into()))?,
        })
    }
    pub fn place<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        random: &mut impl RandomSource,
        origin: [i32; 3],
    ) -> Result<bool, NetherFeatureError<W::Error>> {
        if !is_empty(world, &self.air, origin)?
            || is_empty(world, &self.air, offset(origin, [0, 1, 0]))?
        {
            return Ok(false);
        }
        let mut pos = origin;
        let mut place_north = true;
        let mut place_south = true;
        let mut place_west = true;
        let mut place_east = true;
        while is_empty(world, &self.air, pos)? {
            if world.outside_build_height(pos[1])? {
                return Ok(true);
            }
            set(world, pos, self.basalt)?;
            // Java's `flag = flag && placeHangOff(...)` short-circuits:
            // once a direction fails, no more draws happen for it.
            place_north =
                place_north && self.place_hang_off(world, random, offset(pos, [0, 0, -1]))?;
            place_south =
                place_south && self.place_hang_off(world, random, offset(pos, [0, 0, 1]))?;
            place_west =
                place_west && self.place_hang_off(world, random, offset(pos, [-1, 0, 0]))?;
            place_east =
                place_east && self.place_hang_off(world, random, offset(pos, [1, 0, 0]))?;
            pos = offset(pos, [0, -1, 0]);
        }
        pos = offset(pos, [0, 1, 0]);
        self.place_base_hang_off(world, random, offset(pos, [0, 0, -1]))?;
        self.place_base_hang_off(world, random, offset(pos, [0, 0, 1]))?;
        self.place_base_hang_off(world, random, offset(pos, [-1, 0, 0]))?;
        self.place_base_hang_off(world, random, offset(pos, [1, 0, 0]))?;
        pos = offset(pos, [0, -1, 0]);
        for dx in -3i32..=3 {
            for dz in -3i32..=3 {
                let probability = dx.abs() * dz.abs();
                if random.next_int_bound(10)? < 10 - probability {
                    let mut base_pos = [pos[0] + dx, pos[1], pos[2] + dz];
                    let mut max_drop = 3;
                    while is_empty(world, &self.air, offset(base_pos, [0, -1, 0]))? {
                        base_pos = offset(base_pos, [0, -1, 0]);
                        max_drop -= 1;
                        if max_drop <= 0 {
                            break;
                        }
                    }
                    if !is_empty(world, &self.air, offset(base_pos, [0, -1, 0]))? {
                        set(world, base_pos, self.basalt)?;
                    }
                }
            }
        }
        Ok(true)
    }
    fn place_hang_off<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        random: &mut impl RandomSource,
        pos: [i32; 3],
    ) -> Result<bool, NetherFeatureError<W::Error>> {
        if random.next_int_bound(10)? != 0 {
            set(world, pos, self.basalt)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn place_base_hang_off<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        random: &mut impl RandomSource,
        pos: [i32; 3],
    ) -> Result<(), NetherFeatureError<W::Error>> {
        if random.next_boolean() {
            set(world, pos, self.basalt)?;
        }
        Ok(())
    }
}

/// Vanilla 1.21.7 `GlowstoneFeature`: a glowstone blob hanging from netherrack,
/// basalt or blackstone, grown by 1500 single-neighbour attempts.
#[derive(Clone, Debug)]
pub struct GlowstoneFeature {
    air: [BlockStateId; 3],
    glowstone: BlockStateId,
    glowstone_states: Arc<BTreeSet<BlockStateId>>,
    ceiling: Arc<BTreeSet<BlockStateId>>,
}
impl GlowstoneFeature {
    pub fn new(bindings: &FeatureBlockBindings<'_>) -> Result<Self, DecodeError> {
        let registry = bindings.registry();
        Ok(Self {
            air: bindings.air,
            glowstone: registry
                .default_state("minecraft:glowstone")
                .ok_or_else(|| DecodeError::Registry("unknown glowstone block".into()))?,
            glowstone_states: block_set(bindings, "minecraft:glowstone")?,
            ceiling: union(
                bindings,
                &[
                    "minecraft:netherrack",
                    "minecraft:basalt",
                    "minecraft:blackstone",
                ],
            )?,
        })
    }
    pub fn place<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        random: &mut impl RandomSource,
        origin: [i32; 3],
    ) -> Result<bool, NetherFeatureError<W::Error>> {
        if !is_empty(world, &self.air, origin)? {
            return Ok(false);
        }
        let above = world
            .block(offset(origin, [0, 1, 0]))
            .map_err(NetherFeatureError::World)?;
        if !self.ceiling.contains(&above) {
            return Ok(false);
        }
        set(world, origin, self.glowstone)?;
        for _ in 0..1500 {
            let place_pos = [
                origin[0] + random.next_int_bound(8)? - random.next_int_bound(8)?,
                origin[1] - random.next_int_bound(12)?,
                origin[2] + random.next_int_bound(8)? - random.next_int_bound(8)?,
            ];
            if is_empty(world, &self.air, place_pos)? {
                let mut neighbours = 0;
                for d in NEIGHBORS {
                    let state = world
                        .block(offset(place_pos, d))
                        .map_err(NetherFeatureError::World)?;
                    if self.glowstone_states.contains(&state) {
                        neighbours += 1;
                    }
                    if neighbours > 1 {
                        break;
                    }
                }
                if neighbours == 1 {
                    set(world, place_pos, self.glowstone)?;
                }
            }
        }
        Ok(true)
    }
}

/// Vanilla 1.21.7 `NetherForestVegetationFeature`: provider-chosen plants
/// scattered over nylium. The provider state is drawn before the
/// empty/survival checks, so its random draws happen for every attempt.
#[derive(Clone, Debug)]
pub struct NetherForestVegetationFeature {
    provider: BlockStateProvider,
    spread_width: i32,
    spread_height: i32,
    air: [BlockStateId; 3],
    nylium: Arc<BTreeSet<BlockStateId>>,
}
impl NetherForestVegetationFeature {
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
            return Err(DecodeError::Field("nether forest vegetation depth or work"));
        }
        *remaining -= 1;
        let object: Object = serde_json::from_str(json)?;
        let (providers, catalog, _) = bindings.simple_blocks.as_ref().ok_or(DecodeError::Field(
            "nether forest vegetation provider bindings",
        ))?;
        let provider = catalog.provider_with_budget(
            required(&object, "state_provider")?.get(),
            providers,
            depth + 1,
            remaining,
        )?;
        Ok(Self {
            provider,
            spread_width: int_range(
                required(&object, "spread_width")?,
                "nether forest vegetation spread width",
                1,
                i32::MAX,
            )?,
            spread_height: int_range(
                required(&object, "spread_height")?,
                "nether forest vegetation spread height",
                1,
                i32::MAX,
            )?,
            air: bindings.air,
            nylium: bindings
                .tags
                .get("minecraft:nylium")
                .cloned()
                .ok_or(DecodeError::Field("nylium tag"))?,
        })
    }
    pub fn place<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        random: &mut impl RandomSource,
        origin: [i32; 3],
    ) -> Result<bool, NetherFeatureError<W::Error>> {
        let below = world
            .block(offset(origin, [0, -1, 0]))
            .map_err(NetherFeatureError::World)?;
        if !self.nylium.contains(&below) {
            return Ok(false);
        }
        let min_y = world.min_y()?;
        let max_y = world.max_y()?;
        let y = origin[1];
        if y >= min_y.wrapping_add(1) && y.wrapping_add(1) <= max_y {
            let mut placed = 0;
            for _ in 0..self.spread_width.wrapping_mul(self.spread_width) {
                let final_pos = [
                    origin[0] + random.next_int_bound(self.spread_width)?
                        - random.next_int_bound(self.spread_width)?,
                    y + random.next_int_bound(self.spread_height)?
                        - random.next_int_bound(self.spread_height)?,
                    origin[2] + random.next_int_bound(self.spread_width)?
                        - random.next_int_bound(self.spread_width)?,
                ];
                let state = self.provider.get_state(world, random, final_pos)?;
                if is_empty(world, &self.air, final_pos)?
                    && final_pos[1] > min_y
                    && world.block_survives(state, final_pos)?
                {
                    set(world, final_pos, state)?;
                    placed += 1;
                }
            }
            Ok(placed > 0)
        } else {
            Ok(false)
        }
    }
}

/// Vanilla 1.21.7 `TwistingVinesFeature`: upward-growing twisting vines on
/// netherrack, warped nylium or warped wart blocks.
#[derive(Clone, Debug)]
pub struct TwistingVinesFeature {
    spread_width: i32,
    spread_height: i32,
    max_height: i32,
    air: [BlockStateId; 3],
    ground: Arc<BTreeSet<BlockStateId>>,
    head: [BlockStateId; 9],
    plant: BlockStateId,
}
impl TwistingVinesFeature {
    pub fn from_json(json: &str, bindings: &FeatureBlockBindings<'_>) -> Result<Self, DecodeError> {
        let object: Object = serde_json::from_str(json)?;
        let registry = bindings.registry();
        let head_default = registry
            .default_state("minecraft:twisting_vines")
            .ok_or_else(|| DecodeError::Registry("unknown twisting vines block".into()))?;
        let mut head = [head_default; 9];
        for (i, id) in head.iter_mut().enumerate() {
            let mut state = registry.state(head_default).unwrap().clone();
            state.properties.insert("age".into(), (17 + i).to_string());
            *id = registry
                .id(&state)
                .ok_or(DecodeError::Field("twisting vines age"))?;
        }
        Ok(Self {
            spread_width: int_range(
                required(&object, "spread_width")?,
                "twisting vines spread width",
                1,
                i32::MAX,
            )?,
            spread_height: int_range(
                required(&object, "spread_height")?,
                "twisting vines spread height",
                1,
                i32::MAX,
            )?,
            max_height: int_range(
                required(&object, "max_height")?,
                "twisting vines max height",
                1,
                i32::MAX,
            )?,
            air: bindings.air,
            ground: union(
                bindings,
                &[
                    "minecraft:netherrack",
                    "minecraft:warped_nylium",
                    "minecraft:warped_wart_block",
                ],
            )?,
            head,
            plant: registry
                .default_state("minecraft:twisting_vines_plant")
                .ok_or_else(|| DecodeError::Registry("unknown twisting vines plant".into()))?,
        })
    }
    pub fn place<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        random: &mut impl RandomSource,
        origin: [i32; 3],
    ) -> Result<bool, NetherFeatureError<W::Error>> {
        if self.is_invalid(world, origin)? {
            return Ok(false);
        }
        for _ in 0..self.spread_width.wrapping_mul(self.spread_width) {
            let mut place_pos = [
                origin[0] + next_int_range(random, -self.spread_width, self.spread_width)?,
                origin[1] + next_int_range(random, -self.spread_height, self.spread_height)?,
                origin[2] + next_int_range(random, -self.spread_width, self.spread_width)?,
            ];
            if self.find_first_air_above_ground(world, &mut place_pos)?
                && !self.is_invalid(world, place_pos)?
            {
                let mut vine_height = next_int_range(random, 1, self.max_height)?;
                if random.next_int_bound(6)? == 0 {
                    vine_height *= 2;
                }
                if random.next_int_bound(5)? == 0 {
                    vine_height = 1;
                }
                self.place_column(world, random, place_pos, vine_height)?;
            }
        }
        Ok(true)
    }
    fn find_first_air_above_ground<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        place_pos: &mut [i32; 3],
    ) -> Result<bool, NetherFeatureError<W::Error>> {
        loop {
            *place_pos = offset(*place_pos, [0, -1, 0]);
            if world.outside_build_height(place_pos[1])? {
                return Ok(false);
            }
            if !is_empty(world, &self.air, *place_pos)? {
                break;
            }
        }
        *place_pos = offset(*place_pos, [0, 1, 0]);
        Ok(true)
    }
    fn place_column<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        random: &mut impl RandomSource,
        mut pos: [i32; 3],
        total_height: i32,
    ) -> Result<(), NetherFeatureError<W::Error>> {
        let mut height = 1;
        while height <= total_height {
            if is_empty(world, &self.air, pos)? {
                if height == total_height || !is_empty(world, &self.air, offset(pos, [0, 1, 0]))? {
                    let age = next_int_range(random, 17, 25)?;
                    set(world, pos, self.head[(age - 17) as usize])?;
                    break;
                }
                set(world, pos, self.plant)?;
            }
            pos = offset(pos, [0, 1, 0]);
            height += 1;
        }
        Ok(())
    }
    fn is_invalid<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        pos: [i32; 3],
    ) -> Result<bool, NetherFeatureError<W::Error>> {
        if !is_empty(world, &self.air, pos)? {
            return Ok(true);
        }
        let below = world
            .block(offset(pos, [0, -1, 0]))
            .map_err(NetherFeatureError::World)?;
        Ok(!self.ground.contains(&below))
    }
}

/// Vanilla 1.21.7 `WeepingVinesFeature`: nether wart blocks and downward
/// hanging weeping vines under a netherrack or wart-block ceiling.
#[derive(Clone, Debug)]
pub struct WeepingVinesFeature {
    air: [BlockStateId; 3],
    wart: BlockStateId,
    roof: Arc<BTreeSet<BlockStateId>>,
    head: [BlockStateId; 9],
    plant: BlockStateId,
}
impl WeepingVinesFeature {
    pub fn new(bindings: &FeatureBlockBindings<'_>) -> Result<Self, DecodeError> {
        let registry = bindings.registry();
        let head_default = registry
            .default_state("minecraft:weeping_vines")
            .ok_or_else(|| DecodeError::Registry("unknown weeping vines block".into()))?;
        let mut head = [head_default; 9];
        for (i, id) in head.iter_mut().enumerate() {
            let mut state = registry.state(head_default).unwrap().clone();
            state.properties.insert("age".into(), (17 + i).to_string());
            *id = registry
                .id(&state)
                .ok_or(DecodeError::Field("weeping vines age"))?;
        }
        Ok(Self {
            air: bindings.air,
            wart: registry
                .default_state("minecraft:nether_wart_block")
                .ok_or_else(|| DecodeError::Registry("unknown nether wart block".into()))?,
            roof: union(
                bindings,
                &["minecraft:netherrack", "minecraft:nether_wart_block"],
            )?,
            head,
            plant: registry
                .default_state("minecraft:weeping_vines_plant")
                .ok_or_else(|| DecodeError::Registry("unknown weeping vines plant".into()))?,
        })
    }
    pub fn place<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        random: &mut impl RandomSource,
        origin: [i32; 3],
    ) -> Result<bool, NetherFeatureError<W::Error>> {
        if !is_empty(world, &self.air, origin)? {
            return Ok(false);
        }
        let above = world
            .block(offset(origin, [0, 1, 0]))
            .map_err(NetherFeatureError::World)?;
        if !self.roof.contains(&above) {
            return Ok(false);
        }
        self.place_roof_wart(world, random, origin)?;
        self.place_roof_vines(world, random, origin)?;
        Ok(true)
    }
    fn place_roof_wart<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        random: &mut impl RandomSource,
        origin: [i32; 3],
    ) -> Result<(), NetherFeatureError<W::Error>> {
        set(world, origin, self.wart)?;
        for _ in 0..200 {
            let place_pos = [
                origin[0] + random.next_int_bound(6)? - random.next_int_bound(6)?,
                origin[1] + random.next_int_bound(2)? - random.next_int_bound(5)?,
                origin[2] + random.next_int_bound(6)? - random.next_int_bound(6)?,
            ];
            if is_empty(world, &self.air, place_pos)? {
                let mut neighbours = 0;
                for d in NEIGHBORS {
                    let state = world
                        .block(offset(place_pos, d))
                        .map_err(NetherFeatureError::World)?;
                    if self.roof.contains(&state) {
                        neighbours += 1;
                    }
                    if neighbours > 1 {
                        break;
                    }
                }
                if neighbours == 1 {
                    set(world, place_pos, self.wart)?;
                }
            }
        }
        Ok(())
    }
    fn place_roof_vines<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        random: &mut impl RandomSource,
        origin: [i32; 3],
    ) -> Result<(), NetherFeatureError<W::Error>> {
        for _ in 0..100 {
            let place_pos = [
                origin[0] + random.next_int_bound(8)? - random.next_int_bound(8)?,
                origin[1] + random.next_int_bound(2)? - random.next_int_bound(7)?,
                origin[2] + random.next_int_bound(8)? - random.next_int_bound(8)?,
            ];
            if is_empty(world, &self.air, place_pos)? {
                let above = world
                    .block(offset(place_pos, [0, 1, 0]))
                    .map_err(NetherFeatureError::World)?;
                if self.roof.contains(&above) {
                    let mut vine_height = next_int_range(random, 1, 8)?;
                    if random.next_int_bound(6)? == 0 {
                        vine_height *= 2;
                    }
                    if random.next_int_bound(5)? == 0 {
                        vine_height = 1;
                    }
                    self.place_column(world, random, place_pos, vine_height)?;
                }
            }
        }
        Ok(())
    }
    fn place_column<W: FeatureWorld + PlacementContext>(
        &self,
        world: &mut W,
        random: &mut impl RandomSource,
        mut pos: [i32; 3],
        total_height: i32,
    ) -> Result<(), NetherFeatureError<W::Error>> {
        let mut height = 0;
        while height <= total_height {
            if is_empty(world, &self.air, pos)? {
                if height == total_height || !is_empty(world, &self.air, offset(pos, [0, -1, 0]))? {
                    let age = next_int_range(random, 17, 25)?;
                    set(world, pos, self.head[(age - 17) as usize])?;
                    break;
                }
                set(world, pos, self.plant)?;
            }
            pos = offset(pos, [0, -1, 0]);
            height += 1;
        }
        Ok(())
    }
}
