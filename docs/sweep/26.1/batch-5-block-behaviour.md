# Batch 5 — Block behaviour (26.1 parity sweep)

Checks + triage only, against VANILLA 26.1's own jar and data (named
server-game jar `~/workspace/mcir-work/26.1.x/versions/26.1/jars/server-game.jar`,
Mojang-mapped; client game jar `.../jars/client.jar`; prepared 26.1 data in
`versions/26.1/.local/horizon-client/26.1/`). Static checks only: javap on the
named jars, Rust source comparison, vanilla/prepared data. No fixes, no cargo
builds, no driver runs.

**Headline:** 26.1 sits between 1.21.7 and 26.2, and the batch shows it. The
two systemic causes behind 11 of 1.21.7's 15 defects — class-name drift and
26.x tags absent in the older version — evaporate for 26.2 but only PARTLY
evaporate for 26.1: the 26.1 port dropped two class aliases 26.2's tree has
(`DirtPathBlock`, `RedStoneWireBlock`), 26.1 has no `SpeleothemBlock` class at
all, and three 26.x tags the port reads do not exist yet in 26.1 data
(`#speleothems`, `#nether_portal_frame`, `#blocks_motion_in_heightmap`) —
plus the mud tag is spelled `convertable_to_mud` in 26.1, not
`convertible_to_mud`. Result: 12 of the 15 1.21.7 defects evaporate, G03 and
G13 reproduce, G15 mutates, and three NEW defects were found — two of them
major (no nether portal can ever be created, B5-G20; snow never accumulates
on land, B5-G21). Three majors in total (G13, G20, G21), five minors.

## Tally

| Family | Checks | Pass | Gap |
|---|---:|---:|---:|
| Random/scheduled ticks | 37 | 33 | 4 (B5-G03, B5-G13, B5-G17, B5-G21) |
| Survival/support | 28 | 26 | 2 (B5-G03, B5-G13) |
| Updates | 16 | 14 | 2 (B5-G03, B5-G13) |
| Placement | 11 | 9 | 2 (B5-G03, B5-G13) |
| Interaction | 20 | 16 | 4 (B5-G15, B5-G18, B5-G19, B5-G20) |
| Breaking/removal | 13 | 13 | 0 |
| Structural (class chains, tags) | 2 | 0 | 2 (class-chain liveness, tag inventory) |
| **Total** | **127** | **111** | **16** |

Behaviour checks only; oracle triage is counted separately below. The
structural gaps are the systemic roots of G03/G13/G19/G20/G21 — the checks
are the census and the inventory themselves, not a re-count of the defects.
G03 and G13 each appear in four families because each is a distinct code path
(tick, survive, updateShape, placement) sharing one root cause.

3 gaps are MAJOR (G13, G20, G21), 5 are MINOR (G03, G15, G17, G18, G19).
No B5-H (needs-harness) items: prepared data landed before this batch, so
every check in scope was decided statically.

Oracle defects: 6 (B5-O01, O05, O06, O07, O08, O09) — 5 minor, 1 major
(O09, FaceSupport, blocks a driver family). B5-O02 is a cascade with no
independent defect (do not mint); B5-O03–O04 evaporate (PASS all classpaths
in the measured Batch 0 table).

## Systemic picture

**1.21.7's two systemic causes HALF-return for 26.1.** Verified exhaustively:

- **Class names mostly match, with three holes.** The prepared export
  (`block-behaviours.json`, 1,168 blocks) carries 291 distinct class names;
  271 are known in `level/classes.rs`. 28 are unmapped; 26 fall back to a
  mapped ancestor WITH working arms (furnaces → `AbstractFurnace`, skulls →
  `Skull`, leaves → `LeavesBlock`, weathering copper → `Door`/`CopperBulb`/
  `CopperChest`/etc., glass → `HalfTransparent`; `AmethystBlock` → `Block`
  only, with no behaviour arms — fine). Two arm-bearing classes fall back to
  plain `Block`:
  - `DirtPathBlock` — the 26.1 jar HAS the class, but the 26.1 tree's
    `classes.rs:208` dropped the alias 26.2's tree carries
    (`Path = "PathBlock"` only, no `"DirtPathBlock"`) → B5-G03.
  - `RedStoneWireBlock` (capital S) — the 26.1 jar's name; the 26.1 tree's
    `classes.rs:224` only knows `RedstoneWireBlock` → the wire arms are dead
    in 26.1. Batch 2's scope; flagged prominently in Cross-references.
  - `SpeleothemBlock` does NOT exist in the 26.1 jar at all
    (`PointedDripstoneBlock` extends `Block` directly; export chain
    `['PointedDripstoneBlock','Block']`) → B5-G13. There is also no
    `FletchingTableBlock` and no `FarmBlock` (both correct evaporations),
    and no sulfur family (`SulfurSpikeBlock`, `PotentSulfur` — 26.2
    additions, absent here).
- **Tags: three absent, one misspelled.** All 26.x tags the port reads exist
  in 26.1's `block-tags.json` (248 tags) EXCEPT:
  - `#speleothems` — absent (verified in `block-tags.json` and in the client
    jar: no `speleothems.json`) → B5-G13. Vanilla 26.1 uses hardcoded
    `Blocks.POINTED_DRIPSTONE` checks.
  - `#nether_portal_frame` — absent → B5-G20. Vanilla 26.1's
    `PortalShape.FRAME` is hardcoded `state.is(Blocks.OBSIDIAN)`.
  - `#blocks_motion_in_heightmap` — absent → B5-G21. Vanilla 26.1's
    `Heightmap.MATERIAL_MOTION_BLOCKING` is a method reference to
    `BlockStateBase.blocksMotion()`.
  - `#convertable_to_mud` (with 'a') is the 26.1 tag; the port reads
    `#convertible_to_mud` (with 'i', the 26.3 spelling) → B5-G19. 26.2's
    tree uses the correct 'a' spelling; the 26.1 port wrongly took 26.3's.
  - Present with verified contents: `grows_crops` [farmland],
    `cannot_support_snow_layer` [ice, packed_ice, barrier],
    `support_override_snow_layer` [honey_block, soul_sand, mud],
    `cannot_support_seagrass` [magma_block], `cannot_support_kelp`
    [magma_block], `support_override_cactus_flower` [cactus, farmland],
    `overrides_mushroom_light_requirement` [mycelium, podzol, crimson/warped
    nylium], `maintains_farmland` [13 entries incl. crops + moving_piston +
    #fence_gates], `prevents_nearby_leaf_decay` [#logs],
    `supports_vegetation`/`supports_cactus`/`supports_sugar_cane`/
    `supports_bamboo`/`supports_chorus_flower`/`supports_chorus_plant`/
    `supports_stem_crops`/`supports_stem_fruit`/`supports_crimson_fungus`/
    `supports_warped_fungus`/`supports_crimson_roots`/`supports_warped_roots`,
    `sniffer_egg_hatch_boost` [moss_block], `supports_nether_wart`
    [soul_sand], `supports_hanging_mangrove_propagule` [mangrove_leaves],
    `supports_mangrove_propagule`. The 1.21.7 tag names
    (`mushroom_grow_block`, `snow_layer_cannot_survive_on`,
    `snow_layer_can_survive_on`) are correctly absent. Also absent (26.3-only):
    `blocks_motion`, `washed_away_by_fluids`, `blocks_lava_fire_spread`,
    `sculk_growth_inhibitors`.
  - All `#`-references in block support fields resolve; `data.rs` maps them
    to `support_tag`/`support_tag2`.

**The remaining defects are one JSON-nesting bug (B5-G15), one off-by-one at
the build ceiling (B5-G17), one inverted comparison (B5-G18), and the three
tag/class systemic holes above.** The port's behaviour logic otherwise matches
vanilla 26.1 method-for-method, including several places where 26.1 differs
from BOTH 1.21.7 and 26.3 and the port follows 26.1 exactly: water (not air)
left by tree growth from a waterlogged sapling (B5-G14, opposite of 1.21.7);
`TurtleEggBlock.canDestroyEgg` with no spawn-protection check (26.3 added
one); sweet-berry bonemeal validity keeping the air-above + build-height
checks 26.2 dropped; tripwire scheduling a tick 10 ONLY while pressed (no
delay-0 re-check when unpressed — a 26.2-only form); `BoneMealItem.useOn`
returning PASS on the client for growCrop success (SUCCESS_SERVER on server)
but SUCCESS both sides for the water-plant path; `DiodeBlock.neighborChanged`
with no guard on the block still being this diode.

**26.1 gamerule note (matters for Batch 1):** there is NO `doFireTick`
gamerule in vanilla 26.1 at all. `GameRules` has 59 rules (counted by javap
of `GameRule<>` fields); the only fire-related ones are `FIRE_DAMAGE` and
`FIRE_SPREAD_RADIUS_AROUND_PLAYER`. The only class in the whole jar mentioning
"doFireTick" is `GameRuleRegistryFix`, a datafix migrating the removed rule.
Fire spread is governed by `fire_spread_radius_around_player`, which the port
implements via `can_spread_fire_around` (`game.rs:1560`). See
Cross-references.

## Defects

### B5-G03 — Dirt path never reverts: `DirtPathBlock` unmapped in `classes.rs` (MINOR, REPRODUCES)

26.1's jar HAS `DirtPathBlock` (no `PathBlock`); the export chain is
`['DirtPathBlock','Block']`. But the 26.1 tree's `classes.rs:208` reads
`Path = "PathBlock"` only — the `"DirtPathBlock"` alias 26.2's tree carries
was dropped. The class resolves to plain `Block`, so every path arm is dead:
survive (`survive.rs:273`), updateShape, scheduled tick (`tick.rs:202`,
unconditional `turn_to_base_block`), placement. Dirt paths never revert to
dirt when the block above becomes solid, and never revert on schedule.
Vanilla 26.1 `DirtPathBlock` behaviour itself is unchanged from 26.2.
Fix note: add `"DirtPathBlock"` to the `Path` alias (the 26.2 form), with the
unit test 26.2's tree carries.

### B5-G13 — Dripstone fully dead: no `SpeleothemBlock` class in 26.1 AND `#speleothems` tag absent (MAJOR, REPRODUCES)

Two independent causes, either alone sufficient:

(a) **Class dispatch dead.** 26.1 has no `SpeleothemBlock` class
(`PointedDripstoneBlock` extends `Block` directly; export chain
`['PointedDripstoneBlock','Block']`). The `Class::Speleothem` dispatch arms
never fire: scheduled tick → `spawn_falling_stalactite` (`tick.rs:248`),
random tick → growth (`tick.rs:434`), survive (`survive.rs:307`),
updateShape (`shapes.rs:728`), placement (`place.rs:1156`).

(b) **Tag predicates dead.** `#speleothems` is ABSENT from 26.1 data
(verified in `block-tags.json` and in the client jar — no
`speleothems.json`), so all tag predicates fail: `speleothem.rs:15`,
`falling.rs:55`, `place.rs:1407`, `shapes.rs:1189`. Vanilla 26.1 uses
hardcoded `Blocks.POINTED_DRIPSTONE` checks instead.

Observable: dripstone never grows, stalactites never fall,
`can_survive` defaults true (floating dripstone never pops), placement tip
direction/thickness never computed. The cauldron-fill half remains Batch 1's
G6.

Fix note (for the fix agent): 26.1 needs the 26.2 pattern inverted — dispatch
on `PointedDripstoneBlock` directly (or add a `PointedDripstone` class
mapping) and replace the `#speleothems` tag reads with a hardcoded
pointed-dripstone check, matching vanilla 26.1's own form. While fixing,
also fix the latent B5-G16: vanilla 26.1 `PointedDripstoneBlock.canGrow`
requires the fluid two blocks above the stalactite start to be WATER **and a
source** (`isSource()`); the port (`speleothem.rs:104-107`) checks only
`fluid.kind() == FluidKind::Water`. Unobservable while growth is dead, so not
minted separately.

### B5-G15 — Bonemeal validity too permissive at the build ceiling: sapling/azalea/mushroom height gates read the wrong JSON level (MINOR, MUTATED)

1.21.7's form (over-strict: refused bonemeal on un-growable saplings) is GONE
— the port comment explicitly says `canGrow` is 26.3-only, and the 1.21.7
over-strict gates are removed. But vanilla 26.1 HAS gates (unlike 1.21.7), and
the port's gates read the wrong JSON level, so they never fire. Same defect
as 26.2's B5-G15, reproduced in 26.1:

- Vanilla 26.1 `SaplingBlock.isValidBonemealTarget` = ServerLevel AND
  `isInsideBuildHeight(pos.above(getMinimumHeight))`; `AzaleaBlock` = same + 2
  and fluid above empty; `MushroomBlock` = feature exists AND
  `AbstractHugeMushroomFeature` AND `HugeMushroomFeatureConfiguration` AND
  `isInsideBuildHeight(pos.above(4 + foliageRadius))`.
  `getMinimumHeight` = the primary tree's `trunkPlacer.getBaseHeight()`.
- The port's `bonemeal_valid` (`bonemeal.rs:331-360`) matches in form, but
  `tree_minimum_height` (`growth.rs:434`) reads
  `json["trunk_placer"]["base_height"]` at the JSON top level, while the
  stored feature JSON (verified in
  `data/minecraft/worldgen/configured_feature/*.json`) nests it under
  `config` — `configured_json` (`worldgen/live.rs:234`) returns the raw whole
  feature JSON `{"type":..., "config": {...}}`. So it always returns 0.
  Vanilla trunk base heights: oak 4, spruce 5, mangrove 2, azalea 4, birch 5,
  jungle 4, acacia 5, cherry 7, dark/pale oak 6. The port accepts bonemeal on
  saplings/azalea at heights vanilla refuses.
- Same nesting bug for mushrooms: the port reads `json["foliage_radius"]`
  top-level (`bonemeal.rs:328`) but it lives under `config` (brown = 3, red
  = 2). The port always uses 2 — red correct by luck, brown's gate is
  pos.y+6 instead of pos.y+7 (one block too permissive).
- `isBonemealSuccess` (sapling/azalea 0.45, mushroom/fungus 0.4) matches
  vanilla exactly. The mushroom radius issue is kept inside B5-G15 because
  the 1.21.7 item covered saplings/azalea/mushrooms together.
- Note: `worldgen/features.json` in the prepared data is EMPTY (`{}`); the
  live features come from `data/minecraft/worldgen/configured_feature/*.json`
  via `configured_json`.

Fix note: read `config.trunk_placer.base_height` and `config.foliage_radius`
(or make `configured_json` return the `config` object — its only other caller
is the mushroom check).

### B5-G17 — Chorus flower never ticks at the build ceiling (MINOR, REPRODUCES from 26.2)

Vanilla 26.1 `ChorusFlowerBlock.randomTick` proceeds when
`above.y <= getMaxY()` (320) — bytecode `if_icmple 28`. The port
(`tick.rs:1424`, `chorus_flower_random_tick`) returns early when
`above_pos.y > max_y() - 1`, i.e. above.y >= 320. A flower at pos.y == 319:
vanilla still ages/branches (horizontal targets at y=319 are in-bounds); the
port returns early and the flower stays forever young. The rest of the chorus
logic matches exactly (age < 5, support/pillar logic,
`height < 2 || height <= rand(5 or 4)`, grow-up with `allNeighborsEmpty` +
empty at +2, branch attempts `rand(4) + pillar?1:0`, events 1033/1034, age-5
death).

Note for the fix agent: the analogous vine up-branch guard
(`pos.y < max_y() - 1` vs vanilla `pos.y < maxY`) is NOT a defect — at y=319
vanilla's up-branch has no observable effect either (no block can exist at
y=320 to attach to). Only the chorus-flower gate is wrong, because its
horizontal branches at y=319 are observable.

### B5-G18 — Bell `isProperHit` height comparison inverted (MINOR, REPRODUCES from 26.2)

Vanilla 26.1 `BellBlock.isProperHit` (bytecode read in full) requires
`click_y > 0.8123999834060669` — hit the UPPER part of the bell — plus a
non-Y face plus attachment/axis rules (floor: facing axis == hit axis;
single/double wall: axes differ; ceiling: always). The port
(`redstone.rs:1433`, `bell_hit`) requires `click_y <= 0.8124f32` — inverted.
Hits at the top ring ring in vanilla but not in the port; hits at the bottom
ring ring in the port but not in vanilla. The attachment/axis arms match;
`bell_ring` (block event 1 + sound volume 2.0 pitch 1.0 + BLOCK_CHANGE)
matches vanilla `attemptToRing` (null direction → facing fallback; the port
passes facing directly — same result). Bell `neighborChanged` (powered toggle
+ ring), placement, survive and updateShape all match.

### B5-G19 — Water bottle on dirt never makes mud: port reads `#convertible_to_mud`, 26.1's tag is `#convertable_to_mud` (MINOR, NEW)

The port reads `#convertible_to_mud` (with 'i') in four places:
`data.rs:459` (tag table), `horizon-client/src/game/potions.rs:710` (water
bottle use), `horizon-client/src/game/remote.rs:8690`,
`horizon-client/src/game/dispenser.rs:2249` (dispenser water bottle). 26.1's
tag is `#convertable_to_mud` (with 'a') — verified three ways:
`BlockTags.CONVERTABLE_TO_MUD` in the jar,
`data/minecraft/tags/block/convertable_to_mud.json` in the client jar
(values dirt, coarse_dirt, rooted_dirt), and vanilla `PotionItem.useOn`
checks it. The port's tag read returns nothing, so the mud conversion never
fires. 26.2's tree uses the correct 'a' spelling; the 26.1 port wrongly took
the 26.3 spelling. Fix note: rename all four reads to `convertable_to_mud`.

### B5-G20 — No nether portal can ever be created: `portal_frame` reads the absent `#nether_portal_frame` tag (MAJOR, NEW)

The port's `portal_frame()` (`portal.rs:40-44`) reads
`#nether_portal_frame`, which is ABSENT from 26.1 data (verified in
`block-tags.json`; no synthesis anywhere in the tree) → always false.
Vanilla 26.1 `PortalShape.FRAME` is hardcoded
`state.is(Blocks.OBSIDIAN)` (lambda$static$0 in `PortalShape.txt`).
Consequence: `find_portal_shape` (`portal.rs:67`) always returns width/height
0, `find_empty_portal_shape` (`portal.rs:131`) always returns None, and
`hooks.rs:588-597` `base_fire_on_place` never creates portal blocks — **no
nether portal can ever be created in the 26.1 port**. 26.2's tree correctly
hardcoded obsidian. Fix note: hardcode obsidian, matching vanilla 26.1 and
the 26.2 tree's form.

### B5-G21 — Snow never accumulates on land, ice never forms except on water: `motion_blocking_top` reads the absent `#blocks_motion_in_heightmap` tag (MAJOR, NEW)

The port's `motion_blocking_top` (`weather.rs:41-59`) reads
`#blocks_motion_in_heightmap`, which is ABSENT from 26.1 data (verified in
`block-tags.json`; no synthesis anywhere in the tree) → only fluids count as
motion-blocking. Vanilla 26.1 `Heightmap.MATERIAL_MOTION_BLOCKING` is a
method reference to `BlockStateBase.blocksMotion()` (bootstrap method 1 in
`Heightmap.txt`); the MOTION_BLOCKING heightmap type uses it. Consequence:
`precipitation_info` computes top = min_y in fluidless columns → snow never
accumulates on land and ice never forms except on water. The same wrong top
also feeds `is_raining_at` (under-glass check, `weather.rs:127`), lightning
strike height (`lightning.rs:199`), and portal-travel spawn height
(`portal_travel.rs:506`). 26.2's tree correctly used per-state
`blocksMotion()`. Fix note: replace the tag read with the per-state
`blocksMotion()` value from the prepared data (26.2's form), or synthesise
the tag at prepare time from it.

## 1.21.7 defect re-checks (all 15, against 26.1's own jar/data)

| ID | 1.21.7 defect | 26.1 verdict |
|---|---|---|
| B5-G01 | Farmland static (FarmBlock unmapped, no trample) | EVAPORATES. `FarmlandBlock` maps; vanilla 26.1 `canSurvive` = `!above.isSolid() \|\| shouldMaintainFarmland` where shouldMaintainFarmland = above in `#maintains_farmland`; port `survive.rs:194-196` matches. Trample (fallOn: nextFloat, 0.5d, MOB_GRIEFING, width²×height > 0.512f, turnToDirt) matches port `block_landing.rs` (identical to 26.2's verified form). |
| B5-G02 | Grass/mycelium never spread | EVAPORATES. Chains map (`SpreadingSnowyBlock`/`SnowyBlock`); `isSnowySetting` = `#snow` tag (verified in `SnowyBlock.txt` and in the tag data). |
| B5-G03 | Dirt path never reverts | REPRODUCES (MINOR) — see Defects. The 26.1 jar has `DirtPathBlock`, but the 26.1 tree dropped the alias 26.2's tree has. |
| B5-G04 | Fungi/roots can't survive on nylium/soul soil | EVAPORATES. Vanilla `mayPlaceOn` = `supportBlocks` tag only; export carries `#supports_crimson_fungus` etc.; port reads `support_tag`. |
| B5-G05 | Stems can't be planted, never fruit | EVAPORATES. Vanilla uses `stemSupportBlocks`/`fruitSupportBlocks` tag fields; export carries them (`#supports_pumpkin_stem`, etc.). |
| B5-G06 | Crops grow ~4x too slow (`grows_crops` absent) | EVAPORATES. Vanilla `CropBlock.getGrowthSpeed` is tag-based (`#grows_crops`, moisture > 0 → 3.0f); port matches. |
| B5-G07 | Mushrooms die in bright light (wrong tag) | EVAPORATES. Vanilla uses `OVERRIDES_MUSHROOM_LIGHT_REQUIREMENT` + rawBrightness < 13 + mayPlaceOn; port matches. |
| B5-G08 | Snow layer survival wrong (absent tags) | EVAPORATES. Vanilla uses `CANNOT_SUPPORT_SNOW_LAYER`/`SUPPORT_OVERRIDE_SNOW_LAYER` + isFaceFull + 8-layer check; port matches. |
| B5-G09 | Seagrass survives on magma | EVAPORATES. Vanilla = face sturdy UP && !in `CANNOT_SUPPORT_SEAGRASS`; port matches. |
| B5-G10 | Kelp attaches to magma | EVAPORATES. Vanilla `canAttachTo` = !in `CANNOT_SUPPORT_KELP`; port matches. |
| B5-G11 | Cactus flowers can't survive on cacti/farmland | EVAPORATES. Vanilla = in `SUPPORT_OVERRIDE_CACTUS_FLOWER` \|\| face-sturdy CENTER; port matches. |
| B5-G12 | Fletching table opens crafting menu | EVAPORATES. No `FletchingTableBlock` in the 26.1 jar; export chain `[Block]`; port has no menu arm for plain Block. |
| B5-G13 | Dripstone never grows, stalactites never fall | REPRODUCES (MAJOR) — see Defects. Two independent causes: no `SpeleothemBlock` class and no `#speleothems` tag. |
| B5-G14 | Waterlogged sapling tree growth leaves water | EVAPORATES — and the 1.21.7 form is reversed in 26.1. Vanilla 26.1 `TreeGrower.growTree` single path uses `getFluidState(pos).createLegacyBlock()` (flags 260) — water IS vanilla 26.1's form (1.21.7 used AIR). Port `growth.rs` `grow_tree` matches; mega path clears 2×2 to AIR flags 260 and restores the passed state on failure; `two_by_two` order [0,−1]×[0,−1] matches vanilla. GROWERS table verified exact for all 10 growers (oak 0.1 fancy_oak/oak_bees_005/fancy_oak_bees_005; spruce 0.5 mega_spruce/mega_pine/spruce; mangrove 0.85 mangrove/tall_mangrove; azalea azalea_tree; birch birch/birch_bees_005; jungle mega_jungle_tree/jungle_tree_no_vine; acacia acacia; cherry cherry/cherry_bees_005; dark_oak mega dark_oak; pale_oak mega pale_oak_bonemeal). |
| B5-G15 | Bonemeal validity stricter than vanilla | MUTATED, still a gap (MINOR) — see Defects. The 1.21.7 over-strict form is gone; the 26.1 form is too permissive at the build ceiling (JSON nesting bug). |

## Cross-referenced gaps (not re-minted)

- **Redstone wire unmapped (Batch 2).** 26.1's jar has `RedStoneWireBlock`
  (capital S); export chain `['RedStoneWireBlock','Block']`; the 26.1 tree's
  `classes.rs:224` only knows `RedstoneWireBlock` (26.2's alias dropped) →
  the wire arms are dead in 26.1. **The 26.2 cross-reference REVERSES for
  26.1**: the Batch 2 G7 toggle half evaporated for 26.2 (aliased, live arm)
  but REPRODUCES for 26.1. Flagged prominently for Batch 2 — do not carry the
  26.2 evaporation forward.
- **PushReaction facts use OLD names (Batch 2, pistons).** 26.1's
  `gameplay-facts.json` push values are ONLY old names: NORMAL 22,869,
  DESTROY 6,849, BLOCK 91, PUSH_ONLY 64 — zero new names. The 26.1 tree's
  `data.rs` maps only new names (PUSH/POPPED/IMMOVEABLE/IGNORE_ENTITY) and
  falls through to PushPull for everything → EVERY block reads as PushPull
  (obsidian/bedrock pushable, torches/flowers don't pop). Compounded by
  `pistons.rs` removing 26.2's explicit unpushable list
  (obsidian/crying_obsidian/respawn_anchor/reinforced_deepslate). Batch 2's
  scope (piston pushing) — not counted here.
- **Fire spread ignores `doFireTick` (Batch 1 G4) — EVAPORATES for 26.1.**
  There is no `doFireTick` gamerule in vanilla 26.1 at all (see Systemic
  picture). Fire spread is governed by `fire_spread_radius_around_player`,
  which the port implements. **Note for the coordinator: Batch 1's G4 needs
  re-examination for 26.1** — its premise does not exist in this version. The
  fire tick itself was verified in full in this batch (delay 30+rand(10)
  scheduled first, canSurvive→remove, rain check `0.2 + age*0.03`, age
  `min(15, age + rand(3)/2)` flags 260, `isValidFireLocation`, age==15 burnout
  removal, `INCREASED_FIRE_BURNOUT` attribute (−50 burnout chances, ignite
  odds halved), checkBurnOut east/west 300, down/up 250, north/south 300 in
  that order, spread loop x/z −1..1, y −1..4, rate 100 + (yy−1)*100 above y=1,
  odds `(ignite + 40 + difficulty*7)/(age+30)`, `rand(rate) <= odds`, spread
  age `min(15, age + rand(5)/4)`; checkBurnOut: `rand(chance) < burnOdds`,
  `rand(age+10) < 5 && !rainingAt` → ignite else remove, TNT → prime).
- **Dripstone cauldron fill (Batch 1 G6) — still Batch 1's.** The port
  implements `cauldron_tick` + `dripstone_transfer_fluid` matching vanilla
  `maybeTransferFluid`, but ownership is Batch 1; recorded here only as a
  cross-reference. Note the dripstone side of the transfer is dead in 26.1
  via B5-G13, which Batch 1 should account for.
- **Potent sulfur — does not exist in 26.1.** No sulfur classes, no sulfur
  tags, no sulfur families in the jar or the data (verified by jar listing).
  Batch 4's scope is moot for 26.1 on this item; the 26.2 cross-reference
  does not carry.
- **`update.rs` block-entity creation change (Batch 4).** The 26.1 tree
  creates the block entity unconditionally after `onPlace`; 26.2 re-checks.
  Batch 4's scope to verify against 26.1's `LevelChunk` bytecode.
- **DiodeBlock.neighborChanged (Batch 2, verified for them).** Vanilla 26.1
  has NO guard on the block still being this diode (starts directly with
  canSurvive); the 26.1 tree's guard removal in `redstone.rs` is CORRECT.
- **TreeFeatureOracle (Batch 6, worldgen).** 8e no-override:68 —
  `markPosForPostProcessing` vs 26.1's `markPosForPostprocessing` (lowercase
  p, javap-verified in ProtoChunk). Worldgen scope, not B5.
- **CushionPlacementOracle (entity decoration).** 14e missing-symbol:12 —
  `net.minecraft.world.entity.decoration.Cushion` does not exist in 26.1
  (verified by jar listing). Entity-decoration scope, not B5.
- **Loot tables behind `drop_resources` are Batch 3's scope.**
- **Tripwire/pressure-plate entity filter (`isIgnoringBlockTriggers`) rides
  on the shared physical-trigger query — Batch 2's scope.**

## Oracle defects (for Batch 0)

All statuses quoted from the MEASURED Batch 0 table
(`docs/sweep/26.1/batch-0-oracles.json`, measured Nov 24, 2026; headline:
199/391 oracles compile on at least one classpath; 192 fail). Oracle sources
read to confirm causes; javap used for `Player.drop`, `ChunkMap` ctors,
`ServerPlayer.onEquippedItemBroken`, `MaterialBaker`, `Entity.getRelativePortalPosition`,
`ClockInstance`, `CubeVoxelShape`, `BlockModelFeatureRenderer`.

### B5-O01 — VanillaCauldronOracle: `Player.drop` override uses a 3-arg signature 26.1 lacks (MINOR)

FAIL all classpaths, 1 error, missing-symbol, line 118 "cannot find symbol".
Same cause as 1.21.7 and 26.2: the oracle's FakePlayer overrides
`drop(ItemStack, boolean, net.minecraft.util.Prediction)` (line 118);
`net.minecraft.util.Prediction` does NOT exist in the 26.1 jar ("Error: class
not found") and 26.1's `Player.drop` is the 2-arg `drop(ItemStack, boolean)`
(`Player.txt` line 1128). Fix pattern same as 1.21.7 B5-O01: drop the
Prediction parameter. Overlap B1. Blocks test_cauldron_parity.py.

### B5-O02 — VanillaPortalOracle: cascade via VanillaBlockUpdateOracle (no independent defect — do NOT mint)

FAIL all classpaths, cascade, "depends on broken: VanillaBlockUpdateOracle".
Portal's own 1.21.7 cause EVAPORATES: `Entity.getRelativePortalPosition`
EXISTS in 26.1 (`Entity.txt:8898`). The cascade root is BlockUpdateOracle,
32e missing-symbol:450 — `net.minecraft.world.clock.ClockInstance` does not
exist in 26.1 (`world/clock/` has ClockManager, ClockNetworkState,
ClockState, ClockTimeMarker only). BlockUpdateOracle is Batch 2's.

### B5-O03 — VanillaPickBlockDataOracle: EVAPORATES (PASS all classpaths)

`DataComponents.BLOCK_ENTITY_DATA` exists in 26.1 as
`DataComponentType<TypedEntityData<BlockEntityType<?>>>`.

### B5-O04 — VanillaReplaceBlockOracle: EVAPORATES (PASS all classpaths)

The oracle uses `ConfiguredFeature.DIRECT_CODEC` (exists in 26.1), not
`Feature.DIRECT_CODEC` (absent in 26.1 — moot).

### B5-O05 — VanillaFallingBlockOracle: `ChunkMap` constructor mismatch (MINOR)

FAIL all classpaths, 1 error, constructor-mismatch, line 30. The oracle's
`NoopChunkMap` calls `super(...)` with 12 args; 26.1's `ChunkMap` ctor takes
13 (adds `TicketStorage` before the int/boolean — `ChunkMap.txt:83`). Same
cause as 26.2. Overlap B1. Blocks test_falling_block_parity.py.

### B5-O06 — VanillaDispenserOracle: `EntityTypes` holder does not exist in 26.1 (MINOR, NEW primary cause)

FAIL all classpaths, 15 errors, missing-package, line 212 "package
EntityTypes does not exist" (Batch 0 group G4). The oracle references
`EntityTypes.PLAYER`/`VILLAGER` at lines 212, 235-236, 560-561, 618-619, 725,
728, 780; `EntityTypes` has zero entries in the 26.1 jar — the constants live
on `EntityType` (e.g. `EntityType.PLAYER`, `EntityType.txt:325`). NEW primary
cause vs 26.2. Latent second cause behind it: the oracle's
`onEquippedItemBroken(ItemStack, EquipmentSlot)` at line 360 vs 26.1's
`ServerPlayer.onEquippedItemBroken(Item, EquipmentSlot)`
(`ServerPlayer.txt:5333`) — same as 26.2's cause. Minor per-batch-local;
note B4 may record major (blocks 6 drivers). Blocks
test_block_removal_parity.py (dispenser side).

### B5-O07 — VanillaSectionMeshOracle: `MaterialBaker` is an interface in 26.1, cannot be extended (MINOR, NEW cause)

FAIL, 7 errors, other, line 84 "no interface expected here". The oracle's
`Materials extends MaterialBaker` + `super(atlas, atlas)`; but 26.1's
`MaterialBaker` is an INTERFACE (`public interface` in the client jar,
`MaterialBaker.txt`) with abstract `get(Material, ModelDebugName)` and
`reportMissingReference(String, ModelDebugName)` plus default `resolveSlot`
— no ctor, cannot extend. NEW cause vs 26.2 (abstract `bake(Material)`) and
1.21.7 (blaze3d). Blocks test_breaking_model_parity.py.

### B5-O08 — VanillaBreakingModelOracle: `BlockModelFeatureRenderer` does not exist in 26.1 (MINOR, NEW id — independent defect)

FAIL, 2 errors, missing-symbol, line 12. Import
`net.minecraft.client.renderer.feature.BlockModelFeatureRenderer` (:12) +
reflective use at :68 (`putPartQuads`); the class does not exist in the 26.1
client jar (the package has `BlockFeatureRenderer` instead; 21 entries, no
BlockModelFeatureRenderer). INDEPENDENT defect in 26.1 — in 26.2 this oracle
was a cascade via SectionMesh (B5-O07). Blocks test_breaking_model_parity.py.

### B5-O09 — VanillaFaceSupportOracle: `CubeVoxelShape(DiscreteVoxelShape)` has protected access in 26.1 (MAJOR)

FAIL, 1 error, other, line 45 — `CubeVoxelShape(DiscreteVoxelShape) has
protected access in CubeVoxelShape` (javap-verified: the ctor is `protected`).
MAJOR: blocks 19 drivers + 5 cascade oracles (BlockCollisionScan,
DynamicSupport, EntityCollision, PistonShape, WorldBorder). Cross-ref note:
the blocked drivers span collision/piston/worldborder batches; the
coordinator may reassign this item; recorded here because FaceSupport is
in-scope for Batch 5's support checks.

**Also passing (measured):** VanillaPickBlockDataOracle,
VanillaReplaceBlockOracle, VanillaBlockSurvivalOracle,
VanillaBlockSupportOracle, VanillaBlockPropertiesOracle,
VanillaBlockStateCodecOracle, VanillaCollisionOracle, VanillaHeightmapOracle
— all PASS all classpaths. (VanillaBlockUpdateOracle FAILS in 26.1 —
Batch 2's; it passed in 26.2.)

**Full failing-oracle list scanned for B5-scope items:** the remaining 192
failures were checked; others cross-referenced (BlockEntityAnim/Render → B4;
RedstoneCatalog → B2; FireStarterSound → G2 LocalPlayer ctor; WetFire,
LavaCauldronItem, BlockingItemDamage/ItemBlocking, FluidFlow/Fluid → B1;
CollisionContext/RegistryCollisionScan/Suffocation → collision batches;
Restitution → B1 landing).

## Passes (111 checks)

Grouped by behaviour class; all colours of one block are one check. File:line
references are `versions/26.1/crates/horizon-world/src/level/` unless the
client is named. Where the 26.1 port code is byte-identical to the 26.2 tree's
verified form, the pass rests on that verification plus the 26.1 vanilla-side
spot checks recorded here; the 26.1 port authors' own diffs show they adapted
code precisely where 26.1 differs from 26.2 (tripwire, sweet berry, bonemeal
client, diode, tags, classes).

### Ticks (33 pass, 4 gap)

- **Farmland** random tick (moisture) and scheduled tick (revert) — see G01.
- **Spreading snowy** tick — see G02. **Dirt path** tick — GAP B5-G03.
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
  cave vines 0.1 DOWN — verified in `block-behaviours.json`), canGrowInto:
  kelp→water, others→air; grow_into_state cycles age, cave vines berries roll
  0.11; bonemeal: nether vines geometric count with 0.826 decay, others 1;
  head→body conversion via updateShape verified against raw bytecode,
  including the `dir == opposite && canSurvive` gating of the
  growth-neighbour check.
- **Chorus flower** — GAP B5-G17 (ceiling edge only; all other logic matches:
  1/5 branch chance, `allNeighborsEmpty`, placeGrownFlower/placeDeadFlower
  verified in bytecode).
- **Sweet berry bush**: age<3, 1/5, raw brightness above ≥9, +1 flags 2,
  BLOCK_CHANGE. **Cocoa**: 1/5, age<2, +1 flags 2; bonemeal valid age<2,
  success always, +1; berry bonemeal valid age<3, success always,
  min(age+1,3). **26.1 note:** vanilla 26.1 sweet-berry bonemeal validity is
  age<3 AND air above AND inside build height; the port keeps all three
  (26.2 dropped the air/height checks — the 26.1 tree's form is the correct
  26.1 form).
- **Turtle egg**: randomTick = `shouldUpdateHatchLevel` (26.1 reads the
  `TURTLE_EGG_HATCH_CHANCE` environment attribute; the day timeline sets 1.0
  at ticks 21062–21905, default 0.002) && onSand (#sand below) → hatch<2 crack
  (0.7, pitch 0.9+rand*0.2, hatch+1 flags 2, BLOCK_CHANGE) else hatch (sound,
  removeBlock, BLOCK_DESTROY, spawn turtles via `hatch_turtles` consumer).
  The port's hardcoded window `(21062..21905) → 1.0, else 0.002`
  (`timeline.rs:39`) matches vanilla 26.1's timeline data exactly — pass with
  note (hardcoded overworld timeline; Nether fixed time 18000 and End 6000 are
  outside the window, so no divergence). `TURTLE_EGG_HATCH_CHANCE` default
  0.002f confirmed in `EnvironmentAttributes` static init; overworld
  dimension type sets no turtle-egg override.
- **Sniffer egg**: onPlace hatchBoost = below in `#sniffer_egg_hatch_boost`
  (moss_block only); boosted → level event 3009; delay (12000 or 24000)/3 +
  nextInt(300) (all three constants verified in bytecode); tick: hatch<2
  crack sound 0.7/pitch 0.9+rand*0.2 + hatch+1 flags 2, else hatch sound +
  destroyBlock(false) + baby sniffer via `hatch_sniffer` consumer.
  Re-scheduling works via onPlace-on-setBlock, matching vanilla's engine
  behaviour (`LevelChunk.setBlockState` calls `BlockState.onPlace` on EVERY
  setBlockState, server-side, unless flags & 512 — verified in bytecode; the
  port's `set_block` does the same unless `UPDATE_SKIP_ON_PLACE`). The port
  hardcodes moss_block instead of the tag — matches shipped 26.1 data, pass
  with note.
- **Frogspawn**: onPlace delay nextInt(3600,12000); tick: !canSurvive →
  destroy no-drop else hatch: destroy + sound + 2–5 tadpoles via
  `game.rs:4659` consumer (offsets 0.2..0.8, y = pos.y−0.5, yaw nextInt(1,361)).
- **Frosted ice**: scheduled `tick` with `NEIGHBORS_TO_MELT` constant; 1/3 or
  fewer-than-4 neighbours; END→block light else maxLocalRawBrightness;
  > 11−age−dampening; slightlyMelt; neighbour melt + reschedule 20–40
  (`frosted_ice_tick`, `tick.rs:758`).
- **Ice melt**: block light > 11−dampening; water_evaporates → removeBlock
  else setBlockAndUpdate(water) + neighborChanged; no game event in 26.1 —
  the port comment notes BLOCK_DESTROY is a 26.3 addition (`hooks.rs:691`).
- **Leaves decay**: randomTick: !persistent && distance==7 → drop + remove;
  tick: updateDistance + setBlock flags 3; updateShape: waterlogged tick +
  schedule tick 1 when `distance != 1 || state.distance != distance`.
  `leaf_distance_at` (`shapes.rs:1079`) reads `prevents_nearby_leaf_decay`
  (= `#logs` in 26.1 data); `leaves_update_distance` (`tick.rs:526`) matches
  vanilla.
- **Scaffolding**: getDistance (below scaffolding → same distance, below
  sturdy → 0, horizontal min(distance+1) break at 1); tick: distance==7 →
  old==7 falls as entity else destroyBlock(true), else setBlock flags 3 if
  changed; isBottom: distance>0 && below not scaffolding.
  `STABILITY_MAX_DISTANCE` constant exists in vanilla 26.1.
- **Budding amethyst**: 1/5 (javap-verified `iconst_5`), random of 6 dirs,
  canClusterGrowAtState = air or water source amount 8, stage advance
  small→medium→large→cluster when facing matches, waterlogged = fluid is
  WATER source, setBlockAndUpdate.
- **Coral**: scanForWater = 6 adjacent neighbours with water fluid
  (`CoralBlock.scanForWater` verified present, WATER check in bytecode);
  tick → dead variant flags 2; plant scan = waterlogged || adjacent; die
  tick 60+rand(40).
- **Dried ghast**: tick: waterlogged → hydration<3 +1 + transition sound +
  BLOCK_CHANGE, ==3 → remove + `spawn_ghastling` event; dry → hydration>0 −1
  + event; randomTick keeps a 5000-tick hydration tick scheduled (level 3 and
  delay 5000 both verified in bytecode).
- **Eyeblossom**: open/close tick in `timeline.rs`; the port's window
  `(12600..23401)` (`timeline.rs:34`) matches vanilla's day-timeline
  keyframes exactly (12600 open, 23401 close); `tryChangingState` (verified
  in `EyeblossomBlock.tick`) + neighbour cascade (7×5×7, delay
  distance×5..×10) + 4 particle draws match.
- **Composter**: addItem: level 0 always adds if chance>0 else roll < chance;
  addLayer clamp 0..7, flags 3, BLOCK_CHANGE, schedule 20 at level 7; tick
  level 7 → 8 + ready sound; extract produce. **Compost chance table verified
  115/115 exact vs vanilla `bootStrap()`** (port `compost_chance`,
  `horizon-items/src/items.rs:612`; extraction paired each float with the
  FOLLOWING item — vanilla calls `add(float, ItemLike)` with the float first;
  the only naming difference is vanilla's Java fields `DRY_SHORT_GRASS`/
  `DRY_TALL_GRASS` vs registry ids `short_dry_grass`/`tall_dry_grass` —
  the port matches the registry id, correct).
- **Fire**: full tick verified — see Cross-references (Batch 1 G4).
- **Vine**: spread_vines rule (gamerule `SPREAD_VINES`, verified in
  bytecode), 1/4 (`nextInt(4)`), random dir of 6, horizontal spread with
  cw/ccw attach + corner growth + 0.05 up-growth, up branch, down branch with
  copyRandomFaces, canSpread 9×3×9 max 5, updateShape/getUpdatedState.
  (The port reads `game_rules().spread_vines`.)
- **Mangrove propagule**: not hanging: 1/7 advanceTree with NO light check —
  vanilla has none either (no brightness call in `randomTick`); hanging:
  cycle age if!=4; bonemeal: hanging → !fullyGrown valid, success always,
  cycle age; else sapling rules.
- **Nether portal piglin spawn**: overworld-only
  `NETHER_PORTAL_SPAWNS_PIGLINS` attribute (verified present in
  `EnvironmentAttributes.txt:69`); `portal_spawn` event consumer exists in
  `game.rs`.
- **Dripstone** — GAP B5-G13 (fully dead; see Defects).
- **Weathering copper**: 60 weathering blocks carry the `WeatheringCopper`
  interface in the export, so the interface fallback in `random_tick_block`
  (`tick.rs:486`) covers the unmapped `WeatheringCopper*` terminal classes
  (with upper-door/right-chest/open-chest exclusions).
- **Precipitation motion-blocking top** — GAP B5-G21 (see Defects).

### Survival/support (26 pass, 2 gap)

Farmland, grass/mycelium, dirt path (GAP B5-G03), fungi, roots, stems, crops,
mushroom, snow layer, seagrass, kelp, cactus flower — see G01–G11. Plus:

- **Cactus** (`survive.rs:85`): no solid/lava adjacent, below cactus or
  `supports_cactus`, above not liquid (all three verified in bytecode).
  **Sugar cane** (`survive.rs:317`): below cane or `supports_sugar_cane` +
  water-adjacent check. **Bamboo**: `supports_bamboo` tag. **Nether wart**:
  `supports_nether_wart` (soul_sand). **Vegetation**: `supports_vegetation`.
  **Chorus flower**: end stone / chorus plant pillar rules (`SUPPORTS_CHORUS`
  verified in bytecode). **Bell**: connected-direction support.
  **Torch/candle**: centre support. **Door**: lower — below face-sturdy UP;
  upper — below is this door block.
- **Grindstone**: vanilla 26.1 `GrindstoneBlock.canSurvive` returns `true`
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
- **Dripstone** — GAP B5-G13.

### Updates (14 pass, 2 gap)

- **Door** (vertical matching-half copy else air; lower+down+!canSurvive →
  air), **double-plant**, **bell** (connected-dir survival + double↔single
  wall transitions), **leaves**, **vine** (dir==Down → super, else
  getUpdatedState, 0 faces → air), **scaffolding**, **chorus**, **growing-plant
  head/body** (verified vs raw bytecode), **coral**, **bed** (dir == neighbour
  direction: same-bed different part → copy occupied, else air;
  `bed_neighbour_direction`), **lantern/face-attached** (survive_or_air on
  connected dir), **farmland** (`shapes.rs:465`), **snowy** (`shapes.rs:721`),
  **path** (GAP B5-G03 — the path updateShape arm is dead via the unmapped
  class), **dripstone thickness** (GAP B5-G13 — `calculateSpeleothemThickness`
  never runs).
- **Tripwire**: horizontal dir → recompute connection prop via
  `tripwire_should_connect_to` (`shapes.rs:759, 1066`), matching vanilla's
  `shouldConnectTo` (hook facing back, or tripwire). Tick/checkPressed/
  updateSource verified separately (see below).

### Placement (9 pass, 2 gap)

- **Door**: y < maxY−1, above replaceable, powered = signal at pos or above,
  open = powered, half lower; **hinge verified against raw bytecode**: RIGHT
  if (doorLeft && !doorRight) OR balance>0; LEFT if (doorRight && !doorLeft)
  OR balance<0; else click test (counter-clockwise + clockwise neighbours,
  click-location 0.5 test, LEFT/RIGHT — all verified in `getHinge`) — port
  `door_hinge` (`place.rs:1434`) matches exactly, including the full-collision
  balance and click-position formula.
- **Double-plant**, **bed** (facing neighbour replaceable → foot;
  `setPlacedBy` places head at facing with same state part=head), **bell**
  (vertical → ceiling/floor + horizontal facing + canSurvive; horizontal →
  double_wall if both sides sturdy else single_wall, facing = face.opposite,
  floor/ceiling fallback), **chorus**, **scaffolding** (placement +
  `ScaffoldingBlockItem.updatePlacementContext` at `place.rs:1677`: distance-7
  refusal, sneaking/inside/face direction logic), **turtle egg** (onPlace:
  sand below → level event 2012 data 15), **farmland** (`place.rs:922`),
  **snowy** (`place.rs:1152`), **path** (GAP B5-G03 — place arm dead via the
  unmapped class), **dripstone** (GAP B5-G13 —
  `is_valid_speleothem_placement`, `survive.rs:547`, never runs).

### Interaction (16 pass, 4 gap)

- **Cake** (can_eat gate; bites<6 +1 else remove + BLOCK_DESTROY),
  **candle cake** (→ plain cake bites 1 + drop candle), **composter** (see
  Ticks), **sweet berry harvest** (`interact.rs:328`: InteractLoot
  `minecraft:harvest/sweet_berry_bush` + pick sound), **cave vines harvest**
  (`interact.rs:307`: CaveVines/CaveVinesPlant arm + pick_berries sound),
  **flower pot** (give potted item, revert to empty pot, BLOCK_CHANGE;
  `FLOWER_POT` verified in bytecode), **dragon egg** (teleport: 1000 attempts
  — `sipush 1000` verified — dx/dz ±16, dy ±8, air + solid below + world
  border + build height, level event 2015 with packed coords, set_block flags
  2, remove).
- **Bell** — GAP B5-G18 (height comparison only; ring/placement/survive
  match).
- **Bed**: foot click → redirect to head if head is this bed else Consume;
  head click → Sleep; `beds_destroy_on_use()` → explosion path, else
  `try_sleep`; bed rule read from dimension_type (the full
  `useWithoutItem` structure verified in bytecode: client → SUCCESS_SERVER,
  foot → redirect via FACING, `BED_RULE` environment attribute read).
  **Door**: canOpenByHand else Pass; cycle open, setBlock flags 10, sound,
  BLOCK_OPEN/CLOSE; setOpen same. **Note block**: cycle note 0–24, playNote
  gated on above-air or mob-head instrument, block event + NOTE_BLOCK_PLAY
  (cycle + setBlock flags 3 + playNote + TUNE_NOTEBLOCK stat verified in
  bytecode). **Jukebox**: has_record → Success; record insert/eject is
  container-side. **Daylight detector**: may_build gate (verified:
  `Player.mayBuild`), cycle inverted, flags 2, BLOCK_CHANGE, signal update.
  **Respawn anchor**: charges==0 → Pass (verified: `CHARGE` field read,
  PASS return) else effect. **Grindstone**: menu provider — container-side;
  block side is canSurvive=true + menu. **Containers**
  (furnace/anvil/barrel/beacon/brewing/cartography/chest/copper
  chest/crafter/crafting/dispenser/dropper/enchanting/ender chest/grindstone/
  hopper/loom/shulker/sign): menu-open arms exist in `interact.rs`; menu
  contents are other batches' scope.
- **Bonemeal growth** — GAP B5-G15 (validity gates only; success rolls and
  growth effects match: sapling/azalea 0.45, mushroom/fungus 0.4, cocoa/berry
  always, pitcher lower-half grow, torchflower age 2 → flower).
  **26.1 note:** vanilla 26.1 `BoneMealItem.useOn` growCrop success returns
  PASS on the client (SUCCESS_SERVER on server); growWaterPlant success
  returns SUCCESS both sides; the port's `bone_meal_succeeds_on_client`
  (water path only) matches.
- **Fletching table** — see G12 (no menu arm, matches vanilla).
- **Mud conversion** — GAP B5-G19 (see Defects).
- **Portal lighting** — GAP B5-G20 (see Defects).

### Breaking/removal (13 pass)

- **Drops** via `drop_resources` (loot-table driven; the tables themselves are
  Batch 3's scope).
- **Turtle egg decreaseEggs** (break sound, eggs>1 → eggs−1 flags 2 +
  BLOCK_DESTROY + level event 2001, else destroyBlock(false)) and
  **playerDestroy** path via `turtle_egg_destroyed_by_player` (`hooks.rs:495`).
- **Turtle egg trample**: stepOn 1/100 (player: `motion.rs:1607-1628` →
  `landing.step_on` when not sneaking; mobs: `mobs/ai.rs:856-863`, 1/100,
  turtle/bat excluded, mobGriefing-gated; armor stands:
  `game/armor_stands.rs:1160`), fallOn non-zombie 1/3 — all through
  `destroy_egg` (`block_landing.rs:304-336`: server-only, turtle/bat
  excluded, player always allowed, other living need mobGriefing, sound 0.7
  pitch 0.9+rand*0.2, set_block flags 2 + BLOCK_DESTROY). Matches vanilla
  `destroyEgg`/`canDestroyEgg` exactly (full bytecode read: turtle/bat
  excluded, living entity → player always / else MOB_GRIEFING, non-living →
  false), including the note that 26.3 added a spawn-protection check 26.1
  lacks.
- **Farmland trample** — see G01.
- **Ice mined** → water over solid/liquid unless silk touch, nether stays air
  (`ice_destroyed_by_player`, `hooks.rs`).
- **Sponge absorb**: BFS depth 6, max 65 nodes (`bipush 6`, `bipush 65`
  verified in `removeWaterBreadthFirstSearch`), count>1 → wet sponge flags 2
  + absorb sound; BucketPickup → pickupBlock, LiquidBlock → air, kelp/seagrass
  family → drop + air, else skip. **Wet sponge** onPlace in a
  water-evaporates dimension → dry sponge flags 3 + level event 2009 + dries
  sound pitch (1.0+rand*0.2)*0.7 (both `WATER_EVAPORATES` and event 2009
  verified in bytecode).
- **Door playerWillDestroy** (`hooks.rs:427-460`): creative (or wrong-tool)
  upper-half break clears the lower half (water→water else air, flags 35,
  level event 2001) — matches vanilla
  `preventCreativeDropFromBottomPart`. Nit: the port also applies it when
  `!correct_tool` in survival, emitting an extra 2001 particles event at the
  lower half; vanilla's lower half pops via updateShape with no event. No
  drop difference. **Double-plant** (creative only — full bytecode read:
  `preventsBlockDrops` → `preventDropFromBottomPart` else `dropResources`),
  **bed** (creative foot break clears head, flags 35 + 2001), **piston head**
  (creative destroys the base without drops), **TNT** (unstable primes unless
  creative) — all match.
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
  schedule 10 when pressed. **26.1 note:** vanilla 26.1 schedules a tick 10
  ONLY while pressed — no delay-0 re-check when unpressed (a 26.2-only form);
  the port's removal of the `else if was` arm in `rails.rs` is CORRECT. The
  entity filter (`isIgnoringBlockTriggers`, i.e. spectators) rides on the
  shared physical-trigger query (`game/physical_trigger.rs`), which is Batch
  2's redstone scope — not re-verified here. onPlace (`hooks.rs:282`: `!same`
  → update_source), entityInside (`hooks.rs:569`: checkPressed when
  unpowered and no tick pending) match.

### Structural (0 pass, 2 gap)

- **Class-chain liveness** — GAP. 291 distinct class names in the export; 271
  known in `classes.rs`; 28 unmapped. 26 fall back to a mapped ancestor WITH
  working arms; 2 arm-bearing classes fall back to plain `Block`:
  `DirtPathBlock` (B5-G03) and `RedStoneWireBlock` (Batch 2 cross-ref). Plus
  `SpeleothemBlock` does not exist in 26.1 at all (B5-G13).
- **Tag inventory** — GAP. Three tags the port reads are absent from 26.1
  data (`#speleothems`, `#nether_portal_frame`,
  `#blocks_motion_in_heightmap`), and one is misspelled by the port
  (`convertible_to_mud` vs 26.1's `convertable_to_mud`). All other 26.x tags
  the port reads exist with verified contents (see Systemic picture).

## Pilot leftovers diagnosed

There is no 26.1 pilot test run, so there are no 26.1 pilot failures to
diagnose; the 1.21.7 pilot items are dispositioned against 26.1 instead
(same convention as the 26.2 report).

- `crops_grow_and_leaves_decay`: no 26.1 pilot failure. The leaf-distance half
  is verified passing (leaf distance reads `prevents_nearby_leaf_decay`, which
  is `#logs` in 26.1 data); the crop half passes too — `crop_growth_speed`
  matches vanilla exactly including the halving rule (B5-G06 evaporates).
- `doors_beds_and_plants_place_both_halves`: no 26.1 pilot failure. Door hinge
  verified against raw bytecode; bed placement verified (foot + head via
  `setPlacedBy`); double-plant placement passes; `supports_vegetation` is
  populated in 26.1 data.
- `snowy_grass_and_chests_pair`: the snowy half passes (B5-G02 evaporates —
  spreading-snowy tick matches vanilla). The chest half is Batch 4's scope.
- `a_viewed_copper_chest_does_not_weather`: copper chest EXISTS in 26.1 (a
  26.x block; Batch 4's scope). Not a 26.1 gap.
- `levers_power_wire_through_interaction`, `redstone_lever_wire_and_lamp`,
  `pistons_pop_fragile_blocks_and_refuse_obsidian`: Batch 2's scope — but
  NOTE: unlike 26.2, where all three 1.21.7 majors (G7/G8/G9) were fixed in
  the tree, 26.1's tree has the redstone-wire class UNMAPPED and the
  PushReaction facts in OLD names only (see Cross-references). Batch 2 must
  not carry the 26.2 verdicts forward for 26.1.
- `torch_and_door_halves_break_with_support`: door `playerWillDestroy`
  verified matching vanilla (`hooks.rs:427-460`); torch support verified via
  the centre-support arm. No 26.1 gap.

## Drivers and oracle status

No drivers were run in this batch (static checks only, per the brief). Oracle
status is quoted from the MEASURED Batch 0 table, not re-measured:

- In-scope oracles failing: VanillaCauldronOracle (B5-O01),
  VanillaFallingBlockOracle (B5-O05), VanillaDispenserOracle (B5-O06),
  VanillaSectionMeshOracle (B5-O07), VanillaBreakingModelOracle (B5-O08,
  independent in 26.1), VanillaFaceSupportOracle (B5-O09, major);
  VanillaPortalOracle blocked by VanillaBlockUpdateOracle (cascade, B5-O02,
  no independent defect).
- In-scope oracles passing: VanillaPickBlockDataOracle,
  VanillaReplaceBlockOracle, VanillaBlockSurvivalOracle,
  VanillaBlockSupportOracle, VanillaBlockPropertiesOracle,
  VanillaBlockStateCodecOracle, VanillaCollisionOracle, VanillaHeightmapOracle.
- The six failing oracles block their parity drivers
  (test_cauldron_parity.py, test_falling_block_parity.py,
  test_block_removal_parity.py's dispenser side, test_breaking_model_parity.py,
  plus FaceSupport's 19 drivers) until repaired; five of the six repairs are
  small (a dropped parameter, a ctor arg, a holder rename + a signature
  change, an interface implementation, a renamed class import). FaceSupport
  (protected ctor) needs a different construction path — Batch 0's call.

## Method notes and limitations

- **Ground truth**: vanilla 26.1's OWN jar/data only. Named server-game jar
  `~/workspace/mcir-work/26.1.x/versions/26.1/jars/server-game.jar`
  (Mojang-mapped); client game jar
  `~/workspace/mcir-work/26.1.x/versions/26.1/jars/client.jar` (for
  `MaterialBaker`, `BlockModelFeatureRenderer`, tag JSONs); NEVER the
  leakage-audit 26.1.x server jars or the bundler `server.jar` (no game
  classes). Prepared data from `versions/26.1/.local/horizon-client/26.1/`
  (`block-behaviours.json`, `gameplay-facts.json`, `block-tags.json`,
  `data/minecraft/worldgen/configured_feature/*.json`,
  `data/minecraft/timeline/day.json`, `assets/minecraft/lang/en_us.json`).
  Rust source read-only from the sweep worktree (branch `parity-sweep-26.1`).
- **javap classes dumped** (JDK 25, `-c -p`, under `evidence/javap/`):
  FarmlandBlock, SpreadingSnowyBlock, SnowyBlock, DirtPathBlock,
  NetherFungusBlock, NetherRootsBlock, StemBlock, CropBlock, MushroomBlock,
  SnowLayerBlock, SeagrassBlock, KelpBlock, CactusFlowerBlock,
  PointedDripstoneBlock, TreeGrower, SaplingBlock, AzaleaBlock,
  BambooStalkBlock, SugarCaneBlock, CactusBlock, NetherWartBlock,
  TurtleEggBlock, FrostedIceBlock, IceBlock, LeavesBlock, ChorusFlowerBlock,
  SweetBerryBushBlock, CocoaBlock, ComposterBlock, ScaffoldingBlock, DoorBlock,
  BellBlock, SpongeBlock, BuddingAmethystBlock, FrogspawnBlock,
  SnifferEggBlock, VineBlock, MangrovePropaguleBlock, GrindstoneBlock,
  FireBlock, GrowingPlantHeadBlock, WetSpongeBlock, CoralBlock,
  CoralPlantBlock, NoteBlock, DriedGhastBlock, BedBlock, TripWireBlock,
  TorchflowerCropBlock, PitcherCropBlock, BeetrootBlock, LevelChunk, Level,
  Player, ChunkMap, BlockEntityType, EntityType, ServerPlayer, Entity,
  Feature, DataComponents, GameRules, EnvironmentAttributes,
  GameRuleRegistryFix, Heightmap, Heightmap$Types, PortalShape, BoneMealItem,
  DiodeBlock, MaterialBaker (client jar), EyeblossomBlock, CakeBlock,
  CandleCakeBlock, DragonEggBlock, FlowerPotBlock, DoublePlantBlock,
  JukeboxBlock, DaylightDetectorBlock, RespawnAnchorBlock, LanternBlock,
  TorchBlock, WallTorchBlock, BigDripleafBlock. FAILs (class not found —
  findings, not errors): SpeleothemBlock, FletchingTableBlock, FarmBlock,
  SulfurSpikeBlock, `net.minecraft.util.Prediction`, `EntityTypes`.
- **Data notes**: `en_us.json` is present in the prepared data (502,008 bytes).
  `sounds.json` is genuinely absent (separate step not run) — noted, not
  treated as a port defect. `worldgen/features.json` is EMPTY (`{}`); live
  features come from `data/minecraft/worldgen/configured_feature/*.json`.
  GameRules: 59 rules (javap count of `GameRule<>` fields); no `doFireTick`.
- **Limits**: static checks only — no driver runs, no cargo builds, no
  in-game observation. Random-number consumption order was checked where the
  bytecode made it visible (e.g. torchflower/beetroot gates roll before the
  light check) but full RNG-stream equivalence was not proven for every tick.
  The tripwire/pressure-plate entity filter (`isIgnoringBlockTriggers`)
  rides on the shared physical-trigger query and is Batch 2's scope.
  Potent sulfur does not exist in 26.1. Loot tables behind `drop_resources`
  are Batch 3's scope.
- **Nits recorded as passes, not defects**: dirt-path place arm (no
  pushEntitiesUp — moot while the class is unmapped); door `!correct_tool`
  extra 2001 event; sniffer-egg hardcoded moss_block (matches shipped data);
  turtle-egg/eyeblossom hardcoded overworld timelines (match vanilla 26.1's
  timeline data); tripwire disarm flags 260 vs vanilla 3 (unobservable);
  bamboo dead `bamboo_height_above` query.
- **Worktree discipline**: the sweep worktree was treated as read-only
  throughout — no git commands run in it, no writes under it. All artifacts
  are under `~/workspace/mcir-work/sweep-26.1-batches/b05/` (report.md,
  slice.json, evidence/).
