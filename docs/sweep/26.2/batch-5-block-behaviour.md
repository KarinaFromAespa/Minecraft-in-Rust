# Batch 5 — Block behaviour (26.2 parity sweep)

Checks + triage only, against VANILLA 26.2's own jar and data (server-game jar
`26.1-adapters/versions/26.2/jars/server-game.jar`, Mojang-mapped; prepared
26.2 data in `versions/26.2/.local/horizon-client/26.2/`). Static checks only:
javap on the named jar, Rust source comparison, vanilla/prepared data. No fixes,
no cargo builds, no driver runs.

**Prepare landed during this run.** At spawn time `block-behaviours.json` and
`gameplay-facts.json` were still pending; horizon-prepare for 26.2 finished
during the run (2048.7 s, EXIT 0, 203 MB), so the chain-liveness half of this
batch was done directly against the prepared export rather than recorded as
B5-H. `sounds.json` and lang files are still absent — the VM lacks the official
launcher's asset index 32.json, so this prepare flow never imports them. That is
expected, not a port defect.

**Headline:** the two systemic causes behind 11 of 1.21.7's 15 defects —
class-name drift and 26.x tags absent in 1.21.7 — both evaporate for 26.2,
because 26.2 IS a 26.x version: its class names and tag set match the port's
26.x-shaped table. 14 of the 15 1.21.7 defects evaporate outright; the 15th
(B5-G15) survives in mutated, opposite-direction form. Three new small defects
were found (B5-G16–G18), all minor. No major defects in this batch.

## Tally

| Family | Checks | Pass | Gap |
|---|---:|---:|---:|
| Random/scheduled ticks | 36 | 34 | 2 (B5-G16, B5-G17) |
| Survival/support | 28 | 28 | 0 |
| Updates | 15 | 15 | 0 |
| Placement | 11 | 11 | 0 |
| Interaction | 18 | 16 | 2 (B5-G15, B5-G18) |
| Breaking/removal | 13 | 13 | 0 |
| Structural (class chains, tags) | 2 | 2 | 0 |
| **Total** | **123** | **119** | **4** |

All 4 gaps are MINOR. No majors. No B5-H (needs-harness) items: prepare
landed, so every check in scope was decided statically.

Oracle defects: 4 (B5-O01, O05, O06, O07), all minor, all with NEW causes
vs 1.21.7. B5-O02–O04 evaporate (PASS all classpaths in the measured Batch 0
table). VanillaBreakingModelOracle is blocked by B5-O07 (cascade, no
independent defect).

## Systemic picture

**1.21.7's two systemic causes are gone for 26.2.** Verified exhaustively:

- **Class names match.** The prepared export (`block-behaviours.json`) carries
  294 distinct class names; 273 are known in `level/classes.rs`. The 26
  unmapped names are all *terminal* classes that fall back to a mapped ancestor
  (furnaces → `AbstractFurnace`, skulls → `Skull`/`WallSkull`, leaves →
  `LeavesBlock`, weathering copper → working ancestors, glass →
  `HalfTransparent`; `AmethystBlock` → `Block` only, with no behaviour arms).
  Zero unmapped non-terminal classes. The jar confirms the 26.x names:
  `FarmlandBlock`, `SpreadingSnowyBlock`, `SnowyBlock`, `DirtPathBlock`,
  `NetherFungusBlock`, `NetherRootsBlock`, `SpeleothemBlock`,
  `SulfurSpikeBlock`, `PointedDripstoneBlock` all exist; there is no
  `FletchingTableBlock` and no `FarmBlock`. `classes.rs` already aliases
  26.2's renames (`Path = "PathBlock" | "DirtPathBlock"`,
  `RedstoneWire = "RedstoneWireBlock" | "RedStoneWireBlock"`, with a unit test).
- **Tags are present.** All 26.x tags the port reads exist in 26.2's
  `block-tags.json` (265 tags): `grows_crops`, `overrides_mushroom_light_requirement`,
  `cannot_support_snow_layer`, `support_override_snow_layer`,
  `cannot_support_seagrass`, `cannot_support_kelp`,
  `support_override_cactus_flower`, `speleothems`, `maintains_farmland`,
  `prevents_nearby_leaf_decay` (= `#logs`), `supports_vegetation`,
  `supports_cactus`, `supports_sugar_cane`, `supports_bamboo`,
  `supports_chorus_flower`/`supports_chorus_plant`, `supports_stem_crops` /
  `supports_stem_fruit` (via the pumpkin/melon stem support tags),
  `supports_crimson_fungus`/`supports_crimson_roots` (via the warped tags +
  nylium/mycelium/soul soil), `sniffer_egg_hatch_boost` (moss_block only).
  The 1.21.7 tag names (`mushroom_grow_block`, `snow_layer_cannot_survive_on`,
  `snow_layer_can_survive_on`) are correctly absent — 26.2 uses the 26.x names
  the port reads. All 8 `#`-references in block support fields resolve; only 8
  blocks carry support fields (2 stems, 2 attached stems, 2 fungi, 2 roots),
  and `data.rs` maps them to `support_tag`/`support_tag2` (lines 980–985).

**What remains is four small logic slips, each a single wrong comparison or a
wrong JSON path** — not dispatch failures: a feature-JSON nesting bug (B5-G15),
a missing `isSource()` check (B5-G16), an off-by-one at the build ceiling
(B5-G17), and an inverted height comparison (B5-G18). The port's behaviour
logic otherwise matches vanilla 26.2 method-for-method, including several
places where 26.2 differs from BOTH 1.21.7 and 26.3 and the port follows 26.2
exactly: water (not air) left by tree growth from a waterlogged sapling
(B5-G14, opposite of 1.21.7); `TurtleEggBlock.canDestroyEgg` with no
spawn-protection check (26.3 added one — `block_landing.rs:318-320` notes it);
tripwire re-check at delay 0 when unpressed (26.3 waits a tick —
`rails.rs` comment); `FarmlandBlock.canSurvive` using `#maintains_farmland`.

**26.2 gamerule note (matters for Batch 1):** there is NO `doFireTick`
gamerule in vanilla 26.2 at all. `GameRules` has 59 rules; the only
fire-related ones are `FIRE_DAMAGE` and `FIRE_SPREAD_RADIUS_AROUND_PLAYER`.
The only class in the whole jar mentioning "doFireTick" is
`GameRuleRegistryFix`, a datafix migrating the removed rule. Fire spread is
governed by `fire_spread_radius_around_player`, which the port implements via
`can_spread_fire_around` (`game.rs:1560`). See Cross-references.

## Defects

### B5-G15 — Bonemeal validity too permissive at the build ceiling: sapling/azalea/mushroom height gates read the wrong JSON level (MINOR, MUTATED)

1.21.7's form (over-strict: refused bonemeal on un-growable saplings) is GONE —
the port comment explicitly says `canGrow` is 26.3-only, and the 1.21.7
over-strict gates are removed. But vanilla 26.2 HAS gates (unlike 1.21.7), and
the port's gates read the wrong JSON level, so they never fire:

- Vanilla 26.2 `SaplingBlock.isValidBonemealTarget` = ServerLevel AND
  `isInsideBuildHeight(pos.above(getMinimumHeight))`; `AzaleaBlock` = same + 2
  and fluid above empty; `MushroomBlock` = feature exists AND
  `AbstractHugeMushroomFeature` AND `HugeMushroomFeatureConfiguration` AND
  `isInsideBuildHeight(pos.above(4 + foliageRadius))`.
- The port's `bonemeal_valid` (`bonemeal.rs:331-360`) matches in form, but
  `tree_minimum_height` (`growth.rs:437`) reads
  `json["trunk_placer"]["base_height"]` at the JSON top level, while the stored
  feature JSON (verified in `.local/horizon-client/26.2/worldgen/features.json`)
  nests it under `config` — `configured_json` (`worldgen/live.rs:234`) returns
  the raw whole feature JSON `{"type":..., "config": {...}}`. So it always
  returns 0. Vanilla trunk base heights: oak 4, spruce 5, mangrove 2, azalea 4,
  birch 5, jungle 4, acacia 5, cherry 7 (dark/pale oak have no primary tree → 0,
  correct by luck). The port accepts bonemeal on saplings/azalea at heights
  vanilla refuses.
- Same nesting bug for mushrooms: the port reads `json["foliage_radius"]`
  top-level (`bonemeal.rs:343`) but it lives under `config` (brown = 3, red
  absent → default 2). The port always uses 2 — red correct by luck, brown's
  gate is pos.y+6 instead of pos.y+7 (one block too permissive).
- `isBonemealSuccess` (sapling/azalea 0.45, mushroom/fungus 0.4) matches
  vanilla exactly. The mushroom radius issue is kept inside B5-G15 because the
  1.21.7 item covered saplings/azalea/mushrooms together.

Fix note: read `config.trunk_placer.base_height` and `config.foliage_radius`
(or make `configured_json` return the `config` object — its only other caller
is the mushroom check).

### B5-G16 — Dripstone grows under flowing water (MINOR, NEW)

Vanilla `PointedDripstoneBlock.canGrow` requires the fluid at pos.above(2) to
be WATER **and a source** (`isSource()`). The port (`speleothem.rs:104-107`)
checks only `fluid.kind() == FluidKind::Water`. Flowing water two blocks above
the stalactite start lets dripstone grow in the port, not in vanilla. The rest
of the speleothem logic matches exactly (growth chance 0.011377778, `canGrow`
via `link`/blockToGrowOn, max length 2 sulfur / 7 dripstone, 50/50 grow-down
vs stalagmite-below, `maybeTransferFluid` probabilities, falling-stalactite
damage `max(1 + pos.y - fall.y, 6)`).

### B5-G17 — Chorus flower never ticks at the build ceiling (MINOR, NEW)

Vanilla `ChorusFlowerBlock.randomTick` proceeds when `above.y <= getMaxY()`
(320) — bytecode `if_icmple 28`. The port (`tick.rs:1425`,
`chorus_flower_random_tick`) returns early when `above_pos.y > max_y() - 1`,
i.e. above.y >= 320. A flower at pos.y == 319: vanilla still ages/branches
(horizontal targets at y=319 are in-bounds); the port returns early and the
flower stays forever young. The rest of the chorus logic matches exactly
(age < 5, support/pillar logic, `height < 2 || height <= rand(5 or 4)`,
grow-up with `allNeighborsEmpty` + empty at +2, branch attempts
`rand(4) + pillar?1:0`, events 1033/1034, age-5 death).

Note for the fix agent: the analogous vine up-branch guard
(`pos.y < max_y() - 1` vs vanilla `pos.y < maxY`) is NOT a defect — at y=319
vanilla's up-branch has no observable effect either (no block can exist at
y=320 to attach to). Only the chorus-flower gate is wrong, because its
horizontal branches at y=319 are observable.

### B5-G18 — Bell `isProperHit` height comparison inverted (MINOR, NEW)

Vanilla `BellBlock.isProperHit` (bytecode read in full) requires
`click_y > 0.8123999834060669` — hit the UPPER part of the bell — plus a
non-Y face plus attachment/axis rules (floor: facing axis == hit axis;
single/double wall: axes differ; ceiling: always). The port (`redstone.rs:1432`,
`bell_hit`) requires `click_y <= 0.8124f32` — inverted. Hits at the top ring
ring in vanilla but not in the port; hits at the bottom ring ring in the port
but not in vanilla. The attachment/axis arms match; `bell_ring` (block event 1
+ sound volume 2.0 pitch 1.0 + BLOCK_CHANGE) matches vanilla `attemptToRing`
(null direction → facing fallback; the port passes facing directly — same
result). Bell `neighborChanged` (powered toggle + ring), placement, survive
and updateShape all match.

## 1.21.7 defect re-checks (all 15, against 26.2's own jar/data)

| ID | 1.21.7 defect | 26.2 verdict |
|---|---|---|
| B5-G01 | Farmland static (FarmBlock unmapped, no trample) | EVAPORATES. `FarmlandBlock` maps; random tick (`tick.rs:1019`), scheduled tick (`tick.rs:197`), survive (`survive.rs:194`, `!solid(above) \|\| maintains_farmland`), updateShape (`shapes.rs:465`), placement (`place.rs:922`) all match vanilla 26.2. Trampling IS implemented (`block_landing.rs:445`: server-only, `nextFloat < distance-0.5`, living entity, player\|\|mobGriefing, width²×height > 0.512, pushEntitiesUp, flags 3, BlockChange). Vanilla 26.2 `canSurvive` uses `#maintains_farmland`, so the latent error evaporates too. |
| B5-G02 | Grass/mycelium never spread | EVAPORATES. Classes map; `spreading_snowy_tick` (`tick.rs:1127`), `snowy_can_stay_alive` (`tick.rs:1116`), Snowy updateShape (`shapes.rs:721`), placement (`place.rs:1152`) match vanilla (`isSnowySetting` = `#snow` tag). Base link = dirt. |
| B5-G03 | Dirt path never reverts | EVAPORATES. `DirtPathBlock` aliased to `Path`. Survive (`survive.rs:273`), updateShape, tick (`tick.rs:202`, unconditional `turn_to_base_block`), `turn_to_base_block` (`tick.rs:501`: pushEntitiesUp + set_block_and_update + block_change) match. Nit (pass with note): the place arm returns default dirt without pushEntitiesUp while vanilla `getStateForPlacement` uses pushEntitiesUp. |
| B5-G04 | Fungi/roots can't survive on nylium/soul soil | EVAPORATES. Vanilla 26.2 `mayPlaceOn` is purely the `supportBlocks` tag (javap); the export carries the tags; the port's `may_place_on` arm (`survive.rs:375`) reads `support_tag`. Fungus bonemeal: valid = below is `requiredBlock` + above inside build height; success 0.4. |
| B5-G05 | Stems can't be planted, never fruit | EVAPORATES. Export carries `stemSupportBlocks`/`fruitSupportBlocks`; `stem_random_tick` matches vanilla exactly (light≥9, growth-speed roll, age<7 +1, fruit branch: random horizontal dir, target air, below in `support_tag2`, set fruit + attached stem facing dir). Vanilla 26.2's fruit check IS tag-based, so the broader fruit ground (`supports_vegetation`) is vanilla 26.2's rule. |
| B5-G06 | Crops grow ~4x too slow (`grows_crops` absent) | EVAPORATES. Vanilla 26.2 `CropBlock.getGrowthSpeed` IS tag-based (`#grows_crops`, moisture via `getValueOrElse(MOISTURE,0) > 0` → 3.0). Port `crop_growth_speed` (`tick.rs:819`) matches exactly including the halving rule. `crop_random_tick` and crop survive (`survive.rs:140`: raw brightness ≥8 + super) match. |
| B5-G07 | Mushrooms die in bright light (wrong tag) | EVAPORATES. Vanilla uses `overrides_mushroom_light_requirement`; port `survive.rs:266` matches; `mayPlaceOn` = isSolidRender. Mushroom randomTick matches (1/25, 9×3×9 max 5, 4 attempts, offsets). |
| B5-G08 | Snow layer survival wrong (absent tags) | EVAPORATES. Vanilla uses `cannot_support_snow_layer`/`support_override_snow_layer`; port `survive.rs:293` matches including the 8-layer snow check; `weather.rs` `snow_survives` matches. Snow melt tick (block light > 11 → drop + remove) matches. |
| B5-G09 | Seagrass survives on magma | EVAPORATES. Vanilla `mayPlaceOn` = face sturdy UP && !in `cannot_support_seagrass`; port `survive.rs:407` matches. |
| B5-G10 | Kelp attaches to magma | EVAPORATES. Vanilla `canAttachTo` = !in `cannot_support_kelp`; port `growing_plant_can_attach_to` (`survive.rs:480`) matches. |
| B5-G11 | Cactus flowers can't survive on cacti/farmland | EVAPORATES. Vanilla `mayPlaceOn` = in `support_override_cactus_flower` \|\| isFaceSturdy(UP, CENTER); port `survive.rs:383` matches. |
| B5-G12 | Fletching table opens crafting menu | EVAPORATES. No `FletchingTableBlock` class in the 26.2 jar; export chain is `[Block]`; the port resolves to plain Block with no menu arm — matches vanilla (no `useWithoutItem` override). |
| B5-G13 | Dripstone never grows, stalactites never fall | EVAPORATES for growth+falling. `#speleothems` exists; `speleothem.rs` matches vanilla (see B5-G16 for the one exception); tick arm `tick.rs:248`; `spawn_falling_stalactite` `falling.rs:51`; `is_valid_speleothem_placement` `survive.rs:547`. Cauldron-fill half remains Batch 1's G6. |
| B5-G14 | Waterlogged sapling tree growth leaves water | EVAPORATES — and the 1.21.7 form is reversed in 26.2. Vanilla 26.2 `TreeGrower.growTree` single path uses `getFluidState(pos).createLegacyBlock()` (flags 260) — water IS vanilla 26.2's form (1.21.7 used AIR). Port `growth.rs` `grow_tree` matches; mega path clears to AIR flags 260 and restores the passed state; `flags::UPDATE_NONE` = 260 (`level/mod.rs:164`). GROWERS table verified exact for all 10 growers (oak 0.1 fancy_oak/oak_bees_005/fancy_oak_bees_005; spruce 0.5 mega_spruce/mega_pine/spruce; mangrove 0.85 mangrove/tall_mangrove; azalea azalea_tree; birch birch/birch_bees_005; jungle mega_jungle_tree/jungle_tree_no_vine; acacia acacia; cherry cherry/cherry_bees_005; dark_oak mega dark_oak; pale_oak mega pale_oak_bonemeal). `two_by_two` order [0,−1]×[0,−1] matches vanilla. |
| B5-G15 | Bonemeal validity stricter than vanilla | MUTATED, still a gap (MINOR) — see Defects. The 1.21.7 over-strict form is gone; the 26.2 form is too permissive at the build ceiling (JSON nesting bug). |

## Cross-referenced gaps (not re-minted)

- **Fire spread ignores `doFireTick` (Batch 1 G4) — EVAPORATES for 26.2.**
  There is no `doFireTick` gamerule in vanilla 26.2 at all (see Systemic
  picture). Fire spread is governed by `fire_spread_radius_around_player`,
  which the port implements. **Note for the coordinator: Batch 1's G4 needs
  re-examination for 26.2** — its premise does not exist in this version. The
  fire tick itself was verified in full in this batch (delay 30+rand(10)
  scheduled first, canSurvive→remove, rain check `0.2 + age*0.03`, age
  `min(15, age + rand(3)/2)` flags 260, `isValidFireLocation`, age==15 burnout
  removal, `INCREASED_FIRE_BURNOUT` attribute (−50 burnout chances, ignite
  odds halved), checkBurnOut east/west 300, down/up 250, north/south 300 in
  that order, spread loop x/z −1..1, y −1..4, rate 100 + (yy−1)*100 above y=1,
  odds `(ignite + 40 + difficulty*7)/(age+30)`, `rand(rate) <= odds`, spread
  age `min(15, age + rand(5)/4)`; checkBurnOut: `rand(chance) < burnOdds`,
  `rand(age+10) < 5 && !rainingAt` → ignite else remove, TNT → prime).
- **Redstone wire dot/cross toggle (Batch 2 G7) — EVAPORATES for the toggle
  half.** `RedStoneWireBlock` is aliased in `classes.rs` and the `wire_use`
  arm is live (`interact.rs:263`, implementation at line 490). The wire
  power-update side remains Batch 2's.
- **Dripstone cauldron fill (Batch 1 G6) — still Batch 1's.** The port
  implements `cauldron_tick` + `dripstone_transfer_fluid` matching vanilla
  `maybeTransferFluid`, but ownership is Batch 1; recorded here only as a
  cross-reference.
- **Potent sulfur (26.2 addition) — Batch 4's scope.** The behaviour is
  block-entity-driven (`PotentSulfurBlockEntity.resetCountdown`;
  `level/mod.rs:717-719` is a no-op default, the real implementation is in the
  client). The block-side shape/state arm (`shapes.rs:820-829`) exists. Not
  counted as a B5 check.

## Oracle defects (for Batch 0)

All statuses quoted from the MEASURED Batch 0 table
(`docs/sweep/26.2/batch-0-oracles.json`, measured Nov 24, 2026; headline:
262/391 oracles compile on at least one classpath; 129 fail on all three;
median 2 errors per failing file). Oracle sources read to confirm causes;
javap used for `Player.drop`, `ChunkMap` ctors,
`ServerPlayer.onEquippedItemBroken`, `MaterialBaker`.

### B5-O01 — VanillaCauldronOracle: `Player.drop` override uses a 3-arg signature 26.2 lacks (MINOR)

FAIL all classpaths, 1 error, missing-symbol, line 118 "cannot find symbol".
Same cause as 1.21.7: the oracle's FakePlayer overrides
`drop(ItemStack, boolean, net.minecraft.util.Prediction)` (line 118);
`net.minecraft.util.Prediction` does NOT exist in the 26.2 jar ("Error: class
not found") and 26.2's `Player.drop` is the 2-arg `drop(ItemStack, boolean)`
(`Player.txt` line 92). Fix pattern same as 1.21.7 B5-O01: drop the Prediction
parameter. Blocks test_cauldron_parity.py.

### B5-O02 — VanillaPortalOracle: EVAPORATES (PASS all classpaths)

### B5-O03 — VanillaPickBlockDataOracle: EVAPORATES (PASS all classpaths)

### B5-O04 — VanillaReplaceBlockOracle: EVAPORATES (PASS all classpaths)

### B5-O05 — VanillaFallingBlockOracle: `ChunkMap` constructor mismatch (MINOR, NEW cause)

FAIL all classpaths, 1 error, constructor-mismatch, line 30 (Batch 0 group G5).
The oracle's `NoopChunkMap` calls `super(...)` with 12 args (10 nulls + 0 +
false); 26.2's `ChunkMap` ctor takes 13 args (adds `TicketStorage` before the
int/boolean). 1.21.7's cause was the debug package — different.
Blocks test_falling_block_parity.py.

### B5-O06 — VanillaDispenserOracle: `onEquippedItemBroken` signature changed from ItemStack to Item (MINOR, NEW cause)

FAIL all classpaths, 1 error, no-override, line 360. The oracle's
`BlockingWearer` overrides `onEquippedItemBroken(ItemStack, EquipmentSlot)`;
26.2's `ServerPlayer.onEquippedItemBroken` takes `(Item, EquipmentSlot)`.
1.21.7's cause was the gamerules package (which exists in 26.2) — different.
Blocks test_block_removal_parity.py (dispenser side effects).

### B5-O07 — VanillaSectionMeshOracle: `MaterialBaker` abstract method not implemented (MINOR, NEW cause)

FAIL, 4 errors, missing-override, line 84. The oracle's `Materials extends
MaterialBaker` overrides `get(Material, ModelDebugName)` (concrete in the base)
but never implements the abstract `bake(Material)`; 26.2's `MaterialBaker`
lives at `net.minecraft.client.resources.model.sprite.MaterialBaker` (client
jar) with `protected abstract Material.Baked bake(Material)` and a 1-arg ctor
`MaterialBaker(TextureAtlasSprite)` (the oracle calls `super(atlas, atlas)`).
1.21.7's cause was blaze3d — different. Blocks test_breaking_model_parity.py.

**VanillaBreakingModelOracle: FAIL, cascade via VanillaSectionMeshOracle
(Batch 0 group G8)** — no independent defect; record as blocked-by B5-O07.

**Also passing (measured):** VanillaBlockUpdateOracle (Batch 2's O1
evaporates for 26.2), VanillaBlockSurvivalOracle, VanillaBlockSupportOracle,
VanillaBlockPropertiesOracle, VanillaBlockStateCodecOracle,
VanillaCollisionOracle, VanillaHeightmapOracle — all PASS all classpaths.

## Passes (119 checks)

Grouped by behaviour class; all colours of one block are one check. File:line
references are `versions/26.2/crates/horizon-world/src/level/` unless the
client is named.

### Ticks (34 pass, 2 gap)

- **Farmland** random tick (moisture) and scheduled tick (revert) — see G01.
- **Spreading snowy** tick — see G02. **Dirt path** tick — see G03.
  **Mushroom spread** — see G07. **Snow melt** — see G08. **Crop** tick — see
  G06. **Stem** tick — see G05.
- **Torchflower crop**: 2/3 random-tick gate (`rand_int(3) != 0`,
  `tick.rs:333`), `getMaxAge()` = 2 (javap: `iconst_2`), bonemeal increase 1,
  age 2 → torchflower (`bonemeal.rs:722-727`). **Beetroot**: same 2/3 gate
  (javap-verified identical bytecode), bonemeal increase
  `next_int_between(2,5)/3` (vanilla: `super.getBonemealAgeIncrease / 3`).
- **Bamboo** stalk/sapling: stage 0, 1/3, light≥9, height<16; `bamboo_grow`
  fully verified vs `growBamboo` (leaves none/small/large logic with
  below/below-2 updates, age `!=1 && !below2 bamboo → 0 else 1`, stage
  `(height<11 || rand≥0.25) && height!=15 → 0 else 1`, grow_set above flags 3;
  the port calls `bamboo_height_above` and discards the result — harmless dead
  query).
- **Sugar cane**: age 15, height 3, flags 260. **Cactus**: full — height loop
  with early return at height==3 && age==15, age==8 flower trigger checks the
  CACTUS's own `canSurvive` at above (verified in bytecode), chance 0.25 if
  height≥3 else 0.1, `nextDouble <= chance`, age==15 grow + reset 0 +
  neighborChanged, age<15 +1.
- **Nether wart**: age<3, 1/10, flags 2. **Sapling advance**: light≥9 above,
  1/7, `advanceTree` (javap-verified) — see G14 for the growers table.
- **Growing-plant heads** (kelp/vines): age<25, `nextDouble < probability`
  from the export (kelp 0.14 UP, twisting vines 0.1 UP, weeping vines 0.1 DOWN,
  cave vines 0.1 DOWN — `data.rs:995, 1027-1030`), canGrowInto: kelp→water,
  others→air; grow_into_state cycles age, cave vines berries roll 0.11;
  bonemeal: nether vines geometric count with 0.826 decay, others 1; head→body
  conversion via updateShape verified against raw bytecode, including the
  `dir == opposite && canSurvive` gating of the growth-neighbour check.
- **Chorus flower** — GAP B5-G17 (ceiling edge only; all other logic matches).
- **Sweet berry bush**: age<3, 1/5, raw brightness above ≥9, +1 flags 2,
  BLOCK_CHANGE. **Cocoa**: 1/5, age<2, +1 flags 2; bonemeal valid age<2,
  success always, +1; berry bonemeal valid age<3, success always,
  min(age+1,3).
- **Turtle egg**: randomTick = `shouldUpdateHatchLevel` (26.2 reads the
  `TURTLE_EGG_HATCH_CHANCE` environment attribute; the day timeline sets 1.0
  at ticks 21062–21905, default 0.002) && onSand (#sand below) → hatch<2 crack
  (0.7, pitch 0.9+rand*0.2, hatch+1 flags 2, BLOCK_CHANGE) else hatch (sound,
  removeBlock, BLOCK_DESTROY, spawn turtles via `hatch_turtles` consumer).
  The port's hardcoded window `(21062..21905) → 1.0, else 0.002`
  (`timeline.rs:39`) matches vanilla 26.2's timeline data exactly — pass with
  note (hardcoded overworld timeline; Nether fixed time 18000 and End 6000 are
  outside the window, so no divergence). `TURTLE_EGG_HATCH_CHANCE` default
  0.002f confirmed in `EnvironmentAttributes` static init; overworld
  dimension type sets no turtle-egg override.
- **Sniffer egg**: onPlace hatchBoost = below in `#sniffer_egg_hatch_boost`
  (moss_block only); boosted → level event 3009; delay (12000 or 24000)/3 +
  nextInt(300); tick: hatch<2 crack sound 0.7/pitch 0.9+rand*0.2 + hatch+1
  flags 2, else hatch sound + destroyBlock(false) + baby sniffer via
  `hatch_sniffer` consumer. Re-scheduling works via onPlace-on-setBlock,
  matching vanilla's engine behaviour (`LevelChunk.setBlockState` calls
  `BlockState.onPlace` on EVERY setBlockState, server-side, unless flags &
  512 — verified in bytecode; the port's `set_block` does the same unless
  `UPDATE_SKIP_ON_PLACE`). The port hardcodes moss_block instead of the tag —
  matches shipped 26.2 data, pass with note.
- **Frogspawn**: onPlace delay nextInt(3600,12000); tick: !canSurvive →
  destroy no-drop else hatch: destroy + sound + 2–5 tadpoles via
  `game.rs:4659` consumer (offsets 0.2..0.8, y = pos.y−0.5, yaw nextInt(1,361)).
- **Frosted ice**: 1/3 or fewer-than-4 neighbours; END→block light else
  maxLocalRawBrightness; > 11−age−dampening; slightlyMelt; neighbour melt +
  reschedule 20–40 (`frosted_ice_tick`, `tick.rs:758`).
- **Ice melt**: block light > 11−dampening; water_evaporates → removeBlock
  else setBlockAndUpdate(water) + neighborChanged; no game event in 26.2 —
  the port comment notes BLOCK_DESTROY is a 26.3 addition (`hooks.rs:691`).
- **Leaves decay**: randomTick: !persistent && distance==7 → drop + remove;
  tick: updateDistance + setBlock flags 3; updateShape: waterlogged tick +
  schedule tick 1 when `distance != 1 || state.distance != distance`.
  `leaf_distance_at` (`shapes.rs:1079`) reads `prevents_nearby_leaf_decay`
  (= `#logs` in 26.2 data); `leaves_update_distance` (`tick.rs:526`) matches
  vanilla.
- **Scaffolding**: getDistance (below scaffolding → same distance, below
  sturdy → 0, horizontal min(distance+1) break at 1); tick: distance==7 →
  old==7 falls as entity else destroyBlock(true), else setBlock flags 3 if
  changed; isBottom: distance>0 && below not scaffolding.
- **Budding amethyst**: 1/5, random of 6 dirs, canClusterGrowAtState = air or
  water source amount 8, stage advance small→medium→large→cluster when facing
  matches, waterlogged = fluid is WATER source, setBlockAndUpdate.
- **Coral**: scanForWater = 6 adjacent neighbours with water fluid (verified
  in bytecode, not a 5×5×5 scan); tick → dead variant flags 2; plant scan =
  waterlogged || adjacent; die tick 60+rand(40).
- **Dried ghast**: tick: waterlogged → hydration<3 +1 + transition sound +
  BLOCK_CHANGE, ==3 → remove + `spawn_ghastling` event; dry → hydration>0 −1
  + event; randomTick keeps a 5000-tick hydration tick scheduled.
- **Eyeblossom**: open/close tick in `timeline.rs`; the port's window
  `(12600..23401)` (`timeline.rs:34`) matches vanilla's day-timeline
  keyframes exactly (12600 open, 23401 close); `tryChangingState` + neighbour
  cascade (7×5×7, delay distance×5..×10) + 4 particle draws match.
- **Composter**: addItem: level 0 always adds if chance>0 else roll < chance;
  addLayer clamp 0..7, flags 3, BLOCK_CHANGE, schedule 20 at level 7; tick
  level 7 → 8 + ready sound; extract produce. **Compost chance table verified
  115/115 exact vs vanilla `bootStrap()`** (port `compost_chance`,
  `horizon-items/src/items.rs:612`; the only naming difference is vanilla's
  Java fields `DRY_SHORT_GRASS`/`DRY_TALL_GRASS` vs registry ids
  `short_dry_grass`/`tall_dry_grass` confirmed in items.json — the port
  matches the registry id, correct).
- **Fire**: full tick verified — see Cross-references (Batch 1 G4).
- **Vine**: spread_vines rule, 1/4, random dir of 6, horizontal spread with
  cw/ccw attach + corner growth + 0.05 up-growth, up branch, down branch with
  copyRandomFaces, canSpread 9×3×9 max 5, updateShape/getUpdatedState.
  (`spread_vines` is a gamerule, not a tag — the port reads
  `game_rules().spread_vines`.)
- **Mangrove propagule**: not hanging: 1/7 advanceTree with NO light check —
  vanilla has none either; hanging: cycle age if!=4; bonemeal: hanging →
  !fullyGrown valid, success always, cycle age; else sapling rules.
- **Nether portal piglin spawn**: overworld-only
  `NETHER_PORTAL_SPAWNS_PIGLINS` attribute; `portal_spawn` event consumer
  exists in `game.rs`.
- **Dripstone** — GAP B5-G16 (flowing-water edge only; all other logic
  matches).
- **Weathering copper**: all weathering blocks carry the `WeatheringCopper`
  interface in the export, so the interface fallback in `random_tick_block`
  (`tick.rs:486`) covers the unmapped `WeatheringCopper*` terminal classes
  (with upper-door/right-chest/open-chest exclusions).

### Survival/support (28 pass)

Farmland, grass/mycelium, dirt path, fungi, roots, stems, crops, mushroom,
snow layer, seagrass, kelp, cactus flower — see G01–G11. Plus:

- **Cactus** (`survive.rs:85`): no solid/lava adjacent, below cactus or
  `supports_cactus`, above not liquid. **Sugar cane** (`survive.rs:317`):
  below cane or `supports_sugar_cane` + water-adjacent check. **Bamboo**:
  `supports_bamboo` tag. **Nether wart**: `supports_nether_wart` (soul_sand).
  **Vegetation**: `supports_vegetation`. **Chorus flower**: end stone / chorus
  plant pillar rules. **Bell**: connected-direction support. **Torch/candle**:
  centre support. **Door**: lower — below face-sturdy UP; upper — below is
  this door block.
- **Grindstone**: vanilla 26.2 `GrindstoneBlock.canSurvive` returns `true`
  unconditionally (bytecode `iconst_1; ireturn`) — so the port's
  `Class::Grindstone => true` (`survive.rs:192`) is CORRECT, not a shadowing
  bug; the `FaceAttachedHorizontalDirectional` arm below it is the parent
  fallback for other blocks.
- **Face-attached**: connected direction sturdy. **Lantern**: hanging → above,
  else below. **Mangrove propagule**: hanging → above in
  `supports_hanging_mangrove_propagule`; else super →
  `#supports_mangrove_propagule`. **Scaffolding**: distance < 7. **Vine**:
  getUpdatedState (up face → acceptable neighbour above DOWN; horizontal →
  canSupportAtFace || above vine has face; canSupportAtFace: Down → false,
  attach check, Up → false, else above vine face). **Growing plants**:
  `growing_plant_can_attach_to` (`survive.rs:480`).

### Updates (15 pass)

- **Door** (vertical matching-half copy else air; lower+down+!canSurvive →
  air), **double-plant**, **bell** (connected-dir survival + double↔single
  wall transitions), **leaves**, **vine** (dir==Down → super, else
  getUpdatedState, 0 faces → air), **scaffolding**, **chorus**, **growing-plant
  head/body** (verified vs raw bytecode), **coral**, **bed** (dir == neighbour
  direction: same-bed different part → copy occupied, else air;
  `bed_neighbour_direction`), **lantern/face-attached** (survive_or_air on
  connected dir), **farmland** (`shapes.rs:465`), **snowy** (`shapes.rs:721`),
  **path**.
- **Tripwire**: horizontal dir → recompute connection prop via
  `tripwire_should_connect_to` (`shapes.rs:759, 1066`), matching vanilla's
  `shouldConnectTo` (hook facing back, or tripwire). Tick/checkPressed/
  updateSource verified separately (see below).

### Placement (11 pass)

- **Door**: y < maxY−1, above replaceable, powered = signal at pos or above,
  open = powered, half lower; **hinge verified against raw bytecode**: RIGHT
  if (doorLeft && !doorRight) OR balance>0; LEFT if (doorRight && !doorLeft)
  OR balance<0; else click test — port `door_hinge` (`place.rs:1434`) matches
  exactly, including the full-collision balance and click-position formula.
- **Double-plant**, **bed** (facing neighbour replaceable → foot;
  `setPlacedBy` places head at facing with same state part=head), **bell**
  (vertical → ceiling/floor + horizontal facing + canSurvive; horizontal →
  double_wall if both sides sturdy else single_wall, facing = face.opposite,
  floor/ceiling fallback), **chorus**, **scaffolding** (placement +
  `ScaffoldingBlockItem.updatePlacementContext` at `place.rs:1677`: distance-7
  refusal, sneaking/inside/face direction logic), **turtle egg** (onPlace:
  sand below → level event 2012 data 15), **farmland** (`place.rs:922`),
  **snowy** (`place.rs:1152`), **path** (see G03 nit), **dripstone**
  (`is_valid_speleothem_placement`, `survive.rs:547`).

### Interaction (16 pass, 2 gap)

- **Cake** (can_eat gate; bites<6 +1 else remove + BLOCK_DESTROY),
  **candle cake** (→ plain cake bites 1 + drop candle), **composter** (see
  Ticks), **sweet berry harvest** (`interact.rs:328`: InteractLoot
  `minecraft:harvest/sweet_berry_bush` + pick sound), **cave vines harvest**
  (`interact.rs:307`: CaveVines/CaveVinesPlant arm + pick_berries sound),
  **flower pot** (give potted item, revert to empty pot, BLOCK_CHANGE),
  **dragon egg** (teleport: 1000 attempts, dx/dz ±16, dy ±8, air + solid below
  + world border + build height, level event 2015 with packed coords,
  set_block flags 2, remove).
- **Bell** — GAP B5-G18 (height comparison only; ring/placement/survive
  match).
- **Bed**: foot click → redirect to head if head is this bed else Consume;
  head click → Sleep; `beds_destroy_on_use()` → explosion path, else
  `try_sleep`; bed rule read from dimension_type. **Door**: canOpenByHand
  else Pass; cycle open, setBlock flags 10, sound, BLOCK_OPEN/CLOSE; setOpen
  same. **Note block**: cycle note 0–24, playNote gated on above-air or
  mob-head instrument, block event + NOTE_BLOCK_PLAY. **Jukebox**: has_record
  → Success; record insert/eject is container-side. **Daylight detector**:
  may_build gate, cycle inverted, flags 2, BLOCK_CHANGE, signal update.
  **Respawn anchor**: charges==0 → Pass else effect. **Grindstone**: menu
  provider — container-side; block side is canSurvive=true + menu.
  **Containers** (furnace/anvil/barrel/beacon/brewing/cartography/chest/copper
  chest/crafter/crafting/dispenser/dropper/enchanting/ender chest/grindstone/
  hopper/loom/shulker/sign): menu-open arms exist in `interact.rs`; menu
  contents are other batches' scope.
- **Bonemeal growth** — GAP B5-G15 (validity gates only; success rolls and
  growth effects match: sapling/azalea 0.45, mushroom/fungus 0.4, cocoa/berry
  always, pitcher lower-half grow, torchflower age 2 → flower).
- **Fletching table** — see G12 (no menu arm, matches vanilla).

### Breaking/removal (13 pass)

- **Drops** via `drop_resources` (loot-table driven; the tables themselves are
  Batch 3's scope).
- **Turtle egg decreaseEggs** (break sound, eggs>1 → eggs−1 flags 2 +
  BLOCK_DESTROY + level event 2001, else destroyBlock(false)) and
  **playerDestroy** path via `turtle_egg_destroyed_by_player` (`hooks.rs:495`).
- **Turtle egg trample**: stepOn 1/100 (player: `motion.rs:1607-1628` →
  `landing.step_on` when not sneaking; mobs: `mobs/ai.rs:856-863`, 1/100,
  turtle/bat excluded, mobGriefing-gated; armor stands:
  `game/armor_stands.rs:1160`), fallOn zombie 1/3 — all through
  `destroy_egg` (`block_landing.rs:304-336`: server-only, turtle/bat
  excluded, player always allowed, other living need mobGriefing, sound 0.7
  pitch 0.9+rand*0.2, set_block flags 2 + BLOCK_DESTROY). Matches vanilla
  `destroyEgg`/`canDestroyEgg` exactly, including the note that 26.3 added a
  spawn-protection check 26.2 lacks.
- **Farmland trample** — see G01.
- **Ice mined** → water over solid/liquid unless silk touch, nether stays air
  (`ice_destroyed_by_player`, `hooks.rs`).
- **Sponge absorb**: BFS depth 6, max 65 nodes, count>1 → wet sponge flags 2
  + absorb sound; BucketPickup → pickupBlock, LiquidBlock → air, kelp/seagrass
  family → drop + air, else skip. **Wet sponge** onPlace in a
  water-evaporates dimension → dry sponge flags 3 + level event 2009 + dries
  sound pitch (1.0+rand*0.2)*0.7.
- **Door playerWillDestroy** (`hooks.rs:427-460`): creative (or wrong-tool)
  upper-half break clears the lower half (water→water else air, flags 35,
  level event 2001) — matches vanilla
  `preventCreativeDropFromBottomPart`. Nit: the port also applies it when
  `!correct_tool` in survival, emitting an extra 2001 particles event at the
  lower half; vanilla's lower half pops via updateShape with no event. No
  drop difference. **Double-plant** (creative only), **bed** (creative foot
  break clears head, flags 35 + 2001), **piston head** (creative destroys the
  base without drops), **TNT** (unstable primes unless creative) — all match.
- **Tripwire playerWillDestroy**: shears disarm (`game.rs:5015-5024`: set
  disarmed=true + SHEAR game event) matches vanilla; the port uses flags 260
  where vanilla uses 3, unobservable since the block is destroyed immediately
  after (the hook recalculation reads the disarmed state either way).
  Removal: `affectNeighborsAfterRemoval` (`!moved` → updateSource with
  powered=true) matches (`hooks.rs:396-399`).
- **Tripwire tick/checkPressed/updateSource** (`rails.rs:499-600`): scan
  South/West up to 41 blocks for hooks facing back (vanilla: `i < 42`),
  `calculateState` args (false, true, i, state) match; checkPressed AABB
  y+1/16 to y+2.5/16 (attached) or y+8/16, toggle + updateSource on change,
  schedule 10 when pressed, delay 0 when unpressed-but-was (vanilla bytecode
  `iconst_0` — the port's comment notes 26.3 waits a tick instead). The
  entity filter (`isIgnoringBlockTriggers`, i.e. spectators) rides on the
  shared physical-trigger query (`game/physical_trigger.rs`), which is Batch
  2's redstone scope — not re-verified here. onPlace (`hooks.rs:282`: `!same`
  → update_source), entityInside (`hooks.rs:569`: checkPressed when
  unpowered and no tick pending) match.

### Structural (2 pass)

- **Class-chain liveness**: 294 distinct class names in the export; 273 known
  in `classes.rs`; 26 unmapped terminal classes, ALL falling back to a mapped
  ancestor (see Systemic picture). Zero unmapped non-terminal classes.
- **Tag inventory**: all 26.x tags the port reads exist in 26.2 data (see
  Systemic picture); all 8 `#`-references in block support fields resolve.

## Pilot leftovers diagnosed

[Coordinator reconstruction — the agent's draft lost this section to a
truncated write; the text below is rebuilt from facts verified in this batch
and in committed batches B1/B2/B4. There is no 26.2 pilot test run, so there
are no 26.2 pilot failures to diagnose; the 1.21.7 pilot items are dispositioned
against 26.2 instead.]

- `crops_grow_and_leaves_decay`: no 26.2 pilot failure. The leaf-distance half
  is verified passing (leaf distance reads `prevents_nearby_leaf_decay`, which
  is `#logs` in 26.2 data); the crop half passes too — `crop_growth_speed`
  matches vanilla exactly including the halving rule (B5-G06 evaporates).
- `doors_beds_and_plants_place_both_halves`: no 26.2 pilot failure. Door hinge
  verified against raw bytecode; bed placement verified (foot + head via
  `setPlacedBy`); double-plant placement passes; `supports_vegetation` is
  populated in 26.2 data.
- `snowy_grass_and_chests_pair`: the snowy half passes (B5-G02 evaporates —
  spreading-snowy tick matches vanilla). The chest half is Batch 4's scope.
- `a_viewed_copper_chest_does_not_weather`: copper chest EXISTS in 26.2 (a
  26.x block; Batch 4's scope — see the Batch 4 report). Not a 26.2 gap.
- `levers_power_wire_through_interaction`, `redstone_lever_wire_and_lamp`,
  `pistons_pop_fragile_blocks_and_refuse_obsidian`: Batch 2's scope — all
  three 1.21.7 majors (G7/G8/G9) are fixed in the 26.2 tree (Batch 2 report).
- `torch_and_door_halves_break_with_support`: door `playerWillDestroy`
  verified matching vanilla (`hooks.rs:427-460`); torch support verified via
  the centre-support arm. No 26.2 gap.
## Drivers and oracle status

No drivers were run in this batch (static checks only, per the brief). Oracle
status is quoted from the MEASURED Batch 0 table, not re-measured:

- In-scope oracles failing: VanillaCauldronOracle (B5-O01),
  VanillaFallingBlockOracle (B5-O05), VanillaDispenserOracle (B5-O06),
  VanillaSectionMeshOracle (B5-O07); VanillaBreakingModelOracle blocked by
  B5-O07 (cascade).
- In-scope oracles passing: VanillaPortalOracle, VanillaPickBlockDataOracle,
  VanillaReplaceBlockOracle, VanillaBlockUpdateOracle, VanillaBlockSurvivalOracle,
  VanillaBlockSupportOracle, VanillaBlockPropertiesOracle,
  VanillaBlockStateCodecOracle, VanillaCollisionOracle, VanillaHeightmapOracle.
- The four failing oracles block their parity drivers
  (test_cauldron_parity.py, test_falling_block_parity.py,
  test_block_removal_parity.py's dispenser side, test_breaking_model_parity.py)
  until repaired; all four repairs are small (a dropped parameter, a ctor arg,
  a signature change, an abstract-method implementation).

## Method notes and limitations

- **Ground truth**: vanilla 26.2's OWN jar/data only. Named server-game jar
  `~/workspace/mcir-work/26.1-adapters/versions/26.2/jars/server-game.jar`
  (Mojang-mapped); client game jar
  `~/workspace/mcir-work/leakage-audit/jars/26.2-client.jar` (for
  `MaterialBaker`); NEVER the bundler `26.2-server.jar`. Datagen reports from
  `~/workspace/mcir-work/26.2/versions/26.2/datagen/reports/` (NOT the 26.3
  dump). Prepared data from `versions/26.2/.local/horizon-client/26.2/`
  (`block-behaviours.json`, `gameplay-facts.json`, `block-tags.json`,
  `facts.json`, `worldgen/features.json`, `data/minecraft/timeline/day.json`,
  `dimension_type/overworld.json`). Rust source read-only from the sweep
  worktree (branch `parity-sweep-26.2`, synced to main `732c800f`).
- **javap classes dumped** (JDK 25, `-c -p`): FarmlandBlock,
  SpreadingSnowyBlock, SnowyBlock, DirtPathBlock, NetherFungusBlock,
  NetherRootsBlock, StemBlock, CropBlock, MushroomBlock, SnowLayerBlock,
  SeagrassBlock, KelpBlock, CactusFlowerBlock, SpeleothemBlock,
  PointedDripstoneBlock, SulfurSpikeBlock, TreeGrower, SaplingBlock,
  AzaleaBlock, BambooStalkBlock, SugarCaneBlock, CactusBlock, NetherWartBlock,
  TurtleEggBlock, FrostedIceBlock, IceBlock, LeavesBlock,
  EnvironmentAttributes, ChorusFlowerBlock, SweetBerryBushBlock, CocoaBlock,
  ComposterBlock, ScaffoldingBlock, DoorBlock, BellBlock, SpongeBlock,
  BuddingAmethystBlock, FrogspawnBlock, SnifferEggBlock, VineBlock,
  MangrovePropaguleBlock, GrindstoneBlock, FireBlock, GrowingPlantHeadBlock,
  WetSpongeBlock, LevelChunk, Level, Player, ChunkMap, GameRules, CoralBlock,
  CoralPlantBlock, NoteBlock, DriedGhastBlock, BedBlock, TripWireBlock,
  TorchflowerCropBlock, PitcherCropBlock, BeetrootBlock. FletchingTableBlock
  and `net.minecraft.util.Prediction`: class not found (both findings, not
  errors).
- **Environment incident**: a service restart wiped `/tmp` mid-run,
  destroying the first set of javap dumps and scratch JSON. All findings were
  preserved in the run record; the dumps needed for the remaining batches were
  re-created afterwards. Nothing in the workspace was lost, and no finding in
  this report rests on a lost dump alone — every defect was re-verified
  against a re-created dump or the prepared data.
- **Limits**: static checks only — no driver runs, no cargo builds, no
  in-game observation. Random-number consumption order was checked where the
  bytecode made it visible (e.g. torchflower/beetroot gates roll before the
  light check) but full RNG-stream equivalence was not proven for every tick.
  The tripwire/pressure-plate entity filter (`isIgnoringBlockTriggers`)
  rides on the shared physical-trigger query and is Batch 2's scope.
  Potent sulfur's block-entity tick is Batch 4's scope. Loot tables behind
  `drop_resources` are Batch 3's scope. `sounds.json`/lang absence is
  expected (no asset index 32.json on this VM), not a defect.
- **Nits recorded as passes, not defects**: dirt-path place arm (no
  pushEntitiesUp); door `!correct_tool` extra 2001 event; sniffer-egg
  hardcoded moss_block (matches shipped data); turtle-egg/eyeblossom
  hardcoded overworld timelines (match vanilla 26.2's timeline data);
  tripwire disarm flags 260 vs vanilla 3 (unobservable); bamboo dead
  `bamboo_height_above` query.

## Coordinator note

Reconciled Nov 24, 2026 against the measured Batch 0 table
(`docs/sweep/26.2/batch-0-oracles.json`) and the 26.2 tree/data:

- **Oracle claims all verify exactly** (class, error count, first-error line):
  Cauldron fail-all 1e missing-symbol :118; FallingBlock fail-all 1e
  constructor-mismatch :30; Dispenser fail-all 1e no-override :360;
  SectionMesh fail-all 4e missing-override :84; BreakingModel cascade on
  SectionMesh. B5-O02–O04 evaporations confirmed (Portal, PickBlockData,
  ReplaceBlock all pass all three classpaths).
- **Cross-batch overlaps — dedup at merge time, do not double-count:**
  B5-O01 is the SAME oracle defect as B1-O02 (VanillaCauldronOracle,
  Prediction cause); B5-O05 is the SAME as B1-O03 (VanillaFallingBlockOracle,
  ChunkMap 13-vs-12); B5-O06 is the SAME as B4-O06 (VanillaDispenserOracle,
  `onEquippedItemBroken` ItemStack→Item) — B4 records it MAJOR (blocks 6
  drivers sweep-wide) and that severity stands in the merged ledger; B5's
  "minor" is per-batch-local only. B5-O07 (SectionMesh `bake(Material)`) is
  the FIRST per-oracle record of that defect (Batch 0 group G1 member) and
  is canonical at merge. Keep the earliest batch's item canonical in each
  case, the later ones as cross-references.
- **Coordinator corrections to the agent draft:** (1) the draft carried a
  literal `...[truncated 10445 chars]` marker at line 561 — the agent's
  claim of having repaired the mid-write cut was wrong. The lost section was
  the Pilot-leftovers section; it is reconstructed above from verified facts
  (no 26.2 pilot run exists; per-item statuses grounded in this batch and in
  committed B1/B2/B4). (2) "`GameRules` has 86 rules" corrected to 59 —
  javap counts 59 `GameRule<>` fields in
  `net.minecraft.world.level.gamerules.GameRules`, matching the inventory
  count. The substantive claim stands verified: no `doFireTick` in 26.2, and
  the only fire-related rules are `FIRE_DAMAGE` and
  `FIRE_SPREAD_RADIUS_AROUND_PLAYER`.
- **Batch 1 G4 note — already handled:** the agent flagged G4 for
  re-examination, but Batch 1 already recorded G4 as NOT APPLICABLE as framed
  for 26.2 (no `doFireTick`; fire spread governed by the radius rule, which
  the port implements). B5's finding agrees with B1's conclusion; no action.
- **Defect spot-checks (all four verified in source + data):** B5-G15 —
  `growth.rs:439` reads `json["trunk_placer"]["base_height"]` at top level
  while `features.json` nests it under `config` (oak base_height 4 confirmed);
  `bonemeal.rs:343` reads `json["foliage_radius"]` top-level while brown
  mushroom's 3 lives under `config`. B5-G16 — `speleothem.rs:104-107` checks
  water kind only, no `isSource()`. B5-G17 — `tick.rs:1431` early-returns
  when `above_pos.y > max_y() - 1` (function starts at :1425 as cited).
  B5-G18 — `redstone.rs:1443` requires `click_y <= 0.8124f32` (function
  starts at :1432 as cited), inverted vs vanilla's `> 0.8124`.
- **Tally arithmetic verified:** 36+28+15+11+18+13+2 = 123 checks;
  34+28+15+11+16+13+2 = 119 pass; gaps 2+2 = 4. No corrections needed.
