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

Twelfth launch crash, same morning (12:50 NPT): "invalid world
generation field: chunk_surface_level". 26.x added a
chunk_surface_level slot to the noise router; 1.21.7's router has
no such field. When it is absent, the router now synthesizes it
from the preliminary surface level, exactly as the settings
upgrader does — interpolated on a 16x1 cell.

End-to-end worldgen pass, same afternoon (13:00–14:20 NPT): the
owner asked whether the worldgen had been tested at all — the
decoders had only ever been unit-tested, never run against the real
1.21.7 data. A test was added that loads the real prepared data
(`GeneratorData::load` on the laptop's `.local` copy) and builds
the Nether, End and overworld pipelines, then generates chunks
(0,0) and (11,-7) and asserts the unsupported-features list is
empty for all three dimensions
(`prepared_1_21_7_overworld_builds_and_generates_a_chunk` in
`generator.rs`). It found twenty gaps in one pass; all are fixed
below, and the test itself is the coverage for the Pipeline-level
fixes — no separate unit tests were added for the individual
feature decoders, the empty unsupported list across all three
dimensions is the assertion.

Correction to the twelfth crash entry above: the 0f256e20 fix
cannot work on real data. 1.21.7's router has neither
`chunk_surface_level` nor `preliminary_surface_level` — 1.21.6
replaced the latter with `initial_density_without_jaggedness`.
The synthesis now follows vanilla's
`NoiseChunk.computePreliminarySurfaceLevel` (verified in the
remapped jar): scan down from the top in cell-height steps over
`initial_density_without_jaggedness` and return the first Y where
the density exceeds 0.390625, else `Integer.MAX_VALUE`. It is
synthesized as a `find_top_surface` over the raw initial density
minus 0.390625, bounded by the dimension's min Y and height with
a cell height of `size_vertical * 4`. Only the overworld uses
it (`AbovePreliminarySurface` is the only reader).

1.21.7 settings carry an inline `surface_rule` (or a holder
reference to one), not 26.3's `material_rule`; the pipeline now
reads `surface_rule` first and falls back to `material_rule`,
feeding it to `MaterialRuleLibrary` under the inline name
`minecraft:__settings_surface_rule`.

The spawn finder was carried from 26.2 verbatim: 1.21.7's
`spawn_target` is parameter-keyed (temperature, humidity,
continentalness, erosion, depth, weirdness, offset), not
density-function-keyed as 26.3 expects, so the old decoder died
with "unknown material density". The pipeline stores the decoded
parameter points and the finder samples them directly.

The feature catalog's `expand_feature` read nested fields from
the top-level object, but 1.21.7 definitions wrap them in
`config`; it now unwraps the legacy config first.

State providers: 26.3 shortened the provider type names, but
1.21.7 data uses the long forms — `simple_state_provider` (199
uses), `weighted_state_provider` (41), `rule_based_state_provider`,
`rotated_block_provider`, `randomized_int_state_provider`, and the
three noise providers — so all are accepted as aliases, in both
the typed decoder and the noise-provider decoder (the latter was
behind the flower-plain/meadow/flower-forest failures). Typeless
`{fallback, rules}` objects (disk configs) now decode as
rule-based when `type` is absent but `rules` is present.

Spring fluids: `FluidState::from_json` now accepts `Name` as well
as `id` and `Properties` as well as `properties` (26.2's fix,
carried verbatim) — spring_water, spring_lava and
spring_lava_frozen were failing with "missing fluid identifier".

Dripstone: `SpeleothemClusterFeature` now accepts
`minecraft:dripstone_cluster` as well as
`minecraft:speleothem_cluster`; 1.21.7's cluster has no
`base_block`/`pointed_block`/`replaceable_blocks`, so it defaults
to dripstone_block/pointed_dripstone and the
`#minecraft:dripstone_replaceable_blocks` tag, and accepts the
legacy field names (`dripstone_block_layer_thickness`,
`chance_of_dripstone_column_at_max_distance_from_center`,
`max_distance_from_edge_affecting_chance_of_dripstone_column`).
The same replaceable default was added to large dripstone, and
the `column_radius` bounds widened from 1..16 to 1..32 (1.21.7
data runs 3..=19).

Aquatic, nether and desert-well features were carried from 26.2
(verified field-by-field against the 1.21.7 configs): kelp,
seagrass and sea pickle; basalt columns/pillars, glowstone blobs,
nether forest vegetation, twisting and weeping vines; and the
desert well, which needed 26.2's seeded loot-table trait method
carried too (`set_block_entity_loot_table_seeded`, seed from the
block position, for suspicious sand/gravel).

Random patch / flower: verified in the jar that 1.21.7's FLOWER,
NO_BONEMEAL_FLOWER and RANDOM_PATCH are all `RandomPatchFeature`
with `RandomPatchConfiguration`. A new decoder places the nested
placed feature `tries` times with the vanilla
`nextInt(spread+1) - nextInt(spread+1)` offset per axis, and the
catalog gained the missing flower/random_patch expansion arm.

Ice spike: ported from the jar (`IceSpikeFeature.place`) — sink
while air above minY+2, require snow_block below, height
nextInt(4)+7 with a 1/60 tall-spike chance, tapered packed-ice
cone with the 0.75 edge skip, mirrored root below, base columns
to y=50. Replaceable is air plus `#minecraft:dirt`, snow_block
and ice (`Feature.isDirt` is the dirt tag, `Feature.isStone` the
base-stone-overworld tag, both verified in the jar).

Forest rock: 1.21.7's FOREST_ROCK is a `BlockBlobFeature` with a
bare `state` config (verified in the jar); the decoder builds the
support predicate as dirt-or-stone, which vanilla hardcodes.

Pointed dripstone: ported from the jar
(`PointedDripstoneFeature.place`, `createPatchOfDripstoneBlocks`,
`DripstoneUtils.growPointedDripstone`) — tip direction from the
dripstone base above/below (both present: a coin flip), patch at
the base position with the horizontal-plane order and the
directional/radius chances, height 1 or 2. The config carries no
blocks, so dripstone blocks and the replaceable tag are
hardcoded.

The router now carries 15 fields, not 8: 1.21.7 routers (all
three dimensions verified) add `barrier`,
`fluid_level_floodedness`, `fluid_level_spread`, `lava`,
`vein_toggle`, `vein_ridged` and `vein_gap`. The six climate
fields stay first so the climate compiler's `functions[..6]`
still works.

The synthesize module was carried from 26.2 (minus
`peaks_and_valleys`, which 1.21.7 never uses): `y_function`, the
aquifer exclusion (`min(-0.225 - erosion, max(depth - 0.9, 0))`),
and copper/iron ore-vein roots. The aquifer is now built from
the router when `aquifers_enabled` (overworld only), with the
surface sampler being the synthesized `find_top_surface` — the
`PreliminarySurfaceLevel` sampler does not exist in 1.21.7.
Veins are synthesized when `ore_veins_enabled` (overworld only)
and run BEFORE the surface rule as a `MaterialRule::Sequence`,
or the deepslate gradient shadows them; the carver's top
material uses the vein-free surface rule, matching 26.2's
`CarvingContext.topMaterial` semantics.

Trees: 1.21.7 names the below-trunk provider `dirt_provider`
(39/39 simple providers in the data); 26.x renamed it
`below_trunk_provider`. Either spelling is accepted. Vanilla
skips the set when the block is already dirt and `force_dirt`
is false, but every 1.21.7 dirt provider is simple, so the
unconditional set draws nothing extra.

Placement: `random_offset` (`xz_spread`/`y_spread`) was carried
from 26.2 verbatim — it covers disk_grass, ice_patch, the lush
caves features, cave vines and spore blossom.

Lakes: 1.21.7 lake configs carry only `fluid` and `barrier`.
Verified in the jar (`LakeFeature.place`, `canReplaceBlock`):
the can-place check defaults to always-true (vanilla has no
can-place check in the edge loop), air/fluid replacement
defaults to NOT `#minecraft:features_cannot_replace`, and
barrier replacement to NOT
`#minecraft:lava_pool_stone_cannot_replace`.

Huge mushrooms: 1.21.7 configs have no `can_place_on`. Verified
in the jar (`AbstractHugeMushroomFeature.isValidPosition`): the
default is dirt or `#minecraft:mushroom_grow_block`.

Coral: 1.21.7's coral features carry empty configs and are a
different shape from 26.3's. Ported from the jar
(`CoralFeature`, `CoralTreeFeature`, `CoralClawFeature`,
`CoralMushroomFeature`): the base picks a random block from
`#minecraft:coral_blocks` (one bounded draw over the tag in tag
order), requires water-or-coral at the position and water above,
then 0.25 coral fan above (else 0.05 sea pickle, 1..4 pickles)
and 0.2 wall coral per horizontal side. Tree (trunk 1..3 plus
2..4 shuffled branches), claw (base plus 2..3 of claw/clockwise/
counter-clockwise, shuffled) and mushroom (3..5 box per axis,
face cells only, 0.1 skip) shapes are all ported. The decoder
dispatches on the presence of a nested `feature`: 26.3-form
goes to the existing `CoralFeature`, empty-config 1.21.7-form
to the new legacy decoder.

Root systems: 1.21.7's `RootSystemConfiguration` has no
`level_test_distance`/`max_level_deviation` (26.x additions);
both are now optional and default to 0, which disables the
level test.

Known approximation carried from the ninth crash entry:
`weird_scaled_sampler` remains clamp(input, 0, 1) * noise, not
vanilla parity — spaghetti caves land roughly where vanilla
puts them, not exactly. SUPERSEDED 2026-11-21 (see the
weird_scaled_sampler entry at the end of this file): the
approximation turned out to void the whole overworld, and the
sampler is now implemented at vanilla parity.

Full-suite note, same afternoon: the horizon-world lib suite
stands at 428 passed, 23 failed, 3 ignored with this pass
applied. All 23 failures are pre-existing, verified by running
the same suite at the parent commit (0f256e20) in a worktree
with the same `.local` data: the baseline fails the same 20
level tests plus 4 more growth tests (a_tree_with_bees,
azalea_and_mangrove, oak_sapling_grows_with_bone_meal,
random_ticks_advance_sapling_stage_then_grow) that this pass
fixes via the tree `dirt_provider` change — 24 level failures
at baseline, 20 with the pass. The 20 are level-simulation
tests (water spread, redstone, pistons, doors/beds, crops,
chests, dripstone cauldrons, bone-meal grass, saplings, huge
mushrooms), the other 3 are datafix tests; none touch worldgen
decoding and none are in scope for this pass. Beware comparing
against a run without `.local`: the level tests silently
`return` (pass) when the fixture is missing, so a bare worktree
reports 445 passed / 3 failed and looks healthier than it is.

Movement attributes: vanilla 1.21.7's attribute registry has no
`air_drag_modifier`, `bounciness` or `friction_modifier` (26.x
additions), but the movement code reads all three for every living
entity — the first world creation crashed on the render thread with
"missing movement attribute minecraft:air_drag_modifier" right after
the player logged in. `Catalog::from_json` now synthesizes the three
when the catalog lacks them, with the 26.2-verified registry values
(air drag 1.0 in [0, 2048], bounciness 0.0 in [0, 1], friction 1.0
in [0, 2048], all syncable), and gives every entity supplier the
registry default as its base — all 93 of 26.2's suppliers list them,
with player bases exactly these defaults. Catalogs that already list
an attribute are untouched, and a supplier that omits a listed
attribute stays omitted. Two unit tests pin the synthesis and the
no-duplicate scoping.

End main island (worldgen-verify): vanilla 1.21.7's
`minecraft:end_islands` includes the central island cone in the
function itself, but the end-to-end pass had aliased both
`end_islands` and `end_outer_islands` to the 26.3 outer-only
sampler, so the End generated as void (0 non-air blocks in
chunk (0, 0)). Carried 26.2's fix: `GraphNode::EndIslands
{ central }`, with `end_islands` decoding to central: true and
`end_outer_islands` to central: false; the cone formula was
verified against the 1.21.11 named jar's
EndIslandDensityFunction bytecode. The 1.21.7 end-to-end test is
now the shared `prepared_world_builds_and_generates_chunks`
(two chunks per dimension plus a JSON dump, run by the
worldgen-verify CI job); it caught this gap.

Overworld void terrain: the `weird_scaled_sampler` approximation
(2026-11-21). The owner generated three new worlds and reported
them "too weird": no sky, spawning underground. The CI
worldgen-verify dump (seed 12345) showed chunk (11, -7) — grove,
a mountain biome — as 95.5% air with its surface at y ~ -48,
while the same chunk/seed in 26.2 is a mountain surfacing at
y ~ 133. Probing the density chain at the grove column showed
the climate inputs correct (continents 0.116, erosion -0.666,
ridges 0.877) and `sloped_cheese` crossing zero at y ~ 130, but
the final density stuck at ~ -0.02 at every y above -48. The
culprit was the approximation: vanilla's real formula is
`d * |noise(x/d, y/d, z/d)|` with `d` the rarity-mapped input
(always >= 0), while the stand-in was `clamp(input, 0, 1) *
noise(raw coords)` — wrong mapper curves, unscaled coordinates,
and no absolute value, so whenever the rarity noise went
negative the cave branch collapsed to a constant ~ -0.068 and
the final density went negative everywhere. The sampler is now
implemented at vanilla parity as `WeirdScaledSampler` in
horizon-core (graph node tag 23, compiler arm, decoder arm),
with both mapper curves verified against the 1.21.11 named
jar's `DensityFunctions$WeirdScaledSampler` bytecode: type_1
steps 0.75/1.0/1.5/2.0 at -0.5/0.0/0.5, type_2 steps
0.5/0.75/1.0/2.0/3.0 at -0.75/-0.5/0.5/0.75, NaN falling
through to the last step as in Java. After the fix the probe
reads surface heights 132-135 across the chunk and the
worldgen dump shows 24,706 stone + 11,874 deepslate with a
heightmap mean of 132.7 (26.2 control: 133.2). The aquifer
gap in the same report — the mountain chunk got no water while
the ocean chunk did — was a symptom, not a second bug: the
aquifer keys off the synthesized chunk surface level, which
was correct (~133), so it saw no below-sea-level surface to
fill; with the density fixed the ocean chunk's water fills
from the floor with no air gap. Spawn and sky needed no code:
`level_respawn_pos` already places the player on the top
full-topped block, and the overworld dimension data carries
`has_skylight: true` with the shared sky path — both symptoms
were the missing terrain. Regression test
`grove_chunk_surfaces_as_mountain` pins the four grove
columns to 90..=160.

## Too cavey: the cave carver's 26.x defaults (2026-11-21)

The owner playtested the terrain-fixed build and reported the terrain
still too mountainy and too cavey across three worlds (Test 1, 2, 3).
Measured against the vanilla 1.21.7 server at Test 1's seed
(-8343112210435233944) over 1,018 paired chunks (260,608 columns),
the terrain height was NOT wrong — surface delta mean -0.87, ground
delta mean -0.29, median 0.0 both, biome agreement 1.000 — but the
caves were: +8.4 cave-air blocks per column (horizon 21.7 vs vanilla
9.4 over 12 sampled chunks), spread across every depth band below
y=96. (An earlier reading of +7.4/+9.1 too-high terrain was a
measurement artifact: horizon writes some palette entries as
`{'': name}` and `{'id': name, 'properties': ...}`, which the first
parser read as empty names and classified as solid ground — air
above the surface counted as rock.)

The density graph was exonerated first: a Java probe driving
vanilla 1.21.11's own density codec against the Rust decoder
agreed on all 1,607 aligned queries (max diff 0.000024), the
final-density zero crossings reproduced vanilla's ground surface
at every sampled column, and the fill's bulk `sample_volume`
matched per-point evaluation exactly. The density alone predicts
7.8 cave-air blocks per column; vanilla's save has 9.4 (carvers
add +1.6); the horizon save had 21.7 (carvers added +13.9).

Root cause: `CaveCarver::from_json` decoded the fields 1.21.7's
carver configs lack with 26.x-era defaults that do not match
vanilla 1.21.7's hardcoded `CaveWorldCarver` (verified against
the 1.21.11 named jar's bytecode):

- tunnel count bound 40 instead of 15 (`getCaveBound`; 10 for
  nether caves) — ~2.8x too many tunnel starts;
- tunnel thickness uniform 1..7 instead of `getThickness`:
  `nextFloat() * 2 + nextFloat()` (mean 1.5, range 0..3) —
  tunnels ~2.7x too thick. The old default was the ROOM
  thickness formula (`1 + nextFloat() * 6`) applied to tunnels;
- the 1-in-10 thickness bias multiplier defaulted off, while
  vanilla always applies it;
- room height scaled by the vertical multiplier (0.8..1.3)
  instead of the config's yScale (0.1..0.9) — rooms ~2x too
  tall. The tunnel start's vertical scale also now defaults to
  `getYScale`: 1.0 for caves, 5.0 for nether caves.

The fix (carver.rs) defaults count to
`VeryBiasedToBottom { 0, 14 }` (nether `{ 0, 9 }`), thickness
to `Trapezoid { 0, 3, 1 }` (samples `2*f1 + f2`, the same two
floats in the same order as vanilla), the bias on, rooms on
yScale, and nether start-vertical on 5.0. Configs that carry
the fields (26.x) still use them. After the fix, generating
the same 12 chunks gives 9.54 cave-air blocks per column
against vanilla's 9.40 (delta +0.14, median 0.0; every chunk
within +0.52, three exact). Regression test
`cave_carving_matches_vanilla_at_test1_seed` pins chunks
(-13, -1) and (3, 14) to vanilla's measured means (4.50 and
40.97, tolerance 2.0); the old defaults gave 32.7 and 55.8.
The "too mountainy" half of the report is not a generator
defect: the terrain matches vanilla, and the spawn area is
genuinely mountainous in vanilla too (grove/snowy slopes).

Black sky: legacy environment attributes synthesized (2026-11-21).
After the terrain fix the owner reported the sky all black in
daytime (Level.dat Time=2002). The cause: 1.21.7's dimension types
and biomes carry no `attributes` key — the environment-attribute
system arrived in 1.21.11 (Polytone documents its
`attributes_modifiers` as "1.21.11+ only"; 1.21.11's named jar
already drives its SkyRenderer from `EnvironmentAttributes`, while
1.21.7 still colours its sky from the biome's legacy `effects`) —
so crash #3's fallback (warn + empty map) left every visual
attribute at its default: sky and fog black, sun/moon/star angles
0, star brightness 0, and no clouds at all (the cloud colour's
alpha gates them). The run log confirmed both sites: the dimension
map was empty and every biome was dropped with "attributes are not
an object". The fix synthesizes the missing data in
`crates/horizon-client-render/src/environment.rs` (`mod legacy`),
the same pattern as the missing movement attributes (83d24e28):
each biome's map comes from its legacy `effects` (sky_color,
fog_color, water_fog_color — plains 0x78A7FF / 0xC0D8FF), each
dimension gets constants from its legacy fields (overworld: plains
sky/fog plus its own cloud_height 192; Nether: wastes fog 0x330808
and fixed-midnight sky light 4; End: fog 0xA080A0, black sky with
stars at vanilla's maximum 0.5), and the overworld gets a
synthesized day timeline (period 24000) whose keyframe tracks are
vanilla's classic formulas — linear sun/moon/star angles (0° at
noon), star brightness from `Level.getStarBrightness`, sky light
from `15 - Level.getSkyDarken`, sky and fog colours multiplied by
`ClientLevel.getSkyColor`'s daylight factor, sky light factor as
the level's fraction of 15, sky light colour to the engine's night
colour, cloud colour white by day and black at night. The classic
formulas couldn't be jar-verified (the 1.21.11 and 26.2 jars both
already use attributes); they follow the Minecraft Wiki's Daylight
cycle page and the engine's own probe-verified angle convention.
One gap is deliberate: the sunrise/sunset colour is not
synthesized (its exact vanilla constants couldn't be verified, and
a wrong glow would look like a bug), so dawn and dusk have no
sunset fan until the 26.x timeline values are available.
Regression tests: `legacy_overworld_sky_follows_the_day` and
`legacy_nether_and_end_get_sane_values` (hermetic fixtures),
`prepared_1_21_7_overworld_sky_is_not_black` (real prepared data,
skips without it).

Commands: `/time set day` and the argument-type registry
(2026-11-21). The owner couldn't set the time or run other
commands. Two 1.21.7-only gaps were found and fixed. First, the
Commands packet's argument-type list was still 26.3's (62
entries): the five 26.3-only types (`context_float_provider`,
`context_int_provider`, `slot_source`, `feature`,
`swing_animation`) shifted `dialog` (55) and `uuid` (56) onto the
wrong ids, so their tree nodes decoded with the wrong parsers.
`ARGUMENT_TYPES` in
`crates/horizon-protocol/src/packets/play/commands.rs` is now
1.21.7's registry (57 entries, 26.2's list — the adapter's 1.21.7
remap is the identity and the 26.2 jar's
`ArgumentTypeInfos.bootstrap` order confirms it), and the five
26.3-only arms are gone from `horizon-command/src/network.rs`.
That alone didn't explain `/time set`: the send path, the
integrated-server dispatch and the permissions (owner op 4,
allowCommands=1) are all intact and identical to 26.2. The real
`/time` gap was the tree's shape: 1.21.7's TimeCommand takes
literal times (`set day|noon|night|midnight`, verified in the
1.21.11 named jar's TimeCommand bytecode), but the ported tree
only had the 26.x forms — a numeric time and a `timemarker`
argument — and 1.21.7 has no time markers, so every named time
failed both ways. `clock_nodes` in
`horizon-command/src/vanilla.rs` now registers the four literals
(they execute `time.set`; the executor resolves them to
1000/6000/13000/18000 in `game/commands.rs`), plus 1.21.7's
`query daytime` and `query day` forms, which were missing too
(`query time` is the 26.x name). The 26.3-only already-at-time
and already-at-marker errors are removed from `cmd_time`,
mirroring 26.2. No systemic break was found for non-time
commands: the full pipeline was traced in code and matches 26.2
(`remote_chat.rs`, `lan_chat.rs`, `lan/commands.rs` are
byte-identical); if other commands still fail in-game, the exact
commands and what the owner sees (error text vs nothing) are
needed, plus a check that the laptop's prepared data has
`commands.json` (without it the server sends an empty tree — no
suggestions, though execution still works). Regression tests:
`argument_types_are_1_21_7s_registry` (protocol) and the updated
`parses_vanilla_commands` / suggestion tests (horizon-command).

No mobs spawn at all (2026-11-21). The owner reported no mobs in
any of three fresh worlds — only terrain. Two independent 26.x-only
assumptions, each enough on its own to leave a world empty.

First, every biome's spawn list was empty. `BiomeInfo::from_json`
(`crates/horizon-client-render/src/biome.rs`) read spawns only from
the `minecraft:gameplay/natural_mob_spawns` environment attribute,
but 1.21.7 biomes have no `attributes` key at all (the attribute
system arrived in 1.21.11) — their spawns live in the legacy
top-level `spawners` field (plains: 6 creature + 8 monster entries).
So `NaturalSpawner` found no potential spawns anywhere, and
`spawnMobsForChunkGeneration` returned early on zero weight. The
26.2 tree carries the legacy fallback for exactly this and the
1.21.7 port was missing it; it is ported here (the attribute wins
when present): `spawners` when the attribute list is empty,
`spawn_costs` likewise, `creature_spawn_probability` before the
attribute, default 0.1. `parse_entries` also gains the legacy
`minCount`/`maxCount` count form (a draw even when min == max, so
never constant), which the 26.3 `count` forms alone didn't cover —
the same fix covers structure `spawn_overrides`, which use the same
entry shape.

Second, mob data failed to load at all. `StructureMobFinalizer::load`
(`crates/horizon-client/src/vanillagen/structure_mobs.rs`) required
`minecraft:camel_husk` in `entity-types.json`, a 26.x-only entity
(1.21.7's registry has 151 entries, no camel_husk; 26.2's has 158,
with it), so `load()` returned None and the run log read "no mob
data, structure mobs are finalized when loaded" — chunk-generation
mobs (`spawn_generated`) and structure mobs (`finalize`) both dead.
The husk reuses the camel's model (`camel#main`), so the camel's own
size now stands in when the husk type is absent; the jockey can never
materialize here anyway, since creating a `minecraft:camel_husk`
finds no such type in the registry.

Ruled out along the way: difficulty is `normal` in all three worlds
(level.dat), the `spawn_mobs` game rule defaults true, and the
natural-spawn tick itself is identical to 26.2. One wart remains: a
husk that rolls the 26.x jockey attempt still gets an iron spear in
hand (a 26.x item 1.21.7 lacks) before the camel create fails — a
cosmetic oddity for desert husks, not a spawn blocker.
Regression tests: `legacy_spawners_fill_spawns_without_attributes`
and `attribute_spawns_win_over_legacy_spawners` (hermetic biome
fixtures), `mob_data_loads_without_camel_husk` (minimal fixture);
the existing bastion structure-mob tests pass against real 1.21.7
prepared data — the chunk test matches the 26.3 vanilla list exactly.
The ocean ruin test's expectation is adapted for 1.21.7: there is no
nautilus here (a 26.x entity), so the trident drowned stays unmounted
and the draws the nautilus would have taken shift the two mobs
finalized after it; the first drowned and the trident bonus still
match 26.3 exactly. Incidental: the horizon-client-lib test suite
didn't compile on main (`arrows_client_tests.rs` still passed a
`BTreeSet` where `FluidResources::from_registry` now takes a
`&[bool]`); the call now passes `&[]`, same meaning (no state blocks
fluid flow), and the arrow tests pass.

Deep caves missing at y -64..-32: the beardifier was never added to
final_density (2026-11-21). The port generated only ~55% of vanilla's
deep-band cave air (1.89 vs 3.41-3.45 blocks/column at the Test 1 seed
8257987075890829262, confirmed against both the owner's Prism world and
a vanilla server run), while every other Y band matched. Density,
carver and aquifer were all exonerated by probes first: a 9,060-point
differential density probe agreed with vanilla's codec exactly, and the
deficit turned out to be spatially concentrated in the 244 structure
chunks (99.2% of it; the 1,346 clean chunks matched at -0.015). The
mechanism: the ancient-city, trial-chamber and village templates
contain no air blocks at all — vanilla clears the halls with the
Beardifier, which NoiseChunk adds to final_density in code
(`add(finalDensity, BeardifierMarker)`, the marker swapped for the
chunk's real beardifier). Horizon built the beardifier and registered
it as a world field, but the density graph has no beardifier node (nor
does vanilla's data), so it was never sampled and the halls stayed
solid rock. `NoiseRouter::compile` in
`crates/horizon-world/src/worldgen/router.rs` now wraps FinalDensity in
`final_density + ContextBoundSampler(Beardifier)`, mirroring vanilla's
NoiseChunk; where no beardifier is bound the sampler falls back to 0,
so unaffected chunks sample exactly as before. Verified: chunk
(2,-15) post-decorate deep air 18.16 vs vanilla 18.08 (was 0.04), city
blocks 214 = 214; over all 244 structure chunks the terrain-level deep
band matches vanilla within -0.03 on average (was -9.83), and the
1590-chunk paired mean is ~+0.04 (was -1.52). Other bands unchanged
(0..32: -0.02, 32..64: +0.01). Regression test:
`beardifier_shapes_deep_band_at_test1_seed` (generator.rs) pins the
deep band at (2,-15) and (13,-14) against vanilla-derived values.
Known follow-up, not fixed here: trial chambers and mineshaft-adjacent
chunks over-fill slightly (trial terrain delta -2.60, mine -1.95 across
63 chunks; worst single chunk (12,-14) at -6.4 post-decorate) — the
beardifier math, piece/junction selection and jigsaw assembly were all
verified against vanilla 1.21.7 bytecode and the placed blocks match
exactly, so the residual is a second-order effect (interpolation order
or a junction subtlety) needing a Java-oracle comparison to isolate.
RESOLVED 2026-11-22 — see the next entry: the comparison showed the
residual was a stage-mismatch artifact, and the beardifier matches
vanilla exactly; no code change was needed.

Beardifier "residual" at trial chambers: a measurement artifact, not a
bug (2026-11-22). The follow-up above (worst chunk (12,-14) at -6.4
post-decorate) compared the port's TERRAIN-level output (generate_proto,
before placement) against vanilla's POST-decorate world. That comparison
is only valid where the templates contain no air — true for ancient city
(0 air blocks in the city_center/city templates), false for trial
chambers (atrium_1: 4174 air of 12180 volume; end_1: 3076 of 7220) and
villages (armorer_house_1: 337 air). Placement carves the trial-chamber
halls; the beardifier's encapsulate term only buries the chamber, so the
"terrain-level" figure (port 1.79 vs a derived 17.16 at (12,-14)) was
never a real divergence. Post-decorate vs post-decorate at the Test 1
seed, the port matches vanilla: (12,-14) 8.10 vs 8.18, (13,-14) 1.02 vs
0.94, (2,-15) 18.16 vs 18.08, (11,-14) 7.26 vs 6.85, (12,-15) 2.62 vs
2.62 — no systematic over-fill. The beardifier itself was exonerated
three ways: its math matches the vanilla named jar's Beardifier bytecode
exactly (compute, getBuryContribution, getBeardContribution,
forStructuresInChunk, kernel table), piece/junction selection matches the
vanilla NBT exactly (141 near pieces + 249 junctions at (12,-14),
identical bounding boxes), and a block-level diff of the deep band shows
zero port-solid/vanilla-air cells outside piece boxes. The small
remaining post-decorate diffs are placed-feature parity, not density:
sculk_vein placement differs (vanilla 61/251/22 cells at
(12,-14)/(11,-14)/(2,-15); the port's sculk_patch places different
cells), tuff-blob positions differ (639 cells at (13,-14)), and (13,-14)
has a 21-cell aquifer/fluid divergence (vanilla water + obsidian where
the port has air/fluid). No code change: the fix is the test.
Regression test: `postdecorate_deep_band_matches_vanilla_at_trial_chambers`
(generator.rs) pins structures-only post-decorate deep-band air at
(12,-14), (11,-14), (12,-15) and (2,-15) to vanilla-derived exact
counts (vanilla post-decorate air + sculk_vein cells, which
structures-only leaves as air). The old "terrain-level" derivation
(post-decorate air + structure blocks) is valid only for ancient city;
do not reuse it for trial chambers or villages. One adjacent finding,
not acted on: the port computes nonzero ground_level_delta on trial
pieces where the vanilla NBT has GD=0 — irrelevant for encapsulate
(GD is unused there in both vanilla and the port) but it would matter
for BEARD_THIN/BEARD_BOX/BURY pieces, so it may be a separate bug.

Water never spreads: `canHoldAnyFluid` read a tag 1.21.7 does not have
(2026-11-22). The owner reported water placed on top of a block at height
just sitting there — no falling, no spreading. Root cause:
`FlowingFluid.canHoldAnyFluid` in `crates/horizon-world/src/level/fluids.rs`
was the 26.3 form, `is_liquid_container(state) ||
in_set(state, #washed_away_by_fluids)`, but the `#washed_away_by_fluids`
tag is a 26.3 addition: 1.21.7's prepared block tags (194 tags) do not
contain it, so `Tags::resolve` gave it an empty set and
`can_hold_any_fluid` was false for every non-container block — water
could not spread down OR sideways, and a placed source never moved.
Vanilla 1.21.7's `FlowingFluid.canHoldAnyFluid` (verified against the
named 1.21.7 server jar's bytecode) is the list form: a liquid
container, or a state that does not block motion and is none of door,
signs, ladder, sugar cane, bubble column, nether/end portal, end
gateway, structure void — exactly the form the 26.2 tree already
carries. The fix restores that form; `blocks_motion` + the class/block
exclusions match the bytecode instruction for instruction (the port's
`Class::Sign` check covers the same blocks as vanilla's `#signs` tag).
The same 26.x-tag pattern also affects `LavaFluid.randomTick` in the
same file (it reads the `#blocks_lava_fire_spread` tag, also absent
from 1.21.7, so the fire walk never stops at a motion-blocking state;
vanilla 1.21.7 uses `blocksMotion()` there) — deliberately NOT fixed
here; it is parity-sweep defect G3, a separate follow-up. Verified: the
four water tests that failed on main (`water_spreads_on_a_floor`,
`water_falls_and_makes_sources`,
`water_keeps_spreading_over_waterlogged_scaffolding`,
`refused_spread_keeps_water_out_of_a_guarded_block`) now pass.
Regression test: `water_on_a_pillar_flows_off_the_edge_and_falls`
(level/tests.rs) places a source on a one-block pillar and pins the
falling column beside it (level 8 at the pillar's side down to the
floor) and the spread on the floor. Known carry-over, not fixed here:
the same tag-form `can_hold_any_fluid` is in 1.21.5, 1.21.6, 1.21.8,
1.21.9 and 1.21.11 — all 1.21.x versions lack the tag, so their water
is broken the same way.

Trees missing outside groves: the 1.21.11 `supports_*` tags don't
exist in 1.21.7 data (2026-11-22). The owner reported the spawn area
bare in the port while vanilla at the same seed
(8257987075890829262) spawns ringed by birch forest. Paired
comparison of 1,498 chunks (port's fresh "New World" vs his vanilla
world, same seed): port 978 trees vs vanilla 5,363 (ratio 0.18);
birch 0 vs 2,108, oak 42 vs 1,760, spruce 934 vs 1,493. By biome,
grove matched (3.94 vs 4.02 trees/chunk) and frozen_peaks matched,
but taiga was 0.00 vs 4.59, birch_forest 0.03 vs 5.21, forest 0.10
vs 4.95. Spawn 7x7 region: port 0 trees, vanilla 130 (all birch).
The trees the port did place were in the right positions (90.8% of
its trunk columns coincide with vanilla's), so this was feature
selection, not RNG. Root cause: the Java-exported survival rules
name `TagBelow:minecraft:supports_vegetation` for every sapling, but
that tag (added in 1.21.11, when Mojang replaced the hardcoded
`mayPlaceOn` checks with tags) doesn't exist in 1.21.7 — the lookup
resolves to an empty set, so `would_survive` fails and every
`_checked` / `_bees_*` tree child is filtered at placement. Grove
survives because `spruce_on_snow` / `pine_on_snow` filter on
matching snow blocks instead. The same gap also killed mangrove
trees, cacti and sugar cane (`would_survive` states in the feature
bundles), while flowers don't use `would_survive` and were
unaffected. Fix: `GeneratorData::load` now synthesizes the missing
tags from their 1.21.7 equivalents, verified against the 1.21.7
jar's bytecode — `supports_vegetation` = dirt + farmland
(`VegetationBlock.mayPlaceOn`), `supports_azalea` adds clay,
`supports_mangrove_propagule` adds clay (hanging: mangrove leaves),
`supports_dry_vegetation` = `dry_vegetation_may_place_on`,
`supports_nether_sprouts` = nylium + soul soil + dirt + farmland,
`supports_nether_wart` = soul sand, `supports_wither_rose` adds
netherrack, soul sand, soul soil, `supports_cactus` = sand,
`supports_sugar_cane` = dirt + sand with water / frosted ice beside
(block + fluid tags). Tags the bundle already has keep their own
members. Regression tests in generator.rs:
`legacy_support_tags_synthesize_from_equivalents` (hermetic, pins
the synthesis) and `birch_trees_decorate_at_test1_seed` (decorates
chunk (-2,-2) at the owner's seed; vanilla has 40 birch logs / 271
birch leaves there, the test requires >= 20 logs and >= 100 leaves).
Known follow-up, not fixed here: the other 1.21.11-era survival tags
(`supports_crops`, `supports_cocoa`, `supports_lily_pad`,
`supports_frogspawn`, `supports_chorus_*`,
`supports_small_dripleaf`, `supports_big_dripleaf`,
`cannot_support_*`, `enables_bubble_column_*`, and the fluid
`supports_lily_pad` / `supports_frogspawn`)
are also absent from 1.21.7 data, but no worldgen feature uses
`would_survive` with those states, so they are dormant in worldgen;
gameplay survival uses the separate class-based implementation in
`level/survive.rs`. Same defect class as the carver-registry and
motion-tag gaps: other pre-1.21.11 trees (1.21.5/1.21.6/1.21.8/1.21.9,
1.20.x) need the same synthesis carried over.

Undead daylight burning: no defect in the burn path, but the burn list
over-burned (2026-11-22). The laptop playtest reported a skeleton in
direct sunlight not catching fire. The burn path itself checks out end
to end against the vanilla 1.21.7 jar and the live save: `burn_undead`
/ `sun_burn_tick` (mobs/ai.rs) match `Mob.isSunBurnTick` exactly — the
`monsters_burn` day gate, brightness over 0.5, the `(br - 0.4) * 2 / 30`
per-tick chance, the wet and sky-exposure gates, 160 ticks (8 s) of
fire, and the helmet exemption (which wears the helmet instead). The
"New World" save proves it in the live game: 14 surface undead were
burning (7 skeletons, 6 zombies, 1 zombie villager; Fire 56-157,
health 14-16 from fire damage), and none of the 24 unburned
skeletons/zombies was sky-exposed (sky light 0 at the eye for all of
them — all underground). The client side was verified link by link too:
the server sends the on-fire shared flag in the mob metadata, the
tracker diffs and sends it on change, the client's `apply` sets
`fire_ticks = 1` and never counts it down, and the renderer draws
`block/fire_0`/`block/fire_1` flames for `is_on_fire` mobs (both
sprites exist in the prepared data). The synthesized overworld day
timeline (481494bd) does feed the mob logic's no-biome `sky_darken`
path — noon gives sky_darken 0, midnight >= 4. The skeleton the owner
saw was most likely shaded, helmeted, in water, or freshly spawned:
ignition is probabilistic (4% a tick at full brightness, mean 1.25 s,
about one in five takes over 2 s). No code change to the burn path was
needed. What the jar check did turn up is a real list error in the
other direction: `burns_in_day` (mobs/species.rs) burned three species
vanilla 1.21.7 does not — the drowned and the zombie horse never call
`isSunBurnTick` (Drowned.aiStep has no such call; ZombieHorse has no
aiStep override at all), and the zombie nautilus does not exist in
1.21.7. The list's comment claimed a `burn_in_daylight` entity tag
that does not exist in 1.21.7's data. Corrected to the jar's list:
skeleton, stray, bogged, wither skeleton (AbstractSkeleton.aiStep),
zombie, zombie villager (Zombie.aiStep gated on isSunSensitive; the
husk returns false there), phantom (Phantom.aiStep) — and the comment
now cites the jar. The same 10-species list is in 1.21.5, 1.21.6,
1.21.8, 1.21.9, 1.21.10, 1.21.11 and 26.2; each needs its own jar check
before correcting (not done here). Regression tests: `sun_burn_tests`
(mobs/ai.rs, 6 tests: skeleton burns in open daylight for 160 ticks,
not at night, not under a roof; helmet wears instead of burning; zombie
and phantom burn; drowned, zombie horse and husk do not) and
`legacy_day_timeline_feeds_sky_darken` (environment.rs), all passing.

Hand-planted plants fail survival: the runtime half of the trees bug
(2026-11-22). The trees fix synthesized the missing `supports_*` tags
only in worldgen, but the runtime survival checks in
`level/survive.rs` read the same tags from `b.tags` and still got
empty sets, so hand-planted saplings, crops, cacti and sugar cane
failed survival in-game while worldgen trees decorated fine. Fix:
`synthesize_legacy_support_tags` moved to `worldgen/tags.rs`
(shared), split into block synthesis (19 tags) and fluid synthesis
(3 tags via `LEGACY_SUPPORT_FLUID_TAGS`), and `level/data.rs`
`build_with_gameplay` now applies both before `Tags::resolve` /
`fluid_mask` — the single synthesis point, since `load` and
`load_with_gameplay` both funnel through it. Tags the bundle already
has keep their own members. New mappings, all verified against the
1.21.7 jar's bytecode: `supports_bamboo` =
`#bamboo_plantable_on` (`BambooStalkBlock`/`BambooSaplingBlock`),
`supports_cocoa` = `#jungle_logs`, `supports_crops` = farmland,
`supports_chorus_plant` / `supports_chorus_flower` = end stone,
`supports_big_dripleaf` = `#big_dripleaf_placeable`,
`supports_small_dripleaf` = `#small_dripleaf_placeable`,
`supports_lily_pad` = ice + frosted ice (block) and still water
only (fluid), `supports_frogspawn` = still water only (fluid).
`supports_frogspawn` is deliberately NOT synthesized as a block tag:
`FrogspawnBlock.mayPlaceOn` is fluid-only in the 1.21.7 jar, so the
block tag is correctly empty. Lily-pad subtlety: packed ice and blue
ice are NOT `IceBlock` subclasses in 1.21.7 (packed ice is a plain
`Block`, blue ice a `HalfTransparentBlock`), so only ice and
frosted ice count as ground — the test pins both negatives. A second
defect surfaced while testing: 1.21.7 still names the class
`WaterlilyBlock` while the port's class table carries the later
`LilyPadBlock`, so the `LilyPad` arm in `survive.rs` never ran at
all; `level/classes.rs` now accepts the legacy name (the `classes!`
macro takes `|` aliases). Regression tests:
`legacy_support_tags_synthesize_from_equivalents` moved to
`worldgen/tags.rs` and extended (all runtime tags, flowing-water and
packed/blue-ice negatives, frogspawn-block absence), and
`hand_planted_plants_survive_on_their_supports` (level/tests.rs) —
a 23-case grid plus cocoa and sugar-cane cases through the runtime
path; it fails with the synthesis disabled (negative control). The
fix also repairs three pre-existing level failures
(`grass_bone_meal_places_grass_and_flowers`,
`tall_flowers_and_seagrass`, `two_by_two_saplings_grow`). Suites:
worldgen 147 pass / 0 fail; level 113 pass / 10 fail, all 10
pre-existing with identical symptoms on the unmodified tree
(baseline 109 pass / 13 fail). Leakage audit: 1.21.7 leaks 62 -> 39;
all 23 `supports_*` findings now `accepted` via
`tools/leakage-exceptions.json` (one entry per name, with the
jar-verified reason), and the baseline shrank 59 -> 39 names. Known
adjacent gaps, not fixed here: `support_override_cactus_flower` is
a separate tag family (cactus flower has an
`is_face_sturdy_center` fallback, so it is a lesser gap); 28 class
names in the 1.21.7 export don't map to the port's class table —
the ones with live `survive.rs` arms are `FarmBlock` (farmland
decay never runs), `FungusBlock` and `RootsBlock` (nether fungi /
roots fall back to `supports_vegetation` instead of their
`support_tag`), and `RedStoneWireBlock` (capital S; the redstone
wire arm never runs); `cannot_support_*` /
`enables_bubble_column_*` tags are still absent (bubble columns
never form, parity-sweep G1). Same synthesis is needed in
1.21.5/1.21.6/1.21.8/1.21.9/1.21.10 — carry-over sequenced
separately.

Tree leaves disappear in fresh worlds (2026-11-22). The owner
reported leaves vanishing over time in two new worlds on the
verification build. Root cause: the runtime leaf-distance lookup
`leaf_distance_at` in `level/shapes.rs` checked the 26.x tag
`#prevents_nearby_leaf_decay` for distance 0, but that tag does not
exist in 1.21.7 data, so the set resolved empty and logs never
counted as distance 0. Vanilla 1.21.7's
`LeavesBlock.getOptionalDistanceAt` returns 0 for `BlockTags.LOGS`
(jar-verified against the named 1.21.7 server jar). Every leaf tick
and neighbour update therefore recomputed distances without log
anchors: log-adjacent leaves inflated 1 -> 2 -> 3 ..., the wave
spread across the canopy, leaves hit distance 7 and the random
tick decayed them. The trigger is self-feeding: decoration runs
the runtime `updateShape` on tree faces (`update_shape_at_edge`),
and the broken lookup schedules spurious leaf ticks for
log-adjacent leaves, which persist into the saved chunk and run
on load. Worldgen itself was exonerated: a new test
(`generated_tree_leaves_carry_distances_from_their_trunks`)
generates the 3x3 region around chunk (-2,-2) at the Test 1 seed
and finds zero birch leaves at distance 7 — the BFS in
`TreeFeature.updateLeaves` places correct distances; the runtime
destroys them. Region analysis of the owner's two fresh worlds
confirmed the runtime signature: ~40-49% of leaves at default
distance 7 (vs 2 leaves in the whole vanilla reference world),
per-tree all-or-nothing, dark oak 0% (its dense 2x2 canopy
inflates slowest). Fix: `leaf_distance_at` now reads `b.tags.logs`,
and the worldgen `leaf_distance` helper in
`worldgen/tree_blocks.rs` does the same (it is `getOptionalDistanceAt`
by contract); the dead `prevents_nearby_leaf_decay` tag field and
`PREVENTS_LEAF_DECAY` flag are removed. Regression tests: the
existing `crops_grow_and_leaves_decay` (a leaf next to a log must
get distance 1 after a tick) failed on the unmodified tree
(distance 7) and passes with the fix; the new worldgen test
guards the BFS. Leakage audit: 1.21.7 leaks 39 -> 38, baseline
shrunk by the one name. Same defect class as the water and trees
bugs: 26.x tag-based logic for a tag absent in 1.21.7. The same
`prevents_nearby_leaf_decay` reference is a recorded leak for
1.21.11 too — carry-over sequenced separately.

Undead don't burn at dawn: the synthesized day timeline used the wrong
time-of-day curve (2026-11-22). The owner reported, playing the
verification build, that undead spawned near him when the game first
loads don't burn immediately -- some burn after a while, some never.
The burn formula itself was already verified exact against vanilla
1.21.7 (651b43d3); the gap was in the daylight gate around it. New
worlds start at tick 0 (dawn), like vanilla, but the port's
synthesized day timeline (`legacy::time_of_day` in
crates/horizon-client-render/src/environment.rs) used the LINEAR day
fraction `(ticks/24000 - 0.25) mod 1` where vanilla's
`DimensionType.timeOfDay` is the non-linear blend `(2*d3 + d5)/3`
with `d5 = 0.5 - cos(d3*PI)/2`, and its sky light used
`(1 - (cos*2 + 0.2))` where vanilla's `Level.updateSkyBrightness`
is `(1 - (0.5 + 2*clamp(cos(timeOfDay*2*PI), -0.25, 0.25))) * 11`
(both verified against the 1.21.7 server jar's bytecode; the
`getSkyColor` daylight factor against the 1.21.6 client jar). The
port's curve kept `sky_darken` >= 4 -- no daylight, no burning -- for
ticks 0..840 (~42 seconds of every dawn, including world creation)
and from tick 11160 (dusk ~69 seconds early), while vanilla's
`skyDarken` is already 0 at tick 0 and stays bright until tick
12541: vanilla's night is 12541..23459, the port's was 0..840 plus
11160..23999. The `monsters_burn` gate ([12542, 23460), the day.json
step track) was already vanilla-correct; only the daylight curve
was wrong. The fix replaces both formulas with vanilla's; the
1000-tick keyframe interpolation shifts the night boundary by at
most ~7 ticks (dawn opens at 23467 vs vanilla's 23460, dusk closes
at 12534 vs 12541), and the exact `monsters_burn` gate masks the
residual for burning. The same wrong base also made the visual
daylight factor and star brightness wrong at dawn/dusk -- fixed by
the same change, since they share `time_of_day`. The "some never
burn" half is vanilla-correct: the 13 undead in the owner's fresh
world ("New World (1)", Time 2976) are all genuinely underground
(sky light 0 at the eye, solid roof above), and shade/canopy mobs
don't burn in vanilla either. Regression test:
`legacy_day_timeline_feeds_sky_darken` now pins dawn (0, 500),
sunset (12000, 12500), dusk (13000), pre-dawn (23000) and night-end
(23500), plus a 250-tick sweep of the whole day; it fails on the old
curve at "clear dawn" (negative control). Suites: environment 15
pass / 0 fail. The `legacy` synthesis exists only in 1.21.7 -- no
sibling carry-over. Known adjacent gap, not fixed here: the
sunrise/sunset colour is still not synthesized (its constants
couldn't be verified), so there is no dawn/dusk glow.

Menus show raw language keys (2026-11-22). The owner reported "the
menu are bugged out": the World Options screen showed raw keys for
nearly every label (`options.worldOptions.title`,
`options.worldOptions.game_mode`, `editGamerule.inGame.button`,
`restrictions_screen.button`, `menu.multiplayerOptions.lan`,
`menu.multiplayerOptions.applyChanges`, ...), the pause menu showed
`worldOptions.button`, and the title screen's friends tooltip showed
`gui.friends.open.tooltip`. Root cause: all of these are 26.x-only
keys. Vanilla 1.21.7's en_us.json (verified against the 1.21.7
client jar, 7,392 keys) has none of them, and the port's bundled
horizon en_us.json didn't carry them either -- in 26.3 they resolve
from vanilla's lang, so the bundle never needed them. Same defect
class as the water/trees/leaves bugs: 26.x data the port reads that
does not exist in 1.21.7. Fix: the 1.21.7 bundled horizon en_us.json
now carries the 36 keys the menu code names that are missing from
both vanilla 1.21.7 and the bundle, with vanilla 26.3's wording
(verified against the Mojang 26.3 client jar, sha1-checked) -- the
World Options labels, friends labels, title-screen tooltips, the
upgrade-flow strings and the Edit Game Rules discard dialog. The
bundled layer loads first, so a resource pack or a newer vanilla
lang file can still override them; other languages fall back to
these English strings, as vanilla does. Regression test:
`bundled_layer_carries_the_26x_menu_keys_1_21_7_lacks` in
horizon-client-core's lang.rs pins all 36 through `Language::load`.
Not fixed here, for the owner to decide: (1) the pause menu's four
icon buttons and the title screen's friends button are empty grey
squares -- their sprites (`gui/sprites/pause_menu/bug`,
`social_interactions`, `player_reporting`, `friends/friends`) do
not exist in 1.21.7's assets (verified against the 1.21.7 client
jar; `sprite_scaled` silently draws nothing for a missing sprite),
and bundling Mojang's 26.3 sprites is forbidden. Vanilla 1.21.7
has no icon row at all: its pause menu uses text buttons for Give
Feedback / Report Bugs (`PauseScreen.addFeedbackButtons` builds
plain `Button`s, jar-verified) and its only pause sprite is
`icon/draft_report` for player reporting; its title screen has no
friends button. (2) The World Options screen itself is a 26.x
feature -- vanilla 1.21.7's pause menu has "Open to LAN" instead
(`menu.shareToLan`). Restoring the 1.21.7 pause menu and title
screen to their vanilla forms is a screen-parity rework, not part
of this translation fix.
Survival block breaking drops nothing: the loot evaluators read the
26.3 field names (2026-11-22). The owner reported, playing the
verification build, that breaking blocks in survival dropped nothing
-- a log gave no wood, dirt gave no dirt; blocks vanished as if in
creative mode, though the HUD and World Options both showed survival.
Two hypotheses were checked. The game-mode desync was exonerated:
the world was created creative and switched to survival in-game, but
the mode change propagates intact (window `apply_world_options` ->
server thread `set_owner_mode` -> `hub.set_game_mode` -> `GameEvent`
packet -> window `set_mode_id`), the LAN hub exists at integrated
server startup, and server-side breaks run inside `as_guest`, which
swaps in the guest's mode -- the break path sees survival. The defect
was the loot tables. `LootContext::condition` (horizon-items/items.rs)
read a condition object's kind only from `"type"`, but 1.21.7 data
names it `"condition"`: `dirt.json` and `oak_log.json` both carry
`"conditions": [{"condition": "minecraft:survives_explosion"}]`, so
the kind resolved to `""`, matched no arm, and the pool was skipped
-- 733 of the 1,023 block tables carry that condition (counted in the
laptop's prepared data), so most blocks dropped nothing at all. The
failure is silent (an empty drop vec, no log error). This is the same
defect class the 26.2 port fixed, never carried to 1.21.7; the fix
mirrors 26.2 across all three evaluators. In `horizon-items` (block
and entity drops): `condition()` now reads `"type"` or `"condition"`;
`evaluate()` / `expand()` / nested entries apply `"functions"`
instead of `"modifier"` (1.21.7 data puts entry functions there --
diamond ore's fortune `apply_bonus` and `explosion_decay` were
silently dropped too); `conditions()` drops the single-`"condition"`
check (26.3's form). In `horizon-world` loot.rs (chests, fishing,
dispensers, vaults, decorated pots): the same condition-kind fallback,
plus the function-kind fallback (`"type"` or `"function"`) in `run()`
and `"functions"` / `"conditions"` in `table_raw` / `expand` /
`create` / `modify`. In `loot/odds.rs` (the odds calculator, whose
kind reading already handled both keys): the container keys migrate
to `"functions"` / `"conditions"`; odds.rs and odds_tests.rs are now
byte-identical to 26.2's. Test fixtures in items.rs, loot.rs,
odds_tests.rs and terrain_cmds.rs move to the 1.21.7 form (26.2 left
its terrain_cmds fixtures in 26.3 form -- worth a look there).
Regression tests: items.rs
`conditions_read_the_1_21_7_condition_key_as_well_as_type` (a dirt
table naming `survives_explosion` via each key drops dirt), loot.rs
`conditions_read_the_1_21_7_condition_key_as_well_as_type`
(`random_chance` 1.0 via each key) and
`entry_functions_apply_before_pool_and_table_functions` (entry 2 +
pool 3 + table 10 = 15); all three fail with their production line
reverted (negative control) and pass with the fix. Suites:
horizon-items lib 111 pass / 0 fail (1 ignored); horizon-world lib
loot filter 39 pass / 0 fail. `cargo check` on both crates passes;
clippy and the horizon-client suite were not run locally (VM
contention -- one test build OOM-killed, the next needed
`taskset -c 0`); CI covers them. Known adjacent gaps, not fixed here:
loot.rs has no `survives_explosion` arm (no table reaching that
evaluator carries it in 1.21.7 data, so it is dormant); the prepared
data has no `item_modifier` / `predicate` directories, so registry
references to them resolve to nothing; `horizon-port-pack`'s carry
code still writes 26.3-form `"modifier"` / `"condition"` keys (26.2
left it the same way). Same defect class as the water, trees and
leaves bugs: 26.x field/tag names for things 1.21.7 names
differently. Sibling versions that inherited the same 26.3-form
evaluators need the same carry-over -- sequenced separately.
Underground water deficit: not the aquifer -- the waterfalls never
ticked (2026-11-22). The world comparison (same seed, 4,145
paired-full chunks) showed water 1,392,233 vs vanilla 1,406,861
(-14,628, -1.04%), all of it underground; worst chunk (-22,-18)
43 vs 745. The aquifer is exonerated: a Java oracle driving
vanilla 1.21.7's own `NoiseBasedAquifer` over that chunk agrees
with the port on all 94 common cells -- identical levels,
identical points, floodedness/spread/barrier identical to 4
decimals -- and vanilla's aquifer also returns air at both water
columns at density -1.0. The vanilla water is not placed water at
all: the cave columns are level-8 falling / level-1 flowing water,
one connected waterfall system fed by a single spring source at
local (8,91,0) -- the only level-0 source in the chunk. Fresh
generate+decorate of the port places exactly that source (and its
delay-0 tick) and nothing else; the waterfall only exists after
the chunk is ticked. Vanilla's chunk has InhabitedTime 0 in both
worlds, yet vanilla's waterfall formed: InhabitedTime increments
only for spawning chunks (within 128 blocks of a player), while
fluid ticks run at simulation distance (10 chunks), so a chunk
129-160 blocks from the player flows its waterfalls without ever
incrementing InhabitedTime. The port's machinery is
vanilla-faithful end to end -- postprocessing marks, scheduled
ticks, persistence, the tick gate -- and where both sessions
ticked a chunk the counts match exactly (e.g. (-15,-12) 1274=1274
with 923 underground). The deficit is session geometry: the port
world is a pregen-shaped rectangle (10,744 chunks, 2,653 with
pending fluid ticks) whose chunks were generated and postprocessed
but never ticked, while vanilla's world is an explored world whose
chunks sat in ticking range. 55% of the deficit (7,994 of 14,628)
sits in chunks with pending port fluid ticks -- waterfalls started
and stalled. No behaviour fix: changing tick gating or postprocess
timing would diverge from vanilla, which also leaves water
unflowed in unticked chunks. Regression tests:
`worst_chunk_places_the_single_spring_source` pins the placement
(exactly one water block at (8,91,0) plus its delay-0 water tick);
`pond_spring_waterfall_flows_into_cave_at_worst_chunk` and
`spring_waterfall_flows_in_cave_at_18_6` generate + decorate +
tick 1000 ticks and assert the waterfalls form ((-22,-18): 843
total / 652 underground after 1000 ticks, vanilla 745/554 --
vanilla's saved chunk was still mid-cascade with 13 pending ticks;
(18,6): 312/312, vanilla 288/288). The (18,6) test also resolves
that chunk's anomaly: the saved port world has stone at the spring
and no water at all, but current code flows it correctly, so the
saved chunk is stale morning generation (world created 08:06 with
build 0e1b38b2, before the water-spread fix), not a current bug.
The old probe test asserted `generate_proto` alone floods the
cave -- a wrong premise, since neither vanilla nor the port places
this water at generation; it is replaced by the tests above.
Separate note, not this fix: the port's `FindTopSurfaceSampler`
returns `lower_bound` (-64) when no surface is found; vanilla's
`NoiseChunk.computePreliminarySurfaceLevel` returns
`Integer.MAX_VALUE`. Real divergence, but not the (-22,-18)
mechanism (mountains, surface found); the sampler is shared with
26.3-form graphs, so check 26.3's not-found semantics before
changing it.
Worlds generated by this client show "Upgrade and Play" instead of
"Play" (2026-11-22). The owner generated a world with the 1.21.7
client and the world list offered "Upgrade and Play", with Edit and
Re-Create greyed out -- even though the world is stamped correctly
(DataVersion 4438, the 1.21.7 value). Root cause: the world-list
`file_fixing` flag comes from `save::needs_file_fixes`, which calls
`datafix::files::requires_file_fixing`, and that function carried
the 26.x form verbatim: it compared against the last file fix
overall (4899) and mapped every pre-4772 world to 0. In 26.3 that
is coherent (current version 5023 > 4899), but 1.21.7's current
version is 4438, below both constants, so the check fired for
every 1.21.7 world. Jar-verified: vanilla 1.21.7 has no
`FileFixerUpper` at all (no `net.minecraft.util.filefix` in the
1.21.7 server jar -- it arrived with the 26.x layout moves at
DataVersion 4772), 1.21.7's `LevelSummary` has no
`requiresFileFixing` method, its `BackupStatus` is only NONE /
DOWNGRADE / UPGRADE_TO_SNAPSHOT, and its `primaryActionMessage`
always returns PLAY_WORLD. Fix: `requires_file_fixing` now returns
true only when a fix newer than the world exists at or below the
current version -- in this build, only 1493
(`LegacyStructureFileFix`, for 1.12 worlds). A world at 4438, or
at 4435 (1.21.6) or 1519 (1.13), needs no file fixing; 1343 (1.12)
and -1 (no DataVersion) still do. The upgrade path is unchanged
for worlds that genuinely need fixing: `fix_world_files_deferred`
still runs 1493 for pre-1493 worlds, and the 4772/4773/4899 fixes
were already filtered out by `version <= CURRENT`. The one test
that encoded the 26.x form
(`world_gen_settings_of_a_world_needing_file_fixes_are_read_in_memory`,
fixture at DataVersion 4700 -- a future version in this build) now
uses a 1.12.2 fixture (1343). Regression test:
`a_current_world_needs_no_file_fixing` pins the check at 4438,
4435, 1519, 1343 and -1. Same defect class as the water, trees and
leaf-decay bugs: 26.x logic for a system absent in 1.21.7. Note:
`failed_upgrade_keeps_old_data_version` fails in this tree both
before and after the fix (pre-existing: its 4899 fixture is a
future version here, so the upgrade returns early and never stamps
CURRENT).

The 1343 fixture's settings path is pinned by a second test,
`current_world_gen_settings_of_a_1_12_world_are_vanilla_like`, in
`horizon-world` (a mirror of the save test's datafix assertions:
the 2550 fixer rebuilds the settings as vanilla levels, 2832
converts the Overworld's `vanilla_layered` source to the
multi-noise overworld preset, 4771 renames the key, and the three
dimensions come out in exactly the shape
`dimension_is_vanilla_like` accepts). It skips without the
prepared datafix tables, like the other pre-flattening tests --
no `.local` exists on this VM, so it skips here and runs in a
prepared tree. The `horizon-client` lib test itself could not be
run on this VM: the `horizon-client` test compile was OOM-killed
three times (single rustc past 3 GB against the shared cell),
even single-threaded via `taskset -c 0`; the save test's
assertions were instead verified by enumerating every fixer that
touches `Ty::WorldGenSettings` between 1343 and 4772 (2550, 2825
`add_flag`, 2832, 2967 no-op on string settings, 3440 no-op on
these presets, 4771) plus the schema walker's nested visits
(none apply: multi-noise/`the_end` sources, string settings).
Worth a run where memory allows before relying on it. Suites:
`horizon-world` `datafix::files::upgrade_tests` 6 pass / 1 fail
(the pre-existing one above).

Drowned DOES burn in daylight: 651b43d3 was partly wrong (2026-11-22).
The parity sweep's Batch 3 (item B3-G08) contradicted the burn-list
fix: it said the drowned is missing from `burns_in_day`. A jar
re-check (twice, against the named 1.21.7 server jar) proves the
sweep right and the fix wrong about the drowned. `Drowned extends
Zombie` and declares NEITHER `aiStep` NOR `isSunSensitive` — it
inherits both from Zombie, and `Zombie.aiStep` burns when
`isSunSensitive() && isSunBurnTick()` (Zombie.isSunSensitive returns
true; headgear absorbs, else ignite 8 s). `Mob.isSunBurnTick`
excludes being in water/rain/powder snow, so a drowned IN water does
not burn, but a drowned ON LAND in daylight does — that is vanilla
behaviour. The mechanism is proven by the husk, which also extends
Zombie but OVERRIDES `isSunSensitive()` to return false; that is how
vanilla exempts a Zombie subclass, and the drowned has no such
override. The zombie villager is exactly like the drowned (declares
neither method, burns purely by inheritance) and IS in the fix's
list — same inheritance, opposite treatment. The fix agent had
grepped each class for a direct `isSunBurnTick` call site and missed
inheritance. The other two 651b43d3 removals are correct and stand:
the zombie horse has no aiStep/isSunSensitive anywhere in its chain,
and the zombie nautilus does not exist in the 1.21.7 jar at all; the
husk stays out (isSunSensitive false). Vanilla has no `burns_in_day`
list or tag; the behaviour is emergent. The port's synthesized table
is now the fix's 7 plus the drowned: skeleton, stray, bogged, wither
skeleton, zombie, zombie villager, phantom, drowned
(mobs/species.rs). Tests: the drowned moved out of the non-burners
test into the burners test (`zombie_drowned_and_phantom_burn_in_
daylight`; the zombie-horse and husk halves stay in
`zombie_horse_and_husk_do_not_burn_in_daylight`). The E6 re-base
triage addendum (84200bac) carried the 651b43d3 list as a shared fix
to propagate — corrected on the e6-rebase-triage branch so the
drowned error is not propagated to siblings.
Snow layers on packed ice in frozen peaks (2026-11-22). The
world-compare against vanilla (same seed 8257987075890829262, 4,145
paired chunks) found the port placing +14,242 snow layers (+7.6%),
92% of the excess in frozen_peaks: worst chunk (7,19) had 256 snow
layers, one on every column, all on packed ice, while vanilla has
zero snow in the chunk. Root cause: vanilla 1.21.7's
`SnowLayerBlock.canSurvive` rejects the `snow_layer_cannot_survive_on`
tag first ([ice, packed_ice, barrier], jar-verified, and
`snow_layer_can_survive_on` = [honey_block, soul_sand, mud] overrides
second), but the port bound the 26.x tag names
`cannot_support_snow_layer` / `support_override_snow_layer`, which do
not exist in 1.21.7 data -- both sets resolved empty, so snow survived
on packed ice and `SnowAndFreezeFeature` placed it there. Same defect
class as the water, trees and leaf-decay bugs: 26.x tag names for data
1.21.7 carries under its own names. Fix: bind the two tag fields in
`level/data.rs` (read by survive.rs and weather.rs) to the 1.21.7
names, and the two literal readers (`block_survival.rs`, snow-golem
trail in `mobs/golem.rs`) likewise. No synthesis needed -- the 1.21.7
tags exist with the right contents. Regression tests:
`snow_does_not_survive_on_ice` (fast: no snow settles on packed ice
or ice, still settles on grass and soul sand) and
`frozen_peaks_place_no_snow_on_packed_ice` (worldgen: chunk (7,19)
has zero snow after decoration, zero snow on packed ice in the 3x3
region -- both match vanilla). Leakage audit: 1.21.7 baseline 38 ->
36, ratchet passes (exit 0). The same two 26.x names are recorded
leaks for every version 1.20.5-1.21.11 -- carry-over sequenced
separately (each version needs its own jar check for the tag names).

Upgrade-and-Play follow-up: two test repairs (2026-11-22). The
first crate-tests run covering horizon-client-lib (run 45035966158)
found one real regression from the fix: the title test
`a_world_that_needs_file_fixing_cannot_be_optimized` used a
DataVersion 3000 fixture, which needs no file fixing under the
corrected rule (1493 already applied, 26.x fixes past CURRENT) --
fixture moved to 1343 (1.12), the only version that genuinely needs
fixing here. Second, the save test
`world_gen_settings_of_a_world_needing_file_fixes_are_read_in_memory`
panicked on a clean runner: the 1343 fixture triggers the pre-1.13
upgrade path, which needs `datafix-tables.json` from the prepared
`.local`, and `tables()` panics by design when it is absent. The
test now skips the table-dependent read when
`horizon_world::datafix::tables_available()` is false (same pattern
as the horizon-world mirror test); the `needs_file_fixes` gate
assertion still runs everywhere. The full read still needs a run
where `.local` exists (the laptop).
Remote Getaway fires at spawn, with a stone icon (2026-11-22). The
owner reported the toast "Advancement Made! Remote Getaway" (and
the chat announcement) in a fresh survival world, standing at spawn
with an empty hotbar, and the toast's icon a grey block instead of
an ender pearl. Correction to the report's premise: in vanilla
1.21.7 Remote Getaway is `minecraft:end/enter_end_gateway` (parent
`minecraft:end/kill_dragon`, criterion `entered_end_gateway`,
trigger `minecraft:enter_block`, conditions `{"block":
"minecraft:end_gateway"}`, icon ender pearl) -- verified from the
1.21.7 client named jar and the laptop's prepared data, not the
pearl-through-a-portal of older versions. The misfire is real
either way: no end gateway exists at an overworld spawn.

Trigger root cause: the port read the wrong condition key.
`trigger_enter_block` (`game/advancements.rs`) checked
`adv::id_matches(&c["blocks"], ...)` -- the 26.x plural key -- but
1.21.7 data uses the singular `block`, so `c["blocks"]` is null for
every 1.21.7 criterion and `id_matches` returns true for null ("no
condition"). The block condition was never checked: any
`enter_block` event matched every `enter_block` criterion, and the
motion system emits `EnterBlock` for every block the player's box
intersects, so standing at spawn fired it. The same defect in four
places: `trigger_enter_block` and `trigger_bee_nest_destroyed`
read `c["blocks"]`; `adv::block` read only `v["blocks"]`, so a
bare-string BlockPredicate matched everything; `trigger_honey_slide`
passed the whole conditions object to `adv::block` instead of
`c["block"]`. The fix adds `adv::block_condition`, which reads the
singular `block` key; `adv::block` now accepts bare HolderSet forms
(string, array, or object); the three triggers use it. Blast radius
(census of all 1.21.7 advancement JSONs): top-level `"block"`
conditions appear only for `enter_block` (11 criteria: 10 boat/raft
recipe advancements with `{"block": "minecraft:water"}` plus
`enter_end_gateway`), `slide_down_block` (1, honey block) and
`bee_nest_destroyed` (1, bee nest). The fix also repairs location
predicates with string-form blocks and the `match_block`
sub-predicate. No parent gate was added: vanilla 1.21.7 has none --
`PlayerAdvancements` registers listeners for every incomplete
criterion regardless of parent, and awards whenever the
advancement's own requirements complete -- and the port already
matches that.

Icon root cause: the server thread wrote the icon item-first
(`lan/advancements.rs` `write_advancement`: item id, count 1, 0, 0
-- the 26.3 base's Template layout), but the 1.21.7 tree's pinned
quirks are 1.21.8's, whose Template codec is count-first (vanilla's
`ItemStack.STREAM_CODEC` since 1.20.5). The client therefore read
count 1070 (the ender pearl's id) and item 1 (stone): the grey cube
in the screenshot. The fix writes count-first. The same swap was
applied to the recipe-book result display and the transmute stack
display in the same file (both `ItemStackSlotDisplay`, same codec);
26.3's identical item-first write is correct for 26.3 (its pinned
codec is item-first), so this is a 1.21.7-tree divergence, not an
upstream bug.

Regression tests: `single_block_trigger_conditions_match_only_their_block`
(real `enter_end_gateway` conditions: grass block false, end gateway
true, plus the water form, empty conditions, and the ignored
`{"blocks": ...}` key), `block_predicate_accepts_a_bare_holder_set`
(bare string/array/object forms, null, and a location predicate with
a bare-string block), `an_advancement_icon_reaches_the_client_as_its_item`
(the real 1.21.7 `enter_end_gateway.json` plus a root `kill_dragon`,
granted and read back as `UpdateAdvancements`: icon ender pearl,
count 1), and
`a_recipe_result_reaches_the_client_as_its_item_and_count` (a
no-ingredients shapeless recipe, so no tag display is written;
result copper ingot, count 2). Suites, run under
`--no-default-features` (the documented server build -- the
full-features `horizon-client` lib test compile OOM-kills on this
VM, a single rustc past 2.4 GB against the shared 6 GB cell, so
full-features verification belongs to CI): `horizon-client`
`game::advancements` 11 pass / 0 fail / 2 ignored (need prepared
resources); `horizon-client-lib` `advancements::tests` 48 pass / 5
fail / 1 ignored. The 5 failures are pre-existing and unrelated,
verified identical on unmodified code by a baseline run: the five
recipe-book entry tests misalign because the pinned quirks read a
`TagSlotDisplay` as a bare id string while the writer writes a full
HolderSet (`"unknown slot display 200"` is the cooking time read as
a display type after the misalignment); the swapped-input test
itself expects `HolderSet::Items`. That tag-display mismatch is a
separate defect, out of scope here. Carry-over: the trigger-key
defect likely affects every E6 sibling (the 26.3 base reads
`c["blocks"]` too, and sibling data uses the singular key) -- a
re-base candidate; the icon swap needs a per-tree check against each
tree's pinned Template codec. Notes: the transmute `stack` closure
has no test coverage (no transmute tests exist); the `{"blocks":
{"tag": ...}}` HolderSet-object form is unused in 1.21.7 data and
`id_matches` returns false for it.

Menu sprites fall back to the 26.3 client jar at prepare time
(2026-11-22). The pause-menu icon row (four buttons) and the
title-screen friends button rendered as empty grey squares: a
missing sprite resolves to the MISSING_SPRITE rect, and 1.21.7's
jar ships no `pause_menu/` or `friends/` sprites at all. Owner
decision 22:38 NPT Nov 22, reversing the 18:30 call to leave the
icons empty: menu chrome does NOT need to be version-faithful --
only the game, items, entities and their behaviours must
correspond to the respective version -- so the 26.3 sprites are
used for the 26.x menus. The owner's clarification ("or whatever
the horizon launcher had set the menu for") was checked first:
Horizon ships no menu icons of its own (bundled assets are lang
files plus controller glyphs; launcher assets are loader logos
and art), so the 26.3 fallback is the path.

Census, jar-verified against sha1-verified client jars from
Mojang: 1.21.7 has 442 `gui/sprites`, 26.3 has 469. Of the 129
literal sprite references in the client code, 8 are missing from
1.21.7, all present in 26.3: the four pause icons
(`pause_menu/bug`, `pause_menu/social_interactions`,
`pause_menu/player_reporting`, `friends/friends`), the two
inventory effect backgrounds, `container/slot/nautilus_armor_inventory`
and `widget/preedit`. The full jar diff is 29 sprites 26.3 has
that 1.21.7 lacks: `friends/` 17, `pause_menu/` 3, the effect
backgrounds 2, nautilus/spear slots 3, `widget/preedit` plus the
three realms buttons. `assets/minecraft/atlases/gui.json` is
identical in both jars (a single directory source over
`gui/sprites`), so the dropped PNGs appear in the atlas with no
atlas-builder change.

Mechanism: `fallback_menu_sprites` in
`crates/horizon-prepare/src/lib.rs`, called in the `files` lane
after the version's own resources are extracted. It opens the
sibling 26.3 client jar
(`versions/26.3/.local/minecraft/26.3/jars/client.jar`, located
through the sibling's `research-lock.json`) and extracts every
member under `assets/minecraft/textures/gui/sprites/` whose file
is missing from the prepared resources. Only missing files are
filled, so 1.21.7's own art always wins; the fallback is bounded
to the `gui/sprites/` subtree, so gameplay assets (blocks, items,
entities, sounds) stay strictly 1.21.7. When the 26.3 jar is not
prepared it logs one line and fills 0 -- no failure, and the menus
stay as 1.21.7 ships them. No Mojang file is committed: the
sprites arrive from the player's own jar at prepare time. The
manifest's `resource_files` count is unchanged (it counts the
version's own jar members only).

No `OUTPUT_VERSION` bump, deliberately: what the fallback fills
depends on an unpinned input (the sibling 26.3 jar), which the
output digest cannot represent, and in any environment that
enforces the digest (a single-version prepare, no sibling jar)
the fallback is inert, so the v4 digest still holds. Consequence:
an existing preparation needs one manual re-prepare to pick the
sprites up; fresh preparations get them automatically.

Test: `menu_sprites_fall_back_to_the_26_3_jar` (fake 1.21.7
resources plus a fake 26.3 jar) -- fills only the missing
sprites, never overwrites the version's own file, copies nothing
outside `gui/sprites/`, fills 0 on a second prepare, and is a
silent no-op without the jar. `horizon-prepare` lib suite on the
VM: 66 pass / 0 fail; `cargo fmt --check` clean except one
pre-existing diff at lib.rs:719, and `cargo clippy -D warnings`
fails only on a pre-existing `nonminimal_bool` in
artifacts.rs:1367, untouched here. Visual verification on the
laptop is still owed, and it depends on the 26.3 client jar being
prepared there -- if it is absent the fallback silently does
nothing and the icons stay grey.
Recipe/trim advancement misfire (2026-11-22). The owner crafted birch
planks and was granted "Smithing with Style" (challenge) and "Crafting
a New Look" -- both smithing-trim advancements. Same defect class as
the Remote Getaway fix above, in four more triggers: the port read
the 26.x plural condition keys while 1.21.7 data uses singular ones,
so the condition was null and `id_matches` treated it as
unconstrained. (1) `recipe_crafted` read `c["recipes"]`; 1.21.7 data
uses `recipe_id` (27 criteria, all strings; jar-verified, and
`RecipeCraftedTrigger$TriggerInstance` carries `recipeId`). Any craft
matched every `recipe_crafted` criterion -- one birch-plank craft
satisfied all 8 trim criteria of "Smithing with Style" at once.
(2) `player_generates_container_loot` read `c["loot_tables"]`; 1.21.7
uses `loot_table` (10 criteria: archaeology + bastion chests;
`LootTableTrigger$TriggerInstance` carries `lootTable`). Any loot
container opened matched every loot criterion. (3) `recipe_unlocked`
read `c["recipes"]` in both `index_recipes` and the trigger
predicate; 1.21.7 uses `recipe` (1,395 criteria). Opposite failure
mode: the predicate required the absent key, so NO recipe advancement
ever granted (and every criterion fell into the `recipes_any`
fallback bucket). (4) `crafter_recipe_crafted` read `c["recipes"]`;
1.21.7 uses `recipe_id` (1 criterion, `crafters_crafting_crafters`
naming `minecraft:crafter`; the crafter trigger reuses
`RecipeCraftedTrigger`'s instance). Any crafter craft granted it.
Fix: new `adv::recipe_condition` (reads `recipe_id`) and
`adv::loot_table_condition` (reads `loot_table`), used by the
recipe, crafter and loot triggers; `index_recipes` and the
`recipe_unlocked` predicate read `recipe`. The crafter's private
`recipe_matches` helper is deleted (the shared helper covers it).
Vanilla's `recipe`/`recipeId`/`lootTable` fields are required, not
optional (javap on the 1.21.7 named jar), so a criterion naming none
is invalid data and must not match -- the `recipe_unlocked` predicate
keeps `is_some_and`. Full audit: every other trigger in
`game/advancements.rs` was checked against a census of all 1,520
1.21.7 advancement JSONs from the jar (condition keys per trigger)
and reads the correct 1.21.7 keys, including `inventory_changed`
(`items`/`slots`), `location` (`player` only -- the location
predicate nests inside it), `bred_animals` (`parent`/`partner`/
`child`), `killed_by_arrow` (`fired_from_weapon`/`victims`/
`unique_entity_types`) and `placed_block`/`item_used_on_block`
(`location` only in 1.21.7 data). The 26.x-only triggers
(`spear_mobs`, `default_block_use`, `any_block_use`) have no 1.21.7
data and cannot misfire. Diagnosed gaps, not fixed: `trigger_block`
does not evaluate vanilla's `block`/`item` keys for placed_block /
item_used_on_block (no 1.21.7 criterion uses them -- datapacks only);
the 1.21.11 jar uses the same singular keys, so the E6 siblings
share all four defects (re-base candidate, per-version jar checks
still required). Regression tests: `recipe_conditions_match_only_
their_recipe` (real trim + crafter criteria; birch planks do not
match the trim), `loot_table_conditions_match_only_their_table`
(real bastion criteria), the whole-id test rewritten through
`recipe_condition`, and the recipe-index + bare-id fixtures moved
from `recipes` to `recipe` (they encoded the 26.x form and passed
while real data failed).
## Crate-test repairs: three 26.x datafix fixtures (2026-11-22)

Three `horizon-world` datafix tests failed deterministically in
crate-tests runs 4-6, all three test bugs, not product defects: each
pinned a 26.x behaviour that vanilla 1.21.7 does not have, against a
fixture from a future version. All three verified against the 1.21.7
named jar (`1.21.7-server-named.jar`). No production code changed;
the rewritten tests are the regression tests.

- `failed_upgrade_keeps_old_data_version` (`datafix/files.rs`): the
  fixture world was DataVersion 4899, a 26.x version in the future
  for 1.21.7 (CURRENT 4438), so the upgrade correctly left it alone
  and the stamp assertion compared 4899 to 4438. Fixture now 4000
  (1.20-era, past the only applicable file fix, 1493). The same
  future-version fixture in `upgrade_reports_one_operation_per_file`
  and `cancel_stops_the_data_upgrade_and_keeps_the_version` --
  passing only vacuously, since a future world has no pending stamp
  for the cancel to withhold -- was changed to 4000 too.
- `block_states_take_their_new_field_names`, renamed
  `block_states_keep_their_1_21_7_field_names` (`datafix/tests.rs`):
  the palette rename `Name`/`Properties` to `id`/`properties` is
  fix 5006, a 26.3 fix above CURRENT, and the old fixture was a 4903
  chunk (future here, left alone). Vanilla 1.21.7 palettes still use
  `Name`/`Properties`: its jar's DataFixers top out at V4421, with no
  5006 rename fix. The test now upgrades a 4435 (1.21.6) chunk and
  asserts the fields are kept, `id` is absent, and the version is
  stamped.
- `level_data_of_1_20_moves_to_the_new_fields`, renamed
  `level_data_of_1_20_keeps_its_1_21_7_fields` (`datafix/tests.rs`):
  the expected fields are all 26.x moves above CURRENT --
  `difficulty_settings` (4771), `game_rules` (4658), `spawn` (4548).
  Vanilla 1.21.7's `PrimaryLevelData` still reads `Difficulty`,
  `DifficultyLocked`, `GameRules` and `SpawnX`, and its jar's only
  `Level*` fixes are the pre-1.20 ones
  (`LevelDataGeneratorOptionsFix`, `LevelLegacyWorldGenSettingsFix`,
  `LevelUUIDFix`, `LevelFlatGeneratorInfoFix`). The test now asserts
  a 1.20 level keeps its fields and gains none of the 26.x
  compounds. No fix in the 1.20 -> 1.21.7 range touches these
  fields (checked: no `Ty::Level`/`GameRules` fix in `fixes_a`/
  `fixes_b`, and the in-range `fixes_c` fixes are entity/player
  only).

Verification, run locally on this VM (`taskset -c 0 cargo test
--offline -p horizon-world --lib`, ~20 min compile under load, suite
325s): full lib suite **471 passed / 0 failed / 3 ignored** (the CI
baseline was 468 passed / 3 failed / 3 ignored). The datafix subset
is 35/35 and the `files::upgrade_tests` subset 7/7.
## Pack compatibility: 26.x fixtures, 1.21.7 formats (2026-11-22)

Three `horizon-client-render` crate-test failures (identical sets
across crate-tests runs 4, 5 and 6) were test bugs, not product bugs:
the fixtures carried 26.3-era pack formats (resource 97-98, data 121)
while vanilla 1.21.7's own `version.json` (in the 1.21.7 server jar)
pins resource 64 and data 81 -- and the tree's `version.rs`
constants were already right. The failing tests
(`create_world_installs_its_packs`, `compatibility_ranges`,
`overlays_and_filters`) now use 1.21.7 forms: `pack_format` /
`supported_formats` only, against 64 (resource) and 81 (data).

Fixing the tests exposed two real product divergences in the same
code, both jar-verified against 1.21.7's `Pack.readPackMetadata` /
`PackCompatibility.forVersion` (which takes a plain int version --
1.21.7 has no minor pack versions -- and no Unknown state: an
unparseable pack.mcmeta means `readMetaAndCreate` returns null and no
pack is created):
- a lone `pack_format` above the current version now reads as a
  single-value range, so the pack is TooNew instead of Unknown (the
  26.x code refused it for missing min/max_format, fields 1.21.7
  does not have);
- `supported_formats` no longer has an upper bound: the range stands
  when it holds `pack_format`, and otherwise vanilla's fallback
  applies (warn and default to the single-value `pack_format` range).

Regression tests: the three rewritten tests plus the new
`pack_format_validation` assertions (a lone pack_format 97 reads as
its own range; a [34,70] range stands; a pack_format outside its
range defaults to itself) and the TooNew assertions in
`compatibility_ranges`. Known gap, not fixed here: the
min_format/max_format validation machinery and the
min_format/max_format overlay-entry form are 26.x carry-overs --
1.21.7 vanilla has neither (its `PackMetadataSection` codec is
description + required pack_format + optional supported_formats; its
overlay entries carry `formats` only). They are harmless for real
1.21.7 packs, which never declare those fields, but a 26.x pack
dropped into 1.21.7 is judged by the 26.x rules instead of vanilla's
pack_format-only reading. Stripping the machinery is a re-base
candidate, not part of this fix. (The `a_resource_pack_overrides_a_texture`
fixture still says pack_format 97; it never asserts compatibility,
so it was left alone.)
Crate-test repairs: vibration particle id, remote Game Rules button,
launch reference (2026-11-22). Three pre-existing `horizon-client`
failures from crate-tests run 45039219772, all 26.3-base leakage of
one kind or another. (1) TEST BUG: the vibration test
(`vibration_particles_carry_their_listener_and_arrival_ticks`) built
its hub registry with `minecraft:vibration` at id 58, but vanilla
1.21.7's `ParticleTypes` registration order puts it at 47 (verified
with `javap -c` on the 1.21.7 named server jar's
`ParticleTypes.<clinit>`; id 58 is `rain`, which carries no data --
hence the 10 unread bytes: 1-byte source kind + 8-byte block position
+ 1-byte arrival ticks). Production `Particle::read` already reads
vibration data at 47 (`horizon-protocol/src/entity_data.rs`), so only
the fixture and the id assertion changed. (2) PRODUCT FIX:
`remote_options_follow_vanilla_guest_layout_and_permissions` -- the
remote World Options screen
disabled the Game Rules button unconditionally, on a 1.21.11-era
comment claiming there was no packet to fetch a server's rules.
Vanilla 1.21.7 has no game-rule packets at all (jar check: no
`*GameRule*` packet class among the 155 clientbound play packets;
`/gamerule` requires permission level 2), and `close_game_rules`
already sends the changes as `/gamerule` commands on a remote server,
gated on permission >= 2 -- so the button now follows `gamemaster`
(permission >= 2), including in hardcore, and the stale 1.21.11
comments now say 1.21.7. (3) STALE DOC: `launch_api_reference_is_current` -- the
`--resources` option text named
`.local/horizon-client/26.3`, while the real default is
`.local/horizon-client/<VANILLA_VERSION>` (`launch_options.rs`), i.e.
`1.21.7` here. Regenerating `docs/api/launch-reference.md` with
`HORIZON_WRITE_DOCS=1` also fixed three more 26.3 leftovers the doc
had carried since it was copied from the base: `"minecraft": "26.3"`
-> `"1.21.7"`, datapack format [121, 0] -> [81, 0], resourcepack
format [97, 1] -> [64, 0]; everything else in the regen reproduced
identically. Verification, run under `--no-default-features` (the
documented server build -- the full-features `horizon-client` lib
test compile OOM-kills on this VM, so full-features verification
belongs to CI): the whole `horizon-client` lib suite passes, 1242
pass / 0 fail / 470 ignored (need prepared resources), with the
three target tests green and their module suites clean
(`lan_particles` 5/5, `lan_screen` 6/6, `launch_api` 6/6).
Recipe-book entries misaligned on tag displays (2026-11-22). The
five `lan::advancements` crate-tests failures (`smelting_reaches_the_
furnace_book`, `blasting_reaches_the_blast_furnace_book`,
`a_recipe_filters_swapped_input_shows_swapped_in_the_book`,
`a_neoforge_ingredient_in_the_key_reaches_the_client_with_items`,
`an_owned_content_recipe_reaches_the_client_book` -- identical in
crate-tests runs 4, 5 and 6) shared one cause: the pinned quirks read
a `TagSlotDisplay` as a bare tag id while `book_entries` wrote a full
HolderSet after the tag type, so the entry read misaligned and the
cooking time (200) decoded as a display type ("unknown slot display
200"). Jar-verified against 1.21.7's own server jar, the READER was
right and the writer wrong: vanilla 1.21.7's `TagSlotDisplay` holds a
`TagKey` (its `STREAM_CODEC` is `TagKey.streamCodec`), and
`Ingredient.display` -- which the cooking and crafting displays both
use -- maps a tag to that display and a direct item list to a
composite of `displayForSingleItem` each, even for one item. Fix
(`lan/advancements.rs`): a new `ingredient_display` writer that emits
the tag's bare id for `#tag` values and a composite of item displays
for single items and flat lists, used by the shaped/shapeless cells,
the furnace ingredient, and the transmute input/material (which had
the same latent bug; still no transmute tests). The placement info in
`entry_tail` still takes the holder set -- vanilla's `PlacementInfo`
is a list of HolderSets, so that side was already right. The three
failing tests that expected `SlotDisplay::Tag(HolderSet::Items(...))`
pinned the 26.3 form, which 1.21.7's wire has no equivalent of; their
expectations are now the composite form, and the NeoForge and
owned-content tests gained display assertions (the compound's two
items; one item as a composite of one). Known gap, unchanged by this
fix: vanilla wraps an item with a crafting remainder in
`WithRemainder`, which this writer cannot know without the items'
data. The `a_horizon_server_of_the_real_recipes` test (needs prepared
resources, skipped in CI) reads every entry back through the same
codec, so it now exercises the fixed writer on the real recipe set.
Crate-tests misc group: five `horizon-client-lib` failures, four test
bugs and one port defect (2026-11-22, crate-tests runs 4-6, identical
failure sets). Painting: `a_painting_keeps_its_variant_across_the_wire`
failed with "unsupported item component 109" -- but 109 is the 26.x id;
vanilla 1.21.7's `painting/variant` is id 89, jar-verified from
`DataComponents`' registration order (96 components, 0..=95), and the
codec already handled 89 (`Component::PaintingVariant`, the skip arm,
the `PaintingVariant` direct layout width/height/assetId/title?/author?
all match the jar). Test fixture corrected to 89 (registry sized to
the real 96); `skip.rs`' header comment corrected too -- its table is
1.21.7's order, not 1.21.10's, every arm checked against the jar. Sign:
`a_signs_text_crosses_the_wire` failed because the test's premise is
false for 1.21.7: `sign_text_front`/`sign_text_back` do not exist here
(no `sign_text` string anywhere in the 1.21.7 server jar, and
`SignBlockEntity` has no `collectImplicitComponents`). The test is
rewritten as `a_sign_crosses_the_wire_without_text_components` and pins
the 1.21.7 truth instead: text held in the saved form never reaches
the wire (the -1001/-1002 internal-marker kinds are filtered by
`write_patch`), the stack crosses as a plain sign (`[1, 1, 0, 0]`) and
comes back with no components. The dead sign-text arms in `guest.rs` /
`item_sync.rs` (keyed on those markers) are left in place. Projectiles:
`mod_thrown_types_are_found_by_id_and_item_once_registered` failed
because `lan::tracking::settings` lost the mod-thrown-type fallback in
the port -- the one real product fix of the group: a registered mod
thrown type takes its template snowball's settings (4, 10, true)
instead of the builder defaults (5, 3, true), restoring the 26.3 form
of `settings`. Realms:
`requests_send_the_session_cookie_only_and_errors_never_hold_the_token`
hardcoded 26.3 literals (cookie `version=26.3`, `Client-Ref: 26.3`,
protocol 777, `Java/25`); the code already sent the right 1.21.7 values
(1.21.7, 1.21.7, 772, `Java/21`), so the test's expectations were
corrected. Server packs:
`a_pack_lands_under_its_hash_and_is_reused_from_the_cache` hardcoded
`X-Minecraft-Pack-Format: 97.1` (26.3); the code sends 64.0 from
`RESOURCE_PACK_FORMAT`, correct for 1.21.7, so the test now expects
64.0. Verification: the `horizon-client-lib` test compile OOM-kills
on this VM even single-threaded, so no test in this group ran locally;
execution is owed to the CI crate-tests run after merge.
Save/title crate-test failures: four pre-existing failures, three
26.x carry-overs and one test bug (2026-11-22). All four failed
identically across crate-tests runs 4, 5 and 6, so they are
deterministic defects, not flakes.

`a_chunk_from_before_1_18_is_brought_to_the_current_layout`: the
upgraded 1.17.1 chunk had no `blending_data`. The 2832
`ChunkHeightAndBiomeFix` is vanilla-faithful — it blends only when
`__context.generator` is `minecraft:noise` (jar-verified) — but the
port's generator lookup, `datafix::files::generators`, read only
`data/minecraft/world_gen_settings.dat`, a file vanilla 1.21.7 never
writes (the string occurs in the 1.21.7 jar only in the datafix
`References` type; the file is the 26.x `LevelDatToSavedDataFileFix`
move at 4772, past this build's CURRENT), so the generator was None
for every 1.21.7 world and blending never happened. `generators` now
reads level.dat's `Data.WorldGenSettings` (the 1.21.7 location), the
file still winning per dimension when a 26.x client moved the
settings there, and a vanilla dimension with no recorded generator
gets vanilla's synthesized default — noise, except an overworld
whose level.dat says flat or debug (`generatorName`, the pre-1.16
field). This fixes the lazy-read path (`DatafixContext::of`) and the
world-upgrade path (`upgrade_world_data_with`) together.

`old_chunks_are_found_in_the_saved_records_and_old_poi_upgrade`: a
chunk this version saved was classified old. `record_is_old` used
the 26.3 base's threshold 4997, past this build's CURRENT (4438), so
every 1.21.7 chunk counted as old. Vanilla 1.21.7's
`IOWorker.isOldChunk` uses 4295 (jar-verified:
`getIntOr("DataVersion", 0) < 4295 || blending_data present`); the
threshold is now 4295.

`older_records_of_a_current_world_upgrade_as_they_are_read`: the
upgraded 1.20.1 palette had no `id` entry. The 1.20.3
grass-to-short_grass rename (key 3692) ran; what was missing was the
palette-key conversion `Name`/`Properties` to `id`/`properties`,
which the 26.3 base does at key 5006 — past this build's CURRENT, so
it never ran. The first fix was a port-adaptation fix at key 4438
(CURRENT) doing the same rename, last, after every fix that reads
`Name` — but that conflicted with the world group's fix (a 4435
chunk must keep `Name` through the upgrade, vanilla-faithfully), so
it was dropped on adjudication. The fix instead is a decoder
fallback: `BlockState::from_nbt` reads `id`/`properties` first and
falls back to `Name`/`Properties`, so upgraded palettes keep vanilla
1.21.7's keys (`StateHolder.codec`, jar-verified) and still decode.
That also closes the gap the adaptation left open: a chunk vanilla
1.21.7 itself saved (DataVersion 4438, `Name` palettes) skips the
upgrade entirely and used to decode as air. The test now asserts the
palette keeps `Name` = `minecraft:short_grass` with `id` absent, and
`chunk::tests::block_states_decode_from_either_key_form` pins the
decoder fallback. Note the writer still writes the 26.x
`id`/`properties` form (`BlockState::to_nbt`): the port reads both
forms but writes one, so a save the port writes is not one vanilla
1.21.7 reads — a diagnosed gap, not fixed here.

`a_finished_optimization_stamps_the_level_data`: a test bug, not a
product gap. The fixture set level.dat's DataVersion to 4950, a
26.x-era version that is in the future for 1.21.7 (CURRENT 4438),
which `fix_world_files` correctly leaves unstamped — vanilla never
downgrades a DataVersion. The fixture is now 3955 (1.21), genuinely
older than CURRENT and past the only file fix (1493), so the
optimization runs and `add_versions` stamps 4438. Regression tests:
the four named tests themselves (horizon-client-lib, CI execution
owed — the lib test compile OOM-kills on this VM) plus the new
`generators_come_from_level_dat_with_noise_defaults`
(horizon-world `datafix::files::upgrade_tests`: bare world defaults
to noise, a superflat level.dat yields flat, the 26.x file wins per
dimension).

Sheep drop no wool (2026-11-23). The owner killed 4-6 sheep and got no
wool -- and reported no mutton either. The wool half is a real defect
in `LootData::entity_matches` (horizon-world/loot.rs), the same defect
class as the block drops: it matched only 26.3's entity-sub-predicate
registry keys (`minecraft:flags`, `minecraft:components`,
`minecraft:type_specific/sheep`), while 1.21.7 data names an entity
predicate's parts bare (`flags`, `components`, `type_specific`, `type`,
`equipment`, `vehicle`) -- and `type_specific` carries its type inside
(`{"type": "minecraft:sheep", "sheared": false}`) instead of in the key.
Every bare key fell to the `_ => false` arm, so NO `entity_properties`
condition in any 1.21.7 entity table could hold (59 of them across the
102 entity tables, counted in the prepared data): the sheep wool pool
never matched, slimes never dropped balls (pool gated on slime size 1),
and nothing smelted on fire (the `furnace_smelt` gate reads `flags`).
The fix accepts both forms, dispatching 1.21.7's `type_specific` on its
inner type (sheep/slime/raider -- the only three in 1.21.7's loot data;
slime maps to the cube-size check). The mutton half did NOT reproduce:
rolling the real prepared `entities/sheep` table through
`roll_entity` yields mutton 1-2 before and after the fix, cow and
chicken drops work in-game (the owner's own screenshots show raw beef
and feathers), and the cow beef pool is structurally identical to the
sheep mutton pool -- so the mutton most likely did drop and went
unnoticed while he was hunting wool. The death path itself
(`drop_death_loot` -> `drops::die` -> `roll_entity` -> spawn) was
traced end to end and the `mob_drops` rule defaults true; if mutton is
still missing on the next playtest, the laptop log's "loot tables
unavailable" warning (a `LootData::load` failure makes `drop_death_loot`
return before any drop or XP orb) is the next thing to check. Shearing
is unaffected: the game rolls the per-colour `shearing/sheep/<color>`
table directly, bypassing the predicated parent table. Regression
tests (horizon-world `loot::entity_predicate_1_21_7_tests`): a sheep
drops its colour wool and mutton (sheared: mutton only; burning:
cooked mutton, no raw), a size-1 slime drops balls, a size-2 slime
drops none, a frog's kill drops exactly one ball, plus the real
prepared sheep table over four colours (skipped without prepared
data). Before the fix the sheep test's wool assertions all fail (the
diagnostic roll returned mutton-only). Adjacent, not fixed here:
horizon-client's advancement entity predicates
(`advancements.rs`) match the same 26.3 namespaced keys -- the same
defect class may be behind advancement predicates that never hold.
