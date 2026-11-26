//! `PatrolSpawner`: every ten to eleven minutes of daylight, one time in five,
//! a pillager patrol gathers 24 to 47 blocks from a random player who is not
//! near a village. The level is read through [`PatrolLevel`] and the pillagers
//! are made by the caller, so the game and the differential test
//! (`tools/test_raid_parity.py`) run the same decisions and random draws.
use crate::mobs::{
    Species,
    spawn_rules::{self, Reason, SpawnLevel},
};
use horizon_core::random::RandomSource;

/// What the patrol spawner reads from the level.
pub trait PatrolLevel: SpawnLevel {
    /// The `spawn_patrols` game rule.
    fn spawn_patrols(&self) -> bool;
    /// `Level.getDayTime`: patrols only start on day 5 (`getDayTime() /
    /// 24000 >= 5`).
    fn day_time(&self) -> i64;
    /// `Level.isBrightOutside`: no fixed time and a sky darkening below 4.
    fn bright_outside(&self) -> bool;
    /// The level's players, in `level.players()` order: block position and
    /// whether the player is a spectator.
    fn patrol_players(&self) -> Vec<([i32; 3], bool)>;
    /// `ServerLevel.sectionsToVillage` (`PoiManager`): 0 to 7.
    fn sections_to_village(&self, pos: [i32; 3]) -> i32;
    /// The chunk at chunk coordinates is loaded (`hasChunk`).
    fn has_chunk(&self, cx: i32, cz: i32) -> bool;
    /// Whether a patrol may spawn at `pos`: the biome there is not in
    /// `#minecraft:without_patrol_spawns` (mushroom fields in 1.21.10).
    fn can_pillager_patrol_spawn(&self, pos: [i32; 3]) -> bool;
    /// `getCurrentDifficultyAt(pos).getEffectiveDifficulty()`.
    fn effective_difficulty(&self, pos: [i32; 3]) -> f32;
    /// `Heightmap.Types.MOTION_BLOCKING_NO_LEAVES` height of a column.
    fn motion_blocking_no_leaves(&self, x: i32, z: i32) -> i32;
}

/// `ServerLevel.isCloseToVillage`.
pub fn close_to_village<L: PatrolLevel + ?Sized>(l: &L, pos: [i32; 3], sections: i32) -> bool {
    sections <= 6 && l.sections_to_village(pos) <= sections
}

/// `Level.hasChunksAt(x0, z0, x1, z1)`.
pub fn has_chunks_at<L: PatrolLevel + ?Sized>(l: &L, x0: i32, z0: i32, x1: i32, z1: i32) -> bool {
    (x0 >> 4..=x1 >> 4).all(|cx| (z0 >> 4..=z1 >> 4).all(|cz| l.has_chunk(cx, cz)))
}

/// A pillager the spawner asks for: where, and whether it leads the patrol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Member {
    pub pos: [i32; 3],
    pub leader: bool,
}

/// `PatrolSpawner` (its countdown lives in memory: a fresh level tries at once).
#[derive(Clone, Debug, Default)]
pub struct PatrolSpawner {
    pub next_tick: i32,
}

impl PatrolSpawner {
    /// `PatrolSpawner.tick`. `spawn_enemies` is the chunk source's flag
    /// (`spawn_mobs` and `spawn_monsters`); `random` is the level random.
    /// `spawn` creates, finalizes and adds a pillager at a position that
    /// passed the spawn checks (`EntityType.create` succeeding), and says
    /// whether it did.
    pub fn tick<L: PatrolLevel, R: RandomSource>(
        &mut self,
        level: &L,
        spawn_enemies: bool,
        random: &mut R,
        spawn: &mut dyn FnMut(&L, Member, &mut R) -> bool,
    ) {
        if !spawn_enemies || !level.spawn_patrols() {
            return;
        }
        self.next_tick -= 1;
        if self.next_tick > 0 {
            return;
        }
        self.next_tick += 12000 + random.next_int_bound(1200).unwrap_or(0);
        // PatrolSpawner.tick: no patrols before day 5, and only in daylight
        // (1.21.11 dropped the day gate for the early-game timeline).
        if level.day_time() / 24000 < 5 {
            return;
        }
        if !level.bright_outside() || random.next_int_bound(5).unwrap_or(1) != 0 {
            return;
        }
        let players = level.patrol_players();
        if players.is_empty() {
            return;
        }
        let (player, spectator) =
            players[random.next_int_bound(players.len() as i32).unwrap_or(0) as usize];
        if spectator || close_to_village(level, player, 2) {
            return;
        }
        let x = (24 + random.next_int_bound(24).unwrap_or(0))
            * if random.next_boolean() { -1 } else { 1 };
        let z = (24 + random.next_int_bound(24).unwrap_or(0))
            * if random.next_boolean() { -1 } else { 1 };
        let mut pos = [player[0] + x, player[1], player[2] + z];
        if !has_chunks_at(level, pos[0] - 10, pos[2] - 10, pos[0] + 10, pos[2] + 10)
            || !level.can_pillager_patrol_spawn(pos)
        {
            return;
        }
        let group = level.effective_difficulty(pos).ceil() as i32 + 1;
        for i in 0..group {
            pos[1] = level.motion_blocking_no_leaves(pos[0], pos[2]);
            let leader = i == 0;
            if !spawn_member(level, pos, leader, random, spawn) && leader {
                break;
            }
            pos[0] += random.next_int_bound(5).unwrap_or(0) - random.next_int_bound(5).unwrap_or(0);
            pos[2] += random.next_int_bound(5).unwrap_or(0) - random.next_int_bound(5).unwrap_or(0);
        }
    }
}

/// `PatrolSpawner.spawnPatrolMember`: an empty spawn block for a pillager
/// whose `checkPatrollingMonsterSpawnRules` pass.
fn spawn_member<L: PatrolLevel, R: RandomSource>(
    level: &L,
    pos: [i32; 3],
    leader: bool,
    random: &mut R,
    spawn: &mut dyn FnMut(&L, Member, &mut R) -> bool,
) -> bool {
    let [x, y, z] = pos;
    if !spawn_rules::is_valid_empty_spawn(Species::Pillager, level, x, y, z) {
        return false;
    }
    if !spawn_rules::check_spawn_rules(Species::Pillager, level, Reason::Patrol, random, x, y, z) {
        return false;
    }
    spawn(level, Member { pos, leader }, random)
}

/// `PatrollingMonster.findPatrolTarget` from a block position.
pub fn patrol_target(at: [i32; 3], random: &mut impl RandomSource) -> [i32; 3] {
    let dx = -500 + random.next_int_bound(1000).unwrap_or(0);
    let dz = -500 + random.next_int_bound(1000).unwrap_or(0);
    [at[0] + dx, at[1], at[2] + dz]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blocks::BlockFacts;
    use crate::mobs::data::MobData;
    use horizon_core::random::LegacyRandom;
    use horizon_world::{dimension_type::DimensionFacts, registry::BlockStateId};
    use std::cell::Cell;

    /// A level with one player in daylight far from any village, in a biome
    /// that forbids patrols; the spawn-rule reads never run (the biome check
    /// stops the tick first), so they are unreachable.
    struct Stub {
        day_time: i64,
        biome_checked: Cell<bool>,
    }

    impl SpawnLevel for Stub {
        fn state(&self, _: i32, _: i32, _: i32) -> BlockStateId {
            unreachable!()
        }
        fn facts(&self) -> &BlockFacts {
            unreachable!()
        }
        fn mob_data(&self) -> &MobData {
            unreachable!()
        }
        fn block_tag(&self, _: &str, _: &str) -> bool {
            unreachable!()
        }
        fn biome_tag(&self, _: &str, _: i32, _: i32, _: i32) -> bool {
            unreachable!()
        }
        fn sky_light(&self, _: i32, _: i32, _: i32) -> i32 {
            unreachable!()
        }
        fn block_light(&self, _: i32, _: i32, _: i32) -> i32 {
            unreachable!()
        }
        fn sky_darken(&self) -> i32 {
            unreachable!()
        }
        fn thundering(&self) -> bool {
            unreachable!()
        }
        fn sea_level(&self) -> i32 {
            unreachable!()
        }
        fn world_surface(&self, _: i32, _: i32) -> i32 {
            unreachable!()
        }
        fn peaceful(&self) -> bool {
            unreachable!()
        }
        fn dimension_type(&self) -> &DimensionFacts {
            unreachable!()
        }
        fn moon_brightness(&self) -> f32 {
            unreachable!()
        }
        fn world_seed(&self) -> i64 {
            unreachable!()
        }
        fn player_near(&self, _: f64, _: f64, _: f64, _: f64) -> bool {
            unreachable!()
        }
    }

    impl PatrolLevel for Stub {
        fn spawn_patrols(&self) -> bool {
            true
        }
        fn day_time(&self) -> i64 {
            self.day_time
        }
        fn bright_outside(&self) -> bool {
            true
        }
        fn patrol_players(&self) -> Vec<([i32; 3], bool)> {
            vec![([0, 64, 0], false)]
        }
        fn sections_to_village(&self, _: [i32; 3]) -> i32 {
            7
        }
        fn has_chunk(&self, _: i32, _: i32) -> bool {
            true
        }
        fn can_pillager_patrol_spawn(&self, _: [i32; 3]) -> bool {
            self.biome_checked.set(true);
            false
        }
        fn effective_difficulty(&self, _: [i32; 3]) -> f32 {
            unreachable!()
        }
        fn motion_blocking_no_leaves(&self, _: i32, _: i32) -> i32 {
            unreachable!()
        }
    }

    /// A seed whose 1-in-5 patrol roll passes: the second draw, after the
    /// countdown reset's 1200-bound draw.
    fn passing_seed() -> i64 {
        for seed in 0..100 {
            let mut r = LegacyRandom::new(seed);
            let _ = r.next_int_bound(1200);
            if r.next_int_bound(5) == Ok(0) {
                return seed;
            }
        }
        panic!("no passing seed in 0..100")
    }

    #[test]
    fn patrols_wait_until_day_five() {
        let seed = passing_seed();
        let mut spawn = |_: &Stub, _: Member, _: &mut LegacyRandom| -> bool {
            panic!("no patrol member spawns in this test")
        };
        // Day 4, daylight, a player far from any village: the gate stops
        // the tick before the biome check.
        let day4 = Stub {
            day_time: 4 * 24000 + 12000,
            biome_checked: Cell::new(false),
        };
        let mut spawner = PatrolSpawner::default();
        spawner.tick(&day4, true, &mut LegacyRandom::new(seed), &mut spawn);
        assert!(
            !day4.biome_checked.get(),
            "day 4 never reaches the biome check"
        );
        // The last tick of day 4: still gated.
        let last = Stub {
            day_time: 5 * 24000 - 1,
            biome_checked: Cell::new(false),
        };
        let mut spawner = PatrolSpawner::default();
        spawner.tick(&last, true, &mut LegacyRandom::new(seed), &mut spawn);
        assert!(
            !last.biome_checked.get(),
            "day 4's last tick never reaches the biome check"
        );
        // Day 5: the gate opens and the tick reaches the biome check.
        let day5 = Stub {
            day_time: 5 * 24000,
            biome_checked: Cell::new(false),
        };
        let mut spawner = PatrolSpawner::default();
        spawner.tick(&day5, true, &mut LegacyRandom::new(seed), &mut spawn);
        assert!(day5.biome_checked.get(), "day 5 reaches the biome check");
    }
}
