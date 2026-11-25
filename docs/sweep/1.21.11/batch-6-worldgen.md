# Batch 6 — Worldgen / features (1.21.11 parity sweep)

Date: 2026-11-25. Version: 1.21.11 (DataVersion 4671, protocol 774).
Tree: `versions/1.21.11/` on branch `parity-sweep-1.21.11`
(`~/workspace/mcir-work/sweep-1.21.11`). 1.21.11 IS the E6 era
template — the tree every other E6 version derives from; the defects
found here are the template's own, and 1.21.10 (its first child) mostly
inherited them verbatim. Checks + triage ONLY; no fixes, no code
changes, no commits.

Jars (sha1-verified 2026-11-25, `e6-deltas/versions/1.21.11/jars/`):
`client-named.jar` 29bddf144a2f2907528b25d616770bfea67c4792 (10,291
classes), `server-game-named.jar` 7f0d85c12aeb57f14687b7ab2ae2e5d825fbf0d2
(7,163 classes). `server.jar` is a bundler — never javap'd. All vanilla
claims below are verified against 1.21.11's OWN jar/data, never 26.3,
1.21.10 or 1.21.7 forms.

## §0 Execution status

Batch 0 has NOT landed for 1.21.11 (`b00/` holds only h01 files; no
`docs/sweep/1.21.11/batch-0-oracles.json`). Oracle triage is therefore
provisional, built from the 1.21.10 measured Batch 0 table
(`sweep-1.21.11-ref/1.21.10/batch-0-oracles.json`) plus fresh static
and javap verification against the 1.21.11 jar, plus targeted
single-file javac checks of every status-questioned oracle (see the
Oracle defects section). The javac checks are server-classpath only;
the client-classpath column is NOT measured here — Batch 0
re-verification is still owed when it lands.

No prepared 1.21.11 horizon-client `.local` exists anywhere on the VM
(checked candidate-test-1.21.11, e6-1.21.11-behaviour, 1.21.11/*,
e6-deltas, rewire/fix/adapters). Consequence: the datadiff is jar-side
only; the G06 bundle-emptiness claim is INFERRED from prepare code
identical to 1.21.10's (whose bundles were measured 0/0/0), and is
stated as inferred, not measured, for 1.21.11.

Static checks only: no cargo builds, no driver runs, no commits, no
git anywhere (coordinator owns all git). The reference world was read
in place with an own NBT parser; no world files were copied or moved.

## Tally (behaviour checks only)

**37 checks: 22 pass, 15 gap.**

Behaviour/non-behaviour split: **37 behaviour / 0 non-behaviour**
(as-tallied; `checks.total` is the as-tallied count). Oracle triage and
driver-status work are NOT behaviour checks and are counted separately
below.

| Group | Total | Pass | Gap |
|---|---|---|---|
| PRIMARY G re-checks (1.21.10 items) | 18 | 3 (fixed) | 15 |
| Jar-side data diffs | 6 | 6 | 0 |
| Code-vs-jar spot checks | 8 | 8 | 0 |
| Reference-world census | 5 | 5 | 0 |
| **Total** | **37** | **22** | **15** |

Items minted: **28** — 18 G (15 gap + 3 fixed), 9 O groups
(oracle-defect), 1 H (needs-harness).

Oracle counts (separate from the behaviour tally): **57 oracles, 18
pass / 39 fail** (1.21.10: 17/40 — VanillaChunkGenerationOracle flips
to pass). Drivers (64, counted fresh from `versions/1.21.11/tools/`):
**12 runnable / 38 blocked / 14 no-oracle** (1.21.10: 8/42/14 — the 4
drivers blocked solely by VanillaChunkGenerationOracle flip to
runnable).

## Defects

### B6-G01 (major, 1.21.7 defect — REPRODUCES) — sculk_patch catalyst / extra rare growths tail missing

`crates/horizon-world/src/worldgen/sculk.rs` contains no `catalyst`
string and decodes only 5 fields of the sculk patch configuration.
Vanilla 1.21.11 `SculkPatchFeature.place` rolls `catalystChance` →
SCULK_CATALYST, then `extraRareGrowths.sample` → SCULK_SHRIEKER
(`javap-sculkpatch.txt:88-148`); `SculkPatchConfiguration` has 7 fields
including `extraRareGrowths` (IntProvider) and `catalystChance` (float)
(`javap-sculkpatchconfig.txt`). The port therefore never places a
catalyst or a shrieker from a sculk patch. Note: the 1.21.11 reference
world contains NO sculk blocks at all (deep_dark only 1,895 cells), so
the vanilla ground truth here comes from the jar only, not the census
— unlike 1.21.10, whose world held 25 catalysts + 12 shriekers.

### B6-G02 (major, 1.21.7 defect — REPRODUCES) — snow survives on packed ice

The port reads the 26.x snow-tag names at `block_survival.rs:598-599`,
`level/data.rs:431-432`, `level/survive.rs:314,316`,
`level/weather.rs:113,116`. Vanilla 1.21.11 renamed them:
`SnowLayerBlock.canSurvive` reads `SNOW_LAYER_CANNOT_SURVIVE_ON` /
`SNOW_LAYER_CAN_SURVIVE_ON` (`javap-snowlayer.txt:128,134`), and the
jar carries `snow_layer_can_survive_on` = [honey_block, soul_sand,
mud] and `snow_layer_cannot_survive_on` = [ice, packed_ice, barrier]
(`datadiff.log`); the 26.x names (`cannot_support_snow_layer`,
`support_override_snow_layer`) are ABSENT. The tag loaders load
silent-empty, so snow survives on packed ice (and fails to survive on
honey/soul sand/mud). Live confirmation is vacuous for this world:
the census found 0 snow blocks and 72 packed_ice, and no frozen-peaks
biome — the verdict rests on code + javap.

### B6-G03 (major, 1.21.7 defect — REPRODUCES) — underground water deficit; the entire aquifer system is absent

Reproduces via G09 (total aquifer absence). Reference-world water is
5,729,767 blocks; no percentage is claimed (no paired port world
exists). The full aquifer audit is in G09.

### B6-G05 (minor, 1.21.7 defect — REPRODUCES) — sculk spreader water gates narrowed to water sources

`sculk.rs:305,548` require WATER_SOURCE. Vanilla accepts any water:
`SculkBehaviour$1` bytecode (49-66) accepts air OR
`fluidState.is(Fluids.WATER)`; `SculkBlock.canPlaceGrowth` is the
same. The live code in `level/sculk.rs` (~611, ~385) shows the same
pattern — flagged for B4, not minted here.

### B6-G06 (major — REPRODUCES) — prepared `carvers.json` and `features.json` are EMPTY; `GeneratorData::load` fails

`WORLDGEN_BUNDLES` at `horizon-prepare/src/lib.rs:1479-1499` maps
carvers → `worldgen/carver`, features → `worldgen/feature`,
feature_tags → `tags/worldgen/feature` — 26.3 folder names; the
1.21.11 jar has no such folders (carvers live at
`worldgen/configured_carver`, features at
`worldgen/configured_feature`). `GeneratorData::load` reads bundles
only: `generator.rs:486` (carvers catalog), `:497`
(`decode_set(...)?` fails for 59/65 biomes, whose carver lists name
carvers the empty bundle cannot resolve). The 1.21.11 prepare code is
identical to 1.21.10's except the jsr305 classpath helper (present in
1.21.10, absent in 1.21.11); 1.21.10's bundles were MEASURED 0/0/0.
Bundle emptiness for 1.21.11 is therefore INFERRED, not measured — no
prepared 1.21.11 `.local` exists on the VM to measure against.

### B6-G07 (minor — REPRODUCES) — sculk `can_place_growth` density cap is dead (26.3-only tag)

`sculk.rs:108` reads `minecraft:sculk_growth_inhibitors`, which is
ABSENT from the jar (`datadiff.log`). Vanilla counts SCULK_SENSOR /
SHRIEKER directly in offset(-4,0,-4)..(4,2,4) and refuses growth when
the count exceeds 2 (`javap-sculkblock.txt:195-250`). The port's own
live code (`level/sculk.rs`) counts directly, with the comment "no
growth-inhibitors tag in 1.21.11" — the worldgen copy was never
updated to match.

### B6-G08 (minor — REPRODUCES) — sculk cursor movement uses the 26.3 form

`sculk.rs:263-270` kills a stuck cursor (None → on_discharged +
charge=0 + return) and applies a pre-move 12-block filter in
`valid_movement` (`:659-663`, dx²+dz² > 144), with no post-move
15-block check. Vanilla: a null target falls through
(`javap-chargecursor.txt:206`, `ifnull 272`) and the cursor is killed
post-move only when NOT `closerThan(..., 15.0d)` (`:233-234`).

### B6-G09 (major — REPRODUCES) — aquifers are entirely disabled (26.3 config-object form)

`generator.rs:1115` reads `.get("aquifers")` — the 26.3 config object.
All 7 noise settings carry `aquifers_enabled` only (verified in
`datadiff.log`; NO `aquifers` key in any setting). The aquifer engine
itself is faithful (see Verified passes: `noise_based.rs` is
byte-identical to 1.21.10's, and the vanilla `Aquifer` ctor takes a
NoiseRouter — `javap-aquifer.txt:94-99`); only the wiring is dead.
Consequence: no aquifer water anywhere in a generated world (G03).

### B6-G11 (major — REPRODUCES) — `Pipeline::new` requires `material_rule`; 1.21.11 settings carry an inline `surface_rule`

`generator.rs:1136-1138` requires `settings_json["material_rule"]`.
All 7 settings carry an inline `surface_rule` only (`datadiff.log`).
Pipeline construction fails before any chunk generates.

### B6-G12 (major — REPRODUCES) — spawn_target decode fails (named-density lookups); depth not zeroed

`generator.rs:1141-1152` maps spawn-target fields through
`library.named_density`, which errors on the unregistered bare field
names the settings actually carry (overworld/amplified/large_biomes
spawn targets use 7 bare field names). Latent half: vanilla
`Climate$SpawnFinder` rebuilds the TargetPoint with depth FORCED to 0
(`lconst_0`, `javap-spawnfinder.txt:124`) plus `Mth.square(offset)`.
The radial search itself matches the jar.

### B6-G13 (major — REPRODUCES) — density decode missing seven types

`density.rs` has NO arms for cache_2d, cache_once, flat_cache,
shifted_noise, y_clamped_gradient, weird_scaled_sampler,
end_islands; the default errors `unknown density type` (`:476`). All
seven are used in 1.21.11 data: density functions (35 files) use
cache_2d ×11, cache_once ×12, flat_cache ×16, shifted_noise ×5,
y_clamped_gradient ×6, weird_scaled_sampler ×3, end_islands ×1, and
the noise settings add cache_2d ×13, shifted_noise ×12,
y_clamped_gradient ×23, end_islands ×1 (`recursive-census.log`).
`minecraft:y` IS y_clamped_gradient (-4064/4062). There is no
`minecraft:gradient` / `minecraft:cache` type in the data.

### B6-G14 (major — REPRODUCES) — 11 configured-feature types + `coral_mushroom` + `nether_cave` have no decode arms

`feature_graph.rs` has NO arms for the 11 missing types (default
`:812` → BasicFeature::from_json) and carries the 26.3 name
`minecraft:speleothem_cluster` (`:633`) instead of
`dripstone_cluster`. `coral_mushroom` errors in
`vegetation.rs:293-295` (coral_tree|coral_claw only). `nether_cave`
is missing in `carver/generation.rs:26-27` (cave + canyon only) — 5
nether biomes name it as their sole carver. Blast radius: 21 broken
configured features (11 missing types) + coral_mushroom = 22; 24
placed features affected (`recursive-census2.log`,
`decode-audit.json`).

### B6-G16 (major — REPRODUCES) — `random_offset` placement modifier has no decode arm

0 occurrences of `random_offset` / `RandomOffset` in horizon-world.
10 placed features affected (same list as 1.21.10), plus 2× nested in
configured `pointed_dripstone`.

### B6-G17 (minor — REPRODUCES) — worldgen numeric pipeline is f32 where vanilla 1.21.11 computes in f64

`GraphNode::Constant(f32)` (`horizon-core/src/density/graph/mod.rs:64`),
gradient/clamp/choice bounds f32; `climate.rs`, `blending.rs`,
`biome_temperature.rs` f32; `PerlinSimplexNoise` has 0 occurrences in
horizon-core. Vanilla: `DensityFunction.compute` returns double,
`fillArray(double[], ...)` (`javap-densityfunction.txt:9,11`);
`BlendingData.NO_VALUE` is double (`javap-blendingdata.txt:33`);
`Biome` uses PerlinSimplexNoise `getValue(DDZ)D`
(`javap-biome.txt:119`).

### B6-G18 (major — REPRODUCES; originates in the template) — ore veins never generate (no OreVeinifier path)

0 occurrences of OreVeinifier / ore_veins_enabled / vein_toggle /
veinToggle in horizon-world. Vanilla: `NoiseChunk` calls
`OreVeinifier.create(...)` gated on `oreVeinsEnabled()`
(`javap-noisechunk.txt:691-702`). The port HAS
`MaterialRule::OreVein` (`rule.rs`, `library.rs:153` arm) but it is
driven only by `minecraft:ore_vein` rule nodes, which no 1.21.11 data
contains. All 7 settings carry `ore_veins_enabled` and the router
carries vein_toggle / vein_ridged / vein_gap — none of it is read.
The gap originates HERE, in the template; 1.21.10 inherited it.

Cascade (same as 1.21.10): G06 → G13 → G11 → G12, with G09 silent
behind. No 1.21.11 world can generate today.

## Fixed / withdrawn defects (verified absent in 1.21.11)

### B6-G04 (minor) — FIXED: PoolPiece NBT writes `GD`

`jigsaw.rs:682-683` save → `legacy::piece_tag(..., 0)`;
`legacy.rs:339` writes `("GD", Tag::Int(gen_depth))`. The census
confirms the vanilla form (all jigsaw pieces GD=0; mineshafts 0-9).
Fixed in the template itself.

### B6-G10 (major) — FIXED (router): `preliminary_surface_level` fills the surface field; vein-field residual is G18

`router.rs:57-62` falls back to `preliminary_surface_level` when
`chunk_surface_level` is absent — the fallback EXISTS in the template
(the child inherited it, not the reverse). Vanilla's NoiseRouter is a
15-arg record (`javap-noiserouter.txt:35`) with NO
chunk_surface_level; all 7 settings carry
`preliminary_surface_level`. The vein-field residual is G18.

### B6-G15 (major) — FIXED: state-provider decode accepts the long-form names

`state_provider.rs` `decode_typed` (`:333-456`) accepts BOTH spellings
for every provider used (simple/weighted/noise/noise_threshold/
dual_noise/rotated/randomized_int + `_provider` forms); rule-based
providers are decoded separately (`:505-518`), used by `disk.rs:65`.
Fixed in the template itself. 113/224 configured features contain a
long-form `_provider` (`recursive-census.log`).

## Oracle defects (provisional — Batch 0 has NOT landed for 1.21.11; 57 oracles, 18 pass / 39 fail)

Triage method: the 1.21.10 measured Batch 0 table gives the starting
status for all 57 B6 oracles (17 pass / 40 fail). Every oracle whose
status could have changed was re-verified fresh: the 6 oracles with
real source diffs vs 1.21.10, the 5 rename-only passing oracles, and
the 12 byte-identical passing oracles were all javac-checked against
the 1.21.11 server classpath (JDK 25.0.5, `javac --release 21
-proc:none -sourcepath <parity dir>`, server-game-named.jar + 99
server library jars + jsr305; logs in `evidence/javac-check/`). The
remaining 27 failing oracles carry their 1.21.10 measured status,
re-verified statically against the 1.21.11 jar (the packages/symbols
they need are still absent). **Lesson: compile WITH `-sourcepath`** —
without it, oracles referencing sibling oracles falsely fail
(VanillaBlockSurvivalOracle "failed" with 4 sibling-symbol errors
without sourcepath, PASSES with it).

Passing (18, all javac-verified on the server classpath): ParityData,
VanillaBiomeExport, VanillaBiomeTemperatureOracle,
VanillaBlockPropertiesOracle, VanillaBlockStateCodecOracle,
VanillaBlockSupportOracle, VanillaBlockSurvivalOracle,
**VanillaChunkGenerationOracle (NEW pass — see below)**,
VanillaChunkStatusOracle, VanillaCollisionOracle,
VanillaGatewayReplacementOracle, VanillaHeightmapOracle,
VanillaPlacementOracle, VanillaRetrogenDataOracle,
VanillaSimpleBlockOracle, VanillaTagExport, VanillaWorldSpawnPacketOracle,
VanillaWorldgenRandomOracle.

The flagship flip: **VanillaChunkGenerationOracle PASSES** (exit 0, 0
errors, 7 class files). It was thoroughly and CORRECTLY adapted for
1.21.11: WORLDGEN/WORLDGEN_REGISTRIES/DIMENSION_REGISTRIES 3-arg sync
load, `RandomState.create(settings, noise, seed)`, 4-arg
createForNormal, NOISE/SURFACE/CARVERS + fillFromNoise/buildSurface/
applyCarvers, ServerLevel 11-arg with RandomSequences, ServerChunkCache
11-arg, and it removed getNextEntityId / uncachedBiomeResolver /
NoiseRouterData.none() / TERRAIN. This flips 4 drivers to runnable
(chunk_generation, datapack_worldgen, flat_generation,
structure_layout).

The other five real-diff oracles were adapted but still fail:
VanillaFinalizeSpawnOracle (removes DATA_SOUND_VARIANT_ID reads for
pig/cow/chicken/cat — only wolves have them in 1.21.11; still fails on
the ServerLevel ctor), VanillaMaterialStageOracle (`ChunkPos.asLong`
→ `ChunkPos.pack` — WRONG for 1.21.11, `pack` does not exist),
VanillaNaturalSpawnOracle and VanillaSpawnRulesOracle (adapted toward
1.21.11 GameRules + EnvironmentAttributeSystem, but wrong in places:
Lvl super 10 args where the jar wants 11, re-added `@Override
getNextEntityId` which does not exist, `.addStaticLayers(level)`
which does not exist, OracleServer super 10 args where DedicatedServer
wants 7, Map2 super 12 args where ChunkMap wants 13),
VanillaStructurePlacementOracle (`asLong` → `pack` only — wrong for
1.21.11).

Of the 57 B6 oracles, 27 are byte-identical to 1.21.10's, 24 differ by
the ResourceLocation → Identifier rename only, and 6 have real diffs
(the five above + ChunkGeneration). The 12 byte-identical PASSING
oracles were grepped for every changed 1.21.11 API (GameRules move,
Biome special-effects getter removal, getNextEntityId,
markPosForPostProcessing, ChunkStatus.TERRAIN, RegistryLayer.WORLD,
WORLD_REGISTRIES, NoiseRouterData.none, densityfunction,
material/rule, ChunkPos.pack, gamerules/gizmos/world.clock) and are
clean; all 12 also passed a single-invocation javac check (exit 0).

### B6-O01 (major) — Oracles reference the 26.3-only `densityfunction` package (7 oracles)

VanillaAnchorOracle, VanillaBiomeSourceCodecOracle,
VanillaBiomeSourceOracle, VanillaFloatLogOracle*,
VanillaMaterialExtensionOracle, VanillaStructureDensityOracle,
VanillaWorldgenOracle. First errors from the measured 1.21.10 table,
re-verified against the 1.21.11 jar (package still has 0 entries):
Anchor :14, BiomeSourceCodec :12, BiomeSource :11, FloatLog :5
(densityfunction.op), MaterialExtension :17, StructureDensity :8,
Worldgen :132 (SamplerContext use). `VanillaAnchorOracle` blocks 7
drivers (biome_source_codec, canyon_carver, cave_carver,
height_provider, material_condition, material_rule_library,
material_stage). `VanillaStructureDensityOracle` imports the 26.x-only
densityfunction API (`SamplerContext`/`DensityVolume`) — record as an
oracle defect, UNREPAIRABLE by design (the API does not exist in
1.21.11), do not force. (*FloatLog is group-only: no driver references
it.) Three oracles LEFT this group vs the 1.21.10 report: Density
(first error is the Interval import, :11), MaterialSurface (first
error is the MaterialSystem import, :16) and WorldSpawn (first error
is the NoiseSpawnFinder import, :23) — all three move to O04 on
measured first-error evidence; the 1.21.10 report's O01 placement of
MaterialSurface was based on a misread first error.

### B6-O02 (major) — Oracles reference the missing `material/rule` package (4 oracles)

VanillaBiomeGenerationOracle, VanillaMaterialConditionOracle,
VanillaMaterialStageOracle, VanillaTerrainOracle. First errors:
BiomeGeneration :18, MaterialStage :22 (javac-verified for 1.21.11:
100 errors with -sourcepath, first material/rule :22 then
densityfunction :21), Terrain :21 — all `package ...material.rule
does not exist`; MaterialCondition :27 is a use of a material.rule
type (`MappedRegistry<MaterialCondition>`). The package has 0 entries
in the 1.21.11 jar (surface rules live in `levelgen.SurfaceRules`).

### B6-O03 (major) — Oracles use the 26.3 `Feature` constructor signature (3 oracles)

VanillaDecorationOracle, VanillaBiomePlacementOracle,
VanillaFeaturePlacementOracle. All 3 still use `new Feature(){...}`
(no-arg anonymous subclass): DecorationOracle:48, BiomePlacementOracle:39
and :51, FeaturePlacementOracle:33. The 1.21.11 jar's `Feature` ctor
takes only `Codec<FC>` (fresh javap, `javap-feature.txt`); there is no
no-arg ctor, and `Feature` has NO `DIRECT_CODEC`
(`ConfiguredFeature` has it). Measured first errors (constructor-
mismatch): :48, :39, :33.

### B6-O04 (major) — Oracles fail on missing symbols / signature drift (15 oracles)

VanillaBasicFeatureOracle, VanillaBiomeChunkOracle,
VanillaCanyonCarverOracle, VanillaCaveCarverOracle,
VanillaChunkStageOracle, VanillaCushionPlacementOracle,
VanillaDensityOracle, VanillaFeatureGraphOracle,
VanillaFeatureRegionOracle, VanillaFeatureSortOracle,
VanillaMaterialSurfaceOracle, VanillaNoiseOracle,
VanillaScatteredOreOracle, VanillaSculkOracle, VanillaWorldSpawnOracle.
First-error symbols, re-verified against the 1.21.11 jar:
BasicFeature/FeatureSort/ScatteredOre/FeatureGraph —
`Feature.DIRECT_CODEC` (absent; `ConfiguredFeature.DIRECT_CODEC`
exists). BiomeChunk — lambda parameter types at :37 (measured first
error; the 1.21.10 report's claim that the failure is the
fillBiomesFromNoise arity is contradicted by the measured table — the
lambda errors first, the 1-arg `fillBiomesFromNoise(resolver)` arity
failure at :45 is second; the jar's signature takes
`BiomeResolver, Climate$Sampler`). CanyonCarver/CaveCarver —
`WorldCarver.DIRECT_CODEC` (absent; only `configuredCodec()`),
`CarvingMask` set/get/stream/toArray only (no minY/maxY/carve).
ChunkStage/FeatureRegion — `ChunkStatus.TERRAIN` (the 1.21.11 set has
12 statuses, no TERRAIN; both oracles still reference it:
ChunkStage:27, FeatureRegion:77-78,154). CushionPlacement — the
`blockscan` package (0 entries) AND `Cushion` itself (0 entries; a
26.x entity). Density — `net.minecraft.util.Interval` import (:11;
0 entries in the jar). MaterialSurface —
`levelgen.material.MaterialSystem` import (:16; `levelgen/material/`
holds only MaterialRuleList, WorldGenMaterialRule, package-info).
Noise — `GradientNoise`/`Noise`/`NoiseStack` (synth has only
BlendedNoise/ImprovedNoise/NoiseUtils/NormalNoise/PerlinNoise/
PerlinSimplexNoise/SimplexNoise). Sculk — javac-verified for 1.21.11:
18 errors, first `package net.minecraft.world.clock does not exist`
(:66; the 1.21.10 gizmos-package error is GONE, replaced by a NEW
clock-package failure), then `randomlyDelayStart(RandomSource)` (:281),
`DifficultySettings` (:322), `datapackWorldRegistries()` (:327;
`DataLoadContext` has `datapackWorldgen()` only), PrimaryLevelData
ctor (:330), MinecraftServer ctor (:356), tickServer/onServerExit/
onServerCrash override failures (:394/:759/:760), WardenSpawnTracker
Optional wrapping (:633). Cross-batch overlap: Sculk→B4. WorldSpawn —
`NoiseSpawnFinder` import (:23; 0 entries in the jar), with
densityfunction and RegistryLayer.WORLD as secondary causes.
Cross-batch overlap: WorldSpawn→B3.

### B6-O05 (major) — Oracles fail on missing-override lookup (2 oracles)

VanillaDecorationRegionOracle, VanillaFeatureCatalogOracle. Both
sources differ from 1.21.10's ONLY by the ResourceLocation→Identifier
rename, so the measured 1.21.10 failure modes carry verbatim:
DecorationRegion :49 (anonymous class does not override abstract
`lookup(ResourceKey)` in RegistryInfoLookup); FeatureCatalog :31
(`Feature.CODEC` — no such field in 1.21.11 — plus the Feature-ctor
error and a missing `place(FeaturePlaceContext)` override).

### B6-O06 (major) — Oracles fail on no-override (1 oracle)

VanillaTreeFeatureOracle only. Line 67 still declares `@Override
public void markPosForPostProcessing(...)` with a
`super.markPosForPostProcessing(...)` call — the 1.21.11 method is
`markPosForPostprocessing` (lowercase p), so both the @Override and
the super call fail (measured first error :67, class no-override,
n=16). VanillaFinalizeSpawnOracle LEFT this group: its 1.21.11 first
error is the ServerLevel ctor (:97), with the getNextEntityId
no-override (:119) and startsForStructure no-override (:172) now
second — see O09.

### B6-O07 (major) — Other oracle failures (2 oracles)

VanillaBiomeZoomOracle, VanillaHeightProviderOracle (cascade via
Anchor). BiomeZoom: calls `.getBiome(x, y, z)` with three ints (line
22); the 1.21.11 `BiomeManager` has only `getBiome(BlockPos)`
(verified by javap; measured first error :22, n=1). HeightProvider:
cascade via Anchor (Anchor still broken — O01). StructureStart and
StructurePlacement LEFT this group: StructureStart's first error is
RegistryLayer.WORLD (:45) — see O08; StructurePlacement's first error
was ALREADY the ServerLevel ctor (:55) in the measured 1.21.10 table
(the 1.21.10 report's O07 placement rested on the provisional
RandomState.create reading, not the measured first error) — see O09.

### B6-O08 (major) — Oracles use `RegistryLayer.WORLD` / `WORLD_REGISTRIES` / `RandomState.create` order absent from 1.21.11 (1 oracle)

VanillaStructureStartOracle only (javac-verified for 1.21.11: 5
errors — WORLD :45, WORLD_REGISTRIES :46, WORLD :48,
`RandomState.create(Registry, long, Settings)` no-suitable-method
:51). CORRECTION to the 1.21.10 report: its framing ("1.21.11-template
API absent in 1.21.10") was wrong for these symbols — they exist in
NEITHER version. In 1.21.11, `RegistryLayer` has STATIC/WORLDGEN/
DIMENSIONS/RELOADABLE only (no WORLD),
`RegistryDataLoader` has WORLDGEN/DIMENSION/SYNCHRONIZED only (no
WORLD_REGISTRIES), `RandomState.create` has only 2 overloads (no
`(Registry, long, Settings)`), `NoiseRouterData.none()` is PROTECTED,
`BiomeSource.createUncachedResolver` and `Level.uncachedBiomeResolver`
are absent (all verified by javap against the 1.21.11 jar). The other
two 1.21.10 O08 members are gone: ChunkGeneration PASSES in 1.21.11,
and NaturalSpawn/SpawnRules now fail first on the ServerLevel ctor —
see O09. 17 oracle sources in the 1.21.11 tree use
RegistryLayer.WORLD/WORLD_REGISTRIES, 11 of them outside B6 scope
(AudioAttributes, BlockUpdate, ChatScreen, Enchanting,
EntityDeathLoot, Environment, LootFill, MobAttributes, Raid,
SituationalMusic, TradeComponents) — flagged for the coordinator.

### B6-O09 (major — NEW to 1.21.11) — Oracles fail on constructor arity mismatch: ServerLevel 11-arg (4 oracles)

VanillaFinalizeSpawnOracle, VanillaNaturalSpawnOracle,
VanillaSpawnRulesOracle, VanillaStructurePlacementOracle*.
All four first-fail on the ServerLevel constructor: the oracles
construct ServerLevel with 10 args; the 1.21.11 ctor takes 11, ending
in RandomSequences (verified by javap). javac-verified for 1.21.11
(all with -sourcepath): FinalizeSpawn 13 errors, first :97 (gamerules-
package error GONE — the package exists in 1.21.11); NaturalSpawn 30
errors, first :141 (also no-override :191, `ChunkPos.pack()` :273 —
`pack` does not exist in 1.21.11, only `asLong`/`toLong`);
SpawnRules 44 errors, first :61 (also no-override :81, :111);
StructurePlacement 17 errors, first :55 (also WORLD :75/:76).
Cross-batch overlap: FinalizeSpawn→B3. (*StructurePlacement is
group-only: no driver references it.)

## Needs-harness

### B6-H (minor) — Paired port-vs-vanilla chunk comparison for 1.21.11

Generate port chunks at seed -76134417269081163 (the reference
world's seed, confirmed from level.dat's WorldGenSettings) and
compare against the vanilla reference world
(`~/workspace/vanilla-world-mc/1.21.11/New World/`, 9,093 chunks, 15
regions). BLOCKED on G06/G13/G11/G12 — no port world can generate
until the construction cascade is fixed. Stage rule: the port's
`generate_with_context` runs fill_noise → build_surface_shared →
carvers → heightmaps (Terrain stage) with `decorate` separate over a
3×3 region (`generator_decoration.rs:396`); compare stage-matched
chunks only (terrain-level port output vs post-decorate vanilla is a
measurement artifact, not a defect — the 1.21.7 −6.4 beardifier
lesson). Caveats specific to this world: the snow check needs a
frozen-peaks seed (this world has no snow and no frozen-peaks biome);
sculk ground truth is absent from this world (0 sculk blocks) — G01
verification needs a deep-dark seed or jar-side fixtures.

## Verified passes

### Data (exact diffs vs the jar)

Jar-side datadiff only (`evidence/datadiff.py` → `datadiff.log`): no
prepared 1.21.11 `.local` exists, so there is no prepared side to
diff. Registry counts match the inventory exactly: biomes 65,
configured carvers 4, configured features 224, density functions 35,
noises 60, noise settings 7, placed features 258, processor lists 40,
structures 34, structure sets 20, template pools 188, world presets
6 (amplified, debug_all_block_states, flat, large_biomes, normal,
single_biome_surface) — total 952. Structure NBTs: 1,202. Flat
presets: 9 (bottomless_pit, classic_flat, desert, overworld,
redstone_ready, snowy_kingdom, the_void, tunnelers_dream,
water_world). Multi-noise lists are presets
(`minecraft:nether`, `minecraft:overworld`) — the parameter points
are code-defined in MultiNoiseBiomeSourceParameterLists (the nether
preset's values were verified from javap: identical to 1.21.10's,
−0.5/0.4/0.5+0.375/−0.5+0.175). Biome carver lists: 54×
[cave, cave_extra_underground, canyon], 5× single `minecraft:nether_cave`
(a bare string, not a list), 6× empty. All 7 noise settings carry
`ore_veins_enabled`, `aquifers_enabled`, `surface_rule`,
`spawn_target`; NO `aquifers`, NO `material_rule`, ore_vein ×0.
Routers: 15 keys including vein_toggle/vein_ridged/vein_gap +
preliminary_surface_level, NO chunk_surface_level. Server-jar data
verified byte-identical to client data (diff exit 0). No
`data/minecraft/dimension/` or `environment_attribute/` dirs in
either jar (environment attributes are code-defined); the 4
`dimension_type/*.json` DO carry an `"attributes"` map (e.g.
`minecraft:audio/*`, `minecraft:gameplay/bed_rule`,
`minecraft:visual/*`).

### Vanilla 1.21.11 reference world (own NBT parser, read in place)

Seed **-76134417269081163 confirmed** from level.dat
WorldGenSettings; Level "New World", DataVersion 4671, spawn [0,72,0],
GameType 1, Time 2355; datapacks: vanilla only enabled. 15 regions,
**9,093 chunks**, x −51..49, z −49..50. Total blocks 893,878,272 (=
9093×24×4096 exactly). Statuses are NAMESPACED in 1.21.11 (unlike
1.21.10): `minecraft:full` 5,114; `structure_starts` 2,960; `biomes`
336; `noise` 9; `carvers` 333; `initialize_light` 335; `features` 1;
`light` 5. No TERRAIN. 18 biomes present; NO frozen_peaks / grove /
snowy_slopes / jagged_peaks. Snow: 0 blocks; packed_ice: 72. Water:
5,729,767. **Sculk family: {} — no sculk blocks at all** (deep_dark
only 1,895 cells). Structures: mineshaft 26/3,725 pieces (GD 0–9),
trial_chambers 9/2,346 (GD all 0, ground_level_delta −48..+20),
jungle_pyramid 1, ruined_portal 4, trail_ruins 1/19 (GD 0,
ground_level_delta −12..+12), ocean_ruin_cold 4/12, ocean_ruin_warm
1, ruined_portal_jungle 1, buried_treasure 6, village_savanna 1/93
(GD 0, ground_level_delta −1..+2), shipwreck 1. All non-mineshaft
pieces GD=0. Block entities: chest 242, mob_spawner 180,
decorated_pot 180, dispenser 126, vault 118, brushable_block 108,
barrel 78, trial_spawner 75, hopper 43, bed 30, beehive 15. No DUMMY
BE (1.21.10 had 1). 0 chunks with blending_data/retrogen. First
palette entry `{'Name': 'minecraft:bedrock'}` (Name/Properties form —
Batch 11 note). Full census: `evidence/world-1.21.11-census.json`,
`evidence/census.log`.

### Code-vs-jar spot checks (javap + source)

1. **Environment attributes (PASS — the 1.21.10 pattern inverts
   here).** 1.21.11 HAS the `minecraft:environment_attribute`
   registry natively (45 entries, NEW vs 1.21.10). The port's
   `dimension_type.rs:257-258` decodes the `"attributes"` map from
   dimension_type JSON, and a full `EnvironmentAttributeMap`
   implementation exists (`environment_attributes.rs`).
   `Level.environmentAttributes()` exists in the jar (abstract);
   `EnvironmentAttributeSystem.builder()` exists with
   `addConstantLayer` only (no `addStaticLayers` — the oracles'
   call is wrong, see O09).
2. **Chunk stages (PASS).** `generate_with_context` runs fill_noise
   (`generator.rs:1800`) → build_surface_shared (`:1820`) → carvers
   → heightmaps, Terrain stage; `decorate` is separate over a 3×3
   region (`generator_decoration.rs:396`). Matches vanilla's stage
   order; this is the B6-H stage-rule basis.
3. **Aquifer engine (PASS).** `aquifer/noise_based.rs` is byte-
   identical to 1.21.10's; the vanilla `Aquifer` bytecode is
   identical modulo one lossless call-shape change
   (`MutableDouble.getValue().doubleValue()` → `doubleValue()`).
   Only the wiring is dead (G09).
4. **Ore feature (PASS).** The port uses `jmath::sin(f64)`, matching
   1.21.11's `Mth.sin(double)` (the 1.21.10→1.21.11 orefeature diff is
   one lossless added `f2d` cast).
5. **Snow/freeze placement (PASS).** `snow_freeze.rs` is byte-
   identical to 1.21.10's: MOTION_BLOCKING height, `biome_should_freeze`
   → ice below, `biome_should_snow` → snow on top + snowy state
   below — matching `SnowFreezeFeature.place`
   (`javap-snowfreeze.txt`). Vanilla's `Biome.shouldFreeze`/
   `shouldSnow` exist with the same signatures as 1.21.10.
6. **Tree decode (PASS).** All 21 trunk/foliage placer types used in
   1.21.11 data decode "ok" (`decode_audit.log`); the 22 MISSING
   entries are exactly the G13/G14/G16 defects, nothing else.
7. **Biome weather methods (PASS).** All 6 weather methods are
   bytecode-identical 1.21.10→1.21.11 (normalised javap diff: 0
   lines); the biome diff is only the new `EnvironmentAttributeMap`
   field / 5-arg ctor and the removed special-effects getters.
8. **Jigsaw ground_level_delta round-trip (PASS).** `jigsaw.rs:611`
   field, `:674` reads `int(v, "ground_level_delta")`, `:704` writes
   `("ground_level_delta", Tag::Int(...))` — the same long-form key
   vanilla writes (the census reads `ground_level_delta`, not "GLD";
   vanilla's `StructurePiece` bytecode confirms the "GD"/"BB" short
   keys alongside it). GD writes 0 via `legacy::piece_tag` (G04).

## §5 Re-check verdicts

### 1.21.10 items (PRIMARY — all 27 decided)

| 1.21.10 item | Verdict | 1.21.11 item |
|---|---|---|
| B6-G01 sculk_patch tail | REPRODUCES | B6-G01 (gap, major) |
| B6-G02 snow on packed ice | REPRODUCES | B6-G02 (gap, major) |
| B6-G03 water deficit | REPRODUCES (via total aquifer absence, G09) | B6-G03 (gap, major) |
| B6-G04 PoolPiece GD | FIXED (in the template itself) | B6-G04 (fixed, minor) |
| B6-G05 sculk water gates | REPRODUCES | B6-G05 (gap, minor) |
| B6-G06 empty carver/feature bundles | REPRODUCES (inferred — no prepared .local exists to measure) | B6-G06 (gap, major) |
| B6-G07 sculk density cap | REPRODUCES | B6-G07 (gap, minor) |
| B6-G08 sculk cursor 26.3 form | REPRODUCES | B6-G08 (gap, minor) |
| B6-G09 aquifers disabled | REPRODUCES | B6-G09 (gap, major) |
| B6-G10 NoiseRouter fallback | FIXED (fallback exists in the template; child inherited it) | B6-G10 (fixed, major) |
| B6-G11 material_rule required | REPRODUCES | B6-G11 (gap, major) |
| B6-G12 spawn_target decode | REPRODUCES | B6-G12 (gap, major) |
| B6-G13 density arms | REPRODUCES | B6-G13 (gap, major) |
| B6-G14 feature types | REPRODUCES | B6-G14 (gap, major) |
| B6-G15 state-provider names | FIXED (long-form names accepted in the template) | B6-G15 (fixed, major) |
| B6-G16 random_offset | REPRODUCES | B6-G16 (gap, major) |
| B6-G17 f32 pipeline | REPRODUCES | B6-G17 (gap, minor) |
| B6-G18 ore veins | REPRODUCES (originates in the template) | B6-G18 (gap, major) |
| B6-O01 densityfunction group | REPRODUCES (→ B6-O01; 7 oracles — Density, MaterialSurface, WorldSpawn move to O04 on measured first errors) | B6-O01 |
| B6-O02 material/rule group | REPRODUCES (→ B6-O02; 4 oracles) | B6-O02 |
| B6-O03 Feature ctor group | REPRODUCES (→ B6-O03; 3 oracles) | B6-O03 |
| B6-O04 missing-symbols group | REPRODUCES (→ B6-O04; 15 oracles — +Density, +MaterialSurface, +WorldSpawn; Sculk's first error is now the world.clock package) | B6-O04 |
| B6-O05 missing-override group | REPRODUCES (→ B6-O05; 2 oracles) | B6-O05 |
| B6-O06 no-override group | REPRODUCES (→ B6-O06, 1 oracle — TreeFeature only; FinalizeSpawn moves to O09) | B6-O06 + B6-O09 |
| B6-O07 other oracle failures | REPRODUCES (→ B6-O07, 2 oracles — BiomeZoom + HeightProvider; StructureStart → O08, StructurePlacement → O09) | B6-O07 + B6-O08 + B6-O09 |
| B6-O08 1.21.11-template API group | PARTIALLY WITHDRAWN — the framing was wrong: RegistryLayer.WORLD etc. exist in NEITHER 1.21.10 NOR 1.21.11; ChunkGeneration PASSES, NaturalSpawn/SpawnRules move to O09, StructureStart remains | B6-O08 (1 oracle) + B6-O09 |
| B6-H paired comparison | needs-harness (new seed/world) | B6-H |

### 1.21.7 items (SECONDARY — all 10 decided)

| 1.21.7 item | Verdict | 1.21.11 item |
|---|---|---|
| B6-G01 sculk_patch tail | REPRODUCES | B6-G01 (gap, major) |
| B6-G02 snow on packed ice | REPRODUCES | B6-G02 (gap, major) |
| B6-G03 water deficit | REPRODUCES (via total aquifer absence, G09) | B6-G03 (gap, major) |
| B6-G04 PoolPiece GD | FIXED | B6-G04 (fixed, minor) |
| B6-O01 densityfunction oracles | REPRODUCES (7 oracles) | B6-O01 (oracle-defect, major) |
| B6-O02 material/rule oracles | REPRODUCES (4 oracles) | B6-O02 (oracle-defect, major) |
| B6-O03 Feature ctor oracles | REPRODUCES (3 oracles) | B6-O03 (oracle-defect, major) |
| B6-O04 missing-symbol oracles | REPRODUCES (15 oracles) | B6-O04 (oracle-defect, major) |
| B6-O05 other missing packages | REPRODUCES (→ B6-O04/O07; blockscan, etc.) | B6-O04/O07 |
| B6-O06 BiomeZoom group | REPRODUCES | B6-O07 (oracle-defect, major) |

No UNVERIFIED carry-overs. The four NEVER EXISTED corrections from
other batches (B5-G17 chorus ceiling, B5-G18 bell isProperHit,
B2-G11/G12, B10-G04 BE renderers) are out of B6 scope and were not
re-minted.

## §6 Systemic findings

1. **The construction cascade is the whole story for 1.21.11
   worldgen, and it originates in the template.** G06 → G13 → G11 →
   G12, with G09 silent behind, means no 1.21.11 world generates at
   all. Every downstream defect (G01, G03, G05, G07, G08, G14, G16,
   G18) is latent behind it. The fixes are ordered: prepare folder
   names (G06), density arms (G13), surface_rule lookup (G11),
   spawn-target field handling (G12), aquifer wiring (G09). 1.21.10
   inherited the whole cascade verbatim — nothing in it is a
   child-only regression.
2. **The template is the origin of G18 too.** The OreVeinifier path
   is absent here (0 occurrences of the vein symbols), so 1.21.10's
   "NEW" G18 was never new — it was inherited. Same for the palette
   reader and motion-tag derivation noted by the coordinator: the
   template is where these gaps live.
3. **The 1.21.11 oracle adaptations are half-done and partly wrong.**
   ChunkGeneration is the model adaptation (correct, passes). The
   others adapted toward 1.21.11 APIs but got the details wrong:
   `ChunkPos.pack` (does not exist), `addStaticLayers` (does not
   exist), re-added `getNextEntityId` overrides (does not exist),
   wrong ctor arities (ServerLevel 11, DedicatedServer 7, ChunkMap
   13). The adaptations were written against an assumed 1.21.11 API,
   not the jar. Any further adaptation work must javap-verify each
   symbol first.
4. **The 1.21.10 report's O08 framing was wrong and is corrected
   here.** `RegistryLayer.WORLD`, `WORLD_REGISTRIES` and the
   3-arg `RandomState.create` order exist in NEITHER 1.21.10 NOR
   1.21.11 — they are 26.x API, not "1.21.11-template API". The
   1.21.11 O08 keeps the group name's symbols but re-grounds it: 1
   oracle (StructureStart) still fails on them first.
5. **First-error evidence beats provisional grouping.** The measured
   1.21.10 Batch 0 table corrected four of the 1.21.10 report's
   group placements (Density, MaterialSurface, WorldSpawn,
   StructurePlacement) and one of its failure-mode claims
   (BiomeChunk's lambda, not the arity, errors first). The 1.21.11
   grouping follows measured first errors throughout, and the
   javac checks done here (server classpath) are the first
   1.21.11-side measurements — Batch 0 re-verification is still
   owed, especially the client column.
6. **The 1.21.11 reference world is a weak ground truth for snow and
   sculk.** No snow, no frozen-peaks biome, no sculk blocks. G01 and
   G02 rest on code + javap, not on the census. A frozen-peaks seed
   and a deep-dark seed are needed for live confirmation (see B6-H).

## Prior verification referenced (not re-diagnosed)

- 1.21.10 measured Batch 0 oracle table
  (`sweep-1.21.11-ref/1.21.10/batch-0-oracles.json`): starting status
  for all 57 oracles; first errors quoted verbatim where used.
- 1.21.10 Batch 6 report + slice (template for this report's
  structure and the PRIMARY re-check set).
- Coordinator-verified template facts (brief): palette reader absent
  (B11-G10 inherited FROM the template), ServerLevel ctor 11-arg
  with RandomSequences, supports_* synthesis absent, motion-tag
  derivation absent (`Assets::load` hard-requires the three motion
  tags). Verified by the coordinator; not re-diagnosed here except
  where B6 evidence touches them (ServerLevel ctor form re-verified
  by javap for O09).
- 1.21.7 sweep items (SECONDARY re-check set) and the 26.1.x sweep
  water-deficit demonstrations (G03's provenance).

## Open items (not defects)

- Batch 0 has NOT landed for 1.21.11: the oracle triage is
  provisional. Re-verify against
  `docs/sweep/1.21.11/batch-0-oracles.json` when it lands, especially
  the client-classpath column (this batch measured server-classpath
  only).
- No prepared 1.21.11 `.local` exists on the VM: G06's bundle
  emptiness is inferred from identical prepare code + the 1.21.10
  measured 0/0/0. A prepared 1.21.11 `.local` would turn the
  inference into a measurement.
- The multi-noise prepared-side equality (port reads nether/overworld
  parameter points from prepared `biome_parameters/` reports) is NOT
  re-verified for 1.21.11 — no prepared reports exist on the VM. The
  code path is the same as the verified 1.21.10 run, and the vanilla
  nether preset values were verified from javap.
- The 11 non-B6 oracle sources using RegistryLayer.WORLD /
  WORLD_REGISTRIES need their owning batches' triage (flagged for
  the coordinator).

## Adjacent findings (not B6 defects)

- **Live-code sculk water gates (flag for B4).** `level/sculk.rs`
  (~611, ~385) shows the same WATER_SOURCE narrowing as B6-G05 in
  the live sculk code. Not minted here — B4 owns live sculk.
- **Live-code sculk density cap is correct (B4 note).**
  `level/sculk.rs` counts sensors/shriekers directly (the vanilla
  form) — only the worldgen copy in `sculk.rs` reads the dead tag
  (G07).
- **11 non-B6 oracles use RegistryLayer.WORLD/WORLD_REGISTRIES**
  (see O08): AudioAttributes, BlockUpdate, ChatScreen, Enchanting,
  EntityDeathLoot, Environment, LootFill, MobAttributes, Raid,
  SituationalMusic, TradeComponents. They fail on the 1.21.11 jar
  for the same reason StructureStart does.
- **First palette entry is Name/Properties form** (Batch 11 note):
  the reference world's first palette entry is
  `{'Name': 'minecraft:bedrock'}` — relevant to the palette-reader
  work, not a B6 defect.
- **No DUMMY block entity in this world** (1.21.10 had 1) and 0
  chunks with blending_data/retrogen — census observations, not
  defects.

## Method notes

- javap: JDK 25.0.5 (`~/workspace/tools/jdk-25/bin`), ~30 s per
  class; dumps in `evidence/javap-*.txt`. Normalised javap diffs vs
  1.21.10 in `evidence/norm-tmp/` (`norm.py`, `norm2.py`).
- javac: JDK 25.0.5, `javac --release 21 -proc:none -sourcepath
  <parity dir>`, classpath server-game-named.jar + 99 server library
  jars + jsr305-3.0.2.jar. Logs in `evidence/javac-check/` (`sp-*`
  logs are the -sourcepath runs; non-`sp-` logs lacking -sourcepath
  are superseded for BlockSurvival).
- Census: own NBT parser (`nbtmini.py`, `coldump.py`, `census.py`),
  read in place from `~/workspace/vanilla-world-mc/1.21.11/New
  World/`; no git in that clone, no world files copied or moved.
  The wrong-seed VM world (terrain2/vanilla/world,
  -8343112210435233944) was never used.
- Jar data: extracted from client-named.jar to
  `evidence/jar-data/` (7,557 files); server-jar data verified
  byte-identical (diff exit 0), duplicate removed.
- Transient "Disk quota exceeded" write failures occurred twice
  during the run (46G free on disk); writes recovered on retry —
  the same intermittent VM flakiness as the documented EPERM
  failures, not real quota exhaustion.
- Source diffs 1.21.11 vs 1.21.10 were taken on the two sweep
  worktrees. Byte-identical B6-relevant files: `worldgen/sculk.rs`,
  `placement.rs`, `climate.rs`, `blending.rs`,
  `biome_temperature.rs`, `spawn_finder.rs`, `ore_feature.rs`,
  `structure/jigsaw.rs`, `aquifer/noise_based.rs`,
  `feature_graph.rs`, `carver/generation.rs`, `vegetation.rs`,
  `material_condition/library.rs`, `material_condition/rule.rs`,
  `material_stage.rs`, `horizon-core/src/density/graph/mod.rs`,
  `level/weather.rs`, `snow_freeze.rs`. Comment-only diffs
  (version-number mentions): `density.rs`, `router.rs`,
  `state_provider.rs`, `generator.rs`, `block_survival.rs`,
  `level/data.rs`, `level/survive.rs`, `level/sculk.rs`,
  `worldgen/disk.rs`. `horizon-prepare/src/lib.rs` differs only in
  the jsr305 classpath helper; WORLDGEN_BUNDLES identical.

## §7 Evidence index

All paths under `~/workspace/mcir-work/sweep-1.21.11-batches/b06/`
unless noted.

- `report.md` (this file), `slice.json`
- `evidence/datadiff.py`, `evidence/datadiff.log` — jar-side
  registry counts, presets, carver lists, noise-settings keys, tags
- `evidence/decode_audit.py`, `evidence/decode_audit.log`,
  `evidence/decode-audit.json` — configured-feature type audit (22
  MISSING = G13/G14/G16)
- `evidence/recursive-census.py`, `evidence/recursive-census.log`,
  `evidence/recursive-census2.log` — density-function / feature /
  pool / processor census, G14 blast radius, random_offset list
- `evidence/census.py`, `evidence/census.log`,
  `evidence/world-1.21.11-census.json` — reference-world census
  (9,093 chunks)
- `evidence/coldump.py`, `evidence/nbtmini.py` — NBT reader helpers
- `evidence/jar-data/` — extracted client-named.jar `data/` tree
  (7,557 files)
- `evidence/javap-*.txt` (~25 dumps: sculkpatch, sculkpatchconfig,
  sculkblock, sculkbehaviour1, chargecursor, sculkspreader,
  snowlayer, snowfreeze, aquifer, spawnfinder, noisechunk,
  noiserouter, orefeature, densityfunction, blendingdata, biome,
  feature, chunkstatus, multinoise-preset, noisegen, registrylayer,
  randomstate, noiserouterdata, structs, protochunk, level,
  biomesource, chunkaccess, dataloadcontext, dimensiontype)
- `evidence/norm.py`, `evidence/norm2.py`, `evidence/norm-tmp/` —
  normalised javap diffs vs 1.21.10
- `evidence/oracle-diff.log`, `evidence/oracle-diff2.log` — oracle
  source diffs vs 1.21.10 (27 identical / 24 rename-only / 6 real)
- `evidence/javac-check/` — targeted javac logs (`sp-*.log` are the
  -sourcepath runs) + compiled classes dirs
- `evidence/drivers.py`, `evidence/drivers-1.21.11.json` — driver
  map (64 drivers, 12 runnable / 38 blocked / 14 no-oracle)
- Reference: `~/workspace/mcir-work/sweep-1.21.11-ref/1.21.10/reports/batch-6-worldgen.md`,
  `~/workspace/mcir-work/sweep-1.21.11-ref/1.21.10/slices/batch-06.json`,
  `~/workspace/mcir-work/sweep-1.21.11-ref/1.21.10/batch-0-oracles.json`,
  `~/workspace/mcir-work/sweep-1.21.10-batches/b06/evidence/drivers-1.21.10.json`

## §8 Verdict

Batch 6 complete for 1.21.11. **37 checks: 22 pass, 15 gap** (37
behaviour / 0 non-behaviour as-tallied). **28 items**: 15 behaviour
gaps (11 major, 4 minor), 3 fixed, 9 oracle-defect groups (39 failing
oracles of 57), 1 needs-harness.

The headline: the template's worldgen does not generate. The
G06→G13→G11→G12 construction cascade, with G09's silent aquifer
absence behind it, is inherited wholesale by 1.21.10 — and G18 (ore
veins) originates here too, not in the child. The three fixes the
child was credited with (G04, G10, G15) all exist in the template
itself; the child inherited them. On the oracle side, the story is
the opposite: the template's ChunkGeneration oracle is the one
correctly adapted oracle in the batch, and it passes — flipping 4
drivers to runnable — while the other 1.21.11 adaptations are
half-done and partly wrong (pack, addStaticLayers, re-added
getNextEntityId, wrong ctor arities), and the 1.21.10 report's O08
framing is corrected: RegistryLayer.WORLD never existed in either
version.
