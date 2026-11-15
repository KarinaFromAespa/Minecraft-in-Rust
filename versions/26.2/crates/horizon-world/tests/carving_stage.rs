use horizon_core::{
    carving::CarvingMask,
    density::{ConstantSampler, SamplerContext},
    position::ChunkPos,
    random::{RandomSource, XoroshiroRandom},
};
use horizon_world::{
    DATA_VERSION,
    heightmap::HeightmapState,
    nbt::Tag,
    registry::{BlockRegistry, BlockStateId},
    worldgen::{
        anchor::WorldGenerationContext,
        aquifer::AquiferError,
        carving::{CarveProfile, CarvingBlocks, apply_carving_mask},
        material_condition::{MaterialRule, MaterialSurfaceQueries},
        material_stage::TopMaterialInputs,
        terrain::{TerrainAquifer, TerrainBlockStates, TerrainChunk},
    },
};
use std::{collections::BTreeSet, sync::Arc};
#[derive(Clone, Copy)]
struct State(u32);
impl HeightmapState for State {
    fn is_air(self) -> bool {
        self.0 < 2
    }
    fn is_air_block(self) -> bool {
        self.0 == 0
    }
    fn has_fluid(self) -> bool {
        self.0 == 3
    }
    fn blocks_motion_in_heightmap(self) -> bool {
        self.0 >= 2 && self.0 != 3
    }
    fn blocks_motion_in_heightmap_no_leaves(self) -> bool {
        self.blocks_motion_in_heightmap()
    }
}
struct Properties;
impl TerrainBlockStates for Properties {
    type State = State;
    fn heightmap_state(&self, id: BlockStateId) -> State {
        State(id.0)
    }
}
struct Surface;
impl MaterialSurfaceQueries for Surface {
    fn surface_depth(&mut self, _: i32, _: i32) -> i32 {
        1
    }
    fn surface_secondary(&mut self, _: i32, _: i32) -> f64 {
        0.
    }
}
struct Aquifer(Vec<i32>);
impl TerrainAquifer for Aquifer {
    fn compute_substance(
        &mut self,
        _: &mut SamplerContext,
        [_x, y, _z]: [i32; 3],
        density: f64,
    ) -> Result<Option<BlockStateId>, AquiferError> {
        assert_eq!(density.to_bits(), 0);
        self.0.push(y);
        match y {
            8 => Ok(None),
            7 => Ok(Some(BlockStateId(0))),
            _ => Err(AquiferError::OutsideGrid),
        }
    }
    fn should_schedule_fluid_update(&self) -> bool {
        false
    }
}
fn registry() -> BlockRegistry {
    let names = [
        "minecraft:air",
        "minecraft:void_air",
        "minecraft:stone",
        "minecraft:water",
        "minecraft:grass_block",
        "minecraft:mycelium",
        "minecraft:dirt",
        "minecraft:lava",
        "minecraft:cave_air",
    ];
    BlockRegistry::from_nbt(&Tag::compound([
        ("DataVersion", Tag::Int(DATA_VERSION)),
        (
            "states",
            Tag::List(
                names
                    .iter()
                    .map(|name| Tag::compound([("id", Tag::String((*name).into()))]))
                    .collect(),
            ),
        ),
        (
            "defaults",
            Tag::Compound(
                names
                    .iter()
                    .enumerate()
                    .map(|(id, name)| ((*name).into(), Tag::Int(id as i32)))
                    .collect(),
            ),
        ),
    ]))
    .unwrap()
}
fn overworld(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}
#[test]
fn filters_and_unreplaceable_blocks_precede_aquifers_and_failures_keep_surface_restoration() {
    let registry = registry();
    for scenario in 0..3 {
        // Scenario 2 leaves grass out of the replaceable set: it is skipped
        // before the aquifer and before the surface is remembered.
        let replaceable = if scenario == 2 {
            overworld(&["minecraft:stone", "minecraft:water", "minecraft:dirt"])
        } else {
            overworld(&[
                "minecraft:stone",
                "minecraft:water",
                "minecraft:grass_block",
                "minecraft:dirt",
            ])
        };
        let blocks =
            CarvingBlocks::from_registry(&registry, &replaceable, &BTreeSet::new()).unwrap();
        let mut chunk =
            TerrainChunk::new(ChunkPos::new(-1, 2), 0, 16, &registry, Properties).unwrap();
        for (y, state) in [(8, 4), (7, 2), (6, 6)] {
            chunk.set_block_raw(0, y, 0, BlockStateId(state)).unwrap();
        }
        let mut mask = CarvingMask::new(0, 15).unwrap();
        for y in 6..=8 {
            mask.carve(0, y, 0).unwrap();
        }
        let mut context = SamplerContext::cached();
        let mut aquifer = Aquifer(Vec::new());
        let mut filtered = Vec::new();
        let result = apply_carving_mask(
            &mut chunk,
            &mask,
            MaterialRule::Block(BlockStateId(3)),
            TopMaterialInputs::<XoroshiroRandom, _> {
                generation: WorldGenerationContext::new(0, 16, 8),
                root: XoroshiroRandom::new(0).fork_positional(),
                surface: Surface,
                preliminary_surface: Arc::new(ConstantSampler(0.)),
                density_context: &mut context,
            },
            &mut aquifer,
            &blocks,
            CarveProfile {
                nether: false,
                lava_y: 0,
            },
            |position| {
                filtered.push(position);
                scenario != 1 || position[1] != 8
            },
        );
        assert!(result.is_err());
        assert_eq!(filtered, [[0, 8, 0], [0, 7, 0], [0, 6, 0]]);
        assert_eq!(
            aquifer.0,
            if scenario == 0 {
                vec![8, 7, 6]
            } else {
                vec![7, 6]
            }
        );
        assert_eq!(chunk.block(0, 8, 0), BlockStateId(4));
        assert_eq!(chunk.block(0, 7, 0), BlockStateId(0));
        assert_eq!(
            chunk.block(0, 6, 0),
            BlockStateId(if scenario == 0 { 3 } else { 6 })
        );
        assert_eq!(
            chunk.postprocessing()[0],
            if scenario == 0 { vec![6 << 4] } else { vec![] }
        );
    }
}
#[test]
fn lava_level_replaces_cells_and_skips_top_material() {
    let registry = registry();
    let replaceable = overworld(&[
        "minecraft:stone",
        "minecraft:water",
        "minecraft:grass_block",
        "minecraft:dirt",
    ]);
    let blocks = CarvingBlocks::from_registry(&registry, &replaceable, &BTreeSet::new()).unwrap();
    let mut chunk = TerrainChunk::new(ChunkPos::new(-1, 2), 0, 16, &registry, Properties).unwrap();
    for (y, state) in [(8, 4), (7, 2), (6, 6)] {
        chunk.set_block_raw(0, y, 0, BlockStateId(state)).unwrap();
    }
    // The dirt below the lava cell is outside the mask: it can only change
    // through the top-material pass, which the lava branch must skip.
    let mut mask = CarvingMask::new(0, 15).unwrap();
    for y in 7..=8 {
        mask.carve(0, y, 0).unwrap();
    }
    let mut context = SamplerContext::cached();
    let mut aquifer = Aquifer(Vec::new());
    let result = apply_carving_mask(
        &mut chunk,
        &mask,
        MaterialRule::Block(BlockStateId(3)),
        TopMaterialInputs::<XoroshiroRandom, _> {
            generation: WorldGenerationContext::new(0, 16, 8),
            root: XoroshiroRandom::new(0).fork_positional(),
            surface: Surface,
            preliminary_surface: Arc::new(ConstantSampler(0.)),
            density_context: &mut context,
        },
        &mut aquifer,
        &blocks,
        CarveProfile {
            nether: false,
            lava_y: 7,
        },
        |_| true,
    );
    assert!(result.is_ok());
    // The lava cell never reaches the aquifer, and the lava branch returns
    // before the top-material pass, so the dirt below stays dirt even though
    // the grass above was carved.
    assert_eq!(aquifer.0, vec![8]);
    assert_eq!(chunk.block(0, 8, 0), BlockStateId(4));
    assert_eq!(chunk.block(0, 7, 0), BlockStateId(7));
    assert_eq!(chunk.block(0, 6, 0), BlockStateId(6));
}
#[test]
fn nether_profile_writes_lava_and_cave_air_without_aquifer() {
    let registry = registry();
    let replaceable = overworld(&["minecraft:stone", "minecraft:dirt"]);
    let blocks = CarvingBlocks::from_registry(&registry, &BTreeSet::new(), &replaceable).unwrap();
    let mut chunk = TerrainChunk::new(ChunkPos::new(-1, 2), 0, 16, &registry, Properties).unwrap();
    for (y, state) in [(8, 2), (7, 2), (6, 6)] {
        chunk.set_block_raw(0, y, 0, BlockStateId(state)).unwrap();
    }
    let mut mask = CarvingMask::new(0, 15).unwrap();
    for y in 6..=8 {
        mask.carve(0, y, 0).unwrap();
    }
    let mut context = SamplerContext::cached();
    let mut aquifer = Aquifer(Vec::new());
    let result = apply_carving_mask(
        &mut chunk,
        &mask,
        MaterialRule::Block(BlockStateId(3)),
        TopMaterialInputs::<XoroshiroRandom, _> {
            generation: WorldGenerationContext::new(0, 16, 8),
            root: XoroshiroRandom::new(0).fork_positional(),
            surface: Surface,
            preliminary_surface: Arc::new(ConstantSampler(0.)),
            density_context: &mut context,
        },
        &mut aquifer,
        &blocks,
        CarveProfile {
            nether: true,
            lava_y: 7,
        },
        |_| true,
    );
    assert!(result.is_ok());
    assert!(aquifer.0.is_empty());
    assert_eq!(chunk.block(0, 8, 0), BlockStateId(8));
    assert_eq!(chunk.block(0, 7, 0), BlockStateId(7));
    assert_eq!(chunk.block(0, 6, 0), BlockStateId(7));
    assert!(chunk.postprocessing()[0].is_empty());
}
