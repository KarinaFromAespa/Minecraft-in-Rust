# Batch 3 — Entities / mobs (1.21.10 parity sweep)

**Version:** 1.21.10 · **Date:** 2026-11-25 · **Branch:** `parity-sweep-1.21.10`
(workdir `~/workspace/mcir-work/sweep-1.21.10`) · **Scope:**
`docs/PARITY-SWEEP.md` §4 Batch 3 — mob AI, spawning, trading, attributes,
projectiles, vehicles; 153 entity types (72 MISC, 32 CREATURE, 40 MONSTER,
1 AXOLOTLS, 1 AMBIENT, 4 WATER_AMBIENT, 2 WATER_CREATURE, 1 UNDERGROUND
WATER_CREATURE); 144 drivers curated to 135 core (see §D).

**Status: checks + triage only, per the brief: no fixes, no edits under
`versions/1.21.10/`, no cargo builds, no driver runs, no oracle repairs, no
commits — the coordinator commits these deliverables.** Everything below is
static analysis: `javap -c` on the named 1.21.10 server-game jar
(`~/workspace/mcir-work/e6-deltas/versions/1.21.10/jars/server-game-named.jar`,
sha1 `ba1fc89ead95b8ebdb8727f84e2bd487533c8b7f`, 6,964 classes; client-named
jar sha1 `715f58165acfaf8b61ab921064f08c202b41bd0c`, 9,969 classes), source
comparison against `versions/1.21.10/crates/`, JVM-free class-file scans
over the jar's entity classes, and checks against the 1.21.10 datagen
reports (`e6-deltas/versions/1.21.10/datagen/reports/`). **No prepared
1.21.10 `.local` data exists** — every data-dependent check below rests on
the jars and the datagen reports, never on prepared entity data; the one
check that needs prepared data (A3 attribute values) is recorded as
not-verifiable-here rather than guessed.

**Method — fresh anchors, no carry-over assumed.** 1.21.10 is a different
lineage from 26.1.2 (E6, not 26.x): its entity package layout is flatter
(`animal/Cow`, `monster/Zombie`, `projectile/Arrow`, `vehicle/Boat`,
`vehicle/AbstractMinecart`, `npc/Villager`), it has no `EntityTypes` /
`BlockEntityTypes` holder classes, no `gamerules` / `world.clock` /
`densityfunction` / `environment_attribute` packages, no
`ServerboundPunchPacket`, and its advancements criterion package is spelled
`critereon` (164 classes). Every vanilla anchor below was verified fresh
against 1.21.10's own jars (88 `javap` dumps in `evidence/javap/`, plus
fresh class-parser extractions: the 13 `BABY_DIMENSIONS` declarers, the
`getDimensions` / `getScale` / `getDefaultDimensions` / `getAgeScale`
override sets, the 70 `createAttributes` suppliers, the 15 fire-immune
types, all 153 adult dimensions + eye heights), never carried from 26.1.2
or 1.21.7. Where the 1.21.10 port was adapted for its own version (the
baby-dims table, the piglin finalize, the cat goals, the drowned burn), the
adaptation was verified line by line rather than assumed correct.

**Re-check first:** every re-check was decided against 1.21.10's own
jar/data — the 26 PRIMARY items from the 1.21.7 run and the 46 SECONDARY
items from the 26.1.2 run. Of the 1.21.7 items, 19 reproduce, 4 are FIXED
(G02 slime moon phase, G08 drowned burn, G09 powder-snow burn, G16 trades),
3 are WITHDRAWN (G12 enderman walk-target — vanilla 1.21.10 changed;
G17 `reduced()` — vanilla 1.21.10 changed; O02 client.multiplayer — the
package exists in 1.21.10's client jar). Of the 26.1.2 items, 42 reproduce,
2 are FIXED (G09 powder-snow burn, G36 cat goals), 1 NEVER EXISTED (G42
piglin offhand — vanilla sets persistence + 2.0 drop, exactly what the port
does), 1 is WITHDRAWN (G45 spear stab — no kinetic weapons in 1.21.10).
Two NEW defects were found (B3-G48, zombie horses burn in daylight where
vanilla spares them; B3-G49, the LAN idle timer resets before the
`hasClientLoaded` gate). The two majors reproduce: **B3-G14 endermen never
teleport** and **B3-G22 the suffocation startup blocker** — the 1.21.10
client cannot load its core tables with prepared 1.21.10 data.

## Tally

- **Checks: 97 — 61 pass, 36 gap as-tallied** (39 G items: 36 port +
  3 driver — the A3 food-table check yields three items, G05/G27/G28, and
  the B3 enderman/cat check yields two, G14/G15; every other gap check
  yields one). **Behaviour/non-behaviour split:** 94 behaviour checks
  (61 pass, 33 gap) + 3 non-behaviour driver-code checks (0 pass, 3 gap:
  G18, G19, G20) — driver-code-defect checks counted as-tallied per the
  26.2 precedent, split reported explicitly.
- **Oracle defects: 11 groups** (B3-O01–B3-O11 — not counted in the check
  tally). O01–O08 carry the 26.1.2 grouping; O09–O11 are the 1.21.7-only
  groups (gamerules package, entity subpackages, world.clock package),
  which have no 26.1.2 counterpart because those packages exist in 26.x.
- **Oracle repairs done: none** (checks + triage only).
- **Severity: 2 major, 48 minor** — 2 major port (B3-G22, the
  `#causes_suffocation` startup blocker; B3-G14, endermen never teleport),
  34 minor port (incl. new B3-G48, B3-G49), 3 minor driver-side (B3-G18,
  G19, G20), 11 minor oracle groups.
- **Slice items:** `slice.json` carries 59 items — the 50 live defects
  above plus 9 resolved re-checks per common.md (5 fixed: G02, G08, G09,
  G16, G36; 4 closed: G12, G17, G42, G45). The withdrawn 1.21.7 oracle
  group O02 (client.multiplayer) is recorded in the report only, since
  its ID is taken by the live constructor-mismatch group.
- **Carry-over:** 19 of the 26 1.21.7 items re-mint (G01, G03, G04, G05,
  G06, G10, G11, G13, G14, G15, G18, G19, G20, G21, O01, O03, O04, O05,
  O06 — O01/O03/O04 re-mint as O09/O10/O11 under the 26.1.2 grouping's
  numbering); 42 of the 46 26.1.2 items re-mint. The 1.21.10 port fixed
  six carried defects outright (G02, G08, G09, G16, G36, and the G21
  finalize arm) and narrowed two more (G30: 29 species wrong → 18; G15:
  the goal-start gate fixed, the hop still broken).

Per-section counts (must sum to 97): A1 1 (1 pass), A1b 11 (9 pass,
2 gap), A2 1 (1 pass), A3 10 (5 pass, 5 gap), A4 5 (4 pass, 1 gap),
B1 3 (1 pass, 2 gap), B2 27 (20 pass, 7 gap), B7 4 (3 pass, 1 gap),
B8 4 (2 pass, 2 gap), B5 4 (3 pass, 1 gap), B4 6 (4 pass, 2 gap),
B6 3 (2 pass, 1 gap), B3 12 (4 pass, 8 gap), B9 3 (2 pass, 1 gap),
Drivers 3 (0 pass, 3 gap).

## Defects

### B3-G01 — Bat: missing Halloween branch (minor, port)

Vanilla 1.21.10 `Bat.checkBatSpawnRules` (fresh dump `Bat.txt:540-600`):
on Halloween (`isHalloween`, `LocalDate` Oct 31) the light limit is 7 and
the 50% `nextBoolean()` coin flip is skipped; otherwise limit 4 + the coin
flip. The port's Bat arm (`mobs/spawn_rules.rs`, `check_spawn_rules`) has
no Halloween branch at all — it always applies the coin flip +
`next_int_bound(4)` — and `SpawnLevel` has no date source, so the branch
needs a trait addition. Reproduces from 1.21.7 (it was fixed in 26.1.2;
the 1.21.10 port does not carry that fix).

### B3-G03 — `immune_tag` tags don't exist (minor, port)

Vanilla 1.21.10 `EntityType.<clinit>` passes LITERAL block sets to
`immuneTo` (6 calls, verified in the fresh dump: fox→sweet_berry_bush;
polar_bear/snow_golem/stray→powder_snow; wither/wither_skeleton→
wither_rose). The port's `immune_tag` (`mobs/spawn_rules.rs:181-190`)
resolves six block tags (`fox_immune_to`, `polar_bear_immune_to`,
`snow_golem_immune_to`, `stray_immune_to`, `wither_immune_to`,
`wither_skeleton_immune_to`) — none of which exists in 1.21.10 data
(`wither_immune_to` exists only as a damage_type tag). Reproduces from
1.21.7 and 26.1.2.

### B3-G04 — Hoglin wrongly disallowed in peaceful (minor, port)

Vanilla 1.21.10 `notInPeaceful` = 36 types (parsed fresh from
`EntityType.<clinit>`; PHANTOM included, HOGLIN not among them). The
port's `allowed_in_peaceful` (`mobs/spawn_rules.rs:136`) disallows 38,
including Hoglin (plus Parched, a dead 26.x-only arm). Hoglins spawn and
persist in peaceful in vanilla 1.21.10; the port despawns them.
Reproduces from 1.21.7 and 26.1.2.

### B3-G05 — Camel has no food (minor, port)

Vanilla 1.21.10 `Camel.isFood` = `#camel_food` (the tag exists in the
client jar). The port's `food_tag()` (`mobs/species.rs:1666`) has no
Camel arm (it covers CamelHusk, Nautilus, ZombieNautilus and SulfurCube
instead — all 26.x-only). Camels cannot breed in the port. Reproduces
from 1.21.7 and 26.1.2.

### B3-G06 — Armadillo scared head rotation ignored (minor, port)

Vanilla 1.21.10 `Armadillo.getMaxHeadYRot` = `isScared() ? 0 : 32`. The
port's `max_head_y_rot` (`mobs/species.rs:1745`) returns 32.0
unconditionally. Reproduces from 1.21.7 and 26.1.2.

### B3-G10 — Creatures spawn too far from players (minor, port)

Vanilla 1.21.10 `canSpawnFarFromPlayer` defaults to MISC, with exactly
two explicit setters (PILLAGER, SHULKER — parsed fresh from
`EntityType.<clinit>`). The port's `spawns_far_from_player`
(`mobs/spawn_selection.rs:317-320`) returns true for CREATURE and MISC
plus Pillager/Shulker, so creatures spawn in the far band where vanilla
never places them. Reproduces from 1.21.7 (it was fixed in 26.1.2; the
1.21.10 port does not carry that fix).

### B3-G11 — `getRandomPosWithin` y-range off by one (minor, port)

Vanilla 1.21.10 `NaturalSpawner.getRandomPosWithin`: top =
`getHeight(WORLD_SURFACE)+1`, y drawn with `randomBetweenInclusive(minY,
top)` — H+1 is reachable. The port's `spawn_category_for_chunk`
(`mobs/spawn_selection.rs:1090`) uses top = H, y in [minY, H], so the top
level never spawns and the RNG stream diverges. Reproduces from 1.21.7
and 26.1.2.

### B3-G13 — Constant-count packs skip an RNG draw (minor, port)

Vanilla 1.21.10 draws `minCount + nextInt(1+maxCount-minCount)`
unconditionally at both sample sites in `NaturalSpawner` (offsets ~298–318
and ~91–108 in the fresh dump) — even when min == max. The port's
`sample_count` (`mobs/spawn_selection.rs:780`) returns the constant
without drawing, desyncing every later draw in the chunk's spawn pass.
Reproduces from 1.21.7 and 26.1.2.

### B3-G14 — Endermen never teleport (MAJOR, port)

Vanilla 1.21.10 `EnderMan.teleport` + `LivingEntity.randomTeleport`
ground the destination search on `blocksMotion()`. The port
(`mobs/enderman.rs:117`) grounds on `#entities_can_teleport_to`, a tag
that does not exist in 1.21.10 data (0 entries in the client jar) — the
search finds nothing and the teleport never executes. Reproduces from
1.21.7 and 26.1.2.

### B3-G15 — Cat morning gift never happens (minor, port)

Half fixed, half not. The goal-start gate is FIXED: it now uses `#beds`
(`mobs/tame.rs:1246`). But the hop still grounds on the absent
`#entities_can_teleport_to` (`mobs/tame.rs:1341`, scanning to the world
bottom, never executing) and the leash-holder origin is dropped (vanilla
uses `getLeashHolder().blockPosition()` when leashed). The gift itself
still drops (`DropGift` sits outside the hop gate), so the observable
defect is the missing morning hop. Reproduces from 1.21.7 and 26.1.2.

### B3-G21 — Zombies spawn holding iron spears (minor, port)

Vanilla 1.21.10 `Zombie.populateDefaultEquipmentSlots`: nextFloat <
(hard ? 0.05 : 0.01), then nextInt(3): 0→iron sword, else iron shovel —
no spear; `iron_spear` does not exist in 1.21.10 (0 matches in datagen
`items.json`). The port is split: `finalize.rs:1346-1348` is FIXED
(nextInt(3), sword/shovel, comment "(no spear)"), but `spawning.rs:221-223`
still uses nextInt(6) with 1→`minecraft:iron_spear` — a nonexistent item
in the zombie's hand. Reproduces from 1.21.7 (one arm only).

### B3-G22 — Default suffocation is tag-gated; startup blocker (MAJOR, port)

The suffocation **rule** is fixed in 1.21.10: vanilla's default is
`blocksMotion() && isCollisionShapeFullBlock()` (indy#6 →
`lambda$new$7` in `BlockProperties`, confirmed via `javap -v`
BootstrapMethods — no tag), and the port's bind has `_ =>
MotionFullCollision` (`mobs/suffocation.rs`; `horizon-world/src/
suffocation.rs` documents that 1.21.10 has no tag). But the startup path
still fails: `resource_load.rs:324` `gameplay_tables` →
`MotionResources::load_with_facts` requires `tag("minecraft:
suppresses_bounce")?` (`motion.rs:142`, first failure in evaluation
order) and `tag("minecraft:causes_suffocation")?` (`motion.rs:145`), both
absent from 1.21.10 data; `tag()` errors on missing tags, and
`horizon-prepare` synthesizes neither. So the 1.21.10 client cannot load
its core tables with prepared 1.21.10 data. (The `world_entry.rs:266`
path uses a tolerant `.get().into_iter().flatten()` form and is fine.)
Reproduces from 26.1.2.

### B3-G23 — Sleeping mobs suffocate (minor, port)

Vanilla 1.21.10 `LivingEntity.isInWall` = `!isSleeping() &&
super.isInWall()` — a sleeper never suffocates. The port's base_tick
in-wall check (`mobs/ai.rs:1266`) has the Vex exclusion but no sleeping
check, so a villager asleep with its head in a wall takes suffocation
damage. Reproduces from 26.1.2.

### B3-G24 — Villager level-up is immediate (minor, port)

Vanilla 1.21.10 `Villager.rewardTradeXp` defers the level-up:
`updateMerchantTimer = 40` + `increaseProfessionLevelOnUpdate = true`.
The port's `Trader::notify_trade` (`merchant.rs:404-405`) levels up
immediately and calls `update_special_prices()` on the new offers (lines
411, 422). No `level_up_pending` / `update_merchant_timer` exists
anywhere in the tree. Reproduces from 26.1.2.

### B3-G25 — Furnace minecart too fast under `minecart_improvements` (minor, port)

Vanilla 1.21.10 `MinecartFurnace.getMaxSpeed` = super × 0.75 (wet) / 0.5
(dry), and the new behaviour calls the VIRTUAL in `calculateTrackSpeed`
and `comeOffTrack` (confirmed in the fresh dumps). The port's old-behavior
`max_speed` (`game/minecarts.rs:750-757`) HAS the furnace factor, but the
new-behavior `max_speed_new` (`:991-994`) has none and is used at all
three sites — the step cap (:1069) is correct per vanilla, but
`calculateTrackSpeed` (:1208) and `comeOffTrack` (:1344) are wrong.
Experiment-gated: `DEFAULT_FLAGS` = `VANILLA_SET` (`FeatureFlags.txt`),
`minecart_improvements` a separate flag, so the defect only manifests
with the experiment on. Reproduces from 26.1.2.

### B3-G26 — BreedGoal ignores partner panic (minor, port)

Vanilla 1.21.10 checks `isPanicking()` in both `BreedGoal.getFreePartner`
and `canContinueToUse`. The port's partner search and `can_continue`
(`mobs/goals.rs`, `alive && in_love && love_time < 60`) omit the panic
clause, so a panicking animal keeps breeding. Reproduces from 26.1.2.

### B3-G27 — Zombie horse has no food (minor, port)

Reproduces with corrected evidence. The 26.1.2 item's tag premise does
not carry: vanilla 1.21.10 has NO `#zombie_horse_food` tag and
`ZombieHorse` does NOT override `isFood` (0 occurrences in the complete
133-line dump) — it inherits `AbstractHorse.isFood` = `#horse_food`. The
port's `food_tag()` has no ZombieHorse arm, so zombie horses cannot
breed. Reproduces from 26.1.2.

### B3-G28 — Skeleton horse has no food (minor, port)

`SkeletonHorse` has 0 `isFood` occurrences → inherits `#horse_food`;
the port covers Horse|Donkey|Mule only. Reproduces from 26.1.2.

### B3-G29 — Peaceful gate moved into `checkSpawnRules` (minor, port)

Vanilla 1.21.10 `SpawnPlacements.checkSpawnRules` has NO peaceful gate
(only `Map.get` + `SpawnPredicate.test`); the gate lives in the per-type
predicates. The port moved the gate into `check_spawn_rules`
(`mobs/spawn_rules.rs:322`) and dropped the per-type gates from
`monster_rules` (:285), blaze/breeze/zoglin (:376), endermite/silverfish
(:395) and pillager (:439) — so phantom/shulker now fail in peaceful
where vanilla lets them spawn. (Drowned/Guardian/Ghast/MagmaCube/
ZombifiedPiglin arms retain their own `!l.peaceful()` gates.)
Reproduces from 26.1.2.

### B3-G30 — Baby-dimensions table carries the 26.2 form; 18 species wrong (minor, port)

Vanilla 1.21.10 baby dims, re-extracted FRESH from the 1.21.10 jar with
the JVM-free class parser (`evidence/baby_dims2-1.21.10.py`, output in
`evidence/baby-dims-1.21.10.txt` + `evidence/adult-dims-1.21.10.txt`):
13 `BABY_DIMENSIONS` declarers (AbstractCow scale 0.5 eye 0.665, Chicken
0.5/0.2975, Fox 0.5/0.2975, Panda 0.5, Turtle 0.3, Horse/Llama/
SkeletonHorse/ZombieHorse 0.5, ArmorStand 0.5/0.9875, Zombie 0.5/0.93,
ZombifiedPiglin 0.5/0.97, Piglin 0.5/0.97); Drowned/Husk/ZombieVillager
inherit Zombie's; Donkey/Mule inherit `AbstractChestedHorse.babyDimensions`
(adult × 0.5); every other ageable species scales by its age scale via
`LivingEntity.getDefaultDimensions` (0.5 default; Dolphin 0.65,
HappyGhast 0.2375 — the ghastling IS a baby happy ghast, no separate
type — Armadillo 0.6, Camel 0.45, Turtle 0.3, all extracted fresh from
`getAgeScale` bytecode). The port's table (`mobs/species.rs` `Def.baby`,
41 non-None entries parsed fresh) was partly adapted for 1.21.10
(Drowned/ZombieVillager now inherit Zombie's; Sniffer's swap is gone;
Villager/Squid/Axolotl/Fox/Turtle correct) but 18 species are still
wrong: pig eye 0.40625 vs 0.3825; cow/mooshroom eye 0.69 vs 0.665;
sheep eye 0.65625 vs 0.6175; chicken w/h/eye 0.3/0.4/0.28125 vs
0.2/0.35/0.2975; goat eye 0.59375 vs 0.5525; panda eye 0.28125 vs
0.53125; polar bear eye 0.34375 vs 0.595; wolf eye 0.34375 vs 0.34; cat
eye 0.34375 vs 0.175; ocelot eye 0.34375 vs 0.2975; armadillo eye
0.21875 vs 0.156 (scale 0.6 correct); dolphin eye 0.09375 vs 0.195
(scale 0.65 correct); happy ghast eye 0.46875 vs 0.6175 (scale 0.2375
correct); camel w/h/eye 0.95/1.4/1.38 vs 0.765/1.06875/1.02375; hoglin
and zoglin w/h/eye 0.75/0.85/0.625 vs 0.6982422/0.7/0.595; strider eye
0.4375 vs 0.7225. Reproduces from 26.1.2 (29 species wrong there; 11
fixed here).

### B3-G31 — Camel sitting baby dims wrong; body-anchor offset uses the 26.2 double form (minor, port)

Vanilla 1.21.10 `Camel.SITTING_DIMENSIONS` = scalable(width, height−1.43)
with eye 0.845 (verified in the fresh Camel dump: `1.43f`/`0.845f` in
`<clinit>`); sitting adult = (1.7, 0.945), eye 0.845, and the sitting
baby scales by the camel age scale 0.45 (extracted fresh from
`Camel.getAgeScale`) to (0.765, 0.425, 0.38).
The port's `sitting_dims` (`mobs/camel.rs:52-54`) returns (0.95, 0.425,
0.41) for a baby — width and eye wrong (height coincidentally right). The port's `body_anchor_y` (`camel.rs:124-125`) uses
the double form (0.09375 baby / 0.375 adult offsets) vs vanilla float
`height − 0.375F * scale`. Reproduces from 26.1.2.

### B3-G32 — Heightmap predicates use absent tags (minor, port)

Vanilla 1.21.10 heightmap predicates are code-based, never tag-based:
`MATERIAL_MOTION_BLOCKING` = `BlockStateBase::blocksMotion` (method
reference, `Heightmap-v.txt` BootstrapMethods #1);
`MOTION_BLOCKING` = blocksMotion() || fluid not empty; `MOTION_BLOCKING_
NO_LEAVES` = the same && !(block instanceof LeavesBlock)
(`HeightmapTypes.txt:119-144`); `WORLD_SURFACE` = NOT_AIR = !isAir. The
port's predicates (`mobs/spawn_selection.rs:417-422`) ground on
`#blocks_motion_in_heightmap*` tags that do not exist in 1.21.10 data.
Reproduces from 26.1.2.

### B3-G33 — Worn heads never reduce detection range (minor, port)

Vanilla 1.21.10 `LivingEntity.getVisibilityPercent` hardcodes five head
rules (skeleton, zombie, piglin, piglin-brute, creeper heads, each ×0.5)
and has NO clamp (plain `dreturn`). The port's `visibility_percent`
(`mobs.rs:3171`) reads the `mob_visibility` item component — 0
occurrences in 1.21.10's 1,488 datagen items — and applies a 0..=10 clamp
vanilla lacks (dead code: visibility never exceeds 1.0). Reproduces
from 26.1.2.

### B3-G34 — Trader hurt gating uses the 10-tick hurt window (minor, port)

Vanilla 1.21.10 `TradeWithPlayerGoal` / `LookAndFollowTradingPlayerSink`
gate on `!hurtMarked`. The port gates on `m.hurt_time <= 0`
(`mobs/trader.rs:198`, `mobs/villager_brain.rs:3464`) — `hurt_time` is
set to 10 on hit (`mobs.rs:3885`) and decremented per tick (`mobs/
ai.rs:1353`), so a trader stops trading for up to 10 ticks after any hit
where vanilla only stops while `hurtMarked`. Reproduces from 26.1.2.

### B3-G35 — BreathAirGoal uses the five-column search with swapped start order (minor, port)

Vanilla 1.21.10 `BreathAirGoal.findAirPosition` searches
`BlockPos.betweenClosed(x-1, y, z-1, x+1, y+8, z+1)` (3×9×3 box), and
`start()` calls `findAirPosition()` ONLY — there is no `nav.stop()` at
all in 1.21.10 (this corrects the 26.1.2 report's "findAirPosition THEN
nav.stop()" framing). The port uses the `neighborColumn` five-column
search (`mobs/goals.rs` `air_position`) with `start()` calling
`nav_stop()` FIRST, then `find_air`. Reproduces from 26.1.2.

### B3-G37 — Inhabited time accrues +1 unconditionally (minor, port)

Vanilla 1.21.10 `ServerChunkCache.tickChunks` computes `timeDiff =
gameTime − lastInhabitedUpdate` → `incrementInhabitedTime(timeDiff)`.
The port adds 1 unconditionally (`game/natural_spawning.rs:808`); no
`last_inhabited_update` field exists in the render world. Reproduces
from 26.1.2.

### B3-G38 — `fresh_dimensions` removed; pufferfish spawn at full size (minor, port)

Vanilla 1.21.10 `Pufferfish.getScale(0) = 0.5` — a fresh spawn is
half-size. 0 `fresh_dimensions` references in
`game/natural_spawning.rs`. Reproduces from 26.1.2.

### B3-G39 — Spawn-egg entity data applied to spawners (minor, port)

Vanilla 1.21.10 `SpawnEggItem.useOn` calls only
`Spawner.setEntityId(type, random)`; `TrialSpawnerBlockEntity` has no
`setEntityData` method (0 occurrences). The port applies egg data to
both: the trial arm (`game/spawner.rs:450` +
`mobs/trial_spawner.rs` `override_entity_data`) and the regular arm
(`game/spawner.rs:464` `set_entity_data`). Reproduces from 26.1.2.

### B3-G40 — Armadillo ball-up refuses in liquid (minor, port)

Vanilla 1.21.10 `ArmadilloBallUp.checkExtraStartConditions` =
`onGround()` only. The port (`mobs/armadillo.rs:379`) adds `!liquid(m)`
(in water or lava). Reproduces from 26.1.2.

### B3-G41 — Slime dealDamage adds a team check vanilla lacks (minor, port)

Vanilla 1.21.10 `Slime.dealDamage` = alive + `isWithinMeleeAttackRange`
+ `hasLineOfSight` + `mobAttack`, NO team check. The port
(`mobs/cube.rs:242, 296`) adds `teams_allow_damage`. Reproduces from
26.1.2.

### B3-G43 — Fox chorus-fruit teleport emits a level event 2017 trail (minor, port)

Vanilla 1.21.10 `TeleportRandomlyConsumeEffect` has ONLY a `diameter`
field; `apply` emits the TELEPORT game event + sound, no level event.
The port parses `directional_particles` with `.unwrap_or(true)`
(`horizon-items/src/items.rs:157-160`); 1.21.10 chorus-fruit data is
`{"type": "minecraft:teleport_randomly"}` with no such field, so the fox
path (`mobs/fox.rs:388-398`) emits level event 2017 on every
chorus-fruit teleport. Reproduces from 26.1.2.

### B3-G44 — Loaded sleeping villager position never refreshed (minor, port)

Vanilla 1.21.10 `LivingEntity.baseTick` calls `setPosToBed` (flat
0.6875 above the head-block base). The port's `sleeping_position`
(`mobs/world.rs:141-145`) gates on `facts.has_class("AbstractBedBlock")`
— a class with 0 entries in the 1.21.10 jar (only plain `BedBlock`
exists) — so it always returns None and `refresh_sleeping_position`
(`mobs/villager.rs:117-121`) never moves the sleeper. Reproduces from
26.1.2.

### B3-G46 — Item-entity landing bounce uses the restitution form; bed bounce gone (minor, port)

Vanilla 1.21.10 has NO `restituteMovementAfterCollisions` /
`bounceRestitution` (0 occurrences in the `Entity` dump); bounce lives
in the blocks: `SlimeBlock.bounceUp` = −vy × (1.0 living / 0.8
non-living) at any falling speed, `BedBlock.bounceUp` = −vy × 0.66 ×
(1.0 / 0.8). The port's `vertical_collision_velocity`
(`entities.rs:36-52`) uses the 26.2 restitution form with a gravity
threshold (`-current_y > gravity`) and no bed branch. Reproduces from
26.1.2.

### B3-G47 — AI walk-target value misses the Zero arms and the Giant/Mooshroom arms (minor, port)

The spawn-path table is correct (`mobs/spawn_selection.rs:145`:
Enderman/Creaking/Pillager/Warden → Zero, Giant → +cost, Mooshroom →
mycelium — all verified against the fresh dumps). But the AI-path
`walk_target_value` (`mobs/ai.rs:2608-2682`) misses them: Enderman,
Pillager and Warden fall into the hostile branch → −light() where
vanilla returns 0.0f (fresh dumps: `EnderMan`, `Pillager`, `Warden`
all `fconst_0; freturn`); Giant → −light() where vanilla returns
+`getPathfindingCostFromLightLevels` (unnegated); Mooshroom → 10.0 over
GRASS_BLOCK where vanilla wants MYCELIUM. (Creaking IS correct in the AI
path here — `ai.rs:2653-2659` — unlike 26.1.2.) Silverfish is correct
(10.0 over infested host, else Monster's −cost). Reproduces from 26.1.2
(four wrong arms, one fewer than 26.1.2).

### B3-G48 — NEW: Zombie horses burn in daylight where vanilla spares them (minor, port)

Vanilla 1.21.10's burn set is code-driven (no `#burn_in_daylight` tag
exists): {Zombie, Drowned, ZombieVillager} via `Zombie.aiStep` +
`isSunSensitive()`; {Skeleton, Stray, Bogged, WitherSkeleton} via
`AbstractSkeleton.aiStep` (burns on `isSunBurnTick`, HEAD-slot
protection); {Phantom} via `Phantom.aiStep`. Husk overrides
`isSunSensitive()` = false (does not burn — the port correctly excludes
it). **ZombieHorse has no burn path at all** (complete 133-line dump:
no `aiStep`, no `isSunSensitive`, no `isSunBurnTick`; `AbstractHorse`
also has none). The port's `burns_in_day` (`mobs/species.rs:1576`)
includes ZombieHorse (plus dead ZombieNautilus), and `burn_undead`
(`mobs/ai.rs:2004`) ignites it for 160 ticks with body-slot protection
via `drops::body_slot_species` — doubly wrong, since vanilla 1.21.10
has no `getSunProtectionSlot` / body-slot protection at all (all
burners protect via HEAD). NEW in 1.21.10.

### B3-G49 — NEW: Idle timer resets before the `hasClientLoaded` gate (minor, port)

Vanilla 1.21.10 `handleInteract` returns BEFORE `resetLastActionTime`
when `!hasClientLoaded()` (`ServerGamePacketListenerImpl.txt:4962-5019`).
The port's `process_inbox` (`lan/hub.rs:4039-4060`) resets
`last_action_tick` for Interact/UseItem/PlayerAction/PlayerCommand
(`acted = true` via the `_` arm) BEFORE the `client_loaded()` gate in
the dispatch match (`:4063-4068`), so a not-yet-loaded client's
interacts keep its idle timer fresh where vanilla leaves it untouched.
(The spectator form matches vanilla 1.21.10 — see B8 — this is only the
gate order.) NEW in 1.21.10.

### B3-G18 — DRIVER: `test_entity_components_parity.py` crashes on 1.21.10 advancement format (minor, driver)

The driver reads `criterion['conditions']['entity']['predicate']` then
`predicate['minecraft:components']` (26.x form). In 1.21.10,
`conditions.entity` is a LIST of `entity_properties` loot conditions
(113 list-form entity conditions tree-wide, 0 dict-form — verified in
the client jar; the three advancements use `predicate: {components:
{...}}` with key `components`, not `minecraft:components`), so the
driver raises TypeError. Reproduces from 1.21.7 and 26.1.2.

### B3-G19 — DRIVER: state-count hardcode 35723 (minor, driver)

`tools/test_suffocation_parity.py:66` and
`tools/test_registered_entity_fluid_parity.py:66` assert the state count
is 35723 (26.3's count). 1.21.10 has 29,671 states, so both drivers
always fail. Reproduces from 1.21.7 and 26.1.2. (36 drivers tree-wide
hardcode 35723 — see §D.)

### B3-G20 — DRIVER: 26.3 resources path (minor, driver)

Five drivers build the Rust server command with `lab.LOCAL /
'horizon-client/26.3'`: `test_locator_bar_parity.py:269`,
`test_server_dragon_parity.py:111`,
`test_server_entities_parity.py:186`,
`test_server_knockback_parity.py:435`, `test_projectile_shoot.py:46`.
1.21.10 resources live at `horizon-client/1.21.10`. Reproduces from
1.21.7 and 26.1.2. (45 drivers tree-wide use the narrow
`horizon-client/26.3` path — see §D.)

## Re-checked, not defects

- **B3-G02 (1.21.7) — FIXED.** The port's `slime_rules`
  (`mobs/spawn_rules.rs:511`) draws twice (0.5, then moon brightness),
  with an explicit comment noting the 1.21.10 vs 1.21.11 form. The 26.x
  environment-attribute form is gone.
- **B3-G08 (1.21.7) — FIXED.** `burns_in_day` includes Drowned
  (`mobs/species.rs:1576`); Drowned inherits `Zombie.aiStep` +
  `isSunSensitive() == true` (0 occurrences in the Drowned dump — it
  inherits, it does not override).
- **B3-G09 (1.21.7, 26.1.2) — FIXED.** `sun_burn_tick`
  (`mobs/ai.rs:2036`) includes `m.in_powder_snow` in `wet`. Residual
  (not minted): `wasInPowderSnow` has no previous-tick field, so a mob
  burns one tick early after leaving powder snow.
- **B3-G12 (1.21.7) — WITHDRAWN as framed.** Vanilla 1.21.10
  `EnderMan.getWalkTargetValue` IS overridden, returning 0.0f (changed
  since 1.21.7, where it had 0 occurrences); the port's spawn-table Zero
  is correct. (The AI-path check is separate — see G47.)
- **B3-G16 (1.21.7) — FIXED.** The port has hardcoded
  `villager_trades.rs` (1,313 lines): all 13 trade professions × levels
  1–5, wandering trader 3 groups (2/2/5 = 9 offers); `merchant.rs` falls
  back to listings when no trade set exists.
- **B3-G17 (1.21.7) — WITHDRAWN.** Vanilla 1.21.10 `Goal` HAS
  `reducedTickDelay` = `positiveCeilDiv(t,2)` and `adjustedTickDelay`;
  the port's `reduced(ticks)=(ticks+1)/2` + `adjusted`
  (`mobs/goals.rs:66-71`) matches. Vanilla changed since 1.21.7.
- **B3-G36 (26.1.2) — FIXED.** `can_lie_on` (`mobs/tame.rs:1390`) uses
  `#beds` + empty-above; `can_sit_on` (`mobs/tame.rs:1430`) uses chest /
  lit furnace / bed foot — both match vanilla `CatLieOnBedGoal` /
  `CatSitOnBlockGoal` (verified: BEDS / CHEST / FURNACE / BEDS, no tag
  gate).
- **B3-G42 (26.1.2) — NEVER EXISTED.** Vanilla 1.21.10
  `Mob.setItemSlotAndDropWhenKilled` = setItemSlot + setGuaranteedDrop
  (2.0) + `persistenceRequired = true` (verified in the Mob dump). So
  vanilla's non-barter arm = persistent + 2.0 drop, barter arm =
  setItemSlot + setGuaranteedDrop = 2.0, no persistence — exactly what
  the port does (`mobs/piglin.rs:31-33`). (Note: the 26.1.2 run's G42
  evidence — "NO persistence is set either way" — contradicts the 26.1
  correction; for 1.21.10 the port matches vanilla exactly.)
- **B3-G45 (26.1.2) — WITHDRAWN.** Vanilla 1.21.10 has no kinetic
  weapons at all (0 occurrences of `recentKineticEnemies` /
  `KineticWeapon` / `iron_spear` in `LivingEntity`; 0 items with the
  `kinetic_weapon` component in datagen `items.json`). The port's
  `mobs/spear_use.rs` stab path is gated on `main_hand_kinetic(m)` and
  is unreachable dead code — the double-push it contains can never fire.
- **B3-O02 (1.21.7) — WITHDRAWN as framed.** The `client.multiplayer`
  package EXISTS in 1.21.10's client jar (123 entries); server-only
  failures on it are expected, not breakage. The real failures of four
  of the five oracles are triaged under the 26.1.2 groups (O02, O03,
  O05); `VanillaZombieCureSoundOracle` passes client+both.
- **Cat gift-chance logic matches vanilla** (not a defect):
  `CatRelaxOnOwnerGoal.stop` gives the gift iff sleepTimer ≥ 100 AND
  timeOfDay in (0.77, 0.8) AND nextFloat < 0.7; the port's window (day
  23667–362) correctly inverts that window through the celestial-angle
  warp, and `DimensionFacts::overworld()` hardcodes the
  `"minecraft:day"` timeline so the gate works.
- **Drowned equipment matches** (not a defect): vanilla nextFloat >
  0.9, nextInt(16) < 10 → trident else fishing rod; port
  `spawning.rs` matches.
- **Piglin finalize matches** (not a defect): reason != Structure →
  float < 0.2 → baby, else adult gets `createSpawnWeapon` (crossbow
  0.5 / golden sword); the port's `finalize.rs:1624-1644` matches,
  with a comment noting the 1-in-10 golden spear is 1.21.11-only.
- **`handleInteract` spectator form matches** (not a defect — see B8):
  vanilla 1.21.10 has no spectator early-return before
  `resetLastActionTime` (unlike 26.1.2's `handleAttack`); the port
  resets the timer for spectator attacks too and sets the camera
  instead of attacking (`game/lan_actions.rs:3436-3441`).

## Recorded, not minted

- Dead 26.x-only `Species` arms with no vanilla 1.21.10 counterpart:
  CamelHusk, Nautilus, ZombieNautilus, Parched, SulfurCube (5 variants;
  `food_tag`, `allowed_in_peaceful`, `burns_in_day`, `check_spawn_rules`
  and the baby table all carry arms for them). Mannequin is NOT dead —
  it exists in 1.21.10.
- Port `burn_undead` head-slot protection + 160-tick ignite matches
  vanilla `Zombie.aiStep` / `AbstractSkeleton.aiStep` structure; only
  the ZombieHorse membership (G48) and the body-slot form are wrong.
- Port `walk_target` spawn table (`mobs/spawn_selection.rs:145`) is
  correct for 1.21.10 (Enderman arm now correct — see G12).
- `guest_attack` lacks vanilla's invalid-entity disconnect (item entity
  / XP orb / self / non-attackable arrow → disconnect + warn) — inherited
  26.3 form, reachable only with a modified client (vanilla's client
  never sends such packets); not minted, same as 26.1.2.
- `wasInPowderSnow` one-tick-early burn (see G09 above).
- Trial-spawner `spawnsInPeaceful` handling matches; spawner-block
  `getRandomSpawnMobAt` peaceful omission is spawner-only.

## Re-check verdicts (1.21.7 PRIMARY items — all 26)

| Item | Verdict | Item | Verdict |
|------|---------|------|---------|
| B3-G01 | REPRODUCES | B3-G15 | REPRODUCES |
| B3-G02 | FIXED | B3-G16 | FIXED |
| B3-G03 | REPRODUCES | B3-G17 | WITHDRAWN |
| B3-G04 | REPRODUCES | B3-G18 | REPRODUCES |
| B3-G05 | REPRODUCES | B3-G19 | REPRODUCES |
| B3-G06 | REPRODUCES | B3-G20 | REPRODUCES |
| B3-G08 | FIXED | B3-G21 | REPRODUCES |
| B3-G09 | FIXED | B3-O01 | REPRODUCES (→ O09) |
| B3-G10 | REPRODUCES | B3-O02 | WITHDRAWN |
| B3-G11 | REPRODUCES | B3-O03 | REPRODUCES (→ O10) |
| B3-G12 | WITHDRAWN | B3-O04 | REPRODUCES (→ O11) |
| B3-G13 | REPRODUCES | B3-O05 | REPRODUCES (→ O05) |
| B3-G14 | REPRODUCES (major) | B3-O06 | REPRODUCES (→ O03) |

19 reproduce, 4 fixed, 3 withdrawn, 0 never-existed, 0 unverified.
(The 1.21.7 oracle groups re-mint under the 26.1.2 grouping's numbering
where they overlap — O05 into O05, O06 into O03 — and as new groups
O09–O11 where they don't.)

## Re-check verdicts (26.1.2 SECONDARY items — all 46)

| Item | Verdict | Item | Verdict |
|------|---------|------|---------|
| B3-G03 | REPRODUCES | B3-G30 | REPRODUCES |
| B3-G04 | REPRODUCES | B3-G31 | REPRODUCES |
| B3-G05 | REPRODUCES | B3-G32 | REPRODUCES |
| B3-G06 | REPRODUCES | B3-G33 | REPRODUCES |
| B3-G09 | FIXED | B3-G34 | REPRODUCES |
| B3-G11 | REPRODUCES | B3-G35 | REPRODUCES |
| B3-G13 | REPRODUCES | B3-G36 | FIXED |
| B3-G14 | REPRODUCES (major) | B3-G37 | REPRODUCES |
| B3-G15 | REPRODUCES | B3-G38 | REPRODUCES |
| B3-G18 | REPRODUCES | B3-G39 | REPRODUCES |
| B3-G19 | REPRODUCES | B3-G40 | REPRODUCES |
| B3-G20 | REPRODUCES | B3-G41 | REPRODUCES |
| B3-G22 | REPRODUCES (major) | B3-G42 | NEVER EXISTED |
| B3-G23 | REPRODUCES | B3-G43 | REPRODUCES |
| B3-G24 | REPRODUCES | B3-G44 | REPRODUCES |
| B3-G25 | REPRODUCES | B3-G45 | WITHDRAWN |
| B3-G26 | REPRODUCES | B3-G46 | REPRODUCES |
| B3-G27 | REPRODUCES | B3-G47 | REPRODUCES |
| B3-G28 | REPRODUCES | B3-O01–O08 | REPRODUCE (8 groups) |
| B3-G29 | REPRODUCES | | |

42 reproduce, 2 fixed, 1 never-existed, 1 withdrawn, 0 unverified.

## Passes

### A1 — Entity registry / spawn placements (1 check, 1 pass)

153 `EntityType` registrations in `EntityType.<clinit>` (parsed fresh);
79 `SpawnPlacements.register` calls (counted fresh in the dump). The
port's `check_spawn_rules` covers 78 of the 79 explicitly (the missing
one is ENDER_DRAGON, handled in the dragon module and never naturally
spawned); the `_ => true` default matches vanilla's unregistered → true.
PASS. (No prepared `mob-data.json` exists for 1.21.10, so the placement
types/heightmaps sub-check from 26.1.2 could not run — noted, not
assumed.)

### A1b — Spawn rules (11 checks, 9 pass, 2 gap)

Per-type spawn rules match vanilla for slime (G02 fixed — moon-phase
draws), drowned/guardian/ghast/magma-cube/zombified-piglin light and
height rules, endermite/silverfish, blaze/breeze/zoglin, pillager
(light ≤ 8), cat/ocelot, wolf/fox, parrot, and the
`isValidPositionForMob` block table. GAPs: G01 (bat Halloween branch
missing) and G29 (peaceful gate moved — only phantom/shulker diverge).

### A2 — Species coverage (1 check, 1 pass)

90 `Species` variants = 90 `def!` entries in `mobs/species.rs`. 85 are
live 1.21.10 types; 5 are dead 26.x-only arms (CamelHusk, Nautilus,
Parched, SulfurCube, ZombieNautilus — recorded, not minted). The two
vanilla living types without a `Species` variant are handled elsewhere:
ArmorStand in `entities.rs`, EnderDragon in `game/creatures.rs` +
`game.rs`; Player is separate; EvokerFangs/LlamaSpit/ShulkerBullet are
projectiles. PASS.

### A3 — Attributes / food / dimensions (10 checks, 5 pass, 5 gap)

Attributes registry 35/35 (jar `Attributes` fields = datagen
`minecraft:attribute` entries, same names incl. `tempt_range` and the
`waypoint_*` pair); `createAttributes` suppliers 70 (fresh class-parser
count); categories exact (8 categories, counts from the fresh
`EntityType` parse); fire-immune 15 (fresh count: AREA_EFFECT_CLOUD,
BLAZE, ENDER_DRAGON, END_CRYSTAL, GHAST, MAGMA_CUBE, SHULKER, STRIDER,
TNT, VEX, WARDEN, WITHER, WITHER_SKELETON, ZOGLIN, ZOMBIFIED_PIGLIN);
the port's loader maps all 16 attribute ids it reads
(`mobs.rs:652-672`) and carries the full `attributes` map through.
Per-species attribute VALUES are not verifiable without prepared
`entity-types.json` (no `.local` for 1.21.10) — excluded from the tally,
not guessed. GAPs: G04 (peaceful table), G05/G27/G28 (food table — one
check, three items), G06 (head rotation), G30 (baby dims), G31 (camel
sitting dims).

### A4 — Sun-burn / protection slots (5 checks, 4 pass, 1 gap)

Burn set exact incl. Drowned (G08 fixed); powder-snow exclusion present
(G09 fixed); head-slot protection + 160-tick ignite match vanilla
`Zombie.aiStep` / `AbstractSkeleton.aiStep`; Husk correctly excluded
(`isSunSensitive() == false`). GAP: G48 (zombie horse burns; vanilla
spares it — NEW).

### B1 — Suffocation / fluid (3 checks, 1 pass, 2 gap)

15 `isSuffocating` calls in `Blocks.txt` (matches the 26.1.1
resolution); the port's suffocation rule is fixed (`_ =>
MotionFullCollision`). GAPs: G22 (startup blocker — the rule is fixed
but the load path still requires the absent tags) and G23 (sleeping
mobs suffocate).

### B2 — Spawning (27 checks, 20 pass, 7 gap)

Spot-verified fresh: `magic_light` exact (`mobs/spawn_selection.rs:
427-434`, the curved `f/(4−3f)` form with ambient); `checkDespawn`
structure matches (`mobs/ai.rs:2298+`, peaceful discard, persistent,
removeWhenFarAway, 128/64 distances); `MeleeAttackGoal` 20-tick gate
confirmed in the fresh dump (`bipush 20`). Carried on the fresh-dump
basis (every vanilla anchor re-read in the 1.21.10 dumps; port files
read fresh): mob categories, `getRandomSpawnMobAt`, `mobsAt`/fortress,
`right_distance`, `valid_position_for_type`, `isValidPositionForMob`,
obstruction table, `max_cluster_size`, `isMaxGroupSizeReached`,
`removeWhenFarAway`, `isSpawnPositionOk`, `getSpawnAABB` /
`spawnDimensionsScale` / `canSummon` / `EntitySpawnReason`,
`SpawnState`/`createState`, `PotentialCalculator`,
`getFilteredSpawningCategories`/local cap, spawn-rules tables. GAPs:
G03 (immune tags), G10 (far spawns), G11 (y-range), G13 (RNG draw),
G32 (heightmap tags), G37 (inhabited time), G38 (fresh dimensions).

### B7 — Spawners (4 checks, 3 pass, 1 gap)

Spawner defaults exact (vanilla `BaseSpawner` ctor, fresh dump: delay
20, min 200, max 800, spawn count 4, max nearby 6, range 16, spawn
range 4; port `chunk/block_entity.rs:604-619` matches); `delay()`
matches (`mobs/spawner.rs:449-450`, max≤min→min); trial config defaults
exact (vanilla `TrialSpawnerConfig$Builder`, fresh dump: spawn_range 4,
total 6.0, simultaneous 2.0, +2.0/+1.0 per player, 40 ticks; port
`chunk/block_entity.rs:638-656` matches). GAP: G39 (spawn-egg entity
data applied).

### B8 — Combat / LAN (4 checks, 2 pass, 2 gap)

`CombatRules` methods confirmed in the fresh dump; port
`combat_rules.rs` matches. **NEW PASS — `handleInteract` spectator
form.** Vanilla 1.21.10 has NO separate `handleAttack` and NO spectator
early-return: `handleInteract` (`ServerGamePacketListenerImpl.txt:
4962-5019`) is `if (!hasClientLoaded()) return;` → getTarget →
`resetLastActionTime()` (unconditional) → world-border check →
`canInteractWithEntity(AABB, 3.0)` → dispatch. The port matches: the hub
resets `last_action_tick` for spectator attacks too (`lan/hub.rs:4040-
4060`, `S::Interact` falls in `_ => true`) and `guest_attack`
(`game/lan_actions.rs:3436-3441`) sets the camera for spectators
instead of attacking — the 1.21.10 form, NOT the 26.1.2 delta's
early-return. GAPs: G41 (slime team check), G49 (idle-timer gate order
— NEW). Not minted: `guest_attack` lacks vanilla's invalid-entity
disconnect — reachable only with a modified client. (G45 retired: no
kinetic weapons in 1.21.10.)

### B5 — Projectiles (4 checks, 3 pass, 1 gap)

`AbstractHurtingProjectile` getInertia 0.95, getLiquidInertia 0.8
(fresh dumps; port `game/remote_projectiles.rs:970` matches);
`ThrownTrident.getWaterInertia` 0.99 (fresh dump; port
`game/trident.rs:334-335` matches); throwable drag 0.99 (port
`game/remote_projectiles.rs:948-950`). GAP: G46 (item-entity bounce
restitution form).

### B4 — Trading (6 checks, 4 pass, 2 gap)

`LEVEL_XP` [0,10,70,150,250] (`merchant.rs:23`); `cost_a_count` exact
(`:171-179`, int product, floor, max(0), clamp); `update_demand` exact
(`:192-194`); piglin offhand matches vanilla (G42 never-existed).
GAPs: G24 (level-up immediate), G34 (hurt gating).

### B6 — Vehicles (3 checks, 2 pass, 1 gap)

Boat gravity 0.04 in `tick_boat` (`game/boats.rs:822, 840`); the
minecart step cap correctly uses `max_speed_new` without the furnace
factor (`game/minecarts.rs:1069` — vanilla calls the base form there).
GAP: G25 (furnace minecart speed, experiment-gated).

### B3 — Mob AI (12 checks, 4 pass, 8 gap)

PASS: `reduced()` (G17 not-a-defect — `Goal.reducedTickDelay` /
`adjustedTickDelay` match `mobs/goals.rs:66-71`); the spawn-path
walk-target table (`mobs/spawn_selection.rs:145` — Enderman/Creaking/
Pillager/Warden → Zero, Giant → +cost, Mooshroom → mycelium, all
correct); silverfish walk-target (10.0 on infested host else −cost —
the AI path matches vanilla's `Silverfish.getWalkTargetValue`); cat
lie-on/sit-on goals (G36 fixed). GAPs (8 checks, 9 items — the
enderman/cat check yields two): G14 (enderman teleport) + G15 (cat
morning gift), G26 (BreedGoal panic), G33 (worn heads), G35
(BreathAirGoal), G40 (armadillo ball-up), G43 (fox chorus-fruit 2017),
G44 (sleeping position), G47 (AI walk-target value).

### B9 — Finalize / equipment (3 checks, 2 pass, 1 gap)

Piglin finalize `float() < 0.2` at `mobs/finalize.rs:1624-1644`
(matches vanilla, incl. the crossbow/golden-sword weapon split and the
1.21.11-only spear note); drowned equipment verified fresh (vanilla:
`nextFloat > 0.9`, `nextInt(16)`, <10 → TRIDENT else FISHING_ROD; port
`spawning.rs` matches). GAP: G21 (zombie spear — the `spawning.rs` arm
still uses nextInt(6) with `iron_spear`).

## Oracle defects

**Basis (provisional):** the 1.21.10 Batch 0 table had NOT landed when
this batch finished (checked 2026-11-25; `docs/sweep/1.21.10/` holds
only inventory files + `merge.py`). Oracle triage therefore uses the
26.1.2 Batch 0 table
(`sweep-1.21.10-ref/26.1.2/batch-0-oracles.json`, 391 files, generated
by fixpoint big-compile per classpath against the named 26.1.2 jars),
per the brief — PLUS the 1.21.7 oracle groups, which matter here because
1.21.10 shares 1.21.7's missing packages. The basis is sound but NOT a
carry-over: 1.21.10's API surface differs from 26.1.2's in ways that
change verdicts (no `gamerules` / `world.clock` packages, flatter entity
packages, `LocalPlayer` 7-arg / `MinecraftServer` 8-arg ctors vs 8/10
in 26.1.2, `BlockTags` 203 fields vs 248), so every anchor below was
re-verified fresh against the 1.21.10 jars and the 1.21.10 oracle
sources (357 `Vanilla*Oracle.java` files; all Batch 3-scope sources
present). The coordinator should re-verify these 11 groups against the
1.21.10 table when it lands; no re-verification is expected to change
any group verdict, though n-values will differ.

- **B3-O01 — missing-override (3 oracles, n=1 each).** `Level.fuelValues()`
  is `public abstract` in 1.21.10 (fresh dump `Level.txt:1995`); the
  oracle level stubs lack the override (grep-confirmed absent in the
  1.21.10 tree sources: `VanillaActiveEffectTickOracle`,
  `VanillaAttributeUpdatesOracle`, `VanillaLivingLandingOracle`).
  The `Level` ctor is 8-arg in 1.21.10, matching the stubs, so the
  fuelValues error is the only one. Reproduces from 26.1.2.
- **B3-O02 — constructor-mismatch (5 oracles).** `LocalPlayer` ctor takes
  7 args in 1.21.10 (fresh dump from the client jar);
  `MinecraftServer` ctor takes 8 args (fresh dump). Oracle sources
  construct with the 26.3 form (e.g. `VanillaAllayInteractionStateOracle`
  has `Actor extends LocalPlayer`; `VanillaEntitySoundPacketOracle`
  constructs `MinecraftServer`). Reproduces from 26.1.2 (different
  arities than 26.1.2's 8/10).
- **B3-O03 — no-override (14 oracles).** 26.3 methods absent in 1.21.10;
  n_errors per the 26.1.2 table: Arrow 1, BlockingItemDamage 2,
  BoatRider 4, ClientArrow 1, DamageImmunity 2, EntityDeathLoot 4,
  FinalizeSpawn 7, LateEntityPair 4, MobAttributes 1, NaturalSpawn 16,
  PlayerTravel 1, RegisteredPlayerCombat 6, ServerPlayerCombat 6,
  SpawnRules 1. (Includes 1.21.7's O06 content: MobAttributes,
  BlockingItemDamage and LivingFall — LivingFall confirmed fresh, since
  1.21.10 lacks `resetCurrentImpulseContext` /
  `tryResetCurrentImpulseContext` / `propagateFallToPassengers` —
  reproduce; EffectNotifications' `MobEffect` overrides resolve in
  1.21.10 and it is not counted here.) Reproduces from 26.1.2.
- **B3-O04 — `EntityTypes` holder move (9 oracles, incl. transcripts
  EntityTranscript n=7, RideTranscript n=4).** All 9 sources reference
  `EntityTypes` (grep-verified in the 1.21.10 tree; 33 files tree-wide);
  `EntityTypes` has 0 entries in either 1.21.10 jar (1.21.10 predates
  the holder move). Reproduces from 26.1.2.
- **B3-O05 — missing-symbol (16 oracles).** Anchors verified fresh:
  `BlockTags` has 203 fields, none of CAUSES_SUFFOCATION /
  SUPPRESSES_BOUNCE / BLOCKS_FLUID_FLOW; `ClientboundUpdateAdvancements-
  Packet` is a plain class (no Record attribute) with private `added` /
  `removed` / `progress` fields and only `getAdded()` / `getRemoved()` /
  `getProgress()` accessors (no record-style accessors) —
  LocatorBarClient n=13 reproduces; `Enderman` (26.3 name) absent,
  `EnderMan` present; `densityfunction` package 0;
  `ServerboundPunchPacket` 0; `EntitySpawnRequest` / `NoiseSpawnFinder` /
  `MoveSimulationType` 0. (Includes 1.21.7's O05 content except
  VanillaTradeComponentsOracle, VanillaVillagerFarmingOracle and
  VanillaWorldSpawnPacketOracle, which pass in 26.1.2 and whose key
  symbols resolve in 1.21.10 — e.g. `LevelData$RespawnData.of` exists —
  so their 1.21.7 errors are provisional pending the Batch 0 table.)
  Reproduces from 26.1.2.
- **B3-O06 — missing-package (2 oracles).** Advancements `triggers` /
  `predicates` packages absent (only `critereon`, 164 entries —
  verified fresh). Reproduces from 26.1.2.
- **B3-O07 — cascade (2 oracles).** Depend on broken
  VanillaFaceSupportOracle / VanillaBlockUpdateOracle. Reproduces from
  26.1.2.
- **B3-O08 — `VanillaEntityFluidOracle` (1 oracle, n=1).** "'void' type
  not allowed here" at line 74. Reproduces from 26.1.2.
- **B3-O09 — `gamerules` package (6 oracles). NEW numbering; 1.21.7's
  O01.** `net.minecraft.world.level.gamerules` has 0 entries in either
  1.21.10 jar (GameRules lives in `net.minecraft.world.level` here);
  6 oracle sources still reference it (grep-verified):
  VanillaAttributeHealthOracle, VanillaDamageImmunityOracle,
  VanillaEntityDeathLootOracle, VanillaRaidOracle,
  VanillaRegisteredPlayerCombatOracle, VanillaServerPlayerCombatOracle.
  (VanillaNaturalSpawnOracle, the 7th in 1.21.7's group, no longer
  references it in the 1.21.10 sources; its failure is the no-override
  class, O03.) Reproduces from 1.21.7.
- **B3-O10 — 26.x entity subpackages (6 oracles). NEW numbering; 1.21.7's
  O03.** `projectile/arrow`, `animal/cow`, `vehicle/minecart`,
  `vehicle/boat`, `monster/zombie` all have 0 entries in both 1.21.10
  jars (the classes live flat: `projectile/Arrow`, `animal/Cow`,
  `vehicle/AbstractMinecart`, `vehicle/Boat`, `monster/Zombie`):
  VanillaArrowOracle, VanillaLateEntityPairOracle,
  VanillaMobBreathingOracle, VanillaEntitySoundPacketOracle,
  VanillaFluidBoxOracle, VanillaEffectSyncOracle. (VanillaSpawnRulesOracle,
  the 7th in 1.21.7's group, references `util/debug`, which EXISTS in
  1.21.10 (34 entries) — its failure is the no-override class, O03.)
  Reproduces from 1.21.7.
- **B3-O11 — `world.clock` package (2 oracles). NEW numbering; 1.21.7's
  O04.** `net.minecraft.world.clock` has 0 entries in either 1.21.10 jar:
  VanillaAttributeUpdatesOracle, VanillaActiveEffectTickOracle (both
  also carry the O01 fuelValues error). Reproduces from 1.21.7.

**Passing Batch 3-scope oracles** (23 pass all three classpaths in the
26.1.2 table; `VanillaZombieCureSoundOracle` passes client+both and fails
server-only on `client.multiplayer` — expected, not breakage; in 1.21.10
the package exists in the client jar, so the same pattern holds):
VanillaAttributeHealthOracle*, VanillaEffectNotificationsOracle,
VanillaEffectSyncOracle*, VanillaFluidBoxOracle*, VanillaLivingFallOracle*,
VanillaTradeComponentsOracle, VanillaVillagerFarmingOracle,
VanillaWorldSpawnPacketOracle, VanillaCombatRulesOracle,
VanillaEntityAttributeOracle, VanillaEntityObstructionOracle,
VanillaDamageSourceOracle, VanillaFallDamageOracle, VanillaSwimmingOracle,
VanillaLivingDamageOracle, VanillaLivingHealthOracle, VanillaMobAirOracle,
VanillaMobDiscardOracle, VanillaRaiderCelebrationOracle,
VanillaServerPlayerDamageOracle, VanillaPlayerEquipmentOracle,
VanillaPlayerFoodOracle, VanillaPlayerHealthOracle. (* = passes in the
26.1.2 table but FAILS in 1.21.10 on a 1.21.x-specific ground:
AttributeHealth on O09 gamerules; EffectSync and FluidBox on O10
subpackages; LivingFall on O03 no-override. The 1.21.10 Batch 0 table
will settle the exact split.)

## Drivers (§D)

144 drivers were curated to 135 core (see common.md). Batch 3's driver-
code defects are G18, G19, G20 (3 checks, 0 pass, 3 gap — counted
as-tallied per the 26.2 precedent, split reported in the Tally). No driver
was run (checks + triage only). Tree-wide driver-code defect counts
(re-counted fresh, listed in the slice's `out_of_scope_failures`):
36 drivers hardcode the 26.3 state count 35723 (2 in Batch 3 scope, 34
others); 45 drivers use a 26.3 resources path (5 in Batch 3 scope, 40
others — the narrow `horizon-client/26.3` pattern alone; the broad
all-`'26.3'`-path-forms count was not separately tallied for 1.21.10).
Full lists in `evidence/drivers-35723.txt` and
`evidence/drivers-263-paths.txt`.

## Method notes

- **Vanilla-check rule:** every vanilla anchor was verified against
  1.21.10's own jars (server-game for entity logic, client for data/tags/
  advancements), never the 26.3 base's form and never carried from
  26.1.2 or 1.21.7. The server-game jar's bundler wrapper (`server.jar`)
  was never javap'd.
- **No prepared data:** no 1.21.10 `.local` exists, so the A3 attribute
  values check and the A1 placement-types sub-check could not run; both
  are recorded as not-verifiable-here, not passed by assumption. All
  other data checks rest on the jars + the datagen reports (1,488
  items, 153 entity types, 35 attributes, 29,671 states).
- **Class-parser extractions (JVM-free):** `evidence/baby_dims2-1.21.10.py`
  (adapted from the 26.1.2 script) for the 13 `BABY_DIMENSIONS`
  declarers; inline scans for the `getDimensions` / `getScale` /
  `getDefaultDimensions` / `getAgeScale` override sets, the 70
  `createAttributes` suppliers, the 15 fire-immune types, and all 153
  adult dimensions + eye heights (`evidence/adult-dims-1.21.10.txt`).
  `evidence/parse_entitytype.py` for categories / notInPeaceful /
  canSpawnFarFromPlayer / immuneTo.
- **Vanilla changed since 1.21.7 in two places this batch caught:**
  `EnderMan.getWalkTargetValue` is now overridden (0.0f) and `Goal` now
  has `reducedTickDelay` / `adjustedTickDelay` — both re-checks
  withdrawn, not reproduced. The port matched vanilla in both cases.
- **The 1.21.10 port is genuinely adapted, not a copy:** the baby-dims
  table (Drowned/ZombieVillager inheritance comments), the piglin
  finalize (1.21.11-only spear note), the cat goals, the slime
  moon-phase draws, the drowned burn, the suffocation rule, the
  `handleInteract` form — each verified fresh. The remaining defects
  are the un-adapted residue, not a wholesale 26.x carry-over.
- **Oracle triage:** provisional from the 26.1.2 Batch 0 table +
  the 1.21.7 oracle groups (1.21.10 shares 1.21.7's missing packages),
  all anchors re-verified fresh. Re-verify against
  `docs/sweep/1.21.10/batch-0-oracles.json` when it lands.
- **Evidence:** `b03/evidence/` — `run-javap.sh`, `run-javap2.sh`,
  `javap/` (88 dumps incl. `BlockProperties-v.txt`,
  `Heightmap-v.txt`, `HeightmapTypes.txt`, `FeatureFlags.txt`,
  `BaseSpawner.txt`, `TrialSpawnerConfig.txt`,
  `TrialSpawnerConfigBuilder.txt`), `baby_dims2-1.21.10.py`,
  `baby-dims-1.21.10.txt`, `adult-dims-1.21.10.txt`,
  `entity-types-1.21.10.txt`, `port-baby-table.txt`,
  `port-species.txt`, `oracle-table-b3.txt`, `drivers-35723.txt`,
  `drivers-263-paths.txt`, `server-game-listing.txt`,
  `client-listing.txt`, `tag-counts.txt`, `class-paths.txt`,
  `parse_entitytype.py`.

## Verdict

**97 checks as-tallied: 61 pass, 36 gap** (94 behaviour: 61 pass, 33 gap;
3 driver-code: 0 pass, 3 gap). **50 items: 39 G (36 port + 3 driver) +
11 O-groups; 2 major, 48 minor.** Of the 72 re-checks, 61 reproduce
(19 of 26 1.21.7 PRIMARY, 42 of 46 26.1.2 SECONDARY), 6 are fixed, 1
never existed, 4 withdrawn, 0 unverified. Two new defects (B3-G48,
zombie-horse daylight burning; B3-G49, idle-timer gate order) and one
new passing check (B8 `handleInteract` — the port carries the correct
1.21.10 form, not the 26.1.2 delta's early-return). The two majors both
reproduce: **B3-G14 endermen never teleport (major)** and **B3-G22 the
suffocation startup blocker (major)** — the 1.21.10 client cannot load
its core tables with prepared 1.21.10 data. Oracle triage is provisional
from the 26.1.2 Batch 0 table + the 1.21.7 oracle groups (the 1.21.10
table had not landed); re-verify when it does.
