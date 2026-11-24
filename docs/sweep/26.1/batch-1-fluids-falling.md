# Batch 1 — Fluids + falling blocks (26.1 parity sweep)

**Version:** 26.1 · **Date:** 2026-11-24 · **Branch:** `parity-sweep-26.1`
(workdir `~/workspace/mcir-work/sweep-26.1`) ·
**Scope:** `docs/PARITY-SWEEP.md` §4 Batch 1 — the pilot scope re-run against
26.1: water/lava spreading and flow, fluid pushing, bubble columns, concrete
powder solidification, lava fire spread + ignition + game rules, dripstone
growth + cauldron fill, falling blocks, cauldron fill levels, splash water.

**Status: DRAFT — not committed.** Checks + triage only, per the brief: no
fixes, no edits under `versions/26.1/`, no cargo builds, no oracle
measurement of my own, no commits. Everything below is static analysis:
`javap -c` on the named 26.1 server-game jar
(`~/workspace/mcir-work/26.1.x/versions/26.1/jars/server-game.jar`,
DataVersion 4786, protocol 775, Java 25), source comparison against
`versions/26.1/crates/`, tag/data reads from the named 26.1 client jar and
the 26.1 datagen reports, and the prepared data at
`versions/26.1/.local/horizon-client/26.1/` (GATE OK — landed before this
batch). Oracle triage is from the measured Batch 0 table only
(`docs/sweep/26.1/batch-0-oracles.json`: 199/391 oracles compile, 192 fail).
The coordinator commits these deliverables to
`docs/sweep/26.1/batch-1-fluids-falling.md` and
`pins/26.1/sweep/batch-01.json` (or their chosen paths).

## Tally

- **Behaviour checks: 93 — 78 pass, 15 gap** (the 15 gap checks are the
  four defect items below: B1-G01 × 8, B1-G02 × 2, B1-G03 × 4, B1-G04 × 1;
  see §Defects).
- **Defect items: 4** — B1-G01 (major), B1-G02 (minor), B1-G03 (major),
  B1-G04 (minor).
- **Oracle defects: 7** (B1-O01–B1-O07, all minor — see §Oracle defects in
  scope: 9 of the 13 in-scope oracles fail on all three classpaths). Not
  counted in the check tally, following the 1.21.7 convention.
- **Needs-harness: none** — no B1-Hxx items. Every in-scope check was
  decidable statically; the few untraced sub-details are noted in §Method,
  not faked as passes.
- **Withdrawn candidates:** one (the cauldron mayInteract gate — a nuance,
  not a defect; see §Withdrawn). No defect number was withdrawn: B1-G01–G04
  are all attached to defects.
- **1.21.7 re-checks (§Closed):** G1, G3, G5 CLOSED; G2, G6, B11-G06,
  B11-G13 REPRODUCE (as B1-G02, B1-G01, B1-G01, B1-G03 respectively); G4
  NOT APPLICABLE as framed (no `doFireTick` rule exists in 26.1).

## Defects

### B1-G01 — Entire pointed-dripstone system dead (major, REPRODUCES pilot G6 + B11-G06)

**Checks failed (8):** random-tick transfer; random-tick growth; cauldron
fill from dripstone; placement (tip direction/thickness); updateShape
(thickness recalc + survival scheduling); canSurvive (floating dripstone
never breaks); scheduled tick (unsupported stalagmites / falling
stalactites); falling stalactite spawn.

**Vanilla 26.1.** There is NO `SpeleothemBlock` class in the 26.1 jar at
all: `PointedDripstoneBlock extends Block` directly (implements
`SimpleWaterloggedBlock`, `Fallable`) — the 1.21.7 shape, not 26.2's.
Its bytecode (1716-line `javap -c` dump read in full) contains ZERO
`BlockTags` references: every predicate is same-block
(`isPointedDripstoneWithDirection` = is(POINTED_DRIPSTONE) &&
TIP_DIRECTION==dir; `isTip`, `isStalactiteStartPos`,
`isValidPointedDripstonePlacement`, `calculateDripstoneThickness` all
same-block). `PDB.canGrow` hardcodes `Blocks.DRIPSTONE_BLOCK` + a water
source at above(2) — no `blockToGrowOn` field exists in 26.1. Constants:
growth 0.011377778, transfer 0.17578125 water / 0.05859375 lava, findTip
max 11, mud→clay + pushEntitiesUp + BLOCK_CHANGE + level event 1504,
cauldron delay 50 + tip.y − cauldron.y, tick delays 2/1, trident speed
0.6, fallOn +2.5 / ×2.0 stalagmite. `canDrip` = isStalactite && TIP &&
!waterlogged. `randomTick` draws nextFloat for transfer first, then for
growth. `maybeTransferFluid`'s double-threshold early return is equivalent
to a single check (0.17578125 > 0.05859375).

**The port is dead three independent ways:**

1. **Dead dispatch.** `Class::Speleothem = "SpeleothemBlock"`
   (`level/classes.rs:215`) gates five dispatch sites — `place.rs:1156`
   (placement), `shapes.rs:728` (updateShape/thickness), `survive.rs:307`
   (canSurvive), `tick.rs:248` (scheduled tick), `tick.rs:434` (random
   tick: growth + transfer). Dispatch iterates the block's exported class
   chain; pointed_dripstone's prepared chain is
   `[PointedDripstoneBlock, Block]` — the Speleothem arm never fires.
   `speleothem.rs` itself uses `Class::PointedDripstone` (lines 86, 103,
   206) but is unreachable via dispatch.
2. **Dead tag.** `speleothem_tag()` reads `#minecraft:speleothems` — absent
   from 26.1 data (no `speleothems.json` in the client jar, no
   `SPELEOTHEMS` field in `BlockTags`, no key in prepared
   `block-tags.json`, 248 keys) — so `speleothem_with`,
   `speleothem_is_tip`, `is_free_hanging_stalactite`,
   `speleothem_unmerged_tip` are all always false, and
   `falling.rs:55` `spawn_falling_stalactite` is tag-gated too.
3. **Missing link.** Prepared `fields` for pointed_dripstone is `{}` — no
   `blockToGrowOn`; `data.link` resolves to None (`level/data.rs:959-974`),
   so `speleothem_grow`'s `can_grow` is always false even if reached.

**Observable consequences** (each a §G gap check): random-tick transfer
never runs; growth never runs; cauldron fill never runs (`cauldron_tick`
needs `is_free_hanging_stalactite`); placement falls to default (no tip
direction/thickness calc); updateShape thickness recalc + survival
scheduling never run; `can_survive` falls through to `true`
(`survive.rs:20-27` — no arm → returns true, so floating dripstone never
breaks); scheduled tick never destroys unsupported stalagmites / spawns
falling stalactites; falling stalactite spawn tag-dead.

**NOT dead (name/data-keyed, verified working):** `fallOn` stalagmite
damage — `block_landing.rs` `FallDefinition::PointedDripstone` keyed via
`landing-definitions.json` (`pointed_dripstone →
"PointedDripstoneBlock"`), Rule::DripstoneTip (up + tip → Stalagmite,
distance+2.5, ×2.0) matches vanilla `PDB.fallOn`. Trident break —
`creatures.rs:5081-5084` name match `"minecraft:pointed_dripstone"` +
Trident + hit_speed > 0.6 → destroy, with the `projectile_breaks_block`
gates (impact kind + shooter_may_break + `projectiles_can_break_blocks`
rule, `creatures.rs:5037-5067`) matching vanilla `onProjectileHit`
(server-side, mayInteract/mayBreak gates).

**Severity rationale:** the whole dripstone feature set is dead —
comparable in scope to pilot G1 (bubble columns never form, major), not
just the narrow G6 cauldron-fill framing (minor in the pilot). The
dead-code constants in `speleothem.rs` match vanilla exactly, but they
were NOT counted as passes (§G counts observable behaviours only).

### B1-G02 — Concrete powder falling into water does not solidify on contact (minor, CARRIES OVER from pilot G2)

**Checks failed (2):** first-water-cell landing check; fast-fall clip check.

**Vanilla 26.1.** `FallingBlockEntity.tick` (server-game jar): a falling
concrete-powder entity computes `inWater = block is ConcretePowderBlock
&& fluid at blockPosition is in the WATER tag`; if the delta's
length-squared is non-zero it additionally clips from (xo,yo,zo) to the
position with `ClipContext.Block.COLLIDER` + `ClipContext.Fluid.SOURCE_ONLY`
— a non-MISS hit whose fluid is WATER sets the landing pos to the hit pos
and `inWater = true`. The landing branch runs when `onGround() ||
inWater`. `ConcretePowderBlock.shouldSolidify` = `canSolidify(state)` (fluid
is WATER) OR `touchesLiquid` (the mutable-position quirk: DOWN handled
specially, `isFaceSturdy(opposite)` check); `onLand` replaces with the
concrete block.

**The port.** `game/falling_blocks.rs` `fall_step` has NO concrete-powder
water check (landing only on `on_ground`); `game/falling_damage.rs`
`finish_falling_block` calls `falling_block_landed(..., false)` —
`stuck_in_water=false` always. So powder sinks to the lakebed and lands
there as powder. The horizon-world landing branch itself
(`level/falling.rs` `falling_block_landed`) matches vanilla, including
the `onLand` solidify check (`falling.rs:127`: water fluid at current OR
`concrete_touches_liquid`, via the data link) and the
`would_continue = isFree(below) && (!concrete || !stuck_in_water)` logic —
the fix is local to the client fall step. Same defect as 26.2's B1-G02.
Note: `finish_falling_block` also has a dead `"minecraft:sulfur_spike" =>
Some(1052)` arm (harmless — no sulfur_spike in 26.1; the 1052 sound event
does not exist for this version).

### B1-G03 — Splash water never douses fire / hurts / extinguishes / rehydrates (major, REPRODUCES B11-G13)

**Checks failed (4):** dowse fire on block hit; hurt water-sensitive
entities; extinguish burning entities; rehydrate axolotls.

**Vanilla 26.1.** `AbstractThrownPotion.onHitBlock` gates on
`PotionContents.is(Potions.WATER)` — Holder equality, NOT a tag. Dowse
targets: hitPos.relative(dir), hitPos itself, and the 4 horizontal
neighbours of hitPos.relative(dir). `dowseFire`: `#fire` tag →
destroyBlock(pos, false, this); else lit candle → extinguish; else lit
campfire → level event 1009 + `CampfireBlock.dowse`. `onHitAsWater`:
LivingEntity in the potion's box inflated (4.0, 2.0, 4.0) with
WATER_SENSITIVE_OR_ON_FIRE (isSensitiveToWater || isOnFire),
distanceToSqr < 16.0; water-sensitive → hurtServer(indirectMagic(owner,
this), 1.0); on fire && alive → extinguishFire; Axolotl in the same box →
`rehydrate()` (air = min(air + 1800, maxAir), javap-verified).

**The port.** 26.1's `game/splash_water.rs` ships the 26.3 tag form:
`potion_in_tag(stack, "minecraft:douses_fire")`,
`"minecraft:hurts_water_sensitive_entities"`,
`"minecraft:extinguishes_entities"`,
`"minecraft:rehydrates_axolotls"` — read from `trades.potion_tags`, built
from the prepared data's `tags/potion` directory (`merchant.rs:488-489`,
574). That directory contains ONLY `tradeable.json` in 26.1 (verified in
the client jar and in prepared data) — none of the four tags exists, so
all four gates are always closed and no splash water ever does anything.
26.2's file uses `is_water_potion` (Potions.WATER equality) and works.
The downstream code itself matches vanilla exactly (dowse targets,
dowseFire, the (4,2,4) box, dist²<16, hurt 1.0 indirectMagic, Enderman
teleport counter, axolotl +1800 air capped at MAX_AIR) — it is verified
matching but unreachable, so it is recorded as analysis, NOT counted as
passes (§I counts the four observable outcomes).

**Severity rationale:** all splash-water functionality is dead — four
distinct observable behaviours, a whole mechanic, same scope class as
B1-G01. (26.1 also uses "douse" naming vs 26.2 "dowse" — cosmetic only.)

### B1-G04 — Entity fluid interaction ships the 26.3 form; eye check uses camera height (minor)

**Checks failed (1):** eye-in-fluid height check.

**Vanilla 26.1.** `EntityFluidInteraction` is the 26.2 form
(javap-verified): `trackerByFluid` Map<TagKey, Tracker>; `Tracker
{height, eyesInside, accumulatedCurrent, currentCount}` with the current
ON the tracker; NO `CurrentAccumulator` class; `update` returns void;
`getTrackerFor(Fluid)` = first tag holding the fluid. `update` bytecode
read in full: reset all trackers; box null → return; floor/ceil bounds;
`hasFluidAndLoaded(minX−1, minY, minZ−1, maxX+1, maxY, maxZ+1)` else
return; per cell (x outer, y middle, z inner): fluid non-empty; top = y +
**plain** `FluidState.getHeight`; top < box.minY skip; tracker =
getTrackerFor (cached by last kind); null tracker → skip cell entirely;
eyes: x==blockX && z==blockZ && eyeY in [y, top] → eyesInside; height =
max(top − bbox.minY, height); if !skipPush: flow = getFlow; if
tracker.height < 0.4 → flow.scale(tracker.height) (the TAG tracker's
height); accumulate.

**The port.** 26.1's `horizon-world/src/entity_fluid.rs` (247 diff lines
vs 26.2) ships the 26.3 form: per-kind trackers + separate
CurrentAccumulators, `update` returns `Result<bool>`, tracker pruning.
Most of that is behaviourally equivalent for vanilla fluids (see §D) —
but the eye check is not: `entity_fluid.rs:389-398` computes the top with
`world.camera_height(state, p)` (`fluid.rs:504` `camera_height_at`: a
source fluid with a sturdy block above → 1.0, else `height_at`) instead
of the plain top. Observable: an entity whose eye is in (0.889, 1.0]
above a source's block bottom with a solid ceiling → the port says
eyes-in-fluid, vanilla 26.1 says not. Affects `was_eye_in_water` /
`is_under_water` (swimming, overlay, air/drowning).

**Verified NOT divergences** (analysis, not gaps): flow scaling — the
Rust scales by current height = max over kind-trackers fed = the same
tag-level max vanilla uses (same value, same order);
`fluid_height`/`is_in_fluid`/`is_eye_in_fluid` — same values; tracker
pruning — internal only; no-tag kinds get trackers in the Rust but
contribute nothing to any tag (mod fluids only). The `update` bool return
(`last.is_some()` = any fluid found, vs vanilla inWater||inLava) differs
only in a 1mm band (fluid surface within 0.001 below the entity's feet:
the interaction box is deflated 0.001) or for untagged mod fluids — and
ALL callers discard the bool (`motion/fluids.rs:312,343` `?`,
`pose.rs:213`, `landing.rs:427` via `living_landing.rs:34` `?`,
`arrows.rs:855` `let _ =`, `entities/fluids.rs` checks only `.is_err()`).
Recorded as nuance, not a gap.

## Closed — pilot defects re-checked against 26.1

All six 1.21.7 pilot defects and both Batch 11 findings were re-checked
against 26.1's OWN jar and data. Three pilot defects are CLOSED (verified
matching); G2, G6 and both Batch 11 findings REPRODUCE; G4 is NOT
APPLICABLE as framed. The pilot's systemic root cause — 26.x tag-based
logic for tags absent in the target version's data — DOES apply to 26.1
again, in two places: the speleothem system (B1-G01: `#speleothems` and
`SpeleothemBlock` both absent) and splash water (B1-G03: the four potion
tags absent). Where 26.1's data HAS the tag, the port reads it; where
26.1 still lacks the tag, the port carries the hardcoded 1.21.7-style
list form — except in those two places, where it ships the 26.3 form.

- **G1 bubble columns — CLOSED.** All three tags EXIST in 26.1's data:
  `enables_bubble_column_push_up` (`soul_sand`),
  `enables_bubble_column_drag_down` (`magma_block`), fluid tag
  `bubble_column_can_occupy` (`water`). The logic matches vanilla
  (`level/shapes.rs` `bubble_column_can_occupy` 1127,
  `should_bubble_column_occupy` 1136, `try_schedule_bubble_column` 1143
  with delay 20 ✓; `level/tick.rs` `bubble_column_update` 66 matches
  `updateColumn`; updateShape `shapes.rs:270` tick 5; `canSurvive`
  `level/survive.rs`). Entity motion matches exactly
  (`motion/fluids.rs` `BubbleColumn::push`: surface drag max(-0.9, vy−0.03),
  surface push min(1.8, vy+0.1), inside drag max(-0.3, vy−0.03), inside push
  min(0.7, vy+0.06); push returns !surface = reset fall distance). §B
  10/10 pass.
- **G2 concrete powder — REPRODUCES as B1-G02** (see §Defects).
- **G3 lava fire spread through solids — CLOSED.** Vanilla 26.1
  `LavaFluid.randomTick` bytecode read in full: matches the 26.2 form
  exactly — `canSpreadFireAround` gate first, `nextInt(3)` passes, first
  branch cumulative offset(nextInt(3)-1, 1, nextInt(3)-1), isLoaded else
  return, air + hasFlammableNeighbours →
  setBlockAndUpdate(BaseFireBlock.getState(level, test)) + return,
  `blocksMotion` → return; else branch 3 iters offset(dx,0,dz) from
  origin, isEmptyBlock(above) && isFlammable(test) →
  setBlockAndUpdate(above(test), getState(level, test)). `isFlammable`:
  inside build height AND NOT hasChunkAt → false, else `ignitedByLava()`.
  `entityInside`: CLEAR_FREEZE + LAVA_IGNITE + runAfter(LAVA_IGNITE,
  lambda). Rust `lava_random_tick` identical to 26.2's verified code;
  `blocks_lava_fire_spread` absent from 26.1 data. Rust `lava_flammable`
  nuance from 26.2 carries (below/above build height the Rust proceeds
  to the flag check where vanilla returns false — presumably air → false
  anyway; recorded as nuance, §Method).
- **G4 lava ignition ignores doFireTick — NOT APPLICABLE as framed.**
  Vanilla 26.1 has NO `doFireTick` rule: `GameRules` has exactly 59
  `public static final GameRule` fields (counted from the javap dump —
  independently confirming Batch 5's finding), none of them `doFireTick`.
  What exists instead is `FIRE_SPREAD_RADIUS_AROUND_PLAYER`
  (`registerInteger(..., 128, -1)` — default 128, min −1), and the port
  implements it: `level/mod.rs:219-220,234` rule field default 128;
  `game.rs:1562-1574` `can_spread_fire_around` (radius == -1 → true, else
  any fire_player within radius); `game.rs:3845` `fire_players` excludes
  spectators (local player filtered by `!spectator`, LAN guests by
  `here(g) && !g.spectator()`), matching vanilla
  `ServerLevel.canSpreadFireAround` (rule == -1 → true, else
  `ChunkMap.anyPlayerCloseEnoughTo`). §C 16/16 pass.
- **G5 no fluid spreading — CLOSED.** Vanilla 26.1
  `FlowingFluid.canHoldAnyFluid` bytecode read: hardcoded list —
  LiquidBlockContainer → true; blocksMotion → false; excludes DoorBlock,
  `#signs`, ladder, sugar cane, bubble column, nether/end portals, end
  gateway, structure void; else true. `washed_away_by_fluids` absent from
  26.1 data. Rust `can_hold_any_fluid` (`level/fluids.rs:359-376`,
  comment-only diff vs 26.2) matches; `is_liquid_container` = LBC OR
  SimpleWaterlogged (required — the prepared export doesn't expand
  interface inheritance; the jar confirms `SimpleWaterloggedBlock extends
  BucketPickup, LiquidBlockContainer` in 26.1 too). `#signs` vs
  `Class::Sign` (all signs incl. hanging) is moot: all signs are
  SimpleWaterlogged so the LBC branch returns true first. §A 18/18 pass.
- **G6 dripstone never fills cauldrons — REPRODUCES as part of B1-G01**
  (see §Defects).
- **B11-G06 speleothem class name — REPRODUCES as part of B1-G01:** no
  `SpeleothemBlock` in the 26.1 jar; `Class::Speleothem` matches nothing.
- **B11-G13 splash water gate — REPRODUCES as B1-G03** (see §Defects).

## Withdrawn

- **Cauldron mayInteract candidate — NOT A DEFECT (nuance, §H).**
  Vanilla's `LayeredCauldronBlock.entityInside` runBefore lambda requires
  `entity.isOnFire() && entity.mayInteract(level, pos)` before lowering
  the level (`lambda$entityInside$0`, javap-verified for 26.1); the port
  gates on fire_ticks>0 && !spectator and has no mayInteract check. The
  gate is spawn protection + world border (`ServerLevel.mayInteract`:
  non-player → true; player → `!isUnderSpawnProtection &&
  worldBorder.isWithinBounds`), which the port's single-player/LAN-host
  model does not implement as a concept. Recorded as a nuance, not a
  defect — the core behaviour (level lowering, powder-snow conversion,
  extinguish) matches. Same as 26.2.
- No defect number was withdrawn in this batch: B1-G01–G04 are all
  attached to defects.

## Passes by section

### §A. Fluid spreading and flow — 18 checks, 18 pass

- `canHoldAnyFluid` hardcoded form (LBC → true; blocksMotion → false;
  DoorBlock / `#signs` / ladder / sugar cane / bubble column / portals /
  end gateway / structure void excluded; else true) — pass (see G5).
- `is_liquid_container` = LiquidBlockContainer OR SimpleWaterlogged —
  pass; required because the prepared export doesn't expand interface
  inheritance (jar: `SimpleWaterloggedBlock extends BucketPickup,
  LiquidBlockContainer` in 26.1).
- Direct LiquidBlockContainer blocks: 4 (kelp, kelp_plant, seagrass,
  tall_seagrass) — same as 26.2 — pass.
- Waterlogging exhaustive set: **410/410 EXACT MATCH** (vanilla
  `blocks.json` blocks with any `waterlogged` property = 410; prepared
  `block-behaviours.json` blocks with `SimpleWaterloggedBlock` interface =
  410; zero diffs either way). Total states 29,873 confirmed — pass.
- `#signs` exclusion moot (all signs SimpleWaterlogged, LBC branch
  returns true first) — pass.
- Water numbers: slope 4, dropOff 1, tickDelay 5 — pass.
- Lava numbers: slope 4 fast / 2 slow, dropOff 1 fast / 2 slow, tickDelay
  10 fast / 30 slow; `isFastLava` via `FAST_LAVA` environment attribute —
  pass.
- Lava `getSpreadDelay` ×4 (both fluids non-empty, neither FALLING, target
  height higher, nextInt(4) != 0) — pass.
- Spread machinery: `spread` / `spreadToSides` / `getNewLiquid` /
  `getSpread` / `getSlopeDistance` / `spread_targets` — pass (vanilla
  `getNewLiquid`: Plane iterator, `canPassThroughWall`, isSource, amount
  max, 2-source + `canConvertToSource` → `getSource`; Rust
  `level/fluids.rs` identical to 26.2's verified code except comments).
- `spread_to` incl. lava-into-water: `LavaFluid.spreadTo` override —
  direction == DOWN && this is LAVA tag && fluid at pos is WATER → if
  LiquidBlock → stone flag 3; fizz (level event 1501); else
  `FlowingFluid.spreadTo` (LBC.placeLiquid; else beforeDestroyingBlock
  unless air; createLegacyBlock flag 3) — pass.
- `can_be_replaced_with`: WaterFluid — DOWN && fluid not in WATER tag;
  LavaFluid — height ≥ 0.44444445f && fluid is WATER — pass.
- `can_hold_specific_fluid`: LiquidBlockContainer.canPlaceLiquid — pass.
- `place_liquid` / `pickup_block`: `LiquidBlock.pickupBlock` LEVEL==0 →
  setBlock(AIR, 11) + bucket item; SimpleWaterlogged drain; bubble column
  → air + water bucket; powder snow → air + event 2001 — pass.
- `should_spread_liquid`: lava + water neighbour (POSSIBLE_FLOW_DIRECTIONS
  via opposite) → obsidian if source else cobblestone + fizz; soul soil
  below + blue ice neighbour → basalt + fizz — pass.
- `liquid_block_changed` scheduling: onPlace = shouldSpreadLiquid →
  scheduleTick(fluid, tickDelay) + tryScheduleBubbleBlockColumn;
  updateShape = source transitions schedule fluid tick, DOWN +
  shouldBubbleColumnOccupy → trySchedule — pass.
- `getFlow` vector: 0.8888889f height-difference algorithm, below-check
  via `blocksMotion()`, `affectsFlow`, `isSolidFace` (same-family →
  false, UP → true, ice → false, else isFaceSturdy), normalize — pass.
- `can_convert_to_source` + 2-source rule: `water_source_conversion`
  default true, `lava_source_conversion` default false (GameRules static
  init) — pass.
- `blocks_fluid_flow` := `blocks_motion` mapping (`assets.rs:74,212,283`)
  — pass; the tag does not exist in 26.1 data (no field in `BlockTags`),
  so the port's deliberate mapping is the correct 26.1 form.

### §B. Bubble columns — 10 checks, 10 pass

- `canOccupy`: is bubble_column OR (fluid in
  `bubble_column_can_occupy` AND block is LiquidBlock AND isSource AND
  amount ≥ 8) — pass.
- `shouldBubbleColumnOccupy`: fluid in tag + source + full — pass.
- `getColumnState`: below is column → below state; push_up tag →
  drag=false; drag_down tag → drag=true; occupy is column → water
  default; else occupy — pass.
- `updateColumn`: canOccupy → setBlock(getColumnState, 2), then walk up
  while canOccupy — pass (`level/tick.rs:66` `bubble_column_update`).
- updateShape: canSurvive check + scheduleTick(self, 5) when below is
  column / canOccupy — pass.
- `canSurvive`: below is column OR push_up OR drag_down tag — pass.
- `tryScheduleBubbleBlockColumn`: below in drag_down OR push_up tag →
  scheduleTick(self, 20) — pass.
- `entityInside` surface-vs-inside split: above collision empty AND
  above fluid empty → `onAboveBubbleColumn(drag)` else
  `onInsideBubbleColumn(drag)` — pass.
- Entity motion constants: above drag max(-0.9, vy−0.03), above push
  min(1.8, vy+0.1) + splash particles; inside drag max(-0.3, vy−0.03),
  inside push min(0.7, vy+0.06) + resetFallDistance — pass
  (`motion/fluids.rs` `BubbleColumn::push`, only API-arg diffs vs 26.2).
- `bubble_column_at` surface detection + push returns !surface — pass.

### §C. Lava fire, ignition and game rules — 16 checks, 16 pass

- `lava_random_tick` — pass (see G3).
- `lava_flammable` — pass, with the below/above-build-height nuance
  (§Method).
- Fire odds: **207/207 FULL MATCH** (parsed vanilla
  `FireBlock.bootStrap` setFlammable entries vs prepared
  `fire.igniteOdds`/`burnOdds`; zero value diffs, zero only-either-side;
  white_wool (30,60), white_carpet (60,20) spot-checked) — pass.
- `FireBlock.tick`: scheduleTick(getFireTickDelay) first;
  canSpreadFireAround gate → return; canSurvive → removeBlock(0);
  infiniburn check; rain 0.2+0.03*age; age++ min(15, age+nextInt(3)/2)
  setBlock 260; isValidFireLocation; checkBurnOut ×6 (300/300/250/250/
  300/300) — pass.
- `checkBurnOut`: waterlogged → 0; igniteOdds map lookup; nextInt(burnOdds)
  < chance; nextInt(10) < 5, !isRainingAt → fire via getStateWithAge flag
  3, else removeBlock(0); TNT prime — pass.
- `getFireTickDelay`: 30 + nextInt(10) — pass.
- `BaseFireBlock.getState`: below canSurviveOnBlock → SOUL_FIRE else
  FIRE.getStateForPlacement — pass.
- `fireIgnite`: !fireImmune; remaining<0 → setRemainingFireTicks(1); +
  nextInt(1,3) ticks — pass.
- `lavaIgnite`: !fireImmune → igniteForSeconds(15.0) — pass.
- `lavaHurt`: !fireImmune, server-side, hurtServer(lava, 4.0) →
  shouldPlayLavaHurtSound && !isSilent → GENERIC_BURN 0.4 vol, pitch
  2+rand*0.4 — pass on the damage; the sound was NOT traced in the Rust
  (untraced sub-detail, same as 26.2).
- `LivingEntity.igniteForTicks`: ceil(ticks × BURNING_TIME) →
  Entity.igniteForTicks — pass.
- `LavaCauldronBlock.entityInside`: CLEAR_FREEZE + LAVA_IGNITE +
  runAfter(LAVA_IGNITE, lambda) — pass (same as `LavaFluid.entityInside`).
- `InsideBlockEffectType` mapping: 5 values (FREEZE, CLEAR_FREEZE,
  FIRE_IGNITE, LAVA_IGNITE, EXTINGUISH) exist in 26.1; the Rust's
  StepBasedCollector mapping (verified in 26.2 against identical Rust)
  applies — pass.
- `canSpreadFireAround` + spectator exclusion — pass (see G4).
- Infiniburn sets: overworld magma_block + netherrack, nether same, end
  + bedrock (tags verified in 26.1 data) — pass.
- `Entity.baseTick` fire phase: matches the 26.2 form incl. fallDistance
  ×0.5 in lava, `setSharedFlagOnFire`, firstTick=false (a leash tick at
  the end is extra vs the 26.2 note) — pass.

### §D. Entity fluid interaction and pushing — 7 checks, 6 pass, 1 gap

- `update` structure: reset all trackers; box null → return; floor/ceil
  bounds; `hasFluidAndLoaded` gate; per-cell loop (x outer, y middle, z
  inner); tracker = getTrackerFor cached by last kind; null tracker →
  skip cell entirely — pass.
- Eye-height check: Rust uses `camera_height` instead of the plain top —
  **GAP (B1-G04)**.
- Flow scaling: Rust scales by current height = max over kind-trackers
  fed = the same tag-level max vanilla uses (same value, same order) —
  pass.
- `Tracker.applyCurrentTo` constants: count==0 OR lengthSqr <
  9.999999747378752E-6 → return; non-Player → normalize; Player →
  scale(1/count); scale by speed; if |dx|<0.003 && |dz|<0.003 &&
  length<0.0045 → normalize.scale(0.0045); addDeltaMovement — pass
  (Rust `CurrentAccumulator::apply_to` matches exactly,
  `f64::from(1.0e-5f32)` floor).
- `hasFluidAndLoaded`: Z-then-X chunk loop, FULL chunk, null → false,
  section index bounds, `present |= hasFluid` — pass.
- `Entity.updateFluidInteraction`: `interaction.update(entity,
  !isPushedByFluid)`; inWater/isInFluid(WATER), inLava; inWater →
  resetFallDistance + splash if !wasTouchingWater && !firstTick;
  wasTouchingWater = inWater; if isPushedByFluid: water →
  applyCurrentTo(WATER, 0.014); lava → FAST_LAVA? 0.007: 0.0023333333333333335;
  return inWater || inLava — pass. `Entity.isInLava` = !firstTick &&
  isInFluid(LAVA) — pass.
- Fluid interaction box: bbox.deflate(0.001) then vehicle override;
  `AbstractBoat.modifyPassengerFluidInteractionBox` (isUnderWater →
  unchanged; boatTop >= passenger maxY → null; else minY = max(minY,
  boatTop)) — pass (Rust `fluid_box.rs`, 0 diff vs 26.2).

### §E. Concrete powder — 7 checks, 5 pass, 2 gap

- `onPlace` solidify: shouldSolidify → setBlock(concrete, 3) — pass
  (`place.rs:775`: water fluid at replaced OR `concrete_touches_liquid`).
- `getStateForPlacement` solidify: shouldSolidify → concrete, else super
  — pass.
- `shouldSolidify`: canSolidify(state) OR touchesLiquid — pass.
- `touchesLiquid` mutable-position quirk: iterates Direction.values(),
  DOWN handled specially (canSolidify check on the DOWN neighbour first),
  setWithOffset, canSolidify + !isFaceSturdy(opposite) — pass
  (`shapes.rs:1090` `concrete_touches_liquid` reproduces the quirk
  exactly, including the `test` variable only advancing when the
  DOWN-check passes).
- `canSolidify`: fluid state is WATER tag — pass.
- First-water-cell landing check — **GAP (B1-G02)**.
- Fast-fall clip check (COLLIDER + SOURCE_ONLY) — **GAP (B1-G02)**.

### §F. Falling blocks — 13 checks, 13 pass

- `FallingBlock.tick`: below isFree && y >= minY →
  `FallingBlockEntity.fall` + `falling(entity)` — pass.
- `getDelayAfterPlace`: base 2 (`iconst_2`), `DragonEggBlock` 5
  (`iconst_5`); Rust `hooks.rs:162` uses 5 for DragonEgg, 2 otherwise —
  pass.
- `isFree`: isAir || is(`#fire`) || liquid() || canBeReplaced() — pass.
- `FallingBlockEntity.tick` structure: air → discard; time++;
  applyGravity; move SELF; applyEffectsFromBlocks; handlePortal;
  server-side + alive → water/landing/timeout branches — pass.
- Landing machinery: MOVING_PISTON → keep falling (NOT discarded — the
  port's `falling_block_landed` Continue branch matches); cancelDrop →
  callOnBrokenAfterFall only; canBeReplaced (DirectionalPlaceContext
  DOWN/UP) + isFree(below) + canSurvive; waterlogged set if fluid is
  WATER; setBlock(blockState, 3); block-entity data restore via
  TagValueOutput/Input — pass. (The `stuck_in_water` parameter exists in
  the landing function; the gap is only that the caller always passes
  false and the fall step never lands on water contact — B1-G02, §E.)
- Timeout: (time > 100 && y outside [minY, maxY]) || time > 600 →
  dropItem && ENTITY_DROPS → spawnAtLocation; discard — pass.
- Air drag: deltaMovement.scale(0.98) at end of tick — pass (Rust uses
  the 0.98 double literal in 26.1, matching vanilla; 26.2 calls
  `Entity.getAirDrag()`).
- `causeFallDamage`: hurtEntities gate; ceil(fallDistance)-1;
  NO_CREATIVE_OR_SPECTATOR + LIVING_ENTITY_STILL_ALIVE predicates;
  `Fallable.getFallDamageSource` or `fallingBlock`; min(floor(perDistance),
  max); anvil: damage>0 && nextFloat < 0.05 + damage*0.05 →
  `AnvilBlock.damage` — pass.
- Anvil events: `onLand` !isSilent → levelEvent 1031; `onBrokenAfterFall`
  !isSilent → levelEvent 1029 — pass (Rust `falling.rs:133` emits 1031;
  `falling_damage.rs:95` emits 1029; `game.rs:4458-4459` maps both to
  sounds).
- `AnvilBlock.damage` chain: ANVIL → CHIPPED_ANVIL → DAMAGED_ANVIL,
  FACING preserved — pass.
- `getFallDamageSource`: `damageSources.anvil(entity)` — pass.
- `game.rs` FallingBlock event consumer (4574-4610): spawns the entity
  at pos+0.5, y, z+0.5, velocity ZERO, drop_item true,
  `Damage::falling(name)`, hurt_per_distance → hurt_entities — pass
  (re-located in 26.1's game.rs; 26.2 cited 4554-4590).
- `Damage::falling` defaults (anvil 2.0 per distance / 40 max) — pass
  (`falling_block.rs`, 0 diff vs 26.2).

Out-of-scope observation (not counted): dragon-egg teleport on use
matches vanilla exactly (1000 tries — `sipush 1000` in the 26.1 jar, not
16; offsets nextInt(16)-nextInt(16), nextInt(8)-nextInt(8),
nextInt(16)-nextInt(16); target air && below not air && within world
border && inside build height; portal particles along lerp; setBlock flag
2) — but teleport on use is out of scope for this batch (falling-block
scope covers fall/landing/conversion), same as 26.2.

### §G. Dripstone and speleothem — 10 checks, 2 pass, 8 gap

Restructured vs 26.2's 26 checks: 26.2's speleothem system works, so its
checks compare live code against vanilla. In 26.1 the system is dead
(B1-G01), so §G counts OBSERVABLE behaviours only — the dead-code
constant matches in `speleothem.rs` (growth 0.011377778, transfer
0.17578125/0.05859375, findTip max 11, cauldron delay 50, tick delays
2/1, mud→clay event 1504, `randomTick` draw order) are verified matching
but NOT counted as passes, because the code never runs.

- Random-tick transfer (stalactite → stalagmite fluid transfer) —
  **GAP (B1-G01)**.
- Random-tick growth — **GAP (B1-G01)**.
- Cauldron fill from dripstone (`cauldron_tick`'s tip search needs
  `is_free_hanging_stalactite`) — **GAP (B1-G01)**. (The cauldron-side
  receive logic itself is separately verified in §H.)
- Placement (tip direction/thickness calc via `place.rs:1156`) —
  **GAP (B1-G01)**.
- updateShape (thickness recalc + survival scheduling via
  `shapes.rs:728`) — **GAP (B1-G01)**.
- canSurvive (floating dripstone never breaks — falls through to `true`)
  — **GAP (B1-G01)**.
- Scheduled tick (unsupported stalagmites destroyed / falling stalactites
  spawned via `tick.rs:248`) — **GAP (B1-G01)**.
- Falling stalactite spawn (tag-gated, `falling.rs:55`) —
  **GAP (B1-G01)**.
- `fallOn` stalagmite damage (up + tip → Stalagmite, distance+2.5,
  ×2.0) — pass (name/data-keyed, not dispatch-gated).
- Trident break (Trident + hit_speed > 0.6 → destroy, with mayInteract /
  mayBreak / `projectiles_can_break_blocks` gates) — pass
  (name-keyed, `creatures.rs:5037-5095`).

### §H. Cauldrons — 8 checks, 8 pass

- `CauldronBlock.handlePrecipitation`: RAIN → nextFloat < 0.05 → water
  cauldron level 1; SNOW → nextFloat < 0.1 → powder snow cauldron;
  BLOCK_CHANGE game event — pass (`tick.rs:1511` `precipitation_tick`:
  chances 0.05/0.1, defaults level 1, BLOCK_CHANGE).
- `LayeredCauldronBlock.handlePrecipitation`: shouldHandlePrecipitation
  && level != 3 && precipitationType matches → cycle(LEVEL) +
  setBlockAndUpdate + BLOCK_CHANGE — pass.
- `canReceiveStalactiteDrip`: empty cauldron → true; layered → fluid ==
  WATER && precipitationType == RAIN; lava cauldron → false (no
  override, AbstractCauldronBlock default) — pass
  (`speleothem.rs:314` `cauldron_can_receive`: Cauldron → true;
  LayeredCauldron → water && water_cauldron; else false).
- `receiveStalactiteDrip` state transitions + events: empty + WATER →
  WATER_CAULDRON + event 1047; empty + LAVA → LAVA_CAULDRON + event 1046;
  layered !isFull → LEVEL+1 + event 1047; BLOCK_CHANGE each — pass
  (`speleothem.rs:335` `cauldron_tick` second half matches exactly).
  Note: the drip trigger is dead via B1-G01 (§G gap) — the receive logic
  itself is correct and is counted here once; the observable "cauldrons
  never fill from dripstone" failure is counted once in §G.
- `entityInside` EXTINGUISH + runBefore: server-side → runBefore
  (EXTINGUISH, handleEntityOnFireInside lambda); always apply(EXTINGUISH)
  — pass. The lambda requires `isOnFire() && mayInteract(level, pos)`
  (javap-verified); the Rust gates on fire_ticks>0 && !spectator — the
  mayInteract part is the §Withdrawn nuance.
- `handleEntityOnFireInside` + `lowerFillLevel`: precipitationType==SNOW
  → convert to WATER_CAULDRON at same level, then lowerFillLevel; else
  lowerFillLevel; level−1; 1 → empty cauldron default; setBlockAndUpdate
  + BLOCK_CHANGE — pass (`cauldron.rs:112-120` `extinguished`/`lowered`;
  `game.rs:8047` `cauldron_extinguish` incl. the
  `native_cauldron_level_change_allowed` gate and BLOCK_CHANGE event).
- `lava_cauldron_inside`: CLEAR_FREEZE (ticks_frozen=0) + LAVA_IGNITE
  (ignite_for_ticks(300) = 15s) + lava hurt 4.0 — pass
  (`motion.rs:2019`).
- CauldronExtinguish emit conditions (burning, !spectator) + LAN
  consumption (`lan_actions.rs:5265` → `game.cauldron_extinguish`) —
  pass. Content heights: lava 15/16, layered (6.0 + 3.0*level)/16 —
  verified (`motion.rs:1811-1816`).

### §I. Splash water — 4 checks, 0 pass, 4 gap

Restructured vs 26.2's 6 checks: 26.2's gate works, so its checks compare
live code. In 26.1 the gate kills all four outcomes at once (B1-G03), so
§I counts the four observable outcomes — the gate itself is the shared
cause, described in B1-G03, and the downstream code (verified matching
vanilla exactly, see B1-G03) is recorded as analysis, NOT counted as
passes.

- Dowse fire on block hit — **GAP (B1-G03)**.
- Hurt water-sensitive entities (1.0 indirectMagic) — **GAP (B1-G03)**.
- Extinguish burning entities — **GAP (B1-G03)**.
- Rehydrate axolotls (+1800 air, capped) — **GAP (B1-G03)**.

### Tick dispatch callers — verified (folded into the sections above)

- `level/tick.rs` scheduled-tick dispatch: bubble column (66),
  AbstractCauldron (63), falling blocks, fire — verified live. The
  Speleothem arms (248, 434) are dead (B1-G01).
- `level/shapes.rs` updateShape dispatch: bubble column (270), falling
  (617-618), liquid (617-618) — verified live. The Speleothem arm (728)
  is dead (B1-G01).
- `level/place.rs` placement dispatch: concrete solidify (775), liquid
  (617-618) — verified live. The Speleothem arm (1156) is dead (B1-G01).
- `level/survive.rs` canSurvive dispatch: bubble column, falling —
  verified live. The Speleothem arm (307) is dead (B1-G01); the
  fall-through returns `true`, so floating dripstone never breaks.

### Prepared-data class/interface spot checks

- pointed_dripstone: class chain `[PointedDripstoneBlock, Block]`,
  fields `{}` (no `blockToGrowOn`) — the B1-G01 causes #1 and #3.
- Waterlogging census 410/410 (§A); fire odds census 207/207 (§C).
- `block-behaviours.json` structure: top keys `blocks` (1,168), `states`,
  `items`, `fluids`, `fire` (shapes/igniteOdds/burnOdds/fireDamage=1.0).
- `datagen/reports/registries.json` entries are ALPHABETICAL, not id
  order — sort by `protocol_id` before cumulative state counting
  (total states 29,873 confirms the count).
- Prepared `block-tags.json`: 248 keys, no `speleothems`. Prepared tag
  values are raw JSON strings; nested refs are `#minecraft:<name>`
  (strip leading `#` only when resolving).

## Oracle defects in scope

Authority: the measured Batch 0 table
(`docs/sweep/26.1/batch-0-oracles.json`; 199/391 compile, 192 fail) —
triage from it ONLY, never from an import-level check. 13 in-scope
oracles are present in
`versions/26.1/mods/horizon/src/test/java/dev/horizonlab/parity/`.
**9 fail on all three classpaths, 4 pass** (vs 26.2's 7 fail / 6 pass).
Full error lists were extracted from
`versions/26.1/.local/measure-oracles/all-client.log`; every missing
symbol below was javap-verified against the 26.1 jar by this batch.

### B1-O01 — `BlockTags.BLOCKS_FLUID_FLOW` is 26.3-only; breaks 3 oracles (MINOR)

- VanillaFluidFlowOracle: 1 error, line 15, missing-symbol.
- VanillaRegisteredEntityFluidOracle: 2 errors, lines 9–10.
- VanillaRegistryFluidOracle: 3 errors, lines 11, 14 (BLOCKS_FLUID_FLOW;
  line 17 is B1-O04's symbol).
- javap-verified: no `BLOCKS_FLUID_FLOW` field in 26.1 `BlockTags`; the
  tag file is absent from 26.1 data. Same as 26.2 B1-O01. The port
  deliberately maps blocks_fluid_flow := blocks_motion for 26.1, so the
  repair is to bind the tag from the blocks_motion table, not to add the
  tag.

### B1-O02 — `VanillaCauldronOracle` BROKEN, 1 compile error (MINOR)

- 1 error, line 118, missing-symbol: the `drop()` override references
  `net.minecraft.util.Prediction` — ABSENT from both 26.1 jars (jar
  listing verified). Same as 26.2 B1-O02.

### B1-O03 — `VanillaFallingBlockOracle` BROKEN, 1 compile error (MINOR)

- 1 error, line 30, constructor-mismatch: `NoopChunkMap` calls `super()`
  with 12 args; 26.1 `ChunkMap` ctor takes 13 params (ServerLevel,
  LevelStorageAccess, DataFixer, StructureTemplateManager, Executor,
  BlockableEventLoop, LightChunkGetter, ChunkGenerator,
  ChunkStatusUpdateListener, Supplier, TicketStorage, int, boolean —
  javap-verified). Same as 26.2 B1-O03; Batch 0 group G6.

### B1-O04 — `VanillaFluidOracle` BROKEN, 1 compile error (MINOR)

- 1 error, line 50, missing-symbol: `state.getHeightForCamera(world, p)`.
  26.1 `FluidState` has only `getHeight(BlockGetter, BlockPos)`
  (javap-verified). Same as 26.2 B1-O04. (VanillaRegistryFluidOracle
  line 17 shares the symbol.)

### B1-O05 — `VanillaLivingFluidOracle` BROKEN, 9 compile errors (MINOR)

- 9 errors (vs 26.2's 3): lines 30 + 61 `Attributes.FRICTION_MODIFIER`
  and `Attributes.AIR_DRAG_MODIFIER` missing (javap: 26.1 `Attributes`
  has only BURNING_TIME and WATER_MOVEMENT_EFFICIENCY among the greps);
  lines 50 + 81 + 97 `FluidTags.ENTITY_FLOATABLE` missing (26.1
  `FluidTags` has exactly 6 fields, no ENTITY_FLOATABLE); lines 54 + 85
  `omnidirectionalAirMover` no-override (absent from 26.1
  `LivingEntity`). Note: 26.1 `LivingEntity` also lacks `travelInFluid`,
  `travelInLava`, `floatInLiquidWhileRidden`, `jumpOutOfFluid` (used via
  reflection in the oracle — would fail at runtime too, not compile
  errors); `travelInWater` and `getFluidFallingAdjustedMovement` exist.
  Expanded vs 26.2 B1-O05 (which had only the ENTITY_FLOATABLE breaks).

### B1-O06 — `VanillaEntityFluidOracle` BROKEN, 1 compile error (MINOR, NEW vs 26.2)

- 1 error, line 74, class `other`: `'void' type not allowed here` —
  `interaction.update(...)` used in string concat; 26.1
  `EntityFluidInteraction.update` returns **void** (javap-verified).
  Passed in 26.2 (where the oracle's form matched).

### B1-O07 — `VanillaLavaCauldronItemOracle` BROKEN, 1 compile error (MINOR, NEW vs 26.2)

- 1 error, line 28, no-override: `getNextEntityId()` does not override —
  absent from both 26.1 `ServerLevel` and `Level` (javap-verified).
  Passed in 26.2.

### Oracles assessed PASSING for 26.1 (Batch 0 measurement)

All three classpaths: VanillaFluidBoxOracle, VanillaFluidOccupancyOracle,
VanillaFluidShapeOracle, VanillaWaterSplashOracle.

## Method notes

- **Prepared data landed before the run** (GATE OK):
  `versions/26.1/.local/horizon-client/26.1/` was available from the
  start. Two checks depended on it and are counted as passes: the
  waterlogging exhaustive set (§A, 410/410) and the fire odds values
  (§C, 207/207). `sounds.json` is genuinely absent (a separate step not
  run) — noted, not treated as a port defect; `en_us.json` is present
  (Batch 0 verified md5-identical to the client jar's file).
- **Vanilla ground truth is 26.1's own jar/data only** (owner's
  vanilla-check rule). Every vanilla claim above cites the 26.1
  server-game jar's bytecode or the 26.1 client jar's data. Where 26.1's
  form differs from 26.3's (no `washed_away_by_fluids`, no
  `blocks_lava_fire_spread`, no `douses_fire`, no `doFireTick`, no
  `SpeleothemBlock`, no `#speleothems`, no sulfur_spike), the port was
  checked against the 26.1 form.
- **Tag inventory verified in 26.1's own data** (client jar + server-game
  jar data + `BlockTags`/`FluidTags` javap): `#speleothems` ABSENT (key
  finding — existed in 26.2); `washed_away_by_fluids`,
  `blocks_lava_fire_spread`, `blocks_fluid_flow` ABSENT;
  `enables_bubble_column_push_up` / `enables_bubble_column_drag_down`
  EXIST; fluid tags total six (bubble_column_can_occupy, lava,
  supports_frogspawn, supports_lily_pad, supports_sugar_cane_adjacently,
  water); potion tags: only `tradeable` (41 values); block tags `signs`
  (24 = standing + wall; hanging signs 24 separate), `anvil` (3),
  `fire` (fire, soul_fire), `infiniburn_overworld` (magma_block,
  netherrack), `infiniburn_nether` (= overworld), `infiniburn_end` (+
  bedrock) all exist. No sulfur_spike in 26.1 (0 entries in the
  server-game jar; absent from prepared block-behaviours).
- **26.1 universe** (established): DataVersion 4786, protocol 775, Java
  25; 1,168 blocks, 29,873 states, 157 entities, 1,506 items, 117
  particles, 65 biomes, 49 BE types. Registration still lives on
  EntityType/BlockEntityType (no EntityTypes/BlockEntityTypes holder);
  bed BE type present.
- **Rust-vs-26.2 diffs.** Most in-scope files are identical to 26.2's
  verified code (0 diff lines): `level/shapes.rs`, `level/speleothem.rs`,
  `level/falling.rs`, `level/survive.rs`, `level/place.rs`,
  `fluid_box.rs`, `motion/environment.rs`, `falling_block.rs`,
  `cauldron.rs`. Comment-only diffs: `level/fluids.rs`,
  `level/tick.rs`, `level/hooks.rs`, `level/mod.rs`,
  `game/falling_blocks.rs` (incl. 26.1 air-drag 0.98 double literal vs
  26.2 `Entity.getAirDrag()`), `game/falling_damage.rs`, `assets.rs`.
  Real diffs, all verified in this batch: `entity_fluid.rs` (26.1 ships
  the 26.3 form — B1-G04), `game/splash_water.rs` (26.1 ships the 26.3
  tag form — B1-G03), `motion/fluids.rs` + `entities/fluids.rs` (26.3-form
  interaction API), `entity_fluid_tick.rs` (26.1 returns `Ok(changed)`).
  Files cited in the 26.2 report live at
  `horizon-world/src/block_landing.rs`,
  `horizon-world/src/entity_fluid_tick.rs`,
  `horizon-client/src/cauldron.rs` (not the paths the 26.2 report
  printed); the `game.rs` regions cited there were re-located in 26.1's
  game.rs (320 diff lines): FallingBlock consumer 4574-4610,
  fire_players 3845, cauldron_extinguish 8047.
- **Nuances recorded, not defects:** (1) Rust `lava_flammable` returns
  false only if inside build height AND not loaded, then checks the flag;
  vanilla returns false whenever NOT (inside && hasChunkAt) — below/above
  build height the Rust proceeds to the flag check (state lookup there is
  presumably air → false anyway). (2) `EntityFluidInteraction.update`
  bool return differs from vanilla only in a 1mm band or for untagged
  mod fluids, and all callers discard it (B1-G04). (3) Cauldron
  mayInteract gate (§Withdrawn). (4) The dead `"minecraft:sulfur_spike"
  => Some(1052)` arm in `finish_falling_block` (harmless, B1-G02).
  (5) 26.1 uses "douse" naming vs 26.2 "dowse" (cosmetic, B1-G03).
- **Untraced sub-details (NOT counted as passes, NOT needs-harness — they
  are sub-details of passing checks or out of scope):** the lava-hurt
  GENERIC_BURN sound (vol 0.4, pitch 2+rand*0.4) was not traced in the
  Rust; `PointedDripstoneBlock.animateTick` drip particles were not
  compared; dragon-egg teleport on use is out of scope (falling-block
  scope covers fall/landing/conversion) — though it was verified matching
  anyway (see §F).
- **No cargo builds, no unit tests, no oracle measurement** were run in
  this batch (brief constraints + the shared 6 GB VM). All verdicts are
  javap + source + data comparisons. Oracle triage used the measured
  Batch 0 table, not a fresh measurement.
- **Intermediate evidence.** 36 `javap -c` dumps are kept under
  `b01/evidence/javap/` in the batch directory (fluids, fire, falling
  blocks, cauldrons, boats, entity fluid interaction, game rules).
  All facts cited above trace to those dumps, the named jars, or the
  prepared data.
- **Check-count hygiene.** The 93 checks count only comparisons where
  both sides were actually read. Overlapping sub-checks were not
  double-counted (e.g. cauldron receive vs cauldron_tick are separate
  checks only where the vanilla methods are separate, and the drip-fill
  failure is counted once in §G, not again in §H); dead-code matches
  were not counted as passes (§G, §I); untraced items were not counted
  as passes. Per-section counts: §A 18/18/0, §B 10/10/0, §C 16/16/0,
  §D 7/6/1, §E 7/5/2, §F 13/13/0, §G 10/2/8, §H 8/8/0, §I 4/0/4 —
  summing to 93 checks / 78 pass / 15 gap.

## Ledger

`slice.json` in this directory — 11 items (B1-G01–G04, B1-O01–O07),
checks 93/78/15.
