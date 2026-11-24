# Port notes: Minecraft 1.21.10

1.21.10 is the first adjacent delta port in the **Components era (E6,
1.20.5–1.21.11)**, taken from the era template 1.21.11. Unlike the 26.1
patch deltas, which shared a protocol version, this one crosses one:
protocol **774 → 773**, world version 4671 → 4556, data pack 94.1 → 88.0,
resource pack 75.0 → 69.0. The port is the mechanical layer: the pin is
set, all six generated tables are regenerated from 1.21.10's own jar, the
one packet change is a pure rename, the command-parse oracle is ported
back to the pre-permissions API, and the cross-version adapter ladder is
re-based to the 773 pin. The behaviour code in this tree is still the
26.3 base's, inherited through 1.21.11; the port is **in progress**, not
parity-verified.

## Provenance

- Base: the programme's completed 1.21.11 tree (main@2e2ef22: protocol
  rewire d8479b1 + adapter re-base to the 774 pin, 6/6 tables, protocol
  tests 438 pass / 2 fail), with the measured 1.21.11 → 1.21.10 delta
  re-applied (method (b) from the 26.1 carries).
- 1.21.10's facts, from its own version JSON and prepared store: protocol
  **773**, world (data) version **4556**, data pack format **88.0**,
  resource pack format **69.0**, Java **21**, stable release.
- The specification sources were 1.21.10's client and server jars,
  deobfuscated with Mojang's official mapping tables by the programme's
  mappings front-end. The prepared store was reused from the 26.1
  adapter re-base work (verified download + front-end remap + datagen
  already proven there); nothing Mojang-derived is committed in this tree.
- Diff 1.21.11 → 1.21.10: **719 ranked work items** (rank 1: 10,
  rank 2: 454, rank 3: 211, rank 4: 42, rank 5: 2). Classes +119 −441,
  585 moved, 3,174 changed (17,044 methods); 2,527 of the changed or
  removed ones concern Horizon code or harnesses. Registries +1 −6,
  17 changed; blocks unchanged in count (29,671 states), 3 changed;
  command nodes +118 −250, 87 changed.

## The packet change: a pure rename

The only packet change in the diff is clientbound id 40:
`minecraft:mount_screen_open` removed, `minecraft:horse_screen_open`
added — same id, same shape. Verified by javap against 1.21.10's named
client jar: `ClientboundHorseScreenOpenPacket(int, int, int)` with
`getContainerId()`, `getInventoryColumns()`, `getEntityId()`, matching
the Rust `MountScreenOpen { container, columns, entity }` field for
field. The rename was applied across 13 files
(`MountScreenOpen` → `HorseScreenOpen`, `MOUNT_SCREEN_OPEN` →
`HORSE_SCREEN_OPEN`, `mount_screen_open` → `horse_screen_open`,
`handleMountScreenOpen` → `handleHorseScreenOpen`), including the
adapter translate layers and `tools/lan_e2e.py`. No rewire was needed:
no packet is absent or reshaped in 1.21.10 relative to 1.21.11.

## Tables: 6/6 regenerated

| Generator | Result |
|---|---|
| `version_consts` | regenerated (+8/−8) |
| `packet_ids` | regenerated (+3/−3, the mount→horse rename) |
| `entity_tracking` | regenerated (+0/−4) |
| `lan_command_order` | regenerated (+3/−4) — after the oracle port below |
| `command_suggestions` | regenerated (+0/−5) |
| `vanilla_command_order` | regenerated (+0/−3) — after the packet rename below |

Two failures on the first regen run, one root cause: the 1.21.11
`VanillaCommandParseOracle` did not compile against 1.21.10's jar
(18 javac errors). The oracle was ported to 1.21.10's API, with every
change verified by javap against the named jars:

- **No permissions package.** `net.minecraft.server.permissions` does
  not exist in 1.21.10. `CommandSourceStack` takes an `int` permission
  level (same 9-arg shape otherwise); the oracle's `Src` constructors
  take `int` now, and `ReloadableServerResources.loadResources` is
  passed `2` — the function-compilation level vanilla reads from
  `server.properties` (`function-permission-level`, default 2).
- **No stopwatches.** `net.minecraft.world.Stopwatches` does not exist
  and `MinecraftServer` has no `stopwatches` field; the oracle's
  stopwatches line was removed.
- **`ResourceKey.location()`, not `identifier()`.** 1.21.10's
  `ResourceKey` has `location()` returning `ResourceLocation`; all six
  `identifier()` uses (recipe id, component key, block tag member,
  by-registry sort/map/key) were changed.
- **No timelines.** `Registries.TIMELINE` does not exist in 1.21.10;
  the timelines export is an empty array.

The second regen run then failed at the Rust probe build only: the
regenerated `ids.rs` had renamed `MOUNT_SCREEN_OPEN` →
`HORSE_SCREEN_OPEN` while `packets/play.rs` still referenced the old
name (6 errors). The packet rename above fixed it, and the third run
regenerated `vanilla_command_order`, completing 6/6.

## Adapter ladder: re-based to the 773 pin

The cross-version adapter ladder was re-based whole to the 773 pin,
following the ff7d657 / 2e2ef22 method and reusing the 11 prepared
legacy versions (no re-prepare). Changes, by family:

- **Packet-id tables:** the `horse_screen_open=mount_screen_open`
  arbitration in `tools/generate_packet_id_adapter.py` was removed
  (in this tree the pin name is `horse_screen_open`, so 1.21.3–1.21.10
  need no arbitration) and the reverse arbitration added for
  1.21.11/26.x. `step_1_21_10` is now the pin (identity); a step for
  1.21.11 (774) was added; `v1_21_11_ids.rs` became a real 774→773
  table instead of a pin-identity table.
- **Item/component id space:** 1.21.10 has 96 data component types
  (1.21.11: 104). Eight 1.21.11-only components are gone
  (`minimum_attack_charge`, `use_effects`, `piercing_weapon`,
  `kinetic_weapon`, `swing_animation`, `damage_type`,
  `zombie_nautilus/variant`, plus the `kinetic_weapon` skip helper);
  every remaining component id was re-keyed by registry name in the
  converters, the hash tables, and all test fixtures (e.g.
  `potion_contents` 49 → 42, `trim` 54 → 47, `instrument` 59 → 52,
  `banner_patterns` 70 → 63). Four entities were removed 774 → 773
  (`camel_husk` 20, `nautilus` 88, `parched` 97, `zombie_nautilus`
  152); none added.
- **SlotDisplay trap carries forward.** The SlotDisplay codec still
  uses 26.3 numbering (item=4, smithing_trim=8) while 1.21.10's
  registry is 8 entries in a different order; the five hand-corrected
  tables from the 1.21.11 re-base were carried forward with their
  corrections re-keyed to the 773 pin, and MUST be regenerated to
  old→pin when the behaviour port rewires the codec — same standing
  trap as in 1.21.11's notes.
- **Two bugs found and fixed in passing:** a double translation in the
  v1_20_3 converter (`*kind == 13` translated twice, 13→10→7) and a
  `..=` range the batch regex missed (`1..=3 | 19 | 46` →
  `1..=3 | 16 | 39`).

Verification: `cargo test -p horizon-protocol` **438 pass / 2 fail**
— the same split as the completed 1.21.11 tree, with only the two
pre-existing sandbox-network tests failing
(`connection::tests::a_dead_address_does_not_hold_up_the_next`,
`resolver::dns::tests::asks_each_server_in_turn_and_retries_truncated_replies_over_tcp`),
in untouched code, out of scope. `cargo check --workspace` is clean;
`cargo check --workspace --all-targets` is clean (0 errors, 0 warnings).

## Oracle ports to the 1.21.10 API (for horizon-prepare)

`horizon-prepare` compiles the exporter mains against the pinned jar,
and CI's `port-verify` fails the job if prepare fails — so the
exporter mains must compile against 1.21.10's jar. The first harness
run's prepare failed with 72 javac errors across the mains. All were
1.21.11-only API uses, fixed with javap evidence from 1.21.10's named
jars:

- **Identifier → ResourceLocation.** Mojang renamed
  `net.minecraft.resources.Identifier` to `ResourceLocation` in
  1.21.11; 1.21.10 has only `ResourceLocation`, with the same API
  surface (parse, fromNamespaceAndPath, tryParse, withPath, getPath,
  getNamespace). Global rename: 524 occurrences in 149 oracle files,
  plus `.identifier()` → `.location()` (61 occurrences, all receivers
  verified `ResourceKey`-typed) and `Holder.id()` →
  `unwrapKey().orElseThrow().location()` (7 occurrences in 3 files).
- **ChunkPos.pack → ChunkPos.asLong.** 1.21.11 renamed the static
  `asLong(int, int)` to `pack`; 1.21.10 has only `asLong`. 26
  occurrences in 8 files.
- **VanillaBlockLandingOracle.** `net.minecraft.world.level.gamerules`,
  `net.minecraft.server.permissions`, and `net.minecraft.world.attribute`
  do not exist in 1.21.10; the fake `GameRules` now overrides
  `getBoolean(Key<BooleanValue>)`, the fake server returns int
  permission levels (`operatorUserPermissionLevel()`,
  `getFunctionCompilationLevel()`, `isEpollEnabled()`), and the fake
  entities extend `monster.Zombie` / `animal.Turtle` (no
  `monster.zombie` / `animal.turtle` subpackages in 1.21.10).
- **VanillaCommandExecOracle.** Permission levels are ints in 1.21.10:
  `GAMEMASTER` → 2, `OWNER` → 4 (verified by `javap -c` on the
  1.21.11 `PermissionLevel` enum's `<clinit>`); `useNativeTransport()`
  → `isEpollEnabled()`.
- **VanillaCuboidModelOracle.** 1.21.10's `BlockElementRotation` is a
  single-axis record `(origin, axis, angle, rescale)`; the rotation
  export maps axis + angle bits.
- **VanillaFaceVisibilityOracle.** 1.21.10 has no
  `LeavesBlock.setCutoutLeaves` toggle; the cutout loop runs without
  the setter (both rows agree in 1.21.10).

### The jsr305 problem

After those ports, prepare failed again — `javac` crashed with
`CompletionFailure: class file for javax.annotation.Nullable not
found` (exit 4). Root cause, verified by scanning class bytes:
**1.21.10's server jar has 1,219 classes referencing
`javax/annotation/Nullable`; 1.21.11's has zero** — Mojang migrated
off jsr305 in 1.21.11, but never shipped the jar as a library in any
version. A per-main bisection (all 16 server mains compiled alone)
found exactly one trigger in the prepare set:
`VanillaBlockLandingOracle`; the other 15 compile clean without it,
and `FaceVisibility` compiles clean after its fix above.

The fix is in this tree: `tools/jsr305-3.0.2.jar` (20 KB, BSD,
SHA-256 `766ad2a0…928d0c7`, verified against Maven Central) is
committed, allowlisted by exact path in `tools/check_public_tree.py`
(following the existing `ORIGINAL_JDK_TOOLS` pattern) and negated in
`.gitignore`, and appended to the javac classpaths in
`crates/horizon-prepare/src/lib.rs` and the two Python classpath
builders (`tools/test_rust_parity.py` `verify_artifacts()`,
`tools/port_regen.py` `classpaths()`), presence-gated so trees
without the jar are unaffected. With it, `VanillaBlockLandingOracle`
compiles clean. Every version before 1.21.11 will need this; the fix
carries forward with the tree under method (b).

Deliberate scope line: the wider oracle corpus still uses
1.21.11-only APIs (`world.attribute`, `gamerules`, `permissions`,
`monster.zombie`, `Stopwatches`/`TIMELINE`) across dozens of files —
those are per-harness oracle ports, i.e. behaviour-port programme
work, not this port. The two oracle failures in the harness profile
below are of that kind.

## Harness subset (measured on this VM)

9 of the 17-harness subset measured locally on this VM (run 2,
2026-11-12 01:55–02:39 UTC, 43.1 min for 14 runs; commit b5dd4a6).
**1 pass / 4 oracle-compile-fail / 4 fail.** The 8 game-binary
harnesses (hotbar, item_model, lan, light, options, resource_pack,
save_roundtrip, sound_event) were not measured locally; CI covers
them. (`version_port.py`'s own summary counts 1 pass / 2 oracle /
6 fail — its classifier only recognises chunk_generation and
command_exec as oracle; first_person and food are oracle-compile
failures too, counted here honestly.)

| Harness | Verdict | Cause |
|---|---|---|
| block_properties | **pass** (117 s) | — |
| chunk_generation | oracle | 24 javac errors: no `ChunkStatus.TERRAIN` (×4), `RandomState.create` signature, `ChunkPos` |
| command_exec | oracle | 44 javac errors: `ContainerInput` class, `npc.villager` package, `Util`, `teleportToPortalDestination`, `ChunkPos.asLong()` |
| first_person | oracle | 25 javac errors: `LocalPlayer` ctor signature, missing overrides |
| food | oracle | 8 javac errors: no `gamerules` package, `ServerLevel` ctor signature |
| command_parse | fail | Runs to a real verdict: **7,488 mismatches**. Rust still has `stopwatch` / `compute` / `posteffect` commands 1.21.10 lacks; permission 0: 32,365/32,367 match; permission 2: 28,631/32,367 |
| component_patch | fail | Oracle compiles but crashes at runtime: `ClassNotFoundException: net.minecraft.core.component.Removed` (1.21.11-only class) |
| datafix | fail (6/6 shards) | Fixture-world generation: `datafix_worlds.py` hit a truncated Mojang manifest download (`IncompleteRead`) — environmental, not behaviour; not re-run locally |
| entity_components | fail | Harness-side Python bug: 1.21.10's advancement entity predicate is a list, the harness indexes it as a dict (`TypeError`) |

Run 1 (01:31–01:36 UTC) died during command_parse's probe build
(VM OOM under 4 concurrent rustc); its three verdicts were stale
(pre-oracle-fix) and were re-measured in run 2. The prepare step in
run 2 failed pre-fix on the jsr305 crash above; prepare was
re-verified end-to-end after the fix (`prepare_resources`: binary
173.2 s, all exporters compiled and ran; launcher assets, sounds,
panorama steps ok). One local-only hiccup: `lab.download` (urllib)
truncated the 27.json asset index twice on this VM's flaky network
(two different wrong hashes); a curl-fetched copy, SHA-1-verified
against the manifest, was placed in the launcher tree and the
prepare then passed. CI downloads fresh on reliable network, so
this should not recur there.

## What remains, by cause

### Behaviour work from the diff (not started)

The 719 ranked items are the port's work list; **0 of 719 are closed**.
Largest first: the data-component id-space shift (−8 types, every
component id re-keyed), the registry shifts (item −17, sound_event
−67, entity_type −4, sensor_type +6 −2), and the 3,174 changed
classes. The behaviour code in this tree is still the 26.3 base's.

### Harness-oracle era ports

Measured in this pass (error counts from the run-2 harness logs):
the chunk-generation oracle (24 errors: 1.21.10 generates through
the separate `createBiomes` / noise / surface / carver / feature
stages — no `TERRAIN` status, no `buildTerrain`, plus a
`RandomState.create` signature change), the command-exec oracle
(44 errors: `ContainerInput`, `npc.villager`, `Util`,
`teleportToPortalDestination`, `ChunkPos.asLong()`), the first-person
oracle (25 errors: `LocalPlayer` ctor, missing overrides), the food
oracle (8 errors: no `gamerules` package, `ServerLevel` ctor), the
component-patch oracle (compiles, crashes at runtime on the
1.21.11-only `Removed` component type), and the entity-components
fixture builder (1.21.10's advancement entity predicate is a list,
the harness indexes it as a dict). The command-parse oracle is the
one oracle ported in this pass (see above); the exporter mains were
ported for prepare (see above).

### Full harness profile

The CI `port-verify` workflow on main completes the profile on GitHub
runners.

## Cost experiment

This port is the programme's falsification experiment for the cost
model: one adjacent delta port, timed end to end, measured against
1.21.11. Times are reconstructed from session file timestamps (VM
clock, UTC) and are approximate to about ±10 minutes per bucket.

- **Wall clock:** start ~2026-11-11 23:45 UTC (task spawn; repo clone
  and environment setup first), end 2026-11-12 02:55 UTC (publish
  commit e8d85d1). Total: ~3 h 10 min. (Corrected 2026-11-12: the
  end time was estimated 15 minutes late before the commit landed.)
- **Work items closed vs total:** 0 / 719 behaviour items closed. The
  port closed the infrastructure layer only: pin, 6/6 tables, one
  packet rename, one oracle port, the adapter re-base. No behaviour
  item was attempted; the behaviour code is the 26.3 base's.
- **Harness profile:** 9 of 17 measured locally: 1 pass
  (block_properties), 4 oracle-compile-fail (chunk_generation 24
  errors, command_exec 44, first_person 25, food 8), 4 fail
  (command_parse 7,488 mismatches; component_patch oracle runtime
  crash; datafix fixture-download failure; entity_components harness
  bug). 8 unmeasured locally (game-binary harnesses; CI covers them).
- **Sweep minutes:** run 2: 43.1 min for 14 runs (9 harnesses),
  dominated by cold probe builds (~36 min: client_probe ~27 min,
  world_probe ~9 min); harness execution itself ~7 min. Run 1:
  ~20 min, died during command_parse's probe build (OOM). Total
  sweep wall: ~63 min across both runs. A warm re-sweep (probes
  cached) would be ~7–10 min.

### Where the time actually went (approximate)

Buckets are activity time and overlap (the 1.21.9 prepare ran during
harness run 1; oracle edits landed in the tree during run 2), so they
sum to more than wall clock. Uncertainty about ±10 minutes per bucket.

| Activity | Time |
|---|---|
| Prepare / diff tooling (clone, prepared-store reuse, env, diff, report) | ~20 min |
| Table regen + oracle rework (3 regen runs, command-parse oracle port, packet rename) | ~25 min |
| Adapter re-base (6 item/component batches, fixture regen, test-fix passes) | ~55 min |
| Verification (protocol test runs, workspace checks) | ~15 min |
| Oracle rework for prepare (Identifier rename, BlockLanding/CommandExec/FaceVisibility ports, jsr305 investigation + fix) | ~35 min |
| Harness runs / debugging (run 1 OOM death ~20 min, run 2 43 min, cause triage ~10 min) | ~75 min |
| Behaviour-item porting | 0 min |
| Publish (docs, notes, audit, commit) | ~25 min |

The story this delta tells is that the cost of an adjacent delta port
is dominated by **id-space re-keying across the adapter ladder and its
fixtures** — not by behaviour work (none was done) and not by the
oracle corpus (one oracle, one port, ~15 minutes of the regen bucket).
The single biggest sub-cost was the component/entity id shift
propagating through converters, hash tables, and pinned test fixtures,
including two oracle-regeneration passes over the v1_20_3/v1_20_5/
v1_21_1/v1_21_4/v1_21_5 fixtures. A second adjacent delta should cost
less: the 773-pin tables and fixtures are now the starting point, and
the re-keying scripts (`batch*.py`, `apply_regen.py` in the agent
workdir) are reusable.

### Cold-seed split (approximate, from run logs — no reruns)

- **Probe build: ~36 min** (client_probe fast-profile ~27 min,
  world_probe release ~9 min). Dominates the cold seed by an order of
  magnitude. The 4-job build also OOM-killed run 1; a programme-wide
  `jobs = 2` cargo config was added at `~/workspace/tools/cargo/config.toml`
  during this port.
- **Oracle javac: seconds per oracle** (chunk_generation oracle
  compile failed in 18 s total incl. JVM start; command_exec in 24 s;
  the full exporter bundle compiles in ~35 s wall for both sides).
- **Prepare: ~2–3 min** (game-files check 6.4 s, assets copy 11.6 s,
  worldgen bundle 2.0 s, datagen 76.2 s cold / 23.5 s warm, exporter
  javac ~35 s).

Conclusion: **pre-warming the probe-build cache is the lever** (redraft
§9.3); oracle javac and prepare are negligible by comparison.

## Status

Pin set (protocol 773, world 4556); 6 of 6 tables regenerated;
packet rename closed (mount→horse, same id, same shape);
command-parse oracle ported to the pre-permissions API; adapter
ladder re-based to the 773 pin; exporter mains ported to the 1.21.10 API and jsr305 on the javac classpath (prepare
verified ok end-to-end);
workspace compiles; protocol tests 438 pass / 2 fail
(sandbox-network pair only). Behaviour port: rank 1 closed (10/719).
Harness profile (9 of 17 measured locally): 1 pass, 4 oracle, 4 fail.

## Behaviour port — rank 1 (10 items, CLOSED 2026-11-24)

- Item 1 components (verified, no code change): the wire layer is
  1.21.10's — the component id table in
  `crates/horizon-protocol/src/item.rs` lists 56 named consts with the
  8 removed types absent (`ATTACK_RANGE`, `DAMAGE_TYPE`,
  `ZOMBIE_NAUTILUS_VARIANT` dropped vs 1.21.11's 59; the other five
  never had consts), and `skip_dropped` reads-and-drops all 8 with
  their 1.21.11 stream codecs. No code references the removed id
  constants (only doc comments, plus the unrelated entity-data
  serializer `ZOMBIE_NAUTILUS_VARIANT`). Behaviour fallout — code
  that reads the removed components by name (`items.rs` use_effects,
  `hand.rs` swing/attack-range, `spear_use.rs` kinetic_weapon) —
  belongs to the rank-2 items for those modules.
- Item 2 protocol (verified): the mount→horse rename is complete
  (`HORSE_SCREEN_OPEN` = 40); zero `MountScreenOpen` remnants in the
  tree.
- Items 3–8 regenerate (verified): `regen-determinism.json` shows all
  six generators reproduce the committed tables byte-identical, with
  no overrides.
- Item 9 save-format (PORTED from the 1.21.11 port's 2b1a914, with one
  1.21.10 adjustment): 1.21.10 has no `FileFixerUpper`
  (`net/minecraft/util/filefix` is absent from the named server jar)
  and no `FILE_FIXING_REQUIRED` backup status (`BackupStatus` is only
  NONE/DOWNGRADE/UPGRADE_TO_SNAPSHOT, javap-verified from the named
  client jar). Deleted fix_1493/4772/4773/4899,
  `level_dat_to_saved_data` and their helpers; `requires_file_fixing`
  is always false; the whole-world upgrade follows the pre-26.x
  layout (confirmed against the real 1.21.10 world in
  vanilla-world-mc: flat `scoreboard.dat`/`chunks.dat` under `data/`,
  DIM-1/DIM1, `playerdata/`, `advancements/`, `stats/`);
  `saved_data_type` keeps only the types 1.21.10's `DataFixTypes`
  defines — 1.21.11's set minus stopwatches (`SAVED_DATA_STOPWATCHES`
  and `DEBUG_PROFILE` are not in 1.21.10's enum, javap-verified).
  `DataFixers`: 1.21.10's fix versions are a strict subset of
  1.21.11's (247 vs 256; the 8 extras are all above 4556, plus one
  schema version, 2556), so the CURRENT-gated fix table is correct
  as-is. The 45 changed datafix classes are mechanical noise only
  (sampled `EntityCatSplitFix`, `ChunkHeightAndBiomeFix`,
  `EntityRenameFix`: `isEmpty`→`length`, `Util` moved packages,
  `MutableObject.get`→`getValue` — no behaviour change), so the
  Rust fix implementations are untouched. Files:
  `datafix/{files,mod,progress,world_upgrader,tests}.rs`, `save.rs`
  (`needs_file_fixes` doc + test assertion), `datafix_world.rs`
  (doc header). `requires_file_fixing` is kept as an always-false
  stub — its full removal is rank-2 item 205's, matching the 1.21.11
  port's item 21. NOT in item 9: the `save.rs` settings store still
  writes `world_gen_settings.dat` / `world_clocks.dat` /
  `game_rules.dat` as saved-data files (26.x) instead of level.dat's
  `Data` — that rewire is rank-2 save_roundtrip work, as in 1.21.11.
- Item 10 facts (verified): `version.rs` reads protocol 773, data
  version 4556, data pack 88.0, resource pack 69.0.

## Behaviour port — rank 2

Triage (2026-11-24, `port-1.21.10-triage/`): a batched javap jardiff
of all 2,472 rank-2/3 classes, 1.21.11 client jar vs 1.21.10 client
jar — 1,801 common, 452 in 1.21.10 only, 219 in 1.21.11 only, 1,758
method-changed, 0 javap failures — crossed with the per-module tree
diff (1.21.11 tree vs 1.21.10 tree). Verdicts: 129 carry-over (no
named method changed; apply the 1.21.11 port's hunks), 34 no-change
(module identical in both trees and no named method changed), 291
review (a named method — or a lambda inside one — changed, or a
widely-named item with no single module). Per-item verdicts and
evidence in `rank2-progress.json`.

Batch 1 (34 no-change): 201, 207, 260, 264, 270, 279, 300, 318, 330,
346, 352, 361, 364, 365, 367, 369, 389, 390, 391, 392, 396, 397, 398,
399, 427, 431, 434, 435, 438, 439, 440, 441, 443, 444. Each module is
byte-identical between the trees and none of its named methods
changed 1.21.11→1.21.10 per the jardiff (lambdas counted against
their enclosing method — item 182's `MappedRegistry.freeze` went to
review on exactly that: `lambda$freeze$5/7` changed).

Batch 2 (80 carry-over): the 1.21.11 port's hunks applied by
three-way merge (`git merge-file`, base = the mechanical-publish base
2e2ef22, ours = the 1.21.10 tree, theirs = current 1.21.11) — 285
carry-over files + 70 files the 1.21.11 port touched that no delta
item covers (survive.rs, live.rs, block_survival.rs among them: the
719-item delta only tracks 1.21.11→1.21.10 differences, so 89
port files have no item). Hand-merged where the E6 triage flagged
1.21.11-specific or uncertain units; 1.21.11-only parts reverted with
jar evidence: the Horse/ZombieHorse DANGER_OTHER/DAMAGE_OTHER -1.0
ctor malus (1.21.10's ctor returns before it), nautilus feeding
(no nautilus in the 1.21.10 jar), the Timeline/WorldClock system
(no `world/timeline`, ClockManager or WorldClock in the jar;
tracks sample at day time instead), the preedit overlay (no
PreeditEvent/IMEPreeditOverlay/preedit.png — preedit.rs deleted).
Jar-verified carry-overs: shulker `orientation` (present in
1.21.10), piglin 30-degree baby ears (no 5-degree angle),
VillagerTrades bytecode-identical (modulo the npc→npc/villager
package move), baby dims (Zombie/Fox/Piglin/ZombifiedPiglin
BABY_DIMENSIONS values match 1.21.11's), texture renames
(temperate_pig et al. present in 1.21.10), Particle.getLightColor
(identical `hasChunkAt` guard), warnOnLowDiskSpace (present in
both), ShareToLanScreen/WaterlilyBlock/EndDragonFight (all
1.21.10's names). Also in this batch: item_model.rs rebuilt with
the 96-entry 1.21.10 DATA_COMPONENT_TYPES list (1.21.11's 104
minus the 8 removed) and no per-model `transformation` (no such
field in 1.21.10's Unbaked records); game.rs, save.rs (ImpulseSave,
`chunks` saved-data id, DragonFight/ScheduledEvents in level.dat
— verified against the real 1.21.10 world), villager_trades.rs
(new file), ice_spike.rs + random_patch.rs (new files; 26.x-only
`minecraft:spike` removed with spike.rs); requires_file_fixing
deleted (item 205); the age-lock native hook dropped from
native-hooks.json. `cargo check -p horizon-world --lib` and
`-p horizon-client --lib` both clean. 49 carry-over items stay
Batch 3 (4 carry-over + all 22 batch-2 CI failures fixed): the
pack-format fixtures (datapack/packs at 88/69), Realms headers
(1.21.10/773/Java 21), the GameRules form (level.dat's `Data.GameRules`,
camelCase keys, string values — verified against the real 1.21.10
world's 59 rules; the fire pair maps to/from
`fire_spread_radius_around_player`, the `disable*` rules invert), the
shipwreck `height_adjusted` removal (simple.rs, jar-verified: neither
version's ShipwreckPiece writes it), the tracking fallback for mod
thrown types (from the 1.21.7 port), and the 1.21.10-form test fixes:
datafix tests (no `difficulty_settings`/`game_rules`/`id` palette keys
— all 1.21.11 fixes above 4556), the optimization stamp start (4550,
not the 26.x 4950), the horse-malus test (no malus in 1.21.10),
painting/variant id 89 (as in 1.21.7), the sign-text test removed (no
SIGN_TEXT in 1.21.10's DataComponents), vibration particle id 48
(jar-verified registration order), server-packs header 69.0, the
pause-screen loop ([1, 2] — Share to LAN is singleplayer-only),
en_us.json wording, launch-reference.md numbers, bossbars `#[ignore]`
(needs prepared resources), `record_is_old` at 4295 and
`BlendingDataFix` at 4295 (both jar-verified; the 26.x table's 4997
is past CURRENT 4556, so every chunk read as old and pre-1.18
chunks kept no blending data). horizon-world fully green locally
(482 passed, 0 failed); level_save_keeps_changed_rules verified
locally. 45 carry-over items stay open behind 27 hand-merge files
(SlotDisplay renumbering, spear, hand.rs, item.rs, menu/hub, cauldron,
furnace, component, frames, end.rs, recipe_book, advancements).

## Rank 2, batch 4 — remaining carry-over blockers (45 items, Nov 24, 2026)

Closes the last 45 carry-over items (43 by merge, 2 by no-change
evidence). All 129 carry-over items are now closed; rank 2 stands at
163/454, with the 291 review items remaining.

Hand-merges, each jar-verified against 1.21.10's own jars:

- `game/end.rs`: EnderDragonFight → EndDragonFight; the flash
  ambient colour ([0.99, 1.12, 1.0], `END_FLASH_SKY_LIGHT_COLOR`,
  0.99f/1.12f in 1.21.10's LightTexture) and the flash tick from
  game time (1.21.10's ClientLevel calls
  `EndFlashState.tick(getGameTime())`).
- `component.rs`: ObjectContents `fallback` removed — the record is
  identical in both versions (only `contents`, no fallback field).
- `furnace.rs`: canBurn needs room for one more item only
  (`getCount() >= getMaxStackSize()`), burn grows the output by one
  (`grow(1)`), serverTick resets the timer when the ingredient has
  no recipe — all three byte-identical between the jars.
- `game/commands/items.rs`: createItemStack does NOT validateStrict
  afterwards — byte-identical in both jars, no validateStrict call;
  the damageable+stackable rejection and `item_validation_components`
  are deleted.
- `game/frames.rs`: `isOutsideBuildHeight` (both jars), the frame
  board/needle owner (ItemFrame.setDirection uses get2DDataValue in
  both; NeedleDirectionHelper exists in both client jars), the map
  bounds widening removed.
- `block_entity_render.rs`: sign boards as entity models
  (SignRenderer/HangingSignRenderer + AttachmentType exist in
  1.21.10; the sign block models are empty), copper-golem statue
  textures (`exposed_copper_golem.png` etc. in 1.21.10's assets),
  banner cloth at `entity/banner_base.png`, the end-gateway beam at
  `entity/end_gateway_beam.png`, the enchanting-table book at
  `entity/enchanting_table_book.png`, copper chests wearing the
  christmas textures (1.21.10's getChestMaterial checks xmas before
  copper: ENDER → CHRISTMAS → TRAPPED → copper → REGULAR), the sign
  atlas-revision cache.
- `environment_attributes.rs`: the 3 26.1-only light attributes
  removed (absent from both jars — 1.21.10 has no
  `net/minecraft/world/attribute` classes at all; the table is the
  engine's environment model carried from 1.21.11), turtle-egg
  hatch default 0.002 → 0.0.
- `packets/recipes.rs` + `recipe_book.rs` + `lan/advancements.rs` +
  adapter ids: the SlotDisplay codec rewired to 1.21.10's 8-entry
  registry (empty, any_fuel, item, item_stack, tag, smithing_trim,
  with_remainder, composite — identical order in both versions; the
  26.x with_any_potion/only_with_component/dyed do not exist here).
  The hand-corrected tables are gone: v1_21_8/v1_21_10/v1_21_11_ids
  are now `Remap::Same`, and v26_1_2/v26_2_ids carry the 26.x →
  1.21.10 remap (26.x 4/5/6 → 2/3/4, 8/9/10 → 5/6/7).
- `container_screen.rs`, `game/remote_stations.rs`, `lan/hub.rs`,
  `menu.rs`, `cauldron.rs`, `game/enchant_effects.rs`: merged to
  0 diff (ClickType in both jars, no ContainerInput;
  CauldronInteraction at `net/minecraft/core/cauldron/` in both;
  `doLungeEffects` only in 1.21.11).
- `game/spear.rs`: comments reworded only — `lungeForwardMaybe`
  exists only in 1.21.11's Player; 1.21.10 has no spear lunge.

Deliberately NOT merged, with jar evidence:

- Top-level `spear.rs`: the 1.21.11 changes are spear dismount
  thresholds and sway timing for a mechanic 1.21.10 lacks
  (iron_spear absent from the 1.21.10 jar). Items 275 and 322 close
  as no-change on this evidence.
- `protocol/item.rs`: the whole diff is 1.21.10's own 96-component
  numbering (correct) plus 1.21.11-only components (ATTACK_RANGE,
  DAMAGE_TYPE, ZOMBIE_NAUTILUS_VARIANT, kinetic_weapon); 1.21.10's
  skip_dropped arms for the 8 removed components are correct as-is.
- `game/hand.rs`: only the `Entity.updateFluidOnEyes` comment
  applies (the method exists in 1.21.10's Entity). The spectator
  min-range and getActiveItem changes concern `attack_range`, which
  1.21.10 lacks entirely (no `entityAttackRange` in its
  LivingEntity); the code stays as inert 26.3-base dead code.
- `packets/play.rs`: only the Horse/Mount rename remains — 1.21.10's
  form is correct and kept.

Batch-3 CI fixes (run 45305592498: 2 failures, down from 22):
the realms upload test now asserts `GameRules` (the 1.21.10 vanilla
save form from batch 3, not the 1.21.11 `game_rules` key), and the
86 bundled lang files were synced from 1.21.11 (pure stale-key
removals: "Fills the screen (borderless).", "Fullscreen takes the
display over, at the mode below.", "The display mode exclusive
fullscreen uses. Applies when you leave." — all removed from
en_us.json in batch 3, so the lang test's no-stale-keys check
failed). Lang test verified locally passing.

Checkpoint verification (Nov 24, continuation session): the
8007a10b WIP checkpoint moved the ledger 173/719 -> 237/719 by
closing 64 rank-2 items no-change on the semantic bytecode diff.
The continuation verified all 64 against review-per-item.json
(all present; 63 with real=0, missing=0) and against the actual
code (each module diffed 1.21.10 vs 1.21.11 tree, numbers and
version mentions normalised). 63 stand as closed.

One correction — item 126 (spectator_menu.rs): the semantic diff
was right (all 4 named methods same), but the 1.21.10 tree still
carried the 26.3-base form — the 1.21.11 item-122 hunk (83456f28)
was missed by the carry-over (spectator_menu.rs absent from
carryover-units). Carried over verbatim in this commit: the team
icon's colour square now fades with the menu alpha via
color_from_float (was argb(255,...)), doc citations reworded to
the 1.21.10 names, regression test added. Jar-verified: 1.21.10's
SpectatorGui has renderHotbar/renderAction/getHotbarAlpha and
SpectatorMenuItem has renderIcon, so the 1.21.11 names hold in
1.21.10. Item 126 reclassified no-change -> ported; count
unchanged at 237/719 (rank 2: 227/454).

Minor correction — item 136 (witch.rs): evidence now records 2
named methods checked, not 3 (the triage title counts 3 but
named_changed lists 2; both same). Verdict unchanged.

Lesson for the remaining ranks: a no-change verdict needs BOTH
the semantic diff AND a tree-diff check that the 1.21.10 tree
already contains the 1.21.11 behaviour — the carry-over unit
list missed at least one file.

CI triage (Nov 24, continuation session):
- Run 45292840975 (rank 1 head ce7a7b4c): failure — baseline; superseded.
- Run 45298555004 (batch 2 head e24143e9): 22 failures
  (datafix/saved_templates/datapack/packs/lan) — all fixed in
  batch 3, all port regressions, none pre-existing.
- Run 45305592498 (batch 3 head 0864c72c): 2 failures —
  lang::tests::bundled_files_are_served_and_have_no_stale_or_repeated_keys
  (client-core) and
  realms::tests::a_generated_upload_world_carries_only_what_the_realm_needs
  (client-lib, asserted the 1.21.11 `game_rules` key) — both
  fixed in batch 4; batch 4's run shows both suites green.
- Run 45307802187 (batch 4 head 5919dcb0): 6/7 suites green
  (protocol 440, items 110, world 483, client-core 83, client
  1286, client-lib 2510 — all 0 failed); only
  horizon-client-render failed, on one compile error E0063:
  the sign_text_cache test's Kind::Sign initializer missed the
  `wood`/`attached` fields batch 4's block_entity_render merge
  added. Fixed by adding wood: "oak", attached: false, matching
  the 1.21.11 test file. No pre-existing failures identified so
  far — every CI failure on this branch has been a port
  regression, fixed in the following batch.

CI run 45315118303 (batch 5 head d481e74d): render suite 2067 passed,
2 failed — banner_cache (8 vertices, expected 16) and
copper_statue_cache (0 vertices, expected 4). Root cause: the test
file kept the 26.3-base sprite names (entity/banner/banner_base,
entity/copper_golem/copper_golem_*) while the implementation uses
the 1.21.10 names. Verified against the 1.21.10 client jar:
textures/entity/banner_base.png exists at top level (no
banner_base.png under entity/banner/), and the golem textures are
exposed/weathered/oxidized_copper_golem.png. Fixed the test file
(8 names); it is now byte-identical to 1.21.11's. The implementation
was already right.

Rank 2 batch 7 (sample of the real-method tail): the 622 "real"
methods include normalisation gaps (GameRules API, float-vs-double
trig, renames) and genuinely different but unmodelled methods.
Each closure below cites the method diff + the Rust-side check.
Item 57's module diff (Horse/ZombieHorse pathfinding malus) is a
second item-126-style carry-over gap, resolved jar-side: 1.21.10's
Horse/ZombieHorse/AbstractHorse constructors set no malus, so the
1.21.11 malus hunk is 1.21.11-only and correctly absent.

CI run 45318465400 (head 3b4d2c4e, sprite fix + batch 7): SUCCESS —
first fully-green crate-tests run on this branch (all 7 suites).

CI run 45322694411 (head 1061fc67, batches 9-12: piglin spear, furnace
Math.random, spawn-mobs gate): SUCCESS — all 7 suites green.

CI run 45326110344 (head 54819e07, batches 15-16: sea-pickle bonemeal,
moving-border current size, slime moon-brightness two-draw): in progress
at the batch-17 commit; conclusion to be recorded with the next batch.

Batch 17 (rank 2 372 -> 380, total 390/719): 3 ported, 5 no-change.
261 equip.rs: 1.21.10's ItemAttributeModifiers.compute(base, slot) applies
EVERY slot-matching modifier regardless of attribute (bytecode-verified;
its only caller is Mob#getApproximateAttributeWith) — added
approximate_with for the mob-looting comparison, kept attribute_with
filtered for gear.rs/legacy_rules.rs (real per-attribute application).
71 particles.rs: 1.21.10's ClientLevel#tick ticks the border first and
unconditionally; ungated tick_remote's border.tick(). (ServerLevel#tick
gates it on runsNormally in 1.21.10 too — the server side was already
right.) 68 enchant_effects.rs: PlaySoundEffect is a single soundEvent in
1.21.10, a per-level list in 1.21.11 — simplified to the single sound
(channeling/soul_speed data confirmed single-string). No-change: 34
(Christmas range Dec 24-26 in both), 85 (chunk save/flush unmodelled),
116 (singleCodec is codec plumbing; JSON shape same), 117 (spear path
unreachable in 1.21.10 — Kinetic::of matches only 1.21.11 spear ids;
full spear-system removal is a deferred cross-cutting cleanup, no
triage item covers spear.rs), 173 (raid spawn-pos trig is f32-vs-f64
noise on a random candidate).

CI run 45326110344 (head 54819e07, batches 15-16): SUCCESS — all 7 suites
green. CI run 45327263625 (head 0cd7e906, batch 17: attribute compute,
border tick, play_sound): SUCCESS — all 7 suites green.

Batch 18 (rank 2 380 -> 388, total 398/719): 3 ported, 5 no-change.
172 lan/data.rs: SYNCHRONIZED trimmed to 1.21.10's 21 registries
(javap-verified against the 1.21.10 server jar): removed
zombie_nautilus_variant, world_clock, timeline, decorated_pot_pattern,
block_transformer, worldgen/block_state_provider. sulfur_cube_archetype
kept (Horizon-only, in neither vanilla jar). 202 maps.rs: Nether map
markers spin on the day time in 1.21.10 (getDayTime), the game time in
1.21.11 — the rotation chain (add_decoration, add_item_decorations,
toggle_banner, tick_framed, decoration_rotation) now takes day_time;
holder tracking stays on game time. 25 foxes.rs: Fox.canHoldItem tests
FOOD only in 1.21.10 (isConsumableFood is 1.21.11-only). No-change: 15
(criterion renames), 27 (panda trig noise; MOB_DROPS modelled), 61
(video-settings layout is menu chrome), 69 (options migration + toast
rename unmodelled), 84 (villager renames; dragging assign already the
1.21.10 form).

Batch 19 (rank 2 388 -> 396, total 406/719): 1 ported, 7 no-change.
40 copper_golems.rs: turnToStatue's lead drop is gated on the entity-drops
rule in both versions (RULE_DOENTITYDROPS / ENTITY_DROPS) — the Rust
dropped it unconditionally; gated on ui.rules.bool("entity_drops").
No-change: 23 (wolf anger is already the 1.21.10 int countdown; the NBT
anger_end_time long form belongs to item 11), 65 (AbstractGolem package
moves + needsSync rename + f32 sin), 72 (font pipeline refactor; debug
dump rename), 99 (Netty ping plumbing unmodelled), 119 (frame_survives
already checks blocks + entities + border, the 1.21.10 noCollision form;
1.21.11's hasLevelCollision is blocks + border only), 229 (bonemeal
emits ITEM_INTERACT_FINISH at the player in both), 278 (screen-level
double-click state unmodelled; container double-click identical).

Batch 20 (rank 2 396 -> 404, total 414/719): 0 ported, 8 no-change,
ledger only. 100 isMoonVisible is natural + dayTime%24000 in
[12600, 23400] (javap-verified; isNaturalNight just delegates to it)
and the Rust's render_tick computes exactly that window. 109 Mth.sin/cos
index the table in double (1.21.11) vs float (1.21.10) — at most one
table step, noise; clampedLerp/clampedMap are parameter-order swaps
with identical semantics. 130's phantom rule resolves through the
gamerule name-translation table (spawn_phantoms <-> doInsomnia).

Batch 21 (rank 2 404 -> 412, total 422/719): 3 ported, 5 no-change.
76 piglin.rs: removed the 1.21.11-only piglin spear behaviours
(SpearApproach/SpearAttack/SpearRetreat: SpearStatus, Memories,
PiglinSpear goal, NoMeleeWithSpear — spear_use.rs lines 519-898 — plus
the piglin state's spear field); initFightActivity now registers plain
MeleeAttack. converting() already the 1.21.10 form (presets:
piglins_zombify true overworld/end, false nether = !piglinSafe()).
96 stations.rs: the netherite-upgrade smithing icon list is 9 in 1.21.10
(nautilus_armor + spear are 1.21.11 additions); smithing_slot_icons
narrowed 11 -> 9 with its test. Stonecutter scrolling reset and the
instrument description were already the 1.21.10 form. 33 remote.rs:
ShelfBlock#useItemOn returns SUCCESS on the client unconditionally in
1.21.10, so an empty main-hand click now consumes the click (the
UseItemOn packet is sent before this check). No-change: 114 (guardian
needsSync rename + trig noise), 301 (the falling-block fixture probes
landing only; the ENTITY_DROPS drop rule is unmodelled in it), 29
(step_move_and_hit is single-nearest, the 1.21.10 findHitEntity form),
48 (DataVersion defaults to -1; backup stamp is local time), 181
(ambience biome lookup path differs, same biome).

CI run 45330007250 (head 8f229a80, batches 18-19): SUCCESS, all 7 suites
green.

Batch 22 (rank 2 412 -> 420, total 430/719): 2 ported, 6 no-change.
98 music.rs: the music volume source + selection order, ported to the
1.21.10 form (javap-verified against Minecraft#getSituationalMusic).
Layer::parse reads effects.music (weighted list) + effects.music_volume
instead of the 1.21.11 environment attributes; BackgroundMusic is the
biome's list with a weighted pick; Situation follows screen > menu >
End (boss END_BOSS else END) > biome list (an empty list plays nothing)
> underwater (global, tag-gated or already playing) > creative (global,
never Nether) > GAME; the fade sets the current instance's volume, not
the MUSIC category. The situational-music parity oracle does not compile
against 1.21.10 jars (1.21.11-only RegistryLayer/attribute APIs) — the
fixture was updated to the new shape, the oracle is a separate job.
18 equine.rs/mounts.rs: no horse eats a red mushroom in 1.21.10 (the
handleEating branch and the zombie_horse_food tag are 1.21.11-only;
1.21.10 has neither, javap-verified); a zombie horse passes while
untamed and uses AbstractHorse.mobInteract when tamed. No-change: 47
(evoker griefing gap + trig + same-level difficulty), 21 (villager
activity windows match the 1.21.10 Schedule exactly, javap-verified;
restock day is day_time/24000), 53 (dragon trig + MOB_DROPS gap +
WindCharge move), 131 (fishing trig + criterion renames + game-event
form), 59 (title splash dates identical; menu chrome), 26 (criterion
renames).

Batch 21 follow-up (in the batch 22 commit): the piglin spear removal
also dropped the melee goal's crossbow gate, which broke the lib test
compile (holds_non_melee_weapon was deleted with the spear code).
Restored as NoMeleeWithCrossbow in piglin.rs — vanilla 1.21.10's
MeleeAttack behaviour does not start while holding a usable projectile
weapon (javap-verified), so a crossbow piglin shoots instead of
meleeing; only the spear half of the old gate is gone. CI run
45332512840 (batch 21 head) FAILED on exactly this compile error.

CI run 45336648733 (head 8c5305fa, batch 22): SUCCESS, all 7 suites
green.

Batch 23 (rank 2 420 -> 428, total 438/719): 4 ported, 4 no-change.
199 border_render.rs: the renderer reads the border's current size now
(1.21.10's getMinX() etc.), not the partial-tick interpolation; the
no-arg WorldBorder::bounds() is fixed the same way (javap-verified:
1.21.10 reads the extent directly, 1.21.11 delegates to getMinX(0.0F),
the previous size — this corrects item 112's "same value" claim for a
moving border). 31 undead.rs: the zombified piglin loses its spear goal
and its 1-in-20 golden-spear roll (always a golden sword); the drowned
loses its zombie-nautilus jockey; the husk loses its camel-husk-parched
jockey (a triage gap — Husk#finalizeSpawn was assigned to no item), in
both finalize.rs and the game's spawn_jockeys path, which also loses
the zombie-horse jockey with its iron spear (ZombieHorse#finalizeSpawn
is 1.21.11-only). The 1.21.10 jar has no ZombieNautilus, camel husk or
parched at all. Removing SpawnData::camel_husk_size also unblocked
StructureMobFinalizer::load, which returned None on 1.21.10 data for
want of a camel_husk type — no structure mob was being finalized.
Zombie's SpearUse stays (item 12). 32 commands.rs: /spawnpoint and
/setworldspawn store the respawn angle as given (1.21.11's
RespawnData.of wraps the yaw and clamps the pitch). 169 lightning.rs:
spawnFire gates on doFireTick only (1.21.11's canSpreadFireAround is a
1.21.11-only method, absent from the 1.21.10 jar); the copper cleaning
never unwaxed, already the 1.21.10 form. No-change: 24 (the STAB render
branch is dead — a kind-2 swing is constructed only for spears; jeb
wool lerps linearly, the 1.21.10 form; parrot package move; hitbox
submit unmodelled), 67 (chat insertion-click mode unmodelled; menu
chrome), 70 (smithing armor-stand preview unmodelled; turtle eggs use
the level day-time window; armor-stand griefing reads the rule), 14
(computeSpeed unmodelled; minecart max speed via the camelCase table;
TNT fuse + spawner gate read their rules; command-cart interact has the
same outcome as usedBy).
Batch 24 (rank 2 428 -> 436, total 446/719): 2 ported, 6 no-change.
11 entity_nbt.rs: the NeutralMob anger save is the 1.21.10 form now —
AngerTime (int, remaining ticks) + AngryAt, not 1.21.11's anger_end_time
(long, absolute game time) + angry_at (javap-verified against NeutralMob
on the 1.21.10 server jar). 1.21.10 has no getPersistentAngerEndTime
anywhere, so the absolute-end-time model was removed wholesale:
Extra::anger_end and Extra::anger_stamp are gone, neutral_anger.rs keeps
only persistent_anger + stop, and the countdown ticks down once per tick
the mob ticks and freezes while it does not (missed ticks never count —
the rewritten h13076 test pins 999999 - 2 after two ticks, not 999999 -
7). Save/load/tests rewritten in entity_nbt.rs, persist.rs (incl. the
h262 enderman tests), commands/data.rs, anger_tests.rs, and the
enderman_anger_saved_fixture.py harness tool. Deferred: the bee's own
anger save (item 20), the bee/wolf metadata sync + wolf_anger_corpus.py
+ the mob_anger harness op (item 168). 46 native_hooks.rs: the End has
weather in 1.21.10 — level_can_have_weather dropped the 1.21.11-only End
exclusion (verified: the_end.json has has_skylight=true, has_ceiling=
false; advanceWeatherCycle is gated only on hasSkyLight + doWeatherCycle;
non-overworld dimensions read weather through DerivedLevelData, so the
End shares the overworld's weather — which the Rust's single shared
weather state already matches). No-change: 77 (hub.rs stats unmodelled;
suspendFlushing is network optimisation; GameRules API; permission ints;
view-distance null check trivia), 97 (entity-selector suggestions use
permission level >= 2, the 1.21.10 form; Coordinates::position matches
the 1.21.10 inlined local-coordinates formula exactly), 17 (no isFinite
check on delta movement, the 1.21.10 form; computeSpeed unmodelled; trig
noise; handlePortal method-location move; leash CONSUME is a client
InteractionResult distinction the Rust does not model; FAST_LAVA vs
ultraWarm unmodelled in boats.rs — datapack dimensions are item 157's;
animatePaddle arg-order + trig noise; MOB_GRIEFING API), 87 (book/lectern
screen chrome), 92 (debug-entry renames; the Rust already handles both
inF3/inOverlay statuses), 22 (all 16 Realms screen methods are menu
chrome or renames; the settings-save name trim is a whitespace edge case
on a Realms screen).

Probe repairs (with batches 23+24): the batches 23+24 check exposed two
test-compile breaks in commands/data.rs (a dangling `{end}` format capture
left by the item-11 rewrite, and a `failures` vec whose `String` type lost
its inference pin when the anger_end assertions went — both fixed), and the
first-ever client_probe check on this branch exposed 7 pre-existing
fixture breaks: audio_attributes_fixture.rs still used batch 22's old
BackgroundMusic API (default/creative/underwater + select — rewritten to
the biome-list form with a per-line fixed-seed Situation pick),
environment_fixture.rs read Sky.block_light_tint/ambient_light_color,
removed by batch 2's carry-over in favour of ambient_light/ambient_color
(the two puts dropped — 1.21.10 has no such environment attributes), and
environment_system_fixture.rs built Conditions with the removed `clock`
field (now `day_time`). The probe compiles clean now; note the parity
oracles still do not compile against 1.21.10 jars, so these fixtures are
compile-verified only, not parity-run.

CI run 45345350796 (batches 23+24, head c6512557): SUCCESS — all 7
suites green, 12 minutes. No pre-existing failures identified so far on
this branch; every CI failure has been a port regression fixed in the
following batch.

Batch 25 (8 items, Nov 24, 2026; rank 2: 436 -> 444, total 454/719):
3 ported, 5 no-change. 20 bee.rs: the bee's anger from the 1.21.11
absolute-end-time model to 1.21.10's remaining-ticks countdown
(javap-verified: Bee.DATA_REMAINING_ANGER_TIME is an int;
updatePersistentAnger restarts the timer ONLY on target change, not
every tick while held, decrements every tick with stop at 0, and stops
when the stored target is a dead/absent Mob, never for a stored player).
Bee.anger_end -> anger: i32; entity_nbt.rs saves/loads the bee's
AngerTime + AngryAt (the bee's AngryAt was never saved before); the sync
sends Value::Int at index 18. wantsToEnterHive and isAngryAt already
match 1.21.10 (the End has no day timeline, matching the != END
exclusion). The wolf's metadata sync stays item 168's. 49 motion.rs:
modifyInput scales by a flat 0.2f for any item in use (not the per-item
use_effects multiplier), and any item use keeps a sprint from starting
(isUsingItem, not isSlowDueToUsingItem) — game.rs and game/prediction.rs
now hard-code Some(0.2) / is_some(); the Item use_speed/can_sprint
fields stay parsed but unread. isSprintingPossible is a NO-CHANGE
despite the reorder: javap-verified hasEnoughFoodToSprint is
isPassenger() || food > 6 || mayfly, so the food gate only ever bites a
non-passenger, exactly what sprint.rs possible() already does.
isAlwaysFlying is a pure rename of isSpectator (javap: both are
localPlayerMode == SPECTATOR); the 1.21.11 jumpableVehicle check in the
swim-jump branch is unmodelled in the Rust's movement tick. 91
mipmap.rs: the mipmap generation from 1.21.11's MipmapStrategy system
to 1.21.10's alphaBlend form (javap-verified: hasTransparentPixel once
from the base, transparent pixels skipped, sums / 4.0, each channel
(int)(Math.pow(mean, 1/2.2) * 255.0), computed alpha below 96 cut to 0;
POW22[i] = (float)Math.pow((float)i/255.0f, 2.2)). Strategy, the .mcmeta
mipmap_strategy/alpha_cutoff_bias parse, solidify, fill_empty_dark, the
alpha-coverage system and the sRGB tables are all removed; the sprite
cache format is bumped to version 2; the dither (a Horizon enhancement,
off by default) is kept, keyed on has_transparent. 1.21.10's
TextureMetadataSection has only blur/clamp, so the 1.21.11 keys are
unknown fields the codec ignores (glint validation updated). New tests
pin the verified values (135 for one visible pixel in four, 0 for a
faint alpha below the cutoff, 186 for white+black alpha). No-change:
180 (anisotropy already ported in batch 2; the minWidth/minHeight
max is a no-shrink re-stitch the Rust's fresh-texture path does not
need; the mip-0 async skip is scheduling), 38 (the SPEAR arm pose and
the STAB check are dead — no piglin can hold a spear in 1.21.10;
crossbow charge already passes int ticks, the 1.21.10 form;
getArmPose is just return EMPTY in 1.21.10), 144 (Camera renames +
trig noise), 45 (server console source, scoreboard plumbing,
permission ints, no game-rule-change hook, loop scheduling), 36 (the
Rust already uses int distances at the RandomPos API boundary, the
1.21.10 form; package moves and criterion renames). Also fixed a
batch-24 leftover: finalize_load_tests.rs used anger_end_time:5000L,
ignored since item 11 — now AngerTime:5000.

Batch 26 (8 items, rank 2 444 -> 452, 462/719). 93 raids.rs: the
1.21.11 GameplayAttributes system is deleted outright; the raid gate
is DimensionFacts.has_raids (renamed from can_start_raid, which had
no readers; datapack dimensions default to false, matching vanilla),
the patrol gate is the hardcoded day-5 check plus the
#without_patrol_spawns biome tag (single member mushroom_fields).
PatrolLevel gained day_time(); the day check sits between the
countdown reset and the bright check, matching 1.21.10's javap
order. 58 fog.rs: the DimensionOrBoss fog environment
(javap-verified: 7 environments in 1.21.10, DimensionOrBoss between
Water and Atmospheric; applies when the dimension is foggy at the
camera - always true for the Nether - or a boss overlay wants world
fog). New FogType::DimensionOrBoss, resolved in frame_fog; start
rd*0.05, end min(rd,192)*0.5, sky/cloud = end; Atmospheric
hard-coded to the 1.21.10 constants with the boss min-block removed.
30 melee.rs: the item-swap ticker is 1.21.11-only (no itemSwapTicker
field, no getItemSwapScale in 1.21.10) - swap_ticker, swap_scale()
and reset_attack() are gone, the hand height uses the attack-strength
scale; plus Player#tick wakes a sleeper when bright outside (new
Game::bright_outside, also used by the patrol level). 19
creatures.rs: melee reach is always sqrt(2.04)-0.6 (no attack_range
component in 1.21.10); sun_burn_tick gates on bright-outside and the
monsters_burn field is deleted; the wet check includes powder snow.
12 mobs.rs: zombie spear goal + spear equipment roll + zombie-horse
jockey all removed (javap-verified 1.21.11-only); the
can_glide_through launch-blocker fixed by mirroring the 1.21.7 tree
(empty set at load, no glide exception on climbables - 1.21.10 has no
such tag); disable-blocking reads the weapon unconditionally;
knockback fallback uses math_random(); armadillo babies' timers never
count down and the mob_drops gate is gone; dolphins grace sets the
water drag to 0.96. 16 lan_actions.rs: handlePlayerAction case 8
(spear STAB) and guest_jab removed (1.21.10's switch ends at 7);
entering spectator no longer stops item use. No-change: 73
(core_sky - JOML interface/class types, trig noise, render-pass
plumbing; the single-rotation sun/moon/stars form is geometrically
identical to the Rust's separate rotations provided moon_angle =
sun_angle + 180, an item-351 attribute-value question), 42 (camera -
Level/BlockGetter types, JOML types, pick plumbing, render scheduling;
the 1.21.10 renderLevel fog boolean confirms the item-58 analysis;
the player's attack_range component read is inert on 1.21.10 data).

CI run 45357219692 (batch 25, head d72cd46d): FAILURE - 2 tests in
horizon-client-render, both batch-25 regressions from the mipmap
rewrite: custom_item_textures_mip_as_items and
identical_still_textures_share_a_layer asserted that item textures
get a layer of their own. The old item flag only gated solidify,
which the 1.21.10 form has no use for (vanilla 1.21.10's
MipmapGenerator has no item parameter, javap-verified), so identical
pixels now mip identically and share a layer - the tests were
updated to the 1.21.10 form (item shares layer 0 with the block
texture; counts 2 layers / 4 names / 2 mips / 2*16*16*4 pixels),
not the code. All other suites green (440 + 3 + 110 + 483 + 83 +
1285 + 2510 passed). Fixed in this batch (sprites.rs).

Batch 27 (2 items, rank 2 452 -> 454, 464/719; RANK 2 COMPLETE). 13
game.rs: startAttack loses the cannotAttackWithItem gate and the
PIERCING_WEAPON branch, continueAttack loses the piercing gate
(1.21.11's spear attack; the ATTACK_RANGE check was never modelled in
the Rust's start_attack). The HandWorld spear methods stay for the
deferred spear cleanup, docs noted. jukebox_pop_position now draws X,
Y, Z: 1.21.10's Vec3 has no offsetRandomXZ (javap-verified) and both
the jukebox and the composter use offsetRandom in 1.21.10; a test pins
the draw order. No-change in 13: isSpawningMonsters is identical in
both versions (javap-verified; 1.21.10 DOES have RULE_SPAWN_MONSTERS),
disconnect is a method split, getSituationalMusic was item 98's,
hasControlDown is a pure rename (1.21.10's checks
EDIT_SHORTCUT_KEY_MODIFIER, exactly 1.21.11's hasControlDownWithQuirk),
and the rest are GameRules renames or unmodelled plumbing. 52
debug_keys.rs: no-change — the Rust's keybind defaults are exactly
the 1.21.10 hardcoded debug keys (F3/A/B/C/D/G/H/I/L/N/P/S/T/V/1/2/3/
F4/F6); the rebindable surface is a Horizon feature on top, and F3+Q
is unmodelled in both trees.

Rank 3 batch 1 (108 items, rank 3 0 -> 108, 572/719). All named-0
modules whose current tree diff vs 1.21.11 is zero (73 no-change) or
comment-only version labels (34 carry-over) plus stitch.rs (499: its
one named method is unchanged, diff 0). Every file was spot-checked
for 1.21.11-only content (spear/nautilus/parched/camel_husk/timeline/
attributes/attack_range): the only hits are dead name lists and
empty-on-1.21.10-data registries (shadows, dolphin, mount_menu,
stats_screen, armadillo, legacy_combat, registries' minecraft:timeline
field), each noted in its ledger entry. One stale comment fixed
(cube.rs: the spear minimum-range parenthetical; melee_reaches is
already the ported 1.21.10 form from item 19).

CI runs 45362866602 (batch 26, 7b9ffd68) and 45364393502 (batch 27,
dd457e07): both SUCCESS, all suites green.

Rank 3 batch 2 (32 items, rank 3 108 -> 140, 604/719): the review
items with zero real methods (semantic diff run fresh for rank 3:
175 methods, 94 real, 14 missing-new, 67 same). 513
gamemode_switcher.rs PORTED: 1.21.10's GameModeSwitcherScreen has no
mouseReleased, so the mouse-release apply is gone (game.rs arm,
switcher_release, release_applies + test); the keyboard release path
is untouched. The other four missing-new items are no-change: 520
(dialog text clicks run through DialogScreen.runAction in 1.21.10),
542 (getItemDamageSource is 1.21.11's rename of getDamageSource —
same mace-smash logic), 527 (getVisibility unreferenced), 547 (the
Rust's JSON-RPC endpoint is its own implementation). The remaining
27 are all-same. Three carry already-ported tree diffs, noted in
their entries: 512 (anger work), 544 (mipmap metadata), 559
(offsetRandom + create_experience).

Rank 3 batch 3 (46 items, rank 3 140 -> 186, 650/719): the review
items with 1-3 real methods, read from review-real-diffs-r3.json.
Six ports. 505 debug_options.rs: getRowWidth is 310 in 1.21.10, 350
in 1.21.11 (one constant). 548 query.rs: 1.21.10's
NetworkDataOutputStream.writeString is DataOutputStream.writeBytes
(the low byte of each char), not UTF-8 — query strings now send low
bytes, with a test. 477 test_instances.rs: the gametest highlight
fill is 0x8000FF00 in 1.21.10, 0x6000FF00 in 1.21.11. 469
management.rs: 1.21.10's validateAuthentication takes a Bearer
token only — the WebSocket-subprotocol token and origin check are
1.21.11's — denying everything with "Invalid or missing API key";
the management-server-allowed-origins property is removed
(1.21.10's SecurityConfig has only secretKey); the handshake echoes
minecraft-v1 when the client offers it. 475
dedicated/management.rs: 1.21.10's gamerule RPC takes an UntypedRule
(a string value: Boolean.parseBoolean / Integer.parseInt, no range
check — IntegerValue.set just sets) and returns a TypedRule whose
value is likewise a string. 567 vanilla.rs: the time command's
clock/timeline nodes were 26.x leakage — both vanilla TimeCommands
are the simple set/add/query form (javap-verified; 1.21.10 has no
world/timeline package) — so the section is the 1.21.11 tree's form
verbatim and clock_nodes is deleted; the Resource::Clock/Timeline
variants and Suggests::TimeMarkers stay as dead pub API. That also
fixes parses_vanilla_commands (time set day, time query daytime),
which was failing silently: horizon-command is not in CI's 7
suites. Three doc fixes on no-change verdicts: 530
(can_spread_fire_around already implements the 1.21.10 two-rule fire
gate via fire_radius), 557 (the level-load callback is compiled-only,
the 1.21.10 isSectionCompiled form), 483 (canSetSpawn reads the
dimension type's respawnAnchorWorks). The other 37 are renames,
empty diffs, trig noise, or render plumbing.
