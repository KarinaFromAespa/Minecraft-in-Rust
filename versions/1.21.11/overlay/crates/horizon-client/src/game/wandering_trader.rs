//! `WanderingTraderSpawner`: every 1200 ticks the spawn delay counts down;
//! when it runs out a trader may appear near the player (or the nearest
//! bell), bringing two llamas on leads, and wanders off after two days.
use super::{Dim, Game, creatures::Creatures};
use horizon_client::{
    mobs::{Species, leash::Holder, spawning, trader},
    save::read_dat,
};
use horizon_core::{geometry::Vec3, random::RandomSource};
use horizon_world::nbt::Tag;

const TICK_DELAY: i32 = 1200;
const SPAWN_DELAY: i32 = 24000;
const MIN_CHANCE: i32 = 25;
const MAX_CHANCE: i32 = 75;
const ATTEMPTS: i32 = 10;

/// The spawner's clock, kept in level.dat's `WanderingTraderSpawnDelay`,
/// `WanderingTraderSpawnChance` and `WanderingTraderId` (1.21.11 keeps the
/// state there; the 26.x `WanderingTraderData` file is only a read fallback).
pub(super) struct Spawner {
    tick_delay: i32,
    spawn_delay: i32,
    chance: i32,
    trader_id: Option<[i32; 4]>,
    loaded: bool,
    dirty: bool,
}
impl Default for Spawner {
    fn default() -> Self {
        Self {
            tick_delay: TICK_DELAY,
            spawn_delay: SPAWN_DELAY,
            chance: MIN_CHANCE,
            trader_id: None,
            loaded: false,
            dirty: false,
        }
    }
}
impl Creatures {
    /// A fresh spawner for a newly opened world.
    pub(super) fn reset_trader(&mut self) {
        self.trader = Spawner::default();
    }
}
impl Game<'_> {
    /// The 26.x trader file (`data/minecraft/wandering_trader.dat`):
    /// 1.21.11 keeps the clock in level.dat, so this is only a read
    /// fallback for worlds this tree already saved in the 26.x form.
    fn legacy_trader_path(&self) -> std::path::PathBuf {
        self.world_root
            .join("data")
            .join("minecraft")
            .join("wandering_trader.dat")
    }
    fn load_trader_data(&mut self) {
        if self.creatures.trader.loaded {
            return;
        }
        let level = read_dat(&self.world_root.join("level.dat"));
        let data = level.as_ref().and_then(|t| t.get("Data"));
        let int = |d: Option<&Tag>, k: &str| match d.and_then(|d| d.get(k)) {
            Some(Tag::Int(v)) => Some(*v),
            _ => None,
        };
        let (delay, chance, id) = match (int(data, "WanderingTraderSpawnDelay"), int(data, "WanderingTraderSpawnChance")) {
            (Some(delay), Some(chance)) => (
                delay,
                chance,
                match data.and_then(|d| d.get("WanderingTraderId")) {
                    Some(Tag::IntArray(v)) if v.len() == 4 => {
                        Some([v[0], v[1], v[2], v[3]])
                    }
                    _ => None,
                },
            ),
            _ => {
                let tag = read_dat(&self.legacy_trader_path());
                let legacy = tag.as_ref().and_then(|t| t.get("data"));
                (
                    int(legacy, "spawn_delay").unwrap_or(SPAWN_DELAY),
                    int(legacy, "spawn_chance").unwrap_or(MIN_CHANCE),
                    None,
                )
            }
        };
        let t = &mut self.creatures.trader;
        // `WanderingTraderSpawner`'s constructor: a world with no clock yet
        // (both values zero) starts at the defaults.
        if delay == 0 && chance == 0 {
            t.spawn_delay = SPAWN_DELAY;
            t.chance = MIN_CHANCE;
            t.dirty = true;
        } else {
            t.spawn_delay = delay;
            t.chance = chance;
        }
        t.trader_id = id;
        t.loaded = true;
    }
    /// `PrimaryLevelData.createTag`: the clock goes back into level.dat's
    /// `Data` with the level write, so this hands the values to the level
    /// instead of writing a file of its own.
    pub(super) fn save_wandering_trader(&mut self) {
        if !self.creatures.trader.loaded || !self.creatures.trader.dirty {
            return;
        }
        let t = &self.creatures.trader;
        self.level.wandering_trader = Some((t.spawn_delay, t.chance, t.trader_id));
        self.creatures.trader.dirty = false;
    }
    /// `WanderingTraderSpawner.tick`.
    pub(super) fn tick_wandering_trader(&mut self) {
        if self.dim != Dim::Overworld || !self.ui.rules.bool("spawn_wandering_traders") {
            return;
        }
        self.load_trader_data();
        self.creatures.trader.tick_delay -= 1;
        if self.creatures.trader.tick_delay > 0 {
            return;
        }
        self.creatures.trader.tick_delay = TICK_DELAY;
        let t = &mut self.creatures.trader;
        t.spawn_delay -= TICK_DELAY;
        t.dirty = true;
        if t.spawn_delay > 0 {
            return;
        }
        t.spawn_delay = SPAWN_DELAY;
        let chance = t.chance;
        t.chance = (chance + 25).clamp(MIN_CHANCE, MAX_CHANCE);
        if self.world.random.next_int_bound(100).unwrap_or(100) <= chance && self.spawn_trader() {
            self.creatures.trader.chance = MIN_CHANCE;
        }
    }
    /// `WanderingTraderSpawner.spawn`.
    fn spawn_trader(&mut self) -> bool {
        // ServerLevel.getRandomPlayer: one of the living players, drawn
        // from the level random.
        let alive: Vec<Vec3> = self
            .level_players()
            .into_iter()
            .filter(|p| p.alive)
            .map(|p| p.position)
            .collect();
        if alive.is_empty() {
            return true;
        }
        let pick = self
            .world
            .random
            .next_int_bound(alive.len() as i32)
            .unwrap_or(0);
        let p = alive[pick as usize];
        if self.world.random.next_int_bound(10).unwrap_or(1) != 0 {
            return false;
        }
        let player_pos = [p.x.floor() as i32, p.y.floor() as i32, p.z.floor() as i32];
        let reference = self.nearest_bell(player_pos, 48).unwrap_or(player_pos);
        let Some(pos) = self.find_trader_spawn_near(reference, 48) else {
            return false;
        };
        if !self.trader_has_space(pos) {
            return false;
        }
        let biome = self.world.chunks.get(&(pos[0] >> 4, pos[2] >> 4)).map(|c| {
            self.res
                .biomes()
                .get(c.biome(pos[0], pos[1], pos[2]))
                .name
                .clone()
        });
        if biome.is_some_and(|b| {
            self.res
                .mob_data()
                .biome_has("minecraft:without_wandering_trader_spawns", &b)
        }) {
            return false;
        }
        let at = [
            f64::from(pos[0]) + 0.5,
            f64::from(pos[1]),
            f64::from(pos[2]) + 0.5,
        ];
        let Some(id) = self.spawn_mob("minecraft:wandering_trader", at, true) else {
            return false;
        };
        for _ in 0..2 {
            self.spawn_trader_llama(id, 4);
        }
        // `ServerLevelData.setWanderingTraderId`: the trader's id goes
        // into level.dat with the next save.
        if let Some(uuid) = self.persist.uuid_of(id) {
            self.creatures.trader.trader_id = Some(uuid);
            self.creatures.trader.dirty = true;
        }
        if let Some(m) = self.mobs.iter_mut().find(|m| m.id == id) {
            trader::start_visit(m, reference);
        }
        true
    }
    /// `tryToSpawnLlamaFor`: a trader llama near the trader, on its lead.
    fn spawn_trader_llama(&mut self, trader: u32, radius: i32) {
        let Some(at) = self.mobs.iter().find(|m| m.id == trader).map(|m| {
            [
                m.position.x.floor() as i32,
                m.position.y.floor() as i32,
                m.position.z.floor() as i32,
            ]
        }) else {
            return;
        };
        let Some(pos) = self.find_trader_spawn_near(at, radius) else {
            return;
        };
        let spot = [
            f64::from(pos[0]) + 0.5,
            f64::from(pos[1]),
            f64::from(pos[2]) + 0.5,
        ];
        if let Some(id) = self.spawn_mob("minecraft:trader_llama", spot, false)
            && let Some(l) = self.mobs.iter_mut().find(|m| m.id == id)
        {
            // EntitySpawnReason.EVENT: grown up.
            l.extra.age = 0;
            l.extra.leash = Some(Holder::Mob(trader));
        }
    }
    /// `findSpawnPositionNear`: ten tries at a column near the reference
    /// where the trader could stand. The height is the world surface
    /// (`Heightmap$Types.WORLD_SURFACE`); 26.x reads the trader's
    /// registered heightmap type instead (`MOTION_BLOCKING_NO_LEAVES`).
    fn find_trader_spawn_near(&mut self, reference: [i32; 3], radius: i32) -> Option<[i32; 3]> {
        for _ in 0..ATTEMPTS {
            let x =
                reference[0] + self.world.random.next_int_bound(radius * 2).unwrap_or(0) - radius;
            let z =
                reference[2] + self.world.random.next_int_bound(radius * 2).unwrap_or(0) - radius;
            if !self.world.loaded(x, z) {
                continue;
            }
            let y = self.world_surface(x, z);
            let w = self.mob_world_base(&[]);
            if spawning::placement_ok(Species::WanderingTrader, &w, x, y, z) {
                return Some([x, y, z]);
            }
        }
        None
    }
    /// `hasEnoughSpace`: no collision in the 2 by 3 by 2 box.
    fn trader_has_space(&self, pos: [i32; 3]) -> bool {
        (0..=1).all(|dx| {
            (0..=2).all(|dy| {
                (0..=1).all(|dz| {
                    !self
                        .res
                        .facts()
                        .get(self.world.block(pos[0] + dx, pos[1] + dy, pos[2] + dz))
                        .full_collision
                })
            })
        })
    }
    /// The closest bell (`PoiTypes.MEETING`) within `range` blocks.
    fn nearest_bell(&self, from: [i32; 3], range: i32) -> Option<[i32; 3]> {
        let mut best: Option<([i32; 3], i32)> = None;
        for x in from[0] - range..=from[0] + range {
            for z in from[2] - range..=from[2] + range {
                if !self.world.loaded(x, z) {
                    continue;
                }
                for y in (from[1] - range).max(self.world.bottom_y())..=from[1] + range {
                    if &*self.res.facts().get(self.world.block(x, y, z)).name != "minecraft:bell" {
                        continue;
                    }
                    let d = (x - from[0]).pow(2) + (y - from[1]).pow(2) + (z - from[2]).pow(2);
                    if best.is_none_or(|b| d < b.1) {
                        best = Some(([x, y, z], d));
                    }
                }
            }
        }
        best.map(|b| b.0)
    }
}
