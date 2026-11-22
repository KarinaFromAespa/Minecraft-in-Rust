# Batch 2 — Redstone + pistons (1.21.7 parity sweep)

**Version:** 1.21.7 · **Date:** 2026-11-22 · **Branch:** `parity-sweep` (workdir
`~/workspace/mcir-work/sweep-batch-2`, tip `adc7e28a`) · **Scope:**
`docs/PARITY-SWEEP.md` §4 Batch 2 — redstone ticking, pistons, observers,
comparators; 10 drivers + circuit probes; pilot leftovers (3 level::tests).

**Status: DRAFT — not committed.** An execution-risk notice on the setup step
blocks running, building, testing, or issuing git commands inside the repo
without the user's go-ahead, so no driver runs, no cargo tests, and no
commit/push happened in this batch. Everything below is static analysis:
`javap -c` on the named 1.21.7 server jar, source comparison against
`versions/1.21.7/crates/`, and checks of the prepared 1.21.7 data
(`~/workspace/mcir-work/mob-spawn/resources/`, cross-checked against the main
checkout's `versions/1.21.7/.local/horizon-client/1.21.7/` copy). The
coordinator needs the user's go-ahead to run the drivers/tests and to commit
these deliverables to `docs/sweep/batch-2-redstone.md` and
`pins/1.21.7/sweep/batch-02.json`.

## Tally

- **Behaviour checks: 50 — 34 pass, 16 gap** (across 8 defect items G7–G14).
- **Oracle defects: 5** (O1–O5, for Batch 0 — not counted in the check tally).
- **Pilot leftovers: all 3 diagnosed.** Both wire tests and the piston test
  fail for identified, jar-verified reasons (G7, G8, G9). No pilot failure
  remains unexplained.

## Defects

### G7 — Redstone wire is completely non-functional (MAJOR, NEW)

**Checks failed (6):** lever→wire power propagation (15..11 down the line);
wire signal emission; lamp lighting via wire; lever-interaction wire power;
wire side-connection updates; wire dot/cross toggle on use. The button
half of `levers_power_wire_through_interaction` is untested, blocked behind
the wire assert — not itself failed.

**Cause — a class-name mismatch, not a logic bug.** Vanilla 1.21.7 names the
class `RedStoneWireBlock` (capital S). The 1.21.7 export says so:
`block-behaviours.json`, `minecraft:redstone_wire` → class chain
`['RedStoneWireBlock', 'Block']`. The port's class table
(`level/classes.rs:227`) only knows `"RedstoneWireBlock"` (lowercase s), so
`Class::from_java("RedStoneWireBlock")` returns `None`, the name is skipped
when the chain is built (`level/data.rs:943-955`), and the wire resolves to
`Class::Block`.

Every class-dispatched wire arm therefore never runs:
`neighbor_changed` → `wire_update_power` (`hooks.rs:100`), `on_place`
(`hooks.rs:211`), `affectNeighborsAfterRemoval` (`hooks.rs:380`),
`update_shape` → `redstone_wire_update_shape` (`shapes.rs:698`),
`vanilla_signal`/`vanilla_direct_signal` → `wire_signal`
(`redstone.rs:65,101`), `interact` → dot/cross toggle (`interact.rs:263`),
placement (`place.rs:1089`), survival (`survive.rs:281` — falls back to the
`can_survive` default `true`, so wires still survive).

**Effect:** a wire never changes power (stays 0), never emits signal
(`vanilla_signal` falls through to the weak nibble, which is 0 for wire),
never updates its side connections. `redstone_lever_wire_and_lamp` fails at
its first assert (power `"0"` vs `"15"`); `levers_power_wire_through_interaction`
fails at the wire-power assert (power `"0"` vs `"13"`).

**The wire logic itself is correct.** Verified method-by-method against the
1.21.7 jar: `wire_update_power` / `wire_incoming_signal` /
`wire_block_signal` (power math, 15-cap, decay), `wire_connection_state`
(dot early-return + straight-line fixups, byte-identical structure to
`RedStoneWireBlock.getConnectionState`), `wire_connecting_side` (up-connection
via trapdoor/survive + face-sturdy, conductor/below rules), `wire_signal`
(down=0, up=power, opposite-direction connection check — the
inverted-direction hypothesis for the lamp failure is refuted; the port
matches vanilla), `should_wire_connect_to` (wire/observer/repeater/signal-source
arms). The fix is a one-line table alias (`"RedStoneWireBlock"`), not a logic
change. The same mismatch is in `horizon-prepare/src/survival.rs:43`
(`"RedstoneWireBlock"`), worth checking in the fix.

**Cross-version note:** only 1.21.7's `block-behaviours.json` exists on this
VM, so whether other versions' exports also write `RedStoneWireBlock` could
not be checked locally. Vanilla's class name is `RedStoneWireBlock` in every
version, so any version whose exporter writes the real class name has the
same defect; the fix agent should check the other versions' tables/data.

### G8 — Push-reaction loader mapping matches no 1.21.7 value (MAJOR, NEW)

**Checks failed (4):** torch popped by a piston; DESTROY blocks popped in
general; BLOCK blocks refusing a push (anvil, grindstone, lodestone,
piston_head); glazed terracotta push-only (not pulled by sticky pistons).

**Cause.** `gameplay-facts.json` uses vanilla `PushReaction` names:
`NORMAL` (21,237 states), `DESTROY` (6,554), `BLOCK` (91), `PUSH_ONLY` (64).
The loader (`level/data.rs`) matches `"PUSH"` / `"POPPED"` /
`"IMMOVEABLE"` / `"IGNORE_ENTITY"` — none of which occur in the data — so
**every one of the 27,946 states loads as `PushReaction::PushPull`**.

**Effect:** 335 DESTROY blocks (torches, wire, flowers, rails, …) are pushed
instead of popped; 13 BLOCK blocks (anvils, grindstone, lodestone, barrier,
portals, piston_head — pistons themselves are saved by the explicit
`is_piston` check, moving_piston by hardness −1) are pushed instead of
blocking; 16 glazed terracottas become pullable by sticky pistons
(`piston_trigger_event`'s pull guard compares against `PushPull`). This is
the torch half of `pistons_pop_fragile_blocks_and_refuse_obsidian`: the
torch is pushed to (2,1,0) instead of dropping, so the `DropResources`
assert fails.

### G9 — No hardcoded obsidian-family refusal in `is_pushable` (MAJOR, NEW)

**Check failed (1):** obsidian (and crying obsidian, respawn anchor,
reinforced deepslate) refusing a push.

**Cause.** Vanilla 1.21.7 `PistonBaseBlock.isPushable` (bytecode-verified)
refuses `OBSIDIAN`, `CRYING_OBSIDIAN`, `RESPAWN_ANCHOR` and
`REINFORCED_DEEPSLATE` explicitly, before the reaction switch — all four
carry the `NORMAL` push reaction (obsidian hardness 50.0, so the
hardness −1 check does not save it either). The port's `is_pushable`
(`level/pistons.rs:328`) has no such check. This is the obsidian half of
`pistons_pop_fragile_blocks_and_refuse_obsidian`: the piston extends and
pushes the obsidian instead of staying retracted.

### G10 — `is_pushable` Y-bound off by one at the ceiling (MINOR, NEW)

**Check failed (1):** pushing upward at the build ceiling.

Vanilla bounds-checks `y < minY || y > maxY` (y == 320 allowed) and refuses
only an UP-push *from* y == maxY. The port uses `max_y = level.max_y() - 1`
(319): it refuses y == 320 outright and refuses UP-pushes from y == 319.
A piston at y == 319 pushing up fails in the port (the y == 320 air cell
reads unpushable) where vanilla extends. The floor side matches vanilla
exactly. `LevelAccess::max_y()` default is 320, `min_y()` −64.

### G11 — Comparator conductor-branch precedence: frame vs container (MINOR, NEW)

**Check failed (1):** item frame and analog-output block both behind a
conductor, frame weaker than the container.

Vanilla `ComparatorBlock.getInputSignal` takes the item frame first
(`getItemFrame`, then `getAnalogOutput`; the further block's analog output
only if there is no frame). The port (`diode_input_signal`,
`level/redstone.rs`) takes `max(frame, block)`. When both exist and the
frame is weaker, the port reads higher than vanilla. Everything else about
the comparator matches (see passes).

### G12 — Piston-destroyed blocks notify neighbours once, vanilla twice (MINOR, NEW)

**Check failed (1):** neighbour-notification count/timing for blocks a
piston destroys.

Vanilla `moveBlocks` destroys via `Level.destroyBlock` (flags 3 → neighbour
updates fire *during* the destroy loop) and then runs its explicit
`updateNeighborsAt` loop *after* the move — two notifications. The port
(`piston_move_blocks`) sets the destroyed cells to air with flags 18 (no
neighbour updates) and only runs the explicit loop — one notification, after
the move. Final states agree; the difference is observable only through
BUD/quasi-connectivity timing edge cases.

### G13 — Piston-shape driver hardcodes the 26.3 state count (MINOR, harness)

`tools/test_piston_shape_parity.py` asserts
`len(facts['states']) == 35723` — the 26.3 count. 1.21.7 has 27,946 states,
so the driver fails outright on 1.21.7 before comparing anything. The
oracle itself (`VanillaPistonShapeOracle`) is likely fine (see O-section);
the driver constant is the defect.

### G14 — Documented 0.01 piston-push height divergence (MINOR, pre-existing)

`tools/test_projectile_piston_cobweb_parity.py` documents in its own
docstring that "vanilla's second push here is 0.01 higher than Horizon's,
by the cobweb or not", and deliberately does not compare height. Cause
unknown; the driver passes around it. Recorded here as a known divergence,
not an accepted gap (no cause or owner established).

## Oracle defects (for Batch 0 — do not repair here)

All assessed statically with `javap -p/-c` against the named 1.21.7 jars
(`~/workspace/mcir-work/leakage-audit/jars/1.21.7-{server,client}-named.jar`;
jar `version.json`: id 1.21.7, world_version 4438, protocol 772).

### O1 — `VanillaBlockUpdateOracle` BROKEN, ~25 compile breaks (MAJOR)

Behind `test_redstone_parity` and many other drivers. 26.x APIs missing or
changed in 1.21.7: `GameRules` in the wrong package (`world.level.gamerules`
vs `world.level.GameRules`); `GameRule` type + wrong rule constants
(1.21.7 uses `RULE_*`); no `GameRules.get()` (it's `getRule()`);
MinecraftServer 11-arg super-call vs the 8-arg ctor; ServerLevel 10-arg
super-call vs the 12-arg ctor; ServerChunkCache 10-arg super-call vs the
13-arg ctor; overrides that don't exist in 1.21.7
(`operatorUserPermissions`, `getFunctionCompilationPermissions`,
`getCommandSpamThresholdSeconds`, `getChatSpamThresholdSeconds`,
`useNativeTransport`, `isSingleplayerOwner(NameAndId)`,
`getRandomSequence` on MinecraftServer, `findBlocksIn`, `getOverworldClockTime`,
`getMoonBrightness`, `canSpreadFireAround`, `getNextEntityId`, `getBrightness`,
`getRawBrightness`, `hasEntities`, `environmentAttributes`); missing classes
(`world.clock.ClockInstance`, `world.level.blockscan.*`, `util.Continuation`,
`core.particles.ExplosionParticleInfo`,
`world.attribute.EnvironmentAttributeSystem`); no `RegistryLayer.WORLD`
(STATIC, WORLDGEN, DIMENSIONS, RELOADABLE only); no
`RegistryDataLoader.WORLD_REGISTRIES` and no 4-arg load with Executor; no
`Level.border` field (private `worldBorder`); no
`BlockStateBase.shouldChangedStateKeepBlockEntity`; 13-arg `explode`
override vs 1.21.7's 12-arg abstract (no `WeightedList` param).

### O2 — `VanillaRedstoneCatalogOracle` BROKEN, ~15 compile breaks (MAJOR)

Imports 7 missing classes (`Gizmos`, `GizmoCollector`,
`LevelBasedPermissionSet`, `PermissionSet`, `NameAndId`,
`EmptyNotificationService`, `NotificationManager`); MinecraftServer 11-arg
super-call (1.21.7: 8 args); overrides `operatorUserPermissions` /
`getFunctionCompilationPermissions` / `isSingleplayerOwner(NameAndId)`
don't exist (1.21.7: `getOperatorUserPermissionLevel()` /
`getFunctionCompilationLevel()` ints, `isSingleplayerOwner(GameProfile)`);
PlayerList 4-arg ctor wrong (1.21.7: MinecraftServer,
LayeredRegistryAccess, PlayerDataStorage, int); LevelSettings 5-arg ctor
with `DifficultySettings` (1.21.7 has no `DifficultySettings`; 7-arg ctor);
`WorldLoader.InitConfig` 3rd arg is an int function-compilation level, not a
permission set; PrimaryLevelData 3-arg call vs 4-arg ctor (missing
WorldOptions); no nested `LevelDataAndDimensions.WorldDataAndGenSettings`
(it's a record of WorldData + Complete, does not implement WorldData);
GameRules wrong package + wrong constants + no `set()` method;
`BlockState.getAnalogOutputSignal` is 2-arg in 1.21.7 (oracle passes a
Direction); `Level.tickBlockEntities` is protected (oracle calls it
directly); overrides `getCommandSpamThresholdSeconds` /
`getChatSpamThresholdSeconds` / `useNativeTransport` don't exist.

### O3 — `VanillaRedstonePersistenceOracle` BROKEN, 1 break (MINOR)

10-arg ServerLevel super-call vs 1.21.7's 12-arg ctor.

### O4 — `VanillaPistonPushOracle` BROKEN, 1 break (MINOR)

Imports `net.minecraft.world.entity.monster.zombie.Zombie`; 1.21.7 has
`net.minecraft.world.entity.monster.Zombie`. All reflective names it uses
were verified present (`progress`, `moveByPositionAndProgress`,
`getExtendedProgress`, `getMovement`, `matchesStickyCritera`,
`getMovementDirection`, entity position/bb/onGround/pistonDeltas/
pistonDeltasGameTime/mainSupportingBlockPos, `limitPistonMovement`,
`PistonMath.getMovementArea`).

### O5 — `VanillaPistonMovementOracle` BROKEN, 1 break (MINOR)

10-arg ServerLevel super-call vs 1.21.7's 12-arg ctor.

### Oracles assessed LIKELY OK (no repair needed)

- `VanillaClientRedstonePreviewOracle` — ClientLevel 10-arg ctor matches
  1.21.7; `hasNeighborSignal` override valid (default on `SignalGetter`);
  `Bootstrap` present in the client jar.
- `VanillaPistonShapeOracle` — `progress`, `progressO`, `NOCLIP`,
  `getCollisionRelatedBlockState`, `getMovementDirection`, `getProgress`,
  `getXOff/YOff/ZOff`, `getCollisionShape` all present. (Its driver is
  broken by G13, not the oracle.)
- Helpers `VanillaCollisionOracle`, `VanillaBlockStateCodecOracle`,
  `VanillaFaceSupportOracle` — all reflective names verified present.
- `ParityData.java` — clean for 1.21.7 (`RegistryDataLoader` 3-arg sync
  load, `ReloadableServerRegistries.reload`/`LoadResult.layers()`/
  `lookupWithUpdatedTags()`, `LayeredRegistryAccess` methods, `TagLoader`
  methods, `Level.registryAccess` field all verified).

## Passes (34 checks — javap + source/data agree)

**Wire (logic — all moot at runtime until G7 is fixed):** power math
(block signal / incoming signal / 15-cap / decay); connection state
(dot + straight-line fixups); connecting side (incl. up-connection);
`getSignal` direction rules; `SignalGetter` defaults
(`getSignal`/`hasNeighborSignal`/`getBestNeighborSignal`/`getDirectSignal`/
`getControlInputSignal` — these live on the `SignalGetter` interface, not on
`Level`); lever/button weak/strong nibbles, data-verified per state
(floor lever strong up, wall lever strong toward facing, torch weak 15
except up / strong down, redstone block weak 15 / strong 0, lamp is a
conductor).

**Consumers:** lamp on/off (immediate on, 4-tick off delay); torch inversion
and wall/floor attachment signal; torch burnout (8 toggles / 60-tick window /
160-tick relight / level event 1502); repeater delay (2×delay), lock, tick
priorities (ExtremelyHigh/VeryHigh/High); diode input (incl. wire-power
fallback), alternate signal (diodes-only for repeaters, any source for
comparators), `shouldTurnOn`, `checkTickOnNeighbor`, tick; comparator
subtract/compare output math; comparator container measurement (analog
output, conductor pass-through, 2-tick refresh, stored output); dispenser
trigger (signal at pos or above, 4-tick delay, `triggered` flag); button
press/release (stone `ticksToStayPressed` = 20 in the export; press sets
powered + schedules the release tick, tick releases when unpressed — source +
data, not javap); hopper power lock (`enabled = !signal`); daylight detector
(sky light, inversion,
cos sun shaping, clamp — the `.max(0)` before inversion is harmless: for
inverted detectors at night both forms read 15); target block (distance
strength, ceil, min 1, arrow 20 / other 8 ticks).

**Observers:** 2-tick pulse on/off, `updateNeighborsInFront`, trigger via
`updateShape` on the facing side only, `onPlace` and
`affectNeighborsAfterRemoval` arms.

**Pistons:** quasi-connectivity (`getNeighborSignal` — direct neighbours
except the push direction, signal at self/down, and the above-block scan);
`checkIfExtend` events 0/1/2 with the moving-piston progress rule
(progress < 0.5 or same-tick or handling-tick); resolver structure
(`resolve` / `addBlockLine` / `reorderListAtCollision` / branching blocks)
and the 12-block limit (`bipush 12`, twice); `moveBlocks` structure and
flags (324/276/82/18, destroy-then-push order, shape updates at limit 512);
`triggerEvent` (server-side signal re-check, extend/retract, sticky-pull
guard structure); moving-piston tick/finalTick (progress +0.5/tick,
deathTicks client wait, waterlogged strip, air fallback); entity piston
movement limit (0.51 blocks/tick/axis, first-nonzero-axis only, 1e-5
threshold, per-tick reset — `piston_movement.rs` matches
`Entity.limitPistonMovement` + `applyPistonMovementRestriction`); normal
push + sticky pull mechanism (consistent with the passing
`pistons_push_and_sticky_pistons_pull` test).

**Machinery:** neighbour-update order (`UPDATE_ORDER` West/East/Down/Up/
North/South matches vanilla) and synchronous drain; `Direction` enum order
matches vanilla (Down=0 … East=5), so weak/strong nibble indexing is sound.

## Pilot leftover diagnosis (summary)

| Test | Verdict | Root cause |
|---|---|---|
| `redstone_lever_wire_and_lamp` | Fails, first assert | G7 — wire class never dispatches; power stays 0 |
| `levers_power_wire_through_interaction` | Fails, wire-power assert | G7 (same); button half untested behind it |
| `pistons_pop_fragile_blocks_and_refuse_obsidian` | Fails, both halves | G8 (torch pushed, not popped) + G9 (obsidian pushed, not refused) |

The pilot's hypothesis (a 26.x-tag pattern extending here) is **not** the
cause of any of these three. G7 is a class-table string mismatch, G8 a
data-loader enum mismatch, G9 a missing hardcoded vanilla rule. The
supports-runtime fix (6e6cb55b, landed after the pilot ran) does not touch
any of these paths.

## Method notes / limitations

- **No driver runs.** All 10 Batch 2 drivers were read; none could run:
  the execution-risk block (no repo execution without user go-ahead), and
  independently the oracles behind the main drivers don't compile for
  1.21.7 (O1–O5, Batch 0's scope) and no worktree has
  `.local/minecraft/1.21.7/` for the harness gate. `docs/sweep/`
  does not exist yet — Batch 0's measurement doc had not landed as of
  2026-11-22 ~13:00 NPT.
- **No cargo tests.** The three pilot failures were diagnosed from source +
  jar + data only. A test run (once unblocked) should confirm: the wire
  tests pass after the G7 table fix, the piston test passes after the G8
  loader fix + G9 hardcode, and no other level::tests regress.
- **What a driver run would still add:** tick-by-tick circuit equivalence
  (random circuits, catalog contraptions), piston push/shape/movement
  corpus comparison, and persistence round-trips — all blocked on Batch 0's
  oracle repair. The static checks above cover the logic; the drivers
  cover the corpus.
- **Not re-verified here:** the experimental redstone evaluator
  (`redstone/experimental.rs` — behind the `redstone_experiments` flag,
  off in the test level); dispense/dropper *behaviours* beyond the
  redstone trigger; sculk sensors (own family); command-block powering
  beyond the `neighborChanged` arm.
- **Adjacent observation (not a Batch 2 finding):** 28 exported class names
  don't map to the port's class table (verified exhaustively: 283 exported
  vs 272 known). Only `RedStoneWireBlock` is in Batch 2 scope. The rest
  mostly fall back to a recognised ancestor (leaves, glass, furnaces,
  copper variants, skulls); four fall back to plain `Block` — `FarmBlock`
  (farmland), `DirtPathBlock` (dirt path), `AmethystBlock` (amethyst
  block), `SnowyDirtBlock` (podzol; grass/mycelium fall back to
  `SpreadingSnowyDirtBlock`, also unmapped, then `Block`). Flagged for the
  coordinator / the systematic sweep noted in PORT-NOTES.
- **Disk:** quota was tight at setup (writes failed until the regenerable
  `versions/26.2/.local` was deleted per the owner's standing rule);
  52G free at the end of this batch. The workdir is 1.6G (sparse checkout).
