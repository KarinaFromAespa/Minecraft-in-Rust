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

### Rank 2: 7/317 (items 10, 11, 12, 13, 17 + 2 zero-diff)

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
