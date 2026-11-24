# Batch 6 — Worldgen / features (26.1.2 parity sweep)

Date: 2026-11-25. Checks + triage only; no fixes, no code changes, no commits.
Worktree `~/workspace/mcir-work/sweep-26.1.2`, branch `parity-sweep-26.1.2` (read-only).
Vanilla reference: the named 26.1.2 jars
`~/workspace/mcir-work/26.1.x/versions/26.1.2/jars/{client,server-game}.jar`
(sha1-verified: client `4e618f09…`, server-game `83eb1106…`; never the
bundler `server.jar`). All behaviour claims verified against 26.1.2's OWN
jar/data, never the 26.3 base's form.

## §0 Execution status

Complete. All 25 26.1.1 items and all 10 1.21.7 items (4 behaviour + 6 oracle
groups) decided; no UNVERIFIED carry-overs. Static checks only: no cargo
builds, no driver runs, no commits, no git in the worktree. The reference
world was read in place with an own NBT parser; no world files copied or
moved; no git commands inside `vanilla-world-mc`.

Key context for every verdict below: the 26.1.1→26.1.2 jar delta touches NO
worldgen class (Batch 0 jar-diff: server-game.jar changed only
`DetectedVersion`, `SharedConstants` and
`ServerGamePacketListenerImpl`; client.jar those three plus `Checkbox`
×3 and `AbstractReportScreen` ×2). The 26.1.2 tree is a delta port of the
26.1.1 tree: `crates/horizon-world/src/worldgen/` is byte-identical to
26.1.1's, `crates/horizon-core/src/density/` is byte-identical,
`crates/horizon-prepare/src/lib.rs` is byte-identical, the oracle
sources and the `tests/` drivers are byte-identical, and the 24 crate
files that do differ are all outside B6 scope (horizon-client UI/LAN,
render datapack, `version.rs`, protocol ids/items/packets, `chunk.rs`,
datafix). The Batch 0 oracle table is byte-identical to 26.1.1's
(391 files, 0 differing entries) and the driver map is identical
(648 drivers, same values). The jar's `worldgen/` data directory is
byte-identical to 26.1.1's (`diff -rq`, no differences). So the 26.1.1
run's javap evidence for unchanged classes IS 26.1.2 evidence — and
every defect item below was still re-verified fresh against the 26.1.2
jar and the 26.1.2 tree (fresh javap dumps, fresh data diffs, fresh
decode audit, fresh census). The 26.1.2 delta's two behaviour-code
items (#4 `handleAttack`, #6 Checkbox tooltip) are outside B6 scope.

## Tally (behaviour checks only)

**27 checks: 19 pass, 8 gap. Behaviour/non-behaviour split: 27 behaviour +
0 non-behaviour as-tallied** (oracle triage is counted separately, not
tallied — see below).

| Group | Checks | Pass | Gap |
|---|---|---|---|
| 1.21.7 defect re-checks | 4 | 2 (G02, G04) | 2 (G01, G03) |
| Data diffs (prepared vs jar) | 6 | 5 | 1 (carver/feature bundles) |
| Code-vs-jar | 12 | 7 | 5 (sculk, spawn finder, generator construction, decode-coverage, numeric precision) |
| Reference-world census | 5 | 5 | 0 |
| **Total** | **27** | **19** | **8** |

Oracle triage is counted separately (per the 26.1 tally convention):
**57 oracles** (55 B6-referenced + 2 group-only), **18 pass, 39 fail**;
**64 drivers: 9 runnable, 17 no-oracle, 38 blocked** (see Oracle defects).

Defect items: **17 G items** — 15 gap (11 major, 4 minor), 2 fixed
(G02 major, G04 minor). All 15 gaps are 26.1.1 reproductions; no NEW
26.1.2 defects were found in scope. Withdrawn 1.21.7 defects: **2**
(G02, G04). Oracle items: **7** (B6-O01…O07). Needs-harness: **1**
(B6-H). Total items: **25**.

Headline: the 26.1.2 worldgen port is the 26.1.1 port, verbatim — every
live 26.1.1 defect in scope reproduces with fresh 26.1.2 evidence, and
the two withdrawn defects stay withdrawn. **No 26.1.2 world can generate
at all with the current tree + prepared data** — `GeneratorData::load`
fails on the empty carver bundle (G06), and behind it `Pipeline::new`
fails on the router (G10), the density caches (G13), the material rule
(G11) and the spawn target (G12), in that order; even with all of those
fixed, aquifers would be silently absent (G09), 126 of 221 configured
features would fail at decoration (G14∪G15), and the density pipeline
would run in f32 where vanilla computes in f64 (G17).

## Defects

### B6-G01 (major, 1.21.7 defect — REPRODUCES) — sculk_patch catalyst / extra rare growths tail missing

`versions/26.1.2/crates/horizon-world/src/worldgen/sculk.rs` (727 lines,
byte-identical to the 26.1.1 file) has no `catalyst_chance`/
`extra_rare_growths` decode and no catalyst handling: `decode` reads
only charge_count, amount_per_charge, spread_attempts, growth_rounds,
spread_rounds; `place()` returns `Ok(true)` at line 163 immediately
after the spreader rounds; the string `catalyst` appears nowhere in the
file.

Vanilla 26.1.2 (`SculkPatchFeature.place`, fresh javap on the 26.1.2
server-game jar, `evidence/javap-sculkpatch.txt`, byte-identical to the
26.1.1 dump): after the spreader rounds, `nextFloat() <= catalystChance`
with a collision-full block below places `SCULK_CATALYST` at the origin
(flag 3), then `extraRareGrowths.sample` × `origin.offset(nextInt(5)-2, 0,
nextInt(5)-2)` with air above and a face-sturdy block below places
`SCULK_SHRIEKER[can_summon=true]` (flag 3). The catalyst roll happens
BEFORE the growth sample, so the missing tail also shifts the RNG stream
for every feature placed after a sculk_patch in the same chunk.

26.1.2 data: `sculk_patch_deep_dark` catalyst_chance 0.5,
extra_rare_growths 0; `sculk_patch_ancient_city` 0.5, uniform 1..3. The
26.2 tree HAS the tail; the 26.1, 26.1.1 and 26.1.2 trees do not.
Consequence: no catalysts/shriekers from sculk_patch in 26.1.2.
Vanilla-side ground truth from the reference world: 364 sculk_catalyst
blocks, 179 sculk_shrieker blocks (deep_dark present, 228,190 biome
cells).

### B6-G03 (major, 1.21.7 defect — REPRODUCES) — underground water deficit; the entire aquifer system is absent

The 1.21.7 world comparison measured water −1.04% underground with surface
water matching; never root-caused. For 26.1.2 the mechanism is
demonstrated, and it is total: **the port never constructs a noise-based
aquifer** (see B6-G09). `Pipeline::new` reads the 26.3 `aquifers` config
object from the noise settings; no 26.1.2 noise settings has that key, so
the aquifer is `None` and generation uses `DisabledAquifer` — sea-level
picker water only, no underground lakes, no flooded caves, no aquifer
lava. The 1.21.7 deficit (−1.04%) would be far larger in 26.1.2: ALL
aquifer water is missing.

The full aquifer audit (same as 26.1.1's) finds the ENGINE faithful:
`aquifer/noise_based.rs` is byte-identical to the 26.1.1 file; the ctor
reads barrier / fluidLevelFloodedness / fluidLevelSpread / lava /
erosion / depth from the router (matching the jar ctor, which takes the
`NoiseRouter` — `javap-aquifer.txt:85-102`), the 13 `SURFACE_OFFSETS`
match the jar's `SURFACE_SAMPLING_OFFSETS_IN_CHUNKS`,
`computeSurfaceLevel` uses `OverworldBiomeBuilder.isDeepDarkRegion`
with the same clampedMap / thresholds, `FLOWING_UPDATE_SIMILARITY =
1.0 − 44/25` (`noise_based.rs:13-14`). The defect is purely the wiring:
the engine is never built. Vanilla-side water: 3,666,146 blocks over
7,074 chunks in the reference world.

### B6-G05 (minor, 1.21.7 defect — REPRODUCES) — sculk spreader water gates narrowed to water sources

Two sites in `worldgen/sculk.rs` require `WATER_SOURCE` where the 26.1.2
jar tests fluid-is-water:
- `sculk.rs:305` (`attempt_spread_vein`, Default branch, non-empty facings);
- `sculk.rs:548` (`can_place_growth` above-open gate).

Vanilla 26.1.2: `SculkBehaviour$1.attemptSpreadVein` regrows when the
state is air OR `fluidState.is(Fluids.WATER)` (fresh javap, identical
to the 26.1.1 dump, `javap-sculkbehaviour1.txt`, `getFluidState` +
`Fluids.WATER` at bytecode 57–60);
`SculkBlock.canPlaceGrowth` accepts above air OR (is(WATER) AND
fluidState.is(WATER)) (`javap-sculkblock.txt:185-242`). The port's
`WATER_SOURCE` is set only for source water (`tree_blocks.rs:183-184`;
flowing water gets `WATER_FLUID|ANY_FLUID` only), so flowing water blocks
sculk vein regrowth and growth placement that vanilla allows. No RNG
effect (no random calls in either gate). Same defect as in 26.1, 26.1.1
and 26.2. Note: the live-gameplay `level/sculk.rs:611` has the same
narrowing (`b.fluid(state_above).is_source()`) — flagged for B4, not
minted here.

### B6-G06 (major — REPRODUCES) — prepared `carvers.json` and `features.json` are EMPTY; `GeneratorData::load` fails

Prepared `worldgen/carvers.json` (0 entries), `worldgen/features.json`
(0 entries) and `worldgen/feature_tags.json` (empty). Cause:
`horizon-prepare` `WORLDGEN_BUNDLES`
(`crates/horizon-prepare/src/lib.rs:1463-1483`, byte-identical to the
26.1.1 file) maps `("carvers","worldgen/carver")`,
`("features","worldgen/feature")`,
`("feature_tags","tags/worldgen/feature")` — the 26.3 folder names. The
26.1.2 jar uses `worldgen/configured_carver` (4: canyon, cave,
cave_extra_underground, nether_cave), `worldgen/configured_feature`
(221), `tags/worldgen/configured_feature` (1 file:
`can_spawn_from_bone_meal.json`). There is no `worldgen/carver`,
`worldgen/feature` or `tags/worldgen/feature` folder in the jar at all.

The raw copies under prepared `data/minecraft/worldgen/` are 4/4 and
221/221 identical to the jar, but the generator does NOT read them:
`GeneratorData::load` (`generator.rs:486,497`) reads the `worldgen/*.json`
bundles only. Runtime effect: `CarverCatalog::from_json({})` succeeds,
then `decode_set` errors `unknown carver` for 59 of 65 biomes (54 list
`[cave, cave_extra_underground, canyon]`, 5 nether biomes use
single-string `minecraft:nether_cave`, 6 have empty lists) with `?`, so
**`GeneratorData::load` fails and no 26.1.2 world can generate with this
prepared data**; separately `FeatureCatalog::named` errors `unknown
feature reference` for every placed feature. This is the FIRST failure in
the construction cascade.

### B6-G07 (minor — REPRODUCES) — sculk `can_place_growth` density cap is dead (26.3-only tag)

Port `worldgen/sculk.rs:66,108` reads tag
`minecraft:sculk_growth_inhibitors`, which does NOT exist in 26.1.2 —
26.3 added it; absent from both jars (the jar's block tags carry only
`sculk_replaceable` and `sculk_replaceable_world_gen`) and from prepared
`block_tags.json`. `tag()` returns an empty set, so the ≥3 count at
`sculk.rs:556-558` never fires and sensors/shriekers pile up without
limit. Vanilla 26.1.2 `SculkBlock.canPlaceGrowth` counts
`SCULK_SENSOR`/`SCULK_SHRIEKER` blocks directly in the −4..+4 × 0..+2 ×
−4..+4 box (`javap-sculkblock.txt:103-125`). Both the 26.2 worldgen
sculk.rs and the 26.1 `level/sculk.rs` (lines 620–623, with an explicit
comment) check sensor/shrieker directly. Note: a wrongly-passing gate
also consumes RNG (`nextInt(50)`, `nextInt(11)`) and returns charge−50
instead of the decay path. Adjacent: the 26.1.2 `level/sculk.rs:620`
reads the same absent tag in live gameplay (the 26.1 tree fixed it
there too) — flagged for B4, not minted here.

### B6-G08 (minor — REPRODUCES) — sculk cursor movement uses the 26.3 form

Port `update()` (`sculk.rs:263-270`) kills a stuck cursor (None →
on_discharged + charge=0 + return) and has NO post-move 15-block check;
`valid_movement` (`sculk.rs:642,659-663`) applies a pre-move 12-block
filter (dx²+dz² > 144). The 26.1.2 jar `ChargeCursor.update`: a null
movement pos falls through to facings/decay-delay/update-delay (NOT
killed); the post-move worldgen check kills the cursor when NOT
`closerThan(origin-x/z, 15.0)` (`javap-chargecursor.txt:233-234`,
`ldc 15.0d` + `BlockPos.closerThan`); `getValidMovementPos` takes no
origin (`LevelAccessor, BlockPos, RandomSource` only) and has no
12-block filter. The 26.2 tree has the post-move dx²+dz² >= 225 check
and no pre-filter (with a comment stating 26.3 checks 12 before the move
and kills stuck cursors).

### B6-G09 (major — REPRODUCES) — aquifers are entirely disabled (26.3 config-object form)

`Pipeline::new` (`generator.rs:1115-1119`) reads
`settings_json["aquifers"]` — a 26.3 config-object key — and builds the
aquifer config from it. NO 26.1.2 noise settings has that key (all 7
checked: overworld, amplified and large_biomes have
`aquifers_enabled: true`; caves, end, floating_islands, nether have
false; none has `aquifers`). So the aquifer is always `None` and
generation uses `DisabledAquifer`: density ≤ 0 → the global fluid picker
only (sea-level water, lava below the picker threshold), no barrier /
floodedness / spread noise, no underground lakes or flooded caves.

Vanilla 26.1.2 builds `NoiseBasedAquifer` from the ROUTER (jar ctor takes
`NoiseRouter` and reads barrierNoise, fluidLevelFloodednessNoise,
fluidLevelSpreadNoise, lavaNoise, erosion, depth —
`javap-aquifer.txt:85-102`). The 26.2 tree fixed exactly this
(`from_router` + `aquifers_enabled` gate, with
`synthesize::aquifer_exclusion`); the 26.1.2 tree has neither
`from_router` nor any `aquifer_exclusion` synthesis. This is the
mechanism behind B6-G03. Silent: no error is raised, the aquifer is just
absent.

### B6-G10 (major — REPRODUCES) — NoiseRouter is the 26.3 eight-field form; `Pipeline::new` fails on 26.1.2 settings

`worldgen/router.rs` declares 8 fields (temperature, vegetation,
continents, erosion, depth, ridges, `chunk_surface_level`,
final_density) and `from_json` requires all of them. The 26.1.2 jar's
`NoiseRouter` record has 15 components (barrier,
fluid_level_floodedness, fluid_level_spread, lava,
`preliminary_surface_level`, vein_toggle, vein_ridged, vein_gap, plus
the eight) — there is NO `chunk_surface_level` in 26.1.2 (fresh javap
on the 26.1.2 server-game jar, `javap-noiserouter.txt`: 15 accessor
methods). All 7 prepared noise settings lack that key, so
`NoiseRouter::from_json` errors at `generator.rs:1093` for every
settings object, before the aquifer, rule and spawn-target steps.
`OverworldGenerator::for_world_with_settings` (the real
world-construction path) and the flat path both go through
`Pipeline::new`, so no noise or flat world can be constructed. The 26.2
tree has the correct 15-field form (`router.rs`, `ALL: [Self; 15]`,
`PreliminarySurfaceLevel`). Second failure in the construction cascade
(after G06).

### B6-G11 (major — REPRODUCES) — `Pipeline::new` requires `material_rule`; 26.1.2 settings carry an inline `surface_rule`

`generator.rs:1136-1138`: `settings_json["material_rule"].as_str()` is
required, and the name is looked up in the material-rule library. No
26.1.2 noise settings has a `material_rule` key — they carry an inline
`surface_rule` object (`{type: sequence, ...}` — `end` uses
`{type: block}`), the 1.21.x/26.2 form, and `rules.json`/
`conditions.json` are legitimately empty (no such data folders exist in
the 26.1.2 jar; SurfaceRules are inline in the settings). The 26.2 tree
feeds the inline rule to the library under a synthetic name
(`INLINE_SURFACE_RULE`, `generator.rs` surface_rule handling). Third
failure in the construction cascade (after G06, G10 — and after G13,
which fires during the router decode's density resolution).

### B6-G12 (major — REPRODUCES) — spawn_target decode fails (named-density lookups); depth not zeroed

`generator.rs:1141-1164` decodes each `spawn_target` point by mapping
every field name through `library.named_density(name)` — i.e. looking up
a density function named `minecraft:<field>`. None of the 7 field names
used by the 26.1.2 overworld/amplified/large_biomes spawn targets
(continentalness, depth, erosion, humidity, offset, temperature,
weirdness) is a registered density function (the 35 registry entries are
all `minecraft:overworld/...`, `minecraft:end/...`,
`minecraft:nether/...`, `minecraft:shift_x`, `minecraft:shift_z`,
`minecraft:y`, `minecraft:zero`), so the decode errors with `unknown
material density` for every settings object with a non-empty
spawn_target.

Latent second half: even if the lookups resolved, the port's
`SpawnTargetPoint::sample_fitness` samples each named density, but
vanilla's `Climate$SpawnFinder.getSpawnPositionAndFitness` rebuilds the
sampled `TargetPoint` with depth FORCED to 0 (fresh javap:
`lconst_0` as the depth argument, `javap-spawnfinder.txt:124`,
bytecode 39) and adds `square(offset)` as a constant — it never samples
a depth or offset function. The 26.1.2 overworld depth function at y=0
is ~1.0 + the offset spline (NOT 0), so sampling it would add a large
position-varying bias to every candidate's fitness and distort the spawn
search. The 26.2 tree decodes spawn_target via
`decode_parameter_point` (six climate parameters + offset constant) and
its spawn_finder zeroes the depth. The radial-search / fitness FORMULA
in the port matches the jar exactly (verified — see Verified passes);
only the decode and the depth handling are wrong.

### B6-G13 (major — REPRODUCES) — density decode missing seven types

`worldgen/density.rs` has no arms for `cache_2d`, `cache_once`,
`flat_cache`, `shifted_noise`, `y_clamped_gradient`,
`weird_scaled_sampler`, `end_islands`; the default arm errors `unknown
density type: {kind}` (`density.rs:458-462`). Recursive census of all 35
density-function files: cache_2d ×11, cache_once ×12, flat_cache ×16,
shifted_noise ×5, y_clamped_gradient ×6, weird_scaled_sampler ×3,
end_islands ×1 — plus surface-rule uses (cache_2d ×13, shifted_noise ×8,
y_clamped_gradient ×23 across all 7 settings) and `end.json`'s
`end_islands`. Counts are identical to 26.1.1's (the jar data is
byte-identical).

- `minecraft:overworld/offset` is `flat_cache` > `cache_2d` > `add(...)`
  (with `cache_once` around `blend_alpha` inside) — and `offset` is an
  argument of `minecraft:overworld/depth`, a router field. Decoding the
  router's depth function therefore fails.
- `minecraft:y` IS `y_clamped_gradient` (from_y −4064, to_y 4062) — the
  base Y function itself cannot decode. (Correction to the 26.1.1
  report, which wrote from_y −64 / to_y 320: the data in both versions
  is −4064 / 4062; verified fresh from the 26.1.2 jar.)
- `weird_scaled_sampler` ×3 in `overworld/caves/entrances` and
  `spaghetti_2d` — the overworld spaghetti caves cannot decode; the
  26.1.2 horizon-core has no `WeirdScaledSampler` at all (the 26.1 core
  has it, `graph/mod.rs:127` + `compiler.rs:220`).
- `end_islands` ×1 in `end/sloped_cheese` (+ `end.json`) — the End's
  density cannot decode. The 26.1.2 tree has only the 26.3 name
  `end_outer_islands` (outer-only); the 26.1 tree has both arms
  (`EndIslands { central: true/false }`).
- The 26.1.2 tree's `minecraft:gradient` arm (axis-based, 26.3 form) is
  dead code: no `minecraft:gradient` type occurs anywhere in the
  26.1.2 worldgen data.

The 26.2 tree has the cache/shifted-noise arms; the 26.1 tree has all
seven (y_clamped_gradient `density.rs:277`, weird_scaled_sampler
`density.rs:326-340`, end_islands `density.rs:268-269`). The 26.1.2
compiler even has a `ShiftedNoiseSampler`
(`horizon-core/src/density/mod.rs:437`) that the JSON decoder never
produces. Fires during the router decode in `Pipeline::new` (after
G10's missing-field error, before G11).

### B6-G14 (major — REPRODUCES) — 11 configured-feature types + `coral_mushroom` + `nether_cave` have no decode arms

Top-level configured-feature types in the 26.1.2 data: 53 distinct, 221
total. Eleven have NO arm in the port's feature dispatch
(`feature_graph.rs`, byte-identical to the 26.1.1 file, plus `feature.rs`
for the end features) — the default arm falls through to
`BasicFeature::from_json`, which errors `basic feature type` for
anything but the three end features:

| Missing type | Configured features | 26.1 tree |
|---|---|---|
| `basalt_columns` | 2 | also missing |
| `basalt_pillar` | 1 | also missing |
| `desert_well` | 1 | also missing |
| `dripstone_cluster` | 1 | also missing (26.3 name `speleothem_cluster` present instead) |
| `glowstone_blob` | 1 | also missing |
| `kelp` | 1 | also missing |
| `nether_forest_vegetation` | 6 | also missing |
| `sea_pickle` | 1 | also missing |
| `seagrass` | 4 | also missing |
| `twisting_vines` | 2 | also missing |
| `weeping_vines` | 1 | also missing |

Plus `coral_mushroom`, nested inside `warm_ocean_vegetation`'s
simple_random_selector (dispatch has coral_tree|coral_claw only;
`CoralFeature::decode` errors on other kinds). Total: **22 broken
configured features, affecting 24 placed features** (basalt_pillar,
crimson/warped forest vegetation, desert_well, dripstone_cluster,
glowstone, glowstone_extra, kelp_cold/warm, large/small basalt columns,
nether_sprouts, sea_pickle, seagrass ×8, twisting_vines,
warm_ocean_vegetation, weeping_vines — verified by name against the
26.1.2 placed-feature data). Feature decoding is lazy
(`FeatureCatalog::named`), so these fail at decoration time when a chunk
references one of the 24 placed features — a hard error during
decoration, not at load. (Latent behind G06: `features.json` is empty,
so today every placed feature fails earlier with `unknown feature
reference`.)

Also missing: the `nether_cave` carver arm (`carver/generation.rs` has
cave + canyon only, byte-identical to 26.1.1; 26.2 has `cave |
nether_cave`). The 5 nether biomes list `minecraft:nether_cave` as
their carver, so fixing G06 alone would surface this next for the
nether.

Overlap with G15: 6 of the 22 also use long-form state providers
(crimson/warped forest vegetation + nether_sprouts, incl. bonemeal
variants); 16 fail on the feature type alone. Union G14∪G15: 126 of
221 configured features fail to decode.

Present and verified (see Verified passes): 32 of 33 placement-modifier
types (only `random_offset` missing — G16), all 8 int-provider and 6
height-provider types, all 9 trunk / 11 foliage / 1 root placers, all 9
tree decorators, all 5 pool element types, all 4 processor types used,
both structure-set placements.

### B6-G15 (major — REPRODUCES) — state-provider decode accepts only the 26.3 short names; 110 configured features fail

`worldgen/state_provider.rs:353-456` (`decode_typed`) matches only the
26.3 short provider names: `minecraft:rule_based`, `minecraft:noise` |
`noise_threshold` | `dual_noise`, `minecraft:simple`,
`minecraft:weighted`, `minecraft:random_block`, `minecraft:rotated`,
`minecraft:copy_properties`, `minecraft:randomized_int`. The 26.1.2
data uses ONLY the long 26.1 names — 287 uses, zero short-form uses:

| Long name (data) | Uses | Short arm in 26.1.2 tree |
|---|---|---|
| `simple_state_provider` | 197 | `minecraft:simple` only |
| `rule_based_state_provider` | 42 | `minecraft:rule_based` only |
| `weighted_state_provider` | 40 | `minecraft:weighted` only |
| `randomized_int_state_provider` | 4 | `minecraft:randomized_int` only |
| `noise_provider` | 1 | `minecraft:noise` only |
| `dual_noise_provider` | 1 | `minecraft:dual_noise` only |
| `noise_threshold_provider` | 1 | `minecraft:noise_threshold` only |
| `rotated_block_provider` | 1 | `minecraft:rotated` only |

**110 of 221 configured features** contain at least one long-form
provider and fail decode with `block state provider type`. There is no
name normalisation anywhere on the path (`identifier()` only validates
the namespace; the 26.1 tree needed explicit alternates, proving it).
The 26.1 tree accepts both spellings for every provider
(`state_provider.rs:357,393-398,401,406,415,447`,
`state_provider/noise.rs:45-55`); the 26.1.2 tree has only the short
ones. The decode audit's "ok" for `simple_state_provider` /
`weighted_state_provider` was a false positive again: those strings occur
only in `structure/processor/litho.rs` (a structure-processor local
match), not on the feature decode path. The same grep false-positive
class shows in `seagrass` (string present in worldgen sources, no
dispatch arm — counted in G14) and in the structure audit's
`magma_cube`/`wither_skeleton`/`zombified_piglin` "MISSING" hits, which
are entity-type strings inside `fortress.json` spawn overrides, not
missing structure decode arms. The 26.1 tree is the repair template
(add the long names as alternates).

### B6-G16 (major — REPRODUCES) — `random_offset` placement modifier has no decode arm; 70 placed features fail

`worldgen/placement.rs` (and the whole `horizon-world` tree — zero
occurrences of `random_offset` / `RandomOffset`) has no arm for
`minecraft:random_offset`; the default arm errors `placement modifier
type`. The 26.1.2 data uses it **77 times**: 70 placed features
(every `patch_*` vegetation patch, mushroom patches, flower patches,
`pointed_dripstone`, `spore_blossom`, `rooted_azalea_tree`,
`wildflowers_*`, `ice_patch`, `lush_caves_*`, `cave_vines`,
`end_gateway_return`, `disk_grass`, `patch_soul_fire`, `patch_fire`,
`patch_waterlily`, `patch_melon`/`patch_pumpkin`, `patch_cactus_*`,
`patch_sugar_cane_*`, `patch_dead_bush_*`, `patch_dry_grass_*`,
`patch_grass_*`, `patch_tall_grass*`, `patch_taiga_grass`,
`patch_sunflower`, `patch_berry_*`, `patch_bush`, `patch_leaf_litter`,
`patch_large_fern`, `brown/red_mushroom_*`) plus 7 uses nested in
configured features. The 26.1 tree HAS the arm (`placement.rs`); the
26.1.2 tree does not. 32 of the 33 placement-modifier types used in the
data are present; this is the only missing one — but it is the
most-used modifier after `count`/`in_square`, and it gates nearly all
surface vegetation decoration.

### B6-G17 (minor — REPRODUCES) — worldgen numeric pipeline is f32 where vanilla 26.1.2 computes in f64

Vanilla 26.1.2 density is double throughout:
`DensityFunction.compute` returns `double`, `fillArray(double[], ...)`
(fresh javap on the 26.1.2 server-game jar,
`javap-densityfunction.txt:9-11`); `Biome`'s three noises are
`PerlinSimplexNoise` with `getValue(DDZ)D`. The 26.1.2 port computes the
same pipeline in f32 at four sites:

- **Density graph**: `horizon-core/src/density/graph/mod.rs` —
  `GraphNode::Constant(f32)`, `Gradient { low/high: f32 }`, `Clamp/
  Choice { min/max: f32 }`; all samplers return `Result<f32,
  SampleError>` (`arithmetic.rs`, `beardifier.rs:237`,
  `end_islands.rs:116`, `cache.rs:88`). The 26.1 core is f64 throughout
  (`Constant(f64)`, samplers return f64).
- **Climate fields**: `worldgen/climate.rs` — `from_values([f32; 6])`,
  `fields: [Vec<f32>; 6]`. Vanilla samples the six climate fields in
  double, then casts each to float for `Climate.quantizeCoord(float)`
  (26.1 tree: `from_values([f64; 6])`, `values.map(|v| quantize(v as
  f32))`).
- **Blending**: `worldgen/blending.rs` — `NO_VALUE: f32 = f32::MAX`,
  `heights: [f64→f32; ...]`, `densities: [Option<Vec<f32>>; ...]`.
  Vanilla `BlendingData.NO_VALUE` is `double` (fresh javap on the
  26.1.2 jar: `protected static final double NO_VALUE`; 26.1 tree:
  `NO_VALUE: f64 = f64::MAX`). Note the port's own comment at
  `blending.rs:21` cites `Float.MAX_VALUE` — the comment is wrong for
  26.1.2, not just the code.
- **Biome temperature noises**: `worldgen/biome_temperature.rs` uses
  f32 `SimplexNoise` / `NoiseStack` (seeds 1234/3456/2345, same
  structure as vanilla) where vanilla uses double
  `PerlinSimplexNoise`; the 26.1.2 horizon-core has no
  `PerlinSimplexNoise` at all (the 26.1 tree added it). The 26.1
  tree's comment notes the f32 engine flips
  `NoiseBasedCountPlacement` counts at extreme ratios
  (`ceil((noise + offset) * ratio)`).

Effect: rounding-level divergence in every density sample, climate
quantisation, blend height and frozen-biome temperature. Magnitude is
unmeasured statically — a density value within ~1e-7 of zero can flip
sign and move a surface by a block; over millions of columns some
blocks will differ. B6-H arbitrates the magnitude. The 26.1 tree is the
repair template for all four sites. Severity minor: systematic but
small per-sample; the decode failures (G06/G13/G15/G16) dominate
26.1.2 worldgen fidelity today.

## Withdrawn 1.21.7 defects (verified absent in 26.1.2)

### B6-G02 (major) — WITHDRAWN: snow-on-packed-ice mechanism verified closed

Mechanism root-caused and verified closed, with the same evidence as
the 26.1.1 withdrawal, re-verified fresh:
- The jar tag `cannot_support_snow_layer` = [ice, packed_ice, barrier]
  and `support_override_snow_layer` = [honey_block, soul_sand, mud];
  the prepared `block_tags.json` contains both with exactly the jar's
  members.
- The port's `Rule::SnowLayer` (`block_survival.rs:793`, byte-identical
  to the 26.1.1 file) and `survive.rs:293` (`Class::SnowLayer`; the file
  is byte-identical to 26.1.1's) both implement deny-tag →
  allow-tag → face-full collision UP or same-block layers==8, matching
  jar `SnowLayerBlock.canSurvive` exactly (`javap-snowlayer.txt:120-167`,
  fresh javap on the 26.1.2 jar, byte-identical to the 26.1.1 dump);
  both survival paths load the tags from prepared data, not hardcoded.
- `SnowAndFreezeFeature.place` matches the jar exactly (16×16,
  MOTION_BLOCKING height, biome at top, `shouldFreeze(below, false)` →
  ice flag 2, `shouldSnow(top)` → snow flag 2 + snowy=true below);
  `snow_freeze.rs` and `biome_weather.rs` are byte-identical to the
  26.1.1 files, and the jar's `Biome.shouldFreeze`/`shouldSnow`/
  `coldEnoughToSnow` are unchanged from 26.1.1 (jar delta touches no
  worldgen class).
- **Vanilla-side ground truth from the reference world: 0 snow blocks
  on packed ice world-wide** — 0 of 31,772 snow blocks in full chunks
  (same-section below-check) plus 0 of 2,190 snow blocks at section
  bottoms (cross-section check), and 0 of 8,066 snow blocks in
  frozen_peaks biome quarts. Like the 26.1.1 world, the 26.1.2 world
  HAS frozen_peaks (56,828 cells) and deep_dark, so this is a live
  negative, not an absence of test terrain.

Caveat: no port world exists, so the port's live snow placement cannot
be re-measured; live confirmation is B6-H.

### B6-G04 (minor) — WITHDRAWN: PoolPiece NBT writes `GD`

`PoolPiece::save` (`worldgen/structure/jigsaw.rs:683`, byte-identical to
the 26.1.1 file) goes through
`legacy::piece_tag("minecraft:jigsaw", bbox, None, 0)`, which writes
`("GD", Tag::Int(gen_depth))` (`legacy.rs:339`, byte-identical). The
reference world confirms the vanilla form: trial chambers 1,691/1,691
pieces GD=0, trail ruins 19/19 GD=0, village_plains 103/103 GD=0,
ancient_city 172/172 GD=0, mineshaft GD 0–9.

## Oracle defects (Batch 0 measured status, 57 oracles, 18 pass / 39 fail)

Authority: `docs/sweep/26.1.2/batch-0-oracles.json` (byte-identical to the
26.1.1 table — 391 files, 0 differing entries; 189/391 oracles compile).
B6-referenced oracles: 55 (37 fail on all 3 classpaths, 18 pass on ≥1),
plus 2 group-only failing oracles swept from Batch 0 groups
(`VanillaFloatLogOracle`, `VanillaStructurePlacementOracle`). Drivers:
64 total (9 runnable, 17 no-oracle, 38 blocked) — driver list in
`evidence/drivers-26.1.2.json`. The driver map is identical to 26.1.1's
(648 drivers, same values); the oracle sources are byte-identical to
26.1.1's. No change vs 26.1.1 in any group.

Runnable drivers (9): biome_temperature, chunk_generation, chunk_status,
gateway_replacement, heightmap, placement, retrogen_data,
world_spawn_sync, worldgen_random. No-oracle (17): biome_weather,
chunk_restart_ticks, datapack_worldgen, end_gateway, end_podium,
flat_generation, flat_place, folder_datapack_metadata,
inline_feature_parse, place_feature_inline, sculk_saved_fields,
server_player_sculk, spawn_loot, spring, structure_blocks,
structure_layout, unused_feature_startup. Blocked (38): basic_feature,
biome_chunk, biome_generation, biome_placement, biome_source,
biome_source_codec, biome_zoom, canyon_carver, cave_carver, chunk_stage,
cushion_placement, decoration, decoration_region, density,
feature_catalog, feature_graph, feature_placement, feature_region,
feature_sort, finalize_spawn, height_provider, material_condition,
material_extension, material_rule_library, material_stage,
material_surface, natural_spawn, noise, retrogen_chunk, scattered_ore,
sculk, spawn_rules, structure_density, structure_start_nbt, terrain,
tree_feature, world_spawn, worldgen.

Passing oracles (18): ParityData, VanillaBiomeExport,
VanillaBiomeTemperatureOracle, VanillaBlockPropertiesOracle,
VanillaBlockStateCodecOracle, VanillaBlockSupportOracle,
VanillaBlockSurvivalOracle, VanillaChunkGenerationOracle,
VanillaChunkStatusOracle, VanillaCollisionOracle,
VanillaGatewayReplacementOracle, VanillaHeightmapOracle,
VanillaPlacementOracle, VanillaRetrogenDataOracle,
VanillaSimpleBlockOracle, VanillaTagExport,
VanillaWorldSpawnPacketOracle, VanillaWorldgenRandomOracle.

### B6-O01 (major) — Oracles reference the 26.3-only `densityfunction` package (9 oracles)

VanillaAnchorOracle, VanillaBiomeSourceCodecOracle,
VanillaBiomeSourceOracle, VanillaDensityOracle,
VanillaFloatLogOracle*, VanillaMaterialExtensionOracle,
VanillaStructureDensityOracle, VanillaWorldSpawnOracle,
VanillaWorldgenOracle. Verified by jar listing: the `densityfunction`
package has **0 entries in BOTH 26.1.2 jars**;
`DensityFunction` lives at `levelgen.DensityFunction` (1.21.x shape).
`VanillaAnchorOracle` blocks 9 drivers including non-B6
`test_anchor_parity` and `test_state_provider_parity`. (*FloatLog is
group-only: its driver `test_float_log_parity` is not a B6 driver;
recorded here with a cross-ref.) `VanillaStructureDensityOracle` is
UNREPAIRABLE by design if it exercises the 26.3-only API (same ruling
as 1.21.7/26.2) — recorded, not forced. Grouping verified from the
oracle sources: VanillaWorldSpawnOracle imports
`densityfunction.SamplerContext` (line 25) and the absent
`NoiseSpawnFinder`; VanillaDensityOracle imports `densityfunction.*`,
`.generator.*`, `.op.*`; VanillaWorldgenOracle references
`densityfunction.DensityFunction` / `DensityFunctions` (lines 23, 38,
41). VanillaNoiseOracle stays in O04 (it imports only `synth` classes,
no `densityfunction` import).

### B6-O02 (major) — Oracles reference the missing `material/rule` package (5 oracles)

VanillaBiomeGenerationOracle, VanillaMaterialConditionOracle,
VanillaMaterialStageOracle, VanillaMaterialSurfaceOracle,
VanillaTerrainOracle. Verified: `material/rule` has 0 entries in both
jars (`material/` holds only `MaterialRuleList`); 26.1.2 surface rules
live at `levelgen.SurfaceRules`. First errors: BiomeGeneration /
MaterialStage / Terrain — the package itself;
MaterialCondition — `MaterialCondition`/`MaterialRule` symbols;
MaterialSurface — `MaterialSystem` (it also imports `densityfunction`,
recorded here per the 26.1 grouping).

### B6-O03 (major) — Oracles use the 26.3 `Feature` constructor signature (3 oracles)

VanillaDecorationOracle, VanillaBiomePlacementOracle,
VanillaFeaturePlacementOracle. The 26.1.2 jar's `Feature` ctor takes
only `Codec<FC>` (fresh javap, `javap-feature.txt:134`); `Feature` has
NO `DIRECT_CODEC`. All three oracles construct a raw `Feature()` with a
`MapCodec` and a 4-arg `place` — the 26.3 form. The 26.1 tree's
adapted oracles used `Feature<NoneFeatureConfiguration>` with
`FeaturePlaceContext`.

### B6-O04 (major) — Oracles fail on missing symbols / signature drift (12 oracles)

VanillaBasicFeatureOracle, VanillaBiomeChunkOracle,
VanillaCanyonCarverOracle, VanillaCaveCarverOracle,
VanillaChunkStageOracle, VanillaCushionPlacementOracle,
VanillaFeatureGraphOracle, VanillaFeatureRegionOracle,
VanillaFeatureSortOracle, VanillaNoiseOracle, VanillaScatteredOreOracle,
VanillaSculkOracle. First-error symbols from the raw
`big-compile-client.log` (identical to 26.1.1's):
BasicFeature/FeatureSort/ScatteredOre — `Feature.DIRECT_CODEC`
(absent; `ConfiguredFeature.DIRECT_CODEC` exists) and `FeaturePlacer`;
BiomeChunk — lambda parameter types + `ChunkAccess.fillBiomesFromNoise`
signature; CanyonCarver/CaveCarver — `WorldCarver.DIRECT_CODEC`
(absent; only `configuredCodec()`), `CarverOutput`,
`CarvingMask.minY/maxY/carve` (`CarvingMask` is at
`world.level.chunk.CarvingMask` with set/get/stream/toArray only);
ChunkStage/FeatureRegion — `ChunkStatus.TERRAIN` (the 26.1.2 set has
NOISE/SURFACE/CARVERS, no TERRAIN — fresh javap-verified);
CushionPlacement — the `blockscan` package; Noise —
`GradientNoise`/`Noise`/`NoiseStack` (synth has only
NormalNoise/PerlinNoise, plus BlendedNoise/ImprovedNoise/
PerlinSimplexNoise/SimplexNoise — verified by jar listing); Sculk —
`datapackWorldRegistries()` (absent; `datapackWorldgen()` exists),
MinecraftServer ctor, `WardenSpawnTracker` Optional wrapping. Also
absent from the jar: Interval, DensitySampler/SamplerContext,
NoiseSpawnFinder, MaterialCondition/MaterialRule/MaterialSystem,
Cushion, Continuation, BlockMatcher, NativeDestruction, cubemob,
`Registries.BLOCK_STATE_PROVIDER` (26.1.2 has
`BLOCK_STATE_PROVIDER_TYPE`).

### B6-O05 (major) — Oracles fail on missing-override lookup (2 oracles)

VanillaDecorationRegionOracle, VanillaFeatureCatalogOracle. The named
methods have no override in the 26.1.2 jar (`getNextEntityId()`,
`startsForStructure` overloads, `getAllStructuresAt`,
`getBiome(int,int,int)`, `markPosForPostProcessing` — note
`ProtoChunk.markPosForPostprocessing`, lowercase p).
FeatureCatalog's errors (11, same as 26.1.1) include a `Feature`-ctor
error and a missing `place(FeaturePlaceContext)` override (verified in
the 26.1.2 `big-compile-client.log`).

### B6-O06 (major) — Oracles fail on no-override (4 oracles)

VanillaFinalizeSpawnOracle, VanillaNaturalSpawnOracle,
VanillaSpawnRulesOracle, VanillaTreeFeatureOracle. Cross-batch overlaps
(coordinator dedups): FinalizeSpawn→B3, NaturalSpawn also referenced by
spawner/trial_spawner/vault drivers. TreeFeature's errors: 11 (same as
26.1.1).

### B6-O07 (major) — Other oracle failures (4 oracles)

VanillaBiomeZoomOracle, VanillaStructureStartOracle,
VanillaHeightProviderOracle (cascade via Anchor),
VanillaStructurePlacementOracle* (group-only, no driver references it).
BiomeZoom: `BiomeManager.getBiome` signature. StructureStart /
StructurePlacement: no suitable `create(Registry<NoiseParameters>,
long,NoiseGeneratorSettings)` method. WorldSpawn→B3, Sculk→B4 per the
26.2 numbering.

## Needs-harness

### B6-H (minor) — Paired port-vs-vanilla chunk comparison for 26.1.2

Generate port chunks at seed 4656297851632877335 (the reference world's
seed, confirmed from `data/minecraft/world_gen_settings.dat`) and
compare against the vanilla reference world
(`~/workspace/vanilla-world-mc/26.1.2/26_1_2/`, 7,074 chunks).
**STAGE RULE: the port's `generate_proto` output is terrain-level**
(biomes, noise fill, aquifers, material surface, carvers — the 26.1.2
statuses up to CARVERS); **vanilla region files are post-decorate**.
Compare post-decorate to post-decorate for blocks, terrain-level to
terrain-level for density. Direct comparison is only valid where
structure templates contain no air (ancient city: 0 air) — trial
chambers/villages carve air during placement, so a terrain-vs-
post-decorate gap there is a measurement artifact, not a density bug
(the 1.21.7 −6.4 'beardifier residual' was exactly this). B6-H
arbitrates: G02/G04 live confirmation, G03's magnitude once aquifers
exist, the sculk/G07/G08 RNG-stream effects, and the G17 f32 magnitude.
Blocked until the construction cascade (G06/G10/G13/G11/G12) is fixed —
no port world can be generated today.

## Verified passes

### Data (exact diffs vs the jar)

- **Populated registries 715/715 entries name-set- and
  content-identical** (exact JSON equality after parsing prepared
  JSON-encoded strings): biomes 65, densities 35, noises 62, settings
  7, placed_features 258, processor_lists 40, structures 34,
  structure_sets 20, template_pools 188, world_presets 6.
  `rules.json`/`conditions.json`/`state_providers.json` legitimately
  empty (no such folders in the jar — code-only in 26.1.2).
- **Structure NBTs 1202/1202 byte-identical** (jar
  `data/minecraft/structure/` vs prepared `worldgen/structure/minecraft/`).
- **Flat presets 9/9 identical**: bottomless_pit, classic_flat, desert,
  overworld, redstone_ready, snowy_kingdom, the_void, tunnelers_dream,
  water_world; `visible` tag lists all 9.
- **Nether multi-noise preset verified exactly** against 26.1.2 jar
  bytecode (`MultiNoiseBiomeSourceParameterList$Preset$1`,
  `Climate.parameters(t,h,c,e,d,w,o)` order — class unchanged from
  26.1.1, jar delta touches no worldgen class): nether_wastes all 0;
  soul_sand_valley humidity −0.5; crimson_forest temperature 0.4;
  warped_forest humidity 0.5, offset 0.375; basalt_deltas temperature
  −0.5, offset 0.175. Prepared `biome-parameters/nether.json` matches.
- **Overworld expansion: 7,593 entries** (same as 26.1.1; one fewer than
  26.2's 7,594 = no sulfur_caves), first mushroom_fields, last
  deep_dark, 54 distinct biomes, md5-identical to the 26.1.1 prepared
  file (`79a58ecdb5695a3c8b97f780131d320c`); both prepared copies
  (`biome-parameters/` and `worldgen/biome_parameters/`) identical.

### Vanilla 26.1.2 reference world (own NBT parser, read in place)

World at `~/workspace/vanilla-world-mc/26.1.2/26_1_2/`. Seed
**4656297851632877335** confirmed from
`data/minecraft/world_gen_settings.dat` (`data.seed`). Level name
"26.1.2", DataVersion 4790, spawn `{pos:[-16,65,-48],
dimension:'minecraft:overworld'}` (new lowercase `spawn` compound),
GameType 1, Time 1692, datapacks vanilla only
(minecart_improvements, redstone_experiments, trade_rebalance
disabled), bonus_chest 0, generate_structures 1, NO `WorldGenSettings`
in level.dat. Generators: overworld `minecraft:noise`, settings
`minecraft:overworld`, biome source multi_noise preset overworld;
nether noise/nether, multi_noise nether; end noise/end, `the_end`
biome source. No nether/end regions (dirs exist, only `data/`
subdirs, no region subdirs).

- **15 regions, 7,074 chunks** (3,589 `minecraft:full`, 2,604
  structure_starts, 294 biomes, 288 carvers, 287 initialize_light,
  8 noise, 4 surface), x −43..37, z −68..28. Statuses use the 26.1.2 set
  (noise/surface/carvers, no TERRAIN).
- **695,402,496 blocks counted** (= 7074×24×4096 exactly). Water
  3,666,146; snow 33,308; packed_ice 26,523. Top: air 547,044,001;
  stone 59,242,561; deepslate 54,457,079.
- **18 biomes present**, INCLUDING deep_dark (228,190 cells) and
  frozen_peaks (56,828) — unlike 26.2's world. Full list with cells:
  plains 5,489,864; forest 2,456,142; old_growth_birch_forest 766,270;
  river 668,547; birch_forest 512,589; deep_dark 228,190; cherry_grove
  200,831; grove 142,883; dark_forest 97,494; dripstone_caves 92,287;
  frozen_peaks 56,828; beach 52,992; meadow 35,577; stony_shore
  33,481; lush_caves 22,149; snowy_slopes 5,392; jagged_peaks 3,572;
  ocean 576.
- Sculk family: sculk 188,033; sculk_vein 68,677; sculk_sensor 1,533;
  **sculk_catalyst 364; sculk_shrieker 179** (vanilla-side ground truth
  for G01).
- Structure starts: ruined_portal 5/5, trial_chambers 7/1,691,
  ancient_city 2/172, mineshaft 22/2,815, trail_ruins 1/19,
  village_plains 1/103, shipwreck 1/1. GD: all jigsaw pieces GD=0
  (trial chambers 1691/1691, trail ruins 19/19, village_plains
  103/103, ancient_city 172/172); mineshaft GD 0–9. GLD:
  trial_chambers −54..+18; trail_ruins −12..+5; village_plains
  −1..+1; ancient_city −4..+1.
- Block entities: sculk_sensor 1,507, sculk_catalyst 354, chest 229,
  decorated_pot 224, sculk_shrieker 173, dispenser 119, mob_spawner
  117, vault 111, brushable_block 108, trial_spawner 92, barrel 86,
  hopper 51, **DUMMY 42** (vanilla-side oddity, adjacent note only —
  BEs owned by another batch), bed 40, beehive 36, comparator 17,
  skull 8, campfire 3, furnace 2, lectern 2.
- **Snow-on-packed-ice: 0 world-wide** (see G02).

### Code-vs-jar spot checks (javap + source)

- **Sculk spreader core — VERIFIED faithful** (the G01/G05/G07/G08
  defects are at the edges, not the core): `sculk.rs` is byte-identical
  to the 26.1.1 file, and the 26.1.2 jar's sculk classes are
  byte-identical to 26.1.1's (fresh javap dumps identical; jar delta
  touches no worldgen class), so the 26.1.1 verification carries in
  full: `createWorldGenSpreader` args (true, SCULK_REPLACEABLE_WORLD_GEN,
  50, 1, 5, 10) match port constants; `can_spread_into` matches
  `SculkVeinSpreaderConfig.stateCanBeReplaced`; `SculkBlock.
  attemptUseCharge` + `getDecayPenalty` match the port's Sculk branch
  exactly; Default attemptUseCharge; Vein branch; `updateCursors`;
  `getValidMovementPos` (shuffled 18 non-corner neighbours);
  `isMovementUnobstructed`; only SculkBlock and SculkVeinBlock
  implement SculkBehaviour in 26.1.2, so the port's 3-way
  behaviour() is exact; `canSpreadFrom` matches.
- **Jigsaw ground_level_delta — VERIFIED faithful** (`jigsaw.rs`
  byte-identical to 26.1.1; jar unchanged): delta_y, target_y, offset,
  child GLD, junction Y and both junctions' fields match
  `JigsawPlacement$Placer` bytecode.
- **Ore feature — VERIFIED faithful**: `ore_feature.rs` byte-identical
  to the 26.1.1 file — the geometry is untouched (angle nextFloat×π,
  spread size/8, max_radius, ends via sin/cos, y0/y1, OCEAN_FLOOR_WG
  gate, doPlace lerp/step/radius, the f32 π×step into the Mth.sin
  table, sphere-culling loop, BitSet index, `shouldSkipAirCheck`,
  section-block writes). Jar unchanged.
- **Spawn finder search — VERIFIED faithful** (`spawn_finder.rs`
  byte-identical to 26.1.1; the f32/f64 quantize cast is G17):
  origin sample, radial searches (2048, 512) then (512, 32), rings,
  x/z via Math.sin/cos × radius truncated to int, angle +=
  increment/radius with 2π reset, fitness = min over targets of Σ
  square(distance) + square(offset), scaled by 2048² + x²+z² bias,
  quart-corner sampling — all match the 26.1.2 jar
  (`javap-spawnfinder.txt`). (The DECODE is broken — G12 — but the
  search itself matches.)
- **Chunk stages — VERIFIED**: carvers run inside `generate_proto`
  (`Pipeline::generate_with_context`, noise fill → surface at
  `generator.rs:1819` → carvers at `:1849`, gated by
  `run_surface`/`run_carvers`); `decorate` is a separate step over a
  3×3 region. Basis for the B6-H STAGE RULE. Note: the doc comment
  says "up to TERRAIN" — 26.3 terminology for what 26.1.2 calls
  NOISE/SURFACE/CARVERS; doc-only.
- **Flat preset loading — VERIFIED**: `FlatSettings` fields match jar
  `FlatLevelGeneratorSettings` (structureOverrides, layersInfo, biome,
  decoration, addLakes); `settings.rs` byte-identical to 26.1.1; the
  flat path's layer handling matches vanilla (MOTION_BLOCKING
  non-opaque → null layer, void_gen when all air, sea level −63).
  The flat path still goes through `Pipeline::new`, so G10 blocks it
  in practice.
- **would_survive spot check — VERIFIED**: `block_survives`
  delegates to the sim's `can_survive`. SnowLayer and SugarCane both
  match their jar `canSurvive` exactly (`block_survival.rs`
  byte-identical to 26.1.1; `survive.rs` byte-identical).
- **Spring feature — VERIFIED faithful** (`spring.rs` byte-identical
  to 26.1.1; jar unchanged): above in validBlocks; requiresBlockBelow →
  below in validBlocks; origin air or validBlocks; rock/hole counts
  over the 5 neighbours (W/E/N/S/below, NOT above); rock==rockCount
  && holes==holeCount → setBlock legacy fluid block flag 2 +
  scheduleTick fluid 0; codec defaults requires_block_below=true,
  rock_count=4, hole_count=1 match the port's. 6 spring configured
  features in the data (the 26.1.1 report's "7" was a miscount —
  the audit walk counts `spring_feature` ×6 total).
- **Aquifer engine — VERIFIED faithful** (see B6-G03): the engine
  matches the 26.1.2 jar; only the wiring is missing (B6-G09). The
  sole diff vs the 26.1 file is the f32 sampler return (G17).
- **API-shape conformance — PASS with a note**: no `densityfunction` /
  `material/rule` references in Rust worldgen code; the port's
  `ChunkStatus` enum is the 26.3 form (single TERRAIN, no
  NOISE/SURFACE/CARVERS) but behaviourally inert — `generate_proto`
  performs all three 26.1.2 stages in one call, and the only NBT use
  of the enum is retrogen `target_status` (`retrogen.rs`,
  `terrain.rs:337,348`), which no reference-world chunk carries (0 of
  7,074 chunks with retrogen/blending data). The 26.1, 26.1.1 and
  26.2 trees have the identical enum.

## §5 Re-check verdicts

### 26.1.1 items (PRIMARY — all 25 decided)

| 26.1.1 item | Verdict for 26.1.2 | 26.1.2 item |
|---|---|---|
| B6-G01 sculk_patch tail | REPRODUCES | B6-G01 (gap, major) |
| B6-G02 snow-on-packed-ice | FIXED (withdrawn — verified absent) | B6-G02 (fixed, major) |
| B6-G03 water deficit | REPRODUCES | B6-G03 (gap, major) |
| B6-G04 PoolPiece GD | FIXED (withdrawn — verified absent) | B6-G04 (fixed, minor) |
| B6-G05 sculk water gates | REPRODUCES | B6-G05 (gap, minor) |
| B6-G06 empty carvers/features | REPRODUCES | B6-G06 (gap, major) |
| B6-G07 sculk density cap | REPRODUCES | B6-G07 (gap, minor) |
| B6-G08 sculk cursor 26.3 form | REPRODUCES | B6-G08 (gap, minor) |
| B6-G09 aquifers disabled | REPRODUCES | B6-G09 (gap, major) |
| B6-G10 NoiseRouter 8-field | REPRODUCES | B6-G10 (gap, major) |
| B6-G11 material_rule required | REPRODUCES | B6-G11 (gap, major) |
| B6-G12 spawn_target decode | REPRODUCES | B6-G12 (gap, major) |
| B6-G13 density arms missing | REPRODUCES | B6-G13 (gap, major) |
| B6-G14 feature types missing | REPRODUCES | B6-G14 (gap, major) |
| B6-G15 state-provider names | REPRODUCES | B6-G15 (gap, major) |
| B6-G16 `random_offset` missing | REPRODUCES | B6-G16 (gap, major) |
| B6-G17 f32 numeric pipeline | REPRODUCES | B6-G17 (gap, minor) |
| B6-O01 densityfunction oracles | REPRODUCES (9 oracles) | B6-O01 (oracle-defect, major) |
| B6-O02 material/rule oracles | REPRODUCES (5 oracles) | B6-O02 (oracle-defect, major) |
| B6-O03 Feature ctor oracles | REPRODUCES (3 oracles) | B6-O03 (oracle-defect, major) |
| B6-O04 missing-symbol oracles | REPRODUCES (12 oracles) | B6-O04 (oracle-defect, major) |
| B6-O05 missing-override oracles | REPRODUCES (2 oracles) | B6-O05 (oracle-defect, major) |
| B6-O06 no-override oracles | REPRODUCES (4 oracles) | B6-O06 (oracle-defect, major) |
| B6-O07 other oracle failures | REPRODUCES (4 oracles) | B6-O07 (oracle-defect, major) |
| B6-H paired comparison | REPRODUCES (needs-harness) | B6-H (needs-harness, minor) |

New in 26.1.2 (no 26.1.1 counterpart): none.

### 1.21.7 items (all 10 decided)

| 1.21.7 item | Verdict for 26.1.2 |
|---|---|
| B6-G01 sculk_patch tail | REPRODUCES (→ B6-G01) |
| B6-G02 snow on packed ice | WITHDRAWN (→ B6-G02, fixed) |
| B6-G03 water deficit | REPRODUCES via total aquifer absence (→ B6-G03) |
| B6-G04 PoolPiece GD | WITHDRAWN (→ B6-G04, fixed) |
| B6-O01 densityfunction group | REPRODUCES (→ B6-O01; jar listing: 0 entries in both 26.1.2 jars) |
| B6-O02 material/rule group | REPRODUCES (→ B6-O02; 0 entries in both jars) |
| B6-O03 Feature ctor group | REPRODUCES (→ B6-O03; 3 oracles) |
| B6-O04 missing-symbols group | REPRODUCES (→ B6-O04; 12 oracles) |
| B6-O05 other missing packages | REPRODUCES (→ B6-O04/O07; blockscan, etc.) |
| B6-O06 BiomeZoom group | REPRODUCES (→ B6-O07) |

## §6 Systemic findings

Two systemic causes, both 26.3-form carry-over — unchanged from 26.1.1:

1. **The 26.1.2 tree carries 26.3 forms where 26.1.2 uses the 1.21.x
   forms** — the prepare folder names (G06), the NoiseRouter (G10), the
   aquifer config object (G09), the material-rule name (G11), the
   spawn-target named densities (G12), the density cache/shifted-noise
   arms (G13), 11 feature types + the nether carver (G14), the
   state-provider short names (G15), and the missing `random_offset`
   arm (G16). The 26.2 tree is the repair template for G06/G09–G14;
   the merged 26.1 tree is the repair template for G13's three extra
   arms, G15 and G16.
2. **The 26.1.2 tree's numeric pipeline is f32 where vanilla is f64**
   (G17) — density graph, climate fields, blending data, biome
   temperature noises. The 26.1 tree is the repair template (f64
   graph + `PerlinSimplexNoise`).

The sculk defects (G01/G05/G07/G08) are the exception: same
narrow-predicate pattern as 26.2's G05, found only by javap-diffing the
feature's gates. Arm presence is not correctness — G05, G07 and G08
were silent narrowings inside present arms, and G15's audit "ok" for
`simple_state_provider`/`weighted_state_provider` was a false positive
from an unrelated file (`litho.rs`); this run adds two more of the same
class (`seagrass` grep-ok with no dispatch arm; the fortress spawn
overrides in the structure audit).

## Prior verification referenced (not re-diagnosed)

- Batch 0 oracle measurement (`docs/sweep/26.1.2/batch-0-oracles.json`,
  189/391 compile, byte-identical to the 26.1.1 table) is the oracle
  authority; first-error symbols pulled from the raw
  `big-compile-client.log` in `b0/evidence/`.
- Batch 0's jar-diff (26.1.1→26.1.2: only `DetectedVersion`,
  `SharedConstants`, `ServerGamePacketListenerImpl`, `Checkbox` ×3,
  `AbstractReportScreen` ×2 changed) is the basis for carrying
  26.1.1's javap evidence for unchanged classes; every defect item was
  still re-verified fresh against the 26.1.2 jar.
- Batch 0's G10 group (17 re-ported 26.3-form oracle sources) is the
  established cause of the 5 newly-failing B6 oracles (vs 26.1).
- The 26.2 sweep's verification of `aquifer/noise_based.rs` against
  the 26.2 jar stands for the 26.1.2 file (byte-identical to 26.1.1's),
  spot-checked here against the 26.1.2 jar (ctor, 13 offsets,
  computeSurfaceLevel).
- The 1.21.7 world comparison (2026-11-22, 4,145 paired chunks) is the
  source of the G03 −1.04% figure; the 26.1.2 mechanism (total aquifer
  absence) is demonstrated here, not carried over.

## Open items (not defects)

- The 9 runnable + 17 no-oracle drivers were not run (execution
  block) — they are unmeasured, not passing. When the block lifts, the
  9 runnable drivers are ready; the 38 blocked drivers need their
  oracles fixed first.
- B6-H is blocked until the construction cascade (G06/G10/G13/G11/G12)
  is fixed; no port world can be generated today, so no live
  measurement in this batch is possible beyond the vanilla-side census.
- G17's magnitude (how many blocks differ from f32 rounding) is
  unmeasured statically; B6-H arbitrates it.
- The 26.2 tree's `feature_tags` prepare gap (line 1478 still
  `tags/worldgen/feature`) is noted under G06 for the 26.2
  coordinator; the 26.1.2 tree has the identical gap at the identical
  line (its prepare lib.rs is byte-identical to 26.1.1's).
- `dripstone_cluster` is missing in the 26.1.2 tree (26.3 name
  `speleothem_cluster` present instead), as in 26.1, 26.1.1 and 26.2.
- `VanillaSteppedInterpolationOracle` (16 errors, new file replacing
  the passing `VanillaInterpolationOracle`) and its driver
  `test_stepped_interpolation_parity` are out of the B6 driver set by
  the keyword rule (no worldgen keyword); flagged for the batch that
  owns interpolation. Same for the 4 non-B6 newly-failing oracles in
  Batch 0's G10 group.

## Adjacent findings (not B6 defects)

- **DUMMY block entities (42)** in the reference world — vanilla-side
  oddity, BEs owned by another batch.
- The `tag()` silent-empty footgun (missing tag → empty set, no error)
  is the mechanism behind G07; it is a robustness gap in the shared
  survival/tag loader, not a worldgen defect per se (all tags used by
  the verified snow path are present in the prepared data).
- The 26.1.2 `level/sculk.rs` reads the absent
  `sculk_growth_inhibitors` tag at :620 (dead cap in live gameplay)
  and narrows the water gate to sources at :611 — same patterns as
  G07/G05, in B4's live-sculk scope; flagged for B4, not minted here.
- The port's `ChunkStatus` enum (26.3 form) is noted under API-shape
  above; inert today.
- `generator.rs` tests still reference a `26.3` prepared-data dir
  (`../../.local/horizon-client/26.3`) in several test paths (lines
  4077, 4116, 4309) — test-only, not a behaviour defect.
- The decode audit's density-function glob is non-recursive
  (`density_function/*.json` misses the `overworld/`/`end/`/`nether/`
  subdirectories); the recursive census in this report supersedes its
  density counts (cache_2d ×11 not ×2, flat_cache ×16 not ×2, plus
  cache_once ×12, weird_scaled_sampler ×3, y_clamped_gradient ×6,
  end_islands ×1).
- **NEW in the 26.1.1→26.1.2 delta (chunk NBT, not worldgen — flagged
  for the owning batch):** `horizon-world/src/chunk.rs` dropped the
  legacy palette reader — `BlockState::from_palette_nbt` (which
  accepted the `Name`/`Properties` form) is gone, and `from_nbt` now
  reads only the strict `id`/`properties` form. The vanilla 26.1.2
  reference world still writes palettes as `Name`/`Properties`
  (verified: first palette entry `{'Name': 'minecraft:air'}`), so a
  strict-`id` reader would fail on vanilla 26.1.2 saves. The 26.1.1
  tree's comment said the rename to `id` was "a later fix, key 5006"
  — the 26.1.2 tree adopted the new form while vanilla 26.1.2 still
  writes the old one. Chunk save/load is outside B6 scope; not minted
  here.
- Corrections to the 26.1.1 record found during this run (both are
  26.1.1-report wording errors, not 26.1.2 changes — the underlying
  data is byte-identical between the two versions): (1)
  `minecraft:y` is `y_clamped_gradient` from_y −4064 / to_y 4062,
  not −64 / 320; (2) there are 6 `spring_feature` configured
  features, not 7; (3) biome carver lists read
  `[cave, cave_extra_underground, canyon]` in data order.

## Method notes

- javap `-c -p` on the named 26.1.2 server-game jar for every vanilla
  method spot-checked (SculkPatchFeature, SculkBehaviour/SculkBlock/
  SculkSpreader/ChargeCursor, SnowAndFreezeFeature (via 26.1.1 carry +
  jar-delta), Biome.shouldSnow/shouldFreeze/coldEnoughToSnow/
  getTemperature/getHeightAdjustedTemperature, SnowLayerBlock.
  canSurvive, JigsawPlacement/Placer (via 26.1.1 carry + identical
  source), OreFeature/canPlaceOre (via 26.1.1 carry + geometry-identical
  diff), Aquifer$NoiseBasedAquifer incl. ctor/computeFluid/
  computeSurfaceLevel/calculatePressure, OverworldBiomeBuilder.
  isDeepDarkRegion (via 26.1.1 carry), Climate$SpawnFinder/
  ParameterPoint/Parameter, MultiNoiseBiomeSourceParameterList Preset
  (via 26.1.1 carry + prepared-data check), SpringFeature/
  SpringConfiguration (via 26.1.1 carry + identical source),
  FlatLevelGeneratorSettings (via 26.1.1 carry + identical source),
  NoiseRouter, NoiseGeneratorSettings (via prepared-data check),
  DensityFunction, Feature, ConfiguredFeature, ChunkStatus,
  BlendingData.NO_VALUE). Fresh dumps in `evidence/javap-*.txt`; the
  12 classes dumped in both runs are byte-identical to the 26.1.1
  dumps (modulo `-p` private members and bytecode sections the
  26.1.1 dumps omitted for 4 classes).
- Exact data diffs of prepared resources vs jar (715 populated
  registry entries; structure NBTs 1202/1202; flat presets 9/9;
  multi-noise presets). Jar data extracted to `evidence/jar-data/`.
- Vanilla 26.1.2 reference world census with an own NBT parser (read
  in place; no git commands inside vanilla-world-mc; no world files
  copied or moved). Census scripts + `world-26.1.2-census.json` in
  `evidence/`.
- Decode-coverage audit of every worldgen dispatch arm against the
  jar data (feature_graph.rs / state_provider.rs / placement.rs /
  density.rs dispatch tables are the authority, not string presence —
  the audit's grep produced false "ok" verdicts for
  `simple_state_provider`/`weighted_state_provider` from `litho.rs`
  and for `seagrass`, and its density glob was non-recursive). Script
  `evidence/decode_audit.py`, results `evidence/decode-audit.json`,
  recursive density census in `evidence/recursive-census.log`.
- Batch 0 oracle-status triage for all 57 oracles referenced by the
  batch's 64 drivers (18 pass / 39 fail). Driver list
  `evidence/drivers-26.1.2.json`.
- No driver was run, no cargo builds, no commits, no writes under the
  worktree. All artifacts under
  `~/workspace/mcir-work/sweep-26.1.2-batches/b06/`.

## §7 Evidence index

All under `~/workspace/mcir-work/sweep-26.1.2-batches/b06/evidence/`:

- `world-26.1.2-census.json` — reference-world census (seed
  4656297851632877335, 7,074 chunks, 695,402,496 blocks, 18 biomes,
  structure starts + GD/GLD spreads, block entities).
- `census.py`, `census2.py`, `census3.py`, `census.log`,
  `census2.log`, `census3.log` — census scripts + logs (seed
  confirmation, generator info, snow-on-packed-ice 0/31,772 +
  0/2,190 + 0/8,066).
- `nbtmini.py`, `coldump.py` — own NBT parser (reused from the 26.2
  run via the 26.1.1 run).
- `datadiff.py`, `datadiff.log` — prepared-vs-jar data diff (715/715,
  NBTs 1202/1202, flat 9/9, carvers/features/feature_tags empty).
- `decode_audit.py`, `decode-audit.json`, `decode_audit.log`,
  `recursive-census.log` — decode-coverage audit (non-recursive
  density glob noted; recursive census supersedes it).
- `triage.py`, `triage.log`, `drivers-26.1.2.json` — Batch 0 oracle
  triage (64 drivers: 9 runnable / 17 no-oracle / 38 blocked; 55
  referenced oracles: 18 pass-any / 37 fail-all).
- `jar-data/` — 26.1.2 server-game.jar `data/` + client.jar
  `data/`/`assets/` extract.
- `javap-sculkpatch.txt`, `javap-sculkblock.txt`,
  `javap-sculkbehaviour1.txt`, `javap-chargecursor.txt`,
  `javap-sculkspreader.txt`, `javap-aquifer.txt`,
  `javap-spawnfinder.txt`, `javap-snowlayer.txt`, `javap-biome.txt`,
  `javap-densityfunction.txt`, `javap-feature.txt`,
  `javap-configuredfeature.txt`, `javap-chunkstatus.txt`,
  `javap-noiserouter.txt` — fresh javap dumps on the 26.1.2
  server-game jar.

## §8 Verdict

**27 behaviour checks: 19 pass, 8 gap (27 behaviour + 0 non-behaviour
as-tallied). 25 items: 15 gap (11 major, 4 minor), 2 fixed, 7
oracle-defect (major), 1 needs-harness (minor).** Oracle triage
separate: 57 oracles, 18 pass / 39 fail; 64 drivers, 9 runnable /
17 no-oracle / 38 blocked.

The 26.1.2 worldgen port reproduces all 15 live 26.1.1 defects in scope,
withdraws the same 2 (G02, G04), and adds no new defects of its own —
the worldgen tree, the oracle sources, the drivers, the Batch 0 oracle
table and the jar's worldgen data are all byte-identical to 26.1.1's.
No 26.1.2 world can generate today: the construction cascade
(G06 → G10 → G13 → G11 → G12) blocks all construction, aquifers are
silently absent behind it (G09/G03), and 126 of 221 configured
features cannot decode (G14∪G15). The 26.2 tree is the repair template
for the cascade; the merged 26.1 tree is the repair template for G13's
extra arms, G15, G16, G17 and the 5 regressed oracles. Nothing in this
batch is UNVERIFIED; B6-H (paired chunk comparison) is the only
open measurement, blocked on the cascade.
