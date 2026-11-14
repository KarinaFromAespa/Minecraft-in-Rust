//! Feature-placing growth: `TreeGrower.growTree` (saplings, mangrove
//! propagules, azaleas), huge mushrooms and nether fungi, run on the live level
//! through [`LiveFeatures`].
use super::{Behaviours, LevelAccess, LevelEvent, Sim, feature_world::LevelFeatureWorld, flags};
use crate::{registry::BlockStateId, worldgen::live::LiveFeatures};
use horizon_core::{position::BlockPos, random::RandomSource};
use std::sync::{Arc, OnceLock};

/// Runtime extensions of [`Behaviours`] that depend on the worldgen bundle.
#[derive(Default)]
pub struct Growth {
    features: OnceLock<Arc<LiveFeatures>>,
}

impl Behaviours {
    /// Install the live feature registry used by sapling growth and bone meal.
    /// Without it, saplings report [`LevelEvent::GrowTree`] instead of growing.
    pub fn set_live_features(&self, features: Arc<LiveFeatures>) -> bool {
        self.growth.features.set(features).is_ok()
    }
    pub fn live_features(&self) -> Option<&Arc<LiveFeatures>> {
        self.growth.features.get()
    }
    /// `JigsawPlacement.generateJigsaw`: `assemble` builds the pieces (from
    /// the generation context's own random), then each piece is placed into
    /// the level with the level random, an unbounded box and `reference` as
    /// the origin; `keep_jigsaws` skips the jigsaw replacement. Returns whether pieces were assembled
    /// and the entities the pieces carried (their NBT, for the caller to spawn).
    pub fn place_jigsaw<L: LevelAccess + ?Sized>(
        &self,
        level: &mut L,
        ticks: &mut super::Ticks,
        reference: [i32; 3],
        keep_jigsaws: bool,
        assemble: impl FnOnce() -> Option<crate::worldgen::generator::JigsawPieces>,
    ) -> (bool, Vec<crate::nbt::Compound>) {
        let Some(features) = self.live_features() else {
            return (false, Vec::new());
        };
        let Some(jigsaw) = assemble() else {
            return (false, Vec::new());
        };
        let mut sim = self.sim(level, ticks);
        let mut random = sim.random().clone();
        let entities = {
            let mut world = LevelFeatureWorld {
                sim: &mut sim,
                features,
                entities: Vec::new(),
            };
            let unbounded =
                crate::worldgen::structure::BoundingBox::new([i32::MIN; 3], [i32::MAX; 3]);
            for piece in &jigsaw.pieces {
                let mut place_feature =
                    |w: &mut LevelFeatureWorld<'_, '_, L>,
                     r: &mut horizon_core::random::LegacyRandom,
                     name: &str,
                     p: [i32; 3]| { Ok(features.place_placed(name, w, r, p)) };
                if let Err(e) = piece.place(
                    &mut world,
                    &jigsaw.resources,
                    &mut random,
                    unbounded,
                    reference,
                    keep_jigsaws,
                    &mut place_feature,
                ) {
                    eprintln!("Horizon: placing a jigsaw piece failed: {e}");
                }
            }
            world.entities
        };
        *sim.random() = random;
        (true, entities)
    }
    /// `/place structure`: `PlaceCommand.placeStructure` after the start is
    /// generated, `StructureStart.placeInChunk` for every chunk of the start's
    /// bounding box (`ChunkPos.rangeClosed` of its corners), each clipped to
    /// the chunk's whole height, drawing from the level random. False when the
    /// level has no structure resources, or a piece failed to place. The
    /// entities the pieces carried (villagers, guardians, template entities)
    /// come back as NBT for the caller to spawn.
    pub fn place_structure<L: LevelAccess + ?Sized>(
        &self,
        level: &mut L,
        ticks: &mut super::Ticks,
        start: &crate::worldgen::structure::start::StructureStart,
    ) -> (bool, Vec<crate::nbt::Compound>) {
        let Some(features) = self.live_features() else {
            return (false, Vec::new());
        };
        let (Some(resources), Some(blocks)) = (features.structures(), features.tree_blocks())
        else {
            return (false, Vec::new());
        };
        let (min, max) = (start.bounding_box.min, start.bounding_box.max);
        let (cx0, cx1) = (min[0] >> 4, max[0] >> 4);
        let (cz0, cz1) = (min[2] >> 4, max[2] >> 4);
        let mut sim = self.sim(level, ticks);
        // PlaceCommand's per-chunk clip box: BoundingBox(minX, getMinY(), minZ,
        // maxX, getMaxY() + 1, maxZ), the level's whole height of the chunk
        // (`max_y` is exclusive here, so the top layer is `max_y - 1`).
        let (low_y, high_y) = (sim.level.min_y(), sim.level.max_y() - 1);
        let mut random = sim.random().clone();
        let mut placed = true;
        let entities = {
            let mut world = LevelFeatureWorld {
                sim: &mut sim,
                features,
                entities: Vec::new(),
            };
            let mut place_feature =
                |w: &mut LevelFeatureWorld<'_, '_, L>,
                 r: &mut horizon_core::random::LegacyRandom,
                 name: &str,
                 p: [i32; 3]| { Ok(features.place_placed(name, w, r, p)) };
            for cx in cx0..=cx1 {
                for cz in cz0..=cz1 {
                    let chunk_box = crate::worldgen::structure::BoundingBox::new(
                        [cx * 16, low_y, cz * 16],
                        [cx * 16 + 15, high_y, cz * 16 + 15],
                    );
                    if crate::worldgen::structure::start::place_in_chunk(
                        start,
                        &mut world,
                        resources,
                        &blocks,
                        &mut random,
                        chunk_box,
                        &mut place_feature,
                    )
                    .is_err()
                    {
                        placed = false;
                    }
                }
            }
            world.entities
        };
        *sim.random() = random;
        (placed, entities)
    }
}

/// `TreeGrower` (vanilla's registered growers, by name): 26.2's
/// secondary-chance form — one primary and one secondary tree, mega tree
/// and flower tree each (26.3 replaced them with weighted lists, and added
/// the poplar grower).
pub(crate) struct TreeGrower {
    pub name: &'static str,
    secondary_chance: f32,
    mega_tree: Option<&'static str>,
    secondary_mega_tree: Option<&'static str>,
    tree: Option<&'static str>,
    secondary_tree: Option<&'static str>,
    flowers: Option<&'static str>,
    secondary_flowers: Option<&'static str>,
}

pub(crate) const GROWERS: &[TreeGrower] = &[
    TreeGrower {
        name: "oak",
        secondary_chance: 0.1,
        mega_tree: None,
        secondary_mega_tree: None,
        tree: Some("minecraft:oak"),
        secondary_tree: Some("minecraft:fancy_oak"),
        flowers: Some("minecraft:oak_bees_005"),
        secondary_flowers: Some("minecraft:fancy_oak_bees_005"),
    },
    TreeGrower {
        name: "spruce",
        secondary_chance: 0.5,
        mega_tree: Some("minecraft:mega_spruce"),
        secondary_mega_tree: Some("minecraft:mega_pine"),
        tree: Some("minecraft:spruce"),
        secondary_tree: None,
        flowers: None,
        secondary_flowers: None,
    },
    TreeGrower {
        name: "mangrove",
        secondary_chance: 0.85,
        mega_tree: None,
        secondary_mega_tree: None,
        tree: Some("minecraft:mangrove"),
        secondary_tree: Some("minecraft:tall_mangrove"),
        flowers: None,
        secondary_flowers: None,
    },
    TreeGrower {
        name: "azalea",
        secondary_chance: 0.0,
        mega_tree: None,
        secondary_mega_tree: None,
        tree: Some("minecraft:azalea_tree"),
        secondary_tree: None,
        flowers: None,
        secondary_flowers: None,
    },
    TreeGrower {
        name: "birch",
        secondary_chance: 0.0,
        mega_tree: None,
        secondary_mega_tree: None,
        tree: Some("minecraft:birch"),
        secondary_tree: None,
        flowers: Some("minecraft:birch_bees_005"),
        secondary_flowers: None,
    },
    TreeGrower {
        name: "jungle",
        secondary_chance: 0.0,
        mega_tree: Some("minecraft:mega_jungle_tree"),
        secondary_mega_tree: None,
        tree: Some("minecraft:jungle_tree_no_vine"),
        secondary_tree: None,
        flowers: None,
        secondary_flowers: None,
    },
    TreeGrower {
        name: "acacia",
        secondary_chance: 0.0,
        mega_tree: None,
        secondary_mega_tree: None,
        tree: Some("minecraft:acacia"),
        secondary_tree: None,
        flowers: None,
        secondary_flowers: None,
    },
    TreeGrower {
        name: "cherry",
        secondary_chance: 0.0,
        mega_tree: None,
        secondary_mega_tree: None,
        tree: Some("minecraft:cherry"),
        secondary_tree: None,
        flowers: Some("minecraft:cherry_bees_005"),
        secondary_flowers: None,
    },
    TreeGrower {
        name: "dark_oak",
        secondary_chance: 0.0,
        mega_tree: Some("minecraft:dark_oak"),
        secondary_mega_tree: None,
        tree: None,
        secondary_tree: None,
        flowers: None,
        secondary_flowers: None,
    },
    TreeGrower {
        name: "pale_oak",
        secondary_chance: 0.0,
        mega_tree: Some("minecraft:pale_oak_bonemeal"),
        secondary_mega_tree: None,
        tree: None,
        secondary_tree: None,
        flowers: None,
        secondary_flowers: None,
    },
];

pub(crate) fn grower(name: &str) -> Option<&'static TreeGrower> {
    GROWERS.iter().find(|g| g.name == name)
}

/// `TreeGrower.getConfiguredFeature`: one draw against the secondary chance
/// (always drawn, even at 0); a secondary win falls through to the primary
/// when the secondary tree is absent.
fn configured_feature<R: RandomSource>(
    grower: &TreeGrower,
    random: &mut R,
    has_flowers: bool,
) -> Option<&'static str> {
    if random.next_float() < grower.secondary_chance {
        if has_flowers && let Some(flowers) = grower.secondary_flowers {
            return Some(flowers);
        }
        if let Some(tree) = grower.secondary_tree {
            return Some(tree);
        }
    }
    if has_flowers && let Some(flowers) = grower.flowers {
        return Some(flowers);
    }
    grower.tree
}

/// `TreeGrower.getConfiguredMegaFeature`: the draw happens only when a
/// secondary mega tree exists (the short-circuit is vanilla's).
fn configured_mega_feature<R: RandomSource>(
    grower: &TreeGrower,
    random: &mut R,
) -> Option<&'static str> {
    if grower.secondary_mega_tree.is_some() && random.next_float() < grower.secondary_chance {
        grower.secondary_mega_tree
    } else {
        grower.mega_tree
    }
}

impl<L: LevelAccess + ?Sized> Sim<'_, L> {
    /// `ConfiguredFeature.place(level, generator, level.random, pos)`. The
    /// feature draws from a copy of the level random that replaces it after.
    pub(crate) fn place_configured_feature(&mut self, name: &str, pos: BlockPos) -> bool {
        let b = self.b;
        let Some(features) = b.live_features() else {
            return false;
        };
        let mut random = self.random().clone();
        let placed = {
            let mut world = LevelFeatureWorld {
                sim: self,
                features,
                entities: Vec::new(),
            };
            features.place_configured(name, &mut world, &mut random, [pos.x, pos.y, pos.z])
        };
        *self.random() = random;
        placed
    }
    /// `Feature.place(level, generator, level.random, pos)` for an inline
    /// feature already decoded, with the same level random as a named one.
    pub(crate) fn place_inline_feature(
        &mut self,
        feature: &crate::worldgen::feature_graph::FeatureDefinition,
        pos: BlockPos,
    ) -> bool {
        let b = self.b;
        let Some(features) = b.live_features() else {
            return false;
        };
        let mut random = self.random().clone();
        let placed = {
            let mut world = LevelFeatureWorld {
                sim: self,
                features,
                entities: Vec::new(),
            };
            features.place_definition(
                feature,
                "an inline feature",
                &mut world,
                &mut random,
                [pos.x, pos.y, pos.z],
            )
        };
        *self.random() = random;
        placed
    }
    /// `PlacedFeature.place(level, generator, level.random, pos)`.
    pub(crate) fn place_placed_feature(&mut self, name: &str, pos: BlockPos) -> bool {
        let b = self.b;
        let Some(features) = b.live_features() else {
            return false;
        };
        let mut random = self.random().clone();
        let placed = {
            let mut world = LevelFeatureWorld {
                sim: self,
                features,
                entities: Vec::new(),
            };
            features.place_placed(name, &mut world, &mut random, [pos.x, pos.y, pos.z])
        };
        *self.random() = random;
        placed
    }
    fn feature_exists(&self, name: &str) -> bool {
        self.b
            .live_features()
            .is_some_and(|f| f.configured(name).is_some())
    }

    /// `TreeGrower.hasFlowers`: a flower within 2 blocks horizontally, 1 vertically.
    fn tree_has_flowers(&self, pos: BlockPos) -> bool {
        let b = self.b;
        let Some(flowers) = b.tag("minecraft:flowers") else {
            return false;
        };
        for y in -1..=1 {
            for z in -2..=2 {
                for x in -2..=2 {
                    if b.in_set(self.state(pos.offset(x, y, z)), flowers) {
                        return true;
                    }
                }
            }
        }
        false
    }
    /// `TreeGrower.isTwoByTwoSapling`, over the four offsets `growTree` tries.
    fn two_by_two(&self, state: BlockStateId, pos: BlockPos) -> Option<(i32, i32)> {
        let block = self.b.block(state);
        for dx in [0, -1] {
            for dz in [0, -1] {
                let at = |x: i32, z: i32| pos.offset(dx + x, 0, dz + z);
                if [at(0, 0), at(1, 0), at(0, 1), at(1, 1)]
                    .iter()
                    .all(|&p| self.b.is_block(self.state(p), block))
                {
                    return Some((dx, dz));
                }
            }
        }
        None
    }
    /// `TreeGrower.growTree`.
    pub(crate) fn grow_tree(
        &mut self,
        grower: &TreeGrower,
        pos: BlockPos,
        state: BlockStateId,
    ) -> bool {
        if let Some(mega) = configured_mega_feature(grower, self.random())
            && self.feature_exists(mega)
            && let Some((dx, dz)) = self.two_by_two(state, pos)
        {
            // 26.2 clears the square to air (flag 260) and restores the passed
            // sapling state on failure; 26.3 removes each sapling to its
            // fluid's legacy block (flag 818) and restores the collected
            // per-position states.
            let air = self.b.air();
            let square = [
                pos.offset(dx, 0, dz),
                pos.offset(dx + 1, 0, dz),
                pos.offset(dx, 0, dz + 1),
                pos.offset(dx + 1, 0, dz + 1),
            ];
            for &p in &square {
                self.set_block(p, air, flags::UPDATE_NONE);
            }
            if self.place_configured_feature(mega, pos.offset(dx, 0, dz)) {
                return true;
            }
            for &p in &square {
                self.set_block(p, state, flags::UPDATE_NONE);
            }
            return false;
        }
        let flowers = self.tree_has_flowers(pos);
        let Some(feature) = configured_feature(grower, self.random(), flowers) else {
            return false;
        };
        if !self.feature_exists(feature) {
            return false;
        }
        // 26.2 clears the sapling to its fluid's legacy block with flag 260
        // (26.3's removeSapling uses 818) and, on a successful place, sends
        // the update explicitly when the position is still empty — the Sim's
        // block observer already logs every change, so no extra call is
        // needed here.
        let empty = self.b.legacy_block(self.fluid_at(pos));
        self.set_block(pos, empty, flags::UPDATE_NONE);
        if self.place_configured_feature(feature, pos) {
            return true;
        }
        self.set_block(pos, state, flags::UPDATE_NONE);
        false
    }
    /// `TreeGrower.getMinimumHeight`: the primary tree's trunk base height
    /// (26.3 reads the same key through a separate `shortestTreeType`).
    pub(crate) fn tree_minimum_height(&self, grower: &TreeGrower) -> i32 {
        grower
            .tree
            .and_then(|name| self.b.live_features()?.configured_json(name))
            .filter(|json| json["type"] == "minecraft:tree")
            .and_then(|json| json["trunk_placer"]["base_height"].as_i64())
            .map_or(0, |h| h as i32)
    }
    /// `SaplingBlock.advanceTree` with the grower run inline when features are
    /// installed; otherwise reported as [`LevelEvent::GrowTree`].
    pub(crate) fn advance_tree(&mut self, pos: BlockPos, state: BlockStateId) {
        let b = self.b;
        if b.table.int(state, b.props.stage) == 0 {
            self.set_block(pos, b.table.cycle(state, b.props.stage), flags::UPDATE_NONE);
            return;
        }
        let Some(name) = b.data(state).feature.clone() else {
            return;
        };
        match grower(&name) {
            Some(g) if b.live_features().is_some() => {
                self.grow_tree(g, pos, state);
            }
            _ => self.emit(LevelEvent::GrowTree {
                pos,
                state,
                grower: name,
            }),
        }
    }
    /// `MushroomBlock.growMushroom`.
    pub(crate) fn grow_mushroom(&mut self, pos: BlockPos, state: BlockStateId) -> bool {
        let Some(feature) = self.b.data(state).feature.clone() else {
            return false;
        };
        if !self.feature_exists(&feature) {
            return false;
        }
        self.remove_block(pos, false);
        if self.place_configured_feature(&feature, pos) {
            return true;
        }
        self.set_block_and_update(pos, state);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_core::random::LegacyRandom;

    /// `TreeGrower.getConfiguredFeature`: one draw against the secondary
    /// chance; a win picks the secondary flower tree with flowers, else the
    /// secondary tree; a loss picks the flower tree with flowers, else the
    /// primary. The probe RNG steps the same draws, so the branch — not the
    /// random — is what the test pins.
    #[test]
    fn configured_feature_draws_once_and_picks_by_the_secondary_chance() {
        let oak = grower("oak").unwrap();
        let mut probe = LegacyRandom::new(0);
        let first = probe.next_float();
        let mut random = LegacyRandom::new(0);
        assert_eq!(
            configured_feature(oak, &mut random, false),
            if first < 0.1 {
                Some("minecraft:fancy_oak")
            } else {
                Some("minecraft:oak")
            }
        );
        // Exactly one draw: the next float is the probe's second.
        assert_eq!(random.next_float(), probe.next_float());
        let mut random = LegacyRandom::new(0);
        assert_eq!(
            configured_feature(oak, &mut random, true),
            if first < 0.1 {
                Some("minecraft:fancy_oak_bees_005")
            } else {
                Some("minecraft:oak_bees_005")
            }
        );
    }

    /// A 0 secondary chance still draws (the draw is unconditional) and
    /// never wins: birch always picks its flower tree with flowers, else
    /// the birch itself.
    #[test]
    fn a_zero_secondary_chance_still_draws_and_never_wins() {
        let birch = grower("birch").unwrap();
        let mut probe = LegacyRandom::new(7);
        let _ = probe.next_float();
        let mut random = LegacyRandom::new(7);
        assert_eq!(
            configured_feature(birch, &mut random, true),
            Some("minecraft:birch_bees_005")
        );
        assert_eq!(random.next_float(), probe.next_float());
        let mut random = LegacyRandom::new(7);
        assert_eq!(
            configured_feature(birch, &mut random, false),
            Some("minecraft:birch")
        );
    }

    /// `TreeGrower.getConfiguredMegaFeature`: the draw happens only when a
    /// secondary mega tree exists — spruce draws once, jungle not at all.
    #[test]
    fn configured_mega_feature_draws_only_with_a_secondary_mega_tree() {
        let spruce = grower("spruce").unwrap();
        let mut probe = LegacyRandom::new(3);
        let first = probe.next_float();
        let mut random = LegacyRandom::new(3);
        assert_eq!(
            configured_mega_feature(spruce, &mut random),
            if first < 0.5 {
                Some("minecraft:mega_pine")
            } else {
                Some("minecraft:mega_spruce")
            }
        );
        assert_eq!(random.next_float(), probe.next_float());
        let jungle = grower("jungle").unwrap();
        let mut probe = LegacyRandom::new(3);
        let mut random = LegacyRandom::new(3);
        assert_eq!(
            configured_mega_feature(jungle, &mut random),
            Some("minecraft:mega_jungle_tree")
        );
        // No draw: the next float is still the probe's first.
        assert_eq!(random.next_float(), probe.next_float());
    }
}
