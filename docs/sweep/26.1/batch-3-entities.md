# Batch 3 — Entities / mobs (26.1 parity sweep)

**Version:** 26.1 · **Date:** 2026-11-24 · **Branch:** `parity-sweep-26.1`
(workdir `~/workspace/mcir-work/sweep-26.1`) · **Scope:**
`docs/PARITY-SWEEP.md` §4 Batch 3 — mob AI, spawning, trading, attributes,
projectiles, vehicles; 157 entity types (44 monster, 31 creature, 72 misc,
4 water ambient, 3 water creature, 1 axolotl, 1 ambient, 1 underground
water creature); 144 drivers curated to 135 core (see §D).

**Status: checks + triage only, per the brief: no fixes, no edits under
`versions/26.1/`, no cargo builds, no driver runs, no oracle repairs, no
commits — the coordinator commits these deliverables.** Everything below is
static analysis: `javap -c` on the named 26.1 server-game jar
(`~/workspace/mcir-work/26.1.x/versions/26.1/jars/server-game.jar`,
DataVersion 4786, protocol 775, Java 25), source comparison against
`versions/26.1/crates/`, and checks of the prepared 26.1 data
(`versions/26.1/.local/horizon-client/26.1/`). `sounds.json` is genuinely
absent (separate step not run); `en_us.json` is present.

## Tally

- **Behaviour checks: 80 — 67 pass, 13 gap** (15 defect items: B3-G04,
  B3-G05, B3-G06, B3-G09, B3-G11, B3-G18, B3-G19, B3-G20, B3-G22, B3-G23,
  B3-G24, B3-G25, B3-G26, B3-G27, B3-G28 — the A3 food-table check yields
  three items; every other check yields one). Every other 1.21.7 Batch 3
  defect was re-checked and does NOT carry over — see "Re-checked, not
  defects".
- **Oracle defects: 8 groups** (B3-O01–B3-O08, 52 oracles — not counted in
  the check tally).
- **Oracle repairs done: none** (checks + triage only).
- **Severity: 1 major, 14 minor** — 1 major port (B3-G22, the
  `#causes_suffocation` startup blocker), 11 minor port (B3-G04, G05, G06,
  G09, G11, G23, G24, G25, G26, G27, G28), 3 minor driver-side (B3-G18,
  G19, G20). All three 1.21.7 majors (G14 enderman teleport, G16 no
  trades, G17 `reduced()`) do NOT carry over: 26.1 is a 26.x version, so
  the 26.x systems those defects turned on (blocksMotion-grounded
  teleport, data-driven trades, `reducedTickDelay`) exist in vanilla 26.1
  and the port matches them. Several other 1.21.7 "defects" turned out to
  be genuine 1.21.7↔26.1 vanilla differences (G02 environment attributes,
  G03 literal immune blocks, G08 tag-driven burn, G10 Builder default,
  G12 Enderman override, G21 iron spear).

Per-section counts (must sum to 80): A1 1 (1 pass), A1b 11 (11 pass),
A2 1 (1 pass), A3 9 (6 pass, 3 gap), A4 5 (4 pass, 1 gap), B1 3 (1 pass,
2 gap), B2 25 (24 pass, 1 gap), B7 3 (3 pass), B8 1 (1 pass), B5 3
(3 pass), B4 4 (3 pass, 1 gap), B6 3 (2 pass, 1 gap), B3 5 (4 pass,
1 gap), B9 3 (3 pass), Drivers 3 (0 pass, 3 gap).

## Defects

### B3-G04 — Hoglin wrongly disallowed in peaceful (minor, port)

Vanilla 26.1 `EntityType.<clinit>` marks 37 types `notInPeaceful`; hoglin
is NOT among them (verified directly in the dump: the HOGLIN builder chain
has no `notInPeaceful` call; GUARDIAN has one). The port's
`allowed_in_peaceful` disallows 38 types including hoglin, in BOTH
`mobs/spawn_rules.rs:134` and `mobs/finalize.rs:897`. Live paths:
`ai.rs:2305` checkDespawn, `game/commands.rs:2589`, `passengers.rs`,
`stations.rs`, `structure_mobs.rs` — hoglins despawn / refuse to spawn in
peaceful in the port where vanilla keeps them. (Opposite of 26.2, where
hoglin IS `notInPeaceful`.) Carried over from 1.21.7.

### B3-G05 — Camel has no food (minor, port)

Vanilla 26.1 `Camel.isFood` = `#camel_food` (cactus; the tag exists in the
jar's data). Port `food_tag()` (`mobs/species.rs:1641`) covers `CamelHusk`
but NOT `Camel`, so `MobType.food` is empty for camels and
feeding/breeding/tempting never fire. Camels cannot breed. Carried over
unchanged from 1.21.7 (and shared with the 26.2 port's identical table —
flag for the 26.2 record).

### B3-G06 — Armadillo scared head rotation ignored (minor, port)

Vanilla 26.1 `Armadillo.getMaxHeadYRot = isScared() ? 0 : 32`. Port
`max_head_y_rot` (`mobs/species.rs:1727`) returns 32.0 unconditionally, so
a scared (rolled-up) armadillo still turns its head. Carried over from
1.21.7.

### B3-G09 — Sun-burn ignores powder snow (minor, port)

Vanilla 26.1 `Mob.isSunBurnTick` excludes mobs that are `isInWaterOrRain
|| isInPowderSnow || wasInPowderSnow`. The port's `sun_burn_tick`
(`ai.rs:2024-2037`) uses `wet = in_water || in_rain` only, although `Mob`
HAS `in_powder_snow` state (`mobs.rs:1622`, updated `ai.rs:1404-1405`) —
undead standing in powder snow burn at noon in the port where vanilla
spares them. Carried over from 1.21.7.

### B3-G11 — `getRandomPosWithin` y-range off by one (minor, port)

Vanilla 26.1 `NaturalSpawner.spawnCategoryForChunk` computes
top = `getHeight(WORLD_SURFACE) + 1` and samples
y = `randomBetweenInclusive(minY, top)` → [minY, H+1]. The port's
`spawn_category_for_chunk` (`spawn_selection.rs:1115-1117`) uses top = H
and y = min + `nextInt(top-min+1)` → [minY, H], so the topmost spawnable
y is never chosen. (The port's `world_surface` = H is confirmed by
`horizon-client-render/src/world.rs:407`: "one above the highest non-air
block".) Carried over from 1.21.7.

### B3-G18 — DRIVER: entity_components predicate form (minor, driver)

`tools/test_entity_components_parity.py:27-29` reads
`criterion['conditions']['entity']['predicate']` and then
`predicate['minecraft:components']`. 26.1 advancement data uses the list
form with `predicate: {components: {...}}` — verified in all 3
advancements that use it (`complete_catalogue` 11 criteria, `whole_pack`
9, `leash_all_frog_variants` 3). The driver raises TypeError on the first
criterion. (26.2 uses the new form; 26.1 the old — the driver was written
for 26.2/26.3 data.) Carried over from 1.21.7; in 26.2 this check PASSED.

### B3-G19 — DRIVER: state-count hardcode 35723 (minor, driver)

Two Batch 3 drivers hardcode the 26.3 block-state count:
`test_suffocation_parity.py:66` and
`test_registered_entity_fluid_parity.py:66` both assert the state count
equals 35723. 26.1 has 29,873 states, so both drivers fail their own
completeness check. (In 26.2 the suffocation driver was fixed to 32366;
in 26.1 it was not.) 34 further drivers tree-wide hardcode 35723 — listed
in the slice's `out_of_scope_failures`, not defected here.

### B3-G20 — DRIVER: 26.3 resources path (minor, driver)

Five Batch 3 drivers build the server command with
`lab.LOCAL / 'horizon-client/26.3'`:
`test_locator_bar_parity.py:269`, `test_server_dragon_parity.py:111`,
`test_server_entities_parity.py:186`,
`test_server_knockback_parity.py:435`, `test_projectile_shoot.py:46` — so
they run the horizon server against 26.3 prepared data (or fail outright
where none exists) instead of 26.1's. 28 further drivers tree-wide use a
26.3 resources path — listed in the slice's `out_of_scope_failures`.

### B3-G22 — Default suffocation is tag-gated on absent `#causes_suffocation` (MAJOR, port)

Vanilla 26.1's default suffocation rule is `Properties.<init>` indy#6 →
`lambda$new$5` = `blocksMotion() && isCollisionShapeFullBlock()` — NO tag.
The port's default is `TaggedFullCollision`
(`horizon-world/src/suffocation.rs:72`) gated on `#causes_suffocation`, a
tag absent from 26.1 jar data (0 entries) and from prepared
`block-tags.json` (248 tags; the jar set == the prepared set modulo 4
`mineable/*` renames). `motion.rs:139` requires it via `tag(...)?`, which
errors on missing tags; `TagDefinitions::build` synthesizes nothing. The
startup path `resource_load.rs load_resources` → `load_core` (151) →
`gameplay_tables` (291) → `MotionResources::load_with_facts` (324)
propagates the error with `motion?` — **the 26.1 client cannot load its
core tables with prepared 26.1 data**. The first failure in evaluation
order is actually `#suppresses_bounce` (`motion.rs:138`, also absent;
1.21.7 hardcodes `{minecraft:honey_block}` for it at `motion.rs:144`),
then `#causes_suffocation`. Even if load passed, no default block would
suffocate. Contrast: 26.2's `motion.rs:65` documents "26.3 replaced this
with the `#causes_suffocation` tag, which 26.2 data has not" and uses
`blocks_motion` + `MotionFullCollision`. CI crate tests do not catch this:
data-dependent tests skip quietly without `.local` ("Skips quietly
without the vanilla data (CI has none)"). NEW in 26.1 — the 26.3-form tag
gate is a template-level leak; flag `#suppresses_bounce` (landing-area)
for the coordinator/other batches, since it is the first-failing tag.

### B3-G23 — Sleeping mobs suffocate (minor, port)

Vanilla 26.1 `LivingEntity.isInWall` = `!isSleeping() &&
super.isInWall()`; `Entity.isInWall` = noPhysics→false, eye box
`AABB.ofSize(eye, width*0.8, 1e-6, width*0.8)`, any block `!isAir &&
isSuffocating && collisionShape ∩ box`; `baseTick` hurts 1.0 InWall. The
port's `ai.rs base_tick` (1209) matches the geometry, damage, and Vex
exclusion but has NO sleeping check — a sleeper whose eye is inside a
suffocating shape takes in-wall damage in the port. Edge case. NEW.

### B3-G24 — Villager level-up is immediate and new offers are discounted (minor, port)

Vanilla 26.1 `rewardTradeXp` does NOT level up on trade: it sets
`updateMerchantTimer = 40` and `increaseProfessionLevelOnUpdate = true`
(pop += 5). `customServerAiStep` decrements the timer ONLY while not
trading (isTrading gate); at 0 it calls `increaseMerchantCareer` (level+1
+ updateTrades) — with NO `updateSpecialPrices` call, so new offers carry
`specialPriceDiff` 0 until the next session. The 26.1 port levels up
IMMEDIATELY in `Trader::notify_trade` (`merchant.rs:399-407`) and
recomputes special prices for ALL offers including the new ones, with the
session-start reputation/hero. Observable: newly-unlocked trades show
discounted prices mid-session in the port (full price in vanilla), and
the level-up appears at once instead of ~2s after trading ends. The 26.2
port has the correct deferred form (`level_up_pending`, `merchant.rs:413`
+ `trading.rs:333`, with the comment "the career increases only once the
villager is not trading and 40 ticks have passed") — the 26.1 port
diverged from it. No `level_up_pending` / `update_merchant_timer` exists
anywhere in the 26.1 port. NEW.

### B3-G25 — Furnace minecart too fast under `minecart_improvements` (minor, port)

Vanilla 26.1 `NewMinecartBehavior.getMaxSpeed` = `MAX_MINECART_SPEED *
(inWater ? 0.5 : 1.0) / 20.0` (rule default 8, min 1, max 1000; the port's
formula in `max_speed_new` (991-994) matches). But
`AbstractMinecart.getMaxSpeed` delegates to the behavior, and
`MinecartFurnace.getMaxSpeed` multiplies the result by 0.75 (wet) / 0.5
(dry) under EITHER behavior. Vanilla calls the VIRTUAL (furnace applies)
in `calculateTrackSpeed` (NewMinecartBehavior:1159) and
`AbstractMinecart.comeOffTrack` (AbstractMinecart:698), but the behavior's
OWN getMaxSpeed (no furnace) in `moveAlongTrack`'s step cap (1020). The
port uses `max_speed_new` (no furnace factor) at ALL three sites: step cap
(1069 — correct), calculateTrackSpeed cap (1208 — wrong) and comeOffTrack
(1344 — wrong). In a `minecart_improvements` world a furnace minecart is
capped at 0.4 dry / 0.2 wet instead of 0.2 / 0.15. Experiment-gated:
26.1 `DEFAULT_FLAGS` = VANILLA_SET only (`FeatureFlags.<clinit>`), so the
default (old-behavior) path is correct; the divergence is live only in
worlds with the experiment enabled. The 26.2 port has identical code at
all three sites. NEW.

### B3-G26 — BreedGoal ignores partner panic (minor, port)

Vanilla 26.1 checks the partner is NOT PANICKING in both places:
`getFreePartner` skips panicking candidates, and `canContinueToUse` is
`partner.isAlive() && partner.isInLove() && loveTime < 60 &&
!partner.isPanicking()`. The port's partner search and `can_continue`
omit the panic clause entirely (the `Seen` struct carries no panicking
field). Panic does not clear in-love in either implementation (the port
clears `in_love` only on age != 0 / countdown, `ai.rs:1593`), so the gap
is observable: hit one of two in-love animals and the other keeps
following it in the port; vanilla stops. Shared with the 26.2 port's
identical code (26.2's sweep passed B3 with it) — flag for the 26.2
record too. NEW.

### B3-G27 — Zombie horse has no food (minor, port)

Vanilla 26.1 `ZombieHorse.isFood` = `#zombie_horse_food` (red mushroom;
the tag exists in the jar's data). The port's `food_tag()` has no
`ZombieHorse` arm, so zombie horses cannot be fed for healing/temper in
the port. Shared with the 26.2 port's identical table — flag for the 26.2
record. NEW.

### B3-G28 — Skeleton horse has no food (minor, port)

Vanilla 26.1 `SkeletonHorse` extends `AbstractHorse` and does not override
`isFood`, so it inherits `#horse_food`. The port's `food_tag()` covers
`Horse | Donkey | Mule` but not `SkeletonHorse`, so skeleton horses cannot
be fed in the port. Shared with the 26.2 port's identical table — flag for
the 26.2 record. NEW.

## Re-checked, not defects

Thirteen 1.21.7 Batch 3 items were re-checked against 26.1's own jar and
do NOT carry over. (G07 was withdrawn in 1.21.7 — not re-minted.)

- **G01 bat:** no Halloween branch in 26.1 vanilla; the port matches
  (y < surface, `nextBoolean`, brightness > `nextInt(4)`,
  `#bats_spawnable_on`, mob rules).
- **G02 slime:** vanilla uses the environment attribute
  `SURFACE_SLIME_SPAWN_CHANCE` with a single `nextFloat`; underground:
  slime-chunk seed 987234911, `nextInt(10)==0`, y < 40 — the port matches.
- **G03 immune_tag:** the port's `immune_to` uses the literal block sets
  matching 26.1 vanilla (fox→sweet_berry_bush;
  polar_bear/snow_golem/stray→powder_snow; wither/wither_skeleton→
  wither_rose), with a test at `spawn_rules.rs:855`. 26.1 uses the
  1.21.7 literal form, not 26.2's tags.
- **G08 drowned burns:** tag-driven; `#burn_in_daylight` holds exactly 10
  types including drowned, and the port's list matches.
- **G10 far spawns:** the `EntityType.Builder` default is
  `canSpawnFarFromPlayer = (category == CREATURE || category == MISC)`;
  the port's `spawns_far_from_player` = Creature|Misc + Pillager|Shulker
  matches (pillager and shulker are the only explicit setters).
- **G12 enderman walk-target:** 26.1 `EnderMan` DOES override
  `getWalkTargetValue`, returning 0.0f; the port matches.
- **G13 constant-count RNG:** the port's `sample_count` draws
  unconditionally (`spawn_selection.rs:798-801`).
- **G14 enderman teleport:** vanilla `EnderMan.teleport` +
  `LivingEntity.randomTeleport` ground on `blocksMotion()`; no teleport
  tags exist in 26.1 data (verified by jar listing); the port's
  `enderman.rs` `teleport_to`/`random_teleport` use blocksMotion, event
  46, nav.stop.
- **G15 cat morning gift:** `tame.rs:1353` uses
  `enderman::random_teleport`; only doc-comment tag mentions remain.
- **G16 trades:** the jar has 561 `villager_trade` + 83 `trade_set` + 102
  tag files; prepared data 387 + 68 + 73; the port's `Trades::load`
  (`merchant.rs:482`) reads all three dirs.
- **G17 `reduced()`:** 26.1 `Goal` HAS `reducedTickDelay` =
  `Mth.positiveCeilDiv(ticks, 2)` and `adjustedTickDelay`; the port's
  `goals.rs:66-71` matches.
- **G21 zombie spear:** vanilla `Zombie.populateDefaultEquipmentSlots`:
  `nextFloat < (hard ? 0.05 : 0.01)`, `nextInt(6)`: 0→IRON_SWORD,
  1→IRON_SPEAR, else IRON_SHOVEL; `iron_spear` exists in 26.1 items; the
  port matches in `spawning.rs:222` and `finalize.rs:1347`.
- **G07:** withdrawn in 1.21.7 — not re-minted.

## Passes by section

### A1 — Spawn placements (1 check, PASS)

82 vanilla `SpawnPlacements.register` calls (ON_GROUND 59, IN_WATER 12,
NO_RESTRICTIONS 10, IN_LAVA 1; heightmaps MOTION_BLOCKING_NO_LEAVES 80,
MOTION_BLOCKING 2: ocelot, parrot), 38 distinct predicates (resolved via
`javap -v` BootstrapMethods). Full table in
`evidence/spawn-placements-vanilla.json`. Versus `mob-data.json` (157
entries): 0 mismatches; the 75 unregistered types all default to
`no_restrictions`.

### A1b — Spawn predicates (11 checks, all PASS)

Bat, slime, camel, drowned (including roll order: frequent→
`nextInt(15)==0`, else `nextInt(40)==0` && y < sea−5; spawner/reinforcement
early return), hoglin/piglin (below ≠ nether_wart_block only), guardian,
ghast, magma cube, zombified piglin, monster rules (`!peaceful &&
(ignoresLight || darkEnough) && mobRules`; surface form adds `isSpawner ||
canSeeSky`), mob rules, animal rules (`#animals_spawnable_on` + bright),
`isDarkEnoughToSpawn` (sky > `nextInt(32)`; block-light limit when < 15;
thundering darken 10 else sky_darken; `monsterSpawnLightTest.sample`),
bright = raw brightness > 8.

### A2 — Species coverage (1 check, PASS)

The port has 90 `Species` enum variants = 89 vanilla types + dead
`sulfur_cube`; 90 `Def` entries in `species.rs` `ALL` (line 141), all
matching the enum. Attribute suppliers missing from the port are only
armor_stand, ender_dragon, player (handled separately by design).
Mannequin is a real 26.1 type (misc) with a supplier and a Def. (The port
still carries dead SulfurCube arms, e.g. `spawn_rules.rs:440`,
`spawn_selection.rs:149/584` — noted, not defects.)

### A3 — Attributes + species tables (9 checks, 6 pass, 3 gap)

- **Attributes registry (PASS):** `Attributes.<clinit>` parsed to
  `evidence/attributes-vanilla.json`: 35/35 match prepared
  `attributes.json` (default/min/max/syncable). Note the vanilla field
  `SPAWN_REINFORCEMENTS_CHANCE` registers id `spawn_reinforcements`.
- **Entity attribute values (PASS):** 2,131 entity attribute values
  identical between `attributes.json[entities]` and `entity-types.json`.
- **Suppliers (PASS):** 92 `DefaultAttributes` suppliers = 92 prepared
  entities.
- **Categories (PASS):** match the inventory exactly (44/31/72/4/3/1/1/1).
- **Fire-immune (PASS):** 15 types, matching `entity-types.json` (the
  port's 11 mob types + 4 non-mobs: area_effect_cloud, end_crystal,
  ender_dragon, tnt).
- **Head rotation (GAP — B3-G06):** Mob defaults Y75/X40/rotSpeed 10; all
  overrides verified (Axolotl/Dolphin 1/1, Frog Y5/rotSpeed 35, Breeze
  Y30/rotSpeed 25, Camel Y30, Goat Y15, Ravager Y45, Sniffer Y50, Shulker
  180/180, Guardian X180, Slime X0) — except the armadillo scared case.
  Sitting-wolf X=20 IS handled at Mob level (`control.rs`
  `Mob::max_head_x_rot`).
- **Ambient intervals (PASS):** all match (Turtle 200, AbstractHorse 400,
  Ocelot 900, Guardian 160, Animal/WaterAnimal/AgeableWaterCreature/
  AbstractGolem/Cat 120, default 80; HappyGhast 120 ×6 when vehicle —
  handled at `mobs.rs:4676`).
- **Food tags (GAP — B3-G05, B3-G27, B3-G28):** complete `isFood`
  declarer set from the jar (31 classes; `Animal.isFood` is abstract).
  All match the port's `food_tag()` except Camel (`#camel_food`),
  ZombieHorse (`#zombie_horse_food` = red mushroom), SkeletonHorse
  (inherits `#horse_food`). PolarBear returns false in vanilla and has no
  arm in the port — correct. Full table in
  `evidence/a3-food-table.md`.
- **Peaceful-allowed table (GAP — B3-G04):** full-table comparison — 37
  vanilla `notInPeaceful` types vs the port's 38 (the extra one is
  hoglin). `EntityType.<clinit>` parsed to
  `evidence/entity-type-flags.json` (157 entries): fire_immune 15;
  `canSpawnFarFromPlayer` explicit setters only pillager + shulker;
  `immuneTo` 6 types with LITERAL blocks (the 1.21.7 form, not 26.2's
  tags); `spawnDimensionsScale` 4.0 only SLIME and MAGMA_CUBE; `noSummon()`
  only PLAYER and FISHING_BOBBER; Builder default `summon=true`.

### A4 — Daylight burning (5 checks, 4 pass, 1 gap)

- **Burn tag (PASS):** `Mob.aiStep` calls `burnUndead()` iff the type is
  in `#burn_in_daylight`; the tag holds exactly 10 types including
  drowned (jar `data/minecraft/tags/entity_type/burn_in_daylight.json`);
  the port's `burns_in_day` list (`species.rs:1551`) matches exactly.
- **burnUndead mechanics (PASS):** helmet damage `nextInt(2)`, break at
  max damage, else `igniteForSeconds(8.0)`; the port ignites 160 ticks.
- **sunProtectionSlot (PASS):** Mob HEAD; ZombieHorse/ZombieNautilus
  BODY — the port's `body_slot_species` (`drops.rs:186`) covers both.
- **isSunBurnTick (GAP — B3-G09):** powder-snow exclusion missing in the
  port (see Defects).
- **aiStep burn gate (PASS):** the tag check gates the burn call in both.

### B1 — Suffocation (3 checks, 1 pass, 2 gap)

Evidence: `evidence/b1-suffocation.md`.

- **Override table (PASS):** 15 `Blocks` `isSuffocating` calls resolved
  (BootstrapMethods: indy#8 = `Blocks.never`, indy#118 = `Blocks.always`).
  Always: DIRT_PATH, FARMLAND, MUD, SOUL_SAND. Never: MANGROVE_ROOTS,
  MOVING_PISTON, CHERRY_LEAVES, PALE_OAK_LEAVES, GLASS, TINTED_GLASS,
  COPPER_GRATE + `registerStainedGlass` helper + `leavesProperties`
  helper. NOT_CLOSED_SHULKER via `shulkerBoxProperties`;
  NOT_EXTENDED_PISTON via `pistonProperties`. The port's
  `mobs/suffocation.rs::bind` matches exactly.
- **Default rule (GAP — B3-G22, major):** see Defects.
- **In-wall (GAP — B3-G23):** see Defects.

### B2 — Natural spawning (25 checks, 24 pass, 1 gap)

Evidence: `evidence/b2-natural-spawning.md`. All verified against
`NaturalSpawner`, `SpawnPlacements`, `SpawnPlacementTypes`, `MobCategory`,
`EntitySpawnReason`, `PotentialCalculator`, `LocalMobCapCalculator`,
`ServerChunkCache` dumps and the 18 walk-target / 13 obstruction / 20
remove-far declarer dumps (complete sets from the class-file scan).

- **MobCategory values (PASS):** max/friendly/persistent/despawn;
  `getNoDespawnDistance` = 32 const.
- **`spawnCategoryForPosition` order (PASS):** redstone early return, 3
  attempts, `ceil(nextFloat*4)` reassigned by `sample_count`,
  `nextInt(6)-nextInt(6)` jitter, nearest-player, right-distance, pick,
  `isValidSpawnPostitionForType`, predicate, getMobForSpawn null→return,
  yaw draw, `isValidPositionForMob`, finalizeSpawn, cluster-size return,
  `isMaxGroupSizeReached` break — including RNG draw order.
- **`spawnCategoryForChunk` y-range (GAP — B3-G11):** see Defects.
- **`getRandomSpawnMobAt` (PASS):** WATER_AMBIENT +
  `#reduce_water_ambient_spawns` + `nextFloat < 0.98`.
- **`mobsAt` / fortress bounds (PASS):** FORTRESS_ENEMIES (blaze 10/2-3,
  zombified_piglin 5/4-4, wither_skeleton 8/5-5, skeleton 2/5-5,
  magma_cube 3/4-4).
- **`right_distance` (PASS):** d≤576 reject; respawn-centre <24 reject,
  dimension-gated; same chunk or `canSpawnEntitiesInChunk`.
- **`valid_position_for_type` (PASS):** MISC reject; far/despawn;
  canSummon moot; canSpawnMobAt — the port's `listed` optimisation is
  sound; isSpawnPositionOk; checkSpawnRules; noCollision on getSpawnAABB.
- **`isValidPositionForMob` (PASS).**
- **Walk-target table (PASS):** all 18 declarers read; cost =
  magic−0.5, magic = `lerp(ambient, v/(4−3v), 1)`, v=brightness/15 — the
  port's `magic_light` is exact; fresh-mob state notes hold (hoglin no
  repellent memory, strider not in lava, turtle `!goingHome`).
- **Obstruction table (PASS):** all 13 declarers; Ocelot = unobstructed
  && !liquid && y≥sea && below grass_block/#leaves; Ravager = !liquid
  only; ZombifiedPiglin = Mob default; IronGolem/Warden custom but never
  spawn naturally.
- **`max_cluster_size` (PASS):** Mob 4, AbstractHorse 6, AbstractFish 8,
  AbstractSchoolingFish→`getMaxSchoolSize` with Salmon 5, Wolf 8,
  Ghast/HappyGhast/Pillager 1; Tadpole 8 via AbstractFish, Camel/CamelHusk
  6 via AbstractHorse.
- **`isMaxGroupSizeReached` (PASS):** Mob false; TropicalFish `!isSchool`.
- **`removeWhenFarAway` (PASS):** all 20 declarers read; Cat `!tame &&
  tickCount>2400`, Ocelot `!trusting && tickCount>2400`, Chicken
  `isChickenJockey`, PatrollingMonster `!patrolling || d>16384`, Raider
  in-raid→false, ZombieVillager `!converting && xp==0`, AbstractFish/
  Axolotl `!fromBucket && !hasCustomName`. The port's `check_despawn`
  helpers (`raider::keeps_alive`, `zombie_villager::keeps_alive`,
  `feline_removed_when_far` on Entity.tickCount `m.age: u32` — note there
  are two `age` fields: `mobs.rs:1115` AgeableMob.age i32 vs `mobs.rs:1561`
  Mob age u32 tick count; `ai.rs` uses the latter) and spawn-time
  `removed_when_far` both match. ZombieHorse/CamelHusk are MONSTER
  category, so the port's creature-animal clause does not wrongly keep
  them (a hypothesised divergence was disproved by checking
  `entity-types.json` categories).
- **`checkDespawn` (PASS):** peaceful gate; persistent→noActionTime=0; no
  player→return; far-despawn; noActionTime>600 && `nextInt(800)==0` &&
  d>32² && removeWhenFarAway; d<32²→reset; WitherBoss override = peaceful
  gate only. (The port's order difference — nearest-player check before
  persistent reset — is unobservable since noActionTime is only read when
  !persistent.)
- **`isValidSpawn` block table (PASS):** 34 Blocks calls resolved
  (indy#7=`Blocks.never`, indy#9=`Blocks.ocelotOrParrot`,
  indy#187=lambda$static$129 ICE=polar-bear-only, indy#195=`Blocks.always`,
  indy#449=lambda$static$321 FROSTED_ICE=polar-bear-only,
  indy#452=lambda$static$323 MAGMA_BLOCK=`fireImmune()`; never =
  bedrock/glass/tinted/moving_piston/all trapdoors/barrier/chorus_flower/
  scaffolding/copper_grate/all stained glass; leaves = ocelotOrParrot;
  always = soul_sand/carved_pumpkin/jack_o_lantern/redstone_lamp/mud;
  default = face-sturdy-UP && light<14). Prepared mob-data flags sampled
  for 24 blocks — all consistent (dirt_path/farmland correctly NOT valid:
  15/16-high top face not sturdy; blocks valid for all three type classes
  carry all three bits, which is correct, not a defect).
- **`isSpawnPositionOk` (PASS):** `SpawnPlacementType` is an INTERFACE in
  26.1; NO_RESTRICTIONS always true with no border check;
  IN_WATER/IN_LAVA/ON_GROUND border-gated.
- **`isValidEmptySpawnBlock` / `isBlockDangerous` / `isBurningBlock`
  (PASS):** burning = #fire || lava || magma || lit campfire ||
  lava_cauldron; dangerous = !immuneTo && ((!fireImmune && burning) ||
  wither_rose/sweet_berry/cactus/powder_snow). The port's zombie-flag +
  recompute split is sound (no block is both burning and a signal source).
- **`getSpawnAABB` / `spawnDimensionsScale`, `canSummon`,
  `EntitySpawnReason` (PASS):** isSpawner = SPAWNER|TRIAL_SPAWNER;
  ignoresLightRequirements = TRIAL_SPAWNER only.
  `SpawnPlacements.checkSpawnRules` has NO peaceful gate in 26.1 (26.2
  added one — the port's doc comment is correct).
- **`SpawnState` / `createState` (PASS):** skip persistent/
  custom-persistent mobs and MISC; `Counted.persistent` = m.persistent ||
  requires_custom_persistence.
- **`PotentialCalculator` (PASS):** charge 0 skipped; energy = Σ(c/dist,
  ∞ at 0) × charge; canSpawn energy ≤ budget; afterSpawn last-charge
  reuse.
- **`getFilteredSpawningCategories` / local cap (PASS):** vanilla caller
  `ServerChunkCache` passes (true, spawnEnemies, gameTime%400==0); global
  cap max×chunks/289 with MAGIC_NUMBER = 17²; local cap = any near player
  under max; players-near = 8-chunk tracker, non-spectators within 128 of
  chunk centre; the port calls `filtered_categories(spawn_enemies,
  game_time % 400 == 0)`.
- **`spawnMobsForChunkGeneration` + `getTopNonCollidingPos` (PASS):**
  creature list, SPAWN_MOBS rule, `nextFloat < creatureProbability` loop,
  4 attempts with 5-jitter — noCollision/checkSpawnRules/create failures
  SKIP the jitter, obstruction/mob-rules failures and success jitter,
  out-of-chunk re-roll from start; top pos = heightmap, ceiling descent
  air-seeking minY-bounded, ON_GROUND adjust pathfindable-LAND-below →
  y−1.
- **Spawn-rules tables (PASS):** `checkSpawnRules` declarer set (Mob,
  PathfinderMob, SpawnPlacements), `checkDespawn` declarer set (5),
  `getMaxHeadYRot`/`getMaxHeadXRot` declarer sets — all from the
  class-file scan, all consistent with the port.

### B7 — Spawners (3 checks, all PASS)

Evidence: `evidence/b7-spawners.md`.

- **BaseSpawner defaults + serverTick (PASS):** vanilla ctor spawnDelay
  20, minSpawnDelay 200, maxSpawnDelay 800, spawnCount 4,
  maxNearbyEntities 6, requiredPlayerRange 16, spawnRange 4; the port
  (`spawner.rs:317,350-355`) is identical. `delay()`: max<=min → min, else
  min + nextInt(max-min); the port (448-453) is identical. serverTick
  order verified: !nearPlayer || !isSpawnerBlockEnabled → return;
  delay==-1 → delay(); delay>0 → decrement, return; else spawnCount
  attempts: unknown type → delay + return; pos = stored Pos or
  x+(d-d)*range+0.5 / y+nextInt(3)-1 / z+(d-d)*range+0.5; noCollision
  (spawnAABB) else continue; custom rules present → peaceful check
  (!friendly && peaceful → continue) + isValidPosition, else
  checkSpawnRules(SPAWNER); loadEntityRecursive null → delay + return;
  exact-class count in AABB(pos,pos+1).inflate(spawnRange) >= maxNearby →
  delay + return; yaw draw nextFloat*360; Mob: checkSpawnRules skipped iff
  custom rules present, but checkSpawnObstruction ALWAYS checked
  (bytecode offsets 508-551: isEmpty?→537 skips only the rules check) —
  the port's `rules_ok = custom.is_some() || mob_spawn_rules(...)` +
  always spawn_obstruction matches exactly; finalizeSpawn(SPAWNER, null)
  iff unconfigured (tag size==1 && has id); equipment;
  tryAddFreshEntityWithPassengers fail → delay + return; else levelEvent
  2004 + ENTITY_PLACE + spawnAnim (mobs); any spawn → delay().
- **TrialSpawnerConfig + FullConfig defaults (PASS):** vanilla Builder
  defaults spawnRange 4, totalMobs 6.0, simultaneousMobs 2.0,
  totalMobsAddedPerPlayer 2.0, simultaneousMobsAddedPerPlayer 1.0,
  ticksBetweenSpawn 40, loot tables consumables + key, ominous drop
  table; the port's `TrialConfig::default`
  (`horizon-client-render/src/chunk/block_entity.rs:636-655`) is
  identical. FullConfig codec defaults target_cooldown_length 36000,
  required_player_range 14; the port (`trial_spawner.rs:537-538`) is
  identical. MAX_MOB_TRACKING_DISTANCE_SQR = 47² (port const 47).
  DETECT_PLAYER_SPAWN_BUFFER = 40 (port: next_mob_spawns_at =
  game_time+40).
- **Trial spawnMob + state machine + canSpawnInLevel (PASS):** vanilla
  spawnMob (unknown type → empty; pos = stored or same jitter formula
  with activeConfig().spawnRange; noCollision + inLineOfSight (VISUAL
  clip, fluid NONE); checkSpawnRules(TRIAL_SPAWNER); custom rules →
  isValidPosition; loadEntityRecursive; Mob.checkSpawnObstruction;
  finalizeSpawn iff unconfigured; setPersistenceRequired; equipment;
  tryAddFreshEntityWithPassengers) matches the port's `spawn_mob`
  (`trial_spawner.rs:1029-1113`); the yaw draw position differs trivially
  (before obstruction vs after load) with no RNG consumer between.
  canSpawnInLevel = spawner_blocks_work && (override || (!peaceful &&
  spawn_mobs)); the port omits the override — NOT a defect: a jar-wide
  constant-pool scan finds NO caller of
  `overridePeacefulAndMobSpawnRule()` in either 26.1 jar (dead in vanilla
  play); the 26.2 port has the same form. tryDetectPlayers cadence
  `(pos.asLong() + gameTime) % 20 != 0` → return; the port
  `packed.wrapping_add(l.game_time()) % 20 != 0` is identical.
  Cooldown+ominous → return; Bad Omen → Trial Omen transform + levelEvent
  3020 at eye pos; WAITING state machine matches the port.

### B8 — Combat (1 check, PASS)

Vanilla `CombatRules.getDamageAfterAbsorb`: divisor 2+toughness/4;
protection = clamp(armor − damage/divisor, armor*0.2, 20.0);
effectiveness = protection/25; if weaponItem != null && level is
ServerLevel: effectiveness =
clamp(EnchantmentHelper.modifyArmorEffectiveness(...), 0, 1); return
damage*(1−effectiveness). `getDamageAfterMagicAbsorb`: protection clamped
[0,20]; damage*(1−protection/25). The port's
`horizon-world/src/combat_rules.rs` `after_armor`/`after_magic` match
exactly (including the NaN/−0.0-aware clamp mirroring Mth.clamp).

### B5 — Projectiles (3 checks, all PASS)

Evidence: `evidence/b5-projectiles.md`.

- **Gravities (PASS):** AbstractArrow 0.05, ThrowableProjectile 0.03,
  LlamaSpit 0.06, ThrownExperienceBottle 0.07 (`throwableitemprojectile`
  pkg), AbstractThrownPotion 0.05, ShulkerBullet 0.04. The port
  (`projectiles.rs:400-412`) matches exactly; hurting projectiles 0.0
  (their tick applies acceleration, never gravity).
- **Hurting-projectile inertia + acceleration (PASS):** vanilla
  AbstractHurtingProjectile INITAL_ACCELERATION_POWER 0.1, getInertia
  0.95, getLiquidInertia 0.8; applyInertia = (v + normalize(v)*
  accelerationPower) * inertia, inertia = inWater? liquid: normal; 4
  bubble particles at pos − movement*0.25 in water. AbstractWindCharge:
  getInertia 1.0, getLiquidInertia = getInertia, all three ctors set
  accelerationPower 0.0. WitherSkull: dangerous? 0.73: super. The port:
  `Projectile::new` sets acceleration 0.0 for wind charges, 0.1 otherwise
  (`projectiles.rs:451-459`); `apply_hurting_inertia` (1142-1160) matches
  the formula and the 0.73/1.0/0.8/0.95 table; `water_bubble_at` matches
  the 0.25 offset. Note (not a defect): the port floors |v| at 1e-9 where
  vanilla Vec3.normalize returns ZERO below 1e-4 — only differs for
  sub-1e-4 speeds, unreachable in practice; the same code passed in 26.2.
- **Trident water inertia (PASS):** vanilla ThrownTrident.getWaterInertia
  = 0.99, AbstractArrow default 0.6 (both verified in dumps). The port
  (`arrows.rs:776`): trident 0.99, else 0.6.

### B4 — Trading (4 checks, 3 pass, 1 gap)

Evidence: `evidence/b4-trading.md`.

- **cost_a_count (PASS):** vanilla `getModifiedCostCount`: base + max(0,
  floor(base*demand*priceMultiplier)) + specialPriceDiff, clamped [1,
  maxStackSize]. The port (`merchant.rs:164-171`) is identical (int
  product base*demand first, then f32 multiply — matches the bytecode
  `imul; i2f; fmul; floor; max(0,...)`).
- **LEVEL_XP (PASS):** vanilla 26.1 thresholds live in
  `VillagerData.NEXT_LEVEL_XP_THRESHOLDS` = [0, 10, 70, 150, 250];
  getMinXpPerLevel(l) = canLevelUp? T[l-1]: 0, getMaxXpPerLevel(l) =
  canLevelUp? T[l]: 0, canLevelUp = 1 <= l < 5. The port
  (`merchant.rs:16,22,29`) is identical, with a doc citing the 26.1 class.
- **updateDemand (PASS):** vanilla demand += uses − (maxUses − uses). The
  port (`merchant.rs:185-187`) is identical.
- **updateSpecialPrices + level-up deferral (GAP — B3-G24):** see Defects.
  Notes on the parts that match: vanilla updateSpecialPrices itself
  matches — reputation term
  `addToSpecialPriceDiff(-floor(reputation * priceMultiplier))` (note the
  `ineg` at offset 51; gossip values MAJOR_NEGATIVE −5, MINOR_NEGATIVE
  −1, MINOR_POSITIVE +1, MAJOR_POSITIVE +5, so positive reputation =
  liked = cheaper; the port subtracts the same floor,
  `merchant.rs:113`). Hero term: −max(1, floor((0.3 + 0.0625*amp) *
  baseCount)) in DOUBLE arithmetic; the port computes the modifier in f32
  then widens — brute-forced all amp 0..255 × count 1..64: 0 mismatches
  for amp ≤ 11 (legitimate max is 4); first mismatch at amp 12, counts
  20/40/60 (commands-only, 1 emerald). NOT minted: unreachable in normal
  play. `resetSpecialPrices` (server-side, all offers) is called from
  `stopTrading`; `updateSpecialPrices` only from `startTrading` — the port
  resets inside update_special_prices, equivalent on the session-start
  path. Trade-gossip timing is fine in the port: last_traded_player is
  consumed deferred in `ai.rs:2090`, matching vanilla's
  customServerAiStep TRADE event.

### B6 — Vehicles (3 checks, 2 pass, 1 gap)

Evidence: `evidence/b6-vehicles.md`.

- **Boat floatBoat + controlBoat (PASS):** vanilla AbstractBoat
  .getDefaultGravity = 0.04. floatBoat verified in full: vspeed =
  −getGravity() (0 with NoGravity); buoyancy 0; friction 0.05 default.
  Water-entry (oldStatus IN_AIR, status not IN_AIR/ON_LAND): waterLevel =
  getY(1.0); target = (float)(getWaterLevelAbove() − getBbHeight()) +
  0.101; if the box moved to target is unobstructed: setPos(x, target,
  z), zero Y velocity, lastYd = 0; status = IN_WATER; skips the friction
  block. IN_WATER: buoyancy = (waterLevel − y)/bbHeight, friction 0.9.
  UNDER_FLOWING_WATER: vspeed −7.0E-4, friction 0.9. UNDER_WATER:
  buoyancy (double)0.01f, friction 0.45. IN_AIR: friction 0.9. ON_LAND:
  friction = landFriction; if controlling passenger instanceof Player:
  landFriction /= 2 (persisted; the tick uses the pre-halved value).
  Final: v = (vx*f, vy+vspeed, vz*f); deltaRotation *= f; if buoyancy >
  0: vy = (vy + buoyancy*(0.04/0.65))*0.75. The port's `tick_boat`
  (`boats.rs:728-892`) matches all of it (HEIGHT 0.5625; buoyancy term
  hardcodes 0.04 = getDefaultGravity). controlBoat: deltaRotation −1/+1
  for left/right; right!=left && !up && !down adds 0.005 acceleration; up
  +0.04, down −0.005 along facing; paddling flags — all match. NOTE (not
  minted): vanilla getWaterLevelAbove scans y from floor(maxY) to
  ceil(maxY − lastYd) — the fall speed extends the scan upward; the port
  has no lastYd and scans one row, returning (y1+1) when full. A
  fast-falling boat's entry snap target can sit lower than vanilla's for
  one tick; buoyancy lifts it to the surface afterward. The 26.2 port is
  byte-identical here and passed B6. The port also halves landFriction on
  `controlled` (local driver) where vanilla checks the controlling
  passenger is a Player — same in local play, remote-only nuance.
- **Old minecart max speed (PASS):** vanilla OldMinecartBehavior
  .getMaxSpeed: inWater? 0.2: 0.4; getSlowdownFactor: isVehicle? 0.997:
  0.96. MinecartFurnace.getMaxSpeed: super * (inWater? 0.75: 0.5) — the
  override is on the cart, applied through the virtual. The port's
  `max_speed` (`minecarts.rs:750-757`) matches, and the old path uses it
  for the track cap (874) and comeOffTrack (1346).
- **New minecart max speed (GAP — B3-G25):** see Defects.

### B3 — Mob AI (5 checks, 4 pass, 1 gap)

Evidence: `evidence/b3-mob-ai.md`.

- **serverAiStep alternation (PASS):** vanilla: noActionTime++;
  sensing.tick(); if ((tickCount + id) % 2 == 0 || tickCount <= 1) {
  targetSelector.tick(); goalSelector.tick(); } else {
  targetSelector.tickRunningGoals(false);
  goalSelector.tickRunningGoals(false); }; navigation.tick();
  customServerAiStep; move/look/jump controls. The port (`ai.rs:2039+`)
  matches: the negated condition `id_based % 2 != 0 && tick_count > 1`
  for the running-only branch, targets before goals, nav → custom →
  move/look/jump order, no_action_time first. Sensing is a per-tick cache
  cleared in the mob tick (`ai.rs:649`), equivalent to Sensing.tick().
  The port's ridden-mount early return parallels vanilla's
  updateControlFlags disabling MOVE/JUMP/LOOK when a controlling passenger
  rides (not minted: same observable effect — no goal starts).
- **MeleeAttackGoal (PASS, with note):** canUse (20-tick gate on gameTime,
  target alive, createPath(target, 0) non-null OR within melee range, path
  stashed), canContinueToUse (target alive; !followUnseen → !nav.isDone;
  follow branch → isWithinHome(TARGET.blockPosition()) + refuse
  spectator/creative players), start (moveTo path, aggressive, recalc 0,
  attack 0), tick (look 30/30; recalc = max(−1, 0); re-path iff
  (followUnseen || hasLineOfSight) && recalc <= 0 && (pathed == (0,0,0)
  || moved >= 1.0 sq || nextFloat < 0.05); recalc = 4 + nextInt(7), +10
  if d > 1024 else +5 if d > 256, +15 if moveTo fails, adjustedTickDelay;
  attack timer max(−1, 0); checkAndPerformAttack iff time-to-attack &&
  in range && line of sight: reset = adjustedTickDelay(20), swing,
  doHurtTarget). adjustedTickDelay = requiresUpdateEveryTick? t:
  positiveCeilDiv(t, 2); the port's `adjusted`/`reduced` match (G17). All
  match the port's MeleeAttack (`goals.rs:1346+`). NOTE (not minted):
  vanilla stop() clears the target when !NO_CREATIVE_OR_SPECTATOR
  .test(target) — i.e. creative OR spectator players
  (lambda$static$3: `!(entity instanceof Player) || (!isSpectator &&
  !isCreative)`; test(null) = true); the port clears only creative
  players (`goals.rs:1450-1457`). The 26.2 port is identical and passed
  B3; the stale spectator target is overwritten by the next
  target-selector tick, so the effect is a transient stale pointer.
- **BreedGoal (GAP — B3-G26):** see Defects. The matching parts: canUse
  (in love; getFreePartner: nearest canMate candidate within the box
  inflated 8.0, PARTNER_TARGETING, not panicking), tick (look
  10/maxHeadXRot; moveTo partner; loveTime++; breed iff loveTime >=
  adjustedTickDelay(60) = reduced(60) = 30 — no requiresUpdateEveryTick
  override — && distance² < 9.0), stop (partner null, loveTime 0) all
  match the port (`goals.rs:2160+`; `nearby()` sorts by distance, so
  `.next()` is the nearest; species-specific canMate gates — wolf/cat
  tame, mule never, turtle egg, sniffer calm, equine/camel canParent —
  are all present).
- **Goal tick delays / G17 re-check (PASS):** 26.1 `Goal` HAS
  reducedTickDelay = positiveCeilDiv(ticks, 2) and adjustedTickDelay; the
  port's `goals.rs:66-71` matches. (1.21.7 G17 does not carry over.)
- **Enderman teleport + cat morning gift / G14/G15 re-check (PASS):** 26.1
  `EnderMan.teleport` + `LivingEntity.randomTeleport` ground on
  `blocksMotion()`; no teleport tags exist in 26.1 data; the port's
  `enderman.rs` `teleport_to`/`random_teleport` use blocksMotion, event
  46, nav.stop. Cat gift uses `enderman::random_teleport`
  (`tame.rs:1353`). (1.21.7 G14/G15 do not carry over.)

### B9 — Finalize spawn (3 checks, all PASS)

Evidence: `evidence/b9-finalize.md`.

- **Piglin finalize (PASS):** vanilla `Piglin.finalizeSpawn`: if reason !=
  STRUCTURE: `nextFloat < 0.2` → setBaby(true) (float drawn from the
  level random for ALL non-structure spawns); if isAdult AFTER the roll:
  setItemSlot(MAINHAND, createSpawnWeapon()). createSpawnWeapon (the
  entity's own random): `nextFloat < 0.5` → CROSSBOW, else `nextInt(10)
  == 0` → GOLDEN_SPEAR else GOLDEN_SWORD. Then `PiglinAi.initMemories`
  (HUNTED_RECENTLY = `TimeUtil.rangeOfSeconds(30, 120)` = 600–2400
  ticks), `populateDefaultEquipmentSlots` (adult: `maybeWearArmor` per
  golden piece, `nextFloat < 0.1f`), `populateDefaultEquipmentEnchantments`,
  `super.finalizeSpawn` (AbstractPiglin). The port (`finalize.rs:1702`)
  matches at every step: the baby roll is drawn unconditionally
  (non-structure), the weapon is given iff the piglin is adult after the
  roll, the weapon draws come from `m.random`, the hunt is
  `self.uniform(600, 2400)`, populate is adult-only with per-piece
  `self.float() < 0.1`, and hunt/populate/enchant/mob sit OUTSIDE the
  reason check exactly as in vanilla (bytecode offsets 8–12: `if (reason
  == STRUCTURE) goto 54` skips the whole baby+weapon block; initMemories
  at 54, populate 64, enchant 71, super 79). An earlier mid-session
  hypothesis that vanilla arms STRUCTURE-spawned adult piglins was
  disproved by re-reading the branch structure.
- **Drowned equipment (PASS):** vanilla
  `Drowned.populateDefaultEquipmentSlots`: `if ((double) nextFloat() >
  0.9) { int i = nextInt(16); if (i < 10) MAINHAND = TRIDENT else
  FISHING_ROD }`. The port's Drowned arm (`finalize.rs`) is an exact
  match (double comparison, 16-bound, 10-split); the spawn-path arm in
  `spawning.rs:228` is the same logic.
- **Zombie spear / G21 re-check (PASS):** see "Re-checked, not defects".

Note (not a defect): finalize RNG draw-order divergence (values correct,
streams diverge) — same note as 26.2: the port's finalizer draws from its
own RNG in an order that does not always match vanilla's per-call
sequence, so identical seeds do not reproduce identical equipment; the
distributions and per-item probabilities are correct.

## Oracle defects (Batch 0's measured table, Batch 3 scope)

No oracle repairs were done (checks + triage only). **52 failing oracles
in 8 groups**, all failing on all three classpaths in the MEASURED Batch 0
table (`docs/sweep/26.1/batch-0-oracles.json`); class + error count quoted
from that table, first-error symbols from the Batch 0 big-compile client
log (`sweep-26.1-batches/b0/big-compile-client.log`, exact symbols in
`evidence/client-errors.json` / `client-errors-full.json`). All 30 oracles
from the 26.2 failing roster still fail in 26.1, PLUS 22 from the 26.2
pass/client+both roster now fail — the 26.1 API gap is wider than 26.2's,
dominated by the absent `EntityTypes` holder (Batch 0 group G4).

- **B3-O01 — missing-override `fuelValues()` (3 oracles, 1 error each,
  Batch 0 group G1):** VanillaActiveEffectTickOracle,
  VanillaAttributeUpdatesOracle, VanillaLivingLandingOracle — "Client*
  Level is not abstract and does not override abstract method fuelValues()
  in Level". 26.1's `Level` declares `public abstract FuelValues
  fuelValues()` (verified in the dump, `Level.txt:1929`); the oracles'
  26.3-form level stubs lack it. Mechanical repair: add a `fuelValues()`
  override. Same group as 26.2.
- **B3-O02 — constructor mismatch (5 oracles):** VanillaAllayInteraction
  StateOracle (n=3), VanillaPlayerSyncOracle (n=1),
  VanillaRemoteAttackOracle (n=8), VanillaSheepDyeOracle (n=4) —
  "constructor LocalPlayer in class LocalPlayer cannot be applied to
  given types"; 26.1's `LocalPlayer` ctor takes 8 arguments (Minecraft,
  ClientLevel, ClientPacketListener, StatsCounter, ClientRecipeBook,
  Input, boolean, ChatAbilities — verified in the dump), 26.3's takes 9.
  Plus VanillaEntitySoundPacketOracle (n=3) — "constructor MinecraftServer
  ... cannot be applied to given types"; 26.1's `MinecraftServer` ctor
  takes 10 arguments (Thread, LevelStorageAccess, PackRepository,
  WorldStem, Optional\<GameRules\>, Proxy, DataFixer, Services,
  LevelLoadListener, boolean). Note VanillaBoatRiderOracle moved to B3-O03
  in 26.1 (no-override first, n=4) but carries the LocalPlayer ctor error
  behind it.
- **B3-O03 — no-override (14 oracles):** VanillaArrowOracle (n=1),
  VanillaBlockingItemDamageOracle (n=2), VanillaBoatRiderOracle (n=4),
  VanillaClientArrowOracle (n=1), VanillaDamageImmunityOracle (n=2),
  VanillaEntityDeathLootOracle (n=4), VanillaFinalizeSpawnOracle (n=7),
  VanillaLateEntityPairOracle (n=4), VanillaMobAttributesOracle (n=1),
  VanillaNaturalSpawnOracle (n=16), VanillaPlayerTravelOracle (n=1),
  VanillaRegisteredPlayerCombatOracle (n=6),
  VanillaServerPlayerCombatOracle (n=6), VanillaSpawnRulesOracle (n=1) —
  "method does not override or implement a method from a supertype" (26.3
  methods absent in 26.1). 26.2's group had 6; 26.1 adds Arrow,
  ClientArrow, EntityDeathLoot, LateEntityPair, MobAttributes,
  NaturalSpawn, PlayerTravel, SpawnRules, and BoatRider (moved from the
  ctor group); Raid moved out to B3-O06. NaturalSpawn (n=16) also carries
  DedicatedServer/ServerChunkCache/ChunkMap ctor mismatches behind the
  first no-override error.
- **B3-O04 — `EntityTypes` holder absent (9 oracles, Batch 0 group G4):**
  EntityTranscript (n=7), RideTranscript (n=4),
  VanillaBoatAuthorityOracle (n=4), VanillaBreezeHookOracle (n=3),
  VanillaMobBreathingOracle (n=1), VanillaMobFallResetOracle (n=2),
  VanillaShulkerBulletOracle (n=4), VanillaShulkerSpaceOracle (n=2),
  VanillaSquidClientOracle (n=2) — first error "cannot find symbol:
  class EntityTypes, location: package net.minecraft.world.entity".
  Jar-listing verified: `EntityTypes`/`BlockEntityTypes` have 0 entries
  in either 26.1 jar; registration still lives on `EntityType`/
  `BlockEntityType`. (VanillaMobAiOracle and VanillaTradingOracle also
  reference `EntityTypes` — see B3-O05.) NEW group for 26.1 — 26.2 has
  the holder.
- **B3-O05 — missing-symbol, other 26.x APIs (16 oracles):**
  VanillaDragonContactsOracle (n=3; `Enderman` — the 26.3 renamed class;
  26.1 has `EnderMan`, verified by jar listing), VanillaEntityMoveOracle
  (n=8; `Actor.MoveSimulationType` + `BlockTags.SUPPRESSES_BOUNCE`),
  VanillaLivingAirOracle (n=6), VanillaLivingFluidOracle (n=9),
  VanillaLivingMotionOracle (n=2; `Attributes.FRICTION_MODIFIER` /
  `AIR_DRAG_MODIFIER` — absent from the 35-attribute registry, verified in
  `attributes-vanilla.json` — plus `FluidTags.ENTITY_FLOATABLE` for
  LivingFluid), VanillaLivingHurtOracle (n=4), VanillaPlayerHurtOracle
  (n=4; `Actor`/`Human.damageCooldownTime`),
  VanillaPlayerInventoryOracle (n=1;
  `VanillaRegistries.createWorldLookup()`),
  VanillaRegisteredEntityFluidOracle (n=2;
  `BlockTags.BLOCKS_FLUID_FLOW`), VanillaSuffocationOracle (n=2;
  `BlockTags.CAUSES_SUFFOCATION` — the same 26.3-form tag behind B3-G22),
  VanillaMobAiOracle (n=12; `EntitySpawnRequest` + `EntityTypes` +
  `DataLoadContext.datapackWorldRegistries()`), VanillaTradingOracle
  (n=6; `datapackWorldRegistries()` + MinecraftServer ctor +
  `EntityTypes` package), VanillaVehicleContainerOracle (n=3;
  `EntitySpawnRequest`), VanillaWorldSpawnOracle (n=5; `NoiseSpawnFinder`
  + `densityfunction` package (0 entries in the jar) +
  `RandomState.create(Registry,long,NoiseGeneratorSettings)` overload),
  KnockbackClient (n=6; `ServerboundPunchPacket` — 0 entries in the jar),
  LocatorBarClient (n=13;
  `ClientboundUpdateAdvancementsPacket.removed()/added()/progress()` —
  26.1's packet is a plain class with private fields and NO public
  accessors; 26.3 made it a record). `BlockTags` in 26.1 has 248 fields
  (verified in the dump), none of CAUSES_SUFFOCATION / SUPPRESSES_BOUNCE /
  BLOCKS_FLUID_FLOW. Other jar-verified absences behind these errors:
  `SwingState`, `cubemob` package, `ServerboundPunchPacket`,
  `MoveSimulationType`, `EntitySpawnRequest`, `NoiseSpawnFinder`,
  `advancements.triggers`/`predicates` subpackages (only `criterion`
  exists), top-level `ClockInstance` (nested
  `ServerClockManager$ClockInstance` only).
- **B3-O06 — missing-package (2 oracles):** VanillaRaidOracle (n=11;
  `package net.minecraft.advancements.triggers does not exist`),
  VanillaEntityComponentsOracle (n=3; `package
  net.minecraft.advancements.predicates.entity does not exist`). 26.1's
  advancements package has only the `criterion` subpackage.
- **B3-O07 — cascade (2 oracles):** VanillaEntityCollisionOracle
  ("depends on broken: VanillaFaceSupportOracle"),
  VanillaHelmetDamageOracle ("depends on broken: VanillaBlockUpdateOracle")
  — no errors of their own in the client log; they fail only through
  their dependencies.
- **B3-O08 — other (1 oracle):** VanillaEntityFluidOracle (n=1; "'void'
  type not allowed here" at line 74).

Cross-batch overlaps (recorded, not dropped — 26.2's B3-O04 folded
VanillaRegisteredEntityFluidOracle→B1, VanillaLivingFluidOracle→B1,
VanillaTradingOracle→B4 at merge): VanillaRegisteredEntityFluidOracle and
VanillaLivingFluidOracle overlap B1's fluid/suffocation area;
VanillaTradingOracle overlaps B4. LocatorBarClient overlaps the
locator-bar driver area (G18's driver family).

Passing Batch 3-scope oracles (measured, all three classpaths):
VanillaAttributeHealthOracle, VanillaEffectNotificationsOracle,
VanillaEffectSyncOracle, VanillaFluidBoxOracle, VanillaLivingFallOracle,
VanillaTradeComponentsOracle, VanillaVillagerFarmingOracle,
VanillaWorldSpawnPacketOracle, VanillaCombatRulesOracle,
VanillaEntityAttributeOracle, VanillaEntityObstructionOracle,
VanillaDamageSourceOracle, VanillaFallDamageOracle, VanillaSwimmingOracle,
VanillaLivingDamageOracle, VanillaLivingHealthOracle, VanillaMobAirOracle,
VanillaMobDiscardOracle, VanillaRaiderCelebrationOracle,
VanillaServerPlayerDamageOracle, VanillaPlayerEquipmentOracle,
VanillaPlayerFoodOracle, VanillaPlayerHealthOracle. Pass on client+both
(server fails on client packages — expected, not breakage):
VanillaZombieCureSoundOracle.

Adjacent, not Batch 3: VanillaBlockEntityAnimOracle/
VanillaBlockEntityRenderOracle (Batch 4), VanillaFeatureCatalogOracle,
VanillaItemUseCatalogOracle, VanillaRedstoneCatalogOracle (Batch 2),
VanillaEntityAnimOracle/VanillaDragonRenderOracle (Batch 10),
VanillaAudioAttributesOracle (missing-symbol n=1),
VanillaTeleportConsumeOracle (now FAILS in 26.1, no-override n=5 — Batch 7
area), VanillaFallingBlockOracle (Batch 1, ChunkMap ctor).

## Drivers (D)

Curated Batch 3 driver list: **144 files** (keyword cut over the 647
`tools/test_*_parity.py` — 174 raw — excluding other batches by regex
(block_entity, data catalogs/codecs, command/scoreboard, item/inventory/
loot, worldgen, render/model/particles, sound-only, dedicated-server
infra, respawn/hardcore, turtle-egg block tests/dripleaf, 23 files) and 7
manual drops (keyword false positives from "cat" inside other words —
`test_secure_chat_parity`, `test_predicate_reference_startup_parity`,
`test_named_predicate_reference_parity`, `test_scattered_ore_parity` —
plus data-catalog/sound files `test_damage_catalog_parity`,
`test_feature_catalog_parity`, `test_hostile_sound_categories_parity`));
**135 core** after removing 9 borderline entries (spawn_loot,
underwater_magma, fishing ×4, piston-projectile ×2, squid_client). Full
lists in `evidence/drivers-curated.json`. No driver was run (static
analysis only). Driver checks: 3 — 0 pass, 3 gap (B3-G18, B3-G19, B3-G20,
found by reading). In 26.1 all three driver checks are gaps, unlike 26.2
where G18 passed.

Stale 26.3 docstrings only (no functional effect): dragon_contacts,
mob_discard, mob_quad_leash, mount_control, vehicle_container,
villager_farming, zombie_cure_sound, fishing_rng (borderline),
predicate_reference_startup (comment only — dropped from scope as a
false positive, but its lines 78/80 DO build a functional 26.3 server
command; flagged for whichever batch owns it).
`test_spawn_loot_parity.py:34` hardcodes
`RESOURCES = ROOT / '.local' / 'horizon-client' / '26.3'` (functional,
borderline scope — loot, Batch 8's area; noted here, not defected).

## Method notes

- **VANILLA-CHECK RULE throughout:** every behaviour claim was verified
  against 26.1's OWN vanilla jar/data, never the 26.3 base's form. Where
  26.1's form differs from both 1.21.7 and 26.2 (hoglin NOT
  `notInPeaceful`; literal `immuneTo` blocks; `SpawnPlacementType` an
  interface; no `EntityTypes` holder; no `densityfunction` package; no
  teleport tags; `EnderMan` not `Enderman`; `ClientboundUpdateAdvancementsPacket`
  a plain class, not a record), the report says so explicitly.
- **Class-file scan:** `evidence/scan_classes.py` (JVM-free parser) over
  all 708 entity classes in the server-game jar established the complete
  declarer sets for `checkSpawnRules` (3), `getWalkTargetValue` (18),
  `checkSpawnObstruction` (13), `removeWhenFarAway` (20),
  `getMaxSpawnClusterSize` (8), `getMaxHeadYRot` (11), `getMaxHeadXRot`
  (7), `getAmbientSoundInterval` (11), `isSunSensitive` (2),
  `getHeadRotSpeed` (3), `sunProtectionSlot` (3), `isMaxGroupSizeReached`
  (2), `checkDespawn` (5), and `isFood` (31, `Animal` abstract).
  `EntityType.<clinit>` was parsed to `evidence/entity-type-flags.json`
  (157 entries); `Attributes.<clinit>` to
  `evidence/attributes-vanilla.json` (35 attributes);
  `SpawnPlacements` registrations to
  `evidence/spawn-placements-vanilla.json` (82 calls, 38 predicates).
- **javap is slow on this VM** (~10–15 s/class under load); ~140 dumps
  in `evidence/javap/` were run in background batches. Never javap the
  bundler `server.jar` or the leakage-audit 26.1.x server jars (no game
  classes) — only the named `client.jar` / `server-game.jar`.
- **Tally convention:** behaviour checks only; oracle triage has its own
  counts. A check is one vanilla-vs-port comparison of a named behaviour;
  a check that finds multiple defects (the A3 food table: G05, G27, G28)
  counts once in the tally but yields one slice item per defect.
- **Notes recorded, NOT minted as defects:** dead SulfurCube arms in the
  port (`spawn_rules.rs:440`, `spawn_selection.rs:149/584`); the port's
  `checkDespawn` order difference (nearest-player check before persistent
  reset — unobservable); the port's `spawn_selection` `listed`
  optimisation and zombie-flag + recompute split for burning/dangerous
  blocks (both sound); trial spawner
  `overridePeacefulAndMobSpawnRule` omitted by the port (dead in vanilla
  — no caller in either jar); trial spawnMob yaw draw position (trivial,
  no RNG consumer between); Hero-of-the-Village modifier computed in f32
  then widened vs vanilla double (brute-forced amp 0..255 × count 1..64:
  0 mismatches for amp ≤ 11; first mismatch at amp 12, commands-only);
  boat `getWaterLevelAbove` lacking the lastYd scan extension (one-tick
  snap difference; identical to 26.2's accepted form) and landFriction
  halving keyed on `controlled` vs vanilla's controlling-passenger-is-
  Player (remote-only nuance); hurting-projectile normalize floor 1e-9 vs
  vanilla's 1e-4 (sub-1e-4 speeds only; same code passed in 26.2);
  MeleeAttack `stop()` clearing creative but not spectator targets
  (transient stale pointer; identical to 26.2); the ridden-mount early
  return in `server_ai_step` (parallels vanilla's control-flag
  disabling); finalize RNG draw-order divergence (values correct, streams
  diverge — same note as 26.2).
- **Findings shared with the 26.2 port (flag for the 26.2 record):** the
  three food-table gaps (G05, G27, G28 — byte-identical `food_tag()`),
  BreedGoal panic (G26 — identical code), furnace minecart cap (G25 —
  identical code at all three sites), MeleeAttack spectator-target note,
  boat lastYd note, hurting-projectile normalize note.
- **UNVERIFIED carry-overs: none.** Every 1.21.7 defect was re-checked
  against 26.1's own jar and is either a verified defect above or listed
  under "Re-checked, not defects" with its evidence.
