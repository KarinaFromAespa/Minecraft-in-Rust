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
  and environment setup first), end ~2026-11-12 03:10 UTC (publish
  commit; approximate). Total: ~3 h 25 min.
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
(sandbox-network pair only). Behaviour port unstarted (0/719).
Harness profile (9 of 17 measured locally): 1 pass, 4 oracle, 4 fail.
