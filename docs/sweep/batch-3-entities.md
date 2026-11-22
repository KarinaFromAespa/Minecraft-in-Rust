# Batch 3 — Entities / mobs (1.21.7 parity sweep)

**Version:** 1.21.7 · **Date:** 2026-11-22 · **Branch:** `parity-sweep` (workdir
`~/workspace/mcir-work/sweep-batch-3`, tip `d27d1d33`) · **Scope:**
`docs/PARITY-SWEEP.md` §4 Batch 3 — mob AI, spawning, trading, attributes,
projectiles, vehicles; 151 entity types (81 living + 70 misc); ~109 drivers
curated to 121 files (see §D).

**Status: DRAFT — not committed.** An execution-risk notice on the setup step
blocks running, building, testing, or issuing git commands inside the repo
without the user's go-ahead, so no driver runs, no cargo tests, no oracle
repairs, and no commit/push happened in this batch. Everything below is static
analysis: `javap -c` on the named 1.21.7 server jar
(`~/workspace/mcir-work/leakage-audit/jars/1.21.7-server-named.jar`, world
version 4438, protocol 772), source comparison against
`versions/1.21.7/crates/`, and checks of the prepared 1.21.7 data
(`~/workspace/mcir-work/mob-spawn/resources/`, cross-checked against the main
checkout's `versions/1.21.7/.local/horizon-client/1.21.7/` copy). The
coordinator needs the user's go-ahead to run the drivers/tests, repair the
oracles, and commit these deliverables to
`docs/sweep/batch-3-entities.md` and `pins/1.21.7/sweep/batch-03.json`.

## Tally

- **Behaviour checks: 90 — 70 pass, 20 gap** (across 20 defect items
  B3-G01–B3-G06, B3-G08–B3-G21; B3-G07 was assigned and withdrawn after
  verification — see §A3).
- **Oracle defects: 6 groups** (B3-O01–B3-O06, 40 oracles — not counted in
  the check tally).
- **Oracle repairs done: none** (blocked by the execution-risk notice).
- **Severity: 3 major** (B3-G14 endermen never teleport, B3-G16 villagers
  have no trades, B3-G17 goal timers run 2× fast), **14 minor port**,
  **3 minor driver-side** (B3-G18–B3-G20).

## Defects

### B3-G14 — Endermen never teleport (MAJOR, NEW)

**Checks failed (1):** enderman teleport (random, towards, daylight flee,
projectile dodge — all funnel through one function).

**Cause — 26.x tag grounding for a 1.21.7 `blocksMotion()` check.** Vanilla
1.21.7 `EnderMan.teleport(x, y, z)` scans down while
`!getBlockState(pos).blocksMotion()`, requires `blocksMotion() && fluid not
WATER`, then calls `LivingEntity.randomTeleport(x, y, z, true)`, which itself
grounds on `blocksMotion()` (scan down for a blocks-motion block below the
target), then `teleportTo` + `noCollision` + `!containsAnyLiquid`, else
restores the old position. No tag anywhere; `randomTeleport(DDDZ)` has no tag
parameter. The port's `mobs/enderman.rs:117` grounds on
`#entities_can_teleport_to` and refuses `#enderman_does_not_teleport_to` —
neither tag exists in the 1.21.7 jar's data (zero teleport tags in the jar)
or in the prepared `block-tags.json` (194 tags). `has_tag` returns false for
missing tags (`mobs/world.rs:236-238`), so the ground search never matches
and `teleport_to` always returns false. The port's own doc comment cites the
26.x signature
`LivingEntity.randomTeleport(x, y, z, true, #enderman_does_not_teleport_to)`.
Everything else in the path matches vanilla (random target
`(nextDouble-0.5)*64` / `nextInt(64)-32`; towards-target 16-block + jitter;
daylight flee; projectile dodge; event 46). Secondary nuance (folded in, not
separate): vanilla plays the first teleport sound at `xo/yo/zo` (old
position) and the second at the new position; the port plays both at the new
position. Moot while teleport never succeeds. The unit test
`an_enderman_teleports_inside_the_world_border` asserts `teleport_to`
succeeds on a stone floor while loading REAL 1.21.7 resources — it cannot
pass against real 1.21.7 data.

**Cross-reference:** same absent tag and same root cause as Batch 7's B7-G09
(chorus fruit never teleports, `consume.rs:134` — Batch 7 owns that
instance). The enderman instance is Batch 3's. `/spreadplayers` safety
(`game/commands/world_cmds.rs` `spread_safe`) grounds on the same absent tag
— Batch 9 (commands) to record.

### B3-G16 — Villagers and wandering traders have NO trades (MAJOR, NEW)

**Checks failed (1):** trade tables (pricing, demand, and special-price
machinery all pass — see §B4).

**Cause — the port implements only the 26.x data-driven trade format.**
`Trades::load` reads `data/minecraft/villager_trade/`,
`tags/villager_trade/`, and `trade_set/`; the 1.21.7 jar's data contains ZERO
such files (jar listing: 0 matches) and the prepared `.local` data has no
`villager_trade`/`trade_set` directories. In 1.21.7 trades are CODE-defined
in `VillagerTrades` (`ItemListing` classes: `EmeraldForItems`,
`ItemsForEmeralds`, `EnchantedItemForEmeralds`, `EnchantBookForEmeralds`,
`TreasureMapForEmeralds`, `ItemsAndEmeraldsToItems`,
`TippedArrowForItemsAndEmeralds`, `SuspiciousStewForEmerald`,
`DyedArmorForEmeralds`, `EmeraldsForVillagerTypeItem`, `TypeSpecificTrade`).
The port has NO code-path implementation of any of them (no
`EmeraldForItems`/`ItemsForEmeralds` anywhere in the crate), and
`ensure_offers` returns `Vec::new()` when the set is missing — so every
villager and wandering trader opens with zero offers (`interact_trader`
shakes its head). The `trade_rebalance` datapack present in 1.21.7 data
carries only enchantment providers/tags, not trade tables.

### B3-G17 — `reduced()` halves all non-every-tick goal delays: goal-timed behaviours run 2× fast (MAJOR, NEW, systemic)

**Checks failed (1):** goal delay scaling (`reduced` / `adjusted(false, …)`),
~40 call sites.

**Cause — a wrong cadence conversion.** Vanilla `Mob.serverAiStep`
alternates on `(tickCount + getId()) % 2` (bytecode-verified: odd and
`tickCount > 1` → `tickRunningGoals(false)`, else full `tick()`), and the
port mirrors it exactly (`ai.rs:2067-2071`). So non-every-tick goals tick
every OTHER game tick in BOTH, and vanilla goal counters increment per
goal-tick — a threshold of N goal-ticks is 2N game ticks in both. The
correct conversion is identity. The port's `goals.rs:66`
`reduced(ticks) = (ticks + 1) / 2` halves every threshold/interval for
non-every-tick goals, so all such behaviours run twice as fast. Evidence:
`LookAtPlayerGoal.start` sets `lookTime = 40 + nextInt(40)` (javap bipush
40/40), decremented per goal-tick → 80–160 game ticks in vanilla; the port
sets `reduced(40 + nextInt(40))` = 20–40 goal-ticks → 40–80 game ticks.
Breed: vanilla 60 goal-ticks = 120 game ticks; port `reduced(60)` = 30 →
60 game ticks. Random rolls (`next_int_bound(reduced(N)) == 0` per
goal-tick) fire at twice the per-game-tick rate. Affected: breed love time,
look times, path-recalc intervals, bee travel timeouts, slime tired,
fox/enderman/dolphin/door/caravan timers, tempt calm-down. Call sites:
`goals.rs` (1106, 1283, 1545, 1618, 1637, 1710, 2250, 2356, 2456, 2575,
2589, 2609), `goals/tempt.rs:65`, `aquatic.rs` (661, 833, 923), `bee.rs`
(1002, 1067, 1099), `caravan.rs:148`, `cube.rs` (370, 411), `dolphin.rs`
(195, 364), `doors.rs:172`, `enderman.rs` (370, 423, 444, 502), `fox.rs`
(155, 528, 571, 750, 925, 982, 1131, 1281). `adjusted(false, …)` applies the
same halving. Every-tick goals (`adjusted(true, …)` = identity) are
unaffected.

### B3-G01 — Bat: missing Halloween branch (minor)

Vanilla `Bat.checkBatSpawnRules`: limit 7 + NO coin flip on Halloween
(`LocalDate` Oct 31), else limit 4 + 50% `nextBoolean` fail. The port always
applies the coin flip + limit 4. `SpawnLevel` has no date, so the branch
needs a trait addition.

### B3-G02 — Slime surface spawn: 26.x environment-attribute form (minor)

Vanilla: `nextFloat() < 0.5 && nextFloat() < moonBrightness && brightness <=
nextInt(8)` (two float draws, moon-phase dependent). The port uses a single
`nextFloat() < surface_slime_chance`, reading the 26.x environment attribute
`gameplay/surface_slime_spawn_chance`; the test fixture at
`spawn_rules.rs:600-603` computes `0.5 * MOON_BRIGHTNESS[phase]`, proving the
port knows the right formula. Wrong rate (moon phase ignored) + wrong RNG
stream. The underground branch matches (slime-chunk seed 987234911,
`nextInt(10)`, y<40).

### B3-G03 — `immune_tag` tags don't exist in 1.21.7 (minor)

Vanilla `EntityType.Builder.immuneTo` is hardcoded blocks: fox →
sweet_berry_bush; polar_bear/snow_golem/stray → powder_snow; wither /
wither_skeleton → wither_rose. The port's `immune_tag()` maps the same 6
species to tags `minecraft:fox_immune_to` etc. — none exist in the 1.21.7
jar's data or `block-tags.json` (194 tags; only `dragon_immune` /
`wither_immune`, which are explosion tags). So `is_valid_empty_spawn`
treats those blocks as dangerous for those species. Same bug class as the
pilot's tag defects.

### B3-G04 — Hoglin blocked in peaceful (minor)

Vanilla: `checkHoglinSpawnRules` has NO difficulty check, and Hoglin
(extends Animal) has `shouldDespawnInPeaceful() == false` (Mob base) —
hoglins spawn and persist in peaceful. The port's `allowed_in_peaceful`
disallows Hoglin. Verified: no `isAllowedInPeaceful` exists anywhere in
1.21.7 (26.x-shaped mechanism); `Monster.shouldDespawnInPeaceful()` returns
true; Piglin overrides false; Ghast/Phantom override true; Slime is
`getSize() > 0`. The port's disallowed list is otherwise exactly right.

### B3-G05 — Camel has no food (minor)

The jar has 25 `*_food` item tags including `camel_food`. The port's
`food_tag()` covers all EXCEPT camel: "camel_food" appears nowhere in the
port, so `MobType.food` is empty for camel and feeding/breeding/tempting
(`farm.rs`, `animal_brain.rs` sense_tempting) never fire. Camels cannot
breed in the port. Vanilla `Camel.isFood` = `#camel_food` (cactus).

### B3-G06 — Armadillo scared head rotation (minor)

Vanilla `Armadillo.getMaxHeadYRot` = `isScared() ? 0 : 32`. The port's
`species.rs` returns 32.0 unconditionally.

### B3-G08 — Drowned missing from `burns_in_day` (minor)

Drowned extends Zombie and defines NEITHER `aiStep` NOR `isSunSensitive`
(full method list verified) — it inherits `Zombie.aiStep` +
`isSunSensitive() == true`, so a drowned on land in daylight burns in
vanilla (in water, `isSunBurnTick` is false, so it never burns there). The
port's `species.rs:1550` list excludes Drowned and its comment claims "the
drowned … never call it" — wrong. This corrects the 651b43d3 claim (which
removed Drowned from the list as over-burning). The burn formula itself
matches vanilla exactly (see §A4).

### B3-G09 — Sun-burn ignores powder snow (minor)

Vanilla `isSunBurnTick` excludes `isInPowderSnow || wasInPowderSnow`; the
port's `sun_burn_tick` checks only in-water / in-rain, although `Mob` has
`in_powder_snow` state (`mobs.rs:1609`). A skeleton standing in powder snow
burns in the port, not in vanilla. (`was_in_powder_snow` exists only for the
player.)

### B3-G10 — Creatures spawn too far from players (minor)

Vanilla `EntityType.canSpawnFarFromPlayer` is true only for MISC-category
types (Builder default: `category == MISC`) + pillager + shulker (the only
two explicit setter calls in `EntityType.<clinit>`). The port's
`spawns_far_from_player` also returns true for all CREATURE types, so
creature attempts more than 128 blocks from the nearest player pass
`valid_position_for_type` where vanilla rejects them. Observable: candidate
chunks need only their CENTRE within 128 blocks of a player
(`close_enough_for_spawning`, strict < 16384 on horizontal distance), so
attempt positions at far corners exceed 128.

### B3-G11 — `getRandomPosWithin` y-range off by one (minor)

Vanilla: `y = Mth.randomBetweenInclusive(random, minY, getHeight(WORLD_SURFACE) + 1)`
→ y ∈ [minY, H+1]. Port (`spawn_category_for_chunk`):
`y = min + next_int_bound(top - min + 1)` where top = `world_surface` =
"one above the highest non-air block" = vanilla's getHeight value H →
y ∈ [minY, H]. The port never picks y = H+1 and consumes a different
`nextInt` bound (RNG stream diverges).

### B3-G12 — Enderman walk-target value (minor)

`EnderMan.class` contains ZERO occurrences of `getWalkTargetValue` — Enderman
inherits Monster's `-cost_from_light` (negative in bright light → spawn
rejected). The port maps Enderman → Zero with the comment "Monster
subclasses that override it with 0" — factually wrong for Enderman
(Creaking/Pillager/Warden do override to 0; Enderman doesn't). Narrowly
observable (monster spawn rules already require near-darkness, where the
Monster value is ≥ 0).

### B3-G13 — Constant-count packs skip an RNG draw (minor)

1.21.7 `SpawnerData` is a plain record (`minCount`, `maxCount`) — no
`IntProvider`. Vanilla samples `min + nextInt(max + 1 - min)` UNCONDITIONALLY
(`nextInt(1)` still consumes a LegacyRandom draw when min == max). The
port's `sample_count` draws nothing for constant entries (26.x
`count().sample` form). Shifts the RNG stream for the rest of every attempt
whose entry has min == max (4 of 5 fortress entries; many biome entries).

### B3-G15 — Cat morning-gift hop never happens (minor)

`tame.rs` `give_morning_gift` grounds the hop on the same absent
`#entities_can_teleport_to` (line 1363), so the cat never hops; the gift
itself still drops (`DropGift` is outside the hop gate). Same root cause as
B3-G14 / B7-G09. (Also references `#dangerous_for_teleportation`, likewise
absent — harmless, never dangerous.)

### B3-G18 — `test_entity_components_parity.py` crashes on 1.21.7 advancement format (driver, minor)

The driver reads `criterion['conditions']['entity']['predicate']` and
`predicate['minecraft:components']` (the 26.x format). In 1.21.7,
`conditions.entity` is a LIST of `entity_properties` loot conditions
(verified in all 3 advancements: complete_catalogue 11 criteria, whole_pack
9, leash_all_frog_variants 3), each with
`predicate: {components: {minecraft:cat/variant: …}}` — key `components`,
not `minecraft:components`. The driver raises TypeError on the first
criterion. This is the "1 harness Python bug (entity_components)" in
PARITY-SWEEP.md §2 — it blocks the only Batch 3 harness in the measured
set. (Also: the "wrong type" control always uses `minecraft:pig`, so for
pig cases it is not actually wrong — mislabelled control, not a crash.)

### B3-G19 — `test_suffocation_parity.py` hardcodes the 26.3 state count (driver, minor)

Line 66: `if len(facts['states']) != 35723: raise
AssertionError('Incomplete pinned registry')`. 1.21.7 has 27,946 states, so
the driver always fails. Same class as Batch 2's G13. (Other 35723
hardcodes — `test_scattered_ore`, `test_block_landing`,
`test_block_properties`, `test_block_state_codec`, `test_block_support`,
`test_block_survival` — belong to other batches.)

### B3-G20 — `test_locator_bar_parity.py` / `test_server_dragon_parity.py` hardcode the 26.3 resources path (driver, minor)

Both build the Rust server command with
`lab.LOCAL / 'horizon-client/26.3'` (lines 269 and 111); 1.21.7 resources
live at `horizon-client/1.21.7`. (`test_predicate_reference_startup_parity.py`
has the same hardcode but is a commands/predicate driver — adjacent, not
Batch 3.)

### B3-G21 — Zombies spawn holding iron spears, with wrong sword/shovel odds (minor)

Vanilla `Zombie.populateDefaultEquipmentSlots`: super (Monster armour), then
`nextFloat < (hard ? 0.05 : 0.01)`, then `nextInt(3)`: 0 → iron sword, else
iron shovel (1/3 sword, 2/3 shovel). The port uses `nextInt(6)`: 0 → sword,
1 → `minecraft:iron_spear` (a 26.x-only item — does not exist in 1.21.7),
else shovel (1/6 sword, 1/6 spear, 4/6 shovel). The bug is in BOTH
implementations: `spawning.rs` finalize Zombie arm AND `finalize.rs`
`populate()` Zombie arm.

### B3-G07 — WITHDRAWN (happy-ghast tempt)

Assigned to a happy-ghast-tempt candidate, then withdrawn after
verification: the port's `GhastTempt::wants` (harness-if-bare-adult ||
food) matches vanilla `lambda$registerGoals$1` exactly (bare adult →
`#happy_ghast_tempt_items` = snowball + harnesses, else IS_FOOD = snowball);
`TemptGoal.ForNonPathfinders` prio 4, speed 1.0, stopDistance 7.0,
`FloatGoal` prio 3, `RandomFloatAroundGoal` prio 5 range 16 — all match.
Do not reuse the number without noting the withdrawal.

## Passes (by section)

### A1 — Spawn placements (151): PASS

Vanilla `SpawnPlacements.<clinit>`: 79 register calls (ON_GROUND 57,
IN_WATER 11, NO_RESTRICTIONS 10, IN_LAVA 1; heightmap
MOTION_BLOCKING_NO_LEAVES 77, MOTION_BLOCKING 2: ocelot, parrot).
Unregistered types default NO_RESTRICTIONS + MOTION_BLOCKING_NO_LEAVES.
`mob-data.json` placements (151) vs vanilla (79 + defaults): **0
mismatches**. 38 distinct predicates inventoried.

### A1b — Spawn predicates (38): 34 pass, 4 gap (G01–G04)

Verified method-by-method against `check_spawn_rules`. Passes include:
surface water (incl. the above-block water check), tropical fish, ageable
water creature, `isDarkEnoughToSpawn` (sky > nextInt(32); block-light limit;
thundering darken 10; `monsterSpawnLightTest` sample), monster/animal/mob
rules, drowned (15/40 rolls, deep-enough y < sea−5, reinforcement), guardian
(1-in-20 or no sky-from-below-water), patrolling monster (block light ≤ 8 +
ANY-light rules, NOT monster rules), endermite/silverfish (any-light +
nearest-player 5.0), hoglin/piglin (wart-block only, NO mob_rules),
skeleton horse (spawner branch incl. ignores-light carve-out), magma cube
(peaceful only), ghast (peaceful + 1-in-20 + mob rules), strider (lava
column + air), ocelot (nextInt(3)≠0), stray (powder-snow scan + canSeeSky —
exact), axolotl (tag), frog/goat/camel/armadillo/parrot/rabbit/wolf/
mooshroom/fox (tag + bright), turtle (y < sea+4 + `TurtleEggBlock.onSand` =
below in SAND tag + bright), polar bear (alternate-biome branch + animal
rules), zombified piglin (peaceful + no wart), husk (monster + sky).
`EntitySpawnReason.isSpawner` = SPAWNER||TRIAL_SPAWNER;
`ignoresLightRequirements` = TRIAL_SPAWNER only — port matches.
`isValidEmptySpawnBlock` and `isBlockDangerous` (immuneTo set, then
!fireImmune && `isBurningBlock` — FIRE tag/lava/magma/lit campfire/lava
cauldron — then wither rose/sweet berry/cactus/powder snow) match.
Noted, not defects: `MobData::placement` default is OnGround where vanilla
defaults NO_RESTRICTIONS — unreachable for 1.21.7 types (all 151 in the
map); EnderDragon falls to `_ => true` in `check_spawn_rules` (vanilla:
`Mob.checkMobSpawnRules`) — unreachable in practice.

### A2 — Species coverage: PASS

`species.rs` `ALL`: 90 defs. 1.21.7 mob types = 84 (81 living + iron_golem,
snow_golem, villager). Extra defs are 7 26.x-only types (camel_husk,
copper_golem, mannequin, nautilus, parched, sulfur_cube, zombie_nautilus) —
dead in 1.21.7. Only 1.21.7 mob missing from Species: **ender_dragon**
(handled separately in `game/dragon.rs`, by design). All 84 mobs have
attribute sets; 86 total with attributes (84 mobs + armor_stand + player).

### A3 — Attributes + species tables: 6 pass, 2 gap (G05, G06)

- 35 attributes in `attributes.json` vs jar `Attributes.<clinit>`
  (RangedAttribute default/min/max/syncable): **35/35 match exactly**.
- `attributes.json[entities]` vs `entity-types.json` attributes:
  numerically identical (0 diffs). Game loads via `exact_attributes`
  (`mobs.rs:466`, raw f64 parse). `MobType` hardcoded fallbacks are dead
  for 1.21.7 (all species have full data).
- `fire_immune`: jar has 15 types; port covers all 11 that are Species.
- Per-entity `createAttributes`: LivingEntity base = 20 DEFAULT adds; Mob
  adds FOLLOW_RANGE 16; Monster adds ATTACK_DAMAGE; Animal adds TEMPT_RANGE
  10. Full-set comparison vs `entity-types.json` for 8 sampled types —
  zombie 23, pig 22, ender_dragon 21, warden 22, player 31, armor_stand 20,
  allay 23, creeper 22 — **ALL exact** (incl. float-widened values like
  0.23000000417232513). Skeleton and Horse have no own `createAttributes`
  (inherit).
- Ambient intervals (javap): Mob 80, Animal 120, Turtle 200, Ocelot 900,
  Guardian 160, AbstractHorse 400, WaterAnimal 120, AbstractGolem 120 — ALL
  match `species.rs`.
- Head rotation (javap): Mob defaults 40/75/10; Dolphin/Axolotl 1/1; Slime
  maxHeadX 0 (MagmaCube inherits); Shulker 180/180; Guardian 180 (X); Frog
  headRot 35, maxHeadY 5; Breeze 30/25; Sniffer 50; Goat 15; Ravager 45;
  Camel 30 — ALL match except Armadillo (G06).
- Water slowdown: LivingEntity 0.8, PolarBear 0.98, SkeletonHorse 0.96 —
  match. Air speeds: Phantom `travelFlying(0.2)`; HappyGhast
  FLYING_SPEED×5/3; Allay `travelFlying(getSpeed())`; Ghast 0.02;
  `LivingEntity.travelFlying` delegates — all match.

### A4 — Daylight burning: formula PASS, 2 gaps (G08, G09)

`isSunBurnTick` callers in the jar: EXACTLY Zombie, Phantom,
AbstractSkeleton. Zombie.aiStep: flag = `isSunSensitive() &&
isSunBurnTick()`; helmet damage `nextInt(2)`, break at max damage, else
`igniteForSeconds(8.0)`. AbstractSkeleton.aiStep: same, no isSunSensitive
gate. Phantom.aiStep: `isSunBurnTick` → `igniteForSeconds(8)`, no helmet
check. `Mob.isSunBurnTick`: `level.isBrightOutside()` && !isClientSide;
light magic value > 0.5; `nextFloat()*30 < (f-0.4)*2`; wet =
`isInWaterOrRain() || isInPowderSnow || wasInPowderSnow`;
`canSeeSky(eye pos)`. Port `sun_burn_tick` matches (monsters_burn =
synthesized isBrightOutside per a2f95487; brightness at eye; same formula;
160 ticks = 8s). Helmet damage via SunProtectionWear event →
`creatures.rs` wear_sun_protection (`nextInt(2)`, damageable check)
matches. Zombie.isSunSensitive = true; Husk = false; ZombieVillager
inherits true (burns). ZombieHorse (AbstractHorse) never calls it.

### B1 — Suffocation: PASS (Batch 0 O9 confirmed implemented)

Vanilla default (Properties ctor bootstrap #6 → `lambda$new$7`):
`blocksMotion() && isCollisionShapeFullBlock()` — exactly the port's
MotionFullCollision. All 15 `Blocks.<clinit>` overrides enumerated via
javap + bootstrap table (#8 = `Blocks.never`, #93 = `Blocks.always`):
Always (#93): farmland, soul_sand, dirt_path, mud. Never (#8): stained
glass, leaves (incl. cherry/pale_oak direct), glass, tinted_glass,
copper_grate (+7 variants via ofFullCopy), mangrove_roots, moving_piston.
NOT_EXTENDED_PISTON (`lambda$static$1`): !EXTENDED. NOT_CLOSED_SHULKER
(`lambda$static$0`): BE is ShulkerBox ? isClosed() : true — matches port
`Rule::Shulker` (`closed != Some(false)`); the field name is misleading,
semantics are "suffocates iff closed". `Entity.isInWall`: noPhysics false;
box AABB.ofSize(eye, 0.8w, 1e-6, 0.8w); betweenClosedStream; !isAir &&
isSuffocating && joinIsNotEmpty(collision.move(pos), Shapes.create(eyeAABB),
AND). Port `in_wall` (`ai.rs:1172`) + closure matches exactly. Damage 1.0
InWall matches. Vex exclusion matches: `Vex.tick` sets noPhysics=true
around `super.tick()` (javap-verified).

### B2 — Natural spawning: 8 pass, 4 gap (G10–G13)

`spawnCategoryForPosition` structure matches: redstone-conductor early
return; 3 attempts; pack size `ceil(nextFloat*4)` then
`min + nextInt(max+1-min)`; walk `nextInt(6)-nextInt(6)`; nearest player
(any distance); right-distance 576 + spawn-point 24 +
same-chunk-or-canSpawnEntitiesInChunk; `getRandomSpawnMobAt` (water-ambient
0.98 + `#reduce_water_ambient_spawns`); `isValidSpawnPostitionForType`;
SpawnPredicate (spawn-cost state); yaw `nextFloat*360` AFTER the
spawn-rules draws; `isValidPositionForMob`; finalizeSpawn NATURAL; cluster
cap; TropicalFish `isMaxGroupSizeReached` = `!isSchool` (port:
`!result.school`). `MobCategory` values from `<clinit>` (name, max,
friendly, persistent, despawn): MONSTER 70/false/false/128, CREATURE
10/true/true/128, AMBIENT 15/true/false/128, AXOLOTLS 5/true/false/128,
UNDERGROUND_WATER_CREATURE 5/true/false/128, WATER_CREATURE 5/true/false/128,
WATER_AMBIENT 20/true/false/64, MISC −1/true/true/128 — ALL match.
`getNoDespawnDistance` = 32. `max_cluster_size`: Mob 4; AbstractFish 8;
AbstractSchoolingFish = `getMaxSchoolSize` (Salmon 5, others 8);
AbstractHorse 6; Wolf 8; Ghast 1; HappyGhast 1; Pillager 1 — ALL match.
`removeWhenFarAway` (fresh mob) verified exhaustively via javap of all 16
definers — port matches for all EXCEPT ZombieHorse (port true; vanilla
inherits Animal's false) — **not a defect**: the function is used only in
`spawn_category_for_position` and zombie horses never spawn naturally (dead
arm). Walk-target values verified for all overriders (PathfinderMob 0;
Monster −cost; Animal grass 10 else cost; Mooshroom mycelium 10 else cost;
Turtle water/onSand 10 else cost; Guardian water 10+cost else −cost;
Silverfish infested-host 10 else Monster's; Strider lava 10 else 0; Hoglin
repellent −1 / crimson_nylium 10 else 0; Bee air 10 else 0; HappyGhast
!empty 0 / air-below-solid 10 / else 5; Giant +cost; Creaking/Pillager/
Warden/Axolotl 0); `cost_from_light` == `getPathfindingCostFromLightLevels`
exactly (magic = lerp(dimension ambientLight, v/(4−3v), 1), v =
max(block, sky−darken)/15). Obstruction verified: Mob !liquid &&
unobstructed; WaterAnimal/AgeableWaterCreature (Squid, Dolphin extend
it)/Axolotl/Drowned/Guardian/Strider = unobstructed only; Ravager !liquid
only; Ocelot unobstructed && !liquid && y≥sea && (grass_block || #leaves)
— exact; ZombifiedPiglin = Mob default. IronGolem/Warden have own overrides
but never spawn naturally (unreachable). `mobsAt` verified: fortress
(MONSTER + nether_bricks below + in-bounds) → FORTRESS_ENEMIES exactly
(blaze 10, 2–3; zombified_piglin 5, 4–4; wither_skeleton 8, 5–5; skeleton
2, 5–5; magma_cube 3, 4–4); then structure overrides; then biome list.
`weighted_pick` matches `WeightedList.getRandom`. Side note: `SpawnerData`'s
ctor replaces MISC-category types with PIG — unreachable, no biome list
contains MISC types.

### B7 — Spawner / trial spawner: PASS

BaseSpawner: defaults 200/800/4/6/16/4 (min/max delay, count, max nearby,
player range, spawn range) match; `delay()` = min + nextInt(max−min) (or
min if max≤min) + weighted potential pick + broadcastEvent(1) — port
matches; `isNearPlayer` = `hasNearbyAlivePlayer(x+0.5, y+0.5, z+0.5, range)`
matches; random pos = x/z `(nextDouble−nextDouble)*range+0.5`, y +
`nextInt(3)−1`, draw order matches; `noCollision(getSpawnAABB)`; custom
rules: non-friendly + peaceful → skip, `isValidPosition` → skip; NO custom
rules → `SpawnPlacements.checkSpawnRules(SPAWNER)` — the port's else-branch
`check_spawn_rules(Reason::Spawner)` is CORRECT. NOTE (not a defect): the
port gates on game rule `spawner_blocks_work`, which does NOT exist in
1.21.7's GameRules (javap-verified full rule list); the port's default is 1
(true), so the gate is inert in default play. TrialSpawner: FullConfig
codec defaults `target_cooldown_length` 36000 + `required_player_range` 14
match the port; `TrialConfig::default` (4 / 6.0 / 2.0 / 2.0 / 1.0 / 40)
matches `TrialSpawnerConfig.Builder` defaults; loot tables match.

### B8 — Combat: PASS

`CombatRules.getDamageAfterAbsorb` (javap): divisor 2 + toughness/4;
`clamp(armor − damage/divisor, armor*0.2, 20)`; /25; weapon via
`DamageSource.getWeaponItem` → `EnchantmentHelper.modifyArmorEffectiveness`
on server level, clamped [0,1]; `damage*(1−eff)`. Port
`combat_rules::after_armor` matches exactly (incl. the `Mth.clamp` NaN/−0.0
form). `getDamageAfterMagicAbsorb`: `clamp(prot, 0, 20)`;
`damage*(1−prot/25)` — matches. `LivingEntity.knockback` (port
`mobs.rs` `knockback_by`): `power *= (1 − knockback_resistance)`; ≤0
return; random dir while len² < 1e-5 (`(nd−nd)*0.01`); vel = (x/2 − kx,
on_ground ? min(y/2 + power, 0.4) : y, z/2 − kz) — matches vanilla.
Creaking immovable check matches `Creaking.knockback`. SulfurCube arm is
26.x-only (dead). CombatTracker: recheck 100 ticks (300 in combat), record
before actuallyHurt, living cause enters combat — matches vanilla.

### B5 — Projectiles: PASS

Gravity (javap, all exact): AbstractArrow/Trident 0.05;
ThrowableProjectile 0.03 (port default); AbstractThrownPotion 0.05;
LlamaSpit 0.06; ThrownExperienceBottle 0.07; ShulkerBullet 0.04;
fireballs/wither skull 0.0. Inertia: AbstractArrow water 0.6
(`getWaterInertia`), air 0.99; ThrownTrident water 0.99;
ThrowableProjectile water 0.8 / air 0.99; AbstractHurtingProjectile
`getInertia` 0.95, `getLiquidInertia` 0.8, `accelerationPower` 0.1,
`applyInertia` = (v + normalize(v)*accel) * inertia; WitherSkull dangerous
0.73 else 0.95; AbstractWindCharge (in the `windcharge` subpackage)
`getInertia` 1.0, `getLiquidInertia` = `getInertia`, `accelerationPower`
0.0 in all ctors. Port matches on every value (`projectiles.rs`
`get_default_gravity` + `apply_hurting_inertia` + `tick_inner`;
`arrows.rs` 0.6/0.99). Bubble particles: 4 at position − movement*0.25 —
matches `water_bubble_at`.

### B4 — Trading: machinery PASS, tables absent (G16)

`MerchantOffer` pricing matches vanilla exactly: cost_a =
`clamp(base + max(0, floor(base*demand*priceMultiplier)) + specialPriceDiff,
1, maxStack)` (int product, then float, then floor — port `merchant.rs`
`cost_a_count` identical); cost_b raw; `updateDemand` = demand + uses −
(maxUses − uses) — identical; `updateSpecialPrices`: reputation
`−floor(rep * priceMultiplier)`, hero 0.3 + 0.0625*amp, reduction
`max(1, floor(hero_mod * base_count))` — matches (note: vanilla multiplies
count*f in FLOAT, the port in f64 after widening the modifier — floors agree
for all practical values; a precision nuance, not a defect). LEVEL_XP
[0, 10, 70, 150, 250] matches `VillagerData.NEXT_LEVEL_XP_THRESHOLDS`.

### B6 — Vehicles: PASS (sampled)

Boat (port `game/boats.rs` `tick_boat` vs AbstractBoat): `floatBoat`
statuses match (InWater buoyancy=(waterLevel−y)/height, friction 0.9;
UnderFlowingWater vspeed −7e-4, 0.9; UnderWater buoyancy 0.01, friction
0.45; InAir 0.9; OnLand land_friction, halved when controlled); buoyancy
term `(y + buoyancy*(0.04/0.65))*0.75`; `controlBoat`: delta_rotation ∓1,
turn-only +0.005, up +0.04, down −0.005, paddling flags — all match
vanilla's constants. (`Boat` itself is a thin subclass of `AbstractBoat`
in 1.21.7.) Minecart: port max_speed 0.4 on rails / 0.2 in water; vanilla
delegates `getMaxSpeed` to `MinecartBehavior` (abstract) with old/new
behaviour implementations — the port has `max_speed` + `max_speed_new`
mirroring them. Sampled only; full minecart-behaviour audit not done.

### B3 — Mob AI: sampled PASS except G17

- MeleeAttackGoal: can_use every 20 ticks, path-or-range, target alive;
  can_continue follow_unseen home + spectator/creative refusal; tick
  lookAt 30/30, recalc 4+nextInt(7) (+10 if d²>1024, +5 if >256, +15 if
  path fails), attack when until_attack≤0 && in range && sees, interval
  adjusted, swing + doHurtTarget; zombie raise-arm aggressive =
  raiseArm≥5 && until_attack<10 — all match vanilla (port
  `adjusted(true, 20)/2` = 10 = vanilla `adjustedTickDelay(10)` at 20tps;
  the port's `adjusted()` ignores the /tick rate, a nuance only under
  non-20 tick rates).
- `Mob.isWithinMeleeAttackRange` =
  `getAttackBoundingBox().intersects(target.getHitbox())`;
  `getAttackBoundingBox` = vehicle-box union (X/Z min/max, own Y) inflated
  by `DEFAULT_ATTACK_REACH` on X/Z — port `melee_reaches` matches.
  `DEFAULT_ATTACK_REACH` = `sqrt(2.04f) − 0.6` (Mob `<clinit>` shows the
  widened double 2.0399999618530273); the port computes the sqrt in float
  and widens — differs by ~1e-8, not a defect.
- BreedGoal: partner search 8 blocks, canMate checks (wolf/cat tame, mule
  never, turtle egg, sniffer calm, equine/camel tame+adult+unridden+full
  health), can_continue partner alive+in love && love_time<60, breed at
  love_time≥60 && d²<9, in_love=0 + age 6000, turtle egg / frog pregnant /
  llama offspring + XP 1+nextInt(7) inside mob_drops — all match vanilla
  EXCEPT the love_time threshold (G17).
- Love mode: in_love = 600 ticks, hearts every 10 ticks — matches Animal.
- `checkDespawn`: peaceful branch BEFORE persistence (discards disallowed
  even when persistent/NoAI) ✓; nearest player, persistence (NBT +
  requiresCustomPersistence: passenger/leashed/in-raid/enderman-carrying/
  from-bucket) ✓; despawn distance water_ambient 64 else 128 ✓;
  noActionTime>600 && nextInt(800)==0 && d>32² ✓; d<32² resets
  noActionTime ✓; feline rule (untamed/untrusting + age>2400) ✓; chicken
  jockey ✓; raider keeps_alive (in-raid || patrolling && d²≤16384) ✓;
  zombie-villager keeps ✓; never_removed (villager/wandering trader/
  golems/shulker/allay/warden/wither) ✓. `WitherBoss.checkDespawn`
  verified: peaceful && shouldDespawnInPeaceful → discard, else
  noActionTime=0 — the Wither DOES despawn in peaceful in vanilla, so the
  port matches.
- GoalSelector cadence: vanilla `Mob.serverAiStep` alternates on
  `(tickCount + id) % 2` (bytecode-verified) — the port mirrors it exactly
  (`ai.rs:2067-2071`). `GoalSelector.tick` ends with
  `tickRunningGoals(true)`; `tickRunningGoals(force)` ticks running goals
  if force || requiresUpdateEveryTick — port matches.

### B9 — Finalize spawn (sampled): 1 gap (G21)

- Piglin.finalizeSpawn verified: reason != STRUCTURE → nextFloat<0.2 baby,
  else adult gets `createSpawnWeapon` (nextFloat<0.5 crossbow else golden
  sword); `PiglinAi.initMemories`; `populateDefaultEquipmentSlots` (adult
  only, `maybeWearArmor` golden pieces — 0.1 each);
  `populateDefaultEquipmentEnchantments`; super.finalizeSpawn. Port matches
  on values (`spawning.rs` Piglin arm). ORDER note: vanilla draws the
  weapon BEFORE the armour rolls; the port draws armour first — RNG stream
  diverges (same class as B3-G13, noted not separately defected).
  `Piglin.shouldDespawnInPeaceful` = false confirmed (port allows Piglin).
- `Drowned.populateDefaultEquipmentSlots` verified: nextFloat>0.9,
  nextInt(16)<10 trident else fishing rod — port matches exactly.
- AgeableMob baby groups: port `finalize.rs` Ageable model
  (should_spawn_baby, baby_chance, size>0, float≤chance) matches vanilla's
  `AgeableMobGroupData` structure; horse/panda 0.2, polar bear/ocelot 1.0
  (not individually javap-verified — spot check only).
- Method note: the port's centralized `finalize()` draws in a different
  ORDER from vanilla's per-class `finalizeSpawn` chain (vanilla:
  super.finalizeSpawn/equipment first, then loot/baby/doors rolls; the
  port: loot/baby/doors first, then equipment) — RNG streams diverge for
  all species even where values match. Values verified correct for the
  sampled species.

## Oracle defects (Batch 0's remaining list, Batch 3 scope)

No oracle repairs were done (blocked). 40 failing oracles in 6 groups;
first-error evidence from `docs/sweep/batch-0-oracles.json`:

- **B3-O01 — `gamerules` package (7 oracles):**
  VanillaAttributeHealthOracle, VanillaDamageImmunityOracle,
  VanillaEntityDeathLootOracle, VanillaNaturalSpawnOracle, VanillaRaidOracle,
  VanillaRegisteredPlayerCombatOracle, VanillaServerPlayerCombatOracle — all
  "package net.minecraft.world.level.gamerules does not exist" (26.x
  package; 1.21.7 GameRules lives in `net.minecraft.world.level`).
- **B3-O02 — `client.multiplayer` package (5 oracles):**
  VanillaAllayInteractionStateOracle, VanillaBoatRiderOracle,
  VanillaBreezeHookOracle, VanillaClientArrowOracle,
  VanillaZombieCureSoundOracle — "package net.minecraft.client.multiplayer
  does not exist".
- **B3-O03 — 26.x entity subpackages (7 oracles):** VanillaArrowOracle +
  VanillaLateEntityPairOracle (`projectile.arrow`), VanillaMobBreathingOracle
  (`animal.cow`), VanillaEntitySoundPacketOracle (`vehicle.minecart`),
  VanillaFluidBoxOracle (`vehicle.boat`), VanillaEffectSyncOracle
  (`monster.zombie`), VanillaSpawnRulesOracle (`util.debug`).
- **B3-O04 — `world.clock` package (2 oracles):**
  VanillaAttributeUpdatesOracle, VanillaActiveEffectTickOracle — "package
  net.minecraft.world.clock does not exist".
- **B3-O05 — missing-symbol 26.x APIs (15 oracles):**
  VanillaBoatAuthorityOracle (n=26), VanillaDragonContactsOracle (5),
  VanillaEntityFluidOracle (14), VanillaEntityMoveOracle (8),
  VanillaMobFallResetOracle (15), VanillaRegisteredEntityFluidOracle (2),
  VanillaShulkerBulletOracle (4), VanillaShulkerSpaceOracle (2),
  VanillaTradeComponentsOracle (7), VanillaVehicleContainerOracle (5),
  VanillaVillagerFarmingOracle (2), VanillaWorldSpawnOracle (8),
  VanillaWorldSpawnPacketOracle (2), EntityTranscript, LocatorBarClient (20).
- **B3-O06 — no-override (4 oracles):** VanillaEffectNotificationsOracle,
  VanillaMobAttributesOracle, VanillaBlockingItemDamageOracle,
  VanillaLivingFallOracle — "method does not override or implement a method
  from a supertype" (26.x methods absent in 1.21.7).

Adjacent, not Batch 3: VanillaTeleportConsumeOracle (`animal.fox`; Batch 7's
chorus-fruit area), VanillaAudioAttributesOracle +
VanillaEnvironmentAttributeModifierOracle (borderline attributes),
VanillaBlockEntity* / EntityAnim / EntityRender / EntityParticles (Batches
4/8/10), VanillaFallingBlockOracle (Batch 1).

## Drivers (D)

Curated Batch 3 driver list: 121 files (keyword cut over the 648
`tools/test_*_parity.py`, excluding other batches: block_entity→4,
entity_model/anim/render/sound→10, entity_particles→8, falling/fluid→1,
redstone/piston→2, command/scoreboard→9, item/inventory→7, worldgen→6, data
catalogs→12); ~109 core after removing borderline entries
(dedicated_server, scattered_ore, respawn/hardcore, frame_identity,
java_identity_map, turtle-egg block tests, dripleaf). No driver was run
(blocked); three driver-side defects found by reading (B3-G18–B3-G20).
Stale 26.3 docstrings only (no functional effect): dragon_contacts,
mob_discard, mob_quad_leash, vehicle_container, villager_farming.

## Method notes

- Vanilla 1.21.7's own jar/data is the spec throughout; where a behaviour
  was verified, the exact vanilla method + bytecode is cited in the notes
  above. The 26.3 base's form was never assumed (owner's vanilla-check
  rule).
- `javap` dumps and extracted classes were lost twice to `/tmp` wipes
  (service restarts); the second extraction went to a persistent scratch
  dir under the deliverables folder and was deleted before delivery.
- Disk quota blocked two note appends mid-session (transient; another agent
  freed space — disk went from 104G to 87G used). All findings were
  re-verified against the summary and are recorded in this report.
- Not re-diagnosed (established conclusions): the burn path matches vanilla
  exactly (651b43d3's list corrected — except B3-G08, which shows the
  corrected list itself omits Drowned); the no-mobs bug fix (dd966dea);
  MobAi's 27 gated scenarios (Batch 0, c9896b77).
- Noted, not defects: `MobData::placement` default OnGround vs vanilla
  NO_RESTRICTIONS (unreachable for 1.21.7 types); EnderDragon `_ => true`
  in `check_spawn_rules` (unreachable); ZombieHorse `removed_when_far`
  (dead arm — zombie horses never spawn naturally); IronGolem/Warden
  obstruction overrides (never spawn naturally); `spawner_blocks_work`
  game-rule gate (26.x rule absent in 1.21.7, default true → inert);
  hero-discount float-vs-f64 multiply (floors agree for all practical
  values); `DEFAULT_ATTACK_REACH` float-vs-double sqrt (~1e-8); port's
  `adjusted()` ignoring the /tick rate (nuance only under non-20 tick
  rates); finalize RNG draw order (values correct, streams diverge).
