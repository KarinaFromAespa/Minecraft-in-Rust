# Batch 3 — Entities / mobs (26.2 parity sweep)

**Version:** 26.2 · **Date:** 2026-11-24 · **Branch:** `parity-sweep-26.2`
(workdir `~/workspace/mcir-work/sweep-26.2`, tip `9b2bad5e` at delivery; the
worktree advanced from `e30ceac7` during this run when Batch 7 committed —
`versions/26.2/` content unchanged by that commit) · **Scope:**
`docs/PARITY-SWEEP.md` §4 Batch 3 — mob AI, spawning, trading, attributes,
projectiles, vehicles; 158 entity types (86 living + 72 misc); 167 drivers
curated to 157 core (see §D).

**Status: committed on branch `parity-sweep-26.2` (coordinator commit).** Checks + triage only, per the brief: no
fixes, no edits under `versions/26.2/`, no cargo builds (another agent shares
the 6 GB VM), no driver runs, no oracle repairs, no commits — the coordinator
commits these deliverables. Everything below is static analysis: `javap -c`
on the named 26.2 server-game jar
(`~/workspace/mcir-work/26.1-adapters/versions/26.2/jars/server-game.jar`,
DataVersion 4903, protocol 776), source comparison against
`versions/26.2/crates/`, and checks of the prepared 26.2 data
(`versions/26.2/.local/horizon-client/26.2/`).

**Prepare landed during this run** (coordinator update 09:11 NPT Nov 24):
horizon-prepare finished in 2048.7 s, EXIT 0, 203 MB, so
`entity-types.json`, `attributes.json`, `block-behaviours.json` and
`gameplay-facts.json` (51 MB) were available for the data cross-checks below.
`sounds.json` and lang files are still absent (the VM lacks the official
launcher's asset index 32.json) — expected, not a port defect.

## Tally

- **Behaviour checks: 78 — 72 pass, 6 gap** (6 defect items B3-G05, B3-G06,
  B3-G09, B3-G11, B3-G19, B3-G20; every other 1.21.7 Batch 3 defect was
  re-checked and does NOT carry over — see "Re-checked, not defects").
- **Oracle defects: 5 groups** (B3-O01–B3-O05, 30 oracles — not counted in
  the check tally).
- **Oracle repairs done: none** (checks + triage only).
- **Severity: 0 major, 4 minor port** (B3-G05, G06, G09, G11),
  **2 minor driver-side** (B3-G19, G20). All three 1.21.7 majors (G14
  enderman teleport, G16 no trades, G17 `reduced()`) do NOT carry over:
  26.2 is a 26.x version, so the 26.x systems those defects turned on
  (tag-grounded teleport, data-driven trades, `reducedTickDelay`) exist in
  vanilla 26.2 and the port matches them. Several other 1.21.7 "defects"
  turned out to be genuine 1.21.7↔26.2 vanilla differences (G02 environment
  attributes, G04 `notInPeaceful`, G08 tag-driven burn, G10 Builder default,
  G12 Enderman override, G17 `reducedTickDelay`, G21 iron spear).

## Defects

### B3-G05 — Camel has no food (minor)

Vanilla 26.2 `Camel.isFood` = `#camel_food` (cactus; the tag exists in the
jar's data and in prepared `block-tags.json`). Port `food_tag()`
(`horizon-client/src/mobs/species.rs:1633`) covers `CamelHusk` but NOT
`Camel`; `camel_food` appears nowhere in the port, so `MobType.food` is
empty for camels and feeding/breeding/tempting never fire. Camels cannot
breed. (Carried over unchanged from 1.21.7.)

### B3-G06 — Armadillo scared head rotation ignored (minor)

Vanilla 26.2 `Armadillo.getMaxHeadYRot = isScared() ? 0 : 32`. Port
`max_head_y_rot` (`mobs/species.rs:1712`) returns 32.0 unconditionally,
although the port HAS `armadillo::scared(m)` state
(`mobs/armadillo.rs:160`). A scared armadillo's head should lock (0° yaw).
(Carried over unchanged from 1.21.7.)

### B3-G09 — Sun-burn ignores powder snow (minor)

Vanilla 26.2 `Mob.isSunBurnTick` excludes `isInWaterOrRain ||
isInPowderSnow || wasInPowderSnow`. Port `sun_burn_tick`
(`mobs/ai.rs:2033`): `wet = in_water || in_rain` only. `Mob` HAS
`in_powder_snow` state (`mobs.rs:1617`, updated `ai.rs:1413-1414`) but it
is not consulted. A skeleton standing in powder snow burns in the port, not
in vanilla. (Carried over unchanged from 1.21.7.)

### B3-G11 — `getRandomPosWithin` y-range off by one (minor)

Vanilla 26.2 `NaturalSpawner.getRandomPosWithin`: top =
`getHeight(WORLD_SURFACE)+1`, y = `randomBetweenInclusive(minY, top)` →
[minY, H+1]. Port `spawn_category_for_chunk`
(`mobs/spawn_selection.rs:1117`): top = `heightmap(WorldSurface)` = H (the
port's `world_surface` is "one above the highest non-air block",
`horizon-client-render/src/world.rs:407`), y = min +
`nextInt(top-min+1)` → [minY, H]. The port never picks H+1 and consumes a
different `nextInt` bound, so the RNG stream diverges from vanilla for
every spawn attempt. (Carried over unchanged from 1.21.7.)

### B3-G19 — DRIVER: `test_registered_entity_fluid_parity.py` hardcodes the 26.3 state count (minor)

Line 66: `len(facts['states']) != 35723` → AssertionError. 26.2 has 32,366
block states, so the driver always fails. Note the 1.21.7 instance of this
defect (`test_suffocation_parity.py`) is FIXED in 26.2: its line 68 checks
32366, correct for 26.2. The remaining 35723 hardcodes in the 26.2 tree
belong to other batches (see `out_of_scope_failures` in the JSON slice);
the Batch 3 one is `test_registered_entity_fluid_parity.py` only.

### B3-G20 — DRIVER: five Batch 3 drivers hardcode the 26.3 resources path (minor)

`test_server_dragon_parity.py:111`,
`test_locator_bar_parity.py:269`, `test_projectile_shoot.py:46`,
`test_server_entities_parity.py:186`,
`test_server_knockback_parity.py:435` all build the Rust server command
with `lab.LOCAL / 'horizon-client/26.3'`; 26.2 resources live at
`horizon-client/26.2`. `lab.RESOURCES = LOCAL / "horizon-client" /
VERSION` is version-parameterised — the drivers should use it. (The
1.21.7 instance listed two drivers; the 26.2 tree has three more in Batch 3
scope. Further 26.3-path hardcodes in other batches' drivers are listed in
the JSON slice's `out_of_scope_failures`.)

### B3-G07 — WITHDRAWN in 1.21.7 (happy-ghast tempt)

Assigned and withdrawn in the 1.21.7 batch after verification (happy-ghast
tempt matches vanilla exactly). The number is not reused here.

## Re-checked, not defects in 26.2

Every 1.21.7 Batch 3 defect not listed above was re-checked against
vanilla 26.2's own jar/data and does NOT carry over. The deciding evidence:

- **B3-G01 bat Halloween:** 26.2 `Bat.checkBatSpawnRules` has NO Halloween
  branch (always coin flip + limit 4); the port matches (bat arm checks
  y ≥ world surface, `nextBoolean`, brightness > `nextInt(4)`,
  `#bats_spawnable_on` below + mob rules). The 1.21.7 defect was a
  1.21.7-only vanilla form.
- **B3-G02 slime surface spawn:** 26.2 `Slime.checkSlimeSpawnRules`
  genuinely uses the environment attribute
  `SURFACE_SLIME_SPAWN_CHANCE` (single `nextFloat` draw); the port
  matches. The underground branch (slime-chunk seed 987234911,
  `nextInt(10)`, y < 40) also matches.
- **B3-G03 immune_tag:** all 6 `*_immune_to` block tags EXIST in the 26.2
  jar + prepared data with correct contents (fox → sweet_berry_bush;
  polar_bear/snow_golem/stray → powder_snow; wither/wither_skeleton →
  wither_rose); `EntityTypes.<clinit>` has exactly the 6 matching
  `immuneTo` calls.
- **B3-G04 hoglin peaceful:** HOGLIN **is** built with `notInPeaceful()`
  in 26.2 vanilla; the port's 38-type disallow list matches vanilla's 38
  `notInPeaceful` types exactly (both `spawn_rules.rs:134` and
  `finalize.rs:897`).
- **B3-G08 drowned burns:** 26.2 burn is tag-driven — `Mob.aiStep` calls
  `burnUndead()` iff the type is in `#burn_in_daylight`; the tag holds
  exactly the 10 types in the port's `burns_in_day` list (skeleton,
  stray, wither_skeleton, bogged, zombie, zombie_horse, zombie_villager,
  drowned, zombie_nautilus, phantom). Drowned defines no `aiStep` /
  `isSunSensitive` (inherits).
- **B3-G10 spawns far from player:** 26.2 `EntityType.Builder` ctor
  default is `canSpawnFarFromPlayer = (category == CREATURE || category
  == MISC)` — a genuine 1.21.7↔26.2 vanilla difference. Port
  `spawns_far_from_player` (`spawn_selection.rs:317`) = Creature|Misc +
  Pillager|Shulker matches exactly (explicit setters in
  `EntityTypes.<clinit>`: only PILLAGER + SHULKER).
- **B3-G12 enderman walk-target:** 26.2 `EnderMan` DOES override
  `getWalkTargetValue` returning 0.0f; the port's Zero mapping is correct.
- **B3-G13 constant-count RNG:** 26.2 `SpawnerData` is still a plain
  record (minCount, maxCount), no IntProvider; port `sample_count`
  (`spawn_selection.rs:798`) now draws unconditionally (`min +
  nextInt(max-min+1)`).
- **B3-G14 enderman teleport (was MAJOR):** 26.2
  `LivingEntity.randomTeleport(DDDZ)` has NO tag parameter;
  `EnderMan.teleport` grounds on `blocksMotion()` (scan down, require
  blocksMotion && fluid not WATER). NO teleport tags exist in the 26.2
  jar data or prepared `block-tags.json` (265 tags). Port `enderman.rs`
  `teleport_to` (line 162) grounds on `blocks_motion`;
  `random_teleport` matches vanilla's scan (blocksMotion below,
  teleportTo, noCollision, !containsAnyLiquid, event 46, nav.stop).
  **Correction to the 1.21.7 report's secondary nuance:** vanilla's first
  teleport sound is NOT at the old position — `teleportTo`→`snapTo`
  calls `setOldPosAndRot()`, moving xo/yo/zo to the new position before
  the sound plays, so both sounds are at the new position; the port's
  comment is right. (`Entity.teleportTo`/`snapTo`/`setOldPosAndRot`
  bytecode-verified.)
- **B3-G15 cat morning gift (was tied to G14):** port `tame.rs`
  `give_morning_gift` (line 1353) uses `enderman::random_teleport`
  (blocksMotion form); no teleport-tag references remain.
- **B3-G16 villagers have no trades (was MAJOR):** the 26.2 jar HAS
  data-driven trades (562 `villager_trade` + 83 `trade_set` entries in
  server-game.jar incl. datapacks); prepared data has 388
  `villager_trade` + 68 `trade_set` files; port `Trades::load`
  (`merchant.rs:467`) reads `villager_trade/`, `tags/villager_trade/`,
  `trade_set/`.
- **B3-G17 `reduced()` (was MAJOR, systemic):** 26.2 vanilla `Goal` HAS
  `reducedTickDelay(int)` = `Mth.positiveCeilDiv(ticks, 2)` and
  `adjustedTickDelay` (identity if `requiresUpdateEveryTick`, else
  reduced); `LookAtPlayerGoal.start` calls `adjustedTickDelay(40 +
  nextInt(40))`. Port `goals.rs:66` `reduced = (ticks+1)/2` matches.
  `Mob.serverAiStep` alternation (`(tickCount + id) % 2` odd &&
  `tickCount > 1` → `tickRunningGoals(false)`, else full `tick()`)
  matches port `ai.rs:2075` exactly. The 1.21.7 defect was a 1.21.7-only
  vanilla form (no `reducedTickDelay` there).
- **B3-G18 entity_components driver:** the 26.2 driver handles the list
  form AND the `minecraft:components` key; 26.2 advancement data uses
  list-of-conditions with `predicate.minecraft:components` (verified
  `complete_catalogue` 11 criteria, `whole_pack` 9;
  `leash_all_frog_variants.json` IS at
  `husbandry/leash_all_frog_variants.json`).
- **B3-G21 zombie spear:** 26.2 vanilla
  `Zombie.populateDefaultEquipmentSlots`: super (Monster armour),
  `nextFloat < (hard ? 0.05 : 0.01)`, `nextInt(6)`: 0 → IRON_SWORD, 1 →
  IRON_SPEAR, else IRON_SHOVEL. The port matches exactly in BOTH
  `spawning.rs:222` and `finalize.rs:1347`. Iron spear is a genuine 26.2
  item and the 1/6 odds are vanilla's.

## Passes (by section)

### A1 — Spawn placements (158): PASS (1 check)

83 vanilla `SpawnPlacements.register` calls (ON_GROUND 60, IN_WATER 12,
NO_RESTRICTIONS 10, IN_LAVA 1; heightmap MOTION_BLOCKING_NO_LEAVES 81,
MOTION_BLOCKING 2: ocelot, parrot) + defaults vs `mob-data.json` 158
entries — **0 mismatches** (placement type, heightmap and predicate name
for every entity). 39 distinct predicates inventoried; the full table is
in the scratch dir (`spawn-placements-vanilla.json`, not a deliverable).
Entity categories parsed from `EntityTypes.<clinit>` (158 types): MONSTER
45, CREATURE 31, MISC 72, WATER_AMBIENT 4, WATER_CREATURE 3, AXOLOTLS 1,
AMBIENT 1, UNDERGROUND_WATER_CREATURE 1 — matches the inventory.
HOGLIN MONSTER, SULFUR_CUBE MONSTER.

### A1b — Spawn predicates: 11 method-by-method checks, all PASS

Bat (G01), slime (G02), camel (`#camels_spawnable_on` + bright;
`bright` = raw_brightness > 8 = `Animal.isBrightEnoughToSpawn`'s
`getRawBrightness(pos, 0) > 8`), sulfur cube (vanilla
`checkSulfurCubeSpawnRules` returns `true` unconditionally; port `=>
true`), `isDarkEnoughToSpawn` (sky > `nextInt(32)`; block-light limit
when < 15; thundering darken 10 else sky_darken;
`monsterSpawnLightTest.sample`), `checkMobSpawnRules` (isSpawner ||
`isValidSpawn` below), `checkMonsterSpawnRules` ((ignoresLight ||
darkEnough) && mobRules), `checkAnimalSpawnRules` ((ignoresLight ||
bright) && tag below), drowned (below-fluid WATER || isSpawner;
difficulty != PEACEFUL; (ignoresLight || darkEnough); (isSpawner ||
fluid at pos WATER); spawner/reinforcement early return; else
`#more_frequent_drowned_spawns` → `nextInt(15)==0`, else `nextInt(40)==0`
&& `isDeepEnoughToSpawn` (y < sea−5) — port matches, including the roll
order), hoglin/piglin (below != `minecraft:nether_wart_block` only — no
light or mob-rules check in vanilla; port matches). Port
`raw_brightness` = max(block_light, sky_light − darkening) = vanilla
`getRawBrightness` form. The remaining 28 predicates are covered by the
A1 placement-table match (predicate names) and were not verified
method-by-method.

### A2 — Species coverage: PASS (1 check)

90 port species defs (`species.rs` ALL); all 90 have attribute suppliers;
suppliers missing from the port are only armor_stand, ender_dragon,
player (handled separately by design). `sulfur_cube`, `camel_husk`,
`parched`, `copper_golem`, `mannequin`, `nautilus`, `zombie_nautilus`
are all LIVE in 26.2 (they were dead 26.x-only types in 1.21.7).

### A3 — Attributes + species tables: 5 pass, 2 gap (G05, G06)

- Attributes **40/40 match** (default/min/max/syncable): jar
  `Attributes.<clinit>` vs `attributes.json`.
- **2,619 entity attribute values identical** between
  `attributes.json[entities]` and `entity-types.json`; 93 suppliers in
  `DefaultAttributes` = 93 entities in `attributes.json`.
- Head rotation: Mob defaults (ambient 80, maxHeadY 75, maxHeadX 40,
  headRotSpeed 10) match the port's defaults; every override matches
  (Frog 35/5, Breeze 25/30, Dolphin/Axolotl 1/1, Slime/MagmaCube/
  SulfurCube maxHeadX 0 via `AbstractCubeMob.getMaxHeadXRot = 0`,
  Guardian/ElderGuardian/Shulker 180 X, Sniffer 50, Goat 15, Ravager 45,
  Camel/CamelHusk 30 — CamelHusk extends Camel; Shulker 180 Y) except
  G06 (armadillo scared).
- Ambient intervals match (Turtle 200, AbstractHorse family incl.
  Camel/CamelHusk 400, Ocelot 900, Guardian/ElderGuardian 160,
  Hoglin/Shulker 120, Allay 80, water animals + golems incl.
  CopperGolem 120, animals 120, default 80).

### A4 — Daylight burning: 4 pass, 1 gap (G09)

`Mob.aiStep` → `burnUndead()` iff `#burn_in_daylight` (G08);
`burnUndead`: alive && `isSunBurnTick` → helmet damage `nextInt(2)`,
break at max damage, else `igniteForSeconds(8.0)`. Port `burn_undead`
(`ai.rs:2018`): `burns_in_day` flag + SunProtectionWear event or ignite
160 ticks. `sunProtectionSlot`: ZombieHorse → BODY, ZombieNautilus →
BODY (javap-verified); port `body_slot_species` (`drops.rs:186`) covers
both. `isSunBurnTick`: MONSTERS_BURN env attribute, magic > 0.5,
`nextFloat*30 < (f−0.4)*2`, wet check, `canSeeSky(eye)` — port matches
except powder snow (G09).

### B1 — Suffocation: PASS (3 checks)

- Override table: exactly 15 `isSuffocating` calls in
  `Blocks.<clinit>`; bootstrap resolution (#5/#16 = `Blocks.never`,
  #130 = `Blocks.always`) gives Always: dirt_path, farmland, mud,
  soul_sand; Never: leaves helper, glass/stained-glass helpers,
  mangrove_roots, moving_piston, cherry/pale_oak leaves, glass,
  tinted_glass, copper grates (via `ofFullCopy`); NotExtendedPiston:
  piston helper; ClosedShulker: `shulkerBoxProperties`. Port
  `mobs/suffocation.rs` `bind()` matches exactly.
- Default rule: `Properties` ctor default `isSuffocating` =
  `lambda$new$5` (blocksMotion + full collision shape) — port
  `MotionFullCollision` matches; no tag involved in 26.2.
- `in_wall` (`mobs/ai.rs:1181`): eye box width*0.8 × 1e-6, suffocating
  shape intersection; Vex excluded (`ai.rs:1269`, noPhysics while it
  ticks) — matches `Entity.isInWall`.

### B2 — Natural spawning: 24 pass, 1 gap (G11)

- MobCategory values (MONSTER 70/false/false/128, CREATURE 10/true/
  true/128, AMBIENT 15/true/false/128, AXOLOTLS 5/true/false/128,
  UNDERGROUND_WATER_CREATURE 5/true/false/128, WATER_CREATURE
  5/true/false/128, WATER_AMBIENT 20/true/false/64, MISC
  −1/true/true/128) — port `Category` (`spawn_selection.rs:108-123`)
  matches exactly.
- G10, G12, G13 (see "Re-checked"); G03 immune_tag and G04 hoglin
  peaceful (see "Re-checked").
- `spawnCategoryForPosition` order: redstone-conductor early return, 3
  packs, pack size `ceil(nextFloat*4)`, walk `nextInt(6)−nextInt(6)`,
  count sample replaces the pack bound, yaw `nextFloat*360` drawn after
  the placement/species checks and before the walk-target/obstruction
  checks (vanilla offsets 337 → 397 → 415), cluster cap, TropicalFish
  break when `!school` (`isMaxGroupSizeReached`) — port
  (`spawn_selection.rs:1104-1240`) matches.
- `getRandomSpawnMobAt`: WATER_AMBIENT + `#reduce_water_ambient_spawns`
  + `nextFloat < 0.98` → empty; else weighted pick from `mobsAt` —
  port matches.
- `valid_position_for_type` (`spawn_selection.rs:1360`):
  Misc → false; !canSpawnFarFromPlayer && distance > despawn² → false;
  canSpawnMobAt (documented reorder — order cannot change the answer);
  `isSpawnPositionOk`; `checkSpawnRules(NATURAL)`; `noCollision(
  getSpawnAABB(x+0.5, y, z+0.5))` — matches
  `isValidSpawnPostitionForType`. `canSummon()` is vacuous in 26.2 (no
  `canSummon` calls in `EntityTypes.<clinit>`), so the port's omission
  is fine.
- `right_distance` (`spawn_selection.rs:1340`): distance > 576 (24²),
  spawn-point 24², chunk check — matches.
- `isValidPositionForMob`: far check (removeWhenFarAway + despawn²) +
  `checkSpawnRules` + `checkSpawnObstruction` — port line 1205
  (`far || !mob_spawn_rules || !spawn_obstruction`) matches.
- `checkSpawnRules` overrides: a class-file scan of all 716 entity
  classes proves ONLY `Mob` (returns true), `PathfinderMob`
  (`getWalkTargetValue >= 0.0F`) and `SpawnPlacements` (the static
  method) declare it — no other Mob subclass overrides it. The port's
  walk-target gate is correct for every naturally-spawning mob.
- Walk-target table: all 18 `getWalkTargetValue` declarers covered
  (PathfinderMob base 0.0; Monster −cost; Animal grass→10 else cost;
  Mooshroom mycelium→10 else cost; Guardian water→10+cost else −cost;
  Silverfish infested-host→10 else −cost; Giant +cost; Turtle, Strider,
  Hoglin, Bee, HappyGhast arms; Zero: Enderman, Creaking, Pillager,
  Warden, Axolotl, AbstractNautilus; NotPathfinder 0.0: Bat, Slime,
  MagmaCube, SulfurCube, Ghast, Phantom; default Zero for water
  animals, golems, villagers, allays). `cost_from_light` = magic_light
  − 0.5 = `getPathfindingCostFromLightLevels`.
- Obstruction table: all 13 `checkSpawnObstruction` declarers covered
  (WaterAnimal/AgeableWaterCreature/Axolotl/Drowned/Guardian/Strider/
  AbstractNautilus → unobstructed-only; Ocelot arm; Ravager liquid-only;
  ZombifiedPiglin = Mob default's two tests; IronGolem/Warden never
  spawn naturally — noted, not defects).
- `max_cluster_size`: AbstractHorse 6 (incl. Camel/CamelHusk),
  AbstractFish 8 (Cod/TropicalFish/Pufferfish/Tadpole — Tadpole extends
  AbstractFish), Salmon 5 (`getMaxSchoolSize`, Cod/TropicalFish inherit
  the default 8 via `AbstractSchoolingFish.getMaxSchoolSize =
  super.getMaxSpawnClusterSize()`), Wolf 8, Ghast/HappyGhast/Pillager 1,
  default Mob 4 — port matches.
- `removeWhenFarAway`: all 20 declarers checked (Animal/AbstractGolem/
  Allay/Villager/WanderingTrader/Warden false; AbstractFish/
  AbstractNautilus/CamelHusk/Hoglin/ZombieHorse/Axolotl true; Cat/Ocelot
  conditional on `Entity.tickCount > 2400` — port passes `Mob.age: u32`
  (entity age), not `AgeableMob.age`, in `ai.rs:2332`; PatrollingMonster
  false iff patrolling && d ≤ 16384 — port `raider::keeps_alive`
  matches; Piglin `!isPersistenceRequired` — fresh piglin true, port
  true; Raider/ZombieVillager handled by `keeps_alive` helpers).
  CopperGolem's spawn-selection `removed_when_far = true` is a DEAD arm
  (AbstractGolem returns false, but copper golems never spawn
  naturally) — noted, not a defect, like the 1.21.7 ZombieHorse note.
- Fortress enemies (`fortress_enemies`, `spawn_selection.rs:705`):
  blaze 10, 2–3; zombified_piglin 5, 4–4; wither_skeleton 8, 5–5;
  skeleton 2, 5–5; magma_cube 3, 4–4 — matches
  `NetherFortressStructure.FORTRESS_ENEMIES`.
- `isValidSpawn`: default = `isFaceSturdy(UP) && lightEmission < 14`
  (`Properties.lambda$new$4`, prepare-computed bitmask); overrides:
  magma = `type.fireImmune()` (bootstrap #410 →
  `lambda$static$272`), ice/frosted ice = polar bear only
  (`lambda$static$133`/`270`), leaves = ocelot/parrot
  (`ocelotOrParrot`), never for glass/trapdoors/barrier/etc., always
  for soul sand/carved pumpkin/redstone lamp/mud — port
  `is_valid_spawn` (`spawn_rules.rs:192`) matches.
- `isBlockDangerous` (`EntityType`): immune tag → false; !fireImmune +
  `NodeEvaluator.isBurningBlock` → true; wither_rose/sweet_berry_bush/
  cactus/powder_snow → true — port matches (campfire lit ⟺ emission >
  0). `isValidEmptySpawnBlock`: !full-collision && !signal-source &&
  fluid empty && !`#prevent_mob_spawning_inside` && !isBlockDangerous —
  port matches (the signal-source check rides in the
  VALID_EMPTY_SPAWN flag; the fallback path never needs it since no
  burning/other block is a signal source).
- `EntitySpawnReason`: `isSpawner` = SPAWNER|TRIAL_SPAWNER,
  `ignoresLightRequirements` = TRIAL_SPAWNER only — port `Reason`
  (`spawn_rules.rs:35-38`) matches exactly.
- Fire immune: vanilla has 15 `fireImmune` types; the port's 11 mob
  types match exactly (the other 4 — area_effect_cloud, ender_dragon,
  end_crystal, tnt — are not mobs).
- `checkDespawn` port (`ai.rs:2308`): peaceful branch first, persistent
  reset (26.2 form — "26.2 resets a persistent mob's no-action time on
  every despawn check; 26.3 leaves it growing"), nearest player,
  category despawn distance, `noActionTime > 600 && nextInt(800)==0 &&
  d > 32²`, `d < 32²` resets no-action time — matches vanilla
  `Mob.checkDespawn` structure.
- Spawn-cost / potential energy (`SpawnState.can_spawn`,
  `energy_change`): `sum(c/dist) * charge`, dist==0 → INFINITY, budget
  check `<= energy_budget` — matches `PotentialCalculator`.

### B7 — Spawner / trial spawner: PASS (3 checks)

`BaseSpawner` defaults 200/800/4/6/16/4, initial spawnDelay 20;
`delay()` = min + `nextInt(max−min)` (or min) + weighted potential pick
+ broadcastEvent(1). Port `mobs/spawner.rs:343-348` defaults match;
`delay()` at 428 matches; `near_player` uses +0.5 offsets; spawn pos
`(nd−nd)*range+0.5`. `SPAWNER_BLOCKS_WORK` game rule EXISTS in 26.2 (so
the port's gate is real, not inert as in 1.21.7). Trial spawner:
FullConfig defaults `target_cooldown_length` 36000 +
`required_player_range` 14 (port `trial_spawner.rs:537-538` matches);
Builder defaults 4 / 6.0 / 2.0 / 2.0 / 1.0 / 40 + loot tables
SPAWNER_TRIAL_CHAMBER_CONSUMABLES + KEY (port `TrialConfig::default` in
`horizon-client-render/src/chunk/block_entity.rs:636` matches).

### B8 — Combat: PASS (1 check)

`CombatRules.getDamageAfterAbsorb` (divisor 2 + toughness/4;
`clamp(armor − damage/divisor, armor*0.2, 20)`; /25;
`EnchantmentHelper.modifyArmorEffectiveness` clamped [0,1];
`damage*(1−eff)`) — port `after_armor`
(`horizon-world/src/combat_rules.rs`) matches.

### B5 — Projectiles: PASS (3 checks)

- Gravities: AbstractArrow 0.05, water inertia 0.6; ThrowableProjectile
  0.03; LlamaSpit 0.06; ThrownExperienceBottle 0.07; AbstractThrownPotion
  0.05; ShulkerBullet 0.04 — port `get_default_gravity`
  (`entities.rs:338`) matches all.
- Hurting projectiles: AbstractHurtingProjectile inertia 0.95, liquid
  inertia 0.8, accelerationPower 0.1 (field default); AbstractWindCharge
  inertia 1.0, liquid = inertia, accel 0.0 in all ctors; WitherSkull
  dangerous 0.73 — port `apply_hurting_inertia`
  (`projectiles.rs:1142`) matches, including the `(v +
  normalize(v)*accel) * inertia` form.
- ThrownTrident water inertia 0.99 — matches.

### B4 — Trading: PASS (4 checks)

G16 (trades exist + `Trades::load` reads them); merchant `cost_a_count`
matches exactly (int product, then float, then floor, max(0), clamp
1..maxStack); `LEVEL_XP` [0, 10, 70, 150, 250]; `updateDemand`
(`Villager`, private in 26.2 — moved out of `AbstractVillager`):
iterates offers, `MerchantOffer.updateDemand` = `demand + uses −
(maxUses − uses)` — port `merchant.rs:66` + `:189` matches;
`updateSpecialPrices`: reputation != 0 → `addToSpecialPriceDiff(
−floor(reputation * priceMultiplier))`; Hero of the Village →
`d = 0.3 + 0.0625 * amplifier` (DOUBLE in 26.2, not 26.3's float),
`addToSpecialPriceDiff(−max(1, floor(d * baseCostA.count)))` — port
`update_special_prices` (`merchant.rs:112`) matches exactly, including
the no-reset-first 26.2 form (26.3's `resetSpecialPrices` is new).

### B6 — Vehicles: PASS, sampled (3 checks)

- Boat `floatBoat` (`game/boats.rs` `tick_boat`): InAir→water transition
  (waterLevel = getY(1.0), target = waterLevelAbove − bbHeight + 0.101,
  noCollision move, velocity.y = 0, status = IN_WATER); per-status
  branches (InWater buoyancy = (waterLevel − y)/height, friction 0.9;
  UnderFlowingWater vspeed −7e-4, 0.9; UnderWater buoyancy 0.01,
  friction 0.45; InAir 0.9; OnLand landFriction, halved when
  controlled); velocity scaling, `deltaRotation *= friction`, buoyancy
  `(vy + buoyancy * getDefaultGravity() / 0.65) * 0.75` — all constants
  match, and `AbstractBoat.getDefaultGravity() = 0.04` matches the
  port's hardcoded 0.04.
- Minecart old behaviour: `OldMinecartBehavior.getMaxSpeed` = in water ?
  0.2 : 0.4 — port `max_speed` (`game/minecarts.rs:750`) matches.
- Minecart new behaviour: `NewMinecartBehavior.getMaxSpeed` =
  `MAX_MINECART_SPEED rule * (0.5 if in water else 1.0) / 20.0` — port
  `max_speed_new` (`game/minecarts.rs:991`) matches exactly, including
  the /20.

### B3 — Mob AI: PASS, sampled (5 checks)

G17 (`reduced()` correct for 26.2); G14/G15 (teleport, see
"Re-checked"); MeleeAttackGoal: canUse 20-tick gate (`gameTime −
lastCanUseCheck < 20`), target alive, createPath with attack-reach
fallback; tick: lookAt 30/30, recalc decrement, recalc `4+nextInt(7)`
(+10 if d>1024, +5 if d>256, +15 if moveTo fails),
`adjustedTickDelay` — port `MeleeAttack` (`goals.rs:1346`) matches, and
vanilla MeleeAttackGoal DOES override `requiresUpdateEveryTick = true`,
so the port's `adjusted(true, ...)` identity is right (the 2-arg
`moveTo(target, speed)` reach-1 form is noted as 26.2's, vs 26.3's
reach-0 3-arg form); BreedGoal: canUse = isInLove + partner within 8
(canMate, not panicking, nearest); tick: lookAt(10, maxHeadXRot),
moveTo partner, loveTime++, breed at `loveTime >=
adjustedTickDelay(60)` (= reduced(60) = 30, since BreedGoal does NOT
require every tick) && distanceToSqr < 9.0; canContinue uses literal 60
— port `Breed` (`goals.rs:2160`) matches both the adjusted threshold and
the literal canContinue bound.

### B9 — Finalize spawn: PASS, sampled (3 checks)

G21 (zombie equipment); Piglin finalize: reason != STRUCTURE →
`nextFloat < 0.2` baby, else adult gets `createSpawnWeapon`
(`nextFloat < 0.5` → crossbow, else `nextInt(10)==0` → GOLDEN_SPEAR,
else GOLDEN_SWORD — the golden-spear arm is genuine 26.2);
`PiglinAi.initMemories` (HUNTED_RECENTLY 600–2400 ticks);
`populateDefaultEquipmentSlots`; `populateDefaultEquipmentEnchantments`;
super.finalizeSpawn — port `finalize.rs:1702` `piglin()` matches order
and values; Drowned equipment (`nextFloat > 0.9`, `nextInt(16) < 10`
trident else fishing rod) matches in both `spawning.rs` and
`finalize.rs`. Note (not a defect): finalize RNG draw order differs
between port and vanilla (port draws loot/baby/doors before equipment;
vanilla per-class chains differ) — values correct, streams diverge.

## Oracle defects (Batch 0's measured table, Batch 3 scope)

No oracle repairs were done (checks + triage only). **30 failing oracles
in 5 groups**, all failing on all three classpaths in the MEASURED Batch 0
table (`docs/sweep/26.2/batch-0-oracles.json`); class + error count quoted
from that table:

- **B3-O01 — missing-override `fuelValues()` (3 oracles, 1 error each,
  Batch 0 group G1):** VanillaActiveEffectTickOracle,
  VanillaAttributeUpdatesOracle, VanillaLivingLandingOracle — "Client*
  Level is not abstract and does not override abstract method
  fuelValues() in Level". 26.2's `Level` declares `public abstract
  FuelValues fuelValues()`; the oracles' 26.3-form level stubs lack it.
  Mechanical repair: add a `fuelValues()` override.
- **B3-O02 — `LocalPlayer` constructor mismatch (5 oracles, Batch 0
  group G2):** VanillaAllayInteractionStateOracle (n=2),
  VanillaBoatRiderOracle (n=2), VanillaSheepDyeOracle (n=1),
  VanillaRemoteAttackOracle (n=1), VanillaPlayerSyncOracle (n=1) —
  "constructor LocalPlayer in class LocalPlayer cannot be applied to
  given types". 26.2's `LocalPlayer` constructor takes different
  arguments than 26.3's.
- **B3-O03 — no-override (6 oracles):** VanillaDamageImmunityOracle
  (n=2), VanillaRaidOracle (n=7), VanillaRegisteredPlayerCombatOracle
  (n=4), VanillaServerPlayerCombatOracle (n=4),
  VanillaBlockingItemDamageOracle (n=2), VanillaFinalizeSpawnOracle
  (n=4) — "method does not override or implement a method from a
  supertype" (26.3 methods absent in 26.2).
- **B3-O04 — missing-symbol 26.x APIs (15 oracles):**
  VanillaEntityDeathLootOracle (n=1), VanillaEntityMoveOracle (n=4),
  VanillaLateEntityPairOracle (n=1),
  VanillaRegisteredEntityFluidOracle (n=2), VanillaWorldSpawnOracle
  (n=5), VanillaMobAiOracle (n=2), VanillaLivingHurtOracle (n=2),
  VanillaLivingFluidOracle (n=3), VanillaTradingOracle (n=1),
  VanillaPlayerHurtOracle (n=2), VanillaPlayerInventoryOracle (n=1),
  EntityTranscript (n=5), LocatorBarClient (n=10), KnockbackClient
  (n=4), RideTranscript (n=2) — all "cannot find symbol".
- **B3-O05 — cascade (1 oracle):** VanillaBoatAuthorityOracle — "depends
  on broken: VanillaRemoteAttackOracle" (B3-O02).

Passing Batch 3-scope oracles (measured, all three classpaths):
VanillaArrowOracle, VanillaAttributeHealthOracle,
VanillaDragonContactsOracle, VanillaEffectNotificationsOracle,
VanillaEffectSyncOracle, VanillaEntityFluidOracle,
VanillaEntitySoundPacketOracle, VanillaFluidBoxOracle,
VanillaLivingFallOracle, VanillaMobAttributesOracle,
VanillaMobBreathingOracle, VanillaMobFallResetOracle,
VanillaNaturalSpawnOracle, VanillaShulkerBulletOracle,
VanillaShulkerSpaceOracle, VanillaSpawnRulesOracle,
VanillaSuffocationOracle, VanillaTradeComponentsOracle,
VanillaVehicleContainerOracle, VanillaVillagerFarmingOracle,
VanillaWorldSpawnPacketOracle, VanillaCombatRulesOracle,
VanillaEntityCollisionOracle, VanillaEntityComponentsOracle,
VanillaEntityAttributeOracle, VanillaEntityObstructionOracle,
VanillaDamageSourceOracle, VanillaFallDamageOracle,
VanillaHelmetDamageOracle, VanillaSwimmingOracle,
VanillaLivingDamageOracle, VanillaLivingHealthOracle,
VanillaLivingAirOracle, VanillaLivingMotionOracle, VanillaMobAirOracle,
VanillaMobDiscardOracle, VanillaRaiderCelebrationOracle,
VanillaServerPlayerDamageOracle, VanillaPlayerTravelOracle,
VanillaPlayerEquipmentOracle, VanillaPlayerFoodOracle,
VanillaPlayerHealthOracle. Pass on client+both (server fails on client
packages — expected, not breakage): VanillaBreezeHookOracle,
VanillaClientArrowOracle, VanillaZombieCureSoundOracle,
VanillaSquidClientOracle.

Adjacent, not Batch 3: VanillaAudioAttributesOracle (missing-symbol,
n=1 — borderline attributes), VanillaTeleportConsumeOracle (PASS —
Batch 7 chorus-fruit area), VanillaEntityAnimOracle (no-override, n=5 —
Batch 10), VanillaFallingBlockOracle (Batch 1), VanillaDragonRenderOracle
(no-override, n=1 — render).

## Drivers (D)

Curated Batch 3 driver list: **167 files** (keyword cut over the 656
`tools/test_*_parity.py` — 209 raw — excluding other batches:
block_entity→4, data catalogs/codecs→13, command/scoreboard→4,
item/inventory/loot→4, worldgen→3, render/model/particles→3,
sound-only→7, dedicated-server infra→2, respawn/hardcore→2, turtle-egg
block tests/dripleaf→3); **157 core** after removing borderline entries
(spawn_loot, underwater_magma, fishing/piston-projectile, squid_client,
server_dimension_players, server_player_targets, server_player_sculk).
No driver was run (static analysis only). Driver checks: 3 — 1 pass
(G18, see "Re-checked"), 2 gap (B3-G19, B3-G20, found by reading). Stale 26.3 docstrings only (no functional
effect): dragon_contacts, mob_discard, mob_quad_leash,
vehicle_container, villager_farming, audio_attributes,
entity_components (comment only — the driver itself handles the 26.2
format, see G18). `test_spawn_loot_parity.py:34` hardcodes
`RESOURCES = ROOT / '.local' / 'horizon-client' / '26.3'` (functional,
borderline scope — loot, Batch 8's area; noted here, not defected).

## Method notes

- Vanilla 26.2's own jar/data is the spec throughout; where a behaviour
  was verified, the exact vanilla method + bytecode is cited in the
  notes above. The 26.3 base's form was never assumed (owner's
  vanilla-check rule). Where 26.2 differs from 26.3, the port carries
  the 26.2 form (e.g. `checkDespawn` persistent reset, hero discount in
  double, no `resetSpecialPrices`, MeleeAttackGoal 2-arg `moveTo`,
  `updateDemand`/`updateSpecialPrices` private in `Villager`).
- Never javap the bundler jar: `leakage-audit/jars/26.2-server.jar` has
  no game classes; all javap here used the named server-game jar in
  `26.1-adapters/versions/26.2/jars/`. Datagen reports were read from
  `26.2/versions/26.2/datagen/reports/`, never from
  `26.2/pipeline/.local/datagen/` (26.3's dump).
- `/tmp` was wiped mid-session (service restart): `/tmp/jarlist.txt`
  was regenerated with `jar tf`, and all javap dumps live in the
  scratch `javap/` dir under the deliverables folder (deleted before
  delivery, per the 1.21.7 precedent).
- A class-file scan (Python `zipfile` + methods-table parse, no JVM)
  over all 716 entity classes established the complete declarer sets
  for `checkSpawnRules` (3), `getWalkTargetValue` (18),
  `checkSpawnObstruction` (13), `removeWhenFarAway` (20) and
  `getMaxSpawnClusterSize` (8) — cheaper and more complete than
  javap-per-class, and it replaced a slow javap loop that was killed.
- Noted, not defects: `MobData::placement` default OnGround vs vanilla
  NO_RESTRICTIONS (unreachable for 26.2 types — all 158 match in
  `mob-data.json`); EnderDragon `_ => true` in `check_spawn_rules`
  (unreachable); ZombieHorse `removed_when_far` (dead arm — zombie
  horses never spawn naturally); CopperGolem `removed_when_far` (dead
  arm — copper golems never spawn naturally; the despawn list in
  `ai.rs` correctly includes it); IronGolem/Warden obstruction overrides
  (never spawn naturally); `spawner_blocks_work` game-rule gate (real
  in 26.2, default true); `DEFAULT_ATTACK_REACH` float-vs-double sqrt
  (~1e-8); port's `adjusted()` ignoring the /tick rate (nuance only
  under non-20 tick rates); finalize RNG draw order (values correct,
  streams diverge).
- Not re-diagnosed (established conclusions): the burn path matches
  vanilla exactly (651b43d3's list corrected — except B3-G08 in 1.21.7,
  which does not apply to 26.2's tag-driven burn); the no-mobs bug fix
  (dd966dea); MobAi's 27 gated scenarios (Batch 0, c9896b77).
- 26.2 inventory facts used throughout: DataVersion 4903, protocol 776,
  158 entity types, 32,366 block states, 1,196 blocks, 40 attributes,
  1,537 items, 93 entity types with attribute suppliers.

## Coordinator note

Reconciled Nov 24, 2026 against the measured Batch 0 table
(`docs/sweep/26.2/batch-0-oracles.json`) and the 26.2 tree:

- **Oracle claims all verify exactly (0 mismatches across 80 checks):** all
  30 failing oracles in B3-O01–O05 (class, error count, fail on all three
  classpaths); all 42 oracles listed as passing all three classpaths; the 4
  client+both oracles (server fails only on client packages, expected); and
  the 4 adjacent claims (AudioAttributes missing-symbol n=1, TeleportConsume
  PASS, EntityAnim no-override n=5, DragonRender no-override n=1).
- **Cross-batch overlaps — per-oracle dedup at merge, inside B3-O04:** three
  of B3-O04's 15 oracles are already recorded elsewhere and fold into the
  canonical items at merge — VanillaRegisteredEntityFluidOracle into B1-O01
  (group member), VanillaLivingFluidOracle into B1-O05, VanillaTradingOracle
  into B4-O25. The other 12 oracles in B3-O04 and all of B3-O01/O02/O03/O05
  are first per-batch records. No whole-item duplicates in this batch.
- **Defect spot-checks (all six verified in source):** B3-G05 — `camel_food`
  appears nowhere in `crates/`; `food_tag()` has a `CamelHusk` arm
  (:1646) and no `Camel` arm. B3-G06 — `Species::Armadillo => 32.0` at
  :1719, unconditional (function starts at :1712 as cited). B3-G09 —
  `sun_burn_tick` computes `wet = in_water || in_rain` only, no
  powder-snow check. B3-G11 — `spawn_selection.rs:1117` uses
  `top = heightmap(WorldSurface)` = H with `nextInt(top-min+1)`, so y is
  [min, H], never H+1. B3-G19 — line 66 hardcodes 35723, confirmed.
  B3-G20 — all five drivers build the server command with
  `lab.LOCAL / 'horizon-client/26.3'` at the cited lines, confirmed.
- **Tally arithmetic verified:** section checks sum to 75 (A1 1, A1b 11,
  A2 1, A3 7, A4 5, B1 3, B2 25, B7 3, B8 1, B5 3, B4 4, B6 3, B3 5, B9 3)
  plus 3 driver checks in §D (1 pass G18, 2 gap G19/G20) = 78 total;
  71 + 1 = 72 pass; 4 + 2 = 6 gap. No corrections needed.
- **Sweep-wide pattern flag (recorded, not re-minted):** the 35723
  state-count hardcode appears in 25 further drivers beyond B3-G19
  (listed in the JSON slice's `out_of_scope_failures`, including
  `test_piston_shape` = B2-G02), and 26.3-resources-path hardcodes appear
  in ~20 further drivers beyond B3-G20 (same list, including the
  functional `test_spawn_loot_parity.py:34`). These belong to other
  batches' scopes; the merge should note the pattern once rather than
  minting per-batch duplicates.
- **Shape note for the merge:** this slice uses `title` for item titles;
  the Batch 4 slice uses `name`. Normalize at merge.
- **Count consistency:** the report's "checkSpawnRules (3)" = Mob,
  PathfinderMob, and SpawnPlacements (the static method) — internally
  consistent; the handoff summary's "only Mob/PathfinderMob" omitted the
  static method.
