# Batch 6 — Worldgen / features (26.1 parity sweep)

Date: 2026-11-24. Checks + triage only; no fixes, no code changes, no commits.
Worktree `~/workspace/mcir-work/sweep-26.1`, branch `parity-sweep-26.1` (read-only).
Vanilla reference: the named 26.1 jars
`~/workspace/mcir-work/26.1.x/versions/26.1/jars/{client,server-game}.jar`
(never the leakage-audit bundler jars). All behaviour claims verified against
26.1's OWN jar/data, never the 26.3 base's form.

## Tally (behaviour checks only)

**25 checks: 18 pass, 7 gap.**

| Group | Checks | Pass | Gap |
|---|---|---|---|
| 1.21.7 defect re-checks | 4 | 2 (G02, G04) | 2 (G01, G03) |
| Data diffs (prepared vs jar) | 6 | 5 | 1 (carver/feature bundles) |
| Code-vs-jar | 10 | 6 | 4 (sculk, spawn finder, decode-coverage, generator construction) |
| Reference-world census | 5 | 5 | 0 |
| **Total** | **25** | **18** | **7** |

Oracle triage is counted separately (per the 26.1 tally convention; the 26.2
batch counted oracle-status checks in its tally — this batch does not):
**57 oracles referenced** (55 B6-referenced + 2 group-only), **23 pass, 34 fail**;
**63 drivers: 14 runnable, 16 no-oracle, 33 blocked** (see Oracle defects).

Defect items: **12** (3 reproduced 1.21.7 defects, 9 new) — 9 major, 3 minor.
Withdrawn 1.21.7 defects: **2** (G02, G04). Oracle items: **7** (B6-O01…O07).
Needs-harness: **1** (B6-H).

Headline: the 26.1 worldgen port is materially LESS faithful than the 26.2
tree was at sweep time. The 26.2 port had carried the 1.21.x/26.2 forms
(router, aquifer wiring, surface rule, density caches, feature arms) into its
tree; the 26.1 tree still carries the 26.3 forms in all of them. The result is
a construction cascade: **no 26.1 world can generate at all with the current
tree + prepared data** — `GeneratorData::load` fails on the empty carver
bundle (G06), and behind it `Pipeline::new` fails on the router (G10), the
density caches (G13), the material rule (G11) and the spawn target (G12), in
that order; even with all of those fixed, aquifers would be silently absent
(G09) and 22 configured features would fail at decoration (G14).

## Defects

### B6-G01 (major, 1.21.7 defect — REPRODUCES) — sculk_patch catalyst / extra rare growths tail missing

`versions/26.1/crates/horizon-world/src/worldgen/sculk.rs` (727 lines) has no
`catalyst_chance`/`extra_rare_growths` decode and no catalyst handling:
`decode` reads only charge_count, amount_per_charge, spread_attempts,
growth_rounds, spread_rounds; `place()` returns after the spreader rounds; the
string `catalyst` appears nowhere in the file.

Vanilla 26.1 (`SculkPatchFeature.place`, javap on the 26.1 server-game jar):
after the spreader rounds, `nextFloat() <= catalystChance` with a
collision-full block below places `SCULK_CATALYST` at the origin (flag 3),
then `extraRareGrowths.sample` × `origin.offset(nextInt(5)-2, 0,
nextInt(5)-2)` with air above and a face-sturdy block below places
`SCULK_SHRIEKER[can_summon=true]` (flag 3). The catalyst roll happens BEFORE
the growth sample, so the missing tail also shifts the RNG stream for every
feature placed after a sculk_patch in the same chunk.

26.1 data: `sculk_patch_deep_dark` catalyst_chance 0.5, extra_rare_growths 0;
`sculk_patch_ancient_city` 0.5, uniform 1..3. The 26.2 tree HAS the tail
(sculk.rs lines 59–60, 97–112, 178, 188); the 26.1 tree does not.
Consequence: no catalysts/shriekers from sculk_patch in 26.1. Vanilla-side
ground truth from the reference world: 118 sculk_catalyst blocks, 32
sculk_shrieker blocks (deep_dark present, 123,723 biome cells).

### B6-G03 (major, 1.21.7 defect — REPRODUCES) — underground water deficit; the entire aquifer system is absent

The 1.21.7 world comparison measured water −1.04% underground with surface
water matching; never root-caused. For 26.1 the mechanism is demonstrated,
and it is total: **the port never constructs a noise-based aquifer**
(see B6-G09). `Pipeline::new` reads the 26.3 `aquifers` config object from the
noise settings; no 26.1 noise settings has that key, so the aquifer is `None`
and generation uses `DisabledAquifer` — sea-level picker water only, no
underground lakes, no flooded caves, no aquifer lava. The 1.21.7 deficit
(−1.04%) would be far larger in 26.1: ALL aquifer water is missing.

The aquifer ENGINE itself matches the 26.1 jar: `aquifer/noise_based.rs` is
byte-identical to the 26.2 file (verified against the 26.2 jar in that sweep),
and spot-checked against the 26.1 jar here — the ctor reads
barrier/fluidLevelFloodedness/fluidLevelSpread/lava/erosion/depth from the
router, the 13 `SURFACE_SAMPLING_OFFSETS_IN_CHUNKS` match the port's
`SURFACE_OFFSETS` entry for entry, `computeSurfaceLevel` uses
`OverworldBiomeBuilder.isDeepDarkRegion` with the same clampedMap/thresholds,
`FLOWING_UPDATE_SIMILARITY = 1.0 − 44/25`. The defect is purely the wiring:
the engine is never built. Vanilla-side water: 4,876,455 blocks over 8,138
chunks in the reference world.

### B6-G05 (minor, 1.21.7 defect — REPRODUCES) — sculk spreader water gates narrowed to water sources

Two sites in `worldgen/sculk.rs` require `WATER_SOURCE` where the 26.1 jar
tests fluid-is-water:
- `sculk.rs:305` (`attempt_spread_vein`, Default branch, non-empty facings);
- `sculk.rs:548` (`can_place_growth` above-open gate).

Vanilla 26.1: `SculkBehaviour$1.attemptSpreadVein` regrows when the state is
air OR `fluidState.is(Fluids.WATER)`; `SculkBlock.canPlaceGrowth` accepts
above air OR (is(WATER) AND fluidState.is(WATER)). The port's `WATER_SOURCE`
is set only for source water (`tree_blocks.rs:184`; flowing water gets
`WATER_FLUID|ANY_FLUID` only), so flowing water blocks sculk vein regrowth
and growth placement that vanilla allows. No RNG effect (no random calls in
either gate). Same defect as in 26.2 (B6-G05 there too).

### B6-G06 (major, NEW) — prepared `carvers.json` and `features.json` are EMPTY; `GeneratorData::load` fails

Prepared `worldgen/carvers.json` (0 entries), `worldgen/features.json`
(0 entries) and `worldgen/feature_tags.json` (empty). Cause:
`horizon-prepare` `WORLDGEN_BUNDLES`
(`crates/horizon-prepare/src/lib.rs:1469-1478`) maps
`("carvers","worldgen/carver")`, `("features","worldgen/feature")`,
`("feature_tags","tags/worldgen/feature")` — the 26.3 folder names. The 26.1
jar uses `worldgen/configured_carver` (4: canyon, cave,
cave_extra_underground, nether_cave), `worldgen/configured_feature` (221),
`tags/worldgen/configured_feature` (2 files). The 26.2 tree's prepare fixed
carvers/features (lines 1471/1474) but NOT feature_tags (line 1478 still
`tags/worldgen/feature`; the 26.2 jar also uses `configured_feature` tags, so
26.2's feature_tags is empty too — latent gap there).

The raw copies under prepared `data/minecraft/worldgen/` are 4/4 and 221/221
identical to the jar, but the generator does NOT read them:
`GeneratorData::load` (`generator.rs:486,497`) reads the `worldgen/*.json`
bundles only. Runtime effect: `CarverCatalog::from_json({})` succeeds, then
`decode_set` errors `unknown carver` for 59 of 65 biomes (54 list `[canyon,
cave, cave_extra_underground]`, 5 nether biomes use single-string
`minecraft:nether_cave`, 6 have empty lists) with `?`, so
**`GeneratorData::load` fails and no 26.1 world can generate with this
prepared data**; separately `FeatureCatalog::named` errors `unknown feature
reference` for every placed feature. This is the FIRST failure in the
construction cascade.

### B6-G07 (minor, NEW) — sculk `can_place_growth` density cap is dead (26.3-only tag)

Port `sculk.rs:66,108` reads tag `minecraft:sculk_growth_inhibitors`, which
does NOT exist in 26.1 (nor 26.2) — 26.3 added it; absent from both jars and
from prepared `block_tags.json`. `tag()` returns an empty set, so the ≥3 count
at `sculk.rs:556-558` never fires and sensors/shriekers pile up without
limit. Both the 26.2 worldgen sculk.rs (lines 606–609) and the 26.1
`level/sculk.rs` (lines 620–623) check sensor/shrieker directly with an
explicit comment. Note: a wrongly-passing gate also consumes RNG
(`nextInt(50)`, `nextInt(11)`) and returns charge−50 instead of the decay path.

### B6-G08 (minor, NEW) — sculk cursor movement uses the 26.3 form

Port `update()` (`sculk.rs:263-270`) kills a stuck cursor (None →
on_discharged + charge=0 + return) and has NO post-move 15-block check;
`valid_movement` (`sculk.rs:642,659-663`) applies a pre-move 12-block filter
(dx²+dz² > 144). The 26.1 jar `ChargeCursor.update`: a null movement pos
falls through to facings/decay-delay/update-delay (NOT killed); the post-move
worldgen check kills the cursor when NOT `closerThan(origin-x/z, 15.0)`;
`getValidMovementPos` takes no origin and has no 12-block filter. The 26.2
tree has the post-move `dx²+dz² >= 225` check and no pre-filter (with a
comment stating 26.3 checks 12 before the move and kills stuck cursors).

### B6-G09 (major, NEW) — aquifers are entirely disabled (26.3 config-object form)

`Pipeline::new` (`generator.rs:1126-1129`) reads
`settings_json["aquifers"]` — a 26.3 config-object key — and builds the
aquifer config from it. NO 26.1 noise settings has that key (all 7 checked:
overworld, amplified and large_biomes have `aquifers_enabled: true`; caves,
end, floating_islands, nether have false; none has `aquifers`). So the
aquifer is always `None` and generation uses `DisabledAquifer`
(`generator.rs:1770, 2031`): density ≤ 0 → the global fluid picker only
(sea-level water, lava below the picker threshold), no barrier /
floodedness / spread noise, no underground lakes or flooded caves.

Vanilla 26.1 builds `NoiseBasedAquifer` from the ROUTER (jar ctor takes the
`NoiseRouter` and reads barrierNoise, fluidLevelFloodednessNoise,
fluidLevelSpreadNoise, lavaNoise, erosion, depth — the last two for the
deep-dark exclusion). The 26.2 tree fixed exactly this (`from_router` +
`aquifers_enabled` gate, `generator.rs:1164-1171`, with
`synthesize::aquifer_exclusion`); the 26.1 tree has neither `from_router`
nor any `aquifer_exclusion` synthesis. This is the mechanism behind B6-G03.
Silent: no error is raised, the aquifer is just absent.

### B6-G10 (major, NEW) — NoiseRouter is the 26.3 eight-field form; `Pipeline::new` fails on 26.1 settings

`worldgen/router.rs` declares 8 fields (temperature, vegetation, continents,
erosion, depth, ridges, `chunk_surface_level`, final_density) and
`from_json` requires all of them. The 26.1 jar's `NoiseRouter` record has 15
components (barrier, fluid_level_floodedness, fluid_level_spread, lava,
`preliminary_surface_level`, vein_toggle, vein_ridged, vein_gap, plus the
eight) — there is NO `chunk_surface_level` in 26.1. All 7 prepared noise
settings lack that key, so `NoiseRouter::from_json` errors at
`generator.rs:1105` for every settings object, before the aquifer, rule and
spawn-target steps. `OverworldGenerator::for_world_with_settings` (the real
world-construction path) and the flat path both go through `Pipeline::new`,
so no noise or flat world can be constructed. The 26.2 tree has the correct
15-field form (`router.rs`, `ALL: [Self; 15]`, `PreliminarySurfaceLevel`).
Second failure in the construction cascade (after G06).

### B6-G11 (major, NEW) — `Pipeline::new` requires `material_rule`; 26.1 settings carry an inline `surface_rule`

`generator.rs:1148-1151`: `settings_json["material_rule"].as_str()` is
required, and the name is looked up in the material-rule library. No 26.1
noise settings has a `material_rule` key — they carry an inline
`surface_rule` object (`{type: sequence, ...}`), the 1.21.x/26.2 form, and
`rules.json`/`conditions.json` are legitimately empty (no such data folders
exist in the 26.1 jar; SurfaceRules are inline in the settings). The 26.2
tree feeds the inline rule to the library under a synthetic name
(`INLINE_SURFACE_RULE`, `generator.rs` surface_rule handling). Third failure
in the construction cascade (after G06, G10 — and after G13, which fires
during the router decode's density resolution).

### B6-G12 (major, NEW) — spawn_target decode fails (named-density lookups); depth not zeroed

`generator.rs:1153-1175` decodes each `spawn_target` point by mapping every
field name through `library.named_density(name)` — i.e. looking up a density
function named `minecraft:<field>`. None of the 7 field names used by the
26.1 overworld/amplified/large_biomes spawn targets (continentalness, depth,
erosion, humidity, offset, temperature, weirdness) is a registered density
function (the 35 registry entries are all `minecraft:overworld/...`,
`minecraft:end/...`, `minecraft:nether/...`, `minecraft:shift_x`,
`minecraft:shift_z`, `minecraft:y`, `minecraft:zero`), so the decode errors
with `unknown material density` for every settings object with a non-empty
spawn_target.

Latent second half: even if the lookups resolved, the port's
`SpawnTargetPoint::sample_fitness` samples each named density, but vanilla's
`Climate$SpawnFinder.getSpawnPositionAndFitness` rebuilds the sampled
`TargetPoint` with depth FORCED to 0 (javap: `lconst_0` as the depth
argument) and adds `square(offset)` as a constant — it never samples a depth
or offset function. The 26.1 overworld depth function at y=0 is ~1.0 + the
offset spline (NOT 0), so sampling it would add a large position-varying
bias to every candidate's fitness and distort the spawn search. The 26.2
tree decodes spawn_target via `decode_parameter_point` (six climate
parameters + offset constant) and its spawn_finder zeroes the depth. The
radial-search / fitness FORMULA in the port matches the jar exactly (verified
— see Verified passes); only the decode and the depth handling are wrong.

### B6-G13 (major, NEW) — density decode missing `cache_2d`, `cache_once`, `flat_cache`, `shifted_noise`

`worldgen/density.rs` has no arms for these four types; the default arm
errors `unknown density type: {kind}`. All four are used in 26.1 data:
- `minecraft:overworld/offset` is `flat_cache` > `cache_2d` > `add(...)`
  (with `cache_once` around `blend_alpha` inside) — and `offset` is an
  argument of `minecraft:overworld/depth`, a router field. Decoding the
  router's depth function therefore fails.
- `cache_2d` ×13 and `shifted_noise` ×8 in the noise settings' surface rules.
- `cache_2d` ×2 / `flat_cache` ×2 across the density-function registry.

The 26.2 tree has all four arms (`density.rs:362` cache_2d|cache_once|
flat_cache, `:402` noise|shifted_noise); the 26.1 compiler even has a
`ShiftedNoiseSampler` (`horizon-core/src/density/mod.rs:437`) that the JSON
decoder never produces. Fires during the router decode in `Pipeline::new`
(after G10's missing-field error, before G11).

### B6-G14 (major, NEW) — 11 configured-feature types + `coral_mushroom` + `nether_cave` have no decode arms

Top-level configured-feature types in the 26.1 data: 53 distinct, 221 total.
Eleven have NO arm in the port's feature dispatch (`feature_graph.rs`, plus
`feature.rs` for the end features) — the default arm falls through to
`BasicFeature::from_json`, which errors `basic feature type` for anything
but the three end features:

| Missing type | Configured features | 26.2 tree |
|---|---|---|
| `basalt_columns` | 2 | HAS arm |
| `basalt_pillar` | 1 | HAS arm |
| `desert_well` | 1 | HAS arm |
| `dripstone_cluster` | 1 | also missing (26.3 name `speleothem_cluster` present instead) |
| `glowstone_blob` | 1 | HAS arm |
| `kelp` | 1 | HAS arm (`aquatic.rs` — file absent in 26.1) |
| `nether_forest_vegetation` | 6 | HAS arm |
| `sea_pickle` | 1 | HAS arm (`aquatic.rs`) |
| `seagrass` | 4 | HAS arm (`aquatic.rs`) |
| `twisting_vines` | 2 | HAS arm (`nether.rs`) |
| `weeping_vines` | 1 | HAS arm (`nether.rs`) |

Plus `coral_mushroom`, nested inside `warm_ocean_vegetation`'s
simple_random_selector (dispatch has coral_tree|coral_claw only;
`CoralFeature::decode` errors on other kinds). Total: **22 broken configured
features, affecting 24 placed features** (basalt_pillar, crimson/warped
forest vegetation, desert_well, dripstone_cluster, glowstone,
glowstone_extra, kelp_cold/warm, large/small basalt columns, nether_sprouts,
sea_pickle, seagrass ×8, twisting_vines, warm_ocean_vegetation,
weeping_vines). Feature decoding is lazy (`FeatureCatalog::named`), so these
fail at decoration time when a chunk references one of the 24 placed
features — a hard error during decoration, not at load. (Latent behind G06:
`features.json` is empty, so today every placed feature fails earlier with
`unknown feature reference`.)

Also missing: the `nether_cave` carver arm (`carver/generation.rs` has cave
+ canyon only; 26.2 has `cave | nether_cave`). The 5 nether biomes list
`minecraft:nether_cave` as their carver, so fixing G06 alone would surface
this next for the nether.

Present and verified (see Verified passes): all 33 placement-modifier types,
all 8 int-provider and 6 height-provider types, all 8 state-provider types,
all 9 trunk / 11 foliage / 1 root placers, all 9 tree decorators, all 5 pool
element types, all 4 processor types used, both structure-set placements.

## Withdrawn 1.21.7 defects (verified absent in 26.1)

### B6-G02 (major) — WITHDRAWN: snow-on-packed-ice mechanism verified closed

Mechanism root-caused and verified closed, with stronger evidence than the
26.2 withdrawal:
- The jar tag `cannot_support_snow_layer` = [ice, packed_ice, barrier] and
  `support_override_snow_layer` = [honey_block, soul_sand, mud]; the
  prepared `block-tags.json` contains both with exactly the jar's members.
- The port's `Rule::SnowLayer` (`block_survival.rs:793`) and
  `survive.rs:293` (`Class::SnowLayer`) both implement deny-tag → allow-tag
  → face-full collision UP or same-block layers==8, matching jar
  `SnowLayerBlock.canSurvive` exactly; both survival paths load the tags
  from prepared data (`block_survival.rs:529-530`,
  `level/data.rs:428-429`), not hardcoded.
- `SnowAndFreezeFeature.place` matches the jar exactly (16×16,
  MOTION_BLOCKING height, biome at top, `shouldFreeze(below, false)` → ice
  flag 2, `shouldSnow(top)` → snow flag 2 + snowy=true below).
- `BiomeWeatherBindings::should_freeze` / `should_snow`
  (`biome_weather.rs:46,74`) match jar `Biome.shouldFreeze`/`shouldSnow`
  step by step (warmEnoughToRain gate, build height, block light < 10,
  fluid-is-water + LiquidBlock, the require-edge all-four-water rule;
  precipitation SNOW, air-or-snow cell, `canSurvive` of the snow default
  state), and `height_adjusted` matches `getHeightAdjustedTemperature`
  (modifier, sea_level+17 start, `(float)(noise(x/8, z/8) * 8)`,
  `base − (variation + y − start) × 0.05 / 40`).
- **Vanilla-side ground truth from the reference world: 0 snow blocks on
  packed ice world-wide** — 0 of 29,442 snow blocks in full chunks
  (same-section below-check) plus 0 of 2,357 snow blocks at section bottoms
  (cross-section check), and 0 of 10,610 snow blocks in frozen_peaks biome
  quarts. Unlike the 26.2 reference world, the 26.1 world HAS frozen_peaks
  (70,794 cells) and deep_dark, so this is a live negative, not an absence
  of test terrain.

The snow path (`snow_freeze.rs`, `biome_weather.rs`, `block_survival.rs`) is
byte-identical between the 26.1 and 26.2 trees. Caveat: no port world exists,
so the port's live snow placement cannot be re-measured; live confirmation
is B6-H.

### B6-G04 (minor) — WITHDRAWN: PoolPiece NBT writes `GD`

`PoolPiece::save` (`worldgen/structure/jigsaw.rs:683`) goes through
`legacy::piece_tag("minecraft:jigsaw", bbox, None, 0)`, which writes
`("GD", Tag::Int(gen_depth))` (`legacy.rs:339`). The reference world confirms
the vanilla form: trial chambers 1,750/1,750 pieces GD=0, trail ruins 33/33
GD=0, villages 549/549 GD=0, mineshaft GD 0–9.

## Oracle defects (Batch 0 measured status, 57 oracles, 23 pass / 34 fail)

Authority: `docs/sweep/26.1/batch-0-oracles.json` (199/391 oracles compile;
192 fail). B6-referenced oracles: 55 (32 fail on all 3 classpaths, 23 pass on
≥1), plus 2 group-only failing oracles swept from Batch 0 groups
(`VanillaFloatLogOracle`, `VanillaStructurePlacementOracle`). Drivers: 63
total (14 runnable, 16 no-oracle, 33 blocked) — driver list in
`evidence/drivers-26.1.json`.

Runnable drivers (14): basic_feature, biome_placement, biome_temperature,
chunk_generation, chunk_status, feature_placement, feature_sort,
gateway_replacement, heightmap, placement, retrogen_data, scattered_ore,
world_spawn_sync, worldgen_random. No-oracle (16): biome_weather,
chunk_restart_ticks, datapack_worldgen, end_gateway, flat_generation,
flat_place, folder_datapack_metadata, sculk_saved_fields,
server_player_sculk, spawn_loot, structure_blocks, structure_layout,
unused_feature_startup, spring, end_podium, place_feature_inline. Blocked
(33): the 32 from the 26.2-list subset plus height_provider.

Passing oracles (23): ParityData, BasicFeature, BiomeExport, BiomePlacement,
BiomeTemperature, BlockProperties, BlockStateCodec, BlockSupport,
BlockSurvival, ChunkGeneration, ChunkStatus, Collision, FeaturePlacement,
FeatureSort, GatewayReplacement, Heightmap, Placement, RetrogenData,
ScatteredOre, SimpleBlock, TagExport, WorldSpawnPacket, WorldgenRandom.

### B6-O01 (major) — Oracles reference the 26.3-only `densityfunction` package (10 oracles)

Anchor, BiomeSourceCodec, BiomeSource, Density, FloatLog*, MaterialExtension,
Noise, StructureDensity, WorldSpawn, Worldgen. Verified by jar listing: the
`densityfunction` package has **0 entries in BOTH 26.1 jars**;
`DensityFunction` lives at `levelgen.DensityFunction` (1.21.x shape).
`Anchor` blocks 9 drivers including non-B6 `test_anchor_parity` and
`test_state_provider_parity`. (*FloatLog is group-only: its driver
`test_float_log_parity` is not a B6 driver; recorded here with a cross-ref.)

### B6-O02 (major) — Oracles reference the missing `material/rule` package (5 oracles)

BiomeGeneration, MaterialCondition, MaterialStage, MaterialSurface, Terrain.
Verified: `material/rule` has 0 entries in both jars (`material/` holds only
`MaterialRuleList`); 26.1 surface rules live at `levelgen.SurfaceRules`.

### B6-O03 (major) — Oracle uses the 26.3 `Feature` constructor signature (1 oracle)

Decoration. The 26.1 jar's `Feature` ctor takes only `Codec<FC>`.

### B6-O04 (major) — Oracles fail on missing symbols / signature drift (8 oracles)

BiomeChunk, CanyonCarver, CaveCarver, ChunkStage, CushionPlacement,
FeatureGraph, FeatureRegion, Sculk. First-error symbols from the raw
`big-compile-client.log`: CanyonCarver/CaveCarver —
`WorldCarver.DIRECT_CODEC` (absent; only `configuredCodec()`), `CarverOutput`,
`CarvingMask.minY/maxY/carve` (`CarvingMask` is at
`world.level.chunk.CarvingMask` with set/get/stream/toArray only);
ChunkStage/FeatureRegion — `ChunkStatus.TERRAIN` (the 26.1 set has
NOISE/SURFACE/CARVERS, no TERRAIN); Sculk —
`datapackWorldRegistries()` (absent; `datapackWorldgen()` exists),
MinecraftServer ctor, `WardenSpawnTracker` Optional wrapping. Also absent
from the jar: Interval, GradientNoise/Noise/NoiseStack/SmearedPerlinNoise
(synth has only NormalNoise/PerlinNoise), NoiseSpawnFinder, DensitySampler/
SamplerContext, MaterialCondition/MaterialRule/MaterialSystem, Cushion,
Continuation, blockscan package, BlockMatcher, NativeDestruction, cubemob,
`Registries.BLOCK_STATE_PROVIDER` (26.1 has `BLOCK_STATE_PROVIDER_TYPE`).

### B6-O05 (major) — Oracles fail on missing-override lookup (2 oracles)

DecorationRegion, FeatureCatalog. The named methods have no override in the
26.1 jar (`getNextEntityId()`, `startsForStructure` overloads,
`getAllStructuresAt`, `getBiome(int,int,int)`, `markPosForPostProcessing`
— note `ProtoChunk.markPosForPostprocessing`, lowercase p).

### B6-O06 (major) — Oracles fail on no-override (4 oracles)

FinalizeSpawn, NaturalSpawn, SpawnRules, TreeFeature. Cross-batch overlaps
(coordinator dedups): FinalizeSpawn→B3, NaturalSpawn also referenced by
spawner/trial_spawner/vault drivers.

### B6-O07 (major) — Other oracle failures (4 oracles)

BiomeZoom, StructureStart, HeightProvider (cascade via Anchor),
StructurePlacement* (group-only, no driver references it). WorldSpawn→B3,
Sculk→B4 per the 26.2 numbering.

## Needs-harness

### B6-H (minor) — Paired port-vs-vanilla chunk comparison for 26.1

Generate port chunks at seed −1600712240839185950 (the reference world's
seed, confirmed from `data/minecraft/world_gen_settings.dat`) and compare
against the vanilla reference world (`~/workspace/vanilla-world-mc/26.1/`,
8,138 chunks). **STAGE RULE: the port's `generate_proto` output is
terrain-level** (biomes, noise fill, aquifers, material surface, carvers —
the 26.1 statuses up to CARVERS); **vanilla region files are post-decorate**.
Compare post-decorate to post-decorate for blocks, terrain-level to
terrain-level for density. Direct comparison is only valid where structure
templates contain no air (ancient city: 0 air) — trial chambers/villages
carve air during placement, so a terrain-vs-post-decorate gap there is a
measurement artifact, not a density bug (the 1.21.7 −6.4 'beardifier
residual' was exactly this). B6-H arbitrates: G02/G04 live confirmation,
G03's magnitude once aquifers exist, and the sculk/G07/G08 RNG-stream
effects. Blocked until the construction cascade (G06/G10/G13/G11/G12) is
fixed — no port world can be generated today.

## Verified passes

### Data (exact diffs vs the jar)

- **Populated registries 715/715 entries name-set- and content-identical**
  (exact JSON equality after parsing prepared JSON-encoded strings): biomes
  65, densities 35, noises 62, settings 7, placed_features 258,
  processor_lists 40, structures 34, structure_sets 20, template_pools 188,
  world_presets 6. `rules.json`/`conditions.json`/`state_providers.json`
  legitimately empty (no such folders in the jar — code-only in 26.1).
- **Structure NBTs 1202/1202 byte-identical** (jar
  `data/minecraft/structure/` vs prepared `worldgen/structure/minecraft/`).
- **Flat presets 9/9 identical**: bottomless_pit, classic_flat, desert,
  overworld, redstone_ready, snowy_kingdom, the_void, tunnelers_dream,
  water_world; `visible` tag lists all 9.
- **Nether multi-noise preset verified exactly** against 26.1 jar bytecode
  (`MultiNoiseBiomeSourceParameterList$Preset$1`,
  `Climate.parameters(t,h,c,e,d,w,o)` order): nether_wastes all 0;
  soul_sand_valley humidity −0.5; crimson_forest temperature 0.4;
  warped_forest humidity 0.5, offset 0.375; basalt_deltas temperature −0.5,
  offset 0.175. Prepared `biome-parameters/nether.json` matches.
- **Overworld expansion: 7,593 entries** (26.2 had 7,594; one fewer = no
  sulfur_caves), first mushroom_fields, last deep_dark, 54 distinct biomes,
  md5-identical to the datagen report
  (`79a58ecdb5695a3c8b97f780131d320c`); both prepared copies
  (`biome-parameters/` and `worldgen/biome_parameters/`) identical.

### Vanilla 26.1 reference world (own NBT parser, read in place)

World at `~/workspace/vanilla-world-mc/26.1/26_1/` (the outer `26.1/level.dat`
is a wrapper; regions live under `dimensions/minecraft/overworld/region/`).
Seed **−1600712240839185950** confirmed from
`data/minecraft/world_gen_settings.dat` (`data.seed`). Level name "26.1",
DataVersion 4786, spawn `{pos:[16,84,-48], dimension:'minecraft:overworld'}`
(new lowercase `spawn` compound), GameType 1, Time 1765, datapacks vanilla
only (minecart_improvements, redstone_experiments, trade_rebalance disabled),
bonus_chest 0, generate_structures 1, NO `WorldGenSettings` in level.dat.
Generator: `minecraft:noise`, settings `minecraft:overworld`, biome source
multi_noise preset overworld. No nether/end regions (dirs exist with data/
only).

- **15 regions, 8,138 chunks** (4,370 `minecraft:full`, 2,817
  structure_starts, 318 biomes, 310 carvers, 309 initialize_light, 11 noise,
  3 surface), x −63..43, z −35..49. Statuses use the 26.1 set
  (noise/surface/carvers, no TERRAIN).
- **799,997,952 blocks counted** (= 8138×24×4096 exactly). Water 4,876,455;
  snow 32,522; packed_ice 13,869. Top: air 628,217,894; deepslate
  66,352,891; stone 64,684,078.
- **18 biomes present**, INCLUDING deep_dark (123,723 cells) and
  frozen_peaks (70,794) — unlike 26.2's world. Full list with cells: plains
  6,275,507; forest 2,131,172; taiga 1,601,483; river 621,873; cold_ocean
  364,896; birch_forest 306,978; meadow 279,387; old_growth_birch_forest
  243,027; beach 228,960; deep_dark 123,723; grove 122,847; frozen_peaks
  70,794; dripstone_caves 58,894; lush_caves 26,164; snowy_slopes 19,474;
  stony_shore 11,829; ocean 8,256; flower_forest 4,704.
- Sculk family: sculk 73,335; sculk_vein 24,830; sculk_sensor 406;
  **sculk_catalyst 118; sculk_shrieker 32** (vanilla-side ground truth for
  G01).
- Structure starts: mineshaft 29/3,846 pieces; trial_chambers 7/1,750;
  village_plains 2/324; village_taiga 2/225; trail_ruins 2/33;
  pillager_outpost 2/16; ruined_portal 6/6; ruined_portal_mountain 2/2;
  ocean_ruin_cold 1/3; shipwreck 1/1; buried_treasure 1/1. GD: all jigsaw
  pieces GD=0 (trial chambers 1750/1750, trail ruins 33/33, villages
  324+225); mineshaft GD 0–9. GLD: trial_chambers −37..+26; trail_ruins
  −12..+7; village_plains −1..+4; village_taiga −1..+1; pillager_outpost
  0..1.
- Block entities: sculk_sensor 354, chest 222, decorated_pot 192,
  brushable_block 177, mob_spawner 164, sculk_catalyst 105, vault 92,
  dispenser 85, barrel 77, **DUMMY 68** (vanilla-side oddity, adjacent note
  only — BEs owned by another batch), trial_spawner 66, hopper 49, bed 44,
  sculk_shrieker 29, banner 16, beehive 16, sign 2.
- **Snow-on-packed-ice: 0 world-wide** (see G02).

### Code-vs-jar spot checks (javap + source)

- **Sculk spreader core — VERIFIED faithful** (the G01/G05/G07/G08 defects
  are at the edges, not the core): `createWorldGenSpreader` args (true,
  SCULK_REPLACEABLE_WORLD_GEN, 50, 1, 5, 10) match port constants;
  `can_spread_into` matches `SculkVeinSpreaderConfig.stateCanBeReplaced`
  (blocker set {sculk, sculk_catalyst, moving_piston} =
  `tb::SCULK_BLOCKER`; manhattan-2 sturdy check; fluid gate non-empty
  non-water → false; fire tag; `canBeReplaced` OR air/vein/water-source —
  flowing water is rejected at the replaceable step in vanilla and at the
  fluid gate in the port, same outcome); `SculkBlock.attemptUseCharge` +
  `getDecayPenalty` match the port's Sculk branch exactly (decay roll
  nextInt(5), closerThan 1.0, growthSpawnCost 50 roll, 1-in-11
  shrieker[can_summon=isWorldGen] else sensor, waterlog iff fluid
  non-empty, return max(0, charge−50); penalty `(sqrt(distSqr)−1)²`, reach
  529, factor min(1, outer/529), max(1, (int)(charge×factor×0.5))); Default
  attemptUseCharge (decay_delay>0? charge: 0); Vein branch (spread_veins &&
  place → charge−1, else 1-in-5 floor(charge×0.5)); `updateCursors`
  (chessboard>1024 skip before update, charge≤0 drop, worldgen never
  merges); `getValidMovementPos` (shuffled 18 non-corner neighbours,
  SculkBehaviour block + unobstructed, `hasSubstrateAccess` early-break via
  #sculk_replaceable); `isMovementUnobstructed`; only SculkBlock and
  SculkVeinBlock implement SculkBehaviour in 26.1 (sensor/shrieker/catalyst
  do not), so the port's 3-way behaviour() is exact; `canSpreadFrom`
  matches (SculkBehaviour → true; else air or water-block+source + any
  neighbour collision-full).
- **Jigsaw ground_level_delta — VERIFIED faithful** (`JigsawPlacement$Placer`
  bytecode vs `jigsaw.rs`): delta_y = parentJigsawY − childJigsawY +
  facingStepY; target_y = parentMinY + delta when parent- AND child-rigid,
  else base_height − childLocalY; offset = target_y − rawMinY; child GLD =
  parentGLD − delta when child-rigid, else the element's own GLD; junction
  Y = parentMinY + localY (parent rigid), target_y + childLocalY (child
  rigid), else base_height + delta/2; both junctions' fields match
  (source_ground_y, ±delta, projections).
- **Ore feature — VERIFIED faithful** (`OreFeature` + `canPlaceOre`
  bytecode vs `ore_feature.rs`): angle nextFloat×π, spread size/8,
  max_radius ceil((size/16×2+1)/2), ends via Math.sin/cos, y0/y1 =
  origin.y + nextInt(3) − 2 in order, x/y/z starts and sizes, the
  OCEAN_FLOOR_WG height gate (first qualifying column calls doPlace),
  doPlace's lerp/step/radius — including the f32 π×step into the Mth.sin
  65536-entry table (`trig::sin`) and the sphere-culling loop, the BitSet
  index, `shouldSkipAirCheck` (discard ≤ 0 → skip, ≥ 1 → check, else
  nextFloat ≥ discard), and section-block writes.
- **Spawn finder search — VERIFIED faithful** (`Climate$SpawnFinder` /
  `ParameterPoint.fitness` bytecode vs `spawn_finder.rs`): origin sample,
  radial searches (2048, 512) then (512, 32), rings `increment` apart,
  x/z via Math.sin/cos × radius truncated to int, angle += increment/radius
  with 2π reset, fitness = min over targets of Σ square(distance) +
  square(offset), scaled by 2048² + x²+z² bias, quart-corner sampling.
  (The DECODE is broken — G12 — but the search itself matches.)
- **Chunk stages — VERIFIED**: carvers run inside `generate_proto`
  (`Pipeline::generate_with_context`, after noise fill + surface);
  `decorate` is a separate step over a 3×3 region. Basis for the B6-H
  STAGE RULE. Note: the doc comment says "up to TERRAIN" — 26.3
  terminology for what 26.1 calls NOISE/SURFACE/CARVERS; doc-only.
- **Flat preset loading — VERIFIED**: `FlatSettings` fields match jar
  `FlatLevelGeneratorSettings` (structureOverrides, layersInfo, biome,
  decoration, addLakes); the flat path's layer handling matches vanilla
  (MOTION_BLOCKING non-opaque → null layer, void_gen when all air, sea
  level −63). The flat path still goes through `Pipeline::new`, so G10
  blocks it in practice.
- **would_survive spot check — VERIFIED**: `block_survives` delegates to
  the sim's `can_survive` (block_survival.rs Rule classes). SnowLayer and
  SugarCane both match their jar `canSurvive` exactly (SugarCane: below is
  sugar cane, or below in #supports_sugar_cane with a horizontal neighbour
  of below having fluid/block in #supports_sugar_cane_adjacently).
- **Spring feature — VERIFIED faithful** (`SpringFeature.place` + codec
  bytecode vs `spring.rs`): above in validBlocks; requiresBlockBelow →
  below in validBlocks; origin air or validBlocks; rock/hole counts over
  the 5 neighbours (W/E/N/S/below, NOT above); rock==rockCount &&
  holes==holeCount → setBlock legacy fluid block flag 2 + scheduleTick
  fluid 0; codec defaults requires_block_below=true, rock_count=4,
  hole_count=1 match the port's. 7 spring configured features in the data.
- **Aquifer engine — VERIFIED faithful** (see B6-G03): the engine matches
  the 26.1 jar; only the wiring is missing (B6-G09).
- **API-shape conformance — PASS with a note**: no `densityfunction` /
  `material/rule` references in Rust worldgen code; the port's
  `ChunkStatus` enum is the 26.3 form (single TERRAIN, no
  NOISE/SURFACE/CARVERS) but behaviourally inert — `generate_proto`
  performs all three 26.1 stages in one call, and the only NBT use of the
  enum is retrogen `target_status` (`retrogen.rs:40`), which no
  reference-world chunk carries (0 chunks with retrogen data). The 26.2
  tree has the identical enum.

### Prior verification referenced (not re-diagnosed)

- Batch 0 oracle measurement (`docs/sweep/26.1/batch-0-oracles.json`,
  199/391 compile) is the oracle authority; first-error symbols pulled
  from the raw `big-compile-client.log`.
- The 26.2 sweep's verification of `aquifer/noise_based.rs` against the
  26.2 jar stands for the 26.1 file (byte-identical), spot-checked here
  against the 26.1 jar (ctor, 13 offsets, computeSurfaceLevel).
- The 1.21.7 world comparison (2026-11-22, 4,145 paired chunks) is the
  source of the G03 −1.04% figure; the 26.1 mechanism (total aquifer
  absence) is demonstrated here, not carried over.

## Open items (not defects)

- The 14 runnable + 16 no-oracle drivers were not run (execution block) —
  they are unmeasured, not passing. When the block lifts, the 14 runnable
  drivers are ready; the 33 blocked drivers need their oracles fixed first.
- B6-H is blocked until the construction cascade (G06/G10/G13/G11/G12) is
  fixed; no port world can be generated today, so no live measurement in
  this batch is possible beyond the vanilla-side census.
- The 26.2 tree's `feature_tags` prepare gap (line 1478 still
  `tags/worldgen/feature`) is noted under G06 for the 26.2 coordinator; not
  a 26.1 item.
- `dripstone_cluster` is missing in BOTH the 26.1 and 26.2 trees (both carry
  the 26.3 name `speleothem_cluster` instead) — the 26.2 coordinator may
  want to check whether 26.2's data uses it too.

## Adjacent findings (not B6 defects)

- **DUMMY block entities (68)** in the reference world — vanilla-side
  oddity, BEs owned by another batch.
- The `tag_states` silent-empty footgun (missing tag → empty set, no error)
  is the mechanism behind G07 and likely explains the 1.21.7 snow
  measurement; it is a robustness gap in the shared survival/tag loader,
  not a worldgen defect per se (all tags used by the verified snow path are
  present in the prepared data).
- The port's `ChunkStatus` enum (26.3 form) is noted under API-shape above;
  inert today.
- `generator.rs` tests still reference a `26.3` prepared-data dir
  (`../../.local/horizon-client/26.3`) in several test paths — test-only,
  not a behaviour defect.
- The 26.2 tree's `feature_tags` gap (see Open items) is the only
  cross-version item; everything else in this report is 26.1-specific.

## Method notes

- javap `-c -p` on the named 26.1 server-game jar for every vanilla method
  spot-checked (SculkPatchFeature, SculkBehaviour/SculkBlock/SculkSpreader/
  ChargeCursor, SnowAndFreezeFeature, Biome.shouldSnow/shouldFreeze/
  coldEnoughToSnow/getTemperature/getHeightAdjustedTemperature,
  SnowLayerBlock.canSurvive, SugarCaneBlock.canSurvive,
  JigsawPlacement/Placer, OreFeature/canPlaceOre, Aquifer$NoiseBasedAquifer
  incl. ctor/computeFluid/computeSurfaceLevel/calculatePressure,
  OverworldBiomeBuilder.isDeepDarkRegion, Climate$SpawnFinder/
  ParameterPoint/Parameter, MultiNoiseBiomeSourceParameterList Preset,
  SpringFeature/SpringConfiguration, FlatLevelGeneratorSettings,
  NoiseRouter, NoiseGeneratorSettings). Dumps in `evidence/javap-*.txt`.
- Exact data diffs of prepared resources vs jar data (715 populated
  registry entries; structure NBTs 1202/1202; flat presets 9/9; multi-noise
  presets). Jar data extracted to `evidence/jar-data/`.
- Vanilla 26.1 reference world census with an own NBT parser (read in
  place; no git commands inside vanilla-world-mc; no world files copied or
  moved). Census scripts + `world-26.1-census.json` in `evidence/`.
- Decode-coverage audit of every worldgen dispatch arm against the jar
  data (feature_graph.rs dispatch table is the authority for feature
  types, not string presence — several audit "ok" verdicts were false
  positives from block names). Script `evidence/decode_audit.py`,
  results `evidence/decode-audit.json`.
- Batch 0 oracle-status triage for all 57 oracles referenced by the
  batch's 63 drivers (23 pass / 34 fail). Driver list
  `evidence/drivers-26.1.json`.
- No driver was run, no cargo builds, no commits, no writes under the
  worktree. All artifacts under `~/workspace/mcir-work/sweep-26.1-batches/b06/`.

## Coordinator note

Reconciled 2026-11-24 against the measured Batch 0 table, the 26.1 tree, the
26.1 jars, and the vanilla 26.1 reference world. TALLY CONVENTION: this
batch counts BEHAVIOUR checks only (25 = 4 defect + 6 data + 10 code-vs-jar
+ 5 census; 18 pass, 7 gap) — oracle triage is separate (57 oracles, 23
pass / 34 fail; 63 drivers, 14 runnable / 16 no-oracle / 33 blocked). Do NOT
merge the oracle counts into the behaviour tally the way the 26.2 slice did.

"Fixed" on B6-G02/G04 means "1.21.7 defect withdrawn for 26.1 — verified
absent", not a fix applied during the sweep. B6-G03 is REPRODUCES for 26.1
with a demonstrated mechanism (total aquifer absence, B6-G09) — unlike 26.2,
where it was carried UNVERIFIED. B6-G01/G05 REPRODUCE; the 26.2 tree had
fixed G01 but not G05.

PER-ORACLE OVERLAPS (dedup at merge): VanillaFinalizeSpawnOracle folds into
B3, VanillaWorldSpawnOracle into B3, VanillaSculkOracle into B4 (26.2's
numbering); VanillaFloatLogOracle and VanillaStructurePlacementOracle are
group-only records with no B6 driver. Anchor blocks 9 drivers including
non-B6 ones.

SYSTEMIC NOTE: unlike 26.2, this batch HAS a single systemic cause across
most of its open defects: the 26.1 tree still carries the 26.3 forms where
26.1 uses the 1.21.x forms — the prepare folder names (G06), the NoiseRouter
(G10), the aquifer config object (G09), the material-rule name (G11), the
spawn-target named densities (G12), the density cache/shifted-noise arms
(G13), and 11 feature types + the nether carver (G14) were all fixed in the
26.2 tree and are all still 26.3-form in 26.1. The 26.2 tree is the repair
template for every one of them. The sculk defects (G01/G05/G07/G08) are the
exception: same narrow-predicate pattern as 26.2's G05, found only by
javap-diffing the feature's gates. Arm presence is not correctness — G05,
G07 and G08 were silent narrowings inside present arms.

Reference seed confirmed from world_gen_settings.dat (data.seed =
−1600712240839185950); census numbers verified against
evidence/world-26.1-census.json.
