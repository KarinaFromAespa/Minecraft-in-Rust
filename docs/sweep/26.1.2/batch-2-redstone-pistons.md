# Batch 2 — Redstone + pistons (26.1.2 parity sweep)

**Version:** 26.1.2 · **Date:** 2026-11-25 · **Branch:** `parity-sweep-26.1.2`
(workdir `~/workspace/mcir-work/sweep-26.1.2`) ·
**Scope:** `docs/PARITY-SWEEP.md` §4 Batch 2 — redstone ticking, pistons,
observers, comparators; 10 drivers + circuit probes; pilot leftovers
(4 level::tests).

## §0 Execution status

Checks + triage only. No file under `versions/26.1.2/` was modified,
nothing was committed, no git commands ran in the worktree, no cargo
builds ran. Everything below is static analysis: `javap -c` on the named
26.1.2 server-game jar
(`~/workspace/mcir-work/26.1.x/versions/26.1.2/jars/server-game.jar`,
sha1 `83eb1106f12f24c9127ce22070d71853157af127`, verified; the bundler
`server.jar`, sha1 `97ccd4c0ed3f81bbb7bfacddd1090b0c56f9bc51`, was never
javap'd; `client.jar` sha1 `4e618f09a0c649dde3fdf829df443ce0b8831e65`,
verified), source comparison against `versions/26.1.2/crates/`, the
26.1.2 datagen reports
(`~/workspace/mcir-work/26.1.x/versions/26.1.2/datagen/reports/`), the
26.1.1 prepared data as the proven-identical export of this jar lineage
(see §2.4 — no prepared 26.1.2 data exists in the worktree), and Batch
0's 26.1.2 oracle measurement
(`versions/26.1.2/.local/measure-oracles/results.json` + the
big-compile logs, 391 oracles, 189 pass on the client classpath /
202 fail-all — see the note on the landed table below).

**Headline:** all seventeen 26.1.1 items REPRODUCE in 26.1.2 with no
movement at all — no fix, no new defect, no withdrawal change. The
26.1.2 tree is a faithful delta port of the 26.1.1 tree in this scope:
`crates/horizon-world/src/level/` is byte-identical to 26.1.1's,
`tools/` differs only by `__pycache__` + `jsr305-3.0.2.jar`, and the
oracle sources are byte-identical. The 26.1.1→26.1.2 jar delta is
exactly 3 server classes (none in scope) and 8 client classes (GUI +
version plumbing only). The wire is still dead (G01), every state still
loads PushPull (G02), obsidian still pushes (G03), the ceiling bound is
still off by one (G04), the shape driver still cannot run (G05), the
0.01 height divergence still stands (G06), and diodes still wrongly
skip `neighborChanged` when already replaced (G07). The three 1.21.7
persistence/piston oracles stay fixed; the block-update, catalog and
face-support oracles stay broken with identical error counts.

**One delta vs 26.1.1 (precondition, not a defect):** 26.1.2 has NO
prepared horizon-client data in the worktree
(`versions/26.1.2/.local/` holds `minecraft/` + `measure-oracles/`
only). The 26.1.1 run could point at `.local/horizon-client/26.1.1/`;
for 26.1.2 the driver runs (B2-H01) and the level::tests (B2-H02) need
a prepare run first, on top of the oracle repairs. The census/nibble
checks in this report were still run as real checks, against the
26.1.1 export via the proven identity chain (§2.4).

**Note on the Batch 0 table:** the landed
`docs/sweep/26.1.2/batch-0-oracles.json` had NOT appeared at handoff
(checked 00:50 NPT Nov 25; `docs/sweep/26.1.2/` holds only the Batch 1
report, the inventory extractor and the inventory). Triage here uses
Batch 0's raw 26.1.2 measurement directly —
`versions/26.1.2/.local/measure-oracles/results.json` (391 files, all
three classpaths) plus the full 32-error list from
`big-compile-server.log` — which is the measurement the landed table
will be built from, not merely the 26.1.1 table. Every failing
in-scope oracle's causes were additionally javap-verified fresh
against the 26.1.2 jar. If the landed table disagrees with results.json
on any in-scope oracle, the landed table wins and this batch's O-items
need a re-read — but the counts here (32 / 4 / 1 / cascade, O3/O4/O5
pass) reproduce 26.1.1's exactly, as the identical oracle sources and
byte-identical jar classes predict.

## §1 Scope

Redstone wire power/updates, comparators, repeaters, observers, pistons
(push/pull/destroy, `is_pushable`, shape driver), block-update ordering,
redstone persistence. Method: `javap -c` on the named 26.1.2
server-game jar + source comparison against `versions/26.1.2/crates/` +
tag/data reads. No cargo builds, no driver runs. The 4 level::tests and
the circuit-equivalence / signal-nibble checks that need prepared data
or a test run are needs-harness items (B2-Hxx), not faked passes —
except that the nibble/press-time checks were run here as real checks
against the proven-identical prepared data, as in the 26.1.1 run.

Out of scope for this batch (noted, not verified): the 26.1.2 delta's
two behaviour items — `ServerGamePacketListenerImpl.handleAttack`
(spectator guard, ported in `lan/hub.rs` + `lan_actions.rs`) and the
Checkbox row-limit tooltip (`dialog_screen.rs`) — are combat and dialog
screen, not redstone/pistons; the twelve 26.1 regressions the brief
names for re-check-first (B1-G05, B3-G14, B4-G01, B4-G02, B7-G07,
B7-G08, B9-G03/G05/G07b/G12/G13) belong to their owning batches, as
does the B3-G22 motion-tag startup blocker.

## §2 Method

Four verification layers, all fresh against 26.1.2 evidence:

1. **Jar identity proof.** A full sha1 comparison of ALL classes
   between the 26.1.1 and 26.1.2 server-game jars (7,351 classes each,
   no additions/removals): exactly **3 changed classes** —
   `net.minecraft.DetectedVersion`, `net.minecraft.SharedConstants`,
   `net.minecraft.server.network.ServerGamePacketListenerImpl` — none
   in Batch 2 scope. The client jars (10,682 classes each) differ in
   exactly **8 classes** — those 3 plus `Checkbox`,
   `Checkbox$Builder`, `Checkbox$OnValueChange`,
   `AbstractReportScreen`, `AbstractReportScreen$DiscardReportWarningScreen`
   — GUI + version plumbing only, none in scope. This matches the
   documented 26.1.1→26.1.2 delta (PORT-NOTES-26.1.2.md: 6 changed
   classes — 3 server-side, 3 client-side, counting the two nested
   classes separately in the jar listing). All 41 in-scope classes
   were checked individually and are byte-identical, including
   `RedStoneWireBlock`, `RedstoneWireEvaluator`,
   `DefaultRedstoneWireEvaluator`, `DiodeBlock`, `ComparatorBlock`,
   `RepeaterBlock`, `ObserverBlock`, `RedstoneTorchBlock`,
   `RedstoneLampBlock`, `LeverBlock`, `ButtonBlock`, `TargetBlock`,
   `CopperBulbBlock`, `DaylightDetectorBlock`, `DispenserBlock`,
   `HopperBlock`, `PistonBaseBlock`, `PistonStructureResolver`,
   `PistonMovingBlockEntity`, `MovingPistonBlock`, `PistonHeadBlock`,
   `PushReaction`, `SignalGetter`, `NeighborUpdater` (at
   `net.minecraft.world.level.redstone.NeighborUpdater` — the 26.1.1
   report's `block/` path was wrong; the class lives in `redstone/`),
   `DetectorRailBlock`, `PoweredRailBlock`, `TripWireHookBlock`,
   `TripWireBlock`, `SculkSensorBlock`, `NoteBlock`, `DropperBlock`,
   `CommandBlock`, `JukeboxBlock`, `LecternBlock`,
   `CalibratedSculkSensorBlock` and the pressure-plate classes.
   Every jar-side verification from the 26.1.1 run therefore carries
   to 26.1.2 by identity, not by assumption.
2. **Fresh javap on the 26.1.2 jar** for every defect-bearing method:
   `PistonBaseBlock.isPushable` (obsidian refusal + Y bounds),
   `moveBlocks` (destroy flags), `ComparatorBlock.getInputSignal`
   (max rule), `DiodeBlock.neighborChanged` (no guard), the
   `PushReaction` enum, the `RedStoneWireBlock` class listing, and
   every missing symbol behind the oracle defects (blockscan,
   `Continuation`, `ClockInstance`, spam thresholds, the two ctor
   arities, `datapackWorldRegistries`, the `CubeVoxelShape` ctor).
   Dumps in `evidence/` (§7). All offsets match the 26.1.1 dumps
   exactly, as byte-identical classes must.
3. **Tree diff.** A full diff of `crates/horizon-world/src/level/`
   between the 26.1.1 and 26.1.2 trees in this worktree: **byte-identical,
   zero differences** (`diff -rq` exit 0). `tools/` differs only by
   `__pycache__` and `jsr305-3.0.2.jar` (no source difference). The
   parity oracle sources are byte-identical between the trees. The 45
   pass checks rest on (1)+(2)+(3): the jar methods are the ones the
   26.1.1 run verified, and the port code deciding each check is
   unchanged from the 26.1.1 tree — there is no in-scope difference
   at all this round, unlike 26.1→26.1.1 (which had the diode guard).
4. **Fresh data checks.** The push-reaction census, the class-table
   fallback census, the nibble spot-checks and the button press-time
   checks were re-run (`evidence/census_nibbles_26_1_2.py`, log in
   `evidence/`) against the 26.1.2 datagen reports (byte-identical to
   26.1.1's — verified sha1-equal for blocks, commands, packets and
   registries.json; 1,168 blocks, 29,873 states), the 26.1.2 tree's
   `classes.rs`, and the 26.1.1 prepared export. The prepared export
   is the 26.1.1 one because no 26.1.2 export exists (§0) — it is
   provably the 26.1.2 export by the identity chain in (1): the
   export derives from block classes + datagen, both byte-identical
   between the versions. The census output is identical to the
   26.1.1 log except for four added lines (see the method note in §6
   on the corrected fallback count).

**Counting rule:** one check = one vanilla-method/behaviour comparison
(jar + port source, or jar + prepared data). Wire-logic and
reaction-switch checks count the Rust functions that exist — they match
vanilla method-for-method — while the dispatch/table failures that make
them unreachable are counted once each as the G01/G02 gap checks (the
1.21.7 report's approach, not a pass for dead code). Oracle/driver-path
checks are not in the behaviour tally, except the two driver-constant
checks (G05/G06), counted as in the 26.1.1 template.

**Note on the 26.1.1 summary:** the common brief points at a
"26.1-record corrections" section in the 26.1.1 `summary.md`; it exists
and contains NO Batch 2 correction (B3-G33, B8, B8-G10, B10-G15,
B3-G34 only), and the coordinator corrections section records
"B0/B1/B2/…: no corrections needed". The 26.1.1 template report is the
authority used here, with one script correction of my own recorded in
§6 (the fallback census).

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
  and G12 do not apply to 26.1.2; vanilla changed, the port matches
  26.1.2's form).
- **1.21.7 majors: all three REPRODUCE in 26.1.2** (G7, G8, G9 → B2-G01,
  B2-G02, B2-G03). Like the 26.1.1 tree, the 26.1.2 tree does NOT carry
  the 1.21.11-template fixes: its `classes.rs` has no alias mechanism
  and no `RedStoneWireBlock` spelling, its push-reaction loader matches
  only the 26.3 enum names, and its `is_pushable` has no obsidian-family
  check. The 26.2 tree carries all three fixes; the 26.1.2 tree carries
  none of them.
- **NEW vs 26.1.1: none.** The 26.1.1→26.1.2 delta touches nothing in
  scope (jar delta: version plumbing + handleAttack + Checkbox;
  tree delta in scope: zero files), so the tally is identical to
  26.1.1's (52 / 45 / 7) and every item reproduces with the same id.

Per-section counts sum to the tally: wire 9 (8 pass + G01), machinery 4
(3 pass + G07), consumers 16, nibbles/data 3 (2 pass + G02), observers
5, pistons 13 (11 pass + G03 + G04), driver constants 2 (G05 + G06)
= 52; 45 + 7 = 52.

## §4 Defects

### B2-G01 — Redstone wire completely non-functional: class-name mismatch (MAJOR; 26.1.1 B2-G01 reproduces; 1.21.7 G7 reproduces)

**Check failed (1):** wire class dispatch.

Vanilla 26.1.2 names the class `RedStoneWireBlock` (capital S;
jar-verified — `net/minecraft/world/level/block/RedStoneWireBlock.class`
exists, `RedstoneWireBlock` does not; the class file is byte-identical
to 26.1.1's). The prepared data is faithful:
`block-behaviours.json` carries redstone_wire's class chain as
`["RedStoneWireBlock", "Block"]`. But the port's class table
(`level/classes.rs:224`, byte-identical to 26.1.1's) knows only
`"RedstoneWireBlock"`, and — unlike the 26.2 tree — the 26.1.2 tree has
no alias mechanism at all (no `|` alternatives in the `classes!` macro,
no unit test). `Class::from_java` returns `None` for the real name, so
`data.rs:920-928` drops it from the chain and the wire resolves to
`Class::Block`. Every wire arm exists but never runs: `hooks.rs:100`
(neighborChanged → `wire_update_power`), `hooks.rs:211` (onPlace),
`hooks.rs:380` (affectNeighborsAfterRemoval), `shapes.rs:698`
(updateShape), `redstone.rs:65,101` (signal), `interact.rs:263`
(dot/cross toggle), `place.rs:1089`, `survive.rs:281`. Wire power stays
0, wire emits no signal — exactly the 1.21.7/26.1/26.1.1 failure,
including the pilot wire-test failures it explains (see §6). The wire
logic itself matches vanilla 26.1.2 method-for-method (see Passes); the
defect is the one-line table miss. The prepare-side half is unfixed too:
`crates/horizon-prepare/src/survival.rs:43` still reads
`("RedstoneWireBlock", "RedstoneWire")` (horizon-prepare is
byte-identical to 26.1.1's in this file — verified by grep).

A full class-table census (all 1,168 exported blocks vs the 271 known
classes, `evidence/census_nibbles_26_1_2.py`) finds exactly three blocks
resolving to plain `Block` despite a non-Block most-derived class:
`redstone_wire` (this item), `dirt_path` (`DirtPathBlock`) and
`amethyst_block` (`AmethystBlock`) — identical to 26.1.1. The latter two
are out of Batch 2 scope — flagged for the coordinator in §6. (87 blocks
have an unrecognised most-derived class but a recognised ancestor, also
identical to 26.1.1.) Note the census script now computes this count
correctly; see the method note in §6.

### B2-G02 — Push-reaction loader mapping matches no 26.1.2 value: every state loads as PushPull (MAJOR; 26.1.1 B2-G02 reproduces; 1.21.7 G8 reproduces)

**Check failed (1):** push-reaction loader mapping.

Vanilla 26.1.2's `PushReaction` enum is `NORMAL, DESTROY, BLOCK, IGNORE,
PUSH_ONLY` (jar-verified fresh — `evidence/pushreaction.txt`; the
pre-26.3 names, byte-identical class to 26.1.1's). The prepared
`gameplay-facts.json` carries exactly those names: NORMAL 22,869 states,
DESTROY 6,849, BLOCK 91, PUSH_ONLY 64 (no IGNORE states exist in
26.1.2) — identical counts to 26.1.1. But the loader
(`level/data.rs:886-890`) matches only the 26.3 names — `"PUSH"`,
`"POPPED"`, `"IMMOVEABLE"`, `"IGNORE_ENTITY"` — and defaults everything
else to `PushPull`. All 29,873 states load as `PushPull`. Consequences,
from the fresh census (`evidence/census_nibbles_26_1_2.py`, state→block
map from the 26.1.2 datagen `blocks.json`):

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
value. The 26.2 tree's loader matches both name sets; the 26.1.2 tree's
does not.

### B2-G03 — `is_pushable` lacks vanilla's hardcoded obsidian-family refusal (MAJOR; 26.1.1 B2-G03 reproduces; 1.21.7 G9 reproduces)

**Check failed (1):** obsidian-family refusal in `is_pushable`.

Vanilla 26.1.2 `PistonBaseBlock.isPushable`
(`net.minecraft.world.level.block.piston.PistonBaseBlock`, jar-verified
fresh — `evidence/pistonbase.txt`, bytecode offsets 44–85) refuses
`OBSIDIAN`, `CRYING_OBSIDIAN`, `RESPAWN_ANCHOR` and
`REINFORCED_DEEPSLATE` explicitly before the reaction switch. All four
are NORMAL reaction with hardness ≠ −1 (obsidian hardness 50.0), so
nothing else in the check stops them. The port (`level/pistons.rs:328-372`,
byte-identical to 26.1.1's) has no such check — `grep obsidian
pistons.rs` returns nothing — and the 26.2 tree's obsidian block
(`pistons.rs:344-355` there, with the comment noting 26.3 gave them
IMMOVEABLE instead) is absent from the 26.1.2 tree, as are the
`crying_obsidian` / `reinforced_deepslate` named entries in `data.rs`
(the table has `obsidian` at :267 and `respawn_anchor` at :369, but
neither of the other two). A piston extends and pushes obsidian instead
of staying retracted. Explains the obsidian half of the pilot piston
test.

### B2-G04 — `is_pushable` Y-bound off by one at the ceiling (MINOR; 26.1.1 B2-G04 reproduces; 1.21.7 G10 survives)

**Check failed (1):** pushing upward at the build ceiling.

Vanilla 26.1.2 `isPushable` bounds-checks `y < getMinY() || y >
getMaxY()` — y == maxY (320) is allowed (`if_icmplt`/`if_icmpgt` at
offsets 8/19) — and refuses only an UP-push *from* y == maxY (bytecode
offsets 0–34 and 106–125, fresh javap in `evidence/pistonbase.txt`;
overworld min_y −64, height 384, so maxY = 320). The port
(`level/pistons.rs:337`, byte-identical to 26.1.1's) uses
`max_y = self.level.max_y() - 1` (319; `LevelAccess::max_y()` default is
320, `level/mod.rs:606`): it refuses y == 320 outright and refuses
UP-pushes from y == 319. A piston at y == 319 pushing up fails in the
port (the y == 320 air cell reads unpushable) where vanilla extends.
The floor side matches vanilla exactly (`min_y` −64, Down-push refused
only from y == min_y, `pistons.rs:344-346`). Same defect as 26.2's
B2-G01.

### B2-G05 — Piston-shape driver hardcodes the 26.3 state count (MINOR, harness; 26.1.1 B2-G05 reproduces; 1.21.7 G13 survives)

`tools/test_piston_shape_parity.py:63` (byte-identical to 26.1.1's)
asserts `len(facts['states'])!=35723` — the 26.3 count. 26.1.2 has
29,873 states (`gameplay-facts.json`, 29,873 entries; datagen census
agrees), so the driver fails outright on 26.1.2 before comparing
anything. No other Batch 2 driver hardcodes 35723 (grep over all 10
drivers + the redstone/piston helper tools — only this line). Note the
oracle behind this driver (`VanillaPistonShapeOracle`) is itself
blocked only by cascade (B2-O04), so the driver is two repairs away
from runnable, not one. Same defect as 26.2's B2-G02 (there: 32,366
states).

### B2-G06 — Documented 0.01 piston-push height divergence (MINOR, pre-existing; 26.1.1 B2-G06 reproduces; 1.21.7 G14 survives)

`tools/test_projectile_piston_cobweb_parity.py:16` (byte-identical to
26.1.1's) documents in its own docstring that "vanilla's second push here
is 0.01 higher than Horizon's, by the cobweb or not", and deliberately
does not compare height. Cause unknown; the driver passes around it.
Recorded here as a known divergence, not an accepted gap (no cause or
owner established). Note this is about entity lift height, not the
`Entity.limitPistonMovement` clamp — that clamp itself matches vanilla
26.1.2 exactly (see passes). Same defect as 26.2's B2-G03.

### B2-G07 — `DiodeBlock.neighborChanged` carries the 26.2 same-block guard vanilla 26.1.2 does not have (MINOR; 26.1.1 B2-G07 reproduces)

**Check failed (1):** diode `neighborChanged` guard.

Vanilla 26.1.2 `DiodeBlock.neighborChanged` has NO same-block guard:
the method goes straight to `canSurvive` (bytecode offset 0), then
`checkTickOnNeighbor`, else dropResources + removeBlock +
updateNeighborsAt in all directions (fresh javap,
`evidence/diode.txt`, offsets 0–94; the class is byte-identical to
26.1.1's). The 26.1 tree carries this correctly — its `redstone.rs` has
an explicit comment noting 26.1 has no guard and 26.2 returns early.
The 26.1.2 tree instead carries the 26.2 form:
`level/redstone.rs:673-675` returns early when the block at the
position is no longer the diode:

```rust
if !self.b.is_block(self.state(pos), block) {
    return;
}
```

This is the stale-carry regression the 26.1.1 run found as NEW vs 26.1;
the 26.1.2 tree inherits it unchanged (`redstone.rs` byte-identical to
26.1.1's). Effect: when a neighbour change is delivered to a diode
position whose block has already been replaced, vanilla 26.1.2 still
runs the survive/tick path on the passed state (and drops the diode's
resources + notifies neighbours if it cannot survive), while the port
silently does nothing. Edge-case divergence — minor — but a genuine
regression against the 26.1 tree, and the clearest in-scope instance
of the stale-carry problem (§6).

## §5 Re-check verdicts

### 26.1.1 re-checks (PRIMARY — all 17 decided)

| 26.1.1 item | Verdict for 26.1.2 | Evidence |
|---|---|---|
| B2-G01 wire class name (major) | **REPRODUCES** → B2-G01 | Jar has only `RedStoneWireBlock` (byte-identical class); export chain `["RedStoneWireBlock", "Block"]`; `classes.rs:224` knows only `"RedstoneWireBlock"`; no alias mechanism |
| B2-G02 push-reaction loader (major) | **REPRODUCES** → B2-G02 | Jar enum NORMAL/DESTROY/BLOCK/IGNORE/PUSH_ONLY; facts carry those names (22,869/6,849/91/64); loader `data.rs:886-890` matches only 26.3 names |
| B2-G03 obsidian refusal (major) | **REPRODUCES** → B2-G03 | Jar `isPushable` offsets 44–85 list the four blocks; port has no such check; `named` lacks crying_obsidian/reinforced_deepslate |
| B2-G04 Y-bound off by one (minor) | **REPRODUCES** → B2-G04 | Jar allows y == maxY, refuses UP-push only from maxY; port uses `max_y() - 1` (319) |
| B2-G05 shape-driver count (minor) | **REPRODUCES** → B2-G05 | `test_piston_shape_parity.py:63` asserts 35723; 26.1.2 has 29,873 states |
| B2-G06 0.01 height divergence (minor) | **REPRODUCES** → B2-G06 | Cobweb driver docstring documents it; height deliberately uncompared |
| B2-G07 diode same-block guard (minor) | **REPRODUCES** → B2-G07 | Jar `neighborChanged` has no guard (offset 0 → canSurvive); port `redstone.rs:673-675` returns early |
| B2-O01 BlockUpdate oracle (major) | **REPRODUCES** → B2-O01 | Batch 0 raw 26.1.2 measurement: 32 errors, first "450: cannot find symbol" — identical to 26.1.1; every missing symbol re-verified against the 26.1.2 jar |
| B2-O02 RedstoneCatalog oracle (minor) | **REPRODUCES** → B2-O02 | Batch 0 raw 26.1.2 measurement: 4 errors, first "188: cannot find symbol" — identical to 26.1.1 |
| B2-O03 FaceSupport oracle (major) | **REPRODUCES** → B2-O03 | Batch 0 raw 26.1.2 measurement: 1 error, protected `CubeVoxelShape` ctor — identical to 26.1.1 |
| B2-O04 PistonShape cascade (minor) | **REPRODUCES** → B2-O04 | Batch 0 raw 26.1.2 measurement: class "cascade", depends on broken `VanillaFaceSupportOracle` |
| B2-O05 redstone_diff 26.3 default (minor) | **REPRODUCES** → B2-O05 | `redstone_diff.py:453` defaults `--resources` to the 26.3 path (file byte-identical to 26.1.1's) |
| B2-O06 piston_step_e2e 26.3 default (minor) | **REPRODUCES** → B2-O06 | `piston_step_e2e.py:31` defaults `--resources` to the 26.3 path (file byte-identical to 26.1.1's) |
| B2-H01 driver runs (minor) | **REPRODUCES** → B2-H01 | Oracles still broken (O01/O02/O03), driver constant still wrong (G05); prepared data NOT present for 26.1.2 — a prepare run is now also needed |
| B2-H02 level::tests run (minor) | **REPRODUCES** → B2-H02 | `tests.rs` byte-identical; 3 of 4 still expected to fail on G01/G02/G03; a 26.1.2 prepare is needed before the run |
| B2-W01 G11 comparator (minor, closed) | **CLOSED again** → B2-W01 | Vanilla 26.1.2 still takes max(frame, block) (offset 150); port still takes max |
| B2-W02 G12 destroy notify (minor, closed) | **CLOSED again** → B2-W02 | Vanilla 26.1.2 still uses setBlock(air, 18) (offset 302); port still matches |

No 26.1.1 item was FIXED in the 26.1.2 tree, and no item changed
verdict. The 26.1.1→26.1.2 delta touches nothing in scope, so this is
the expected outcome — verified fresh, not assumed.

### 1.21.7 re-check verdicts (all 8 behaviour + 5 oracle, mandatory)

| 1.21.7 item | Verdict for 26.1.2 | Evidence |
|---|---|---|
| G7 wire class name (major) | **REPRODUCES** → B2-G01 | Jar has only `RedStoneWireBlock`; export chain `["RedStoneWireBlock", "Block"]`; `classes.rs:224` knows only `"RedstoneWireBlock"`; no alias mechanism |
| G8 push-reaction loader (major) | **REPRODUCES** → B2-G02 | Jar enum NORMAL/DESTROY/BLOCK/IGNORE/PUSH_ONLY; facts carry those names (22,869/6,849/91/64); loader matches only 26.3 names |
| G9 obsidian refusal (major) | **REPRODUCES** → B2-G03 | Jar `isPushable` offsets 44–85 list the four blocks; port has no such check; `named` lacks crying_obsidian/reinforced_deepslate |
| G10 Y-bound off by one (minor) | **REPRODUCES** → B2-G04 | Jar allows y == maxY, refuses UP-push only from maxY; port uses `max_y() - 1` (319) |
| G11 comparator conductor (minor) | **WITHDRAWN** — vanilla changed → B2-W01 (closed) | 26.1.2 `ComparatorBlock.getInputSignal` takes `Math.max(frame, block)` (offset 150); port takes max (`redstone.rs:544`) |
| G12 destroy notify twice (minor) | **WITHDRAWN** — vanilla changed → B2-W02 (closed) | 26.1.2 `moveBlocks` destroy loop uses `setBlock(air, 18)` (offset 302, `bipush 18`) + explicit `updateNeighborsAt` loop; port matches (`pistons.rs:599`, loop at 671–683) |
| G13 shape-driver count (minor) | **REPRODUCES** → B2-G05 | `test_piston_shape_parity.py:63` asserts 35723; 26.1.2 has 29,873 states |
| G14 0.01 height divergence (minor) | **REPRODUCES** → B2-G06 | Cobweb driver docstring documents it; height deliberately uncompared |
| O1 BlockUpdate oracle (major) | **REPRODUCES** → B2-O01 | 32 errors for 26.1.2 (Batch 0 raw measurement + jar-verified causes); 1.21.7 had ~25 |
| O2 RedstoneCatalog oracle (major) | **REPRODUCES, smaller** → B2-O02 | 4 errors for 26.1.2 (was ~15 for 1.21.7, 1 for 26.2) |
| O3 RedstonePersistence oracle (minor) | **FIXED** — passes all three classpaths | Batch 0 raw 26.1.2 measurement: pass/pass/pass |
| O4 PistonPush oracle (minor) | **FIXED** — passes all three classpaths | Batch 0 raw 26.1.2 measurement: pass/pass/pass; 26.1.2 has `net.minecraft.world.entity.monster.zombie.Zombie` |
| O5 PistonMovement oracle (minor) | **FIXED** — passes all three classpaths | Batch 0 raw 26.1.2 measurement: pass/pass/pass |

### Withdrawn candidates (recorded explicitly — numbers not reused; land as closed items B2-W01/B2-W02)

#### G11 — Comparator conductor branch: WITHDRAWN, vanilla changed

1.21.7's G11 was that vanilla takes the item frame first and the port
takes `max(frame, block)`. In 26.1.2, vanilla CHANGED (same as 26.1,
26.1.1 and 26.2): `ComparatorBlock.getInputSignal` (bytecode offset 150,
`Math.max`, fresh javap in `evidence/comparator.txt`) computes
`Math.max(frameAnalogOrMIN, blockAnalogOrMIN)` and uses it unless both
are absent — i.e. vanilla 26.1.2 takes the max, exactly as the port does
(`redstone.rs:541-546`, `frame.into_iter().chain(block).max()`). The
port matches 26.1.2; there is no defect here. (The 1.21.7 G11 stands for
1.21.7 only.)

#### G12 — Piston-destroyed blocks notify twice: WITHDRAWN, vanilla changed

1.21.7's G12 was that vanilla destroys via `Level.destroyBlock` (flags
3, neighbours update during the destroy loop) plus an explicit loop,
while the port sets air with flags 18. In 26.1.2, vanilla CHANGED (same
as 26.1, 26.1.1 and 26.2): `moveBlocks`' destroy loop calls `setBlock(air, 18)`
(bytecode offset 302, `bipush 18`, fresh javap in
`evidence/pistonbase.txt`), then runs the explicit `updateNeighborsAt`
loop after — exactly what the port does (`pistons.rs:599`,
`set_block(*pos, b.air(), 18)`, explicit loop at `pistons.rs:671-683`).
The port matches 26.1.2; there is no defect here. (The 1.21.7 G12 stands
for 1.21.7 only.)

### Fixed in 26.1.2 (1.21.7 oracle defects, with evidence)

1.21.7's O3, O4 and O5 are FIXED for 26.1.2 — all three oracles pass on
all three classpaths in Batch 0's raw 26.1.2 measurement
(`versions/26.1.2/.local/measure-oracles/results.json`):
`VanillaRedstonePersistenceOracle`, `VanillaPistonPushOracle` (26.1.2
has `net.minecraft.world.entity.monster.zombie.Zombie`, the 1.21.7
break), `VanillaPistonMovementOracle`. No 1.21.7 behaviour item was
fixed in the 26.1.2 tree — all three majors reproduce, and G10/G13/G14
reproduce as well.

## Oracle defects (for Batch 0 — do not repair here)

From Batch 0's raw 26.1.2 measurement
(`versions/26.1.2/.local/measure-oracles/results.json` +
`big-compile-server.log`, 391 oracles measured, 189 pass on the client
classpath / 202 fail-all — identical headline to 26.1.1; see §0 on the
landed table). This batch did not re-measure: the table is the oracle
authority. The three failing Batch 2 oracle sources are byte-identical
between the 26.1.1 and 26.1.2 trees, the Batch 0 counts reproduce
26.1.1's exactly (32 / 4 / 1 / cascade), and this batch javap-verified
every missing symbol below against the 26.1.2 jar fresh. The full
32-error list was read from `big-compile-server.log` and matches the
26.1.1 anatomy line for line (same lines, same messages).

### B2-O01 — `VanillaBlockUpdateOracle` BROKEN, 32 errors (MAJOR)

Batch 0: fails on all three classpaths, 32 errors, first error
"450: cannot find symbol". Blocks `test_redstone_parity` (via
`block_update_harness.py`, which compiles and runs this oracle) plus
5 other drivers, and anchors 6 cascade oracles (`VanillaBrewingOracle`,
`VanillaEnchantDurabilityOracle`, `VanillaFurnaceOracle`,
`VanillaHelmetDamageOracle`, `VanillaOpenedLootOracle`,
`VanillaPortalOracle` — other batches' scope, recorded here for the
cascade). Error anatomy (full list from `big-compile-server.log`;
every cause javap-verified against the 26.1.2 jar by this batch):

- **`net.minecraft.world.level.blockscan` absent from 26.1.2** (5 errors:
  319, 322, 411, 427, 444 — 0 entries in the jar): the `findBlocksIn`
  overrides and the `Scan` class (`extends
  blockscan.OrderedBlockMatcher`, `BlockStateConsumer` params) are
  26.3-only API. `findBlocksIn` does not exist on 26.1.2's `Level` or
  `ServerLevel` either (javap: 0 matches).
- **`net.minecraft.util.Continuation` absent** (3 errors: 436, 439, 442 —
  0 entries for the top-level class in the jar; the only `Continuation`
  hits are nested classes — `AbortableIterationConsumer$Continuation`,
  `ContinuationTask` — not the imported type).
- **`net.minecraft.world.clock.ClockInstance` absent** (2 errors at 450,
  + 4 no-override at 451–454 for the anonymous class's
  totalTicks/partialTick/rate/isPaused — 0 entries for the top-level
  class; the only hit is the nested
  `ServerClockManager$ClockInstance`, not the imported type).
- **No-override** (15 total): `findBlocksIn` ×2 (319, 322),
  `getNextEntityId` (357 — absent from 26.1.2's `Level`/`ServerLevel`),
  `Scan` overrides ×6 (419, 423, 434, 438, 441, 444 — superclass
  missing), `ClockInstance` methods ×4 (451–454),
  `getCommandSpamThresholdSeconds` / `getChatSpamThresholdSeconds`
  (530, 531 — absent from 26.1.2's `MinecraftServer`; only
  `getCompressionThreshold` matches "threshold").
- **`MinecraftServer` ctor** (508): the oracle's 11-arg super call
  (…, `false`, `null` — the 26.3 `NotificationManager` slot) vs 26.1.2's
  10-arg ctor (javap-verified: `Thread, LevelStorageAccess,
  PackRepository, WorldStem, Optional<GameRules>, Proxy, DataFixer,
  Services, LevelLoadListener, boolean`).
- **`ServerChunkCache` ctor** (621): the oracle's 10-arg super call vs
  26.1.2's 11-arg ctor (javap-verified: `ServerLevel,
  LevelStorageAccess, DataFixer, StructureTemplateManager, Executor,
  ChunkGenerator, int, int, boolean, ChunkStatusUpdateListener,
  Supplier<SavedDataStorage>`).
- Plus 1 "non-static variable super" (424, `Scan.filterState`) and
  3 `statePredicate` cannot-find-symbol (436, 442, 445 — field of the
  missing superclass). (The log's 33rd grep hit is a source line, not
  an error: `EntityTypes.PLAYER.getDimensions()` inside the oracle.)

Notably GONE vs 1.21.7's O1: the permission APIs exist in 26.1.2 —
`operatorUserPermissions()` and `getFunctionCompilationPermissions()`
are abstract methods on 26.1.2's `MinecraftServer` (javap-verified
fresh), as are `useNativeTransport()` and
`isSingleplayerOwner(NameAndId)`, and `environmentAttributes()` exists
on 26.1.2's `Level`. The 26.1.2 breaks are the 26.3-only APIs (blockscan,
ClockInstance, Continuation, spam thresholds) plus the two ctor
mismatches.

### B2-O02 — `VanillaRedstoneCatalogOracle` BROKEN, 4 errors (MINOR)

Batch 0: fails on all three classpaths, 4 errors, first error
"188: cannot find symbol". Blocks `test_redstone_catalog_parity` only.
Errors (each javap-verified against the 26.1.2 jar):

- **188: `datapackWorldRegistries()`** — cannot find symbol. 26.1.2's
  `WorldLoader$DataLoadContext` (javap-verified,
  `evidence/dataloadcontext.txt`) has no such accessor — its record
  components are `resources()`, `dataConfiguration()`,
  `datapackWorldgen()`, `datapackDimensions()`;
  `datapackWorldRegistries()` is the 26.3 name. The 26.1.2 equivalent
  for the world-preset lookup is `datapackDimensions()` (a
  `RegistryAccess.Frozen`, which has `lookupOrThrow`). Same single break
  as 26.2's B2-O01.
- **212: `MinecraftServer` ctor** — the oracle's `Server` subclass calls
  the 11-arg super (…, `false`, `new NotificationManager()`); 26.1.2's
  ctor is 10-arg (same as B2-O01's 508).
- **464, 465: `getCommandSpamThresholdSeconds` /
  `getChatSpamThresholdSeconds`** — methods do not override; absent from
  26.1.2's `MinecraftServer` (same as B2-O01's 530/531).

### B2-O03 — `VanillaFaceSupportOracle` BROKEN, 1 error (MAJOR)

Batch 0: fails on all three classpaths, 1 error, "45:
`CubeVoxelShape(DiscreteVoxelShape)` has protected access in
`CubeVoxelShape`". Javap-verified fresh: 26.1.2's `CubeVoxelShape` ctor
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

`tools/redstone_diff.py:453` (byte-identical to 26.1.1's):
`p.add_argument('--resources', default=str(ROOT / '.local/horizon-client/26.3'))`.
On 26.1.2 the default points at data that does not exist (and the
docstring, line 2, says "vanilla 26.3"). Overridable via `--resources`,
so minor, but the default is wrong for this tree. Same as 26.2's
B2-O02.

### B2-O06 — `piston_step_e2e.py` defaults `--resources` to the 26.3 path (MINOR)

`tools/piston_step_e2e.py:31` (byte-identical to 26.1.1's):
`parser.add_argument('--resources', type=Path, default=ROOT / '.local/horizon-client/26.3')`.
Same shape as B2-O05 (docstring line 2: "actual vanilla 26.3 server").
Overridable, minor. Same as 26.2's B2-O03.

### Oracles assessed PASSING for 26.1.2 (Batch 0 raw measurement)

- `VanillaRedstonePersistenceOracle` — passes all three classpaths
  (1.21.7's O3 gone).
- `VanillaPistonPushOracle` — passes all three (1.21.7's O4 gone; 26.1.2
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
  G4: the 26.1.2 `EntityType` holder). No Batch 2 driver references it
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
the B2-G05 driver-constant fix — AND, new for 26.1.2, (e) a prepare run:
unlike 26.1.1, 26.1.2 has no prepared horizon-client data in the worktree
(`versions/26.1.2/.local/` has no `horizon-client/` export), so the
prepared-data precondition that was MET for 26.1.1 is NOT met for 26.1.2.
The nibble/press-time checks 26.2 deferred to H01 were still run here as
real checks (see Passes), via the identity chain. The static checks above
cover the logic; the drivers cover the corpus — and, once the oracles
compile and the data is prepared, the corpus runs will exercise the
G7/G8/G9 defects behaviourally (a wire/piston corpus cannot pass while
the wire is dead and every state loads PushPull).

### B2-H02 — The 4 level::tests need a cargo test run

`level/tests.rs` carries `redstone_lever_wire_and_lamp` (:1253),
`levers_power_wire_through_interaction` (:1734),
`pistons_push_and_sticky_pistons_pull` (:1781) and
`pistons_pop_fragile_blocks_and_refuse_obsidian` (:2120) — the file is
byte-identical to 26.1.1's. Unlike 26.2, where the root causes were fixed
and the tests were expected to pass, for 26.1.2 three of the four are
expected to FAIL (see Pilot leftover diagnosis) — but no cargo builds
were allowed in this batch (shared 6 GB VM), so a test run is still
needed to confirm behaviourally. Note the tests load the real prepared
resources and silently skip (the `world!()` macro returns early) when
`.local` is absent — which is why a crate-tests CI run could go green
while these defects stand: a clean runner has no prepared data, so the
level tests never ran there. A run with `HORIZON_RESOURCES` pointed at
prepared 26.1.2 data is what would demonstrate the failures — and that
data does not exist yet for 26.1.2 (§0), so the prepare is a
precondition for this run too.

## Passes (45 checks — javap + source/data agree)

Every pass below rests on the §2 identity chain: the vanilla class is
byte-identical to 26.1.1's (full-jar comparison, §2.1) and the port code
is byte-identical to 26.1.1's (full `level/` diff, §2.3), so the 26.1.1
run's javap-vs-source verification carries to 26.1.2 by identity. The
defect-bearing methods were additionally re-javap'd fresh on the 26.1.2
jar (§4, §5); the pass-critical port lines were spot-grepped in the
26.1.2 tree (wire arms `hooks.rs:100,211,380`; `redstone.rs` wire
functions at :126/:216/:236/:293/:408/:421/:439; `update.rs:15`
UPDATE_ORDER; `pistons.rs` flags at :564/:599/:612/:638; 12-block
literals at :447/:468/:506; `piston_movement.rs:31-36` ±0.51 clamp;
`geometry/mod.rs:44` Direction order).

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
(Jar classes byte-identical to 26.1.1's; port code byte-identical to
26.1.1's in this section.)

**Machinery (3 pass + B2-G07):** `SignalGetter` defaults live on the
interface in 26.1.2 — note it is at `net.minecraft.world.level.
SignalGetter`, not `.redstone.` (`getSignal`/`hasNeighborSignal`/
`getBestNeighborSignal`/`getDirectSignal`/`getControlInputSignal` —
the last bytecode-verified against the port's `control_input_signal`
(`redstone.rs:194-213`): diodes-only → diode direct signal, redstone
block → 15, wire → power, signal source → direct signal, else 0);
`NeighborUpdater.UPDATE_ORDER` is West/East/Down/Up/North/South in the
jar (static init verified fresh, `evidence/neighborupdater.txt`) and in
the port (`update.rs:15`); `Direction` enum order matches vanilla
(Down=0 … East=5, jar + `geometry/mod.rs:44`). The fourth machinery
check — **`DiodeBlock.neighborChanged` has no same-block guard in
26.1.2** — FAILS in this tree (B2-G07): the port adds the guard the
26.2 tree carries.

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
measurement (analog output, conductor pass-through with the 26.1.2 max
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
(Jar classes byte-identical to 26.1.1's; consumer code byte-identical to
26.1.1's.)

**Nibbles + press times (2 checks — re-run fresh against the
proven-identical prepared data + 26.1.2's own datagen):** weak/strong
nibble spot-checks against jar-derived expectations, all matching —
lever powered floor north (weak 15 all directions; strong 15 Up only,
via `getConnectedDirection`: FLOOR → UP, verified through the
`$SwitchMap`); stone button powered (same shape); redstone torch lit
(weak 15 except Up; strong Down only); redstone block (weak 15 all,
strong 0 all — the port's `control_input_signal` special case is the
correct handling, mirroring vanilla's); redstone wire power-15 cross
(weak/strong 15 except Down); observer powered north (15 north only);
repeater powered north (15 north only); comparator (0 in the export —
the export level has no block entity, so `getOutputSignal` reads 0; the
port correctly special-cases comparators instead of using the nibble).
Loader columns verified: `weak: int(5), strong: int(6)` in
`data.rs:879-880` match the export row layout. Button press times:
stone 20, wood 30 — datagen definition and export `fields`
(`ticksToStayPressed`) agree; the port reads the field at
`data.rs:1015`. Full output:
`evidence/census_nibbles_26_1_2.log` (identical to the 26.1.1 log
except the four added fallback lines).

**Observers (5 checks):** 2-tick pulse on/off, `updateNeighborsInFront`,
trigger via `updateShape` on the facing side only when unpowered
(`shapes.rs:668`), `startSignal` (schedule 2 ticks if none scheduled),
`onPlace` and `affectNeighborsAfterRemoval` arms (`hooks.rs:200,368` —
onPlace: powered && !hasScheduledTick → set powered=false flags 18 +
updateNeighborsInFront; removal: powered && hasScheduledTick →
updateNeighborsInFront with the powered=false state, no setBlock).
(`hooks.rs` byte-identical to 26.1.1's.)

**Pistons (11 checks):** quasi-connectivity (`getNeighborSignal` —
direct neighbours except the push direction, signal at self/down, and
the above-block scan except down); `checkIfExtend` events 0/1/2 with
the moving-piston progress rule (progress < 0.5 or same-tick or
handling-tick); `is_pushable` reaction-switch structure (BLOCK→false,
DESTROY→allow_destroyable, PUSH_ONLY→direction==connection — matches
vanilla's switch map; the values never arrive, B2-G02); resolver
structure (`resolve` / `addBlockLine` / `reorderListAtCollision` /
branching blocks) and the 12-block limit (literal `12` at three sites,
`pistons.rs:447,468,506`, matching the jar's three `bipush 12` in
`addBlockLine`); `moveBlocks` structure and flags (276 head removal /
18 destroy / 324 moving blocks / 82 head placement, destroy-then-push
order, shape updates at limit 512 — `UPDATE_LIMIT`, `mod.rs:169`);
destroy notification (G12 — withdrawn, port matches vanilla 26.1.2);
`triggerEvent` (server-side signal re-check, extend/retract, sticky-pull
guard: event 1, non-air, pushable, NORMAL reaction or piston);
moving-piston tick/finalTick (progress +0.5/tick clamped to 1.0,
deathTicks client wait < 5, waterlogged strip, air fallback flags 340 +
updateOrDestroy flags 3, normal path flags 67 + neighborChanged);
entity piston movement limit (±0.51 blocks/tick/axis, first-nonzero-axis
only, 1e-5 threshold, per-tick reset, lengthSqr 1e-7 early-out —
`Entity.limitPistonMovement` +
`applyPistonMovementRestriction` match `piston_movement.rs`);
slime/honey stick rules (`canStickToEachOther`: honey/slime mutually
exclusive, either sticky → stick); piston head survival (fitting base
or same-facing moving piston). (`pistons.rs` byte-identical to 26.1.1's;
jar classes byte-identical to 26.1.1's.)

## Pilot leftover diagnosis (summary)

| Test (`level/tests.rs`) | Verdict | Root cause |
|---|---|---|
| `redstone_lever_wire_and_lamp` (:1253) | **Expected FAIL** — run owed (B2-H02) | Wire class never dispatches (B2-G01); first assert (power "15") fails with power "0" |
| `levers_power_wire_through_interaction` (:1734) | **Expected FAIL** — run owed (B2-H02) | Same (B2-G01) |
| `pistons_push_and_sticky_pistons_pull` (:1781) | Expected pass — run owed (B2-H02) | Dirt/redstone_block are NORMAL reaction → PushPull either way; push/pull logic verified |
| `pistons_pop_fragile_blocks_and_refuse_obsidian` (:2120) | **Expected FAIL** — run owed (B2-H02) | Torch (DESTROY) pushed, no DropResources event (B2-G02); obsidian pushed instead of refused (B2-G03) |

No pilot failure remains unexplained for 26.1.2 — but unlike 26.2, three
of the four are expected failures, not expected passes. The 26.1.2 tree
is the 1.21.7 shape here, not the 26.2 shape: the template fixes never
landed in this tree. A CI green run for 26.1.2 would not contradict
this: the level tests skip silently without prepared data (`world!()`
returns early when `fixture()` is `None`), and the crate-tests workflow
has no prepare step — and for 26.1.2 the prepared data does not even
exist in the worktree yet (§0), so the run needs a prepare first.

## §6 Systemic findings

### The 26.1.2 tree inherits the 26.1.1 tree's stale-carry state wholesale

The 26.1.1 run established that the 26.1.1 tree did not inherit the
26.1 tree's `level/` behaviour fixes — it carries 26.2/26.3-form
behaviour the 26.1 tree deliberately removed, with B2-G07 (diode guard)
the in-scope instance. The 26.1.2 tree is `level/`-byte-identical to
the 26.1.1 tree, so every one of those instances carries to 26.1.2
unchanged. The out-of-scope instances below are flagged again for their
owning batches — each is a candidate defect of the same stale-carry
class, verified here only to the extent of the tree identity plus the
jar-identity fact (the 26.1.1→26.1.2 delta touches none of the classes
involved, so the 26.1.1 tree's vanilla form is the 26.1.2 vanilla form;
the tripwire claim was additionally re-verified fresh — vanilla
26.1.2's `TripWireHookBlock` still has exactly ONE `scheduleTick`, the
10-tick attached-circuit re-check at offset 281,
`evidence/tripwirehook.txt` line 323):

- **`interact.rs` bed use** (26.1.2 :367-384): the 26.3 pair check
  (`AbstractBedBlock` form); the 26.1 tree has the 26.1 form (foot
  click redirects to head, head click needs no foot check).
- **`interact.rs` composter** (26.1.2 :560-633): `composter_add_layer`
  (26.3 form) instead of 26.1's `composter_add_item` (level-0 always
  adds, above level 0 rolls under the item's chance).
- **`hooks.rs` ice melt** (26.1.2 :706): fires the `BLOCK_DESTROY` game
  event; the 26.1 tree notes the event is a 26.3 addition and 26.1's
  melt fires none.
- **`rails.rs` tripwire re-check** (26.1.2 :595-596): `tripwire_check_pressed`
  adds `else if was { schedule_tick(pos, block, 1) }` — a 1-tick
  pressed→unpressed re-check. Vanilla 26.1.2's `TripWireHookBlock`
  (byte-identical to 26.1.1's, re-verified fresh) has exactly ONE
  `scheduleTick` in the whole class — the 10-tick attached-circuit
  re-check inside `calculateState` (offset 281) — and no 1-tick unpress
  re-check. The 26.1 tree's comment states the unpress re-check is
  26.2-only.
- **`sculk.rs`** (26.1.2 :620): reads the
  `minecraft:sculk_growth_inhibitors` tag; the 26.1 tree notes 26.1 has
  no such tag (26.3 added it) and hardcodes sensor + shrieker.
- **`mod.rs` / `data.rs` / `hooks.rs` motion tags**: the 26.1.2 tree
  removed the per-state `blocks_motion()` rule (`mod.rs:975-985` in
  26.1) and reads `blocks_motion` / `blocks_fluid_flow` tags instead
  (`data.rs:426-427`, `hooks.rs:539`), and dropped the `cobweb` named
  entry (`data.rs:313` in 26.1). 26.1 has no `blocks_motion` tag
  family (26.3 added it) — whether 26.1.2's data carries these tags is
  Batch 3's question (cf. the 26.1 run's B3-G22 startup blocker).
- **`update.rs` `ensure_block_entity`** (26.1.2 :385-388): re-checks the
  block is still the new block after onPlace (26.2 form); the 26.1
  tree creates the block entity unconditionally.
- **`timeline.rs` scan order**: `between_closed` (26.1: x innermost,
  then y, then z) replaced by `box_scan_order` (26.3 `findBlocksIn`
  order: sections by x, z, y; blocks by y, z, x), used by
  `eyeblossom_tick` (:92) — the scan order assigns the per-neighbour
  random draws in a different order. Vanilla 26.1.2 has no blockscan
  package at all (0 jar entries).

`PORT-NOTES-26.1.2.md` states the carry-forward took "the completed
26.1 code" as base; the observed `level/` state contradicts that for
the behaviour fixes above — exactly as the 26.1.1 run found, because
the 26.1.2 tree is the 26.1.1 tree in `level/`. The coordinator may
want to check whether the carry-forward actually covered `level/` — if
it did not, every batch should expect this stale-carry class in its
scope, and the fix for each is the 26.1 tree's form, already written
and verified.

### The vanilla-check rule mattered in both directions again

Assuming the 1.21.7 form would have filed two false defects (G11, G12
— vanilla changed); assuming the 26.3 form (PushReaction renames,
obsidian IMMOVEABLE, `datapackWorldRegistries`, blockscan/
ClockInstance/Continuation) would have missed the real ones. The
26.1.1→26.1.2 round adds a third direction: assuming the 26.1.1 tree's
form carries to the 26.1.2 tree would have been right this time — but
only the full jar comparison + full tree diff prove it, and the one
thing that did NOT carry (prepared data) is the one thing the
assumption would have silently missed.

## Method notes / limitations

- **No driver runs, no cargo tests.** Blocked by the batch constraints
  (no builds on the shared VM) and independently by the broken oracles
  (B2-O01/B2-O02/B2-O03), the B2-G05 driver constant, and — new for
  26.1.2 — the absence of prepared 26.1.2 data. The oracle repairs +
  a prepare run + driver runs are the follow-up work (B2-H01/B2-H02).
- **Census-script correction (26.1.1 record).** The 26.1.1 run's census
  script computes `fallback_plain` as blocks whose chain has NO
  recognised class — but `'Block'` itself is in `known`
  (`classes.rs: Block = "Block"`), so every chain resolves to at least
  `['Block']` and that count is always 0 (both the 26.1.1 log on disk
  and a fresh re-run of the original script print 0). The 26.1.1
  report's "exactly three blocks" claim is nonetheless correct in
  substance: this batch's script adds the right computation — blocks
  whose most-derived RECOGNISED class is plain `Block` while the
  chain's most-derived class is not `Block` — and it yields exactly
  the three named blocks (redstone_wire, dirt_path, amethyst_block).
  The number was right; the shipped script did not compute it.
- **Vanilla 26.1.2 vs 26.2 deltas found in scope:** `DiodeBlock.
  neighborChanged` has no same-block guard in 26.1.2 (26.2 returns
  early when `getBlockState(pos).is(this)` fails) — the 26.1.2 tree
  carries the 26.2 form, wrongly (B2-G07); `SignalGetter` lives at
  `net.minecraft.world.level.SignalGetter` in 26.1.2; `MinecraftServer`'s
  ctor is 10-arg (no `NotificationManager`) and `ServerChunkCache`'s is
  11-arg; `WorldLoader$DataLoadContext` has no `datapackWorldRegistries()`
  (same as 26.2); the blockscan package, `Continuation` and
  `ClockInstance` do not exist in 26.1.2 at all (only unrelated nested
  classes with those names). Each verified against the 26.1.2 jar, not
  assumed from 26.1, 26.1.1, 26.2 or 26.3.
- **26.1.2 tree vs 26.2 tree:** the in-scope Rust code is otherwise
  identical to the verified 26.2 tree except where noted — `redstone.rs`
  differs from 26.1 only by the diode guard (which makes it match 26.2
  there, wrongly for 26.1.2); `pistons.rs` differs from 26.2 only by the
  missing obsidian block; `hooks.rs`/`interact.rs`/`mod.rs` differ from
  26.1 only in version comments plus the out-of-scope items in §6;
  `redstone/` and `tests/` are byte-identical to 26.1.1's. The behaviour
  deltas are concentrated in the table layer: `classes.rs` has no alias
  mechanism and no `RedStoneWireBlock` spelling, and `data.rs`'s
  push-reaction loader matches only the 26.3 enum names. The 26.2 tree
  is the natural reference for the G7/G8/G9 repairs.
- **Not re-verified here:** the experimental redstone evaluator
  (`redstone/experimental.rs` — behind the `redstone_experiments` flag,
  off in the test level; vanilla 26.1.2 has
  `ExperimentalRedstoneWireEvaluator` and `ExperimentalRedstoneUtils`,
  same as 26.2); dispense/dropper *behaviours* beyond the redstone
  trigger; sculk sensors (own family); command-block powering beyond
  the `neighborChanged` arm; the full per-state nibble diff (spot
  checks only — 8 block families verified, not all 29,873 states).
- **Adjacent observations (not Batch 2 findings):**
  - `dirt_path` (`DirtPathBlock`) and `amethyst_block` (`AmethystBlock`)
    fall back to plain `Block` in the 26.1.2 tree — the same bug class
    as B2-G01, for other batches (paths / amethyst). Full census: 3
    blocks fall back to plain Block despite a non-Block most-derived
    class; 87 blocks have an unrecognised most-derived class but a
    recognised ancestor (leaves, glass, furnaces, copper variants —
    behaviour largely carried by the ancestor).
  - `VanillaDispenserOracle` fails for 26.1.2 (15 errors, EntityTypes
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
  - `NeighborUpdater` lives at `net.minecraft.world.level.redstone.
    NeighborUpdater`, not `net.minecraft.world.level.block.
    NeighborUpdater` as the 26.1.1 report's §2 list implied — a path
    correction only; the class is byte-identical and the UPDATE_ORDER
    check passes either way.
- **Disk / artifacts:** javap dumps, the class-comparison log, the
  census script and its log are in `evidence/` under this batch's
  directory (`~/workspace/mcir-work/sweep-26.1.2-batches/b02/evidence/`).
  No files were written under `/tmp` or under the worktree.

## §7 Evidence index

Worktree `~/workspace/mcir-work/sweep-26.1.2`; batch dir
`~/workspace/mcir-work/sweep-26.1.2-batches/b02/`; jars
`~/workspace/mcir-work/26.1.x/versions/26.1.2/jars/`.

| Evidence | Location |
|---|---|
| `RedStoneWireBlock` only; no `RedstoneWireBlock` | jar listing, `server-game.jar` (unzip -l grep) |
| `PushReaction` enum NORMAL/DESTROY/BLOCK/IGNORE/PUSH_ONLY | `b02/evidence/pushreaction.txt` (javap on 26.1.2 server-game.jar) |
| `isPushable` obsidian refusal (offsets 44–85) + Y bounds (0–34, 106–125) | `b02/evidence/pistonbase.txt` (lines 582–712) |
| `moveBlocks` destroy `bipush 18` (offset 302) | `b02/evidence/pistonbase.txt` (line 830) |
| `ComparatorBlock.getInputSignal` `Math.max` (offset 150) | `b02/evidence/comparator.txt` (line 227) |
| `DiodeBlock.neighborChanged` no guard (offsets 0–94) | `b02/evidence/diode.txt` (lines 140–210) |
| `DetectorRailBlock.checkPressed` scheduleTick gated on minecart present | `b02/evidence/detectorrail.txt` (lines 116–240) |
| `TripWireHookBlock` single `scheduleTick` (10-tick, offset 281) | `b02/evidence/tripwirehook.txt` (line 323) |
| `WorldLoader$DataLoadContext` components (no `datapackWorldRegistries`) | `b02/evidence/dataloadcontext.txt` |
| `findBlocksIn`/`getNextEntityId` absent from `Level`/`ServerLevel` | `b02/evidence/level.txt`, `serverlevel.txt` (0 matches) |
| blockscan/Continuation/ClockInstance 0 jar entries (top-level); `CubeVoxelShape` ctor protected; `MinecraftServer` 10-arg; `ServerChunkCache` 11-arg; no spam thresholds | `b02/evidence/cubevoxelshape.txt`, `minecraftserver.txt`, `serverchunkcache.txt` + jar listing (§4 B2-O01–O03) |
| `NeighborUpdater.UPDATE_ORDER` West/East/Down/Up/North/South | `b02/evidence/neighborupdater.txt` (lines 184–214) |
| Full-jar class comparison 26.1.1 ↔ 26.1.2 (3 server / 8 client changed, none in scope; 41 in-scope classes byte-identical) | `b02/evidence/class_compare_26_1_2.log` |
| Datagen reports byte-identical 26.1.1 ↔ 26.1.2 (blocks/commands/packets/registries.json) | sha1 comparison (transcript in §2.4; 1,168 blocks, 29,873 states) |
| `classes.rs:224` wire spelling; `data.rs:886-890` loader; `data.rs:928` chain fallback; `data.rs:879-880` nibble columns; `data.rs:1015` press times; named `obsidian` :267, `respawn_anchor` :369 | `versions/26.1.2/crates/horizon-world/src/level/` |
| `pistons.rs:337` `max_y() - 1`; no obsidian check (grep empty) | `versions/26.1.2/crates/horizon-world/src/level/pistons.rs` |
| Diode guard `redstone.rs:673-675`; comparator max `redstone.rs:541-546` | `versions/26.1.2/crates/horizon-world/src/level/redstone.rs` |
| Wire arms `hooks.rs:100,211,380`; `survival.rs:43` prepare half | `versions/26.1.2/crates/horizon-world/src/level/hooks.rs`, `versions/26.1.2/crates/horizon-prepare/src/survival.rs` |
| Push census (22,869/6,849/91/64), class census (3 fallbacks, 87 ancestor-only), nibbles, press times | `b02/evidence/census_nibbles_26_1_2.py` + `census_nibbles_26_1_2.log` |
| Prepared data: redstone_wire chain `["RedStoneWireBlock", "Block"]`; 29,873 states | `~/workspace/mcir-work/sweep-26.1.1/versions/26.1.1/.local/horizon-client/26.1.1/{block-behaviours,gameplay-facts}.json` (proven-identical export; no 26.1.2 export exists — §0/§2.4) |
| Driver lines: shape `:63`, cobweb `:16`, `redstone_diff.py:453`, `piston_step_e2e.py:31` | `versions/26.1.2/tools/` (byte-identical to 26.1.1's) |
| Level tests `:1253, :1734, :1781, :2120` | `versions/26.1.2/crates/horizon-world/src/level/tests.rs` (byte-identical to 26.1.1's) |
| Oracle table (32/4/1/cascade; O3/O4/O5 pass) | `versions/26.1.2/.local/measure-oracles/results.json` + `big-compile-server.log` (Batch 0 raw 26.1.2 measurement; landed `docs/sweep/26.1.2/batch-0-oracles.json` not yet present at handoff) |
| `level/` tree diff 26.1.1 ↔ 26.1.2 (byte-identical, exit 0) | `diff -rq versions/26.1.1/crates/horizon-world/src/level versions/26.1.2/crates/horizon-world/src/level` |
| Oracle sources byte-identical 26.1.1 ↔ 26.1.2 (BlockUpdate/Catalog/FaceSupport) | `diff -rq` on `versions/26.1{,.2}/mods/horizon/src/test/java/dev/horizonlab/parity/` |

## §8 Verdict

Batch 2 for 26.1.2 is complete: **52 behaviour checks — 45 pass, 7
gap**; 17 items (7 behaviour gaps, 6 oracle defects, 2 needs-harness,
2 closed). All 17 26.1.1 items were re-checked and decided (all 17
reproduce with the same ids, including the two closed items landing
closed again); all 13 1.21.7 items were re-checked and decided (8
reproduce, 2 withdrawn, 3 fixed). No item is left UNVERIFIED.

The 26.1.2 redstone/piston picture is the 26.1.1 picture exactly: the
wire is dead (G01), every state loads PushPull (G02), obsidian pushes
(G03), the ceiling bound is off by one (G04), the shape driver cannot
run (G05), the 0.01 height divergence stands (G06), and diodes now
wrongly skip `neighborChanged` when already replaced (G07, the
26.1.1-era stale-carry regression). The three 1.21.7 persistence/piston
oracles stay fixed; the block-update, catalog and face-support oracles
stay broken with identical error counts. The corpus and level-test runs
that would demonstrate all of this behaviourally are blocked on the
oracle repairs (B2-H01/B2-H02) — and, new for 26.1.2, on a prepare run,
since no prepared 26.1.2 data exists in the worktree yet.

The finding the coordinator should weigh most is §6: the 26.1.2 tree
inherited the 26.1.1 tree's stale-carry state wholesale — `level/` is
byte-identical — so the 26.1 tree's behaviour fixes are still absent
from this line, and the diode guard is unlikely to be the only
stale-carry defect in the programme. Every batch should diff its scope
against the 26.1 tree the way the 26.1.1 run did; this batch's
26.1.1↔26.1.2 diff proves the delta port carried the defects forward
without change.
