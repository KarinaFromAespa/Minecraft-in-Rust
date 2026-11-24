# Batch 2 — Redstone + pistons (26.2 parity sweep)

**Version:** 26.2 · **Date:** 2026-11-24 · **Branch:** `parity-sweep-26.2`
(workdir `~/workspace/mcir-work/sweep-26.2`, synced to main `732c800f`) ·
**Scope:** `docs/PARITY-SWEEP.md` §4 Batch 2 — redstone ticking, pistons,
observers, comparators; 10 drivers + circuit probes; pilot leftovers
(4 level::tests).

**Status: DRAFT — checks + triage only, no fixes.** No file under
`versions/26.2/` was modified, nothing was committed, no cargo builds ran.
Everything below is static analysis: `javap -c` on the named 26.2
server-game jar
(`~/workspace/mcir-work/26.1-adapters/versions/26.2/jars/server-game.jar`,
Mojang-mapped classes AND methods), source comparison against
`versions/26.2/crates/`, the 26.2 datagen reports
(`~/workspace/mcir-work/26.2/versions/26.2/datagen/reports/`), and the 26.2
client jar datapack (`leakage-audit/jars/26.2-client.jar`). No prepared 26.2
data exists (`.local` deleted), so per-state data checks that need
`gameplay-facts.json` are recorded as needs-harness, not faked.

**Coordinator note (2026-11-24, after this batch ran):** every oracle
claim in this report was verified against the measured Batch 0 table
(`docs/sweep/26.2/batch-0-oracles.json`) — no corrections needed.
`horizon-prepare` for 26.2 finished after this batch ran (2048.7 s,
EXIT 0); `.local/horizon-client/26.2/` now holds `block-behaviours.json`,
`gameplay-facts.json` and the rest (203 MB), so B2-H01/H02's data
precondition is met — the diffs and driver runs themselves remain
follow-up work.

## Tally

- **Behaviour checks: 45 — 42 pass, 3 gap** (across 3 defect items
  B2-G01–B2-G03, all minor).
- **Oracle/driver defects: 3** (B2-O01–B2-O03, all minor — not counted in
  the check tally, following the 1.21.7 convention).
- **Needs-harness: 3** (B2-H01–B2-H03).
- **Withdrawn candidates: 2** (1.21.7's G11 and G12 do not apply to 26.2 —
  vanilla changed; the port matches 26.2's form).
- **No majors in this batch.** All three 1.21.7 majors (G7, G8, G9) are
  FIXED in the 26.2 tree — the 1.21.11 template fixes carried over.

## Defects

### B2-G01 — `is_pushable` Y-bound off by one at the ceiling (MINOR, NEW; 1.21.7 G10 survives)

**Check failed (1):** pushing upward at the build ceiling.

Vanilla 26.2 `PistonBaseBlock.isPushable`
(`net.minecraft.world.level.block.piston.PistonBaseBlock`, jar-verified)
bounds-checks `y < getMinY() || y > getMaxY()` — y == maxY (320) is
allowed — and refuses only an UP-push *from* y == maxY (bytecode offsets
0–34 and 106–125; overworld `dimension_type`: min_y −64, height 384, so
maxY = 320). The port (`level/pistons.rs:337`) uses
`max_y = self.level.max_y() - 1` (319; `LevelAccess::max_y()` default is
320, `level/mod.rs:606`): it refuses y == 320 outright and refuses
UP-pushes from y == 319. A piston at y == 319 pushing up fails in the port
(the y == 320 air cell reads unpushable) where vanilla extends. The floor
side matches vanilla exactly (`min_y` −64, Down-push refused only from
y == min_y, `pistons.rs:356-358`).

### B2-G02 — Piston-shape driver hardcodes the 26.3 state count (MINOR, harness; 1.21.7 G13 survives)

`tools/test_piston_shape_parity.py:63` asserts
`len(facts['states']) == 35723` — the 26.3 count. 26.2 has 32,366 states
(`docs/sweep/26.2/inventory.md`, from the 26.2 datagen reports), so the
driver fails outright on 26.2 before comparing anything. The oracle itself
(`VanillaPistonShapeOracle`) compiles clean against the 26.2 jars (Batch 0
measurement, all three classpaths pass); the driver constant is the defect.
No other Batch 2 driver hardcodes 35723 (grep over all 10 drivers + the
redstone/piston helper tools — only this line).

### B2-G03 — Documented 0.01 piston-push height divergence (MINOR, pre-existing; 1.21.7 G14 survives)

`tools/test_projectile_piston_cobweb_parity.py:16` documents in its own
docstring that "vanilla's second push here is 0.01 higher than Horizon's,
by the cobweb or not", and deliberately does not compare height. Cause
unknown; the driver passes around it. Recorded here as a known divergence,
not an accepted gap (no cause or owner established). Note this is about
entity lift height, not the `Entity.limitPistonMovement` clamp — that
clamp itself matches vanilla 26.2 exactly (see passes).

## Fixed in 26.2 (1.21.7 majors — all three verified gone)

### G7 — Redstone wire class name: FIXED

Vanilla 26.2 names the class `RedStoneWireBlock` (capital S; jar-verified,
and `RedstoneWireBlock` does not exist in the jar). The 26.2 port's class
table carries the dual name: `level/classes.rs:227`
`RedstoneWire = "RedstoneWireBlock" | "RedStoneWireBlock"` (with a unit
test at `classes.rs:351-363` asserting both spellings map). All wire arms
dispatch: `hooks.rs:100` (neighborChanged → `wire_update_power`),
`hooks.rs:211` (onPlace), `hooks.rs:380` (affectNeighborsAfterRemoval),
`shapes.rs:698` (updateShape), `redstone.rs:65,101` (signal),
`interact.rs:263` (dot/cross toggle), `place.rs:1089`, `survive.rs:281`.
The `horizon-prepare` spot the 1.21.7 report flagged is fixed too:
`crates/horizon-prepare/src/survival.rs:44` now reads
`("RedStoneWireBlock", "RedstoneWire")`. Batch 11's B11-G02 cross-ref is
resolved for 26.2.

### G8 — Push-reaction loader mapping: FIXED

Vanilla 26.2's `PushReaction` enum is `NORMAL, DESTROY, BLOCK, IGNORE,
PUSH_ONLY` (jar-verified — the pre-26.3 names). The 26.2 loader
(`level/data.rs:886-893`) matches both the old and the 26.3 names:
`"PUSH" | "PUSH_ONLY" => Push`, `"POPPED" | "DESTROY" => Popped`,
`"IMMOVEABLE" | "BLOCK" => Immoveable`,
`"IGNORE_ENTITY" | "IGNORE" => IgnoreEntity`, default `PushPull`.
DESTROY blocks pop, BLOCK blocks refuse, PUSH_ONLY (glazed terracotta) is
push-only. The `is_pushable` reaction switch (`pistons.rs:368-372`) matches
vanilla's switch map exactly (verified against
`PistonBaseBlock$1`: BLOCK→false, DESTROY→allow_destroyable,
PUSH_ONLY→direction==connection). Caveat: no prepared 26.2
`gameplay-facts.json` exists to diff the per-state values directly; the
check is loader-source vs jar-enum, and the loader's own comment
(`data.rs:886`) states the 26.2 facts carry the old names.

### G9 — Obsidian-family refusal in `is_pushable`: FIXED

Vanilla 26.2 `isPushable` refuses `OBSIDIAN`, `CRYING_OBSIDIAN`,
`RESPAWN_ANCHOR` and `REINFORCED_DEEPSLATE` explicitly before the reaction
switch (bytecode offsets 44–85). The port has all four
(`level/pistons.rs:349-353`), with a comment noting 26.3 gave them
IMMOVEABLE instead and dropped the list — the 26.2 tree correctly carries
the 26.2 form, not the 26.3 form.

## Withdrawn candidates (recorded explicitly — numbers not reused)

### G11 — Comparator conductor branch: WITHDRAWN, vanilla changed

1.21.7's G11 was that vanilla takes the item frame first and the port
takes `max(frame, block)`. In 26.2, vanilla CHANGED: `ComparatorBlock.
getInputSignal` (bytecode offsets 98–164) computes
`Math.max(frameAnalogOrMIN, blockAnalogOrMIN)` and uses it unless both are
absent — i.e. vanilla 26.2 takes the max, exactly as the port does
(`redstone.rs:543-549`, `frame.into_iter().chain(block).max()`). The
port matches 26.2; there is no defect here. (The 1.21.7 G11 stands for
1.21.7 only.)

### G12 — Piston-destroyed blocks notify twice: WITHDRAWN, vanilla changed

1.21.7's G12 was that vanilla destroys via `Level.destroyBlock` (flags 3,
neighbours update during the destroy loop) plus an explicit loop, while
the port sets air with flags 18. In 26.2, vanilla CHANGED: `moveBlocks`'
destroy loop calls `setBlock(air, 18)` (bytecode offset 302, `bipush 18`),
then runs the explicit `updateNeighborsAt` loop after — exactly what the
port does (`pistons.rs:611`, `set_block(*pos, b.air(), 18)`, explicit loop
at `pistons.rs:679-683`). The port matches 26.2; there is no defect here.
(The 1.21.7 G12 stands for 1.21.7 only.)

## Oracle defects (for Batch 0 — do not repair here)

From the 26.2 Batch 0 measurement
(`docs/sweep/26.2/batch-0-measurement.md` + `batch-0-oracles.json`,
2026-11-24, 391 oracles measured, no repairs): Batch 2's oracles are in
far better shape for 26.2 than for 1.21.7. **Five of the six Batch 2
oracles compile clean on all classpaths**; only one is broken.

### B2-O01 — `VanillaRedstoneCatalogOracle` BROKEN, 1 compile error (MINOR)

Batch 0: fails on all three classpaths, 1 error, "188: cannot find
symbol". The symbol is `datapackWorldRegistries()` on the `WorldLoader`
load-context (`VanillaRedstoneCatalogOracle.java:188`):
`context.datapackWorldRegistries().lookupOrThrow(Registries.WORLD_PRESET)`.
26.2's `WorldLoader$DataLoadContext` (jar-verified) has no such accessor —
its record components are `resources()`, `dataConfiguration()`,
`datapackWorldgen()`, `datapackDimensions()`. `datapackWorldRegistries()`
is the 26.3 name. The 26.2 equivalent for the world-preset lookup is
`datapackDimensions()` (a `RegistryAccess.Frozen`, which has
`lookupOrThrow`). Blocks `test_redstone_catalog_parity` only.

### B2-O02 — `redstone_diff.py` defaults `--resources` to the 26.3 path (MINOR)

`tools/redstone_diff.py:453`:
`p.add_argument('--resources', default=str(ROOT / '.local/horizon-client/26.3'))`.
On 26.2 the default points at data that does not exist (and the docstring,
line 2, says "vanilla 26.3"). Overridable via `--resources`, so minor, but
the default is wrong for this tree.

### B2-O03 — `piston_step_e2e.py` defaults `--resources` to the 26.3 path (MINOR)

`tools/piston_step_e2e.py:31`:
`parser.add_argument('--resources', type=Path, default=ROOT / '.local/horizon-client/26.3')`.
Same shape as B2-O02 (docstring line 2: "actual vanilla 26.3 server").
Overridable, minor.

### Oracles assessed PASSING for 26.2 (Batch 0 measurement)

- `VanillaBlockUpdateOracle` — passes all three classpaths (1.21.7's O1,
  ~25 breaks, is gone for 26.2).
- `VanillaRedstonePersistenceOracle` — passes all three (1.21.7's O3 gone).
- `VanillaPistonPushOracle` — passes all three (1.21.7's O4 gone; 26.2 has
  `net.minecraft.world.entity.monster.zombie.Zombie`).
- `VanillaPistonMovementOracle` — passes all three (1.21.7's O5 gone).
- `VanillaPistonShapeOracle` — passes all three (its driver is broken by
  B2-G02, not the oracle).
- `VanillaClientRedstonePreviewOracle` — passes on the client classpath
  (its real classpath); fails on server cp only because it is client-only
  (expected, not breakage).
- Helpers `VanillaCollisionOracle`, `VanillaBlockStateCodecOracle`,
  `VanillaFaceSupportOracle`, `ParityData` — all pass on all classpaths.

## Needs-harness

### B2-H01 — Per-state weak/strong signal nibbles + button press times need prepared 26.2 data

The lever/button/torch/redstone-block weak/strong nibbles and the
`ticksToStayPressed` values (stone 20, wood 30, …) live in prepared
`gameplay-facts.json` / `block-behaviours.json`, and no prepared 26.2 data
exists. The logic is jar-verified (`LeverBlock.getDirectSignal`: powered
&& connected direction → 15; `ButtonBlock.press`/`tick`/`checkPressed`
structure; the port reads `ticks_to_stay_pressed` from the export at
`data.rs:1018`), but the per-state values cannot be checked until a 26.2
prepare run lands. Design: prepare 26.2, then diff the nibble tables
against the jar-derived expectations per state (same method as the 1.21.7
batch).

### B2-H02 — Tick-by-tick circuit equivalence + piston corpus driver runs

The 10 Batch 2 drivers (circuits, catalog contraptions, piston push/shape/
movement corpus, persistence round-trips) need (a) a 26.2 prepare run
(no `.local/horizon-client/26.2/` exists) and (b) the B2-O01 catalog-oracle
repair. The static checks above cover the logic; the drivers cover the
corpus. 9 of 10 drivers' oracles already compile for 26.2.

### B2-H03 — The 4 level::tests need a cargo test run

`level/tests.rs` carries `redstone_lever_wire_and_lamp` (:1253),
`levers_power_wire_through_interaction` (:1734),
`pistons_push_and_sticky_pistons_pull` (:1781) and
`pistons_pop_fragile_blocks_and_refuse_obsidian` (:2120). The three pilot
failures' root causes (G7/G8/G9) are all fixed in this tree, so the tests
are expected to pass — but no cargo builds were allowed in this batch
(shared 6 GB VM), so a test run is still needed to confirm behaviourally.

## Passes (42 checks — javap + source/data agree)

**Wire (now live, G7 fixed):** power math (`calculateTargetStrength`:
block-signal 15 short-circuit, else max with incoming wire signal);
block signal (`shouldSignal` off → `getBestNeighborSignal` → on);
incoming signal (horizontal neighbours, conductor-above / non-conductor-
below rules, −1 decay, floor 0); connection state (dot early-return +
straight-line fixups, same order as vanilla: West/East when N/S empty,
North/South when E/W empty); connecting side (up-connection via trapdoor/
survive + face-sturdy, conductor/below rules); `getSignal` direction rules
(down=0, up=power, opposite-direction connection check);
`updatePowerStrength` neighbour updates (setBlock flags 2, HashSet of pos
+ six neighbours, `updateNeighborsAt` each — the port reproduces the
HashSet order in `java_hash_set_order`); `shouldConnectTo` arms
(wire/repeater-facing-or-opposite/observer-facing/signal-source).

**Machinery:** `SignalGetter` defaults live on the interface in 26.2
(`getSignal`/`hasNeighborSignal`/`getBestNeighborSignal`/`getDirectSignal`/
`getControlInputSignal` — the last bytecode-verified against the port's
`control_input_signal`: diodes-only → diode direct signal, redstone block
→ 15, wire → power, signal source → direct signal, else 0);
`NeighborUpdater.UPDATE_ORDER` is West/East/Down/Up/North/South in the jar
(static init verified) and in the port (`update.rs:15`); `Direction` enum
order matches vanilla (Down=0 … East=5, jar + `geometry/mod.rs:44`).

**Consumers:** lamp on/off (immediate on, 4-tick off delay); torch
inversion and burnout (8 toggles / 60-tick window / 160-tick relight /
level event 1502 — all constants jar-verified); repeater delay (2×delay),
lock, `sideInputDiodesOnly` true for repeaters / false for comparators
(`DiodeBlock` default false, jar-verified; `ComparatorBlock` does not
override); diode tick (lock, powered toggle, VERY_HIGH reschedule);
`checkTickOnNeighbor` priorities (ExtremelyHigh/VeryHigh/High);
diode input (incl. wire-power fallback); diode alternate signal;
comparator `shouldTurnOn` (input > side, or input == side in compare
mode); comparator subtract/compare output math; comparator container
measurement (analog output, conductor pass-through with the 26.2 max rule,
2-tick refresh, stored output in `comparator_refresh` — matches
`refreshOutputState` including the `old != output || mode == COMPARE`
gate); dispenser trigger (signal at pos or above, 4-tick delay,
`triggered` flag); button press/release (press sets powered + schedules
the release tick, tick releases when unpressed, arrow `checkPressed` with
reschedule — values data-driven, see B2-H01); hopper power lock
(`enabled = !signal`); daylight detector (effective sky brightness,
inversion, cos sun shaping, clamp — the port's `.max(0)` before inversion
is harmless, same as 1.21.7); target block (distance strength, ceil,
min 1, arrow 20 / other 8 ticks); copper bulb `checkAndFlip` (cycle lit +
sound on the rising edge only, set powered=signal flags 3).

**Observers:** 2-tick pulse on/off, `updateNeighborsInFront`, trigger via
`updateShape` on the facing side only when unpowered, `startSignal`
(schedule 2 ticks if none scheduled), `onPlace` and
`affectNeighborsAfterRemoval` arms (`hooks.rs:200,368`).

**Pistons:** quasi-connectivity (`getNeighborSignal` — direct neighbours
except the push direction, signal at self/down, and the above-block scan
except down); `checkIfExtend` events 0/1/2 with the moving-piston progress
rule (progress < 0.5 or same-tick or handling-tick); resolver structure
(`resolve` / `addBlockLine` / `reorderListAtCollision` / branching blocks)
and the 12-block limit (`MAX_PUSH_DEPTH = 12`, jar constant;
`bipush 12` twice in `addBlockLine`); `moveBlocks` structure and flags
(276 head removal / 18 destroy / 324 moving blocks / 82 head placement,
destroy-then-push order, shape updates at limit 512); `triggerEvent`
(server-side signal re-check, extend/retract, sticky-pull guard:
event 1, non-air, pushable, NORMAL reaction or piston); moving-piston
tick/finalTick (progress +0.5/tick clamped to 1.0, deathTicks client wait
< 5, waterlogged strip, air fallback flags 340 + updateOrDestroy flags 3,
normal path flags 67 + neighborChanged); entity piston movement limit
(±0.51 blocks/tick/axis, first-nonzero-axis only, 1e-5 threshold,
per-tick reset, lengthSqr 1e-7 early-out — `Entity.
limitPistonMovement` + `applyPistonMovementRestriction` match
`piston_movement.rs`); slime/honey stick rules (`canStickToEachOther`:
honey/slime mutually exclusive, either sticky → stick); piston head
survival (fitting base or same-facing moving piston).

## Pilot leftover diagnosis (summary)

| Test (`level/tests.rs`) | Verdict | Root cause |
|---|---|---|
| `redstone_lever_wire_and_lamp` (:1253) | Root cause fixed (G7) — run owed (B2-H03) | Wire class now dispatches |
| `levers_power_wire_through_interaction` (:1734) | Root cause fixed (G7) — run owed (B2-H03) | Same |
| `pistons_push_and_sticky_pistons_pull` (:1781) | Expected pass — run owed (B2-H03) | Passed on 1.21.7; logic unchanged |
| `pistons_pop_fragile_blocks_and_refuse_obsidian` (:2120) | Root causes fixed (G8 + G9) — run owed (B2-H03) | Loader + obsidian hardcode |

No pilot failure remains unexplained for 26.2. The 26.2 tree's port code
is the 1.21.11-template lineage: the three 1.21.7 majors were fixed at the
template and carried into 26.2, and the two 1.21.7 minors that survive
here (G10) or partially (G13/G14 drivers) are the template's remaining
gaps, not new 26.2 regressions.

## Method notes / limitations

- **No driver runs, no cargo tests.** Blocked by the batch constraints
  (no builds on the shared VM; no prepared 26.2 data; one oracle broken).
  The 10 Batch 2 drivers were read; 9 of 10 have compiling oracles for
  26.2 (Batch 0 measurement), so the driver suite is one oracle repair +
  one prepare run away from runnable — unlike 1.21.7, where five oracles
  were broken.
- **Vanilla 26.2 vs 1.21.7 deltas found in scope:** piston classes moved
  to `net.minecraft.world.level.block.piston.*`; wire power computation
  moved behind `RedstoneWireEvaluator` /
  `DefaultRedstoneWireEvaluator` (same math, new home);
  `ComparatorBlock.getInputSignal` now takes max(frame, block) (was
  frame-first); `moveBlocks` now destroys with `setBlock(air, 18)` (was
  `destroyBlock` flags 3); `PushReaction` keeps the pre-26.3 names
  (26.3 renamed them); the obsidian-family list is still explicit in
  `isPushable` (26.3 dropped it for IMMOVEABLE). Each of these was
  verified against the 26.2 jar, not assumed from 1.21.7 or 26.3.
- **Not re-verified here:** the experimental redstone evaluator
  (`redstone/experimental.rs` — behind the `redstone_experiments` flag,
  off in the test level); dispense/dropper *behaviours* beyond the
  redstone trigger; sculk sensors (own family); command-block powering
  beyond the `neighborChanged` arm; the per-state weak/strong nibbles
  (B2-H01); the full class-table coverage sweep (needs
  `block-behaviours.json`, which does not exist for 26.2 — the Batch 2
  classes were all verified present in `classes.rs` individually).
- **Adjacent observation (not a Batch 2 finding):** the 35723 hardcode
  appears in many other drivers outside Batch 2 scope
  (`test_block_properties_parity.py`, `test_collision_parity.py`, …) —
  the same one-line fix (read the count from the facts) applies
  programme-wide; flagged for the coordinator.
- **Disk:** javap dumps were written to `/tmp/b2/` (ephemeral) and the
  deliverables are the only files written by this batch.
