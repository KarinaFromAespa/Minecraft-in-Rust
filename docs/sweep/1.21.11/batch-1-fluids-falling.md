# Batch 1 — Fluids + falling blocks (1.21.11 parity sweep)

**Version:** 1.21.11 · **Date:** 2026-11-25 · **Branch:** `parity-sweep-1.21.11`
(workdir `~/workspace/mcir-work/sweep-1.21.11`) ·
**Scope:** `docs/PARITY-SWEEP.md` §4 Batch 1 — the pilot scope re-run against
1.21.11: water/lava spreading and flow, fluid pushing, bubble columns, concrete
powder solidification, lava fire spread + ignition + game rules, dripstone
growth + cauldron fill, falling blocks, cauldron fill levels, splash water.

**Status: DRAFT — not committed.** Checks + triage only, per the brief: no
fixes, no edits under `versions/1.21.11/`, no cargo builds, no oracle
measurement of my own, no commits, no git in the worktree. Everything below
is static analysis: `javap -c` on the named 1.21.11 server-game jar
(`~/workspace/mcir-work/e6-deltas/versions/1.21.11/jars/server-game-named.jar`,
sha1 `7f0d85c12aeb57f14687b7ab2ae2e5d825fbf0d2`, DataVersion 4671,
protocol 774, Java 21), source comparison against
`versions/1.21.11/crates/`, tag/data reads from the named 1.21.11 client jar
(sha1 `29bddf144a2f2907528b25d616770bfea67c4792`; client.jar
`ba2df812c2d12e0219c489c4cd9a5e1f0760f5bd`) and the 1.21.11 datagen
reports. Oracle triage is from the MEASURED 1.21.11 Batch 0 table
(`docs/sweep/1.21.11/batch-0-oracles.json`, generated 2026-11-25, 391
oracles, 102 server-pass) plus the raw big-compile logs
(`b00/evidence/big-compile-server.log`) — the table LANDED during this
batch (checked before finishing), so no oracle verdict here is
provisional. The coordinator commits these deliverables to
`docs/sweep/1.21.11/batch-1-fluids-falling.md` and
`pins/1.21.11/sweep/batch-01.json` (or their chosen paths).

**Prepared-data note.** Unlike the 1.21.10 run, this batch had NO
1.21.11 prepare: no `.local` directory for 1.21.11 exists anywhere in
the worktree or the candidate dirs (checked before writing). Both
censuses (§A waterlogging 410/410, §C fire odds 206/206) were therefore
counted against the real 1.21.10 prepare as the prepared side, with the
equivalence to 1.21.11 established by jar comparison, not assumed: the
vanilla sides were counted FRESH from 1.21.11's own jar/reports
(`blocks.json` waterlogged properties; `FireBlock.bootStrap`
setFlammable parse) and match the 1.21.10 prepared sets exactly, and a
class-hierarchy parse of both jars shows 41 classes changed overall
between 1.21.10 and 1.21.11, ZERO in the block package — the only new
block-package class is the inner `Block$UpdateFlags`. The prepared
1.21.10 SimpleWaterlogged-interface set (410) exactly equals vanilla
1.21.11's waterlogged set, so the census is 410/410 by construction.
This is documented honestly in §A/§C and §Method; the coordinator should
re-count against a real 1.21.11 prepare when one exists, but no verdict
depends on the prepared side alone — every defect's cause is a
source-vs-jar comparison, not a prepared-data comparison.

**Provenance note.** The 1.21.11 tree is the E6 ERA TEMPLATE itself
(coordinator Step 0, PROVENANCE verified), NOT a delta port and NOT
derived from 26.x — the 1.21.10 child was derived FROM this tree. That
inverts the usual re-check direction: where a 1.21.10 defect reproduces
here, the defect lives in the template and was inherited DOWN by the
child, not the other way round. No verdict below is carried from any
earlier run: every re-check was decided fresh against 1.21.11's own
jar/data + the 1.21.11 tree, and the tree genuinely differs from the
1.21.10 child in scope (the fire gate implements the 1.21.11 radius
rule, the burnout gate reads the environment attribute that vanilla
1.21.11 itself uses). Diffed against the 1.21.10 tree, the Batch 1
files are code-identical except `game.rs`'s fire gate — all other
diffs in scope are version-number comments only.

## Tally

- **Behaviour checks: 95 — 78 pass, 17 gap** (all behaviour; non-behaviour
  0. As-tallied 95 = 95 behaviour + 0 non-behaviour). The 17 gap checks are
  the eight defect items below: B1-G01 × 5, B1-G02 × 2, B1-G03 × 4,
  B1-G04 × 1, B1-G05 × 1, B1-G06 × 1, B1-G07 × 2, B1-G08 × 1;
  see §Defects. (One gap fewer than 1.21.10's 18: B1-G09 withdrew — see
  §Closed.)
- **Defect items: 8** — B1-G01 (major), B1-G02 (minor), B1-G03 (major),
  B1-G04 (minor), B1-G05 (major), B1-G06 (minor), B1-G07 (minor),
  B1-G08 (minor).
- **Closed items: 1** (B1-G09 — WITHDRAWN: vanilla 1.21.11 replaced the
  biome tag with an environment attribute, and the port already reads the
  attribute; see §Closed).
- **Fixed items: 2** (B1-G10, B1-G11 — 1.21.7 pilot defects G1 and G4,
  verified absent in the 1.21.11 tree; see §Fixed).
- **Oracle defects: 9** (B1-O01–B1-O09, all minor — see §Oracle defects in
  scope: 11 of the 13 in-scope oracles fail, 2 pass). Not counted in the
  check tally, following the 1.21.7 convention. Measured against the
  landed 1.21.11 Batch 0 table, not provisional.
- **Needs-harness: none** — no B1-Hxx items. Every in-scope check was
  decidable statically; the few untraced sub-details are noted in §Method,
  not faked as passes.
- **Withdrawn candidates:** one number (B1-G09, closed — see §Closed)
  plus one nuance (the cauldron mayInteract gate — not a defect; see
  §Withdrawn).
- **1.21.10 re-checks (§Re-checks: 1.21.10 — PRIMARY):** all 21 DECIDED —
  18 REPRODUCE (B1-G01–G08, B1-O01–O08, B11-G06, B11-G13), 1 WITHDRAWN
  (B1-G09 → closed), 2 FIXED again (B1-G10, B1-G11). No NEVER EXISTED, no
  UNVERIFIED. B11-G06 and B11-G13 reproduce with fresh evidence but are
  NOT minted here — Batch 11 owns that namespace; they are flagged for
  its ledger in §Re-checks and §Method.
- **1.21.7 re-checks (§Fixed + §Defects — SECONDARY):** all 8 DECIDED — 2
  FIXED (G1 as B1-G10, G4 as B1-G11), 6 REPRODUCE (G2 as B1-G02, G3 as
  B1-G06, G5 as B1-G05, G6 and B11-G06 as parts of B1-G01, B11-G13 as
  B1-G03). No NEVER EXISTED, no WITHDRAWN, no UNVERIFIED.

## Defects

### B1-G01 — Pointed-dripstone system half dead: transfer, growth, cauldron fill, thickness, falling stalactites (major, REPRODUCES pilot G6 + B11-G06 in narrower form; REPRODUCES 1.21.10 B1-G01 in narrower form)

**Checks failed (5):** random-tick transfer; random-tick growth; cauldron
fill from dripstone; thickness calc (placement + updateShape); falling
stalactite spawn.

**Vanilla 1.21.11.** There is NO `SpeleothemBlock` class in the 1.21.11 jar
at all (0 matches in either jar): `PointedDripstoneBlock extends Block`
directly (implements `SimpleWaterloggedBlock`, `Fallable`) — the 1.21.7
shape, not 26.2's. Its bytecode contains ZERO `BlockTags` references:
every predicate is same-block (`isPointedDripstoneWithDirection`,
`isValidPointedDripstonePlacement`: face-sturdy behind OR same-block
same-direction). `PDB.canGrow` hardcodes `Blocks.DRIPSTONE_BLOCK` + a
water source at above(2) — no `blockToGrowOn` field exists in 1.21.11.
`calculateDripstoneThickness` uses same-block checks only (TIP /
TIP_MERGE / FRUSTUM / MIDDLE / BASE). Constants re-verified fresh:
growth 0.011377778, transfer 0.17578125 water / 0.05859375 lava,
findTip max 11.

**What the 1.21.11 tree (the template) HAS vs the 26.x trees.** The
dispatch was adapted IN THE TEMPLATE: all five dispatch sites arm on
`Class::Speleothem | Class::PointedDripstone` — `place.rs:1156`
(placement), `shapes.rs:745` (updateShape), `survive.rs:326`
(canSurvive), `tick.rs:257` (scheduled tick), `tick.rs:443` (random
tick) — so placement, updateShape, canSurvive, scheduled-tick and
random-tick dispatch FIRE for pointed_dripstone. And
`is_valid_speleothem_placement` (`survive.rs:600`) is NOT tag-gated
(face-sturdy OR same-block same-direction, matching vanilla
`isValidPointedDripstonePlacement`), so canSurvive works, placement
direction works (`calculateTipDirection` via the nearest looking vertical
direction), updateShape survival scheduling works (scheduleTick 2/1),
and the scheduled tick destroys unsupported stalagmites. Those are §G
passes — the 26.1.2 run's B1-G01 cause #1 (dead dispatch) is gone, and
the 1.21.10 child inherited the fix from this template. (The 1.21.7
B11-G06 evidence note "Dead in the 1.21.11 template too" is outdated
for the dispatch arms — the template was adapted; what remains dead is
below.)

**What is still dead — two independent causes:**

1. **Dead tag.** `speleothem_tag()` reads `#minecraft:speleothems`
   (`speleothem.rs:13`) — absent from 1.21.11 data (no `speleothems.json`
   in the client jar, no `SPELEOTHEMS` field in `BlockTags`, no key in
   the prepared tag bundle) — so `speleothem_with`
   (`speleothem.rs:18`), `speleothem_is_tip`,
   `is_free_hanging_stalactite` (`speleothem.rs:29`) are all always
   false, and `falling.rs:51` `spawn_falling_stalactite` is tag-gated
   too. The thickness calc is tag-gated the same way: `shapes.rs:1176`
   `speleothem_thickness` and `place.rs:1398`
   `speleothem_thickness_for` build their `is_with` closures from the
   absent tag (`shapes.rs:1186`), so every `is_with` is false and the
   thickness is always `"tip"` — a placed or updated dripstone never
   becomes frustum/middle/base/tip_merge.
2. **Missing link.** Prepared `fields` for pointed_dripstone is `{}` —
   no `blockToGrowOn`; `data.link` resolves to None, so
   `speleothem_grow`'s `can_grow` is always false even if reached
   (`speleothem.rs:96`). (Prepared side is the 1.21.10 prepare; the
   1.21.11 jar has no `blockToGrowOn` field either, and 0 classes
   changed in the block package — see the header note.)

**Observable consequences** (each a §G gap check): random-tick transfer
never runs (`dripstone_transfer_fluid`, `speleothem.rs:277`, requires
`speleothem_with` Down); growth never runs (`speleothem_random_tick`,
`speleothem.rs:84`, requires `speleothem_with` at its start AND the
missing link); cauldron fill never runs (`cauldron_tick`,
`speleothem.rs:338`, requires `is_free_hanging_stalactite`); thickness
is always tip; falling stalactites never spawn (the scheduled tick's
destroy arm runs, but the spawn arm is tag-dead).

**NOT dead (verified working):** placement direction, updateShape
survival scheduling, canSurvive, scheduled-tick destroy, `fallOn`
stalagmite damage (`block_landing.rs` `FallDefinition::PointedDripstone`
keyed via `landing-definitions.json`, Rule::DripstoneTip (up + tip →
Stalagmite, distance+2.5, ×2.0) matches vanilla `PDB.fallOn`), trident
break (`game/creatures.rs:5095` name match
`"minecraft:pointed_dripstone"` + Trident + hit_speed > 0.6 → destroy,
with the `projectile_breaks_block` gates matching vanilla
`onProjectileHit`).

**Severity rationale:** five observable behaviours dead — transfer,
growth, cauldron fill, thickness, falling stalactites — a large part of
the dripstone feature set, though narrower than the 26.x run's
whole-system framing. The dead-code constants in `speleothem.rs` match
vanilla exactly (growth `0.011_377_778` at line 92, transfer
`0.175_781_25` / `0.058_593_75` at lines 279–292, findTip max 11,
cauldron delay 50), but they were NOT counted as passes (§G counts
observable behaviours only).

### B1-G02 — Concrete powder falling into water does not solidify on contact (minor, REPRODUCES pilot G2; REPRODUCES 1.21.10 B1-G02)

**Checks failed (2):** first-water-cell landing check; fast-fall clip check.

**Vanilla 1.21.11.** `FallingBlockEntity.tick` (server-game jar, re-verified
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
(line 128–133). So powder sinks to the lakebed and lands there as powder.
The horizon-world landing branch itself (`level/falling.rs`
`falling_block_landed`) matches vanilla, including the `onLand` solidify
check (`falling.rs:127`: water fluid at current OR
`concrete_touches_liquid`, via the data link) and the
`would_continue = isFree(below) && (!concrete || !stuck_in_water)` logic —
the fix is local to the client fall step. Same defect as 26.2's B1-G02,
26.1's B1-G02, 26.1.1's B1-G02, 26.1.2's B1-G02 and 1.21.10's B1-G02.
Note: `finish_falling_block` also has a dead `"minecraft:sulfur_spike" =>
Some(1052)` arm (harmless — no sulfur_spike in 1.21.11; the 1052 sound
event does not exist for this version).

### B1-G03 — Splash water never douses fire / hurts / extinguishes / rehydrates (major, REPRODUCES B11-G13; REPRODUCES 1.21.10 B1-G03)

**Checks failed (4):** dowse fire on block hit; hurt water-sensitive
entities; extinguish burning entities; rehydrate axolotls.

**Vanilla 1.21.11.** `AbstractThrownPotion.onHitBlock` gates on
`PotionContents.is(Potions.WATER)` — Holder equality, NOT a tag
(re-verified fresh; `Potions.WATER` getstatic at two gate sites, zero
`douses_fire` / `PotionTags` references in the class). Dowse targets:
hitPos.relative(dir), hitPos itself, and the 4 horizontal neighbours of
hitPos.relative(dir). `dowseFire`: `#fire` tag → destroyBlock(pos, false,
this); else lit candle → extinguish; else lit campfire → level event
1009 + `CampfireBlock.dowse` (verified to exist in the 1.21.11 jar).
`onHitAsWater`: LivingEntity in the potion's box inflated (4.0, 2.0, 4.0)
with WATER_SENSITIVE_OR_ON_FIRE (isSensitiveToWater || isOnFire),
distanceToSqr < 16.0; water-sensitive → hurtServer(indirectMagic(owner,
this), 1.0); on fire && alive → extinguishFire; Axolotl in the same box
→ `rehydrate()` (air = min(air + 1800, maxAir), re-verified fresh in the
1.21.11 `Axolotl` class).

**The port.** 1.21.11's `game/splash_water.rs` ships the 26.3 tag form:
`potion_in_tag(stack, "minecraft:douses_fire")`,
`"minecraft:hurts_water_sensitive_entities"`,
`"minecraft:extinguishes_entities"`,
`"minecraft:rehydrates_axolotls"` — read from `trades.potion_tags`,
built from the prepared data's `tags/potion` directory. That directory
is EMPTY for 1.21.11 — zero potion tag files in the client jar (verified
fresh; 26.1.2 at least had `tradeable.json`) — so all four gates are
always closed and no splash water ever does anything. 26.2's file uses
`is_water_potion` (Potions.WATER equality) and works. The downstream code
itself matches vanilla exactly (dowse targets, dowseFire, the (4,2,4)
box, dist²<16, hurt 1.0 indirectMagic, Enderman teleport counter, axolotl
+1800 air capped at MAX_AIR) — it is verified matching but unreachable,
so it is recorded as analysis, NOT counted as passes (§I counts the four
observable outcomes).

**Severity rationale:** all splash-water functionality is dead — four
distinct observable behaviours, a whole mechanic, same scope class as
B1-G01.

### B1-G04 — Entity fluid interaction ships the 26.3 form; eye check uses camera height (minor; REPRODUCES 1.21.10 B1-G04)

**Checks failed (1):** eye-in-fluid height check.

**Vanilla 1.21.11.** The CLASSIC 1.21.x form — there is NO
`EntityFluidInteraction` class at all in the 1.21.11 jar (0 matches in
either jar). `Entity.updateFluidHeightAndDoFluidPushing(TagKey, double)`
is a plain method on Entity: touchingUnloadedChunk gate, bbox.deflate
(0.001), floor/ceil bounds, per-cell loop (x outer, y middle, z inner),
top = y + plain `FluidState.getHeight`, height = max(top − minY),
flow scaled by the running max height if < 0.4, accumulate; tail:
count>0 → scale 1/count, non-Player normalize, scale by speed, min-push
rule (|dx|<0.003 && |dz|<0.003 && length<0.0045 →
normalize.scale(0.0045)). `updateFluidOnEyes` is a separate single-cell
check: top = eye-cell y + plain `getHeight`, top > eyeY → tags added
(with a boat override: vehicle is boat, not underwater, boat bbox spans
eyeY → return). `updateInWaterStateAndDoWaterCurrentPushing` short-circuits
the whole water update for boat passengers (vehicle is boat NOT
underwater → `wasTouchingWater=false`, water update skipped; the lava
update still runs with the full box). Speeds: water 0.014, lava fast
0.007 / slow 0.0023333333333333335 (FAST_LAVA environment attribute).
`isInLava` = !firstTick && fluidHeight(LAVA) > 0.

**The port.** 1.21.11's `horizon-world/src/entity_fluid.rs` ships the
26.3 form: per-kind trackers + separate CurrentAccumulators, `update`
returns `Result<bool>`, tracker pruning. Most of that is behaviourally
equivalent for vanilla fluids (see §D) — but the eye check is not:
`entity_fluid.rs:389-401` computes the top with
`world.camera_height(state, p)` (`fluid.rs` `camera_height_at`: a source
fluid with a sturdy block above → 1.0, else `height_at`) instead of the
plain top. Observable: an entity whose eye is in (0.889, 1.0] above a
source's block bottom with a solid ceiling → the port says
eyes-in-fluid, vanilla 1.21.11 says not. Affects `was_eye_in_water` /
`is_under_water` (swimming, overlay, air/drowning).

**Verified NOT divergences** (analysis, not gaps): flow scaling — the
Rust scales by the running max height, same value, same order as vanilla;
`apply_to` has NO 1e-5 floor (the comment says the 26.x floor is gone;
matches vanilla's length > 0); `fluid_height`/`is_in_fluid`/
`is_eye_in_fluid` — same values; tracker pruning — internal only. The
boat interaction-box clip (`motion/fluids.rs` `interaction_box`, whose
comment explicitly addresses this) is a NUANCE, not a gap: vanilla has
no box override but short-circuits the water update for boat passengers
instead, and the port's clip gives the same answer for water; the
lava-path asymmetry (port clips, vanilla doesn't) is unobservable since
boats don't survive lava. `fluid_box.rs` `boat_passenger_box` is used
only by the probe fixture, not production.

### B1-G05 — NO fluid spreading at all, water or lava (major, REPRODUCES pilot G5 — was "fixed" in the 1.21.7 ledger; REPRODUCES 1.21.10 B1-G05)

**Checks failed (1):** `canHoldAnyFluid`. **Re-checked FIRST per the brief.**

**Vanilla 1.21.11.** `FlowingFluid.canHoldAnyFluid` bytecode read in full,
fresh against the 1.21.11 server-game jar (`FlowingFluid.txt:1012-1075`):
hardcoded list — LiquidBlockContainer → true; blocksMotion → false;
excludes DoorBlock, `#signs`, ladder, sugar cane, bubble column, nether/end
portals, end gateway, structure void; else true (air passes). Zero
`washed` references anywhere in `FlowingFluid`. `washed_away_by_fluids`
is absent from 1.21.11 data (no tag file in the client jar, no field in
`BlockTags`, no key in the prepared tag bundle).

**The port.** 1.21.11's `level/fluids.rs:359-361` ships the 26.3 tag form:

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
`should_spread_liquid` / `liquid_block_changed`) matches vanilla exactly
— the failure is isolated to the predicate, so those checks still pass
as code comparisons (same convention as B1-G02, where the matching
landing branch counts as pass and only the caller is gap). The observable
"fluid spreads" failure is counted once, on the predicate check.

**Note on G5's ledger status.** G5's status in the 1.21.7 merged ledger
is "fixed", but the 1.21.11 template carries the tag form — the fix
never reached the template, and the 1.21.10 child inherited the defect
from it. Recorded explicitly: this is a REPRODUCES, not a carry of the
ledger's fixed status.

**Severity rationale:** a whole mechanic dead — water and lava never move.
Same scope class as the pilot's G5 (major).

### B1-G06 — Lava fire spread walks through solid blocks (minor, REPRODUCES pilot G3 — was CLOSED in 26.1; REPRODUCES 1.21.10 B1-G06)

**Checks failed (1):** `lava_random_tick`.

**Vanilla 1.21.11.** `LavaFluid.randomTick` bytecode re-verified fresh:
gates on `ServerLevel.canSpreadFireAround(pos)` (the radius rule — see
B1-G11); `nextInt(3)` passes, first branch cumulative offset(nextInt(3)-1,
1, nextInt(3)-1), isLoaded else return, air + hasFlammableNeighbours →
setBlockAndUpdate(BaseFireBlock.getState) + return, **`blocksMotion` →
return** (offset 115–123); else branch 3 iters offset(dx,0,dz) from
origin, isEmptyBlock(above) && isFlammable(test) →
setBlockAndUpdate(above(test), getState(level, test)).

**The port.** 1.21.11's `level/fluids.rs:776-785` stops only for the tag:

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
and `blocks_lava_fire_spread` does not exist in 1.21.11 data (no tag file,
no `BlockTags` field, no prepared key) — so the walk never stops at a
solid block: fire spreads through stone, and the walk keeps drawing RNG
after the first motion-blocking block, perturbing the random stream vs
vanilla (same second-order effect the pilot noted for 1.21.7). The rest
of `lava_random_tick` (`fluids.rs:747-808`) matches vanilla exactly,
including the radius-rule gate (see B1-G11).

### B1-G07 — Fluid-flow vector table reads the absent `#blocks_fluid_flow` tag (minor, NEW vs 26.1; REPRODUCES 1.21.10 B1-G07)

**Checks failed (2):** `getFlow` vector (below-check input); the
`blocks_fluid_flow := blocks_motion` mapping.

**Vanilla 1.21.11.** `FlowingFluid.getFlow`'s below-check reads
`BlockState.blocksMotion()` (per-state rule: solid except cobweb and
bamboo saplings — `BlockBehaviour$BlockStateBase.blocksMotion` verified
in the jar). There is no `blocks_fluid_flow` tag in 1.21.11 data (no tag
file, no `BlockTags` field, no prepared key).

**The port.** 1.21.11 ships the 26.3 tag form in both places:

- `horizon-client-render/src/assets.rs:224-232` — `Assets::load` reads
  `minecraft:blocks_fluid_flow` from the prepared tag bundle with
  `.ok_or_else(|| invalid("missing fluid flow tag"))?` (it would error
  even earlier on the equally absent `blocks_motion_in_heightmap` tag).
- `horizon-client/src/motion/fluids.rs:27-54` — `FluidResources` builds
  `blocks_flow` from `assets.blocks_fluid_flow: BTreeSet<String>`
  (tag membership by block name) instead of the per-state table.

`fluid_flow::flow_at` (`horizon-world/src/fluid_flow.rs`) consults
`blocks_fluid_flow` for its below-check: when a horizontal neighbour has
zero own-height and does NOT block fluid flow, the flow below it is read
instead. With the tag absent, every neighbour reads as non-blocking, so
flow vectors near solid blocks are computed as if the wall were not
there — wrong push directions/magnitudes for entities, arrows, boats and
mobs in water next to solid blocks (all `flow_at` callers read the same
table: `motion/fluids.rs:446`, `entities/fluids.rs:45,232`,
`arrows.rs:1490`, `mobs/living.rs:331`).

**Masking note.** `Assets::load` is only called from render
tests/examples, not from the game's production startup path (verified:
every non-test caller is a bench/example/probe), so the hard error breaks
resource-loading tests, not the game itself. But the game's
`MotionResources::load` fails on the missing `#causes_suffocation` /
`#suppresses_bounce` tags (the 26.1 run's B3-G22 startup blocker —
reproduces in 1.21.11, Batch 3's scope; `motion.rs:142,145` require them
with `?`, and both are absent from the prepared tag bundle).
B1-G07 is the fluid-specific defect underneath: fixing the loader
without restoring the blocksMotion-derived flow table would still leave
every flow vector wrong near solids.

### B1-G08 — Falling-block air drag uses 0.98F widened instead of the double literal 0.98 (minor, NEW vs 26.1; REPRODUCES 1.21.10 B1-G08)

**Checks failed (1):** air drag in `FallingBlockEntity.tick`.

**Vanilla 1.21.11.** `FallingBlockEntity.tick` ends with
`deltaMovement.scale(0.98d)` — the DOUBLE literal (`ldc2_w // double
0.98d`, offset 889, re-verified fresh).

**The port.** 1.21.11's `horizon-client/src/entity_rules.rs:11-14` ships
the 26.2 form:

```rust
pub fn scale_air_drag(velocity: Vec3) -> Vec3 {
    let drag = f64::from(0.98_f32);
    ...
}
```

`f64::from(0.98_f32)` is 0.9800000190734863, not 0.98. Effect size:
after 4 ticks, 0.1 × 0.98⁴ = 0.092236816 vs 0.1 × 0.98F⁴ =
0.09223682318… — a ~7e-9 relative drift per the 26.1 comment; over a
600-tick fall the accumulated relative difference is ~1.2e-5, sub-mm in
position. Tiny but verified-wrong against the bytecode, and 26.1 had it
right. `scale_air_drag` is shared with `PrimedTnt.tick` (out of scope —
flagged for the owning batch in §Method notes).

## Closed — re-check withdrawn

### B1-G09 — Fire burnout — WITHDRAWN: vanilla 1.21.11 changed the mechanism to the form the port already ships (CLOSED)

**Checks failed (0).** The §C fire-burnout check PASSES for 1.21.11.

**Vanilla 1.21.11 changed the mechanism.** 1.21.10's `FireBlock.tick`
read the `#increased_fire_burnout` BIOME tag — that is what 1.21.10's
B1-G09 was about (the port read the 26.x environment attribute instead,
always false in 1.21.10). In 1.21.11 the biome tag is GONE (no
`BiomeTags.INCREASED_FIRE_BURNOUT` field, no tag file in the client
jar) and `FireBlock.tick` reads the environment attribute instead:
`EnvironmentAttributes.INCREASED_FIRE_BURNOUT` via
`environmentAttributes().getValue(...)` (javap-verified fresh). All 8
biomes (bamboo_jungle, mushroom_fields, mangrove_swamp, snowy_slopes,
frozen_peaks, jagged_peaks, swamp, jungle) set
`minecraft:gameplay/increased_fire_burnout: true` in their biome JSON;
plains (control) does not. The effect is unchanged in form: `extra =
burnout ? −50 : 0` on the six checkBurnOut chances (300/300/250/250/
300/300), and the spread odds are halved in burnout biomes
(`odds = (ignite + 40 + difficulty*7)/(age+30)`, `if (burnout) odds /= 2`).

**The port already matches.** `horizon-client-render/src/biome.rs:226`
reads `attributes["minecraft:gameplay/increased_fire_burnout"]` from the
biome's JSON, consumed at `game.rs:1577` `increased_fire_burnout` and
`tick.rs:678-686` (`let extra = if burnout { -50 } else { 0 }`, applied
to the 300/300/250/250/300/300 chances) plus the `odds /= 2` halving at
`tick.rs:704` — the exact vanilla form, gate and constants. The same
code that WAS the defect in 1.21.10 is CORRECT in 1.21.11, because
vanilla moved to the attribute form between the two versions.

**Verdict:** the 1.21.10 item's premise ("vanilla reads the biome tag,
the port reads the attribute") fails on 1.21.11 evidence — WITHDRAWN,
landed as a closed item per common.md. This is the one re-check where
vanilla moved toward the template rather than the template being wrong.

## Fixed — pilot defects re-checked against 1.21.11

All six 1.21.7 pilot defects and both Batch 11 findings were re-checked
against 1.21.11's OWN jar and data (SECONDARY). Two are FIXED (the defect
is verified absent in the 1.21.11 tree — landed as fixed items B1-G10/B1-G11);
the other six REPRODUCE (see §Defects).

- **G1 bubble columns — FIXED (B1-G10).** The 1.21.11 tree ships the
  hardcoded 1.21.11 form: `bubble_column_can_exist_in`
  (`shapes.rs:1149`, source water — vanilla `canExistIn`), `getColumnState`
  hardcoded soul sand / magma (`tick.rs:551-590` — vanilla
  `getColumnState`, no tags; the comment in the 1.21.11 tree names
  1.21.11), `canSurvive` hardcoded (`survive.rs:79`),
  Magma/SoulSand updateShape schedules tick 20 (`shapes.rs:621-638`,
  vanilla `tryScheduleBubbleBlockColumn`), tick arms (`tick.rs:64-82`),
  updateShape tick 5 (`shapes.rs:264`). Vanilla 1.21.11 uses no tags for
  any of this (0 tag refs in `BubbleColumnBlock`; the
  `enables_bubble_column_*` / `bubble_column_can_occupy` tags do not
  exist in 1.21.11 data). Entity motion matches exactly
  (`motion/fluids.rs` `BubbleColumn::push`: surface drag max(-0.9,
  vy−0.03), surface push min(1.8, vy+0.1), inside drag max(-0.3,
  vy−0.03), inside push min(0.7, vy+0.06); push returns !surface =
  reset fall distance). §B 10/10 pass. Also covers 1.21.10 B1-G10
  (fixed again — see §Re-checks).
- **G2 concrete powder — REPRODUCES as B1-G02** (see §Defects).
- **G3 lava fire spread through solids — REPRODUCES as B1-G06** (see
  §Defects).
- **G4 lava ignition ignores doFireTick — FIXED (B1-G11).** Vanilla
  1.21.11 REPLACED the rule pair: `GameRules` (now in
  `net.minecraft.world.level.gamerules`, 59 fields, no `RULE_` prefix)
  has NO doFireTick pair at all — instead `FIRE_SPREAD_RADIUS_AROUND_PLAYER`
  (Integer, default 128, min −1), and `LavaFluid.randomTick` gates only
  on `ServerLevel.canSpreadFireAround(pos)`, which reads the radius
  rule (−1 → true, else `ChunkMap.anyPlayerCloseEnoughTo(pos, radius)`;
  spectators excluded, `distance < radius`). The port implements the
  radius form directly: `game.rs:1562` `can_spread_fire_around`
  (`radius == -1 || any fire_player with dist² < radius²`, to the
  block's corner), default 128 at `save.rs:3548`; `game.rs:3847`
  `fire_players` excludes spectators (local player filtered by
  `!spectator`, LAN guests by `here(g) && !g.spectator()`), matching
  vanilla. §C 16/17 pass (the one gap is B1-G06, not the gamerule).
  Also covers 1.21.10 B1-G11 (fixed again — see §Re-checks). Note the
  form change: the 1.21.10 item's "doFireTick pair" framing does not
  apply to 1.21.11 at all — the underlying fact (lava ignition respects
  the fire rules) holds under the new rule.
- **G5 no fluid spreading — REPRODUCES as B1-G05** (see §Defects). Note:
  G5's status in the 1.21.7 ledger is "fixed", but the 1.21.11 template
  carries the tag form — the fix never reached the template.
- **G6 dripstone never fills cauldrons — REPRODUCES as part of B1-G01**
  (see §Defects).
- **B11-G06 speleothem class name — REPRODUCES as part of B1-G01 in
  narrower form:** the dispatch cause is FIXED (the PointedDripstone
  arms fire), but the transfer/growth/cauldron-fill/thickness/falling
  causes remain — see B1-G01.
- **B11-G13 splash water gate — REPRODUCES as B1-G03** (see §Defects).

## Re-checks: 1.21.10 items (PRIMARY)

All 21 items from the 1.21.10 run's slice (`batch-01.json` plus the two
Batch 11 items it flagged) were re-checked against 1.21.11's OWN jar/data
+ the 1.21.11 tree. **18 REPRODUCE** with fresh evidence (same IDs, new
evidence — see §Defects and §Oracle defects in scope); **1 WITHDRAWN**
(B1-G09 → closed, see §Closed); **2 FIXED again** (B1-G10, B1-G11 — see
§Fixed). No 1.21.10 item is NEVER EXISTED or UNVERIFIED for 1.21.11.
Note the direction: the 1.21.11 tree is the TEMPLATE the 1.21.10 child
was derived from, so a reproduction here means the defect lives in the
template and was inherited DOWN — the 1.21.10 run's defects are the
template's own, except B1-G09 (where the child inherited template code
that was wrong for 1.21.10's older vanilla form but is right for
1.21.11's).

| 1.21.10 item | 1.21.11 verdict | 1.21.11 item |
|---|---|---|
| B1-G01 dripstone dead (narrower) | REPRODUCES (same narrower form) | B1-G01 |
| B1-G02 concrete powder | REPRODUCES | B1-G02 |
| B1-G03 splash water | REPRODUCES | B1-G03 |
| B1-G04 fluid eye check | REPRODUCES | B1-G04 |
| B1-G05 no fluid spreading | REPRODUCES | B1-G05 |
| B1-G06 lava fire through solids | REPRODUCES | B1-G06 |
| B1-G07 flow-vector table | REPRODUCES | B1-G07 |
| B1-G08 air drag 0.98F | REPRODUCES | B1-G08 |
| B1-G09 fire burnout | WITHDRAWN (vanilla changed to the attribute form) | B1-G09 (closed) |
| B1-G10 bubble columns [fixed] | FIXED again | B1-G10 |
| B1-G11 doFireTick [fixed] | FIXED again (radius-rule form) | B1-G11 |
| B1-O01 BLOCKS_FLUID_FLOW (3 oracles) | REPRODUCES (+ FluidFlow ctor break) | B1-O01 |
| B1-O02 Prediction (Cauldron) | REPRODUCES (2 errors: + ServerLevel ctor) | B1-O02 |
| B1-O03 ChunkMap arity (FallingBlock) | REPRODUCES | B1-O03 |
| B1-O04 getHeightForCamera (Fluid) | REPRODUCES (2 errors: + isFull line 26) | B1-O04 |
| B1-O05 LivingFluid (9 breaks) | REPRODUCES (12 errors; runtime 2 lookups fail vs 5) | B1-O05 |
| B1-O06 EntityFluid (class absent) | REPRODUCES (7 errors; attribute break gone) | B1-O06 |
| B1-O07 LavaCauldronItem (2 breaks) | REPRODUCES (2 errors; package break gone) | B1-O07 |
| B1-O08 FluidBox (package break) | REPRODUCES (2 errors; package break gone) | B1-O08 |
| B11-G06 number providers | REPRODUCES — flagged for Batch 11, not minted here | (Batch 11) |
| B11-G13 player data path | REPRODUCES — flagged for Batch 11, not minted here | (Batch 11) |

**B11-G06 (number providers) — REPRODUCES, fresh 1.21.11 evidence.**
The 1.21.11 jar has 6 number providers (constant, uniform, binomial,
enchantment_level, score, storage). The Rust implements only 3:
`loot.rs:1330-1370` int provider implements constant/uniform/binomial
with `_ => 0` fallthrough, float provider implements constant/uniform
only (no binomial) with `_ => 0.`; `horizon-items/src/items.rs:1934`
`number()` implements constant/uniform/binomial with `_ => 0.0`.
So enchantment_level/score/storage return 0 everywhere, and binomial
returns 0 in the float path. Datapack-only for vanilla: 0 of the
1,324 loot tables in the 1.21.11 client jar use those three providers
(counted fresh). NOT minted in this slice — Batch 11 owns the B11-Gxx
namespace; flagged here so its ledger records the reproduction.

**B11-G13 (player data path) — REPRODUCES, fresh 1.21.11 evidence.**
`lan/playerdata.rs:18` still writes `players/data/`; vanilla 1.21.11
`LevelResource` has `PLAYER_DATA_DIR = "playerdata"` and
`PLAYER_OLD_DATA_DIR = "players"` — the game reads saves flat from
`playerdata/`, so port-written player data lands in the legacy
directory. NOT minted in this slice — Batch 11 owns the namespace;
flagged here for its ledger.

## Withdrawn

- **B1-G09 — WITHDRAWN as a defect number** (see §Closed): vanilla
  1.21.11 replaced the burnout biome tag with the environment attribute
  the port already reads. Landed as a closed item, not deleted.
- **Cauldron mayInteract candidate — NOT A DEFECT (nuance, §H).**
  Vanilla's `LayeredCauldronBlock.entityInside` runBefore lambda requires
  `entity.isOnFire() && entity.mayInteract(level, pos)` before lowering
  the level (javap-verified for 1.21.11); the port gates on fire_ticks>0
  && !spectator and has no mayInteract check. The gate is spawn protection
  + world border (`ServerLevel.mayInteract`: non-player → true; player →
  `!isUnderSpawnProtection && worldBorder.isWithinBounds`), which the
  port's single-player/LAN-host model does not implement as a concept.
  Recorded as a nuance, not a defect — the core behaviour (level
  lowering, powder-snow conversion, extinguish) matches. Same as 26.2,
  26.1, 26.1.1, 26.1.2 and 1.21.10.

## Passes by section

### §A. Fluid spreading and flow — 18 checks, 15 pass, 3 gap

- `canHoldAnyFluid` hardcoded form (LBC → true; blocksMotion → false;
  DoorBlock / `#signs` / ladder / sugar cane / bubble column / portals /
  end gateway / structure void excluded; else true) — **GAP (B1-G05)**:
  the port reads the absent `#washed_away_by_fluids` tag instead.
- `is_liquid_container` = LiquidBlockContainer OR SimpleWaterlogged —
  pass; required because the prepared export doesn't expand interface
  inheritance (jar: `SimpleWaterloggedBlock extends BucketPickup,
  LiquidBlockContainer` in 1.21.11).
- Direct LiquidBlockContainer blocks: 4 (kelp, kelp_plant, seagrass,
  tall_seagrass) — pass.
- Waterlogging exhaustive set: **410/410 EXACT MATCH** (vanilla 1.21.11
  `blocks.json` blocks with any `waterlogged` property = 410, counted
  fresh; prepared `block-behaviours.json` blocks with
  `SimpleWaterloggedBlock` interface = 410; zero diffs either way).
  Total states 29,671 confirmed — pass. (Prepared side is the real
  1.21.10 prepare; the equivalence chain is in the header note: vanilla
  1.21.11's set was counted fresh from its own jar, and 0 classes
  changed in the block package between the versions.)
- `#signs` exclusion moot (all signs SimpleWaterlogged, LBC branch
  returns true first) — pass.
- Water numbers: slope 4, dropOff 1, tickDelay 5 — pass (re-verified
  fresh: `iconst_4` / `iconst_1` / `iconst_5`).
- Lava numbers: slope 4 fast / 2 slow, dropOff 1 fast / 2 slow, tickDelay
  10 fast / 30 slow; `isFastLava` via `EnvironmentAttributes.FAST_LAVA`
  — pass. The built-in Nether has `fast_lava: true` hardcoded
  (`dimension_type.rs:189`), overworld false (`:157`), and
  datapack-dimension `from_json` reads the `gameplay/fast_lava`
  environment attribute (`:322`) — the SAME form vanilla 1.21.11 uses
  (the Nether dimension JSON sets `gameplay/fast_lava: true` and has no
  `ultrawarm` key). The 1.21.10 §A datapack-ultrawarm nuance is GONE
  for 1.21.11: vanilla moved to the attribute, and the port matches.
- Lava `getSpreadDelay` ×4 (both fluids non-empty, neither FALLING, target
  height higher, nextInt(4) != 0) — pass.
- Spread machinery: `spread` / `spreadToSides` / `getNewLiquid` /
  `getSpread` / `getSlopeDistance` / `spread_targets` — pass (vanilla
  `getNewLiquid`: Plane iterator, `canPassThroughWall`, isSource, amount
  max, 2-source + `canConvertToSource` → `getSource`). Note: the
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
  (`fluid_flow.rs`) is right, but its below-check input
  (`blocks_fluid_flow` table) now comes from the absent tag, so every
  neighbour reads as non-blocking.
- `can_convert_to_source` + 2-source rule: `water_source_conversion`
  default true, `lava_source_conversion` default false (GameRules static
  init re-verified fresh: `iconst_1` / `iconst_0`; port
  `gamerules.rs:126,76` + `level/mod.rs:226-227` + `save.rs:3575,3586`
  match) — pass.
- `blocks_fluid_flow` := `blocks_motion` mapping — **GAP (B1-G07)**: the
  26.1 mapping is gone; 1.21.11 reads the tag, which does not exist in
  1.21.11 data (no field in `BlockTags`).

### §B. Bubble columns — 10 checks, 10 pass

Same form as 1.21.10: 1.21.11's form is hardcoded (source water, soul
sand, magma block) and the port was adapted to it — same count, same
predicates. (The 1.21.11 tree's `tick.rs` comment names 1.21.11.)

- `canExistIn`: is bubble_column OR (block is water source) — pass
  (`shapes.rs:1149` `bubble_column_can_exist_in`).
- `getColumnState`: below is column → below state; below soul sand →
  drag=false; below magma → drag=true; else water default — pass
  (`tick.rs:551-590` `bubble_column_update_for`, hardcoded).
- `updateColumn`: canExistIn → setBlock(getColumnState, 2), then walk up
  while canExistIn — pass (`tick.rs:542` `bubble_column_update`).
- updateShape: canSurvive check + scheduleTick(self, 5) when below is
  column / canExistIn — pass (`shapes.rs:264`).
- `canSurvive`: below is column OR soul sand OR magma — pass
  (`survive.rs:79`, hardcoded).
- `tryScheduleBubbleBlockColumn`: MagmaBlock/SoulSandBlock updateShape →
  scheduleTick(self, 20) — pass (`shapes.rs:621-638`).
- Tick dispatch: bubble column + magma/soul-sand arms live — pass
  (`tick.rs:64-82`).
- `entityInside` surface-vs-inside split: above collision empty AND
  above fluid empty → `onAboveBubbleColumn(drag)` else
  `onInsideBubbleColumn(drag)` — pass.
- Entity motion constants: above drag max(-0.9, vy−0.03), above push
  min(1.8, vy+0.1) + splash particles; inside drag max(-0.3, vy−0.03),
  inside push min(0.7, vy+0.06) + resetFallDistance — pass
  (`motion/fluids.rs` `BubbleColumn::push`, constants re-verified fresh
  from `Entity.txt`).
- `bubble_column_at` surface detection + push returns !surface — pass.

### §C. Lava fire, ignition and game rules — 17 checks, 16 pass, 1 gap

One pass more than 1.21.10's 15: the fire-burnout check now PASSES
(B1-G09 withdrawn — vanilla moved to the attribute form), and the
doFireTick-pair check is replaced by the radius-rule check (also a
pass). Only B1-G06 remains a gap.

- `lava_random_tick` — **GAP (B1-G06)** (see §Defects).
- `lava_flammable` — pass, with the below/above-build-height nuance
  (§Method).
- Fire odds: **206/206 FULL MATCH** (parsed vanilla 1.21.11
  `FireBlock.bootStrap` setFlammable entries fresh vs prepared
  `fire.igniteOdds`/`burnOdds`; zero value diffs, zero only-either-side;
  white_wool (30,60), white_carpet (60,20) spot-checked) — pass.
  (Prepared side is the real 1.21.10 prepare; the vanilla side was
  parsed fresh from 1.21.11's jar and is byte-identical to 1.21.10's
  206 entries — see the header note.)
- `FireBlock.tick`: scheduleTick(getFireTickDelay) first;
  canSpreadFireAround gate → return; canSurvive → removeBlock(0);
  infiniburn check; rain 0.2+0.03*age; age++ min(15, age+nextInt(3)/2)
  setBlock 260; isValidFireLocation; checkBurnOut ×6 (300/300/250/250/
  300/300) — pass.
- `checkBurnOut`: waterlogged → 0; igniteOdds map lookup; nextInt(burnOdds)
  < chance; nextInt(age+10) < 5, !isRainingAt → fire via getStateWithAge
  flag 3, else removeBlock(0); TNT prime — pass (`tick.rs:742`
  `fire_check_burn_out` matches; the `allow_*` gates are native-mod
  gates defaulting to true — nuance, §Method).
- `getFireTickDelay`: 30 + nextInt(10) — pass (re-verified fresh:
  `bipush 30`).
- `BaseFireBlock.getState`: below canSurviveOnBlock → SOUL_FIRE else
  FIRE.getStateForPlacement — pass (`shapes.rs:1225` `fire_state_for`,
  `#soul_fire_base_blocks` tag exists in 1.21.11).
- `fireIgnite`: !fireImmune; remaining<0 → +1; ServerPlayer →
  +nextInt(1,3); remaining>=0 → igniteForSeconds(8.0) — pass
  (`motion/environment.rs:366` `ignite_contact` matches exactly,
  including the player-only random draw).
- `lavaIgnite`: !fireImmune → igniteForSeconds(15.0) — pass.
- `lavaHurt`: !fireImmune, server-side, hurtServer(lava, 4.0) →
  shouldPlayLavaHurtSound && !isSilent → GENERIC_BURN 0.4 vol, pitch
  2+rand*0.4 — pass on the damage; the sound was NOT traced in the Rust
  (untraced sub-detail, same as 26.2, 26.1, 26.1.1, 26.1.2 and 1.21.10).
- `LivingEntity.igniteForTicks`: ceil(ticks × BURNING_TIME) →
  Entity.igniteForTicks — pass (`mobs.rs:2758`, `motion.rs:520`).
- `LavaCauldronBlock.entityInside`: CLEAR_FREEZE + LAVA_IGNITE +
  runAfter(LAVA_IGNITE, lavaHurt) — pass (`motion.rs:2019`
  `lava_cauldron_inside`: ticks_frozen=0, ignite 300, hurt 4.0).
- `InsideBlockEffectType` mapping: 5 values (FREEZE, CLEAR_FREEZE,
  FIRE_IGNITE, LAVA_IGNITE, EXTINGUISH) exist in 1.21.11 (javap-verified
  fresh, incl. the bootstrap method refs: CLEAR_FREEZE =
  `Entity.clearFreeze`, FIRE_IGNITE = `BaseFireBlock.fireIgnite`,
  LAVA_IGNITE = `Entity.lavaIgnite`, EXTINGUISH = `Entity.clearFire`);
  the Rust's StepBasedCollector mapping applies — pass. (Class moved to
  `net.minecraft.world.entity` in 1.21.11.)
- `canSpreadFireAround` + spectator exclusion — pass (see B1-G11): the
  radius rule, default 128, −1 = everywhere, spectators excluded.
- Infiniburn sets: overworld magma_block + netherrack, nether same, end
  + bedrock (tags verified in 1.21.11 data) — pass.
- `Entity.baseTick` fire phase: fireImmune → clearFire; %20==0 &&
  !isInLava → hurtServer(onFire, 1.0); decrement; fallDistance ×0.5 in
  lava; `setSharedFlagOnFire`; firstTick=false — pass. The −20 player
  immunity reset (`applyEffectsFromBlocks` tail:
  `setRemainingFireTicks(-getFireImmuneTicks())`, `Player.getFireImmuneTicks`
  = 20, javap-verified) is implemented (`motion/environment.rs:166`
  `fire_ticks = -20` when no fire contact; initial −20 at
  `motion.rs:601`). Item entities (`entities.rs:484`: water → 0, %20
  → hurt 1, lava → max(300) + hurt 4 per contact, fire contact →
  max(160)), armor stands (`entities.rs:2488`: %20 → −4.0, matching
  vanilla's BURNS_ARMOR_STANDS branch — `#burns_armor_stands` holds
  only `minecraft:on_fire`, verified in the client jar), and mobs
  (`mobs/ai.rs:42` `block_contact_damage`) all match.
- Fire burnout environment attribute — **PASS** (B1-G09 withdrawn): vanilla
  reads `EnvironmentAttributes.INCREASED_FIRE_BURNOUT`; the port reads
  the same attribute from the biome JSON, and the 8 biomes carry it —
  see §Closed.

### §D. Entity fluid interaction and pushing — 7 checks, 6 pass, 1 gap

Same classic 1.21.x form as 1.21.10 (per-tag methods, no
`EntityFluidInteraction` class) — same count, same methods. One vanilla
delta: the lava speed gate is now the FAST_LAVA environment attribute
instead of `ultrawarm`, and the port's dimension facts carry the same
attribute value.

- `updateFluidHeightAndDoFluidPushing` structure: touchingUnloadedChunk
  gate; bbox.deflate(0.001); floor/ceil bounds; per-cell loop (x outer,
  y middle, z inner); top = y + plain `FluidState.getHeight`; height =
  max(top − minY); flow scaled by the running max height if < 0.4;
  accumulate — pass (Rust `entity_fluid.rs` mirrors it; the height is
  measured from the deflated box's bottom, as vanilla does).
- Eye-height check: Rust uses `camera_height` instead of the plain top —
  **GAP (B1-G04)**.
- Flow scaling: Rust scales by the running max height (< 0.4) = the
  same value vanilla uses, same order — pass.
- `apply_to` constants: no 1e-5 floor (length > 0, matching vanilla);
  non-Player → normalize; scale(1/count); scale by speed; if
  |dx|<0.003 && |dz|<0.003 && length<0.0045 →
  normalize.scale(0.0045); addDeltaMovement — pass.
- `updateInWaterStateAndDoWaterCurrentPushing`: boat-passenger
  short-circuit (vehicle is boat NOT underwater → water update skipped
  entirely; lava update still runs with the full box); water →
  applyCurrentTo(WATER, 0.014); lava → FAST_LAVA? 0.007 :
  0.0023333333333333335; inWater → resetFallDistance + splash if
  !wasTouchingWater && !firstTick; wasTouchingWater = inWater — pass
  (`motion/fluids.rs` `entity_fluid_tick::update` mirrors it; the lava
  speeds live at `entities/fluids.rs:108-115`, gated on
  `world.fast_lava`, itself set from the dimension facts at
  `game/levels.rs:261` — the attribute value, matching vanilla).
- `updateFluidOnEyes` structure: single eye cell, top strictly above
  eyeY → tags added; `isInLava` = !firstTick && fluidHeight(LAVA) > 0
  — pass (the top VALUE is the gap, counted once in the eye-height
  check above).
- Fluid interaction box: bbox.deflate(0.001) — pass, with the boat-clip
  nuance noted in B1-G04 (vanilla short-circuits the water update for
  boat passengers instead of clipping the box; the port's clip gives
  the same answer for water).

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
  dropItem && ENTITY_DROPS → spawnAtLocation; discard — pass
  (`bipush 100` / `sipush 600` re-verified fresh).
- Air drag: vanilla `deltaMovement.scale(0.98)` — the DOUBLE literal —
  **GAP (B1-G08)**: the port uses `f64::from(0.98_f32)`.
- `causeFallDamage`: hurtEntities gate; ceil(fallDistance)-1;
  NO_CREATIVE_OR_SPECTATOR + LIVING_ENTITY_STILL_ALIVE predicates;
  `Fallable.getFallDamageSource` or `fallingBlock`; min(floor(perDistance),
  max); anvil: damage>0 && nextFloat < 0.05 + damage*0.05 →
  `AnvilBlock.damage` — pass (`game/falling_damage.rs:66-71` anvil wear
  via `falling_block::wear_anvil`).
- Anvil events: `onLand` !isSilent → levelEvent 1031; `onBrokenAfterFall`
  !isSilent → levelEvent 1029 — pass (Rust `falling.rs:133` emits 1031;
  `falling_damage.rs:95` emits 1029; `game.rs:4493-4494` maps both to
  sounds; vanilla `AnvilBlock` sipush 1031/1029 re-verified fresh).
- `AnvilBlock.damage` chain: ANVIL → CHIPPED_ANVIL → DAMAGED_ANVIL,
  FACING preserved — pass (`falling_block::wear_anvil`).
- `getFallDamageSource`: `damageSources.anvil(entity)` — pass
  (`falling_damage.rs:36-37` `"minecraft:falling_anvil"`).
- `game.rs` FallingBlock event consumer (4609-4634): spawns the entity
  at pos+0.5, y, z+0.5, velocity ZERO, drop_item true,
  `Damage::falling(name)`, hurt_per_distance → hurt_entities — pass.
- `Damage::falling` defaults (anvil 2.0 per distance / 40 max) — pass
  (`falling_block.rs:35,41`).

Out-of-scope observation (not counted): dragon-egg teleport on use —
delay 5 (`iconst_5`) and 1000 tries (`sipush 1000`) re-verified fresh
in the 1.21.11 jar, but the full teleport path (offsets, gates,
particles) was not compared — teleport on use is out of scope for this
batch (falling-block scope covers fall/landing/conversion), same as 26.2,
26.1, 26.1.1, 26.1.2 and 1.21.10.

### §G. Dripstone and speleothem — 11 checks, 6 pass, 5 gap

Same structure as 1.21.10's 11 checks: the dispatch was adapted in the
template and five behaviours work, so §G splits placement into direction
(pass) + thickness (gap) and counts the working arms — 11 checks, because
placement direction and thickness are separate vanilla methods
(`calculateTipDirection` vs `calculateDripstoneThickness`). The dead-code
constant matches in `speleothem.rs` (growth 0.011377778, transfer
0.17578125/0.05859375, findTip max 11, cauldron delay 50, tick delays 2/1,
mud→clay event 1504, `randomTick` draw order) are verified matching but NOT
counted as passes, because the code never runs.

- Random-tick transfer (stalactite → stalagmite fluid transfer) —
  **GAP (B1-G01)**.
- Random-tick growth — **GAP (B1-G01)**.
- Cauldron fill from dripstone (`cauldron_tick`'s tip search needs
  `is_free_hanging_stalactite`) — **GAP (B1-G01)**. (The cauldron-side
  receive logic itself is separately verified in §H.)
- Placement direction (tip direction via `calculateTipDirection`) —
  pass (dispatch arm `place.rs:1156` fires; direction calc matches).
- Thickness calc (placement + updateShape, one underlying fact) —
  **GAP (B1-G01)**: both call sites' `is_with` closures read the absent
  `#speleothems` tag, so thickness is always `"tip"`.
- updateShape survival scheduling (scheduleTick 2/1 when !canSurvive) —
  pass (dispatch arm `shapes.rs:745` fires; scheduling matches).
- canSurvive (floating dripstone breaks — `is_valid_speleothem_placement`
  is not tag-gated) — pass (dispatch arm `survive.rs:326` fires).
- Scheduled tick destroy (unsupported stalagmites destroyed) — pass
  (dispatch arm `tick.rs:257` fires; destroy arm matches).
- Falling stalactite spawn (tag-gated, `falling.rs:51`) —
  **GAP (B1-G01)**.
- `fallOn` stalagmite damage (up + tip → Stalagmite, distance+2.5,
  ×2.0) — pass (name/data-keyed, not dispatch-gated).
- Trident break (Trident + hit_speed > 0.6 → destroy, with mayInteract /
  mayBreak / `projectiles_can_break_blocks` gates) — pass
  (name-keyed, `game/creatures.rs:5095`).

### §H. Cauldrons — 8 checks, 8 pass

- `CauldronBlock.handlePrecipitation`: RAIN → nextFloat < 0.05 → water
  cauldron level 1; SNOW → nextFloat < 0.1 → powder snow cauldron;
  BLOCK_CHANGE game event — pass (`tick.rs:1524` `precipitation_tick`:
  chances 0.05/0.1, defaults level 1, BLOCK_CHANGE).
- `LayeredCauldronBlock.handlePrecipitation`: shouldHandlePrecipitation
  && level != 3 && precipitationType matches → cycle(LEVEL) +
  setBlockAndUpdate + BLOCK_CHANGE — pass.
- `canReceiveStalactiteDrip`: empty cauldron → true; layered → fluid ==
  WATER && precipitationType == RAIN; lava cauldron → false (no
  override, AbstractCauldronBlock default) — pass
  (`speleothem.rs:317` `cauldron_can_receive`: Cauldron → true;
  LayeredCauldron → water && water_cauldron; else false).
- `receiveStalactiteDrip` state transitions + events: empty + WATER →
  WATER_CAULDRON + event 1047; empty + LAVA → LAVA_CAULDRON + event 1046;
  layered !isFull → LEVEL+1 + event 1047; BLOCK_CHANGE each — pass
  (`speleothem.rs:338` `cauldron_tick` second half matches exactly,
  events at lines 361/363/386; vanilla events 1046/1047 re-verified
  fresh).
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
  + BLOCK_CHANGE — pass (`cauldron.rs:113-120` `extinguished`/`lowered`;
  `game.rs:8107` `cauldron_extinguish` incl. the
  `native_cauldron_level_change_allowed` gate and BLOCK_CHANGE event).
- `lava_cauldron_inside`: CLEAR_FREEZE (ticks_frozen=0) + LAVA_IGNITE
  (ignite_for_ticks(300) = 15s) + lava hurt 4.0 — pass
  (`motion.rs:2019`).
- CauldronExtinguish emit conditions (burning, !spectator) + LAN
  consumption (`lan_actions.rs:5186` → `game.cauldron_extinguish`) —
  pass. Content heights: lava 15/16, layered (6.0 + 3.0*level)/16 —
  verified (`motion.rs:1820-1835`).

### §I. Splash water — 4 checks, 0 pass, 4 gap

Same structure as 1.21.10's §I: the gate kills all four outcomes at once
(B1-G03), so §I counts the four observable outcomes — the gate itself is
the shared cause, described in B1-G03, and the downstream code (verified
matching vanilla exactly, see B1-G03) is recorded as analysis, NOT counted
as passes.

- Dowse fire on block hit — **GAP (B1-G03)**.
- Hurt water-sensitive entities (1.0 indirectMagic) — **GAP (B1-G03)**.
- Extinguish burning entities — **GAP (B1-G03)**.
- Rehydrate axolotls (+1800 air, capped) — **GAP (B1-G03)**.

### Tick dispatch callers — verified (folded into the sections above)

- `level/tick.rs` scheduled-tick dispatch: bubble column (64–82),
  AbstractCauldron, falling blocks, fire — verified live. The
  Speleothem/PointedDripstone arms (257, 443) FIRE (B1-G01's dispatch
  cause is fixed in the template); only the transfer/growth/cauldron-fill
  bodies they call are dead.
- `level/shapes.rs` updateShape dispatch: bubble column (264), falling
  (617–618), liquid (617–618) — verified live. The Speleothem arm (745)
  fires; only its thickness recalc is dead (B1-G01).
- `level/place.rs` placement dispatch: concrete solidify (775), liquid
  (617–618) — verified live. The Speleothem arm (1156) fires; only its
  thickness calc is dead (B1-G01).
- `level/survive.rs` canSurvive dispatch: bubble column (79), falling —
  verified live. The Speleothem arm (326) fires and works (B1-G01's
  canSurvive is a pass).

### Prepared-data class/interface spot checks

- pointed_dripstone: class chain `[PointedDripstoneBlock, Block]`,
  fields `{}` (no `blockToGrowOn`), interfaces include
  `SimpleWaterloggedBlock`, `Fallable` — the B1-G01 causes #2 (tag) and
  the missing link. The dispatch cause is fixed in this tree.
- Waterlogging census 410/410 (§A); fire odds census 206/206 (§C).
- `block-behaviours.json` structure (1.21.10 prepare, the only prepare
  available): top keys `blocks` (1,166), `states` (29,671), `items`,
  `fluids`, `fire` (shapes/igniteOdds/burnOdds/ fireDamage=1.0).
- `datagen/reports/registries.json` entries are ALPHABETICAL, not id
  order — sort by `protocol_id` before cumulative state counting
  (total states 29,671 confirms the count).
- Prepared `block-tags.json`: 203 keys, no `speleothems`, no
  `washed_away_by_fluids`, no `blocks_fluid_flow`, no `blocks_motion`,
  no `blocks_lava_fire_spread`, no `blocks_motion_in_heightmap`, no
  `causes_suffocation`, no `suppresses_bounce`. Prepared tag values are
  raw JSON strings; nested refs are `#minecraft:<name>` (strip leading
  `#` only when resolving).
- Concrete powders have `fields: {concrete: minecraft:<colour>_concrete}`
  (link resolves — the B1-G02 fix is local to the fall step, not the
  data).
- (Prepared-data bullets above are from the real 1.21.10 prepare —
  see the header note. No 1.21.11 prepare exists; the 1.21.11 jar-side
  checks (0 classes changed in the block package, 410 waterlogged
  blocks counted fresh, 206 fire entries parsed fresh) are what bind
  them to 1.21.11.)

## Oracle defects in scope

Authority: the MEASURED 1.21.11 Batch 0 table
(`docs/sweep/1.21.11/batch-0-oracles.json`, generated 2026-11-25; 391
oracles, 102 compile on the server classpath, 127 on at least one
classpath) plus the raw big-compile logs
(`b00/evidence/big-compile-server.log`, from which every error line
below was read). This batch ran no measurement of its own — the table
landed during the batch and the triage below is measured, not
provisional. All 13 in-scope oracles are present in
`versions/1.21.11/mods/horizon/src/test/java/dev/horizonlab/parity/`.
**11 fail, 2 pass** (vs 1.21.10's measured 11 fail / 2 pass — same
count, different error profiles: three 1.21.10 package breaks healed
because vanilla 1.21.11 ADOPTED the 26.x packages).

**Correction to the 1.21.10 report's provisional triage.** The 1.21.10
report triaged from the 26.1.2 table (its own table had not landed) and
provisionally passed VanillaWaterSplashOracle, counted 1 error for
VanillaFluidOracle and VanillaCauldronOracle, and 9 for
VanillaLivingFluidOracle. The measured 1.21.10 table (which landed after
that report) and the measured 1.21.11 table both show: WaterSplash FAILS
(1 error — B1-O09 below, new item for this run), FluidOracle has 2
errors (line 26 is `isFull`, line 50 `getHeightForCamera`), Cauldron has
2 errors (line 118 Prediction + line 93 ServerLevel ctor), LivingFluid
has 12 errors in 1.21.11 (14 in the measured 1.21.10 table — the two
fewer here are the `is(TagKey)` overrides at lines 41/72, which fail in
both; the 1.21.10 table's extra two are its own ctor-shape errors).
The counts below are the measured 1.21.11 numbers, read from the
big-compile log.

Oracle sources: 10 of the 13 are byte-identical to 1.21.10's; the other
3 (VanillaCauldronOracle, VanillaEntityFluidOracle,
VanillaWaterSplashOracle) differ ONLY by the ResourceLocation→Identifier
rename (1.21.11 renamed the class; the oracles were renamed with it).

### B1-O01 — `BlockTags.BLOCKS_FLUID_FLOW` is 26.3-only; breaks 3 oracles (MINOR)

- VanillaFluidFlowOracle: 2 errors — line 14, incompatible-types
  (`Property<?>[]` cannot convert to `Reference2ObjectArrayMap`, the
  BlockState ctor shape — an ADDITIONAL break vs the 1.21.10 report's
  1-error claim), and line 15, missing-symbol (BLOCKS_FLUID_FLOW).
- VanillaRegisteredEntityFluidOracle: 2 errors, lines 9–10.
- VanillaRegistryFluidOracle: 3 errors, lines 11, 14 (BLOCKS_FLUID_FLOW;
  line 17 is B1-O04's symbol).
- javap-verified fresh: no `BLOCKS_FLUID_FLOW` field in 1.21.11 `BlockTags`;
  the tag file is absent from 1.21.11 data. Same as 1.21.10 B1-O01, 26.1.2
  B1-O01, 26.1.1 B1-O01, 26.1 B1-O01 and 26.2 B1-O01. All three sources
  byte-identical to 1.21.10's. The port deliberately maps
  blocks_fluid_flow := blocks_motion for 26.1 (the 1.21.11 tree lost that
  mapping — B1-G07), so the repair is to bind the tag from the
  blocksMotion table, not to add the tag.

### B1-O02 — `VanillaCauldronOracle` BROKEN, 2 compile errors (MINOR)

- 2 errors: line 118, missing-symbol: the `drop()` override references
  `net.minecraft.util.Prediction` — ABSENT from both 1.21.11 jars (jar
  listing verified, 0 matches in either); PLUS line 93,
  constructor-mismatch: the `World` subclass calls the 10-arg
  `ServerLevel` super ctor, but 1.21.11's `ServerLevel` takes 11 params
  (ending in `RandomSequences`). Same as 1.21.10 B1-O02 (whose report
  counted only the Prediction error — the measured tables show 2 in both
  versions), 26.1.2 B1-O02, 26.1.1 B1-O02, 26.1 B1-O02 and 26.2 B1-O02.
  The source differs from 1.21.10's only by the ResourceLocation→Identifier
  rename.

### B1-O03 — `VanillaFallingBlockOracle` BROKEN, 1 compile error (MINOR)

- 1 error, line 30, constructor-mismatch: `NoopChunkMap` calls `super()`
  with 12 args; 1.21.11 `ChunkMap` ctor takes 13 params (ServerLevel,
  LevelStorageAccess, DataFixer, StructureTemplateManager, Executor,
  BlockableEventLoop, LightChunkGetter, ChunkGenerator,
  ChunkStatusUpdateListener, Supplier, TicketStorage, int, boolean —
  javap-verified fresh). Same as 1.21.10 B1-O03, 26.1.2 B1-O03, 26.1.1
  B1-O03, 26.1 B1-O03 and 26.2 B1-O03; Batch 0 group G6. Source
  byte-identical to 1.21.10's.

### B1-O04 — `VanillaFluidOracle` BROKEN, 2 compile errors (MINOR)

- 2 errors: line 26, missing-symbol: `state.isFull()` — `FluidState`
  has NO `isFull` in 1.21.11 (also absent from `StateHolder`;
  javap-verified fresh); line 50, missing-symbol:
  `state.getHeightForCamera(world, p)` — 1.21.11 `FluidState` has only
  `getHeight(BlockGetter, BlockPos)` (javap-verified fresh). Same as
  1.21.10 B1-O04 (whose report counted only the line-50 error — the
  measured tables show 2 in both versions), 26.1.2 B1-O04, 26.1.1 B1-O04,
  26.1 B1-O04 and 26.2 B1-O04. (VanillaRegistryFluidOracle line 17
  shares the `getHeightForCamera` symbol.) Source byte-identical to
  1.21.10's.

### B1-O05 — `VanillaLivingFluidOracle` BROKEN, 12 compile errors (MINOR)

- 12 errors (measured 1.21.11; 14 in the measured 1.21.10 table):
  line 20 ServerLevel ctor (10-arg super vs 11-param); lines 30 + 61
  ×2 `Attributes.FRICTION_MODIFIER` and `Attributes.AIR_DRAG_MODIFIER`
  missing (javap: 1.21.11 `Attributes` has no FRICTION_MODIFIER /
  AIR_DRAG_MODIFIER; `WATER_MOVEMENT_EFFICIENCY` and `BURNING_TIME`
  exist); lines 41 + 72 `is(TagKey<EntityType<?>>)` no-override —
  `Entity` has `is(Entity)` but NO `is(TagKey)` in 1.21.11 (javap-verified
  fresh; the `EntityTypeTags.CAN_FLOAT_WHILE_RIDDEN` /
  `POWDER_SNOW_WALKABLE_MOBS` fields DO exist, so only the override
  fails); lines 50 + 81 + 97 `FluidTags.ENTITY_FLOATABLE` missing
  (1.21.11 `FluidTags` has exactly 2 fields, WATER and LAVA — no
  ENTITY_FLOATABLE); lines 54 + 85 `omnidirectionalAirMover`
  no-override (absent from 1.21.11 `LivingEntity`).
- Runtime note (BETTER than 1.21.10): the oracle looks the travel methods
  up with `getDeclaredMethod` + `setAccessible` (line 88), so PRIVATE
  methods are found. In 1.21.11, `travelInWater`, `travelInLava` and
  `jumpOutOfFluid` NOW EXIST (protected, javap-verified fresh in
  `LivingEntity.txt`) — three of the five 1.21.10 runtime failures are
  healed. Only TWO lookups still fail: `floatInLiquidWhileRidden` and
  `computeModifiedFriction` — so even after the compile errors are fixed
  the oracle would fail at startup on the first of those, plus the
  `ENTITY_FLOATABLE` reference in "float" mode (line 97). Source
  byte-identical to 1.21.10's.

### B1-O06 — `VanillaEntityFluidOracle` BROKEN, class absent entirely (MINOR)

- 7 errors (measured 1.21.11; 11 in the measured 1.21.10 table — the
  attribute break is gone): line 41 ServerLevel ctor; lines 52 + 60
  `getFluidInteractionBox` no-override (absent from 1.21.11 `Entity`);
  lines 65 ×2 + 68 cannot-find-symbol (`EntityFluidInteraction` class,
  `$Tracker`, `$CurrentAccumulator` — the class does not exist AT ALL in
  1.21.11, 0 matches in either jar); line 69 incompatible-types (the
  FluidState ctor shape, same BlockState-ctor break as B1-O01).
- HEALED vs 1.21.10: oracle line 42 references
  `net.minecraft.world.attribute.EnvironmentAttributeSystem` /
  `EnvironmentAttributes.FAST_LAVA` — the `net.minecraft.world.attribute`
  package EXISTS in 1.21.11 (53 classes; `EnvironmentAttributes` carries
  FAST_LAVA + INCREASED_FIRE_BURNOUT), so that break is gone. The
  reflective lookups of `Entity.fluidInteraction` (field) and
  `Entity.updateFluidInteraction` (method) still fail — 1.21.11's Entity
  has the `fluidHeight` map + `fluidOnEyes` set and the
  `updateFluidHeightAndDoFluidPushing` /
  `updateInWaterStateAndDoWaterCurrentPushing` methods instead. The oracle
  cannot be repaired by a signature fix; it needs the classic-form
  rewrite. The source differs from 1.21.10's only by the
  ResourceLocation→Identifier rename.

### B1-O07 — `VanillaLavaCauldronItemOracle` BROKEN, 2 compile errors (MINOR)

- 2 errors: line 27, constructor-mismatch: the 10-arg `ServerLevel`
  super call vs the 11-param ctor; line 28, no-override:
  `getNextEntityId()` does not override — absent from both 1.21.11
  `ServerLevel` and `Level` (javap-verified fresh).
- HEALED vs 1.21.10: line 27's reference to
  `net.minecraft.world.level.gamerules.GameRules` COMPILES in 1.21.11 —
  the `net.minecraft.world.level.gamerules` package EXISTS (9 classes;
  game rules moved there). Same count as 1.21.10 (2), different second
  break. Source byte-identical to 1.21.10's. Passed in 26.2.

### B1-O08 — `VanillaFluidBoxOracle` BROKEN for 1.21.11 (MINOR)

- 2 errors (measured): line 16, no-override:
  `AbstractBoat.modifyPassengerFluidInteractionBox` — absent from 1.21.11
  (no `modifyPassengerFluidInteractionBox` anywhere in the jar); line 21,
  cannot-find-symbol: `Entity.getFluidInteractionBox()` — absent from
  1.21.11 (no `getFluidInteractionBox` anywhere; 1.21.11 has no
  interaction-box override — the boat passenger case is handled by the
  water-update short-circuit in
  `updateInWaterStateAndDoWaterCurrentPushing`).
- HEALED vs 1.21.10: the oracle imports
  `net.minecraft.world.entity.vehicle.boat.*` — the package EXISTS in
  1.21.11 (8 classes; `AbstractBoat` moved there). Source byte-identical
  to 1.21.10's. (The `Actor extends Marker` base is fine:
  `net.minecraft.world.entity.Marker` exists in the 1.21.11 jar.)

### B1-O09 — `VanillaWaterSplashOracle` BROKEN, 1 compile error (MINOR, NEW)

- 1 error, line 14, constructor-mismatch: the `World` subclass calls the
  10-arg `ServerLevel` super ctor vs the 11-param ctor. Measured in the
  1.21.11 Batch 0 table (and in the measured 1.21.10 table — the 1.21.10
  report provisionally PASSED it from the 26.1.2 table, which is why this
  is a new item for this run rather than a new-to-1.21.11 break). The
  oracle's other symbols all verify: `Entity.doWaterSplashEffect`
  (reflective) + `position`/`dimensions`/`level`/`random` fields +
  `GameEvent.SPLASH` + `ParticleTypes.BUBBLE`/`SPLASH` +
  `EntityDimensions.fixed` + `SoundEvent.createVariableRangeEvent` +
  the bootstrap call `SharedConstants.tryDetectVersion()` (verified
  present in the 1.21.11 jar). The source differs from 1.21.10's only by
  the ResourceLocation→Identifier rename.

### Oracles assessed PASSING for 1.21.11 (measured Batch 0 table)

Both classpaths: VanillaFluidOccupancyOracle, VanillaFluidShapeOracle.
Every symbol they reference was javap-verified against the 1.21.11 jar in
this batch: FluidOccupancy's 5-arg `ProtoChunk` ctor,
`PalettedContainerFactory`, `Strategy.createForBlockStates/createForBiomes`,
`LevelReader.containsAnyLiquid`, `Holder.Reference.createStandAlone`,
`FlowingFluid.shapes`, `FluidState.CODEC`,
`Fluid.FLUID_STATE_REGISTRY`, `FluidState.getShape`, `Block.stateById`;
FluidShape's `FluidState.getShape` / `getHeight`. Their only reference
to a changed class is the bootstrap call
`SharedConstants.tryDetectVersion()`, verified present in the 1.21.11
jar. Both sources byte-identical to 1.21.10's.

## Systemic findings

The 1.21.7 pilot's systemic root cause — 26.x tag-based logic for tags
absent in the target version's data — applies to 1.21.11 in the same
**five** places as 1.21.10: the speleothem system (B1-G01:
`#speleothems` and `SpeleothemBlock` both absent — though the dispatch
cause is fixed in the template) and splash water (B1-G03: the four potion
tags absent — 1.21.11 has ZERO potion tag files at all) as in 26.1, PLUS
`can_hold_any_fluid` (B1-G05: `#washed_away_by_fluids`), the lava-fire
walk (B1-G06: `#blocks_lava_fire_spread`), and the flow-vector table
(B1-G07: `#blocks_fluid_flow`). A second regression is numeric, not
tag-based: the falling-block air drag (B1-G08: 0.98F widened instead of
the double literal). B1-G04 is the same pattern at a smaller scale
(26.3-form entity fluid interaction, camera-height eye check).

What is DIFFERENT about 1.21.11 — and worth the coordinator's attention —
is the direction of the remaining divergence. This tree is the TEMPLATE,
so the five tag reads are the template's own unadapted residue, inherited
down by the 1.21.10 child — but vanilla 1.21.11 itself MOVED TOWARD the
template in three places, healing what were defects in 1.21.10: the
environment-attribute system arrived (FAST_LAVA + INCREASED_FIRE_BURNOUT
— the burnout gate B1-G09 is now correct, and the datapack-dimension
fast_lava read is now the vanilla form), the game rules moved to the
`gamerules` package with the radius rule replacing the doFireTick pair
(the port's radius-form fire gate is now the vanilla form), and the
`vehicle.boat` package arrived (healing one oracle break). The pattern
is therefore not "the tree ships 26.3 everywhere" but "the tree ships
26.3 exactly where the port did not adapt — and vanilla has started
adopting the 26.x forms the template already ships". Where 1.21.11's data
HAS the tag (`#signs`, `#fire`, `#anvil`, infiniburn,
`#soul_fire_base_blocks`, `#shovels`, `can_glide_through`), the port
reads it; where 1.21.11 still lacks the tag, the port usually carries the
hardcoded 1.21.7-style list form — except in the five places above, where
it ships the 26.3 form. The two template-native adaptations (bubble
columns hardcoded, B1-G10; fire rules honoured, B1-G11) and the adapted
dripstone dispatch are the proof the template CAN carry the 1.21.x form
where the port chose to adapt.

## Method notes

- **NO 1.21.11 prepare exists** (checked all candidate `.local` dirs
  before writing). Both censuses used the real 1.21.10 prepare as the
  prepared side, bound to 1.21.11 by jar comparison: the vanilla sides
  were counted FRESH from 1.21.11's own jar/reports (410 waterlogged
  blocks from `blocks.json`; 206 fire entries parsed from
  `FireBlock.bootStrap`), and a class-hierarchy parse of both jars shows
  41 classes changed overall, ZERO in the block package (the only new
  block-package class is the inner `Block$UpdateFlags`). The coordinator
  should re-count against a real 1.21.11 prepare when one exists; no
  verdict depends on the prepared side alone.
- **Vanilla ground truth is 1.21.11's own jar/data only** (owner's
  vanilla-check rule). Every vanilla claim above cites the 1.21.11
  server-game jar's bytecode or the 1.21.11 client jar's data. Where
  1.21.11's form differs from 26.3's (no `washed_away_by_fluids`, no
  `blocks_lava_fire_spread`, no `blocks_fluid_flow`, no `blocks_motion`,
  no `douses_fire`, no `SpeleothemBlock`, no `#speleothems`, no
  `EntityFluidInteraction`, no `Prediction`, no `getHeightForCamera`, no
  `isFull`, no `FRICTION_MODIFIER`/`AIR_DRAG_MODIFIER`, no
  `ENTITY_FLOATABLE`, no `getFluidInteractionBox`, no
  `getNextEntityId`, no `is(TagKey)` on Entity), the port was checked
  against the 1.21.11 form. Where 1.21.11's form differs from 1.21.10's
  (it HAS the `world.attribute` package with FAST_LAVA +
  INCREASED_FIRE_BURNOUT, HAS the `gamerules` package with the radius
  rule and NO doFireTick pair, HAS the `vehicle.boat` package, HAS
  `Identifier` and NO `ResourceLocation`, HAS `travelInWater` /
  `travelInLava` / `jumpOutOfFluid` on LivingEntity, has the burnout
  attribute instead of the biome tag, has 0 potion tag files, has 2
  fluid tags not 6), the port was checked against the 1.21.11 form —
  the 1.21.10 run's answers were never assumed.
- **Tag inventory verified in 1.21.11's own data** (client jar, 666 tag
  files + `BlockTags`/`FluidTags` javap + prepared `block-tags.json`,
  203 keys): `#speleothems` ABSENT; `washed_away_by_fluids`,
  `blocks_lava_fire_spread`, `blocks_fluid_flow`, `blocks_motion`,
  `blocks_motion_in_heightmap`, `causes_suffocation`,
  `suppresses_bounce`, `enables_bubble_column_push_up`,
  `enables_bubble_column_drag_down`, `douses_campfires` ABSENT; fluid
  tags total two (lava, water); potion tags: ZERO files; item tag
  `shovels` exists, `douses_campfires` does not; block tags `signs`,
  `anvil`, `fire`, `infiniburn_overworld` (magma_block, netherrack),
  `infiniburn_nether` (= overworld), `infiniburn_end` (+ bedrock),
  `soul_fire_base_blocks`, `can_glide_through` all exist; biome tag
  `increased_fire_burnout` ABSENT (replaced by the environment
  attribute — all 8 biomes carry it in JSON). No sulfur_spike in
  1.21.11 (0 entries in either jar; absent from prepared
  block-behaviours).
- **1.21.11 universe** (established): DataVersion 4671, protocol 774,
  Java 21; 10,291 client-named classes, 7,163 server-game classes;
  29,671 states, 410 waterlogged blocks, 206 fire-odds entries, 1,324
  loot tables, 666 tag files, 65 biomes. Registration still lives on
  EntityType/BlockEntityType (no EntityTypes/BlockEntityTypes holder —
  verified by jar listing).
- **1.21.10 corrections owed (this batch's findings):** the 1.21.10
  report's provisional oracle triage needs three corrections in ITS
  ledger (WaterSplash fails, not passes; FluidOracle 2 errors, not 1;
  Cauldron 2 errors, not 1; LivingFluid 14 errors in the measured
  1.21.10 table, not 9) — flagged for the coordinator, not acted on
  here. No Batch 1 record needed correction for the 26.1.2 items
  (B5-G18 bell, B10-G04 BE renderers are outside Batch 1 scope).
- **Nuances recorded, not defects:** (1) Rust `lava_flammable` returns
  false only if inside build height AND not loaded, then checks the flag;
  vanilla returns false whenever NOT (inside && hasChunkAt) — below/above
  build height the Rust proceeds to the flag check (state lookup there is
  presumably air → false anyway). (2) The boat interaction-box clip
  (B1-G04): vanilla short-circuits the water update for boat passengers
  instead of clipping the box; the port's clip gives the same answer for
  water, and the lava-path asymmetry is unobservable. (3) Cauldron
  mayInteract gate (§Withdrawn). (4) The dead `"minecraft:sulfur_spike"
  => Some(1052)` arm in `finish_falling_block` (harmless, B1-G02).
  (5) `allow_block_ignite` / `allow_spread` /
  `allow_fluid_level_change` are native-mod gates defaulting to true
  (`level/mod.rs:380`, `game.rs:1135`) — not vanilla divergences.
  (6) `FallingBlockEntity.tick`'s fast-fall clip runs when the delta's
  length-squared is greater than 1.0 — the port has no clip check at all,
  so the verdict is unchanged. (7) The 1.21.10 §A datapack-ultrawarm
  nuance is GONE for 1.21.11 (vanilla reads the attribute too — §A).
- **Untraced sub-details (NOT counted as passes, NOT needs-harness — they
  are sub-details of passing checks or out of scope):** the lava-hurt
  GENERIC_BURN sound (vol 0.4, pitch 2+rand*0.4) was not traced in the
  Rust; `PointedDripstoneBlock.animateTick` drip particles were not
  compared; dragon-egg teleport on use is out of scope (falling-block
  scope covers fall/landing/conversion) — though its delay (5) and
  tries (1000) were re-verified fresh anyway (see §F).
- **Out-of-scope observations (flagged for the coordinator / owning
  batches — NOT B1 items, NOT counted):** (a) `level/hooks.rs:549`
  `ice_destroyed_by_player` reads the absent `#blocks_motion` tag
  instead of the `blocksMotion()` rule vanilla `IceBlock.playerDestroy`
  uses — mined ice over solid ground will not turn into water in
  1.21.11 (vanilla `blocksMotion()` call re-verified fresh in the
  1.21.11 jar). (b) `game/campfire_douse.rs` ships the 26.3 form
  (`CampfireBlock.useItemOn` + `#minecraft:douses_campfires`) instead of
  1.21.11's `ShovelItem.useOn` → `CampfireBlock.dowse` (javap-verified
  fresh; `#shovels` exists in 1.21.11 data, `#douses_campfires` does
  not), so shovel-dousing a campfire can never fire. (Splash-water
  campfire dousing is B1-G03's downstream code and is a separate path.)
  (c) `horizon-client-render/src/assets.rs:load` errors on the missing
  `blocks_motion_in_heightmap` tag (and would then error on
  `blocks_fluid_flow`) — but `Assets::load` is only called from render
  tests/examples, not from the game's production startup path, so this
  breaks resource-loading tests, not the game itself. (d) `motion.rs`
  `MotionResources::load` requires the absent `#causes_suffocation` /
  `#suppresses_bounce` tags (`motion.rs:142,145`, `?` on a helper that
  errors "missing motion tag {name}"; both absent from the prepared
  203-key bundle) — the 26.1 run's B3-G22 startup blocker reproduces
  in 1.21.11 (Batch 3's scope); it masks the client-side observable
  consequences of B1-G07. (e) The B1-G08 air-drag regression also
  affects dropped items (`entities.rs`: Y drag widens 0.98F) and TNT
  (`scale_air_drag` is shared with `PrimedTnt.tick`) — both out of
  Batch 1 scope. (f) The palette-reader question (common.md: 1.21.11's
  `chunk.rs` has no `from_palette_nbt`, `BlockState::from_nbt` reads
  the strict `"id"` key only) is Batch 11's scope — not investigated
  here. (g) B11-G06 (number providers) and B11-G13 (player data path)
  REPRODUCE with fresh 1.21.11 evidence (see §Re-checks) but are NOT
  minted in this slice — Batch 11 owns that namespace.
- **No cargo builds, no unit tests, no oracle measurement** were run in
  this batch (brief constraints + the shared 6 GB VM). All verdicts are
  javap + source + data comparisons. Oracle triage used the MEASURED
  1.21.11 Batch 0 table (landed during the batch) plus the raw
  big-compile logs; every missing symbol was javap-verified against the
  1.21.11 jar.
- **Check-count hygiene.** The 95 checks count only comparisons where
  both sides were actually read. Overlapping sub-checks were not
  double-counted (e.g. cauldron receive vs cauldron_tick are separate
  checks only where the vanilla methods are separate, and the drip-fill
  failure is counted once in §G, not again in §H; the dripstone
  thickness calc is one check covering both call sites, not two);
  dead-code matches were not counted as passes (§G, §I); untraced items
  were not counted as passes. Per-section counts: §A 18/15/3, §B 10/10/0,
  §C 17/16/1, §D 7/6/1, §E 7/5/2, §F 13/12/1, §G 11/6/5, §H 8/8/0,
  §I 4/0/4 — summing to 95 checks / 78 pass / 17 gap.
- **Transient disk quota (EDQUOT).** Writes to `~/workspace` failed
  briefly around 01:03 UTC on 2026-11-25 (same window Batch 0's first
  measurement run died) despite `df` showing 46G free — a transient
  quota burst under concurrent load, not a full disk. Writes recovered
  within the hour; no deliverable was lost and no partial numbers needed
  reconciling.

## Evidence index

All under `~/workspace/mcir-work/sweep-1.21.11-batches/b01/evidence/`:

- `javap/` — ~30 `javap -c`/`-p` dumps against the named 1.21.11
  server-game jar: FlowingFluid (canHoldAnyFluid 1012-1075, getFlow
  0.8888889f, canConvertToSource), LavaFluid (randomTick radius-rule
  gate, blocksMotion stop offset 115–123), WaterFluid (slope 4 / dropOff
  1 / tickDelay 5), PointedDripstoneBlock (extends Block, 0 BlockTags
  refs, constants 0.011377778 / 0.17578125 / 0.05859375, calculateTipDirection
  + calculateDripstoneThickness same-block), FallingBlockEntity (0.98d
  ldc2_w offset 889, concrete-powder water check + COLLIDER/SOURCE_ONLY
  clip, timeout 100/600), AbstractThrownPotion (Potions.WATER gates,
  WATER_SENSITIVE_OR_ON_FIRE, (4,2,4) box, dist²<16, rehydrate),
  Axolotl (rehydrate +1800), Entity (classic fluid interaction:
  updateFluidHeightAndDoFluidPushing, updateFluidOnEyes,
  updateInWaterStateAndDoWaterCurrentPushing with the boat short-circuit,
  baseTick fire phase, applyEffectsFromBlocks −20 reset, bubble constants
  −0.9/0.03/1.8/0.1/−0.3/0.06, igniteForTicks/igniteForSeconds/clearFire/
  lavaIgnite; is(Entity) but NO is(TagKey)), BlockTags (no
  BLOCKS_FLUID_FLOW / SPELEOTHEMS / WASHED_AWAY_BY_FLUIDS /
  BLOCKS_LAVA_FIRE_SPREAD / BLOCKS_MOTION), FluidTags (exactly 2
  fields), BiomeTags (no INCREASED_FIRE_BURNOUT), GameRules (59 fields
  in the gamerules package, no RULE_ prefix, FIRE_SPREAD_RADIUS_AROUND_PLAYER
  default 128, water/lava source-conversion defaults true/false) +
  GameRules-c (static init), FireBlock (bootStrap 206 setFlammable
  entries, tick delay 30+10, burnout ATTRIBUTE read), BaseFireBlock
  (fireIgnite: −1 countdown, player nextInt(1,3), igniteForSeconds(8.0);
  getState), InsideBlockEffectType (5 values, bootstrap method refs),
  ConcretePowderBlock, BubbleColumnBlock (0 tag refs, hardcoded soul
  sand/magma, updateShape tick 5), Cauldrons (events 1046/1047, content
  heights 15/16 and (6+3·level)/16, LavaCauldron CLEAR_FREEZE +
  LAVA_IGNITE + runAfter), FluidState (getHeight only, no isFull, no
  getHeightForCamera), ChunkMap (13-param ctor), ServerLevel (11-arg
  ctor ending in RandomSequences; canSpreadFireAround; no
  getNextEntityId), Level (no getNextEntityId), EnvironmentAttributes
  (FAST_LAVA + INCREASED_FIRE_BURNOUT), EnvironmentAttributeSystem /
  EASBuilder, Attributes (no FRICTION_MODIFIER / AIR_DRAG_MODIFIER;
  BURNING_TIME + WATER_MOVEMENT_EFFICIENCY exist), LivingEntity
  (igniteForTicks BURNING_TIME scaling; travelInWater/travelInLava/
  jumpOutOfFluid PRESENT protected; floatInLiquidWhileRidden/
  computeModifiedFriction/omnidirectionalAirMover absent), LevelResource
  (PLAYER_DATA_DIR = "playerdata", PLAYER_OLD_DATA_DIR = "players"),
  CampfireBlock (dowse exists), ShovelItem (useOn → CampfireBlock.dowse),
  AnvilBlock (events 1031/1029), DragonEggBlock (sipush 1000, delay 5),
  BlockStateBase (blocksMotion), IceBlock (playerDestroy uses
  blocksMotion()), AbstractBoat (vehicle.boat package, 3-arg ctor),
  FallingBlock (delay 2, isFree), SharedConstants (tryDetectVersion
  present), BlockState (3-arg ctor), StateHolder, ProtoChunk,
  PalettedContainerFactory, LevelReader, Fluid, Strategy,
  HolderReference, Blocks, GameRules.
- `data/tag-files-1.21.11.txt` — client-jar tag file inventory (666
  files; fluid: 2 files; potion: ZERO files; key block tags
  present/absent).
- `data/server-game-listing.txt`, `data/client-named-listing.txt` — full
  jar listings (class-absence claims: SpeleothemBlock 0,
  EntityFluidInteraction 0, Prediction 0, EntityTypes/BlockEntityTypes
  0, ResourceLocation 0; class-presence claims: world.attribute package
  53 classes, gamerules package 9 classes, vehicle.boat package 8
  classes, Identifier present).
- `data/vanilla-waterlogged.json` — vanilla side of the §A census
  (410 blocks with a waterlogged property).
- `data/fire-vanilla-1.21.11.json` — vanilla side of the §C census (206
  setFlammable entries).
- `data/waterlogging-jar-side.json` — class-hierarchy parse of both jars
  (41 classes changed, 0 in the block package).
- Named jars: `~/workspace/mcir-work/e6-deltas/versions/1.21.11/jars/`
  (server-game-named.jar sha1 7f0d85c1…, client-named.jar sha1
  29bddf14…, client.jar sha1 ba2df812… — all verified before use;
  server.jar is a bundler jar, never javap'd).
- Prepared data (real 1.21.10 prepare — no 1.21.11 prepare exists):
  `~/workspace/mcir-work/candidate-test-1.21.10/.local/horizon-client/1.21.10/`
  (block-behaviours.json, block-tags.json).
- Measured 1.21.11 Batch 0 table (oracle authority):
  `~/workspace/mcir-work/sweep-1.21.11/docs/sweep/1.21.11/batch-0-oracles.json`
  + raw logs at `~/workspace/mcir-work/sweep-1.21.11-batches/b00/evidence/big-compile-server.log`.

## Ledger

`slice.json` in this directory — 20 items (B1-G01–G08 gap, B1-G09
closed, B1-G10–G11 fixed, B1-O01–O09 oracle-defect), checks 95/78/17.

## Verdict

Batch 1 for 1.21.11 is complete: 95 checks, 78 pass, 17 gap, 20 items.
All 21 of the 1.21.10 run's items are decided — 18 reproduce with fresh
1.21.11 evidence, 1 withdrawn (B1-G09, on vanilla's attribute change),
2 fixed again (B1-G10 bubble columns, B1-G11 fire rules under the new
radius rule) — and all 8 of the 1.21.7 run's items are decided — 2 fixed
(G1, G4) and 6 reproducing. This tree is the era template, and it shows
both ways: the dripstone dispatch was adapted here (five behaviours work
that were dead in 26.x), the bubble-column code is vanilla's hardcoded
form, and the fire gate implements 1.21.11's radius rule — but the
headline defect stands: no fluid spreading at all (B1-G05, major),
alongside the half-dead dripstone system (B1-G01, major) and dead splash
water (B1-G03, major), all template-native defects the 1.21.10 child
inherited. One item withdrew outright: fire burnout (B1-G09) is correct
here because vanilla 1.21.11 adopted the environment-attribute form the
template already ships — the one place vanilla moved toward the port.
Nothing is unproven and nothing stalled: every in-scope check was decided
statically against 1.21.11's own jar/data, every oracle failure was read
from the measured Batch 0 table and its big-compile logs, and the only
open re-verification is the two censuses against a real 1.21.11 prepare
when one exists (no verdict depends on them).
