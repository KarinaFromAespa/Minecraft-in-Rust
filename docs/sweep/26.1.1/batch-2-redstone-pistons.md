# Batch 2 — Redstone + pistons (26.1.1 parity sweep)

**Version:** 26.1.1 · **Date:** 2026-11-24 · **Branch:** `parity-sweep-26.1.1`
(workdir `~/workspace/mcir-work/sweep-26.1.1`) ·
**Scope:** `docs/PARITY-SWEEP.md` §4 Batch 2 — redstone ticking, pistons,
observers, comparators; 10 drivers + circuit probes; pilot leftovers
(4 level::tests).

## §0 Execution status

Checks + triage only. No file under `versions/26.1.1/` was modified,
nothing was committed, no git commands ran in the worktree, no cargo
builds ran. Everything below is static analysis: `javap -c` on the named
26.1.1 server-game jar
(`~/workspace/mcir-work/26.1.x/versions/26.1.1/jars/server-game.jar`,
sha1 `63e2f8ca1ccbcfda2f72ad0e8278f975d5a6a011`; the bundler `server.jar`
was never javap'd), source comparison against
`versions/26.1.1/crates/`, the 26.1.1 datagen reports
(`~/workspace/mcir-work/26.1.x/versions/26.1.1/datagen/reports/`), the
prepared 26.1.1 data (`versions/26.1.1/.local/horizon-client/26.1.1/` —
Batch 0's prepare: `block-behaviours.json`, `gameplay-facts.json`,
29,873 states), and the 26.1.1 Batch 0 oracle measurement
(`docs/sweep/26.1.1/batch-0-oracles.json` + `batch-0-measurement.md`,
391 oracles, 189 pass / 202 fail-all).

**Headline:** all six 26.1 behaviour defects REPRODUCE in 26.1.1, and
the 26.1.1 tree carries one NEW defect the 26.1 tree does not have:
`DiodeBlock.neighborChanged` gained the 26.2 same-block guard
(B2-G07) — vanilla 26.1.1 has no guard, and the 26.1 tree correctly
omits it. The 26.1.1 tree is stale relative to the 26.1 tree in
`level/` in several places (see §6): the 26.1 completion fixes never
fully carried across, so 26.2-form behaviour reappears in the 26.1.1
tree — the diode guard is the in-scope instance; bed use, composter,
ice melt, sculk, tripwire, `blocks_motion` and the eyeblossom scan
order are out-of-scope instances flagged for their batches.

## §1 Scope

Redstone wire power/updates, comparators, repeaters, observers, pistons
(push/pull/destroy, `is_pushable`, shape driver), block-update ordering,
redstone persistence. Method: `javap -c` on the named 26.1.1
server-game jar + source comparison against `versions/26.1.1/crates/` +
tag/data reads. No cargo builds, no driver runs. The 4 level::tests and
the circuit-equivalence / signal-nibble checks that need prepared data
or a test run are needs-harness items (B2-Hxx), not faked passes —
except that the nibble/press-time checks were run here as real checks
against the prepared 26.1.1 data, as in the 26.1 run.

## §2 Method

Three verification layers, all fresh against 26.1.1 evidence:

1. **Jar identity proof.** 41 in-scope classes were byte-compared
   (sha1 of the class file) between the 26.1 and 26.1.1 server-game
   jars — all 41 IDENTICAL, including `RedStoneWireBlock`,
   `RedstoneWireEvaluator`, `DefaultRedstoneWireEvaluator`, `DiodeBlock`,
   `ComparatorBlock`, `RepeaterBlock`, `ObserverBlock`,
   `RedstoneTorchBlock`, `RedstoneLampBlock`, `LeverBlock`, `ButtonBlock`,
   `TargetBlock`, `CopperBulbBlock`, `DaylightDetectorBlock`,
   `DispenserBlock`, `HopperBlock`, `PistonBaseBlock`,
   `PistonStructureResolver`, `PistonMovingBlockEntity`,
   `MovingPistonBlock`, `PistonHeadBlock`, `PushReaction`,
   `SignalGetter`, `NeighborUpdater`, `DetectorRailBlock`,
   `PoweredRailBlock`, `TripWireHookBlock`, `TripWireBlock`,
   `SculkSensorBlock`, `NoteBlock`, `DropperBlock`, `CommandBlock`,
   `JukeboxBlock`, `LecternBlock`, `CalibratedSculkSensorBlock` and the
   pressure-plate classes. This is consistent with the documented
   26.1→26.1.1 delta (3 changed classes: `PlayerEntry.<init>`,
   `DetectedVersion.createBuiltIn`, `SharedConstants` version fields —
   none in Batch 2 scope; `versions/26.1.1/PORT-NOTES-26.1.1.md`).
   Every jar-side verification from the 26.1 run therefore carries to
   26.1.1 by identity, not by assumption.
2. **Fresh javap on the 26.1.1 jar** for every defect-bearing method:
   `PistonBaseBlock.isPushable` (obsidian refusal + Y bounds),
   `moveBlocks` (destroy flags), `ComparatorBlock.getInputSignal`
   (max rule), `DiodeBlock.neighborChanged` (no guard), the
   `PushReaction` enum, the `RedStoneWireBlock` class listing, and every
   missing symbol behind the oracle defects (blockscan, `Continuation`,
   `ClockInstance`, spam thresholds, the two ctor arities,
   `datapackWorldRegistries`, the `CubeVoxelShape` ctor). Dumps in
   `evidence/` (§7).
3. **Tree diff.** A full diff of `crates/horizon-world/src/level/`
   between the 26.1 and 26.1.1 trees in this worktree. In-scope files
   `classes.rs`, `pistons.rs`, `shapes.rs`, `place.rs`, `tests.rs` and
   all 7 Batch 2 drivers are byte-identical; `redstone.rs` differs ONLY
   by the diode guard (B2-G07); `hooks.rs`/`interact.rs`/`survive.rs`
   differ only outside the wire/redstone arms (ice melt, bed use,
   composter, comments); `data.rs` differs only by the `cobweb` named
   removal and the two motion-tag additions (out of scope) plus the
   resulting +1 line shift. The 45 pass checks rest on (1)+(2)+(3):
   the jar methods are the ones the 26.1 run verified, and the port
   code deciding each check is unchanged from the 26.1 tree except
   where this report says otherwise.
4. **Fresh data checks.** The push-reaction census, the class-table
   fallback census, the nibble spot-checks and the button press-time
   checks were re-run against the prepared 26.1.1 data
   (`evidence/census_nibbles_26_1_1.py`, log in `evidence/`).

**Counting rule:** one check = one vanilla-method/behaviour comparison
(jar + port source, or jar + prepared data). Wire-logic and
reaction-switch checks count the Rust functions that exist — they match
vanilla method-for-method — while the dispatch/table failures that make
them unreachable are counted once each as the G01/G02 gap checks (the
1.21.7 report's approach, not a pass for dead code). Oracle/driver-path
checks are not in the behaviour tally, except the two driver-constant
checks (G05/G06), counted as in the 26.1 template.

**Note on the 26.1 summary:** the common brief points at a
"26.2-record corrections" section in the 26.1 `summary.md`; no such
file exists in the ref dir (`sweep-26.1.1-ref/26.1/` holds only
`batch-0-oracles.json`, `reports/`, `slices/`, `sweep-items.json`).
The 26.1 template report itself is the authority used here.

## §3 Checks table

- **Behaviour checks: 52 — 45 pass, 7 gap** (across 7 defect items
  B2-G01–B2-G07: 3 major, 4 minor).
- **Behaviour/non-behaviour split: 52 behaviour + 0 non-behaviour.**
  Batch 2 is not one of the three as-tallied exception batches; the
  tally counts behaviour checks only.
- **Oracle/driver defects: 6** (B2-O01–B2-O06: 2 major, 4 minor — not
  counted in the check tally).
- **Needs-harness: 2** (B2-H01–B2-H02).
- **Closed (withdrawn re-checks): 2** (B2-W01, B2-W02 — 1.21.7's G11
  and G12 do not apply to 26.1.1; vanilla changed, the port matches
  26.1.1's form).
- **1.21.7 majors: all three REPRODUCE in 26.1.1** (G7, G8, G9 → B2-G01,
  B2-G02, B2-G03). Like the 26.1 tree, the 26.1.1 tree does NOT carry
  the 1.21.11-template fixes: its `classes.rs` has no alias mechanism
  and no `RedStoneWireBlock` spelling, its push-reaction loader matches
  only the 26.3 enum names, and its `is_pushable` has no obsidian-family
  check. The 26.2 tree carries all three fixes; the 26.1.1 tree carries
  none of them.
- **NEW vs 26.1:** B2-G07 (diode same-block guard) — the 26.1 tree
  correctly omits the guard; the 26.1.1 tree adds it. This is the one
  check that flips pass→gap against the 26.1 run (46 pass → 45 pass).

Per-section counts sum to the tally: wire 9 (8 pass + G01), machinery 4
(3 pass + G07), consumers 16, nibbles/data 3 (2 pass + G02), observers
5, pistons 13 (11 pass + G03 + G04), driver constants 2 (G05 + G06)
= 52; 45 + 7 = 52.

## §4 Defects

### B2-G01 — Redstone wire completely non-functional: class-name mismatch (MAJOR; 26.1 B2-G01 reproduces; 1.21.7 G7 reproduces)

**Check failed (1):** wire class dispatch.

Vanilla 26.1.1 names the class `RedStoneWireBlock` (capital S;
jar-verified — `net/minecraft/world/level/block/RedStoneWireBlock.class`
exists, `RedstoneWireBlock` does not; the class file is byte-identical
to 26.1's). The prepared 26.1.1 data is faithful:
`block-behaviours.json` carries redstone_wire's class chain as
`["RedStoneWireBlock", "Block"]`. But the port's class table
(`level/classes.rs:224`, byte-identical to 26.1's) knows only
`"RedstoneWireBlock"`, and — unlike the 26.2 tree — the 26.1.1 tree has
no alias mechanism at all (no `|` alternatives in the `classes!` macro,
no unit test). `Class::from_java` returns `None` for the real name, so
`data.rs:920-928` drops it from the chain and the wire resolves to
`Class::Block`. Every wire arm exists but never runs: `hooks.rs:100`
(neighborChanged → `wire_update_power`), `hooks.rs:211` (onPlace),
`hooks.rs:380` (affectNeighborsAfterRemoval), `shapes.rs:698`
(updateShape), `redstone.rs:65,101` (signal), `interact.rs:263`
(dot/cross toggle), `place.rs:1089`, `survive.rs:281`. Wire power stays
0, wire emits no signal — exactly the 1.21.7/26.1 failure, including the
pilot wire-test failures it explains (see §6). The wire logic itself
matches vanilla 26.1.1 method-for-method (see Passes); the defect is
the one-line table miss. The prepare-side half is unfixed too:
`crates/horizon-prepare/src/survival.rs:43` still reads
`("RedstoneWireBlock", "RedstoneWire")` (horizon-prepare is
byte-identical to 26.1's).

A full class-table census (all 1,168 exported blocks vs the 271 known
classes, `evidence/census_nibbles_26_1_1.py`) finds exactly three blocks
resolving to plain `Block` despite a non-Block chain: `redstone_wire`
(this item), `dirt_path` (`DirtPathBlock`) and `amethyst_block`
(`AmethystBlock`) — identical to 26.1. The latter two are out of Batch 2
scope — flagged for the coordinator in §6. (87 blocks have an
unrecognised most-derived class but a recognised ancestor, also
identical to 26.1.)

### B2-G02 — Push-reaction loader mapping matches no 26.1.1 value: every state loads as PushPull (MAJOR; 26.1 B2-G02 reproduces; 1.21.7 G8 reproduces)

**Check failed (1):** push-reaction loader mapping.

Vanilla 26.1.1's `PushReaction` enum is `NORMAL, DESTROY, BLOCK, IGNORE,
PUSH_ONLY` (jar-verified — the pre-26.3 names, byte-identical class to
26.1's). The prepared 26.1.1 `gameplay-facts.json` carries exactly those
names: NORMAL 22,869 states, DESTROY 6,849, BLOCK 91, PUSH_ONLY 64 (no
IGNORE states exist in 26.1.1) — identical counts to 26.1. But the
loader (`level/data.rs:886-890`) matches only the 26.3 names — `"PUSH"`,
`"POPPED"`, `"IMMOVEABLE"`, `"IGNORE_ENTITY"` — and defaults everything
else to `PushPull`. All 29,873 states load as `PushPull`. Consequences,
from the fresh census (`evidence/census_nibbles_26_1_1.py`, state→block
map from the 26.1.1 datagen `blocks.json`):

- **355 DESTROY blocks** (6,849 states — torches, wire, flowers, buttons,
  doors, beds, shulker boxes, comparators, repeaters, levers, …) are
  pushed instead of popped; no `DropResources` event fires.
- **13 BLOCK blocks** (91 states — anvils, grindstone, lodestone,
  piston, sticky_piston, piston_head, moving_piston, barrier, end/nether
  portals, end gateway) are pushed instead of blocking the move.
- **16 PUSH_ONLY blocks** (64 states — all glazed terracotta) become
  pullable by sticky pistons.

The `is_pushable` reaction switch itself (`pistons.rs:368-372`) matches
vanilla's switch map exactly (BLOCK→false, DESTROY→allow_destroyable,
PUSH_ONLY→direction==connection) — it simply never sees a non-PushPull
value. The 26.2 tree's loader matches both name sets; the 26.1.1 tree's
does not.

### B2-G03 — `is_pushable` lacks vanilla's hardcoded obsidian-family refusal (MAJOR; 26.1 B2-G03 reproduces; 1.21.7 G9 reproduces)

**Check failed (1):** obsidian-family refusal in `is_pushable`.

Vanilla 26.1.1 `PistonBaseBlock.isPushable`
(`net.minecraft.world.level.block.piston.PistonBaseBlock`, jar-verified
fresh — `evidence/pistonbase.txt`, bytecode offsets 44–85) refuses
`OBSIDIAN`, `CRYING_OBSIDIAN`, `RESPAWN_ANCHOR` and
`REINFORCED_DEEPSLATE` explicitly before the reaction switch. All four
are NORMAL reaction with hardness ≠ −1 (obsidian hardness 50.0), so
nothing else in the check stops them. The port (`level/pistons.rs:328-372`,
byte-identical to 26.1's) has no such check — `grep obsidian
pistons.rs` returns nothing — and the 26.2 tree's obsidian block
(`pistons.rs:344-355` there, with the comment noting 26.3 gave them
IMMOVEABLE instead) is absent from the 26.1.1 tree, as are the
`crying_obsidian` / `reinforced_deepslate` named entries in `data.rs`
(the table has `obsidian` at :267 and `respawn_anchor` at :369, but
neither of the other two). A piston extends and pushes obsidian instead
of staying retracted. Explains the obsidian half of the pilot piston
test.

### B2-G04 — `is_pushable` Y-bound off by one at the ceiling (MINOR; 26.1 B2-G04 reproduces; 1.21.7 G10 survives)

**Check failed (1):** pushing upward at the build ceiling.

Vanilla 26.1.1 `isPushable` bounds-checks `y < getMinY() || y >
getMaxY()` — y == maxY (320) is allowed — and refuses only an UP-push
*from* y == maxY (bytecode offsets 0–34 and 106–125, fresh javap in
`evidence/pistonbase.txt`; overworld min_y −64, height 384, so maxY =
320). The port (`level/pistons.rs:337`, byte-identical to 26.1's) uses
`max_y = self.level.max_y() - 1` (319; `LevelAccess::max_y()` default is
320, `level/mod.rs:606`): it refuses y == 320 outright and refuses
UP-pushes from y == 319. A piston at y == 319 pushing up fails in the
port (the y == 320 air cell reads unpushable) where vanilla extends.
The floor side matches vanilla exactly (`min_y` −64, Down-push refused
only from y == min_y, `pistons.rs:344-346`). Same defect as 26.2's
B2-G01.

### B2-G05 — Piston-shape driver hardcodes the 26.3 state count (MINOR, harness; 26.1 B2-G05 reproduces; 1.21.7 G13 survives)

`tools/test_piston_shape_parity.py:63` (byte-identical to 26.1's)
asserts `len(facts['states'])!=35723` — the 26.3 count. 26.1.1 has
29,873 states (`gameplay-facts.json`, 29,873 entries; datagen census
agrees), so the driver fails outright on 26.1.1 before comparing
anything. No other Batch 2 driver hardcodes 35723 (grep over all 10
drivers + the redstone/piston helper tools — only this line). Note the
oracle behind this driver (`VanillaPistonShapeOracle`) is itself
blocked only by cascade (B2-O04), so the driver is two repairs away
from runnable, not one. Same defect as 26.2's B2-G02 (there: 32,366
states).

### B2-G06 — Documented 0.01 piston-push height divergence (MINOR, pre-existing; 26.1 B2-G06 reproduces; 1.21.7 G14 survives)

`tools/test_projectile_piston_cobweb_parity.py:16` (byte-identical to
26.1's) documents in its own docstring that "vanilla's second push here
is 0.01 higher than Horizon's, by the cobweb or not", and deliberately
does not compare height. Cause unknown; the driver passes around it.
Recorded here as a known divergence, not an accepted gap (no cause or
owner established). Note this is about entity lift height, not the
`Entity.limitPistonMovement` clamp — that clamp itself matches vanilla
26.1.1 exactly (see passes). Same defect as 26.2's B2-G03.

### B2-G07 — `DiodeBlock.neighborChanged` carries the 26.2 same-block guard vanilla 26.1.1 does not have (MINOR, NEW vs 26.1)

**Check failed (1):** diode `neighborChanged` guard.

Vanilla 26.1.1 `DiodeBlock.neighborChanged` has NO same-block guard:
the method goes straight to `canSurvive` (bytecode offset 0), then
`checkTickOnNeighbor`, else dropResources + removeBlock +
updateNeighborsAt in all directions (fresh javap,
`evidence/diode.txt`, offsets 0–94; the class is byte-identical to
26.1's). The 26.1 tree carries this correctly — its `redstone.rs` has
an explicit comment noting 26.1 has no guard and 26.2 returns early.
The 26.1.1 tree instead carries the 26.2 form:
`level/redstone.rs:673-675` returns early when the block at the
position is no longer the diode:

```rust
if !self.b.is_block(self.state(pos), block) {
    return;
}
```

This is the ONLY behavioural difference in `redstone.rs` between the
26.1 and 26.1.1 trees (full diff: the 4-line 26.1 comment deleted, the
3-line guard added). Effect: when a neighbour change is delivered to a
diode position whose block has already been replaced, vanilla 26.1.1
still runs the survive/tick path on the passed state (and drops the
diode's resources + notifies neighbours if it cannot survive), while
the port silently does nothing. Edge-case divergence — minor — but a
genuine regression against the 26.1 tree, and the clearest in-scope
instance of the 26.1.1 tree's stale-carry problem (§6).

## §5 Re-check verdicts

### 26.1 re-checks (PRIMARY — all 14 decided)

| 26.1 item | Verdict for 26.1.1 | Evidence |
|---|---|---|
| B2-G01 wire class name (major) | **REPRODUCES** → B2-G01 | Jar has only `RedStoneWireBlock` (byte-identical class); export chain `["RedStoneWireBlock", "Block"]`; `classes.rs:224` knows only `"RedstoneWireBlock"`; no alias mechanism |
| B2-G02 push-reaction loader (major) | **REPRODUCES** → B2-G02 | Jar enum NORMAL/DESTROY/BLOCK/IGNORE/PUSH_ONLY; facts carry those names (22,869/6,849/91/64); loader `data.rs:886-890` matches only 26.3 names |
| B2-G03 obsidian refusal (major) | **REPRODUCES** → B2-G03 | Jar `isPushable` offsets 44–85 list the four blocks; port has no such check; `named` lacks crying_obsidian/reinforced_deepslate |
| B2-G04 Y-bound off by one (minor) | **REPRODUCES** → B2-G04 | Jar allows y == maxY, refuses UP-push only from maxY; port uses `max_y() - 1` (319) |
| B2-G05 shape-driver count (minor) | **REPRODUCES** → B2-G05 | `test_piston_shape_parity.py:63` asserts 35723; 26.1.1 has 29,873 states |
| B2-G06 0.01 height divergence (minor) | **REPRODUCES** → B2-G06 | Cobweb driver docstring documents it; height deliberately uncompared |
| B2-O01 BlockUpdate oracle (major) | **REPRODUCES** → B2-O01 | Batch 0 table: 32 errors, first "450: cannot find symbol" — identical to 26.1; every missing symbol re-verified against the 26.1.1 jar |
| B2-O02 RedstoneCatalog oracle (minor) | **REPRODUCES** → B2-O02 | Batch 0 table: 4 errors, first "188: cannot find symbol" — identical to 26.1 |
| B2-O03 FaceSupport oracle (major) | **REPRODUCES** → B2-O03 | Batch 0 table: 1 error, protected `CubeVoxelShape` ctor — identical to 26.1 |
| B2-O04 PistonShape cascade (minor) | **REPRODUCES** → B2-O04 | Batch 0 table: class "cascade", depends on broken `VanillaFaceSupportOracle` |
| B2-O05 redstone_diff 26.3 default (minor) | **REPRODUCES** → B2-O05 | `redstone_diff.py:453` defaults `--resources` to the 26.3 path (file byte-identical to 26.1's) |
| B2-O06 piston_step_e2e 26.3 default (minor) | **REPRODUCES** → B2-O06 | `piston_step_e2e.py:31` defaults `--resources` to the 26.3 path (file byte-identical to 26.1's) |
| B2-H01 driver runs (minor) | **REPRODUCES** → B2-H01 | Oracles still broken (O01/O02/O03), driver constant still wrong (G05); prepared data present |
| B2-H02 level::tests run (minor) | **REPRODUCES** → B2-H02 | `tests.rs` byte-identical; 3 of 4 still expected to fail on G01/G02/G03 |

No 26.1 item was FIXED in the 26.1.1 tree. The only 26.1→26.1.1
movement in scope is the NEW B2-G07 regression (the 26.1 tree's diode
pass became a 26.1.1 gap).

### 1.21.7 re-check verdicts (all 8 behaviour + 5 oracle, mandatory)

| 1.21.7 item | Verdict for 26.1.1 | Evidence |
|---|---|---|
| G7 wire class name (major) | **REPRODUCES** → B2-G01 | Jar has only `RedStoneWireBlock`; export chain `["RedStoneWireBlock", "Block"]`; `classes.rs:224` knows only `"RedstoneWireBlock"`; no alias mechanism |
| G8 push-reaction loader (major) | **REPRODUCES** → B2-G02 | Jar enum NORMAL/DESTROY/BLOCK/IGNORE/PUSH_ONLY; facts carry those names (22,869/6,849/91/64); loader matches only 26.3 names |
| G9 obsidian refusal (major) | **REPRODUCES** → B2-G03 | Jar `isPushable` offsets 44–85 list the four blocks; port has no such check; `named` lacks crying_obsidian/reinforced_deepslate |
| G10 Y-bound off by one (minor) | **REPRODUCES** → B2-G04 | Jar allows y == maxY, refuses UP-push only from maxY; port uses `max_y() - 1` (319) |
| G11 comparator conductor (minor) | **WITHDRAWN** — vanilla changed → B2-W01 (closed) | 26.1.1 `ComparatorBlock.getInputSignal` takes `Math.max(frame, block)` (offset 150); port takes max (`redstone.rs:544`) |
| G12 destroy notify twice (minor) | **WITHDRAWN** — vanilla changed → B2-W02 (closed) | 26.1.1 `moveBlocks` destroy loop uses `setBlock(air, 18)` (offset 302, `bipush 18`) + explicit `updateNeighborsAt` loop; port matches (`pistons.rs:599`, loop at 671–683) |
| G13 shape-driver count (minor) | **REPRODUCES** → B2-G05 | `test_piston_shape_parity.py:63` asserts 35723; 26.1.1 has 29,873 states |
| G14 0.01 height divergence (minor) | **REPRODUCES** → B2-G06 | Cobweb driver docstring documents it; height deliberately uncompared |
| O1 BlockUpdate oracle (major) | **REPRODUCES** → B2-O01 | 32 errors for 26.1.1 (Batch 0 table + jar-verified causes); 1.21.7 had ~25 |
| O2 RedstoneCatalog oracle (major) | **REPRODUCES, smaller** → B2-O02 | 4 errors for 26.1.1 (was ~15 for 1.21.7, 1 for 26.2) |
| O3 RedstonePersistence oracle (minor) | **FIXED** — passes all three classpaths | Batch 0 table: pass/pass/pass |
| O4 PistonPush oracle (minor) | **FIXED** — passes all three classpaths | Batch 0 table: pass/pass/pass; 26.1.1 has `net.minecraft.world.entity.monster.zombie.Zombie` |
| O5 PistonMovement oracle (minor) | **FIXED** — passes all three classpaths | Batch 0 table: pass/pass/pass |

### Withdrawn candidates (recorded explicitly — numbers not reused; land as closed items B2-W01/B2-W02)

#### G11 — Comparator conductor branch: WITHDRAWN, vanilla changed

1.21.7's G11 was that vanilla takes the item frame first and the port
takes `max(frame, block)`. In 26.1.1, vanilla CHANGED (same as 26.1 and
26.2): `ComparatorBlock.getInputSignal` (bytecode offset 150,
`Math.max`, fresh javap in `evidence/comparator.txt`) computes
`Math.max(frameAnalogOrMIN, blockAnalogOrMIN)` and uses it unless both
are absent — i.e. vanilla 26.1.1 takes the max, exactly as the port does
(`redstone.rs:541-546`, `frame.into_iter().chain(block).max()`). The
port matches 26.1.1; there is no defect here. (The 1.21.7 G11 stands for
1.21.7 only.)

#### G12 — Piston-destroyed blocks notify twice: WITHDRAWN, vanilla changed

1.21.7's G12 was that vanilla destroys via `Level.destroyBlock` (flags
3, neighbours update during the destroy loop) plus an explicit loop,
while the port sets air with flags 18. In 26.1.1, vanilla CHANGED (same
as 26.1 and 26.2): `moveBlocks`' destroy loop calls `setBlock(air, 18)`
(bytecode offset 302, `bipush 18`, fresh javap in
`evidence/pistonbase.txt`), then runs the explicit `updateNeighborsAt`
loop after — exactly what the port does (`pistons.rs:599`,
`set_block(*pos, b.air(), 18)`, explicit loop at `pistons.rs:671-683`).
The port matches 26.1.1; there is no defect here. (The 1.21.7 G12 stands
for 1.21.7 only.)

### Fixed in 26.1.1 (1.21.7 oracle defects, with evidence)

1.21.7's O3, O4 and O5 are FIXED for 26.1.1 — all three oracles pass on
all three classpaths in the Batch 0 measured table
(`docs/sweep/26.1.1/batch-0-oracles.json`):
`VanillaRedstonePersistenceOracle`, `VanillaPistonPushOracle` (26.1.1
has `net.minecraft.world.entity.monster.zombie.Zombie`, the 1.21.7
break), `VanillaPistonMovementOracle`. No 1.21.7 behaviour item was
fixed in the 26.1.1 tree — all three majors reproduce, and G10/G13/G14
reproduce as well.

## Oracle defects (for Batch 0 — do not repair here)

From the 26.1.1 Batch 0 measurement
(`docs/sweep/26.1.1/batch-0-oracles.json` + `batch-0-measurement.md`,
2026-11-24, 391 oracles measured, 189 pass / 202 fail-all). This batch
did not re-measure: the table is the oracle authority. The three
failing Batch 2 oracle sources are byte-identical between the 26.1 and
26.1.1 trees, the Batch 0 counts reproduce 26.1's exactly (32 / 4 / 1 /
cascade), and this batch javap-verified every missing symbol below
against the 26.1.1 jar fresh. The error anatomy from the 26.1 run's
recompile therefore carries to 26.1.1 on verified grounds.

### B2-O01 — `VanillaBlockUpdateOracle` BROKEN, 32 errors (MAJOR)

Batch 0: fails on all three classpaths, 32 errors, first error
"450: cannot find symbol". Blocks `test_redstone_parity` (via
`block_update_harness.py`, which compiles and runs this oracle) plus
5 other drivers, and anchors 6 cascade oracles (`VanillaBrewingOracle`,
`VanillaEnchantDurabilityOracle`, `VanillaFurnaceOracle`,
`VanillaHelmetDamageOracle`, `VanillaOpenedLootOracle`,
`VanillaPortalOracle` — other batches' scope, recorded here for the
cascade). Error anatomy (26.1 recompile; every cause javap-verified
against the 26.1.1 jar by this batch):

- **`net.minecraft.world.level.blockscan` absent from 26.1.1** (5 errors:
  319, 322, 411, 427, 444 — 0 entries in the jar): the `findBlocksIn`
  overrides and the `Scan` class (`extends
  blockscan.OrderedBlockMatcher`, `BlockStateConsumer` params) are
  26.3-only API. `findBlocksIn` does not exist on 26.1.1's `Level` or
  `ServerLevel` either (javap: 0 matches).
- **`net.minecraft.util.Continuation` absent** (3 errors: 436, 439, 442 —
  0 entries in the jar).
- **`net.minecraft.world.clock.ClockInstance` absent** (2 errors at 450,
  + 4 no-override at 451–454 for the anonymous class's
  totalTicks/partialTick/rate/isPaused — 0 entries in the jar).
- **No-override** (15 total): `findBlocksIn` ×2 (319, 322),
  `getNextEntityId` (357 — absent from 26.1.1's `Level`/`ServerLevel`),
  `Scan` overrides ×6 (419, 423, 434, 438, 441, 444 — superclass
  missing), `ClockInstance` methods ×4 (451–454),
  `getCommandSpamThresholdSeconds` / `getChatSpamThresholdSeconds`
  (530, 531 — absent from 26.1.1's `MinecraftServer`; only
  `getCompressionThreshold` matches "threshold").
- **`MinecraftServer` ctor** (508): the oracle's 11-arg super call
  (…, `false`, `null` — the 26.3 `NotificationManager` slot) vs 26.1.1's
  10-arg ctor (javap-verified: `Thread, LevelStorageAccess,
  PackRepository, WorldStem, Optional<GameRules>, Proxy, DataFixer,
  Services, LevelLoadListener, boolean`).
- **`ServerChunkCache` ctor** (621): the oracle's 10-arg super call vs
  26.1.1's 11-arg ctor (javap-verified: `ServerLevel,
  LevelStorageAccess, DataFixer, StructureTemplateManager, Executor,
  ChunkGenerator, int, int, boolean, ChunkStatusUpdateListener,
  Supplier<SavedDataStorage>`).
- Plus 1 "non-static variable super" (424, `Scan.filterState`) and
  3 `statePredicate` cannot-find-symbol (436, 442, 445 — field of the
  missing superclass).

Notably GONE vs 1.21.7's O1: the permission APIs exist in 26.1.1 —
`operatorUserPermissions()` and `getFunctionCompilationPermissions()`
are abstract methods on 26.1.1's `MinecraftServer` (javap-verified in
the 26.1 run; the class is untouched by the 26.1→26.1.1 delta), as
are `useNativeTransport()` and `isSingleplayerOwner(NameAndId)`, and
`environmentAttributes()` exists on 26.1.1's `Level`. The 26.1.1 breaks
are the 26.3-only APIs (blockscan, ClockInstance, Continuation, spam
thresholds) plus the two ctor mismatches.

### B2-O02 — `VanillaRedstoneCatalogOracle` BROKEN, 4 errors (MINOR)

Batch 0: fails on all three classpaths, 4 errors, first error
"188: cannot find symbol". Blocks `test_redstone_catalog_parity` only.
Errors (26.1 recompile; each javap-verified against the 26.1.1 jar):

- **188: `datapackWorldRegistries()`** — cannot find symbol. 26.1.1's
  `WorldLoader$DataLoadContext` (javap-verified,
  `evidence/dataloadcontext.txt`) has no such accessor — its record
  components are `resources()`, `dataConfiguration()`,
  `datapackWorldgen()`, `datapackDimensions()`;
  `datapackWorldRegistries()` is the 26.3 name. The 26.1.1 equivalent
  for the world-preset lookup is `datapackDimensions()` (a
  `RegistryAccess.Frozen`, which has `lookupOrThrow`). Same single break
  as 26.2's B2-O01.
- **212: `MinecraftServer` ctor** — the oracle's `Server` subclass calls
  the 11-arg super (…, `false`, `new NotificationManager()`); 26.1.1's
  ctor is 10-arg (same as B2-O01's 508).
- **464, 465: `getCommandSpamThresholdSeconds` /
  `getChatSpamThresholdSeconds`** — methods do not override; absent from
  26.1.1's `MinecraftServer` (same as B2-O01's 530/531).

### B2-O03 — `VanillaFaceSupportOracle` BROKEN, 1 error (MAJOR)

Batch 0: fails on all three classpaths, 1 error, "45:
`CubeVoxelShape(DiscreteVoxelShape)` has protected access in
`CubeVoxelShape`". Javap-verified fresh: 26.1.1's `CubeVoxelShape` ctor
is `protected CubeVoxelShape(DiscreteVoxelShape)`; the oracle (in
`dev.horizonlab.parity`, outside `net.minecraft.world.phys.shapes`)
calls it at line 45. Per Batch 0 this is the cheapest high-leverage
repair in the whole sweep: it blocks 19 drivers programme-wide and 5
cascade oracles, including Batch 2's `VanillaPistonShapeOracle`
(B2-O04). Shared helper — other batches will record it too; the
coordinator dedups.

### B2-O04 — `VanillaPistonShapeOracle` BROKEN by cascade, no own errors (MINOR)

Batch 0: class "cascade", first error "depends on broken:
`VanillaFaceSupportOracle`". The 26.1 run's recompile confirms it has
no own errors — it fails only via the broken helper — and the oracle
source is byte-identical between the trees. Passes for free once
B2-O03 is repaired; do not schedule separately. (Its driver is
separately broken by B2-G05.)

### B2-O05 — `redstone_diff.py` defaults `--resources` to the 26.3 path (MINOR)

`tools/redstone_diff.py:453` (byte-identical to 26.1's):
`p.add_argument('--resources', default=str(ROOT / '.local/horizon-client/26.3'))`.
On 26.1.1 the default points at data that does not exist (and the
docstring, line 2, says "vanilla 26.3"). Overridable via `--resources`,
so minor, but the default is wrong for this tree. Same as 26.2's
B2-O02.

### B2-O06 — `piston_step_e2e.py` defaults `--resources` to the 26.3 path (MINOR)

`tools/piston_step_e2e.py:31` (byte-identical to 26.1's):
`parser.add_argument('--resources', type=Path, default=ROOT / '.local/horizon-client/26.3')`.
Same shape as B2-O05 (docstring line 2: "actual vanilla 26.3 server").
Overridable, minor. Same as 26.2's B2-O03.

### Oracles assessed PASSING for 26.1.1 (Batch 0 measurement)

- `VanillaRedstonePersistenceOracle` — passes all three classpaths
  (1.21.7's O3 gone).
- `VanillaPistonPushOracle` — passes all three (1.21.7's O4 gone; 26.1.1
  has `net.minecraft.world.entity.monster.zombie.Zombie`).
- `VanillaPistonMovementOracle` — passes all three (1.21.7's O5 gone).
- `VanillaBlockPushOracle` — passes all three (block-pushing oracle,
  in Batch 2 scope).
- `VanillaClientRedstonePreviewOracle` — passes on the client classpath
  (its real classpath) and on `both`; fails on server cp only because it
  is client-only (expected, not breakage).
- Helpers `VanillaCollisionOracle`, `VanillaBlockStateCodecOracle`,
  `ParityData` — all pass on all classpaths.

### Adjacent failing oracle (out of Batch 2 scope — recorded, not minted)

- `VanillaDispenserOracle` — fails all three classpaths, 15 errors,
  first error "212: package EntityTypes does not exist" (Batch 0 group
  G4: the 26.1.1 `EntityType` holder). No Batch 2 driver references it
  (its users are `test_block_removal_parity.py` and
  `test_composter_parity.py`); dispense behaviours beyond the redstone
  trigger are out of Batch 2 scope. Flagged here with its Batch 0 data
  so the owning batch / coordinator routes it; not a B2 item.

## Needs-harness

### B2-H01 — Tick-by-tick circuit equivalence + piston corpus driver runs

The 10 Batch 2 drivers (circuits, catalog contraptions, piston push/shape/
movement corpus, persistence round-trips) need (a) the B2-O01 block-update
oracle repair (the circuit driver's oracle), (b) the B2-O02 catalog-oracle
repair, (c) the B2-O03 face-support repair (piston-shape cascade) and (d)
the B2-G05 driver-constant fix. The prepared-data precondition that
blocked 26.2's H01/H02 is MET for 26.1.1
(`.local/horizon-client/26.1.1/` exists with `block-behaviours.json` +
`gameplay-facts.json`), and the nibble/press-time checks 26.2 deferred to
H01 were run here as real checks (see Passes). The static checks above
cover the logic; the drivers cover the corpus — and, once the oracles
compile, the corpus runs will exercise the G7/G8/G9 defects behaviourally
(a wire/piston corpus cannot pass while the wire is dead and every state
loads PushPull).

### B2-H02 — The 4 level::tests need a cargo test run

`level/tests.rs` carries `redstone_lever_wire_and_lamp` (:1253),
`levers_power_wire_through_interaction` (:1734),
`pistons_push_and_sticky_pistons_pull` (:1781) and
`pistons_pop_fragile_blocks_and_refuse_obsidian` (:2120) — the file is
byte-identical to 26.1's. Unlike 26.2, where the root causes were fixed
and the tests were expected to pass, for 26.1.1 three of the four are
expected to FAIL (see Pilot leftover diagnosis) — but no cargo builds
were allowed in this batch (shared 6 GB VM), so a test run is still
needed to confirm behaviourally. Note the tests load the real prepared
resources and silently skip (the `world!()` macro returns early) when
`.local` is absent — which is why the 26.1.1 crate-tests CI run could
go green while these defects stand: a clean runner has no prepared
data, so the level tests never ran there. A run with
`HORIZON_RESOURCES` pointed at the prepared 26.1.1 data (present in
this worktree) is what would demonstrate the failures.

## Passes (45 checks — javap + source/data agree)

**Wire logic (8 checks — the functions match vanilla; only dispatch is
dead, B2-G01):** power math (`calculateTargetStrength`: block-signal 15
short-circuit, else max with incoming wire signal —
`DefaultRedstoneWireEvaluator`, offsets 0–25; port
`redstone.rs:439-476`); block signal (`getBlockSignal`: shouldSignal off
→ `getBestNeighborSignal` → on — `RedStoneWireBlock`, offsets 0–17; port
`redstone.rs:408-413`); incoming signal (horizontal neighbours,
conductor-above / non-conductor-below rules, −1 decay, floor 0 —
`RedstoneWireEvaluator.getIncomingWireSignal`, offsets 0–165; port
`redstone.rs:421-437`); connection state (dot early-return +
straight-line fixups, same order as vanilla: West/East when N/S empty,
North/South when E/W empty — `getConnectionState`, offsets 36–49,
144–239; port `redstone.rs:293-332`); connecting side (up-connection via
trapdoor/survive + face-sturdy, conductor/below rules —
`getConnectingSide`, offsets 0–141; port `redstone.rs:236-264`);
`getSignal` direction rules (down=0, up=power, opposite-direction
connection check — offsets 0–86; port `redstone.rs:126-146`);
`updatePowerStrength` neighbour updates (setBlock flags 2, HashSet of pos
+ six neighbours, `updateNeighborsAt` each — `DefaultRedstoneWireEvaluator`,
offsets 0–162; port reproduces the HashSet order in
`java_hash_set_order`); `shouldConnectTo` arms
(wire/repeater-facing-or-opposite/observer-facing/signal-source —
offsets 0–95; port `redstone.rs:216-226`).
(Jar classes byte-identical to 26.1's; port code byte-identical to
26.1's in this section.)

**Machinery (3 pass + B2-G07):** `SignalGetter` defaults live on the
interface in 26.1.1 — note it is at `net.minecraft.world.level.
SignalGetter`, not `.redstone.` (`getSignal`/`hasNeighborSignal`/
`getBestNeighborSignal`/`getDirectSignal`/`getControlInputSignal` —
the last bytecode-verified against the port's `control_input_signal`
(`redstone.rs:194-213`): diodes-only → diode direct signal, redstone
block → 15, wire → power, signal source → direct signal, else 0);
`NeighborUpdater.UPDATE_ORDER` is West/East/Down/Up/North/South in the
jar (static init verified) and in the port (`update.rs:15`); `Direction`
enum order matches vanilla (Down=0 … East=5, jar +
`geometry/mod.rs:44`). The fourth machinery check — **`DiodeBlock.
neighborChanged` has no same-block guard in 26.1.1** — FAILS in this
tree (B2-G07): the port adds the guard the 26.2 tree carries. This is
the one 26.1 pass that does not carry to 26.1.1.

**Consumers (16 checks):** lamp on/off (immediate on, 4-tick off delay);
torch inversion and burnout (8 toggles / 60-tick window / 160-tick
relight / level event 1502 — all constants jar-verified; port
`redstone.rs:751-820`); repeater delay (2×delay), lock
(`RepeaterBlock.isLocked` override), `sideInputDiodesOnly` true for
repeaters / false for comparators (`DiodeBlock` default false,
jar-verified; `ComparatorBlock` does not override); diode tick (lock,
powered toggle, VERY_HIGH reschedule); `checkTickOnNeighbor` priorities
(ExtremelyHigh/VeryHigh/High; comparator override: 2-tick, High if
prioritize else Normal, gated on output-or-powered change); diode input
(incl. wire-power fallback); diode alternate signal (control input on
clockwise/counter-clockwise sides); comparator `shouldTurnOn` (input >
side, or input == side in compare mode); comparator subtract/compare
output math (`calculateOutputSignal`: input 0 → 0, alternate > input → 0,
subtract → input − alternate, else input); comparator container
measurement (analog output, conductor pass-through with the 26.1.1 max
rule — G11 withdrawn — 2-tick refresh, stored output in
`comparator_refresh` — matches `refreshOutputState` including the
`old != output || mode == COMPARE` gate); dispenser trigger (signal at
pos or above, 4-tick delay, `triggered` flag); button press/release
(press sets powered + schedules the release tick, tick releases when
unpressed, arrow `checkPressed` with reschedule); hopper power lock
(`enabled = !signal`); daylight detector (effective sky brightness,
inversion, cos sun shaping, clamp — the port's `.max(0)` before
inversion is harmless, same as 26.2); target block (distance strength,
ceil, min 1, arrow 20 / other 8 ticks); copper bulb `checkAndFlip`
(cycle lit + sound on the rising edge only, set powered=signal flags 3).
(Jar classes byte-identical to 26.1's; consumer code byte-identical to
26.1's — the `redstone.rs` diff touches only the diode guard.)

**Nibbles + press times (2 checks — re-run fresh against the prepared
26.1.1 data):** weak/strong nibble spot-checks against jar-derived
expectations, all matching — lever powered floor north (weak 15 all
directions; strong 15 Up only, via `getConnectedDirection`: FLOOR →
UP, verified through the `$SwitchMap`); stone button powered (same
shape); redstone torch lit (weak 15 except Up; strong Down only);
redstone block (weak 15 all, strong 0 all — the port's
`control_input_signal` special case is the correct handling, mirroring
vanilla's); redstone wire power-15 cross (weak/strong 15 except Down);
observer powered north (15 north only); repeater powered north (15
north only); comparator (0 in the export — the export level has no
block entity, so `getOutputSignal` reads 0; the port correctly
special-cases comparators instead of using the nibble). Loader columns
verified: `weak: int(5), strong: int(6)` in `data.rs:879-880` match
the export row layout. Button press times: stone 20, wood 30 — datagen
definition and export `fields` (`ticksToStayPressed`) agree; the port
reads the field at `data.rs:1015`. Full output:
`evidence/census_nibbles_26_1_1.log`.

**Observers (5 checks):** 2-tick pulse on/off, `updateNeighborsInFront`,
trigger via `updateShape` on the facing side only when unpowered
(`shapes.rs:668`), `startSignal` (schedule 2 ticks if none scheduled),
`onPlace` and `affectNeighborsAfterRemoval` arms (`hooks.rs:200,368` —
onPlace: powered && !hasScheduledTick → set powered=false flags 18 +
updateNeighborsInFront; removal: powered && hasScheduledTick →
updateNeighborsInFront with the powered=false state, no setBlock).
(`hooks.rs` diff touches only lines 539 and 706; observer arms
unaffected.)

**Pistons (11 checks):** quasi-connectivity (`getNeighborSignal` —
direct neighbours except the push direction, signal at self/down, and
the above-block scan except down); `checkIfExtend` events 0/1/2 with
the moving-piston progress rule (progress < 0.5 or same-tick or
handling-tick); `is_pushable` reaction-switch structure (BLOCK→false,
DESTROY→allow_destroyable, PUSH_ONLY→direction==connection — matches
vanilla's switch map; the values never arrive, B2-G02); resolver
structure (`resolve` / `addBlockLine` / `reorderListAtCollision` /
branching blocks) and the 12-block limit (`MAX_PUSH_DEPTH = 12`, jar
constant; `bipush 12` three times in `addBlockLine`); `moveBlocks`
structure and flags (276 head removal / 18 destroy / 324 moving blocks
/ 82 head placement, destroy-then-push order, shape updates at limit
512 — `UPDATE_LIMIT`, `mod.rs:169`); destroy notification (G12 —
withdrawn, port matches vanilla 26.1.1); `triggerEvent` (server-side
signal re-check, extend/retract, sticky-pull guard: event 1, non-air,
pushable, NORMAL reaction or piston); moving-piston tick/finalTick
(progress +0.5/tick clamped to 1.0, deathTicks client wait < 5,
waterlogged strip, air fallback flags 340 + updateOrDestroy flags 3,
normal path flags 67 + neighborChanged); entity piston movement limit
(±0.51 blocks/tick/axis, first-nonzero-axis only, 1e-5 threshold,
per-tick reset, lengthSqr 1e-7 early-out —
`Entity.limitPistonMovement` +
`applyPistonMovementRestriction` match `piston_movement.rs`);
slime/honey stick rules (`canStickToEachOther`: honey/slime mutually
exclusive, either sticky → stick); piston head survival (fitting base
or same-facing moving piston). (`pistons.rs` byte-identical to 26.1's;
jar classes byte-identical to 26.1's.)

## Pilot leftover diagnosis (summary)

| Test (`level/tests.rs`) | Verdict | Root cause |
|---|---|---|
| `redstone_lever_wire_and_lamp` (:1253) | **Expected FAIL** — run owed (B2-H02) | Wire class never dispatches (B2-G01); first assert (power "15") fails with power "0" |
| `levers_power_wire_through_interaction` (:1734) | **Expected FAIL** — run owed (B2-H02) | Same (B2-G01) |
| `pistons_push_and_sticky_pistons_pull` (:1781) | Expected pass — run owed (B2-H02) | Dirt/redstone_block are NORMAL reaction → PushPull either way; push/pull logic verified |
| `pistons_pop_fragile_blocks_and_refuse_obsidian` (:2120) | **Expected FAIL** — run owed (B2-H02) | Torch (DESTROY) pushed, no DropResources event (B2-G02); obsidian pushed instead of refused (B2-G03) |

No pilot failure remains unexplained for 26.1.1 — but unlike 26.2, three
of the four are expected failures, not expected passes. The 26.1.1 tree
is the 1.21.7 shape here, not the 26.2 shape: the template fixes never
landed in this tree. The CI green run for 26.1.1 does not contradict
this: the level tests skip silently without prepared data (`world!()`
returns early when `fixture()` is `None`), and the crate-tests workflow
has no prepare step, so a clean runner never executes them.

## §6 Systemic findings

### The 26.1.1 tree is stale against the 26.1 tree in `level/`

The 26.1→26.1.1 delta is tiny (3 changed classes, none in scope), so
the 26.1 tree's 26.1-form fixes should all be present in the 26.1.1
tree. They are not. The full `level/` diff between the trees in this
worktree shows the 26.1.1 tree carrying 26.2/26.3-form behaviour the
26.1 tree deliberately removed, with the 26.1 tree's comments
documenting each delta. B2-G07 (diode guard) is the in-scope instance.
The out-of-scope instances below are flagged for their owning batches
— each is a candidate defect of the same stale-carry class, verified
here only to the extent of the tree diff plus the jar-identity fact
(the 26.1→26.1.1 delta touches none of the classes involved, so the
26.1 tree's vanilla form is the 26.1.1 vanilla form):

- **`interact.rs` bed use** (26.1.1 :367-384): the 26.1.1 tree has the
  26.3 pair check (`AbstractBedBlock` form); the 26.1 tree has the 26.1
  form (foot click redirects to head, head click needs no foot check).
- **`interact.rs` composter** (26.1.1 :560-633): `composter_add_layer`
  (26.3 form) instead of 26.1's `composter_add_item` (level-0 always
  adds, above level 0 rolls under the item's chance).
- **`hooks.rs` ice melt** (26.1.1 :706): fires the `BLOCK_DESTROY` game
  event; the 26.1 tree notes the event is a 26.3 addition and 26.1's
  melt fires none.
- **`rails.rs` tripwire re-check** (26.1.1 :595-596): `tripwire_check_pressed`
  adds `else if was { schedule_tick(pos, block, 1) }` — a 1-tick
  pressed→unpressed re-check. Vanilla 26.1.1's `TripWireHookBlock`
  (byte-identical to 26.1's) has exactly ONE `scheduleTick` in the
  whole class — the 10-tick attached-circuit re-check inside
  `calculateState` (offset 281) — and no 1-tick unpress re-check
  (`evidence/tripwirehook.txt`). The 26.1 tree's comment states the
  unpress re-check is 26.2-only. (Note: the function is tripwire, not
  detector rail — the 26.1 comment sits in `tripwire_check_pressed`.)
- **`sculk.rs`** (26.1.1 :620): reads the `minecraft:sculk_growth_inhibitors`
  tag; the 26.1 tree notes 26.1 has no such tag (26.3 added it) and
  hardcodes sensor + shrieker.
- **`mod.rs` / `data.rs` / `hooks.rs` motion tags**: the 26.1.1 tree
  removed the per-state `blocks_motion()` rule (`mod.rs:975-985` in
  26.1) and reads `blocks_motion` / `blocks_fluid_flow` tags instead
  (`data.rs:426-427`, `hooks.rs:539`), and dropped the `cobweb` named
  entry (`data.rs:313` in 26.1). 26.1 has no `blocks_motion` tag
  family (26.3 added it) — whether 26.1.1's data carries these tags is
  Batch 3's question (cf. the 26.1 run's B3-G22 startup blocker).
- **`update.rs` `ensure_block_entity`** (26.1.1 :385-388): re-checks the
  block is still the new block after onPlace (26.2 form); the 26.1
  tree creates the block entity unconditionally.
- **`timeline.rs` scan order**: `between_closed` (26.1: x innermost,
  then y, then z) replaced by `box_scan_order` (26.3 `findBlocksIn`
  order: sections by x, z, y; blocks by y, z, x), used by
  `eyeblossom_tick` (:92) — the scan order assigns the per-neighbour
  random draws in a different order. Vanilla 26.1.1 has no blockscan
  package at all (0 jar entries).

`PORT-NOTES-26.1.1.md` states the carry-forward took "the completed
26.1 code" as base; the observed `level/` state contradicts that for
the behaviour fixes above. The coordinator may want to check whether
the carry-forward actually covered `level/` — if it did not, every
batch should expect this stale-carry class in its scope, and the fix
for each is the 26.1 tree's form, already written and verified.

### The vanilla-check rule mattered in both directions again

Assuming the 1.21.7 form would have filed two false defects (G11, G12
— vanilla changed); assuming the 26.3 form (PushReaction renames,
obsidian IMMOVEABLE, `datapackWorldRegistries`, blockscan/
ClockInstance/Continuation) would have missed the real ones. The new
B2-G07 is a third direction: assuming the 26.1 tree's form carries to
the 26.1.1 tree would have missed a regression the tree diff exposes.

## Method notes / limitations

- **No driver runs, no cargo tests.** Blocked by the batch constraints
  (no builds on the shared VM) and independently by the broken oracles
  (B2-O01/B2-O02/B2-O03) and the B2-G05 driver constant. The prepared
  26.1.1 data exists, so the data precondition is met — the oracle
  repairs + driver runs are the follow-up work (B2-H01/B2-H02).
- **Vanilla 26.1.1 vs 26.2 deltas found in scope:** `DiodeBlock.
  neighborChanged` has no same-block guard in 26.1.1 (26.2 returns
  early when `getBlockState(pos).is(this)` fails) — the 26.1.1 tree
  carries the 26.2 form, wrongly (B2-G07); `SignalGetter` lives at
  `net.minecraft.world.level.SignalGetter` in 26.1.1; `MinecraftServer`'s
  ctor is 10-arg (no `NotificationManager`) and `ServerChunkCache`'s is
  11-arg; `WorldLoader$DataLoadContext` has no `datapackWorldRegistries()`
  (same as 26.2); the blockscan package, `Continuation` and
  `ClockInstance` do not exist in 26.1.1 at all. Each verified against
  the 26.1.1 jar, not assumed from 26.1, 26.2 or 26.3.
- **26.1.1 tree vs 26.2 tree:** the in-scope Rust code is otherwise
  identical to the verified 26.2 tree except where noted — `redstone.rs`
  differs from 26.1 only by the diode guard (which makes it match 26.2
  there, wrongly for 26.1.1); `pistons.rs` differs from 26.2 only by the
  missing obsidian block; `hooks.rs`/`interact.rs`/`mod.rs` differ from
  26.1 only in version comments plus the out-of-scope items in §6;
  `redstone/` and `tests/` are byte-identical to 26.1's. The behaviour
  deltas are concentrated in the table layer: `classes.rs` has no alias
  mechanism and no `RedStoneWireBlock` spelling, and `data.rs`'s
  push-reaction loader matches only the 26.3 enum names. The 26.2 tree
  is the natural reference for the G7/G8/G9 repairs.
- **Not re-verified here:** the experimental redstone evaluator
  (`redstone/experimental.rs` — behind the `redstone_experiments` flag,
  off in the test level; vanilla 26.1.1 has
  `ExperimentalRedstoneWireEvaluator` and `ExperimentalRedstoneUtils`,
  same as 26.2); dispense/dropper *behaviours* beyond the redstone
  trigger; sculk sensors (own family); command-block powering beyond
  the `neighborChanged` arm; the full per-state nibble diff (spot
  checks only — 8 block families verified, not all 29,873 states).
- **Adjacent observations (not Batch 2 findings):**
  - `dirt_path` (`DirtPathBlock`) and `amethyst_block` (`AmethystBlock`)
    fall back to plain `Block` in the 26.1.1 tree — the same bug class
    as B2-G01, for other batches (paths / amethyst). Full census: 3
    blocks fall back to plain Block despite a non-Block chain; 87 blocks
    have an unrecognised most-derived class but a recognised ancestor
    (leaves, glass, furnaces, copper variants — behaviour largely
    carried by the ancestor).
  - `VanillaDispenserOracle` fails for 26.1.1 (15 errors, EntityTypes
    G4) — see Oracle defects.
  - The 35723 hardcode appears in many other drivers outside Batch 2
    scope (`test_block_properties_parity.py`,
    `test_collision_parity.py`, …) — the same one-line fix (read the
    count from the facts) applies programme-wide; flagged for the
    coordinator.
  - The stale-carry instances in §6 (bed use, composter, ice melt,
    tripwire re-check, sculk tag, motion tags, `ensure_block_entity`,
    eyeblossom scan order) belong to their owning batches; the diode
    guard is the only one minted here.
- **Disk / artifacts:** javap dumps, the census script and its log are
  in `evidence/` under this batch's directory
  (`~/workspace/mcir-work/sweep-26.1.1-batches/b02/evidence/`). No files
  were written under `/tmp` or under the worktree.

## §7 Evidence index

Worktree `~/workspace/mcir-work/sweep-26.1.1`; batch dir
`~/workspace/mcir-work/sweep-26.1.1-batches/b02/`; jars
`~/workspace/mcir-work/26.1.x/versions/26.1.1/jars/`.

| Evidence | Location |
|---|---|
| `RedStoneWireBlock` only; no `RedstoneWireBlock` | jar listing, `server-game.jar` (unzip -l grep) |
| `PushReaction` enum NORMAL/DESTROY/BLOCK/IGNORE/PUSH_ONLY | `javap` on `server-game.jar` (transcript in §4 B2-G02) |
| `isPushable` obsidian refusal (offsets 44–85) + Y bounds (0–34, 106–125) | `b02/evidence/pistonbase.txt` (lines 582–712) |
| `moveBlocks` destroy `bipush 18` (offset 302) | `b02/evidence/pistonbase.txt` (line 830) |
| `ComparatorBlock.getInputSignal` `Math.max` (offset 150) | `b02/evidence/comparator.txt` (line 227) |
| `DiodeBlock.neighborChanged` no guard (offsets 0–94) | `b02/evidence/diode.txt` (lines 140–210) |
| `DetectorRailBlock.checkPressed` scheduleTick gated on minecart present | `b02/evidence/detectorrail.txt` (lines 116–240) |
| `TripWireHookBlock` single `scheduleTick` (10-tick, offset 281) | `b02/evidence/tripwirehook.txt` (line 323) |
| `WorldLoader$DataLoadContext` components (no `datapackWorldRegistries`) | `b02/evidence/dataloadcontext.txt` |
| `findBlocksIn`/`getNextEntityId` absent from `Level`/`ServerLevel` | `b02/evidence/level.txt`, `serverlevel.txt` (0 matches) |
| blockscan/Continuation/ClockInstance 0 jar entries; `CubeVoxelShape` ctor protected; `MinecraftServer` 10-arg; `ServerChunkCache` 11-arg; no spam thresholds | `javap`/jar-listing transcripts (§4 B2-O01–O03) |
| 41 in-scope classes byte-identical 26.1 ↔ 26.1.1 | sha1 comparison transcript (§2) |
| `classes.rs:224` wire spelling; `data.rs:886-890` loader; `data.rs:928` chain fallback; `data.rs:879-880` nibble columns; `data.rs:1015` press times; named `obsidian` :267, `respawn_anchor` :369 | `versions/26.1.1/crates/horizon-world/src/level/` |
| `pistons.rs:337` `max_y() - 1`; no obsidian check (grep empty) | `versions/26.1.1/crates/horizon-world/src/level/pistons.rs` |
| Diode guard `redstone.rs:673-675`; comparator max `redstone.rs:541-546` | `versions/26.1.1/crates/horizon-world/src/level/redstone.rs` |
| Wire arms `hooks.rs:100,211,380`; `survival.rs:43` prepare half | `versions/26.1.1/crates/horizon-world/src/level/hooks.rs`, `versions/26.1.1/crates/horizon-prepare/src/survival.rs` |
| Push census (22,869/6,849/91/64), class census (3 fallbacks, 87 ancestor-only), nibbles, press times | `b02/evidence/census_nibbles_26_1_1.py` + `census_nibbles_26_1_1.log` |
| Prepared data: redstone_wire chain `["RedStoneWireBlock", "Block"]`; 29,873 states | `versions/26.1.1/.local/horizon-client/26.1.1/{block-behaviours,gameplay-facts}.json` |
| Driver lines: shape `:63`, cobweb `:16`, `redstone_diff.py:453`, `piston_step_e2e.py:31` | `versions/26.1.1/tools/` (byte-identical to 26.1's) |
| Level tests `:1253, :1734, :1781, :2120` | `versions/26.1.1/crates/horizon-world/src/level/tests.rs` (byte-identical to 26.1's) |
| Oracle table (32/4/1/cascade; O3/O4/O5 pass) | `docs/sweep/26.1.1/batch-0-oracles.json` |
| `level/` tree diff 26.1 ↔ 26.1.1 (diode guard + §6 items) | `diff -r versions/26.1/crates/horizon-world/src/level versions/26.1.1/crates/horizon-world/src/level` |
| Oracle sources byte-identical 26.1 ↔ 26.1.1 (BlockUpdate/Catalog/FaceSupport) | `diff -q` on `versions/26.1{,.1}/mods/horizon/src/test/java/dev/horizonlab/parity/` |

## §8 Verdict

Batch 2 for 26.1.1 is complete: **52 behaviour checks — 45 pass, 7
gap**; 17 items (7 behaviour gaps, 6 oracle defects, 2 needs-harness,
2 closed). All 14 26.1 items were re-checked and decided (13 reproduce,
1 — the diode machinery check — is a NEW gap in 26.1.1, B2-G07); all 13
1.21.7 items were re-checked and decided (8 reproduce, 2 withdrawn,
3 fixed). No item is left UNVERIFIED.

The 26.1.1 redstone/piston picture is the 26.1 picture plus one
regression: the wire is dead (G01), every state loads PushPull (G02),
obsidian pushes (G03), the ceiling bound is off by one (G04), the
shape driver cannot run (G05), the 0.01 height divergence stands
(G06), and diodes now wrongly skip `neighborChanged` when already
replaced (G07). The three 1.21.7 persistence/piston oracles stay fixed;
the block-update, catalog and face-support oracles stay broken with
identical error counts. The corpus and level-test runs that would
demonstrate all of this behaviourally are blocked on the oracle
repairs (B2-H01/B2-H02).

The finding the coordinator should weigh most is §6: the 26.1.1 tree
did not inherit the 26.1 tree's `level/` behaviour fixes, and the
diode guard is unlikely to be the only stale-carry defect in the
programme — every batch should diff its scope against the 26.1 tree
the way this batch did.
