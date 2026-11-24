# Batch 3 — Entities / mobs (26.1.1 parity sweep)

**Version:** 26.1.1 · **Date:** 2026-11-24 · **Branch:** `parity-sweep-26.1.1`
(workdir `~/workspace/mcir-work/sweep-26.1.1`) · **Scope:**
`docs/PARITY-SWEEP.md` §4 Batch 3 — mob AI, spawning, trading, attributes,
projectiles, vehicles; 157 entity types (44 monster, 31 creature, 72 misc,
4 water ambient, 3 water creature, 1 axolotl, 1 ambient, 1 underground
water creature); 144 drivers curated to 135 core (see §D).

**Status: checks + triage only, per the brief: no fixes, no edits under
`versions/26.1.1/`, no cargo builds, no driver runs, no oracle repairs, no
commits — the coordinator commits these deliverables.** Everything below is
static analysis: `javap -c` on the named 26.1.1 server-game jar
(`~/workspace/mcir-work/26.1.x/versions/26.1.1/jars/server-game.jar`,
sha1 `63e2f8ca1ccbcfda2f72ad0e8278f975d5a6a011`, DataVersion 4788, Java 25),
source comparison against `versions/26.1.1/crates/`, and checks of the
prepared 26.1.1 data (`versions/26.1.1/.local/horizon-client/26.1.1/`,
Batch 0's prepare).

**Method — diff-first.** Batch 1 reported the 26.1.1 tree regressed four
26.1 fixes, and Batch 2 found the same pattern for level/ behaviour, so
every Batch 3 file was diffed 26.1-vs-26.1.1 (`evidence/diff-*.txt`) rather
than assuming the 26.1 verdicts carry. That proved essential: the 26.1.1
tree carries 26.2/26.3-form behaviour in ~20 mob files plus several `game/`
files, contradicting `PORT-NOTES-26.1.1.md`'s claim that the completed 26.1
code was taken as the carry-forward base. Every 26.1 item was re-verified
fresh against the 26.1.1 jar — the 26.1→26.1.1 vanilla delta is only 3
changed classes (`PlayerEntry.<init>`, `DetectedVersion.createBuiltIn`,
`SharedConstants` fields) plus DataVersion 4786→4788, with no entity class
changed, but no verdict below rests on that delta alone.

## Tally

- **Behaviour checks: 96 — 62 pass, 34 gap as-tallied** (37 defect items:
  B3-G03, G04, G05, G06, G09, G11, G13, G14, G15, G18, G19, G20, G22, G23,
  G24, G25, G26, G27, G28, G29–G46 — the A3 food-table check yields three
  items and the B3 enderman/cat check yields two; every other check yields
  one). **Behaviour/non-behaviour split:** 93 behaviour checks (62 pass,
  31 gap) + 3 non-behaviour driver-code checks (0 pass, 3 gap: G18, G19,
  G20) — driver-code-defect checks counted as-tallied per the 26.2
  precedent, split reported explicitly.
- **Oracle defects: 8 groups** (B3-O01–B3-O08, 52 oracles — not counted in
  the check tally).
- **Oracle repairs done: none** (checks + triage only).
- **Severity: 2 major, 35 minor** — 2 major port (B3-G22, the
  `#causes_suffocation` startup blocker, carried from 26.1; B3-G14,
  endermen never teleport, re-minted from 1.21.7), 32 minor port, 3 minor
  driver-side (B3-G18, G19, G20).
- **Carry-over:** all 15 of 26.1's Batch 3 defect items reproduce in
  26.1.1 (fresh items, same IDs, fresh evidence). Four 1.21.7 items that
  26.1 had cleared REGRESS in 26.1.1 (G03, G13, G14, G15 — the 26.1 tree's
  corrections were not carried forward). 18 NEW defects (G29–G46), all
  26.2/26.3-form regressions against the 26.1 tree's corrected form, except
  G39's regular-spawner arm, which is shared with 26.1 and was missed by
  the 26.1 sweep.

Per-section counts (must sum to 96): A1 1 (1 pass), A1b 11 (10 pass,
1 gap), A2 1 (1 pass), A3 11 (6 pass, 5 gap), A4 5 (4 pass, 1 gap),
B1 3 (1 pass, 2 gap), B2 27 (21 pass, 6 gap), B7 4 (3 pass, 1 gap),
B8 3 (1 pass, 2 gap), B5 4 (3 pass, 1 gap), B4 6 (3 pass, 3 gap),
B6 3 (2 pass, 1 gap), B3 11 (3 pass, 8 gap), B9 3 (3 pass),
Drivers 3 (0 pass, 3 gap).

## Defects

### B3-G03 — `immune_tag` tags don't exist (minor, port)

Vanilla 26.1.1 `EntityType.<clinit>` passes LITERAL block sets to
`immuneTo` (fox→sweet_berry_bush; polar_bear/snow_golem/stray→powder_snow;
wither/wither_skeleton→wither_rose — 6 calls, verified in the dump). The
26.1.1 port's `immune_tag` (`mobs/spawn_rules.rs:182-188`) resolves
`#fox_immune_to`, `#polar_bear_immune_to`, `#snow_golem_immune_to`,
`#stray_immune_to`, `#wither_immune_to`, `#wither_skeleton_immune_to` —
all six absent from the 26.1.1 jar data and from prepared `block-tags.json`
(248 tags), so `immune_to` answers false for every block: foxes take sweet
berry bush damage, strays freeze in powder snow, withers take wither-rose
damage. The 26.1 tree had the correct literal form (`immune_to` + tests);
the 26.1.1 tree reverted to the 26.2 tag form. Re-minted from 1.21.7
(26.1 had cleared it).

### B3-G04 — Hoglin wrongly disallowed in peaceful (minor, port)

Vanilla 26.1.1 `EntityType.<clinit>` marks 37 types `notInPeaceful`;
hoglin is NOT among them (verified directly in the dump). The port's
`allowed_in_peaceful` disallows 38 types including hoglin, in BOTH
`mobs/spawn_rules.rs:134` and `mobs/finalize.rs:897` (identical to 26.1).
Live paths: `ai.rs` checkDespawn, `game/commands.rs`, `passengers.rs`,
`stations.rs`, `structure_mobs.rs` — hoglins despawn / refuse to spawn in
peaceful in the port where vanilla keeps them. (Opposite of 26.2, where
hoglin IS `notInPeaceful`.) Reproduces from 26.1. The hoglin facet also
reaches the spawn-egg gate (`game/creatures.rs` `spawn_egg_allowed`):
hoglin eggs are refused in peaceful in the port where vanilla allows them
(see "Re-checked, not defects" — the gate itself is correct).

### B3-G05 — Camel has no food (minor, port)

Vanilla 26.1.1 `Camel.isFood` = `#camel_food` (cactus; the tag exists in
the jar's data). Port `food_tag()` (`mobs/species.rs:1633-1665`) covers
`CamelHusk` but NOT `Camel`, so `MobType.food` is empty for camels and
feeding/breeding/tempting never fire. Camels cannot breed. Reproduces from
26.1 (and shared with the 26.2 port's identical table — flag for the 26.2
record).

### B3-G06 — Armadillo scared head rotation ignored (minor, port)

Vanilla 26.1.1 `Armadillo.getMaxHeadYRot = isScared() ? 0 : 32`. Port
`max_head_y_rot` (`mobs/species.rs:1719`) returns 32.0 unconditionally, so
a scared (rolled-up) armadillo still turns its head. Reproduces from 26.1.

### B3-G09 — Sun-burn ignores powder snow (minor, port)

Vanilla 26.1.1 `Mob.isSunBurnTick` excludes mobs that are `isInWaterOrRain
|| isInPowderSnow || wasInPowderSnow`. The port's `sun_burn_tick`
(`ai.rs:2035`) uses `wet = in_water || in_rain` only, although `Mob` HAS
`in_powder_snow` state — undead standing in powder snow burn at noon in
the port where vanilla spares them. Reproduces from 26.1.

### B3-G11 — `getRandomPosWithin` y-range off by one (minor, port)

Vanilla 26.1.1 `NaturalSpawner.spawnCategoryForChunk` computes
top = `getHeight(WORLD_SURFACE) + 1` and samples
y = `randomBetweenInclusive(minY, top)` → [minY, H+1]. The port's
`spawn_category_for_chunk` (`spawn_selection.rs:1103-1105`) uses top = H
and y = min + `nextInt(top-min+1)` → [minY, H], so the topmost spawnable
y is never chosen. Reproduces from 26.1.

### B3-G13 — Constant-count packs skip an RNG draw (minor, port)

Vanilla 26.1.1 `SpawnerData` is a plain record (minCount, maxCount) and
the sample site draws unconditionally:
`minCount + nextInt(1 + maxCount - minCount)` — even when min == max. The
26.1.1 port's `sample_count` (`spawn_selection.rs:782`) returns the
constant without drawing for constant entries, so every constant-count
pack skips one RNG draw and all downstream draws in the spawn sequence
shift. The 26.1 tree drew unconditionally. Re-minted from 1.21.7 (26.1 had
cleared it).

### B3-G14 — Endermen never teleport (MAJOR, port)

Vanilla 26.1.1 `EnderMan.teleport` + `LivingEntity.randomTeleport` ground
on `blocksMotion()`; no teleport tags exist in 26.1.1 data (verified by
jar listing: `entities_can_teleport_to`, `dangerous_for_teleportation`,
`enderman_does_not_teleport_to` all absent). The 26.1.1 port's
`mobs/enderman.rs:117` grounds teleport on the absent
`#entities_can_teleport_to` tag, so no destination ever qualifies and
endermen never teleport. The 26.1 tree used blocksMotion. Re-minted from
1.21.7 (26.1 had cleared it). Note: the `game/enderman.rs` diff is
comment-only — the regression is in `mobs/enderman.rs`.

### B3-G15 — Cat morning gift never happens (minor, port)

Three stacked failures, all in `mobs/tame.rs`:
1. The `CatRelaxOnOwner` goal's `can_use` (tame.rs:1270) gates on the
   absent `#cats_can_lie_on` tag at the sleeping owner's position, so the
   goal never starts. Vanilla `CatLieOnBedGoal.isValidTarget` checks
   `BlockTags.BEDS` (present in 26.1.1 data).
2. `give_morning_gift` (tame.rs:1350) drops the leash-holder origin —
   vanilla `Cat$CatRelaxOnOwnerGoal.giveMorningGift` teleports from the
   leash holder's block when the cat is leashed; the port always uses the
   cat's own position.
3. The gift hop itself grounds on the absent
   `#entities_can_teleport_to` tag (the scan loop walks down to the world
   bottom and never finds a tagged block, so the hop never executes);
   vanilla uses `randomTeleport(x±5, y±2, z±5, false)`.

The 26.1 tree had the leash-holder origin + `enderman::random_teleport`.
Re-minted from 1.21.7 (26.1 had cleared it).

### B3-G18 — DRIVER: entity_components predicate form (minor, driver)

`tools/test_entity_components_parity.py:27-29` reads
`criterion['conditions']['entity']['predicate']` and then
`predicate['minecraft:components']`. 26.1.1 advancement data uses the list
form with `predicate: {components: {...}}` — verified in all 3
advancements that use it (`complete_catalogue` 11 criteria, `whole_pack`
9, `leash_all_frog_variants` 3). The driver raises TypeError on the first
criterion. (26.2 uses the new form; 26.1.1 the old — the driver was written
for 26.2/26.3 data.) Driver file identical to 26.1. Reproduces from 26.1;
in 26.2 this check PASSED.

### B3-G19 — DRIVER: state-count hardcode 35723 (minor, driver)

Two Batch 3 drivers hardcode the 26.3 block-state count:
`test_suffocation_parity.py:66` and
`test_registered_entity_fluid_parity.py:66` both assert the state count
equals 35723. 26.1.1 has 29,873 states (`mob-data.json`), so both drivers
fail their own completeness check. (In 26.2 the suffocation driver was
fixed to 32366; in 26.1.1 it was not.) Driver files identical to 26.1.
34 further drivers tree-wide hardcode 35723 (36 tree-wide total) — listed
in the slice's `out_of_scope_failures`, not defected here. Reproduces
from 26.1.

### B3-G20 — DRIVER: 26.3 resources path (minor, driver)

Five Batch 3 drivers build the server command with
`lab.LOCAL / 'horizon-client/26.3'`:
`test_locator_bar_parity.py:269`, `test_server_dragon_parity.py:111`,
`test_server_entities_parity.py:186`,
`test_server_knockback_parity.py:435`, `test_projectile_shoot.py:46` — so
they run the horizon server against 26.3 prepared data (or fail outright
where none exists) instead of 26.1.1's. Driver files identical to 26.1.
30 further drivers tree-wide use a 26.3 resources path (35 tree-wide
total; `test_brewing_parity.py` gained one vs 26.1) — listed in the
slice's `out_of_scope_failures`. Reproduces from 26.1.

### B3-G22 — Default suffocation is tag-gated on absent `#causes_suffocation` (MAJOR, port)

Vanilla 26.1.1's default suffocation rule is `Properties.<init>` indy#6 →
`lambda$new$5` = `blocksMotion() && isCollisionShapeFullBlock()` — NO tag.
The port's default is `TaggedFullCollision`
(`horizon-world/src/suffocation.rs:72`, identical to 26.1) gated on
`#causes_suffocation`, a tag absent from 26.1.1 jar data (0 entries) and
from prepared `block-tags.json` (248 tags). `motion.rs:139` requires it
via `tag(...)?`, which errors on missing tags; `TagDefinitions::build`
synthesizes nothing. The startup path `resource_load.rs load_resources` →
`load_core` → `gameplay_tables` → `MotionResources::load_with_facts`
propagates the error — **the 26.1.1 client cannot load its core tables
with prepared 26.1.1 data**. The first failure in evaluation order is
actually `#suppresses_bounce` (`motion.rs:138`, also absent), then
`#causes_suffocation`. Even if load passed, no default block would
suffocate. Contrast: 26.2's `motion.rs:65` documents "26.3 replaced this
with the `#causes_suffocation` tag, which 26.2 data has not" and uses
`blocks_motion` + `MotionFullCollision`. CI crate tests do not catch this:
data-dependent tests skip quietly without `.local`. Reproduces from 26.1
(Batch 1 independently flagged the same reproduction).

### B3-G23 — Sleeping mobs suffocate (minor, port)

Vanilla 26.1.1 `LivingEntity.isInWall` = `!isSleeping() &&
super.isInWall()`; `Entity.isInWall` = noPhysics→false, eye box
`AABB.ofSize(eye, width*0.8, 1e-6, width*0.8)`, any block `!isAir &&
isSuffocating && collisionShape ∩ box`; `baseTick` hurts 1.0 InWall. The
port's `ai.rs base_tick` (1267) matches the geometry, damage, and Vex
exclusion but has NO sleeping check — a sleeper whose eye is inside a
suffocating shape takes in-wall damage in the port. Edge case. Reproduces
from 26.1.

### B3-G24 — Villager level-up is immediate and new offers are discounted (minor, port)

Vanilla 26.1.1 `rewardTradeXp` does NOT level up on trade: it sets
`updateMerchantTimer = 40` and `increaseProfessionLevelOnUpdate = true`
(pop += 5). `customServerAiStep` decrements the timer ONLY while not
trading (isTrading gate); at 0 it calls `increaseMerchantCareer` (level+1
+ updateTrades) — with NO `updateSpecialPrices` call, so new offers carry
`specialPriceDiff` 0 until the next session. The 26.1.1 port levels up
IMMEDIATELY in `Trader::notify_trade` (`merchant.rs:399-407`, identical to
26.1) and recomputes special prices for ALL offers including the new ones,
with the session-start reputation/hero. Observable: newly-unlocked trades
show discounted prices mid-session in the port (full price in vanilla),
and the level-up appears at once instead of ~2s after trading ends. The
26.2 port has the correct deferred form (`level_up_pending`,
`merchant.rs:413` + `trading.rs:333`). No `level_up_pending` /
`update_merchant_timer` exists anywhere in the 26.1.1 port. Reproduces
from 26.1.

### B3-G25 — Furnace minecart too fast under `minecart_improvements` (minor, port)

Vanilla 26.1.1 `NewMinecartBehavior.getMaxSpeed` = `MAX_MINECART_SPEED *
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
26.1.1 `DEFAULT_FLAGS` = VANILLA_SET only, so the default (old-behavior)
path is correct; the divergence is live only in worlds with the experiment
enabled. The 26.2 port has identical code at all three sites. Reproduces
from 26.1 (shared with 26.2 — flag for the 26.2 record).

### B3-G26 — BreedGoal ignores partner panic (minor, port)

Vanilla 26.1.1 checks the partner is NOT PANICKING in both places:
`getFreePartner` skips panicking candidates, and `canContinueToUse` is
`partner.isAlive() && partner.isInLove() && loveTime < 60 &&
!partner.isPanicking()`. The port's partner search and `can_continue`
omit the panic clause entirely (`goals.rs:2142+`; the `Seen` struct
carries no panicking field). Panic does not clear in-love in either
implementation, so the gap is observable: hit one of two in-love animals
and the other keeps following it in the port; vanilla stops. Shared with
the 26.2 port's identical code (26.2's sweep passed B3 with it) — flag for
the 26.2 record. Reproduces from 26.1.

### B3-G27 — Zombie horse has no food (minor, port)

Vanilla 26.1.1 `ZombieHorse.isFood` = `#zombie_horse_food` (red mushroom;
the tag exists in the jar's data). The port's `food_tag()` has no
`ZombieHorse` arm, so zombie horses cannot be fed for healing/temper in
the port. Shared with the 26.2 port's identical table — flag for the 26.2
record. Reproduces from 26.1.

### B3-G28 — Skeleton horse has no food (minor, port)

Vanilla 26.1.1 `SkeletonHorse` extends `AbstractHorse` and does not
override `isFood`, so it inherits `#horse_food`. The port's `food_tag()`
covers `Horse | Donkey | Mule` but not `SkeletonHorse`, so skeleton horses
cannot be fed in the port. Shared with the 26.2 port's identical table —
flag for the 26.2 record. Reproduces from 26.1.

### B3-G29 — Peaceful gate moved into `checkSpawnRules`; phantom/shulker can no longer spawn in peaceful (minor, port)

Vanilla 26.1.1 `SpawnPlacements.checkSpawnRules` has NO peaceful gate;
the peaceful check lives inside the per-type predicates that have it
(`Monster.checkMonsterSpawnRules`, `checkAnyLightMonsterSpawnRules`, the
drowned/guardian/ghast/magma-cube/slime/zombified-piglin rules), and a
type whose predicate lacks it — phantom and shulker use plain
`Mob.checkMobSpawnRules` — answers on its other rules alone. The 26.1.1
port moved the gate up into `checkSpawnRules`
(`mobs/spawn_rules.rs:320-322`: `if !allowed_in_peaceful(s) &&
l.peaceful() { return false; }`) and dropped the per-type gates from
`monster_rules` (297), blaze/breeze/zoglin (376), endermite/silverfish
(395) and pillager (439). Monsters still fail in peaceful (same outcome),
but phantom and shulker now fail too, where vanilla lets them spawn. The
26.1 tree had the correct per-type form (with a test proving the phantom
passes in peaceful); the 26.1.1 tree deleted the test and took the 26.2
form. NEW in 26.1.1.

### B3-G30 — Baby-dimensions table carries the 26.2 form; 29 species wrong (minor, port)

The port's baby-dimensions table (`mobs/species.rs` `Def.baby`) carries
the 26.2 form, not 26.1.1's. Vanilla 26.1.1 `BABY_DIMENSIONS` declarers
(20 classes, extracted with a JVM-free class parser —
`evidence/baby_dims2.py`, tables in `evidence/baby-tables.json`) vs the
port: zombie family + piglin + villager 0.98 vs 0.99 height (7 species);
axolotl wrong absolute dims; chicken eye 0.28125 vs 0.28; cow/mooshroom
absolute dims vs `scale(0.5)` (eye 0.665); fox eye 0.34375 vs 0.2975;
squid/glow squid height 0.5 vs 0.63; 11 `Scale`→`Abs` conversions (camel,
cat, goat, hoglin, ocelot, pig, polar bear, sheep, strider, wolf, zoglin —
wrong at least in eye height, several in width/height too, verified
against adult dims/eyes from `entity-types.json`); 4 wrong eye overrides
(armadillo 0.21875 vs scaled 0.156; dolphin 0.09375 vs 0.195; happy ghast
0.46875 vs 0.6175; panda 0.28125 vs 0.53125). The 26.1 tree's table
matched vanilla. NEW in 26.1.1.

### B3-G31 — Camel sitting baby dims wrong; body-anchor offset uses the 26.2 double form (minor, port)

Vanilla 26.1.1 `Camel.SITTING_DIMENSIONS = scalable(width, height−1.43)`
with eye 0.845, and a sitting baby is
`SITTING_DIMENSIONS.scale(getAgeScale)` = (1.02, 0.567, 0.507). The
port's `sitting_dims` (`mobs/camel.rs:52-54`) returns (0.95, 0.425, 0.41)
for a baby. Separately, vanilla `getBodyAnchorAnimationYOffset` computes
`height − 0.375F * scale` in FLOAT; the port's `body_anchor_y`
(`mobs/camel.rs:124-125`) uses the 26.2 double form (0.09375 baby /
0.375 adult offsets). The 26.1 tree matched vanilla on both. NEW in
26.1.1.

### B3-G32 — Heightmap predicates use absent `#blocks_motion_in_heightmap*` tags (minor, port)

Vanilla 26.1.1 has no `blocks_motion_in_heightmap` /
`blocks_motion_in_heightmap_no_leaves` tags (verified absent from jar data
and prepared `block-tags.json`). The 26.1.1 port's heightmap predicates
(`mobs/spawn_selection.rs:419-425`) ground on them, so
`getTopNonCollidingPos` (MOTION_BLOCKING_NO_LEAVES for 80 types) in
`spawnMobsForChunkGeneration` resolves against empty tags. The 26.1 tree
used the legacy-solid rule. `spawn_category_for_chunk` uses WorldSurface
and is unaffected. NEW in 26.1.1.

### B3-G33 — Worn heads never reduce detection range (minor, port)

Vanilla 26.1.1 `LivingEntity.getVisibilityPercent` hardcodes five head
rules (skeleton, zombie, piglin, piglin brute, creeper heads each halve
detection range — verified in the dump, offsets 128–146, including the
piglin-brute rule; see "26.1-record corrections" — the 26.1 report's claim
that brutes are not fooled is wrong). The 26.1.1 port's
`visibility_percent` (`mobs.rs:3185-3211`) reads the 26.2
`mob_visibility` item component (`horizon-items/src/items.rs:1061`
`worn_visibility`); prepared 26.1.1 data has 0 `mob_visibility`
components across all 1,506 items, so worn heads never reduce detection
range in the port. The 26.1 tree hardcoded the five rules. NEW in 26.1.1.

### B3-G34 — Trader hurt gating uses the 10-tick hurt window instead of `hurtMarked` (minor, port)

Vanilla 26.1.1 `TradeWithPlayerGoal` and
`LookAndFollowTradingPlayerSink` gate on `!hurtMarked` — a public field
on `Entity`, cleared at the end of the hurt tick. The 26.1.1 port removed
`hurt_marked` and gates both on `m.hurt_time <= 0`
(`mobs/trader.rs:198`, `mobs/villager_brain.rs:3461`); `hurt_time` is set
to 10 on a hit (`mobs.rs:3904`) and decremented once per tick
(`ai.rs:1353-1354`), so a trader/villager does not resume trading
behaviour for up to 10 ticks after a hit where vanilla resumes on the
next tick. The `trader.rs` site is a regression (26.1 gated on
`!m.hurt_marked`); the `villager_brain.rs` site is shared with 26.1
(identical code — missed by the 26.1 sweep). NEW in 26.1.1 (trader half);
shared with 26.1 (brain half).

### B3-G35 — BreathAirGoal uses the 26.2 five-column search with swapped start order (minor, port)

Vanilla 26.1.1 `BreathAirGoal.findAirPosition` searches
`BlockPos.betweenClosed(x-1, y, z-1, x+1, y+8, z+1)` — a 3×9×3 box — and
`start()` calls `findAirPosition` THEN `nav.stop()`. The 26.1.1 port
(`mobs/goals.rs:2646+`) uses the 26.2 `neighborColumn` five-column search
with the start order swapped (nav.stop first). The 26.1 tree matched
vanilla. NEW in 26.1.1.

### B3-G36 — Cat lie-on-bed / sit-on-block goals gated on absent tags (minor, port)

Vanilla 26.1.1 `CatLieOnBedGoal.isValidTarget` checks `BlockTags.BEDS`
(present); `CatSitOnBlockGoal` checks unopened chests, lit furnaces, and
the foot of beds (no tag gate). The 26.1.1 port gates both on absent tags:
`can_lie_on` (`mobs/tame.rs:1413`) requires `#cats_can_lie_on` and
`CatSitOnBlock` (`mobs/tame.rs:1449`) requires `#cats_can_sit_on` — both
absent from the 26.1.1 jar data and prepared tags — so tame cats never lie
on beds or sit on chests/furnaces/beds. The 26.1 tree matched vanilla.
NEW in 26.1.1. (Distinct from G15's morning-gift goal, which gates on the
same absent lie-on tag at the owner's position.)

### B3-G37 — Inhabited time accrues +1 unconditionally instead of the game-time difference (minor, port)

Vanilla 26.1.1 `ServerChunkCache.tickChunks` computes
`timeDiff = gameTime − lastInhabitedUpdate` and passes it to
`incrementInhabitedTime(timeDiff)` — a chunk reloaded after 1,000 ticks
accrues 1,000 ticks of inhabited time. The 26.1.1 port
(`game/natural_spawning.rs:811`) adds 1 unconditionally, and the
`last_inhabited_update` field was removed from the render world
(`horizon-client-render/src/world.rs`). Local difficulty (which scales
with inhabited time) therefore grows far too slowly for revisited chunks.
The 26.1 tree matched vanilla. NEW in 26.1.1.

### B3-G38 — `fresh_dimensions` removed; pufferfish spawn at full size (minor, port)

Vanilla 26.1.1 `Pufferfish.getScale(0) = 0.5`: a freshly spawned
pufferfish is half-size until it inflates. The 26.1 tree called
`fresh_dimensions` at spawn (`game/natural_spawning.rs:493`); the 26.1.1
tree removed the call (no `fresh_dimensions` reference remains in
`game/natural_spawning.rs`), so pufferfish spawn at full size. NEW in
26.1.1.

### B3-G39 — Spawn-egg entity data applied to spawners; vanilla drops it (minor, port)

Vanilla 26.1.1 `SpawnEggItem.useOn` calls only
`Spawner.setEntityId(type, random)` — type only, no entity data — for both
spawner types, and `TrialSpawnerBlockEntity` has only `setEntityId` (no
`setEntityData` method exists in the jar). The 26.1.1 port applies egg
data to both: the trial arm is new (`mobs/trial_spawner.rs`
`override_entity_data` + caller `game/spawner.rs:450`), and the regular
arm (`game/spawner.rs:461-462` `set_entity_data`) exists in BOTH 26.1 and
26.1.1 — shared with 26.1, missed by the 26.1 sweep. An op's spawn egg
with entity data (e.g. a named/customised mob) sets the spawner to spawn
that customised mob in the port; vanilla spawns a default one. NEW in
26.1.1 (trial arm); shared with 26.1 (regular arm).

### B3-G40 — Armadillo ball-up refuses in liquid (minor, port)

Vanilla 26.1.1 `ArmadilloAi$ArmadilloBallUp.checkExtraStartConditions`
is `onGround()` only — no liquid check. The 26.1.1 port
(`mobs/armadillo.rs:379`, `liquid` at 390) adds `!liquid(m)`, so an
armadillo standing in water will not ball up in the port where vanilla
does. The 26.1 tree matched vanilla. NEW in 26.1.1.

### B3-G41 — Slime/magma-cube `dealDamage` adds a team check vanilla lacks (minor, port)

Vanilla 26.1.1 `Slime.dealDamage` checks alive, `isWithinMeleeAttackRange`,
`hasLineOfSight`, then `mobAttack` — NO team check. The 26.1.1 port
(`mobs/cube.rs:242, 297`) adds `teams_allow_damage` checks, so a slime or
magma cube on a team with `friendly_fire` off cannot damage a teammate in
the port where vanilla damages regardless. The 26.1 tree matched vanilla.
NEW in 26.1.1.

### B3-G42 — Piglin offhand items made persistent with a 2.0 drop chance (minor, port)

Vanilla 26.1.1 `Piglin.holdInOffHand`: a barter item goes to
`setItemSlot` + `setGuaranteedDrop` (2.0); anything else goes to
`setItemSlotAndDropWhenKilled` (1.0) — NO persistence is set either way.
The 26.1.1 port (`mobs/piglin.rs:31-33`) makes non-barter offhand items
persistent AND gives them drop chance 2.0 always, so a piglin that picks
up a non-barter item in its off hand never despawns and always drops the
item in the port. The 26.1 tree matched vanilla. NEW in 26.1.1.

### B3-G43 — Fox chorus-fruit teleport emits a level event 2017 trail vanilla lacks (minor, port)

Vanilla 26.1.1 `TeleportRandomlyConsumeEffect` has ONLY a `diameter`
field — no `directional_particles` in its codec — and `apply` emits the
TELEPORT game event + sound, no level event. The 26.1.1 port parses
`directional_particles` with `.unwrap_or(true)`
(`horizon-items/src/items.rs:129`); the prepared chorus-fruit data has no
such field (`{"type": "minecraft:teleport_randomly"}`), so the flag
defaults true and the fox path (`mobs/fox.rs:387`) emits the 2017
directional-particle trail on every chorus-fruit teleport. The 26.1 tree
had no 2017 event. NEW in 26.1.1. (The same `TeleportConsume` struct feeds
the player's chorus-fruit path (`teleport_consume.rs:109-116`) — Batch 7's
area; flagged in `out_of_scope_failures`.)

### B3-G44 — Loaded sleeping villager position never refreshed (minor, port)

Vanilla 26.1.1 `LivingEntity.baseTick` calls `setPosToBed` (a flat 0.6875
above the head block's base) for a sleeping entity every tick. The
26.1.1 port's `sleeping_position` (`mobs/world.rs:141-145`) gates on
`facts.has_class("AbstractBedBlock")` — a class with 0 entries in the
26.1.1 jar (only plain `BedBlock` exists) — so it returns `None` always,
and the caller's `refresh_sleeping_position` (`mobs/villager.rs`, called
from `ai.rs:1212`) never updates a loaded sleeping villager's position.
The 26.1 tree matched vanilla. NEW in 26.1.1.

### B3-G45 — Spear stab knockback applied as two pushes instead of one summed push (minor, port)

Vanilla 26.1.1's kinetic-weapon stab (`LivingEntity`, the
`recentKineticEnemies` path) calls `causeExtraKnockback` ONCE with the
summed strength `0.4f + getKnockback(target, source)`. The 26.1.1 port
(`mobs/spear_use.rs:272, 275`) calls it TWICE — 0.4, then the enchantment
halved — which pushes the target twice (halving its velocity between
pushes) and slows the attacker twice. The 26.1 tree pushed once with the
summed strength. NEW in 26.1.1.

### B3-G46 — Item-entity landing bounce uses the 26.2 restitution form; bed bounce gone (minor, port)

Vanilla 26.1.1 has NO `restituteMovementAfterCollisions` and NO
`bounceRestitution` (`Entity.class` lacks both); `SlimeBlock` and
`BedBlock` have `bounceUp`, and the 26.1 form bounces any falling speed
(slime: `−vy × 0.8d`; bed: `−vy × 0.66f × 0.8d`). The 26.1.1 port's
`vertical_collision_velocity` (`entities.rs`) uses the 26.2 restitution
form: a gravity threshold (slow falls don't bounce), gravity/drag
compensation, and NO bed branch at all — items landing on beds don't
bounce. The 26.1 tree matched vanilla. NEW in 26.1.1. (Minor facet, same
area: the item-entity air drag changed from the double literal 0.98 to
0.98F widened — ~2e-8/tick, unobservable; recorded, not a separate item.)

## Re-checked, not defects

Nine 1.21.7 Batch 3 items were re-checked against 26.1.1's own jar and do
NOT carry over. (G07 was withdrawn in 1.21.7 — not re-minted.) Four
1.21.7 items DID re-mint (G03, G13, G14, G15 — see Defects).

- **G01 bat:** no Halloween branch in 26.1.1 vanilla; the port matches
  (y < surface, `nextBoolean`, brightness > `nextInt(4)`,
  `#bats_spawnable_on`, mob rules; `spawn_rules.rs:367`).
- **G02 slime:** vanilla uses the environment attribute
  `SURFACE_SLIME_SPAWN_CHANCE` with a single `nextFloat`; underground:
  slime-chunk seed 987234911, `nextInt(10)==0`, y < 40 — the port matches
  (`slime_rules`, `spawn_rules.rs:508+`). The slime arm keeps its own
  peaceful gate.
- **G08 drowned burns:** tag-driven; `#burn_in_daylight` holds exactly 10
  types including drowned, and the port's list matches
  (`species.rs:1543-1555`).
- **G10 far spawns:** the `EntityType.Builder` default is
  `canSpawnFarFromPlayer = (category == CREATURE || category == MISC)`;
  the port's `spawns_far_from_player` = Creature|Misc + Pillager|Shulker
  matches (pillager and shulker are the only explicit setters).
- **G12 enderman walk-target:** 26.1.1 `EnderMan` DOES override
  `getWalkTargetValue`, returning 0.0f; the port matches (walk-target
  table unchanged in the 26.1 diff).
- **G16 trades:** the jar has 561 `villager_trade` + 83 `trade_set` + 102
  tag files; the port's `Trades::load` (`merchant.rs:482`, identical to
  26.1) reads all three dirs.
- **G17 `reduced()`:** 26.1.1 `Goal` HAS `reducedTickDelay` =
  `Mth.positiveCeilDiv(ticks, 2)` and `adjustedTickDelay`; the port's
  `goals.rs:66-71` matches (unchanged in the diff).
- **G21 zombie spear:** vanilla `Zombie.populateDefaultEquipmentSlots`:
  `nextFloat < (hard ? 0.05 : 0.01)`, `nextInt(6)`: 0→IRON_SWORD,
  1→IRON_SPEAR, else IRON_SHOVEL; `iron_spear` exists in 26.1.1 items; the
  port matches in `spawning.rs:222` and `finalize.rs:1347` (both files
  identical to 26.1).
- **G07:** withdrawn in 1.21.7 — not re-minted.

**26.1-record corrections** (the 26.1 tree was wrong and its sweep missed
it; the 26.1.1 tree happens to be right — do NOT "fix" these back):

- **Mob equipment-pickup persistence:** vanilla 26.1.1
  `Mob.setItemSlotAndDropWhenKilled` DOES set `persistenceRequired = true`
  (`Mob.txt:1234`). The 26.1 port omitted it (its comment claimed "26.1
  does not" set it — wrong; `Mob` is not among the 3 changed classes, so
  vanilla 26.1 does too). The 26.1.1 port's `m.persistent = true`
  (`game/mob_looting.rs:124`) is CORRECT.
- **Spawn-egg peaceful gate:** vanilla 26.1.1 `SpawnEggItem.spawnMob`
  DOES gate on `!EntityType.isAllowedInPeaceful() && difficulty ==
  PEACEFUL → FAIL` (`SpawnEggItem.txt:139-140`). The 26.1 port removed the
  gate (its comment claimed "26.1 has no eligibility gate" — wrong). The
  26.1.1 port's `spawn_egg_allowed` (`game/creatures.rs`) is CORRECT in
  structure; its only error is the hoglin facet of G04.
- **Piglin brutes ARE fooled by piglin heads:** vanilla 26.1.1
  `LivingEntity.getVisibilityPercent` checks PIGLIN_BRUTE + PIGLIN_HEAD →
  ×0.5 (bytecode offsets 128–146). The 26.1 report's claim "brutes are not
  fooled (26.1 checks PIGLIN only)" is wrong. This matters for G33's
  vanilla anchor: all five head rules, including the brute one, are
  hardcoded in vanilla.

**26.2-record corrections** (per the 26.1 summary §"26.2-record
corrections", applied): B3-G27/G28/G25/G26 are shared verbatim with the
26.2 port, which passed those sections — the 26.2 sweep missed them. Per
the 26.1 slice's coordinator note, G05 is NOT flagged for the 26.2 record
(26.2's B3 did defect G05; the 26.1 report's claim there was wrong).

## Passes by section

### A1 — Spawn placements (1 check, PASS)

82 vanilla `SpawnPlacements.register` calls (ON_GROUND 59, IN_WATER 12,
NO_RESTRICTIONS 10, IN_LAVA 1; heightmaps MOTION_BLOCKING_NO_LEAVES 80,
MOTION_BLOCKING 2: ocelot, parrot), 38 distinct predicates. Versus
prepared `mob-data.json` placements (157 entries): ON_GROUND 59, IN_WATER
12, NO_RESTRICTIONS 85 (10 registered + 75 unregistered defaulting to it),
IN_LAVA 1; heightmaps MOTION_BLOCKING_NO_LEAVES 155, MOTION_BLOCKING 2 —
0 mismatches. The vanilla table is unchanged from 26.1 (no
`SpawnPlacements` class in the 3-class delta).

### A1b — Spawn predicates (11 checks, 10 pass, 1 gap)

Bat, slime, camel, drowned (including roll order: frequent→
`nextInt(15)==0`, else `nextInt(40)==0` && y < sea−5; spawner/reinforcement
early return), hoglin/piglin (below ≠ nether_wart_block only), guardian,
ghast, magma cube, zombified piglin, monster rules (`!peaceful &&
(ignoresLight || darkEnough) && mobRules` — outcome unchanged for
monsters), animal rules (`#animals_spawnable_on` + bright),
`isDarkEnoughToSpawn` (sky > `nextInt(32)`; block-light limit when < 15;
thundering darken 10 else sky_darken; `monsterSpawnLightTest.sample`),
bright = raw brightness > 8 — all PASS (arms unchanged in the 26.1 diff,
except the per-type peaceful gates noted below).

- **Mob rules / peaceful-gate placement (GAP — B3-G29):** the gate moved
  into `checkSpawnRules` (see Defects); phantom/shulker now fail in
  peaceful. The drowned/guardian/ghast/magma-cube/slime/zombified-piglin
  arms keep their own (now redundant) peaceful gates.

### A2 — Species coverage (1 check, PASS)

The port has 90 `Species` enum variants = 89 vanilla types + dead
`sulfur_cube`; 90 `Def` entries in `species.rs` `ALL`, all matching the
enum. Attribute suppliers missing from the port are only armor_stand,
ender_dragon, player (handled separately by design). Mannequin is a real
26.1.1 type (misc) with a supplier and a Def. (The port still carries dead
SulfurCube arms, e.g. `spawn_rules.rs:440`, `spawn_selection.rs:149/584` —
noted, not defects.)

### A3 — Attributes + species tables (11 checks, 6 pass, 5 gap)

- **Attributes registry (PASS):** 35/35 match prepared `attributes.json`
  (default/min/max/syncable). Note the vanilla field
  `SPAWN_REINFORCEMENTS_CHANCE` registers id `spawn_reinforcements`.
- **Entity attribute values (PASS):** 2,131 entity attribute values
  identical between `attributes.json[entities]` (hex f64 bit patterns)
  and `entity-types.json` (0 mismatches).
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
  handled at `mobs.rs:4676`). Unchanged in the 26.1 diff.
- **Food tags (GAP — B3-G05, B3-G27, B3-G28):** complete `isFood`
  declarer set from the jar (31 classes; `Animal.isFood` is abstract).
  All match the port's `food_tag()` except Camel (`#camel_food`),
  ZombieHorse (`#zombie_horse_food` = red mushroom), SkeletonHorse
  (inherits `#horse_food`). PolarBear returns false in vanilla and has no
  arm in the port — correct.
- **Peaceful-allowed table (GAP — B3-G04):** full-table comparison — 37
  vanilla `notInPeaceful` types vs the port's 38 (the extra one is
  hoglin). `EntityType.<clinit>` parsed to
  `evidence/entity-type-flags.json` (157 entries): fire_immune 15;
  `canSpawnFarFromPlayer` explicit setters only pillager + shulker;
  `immuneTo` 6 types with LITERAL blocks (the 1.21.7 form, not 26.2's
  tags); `spawnDimensionsScale` 4.0 only SLIME and MAGMA_CUBE; `noSummon()`
  only PLAYER and FISHING_BOBBER; Builder default `summon=true`.
- **Baby dimensions table (GAP — B3-G30):** see Defects.
- **Camel sitting dimensions (GAP — B3-G31):** see Defects.

### A4 — Daylight burning (5 checks, 4 pass, 1 gap)

- **Burn tag (PASS):** `Mob.aiStep` calls `burnUndead()` iff the type is
  in `#burn_in_daylight`; the tag holds exactly 10 types including
  drowned (jar `data/minecraft/tags/entity_type/burn_in_daylight.json`);
  the port's `burns_in_day` list (`species.rs:1543-1555`) matches exactly.
- **burnUndead mechanics (PASS):** helmet damage `nextInt(2)`, break at
  max damage, else `igniteForSeconds(8.0)`; the port ignites 160 ticks.
- **sunProtectionSlot (PASS):** Mob HEAD; ZombieHorse/ZombieNautilus
  BODY — the port's `body_slot_species` (`drops.rs:186`) covers both
  (`drops.rs` identical to 26.1).
- **isSunBurnTick (GAP — B3-G09):** powder-snow exclusion missing in the
  port (see Defects).
- **aiStep burn gate (PASS):** the tag check gates the burn call in both.

### B1 — Suffocation (3 checks, 1 pass, 2 gap)

- **Override table (PASS):** 15 `Blocks` `isSuffocating` calls resolved
  (BootstrapMethods: indy#8 = `Blocks.never`, indy#118 = `Blocks.always`).
  Always: DIRT_PATH, FARMLAND, MUD, SOUL_SAND. Never: MANGROVE_ROOTS,
  MOVING_PISTON, CHERRY_LEAVES, PALE_OAK_LEAVES, GLASS, TINTED_GLASS,
  COPPER_GRATE + `registerStainedGlass` helper + `leavesProperties`
  helper. NOT_CLOSED_SHULKER via `shulkerBoxProperties`;
  NOT_EXTENDED_PISTON via `pistonProperties`. The port's
  `mobs/suffocation.rs::bind` matches exactly (`suffocation.rs` identical
  to 26.1).
- **Default rule (GAP — B3-G22, major):** see Defects.
- **In-wall (GAP — B3-G23):** see Defects.

### B2 — Natural spawning (27 checks, 21 pass, 6 gap)

All verified against `NaturalSpawner`, `SpawnPlacements`,
`SpawnPlacementTypes`, `MobCategory`, `EntitySpawnReason`,
`PotentialCalculator`, `LocalMobCapCalculator`, `ServerChunkCache` dumps
and the 18 walk-target / 13 obstruction / 20 remove-far declarer dumps
(complete sets from the class-file scan).

- **MobCategory values (PASS):** max/friendly/persistent/despawn;
  `getNoDespawnDistance` = 32 const.
- **`spawnCategoryForPosition` order (GAP — B3-G13):** the order matches
  (redstone early return, 3 attempts, `ceil(nextFloat*4)` reassigned by
  `sample_count`, `nextInt(6)-nextInt(6)` jitter, nearest-player,
  right-distance, pick, `isValidSpawnPostitionForType`, predicate,
  getMobForSpawn null→return, yaw draw, `isValidPositionForMob`,
  finalizeSpawn, cluster-size return, `isMaxGroupSizeReached` break —
  including RNG draw order) EXCEPT constant-count packs skip a draw (see
  Defects).
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
  `feline_removed_when_far` on Entity.tickCount `m.age: u32`) and
  spawn-time `removed_when_far` both match. ZombieHorse/CamelHusk are
  MONSTER category, so the port's creature-animal clause does not wrongly
  keep them.
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
  for 24 blocks — all consistent.
- **`isSpawnPositionOk` (PASS):** `SpawnPlacementType` is an INTERFACE in
  26.1.1; NO_RESTRICTIONS always true with no border check;
  IN_WATER/IN_LAVA/ON_GROUND border-gated.
- **`isValidEmptySpawnBlock` / `isBlockDangerous` / `isBurningBlock`
  (GAP — B3-G03):** burning = #fire || lava || magma || lit campfire ||
  lava_cauldron; dangerous = !immuneTo && ((!fireImmune && burning) ||
  wither_rose/sweet_berry/cactus/powder_snow). The burning/dangerous
  tables match, but `immuneTo` now resolves against absent tags (see
  Defects), so the dangerous check mis-fires for the six immune types.
  The port's zombie-flag + recompute split is sound (no block is both
  burning and a signal source).
- **`getSpawnAABB` / `spawnDimensionsScale`, `canSummon`,
  `EntitySpawnReason` (PASS):** isSpawner = SPAWNER|TRIAL_SPAWNER;
  ignoresLightRequirements = TRIAL_SPAWNER only. (The checkSpawnRules
  peaceful-gate change is G29, counted in A1b.)
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
- **`spawnMobsForChunkGeneration` + `getTopNonCollidingPos` (GAP —
  B3-G32):** the creature list, SPAWN_MOBS rule, `nextFloat <
  creatureProbability` loop, 4 attempts with 5-jitter, skip/jitter rules
  and out-of-chunk re-roll all match; the heightmap predicate grounds on
  absent tags (see Defects).
- **Spawn-rules tables (PASS):** `checkSpawnRules` declarer set (Mob,
  PathfinderMob, SpawnPlacements), `checkDespawn` declarer set (5),
  `getMaxHeadYRot`/`getMaxHeadXRot` declarer sets — all from the
  class-file scan, all consistent with the port.
- **Inhabited time accrual (GAP — B3-G37):** see Defects. NEW check.
- **Fresh-spawn dimensions (GAP — B3-G38):** see Defects. NEW check.

### B7 — Spawners (4 checks, 3 pass, 1 gap)

- **BaseSpawner defaults + serverTick (PASS):** vanilla ctor spawnDelay
  20, minSpawnDelay 200, maxSpawnDelay 800, spawnCount 4,
  maxNearbyEntities 6, requiredPlayerRange 16, spawnRange 4; the port
  (`spawner.rs:317,350-355`) is identical. `delay()`: max<=min → min, else
  min + nextInt(max-min); the port (448-453) is identical. serverTick
  order verified (see the 26.1 report for the full order); the port's
  `rules_ok = custom.is_some() || mob_spawn_rules(...)` + always
  spawn_obstruction matches exactly.
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
  spawnMob matches the port's `spawn_mob` (`trial_spawner.rs:1029-1113`);
  the yaw draw position differs trivially (before obstruction vs after
  load) with no RNG consumer between. canSpawnInLevel = spawner_blocks_work
  && (override || (!peaceful && spawn_mobs)); the port omits the override
  — NOT a defect: a jar-wide constant-pool scan finds NO caller of
  `overridePeacefulAndMobSpawnRule()` in either 26.1.1 jar (dead in vanilla
  play); the 26.2 port has the same form. tryDetectPlayers cadence
  `(pos.asLong() + gameTime) % 20 != 0` → return; the port
  `packed.wrapping_add(l.game_time()) % 20 != 0` is identical.
  Cooldown+ominous → return; Bad Omen → Trial Omen transform + levelEvent
  3020 at eye pos; WAITING state machine matches the port. The
  `mobs/trial_spawner.rs` diff is ONLY the `override_entity_data`
  addition (G39).
- **Spawn-egg use on spawners (GAP — B3-G39):** see Defects. NEW check.

### B8 — Combat (3 checks, 1 pass, 2 gap)

- **Damage-after-armor rules (PASS):** vanilla `CombatRules.getDamageAfterAbsorb`:
  divisor 2+toughness/4; protection = clamp(armor − damage/divisor,
  armor*0.2, 20.0); effectiveness = protection/25; if weaponItem != null
  && level is ServerLevel: effectiveness =
  clamp(EnchantmentHelper.modifyArmorEffectiveness(...), 0, 1); return
  damage*(1−effectiveness). `getDamageAfterMagicAbsorb`: protection clamped
  [0,20]; damage*(1−protection/25). The port's
  `horizon-world/src/combat_rules.rs` `after_armor`/`after_magic` match
  exactly (identical to 26.1, including the NaN/−0.0-aware clamp mirroring
  Mth.clamp).
- **Slime dealDamage team check (GAP — B3-G41):** see Defects. NEW check.
- **Spear stab knockback (GAP — B3-G45):** see Defects. NEW check.

### B5 — Projectiles (4 checks, 3 pass, 1 gap)

- **Gravities (PASS):** AbstractArrow 0.05, ThrowableProjectile 0.03,
  LlamaSpit 0.06, ThrownExperienceBottle 0.07 (`throwableitemprojectile`
  pkg), AbstractThrownPotion 0.05, ShulkerBullet 0.04. The port
  (`projectiles.rs:400-412`) matches exactly; hurting projectiles 0.0
  (their tick applies acceleration, never gravity). `projectiles.rs` diff
  is comment-only vs 26.1.
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
  (`arrows.rs:776`): trident 0.99, else 0.6. `arrows.rs` identical to 26.1.
- **Item-entity landing bounce (GAP — B3-G46):** see Defects. NEW check.

### B4 — Trading (6 checks, 3 pass, 3 gap)

- **cost_a_count (PASS):** vanilla `getModifiedCostCount`: base + max(0,
  floor(base*demand*priceMultiplier)) + specialPriceDiff, clamped [1,
  maxStackSize]. The port (`merchant.rs:164-171`, identical to 26.1) is
  identical (int product base*demand first, then f32 multiply — matches
  the bytecode `imul; i2f; fmul; floor; max(0,...)`).
- **LEVEL_XP (PASS):** vanilla 26.1.1 thresholds live in
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
- **Trader hurt gating (GAP — B3-G34):** see Defects. NEW check.
- **Piglin offhand persistence (GAP — B3-G42):** see Defects. NEW check.

### B6 — Vehicles (3 checks, 2 pass, 1 gap)

- **Boat floatBoat + controlBoat (PASS):** vanilla AbstractBoat
  .getDefaultGravity = 0.04. floatBoat verified in full (see the 26.1
  report); the port's `tick_boat` (`boats.rs:728-892`, identical to 26.1)
  matches all of it. controlBoat: deltaRotation −1/+1 for left/right;
  right!=left && !up && !down adds 0.005 acceleration; up +0.04, down
  −0.005 along facing; paddling flags — all match. NOTE (not minted):
  vanilla getWaterLevelAbove scans y from floor(maxY) to ceil(maxY −
  lastYd) — the fall speed extends the scan upward; the port has no lastYd
  and scans one row, returning (y1+1) when full. A fast-falling boat's
  entry snap target can sit lower than vanilla's for one tick; buoyancy
  lifts it to the surface afterward. The 26.2 port is byte-identical here
  and passed B6. The port also halves landFriction on `controlled` (local
  driver) where vanilla checks the controlling passenger is a Player —
  same in local play, remote-only nuance.
- **Old minecart max speed (PASS):** vanilla OldMinecartBehavior
  .getMaxSpeed: inWater? 0.2: 0.4; getSlowdownFactor: isVehicle? 0.997:
  0.96. MinecartFurnace.getMaxSpeed: super * (inWater? 0.75: 0.5) — the
  override is on the cart, applied through the virtual. The port's
  `max_speed` (`minecarts.rs:750-757`) matches, and the old path uses it
  for the track cap (874) and comeOffTrack (1346). The old path is
  unchanged in the 26.1 diff (the minecart drag note: `comeOffTrack`
  drag 0.95f32-widened vs vanilla 0.95 double, ~1.2e-8/tick — recorded,
  not minted).
- **New minecart max speed (GAP — B3-G25):** see Defects.

### B3 — Mob AI (11 checks, 3 pass, 8 gap)

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
  rides (not minted: same observable effect — no goal starts). The
  alternation code is unchanged in the 26.1 diff.
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
  match the port's MeleeAttack (`goals.rs:1346+`, unchanged in the diff).
  NOTE (not minted): vanilla stop() clears the target when !NO_CREATIVE_OR_SPECTATOR
  .test(target) — i.e. creative OR spectator players; the port clears only
  creative players (`goals.rs:1450-1457`). The 26.2 port is identical and
  passed B3; the stale spectator target is overwritten by the next
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
- **Goal tick delays / G17 re-check (PASS):** 26.1.1 `Goal` HAS
  reducedTickDelay = positiveCeilDiv(ticks, 2) and adjustedTickDelay; the
  port's `goals.rs:66-71` matches. (1.21.7 G17 does not carry over.)
- **Enderman teleport + cat morning gift / G14/G15 re-check (GAP —
  B3-G14, B3-G15):** see Defects. Both re-minted in 26.1.1.
- **Visibility percent (GAP — B3-G33):** see Defects. NEW check.
- **BreathAirGoal (GAP — B3-G35):** see Defects. NEW check.
- **Cat lie-on-bed / sit-on-block goals (GAP — B3-G36):** see Defects.
  NEW check.
- **Armadillo ball-up (GAP — B3-G40):** see Defects. NEW check.
- **Sleeping villager position refresh (GAP — B3-G44):** see Defects. NEW
  check.
- **Fox chorus-fruit teleport particles (GAP — B3-G43):** see Defects. NEW
  check.

### B9 — Finalize spawn (3 checks, all PASS)

`finalize.rs` and `spawning.rs` are identical to 26.1.

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
  matches at every step.
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
table (`docs/sweep/26.1.1/batch-0-oracles.json`, 391 oracles, 189 pass /
202 fail-all); class + error count quoted from that table. **All 52 fail
identically in 26.1.1 as in 26.1** (same class, same n_errors, same first
error) — the oracle gap is unchanged, as expected: the 26.1→26.1.1 vanilla
delta touches no entity API.

- **B3-O01 — missing-override `fuelValues()` (3 oracles, 1 error each,
  Batch 0 group G1):** VanillaActiveEffectTickOracle,
  VanillaAttributeUpdatesOracle, VanillaLivingLandingOracle — "Client*
  Level is not abstract and does not override abstract method fuelValues()
  in Level". 26.1.1's `Level` declares `public abstract FuelValues
  fuelValues()` (verified in the dump, `Level.txt:1929`); the oracles'
  26.3-form level stubs lack it. Mechanical repair: add a `fuelValues()`
  override. Same group as 26.2.
- **B3-O02 — constructor mismatch (5 oracles):** VanillaAllayInteraction
  StateOracle (n=3), VanillaPlayerSyncOracle (n=1),
  VanillaRemoteAttackOracle (n=8), VanillaSheepDyeOracle (n=4) —
  "constructor LocalPlayer in class LocalPlayer cannot be applied to
  given types"; 26.1.1's `LocalPlayer` ctor takes 8 arguments (Minecraft,
  ClientLevel, ClientPacketListener, StatsCounter, ClientRecipeBook,
  Input, boolean, ChatAbilities), 26.3's takes 9. Plus
  VanillaEntitySoundPacketOracle (n=3) — "constructor MinecraftServer
  ... cannot be applied to given types"; 26.1.1's `MinecraftServer` ctor
  takes 10 arguments. Note VanillaBoatRiderOracle sits in B3-O03 in 26.1.1
  (no-override first, n=4) but carries the LocalPlayer ctor error behind
  it.
- **B3-O03 — no-override (14 oracles):** VanillaArrowOracle (n=1),
  VanillaBlockingItemDamageOracle (n=2), VanillaBoatRiderOracle (n=4),
  VanillaClientArrowOracle (n=1), VanillaDamageImmunityOracle (n=2),
  VanillaEntityDeathLootOracle (n=4), VanillaFinalizeSpawnOracle (n=7),
  VanillaLateEntityPairOracle (n=4), VanillaMobAttributesOracle (n=1),
  VanillaNaturalSpawnOracle (n=16), VanillaPlayerTravelOracle (n=1),
  VanillaRegisteredPlayerCombatOracle (n=6),
  VanillaServerPlayerCombatOracle (n=6), VanillaSpawnRulesOracle (n=1) —
  "method does not override or implement a method from a supertype" (26.3
  methods absent in 26.1.1). NaturalSpawn (n=16) also carries
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
  in either 26.1.1 jar; registration still lives on `EntityType`/
  `BlockEntityType`. (VanillaMobAiOracle and VanillaTradingOracle also
  reference `EntityTypes` — see B3-O05.) NEW group for 26.1 — 26.2 has
  the holder.
- **B3-O05 — missing-symbol, other 26.x APIs (16 oracles):**
  VanillaDragonContactsOracle (n=3; `Enderman` — the 26.3 renamed class;
  26.1.1 has `EnderMan`, verified by jar listing), VanillaEntityMoveOracle
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
  26.1.1's packet is a plain class with private fields and NO public
  accessors; 26.3 made it a record). `BlockTags` in 26.1.1 has 248 fields
  (verified in the dump), none of CAUSES_SUFFOCATION / SUPPRESSES_BOUNCE /
  BLOCKS_FLUID_FLOW. Other jar-verified absences behind these errors:
  `SwingState`, `cubemob` package, `ServerboundPunchPacket`,
  `MoveSimulationType`, `EntitySpawnRequest`, `NoiseSpawnFinder`,
  `advancements.triggers`/`predicates` subpackages (only `criterion`
  exists, 168 entries), top-level `ClockInstance` (nested
  `ServerClockManager$ClockInstance` only).
- **B3-O06 — missing-package (2 oracles):** VanillaRaidOracle (n=11;
  `package net.minecraft.advancements.triggers does not exist`),
  VanillaEntityComponentsOracle (n=3; `package
  net.minecraft.advancements.predicates.entity does not exist`). 26.1.1's
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

**1.21.7 oracle groups re-checked (B3-O01–O06, 1.21.7 numbering):** none
mint in 26.1.1. No gamerules-package failures anywhere in the Batch 0
table (26.1.1 has the package); no `world.clock` failures; no entity-
subpackage first-errors; the only Batch 3-scope `client.multiplayer`
failure is VanillaZombieCureSoundOracle's expected server-only fail (it
passes client+both). VanillaEntityParticlesOracle fails server-only on
`client.multiplayer` — particles, Batch 10's area; flagged in
`out_of_scope_failures`.

Adjacent, not Batch 3: VanillaBlockEntityAnimOracle/
VanillaBlockEntityRenderOracle (Batch 4), VanillaFeatureCatalogOracle,
VanillaItemUseCatalogOracle, VanillaRedstoneCatalogOracle (Batch 2),
VanillaEntityAnimOracle/VanillaDragonRenderOracle (Batch 10),
VanillaAudioAttributesOracle (missing-symbol n=1),
VanillaTeleportConsumeOracle (FAILS in 26.1.1, no-override n=5 — Batch 7
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
lists in `evidence/drivers-curated.json` (26.1's lists; the 26.1.1
`tools/` tree differs only in `test_brewing_parity.py`,
`test_furnace_parity.py` and a few others — none in Batch 3 scope). No
driver was run (static analysis only). Driver checks: 3 — 0 pass, 3 gap
(B3-G18, B3-G19, B3-G20, found by reading). All three are gaps in 26.1.1,
as in 26.1; in 26.2 G18 passed.

Stale 26.3 docstrings only (no functional effect): dragon_contacts,
mob_discard, mob_quad_leash, mount_control, vehicle_container,
villager_farming, zombie_cure_sound, fishing_rng (borderline),
predicate_reference_startup (comment only — dropped from scope as a
false positive, but its lines 78/80 DO build a functional 26.3 server
command; flagged for whichever batch owns it).
`test_spawn_loot_parity.py:34` hardcodes
`RESOURCES = ROOT / '.local' / 'horizon-client' / '26.3'` (functional,
borderline scope — loot, Batch 8's area; noted here, not defected).
`test_brewing_parity.py:129` gained a 26.3 resources path vs 26.1
(brewing, Batch 4/5's area; noted in `out_of_scope_failures`).

## Method notes

- **VANILLA-CHECK RULE throughout:** every behaviour claim was verified
  against 26.1.1's OWN vanilla jar/data, never the 26.3 base's form. Where
  26.1.1's form differs from both 1.21.7 and 26.2 (hoglin NOT
  `notInPeaceful`; literal `immuneTo` blocks; `SpawnPlacementType` an
  interface; no `EntityTypes` holder; no `densityfunction` package; no
  teleport tags; `EnderMan` not `Enderman`; `ClientboundUpdateAdvancementsPacket`
  a plain class, not a record; `AbstractBedBlock` absent — only plain
  `BedBlock`), the report says so explicitly.
- **Diff-first method:** every Batch 3 file was diffed 26.1-vs-26.1.1
  (`evidence/diff-*.txt`). IDENTICAL: `mobs/finalize.rs`, `merchant.rs`,
  `arrows.rs`, `game/boats.rs`, `game/trial_spawner.rs`,
  `horizon-world/src/suffocation.rs`, `horizon-world/src/combat_rules.rs`,
  `mobs/spawning.rs`, `mobs/leash.rs`, `mobs/drops.rs`, `mobs/raider.rs`,
  `mobs/zombie_villager.rs`, `mobs/villager_brain.rs`, all 8 Batch 3
  drivers. DIFFERS (reviewed): `mobs/spawn_rules.rs`, `species.rs`,
  `mobs.rs`, `mobs/ai.rs`, `mobs/spawn_selection.rs`, `mobs/goals.rs`,
  `mobs/tame.rs`, `mobs/enderman.rs`, `mobs/trader.rs`,
  `mobs/trial_spawner.rs`, `mobs/villager.rs`, `mobs/world.rs`,
  `mobs/armadillo.rs`, `mobs/camel.rs`, `mobs/cube.rs`, `mobs/fox.rs`,
  `mobs/piglin.rs`, `mobs/spear_use.rs`, `mobs/anim.rs` (render — flag
  B10), `game/minecarts.rs`, `game/spawner.rs`, `game/natural_spawning.rs`,
  `game/creatures.rs`, `game/passengers.rs`, `game/stations.rs`,
  `game/dragon.rs`, `game/initial_spawn.rs`, `game/entity.rs`,
  `entities.rs`, `entity_rules.rs`, `game/melee.rs`, `game/mounts.rs`,
  `game/mob_looting.rs`, `game/combat.rs`, `game/remote_mob_data.rs`,
  `game/remote_vehicles.rs`, `game/lan_vehicles.rs`,
  `horizon-client-render/src/world.rs`. Comment/doc-only diffs:
  `mobs/living.rs`, `mobs/sulfur_cube.rs`, `projectiles.rs`, `motion.rs`,
  `game/enderman.rs`, `game/mounts.rs`, `game/remote_mob_data.rs`.
- **Systemic finding:** the 26.1.1 tree did NOT inherit the 26.1 tree's
  behaviour fixes — it carries 26.2/26.3-form behaviour the 26.1 tree
  deliberately removed (same pattern Batch 2 reported for level/
  behaviour, and Batch 1 for fluids). The regressions cluster in exactly
  the places where 26.1's form differs from 26.2/26.3: tag-grounded
  behaviour for tags 26.1.1 lacks (G03, G14, G15, G32, G36, G44 — six
  items), component-form behaviour for components 26.1.1 data lacks (G33),
  26.2-form tables (G30, G31, G46), moved gates (G29), and dropped
  26.1-only code paths (G13, G35, G37, G38, G40, G41, G42, G43, G45).
  `PORT-NOTES-26.1.1.md`'s carry-forward claim is contradicted by the
  diffs. Two of the 26.1 tree's "corrections" were themselves wrong (see
  "26.1-record corrections") — the 26.1.1 tree is right on those two by
  accident of carrying the 26.2 form.
- **Class-file scan:** `evidence/baby_dims2.py` (JVM-free parser) over the
  26.1.1 server-game jar extracted the 20 `BABY_DIMENSIONS` declarers;
  `EntityType.<clinit>` was parsed to `evidence/entity-type-flags.json`
  (157 entries); `Attributes.<clinit>` to
  `evidence/attributes-vanilla.json` (35 attributes);
  `SpawnPlacements` registrations to
  `evidence/spawn-placements-vanilla.json` (82 calls, 38 predicates) —
  the 26.1 evidence files carry over unchanged (no entity class in the
  3-class vanilla delta), and the port-side tables were re-verified
  against the 26.1.1 prepared data.
- **javap is slow on this VM** (~10–15 s/class under load); ~35 fresh
  dumps in `evidence/javap/` were run in background batches. Never javap
  the bundler `server.jar` or the leakage-audit 26.1.x server jars (no game
  classes) — only the named `client.jar` / `server-game.jar`. Caution: in
  one javap batch the `EntityType$Builder` dump overwrote `EntityType.txt`
  (a sed naming bug) — the 4267-line EntityType dump is gone, but all
  needed data had already been extracted into
  `evidence/entity-type-flags.json`.
- **Tally convention:** behaviour checks only; oracle triage has its own
  counts. A check is one vanilla-vs-port comparison of a named behaviour;
  a check that finds multiple defects (the A3 food table: G05, G27, G28;
  the B3 enderman/cat check: G14, G15) counts once in the tally but yields
  one slice item per defect. Driver-code-defect checks are counted
  as-tallied per the 26.2 precedent, with the behaviour/non-behaviour
  split reported explicitly (93 behaviour + 3 driver).
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
  diverge — same note as 26.2); minecart `comeOffTrack` drag 0.95f32-
  widened vs vanilla 0.95 double (~1.2e-8/tick, unobservable); air drag
  0.98f32-widened vs vanilla 0.98 double in `entity_rules.rs`
  (TNT/falling blocks) and the item-entity drag (same negligible class);
  `can_be_a_baby` body identical between trees (doc comment only);
  `villager_food` component form harmless (prepared data has 0
  `villager_food` components; the fallback is vanilla's four foods);
  `pos_to_bed(p, 0.5625)` = 0.6875 = vanilla; player sleep path fine
  (shape-measured height coincides with vanilla's flat 0.6875 for normal
  beds); `layer_prefixes` diffs are render (Batch 10); `anim.rs`
  zombie-arms baby change is animation (flag for Batch 10 in
  `out_of_scope_failures`); projectiles/motion/`game/enderman.rs` diffs
  are comment-only; the cushion entity additions (`game/entity.rs`,
  `game/passengers.rs`, `game/remote_vehicles.rs`, `game/lan_vehicles.rs`,
  `game/melee.rs` `attack_cushion`) are dead code for 26.1.1 (no cushion
  type in 26.1.1 data); the dismount functions moved from
  `game/passengers.rs` to `game/cushions.rs` with identical content (file
  move only); `game/dragon.rs` diff is comment-only plus a test-fixture
  field change; `game/control.rs` diff is ShareToLan UI only.
- **Findings shared with the 26.2 port (flag for the 26.2 record):** the
  food-table gaps G27, G28 (byte-identical `food_tag()`; G05 is NOT
  flagged — 26.2's B3 defected it), BreedGoal panic (G26 — identical
  code), furnace minecart cap (G25 — identical code at all three sites),
  MeleeAttack spectator-target note, boat lastYd note, hurting-projectile
  normalize note.
- **Out-of-scope findings for other batches** (in the slice's
  `out_of_scope_failures`): 35723 hardcodes in 34 other drivers; 26.3
  resources paths in 30 other drivers (incl. `test_brewing_parity.py`,
  newly gained); `anim.rs` zombie-arms baby change + `layer_prefixes`
  diffs (Batch 10, render); `game/stations.rs` brewing-totals change
  (Batch 4/5); `game/initial_spawn.rs` skylight-gate removal (Batch 7,
  player spawn); player chorus-fruit 2017 trail (Batch 7);
  VanillaEntityParticlesOracle server-only `client.multiplayer` failure
  (Batch 10); `#suppresses_bounce` is the first-failing absent tag on the
  26.1.1 startup path (landing/motion area).
- **UNVERIFIED carry-overs: none.** Every 26.1 and 1.21.7 defect was
  re-checked against 26.1.1's own jar and is either a verified defect
  above or listed under "Re-checked, not defects" with its evidence.
