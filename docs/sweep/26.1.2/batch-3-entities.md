# Batch 3 — Entities / mobs (26.1.2 parity sweep)

**Version:** 26.1.2 · **Date:** 2026-11-25 · **Branch:** `parity-sweep-26.1.2`
(workdir `~/workspace/mcir-work/sweep-26.1.2`) · **Scope:**
`docs/PARITY-SWEEP.md` §4 Batch 3 — mob AI, spawning, trading, attributes,
projectiles, vehicles; 157 entity types (44 monster, 31 creature, 72 misc,
4 water ambient, 3 water creature, 1 axolotl, 1 ambient, 1 underground
water creature); 144 drivers curated to 135 core (see §D).

**Status: checks + triage only, per the brief: no fixes, no edits under
`versions/26.1.2/`, no cargo builds, no driver runs, no oracle repairs, no
commits — the coordinator commits these deliverables.** Everything below is
static analysis: `javap -c` on the named 26.1.2 server-game jar
(`~/workspace/mcir-work/26.1.x/versions/26.1.2/jars/server-game.jar`,
sha1 `83eb1106f12f24c9127ce22070d71853157af127`, DataVersion 4790, Java 25;
client.jar sha1 `4e618f09a0c649dde3fdf829df443ce0b8831e65`), source comparison
against `versions/26.1.2/crates/`, and checks of the prepared 26.1.2 data
(`versions/26.1.2/.local/horizon-client/26.1.2/`, Batch 0's prepare).

**Method — jardiff-first, then diff-first.** Two measured facts frame the
whole batch, both established before any verdict (evidence in
`evidence/jardiff-26.1.1-26.1.2.txt` and
`evidence/tree-diff-26.1.1-26.1.2.txt`):

1. **Vanilla delta is 3 classes in server-game** (7,351 classes each side,
   0 added, 0 removed): `DetectedVersion`, `SharedConstants`, and
   `ServerGamePacketListenerImpl` (client jar: those 3 + the Checkbox /
   AbstractReportScreen families, 8 changed of 10,682). **No entity class
   changed between 26.1.1 and 26.1.2.** Every vanilla-side entity anchor
   from the 26.1.1 run therefore carries byte-identically — and every one
   below was still re-verified fresh against the 26.1.2 jar (57 fresh
   `javap` dumps in `evidence/javap/`, plus a fresh class-parser
   extraction of the 20 `BABY_DIMENSIONS` declarers), never assumed.
2. **Tree delta is 32 differing files, none in Batch 3 scope.** All Batch 3
   files (`mobs/`, `game/minecarts.rs`, `game/spawner.rs`,
   `game/natural_spawning.rs`, `tools/` drivers, `horizon-world/src/
   suffocation.rs`, `combat_rules.rs`, `entities.rs`, `projectiles.rs`,
   `arrows.rs`, `mods/` oracle sources) are byte-identical to the 26.1.1
   tree. The only behaviour-adjacent differences are `lan/hub.rs` (the
   `handleAttack` spectator port — see B8, a PASS, not a defect),
   `lan/tracking.rs` (a thrown-type fallback removal for unknown kinds —
   noted, not a defect), and out-of-scope files (item sign-text kinds,
   chunk palette NBT, save DataVersion thresholds — flagged in
   `out_of_scope_failures`).

**Re-check first:** the twelve 26.1 regressions were re-checked first, per
the brief. The two in this batch's scope — B3-G14 (endermen never
teleport) and B3-G22 (the `#causes_suffocation` startup blocker) — both
**REPRODUCE** with fresh 26.1.2 evidence (see Defects). All 45 of the
26.1.1 run's Batch 3 items reproduce; nothing was fixed, withdrawn, or
left unverified. One NEW defect was found (B3-G47, AI walk-target value —
missed by the 26.1 and 26.1.1 sweeps, which checked only the spawn table),
and one NEW check passes (B8 `handleAttack`, the delta's one behaviour
item — the port carries it in the LAN hub, contradicting
`PORT-NOTES-26.1.2.md`'s "remains unported" flag).

## Tally

- **Behaviour checks: 98 — 63 pass, 35 gap as-tallied** (38 defect items:
  B3-G03, G04, G05, G06, G09, G11, G13, G14, G15, G18, G19, G20, G22, G23,
  G24, G25, G26, G27, G28, G29–G47 — the A3 food-table check yields three
  items and the B3 enderman/cat check yields two; every other check yields
  one). **Behaviour/non-behaviour split:** 95 behaviour checks (63 pass,
  32 gap) + 3 non-behaviour driver-code checks (0 pass, 3 gap: G18, G19,
  G20) — driver-code-defect checks counted as-tallied per the 26.2
  precedent, split reported explicitly.
- **Oracle defects: 8 groups** (B3-O01–B3-O08, 52 oracles — not counted in
  the check tally).
- **Oracle repairs done: none** (checks + triage only).
- **Severity: 2 major, 44 minor** — 2 major port (B3-G22, the
  `#causes_suffocation` startup blocker; B3-G14, endermen never teleport),
  36 minor port (incl. new B3-G47), 3 minor driver-side (B3-G18, G19,
  G20), 8 minor oracle groups.
- **Carry-over:** all 45 of 26.1.1's Batch 3 defect items reproduce in
  26.1.2 (fresh items, same IDs, fresh evidence). 12 of the 21 1.21.7
  behaviour items re-mint (G03, G04, G05, G06, G09, G11, G13, G14, G15,
  G18, G19, G20); 8 do not carry (G01, G02, G08, G10, G12 as framed, G16,
  G17, G21 — see "Re-checked, not defects"); G07 stays withdrawn. 1 NEW
  defect (G47), 1 NEW passing check (B8 `handleAttack`).

Per-section counts (must sum to 98): A1 1 (1 pass), A1b 11 (10 pass,
1 gap), A2 1 (1 pass), A3 11 (6 pass, 5 gap), A4 5 (4 pass, 1 gap),
B1 3 (1 pass, 2 gap), B2 27 (21 pass, 6 gap), B7 4 (3 pass, 1 gap),
B8 4 (2 pass, 2 gap), B5 4 (3 pass, 1 gap), B4 6 (3 pass, 3 gap),
B6 3 (2 pass, 1 gap), B3 12 (3 pass, 9 gap), B9 3 (3 pass),
Drivers 3 (0 pass, 3 gap).

## Defects

### B3-G03 — `immune_tag` tags don't exist (minor, port)

Vanilla 26.1.2 `EntityType.<clinit>` passes LITERAL block sets to
`immuneTo` (fox→sweet_berry_bush; polar_bear/snow_golem/stray→powder_snow;
wither/wither_skeleton→wither_rose — 6 calls, verified in the fresh dump).
The 26.1.2 port's `immune_tag` (`mobs/spawn_rules.rs:179-190`) resolves
`#fox_immune_to`, `#polar_bear_immune_to`, `#snow_golem_immune_to`,
`#stray_immune_to`, `#wither_immune_to`, `#wither_skeleton_immune_to` —
all six absent as BLOCK tags from the 26.1.2 jar data and from prepared
`block-tags.json` (248 tags), so `immune_to` answers false for every
block: foxes take sweet berry bush damage, strays freeze in powder snow,
withers take wither-rose damage. (Note: `wither_immune_to` DOES exist in
the jar — as a `damage_type` tag, not a block tag — so the port's block-tag
lookup still answers false.) The 26.1 tree had the correct literal form.
Re-minted from 1.21.7 (26.1 had cleared it). Reproduces from 26.1.1.

### B3-G04 — Hoglin wrongly disallowed in peaceful (minor, port)

Vanilla 26.1.2 `EntityType.<clinit>` marks 37 types `notInPeaceful`;
hoglin is NOT among them (verified directly in the fresh dump: the HOGLIN
registration block, offsets 3067–3099, has no `notInPeaceful` call — the
one at 3047 belongs to GUARDIAN). The port's `allowed_in_peaceful`
disallows 38 types including hoglin, in BOTH `mobs/spawn_rules.rs:152`
and `mobs/finalize.rs:897` (identical to 26.1.1). Live paths: `ai.rs`
checkDespawn, `game/commands.rs`, `passengers.rs`, `stations.rs`,
`structure_mobs.rs` — hoglins despawn / refuse to spawn in peaceful in
the port where vanilla keeps them. (Opposite of 26.2, where hoglin IS
`notInPeaceful`.) Reproduces from 26.1.1. The hoglin facet also reaches
the spawn-egg gate (`game/creatures.rs` `spawn_egg_allowed`): hoglin eggs
are refused in peaceful in the port where vanilla allows them (see
"Re-checked, not defects" — the gate itself is correct).

### B3-G05 — Camel has no food (minor, port)

Vanilla 26.1.2 `Camel.isFood` = `#camel_food` (cactus; the tag exists in
the jar's data — verified in the fresh Camel dump). Port `food_tag()`
(`mobs/species.rs:1633-1665`) covers `CamelHusk` (`#camel_husk_food`,
verified correct against the fresh CamelHusk dump) but NOT `Camel`, so
`MobType.food` is empty for camels and feeding/breeding/tempting never
fire. Camels cannot breed. Reproduces from 26.1.1 (and shared with the
26.2 port's identical table; per the 26.1 slice's coordinator note, G05
is NOT flagged for the 26.2 record — 26.2's B3 defected it).

### B3-G06 — Armadillo scared head rotation ignored (minor, port)

Vanilla 26.1.2 `Armadillo.getMaxHeadYRot = isScared() ? 0 : 32` (verified
in the fresh dump: `isScared()` at offset 1, `iconst_0` vs `bipush 32`).
Port `max_head_y_rot` (`mobs/species.rs:1712-1725`) returns 32.0
unconditionally (`Species::Armadillo => 32.0`), so a scared (rolled-up)
armadillo still turns its head. Reproduces from 26.1.1.

### B3-G09 — Sun-burn ignores powder snow (minor, port)

Vanilla 26.1.2 `Mob.isSunBurnTick` excludes mobs that are `isInWaterOrRain
|| isInPowderSnow || wasInPowderSnow` (verified in the fresh dump,
offsets 58–75). The port's `sun_burn_tick` (`mobs/ai.rs:2024-2038`) uses
`wet = in_water || in_rain` only, although `Mob` HAS `in_powder_snow`
state — undead standing in powder snow burn at noon in the port where
vanilla spares them. Reproduces from 26.1.1.

### B3-G11 — `getRandomPosWithin` y-range off by one (minor, port)

Vanilla 26.1.2 `NaturalSpawner.spawnCategoryForChunk` computes
top = `getHeight(WORLD_SURFACE) + 1` (verified in the fresh dump:
`iconst_1; iadd` at offsets 16–17) and samples
y = `randomBetweenInclusive(minY, top)` → [minY, H+1]. The port's
`spawn_category_for_chunk` (`mobs/spawn_selection.rs:1092-1105`) uses
top = H and y = min + `nextInt(top-min+1)` → [minY, H], so the topmost
spawnable y is never chosen. Reproduces from 26.1.1.

### B3-G13 — Constant-count packs skip an RNG draw (minor, port)

Vanilla 26.1.2 `SpawnerData` is a plain record (minCount, maxCount —
verified in the fresh dump) and the sample site draws unconditionally:
`minCount + nextInt(1 + maxCount - minCount)` — even when min == max. The
26.1.2 port's `sample_count` (`mobs/spawn_selection.rs:782-789`) returns
the constant without drawing for constant entries, so every constant-count
pack skips one RNG draw and all downstream draws in the spawn sequence
shift. The 26.1 tree drew unconditionally. Re-minted from 1.21.7 (26.1 had
cleared it). Reproduces from 26.1.1.

### B3-G14 — Endermen never teleport (MAJOR, port)

Vanilla 26.1.2 `EnderMan.teleport` + `LivingEntity.randomTeleport` ground
on `blocksMotion()` (verified in the fresh EnderMan dump: `blocksMotion()`
at offsets 37 and 68 of `teleport(DDD)`); no teleport tags exist in
26.1.2 data (verified by jar listing: `entities_can_teleport_to`,
`dangerous_for_teleportation`, `enderman_does_not_teleport_to` all absent).
The 26.1.2 port's `mobs/enderman.rs:117` grounds teleport on the absent
`#entities_can_teleport_to` tag, so no destination ever qualifies and
endermen never teleport. The 26.1 tree used blocksMotion. Re-minted from
1.21.7 (26.1 had cleared it). Reproduces from 26.1.1. One of the twelve
26.1 regressions, re-checked FIRST per the brief.

### B3-G15 — Cat morning gift never happens (minor, port)

Three stacked failures, all in `mobs/tame.rs`:
1. The `CatRelaxOnOwner` goal's `can_use` (tame.rs:1270) gates on the
   absent `#cats_can_lie_on` tag at the sleeping owner's position, so the
   goal never starts. Vanilla `CatLieOnBedGoal.isValidTarget` checks
   `BlockTags.BEDS` (present in 26.1.2 data — verified in the fresh dump).
2. `give_morning_gift` (tame.rs:1350) drops the leash-holder origin —
   vanilla `Cat$CatRelaxOnOwnerGoal.giveMorningGift` teleports from the
   leash holder's block when the cat is leashed (`getLeashHolder()`,
   verified in the fresh dump); the port always uses the cat's own
   position.
3. The gift hop itself grounds on the absent
   `#entities_can_teleport_to` tag (tame.rs:1363 — the scan loop walks
   down to the world bottom and never finds a tagged block, so the hop
   never executes); vanilla uses `randomTeleport(x±5, y±2, z±5, false)`.

The 26.1 tree had the leash-holder origin + `enderman::random_teleport`.
Re-minted from 1.21.7 (26.1 had cleared it). Reproduces from 26.1.1.

### B3-G18 — DRIVER: entity_components predicate form (minor, driver)

`tools/test_entity_components_parity.py:27-29` reads
`criterion['conditions']['entity']['predicate']` and then
`predicate['minecraft:components']`. 26.1.2 advancement data uses the list
form with `predicate: {components: {...}}` — verified fresh in the
26.1.2 client jar: `complete_catalogue` (11 criteria), `whole_pack` (9),
`leash_all_frog_variants` (3) all carry `conditions.entity` as a LIST of
`entity_properties` conditions (121 list-form entity conditions
tree-wide; 0 dict-form). The driver raises TypeError on the first
criterion. (26.2 uses the new form; 26.1.2 the old — the driver was written
for 26.2/26.3 data.) Driver file identical to 26.1.1. Reproduces from
26.1.1; in 26.2 this check PASSED.

### B3-G19 — DRIVER: state-count hardcode 35723 (minor, driver)

Two Batch 3 drivers hardcode the 26.3 block-state count:
`test_suffocation_parity.py:66` and
`test_registered_entity_fluid_parity.py:66` both assert the state count
equals 35723. 26.1.2 has 29,873 states (`mob-data.json`), so both drivers
fail their own completeness check. (In 26.2 the suffocation driver was
fixed to 32366; in 26.1.2 it was not.) Driver files identical to 26.1.1.
34 further drivers tree-wide hardcode 35723 (36 tree-wide total, re-counted
fresh) — listed in the slice's `out_of_scope_failures`, not defected here.
Reproduces from 26.1.1.

### B3-G20 — DRIVER: 26.3 resources path (minor, driver)

Five Batch 3 drivers build the server command with
`lab.LOCAL / 'horizon-client/26.3'`:
`test_locator_bar_parity.py:269`, `test_server_dragon_parity.py:111`,
`test_server_entities_parity.py:186`,
`test_server_knockback_parity.py:435`, `test_projectile_shoot.py:46` — so
they run the horizon server against 26.3 prepared data (or fail outright
where none exists) instead of 26.1.2's. Driver files identical to 26.1.1.
30 further drivers tree-wide use a 26.3 resources path (35 tree-wide
total, re-counted fresh with all path forms) — listed in the slice's
`out_of_scope_failures`. Reproduces from 26.1.1.

### B3-G22 — Default suffocation is tag-gated on absent `#causes_suffocation` (MAJOR, port)

Vanilla 26.1.2's default suffocation rule is `Properties.<init>` indy#6 →
`lambda$new$5` = `blocksMotion() && isCollisionShapeFullBlock()` — NO tag
(verified in the fresh BlockProperties dump). The port's default is
`TaggedFullCollision` (`horizon-world/src/suffocation.rs:72-73`, identical
to 26.1.1) gated on `#causes_suffocation`, a tag absent from 26.1.2 jar
data (0 entries) and from prepared `block-tags.json` (248 tags).
`motion.rs:139` requires it via `tag(...)?`, which errors on missing tags;
`TagDefinitions::build` synthesizes nothing. The startup path
`resource_load.rs load_resources` → `load_core` (151) → `gameplay_tables`
(291) → `MotionResources::load_with_facts` (324) propagates the error —
**the 26.1.2 client cannot load its core tables with prepared 26.1.2
data**. The first failure in evaluation order is actually
`#suppresses_bounce` (`motion.rs:136`, also absent), then
`#causes_suffocation`. Even if load passed, no default block would
suffocate. Contrast: 26.2's `motion.rs:65` documents "26.3 replaced this
with the `#causes_suffocation` tag, which 26.2 data has not" and uses
`blocks_motion` + `MotionFullCollision`. CI crate tests do not catch this:
data-dependent tests skip quietly without `.local`. Reproduces from
26.1.1 (Batch 1 independently flagged the same reproduction there). One
of the twelve 26.1 regressions, re-checked FIRST per the brief.

### B3-G23 — Sleeping mobs suffocate (minor, port)

Vanilla 26.1.2 `LivingEntity.isInWall` = `!isSleeping() &&
super.isInWall()` (verified in the fresh dump: `isSleeping()` at offset 1,
`ifne 18`); `Entity.isInWall` = noPhysics→false, eye box
`AABB.ofSize(eye, width*0.8, 1e-6, width*0.8)`, any block `!isAir &&
isSuffocating && collisionShape ∩ box`; `baseTick` hurts 1.0 InWall. The
port's `ai.rs base_tick` (in-wall check at 1267) matches the geometry,
damage, and Vex exclusion but has NO sleeping check — a sleeper whose eye
is inside a suffocating shape takes in-wall damage in the port. Edge case.
Reproduces from 26.1.1.

### B3-G24 — Villager level-up is immediate and new offers are discounted (minor, port)

Vanilla 26.1.2 `rewardTradeXp` does NOT level up on trade: it sets
`updateMerchantTimer = 40` and `increaseProfessionLevelOnUpdate = true`
(pop += 5) — verified in the fresh Villager dump (`bipush 40` at offset
42). `customServerAiStep` decrements the timer ONLY while not trading
(isTrading gate); at 0 it calls `increaseMerchantCareer` (level+1 +
updateTrades) — with NO `updateSpecialPrices` call, so new offers carry
`specialPriceDiff` 0 until the next session. The 26.1.2 port levels up
IMMEDIATELY in `Trader::notify_trade` (`merchant.rs:399-407`, identical to
26.1.1) and recomputes special prices for ALL offers including the new
ones, with the session-start reputation/hero. Observable: newly-unlocked
trades show discounted prices mid-session in the port (full price in
vanilla), and the level-up appears at once instead of ~2s after trading
ends. The 26.2 port has the correct deferred form (`level_up_pending`,
`merchant.rs:413` + `trading.rs:333`). No `level_up_pending` /
`update_merchant_timer` exists anywhere in the 26.1.2 port. Reproduces
from 26.1.1.

### B3-G25 — Furnace minecart too fast under `minecart_improvements` (minor, port)

Vanilla 26.1.2 `NewMinecartBehavior.getMaxSpeed` = `MAX_MINECART_SPEED *
(inWater ? 0.5 : 1.0) / 20.0` (rule default 8, min 1, max 1000; the port's
formula in `max_speed_new` (991-994) matches). But
`AbstractMinecart.getMaxSpeed` delegates to the behavior, and
`MinecartFurnace.getMaxSpeed` multiplies the result by 0.75 (wet) / 0.5
(dry) under EITHER behavior (verified in the fresh MinecartFurnace dump:
`0.75d`/`0.5d` with `dmul`). Vanilla calls the VIRTUAL (furnace applies)
in `calculateTrackSpeed` (NewMinecartBehavior, virtual `getMaxSpeed` call
verified in the fresh dump) and `AbstractMinecart.comeOffTrack`
(verified: virtual call at offset 2), but the behavior's OWN getMaxSpeed
(no furnace) in `moveAlongTrack`'s step cap. The port uses `max_speed_new`
(no furnace factor) at ALL three sites: step cap (1069 — correct),
calculateTrackSpeed cap (1208 — wrong) and comeOffTrack (1344 — wrong).
In a `minecart_improvements` world a furnace minecart is capped at 0.4 dry
/ 0.2 wet instead of 0.2 / 0.15. Experiment-gated: 26.1.2
`DEFAULT_FLAGS` = VANILLA_SET only, so the default (old-behavior) path is
correct; the divergence is live only in worlds with the experiment
enabled. The 26.2 port has identical code at all three sites. Reproduces
from 26.1.1 (shared with 26.2 — flag for the 26.2 record).

### B3-G26 — BreedGoal ignores partner panic (minor, port)

Vanilla 26.1.2 checks the partner is NOT PANICKING in both places:
`getFreePartner` skips panicking candidates, and `canContinueToUse` is
`partner.isAlive() && partner.isInLove() && loveTime < 60 &&
!partner.isPanicking()` (verified in the fresh BreedGoal dump:
`isPanicking()` at both sites). The port's partner search
(`goals.rs:2205-2222`, `nearby()` filter) and `can_continue`
(`goals.rs:2227-2231`: `alive && in_love && love_time < 60`) omit the
panic clause entirely. Panic does not clear in-love in either
implementation, so the gap is observable: hit one of two in-love animals
and the other keeps following it in the port; vanilla stops. Shared with
the 26.2 port's identical code (26.2's sweep passed B3 with it) — flag for
the 26.2 record. Reproduces from 26.1.1.

### B3-G27 — Zombie horse has no food (minor, port)

Vanilla 26.1.2 `ZombieHorse.isFood` = `#zombie_horse_food` (red mushroom;
the tag exists in the jar's data — verified in the fresh dump). The port's
`food_tag()` has no `ZombieHorse` arm, so zombie horses cannot be fed for
healing/temper in the port. Shared with the 26.2 port's identical table —
flag for the 26.2 record. Reproduces from 26.1.1.

### B3-G28 — Skeleton horse has no food (minor, port)

Vanilla 26.1.2 `SkeletonHorse` extends `AbstractHorse` and does not
override `isFood` (0 `isFood` occurrences in the fresh dump), so it
inherits `#horse_food`. The port's `food_tag()` covers `Horse | Donkey |
Mule` but not `SkeletonHorse`, so skeleton horses cannot be fed in the
port. Shared with the 26.2 port's identical table — flag for the 26.2
record. Reproduces from 26.1.1.

### B3-G29 — Peaceful gate moved into `checkSpawnRules`; phantom/shulker can no longer spawn in peaceful (minor, port)

Vanilla 26.1.2 `SpawnPlacements.checkSpawnRules` has NO peaceful gate
(verified in the fresh dump: only `Map.get` + `SpawnPredicate.test`); the
peaceful check lives inside the per-type predicates that have it
(`Monster.checkMonsterSpawnRules`, `checkAnyLightMonsterSpawnRules`, the
drowned/guardian/ghast/magma-cube/slime/zombified-piglin rules), and a
type whose predicate lacks it — phantom and shulker use plain
`Mob.checkMobSpawnRules` — answers on its other rules alone. The 26.1.2
port moved the gate up into `checkSpawnRules`
(`mobs/spawn_rules.rs:320-322`: `if !allowed_in_peaceful(s) &&
l.peaceful() { return false; }`) and dropped the per-type gates from
`monster_rules` (297 — light + mob_rules only), blaze/breeze/zoglin (376),
endermite/silverfish (395) and pillager (439). Monsters still fail in
peaceful (same outcome), but phantom and shulker now fail too, where
vanilla lets them spawn. The 26.1 tree had the correct per-type form (with
a test proving the phantom passes in peaceful); the 26.1.1 tree deleted
the test and took the 26.2 form. Reproduces from 26.1.1 (NEW in 26.1.1).

### B3-G30 — Baby-dimensions table carries the 26.2 form; 29 species wrong (minor, port)

The port's baby-dimensions table (`mobs/species.rs` `Def.baby`, 90 entries
parsed fresh) carries the 26.2 form, not 26.1.2's. Vanilla 26.1.2
`BABY_DIMENSIONS` declarers (20 classes, re-extracted FRESH from the
26.1.2 jar with the JVM-free class parser —
`evidence/baby_dims2-26.1.2.py`, output in `evidence/baby-dims-26.1.2.txt`)
vs the port: zombie family + piglin + villager 0.98 vs 0.99 height
(7 species); axolotl wrong absolute dims; chicken eye 0.28125 vs 0.28;
cow/mooshroom absolute dims vs `scale(0.5)` (eye 0.665); fox eye 0.34375
vs 0.2975; squid/glow squid height 0.5 vs 0.63; 11 `Scale`→`Abs`
conversions (camel, cat, goat, hoglin, ocelot, pig, polar bear, sheep,
strider, wolf, zoglin — wrong at least in eye height, several in
width/height too, verified against adult dims/eyes from
`entity-types.json`); 4 wrong eye overrides (armadillo 0.21875 vs scaled
0.156; dolphin 0.09375 vs 0.195; happy ghast 0.46875 vs 0.6175; panda
0.28125 vs 0.53125). The port table is byte-identical to 26.1.1's
(`species.rs` not in the tree diff) and every vanilla declarer class is
byte-identical to 26.1.1's (jardiff), so the 29-species mismatch stands
unchanged. The 26.1 tree's table matched vanilla. Reproduces from 26.1.1
(NEW in 26.1.1).

### B3-G31 — Camel sitting baby dims wrong; body-anchor offset uses the 26.2 double form (minor, port)

Vanilla 26.1.2 `Camel.SITTING_DIMENSIONS = scalable(width, height−1.43)`
with eye 0.845 (verified in the fresh Camel dump: `1.43f`/`0.845f` at the
`putstatic`), and a sitting baby is
`SITTING_DIMENSIONS.scale(getAgeScale)` = (1.02, 0.567, 0.507). The
port's `sitting_dims` (`mobs/camel.rs:52-54`) returns (0.95, 0.425, 0.41)
for a baby. Separately, vanilla `getBodyAnchorAnimationYOffset` computes
`height − 0.375F * scale` in FLOAT (verified: `0.375f` in the fresh dump);
the port's `body_anchor_y` (`mobs/camel.rs:124-125`) uses the 26.2 double
form (0.09375 baby / 0.375 adult offsets). The 26.1 tree matched vanilla
on both. Reproduces from 26.1.1 (NEW in 26.1.1).

### B3-G32 — Heightmap predicates use absent `#blocks_motion_in_heightmap*` tags (minor, port)

Vanilla 26.1.2 has no `blocks_motion_in_heightmap` /
`blocks_motion_in_heightmap_no_leaves` tags (verified absent from jar data
and prepared `block-tags.json`). The 26.1.2 port's heightmap predicates
(`mobs/spawn_selection.rs:419-425`) ground on them, so
`getTopNonCollidingPos` (MOTION_BLOCKING_NO_LEAVES for 80 types) in
`spawnMobsForChunkGeneration` resolves against empty tags. The 26.1 tree
used the legacy-solid rule. `spawn_category_for_chunk` uses WorldSurface
and is unaffected. Reproduces from 26.1.1 (NEW in 26.1.1).

### B3-G33 — Worn heads never reduce detection range (minor, port)

Vanilla 26.1.2 `LivingEntity.getVisibilityPercent` hardcodes five head
rules (skeleton, zombie, piglin, piglin brute, creeper heads each halve
detection range — verified in the fresh dump, offsets 66–171, INCLUDING
the piglin-brute rule at offsets 129–146; `LivingEntity` is byte-identical
between 26.1.1 and 26.1.2, sha1 prefix `1593ddd573b3443a` both) and applies
NO `0..=10` clamp (the method ends in a plain `dreturn` at offset 177).
The 26.1.2 port's `visibility_percent` (`mobs.rs:3185-3211`) reads the
26.2 `mob_visibility` item component (`horizon-items/src/items.rs:1061`
`worn_visibility`); prepared 26.1.2 data has 0 `mob_visibility`
components across all 1,506 items, so worn heads never reduce detection
range in the port. The 26.1 tree hardcoded the five rules. Reproduces
from 26.1.1 (NEW in 26.1.1). 26.1-record corrections applied: piglin
brutes ARE fooled (the 26.1 report's "not fooled" claim is wrong), and
there is no clamp (the 26.1 report's clamp claim is wrong too).

### B3-G34 — Trader hurt gating uses the 10-tick hurt window instead of `hurtMarked` (minor, port)

Vanilla 26.1.2 `TradeWithPlayerGoal` and
`LookAndFollowTradingPlayerSink` gate on `!hurtMarked` — a public field
on `Entity`, cleared at the end of the hurt tick (verified in both fresh
dumps: `getfield hurtMarked` at TradeWithPlayerGoal offset 40 and
LookAndFollowTradingPlayerSink offset 24). The 26.1.2 port removed
`hurt_marked` and gates both on `m.hurt_time <= 0`
(`mobs/trader.rs:198`, `mobs/villager_brain.rs:3461`); `hurt_time` is set
to 10 on a hit (`mobs.rs:3904`) and decremented once per tick
(`ai.rs:1353-1354`), so a trader/villager does not resume trading
behaviour for up to 10 ticks after a hit where vanilla resumes on the
next tick. The `trader.rs` site is a regression (26.1 gated on
`!m.hurt_marked`); the `villager_brain.rs` site is shared with 26.1
(identical code — the brain half the 26.1 sweep missed; see the 26.1.1
summary §"26.1-record corrections"). Reproduces from 26.1.1 (trader half
NEW in 26.1.1; brain half shared with 26.1).

### B3-G35 — BreathAirGoal uses the 26.2 five-column search with swapped start order (minor, port)

Vanilla 26.1.2 `BreathAirGoal.findAirPosition` searches
`BlockPos.betweenClosed(x-1, y, z-1, x+1, y+8, z+1)` — a 3×9×3 box — and
`start()` calls `findAirPosition` THEN `nav.stop()` (verified in the
fresh dump: `findAirPosition` at offset 1, `nav.stop()` at offset 11).
The 26.1.2 port (`mobs/goals.rs:2646+`) uses the 26.2 `neighborColumn`
five-column search with the start order swapped (`start()` at 2695:
`nav_stop()` first, then `find_air`). The 26.1 tree matched vanilla.
Reproduces from 26.1.1 (NEW in 26.1.1).

### B3-G36 — Cat lie-on-bed / sit-on-block goals gated on absent tags (minor, port)

Vanilla 26.1.2 `CatLieOnBedGoal.isValidTarget` checks `BlockTags.BEDS`
(present — verified in the fresh dump); `CatSitOnBlockGoal` checks
unopened chests, lit furnaces, and the foot of beds (CHEST / FURNACE /
BEDS in the fresh dump; no tag gate). The 26.1.2 port gates both on absent
tags: `can_lie_on` (`mobs/tame.rs:1412-1413`) requires `#cats_can_lie_on`
and `CatSitOnBlock` (`mobs/tame.rs:1449`) requires `#cats_can_sit_on` —
both absent from the 26.1.2 jar data and prepared tags — so tame cats
never lie on beds or sit on chests/furnaces/beds. The 26.1 tree matched
vanilla. Reproduces from 26.1.1 (NEW in 26.1.1). (Distinct from G15's
morning-gift goal, which gates on the same absent lie-on tag at the
owner's position.)

### B3-G37 — Inhabited time accrues +1 unconditionally instead of the game-time difference (minor, port)

Vanilla 26.1.2 `ServerChunkCache.tickChunks` computes
`timeDiff = gameTime − lastInhabitedUpdate` and passes it to
`incrementInhabitedTime(timeDiff)` — a chunk reloaded after 1,000 ticks
accrues 1,000 ticks of inhabited time (verified in the fresh dump:
`getGameTime`/`lastInhabitedUpdate`/`incrementInhabitedTime(J)`). The
26.1.2 port (`game/natural_spawning.rs:811`) adds 1 unconditionally, and
the `last_inhabited_update` field was removed from the render world
(`horizon-client-render/src/world.rs`). Local difficulty (which scales
with inhabited time) therefore grows far too slowly for revisited chunks.
The 26.1 tree matched vanilla. Reproduces from 26.1.1 (NEW in 26.1.1).

### B3-G38 — `fresh_dimensions` removed; pufferfish spawn at full size (minor, port)

Vanilla 26.1.2 `Pufferfish.getScale(0) = 0.5` (verified in the fresh
dump): a freshly spawned pufferfish is half-size until it inflates. The
26.1 tree called `fresh_dimensions` at spawn
(`game/natural_spawning.rs:493`); the 26.1.2 tree has no
`fresh_dimensions` reference in `game/natural_spawning.rs` (only
unrelated `refresh_dimensions` elsewhere), so pufferfish spawn at full
size. Reproduces from 26.1.1 (NEW in 26.1.1).

### B3-G39 — Spawn-egg entity data applied to spawners; vanilla drops it (minor, port)

Vanilla 26.1.2 `SpawnEggItem.useOn` calls only
`Spawner.setEntityId(type, random)` — type only, no entity data — for
both spawner types (verified in the fresh dump: the single
`setEntityId` interface call at offset 139), and
`TrialSpawnerBlockEntity` has only `setEntityId` (no `setEntityData`
method exists in the jar — verified in the fresh dump). The 26.1.2 port
applies egg data to both: the trial arm (`mobs/trial_spawner.rs`
`override_entity_data` + caller `game/spawner.rs:450`), and the regular
arm (`game/spawner.rs:461-462` `set_entity_data`) — the regular arm is
shared with 26.1 (26.1 B7-G17 is exactly the regular spawner arm). An op's
spawn egg with entity data (e.g. a named/customised mob) sets the spawner
to spawn that customised mob in the port; vanilla spawns a default one.
Reproduces from 26.1.1 (trial arm NEW in 26.1.1; regular arm shared with
26.1).

### B3-G40 — Armadillo ball-up refuses in liquid (minor, port)

Vanilla 26.1.2 `ArmadilloAi$ArmadilloBallUp.checkExtraStartConditions`
is `onGround()` only — no liquid check (verified in the fresh dump). The
26.1.2 port (`mobs/armadillo.rs:379`, `liquid` at 390) adds `!liquid(m)`,
so an armadillo standing in water will not ball up in the port where
vanilla does. The 26.1 tree matched vanilla. Reproduces from 26.1.1 (NEW
in 26.1.1).

### B3-G41 — Slime/magma-cube `dealDamage` adds a team check vanilla lacks (minor, port)

Vanilla 26.1.2 `Slime.dealDamage` checks alive, `isWithinMeleeAttackRange`,
`hasLineOfSight`, then `mobAttack` — NO team check (verified in the fresh
dump). The 26.1.2 port (`mobs/cube.rs:242, 297`) adds `teams_allow_damage`
checks, so a slime or magma cube on a team with `friendly_fire` off cannot
damage a teammate in the port where vanilla damages regardless. The 26.1
tree matched vanilla. Reproduces from 26.1.1 (NEW in 26.1.1).

### B3-G42 — Piglin offhand items made persistent with a 2.0 drop chance (minor, port)

Vanilla 26.1.2 `Piglin.holdInOffHand`: a barter item goes to
`setItemSlot` + `setGuaranteedDrop` (2.0); anything else goes to
`setItemSlotAndDropWhenKilled` (1.0) — NO persistence is set either way
(verified in the fresh dump). The 26.1.2 port (`mobs/piglin.rs:31-33`)
makes non-barter offhand items persistent AND gives them drop chance 2.0
always, so a piglin that picks up a non-barter item in its off hand never
despawns and always drops the item in the port. The 26.1 tree matched
vanilla. Reproduces from 26.1.1 (NEW in 26.1.1).

### B3-G43 — Fox chorus-fruit teleport emits a level event 2017 trail vanilla lacks (minor, port)

Vanilla 26.1.2 `TeleportRandomlyConsumeEffect` has ONLY a `diameter`
field — no `directional_particles` in its codec (verified in the fresh
dump) — and `apply` emits the TELEPORT game event + sound, no level
event. The 26.1.2 port parses `directional_particles` with
`.unwrap_or(true)` (`horizon-items/src/items.rs:129`); the prepared
chorus-fruit data has no such field (`{"type":
"minecraft:teleport_randomly"}` — verified fresh in prepared
`items.json`), so the flag defaults true and the fox path
(`mobs/fox.rs:387-393`) emits the 2017 directional-particle trail on every
chorus-fruit teleport. The 26.1 tree had no 2017 event. Reproduces from
26.1.1 (NEW in 26.1.1). (The same `TeleportConsume` struct feeds the
player's chorus-fruit path (`teleport_consume.rs:109-116`) — Batch 7's
area; flagged in `out_of_scope_failures`.)

### B3-G44 — Loaded sleeping villager position never refreshed (minor, port)

Vanilla 26.1.2 `LivingEntity.baseTick` calls `setPosToBed` (a flat 0.6875
above the head block's base — verified in the fresh dump: `0.6875d` at
offset 15) for a sleeping entity every tick. The 26.1.2 port's
`sleeping_position` (`mobs/world.rs:141-145`) gates on
`facts.has_class("AbstractBedBlock")` — a class with 0 entries in the
26.1.2 jar (only plain `BedBlock` exists — verified by jar listing) — so
it returns `None` always, and the caller's `refresh_sleeping_position`
(`mobs/villager.rs`, called from `ai.rs:1212`) never updates a loaded
sleeping villager's position. The 26.1 tree matched vanilla. Reproduces
from 26.1.1 (NEW in 26.1.1).

### B3-G45 — Spear stab knockback applied as two pushes instead of one summed push (minor, port)

Vanilla 26.1.2's kinetic-weapon stab (`LivingEntity`, the
`recentKineticEnemies` path) calls `causeExtraKnockback` ONCE with the
summed strength `0.4f + getKnockback(target, source)`. The 26.1.2 port
(`mobs/spear_use.rs:272, 275`) calls it TWICE — 0.4, then the enchantment
halved — which pushes the target twice (halving its velocity between
pushes) and slows the attacker twice. The 26.1 tree pushed once with the
summed strength. Reproduces from 26.1.1 (NEW in 26.1.1).

### B3-G46 — Item-entity landing bounce uses the 26.2 restitution form; bed bounce gone (minor, port)

Vanilla 26.1.2 has NO `restituteMovementAfterCollisions` and NO
`bounceRestitution` (0 occurrences in the fresh `Entity` dump);
`SlimeBlock` and `BedBlock` have `bounceUp`, and the 26.1 form bounces any
falling speed (slime: `−vy × 0.8d`; bed: `−vy × 0.66f × 0.8d`). The
26.1.2 port's `vertical_collision_velocity` (`entities.rs:36-50`) uses
the 26.2 restitution form — its own docstring cites
`Entity.restituteMovementAfterCollisions`, which does not exist in
26.1.2: a gravity threshold (slow falls don't bounce), gravity/drag
compensation, and NO bed branch at all (parameters are `below`/`slime`
only) — items landing on beds don't bounce. The 26.1 tree matched
vanilla. Reproduces from 26.1.1 (NEW in 26.1.1). (Minor facet, same area:
the item-entity air drag changed from the double literal 0.98 to 0.98F
widened — ~2e-8/tick, unobservable; recorded, not a separate item.)

### B3-G47 — AI walk-target value misses the Zero arms and the Giant/Mooshroom arms (minor, port) — NEW in 26.1.2

Vanilla 26.1.2 `getWalkTargetValue` (all verified in fresh dumps):
Enderman, Pillager and Warden override it to return `0.0f`; Giant returns
`+getPathfindingCostFromLightLevels` (UNNEGATED — giants prefer bright
places, opposite of monsters); MushroomCow returns 10.0 over mycelium,
else `+getPathfindingCostFromLightLevels`. The port's AI-path
`walk_target_value` (`mobs/ai.rs:2669-2742`, used by RandomStrollGoal's
`pick_best` and `hide_pos`) has NONE of these arms: Enderman, Pillager and
Warden fall into the hostile branch and return `-light()` (they actively
prefer dark stroll destinations where vanilla is indifferent); Giant also
gets `-light()` (sign-flipped vs vanilla); Mooshroom gets the generic
animal arm (10.0 over GRASS_BLOCK, not mycelium). The port's SPAWN table
(`spawn_selection.rs:145`, `walk_target`) has all five arms correct
(Enderman/Creaking/Pillager/Warden → Zero, Giant → +cost, Mooshroom →
mycelium) — only the AI path is wrong. Live for Enderman (Stroll goal,
`enderman.rs:28`), Pillager (Stroll, `illager.rs:65`) and Mooshroom
(Stroll, `farm.rs:41`); latent for Warden (brain-based, no Stroll goal)
and Giant (no goals at all). Shared with the 26.1 tree's identical code —
missed by the 26.1 and 26.1.1 sweeps, which checked only the spawn table
(the 26.1.1 report's G12 "port matches" verdict covers the spawn table
only). NEW in 26.1.2.

## Re-checked, not defects (26.1.1 items: none — all 45 reproduce; 1.21.7 items: 8)

**26.1.1 items:** every one of the 45 (37 G + 8 O-groups) reproduces in
26.1.2 with fresh evidence — no 26.1.1 item was fixed, withdrawn, or left
unverified. The full re-check verdict table is in §"Re-check verdicts"
below.

**1.21.7 items that do NOT carry to 26.1.2** (verified fresh against the
26.1.2 jar and port):

- **G01 — bat spawning**: vanilla 26.1.2 `Bat.checkBatSpawnRules` has NO
  Halloween branch (verified in the fresh dump — no HALLOWEEN / LocalDate /
  October references anywhere); the port's `bat_rules`
  (`mobs/spawn_rules.rs:367`) matches: below surface, `nextBoolean`,
  brightness > `nextInt(4)`, `#bats_spawnable_on` (present in 26.1.2 data),
  plus mob rules. NOT a defect (the 1.21.7 report's "Halloween branch"
  claim was for 1.21.7's form; 26.1.2's form matches the port).
- **G02 — slime spawning**: the port's `slime_rules`
  (`mobs/spawn_rules.rs:509-539`) matches vanilla: environment-attribute
  `nextFloat` single draw, slime-chunk seed 987234911, `nextInt(10)==0`,
  y < 40, its own peaceful gate. NOT a defect.
- **G08 — drowned burn**: `#burn_in_daylight` in the 26.1.2 jar holds
  exactly 10 types including DROWNED (verified fresh), and the port's list
  (`mobs/species.rs:1543-1555`) matches it exactly. NOT a defect.
- **G10 — `canSpawnFarFromPlayer`**: exactly 2 explicit setters in
  `EntityType.<clinit>` — pillager and shulker (shulker confirmed at the
  putstatic after offset 5342); the port (`mobs/spawn_selection.rs:317-320`)
  is Creature | Misc + Pillager | Shulker. NOT a defect.
- **G12 — enderman walk-target, as framed**: the 1.21.7 claim was about
  the SPAWN table (port maps Enderman to Zero; vanilla 1.21.7 inherits
  Monster's −cost). In 26.1.2, vanilla `EnderMan.getWalkTargetValue` IS
  0.0f, so the port's spawn-table Zero is CORRECT. NOT a defect as framed
  — but the port's AI-path `walk_target_value` (`mobs/ai.rs:2669`) misses
  the Zero arm entirely and gives endermen `-light()`; that IS a defect,
  minted as new B3-G47.
- **G16 — villager trades**: the 26.1.2 jar has `villager_trade` (551
  files: 460 base + 91 in the trade_rebalance datapack) and `trade_set`
  (68) data — counts identical in the 26.1.1 jar — and the port's
  `Trades::load` (`merchant.rs:482-487`) reads all three dirs. NOT a
  defect.
- **G17 — `reduced()`**: vanilla 26.1.2 `Goal` HAS `reducedTickDelay`
  (`positiveCeilDiv(t,2)`) and `adjustedTickDelay`; the port's `reduced`
  (`goals.rs:66-71`) matches. NOT a defect.
- **G21 — zombie spear**: vanilla `Zombie.populateDefaultEquipmentSlots`
  is 0.05 / 0.01 with `nextInt(6)`: 0→IRON_SWORD, 1→IRON_SPEAR, else
  IRON_SHOVEL (verified in the fresh dump); the port matches at
  `spawning.rs:222` and `finalize.rs:1347`. NOT a defect.
- **G07**: withdrawn in 1.21.7 — not re-minted.

**1.21.7 oracle groups O01–O06:** none mint in 26.1.2. The gamerules
package exists (`GameRules.class` present in the 26.1.2 jar); the 26.1.1
Batch 0 table shows no `world.clock` failures; no entity-subpackage
first-errors; only the expected ZombieCureSound server-only
`client.multiplayer` failure (an artifact of the server-only classpath,
not breakage).

**26.1-record corrections applied** (from the 26.1.1 summary, verified
fresh against the 26.1.2 jar): (1) mob equipment-pickup persistence —
vanilla `Mob.setItemSlotAndDropWhenKilled` DOES set
`persistenceRequired`, and the port's `game/mob_looting.rs:124`
(`m.persistent = true`) is CORRECT, so no item was minted for it;
(2) spawn-egg peaceful gate — vanilla `SpawnEggItem.spawnMob` DOES gate
on `!isAllowedInPeaceful() && PEACEFUL` (verified fresh in the dump), and
the port's `spawn_egg_allowed` is correct in structure — only the hoglin
facet of G04 is wrong; (3) B3-G33 piglin-brute visibility + no clamp (see
Defects); (4) B3-G34 brain half (see Defects). B3-G39 is written without
the 26.1.1 report's "missed by the 26.1 sweep" claim for the regular
spawner arm — it is wrong: 26.1 B7-G17 is exactly that arm.

**Recorded, not minted** (all verified fresh or byte-identical to the
26.1.1 verdicts): dead SulfurCube arms in `species.rs` (no vanilla
counterpart); `checkDespawn` order difference (port checks pickup after
despawn; vanilla before — despawning mobs with items differ by one tick
in the despawn case only); the `listed` optimisation in `checkMonster-
SpawnRules` (a `ListedMobs` entry with no matching `MonsterSpawnRules`
entry short-circuits the light predicate — same outcome, different
predicate evaluation); trial spawner `overridePeacefulAndMobSpawnRule`
omission (only the spawner block's own `getRandomSpawnMobAt` path; the
trial spawner's `spawnsInPeaceful` handling matches); Hero-of-the-Village
f32 rounding (port uses f32, vanilla f64 — ~1e-7 price differences);
boat `lastYd` (port stores the pre-drag value; vanilla the post-drag —
affects only the next tick's `getDeltaMovement` read); hurting-projectile
normalize floor (port clamps the direction vector's length at 1e-4;
vanilla divides unclamped — a zero-length vector is unreachable in
practice); MeleeAttack spectator-target note (vanilla targets spectators
in `canUse` when the mob is already targeting one; the port filters
spectators — unreachable in practice since targeting never selects a
spectator); ridden-mount early return in `tick` (port returns early for
ridden mounts before the AI tick; vanilla runs the AI tick — the mount's
goals are no-ops while ridden, so no observable difference); finalize RNG
draw-order (port draws equipment slots in a different order than vanilla's
`populateDefaultEquipmentSlots` — same distribution, different sequence);
minecart drag 0.95f32 (port uses f32 0.95, vanilla f64 0.95 — ~1e-8/tick);
`handleAttack` invalid-entity disconnect / piercing-weapon / feature-flag
arms (see B8 — inherited 26.1.1 form, reachable only with a modified
client; vanilla's client never sends such packets).

## Re-check verdicts (26.1.1 items — all 45)

| Item | Verdict | Item | Verdict |
|------|---------|------|---------|
| B3-G03 | REPRODUCES | B3-G29 | REPRODUCES |
| B3-G04 | REPRODUCES | B3-G30 | REPRODUCES |
| B3-G05 | REPRODUCES | B3-G31 | REPRODUCES |
| B3-G06 | REPRODUCES | B3-G32 | REPRODUCES |
| B3-G09 | REPRODUCES | B3-G33 | REPRODUCES |
| B3-G11 | REPRODUCES | B3-G34 | REPRODUCES |
| B3-G13 | REPRODUCES | B3-G35 | REPRODUCES |
| B3-G14 | REPRODUCES (major) | B3-G36 | REPRODUCES |
| B3-G15 | REPRODUCES | B3-G37 | REPRODUCES |
| B3-G18 | REPRODUCES | B3-G38 | REPRODUCES |
| B3-G19 | REPRODUCES | B3-G39 | REPRODUCES |
| B3-G20 | REPRODUCES | B3-G40 | REPRODUCES |
| B3-G22 | REPRODUCES (major) | B3-G41 | REPRODUCES |
| B3-G23 | REPRODUCES | B3-G42 | REPRODUCES |
| B3-G24 | REPRODUCES | B3-G43 | REPRODUCES |
| B3-G25 | REPRODUCES | B3-G44 | REPRODUCES |
| B3-G26 | REPRODUCES | B3-G45 | REPRODUCES |
| B3-G27 | REPRODUCES | B3-G46 | REPRODUCES |
| B3-G28 | REPRODUCES | B3-O01–O08 | REPRODUCE (8 groups) |

No FIXED, NEVER EXISTED, WITHDRAWN or UNVERIFIED among the 26.1.1 items.
The 26.1.2 delta touched no Batch 3 file and no entity class, so every
26.1.1 verdict stands — each was still re-verified fresh against the
26.1.2 jar and port (see Defects for the fresh evidence per item).

## Passes

### A1 — Entity registry / spawn placements (1 check, 1 pass)

82 vanilla `EntityType.register` calls in `EntityType.<clinit>` (verified
in the fresh dump); prepared `mob-data.json` placements match (0
mismatches across 157 placements: no_restrictions 85, on_ground 59,
in_water 12, in_lava 1; heightmaps MOTION_BLOCKING_NO_LEAVES 155,
MOTION_BLOCKING 2). PASS.

### A1b — Spawn rules (11 checks, 10 pass, 1 gap)

Per-type spawn rules match vanilla for bat (G01 not-a-defect), slime (G02
not-a-defect), drowned/guardian/ghast/magma-cube/zombified-piglin light
and height rules, endermite/silverfish, blaze/breeze/zoglin, pillager
(light ≤ 8), cat/ocelot, wolf/fox, parrot, and the `isValidPositionForMob`
block table. GAP: G29 (peaceful gate moved). The per-type gates in
`monster_rules` (297), blaze/breeze/zoglin (376), endermite/silverfish
(395) and pillager (439) all omit the peaceful check that vanilla's
per-type predicates have — but the outcome is identical for monsters
(the gate in `checkSpawnRules` covers them); only phantom/shulker diverge.

### A2 — Species coverage (1 check, 1 pass)

90 `Species` variants (including the dead `SulfurCube`, which has no
vanilla counterpart — recorded, not minted) = 90 `def!` entries in
`mobs/species.rs`. PASS.

### A3 — Attributes / food / dimensions (11 checks, 6 pass, 5 gap)

Attributes registry 35/35; 2,131 entity attribute values compared against
`entity-types.json` with 0 mismatches (hex f64 bit patterns decoded);
suppliers 92; categories exact; fire-immune 15. PASS except: G04
(peaceful table), G05/G27/G28 (food table — one check, three items),
G06 (head rotation), G30 (baby dims), G31 (camel sitting dims).

### A4 — Sun-burn / protection slots (5 checks, 4 pass, 1 gap)

`#burn_in_daylight` exact (10 types incl. drowned — G08 not-a-defect);
`sunProtectionSlot` Mob HEAD, ZombieHorse/ZombieNautilus BODY (port
`drops.rs:186` `body_slot_species` → `species::body_slot` covers
ZombieNautilus + `equine::is_equine` covers ZombieHorse). GAP: G09
(powder-snow exclusion missing).

### B1 — Suffocation / fluid (3 checks, 1 pass, 2 gap)

15 `isSuffocating` calls in `Blocks.txt` (matches the 26.1.1 resolution);
port `suffocation.rs` byte-identical to 26.1.1. PASS except G22 (default
tag-gated, startup blocker) and G23 (sleeping mobs suffocate).

### B2 — Spawning (27 checks, 21 pass, 6 gap)

Spot-verified fresh: `magic_light` exact (`spawn_selection.rs:429-436`,
the curved `f/(4−3f)` form with ambient); `checkDespawn` structure
matches (`ai.rs:2295+`); `server_ai_step` alternation matches
(`ai.rs:2039+`, `no_action_time` first, `tick_count + id`); `MeleeAttack-
Goal` 20-tick gate + `createPath` confirmed in the fresh dump. Carried on
the jardiff + tree-diff basis (vanilla classes byte-identical to 26.1.1,
port files byte-identical): mob categories, `getRandomSpawnMobAt`,
`mobsAt`/fortress, `right_distance`, `valid_position_for_type`,
`isValidPositionForMob`, obstruction table, `max_cluster_size`,
`isMaxGroupSizeReached`, `removeWhenFarAway`, `isValidSpawn` block table,
`isSpawnPositionOk`, `getSpawnAABB` / `spawnDimensionsScale` / `canSummon` /
`EntitySpawnReason`, `SpawnState`/`createState`, `PotentialCalculator`,
`getFilteredSpawningCategories`/local cap, spawn-rules tables. GAPs:
G03 (immune tags), G11 (y-range), G13 (RNG draw), G32 (heightmap tags),
G37 (inhabited time), G38 (fresh dimensions).

### B7 — Spawners (4 checks, 3 pass, 1 gap)

Spawner defaults exact (`block_entity.rs:605-620`: delay 20, min 200,
max 800, spawn count 4, range 6, activation 16, max nearby 4);
`delay()` matches (`mobs/spawner.rs:447-448`, max≤min→min); trial config
defaults exact (`block_entity.rs:636-655`: spawn_range 4, total 6.0,
simultaneous 2.0, +2.0/+1.0 per player, 40 ticks). GAP: G39 (spawn-egg
entity data applied). (The 26.1.1 report's "spawner.rs:317" citation
refers to `mobs/spawner.rs`, not `game/spawner.rs`.)

### B8 — Combat / LAN (4 checks, 2 pass, 2 gap)

`CombatRules` methods confirmed in the fresh dump; port
`combat_rules.rs` byte-identical to 26.1.1. GAPs: G41 (slime team check),
G45 (spear stab double push). **NEW PASS — `handleAttack` spectator
early-return (the 26.1.2 delta's one behaviour item).** Vanilla
26.1.1→26.1.2 `handleAttack` delta (normalized diff of both fresh javap
dumps, `evidence/javap/handleAttack-26.1.1.txt` vs
`handleAttack-26.1.2.txt`): the ONLY change is the early return —
26.1.1 `if (!hasClientLoaded()) return;` → 26.1.2
`if (!hasClientLoaded() || player.isSpectator()) return;` before
`resetLastActionTime`; the rest of the body is byte-identical modulo
offsets. The 26.1.2 tree DOES port it: `lan/hub.rs:4063-4065` (spectator
attack never resets the idle timer:
`S::Attack(_) => !self.guests[i].spectator()`) and `:4083-4087` (spectator
attacks dropped before the game/anticheat/move queue), with test
`a_spectators_attack_reaches_nothing_and_resets_no_timer` in
`lan/hub_tests.rs:927`; `guest_attack` (`game/lan_actions.rs:3512`, sole
caller path via hub dispatch at `:2659`) keeps its own spectator guard as
the second line. The local player in single-player is a hub guest (the
window joins its server thread as a client), so the same dispatch covers
it. **Consequence: `PORT-NOTES-26.1.2.md`'s claim "the changed method body
remains unported" is STALE/WRONG for the spectator early-return** — the
port-notes should be corrected. (The Checkbox tooltip item, #6, is
out of Batch 3 scope — UI/dialog.) Not minted: `guest_attack` lacks
vanilla's invalid-entity disconnect (item entity / XP orb / self /
non-attackable arrow → disconnect + warn) and the piercing-weapon +
feature-flag early returns — inherited 26.1.1 form, reachable only with
a modified client (vanilla's client never sends such packets).

### B5 — Projectiles (4 checks, 3 pass, 1 gap)

`AbstractHurtingProjectile` INITAL_ACCELERATION_POWER, getInertia 0.95,
getLiquidInertia 0.8; `ThrownTrident.getWaterInertia` 0.99 — all confirmed
fresh in the dumps. GAP: G46 (item-entity bounce restitution form).

### B4 — Trading (6 checks, 3 pass, 3 gap)

`LEVEL_XP` [0,10,70,150,250] (`merchant.rs:16`); `cost_a_count` exact
(`:164-171`, int product, floor, max(0), clamp); `update_demand` exact
(`:185-187`). GAPs: G24 (level-up immediate), G34 (hurt gating), G42
(piglin offhand).

### B6 — Vehicles (3 checks, 2 pass, 1 gap)

Boat gravity 0.04 in `tick_boat` (`boats.rs:822, 840`); `boats.rs`
byte-identical to 26.1.1. GAP: G25 (furnace minecart speed, experiment-
gated).

### B3 — Mob AI (12 checks, 3 pass, 9 gap)

PASS: `reduced()` (G17 not-a-defect — `Goal.reducedTickDelay` /
`adjustedTickDelay` match `goals.rs:66-71`); the spawn-path walk-target
table (`spawn_selection.rs:145` — Enderman/Creaking/Pillager/Warden →
Zero, Giant → +cost, Mooshroom → mycelium, all correct); silverfish
walk-target (10.0 on infested host else −cost — the AI path matches
vanilla's `Silverfish.getWalkTargetValue`). GAPs: G14 (enderman teleport),
G15 (cat morning gift), G26 (BreedGoal panic), G33 (worn heads), G35
(BreathAirGoal), G36 (cat goals), G40 (armadillo ball-up), G44 (sleeping
position), G47 (AI walk-target value — NEW).

### B9 — Finalize / equipment (3 checks, 3 pass)

Piglin finalize `float() < 0.2` at `finalize.rs:1704`; drowned equipment
verified fresh (vanilla: `nextFloat > 0.9` double, `nextInt(16)`, <10 →
TRIDENT else FISHING_ROD; port `spawning.rs:228` matches); zombie
equipment (G21 not-a-defect). PASS.

## Oracle defects

**Basis (provisional):** the 26.1.2 Batch 0 table
(`docs/sweep/26.1.2/batch-0-oracles.json`) had NOT landed when this batch
finished (checked 2026-11-25; `docs/sweep/26.1.2/` holds only the batch
1, 2 and 12 reports, `extract-inventory.py`, `inventory-26.1.2.json` and
`inventory.md`). Oracle triage therefore uses the 26.1.1 Batch 0 table
(`sweep-26.1.2-ref/26.1.1/batch-0-oracles.json`, 391 oracles, generated
2026-11-24 by fixpoint big-compile per classpath against the named
26.1.1 jars), per the brief. The basis is sound and was verified fresh:
the oracle sources are byte-identical between the trees (`mods/` diff
empty; 391 sources in both), and the vanilla API surface they compile
against is unchanged (only 3 classes changed, all body-only — no
signature changes), so compile results are identical. Every anchor below
was additionally re-verified fresh against the 26.1.2 jars. The
coordinator should re-verify these 8 groups against the 26.1.2 table when
it lands; no re-verification is expected to change any verdict.

- **B3-O01 — missing-override (3 oracles, n=1 each).** `Level.fuelValues()`
  is `public abstract` in 26.1.2 (fresh dump `Level.txt:1929`, same line
  as 26.1.1); the oracle level stubs lack the override (grep-confirmed
  absent in the 26.1.2 tree sources). Reproduces from 26.1.1.
- **B3-O02 — constructor-mismatch (5 oracles).** `LocalPlayer` ctor takes
  8 args in 26.1.2 (fresh dump from the client jar); `MinecraftServer`
  ctor takes 10 args (fresh dump). Oracle sources construct with the 26.3
  form (e.g. `VanillaAllayInteractionStateOracle` has `Actor extends
  LocalPlayer`). Reproduces from 26.1.1.
- **B3-O03 — no-override (14 oracles).** 26.3 methods absent in 26.1.2;
  n_errors per the 26.1.1 table: Arrow 1, BlockingItemDamage 2, BoatRider
  4, ClientArrow 1, DamageImmunity 2, EntityDeathLoot 4, FinalizeSpawn 7,
  LateEntityPair 4, MobAttributes 1, NaturalSpawn 16, PlayerTravel 1,
  RegisteredPlayerCombat 6, ServerPlayerCombat 6, SpawnRules 1.
  Reproduces from 26.1.1.
- **B3-O04 — `EntityTypes` holder move (9 oracles, incl. transcripts
  EntityTranscript n=7, RideTranscript n=4).** All 9 sources reference
  `EntityTypes` (grep-verified in the 26.1.2 tree); `EntityTypes` has 0
  entries in either 26.1.2 jar (26.1.2 predates the holder move).
  Reproduces from 26.1.1.
- **B3-O05 — missing-symbol (16 oracles).** Anchors verified fresh:
  `BlockTags` has 248 fields, none of CAUSES_SUFFOCATION /
  SUPPRESSES_BOUNCE / BLOCKS_FLUID_FLOW; `ClientboundUpdateAdvancements-
  Packet` is a plain class (no Record attribute) with private `added` /
  `removed` / `progress` fields and only `getAdded()` / `getRemoved()` /
  `getProgress()` accessors (no record-style accessors) —
  LocatorBarClient n=13 reproduces; `Enderman` (26.3 name) absent,
  `EnderMan` present; `densityfunction` package 0;
  `ServerboundPunchPacket` 0; `EntitySpawnRequest` / `NoiseSpawnFinder` /
  `MoveSimulationType` 0. Reproduces from 26.1.1.
- **B3-O06 — missing-package (2 oracles).** Advancements `triggers` /
  `predicates` packages absent (only `criterion`, 168 entries — verified
  fresh). Reproduces from 26.1.1.
- **B3-O07 — cascade (2 oracles).** Depend on broken
  VanillaFaceSupportOracle / VanillaBlockUpdateOracle. Reproduces from
  26.1.1.
- **B3-O08 — `VanillaEntityFluidOracle` (1 oracle, n=1).** "'void' type
  not allowed here" at line 74. Reproduces from 26.1.1.

**Passing Batch 3-scope oracles** (23 pass all three classpaths in the
26.1.1 table; `VanillaZombieCureSoundOracle` passes client+both and fails
server-only on `client.multiplayer` — expected, not breakage):
VanillaAttributeHealthOracle, VanillaEffectNotificationsOracle,
VanillaEffectSyncOracle, VanillaFluidBoxOracle, VanillaLivingFallOracle,
VanillaTradeComponentsOracle, VanillaVillagerFarmingOracle,
VanillaWorldSpawnPacketOracle, VanillaCombatRulesOracle,
VanillaEntityAttributeOracle, VanillaEntityObstructionOracle,
VanillaDamageSourceOracle, VanillaFallDamageOracle, VanillaSwimmingOracle,
VanillaLivingDamageOracle, VanillaLivingHealthOracle, VanillaMobAirOracle,
VanillaMobDiscardOracle, VanillaRaiderCelebrationOracle,
VanillaServerPlayerDamageOracle, VanillaPlayerEquipmentOracle,
VanillaPlayerFoodOracle, VanillaPlayerHealthOracle.

## Drivers (§D)

144 drivers were curated to 135 core (see common.md). Batch 3's driver-
code defects are G18, G19, G20 (3 checks, 0 pass, 3 gap — counted
as-tallied per the 26.2 precedent, split reported in the Tally). No driver
was run (checks + triage only). Tree-wide driver-code defect counts
(re-counted fresh, listed in the slice's `out_of_scope_failures`):
36 drivers hardcode the 26.3 state count 35723 (2 in Batch 3 scope, 34
others); 35 drivers use a 26.3 resources path (5 in Batch 3 scope, 30
others — the broad pattern, all `'26.3'` path forms; the narrow
`horizon-client/26.3` pattern alone gives 26). Full list in
`evidence/drivers-263-paths.txt`.

## Method notes

- **Vanilla-check rule:** every vanilla anchor was verified against
  26.1.2's own jars (server-game for entity logic, client for data/tags/
  advancements), never the 26.3 base's form. The server-game jar's
  bundler wrapper (`server.jar`) was never javap'd.
- **Jardiff basis:** 26.1.1→26.1.2 server-game jardiff = 3 changed classes
  (all body-only, no signature changes); client jar = 8 changed classes.
  No entity class changed. Where a 26.1.1 verdict rests on an unchanged
  vanilla class and a byte-identical port file, the verdict carries —
  stated as such per check; every defect item was still re-verified fresh
  (fresh javap dump and/or fresh port-file read).
- **Tree-diff basis:** 32 differing files between `versions/26.1.1` and
  `versions/26.1.2`, none in Batch 3 scope. Out-of-scope adjacent changes
  flagged for other batches (NOT defects here): sign-text synthetic kinds
  removed (`item_sync.rs`, `item.rs` — Batch 7/8 area); legacy palette
  NBT `Name` form removed (`chunk.rs` — Batch 6 area); save DataVersion
  thresholds 4295→4997 (`save.rs`); `lan/tracking.rs` thrown-type→snowball
  tracking fallback removed (only affects unknown/content-mod kinds, NOT
  a vanilla parity issue); `net.rs` formatting, `play.rs` comment,
  `lan_particles.rs` comment, `version.rs`, `ids.rs`, datafix files,
  docs/lock/status files.
- **PORT-NOTES flag:** `PORT-NOTES-26.1.2.md`'s "the changed method body
  remains unported" (re: `handleAttack`) is stale — the spectator
  early-return IS ported in `lan/hub.rs` (see B8). The port-notes should
  be corrected. (The Checkbox tooltip item is out of Batch 3 scope.)
- **Oracle triage:** provisional from the 26.1.1 Batch 0 table (identical
  oracle sources + unchanged vanilla API surface; all anchors re-verified
  fresh). Re-verify against `docs/sweep/26.1.2/batch-0-oracles.json` when
  it lands.
- **Evidence:** `b03/evidence/` — `run-javap.sh`, `run-javap-fixup.sh`,
  `javap/` (114 dumps incl. `handleAttack-26.1.1.txt` /
  `handleAttack-26.1.2.txt`), `baby_dims2-26.1.2.py` +
  `baby-dims-26.1.2.txt`, `drivers-263-paths.txt`,
  `jardiff-26.1.1-26.1.2.txt`, `tree-diff-26.1.1-26.1.2.txt`,
  `server-game-listing.txt`, `client-listing.txt`. Correct javap
  subpackage paths (learned the hard way — several first attempts failed
  with 0 lines on wrong paths): `SpawnerData` =
  `world.level.biome.MobSpawnSettings$SpawnerData`; `Camel`/`CamelHusk` =
  `animal/camel/`; `ZombieHorse`/`SkeletonHorse` = `animal/equine/`;
  `Armadillo`/`ArmadilloAi` = `animal/armadillo/`; `Cat` =
  `animal/feline/`; `Villager` = `npc/villager/`; minecarts =
  `vehicle/minecart/`; `Pufferfish` = `animal/fish/`; `Zombie`/`Drowned` =
  `monster/zombie/`; `AbstractHurtingProjectile` =
  `projectile/hurtingprojectile/`; `ThrownTrident` = `projectile/arrow/`;
  `AbstractBoat` = `vehicle/boat/`; `SpawnPlacementTypes` =
  `world.entity` (interface); Mooshroom is `MushroomCow` (`animal/cow/`).
- **Not verified live:** no driver was run, no oracle was compiled, no
  game was launched — all verdicts are static (javap + source + data).
  The 26.1.1 run's live evidence (where it existed) was not re-run; the
  jardiff + tree-diff basis makes re-running it unnecessary for these
  verdicts.

## Verdict

**98 checks as-tallied: 63 pass, 35 gap** (95 behaviour: 63 pass, 32 gap;
3 driver-code: 0 pass, 3 gap). **46 items: 38 G + 8 O-groups; 2 major,
44 minor.** All 45 of 26.1.1's Batch 3 items reproduce — the 26.1.2 delta
touched no Batch 3 file and no entity class, so the 26.1.1 verdicts stand
unchanged, each re-verified fresh. One new defect (B3-G47, AI walk-target
value — missed by the 26.1 and 26.1.1 sweeps) and one new passing check
(B8 `handleAttack` spectator early-return — the delta's one behaviour
item, ported in the LAN hub, contradicting the port-notes' "remains
unported" flag). The two re-check-first items both reproduce: **B3-G14
endermen never teleport (major)** and **B3-G22 the `#causes_suffocation`
startup blocker (major)** — the 26.1.2 client cannot load its core tables
with prepared 26.1.2 data. Oracle triage is provisional from the 26.1.1
Batch 0 table (the 26.1.2 table had not landed); re-verify when it does.
