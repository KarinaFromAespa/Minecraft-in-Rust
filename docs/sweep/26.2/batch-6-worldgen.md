# Batch 6 — Worldgen / features (26.2 parity sweep)

**Status: committed on branch `parity-sweep-26.2` (coordinator commit).** Static checks only per the task brief (no
cargo builds, no driver runs, no commits — the coordinator commits). Every
result below is javap + source + data-diff evidence, or a re-analysis of
measurements taken before this batch. Nothing here was run.

**Tally:** 79 checks — 51 pass, 28 gap. (25 behaviour checks: 23 pass, 2 gap;
54 oracle status checks: 28 pass, 26 fail.) **60 drivers, 0 run.**
33 drivers have no oracle blocker (19 with passing oracles + 14 with no
oracle) and are ready to run when the block lifts; 27 are blocked on failing
oracles. No driver result is claimed as a pass anywhere in this report.

**Defects:** 2 behaviour items open (`B6-G03` carried over from 1.21.7,
unverified in 26.2; `B6-G05` new, minor), 6 oracle-defect groups
(`B6-O01`…`B6-O06`) covering 26 failing oracles, 1 needs-harness item
(`B6-H`). 3 of the 4 1.21.7 defects are withdrawn with evidence below
(`B6-G01`, `B6-G02`, `B6-G04`). No oracle repairs were done. No production
code was changed (checks + triage only).

Vanilla 26.2's own jar/data is the spec throughout (named server-game jar
`26.1-adapters/versions/26.2/jars/server-game.jar`; client game jar
`leakage-audit/jars/26.2-client.jar`); the 26.3 base's form was never
assumed. Oracle status is the MEASURED Batch 0 table
(`docs/sweep/26.2/batch-0-oracles.json`, javac-measured 2026-11-24).

## Defects

### B6-G03 (major, carried over — unverified in 26.2) — Underground water deficit (−1.04%)

The 1.21.7 world comparison (2026-11-22, port vs vanilla Test World, same
seed, 4,145 paired chunks, both post-decorate) measured water −1.04%
underground with surface water matching. Measured, never root-caused. For
26.2 this batch did the full noise-based aquifer audit the 1.21.7 batch
did not do — and found **no divergence**:

- `computeFluid`: surface sampling over the 13 `SURFACE_OFFSETS` (same
  values and order), `preliminarySurfaceLevel` + 8, center `lower >
  adjusted` → global status, `above` → picker status when non-air, minimum
  taken over the RAW surface — all match the jar.
- `computeSurfaceLevel`: deep-dark exclusion (synthesized from erosion /
  depth: `min(−0.225 − erosion, max(depth − 0.9, 0)) > 0`, matching
  `OverworldBiomeBuilder.isDeepDarkRegion`'s `erosion < −0.225f` and
  `depth > 0.9f` up to float-vs-double constant width ~1e-8 — negligible),
  `clampedMap` factor, floodedness clamp [−1,1], thresholds
  (−0.3→0.8 flooded, −0.8→0.4 dry), flooded → global level, dry →
  randomized level, else `WAY_BELOW_MIN_Y` (−32512).
- `computeRandomizedFluidSurfaceLevel`: `floorDiv` (port `div_euclid`)
  by 16/40/16, base `floorDiv(y,40)*40+20`, spread noise ×10 quantized by
  3 (`Mth.quantize` = `floor(d/3)*3`), `min(minimum, base+offset)`.
- `computeFluidType`: lava gate (level ≤ −10, not WAY_BELOW_MIN_Y, state
  not default lava, `|lavaNoise(floorDiv(x,64), floorDiv(y,40),
  floorDiv(z,64))| > 0.3`).
- `calculatePressure` (lava/water pair → 2.0, gap/midpoint/edge/q with the
  1.5/2.5/3/10 divisors, barrier memoized), `similarity(a,b) = 1 −
  (b−a)/25`, `FLOWING_UPDATE_SIMILARITY = similarity(100,144) = 1 −
  44/25 = −0.76`, grid constants (spacing 16/12, offsets −5/+1).
- Fluid picker: disabled → air at 2×minY; y < min(−54, seaLevel) → lava
  at −54; else default fluid at sea level. Matches the jar.

The aquifer code is also effectively identical between the 1.21.7 and 26.2
trees (the only diff is a test helper's f32→f64 return type), so the 1.21.7
deficit has no identified code mechanism in either tree. It cannot be
re-measured for 26.2: no port world exists on the VM, and the vanilla 26.2
reference world (6,227,889 water blocks over 6,018 chunks) has no port
counterpart. **Status: open, carried over; measurement deferred to B6-H.**
If the paired comparison reproduces the deficit, the cause lies outside the
aquifer core (density, carver fluid fill, or fluid-tick scheduling).

### B6-G05 (minor, new) — Sculk spreader water gates narrowed to water sources

Two sites in `worldgen/sculk.rs` use the `WATER_SOURCE` flag where vanilla
26.2 tests whether the fluid IS water (any water, including flowing):

1. `attempt_spread_vein`, Default branch: vanilla regrows the vein when the
   cursor cell is air OR any water (`fluidState.is(WATER)`); the port
   requires a water source.
2. `can_place_growth`, above-open gate: vanilla accepts air or any water
   above the growth position; the port requires a water source.

Both gates sit beside correct code — the port's `can_spread_into`
(MultifaceSpreader equivalence) and `canChangeBlockStateOnSpread` were
verified against the jar, and flowing water correctly fails `canBeReplaced`
on both sides. Impact is confined to sculk patch generation adjacent to
flowing water (underground waterfalls / lake edges in the deep dark). No
RNG-stream effect: neither gate makes a random call. Same root pattern at
both sites (source-flag used for a fluid-is-water test), so one fix covers
both.

## Withdrawn 1.21.7 defects (fixed in the 26.2 tree, with evidence)

### B6-G01 (major) — WITHDRAWN: sculk_patch tail is implemented

26.2 `worldgen/sculk.rs` decodes `catalyst_chance`, `extra_rare_growths`,
the catalyst (default sculk_catalyst state) and the shrieker (default +
`can_summon=true`), and `place()` implements the full tail, verified
line-by-line against the 26.2 jar `SculkPatchFeature.place` bytecode:
`nextFloat() <= catalystChance` + below collision-full → catalyst at origin,
flag 3; then `extraRareGrowths.sample` ×
`origin.offset(nextInt(5)-2, 0, nextInt(5)-2)` air-above-face-sturdy →
shrieker, flag 3; catalyst roll before the growth sample (same RNG order).
`canSpreadFrom` also matches (SculkBehaviour → true; else air or
water-source + any neighbour collision-full). The vanilla 26.2 reference
world contains no deep dark (0 sculk blocks of any kind), so there is no
world-side catalyst count to re-measure — the withdrawal rests on the
code-vs-jar verification.

### B6-G02 (major) — WITHDRAWN: snow-on-packed-ice mechanism verified closed

The 1.21.7 measurement (+7.6% snow in frozen peaks, on packed ice) is now
root-caused at the mechanism level, and the mechanism is closed in 26.2:

- Vanilla 26.2 `SnowLayerBlock.canSurvive` returns false when the block
  below is in `#cannot_support_snow_layer` — jar tag data:
  `[ice, packed_ice, barrier]` — before the face-full fallback. That is
  why vanilla places no snow on packed ice.
- The port's `Rule::SnowLayer` (`block_survival.rs`) implements the same
  three steps (deny tag → allow tag `support_override_snow_layer` →
  face-full collision UP or same-block layers==8), and both
  `block_survives` paths reach it (decoration `blocks.survival` and
  `behaviours.can_survive` via `level/survive.rs:295`).
- The 26.2 prepared data contains the tag with exactly the jar's members
  (`block-tags.json`: `[ice, packed_ice, barrier]`).
- `SnowAndFreezeFeature.place` and `Biome.shouldSnow` /
  `shouldFreeze` (precipitation SNOW via height-adjusted temperature <
  0.15, build-height + block-light < 10 gates, air-or-snow cell,
  `canSurvive`) match the jar step by step; the snowy-state map covers
  grass_block / podzol / mycelium, the registry's SNOWY blocks.
- The snow path (`snow_freeze.rs`, `biome_weather.rs`,
  `block_survival.rs`) is byte-identical between the 1.21.7 and 26.2
  trees, so the 1.21.7 measurement is not explained by a code difference
  in this path. The likely explanation is stale prepared tag data on the
  laptop at generation time: `tag_states` silently yields an EMPTY set for
  a missing tag (`.get(name).into_iter().flatten()`), so a prepare lacking
  `cannot_support_snow_layer` would place snow on every packed-ice column
  exactly as measured — a latent footgun, noted here, not a 26.2 defect
  (the tag is present in the 26.2 prepared data).

Caveat: the vanilla 26.2 reference world contains no frozen_peaks biome,
and no port world exists, so the +7.6% figure cannot be re-measured for
26.2; live confirmation is B6-H.

### B6-G04 (minor) — WITHDRAWN: PoolPiece NBT writes `GD`

`PoolPiece::save` (`structure/jigsaw.rs`) goes through
`legacy::piece_tag("minecraft:jigsaw", bbox, None, 0)`, which writes
`("GD", Tag::Int(gen_depth))` — GD=0 present, matching vanilla's base
`StructurePiece` serialization. The reference world confirms the vanilla
form: trial chambers 1,112/1,112 pieces GD=0, trail ruins 17/17 GD=0,
mineshaft GD 0–9 by corridor depth, with `ground_level_delta` the live
propagated value (trial chambers −46…+18, trail ruins −12…+11).

## Oracle defects (Batch 0 measured status, 54 oracles, 28 pass / 26 fail)

| ID | Cause | Oracles | Drivers blocked |
|----|-------|---------|-----------------|
| B6-O01 (major) | 26.3-only `densityfunction` package missing | Anchor, BiomeSourceCodec, BiomeSource, Density, MaterialCondition, MaterialExtension, StructureDensity, Worldgen (8) | biome_source_codec, biome_source, density, material_condition, material_extension, material_rule_library, material_stage, structure_density, worldgen |
| B6-O02 (major) | `material/rule` package missing | BiomeGeneration, MaterialStage, Terrain (3) | biome_generation, material_stage, terrain |
| B6-O03 (major) | 26.3 `Feature` constructor signature | Decoration (1) | decoration |
| B6-O04 (major) | Missing symbols (26.x API drift) | BiomeChunk, ChunkStage, CushionPlacement, FeatureGraph, FeatureRegion, MaterialSurface, Sculk, TreeFeature, WorldSpawn (9) | biome_chunk, chunk_stage, retrogen_chunk, cushion_placement, feature_graph, feature_region, material_surface, sculk, tree_feature, world_spawn |
| B6-O05 (major) | Missing-override `lookup` | DecorationRegion, FeatureCatalog (2) | decoration_region, feature_catalog |
| B6-O06 (major) | Other (signature / factory / no-override) | BiomeZoom (`BiomeManager.getBiome` signature), StructureStart (`NoiseGeneratorSettings.create`), FinalizeSpawn (no-override) (3) | biome_zoom, structure_start_nbt, finalize_spawn |

`VanillaStructureDensityOracle` is UNREPAIRABLE against 26.2 per the brief
(26.3-only `densityfunction` API) — do not force it; density is verified
via bytecode/chunk comparison. The other O01 oracles need A3-style rewrites
against 26.2's own density API.

**Passing oracles (28):** BasicFeature, BiomeExport, BiomePlacement,
BiomeTemperature, BlockProperties, BlockStateCodec, BlockSupport,
BlockSurvival, CanyonCarver, CaveCarver, ChunkGeneration, ChunkStatus,
Collision, FeaturePlacement, FeatureSort, GatewayReplacement, Heightmap,
NaturalSpawn, Noise, ParityData, Placement, RetrogenData, ScatteredOre,
SimpleBlock, SpawnRules, TagExport, WorldSpawnPacket, WorldgenRandom.

**Runnable-but-unrun drivers (19, all oracles pass):** basic_feature,
biome_placement, biome_temperature, canyon_carver, cave_carver,
chunk_generation, chunk_status, feature_placement, feature_sort,
gateway_replacement, heightmap, natural_spawn, noise, placement,
retrogen_data, scattered_ore, spawn_rules, world_spawn_sync,
worldgen_random.

**No-oracle drivers (14, also unrun):** biome_weather,
chunk_restart_ticks, datapack_worldgen, end_gateway, flat_generation,
flat_place, folder_datapack_metadata, nether_forest_vegetation,
sculk_saved_fields, server_player_sculk, spawn_loot, structure_blocks,
structure_layout, unused_feature_startup.

**Blocked drivers (27):** biome_chunk, biome_generation, biome_source,
biome_source_codec, biome_zoom, chunk_stage, cushion_placement, decoration,
decoration_region, density, feature_catalog, feature_graph, feature_region,
finalize_spawn, material_condition, material_extension,
material_rule_library, material_stage, material_surface, retrogen_chunk,
sculk, structure_density, structure_start_nbt, terrain, tree_feature,
world_spawn, worldgen.

Batch 6 driver set for 26.2 is 60 drivers: 53 carried from 1.21.7 (2 gone:
`inline_feature_parse`, `place_feature_inline`) + 7 new (`decoration`,
`decoration_region`, `material_condition`, `material_extension`,
`material_rule_library`, `material_stage`, `nether_forest_vegetation`).

## Needs-harness

### B6-H (minor) — Paired port-vs-vanilla chunk comparison for 26.2

No port world exists for 26.2 on the VM (26.2 was played on the laptop).
When a port world is generated at the reference seed −1514548643533799951,
compare it against the vanilla "26.2 World" (6,018 chunks, 3,002
`minecraft:full`) with the STAGE RULE: the port's `generate_proto` output
is terrain-level (before structure placement/decoration); vanilla region
files are post-decorate. Direct comparison is only valid where structure
templates contain no air (ancient city: 0 air) — trial chambers/villages
carve air during placement, so a terrain-vs-post-decorate gap there is a
measurement artifact, not a density bug. Compare post-decorate to
post-decorate (`minecraft:full` on both sides) for blocks, and
terrain-level to terrain-level for density. This is the live confirmation
for B6-G02 (withdrawn on static evidence), the re-measurement for B6-G03,
and the first 26.2 biome/height/cave-air agreement numbers. Vanilla-side
ground truth is recorded in `evidence/world-26.2-census.json` (block
census, biome census, structure starts, GD/`ground_level_delta` spreads).

## Verified passes

### Data (method 4 — exact diffs vs the jar)

- **Worldgen registries: 952/952 entries name-set- and content-identical**
  (exact JSON equality after parsing the prepared values, which are
  JSON-encoded strings): biomes 66, configured carvers 4, configured
  features 226, density functions 35, noises 63, noise settings 7, placed
  features 262, processor lists 40, structures 34, structure sets 20,
  template pools 188, world presets 7. Prepared `state_providers.json` /
  `conditions.json` / `rules.json` are empty dicts — expected in 26.2
  (code-only, no registry resources; confirmed by PORT-NOTES-26.2.md).
- **Structure NBTs: 1212/1212 byte-identical** (jar
  `data/minecraft/structure/` vs prepared `worldgen/structure/minecraft/`).
- **Flat presets 9/9 identical** (jar vs prepared
  `data/minecraft/worldgen/flat_level_generator_preset/`), and the
  `visible` tag lists all 9 (classic_flat, tunnelers_dream, water_world,
  overworld, snowy_kingdom, bottomless_pit, desert, redstone_ready,
  the_void).
- **Multi-noise nether preset: exact vs jar bytecode** — nether_wastes all
  zeros; soul_sand_valley humidity −0.5; crimson_forest temperature 0.4;
  warped_forest humidity 0.5, offset 0.375; basalt_deltas temperature −0.5,
  offset 0.175. Prepared `biome-parameters/nether.json` matches all 5
  entries exactly.
- **Multi-noise overworld expansion (7,594 entries): verified by provenance
  + structure.** The table is vanilla's own datagen export
  (`26.2/versions/26.2/datagen/reports/biome_parameters/minecraft/`),
  copied verbatim by prepare — jar-derived, not hand-transcribed. First
  entry mushroom_fields (continentalness [−1.2,−1.05]), last deep_dark.
  Full entry-by-entry verification would need running the builder.

### Vanilla 26.2 reference world (method — own NBT parser, read in place)

"26.2 World", seed **−1514548643533799951** (confirmed in
`data/minecraft/world_gen_settings.dat` under `data.seed`; no
`WorldGenSettings` in level.dat — 26.x form), spawn (256, 96, 188)
overworld, generator `minecraft:noise`, settings `minecraft:overworld`,
biome source multi_noise preset overworld; DataVersion 4903; datapacks
vanilla only (minecart_improvements, redstone_experiments,
trade_rebalance disabled). 12 overworld regions, **6,018 chunks**
(3,002 `minecraft:full`, 2,288 `structure_starts`, 250 `biomes`, 242
`carvers`, 234 `initialize_light`, 2 `light`), xPos −14…49, zPos −16…78.

- **Biomes present (21 of 66):** plains, cold_ocean, swamp, forest,
  old_growth_birch_forest, taiga, windswept_forest, river,
  windswept_hills, frozen_ocean, ocean, dripstone_caves, dark_forest,
  beach, old_growth_pine_taiga, **sulfur_caves** (48,505 cells),
  birch_forest, lush_caves, snowy_beach, windswept_gravelly_hills,
  ice_spikes. No deep_dark (hence no sculk ground truth), no frozen_peaks
  (hence B6-G02 unmeasurable here), no nether/end regions.
- **Block census (591,593,472 blocks counted):** sulfur 1,124,645,
  cinnabar 869,049, sulfur_spike 3,016, potent_sulfur 70 — the new 26.2
  content generates heavily in vanilla. Snow 2,917 vs packed_ice 128,228
  (snow exists in taiga/snowy beach/ice spikes; the frozen-peaks
  packed-ice question does not arise in this world). Sculk family: 0.
- **Structure starts:** mineshaft 20 (2,678 pieces), ocean_ruin_cold 5,
  ruined_portal 2, shipwreck 3, trail_ruins 1 (17 pieces), trial_chambers
  4 (1,112 pieces). GD/`ground_level_delta` spreads recorded above
  (B6-G04). Block entities include trial_spawner 60, vault 94,
  mob_spawner 112, potent_sulfur 62.

### Code-vs-jar spot checks (method 2 — javap + source)

- **Jigsaw ground_level_delta: PASS.** Vanilla bytecode
  (`JigsawPlacement$Placer`): rigid child → `parentGD − delta_y` where
  `delta_y = parentJigsawY − childJigsawY + frontStepY`; non-rigid child
  → element GD (base `getGroundLevelDelta()` returns 1, no subclass
  overrides). Target Y: both rigid → source_y + delta_y; else base height
  (WORLD_SURFACE_WG at parent jigsaw x/z, cached with MIN sentinel) −
  target local Y. Start-piece adjustment: `piece.move(0, bottom −
  (bbox.minY + groundLevelDelta), 0)`, cx/cz = (max+min)/2. The port
  (`jigsaw.rs`) implements exactly this, with Single-element default 1.
- **Sculk spreader core: PASS.** Worldgen spreader constants match exactly
  (growthSpawnCost 50, noGrowthRadius 1, chargeDecayRate 5,
  additionalDecayRate 10, MAX_CURSORS 32). Post-move 15-block check
  matches (jar kills the cursor when NOT closerThan(origin-x/z, 15.0);
  port `dx² + dz² >= 225 → charge = 0`); stuck cursor (null movement pos)
  is NOT killed in 26.2 — port matches. `canChangeBlockStateOnSpread`
  (SculkBlock overrides false), `getSculkSpreadDelay` default 1, default
  behaviour (attemptUseCharge / updateDecayDelay), vein attemptUseCharge
  (spread_veins && place → charge−1; else 1-in-5 floor(charge×0.5)), Sculk
  attemptUseCharge (decay roll, 1-in-11 shrieker else sensor with
  waterlogging, decay penalty formula), `canPlaceGrowth` box
  (−4,0,−4)..(4,2,4) with ≥3 sensor/shrieker → false, `onDischarged`,
  `valid_movement` (18 non-corner neighbours shuffled; the substrate check
  uses the `sculk_replaceable` TAG — confirmed the port's
  `self.replaceable` is the tag set), `MultifaceSpreader` /
  `stateCanBeReplaced` (sculk-blocker set, manhattan-2 sturdy check,
  fluid/fire gates) — all verified equivalent. Only the two water gates
  diverge (B6-G05).
- **Ore feature: PASS (code level).** `ore_feature.rs` is a bit-careful
  port of vanilla `OreFeature`: direction `nextFloat × π`, spread size/8,
  max_radius `ceil((size/16×2+1)/2)`, ends via sin/cos, y0/y1
  `origin + nextInt(3) − 2`, OCEAN_FLOOR_WG heightmap gate, sphere chain
  (`Mth.lerp`, `ss = nextDouble × size/16`, `r = (sin(π·step)+1)·ss+1)/2`
  with the 65536-entry sine table = `Mth.sin`, float arithmetic preserved),
  sphere culling, BitSet index formula, `ensureCanWrite`,
  `shouldSkipAirCheck` (discard≤0 → skip; ≥1 → check; else
  `nextFloat() >= discard`). The in-tree test proves the sphere cell
  visitor matches vanilla's box scan exactly (20,000 random spheres).
  ScatteredOre (count `nextInt(size+1)`, spread `min(attempt,7)`,
  `(nextFloat−nextFloat)×spread + 0.5` floor) also matches. The `heights`
  memoization is sound by construction (pure rules only, no random draws).
  Ore rule tests used: `tag_match` 43, `block_match` 6 — both have arms.
- **Aquifer core: PASS.** Full audit against the 26.2 jar (see B6-G03
  above): computeFluid, computeSurfaceLevel, computeRandomizedFluidSurfaceLevel,
  computeFluidType, calculatePressure, similarity, grid constants, fluid
  picker. No divergence found.
- **Spawn finding: PASS.** `Climate$SpawnFinder` verified against the jar:
  fitness(0,0), radialSearch(2048, 512) then (512, 32); rings `increment`
  apart sampled every `increment` blocks; origin captured once per search;
  `x = originX + (int)(sin(angle)*radius)`, `z = originZ +
  (int)(cos(angle)*radius)`; angle += increment/radius in f32, reset at
  2π; fitness = best target fitness × 2048² + (x²+z²); climate sampled at
  the quart corner with depth zeroed. `ParameterPoint.fitness` (Σ
  square(distance) + offset²) and `Parameter.distance` (above-max, else
  max(min−value, 0)) match exactly. The generator passes the noise
  settings' spawn target, or 0 0 when empty.
- **Chunk stages: PASS.** Carvers run inside `generate_proto` (terrain
  stage: noise fill → surface/material rules → `generate_carving_mask` +
  carve profile with the lava anchor), and `decorate` is a separate later
  step taking a 3×3 region. Stage discipline holds (basis for the B6-H
  STAGE RULE).
- **Flat preset loading: PASS (data-driven).** `FlatSettings` reads layers
  (bottom-up block + height), biome, structure_overrides, features, lakes
  from the preset settings; `FlatLayers` mirrors
  `FlatLevelGeneratorSettings.getLayers` including null-layer handling
  (skipped by `getBaseHeight`, read as air by `getBaseColumn`, left for
  the `FillLayerFeature` after structures).
- **`would_survive` predicate: PASS (spot).** Delegates to
  `world.block_survives` (the tag-aware survival system).
- **Sulfur features (new 26.2 content): PASS (decode-shape).** All five
  new features use existing types with standard configs:
  `sulfur_spike_cluster` (speleothem_cluster — full key set incl.
  `replaceable_blocks`, `wetness`, `density`; decodes via
  `dripstone.rs::SpeleothemClusterFeature`), `sulfur_pool` (sequence),
  `rooted_sulfur_spring` (root_system), `sulfur_spring`
  (weighted_random_selector), `sulfur_spike` (simple_random_selector).
  `sulfur_caves` biome (temperature 0.8, downfall 0.4, has_precipitation
  true; carvers cave/cave_extra_underground/canyon; sulfur features at the
  lakes and underground-decoration steps; 8 spawner categories; effects
  only grass/water colour) has no exotic field the port's biome handling
  would miss. The 66th biome is `sulfur_caves`.
- **Decode-coverage audit: PASS (dispatch arms present).** All 55
  configured-feature types used in 26.2 data (tree+fallen_tree one arm;
  huge_red+brown mushroom one arm; vegetation_patch+waterlogged one arm;
  end_island/end_platform/void_start_platform via `BasicFeature`; unknown
  types are decode ERRORS, not silent skips), 15 placement modifier types,
  9 trunk + 11 foliage + 1 root placer, 9 tree decorators, 8 state
  providers, 25 density-function types used (56 arms total), 11 material
  conditions, 4 material rules (`vertical_gradient` is a CONDITION arm,
  `bandlands` a RULE arm; no `minecraft:ore_vein` arm is correct for 26.2
  — 26.3-only; 26.2 veins are applied by the generator directly), 16
  structure types, 2 placements (random_spread 19, concentric_rings 1), 3
  carver types, 5 pool-element types, 4 processor types, 7 block-predicate
  types, height providers (uniform/trapezoid/very_biased_to_bottom +
  biased_to_bottom/constant arms), 8 int value providers, 8 rule-test arms
  (always_true, block_match, blockstate_match, random_block_match,
  random_blockstate_match, tag_match + iron_chain/rail). 12 extra feature
  arms exist with no data use. Caveat: arm presence ≠ correct
  implementation — B6-G05 was a silent narrowing inside present arms.

### Prior verification referenced (not re-diagnosed)

26.2's own port notes record the 26.2 worldgen API shape (pre-restructure:
no `densityfunction` / `material.rule` packages; material rules,
conditions and state providers code-only; item 423 sculk in the 26.2 form;
item 347 jigsaw `StructureBlockInfo` rename). PORT-NOTES item 115 records
beardifier float-vs-double precision as density-engine-wide debt, recorded
not reworked — a standing caveat for density comparisons, not a new defect.
The 1.21.7 terrain/carver/beardifier/trees/water fixes are 1.21.7-tree
history and are not re-verified here; 26.2's equivalents are the checks
above.

## Open items (not defects)

1. **B6-H: paired port-vs-vanilla chunk comparison** (see above) — the
   live confirmation for B6-G02 and the re-measurement for B6-G03, plus
   first 26.2 biome/height/cave-air agreement numbers.
2. **Run the 33 unblocked drivers** when the execution block lifts; repair
   oracles for the 27 blocked drivers (B6-O01…O06; O01's StructureDensity
   stays unrepaired by design).
3. **Nether/End: no ground truth exists.** The reference world is
   overworld-only (12 regions). Nether preset verified against jar
   bytecode; End features verified only by dispatch-arm presence.
   Fixture-server comparison or fresh vanilla exploration needed.
4. **Full entry-by-entry overworld multi-noise verification** would need
   running `OverworldBiomeBuilder` (see above — verified by provenance).
5. **Deep dark / frozen peaks are absent from the reference world**, so
   sculk counts and the snow-on-packed-ice figure have no vanilla-side
   numbers in this world; a second reference world (or fresh vanilla
   exploration) covering those biomes would close the loop.

## Adjacent findings (not B6 defects)

- The `tag_states` silent-empty footgun (missing tag → empty set, no
  error) likely explains the 1.21.7 snow measurement; it is a robustness
  gap in the shared survival loader, not a 26.2 worldgen defect (all tags
  used here are present in the prepared data). Flagged for the owning
  batch, not ledgered here.
- The aquifer deep-dark exclusion synthesis uses double constants −0.225 /
  0.9 where vanilla uses the float-widened −0.22499999403953552 /
  0.8999999761581421 — a ~1e-8 boundary window, negligible (continuous
  noise inputs).
- Bubble columns and beds block entities are owned by other batches
  (1.21.7 findings; not re-checked for 26.2 here).

## Method notes

- Stage discipline was enforced throughout: no terrain-level output was
  compared against post-decorate vanilla regions; the vanilla world census
  is vanilla-side ground truth only, with no port counterpart claimed.
- The wrong-seed VM world (`terrain2/vanilla/world`, seed
  −8343112210435233944) was never used. All comparisons use the
  vanilla-world-mc "26.2 World", seed −1514548643533799951. World files
  were read in place with an own NBT parser; no git commands were run in
  `~/workspace/vanilla-world-mc`, and no world files were copied or moved.
- Oracle status was taken from the measured Batch 0 table
  (`docs/sweep/26.2/batch-0-oracles.json`); no import-level static check
  was substituted. Driver→oracle mapping replicates
  `docs/sweep/measure-oracles.py`'s parsing logic (656 drivers total).
- `GD` vs `ground_level_delta` is a trap for future readers: the two keys
  coexist in vanilla piece NBT, `GD` is genDepth (0 for jigsaw pieces),
  `ground_level_delta` is the live propagated value.
- Evidence (javap dumps, driver map, oracle triage, world census, analysis
  scripts) is in `evidence/` next to this report.

## Coordinator note

Reconciled Nov 24, 2026 against the measured Batch 0 table
(`docs/sweep/26.2/batch-0-oracles.json`), the 26.2 tree, and the vanilla
26.2 reference world (read in place; no git commands in
`~/workspace/vanilla-world-mc`, no world files copied or moved):

- **Oracle + driver triage all verify exactly (0 mismatches):** all 26
  failing oracles fail on all three classpaths; all 28 passing oracles pass
  on client+both; all 19 runnable drivers' oracles pass; all 14 no-oracle
  drivers have no oracle in the driver map; all 27 blocked drivers map to
  failing oracles. Verified programmatically against the measured table and
  the batch's `evidence/oracle-triage-26.2.json` / `drivers-26.2.json`.
- **Tally convention matches the 1.21.7 template:** Batch 6 counts oracle
  status checks in its tally (79 = 25 behaviour + 54 oracle status), exactly
  as the 1.21.7 Batch 6 did (69 = 22 + 47). The other 26.2 batches do NOT
  count oracle status in their tallies, so the merged sweep totals mix
  conventions — the merge must present behaviour checks and oracle-status
  checks separately. Behaviour tally reconstruction (the report does not
  itemize per-check): 5 defect checks (G01–G05) + 5 data checks + 10
  code-vs-jar checks + 5 census checks (world facts, biomes, blocks,
  structures, GD spreads) = 25; 23 pass, 2 gap.
- **B6-G03 — kept as recorded, but it is UNVERIFIED for 26.2:** no 26.2
  evidence of the underground water deficit exists. The full aquifer audit
  found no divergence against the 26.2 jar, and the aquifer code is
  effectively identical between the 1.21.7 and 26.2 trees. The item rests
  solely on the 1.21.7 measurement (−1.04%, 4,145 paired chunks) plus the
  identical-code inference. B6-H (paired comparison at the reference seed)
  is the arbiter; if it reproduces, the cause lies outside the aquifer
  core. Do not present G03 as a demonstrated 26.2 defect.
- **"Fixed" status on B6-G01/G02/G04 means "1.21.7 defect withdrawn for
  26.2 — verified absent", NOT a fix applied during this sweep.** No fixes
  were made in any batch. At merge these count as withdrawn, not live
  defects. G02's withdrawal rests on static evidence only (no frozen_peaks
  in the reference world, no port world); B6-H is its live confirmation.
- **Cross-batch overlaps — per-oracle dedup at merge, inside B6-O04/O06:**
  VanillaFinalizeSpawnOracle folds into B3-O03, VanillaWorldSpawnOracle
  into B3-O04, VanillaSculkOracle into B4-O14. The other 23 oracles in
  B6-O01–O06 are first per-batch records. No whole-item duplicates.
- **Defect + census spot-checks (all verified):** B6-G05 — `sculk.rs:352`
  (`attempt_spread_vein` Default branch) and `:595` (`can_place_growth`
  above-open gate) both require `WATER_SOURCE` where vanilla tests
  fluid-is-water. Reference-world seed confirmed by the coordinator from
  `data/minecraft/world_gen_settings.dat` (`data.seed` =
  −1514548643533799951); census numbers verified against
  `evidence/world-26.2-census.json` (6,018 chunks, 3,002 full, water
  6,227,889, sulfur-family 1,127,731, trial chambers 1,112 pieces, trail
  ruins 17, no frozen_peaks / deep_dark, 0 sculk blocks). Note: the census
  JSON's own `seed`/`spawn` fields are null (that script reads level.dat
  only) — the seed confirmation in the report comes from
  world_gen_settings.dat, verified independently above.
