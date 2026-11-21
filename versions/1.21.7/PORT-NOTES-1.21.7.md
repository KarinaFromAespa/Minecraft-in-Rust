# Port notes: Minecraft 1.21.7

1.21.7 is the fourth adjacent delta port in the **Components era (E6,
1.20.5–1.21.11)**, taken from 1.21.8. Like 1.21.9, it shares its
protocol with its neighbour: protocol **772 on both sides**, world
version 4440 → 4438, data pack 81.0 and resource pack 64.0 unchanged.
There is no adapter re-base, no packet rewire and no oracle rework —
the measured delta is just **5 ranked work items**, the smallest yet
(smaller than 1.21.9's 27). The port is the mechanical layer: the pin
is set, the six generated tables are regenerated against 1.21.7's own
jar, the verification gates pass, and the remaining work is sized,
ranked and documented. The behaviour code in this tree is still the
26.3 base's, inherited through 1.21.11, 1.21.10, 1.21.9 and 1.21.8;
the port is **in progress**, not parity-verified.

## Provenance

- Base: the programme's published 1.21.8 tree (main@fb20f190, commit
  b884d16d: cross-protocol delta from 1.21.9, 6/6 tables regenerated,
  protocol tests 430 pass / 2 fail), with the measured 1.21.8 →
  1.21.7 delta re-applied (method (b) from the 26.1 carries).
- 1.21.7's facts, from its own version JSON and generated tables:
  protocol **772**, world (data) version **4438**, data pack format
  **81.0**, resource pack format **64.0**, Java **21**, stable release
  (2025-06-30). (The diff report lists the pack formats as
  `None.None` — a diff-tool artifact; the generated `version.rs` and
  the version JSON both give 81.0 / 64.0.)
- Prepare ran through the pipeline (verified jar download + generalized
  mappings front-end remap + datagen, 2026-11-12 ~14:38–14:41 UTC):
  **9,484/9,484 classes mapped (100%), 9,425 renamed,
  0 placeholders; 83,784 methods mapped (62,742 renamed), 42,848
  fields mapped (42,408 renamed)**. Nothing Mojang-derived is
  committed in this tree.
- Diff 1.21.8 → 1.21.7: **5 ranked work items** (rank 1: 3,
  rank 2: 1, rank 5: 1). Classes +0 −1, 0 moved, **6 changed
  (5 methods); 1 concerns Horizon**. No protocol change, no packet
  changes, no registry/block/item/command changes; data files ~1202
  changed, assets ~0, en_us ~0.

## The delta: a hotfix pin step

Rank 1 is the version pin (`version.rs`), the save-format item
(DataFixers 4440 → 4438: 0 schemas, 0 fixes added, 0 datafix classes
changed) and the version facts. Rank 2 is the one changed class that
concerns Horizon: `SharedConstants` (0 methods changed). Rank 5 is
the 6 changed/removed vanilla classes Horizon does not port (blaze3d
opengl 2, blaze3d/vertex 1, net/minecraft 1, client/multiplayer/chat
report 1, blaze3d 1). There is no behaviour delta to port beyond the
pin: the 1.21.8 behaviour code, adapter ladder, packet set and oracle
ports all carry forward untouched.

## Tables: 2 regenerated, 4 reproduce

| Generator | Result vs 1.21.8 |
|---|---|
| `version_consts` | regenerated (+5 −5 — the pin: 1.21.7 / 772 / 4438 / 81.0 / 64.0) |
| `packet_ids` | regenerated (+1 −1 — header comment only; the packet set is identical) |
| `entity_tracking` | unchanged |
| `lan_command_order` | unchanged |
| `command_suggestions` | unchanged |
| `vanilla_command_order` | unchanged |

The committed `regen.json` lists all six generators (D1 completeness,
per the gate change at main@15ff93a8): the final full regen ran with
no `--only` (2026-11-12 15:05–15:10 UTC, commit 9128f753ed). A fresh
regen reproduces all six committed tables byte-for-byte — see
`regen-determinism.json`, no overrides. (The brief expected the five
SlotDisplay hand-corrected tables to differ from a fresh regen; they
do not — the generators emit the same numbering the hand corrections
carry, as they did for 1.21.9 and 1.21.8.)

Two traps from the regen, for the next agent:

- The `port/1.21.8` branch does **not** contain the 1.21.8 agent's
  post-regen code changes (packet rewire, adapter re-base, oracle
  port): they live only in the published tree (and as uncommitted
  files in that agent's workdir). A regen from the bare branch fails
  at `vanilla_command_order` (`ACCEPT_CODE_OF_CONDUCT` missing from
  `ids`). The fix is to overlay the published 1.21.8 tree onto the
  candidate first — committed here as 7d55fec on `port/1.21.7` so the
  next agent starts from the completed tree.
- The branch's `gen_version_consts.py` handles only the (major,
  minor) pack form; 1.21.7's version.json uses single ints like
  1.21.8. The published tree's both-forms script is the one to use.

## Verification

- `cargo check --workspace --all-targets`: clean, 0 errors,
  0 warnings (11 m 53 s).
- `cargo test -p horizon-protocol`: **430 pass / 2 fail** (432 total —
  identical to 1.21.8, as expected for a same-protocol delta). The 2
  failures are the same pre-existing sandbox-network pair in
  untouched code, out of scope.
- `horizon-prepare` completes end-to-end: exit 0. The
  HORIZON_JAVA_25 datafix routing (v6.4) and
  `tools/jsr305-3.0.2.jar` both carry forward from 1.21.8 (verified
  in the candidate, not assumed).

## Harness subset

Measured locally on this VM (run 4, 2026-11-12 18:31–18:55 UTC,
24 m 25 s; prepare: ok). **2 pass / 2 oracle / 13 fail** of 17 — the
gate's classification from `harnesses.json` (no baseline was passed,
so all 15 non-passing are "new"). Identical to 1.21.8's profile,
as expected for a same-protocol delta with no behaviour change.

- **Pass:** `block_properties`, `lan`.
- **Oracle (gate):** `chunk_generation` (8 javac errors),
  `command_exec` (6 listed).
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
| `command_exec` | oracle compile (`ContainerInput` missing, etc.; the classifier lists 6, the compile log has 71) | 71 |
| `datafix` | oracle compile (`VanillaDataFixOracle`) | 26 |
| `first_person` | oracle compile | 25 |
| `food` | oracle compile | 9 |
| `item_model` | oracle compile | 49 |
| `options` | oracle compile | 1 |
| `resource_pack` | oracle compile | 1 |
| `sound_event` | oracle compile | 3 |

The remaining six fails are real verdicts or harness/environment
causes:

- `command_parse`: real verdict — 7,546 parse mismatches
  (32,319/32,321 commands match at permission 0).
- `component_patch`: the oracle crashes at runtime.
- `entity_components`: harness-side Python bug (the entity predicate
  is a list in 1.21.8; `TypeError: list indices must be integers`).
- `hotbar`: real verdict — 891/905 cases match.
- `light`: real verdict — "light mismatch against vanilla"
  (missing motion tag).
- `save_roundtrip`: the game exits before opening the control socket
  — no display on this headless VM (the same environmental failure
  as the 26.3 baseline).

Datafix reached its true verdict this time without a re-run:
`HORIZON_JAVA_25` was set from the start (unlike 1.21.8's first
sweep), and the fixture worlds were seeded from the 1.21.8 run's
finished worlds — 6 generated fresh (1.8.9–1.13.2), 35 reused. The
worlds are candidate-independent inputs (generated by old-version
servers), so the reuse is exact. Trap: the finished worlds live in
`worlds/` (1.17+) and `worlds-pre-1.17/`; the `gen/` directories are
work dirs the generator ignores — seeding `gen/` does nothing.

Two sweep lessons for the next agent:

- `tools/interaction_lab.py:cargo_example` hardcodes `cargo run -j3`,
  ignoring `CARGO_BUILD_JOBS`/`FLEET_LOAD_JOBS`. On this VM the
  debug `client_probe` build OOM-killed `horizon-client-render`'s
  rustc at `-j3`. The fix was to build the debug probe manually at
  `-j2` with parity's profile env (`CARGO_PROFILE_DEV_OPT_LEVEL=3`
  — without it the fingerprints differ and the harness rebuilds
  everything), then resume the sweep.
- `tools/parity.py` re-runs **all** selected harnesses on every
  invocation (longest-first by recorded seconds); it does not skip
  recorded ones. A killed sweep repeats the whole set, not just the
  remainder. Three runtime restarts killed runs 1–3 here; run 4 is
  the profile.

## What remains, by cause

### Behaviour work from the diff (not started)

The 5 ranked items are the port's work list; **0 of 5 are closed**.
The work is the pin-adjacent behaviour (the `SharedConstants` change
and the save-format step, both rank 1/2) — small, but unattempted.
The behaviour code in this tree is still the 26.3 base's.

### Harness-oracle era ports

The per-harness oracles carry their era gaps forward from 1.21.8
unchanged (chunk-generation, command-exec, first-person, food,
item-model, datafix, options, resource-pack, sound-event) — the
same 9 oracle failures as 1.21.8, with the same error counts.

### Full harness profile

The CI `port-verify` workflow on main completes the profile on GitHub
runners.

## Cost experiment

Same-protocol hotfix delta, recorded for contrast with 1.21.9's
same-protocol delta (~65 min) and 1.21.8's cross-protocol delta
(~10 h 40 min). Times are from session timestamps (VM clock, UTC).

- **Wall clock:** work started 2026-11-12 ~14:32 UTC; end 2026-11-12
  19:15 UTC (publish commit be65dca). Total: ~4 h 44 min.
- **Work items closed vs total:** 0 / 5 behaviour items closed. The
  port closed the infrastructure layer only: pin, tables,
  verification. No behaviour item was attempted.
- **Harness profile:** 2 pass / 2 oracle / 13 fail of 17 (gate
  classification; true oracle count 9 — see above).
- **Sweep minutes:** run 4 (the profile) 24 m 25 s. Runs 1–3 were
  killed (two runtime restarts, one deliberate OOM stop) and their
  ~2 h 30 min of partial sweeps was repeated work, not profile.

### Where the time actually went (approximate)

| Activity | Time |
|---|---|
| Setup + prepare / diff / report | ~10 min |
| Table regen (4 attempts — flaky downloads, dangling symlink, branch-vs-tree traps, disk quota — then the full run) | ~28 min |
| Verification (workspace check 12 min, protocol tests) | ~15 min |
| Harness runs (run 4 profile 24 min; runs 1–3 partial ~2 h 30 min; manual debug-probe build ~17 min) | ~3 h 10 min |
| Determinism regen (first attempt killed by disk quota; retry 6 min) | ~15 min |
| Behaviour-item porting | 0 min |
| Publish (docs, notes, audit, commit) | ~30 min |

The contrast with 1.21.9 (~65 min) is not the delta — 1.21.7's delta
is smaller (5 items vs 27) — it is the VM tax: three runtime
restarts, a disk-quota kill, and the `-j3` OOM in the harness's probe
build. The mechanical work itself (prepare, regen, verify) took
under an hour, as the delta size suggests.

## Status

Pin set (protocol 772, world 4438); 2 of 6 tables regenerated, 4
reproduce byte-for-byte; no packet or adapter change (same protocol);
workspace compiles (0 errors, 0 warnings); protocol tests 432 pass /
0 fail on CI (430/2 locally — the sandbox-network pair passes where
there is network); prepare verified ok end-to-end.
Behaviour port complete 2026-11-19 (5/5, see below). Harness profile:
CI run 44441686979 (main@d37f448, success, 1h35m41s): 2 pass / 2 oracle /
13 fail of 17 (true oracle count 9 — see "CI proof" below). Verdict:
failed (D1 fail, D2 pass, D3 pass, D4/D5/D6 blocked). D6 deferred
(owner). Metadata moved to pins/1.21.7/ with the record.

## CI proof (run 44441686979, 2026-11-19)

port-verify on main@d37f448, success in 1h35m41s. Profile **2 pass /
2 oracle / 13 fail** of 17 — identical to the local run. Passing:
block_properties, lan. Classified oracle: chunk_generation (8 errors),
command_exec (71 errors in its log). The classifier undercounts as
before: 7 more oracle-compile failures hide in the fail column
(component_patch, datafix ×6 worlds, first_person, food, item_model,
options, resource_pack — javac errors in the harness logs), so the
true oracle count is **9** and the honest reading is 2 pass / 9 oracle /
6 real fail. The real fails: command_parse (7,546 mismatches),
entity_components (the harness-side list bug — `TypeError: list
indices must be integers`), hotbar (891/905), light (real mismatch),
save_roundtrip (game launch fails with "missing motion tag", and the
vanilla log shows the same bare-name-vs-map BlockState form divergence
26.2's save fix addressed), sound_event (missing sounds.json — the
sounds step). Published profile: 23 entries (17 + datafix#1..#6),
2 pass / 21 fail.

Verdict (recorded in pins/1.21.7/delta-verdict.json): **failed**.
D1 fail — the tag audit finds the same 37 dead production tags
locally predicted (5 named + 32 more); every other D1 sub-check
passed. D2 pass — cargo check 0 errors/0 warnings, protocol tests
432 with none failing. D3 pass — rank-1 3/3, rank-2 1/1. D4 blocked —
the profile stamp quirk (profile stamped d37f448, judge's staged
candidate HEAD 54a40370; patch 0006 is dead code in the CI judge, as
on 26.2); a local re-check against the recorded profile gives the
substance verdict: FAIL, datafix + save_roundtrip both fail. D5
blocked — no harness-subset.txt existed at dispatch; the E6 subset
is now in the pin, and a local re-check gives FAIL, 7 of 8 failing
(only block_properties passes). D6 blocked — no playtest.md; the
playtest is deferred by the owner, never claimed.

## Behaviour port (2026-11-19, Stream 3)

All 5 items closed, verified against the 1.21.7/1.21.8 jars (the
publish pipeline's named jars; 1.21.7 client/server sha1s match
research-lock). No Rust code changes were needed — the mechanical
publish had already done the pin work, and every other item verifies
as no-change:

- Item 1 (rank 1, regenerate `version.rs`): already regenerated by the
  mechanical publish (+5/−5). Values confirmed against the oracle
  `SharedConstants`: WORLD_VERSION 4438, protocol 772, resource pack
  64, data pack 81, version string 1.21.7.
- Item 2 (rank 1, save-format 4440 → 4438): server-jar class comparison
  (6,728 classes each side) shows exactly two changed classes —
  `DetectedVersion` and `SharedConstants` — and zero datafix classes,
  zero schemas, zero fixes. The data-version constant was already
  4438 in `version.rs`.
- Item 3 (rank 1, version facts): no stale 4440 anywhere in code (the
  one grep hit is a hex test string); the 1.21.8 references are
  comments and the per-version adapter modules, all correct for the
  shared protocol 772.
- Item 4 (rank 2, `SharedConstants`): `javap -p -constants` diff is
  exactly three constants — WORLD_VERSION 4440 → 4438, VERSION_STRING,
  and the snapshot protocol 259 → 258 (which no Rust code references;
  this tree is a stable release). Zero methods changed.
- Item 5 (rank 5, 6 unported classes): client-jar comparison (9,485 →
  9,484 classes) gives the exact six. `DetectedVersion` and
  `ReportEnvironment` differ only in the version-string/data-version
  literals. `GlProgram`, `VertexArrayCache` and `VertexFormat` carry
  real method-body changes, but all three are OpenGL/GPU code Horizon
  does not port (the renderer is wgpu; no Rust references exist).
  `GraphicsWorkarounds` was removed outright by Mojang. The inner
  classes (`GlProgram$1`, `VertexArrayCache$VertexArray`,
  `VertexFormat$Builder/$IndexType/$Mode`) are byte-different but
  bytecode-identical — recompile artifacts, no semantic change.

Manifest pin: Mojang republished the 1.21.7 version JSON after the
publish (live sha1 `5149e9aa…`, jars byte-identical — only the
asset-index-26 hash and one x86 JVM flag changed). Added to
`accepted_manifest_sha1` in research-lock.json (283bff6), the same
mechanism as the 26.2 fix; verified with the real rule
(`lab.manifest_accepted`: pinned accepted, live accepted, bogus
rejected). Without it CI lab-prepare fails on the unknown manifest.

Tag audit (D1 sub-check, local run 2026-11-19): the audit finds **37
dead production tags** (inventory 524 tags, 16 registries, from the
1.21.7 jar's own tag files; 272 literals scanned in 2,871 files; 6
dead test-only tags are deliberate fixtures). All 37 are tags from
1.21.8+ / 26.x that the inherited 26.3-base code references but
1.21.7's data does not define — including the 7 triaged on 26.2
(`brewing_potion_inputs`, `causes_suffocation`, `conduit_effect_block`,
`consumable_does_not_teleport_to`, `entities_can_teleport_to`,
`extendable_maps`, `shulker_does_not_teleport_to`) plus
`nether_portal_frame` and 29 more (copper golem/chest, sulfur cubes,
geysers, nautilus, shelves, and newer block/item tags). They are NOT
triaged here: they are inherited from the unported 1.21.11 behaviour
port, and the fixes belong at the 1.21.11 level, flowing down the chain
— triaging in this tree would duplicate Stream 2's work. D1 will fail
on CI until the triage lands; the per-tag locations are recorded in
the Stream 3 workdir (`dead-tags-1.21.7.json`).

## Launch fix: the 26.3-era block tags (2026-11-21)

The 1.21.7 client crashed on launch with "missing motion tag"
(`Assets::load` requires `minecraft:blocks_motion_in_heightmap`,
which 1.21.7's data does not define — the tag family was added in
26.3). The same crash had already failed the `save_roundtrip` and
`light` harnesses. Six dead tags blocked the launch path; they are
fixed here, following the 26.2 pattern exactly (26.2 triaged its own
launch-blocking tags in-tree the same way), rather than waiting for
the 1.21.11 triage — the remaining ~31 dead tags still belong to
Stream 2.

The rule, in every site: `BlockState.blocksMotion()` is solid
(`isSolid`, bit 0 of the exported behaviour flags), except cobweb and
bamboo saplings. The heightmap, fluid flow, suffocation, spawn
selection, feature placement, weather and ice melt all read that
per-state rule instead of the tags. Leaves are the blocks whose class
chain holds `LeavesBlock`.

- `horizon-client-render/src/assets.rs`: the three tag lookups
  (`blocks_motion_in_heightmap`, `..._no_leaves`, `blocks_fluid_flow`)
  are replaced by the computed rule from `block-behaviours.json`;
  `Assets.blocks_fluid_flow` changes type from `BTreeSet<String>` to
  `Arc<[bool]>` indexed by `BlockStateId.0` (call sites in
  `mesher.rs`, `motion/fluids.rs` and the mesher test updated to
  match).
- `horizon-items/src/blocks.rs`: new `BlockFacts::blocks_motion_names()`
  (per-name form of the rule, for the suffocation registry).
- `horizon-client/src/motion.rs`: `MotionResources.causes_suffocation`
  becomes `blocks_motion`, loaded from the facts; the glide-through
  set passed to `ClimbingRegistry::new` is empty (see below);
  `suppresses_bounce` is honey alone.
- `horizon-world/src/level/mod.rs`: new `Behaviours::blocks_motion()`
  (per-state form); `level/data.rs` gains the `cobweb` named block
  and drops the `blocks_fluid_flow` / `blocks_motion` tag names.
- `horizon-world/src/level/weather.rs`, `level/hooks.rs` (ice melt),
  `level/feature_world.rs` (feature-placement height), and
  `horizon-client/src/mobs/spawn_selection.rs` (spawn heightmaps)
  read `blocks_motion` / `legacy_solid` instead of the tags.
- `horizon-world/src/worldgen/generator.rs`: the heightmap facts are
  built from the loaded behaviours after the join, not from the tags
  (the tag form would have crashed world entry next).
- `horizon-client/src/game/world_entry.rs`: suffocation binds the
  computed blocks-motion set; `game/cushions.rs`
  (`state_suffocating`'s default arm) is `full_collision &&
  legacy_solid`.
- `SuffocationDefinition::TaggedFullCollision` becomes
  `MotionFullCollision` (both suffocation files, both fixtures, and
  the oracle's `tagged_full` key becomes `motion_full`).

The two tags that are NOT blocks-motion, verified against the
remapped 1.21.7 client jar on the playtest machine (`javap -c -p`):

- `can_glide_through`: vanilla 1.21.7's `LivingEntity.onClimbable`
  has no fall-flying exception at all (spectator → false, climbable
  tag → true, trapdoor-over-ladder → true, else false). The
  glide-through early return is removed from `mobs/living.rs` and
  its test now asserts the 1.21.7 rule (a gliding mob in vines is
  still on climbable). The `ClimbingRegistry` machinery keeps its
  parameter for versions that have the tag; 1.21.7 passes an empty
  set.
- `suppresses_bounce`: vanilla 1.21.7's `HoneyBlock.fallOn` never
  bounces — it plays the slide sound and calls `causeFallDamage`
  with a 0.2 multiplier. Honey alone is the set (the 26.2 tag's sole
  member); with no bounciness attribute in 1.21.7 the set is
  behaviourally a no-op either way.

Known remaining debt from this fix: the harness fixtures that index
the motion tags (`heightmap_fixture`, `terrain_fixture`,
`chunk_stage_fixture`, `feature_region_fixture`,
`bulk_section_fixture`, `material_stage_fixture`,
`decoration_storage`, `movement_registry_fixture`, the two
suffocation fixtures) still read them from the prepared tag bundle
and will panic on the missing tags when their harnesses run. 26.2's
fixtures read the per-state facts from the oracle instead
(`facts-server.json` entries 5/6, `motion.json`); carrying that over
needs the oracle + driver changes and is harness work, not launch
work — left for the 1.21.11 triage or a harness pass.

Second launch crash, same morning (08:50 NPT): the client got past
asset loading and died in `landing::load` with "unsupported landing
family". The 1.21.7 `landing-definitions.json` has two families the
inherited 26.3-base code does not know: `BedBlock` (16 beds) and
`FarmBlock` (farmland — 1.21.7 names the class `FarmBlock`, renamed
`FarmlandBlock` later). The bed fix is the 26.2 carry-over verbatim
(commit 576d4a93, which fixed the identical crash in 26.2): a
`FallDefinition::Bed` / `Rule::Bed` that halves the fall distance
through the entity-owned fall source, verified against the remapped
1.21.7 jar — `BedBlock.fallOn` is `super.fallOn(..., fallDistance *
0.5d)`, and `updateEntityMovementAfterFallOn` bounces unless the
entity suppresses it (the data's `bounce: true` for beds flows through
the existing flag). The farmland arm accepts both spellings.

Third launch crash, same morning (09:16 NPT): "attributes are not an
object" on the render thread right after the loading screen. The
environment loader (`environment.rs load_dimension_with`) requires
an "attributes" object in the dimension-type JSON, but 1.21.7
dimension types have none — the environment-attributes system is a
26.x addition (and 1.21.7 biome files have none either, which the
biome arm already tolerates with warn-and-skip). The dimension arm
now does the same: warn and continue with an empty map, leaving
every attribute at its default. 1.21.7's overworld.json was read to
confirm the field is absent (and "timelines" too, which the loader
already tolerates).

Fourth launch crash, same morning (09:34 NPT): "unknown carver
minecraft:cave" when the title panorama (and world creation) started
worldgen. The prepared worldgen bundle had six empty registries:
`carvers`, `features`, `rules`, `conditions`, `state_providers`,
`feature_tags` are all `{}`. Three different causes:

- `carvers` and `features`: 1.21.7 predates the registry renames —
  its jars carry `worldgen/configured_carver` and
  `worldgen/configured_feature`, while prepare read the 26.x paths
  `worldgen/carver` and `worldgen/feature` and found nothing in
  either jar (which prepare treats as "registry this version does
  not have", not an error). Prepare's WORLDGEN_BUNDLES now reads the
  1.21.7 paths.
- `rules`, `conditions`, `state_providers`: those registries do not
  exist in 1.21.7 at all — surface rules and state providers are
  inline in noise settings and feature configs. The decoders already
  accept inline objects (a string is a registry reference, an object
  decodes inline), so the empty bundles are correct for 1.21.7.
  Same for `feature_tags`: 1.21.7 has no feature tags.
- The 1.21.7 definitions themselves wrap their fields in a `config`
  object (`{"type": ..., "config": {...}}`); 26.x flattened the
  config into the definition. The carver and feature decoders unwrap
  it (`worldgen::unwrap_legacy_config`, applied in
  `WorldCarver::from_json` and `FeatureDefinition::decode`).

Two field gaps the unwrap alone does not close, both filled from
vanilla 1.21.7's own behaviour (playability defaults, not oracle
verified): the cave carver's config has no `count`, `thickness` or
room multiplier — count defaults to `VeryBiasedToBottom { 0, 39 }`,
the same three nested `nextInt` draws as vanilla's
`nextInt(nextInt(nextInt(40) + 1) + 1)`; thickness to uniform
1.0–7.0, vanilla's `1.0 + nextFloat() * 6.0` per tunnel; the room
multiplier reuses the config's vertical multiplier, which 26.x
split into its own provider. The canyon's `yScale` sits on the
config in 1.21.7 (3.0 for the overworld canyon) and inside the shape
in 26.x; the decoder falls back to the config-level value.

The laptop's prepared bundles were regenerated from the already
prepared data (configured_carver/configured_feature JSONs) with a
script rather than a full re-prepare, since prepare has no
worldgen-only subcommand and the full flow re-runs the Java
exporters.

Fifth launch crash, same morning (10:01 NPT): "invalid world
generation field: world carver type" — the carver catalog now loaded,
but 1.21.7's `nether_cave` is a third carver type
(`minecraft:nether_cave`) that 26.x folded into the cave carver.
Vanilla's NetherCaveCarver is a CaveCarver subclass with the same
tunnel generation, so both the catalog dispatch and the cave decoder
accept it as a cave (its config has the same fields).

Sixth launch crash, same morning (10:21 NPT): "missing block tag
minecraft:uncarvable". 1.21.7 has no uncarvable tag (26.3 added it);
its carvers declare replaceable sets, one per dimension. The
generator now derives the uncarvable set as every block outside both
replaceable tags — ores, bedrock and fluids included, as in vanilla,
where a carver only replaces what its set names. (26.2 solved the
same gap differently, by porting its carving pass to per-dimension
replaceable sets; that rework is out of scope here.)

Seventh launch crash, same morning (10:42 NPT): "block state
{"Name":"minecraft:stone"}: missing block identifier" when the
integrated server started generating. 1.21.7-era block states use
the `Name`/`Properties` form; 26.x renamed the fields to
`id`/`properties`, and the state codec only read the new names. The
codec now accepts both forms, which also covers state providers and
material rules (all call sites share the codec). The noise settings'
`default_block`/`default_fluid` were the first callers hit.

Eighth launch crash, same morning (11:02 NPT): "invalid world
generation field: base_octave". 1.21.7-era noise parameters use
`firstOctave` and a raw `amplitudes` list; 26.3 replaced them with
`base_octave` and friends, and the noise decoder only read the new
form. The decoder now accepts both: the legacy form goes through
`NormalNoise::create_parity`, the same constructor 26.2's decoder
uses (the 26.3-base tree still carries it, unused until now).

Ninth launch crash, same morning (11:28 NPT): "invalid world
generation field: left". 1.21.7-era density functions name their
children `argument` (unary/cache) and `argument1`/`argument2`
(binary); 26.x renamed them `input` and `left`/`right`. The decoder
accepts either spelling. Four more 1.21.7-era types needed arms:
`flat_cache`/`cache_2d`/`cache_once` (all value-identity, decoded as
the merged 26.x `cache`), `y_clamped_gradient` (the Y-only gradient
26.x generalised), `shifted_noise` (the noise function with shifts,
merged into `noise` in 26.x) and `end_islands` (renamed
`end_outer_islands` in 26.x). `weird_scaled_sampler` has no 26.x
equivalent: it decodes as clamp(input, 0, 1) * noise, an
approximation of vanilla's rarity gating — the exact type_1/type_2
mapper curves are not reproduced, so spaghetti caves land roughly
where vanilla puts them, not exactly.

Tenth launch crash, same morning (11:53 NPT): "invalid world
generation field: cell_size_xz". 1.21.7's `interpolated` takes no
cell sizes — it interpolates on the dimension's noise cell — while
26.x made them explicit fields. They now default to 4 wide, 8 tall,
the overworld's cell; only the overworld uses `interpolated` (noodle
caves), so no other dimension is affected.

Eleventh launch crash, same morning (12:29 NPT): "invalid world
generation field: noise". 1.21.7's `shift_a`/`shift_b` name their
noise through `argument` (a noise holder); 26.x renamed the field to
`noise`, so the decoder died on the overworld's shift_x/shift_z.
Carried 26.2's `shift_noise` helper over verbatim: it accepts either
spelling, resolving the argument string against the noise registry
(`minecraft:offset` is a noise, not a density function — there is no
density_function/offset.json).
