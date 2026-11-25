# Batch 3 — Entities / mobs (1.21.11 parity sweep)

**Version:** 1.21.11 · **Date:** 2026-11-25 · **Branch:** `parity-sweep-1.21.11`
(workdir `~/workspace/mcir-work/sweep-1.21.11`) · **Scope:**
`docs/PARITY-SWEEP.md` §4 Batch 3 — mob AI, spawning, trading, attributes,
projectiles, vehicles; 157 entity types (72 MISC, 31 CREATURE, 44 MONSTER,
1 AXOLOTLS, 1 AMBIENT, 4 WATER_AMBIENT, 3 WATER_CREATURE, 1 UNDERGROUND
WATER_CREATURE); 144 drivers curated to 135 core (see §D).

**Status: checks + triage only, per the brief: no fixes, no edits under
`versions/1.21.11/`, no cargo builds, no driver runs, no oracle repairs, no
commits — the coordinator commits these deliverables.** Everything below is
static analysis: `javap -c` on the named 1.21.11 server-game jar
(`~/workspace/mcir-work/e6-deltas/versions/1.21.11/jars/server-game-named.jar`,
sha1 `7f0d85c12aeb57f14687b7ab2ae2e5d825fbf0d2`, 7,163 classes; client-named
jar sha1 `29bddf144a2f2907528b25d616770bfea67c4792`, 10,291 classes), source
comparison against `versions/1.21.11/crates/`, JVM-free class-file scans
over the jar's entity classes, checks against the 1.21.11 datagen
reports (`e6-deltas/versions/1.21.11/datagen/reports/`), and a
three-classpath `javac` big-compile of all 391 oracle sources against the
prepared 1.21.11 `.local` jars + libraries (see Oracle defects). **A
prepared 1.21.11 `.local` EXISTS** in the worktree
(`versions/1.21.11/.local/minecraft/1.21.11/`, 260 MB, with
`horizon-client/1.21.11/attributes.json`) — the A3 attribute-values check,
not verifiable for 1.21.10, runs here and passes (zombie values spot-checked
against `Zombie.createAttributes`).

**Method — fresh anchors, no carry-over assumed.** 1.21.11 is the E6
template and its own lineage: its entity package layout is the full 26.x
form (`animal/equine`, `animal/feline`, `monster/zombie`,
`monster/skeleton`, `vehicle/minecart`, `npc/villager`,
`projectile/arrow`, `projectile/hurtingprojectile`), it HAS the
`world/level/gamerules` package (10 entries) and the `permissions` package
(19 entries), and its advancements criterion package is spelled
`criterion` (167 entries) — but it still has no `EntityTypes` /
`BlockEntityTypes` holder classes, no `world.clock` / `densityfunction`
packages, no `advancements/triggers` / `advancements/predicates`
packages, no `ServerboundPunchPacket`, and no `environment_attribute`
classes (data-only registry). Every vanilla anchor below was verified fresh
against 1.21.11's own jars (95 `javap` dumps in `evidence/javap/`, plus
fresh class-parser extractions: the 13 `BABY_DIMENSIONS` declarers, the
`getDimensions` / `getScale` / `getDefaultDimensions` / `getAgeScale`
override sets, the 73 `createAttributes` suppliers, the 15 fire-immune
types, all 157 adult dimensions + eye heights), never carried from 1.21.10
or 1.21.7. Where the 1.21.11 port was adapted for its own version (the
piglin finalize with the golden spear, the zombified-piglin spear, the
zombie spear arm, the cat goals, the slime moon-phase draws, the drowned
burn, the suffocation rule, the `handleInteract` form, the entity-components
driver), the adaptation was verified line by line rather than assumed
correct.

**Re-check first:** every re-check was decided against 1.21.11's own
jar/data — the 58 PRIMARY items from the 1.21.10 run and the 26 SECONDARY
items from the 1.21.7 run. Of the 1.21.10 items, 45 reproduce, 5 are FIXED
(G02 slime moon phase, G08 drowned burn, G16 trades, G18 entity-components
driver, G36 cat goals), 7 are WITHDRAWN (G01 bat Halloween — vanilla
1.21.11 removed the branch; G12 enderman walk-target — vanilla 1.21.11
changed; G17 `reduced()` — vanilla has the methods; G21 zombie spear —
`iron_spear` exists in 1.21.11; G48 zombie-horse burn — vanilla 1.21.11
burns it via the new `#burn_in_daylight` tag; O09 gamerules package and
O10 entity subpackages — both arrived in 1.21.11), 1 NEVER EXISTED (G42
piglin offhand). Of the 1.21.7 items, 15 reproduce, 4 are FIXED (G02, G08,
G16, G18), 7 are WITHDRAWN (G01, G12, G17, G21, O01→O09, O02
client.multiplayer, O03→O10). Three verdicts FLIP against the child's
direction and are the headline of this batch: **G09 reproduces in the
template** (the 1.21.10 child fixed the powder-snow burn exclusion; the
template never carried it), **G45 re-mints** (kinetic weapons exist in
1.21.11, so the double-push spear knockback is live again after being dead
code in 1.21.10), and **G48 withdraws** (vanilla 1.21.11's new
`#burn_in_daylight` tag includes the zombie horse — the port was right all
along). The two majors reproduce: **B3-G14 endermen never teleport** and
**B3-G22 the suffocation startup blocker** — the 1.21.11 client cannot load
its core tables with prepared 1.21.11 data. No NEW defects were minted
beyond the re-check set: every live defect is a re-mint, and the two
template-only findings (G09, G45) are re-mints of earlier items, not new
numbers.

## Tally

- **Checks: 99 — 65 pass, 34 gap as-tallied** (37 G items: 35 port +
  2 driver — the A3 food-table check yields three items, G05/G27/G28, and
  the B3 enderman/cat check yields two, G14/G15; every other gap check
  yields one). **Behaviour/non-behaviour split:** 96 behaviour checks
  (64 pass, 32 gap) + 3 non-behaviour driver-code checks (1 pass, 2 gap:
  G18 fixed, G19, G20) — driver-code-defect checks counted as-tallied per
  the 26.2 precedent, split reported explicitly.
- **Oracle defects: 8 groups** (B3-O01–B3-O07, B3-O11 — not counted in the
  check tally). O01–O07 carry the 26.1.2 grouping; O11 is the 1.21.7-only
  group (world.clock package). O08 is CLOSED (the line-74 void error is
  gone); O09–O10 are WITHDRAWN (the packages arrived in 1.21.11).
- **Oracle repairs done: none** (checks + triage only).
- **Severity: 2 major, 43 minor** — 2 major port (B3-G22, the
  `#causes_suffocation` startup blocker; B3-G14, endermen never teleport),
  33 minor port, 2 minor driver-side (B3-G19, G20), 8 minor oracle groups.
- **Slice items:** `slice.json` carries 59 items — the 45 live defects
  above plus 14 resolved re-checks per common.md (5 fixed: G02, G08, G16,
  G18, G36; 9 closed: G01, G12, G17, G21, G42, G48, O08, O09, O10).
- **Carry-over:** 15 of the 26 1.21.7 items re-mint (G03, G04, G05, G06,
  G09, G10, G11, G13, G14, G15, G19, G20, O04, O05, O06 — O04/O05/O06
  re-mint as O11/O05/O03 under the 26.1.2 grouping's numbering); 45 of
  the 58 1.21.10 items re-mint. The 1.21.11 template fixed five carried
  defects outright (G02, G08, G16, G18, G36) and narrowed two more (G04:
  only Hoglin now wrong, the Parched arm is correct; G19: only the
  registered-entity-fluid driver still hardcodes 35723, the suffocation
  driver now asserts the correct 29671). G30 widened: 18 species wrong
  in 1.21.10 → 20 in 1.21.11 (the two new breedable species, CamelHusk
  and ZombieNautilus, have no baby entry at all).

Per-section counts (must sum to 99): A1 1 (1 pass), A1b 11 (10 pass,
1 gap), A2 1 (1 pass), A3 11 (6 pass, 5 gap), A4 5 (4 pass, 1 gap),
B1 3 (1 pass, 2 gap), B2 27 (20 pass, 7 gap), B7 4 (3 pass, 1 gap),
B8 5 (2 pass, 3 gap), B5 4 (3 pass, 1 gap), B4 6 (4 pass, 2 gap),
B6 3 (2 pass, 1 gap), B3 12 (4 pass, 8 gap), B9 3 (3 pass),
Drivers 3 (1 pass, 2 gap).

## Defects

### B3-G03 — `immune_tag` tags don't exist (minor, port)

Vanilla 1.21.11 has no `*_immune_to` damage-type tags at all: all six
`immune_*` tag lookups return 0 entries in the client jar (only
`wither_immune_to` exists, as a damage_type tag, not an entity tag).
Vanilla immunity is code, not tags — the fresh `EntityType` parse shows 6
literal `immuneTo` sets (fox → sweet_berry_bush; polar_bear / snow_golem /
stray → powder_snow; wither / wither_skeleton → wither_rose). The port's
`immune_tag` (`mobs/spawn_rules.rs:179`) maps Fox → `minecraft:fox_immune_to`,
PolarBear → `minecraft:polar_bear_immune_to`, SnowGolem →
`minecraft:snow_golem_immune_to`, etc. — lookups that fail soft, so the
six species take damage vanilla spares them. Reproduces from 1.21.7 and
1.21.10.

### B3-G04 — Hoglin wrongly disallowed in peaceful (minor, port)

Vanilla 1.21.11 `EntityType` parse: `notInPeaceful` has 37 members and
HOGLIN is absent (it spawns and persists in peaceful); PARCHED is present.
The port's `allowed_in_peaceful` (`mobs/spawn_rules.rs:134`) excludes
Hoglin — narrowed since 1.21.10: the Parched arm is now correct, only the
Hoglin arm is wrong. Reproduces from 1.21.7 and 1.21.10 (narrowed).

### B3-G05 — Camel has no food (minor, port)

Vanilla 1.21.11 `Camel.isFood` accepts `#camel_food`; the port's
`food_tag` (`mobs/species.rs:1666`) has no Camel arm (CamelHusk has
`camel_husk_food` at :1679, Camel has none) — camels cannot breed.
Reproduces from 1.21.7 and 1.21.10.

### B3-G06 — Armadillo scared head rotation ignored (minor, port)

Vanilla 1.21.11 `Armadillo.getMaxHeadYRot` = `isScared() ? 0 : 32`
(fresh dump `Armadillo.txt:776-784`). The port's `max_head_y_rot`
(`mobs/species.rs:1745`) returns 32.0 unconditionally
(`Species::Armadillo => 32.0` at :1752). Reproduces from 1.21.7 and
1.21.10.

### B3-G09 — Sun-burn ignores powder snow (minor, port)

Vanilla 1.21.11 `Mob.isSunBurnTick` excludes `isInWaterOrRain ||
isInPowderSnow || wasInPowderSnow` and gates on the `MONSTERS_BURN`
environment attribute. The port's `sun_burn_tick` (`mobs/ai.rs:2024`)
computes `wet = m.fluids.in_water || in_rain(m, w)` (:2035) — no powder
snow term at all. **The 1.21.10 child fixed this; the 1.21.11 template did
not carry the fix.** Reproduces from 1.21.7 (was FIXED in 1.21.10).

### B3-G10 — Creatures spawn too far from players (minor, port)

Vanilla 1.21.11 `EntityType` parse: `canSpawnFarFromPlayer` is explicit
for exactly two types, PILLAGER and SHULKER. The port's
`spawns_far_from_player` (`mobs/spawn_selection.rs:317`) returns true for
the whole CREATURE and MISC categories. Reproduces from 1.21.7 and 1.21.10.

### B3-G11 — `getRandomPosWithin` y-range off by one (minor, port)

Vanilla 1.21.11 `NaturalSpawner.getRandomPosWithin`: top =
`getHeight(WORLD_SURFACE)+1`, y drawn with `randomBetweenInclusive(minY,
top)` — H+1 is reachable. The port's `spawn_category_for_chunk`
(`mobs/spawn_selection.rs:1105-1108`) uses top = `heightmap(WorldSurface)`
and draws `y = min + next_int_bound(top - min + 1)`, i.e. y ∈ [minY, H]:
the port's `world_surface` (`horizon-client-render/src/chunk.rs:724`,
`prime` stores highest+1, same convention as vanilla's getHeight), so the
port is exactly one level short and the RNG stream diverges from vanilla
on every chunk. Reproduces from 1.21.7 and 1.21.10.

### B3-G13 — Constant-count packs skip an RNG draw (minor, port)

Vanilla 1.21.11 always draws `nextInt` for a pack count, even when min ==
max. The port's `sample_count` (`mobs/spawn_selection.rs:782`) returns
`entry.min` directly when `entry.constant`, skipping the draw — the RNG
stream diverges from vanilla on every constant-count pack. Reproduces
from 1.21.7 and 1.21.10.

### B3-G14 — Endermen never teleport (MAJOR, port)

Vanilla 1.21.11 enderman teleport grounds on code — `blocksMotion()` —
not a tag. The port's enderman teleport (`mobs/enderman.rs:76,117`)
grounds on the absent 26.x tag `#entities_can_teleport_to` (0 entries in
either 1.21.11 jar): the ground search never finds a block, so endermen
never teleport. Reproduces from 1.21.7 and 1.21.10.

### B3-G15 — Cat morning gift never happens (minor, port)

Same absent tag, second victim: the port's `give_morning_gift`
(`mobs/tame.rs:1328`) descends from the cat's own position looking for a
block tagged `#entities_can_teleport_to` (:1341) — with the tag absent
everywhere, the ground falls to the world bottom and the hop never
happens. Two defects in one: the tag grounding (vanilla grounds on
`blocksMotion()`) and the origin — vanilla hops from the LEASH-HOLDER's
position (the sleeping owner), the port from the cat's. The gift itself
still drops; only the hop is broken. (The gift-chance gate itself matches
vanilla — see Re-checked, not defects.) Reproduces from 1.21.7 and
1.21.10.

### B3-G22 — Default suffocation is tag-gated; startup blocker (MAJOR, port)

The port's suffocation rule itself is fixed (`_ => MotionFullCollision`),
but the load path still hard-requires the absent tags:
`motion.rs:136` `suppresses_bounce: tag("minecraft:suppresses_bounce")?`
and `motion.rs:139` `causes_suffocation:
tag("minecraft:causes_suffocation")?` — both tags have 0 entries in the
1.21.11 jars, and the `?` fails the whole `MotionResources::load`, so the
client cannot start with prepared 1.21.11 data. Reproduces from 26.1.2
and 1.21.10.

### B3-G23 — Sleeping mobs suffocate (minor, port)

Vanilla 1.21.11 `Entity.isInWall` / the suffocation check exempts
sleeping mobs; the port's in-wall check in `mobs/ai.rs` has no sleeping
check, so a sleeping mob inside a block takes suffocation damage.
Reproduces from 26.1.2 and 1.21.10.

### B3-G24 — Villager level-up is immediate (minor, port)

Vanilla 1.21.11 defers the profession level-up: the trade sets
`updateMerchantTimer = 40` and the level rises in
`increaseProfessionLevelOnUpdate` when the timer expires. The port levels
the villager up immediately on the trade. Reproduces from 26.1.2 and
1.21.10.

### B3-G25 — Furnace minecart too fast under `minecart_improvements` (minor, port)

Vanilla 1.21.11 `AbstractMinecart.getMaxSpeed` applies the 0.75 / 0.5
furnace factor (experiment-gated on `minecart_improvements`). The port's
`max_speed_new` (`game/minecarts.rs:991`) lacks the factor — a furnace
minecart runs at full speed. (The step cap at :1069 correctly uses the
base form — vanilla calls the base form there too.) Reproduces from
26.1.2 and 1.21.10.

### B3-G26 — BreedGoal ignores partner panic (minor, port)

Vanilla 1.21.11 `BreedGoal.canUse` requires the partner not be panicking;
the port's BreedGoal starts the approach regardless of the partner's
panic state. Reproduces from 26.1.2 and 1.21.10.

### B3-G27 — Zombie horse has no food (minor, port)

Vanilla 1.21.11 `ZombieHorse.isFood` overrides to the `#zombie_horse_food`
tag — **the tag EXISTS in 1.21.11** (new since 1.21.10, where the item was
framed as a missing-tag defect; the evidence is corrected here, the
verdict is unchanged). The port's `food_tag` has no ZombieHorse arm, so
zombie horses cannot be fed or bred. Reproduces from 26.1.2 and 1.21.10
(evidence corrected).

### B3-G28 — Skeleton horse has no food (minor, port)

Vanilla 1.21.11 `SkeletonHorse.isFood` overrides to its own food; the
port's skeleton horse inherits the generic `#horse_food` arm, so it
accepts food vanilla refuses and refuses food vanilla accepts.
Reproduces from 26.1.2 and 1.21.10.

### B3-G29 — Peaceful gate moved into `checkSpawnRules` (minor, port)

Vanilla 1.21.11 `SpawnPlacements.checkSpawnRules` has NO peaceful gate —
the gate lives in the per-type predicates (`Monster.checkMonsterSpawnRules`
checks PEACEFUL, `Monster.txt:140`); Phantom and Shulker predicates have
no PEACEFUL reference, so they spawn in peaceful. The port moved the gate
into `check_spawn_rules` (`mobs/spawn_rules.rs:320`:
`if !allowed_in_peaceful(s) && l.peaceful() { return false; }`), which
sits above the Phantom/Shulker arms (:409-410) — phantoms and shulkers
can no longer spawn in peaceful. (The per-type arms themselves — pillager
light ≤ 8 at :439, endermite/silverfish at :394, blaze/breeze/zoglin at
:376 — carry no gate of their own, so the moved gate is the only one.)
Reproduces from 26.1.2 and 1.21.10.

### B3-G30 — Baby-dimensions table carries the 26.2 form; 20 species wrong (minor, port)

The port's baby table (`mobs/species.rs`, 41 non-None entries) was
compared species-by-species against vanilla 1.21.11's own baby
dimensions — the 13 `BABY_DIMENSIONS` declarers (identical set/values to
1.21.10, `evidence/baby-dims-1.21.11.txt`), the `getAgeScale` overrides
(LivingEntity 0.5 default; Armadillo 0.6; Camel 0.45; Dolphin 0.65;
HappyGhast 0.2375; Turtle 0.3), and all 157 adult dimensions
(`evidence/adult-dims-1.21.11.txt`, default eye height = height × 0.85).
**20 species are wrong** — the 18 from 1.21.10 (pig, cow, mooshroom,
sheep, chicken, goat, panda, polar bear, wolf, cat, ocelot, armadillo,
dolphin, happy ghast, camel, hoglin, zoglin, strider) plus the two new
breedable species, **CamelHusk** (port `Baby::None`; vanilla baby =
adult × 0.45 via Camel, (0.765, 1.0688, 1.0238) — CamelHusk has
`getBreedOffspring` and `#camel_husk_food`, so baby camel husks are
gameplay-reachable) and **ZombieNautilus** (port `Baby::None`; vanilla
baby = adult × 0.5, (0.4375, 0.475, 0.1375) — ZombieNautilus has
`getBreedOffspring` and inherits `AbstractNautilus.isFood`, so baby
zombie nautiluses are gameplay-reachable; Nautilus itself has the correct
`Scale(0.5)` entry, so the omission is inconsistent within the port's own
table). Zoglin confirmed fresh: no `getDimensions` override, so baby =
adult × 0.5 = (0.6982422, 0.7, 0.595); the port has `Abs(0.75, 0.85,
0.625)`. Hoglin likewise adult × 0.5. Checked and NOT minted (same
treatment as 1.21.10): Frog, Parrot, WanderingTrader — code-wise ageable
but the baby state is unreachable through gameplay (frogs breed into
tadpoles; parrots and wandering traders don't breed). Reproduces from
26.1.2 and 1.21.10 (widened 18 → 20).

### B3-G31 — Camel sitting baby dims wrong; body-anchor offset uses the 26.2 double form (minor, port)

Vanilla 1.21.11 sitting camel baby dims are (0.765, 0.425, 0.38); the
port has (0.95, 0.425, 0.41). The body-anchor offset uses the 26.2 double
form; vanilla uses float `height − 0.375F*scale`. Reproduces from 26.1.2
and 1.21.10.

### B3-G32 — Heightmap predicates use absent tags (minor, port)

Vanilla 1.21.11 heightmap blocking is code-based (`blocksMotion()`).
The port's `heightmap_blocks` (`mobs/spawn_selection.rs:413-419`) reads
the absent 26.x tags `#blocks_motion_in_heightmap*` (0 entries in either
1.21.11 jar) — the OceanFloor predicate never matches, so ocean-floor
spawns misplace. Reproduces from 26.1.2 and 1.21.10.

### B3-G33 — Worn heads never reduce detection range (minor, port)

Vanilla 1.21.11 hardcodes five head rules (each matching head × 0.5
detection range, no clamp). The port reads the `mob_visibility` component
(0 occurrences in the 1.21.11 jars — the component does not exist here)
and clamps the result to 0..=10, so worn heads never reduce detection
range. Reproduces from 26.1.2 and 1.21.10.

### B3-G34 — Trader hurt gating uses the 10-tick hurt window (minor, port)

Vanilla 1.21.11 gates the wandering trader's hurt response on
`hurtMarked`; the port gates on the 10-tick `hurt_time` window, so the
trader reacts to hurts vanilla ignores and ignores hurts vanilla reacts
to. Reproduces from 26.1.2 and 1.21.10.

### B3-G35 — BreathAirGoal uses the five-column search with swapped start order (minor, port)

Vanilla 1.21.11 `BreathAirGoal` searches a 3×9×3 `betweenClosed` box,
calls `findAirPosition()` only, and never stops navigation (0 `nav.stop`
in the dump). The port uses a five-column search and calls `nav_stop`
first. Reproduces from 26.1.2 and 1.21.10.

### B3-G37 — Inhabited time accrues +1 unconditionally (minor, port)

Vanilla 1.21.11 accrues inhabited time from the game-time difference;
the port adds +1 unconditionally per tick, so inhabited time (and
everything downstream of it — local difficulty, raid eligibility) runs
fast. Reproduces from 26.1.2 and 1.21.10.

### B3-G38 — `fresh_dimensions` removed; pufferfish spawn at full size (minor, port)

Vanilla 1.21.11 has `fresh_dimensions` (newly spawned entities get
their fresh size); the port has no equivalent — `Pufferfish.getScale(0)`
= 0.5 in vanilla, so a pufferfish spawns at half size and grows, while
the port spawns it at full size. Reproduces from 26.1.2 and 1.21.10.

### B3-G39 — Spawn-egg entity data applied to spawners (minor, port)

Vanilla 1.21.11 `SpawnEggItem.useOn` calls only `setEntityId` on a
spawner, and `TrialSpawnerBlockEntity` has 0 `setEntityData` references —
spawn-egg NBT is dropped. The port applies the egg's entity data to the
spawner, so a spawner edited with a named/tagged egg spawns modified
mobs vanilla would never produce. Reproduces from 26.1.2 and 1.21.10.

### B3-G40 — Armadillo ball-up refuses in liquid (minor, port)

Vanilla 1.21.11 `Armadillo.checkExtraStartConditions` = `onGround()`
only — an armadillo balls up in liquid. The port refuses to ball up in
liquid. Reproduces from 26.1.2 and 1.21.10.

### B3-G41 — Slime dealDamage adds a team check vanilla lacks (minor, port)

Vanilla 1.21.11 slime/magma-cube `dealDamage` has no team check; the
port adds one, so slimes on the same team as their target deal no damage
where vanilla deals full damage. Reproduces from 26.1.2 and 1.21.10.

### B3-G43 — Fox chorus-fruit teleport emits a level event 2017 trail (minor, port)

Vanilla 1.21.11 chorus fruit data is `{"type":
"minecraft:teleport_randomly"}` with no `directional_particles` field —
the fox teleport emits no trail. The port's `items.rs` parses
`directional_particles` with `.unwrap_or(true)` and the fox teleport
emits level event 2017. Reproduces from 26.1.2 and 1.21.10.

### B3-G44 — Loaded sleeping villager position never refreshed (minor, port)

The port gates the sleeping-villager position refresh on
`has_class("AbstractBedBlock")` — `AbstractBedBlock` has 0 entries in
either 1.21.11 jar (1.21.11 has only plain `BedBlock`), so the refresh
never runs and a loaded sleeping villager keeps a stale position.
Reproduces from 26.1.2 and 1.21.10.

### B3-G45 — Spear stab knockback applied as two pushes instead of one summed push (minor, port)

**Re-mints in 1.21.11** (was WITHDRAWN in 1.21.10 — no kinetic weapons
there). Kinetic weapons EXIST in 1.21.11: vanilla
`LivingEntity.stabAttack` calls `causeExtraKnockback` ONCE with `0.4f +
getKnockback(...)`. The port's stab path calls it twice — 0.4, then
knockback/2 — so the knockback vector and its resistance scaling differ
from vanilla on every spear stab. Reproduces from 26.1.2 (dead code in
1.21.10, live again in 1.21.11).

### B3-G46 — Item-entity landing bounce uses the restitution form; bed bounce gone (minor, port)

Vanilla 1.21.11 has no `restituteMovementAfterCollisions` — bounce lives
in `SlimeBlock`/`BedBlock` `bounceUp` (−vy × 0.66 × (1.0/0.8) for beds).
The port applies a generic restitution form to item entities, so bed
bounce is gone and slime bounce uses the wrong factor. Reproduces from
26.1.2 and 1.21.10.

### B3-G47 — AI walk-target value misses the Zero arms and the Giant/Mooshroom arms (minor, port)

The port's AI-path walk-target value (`mobs/` AI, distinct from the
correct spawn-path table — see B3): Enderman/Pillager/Warden → −light()
where vanilla returns 0.0f; Giant → −light() where vanilla returns +cost;
Mooshroom → grass_block where vanilla wants MYCELIUM. Creaking and
Silverfish are correct. Reproduces from 26.1.2 and 1.21.10.

### B3-G49 — Idle timer resets before the `hasClientLoaded` gate (minor, port)

Vanilla 1.21.11 `handleInteract` is `if (!hasClientLoaded()) return;` →
getTarget → `resetLastActionTime()` (unconditional) → world-border check
→ `canInteractWithEntity(AABB, 3.0)` → dispatch: the timer reset happens
after the gate. The port's hub (`lan/hub.rs:4059`) resets
`last_action_tick` BEFORE the `hasClientLoaded` gate (:4064-4070), so a
not-yet-loaded guest's actions reset the idle timer vanilla would not
reset. Reproduces from 1.21.10 (NEW there).

### B3-G19 — DRIVER: state-count hardcode 35723 (minor, driver)

Narrowed since 1.21.10: the suffocation driver is FIXED — it now asserts
29671, which matches the 1.21.11 datagen block-state count exactly
(`blocks.json`: 1,166 blocks, 29,671 states). But
`test_registered_entity_fluid_parity.py:66` still asserts 35723 (the 26.3
count), so the driver fails its own registry-completeness check against
prepared 1.21.11 data. Tree-wide, 35 drivers hardcode 35723 (1 in Batch 3
scope, 34 others — full list in `evidence/drivers-35723.txt`).
Reproduces from 1.21.7 and 1.21.10 (narrowed).

### B3-G20 — DRIVER: 26.3 resources path (minor, driver)

Five Batch 3 drivers point at the 26.3 resources dir —
`test_locator_bar_parity.py:269`, `test_server_dragon_parity.py:111`,
`test_server_entities_parity.py:186`,
`test_server_knockback_parity.py:435`, `test_projectile_shoot.py:46` (all
`lab.LOCAL / 'horizon-client/26.3'`) — so they load 26.3 data (or fail to
find any) instead of the prepared 1.21.11 data. Tree-wide, 45 drivers use
the path (5 in Batch 3 scope, 40 others — full list in
`evidence/drivers-263-paths.txt`). Reproduces from 1.21.7 and 1.21.10.

## Re-checked, not defects

- **B3-G01 (1.21.10, 1.21.7) — WITHDRAWN.** Vanilla 1.21.11 REMOVED the
  Halloween branch from `Bat.checkBatSpawnRules` — no Oct 31 special-case
  at all. The port matches vanilla's current form exactly (always the
  coin flip + limit 4). Vanilla changed since 1.21.10.
- **B3-G02 (1.21.10, 1.21.7) — FIXED.** The port's slime spawn uses the
  environment-attribute form, which MATCHES vanilla 1.21.11: the `moon`
  timeline's `surface_slime_spawn_chance` is 0.5/0.375/0.25/0.125/0/
  0.125/0.25/0.375 = 0.5 × MOON_BRIGHTNESS, exactly the port's form.
  (1.21.11 is the version where the environment-attribute form became
  vanilla-correct.)
- **B3-G08 (1.21.10, 1.21.7) — FIXED.** `burns_in_day` includes Drowned
  (`mobs/species.rs`); Drowned inherits `Zombie.aiStep` +
  `isSunSensitive() == true`.
- **B3-G12 (1.21.10, 1.21.7) — WITHDRAWN as framed.** Vanilla 1.21.11
  `EnderMan.getWalkTargetValue` IS overridden, returning 0.0f; the port's
  spawn-table Zero is correct. (The AI-path check is separate — see G47.)
- **B3-G16 (1.21.10, 1.21.7) — FIXED.** The port has hardcoded
  `villager_trades.rs` at the crate root; `merchant.rs` falls back to
  `offers_from_listings` when no trade set exists.
- **B3-G17 (1.21.10, 1.21.7) — WITHDRAWN.** Vanilla 1.21.11 `Goal` HAS
  `reducedTickDelay` = `positiveCeilDiv(t,2)` and `adjustedTickDelay`;
  the port's `reduced(ticks)=(ticks+1)/2` + `adjusted`
  (`mobs/goals.rs:66-71`) matches.
- **B3-G18 (1.21.10, 1.21.7) — FIXED.** `test_entity_components_parity.py`
  now reads the 1.21.11 predicate form — line 32: "1.21.11 predicate keys
  are the bare names (components, type)"; line 41 builds `{'components':
  predicate['components']}`. The 26.x `minecraft:components` crash is
  gone. (The docstring still mentions `minecraft:components`, but the
  code handles the 1.21.11 form.)
- **B3-G21 (1.21.10, 1.21.7) — WITHDRAWN.** `iron_spear` EXISTS in
  1.21.11 (1,505 items in datagen `items.json`), and vanilla Zombie
  `populateDefaultEquipmentSlots` uses nextInt(6): 0 → iron sword, 1 →
  iron spear, else iron shovel — BOTH port arms match
  (`mobs/finalize.rs:1346`). Vanilla changed since 1.21.10.
- **B3-G36 (1.21.10) — FIXED.** Cat lie-on/sit-on goals use `#beds` +
  chest/lit furnace/bed foot, matching vanilla `CatLieOnBedGoal` /
  `CatSitOnBlockGoal`.
- **B3-G42 (1.21.10) — NEVER EXISTED.** Vanilla 1.21.11
  `Mob.setItemSlotAndDropWhenKilled` = setItemSlot + setGuaranteedDrop
  (2.0) + `persistenceRequired = true`; the Piglin barter arm =
  setItemSlot + setGuaranteedDrop only — the port matches exactly.
- **B3-G48 (1.21.10) — WITHDRAWN.** Vanilla 1.21.11 ADDED the
  `#burn_in_daylight` entity-type tag with 10 members including
  `zombie_horse` and `zombie_nautilus`, and `Mob.aiStep` burns via the
  tag (`Mob.txt:838-848`), not per-class overrides. The port's
  `burns_in_day` set matches the 10 exactly — the port was right, the
  1.21.10 item's premise (vanilla spares the zombie horse) fails in
  1.21.11. Vanilla changed since 1.21.10.
- **B3-O08 (1.21.10) — CLOSED.** The "'void' type not allowed here" error
  at `VanillaEntityFluidOracle` line 74 is gone in the 1.21.11 sources.
  The oracle still fails, but on other grounds (ServerLevel ctor at :41,
  two no-override errors at :52/:60, three missing symbols at :65/:68,
  one incompatible-types error at :69) — folded into O02/O03/O05. (O08
  was not in the brief's PRIMARY list; verified anyway.)
- **B3-O09 (1.21.10; 1.21.7's O01) — WITHDRAWN.** The `gamerules`
  package EXISTS in 1.21.11 (`net/minecraft/world/level/gamerules`, 10
  entries in both jars). The six oracles' gamerules failures are gone;
  those oracles now fail on other grounds (mostly O02 ctor-mismatch).
- **B3-O10 (1.21.10; 1.21.7's O03) — WITHDRAWN.** The 26.x entity
  subpackages EXIST in 1.21.11 (`projectile/arrow`, `animal/cow`,
  `vehicle/minecart`, `vehicle/boat`, `monster/zombie`, and the full set
  — horses in `animal/equine`, Cat in `animal/feline`, Villager in
  `npc/villager`, Zombie/Drowned/Husk in `monster/zombie`,
  Skeleton/Stray/Bogged/Parched in `monster/skeleton`, MushroomCow in
  `animal/cow`, Pillager in `monster/illager`, Pufferfish in
  `animal/fish`, Fox in `animal/fox`). The six oracles' subpackage
  failures are gone; those oracles now fail on other grounds.
- **B3-O02 (1.21.7) — WITHDRAWN as framed.** The `client.multiplayer`
  package EXISTS in 1.21.11's client jar (123 entries); server-only
  failures on it are expected, not breakage. Recorded in the report
  only, since its ID is taken by the live constructor-mismatch group.
- **Cat gift-chance logic matches vanilla** (not a defect): the `day`
  timeline's `cat_waking_up_gift_chance` is 0.0@tick 362, 0.7@23667 —
  exactly the port's hardcoded window — and `monsters_burn` is
  false@12542, true@23460, matching the port's burn gate.
- **Drowned equipment matches** (not a defect): vanilla nextFloat >
  0.9, nextInt(16) < 10 → trident else fishing rod; port
  `mobs/finalize.rs` matches.
- **Piglin finalize matches** (not a defect): reason != Structure →
  float < 0.2 → baby, else adult gets `createSpawnWeapon` (crossbow 0.5 /
  nextInt(10)==0 → golden spear / golden sword — verified fresh in
  `Piglin.txt:497-560`); the port's `finalize.rs:1704-1720` matches,
  including the 1.21.11-only golden spear.
- **Zombified-piglin spear matches** (not a defect): vanilla
  nextInt(20)==0 → golden spear else golden sword
  (`ZombifiedPiglin.txt:410-425`); the port matches.
- **`burn_undead` head/body-slot protection matches** (not a defect):
  vanilla `sunProtectionSlot()` = HEAD default, BODY for ZombieHorse and
  ZombieNautilus; the port's `body_slot` (`mobs/species.rs:1814`) covers
  ZombieNautilus (and Nautilus, HappyGhast, Wolf).
- **`handleInteract` spectator form matches** (not a defect — see B8):
  vanilla 1.21.11 has no spectator early-return before
  `resetLastActionTime`; the port resets the timer for spectator attacks
  too and sets the camera instead of attacking
  (`game/lan_actions.rs:3451+`).

## Recorded, not minted

- Dead 26.x-only `Species` arm with no vanilla 1.21.11 counterpart:
  SulfurCube (1 variant; the other four 1.21.10 dead arms — CamelHusk,
  Nautilus, ZombieNautilus, Parched — are LIVE in 1.21.11). Mannequin is
  NOT dead — it exists in 1.21.11.
- Port `burn_undead` head-slot protection + 160-tick ignite matches
  vanilla `Zombie.aiStep` / `AbstractSkeleton.aiStep` structure; only the
  powder-snow exclusion (G09) is missing.
- Port `walk_target` spawn table (`mobs/spawn_selection.rs:145`) is
  correct for 1.21.11 (Enderman/Creaking/Pillager/Warden → Zero, Giant →
  +cost, Mooshroom → mycelium at :458-459, Silverfish correct).
- `guest_attack` lacks vanilla's invalid-entity disconnect (item entity
  / XP orb / self / non-attackable arrow → disconnect + warn) — inherited
  26.3 form, reachable only with a modified client (vanilla's client
  never sends such packets); not minted, same as 1.21.10.
- `wasInPowderSnow` has no previous-tick field in the port, so even after
  a G09 fix a mob would burn one tick early after leaving powder snow
  (residual, not minted).
- Frog / Parrot / WanderingTrader baby dims: code-wise ageable but the
  baby state is unreachable through gameplay — checked, not minted (see
  G30).
- Trial-spawner `spawnsInPeaceful` handling matches; spawner-block
  `getRandomSpawnMobAt` peaceful omission is spawner-only.

## Re-check verdicts (1.21.10 PRIMARY items — all 58)

| Item | Verdict | Item | Verdict |
|------|---------|------|---------|
| B3-G01 | WITHDRAWN | B3-G30 | REPRODUCES |
| B3-G02 | FIXED | B3-G31 | REPRODUCES |
| B3-G03 | REPRODUCES | B3-G32 | REPRODUCES |
| B3-G04 | REPRODUCES | B3-G33 | REPRODUCES |
| B3-G05 | REPRODUCES | B3-G34 | REPRODUCES |
| B3-G06 | REPRODUCES | B3-G35 | REPRODUCES |
| B3-G08 | FIXED | B3-G36 | FIXED |
| B3-G09 | REPRODUCES | B3-G37 | REPRODUCES |
| B3-G10 | REPRODUCES | B3-G38 | REPRODUCES |
| B3-G11 | REPRODUCES | B3-G39 | REPRODUCES |
| B3-G12 | WITHDRAWN | B3-G40 | REPRODUCES |
| B3-G13 | REPRODUCES | B3-G41 | REPRODUCES |
| B3-G14 | REPRODUCES (major) | B3-G42 | NEVER EXISTED |
| B3-G15 | REPRODUCES | B3-G43 | REPRODUCES |
| B3-G16 | FIXED | B3-G44 | REPRODUCES |
| B3-G17 | WITHDRAWN | B3-G45 | REPRODUCES |
| B3-G18 | FIXED | B3-G46 | REPRODUCES |
| B3-G19 | REPRODUCES | B3-G47 | REPRODUCES |
| B3-G20 | REPRODUCES | B3-G48 | WITHDRAWN |
| B3-G21 | WITHDRAWN | B3-G49 | REPRODUCES |
| B3-G22 | REPRODUCES (major) | B3-O01 | REPRODUCES |
| B3-G23 | REPRODUCES | B3-O02 | REPRODUCES |
| B3-G24 | REPRODUCES | B3-O03 | REPRODUCES |
| B3-G25 | REPRODUCES | B3-O04 | REPRODUCES |
| B3-G26 | REPRODUCES | B3-O05 | REPRODUCES |
| B3-G27 | REPRODUCES | B3-O06 | REPRODUCES |
| B3-G28 | REPRODUCES | B3-O07 | REPRODUCES |
| B3-G29 | REPRODUCES | B3-O09 | WITHDRAWN |
| | | B3-O10 | WITHDRAWN |
| | | B3-O11 | REPRODUCES |

45 reproduce, 5 fixed, 7 withdrawn, 1 never-existed, 0 unverified.
(B3-O08 was not in the brief's PRIMARY list; verified anyway — CLOSED,
see Re-checked, not defects.)

## Re-check verdicts (1.21.7 SECONDARY items — all 26)

| Item | Verdict | Item | Verdict |
|------|---------|------|---------|
| B3-G01 | WITHDRAWN | B3-G15 | REPRODUCES |
| B3-G02 | FIXED | B3-G16 | FIXED |
| B3-G03 | REPRODUCES | B3-G17 | WITHDRAWN |
| B3-G04 | REPRODUCES | B3-G18 | FIXED |
| B3-G05 | REPRODUCES | B3-G19 | REPRODUCES |
| B3-G06 | REPRODUCES | B3-G20 | REPRODUCES |
| B3-G08 | FIXED | B3-G21 | WITHDRAWN |
| B3-G09 | REPRODUCES | B3-O01 | WITHDRAWN (→ O09) |
| B3-G10 | REPRODUCES | B3-O02 | WITHDRAWN |
| B3-G11 | REPRODUCES | B3-O03 | WITHDRAWN (→ O10) |
| B3-G12 | WITHDRAWN | B3-O04 | REPRODUCES (→ O11) |
| B3-G13 | REPRODUCES | B3-O05 | REPRODUCES (→ O05) |
| B3-G14 | REPRODUCES (major) | B3-O06 | REPRODUCES (→ O03) |

15 reproduce, 4 fixed, 7 withdrawn, 0 never-existed, 0 unverified.
(The 1.21.7 oracle groups re-mint under the 26.1.2 grouping's numbering
where they overlap — O05 into O05, O06 into O03, O04 into O11 — and
withdraw where the packages arrived in 1.21.11 — O01 into O09, O03 into
O10.)

## Passes

### A1 — Entity registry / spawn placements (1 check, 1 pass)

157 `EntityType` registrations in `EntityType.<clinit>` (parsed fresh);
82 `SpawnPlacements.register` calls (counted fresh in the dump; +3 since
1.21.10: CAMEL_HUSK, NAUTILUS, PARCHED). The port's `check_spawn_rules`
covers 81 of the 82 explicitly (the missing one is ENDER_DRAGON, handled
in the dragon module and never naturally spawned); the `_ => true`
default matches vanilla's unregistered → true. The port carries one extra
arm, SulfurCube (dead). PASS. (Placement types/heightmaps sub-check: the
registered heightmaps are covered by the B2 heightmap check — see G32.)

### A1b — Spawn rules (11 checks, 10 pass, 1 gap)

Per-type spawn rules match vanilla for slime (G02 fixed — moon-phase
draws via the environment-attribute form, which is vanilla-correct in
1.21.11), bat (G01 withdrawn — vanilla removed the Halloween branch, the
port matches), drowned/guardian/ghast/magma-cube/zombified-piglin light
and height rules, endermite/silverfish, blaze/breeze/zoglin, pillager
(light ≤ 8), cat/ocelot, wolf/fox, parrot, and the
`isValidPositionForMob` block table. GAP: G29 (peaceful gate moved — only
phantom/shulker diverge).

### A2 — Species coverage (1 check, 1 pass)

90 `Species` variants = 90 `def!` entries in `mobs/species.rs`. 89 are
live 1.21.11 types; 1 is a dead 26.x-only arm (SulfurCube — recorded, not
minted). The two vanilla living types without a `Species` variant are
handled elsewhere: ArmorStand in `entities.rs`, EnderDragon in
`game/creatures.rs` + `game.rs`; Player is separate;
EvokerFangs/LlamaSpit/ShulkerBullet are projectiles. PASS.

### A3 — Attributes / food / dimensions (11 checks, 6 pass, 5 gap)

Attributes registry 35/35 (jar `Attributes` fields = datagen
`minecraft:attribute` entries, same names incl. `tempt_range` and the
`waypoint_*` pair); `createAttributes` suppliers 73 (fresh class-parser
count, `evidence/createattributes-1.21.11.txt` — +3 since 1.21.10: the
new entity classes); categories exact (8 categories, counts from the
fresh `EntityType` parse: MISC 72, CREATURE 31, AXOLOTLS 1, AMBIENT 1,
MONSTER 44, WATER_AMBIENT 4, WATER_CREATURE 3, UNDERGROUND_WATER_CREATURE
1); fire-immune 15 (fresh count); the port's loader maps all 15 attribute
ids it reads (`mobs.rs`, `attr("...")` calls) and carries the full
`attributes` map through; **per-species attribute VALUES verified** (new
check — a prepared 1.21.11 `.local` exists, unlike 1.21.10: the port
loads `attributes.json` at runtime, `motion.rs:98`, and the prepared
file's zombie values — armor 2.0, attack_damage 3.0, follow_range 35.0,
movement_speed 0.23000000417232513, max_health 20.0 — match
`Zombie.createAttributes` exactly). GAPs: G04 (peaceful table), G05/G27/G28
(food table — one check, three items), G06 (head rotation), G30 (baby
dims), G31 (camel sitting dims).

### A4 — Sun-burn / protection slots (5 checks, 4 pass, 1 gap)

Burn set exact incl. Drowned (G08 fixed) and zombie horse + zombie
nautilus (G48 withdrawn — vanilla 1.21.11's `#burn_in_daylight` tag has
10 members, the port's set matches exactly); head/body-slot protection +
160-tick ignite match vanilla `Mob.aiStep` / `sunProtectionSlot()`
(HEAD default, BODY for zombie horse + zombie nautilus; port
`body_slot` covers ZombieNautilus); Husk correctly excluded
(`isSunSensitive() == false`). GAP: G09 (powder-snow exclusion missing —
the child fixed it, the template did not).

### B1 — Suffocation / fluid (3 checks, 1 pass, 2 gap)

15 `isSuffocating` calls in `Blocks.txt` (matches the 26.1.1
resolution); the port's suffocation rule is fixed (`_ =>
MotionFullCollision`). GAPs: G22 (startup blocker — the rule is fixed
but the load path still requires the absent tags) and G23 (sleeping
mobs suffocate).

### B2 — Spawning (27 checks, 20 pass, 7 gap)

Spot-verified fresh: `magic_light` exact (`mobs/spawn_selection.rs:
429-436`, the curved `f/(4−3f)` form with ambient); `checkDespawn`
structure matches (`mobs/ai.rs:2296+`, peaceful discard, persistent,
removeWhenFarAway, 128/64 distances, 600-tick no-action + 1/800 random,
32-block reset); `MeleeAttackGoal` 20-tick gate confirmed
(`mobs/goals.rs:1398`, `game_time - last_check < 20`). Carried on the
fresh-dump basis (every vanilla anchor re-read in the 1.21.11 dumps; port
files read fresh): mob categories, `getRandomSpawnMobAt`, `mobsAt`/
fortress, `right_distance`, `valid_position_for_type`,
`isValidPositionForMob`, obstruction table, `max_cluster_size`,
`isMaxGroupSizeReached`, `removeWhenFarAway`, `isSpawnPositionOk`,
`getSpawnAABB` / `spawnDimensionsScale` / `canSummon` /
`EntitySpawnReason`, `SpawnState`/`createState`, `PotentialCalculator`,
`getFilteredSpawningCategories`/local cap, spawn-rules tables. GAPs:
G03 (immune tags), G10 (far spawns), G11 (y-range), G13 (RNG draw),
G32 (heightmap tags), G37 (inhabited time), G38 (fresh dimensions).

### B7 — Spawners (4 checks, 3 pass, 1 gap)

Spawner defaults exact (vanilla `BaseSpawner` ctor, fresh dump: delay
20, min 200, max 800, spawn count 4, max nearby 6, range 16, spawn
range 4; port `mobs/spawner.rs:319,352-353,357` matches); `delay()`
matches (`mobs/spawner.rs:449-450`, max≤min→min); trial config defaults
exact (vanilla `TrialSpawnerConfig`, fresh dump: spawn_range 4, total
6.0, simultaneous 2.0, +2.0/+1.0 per player, 40 ticks; port
`horizon-client-render/src/chunk/block_entity.rs:638-656` matches).
GAP: G39 (spawn-egg entity data applied).

### B8 — Combat / LAN (5 checks, 2 pass, 3 gap)

`CombatRules` methods confirmed in the fresh dump; port
`horizon-world/src/combat_rules.rs` matches (`after_armor`: armor −
damage/divisor, clamp armor×0.2..20, /25; `after_magic`: clamp 0..20,
/25). **PASS — `handleInteract` spectator form.** Vanilla 1.21.11 has NO
separate `handleAttack` and NO spectator early-return: `handleInteract`
is `if (!hasClientLoaded()) return;` → getTarget →
`resetLastActionTime()` (unconditional) → world-border check →
`canInteractWithEntity(AABB, 3.0)` → dispatch. The port matches: the hub
resets `last_action_tick` for spectator attacks too and `guest_attack`
(`game/lan_actions.rs:3451+`) sets the camera for spectators instead of
attacking. GAPs: G41 (slime team check), G45 (spear stab double-push —
NEW check, kinetic weapons exist in 1.21.11), G49 (idle-timer gate
order). Not minted: `guest_attack` lacks vanilla's invalid-entity
disconnect — reachable only with a modified client.

### B5 — Projectiles (4 checks, 3 pass, 1 gap)

`AbstractHurtingProjectile` getInertia 0.95, getLiquidInertia 0.8
(fresh dumps; port `game/remote_projectiles.rs` / `game/dragon.rs:5872`
matches); `ThrownTrident.getWaterInertia` 0.99 (fresh dump; port
`game/trident.rs:334-335` matches); throwable drag 0.99 (port
`game/remote_projectiles.rs:948-950`; potion inertia 0.8 water / 0.99 air
at `game/potions.rs:1745` matches vanilla `ThrowableProjectile.tick`).
GAP: G46 (item-entity bounce restitution form).

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

### B9 — Finalize / equipment (3 checks, 3 pass)

Piglin finalize `float() < 0.2` at `mobs/finalize.rs:1704-1720`
(matches vanilla, incl. the crossbow/golden-sword weapon split and the
1.21.11-only golden spear at 1-in-10); zombified-piglin golden spear
1-in-20 matches (`finalize.rs:1364`, vanilla `ZombifiedPiglin.txt:
410-425`); drowned equipment verified fresh (vanilla: `nextFloat > 0.9`,
`nextInt(16)`, <10 → TRIDENT else FISHING_ROD; port `finalize.rs`
matches); zombie spear arm matches (G21 withdrawn — nextInt(6),
0 → sword, 1 → spear, else shovel). No gaps.

## Oracle defects

**Basis (this batch's own measurement, NOT provisional):** the 1.21.11
Batch 0 table had NOT landed when this batch finished (checked 2026-11-25;
`docs/sweep/1.21.11/` in the worktree holds only inventory files). Oracle
triage therefore ran its own measurement, replicating
`docs/sweep/measure-oracles.py`'s method exactly: `javac --release 21
-proc:none -Xmaxerrs 10000` (JDK 25.0.5) over all 391 oracle sources on
three classpaths — server (`.local` server-game.jar + 39 server libraries
+ jsr305), client (`.local` client.jar + 60 client libraries + jsr305),
both — script `evidence/compile-oracles.sh`, logs
`evidence/oracle-{server,client,both}.log`. Results (single pass per
classpath): server 4,786 errors (276 files fail), client 2,162 errors
(249 fail), both 2,162 errors (249 fail) — 142/391 files (36.3%) with
no errors attributed on the both-classpath. **Coordinator correction:
the comparable figure is Batch 0's fixpoint count — 127/391 (32.5%)
compile on at least one classpath** (server 102/289, client 127/264,
both 127/264; `docs/sweep/1.21.11/batch-0-oracles.json`). The 15-file
gap between the single-pass count (142) and the fixpoint count (127)
is cascade dependents: files with no errors of their own in the joint
compile whose symbols resolve only from broken files' sources, removed
by the fixpoint rounds. The correct comparison vs 1.21.10 is 127 vs
104 (both fixpoint). The B3-scope table (42 oracles) is
`evidence/oracle-table-b3-1.21.11.txt` (+ `.json`): **10 pass on at least
one classpath, 32 fail on all three** — the coordinator cross-checked
all 42 against the landed Batch 0 table: 0 disagreements. (The
`evidence/oracle-compile-b3-1.21.11.txt` artifact is a stale
single-round parse and is superseded by the table file; do not use it.)
re-verified fresh against the 1.21.11 jars and the 1.21.11 oracle sources.
The coordinator should still cross-check against the Batch 0 table when
it lands; no group verdict is expected to change, though n-values will
differ (Batch 0 uses a fixpoint big-compile; this measurement is a single
pass per classpath, so cascade-attribution may differ for O07).

- **B3-O01 — missing-override (3 oracles).** `Level.fuelValues()` is
  `public abstract` in 1.21.11 (fresh dump `Level.txt:1914`); the oracle
  level stubs lack the override (grep-confirmed 0 `fuelValues` references
  in the 1.21.11 tree sources: `VanillaActiveEffectTickOracle`,
  `VanillaAttributeUpdatesOracle`, `VanillaLivingLandingOracle`). The
  `Level` ctor is 8-arg in 1.21.11 (`Level.txt:73`), matching the stubs,
  so the fuelValues error is the only Level-related one — masked in the
  compile by the O11 clock error on the same files. Reproduces from
  26.1.2 and 1.21.10.
- **B3-O02 — constructor-mismatch (26 B3 oracles carry ctor errors).**
  `ServerLevel` ctor takes 11 args in 1.21.11 (fresh dump
  `ServerLevel.txt`: MinecraftServer, Executor, LevelStorageAccess,
  ServerLevelData, ResourceKey, LevelStem, boolean, long,
  List<CustomSpawner>, boolean, RandomSequences); oracle `World` stubs
  construct with the 26.3 10-arg form (e.g.
  `VanillaCombatRulesOracle` / `VanillaAllayInteractionStateOracle`
  `World(){super(null,null,null,null,0,0,null,false,0L,0);}` — 10 args,
  missing the final RandomSequences). `LocalPlayer` ctor takes 7 args
  (fresh dump `LocalPlayer.txt`); the oracle `Actor` passes 9
  (`VanillaAllayInteractionStateOracle:34`). `MinecraftServer` ctor
  takes 8 args (fresh dump `MinecraftServer.txt`); the oracle constructs
  with the 26.3 form (`VanillaEntitySoundPacketOracle`). The dominant
  ctor class in 1.21.11 is ServerLevel (17 oracles fail on it first),
  where 1.21.10's group was framed on LocalPlayer/MinecraftServer only.
  Reproduces from 26.1.2 and 1.21.10 (widened).
- **B3-O03 — no-override (19 B3 oracles).** 26.3 methods absent in
  1.21.11; per the both-classpath log: ActiveEffectTick, Arrow,
  AttributeUpdates, BlockUpdate, DamageImmunity, EffectNotifications,
  EntityDeathLoot, EntityFluid, EntitySoundPacket, FluidBox,
  LateEntityPair, LivingFall, LivingLanding, MobBreathing, NaturalSpawn,
  Raid, RegisteredPlayerCombat, ServerPlayerCombat, SpawnRules. (Includes
  1.21.7's O06 content. LivingFall confirmed fresh, since 1.21.11 lacks
  `resetCurrentImpulseContext` / `tryResetCurrentImpulseContext` /
  `propagateFallToPassengers`.) Reproduces from 26.1.2 and 1.21.10
  (widened 14 → 19).
- **B3-O04 — `EntityTypes` holder move (5 B3 oracles use it in code).**
  `EntityTypes` has 0 entries in either 1.21.11 jar (1.21.11 predates
  the holder move); 31 files tree-wide reference it, of which the
  B3-scope `Vanilla*Oracle` users are `VanillaAllayInteractionStateOracle`
  (:68), `VanillaEntityDeathLootOracle` (:303, :308),
  `VanillaLateEntityPairOracle` (:58, :63), `VanillaMobBreathingOracle`
  (:20) and `VanillaNaturalSpawnOracle`. (Transcripts `EntityTranscript`
  and `RideTranscript` and clients `KnockbackClient` /
  `LocatorBarClient` / `ProtocolClient` / `SquidClient` also reference
  it — drivers, not oracles, excluded from the oracle count per the
  brief.) Reproduces from 26.1.2 and 1.21.10.
- **B3-O05 — missing-symbol (15 B3 oracles).** Anchors verified fresh:
  `BlockTags` has 205 fields, none of CAUSES_SUFFOCATION /
  SUPPRESSES_BOUNCE / BLOCKS_FLUID_FLOW; `ClientboundUpdateAdvancements-
  Packet` is a plain class (no Record attribute) with private `added` /
  `removed` / `progress` fields and only `getAdded()` / `getRemoved()` /
  `getProgress()` accessors (no record-style accessors);
  `Enderman` (26.3 name) absent, `EnderMan` present; `densityfunction`
  package 0; `ServerboundPunchPacket` 0; `EntitySpawnRequest` /
  `NoiseSpawnFinder` / `MoveSimulationType` 0. Pure missing-symbol
  oracles: `VanillaPlayerEquipmentOracle` (n=1 — `Holder.bindComponents`
  absent), `VanillaTradeComponentsOracle` (n=7 —
  `RegistryLayer.WORLD` absent), `VanillaVillagerFarmingOracle` (n=2 —
  `FarmlandBlock` absent; 1.21.11 has `FarmBlock`). Reproduces from
  26.1.2 and 1.21.10.
- **B3-O06 — missing-package (RaidOracle + 2 more).** Advancements
  `triggers` / `predicates` packages absent (the real home is
  `advancements/criterion`, 167 entries — verified fresh); `RaidOracle`
  fails on `net.minecraft.advancements.triggers`. Two further 26.x-only
  packages are missing for B3 oracles: `world.entity.monster.cubemob`
  and `world.level.blockscan` (`VanillaBlockUpdateOracle`,
  `VanillaNaturalSpawnOracle`). Reproduces from 26.1.2 and 1.21.10.
- **B3-O07 — cascade (2 oracles).** `VanillaMobBreathingOracle`
  (:32-42, uses `VanillaBlockUpdateOracle.AIR` / `OVERWORLD_TYPE` /
  `build(...)` in code) and `VanillaSpawnRulesOracle` (:136, uses
  `VanillaBlockUpdateOracle.Server.class`) depend on the broken
  `VanillaBlockUpdateOracle` (fails on O11 clock, n=38). Reproduces from
  26.1.2 and 1.21.10.
- **B3-O11 — `world.clock` package (4 B3 oracles).** `net.minecraft.world.clock`
  has 0 entries in either 1.21.11 jar; 15 oracle sources tree-wide
  reference it, of which the B3 failures are `VanillaActiveEffectTickOracle`,
  `VanillaAttributeUpdatesOracle`, `VanillaBlockUpdateOracle`,
  `VanillaLivingLandingOracle` (all also carry the O01 fuelValues error
  except BlockUpdate). Reproduces from 1.21.7 and 1.21.10 (widened 2 → 4
  — BlockUpdate and LivingLanding newly fail on it here; in 1.21.10 their
  first errors were gamerules/subpackage).

**Passing Batch 3-scope oracles** (10 pass on at least one classpath in
this batch's own measurement): VanillaDamageSourceOracle,
VanillaEntityAttributeOracle, VanillaEntityObstructionOracle,
VanillaLivingHealthOracle, VanillaMobAirOracle, VanillaMobDiscardOracle,
VanillaPlayerHealthOracle, VanillaRaiderCelebrationOracle,
VanillaWorldSpawnPacketOracle (9 pass all three classpaths), plus
VanillaZombieCureSoundOracle (passes client+both, fails server-only on
`client.multiplayer` — expected, not breakage).

## Drivers (§D)

144 drivers were curated to 135 core (see common.md). Batch 3's driver-
code defects are G19, G20 (3 checks, 1 pass, 2 gap — counted as-tallied
per the 26.2 precedent, split reported in the Tally; G18 is fixed). No
driver was run (checks + triage only). Tree-wide driver-code defect
counts (re-counted fresh, listed in the slice's `out_of_scope_failures`):
35 drivers hardcode the 26.3 state count 35723 (1 in Batch 3 scope, 34
others); 45 drivers use a 26.3 resources path (5 in Batch 3 scope, 40
others — the narrow `horizon-client/26.3` pattern alone; the broad
all-`'26.3'`-path-forms count was not separately tallied for 1.21.11).
Full lists in `evidence/drivers-35723.txt` and
`evidence/drivers-263-paths.txt`.

## Method notes

- **Vanilla-check rule:** every vanilla anchor was verified against
  1.21.11's own jars (server-game for entity logic, client for data/tags/
  advancements), never the 26.3 base's form and never carried from
  1.21.10 or 1.21.7. The server-game jar's bundler wrapper (`server.jar`)
  was never javap'd.
- **Prepared data exists:** a 1.21.11 `.local` is present in the
  worktree, so the A3 attribute-values check ran (and passed) and the
  oracle measurement used the prepared jars + libraries. All other data
  checks rest on the jars + the datagen reports (1,505 items, 157 entity
  types, 35 attributes, 29,671 states, 1,166 blocks).
- **Class-parser extractions (JVM-free):** `evidence/baby_dims2-1.21.11.py`
  (adapted from the 1.21.10 script) for the 13 `BABY_DIMENSIONS`
  declarers; inline scans for the `getDimensions` / `getScale` /
  `getDefaultDimensions` / `getAgeScale` override sets, the 73
  `createAttributes` suppliers, the 15 fire-immune types, and all 157
  adult dimensions + eye heights (`evidence/adult-dims-1.21.11.txt`).
  `evidence/parse_entitytype.py` for categories / notInPeaceful /
  canSpawnFarFromPlayer / immuneTo.
- **Vanilla changed between 1.21.10 and 1.21.11 in five places this batch
  caught:** the Halloween branch was REMOVED from `Bat.checkBatSpawnRules`
  (G01 withdrawn); `iron_spear` and kinetic weapons arrived (G21
  withdrawn, G45 re-minted); the `#burn_in_daylight` tag arrived with 10
  members incl. zombie horse + zombie nautilus (G48 withdrawn); the
  `gamerules` package arrived (O09 withdrawn); the 26.x entity
  subpackages arrived (O10 withdrawn). The environment_attribute and
  game_rule registries also arrived (the `day`/`moon` timelines now
  carry `cat_waking_up_gift_chance`, `monsters_burn` and
  `surface_slime_spawn_chance` — G02's env-attribute form is
  vanilla-correct in 1.21.11).
- **The template is genuinely adapted, not a copy:** the piglin finalize
  (golden spear), the zombified-piglin spear, the zombie spear arm, the
  cat goals, the slime moon-phase draws, the drowned burn, the
  suffocation rule, the `handleInteract` form, the entity-components
  driver, the suffocation driver's 29671 — each verified fresh. The
  remaining defects are the un-adapted residue, not a wholesale 26.x
  carry-over. Two adaptations run the WRONG way: the 1.21.10 child fixed
  G09 (powder-snow burn) and the template did not, and the child
  narrowed G19's suffocation driver while the template's fluid driver
  still hardcodes 35723.
- **Oracle triage:** this batch's own three-classpath measurement (see
  Oracle defects), all anchors re-verified fresh. Cross-check against
  `docs/sweep/1.21.11/batch-0-oracles.json` when it lands.
- **Disk quota:** the run hit the VM disk quota mid-session (104G used);
  per the owner's standing rule the dead 1.21.8 `.local` (5.0G, merged
  port, last touched Nov 12) was deleted after the safety checks
  (git-ignored, pure prepare outputs + logs, not a live stream's
  `.local`). Disk went 104G → 99G used; no sweep evidence was lost.
- **Evidence:** `b03/evidence/` — `run-javap.sh`, `run-javap3.sh`,
  `javap/` (95 dumps incl. `BlockProperties-v.txt`,
  `Heightmap-v.txt`, `HeightmapTypes.txt`, `FeatureFlags.txt`,
  `BaseSpawner.txt`, `TrialSpawnerConfig.txt`, `ServerLevel.txt`,
  `LocalPlayer.txt`, `MinecraftServer.txt`, `ZombifiedPiglin.txt`,
  `AbstractNautilus.txt`, `Zoglin.txt`), `baby_dims2-1.21.11.py`,
  `baby_compare.py`, `baby-dims-1.21.11.txt`,
  `adult-dims-1.21.11.txt`, `createattributes-1.21.11.txt`,
  `port-baby-table.txt`, `oracle-table-b3-1.21.11.txt` (+ `.json`),
  `oracle-{server,client,both}.log`, `compile-oracles.sh`,
  `oracle-table-b3-1.21.10.txt` (the 1.21.10 reference table),
  `drivers-35723.txt`, `drivers-263-paths.txt`,
  `server-game-listing.txt`, `client-listing.txt`,
  `parse_entitytype.py`.

## Verdict

**99 checks as-tallied: 65 pass, 34 gap** (96 behaviour: 64 pass, 32 gap;
3 driver-code: 1 pass, 2 gap). **45 items: 37 G (35 port + 2 driver) +
8 O-groups; 2 major, 43 minor.** Of the 84 re-checks, 60 reproduce
(45 of 58 1.21.10 PRIMARY, 15 of 26 1.21.7 SECONDARY), 9 are fixed, 1
never existed, 14 withdrawn, 0 unverified. Three verdicts flip against
the child's direction: G09 reproduces in the template (the child fixed
it), G45 re-mints (kinetic weapons exist in 1.21.11), G48 withdraws
(vanilla 1.21.11 burns the zombie horse via the new tag). Two new
passing checks (A3 attribute values — a prepared `.local` exists; B9
fully green — the zombie spear arm matches). The two majors both
reproduce: **B3-G14 endermen never teleport (major)** and **B3-G22 the
suffocation startup blocker (major)** — the 1.21.11 client cannot load
its core tables with prepared 1.21.11 data. Oracle triage rests on this
batch's own three-classpath measurement; the coordinator cross-checked
all 42 in-scope oracles against the landed Batch 0 table: 0
disagreements (the 142/391 global figure in §C is corrected to Batch
0's fixpoint 127/391 — see the coordinator note there).
