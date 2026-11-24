# Batch 1 — Fluids + falling blocks (26.2 parity sweep)

**Version:** 26.2 · **Date:** 2026-11-24 · **Branch:** `parity-sweep-26.2`
(workdir `~/workspace/mcir-work/sweep-26.2`, synced to main `732c800f`) ·
**Scope:** `docs/PARITY-SWEEP.md` §4 Batch 1 — the pilot scope re-run against
26.2: water/lava spreading and flow, fluid pushing, bubble columns, concrete
powder solidification, lava fire spread + ignition + game rules, dripstone
growth + cauldron fill, falling blocks, cauldron fill levels, splash water.

**Status: DRAFT — not committed.** Checks + triage only, per the brief: no
fixes, no edits under `versions/26.2/`, no cargo builds, no oracle
measurement, no commits. Everything below is static analysis: `javap -c` on
the named 26.2 server-game jar
(`~/workspace/mcir-work/26.1-adapters/versions/26.2/jars/server-game.jar`,
DataVersion 4903, protocol 776), source comparison against
`versions/26.2/crates/`, tag/data reads from the 26.2 client jar
(`~/workspace/mcir-work/leakage-audit/jars/26.2-client.jar`) and the 26.2
datagen reports (`~/workspace/mcir-work/26.2/versions/26.2/datagen/reports/`),
and — after horizon-prepare for 26.2 finished during this run (see §Method) —
the prepared data at `versions/26.2/.local/horizon-client/26.2/`. The
coordinator commits these deliverables to
`docs/sweep/26.2/batch-1-fluids-falling.md` and
`pins/26.2/sweep/batch-01.json` (or their chosen paths).

**Coordinator note (2026-11-24):** this batch ran before Batch 0 landed
and recorded no oracle defects. Its oracle claims were reconciled
against the measured Batch 0 table
(`docs/sweep/26.2/batch-0-oracles.json`) — see §Oracle defects in scope:
five oracle-defect items (B1-O01–B1-O05) were added, each missing symbol
`javap`-verified against the 26.2 jar. Check-tally arithmetic verified:
§A–§I sum to 110 checks / 108 pass / 2 gap.

## Tally

- **Behaviour checks: 110 — 108 pass, 2 gap** (both gap checks are the one
  defect item B1-G02; see §Defects).
- **Defect items: 1** — B1-G02, minor. **No majors.** The two pilot majors
  (G1 bubble columns, G5 no fluid spreading) are CLOSED for 26.2.
- **Oracle defects: 5** (B1-O01–B1-O05 — coordinator reconciliation, see
  §Oracle defects in scope: 7 of the 13 in-scope oracles fail all three
  classpaths on 26.3-only API). Not counted in the check tally, following
  the 1.21.7 convention.
- **Needs-harness: none** — no B1-Hxx items. Every in-scope check was
  decidable statically; the few untraced sub-details are noted in §Method,
  not faked as passes.
- **Withdrawn candidates:** B1-G01 (number never attached to a defect in the
  surviving record — see §Withdrawn), plus the six pilot defects and two
  Batch 11 findings re-checked and closed as verified matching (§Closed).

## Defects

### B1-G02 — Concrete powder falling into water does not solidify on contact (minor, CARRIES OVER from pilot G2)

**Checks failed (2):** concrete-powder fall-step water check; landing
`stuck_in_water` flag.

**Vanilla 26.2.** `FallingBlockEntity.tick` (server-game.jar): a falling
concrete-powder entity computes `local4 = blockState.is(ConcretePowderBlock)`
and `local5 = local4 && fluid at blockPos is in the WATER tag`; if `local4`
and the delta's length-squared exceeds 1 it additionally clips with
`ClipContext` (COLLIDER, SOURCE_ONLY) from its previous position, and if the
clip hits water it moves the entity to the hit position and sets `local5 =
true`. The landing branch runs when `onGround || local5` — i.e. at the
FIRST water cell, not at the lakebed. `ConcretePowderBlock.onLand` then calls
`shouldSolidify(level, pos, replacedState)` — the REPLACED state (4th
parameter), which solidifies when the replaced state `canSolidify`
(fluid is WATER tag) or `touchesLiquid`.

**The port.** `horizon-client/src/game/falling_blocks.rs` `fall_step` has
NO concrete-powder water check at all — no `instanceof ConcretePowderBlock
&& fluid is WATER`, no fast-fall clip check — and landing runs only on
`on_ground`. `horizon-client/src/game/falling_damage.rs`
`finish_falling_block` calls `falling_block_landed(..., false)` — i.e.
`stuck_in_water=false` always (lines ~128-135). Powder therefore sinks to
the lakebed instead of solidifying at the first water cell, exactly as in
the 1.21.7 pilot (G2).

**What does match (so the fix is local to the client fall step):** the
`horizon-world` landing branch itself (`level/falling.rs`
`falling_block_landed`) matches vanilla: `would_continue = is_free(below) &&
(!concrete || !stuck_in_water)`, `may_replace` via a
`DirectionalPlaceContext(DOWN, EMPTY, UP)` placement check,
`would_survive`, and the `onLand` concrete check uses the REPLACED state,
correct per vanilla's `onLand` 4th parameter. Neighbour/placement
solidification also matches (`concrete_touches_liquid`,
`shapes.rs:1089`, including vanilla's mutable-position quirk in
`touchesLiquid`; placement `place.rs:773`; updateShape `shapes.rs:366`).

## Closed — pilot defects re-checked against 26.2

All six 1.21.7 pilot defects and both Batch 11 findings were re-checked
against 26.2's OWN jar and data. Five pilot defects and both Batch 11
findings are CLOSED (verified matching); G2 carries over as B1-G02 above;
G4 is NOT APPLICABLE as framed (see below). The pilot's systemic root cause
— 26.x tag-based logic for tags absent in 1.21.7 data — does NOT apply to
26.2 in the way it did to 1.21.7: where 26.2's data HAS the tag, the port
reads it; where 26.2 still lacks the tag, the port carries the hardcoded
1.21.7-style list form with comments noting the 26.3 difference.

- **G1 bubble columns — CLOSED.** All three tags EXIST in 26.2's data:
  `enables_bubble_column_push_up` (`soul_sand`),
  `enables_bubble_column_drag_down` (`magma_block`), fluid tag
  `bubble_column_can_occupy` (`water`). The logic matches vanilla
  (`level/shapes.rs` `bubble_column_can_occupy` 1127,
  `should_bubble_column_occupy` 1136, `try_schedule_bubble_column` 1143
  with delay 20 ✓; `level/tick.rs` `bubble_column_update_for` 543 matches
  `updateColumn`; updateShape `shapes.rs:268-278` tick 5; `canSurvive`
  `level/survive.rs:81-85`). Entity motion matches exactly
  (`motion/fluids.rs` `BubbleColumn::push`: surface drag max(-0.9, vy−0.03),
  surface push min(1.8, vy+0.1), inside drag max(-0.3, vy−0.03), inside push
  min(0.7, vy+0.06); push returns !surface = reset fall distance), and the
  surface detection (`bubble_column_at`) matches `entityInside`'s
  above-collision/above-fluid test.
- **G3 lava fire spread through solids — CLOSED.** `level/fluids.rs`
  `lava_random_tick` (799-854) stops at `blocks_motion` with a comment
  noting 26.3 reads the (absent) tag; `blocks_lava_fire_spread` does NOT
  exist in 26.2's data (no file, no `BlockTags` field). The full
  `randomTick` structure matches vanilla: `canSpreadFireAround` gate,
  cumulative offsets, `fire_state_for(test)` taking the flammable block's
  position.
- **G4 lava ignition ignores doFireTick — NOT APPLICABLE as framed.**
  Vanilla 26.2 has NO `doFireTick` rule at all. Fire spread is gated by
  `FIRE_SPREAD_RADIUS_AROUND_PLAYER` (int, default 128, min −1 —
  `GameRules` static init `registerInteger(..., 128, -1)`). The port
  implements it: `level/mod.rs:219-220,234` (rule field, default 128),
  `horizon-client/src/game.rs:1560-1574` `can_spread_fire_around`
  (radius == −1 → true; any `fire_players` entry within radius, dist² <
  radius² measured to the block's corner — equivalent to vanilla
  `distanceTo < radius` for radius ≥ 0). Spectator exclusion verified:
  `game.rs:3845` builds `fire_players` from the host player filtered by
  `!spectator` plus LAN guests filtered by `here(g) && !g.spectator()`,
  matching `ChunkMap.playerIsCloseEnoughTo` (spectator → false; Euclidean,
  strict `<`).
- **G5 no fluid spreading — CLOSED.** `washed_away_by_fluids` does NOT exist
  in 26.2's data, and the port does not read it: `level/fluids.rs`
  `can_hold_any_fluid` (359-376) implements vanilla 26.2's hardcoded list
  (LiquidBlockContainer → true; blocksMotion → false; excludes DoorBlock,
  `#signs`, ladder, sugar cane, bubble column, nether/end portals, end
  gateway, structure void; else true) with a comment noting 26.3 replaced
  it with the tag 26.2 lacks. The spread machinery matches vanilla
  end-to-end (§A). Note the port's `is_liquid_container`
  (`fluids.rs:78-81`) = implements(LiquidBlockContainer) OR
  implements(SimpleWaterlogged): the OR is REQUIRED and correct — the
  prepared export does not expand interface inheritance (all 429
  SimpleWaterlogged blocks list `SimpleWaterloggedBlock` but NOT
  `LiquidBlockContainer`; only 4 direct LBC blocks: kelp, kelp_plant,
  seagrass, tall_seagrass), while the jar confirms
  `SimpleWaterloggedBlock extends BucketPickup, LiquidBlockContainer`.
- **G6 dripstone never fills cauldrons — CLOSED.**
  `level/speleothem.rs` matches vanilla closely: transfer only for
  `Class::PointedDripstone`, growth chance 0.011377778, start-pos check,
  `find_block_vertical` exact (`1..max_steps`, target check before path
  check), `dripstone_fluid_above` (mud → water if `!water_evaporates`),
  `dripstone_transfer_fluid` (probabilities 0.17578125 water /
  0.05859375 lava, mud→clay + push-entities event + level event 1504,
  cauldron delay 50 + tip.y − cauldron.y), `cauldron_can_receive`
  (Cauldron → true; LayeredCauldron → water + water cauldron; else false),
  `cauldron_tick` (events 1047 water / 1046 lava, level < 3).
- **B11-G06 Speleothem vs PointedDripstoneBlock class name — CLOSED.**
  26.2's jar has `PointedDripstoneBlock extends SpeleothemBlock`, so
  `Class::Speleothem = "SpeleothemBlock"` matches via inheritance —
  confirmed two ways: `classes.rs` matches each block's exported class
  chain (most derived first), `instance_of` answering instanceof for any
  recognised ancestor; prepared chains: pointed_dripstone
  `['PointedDripstoneBlock','SpeleothemBlock','Block']`, sulfur_spike
  `['SulfurSpikeBlock','SpeleothemBlock','Block']`.
- **B11-G13 splash water douses-fire gate — CLOSED.** Vanilla 26.2 gates on
  `PotionContents.is(Potions.WATER)` (`AbstractThrownPotion.onHitBlock`),
  NOT a tag — `douses_fire` does NOT exist in 26.2's data and `PotionTags`
  has only `TRADEABLE`. The port implements exactly that:
  `game/splash_water.rs` `is_water_potion` = `stack.potion() ==
  Some("minecraft:water")`, with a comment noting 26.3 moved to tags 26.2
  lacks.

## Withdrawn

- **B1-G01 — WITHDRAWN (number unused).** The pre-compaction working notes
  assigned B1-G02 to the concrete-powder defect first; B1-G01 was never
  attached to a defect in the surviving record. Recorded here so the gap in
  numbering is explicit, not a missing item. The number is not reused.
- **Cauldron mayInteract candidate — NOT A DEFECT (nuance, §H).** Vanilla's
  `LayeredCauldronBlock.entityInside` runBefore lambda requires
  `entity.mayInteract(level, pos)` before lowering the level; the port has
  no such check. Verified against the jar, the gate is spawn protection +
  world border (`ServerLevel.mayInteract`: non-player → true; player →
  `!isUnderSpawnProtection && worldBorder.isWithinBounds`), which the
  port's single-player/LAN-host model does not implement as a concept.
  Recorded as a nuance, not a defect — the core behaviour (level lowering,
  powder-snow conversion, extinguish) matches.
- **Falling-block MOVING_PISTON branch — initial misread corrected.**
  Vanilla's `FallingBlockEntity.tick` jumps to the end of the tick (the
  entity keeps falling, NOT discarded) when the block at its position is a
  moving piston; the port's `falling_block_landed` Continue branch matches.

## Passes by section

Counts below are behaviour checks where BOTH the vanilla jar/data side and
the Rust side were actually compared. Untraced sub-details are listed in
§Method and are NOT counted as passes.

### §A. Fluid spreading and flow — 18 checks, 18 pass

- `FlowingFluid.canHoldAnyFluid` hardcoded form (see G5 above) — pass.
- `is_liquid_container` LBC OR SWL (see G5 above) — pass.
- `SimpleWaterloggedBlock extends BucketPickup, LiquidBlockContainer` in
  the 26.2 jar — pass.
- **Waterlogging exhaustive set — EXACT MATCH.** Vanilla datagen
  `blocks.json` blocks with any `waterlogged` property = 429; prepared
  `block-behaviours.json` blocks with `SimpleWaterloggedBlock` in
  interfaces = 429; zero differences in either direction. (Unblocked by
  prepare landing during the run.)
- `#signs` exclusion moot: `#signs` = `#standing_signs` + `#wall_signs`
  (24 blocks resolved from prepared `block-tags.json`); hanging signs (24)
  are NOT in `#signs` but DO extend `SignBlock` — moot for
  `can_hold_any_fluid` because ALL signs are SimpleWaterloggedBlock, so the
  LiquidBlockContainer branch returns true before the Sign exclusion is
  reached — pass.
- Water numbers: slope 4, dropOff 1, tickDelay 5 — pass.
- Lava numbers: slope 4 fast / 2 slow, dropOff 1 fast / 2 slow, tickDelay
  10 fast / 30 slow — pass.
- Lava `getSpreadDelay` ×4 when the target fluid height is higher and
  `nextInt(4) != 0` — pass.
- Spread machinery: `spread` / `spreadToSides` / `getNewLiquid` /
  `getSpread` / `slope_distance` / `spread_targets` (sorted by direction
  discriminant = Java `Direction.values()` order; Rust
  `Direction::ALL` in `horizon-core/src/geometry/mod.rs:44` is Down, Up,
  North, South, West, East — same order) — pass.
- `spread_to` incl. lava-into-water: direction == DOWN && this is LAVA tag
  && fluid at pos is WATER → if LiquidBlock → stone flag 3; fizz — pass.
- `can_be_replaced_with` (water: DOWN && !water; lava: height ≥
  0.44444445 && water) — pass.
- `can_hold_specific_fluid` — pass.
- `place_liquid` / `pickup_block` (kelp/seagrass/barrier/slab-double
  exclusions) — pass.
- `should_spread_liquid`: lava + water neighbour (5
  POSSIBLE_FLOW_DIRECTIONS, checked via opposite) → obsidian if this is
  source else cobblestone + fizz; over soul soil + blue ice neighbour →
  basalt — pass.
- `liquid_block_changed` scheduling — pass.
- `getFlow` vector: height-difference algorithm with `0.8888889f`,
  below-check skipped when the neighbour `blocksMotion()`; falling fluid
  normalises and adds (0, −6, 0) if a solid face is at the neighbour or
  above it; `affectsFlow` = empty || same family; `isSolidFace`:
  same-family fluid → false, UP → true, ice → false, else
  `isFaceSturdy` — pass, except the below-check predicate: vanilla uses
  `blocksMotion()`, the Rust uses `blocks_fluid_flow` — but the port
  deliberately maps `blocks_fluid_flow := blocks_motion` for 26.2
  (`horizon-client-render/src/assets.rs:74,212,283`;
  `motion/fluids.rs:74-79` `blocks_flow()` is documented "Per-state
  `BlockState.blocksMotion()`", built from the `blocks_fluid_flow` arg
  which the scene fills from the blocks_motion table). Pass.
- `can_convert_to_source` + 2-source rule: `fluids.rs:151-160` reads the
  game rules `water_source_conversion` (default true) /
  `lava_source_conversion` (default false), matching vanilla
  (`WaterFluid.canConvertToSource` = `WATER_SOURCE_CONVERSION`,
  `iconst_1`; `LavaFluid.canConvertToSource` = `LAVA_SOURCE_CONVERSION`,
  `iconst_0`); `fluids.rs:414` 2-source conversion matches `getNewLiquid`
  (2+ same-family sources horizontally through-wall, solid below or source
  below) — pass.

### §B. Bubble columns — 10 checks, 10 pass

Tags exist (see G1). `canOccupy` = is bubble_column OR (fluid in
`bubble_column_can_occupy` AND LiquidBlock AND isSource AND amount ≥ 8);
`getColumnState`: below is column → below state; push_up tag → drag=false;
drag_down tag → drag=true; occupy is column → water default; else occupy —
pass. updateShape (schedule water fluid tick; schedule block tick 5 if
`!canSurvive` OR dir == DOWN OR (dir == UP AND neighbour not column AND
canOccupy)) — pass. `canSurvive` (below is column or push_up/drag_down
tag) — pass. `tryScheduleBubbleBlockColumn` delay 20 (`bipush 20`) — pass.
BubbleColumn/Liquid tick dispatch (`tick.rs:66`, `:231`) — pass.
`entityInside` surface-vs-inside split (above block empty collision shape
AND empty fluid → `onAboveBubbleColumn`, else `onInsideBubbleColumn`) —
pass. `handleOnAboveBubbleColumn` (drag: vy = max(-0.9, vy−0.03); push: vy
= min(1.8, vy+0.1); then bubble particles) — pass.
`handleOnInsideBubbleColumn` (drag: max(-0.3, vy−0.03); push: min(0.7,
vy+0.06); then `resetFallDistance()`) — pass. `bubble_column_at` surface
detection — pass.

### §C. Lava fire, ignition and game rules — 16 checks, 16 pass

- `lava_random_tick` structure incl. `canSpreadFireAround` gate,
  `nextInt(3)` passes, first branch (cumulative `offset(nextInt(3)-1, 1,
  nextInt(3)-1)`, air + flammable neighbours → set fire via
  `BaseFireBlock.getState(level, test)`, blocksMotion → return), else
  branch (3 iters `offset(dx,0,dz)` from origin, `isEmptyBlock(above) &&
  isFlammable(test)` → `setBlockAndUpdate(above(test),
  BaseFireBlock.getState(level, test))` — getState takes the FLAMMABLE
  block's pos, not the fire pos) — pass.
- `blocks_motion` stop, no tag (see G3) — pass.
- No `doFireTick`; `fire_spread_radius_around_player` default 128, min −1
  (see G4) — pass.
- `can_spread_fire_around` incl. spectator exclusion (see G4) — pass.
- `FireBlock.tick` step by step: scheduleTick(30 + `nextInt(10)`) FIRST,
  then the `canSpreadFireAround` gate; `!canSurvive` → removeBlock (no
  return); infiniburn via `dimensionType.infiniburn` HolderSet; rain
  (`!infini && raining && isNearRain && nextFloat < 0.2 + age*0.03` →
  remove + return); new_age = min(15, age + `nextInt(3)/2`), setBlock flag
  260; `!infini`: `!isValidFireLocation` → if `!sturdy(below, UP) || age >
  3` remove, return; age == 15 && `nextInt(4) == 0` && `!canBurn(below)` →
  remove + return; `INCREASED_FIRE_BURNOUT` env attribute → extra −50 —
  pass.
- `checkBurnOut` (east/west 300, down/up 250, north/south 300 (+extra);
  `nextInt(chance) < burnOdds`; `nextInt(age+10) < 5 && !isRainingAt` →
  setBlock(getStateWithAge, 3) else removeBlock; TNT →
  `TntBlock.prime`) and the ignite loop (x,z −1..1, y −1..4, skip origin,
  rate = 100 (+(y−1)*100 if y > 1), odds = (ignite+40+difficultyId*7) /
  (age+30), burnout odds /= 2, `nextInt(rate) <= odds` &&
  (`!raining || !isNearRain`) → spread_age = min(15, age +
  `nextInt(5)/4`), setBlock(getStateWithAge, 3)) — pass.
- **Fire odds VALUES — FULL MATCH.** Parsed vanilla `FireBlock.bootStrap`
  (178 direct `setFlammable` entries + 16 wool (30,60) + 16 carpet (60,20)
  = 210 raw, of which 3 are parser artifacts: the `wool`/`carpet`
  ColorCollection getstatics and `fire` from the `Blocks.FIRE` load) vs
  prepared `block-behaviours.json` fire.igniteOdds/burnOdds (207 each):
  ZERO value diffs, zero prepared-only entries; only-vanilla = exactly the
  3 artifacts. `getBurnOdds`/`getIgniteOdds`: waterlogged → 0 else map
  value (`level/survive.rs:503`) — pass. (Unblocked by prepare landing
  during the run.)
- `fire_state_for` matches `BaseFireBlock.getState` (below soul-fire-base
  → soul fire, else `FireBlock.getStateForPlacement` directional props if
  `!canBurn(below) && !sturdy(below, UP)`); `fire_state_with_age` (if FIRE
  set age else as-is) — pass.
- `infiniburn_overworld` = netherrack, magma_block; `infiniburn_nether` =
  `#infiniburn_overworld`; `infiniburn_end` = `#infiniburn_overworld` +
  bedrock — pass.
- `LavaFluid.entityInside`: applies `CLEAR_FREEZE` + `LAVA_IGNITE` +
  `runAfter(LAVA_IGNITE, Entity::lavaHurt)` — lambda confirmed via
  `javap -v` BootstrapMethods as method ref `Entity.lavaHurt`. Player
  `motion/environment.rs:121-124` (`is_in_lava` → ignite_for_ticks(300) +
  hurt lava 4.0) and mob `mobs/ai.rs:31-56` `block_contact_damage` (lava
  && !fire_immune → ignite 300 + hurt 4.0 Lava; ignite then hurt) match —
  pass. (The lava-hurt GENERIC_BURN sound — vol 0.4, pitch 2+rand*0.4 —
  was NOT traced in the Rust; see §Method.)
- `Entity.lavaIgnite`: if !fireImmune → `igniteForSeconds(15)` (= 300
  ticks) — pass.
- `Entity.lavaHurt`: if !fireImmune → server-side hurt 4.0 lava — pass.
- `LivingEntity.igniteForTicks` override: `super.igniteForTicks(ceil(ticks
  * getAttributeValue(BURNING_TIME)))` — verified via jar-wide
  constant-pool scan (only LivingEntity, Attributes, Enchantments
  reference BURNING_TIME). Player `motion.rs:513-519` and mob
  `mobs.rs:2778` `ignite_for_ticks` (ceil(ticks × burning_time), max,
  clears frozen) match; base `Entity.igniteForTicks` = max +
  `clearFreeze()` (no scaling) — pass.
- `BaseFireBlock.fireIgnite` (FIRE_IGNITE effect): if fireImmune return;
  if remainingFireTicks < 0 → +1; else if ServerPlayer → + `nextInt(1,3)`
  (+1 or +2); if remaining >= 0 → `igniteForSeconds(8)` (= 160 ticks,
  scaled by BURNING_TIME via the LivingEntity override). Rust
  `ignite_contact` (`motion/environment.rs:366-384`: ticks<0 → +1; player
  → +1+rand(2); if >= 0 → max(ticks, ceil(160 × multiplier))) matches —
  pass. `InsideBlockEffectType` bootstrap mapping verified: FREEZE →
  lambda (setIsInPowderSnow + freeze ticks), CLEAR_FREEZE →
  `Entity.clearFreeze`, FIRE_IGNITE → `BaseFireBlock.fireIgnite`,
  LAVA_IGNITE → `Entity.lavaIgnite`, EXTINGUISH → `Entity.clearFire`;
  `StepBasedCollector` applies in enum order (APPLY_ORDER = values()),
  before-effects, effect, after-effects per type; `applyAndClear` stops
  final effects if the entity dies mid-way (`isAlive` check breaks loop).
- `LavaCauldronBlock.entityInside`: CLEAR_FREEZE + LAVA_IGNITE +
  runAfter(LAVA_IGNITE, lambda) — same shape as LavaFluid. Rust
  `lava_cauldron_inside` (`motion.rs:2019-2024`: ticks_frozen=0, ignite
  300, hurt lava 4.0) matches — pass. (The lambda's identity as
  `Entity::lavaHurt` was not bootstrap-verified for the cauldron class,
  only read; the behaviour matches regardless.)
- `Entity.baseTick` fluid phase order: wasEyeInWater ← isEyeInFluid(WATER);
  updateFluidInteraction(); updateSwimming(); server-side fire (if
  remainingFireTicks > 0: fireImmune → clearFire; else if ticks % 20 == 0
  && !isInLava → hurt onFire 1.0; ticks -= 1; client-side: clearFire());
  if isInLava → fallDistance *= 0.5; checkBelowWorld;
  setSharedFlagOnFire; firstTick = false. Rust `fluid_base_tick` order
  (was_eye_in_water → update_fluids → swimming → tick_remaining_fire →
  lava fall halving ×0.5 → first_tick=false) and `tick_remaining_fire`
  match — pass.

### §D. Entity fluid interaction and pushing — 6 checks, 6 pass

- `EntityFluidInteraction.update` (`horizon-world/src/entity_fluid.rs`):
  reset trackers; box = getFluidInteractionBox (null → return); floor/ceil
  bounds; `hasFluidAndLoaded(level, minX−1, minY, minZ−1, maxX+1, maxY,
  maxZ+1)` else return; per cell: fluid non-empty; top = y +
  fluid.getHeight; top < box.minY skip; tracker = first tag holding the
  fluid type (cached by last kind); eyes: x==blockX && z==blockZ && eyeY
  in [y, top] → eyesInside; height = max(height, top − bbox.minY); if
  !skipPush: flow = getFlow; if height < 0.4 → flow.scale(height);
  accumulate. Loop order x outer, y middle, z inner — pass.
- `has_fluid_and_loaded`: Z-then-X chunk loop, ascending Y sections,
  missing chunk → false immediately, no early success, `present |=
  hasFluid` — pass.
- `Tracker.applyCurrentTo`: count == 0 OR accumulated.lengthSqr() <
  9.999999747378752E-6 (=(float)1e-5 as double) → return — the 1e-5 floor
  lives in the Tracker in 26.2 (the pilot noted 1.21.7 had it inline in
  Entity); the Rust matches with `f64::from(1.0e-5f32)`. Non-Player →
  normalize(accumulated); Player → accumulated.scale(1/count); scale by
  speed; if |delta.x| < 0.003 && |delta.z| < 0.003 && current.length() <
  0.0045 → normalize.scale(0.0045); addDeltaMovement — pass.
- `Entity.updateFluidInteraction` (`entity_fluid_tick.rs` `update`):
  `interaction.update(entity, !isPushedByFluid)` (ignore queried twice);
  inWater/inLava from interaction; inWater → resetFallDistance + splash if
  !wasTouchingWater && !firstTick; wasTouchingWater = inWater; if
  isPushedByFluid: water → applyCurrentTo(WATER, 0.014); lava →
  FAST_LAVA ? 0.007 : 0.0023333333333333335; return inWater || inLava.
  `entities/fluids.rs` `update_entity_fluids` (items/orbs/stands/boats)
  same scales — pass.
- `Entity.isInLava` = `!firstTick && isInFluid(LAVA)`; Rust
  `motion/fluids.rs` `is_in_lava` matches — pass.
- `getFluidInteractionBox`: boundingBox.deflate(0.001), then if a vehicle
  is present, the vehicle's `modifyPassengerFluidInteractionBox(box)`.
  Rust `horizon-world/src/fluid_box.rs` `interaction_box` matches; the
  boat override `boat_passenger_box` matches `AbstractBoat`'s bytecode
  exactly: isUnderWater → box unchanged; boatTop >= passenger maxY →
  null; else new box with minY = max(minY, boatTop) (with Java
  `Math.max` NaN/−0.0 semantics) — pass.

### §E. Concrete powder — 7 checks, 5 pass, 2 gap

Both gap checks are B1-G02 (§Defects). Passing: `falling_block_landed`
branch; `onLand` replaced-state check; `concrete_touches_liquid` incl. the
mutable quirk (`Direction.values()` loop; first `getBlockState` reads the
current mutable; DOWN only proceeds if `canSolidify(current)`);
`canSolidify` = fluid is WATER tag; placement (`place.rs:773`) and
updateShape (`shapes.rs:366`) solidification. Nuance (not a defect): Rust
waterlogged-on-landing uses `is_water_fluid` (WATER tag, incl. flowing)
where vanilla requires source `Fluids.WATER` — unreachable in vanilla
since no falling block has a waterlogged property.

### §F. Falling blocks — 13 checks, 13 pass

- `falling_block_fall` matches `FallingBlockEntity.fall` (entity at
  (x+0.5, y, z+0.5), carried state waterlogged cleared, block at pos set
  to the fluid's legacy block flag 3) — pass.
- `falling_block_tick` matches `FallingBlock.tick` (`isFree(below) && y >=
  minY` → fall + `falling(entity)`) — pass.
- `getDelayAfterPlace`: base 2 (`iconst_2`), `DragonEggBlock` 5
  (`iconst_5`), in both onPlace (`hooks.rs:162`, `on_place` dispatch) and
  updateShape (`shapes.rs:456`) — pass.
- `FallingBlockEntity.tick` structure: air → discard; time++; gravity;
  move; delta *= (0.7, −0.5, 0.7); cancelDrop → discard +
  `callOnBrokenAfterFall`, no drop; `canBeReplaced` via
  `DirectionalPlaceContext(DOWN, EMPTY, UP)`; timeout `(time > 100 && (y
  <= minY || y > maxY)) || time > 600` → drop if dropItem && ENTITY_DROPS,
  discard; air drag at end — pass.
- MOVING_PISTON at pos → jump to end (entity keeps falling, NOT discarded)
  — pass (see §Withdrawn).
- `causeFallDamage`: `!hurtEntities` → false; i = ceil(fallDistance − 1);
  i < 0 → false; damage source = `Fallable.getFallDamageSource` or
  `damageSources.fallingBlock`; amount = min(floor(i ×
  fallDamagePerDistance), fallDamageMax); hurt entities in AABB
  (NO_CREATIVE_OR_SPECTATOR ∧ LIVING_ENTITY_STILL_ALIVE); anvil (`#anvil`
  tag): amount > 0 && `nextFloat() < 0.05 + i*0.05` → `AnvilBlock.damage`;
  null → cancelDrop = true, else blockState = damaged. Rust
  `Damage::impact` (`horizon-client/src/falling_block.rs:69-87`) matches
  exactly — pass.
- `wear_anvil` matches `AnvilBlock.damage` (ANVIL→CHIPPED_ANVIL,
  CHIPPED→DAMAGED, keeping FACING; else null) — pass.
- Anvil methods: `falling` → `setHurtsEntities(2.0F, 40)`; `onLand` →
  levelEvent 1031 unless silent; `onBrokenAfterFall` → 1029 unless silent;
  `getFallDamageSource` → `DamageSources.anvil` — pass.
- Anvil detection by name `ends_with("anvil")` ≡ `#anvil` tag for vanilla
  (`#anvil` = anvil, chipped_anvil, damaged_anvil) — pass.
- FallingBlock event consumer (`game.rs:4554-4590`: hurt_per_distance →
  `Damage::falling` + hurt_entities/per_distance override, entity at
  (x+0.5, y, z+0.5), drop_item=true) — pass.
- cancelDrop handling in caller (discard + onBroken sound, no drop) —
  pass.
- `isFree`: isAir OR `#fire` tag OR `liquid()` OR `canBeReplaced()` — pass.
- `spawn_falling_stalactite` damage: tip →
  `setHurtsEntities(1.0F * max(1 + pos.y − mutable.y, 6), 40)`; Rust
  `falling.rs:51` `(1 + pos.y − fall_pos.y).max(6)` — pass. Sound events
  1045 (pointed dripstone landing) / 1052 (sulfur spike landing) / 1029 /
  1031 match vanilla.

### §G. Dripstone and speleothem — 26 checks, 26 pass

- `PointedDripstoneBlock.randomTick`: `maybeTransferFluid(state, level,
  pos, nextFloat())` then super.randomTick — pass.
- `isStalactiteStartPos`: `isStalactite(state) && !above.is(state.getBlock())`
  (SAME BLOCK, not tag) — pass.
- `growStalactiteOrStalagmiteIfPossible`: canGrow → findTip(max, false) →
  isFreeHangingStalactite && canTipGrow → nextBoolean ? grow(DOWN) :
  growStalagmiteBelow — pass.
- `canGrow`: block above is `blockToGrowOn.getBlock()`. The prepared
  `fields.blockToGrowOn.state` values resolve (datagen registry entries
  sorted by `protocol_id` — the file order is alphabetical, see §Method)
  to pointed_dripstone → `minecraft:dripstone_block` (state 30208) and
  sulfur_spike → `minecraft:sulfur` (state 24687), matching vanilla; the
  Rust resolves via `data.link` (`data.rs:963-978`, falls back to
  `field("blockToGrowOn").state`) — pass.
- PDB `canGrow` adds fluid at above(2) is WATER source (`is(Fluids.WATER)
  && isSource`); Rust via `FluidKind::Water` (source-only: FluidKind is a
  u8 newtype, Empty=0, FlowingWater=1, Water=2, FlowingLava=3, Lava=4 —
  `fluid.rs:278-282`) — pass.
- `getMaxGrowthLength`: base 7; `SulfurSpikeBlock` 2 (static init
  `iconst_2`) — pass.
- `findTip` / `findBlockVertical` (loop i=1..maxSteps: move; state; if
  target → return; if outside build height OR !path → empty) — pass.
- `canTipGrow`: fluid at rel(tip, dir) empty AND (isAir OR
  isUnmergedTipWithDirection(opposite)) — pass.
- `isUnmergedTipWithDirection`: isTip(state,false) && TIP_DIRECTION==dir
  && state.is(this block) — pass.
- `grow`: target = rel(from,dir); unmerged tip opposite → createMergedTips;
  else air or WATER block → createSpeleothem(target, dir, TIP) — pass.
- `createSpeleothem`: default state + direction + thickness + waterlogged
  = `getFluidState(pos).is(Fluids.WATER)` (SOURCE only, not tag);
  setBlock flag 3 — pass.
- `createMergedTips`: existing tip UP → stalactite=above(target),
  stalagmite=target; else stalactite=target, stalagmite=below(target);
  both TIP_MERGE — pass.
- `growStalagmiteBelow`: 10 iterations down; fluid non-empty → return;
  unmerged tip UP && canTipGrow → grow(UP), return;
  isValidSpeleothemPlacement(pos, UP) && !isWaterAt(below) → grow(below,
  UP), return; blocksStalagmiteScan → return — pass.
- `blocksStalagmiteScan`: base returns false; only PDB overrides to
  `!canDripThrough` — the sulfur spike does not override it — pass.
- `PDB.canDripThrough`: isAir → true; isSolidRender → false; fluid
  non-empty → false; else `!joinIsNotEmpty(REQUIRED_SPACE, collisionShape,
  AND)` where REQUIRED_SPACE = `Block.column(4.0, 0.0, 16.0)`. The Rust
  column (0.375,0,0.375)-(0.625,1,0.625) matches `Block.column(4,0,16)`
  exactly (`column(a,b,c)` = `column(a,a,b,c)` = box(8−a/2, b, 8−a/2,
  8+a/2, c, 8+a/2) in pixels) — pass.
- `dripstone_fluid_above`: above(root); mud && !WATER_EVAPORATES → WATER
  else fluid type at above — pass. Nuance: vanilla reads
  WATER_EVAPORATES per-pos via env attributes; the Rust uses
  `level.dimension_type().water_evaporates` (per-dimension) — equivalent
  in practice since it is a dimension-type attribute.
- `dripstone_transfer_fluid`: `f > 0.17578125 && f > 0.05859375` → return;
  `!isStalactiteStartPos` → return; `getFluidAboveStalactite` empty →
  return; WATER → 0.17578125, LAVA → 0.05859375, else return; `f >=
  chance` → return; `findTip(state, level, pos, 11, false)`; mud source +
  WATER → clay + `pushEntitiesUp` + BLOCK_CHANGE + levelEvent 1504 at
  tip, return; else `findFillableCauldronBelowStalactiteTip` → levelEvent
  1504 at tip, delay = 50 + tip.y − cauldron.y, scheduleTick(cauldron
  block, delay) — pass. Nuance: the Rust emits the `push_entities_up`
  event BEFORE setting clay; vanilla sets clay then calls
  `pushEntitiesUp` (mud→clay shapes are both full blocks, so moot in
  practice).
- `cauldron_can_receive` (see G6) — pass.
- updateShape (`shapes.rs:729-749`): waterlogged → schedule water fluid
  tick (delay 5); non-vertical dir → return state; tip==DOWN &&
  hasScheduledTick → return state; dir == tip.opposite() && !canSurvive →
  schedule block tick (2 if DOWN else 1), return state; else thickness =
  `calculateSpeleothemThickness(level, pos, tip, thickness==TIP_MERGE)` —
  pass.
- `calculateSpeleothemThickness` (`shapes.rs:1179`): front = rel(pos,
  tip); if isSpeleothemWithDirection(front, opposite) && front.is(this):
  (flag || front thickness==TIP_MERGE) ? TIP_MERGE : TIP; if
  !isSpeleothemWithDirection(front, tip) → TIP; if front thickness
  TIP/TIP_MERGE → FRUSTUM; behind = rel(pos, opposite);
  !isSpeleothemWithDirection(behind, tip) → BASE else MIDDLE.
  `isSpeleothemWithDirection` = in `#speleothems` tag &&
  TIP_DIRECTION==dir (`#speleothems` EXISTS in 26.2:
  `[pointed_dripstone, sulfur_spike]`) — pass.
- `isValidSpeleothemPlacement`: support = rel(pos, dir.opposite());
  `state.isFaceSturdy(level, support, dir)` OR (isSpeleothemWithDirection
  (state, dir) && state.is(this)). `canSurvive` =
  isValidSpeleothemPlacement(pos, TIP_DIRECTION). Rust `survive.rs:548`
  (tag check implied by the same-block check) — pass. Nuance: the Rust's
  `is_face_sturdy(state, dir)` is position-free; vanilla's
  `isFaceSturdy(level, pos, dir)` is position-aware — a method limitation,
  not a defect.
- `getStateForPlacement` (`place.rs:1155`): direction = opposite of
  nearest looking vertical; `calculateTipDirection` (valid(dir) ? dir :
  valid(opposite) ? opposite : null → return null); thickness =
  calculateSpeleothemThickness(..., !isSecondaryUseActive); waterlogged =
  fluid is(Fluids.WATER) source-only (`place.rs:279,347-348`,
  `fluid_is_water_source`) — pass.
- `PDB.fallOn`: tip direction UP && thickness TIP →
  `entity.causeFallDamage(fallDistance + 2.5, 2.0F, stalagmite)`; else
  super.fallOn. Rust `block_landing.rs:208-223,429-437` (direction up &&
  thickness tip → Stalagmite, distance+2.5, ×2.0) — pass.
- `SpeleothemBlock.onProjectileHit`: server-side; mayInteract &&
  mayBreak && projectile is ThrownTrident && |deltaMovement| > 0.6 →
  destroyBlock(pos, true). Rust `game/creatures.rs:5013-5095` (trident +
  hit_speed > 0.6 → destroy with drops; gates on impact tag + mayInteract
  + `projectiles_can_break_blocks`) — pass.
- B11-G06 class name (see §Closed) — pass.
- Speleothem tick dispatch: scheduled tick `tick.rs:248` (tip==Up &&
  !canSurvive → destroy_block(pos, true, 512) else spawn falling);
  random-tick dispatch `tick.rs:434`; `SpeleothemBlock.tick`:
  stalagmite && !canSurvive → destroyBlock(pos, true), else
  `spawnFallingStalactite` — pass.

### §H. Cauldrons — 8 checks, 8 pass

- `AbstractCauldronBlock.tick`: `findStalactiteTipAboveCauldron` →
  `getCauldronFillFluidType` → if != EMPTY && `canReceiveStalactiteDrip`
  → `receiveStalactiteDrip`. Rust `cauldron_tick` matches; dispatch
  `tick.rs:64` AbstractCauldron → cauldron_tick — pass.
- `CauldronBlock.canReceive` → true; receive: WATER → water_cauldron
  default + BLOCK_CHANGE + levelEvent 1047; LAVA → lava_cauldron + 1046
  — pass.
- `LayeredCauldronBlock.canReceive`: fluid == WATER && precipitationType
  == RAIN; receive: if `isFull` (level == 3) return; level+1,
  BLOCK_CHANGE, 1047 — pass.
- `LavaCauldronBlock` does NOT override `canReceiveStalactiteDrip` (base
  false) — pass.
- AbstractCauldron tick dispatch (`tick.rs:64`) — pass.
- `LayeredCauldronBlock.entityInside`: server-side:
  `runBefore(EXTINGUISH, lambda)` then `apply(EXTINGUISH)`; lambda: if
  entity.isOnFire() && entity.mayInteract(level, pos) →
  `handleEntityOnFireInside`. Rust `layered_cauldron_inside`
  (`motion.rs:2028-2036`): fire_ticks>0 && !spectator → push
  `MotionEvent::CauldronExtinguish`; fire_ticks = min(0) — pass, except
  the mayInteract gate (nuance, §Withdrawn).
- `handleEntityOnFireInside` + `lowerFillLevel`: precipitationType==SNOW
  → convert to WATER_CAULDRON default with same level, then
  `lowerFillLevel`; else `lowerFillLevel(state)`; level−1; 0 → empty
  cauldron default; else same block level−1; setBlockAndUpdate +
  BLOCK_CHANGE game event. Rust `game.cauldron_extinguish`
  (`game.rs:8041-8095`) + `cauldron::extinguished`
  (`cauldron.rs:112-120`: powder-snow → water cauldron then lowered;
  `lowered`: level ≤ 1 → empty cauldron, else level−1) match exactly,
  incl. the BLOCK_CHANGE game event and a
  `native_cauldron_level_change_allowed` gate — pass. The event is
  consumed for LAN guests at `lan_actions.rs:5287`; the host (window)
  player is processed as a guest for server ticks
  (`game.rs:5499` tick comment: "its players are guests, swapped in for
  their turns (`as_guest`), and the window's player is a client of it"),
  so the same handler covers it.
- CauldronExtinguish emit conditions (burning, !spectator) — pass.

### §I. Splash water — 6 checks, 6 pass

- `onHitBlock` gate = `PotionContents.is(Potions.WATER)`, NOT a tag (see
  B11-G13) — pass.
- Dowse targets: hitPos.relative(dir), hitPos itself, and the 4 horizontal
  neighbours of hitPos.relative(dir) — pass.
- `dowseFire`: `#fire` tag → destroyBlock(pos, false, this); else lit
  candle → `AbstractCandleBlock.extinguish(null, ...)`; else lit campfire
  → levelEvent 1009 + `CampfireBlock.dowse(owner, ...)` +
  setBlockAndUpdate(lit=false) — pass. (Splash potions live in
  `net.minecraft.world.entity.projectile.throwableitemprojectile` in 26.2,
  NOT `...projectile`.)
- `onHitAsWater`: box inflate (4,2,4), WATER_SENSITIVE_OR_ON_FIRE
  predicate (isSensitiveToWater || isOnFire), dist² < 16;
  water-sensitive → hurt 1.0 indirectMagic (owner); on-fire + alive →
  extinguishFire; axolotls in box → rehydrate. `onHit`: server-only;
  WATER → onHitAsWater; else if hasEffects → onHitAsPotion; level event
  2007 if the potion has instant effects else 2002; discard — pass.
- `Axolotl.rehydrate`: air = min(air + 1800, maxAir) — pass.
- Potion tags in 26.2's data: ONLY `tradeable.json`; `douses_fire` does
  not exist; `PotionTags` has only `TRADEABLE` — pass.

### Tick dispatch callers — verified (folded into the sections above)

`tick.rs:64` AbstractCauldron → cauldron_tick; `:66` BubbleColumn tick;
`:184` falling_block_tick; `:231` Liquid tick; `:248` Speleothem scheduled
tick; `:434` Speleothem random tick. Lava random tick is dispatched TWICE
for a lava block (`tick.rs:298-301` fluid-kind check after block random
tick + `tick.rs:394-399` Class::Liquid in random_tick_block), matching
vanilla (LiquidBlock.randomTick → fluid randomTick, plus the chunk fluid
randomTick).

### Prepared-data class/interface spot checks

`block-behaviours.json` per-block `class` chain + `interfaces` verified
for in-scope blocks: concrete powder / anvil / dragon egg → FallingBlock +
Fallable; sand → SandBlock→ColoredFallingBlock→FallingBlock; cauldron →
CauldronBlock→AbstractCauldronBlock; water/lava cauldron →
LayeredCauldronBlock/LavaCauldronBlock→AbstractCauldronBlock; water/lava
→ LiquidBlock + BucketPickup; bubble_column → BubbleColumnBlock +
BucketPickup (NOT LiquidBlockContainer — consistent with vanilla's
explicit exclusion); fire → FireBlock→BaseFireBlock; oak_sign →
StandingSignBlock→SignBlock + SimpleWaterloggedBlock; kelp/seagrass →
LiquidBlockContainer direct. `fire` section has shapes/igniteOdds/burnOdds
/fireDamage (fireDamage 1.0). The prepared `classes/` directory is EMPTY
(0 files) — chains live in `block-behaviours.json`.

## Oracle defects in scope

**Coordinator reconciliation (2026-11-24):** the batch agent ran before
Batch 0 landed and recorded no oracle defects ("measurement has not run").
The measurement now exists (`docs/sweep/26.2/batch-0-oracles.json`, 391
oracles, 2026-11-24), and it changes the picture: **7 of the 13 in-scope
oracles fail on all three classpaths**, on five distinct 26.3-only API
references (each verified by the coordinator with `javap` against the
26.2 jar — see below). The agent's static spot-checks
(`net.minecraft.util.debug` and `vehicle.boat` exist in the 26.2 jar) are
correct but do not bear on the actual failures, which are all
method-level, not import-level. The 13 in-scope oracle files are present
in the 26.2 tree
(`versions/26.2/mods/horizon/src/test/java/dev/horizonlab/parity/`):
VanillaCauldronOracle, VanillaEntityFluidOracle, VanillaFallingBlockOracle,
VanillaFluidBoxOracle, VanillaFluidFlowOracle, VanillaFluidOccupancyOracle,
VanillaFluidOracle, VanillaFluidShapeOracle, VanillaLavaCauldronItemOracle,
VanillaLivingFluidOracle, VanillaRegisteredEntityFluidOracle,
VanillaRegistryFluidOracle, VanillaWaterSplashOracle.
(VanillaValueProviderOracle also matched the filename grep but is not
fluid/falling scope — excluded.)

### B1-O01 — `BlockTags.BLOCKS_FLUID_FLOW` is 26.3-only; breaks 3 oracles (MINOR)

`VanillaFluidFlowOracle` (1 error, line 15),
`VanillaRegisteredEntityFluidOracle` (2 errors, line 9) and
`VanillaRegistryFluidOracle` (3 errors, line 11) all fail on the same
missing symbol. `javap` on the 26.2 server-game jar: `BlockTags` has no
`BLOCKS_FLUID_FLOW` field — it is a 26.3-only tag (this batch's own tag
inventory confirms `blocks_fluid_flow` does not exist in 26.2's data).
The port deliberately maps `blocks_fluid_flow := blocks_motion` for 26.2,
so the repair is to bind the tag from the blocks_motion table, not to add
the tag.

### B1-O02 — `VanillaCauldronOracle` BROKEN, 1 compile error (MINOR)

Fails on all three classpaths, missing-symbol at line 118: the `drop()`
override signature references `net.minecraft.util.Prediction`, which does
not exist in the 26.2 jar (jar listing confirms; Batch 11 §6.5
independently lists `Prediction` as a 26.3-only class). The 26.2
`Player.drop` signature differs.

### B1-O03 — `VanillaFallingBlockOracle` BROKEN, 1 compile error (MINOR)

Fails on all three classpaths, constructor-mismatch at line 30:
`NoopChunkMap` calls `super()` with 12 arguments (10 nulls + 0 + false),
but 26.2's `ChunkMap` constructor takes 13 parameters (`javap`-verified:
ServerLevel, LevelStorageAccess, DataFixer, StructureTemplateManager,
Executor, BlockableEventLoop, LightChunkGetter, ChunkGenerator,
ChunkStatusUpdateListener, Supplier, TicketStorage, int, boolean).
Batch 0 group G5.

### B1-O04 — `VanillaFluidOracle` BROKEN, 1 compile error (MINOR)

Fails on all three classpaths, missing-symbol at line 50: the line calls
`state.getHeightForCamera(world, p)`. `javap`-verified: 26.2's `FluidState`
has only `getHeight(BlockGetter, BlockPos)`; `getHeightForCamera` is a
26.3-only method.

### B1-O05 — `VanillaLivingFluidOracle` BROKEN, 3 compile errors (MINOR)

Fails on all three classpaths, missing-symbol at line 50: the line
references `FluidTags.ENTITY_FLOATABLE`. `javap`-verified: 26.2's
`FluidTags` has no `ENTITY_FLOATABLE` field (26.2 has only 6 fluid tags,
per this batch's own inventory); it is a 26.3-only tag.

### Oracles assessed PASSING for 26.2 (Batch 0 measurement)

- `VanillaEntityFluidOracle` — passes all three classpaths.
- `VanillaFluidBoxOracle` — passes all three classpaths (the 1.21.7
  `vehicle.boat` cause indeed does not apply).
- `VanillaFluidOccupancyOracle` — passes all three classpaths.
- `VanillaFluidShapeOracle` — passes all three classpaths.
- `VanillaLavaCauldronItemOracle` — passes all three classpaths.
- `VanillaWaterSplashOracle` — passes all three classpaths.

For reference, the 1.21.7 statuses were: VanillaFluidShapeOracle PASS,
VanillaWaterSplashOracle PASS, the other eleven FAIL with various causes
(missing-symbol, missing-package, incompatible-types).

## Method notes

- **Prepare landed during the run.** The coordinator's update (~09:11 NPT
  2026-11-24) reported horizon-prepare for 26.2 finished (2048.7 s, EXIT
  0); prepared data at `versions/26.2/.local/horizon-client/26.2/` (203
  MB) became available part-way through this batch. Two checks that would
  otherwise have been needs-harness were unblocked by it and are counted
  as passes: the waterlogging exhaustive set (§A, 429/429) and the fire
  odds values (§C, 207/207). No batch scope was expanded beyond what this
  unblocked. `sounds.json` and lang files are still absent (the VM lacks
  the official launcher's asset index 32.json) — expected, not a port
  defect; not treated as a finding.
- **Vanilla ground truth is 26.2's own jar/data only** (owner's
  vanilla-check rule). Every vanilla claim above cites the 26.2
  server-game jar's bytecode or the 26.2 client jar's data. Where 26.2's
  form differs from 26.3's (no `washed_away_by_fluids`, no
  `blocks_lava_fire_spread`, no `douses_fire`, no `doFireTick`), the port
  was checked against the 26.2 form.
- **Tag inventory verified in 26.2's own data** (client jar + server-game
  jar data + `BlockTags`/`FluidTags`/`PotionTags` javap): `speleothems`
  EXISTS `[pointed_dripstone, sulfur_spike]`; `washed_away_by_fluids`,
  `blocks_lava_fire_spread`, `blocks_fluid_flow` do NOT exist;
  `enables_bubble_column_push_up` / `enables_bubble_column_drag_down`
  EXIST; fluid tags total six (bubble_column_can_occupy, lava,
  supports_frogspawn, supports_lily_pad, supports_sugar_cane_adjacently,
  water); potion tags: only `tradeable`.
- **Datagen registry order.** `registries.json` `minecraft:block` entries
  are ALPHABETICAL, not id order — cumulative state counting over file
  order gives wrong answers (it initially resolved the dripstone
  `blockToGrowOn` ids to redstone_wire / warped_trapdoor). Sort by each
  entry's `protocol_id` first; total states 32,366 confirms the count.
  (Those wrong resolutions are NOT a finding — they were a method error,
  corrected.)
- **Prepared `block-tags.json` values are raw JSON strings** — parse them,
  then resolve `#` references. Prepared `states` list entries are opaque
  positional arrays, NOT directly usable as a state-id→block map.
  Prepared `block-behaviours.json` class chains do NOT expand interface
  inheritance (see G5).
- **Nuances recorded, not defects:** (1) Rust `lava_flammable` returns
  false only if inside build height AND not loaded, then checks the flag;
  vanilla returns false whenever NOT (inside && hasChunkAt) — below/above
  build height the Rust proceeds to the flag check (state lookup there is
  presumably air → false anyway). (2) Mud→clay push-entities event order
  (§G). (3) `water_evaporates` per-dimension vs per-pos (§G).
  (4) `is_face_sturdy` position-free (§G). (5) Waterlogged-on-landing tag
  vs source, unreachable (§E). (6) Cauldron mayInteract gate
  (§Withdrawn). (7) Rust `lava_flammable` — see (1).
- **Untraced sub-details (NOT counted as passes, NOT needs-harness — they
  are sub-details of passing checks or out of scope):** the lava-hurt
  GENERIC_BURN sound (vol 0.4, pitch 2+rand*0.4) was not traced in the
  Rust; `PointedDripstoneBlock.animateTick` drip particles were not
  compared; dragon-egg teleport on use is out of scope (falling-block
  scope covers fall/landing/conversion).
- **No cargo builds, no unit tests, no oracle measurement** were run in
  this batch (brief constraints + the shared 6 GB VM). All verdicts are
  javap + source + data comparisons.
- **Intermediate evidence.** `javap -c` dumps were kept under
  `/tmp/b1javap/` during the run; that directory was lost to a /tmp
  cleanup between turns and the AbstractBoat dump was re-made afterwards.
  All facts cited above were recorded in the working notes before the
  loss; nothing in this report depends on a dump that no longer exists.
- **Check-count hygiene.** The 110 checks count only comparisons where
  both sides were actually read. Overlapping sub-checks were not
  double-counted (e.g. cauldron receive vs cauldron_tick are separate
  checks only where the vanilla methods are separate); untraced items
  were not counted as passes; the two gap checks are the two halves of
  one defect (B1-G02).

## Ledger

`batch-01.json` in this directory — 1 item (B1-G02), checks
110/108/2.
