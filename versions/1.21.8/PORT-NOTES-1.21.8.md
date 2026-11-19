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

- **Wall clock:** work started 2026-11-12 ~03:44 UTC; end 2026-11-12
  14:24 UTC (publish commit b884d16d). Total: ~10 h 40 min.
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
| Publish (docs, notes, audit, determinism regen, commit) | ~2 h 30 min |

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

## Behaviour port (2026-11-19)

Direction 1.21.9 -> 1.21.8, 526 items (ranks 9/317/163/35/2).
Verification method: byte-compare of the named 1.21.9/1.21.8 jars
(client 2,357 changed / 127 added / 611 removed classes), then a
Vineflower decompile of the 1,660 named 1.21.9-side and 1,397
1.21.8-side class files and a `diff -r` of the two source trees
(68,444 lines, 1,032 files). The running jar is the oracle;
decompiled code is a reading aid. The report's 1,756 changed classes
is the semantic (token-hash) count; the byte set is a superset that
includes constant-pool churn.

### Rank 1: closed 9/9

- **Facts:** version.rs has PROTOCOL_VERSION 772, DATA_VERSION 4440,
  RESOURCE_PACK_FORMAT (64, 0), DATA_PACK_FORMAT (81, 0) — matches
  1.21.8's version JSON.
- **Six regenerates:** regen.json lists all six generators, all
  "unchanged" against the candidate; regen-determinism.json 6/6
  byte-for-byte.
- **Protocol:** ids.rs counts match the report's 1.21.8 counts in all
  four state/direction groups; all seven removed packets absent;
  `debug_sample_subscription` present.
- **Save format:** decompiled diff of the 10 changed datafix classes.
  1.21.9's DataFixers registers ~10 fix groups at schemas 4530-4548
  that 1.21.8 lacks (copper golem, shelf, mannequin, chain rename
  4541, world border, world spawn/respawn) — but the Rust `update()`
  only applies fixes with `k / 10 <= to_version` (4440), so those
  are inert for 1.21.8 upgrades and need no change. Four real
  divergences were fixed:
  1. `datafix/fixes_c.rs`: removed the two `add_locator_bar_style`
     registrations at key(4424, 0) — 1.21.9-only AddFieldFix at a
     schema below 4440, so it WOULD have run on 1.21.8 upgrades.
  2. `datafix/items.rs` `fix_display`: removed the strict-JSON
     filtering of custom_name/lore (1.21.9-only validation).
  3. `datafix/items.rs` `fix_enchantments`: removed the `level > 0`
     filter (1.21.9-only); the 0..255 clamp is in both versions.
  4. `datafix/schema.rs`: removed the structured `test_instance_block`
     arm (1.21.9-only); 1.21.8's V4302 is `registerSimple`.
  Verified no-ops: ChunkHeightAndBiomeFix (1.21.9's retrogen gate
  defaults false), StructuresBecomeConfiguredFix (log format only),
  SAVED_DATA_WORLD_BORDER (only used by the inert 4544 fix; 1.21.8
  has no world_border.dat file).

### Rank 2: 70/317 (items 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 26, 27, 28, 30, 31, 32, 33, 34, 35, 36, 38, 39, 41, 42, 43, 44, 45, 46, 48, 49, 50, 51, 52, 40, 47, 53, 55, 61, 62, 63, 65, 67, 69, 71, 72, 58, 59, 64, 66, 68, 76, 77, 78, 79, 83, 84, 86, 87, 88, 89, 54 + 4 zero-diff)

- **Item 54 — `crates/horizon-client/src/game/mob_blocks.rs`
  (16 classes, 44 methods, 3 named in Horizon),
  citations only:** the mob block layers
  (MushroomCowMushroomLayer, PlayerItemInHandLayer,
  CarriedBlockLayer, DolphinCarryingItemLayer,
  FoxHeldItemLayer, IronGolemFlowerLayer,
  PandaHoldsItemLayer, SnowGolemHeadLayer) +
  removed-in-1.21.8: CopperGolemModel,
  CopperGolemRenderer, ElderGuardianParticle (+
  Provider, + Group, + 2 inner classes),
  BlockDecorationLayer. 1.21.9's submit-based
  rendering (`SubmitNodeCollector`, `submit`,
  `submitBlock`, `outlineColor`, `appearsGlowing()`
  method) vs 1.21.8's render-based
  (`MultiBufferSource`, `render`, `renderSingleBlock`,
  `appearsGlowing` field) — geometry/transforms are
  IDENTICAL in every layer (verified in the diff).
  Rewords: `CarriedBlockLayer.submit` →
  `CarriedBlockLayer.render` (the matrix matches
  1.21.8's render exactly: translate 0/0.6875/−0.75,
  X 20°, Y 45°, translate 0.25/0.1875/0.25, scale
  (−0.5, −0.5, 0.5), Y 90°);
  `BlockModelRenderState.submit` →
  `BlockRenderDispatcher.renderSingleBlock`;
  `ItemInHandLayer submits` → `renders`;
  `PlayerItemInHandLayer.submitArmWithItem (avatars
  only)` → `renderArmWithItem (players only)`.
  **Apparition:** 1.21.8 has only
  `MobAppearanceParticle`; 1.21.9 has only
  `ElderGuardianParticle` + `ElderGuardianParticleGroup`
  — both implementations term-for-term identical
  (lifetime 30, fade 0.05 + 0.5·sin(age/lifetime·π),
  camera rotation, X rotation 60° − 150°·t, scale
  0.42553192 with −Y/−Z, translate (0, −0.56, 3.5),
  full-bright 15728880, NO_OVERLAY), and the Rust's
  `elder_apparition_geometry` matches; only its
  citations were reworded to `MobAppearanceParticle`.
  Horizon's own API names `submit_item_world` /
  `submit_item_unlit` stay (like `native_mob_submit`).
  Deferred: `BlockDecorationLayer` +
  `CopperGolemModel.applyBlockOnAntennaTransform` /
  `CopperGolemRenderer` / `CopperGolemModel`
  citations → item 229 (owns the golem, which does
  not exist in 1.21.8); the mannequin in the spyglass
  check → item 320; `Avatar.*` → item 196;
  `ItemInHandRenderer.submitArmWithItem` citations in
  first_person.rs → rank-3 item 355. Check 0/0 at
  `-j 1`.

- **Item 89 — `crates/horizon-client/src/vanillagen.rs`
  (2 classes, 5 methods, 4 named in Horizon), zero-diff:**
  ChunkGenerator (applyBiomeDecoration,
  createReferences, createStructures,
  findNearestMapStructure), ChunkStatusTasks. All
  differences are 1.21.9-only debug flags, off by
  default in release vanilla: `DEBUG_DISABLE_FEATURES`
  (findNearestMapStructure + decoration task),
  `DEBUG_FEATURE_COUNT` +
  `FeatureCountTracker.chunkDecorated`,
  `DEBUG_DISABLE_STRUCTURES` (createStructures). No
  debug-flag citations exist anywhere in the Rust;
  the Rust models the always-run path. The 1.21.8-only
  `DebugPackets.sendStructurePacket` in
  createReferences is debug-only and unmodelled.
  `Configuration$Global` is 1.21.9-only with no Rust
  surface. The `createStructures` body (weighted
  structure pick with `LegacyRandomSource(0)` +
  `setLargeFeatureSeed`) is otherwise identical.

- **Item 88 — `crates/horizon-client/src/mobs/panda.rs`
  (2 classes, 5 methods, 4 named in Horizon), zero-diff:**
  Panda (handleEating, handleRoll, mobInteract),
  PandaModel (setupAnim). Panda.java differs only by
  `isClientSide()` vs the field (4 sites). PandaModel
  has no decomp section — the item-25 bridge pattern,
  javap-verified from files extracted from the jars:
  5307 vs 5263 bytes (exactly 44 larger in 1.21.8),
  only difference the `setupAnim` bridge signature
  (`EntityRenderState` vs `Object`). The Rust's panda
  citations name methods identical in both versions.

- **Item 87 — `crates/horizon-client/src/game/structure_blocks.rs`
  (3 classes, 6 methods, 4 named in Horizon), zero-diff:**
  StructureBlockEntity (isStructureLoadable,
  placeStructure), StructureBlock (setPlacedBy),
  StructureTemplateManager (save). `createdBy`:
  1.21.9's `getPlainTextName()` vs 1.21.8's
  `getName().getString()` — identical for a player
  (both return the profile name; verified: 1.21.9
  `Player.getPlainTextName` returns
  `gameProfile.name()`, 1.21.8 `Player.getName()`
  returns `Component.literal(gameProfile.getName())`).
  The Rust uses the player's profile name. The
  1.21.9-only debug branches
  (`SharedConstants.DEBUG_STRUCTURE_EDIT_MODE`
  structure-void pre-fill in `placeStructure`;
  `DEBUG_SAVE_STRUCTURES_AS_SNBT` in `save`) have no
  Rust surface — no debug-flag code anywhere in the
  tree. `place_structure` places directly with no
  pre-fill; `structure_loadable` matches (LOAD mode +
  named template exists).

- **Item 86 — `crates/horizon-client/src/game/fishing.rs`
  (2 classes, 6 methods, 4 named in Horizon), zero-diff:**
  FishingHook (retrieve, shouldStopFishing, tick),
  FishingRodItem (use). The `canInteractWithLevel`
  rework (same as item 72's leash fix): 1.21.9's
  `shouldStopFishing` gates on `canInteractWithLevel`
  (alive + not removed + NOT spectator); 1.21.8's
  checks `!isRemoved() && isAlive()` with NO
  spectator check. The Rust's `tick_fishing` already
  has the 1.21.8 form: rod in either hand
  (`holds_rod`) + distance² ≤ 1024, no spectator
  check. The hooked-entity tick likewise has no
  spectator check (rides the hooked entity at 0.8
  height; unhooks to `State::Flying` when not found;
  mobs need `alive()`). No `asEquipmentSlot` /
  `getSlotForHand` citation (the Rust damages the held
  stack directly via `damage_held`); retrieve damage
  (3 for item, 5 for entity) matches. Known gap (vs
  both, not fixed): the local player's hook is NOT
  discarded on death in the Rust (no death handling
  in fishing.rs; only the LAN-guest path clears the
  hook on death) — both versions stop fishing when
  the owner dies.

- **Item 84 — `crates/horizon-client/src/game/item_entities.rs`
  (1 class, 7 methods, 4 named in Horizon), zero-diff:**
  ItemEntity (getOwner, playerTouch, setThrower, tick).
  `bobOffs` is initialised in the constructor in
  1.21.8 rather than at the field, but the draw order
  is the same in both versions (bob first, then yaw)
  and the Rust draws bob then yaw from the item's own
  random. The 1.21.8-only private copy constructor +
  public `copy()` has no in-game callers in the 1.21.8
  decomp — no Rust surface. `getOwner`/`setThrower`
  differ only by the `EntityReference` API
  (`getEntity`/`of` are 1.21.9-only; 1.21.8 has
  `get(ref, level, Class)` / `new EntityReference(entity)`)
  — the Rust stores the thrower UUID directly with no
  `EntityReference` citations. Tick merge cadence
  (every 2 ticks while moving between blocks, else 40)
  and the 6000-age discard match. Everything else is
  `isClientSide()` vs the field.

- **Item 83 — `crates/horizon-client/src/game/commands/extra.rs`
  (5 classes, 9 methods, 4 named in Horizon):**
  Component (`object`), SpectateCommand (`spectate`),
  FetchProfileCommand (removed), KickCommand
  (`kickPlayers`), VersionCommand (`dumpVersion`).
  **Spectate:** removed the 1.21.9-only
  `cannot_spectate` check — 1.21.8 has no such check,
  you can spectate any entity. **Version:** `/version`
  reported hardcoded 26.3 pack formats ("97.1" /
  "121.0"); it now uses
  `horizon_core::version::RESOURCE_PACK_FORMAT` /
  `DATA_PACK_FORMAT` (64/81), rendered as ints as
  1.21.8's `packVersion` does (1.21.9's
  `PackVersion.toString()` gives "64.0").
  **Object components:** `Component.object(ObjectInfo)`
  / `ObjectContents` do not exist in 1.21.8 (0 classes
  in the jar, 3 in 1.21.9) — removed the whole
  `InlineObject` system (the enum, the `Style` field,
  the parser and round-trip arms, the U+FFFC advance
  and glyph branches, the object-skin publication and
  inline-profile resolution, the probe fixture) and
  `tools/test_inline_object_parity.py`, which pinned
  the 1.21.9-only behaviour. Verified no-change:
  `/fetchprofile` already removed by item 22; kick
  already matches 1.21.8 (filters on the singleplayer
  owner); no `DataSource` package-path citations; the
  protocol text codec has no object handling. Check
  0/0 at `-j 1`.

- **Item 79 — `crates/horizon-client/src/game/trading.rs`
  (2 classes, 10 methods, 4 named in Horizon), zero-diff:**
  MerchantScreen (mouseClicked, mouseDragged),
  AbstractVillager (getOffers, notifyTradeUpdated).
  AbstractVillager differs only by `isClientSide()` vs
  the field (4 sites). MerchantScreen differs only by
  the 1.21.9 input rework (`MouseButtonEvent` vs raw
  doubles, `renderContents` vs `render`) — the math is
  identical and the Rust matches it term by term: the
  drag scroll `(y − 18 − 13.5) / (139 − 27) × max +
  0.5` clamped to [0, offers−7], the scroller hitbox
  (x in (94, 100), y in (18, 158], offers > 7), the
  wheel scroll `(int)(scrollOff − scrollY)` clamped,
  and the 7 offer buttons (88×20 at y 18+20i). The
  `notifyTradeUpdated` sound gate (`ambientSoundTime >
  −80 + 20`, reset to −80) matches. No
  `renderContents` or `MouseButtonEvent` citation
  exists in the Rust; the `mouseClicked`/`mouseDragged`
  citations name methods identical in both versions.

- **Item 78 — `crates/horizon-client/src/mobs/piglin.rs`
  (5 classes, 12 methods, 4 named in Horizon), zero-diff:**
  AbstractPiglinModel, HoglinModel, PiglinModel
  (setupAnim), Hoglin (getAmbientSound),
  MemoryModuleType. HoglinModel and PiglinModel differ
  only by the `setupAnim` bridge signature (1.21.8
  `setupAnim(EntityRenderState)` vs 1.21.9
  `setupAnim(Object)`, each exactly 44 bytes larger in
  1.21.8 — the item-25 bridge pattern, javap-verified
  from files extracted from the jars); no `setupAnim`
  body changed. `AbstractPiglinModel.createArmorMeshSet`
  is 1.21.9-only and has no Rust surface. Hoglin
  differs only by `isClientSide()` vs the field plus
  the 1.21.8-only debug `sendDebugPackets` (debug-only,
  unmodelled, as item 50's camel); the hoglin ambient
  sound is already activity-based
  (`HoglinAi.getSoundForCurrentActivity`). The three
  1.21.9-only memory modules (`VISITED_BLOCK_POSITIONS`,
  `UNREACHABLE_TRANSPORT_BLOCK_POSITIONS`,
  `TRANSPORT_ITEMS_COOLDOWN_TICKS`) are used only by
  the copper golem's transport AI (`transport_items.rs`,
  `CountDownTransport`, `set_transport_cooldown`), so
  their removal belongs to item 229, which owns the
  golem (as items 17/45/58 deferred its other parts).

- **Item 77 — `crates/horizon-client/src/game/native_hooks.rs`
  (3 classes, 12 methods, 4 named in Horizon):**
  BarrelBlockEntity (startOpen), ServerExplosion
  (explode, hurtEntities), Slime (remove). **Real fix:**
  the explosion's zero-distance guard. 1.21.8's
  `hurtEntities` skips an entity whose eye is exactly
  at the blast centre outright (no damage, no push, no
  `onExplosionHit`); 1.21.9's `Vec3.normalize` gives a
  zero direction and still hurts it. The Rust had the
  1.21.9 form in all three paths — `blast_hit` (mobs,
  items, orbs, carts, boats, projectiles),
  `explosion_hurt_player`, and the dragon's
  `explosion_knockback` (which also carried the 1e-5
  epsilon) — all three now skip on an exact-zero eye
  distance, and the epsilon is gone (a non-zero
  distance normalises, as 1.21.8 divides by it).
  Citations: `ContainerOpenersCounter.getEntitiesWithContainerOpen`
  → `getPlayersWithContainerOpen` (3 sites; the
  1.21.9 form is `ContainerUser`-based, and
  `ContainerUser` does not exist in 1.21.8 — 1.21.8's
  counter is player-based, with `incrementOpeners`
  taking no range). Verified no-change: slime peaceful
  despawn is already the 1.21.8 form (`setSize` clamps
  to 1..127, so `getSize() > 0` is always true and the
  unconditional slime/magma-cube entry in
  `despawns_in_peaceful` is correct); the explosion's
  1.21.9-only redirectable-projectile owner-setting is
  never modelled (the tag itself exists in both jars —
  only its use in `hurtEntities` is new); `explode`
  returns `Result<()>` in the Rust, matching 1.21.8's
  void (1.21.9's int count has no surface); primed TNT
  already overrides its explosion eye height to 0.0
  (both versions use its position, not its eye); the
  opener counting matches 1.21.8's counter (first
  opener schedules the 5-tick recheck, count clamped
  at 0, entry dropped at 0).

- **Item 76 — `crates/horizon-client/src/game/native_mob_draw.rs`
  (1 class, 13 methods, 4 named in Horizon):**
  EntityRenderer (extractRenderState, extractShadow,
  submit, submitNameTag). **Real fix:** the shadow scan
  depth. 1.21.8's `EntityRenderDispatcher.renderShadow`
  scans `min(pow / 0.5, radius)` below the feet; 1.21.9's
  `extractShadow` subtracts 1.0. The Rust had the
  1.21.9 form, so shadows stopped one block short — the
  `− 1.0` is gone. The rest of the shadow algorithm is
  identical in both versions (radius clamped to 32,
  strength faded over 256 blocks squared, per-block
  alpha falling 0.5 per block of depth, brightness > 3,
  full-collision blocks only, UVs −dx/2/radius + 0.5) —
  verified term by term against 1.21.8's `renderShadow`
  / `renderBlockShadow`. Citations: `submit` → `render`
  (the Rust API name `native_mob_submit` stays — it is
  Horizon's own), `submitNameTag` → `renderNameTag` (2
  sites), `MAX_SHADOW_RADIUS` → the literal 32 (1.21.8
  inlines it in the dispatcher's call),
  `EntityRenderer.extractShadow` →
  `EntityRenderDispatcher.renderShadow`. Verified
  no-change: the name-tag math matches 1.21.8's
  `renderNameTag` (0.025 scale, y + 0.5, deadmau5 −10,
  see-through + normal double draw, background opacity
  0.25, 4096 distance-squared gate); the leash
  extraction is identical in both versions (quad leashes
  modelled); no `finalizeRenderState`, `shadowPieces`,
  `outlineColor` or server-hitbox code exists in the
  Rust. Deferred: the `CameraRenderState` citation to
  item 32 (which owns it), `SubmitNodeCollector` to item
  214, the hitbox debug-entries gate to rank-3 item 420.

- **Item 68 — `crates/horizon-client/src/game/integrated.rs`
  (2 classes, 17 methods, 4 named in Horizon):**
  IntegratedServer (13; initServer, publishServer,
  tickPaused, tickServer), IntegratedPlayerList (4).
  Citations only. The `MAX_PLAYERS` comment now cites
  the 1.21.8 form (the literal 8 passed to the
  `IntegratedPlayerList` super-constructor;
  `IntegratedServer.MAX_PLAYERS` is 1.21.9-only). The
  pause logic is already 1.21.8's: paused is the window's
  pause only (no empty-player-list clause — that came in
  1.21.9), the pause transition saves, and the paused
  tick drains no packets (`tick_lan` skips
  `tick_lan_packets` while paused, matching 1.21.8's
  `tickPaused`, which has no `tickConnection` call);
  view and simulation distances follow
  max(2, option) with vanilla's log lines. Flight is
  allowed on the integrated server (`HubConfig`,
  `allow_flight: true`), the MOTD names the owner and
  the world, and permission levels match (owner 4/0,
  guests 2/0 per the operator level 2). The PvP gate
  reads the `pvp` game rule — 1.21.8 has no such rule
  (its `MinecraftServer.isPvpAllowed` reads the `pvp`
  field, set true by `initServer`; 1.21.9 made it read
  the rule) — but the rule's removal and its gates are
  the gamerules item's (item 222, as item 13 deferred),
  and the integrated behaviour already matches (allowed
  by default). Known gap (vs both, not fixed):
  `TOTAL_WORLD_TIME` is not awarded while paused, where
  vanilla's `tickPaused` awards it to every player in
  both versions. Deferred (correcting item 24's note,
  which named this item): the `LevelLoadListener`
  citations belong to rank-3 items 408/481, the
  `PrepareSpawnTask` citations to rank-3 item 440, the
  `NameAndId` type to rank-2 item 74, and the
  1.21.8-only `ChunkProgressListener*` classes to rank-4
  item 492. No `selectLevelLoadFocusPos`,
  `getMaxPlayers` or operator-level citations exist in
  the Rust.

- **Item 66 — `crates/horizon-client/src/lan/entity.rs`
  (2 classes, 18 methods, 4 named in Horizon):**
  ChunkMap$TrackedEntity (9; updatePlayer), ServerEntity
  (9; handleMinecartPosRot, sendChanges,
  sendDirtyEntityData). Zero-diff. 1.21.9's
  `ServerEntity.Synchronizer` interface (sendToTracking-
  Players / AndSelf / Filtered) is replaced in 1.21.8 by
  two broadcast consumers plus `broadcastAndSend`; no
  `Synchronizer` or `sendToTrackingPlayers` citation
  exists in the Rust. The passenger filter is the same
  set either way: 1.21.9's predicate (a tracking player
  receives the SetPassengers packet iff its mount status
  is unchanged) and 1.21.8's ignore list (the symmetric
  difference of the passenger lists, via
  `mountedOrDismounted`) both exclude exactly the players
  that mounted or dismounted. The 1.21.9-only
  `isTrackedByAnyPlayer`, `forEachEntityTrackedBy` and
  `forEachReadyToSendChunk`, and the `debugSynchronizers`
  hooks in `updatePlayer`/`removePlayer`, have no Rust
  surface; 1.21.8-only `ChunkMap.getChunks` likewise.
  Verified against both versions' `sendChanges` (the
  logic is identical apart from the broadcast call):
  the 7.6293945E-6 move threshold, 400-tick teleport
  delay, 60-tick position refresh, 1.0E-7 velocity
  threshold, the passenger rotation-only branch, the
  minecart substep queue (`MoveMinecartPacket`, 1.0F
  fallback weight), dirty data then attributes, and the
  ±1 packed head-rotation send. Known gap (vs both, not
  fixed): the Rust broadcasts SetPassengers to every
  tracking viewer, without excluding the players that
  mounted or dismounted as vanilla does in both versions.

- **Item 64 — `crates/horizon-client/src/game/perspective.rs`
  (5 classes, 18 methods, 4 named in Horizon):**
  PlayerCapeModel (7), WingsLayer (5; getPlayerElytraTexture,
  submit), SpinAttackEffectModel (4; setupAnim),
  GuiEntityRenderer (1; renderToTexture), IronBarsBlock
  (1). Zero-diff. The cape box is identical in both
  versions and `setupAnim`'s rotation was verified term
  by term in `mobs/render.rs::cape`; the HumanoidModel-vs-
  PlayerModel parentage is Java plumbing (the `player#cape`
  layer comes from version model data). The spin-attack
  box math matches exactly in `spin_attack_geometry`
  (age × −(45 + (i+1)×5), wrapped); only the render-state
  class was renamed (`AvatarRenderState` is 1.21.9's name
  for `PlayerRenderState` — no such citation exists in the
  Rust). `WingsLayer`'s diff is submit-vs-render plumbing
  plus the elytra/cape texture accessor forms, which have
  no Rust surface; the GUI-entity preview light is full
  bright (240|240<<8 = 15728880), matching 1.21.8's
  render call. Iron bars: the 1.21.9 BARS-tag clause is
  data-driven via `bars_tag`, which the oracle computes
  as `state.is(bars tag)` — false for every state against
  the 1.21.8 jar (the tag does not exist), so the Bars
  rule reduces to 1.21.8's same-block check; no committed
  data has `bars_tag:true`. Known gap (vs both, not
  fixed): the skin's elytra texture is never assigned to
  a mob — wings use the cape texture or the equipment
  default, so a player with a custom elytra texture
  renders with their cape texture. Deferred: the
  `Avatar.*` citations belong to rank-2 item 196 (which
  owns `world/entity/Avatar`), the `AvatarRenderer.*`
  citations to rank-2 item 310, the mannequin Avatar refs
  to item 320.

- **Item 59 — `crates/horizon-client-render/src/options_file.rs`
  (2 classes, 25 methods, 4 named in Horizon):** Options (13;
  processOptions, sprintWindow), InputConstants (12; getKey,
  isKeyDown). The option-definition cleanup this item owns
  from earlier items: `invertXMouse`, `toggleAttack`,
  `toggleUse`, `allowCursorChanges` and `saveChatDrafts`
  are 1.21.9-only, so their `Opt` variants, `Settings`
  fields, options-file table entries, screen-list entries
  and tests are all removed (9 files). Attack and use now
  pass `false` for their toggle option at every call site
  (they are plain mappings; the option was already ignored
  for them). `sprintWindow` was never modelled in the Rust,
  so there was nothing to remove. `invertYMouse` keeps its
  options-file key in both versions; its lang key is
  `options.invertMouse` in 1.21.8 (verified present in the
  1.21.8 `en_us.json`; `options.invertMouseY`,
  `options.invertMouseX`, `options.allowCursorChanges`,
  `options.chat.drafts` and `options.sprintWindow` are all
  absent there). The mouse-settings order test was stale
  since item 34 (it still expected the 26.3 six-option
  list); it now expects vanilla 1.21.8's four
  (sensitivity, invert Y, wheel sensitivity, discrete
  scroll, per `MouseSettingsScreen.options`). Real fix:
  Control-click on macOS is unconditional in vanilla
  (`MouseHandler.onPress` remaps the press and, via the
  `fakeRightMouse` counter, the release; there is no option
  in either version, and 1.21.8's `MacosUtil` has no such
  setter) — the Rust's `ctrlClickEmulatesRightClick`
  option (default false) is removed and the emulation is
  now always on on macOS. The volume-slider preview call
  is removed: 1.21.8's slider callback only updates the
  source volume, and `SoundPreviewHandler` is 1.21.9-only
  (the now-uncalled module's deletion belongs to rank-3
  item 428). Verified no-change: the key names are already
  the 1.21.8 `left.win`/`right.win` form (the 1.21.9
  LSUPER/RSUPER rename has no surface); no `MOD_*`
  constants or vanilla `KeyEvent` exist in the Rust; the
  simulation-distance minimum is already 5 (1.21.9's
  debug-only 2 does not apply); the options loader ignores
  unknown keys, so a 1.21.9 options file still loads
  cleanly (the harness's `key.spectatorHotbar` entry pins
  this). The options test run also caught two assertions
  left stale by item 34: the Accessibility test still
  expected the Controls link (1.21.9's footer button —
  1.21.8's footer has only the help link and Done), and
  the one-page-per-setting test did not know auto-jump,
  toggle-crouch and toggle-sprint repeat on
  `ControlsScreen.options` in 1.21.8 (toggleCrouch,
  toggleSprint, autoJump, operatorItemsTab, verified in
  the 1.21.8 decomp) — both fixed, 106 options tests
  pass. Deferred: the `InputQuirks` citations (the whole
  `client.input` package is 1.21.9-only) belong to rank-2
  item 238; the `sound_preview` module deletion to rank-3
  item 428.

- **Item 58 — `crates/horizon-client-render/src/block_entity_anim.rs`
  (7 classes, 26 methods, 5 named in Horizon):**
  DecoratedPotRenderer (16; extractRenderState, submit),
  ChestModel (3; setupAnim), EndGatewayBlock (2;
  entityInside), EnchantingTableBlockEntity (2),
  ChestRenderer$1 (1; removed), EnchantTableRenderState (1;
  removed), BlockTags (1). Real fix, the one item 18
  deferred here: the decorated-pot wobble style. 1.21.8's
  `DecoratedPotRenderer.render` reads `lastWobbleStyle`
  directly and takes the POSITIVE branch (the X/Z tilt) when
  POSITIVE; 1.21.9's `extractRenderState` never copies the
  style into the render state, so its `submit` always takes
  the non-POSITIVE (Y-rotation) branch. The Rust forced the
  style to non-POSITIVE at the render-state assembly; it now
  passes the real style through. The renderer already
  implemented both branches exactly (verified term by term
  against 1.21.8's `render`), and the style table is
  verified from the jar (POSITIVE ordinal 0, duration 7;
  NEGATIVE ordinal 1, duration 10; `triggerEvent(1, style)`
  sets the start tick and style — the class has no decomp
  file, so it was decompiled from the jar with Vineflower).
  The rest of the pot diff is submit-vs-render API plumbing
  (MaterialSet, SubmitNodeCollector, CrumblingOverlay are
  1.21.9-only). ChestModel differs only by the `Model<Float>`
  vs raw `Model` generics and `setupAnim(Float)` vs
  `setupAnim(float)`; the lid math is identical.
  EndGatewayBlock differs only by `isClientSide()`, and the
  enchanting table only by the `DEFAULT_NAME` constant
  (same `container.enchant` string). The nine 1.21.9-only
  block tags (`wooden_shelves`, `copper_chests`,
  `lightning_rods`, `copper`, `chains`,
  `copper_golem_statues`, `lanterns`, `bars`,
  `incorrect_for_copper_tool`) have three live Rust
  references, all deferred to their owners: `wooden_shelves`
  in the shelf renderer (rank-3 item 425 deletes it),
  `copper_chests` in the golem's item transport and
  `copper` in the golem pattern spawn (rank-2 item 229
  owns the copper golem); `lightning_rods`, `chains`,
  `lanterns`, `bars` and `incorrect_for_copper_tool` are
  unreferenced, and the `minecraft:copper` hits in
  stations.rs are the armour-trim material, not the tag.
  No `ChestRenderer$1` or `EnchantTableRenderState`
  citations exist in the Rust.

- **Item 72 — `crates/horizon-client/src/game/boat_leads.rs`
  (3 classes, 6 methods, 5 named in Horizon):** Leashable (4;
  dropLeash, getLeashHolder, setLeashedTo, tickLeash),
  LeashFeatureRenderer (1; removed in 1.21.8),
  LeashFenceKnotEntity (1; interact). Real fix: in 1.21.8
  `Leashable.tickLeash` breaks a lead only when an end is
  dead (`!isAlive()`); 1.21.9's `Entity.canInteractWithLevel`
  (alive, not removed, not spectating — the method does not
  exist in 1.21.8) also breaks it when the holder goes
  spectator. The Rust modelled the 1.21.9 form in three
  places and all three now drop the spectator condition: the
  mob leash tick (`mobs/leash.rs`), the boat holder lookup
  for a LAN guest and for the local player
  (`boat_holder`). A spectator holder now keeps the lead and
  its position still drives the rope, as vanilla's does.
  Citations: the link packet goes out through
  `ChunkMap.broadcast` / `broadcastAndSend` in 1.21.8
  (javap-verified on both jars; 1.21.9 renamed them
  `sendToTrackingPlayers` / `sendToTrackingPlayersAndSelf`),
  and leashes are drawn by `EntityRenderer.renderLeash` in
  1.21.8 — the whole `client.renderer.feature` package
  (including `LeashFeatureRenderer`) is 1.21.9-only, as item
  18 found for the crumbling overlay. The Rust's rope
  geometry matches `renderLeash`/`addVertexPair` term by
  term (24 segments, the 0.05/2 offset, the slack sag, the
  alternating 0.7 shade, the (0.5, 0.4, 0.3) colour).
  `LeashFenceKnotEntity.interact` differs only by
  `isClientSide()`. Deferred: the mannequin entity is
  1.21.9-only (its Rust references in snbt/skins/entity_nbt
  belong to rank-2 item 320, which owns the removed
  `Mannequin` class). Known gap (vs both, not fixed here):
  `drop_leash` drops a live mob's lead regardless of the
  entity-drops rule, where vanilla's tickLeash gates the
  drop on the rule whenever either end is dead; and the
  rope's light is the average of the two ends where vanilla
  lerps it per segment.

- **Item 71 — `crates/horizon-client/src/game/block_entity_anims.rs`
  (3 classes, 6 methods, 5 named in Horizon) — citations only:**
  ConduitBlockEntity (4; clientTick, selectNewTarget,
  updateAndAttackTarget), BellBlockEntity (1; updateEntities),
  TheEndGatewayBlockEntity (1; triggerCooldown). The conduit
  resolves its target through `EntityReference.get(ref, level,
  LivingEntity.class)` in 1.21.8; 1.21.9's convenience
  `EntityReference.getLivingEntity(ref, level)` (three sites)
  and the `EntityReference.of(entity)` factory in
  `selectNewTarget` do not exist in 1.21.8 (the constructor
  is used instead) — the same resolution either way, and the
  one Rust citation reworded to the 1.21.8 form. The target
  rules match: an unanswered reference is no target with no
  replacement chosen that tick, a living target inside 8
  blocks is kept, otherwise a random hostile in water or rain
  within the ±8 box is picked, and the target takes 4 magic
  damage. Bell and gateway differ only by `isClientSide()`
  (method) vs `isClientSide` (field).

- **Item 69 — `crates/horizon-client/src/game/remote_mob_use.rs`
  (2 classes, 7 methods, 5 named in Horizon) — zero-diff:**
  Armadillo (5; brushOffScute, handleEntityEvent,
  mobInteract), Sheep (2; aiStep, mobInteract). The armadillo
  brush is already the 1.21.8 form in both paths: 1.21.8's
  `brushOffScute()` spawns one scute outright and fires
  `ENTITY_INTERACT`, while 1.21.9's two-argument form drops
  from the `ARMADILLO_BRUSH` entity-interact loot table —
  both the hand interaction (`interact_armadillo`) and the
  dispenser (`dispense_brush`) already spawn one scute with
  the brush sound, the entity-interact event and 16
  durability, with comments naming the 1.21.9 table. The
  1.21.8-only `Armadillo.sendDebugPackets` (brain packet) is
  debug-only and unmodelled, as with the allay. The peek
  event (64) is modelled with its sound. `Sheep.aiStep` and
  both `mobInteract` bodies differ only by `isClientSide()`
  vs `isClientSide` and by `InteractionHand.asEquipmentSlot()`
  (1.21.9) vs `LivingEntity.getSlotForHand` (1.21.8) — the
  same hand-to-slot mapping, and no `asEquipmentSlot` exists
  in the Rust. The sheep shear (per-colour shearing table,
  shear sound, 1 damage) matches. Known gap (vs both, not
  fixed here): the hand sheep shear does not post the SHEAR
  game event that vanilla posts in both versions (the
  dispenser shear, the snow-golem/bogged shear and the
  beehive shear all post it). No check run: no files changed
  since item 67's 0/0 check (item-37 precedent).

- **Item 67 — `crates/horizon-client/src/mobs/spawning.rs` (1 class,
  8 methods, 5 named in Horizon) — citations only:** Zombie
  (finalizeSpawn, hurtServer, killedEntity, setBaby, tick).
  `setBaby`/`tick` differ only by `isClientSide()` (method)
  vs `isClientSide` (field). The reinforcement gate is
  already the 1.21.8 form: `call_reinforcements` checks the
  `spawn_mobs` rule directly (1.21.9's `isSpawningMonsters()`
  is the rule plus non-peaceful, equivalent here because the
  gate is already Hard-only), with the 50 attempts, the
  7..40 range and the ±0.05 caller/callee charges.
  `killedEntity` drops its `DamageSource` parameter in
  1.21.8 (the body is identical); the Rust's `zombie_killed`
  takes no damage source and `infects` matches (always on
  Hard, half the time on Normal). The chicken jockeys mount
  through `riders::link` (no force parameter). The
  1.21.8-only `dropCustomDeathLoot`/`getSkull` skull drop is
  already the Rust's form: `drops.rs` drops a charged
  creeper's victim's head directly (zombie head for a plain
  zombie; Husk, Drowned, Zombie Villager and Zombified
  Piglin all override `getSkull` to return empty, verified
  in the 1.21.8 decomp), with the one-head-per-explosion limit
  in `creatures.rs` (`droppedSkulls < 1`) — 1.21.9 moved the
  drop to the charged-creeper loot table instead.
  `ServerLevel.isSpawningMonsters` is 1.21.9-only
  (javap: absent from 1.21.8's `ServerLevel`); the three
  citations reworded to 1.21.8's inline form (difficulty not
  peaceful and the mob-spawning rule on, as
  `CreakingHeartBlockEntity.serverTick` checks them). Known
  gap (vs both, not fixed here): both strider-jockey paths
  (the game's `spawn_strider_jockey` and finalize's `Ctx`)
  gate the 1-in-30 zombified-piglin jockey on monsters
  spawning, but neither vanilla version gates it —
  `EntityType.create` checks difficulty only, so with
  doMobSpawning off on a non-peaceful difficulty vanilla
  still spawns the jockey and the Rust does not.

- **Item 65 — `crates/horizon-client/src/game/raids.rs` (4 classes,
  8 methods, 5 named in Horizon) — zero-diff:** VillageSiege
  (2; tick), Raid (2; spawnGroup, tick), Raids (2; tick),
  PatrolSpawner (2; tick). The spawner `tick` signature is
  three-argument in 1.21.8 (`CustomSpawner.tick(level,
  spawnEnemies, spawnFriendlies)`) and two-argument in 1.21.9,
  but both spawners read only the first boolean, and the
  Rust's patrol tick takes exactly that one (`spawn_mobs`
  and difficulty not peaceful), as the siege tick does; the
  bodies are identical apart from parameter renumbering (the
  patrol's 12000 + nextInt(1200) cadence, the 1-in-5 roll,
  the 24..48-block offset, the difficulty-based group size,
  the ±4 jitter; the siege's midnight 1-in-10 roll). The
  1.21.9-only pieces have no Rust surface: the `DEBUG_RAIDS`
  boss-bar rename in `Raid.tick`, `Raids.getRaidCentersInChunk`
  (debug-only), and the 1.21.8-only `DebugPackets.sendRaids`
  in `Raids.tick` (debug-only, unmodelled, as with the
  shulker/allay debug packets). The ravager mount in
  `spawnGroup` is `startRiding(vehicle)` in 1.21.8 vs the
  three-argument form in 1.21.9; the Rust mounts through
  `riders::link` with no force/silent parameters. Known gap
  (vs both, not fixed here): the patrol spawner has no
  day-5 gate — vanilla requires `getDayTime() / 24000 >= 5`
  before the 1-in-5 roll, so the Rust can spawn patrols
  before day 5 and consumes the roll's random draw on those
  days where vanilla draws nothing. No check run: no files
  changed since item 55's 0/0 check (item-37 precedent).

- **Item 63 — `crates/horizon-client/src/mobs/shulker.rs` (2 classes,
  9 methods, 5 named in Horizon) — zero-diff:** Shulker (6;
  setPos, setRawPeekAmount, startRiding, tick), ShulkerModel
  (3; setupAnim). `Shulker` differs only by `isClientSide()`
  (method) vs `isClientSide` (field) at five sites, plus
  `startRiding`'s signature: 1.21.9 added a third `force`
  argument that 1.21.8's two-argument form lacks, and the
  Rust's `riders::start_riding` is two-argument, so the extra
  parameter has no surface here; the override body
  (attach face back to DOWN, client teleport slide cleared)
  matches `on_start_riding`. Every named method's body is
  verified against the Rust: `tick` (a shell that rides
  nothing re-attaches or teleports when it cannot stay, then
  eases the peek and pushes on change), `setPos`
  (`client_set_pos`: block-centre snap off a vehicle, the
  `tickCount == 0` guard, the lid closing on a block change,
  the 6-tick slide from the block left), `setRawPeekAmount`
  (open/close sounds at 1.0/1.0, CONTAINER_OPEN/CLOSE game
  events, and the covered-shell +20 armor in `armor::values`
  when the raw peek is 0). ShulkerModel has no decomp diff:
  the bridge pattern again (1.21.8 exactly 44 bytes larger;
  only the bridge `setupAnim(EntityRenderState)` vs
  `setupAnim(Object)`). No check run: no files changed since
  item 55's 0/0 check (item-37 precedent).

- **Item 62 — `crates/horizon-client/src/mobs/tame.rs` (5 classes,
  10 methods, 5 named in Horizon) — zero-diff:** Wolf (4;
  aiStep, getTexture, mobInteract, tick), WolfModel (3;
  setupAnim), CatRenderer (1), CatVariants (1), WolfVariants
  (1). `Wolf` differs only by `isClientSide()` (method) vs
  `isClientSide` (field) at four sites; the shake start/cancel
  (events 8/56), the anger update and the bone-tame gate all
  match, and `getTexture` has no diff at all — the Rust's
  `texture()` (tame → `_tame`, else angry → `_angry`, else
  wild) matches it. The variant registrations differ only by
  the `ClientAsset` form: 1.21.9 wraps each path in
  `ClientAsset.ResourceTexture`, 1.21.8 passes a plain
  `ClientAsset` — the paths are identical, and no
  `ClientAsset`/`ResourceTexture` type exists in the Rust.
  The 9 wolf variants and their biome selectors match
  `WolfVariants.bootstrap` exactly (pale fallback, spotted
  savanna, snowy grove, black old-growth pine taiga, ashen
  snowy taiga, rusty jungle, woods forest, chestnut
  old-growth spruce taiga, striped badlands), as do the 11
  cat variants. WolfModel has no decomp diff: the bridge
  pattern again (1.21.8 exactly 44 bytes larger; only the
  bridge `setupAnim(EntityRenderState)` vs
  `setupAnim(Object)`). CatRenderer also has no decomp diff
  and identical javap signatures — its one changed method is
  `extractRenderState`, whose bytecode differs only by
  `assetInfo()` returning `ClientAsset` vs
  `ClientAsset$ResourceTexture` (the rest is constant-pool
  renumbering), which the decompiler normalises away. Known
  gap (vs both, not fixed here): the Rust appends `_baby`
  to a baby wolf's texture, but no `wolf_baby.png` exists in
  either jar — vanilla renders a baby wolf with the same
  texture, scaled — so a baby wolf's texture lookup misses
  and it draws nothing. No check run: no files changed
  since item 55's 0/0 check (item-37 precedent).

- **Item 61 — `crates/horizon-client/src/mobs/allay.rs` (3 classes,
  11 methods, 5 named in Horizon) — zero-diff:** AllayModel (6;
  setupAnim, translateToHand), Allay (4; aiStep, setDancing,
  tick), FlyingPathNavigation (1). `AllayModel.translateToHand`
  drops its render-state parameter in 1.21.8 (and `ArmedModel`
  loses its type parameter), but the body is identical — the
  state was unused — and the Rust's `allay_hand` already
  implements that body (root and body, translate (0, 0.0625,
  0.1875), right arm's pitch, 0.7 scale, shift (0.0625, 0, 0)).
  `setupAnim` has no decomp diff: the bridge pattern again
  (javap shows only the bridge `setupAnim(EntityRenderState)`
  vs `setupAnim(Object)`, plus 1.21.9's extra `translateToHand`
  bridge from the generic interface; 1.21.8's class is the
  smaller one here, 6693 vs 7029 bytes). `Allay.aiStep`/
  `tick`/`setDancing` differ only by `isClientSide()` (method)
  vs `isClientSide` (field); the heal-every-10-ticks
  (`passive_heal`), the dancing gate (`isEffectiveAi`, not
  panicking) and the hold clock all match. The two 1.21.8-only
  debug calls — `Allay.sendDebugPackets` (sends the brain
  packet) and `FlyingPathNavigation`'s per-tick
  `sendPathFindingPacket` — are debug-only and unmodelled, as
  with the camel in item 50; no `DebugPackets` code exists in
  the Rust. No check run: no files changed since item 55's
  0/0 check (item-37 precedent).

- **Item 55 — `crates/horizon-client/src/game/dragon.rs` (6 classes,
  22 methods, 5 named in Horizon) — citations only:**
  EnderDragonRenderer (10; submitRays), EnderDragonModel (3;
  setupAnim), DragonFireballRenderer (3), EnderDragon (3;
  aiStep), DragonSittingFlamingPhase (2; doClientTick),
  DragonLandingPhase (1; doClientTick). The renderer diff is
  all submission-API plumbing: 1.21.9's `submit` (SubmitNodeCollector
  + CameraRenderState, `submitModel`/`submitCustomGeometry`) vs
  1.21.8's `render` (MultiBufferSource + packed light,
  `renderToBuffer`/direct VertexConsumer) — the geometry and
  math are identical (base transform, death fade
  `ARGB.white(t)` = `ARGB.color(floor(t*255), -1)`, the ray
  seed 432/count/rotations/colours, the eight-segment crystal
  beam, the fireball's scale-2 billboard quad), and the Rust
  models the geometry, not the submission API. Citations
  reworded: `submitRays` → `renderRays` (2 sites),
  `submitCrystalBeams` → `renderCrystalBeams`. EnderDragonModel
  has no decomp diff: the item-25 bridge pattern again
  (1.21.8's class is exactly 44 bytes larger; the only javap
  difference is the bridge `setupAnim(EntityRenderState)` vs
  `setupAnim(Object)`). `EnderDragon.aiStep`/`onFlap`/
  `onSyncedDataUpdated` differ only by `isClientSide()` (method)
  vs `isClientSide` (field). The landing/flaming breath is
  already the 1.21.8 form: both phases spawn plain
  `DRAGON_BREATH` (1.21.9 wraps it in
  `PowerParticleOption.create(..., 1.0F)`, per item 14), and
  the Rust's breath sites and flame cloud both use the simple
  option. Known gaps (vs both, version-identical vanilla, not
  fixed here): the death dissolve is inverted — vanilla's
  `rendertype_entity_alpha.fsh` discards where the mask alpha
  is below the vertex alpha `t`, while the Rust's
  `dissolve_color` passes `1 - t` and its shader discards
  where the vertex alpha is below the mask's, dissolving the
  complementary fragments mid-death (dragon-only overlay 38);
  the fireball quad sits half a block low (vanilla's is
  centred 0.5 above the feet, the Rust's on the feet) and is
  lit full bright on both channels where vanilla forces only
  block light 15.

- **Item 53 — `crates/horizon-client-render/src/multiline_edit.rs` (3
  classes, 24 methods, 5 named in Horizon):** AbstractWidget
  (isMouseOver, mouseClicked), AbstractTextAreaWidget
  (mouseClicked), MultilineTextField (keyPressed,
  selectWordAtCursor). The only behavioural change is the
  double-click word select: 1.21.9's
  `MultilineTextField.selectWordAtCursor` (called from
  `MultiLineEditBox.onClick` on a double click) does not exist
  in 1.21.8, where `onClick` always places the cursor — so the
  Rust's `select_word_at_cursor` is gone and `MultiLineEdit::click`
  loses its `double` parameter (one non-test caller, dialogs.rs).
  Everything else in the diff is event-API plumbing or
  identical logic: 1.21.9's `isActive()` is `visible && active`,
  exactly 1.21.8's inline check, so mouseClicked/nextFocusPath/
  isMouseOver behave the same; the text area's `isMouseOver`
  (box plus the 6-wide scroll bar) is identical in both, as is
  the Rust's `hovered()`; `keyPressed`'s switch is identical
  (select-all/copy/paste/cut, Enter, word/char backspace and
  delete, word/char arrows, line up/down unless Ctrl, PageUp/
  PageDown to the ends, Home/End per line or whole text with
  Ctrl) and the Rust's `key()` matches it case for case;
  `onDrag` drag-selects in both versions (unlike the
  single-line EditBox, item 46), so `drag()` stays; the
  scrollbar press/drag/wheel match `updateScrolling`/
  `mouseDragged`/`mouseScrolled` (rate 4.0, scroller height
  clamped 32..height−8). 1.21.9's `renderScrollbar` takes the
  mouse only to request a hover cursor — the cursor API is
  1.21.9-only (item 46) and the sprites drawn are identical.
  Deferred to rank-3 item 331 (books.rs, owns
  `MultiLineEditBox.onClick`/`onDrag`): the book page's
  double-click, which calls `ui::EditBox::select_word_at_cursor`
  (a different method, on the book page's `ui::EditBox`, not
  this module's) — its removal belongs with the book page's
  click handling, not here.

- **Item 47 — `crates/horizon-client-input/src/keymapping.rs` (2
  classes, 31 methods, 5 named in Horizon):** KeyMapping (click,
  restoreToggleStatesOnScreenClosed, set, setAll),
  ToggleKeyMapping (release). 1.21.9's toggle-restore machinery
  does not exist in 1.21.8: no `shouldRestore` flag, no
  `ToggleKeyMapping.release` override, no
  `restoreToggleStatesOnScreenClosed`, no
  `shouldSetOnIngameFocus`. In 1.21.8 `release()` is private and
  just clears clicks plus `setDown(false)` — which a toggle with
  its option on ignores, so a toggled sneak or sprint stays down
  through a screen instead of dropping and being restored; the
  `KeyState` fields and methods for the restore are gone
  (`released_by_screen_when_down`, `should_restore_state_on_screen_closed`,
  `should_set_on_ingame_focus`, `ToggleKeys::screen_closed`), and
  `release()` is now clicks plus `setDown(false)` for every key.
  Only sneak and sprint are toggles in 1.21.8 (attack/use toggle
  options are 1.21.9-only): slots 2–3 of `ToggleKeys` are plain
  mappings, `reset()` no-ops for them (vanilla's
  `resetToggleKeys` touches toggles only), and `is_toggle`
  matches sneak/sprint only. `setAll` has no toggle exemption in
  1.21.8, so `restore_held_keys` no longer skips toggles with
  their option on — a held shift flips a toggled sneak on
  refocus, as vanilla's does. The key MAP is single-valued in
  1.21.8 (`Map<Key, KeyMapping>`; 1.21.9 holds a list per key),
  so `translate_all`/`translate_mouse_all`/`mappings_for` are
  gone: `translate`/`translate_mouse`/`mapping_for` drive the
  first mapping in list order, and the six call sites change one
  word each (an `Option` iterates). Vanilla's exact winner is
  the MAP entry — last in `ALL` iteration order after a
  `resetMapping` — which the Rust does not reproduce; the first
  in list order is its documented choice. Categories are plain
  strings in 1.21.8 with sort order movement, gameplay,
  inventory, creative, multiplayer, ui, misc (1.21.9's `Category`
  record orders movement, misc, multiplayer, gameplay,
  inventory, creative, spectator), and the controls-screen
  header now translates `key.categories.<name>` — the 1.21.9
  form `key.category.minecraft.<name>` has no 1.21.8 lang entry.
  `key.spectatorHotbar` is 1.21.9-only: removed from MAPPINGS
  and from the `Action` table (the order test ties them), the
  spectator menu is the hardcoded middle click
  (`MouseHandler.onMouseMiddleClick`, as `play_mouse` already
  did — only its comment cited the mapping), `middle_acts`
  drops the spectator branch, and `key.spectatorOutlines` is
  misc in 1.21.8, not spectator. Both harness oracles were
  still 1.21.9-only and never compiled against 1.21.8:
  `VanillaToggleKeysOracle` (toggleAttack/toggleUse options,
  the restore call) is ported and verified to compile against
  the 1.21.8 jar, with the driver restricted to the two toggle
  options and `close` kept as a no-op pin;
  `VanillaGameLoopOracle`'s `key` section is ported (1.21.8
  constructors, no restore/ingame ops) but the oracle as a
  whole still does not compile — its StubServer/StubPlayers
  server stubs are 26.x (permissions package, `NameAndId`,
  NotificationService, an 11-arg `MinecraftServer`
  constructor, missing `isCommandBlockEnabled`), a separate
  porting job for the game-loop harness's server area, recorded
  here, not attempted. Left alone (inherited 26.3-isms, Stream
  2's): `key.friends`, `key.toggleGui`,
  `key.toggleSpectatorShaderEffects` are in neither version's
  Options; the orphaned `toggle_attack`/`toggle_use` settings
  are item 59's. Known gaps (vs both): the MAP winner choice
  above; `KeyState` has no unbound state, so `setAll`'s
  not-UNKNOWN check is unmodelled in the probe.

- **Item 40 — `crates/horizon-assets/src/item_model.rs` (19 classes,
  51 methods, 5 named in Horizon):** CompassAngleState (10;
  calculate), ItemModelResolver (6; appendItemLayers,
  updateForTopItem), NeedleDirectionHelper (4; get),
  BuiltInRegistries (3), MissingItemModel (2),
  RangeSelectItemModel (2), CompassAngleState$CompassTarget (2) +
  CompassTarget$1–$4 (2 each), Time (2; calculate),
  Time$TimeSource (2) + TimeSource$1–$3 (2 each),
  ItemDisplayContext (2), ItemOwner (1),
  ItemOwner$OffsetFromOwner (1). `ItemOwner` is 1.21.9-only:
  1.21.8 passes `LivingEntity` (resolver, missing/range models)
  or `Entity` (compass, time, needle helper) where 1.21.9 passes
  an `ItemOwner` — the Rust already passes a plain `OwnerState`,
  so only the citations change (item_model.rs, item_draw.rs
  `owner_at`/`player_owner`). `ItemDisplayContext.ON_SHELF` is
  1.21.9-only (the shelf block): the variant is removed from
  `DisplayContext` (9 entries), the display-transform arrays
  shrink 9→8 (item_bake.rs, model.rs `DISPLAY_CONTEXTS`), and the
  item-display byte 9 / `"on_shelf"` arms go from lan_entities.rs,
  persist/display.rs and text_displays.rs; shelf.rs renders with
  `Fixed` until rank-3 item 425 deletes the renderer.
  Compass/time/wobbler math verified term-by-term against the
  1.21.8 jar (Wobbler $1/$2 decompiled with Vineflower — the inner
  classes have no decomp files): the spin hash
  (`seed * 1327217883 / 2.1474836E9`), the 0.8/0.9 factors, the
  center-based atan2, the `distToCenterSqr < 1e-5` validity, the
  local-player wobble gate and the 0.5−(y−0.25) targets all match;
  the one open question is the Spawn target's dimension — 1.21.8's
  code pairs `getSharedSpawnPos()` with the current level's
  dimension, which would point in the Nether, against observed
  behaviour (compasses spin there in every version, and no 1.21.x
  changelog lists a compass fix), so the Rust keeps the Overworld
  dimension (the 1.21.9 `RespawnData` form). The clock was frozen:
  `level_state_now` read the sun angle from the environment
  sampler, which has no 1.21.8 data (no `attributes`/`timelines`
  in 1.21.8 dimension data — the environment system is 26.x), so
  it always read the 0.0 default; it now computes
  `DimensionType.timeOfDay(day_time) * 360` directly, with the
  Nether/End fixed times (18000/6000) from the 1.21.8
  dimension-type JSONs. The same gap made `Environment::load`
  fail at startup — `AttributeMap::parse` hard-errors on the
  missing `attributes` field — so a missing field is now an empty
  map (environment.rs has no owning item; the full 1.21.8
  environment synthesis, real sky/fog/light values, is a separate
  job). The clock's random spin in the Nether/End comes from its
  item model (fallback `time` source `random`), identical in both
  versions. `BuiltInRegistries`' 1.21.9-only
  `DEBUG_SUBSCRIPTION`/`INCOMING_RPC_METHOD`/`OUTGOING_RPC_METHOD`
  registries have no Rust counterpart. Deferred: the `ItemOwner`
  citation in native_block_entity_draw.rs (rank-2 item 125 — the
  shelf offset owner is 1.21.9-only) and the `RespawnData.of`
  citation in respawn_anchor.rs (rank-2 item 93). Known gaps (vs
  both): `runs_normally` is hardcoded true in `level_state_now`
  (vanilla reads the tick-rate manager); the compass dimension
  check compares against the render level, vanilla against the
  owner's level (the same in practice); the random streams (Time
  RANDOM, compass spin) are Horizon's env stream, not vanilla's
  per-instance `RandomSource` (unobservable).

- **Item 27 — `crates/horizon-client-render/src/font.rs` (30 classes,
  96 methods, 5 named in Horizon) — zero-diff, citations only:**
  FontSet (20; computeGlyphInfo, getGlyph, getRandomGlyph),
  FontManager (15; getFontSetRaw), UnihexProvider$Glyph$1 (8),
  TrueTypeGlyphProvider (7), UnihexProvider$Glyph (6),
  TrueTypeGlyphProvider$Glyph (5), Font$GlyphVisitor$1 (5),
  Font$GlyphVisitor (4), BitmapProvider$Glyph (4),
  FontTexture (2; add), FontTexture$Node (2), BitmapProvider (2),
  UnihexProvider (2), plus 17 one-or-zero-method classes
  (GlyphBitmap, Font$Provider, FontManager$CachedFontProvider and
  its CachedEntry, FontSet$1/$2/DelayedBake/SelectedGlyphs/Source,
  UnihexProvider$Glyph$2, FontDescription and its
  AtlasSprite/PlayerSprite/Resource, TrueTypeGlyphProvider$Glyph$1
  /GlyphEntry, BitmapProvider$Glyph$1). 1.21.9 rewrote the font
  plumbing — `UnbakedGlyph`/`GlyphBitmap` replace `GlyphInfo`/
  `SheetGlyphInfo`, `SelectedGlyphs`/`DelayedBake`/`GlyphStitcher`
  replace the direct glyph/info caches, `CachedFontProvider`
  replaces the single-entry `lastFontSetCache`, and the new
  `FontDescription` (Resource/AtlasSprite/PlayerSprite, absent
  from the 1.21.8 jar) lets a style name a sprite font — but every
  observable rule the Rust models is identical: the provider
  selection and fishy rule (`computeGlyphInfo` picks the first
  provider's glyph for `any` and the first non-fishy one for the
  obfuscated pick, under both names), the bitmap/unihex
  (width/2+1, 0.5 offsets)/TrueType/space advance formulas, the
  by-width obfuscation bucketing (`ceil(advance)`, MISSING
  excluded), the 0.01 UV inset, the white and missing glyphs.
  The one semantic move, obfuscation's random source (1.21.9
  passes the `Font`'s own random, 1.21.8 uses `FontSet`'s shared
  static `RANDOM`, and 1.21.8's `Font.random` field is never read),
  is unobservable — both are unseeded single-purpose streams —
  and the Rust's per-Font stream, reseeded per frame for
  determinism, is Horizon's own either way. Citations reworded:
  `FontDescription.DEFAULT` → `Style.DEFAULT_FONT`,
  `SelectedGlyphs` → `GlyphInfoFilter` (4 sites),
  `GlyphBitmap` → `SheetGlyphInfo` (2 sites), `Font.random` →
  `FontSet.RANDOM` with the stream note. Deferred: the
  `FontDescription` handling in `component.rs`
  (`PlayerSprite`/`AtlasSprite.defaultFallback`, rank-3 item 435)
  and the `FontDescription.Resource` citation in `text.rs`
  (rank-2 item 73) — the Rust's `inline_object` JSON parsing is
  not 1.21.9's codec (which only serializes Resource), so those
  items adjudicate it. Known gaps (vs both): `fishy_advance`
  tests only the plain advance, while vanilla tests the bold
  advance too (a 31.5-advance bitmap glyph is fishy in vanilla,
  not in the Rust); the obfuscation candidates are the glyphs
  already in the atlas (or below U+0250), not every glyph of the
  font set as vanilla picks.

- **Item 52 — `crates/horizon-client/src/game/recipe_book.rs` (2
  classes, 18 methods, 6 named in Horizon) — zero-diff, citations
  only:** RecipeBookComponent (16; hasClickedOutside, keyPressed,
  mouseClicked, mouseDragged, tryPlaceRecipe, updateNarration),
  OverlayRecipeComponent (2). Every change is event-API plumbing
  or identical logic: the overlay's `mouseClicked` only changes
  signature; the search hint is gray+italic in both versions
  (1.21.9's `SEARCH_HINT_STYLE` is GRAY+ITALIC, 1.21.8 styles
  ITALIC+GRAY, and the Rust renders gray+italic); the render Ctrl
  freeze is `hasControlDown` vs `Screen.hasControlDown`, same
  check; `tryPlaceRecipe` drops its shift parameter in 1.21.8 but
  reads the same Shift state inside; `hasClickedOutside` gains a
  seventh parameter 1.21.8 never uses, and the Rust's
  `clicked_outside` matches the rect math exactly (147-wide strip,
  selected-tab hover); `isSelection` and `CommonInputs.selected`
  are the identical key set (257/32/335). The 1.21.9-only
  `mouseDragged` (drag-select in the search box) was already
  removed by item 38 — no drag references remain. Citations:
  `isSelection` → `CommonInputs.selected`, and the `try_place`
  doc notes where 1.21.8 reads Shift. Known gaps (vs both):
  Escape on the inventory always closes the container in the
  Rust, while vanilla closes only the book on Escape in a narrow
  window; the alternatives overlay closes on ANY click in the
  Rust, while vanilla returns false on a miss and lets the click
  fall through to the book; the chat key that focuses the search
  box is hardcoded to T in the Rust, while vanilla uses the
  rebindable keyChat mapping.

- **Item 51 — `crates/horizon-client/src/game/debug_keys.rs` + the
  `horizon-client/src/debug_keys.rs` engine (5 classes, 18 methods, 6
  named in Horizon):** KeyboardHandler (14; charTyped, getClipboard,
  handleDebugKeys, keyPress, setClipboard, tick), DebugEntryFps (1;
  removed), DebugEntrySimplePerformanceImpactors (1; removed),
  DebugEntrySystemSpecs (1; removed), DebugEntryTps (1; removed).
  The whole `net.minecraft.client.gui.components.debug` package
  exists only in 1.21.9. Fixes:
  1. **F3+B and F3+G are ungated.** 1.21.8 toggles hitboxes
     (`EntityRenderDispatcher.setRenderHitBoxes`) and chunk borders
     (`DebugRenderer.switchRenderChunkborder`) with no player or
     reduced-info gate; 1.21.9 gates both on `player != null &&
     !reducedDebugInfo` and routes through the debug entries. The
     `if player` guards are gone (and the two actions leave the
     fallthrough arm); the entries' flags stay as the Rust's store
     for the two vanilla flags — the registry rework is item 308's.
  2. **Debug keys only run in game.** 1.21.8 runs `handleDebugKeys`
     only under `$$17` (no screen, or a pauseless pause screen);
     1.21.9 runs it whenever F3 is held, whatever the screen. The
     engine now gates the dispatch on `handles_global`, and `$$17`
     no longer includes the game-mode switcher (1.21.9-only). F3
     released over another screen neither toggles the overlay nor
     spends the modifier — 1.21.9 toggles regardless, so releasing
     F3 to confirm a game-mode switch also flipped the overlay.
     Tests rewritten: the behind-a-screen test now asserts nothing
     runs and nothing toggles, plus a switcher-release test; the
     refused-action test now refuses CopyLocation (hitboxes no
     longer refuse without a player).
  3. **The FPS line is 1.21.8's fpsString.** 1.21.8 renders `"%d fps
     T: %s%s%s%s B: %d%s"` on one line (limit, vsync, graphics
     mode, clouds, biome blend); 1.21.9 splits the last three into
     `DebugEntrySimplePerformanceImpactors`. The performance
     content is merged back into `debug_fps_line` (vsync from the
     setting, graphics mode from `graphics_preset` —
     `GraphicsStatus.toString` is lowercase fast/fancy/fabulous,
     javap-verified; Horizon's `custom` prints as it stands —
     clouds, `B: {blend}`); `debug_performance_lines` and its
     dispatch arm are removed, with them the Horizon-only
     "Filtering: None" and "Terrain Rendering:" lines. The present
     mode, refresh rate and frame-generation additions stay, and
     the GPU part stays empty, as in vanilla without a frame
     profiler.
  4. **The charts footer says "For help: press F3 + Q".** 1.21.8's
     footer; 1.21.9's "To edit: press F3 + F6" pointed at the
     debug-options screen, gone in 1.21.8.
  5. **F6 does not interrupt the game-mode switcher.**
     `canInterruptScreen` is 1.21.9-only; in 1.21.8 F6 over the
     switcher does nothing. The interrupt branch is removed; F3+F6
     in game still opens the debug options screen, which stays as
     Horizon debug UI per item 16 (item 468 owns the screen).
  Citations reworded: `DebugEntryFps` → `Minecraft.fpsString`;
  `DebugEntryTps` → `DebugScreenOverlay.getGameInformation` (the
  TPS line is identical in both versions — 1.21.8's third
  game-information line, not 1.21.9-only as first assumed);
  `DebugEntrySystemSpecs` →
  `DebugScreenOverlay.getSystemInformation` (same lines in both);
  `DebugScreenEntries.toggleDebugOverlay` →
  `DebugScreenOverlay.toggleOverlay`.
  Verified no-change: 1.21.8's `handleChunkDebugKeys` is dead code
  (defined, never called anywhere in the client), so the Rust's
  lack of chunk-key actions matches both versions' release builds;
  F3+F1 toggles hideGui with F3 held — the Rust's flow already
  does that, matching 1.21.8 against 1.21.9's else-if chain;
  F3+C/F3+I/F3+N/F3+F4 match 1.21.8's reduced-info and
  permission-2 checks (CopyLocation returns false when reduced,
  CopyRecreate always returns true, Spectate falls back to the
  previous mode or Creative); the TPS line's content matches
  1.21.8's exactly, as does the system-specs content; the crash
  countdown (10 s, message at 1 s, warnings each second) matches;
  the `key.debug.*` mappings exist in NEITHER version — both
  vanilla handlers switch on raw key codes, so the Rust's
  mapping-based dispatch (a 26.x model) stays, its defaults
  matching vanilla's codes. The `simple_performance_impactors`
  registry entries stay (they now dispatch to nothing) for item
  308. Known gaps (vs both): F3+Q help is unimplemented (no Help
  action in the Rust, so the footer advertises a dead key); the
  profiler digit keys have no pie chart to drive
  (Profiling/ProfilingChart are stubs); F1 over a pauseless pause
  screen does not toggle hideGui in the Rust (vanilla toggles
  under `$$17`); the F4 post-effect toggle is unmodelled
  (F3+F4 opens the switcher but never toggles the effect);
  F3+4 lightmap and F3+X improved transparency exist in neither
  version (26.x additions, out of the diff's scope);
  `charTyped`'s surrogate-pair split is a Java UTF-16 artifact
  with no Rust equivalent. Deferred: `version_lines` (item 83's
  file, `game/commands/extra.rs`) hardcodes 26.3 pack versions
  97.1/121.0 — 1.21.8's are 64.0/81.0.

- **Item 49 — `crates/horizon-client/src/lan/conn.rs` (6 classes, 20 methods, 6 named in Horizon):**
  ServerConfigurationPacketListenerImpl (11; addOptionalTasks,
  onDisconnect, startConfiguration, tick),
  ServerCommonPacketListenerImpl (3; handleResourcePackResponse),
  ServerLoginPacketListenerImpl (3; handleHello),
  ServerConnectionListener$2 (1), ServerLoginPacketListenerImpl$1
  (1), ServerCodeOfConductConfigurationTask (1; removed). Fixes:
  1. **The server-side code of conduct is gone.** 1.21.9's
     `ServerCodeOfConductConfigurationTask` and
     `DedicatedServer.readCodeOfConducts` / `getCodeOfConducts`
     (which `addOptionalTasks` called) do not exist in 1.21.8 — the
     whole `net.minecraft.server.network.config` package is absent
     from the 1.21.8 decomp. Item 13 had already cut the wiring and
     the configuration flow already skips the task; this item
     removes the dead `lan/code_of_conduct.rs` module (`read` +
     `pick` + tests, zero callers) and its `pub mod` declaration,
     plus the dangling `ServerCodeOfConductConfigurationTask` doc
     line on `wait_alive`. The client-side prompt is rank-3 item
     378's; the unread `code_of_conduct` server.properties field
     is rank-3 item 483's.
  2. **The configuration log names the profile honestly.** 1.21.9's
     configuration `onDisconnect` logs `name (uuid)`; 1.21.8 logs
     the GameProfile object itself, which authlib 6.0.58 renders
     with a `ToStringBuilder` (`com.mojang.authlib.GameProfile@
     <hash>[id=...,name=...,properties=...]`, verified by
     decompiling the 1.21.8-pinned authlib jar) — its identity hash
     no Rust rendering can reproduce. The six config-phase
     `lost()` call sites now share a `profile_log` helper that
     renders `name (uuid)` and documents why.
  Verified no-change: the login `onDisconnect` is identical in both
  versions (`getUserName()` = name + loggable address, which
  `user_name` matches); `handleHello`'s singleplayer-profile,
  online, forwarding and offline branches match (the name
  comparison is case-insensitive in both); `authenticate`'s
  `hasJoinedServer` log and refusal texts are identical (the
  `getSessionService()` / `services().sessionService()` split has
  no Rust citation); `try_admit`'s duplicate wait matches
  `disconnectAllPlayersWithProfile` + `WAITING_FOR_DUPE_DISCONNECT`
  (by uuid in both); the configuration flow is already 1.21.8's —
  brand, server links, features, known packs, registry data, tags,
  resource pack, finish — with no PrepareSpawnTask anywhere
  (1.21.9's `returnToWorld`/`spawnPlayer` split has no Rust
  equivalent; placement is the hub's, item 24); `tick` is
  keep-alive only in the Rust, as in 1.21.8 (1.21.9 ticks the
  current task and disconnects with `configuration_error`, which
  has no Rust equivalent); keep-alive 15 s and read timeout 30 s
  match; the resource-pack decline log and disconnect reason
  match; `startMemoryChannel` verified present in the 1.21.8 jar
  (javap — the outer class file is missing from the decomp);
  `ServerConnectionListener$1` (ReadTimeoutHandler) is unchanged
  between versions; the `DEBUG_FAKE_LATENCY_MS` / LatencySimulator
  debug pipeline has no Rust equivalent; `NameAndId` is unused by
  conn.rs (the Rust's own type lives in horizon-server-files for
  user lists, item 24's note stands).

- **Item 46 — `crates/horizon-client-render/src/ui.rs` (2 classes, 22 methods, 6 named in Horizon):**
  EditBox (19; charTyped, findClickedPositionInText, keyPressed,
  onClick, onDrag, setHint), SpriteIconButton$TextAndIcon (3). Fixes:
  1. **No cursor API.** 1.21.9's `CursorTypes` / `GuiGraphicsExtractor.
     requestCursor` / `AbstractWidget.handleCursor` do not exist in
     1.21.8 (no `CursorTypes` in the jar). Removed the whole
     machinery: `widgets::Cursor`, `request_cursor`,
     `widget_cursor`, `take_cursor` and their tests, every call site
     (chat links, creative tabs, pack screen, settings shell,
     dialogs, title buttons, `ui::button`), `Game::apply_cursor_icon`
     and the `cursor_icon` field, `Creative::tab_wants_hand` and its
     test, and the now-dead `Game::key_focus_driving`. The window
     always shows the arrow. The `Opt::AllowCursorChanges` option
     definition stays — item 59 (Options) owns it, per items 21/34/41.
  2. **No double-click word select, no drag select.** 1.21.9's
     `findClickedPositionInText` / `selectWord` / `onDrag` are
     1.21.9-only; 1.21.8's `EditBox` has only `onClick` (move the
     cursor, Shift extends) and `AbstractWidget.onDrag` is an empty
     no-op. Removed `Caret`'s `last_click`/`held`, `press`'s 250 ms
     double-click detection, `Caret::drag`/`release`,
     `EditBox::click`'s `double` parameter and word branch,
     `EditBox::drag_to`, `Title::drag_field`/`release_field` and
     their call sites (game.rs, split.rs, startup_menu.rs), and the
     pinning test assertions. `EditBox::select_word_at_cursor`
     stays: it is `MultilineTextField.selectWordAtCursor` for book
     pages, not the single-line path.
  3. **The insert bar is fixed light grey.** 1.21.8's `renderWidget`
     draws it with the literal `-3092272` (`CURSOR_INSERT_COLOR`,
     0xFFD0D0D0); 1.21.9 draws it in the text colour. The bar now
     uses `rgba(208, 208, 208, 255)` in `append_field` and
     `EditBox::draw`, and `-3092272` in `TextField::draw`
     (screen_draw.rs); the "_" keeps the text colour in all three,
     as in both versions.
  Citations: `SpriteIconButton.withTootip` → dropped (the pause
  menu's icon row is Horizon's own; vanilla's `PauseScreen` uses
  plain link buttons in both versions);
  `EditBox.findClickedPositionInText` → `onClick`'s inline measure;
  `EditBox.extractWidgetRenderState` → `EditBox.renderWidget`;
  `AbstractSliderButton.extractWidgetRenderState` →
  `renderWidget`; `TextCursorUtils.isCursorVisible` and
  `setInvertHighlightedTextColor` exist in NEITHER version (26.x
  names) → the blink is `renderWidget`'s inline `(millis -
  focusedTime) / 300 % 2 == 0`, the highlight is
  `GuiGraphics.textHighlight` (invert pass + blue fill).
  Verified no-change: the single `format: Option<&Text>` passed to
  `EditBox::draw` is already 1.21.8's single-formatter model (not
  1.21.9's `TextFormatter` list); hints are explicitly styled
  wherever the Rust sets them (pack screen's gray-italic search
  hints, settings shell's muted hint), so `setHint`'s restyle of
  empty-style hints never applies; the Rust has no `WidgetSprites`
  (icon buttons use a single sprite, as in 1.21.8);
  `TextField::click` already takes only (x, extend);
  `position_at`'s clamp-then-measure gives the same boundary as
  1.21.8's truncate-then-measure; the `insert` rule (`cursor < len
  || len >= max`) and the suggestion colour (-8355712) match
  `renderWidget` exactly. Known gap (vs both): the book screen's
  double-click word select (`book_last_click`, `select_word_at_cursor`)
  belongs to the multiline item, as do the `GuiGraphicsExtractor`
  citations in debug_chart/cartography/inventory_slots/loom/pvp.

- **Item 36 — `crates/horizon-core/src/paletted.rs` (5 classes, 49 methods, 6 named in Horizon):**
  PalettedContainer, Palette, Strategy, Strategy$1, Strategy$2 (the
  top-level 1.21.9 `Strategy` and its two implementations; 1.21.8 nests
  them as `PalettedContainer.Strategy` with anonymous `1`/`2`).
  1.21.9's refactor moved the strategy out of the container and the
  registry out of the palette; 1.21.8 threads the `IdMap` through both.
  Fixes: citations only, plus one dead variant.
  Citations: `Strategy.createForBlockStates` →
  `PalettedContainer.Strategy.SECTION_STATES`,
  `Strategy.createForBiomes` → `...SECTION_BIOMES`,
  `getConfigurationForBitCount` → `Strategy.getConfiguration`
  (paletted.rs, horizon-protocol chunk.rs),
  `getConfigurationForPaletteSize(17)` →
  `calculateBitsForSerialization(17)` (horizon-world chunk.rs test
  doc). Removed the dead `PaletteError::BadBits` variant: it described
  1.21.9's declared-bit-count validation, which `from_packed` never
  performed — 1.21.8's `unpack` computes the bits from the palette size
  and has no `bitsPerEntry` to validate against.
  Verified no-change (thresholds compared term by term, the 1.21.8
  anonymous strategies decompiled from the jar with Vineflower since
  they have no decomp files): block states 0 → single, 1–4 → linear 4,
  5–8 → hashmap, >8 → global with `ceillog2(registry)` bits; biomes
  0 → single, 1–3 → linear with the requested bits, >3 → global —
  identical in both versions, and the Rust's `Strategy::config` matches
  both. The wire form (bits byte + palette + longs) and the NBT form
  (palette list + long stream) are identical; `pack` computes the same
  saved width in both versions (global packs use `ceillog2(palette
  size)`); `unpack` re-encodes global containers in both (1.21.9's
  `Configuration$Global.alwaysRepack()` is true) and reuses the storage
  directly for non-global ones; `onResize` rebuilds with the config
  for bits+1, copies contents in index order, and adds the new value
  last, as the Rust's `resize` does; `getSerializedSize` is `1 +
  palette + raw * 8` in both. The Rust's wire-global `max(registry
  bits, declared bits)` is defensive only — on valid input the declared
  bits are the registry bits.

- **Item 34 — `crates/horizon-client-render/src/options_ui.rs` (15 classes, 51 methods, 6 named in Horizon):**
  KeyBindsList (+CategoryEntry, +KeyEntry), AbstractContainerWidget,
  LanguageSelectScreen (+LanguageSelectionList$Entry),
  AccessibilityOptionsScreen, KeyBindsScreen, VideoSettingsScreen,
  OptionsList (+Entry), ConfirmScreen, LanguageSelectScreen,
  MouseSettingsScreen, ControlsScreen, OptionsScreen. Fixes:
  1. **Accessibility list gains three options, loses the Controls link.**
     1.21.8 lists auto-jump (after high contrast) and toggle-crouch /
     toggle-sprint (after view bobbing); 1.21.9 moved those to the
     Controls screen and put a Controls button beside the narrator
     instead. The list now matches 1.21.8's 27 options in order, and
     the page renders plain pairs.
  2. **Controls list loses toggle-attack and toggle-use** (1.21.9-only
     options, absent from 1.21.8's `Options` entirely; the sprint
     window was already gone via item 12). Horizon's own quit shortcuts
     and the macOS Control-click switch stay.
  3. **Mouse list loses Invert X and the cursor-changes switch**
     (1.21.9-only), reordered to sensitivity, Invert Y, wheel
     sensitivity, discrete scroll — 1.21.8's order.
  4. **Two defaults sharing a key conflict.** 1.21.9 exempts a pair
     when both are at their defaults; 1.21.8 flags any same key.
     `conflicts` drops the exemption. New test
     `two_defaults_sharing_a_key_conflict`: the F3 debug keys (overlay
     + modifier, both default F3) each flag the other.
  5. **Duplicate-key marks are red, not yellow** — the `[ key ]`
     message (`ChatFormatting.RED`) and the 3-wide bar (`-65536`).
  Verified no-change: the confirm screen's Escape always answers no
  (the Rust has no delay ticker, so 1.21.9's delay gate is unmodelled);
  the panorama spins on every screen except during onboarding (the
  Rust has no per-screen `panoramaShouldSpin` override, matching
  1.21.8); keybind category headers are plain centred text, not
  focusable (1.21.9's `FocusableTextWidget` headers are); categories
  are plain strings, as in 1.21.8; the OptionsList layout (width/2 −
  155, 160 spacing) and the key-entry layout are identical; the video
  screen's GPU-warning click and Ctrl+scroll GUI scale, the keybinds
  capture (Escape unbinds, mouse click binds), and the container
  widget's mouse handling differ only by the event API
  (`MouseButtonEvent`/`KeyEvent` vs raw values).
  Known gap (vs both versions, not 1.21.8-specific): the touchscreen
  option is unmodelled — it is in both versions' mouse lists (last in
  1.21.8, second in 1.21.9). Ownership: the option definitions
  (`Opt::ToggleAttack`, `Opt::ToggleUse`, `Opt::InvertX`,
  `Opt::AllowCursorChanges`, the Invert Y lang key
  `options.invertMouseY` → 1.21.8's `options.invertMouse`) belong to
  item 59 (Options); their settings fields and the round-trip test stay
  until then.

- **Item 50 — `crates/horizon-client/src/game/mounts.rs` (4 classes, 9 methods, 7 named in Horizon):**
  Camel, Strider, Pig, FoodOnAStickItem. Zero-diff: no code changed.
  Verified no-change: `Pig` differs only by the `isClientSide` field vs
  method; the camel's 1.21.8-only `sendDebugPackets` override (sending
  the entity-brain debug packet) is debug-only and unmodelled in the
  Rust — no citations; the strider's `getAmbientSound` loses only a
  `@Nullable` annotation, and its `spawnJockey` `startRiding(this,
  true, false)` → `(this, true)` is the 1.21.9-only third argument
  (`force`, false by default) — the Rust mounts the jockey via the
  generic `mount_mob` link with no force model and no citation of the
  3-arg form; `FoodOnAStickItem.use` switches
  `InteractionHand.asEquipmentSlot()` (1.21.9-only) for
  `LivingEntity.getSlotForHand(hand)` with the identical result
  (MAIN_HAND → MAINHAND, OFF_HAND → OFFHAND), and the Rust cites
  neither name (the steering works off main/off-hand items directly).

- **Item 48 — `crates/horizon-client/src/game/llamas.rs` (2 classes, 10 methods, 7 named in Horizon):**
  AbstractHorse, Llama. Zero-diff: no code changed.
  Verified no-change: 1.21.9's `AbstractHorse.addPassenger` override
  (snapping the rider's rotation to the horse's view rotation on any
  passenger add) does not exist in 1.21.8, and the Rust never modelled
  it — both `start_riding` paths (`Game::start_riding`,
  `riders::start_riding`) link without snapping, so the Rust already
  behaves like 1.21.8; the `doPlayerRide` yaw set at the mount call
  sites (mounts.rs, llamas.rs) matches both versions. `setOwner` is
  behaviour-identical (`EntityReference.of(null)` returns null, exactly
  the 1.21.8 ternary) and the Rust stores the owner as a plain UUID
  with no `of` citation. `handleEating`'s food values, the temper and
  love rules, `standIfPossible`, `makeMad`, `openCustomInventoryScreen`
  and `Llama.handleEating` differ only by the `isClientSide` field vs
  method. Known gap (vs both versions, not 1.21.8-specific):
  `doPlayerRide` sets the player's pitch too (`setXRot`), while the
  Rust sets only the yaw on mounting. Deferred: the `EntityReference.of`
  citation in `game/tnt.rs:531` belongs to item 99 (rank 2, tnt.rs).

- **Item 45 — `crates/horizon-client/src/mobs.rs` (6 classes, 14 methods, 7 named in Horizon):**
  Piglin, RemotePlayer, BellBlock, EntityModel, AvatarRenderState
  (1.21.9-only, removed), ContainerUser (1.21.9-only, removed). Fixes:
  1. **A swimming remote player still bobs on the ground.** 1.21.9's
     `RemotePlayer.aiStep` calls `AbstractClientPlayer.updateBob`, which
     zeroes the bob target while swimming; 1.21.8's `RemotePlayer.aiStep`
     never calls `super.aiStep()` — it inlines its own bob update with
     no swimming check (`onGround() && !isDeadOrDying()`), while the
     shared `Player.aiStep` (the local player's path) keeps the check.
     `tick_avatar_pose` no longer excludes swimming; the local player's
     call site (fov.rs) keeps `!swimming`. `update_bob`'s parameter is
     renamed `on_ground_alive` (the caller decides the condition) with
     the split documented. New test
     `a_swimming_remote_player_still_bobs_on_the_ground` (swimming pose,
     on the ground, speed 0.1 → bob 0.04 after one tick, bob_o 0).
     The oBob timing is the same in both versions (`bobO = bob` before
     the ease, in 1.21.9 inside `ClientAvatarState.updateBob`).
  2. **Citation:** the copper golem's `openedChestPos` now notes
     `ContainerUser` is 1.21.9-only and neither the golem nor the
     interface is in 1.21.8 (unreachable dead code; item 229 owns the
     golem). `AvatarRenderState` has no remaining citations (item 35
     reworded the only one).
  Verified no-change: the piglin head drop is already the 1.21.8 form
  (item 17: `drops.rs` gives a piglin killed by a charged creeper a
  piglin head, with the shared skull cap); the piglin stays out of
  `despawns_in_peaceful` (its `shouldDespawnInPeaceful` returns false,
  while PiglinBrute despawns); the baby-piglin hoglin stack
  (`getTopPassenger`, 3) is the same in both versions (only the
  `startRiding` signature's third argument differs); `EntityModel`'s
  `setupAnim` calls `resetPose()` in both versions (1.21.8's own
  `EntityModel.setupAnim`, 1.21.9's generic `Model.setupAnim`) and the
  Rust resets all poses before each frame's setup (`reset_all_poses`,
  `Model.resetPose` in both jars); `BellBlock` differs only by the
  `isClientSide` field vs method; `lerpMotion`'s Vec3-vs-triple form is
  a signature change with the same body.

- **Item 44 — `crates/horizon-client/src/entities.rs` (7 classes, 15 methods, 7 named in Horizon):**
  ExperienceOrbRenderer, LavaCauldronBlock, Direction, ExperienceOrb,
  ItemUtils, NameTagItem, PowderSnowBlock. Zero-diff: no code changed,
  citations only.
  Citations: `ExperienceOrbRenderer.submit` → `.render` (entities.rs ×2,
  game.rs); `Direction.axisStepOrder` → `Entity.axisStepOrder`
  (entities.rs, falling_block.rs) — 1.21.9 moved the method to `Direction`
  and made it public; 1.21.8 keeps it as `Entity`'s own private static
  (Entity.java:1125-1127) with the identical rule (|x| < |z| → YZX, else
  YXZ), used by both `collideWithShapes` and `checkInsideBlocks` (16
  iterations) in both versions, so the Rust's order logic is already the
  1.21.8 form.
  Verified no-change: the orb render math is identical (same icon UVs,
  sin colour, 0.3 scale, camera billboard — only submit/render plumbing
  differs); `ExperienceOrb.tick`, `ItemUtils.onContainerDestroyed`,
  `NameTagItem.interactLivingEntity` and `PowderSnowBlock.entityInside`
  differ only by the `isClientSide` field vs method; the lava cauldron
  already matches 1.21.8 (item 26 removed CLEAR_FREEZE — 1.21.9-only —
  and `lava_cauldron_inside` applies LAVA_IGNITE then lavaHurt, with
  `ignite_for_ticks` clearing the freeze; the analog output is 3 in both
  versions, the Direction parameter only changes the signature).
  **Ownership — the `BlockGetter.forEachBlockIntersectedBetween` rewrite
  belongs to item 178 (rank 2, `horizon-core/src/geometry/sweep.rs`),
  NOT here:** 1.21.9 rewrote the sweep (1e-5 threshold, direction-ordered
  start/end boxes via the 1.21.9-only `BlockPos.betweenCornersInDirection`,
  furthest-corner DDA), while 1.21.8 visits `betweenClosed` under a
  0.99999 threshold with a min-corner DDA and no start-box visit — and
  the Rust's `sweep_intersected_blocks` models the 1.21.9 form
  (`furthest_corner`, `(1e-5)^2` threshold, `between_corners_in_direction`).

- **Item 43 — `crates/horizon-client/src/game/minecarts.rs` (5 classes, 25 methods, 7 named in Horizon):**
  AbstractMinecartRenderer, TntMinecartRenderer, MinecartTNT, BaseSpawner,
  Minecart. Fixes:
  1. **No spawner gamerule.** 1.21.9's `spawnerBlocksEnabled` gamerule
     (`GameRules.java:192-193`, category MISC, default true, read via
     `MinecraftServer.isSpawnerBlockEnabled`) does not exist in 1.21.8:
     `BaseSpawner.serverTick` gates on `isNearPlayer` alone. The Rust
     modelled it as the `spawner_blocks_work` gamerule with gates in the
     spawner tick (`game/spawner.rs`, shared by block-entity spawners and
     spawner minecarts via `server_tick_spawner_of`) and in
     `mobs/spawner.rs`'s `server_tick`. Removed the rule definition
     (`gamerules.rs`), the new-world default (`save.rs` `GAME_RULES`,
     58 → 57 entries), both gates, the `SpawnerLevel::spawner_blocks_work`
     trait method and its two implementations (`natural_spawning.rs`, the
     natural-spawn probe fixture, whose `spawners_work` field and
     `spawners` case parameter went with it). The trial spawner's gate
     (`trial_spawner.rs` `can_spawn_in_level`) went with the rule too: it
     was wrong against both versions — neither version's `TrialSpawner`
     checks the rule. The datafix rename (`fixes_c.rs`,
     `spawnerBlocksEnabled` → `minecraft:spawner_blocks_work`) stays: a
     1.21.9 world's rule renames to a name the registry no longer knows,
     which loads inert, exactly as vanilla 1.21.8 drops unknown rules on
     load — and removing it would risk the D4 fixture outputs. All gates
     and the definition went in one commit: `rules.bool` returns false
     for an unknown rule, so removing the definition first would have
     stopped every spawner.
  2. **Citation:** `submitWhiteSolidBlock` → `renderWhiteSolidBlock`
     (minecarts.rs). `newRender`/`newExtractState` exist in both
     versions and stay.
  Verified no-change: the TNT minecart's collision/fall explosion loses
  its primer attribution in 1.21.8 (`explode(double)` → `explode(null,
  double)`; 1.21.9 passes `ignitionSource`), but the Rust never modelled
  the primer — `explode_cart` attributes every blast to the cart — so
  the change is unobservable here; the fuse-expiry path still uses
  `ignitionSource` in both versions, and the power formula
  (`4.0 + random * 1.5 * min(sqrt(speed), 5)`) matches. The TNT flash and
  swell match 1.21.8 exactly: `fuseRemainingInTicks = fuse - partialTick
  + 1.0` (TntMinecartRenderer.java:53), flash `(int)fuse / 5 % 2 == 0`,
  swell `1 + (1 - fuse/10)^4 * 0.3`. The minecart render is identical
  (same transform, display-block scale/offset, `isNewRender` split — only
  submit/render plumbing differs); `Minecart.interact`/`tick`/
  `positionRider` differ only by the `isClientSide` field vs method;
  `BaseSpawner`'s spin and delay logic is unchanged (the `getOSpin` →
  `getoSpin` rename has no Rust citations). `SpawnEggItem` also reads
  `isSpawnerBlockEnabled` in 1.21.9; the Rust never gated spawn eggs on
  the rule, which is the 1.21.8 behaviour. Known gaps (vs both versions,
  not 1.21.8-specific): the Rust never records who primed a TNT cart, so
  fuse-expiry and flaming-arrow explosions are unattributed where vanilla
  credits the primer; a primed cart with `tnt_explodes` off is stuck at
  fuse 0 instead of being discarded (`explode` discards a primed cart
  when the rule is off); the TNT flash reads the integer fuse where
  vanilla reads `(int)fuseRemainingInTicks`, which differ by one at
  exactly partial tick 0.

- **Item 42 — `crates/horizon-client/src/save.rs` (8 classes, 29 methods, 7 named in Horizon):**
  SerializableChunkData, PlayerDataStorage, ThreadedLevelLightEngine,
  RegionFileStorage, SectionStorage$PackedChunk, SimpleRegionStorage,
  DataVersion, BakedSheetGlyph$EffectInstance (1.21.9-only, removed).
  Zero-diff: no code changed.
  Verified no-change: the three 1.21.9-only debug flags
  (`DEBUG_DONT_SAVE_WORLD`, `DEBUG_OPEN_INCOMPATIBLE_WORLDS`,
  `DEBUG_VERBOSE_SERVER_EVENTS`) are unmodelled anywhere in the Rust —
  correct for 1.21.8, and `incompatible_version` already implements the
  plain series check; the chunk tags `SerializableChunkData.write`
  produces are identical (only the codec source differs — 1.21.9's
  `PalettedContainerFactory` vs 1.21.8's registry + static codecs, same
  strategies and defaults), and the Rust cites no factory; the chunk
  decode keeps air/plains defaults for missing sections, as both versions
  do; `upgrade_record` already uses the 1.21.8 defaults (POI 1945, others
  −1) and the 1.21.8 `getVersion` reading; the player load reads `.dat`
  then `.dat_old`, as both versions do; `BakedSheetGlyph$EffectInstance`
  has no Rust citations (the `EffectInstance` hits are all
  `MobEffectInstance`); the `isLightOn` load path matches (the 1.21.9-only
  bit is a debug log line). The datafix's read-time `DataVersion` stamp
  (`update_chunk`/`update_region_tag`) is the 1.21.9 form
  (`addCurrentDataVersion` — 1.21.8's `upgradeChunkTag` does not stamp),
  but it is unobservable in the game flow (writes stamp fresh; the
  in-memory tag's version is never re-read) and is pinned by the D4
  datafix harness, so it stays. Known gaps (vs both versions, not
  1.21.8-specific): the Rust has no `PackedChunk.versionChanged` dirty
  flag — an upgraded POI chunk is re-saved only when its POIs change,
  while vanilla marks it dirty on load and re-saves it at the next flush;
  the player load writes no `_corrupted_` backup when `.dat` fails to
  parse; the player datafix runs only via the bulk upgrader (older
  level.dat), not per load, so a player file older than its level.dat is
  never upgraded.

- **Item 35 — `crates/horizon-client/src/mobs/anim.rs` (12 classes, 40 methods, 7 named in Horizon):**
  HumanoidModel, PlayerModel, CreeperModel, QuadrupedModel, BoggedModel,
  DonkeyModel, DrownedModel, EndermanModel, EquineSaddleModel, SheepModel,
  DrownedRenderer, HumanoidMobRenderer. Zero-diff: no code changed, citations
  only.
  1. **Citations:** `AvatarRenderState.isSpectator` → `PlayerRenderState`
     (mobs.rs; `AvatarRenderState` is the 1.21.9 name for the player render
     state). `HumanoidModel.ArmPose.SPEAR` → 26.x-only: no `SPEAR` constant
     exists in either version (javap: the enum is identical, 10 constants
     ending in `BRUSH`; the spear pose is `THROW_SPEAR`, which the Rust
     implements as `ThrowTrident`). The spear arm pose, the STAB swing and
     the `Hum.spear` fields are unreachable dead code in 1.21.8 — no spear
     item exists in the 1.21.8 items table (0 matches), so `Kinetic::of`
     never matches and every swing is the default WHACK — left in place per
     the item-17/18/19 pattern, with the comments now saying so; 1.21.8's
     `HumanoidMobRenderer.getArmPose` returns EMPTY.
  Verified no-change: the `setupAnim` bodies are identical in both versions
  (no hunks inside them); the arm-pose formulas were checked term by term
  against 1.21.8's `poseRightArm`/`poseLeftArm`/`poseBlockingArm` (BLOCK's
  −0.9424779 and the −4π/9 clamp, BOW's ±0.4, THROW_SPEAR's `x*0.5−π`,
  SPYGLASS's 1.9198622, TOOT_HORN's −1.4835298, BRUSH's `x*0.5−π/5`);
  the 8 classes with no decomp diff are the item-25 bridge pattern (each
  exactly 44 bytes larger in the 1.21.8 jar); `createArmorMeshSet` /
  `ArmorModelSet` are 1.21.9-only with no Rust citations, and the Rust
  cites no `DROWNED_ARMOR` layer names (the 1.21.8 tables were
  regenerated); `copyPropertiesTo` (1.21.8-only) has no Rust equivalent to
  rename — armor poses flow through the shared `Poses`; the CustomHeadLayer
  constructor change (no render cache in 1.21.8) is plumbing only.

- **Item 31 — `crates/horizon-client/src/item_render/special.rs` (12 classes, 53 methods, 7 named in Horizon):**
  BannerRenderer, PlayerHeadSpecialRenderer, ShieldSpecialRenderer,
  BannerFlagModel, SkullModelBase, SpecialModelRenderer (+ Unbaked,
  BakingContext$Simple — the last three are 1.21.9-only and gone from the
  1.21.8 jar).
  1. **Only a shield's base plate foils:** 1.21.8's `renderPatterns` draws
     the base plate through the foil buffer and the base colour and every
     pattern layer through the plain buffer (`renderPatternLayer`), so a
     patterned shield's patterns never shimmer. The Rust gave every layer
     `Foil::Standard`, citing `patternedShieldGlint` — a RenderType that
     exists in NEITHER version (a 26.x-ism). The base-colour and pattern
     layers now draw with `Foil::None`; the base draw (handle + plate) keeps
     the foil. The shield test now pins
     `[(shield_base, Standard), (base, None), (stripe_bottom, None)]`.
  2. **A head item without a skin ready shows slim steve:** 1.21.8's
     `PlayerHeadSpecialRenderer` falls back to its default render info
     (`DefaultPlayerSkin.getDefaultSkin()`, slim steve) whenever the
     profile is unresolved or its textures are not unpacked yet; 1.21.9's
     `getOrDefault` falls back to the profile's UUID-hash default instead.
     `head_skin` now returns `Option<String>` (`None` until the skin is
     ready) and the PlayerHead arm's existing slim-steve fallback finally
     applies; the tab-list path (remote.rs) keeps the UUID-hash default.
     New test `a_head_without_a_skin_ready_shows_slim_steve`.
  3. **Citations:** `submit` → `render` (module doc, `submit` fn doc,
     `submit_banner_result`'s `submitPatterns` → `renderPatterns`, the
     Banner arm, the shield test doc); `SkullModelBase.setupAnim` now cites
     the 1.21.8 `(float, float, float)` form. The `submit`/`submit_banner_result`
     fn names stay (internal names, per the item-19 precedent).
  Verified no-change: the banner render is identical (same transform, same
  flag formula, same 16-layer cap, same `entitySolid`/`entityNoOutline`
  types — only the submit/render plumbing differs); 1.21.9's
  `Model.setupAnim` resetPose is unobservable for the flag (its only
  animated property is set unconditionally right after); the skull pose
  formulas are unchanged (the concrete skull models are not in the diff);
  `getExtents` is unchanged; the head's `entityTranslucent` render type is
  the same in both versions; the banner phase formula is identical (1.21.9
  only adds a null-level guard that never triggers in practice).

- **Item 30 — `horizon-client-render/src/skins.rs` (12 classes, 65 methods, 7 named in Horizon):**
  ResolvableProfile, SkinManager, SkinTextureDownloader, SkinManager$1,
  SkinManager$TextureCache, PlayerSkinRenderCache (+ $1, $2, $RenderInfo),
  ClientAsset$ResourceTexture, PlayerModelType, ResolvableProfile$Partial
  (the last seven are 1.21.9-only and gone from the 1.21.8 jar).
  1. **A profile without an id has the nil id, not the offline UUID:**
     1.21.9's `createPartialProfile` derives the offline UUID from the name
     when the id is absent; 1.21.8's `createGameProfile` defaults to
     `Util.NIL_UUID`. `parse_profile` (bare-name and compound branches) now
     returns id 0, and `SkullSkin::of`'s fallback is `default_skin(id or
     nil)` — a name-only head falls back to slim alex (the nil-UUID hash),
     not the name-dependent skin, and not slim steve (the old no-id
     fallback). New test `a_name_only_head_falls_back_to_slim_alex`; the
     existing `profiles_parse_like_resolvable_profiles` now pins id 0.
  2. **No skin patch:** 1.21.9's `PlayerSkin.Patch` (a profile's `texture`
     field overriding the skin with a resource-pack texture) does not exist
     in 1.21.8 — the record's codec is name/id/properties only. Removed
     `patch_skin`, `profile_texture_nbt`, `valid_texture_id`, `resample_64`,
     the `SkullSkin.texture` field, the patch checks in `head_skin` and
     `skull_skin_at`, and the Downloader's whole pack-texture machinery
     (`resource_root`/`resources`, `set_resource_root`, `resource()`, the
     `again()` `patch/` branch) plus its two call sites (game.rs, packs.rs)
     and the `a_texture_patch_stays_inside_the_packs` test.
  3. **Citations:** `PlayerModelType.SLIM` → `PlayerSkin.Model.SLIM`;
     `SkinManager.createLookup` → `PlayerInfo.createSkinLookup`
     (`getOrLoad` + the secure rule); `PlayerSkinRenderCache.getOrDefault`
     → `SkullBlockRenderer` via `SkinManager.getInsecureSkin`;
     `getInsecureSkinInformation` (a 26.x name, in neither version) →
     `getInsecureSkin`; `ProfileResolver.Cached` → `SkullBlockEntity`'s
     `profileCacheByName`/`profileCacheById`; `Dynamic`/`create`/`Partial`
     → the record's `isResolved()` / `createGameProfile` (skins.rs,
     block_entity.rs, block_entity_anims.rs, item_draw.rs, game.rs,
     remote.rs, entity_nbt.rs, entity_data.rs). The `entity_data::Profile`
     patch fields stay as unread mannequin-model fields, reworded.
  Verified no-change: the secure rule is identical (only its location
  differs — 1.21.8 `PlayerInfo.createSkinLookup`, 1.21.9 `createLookup`),
  and the Rust's callers already pass the right flag (local player false,
  others true); the resolver caches are identical (256 entries, 10-minute
  expiry, same `getByName` then `getProfileById` endpoints); the
  unresolved rule is identical (no properties and exactly one of name/id);
  `processLegacySkin` is the same conversion in both versions (only the
  static/instance call site differs); `PlayerSkin.textureUrl` is never
  read in 1.21.8; the profile wire codec is already the 1.21.8 form (the
  `optional_profile_fields` quirk, `read_optional_profile`); `DefaultPlayerSkin`
  is identical (item 18). Known gap (vs both versions): the client resolver
  never consults a local profile cache (`usercache.json`) before asking the
  services, so name resolution always hits the network; the usercache exists
  only on the server side.

- **Item 41 — `horizon-client-input/src/mouse.rs` (2 classes, 19 methods, 8 named in Horizon):**
  MouseHandler, MacosUtil.
  1. **No X inversion:** 1.21.9 added `invertMouseX` (`turnPlayer` flips
     `xo` when set); 1.21.8 inverts Y only (`invertYMouse`). Removed
     `invert_x` from `LookOptions`, `turn_player` (yaw is always `+xo`
     now) and `motion_for_turn`, plus the `look_options()` construction,
     the game_loop probe's `ix` read (the 1.21.8 oracle ignores it too)
     and the test literals. Per the item-21 precedent, the orphaned
     option definition (`Opt::InvertX`, `settings.invert_x`, the
     `invertXMouse` options.txt key) is left for item 59 (Options) — it
     is now read nowhere.
  2. **Citations:** `InputQuirks.RESTORE_KEY_STATE_AFTER_MOUSE_GRAB` →
     `!Minecraft.ON_OSX` (pointer_capture.rs grab comment; the quirk
     equals `!ON_OSX`, verified in 1.21.9's InputQuirks).
  Verified no-change: the spectator middle-click already matches 1.21.8
  (middle-click as a spectator opens/uses the menu instead of firing
  pick-item — 1.21.9 dropped `onMouseMiddleClick` entirely and fires the
  key mapping); the double-click is per-screen only in the Rust (no global
  `MouseHandler` tracker — the 1.21.8 form, item 38); the grab key-restore
  and the macOS Ctrl+click emulation are the same in both versions (the
  quirk flags equal `!ON_OSX` / `ON_OSX`); `onScroll`/`onMove`/setup/drop
  are `Window.handle()` vs `getWindow()` renames with identical bodies;
  the sensitivity/smoothing/scoping turn math is unchanged (only the
  invert lines differ); `MacosUtil`'s `Window` → `long` params are API-only.
  For other items: item 59 (Options) owns the `Opt::InvertX` /
  `settings.invert_x` / `invertXMouse` definition removal, the InvertY
  lang key (`options.invertMouseY` → 1.21.8's `options.invertMouse`), and
  the `ctrlClickEmulatesRightClick` option — it exists in NEITHER version
  (vanilla emulates Ctrl+click unconditionally on macOS) and the Rust's
  defaults it to false, so macOS Ctrl+click does nothing by default.

- **Item 39 — `game/boats.rs` (6 classes, 23 methods, 8 named in Horizon):**
  AbstractBoatRenderer, ModelLayers, AbstractBoat, AbstractBoatModel,
  BoatRenderer, AbstractChestBoat.
  1. **Boat boarding is the 1.21.8 instanceof gate, not the tag:** 1.21.8
     keeps `WaterAnimal`s, players and creaking off boats; 1.21.9 replaced
     the checks with the `cannot_be_pushed_onto_boats` tag (which the 1.21.8
     jar does not have at all, so the Rust's tag lookup excluded nothing).
     javap-verified: `WaterAnimal` is the `AbstractFish` subtree only (cod,
     salmon, pufferfish, tropical fish, tadpole) — dolphins, squid and
     guardians are NOT water animals (`AgeableWaterCreature` extends
     `AgeableMob`, `Guardian` extends `Monster`), so they board in 1.21.8
     while the 1.21.9 tag keeps them (and elder guardians) off. New
     `species_boards_boat` in boat_passengers.rs with the six-species
     exclusion, replacing the tag lookup; players were already only pushed,
     never boarded. New test `fish_tadpoles_and_creaking_never_board`.
  2. **Citations:** `AbstractBoatRenderer.submit` → `.render`,
     `BoatRenderer.submitTypeAdditions` → `.renderTypeAdditions`
     (boats.rs). The `extractRenderState` citation stays (item 19's rule).
  Verified no-change: the boat render transform is identical (translate
  0.375, rotY 180−yRot, scale −1/−1/1, rotY 90; the water patch hides unless
  under water, rafts have none); `AbstractBoatModel` is byte-identical in
  decomp and javap shows only the item-25 bridge difference (44 bytes, the
  `EntityRenderState` vs `Object` `setupAnim` target); the out-of-control
  eject (60 ticks), bubble column, `interact` and `remove` are
  `isClientSide` renames only; `AbstractChestBoat.stopOpen`'s
  `ContainerUser` → `Player` is API-only (same CONTAINER_CLOSE event, no
  Rust citation); the `ModelLayers` armor renames are the item-28 pattern
  (same geometry — `ArmorModelSet` vs per-entity INNER/OUTER_ARMOR); the
  1.21.9-only layers (COPPER_GOLEM*, BREEZE_EYES, CREAKING_EYES,
  VILLAGER/ZOMBIE_VILLAGER_NO_HAT, WARDEN_*) have no Rust layer citations
  (the breeze wind texture path is the same file in both versions).
  For other items: the copper-golem render code stays unreachable dead code
  per the item-17/18 pattern (item 229 owns the golem).

- **Item 38 — `crates/horizon-client/src/container_screen.rs` (2 classes, 25 methods, 8 named in Horizon):**
  AbstractContainerScreen, AbstractRecipeBookScreen.
  1. **A release that empties the hand ends the double-click chain:**
     1.21.8's `mouseReleased` ends with `lastClickTime = 0` when the carried
     stack is empty (after the early returns for a cancelled drag and
     `skipNextRelease`), so after placing a stack a quick shift-click on the
     same slot quick-moves only that slot; 1.21.9 tracks the double-click
     globally in `MouseHandler` (same button + < 250 ms, no reset), which is
     what the Rust had. New `Release::End` (the release ran to the end and
     clicked nothing) distinct from `Release::Nothing` (an early return);
     both release paths clear `last_inv_click` when the release ran to the
     end and the hand is empty afterwards. New test
     `a_release_with_an_empty_hand_runs_to_the_end`.
  2. **No drag-selection in the recipe-book search box:** 1.21.9's
     `AbstractRecipeBookScreen.mouseDragged` delegates to
     `RecipeBookComponent.mouseDragged`, which drags the focused search
     box's selection (`EditBox.onDrag`); 1.21.8 has neither override —
     `AbstractContainerScreen.mouseDragged` swallows every drag — so a drag
     never reaches the search box. Removed `recipe_book_drag` /
     `recipe_book_release` and the `search_drag` flag (click-to-place the
     cursor still works; `drag_to` stays as the shared `EditBox.onDrag`
     primitive).
  3. **Citations:** `MouseHandler`'s `doubleClick` → the screen's own
     `lastClickTime`/`lastClickButton` (container_screen.rs double-click doc
     + test doc).
  Verified no-change: the double-click rule itself (same slot + same button
  + < 250 ms, `None` for a click outside) already matches 1.21.8, and the
  triple-click protection (clearing after PICKUP_ALL / QUICK_MOVE_ALL) was
  already there; a book-handled press leaves the chain alone in 1.21.8
  (the Rust returns before `double_click`, matching); the container
  background is already the transparent one (`render_transparent_background`,
  not the 1.21.9 `super.renderBackground`); `hasClickedOutside`'s button
  parameter is unused in both versions (including the recipe-book override);
  key handling (inventory close, hotbar swap, pick-item clone, drop with
  ctrl) and the quick-craft drag are the same logic, API only; the shift /
  control detection (`hasShiftDown` vs the event) reads the same keys.
  Known gaps (pre-existing vs both versions, not fixed): extra mouse
  buttons (4/5) bound to hotbar/offhand keys do nothing over containers
  (`checkHotbarMouseClicked` is unmodelled); touchscreen container input
  (`clickedSlot`/`draggingItem`/quick-drop) is unmodelled — both paths are
  identical in the two versions.

- **Item 37 — `crates/horizon-client/src/block_entity_item.rs` (4 classes, 17 methods, 9 named in Horizon):**
  ItemStack, Spawner, BlockItem, JukeboxBlock. Zero-diff: no code changed.
  `place()` already matches 1.21.8's `updateCustomBlockEntityTag` (the item's
  data merges only when its `id` names the placed type — a missing or
  unparseable id skips the merge, as 1.21.8's `parseEntityType` returning
  null does — op-only types only for a game master); `op_warning` matches
  1.21.8's `shouldPrintOpWarning` (no warning when the id does not parse),
  and the OP_ONLY sets are identical in both versions (6 block entity types,
  3 entity types, verified against both decomps); `carries_record` matches
  `JukeboxBlock.setPlacedBy` (EMPTY contains no `RecordItem`); `pick()`
  writes the `id` into the compound, the 1.21.8 `CustomData` form (the
  1.21.9 `TypedEntityData` wire form is items 257/170's); the spawner
  tooltip matches (`Spawner.getSpawnEntityDisplayName`'s SpawnData → entity
  → id chain, gray, spawner and trial spawner alike, after BLOCK_STATE);
  the Rust renders neither of 1.21.9's new tooltip lines (PROFILE,
  ENTITY_DATA); `hurtAndBreak`'s hand→slot mapping is the same in both
  versions (`getSlotForHand` vs `asEquipmentSlot`, item 17's rename); the
  jukebox comparator output is direction-independent in both versions (item
  16's `getAnalogOutputSignal` finding).
  Known gap (real 1.21.8 difference, not fixed here): `PlayerHeadItem.
  verifyComponentsAfterLoad` — the only override in 1.21.8 (the base is a
  no-op) — resolves a head's profile eagerly on load, so an id-only player
  head gains its name (`%s's Head`) after the fetch; 1.21.9 dropped the calls
  and the Rust resolves head profiles lazily at render (`skins::head_skin`),
  so an id-only head stays "Player Head". Load-time resolution would be a
  Game-layer feature across the inventory-load paths; only command-created
  id-only heads are affected (name-carrying heads and skins already work).
  For other items: the `TypedEntityData.loadInto` citations in
  mobs/spawner.rs:274 and mobs/trial_spawner.rs:401,404 belong to items 151
  and 192; the protocol-layer TypedEntityData sites (item_sync.rs,
  horizon-protocol) belong to items 257/170 (item 17's finding).

- **Item 32 — `horizon-client-render/src/camera.rs` (5 classes, 31 methods, 9 named in Horizon):**
  GameRenderer, AbstractClientPlayer, Frustum, CameraRenderState (1.21.9-only,
  removed), Camera.
  1. **`Player.moveCloak` snaps AND eases:** 1.21.8 snaps the cloak to the
     player from 10 blocks away and then UNCONDITIONALLY eases a quarter of
     the original gap, so after a teleport the cloak overshoots 25% past the
     player (d=20 → cloak = pos+5) and decays back geometrically, pinning the
     cape at max flap/lean for several ticks; 1.21.9's
     `ClientAvatarState.moveCloak` snaps OR eases, mutually exclusive. The
     Rust's `AvatarBob::tick` had the 1.21.9 if/else; the ease is now
     unconditional. A fresh cloak starts at the origin in both versions, so
     the first tick at spawn snaps the whole drop and eases a quarter of it
     past the player — pinned by the new test
     `a_teleport_overshoots_the_cloak` (the walk/cape test now starts the
     cloak at the player's position). The fix reaches remote players too:
     `mobs.rs` ticks `extra.avatar` for them.
  2. **No deferred subtitles:** 1.21.9 defers the subtitles over an in-game
     screen (`Gui.renderDeferredSubtitles`, drawn from
     `Screen.renderBackground`) and keeps them under F1; 1.21.8 draws them in
     the HUD pass (`Gui.renderSubtitleOverlay`) under any open screen, and F1
     hides them with the rest of the HUD. Removed `Subtitles::Deferred` and
     the `in_game_ui` helper (`Screen.isInGameUi` is 1.21.9-only) — the rule
     is now HUD shown → Now, F1 → Hidden — and the F1 branch in game.rs; the
     test is reworked to `the_subtitles_follow_the_hud`.
  3. **Citations:** `ClientAvatarState` → `Player`'s walk/bob/cloak fields
     (module doc, `AvatarBob`, `tick`, `resetBob` → `Player.rideTick`,
     `updateBob` → `Player.aiStep`, `addWalkedDistance` → the direct
     `walkDist +=` in `LocalPlayer.move`, game.rs avatar field, fov.rs
     `tick_avatar`, perspective.rs, mobs.rs avatar tick and field doc);
     `AvatarRenderer.extractCapeState` → `PlayerRenderer.extractCapeState`;
     `CameraRenderState.entityRenderState` → the camera entity itself
     (`BobState`); `AvatarRenderer.renderHand` → `PlayerRenderer.
     renderRightHand`/`renderLeftHand` (frame_geometry.rs).
  Verified no-change: the bob-model formulas are identical in both versions
  (only the location differs — `ClientAvatarState` vs `Player.aiStep`/
  `rideTick`/`AbstractClientPlayer.tick`); the cape math is identical
  (`PlayerRenderer.extractCapeState` vs `AvatarRenderer.extractCapeState`,
  same constants); `FOG_DISTANCE_SCALE` (1/12) is declared in 1.21.8's
  `Camera` but never used anywhere; post-effect clearing is equivalent (the
  Rust's single Option is the 1.21.8 form); the Rust models neither of
  1.21.9's new `Frustum` methods (`offset`, `pointInFrustum`); the debug
  crosshair gate is the same in both versions (`showDebugScreen()` includes
  `!hideGui` in 1.21.8); `LevelRenderer.tick`'s stale-crack cleanup (every
  20 ticks, > 400 ticks) is the same in both versions, only the call site
  moved (`Minecraft.tick` vs `GameRenderer.tick`); the first-person hand
  renders identically (`PlayerRenderer.renderHand`, zRot ±0.1F);
  `renderItemInHand` differs only by variable renumbering; the culling
  projection is the same math (`max(fov, options.fov)`); `getFieldOfViewModifier`
  is not in the diff.
  For other items: the tab list flips a Dinnerbone/Grumm head even when the
  player's cape is hidden (scoreboard.rs local path, remote.rs remote path)
  — 1.21.8's `LivingEntityRenderer.isEntityUpsideDown` does not; the
  PlayerTabOverlay item should consult `MobExtra.upside_down_name` (item
  28), which already encodes the full rule; tab_list.rs:345 still cites
  `AvatarRenderer.isPlayerUpsideDown`. names.rs:919 still cites
  `AvatarRenderer.scale` (0.9375, verified 1.21.8 `PlayerRenderer.scale` in
  item 19). The `Hud.extract*`/`extractBackground` citations (hud_layout,
  hud_overlays, cartography, loom, screens, dialogs, command_blocks,
  game.rs) belong to the HUD/screen items. The debug-entries system stays
  as Horizon debug UI per item 16's decision.

- **Item 28 — `game/armor_stands.rs` (9 classes, 49 methods, 9 named in Horizon):**
  HumanoidArmorLayer, ArmorStandRenderer, LivingEntityRenderer,
  ArmorStandArmorModel, ArmorStandModel, EquipmentLayerRenderer,
  AbstractMinecart, ArmorStand, AbstractSkullBlock.
  1. **Upside-down names (`LivingEntityRenderer.isEntityUpsideDown`):**
     1.21.8 compares the formatting-stripped display name
     (`ChatFormatting.stripFormatting(getName().getString())`) — a mob
     named `§6Dinnerbone` hangs upside down, and a PLAYER flips by its
     profile name (unless its cape is hidden, `PlayerModelPart.CAPE`,
     bit 0 of the skin customisation); 1.21.9 compared the custom name
     exactly, so neither case flipped. New `upside_down` in
     `mobs/render.rs` (custom name first, stripped; else the player's
     profile bit with the cape gate) and `MobExtra.upside_down_name`,
     set for the local player in `tick_player_model`, for remote
     players in `spawn_entity` (from the list entry AddEntity just
     filed) with a PlayerInfoUpdate backfill for a late entry. LAN
     guests come through the same remote path (the host window is a
     client of its own server thread), so they are covered too. New
     test `a_formatted_dinnerbone_mob_hangs_upside_down` (compiles
     under check; runs on CI).
  2. **Citations:** `ArmorStandRenderer.submit` → `.render`,
     `LivingEntityRenderer.submit` → `.render` (armor_stands.rs).
  Verified no-change: the armor-model mesh is identical (only the
  factory changed, `createArmorLayerSet`/`ArmorModelSet` vs
  `createBodyLayer`); `translateToHand`'s body is identical (signature
  only); the armor-layer part visibility is the same set in both
  versions (HEAD head+hat, CHEST body+arms, LEGS body+legs, FEET legs);
  `renderLayers` draws the same layers, colours and first-layer-only
  glint in both; `setupAnim` runs before the body in 1.21.8 and inside
  the layer block in 1.21.9, but the Rust poses before drawing in all
  paths so the observable result is the same; `shouldRenderLayers`'
  extra `!layers.isEmpty()` in 1.21.9 is a pure optimisation;
  `ArmorStand`/`AbstractMinecart.push`/`AbstractSkullBlock` are
  `isClientSide` renames only; `lerpMotion` is the Vec3-vs-triple
  form of the same body (item 26's pattern); the `extractRenderState`
  `customName`/`appearsGlowing` fields are state plumbing, same values.
  Known gap (pre-existing vs both versions, not fixed): the Rust glints
  armour trims when the stack has foil, but vanilla draws the trim
  without foil in both versions (the glint pass is first-layer-only).

- **Item 33 — `game/potions.rs` (3 classes, 18 methods, 10 named in Horizon):**
  Projectile, AreaEffectCloud, PotionItem.
  1. **A deflect drops an owner that no longer resolves (closes the item-26
     gap):** 1.21.8's `Projectile.deflect` takes the resolved owner —
     `hitTargetOrDeflectSelf`, `onHit`'s AIM_DEFLECT, `AbstractArrow.
     onHitEntity` (refusal) and `ThrownTrident.onHitEntity` (bounce) all
     pass `getOwner()` — so an owner unresolvable in the current level is
     dropped, UUID and all; 1.21.9 passes the bare `owner` reference,
     which survives. New `resolved_owner` (the owner's (Who, UUID, guest)
     as it resolves now — a UUID-only owner links to a guest who has
     since joined, else a mob by UUID, like `claim_owner`) and
     `drop_unresolved_owner` in `game/deflect.rs`, applied at the breeze
     REVERSE site, the AIM_DEFLECT site (resolve first, then the look,
     so a UUID-only owner aims too), `finish_arrow_hit` (refusals and
     trident bounces, after the events loop so the shoot event still sees
     the pre-deflect owner), and the thrown-potion breeze deflect
     (thrower, name and guest all dropped). The player-attack deflect
     needs nothing (the player is present).
  2. **Citations:** `isOutsideOwnerCollisionRange` → `checkLeftOwner`
     (the 1.21.8 name; the 1.21.9 name was the private helper);
     `EntityReference.getEntity` → `EntityReference.get` (arrows.rs
     `OwnerLookup`, creatures.rs `player_with_uuid`) — javap confirms
     1.21.8's static `get(ref, UUIDLookup, Class)` with public
     constructors, while `of`/`getEntity(ref, Level)`/`getLivingEntity`/
     `getPlayer` are 1.21.9-only; `store`/`read` exist in both.
  Verified no-change: the thrown-potion leftOwner check runs before the
  move, like `Projectile.tick` in both versions (the check's position in
  `Projectile.tick` is identical; only `AbstractArrow`'s early call and
  the `leftOwnerChecked` guard differ — item 26); `leftOwnerChecked` was
  never modelled (the Rust's `if !left_owner` guard is the 1.21.8 form);
  `onDeflection`'s extra entity arg is unused (only
  `AbstractHurtingProjectile` overrides it, and it uses only the boolean);
  `PotionItem.useOn` already awards the stat (the 1.21.8 form; 1.21.9
  dropped it), once — the caller returns at once; `AreaEffectCloud`
  owners are same-level `Who`s by construction; `setRadius` is the
  `isClientSide` rename.
  Known gaps (pre-existing vs both versions, not fixed): the arrow
  world-border bounce is unmodelled (`AbstractArrow`/`FishingHook`
  return true for `shouldBounceOnWorldBorder`; arrows.rs has no border
  handling); `onDeflection(false)` halving a fireball's acceleration is
  unmodelled (only the 0.1 case is implemented).
  For other items: item 196 (`TamableAnimal.die` — combat.rs
  `tell_owner_pet_died` cites the 1.21.9 all-dimensions `getEntity`;
  1.21.8 resolves same-level only); item 71 (block_entity_anims.rs
  `getLivingEntity` citation); item 399 (commands/execute.rs
  `getLivingEntity` citation).

- **Item 22 — `game/commands.rs` (12 classes, 76 methods, 10 named in Horizon):**
  Level, EntitySelectorParser, WorldBorderCommand, LocalCoordinates,
  SetSpawnCommand, SetWorldSpawnCommand, LevelData, CommandSourceStack,
  Commands, SummonCommand, ListPlayersCommand, AbstractMinecartContainer.
  1. **`/spawnpoint` and `/setworldspawn` take an angle, not a rotation:**
     1.21.8's commands take `AngleArgument` (a single yaw, default 0.0F,
     relative to the source's yaw, wrapped via `Mth.wrapDegrees`);
     1.21.9's take `RotationArgument` (yaw + pitch) with `RespawnData`.
     The trees now use `ArgType::Angle` (arg `angle`); both handlers read
     `Arg::Angle` and resolve against the source yaw. The spawnpoint
     success drops the pitch (x, y, z, angle, dim, name|count);
     setworldspawn's drops the pitch and the dimension (x, y, z, angle).
     The `RespawnConfig` already stored yaw only.
  2. **`/setworldspawn` fails outside the Overworld:** 1.21.8 returns 0
     with `commands.setworldspawn.failure.not_overworld` when the source
     level isn't the Overworld and stores via `setDefaultSpawnPos(pos,
     angle)`; 1.21.9 sets the spawn in any dimension. The command now
     gates on `Dim::Overworld` and stores `[angle, 0.0]` (the `[f32; 2]`
     shape is kept — the save already wrote only `[0]` as `SpawnAngle`).
     The join snap in `lan.rs` now sets pitch 0.0 outright, and the lan
     test is rewritten for the angle form (the pitch-clamp case is gone).
  3. **`/worldborder` edits the Overworld border from any dimension:**
     1.21.8's `WorldBorderCommand` uses `server.overworld().getWorldBorder()`
     in every subcommand; 1.21.9 uses the source level's border. All reads
     go through `overworld_border()` and all edits through a new
     `edit_overworld_border` helper (writes where the Overworld border
     lives — live, parked, or level.dat's — re-derives the active level's
     view, mirrors into the parked Nether/End borders); the old
     `mirror_border_to_other_levels` (Overworld-only tail) is folded into
     it. `getLerpTime` citation reworded to `getLerpRemainingTime`.
  4. **`/fetchprofile` removed (1.21.9-only):** the tree, the dispatch arm,
     `cmd_fetchprofile`, the `ProfileFetch` queue + tick + pending check,
     `report_resolved_profile`, `encode_profile` + its tests, the four
     `commands.fetchprofile.*` quiet keys, and the harness's profile-fetch
     wait. The `skin_profiles` resolver itself stays (used by skins).
  5. **Selector rotation predicate wraps in double:** 1.21.8's
     `createRotationPredicate` uses `WrappedMinMaxBounds` +
     `Mth.wrapDegrees(double)`; 1.21.9's uses `MinMaxBounds$FloatDegrees`
     + float wrap. `rotation_in` now wraps in f64 (the bounds still parse
     as floats, widened exactly). `read_float_bounds` citation reworded
     to `WrappedMinMaxBounds.fromReader`.
  Verified no-change: `/summon` peaceful failure (item 17);
  `LocalCoordinates` record vs class (the Rust's `Coordinates` derives
  `PartialEq` — same value equality); selector distance/level defaults
  (Option-None is ANY; the level predicate fails non-players in both);
  `ListPlayersCommand` getId (API only); `CommandSourceStack`
  isOp/withEntity (Rust-internal op checks; players have no custom names
  in the Rust, so getPlainTextName == getName); Commands debug gates (no
  ChaseCommand/verbose-errors in the Rust; `/raid` behind the Rust's own
  dev flag); `Level.hasEntities`/`getEntityInAnyDimension` (1.21.9-only,
  unmodelled — item 26); `tickBlockEntities`/`PalettedContainerFactory`
  (internal/unmodelled); AbstractMinecartContainer (isClientSide rename).
  Known gaps (pre-existing vs both versions, not fixed): the shared-spawn
  border adjustment (vanilla moves a respawn outside the border to the
  border-centre heightmap position in both versions; the Rust returns the
  stored spawn); `/worldborder add` drops the remaining lerp time
  (vanilla adds `getLerpRemainingTime() + time`).

- **Item 25 — `mobs/body_anim.rs` (15 classes, 40 methods, 11 named in Horizon):**
  Armadillo/Bat/Bee/Fox/Frog/Goat/Llama/Parrot/PolarBear/Rabbit/SnowGolem/
  Strider/Endermite/Silverfish models + `KeyframeAnimation`. Zero-diff —
  no code changes.
  1. **The 12 models differ only by the `setupAnim` bridge signature:**
     their decompiled sources have no diff hunks at all, and javap shows
     the only bytecode difference is the `super.setupAnim` call target
     (`EntityModel.setupAnim(Object)` in 1.21.9 vs
     `setupAnim(EntityRenderState)` in 1.21.8) plus the synthetic
     `setupAnim(Object)` bridge 1.21.9 carries (it just checkcasts and
     delegates to the real method). Every 1.21.8 class file is exactly 44
     bytes larger — the longer type descriptor in the constant pool.
     Bodies and constants identical (the item-17 ArrowModel pattern).
     The shape: 1.21.8's `EntityModel<T extends EntityRenderState>`
     declares `setupAnim(T)`; 1.21.9's `EntityModel` extends `Model<S>`,
     whose `setupAnim(S)` erases to Object.
  2. **Endermite/Silverfish:** only the `@Override` annotation on
     `setupAnim` (1.21.9's `EntityModel` declares no `setupAnim` to
     override) plus the same bridge difference; the bodies are identical,
     and the Rust's arms match them exactly (age*0.9 + i*0.15*π phase,
     cos*π*0.01/0.05 and sin*π*0.1/0.2 with the (1+|i-2|)/|i-2| factors,
     silverfish layer links 2/4/1).
  3. **`KeyframeAnimation`:** 1.21.8 keeps a shared `scratchVector` field;
     1.21.9 allocates a fresh `Vector3f` per `apply()` call. Unobservable
     — the vector is pure per-entry temp, never retained; the Rust's
     `keyframes::Animation::apply` samples into per-channel locals and
     models no scratch vector at all.
  Verified: no `submit`/`extractRenderState`/`RenderState` citations in
  `body_anim.rs` (18 `setupAnim` mentions, all valid in both versions).
- **Item 26 — `arrows.rs` (8 classes, 27 methods, 12 named in Horizon):**
  `EntityReference`, `AbstractArrow`, `EnderMan`, `InsideBlockEffectType`,
  `Arrow`, `ThrownTrident`, `BaseFireBlock`, `LavaFluid` — arrows, their
  owner, and the freeze-clearing fire/lava effects.
  1. **`checkLeftOwner` runs after the move:** 1.21.9's `AbstractArrow.tick`
     calls `checkLeftOwner()` after the rotation sets, before the hit check
     (a fresh value from the current position/velocity); 1.21.8's tick does
     not call it — `Projectile.tick` (`super.tick()`, at the END of the
     flight branch, after inertia and gravity) updates `leftOwner`, so the
     hit check uses the value from the end of the previous tick. The Rust's
     `Ticker::tick` called `check_left_owner` pre-hit (the 1.21.9 form); it
     now runs after `step_move_and_hit` (before the early entity-hit
     return) and after the move in the no-physics branch. It runs there
     rather than in `finish()` (which the entity-hit path skips —
     `finish_entity_hit` runs it later, after the deflection, where the
     per-tick targets are gone), so it sees the pre-drag velocity and, on a
     deflecting hit, the pre-deflection one: both second-order (a 1% sweep
     difference; a deflect while still inside the owner's box). Observable
     differences vs the old code are confined to the tick-1
     spawn-outside-owner-box case, water (1.21.9's check uses the
     water-slowed velocity, 1.21.8's the pre-water one), and owner movement
     across the swept-box boundary between ticks. The in-ground branch
     never updates leftOwner in either version — the Rust matches (it
     returns early, no check, no finish).
  2. **CLEAR_FREEZE is gone (1.21.9-only):** the enum value's only appliers
     were BaseFireBlock, LavaCauldronBlock and LavaFluid, and
     `Entity.igniteForTicks` calls `clearFreeze()` in BOTH versions — so
     the real 1.21.8-vs-1.21.9 differences are only (a) fire-immune
     entities in fire/lava (1.21.9 clears freeze, 1.21.8 does not — both
     `fireIgnite` and `lavaIgnite` gate their whole body on
     `!fireImmune()`), and (b) negative fire ticks in fire (1.21.9 clears
     immediately, 1.21.8 only on ignition once the timer climbs to 0, up to
     20 ticks later). The fixes: the player's `apply_fire_contact` now
     clears only on ignition (the non-water arm tests `ignite_contact`'s
     result for >= 0; the water arm tests `fire_ticks >= -1` with no draw —
     the ignite runs first in vanilla, before EXTINGUISH clears the
     timer); the mob fire clear moved out of the tick head (it ran
     unconditionally, before the baseTick decrement) into
     `block_contact_damage`'s fire arm, gated on the post-ignite timer
     being >= 0 (the mob lava path already gated on `!fire_immune` and
     cleared via `ignite_for_ticks(300)`); the player's
     `lava_cauldron_inside` lost its redundant unconditional clear
     (players are never fire-immune, so the ignite clears in both
     versions); `arrows.rs`'s `Effect::ClearFreeze` variant, its two apply
     calls, the `EFFECT_ORDER` entry and the no-op match arm are removed
     (`step` is now `[bool; 3]`). The fire test now pins the 1.21.8 form:
     140 freeze ticks melt to 138 during the delay (no clear), and the
     ignition at the end of the delay clears to 0.
  Verified already-1.21.8: `EnderMan.getWalkTargetValue` (the 1.21.9-only
  0.0F override was never ported — the Rust's hostile branch returns
  Monster's `-light()`); `EntityReference` (the Rust resolves owners
  same-level only — `target_by_uuid`/`target_with_uuid` search the
  current level — matching 1.21.8's single-level `getEntity(UUIDLookup)`;
  the 1.21.9 any-dimension search was never modelled); `lerpMotion`
  (Vec3 vs three doubles, same body); `Arrow.tick` (only the
  `isClientSide()` rename); the `0.1F`/`(float)0.1F` decompiler artifact
  and the `@Nullable` on `getWeaponItem`. Known gaps, noted not fixed:
  the deflect owner argument (`deflect` takes the resolved owner in
  1.21.8 and the bare reference in 1.21.9, so a deflect drops the owner
  in 1.21.8 when it is not resolvable in the current level) — the Rust's
  `turn_back`/`trident_bounce` never touch the owner, and owners are
  never revalidated anywhere in the Rust; the only observable window is
  damage attribution on a post-deflect second hit by a cross-dimension
  or dead owner's piercing arrow or trident, and a partial fix (mobs
  only, when a UUID is known) would be worse than the documented gap.
  Ownership: `LavaCauldronBlock` itself belongs to item 44 (entities.rs)
  — only its CLEAR_FREEZE arm was this item's.
- **Item 24 — `lan/hub.rs` (1 class, 33 methods, 13 named in Horizon):**
  `PlayerList` — the server player list: joining, the level info, the
  border, bans, ops, respawn.
  1. **`sendLevelInfo` sends the Overworld's border:** 1.21.9 sends the
     player's level border (`$$1.getWorldBorder()`); 1.21.8 sends
     `server.overworld().getWorldBorder()` — a guest joining in the
     Nether sees the Overworld border (unscaled, 8x out in Nether
     coordinates: a vanilla 1.21.8 quirk, kept). Every `TickCtx`
     construction (6 sites: `game/lan.rs` x3, `lan_actions.rs`,
     `server_travel.rs::server_ctx`, the `hub_pass` test helper) now
     carries the Overworld border — the same three arms as
     `overworld_border` (live / parked / level.dat's), borrowed field
     by field so the `&mut world` still holds — and the field's doc
     says so. All `InitializeBorder` sends go through
     `send_level_info`, so join, respawn and level change are all
     covered.
  2. **Border changes broadcast to every player:** 1.21.9's listener
     calls `broadcastAll(packet, dimension)`; 1.21.8's calls
     `broadcastAll(packet)`. `lan_sync_border` now uses
     `hub.broadcast` instead of `broadcast_level`.
  3. **Citations:** `loadPlayerData` -> `load` (7 sites: the 1.21.8
     method loads and applies); `PrepareSpawnTask` / `Ready.spawn`
     -> `PlayerList.placeNewPlayer` (3 sites — the config task is
     1.21.9-only; the snap lives in `placeNewPlayer` in 1.21.8);
     `locateStatsFile` -> `getPlayerStats` (3 sites — the name exists
     in neither version; the rename is inline in `getPlayerStats`);
     `BanListEntry.getReasonMessage` -> `getReason()` (the null
     reason already rendered literal `"null"` per item 13);
     `PlayerList.setMaxPlayers` reworded (no such method in either
     version — the limit is fixed at startup).
  Verified already-1.21.8: the login gate order (ban, whitelist, IP
  ban, then full — `try_admit` + `can_login`); the duplicate-login
  kick; op/deop (the command gates on `isOp`, the save-at-once from
  item 16, the unconditional console `sync_ops` matching 1.21.8's
  unconditional `deop` send); the respawn search reads only block
  state, so the remove-before-search order change is unobservable
  (and the respawn pitch is yaw-only, 0.0F, as in 1.21.8); the parent
  vehicle restores after placement; the stats name->uuid rename;
  the 2-arg `broadcastAll(packet, dimension)` overload exists in
  1.21.8 too, so `broadcast_level` keeps its doc. `ALLOW_LOGOUTIVATOR`
  is a dead constant (declared, never read). No notification service
  is modelled (the 1.21.9-only `playerJoined`/`playerLeft` calls have
  no counterpart). Pre-existing gaps vs BOTH versions, noted not
  fixed: saved ender pearls are never respawned on login
  (`loadAndSpawnEnderPearls` exists in both versions — 1.21.8 calls
  it from `placeNewPlayer`, 1.21.9 from `PrepareSpawnTask`); the
  Rust's `NameAndId` type keeps its 1.21.9 name (a Rust-internal
  type, not renamed).
- **Item 23 — `mobs/ai.rs` (9 classes, 26 methods, 14 named in Horizon):**
  `Mob` and its subclasses' AI step, despawn, equipment and air.
  1. **Spawn armour (`Mob.populateDefaultEquipmentSlots`):** 1.21.9
     rolls `nextInt(3)` plus three 0.1087 upgrades over six materials
     (leather, copper, gold, chainmail, iron, diamond);
     1.21.8 rolls `nextInt(2)` plus three 0.095 upgrades over five
     (leather, gold, chainmail, iron, diamond — no copper). Both Rust
     copies fixed the same way: `spawning.rs::populate_armor` (the
     natural-spawn path) and `finalize.rs::armor` (the finalize-spawn
     path). New test `spawn_armour_is_five_materials_with_no_copper`
     (400 skeletons at a maxed special multiplier: no copper, leather/
     gold/chainmail all appear; it needs the prepared `.local` data,
     so it compiles here and runs on CI).
  2. **Axolotl dry-out (`Axolotl.handleAirSupply`):** 1.21.9 tests
     `shouldTakeDrowningDamage()` (`air <= -20`); 1.21.8 tests
     `air == -20`. The axolotl's check is now `==` (past -20 does not
     dry out — the same rule `breathing.rs` already had for the
     general path, and the WaterAnimal branch already had). The
     nautilus's `<=` is left: the nautilus is in neither jar, so its
     air code is unreachable dead code in this tree.
  3. **Citations:** `sendEventAndTriggers` exists in neither decomp
     (a 26.x extraction — in 1.21.8 the `START_RIDING_TRIGGER` fires
     inline in `Entity.startRiding`); reworded in `advancements.rs`,
     `boarding.rs`, `world_cmds.rs`, `creatures.rs`, `passengers.rs`.
  Verified already-1.21.8: the peaceful despawn already uses
  `despawns_in_peaceful` (item 17's rework — the
  `shouldDespawnInPeaceful` override set); `usePlayerItem` moved
  Animal -> Mob with a byte-identical body (both use `UseRemainder`,
  which exists in 1.21.8 too); the 1.21.9-only `ENTITY_MOUNT`/
  `ENTITY_DISMOUNT` game events on mount/dismount are unmodelled in
  the Rust (only the criterion trigger fires, as in both versions);
  the debug-packet split (`sendDebugPackets` in 1.21.8,
  `registerDebugValues` in 1.21.9) is unmodelled either way; every
  other hunk is the `isClientSide()` -> `isClientSide` rename or the
  `@Nullable` annotation. Pre-existing dead code, noted not removed:
  the `copper_horse_armor` arm of `horse_armor_stats` (no copper
  equipment in the 1.21.8 jar at all) and the nautilus air code.
- **Item 21 — `chat.rs` (4 classes, 36 methods, 15 named in Horizon):**
  the chat screen, its suggestions and the chat component. The 1.21.9
  chat-draft system is gone entirely: closing the chat loses the text.
  1. **Drafts removed:** `ChatComponent.latestDraft`/`Draft`/
     `saveAsDraft`/`discardDraft`/`createScreen`/`openScreen` and
     `ChatScreen.isDraft`/`exitReason`/`formatChat`/`shouldDiscardDraft`
     are 1.21.9-only. Deleted `ChatDraft`, `ChatExit`,
     `should_discard_draft`, `UiState.chat_draft`/`chat_draft_shown`
     and `keep_chat_draft`; `open_chat_method` opens on its prefix,
     `open_chat(initial)` and `close_chat()` drop their draft/exit
     parameters; the grey-italic draft formatter, the
     Backspace-clears-the-draft branch and the draft test are gone
     (the test is reworked to pin the 1.21.8 gating, not deleted).
     The wake-up flow in `tick_in_bed_chat` keeps working: a woken
     sleeper's typed text still reopens as a plain chat screen. The
     chat-search aside is still cleared on close, by the `draw_chat`
     cleanup that already ran when the screen went away.
  2. **Suggestions gated on the initial text:** 1.21.8's `onEdited`
     sets `allowSuggestions` to `!value.equals(initial)` —
     suggestions appear once the text differs from the text the screen
     opened on, and hide again when edited back to exactly that text.
     `ChatScreen` now stores `initial`; `edited()` applies the gate
     (chat layout only — the command-block screen's `onEdited` only
     refreshes the completions, per `AbstractCommandBlockEditScreen`).
     A resize keeps the ORIGINAL initial; a suggest-command link
     resets it to the suggested command (`mouseClicked`), leaving the
     flag alone. History recall still turns suggestions off, and
     coming back to the buffer leaves the flag alone
     (`moveInHistory`).
  3. **Unknown-command usage:** a mistyped literal is
     `dispatcherUnknownCommand().create()` in 1.21.8 — the plain
     `command.unknown.command` line with no context — where 1.21.9
     used `dispatcherUnknownArgument().createWithContext(reader)`
     (`command.unknown.argument` with the `^` marker). The Rust's
     parser tries only the exact-matching literal (`relevant`), so a
     mistyped literal leaves no errors where Brigadier records a
     literalIncorrect per command; `update_usage_info` detects that
     case directly (no errors, unread input, a literal child waiting
     at the deepest node) and shows the plain line. The
     context-wrapped `getParseException` fallback stays for trailing
     garbage after a leaf and for inputs with completions showing
     (`/ `), where vanilla wraps it too.
  4. **Callers:** the native chat API (`native_set_chat_input`), the
     in-bed submit clear and the chat-search toggle now go through
     `edit_input` (an edit, gated on the old initial) instead of
     `suggest_command` (which now resets `initial`, as a link click
     does); the true link paths (chat clicks, dialogs, command-block
     reload) keep `suggest_command`.
  Verified already-1.21.8 or unmodelled: the suggestion-list mouse
  handling and the key bindings are the same in both versions; the
  `clearMessages` flush-vs-clear difference is only the delayed-
  message throttle (`previousMessageTime`), which the Rust does not
  model (no delayed queue in `Chat`); `isAllowedInPortal` is 1.21.9-
  only and was already handled by item 12. Out of scope, recorded for
  their owners: the `saveChatDrafts` option definition
  (`options_file.rs`, `options_ui.rs`, `settings.rs`) is item 59's
  (Options) — defined and shown but now read nowhere;
  `InBedChatScreen` is item 348's (rank 3). New tests
  `suggestions_follow_the_initial_text_and_backspace_eats_one_character`
  and `a_mistyped_command_is_an_unknown_command_without_context`;
  the `should_discard_draft` unit test is removed (it pinned
  1.21.9-only behaviour).
- **Item 20 — `title.rs` (17 classes, 107 methods, 16 named in Horizon):**
  the title screens: world selection, create world, multiplayer join,
  flat-world customize, onboarding, backup/recover. 1.21.9 laid these
  out with `HeaderAndFooterLayout` and the `getContentX/Y` inset;
  1.21.8 uses fixed bounds and draws at the row's own left edge.
  1. **World-list double-click:** 1.21.8's `WorldListEntry.mouseClicked`
     joins on an icon click at once (`x - rowLeft <= 32`) and on a text
     click only within 250 ms of the last click on that entry
     (`lastClickTime`); a world that cannot be played swallows the
     click without even selecting. The Rust joined on any second click
     of the selected row and selected disabled worlds. New
     `last_world_click` field on `Title`; new test
     `a_world_row_joins_on_its_icon_or_a_fast_second_click` (fast
     second click joins, 260 ms later it only selects, icon joins at
     once, locked world swallows). A row-click join is also silent in
     1.21.8 (no sound in `mouseClicked` or `joinWorld`; only the
     list's Enter path plays the click), so `click()` no longer sounds
     `Action::Play` — the Play button sounds via its hit, Enter via
     `confirm_key_size`.
  2. **World-list geometry:** list bounds 49/`h-60` -> 48/`h-64`
     (`SelectWorldScreen` y0 48, height `h-112`); row content at the
     row's left edge (icon, name at +35, id/info lines), not 2 px in;
     the hover icon-light test is x-only (`rx < 32`, as in
     `WorldListEntry.render`). Footer buttons Edit/Delete/Re-Create/
     Back 71 -> 72 wide at -154/-76/+4/+82 (1.21.9's grid used 71).
     New test `the_world_list_footer_buttons_are_72_wide_below`.
  3. **Multiplayer geometry (`multiplayer.rs`):** list bounds 33/`h-60`
     -> 32/`h-64`; title drawn centred at y 20 (was 12, the 1.21.9
     header's centred StringWidget); footer buttons `h-52`/`h-28` ->
     `h-54`/`h-30` (two 20 px rows + 4 px spacer centred in the
     64 px footer). Tests updated (`rows_sit_two_pixels...`,
     drag/icon-arrow anchors 37 -> 36) and a new footer test
     `the_footer_buttons_fill_the_64_pixel_footer`.
  4. **Flat customize (`title/flat.rs`):** the height labels' right
     edge is `rowRight - 8` in 1.21.8 (the 1.21.9 content right is
     `rowRight - 2`) — header and layer rows; material title, slot and
     name sit at the row's left edge, not 2 px in. Layer narration is
     only `narrator.select(item)` (1.21.9 appended the height title
     and value), and the header is not narrated at all (1.21.8 has no
     header entry — `renderHeader`, not a row). The delete-by-index
     and `selection_after_removal` logic already matched 1.21.8.
  5. **Onboarding fade (`title/onboarding.rs`):** `onClose` sets
     `fadeOutStart` unconditionally in 1.21.8, so a second close
     restarts the fade; the Rust guarded it once (the 1.21.9 form).
     Test extended with a double-close restart.
  6. **Citations:** `extractSelection`/`extractContent`/
     `extractWidgetRenderState` -> `renderSelection`/`render`/
     `renderWidget`; `WorldListEntry.keyPressed` -> the list's
     `WorldSelectionList.keyPressed` (Enter joins there in 1.21.8);
     `deleteLayer` -> the inline delete; the multiplayer
     `HeaderAndFooterLayout(33, 60)` note -> the 1.21.8 manual layout.
  Verified already-1.21.8: the seed hint is DARK_GRAY; the search
  filter clears the selection (`clearEntries` nulls it); Enter joins
  the selected world/server; F5 refreshes only the multiplayer list;
  the add-server default name is `selectServer.defaultName`; the
  Play/Edit/Delete/Re-Create gates match `LevelSummary`
  (`primaryActionActive`/`canEdit`/`canRecreate` = `!isDisabled`,
  `canDelete` always true, verified via javap on both jars);
  `CreateWorldScreen.keyPressed` and the `lastScreen` return are the
  same behaviour; `setScreenAndShow`/`forceSetScreen` and the
  `renderCentered`/`renderLeftAligned` label calls are API-only.
  Out of scope, recorded for their owners: the server-row content
  inset and double-click are `ServerSelectionList`'s (item 334, rank
  3) — note the item-15 claim that the server list already does the
  250 ms window is wrong, the Rust joins on any second click there;
  the add/edit-server screen itself is `ManageServerScreen` (item
  334) / `EditServerScreen` (item 496, rank 4); the `PlainTextButton`
  pointing-hand cursor is 1.21.9-only API (the widgets/cursor item).
  Pre-existing gaps vs both versions, noted not fixed: the Rust never
  speaks the onboarding narrator message (1.21.8 at volume 1.0,
  1.21.9 at the voice volume); the flat list uses 20 px rows and a
  fixed header where vanilla uses 24 px rows and a scrolling header;
  the search box sits at y 21 (vanilla 22); `getEntryAtPosition`
  uses -2 where vanilla uses -4; the selection rects sit 2 px high.
- **Item 19 — `mobs/render.rs` (34 classes, 186 methods, 17 named in
  Horizon):** the mob renderers and layers. One real behaviour fix; the
  rest is the 1.21.9 submit-API refactor with identical values, so the
  bulk of the change is citation rewording.
  1. **Rabbit Toast strips formatting:** 1.21.9 checks the exact custom
     name (`checkMagicName(entity, "Toast")`); 1.21.8 compares the
     formatting-stripped display name (`"Toast".equals(
     ChatFormatting.stripFormatting(getName().getString()))`), so a
     rabbit named `§6Toast` is Toast too. The Rust now strips formatting
     before comparing (new test `a_formatted_toast_rabbit_is_toast`).
     The sheep's `jeb_` check is the same in both versions (exact custom
     name, `SheepRenderState.isJebSheep()`); only its citation changed.
  2. **Citations reworded** from the 1.21.9-only `submit` /
     `submitArmWithItem` / `submitOnShoulder` / `submitStuckItem` /
     `submitEntityInSpawner` / `AvatarRenderer` / `extractShadow` /
     `ShadowFeatureRenderer` / `checkMagicName` / `showExtraEars` names
     to the 1.21.8 `render` / `renderArmWithItem` / `renderOnShoulder` /
     `renderStuckItem` / `renderEntityInSpawner` / `PlayerRenderer` /
     `renderShadow` / `renderBlockShadow` / `isJebSheep` / profile-name
     forms, in `mobs/render.rs`, `shadows.rs`, `mobs/spawner.rs`,
     `game/perspective.rs` and `game/creatures.rs`.
     `extractRenderState` citations stay — the extract pattern exists in
     1.21.8 too. The `submit`→`render` rename is global; each item fixes
     its own files' citations (other renderers' files still carry theirs).
  3. **Verified identical:** spawner spin (`lerp × 10`) and scale
     (`0.53125 / max(w, h)`), salmon size models, warden emissive alphas
     (`cos(age·0.045)·0.25`, `+π` variant, tendril/heart), breeze wind
     scroll (0.02/tick) and eyes, energy swirl offsets (creeper `t·0.01`,
     wither `cos(t·0.02)·3`, V `t·0.01`), skeleton `translateToHand`
     (±1) and vex (0.55 + offset), wolf armour crack thresholds (0.95 /
     0.69 / 0.32), witch potion transform, cape `fallFlyingScale`
     (`clamp(t²/100)`), parrot shoulder offsets (±0.4, −1.3/−1.5),
     stuck-in-body angles, sheep wool/undercoat guards, deadmau5 ears
     (profile name), spectator layers rule, shadow radii and the
     `renderShadow`/`renderBlockShadow` math (radius ≤ 32, strength
     `(1 − dist²/256)`, 0.5/depth falloff, brightness > 3, full-collision
     gate, UVs, light 15728880), and the F3+B hitbox gizmos (view arrow
     ×2.0, blue). `SalmonRenderer$1` is only the synthetic `$SwitchMap`
     for the variant switch (1.21.9 uses a switch, 1.21.8 if/else — same
     mapping). The mannequin arm of `avatar()` is unreachable dead code
     (no mannequin in the 1.21.8 registry), left in place.
  - **Out of scope:** the `entity_light_block` crouch drop uses an
    unscaled 0.125 where vanilla uses `scale · −2/16` — a pre-existing
    approximation, identical in both versions. Server-hitbox debug
    rendering ("Missing" text, server vector) is unimplemented in the
    Rust in both versions.

- **Item 14 — `particles.rs` (81 classes, 347 methods, 22 named in Horizon):**
  the particle engine. Two systematic 1.21.9→1.21.8 engine changes, the
  removal of the 1.21.9-only explosion debris tracker, and the
  colour/power option types that do not exist in 1.21.8.
  1. **Provider random stream:** 1.21.9 passes the engine's own random to
     every provider as a `RandomSource` parameter. 1.21.8's providers get
     no parameter — they draw from `level.random` (`ClientLevel.random`).
     Every provider draw in `make` switched from the engine random to
     `level_random` (block crumbs, white ash, sculk charge, witch colour,
     suspended, suspended-town, firefly, glow providers), except draws
     from a static class random (SpellParticle.RANDOM, already correct;
     GlowParticle.RANDOM, new `glow_random` field for the glow-squid
     velocity) and draws from the particle's own random `r`. Not provider
     draws, so kept on the engine random: `crack`/`destroy` positions
     (1.21.8 `ParticleEngine` uses `this.random`), and the pre-existing
     firework-starter / `send_particles` / emitter draws (same in both
     versions).
  2. **Sprite picking:** 1.21.9 picks the sprite in the provider
     (`sprite.get(providerRandom)`) and passes it into the constructor.
     1.21.8 picks via `TextureSheetParticle.pickSprite` (the particle's
     own random), called after construction — by the provider
     (SuspendedTown, Firefly, VibrationSignal, FlyStraightTowards,
     Shriek), in the constructor (SuspendedParticle, between the size
     draw and the size multiplier), or by the engine's `Sprite` wrapper
     (DripParticle static creators). `sprite_from_set` now draws from
     the particle random `r`, placed per arm; the BreakingItem layer
     pick moved after construction too (size draw → pick → uo/vo).
     `TextureSheetParticle` extends `SingleQuadParticle` in 1.21.8 (the
     1.21.9 merge of the two classes is 1.21.9-only); citations fixed
     accordingly, including `SingleQuadParticle.renderRotatedQuad`.
  3. **Evict-oldest cap:** the Rust's "last quarter fills with decreasing
     probability" soft cap was a 26.x-ism. Both versions cap at 16384
     per layer with an `EvictingQueue` that admits the new particle and
     evicts the oldest. `particles` is now a `VecDeque`; the drain loop
     pops the front when full.
  4. **Explosion debris tracker removed:** `ClientExplosionTracker` (and
     its `ExplosionInfo`, the 512-debris weighted throw, the
     `track_explosion` / `tick_explosions` machinery and its three
     tests) is 1.21.9-only. 1.21.8's `ClientLevel.explode` is an empty
     override and `handleExplosion` only plays the sound and adds the
     packet's particle. `explosion()` now takes only `(center, small)`.
  5. **Option types:** EFFECT, INSTANT_EFFECT, FLASH and DRAGON_BREATH
     are all plain `SimpleParticleType`s in 1.21.8 — `SpellParticleOption`
     (colour + power), `PowerParticleOption` and the coloured FLASH do
     not exist. The Effect/InstantEffect colour+power arm and the
     DragonBreath `set_power` are gone; the firework starter now builds
     its flash directly (`flash()` helper: plain FLASH, provider's
     `pickSprite`, then the starter's `setColor`).
  6. **Verified identical:** `addParticle` limiter logic (1.21.9 inlines
     it in ClientLevel, 1.21.8 delegates to `LevelRenderer`
     `addParticleInternal` — same draws, same random), tick order,
     `destroy`/`crack` bodies, `SpriteSet.get`, `ParticleGroup` vs
     `ParticleLimit` (identical, rename only), SingleQuadParticle quad
     math, and the values of every provider (Glow, Suspended,
     SuspendedTown, Spell, Crit, Explode, Gust, SonicBoom, SweepAttack,
     Wake, WaterDrop, Trail, TrialSpawnerDetection, Shriek, Vibration,
     Firefly, FlyStraightTowards, LlamaSpit).
  - **Out of scope (recorded, not fixed here):** item 315 owns a REAL
    protocol bug found here — the wire codec in `entity_data.rs`
    `Particle::read` reads colour/power data for effect/instant_effect
    (ids 15/45), flash (41) and dragon_breath (7), all Simple in 1.21.8,
    so a 1.21.8 server's packets would desync the stream; the potion
    cloud-particle NBT parse has the same 1.21.9 form. Items 54/491 own
    the ElderGuardian apparition (1.21.8 uses MobAppearanceParticle).
    Item 109 owns the rest of TerrainParticle, item 29 the rest of
    BreakingItem/Firefly/Drip. The `particle_providers` oracle fixture
    references 1.21.9-only option types and 26.x types, but that harness
    is not in the 17-harness CI set, so it is harness infrastructure,
    not this item.

- **Item 18 — `block_entity_render.rs` (32 classes, 195 methods, 24 named in
  Horizon):** the block-entity renderers. One real behaviour fix (the
  1.21.9-only crumbling overlay is gone); everything else verified
  identical, with the 1.21.9-only `submit` / `extractRenderState` /
  `*RenderState` citations reworded to the 1.21.8 `render` names.
  1. **No breaking cracks on block entities:** 1.21.9 added the
     crumbling overlay (`ModelFeatureRenderer.CrumblingOverlay`,
     passed into `extractRenderState` so each renderer re-draws its
     model under the destroy stage). 1.21.8 has no such thing — the
     whole `net.minecraft.client.renderer.feature` package is absent,
     the dispatcher carries no destroy state, and
     `LevelRenderer.renderBlockDestroyAnimation` only re-renders the
     block model (`renderBreakingTexture`), which is empty for
     block-entity blocks. `render_breaking`,
     `breaking::entity_geometry` and the `crumbling` parameter
     threading (`render_one`, `render_banner`, the conduit shine
     gate and shell/cage-only early return) are removed, with the
     game.rs call site and the tests that pinned the overlay.
     Block-model breaking (`breaking::geometry`, layers 11/12) is
     untouched — that exists in both versions.
  2. **Conduit eye billboard is the same matrix either way:**
     1.21.9 orients the eye as `orientation · Rz(π) · Ry(π)`,
     1.21.8 as a single `rotationYXZ(−yRot, xRot, π)`; since
     `Camera.rotation` is `rotationYXZ(π − yRot, −xRot, 0)` the two
     compose to the same matrix (verified numerically, max
     difference ~3e-16). The Rust keeps the 1.21.9 form with a
     comment recording the proof — the eye textures are
     substantially asymmetric, so the equivalence matters.
  3. **Verified identical, no code change:** bell swing
     (`sin(ticks/π) / (4 + ticks/3)`, gated on `shaking` with
     `clickDirection`), book poses and page flips, piglin ears and
     dragon jaw, beacon beam geometry (rotation, scroll, radii,
     last section 2048), chest lid (`1-(1-open)³`, combined
     openness = max of halves), shulker lid (`24 − progress·8`,
     270° turn), skull rotation (wall facing via
     `convertToSegment(opposite)` = `toYRot`, conjugated through
     the −1 scale), lectern book (`setupAnim(0, 0.1, 0.9, 1.2)`),
     enchanting-table bob and rotation, end-portal cube (only the
     Y faces, slab 0.375–0.75 — `shouldRenderFace` is Y-only),
     end-gateway beam (spawn/cooldown percents, magenta/purple,
     `sin(percent·π)` scale), sign text (four centred lines,
     glowing full-bright with the 8x dark outline within 16
     blocks or for black text), hanging-sign offsets, default
     player skins (same 18, same UUID-hash selection), the
     `PreparedTextBuilder` glyph/effect math, and every
     block-entity layer in `LayerDefinitions` (the diff there is
     renumbering plus the copper-golem layers and the armor
     registration refactor — entity-model items' scope).
  4. **Out of scope, recorded:** the copper-chest and
     copper-golem-statue code is unreachable dead code in this
     tree (no such blocks in the 1.21.8 registry) — removal
     belongs to the copper-golem/statue items (item 229 owns the
     golem). The shelf renderer registration is 1.21.9-only; the
     shelf itself is another item's. The decorated-pot wobble
     style is a real difference found here but belongs to item 58
     (`DecoratedPotRenderer`): 1.21.9's `extractRenderState`
     never copies `lastWobbleStyle`, so its submit always takes
     the non-POSITIVE branch (which the Rust hardcodes), while
     1.21.8's `render` reads the style and tilts on X/Z for
     POSITIVE — item 58 must set the bool from
     `PotWobble.style` (the render side already implements both
     branches). Pre-existing gap vs both versions, noted not
     fixed: the sign-text outline's scoping arm (first-person +
     scoping shows the outline at any distance; the Rust checks
     distance only).
- **Item 15 — `realms.rs` (50 classes, 271 methods, 25 named in
  Horizon):** the Realms screens, options, invites, upload and
  list narration.
  1. **Renew copies and opens:** 1.21.9 shows a `ConfirmLinkScreen`
     for `onRenew`; 1.21.8 copies the extend link to the clipboard
     AND opens it directly. New `Outcome::CopyAndOpenUri` /
     `Action::CopyAndOpenUri`, handled in both title-action sites
     (clipboard set, then the URI opened). The other confirm-link
     flows stay — notification links confirm in both versions
     (`VisitUrl.buildOpenLinkButton` uses `ConfirmLinkScreen` in
     1.21.8 too).
  2. **Notification layout is multi-row:** 1.21.9 is one
     variable-height entry; 1.21.8 is a message row plus
     `Mth.positiveCeilDiv(wrapHeight + 7, 36) - 1` empty rows and
     a separate button row, the text wrapped at 216 in an outline
     (`-12303292`) and the button 150x20 centred under it.
     `draw_notice` is reworked to that form.
  3. **Slot button action is minigame-based:** 1.21.9 switches
     unless `activeSlot || empty && expired`; 1.21.8 unless
     `activeSlot || minigame && expired`. The click condition now
     reads `minigame`, not `empty`.
  4. **`RealmWorldOptions` has its three dropped fields back:**
     1.21.8's DTO carries `pvp`, `spawnMonsters` and
     `commandBlocks` (nullable); 1.21.9 dropped them. Both upload
     flows set them: create-from-world takes `allowCommands` for
     command blocks, the upload screen passes hardcoded true
     (`createFromSettings(settings, true, version)`); pvp and
     spawn monsters stay false, as Java's defaults do.
  5. **Slot options screen reworked:** PvP, spawn-monsters and
     command-blocks toggles back (ids 25/26/27 — the first
     100–102 run collided with the row-button range arm and was
     moved). NORMAL reads the slot's values; other world types
     default all three true. Spawn-monsters off opens the warning
     popup (`mco.configure.world.spawn_toggle.message`) — only
     Continue clears it. `save_slot_options` saves all three,
     keeping the slot's stored values for ADVENTUREMAP/EXPERIENCE/
     INSPIRATION and gating spawn monsters on NORMAL +
     non-peaceful.
  6. **Pending invites reworked:** footer Accept/Done/Reject
     (100x20, visible only with a selection) plus the per-row
     15x15 Accept/Reject buttons; rows 260 wide, list y 32 to
     h−40, face 32px at the row origin. The row buttons and the
     footer share `answer_pending_invite`; a list refresh clears
     the selection, as vanilla's removal does.
  7. **Upload-select reworked:** no search box (1.21.9-only), no
     icons, no double-click-to-upload; entries are name, id line
     (`mco.upload.entry.id`, folder + short date) and the world
     info line, rows 60% of the screen width, list from row(0)
     to h−40. The list filters by `canUpload` (`!locked`;
     `requiresManualConversion` is true only for two 1.20.5
     snapshots, which the Rust does not model). The orphaned
     `world_icons` cache went with the icons.
  8. **Invite duplicate check removed:** 1.21.9 refuses an invite
     to a player already on the server; 1.21.8 refuses only a
     blank name. The duplicate branch and the `Invite` screen's
     `players` field are gone; the test now pins blank-refused,
     duplicate-goes-through.
  9. **Players tab header scrolls:** 1.21.8 has no `HeaderEntry` —
     the invited count is the list's `renderHeader`, drawn at
     `getY() + 4 - scrollAmount()`, so it scrolls with the list.
     The Rust's heading now subtracts the scroll amount and sits
     inside the clip region.
  10. **Template screen:** the minigame warning lines are at
      row(−1)/row(0) in GREY (not 25/38 in `0xFFBABABA`), the
      list top is 53 when warning, the footer spacing is 10, and
      the entry version/recommended-players colours are
      `0xFF6C6C6C`/`0xFF4C4C4C`.
  11. **Slot names cut to 64:** 1.21.8 truncates the slot name
      inline at 64 wide with "..." whatever the button's size;
      1.21.9 made it a StringWidget with
      `setMaxWidth(width - 2)`. The clamp is now a fixed 64.
  12. **Container-list narration:** 1.21.8's
      `ContainerObjectSelectionList.updateWidgetNarration` adds
      `narration.component_list.usage` unconditionally, and its
      entry adds it for a focused row with more than one
      narratable; 1.21.9 dropped both. The Rust adds both lines
      (the collector keys by type and depth, so the screen, list
      and row lines all read, as in vanilla); the container-list
      test expectations are updated. `ObjectSelectionList` is
      identical in both versions — no change there.
  13. **Slider narration:** 1.21.8's `updateWidgetNarration`
      always uses `narration.slider.usage.focused` for a focused
      slider; the `keyboard_cannot_change_value` variant is
      1.21.9-only. The branch is gone from the narrator and its
      test. The branch was the last reader of the narrator's
      last-input-type tracking, so the `input` field is gone from
      `ScreenNarrator` and `Ctx` (the `set_input` callers stay —
      Tab and the arrows still move the focus off a clicked
      widget).
  14. **Terms link copies and opens:** vanilla's
      `RealmsTermsScreen.mouseClicked` copies the terms URL to the
      clipboard and opens it, in both versions; the Rust only
      opened it. It now returns `CopyAndOpenUri` (a pre-existing
      deviation, fixed with the plumbing this item built for
      renew).
  - **Verified no-change:** server-row double-click (the Rust
    already does same-row + < 250ms, matching 1.21.8's
    `lastClickTime` rule); `renderOnlinePlayers` and the
    refresh-entries server re-selection (same net effect);
    `NotificationButton` tooltip; the subscription tab (labels
    already left-aligned, the info panel already the 1.21.8
    `FocusableTextWidget` form) and the settings tab's region
    label (already left-aligned — 1.21.8's `.alignLeft()`);
    configure-tab reset (the 1.21.9 preservation only shows on
    window resize, and the Rust has no resize hook into the
    Realms screen); feature flags (not implemented in the Rust);
    `setScreenAndShow`/`forceSetScreen`,
    `CreateWorldScreen.openFresh`'s callback form, the generic
    error screen's render and `canInterruptWithAnotherScreen`
    (no interrupt gate in the Rust), the backup-info screen's
    size API; the backup-screen and backup-info entries, the
    region-selection entry and the long-running-task screen
    (render/input API only, same coordinates and colours);
    `RealmsUtil.renderPlayerFace` (session-service skins vs the
    render cache, same visible result); the slider sprites
    (`getSprite`/`getHandleSprite` identical; the `dragging`
    `||` is a provable no-op); the singleplayer world list
    (empty list opens Create New World, no text truncation or
    tooltips, icon-click and Enter join, filter change clears
    the selection — all already the 1.21.8 form; the "World N"
    empty-name branch is dead in vanilla since `getLevelName`
    falls back to the folder).
  - **Out of scope (other items):** the cursor API is entirely
    1.21.9-only (no `CursorTypes`/`requestCursor` in the 1.21.8
    jar) — removing the Rust's cursor system belongs to the
    widgets/ui item, not this one; the Rust has no RESIZE_EW
    cursor for sliders, so nothing slider-specific to remove.
    Two pre-existing deviations vs both versions, noted not
    fixed: the world list joins on any second click of the
    selected row (vanilla wants a 250ms double-click), and an
    empty `LevelName` draws empty (vanilla shows the folder);
    the list-refresh reload on edit/recreate returns is the
    1.21.9 always-reload form, with no visible difference.

- **Item 16 — `game.rs` (37 classes, 178 methods, 33 named in
  Horizon):** buckets, honeycomb, game mode, ops, narration,
  panorama and the F3+F4 gate.
  1. **Bucket shift removed:** 1.21.9 places adjacent when sneaking
     at a replaceable/liquid-container target; 1.21.8 places into
     the block (`isAir || canBeReplaced || canPlaceLiquid`, no
     shift). The `sneaking` parameter is gone from both
     `bucket_empty` fns (game, remote, dispenser call sites and 5
     tests updated); clicking a slab now waterlogs it.
  2. **Honeycomb double-chest event removed:** 1.21.9 fires
     BLOCK_CHANGE + level event 3003 on the connected half of a
     double copper chest; 1.21.8 waxes one block only. The
     second-half event and sound are gone (game + remote);
     `chest_connected_pos` went with them. The wax table itself is
     data-driven from the 1.21.8 pack, so no table change.
  3. **Game mode:** (a) `Guest::change_game_mode` no longer
     records the mode left as previous — 1.21.8 preserves it
     (only set at load from `previousPlayerGameType` and copied on
     restore). (b) The host save no longer writes the client-model
     previous back: the saved value stays the loaded one. The
     client's own `previous_mode` (F3+F4, the switcher) is the
     `setLocalMode` model, byte-identical in both versions —
     untouched. (c) The flying-disable-near-ground on mode change
     is gone (`set_mode` and the hub's `set_game_mode`, with the
     now-dead `world`/`collision` params and their callers).
  4. **Op/deop save at once:** 1.21.8's
     `DedicatedPlayerList.op/deop` call `saveOps()` immediately;
     1.21.9 dropped it (its base `op/deop` never save). A new
     `save_ops` helper (warn on failure, as vanilla does) runs
     after a successful in-game `/op`/`/deop` and console
     op/deop. The management-server callers are Horizon-only
     tooling, left alone.
  5. **Narration suppression is always two seconds:** 1.21.9's
     `init` suppresses forever for keyboard users; 1.21.8 always
     suppresses `NARRATE_SUPPRESS_AFTER_INIT_TIME` (2 s).
     `ScreenNarration::after_init` lost its `keyboard` parameter;
     the keyboard-forever test is rewritten for the 2-second
     rule.
  6. **The onboarding panorama does not spin:**
     `AccessibilityOnboardingScreen.renderPanorama` passes false
     in 1.21.8 (1.21.9's `panoramaShouldSpin` override says the
     same). The title cube now runs at zero speed while the title
     mode is Onboarding — `Rotation::update` still tracks the
     clock, so no jump when the title returns.
  7. **F3+F4 over a pauseless pause screen:**
     `KeyboardHandler`'s gate is no screen, or a PauseScreen
     without its menu, in both versions; the switcher arm now
     accepts `PauseWithoutMenu` too.
  - **Verified no-change:** spawner flags (each spawner ticks with
    its own inline gates; cat and trader match both versions);
    portal ticket expiry (PORTAL semantics identical);
    skeleton-trap lightning rods (only `minecraft:lightning_rod`
    exists in the 1.21.8 registry); explosions (the `Explode`
    packet is already the 1.21.8 form); save format (`SpawnX/Y/Z`
    + `SpawnAngle`, rank 1); whitelist persistence (item 13);
    trapped-chest sounds (the `contains("chest")` arm plays the
    chest sounds, same in both versions); RCON/console source
    position (the spawn pos in both versions); dedicated-start
    whitelist and max-players from properties; absolute max world
    size (the default 29,999,984 already); `getAnalogOutputSignal`
    (bodies identical); the 8 renderer diffs (submit-API refactor,
    identical transforms/values); `isClientSide()` renames;
    AnvilScreen/TitleScreen input API (same behaviour);
    `isInGameUi` (a pure refactor — containers and sign editors
    override `renderBackground` with the transparent gradient in
    1.21.8, the base checks the flag in 1.21.9; same picture);
    `removeWidget` focus-clear (the Rust's widgets are drawn per
    frame, not objects — no `removeWidget` to fix);
    `canInterruptScreen`/F3+F6 (1.21.9-only; the debug options
    screen stays as Horizon debug UI, as the 1.21.7 port kept
    it); F3+F4's WinScreen exclusion (unreachable behind the
    `KeyboardHandler` gate in both versions) and
    `canSwitchGameMode` (subsumed by the player check);
    `getEntityInAnyDimension` (the only Rust cross-dimension
    lookup is the native-mod API); `waitForEntities`
    (server/test plumbing, unmodeled); the debug classes,
    GameTestTicker and TextureManager.
  - **Known gaps (unmodeled, upstream model):** level emptiness
    (`emptyTime`/`hasActiveTickets`) and START/spawn-chunk tickets
    — parked levels do not tick at all. The copper-golem opener
    machinery (`golem_chests`, built from `chest_openers`) is
    dead in this tree and goes with item 229; the container
    recount itself is players-only, as in 1.21.8.

- **Item 17 — `game/creatures.rs` (33 classes, 108 methods, 39
  named):** the peaceful rework plus spawn-egg, turtle and creeper
  fixes.
  1. **No `notInPeaceful` flag:** 1.21.9's `EntityType` flag (36
     types), `spawn_monsters` rule and `ServerLevel.isSpawningMonsters`
     are gone. 1.21.8 has two separate rules instead: the despawn
     (`Mob.checkDespawn` discards in peaceful iff
     `shouldDespawnInPeaceful` — the Monster class except Piglin,
     plus Ghast, Phantom and the slimes, whose `size > 0` is
     always true since sizes clamp to 1..127) and the per-predicate
     spawn gate (each type's own check). `allowed_in_peaceful`
     (both copies) is replaced by `despawns_in_peaceful` — the 36
     flagged types minus Hoglin (an Animal, stays in both versions)
     and Parched (26.x-only).
  2. **Per-predicate spawn gates:** the global flag gate in
     `check_spawn_rules` is gone; the difficulty check moved into
     the predicates that have it in vanilla, verified against the
     1.21.8 `SpawnPlacements` registrations (javap bootstrap
     methods) and the check bodies: `monster_rules`
     (`checkMonsterSpawnRules`) now checks peaceful, as do the
     Blaze/Breeze/Zoglin (`checkAnyLightMonsterSpawnRules`),
     Endermite/Silverfish and Pillager
     (`checkPatrollingMonsterSpawnRules`) arms. Drowned, Guardian,
     Ghast, MagmaCube, Slime and ZombifiedPiglin already checked
     inline. Phantom (`Mob.checkMobSpawnRules`), Hoglin and Piglin
     (custom predicates) correctly check nothing — a spawner-spawned
     phantom passes in peaceful, then despawns.
  3. **ZombieHorse spawn rules:** it sat in the monster group (a
     pre-existing deviation); vanilla registers it with
     `checkZombieHorseSpawnRules` — spawner checks the light, else
     `Animal.checkAnimalSpawnRules`, no difficulty check. It now
     shares the SkeletonHorse arm.
  4. **`/summon` has no peaceful failure** in 1.21.8 (no
     `ERROR_FAILED_PEACEFUL`): the check is gone; a summoned monster
     despawns via `checkDespawn` on its first tick.
  5. **Spawn eggs have no peaceful failure:** `spawn_egg_allowed`
     and the red "Disabled in Peaceful" tooltip
     (`spawn_egg_peaceful_line`, with its test) are gone, server-side
     and in the remote client's prediction. The ENTITY_PLACE event
     is at the CLICKED pos in 1.21.8 (the spawn pos is the 1.21.9
     change). The egg's `entity_data` handling already read the type
     from the compound — the 1.21.8 `CustomData` model — so only the
     `TypedEntityData` comment references changed
     (`CustomData.loadInto`/`contains`).
  6. **No spawner-enabled gate on eggs:** 1.21.8 sets a spawner (or
     trial spawner) unconditionally; the `spawner_blocks_work` gate
     and the `advMode.notEnabled.spawner` message are gone from
     `use_spawn_egg_on_spawner`. (The spawner TICK gates stay —
     the spawner item's scope; they must go before the gamerules
     item removes the rule from the table, or `rules.bool` returns
     false and all spawners stop.)
  7. **`spawn_monsters` gates reverted to difficulty:** natural
     spawning (`spawn_enemies = difficulty != 0`), phantoms,
     patrols, village sieges, zombie reinforcements (Hard +
     doMobSpawning only), the endermite roll on pearl landing
     (doMobSpawning only, no difficulty check), the creaking-heart
     spawn check, and the Nether-portal pigman tick (doMobSpawning
     + the difficulty roll, which fails on its own in peaceful).
     The rule's table entry, save mapping, datafix conversion and
     the `LevelData` field stay for the gamerules/datafix items.
  8. **`EntityType.create` has no peaceful check** (in either
     version — the old comments were wrong): the gates in the
     structure-mob creator, the finalize-spawn fixture's creator
     (its `peaceful` field went with it) and the zombie-horse rider
     spawn are gone; the rider spawns in peaceful and despawns via
     `checkDespawn` (the peaceful discard runs before the
     persistence/passenger checks). The natural-spawn fixture's
     case-entity skip now uses the despawn predicate, citing
     `checkDespawn`.
  9. **Turtle scute is a direct drop:** 1.21.9 drops it from the
     `turtle_grow` loot table; 1.21.8 calls
     `spawnAtLocation(Items.TURTLE_SCUTE, 1)` — no table (it does
     not exist in the 1.21.8 data pack) and no mob-drops gate. All
     three sites (age boundary, fed growth, the age test) now emit
     `MobEvent::DropItem`; the gift-loot handler's mob-drops gate
     keeps only the panda.
  10. **Charged-creeper heads are per-victim direct drops:**
      1.21.9's `Creeper.killedEntity` + `charged_creeper` loot table
      (which does not exist in the 1.21.8 data pack — no heads
      dropped at all) is replaced by the 1.21.8
      `dropCustomDeathLoot` model, inside the loot-table block so
      the order (loot, equipment, head) and the mob-drops gate
      match: creeper, zombie, skeleton, wither skeleton and piglin
      heads, with Stray and Bogged inheriting Skeleton's (the
      1.21.9 table's exact-type conditions exclude them) and
      Husk/Drowned/Zombie Villager/Zombified Piglin returning an
      empty skull. The one-head-per-explosion rule still runs
      through `charged_exploders`.
  - **Verified no-change:** EatBlockGoal (the only diff is the
    canUse chance wrapped in `adjustedTickDelay`, unobservable
    without the /tick rate feature the Rust does not implement —
    `reduced()` is the scheduler's own halving); NetherPortalBlock
    (beyond the gate above); MushroomCow flower-feed particle
    (already plain `EFFECT` — comment only) and DragonFireball
    particles (already plain `DRAGON_BREATH`); ThrownEnderpearl
    vanish/tick (no `wonGame` exemption in the Rust, no pearl
    chunk tickets, and the owner-lookup distinction is
    unobservable in the always-in-memory host/guest model);
    `AbstractHurtingProjectile.onDeflection` (bodies identical,
    signature only); ArrowModel (decompiled sources identical;
    javap shows only the `setupAnim` bridge signature differing);
    all 8 renderer diffs (1.21.9 submit-API refactor, identical
    transforms and values); FlameFeatureRenderer (1.21.9-only name
    for the same fire quads — comment now cites
    `EntityRenderDispatcher.renderFlame`); `isClientSide()` →
    field renames, `asEquipmentSlot()` → `getSlotForHand`,
    `usePlayerItem` moved Animal → Mob (byte-identical body),
    `EntityReference` API change, Bee End clause (redundant),
    Bee debug packets (neither debug system is implemented);
    `loadEntityRecursive` / `STREAM_CODEC` / vehicle-attachment
    plumbing; BaseSpawner's peaceful gate (identical in both
    versions).
  - **Deferred:** the `TypedEntityData` wire format
    (`entity_data`/`block_entity_data`/`bees` as id + compound) is
    items 257/170/92/181/314's scope — the 1.21.8 wire form is the
    plain `CustomData` compound (`skip.rs`, `hash/raw.rs`
    incl. test vectors, `v1_20_3.rs` encode/decode/normalize,
    `item_sync.rs` `block_entity_data_tag`); the copper-golem
    pumpkin pattern (`try_spawn_copper_golem` in `golems.rs`)
    eats the pumpkin and copper block and spawns nothing while
    the type is absent from the 1.21.8 registry — item 229's
    removal; the `spawner_blocks_work` tick gates (spawner item,
    see fix 6); the `spawn_monsters` table entry and friends
    (gamerules/datafix items). Copper golem and mannequin
    `Species` variants, tables and AI modules remain as dead code:
    neither type is in the data-driven 1.21.8 registry, so
    nothing can spawn them except the golem pattern above.

- **Item 13 — `game/dedicated.rs` (5 classes, 167 methods, 44 named,
  985 diff lines):** six fixes.
  1. **World border persistence:** 1.21.9 keeps a border per level in
     `data/minecraft/world_border.dat`; 1.21.8 keeps the one border in
     level.dat's `Border*` keys. `Level` gained a `border` field, read
     and written with the rest of level.dat (`save.rs`); the
     per-level file read/write is gone (`load_border`/`save_border`
     reworked, `border_path` removed, `WorldBorder::pack/unpack`
     replaced by `pack_level_dat/unpack_level_dat`).
  2. **Other dimensions derive their border:** the Nether and the End
     start from the Overworld's border with the centre divided by
     their coordinate scale (`Level$1`: Nether /8, End /1) — a new
     `WorldBorder::for_coordinate_scale`, used at world open (from
     the loaded level), when a parked level opens (from the live
     Overworld border), and by the mod API (`native_world_border`
     now answers each level's own bounds, not the live border's for
     every dimension).
  3. **Overworld edits mirror:** an Overworld `/worldborder` edit
     mirrors into the parked Nether/End borders (the delegate
     listener); a Nether/End edit never propagates back, and only
     the Overworld border reaches level.dat. `save_border` refreshes
     the level's copy from the live Overworld border before every
     level.dat write, including the snapshot path
     (`save_game_metadata`).
  4. **Whitelist kick on enforce alone:** 1.21.8's
     `kickUnlistedPlayers` kicks on `enforceWhitelist` alone; the
     `&& isUsingWhitelist()` came in 1.21.9. A reload or a removal
     with enforce on but the list off still kicks.
  5. **Ban reason null renders literally:** 1.21.8 passes the raw
     reason, and a null one renders as the literal `"null"`
     (`TranslatableContents` TEXT_NULL) — in the ban entry, the
     `/ban` and `/ban-ip` success messages. The default-text form
     came in 1.21.9's `getReasonMessage()`.
  6. **No code of conduct:** `readCodeOfConducts` and the
     `codeofconduct` folder came in 1.21.9. The dedicated start no
     longer reads the folder (an unreadable one never stops the
     start), the property is ignored, and `LanHost.code_of_conducts`
     is gone. (`isAutoSave`/`setAutoSave` are 1.21.9's names for
     1.21.8's per-level `noSave` flip — doc note only; the shared
     flag stays.)
  - **Tests:** the two border persistence tests now assert the
    shared model — a Nether-only size does not survive a reopen,
    the Nether/End re-derive from the Overworld after a set, and
    the Nether centre is the Overworld's /8 — plus a level.dat
    round-trip and a centre-scaling unit test. The in-memory test
    (a Nether set leaves the Overworld alone) is unchanged: it
    holds in 1.21.8 too.
  - **Verified no-change:** spawn protection (Overworld + shared
    spawn), console source in Overworld, default-spawn packet
    pos+yaw only, force-game-mode flag only, integrated PvP/flight
    true, no command-block enable gate, `GameProfileCache` /
    `usercache.json`, `prepareLevels` / spawn-chunk logic is
    server-startup only.
  - **Known gap, not a port gap:** a size lerp in progress
    desyncs across parked levels — the parked Nether resumes its
    lerp from the mirrored state when re-entered, while vanilla's
    Nether border ticks continuously. Parked levels don't tick at
    all in the Rust (upstream model), so this rides with that.
  - **Deferred:** remove 1.21.9-only game rules from the table
    (`spawn_monsters`, `pvp`, `command_blocks_enabled`,
    `spawner_blocks_enabled`,
    `allow_entering_nether_using_portals` — gamerules item);
    `.notInPeaceful()` (entities item); the datafix file fixer's
    `world_border.dat` code is inert here (file fixes gate at
    `version <= CURRENT` = 4440, the border fixes are 4772+ —
    datafix item's scope). The JSON-RPC management server is
    Horizon-only tooling, left alone.

- **Item 12 — `motion.rs` (3 classes, 61 named methods, 840 diff
  lines):** ten fixes.
  1. **Sprint window option removed:** the `sprintWindow` option
     (settings field, options-file key, control slider) came in
     1.21.9; 1.21.8 hardcodes `sprintTriggerTime = 7` in
     `LocalPlayer.aiStep`. The option is gone from settings, the
     options UI, and the options file; the sprint tick hardcodes 7.
  2. **Sprint flying exemption removed:** 1.21.9's
     `isSprintingPossible(flying)` exempts flying players from the
     shallow-water sprint block; 1.21.8 has no flying clause in
     `canStartSprinting` or `shouldStopRunSprinting`. `can_start`
     and the run branch of `should_stop` now call `possible(false)`.
     (`isMobilityRestricted` is 1.21.9's name for 1.21.8's
     `hasBlindness()`, and `isInShallowWater` for its inline
     `isInWater() && !isUnderWater()` — same predicates.)
  3. **Portal transition screens:** 1.21.9 keeps Chat open through
     a portal transition (`isAllowedInPortal`); 1.21.8 closes every
     screen that is not a pause screen or the death/win screen, and
     Chat is not a pause screen. Chat now closes.
  4. **`Entity.move` no-physics flags:** 1.21.9 resets all collision
     flags when `noPhysics`; 1.21.8 leaves them. Removed the reset
     from `move_entity`.
  5. **Fall-damage resetting trace:** 1.21.9 caps the trace at 8.0
     blocks (`MAX_MOVEMENT_RESETTING_TRACE_DISTANCE`); 1.21.8
     traces the whole movement vector. The cap is gone.
  6. **`Movement` record:** 1.21.9 carries the pre-collision
     movement (`Optional<Vec3>`); 1.21.8 carries a boolean
     (`axisIndependant`). `MovementRecord.original` became
     `axis_independent`: real moves record `true`, merged and
     finalizing records `false`.
  7. **`applyEffectsFromBlocks`:** 1.21.9 orders the axis split by
     the pre-collision movement, caps the blocks at 16 a tick, and
     falls back to a point check at the destination; 1.21.8 orders
     by the actual displacement (`axisStepOrder(delta)`), has no
     cap, and no fallback. The budget and fallback are gone from
     the player and the arrow paths, and `check_inside_blocks`
     takes no budget and returns no count.
  8. **Visited blocks:** 1.21.9 only marks blocks it collided with,
     so a near miss could trigger on a later segment; 1.21.8 marks
     every non-air block visited as the sweep reaches it. The
     player's `check_inside_blocks` now inserts first.
  9. **Wet fire clear:** 1.21.9's `clearFire` keeps negative ticks
     (`min(0, remaining)`), preserving the player's -20 fire
     immunity; 1.21.8's zeroes them. Touching water now sets the
     player's fire ticks to 0.
  10. **Nether portal gate:** 1.21.9 gates the Nether on the
      `allow_entering_nether_using_portals` game rule; 1.21.8's
      integrated `isLevelEnabled` always allows every dimension
      (only a dedicated server's `allow-nether` property gates it).
      `may_enter` now always returns true.
  - **Verified no-change:** `shouldStopSwimSprinting` (identical
    once 1.21.9's `isSprintingPossible(true)` is expanded — its
    shallow-water clause is vacuous); walk distance
    (`length(dx, dz) * 0.6`, a pure move to `addWalkedDistance`);
    `Axis::step_order` (the same Y-first order in both versions, a
    pure move to `Direction.axisStepOrder`); the player-tick
    food/stats/container-close (pure move from `Player.tick` to
    `ServerPlayer.doTick`); crit blindness check (same predicate);
    `remove()` container guard (equivalent); `killedEntity`
    (unused DamageSource param); shoulders (20-tick grace, +0.7
    respawn, CompoundTag save, 1/200 ambient + imitation — all
    match); cape (`moveCloak`: 0.25 lerp, 10-block snap per axis);
    bob (0.4 lerp toward `min(0.1, horizontal speed)`, reset to 0
    while riding); player dimensions (all six poses match 1.21.8's
    `POSES` map, including sleeping 0.2/0.2/0.2);
    `getDefaultDimensions`/`getMainArm`/`isModelPartShown`
    (moved to `Avatar` in 1.21.9, same behaviour);
    `getScoreboard()` (pure move); arrows/projectiles `clearFire`
    `min(0)` (unobservable — `getFireImmuneTicks` is 0 for them, so
    they never hold negative ticks); remote-entity rotation
    (no `moveOrInterpolateTo` equivalent — the Rust interpolates
    with `wrap_degrees` deltas, so the `% 360` difference is
    unobservable).
  - **Upstream gaps, not port gaps:** `ENTITY_MOUNT`/`ENTITY_DISMOUNT`
    game events (fired from different places in the two versions,
    implemented in neither side of the Rust — only the native-mod
    `native_entity_mount` hook exists); the fire-immune fire
    countdown for non-living entities (1.21.8 decrements by 4 a
    tick, 1.21.9 clears — the Rust decrements by 1 for all
    projectiles and has no fire-immune branch; living entities are
    cleared by `LivingEntity.baseTick` in both versions, so only
    fireballs are affected).
  - **Deferred:** `setReducedDebugInfo` override + the F3+F6 debug
    options screen question (debug-entries/debug-overlay items);
    the `allow_entering_nether_using_portals` game-rule table entry
    (the rule was added in 1.21.9 and should not be in 1.21.8's
    table at all — gamerules item); `.notInPeaceful()` on monster
    types (entities/mob-spawning item).

- **Item 11 — `game/lan_actions.rs` (12 classes, 68 named methods,
  785 diff lines):** ten fixes.
  1. **Save format — world spawn and player respawn:** 1.21.8
     stores the world spawn as `SpawnX`/`SpawnY`/`SpawnZ` +
     `SpawnAngle` (yaw only) in level.dat, and the player respawn
     as `{dimension, pos, angle, forced}`; 1.21.9 uses the `spawn`
     compound `{pos, dimension, yaw, pitch}` and respawn
     `{dimension, pos, yaw, pitch, forced}` (RespawnData). The
     level read/write and the player respawn read/write now use
     the 1.21.8 form (pitch does not survive a round-trip).
  2. **Spawn search cap:** the 1024 candidate cap is 1.21.9's
     `PlayerSpawnFinder.ABSOLUTE_MAX_ATTEMPTS`; 1.21.8 has no
     `PlayerSpawnFinder` and `adjustSpawnLocation` tries every
     column of the square (cap `i32::MAX`).
  3. **Shoulder entities:** removed the extra `flying` gate in
     `parrot_to_shoulder` — 1.21.8's `Player.setEntityOnShoulder`
     checks passenger/onGround/water/powderSnow only. (The shoulder
     logic moved from `Player` to `ServerPlayer` in 1.21.9; the
     docs now cite the 1.21.8 home.)
  4. **Block-placement attempt:** 1.21.8 counts any `BucketItem`,
     empty or full; the non-empty requirement came in 1.21.9.
     Empty buckets and milk buckets now count (the milk bucket is
     a `BucketItem` that `bucket_contents` answers None for).
  5. **Chat HIDDEN check:** 1.21.9 exempts commands from the
     hidden-chat refusal; 1.21.8 refuses both messages and
     commands. `try_handle_chat` lost its `command` parameter.
  6. **Drowning:** exactly `-20`, not `-20` or less — the `<=`
     `shouldTakeDrowningDamage` came in 1.21.9. Air that jumps
     past -20 without landing on it does not drown (test pinned).
  7. **Shield block:** removed the `blocked <= 0` early return in
     `after_block` — 1.21.8 runs `hurtBlockingItem` and
     `blockUsingItem` even when nothing was blocked (the `> 0`
     gate came in 1.21.9), so a zero-block hit still damages the
     shield per its damage function, knocks back, and disables.
  8. **Armadillo brush:** 1.21.8's `brushOffScute` spawns one
     scute outright; the `brush/armadillo` loot table (and
     `dropFromEntityInteractLootTable`) came in 1.21.9. Rolling
     the missing table dropped nothing — both the player and the
     dispenser paths now spawn a fixed scute.
  9. **`/rotate` for guests:** 1.21.8's `ServerPlayer.forceSetRotation`
     sends the rotation packet only; the server's stored rotation
     waits for the client's next move packet. (1.21.9's 4-arg form
     sets it at once via super.) The `as_guest` swap-back now
     sends the packet with the new rotation without updating the
     guest's stored yaw/pitch.
  10. **Datafix test:** `level_data_of_1_20_moves_to_the_new_fields`
      expected the 4548/4658/4771 fixes, all inert for to=4440;
      rewritten as `level_data_of_1_20_keeps_its_1_21_8_fields`.
  - **Verified no-change:** chest sounds hardcoded (1.21.9's
    per-block getters are 1.21.9-only); `RespawnConfig` already
    yaw-only; idle timeout tick-based with no wonGame exemption;
    container openers player-only (no `ContainerUser`);
    `getAnalogOutputSignal` Direction param and `isClientSide()`
    -> field (Java only); fire clear (mobs clear fire ticks for
    fire-immune, same net effect as 1.21.8's `LivingEntity.baseTick`
    clear); `killedEntity` (identical override, unused DamageSource
    param); `shouldDropLoot`/`doMobLoot` (`death_loot_table` applies
    the 1.21.8 gate); sleep tick (wake on dead/bed-gone only, no
    `canInteractWithLevel`); the doTick food/stats/container-close
    (pure move from `Player.tick` to `ServerPlayer.doTick`, same
    guard and order); `loadGameTypes` (inlined identically in
    1.21.9); WorldBorder safe zone (pure rename
    `getDamageSafeZone` -> `getSafeZone`); `getSlotForHand`,
    `asLivingEntity`, `EntityReference.of` (Java API only);
    `TeleportTransition` (covered by `RespawnConfig` + the spawn
    finder).
  - **Upstream gap, not a port gap:** 1.21.8's debug-sample
    subscription (`debug_sample_subscription` packet + server-sent
    `debug_sample` packets) is implemented on neither side of the
    Rust — same as the 1.21.8 debug custom payloads (item 10).
  - **Deferred:** `/setworldspawn` (1.21.8 takes a single `angle`
    arg and fails outside the Overworld; 1.21.9 takes a `rotation`
    arg) — commands item's fix.

- **Item 10 — `game/remote.rs` (14 classes, 85 named methods, 1,490
  diff lines):** six fixes.
  1. **Chat queue:** 1.21.9's `ChatListener.tick` freezes the delay
     clock while paused and `flushQueue` resets it; 1.21.8 has no
     pause handling and `clearQueue` does not reset it. Removed the
     pause branch from `tick_chat_queue`; `flush_chat_queue` became
     `clear_chat_queue` with 1.21.8 semantics.
  2. **Unattended commands:** 1.21.9's signed-argument case opens a
     signed confirmation window (Suggest / Copy to clipboard);
     1.21.8 just logs the error and does nothing. `SignatureRequired`
     now logs and refuses; the prompt is plain Yes/No
     (`CommandAccept` removed).
  3. **Server list:** removed `accepted_code_of_conduct` from
     `ServerEntry` (1.21.9-only `ServerData` field) and its NBT
     read/write; `save_servers` no longer copies it;
     `store_code_of_conduct` removed (the prompt can never open).
  4. **Resource-pack lang keys:** `manageServer.resourcePack.*` ->
     `addServer.resourcePack.*` (verified in both jars' en_us.json:
     1.21.8 has only the `addServer.*` keys).
  5. **Entity interpolation:** 1.21.9's `interpolateTo` skips the
     update when an interpolation is active and the target is
     unchanged; 1.21.8 always overwrites the data (steps reset to
     full). Removed the guard (`Start::Unchanged` and `PosRot::is`
     went with it).
  6. **Loading screens split back into 1.21.8's two screens:**
     1.21.9 unified them (chunk map + "Downloading terrain" +
     progress bar + 500 ms close delay + eased progress). 1.21.8 has
     `ReceivingLevelScreen` (multiplayer: text only, portal
     background of the reason, 30 s wait from screen creation,
     `narrator.ready_to_play` on close) and `LevelLoadingScreen`
     (single player: chunk map + `loading.progress` percentage, no
     bar, plain background, raw listener progress,
     `narrator.loading.done` on close). `draw()` is now the
     multiplayer screen; `draw_single_player()` is the single-player
     one; the close delay, the smoothing (`smooth()`), and
     `loading_progress_at` are gone; the 30 s timeout fires while
     waiting for the server too.
  - **Verified no-change:** `packetProcessor()` -> `minecraft`
    (threading API, no Rust equivalent); `seenPlayers`;
    PlayerRotation relative flags and SetDefaultSpawn RespawnData
    (mechanical port handled); `startRiding` 3rd arg (client call
    sites equivalent); `startWaitingForNewLevel` (Rust resets fully,
    as 1.21.8's fresh manager does); ContainerSetData null check
    (Rust is None-safe); `determineLevelLoadingReason` (same logic);
    `isClientSide()` -> field (Interaction, PolarBear, Squid,
    ClientSideMerchant); PlayerSkin identical; MultiPlayerGameMode
    (debug logging + equivalent restructure only).
  - **Upstream gap, not a port gap:** 1.21.8's `handleCustomPayload`
    dispatches ~19 debug payloads to debug renderers; 1.21.9 moved
    debug to dedicated packets. Horizon implements neither, same as
    26.2/26.3 — no code change.
  - **Deferred:** sprintWindow option + `shouldStopRunSprinting`
    (item 12, motion.rs); PlayerInfo skin secure check (item 30,
    skins.rs); `setReducedDebugInfo` (debug entries item);
    ItemPickupParticle render-state refactor (item 14); the
    disconnect-screen flow (item 20, title.rs); whether 1.21.8 has
    the F3+F6 debug options screen (debug-overlay item).
- **Zero-diff items 183, 233 (rank 2) and 356, 372, 382, 383, 384,
  385, 396 (rank 3):** all named classes byte-identical in
  decompiled output, none added/removed — no change. (The diff
  tool's semantic hash flagged them; the decompiled sources match.)

### Rank 3: 7/163 (zero-diff items, see above)

### Rank 4 content: closed 2/2 (chain)

1.21.9 renamed `minecraft:chain` to `minecraft:iron_chain` at
DataVersion 4541; 1.21.8 uses `minecraft:chain`. Fixed four places
that used the 1.21.9 id: mineshaft `keep` lists + hanging-chain
state, bastion blackstone replacement (`iron_bars` -> chain),
`rail_or_chain` rule-test set, and the rule_test_bounds test.
Verified fine: datafix_scene.py (version-gated >= 4541), the
adapter id tables (old-version tables correctly use `chain`),
the 4541 rename fix (inert), datafix/tests.rs (already `chain`).

### Rank 5: closed 2/2 (sample-verified)

The 650-class unported claim was checked with a 14-class random
sample (seed 42): 10 have no Rust reference at all, 4 are name-only
mentions (doc comments, data keys), not ports. The 3 new classes
(GlyphEffectRenderState, GuiRenderState$LayeredElementConsumer,
ContextNbtProvider) have no Rust reference. A byte-based recount
gives 566 top-level unnamed changed/removed vs the report's 650 —
expected, since the report hashes semantic tokens, not bytes.

### 1.21.9-only features: dead by data, no code change

- **Copper golem:** the spawn path is gated on the
  `minecraft:copper` block tag, which 1.21.8's data does not define
  (verified in the client jar); the statue/lightning path needs
  statue blocks, which cannot exist; the entity type is absent from
  the registry, so `create_triggered` returns None.
- **Shelf:** interactions need Shelf block entities; none can exist.
- **Mannequin:** entity type absent; all Rust references are
  reactive match arms on `m.kind.id`.
- **Copper chest:** created only by the (dead) copper-golem spawn
  path.
- **Code of conduct:** the packet variant is gone from the protocol
  crate (mechanical rewire); the server-side `code_of_conducts`
  field has no consumer; `enable-code-of-conduct` defaults false.
- **Creative tabs / recipe book:** no copper_golem, shelf, or
  mannequin references; no spawn-egg references outside tests.

### Debt (not fixed here)

- `crates/horizon-port-pack/src/carry.rs` RENAMED_IDS still renames
  chain -> iron_chain when carrying packs (correct for a 26.3
  target, wrong for 1.21.8; Horizon-only tooling, no gate coverage).
- `tools/test_arrow_parity.py` and
  `tools/test_section_mesh_parity.py` use `iron_chain` in test
  fixtures (non-gate harnesses).
- Two `#[test]` functions index `types.types["minecraft:copper_golem"]`
  (mobs/render.rs) and `["minecraft:mannequin"]` (entity_nbt.rs)
  after loading prepared data; both return early when the prepared
  files are absent, so they skip in this tree, but they would panic
  if the data existed without those types.
- Dead tags are inherited 26.3-isms from the unported 1.21.11
  behaviour; triage belongs at 1.21.11, not here (same as 1.21.7 /
  1.21.9).
