# Batch 2 — Redstone + pistons (1.21.11 parity sweep)

**Version:** 1.21.11 · **Date:** 2026-11-25 · **Branch:** `parity-sweep-1.21.11`
(workdir `~/workspace/mcir-work/sweep-1.21.11`) ·
**Scope:** `docs/PARITY-SWEEP.md` §4 Batch 2 — redstone ticking, pistons,
observers, comparators; 10 drivers + circuit probes; pilot leftovers
(4 level::tests).

## §0 Execution status

Checks + triage only. No file under `versions/1.21.11/` was modified,
nothing was committed, no git commands ran in the worktree, no cargo
builds ran. Everything below is static analysis: `javap -c` on the named
1.21.11 server-game jar
(`~/workspace/mcir-work/e6-deltas/versions/1.21.11/jars/server-game-named.jar`,
sha1 `7f0d85c12aeb57f14687b7ab2ae2e5d825fbf0d2`, verified, 7,163
classes, Mojang-mapped; the bundler `server.jar` was never javap'd;
`client-named.jar` sha1 `29bddf144a2f2907528b25d616770bfea67c4792`,
verified, 10,291 classes), source comparison against
`versions/1.21.11/crates/`, the 1.21.11 datagen reports
(`~/workspace/mcir-work/e6-deltas/versions/1.21.11/datagen/reports/`,
1,166 blocks, 29,671 states), a full-jar class comparison against the
1.21.10 server-game jar (sha1
`ba1fc89ead95b8ebdb8727f84e2bd487533c8b7f`), the 1.21.7 prepared export
as the validated census reference
(`~/workspace/mcir-work/mob-spawn/resources/`, see §2.3 — no prepared
1.21.11 data exists in the worktree), and my own targeted javac
compiles of all 9 in-scope oracles on both the server and client
classpaths (15-file closure incl. deps — see the note on the Batch 0
table below).

**Headline:** the template is where the wire was fixed — the 1.21.11
tree carries the class-alias mechanism (1.21.7 G7 closes here, at the
source, not just in the child), and the oracle picture improves
sharply because vanilla 1.21.11 grew the APIs the oracles were written
against: the `gamerules`, `permissions`, `attribute` and `gizmos`
packages all arrived, `ResourceLocation` became `Identifier`, and the
`monster.zombie` subpackage exists — so `VanillaPistonPushOracle`
compiles clean for the first time in the E6 lineage (1.21.7 O4 closes),
`VanillaBlockUpdateOracle` drops from 63 errors to 38 and
`VanillaRedstoneCatalogOracle` from 36 to 10. Everything else from
1.21.10 survives: every state still loads PushPull (G8 → B2-G02),
obsidian still pushes (G9 → B2-G03), the ceiling bound is still off by
one (G10 → B2-G04), the shape driver still cannot run (G13 → B2-G05),
the 0.01 height divergence still stands (G14 → B2-G06), the diode guard
still carries the 26.2 form (B2-G07), and the wire's prepare-side
survival rule still never binds (B2-G08). Two 1.21.7 items are
**NEVER EXISTED, not WITHDRAWN**, confirmed fresh against 1.21.11's own
jar: vanilla 1.21.11's `ComparatorBlock.getInputSignal` still takes
`Math.max` at offset 150, and `PistonBaseBlock.moveBlocks` still uses
`setBlock(air, 18)` at offset 302 with no `destroyBlock` anywhere in
the class (§4 B2-G11/B2-G12). One vanilla behaviour change in scope:
the daylight detector now reads the `SUN_ANGLE` environment attribute
and takes `Mth.cos(double)` — and the port already matches that new
form (§6), because the template was written against 1.21.11's API.

**One delta vs 1.21.10 (precondition, not a defect):** 1.21.11 has NO
prepared horizon-client data in the worktree
(`versions/1.21.11/.local/` does not exist at all). The 1.21.7 run
could point at its prepared export; for 1.21.11 the driver runs
(B2-H01) and the level::tests (B2-H02) need a prepare run first, on top
of the oracle repairs. The census/nibble/press-time checks in this
report were still run as real checks — the census against 1.21.11's own
jar (validated on 1.21.7, §2.3), the nibbles/press-time against the
exporter + loader + datagen chain (§2.4).

**Note on the Batch 0 table:** the landed
`docs/sweep/1.21.11/batch-0-oracles.json` had NOT appeared at handoff
(checked 07:00 NPT Nov 25; `docs/sweep/1.21.11/` holds only
extract-inventory.py, inventory-1.21.11.json, inventory.md), and no
`versions/1.21.11/.local/measure-oracles/results.json` exists either.
Triage here rests on my own targeted javac compiles of all 9 in-scope
oracles on both classpaths (`--release 21`, server classpath =
server-game-named.jar + 39 server library jars + jsr305; client
classpath = client-named.jar + 60 client libraries + jsr305; logs in
`b02/evidence/javac/`), with the dependency closure included
(`ParityData`, `VanillaCollisionOracle`,
`VanillaBlockStateCodecOracle`, `VanillaHeightmapOracle`,
`VanillaStateCodec` — a first compile without the last two produced
five artifact errors, all `cannot find symbol` for the missing sources,
which the closure compile removes; the counts below are from the
closure compile). Every failing oracle's causes were additionally
javap-verified fresh against the 1.21.11 jar. If the landed Batch 0
table disagrees with my compiles on any in-scope oracle, the landed
table wins and this batch's O-items need a re-read. Note the jsr305
jar is not present in the 1.21.11 tree's `tools/`; the compile used
the 1.20.5 tree's copy, sha1-identical to the one the 1.21.10 run used
(`25ea2e8b0c338a877313bd4672d3fe056ea78f0d`).

## §1 Scope

Redstone wire power/updates, comparators, repeaters, observers, pistons
(push/pull/destroy, `is_pushable`, shape driver), block-update ordering,
redstone persistence. Method: `javap -c` on the named 1.21.11
server-game jar + source comparison against `versions/1.21.11/crates/` +
tag/data reads. No cargo builds, no driver runs. The 4 level::tests and
the circuit-equivalence / signal-nibble checks that need prepared data
or a test run are needs-harness items (B2-Hxx), not faked passes —
except that the nibble/press-time checks were run here as real checks
against the exporter + loader + datagen chain (§2.4), and the census
was run as a real check against 1.21.11's own jar (§2.3).

Out of scope for this batch (noted, not verified): the 1.21.11 delta's
behaviour items outside redstone/pistons; the B3-G22 motion-tag startup
blocker (one instance flagged in §6 — `hooks.rs:549` reads
`b.tags.blocks_motion`, a 26.3 motion-tag read); the amethyst-block
class-table fallback (§6); `VanillaLevelShapeOracle` (FIXED for
1.21.11 — 0 errors on both classpaths, was 2 errors in 1.21.10 —
adjacent oracle, recorded in §4, not minted).

## §2 Method

Five verification layers, all fresh against 1.21.11 evidence:

1. **Jar identity proof.** A full sha1 comparison of ALL classes
   between the 1.21.10 and 1.21.11 server-game jars: 6,964 vs 7,163
   classes; 520 only in 1.21.10, 719 only in 1.21.11, 3,131 changed.
   The churn is dominated by Mojang's renames, not behaviour:
   `ResourceLocation` → `Identifier` (constant-pool churn in nearly
   every class), `GameRules` moving from `world.level` to
   `world.level.gamerules`, the `monster.zombie` subpackage arriving,
   the `vehicle/minecart` and `projectile/arrow` package moves, and
   `critereon` → `criterion`. Twelve in-scope core classes are
   byte-identical, including `PushReaction`,
   `PistonStructureResolver`, `PistonMovingBlockEntity`,
   `RepeaterBlock`, `ObserverBlock`, `SignalGetter`,
   `PoweredRailBlock`, `TripWireBlock`, `DropperBlock`,
   `PressurePlateBlock`, `WeightedPressurePlateBlock`,
   `CubeVoxelShape`. Twenty-nine in-scope classes changed and were
   normalized-`javap -c` diffed (`b02/evidence/normdiff.py` — offsets
   and constant-pool indices stripped, the Identifier/GameRules/Util
   renames cancelled): **22 are normalized-identical**, including
   `PistonBaseBlock`, `DiodeBlock`, `RedStoneWireBlock`,
   `DefaultRedstoneWireEvaluator`, `NeighborUpdater`,
   `CollectingNeighborUpdater`, `LeverBlock`, `HopperBlock`,
   `DispenserBlock`, `CopperBulbBlock`, `MovingPistonBlock`,
   `PistonHeadBlock`. Seven differ, all reviewed hunk by hunk:
   `ButtonBlock` and `TargetBlock` (arrow package move only),
   `DetectorRailBlock` (minecart package move only),
   `CommandBlock` (`performCommand(Level)` → `(ServerLevel)`, the
   data-defined `GameRules.get(SEND_COMMAND_FEEDBACK)` /
   `get(MAX_COMMAND_SEQUENCE_LENGTH)` forms, branch-offset shifts),
   `ExperimentalRedstoneWireEvaluator` (`BiConsumer` →
   `ObjectIntBiConsumer`, `Integer` → `int` unboxing),
   `ComparatorBlock` (one hunk: a dead null check removed from the
   `lambda$getItemFrame$0` predicate — the lambda runs over a query
   that never yields null; `getInputSignal`'s `Math.max` is untouched
   at offset 150), and `DaylightDetectorBlock` (the one real
   behaviour-form change in scope — see §6; the port matches the new
   form). So 1.21.10 pass verdicts carry for these classes by
   (jar-method identity modulo reviewed API-shape/debug + port-file
   identity).
2. **Tree identity proof.** `crates/horizon-world/src/level/` between
   the 1.21.10 and 1.21.11 trees in the worktree: `pistons.rs`,
   `redstone.rs`, `update.rs`, `signals.rs`, `hooks.rs`, `mod.rs` and
   the `redstone/` subdir are BYTE-IDENTICAL. The differing in-scope
   files (`classes.rs` — 8 lines, `data.rs` — 4, `shapes.rs` — 10,
   `interact.rs` — 2, `place.rs` — 2, `survive.rs` — 32, `tick.rs` —
   12, `tests.rs` — 44) differ ONLY in comments (`1.21.10` → `1.21.11`
   in doc text) — zero code changes in scope. The wire arms, the
   loader, `is_pushable`, the diode guard and the 4 level::tests are
   at the same lines with the same code. `survival.rs` differs only in
   the same comment renames. All four in-scope tools drivers are
   byte-identical (`test_piston_shape_parity.py`,
   `test_projectile_piston_cobweb_parity.py`, `redstone_diff.py`,
   `piston_step_e2e.py`). Of the 9 in-scope oracle sources, 6 are
   byte-identical; the 3 that differ (`VanillaBlockUpdateOracle`,
   `VanillaRedstoneCatalogOracle`, `VanillaFaceSupportOracle`) differ
   ONLY by the `ResourceLocation` → `Identifier` rename (2 lines
   each).
3. **Census (method validated on 1.21.7, re-run on 1.21.11).** The
   1.21.10 run's extraction script (`b02/evidence/extract_blocks.py`,
   unchanged) run against a fresh `javap -c -p -v` dump of 1.21.11's
   `Blocks` (44,971 lines): all 1,166 blocks, 0 missing — blocks by
   reaction NORMAL 784 / DESTROY 353 / BLOCK 13 / PUSH_ONLY 16; states
   by reaction NORMAL 22,669 / DESTROY 6,847 / BLOCK 91 / PUSH_ONLY 64
   (total 29,671) — identical counts to 1.21.10. Spot checks:
   redstone_wire = RedStoneWireBlock/DESTROY; obsidian =
   Block/NORMAL; piston = PistonBaseBlock/BLOCK. Class-table census:
   278 known java names incl. aliases; the only plain-Block fallback
   is still `amethyst_block`; 72 blocks with an unrecognised head but
   a recognised ancestor — all identical to 1.21.10.
4. **Nibbles/press-time chain.** The loader columns were verified
   against the exporter, not the data: the behaviour exporter
   (`VanillaBlockBehaviourExporter.java:198-220`) packs weak/strong
   signals (4 bits per direction, from `state.getSignal` /
   `state.getDirectSignal` over an empty level) into row columns 5/6,
   and the loader reads `weak: int(5), strong: int(6)`
   (`data.rs:876-877`) — columns agree. Press times: the 1.21.11
   datagen definitions carry `ticks_to_stay_pressed` (stone 20, wood
   30); the jar's `ButtonBlock` has `private final int
   ticksToStayPressed`; the exporter dumps fields reflectively (so the
   key exists with the right values — confirmed in the 1.21.7 export:
   stone_button 20, oak_button 30); the port reads
   `field("ticksToStayPressed")` (`data.rs:1012`). The full per-state
   nibble spot-check still needs prepared 1.21.11 data — folded into
   B2-H01.
5. **Oracle measurement.** My own targeted javac compiles of the 9
   in-scope oracles plus their dependency closure (15 files) on both
   the server and client classpaths (`b02/evidence/javac/`, see §0).
   Every failure cause was javap-verified against the 1.21.11 jar
   (§4 B2-O01–B2-O07). No Batch 0 table existed to cross-check
   against (see §0).

## §3 Checks table

- **Behaviour checks: 53 — 46 pass, 7 gap** (across 7 defect items
  B2-G02–B2-G08: 2 major, 5 minor).
- **Behaviour/non-behaviour split: 53 behaviour + 0 non-behaviour.**
  Batch 2 is not one of the three as-tallied exception batches; the
  tally counts behaviour checks only.
- **Oracle/driver defects: 8** (B2-O01–B2-O03, B2-O05–B2-O09: 3 major,
  5 minor — not counted in the check tally). One 1.21.10 oracle defect
  is FIXED (B2-O04 — the piston-push oracle compiles clean now).
- **Needs-harness: 2** (B2-H01–B2-H02).
- **Fixed: 2** (B2-G01 — the wire; B2-O04 — the piston-push oracle).
- **Closed (never existed): 2** (B2-G11, B2-G12 — 1.21.7's G11 and G12
  never existed; the premises are contradicted by 1.21.11's own
  bytecode, exactly as by 1.21.7's and 1.21.10's).
- **1.21.7 majors: one FIXED, two REPRODUCE in 1.21.11** (G7 → B2-G01
  fixed; G8, G9 → B2-G02, B2-G03 reproduce). The 1.21.11 tree IS the
  template that carries the class-alias mechanism (which fixes G7)
  but NOT the push-reaction loader fix, the obsidian hardcode, or the
  Y-bound fix — so the template fixed the wire at the source and left
  the rest, and the child inherited both the fix and the leftovers.
- **NEW vs the 1.21.10 run: no new checks, no new defects.** The
  tally is identical to 1.21.10's (53 / 46 / 7) with the same gap
  composition (G02–G08). What changed is underneath the tally: the
  oracle counts (38/10/1/0/1/1/cascade vs 63/36/1/2/1/1/cascade),
  one oracle fixed (O04), and one vanilla behaviour-form change
  (daylight detector) that the port already matches.

Per-section counts sum to the tally: wire 10 (9 pass + G08), machinery
4 (3 pass + G07), consumers 16, nibbles/data 3 (2 pass + G02),
observers 5, pistons 13 (11 pass + G03 + G04), driver constants 2
(G05 + G06) = 53; 46 + 7 = 53.

## §4 Defects

### B2-G01 — Redstone wire FIXED: class-name alias lands at the template (was MAJOR; 1.21.7 G7 FIXED; 1.21.10 B2-G01 FIXED)

Vanilla 1.21.11 names the class `RedStoneWireBlock` (capital S; jar
listing — `net/minecraft/world/level/block/RedStoneWireBlock.class`
exists, `RedstoneWireBlock` does not). The 1.21.7 tree knew only
`RedstoneWireBlock`, so the wire never dispatched. The 1.21.11 tree —
the template itself — carries the alias mechanism:
`crates/horizon-world/src/level/classes.rs:228` reads
`RedstoneWire = "RedstoneWireBlock" | "RedStoneWireBlock"`, via the
macro at :8–23 (`from_java` matches `$java $(| $alt)?`), with a
comment (:9–12) saying aliases are for 1.21.11 renames. Aliases also
exist for Farmland/FarmBlock (:155), Snowy/SnowyDirtBlock (:170),
SpreadingSnowy (:171), NetherFungus/FungusBlock (:204),
NetherRoots/RootsBlock (:206), Path/DirtPathBlock (:212) — the 1.21.7
fallbacks FarmBlock, DirtPathBlock, SnowyDirtBlock are all fixed in
1.21.11. The census confirms the chain:
`redstone_wire → ['RedStoneWireBlock', 'Block']` → recognised head
`RedStoneWireBlock`. All wire arms dispatch on `Class::RedstoneWire`:
`hooks.rs:100,221,390`; `shapes.rs:715`; `interact.rs:263` (+
`wire_use` at :492); `place.rs:1089`; `survive.rs:300`. The wire power
math (`redstone.rs:414-433` — max of neighbours, minus 1), connection
state (`redstone.rs:293-360`), and connecting-side logic
(`redstone.rs:236`) are all live now. **The runtime half of G7 is
FIXED — and the fix originates in this template, not in the child.**
The prepare half is not — see B2-G08.

### B2-G02 — Push-reaction loader mapping matches no 1.21.11 value: every state loads as PushPull (MAJOR; 1.21.7 G8 reproduces; 1.21.10 B2-G02 reproduces)

Vanilla 1.21.11's `PushReaction` enum is NORMAL, DESTROY, BLOCK,
IGNORE, PUSH_ONLY (javap-verified; `PushReaction` is byte-identical to
1.21.10's; `b02/evidence/pushreaction.txt`). The gameplay-facts
exporter writes the vanilla name verbatim
(`VanillaGameplayFactsExporter.java:50`: `"push"` =
`state.getPistonPushReaction().name()`). The census (§2.3) counts the
1.21.11 facts: NORMAL 22,669 / DESTROY 6,847 / BLOCK 91 / PUSH_ONLY 64
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

### B2-G03 — `is_pushable` lacks vanilla's hardcoded obsidian-family refusal (MAJOR; 1.21.7 G9 reproduces; 1.21.10 B2-G03 reproduces)

Vanilla 1.21.11's `PistonBaseBlock.isPushable` refuses four blocks by
hardcoded name at bytecode offsets 44–85: OBSIDIAN, CRYING_OBSIDIAN,
RESPAWN_ANCHOR, REINFORCED_DEEPSLATE
(`b02/evidence/pistonbase.txt`, lines 624–635; `PistonBaseBlock` is
normalized-identical to 1.21.10's). The port's
`is_pushable` (`pistons.rs:328-372`) has NO obsidian check at all — a
piston pushes obsidian like stone. The `named` table has `obsidian`
(:270) and `respawn_anchor` (:372) but NO `crying_obsidian` or
`reinforced_deepslate`, so even a name-based fix needs two new named
entries. Note the census: obsidian's reaction is NORMAL (Block/NORMAL),
so the loader bug (B2-G02) is not the cause — this is a separate
missing hardcode, exactly as in 1.21.7.

### B2-G04 — `is_pushable` Y-bound off by one at the ceiling (MINOR; 1.21.7 G10 reproduces; 1.21.10 B2-G04 reproduces)

Vanilla 1.21.11 refuses a block at `y < minY || y > maxY` (offsets
0–33) and refuses an UP-push only from `y == maxY` (offsets 106–125) —
a block AT y == 320 is pushable sideways/down, and can be pushed up
from 319. The port (`pistons.rs:337`) computes `let (min_y, max_y) =
(self.level.min_y(), self.level.max_y() - 1);` — with `max_y() == 320`
(`mod.rs:606-608`), the port's max is 319, so it refuses y == 320
outright and refuses an UP-push from 319. Off by one at the ceiling,
exactly as in 1.21.7.

### B2-G05 — Piston-shape driver hardcodes the 26.3 state count (MINOR, harness; 1.21.7 G13 reproduces; 1.21.10 B2-G05 reproduces)

`tools/test_piston_shape_parity.py:63` asserts
`len(facts['states']) != 35723` — the 26.3 state count. 1.21.11 has
29,671 states (datagen-verified), so the driver fails its own sanity
check before doing anything. The 35723 hardcode is programme-wide
(`test_block_landing_parity.py`,
`test_block_properties_parity.py`, `test_collision_parity.py`, and
more) — flagged for the coordinator in §6.

### B2-G06 — Documented 0.01 piston-push height divergence (MINOR, pre-existing; 1.21.7 G14 reproduces; 1.21.10 B2-G06 reproduces)

`tools/test_projectile_piston_cobweb_parity.py`'s docstring still
documents that "vanilla's second push here is 0.01 higher than
Horizon's", and the height is deliberately uncompared. The port's
±0.51 clamp (`piston_movement.rs:31-36`) is present and matches
vanilla's movement math; the 0.01 divergence in the second push is a
pre-existing documented gap, unchanged from 1.21.7.

### B2-G07 — `DiodeBlock.neighborChanged` carries the 26.2 same-block guard vanilla 1.21.11 does not have (MINOR; 1.21.10 B2-G07 reproduces)

Vanilla 1.21.11's `DiodeBlock.neighborChanged` has NO same-block guard
— it goes straight to `canSurvive` at offset 0, then
`checkTickOnNeighbor`, else dropResources + removeBlock +
updateNeighborsAt in all directions (offsets 0–94;
`b02/evidence/diode.txt`; `DiodeBlock` is normalized-identical to
1.21.10's). The port
(`redstone.rs:673-675`, in `diode_neighbor_changed`) returns early:
`if !self.b.is_block(self.state(pos), block) { return; }` — so a
repeater/comparator that has already been replaced skips the
neighbor-changed handling vanilla would still run. The port carries
the 26.2-form guard. Note: `redstone.rs` is byte-identical between
the 1.21.7, 1.21.10 and 1.21.11 trees, so this defect exists in 1.21.7
too — but the 1.21.7 sweep never minted it (§6).

### B2-G08 — Wire's prepare-side survival rule never binds — `survival.rs` still uses the wrong class name (MINOR; 1.21.10 B2-G08 reproduces)

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
"1.21.11's class is `FarmBlock` (26.1 renamed it `FarmlandBlock`);
without the 1.21.11 name farmland binds no survival rule" — farmland,
lily pad and dry vegetation were all fixed this way, but the wire was
missed. The 1.21.7 tree has the same wrong line, so this is the
un-minted prepare half of 1.21.7's G7: moot in 1.21.7, where the wire
was dead at runtime too, and now the surviving half of the defect.
Practical impact is small (no vanilla feature places redstone wire),
but the binding is objectively broken.

### B2-O01 — `VanillaBlockUpdateOracle` BROKEN, 38 errors (MAJOR; 1.21.7 O1 reproduces, smaller; 1.21.10 B2-O01 reproduces, smaller — was 63)

My targeted compile (15-file closure, both classpaths): fails server
(38 errors) and client (38 errors). Causes, all javap-verified
against the 1.21.11 jar:

- **Missing packages (2 left, was 5):** `net.minecraft.world.clock`
  (:450 — no ClockInstance; jar listing: 0 entries) and
  `net.minecraft.world.level.blockscan` (:319, :322, :411, :427, :444;
  jar listing: 0 entries). GONE since 1.21.10:
  `net.minecraft.world.level.gamerules` (GameRules moved there in
  1.21.11 — the oracle's import now resolves, and the
  `BLOCK_EXPLOSION_DROP_DECAY` / `MOB_GRIEFING` constants exist as
  `GameRule<Boolean>` with a `get(GameRule<T>)` accessor and a
  `(FeatureFlagSet)` ctor),
  `net.minecraft.world.attribute` (55 entries;
  `EnvironmentAttributeSystem.builder()`,
  `DimensionType.attributes()` / `timelines()` all resolve) and
  `net.minecraft.server.permissions` (19 entries;
  `LevelBasedPermissionSet` / `PermissionSet` resolve).
- **Constructor mismatches:** ServerLevel (:136 — 1.21.11's ctor is
  11-arg: MinecraftServer, Executor, LevelStorageAccess,
  ServerLevelData, ResourceKey, LevelStem, boolean, long,
  List\<CustomSpawner\>, boolean, RandomSequences; the oracle calls
  the 26.3 10-arg form), MinecraftServer (:508 — 1.21.11's ctor is
  8-arg: Thread, LevelStorageAccess, PackRepository, WorldStem, Proxy,
  DataFixer, Services, LevelLoadListener; the oracle calls 11-arg),
  ServerChunkCache (:621 — 1.21.11's ctor is 11-arg; the oracle calls
  10-arg).
- **No-override ×17:** :319/:322 `findBlocksIn` (blockscan cascade),
  :357 `getNextEntityId`, :393 `getOverworldClockTime` (clock
  cascade), :419 `filterPos`, :423 `filterState`, :434
  `atLeastMatched`, :438 `anyMatched`, :441 `forEach`, :444
  `forEachUntil` (blockscan `Scan` overrides), :451 `totalTicks`,
  :452 `partialTick`, :453 `rate`, :454 `isPaused` (ClockInstance
  overrides), :530 `getCommandSpamThresholdSeconds`, :531
  `getChatSpamThresholdSeconds`, :538 `getRandomSequence` (a
  ServerLevel method, not a MinecraftServer one). GONE since 1.21.10:
  the `useNativeTransport` / `isEpollEnabled` pair — 1.21.11 renamed
  the abstract method to `useNativeTransport`, matching the oracle —
  so the "Server is not abstract" error is gone.
- **Cannot-find-symbol ×9:** `filterState` (:424), `statePredicate`
  ×3 (:436, :442, :445), `Continuation` ×3 (:436, :439, :442),
  `WORLD` (:480), `WORLD_REGISTRIES` (:481). GONE since 1.21.10:
  GameRules ×14, Util ×3, LevelBasedPermissionSet ×3,
  EnvironmentAttributeReader, `timelines()`, `attributes()`.
- **Lambda** (:363): `builder.addTimelineLayer(timeline, clock ->
  DAY)` — 1.21.11's `addTimelineLayer` takes `(Holder<Timeline>,
  LongSupplier)`; the oracle passes a one-arg clock lambda. A
  clock-package cascade in spirit (DAY is a ClockInstance that does
  not exist), a signature mismatch in form.
- **Non-static `super`** (:424).
- GONE since 1.21.10: the `RandomSequences` no-arg ctor + `get`
  errors — 1.21.11's `net.minecraft.world.RandomSequences` has a
  public no-arg ctor and `get(Identifier, long)`.

The 1.21.7 run reported "≈25 breaks" (distinct causes); the 1.21.10
run 63 javac errors. The 38 here are javac error counts, not distinct
causes — but the cause list is genuinely shorter than 1.21.10's:
1.21.11 grew the gamerules, attribute and permissions APIs the oracle
was written against. What remains is the clock + blockscan gap, the
three ctor arities, and the override/symbol tail.

### B2-O02 — `VanillaRedstoneCatalogOracle` BROKEN, 10 errors (MAJOR; 1.21.7 O2 reproduces, smaller; 1.21.10 B2-O02 reproduces, much smaller — was 36)

My targeted compile: fails server (10 errors) and client (10 errors).
Causes, all javap-verified:

- **Cannot-find-symbol ×3:** `LevelSettings.DifficultySettings`
  (:183 — 1.21.11's `LevelSettings` has no `DifficultySettings` inner
  class; its ctor is 7-arg `(String, GameType, boolean, Difficulty,
  boolean, GameRules, WorldDataConfiguration)` and takes a plain
  `Difficulty`), `datapackWorldRegistries()` (:188 —
  `WorldLoader$DataLoadContext` in 1.21.11 still has only
  `resources()`, `dataConfiguration()`, `datapackWorldgen()`,
  `datapackDimensions()`), `LevelDataAndDimensions.WorldDataAndGenSettings`
  (:193 — the inner class is gone).
- **Constructor mismatches:** `PrimaryLevelData` (:191 — 1.21.11's
  public ctor is 4-arg `(LevelSettings, WorldOptions,
  SpecialWorldProperty, Lifecycle)`; the oracle calls 3-arg, missing
  `WorldOptions`), `MinecraftServer` (:212 — 1.21.11's ctor is 8-arg;
  the oracle calls 11-arg).
- **Cannot override ×3:** `tickServer` (:225), `onServerExit`
  (:456), `onServerCrash` (:457) — the oracle declares them
  `protected`; 1.21.11's are `public`, so the override weakens access.
- **No-override ×2:** :464 `getCommandSpamThresholdSeconds`, :465
  `getChatSpamThresholdSeconds`.
- GONE since 1.21.10 (all 26 errors of the difference): the missing
  packages `net.minecraft.gizmos` (21 entries in 1.21.11),
  `net.minecraft.server.permissions` (19 entries),
  `net.minecraft.world.level.gamerules` (10 entries) — all three now
  exist — plus `isEpollEnabled()` not overridden (1.21.11 renamed it
  `useNativeTransport`, matching the oracle) and the ~17 missing
  symbols (GameRules, PermissionSet, LevelBasedPermissionSet, Zombie,
  ClientLevel, Util, …).

1.21.7 reported "≈15 breaks" (distinct causes); 1.21.10 had 36 javac
errors. Same story as O01 in miniature: 1.21.11 grew most of the API
surface the oracle needs, and what remains is the LevelSettings /
WorldDataAndGenSettings / PrimaryLevelData shape plus the ctor and
visibility tail.

### B2-O03 — `VanillaRedstonePersistenceOracle` BROKEN, 1 error (MINOR; 1.21.7 O3 reproduces; 1.21.10 B2-O03 reproduces)

My compile agrees on both classpaths (1 error): '32: constructor
ServerLevel in class ServerLevel cannot be applied to given types;'.
Same single break as 1.21.7 and 1.21.10: the oracle calls the 26.3
10-arg ServerLevel ctor; 1.21.11's is 11-arg (see B2-O01). It passed
in 26.1.2 (whose ServerLevel ctor matches the oracle) — the break is
version-shape, not oracle rot.

### B2-O04 — `VanillaPistonPushOracle` FIXED: the `monster.zombie` package arrived (was MINOR; 1.21.7 O4 FIXED; 1.21.10 B2-O04 FIXED)

My compile: 0 errors on both classpaths. The oracle imports
`net.minecraft.world.entity.monster.zombie.Zombie` (the 26.x package);
1.21.11 HAS that package (jar-verified — 16 entries under
`net/minecraft/world/entity/monster/zombie/`, `Zombie.class` there),
where 1.21.10 and 1.21.7 had only `monster.Zombie`. The 2-error break
is gone with no oracle change — vanilla moved, and moved toward the
oracle. First E6 version where this oracle compiles.

### B2-O05 — `VanillaPistonMovementOracle` BROKEN, 1 error (MINOR; 1.21.7 O5 reproduces; 1.21.10 B2-O05 reproduces)

My compile agrees on both classpaths (1 error): '16: constructor
ServerLevel in class ServerLevel cannot be applied to given types;'.
Same single break as B2-O03 / 1.21.7 O5.

### B2-O06 — `VanillaFaceSupportOracle` BROKEN, 1 error (MAJOR; 1.21.10 B2-O06 reproduces; 26.1.2 B2-O03 reproduces)

My compile agrees on both classpaths (1 error): '45:
CubeVoxelShape(DiscreteVoxelShape) has protected access in
CubeVoxelShape'. Javap-verified: the ctor is `protected` in the
1.21.11 jar (`CubeVoxelShape` is byte-identical to 1.21.10's). Major
because it blocks the piston-shape cascade (B2-O07) and is referenced
by ~25 collision/support drivers. Not a 1.21.7 item (the 1.21.7 sweep
did not mint FaceSupport separately); carried from the 26.1.2 run.

### B2-O07 — `VanillaPistonShapeOracle` BROKEN by cascade, no own errors (MINOR; 1.21.10 B2-O07 reproduces; 26.1.2 B2-O04 reproduces)

My compile agrees (0 own errors on both classpaths). The dependency
is direct: `VanillaPistonShapeOracle.java:22` references
`VanillaFaceSupportOracle.class.getDeclaredMethod("emit", ...)`. Fix
FaceSupport and this one compiles. (Its other two references —
`VanillaCollisionOracle.main` and
`VanillaBlockStateCodecOracle.main` in the `export` branch — both
compile clean for 1.21.11, so FaceSupport is the only broken
dependency.)

### B2-O08 — `redstone_diff.py` defaults `--resources` to the 26.3 path (MINOR; 1.21.10 B2-O08 reproduces; 26.1.2 B2-O05 reproduces)

`tools/redstone_diff.py:453` defaults `--resources` to
`.local/horizon-client/26.3`. No prepared 1.21.11 data exists at that
path (and no 26.3 data exists in this worktree at all — no `.local`
at all), so the driver fails out of the box for 1.21.11.

### B2-O09 — `piston_step_e2e.py` defaults `--resources` to the 26.3 path (MINOR; 1.21.10 B2-O09 reproduces; 26.1.2 B2-O06 reproduces)

`tools/piston_step_e2e.py:31` defaults `--resources` to
`.local/horizon-client/26.3`. Same failure as B2-O08.

### Oracles assessed PASSING for 1.21.11 (my compiles, both classpaths)

- **`VanillaBlockPushOracle`**: 0 errors server, 0 errors client.
- **`VanillaClientRedstonePreviewOracle`**: FAIL server (8 errors —
  client-only imports, expected), **0 errors client**. Not an oracle
  defect: it is a client oracle and it compiles on its native
  classpath.
- **`VanillaPistonPushOracle`**: 0 errors on both classpaths — see
  B2-O04 (fixed).

### Adjacent oracle FIXED (out of Batch 2 scope — recorded, not minted)

- **`VanillaLevelShapeOracle`**: 0 errors on both classpaths for
  1.21.11. In 1.21.10 it failed with 2 errors (`world.attribute`
  missing); the package arrived in 1.21.11, so the oracle now
  compiles. Flagged for the coordinator / owning batch — a free fix
  from the vanilla side, like B2-O04.

### B2-H01 — Tick-by-tick circuit equivalence + piston corpus driver runs (needs-harness; 1.21.7 + 1.21.10 B2-H01 reproduce)

The 10 Batch 2 drivers (`test_redstone_parity`,
`test_redstone_catalog_parity`, `test_redstone_persistence_parity`,
`test_piston_push_parity`, `test_piston_movement_parity`,
`test_piston_shape_parity`, `test_face_support_parity`,
`test_client_redstone_preview_parity`, `test_block_push_parity`,
`block_update_harness.py`) cannot run for 1.21.11: four in-scope
oracles do not compile (B2-O01–B2-O03, B2-O05–B2-O06), one driver
constant is wrong (B2-G05), two drivers default to the 26.3 resources
path (B2-O08/B2-O09), and — independently — no prepared 1.21.11 data
exists, so a prepare run is needed first. The full per-state nibble
spot-check also waits on that prepare. (One driver fewer is blocked
than in 1.21.10: `test_piston_push_parity`'s oracle now compiles.)

### B2-H02 — The 4 level::tests need a cargo test run (needs-harness; 1.21.7 + 1.21.10 B2-H02 reproduce)

`crates/horizon-world/src/level/tests.rs` carries the same 4 tests at
the same lines as 1.21.10 (:1253, :1734, :1781, :2120). The `world!()`
macro silently skips without prepared data, and no prepared 1.21.11
data exists — a prepare run is needed before the run. Expected
outcomes (from reading the test bodies against the defects above):

| Test (`level/tests.rs`) | Verdict | Root cause |
| `redstone_lever_wire_and_lamp` (:1253) | **Expected PASS** — run owed (B2-H02) | Wire dispatches now (B2-G01 fixed); asserts power "15" at x=1, "11" at x=5, lamp lit then unlit |
| `levers_power_wire_through_interaction` (:1734) | **Expected PASS** — run owed (B2-H02) | Same (B2-G01); also exercises button release after 20 ticks (press time verified, §2.4) |
| `pistons_push_and_sticky_pistons_pull` (:1781) | Expected pass — run owed (B2-H02) | Dirt/redstone_block are NORMAL reaction → PushPull either way; push/pull logic verified |
| `pistons_pop_fragile_blocks_and_refuse_obsidian` (:2120) | **Expected FAIL** — run owed (B2-H02) | Torch (DESTROY) pushed, no DropResources event (B2-G02); obsidian pushed instead of refused (B2-G03) |

Same expectations as 1.21.10: the two wire tests flip to expected-pass
on the G7 fix (which originates here, in the template). Only the
pop/obsidian test is still expected to fail.

### Withdrawn candidates — CORRECTION: G11 and G12 NEVER EXISTED (land as closed items B2-G11/B2-G12)

#### G11 — Comparator conductor branch: NEVER EXISTED (not WITHDRAWN)

The 1.21.7 G11 premise was "vanilla takes the frame first" (the
lectern/item-frame analog signal wins over the block signal), with the
port taking the max. **1.21.11's own bytecode contradicts the premise
exactly as 1.21.7's and 1.21.10's did.** 1.21.11's
`ComparatorBlock.getInputSignal` computes
`Math.max(frameAnalogOrMIN, blockAnalogOrMIN)` at offset 150
(`b02/evidence/comparator.txt`, line 226), and the port takes the max
(`redstone.rs:544`: `frame.into_iter().chain(block).max()`, with the
comment "the larger signal winning"). The one 1.21.10→1.21.11 change
in the class is a dead null check removed from the
`lambda$getItemFrame$0` predicate — the max rule is untouched. The
port was always right and vanilla was always max — there was never a
defect to withdraw. **Coordinator: the 1.21.7 ledger correction for
G11 is still owed (flagged by the 1.21.10 run).**

#### G12 — Piston-destroyed blocks notify twice: NEVER EXISTED (not WITHDRAWN)

The 1.21.7 G12 premise was "vanilla destroys via
`Level.destroyBlock`, flags 3" (so the port's `setBlock(air, 18)` +
explicit neighbour updates double-notified). **1.21.11's own bytecode
contradicts the premise.** 1.21.11's `PistonBaseBlock.moveBlocks` uses
`setBlock(air, 18)` at offset 302 (`bipush 18`, line 828 of the
1.21.11 dump) and has NO `destroyBlock` call anywhere in the class
(grep count 0); `PistonBaseBlock` is normalized-identical to
1.21.10's, whose `moveBlocks` is identical to 1.21.7's. The port
matches (`pistons.rs:599`, flags 276/18/324/82 at
:564/:599/:612,:638/:658). There was never a defect to withdraw.
**Coordinator: the 1.21.7 ledger correction for G12 is still owed
(flagged by the 1.21.10 run).**

## §5 Re-check verdicts

### 1.21.10 re-checks (PRIMARY — all 21 decided)

| 1.21.10 item | Verdict for 1.21.11 | Evidence |
| B2-G01 wire class name (major, fixed) | **FIXED** → B2-G01 (fixed) | The alias mechanism originates in the template: `classes.rs:228` `RedstoneWireBlock \| RedStoneWireBlock`; census chain `['RedStoneWireBlock', 'Block']` recognised; all wire arms dispatch. Prepare half still broken → B2-G08 |
| B2-G02 push-reaction loader (major) | **REPRODUCES** → B2-G02 | Jar enum NORMAL/DESTROY/BLOCK/IGNORE/PUSH_ONLY (byte-identical class); facts carry those names (22,669/6,847/91/64 states); loader `data.rs:883-886` matches only 26.3 names |
| B2-G03 obsidian refusal (major) | **REPRODUCES** → B2-G03 | Jar `isPushable` offsets 44–85 list the four blocks; port has no such check; `named` lacks crying_obsidian/reinforced_deepslate |
| B2-G04 Y-bound off by one (minor) | **REPRODUCES** → B2-G04 | Jar allows y == maxY, refuses UP-push only from maxY; port uses `max_y() - 1` (319) |
| B2-G05 shape-driver count (minor) | **REPRODUCES** → B2-G05 | `test_piston_shape_parity.py:63` asserts 35723; 1.21.11 has 29,671 states |
| B2-G06 0.01 height divergence (minor) | **REPRODUCES** → B2-G06 | Cobweb driver docstring documents it; height deliberately uncompared |
| B2-G07 diode same-block guard (minor) | **REPRODUCES** → B2-G07 | Jar `neighborChanged` has no guard (offset 0 → canSurvive); port `redstone.rs:673-675` returns early |
| B2-G08 prepare-side survival (minor) | **REPRODUCES** → B2-G08 | `survival.rs:45` still `RedstoneWireBlock`; `bind_definitions` binds no rule on miss |
| B2-H01 driver runs (minor) | **REPRODUCES** → B2-H01 | Oracles still broken (O01–O03, O05–O06), driver constant still wrong (G05); prepared data NOT present for 1.21.11 — a prepare run is also needed. One driver fewer blocked than in 1.21.10 (PistonPush compiles now) |
| B2-H02 level::tests run (minor) | **REPRODUCES** → B2-H02 | 4 tests at the same lines; 3 of 4 expected to PASS (wire tests flip on the G7 fix, which originates here); only the pop/obsidian test expected to fail; a 1.21.11 prepare is needed before the run |
| B2-O01 BlockUpdate oracle (major) | **REPRODUCES, smaller** → B2-O01 | 38 errors (my compile, both classpaths) vs 63 in 1.21.10; gamerules/attribute/permissions packages arrived, clock + blockscan still missing |
| B2-O02 RedstoneCatalog oracle (major) | **REPRODUCES, much smaller** → B2-O02 | 10 errors vs 36 in 1.21.10; gizmos/permissions/gamerules packages arrived |
| B2-O03 RedstonePersistence oracle (minor) | **REPRODUCES** → B2-O03 | 1 error, ServerLevel ctor — same single break as 1.21.10 |
| B2-O04 PistonPush oracle (minor) | **FIXED** → B2-O04 (fixed) | 0 errors on both classpaths: 1.21.11 HAS the `monster.zombie` package (16 entries) the oracle imports |
| B2-O05 PistonMovement oracle (minor) | **REPRODUCES** → B2-O05 | 1 error, ServerLevel ctor — same single break as 1.21.10 |
| B2-O06 FaceSupport oracle (major) | **REPRODUCES** → B2-O06 | 1 error, protected `CubeVoxelShape` ctor — identical to 1.21.10 (`CubeVoxelShape` byte-identical) |
| B2-O07 PistonShape cascade (minor) | **REPRODUCES** → B2-O07 | 0 own errors; depends on broken `VanillaFaceSupportOracle` (`:22`); its other two deps compile clean now |
| B2-O08 redstone_diff 26.3 default (minor) | **REPRODUCES** → B2-O08 | `redstone_diff.py:453` defaults `--resources` to the 26.3 path |
| B2-O09 piston_step_e2e 26.3 default (minor) | **REPRODUCES** → B2-O09 | `piston_step_e2e.py:31` defaults `--resources` to the 26.3 path |
| B2-G11 G11 comparator (minor, closed) | **CLOSED again** → B2-G11 | NEVER EXISTED — 1.21.11's own `getInputSignal` takes `Math.max` (offset 150); port takes max |
| B2-G12 G12 destroy notify (minor, closed) | **CLOSED again** → B2-G12 | NEVER EXISTED — 1.21.11's own `moveBlocks` uses `setBlock(air, 18)` (offset 302), no `destroyBlock` anywhere; port matches |

### 1.21.7 re-checks (SECONDARY — all 13 decided)

| 1.21.7 item | Verdict for 1.21.11 | Evidence |
| G7 wire class name (major) | **FIXED** → B2-G01 (fixed) | Same as B2-G01 above: the alias mechanism originates in this template |
| G8 push-reaction loader (major) | **REPRODUCES** → B2-G02 | Same loader, same 26.3-only match (`data.rs:883-886`) |
| G9 obsidian refusal (major) | **REPRODUCES** → B2-G03 | Same missing hardcode (`pistons.rs:328-372`) |
| G10 Y-bound off by one (minor) | **REPRODUCES** → B2-G04 | Same `max_y() - 1` (`pistons.rs:337`) |
| G11 comparator conductor (minor) | **NEVER EXISTED** → B2-G11 (closed) | 1.21.11's own `getInputSignal` takes `Math.max(frame, block)` (offset 150); port takes max |
| G12 destroy notify twice (minor) | **NEVER EXISTED** → B2-G12 (closed) | 1.21.11's own `moveBlocks` uses `setBlock(air, 18)` (offset 302), no `destroyBlock` anywhere; port matches |
| G13 shape-driver count (minor) | **REPRODUCES** → B2-G05 | Same 35723 assert; 1.21.11 has 29,671 states |
| G14 0.01 height divergence (minor) | **REPRODUCES** → B2-G06 | Same docstring, same deliberate non-comparison |
| O1 BlockUpdate oracle (major) | **REPRODUCES, smaller** → B2-O01 | 38 errors (my compile); 1.21.7 had ≈25 distinct causes, 1.21.10 had 63 errors |
| O2 RedstoneCatalog oracle (major) | **REPRODUCES, smaller** → B2-O02 | 10 errors (my compile); 1.21.7 had ≈15 distinct causes, 1.21.10 had 36 errors |
| O3 RedstonePersistence oracle (minor) | **REPRODUCES** → B2-O03 | 1 error, ServerLevel ctor — same single break as 1.21.7 |
| O4 PistonPush oracle (minor) | **FIXED** → B2-O04 (fixed) | 0 errors: 1.21.11 has the `monster.zombie` package; the 1.21.7 break (1 error, zombie package) is gone |
| O5 PistonMovement oracle (minor) | **REPRODUCES** → B2-O05 | 1 error, ServerLevel ctor — same single break as 1.21.7 |

## §6 Systemic findings

### The template fixed the wire at the source and left the rest

The 1.21.10 run could only say the child "carries the 1.21.11-template
class-alias mechanism"; this run verifies the mechanism in the template
itself (`classes.rs:228`, same line, same code — the level files are
byte-identical or comment-only between the two trees). The template's
alias mechanism is the single biggest behaviour difference from the
26.1.2 tree: wire, farmland, snow, fungus, roots and path all dispatch
through aliases that the 26.1.2 tree lacks entirely. But the alias
mechanism is a table-layer fix, and the deeper behaviour defects
survive untouched in the template: the push-reaction loader still
matches only 26.3 names, `is_pushable` still lacks the obsidian
hardcode and still computes the ceiling off by one, and the diode
guard still carries the 26.2 form. The template fixed the dispatch
layer; the logic layer still carries the 26.3 form. The 26.2 tree is
the natural reference for the G8/G9/G10 repairs (it carries all three
fixes, per the 26.1.2 report).

### Vanilla moved toward the oracles in 1.21.11

The oracle story of this run is a version-shape story in the other
direction from 1.21.10's. The oracles were written against 26.3 and
adapted incrementally down the 26.x line; 1.21.10 lacked the whole
26.x API surface (63 / 36 errors). 1.21.11 grew most of it back: the
`gamerules` package (GameRules moved there, data-defined), the
`permissions` package, the `attribute` package (environment
attributes), the `gizmos` package, the `monster.zombie` subpackage,
`useNativeTransport` (renamed from `isEpollEnabled`),
`RandomSequences`' public no-arg ctor, and `Identifier` (with the
oracle sources adapted to it). Result: BlockUpdate 63 → 38, Catalog
36 → 10, PistonPush 2 → 0 (fixed), LevelShape 2 → 0 (fixed, adjacent).
What remains is a smaller, well-defined adaptation job: the clock and
blockscan packages (still absent), the three ctor arities (ServerLevel
11-arg, MinecraftServer 8-arg, ServerChunkCache 11-arg), the
LevelSettings / WorldDataAndGenSettings / PrimaryLevelData shape, the
`tickServer`/`onServerExit`/`onServerCrash` visibility (public in
1.21.11, protected in the oracle), and the CubeVoxelShape factory.
The three single-break oracles (O03/O05) plus the fixed O04 show the
shape of the win: the E6 oracle-adaptation pass is now a small job,
not the rewrite 1.21.10's numbers suggested.

### The daylight detector: vanilla changed, and the port already matches the new form

The one real behaviour-form change in scope between 1.21.10 and
1.21.11 is `DaylightDetectorBlock.updateSignalStrength`: vanilla
1.21.10 called `Level.getSunAngle(1.0F)` and `Mth.cos(float)`;
1.21.11 reads `EnvironmentAttributes.SUN_ANGLE` from the level's
environment-attribute system (degrees, × 0.017453292) and calls
`Mth.cos(double)`. The port already matches the NEW form:
`daylight_detector_update_signal` (`redstone.rs:1101-1120`) takes
`sun_angle * DEG_TO_RAD` and `trig::cos(f64)`, and the real
`sun_angle` impl (`horizon-client/src/game.rs:1627`) samples the
dimension's `Environment` at the world's day time — the port's
environment-attribute system, the same source vanilla 1.21.11 reads.
The check passes, and it passes because the template was written
against 1.21.11's API, not 1.21.10's. (In 1.21.10 the same port code
passed against the old form on float-precision grounds; in 1.21.11
the match is exact in form as well as value.)

### The vanilla-check rule mattered in both directions again

G11 and G12 are the sharpest instance yet of why the rule exists: the
1.21.7 sweep's premises about vanilla 1.21.7 were wrong in both cases,
and the error propagated through the 26.x runs as "vanilla changed"
framing. Checking the target version's own bytecode — not the next
version's, not the base's — is what catches it, and this run confirms
it a third time against 1.21.11's own jar. The diode guard (B2-G07)
is the same rule in the opposite direction: a real defect the 1.21.7
sweep missed, because the 1.21.7 tree's `redstone.rs` carries the
26.2-form guard and the 1.21.7 sweep never compared
`DiodeBlock.neighborChanged` against 1.21.7's own jar. **Coordinator:
the 1.21.7 ledger owes three corrections — G11 and G12 to NEVER
EXISTED, and a new entry for the diode guard** (flagged by the 1.21.10
run, still owed).

### Programme-wide 35723 hardcode

The `test_piston_shape_parity.py:63` assert is one instance of a
programme-wide pattern: many drivers hardcode the 26.3 state count
35723 (`test_block_landing_parity.py`,
`test_block_properties_parity.py`, `test_collision_parity.py`, …).
Every E6 version (29,671 states for 1.21.11) trips every one of them.
Flagged for the coordinator — a single shared constant (or a
datagen-derived count) would fix the whole family.

### Out-of-scope flags for the coordinator

- **`hooks.rs:549` reads `b.tags.blocks_motion`** — the 26.3
  motion-tag read; the 1.21.7 tree used a per-state `blocks_motion()`
  rule. B3-G22 class; belongs to the motion-tag batch.
- **`amethyst_block` falls back to plain Block** despite a non-Block
  head (`AmethystBlock`) — the only such block in the 1.21.11
  class-table census (278 known java names incl. aliases). Out of
  Batch 2 scope; belongs to the block/class batch. (72 blocks have an
  unrecognised head but a recognised ancestor — leaves, stained
  glass, furnaces, WeatheringCopper variants — behaviour largely
  carried by the ancestor.)
- **`VanillaLevelShapeOracle`** compiles clean for 1.21.11 (0 errors
  both classpaths; was 2 errors in 1.21.10) — adjacent oracle, fixed
  by vanilla's `world.attribute` arrival; flagged for the owning
  batch.
- **jsr305 is missing from the 1.21.11 tree's `tools/`** (present in
  the 1.20.5/1.21.10 trees). The oracle compiles in this batch used
  the 1.20.5 copy (sha1-identical). Worth a tree-health note: a fresh
  checkout's oracle builds would miss it.

## §7 Evidence index

| Evidence | Location |
| `RedStoneWireBlock` only; no `RedstoneWireBlock` | jar listing, `server-game-named.jar` (unzip -l grep) |
| `PushReaction` enum NORMAL/DESTROY/BLOCK/IGNORE/PUSH_ONLY | `b02/evidence/pushreaction.txt` (javap on 1.21.11 server-game-named.jar) |
| `isPushable` obsidian refusal (offsets 44–85) + Y bounds (0–33, 106–125) + reaction switch | `b02/evidence/pistonbase.txt` (lines 580–697) |
| `moveBlocks` destroy `bipush 18` (offset 302) + no `destroyBlock` in class (grep count 0) | `b02/evidence/pistonbase.txt` (line 828) |
| `ComparatorBlock.getInputSignal` `Math.max` (offset 150) | `b02/evidence/comparator.txt` (line 226) |
| `DiodeBlock.neighborChanged` no guard (offsets 0–94) | `b02/evidence/diode.txt` |
| `ServerLevel` 11-arg / `MinecraftServer` 8-arg + abstract `useNativeTransport` / `ServerChunkCache` 11-arg / `CubeVoxelShape` protected ctor / `DataLoadContext` (no `datapackWorldRegistries`) / `LevelSettings` (no `DifficultySettings`, 7-arg ctor) / `PrimaryLevelData` 4-arg ctor / `GameRules` data-defined constants + `get(GameRule)` / `RandomSequences` no-arg ctor + `get(Identifier, long)` / `EnvironmentAttributeSystem$Builder.addTimelineLayer(Holder, LongSupplier)` / `DimensionType.attributes()/timelines()` | `b02/evidence/serverlevel.txt`, `minecraftserver.txt`, `serverchunkcache.txt`, `cubevoxelshape.txt`, `dataloadcontext.txt`, `levelsettings.txt`, `primaryleveldata.txt`, `gamerules.txt`, `randomsequences.txt`, `envattr.txt`, `envattr_builder.txt`, `dimensiontype.txt` |
| `Zombie` at `monster/zombie/Zombie` (16 entries); `gizmos` 21, `world.attribute` 55, `server.permissions` 19, `world.level.gamerules` 10 entries; `world.clock` 0, `world.level.blockscan` 0; `GameRules` at `world.level.gamerules`; `Identifier` only, no `ResourceLocation` | jar listing (unzip -l grep) |
| Full-jar class comparison 1.21.10 ↔ 1.21.11 (520 / 719 / 3,131; 12 in-scope byte-identical, 29 changed) | `b02/evidence/class_compare_1_21_10_vs_1_21_11.json` |
| Normalized javap diff of the 29 changed in-scope classes (22 identical, 7 differs — all reviewed) | `b02/evidence/normdiff.py`, `b02/evidence/normdiff_*.txt`, `b02/evidence/javap_changed_1_21_11.txt`, `b02/evidence/javap_all_1_21_10.txt` (incl. `javap_missing_1_21_10.txt` — the first 1.21.10 dump lost 9 classes to truncated javap output and was re-dumped in full) |
| Census: 1.21.11 blocks/states by reaction; class-table census (278 known, amethyst only fallback, 72 ancestor-only) | `b02/evidence/blocks_1_21_11.json`, `b02/evidence/census_1_21_11.log`, `b02/evidence/census_1_21_11.py`, `b02/evidence/extract_blocks.py`, `b02/evidence/chains_1_21_11.json` |
| `Blocks` clinit dump | `b02/evidence/blocks_1_21_11.txt` (44,971 lines) |
| Oracle compiles, 15-file closure (server + client classpaths) | `b02/evidence/javac/server_compile.log` (59 errors), `b02/evidence/javac/client_compile.log` (51 errors), `b02/evidence/javac/{server,client}_cp.txt` |
| `classes.rs:228` wire alias; `data.rs:883-886` loader; `data.rs:876-877` nibble columns; `data.rs:1012` press times; named `obsidian` :270, `respawn_anchor` :372 | `versions/1.21.11/crates/horizon-world/src/level/` |
| `pistons.rs:337` `max_y() - 1`; no obsidian check; flags 276/18/324/82 at :564/:599/:612,:638/:658 | `versions/1.21.11/crates/horizon-world/src/level/pistons.rs` |
| `redstone.rs:673-675` diode guard; `redstone.rs:544` comparator max; `redstone.rs:414-433` wire power; `redstone.rs:1101-1120` daylight detector (DEG_TO_RAD + cos f64) | `versions/1.21.11/crates/horizon-world/src/level/redstone.rs` |
| `sun_angle` real impl samples the dimension `Environment` | `versions/1.21.11/crates/horizon-client/src/game.rs:1627` |
| `survival.rs:45` `RedstoneWireBlock`; `bind_definitions` :102–117 | `versions/1.21.11/crates/horizon-prepare/src/survival.rs` |
| Exporter row layout (weak/strong cols 5/6); `"push"` = vanilla enum name | `VanillaBlockBehaviourExporter.java:198-220`, `VanillaGameplayFactsExporter.java:50` |
| Press times stone 20 / wood 30 | 1.21.11 datagen `blocks.json` definitions + 1.21.7 export `fields` |
| 4 level::tests at :1253/:1734/:1781/:2120 | `versions/1.21.11/crates/horizon-world/src/level/tests.rs` |
| `test_piston_shape_parity.py:63` (35723); cobweb docstring (0.01); `redstone_diff.py:453`; `piston_step_e2e.py:31` | `versions/1.21.11/tools/` |

## §8 Verdict

**53 checks: 46 pass, 7 gap.** The wire works — and this run proves
the fix originates in the template itself, not in the child. Two
1.21.7 majors still reproduce (push-reaction loader, obsidian
refusal), the ceiling bound is still off by one, the shape driver
still cannot run, the 0.01 height divergence still stands, the diode
guard still carries the 26.2 form, and the wire's prepare-side
survival rule never binds. The oracle picture improves sharply on
vanilla's side — BlockUpdate 63 → 38 errors, Catalog 36 → 10, and
the piston-push oracle compiles clean for the first time in the E6
lineage — but four oracles are still broken, the driver runs +
level::tests still need a prepare run first (no `.local` exists for
1.21.11), and two 1.21.7 items never existed — the ledger correction
is owed to the coordinator, not to this batch.
