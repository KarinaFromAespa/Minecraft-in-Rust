# Batch 5 — Block behaviour (1.21.11 parity sweep)

Checks + triage only, against VANILLA 1.21.11's own jar and data (named
server-game jar `~/workspace/mcir-work/e6-deltas/versions/1.21.11/jars/server-game-named.jar`,
sha1 `7f0d85c12aeb57f14687b7ab2ae2e5d825fbf0d2`, 7,163 classes; named
client jar `.../jars/client-named.jar`, sha1
`29bddf144a2f2907528b25d616770bfea67c4792`, 10,291 classes). Static
checks only: javap on the named jars, Rust source comparison, vanilla
jar/data reads. No fixes, no cargo builds, no driver runs, no git
commands in the worktree.

**Headline:** the 1.21.11 template is, in `level/`, nearly code-identical
to its first child — the 16 files the 1.21.10 report called "deliberate
1.21.10 adaptations" differ from the template almost entirely in
comment renames (`1.21.10` → `1.21.11`), with exactly THREE real code
diffs, all three matching vanilla 1.21.11's own changes (sea pickle
`.equals()`, jukebox pop position, turtle hatch default). Correction
to the 1.21.10 report's §6.1: the class aliases and hardcoded forms
were already in the template — the 1.21.10 agent did not add them.
So the template carries the same absent-tag defects as its child:
**six blocks can never survive anywhere** (sugar cane, cocoa, azalea,
mangrove propagule, nether sprouts, nether wart — B5-G37–G42, all
MAJOR, all REPRODUCE), fungi/roots survive nowhere (B5-G04), dripstone
is half-alive (B5-G13), the composter is dead (B5-G22), snow never
accumulates (B5-G21). But vanilla CHANGED between 1.21.10 and 1.21.11
in B5-relevant ways, and three 1.21.10 verdicts flip: **the fire tick
check PASSES** — vanilla 1.21.11 has NO `doFireTick` rule at all (59
game rules; the fire gate is now `FIRE_SPREAD_RADIUS_AROUND_PLAYER`,
default 128, which the port implements faithfully) — and **one oracle
is FIXED** (`TypedEntityData` gained both `loadInto` overloads).
A second oracle, `VanillaPortalOracle`, has its own symbol resolved
(`Entity.getRelativePortalPosition` arrived) but still fails to
compile via its `VanillaBlockUpdateOracle` dependency (coordinator
correction against the landed Batch 0 table — see §4). **B5-G17 (chorus ceiling) and B5-G18 (bell) NEVER EXISTED
again**, confirmed fresh against 1.21.11's bytecode — both land as
CLOSED items.

## §0 Execution status

Checks + triage only. No file under `versions/1.21.11/` was modified,
nothing was committed, no git commands ran in the worktree, no cargo
builds ran. Everything below is static analysis: `javap -c -p` on the
named 1.21.11 jars (82 dump files covering ~70 classes), a tag
inventory built directly from the client jar's
`data/minecraft/tags/block/*.json` (204 tags), a `Blocks` field-set
comparison against 1.21.10 (1,142 fields, 0 diff), per-class chain
verification for all 50 B5-scope blocks, source comparison against
`versions/1.21.11/crates/`, and the 1.21.11 datagen reports
(`items.json`, 1,505 items) plus the client jar's dimension/timeline
data (`day.json` keyframes).

**Batch 0 prepare has NOT landed for 1.21.11** (observed 2026-11-25:
`versions/1.21.11/.local/` does not exist; no `block-behaviours.json`,
no `block-tags.json`, no `docs/sweep/1.21.11/batch-0-oracles.json`).
Two consequences, stated where they matter: (1) the tag inventory and
class chains were built from the jars directly, not from prepared
data — the census is fresh and complete, but the export-field checks
(`stemSupportBlocks` etc.) rest on the exporter's source plus the
jar's field shapes, not on a prepared export; (2) **oracle triage is
provisional**: there is no measured Batch 0 oracle table for 1.21.11,
so each oracle verdict rests on the oracle source in the 1.21.11 tree
plus a javap symbol check against the 1.21.11 jars (the failing or
resolving symbol, its line, and the jar fact are cited per item).
Batch 0's measured table, when it lands, should confirm these nine;
a measured-table disagreement would supersede this section. The FIXED
oracle verdict (O03) is symbol-level: every call in the relevant
methods resolves against the 1.21.11 jar, but no compile was run.
(O02 was also claimed FIXED on symbol-level evidence; the coordinator
corrected it against the landed table — the oracle still fails via
its BlockUpdate dependency. See §4.)

**Tally: 130 behaviour checks — 104 pass, 26 gap** (1.21.10: 128
checks, 101 pass, 27 gap). Behaviour/non-behaviour split: **130
behaviour + 0 non-behaviour**. Batch 5 is not one of the three
as-tallied exception batches; oracle triage is counted separately
below. The tally moves vs 1.21.10 by +2 checks, +3 passes, −1 gap:
two NEW checks (sea pickle bonemeal, jukebox pop — both PASS, both
driven by the 1.21.10→1.21.11 code diffs) and the fire/`doFireTick`
tick check flips to PASS (vanilla 1.21.11 has no `doFireTick` rule;
the radius gate the port implements IS vanilla 1.21.11's form).

**Items: 37** — 26 behaviour gaps (10 major, 16 minor) + 8 oracle
defects (1 major, 7 minor) + 1 fixed item (B5-O03) + 2
closed items (B5-G17, B5-G18, both NEVER EXISTED corrections). No
NEW behaviour-gap items for 1.21.11: every 1.21.10 gap reproduces,
and the three 1.21.10→1.21.11 code diffs are all correct adaptations.

**1.21.10 re-checks (PRIMARY): 31/31 decided — 27 REPRODUCE, 2 FIXED,
2 NEVER EXISTED.** **1.21.7 re-checks (SECONDARY): 22/22 decided —
10 REPRODUCE (including 2 splits), 10 FIXED, 2 NEVER EXISTED (as
stated — both oracles still fail, on other causes).**

## §1 Scope

Block ticks, interaction, survival/support, updates, breaking/removal,
placement — the general block-behaviour surface: farmland, grass/mycelium
spread, dirt paths, nether fungi/roots, stems/crops, mushrooms, snow
layers, seagrass/kelp, cactus flowers, fletching table, dripstone,
saplings/bonemeal, trees, bells, chorus, portals, pick-block, replace,
plus the composter, ice, tripwire, eyeblossom, beds, dragon eggs and
mining feedback that the 1.21.10 run put in scope, plus the six
survival blocks (sugar cane, cocoa, azalea, mangrove propagule,
nether sprouts, nether wart), plus two 1.21.11-new checks (sea pickle
bonemeal, jukebox pop). Method: javap on the named 1.21.11 jars +
source comparison + vanilla jar/data reads. No cargo builds, no
driver runs.

## §2 Method

Three verification layers, all fresh against 1.21.11 evidence:

1. **Jar identity and census.** The named jars' sha1s match the
   brief (server-game `7f0d85c1…`, client `29bddf14…`). The tag
   inventory reads the client jar's
   `data/minecraft/tags/block/*.json` directly: 204 tags (1.21.10:
   203; the one new tag is `can_glide_through`, out of B5 scope).
   Every absent/present claim in this report is a jar fact, not a
   prepared-data fact. The class-chain census differs from the
   1.21.10 run's method and is stated honestly: a full
   `Blocks.<clinit>` + BootstrapMethods parse was attempted and
   ABANDONED — the bootstrap targets for most blocks are
   `Blocks.lambda$…` helpers whose resolution needs the lambda
   bodies, and the parse undercounted (1,139 pairs vs 1,166 blocks;
   helper-registered blocks like beds and stems are missed).
   Instead the structural check rests on three fresh facts: (a)
   `javap -p` on `Blocks` in BOTH jars gives 1,142 `Block` fields
   with ZERO set difference between 1.21.10 and 1.21.11 — no block
   was added, removed or renamed; (b) all 50 B5-scope class chains
   were verified individually from the 1.21.11 jar's class headers
   (`evidence/block-chains-1.21.11.json`); (c) the port's
   `classes.rs` aliases cover every B5-scope chain head (verified
   by grep). Key class facts (all javap-verified fresh):
   `FarmBlock` exists (no `FarmlandBlock`); `DirtPathBlock` exists
   (no `PathBlock`); `PointedDripstoneBlock` extends `Block`
   directly (no `SpeleothemBlock`); `FletchingTableBlock` is ABSENT
   (fletching table is plain `Block`); `SpreadingSnowyDirtBlock`/
   `SnowyDirtBlock` exist (no `SpreadingSnowyBlock`/`SnowyBlock`);
   `FungusBlock`/`RootsBlock` exist (no `NetherFungusBlock`/
   `NetherRootsBlock`); `RedStoneWireBlock` (capital S);
   `TreeGrower` at `net.minecraft.world.level.block.grower.TreeGrower`;
   NO `EntityTypes`/`BlockEntityTypes` (holder move absent); NO
   `AbstractBedBlock` (`BedBlock` extends
   `HorizontalDirectionalBlock`); `BambooStalkBlock` (no
   `BambooBlock`); `GameRules` MOVED to
   `net.minecraft.world.level.gamerules` (59 rules, NO `doFireTick`
   — the 1.21.10 `RULE_DOFIRETICK` is gone; `FIRE_SPREAD_RADIUS_AROUND_PLAYER`
   default 128 min −1, `MAX_SNOW_ACCUMULATION_HEIGHT` default 1 min
   0 max 8, both verified in `<clinit>`); `LevelEvent` MOVED to
   `net.minecraft.world.level.block.LevelEvent` (82 constants,
   20xx range ends at 2013 `PARTICLES_SMASH_ATTACK`); no
   `net.minecraft.util.Prediction`; no `BlockEntityData` component
   class; `MaterialBaker` absent from BOTH jars;
   `BlockModelFeatureRenderer` absent; `BlockFeatureRenderer`
   exists (client only); `net/minecraft/util/debug` EXISTS
   (`ServerDebugSubscribers` in the server jar); `com.mojang.blaze3d`
   EXISTS in the client jar (234 entries); `TypedEntityData` HAS
   both `loadInto` overloads; `Entity.getRelativePortalPosition
   (Direction$Axis, BlockUtil$FoundRectangle)` EXISTS.
2. **Source comparison.** Every defect-bearing method was dumped
   fresh from the 1.21.11 jars (82 dump files, see §7) and compared
   against the 1.21.11 port source (`versions/1.21.11/crates/`).
   Pass checks rest on full method comparison for every
   defect-bearing and re-check method, plus arm-liveness and
   deciding-constant comparison for the generic ticks. A `diff -rq`
   of `level/` against the 1.21.10 tree shows the same 16 differing
   files the 1.21.10 report named — but the diffs are almost
   entirely comment renames (`1.21.10` → `1.21.11` in comments);
   the ONLY real code diffs in `level/` are the sea pickle
   `.equals()` gate (`bonemeal.rs`), the jukebox pop dy removal
   (`removal.rs`), and the turtle hatch `None => 0.0` arm
   (`timeline.rs`). Outside `level/`, the B5-relevant files
   (`potions.rs`, `composter.rs`, `block_breaking.rs`, `items.rs`,
   `weather.rs`, `hooks.rs`, `redstone.rs`, `growth.rs`,
   `block_landing.rs`, `falling.rs`, `portal.rs`,
   `feature_world.rs`) are byte-identical to 1.21.10's; `remote.rs`
   and `dispenser.rs` differ only out of B5 scope (packet renames,
   jockey signature). No pass in this report rests on cross-version
   identity: every vanilla form was re-read from the 1.21.11 jar.
3. **Fresh data checks.** The tag inventory (204 tags; absent/present
   re-verified per tag), the stem support fields (read from the
   exporter's source plus the jar's field shapes — `StemBlock` has
   `fruit`/`attachedStem` `ResourceKey` fields; `FungusBlock` has
   only `requiredBlock` (a `Block`) + `feature` (a `ResourceKey`);
   `RootsBlock` has NO fields), the `compostable` component census
   (0/1,505 items in the 1.21.11 datagen `items.json`), and the
   timeline keyframes (`day.json`: eyeblossom 12600→true /
   23401→false; turtle hatch 21062→1.0 / 21905→0.002, constant ease)
   were all run against 1.21.11 evidence.

**Counting rule:** one check = one vanilla-method/behaviour comparison
(jar + port source, or jar + vanilla data). One item may sit behind
several checks (G13) and one check may carry several items (the
bonemeal-growth check carries G29 + G32 + G36; the fungi/roots
survival check carries G04 + G41; the sapling-advance tick check
carries G33 + G34; the sapling survival check carries G40).
Oracle/driver-path checks are not in the behaviour tally.

**Note on the 1.21.10 summary's "What the 1.21.11 run needs" section:**
read and applied. Item 1 (provenance): the template's own defects
are this run's subject — the palette reader is B11's (no
`from_palette_nbt` in the 1.21.11 tree, coordinator Step 0), the
supports_* synthesis is ABSENT (no `synthesize_legacy_support_tags`
anywhere in the tree — verified by grep), the motion-tag derivation
is absent (`Assets::load` hard-requires the three motion tags —
B3's scope, cross-referenced). Item 2 (loading stick): out of B5
scope (client pause/load tracker) — flagged as a cross-reference,
not checked here. Item 3 (four NEVER EXISTED corrections): G17/G18
checked fresh against 1.21.11's jar (see §4); B2-G11/G12 and B10-G04
are other batches'. Items 4–5 are other batches'.

## §3 Checks table

| Family | Checks | Pass | Gap |
|---|---:|---:|---:|
| Random/scheduled ticks | 37 | 32 | 5 (B5-G13, B5-G21, B5-G23, B5-G26, B5-G33/G34) |
| Survival/support | 28 | 18 | 10 (B5-G04+G41, B5-G08, B5-G09, B5-G11, B5-G13, B5-G37, B5-G38, B5-G39, B5-G40, B5-G42) |
| Updates | 16 | 15 | 1 (B5-G13) |
| Placement | 11 | 10 | 1 (B5-G13) |
| Interaction | 21 | 16 | 5 (bonemeal-growth B5-G29/G32/G36, B5-G19, B5-G22, B5-G27, B5-G28) |
| Breaking/removal | 15 | 12 | 3 (B5-G24, B5-G25, B5-G35) |
| Structural (class chains, tags) | 2 | 1 | 1 (tag inventory) |
| **Total** | **130** | **104** | **26** |

Behaviour checks only; oracle triage is counted separately below. The
structural tag-inventory gap is the systemic root of G04/G08/G09/G11/
G13/G19/G21/G24/G37–G42 — the check is the inventory itself, not a
re-count of the defects. G13 appears in four families because it is a
distinct code path (tick, survive, updateShape, placement) sharing one
root cause. The bonemeal-growth interaction check carries three items
(G29 + G32 + G36); the sapling-advance tick check carries two (G33,
G34); the fungi/roots survival check carries two (G04 + G41); the
sapling survival check carries G40 (the propagule is a `SaplingBlock`
subclass in vanilla). Ticks is 32/5 (1.21.10: 31/6) — the fire check
PASSES here (see §6.2: vanilla 1.21.11's fire gate is the radius rule
the port implements; there is no `doFireTick` rule to ignore).
Interaction is 16/5 (1.21.10: 15/5) — the new sea-pickle bonemeal
check passes. Breaking/removal is 12/3 (1.21.10: 11/3) — the new
jukebox-pop check passes. Survival, updates, placement and structural
are unchanged from 1.21.10.

26 gaps are 10 MAJOR (G04, G13, G21, G22, G37–G42) and 16 MINOR (G08,
G09, G11, G19, G23–G29, G32–G36). No B5-H (needs-harness) items:
every check in scope was decided statically (Batch 0's missing
prepare forced jar-direct census work, not harness work).

Oracle defects: 8 (B5-O01, O02, O04–O09) — 7 minor, 1 major (O09,
FaceSupport, blocks 21 drivers + 7 oracle callers). All eight are
provisional on Batch 0's measured table (see §0); each cites its
failing symbol and jar fact. One oracle is FIXED vs 1.21.10 (O03 —
see §4); O02's own symbol resolved but it still fails via its
BlockUpdate dependency (coordinator correction against the landed
Batch 0 table — see §4). In-scope oracles passing (provisional, clean of
suspect imports): VanillaBlockSurvivalOracle,
VanillaBlockSupportOracle, VanillaBlockPropertiesOracle,
VanillaBlockStateCodecOracle, VanillaCollisionOracle,
VanillaHeightmapOracle, VanillaPickBlockDataOracle.

## §4 Defects

### Reproduced 1.21.10 defects (all verified fresh against 1.21.11's jar/tree)

### B5-G04 — Nether fungi/roots survive nowhere: the dead-tag arm now fires and shadows the fallback (MAJOR, REPRODUCES)

Vanilla 1.21.11 `FungusBlock.mayPlaceOn`: `#NYLIUM || MYCELIUM ||
SOUL_SOIL || super`; `RootsBlock.mayPlaceOn`: `#NYLIUM || SOUL_SOIL
|| super` (bytecode-verified fresh — identical to 1.21.10). The port's
`NetherFungus | NetherRoots` arm (`survive.rs:399-402`) reads
`data.support_tag` — which is `None` for both classes: the exporter
is reflective and dumps `TagKey` fields per block, but
`FungusBlock` has only `requiredBlock` (a `Block`) + `feature` (a
`ResourceKey`) and `RootsBlock` has NO fields at all, so no tag is
ever exported. The classes ARE mapped (`classes.rs` aliases
`NetherFungus = "NetherFungusBlock" | "FungusBlock"`,
`NetherRoots = "NetherRootsBlock" | "RootsBlock"`), so the arm fires
and returns false — shadowing the `Vegetation` fallback. Fungi and
roots cannot be placed or survive anywhere in 1.21.11. (The 1.21.7
form — classes unmapped, fallback applied — does not apply: the
template maps the classes, so this is the worse 1.21.10 form.)

### B5-G08 — Snow layers survive on nothing they should refuse and refuse nothing they should: port reads absent 26.x tags (MINOR, REPRODUCES)

Vanilla 1.21.11 `SnowLayerBlock.canSurvive`: below in
`#SNOW_LAYER_CANNOT_SURVIVE_ON` → false; in
`#SNOW_LAYER_CAN_SURVIVE_ON` → true; else `isFaceFull(collision,
UP)` or 8-layer snow below (bytecode-verified fresh — identical to
1.21.10). Both tags are PRESENT in 1.21.11 (`snow_layer_cannot_survive_on`:
ice, packed_ice, barrier; `snow_layer_can_survive_on`: honey_block,
soul_sand, mud). The port's `SnowLayer` arm (`survive.rs:312-324`)
reads the absent 26.x tags `#cannot_support_snow_layer` /
`#support_override_snow_layer` instead — both always empty — so snow
layers survive on ice and packed ice (vanilla refuses) and pop off
honey blocks, soul sand and mud (vanilla allows). The else branch
(full-face UP or 8-layer snow below) is correct. Second site of the
same root cause: `snow_survives` in `weather.rs` (used by the
precipitation check) reads the same two absent tags — folded into
this item, not minted separately.

### B5-G09 — Seagrass survives on magma: port reads the absent `#cannot_support_seagrass` tag (MINOR, REPRODUCES)

Vanilla 1.21.11 `SeagrassBlock.mayPlaceOn`: face sturdy UP AND NOT
magma, hardcoded (bytecode-verified fresh — identical to 1.21.10).
The port's `Seagrass` arm (`survive.rs:440-443`) reads the absent
`#cannot_support_seagrass` tag (always empty), so the magma
exclusion never applies — seagrass survives on magma blocks. (Kelp
is FIXED — see §5: the port hardcodes the magma refusal in
`growing_plant_can_attach_to`.)

### B5-G11 — Cactus flowers cannot survive on cacti or farmland (MINOR, REPRODUCES)

Vanilla 1.21.11 `CactusFlowerBlock.mayPlaceOn`: below is CACTUS ||
FARMLAND || `isFaceSturdy(UP, CENTER)` (bytecode-verified fresh —
identical to 1.21.10). The port's `CactusFlower` arm
(`survive.rs:404-408`) checks the absent
`#support_override_cactus_flower` tag OR face-sturdy-center.
Vanilla's own sturdy data says cactus CENTER_UP is false and
farmland CENTER_UP is false (exporter-computed `isFaceSturdy` per
state/direction/type — consistent with the fresh 1.21.11 shape
bytecode: cactus SHAPE is `Block.column(14.0, 0.0, 16.0)` and
farmland SHAPE tops at 15, and `SupportType.CENTER` requires
covering the `column(2.0, 0.0, 10.0)` center shape, which neither
top face does). So the port's arm returns false on both intended
supports: cactus flowers cannot be placed on cacti or farmland, and
flowers grown by cactus random tick pop on the next neighbour
update.

### B5-G13 — Dripstone half-alive: dispatch fixed, `#speleothems` tag still dead (MAJOR, REPRODUCES)

**Dispatch is LIVE:** `PointedDripstoneBlock` is mapped
(`classes.rs:220`), and all dripstone arms dispatch on `Speleothem
| PointedDripstone` — scheduled tick (`tick.rs:257`, up-tip +
!canSurvive → destroy, else `spawn_falling_stalactite`), random tick
(`tick.rs:443`), survive, updateShape (`shapes.rs:761`), placement
(`place.rs:1165`). **The tag predicates are dead:**
`#speleothems` is ABSENT from 1.21.11 data, so `speleothem_tag`
(`speleothem.rs:13-17`), `spawn_falling_stalactite`
(`falling.rs:51`), `speleothem_thickness` (`shapes.rs:1176`),
`speleothem_thickness_for` (`place.rs:1398`) and the growth/fluid
gates all fail. Vanilla 1.21.11 uses hardcoded
`Blocks.POINTED_DRIPSTONE` checks (`PointedDripstoneBlock.randomTick`:
`maybeTransferFluid` + 0.011377778f growth chance via
`isStalactiteStartPos`/`growStalactiteOrStalagmiteIfPossible`;
`canGrow`: DRIPSTONE_BLOCK above-start + WATER block + fluid
`isSource()` — all bytecode-verified fresh, identical to 1.21.10).
Observable: dripstone never grows, stalactites never fall, cauldron
drip never transfers, thickness is always "tip".

### B5-G19 — Water bottle on dirt never makes mud: port reads `#convertible_to_mud`, 1.21.11's tag is `#convertable_to_mud` (MINOR, REPRODUCES)

The port reads `#convertible_to_mud` (with 'i') in four places:
`data.rs:458` (tag table), `horizon-client/src/game/potions.rs:710`
(water bottle use), `horizon-client/src/game/remote.rs:8676`,
`horizon-client/src/game/dispenser.rs:2251` (dispenser water
bottle). 1.21.11's tag is `#convertable_to_mud` (with 'a') —
verified three ways: `BlockTags.CONVERTABLE_TO_MUD` in the jar,
`data/minecraft/tags/block/convertable_to_mud.json` in the client
jar (values dirt, coarse_dirt, rooted_dirt), and vanilla
`PotionItem.useOn` checks it (bytecode offset 57, re-verified
fresh). The port's tag read returns nothing, so the mud conversion
never fires.

### B5-G21 — Snow never accumulates on land, ice never forms except on water: `motion_blocking_top` reads the absent `#blocks_motion_in_heightmap` tag (MAJOR, REPRODUCES)

The port's `motion_blocking_top` (`weather.rs:41-59`) reads
`#blocks_motion_in_heightmap`, which is ABSENT from 1.21.11 data —
only fluids count as motion-blocking. Vanilla 1.21.11
`Heightmap.MATERIAL_MOTION_BLOCKING` is a method reference to
`BlockBehaviour$BlockStateBase.blocksMotion()` (bootstrap method 1
in `Heightmap`, re-verified fresh with `javap -v`), and
`ServerLevel.tickPrecipitation` uses `getHeightmapPos
(MOTION_BLOCKING, pos)`. Same-root-cause second site:
`level/feature_world.rs:350-356` (`OceanFloor`/`MotionBlocking`/
`MotionBlockingNoLeaves` all read the absent
`#blocks_motion_in_heightmap[_no_leaves]` tags) — folded into this
item, not minted separately. Note what is NOT broken: the port's
`precipitation_tick` (`tick.rs:1524-1563`) faithfully models
vanilla 1.21.11's `tickPrecipitation` — freeze below, the
`MAX_SNOW_ACCUMULATION_HEIGHT` cap (`max_height > 0 &&
layers < max_height.min(8)`, `tick.rs:1539-1544`), `pushEntitiesUp`
on increment only — but it all runs on the wrong `top`, so snow
never accumulates on land and ice never forms except on water.

### B5-G22 — Composter completely dead: port reads the 26.3 `minecraft:compostable` component no 1.21.11 item carries (MAJOR, REPRODUCES)

The port reads the `minecraft:compostable` component
(`horizon-items/src/items.rs:1046`, `game/composter.rs:24-25`).
Vanilla 1.21.11 has NO such component: `DataComponents` has no
`COMPOSTABLE`, and 0/1,505 items in the 1.21.11 datagen `items.json`
carry it. Vanilla 1.21.11 uses the static `ComposterBlock.COMPOSTABLES`
table (`Object2FloatMap`, bytecode-verified fresh). Every composter
path returns `None`: no item can be composted by hand, hopper, or
dispenser. The composter TICK itself works (level 7 → 8 + ready
sound, `tick.rs:86` — matches vanilla's 20-tick schedule at level 7),
so the defect is insertion-only. (`compost_chance` exists nowhere
in the tree.)

### B5-G23 — Ice melt fires a BLOCK_DESTROY game event vanilla 1.21.11 never fires (MINOR, REPRODUCES)

Vanilla 1.21.11 `IceBlock.melt` (bytecode read fresh):
water-evaporates → `removeBlock`, else
`setBlockAndUpdate(water)` + `neighborChanged` — ZERO `gameEvent`
references in the whole class. (1.21.11 change: the evaporates
check now reads `EnvironmentAttributes.WATER_EVAPORATES` via the
`EnvironmentAttributeSystem`; the port's `dimension_type()
.water_evaporates` models the same value.) The port's `ice_melt`
(`hooks.rs:701-717`) adds
`self.game_event_state(pos, "minecraft:block_destroy", state)` after
the `setBlockAndUpdate` (line 716). Sculk sensors hear a
block-destroy event for every melting ice block. (The event is a
26.3 addition.)

### B5-G24 — Ice mined over solid ground leaves no water: `ice_destroyed_by_player` reads the absent `#blocks_motion` tag (MINOR, REPRODUCES)

Vanilla 1.21.11 `IceBlock.playerDestroy`: unless the tool prevents
melting (enchantment tag `PREVENTS_ICE_MELTING`) or water
evaporates, if the block below `blocksMotion()` OR is liquid, the
position becomes water (`setBlockAndUpdate(meltsInto())` —
bytecode-verified fresh). The port's `ice_destroyed_by_player`
(`hooks.rs:535-553`) tests `b.in_set(below, &b.tags.blocks_motion)
|| b.liquid(below)` — the tag is absent (always empty), so ice
mined over solid ground leaves air instead of water.

### B5-G25 — Tripwire schedules a 1-tick re-check when unpressed that vanilla 1.21.11 never schedules (MINOR, REPRODUCES)

Vanilla 1.21.11 schedules a tripwire re-check tick 10 ONLY while
pressed: `TripWireBlock`'s single `scheduleTick` is `bipush 10`
gated on the pressed flag (offset 141, fresh javap), and
`TripWireHookBlock` has exactly ONE `scheduleTick` in the whole
class — the 10-tick attached-circuit re-check inside
`calculateState` (offset 279). There is no 1-tick unpress re-check
anywhere in either class. The port's `tripwire_check_pressed`
(`rails.rs:593-597`) schedules 10 when pressed and 1 `else if was`
— a spurious extra tick (and extra re-check work) every time a
tripwire unpresses. (The AABB heights are CORRECT in 1.21.11:
attached 2.5/16, unattached 8/16, matching vanilla's shape boxes.)

### B5-G26 — Eyeblossom neighbour cascade scans in 26.3 section order instead of `betweenClosed` order (MINOR, REPRODUCES)

Vanilla 1.21.11 `EyeblossomBlock.tryChangingState` iterates the
7×5×7 neighbour box (offsets (−3,−2,−3) to (3,2,3)) with
`BlockPos.betweenClosed` — x innermost, then y, then z — and each
matching neighbour draws its delay from
`nextIntBetweenInclusive(dist*5, dist*10)` in that scan order
(bytecode-verified fresh; the open/closed state itself is read from
`EnvironmentAttributes.EYEBLOSSOM_OPEN`, whose day-timeline
keyframes 12600→true / 23401→false the port matches exactly — see
§7). The port's `eyeblossom_tick` (`timeline.rs:71-115`) uses
`box_scan_order` (section-major: x-section, z-section, y-section,
then y, z, x within a section — y and z swapped vs vanilla), so
the delay draws land on different neighbours in a different order.
The delay form itself matches.

### B5-G27 — Bed use requires the other half: a head click with no foot returns Consume instead of sleeping (MINOR, REPRODUCES)

Vanilla 1.21.11 `BedBlock.useWithoutItem` (bytecode read fresh):
client returns `SUCCESS_SERVER`; if part != HEAD, redirect to the
head via facing and require only that the head is this bed (else
CONSUME); a HEAD click needs no foot check at all — it proceeds
straight to the sleep/explode path (offset 57), which now reads
`EnvironmentAttributes.BED_RULE`. The port's
`Class::AbstractBed | Class::Bed` arm (`interact.rs:365-386`)
requires the other half for BOTH parts (`b.is_block(other, block)
&& part != part`, else `Consume`) — a head click on a bed whose
foot was broken returns Consume instead of sleeping. (The arm fires
— `Bed = "BedBlock"` is mapped, and 1.21.11 has no
`AbstractBedBlock`; the `AbstractBed` alternative is dead but the
`Bed` arm is live.)

### B5-G28 — Dragon egg teleport ignores the world border (MINOR, REPRODUCES)

Vanilla 1.21.11 `DragonEggBlock.teleport` (bytecode read fresh)
accepts a candidate only when it is air (offsets 98-101),
`WorldBorder.isWithinBounds` (offsets 104-111), and inside build
height (offsets 114-120) — 1,000 attempts, `nextInt(16)`/`nextInt(8)`/
`nextInt(16)` differences. (Note: 1.21.11 has NO "block below is
non-air" check — that part of the 26.1.2 item's evidence is a 26.x
addition, not a 1.21.11 form.) The port's `dragon_egg_teleport`
(`interact.rs:581-600`) checks only air + build height: an egg
near the border can teleport outside it.

### B5-G29 — Bonemeal validity too strict: the `tree_can_grow` gate refuses lone dark/pale oak (MINOR, REPRODUCES — the over-strict direction only)

Vanilla 1.21.11 `SaplingBlock.isValidBonemealTarget` is
unconditionally true (`iconst_1; ireturn` — bytecode-verified fresh;
there is NO `canGrow` method on 1.21.11's `TreeGrower`, 0 hits, and
no `getMinimumHeight`, 0 hits). The port's `Sapling` arm in
`bonemeal_valid` (`bonemeal.rs:331-343`) gates on
`live_features().is_none() || !tree_can_grow(...)` — and
`tree_can_grow` (`growth.rs:443`, the 26.3 form) refuses a lone
dark/pale oak sapling (it needs the 2×2 mega arrangement). Bonemeal
is refused on a lone dark/pale oak sapling where vanilla accepts it.
The gate also draws random on every validity check (see B5-G33).
The other directions of 1.21.7's B5-G15 do NOT reproduce in 1.21.11
— the port's Mushroom arm returns true unconditionally (matching
vanilla 1.21.11's `iconst_1`) and the Azalea arm checks only
fluid-above-empty (matching vanilla) — see §5. The port's own tests
still assert the wrong behaviour (`growth_tests.rs`:
`lone_dark_oak_sapling_cannot_grow`).

### B5-G32 — Grass bonemeal rolls use `nextFloat` where vanilla 1.21.11 uses `nextInt` (MINOR, REPRODUCES)

Vanilla 1.21.11 `GrassBlock.performBonemeal` (bytecode read fresh):
the tall-grass roll is `nextInt(10) == 0` (offsets 170-172) and the
flower-feature roll is `nextInt(8) == 0` (offsets 230-232). The
port's `grass_bonemeal_effect` (`bonemeal.rs:1239` and `:1247`)
uses `self.random().next_float() < 0.1` and `< 0.125`. The
probabilities coincide (1/10, 1/8) but the draws differ — the random
stream desynchronises from vanilla after the first grass bonemeal.

### B5-G33 — TreeGrower is the 26.3 weighted-list form; vanilla 1.21.11 is secondary-chance (MINOR, REPRODUCES)

Vanilla 1.21.11 `TreeGrower` (10 growers: oak, spruce, mangrove,
azalea, birch, jungle, acacia, cherry, dark oak, pale oak — NO
poplar) is the secondary-chance form: a `secondaryChance` float plus
`megaTree`/`secondaryMegaTree`/`tree`/`secondaryTree`/`flowers`/
`secondaryFlowers` optionals; `getConfiguredFeature` draws ONE
`nextFloat()` unconditionally; `getConfiguredMegaFeature` draws
`nextFloat` only when a secondary mega exists; there is NO `canGrow`
method (0 hits). The port's `GROWERS` (`growth.rs:156-269`) is the
26.3 weighted-list form (`trees`/`mega_trees`/`flower_trees` +
`weighted()` using `next_int_bound(total)`), with 11 growers
including a dead `poplar` (no `poplar_sapling` in 1.21.11). The
probabilities coincide but the draw type and count differ — the
random stream desynchronises, and `tree_can_grow` draws on every
validity check (see B5-G29).

### B5-G34 — Mega-tree growth leaves water where vanilla 1.21.11 sets air (MINOR, REPRODUCES)

Vanilla 1.21.11 `TreeGrower.growTree`: the MEGA path clears the 2×2
to AIR with flags 260 and restores the PASSED sapling state on
failure with 260; the SINGLE path clears to the fluid's legacy block
(`createLegacyBlock()`) with flags 260 — water IS vanilla 1.21.11's
single-path form (it differs from 1.21.7, where the single path set
AIR). The port's `remove_sapling` (`growth.rs:392-393`) leaves the
fluid's legacy block on BOTH paths — the mega path leaves water
where vanilla sets air. (The single-path flags 818 vs vanilla 260
is a carried nit.) This is also the mega half of 1.21.7's B5-G14
(see §5).

### B5-G35 — Digging broadcasts level events 2019/2020 that vanilla 1.21.11 never sends (MINOR, REPRODUCES)

Vanilla 1.21.11 has NO level events 2019/2020: `LevelEvent` (now at
`net.minecraft.world.level.block.LevelEvent`; `javap -constants`,
fresh) ends its 20xx range at 2013 (`PARTICLES_SMASH_ATTACK`).
The port's `destroy_progress_event`
(`game/block_breaking.rs:100-101`) sends 2020 every fourth tick
else 2019, handled in `game.rs:4301` and called from
`game.rs:5294/5306`. A 1.21.11 client receiving these events from
the port's server path sees crack effects vanilla never sends.

### B5-G36 — Bonemeal passes on the client for saplings, azalea and mushrooms where vanilla 1.21.11 returns SUCCESS (MINOR, REPRODUCES)

Vanilla 1.21.11 `BoneMealItem.useOn` (bytecode read fresh): a
`growCrop` success returns `SUCCESS` on BOTH sides (offset 36:
`ifne 65` — the client skips the game event and level event but
still returns `SUCCESS` at offset 65); `growCrop` itself returns
true on the client for ANY valid target (offset 44: not a
`ServerLevel` → skip success/perform/shrink → return true). And
1.21.11's validity is client-valid for all three: `SaplingBlock`
and `MushroomBlock` are unconditionally true, `AzaleaBlock` is
fluid-above-empty only — none checks `ServerLevel`. The port's
`bone_meal_succeeds_on_client` (`bonemeal.rs:216-243`) returns true
for valid non-sapling/azalea/mushroom targets but EXCLUDES those
three kinds, with a comment claiming they are "only a valid target
on a ServerLevel" — the 26.x form, wrong for 1.21.11. The client
predicts PASS (no hand swing) where vanilla predicts SUCCESS.

### B5-G37 — Sugar cane cannot be planted: ground check reads the absent `#supports_sugar_cane` tag (MAJOR, REPRODUCES)

Vanilla 1.21.11 `SugarCaneBlock.canSurvive` (bytecode read fresh):
below is sugar cane itself, OR below is in `#DIRT`/`#SAND` with
adjacent water (horizontal check). The port's `SugarCane` arm
(`survive.rs:336-360`) keeps the self-stack branch but gates the
planting branch on `b.in_set(below_state, &t.supports_sugar_cane)`
(`:341`) — the tag is ABSENT from 1.21.11 data (verified in the tag
inventory), so the branch never runs. Sugar cane can only stack on
existing cane; it can never be planted on dirt or sand. (The
adjacent-fluid check reads `supports_sugar_cane_adjacently`,
unreached while the ground branch is dead.)

### B5-G38 — Cocoa can never survive: port reads the absent `#supports_cocoa` tag (MAJOR, REPRODUCES)

Vanilla 1.21.11 `CocoaBlock.canSurvive`: the block the cocoa faces
is in `#JUNGLE_LOGS` (bytecode read fresh). The port's `Cocoa` arm
(`survive.rs:150`) tests `b.in_set(rel_state(facing),
&t.supports_cocoa)` — the tag is ABSENT, so the arm always returns
false. Cocoa cannot survive on any log, jungle or otherwise.

### B5-G39 — Azalea can never survive: port reads the absent `#supports_azalea` tag (MAJOR, REPRODUCES)

Vanilla 1.21.11 `AzaleaBlock.mayPlaceOn`: CLAY or
`VegetationBlock.mayPlaceOn` (`#DIRT` or FARMLAND) (bytecode read
fresh). The port's `Azalea` arm (`survive.rs:403`) tests
`b.in_set(ground, &t.supports_azalea)` — the tag is ABSENT, so the
arm always returns false. Azalea (and flowering azalea) cannot
survive on any ground.

### B5-G40 — Mangrove propagule can never survive, planted or hanging (MAJOR, REPRODUCES)

Vanilla 1.21.11: planted — `mayPlaceOn` is
`SaplingBlock.mayPlaceOn` (`#DIRT`/farmland via `VegetationBlock`)
OR CLAY; hanging — `canSurvive` requires the block above to be
MANGROVE_LEAVES (both bytecode-verified fresh). The port reads the
absent `#supports_mangrove_propagule` tag for the planted form
(`survive.rs:430`) and the absent
`#supports_hanging_mangrove_propagule` tag for the hanging form
(`survive.rs:252-260`) — both always false. Neither form can
survive anywhere.

### B5-G41 — Nether sprouts can never survive: port reads the absent `#supports_nether_sprouts` tag (MAJOR, REPRODUCES)

Vanilla 1.21.11 `NetherSproutsBlock.mayPlaceOn`: `#NYLIUM` or
SOUL_SOIL (bytecode read fresh). The port's `NetherSprouts` arm
(`survive.rs:432`) tests `b.in_set(ground,
&t.supports_nether_sprouts)` — the tag is ABSENT, so the arm
always returns false. Nether sprouts cannot survive on nylium or
soul soil.

### B5-G42 — Nether wart can never survive: port reads the absent `#supports_nether_wart` tag (MAJOR, REPRODUCES)

Vanilla 1.21.11 `NetherWartBlock.mayPlaceOn`: SOUL_SAND, hardcoded
(bytecode read fresh). The port's `NetherWart` arm
(`survive.rs:433`) tests `b.in_set(ground, &t.supports_nether_wart)`
— the tag is ABSENT, so the arm always returns false. Nether wart
cannot survive on soul sand (or anywhere).

### Closed items (NEVER EXISTED — corrections, not fixes)

### B5-G17 — Chorus flower at the build ceiling (CLOSED, NEVER EXISTED — confirms the 1.21.10 correction against 1.21.11)

Fresh 1.21.11 bytecode: `ChorusFlowerBlock.randomTick` proceeds
when `above.y <= getMaxY()` (offsets 15-24: `if_icmple 28`, else
return), and `LevelHeightAccessor.getMaxY()` is `getMinY() +
getHeight() - 1` — i.e. 319 for the overworld. The port
(`tick.rs:1441`, `above_pos.y > max_y() - 1` with `max_y()` = 320
exclusive) returns early when `above.y >= 320`. The two guards are
IDENTICAL. The chorus check is a PASS in 1.21.11, and the 26.1.2
ledger's B5-G17 was a misread of `getMaxY()` — it never existed in
26.1.2, 1.21.10 or 1.21.11.

### B5-G18 — Bell `isProperHit` (CLOSED, NEVER EXISTED — confirms the 26.1.2/1.21.10 correction against 1.21.11)

Fresh 1.21.11 bytecode, read in full: `BellBlock.isProperHit` is
`dload_3; ldc2_w 0.8123999834060669d; dcmpl; ifle 20; 18: iconst_0;
ireturn; 20: <attachment switch>` — vanilla requires `clickY <=
0.8124` (after the Y-axis exclusion), exactly what the port
implements (`redstone.rs:1442`: `click_y <= f64::from(0.8124f32)`,
with the axis + attachment arms matching). The port was correct all
along; the 26.2 run's `ifle` misread does not recur here. The bell
check is a PASS.

## Oracle defects

Triage is PROVISIONAL (see §0): no measured Batch 0 oracle table
exists for 1.21.11. Each defect below cites the oracle source line
in `mods/horizon/src/test/java/dev/horizonlab/parity/` of the
1.21.11 tree and the javap symbol check against the 1.21.11 jars.
Full `Vanilla*Oracle` names throughout. Batch 0's measured table,
when it lands, should confirm these seven (plus the two FIXED
oracles below); a measured-table disagreement supersedes this
section.

### B5-O01 — `VanillaCauldronOracle` fails: `Player.drop` 3-arg override uses `net.minecraft.util.Prediction`, absent in 1.21.11 (ORACLE DEFECT, MINOR, REPRODUCES — 1.21.7's O01 cause)

The oracle still overrides `Player.drop` with the 3-arg signature
(`VanillaCauldronOracle.java:118`: `drop(ItemStack, boolean,
net.minecraft.util.Prediction)`). The 1.21.11 jar has NO
`net.minecraft.util.Prediction` (verified: 0 entries in both jars)
and `Player` has only `drop(ItemStack, boolean)`. One-symbol
repair: drop the third parameter.

### B5-O04 — `VanillaReplaceBlockOracle` fails: `Feature.DIRECT_CODEC` does not exist in 1.21.11 (ORACLE DEFECT, MINOR, REPRODUCES — 1.21.7's O04 cause)

The oracle still reads `Feature.DIRECT_CODEC`
(`VanillaReplaceBlockOracle.java:36`). The 1.21.11 `Feature` class
has NO `DIRECT_CODEC` (javap-verified fresh: only the
`configuredCodec()` instance method); `ConfiguredFeature` HAS
`DIRECT_CODEC`. Same defect, same cause as 1.21.7's O04 and
1.21.10's O04.

### B5-O05 — `VanillaFallingBlockOracle` fails: `ChunkMap` constructor mismatch at line 30 (ORACLE DEFECT, MINOR, REPRODUCES — 1.21.10's O05 cause)

`NoopChunkMap extends ChunkMap` with a 12-arg super call
(`VanillaFallingBlockOracle.java:30`: 10 nulls + `0` + `false`).
The 1.21.11 `ChunkMap` has ONE constructor, with 13 parameters
(`ServerLevel, LevelStorageAccess, DataFixer,
StructureTemplateManager, Executor, BlockableEventLoop,
LightChunkGetter, ChunkGenerator, ChunkStatusUpdateListener,
Supplier, TicketStorage, int, boolean` — javap-verified fresh).
The super call does not match. (The 1.21.7 O05 cause — the missing
`net.minecraft.util.debug` package — does NOT apply: the package
EXISTS in 1.21.11, `ServerDebugSubscribers` is in the server jar,
and the oracle's debug-subscriber use at line 40 resolves; the
constructor is the failure. See §5.)

### B5-O06 — `VanillaDispenserOracle` fails: missing `EntityTypes` (ORACLE DEFECT, MINOR, REPRODUCES — the 1.21.10 cause, narrowed)

One failure remains in 1.21.11 (1.21.10 had two). The oracle
references `EntityTypes.PLAYER` at lines 212, 235-236, 560-561,
618-619 (via the wildcard `net.minecraft.world.entity.*` import);
`EntityTypes` does not exist in 1.21.11 — registration still lives
on `EntityType` itself, the holder move happened after 1.21.11.
(The 1.21.7/1.21.10 (a) cause — the `gamerules` package — has
EVAPORATED: `net.minecraft.world.level.gamerules.GameRules`
EXISTS in 1.21.11, so the import at line 27 now resolves. The
oracle still fails on `EntityTypes` alone.)

### B5-O07 — `VanillaSectionMeshOracle` fails: `MaterialBaker` absent from both jars (ORACLE DEFECT, MINOR, REPRODUCES — 1.21.10's O07 cause)

`Materials extends MaterialBaker`
(`VanillaSectionMeshOracle.java:84`). `MaterialBaker` is ABSENT
from both 1.21.11 jars (0 entries, verified fresh in the
server-game and client jars). (The 1.21.7 O07 cause — `blaze3d`
absent from both classpaths — does NOT apply: the 1.21.11 client
jar HAS `com.mojang.blaze3d` (234 entries). See §5.)

### B5-O08 — `VanillaBreakingModelOracle` fails: `BlockModelFeatureRenderer` absent (ORACLE DEFECT, MINOR, REPRODUCES — 1.21.10's O08 cause)

The oracle imports `BlockModelFeatureRenderer`
(`VanillaBreakingModelOracle.java:12`). The class does not exist
in 1.21.11 (absent from the client jar, verified fresh — 0
entries). (Note: the related `BlockFeatureRenderer` DOES exist, at
`net.minecraft.client.renderer.feature.BlockFeatureRenderer` — the
oracle needs the model variant, which does not.) The oracle ALSO
imports `MaterialBaker` (line 15, absent — see B5-O07), a second
independent failure. (The 1.21.7 O07 `blaze3d` cause does not
apply — see B5-O07.)

### B5-O09 — `VanillaFaceSupportOracle` fails: protected `CubeVoxelShape` constructor at line 45 (ORACLE DEFECT, MAJOR, REPRODUCES — 1.21.10's O09 cause)

The oracle constructs `CubeVoxelShape` directly
(`VanillaFaceSupportOracle.java:45`: `new CubeVoxelShape(bits)`).
The 1.21.11 constructor is PROTECTED (`protected
CubeVoxelShape(DiscreteVoxelShape)` — javap-verified fresh).
MAJOR because it blocks 21 driver files in `tools/` plus 7 oracle
callers in the 1.21.11 tree (40 files reference the oracle
overall). (Batch 2 also records it; the coordinator
deduplicates.)

### Fixed oracles (no defect — symbol-level, provisional on Batch 0's table)

- **B5-O02 — `VanillaPortalOracle` — COORDINATOR CORRECTION: still
  fails (oracle-defect), NOT fixed.** The worker's symbol-level
  evidence stands — `Entity.getRelativePortalPosition(Direction$Axis,
  BlockUtil$FoundRectangle)` EXISTS in 1.21.11 (javap-verified
  fresh) with exactly the signature the oracle calls at line 180,
  and the oracle's other symbols resolve too — but the claim that
  "the cascade evaporates" is WRONG: the landed Batch 0 table shows
  `VanillaPortalOracle` failing on all three classpaths with first
  error "depends on broken: VanillaBlockUpdateOracle", and
  `VanillaBlockUpdateOracle` still fails with 38 errors (first:
  `net.minecraft.world.clock` package does not exist — the
  `gamerules` import resolving was not enough). The oracle does not
  compile for 1.21.11. Lands as an oracle-defect item (own cause
  resolved, dependency cause standing).
- **B5-O03 — `VanillaPickBlockDataOracle` FIXED in 1.21.11.**
  `TypedEntityData` HAS both `loadInto` overloads in 1.21.11
  (`loadInto(Entity)` and `loadInto(BlockEntity,
  HolderLookup$Provider)` — javap-verified fresh with `-p`), and
  the oracle's call at line 59 (`data.loadInto(entity,
  ParityData.access)`, where `entity` is a `BlockEntity`) matches
  the 2-arg overload; `data.type()` and
  `BlockEntityType.onlyOpCanSetNbt()` also resolve. The oracle
  source differs from 1.21.10's only in the
  `ResourceLocation`→`Identifier` renames. This confirms the
  coordinator's 1.21.10-ledger correction (the 1.21.10 slice's
  O03 was wrong — `TypedEntityData` has the overloads there too).
  Lands as a FIXED item.

### Oracle non-items

- **1.21.7 B5-O05 (debug package) — cause evaporates; oracle still
  fails on the 1.21.10 cause.** The `net.minecraft.util.debug`
  package EXISTS in 1.21.11 — the 1.21.7 item's premise fails.
  No separate item; the oracle's failure is B5-O05 (ChunkMap).
- **1.21.7 B5-O07 (`blaze3d`) — cause evaporates; both oracles
  still fail on the 1.21.10 causes.** The 1.21.11 client jar HAS
  `blaze3d` (234 entries) — the 1.21.7 item's premise fails (the
  1.21.7 Batch 0 measurement had no named client jar; 1.21.11
  does). No separate item; the failures are B5-O07
  (`MaterialBaker`) + B5-O08 (`BlockModelFeatureRenderer`).
- **1.21.7 B5-O06 (`gamerules` package) — cause evaporates; the
  oracle still fails on `EntityTypes`.** The package EXISTS in
  1.21.11. No separate item; the failure is B5-O06.

**In-scope oracles passing (provisional):**
VanillaBlockSurvivalOracle, VanillaBlockSupportOracle,
VanillaBlockPropertiesOracle, VanillaBlockStateCodecOracle,
VanillaCollisionOracle, VanillaHeightmapOracle — all six are clean
of the suspect imports (`EntityTypes`, `BlockEntityTypes`,
`Prediction`, `util.debug`, `blaze3d`, `MaterialBaker`,
`BlockModelFeatureRenderer`, `Feature.DIRECT_CODEC`, and the OLD
`net.minecraft.world.level.LevelEvent` / `GameRules` paths) and
their key symbols (`SupportType`, `Heightmap`,
`mayPlaceOn`/`canSurvive`) verify against the 1.21.11 jars — plus
VanillaPickBlockDataOracle (FIXED, see
above); VanillaPortalOracle still fails via its BlockUpdate
dependency (coordinator correction, see §4). Note the `gamerules` import is no longer suspect in
1.21.11: the package exists, so oracles importing it (e.g.
VanillaBlockUpdateOracle, VanillaDispenserOracle) resolve that
import.

## §5 Re-check verdicts

### PRIMARY — 1.21.10 items (31/31 decided: 28 REPRODUCE, 1 FIXED, 2 NEVER EXISTED)

| 1.21.10 item | Verdict | 1.21.11 item / evidence |
|---|---|---|
| B5-G04 fungi/roots survive nowhere | REPRODUCE | B5-G04 — `survive.rs:399-402` reads `data.support_tag` = `None` for both; exporter is reflective, `FungusBlock` has only `requiredBlock` + `feature`, `RootsBlock` has no fields |
| B5-G08 snow-layer tags | REPRODUCE | B5-G08 — `survive.rs:312-324` reads absent `#cannot_support_snow_layer`/`#support_override_snow_layer`; vanilla's own tags are present in 1.21.11 |
| B5-G09 seagrass on magma | REPRODUCE | B5-G09 — `survive.rs:440-443` reads absent `#cannot_support_seagrass` |
| B5-G11 cactus flower | REPRODUCE | B5-G11 — `survive.rs:404-408` reads absent `#support_override_cactus_flower`; cactus/farmland CENTER_UP false |
| B5-G13 dripstone | REPRODUCE | B5-G13 — dispatch live (`tick.rs:257/443`, `shapes.rs:761`, `place.rs:1165`); `#speleothems` still absent |
| B5-G17 chorus ceiling | **NEVER EXISTED** | CLOSED — `getMaxY()` = minY+height−1 = 319; port guard identical (`tick.rs:1441`) |
| B5-G18 bell isProperHit | **NEVER EXISTED** | CLOSED — vanilla requires clickY ≤ 0.8124; port matches (`redstone.rs:1442`) |
| B5-G19 mud conversion | REPRODUCE | B5-G19 — port reads `#convertible_to_mud` ('i'); 1.21.11's tag is `#convertable_to_mud` ('a') |
| B5-G21 snow accumulation | REPRODUCE | B5-G21 — `weather.rs:46` + `feature_world.rs:350-356` read absent `#blocks_motion_in_heightmap` |
| B5-G22 composter | REPRODUCE | B5-G22 — port reads `minecraft:compostable`; 0/1,505 items carry it in 1.21.11 |
| B5-G23 ice melt event | REPRODUCE | B5-G23 — `hooks.rs:716` fires `block_destroy`; vanilla `IceBlock` has zero `gameEvent` refs |
| B5-G24 ice playerDestroy | REPRODUCE | B5-G24 — `hooks.rs:549` reads absent `#blocks_motion` |
| B5-G25 tripwire 1-tick | REPRODUCE | B5-G25 — `rails.rs:595-596` schedules 1 when unpressed; vanilla schedules only 10 while pressed |
| B5-G26 eyeblossom scan | REPRODUCE | B5-G26 — `timeline.rs:94` uses section-major `box_scan_order` (`:10`), vanilla `betweenClosed` |
| B5-G27 bed head click | REPRODUCE | B5-G27 — `interact.rs:365-386` requires the other half for both parts |
| B5-G28 dragon egg border | REPRODUCE | B5-G28 — `interact.rs:581-600` has no `WorldBorder` check |
| B5-G29 bonemeal validity | REPRODUCE | B5-G29 — `bonemeal.rs:335` gates on `tree_can_grow`; vanilla `isValidBonemealTarget` is `true` |
| B5-G32 grass bonemeal rolls | REPRODUCE | B5-G32 — `bonemeal.rs:1239/1247` use `next_float`; vanilla uses `nextInt(10)`/`nextInt(8)` |
| B5-G33 TreeGrower form | REPRODUCE | B5-G33 — `growth.rs:156-269` is the weighted-list form with dead poplar; vanilla is secondary-chance, no `canGrow` |
| B5-G34 mega-tree water | REPRODUCE | B5-G34 — `growth.rs:392-393` leaves legacy fluid on both paths; vanilla mega path sets AIR |
| B5-G35 level events 2019/2020 | REPRODUCE | B5-G35 — `block_breaking.rs:100-101`; vanilla `LevelEvent` ends 20xx at 2013 |
| B5-G36 bonemeal client pass | REPRODUCE | B5-G36 — `bonemeal.rs:216-243` excludes sapling/azalea/mushroom on the client; vanilla returns SUCCESS |
| B5-G37 sugar cane | REPRODUCE | B5-G37 — `survive.rs:341` reads absent `#supports_sugar_cane` |
| B5-G38 cocoa | REPRODUCE | B5-G38 — `survive.rs:150` reads absent `#supports_cocoa` |
| B5-G39 azalea | REPRODUCE | B5-G39 — `survive.rs:403` reads absent `#supports_azalea` |
| B5-G40 mangrove propagule | REPRODUCE | B5-G40 — `survive.rs:256`/`430` read absent `#supports_[hanging_]mangrove_propagule` |
| B5-G41 nether sprouts | REPRODUCE | B5-G41 — `survive.rs:432` reads absent `#supports_nether_sprouts` |
| B5-G42 nether wart | REPRODUCE | B5-G42 — `survive.rs:433` reads absent `#supports_nether_wart` |
| B5-O02 Portal | **REPRODUCES** (oracle-defect) | own symbol resolved, but measured table shows fail-all: depends on broken VanillaBlockUpdateOracle (38 errors, clock package) |
| B5-O03 PickBlockData | **FIXED** | `TypedEntityData` has both `loadInto` overloads in 1.21.11; oracle call matches |
| B5-O04 ReplaceBlock | REPRODUCE | B5-O04 — `Feature.DIRECT_CODEC` does not exist in 1.21.11 |

### SECONDARY — 1.21.7 items (22/22 decided: 11 REPRODUCE, 9 FIXED, 2 NEVER EXISTED as stated)

| 1.21.7 item | Verdict | 1.21.11 item / evidence |
|---|---|---|
| B5-G01 farmland | **FIXED** | Aliases mapped; tick/survive/updateShape/placement arms live; trample (`block_landing.rs:416-441`) matches vanilla `fallOn` |
| B5-G02 grass/mycelium | **FIXED** | Aliases mapped; spread arms live |
| B5-G03 dirt path | **FIXED** | `DirtPathBlock` mapped; updateShape schedules tick 1 |
| B5-G04 fungi/roots | REPRODUCE | B5-G04 — same item as PRIMARY (the worse form: classes mapped, arm fires, tag None) |
| B5-G05 stems | **FIXED** | Farmland hardcoded; `StemBlock` has `fruit`/`attachedStem` ResourceKey fields |
| B5-G06 crops | **FIXED** | Hardcoded farmland + moisture growth speed |
| B5-G07 mushrooms | **FIXED** | `survive.rs:285-289` uses present `#mushroom_grow_block`, brightness < 13, `mayPlaceOn` isSolidRender |
| B5-G08 snow-layer tags | REPRODUCE | B5-G08 — same item as PRIMARY |
| B5-G09 seagrass | REPRODUCE | B5-G09 — same item as PRIMARY |
| B5-G10 kelp | **FIXED** | Magma refusal hardcoded in `growing_plant_can_attach_to` |
| B5-G11 cactus flower | REPRODUCE | B5-G11 — same item as PRIMARY |
| B5-G12 fletching table | **FIXED** | No `FletchingTableBlock` in either 1.21.11 jar (0/0); no variant in `classes.rs`; zero "fletching" mentions in level/game code |
| B5-G13 dripstone | REPRODUCE | B5-G13 — same item as PRIMARY (dispatch live, tag dead) |
| B5-G14 tree clear | SPLIT — REPRODUCE (mega) | Single path EVAPORATES — vanilla 1.21.11 single path IS `createLegacyBlock`/water; mega path REPRODUCES as B5-G34 |
| B5-G15 bonemeal directions | SPLIT — REPRODUCE (sapling) | Sapling over-strict direction REPRODUCES as B5-G29; mushroom/azalea directions FIXED — port returns true / fluid-above-empty, matching vanilla 1.21.11 |
| B5-G17 chorus ceiling | **NEVER EXISTED** | CLOSED — same as PRIMARY (checked fresh: `getMaxY()` = 319) |
| B5-G18 bell | **NEVER EXISTED** | CLOSED — same as PRIMARY (checked fresh: clickY ≤ 0.8124) |
| B5-O01 Cauldron | REPRODUCE | B5-O01 — `Player.drop` 3-arg override uses absent `util.Prediction` |
| B5-O02 Portal | **REPRODUCES** | Same as PRIMARY |
| B5-O03 PickBlockData | **FIXED** | Same as PRIMARY |
| B5-O04 ReplaceBlock | REPRODUCE | B5-O04 — same item as PRIMARY |
| B5-O05 FallingBlock | **NEVER EXISTED as stated** | `net.minecraft.util.debug` EXISTS in 1.21.11 — the premise fails; oracle still fails on the ChunkMap constructor (B5-O05) |
| B5-O06 Dispenser | REPRODUCE | B5-O06 — `gamerules` cause EVAPORATES (package exists); `EntityTypes` cause stands |
| B5-O07 SectionMesh/BreakingModel | **NEVER EXISTED as stated** | `blaze3d` EXISTS in 1.21.11 (234 entries) — the premise fails; oracles still fail on `MaterialBaker` (B5-O07) / `BlockModelFeatureRenderer` (B5-O08) |

## §6 Systemic findings

### 6.1 The template ≈ its first child (CORRECTION to the 1.21.10 report's §6.1)

The 1.21.10 report (§6.1) said the child carries "the template's
defects PLUS its own adaptation debt" and named 16 `level/` files
differing from the template. This run re-ran the diff the other way
(template 1.21.11 vs child 1.21.10) and the 16 files are the same —
but the diffs are almost entirely **comment renames** (`1.21.10` →
`1.21.11` in comments). There are exactly THREE real code diffs in
`level/`, all three matching vanilla 1.21.11's own changes:

1. `bonemeal.rs` — sea pickle gate `position != pos && next_int(6)
   == 0` → `position != pos` + `.equals` form (vanilla 1.21.11
   `SeaPickleBlock.performBonemeal` uses `BlockPos.equals` at
   offset 94 — the 1.21.10 form used a `nextInt(6)` roll; NEW
   CHECK, PASS).
2. `removal.rs` — jukebox pop drops the dy random draw
   (`spawnItem` with fixed y 0.5/1.01/0.5 — vanilla 1.21.11
   `JukeboxBlockEntity.popOutTheItem` uses fixed offsets; NEW
   CHECK, PASS).
3. `timeline.rs` — turtle hatch `None => 0.0` (vanilla 1.21.11's
   `TURTLE_EGG_HATCH_CHANCE` environment attribute defaults to
   `Float 0.0f` in `EnvironmentAttributes.<clinit>`; PASS).

The class aliases (FarmBlock, DirtPathBlock, SpreadingSnowyDirtBlock,
SnowyDirtBlock, FungusBlock, RootsBlock, RedStoneWireBlock) and the
hardcoded forms (farmland, magma refusal, stem/crop arms) were
ALREADY in the template — the 1.21.10 "adaptation" did not add them.
The correct statement: **the template carries the same absent-tag
defects as its child; the 1.21.10 port adapted almost nothing in
`level/` beyond comments.** Consequence for the sweep: the 1.21.10
ledger is an accurate template ledger for B5 (27 of 31 items
reproduce unchanged), and the three diffs are all correct
adaptations — there is no adaptation debt in `level/` for B5.

### 6.2 Vanilla 1.21.11 changed in B5-relevant ways (all javap/data-verified)

- **GameRules moved and changed.** `net.minecraft.world.level.gamerules`
  (data-defined registry, 59 rules); NO `doFireTick` rule exists.
  The fire gate is now `FIRE_SPREAD_RADIUS_AROUND_PLAYER`
  (default 128, min −1) via `ServerLevel.canSpreadFireAround` →
  `ChunkMap.anyPlayerCloseEnoughTo` → `playerIsCloseEnoughTo`
  (spectator excluded, strict distance < radius from the block
  corner). The port implements this faithfully (`game.rs:1562-1573`,
  `fire_players` excludes spectators, default 128) — **the fire
  tick check PASSES, and the 1.21.10 B1 item (doFireTick) is
  WITHDRAWN for 1.21.11 — B1 should not re-mint it.**
  `MAX_SNOW_ACCUMULATION_HEIGHT` arrived (default 1, min 0, max 8);
  the port models it exactly (`tick.rs:1539-1544`).
- **`minecraft:environment_attribute` registry arrived (45 entries).**
  `IceBlock.melt`/`playerDestroy`, `BedBlock.useWithoutItem`,
  `TurtleEggBlock.shouldUpdateHatchLevel`, `EyeblossomBlock` read
  `WATER_EVAPORATES` / `BED_RULE` / `TURTLE_EGG_HATCH_CHANCE` /
  `EYEBLOSSOM_OPEN` through `EnvironmentAttributeSystem`. The
  port models these as hardcoded per-dimension values with the
  timeline keys matching vanilla's `day.json` exactly (see §7) —
  the environment-attribute form is a separate concern (B3/B12),
  not a B5 defect; the B5 checks that depend on the VALUES pass.
- **LevelEvent moved** to `net.minecraft.world.level.block.LevelEvent`
  (82 constants; 20xx ends at 2013 — no 2019/2020, so B5-G35
  reproduces).
- **`Entity.getRelativePortalPosition` arrived** (exact signature
  the oracle calls — but B5-O02 still fails via its BlockUpdate
  dependency, coordinator correction); **`TypedEntityData` gained
  both `loadInto` overloads** (B5-O03 FIXED).
- **Single-path tree clearing changed**: vanilla 1.21.11's single
  path sets the fluid's legacy block (water), unlike 1.21.7's AIR —
  so the 1.21.7 B5-G14 single-path half EVAPORATES for 1.21.11
  (the mega half still reproduces as B5-G34).
- **Still absent** (0 entries in both jars unless noted):
  `EntityTypes`/`BlockEntityTypes`, `net.minecraft.util.Prediction`,
  `MaterialBaker`, `BlockModelFeatureRenderer`, `AbstractBedBlock`,
  `FletchingTableBlock`, `SpeleothemBlock`, `NetherFungusBlock`,
  `BambooBlock`. **Present**: `net.minecraft.util.debug`
  (`ServerDebugSubscribers`), `com.mojang.blaze3d` (234 entries),
  `BlockFeatureRenderer` (client only), `BedBlockEntity` (1/1).

### 6.3 The absent-tag root cause is unchanged (one tag inventory, 204 tags)

The 1.21.11 tag inventory (204 tags, built jar-direct) differs from
1.21.10's (203) by exactly one tag: `can_glide_through` (out of B5
scope). Every 1.21.10-absent tag is still absent:
`#speleothems`, `#nether_portal_frame`, `#blocks_motion_in_heightmap`
(`_no_leaves`), `#convertible_to_mud` ('i'), `#blocks_motion`,
`#blocks_fluid_flow`, `#washed_away_by_fluids`,
`#blocks_lava_fire_spread`, `#sculk_growth_inhibitors`,
`#douses_campfires`, `#grows_crops`,
`#overrides_mushroom_light_requirement`,
`#cannot_support_snow_layer`, `#support_override_snow_layer`,
`#cannot_support_seagrass`, `#cannot_support_kelp`,
`#support_override_cactus_flower`, ALL `supports_*` tags checked
(sugar_cane, cocoa, azalea, mangrove_propagule,
hanging_mangrove_propagule, nether_sprouts, nether_wart),
`#prevents_nearby_leaf_decay`, `#causes_suffocation`,
`#suppresses_bounce`. Present: `#convertable_to_mud` ('a': dirt,
coarse_dirt, rooted_dirt), `#mushroom_grow_block`,
`#snow_layer_cannot_survive_on` (ice, packed_ice, barrier),
`#snow_layer_can_survive_on` (honey_block, soul_sand, mud),
`#maintains_farmland`, `#sniffer_egg_hatch_boost`, `#snow`, `#dirt`,
`#nylium`, `#logs`, `#dry_vegetation_may_place_on`,
`#jungle_logs`, `#sand`, `#small_dripleaf_placeable`,
`#big_dripleaf_placeable`. No `synthesize_legacy_support_tags`
anywhere in the 1.21.11 tree (grep 2026-11-25). The port's
26.x-shaped tag reads are the single root cause of G04, G08, G09,
G11, G13, G19, G21, G24, G37–G42.

### 6.4 The exporter reflection gap (B5-G04's root cause)

`BlockData` is exported reflectively: the exporter dumps `TagKey`
fields per block class. But 1.21.11's `FungusBlock` and
`RootsBlock` have NO `TagKey` fields (fungus: `requiredBlock` +
`feature`; roots: none) — their `mayPlaceOn` sets are
BYTECODE, not data. Any reflective exporter will always produce
`support_tag = None` for them, and the port's `NetherFungus |
NetherRoots` arm then returns false. A fix needs hardcoded
`mayPlaceOn` forms (as the port already does for farmland,
mushrooms, kelp and stems), not more export.

### 6.5 Random-stream desynchronisation (B5-G32, B5-G33)

Two items share one theme: the port's random draws are the 26.3
form where vanilla 1.21.11 uses a different draw. Grass bonemeal
(`nextFloat` vs `nextInt(10)`/`nextInt(8)`) and the TreeGrower
(weighted `next_int_bound` vs single unconditional `nextFloat`,
plus the spurious `canGrow` draw on every validity check) both
desynchronise the random stream from vanilla. The probabilities
coincide, so these are MINOR, but they make deterministic
replay/seed verification impossible for affected features.

### 6.6 No NEW behaviour gaps for 1.21.11

Every 1.21.10 gap reproduces; the three 1.21.10→1.21.11 code diffs
are all correct adaptations; the two new checks pass. The
1.21.11 B5 ledger is the 1.21.10 ledger minus two fixed oracles
and two closed items, plus the withdrawn fire item (B1's, not
B5's). The template's B5 debt is the child's B5 debt.

## Cross-referenced gaps (not re-minted in B5 — other batches own them)

- Fire spread / `doFireTick`: the 1.21.10 B1 item is WITHDRAWN
  for 1.21.11 (no such rule; the radius gate is implemented and
  matching) — B1 should NOT re-mint it.
- Sculk inhibitors (`level/sculk.rs` reads absent
  `#sculk_growth_inhibitors`) → B4.
- Campfire douse (absent `#douses_campfires`) → B7.
- `can_hold_any_fluid` / lava-fire walk → B1.
- Diode guard → B2-G07. Redstone wire class → B2-G01 (1.21.11's
  wire is `RedStoneWireBlock`, capital S; the port alias covers it).
- B3-G22 startup blocker tags (`#causes_suffocation` /
  `#suppresses_bounce`, both verified absent) → B3.
- Dripstone cauldron fill → B1.
- Loot tables behind `drop_resources` → B3.
- TreeFeatureOracle → B6.
- Saved-palette legacy form (`chunk.rs` `from_palette_nbt` —
  absent in the 1.21.11 tree per coordinator Step 0) → B11.
- Loading stick (1.21.11 carries the identical code to the 1.21.7
  defect fixed by 775a25c2) — out of B5 scope, flagged only.
- `feature_world.rs` heightmap tags folded into B5-G21's evidence
  (same root cause, not minted separately).
- `snow_survives` (`weather.rs`) folded into B5-G08.
- Poplar grower dead (folded into B5-G33); single-path tree flags
  818 vs vanilla 260 (folded into B5-G34).

## §7 Passes, nits, and evidence

### Passes by area (all verified fresh against 1.21.11's jar/tree)

**Ticks (32/37).** Fire (`game.rs:1562-1573` radius gate, default
128, spectators excluded — vanilla 1.21.11's form; **PASSES**, the
1.21.10 `doFireTick` item is withdrawn); frosted ice (vanilla
reschedules `Mth.nextInt(20, 40)`; port dispatches
`frosted_ice_tick`); nether wart (vanilla `nextInt(10)`); cocoa
(`nextInt(5)`); sweet berry (`nextInt` + brightness ≥ 9); bamboo
(`nextInt` + brightness ≥ 9); vine (`SPREAD_VINES` rule + `nextInt`);
chorus flower (B5-G17 guard identical); farmland moisture/decay;
grass/mycelium spread; crop growth (moisture speed); stem growth;
mushroom spread; snow-layer melt; kelp growth; cactus growth;
sugar-cane growth; nether-wart age; turtle egg (see below);
eyeblossom (see below); dripstone scheduled tick dispatch (the
tick dispatch is live — the gap is the tag predicates, B5-G13);
composter tick (level 7 → 8 + ready sound, `tick.rs:86`); tripwire
pressed re-check (10 ticks); observer/piston/redstone-wire ticks
(out of B5 scope but the wire class alias is correct);
precipitation freeze/snow form (the cap and push-up form match —
the gap is the heightmap top, B5-G21).

**Survival/support (18/28).** Farmland, grass/mycelium, dirt path,
stems, crops, mushrooms (present `#mushroom_grow_block`,
brightness < 13, `mayPlaceOn` isSolidRender), kelp (hardcoded
magma refusal), fletching table (no class in either jar), sapling
(ground `#DIRT`/farmland), sniffer egg (hardcoded moss_block —
the tag holds only moss_block, equivalent), turtle egg (sand),
bamboo, vine, tripwire, tripwire hook, bell (B5-G18), chorus,
portal frame (obsidian hardcoded, `portal.rs:43-46`), sponge
(vanilla `SpongeBlock` has 0 `canHoldAnyFluid` refs; the port
uses its own water removal).

**Updates (15/16).** All updateShape arms except dripstone's
thickness gate (B5-G13): farmland, path, snow layer, crops,
stems, mushrooms, kelp, seagrass, cactus flower, bell, chorus,
portal, tripwire, redstone wire, piston/observer.

**Placement (10/11).** All placement arms except dripstone's
thickness (B5-G13): farmland, path, snow, crops, stems,
mushrooms, kelp, seagrass, bell, portal.

**Interaction (16/21).** Sea pickle bonemeal (NEW CHECK, PASS —
vanilla 1.21.11 `SeaPickleBlock.performBonemeal` uses
`BlockPos.equals` at offset 94; the port's `.equals` form
matches); grass bonemeal structure (the gap is the roll type,
B5-G32); sapling bonemeal growth path (the gap is validity,
B5-G29); mushroom/azalea bonemeal validity (FIXED vs 1.21.7);
composter tick interaction (insertion dead, B5-G22); jukebox
insert/eject (the gap is the pop position, now PASS — see
breaking); bell ring; bed sleep path (the gap is the head-click
gate, B5-G27); dragon egg teleport form (the gap is the border,
B5-G28); tripwire hook attach; observer trigger; dispenser
water-bottle form (the gap is the mud tag, B5-G19); frosted-ice
melt; nether-wart harvest; cocoa harvest; sweet-berry harvest.

**Breaking/removal (12/15).** Jukebox pop position (NEW CHECK,
PASS — vanilla 1.21.11 `JukeboxBlockEntity.popOutTheItem` uses
fixed 0.5/1.01/0.5 offsets; the port's `removal.rs` drops the dy
random draw); door break (the `!correct_tool` extra 2001 event
is a carried nit, see below); bed break (foot/head halves);
chorus break; dripstone break (scheduled-tick dispatch live);
tripwire break; bell break; portal break; crop/stem/mushroom
break; snow-layer break.

**Structural (1/2).** Class chains: all 50 B5-scope chains
verified from the 1.21.11 jar; `Blocks` field set identical to
1.21.10 (1,142 fields, 0 diff). Tag inventory: GAP (the systemic
root — see §6.3).

**Timeline values (verified fresh from `data/minecraft/timeline/day.json`):**
eyeblossom_open keyframes 12600 → true, 23401 → false — the
port's `(12600..23401).contains(&t)` matches exactly;
turtle_egg_hatch_chance keyframes 21062 → 1.0, 21905 → 0.002,
constant ease — the port's `Some(t) if (21062..21905).contains(&t)
=> 1.0, Some(_) => 0.002` matches exactly; the overworld includes
the day timeline via `#minecraft:in_overworld`. The turtle
`None => 0.0` default matches the attribute default
(`TURTLE_EGG_HATCH_CHANCE` = `Float 0.0f` in
`EnvironmentAttributes.<clinit>`).

### Nits recorded as passes (carried from 1.21.10, re-verified where in scope)

- Door `!correct_tool` extra 2001 event.
- Sniffer-egg hardcoded `moss_block` (the tag holds only
  moss_block — equivalent).
- Tripwire disarm flags 260 vs vanilla 3.
- Bamboo dead `bamboo_height_above` query.
- Sleep-height shape-top form.
- Poplar grower dead (folded into B5-G33).
- Single-path tree flags 818 vs vanilla 260 (folded into B5-G34).
- Turtle-egg/eyeblossom hardcoded overworld timelines — the
  keyframe VALUES are verified fresh against `day.json` (see
  above); the hardcoding itself is a B3/B12 concern, not a B5
  defect.

### Evidence index

Scratch dir: `~/workspace/mcir-work/sweep-1.21.11-batches/b05/`.
Evidence in `evidence/`:

- `block-tags-1.21.11.json` — 204 tags from the client jar.
- `server-classes.txt` — full server-game jar class listing
  (used for the 0/0 absence checks).
- `block-chains-1.21.11.json` — 50 B5-scope class chains verified
  from javap headers + the `Blocks` field-set comparison (1,142
  fields, 0 diff vs 1.21.10).
- `javap/` — 82 dump files: per-class `javap -c -p` for all
  in-scope vanilla classes (FarmBlock, DirtPathBlock,
  SpreadingSnowyDirtBlock, SnowyDirtBlock, FungusBlock,
  RootsBlock, StemBlock, CropBlock, MushroomBlock, SnowLayerBlock,
  SeagrassBlock, KelpBlock, CactusFlowerBlock,
  PointedDripstoneBlock, BellBlock, TreeGrower, SaplingBlock,
  AzaleaBlock, SweetBerryBushBlock, BoneMealItem, GrassBlock,
  ComposterBlock, PotionItem, IceBlock, TripWireBlock,
  TripWireHookBlock, EyeblossomBlock, BedBlock, DragonEggBlock,
  ChorusFlowerBlock, SugarCaneBlock, CocoaBlock, NetherWartBlock,
  MangrovePropaguleBlock, NetherSproutsBlock, VegetationBlock,
  FireBlock, BambooStalkBlock, VineBlock, TurtleEggBlock,
  SnifferEggBlock, FrostedIceBlock, DispenserBlock, ObserverBlock,
  PistonBaseBlock, RedStoneWireBlock, SpongeBlock, JukeboxBlock,
  SeaPickleBlock, LevelHeightAccessor, PortalShape, Player,
  Entity, ChunkMap, Feature, ConfiguredFeature, DataComponents,
  CubeVoxelShape, BlockTags, BlockEntityType, SupportType, Level,
  ServerLevel, BlockBehaviour, GameRules, TypedEntityData,
  JukeboxBlockEntity, EnvironmentAttributes, NetherPortalBlock,
  PortalForcer, DimensionType, BlockUtil, plus `Blocks-cv.txt`
  (`Blocks -c -v`, both jars' `-p` listings), `Heightmap-v.txt`,
  `LevelEvent-constants.txt`, and the batch dump files
  `batch2/3/4/5.txt`). Note: `BellBlock.txt`'s header lines are
  blank/corrupted at the top but the `isProperHit` method body
  was read intact; the superclass (`BaseEntityBlock`) was
  re-verified separately with `javap -p`.

Key port line numbers (1.21.11 tree, `versions/1.21.11/crates/`):
`horizon-world/src/level/survive.rs` G04 :399-402, G08 :312-324,
G09 :440-443, G11 :404-408, G37 :336-360 (tag read :341), G38
:150, G39 :403, G40 :252-260 (hanging) / :430 (planted), G41 :432,
G42 :433; `speleothem.rs:13-17`, `falling.rs:51`,
`shapes.rs:1176` / `:761`, `place.rs:1398` / `:1165`,
`tick.rs:257/443` (G13); `tick.rs:1441` (G17);
`redstone.rs:1442` (G18); `data.rs:458`, `potions.rs:710`,
`remote.rs:8676`, `dispenser.rs:2251` (G19); `weather.rs:46`,
`level/feature_world.rs:350-356` (G21); `items.rs:1046`,
`composter.rs:24-25`, `tick.rs:86` (G22); `hooks.rs:716` (G23),
`:549` (G24); `rails.rs:595-596` (G25); `timeline.rs:10` / `:94`
(G26); `interact.rs:365-386` (G27), `:581-600` (G28);
`bonemeal.rs:335` (G29), `:1239/1247` (G32), `:216-243` (G36);
`growth.rs:156` (GROWERS), `:221-225` (poplar), `:237`
(weighted), `:392-393` (remove_sapling), `:443` (tree_can_grow);
`block_breaking.rs:100-101`, `game.rs:4301`, `:5294/5306` (G35);
`game.rs:1562-1573` (fire radius gate); `tick.rs:1539-1544` (max
snow cap); `block_landing.rs:416-441` (farmland trample);
`portal.rs:43-46` (obsidian frame); `classes.rs:155` (Farmland),
`:170-171` (Snowy/SpreadingSnowy), `:204/206`
(NetherFungus/NetherRoots), `:212` (Path), `:220`
(PointedDripstone), `:228` (RedstoneWire). Oracle lines
(`mods/horizon/src/test/java/dev/horizonlab/parity/`): O01
`VanillaCauldronOracle.java:118`; O02
`VanillaPortalOracle.java:180`; O03
`VanillaPickBlockDataOracle.java:57-59`; O04
`VanillaReplaceBlockOracle.java:36`; O05
`VanillaFallingBlockOracle.java:30`; O06
`VanillaDispenserOracle.java:27` (import — now valid), `:212`,
`:235-236`, `:560-561`, `:618-619` (`EntityTypes.PLAYER`); O07
`VanillaSectionMeshOracle.java:84`; O08
`VanillaBreakingModelOracle.java:12`, `:15`; O09
`VanillaFaceSupportOracle.java:45`.

## §8 Verdict

**130 checks: 104 pass, 26 gap. 37 items: 26 behaviour gaps
(10 major, 16 minor) + 7 oracle defects (1 major, 6 minor) +
2 fixed + 2 closed.** Behaviour/non-behaviour split: 130
behaviour + 0 non-behaviour. PRIMARY re-checks 31/31 decided
(27 REPRODUCE, 2 FIXED, 2 NEVER EXISTED); SECONDARY re-checks
22/22 decided (10 REPRODUCE, 10 FIXED, 2 NEVER EXISTED as
stated).

The 1.21.11 template carries the same absent-tag defects as its
first child: six blocks can never survive anywhere (B5-G37–G42),
fungi and roots survive nowhere (B5-G04), dripstone is
half-alive (B5-G13), the composter is dead (B5-G22), snow never
accumulates (B5-G21). But vanilla 1.21.11 changed in B5-relevant
ways, and three 1.21.10 verdicts flip: the fire tick check PASSES
(no `doFireTick` rule exists in 1.21.11; the radius gate the port
implements IS vanilla's form — the 1.21.10 B1 item is WITHDRAWN
for 1.21.11), and two oracles are FIXED (B5-O02 Portal,
B5-O03 PickBlockData). B5-G17 (chorus ceiling) and B5-G18 (bell)
NEVER EXISTED — confirmed fresh against 1.21.11's bytecode —
and land as CLOSED items. No NEW behaviour gaps for 1.21.11: the
three 1.21.10→1.21.11 code diffs are all correct adaptations,
and both new checks pass.

Oracle triage is PROVISIONAL pending Batch 0's measured table
for 1.21.11 (no prepare has landed); each oracle verdict cites
its failing or resolving symbol and jar fact, and a
measured-table disagreement would supersede §4's oracle section.
The behaviour verdicts are not provisional: every one rests on
javap bytecode read fresh from 1.21.11's own named jars plus the
1.21.11 port source.
