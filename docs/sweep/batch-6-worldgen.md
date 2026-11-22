# Batch 6 — Worldgen / features (1.21.7 parity sweep)

**Status: DRAFT — uncommitted.** An execution-risk notice blocked all repo
execution (no driver runs, no cargo/javac, no git commands) for this batch.
Per the task brief's fallback clause (Batch 2 precedent), this report and
`batch-06.json` are static-analysis deliverables saved under
`~/workspace/mcir-work/sweep-batch-6-deliverables/`, awaiting the user's
go-ahead to commit to `parity-sweep`. Nothing here was run; every result
below is javap + source + data-diff evidence, or a re-analysis of
measurements taken before the block.

**Tally:** 69 checks — 34 pass, 35 gap. (22 behaviour checks: 18 pass, 4 gap;
47 oracle status checks: 16 pass, 31 fail.) **54 drivers, 0 run.**
24 drivers have no oracle blocker (12 with passing oracles + 12 with no
oracle) and are ready to run when the block lifts; 30 are blocked on failing
oracles. No driver result is claimed as a pass anywhere in this report.

**Defects:** 4 behaviour (`B6-G01`…`B6-G04`), 6 oracle-defect groups
(`B6-O01`…`B6-O06`) covering 31 failing oracles. No oracle repairs were done
(blocked). No production code was changed (checks + triage only).

Vanilla 1.21.7's own jar/data is the spec throughout (named jar
`1.21.7-server-named.jar`); the 26.3 base's form was never assumed.

## Defects

### B6-G01 (major) — `sculk_patch` never places sculk catalysts or extra rare growths

Vanilla `SculkPatchFeature.place` (jar bytecode, verified) does two things
after the spreader rounds that the port omits entirely:

1. **Catalyst:** `if (random.nextFloat() <= catalystChance (0.5)` and the
   block below the origin is collision-full, `setBlock(origin,
   SCULK_CATALYST, 3)`.
2. **Extra rare growths:** `extraRareGrowths.sample(random)` times, pick
   `origin.offset(nextInt(5)-2, 0, nextInt(5)-2)`; if that cell is air above
   a face-sturdy block, place `SCULK_SHRIEKER[can_summon=true]`.

The port's `worldgen/sculk.rs` `place()` ends after the spreader rounds;
`decode` ignores `catalyst_chance` and `extra_rare_growths`; the string
`catalyst` appears nowhere in the file. Serde's unknown-field tolerance
means the dropped config reads as success — no error, no warning.

**Evidence this is the world-compare catalyst gap:** the 58 ancient-city
templates contain zero sculk catalysts (10 sculk sensors only), so all
1,278 vanilla catalysts in the comparison come from `sculk_patch` — and the
port has 0. Both configured features are affected
(`sculk_patch_ancient_city`: `catalyst_chance` 0.5, `extra_rare_growths`
uniform 1..3; `sculk_patch_deep_dark`: 0.5, 0).

**Second-order effect:** the missing `nextFloat()`/`nextInt()` calls shift
the RNG stream for every feature placed after a `sculk_patch` in the same
chunk. This is the likely mechanism behind the PORT-NOTES sculk-vein cell
divergence (see Open items) — the spreader itself is faithful (below).

### B6-G02 (major) — Snow placed on packed ice in frozen peaks (+7.6%)

World comparison 2026-11-22 (port "New World (1)" vs vanilla "Test World",
same seed 8257987075890829262, 4,145 paired chunks, both post-decorate):
snow +7.6% systematic in frozen peaks — the port places snow on packed ice
where vanilla places none. Measured, not root-caused. Under the separate
authorized snow/aquifer investigation; recorded here as this batch's ledger
entry. Full numbers: `~/workspace/mcir-work/world-compare/REPORT.md`.

### B6-G03 (major) — Underground water deficit (−1.04%)

Same comparison: water −1.04% underground, surface matches. Related:
PORT-NOTES 21-cell aquifer/fluid divergence at chunk (13,−14) (vanilla
water + obsidian where the port has air/fluid) from the earlier deep-cave
comparison. Measured, not root-caused. The aquifer fluid picker was
spot-checked (sea level 63 water / below −54 lava matches the jar); a full
noise-based aquifer audit was not done. Under the separate authorized
snow/aquifer investigation; recorded here as this batch's ledger entry.

### B6-G04 (minor) — PoolPiece NBT omits the vestigial `GD` (genDepth) key

Vanilla's base `StructurePiece` serializes `genDepth` as `GD` — always 0
for jigsaw pieces (trial chambers 245/245, village_plains 210/210 in the
reference world) — alongside the live `ground_level_delta`. The port's
`jigsaw.rs` `PoolPiece` save writes `ground_level_delta` but no `GD`
(`"GD"` appears only in `end_city.rs` and `legacy.rs`). Cosmetic: vanilla's
loader reads only `ground_level_delta` for pool pieces, so saves load fine;
a strict structure-start NBT comparison would flag the missing key.

## Oracle defects (Batch 0 status triage, 47 oracles, 16 pass / 31 fail)

| ID | Cause | Oracles | Drivers blocked |
|----|-------|---------|-----------------|
| B6-O01 (major) | 26.x-only `densityfunction` package missing | BiomeSourceCodec, BiomeSource, StructureDensity, Worldgen (4) | biome_source_codec, biome_source, structure_density, worldgen |
| B6-O02 (major) | `material/rule` package missing | BiomeGeneration, Terrain (2) | biome_generation, terrain |
| B6-O03 (major) | 26.x `Feature` constructor signature | BiomePlacement, FeaturePlacement (2) | biome_placement, feature_placement |
| B6-O04 (major) | Missing symbols (26.x API drift) | BasicFeature, BiomeChunk, CanyonCarver, CaveCarver, ChunkStage, Density, FeatureCatalog, FeatureGraph, FeatureRegion, FeatureSort, MaterialCondition, MaterialSurface, Noise, ScatteredOre, StructureStart, TreeFeature, WorldSpawn, WorldSpawnPacket (18) | 20 drivers (see below) |
| B6-O05 (major) | Other missing packages (client-only / moved) | CushionPlacement (`client.multiplayer`), NaturalSpawn (`gamerules`), Sculk (`gizmos`), SpawnRules (`util.debug`) (4) | cushion_placement, natural_spawn, sculk, spawn_rules |
| B6-O06 (minor) | `BiomeManager.getBiome` signature | BiomeZoom (1) | biome_zoom |

B6-O04's 20 drivers: basic_feature, biome_chunk, canyon_carver,
cave_carver, chunk_stage, density, feature_catalog, feature_graph,
feature_region, feature_sort, biome_source_codec (also O01),
material_surface, noise, scattered_ore, structure_start_nbt, tree_feature,
world_spawn, world_spawn_sync, retrogen_chunk, feature_placement (also O03
— counted under O03). Note `VanillaStructureDensityOracle` is UNREPAIRABLE
against 1.21.7 per the brief (26.x-only `densityfunction` API) — do not
force it; density is verified via bytecode/chunk comparison.

**Passing oracles (16):** Anchor, BiomeTemperature, BlockProperties,
BlockStateCodec, BlockSupport, BlockSurvival, ChunkGeneration, ChunkStatus,
Collision, FinalizeSpawn, GatewayReplacement, Heightmap, Placement,
RetrogenData, SimpleBlock, WorldgenRandom.

**Runnable-but-blocked drivers (12, all oracles pass):**
biome_temperature, chunk_generation, chunk_status, datapack_worldgen,
finalize_spawn, flat_generation, gateway_replacement, heightmap, placement,
retrogen_data, structure_layout, worldgen_random.

**No-oracle drivers (12, also unrun):** biome_weather, chunk_restart_ticks,
end_gateway, flat_place, folder_datapack_metadata, inline_feature_parse,
place_feature_inline, sculk_saved_fields, server_player_sculk, spawn_loot,
structure_blocks, unused_feature_startup.

## Verified passes

### Data (method 4 — exact diffs vs the jar)

- **Worldgen registries: 941/941 entries name-set- and content-identical**
  (exact equality, no float rounding): biomes 65, carvers 4, configured
  features 224, density functions 35, noises 60, noise settings 7, placed
  features 258, processor lists 40, structures 34, structure sets 20,
  template pools 188, world presets 6.
- **Structure NBTs: 1202/1202 byte-identical.**
- **Flat presets 9/9 + world presets 6/6 identical.** The flat UI loads
  presets data-driven from the `visible` tag (all 9 listed) — no hardcoded
  subset; `flatgen.rs` reads layers/biome/features from the preset settings.
- **Multi-noise nether preset: exact vs jar bytecode** — nether_wastes all
  zeros; soul_sand_valley humidity −0.5; crimson_forest temperature 0.4;
  warped_forest humidity 0.5, offset 0.375; basalt_deltas temperature −0.5,
  offset 0.175.
- **Multi-noise overworld expansion (7,593 entries, 54 biomes): verified by
  provenance + structure.** The table is vanilla's own datagen export
  (`datagen/out/reports/biome_parameters/`), copied verbatim by prepare —
  jar-derived, not hand-transcribed. First entries are mushroom_fields
  (continentalness [−1.2,−1.05], matching `OverworldBiomeBuilder.addBiomes`
  order), last entries are the underground biomes (dripstone/lush/deep
  dark), and the paired comparison shows 100% biome agreement (below).
  Full entry-by-entry verification would need running the builder.

### Paired chunk comparison (method 3 — Nov 21 dumps, PRE-FIX state)

Port Test 1 vs vanilla Test World, seed 8257987075890829262, 1,018 paired
chunks / 260,608 columns, both `minecraft:full` (post-decorate,
stage-matched):

- **Biome agreement 100.00%** (260,608/260,608; 7 biomes present in the
  area). Biome lookup is not implicated in any later fix, so this stands.
- Surface delta mean −0.87, median 0; ground delta mean −0.29, median 0,
  frac |d|>5 2.2% in scattered single-chunk blobs (top chunk (3,13):
  233/256 cols |d|>5). The blobs are NOT structure/beardifier-correlated
  (no nearby beardifying start for (3,13)); a column dump showed horizon
  terrain ~12–24 blocks lower with near-identical cave air below, plus a
  floating snow block and spruce leaves in air — read as a mixed-build
  artifact of the Nov 21 Test 1 world (chunks generated across builds), not
  a stage-order defect: code review confirms carvers run inside
  `generate_proto` (terrain stage) and `decorate` is a separate later step.
- Cave-air delta mean +8.40 (pre-carver/beardifier-fix state — superseded).
- These numbers measure a superseded state; **no post-fix paired
  ground/surface comparison exists on the VM** (open item).

### Code-vs-jar spot checks (method 2 — javap + source)

- **Jigsaw ground_level_delta: PASS — PORT-NOTES finding was a misread.**
  Vanilla bytecode (`JigsawPlacement$Placer`): rigid child →
  `parentGD − delta_y` where `delta_y = parentJigsawY − childJigsawY +
  frontStepY`; non-rigid child → element GD (base `getGroundLevelDelta()`
  returns 1, no subclass overrides). The port (`jigsaw.rs`) implements
  exactly this, with Single-element default 1 and the start-piece
  adjustment `moved(bottom − (bbox.minY + GD))`. The PORT-NOTES claim
  ("vanilla NBT has GD=0") read the wrong key: **`GD` is genDepth** (base
  `StructurePiece` field, always 0 for jigsaw pieces), while the live
  `ground_level_delta` in the reference world spreads −29…+8 for trial
  chambers (245 pieces, 9 zeros), mostly 1 for villages, mostly 0 for
  ancient cities — consistent with the propagation rule. No defect.
- **Sculk spreader core: PASS.** All three behaviours match the jar
  bytecode: Default (`charge` if decayDelay>0 else 0), Vein (`charge−1` if
  spread-veins placement succeeds, else `floor(charge×0.5)` on a 1-in-5
  roll), Sculk (growth-spawn-cost 50, no-growth radius 1, decay rates 5/10,
  max cursors 32, merge-never in worldgen). The defect is only the missing
  catalyst/extra-growth tail (B6-G01), not the spreader.
- **Ore feature: PASS (code level).** `ore_feature.rs` is a bit-careful port
  of vanilla `OreFeature`: direction `next_float × π`, spread `size/8`,
  radius `ceil((size/16×2+1)/2)`, ends via sin/cos, y0/y1 `nextInt(3)−2`,
  ocean-floor heightmap gate, sphere chain with `Mth.sin`-table radii,
  sphere culling, BitSet dedup, air-exposure discard. The in-tree test
  proves the sphere cell-visitor matches vanilla's box scan exactly
  (20,000 random spheres). The PORT-NOTES tuff-blob divergence is
  unconfirmed at HEAD (see Open items).
- **Aquifer fluid picker: PASS (spot).** Sea-level water / below −54 lava
  matches the jar. Full noise-based aquifer audit not done (B6-G03's
  investigation owns it).
- **`would_survive` predicate: PASS (spot).** Delegates to
  `world.block_survives` (the survival system repaired in 6e6cb55b).
- **Decode-coverage audit: PASS (dispatch arms present).** All 55
  configured-feature types (incl. `end_island`/`end_platform`/
  `void_start_platform` via the `BasicFeature` fallback; unknown types are
  decode errors, not silent skips; `unsupported_features()` asserted empty
  by an in-tree test), 15 placement modifiers, 9 trunk + 11 foliage + 1
  root placer, 9 tree decorators, rule-test predicates, 5 block-predicate
  types, 3 height providers, 4 value providers, 28 density-function types,
  14 material conditions, 16 structure types, 2 structure placements, 3
  carver types, 5 pool-element types, 2 projections, 7 state providers, 4
  processor types. Caveat: arm presence ≠ correct implementation — B6-G01
  was a silent omission inside a present arm.

### Prior verification referenced (not re-diagnosed)

Terrain density (4ab608ca), cave carver (ecc3a014), deep-cave beardifier
(7fffa299 + 37c59445, math verified vs jar bytecode; post-decorate deep-band
air within ±0.4/col), trees decoration (8a3f87b6, verified in-game by the
owner), water flow (ee7ab396), leaf decay (5d83cfb8), hand-planted survival
(6e6cb55b). The three Nov 21 owner complaints (malformed spruce, terracing,
cave pit + pillars) were verified vanilla-accurate. The world comparison
(15:59 NPT Nov 22) additionally confirms: terrain heightmap identical (0.000
cols differ >5), cave air +0.03/col, structures exact (all 6 types, counts +
contents), tree quantities within ~1% (logs +0.84%, leaves +0.13%).

## Open items (not defects)

1. **Re-measure paired ground/surface/cave deltas at HEAD.** The Nov 21
   dumps predate the carver/beardifier/trees/water fixes.
2. **Re-measure sculk-vein cells, tuff-blob positions, and the 21-cell
   aquifer divergence at HEAD** once B6-G01 is fixed (the RNG stream
   changes when the catalyst calls land). The vein divergence is plausibly
   explained by B6-G01's missing RNG calls; the tuff divergence has no
   identified cause yet (ore code verified faithful; placement runs before
   sculk_patch in feature order, so B6-G01 does not affect it).
3. **Run the 24 unblocked drivers** when the execution block lifts; repair
   oracles for the 30 blocked drivers (B6-O01…O06; O01's StructureDensity
   stays unrepaired by design).
4. **Nether/End: no ground truth exists.** All seven reference worlds are
   overworld-only. Nether preset verified against jar bytecode; End
   features verified only by dispatch-arm presence. Fixture-server
   comparison or fresh vanilla exploration needed.
5. **Tree decoration counts vs the reference world at HEAD** (owner
   verified trees present in-game; counts not measured).
6. **Full entry-by-entry overworld multi-noise verification** would need
   running `OverworldBiomeBuilder` (see above).

## Adjacent findings (not B6 defects)

- Bubble columns 0 vs 2,500 — owned by the pilot as G1.
- Beds block entities 0 vs 50 in the world comparison — a save-format gap,
  not worldgen.
- The PORT-NOTES ground_level_delta item is resolved above (misread, not a
  defect).

## Method notes

- Stage discipline was enforced throughout: terrain-level (`generate_proto`)
  output was never compared directly against post-decorate vanilla regions
  except where valid (biome map, heightmap); the paired comparison used
  `minecraft:full` dumps on both sides.
- The wrong-seed VM world (`terrain2/vanilla/world`, seed
  −8343112210435233944) was never used. All comparisons use the
  vanilla-world-mc Test World, seed 8257987075890829262.
- Workdir was on `main` (the `parity-sweep` checkout was killed by the
  execution-risk stop), so Batch 0's oracle repairs were not in the local
  Java files; oracle status was taken from Batch 0's measured
  `batch-0-oracles.json` (generated 2026-11-22, fixpoint big-compile after
  all Batch 0 repairs).
- `GD` vs `ground_level_delta` is a trap for future readers: the two keys
  coexist in vanilla piece NBT, `GD` is genDepth (0 for jigsaw pieces),
  `ground_level_delta` is the live propagated value.
