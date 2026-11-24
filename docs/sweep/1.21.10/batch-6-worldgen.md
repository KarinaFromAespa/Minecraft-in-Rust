# Batch 6 — Worldgen / features (1.21.10 parity sweep)

Date: 2026-11-25. Checks + triage only; no fixes, no code changes, no commits.
Worktree `~/workspace/mcir-work/sweep-1.21.10`, branch `parity-sweep-1.21.10` (read-only).
Vanilla reference: the named 1.21.10 jars
`~/workspace/mcir-work/e6-deltas/versions/1.21.10/jars/{client-named,server-game-named}.jar`
(sha1-verified: client `715f5816…`, server-game `ba1fc89e…`; never the
bundler `server.jar`). DataVersion 4556, protocol 773. All behaviour claims
verified against 1.21.10's OWN jar/data, never the 26.3 base's form — and
never the 1.21.11 era template's form either (see O08).

## §0 Execution status

Complete. All 10 1.21.7 items (PRIMARY) and all 25 26.1.2 items (SECONDARY)
decided; no UNVERIFIED carry-overs. Static checks only: no cargo builds, no
driver runs, no commits, no git in the worktree. The reference world was
read in place with an own NBT parser; no world files copied or moved; no
git commands inside `vanilla-world-mc`.

Key context for every verdict below: the 1.21.10 tree derives from the
1.21.11 era template (605 files differ from it), NOT from 26.1.x — so no
26.1.2 answer was assumed to carry; every defect below was re-verified
fresh against the 1.21.10 jar and the 1.21.10 tree (fresh javap dumps,
fresh data diffs, fresh decode audit, fresh census). Where the 1.21.10
tree is byte-identical to the 26.1.2 tree (`worldgen/sculk.rs`,
`placement.rs`, `climate.rs`, `blending.rs`, `biome_temperature.rs`,
`spawn_finder.rs`, `ore_feature.rs`, `spring.rs`, `biome_weather.rs`,
`structure/jigsaw.rs`, `worldgen/aquifer/noise_based.rs`,
`material_condition/library.rs`, `material_stage.rs`,
`level/weather.rs`, `horizon-core/src/density/`), the 26.1.2 jar evidence
was still re-checked: the relevant 1.21.10-vs-26.1.2 javap diffs are
bytecode-identical after normalisation (constant-pool renumbering,
`ldc`/`ldc_w`, branch-target shifts, `BlockState.is(Block)` vs
`is(Object)` erasure, access modifiers, lambda numbering) for
SculkPatchFeature, SculkBlock, SculkBehaviour$1, SculkSpreader$ChargeCursor,
SculkSpreader, Aquifer$NoiseBasedAquifer, Climate$SpawnFinder,
DensityFunction, ConfiguredFeature, ChunkStatus and NoiseRouter (the last
is byte-identical outright). The REAL cross-version differences found by
that diff are: SnowLayerBlock reads `SNOW_LAYER_CANNOT_SURVIVE_ON` /
`SNOW_LAYER_CAN_SURVIVE_ON` in 1.21.10 vs `CANNOT_SUPPORT_SNOW_LAYER` /
`SUPPORT_OVERRIDE_SNOW_LAYER` in 26.1.2 (G02's mechanism); Biome gains
`EnvironmentAttributeMap` in 26.1.2 (weather methods identical);
ConfiguredFeature renames `getFeatures()` to `getSubFeatures()`; Feature
renames several registry fields (`FLOWER`/`ICE_SPIKE`/`FOREST_ROCK` vs
`SPIKE`/`BLOCK_BLOB`); `ResourceLocation` becomes `Identifier`.

**Batch 0 has NOT landed for 1.21.10** (`b0/` is empty; no
`docs/sweep/1.21.10/batch-0-oracles.json` in the worktree). Oracle triage
below is therefore provisional: each group's root cause was verified
statically against the 1.21.10 jar (package listings + javap) and the
1.21.10 oracle sources (diffed against 26.1.2's — 172 of 391 differ,
mostly the `ResourceLocation`→`Identifier` rename), starting from the
26.1.2 Batch 0 table. No big javac compile was run. Re-verify against
the 1.21.10 Batch 0 table when it lands; the two verdicts most sensitive
to a compile are the O08 group (first-error ordering) and the 17
provisionally-passing oracles.

No `.local` exists in the worktree. Prepared 1.21.10 data was read from
`~/workspace/mcir-work/candidate-test-1.21.10/.local/horizon-client/1.21.10/`
(worldgen bundles present, verified).

## Tally (behaviour checks only)

**27 checks: 18 pass, 9 gap. Behaviour/non-behaviour split: 27 behaviour +
0 non-behaviour as-tallied** (oracle triage is counted separately, not
tallied — see below).

| Group | Checks | Pass | Gap |
|---|---|---|---|
| 1.21.7 defect re-checks | 4 | 1 (G04) | 3 (G01, G02, G03) |
| Data diffs (prepared vs jar) | 6 | 5 | 1 (carver/feature bundles) |
| Code-vs-jar | 12 | 7 | 5 (sculk, spawn finder, generator construction incl. ore veins, decode-coverage, numeric precision) |
| Reference-world census | 5 | 5 | 0 |
| **Total** | **27** | **18** | **9** |

Oracle triage is counted separately (per the 26.1 tally convention):
**57 oracles** (55 B6-referenced + 2 group-only), **17 pass, 40 fail**;
**64 drivers: 8 runnable, 14 no-oracle, 42 blocked** (see Oracle defects).
Driver map read fresh from `versions/1.21.10/tools/` (648 drivers total);
all 64 B6 row names present, none missing.

Defect items: **18 G items** — 15 gap (11 major, 4 minor), 3 fixed
(G04 minor, G10 major, G15 major). Of the 15 gaps, 14 are 26.1.2
reproductions and **1 is NEW to 1.21.10** (G18 ore veins). Oracle items:
**8** (B6-O01…O08 — O08 is new). Needs-harness: **1** (B6-H). Total
items: **27**.

Headline: the 1.21.10 worldgen port carries the same construction
failures as 26.1.2, with two real fixes (router fallback, long-form
state providers) and two 1.21.10-specific regressions — snow survives on
packed ice again because vanilla renamed the snow tags between 1.21.10
and 26.1.2 and the port reads the 26.x names (G02), and the oracle
sources were adapted for the 1.21.11 template, whose
`RegistryLayer.WORLD` / `WORLD_REGISTRIES` / `RandomState.create` order
do not exist in 1.21.10, breaking a previously passing oracle
(VanillaChunkGenerationOracle, O08). **No 1.21.10 world can generate at
all with the current tree + prepared data** — `GeneratorData::load`
fails on the empty carver bundle (G06), and behind it `Pipeline::new`
fails on the density caches (G13), the material rule (G11) and the spawn
target (G12), in that order; even with all of those fixed, aquifers
would be silently absent (G09), ore veins would never generate (G18),
22 configured features would fail at decoration (G14) plus 10 placed
features on `random_offset` (G16), and the density pipeline would run
in f32 where vanilla computes in f64 (G17).

## Defects

### B6-G01 (major, 1.21.7 defect — REPRODUCES) — sculk_patch catalyst / extra rare growths tail missing

`worldgen/sculk.rs` (727 lines, byte-identical to 26.1.2's) decodes only
charge_count, amount_per_charge, spread_attempts, growth_rounds,
spread_rounds; `place()` returns `Ok(true)` at line 163 after the
spreader rounds; the string `catalyst` appears nowhere in the file.
Vanilla 1.21.10 `SculkPatchFeature.place` (fresh javap,
`javap-sculkpatch.txt`) continues after the spreader rounds: a
`nextFloat() <= catalystChance` roll with a collision-full block below
places `SCULK_CATALYST` at the origin (flag 3), then
`extraRareGrowths.sample` iterations place `SCULK_SHRIEKER`
(`can_summon=true`, flag 3) at `offset(nextInt(5)-2, 0, nextInt(5)-2)`
where the target is air and the block below is face-sturdy UP. The
catalyst roll comes BEFORE the growth sample, so the port also shifts
the RNG stream for everything after. The reference world holds the
vanilla ground truth: 25 sculk catalysts and 12 sculk shriekers in
deep_dark terrain. The port generates neither, ever.

### B6-G02 (major, 1.21.7 defect — REPRODUCES, new mechanism) — snow survives on packed ice

The 1.21.7 defect reproduces in 1.21.10 with a mechanism that differs
from both 1.21.7's and 26.1.2's. Vanilla 1.21.10 names the snow tags
`snow_layer_can_survive_on` = [honey_block, soul_sand, mud] and
`snow_layer_cannot_survive_on` = [ice, packed_ice, barrier] (verified
in the jar `tags/block/` and in the prepared `block_tags.json`);
`SnowLayerBlock.canSurvive` (fresh javap, `javap-snowlayer.txt`) reads
`SNOW_LAYER_CANNOT_SURVIVE_ON` → false, `SNOW_LAYER_CAN_SURVIVE_ON` →
true, else `Block.isFaceFull(collisionShape, UP)` or (same-block snow
with layers == 8). Between 1.21.10 and 26.1.2 vanilla RENAMED both tags
to `cannot_support_snow_layer` / `support_override_snow_layer`
(normalised javap diff, `javap-snowlayer` — the only real logic
difference in the class). The port reads the 26.x names everywhere:
`block_survival.rs:598-599` (`tag_states("minecraft:cannot_support_snow_layer")`
/ `"minecraft:support_override_snow_layer"`), `level/data.rs:431-432`
(tags! macro), `level/weather.rs:113,116`, `level/survive.rs:314,316`
(`Class::SnowLayer`; the surrounding logic otherwise matches the jar).
Both tag loaders are silent-empty on a missing tag
(`block_survival.rs:481-493` `tags.get(name).into_iter().flatten()`;
`level/data.rs:398-400` `.unwrap_or_default()`), so the deny set is
empty and packed ice — face-full — passes the fallback: snow survives
on packed ice. Second-order divergence: the allow set is also empty, so
honey_block/soul_sand/mud lose their override. Caveat: the reference
world has 0 snow blocks and no frozen_peaks/grove/snowy_slopes/
jagged_peaks biomes, so there is no live census confirmation; the
defect is demonstrated statically (code + javap + tag data). The
placement feature itself is fine: `SnowAndFreezeFeature.place`
(`snow_freeze.rs`) matches the jar exactly (16×16, MOTION_BLOCKING,
biome at top, `shouldFreeze(below,false)` → ice flag 2,
`shouldSnow(top)` → snow flag 2 + snowy=true below) — the defect is in
survival/weather, not placement.

### B6-G03 (major, 1.21.7 defect — REPRODUCES) — underground water deficit; the entire aquifer system is absent

Via G09: the aquifer system is entirely disabled (see below), so no
aquifer water generates at all. The 1.21.7 defect was measured as a
−1.04% water deficit; in 1.21.10 the cause is total absence, not
divergence. The reference world holds 20,398,344 water blocks; the port
would generate only surface water (sea level / lakes / springs). No
paired port world exists, so no percentage is claimed for 1.21.10 —
the defect is the absence itself, demonstrated by the G09 code path.

### B6-G05 (minor, 1.21.7 defect — REPRODUCES) — sculk spreader water gates narrowed to water sources

`sculk.rs:305` (`attempt_spread_vein`, Default branch, non-empty
facings) and `sculk.rs:548` (`can_place_growth` above-open gate)
require `WATER_SOURCE`. Vanilla 1.21.10 accepts any water:
`SculkBehaviour$1.attemptSpreadVein` regrows when the block is air OR
`fluidState.is(Fluids.WATER)` (javap, bytecode 57–60);
`SculkBlock.canPlaceGrowth` accepts above air OR (is(WATER) AND
fluidState.is(WATER)). Flowing water therefore blocks sculk spread in
the port where vanilla spreads through it. Same defect, same lines as
26.1.2 (file byte-identical). The live-gameplay code carries the same
pattern: `level/sculk.rs` `sculk_can_place_growth` (~611) and
`sculk_attempt_spread_vein` (~385) both gate on `is_source()` — flagged
for B4, not minted here.

### B6-G06 (major — REPRODUCES) — prepared `carvers.json` and `features.json` are EMPTY; `GeneratorData::load` fails

`horizon-prepare` `WORLDGEN_BUNDLES` (`lib.rs:1492-1512`) maps
`("carvers","worldgen/carver")`, `("features","worldgen/feature")`,
`("feature_tags","tags/worldgen/feature")` — the 26.3 folder names.
1.21.10 data lives in `worldgen/configured_carver` (4 files) and
`worldgen/configured_feature` (224 files); the prepared bundles are
verified empty (0/0/0) while the raw `data/` copies are 4/4 and 224/224
identical to the jar. `GeneratorData::load` (`generator.rs:486,497`)
reads bundles only; `carvers.decode_set(...)` with `?` fails for 59 of
65 biomes (54 list `[cave, cave_extra_underground, canyon]`, 5 list
`minecraft:nether_cave`) → load fails. This is the first failure in the
construction cascade. (The 26.1.2 report's `features.json` count and the
1.21.10 count agree: 224 configured features in the jar, 0 in the
bundle.)

### B6-G07 (minor — REPRODUCES) — sculk `can_place_growth` density cap is dead (26.3-only tag)

`sculk.rs:108` reads tag `minecraft:sculk_growth_inhibitors`; the tag
is absent from both the 1.21.10 jar block tags (the jar has only
`sculk_replaceable` + `sculk_replaceable_world_gen`) and the prepared
`block_tags.json`. `tag()` returns the empty set, so the ≥3 count at
`sculk.rs:556-558` never fires and growth is never capped. Vanilla
1.21.10 `SculkBlock.canPlaceGrowth` counts `SCULK_SENSOR` /
`SCULK_SHRIEKER` blocks directly in the offset(−4,0,−4)..(4,2,4) box
and returns false when the count exceeds 2 (fresh javap). Notably, the
port's OWN live code knows this: `level/sculk.rs`
`sculk_can_place_growth` counts sensors/shriekers directly with the
comment "there is no growth-inhibitors tag in 1.21.10" — only the
feature code in `worldgen/sculk.rs` was never adapted.

### B6-G08 (minor — REPRODUCES) — sculk cursor movement uses the 26.3 form

`update()` (`sculk.rs:263-270`) kills a stuck cursor (None →
`on_discharged` + charge=0 + return) and has no post-move 15-block
check; `valid_movement` (`sculk.rs:642,659-663`) applies a pre-move
12-block filter (dx²+dz² > 144). Vanilla 1.21.10
`SculkSpreader$ChargeCursor.update` (fresh javap,
`javap-chargecursor.txt`) lets a null movement position fall through to
the facings/decay-delay/update-delay handling (bytecode 184–186 `ifnull
272` — the cursor is NOT killed), and kills the cursor post-move when
it is NOT `closerThan(Vec3i(origin.x, pos.y, origin.z), 15.0d)`
(bytecode 247 `ldc2_w 15.0d`); `getValidMovementPos(LevelAccessor,
BlockPos, RandomSource)` takes no origin and applies no 12-block
filter. Three divergences in one method, all carried from the 26.3
form.

### B6-G09 (major — REPRODUCES) — aquifers are entirely disabled (26.3 config-object form)

`Pipeline::new` (`generator.rs:1114-1119`) still reads
`settings_json.get("aquifers")` (the 26.3 config object). All 7
1.21.10 noise settings carry `aquifers_enabled` (bool) instead — no
`aquifers` key exists in any of them — so the aquifer is always `None`
→ `DisabledAquifer`. There is no `from_router`, no
`aquifers_enabled` gate, no `aquifer_exclusion` synthesis. Silent: no
error, no log, just no aquifer water anywhere. The aquifer ENGINE is
faithful (see Verified passes — `noise_based.rs` byte-identical to
26.1.2's, and the 1.21.10 `NoiseBasedAquifer` bytecode is identical to
26.1.2's after normalisation); only the wiring is dead. Vanilla's
`Aquifer` ctor takes the `NoiseRouter` and reads barrier /
fluidLevelFloodedness / fluidLevelSpread / lava noises (fresh javap,
`javap-aquifer.txt`).

### B6-G11 (major — REPRODUCES) — `Pipeline::new` requires `material_rule`; 1.21.10 settings carry an inline `surface_rule`

`generator.rs:1136-1138` still requires
`settings_json["material_rule"].as_str()`. All 7 1.21.10 noise settings
carry an inline `surface_rule` object only — no `material_rule` key
exists in any of them. `Pipeline::new` fails here, behind G13 in the
cascade. (The surface-rule CONTENT is fine: the datadiff shows all 7
settings name-set- and content-identical to the jar, and the material
surface engine itself is the 26.3-form rule engine — the failure is
purely the lookup key.)

### B6-G12 (major — REPRODUCES) — spawn_target decode fails (named-density lookups); depth not zeroed

`generator.rs:1141-1164` maps every spawn-target field name through
`library.named_density(name)` (`material_condition/library.rs:101`),
which looks up `minecraft:<name>` in the compiled density graph and
errors `unknown material density` for anything unregistered. The
1.21.10 overworld / amplified / large_biomes spawn targets use the 7
bare field names (continentalness, depth, erosion, humidity, offset,
temperature, weirdness) — none of them a registered density function —
so decode fails. Second half, latent: `spawn_finder.rs` (byte-identical
to 26.1.2's) samples every target field as a density, but vanilla
`Climate$SpawnFinder.getSpawnPositionAndFitness` (fresh javap,
`javap-spawnfinder.txt`) rebuilds the sampled TargetPoint with depth
FORCED to 0 (`lconst_0` as the depth argument, bytecode 39) and adds
`square(offset)` as a constant. The radial search itself (coarse
2048/512, fine 512/32, fitness × 2048² + distance bias) matches the
jar. The 1.21.10 `SpawnFinder` bytecode is identical to 26.1.2's after
normalisation (access modifiers only).

### B6-G13 (major — REPRODUCES) — density decode missing seven types

`density.rs` (594 lines) has NO arms for `cache_2d`, `cache_once`,
`flat_cache`, `shifted_noise`, `y_clamped_gradient`,
`weird_scaled_sampler`, `end_islands`; the default arm errors `unknown
density type: {kind}`. Recursive census of the 1.21.10 data: the 35
density-function files use cache_2d ×11, cache_once ×12, flat_cache
×16, shifted_noise ×5, y_clamped_gradient ×6, weird_scaled_sampler ×3,
end_islands ×1; the noise settings add cache_2d ×13, shifted_noise ×12,
y_clamped_gradient ×23, end_islands ×1. `DensityRegistry::from_json`
resolves ALL 35 densities eagerly, so G13 fires first inside
`Pipeline::new`, before the router decode. Notes: the port DID adapt
`shift_noise()` for 1.21.10's `argument`-holding-noise-name form (vs
26.3's `noise` key); `minecraft:y` IS `y_clamped_gradient` (from_y
−4064, to_y 4062, verified fresh from the jar data); no
`minecraft:gradient` or `minecraft:cache` type exists anywhere in the
data, so the port's arms for those are dead code. Revised 1.21.10
cascade: G06 (load) → G13 (density decode) → G11 (material_rule) → G12
(spawn_target), G09 silent behind.

### B6-G14 (major — REPRODUCES) — 11 configured-feature types + `coral_mushroom` + `nether_cave` have no decode arms

Top-level configured-feature types in the 1.21.10 data: 55 distinct,
224 total. Eleven have NO arm in the port's feature dispatch
(`feature_graph.rs`; the default arm falls through to
`BasicFeature::from_json`, `feature_graph.rs:809`):

| Missing type | Configured features |
|---|---|
| `basalt_columns` | 2 |
| `basalt_pillar` | 1 |
| `desert_well` | 1 |
| `dripstone_cluster` | 1 (port has 26.3 name `speleothem_cluster` instead) |
| `glowstone_blob` | 1 |
| `kelp` | 1 |
| `nether_forest_vegetation` | 6 |
| `sea_pickle` | 1 |
| `seagrass` | 4 |
| `twisting_vines` | 2 |
| `weeping_vines` | 1 |

Plus `coral_mushroom`, nested inside `warm_ocean_vegetation`
(`CoralFeature::decode`, `vegetation.rs:284-300`, errors on
non-tree/claw kinds; dispatch `feature_graph.rs:599` has coral_tree |
coral_claw only). Total: **22 broken configured features, affecting 24
placed features** — verified by name against the 1.21.10 placed-feature
data (basalt_pillar, crimson/warped forest vegetation, desert_well,
dripstone_cluster, glowstone, glowstone_extra, kelp_cold/warm,
large/small basalt columns, nether_sprouts, sea_pickle, seagrass ×8,
twisting_vines, warm_ocean_vegetation, weeping_vines). The `seagrass`
×4 and `weeping_vines` ×1 grep-"ok" hits in the decode audit are false
positives (no dispatch arm — verified by direct dispatch read). Feature
decoding is lazy (`FeatureCatalog::named`), so these fail at decoration
time — a hard error during decoration, not at load. (Latent behind G06:
`features.json` is empty, so today every placed feature fails earlier
with `unknown feature reference`.)

Also missing: the `nether_cave` carver arm
(`carver/generation.rs:26-27` has cave + canyon only; the default
errors `world carver type`). The 5 nether biomes list
`minecraft:nether_cave` as their carver, so fixing G06 alone would
surface this next for the nether.

Overlap with G15: none in 1.21.10 — G15 is fixed, so G14∪G15 = G14
only (22 features). (In 26.1.2 the union was 126 of 221.)

Present and verified (see Verified passes): 30 of 31 placed-feature
modifier types (only `random_offset` missing — G16), all 8 int-provider
and 6 height-provider types, all 9 trunk / 11 foliage / 1 root placers,
all 10 tree decorators, all 5 pool element types, all 4 processor
types used, both structure-set placements, all 16 structure types.

### B6-G16 (major — REPRODUCES, smaller blast radius) — `random_offset` placement modifier has no decode arm

Zero occurrences of `random_offset` / `RandomOffset` in the whole
`horizon-world` tree (`placement.rs` byte-identical to 26.1.2's). The
1.21.10 data uses it 10× in placed features (spore_blossom,
rooted_azalea_tree, pointed_dripstone, lush_caves_vegetation,
lush_caves_clay, lush_caves_ceiling_vegetation, ice_patch,
end_gateway_return, disk_grass, cave_vines) plus 2× nested in the
configured feature `pointed_dripstone` — vs 70 placed features in
26.1.2 (1.21.10 has no `patch_*` vegetation patches using it). 30 of
the 31 placement-modifier types used in the data are present; this is
the only missing one.

### B6-G17 (minor — REPRODUCES) — worldgen numeric pipeline is f32 where vanilla 1.21.10 computes in f64

Four sites, all verified fresh: (1) the density graph
(`horizon-core/src/density/graph/mod.rs`, byte-identical to 26.1.2's):
`GraphNode::Constant(f32)`, gradient/clamp/choice bounds in f32 —
vanilla `DensityFunction.compute` returns `double` and
`fillArray(double[],...)` (fresh javap, `javap-densityfunction.txt`);
(2) `climate.rs` (byte-identical): parameter quantisation in f32;
(3) `blending.rs` (byte-identical): blend weights in f32 — vanilla
`BlendingData.NO_VALUE` is a `double` (`javap-blendingdata.txt:33`);
(4) `biome_temperature.rs` (byte-identical): biome temperature noise in
f32, and `PerlinSimplexNoise` — which vanilla `Biome` uses for
TEMPERATURE_NOISE / FROZEN_TEMPERATURE_NOISE / BIOME_INFO_NOISE
(`getValue(DDZ)D`, fresh javap) — is absent from `horizon-core`
entirely. The biome weather methods themselves (`shouldSnow`,
`shouldFreeze`, `coldEnoughToSnow`, `getTemperature`,
`getHeightAdjustedTemperature`, `getPrecipitationAt`) are bytecode-
identical between the 1.21.10 and 26.1.2 jars.

### B6-G18 (major — NEW to 1.21.10) — ore veins never generate (no OreVeinifier path)

Vanilla 1.21.10 has NO `ore_vein` type in any surface-rule data (0
occurrences in all 7 noise-settings files). Ore veins come from code:
`OreVeinifier` (`net/minecraft/world/level/levelgen/OreVeinifier.class`),
whose `create(DensityFunction, DensityFunction, DensityFunction,
PositionalRandomFactory)` is called by `NoiseChunk` (fresh javap,
`javap-noisechunk.txt:365-376`) gated on
`NoiseGeneratorSettings.oreVeinsEnabled()` with
`router.veinToggle()` / `veinRidged()` / `veinGap()` — all three vein
fields exist as inline densities in the overworld router (referencing
ore_gap / ore_vein_a / ore_vein_b / ore_veininess noises), and all 7
settings carry `ore_veins_enabled`. The port has a full
`MaterialRule::OreVein` implementation (`material_condition/rule.rs`,
`library.rs:245` arm, `material_stage.rs:405-410,552-579`
`veins_miss`) but it is driven ONLY by `minecraft:ore_vein` rule
nodes, which no 1.21.10 data contains; `Pipeline::new` never reads
`ore_veins_enabled`, and the 8-field router never decodes the vein
fields (G10's residual). No iron/copper ore veins generate in any
1.21.10 world. (In 26.1.2 this was folded into G10's residual; in
1.21.10 G10-as-titled is FIXED, so the vein gap is minted as its own
item.)

## Fixed / withdrawn defects (verified absent in 1.21.10)

### B6-G04 (minor) — FIXED: PoolPiece NBT writes `GD`

`PoolPiece::save` (`structure/jigsaw.rs:682-683`) goes through
`legacy::piece_tag("minecraft:jigsaw", bbox, None, 0)`, which writes
`("GD", Tag::Int(gen_depth))` (`legacy.rs:339`). The reference world
confirms the vanilla form: trial chambers 1909/1909 pieces GD=0, trail
ruins 18/18 GD=0, mineshaft GD 0–9. (1.21.7 defect withdrawn; 26.1.2
also fixed.)

### B6-G10 (major) — FIXED (router): `preliminary_surface_level` fills the surface field; vein-field residual is G18

`router.rs` (230 lines) still declares 8 fields including
`ChunkSurfaceLevel`, BUT `from_json` now falls back to
`preliminary_surface_level` when `chunk_surface_level` is absent
(`router.rs:57-62`, with dedicated test
`preliminary_surface_level_fills_the_surface_field`). `Pipeline::new`
therefore no longer fails on the router for 1.21.10 settings. Vanilla
1.21.10's `NoiseRouter` record has 15 components (fresh javap,
`javap-noiserouter.txt` — bytecode-identical to 26.1.2's): barrierNoise,
fluidLevelFloodednessNoise, fluidLevelSpreadNoise, lavaNoise,
temperature, vegetation, continents, erosion, depth, ridges,
preliminarySurfaceLevel, finalDensity, veinToggle, veinRidged,
veinGap. NO `chunk_surface_level`. Residual: the 7 jar router fields
the port never decodes (barrier, fluid_level_floodedness,
fluid_level_spread, lava, vein_toggle, vein_ridged, vein_gap) — the
aquifer consequence is G09, the ore-vein consequence is the new G18.
(26.1.2 defect fixed in 1.21.10.)

### B6-G15 (major) — FIXED: state-provider decode accepts the long-form names

`state_provider.rs` `decode_typed` (:333-456) accepts BOTH spellings
for every provider used in 1.21.10 data (simple /
simple_state_provider, weighted / weighted_state_provider, noise /
noise_threshold / dual_noise + `_provider` forms, rotated /
rotated_block_provider, randomized_int /
randomized_int_state_provider). The comment at :349-354 explains that
the rule-based provider is a standalone record in 1.21.10, NOT a
registered subtype — `RuleBasedBlockStateProvider` is decoded
separately (`state_provider.rs:505-518`) and used by `disk.rs:65`
(verified against `disk_clay.json`: the state_provider is `{fallback,
rules}` with no "type" key). 113 of 224 configured features contain a
long-form provider — all now decode. Provider uses in the data:
simple_state_provider ×199, weighted ×41, randomized_int ×4,
rotated_block / noise / dual_noise / noise_threshold ×1 each;
`rule_based_state_provider` 0 uses. (26.1.2 defect fixed in 1.21.10.)

## Oracle defects (provisional — Batch 0 has NOT landed for 1.21.10; 57 oracles, 17 pass / 40 fail)

Authority: the 26.1.2 Batch 0 table (`docs/sweep/26.1.2/batch-0-oracles.json`,
391 files) as the starting point, with every group's root cause
re-verified statically against the 1.21.10 named jars (package listings +
javap) and the 1.21.10 oracle sources (diffed against 26.1.2's: 172 of
391 differ — mostly the `ResourceLocation`→`Identifier` rename, plus
genuine 1.21.11-template adaptations). B6-referenced oracles: 55 (38
fail on the static evidence, 17 pass on ≥1 classpath provisionally),
plus 2 group-only failing oracles swept from Batch 0 groups
(`VanillaFloatLogOracle`, `VanillaStructurePlacementOracle`). Drivers:
64 total (8 runnable, 14 no-oracle, 42 blocked) — driver list in
`evidence/drivers-1.21.10.json` (read fresh from
`versions/1.21.10/tools/`, 648 drivers total; the B6 row names are
identical to 26.1.2's, none missing).

Runnable drivers (8): biome_temperature, chunk_status,
gateway_replacement, heightmap, placement, retrogen_data,
world_spawn_sync, worldgen_random. No-oracle (14): biome_weather,
chunk_restart_ticks, end_gateway, end_podium, flat_place,
folder_datapack_metadata, inline_feature_parse, place_feature_inline,
sculk_saved_fields, server_player_sculk, spawn_loot, spring,
structure_blocks, unused_feature_startup. Blocked (42): anchor-adjacent
via VanillaAnchorOracle (biome_source_codec, canyon_carver,
cave_carver, height_provider, material_condition, material_rule_library,
material_stage), basic_feature, biome_chunk, biome_generation,
biome_placement, biome_source, biome_zoom, chunk_generation,
datapack_worldgen, decoration, decoration_region, density,
feature_catalog, feature_graph, feature_placement, feature_region,
feature_sort, finalize_spawn, flat_generation, material_extension,
material_surface, natural_spawn, noise, retrogen_chunk, scattered_ore,
sculk, spawn_rules, structure_density, structure_layout,
structure_start_nbt, terrain, tree_feature, world_spawn, worldgen.

Passing oracles (17, provisional): ParityData, VanillaBiomeExport,
VanillaBiomeTemperatureOracle, VanillaBlockPropertiesOracle,
VanillaBlockStateCodecOracle, VanillaBlockSupportOracle,
VanillaBlockSurvivalOracle, VanillaChunkStatusOracle,
VanillaCollisionOracle, VanillaGatewayReplacementOracle,
VanillaHeightmapOracle, VanillaPlacementOracle, VanillaRetrogenDataOracle,
VanillaSimpleBlockOracle, VanillaTagExport, VanillaWorldSpawnPacketOracle,
VanillaWorldgenRandomOracle. (26.1.2 had 18 — VanillaChunkGenerationOracle
passed there and FAILS here; see O08.) Of the 17, 10 are byte-identical
to 26.1.2's sources, 4 differ only by the rename, and 3 carry real
adaptations that check out against the 1.21.10 jar: VanillaPlacementOracle
builds a 15-arg `NoiseRouter` explicitly (the jar's record has 15
components; `NoiseRouterData.none()` is protected in 1.21.10),
VanillaSimpleBlockOracle differs only in a comment, and ParityData uses
the 3-arg synchronous `RegistryDataLoader.load` + `WORLDGEN_REGISTRIES`
(both exist in 1.21.10) with the 26.x `DATA_COMPONENT_INITIALIZERS`
code correctly dropped.

### B6-O01 (major) — Oracles reference the 26.3-only `densityfunction` package (10 oracles)

VanillaAnchorOracle, VanillaBiomeSourceCodecOracle,
VanillaBiomeSourceOracle, VanillaDensityOracle, VanillaFloatLogOracle*,
VanillaMaterialExtensionOracle, VanillaStructureDensityOracle,
VanillaWorldSpawnOracle, VanillaWorldgenOracle,
VanillaMaterialSurfaceOracle (moved from O02 — see below). All 10 still
import `levelgen.densityfunction` in the 1.21.10 tree (verified by grep);
the package has 0 entries in both 1.21.10 jars (verified by jar listing:
`DensityFunction` lives at `levelgen.DensityFunction` in 1.21.10, and
`densityfunction/` has 0 entries). `VanillaAnchorOracle` blocks 7
drivers (biome_source_codec, canyon_carver, cave_carver,
height_provider, material_condition, material_rule_library,
material_stage). `VanillaStructureDensityOracle` imports the 26.x-only
densityfunction API (`SamplerContext`/`DensityVolume`) — record as an
oracle defect, UNREPAIRABLE by design (the API does not exist in
1.21.10), do not force. `VanillaWorldSpawnOracle` additionally uses
`RegistryLayer.WORLD` (see O08). (*FloatLog is group-only: no driver
references it.)

VanillaMaterialSurfaceOracle moved groups: in 26.1.2 it sat in O02 with
a missing-symbol first error; the 1.21.10 source (rename-identical to
26.1.2's) imports `densityfunction.*` (line 15) AND the absent
`material.MaterialSystem` (line 16 — `levelgen/material/` in the
1.21.10 jar holds only MaterialRuleList, WorldGenMaterialRule,
package-info). Its 1.21.10 first failure is the densityfunction import.

### B6-O02 (major) — Oracles reference the missing `material/rule` package (4 oracles)

VanillaBiomeGenerationOracle, VanillaMaterialConditionOracle,
VanillaMaterialStageOracle, VanillaTerrainOracle. All 4 still import
`levelgen.material.rule` in the 1.21.10 tree; the package has 0 entries
in both 1.21.10 jars (surface rules live in `levelgen.SurfaceRules`,
38 entries, in 1.21.10). (VanillaMaterialSurfaceOracle left this group
for O01 — see above.)

### B6-O03 (major) — Oracles use the 26.3 `Feature` constructor signature (3 oracles)

VanillaDecorationOracle, VanillaBiomePlacementOracle,
VanillaFeaturePlacementOracle. All 3 still use `new Feature(){...}`
(no-arg anonymous subclass): DecorationOracle:48, BiomePlacementOracle:39
and :51, FeaturePlacementOracle:33. The 1.21.10 jar's `Feature` ctor
takes only `Codec<FC>` (fresh javap, `javap-feature.txt:140`); there is
no no-arg ctor, and `Feature` has NO `DIRECT_CODEC`
(`ConfiguredFeature` has it). Constructor-mismatch persists verbatim.

### B6-O04 (major) — Oracles fail on missing symbols / signature drift (12 oracles)

VanillaBasicFeatureOracle, VanillaBiomeChunkOracle,
VanillaCanyonCarverOracle, VanillaCaveCarverOracle,
VanillaChunkStageOracle, VanillaCushionPlacementOracle,
VanillaFeatureGraphOracle, VanillaFeatureRegionOracle,
VanillaFeatureSortOracle, VanillaNoiseOracle, VanillaScatteredOreOracle,
VanillaSculkOracle. First-error symbols from the 26.1.2 raw log,
re-verified against the 1.21.10 jar:
BasicFeature/FeatureSort/ScatteredOre — `Feature.DIRECT_CODEC`
(absent in 1.21.10; `ConfiguredFeature.DIRECT_CODEC` exists);
`FeaturePlacer` itself EXISTS in the 1.21.10 jar (1 entry) but does not
save them. BiomeChunk — `chunk.fillBiomesFromNoise(resolver)` with ONE
argument (line 45); the 1.21.10 signature takes TWO
(`BiomeResolver, Climate$Sampler`) — arity failure (the 26.1.2 failure
was lambda parameter types; the 1.21.10 failure is the arity itself).
CanyonCarver/CaveCarver — `WorldCarver.DIRECT_CODEC` (absent; only
`configuredCodec()`), `CarverOutput`, `CarvingMask.minY/maxY/carve`
(`CarvingMask` is at `world.level.chunk.CarvingMask` with
set/get/stream/toArray only — verified by javap). ChunkStage/
FeatureRegion — `ChunkStatus.TERRAIN` (the 1.21.10 set has 12 statuses
— EMPTY, STRUCTURE_STARTS, STRUCTURE_REFERENCES, BIOMES, NOISE,
SURFACE, CARVERS, FEATURES, INITIALIZE_LIGHT, LIGHT, SPAWN, FULL —
no TERRAIN; fresh javap-verified; both oracles still reference TERRAIN:
ChunkStage:27, FeatureRegion:77-78,154). CushionPlacement — the
`blockscan` package (0 entries in the jar) AND `Cushion` itself (0
entries; a 26.x entity). Noise — `GradientNoise`/`Noise`/`NoiseStack`
(synth has only BlendedNoise/ImprovedNoise/NoiseUtils/NormalNoise/
PerlinNoise/PerlinSimplexNoise/SimplexNoise — verified by jar listing).
Sculk — `context.datapackWorldRegistries()` (line 327;
`WorldLoader$DataLoadContext` in 1.21.10 is a record with
`datapackWorldgen()` only — verified by javap), MinecraftServer ctor,
`WardenSpawnTracker` Optional wrapping. Also absent from the jar:
Interval, DensitySampler/SamplerContext, NoiseSpawnFinder,
MaterialCondition/MaterialRule/MaterialSystem, Cushion, Continuation,
BlockMatcher, NativeDestruction, cubemob, `Registries.BLOCK_STATE_PROVIDER`
(1.21.10 has `BLOCK_STATE_PROVIDER_TYPE`).

### B6-O05 (major) — Oracles fail on missing-override lookup (2 oracles)

VanillaDecorationRegionOracle, VanillaFeatureCatalogOracle. Both
sources differ from 26.1.2's ONLY by the ResourceLocation→Identifier
rename, so the 26.1.2 failure modes carry verbatim: the named methods
have no override in the 1.21.10 jar (`getNextEntityId()`,
`startsForStructure` overloads, `getAllStructuresAt`,
`getBiome(int,int,int)`, `markPosForPostProcessing` — note
`ProtoChunk.markPosForPostprocessing`, lowercase p, verified by javap).
FeatureCatalog's errors include a `Feature`-ctor error and a missing
`place(FeaturePlaceContext)` override.

### B6-O06 (major) — Oracles fail on no-override (2 oracles)

VanillaFinalizeSpawnOracle, VanillaTreeFeatureOracle. FinalizeSpawn
line 119 still declares `@Override public int getNextEntityId()` —
`getNextEntityId` exists NOWHERE in 1.21.10's Level, ServerLevel or
LevelAccessor (verified by javap). TreeFeature line 67 still declares
`@Override public void markPosForPostProcessing(...)` with a
`super.markPosForPostProcessing(...)` call — the 1.21.10 method is
`markPosForPostprocessing` (lowercase p), so both the @Override and the
super call fail. Cross-batch overlaps (coordinator dedups):
FinalizeSpawn→B3. (VanillaNaturalSpawnOracle and
VanillaSpawnRulesOracle LEFT this group: their 26.x @Overrides
(`getNextEntityId`, `environmentAttributes`, `getProfile`) were
removed in the 1.21.10 adaptation, but they fail on template-drift
symbols instead — see O08. FinalizeSpawn ALSO uses
`RegistryLayer.WORLD` (O08) as a secondary cause.)

### B6-O07 (major) — Other oracle failures (4 oracles)

VanillaBiomeZoomOracle, VanillaStructureStartOracle,
VanillaHeightProviderOracle (cascade via Anchor),
VanillaStructurePlacementOracle* (group-only, no driver references it).
BiomeZoom: calls `.getBiome(x, y, z)` with three ints (line 22); the
1.21.10 `BiomeManager` has only `getBiome(BlockPos)` (verified by
javap). StructureStart / StructurePlacement: call
`RandomState.create(Registry<Noise>, long, NoiseGeneratorSettings)`
(StructureStart:51, StructurePlacement:89); the 1.21.10 jar offers
only `create(HolderGetter.Provider, ResourceKey<NoiseGeneratorSettings>,
long)` and `create(NoiseGeneratorSettings, HolderGetter<NoiseParameters>,
long)` — neither matches (verified by javap). Both ALSO use
`RegistryLayer.WORLD` (O08) as a secondary cause. HeightProvider:
cascade via Anchor (Anchor still broken — O01). WorldSpawn→B3,
Sculk→B4 per the 26.2 numbering.

### B6-O08 (major — NEW to 1.21.10) — Oracles use 1.21.11-template API absent in 1.21.10 (3 oracles)

VanillaChunkGenerationOracle, VanillaNaturalSpawnOracle,
VanillaSpawnRulesOracle. The oracle sources were adapted for the
1.21.11 era template, and 1.21.10's API differs from 1.21.11's in
these spots (all verified by javap against the 1.21.10 jar):

- `RegistryLayer.WORLD` does not exist in 1.21.10 (`RegistryLayer`
  has STATIC, WORLDGEN, DIMENSIONS, RELOADABLE only);
  `RegistryDataLoader.WORLD_REGISTRIES` does not exist (only
  `WORLDGEN_REGISTRIES`). 18 oracle sources across the tree use them
  (7 in B6 scope); all three O08 oracles use both.
- `RandomState.create(Registry, long, NoiseGeneratorSettings)` has no
  matching overload in 1.21.10 (see O07); the 1.21.10
  VanillaChunkGenerationOracle calls it three times, plus a 6-arg
  flat form that also has no overload. (The 26.1.2 source used the
  `(Settings, HolderGetter, long)` order, which DOES match the
  1.21.10 jar — the 1.21.10 "adaptation" moved it to the order that
  fails.)
- `NoiseRouterData.none()` is PROTECTED in 1.21.10 (called by
  ChunkGeneration's flat branch).
- `ChunkGeneratorStructureState.createForNormal` takes 4 args in
  1.21.10 (`RandomState, long, BiomeSource, HolderLookup`) — the
  oracle passes 5 (with a `ChunkPos`); the class itself moved to
  `world.level.chunk.ChunkGeneratorStructureState`.
- `BiomeSource.createUncachedResolver` does not exist in 1.21.10
  (`BiomeSource` has only abstract `getNoiseBiome` + `BiomeResolver`);
  `Level.uncachedBiomeResolver` does not exist either, so the oracle's
  @Override fails too.

VanillaChunkGenerationOracle PASSED in 26.1.2 and fails in 1.21.10 on
at least 8 sites — it is the single most consequential oracle
regression in this batch: it blocks chunk_generation,
datapack_worldgen, flat_generation and structure_layout (the last
three newly reference it in 1.21.10, moving from no-oracle to
blocked). NaturalSpawn and SpawnRules had their 26.1.2 no-override
cause genuinely fixed (26.x @Overrides removed, 1.21.10 ctor arities,
`GameRules` package, moon brightness) but fail on
`RegistryLayer.WORLD` / `WORLD_REGISTRIES` instead. Four more B6
oracles carry template drift as a SECONDARY cause (their primary
group still fails first): FinalizeSpawn (O06), StructureStart and
StructurePlacement (O07), WorldSpawn (O01). This is the
TEMPLATE-FIRST trap in miniature: 1.21.11 adaptations do not all
carry down to 1.21.10, and the oracle sources need a 1.21.10 pass of
their own.

## Needs-harness

### B6-H (minor) — Paired port-vs-vanilla chunk comparison for 1.21.10

Generate port chunks at seed 6826278913115239597 (the reference
world's seed, confirmed from level.dat's WorldGenSettings) and
compare against the vanilla reference world
(`~/workspace/vanilla-world-mc/1.21.10/New World/`, 8,363 chunks).
**STAGE RULE: the port's `generate_proto` output is terrain-level**
(biomes, noise fill, aquifers, material surface, carvers — the
1.21.10 statuses up to CARVERS; `generate_with_context` runs
fill_noise → build_surface_shared → carvers, `generator.rs:1542+`,
and `decorate` is separate over a 3×3 region,
`generator_decoration.rs:396`); **vanilla region files are
post-decorate**. Compare post-decorate to post-decorate for blocks,
terrain-level to terrain-level for density. Direct comparison is only
valid where structure templates contain no air (ancient city: 0 air)
— trial chambers/villages carve air during placement, so a
terrain-vs-post-decorate gap there is a measurement artifact, not a
density bug (the 1.21.7 −6.4 'beardifier residual' was exactly this).
B6-H is blocked on G06/G09/G11/G12/G13 (no port world can generate
until the construction cascade is fixed) and on G02's census caveat
(this world has no snow and no frozen peaks, so the snow check needs
a different seed or a targeted frozen-peaks world).

## Verified passes

### Data (exact diffs vs the jar)

- **Registry bundles: 713/713 populated entries name-set- and
  content-identical** (biomes 65, densities 35, noises 60, settings 7,
  placed_features 258, processor_lists 40, structures 34,
  structure_sets 20, template_pools 188, world_presets 6).
  `rules.json` / `conditions.json` / `state_providers.json` are
  legitimately empty (no such folders in the jar). `carvers.json` 0
  and `features.json` 0 are G06 (see above), not data errors — the raw
  `data/` copies are 4/4 and 224/224 identical. `feature_tags.json`
  0: the jar's `tags/worldgen/configured_feature` has 0 files in
  1.21.10 (differs from 26.1.2's 1 file).
- **Structure NBTs: 1202/1202 byte-identical.**
- **Flat presets: 9/9 identical** (bottomless_pit, classic_flat,
  desert, overworld, redstone_ready, snowy_kingdom, the_void,
  tunnelers_dream, water_world; the `visible` tag lists all 9).
  `FlatSettings` (`generator.rs:973`) has the 1.21.10 fields (layers,
  biome, structure_overrides, features, lakes); the flat path goes
  through `Pipeline::new` and is blocked by the cascade in practice.
- **Multi-noise presets:** nether preset bytecode (`Preset$1`) —
  nether_wastes all 0; soul_sand_valley humidity −0.5; crimson_forest
  temperature 0.4; warped_forest humidity 0.5, offset 0.375;
  basalt_deltas temperature −0.5, offset 0.175 — matches the prepared
  `biome-parameters/nether.json` exactly (5 entries, identical md5).
  Overworld expansion: 7,593 entries, first mushroom_fields, last
  deep_dark, 54 distinct biomes — byte-identical to the 26.1.2
  prepared file (md5 `79a58ecdb5695a3c…`); both prepared copies
  (`biome-parameters/` and `worldgen/biome_parameters/`) identical.
- **Biome carver lists:** 54× `[cave, cave_extra_underground,
  canyon]`, 5× single-string `minecraft:nether_cave`, 6× empty.
- **Noise settings:** all 7 have `aquifers_enabled`, `surface_rule`,
  `spawn_target` — NO `aquifers` key, NO `material_rule` key (the
  G09/G11 evidence). All 7 have `ore_veins_enabled` (the G18 gate).
- **Template pools (recursive walk):** legacy_single_pool_element
  ×603, single_pool_element ×531, feature_pool_element ×36,
  empty_pool_element ×31, list_pool_element ×3; projections rigid
  ×990, terrain_matching ×183. All 5 element types have decode arms
  (`structure/jigsaw.rs:134-183`).
- **Processor lists:** rule ×39, protected_blocks ×7, capped ×4,
  block_rot ×6 — all 4 types have arms (`structure/processor.rs`:
  rule:801, block_rot:767, protected_blocks:784, capped:789).
- **Structure sets:** concentric_rings ×1, random_spread ×19 — both
  placements present.
- **Structure types:** all 16 have decode arms
  (`structure/start.rs:350-416`): jigsaw ×10, swamp_hut, stronghold,
  shipwreck ×2, ruined_portal ×7, ocean_ruin ×2, nether_fossil,
  ocean_monument, mineshaft ×2, woodland_mansion, jungle_temple,
  igloo, fortress, end_city, desert_pyramid, buried_treasure.

### Vanilla 1.21.10 reference world (own NBT parser, read in place)

World `~/workspace/vanilla-world-mc/1.21.10/New World/` (read in
place; seed **6826278913115239597 confirmed from level.dat
WorldGenSettings**; Level "New World", DataVersion 4556, spawn
[640,71,-416] overworld, GameType 1, Time 2109, vanilla datapacks
only). Dimensions: overworld noise/overworld + multi_noise preset
overworld; nether noise/nether + multi_noise nether; end noise/end +
`the_end` biome source. **16 regions, 8,363 chunks**, x −15..74, z
−66..31. Statuses: full 4,680; structure_starts 2,759; biomes 312;
carvers 305; initialize_light 306; light 1 (no noise/surface chunks;
no TERRAIN status — consistent with the 1.21.10 12-status set).
Total blocks 822,116,352 = 8363×24×4096 exactly.
**16 biomes present** (plains 4,246,272; deep_ocean 2,865,559; ocean
1,427,917; forest 1,221,630; old_growth_birch_forest 790,708;
deep_lukewarm_ocean 540,160; mushroom_fields 440,919; birch_forest
278,807; river 276,350; dark_forest 260,669; dripstone_caves 196,349;
lush_caves 108,618; pale_garden 81,206; beach 80,219; deep_dark
27,785; stony_shore 2,400). **NO frozen_peaks, grove, snowy_slopes,
or jagged_peaks. Snow: 0 blocks world-wide; packed_ice: 60** — the
snow-on-packed-ice census check is vacuous (0/0) for this world, so
G02 rests on code + javap evidence (see the G02 caveat). Water
20,398,344. Sculk family: sculk 10,852; sculk_vein 3,202; sculk_sensor
62; **sculk_catalyst 25; sculk_shrieker 12** (vanilla-side ground truth
for G01). Structure starts: mineshaft 23/2,616 pieces, monument 2/2,
ocean_ruin_warm 1/1, ocean_ruin_cold 11/45, ruined_portal_ocean 3/3,
shipwreck 4/4, trial_chambers 8/1,909, ruined_portal 1/1, trail_ruins
1/18. GD: all jigsaw pieces GD=0 (trial chambers 1909/1909, trail
ruins 18/18); mineshaft GD 0–9. GLD: trial_chambers −54..+23,
trail_ruins −12..+3. No ancient_city, no villages in this world.
Block entities: decorated_pot 240, chest 220, brushable_block 178,
vault 138, dispenser 120, mob_spawner 111, barrel 108, trial_spawner
105, hopper 67, sculk_sensor 62, bed 50, sculk_catalyst 25, beehive
17, sculk_shrieker 12, creaking_heart 7, **DUMMY 1** (vanilla-side
oddity, adjacent note only). 0 chunks with blending_data/retrogen.
First palette entry form: `{'Name': 'minecraft:air'}` (Name/Properties
form — Batch 11's palette item; note only).

### Code-vs-jar spot checks (javap + source)

- **Aquifer engine verified faithful:** `noise_based.rs` +
  `aquifer.rs` byte-identical to 26.1.2's; the 1.21.10
  `NoiseBasedAquifer` bytecode is identical to 26.1.2's after
  normalisation (the only non-artifact difference is
  `MutableDouble.getValue().doubleValue()` vs
  `MutableDouble.doubleValue()` — same value). 13 surface offsets,
  `FLOWING_UPDATE_SIMILARITY`, `computeSurfaceLevel` with
  `OverworldBiomeBuilder.isDeepDarkRegion` — all as verified for
  26.1.2. Only the wiring is dead (G09).
- **Ore feature verified faithful:** `ore_feature.rs` byte-identical
  to 26.1.2's; the 1.21.10 `OreFeature` javap shows the same geometry
  (`Math.sin`/`Math.cos`, `nextFloat`, `OCEAN_FLOOR_WG` heightmap).
- **Sculk spreader core verified faithful:** the spread/charge
  mechanics in `sculk.rs` match the jar except for the three
  documented divergences (G05 water gates, G07 density cap, G08
  cursor form) and the missing tail (G01).
- **Jigsaw GLD verified present:** `jigsaw.rs` byte-identical to
  26.1.2's; `ground_level_delta` / `delta_y` round-trip through NBT
  (`jigsaw.rs:604,611,657,674,692,704`); the reference world's GLD
  ranges (trial chambers −54..+23) match the vanilla form.
- **Tree decode complete:** all 9 trunk placers used in the data
  (+ poplar, unused), all 11 foliage placers (+ poplar), all 10
  decorators used (alter_ground, attached_to_leaves,
  attached_to_logs, beehive, cocoa, creaking_heart, leave_vine,
  pale_moss, place_on_ground, trunk_vine), mangrove root placer,
  both feature sizes (two_layers ×34, three_layers ×5).
- **Snow/freeze feature verified faithful** (see G02): placement
  matches the jar exactly.
- **Chunk stages verified:** `generate_with_context` runs
  fill_noise → build_surface_shared → carvers; `decorate` is
  separate over a 3×3 region — the B6-H STAGE RULE basis.
- **Spawn-finder search formula verified** (see G12): radial search
  and fitness match the jar; only the depth-zeroing / offset
  handling diverges.
- **Biome weather methods identical** between the 1.21.10 and 26.1.2
  jars (`shouldSnow`, `shouldFreeze`, `coldEnoughToSnow`,
  `getTemperature`, `getHeightAdjustedTemperature`,
  `getPrecipitationAt` — method-level normalised diff).

## §5 Re-check verdicts

### 1.21.7 items (PRIMARY — all 10 decided)

| 1.21.7 item | Verdict | 1.21.10 item |
|---|---|---|
| B6-G01 sculk_patch tail | REPRODUCES | B6-G01 (gap, major) |
| B6-G02 snow on packed ice | REPRODUCES (new mechanism: 26.x tag names vs 1.21.10's renamed tags) | B6-G02 (gap, major) |
| B6-G03 water deficit | REPRODUCES (via total aquifer absence, G09) | B6-G03 (gap, major) |
| B6-G04 PoolPiece GD | FIXED | B6-G04 (fixed, minor) |
| B6-O01 densityfunction oracles | REPRODUCES (10 oracles) | B6-O01 (oracle-defect, major) |
| B6-O02 material/rule oracles | REPRODUCES (4 oracles; MaterialSurface moved to O01) | B6-O02 (oracle-defect, major) |
| B6-O03 Feature ctor oracles | REPRODUCES (3 oracles) | B6-O03 (oracle-defect, major) |
| B6-O04 missing-symbol oracles | REPRODUCES (12 oracles) | B6-O04 (oracle-defect, major) |
| B6-O05 other missing packages | REPRODUCES (→ B6-O04/O07; blockscan, etc.) | B6-O04/O07 |
| B6-O06 BiomeZoom group | REPRODUCES | B6-O07 (oracle-defect, major) |

### 26.1.2 items (SECONDARY — all 25 decided)

| 26.1.2 item | Verdict | 1.21.10 item |
|---|---|---|
| B6-G01 sculk_patch tail | REPRODUCES | B6-G01 (gap, major) |
| B6-G02 snow on packed ice | REPRODUCES (26.1.2 had it fixed/withdrawn — 1.21.10 differs because vanilla renamed the snow tags between 1.21.10 and 26.1.2) | B6-G02 (gap, major) |
| B6-G03 water deficit | REPRODUCES | B6-G03 (gap, major) |
| B6-G04 PoolPiece GD | FIXED | B6-G04 (fixed, minor) |
| B6-G05 sculk water gates | REPRODUCES | B6-G05 (gap, minor) |
| B6-G06 empty carver/feature bundles | REPRODUCES | B6-G06 (gap, major) |
| B6-G07 sculk density cap | REPRODUCES | B6-G07 (gap, minor) |
| B6-G08 sculk cursor 26.3 form | REPRODUCES | B6-G08 (gap, minor) |
| B6-G09 aquifers disabled | REPRODUCES | B6-G09 (gap, major) |
| B6-G10 NoiseRouter 8-field | FIXED (router fallback; residual vein-field gap → NEW G18) | B6-G10 (fixed, major) + B6-G18 |
| B6-G11 material_rule required | REPRODUCES | B6-G11 (gap, major) |
| B6-G12 spawn_target decode | REPRODUCES | B6-G12 (gap, major) |
| B6-G13 density arms | REPRODUCES | B6-G13 (gap, major) |
| B6-G14 feature types | REPRODUCES | B6-G14 (gap, major) |
| B6-G15 state-provider names | FIXED (long-form names accepted) | B6-G15 (fixed, major) |
| B6-G16 random_offset | REPRODUCES (10 placed features, not 70) | B6-G16 (gap, major) |
| B6-G17 f32 pipeline | REPRODUCES | B6-G17 (gap, minor) |
| B6-O01 densityfunction group | REPRODUCES (→ B6-O01; 10 oracles, +MaterialSurface) | B6-O01 |
| B6-O02 material/rule group | REPRODUCES (→ B6-O02; 4 oracles) | B6-O02 |
| B6-O03 Feature ctor group | REPRODUCES (→ B6-O03; 3 oracles) | B6-O03 |
| B6-O04 missing-symbols group | REPRODUCES (→ B6-O04; 12 oracles) | B6-O04 |
| B6-O05 missing-override group | REPRODUCES (→ B6-O05; 2 oracles) | B6-O05 |
| B6-O06 no-override group | REPRODUCES (→ B6-O06, 2 oracles; NaturalSpawn/SpawnRules moved to O08) | B6-O06 + B6-O08 |
| B6-O07 other oracle failures | REPRODUCES (→ B6-O07; 4 oracles) | B6-O07 |
| B6-H paired comparison | needs-harness (new seed/world) | B6-H |

No UNVERIFIED carry-overs. No NEVER EXISTED verdicts in this batch
(the two known never-existed corrections from other batches — B5-G18
bell `isProperHit`, B10-G04 BE renderers — are out of B6 scope and were
not re-minted).

## §6 Systemic findings

1. **The construction cascade is the whole story for 1.21.10
   worldgen.** G06 → G13 → G11 → G12, with G09 silent behind, means no
   1.21.10 world generates at all. Every downstream defect (G01, G03,
   G05, G07, G08, G14, G16, G18) is latent behind it. The fixes are
   ordered: prepare folder names (G06), density arms (G13), surface_rule
   lookup (G11), spawn-target field handling (G12), aquifer wiring
   (G09) — then decoration (G14/G16) and veins (G18) become reachable.
2. **The 1.21.10 port is two fixes ahead of 26.1.2 and one regression
   behind.** G10 (router fallback) and G15 (long-form providers) are
   genuine 1.21.10 adaptations that 26.1.2 lacks; G02 is a genuine
   1.21.10 regression that 26.1.2 does not have (vanilla's tag rename
   sits between the two versions). Version-specific verification is
   not optional: neither direction of carry-over is safe.
3. **The oracle sources carry 1.21.11-template drift into 1.21.10
   (O08).** 18 oracle sources use `RegistryLayer.WORLD` /
   `WORLD_REGISTRIES`, which exist in neither the 1.21.10 jar nor (by
   the 26.1.2 evidence) the 26.1.2 jar — they are 1.21.11-era API.
   The oracle tree needs a 1.21.10 pass of its own; the most
   consequential casualty is VanillaChunkGenerationOracle, which
   passed in 26.1.2 and fails in 1.21.10 on at least 8 sites, blocking
   4 drivers. The same drift class may affect other batches' oracles
   (11 of the 18 WORLD-users are outside B6 scope) — the coordinator
   should flag it sweep-wide when Batch 0 lands.
4. **Silent-empty tag loads hide version drift.** Both tag loaders
   return empty sets for missing tags with no error (G02, G07). The
   same pattern would hide any future tag rename. A loud failure (or
   at least a warning) on a missing tag that the code expects would
   have surfaced G02 and G07 at load time.
5. **The 26.3-leakage pattern persists in the feature code while the
   live code was adapted.** `level/sculk.rs` counts sensors/shriekers
   directly with a comment noting the tag does not exist in 1.21.10,
   while `worldgen/sculk.rs` still reads the 26.3-only tag (G07); the
   snow survival code reads 26.x tag names (G02) while the placement
   feature was adapted. Adaptation work lands unevenly across the two
   code paths — the leakage-audit tool should cover `worldgen/` tag
   reads, not just the live paths.

## Prior verification referenced (not re-diagnosed)

- The 26.1.2 Batch 6 report (27 checks, 19 pass, 8 gap; 25 items) is
  the template for structure and method; its checks were re-run fresh
  for 1.21.10, not copied. The 1.21.7 Batch 6 slice (69 checks, 34
  pass, 35 gap; 10 items) is the PRIMARY re-check source.
- The 26.1.2 Batch 0 oracle table is the provisional authority for
  oracle status until the 1.21.10 table lands (see §0).
- The 26.2 evidence scripts `coldump.py` + `nbtmini.py` were reused
  verbatim for the census; `datadiff.py` / `decode_audit.py` /
  `census.py` were adapted from the 26.1.2 run for the 1.21.x layout
  (regions at `region/`, seed from level.dat).
- The 1.21.10 inventory (`docs/sweep/1.21.10/inventory.md`: 1,166
  blocks, 29,671 states, 153 entities, 1,488 items, 115 particles, 65
  biomes) was taken as read; B6 counted only its own registries fresh
  (713 populated entries; the brief's "952 entries" sums to 943 by its
  own component list — B6 reports its own counted numbers).

## Open items (not defects)

- **Batch 0 for 1.21.10** (see §0): re-verify the oracle triage
  (especially O08 and the 17 provisionally-passing oracles) against
  `docs/sweep/1.21.10/batch-0-oracles.json` when it lands.
- **G02 live confirmation**: the reference world has no snow and no
  frozen peaks; a frozen-peaks seed (or a targeted world) is needed
  to measure the snow-on-packed-ice divergence in-game.
- **G18 blast radius**: ore-vein density in vanilla 1.21.10 worlds
  (how much iron/copper ore comes from veins vs `scattered_ore`
  features) was not measured; the defect is the absence of the path,
  not a measured ore deficit.
- **G10 residual audit**: the 7 undecoded router fields' full
  consequence set (barrier/fluid-level/lava feed the aquifer — G09;
  veins feed G18) is covered; no other consumer of those fields was
  found in the port.

## Adjacent findings (not B6 defects)

- **Live sculk water gates** (`level/sculk.rs` ~611, ~385): the same
  `WATER_SOURCE` narrowing as G05 in live gameplay code — for B4.
- **Live sculk density cap is CORRECT** (`level/sculk.rs`
  `sculk_can_place_growth` counts sensors/shriekers directly, ≥3 →
  false) — the feature code (G07) is the only wrong copy.
- **Port `ChunkStatus` enum is the 26.3 single-TERRAIN 10-status form**
  (`chunk_status.rs:8`); vanilla 1.21.10 has 12 statuses (NOISE /
  SURFACE / CARVERS, no TERRAIN). Inert as in 26.1.2: the only NBT use
  is retrogen `target_status`, and 0 reference-world chunks carry
  retrogen/blending data. Not minted.
- **Reference-world DUMMY block entity (1)**: a vanilla-side oddity
  in the census; no port implication.
- **First palette entry form** (`{'Name': 'minecraft:air'}`):
  vanilla 1.21.10 writes Name/Properties palettes; `chunk.rs` has no
  `from_palette_nbt` and `BlockState::from_nbt` reads strict `"id"`
  only — Batch 11 owns the palette item; B6 notes only.
- **Generator test paths**: `generator.rs:5056` carries a test
  `#[ignore = "needs prepared resources in HORIZON_TEST_RESOURCES"]`
  — test-only, no 26.3 path hardcoding found in the 1.21.10 tree's
  generator tests (the 26.1.2 run's 4077/4116/4309 lines do not exist
  in this form here).
- **Template-drift oracles outside B6 scope**: 11 of the 18
  `RegistryLayer.WORLD` users belong to other batches
  (VanillaAudioAttributesOracle, VanillaBlockUpdateOracle,
  VanillaChatScreenOracle, VanillaEnchantingOracle,
  VanillaEntityDeathLootOracle, VanillaEnvironmentOracle,
  VanillaLootFillOracle, VanillaMobAttributesOracle, VanillaRaidOracle,
  VanillaSituationalMusicOracle, VanillaTradeComponentsOracle) —
  flagged for the coordinator, not triaged here.

## Method notes

- javap diffs vs 26.1.2 were normalised (constant-pool `#N` →
  `#N`, `Compiled from` line, bytecode offsets stripped, whitespace
  collapsed, `ldc_w` → `ldc`) before comparison; the raw diffs are
  dominated by constant-pool renumbering (the 1.21.10 jar is
  remapped). Normalised files in `evidence/norm-tmp/`. Method-level
  comparison (biome weather methods) used the same normalisation
  plus branch-target stripping.
- The decode audit's glob is non-recursive (same flaw as the 26.1.2
  run); the recursive census (`evidence/recursive-census.log`)
  supersedes it for density/feature type counts. Known grep
  false-positive classes encountered again: `seagrass`/`weeping_vines`
  "ok" (string present, no dispatch arm — counted in G14);
  `end_islands` "ok" (only the `end_outer_islands` arm exists);
  `magma_cube`/`wither_skeleton`/`zombified_piglin` "MISSING"
  (entity strings in `fortress.json` spawn overrides, not missing
  structure arms).
- census2/census3 (snow-on-packed-ice same-section + boundary checks)
  were NOT re-run: `census.py` already established snow = 0
  world-wide across ALL chunks, so both checks are trivially 0/0;
  re-scanning the regions would add nothing. The census log is
  flooded with numpy DeprecationWarnings about uint64 conversion —
  harmless, same as prior runs.
- The 26.1.2 run's "64 drivers" count is the B6 subset of the 648
  drivers in `tools/`; the `tests/` dir holds 56 unittest wrappers
  (55 drivers + the `test_parity.py` meta-test) in both trees.
- No `minecraft:gradient` / `minecraft:cache` density types exist in
  the 1.21.10 data; no `rule_based_state_provider` uses exist in the
  1.21.10 data (the rule-based form is a standalone record in
  `disk_*` features).

## §7 Evidence index

All paths under `~/workspace/mcir-work/sweep-1.21.10-batches/b06/`
unless stated otherwise.

- `evidence/datadiff.py`, `evidence/datadiff.log` — 713/713 registry
  diff; `evidence/jar-data/` — extracted jar data (7,513 files).
- `evidence/decode_audit.py`, `evidence/decode_audit.log`,
  `evidence/decode-audit.json` — decode-coverage audit (G13/G14/G15/
  G16 evidence, incl. false-positive classes).
- `evidence/recursive-census.log` — recursive density/feature census.
- `evidence/census.py`, `evidence/census.log`,
  `evidence/world-1.21.10-census.json` — reference-world census
  (seed, statuses, biomes, structures, BEs).
- `evidence/coldump.py`, `evidence/nbtmini.py` — reused 26.2 NBT
  tooling (verbatim).
- `evidence/drivers-1.21.10.json` — B6 driver→oracle map + states
  (64 drivers; 57 oracles).
- `evidence/javap-*.txt` (22 dumps) — fresh `javap -c -p` on the
  named 1.21.10 server-game jar: SculkPatchFeature, SculkBlock,
  SculkBehaviour$1, SculkSpreader$ChargeCursor, SculkSpreader,
  Aquifer$NoiseBasedAquifer, Climate$SpawnFinder, SnowLayerBlock,
  Biome, DensityFunction, Feature, ConfiguredFeature, ChunkStatus,
  NoiseRouter, BlendingData, SnowAndFreezeFeature,
  MultiNoiseBiomeSourceParameterList$Preset(+$1), OreVeinifier,
  OreFeature, NoiseBasedChunkGenerator, NoiseChunk.
- `evidence/norm-tmp/*.a` / `*.b` — normalised 1.21.10-vs-26.1.2
  javap diffs (see Method notes).
- Source citations (worktree `versions/1.21.10/`): G01
  `crates/horizon-world/src/worldgen/sculk.rs:163`; G02
  `block_survival.rs:598-599,481-493`, `level/data.rs:431-432,398-400`,
  `level/weather.rs:113,116`, `level/survive.rs:314,316`; G05
  `worldgen/sculk.rs:305,548`; G06 `crates/horizon-prepare/src/lib.rs:
  1492-1512`, `worldgen/generator.rs:486,497`; G07
  `worldgen/sculk.rs:108,556-558`; G08 `worldgen/sculk.rs:263-270,
  642,659-663`; G09 `worldgen/generator.rs:1114-1119`; G10
  `worldgen/router.rs:57-62`; G11 `worldgen/generator.rs:1136-1138`;
  G12 `worldgen/generator.rs:1141-1164`,
  `worldgen/material_condition/library.rs:101`,
  `worldgen/spawn_finder.rs`; G13 `worldgen/density.rs`;
  G14 `worldgen/feature_graph.rs:599,809`,
  `worldgen/vegetation.rs:284-300`, `worldgen/carver/generation.rs:
  26-27`; G15 `worldgen/state_provider.rs:333-456,505-518`,
  `worldgen/disk.rs:65`; G16 `worldgen/placement.rs`; G17
  `crates/horizon-core/src/density/graph/mod.rs:64`,
  `worldgen/climate.rs`, `worldgen/blending.rs`,
  `worldgen/biome_temperature.rs`; G18
  `worldgen/material_condition/rule.rs`,
  `worldgen/material_condition/library.rs:245`,
  `worldgen/material_stage.rs:405-410,552-579`.
- Reference world: `~/workspace/vanilla-world-mc/1.21.10/New World/`
  (read in place; no git, no copies).
- Template: `~/workspace/mcir-work/sweep-1.21.10-ref/26.1.2/reports/batch-6-worldgen.md`
  and `.../26.1.2/slices/batch-06.json`; PRIMARY re-check source
  `~/workspace/mcir-work/sweep-1.21.10-ref/1.21.7/slices/batch-06.json`.

## §8 Verdict

Batch 6 for 1.21.10 is complete: **27 checks (18 pass, 9 gap), 27
items (15 behaviour gaps, 3 fixed behaviour items, 8 oracle groups,
1 needs-harness)**. All 35 re-checks decided, none unverified. The
1.21.10 worldgen port cannot generate any world today (G06→G13→G11→
G12 cascade, G09 silent behind); the two genuine 1.21.10 fixes (G10
router fallback, G15 long-form providers) narrow the gap vs 26.1.2,
while the snow-tag regression (G02), the ore-vein absence (G18) and
the 1.21.11-template oracle drift (O08, breaking the previously
passing VanillaChunkGenerationOracle) are 1.21.10-specific findings
that need version-specific fixes. Oracle triage is provisional pending
the 1.21.10 Batch 0 table.
