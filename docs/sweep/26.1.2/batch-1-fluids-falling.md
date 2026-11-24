# Batch 1 — Fluids + falling blocks (26.1.2 parity sweep)

**Version:** 26.1.2 · **Date:** 2026-11-25 · **Branch:** `parity-sweep-26.1.2`
(workdir `~/workspace/mcir-work/sweep-26.1.2`) ·
**Scope:** `docs/PARITY-SWEEP.md` §4 Batch 1 — the pilot scope re-run against
26.1.2: water/lava spreading and flow, fluid pushing, bubble columns, concrete
powder solidification, lava fire spread + ignition + game rules, dripstone
growth + cauldron fill, falling blocks, cauldron fill levels, splash water.

**Status: DRAFT — not committed.** Checks + triage only, per the brief: no
fixes, no edits under `versions/26.1.2/`, no cargo builds, no oracle
measurement of my own, no commits, no git in the worktree. Everything below
is static analysis: `javap -c` on the named 26.1.2 server-game jar
(`~/workspace/mcir-work/26.1.x/versions/26.1.2/jars/server-game.jar`,
sha1 `83eb1106f12f24c9127ce22070d71853157af127`, DataVersion 4790,
protocol 775, Java 25), source comparison against
`versions/26.1.2/crates/`, tag/data reads from the named 26.1.2 client jar
(sha1 `4e618f09a0c649dde3fdf829df443ce0b8831e65`) and the 26.1.2 datagen
reports. Oracle triage is from the 26.1.1 Batch 0 table, provisionally —
the 26.1.2 Batch 0 table (`docs/sweep/26.1.2/batch-0-oracles.json`) had
NOT landed at handoff (checked before finishing); every missing symbol was
javap-verified against the 26.1.2 jar, every in-scope oracle source is
byte-identical between the two trees, and the only reference to a changed
class (`SharedConstants.tryDetectVersion`) was verified present in 26.1.2.
The coordinator commits these deliverables to
`docs/sweep/26.1.2/batch-1-fluids-falling.md` and
`pins/26.1.2/sweep/batch-01.json` (or their chosen paths).

**Prepared-data caveat (read before the censuses).** No prepared `.local`
exists for 26.1.2 in the worktree — Batch 0's prepare has not landed.
Prepared-data claims (the waterlogging set, fire-odds values, the
pointed-dripstone class chain/fields, the `block-tags.json` keys) were
verified against the 26.1.1 prepared data
(`~/workspace/mcir-work/sweep-26.1.1/versions/26.1.1/.local/horizon-client/26.1.1/`),
which is valid for 26.1.2 by the jar-diff keystone (§Method): the jars
differ only in classes outside Batch 1 scope and all five datagen reports
are byte-identical, so a 26.1.2 prepare would produce identical in-scope
prepared data. The vanilla sides of both censuses were counted fresh from
26.1.2's own jar/reports, and every tag presence/absence claim was verified
directly in the 26.1.2 client jar, not via prepared data.

## Tally

- **Behaviour checks: 93 — 73 pass, 20 gap** (all behaviour; non-behaviour
  0. As-tallied 93 = 93 behaviour + 0 non-behaviour). The 20 gap checks are
  the eight defect items below: B1-G01 × 8, B1-G02 × 2, B1-G03 × 4,
  B1-G04 × 1, B1-G05 × 1, B1-G06 × 1, B1-G07 × 2, B1-G08 × 1; see §Defects.
- **Defect items: 8** — B1-G01 (major), B1-G02 (minor), B1-G03 (major),
  B1-G04 (minor), B1-G05 (major), B1-G06 (minor), B1-G07 (minor),
  B1-G08 (minor).
- **Oracle defects: 7** (B1-O01–B1-O07, all minor — see §Oracle defects in
  scope: 9 of the 13 in-scope oracles fail on all three classpaths).
  Not counted in the check tally, following the 1.21.7 convention.
  Provisional on the 26.1.1 Batch 0 table until the 26.1.2 table lands.
- **Closed items: 2** (B1-G09, B1-G10 — pilot re-checks whose premise fails
  on 26.1.2 evidence; see §Closed).
- **Needs-harness: none** — no B1-Hxx items. Every in-scope check was
  decidable statically; the few untraced sub-details are noted in §Method,
  not faked as passes.
- **Withdrawn candidates:** one (the cauldron mayInteract gate — a nuance,
  not a defect; see §Withdrawn). No defect number was withdrawn: B1-G01–G08
  are all attached to defects.
- **26.1.1 re-checks (§Re-checks: 26.1.1 — PRIMARY):** all 17 DECIDED —
  15 REPRODUCE (B1-G01–G08, B1-O01–O07), 2 land closed again (B1-G09,
  B1-G10 — NEVER EXISTED for 26.1.2 as for 26.1.1). No FIXED, no
  WITHDRAWN, no UNVERIFIED.
- **1.21.7 re-checks (§Closed + §Defects):** G1 CLOSED (never existed for
  26.1.2), G4 CLOSED (never existed for 26.1.2); G2, G3, G5, G6, B11-G06,
  B11-G13 REPRODUCE (as B1-G02, B1-G06, B1-G05, B1-G01, B1-G01, B1-G03
  respectively). Same verdicts as the 26.1.1 run — expected: the 26.1.2
  tree is a delta port of the 26.1.1 tree and vanilla changed nothing in
  scope (see §Method).

## Defects

### B1-G01 — Entire pointed-dripstone system dead (major, REPRODUCES pilot G6 + B11-G06; REPRODUCES 26.1.1 B1-G01)

**Checks failed (8):** random-tick transfer; random-tick growth; cauldron
fill from dripstone; placement (tip direction/thickness); updateShape
(thickness recalc + survival scheduling); canSurvive (floating dripstone
never breaks); scheduled tick (unsupported stalagmites / falling
stalactites); falling stalactite spawn.

**Vanilla 26.1.2.** There is NO `SpeleothemBlock` class in the 26.1.2 jar
at all (0 matches in either jar): `PointedDripstoneBlock extends Block`
directly (implements `SimpleWaterloggedBlock`, `Fallable`) — the 1.21.7
shape, not 26.2's. Its bytecode contains ZERO `BlockTags` references:
every predicate is same-block. `PDB.canGrow` hardcodes
`Blocks.DRIPSTONE_BLOCK` + a water source at above(2) — no
`blockToGrowOn` field exists in 26.1.2. Constants re-verified fresh:
growth 0.011377778, transfer 0.17578125 water / 0.05859375 lava,
findTip max 11.

**The port is dead three independent ways** (all in-scope files byte-identical
to 26.1.1's, same line numbers):

1. **Dead dispatch.** `Class::Speleothem = "SpeleothemBlock"`
   (`level/classes.rs:215`) gates five dispatch sites — `place.rs:1156`
   (placement), `shapes.rs:728` (updateShape/thickness), `survive.rs:307`
   (canSurvive), `tick.rs:248` (scheduled tick), `tick.rs:434` (random
   tick: growth + transfer). Dispatch iterates the block's exported class
   chain; pointed_dripstone's prepared chain is
   `[PointedDripstoneBlock, Block]` — the Speleothem arm never fires.
   `speleothem.rs` itself uses `Class::PointedDripstone` (lines 86, 103,
   206) but is unreachable via dispatch.
2. **Dead tag.** `speleothem_tag()` reads `#minecraft:speleothems`
   (`speleothem.rs:13-15`) — absent from 26.1.2 data (no `speleothems.json`
   in the client jar, no `SPELEOTHEMS` field in `BlockTags`, no key in
   prepared `block-tags.json`, 248 keys) — so `speleothem_with`,
   `speleothem_is_tip`, `is_free_hanging_stalactite`,
   `speleothem_unmerged_tip` are all always false, and
   `falling.rs:55` `spawn_falling_stalactite` is tag-gated too.
3. **Missing link.** Prepared `fields` for pointed_dripstone is `{}` — no
   `blockToGrowOn`; `data.link` resolves to None (`level/data.rs:959-974`),
   so `speleothem_grow`'s `can_grow` is always false even if reached
   (`speleothem.rs:100-107`).

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
`game/creatures.rs:5081-5084` name match `"minecraft:pointed_dripstone"` +
Trident + hit_speed > 0.6 → destroy, with the `projectile_breaks_block`
gates (impact kind + shooter_may_break + `projectiles_can_break_blocks`
rule, `creatures.rs:5037-5067`) matching vanilla `onProjectileHit`
(server-side, mayInteract/mayBreak gates).

**Severity rationale:** the whole dripstone feature set is dead —
comparable in scope to pilot G1 (bubble columns never form, major), not
just the narrow G6 cauldron-fill framing (minor in the pilot). The
dead-code constants in `speleothem.rs` match vanilla exactly (growth
`0.011_377_778` at line 92, transfer `0.175_781_25` / `0.058_593_75` at
lines 274-292, findTip max 11), but they were NOT counted as passes (§G
counts observable behaviours only).

### B1-G02 — Concrete powder falling into water does not solidify on contact (minor, CARRIES OVER from pilot G2; REPRODUCES 26.1.1 B1-G02)

**Checks failed (2):** first-water-cell landing check; fast-fall clip check.

**Vanilla 26.1.2.** `FallingBlockEntity.tick` (server-game jar, re-verified
fresh): a falling concrete-powder entity computes `inWater = block is
ConcretePowderBlock && fluid at blockPosition is in the WATER tag`; if the
delta's length-squared is greater than 1.0 it additionally clips from
(xo,yo,zo) to the position with `ClipContext.Block.COLLIDER` +
`ClipContext.Fluid.SOURCE_ONLY` — a non-MISS hit whose fluid is WATER sets
the landing pos to the hit pos and `inWater = true`. The landing branch
runs when `onGround() || inWater`. `ConcretePowderBlock.shouldSolidify` =
`canSolidify(state)` (fluid is WATER) OR `touchesLiquid` (the
mutable-position quirk: DOWN handled specially, `isFaceSturdy(opposite)`
check); `onLand` replaces with the concrete block.

**The port.** `game/falling_blocks.rs` `fall_step` has NO concrete-powder
water check (landing only on `on_ground`; the comment at line 120 even
says "a falling block never runs baseTick, so it is never in water");
`game/falling_damage.rs` `finish_falling_block` calls
`falling_block_landed(..., false)` — `stuck_in_water=false` always
(line 133). So powder sinks to the lakebed and lands there as powder. The
horizon-world landing branch itself (`level/falling.rs`
`falling_block_landed`) matches vanilla, including the `onLand` solidify
check (`falling.rs:127`: water fluid at current OR
`concrete_touches_liquid`, via the data link) and the
`would_continue = isFree(below) && (!concrete || !stuck_in_water)` logic —
the fix is local to the client fall step. Same defect as 26.2's B1-G02,
26.1's B1-G02 and 26.1.1's B1-G02. Note: `finish_falling_block` also has
a dead `"minecraft:sulfur_spike" => Some(1052)` arm (harmless — no
sulfur_spike in 26.1.2; the 1052 sound event does not exist for this
version).

### B1-G03 — Splash water never douses fire / hurts / extinguishes / rehydrates (major, REPRODUCES B11-G13; REPRODUCES 26.1.1 B1-G03)

**Checks failed (4):** dowse fire on block hit; hurt water-sensitive
entities; extinguish burning entities; rehydrate axolotls.

**Vanilla 26.1.2.** `AbstractThrownPotion.onHitBlock` gates on
`PotionContents.is(Potions.WATER)` — Holder equality, NOT a tag
(re-verified fresh; `Potions.WATER` getstatic at two gate sites, zero
`douses_fire` references in the class). Dowse targets: hitPos.relative(dir),
hitPos itself, and the 4 horizontal neighbours of hitPos.relative(dir).
`dowseFire`: `#fire` tag → destroyBlock(pos, false, this); else lit candle
→ extinguish; else lit campfire → level event 1009 + `CampfireBlock.dowse`
(verified to exist in the 26.1.2 jar). `onHitAsWater`: LivingEntity in the
potion's box inflated (4.0, 2.0, 4.0) with WATER_SENSITIVE_OR_ON_FIRE
(isSensitiveToWater || isOnFire), distanceToSqr < 16.0; water-sensitive →
hurtServer(indirectMagic(owner, this), 1.0); on fire && alive →
extinguishFire; Axolotl in the same box → `rehydrate()` (air = min(air +
1800, maxAir), maxAir 6000).

**The port.** 26.1.2's `game/splash_water.rs` ships the 26.3 tag form
(byte-identical to 26.1.1's): `potion_in_tag(stack,
"minecraft:douses_fire")`, `"minecraft:hurts_water_sensitive_entities"`,
`"minecraft:extinguishes_entities"`,
`"minecraft:rehydrates_axolotls"` — read from `trades.potion_tags`, built
from the prepared data's `tags/potion` directory (`merchant.rs:488-489`,
574). That directory contains ONLY `tradeable.json` in 26.1.2 (verified
in the client jar — 1 potion tag file, 41 values — and in prepared data)
— none of the four tags exists, so all four gates are always closed and
no splash water ever does anything. 26.2's file uses `is_water_potion`
(Potions.WATER equality) and works. The downstream code itself matches
vanilla exactly (dowse targets, dowseFire, the (4,2,4) box, dist²<16,
hurt 1.0 indirectMagic, Enderman teleport counter, axolotl +1800 air
capped at MAX_AIR) — it is verified matching but unreachable, so it is
recorded as analysis, NOT counted as passes (§I counts the four observable
outcomes).

**Severity rationale:** all splash-water functionality is dead — four
distinct observable behaviours, a whole mechanic, same scope class as
B1-G01. (26.1.2 also uses "douse" naming vs 26.2 "dowse" — cosmetic only.)

### B1-G04 — Entity fluid interaction ships the 26.3 form; eye check uses camera height (minor; REPRODUCES 26.1.1 B1-G04)

**Checks failed (1):** eye-in-fluid height check.

**Vanilla 26.1.2.** `EntityFluidInteraction` is the 26.2 form
(javap-verified fresh): `trackerByFluid` Map<TagKey, Tracker>; `Tracker
{height, eyesInside, accumulatedCurrent, currentCount}` with the current
ON the tracker; NO `CurrentAccumulator` class; `update` returns void;
`getTrackerFor(Fluid)` = first tag holding the fluid. `update` bytecode:
reset all trackers; box null → return; floor/ceil bounds;
`hasFluidAndLoaded(minX−1, minY, minZ−1, maxX+1, maxY, maxZ+1)` else
return; per cell (x outer, y middle, z inner): fluid non-empty; top = y +
**plain** `FluidState.getHeight`; top < box.minY skip; tracker =
getTrackerFor (cached by last kind); null tracker → skip cell entirely;
eyes: x==blockX && z==blockZ && eyeY in [y, top] → eyesInside; height =
max(top − bbox.minY, height); if !skipPush: flow = getFlow; if
tracker.height < 0.4 → flow.scale(tracker.height) (the TAG tracker's
height); accumulate.

**The port.** 26.1.2's `horizon-world/src/entity_fluid.rs` (byte-identical
to 26.1.1's) ships the 26.3 form: per-kind trackers + separate
CurrentAccumulators, `update` returns `Result<bool>`, tracker pruning.
Most of that is behaviourally equivalent for vanilla fluids (see §D) —
but the eye check is not: `entity_fluid.rs:389-398` computes the top with
`world.camera_height(state, p)` (`fluid.rs:504` `camera_height_at`: a
source fluid with a sturdy block above → 1.0, else `height_at`) instead
of the plain top. Observable: an entity whose eye is in (0.889, 1.0]
above a source's block bottom with a solid ceiling → the port says
eyes-in-fluid, vanilla 26.1.2 says not. Affects `was_eye_in_water` /
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

### B1-G05 — NO fluid spreading at all, water or lava (major, REPRODUCES pilot G5 — was CLOSED in 26.1; REPRODUCES 26.1.1 B1-G05)

**Checks failed (1):** `canHoldAnyFluid`. **Re-checked FIRST per the brief.**

**Vanilla 26.1.2.** `FlowingFluid.canHoldAnyFluid` bytecode read in full,
fresh against the 26.1.2 server-game jar (`FlowingFluid.txt:1012-1075`):
hardcoded list — LiquidBlockContainer → true; blocksMotion → false;
excludes DoorBlock, `#signs`, ladder, sugar cane, bubble column, nether/end
portals, end gateway, structure void; else true (air passes). Zero
`washed` references anywhere in `FlowingFluid`. `washed_away_by_fluids`
is absent from 26.1.2 data (no tag file in the client jar, no field in
`BlockTags`, no key in prepared `block-tags.json`).

**The port.** 26.1.2's `level/fluids.rs:359-361` ships the 26.3 tag form
(byte-identical to 26.1.1's — the coordinator's Step 0 finding confirmed
by direct diff in this batch):

```rust
fn can_hold_any_fluid(&self, state: BlockStateId) -> bool {
    self.is_liquid_container(state) || self.b.in_set(state, &self.b.tags.washed_away_by_fluids)
}
```

`Tags::resolve` (`level/data.rs:396`) fills a missing tag with
`.unwrap_or_default()` — an EMPTY set — so `can_hold_any_fluid` is true
only for liquid containers. Air does not hold fluid, so `spread`,
`spreadToSides`, `getNewLiquid` and every downstream spread path find no
valid target: **no fluid spreads at all**, water or lava. The spread
machinery itself (`spread` / `spreadToSides` / `getNewLiquid` /
`getSpread` / `getSlopeDistance` / `spread_targets` / `spread_to` /
`should_spread_liquid` / `liquid_block_changed`) is byte-identical to
26.1.1's verified code and matches vanilla exactly — the failure is
isolated to the predicate, so those checks still pass as code comparisons
(same convention as B1-G02, where the matching landing branch counts as
pass and only the caller is gap). The observable "fluid spreads" failure
is counted once, on the predicate check.

**Severity rationale:** a whole mechanic dead — water and lava never move.
Same scope class as the pilot's G5 (major).

### B1-G06 — Lava fire spread walks through solid blocks (minor, REPRODUCES pilot G3 — was CLOSED in 26.1; REPRODUCES 26.1.1 B1-G06)

**Checks failed (1):** `lava_random_tick`.

**Vanilla 26.1.2.** `LavaFluid.randomTick` bytecode re-verified fresh:
matches the 26.2 form exactly — `canSpreadFireAround` gate first,
`nextInt(3)` passes, first branch cumulative offset(nextInt(3)-1, 1,
nextInt(3)-1), isLoaded else return, air + hasFlammableNeighbours →
setBlockAndUpdate(BaseFireBlock.getState(level, test)) + return,
**`blocksMotion` → return** (offset 188); else branch 3 iters
offset(dx,0,dz) from origin, isEmptyBlock(above) && isFlammable(test) →
setBlockAndUpdate(above(test), getState(level, test)).

**The port.** 26.1.2's `level/fluids.rs:777-785` stops only for the tag
(byte-identical to 26.1.1's):

```rust
} else if self
    .b
    .tag("minecraft:blocks_lava_fire_spread")
    .is_some_and(|tag| self.b.in_set(state, tag))
{
    return;
}
```

`BlockFacts::tag` (`level/mod.rs:941`) returns `None` for an absent tag,
and `blocks_lava_fire_spread` does not exist in 26.1.2 data (no tag file,
no `BlockTags` field, no prepared key) — so the walk never stops at a
solid block: fire spreads through stone, and the walk keeps drawing RNG
after the first motion-blocking block, perturbing the random stream vs
vanilla (same second-order effect the pilot noted for 1.21.7). The rest
of `lava_random_tick` (`fluids.rs:750-808`) matches vanilla exactly.

### B1-G07 — Fluid-flow vector table reads the absent `#blocks_fluid_flow` tag (minor, NEW vs 26.1; REPRODUCES 26.1.1 B1-G07)

**Checks failed (2):** `getFlow` vector (below-check input); the
`blocks_fluid_flow := blocks_motion` mapping.

**Vanilla 26.1.2.** `FlowingFluid.getFlow`'s below-check reads
`BlockState.blocksMotion()` (per-state rule: solid except cobweb and
bamboo saplings — `BlockBehaviour$BlockStateBase.blocksMotion` verified in
the jar). There is no `blocks_fluid_flow` tag in 26.1.2 data (no tag file,
no `BlockTags` field, no prepared key).

**The port.** 26.1.2 ships the 26.3 tag form in both places (byte-identical
to 26.1.1's):

- `horizon-client-render/src/assets.rs:224-232` — `Assets::load` reads
  `minecraft:blocks_fluid_flow` from the prepared tag bundle with
  `.ok_or_else(|| invalid("missing fluid flow tag"))?` (it would error
  even earlier on the equally absent `blocks_motion_in_heightmap` tag).
- `horizon-client/src/motion/fluids.rs:27-54` — `FluidResources` builds
  `blocks_flow` from `assets.blocks_fluid_flow: BTreeSet<String>`
  (tag membership by block name) instead of the per-state table.

`fluid_flow::flow_at` (`horizon-world/src/fluid_flow.rs`, byte-identical
to 26.1.1's) consults `blocks_fluid_flow` for its below-check: when a
horizontal neighbour has zero own-height and does NOT block fluid flow,
the flow below it is read instead. With the tag absent, every neighbour
reads as non-blocking, so flow vectors near solid blocks are computed as
if the wall were not there — wrong push directions/magnitudes for entities,
arrows, boats and mobs in water next to solid blocks (all `flow_at`
callers read the same table: `motion/fluids.rs:438`,
`entities/fluids.rs:45,235`, `arrows.rs:1493`, `mobs/living.rs:329`).

**Masking note.** In practice the table is never even built: `Assets::load`
errors on the missing motion tags first (render tests only — see §Method),
and the game's `MotionResources::load` fails on the missing
`#causes_suffocation` / `#suppresses_bounce` tags (the 26.1 run's B3-G22
startup blocker, Batch 3's scope — the tags are absent from 26.1.2 data
too). B1-G07 is the fluid-specific defect underneath: fixing the loader
without restoring the blocksMotion-derived flow table would still leave
every flow vector wrong near solids.

### B1-G08 — Falling-block air drag uses 0.98F widened instead of the double literal 0.98 (minor, NEW vs 26.1; REPRODUCES 26.1.1 B1-G08)

**Checks failed (1):** air drag in `FallingBlockEntity.tick`.

**Vanilla 26.1.2.** `FallingBlockEntity.tick` ends with
`deltaMovement.scale(0.98d)` — the DOUBLE literal (`ldc2_w // double
0.98d`, offset 889, re-verified fresh).

**The port.** 26.1.2's `horizon-client/src/entity_rules.rs:11-14` ships
the 26.2 form (byte-identical to 26.1.1's):

```rust
pub fn scale_air_drag(velocity: Vec3) -> Vec3 {
    let drag = f64::from(0.98_f32);
    ...
}
```

`f64::from(0.98_f32)` is 0.9800000190734863, not 0.98. The test was
renamed to match the wrong form
(`four_ticks_of_air_drag_match_vanilla_0_98f`, expected bits
`0x3fb7_9cd5_1b0c_538e` vs 26.1's `0x3fb7_9cd4_fc35_0910`). Effect size:
after 4 ticks, 0.1 × 0.98⁴ = 0.092236816 vs 0.1 × 0.98F⁴ =
0.09223682318… — a ~7e-9 relative drift per the 26.1 comment; over a
600-tick fall the accumulated relative difference is ~1.2e-5, sub-mm in
position. Tiny but verified-wrong against the bytecode, and 26.1 had it
right. `scale_air_drag` is shared with `PrimedTnt.tick` (out of scope —
flagged for the owning batch in §Method notes).

## Closed — pilot defects re-checked against 26.1.2

All six 1.21.7 pilot defects and both Batch 11 findings were re-checked
against 26.1.2's OWN jar and data. Two are CLOSED (their premise fails on
26.1.2 evidence — landed as closed items B1-G09/B1-G10); the other six
REPRODUCE (see §Defects).

- **G1 bubble columns — CLOSED (B1-G09, NEVER EXISTED for 26.1.2).** All
  three tags EXIST in 26.1.2's data: `enables_bubble_column_push_up`
  (`soul_sand`), `enables_bubble_column_drag_down` (`magma_block`), fluid
  tag `bubble_column_can_occupy` (`water`) — all three read fresh from
  the 26.1.2 client jar. The logic matches vanilla (`level/shapes.rs`
  `bubble_column_can_occupy` 1127, `should_bubble_column_occupy` 1136,
  `try_schedule_bubble_column` 1143 with delay 20 ✓; `level/tick.rs`
  `bubble_column_update` 66/533 matches `updateColumn`; updateShape
  `shapes.rs:270` tick 5; `canSurvive` `level/survive.rs`). Entity motion
  matches exactly (`motion/fluids.rs` `BubbleColumn::push`: surface drag
  max(-0.9, vy−0.03), surface push min(1.8, vy+0.1), inside drag
  max(-0.3, vy−0.03), inside push min(0.7, vy+0.06); push returns
  !surface = reset fall distance). §B 10/10 pass.
- **G2 concrete powder — REPRODUCES as B1-G02** (see §Defects).
- **G3 lava fire spread through solids — REPRODUCES as B1-G06** (see
  §Defects). Was CLOSED in the 26.1 run; the 26.1.1 tree regressed the
  `blocks_motion` stop to the absent-tag read, and the 26.1.2 tree
  inherits it byte-identical.
- **G4 lava ignition ignores doFireTick — CLOSED (B1-G10, NEVER EXISTED
  for 26.1.2).** Vanilla 26.1.2 has NO `doFireTick` rule: `GameRules` has
  exactly 59 `public static final GameRule` fields (counted from the
  javap dump — independently confirming Batch 5's finding), none of them
  `doFireTick`. What exists instead is `FIRE_SPREAD_RADIUS_AROUND_PLAYER`
  (`registerInteger(..., 128, -1)` — default 128, min −1), and the port
  implements it: `level/mod.rs:219-220,234` rule field default 128;
  `game.rs:1560-1574` `can_spread_fire_around` (radius == -1 → true, else
  any fire_player within radius); `game.rs:3847` `fire_players` excludes
  spectators (local player filtered by `!spectator`, LAN guests by
  `here(g) && !g.spectator()`), matching vanilla
  `ServerLevel.canSpreadFireAround` (rule == -1 → true, else
  `ChunkMap.anyPlayerCloseEnoughTo`). §C 15/16 pass (the one gap is
  B1-G06, not the gamerule).
- **G5 no fluid spreading — REPRODUCES as B1-G05** (see §Defects). Was
  CLOSED in the 26.1 run; the 26.1.1 tree regressed the hardcoded
  `canHoldAnyFluid` list to the absent-tag read, and the 26.1.2 tree
  inherits it byte-identical.
- **G6 dripstone never fills cauldrons — REPRODUCES as part of B1-G01**
  (see §Defects).
- **B11-G06 speleothem class name — REPRODUCES as part of B1-G01:** no
  `SpeleothemBlock` in the 26.1.2 jar; `Class::Speleothem` matches nothing.
- **B11-G13 splash water gate — REPRODUCES as B1-G03** (see §Defects).

## Re-checks: 26.1.1 items (PRIMARY)

All 17 items from the 26.1.1 run's slice (`batch-01.json`) were re-checked
against 26.1.2's OWN jar/data + the 26.1.2 tree. **15 REPRODUCE** with
fresh evidence (same IDs, new evidence — see §Defects and §Oracle defects
in scope); **2 land closed again** (B1-G09, B1-G10 — their premises still
fail on 26.1.2 evidence, verified fresh):

| 26.1.1 item | 26.1.2 verdict | 26.1.2 item |
|---|---|---|
| B1-G01 dripstone dead | REPRODUCES | B1-G01 |
| B1-G02 concrete powder | REPRODUCES | B1-G02 |
| B1-G03 splash water | REPRODUCES | B1-G03 |
| B1-G04 fluid eye check | REPRODUCES | B1-G04 |
| B1-G05 no fluid spreading | REPRODUCES | B1-G05 |
| B1-G06 lava fire through solids | REPRODUCES | B1-G06 |
| B1-G07 flow-vector table | REPRODUCES | B1-G07 |
| B1-G08 air drag 0.98F | REPRODUCES | B1-G08 |
| B1-G09 bubble columns [closed] | NEVER EXISTED (closed again) | B1-G09 |
| B1-G10 doFireTick [closed] | NEVER EXISTED (closed again) | B1-G10 |
| B1-O01 BLOCKS_FLUID_FLOW (3 oracles) | REPRODUCES | B1-O01 |
| B1-O02 Prediction (Cauldron) | REPRODUCES | B1-O02 |
| B1-O03 ChunkMap arity (FallingBlock) | REPRODUCES | B1-O03 |
| B1-O04 getHeightForCamera (Fluid) | REPRODUCES | B1-O04 |
| B1-O05 LivingFluid (9 breaks) | REPRODUCES | B1-O05 |
| B1-O06 update void (EntityFluid) | REPRODUCES | B1-O06 |
| B1-O07 getNextEntityId (LavaCauldronItem) | REPRODUCES | B1-O07 |

No 26.1.1 item is FIXED, WITHDRAWN or UNVERIFIED for 26.1.2. The two
26.1.2 ledger items that changed behaviour code (#4
`ServerGamePacketListenerImpl.handleAttack`, #6 Checkbox row-limit
tooltip) are outside Batch 1 scope (combat / GUI) — neither touches
fluids, falling blocks, cauldrons or splash water, and no in-scope file
differs between the trees.

## Withdrawn

- **Cauldron mayInteract candidate — NOT A DEFECT (nuance, §H).**
  Vanilla's `LayeredCauldronBlock.entityInside` runBefore lambda requires
  `entity.isOnFire() && entity.mayInteract(level, pos)` before lowering
  the level (`lambda$entityInside$0`, javap-verified for 26.1.2 — the
  class is byte-identical to 26.1.1's); the port gates on fire_ticks>0 &&
  !spectator and has no mayInteract check. The gate is spawn protection +
  world border (`ServerLevel.mayInteract`: non-player → true; player →
  `!isUnderSpawnProtection && worldBorder.isWithinBounds`), which the
  port's single-player/LAN-host model does not implement as a concept.
  Recorded as a nuance, not a defect — the core behaviour (level
  lowering, powder-snow conversion, extinguish) matches. Same as 26.2,
  26.1 and 26.1.1.
- No defect number was withdrawn in this batch: B1-G01–G08 are all
  attached to defects.

## Passes by section

### §A. Fluid spreading and flow — 18 checks, 15 pass, 3 gap

- `canHoldAnyFluid` hardcoded form (LBC → true; blocksMotion → false;
  DoorBlock / `#signs` / ladder / sugar cane / bubble column / portals /
  end gateway / structure void excluded; else true) — **GAP (B1-G05)**:
  the port reads the absent `#washed_away_by_fluids` tag instead.
- `is_liquid_container` = LiquidBlockContainer OR SimpleWaterlogged —
  pass; required because the prepared export doesn't expand interface
  inheritance (jar: `SimpleWaterloggedBlock extends BucketPickup,
  LiquidBlockContainer` in 26.1.2).
- Direct LiquidBlockContainer blocks: 4 (kelp, kelp_plant, seagrass,
  tall_seagrass) — same as 26.1.1 — pass.
- Waterlogging exhaustive set: **410/410 EXACT MATCH** (vanilla 26.1.2
  `blocks.json` blocks with any `waterlogged` property = 410, counted
  fresh; prepared `block-behaviours.json` blocks with
  `SimpleWaterloggedBlock` interface = 410; zero diffs either way).
  Total states 29,873 confirmed — pass. (Prepared side via the 26.1.1
  prepare + jar-diff keystone; see the header caveat.)
- `#signs` exclusion moot (all signs SimpleWaterlogged, LBC branch
  returns true first) — pass.
- Water numbers: slope 4, dropOff 1, tickDelay 5 — pass (re-verified
  fresh: `iconst_4` / `iconst_1` / `iconst_5`).
- Lava numbers: slope 4 fast / 2 slow, dropOff 1 fast / 2 slow, tickDelay
  10 fast / 30 slow; `isFastLava` via `FAST_LAVA` environment attribute —
  pass.
- Lava `getSpreadDelay` ×4 (both fluids non-empty, neither FALLING, target
  height higher, nextInt(4) != 0) — pass.
- Spread machinery: `spread` / `spreadToSides` / `getNewLiquid` /
  `getSpread` / `getSlopeDistance` / `spread_targets` — pass (vanilla
  `getNewLiquid`: Plane iterator, `canPassThroughWall`, isSource, amount
  max, 2-source + `canConvertToSource` → `getSource`; Rust
  `level/fluids.rs` byte-identical to 26.1.1's verified code). Note: the
  machinery is correct but finds no targets while B1-G05 stands — the
  observable failure is counted once, on the predicate.
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
- `getFlow` vector: 0.8888889f height-difference algorithm, below-check,
  `affectsFlow`, `isSolidFace` (same-family → false, UP → true, ice →
  false, else isFaceSturdy), normalize — **GAP (B1-G07)**: the algorithm
  (`fluid_flow.rs`, byte-identical to 26.1.1's) is right, but its
  below-check input (`blocks_fluid_flow` table) now comes from the absent
  tag, so every neighbour reads as non-blocking.
- `can_convert_to_source` + 2-source rule: `water_source_conversion`
  default true, `lava_source_conversion` default false (GameRules static
  init) — pass.
- `blocks_fluid_flow` := `blocks_motion` mapping — **GAP (B1-G07)**: the
  26.1 mapping is gone; 26.1.2 reads the tag, which does not exist in
  26.1.2 data (no field in `BlockTags`).

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
  (`motion/fluids.rs` `BubbleColumn::push`, lines 253-261, unchanged vs
  26.1.1).
- `bubble_column_at` surface detection + push returns !surface — pass.

### §C. Lava fire, ignition and game rules — 16 checks, 15 pass, 1 gap

- `lava_random_tick` — **GAP (B1-G06)** (see §Defects).
- `lava_flammable` — pass, with the below/above-build-height nuance
  (§Method).
- Fire odds: **207/207 FULL MATCH** (parsed vanilla 26.1.2
  `FireBlock.bootStrap` setFlammable entries fresh vs prepared
  `fire.igniteOdds`/`burnOdds`; zero value diffs, zero only-either-side;
  white_wool (30,60), white_carpet (60,20) spot-checked; the apparent
  208th entry was a parser artifact — the bootStrap prologue's
  `Blocks.FIRE` load, not a setFlammable call) — pass. (Prepared side
  via the 26.1.1 prepare + jar-diff keystone; see the header caveat.)
- `FireBlock.tick`: scheduleTick(getFireTickDelay) first;
  canSpreadFireAround gate → return; canSurvive → removeBlock(0);
  infiniburn check; rain 0.2+0.03*age; age++ min(15, age+nextInt(3)/2)
  setBlock 260; isValidFireLocation; checkBurnOut ×6 (300/300/250/250/
  300/300) — pass.
- `checkBurnOut`: waterlogged → 0; igniteOdds map lookup; nextInt(burnOdds)
  < chance; nextInt(10) < 5, !isRainingAt → fire via getStateWithAge flag
  3, else removeBlock(0); TNT prime — pass.
- `getFireTickDelay`: 30 + nextInt(10) — pass (re-verified fresh:
  `bipush 30`).
- `BaseFireBlock.getState`: below canSurviveOnBlock → SOUL_FIRE else
  FIRE.getStateForPlacement — pass.
- `fireIgnite`: !fireImmune; remaining<0 → setRemainingFireTicks(1); +
  nextInt(1,3) ticks — pass.
- `lavaIgnite`: !fireImmune → igniteForSeconds(15.0) — pass.
- `lavaHurt`: !fireImmune, server-side, hurtServer(lava, 4.0) →
  shouldPlayLavaHurtSound && !isSilent → GENERIC_BURN 0.4 vol, pitch
  2+rand*0.4 — pass on the damage; the sound was NOT traced in the Rust
  (untraced sub-detail, same as 26.2, 26.1 and 26.1.1).
- `LivingEntity.igniteForTicks`: ceil(ticks × BURNING_TIME) →
  Entity.igniteForTicks — pass.
- `LavaCauldronBlock.entityInside`: CLEAR_FREEZE + LAVA_IGNITE +
  runAfter(LAVA_IGNITE, lambda) — pass (same as `LavaFluid.entityInside`).
- `InsideBlockEffectType` mapping: 5 values (FREEZE, CLEAR_FREEZE,
  FIRE_IGNITE, LAVA_IGNITE, EXTINGUISH) exist in 26.1.2; the Rust's
  StepBasedCollector mapping (verified in 26.2 against identical Rust)
  applies — pass.
- `canSpreadFireAround` + spectator exclusion — pass (see G4).
- Infiniburn sets: overworld magma_block + netherrack, nether same, end
  + bedrock (tags verified in 26.1.2 data) — pass.
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
  boatTop)) — pass (Rust `fluid_box.rs`, byte-identical to 26.1.1's).

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

### §F. Falling blocks — 13 checks, 12 pass, 1 gap

- `FallingBlock.tick`: below isFree && y >= minY →
  `FallingBlockEntity.fall` + `falling(entity)` — pass.
- `getDelayAfterPlace`: base 2 (`iconst_2`), `DragonEggBlock` 5
  (`iconst_5`); Rust `hooks.rs:162` uses 5 for DragonEgg, 2 otherwise —
  pass (re-verified fresh).
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
- Air drag: vanilla `deltaMovement.scale(0.98)` — the DOUBLE literal —
  **GAP (B1-G08)**: the port uses `f64::from(0.98_f32)`.
- `causeFallDamage`: hurtEntities gate; ceil(fallDistance)-1;
  NO_CREATIVE_OR_SPECTATOR + LIVING_ENTITY_STILL_ALIVE predicates;
  `Fallable.getFallDamageSource` or `fallingBlock`; min(floor(perDistance),
  max); anvil: damage>0 && nextFloat < 0.05 + damage*0.05 →
  `AnvilBlock.damage` — pass.
- Anvil events: `onLand` !isSilent → levelEvent 1031; `onBrokenAfterFall`
  !isSilent → levelEvent 1029 — pass (Rust `falling.rs:133` emits 1031;
  `falling_damage.rs:95` emits 1029; `game.rs:4482-4483` maps both to
  sounds; vanilla `AnvilBlock` sipush 1031/1029 re-verified fresh).
- `AnvilBlock.damage` chain: ANVIL → CHIPPED_ANVIL → DAMAGED_ANVIL,
  FACING preserved — pass.
- `getFallDamageSource`: `damageSources.anvil(entity)` — pass.
- `game.rs` FallingBlock event consumer (4598-4634): spawns the entity
  at pos+0.5, y, z+0.5, velocity ZERO, drop_item true,
  `Damage::falling(name)`, hurt_per_distance → hurt_entities — pass.
- `Damage::falling` defaults (anvil 2.0 per distance / 40 max) — pass
  (`falling_block.rs`, byte-identical to 26.1.1's).

Out-of-scope observation (not counted): dragon-egg teleport on use
matches vanilla exactly (1000 tries — `sipush 1000` in the 26.1.2 jar,
re-verified fresh, not 16; offsets nextInt(16)-nextInt(16),
nextInt(8)-nextInt(8), nextInt(16)-nextInt(16); target air && below not
air && within world border && inside build height; portal particles along
lerp; setBlock flag 2) — but teleport on use is out of scope for this
batch (falling-block scope covers fall/landing/conversion), same as 26.2,
26.1 and 26.1.1.

### §G. Dripstone and speleothem — 10 checks, 2 pass, 8 gap

Restructured vs 26.2's 26 checks: 26.2's speleothem system works, so its
checks compare live code against vanilla. In 26.1.2 the system is dead
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
  (name-keyed, `game/creatures.rs:5037-5095`).

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
  (`speleothem.rs:335` `cauldron_tick` second half matches exactly;
  vanilla events 1046/1047 re-verified fresh).
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
  `game.rs:8095` `cauldron_extinguish` incl. the
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
live code. In 26.1.2 the gate kills all four outcomes at once (B1-G03), so
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
  fields `{}` (no `blockToGrowOn`), interfaces include
  `SimpleWaterloggedBlock`, `Fallable` — the B1-G01 causes #1 and #3.
- Waterlogging census 410/410 (§A); fire odds census 207/207 (§C).
- `block-behaviours.json` structure: top keys `blocks` (1,168), `states`
  (29,873), `items`, `fluids`, `fire` (shapes/igniteOdds/burnOdds/
  fireDamage=1.0).
- `datagen/reports/registries.json` entries are ALPHABETICAL, not id
  order — sort by `protocol_id` before cumulative state counting
  (total states 29,873 confirms the count).
- Prepared `block-tags.json`: 248 keys, no `speleothems`, no
  `washed_away_by_fluids`, no `blocks_fluid_flow`, no `blocks_motion`,
  no `blocks_lava_fire_spread`, no `blocks_motion_in_heightmap`. Prepared
  tag values are raw JSON strings; nested refs are `#minecraft:<name>`
  (strip leading `#` only when resolving).
- (Prepared-data bullets above are from the 26.1.1 prepare + the jar-diff
  keystone; no 26.1.2 prepare exists yet — see the header caveat.)

## Oracle defects in scope

Authority: the measured Batch 0 table — PROVISIONALLY the 26.1.1 table
(`~/workspace/mcir-work/sweep-26.1.2-ref/26.1.1/batch-0-oracles.json`;
189/391 compile, 202 fail), because the 26.1.2 table had NOT landed at
handoff (checked before finishing). The provisional triage is sound:
all 13 in-scope oracle sources are byte-identical between the 26.1.1 and
26.1.2 trees, no in-scope oracle references any changed class except
`SharedConstants.tryDetectVersion()` (verified present in the 26.1.2 jar),
and every missing symbol below was javap-verified against the 26.1.2 jar
by this batch. The coordinator should re-verify against the 26.1.2 table
when it lands; no verdict is expected to change. 13 in-scope oracles are
present in
`versions/26.1.2/mods/horizon/src/test/java/dev/horizonlab/parity/`.
**9 fail on all three classpaths, 4 pass** (same as 26.1.1's 9 fail / 4
pass; same lines, same error counts).

### B1-O01 — `BlockTags.BLOCKS_FLUID_FLOW` is 26.3-only; breaks 3 oracles (MINOR)

- VanillaFluidFlowOracle: 1 error, line 15, missing-symbol.
- VanillaRegisteredEntityFluidOracle: 2 errors, lines 9–10.
- VanillaRegistryFluidOracle: 3 errors, lines 11, 14 (BLOCKS_FLUID_FLOW;
  line 17 is B1-O04's symbol).
- javap-verified fresh: no `BLOCKS_FLUID_FLOW` field in 26.1.2 `BlockTags`;
  the tag file is absent from 26.1.2 data. Same as 26.1.1 B1-O01, 26.1
  B1-O01 and 26.2 B1-O01. The port deliberately maps blocks_fluid_flow :=
  blocks_motion for 26.1 (the 26.1.2 tree lost that mapping — B1-G07), so
  the repair is to bind the tag from the blocks_motion table, not to add
  the tag.

### B1-O02 — `VanillaCauldronOracle` BROKEN, 1 compile error (MINOR)

- 1 error, line 118, missing-symbol: the `drop()` override references
  `net.minecraft.util.Prediction` — ABSENT from both 26.1.2 jars (jar
  listing verified, 0 matches in either). Same as 26.1.1 B1-O02, 26.1
  B1-O02 and 26.2 B1-O02.

### B1-O03 — `VanillaFallingBlockOracle` BROKEN, 1 compile error (MINOR)

- 1 error, line 30, constructor-mismatch: `NoopChunkMap` calls `super()`
  with 12 args; 26.1.2 `ChunkMap` ctor takes 13 params (ServerLevel,
  LevelStorageAccess, DataFixer, StructureTemplateManager, Executor,
  BlockableEventLoop, LightChunkGetter, ChunkGenerator,
  ChunkStatusUpdateListener, Supplier, TicketStorage, int, boolean —
  javap-verified fresh). Same as 26.1.1 B1-O03, 26.1 B1-O03 and 26.2
  B1-O03; Batch 0 group G6.

### B1-O04 — `VanillaFluidOracle` BROKEN, 1 compile error (MINOR)

- 1 error, line 50, missing-symbol: `state.getHeightForCamera(world, p)`.
  26.1.2 `FluidState` has only `getHeight(BlockGetter, BlockPos)`
  (javap-verified fresh). Same as 26.1.1 B1-O04, 26.1 B1-O04 and 26.2
  B1-O04. (VanillaRegistryFluidOracle line 17 shares the symbol.)

### B1-O05 — `VanillaLivingFluidOracle` BROKEN, 9 compile errors (MINOR)

- 9 errors (same as 26.1.1): lines 30 + 61 `Attributes.FRICTION_MODIFIER`
  and `Attributes.AIR_DRAG_MODIFIER` missing (javap: 26.1.2 `Attributes`
  has only BURNING_TIME and WATER_MOVEMENT_EFFICIENCY among the greps);
  lines 50 + 81 + 97 `FluidTags.ENTITY_FLOATABLE` missing (26.1.2
  `FluidTags` has exactly 6 fields, no ENTITY_FLOATABLE); lines 54 + 85
  `omnidirectionalAirMover` no-override (absent from 26.1.2
  `LivingEntity`).
- Runtime note (carried from the 26.1.1 correction): the oracle looks the
  travel methods up with `getDeclaredMethod` + `setAccessible` (line 88),
  so PRIVATE methods are found. In 26.1.2, `travelInFluid`,
  `travelInWater`, `travelInLava`, `getFluidFallingAdjustedMovement`,
  `jumpOutOfFluid`, `travelInAir`, `handleRelativeFrictionAndCalculateMovement`,
  `handleOnClimbable`, `getFrictionInfluencedSpeed` all EXIST (private) —
  those lookups would succeed. Only `floatInLiquidWhileRidden` and
  `computeModifiedFriction` are absent from `LivingEntity`, so even after
  the compile errors are fixed the oracle would fail at runtime on those
  two lookups (lines 92–93), plus the `ENTITY_FLOATABLE` reference in
  "float" mode (line 97).

### B1-O06 — `VanillaEntityFluidOracle` BROKEN, 1 compile error (MINOR)

- 1 error, line 74, class `other`: `'void' type not allowed here` —
  `interaction.update(...)` used in string concat; 26.1.2
  `EntityFluidInteraction.update` returns **void** (javap-verified fresh).
  Same as 26.1.1 B1-O06 and 26.1 B1-O06. Passed in 26.2 (where the
  oracle's form matched).

### B1-O07 — `VanillaLavaCauldronItemOracle` BROKEN, 1 compile error (MINOR)

- 1 error, line 28, no-override: `getNextEntityId()` does not override —
  absent from both 26.1.2 `ServerLevel` and `Level` (javap-verified
  fresh). Same as 26.1.1 B1-O07 and 26.1 B1-O07. Passed in 26.2.

### Oracles assessed PASSING for 26.1.2 (26.1.1 Batch 0 measurement, provisional)

All three classpaths: VanillaFluidBoxOracle, VanillaFluidOccupancyOracle,
VanillaFluidShapeOracle, VanillaWaterSplashOracle. Their only reference
to a 26.1.1→26.1.2 changed class is the bootstrap call
`SharedConstants.tryDetectVersion()`, verified present in the 26.1.2 jar.

## Systemic findings

The 1.21.7 pilot's systemic root cause — 26.x tag-based logic for tags
absent in the target version's data — applies to 26.1.2 in the same **five**
places as 26.1.1: the speleothem system (B1-G01: `#speleothems` and
`SpeleothemBlock` both absent) and splash water (B1-G03: the four potion
tags absent) as in 26.1, PLUS three regressions where the 26.1.1 tree
undid 26.1's fixes and the 26.1.2 tree inherits byte-identical —
`can_hold_any_fluid` (B1-G05: `#washed_away_by_fluids`), the lava-fire
walk (B1-G06: `#blocks_lava_fire_spread`), and the flow-vector table
(B1-G07: `#blocks_fluid_flow`). A fourth regression is numeric, not
tag-based: the falling-block air drag (B1-G08: 0.98F widened instead of
the double literal). Where 26.1.2's data HAS the tag (bubble-column tags,
`#signs`, `#fire`, `#anvil`, infiniburn), the port reads it; where 26.1.2
still lacks the tag, the port usually carries the hardcoded 1.21.7-style
list form — except in the five places above, where it ships the 26.3 form.
B1-G04 is the same pattern at a smaller scale (26.3-form entity fluid
interaction, camera-height eye check).

Unlike 26.1.1, the 26.1.2 tree is a genuine delta port: the full-tree
diff is 34 files (plus the PORT-NOTES swap), none in Batch 1 scope, and
vanilla itself changed nothing in scope (measured jar diff — see §Method:
only `DetectedVersion`, `SharedConstants`, `ServerGamePacketListenerImpl`
in the server jar; plus `Checkbox` ×3 and `AbstractReportScreen` ×2 in
the client jar). Every 26.1.1-vs-26.1.2 verdict in this batch is therefore
unchanged by construction, not by assumption — and every gap item and
every closed item was still re-verified fresh against the 26.1.2 jar in
this batch.

## Method notes

- **Prepared data NOT available for 26.1.2** (Batch 0's prepare has not
  landed — no `versions/26.1.2/.local`). Two checks depended on prepared
  data in the 26.1.1 run and are counted as passes here on the
  keystone basis: the waterlogging exhaustive set (§A, 410/410) and the
  fire odds values (§C, 207/207). In both cases the VANILLA side was
  counted fresh from 26.1.2's own jar/reports (blocks.json waterlogged
  properties; `FireBlock.bootStrap` setFlammable parse), and the
  prepared side was read from the 26.1.1 prepare, which is valid for
  26.1.2 because the jar diff shows no class in scope changed and all
  five datagen reports are byte-identical. If the coordinator wants
  belt-and-braces, re-run those two censuses against a real 26.1.2
  prepare when it lands; no verdict is expected to change.
- **Vanilla ground truth is 26.1.2's own jar/data only** (owner's
  vanilla-check rule). Every vanilla claim above cites the 26.1.2
  server-game jar's bytecode or the 26.1.2 client jar's data. Where
  26.1.2's form differs from 26.3's (no `washed_away_by_fluids`, no
  `blocks_lava_fire_spread`, no `blocks_fluid_flow`, no `blocks_motion`,
  no `douses_fire`, no `doFireTick`, no `SpeleothemBlock`, no
  `#speleothems`, no sulfur_spike), the port was checked against the
  26.1.2 form.
- **Jar-diff keystone (measured by this batch).** Per-class sha1 diff of
  the named jars, 26.1.1 vs 26.1.2: server-game.jar 7,351 classes, 3
  changed (`DetectedVersion`, `SharedConstants`,
  `ServerGamePacketListenerImpl`); client.jar 10,682 classes, 8 changed
  (those three + `Checkbox`, `Checkbox$Builder`, `Checkbox$OnValueChange`,
  `AbstractReportScreen`,
  `AbstractReportScreen$DiscardReportWarningScreen`); 0 added/removed in
  either; data changes are the 1,202 structure NBTs only (the DataVersion
  bump), plus manifest/signature/version.json. No class in Batch 1 scope
  changed, and no gameplay data file changed. The 26.1.1 run's javap
  findings for those classes therefore carry by measurement — and the key
  methods were still re-verified fresh against the 26.1.2 jar in this
  batch (`canHoldAnyFluid`, lava `randomTick`, `FallingBlockEntity.tick`,
  `AbstractThrownPotion`, `EntityFluidInteraction.update`,
  `PointedDripstoneBlock`, `BlockTags`, `FluidTags`, `ChunkMap`,
  `FluidState`, `Attributes`, `LivingEntity`, `GameRules`,
  `ServerLevel`/`Level`, `FireBlock.bootStrap`). Full evidence:
  `b01/evidence/jardiff/26.1.1-vs-26.1.2.txt`.
- **Datagen diff (measured).** `blocks.json`, `registries.json`,
  `commands.json`, `packets.json`, `datapack.json` and the whole
  `datagen/reports/minecraft/` directory are byte-identical between
  26.1.1 and 26.1.2.
- **Tree diff (measured).** Full-tree `diff -rq` 26.1.1 vs 26.1.2: 35
  lines (34 files differ + the PORT-NOTES-26.1.1/26.1.2 swap) — docs,
  version facts, LAN/net, protocol ids, datafix files, research-lock,
  status/work-items. NONE in Batch 1 scope. All 35 in-scope Rust files
  checked are byte-identical, as are all 13 in-scope oracle sources.
- **Tag inventory verified in 26.1.2's own data** (client jar +
  `BlockTags`/`FluidTags` javap + prepared `block-tags.json`, 248 keys):
  `#speleothems` ABSENT; `washed_away_by_fluids`,
  `blocks_lava_fire_spread`, `blocks_fluid_flow`, `blocks_motion`,
  `blocks_motion_in_heightmap`, `blocks_motion_in_heightmap_no_leaves`,
  `causes_suffocation`, `suppresses_bounce` ABSENT;
  `enables_bubble_column_push_up` / `enables_bubble_column_drag_down`
  EXIST; fluid tags total six (bubble_column_can_occupy, lava,
  supports_frogspawn, supports_lily_pad, supports_sugar_cane_adjacently,
  water); potion tags: only `tradeable` (41 values); item tag `shovels`
  exists, `douses_campfires` does not; block tags `signs` (24 = standing
  + wall; hanging signs 24 separate), `anvil` (3), `fire` (fire,
  soul_fire), `infiniburn_overworld` (magma_block, netherrack),
  `infiniburn_nether` (= overworld), `infiniburn_end` (+ bedrock) all
  exist. No sulfur_spike in 26.1.2 (0 entries in either jar; absent from
  prepared block-behaviours).
- **26.1.2 universe** (established): DataVersion 4790, protocol 775,
  Java 25; 1,168 blocks, 29,873 states, 157 entities, 1,506 items, 117
  particles, 65 biomes, 49 BE types. Registration still lives on
  EntityType/BlockEntityType (no EntityTypes/BlockEntityTypes holder —
  verified by jar listing); bed BE type present.
- **26.1-record corrections (26.1.1 summary §"26.1-record corrections"):
  none apply to Batch 1.** The three corrections concern B3-G33
  (visibility), B8 (providers undercount, Villager.handleEntityEvent
  attribution) and B8-G10/B10-G15/B3-G34 (missed by the 26.1 sweep) —
  all outside this batch's scope. No Batch 1 record needed correction.
- **Nuances recorded, not defects:** (1) Rust `lava_flammable` returns
  false only if inside build height AND not loaded, then checks the flag;
  vanilla returns false whenever NOT (inside && hasChunkAt) — below/above
  build height the Rust proceeds to the flag check (state lookup there is
  presumably air → false anyway). (2) `EntityFluidInteraction.update`
  bool return differs from vanilla only in a 1mm band or for untagged
  mod fluids, and all callers discard it (B1-G04). (3) Cauldron
  mayInteract gate (§Withdrawn). (4) The dead `"minecraft:sulfur_spike"
  => Some(1052)` arm in `finish_falling_block` (harmless, B1-G02).
  (5) 26.1.2 uses "douse" naming vs 26.2 "dowse" (cosmetic, B1-G03).
  (6) `FallingBlockEntity.tick`'s fast-fall clip runs when the delta's
  length-squared is greater than 1.0 (not "non-zero" as the 26.1 report
  put it) — corrected in the 26.1.1 run; the port has no clip check at
  all, so the verdict is unchanged.
- **Untraced sub-details (NOT counted as passes, NOT needs-harness — they
  are sub-details of passing checks or out of scope):** the lava-hurt
  GENERIC_BURN sound (vol 0.4, pitch 2+rand*0.4) was not traced in the
  Rust; `PointedDripstoneBlock.animateTick` drip particles were not
  compared; dragon-egg teleport on use is out of scope (falling-block
  scope covers fall/landing/conversion) — though it was verified matching
  anyway (see §F).
- **Out-of-scope observations (flagged for the coordinator / owning
  batches — NOT B1 items, NOT counted):** all carried from the 26.1.1
  run; every file cited is byte-identical between the trees, and the
  vanilla facts were re-verified where they touch this batch's jars:
  (a) `level/hooks.rs:539` `ice_destroyed_by_player` reads the absent
  `#blocks_motion` tag instead of the `blocksMotion()` rule vanilla
  `IceBlock.playerDestroy` uses — mined ice over solid ground will not
  turn into water in 26.1.2; `hooks.rs:706` also fires a BLOCK_DESTROY
  game event on melt that 26.1.2 vanilla does not (26.3 addition).
  (b) `game/campfire_douse.rs` ships the 26.3 form
  (`CampfireBlock.useItemOn` + `#minecraft:douses_campfires`) instead of
  26.1's `ShovelItem.useOn` + `#minecraft:shovels` + DOWN-face pass;
  `#douses_campfires` does not exist in 26.1.2 data, so shovel-dousing
  a campfire can never fire. (Splash-water campfire dousing is B1-G03's
  downstream code and is a separate path.)
  (c) `worldgen/dripstone.rs` ships the 26.2 `maxOffset` clamp on
  large-dripstone column offsets; 26.1.2 vanilla applies the raw floored
  offset, unclamped (worldgen scope).
  (d) `level/feature_world.rs` heightmap reads the absent
  `#blocks_motion_in_heightmap` / `#blocks_motion_in_heightmap_no_leaves`
  tags instead of the `blocksMotion()` rule (worldgen scope).
  (e) `horizon-client-render/src/assets.rs:load` errors on the missing
  `blocks_motion_in_heightmap` tag (and would then error on
  `blocks_fluid_flow`) — but `Assets::load` is only called from render
  tests/examples, not from the game's production startup path, so this
  breaks resource-loading tests, not the game itself.
  (f) `motion.rs` `MotionResources::load` requires the absent
  `#causes_suffocation` / `#suppresses_bounce` tags — the 26.1 run's
  B3-G22 startup blocker reproduces in 26.1.2 (Batch 3's scope); it
  masks the client-side observable consequences of B1-G07.
  (g) The B1-G08 air-drag regression also affects dropped items
  (`entities.rs`: 26.1's Y drag used the double literal 0.98, 26.1.2
  widens 0.98F) and TNT (`scale_air_drag` is shared with
  `PrimedTnt.tick`) — both out of Batch 1 scope.
- **No cargo builds, no unit tests, no oracle measurement** were run in
  this batch (brief constraints + the shared 6 GB VM). All verdicts are
  javap + source + data comparisons. Oracle triage used the 26.1.1 Batch
  0 table provisionally (the 26.1.2 table had not landed), not a fresh
  measurement.
- **Check-count hygiene.** The 93 checks count only comparisons where
  both sides were actually read. Overlapping sub-checks were not
  double-counted (e.g. cauldron receive vs cauldron_tick are separate
  checks only where the vanilla methods are separate, and the drip-fill
  failure is counted once in §G, not again in §H); dead-code matches
  were not counted as passes (§G, §I); untraced items were not counted
  as passes. Per-section counts: §A 18/15/3, §B 10/10/0, §C 16/15/1,
  §D 7/6/1, §E 7/5/2, §F 13/12/1, §G 10/2/8, §H 8/8/0, §I 4/0/4 —
  summing to 93 checks / 73 pass / 20 gap.

## Evidence index

All under `~/workspace/mcir-work/sweep-26.1.2-batches/b01/evidence/`:

- `jardiff/26.1.1-vs-26.1.2.txt` — per-class sha1 diff (server-game 3
  changed, client 8 changed, 0 added/removed; data: 1,202 structure NBTs
  only) + `jardiff/jardiff.py` (the script).
- `javap/` — 31 `javap -c`/`-p` dumps against the named 26.1.2
  server-game jar: FlowingFluid (canHoldAnyFluid 1012-1075, getFlow
  0.8888889f), LavaFluid (randomTick blocksMotion stop, offset 188),
  WaterFluid (slope 4 / dropOff 1 / tickDelay 5), PointedDripstoneBlock
  (extends Block, 0 BlockTags refs, constants 0.011377778 /
  0.17578125 / 0.05859375), FallingBlockEntity (0.98d ldc2_w offset 889,
  concrete-powder water check + COLLIDER/SOURCE_ONLY clip),
  AbstractThrownPotion (Potions.WATER gates, WATER_SENSITIVE_OR_ON_FIRE,
  rehydrate), EntityFluidInteraction (update void, trackerByFluid, plain
  getHeight), BlockTags (no BLOCKS_FLUID_FLOW / SPELEOTHEMS /
  WASHED_AWAY_BY_FLUIDS / BLOCKS_LAVA_FIRE_SPREAD / BLOCKS_MOTION),
  FluidTags (exactly 6 fields), GameRules (59 fields, no doFireTick,
  FIRE_SPREAD_RADIUS_AROUND_PLAYER), FireBlock (bootStrap 207
  setFlammable entries, tick delay 30+10), ConcretePowderBlock,
  BubbleColumnBlock, Cauldrons (events 1046/1047), FluidState
  (getHeight only), ChunkMap (13-param ctor), Attributes (no
  FRICTION_MODIFIER / AIR_DRAG_MODIFIER), LivingEntity (-p: 9 travel
  methods present private; no omnidirectionalAirMover /
  floatInLiquidWhileRidden / computeModifiedFriction), ServerLevel /
  Level (no getNextEntityId), Entity, LavaCauldronBlock, CampfireBlock
  (dowse), Axolotl, AnvilBlock (events 1031/1029), DragonEggBlock
  (sipush 1000, delay 5), BlockStateBase (blocksMotion), AbstractBoat,
  FallingBlock (delay 2, isFree), IceBlock, BlockBehaviour,
  SharedConstants (tryDetectVersion present).
- `data/tag-files-26.1.2.txt` — client-jar tag file inventory (fluid: 6
  files; potion: tradeable only, 41 values; key block tags present/absent).
- `data/fire-prepared-26.1.1.json` — prepared igniteOdds/burnOdds used
  for the §C census (207/207, 0 mismatches).
- Named jars: `~/workspace/mcir-work/26.1.x/versions/26.1.2/jars/`
  (client.jar sha1 4e618f09…, server-game.jar sha1 83eb1106… — both
  verified before use; server.jar is a bundler jar, never javap'd).
- Prepared data (26.1.1 prepare, keystone-justified):
  `~/workspace/mcir-work/sweep-26.1.1/versions/26.1.1/.local/horizon-client/26.1.1/`
  (block-behaviours.json, block-tags.json).
- 26.1.1 Batch 0 table (provisional oracle authority):
  `~/workspace/mcir-work/sweep-26.1.2-ref/26.1.1/batch-0-oracles.json`.

## Ledger

`slice.json` in this directory — 17 items (B1-G01–G08 gap, B1-G09–G10
closed, B1-O01–O07 oracle-defect), checks 93/73/20.

## Verdict

Batch 1 for 26.1.2 is complete: 93 checks, 73 pass, 20 gap, 17 items.
All 17 of the 26.1.1 run's items are decided — 15 reproduce with fresh
26.1.2 evidence and 2 land closed again — and the 26.1.2 tree adds no
regressions of its own in this scope: it is a genuine delta port of the
26.1.1 tree (34 files differ, none in scope), while vanilla changed
nothing in scope (measured jar diff: only `DetectedVersion`,
`SharedConstants`, `ServerGamePacketListenerImpl`, `Checkbox` and
`AbstractReportScreen` differ). The headline defect stands: no fluid
spreading at all (B1-G05, major), alongside the dead dripstone system
(B1-G01, major) and dead splash water (B1-G03, major). Nothing is
unproven and nothing stalled: every in-scope check was decided statically
against 26.1.2's own jar/data, every oracle failure was javap-verified
against the 26.1.2 jar, and the only provisional element is the oracle
triage's basis — the 26.1.1 Batch 0 table, pending the 26.1.2 table's
landing (no verdict is expected to change, and the coordinator should
re-verify it when it lands).
