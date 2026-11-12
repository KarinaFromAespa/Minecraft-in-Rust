# Port notes: Minecraft 1.21.8

1.21.8 is the third adjacent delta port in the **Components era (E6,
1.20.5–1.21.11)**, taken from 1.21.9. Unlike 1.21.9, which shared its
protocol with 1.21.10, this one crosses a protocol version: protocol
**773 → 772**, world version 4554 → 4440, data pack 88.0 → 81.0,
resource pack 69.0 → 64.0. The adapter ladder is re-based to the 772
pin, the packet set is rewired (1.21.9's code-of-conduct and debug
packets go away, 108 play packets renumber), and the parity oracles
are ported back to the 1.21.8 API — seven files, 27 compile errors,
all fixed. The port is the mechanical layer: the pin is set, the six
generated tables are regenerated against 1.21.8's own jar, the
verification gates pass, and the remaining work is sized, ranked and
documented. The behaviour code in this tree is still the 26.3 base's,
inherited through 1.21.11, 1.21.10 and 1.21.9; the port is **in
progress**, not parity-verified.

## Provenance

- Base: the programme's published 1.21.9 tree (main@a1d6494: same-protocol
  delta from 1.21.10, 2/6 tables regenerated, protocol tests 438 pass /
  2 fail), with the measured 1.21.9 → 1.21.8 delta re-applied (method (b)
  from the 26.1 carries).
- 1.21.8's facts, from its own version JSON and generated tables:
  protocol **772**, world (data) version **4440**, data pack format
  **81.0**, resource pack format **64.0**, Java **21**, stable release.
  (The diff report lists the pack formats as `None.None` — a diff-tool
  artifact; the generated `version.rs` and the version JSON both give
  81.0 / 64.0.)
- Prepare ran through the pipeline (verified jar download + generalized
  mappings front-end remap + datagen, 2026-11-12 ~03:44–03:50 UTC,
  ~6 min): **9,485/9,485 classes mapped (100%), 9,426 renamed,
  0 placeholders; 83,789 methods, 42,855 fields**. Nothing
  Mojang-derived is committed in this tree.
- Diff 1.21.9 → 1.21.8: **526 ranked work items** (rank 1: 9,
  rank 2: 317, rank 3: 163, rank 4: 35, rank 5: 2). Classes +120 −604,
  7 moved, **1,756 changed, 1,687 concern Horizon code or harnesses**.

## The delta: a protocol change and a debug-payload rework

Rank 1 is the protocol item, the six table regenerates, the save-format
item (DataFixers 4554 → 4440: 0 schemas, 0 fixes added, 10 datafix
classes changed) and the version facts. The protocol item:

- Configuration: `code_of_conduct` (clientbound) and
  `accept_code_of_conduct` (serverbound) removed — 1.21.9 additions,
  absent in 1.21.8.
- Play clientbound: `debug/block_value`, `debug/chunk_value`,
  `debug/entity_value`, `debug/event` and `game_test_highlight_pos`
  removed; **108 packets renumber**.
- Play serverbound: `debug_subscription_request` reverts to its 1.21.8
  name `debug_sample_subscription` (same id 22).
- The debug-payload classes move the other way: 1.21.8 carries the
  custom-payload debug system (`BeeDebugPayload`, `BrainDebugPayload`,
  `PathfindingDebugPayload`, `RaidsDebugPayload`, … — 28 classes) that
  1.21.9 replaced with dedicated packets.

Rank 2 is entirely 317 harness reruns. Rank 3 is 163 unproved items,
rank 4 is 35, rank 5 is 2 classes Horizon does not port.

## Tables: 6 regenerated

| Generator | Result vs 1.21.9 |
|---|---|
| `version_consts` | regenerated (16 diff lines — the pin: 772 / 4440 / 81.0 / 64.0) |
| `packet_ids` | regenerated (236 diff lines — the renumbers) |
| `entity_tracking` | regenerated (2 diff lines) |
| `lan_command_order` | regenerated (3 diff lines) |
| `command_suggestions` | regenerated (0 diff lines — identical output) |
| `vanilla_command_order` | regenerated (1 diff line) |

The committed `regen.json` lists all six generators (D1 completeness,
per the gate change at main@15ff93a8): the final full regen ran with
no `--only` and reported all six `unchanged` against the candidate,
which already carried the regenerated tables. A fresh regen
reproduces all six committed tables byte-for-byte — see
`regen-determinism.json`, no overrides. (The brief expected the five
SlotDisplay hand-corrected tables to differ from a fresh regen; they
do not — the generators emit the same numbering the hand corrections
carry, as they did for 1.21.9.)

## Packet rewire

Following the d8479b1 pattern, checked with `javap` against the named
1.21.8 jars:

- `CodeOfConduct` (configuration) and `GameTestHighlightPos` (play)
  packet types removed; the four `debug/*` play packets removed.
- `debug_subscription_request` → `debug_sample_subscription` rename
  applied (serverbound, id 22).
- Five-packet codec rewire for the reshaped packets (ShortVec3,
  Explode/rotation/spawn reshaped) via the boundary approach:
  the codec reads the 1.21.8 wire shape and converts at the boundary,
  leaving the game-side types untouched.

## Adapter ladder: re-based to 772

The whole ladder re-based to the new pin, reusing the 1.21.10 re-base
scripts and the prepared legacy versions:

- State aliases: `v1_21_3` state-alias added; BRICK 1025 → 980;
  trim seed 1410 → 1354.
- SlotDisplay fix carried from the 1.21.10 re-base (the codec still
  uses 26.3 numbering; the five generated tables are hand-corrected
  to it and must be regenerated to old→pin when the behaviour port
  rewires the codec).
- Serializer rewire to the 772 order (0–34 from the jars),
  `Quirks::PINNED`, HUMANOID_ARM 38/42, Profile wire codec removed.
- All 22 serializer test failures fixed (Particle::write quirk,
  read_template/write_template, pot/container/damage-resistance
  conversions, EitherHolder flag, hash template_code, oracle REGEN
  hexes), plus the spawn-test rewire fixes.

## Oracle rework for prepare

`horizon-prepare` initially failed with **27 javac errors across 7
oracle files** — the 1.21.9 APIs the oracles used do not exist in
1.21.8. All fixed by porting the oracles back to the 1.21.8 API
(no behaviour change to what the oracles measure):

- `VanillaEntityModelExporter`: 1.21.8's `Vertex` is a
  `(pos, u, v)` record — no `x()`/`y()`/`z()`; read `pos().x/y/z`.
- `VanillaMobDataExporter`: 1.21.8's `Keyframe` has one `target()`;
  the pre/post split arrived in 1.21.9 — both are the same target.
- `VanillaHeightmapOracle`, `VanillaSimpleBlockOracle`: no
  `PalettedContainerFactory` / `Strategy.createFor*` in 1.21.8, and
  no `BuiltInRegistries.BIOME` (biomes are datapack registries, not
  built-in). The sections are now built directly around a standalone
  PLAINS holder and handed to the 8-argument `ProtoChunk`
  constructor with an empty biome registry the oracles never query.
- `VanillaBlockLandingOracle`: `getOperatorUserPermissionLevel()`
  (the short name arrived in 1.21.10), `isSingleplayerOwner(GameProfile)`
  (`NameAndId` came in 1.21.9), `isCommandBlockEnabled()` override
  added (abstract in 1.21.8), `ServerLevel` constructor takes a
  `ChunkProgressListener` (12 args), `Level.explode` has no
  `WeightedList` parameter, `RespawnData` does not exist (both
  overrides removed).
- `VanillaMovementRegistryOracle`: same `ServerLevel` constructor fix.
- `VanillaFaceVisibilityOracle`: no `BlockTags.BARS` constant (the
  bars tag came with the copper bars in 1.21.9) — the `TagKey` is
  built directly; no 1.21.8 block is in it.

## Verification

- `cargo check --workspace`: clean.
- `cargo check --workspace --all-targets`: clean, 0 errors, 0 warnings.
- `cargo test -p horizon-protocol`: **430 pass / 2 fail** (432 total —
  8 fewer than 1.21.9's 440: the removed packets' tests went with
  them). The 2 failures are the same pre-existing sandbox-network pair
  in untouched code, out of scope.
- `horizon-prepare` completes end-to-end: exit 0, "Done in 87.8 s"
  (after the oracle rework). The HORIZON_JAVA_25 datafix routing
  (v6.4) and `tools/jsr305-3.0.2.jar` both carry forward from 1.21.9.

## Harness subset

Measured locally on this VM (run 6, 2026-11-12, 1 h 18 min; prepare:
ok). **2 pass / 2 oracle / 13 fail** of 17 — the gate's
classification from `harnesses.json` (no baseline was passed, so all
15 non-passing are "new").

- **Pass:** `block_properties`, `lan`.
- **Oracle (gate):** `chunk_generation` (8 javac errors),
  `command_exec` (6).
- **Fail:** `command_parse`, `component_patch`, `datafix`,
  `entity_components`, `first_person`, `food`, `hotbar`, `item_model`,
  `light`, `options`, `resource_pack`, `save_roundtrip`,
  `sound_event`.

Seven of the thirteen "fail" entries are really oracle-compile
failures whose javac errors land in separate compile logs the gate's
classifier does not read, so the gate's oracle count (2) understates
the true one (9):

| Harness | True cause | Errors |
|---|---|---|
| `chunk_generation` | oracle compile (`ChunkPos.x/z`, `ChunkStatus.TERRAIN`, `NoiseGeneratorSettings.create`) | 8 |
| `command_exec` | oracle compile (`ContainerInput` missing, etc.) | 6 |
| `datafix` | oracle compile (`VanillaDataFixOracle`: `net.minecraft.util.filefix` does not exist in 1.21.8) | 26 |
| `first_person` | oracle compile | 25 |
| `food` | oracle compile (`net.minecraft.world.level.gamerules` does not exist) | 9 |
| `item_model` | oracle compile | 49 |
| `options` | oracle compile | 1 |
| `resource_pack` | oracle compile | 1 |
| `sound_event` | oracle compile | 3 |

The remaining six fails are real verdicts or harness/environment
causes:

- `command_parse`: real verdict — 7,546 parse mismatches
  (32,319/32,321 commands match at permission 0).
- `component_patch`: the oracle crashes at runtime (`Removed` class
  missing in 1.21.8).
- `entity_components`: harness-side Python bug (the entity predicate
  is a list in 1.21.8; `TypeError: list indices must be integers`).
- `hotbar`: real verdict — 891/905 cases match (the native side
  shows empty slots where the Rust side shows stone).
- `light`: real verdict — "light mismatch against vanilla"
  (missing motion tag).
- `save_roundtrip`: the game exits before opening the control socket
  — no display on this headless VM (the same environmental failure
  as the 26.3 baseline).

Datafix needed a second run to reach its true verdict. The first
sweep's datafix failed at fixture generation: the 26.1/26.1.2/26.2
fixture servers need Java 25 (class file 69.0) but ran under Java 21
(`UnsupportedClassVersionError`), because the local env did not set
`HORIZON_JAVA_25`. Temurin 25.0.5 exists locally at
`~/workspace/tools/jdk-25`; with `HORIZON_JAVA_25` set, the re-run
generated all 25 worlds and reached the oracle compile — one stage
deeper, the same state as 1.21.11's T4. The profile above is the
post-re-run one (datafix's shards all fail at the 26-error oracle
compile).

## What remains, by cause

### Behaviour work from the diff (not started)

The 526 ranked items are the port's work list; **0 of 526 are closed**.
The work is the protocol-shape behaviour (the removed packets' game
effects, the debug-payload system) plus the 317 reruns' differentials
and the 163 unproved items. The behaviour code in this tree is still
the 26.3 base's.

### Harness-oracle era ports

The oracle rework above covers the prepare-time exporters. The
per-harness oracles carry their own era gaps forward from 1.21.10 /
1.21.11 (chunk-generation, command-exec, first-person, food,
component-patch, entity-components) — see those versions' notes for
the per-oracle causes; the local sweep below measures which still fail
on 1.21.8.

### Full harness profile

The CI `port-verify` workflow on main completes the profile on GitHub
runners.

## Cost experiment

Cross-protocol delta, recorded for contrast with 1.21.9's
same-protocol delta (~65 min) and 1.21.10's cross-protocol delta
(~3 h 10 min). Times are reconstructed from session file timestamps
(VM clock, UTC) and are approximate to about ±10 minutes per bucket.

- **Wall clock:** work started 2026-11-12 ~03:44 UTC; [PUBLISH COMMIT
  — to be filled from the actual commit time after pushing].
- **Work items closed vs total:** 0 / 526 behaviour items closed. The
  port closed the infrastructure layer only: pin, tables, packet
  rewire, adapter re-base, oracle rework, verification. No behaviour
  item was attempted.
- **Harness profile:** 2 pass / 2 oracle / 13 fail of 17 (gate
  classification; true oracle count 9 — see above).
- **Sweep minutes:** run 6 (the profile) 1 h 18 min; datafix re-run
  with Java 25, 24 min. Three earlier runs were killed by runtime
  restarts (run 3, run 4, run 5) and their ~2 h 15 min of partial
  sweeps was repeated work, not profile.

### Where the time actually went (approximate)

| Activity | Time |
|---|---|
| Prepare / diff tooling (prepare ~6 min, diff + report ~3 min) | ~10 min |
| Table regen (3 runs: run1, run2, full run — the full run re-ran all six for the D1-complete regen.json) | ~45 min |
| Adapter re-base + serializer/codec rewire (re-base scripts ~05:24–05:29, then the 22 serializer failures and spawn-test fixes) | ~2 h 30 min |
| Verification (protocol test runs, workspace checks) | ~20 min |
| Oracle rework for prepare (7 files, 27 errors, incl. the ProtoChunk/biome-registry rework) | ~25 min |
| Harness runs / debugging (run 6 1 h 18 min, datafix re-run 24 min, three restart-killed runs ~2 h 15 min repeated) | ~3 h 40 min (of which ~1 h 40 min produced the profile) |
| Behaviour-item porting | 0 min |
| Publish (docs, notes, audit, commit) | [to be filled] |

The contrast with 1.21.9 (~65 min) is the protocol crossing: the
adapter re-base and the codec/serializer rewire dominate, and the
oracle corpus does not carry forward untouched across a protocol
version — 1.21.9's APIs (PalettedContainerFactory, NameAndId,
RespawnData, the pre/post Keyframe split) all postdate 1.21.8.
The recurring VM tax is the flaky-network truncated downloads,
fixed by seeding hash-verified copies; this port also paid a
small argparse tax (the `harnesses` names must precede `--worktree`,
and `--worktree` must be absolute or prepare doubles the path).

## Status

Pin set (protocol 772, world 4440); 6 of 6 tables regenerated; packet
set rewired (7 removed, 1 renamed, 108 renumbered); adapter ladder
re-based to 772; oracles ported to the 1.21.8 API; workspace compiles
(0 errors, 0 warnings); protocol tests 430 pass / 2 fail
(sandbox-network pair only); prepare verified ok end-to-end.
Behaviour port unstarted (0/526). Harness profile: 2 pass / 2 oracle /
13 fail of 17 measured locally (true oracle count 9; the other six
fails are real verdicts or environmental). CI port-verify to confirm.
