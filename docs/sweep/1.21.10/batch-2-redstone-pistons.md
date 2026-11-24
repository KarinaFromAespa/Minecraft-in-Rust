# Batch 2 — Redstone + pistons (1.21.10 parity sweep)

**Version:** 1.21.10 · **Date:** 2026-11-25 · **Branch:** `parity-sweep-1.21.10`
(workdir `~/workspace/mcir-work/sweep-1.21.10`) ·
**Scope:** `docs/PARITY-SWEEP.md` §4 Batch 2 — redstone ticking, pistons,
observers, comparators; 10 drivers + circuit probes; pilot leftovers
(4 level::tests).

## §0 Execution status

Checks + triage only. No file under `versions/1.21.10/` was modified,
nothing was committed, no git commands ran in the worktree, no cargo
builds ran. Everything below is static analysis: `javap -c` on the named
1.21.10 server-game jar
(`~/workspace/mcir-work/e6-deltas/versions/1.21.10/jars/server-game-named.jar`,
sha1 `ba1fc89ead95b8ebdb8727f84e2bd487533c8b7f`, verified, 6,964
classes, Mojang-mapped; the bundler `server.jar` was never javap'd;
`client-named.jar` sha1 `715f58165acfaf8b61ab921064f08c202b41bd0c`,
verified), source comparison against `versions/1.21.10/crates/`, the
1.21.10 datagen reports
(`~/workspace/mcir-work/e6-deltas/versions/1.21.10/datagen/reports/`,
1,166 blocks, 29,671 states), a full-jar class comparison against the
1.21.7 server-game jar (sha1
`3db9f7c5532107bebd265623abed0383d16b5ca3`), the 1.21.7 prepared export
as the validated reference (`~/workspace/mcir-work/mob-spawn/resources/`,
see §2.3 — no prepared 1.21.10 data exists in the worktree), and Batch
0's 1.21.10 oracle measurement
(`versions/1.21.10/.local/measure-oracles/results.json`, 391 files, all
three classpaths — see the note on the landed table below), cross-checked
against my own targeted javac compiles.

**Headline:** the wire is FIXED — the 1.21.10 tree carries the
1.21.11-template class-alias mechanism, so redstone wire dispatches for
the first time in the E6 lineage (1.21.7 G7 closes, and with it the two
wire level::tests are now expected to pass). Everything else from 1.21.7
survives: every state still loads PushPull (G8 → B2-G02), obsidian still
pushes (G9 → B2-G03), the ceiling bound is still off by one (G10 →
B2-G04), the shape driver still cannot run (G13 → B2-G05), the 0.01
height divergence still stands (G14 → B2-G06), and all five 1.21.7
oracles are still broken — the two big ones much worse than the 26.x
runs (BlockUpdate 63 errors, Catalog 36), because 1.21.10 lacks the
26.x APIs the oracles were written against. One NEW defect: the wire's
prepare-side survival rule never binds (B2-G08) — the runtime half of
G7 was fixed, the prepare half was not. And two 1.21.7 items are
**NEVER EXISTED, not WITHDRAWN**: 1.21.7's own bytecode contradicts
both the G11 comparator premise and the G12 destroy-notify premise
(§4 B2-W01/B2-W02) — the 26.x runs' "vanilla changed" framing was
wrong, and the 1.21.7 ledger needs correcting.

**One delta vs 1.21.7 (precondition, not a defect):** 1.21.10 has NO
prepared horizon-client data in the worktree
(`versions/1.21.10/.local/` holds `minecraft/` + `measure-oracles/`
only). The 1.21.7 run could point at its prepared export; for 1.21.10
the driver runs (B2-H01) and the level::tests (B2-H02) need a prepare
run first, on top of the oracle repairs. The census/nibble/press-time
checks in this report were still run as real checks — the census
against 1.21.10's own jar (validated on 1.21.7, §2.3), the
nibbles/press-time against the exporter + loader + datagen chain (§2.4).

**Note on the Batch 0 table:** the landed
`docs/sweep/1.21.10/batch-0-oracles.json` had NOT appeared at handoff
(checked 03:50 NPT Nov 25; `docs/sweep/1.21.10/` holds only
extract-inventory.py, inventory-1.21.10.json, inventory.md, merge.py).
Triage here uses Batch 0's raw 1.21.10 measurement directly —
`versions/1.21.10/.local/measure-oracles/results.json` (391 files, all
three classpaths; Java 21, fixpoint big-compile per classpath) plus my
own targeted javac compiles of all 9 in-scope oracles on both the
server and client classpaths, which agree with results.json exactly
(error counts 63 / 36 / 1 / 2 / 1 / 1 / cascade / pass / pass-on-client
— §4 B2-O01–B2-O07). Every failing oracle's causes were additionally
javap-verified fresh against the 1.21.10 jar. If the landed table
disagrees with results.json on any in-scope oracle, the landed table
wins and this batch's O-items need a re-read.

## §1 Scope

Redstone wire power/updates, comparators, repeaters, observers, pistons
(push/pull/destroy, `is_pushable`, shape driver), block-update ordering,
redstone persistence. Method: `javap -c` on the named 1.21.10
server-game jar + source comparison against `versions/1.21.10/crates/` +
tag/data reads. No cargo builds, no driver runs. The 4 level::tests and
the circuit-equivalence / signal-nibble checks that need prepared data
or a test run are needs-harness items (B2-Hxx), not faked passes —
except that the nibble/press-time checks were run here as real checks
against the exporter + loader + datagen chain (§2.4), and the census
was run as a real check against 1.21.10's own jar (§2.3).

Out of scope for this batch (noted, not verified): the 1.21.10 delta's
behaviour items outside redstone/pistons; the B3-G22 motion-tag startup
blocker (one instance flagged in §6 — `hooks.rs:549` reads
`b.tags.blocks_motion`, a 26.3 motion-tag read); the amethyst-block
class-table fallback (§6); `VanillaLevelShapeOracle` (broken, 2
errors — adjacent failing oracle, recorded in §4, not minted).

## §2 Method

Five verification layers, all fresh against 1.21.10 evidence:

1. **Jar identity proof.** A full sha1 comparison of ALL classes
   between the 1.21.7 and 1.21.10 server-game jars: 73 classes only in
   1.21.7, 309 only in 1.21.10, 1,153 changed. Sixteen in-scope classes
   are byte-identical, including `RedstoneWireEvaluator`,
   `DefaultRedstoneWireEvaluator`, `DiodeBlock`, `RepeaterBlock`,
   `ObserverBlock`, `RedstoneTorchBlock`, `TargetBlock`,
   `PistonStructureResolver`, `PushReaction`, `SignalGetter`,
   `NeighborUpdater` (at
   `net.minecraft.world.level.redstone.NeighborUpdater`),
   `PoweredRailBlock`, `TripWireHookBlock`, `DropperBlock`,
   `PressurePlateBlock`, `WeightedPressurePlateBlock`. Twenty-four
   in-scope classes changed (`RedStoneWireBlock`, `ComparatorBlock`,
   `PistonBaseBlock`, `LeverBlock`, `ButtonBlock`, `RedstoneLampBlock`,
   `CopperBulbBlock`, `DaylightDetectorBlock`, `DispenserBlock`,
   `HopperBlock`, `DetectorRailBlock`, `TripWireBlock`,
   `SculkSensorBlock`, `NoteBlock`, `CommandBlock`, `JukeboxBlock`,
   `LecternBlock`, `CalibratedSculkSensorBlock`,
   `BasePressurePlateBlock`, `MovingPistonBlock`,
   `PistonMovingBlockEntity`, `PistonHeadBlock`,
   `CollectingNeighborUpdater`, `ExperimentalRedstoneWireEvaluator`) —
   and a normalized `javap -c` diff of all 24 (556 lines) shows ALL
   differences are API-shape/debug-only, with NO Batch 2 behaviour
   change: `Level.isClientSide` field→method, `entityInside` gained a
   `boolean` param, `getAnalogOutputSignal` gained a `Direction` param
   (2-arg→3-arg), JukeboxBlock/LecternBlock `CustomData.EMPTY` +
   `getOrDefault` → `TypedEntityData` + `get` + null check (same
   behaviour), `CollectingNeighborUpdater` gained a `debugListener` +
   log-format change, `ExperimentalRedstoneWireEvaluator` debug-sync
   plumbing change. ObserverBlock/RepeaterBlock/RedstoneTorchBlock had
   zero hunks. So 1.21.7 pass verdicts carry for these classes by
   (jar-method identity modulo API-shape/debug + port-file identity).
2. **Tree identity proof.** `crates/horizon-world/src/level/` between
   the 1.21.7 and 1.21.10 trees in the worktree: `pistons.rs`,
   `redstone.rs`, `update.rs`, `signals.rs`, the `redstone/` subdir and
   the `tests/` subdir are BYTE-IDENTICAL. Differing in-scope files
   (`classes.rs`, `data.rs`, `hooks.rs` — 20 diff lines, `shapes.rs` —
   73, `interact.rs` — 10, `place.rs` — 10, `survive.rs` — 106,
   `mod.rs`, `tick.rs`, `tests.rs`) were all diff-reviewed; every
   difference is out of Batch 2 scope (bubble columns, beds, dragon
   egg, bamboo/dripleaf/chorus tag renames, dripstone, farmland, crops,
   lily pads, wither rose, magma/soul-sand bubble checks, leaves
   distance). The wire arms are untouched. 1.21.10 vs 1.21.11 `level/`:
   16 files differ (bonemeal, classes, data, fluids, growth_tests,
   interact, place, rails, removal, sculk, shapes, speleothem, survive,
   tests, tick, timeline); `hooks.rs`, `mod.rs`, `pistons.rs`,
   `redstone.rs`, `update.rs`, `signals.rs` are identical to the
   template. All 9 in-scope oracle sources are byte-identical between
   the 1.21.7 and 1.21.10 trees.
3. **Census (new method, validated).** A script
   (`b02/evidence/extract_blocks.py`) extracts every block's
   most-derived class + push reaction from `javap -c -p -v` dumps of
   `Blocks` (`<clinit>` segments split by putstatic; factory via
   invokedynamic bootstrap → constructor, or register* helper → lambda
   → `new XBlock`, or 2-arg register → Block; reaction = last
   PushReaction in the segment/helper chain, or inherited via
   ofLegacyCopy/ofFullCopy/wallVariant/registerStair from the source
   block; `WeatheringCopperBlocks.create` expands to 8 blocks sharing
   class+reaction; superclass chains via a Python class-file parser).
   Validated on 1.21.7: **0 reaction mismatches, 0 class mismatches
   across all 1,105 blocks** against the prepared export
   (`b02/evidence/validate_1_21_7.py`). Applied to 1.21.10: all 1,166
   blocks, 0 missing — blocks by reaction NORMAL 784 / DESTROY 353 /
   BLOCK 13 / PUSH_ONLY 16; states by reaction NORMAL 22,669 /
   DESTROY 6,847 / BLOCK 91 / PUSH_ONLY 64 (total 29,671).
   Spot checks: redstone_wire = RedStoneWireBlock/DESTROY; obsidian =
   Block/NORMAL; piston = PistonBaseBlock/BLOCK; copper_bars =
   IronBarsBlock/NORMAL; copper_lantern = LanternBlock/DESTROY;
   campfire = CampfireBlock/NORMAL.
4. **Nibbles/press-time chain.** The loader columns were verified
   against the exporter, not the data: the behaviour exporter
   (`VanillaBlockBehaviourExporter.java:198-220`) packs weak/strong
   signals (4 bits per direction, from `state.getSignal` /
   `state.getDirectSignal` over an empty level) into row columns 5/6,
   and the loader reads `weak: int(5), strong: int(6)`
   (`data.rs:876-877`) — columns agree. Press times: the 1.21.10
   datagen definitions carry `ticks_to_stay_pressed` (stone 20, wood
   30); the jar's `ButtonBlock` has `private final int
   ticksToStayPressed`; the exporter dumps fields reflectively (so the
   key exists with the right values — confirmed in the 1.21.7 export:
   stone_button 20, oak_button 30); the port reads
   `field("ticksToStayPressed")` (`data.rs:1012`). The full per-state
   nibble spot-check still needs prepared 1.21.10 data — folded into
   B2-H01.
5. **Oracle measurement.** Batch 0's raw results.json (391 files) +
   my own targeted javac compiles of the 9 in-scope oracles
   (`--release 21`, server classpath = server-game-named.jar + 44
   server library jars + jsr305; client classpath = client-named.jar
   + client libraries + jsr305; logs in
   `b02/evidence/javac/`). The two agree exactly on every in-scope
   oracle. Every failure cause was javap-verified against the 1.21.10
   jar (§4 B2-O01–B2-O07).

## §3 Checks table

- **Behaviour checks: 53 — 46 pass, 7 gap** (across 7 defect items
  B2-G02–B2-G08: 2 major, 5 minor).
- **Behaviour/non-behaviour split: 53 behaviour + 0 non-behaviour.**
  Batch 2 is not one of the three as-tallied exception batches; the
  tally counts behaviour checks only.
- **Oracle/driver defects: 9** (B2-O01–B2-O09: 3 major, 6 minor — not
  counted in the check tally).
- **Needs-harness: 2** (B2-H01–B2-H02).
- **Fixed: 1** (B2-G01 — the wire; 1.21.7 G7 closes).
- **Closed (never existed): 2** (B2-W01, B2-W02 — 1.21.7's G11 and G12
  never existed; the premises are contradicted by 1.21.7's own
  bytecode).
- **1.21.7 majors: one FIXED, two REPRODUCE in 1.21.10** (G7 → B2-G01
  fixed; G8, G9 → B2-G02, B2-G03 reproduce). The 1.21.10 tree carries
  the 1.21.11-template class-alias mechanism (which fixes G7) but NOT
  the push-reaction loader fix, the obsidian hardcode, or the Y-bound
  fix — so the template lineage fixed the wire and left the rest.
- **NEW vs the 26.1.2 template run: one check, one defect.** The wire
  section has 10 checks here (not 9): the 10th is the prepare-side
  wire-survival binding, a check the 26.1.2 table did not carry
  (26.1.2's survival table uses the correct 26.1.2 class name, so it
  would have passed there). It fails here → B2-G08. The tally is
  therefore 53 / 46 / 7 against 26.1.2's 52 / 45 / 7: +1 check, +1
  pass, same gap count, different gap composition (G01 out, G08 in).

Per-section counts sum to the tally: wire 10 (9 pass + G08), machinery
4 (3 pass + G07), consumers 16, nibbles/data 3 (2 pass + G02),
observers 5, pistons 13 (11 pass + G03 + G04), driver constants 2
(G05 + G06) = 53; 46 + 7 = 53.

## §4 Defects

### B2-G01 — Redstone wire FIXED: class-name alias lands (was MAJOR; 1.21.7 G7 FIXED; 26.1.2 B2-G01 FIXED)

Vanilla 1.21.10 names the class `RedStoneWireBlock` (capital S; jar
listing — `net/minecraft/world/level/block/RedStoneWireBlock.class`
exists, `RedstoneWireBlock` does not). The 1.21.7 tree knew only
`RedstoneWireBlock`, so the wire never dispatched. The 1.21.10 tree
carries the alias mechanism: `crates/horizon-world/src/level/classes.rs:228`
reads `RedstoneWire = "RedstoneWireBlock" | "RedStoneWireBlock"`, via
the macro at :8–23 (`from_java` matches `$java $(| $alt)?`), with a
comment (:9–12) saying aliases are for 1.21.10 renames. Aliases also
exist for Farmland/FarmBlock (:155), Snowy/SnowyDirtBlock (:170),
SpreadingSnowy (:171), NetherFungus/FungusBlock (:204),
NetherRoots/RootsBlock (:206), Path/DirtPathBlock (:212) — the 1.21.7
fallbacks FarmBlock, DirtPathBlock, SnowyDirtBlock are all fixed in
1.21.10. The census confirms the chain:
`redstone_wire → ['RedStoneWireBlock', 'Block']` → recognised head
`RedStoneWireBlock`. All wire arms dispatch on `Class::RedstoneWire`:
`hooks.rs:100,221,390`; `shapes.rs:715`; `interact.rs:263` (+
`wire_use` at :492); `place.rs:1089`; `survive.rs:300`. The wire power
math (`redstone.rs:414-433` — max of neighbours, minus 1), connection
state (`redstone.rs:293-360`), and connecting-side logic
(`redstone.rs:236`) are all live now. **The runtime half of G7 is
FIXED.** The prepare half is not — see B2-G08.

### B2-G02 — Push-reaction loader mapping matches no 1.21.10 value: every state loads as PushPull (MAJOR; 1.21.7 G8 reproduces; 26.1.2 B2-G02 reproduces)

Vanilla 1.21.10's `PushReaction` enum is NORMAL, DESTROY, BLOCK,
IGNORE, PUSH_ONLY (javap-verified; `b02/evidence/pushreaction.txt`).
The gameplay-facts exporter writes the vanilla name verbatim
(`VanillaGameplayFactsExporter.java:50`: `"push"` =
`state.getPistonPushReaction().name()`). The census (§2.3) counts the
1.21.10 facts: NORMAL 22,669 / DESTROY 6,847 / BLOCK 91 / PUSH_ONLY 64
states. But the loader (`data.rs:883-886`) matches ONLY the 26.3 names
— `"PUSH" => Push`, `"POPPED" => Popped`, `"IMMOVEABLE" =>
Immoveable`, `"IGNORE_ENTITY" => IgnoreEntity` — and defaults
everything else to PushPull. So **all 29,671 states load as PushPull**:
DESTROY blocks (torches, flowers, wire — 6,847 states) are pushed
instead of popped, BLOCK blocks (pistons, portals, anvils — 91 states)
are pushed instead of blocking, and PUSH_ONLY glazed terracotta (64
states) is pushed from any side. The reaction switch itself is correct
(`pistons.rs:368-372` matches vanilla's switch map: Immoveable→false,
Popped→allow_destroyable, Push→direction==connection) — it just never
sees anything but PushPull.

### B2-G03 — `is_pushable` lacks vanilla's hardcoded obsidian-family refusal (MAJOR; 1.21.7 G9 reproduces; 26.1.2 B2-G03 reproduces)

Vanilla 1.21.10's `PistonBaseBlock.isPushable` refuses four blocks by
hardcoded name at bytecode offsets 44–85: OBSIDIAN, CRYING_OBSIDIAN,
RESPAWN_ANCHOR, REINFORCED_DEEPSLATE
(`b02/evidence/pistonbase.txt`, lines 581–697). The port's
`is_pushable` (`pistons.rs:328-372`) has NO obsidian check at all — a
piston pushes obsidian like stone. The `named` table has `obsidian`
(:270) and `respawn_anchor` (:372) but NO `crying_obsidian` or
`reinforced_deepslate`, so even a name-based fix needs two new named
entries. Note the census: obsidian's reaction is NORMAL (Block/NORMAL),
so the loader bug (B2-G02) is not the cause — this is a separate
missing hardcode, exactly as in 1.21.7.

### B2-G04 — `is_pushable` Y-bound off by one at the ceiling (MINOR; 1.21.7 G10 reproduces; 26.1.2 B2-G04 reproduces)

Vanilla 1.21.10 refuses a block at `y < minY || y > maxY` (offsets
0–33) and refuses an UP-push only from `y == maxY` (offsets 106–125) —
a block AT y == 320 is pushable sideways/down, and can be pushed up
from 319. The port (`pistons.rs:337`) computes `let (min_y, max_y) =
(self.level.min_y(), self.level.max_y() - 1);` — with `max_y() == 320`
(`mod.rs:606-608`), the port's max is 319, so it refuses y == 320
outright and refuses an UP-push from 319. Off by one at the ceiling,
exactly as in 1.21.7.

### B2-G05 — Piston-shape driver hardcodes the 26.3 state count (MINOR, harness; 1.21.7 G13 reproduces; 26.1.2 B2-G05 reproduces)

`tools/test_piston_shape_parity.py:63` asserts
`len(facts['states']) != 35723` — the 26.3 state count. 1.21.10 has
29,671 states (datagen-verified), so the driver fails its own sanity
check before doing anything. The 35723 hardcode is programme-wide
(`test_block_landing_parity.py`,
`test_block_properties_parity.py`, `test_collision_parity.py`, and
more) — flagged for the coordinator in §6.

### B2-G06 — Documented 0.01 piston-push height divergence (MINOR, pre-existing; 1.21.7 G14 reproduces; 26.1.2 B2-G06 reproduces)

`tools/test_projectile_piston_cobweb_parity.py`'s docstring still
documents that "vanilla's second push here is 0.01 higher than
Horizon's", and the height is deliberately uncompared. The port's
±0.51 clamp (`piston_movement.rs:31-36`) is present and matches
vanilla's movement math; the 0.01 divergence in the second push is a
pre-existing documented gap, unchanged from 1.21.7.

### B2-G07 — `DiodeBlock.neighborChanged` carries the 26.2 same-block guard vanilla 1.21.10 does not have (MINOR; 26.1.2 B2-G07 reproduces)

Vanilla 1.21.10's `DiodeBlock.neighborChanged` has NO same-block guard
— it goes straight to `canSurvive` at offset 0, then
`checkTickOnNeighbor`, else dropResources + removeBlock +
updateNeighborsAt in all directions (offsets 0–94;
`b02/evidence/diode.txt`, lines 140–210). The port
(`redstone.rs:673-675`, in `diode_neighbor_changed`) returns early:
`if !self.b.is_block(self.state(pos), block) { return; }` — so a
repeater/comparator that has already been replaced skips the
neighbor-changed handling vanilla would still run. The port carries
the 26.2-form guard. Note: `redstone.rs` is byte-identical between
the 1.21.7 and 1.21.10 trees, so this defect exists in 1.21.7 too —
but the 1.21.7 sweep never minted it (§6).

### B2-G08 — NEW: wire's prepare-side survival rule never binds — `survival.rs` still uses the wrong class name (MINOR)

The runtime half of G7 was fixed (B2-G01) but the prepare half was
not. `crates/horizon-prepare/src/survival.rs:45` still reads
`("RedstoneWireBlock", "RedstoneWire")` — the 26.3 spelling. The
table binds the world generator's feature-placement survival rules:
`bind_definitions` (:102–117) looks up each block's owner class in
`CLASS_RULES` and silently binds NO rule on miss, so redstone_wire
gets no entry in `survival-rules.json` (the exporter's
`survival-classes.json` carries the vanilla class,
`RedStoneWireBlock`). The file's own pattern shows the author knew
this failure mode: the FarmBlock entry (:29–31) carries the comment
"1.21.10's class is `FarmBlock` (26.1 renamed it `FarmlandBlock`);
without the 1.21.10 name farmland binds no survival rule" — farmland,
lily pad and dry vegetation were all fixed this way, but the wire was
missed. The 1.21.7 tree has the same wrong line (the diff shows only
the farmland/lily-pad/vegetation lines differ), so this is the
un-minted prepare half of 1.21.7's G7: moot in 1.21.7, where the wire
was dead at runtime too, and now the surviving half of the defect.
Practical impact is small (no vanilla feature places redstone wire),
but the binding is objectively broken.

### B2-O01 — `VanillaBlockUpdateOracle` BROKEN, 63 errors (MAJOR; 1.21.7 O1 reproduces, bigger; 26.1.2 B2-O01 reproduces, bigger)

Batch 0 raw measurement: fails server (63 errors), client, and both.
My targeted compile agrees exactly (63 errors, same lines). Causes,
all javap-verified against the 1.21.10 jar:

- **Missing packages (5):** `net.minecraft.world.level.gamerules`
  (:16, :379 — GameRules lives at `net.minecraft.world.level.GameRules`
  in 1.21.10), `net.minecraft.world.clock` (:450 — no ClockInstance),
  `net.minecraft.world.level.blockscan` (:319, :322, :411, :427, :444),
  `net.minecraft.world.attribute` (:356, :360, :362, :395 — no
  environment attributes), `net.minecraft.server.permissions` (:522,
  :523). Jar listing: 0 entries for blockscan/clock/attribute/
  permissions/gizmos.
- **Missing symbols:** GameRules ×14 (class + variable),
  `statePredicate` ×3, `Util` ×3, `LevelBasedPermissionSet` ×3,
  `Continuation` ×3, `filterState`, `WORLD_REGISTRIES`, `WORLD`,
  `EnvironmentAttributeReader`, `timelines()`, `attributes()`.
- **Constructor mismatches:** ServerLevel (:136 — 1.21.10's ctor is
  11-arg: MinecraftServer, Executor, LevelStorageAccess,
  ServerLevelData, ResourceKey, LevelStem, boolean, long,
  List\<CustomSpawner\>, boolean, RandomSequences; the oracle calls
  the 26.3 10-arg form), MinecraftServer (:508 — 1.21.10's ctor is
  8-arg: Thread, LevelStorageAccess, PackRepository, WorldStem, Proxy,
  DataFixer, Services, LevelLoadListener; the oracle calls 11-arg),
  ServerChunkCache (:621 — 1.21.10's ctor is 11-arg; the oracle calls
  10-arg).
- **No-override ×~20** (:315 `canSpreadFireAround`, :357
  `getNextEntityId`, :393 `getOverworldClockTime`, :394
  `getMoonBrightness`, :530 `getCommandSpamThresholdSeconds`, :531
  `getChatSpamThresholdSeconds`, :532 `useNativeTransport` — 1.21.10's
  name is `isEpollEnabled`, :538 `getRandomSequence`, + others).
- **`Server is not abstract and does not override abstract method
  isEpollEnabled()`** (:505) — 1.21.10's MinecraftServer declares
  `public abstract boolean isEpollEnabled()`; 26.x renamed it
  `useNativeTransport`.
- `RandomSequences` no-arg ctor (:539) + `get` (:540); non-static
  `super` (:424).

The 1.21.7 run reported "≈25 breaks" (distinct causes); the 26.1.2 run
32 javac errors. The 63 here are javac error counts, not distinct
causes — but the cause list is genuinely longer than 26.1.2's: 1.21.10
lacks the whole 26.x API surface (gamerules package, clock, blockscan,
attributes, permissions) that 26.1.2 partially has.

### B2-O02 — `VanillaRedstoneCatalogOracle` BROKEN, 36 errors (MAJOR; 1.21.7 O2 reproduces, bigger; 26.1.2 B2-O02 reproduces, bigger)

Batch 0 raw measurement: fails all three classpaths (36 errors
server). My targeted compile agrees exactly. Causes, all
javap-verified:

- **Missing packages:** `net.minecraft.gizmos` (:28, :29 — Gizmos,
  GizmoCollector), `net.minecraft.server.permissions` (:42, :43),
  `net.minecraft.world.level.gamerules` (:73).
- **Missing symbols ×~17:** GameRules, WorldDataAndGenSettings,
  DifficultySettings, `datapackWorldRegistries()`, PermissionSet,
  LevelBasedPermissionSet, Zombie, ClientLevel, Util, etc.
  (`WorldLoader$DataLoadContext` in 1.21.10 has only
  `resources()`, `dataConfiguration()`, `datapackWorldgen()`,
  `datapackDimensions()` — no `datapackWorldRegistries()`.)
- **`isEpollEnabled()` not overridden** (:161).
- `PrimaryLevelData` ctor (:191).
- `tickServer` / `onServerExit` / `onServerCrash` cannot override
  (:225, :456, :457).
- No-override ×5 (:459, :460, :464–466 — spam thresholds etc.).

1.21.7 reported "≈15 breaks" (distinct causes); 26.1.2 had 4 javac
errors. Same story as O01: the 26.x API surface is missing in 1.21.10.

### B2-O03 — `VanillaRedstonePersistenceOracle` BROKEN, 1 error (MINOR; 1.21.7 O3 reproduces; FIXED in 26.1.2)

Batch 0: fails all three classpaths, class "constructor-mismatch",
first error "32: constructor ServerLevel in class ServerLevel cannot
be applied to given types;". My compile agrees (1 error). Same single
break as 1.21.7: the oracle calls the 26.3 ServerLevel ctor; 1.21.10's
is 11-arg (see B2-O01). It passed in 26.1.2 (whose ServerLevel ctor
matches the oracle) — the break is version-shape, not oracle rot.

### B2-O04 — `VanillaPistonPushOracle` BROKEN, 2 errors (MINOR; 1.21.7 O4 reproduces; FIXED in 26.1.2)

Batch 0: fails all three classpaths, class "missing-package", errors
"9: package net.minecraft.world.entity.monster.zombie does not exist"
+ "69: cannot find symbol". My compile agrees (2 errors). The oracle
imports `net.minecraft.world.entity.monster.zombie.Zombie` (the 26.x
package); 1.21.10 has `net.minecraft.world.entity.monster.Zombie`
(jar-verified — `Zombie.class` exists there, no `monster/zombie/`
package), exactly like 1.21.7. It passed in 26.1.2 (which has the
`zombie` subpackage).

### B2-O05 — `VanillaPistonMovementOracle` BROKEN, 1 error (MINOR; 1.21.7 O5 reproduces; FIXED in 26.1.2)

Batch 0: fails all three classpaths, class "constructor-mismatch",
first error "16: constructor ServerLevel in class ServerLevel cannot
be applied to given types;". My compile agrees (1 error). Same single
break as B2-O03 / 1.21.7 O5.

### B2-O06 — `VanillaFaceSupportOracle` BROKEN, 1 error (MAJOR; 26.1.2 B2-O03 reproduces)

Batch 0: fails all three classpaths, class "other", first error "45:
CubeVoxelShape(DiscreteVoxelShape) has protected access in
CubeVoxelShape". My compile agrees on both classpaths (1 error).
Javap-verified: `CubeVoxelShape(DiscreteVoxelShape)` is `protected`
in the 1.21.10 jar. Major because it blocks the piston-shape cascade
(B2-O07) and is referenced by ~25 collision/support drivers
(`test_face_support_parity.py`, `test_block_collision_scan_parity.py`,
`test_dynamic_support_parity.py`, `test_ground_support_parity.py`,
and the collision family). Not a 1.21.7 item (the 1.21.7 sweep did
not mint FaceSupport separately); carried from the 26.1.2 run.

### B2-O07 — `VanillaPistonShapeOracle` BROKEN by cascade, no own errors (MINOR; 26.1.2 B2-O04 reproduces)

Batch 0: fails all three classpaths, class "cascade", "depends on
broken: VanillaFaceSupportOracle", `cascade_via`:
[VanillaFaceSupportOracle]. My compile agrees (0 own errors on both
classpaths). The dependency is direct: `VanillaPistonShapeOracle.java:22`
references `VanillaFaceSupportOracle.class.getDeclaredMethod("emit",
...)`. Fix FaceSupport and this one compiles.

### B2-O08 — `redstone_diff.py` defaults `--resources` to the 26.3 path (MINOR; 26.1.2 B2-O05 reproduces)

`tools/redstone_diff.py:453` defaults `--resources` to
`.local/horizon-client/26.3`. No prepared 1.21.10 data exists at that
path (and no 26.3 data exists in this worktree at all), so the driver
fails out of the box for 1.21.10.

### B2-O09 — `piston_step_e2e.py` defaults `--resources` to the 26.3 path (MINOR; 26.1.2 B2-O06 reproduces)

`tools/piston_step_e2e.py:31` defaults `--resources` to
`.local/horizon-client/26.3`. Same failure as B2-O08.

### Oracles assessed PASSING for 1.21.10 (Batch 0 raw measurement + my compiles)

- **`VanillaBlockPushOracle`**: pass / pass / pass (server / client /
  both). My compile agrees on both classpaths (0 errors).
- **`VanillaClientRedstonePreviewOracle`**: FAIL server (8 errors —
  client-only imports, expected), **pass client, pass both**. My
  client-classpath compile agrees (0 errors). Not an oracle defect:
  it is a client oracle and it compiles on its native classpath.

### Adjacent failing oracle (out of Batch 2 scope — recorded, not minted)

- **`VanillaLevelShapeOracle`**: fails all three classpaths (2
  errors, first "16: package net.minecraft.world.attribute does not
  exist"). Batch 0's cascade analysis puts PistonShape's dependency
  on FaceSupport only, so LevelShape is not in this batch's closure —
  flagged for the coordinator / owning batch.

### B2-H01 — Tick-by-tick circuit equivalence + piston corpus driver runs (needs-harness; 1.21.7 + 26.1.2 B2-H01 reproduce)

The 10 Batch 2 drivers (`test_redstone_parity`,
`test_redstone_catalog_parity`, `test_redstone_persistence_parity`,
`test_piston_push_parity`, `test_piston_movement_parity`,
`test_piston_shape_parity`, `test_face_support_parity`,
`test_client_redstone_preview_parity`, `test_block_push_parity`,
`block_update_harness.py`) cannot run for 1.21.10: five in-scope
oracles do not compile (B2-O01–B2-O06), one driver constant is wrong
(B2-G05), two drivers default to the 26.3 resources path (B2-O08/
B2-O09), and — independently — no prepared 1.21.10 data exists, so a
prepare run is needed first. The full per-state nibble spot-check
also waits on that prepare.

### B2-H02 — The 4 level::tests need a cargo test run (needs-harness; 1.21.7 + 26.1.2 B2-H02 reproduce)

`crates/horizon-world/src/level/tests.rs` carries the same 4 tests at
the same lines as 26.1.2 (:1253, :1734, :1781, :2120). The `world!()`
macro silently skips without prepared data, and no prepared 1.21.10
data exists — a prepare run is needed before the run. Expected
outcomes (from reading the test bodies against the defects above):

| Test (`level/tests.rs`) | Verdict | Root cause |
| `redstone_lever_wire_and_lamp` (:1253) | **Expected PASS** — run owed (B2-H02) | Wire dispatches now (B2-G01 fixed); asserts power "15" at x=1, "11" at x=5, lamp lit then unlit |
| `levers_power_wire_through_interaction` (:1734) | **Expected PASS** — run owed (B2-H02) | Same (B2-G01); also exercises button release after 20 ticks (press time verified, §2.4) |
| `pistons_push_and_sticky_pistons_pull` (:1781) | Expected pass — run owed (B2-H02) | Dirt/redstone_block are NORMAL reaction → PushPull either way; push/pull logic verified |
| `pistons_pop_fragile_blocks_and_refuse_obsidian` (:2120) | **Expected FAIL** — run owed (B2-H02) | Torch (DESTROY) pushed, no DropResources event (B2-G02); obsidian pushed instead of refused (B2-G03) |

This is a change from 26.1.2, where 3 of 4 were expected to fail: the
two wire tests flip to expected-pass on the G7 fix. Only the
pop/obsidian test is still expected to fail.

### Withdrawn candidates — CORRECTION: G11 and G12 NEVER EXISTED (land as closed items B2-W01/B2-W02)

#### G11 — Comparator conductor branch: NEVER EXISTED (not WITHDRAWN)

The 1.21.7 G11 premise was "vanilla takes the frame first" (the
lectern/item-frame analog signal wins over the block signal), with the
port taking the max. **1.21.7's own bytecode contradicts the premise.**
1.21.7's `ComparatorBlock.getInputSignal` computes
`Math.max(frameAnalogOrMIN, blockAnalogOrMIN)` at offset 140
(`b02/evidence/piston_comparator_1_21_7.txt`). The 1.21.7↔1.21.10
ComparatorBlock diff (`b02/evidence/piston_comparator.diff`) shows
ONLY the `getAnalogOutputSignal` signature change (2-arg→3-arg) —
vanilla never changed the max rule. 1.21.10's
`getInputSignal` takes the same max at offset 150
(`b02/evidence/comparator.txt`, lines 156–168), and the port takes the
max (`redstone.rs:544`: `frame.into_iter().chain(block).max()`, with
the comment "the larger signal winning"). The port was always right
and vanilla was always max — there was never a defect to withdraw.
The 26.x runs' "vanilla changed" framing was wrong (their verdict —
port matches — was right; the reason was wrong). **Coordinator: please
correct the 1.21.7 ledger entry for G11 and the 26.x summaries'
framing.** Same correction class as the 26.1.2 B5-G18 bell correction
(a misread branch direction that never existed).

#### G12 — Piston-destroyed blocks notify twice: NEVER EXISTED (not WITHDRAWN)

The 1.21.7 G12 premise was "vanilla destroys via
`Level.destroyBlock`, flags 3" (so the port's `setBlock(air, 18)` +
explicit neighbour updates double-notified). **1.21.7's own bytecode
contradicts the premise.** 1.21.7's `PistonBaseBlock.moveBlocks` uses
`setBlock(air, 18)` at offset 302 (`bipush 18`, line 1383 of the
1.21.7 dump) and has NO `destroyBlock` call anywhere in the class.
The PistonBaseBlock diff shows ONLY `isClientSide` field→method
changes — `moveBlocks` is identical between 1.21.7 and 1.21.10.
1.21.10's `moveBlocks` destroy loop uses the same `setBlock(air, 18)`
at offset 302, plus dropResources, the BLOCK_DESTROY game event, and
the client-side levelEvent 2001 (`b02/evidence/pistonbase.txt`, lines
698–900+); the port matches (`pistons.rs:599`, flags 276/18/324/82 at
:564/:599/:612,:638/:658). There was never a defect to withdraw.
**Coordinator: please correct the 1.21.7 ledger entry for G12 and the
26.x summaries' framing.**

## §5 Re-check verdicts

### 1.21.7 re-checks (PRIMARY — all 13 decided)

| 1.21.7 item | Verdict for 1.21.10 | Evidence |
| G7 wire class name (major) | **FIXED** → B2-G01 (fixed) | `classes.rs:228` alias `RedstoneWireBlock \| RedStoneWireBlock`; census chain `['RedStoneWireBlock', 'Block']` recognised; all wire arms dispatch. Prepare half still broken → B2-G08 |
| G8 push-reaction loader (major) | **REPRODUCES** → B2-G02 | Jar enum NORMAL/DESTROY/BLOCK/IGNORE/PUSH_ONLY; facts carry those names (22,669/6,847/91/64 states); loader `data.rs:883-886` matches only 26.3 names |
| G9 obsidian refusal (major) | **REPRODUCES** → B2-G03 | Jar `isPushable` offsets 44–85 list the four blocks; port has no such check; `named` lacks crying_obsidian/reinforced_deepslate |
| G10 Y-bound off by one (minor) | **REPRODUCES** → B2-G04 | Jar allows y == maxY, refuses UP-push only from maxY; port uses `max_y() - 1` (319) |
| G11 comparator conductor (minor) | **NEVER EXISTED** → B2-W01 (closed) | 1.21.7's own `getInputSignal` takes `Math.max(frame, block)` (offset 140); 1.21.10 same (offset 150); port takes max |
| G12 destroy notify twice (minor) | **NEVER EXISTED** → B2-W02 (closed) | 1.21.7's own `moveBlocks` uses `setBlock(air, 18)` (offset 302), no `destroyBlock` anywhere; 1.21.10 identical; port matches |
| G13 shape-driver count (minor) | **REPRODUCES** → B2-G05 | `test_piston_shape_parity.py:63` asserts 35723; 1.21.10 has 29,671 states |
| G14 0.01 height divergence (minor) | **REPRODUCES** → B2-G06 | Cobweb driver docstring documents it; height deliberately uncompared |
| O1 BlockUpdate oracle (major) | **REPRODUCES** → B2-O01 | 63 errors (Batch 0 + my compile); 1.21.7 had ≈25 distinct causes |
| O2 RedstoneCatalog oracle (major) | **REPRODUCES, bigger** → B2-O02 | 36 errors (Batch 0 + my compile); 1.21.7 had ≈15 distinct causes |
| O3 RedstonePersistence oracle (minor) | **REPRODUCES** → B2-O03 | 1 error, ServerLevel ctor — same single break as 1.21.7 |
| O4 PistonPush oracle (minor) | **REPRODUCES** → B2-O04 | 2 errors, `monster.zombie` package — same break as 1.21.7 |
| O5 PistonMovement oracle (minor) | **REPRODUCES** → B2-O05 | 1 error, ServerLevel ctor — same single break as 1.21.7 |

### 26.1.2 re-checks (SECONDARY — all 17 decided)

| 26.1.2 item | Verdict for 1.21.10 | Evidence |
| B2-G01 wire class name (major) | **FIXED** → B2-G01 (fixed) | Same as G7 above: the alias mechanism exists in the 1.21.10 tree |
| B2-G02 push-reaction loader (major) | **REPRODUCES** → B2-G02 | Same loader, same 26.3-only match (`data.rs:883-886`) |
| B2-G03 obsidian refusal (major) | **REPRODUCES** → B2-G03 | Same missing hardcode (`pistons.rs:328-372`) |
| B2-G04 Y-bound off by one (minor) | **REPRODUCES** → B2-G04 | Same `max_y() - 1` (`pistons.rs:337`) |
| B2-G05 shape-driver count (minor) | **REPRODUCES** → B2-G05 | Same 35723 assert; 1.21.10 has 29,671 states |
| B2-G06 0.01 height divergence (minor) | **REPRODUCES** → B2-G06 | Same docstring, same deliberate non-comparison |
| B2-G07 diode same-block guard (minor) | **REPRODUCES** → B2-G07 | Jar `neighborChanged` has no guard (offset 0 → canSurvive); port `redstone.rs:673-675` returns early |
| B2-O01 BlockUpdate oracle (major) | **REPRODUCES, bigger** → B2-O01 | 63 errors vs 26.1.2's 32 (Batch 0 + my compile) |
| B2-O02 RedstoneCatalog oracle (minor) | **REPRODUCES, bigger** → B2-O02 | 36 errors vs 26.1.2's 4 (Batch 0 + my compile) |
| B2-O03 FaceSupport oracle (major) | **REPRODUCES** → B2-O06 | 1 error, protected `CubeVoxelShape` ctor — identical to 26.1.2 |
| B2-O04 PistonShape cascade (minor) | **REPRODUCES** → B2-O07 | Batch 0 class "cascade", depends on broken `VanillaFaceSupportOracle` |
| B2-O05 redstone_diff 26.3 default (minor) | **REPRODUCES** → B2-O08 | `redstone_diff.py:453` defaults `--resources` to the 26.3 path |
| B2-O06 piston_step_e2e 26.3 default (minor) | **REPRODUCES** → B2-O09 | `piston_step_e2e.py:31` defaults `--resources` to the 26.3 path |
| B2-H01 driver runs (minor) | **REPRODUCES** → B2-H01 | Oracles still broken (O01–O06), driver constant still wrong (G05); prepared data NOT present for 1.21.10 — a prepare run is also needed |
| B2-H02 level::tests run (minor) | **REPRODUCES, improved** → B2-H02 | 4 tests at the same lines; 3 of 4 now expected to PASS (wire tests flip on the G7 fix); only the pop/obsidian test expected to fail; a 1.21.10 prepare is needed before the run |
| B2-W01 G11 comparator (minor, closed) | **CLOSED again, corrected** → B2-W01 | NEVER EXISTED — see §4; 1.21.7's own bytecode contradicts the premise |
| B2-W02 G12 destroy notify (minor, closed) | **CLOSED again, corrected** → B2-W02 | NEVER EXISTED — see §4; 1.21.7's own bytecode contradicts the premise |

**O-item numbering note:** the 1.21.10 O-series is ordered by the
1.21.7-primary spine (O01 BlockUpdate, O02 Catalog, O03 Persistence,
O04 PistonPush, O05 PistonMovement — matching the 1.21.7 O1–O5 order)
and extended with the 26.1.2-only items (O06 FaceSupport = 26.1.2
B2-O03, O07 PistonShape = 26.1.2 B2-O04, O08 redstone_diff = 26.1.2
B2-O05, O09 piston_step_e2e = 26.1.2 B2-O06). The mapping is in the
table above and in each item's header.

## §6 Systemic findings

### The 1.21.11-template lineage fixed the wire and left the rest

The 1.21.10 tree is a delta port of the 1.21.11 template, and the
template's class-alias mechanism is the single biggest behaviour
difference from the 26.1.2 tree: wire, farmland, snow, fungus, roots
and path all dispatch through aliases that the 26.1.2 tree lacks
entirely. But the alias mechanism is a table-layer fix, and the
deeper behaviour defects survive untouched: the push-reaction loader
still matches only 26.3 names, `is_pushable` still lacks the obsidian
hardcode and still computes the ceiling off by one, and the diode
guard still carries the 26.2 form. The template fixed the dispatch
layer; the logic layer still carries the 26.3 form. The 26.2 tree is
the natural reference for the G8/G9/G10 repairs (it carries all three
fixes, per the 26.1.2 report).

### The vanilla-check rule mattered in both directions again

G11 and G12 are the sharpest instance yet of why the rule exists: the
1.21.7 sweep's premises about vanilla 1.21.7 were wrong in both cases,
and the error propagated through the 26.x runs as "vanilla changed"
framing. Checking the target version's own bytecode — not the next
version's, not the base's — is what catches it. The diode guard (B2-G07)
is the same rule in the opposite direction: a real defect the 1.21.7
sweep missed, because the 1.21.7 tree's `redstone.rs` carries the
26.2-form guard and the 1.21.7 sweep never compared
`DiodeBlock.neighborChanged` against 1.21.7's own jar. **Coordinator:
the 1.21.7 ledger owes three corrections — G11 and G12 to NEVER
EXISTED, and a new entry for the diode guard.**

### The oracle gap is a version-shape gap, not oracle rot

The 26.1.2 run's oracle picture (32 / 4 / 1 / cascade, with the three
persistence/piston oracles passing) looks nothing like 1.21.10's
(63 / 36 / 1 / 2 / 1 / 1 / cascade). The difference is the API
surface: 26.1.2 has the `zombie` subpackage, the 26.3-form ServerLevel
ctor, and most of the packages the oracles import; 1.21.10 has none
of them. The oracles were written against 26.3 and adapted
incrementally down the 26.x line; the E6 line needs a proper
adaptation pass (GameRules package, ServerLevel/MinecraftServer/
ServerChunkCache ctor arities, `isEpollEnabled` vs
`useNativeTransport`, `monster.Zombie`, the CubeVoxelShape factory)
before any Batch 2 driver can run for any E6 version. The three
single-break oracles (O03/O04/O05) are the cheapest wins: one ctor
call / one import each.

### Programme-wide 35723 hardcode

The `test_piston_shape_parity.py:63` assert is one instance of a
programme-wide pattern: many drivers hardcode the 26.3 state count
35723 (`test_block_landing_parity.py`,
`test_block_properties_parity.py`, `test_collision_parity.py`, …).
Every E6 version (29,671 states for 1.21.10) trips every one of them.
Flagged for the coordinator — a single shared constant (or a
datagen-derived count) would fix the whole family.

### Out-of-scope flags for the coordinator

- **`hooks.rs:549` reads `b.tags.blocks_motion`** — the 26.3
  motion-tag read; the 1.21.7 tree used a per-state `blocks_motion()`
  rule. B3-G22 class; belongs to the motion-tag batch.
- **`amethyst_block` falls back to plain Block** despite a non-Block
  head (`AmethystBlock`) — the only such block in the 1.21.10
  class-table census (278 known java names incl. aliases). Out of
  Batch 2 scope; belongs to the block/class batch. (72 blocks have an
  unrecognised head but a recognised ancestor — leaves, stained
  glass, furnaces, WeatheringCopper variants — behaviour largely
  carried by the ancestor.)
- **`VanillaLevelShapeOracle`** fails all three classpaths (2 errors,
  `world.attribute` missing) — adjacent failing oracle, not in this
  batch's cascade closure (§4).

## §7 Evidence index

| Evidence | Location |
| `RedStoneWireBlock` only; no `RedstoneWireBlock` | jar listing, `server-game-named.jar` (unzip -l grep) |
| `PushReaction` enum NORMAL/DESTROY/BLOCK/IGNORE/PUSH_ONLY | `b02/evidence/pushreaction.txt` (javap on 1.21.10 server-game-named.jar) |
| `isPushable` obsidian refusal (offsets 44–85) + Y bounds (0–33, 106–125) + reaction switch (179–232) | `b02/evidence/pistonbase.txt` (lines 581–697) |
| `moveBlocks` destroy `bipush 18` (offset 302) + dropResources + BLOCK_DESTROY + levelEvent 2001 | `b02/evidence/pistonbase.txt` (lines 698–900+) |
| `ComparatorBlock.getInputSignal` `Math.max` (offset 150) | `b02/evidence/comparator.txt` (lines 156–168) |
| `DiodeBlock.neighborChanged` no guard (offsets 0–94) | `b02/evidence/diode.txt` (lines 140–210) |
| 1.21.7 `ComparatorBlock.getInputSignal` `Math.max` (offset 140) + 1.21.7↔1.21.10 comparator diff | `b02/evidence/piston_comparator_1_21_7.txt`, `b02/evidence/piston_comparator.diff` |
| 1.21.7 `moveBlocks` `bipush 18` (offset 302, line 1383) + no `destroyBlock` in class | `b02/evidence/piston_comparator_1_21_7.txt` (PistonBaseBlock at line 556; `destroyBlock` count 0) |
| `NeighborUpdater.UPDATE_ORDER` West/East/Down/Up/North/South | `b02/evidence/neighborupdater.txt` (static init, lines 174–178) |
| `ServerLevel` 11-arg / `MinecraftServer` 8-arg + abstract `isEpollEnabled` / `ServerChunkCache` 11-arg / `CubeVoxelShape` protected ctor / `DataLoadContext` (no `datapackWorldRegistries`) | `b02/evidence/serverlevel.txt`, `minecraftserver.txt`, `serverchunkcache.txt`, `cubevoxelshape.txt`, `dataloadcontext.txt` |
| `Zombie` at `monster.Zombie`; no `monster/zombie` package; `GameRules` at `world.level.GameRules`; 0 entries for blockscan/clock/attribute/permissions/gizmos | jar listing (unzip -l grep) |
| Full-jar class comparison 1.21.7 ↔ 1.21.10 (73 / 309 / 1,153; 16 in-scope identical, 24 changed) | `b02/evidence/class_compare_1_21_7_vs_1_21_10.json` (`class_compare.py`) |
| Normalized javap diff of the 24 changed in-scope classes (556 lines, API-shape/debug only) | `b02/evidence/javap_changed.diff` (+ `javap_changed_1_21_10.txt` / `javap_changed_1_21_7.txt`) |
| Census: 1.21.10 blocks/states by reaction; 1.21.7 validation (0 mismatches, 1,105 blocks) | `b02/evidence/blocks_1_21_10.json`, `b02/evidence/census_1_21_10.log`, `b02/evidence/validate_1_21_7.py`, `b02/evidence/extract_blocks.py` |
| `Blocks` clinit dumps | `b02/evidence/blocks_1_21_10.txt` (44,959 lines), `b02/evidence/blocks_1_21_7.txt` (43,163 lines) |
| Oracle compiles (server + client classpaths) | `b02/evidence/javac/server_compile.log`, `b02/evidence/javac/client_compile.log`, `b02/evidence/javac/{server,client}_cp.txt` |
| Batch 0 raw measurement (391 files, all three classpaths) | `versions/1.21.10/.local/measure-oracles/results.json` |
| `classes.rs:228` wire alias; `data.rs:883-886` loader; `data.rs:876-877` nibble columns; `data.rs:1012` press times; named `obsidian` :270, `respawn_anchor` :372 | `versions/1.21.10/crates/horizon-world/src/level/` |
| `pistons.rs:337` `max_y() - 1`; no obsidian check; flags 276/18/324/82 at :564/:599/:612,:638/:658 | `versions/1.21.10/crates/horizon-world/src/level/pistons.rs` |
| `redstone.rs:673-675` diode guard; `redstone.rs:544` comparator max; `redstone.rs:414-433` wire power | `versions/1.21.10/crates/horizon-world/src/level/redstone.rs` |
| `survival.rs:45` `RedstoneWireBlock`; `bind_definitions` :102–117 | `versions/1.21.10/crates/horizon-prepare/src/survival.rs` |
| Exporter row layout (weak/strong cols 5/6); `"push"` = vanilla enum name | `VanillaBlockBehaviourExporter.java:198-220`, `VanillaGameplayFactsExporter.java:50` |
| Press times stone 20 / wood 30 | 1.21.10 datagen `blocks.json` definitions + 1.21.7 export `fields` |
| 4 level::tests at :1253/:1734/:1781/:2120 | `versions/1.21.10/crates/horizon-world/src/level/tests.rs` |
| `test_piston_shape_parity.py:63` (35723); cobweb docstring (0.01); `redstone_diff.py:453`; `piston_step_e2e.py:31` | `versions/1.21.10/tools/` |

## §8 Verdict

**53 checks: 46 pass, 7 gap.** The wire works — the headline fix of
this run, and the first E6 version where redstone wire dispatches.
Two 1.21.7 majors still reproduce (push-reaction loader, obsidian
refusal), the ceiling bound is still off by one, the shape driver
still cannot run, the 0.01 height divergence still stands, the diode
guard still carries the 26.2 form, and the wire's prepare-side
survival rule never binds (new). All five 1.21.7 oracles are still
broken, the two big ones much worse than in the 26.x runs, and the
driver runs + level::tests still need a prepare run first. Two 1.21.7
items never existed — the ledger correction is owed to the
coordinator, not to this batch.
