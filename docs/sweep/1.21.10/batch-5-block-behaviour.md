# Batch 5 — Block behaviour (1.21.10 parity sweep)

Checks + triage only, against VANILLA 1.21.10's own jar and data (named
server-game jar `~/workspace/mcir-work/e6-deltas/versions/1.21.10/jars/server-game-named.jar`,
sha1 `ba1fc89ead95b8ebdb8727f84e2bd487533c8b7f`, 6,964 classes; named
client jar `.../jars/client-named.jar`, sha1
`715f58165acfaf8b61ab921064f08c202b41bd0c`, 9,969 classes). Static
checks only: javap on the named jars, Rust source comparison, vanilla
jar/data reads. No fixes, no cargo builds, no driver runs, no git
commands in the worktree.

**Headline:** the 1.21.10 tree is a deliberate 1.21.10 adaptation of
the 1.21.11 template — and the adaptation is half done. The port agent
fixed the 1.21.7-era dispatch defects outright (farmland, grass
spread, dirt paths, stems, crops, mushrooms, kelp, fletching table:
eight 1.21.7 items FIXED, plus the nether portal frame and dirt path
from 26.1.2), but left the 26.x tag reads in place: **six blocks can
never survive anywhere** — sugar cane cannot be planted, cocoa,
azalea, mangrove propagule, nether sprouts and nether wart survive on
nothing (B5-G37–G42, all NEW, all MAJOR) — because the port reads
`supports_*` tags 1.21.10 never had, while vanilla 1.21.10 hardcodes
dirt, soul sand, clay, jungle logs and nylium. The same absent-tag
root cause carries the 1.21.7 fungi/roots defect in a worse form
(B5-G04: the class is mapped now, so the dead-tag arm fires and
shadows the fallback — fungi/roots survive nowhere), plus snow layer,
seagrass and cactus flower (B5-G08/G09/G11). Dripstone is half-alive
(B5-G13: dispatch fixed, tag dead). The 26.3-form TreeGrower and
bonemeal gates carry over unchanged (B5-G29/G32–G34), the composter is
still dead (B5-G22), and the client bonemeal prediction is inverted
from the 26.1.2 item: vanilla 1.21.10 returns SUCCESS on the client
for every valid target, and the port passes on saplings, azalea and
mushrooms (B5-G36, NEW). **Two corrections headline the re-checks:
B5-G17 (chorus ceiling) NEVER EXISTED — the 26.1.2 item misread
`getMaxY()` as 320; it is `minY + height - 1` = 319 in both jars,
verified fresh in each — and B5-G18 (bell) NEVER EXISTED again,
confirmed fresh against 1.21.10's bytecode.** Both land as CLOSED
items with the corrections recorded.

## §0 Execution status

Checks + triage only. No file under `versions/1.21.10/` was modified,
nothing was committed, no git commands ran in the worktree, no cargo
builds ran. Everything below is static analysis: `javap -c -p` on the
named 1.21.10 jars (73 dumps), a class-chain census built from the
jar (`Blocks.<clinit>` + BootstrapMethods parse, 1,166 blocks), a tag
inventory built directly from the client jar's
`data/minecraft/tags/block/*.json` (203 tags), source comparison
against `versions/1.21.10/crates/`, and the 1.21.10 datagen reports
(`items.json`, 1,488 items).

**Batch 0 prepare has NOT landed for 1.21.10** (observed 2026-11-24:
`versions/1.21.10/.local/` holds only an empty `minecraft/1.21.10`
dir; no `block-behaviours.json`, no `block-tags.json`, no
`docs/sweep/1.21.10/batch-0-oracles.json`; `b0/evidence/` is empty).
Two consequences, stated where they matter: (1) the tag inventory and
class chains were built from the jars directly, not from prepared
data — the census is fresh and complete, but the export-field checks
(`stemSupportBlocks` etc.) rest on the exporter's source plus the
jar's field shapes, not on a prepared export; (2) **oracle triage is
provisional**: there is no measured Batch 0 oracle table for 1.21.10,
so each oracle verdict rests on the oracle source in the 1.21.10 tree
plus a javap symbol check against the 1.21.10 jars (the failing
symbol, its line, and the jar fact are cited per item). Batch 0's
measured table, when it lands, should confirm these nine; a
measured-table disagreement would supersede this section.

**Tally: 128 behaviour checks — 101 pass, 27 gap** (26.1.2: 128
checks, 103 pass, 25 gap). Behaviour/non-behaviour split: **128
behaviour + 0 non-behaviour**. Batch 5 is not one of the three
as-tallied exception batches; oracle triage is counted separately
below. The tally moves vs 26.1.2 by +2 gaps net: eight check-slots close
(G03 ×4 — ticks, survival, updates, placement — plus G17 ticks,
G20 interaction, the G22 tick check, and the class-chain
structural check) and ten open (nine survival — G04/G41, G08, G09,
G11, G37–G42 — plus the fire/doFireTick tick check, whose item
Batch 1 owns). Interaction gains the bonemeal-growth check's new
G36 item, but the check already gapped.

**Items: 37** — 26 behaviour gaps (10 major, 16 minor) + 9 oracle
defects (1 major, 8 minor) + 2 closed items (B5-G17, B5-G18, both
NEVER EXISTED corrections). Seven items are NEW for 1.21.10
(B5-G36–G42).

**1.21.7 re-checks (PRIMARY): 22/22 decided — 12 REPRODUCE, 8 FIXED,
2 NEVER EXISTED.** **26.1.2 re-checks (SECONDARY): 29/29 decided —
22 REPRODUCE, 2 FIXED, 1 WITHDRAWN, 4 NEVER EXISTED.**

## §1 Scope

Block ticks, interaction, survival/support, updates, breaking/removal,
placement — the general block-behaviour surface: farmland, grass/mycelium
spread, dirt paths, nether fungi/roots, stems/crops, mushrooms, snow
layers, seagrass/kelp, cactus flowers, fletching table, dripstone,
saplings/bonemeal, trees, bells, chorus, portals, pick-block, replace,
plus the composter, ice, tripwire, eyeblossom, beds, dragon eggs and
mining feedback that the 26.1.2 run put in scope, plus the six
newly-defected survival blocks (sugar cane, cocoa, azalea, mangrove
propagule, nether sprouts, nether wart) this run found. Method: javap
on the named 1.21.10 jars + source comparison + vanilla jar/data
reads. No cargo builds, no driver runs.

## §2 Method

Three verification layers, all fresh against 1.21.10 evidence:

1. **Jar identity and census.** The named jars' sha1s match the
   brief (server-game `ba1fc89e…`, client `715f5816…`). The
   class-chain census parses `Blocks.<clinit>` (the first
   `InvokeDynamic #N:apply` after each `ldc "name"` is the block
   factory; later invokedynamics are `Properties` lambdas) and
   resolves the bootstrap entries via `javap -v` BootstrapMethods:
   1,166 blocks, 291 distinct class names, 26 unmapped, 1 plain-
   `Block` fallback (`minecraft:amethyst_block`), 90
   ancestor-recognised. The tag inventory reads the client jar's
   `data/minecraft/tags/block/*.json` directly: 203 tags. Every
   absent/present claim in this report is a jar fact, not a prepared-
   data fact. Key class facts (all javap-verified fresh):
   `FarmBlock` exists (no `FarmlandBlock`); `DirtPathBlock` exists
   (no `PathBlock`); `PointedDripstoneBlock` extends `Block`
   directly (no `SpeleothemBlock`); `FletchingTableBlock` is ABSENT
   (fletching table is plain `Block`); `SpreadingSnowyDirtBlock`/
   `SnowyDirtBlock` exist (no `SpreadingSnowyBlock`/`SnowyBlock`);
   `FungusBlock`/`RootsBlock` exist (no `NetherFungusBlock`/
   `NetherRootsBlock`); `RedStoneWireBlock` (capital S); `TreeGrower`
   at `net.minecraft.world.level.block.grower.TreeGrower`; NO
   `EntityTypes`/`BlockEntityTypes` (holder move absent); NO
   `AbstractBedBlock` (`BedBlock` extends
   `HorizontalDirectionalBlock`); `BambooStalkBlock` (no
   `BambooBlock`); `GameRules` at `net.minecraft.world.level`
   (60 rules, `RULE_DOFIRETICK` present — unlike 26.1.2's 59);
   no `net.minecraft.util.Prediction`; no `BlockEntityData`
   component class; `MaterialBaker` absent from BOTH jars;
   `BlockModelFeatureRenderer` absent; `BlockFeatureRenderer`
   exists (client only); `net/minecraft/util/debug` EXISTS (43
   entries, `ServerDebugSubscribers` in both jars).
2. **Source comparison.** Every defect-bearing method was dumped
   fresh from the 1.21.10 jars (73 dumps, see §7) and compared
   against the 1.21.10 port source (`versions/1.21.10/crates/`).
   Pass checks rest on full method comparison for every
   defect-bearing and re-check method, plus arm-liveness and
   deciding-constant comparison for the generic ticks (nether wart,
   cocoa, sweet berry, bamboo, vine, composter verified in full;
   fire, frosted ice, sniffer/turtle egg, dispenser, observer,
   piston, wire, sponge and the falling blocks verified for dispatch
   liveness and form). The port's `level/` is a deliberate 1.21.10
   adaptation: its comments cite 1.21.10-vs-26.1 differences
   throughout (`survive.rs`, `tick.rs`, `bonemeal.rs`), and a
   `diff -rq` of `level/` against the 1.21.11 template tree shows
   16 differing files — exactly the B5-scope files
   (`bonemeal.rs`, `classes.rs`, `data.rs`, `fluids.rs`,
   `growth_tests.rs`, `interact.rs`, `place.rs`, `rails.rs`,
   `removal.rs`, `sculk.rs`, `shapes.rs`, `speleothem.rs`,
   `survive.rs`, `tests.rs`, `tick.rs`, `timeline.rs`). Provenance:
   1.21.10 derives from the 1.21.11 template with 1.21.10-specific
   adaptations — NOT a re-port from 26.3, and NOT byte-identical to
   any 26.x tree. No pass in this report rests on cross-version
   identity.
3. **Fresh data checks.** The tag inventory (203 tags; absent/present
   re-verified per tag), the stem support fields (read from the
   exporter's source plus the jar's field shapes — `StemBlock` has
   `fruit`/`attachedStem` `ResourceKey` fields; `FungusBlock` has
   only `requiredBlock` + `feature`; `RootsBlock` has NO fields),
   and the `compostable` component census (0/1,488 items in the
   1.21.10 datagen `items.json`) were all run against 1.21.10
   evidence.

**Counting rule:** one check = one vanilla-method/behaviour comparison
(jar + port source, or jar + vanilla data). One item may sit behind
several checks (G13) and one check may carry several items (the
bonemeal-growth check carries G29 + G32 + G36; the fungi/roots
survival check carries G04 + G41; the sapling survival check carries
G40). Oracle/driver-path checks are not in the behaviour tally.

**Note on the 26.1.2 summary's "26.1.1-record corrections" section**
(`sweep-1.21.10-ref/26.1.2/summary.md`, line 430): read and applied.
Only one correction touches Batch 5 scope — B5-G18 (bell), which
this run re-verified fresh against 1.21.10's bytecode and confirms
(see §4). The rest are other batches'. No other B5 verdict changes
from it. The G17 correction in this report is NEW and belongs in the
1.21.10 run's own corrections section (see §6.4).

## §3 Checks table

| Family | Checks | Pass | Gap |
|---|---:|---:|---:|
| Random/scheduled ticks | 37 | 31 | 6 (B5-G13, B5-G21, B5-G23, B5-G26, B5-G33/G34, fire/doFireTick → B1) |
| Survival/support | 28 | 18 | 10 (B5-G04+G41, B5-G08, B5-G09, B5-G11, B5-G13, B5-G37, B5-G38, B5-G39, B5-G40, B5-G42) |
| Updates | 16 | 15 | 1 (B5-G13) |
| Placement | 11 | 10 | 1 (B5-G13) |
| Interaction | 20 | 15 | 5 (bonemeal-growth B5-G29/G32/G36, B5-G19, B5-G22, B5-G27, B5-G28) |
| Breaking/removal | 14 | 11 | 3 (B5-G24, B5-G25, B5-G35) |
| Structural (class chains, tags) | 2 | 1 | 1 (tag inventory) |
| **Total** | **128** | **101** | **27** |

Behaviour checks only; oracle triage is counted separately below. The
structural tag-inventory gap is the systemic root of G04/G08/G09/G11/
G13/G19/G21/G24/G37–G42 — the check is the inventory itself, not a
re-count of the defects. G13 appears in four families because it is a
distinct code path (tick, survive, updateShape, placement) sharing one
root cause. The bonemeal-growth interaction check carries three items
(G29 + G32 + G36); the sapling-advance tick check carries two (G33,
G34); the fungi/roots survival check carries two (G04 + G41); the
sapling survival check carries G40 (the propagule is a `SaplingBlock`
subclass in vanilla). The fire/doFireTick tick gap is owned by Batch 1
(B1-G04, cross-reference — not minted here); the check gaps because
the port's `GameRules` struct has no `do_fire_tick` field at all.
Interaction is 15/5 (26.1.2: 14/6) — G20 closes (portal frame
hardcoded) while the bonemeal-growth check gains G36 (it already
gapped). Survival is 18/10 (26.1.2: 26/2) — the six new absent-tag
defects plus the three 1.21.7 reproductions, against G03 closing.
Structural is 1/1 (26.1.2: 0/2) — the class-chain check PASSES for
B5 scope (every in-scope chain resolves to a live `Class` variant;
the 26 unmapped classes are all out of B5 scope), while the tag
inventory still gaps. Ticks is 31/6 (26.1.2: 29/8) — G03 and G17
close, G22's tick check is a PASS here (see §7: the 26.1.2 table
listed G22 under ticks but its own §7 text calls the composter tick
a pass; the 1.21.10 tick arm matches vanilla — the defect is
insertion-only), and the fire/doFireTick check gaps.

10 gaps are MAJOR (G04, G13, G21, G22, G37–G42), 16 are MINOR (G08,
G09, G11, G19, G23–G29, G32–G36). No B5-H (needs-harness) items:
every check in scope was decided statically (Batch 0's missing
prepare forced jar-direct census work, not harness work).

Oracle defects: 9 (B5-O01–O09) — 8 minor, 1 major (O09,
FaceSupport, blocks 21 drivers + 7 oracle callers). All nine are
provisional on Batch 0's measured table (see §0); each cites its
failing symbol and jar fact. In-scope oracles passing (provisional,
clean of suspect imports): VanillaBlockSurvivalOracle,
VanillaBlockSupportOracle, VanillaBlockPropertiesOracle,
VanillaBlockStateCodecOracle, VanillaCollisionOracle,
VanillaHeightmapOracle.

## §4 Defects

### Reproduced 1.21.7 defects (all verified fresh against 1.21.10's jar/tree)

### B5-G04 — Nether fungi/roots survive nowhere: the dead-tag arm now fires and shadows the fallback (MAJOR, REPRODUCES — worse than 1.21.7)

Vanilla 1.21.10 `FungusBlock.mayPlaceOn`: `#NYLIUM || MYCELIUM ||
SOUL_SOIL || super`; `RootsBlock.mayPlaceOn`: `#NYLIUM || SOUL_SOIL
|| super` (bytecode-verified fresh). The port's
`NetherFungus | NetherRoots` arm (`survive.rs:399-402`) reads
`data.support_tag` — which is `None` for both classes: the exporter
is reflective and dumps `TagKey` fields per block, but
`FungusBlock` has only `requiredBlock` (a `Block`) + `feature` (a
`ResourceKey`) and `RootsBlock` has NO fields at all, so no tag is
ever exported. In 1.21.7 the classes were unmapped, so the arm never
fired and the `Vegetation` fallback (dirt tag / farmland) applied —
wrong supports, but survivable. In 1.21.10 the classes ARE mapped
(`classes.rs` aliases `NetherFungus = "NetherFungusBlock" |
"FungusBlock"`, `NetherRoots = "NetherRootsBlock" | "RootsBlock"`),
so the arm fires and returns false — shadowing the fallback.
Fungi and roots cannot be placed or survive anywhere in 1.21.10.
The port's own comment names the gap ("the fungus/root classes keep
their tag until their own item lands (FungusBlock/RootsBlock, item
757)") — the item never landed. Fix note: hardcode vanilla 1.21.10's
form (`#nylium` / mycelium / soul soil), matching the port's own
hardcoded arms for stems, crops and chorus.

### B5-G08 — Snow layers survive on nothing they should refuse and refuse nothing they should: port reads absent 26.x tags (MINOR, REPRODUCES)

Vanilla 1.21.10 `SnowLayerBlock.canSurvive`: below in
`#SNOW_LAYER_CANNOT_SURVIVE_ON` → false; in
`#SNOW_LAYER_CAN_SURVIVE_ON` → true; else `isFaceFull(collision,
UP)` or 8-layer snow below (bytecode-verified fresh). Both tags are
PRESENT in 1.21.10 (`snow_layer_cannot_survive_on`: ice,
packed_ice, barrier; `snow_layer_can_survive_on`: honey_block,
soul_sand, mud). The port's `SnowLayer` arm (`survive.rs`) reads the
absent 26.x tags `#cannot_support_snow_layer` /
`#support_override_snow_layer` instead — both always empty — so snow
layers survive on ice and packed ice (vanilla refuses) and pop off
honey blocks, soul sand and mud (vanilla allows). Fix note: read the
1.21.10 tags the port already knows exist.

### B5-G09 — Seagrass survives on magma: port reads the absent `#cannot_support_seagrass` tag (MINOR, REPRODUCES)

Vanilla 1.21.10 `SeagrassBlock.mayPlaceOn`: face sturdy UP AND NOT
magma, hardcoded (bytecode-verified fresh). The port's `Seagrass`
arm reads the absent `#cannot_support_seagrass` tag (always empty),
so the magma exclusion never applies — seagrass survives on magma
blocks. (Kelp is FIXED — see §5: the port hardcodes the magma
refusal in `growing_plant_can_attach_to`.)

### B5-G11 — Cactus flowers cannot survive on cacti or farmland (MINOR, REPRODUCES)

Vanilla 1.21.10 `CactusFlowerBlock.mayPlaceOn`: below is CACTUS ||
FARMLAND || `isFaceSturdy(UP, CENTER)` (bytecode-verified fresh).
The port's `CactusFlower` arm (`survive.rs:404-408`) checks the
absent `#support_override_cactus_flower` tag OR face-sturdy-center.
Vanilla's own sturdy data says cactus CENTER_UP is false and
farmland CENTER_UP is false (exporter-computed `isFaceSturdy` per
state/direction/type — the 1.21.7 run's data, consistent with the
fresh 1.21.10 shape bytecode: cactus SHAPE is
`Block.column(14.0, 0.0, 16.0)` and farmland SHAPE tops at 15, and
`SupportType.CENTER` requires covering the `column(2.0, 0.0, 10.0)`
center shape, which neither top face does). So the port's arm
returns false on both intended supports: cactus flowers cannot be
placed on cacti or farmland, and flowers grown by cactus random tick
pop on the next neighbour update. Fix note: hardcode
`CACTUS || FARMLAND || sturdy-center`, matching vanilla.

### B5-G13 — Dripstone half-alive: dispatch fixed, `#speleothems` tag still dead (MAJOR, REPRODUCES — narrower form than 26.1.2)

One cause only in 1.21.10 (26.1.2 had two). **Dispatch is LIVE:**
`PointedDripstoneBlock` is mapped (`classes.rs`), and all dripstone
arms dispatch on `Speleothem | PointedDripstone` — scheduled tick
(`tick.rs:257`, up-tip + !canSurvive → destroy, else
`spawn_falling_stalactite`), random tick (`tick.rs:443`), survive,
updateShape (`shapes.rs:745`), placement (`place.rs:1156`). **The
tag predicates are dead:** `#speleothems` is ABSENT from 1.21.10
data, so `speleothem_tag` (`speleothem.rs:15`), `spawn_falling_stalactite`
(`falling.rs:55`), `speleothem_thickness` (`shapes.rs:1186`),
`speleothem_thickness_for` (`place.rs:1409`) and the growth/fluid
gates all fail. Vanilla 1.21.10 uses hardcoded
`Blocks.POINTED_DRIPSTONE` checks (`PointedDripstoneBlock.randomTick`:
`maybeTransferFluid` + 0.011377778f growth chance via
`isStalactiteStartPos`/`growStalactiteOrStalagmiteIfPossible`;
`canGrow`: DRIPSTONE_BLOCK above-start + WATER block + fluid
`isSource()`). Observable: dripstone never grows, stalactites never
fall, cauldron drip never transfers, thickness is always "tip".
The latent 26.1.2 G16 (water-source check) is FIXED in the dead code
— `speleothem_grow`'s `canGrow` already uses the 1.21.10 form
(water BLOCK two above + `isSource()`). Fix note: replace the
`#speleothems` reads with a hardcoded pointed-dripstone check,
matching vanilla 1.21.10's own form.

### B5-G29 — Bonemeal validity too strict: the `tree_can_grow` gate refuses lone dark/pale oak (MINOR, REPRODUCES — the over-strict direction only)

Vanilla 1.21.10 `SaplingBlock.isValidBonemealTarget` is
unconditionally true (`iconst_1; ireturn` — bytecode-verified fresh;
there is NO `canGrow` method on 1.21.10's `TreeGrower`, 0 hits).
The port's `Sapling` arm in `bonemeal_valid` gates on
`live_features().is_none() || !tree_can_grow(...)` — and
`tree_can_grow` (`growth.rs:425-444`, the 26.3 form) refuses a lone
dark/pale oak sapling (it needs the 2×2 mega arrangement). Bonemeal
is refused on a lone dark/pale oak sapling where vanilla accepts it.
The gate also draws random on every validity check (see B5-G33).
The over-permissive direction (26.1.2's B5-G15) does NOT reproduce
in 1.21.10 — vanilla has no bonemeal height gates here, and the port
has none either (see §5). The port's own tests still assert the wrong
behaviour (`growth_tests.rs`: `lone_dark_oak_sapling_cannot_grow`).

### Reproduced 26.1.2 defects (all verified fresh against 1.21.10's jar/tree)

### B5-G19 — Water bottle on dirt never makes mud: port reads `#convertible_to_mud`, 1.21.10's tag is `#convertable_to_mud` (MINOR, REPRODUCES)

The port reads `#convertible_to_mud` (with 'i') in four places:
`data.rs:458` (tag table), `horizon-client/src/game/potions.rs:710`
(water bottle use), `horizon-client/src/game/remote.rs:8677`,
`horizon-client/src/game/dispenser.rs:2251` (dispenser water
bottle). 1.21.10's tag is `#convertable_to_mud` (with 'a') —
verified three ways: `BlockTags.CONVERTABLE_TO_MUD` in the jar,
`data/minecraft/tags/block/convertable_to_mud.json` in the client
jar (values dirt, coarse_dirt, rooted_dirt), and vanilla
`PotionItem.useOn` checks it (bytecode offset 57, re-verified
fresh). The port's tag read returns nothing, so the mud conversion
never fires. Fix note: rename all four reads to
`convertable_to_mud`.

### B5-G21 — Snow never accumulates on land, ice never forms except on water: `motion_blocking_top` reads the absent `#blocks_motion_in_heightmap` tag (MAJOR, REPRODUCES)

The port's `motion_blocking_top` (`weather.rs:41-59`) reads
`#blocks_motion_in_heightmap`, which is ABSENT from 1.21.10 data —
only fluids count as motion-blocking. Vanilla 1.21.10
`Heightmap.MATERIAL_MOTION_BLOCKING` is a method reference to
`BlockStateBase.blocksMotion()` (bootstrap method 1 in `Heightmap`,
re-verified fresh with `javap -v`). Same-root-cause second site:
`feature_world.rs:350-360` (`OceanFloor`/`MotionBlocking`/
`MotionBlockingNoLeaves` all read the absent
`#blocks_motion_in_heightmap[_no_leaves]` tags) — folded into this
item, not minted separately. Fix note: use the per-state
`blocksMotion()` rule, as the 26.1 tree did.

### B5-G22 — Composter completely dead: port reads the 26.3 `minecraft:compostable` component no 1.21.10 item carries (MAJOR, REPRODUCES)

The port reads the `minecraft:compostable` component
(`horizon-items/src/items.rs:1049`, `game/composter.rs:24-25`).
Vanilla 1.21.10 has NO such component: `DataComponents` has no
`COMPOSTABLE`, and 0/1,488 items in the 1.21.10 datagen `items.json`
carry it. Vanilla 1.21.10 uses the static `ComposterBlock.COMPOSTABLES`
table (`Object2FloatMap`, bytecode-verified fresh). Every composter
path returns `None`: no item can be composted by hand, hopper, or
dispenser. The composter TICK itself works (level 7 → 8 + ready
sound, `tick.rs:86` — matches vanilla's 20-tick schedule at level 7),
so the defect is insertion-only. Fix note: restore the
`compost_chance` table (absent from the tree — no `compost_chance`
anywhere in `crates/`).

### B5-G23 — Ice melt fires a BLOCK_DESTROY game event vanilla 1.21.10 never fires (MINOR, REPRODUCES)

Vanilla 1.21.10 `IceBlock.melt` (bytecode read fresh):
water-evaporates → `removeBlock`, else
`setBlockAndUpdate(water)` + `neighborChanged` — ZERO `gameEvent`
references in the whole class. The port's `ice_melt`
(`hooks.rs:701-717`) adds
`self.game_event_state(pos, "minecraft:block_destroy", state)` after
the `setBlockAndUpdate`. Sculk sensors hear a block-destroy event
for every melting ice block. Fix note: delete the event line (the
event is a 26.3 addition).

### B5-G24 — Ice mined over solid ground leaves no water: `ice_destroyed_by_player` reads the absent `#blocks_motion` tag (MINOR, REPRODUCES)

Vanilla 1.21.10 `IceBlock.playerDestroy`: unless the tool prevents
melting or water evaporates, if the block below `blocksMotion()` OR
is liquid, the position becomes water (`setBlockAndUpdate(meltsInto())`).
The port's `ice_destroyed_by_player` (`hooks.rs:535-553`) tests
`b.in_set(below, &b.tags.blocks_motion) || b.liquid(below)` — the
tag is absent (always empty), so ice mined over solid ground leaves
air instead of water. Fix note: restore the per-state
`blocks_motion()` rule, as the 26.1 tree did.

### B5-G25 — Tripwire schedules a 1-tick re-check when unpressed that vanilla 1.21.10 never schedules (MINOR, REPRODUCES)

Vanilla 1.21.10 schedules a tripwire re-check tick 10 ONLY while
pressed: `TripWireBlock`'s single `scheduleTick` is `bipush 10`
gated on the pressed flag (offsets 124-141, fresh javap), and
`TripWireHookBlock` has exactly ONE `scheduleTick` in the whole
class — the 10-tick attached-circuit re-check inside
`calculateState` (offset 281). There is no 1-tick unpress re-check
anywhere in either class. The port's `tripwire_check_pressed`
(`rails.rs:593-597`) schedules 10 when pressed and 1 `else if was`
— a spurious extra tick (and extra re-check work) every time a
tripwire unpresses. (The AABB heights are CORRECT in 1.21.10:
attached 2.5/16, unattached 8/16, matching vanilla's shape boxes —
the 26.1.2 item was never about the heights.)

### B5-G26 — Eyeblossom neighbour cascade scans in 26.3 section order instead of `betweenClosed` order (MINOR, REPRODUCES)

Vanilla 1.21.10 `EyeblossomBlock.tryChangingState` iterates the
7×5×7 neighbour box (offsets (−3,−2,−3) to (3,2,3)) with
`BlockPos.betweenClosed` (offset 99, fresh javap) — x innermost,
then y, then z — and each matching neighbour draws its delay from
`nextIntBetweenInclusive(dist*5, dist*10)` in that scan order. The
port's `eyeblossom_tick` (`timeline.rs:69-115`) uses
`box_scan_order` (section-major: x-section, z-section, y-section,
then y, z, x within a section — y and z swapped vs vanilla), so
the delay draws land on different neighbours in a different order.
The delay form itself matches. Fix note: restore the
`between_closed` order, as the 26.1 tree did (with its unit test
pinning the order).

### B5-G27 — Bed use requires the other half: a head click with no foot returns Consume instead of sleeping (MINOR, REPRODUCES)

Vanilla 1.21.10 `BedBlock.useWithoutItem` (bytecode read fresh):
client returns `SUCCESS_SERVER`; if part != HEAD, redirect to the
head via facing and require only that the head is this bed (else
CONSUME); a HEAD click needs no foot check at all — it proceeds
straight to the sleep/explode path (offset 57). The port's
`Class::AbstractBed | Class::Bed` arm (`interact.rs:365-386`)
requires the other half for BOTH parts (`b.is_block(other, block)
&& part != part`, else `Consume`) — a head click on a bed whose
foot was broken returns Consume instead of sleeping. (The arm fires
— `Bed = "BedBlock"` is mapped, and 1.21.10 has no
`AbstractBedBlock`; the `AbstractBed` alternative is dead but the
`Bed` arm is live.)

### B5-G28 — Dragon egg teleport ignores the world border (MINOR, REPRODUCES)

Vanilla 1.21.10 `DragonEggBlock.teleport` (bytecode read fresh)
accepts a candidate only when it is air (offsets 98-101),
`WorldBorder.isWithinBounds` (offsets 104-111), and inside build
height (offsets 114-120) — 1,000 attempts, `nextInt(16)`/`nextInt(8)`/
`nextInt(16)` differences. (Note: 1.21.10 has NO "block below is
non-air" check — that part of the 26.1.2 item's evidence is a 26.x
addition, not a 1.21.10 form.) The port's `dragon_egg_teleport`
(`interact.rs:581-600`) checks only air + build height: an egg
near the border can teleport outside it. Fix note: restore the
`within_world_border(test)` candidate test, as the 26.1 tree did.

### B5-G32 — Grass bonemeal rolls use `nextFloat` where vanilla 1.21.10 uses `nextInt` (MINOR, REPRODUCES)

Vanilla 1.21.10 `GrassBlock.performBonemeal` (bytecode read fresh):
the tall-grass roll is `nextInt(10) == 0` (offsets 170-172) and the
flower-feature roll is `nextInt(8) == 0` (offsets 230-232). The
port's `grass_bonemeal_effect` (`bonemeal.rs:1240-1252`) uses
`self.random().next_float() < 0.1` and `< 0.125`. The probabilities
coincide (1/10, 1/8) but the draws differ — the random stream
desynchronises from vanilla after the first grass bonemeal. Fix
note: use `next_int(10)` / `next_int(8)`, as the 26.1 tree did.

### B5-G33 — TreeGrower is the 26.3 weighted-list form; vanilla 1.21.10 is secondary-chance (MINOR, REPRODUCES)

Vanilla 1.21.10 `TreeGrower` (10 growers: oak, spruce, mangrove,
azalea, birch, jungle, acacia, cherry, dark oak, pale oak — NO
poplar) is the secondary-chance form: a `secondaryChance` float plus
`megaTree`/`secondaryMegaTree`/`tree`/`secondaryTree`/`flowers`/
`secondaryFlowers` optionals; `getConfiguredFeature` draws ONE
`nextFloat()` unconditionally; `getConfiguredMegaFeature` draws
`nextFloat` only when a secondary mega exists; there is NO `canGrow`
method (0 hits). The port's `GROWERS` (`growth.rs:145-269`) is the
26.3 weighted-list form (`trees`/`mega_trees`/`flower_trees` +
`weighted()` using `next_int_bound(total)`), with 11 growers
including a dead `poplar` (no `poplar_sapling` in 1.21.10). The
probabilities coincide but the draw type and count differ — the
random stream desynchronises, and `tree_can_grow` draws on every
validity check (see B5-G29). Fix note: port the secondary-chance
form from vanilla 1.21.10's `TreeGrower`.

### B5-G34 — Mega-tree growth leaves water where vanilla 1.21.10 sets air (MINOR, REPRODUCES)

Vanilla 1.21.10 `TreeGrower.growTree`: the MEGA path clears the 2×2
to AIR with flags 260 and restores the PASSED sapling state on
failure with 260; the SINGLE path clears to the fluid's legacy block
(`createLegacyBlock()`) with flags 260 — water IS vanilla 1.21.10's
single-path form (it differs from 1.21.7, where the single path set
AIR). The port's `remove_sapling` (`growth.rs:375-382`) leaves the
fluid's legacy block on BOTH paths — the mega path leaves water
where vanilla sets air. (The single-path flags 818 vs vanilla 260
is a carried nit.) This is also the mega half of 1.21.7's B5-G14
(see §5).

### B5-G35 — Digging broadcasts level events 2019/2020 that vanilla 1.21.10 never sends (MINOR, REPRODUCES)

Vanilla 1.21.10 has NO level events 2019/2020: `LevelEvent`
(`javap -constants`, fresh) ends its 20xx range at 2013
(`PARTICLES_SMASH_ATTACK`). The port's `destroy_progress_event`
(`game/block_breaking.rs:100-101`) sends 2020 every fourth tick
else 2019, handled in `game.rs:4312` and called from
`game.rs:5305/5317`. A 1.21.10 client receiving these events from
the port's server path sees crack effects vanilla never sends.
Fix note: delete the send (26.1 sends no level event while a player
digs).

### NEW 1.21.10 defects (found by this run — no 1.21.7 or 26.1.2 source item)

### B5-G36 — Bonemeal passes on the client for saplings, azalea and mushrooms where vanilla 1.21.10 returns SUCCESS (MINOR, NEW — inverse of 26.1.2's G31)

Vanilla 1.21.10 `BoneMealItem.useOn` (bytecode read fresh): a
`growCrop` success returns `SUCCESS` on BOTH sides (offset 36:
`ifne 59` — the client skips the game event and level event but
still returns `SUCCESS` at offset 59); `growCrop` itself returns
true on the client for ANY valid target (offset 44: not a
`ServerLevel` → skip success/perform/shrink → return true). And
1.21.10's validity is client-valid for all three: `SaplingBlock`
and `MushroomBlock` are unconditionally true, `AzaleaBlock` is
fluid-above-empty only — none checks `ServerLevel`. The port's
`bone_meal_succeeds_on_client` (`bonemeal.rs:216-243`) returns true
for valid non-sapling/azalea/mushroom targets but EXCLUDES those
three kinds, with a comment claiming they are "only a valid target
on a ServerLevel" — the 26.x form, wrong for 1.21.10. The client
predicts PASS (no hand swing) where vanilla predicts SUCCESS.
This is the INVERSE of 26.1.2's B5-G31 (see §5).

### B5-G37 — Sugar cane cannot be planted: ground check reads the absent `#supports_sugar_cane` tag (MAJOR, NEW)

Vanilla 1.21.10 `SugarCaneBlock.canSurvive` (bytecode read fresh):
below is sugar cane itself, OR below is in `#DIRT`/`#SAND` with
adjacent water (horizontal check). The port's `SugarCane` arm
(`survive.rs:336-360`) keeps the self-stack branch but gates the
planting branch on `b.in_set(below_state, &t.supports_sugar_cane)`
— the tag is ABSENT from 1.21.10 data (verified in the tag
inventory), so the branch never runs. Sugar cane can only stack on
existing cane; it can never be planted on dirt or sand. Fix note:
hardcode `#dirt`/`#sand` + adjacent water, matching vanilla
1.21.10's form. (The adjacent-fluid check reads
`supports_sugar_cane_adjacently`, unreached while the ground
branch is dead.)

### B5-G38 — Cocoa can never survive: port reads the absent `#supports_cocoa` tag (MAJOR, NEW)

Vanilla 1.21.10 `CocoaBlock.canSurvive`: the block the cocoa faces
is in `#JUNGLE_LOGS` (bytecode read fresh). The port's `Cocoa` arm
(`survive.rs:150`) tests `b.in_set(rel_state(facing),
&t.supports_cocoa)` — the tag is ABSENT, so the arm always returns
false. Cocoa cannot survive on any log, jungle or otherwise. Fix
note: test `#jungle_logs`, matching vanilla 1.21.10's form.

### B5-G39 — Azalea can never survive: port reads the absent `#supports_azalea` tag (MAJOR, NEW)

Vanilla 1.21.10 `AzaleaBlock.mayPlaceOn`: CLAY or
`VegetationBlock.mayPlaceOn` (`#DIRT` or FARMLAND) (bytecode read
fresh). The port's `Azalea` arm (`survive.rs:403`) tests
`b.in_set(ground, &t.supports_azalea)` — the tag is ABSENT, so the
arm always returns false. Azalea (and flowering azalea) cannot
survive on any ground. Fix note: hardcode clay/dirt/farmland,
matching vanilla 1.21.10's form.

### B5-G40 — Mangrove propagule can never survive, planted or hanging (MAJOR, NEW)

Vanilla 1.21.10: planted — `mayPlaceOn` is
`SaplingBlock.mayPlaceOn` (`#DIRT`/farmland via `VegetationBlock`)
OR CLAY; hanging — `canSurvive` requires the block above to be
MANGROVE_LEAVES (both bytecode-verified fresh). The port reads the
absent `#supports_mangrove_propagule` tag for the planted form
(`survive.rs:430`) and the absent
`#supports_hanging_mangrove_propagule` tag for the hanging form
(`survive.rs:252-260`) — both always false. Neither form can
survive anywhere. Fix note: hardcode both vanilla 1.21.10 forms.

### B5-G41 — Nether sprouts can never survive: port reads the absent `#supports_nether_sprouts` tag (MAJOR, NEW)

Vanilla 1.21.10 `NetherSproutsBlock.mayPlaceOn`: `#NYLIUM` or
SOUL_SOIL (bytecode read fresh). The port's `NetherSprouts` arm
(`survive.rs:432`) tests `b.in_set(ground,
&t.supports_nether_sprouts)` — the tag is ABSENT, so the arm
always returns false. Nether sprouts cannot survive on nylium or
soul soil. Fix note: hardcode `#nylium`/soul soil, matching vanilla
1.21.10's form (and B5-G04's fix).

### B5-G42 — Nether wart can never survive: port reads the absent `#supports_nether_wart` tag (MAJOR, NEW)

Vanilla 1.21.10 `NetherWartBlock.mayPlaceOn`: SOUL_SAND, hardcoded
(bytecode read fresh). The port's `NetherWart` arm
(`survive.rs:433`) tests `b.in_set(ground, &t.supports_nether_wart)`
— the tag is ABSENT, so the arm always returns false. Nether wart
cannot survive on soul sand (or anywhere). Fix note: hardcode soul
sand, matching vanilla 1.21.10's form.

### Closed items (NEVER EXISTED — corrections, not fixes)

### B5-G17 — Chorus flower at the build ceiling (CLOSED, NEVER EXISTED — corrects the 26.1.2 ledger)

The 26.1.2 item claimed vanilla `ChorusFlowerBlock.randomTick`
proceeds when `above.y <= getMaxY()` (320) while the port returns
early at `above.y >= 320`. Fresh evidence in BOTH jars refutes the
premise: `LevelHeightAccessor.getMaxY()` is `getMinY() + getHeight()
- 1` — bytecode-verified fresh in the 1.21.10 jar AND in the 26.1.2
jar (`~/workspace/mcir-work/26.1.x/versions/26.1.2/jars/server-game.jar`)
— i.e. 319 for the overworld in both versions. Vanilla proceeds
when `above.y <= 319`; the port (`tick.rs:1441`,
`above_pos.y > max_y() - 1` with `max_y()` = 320 exclusive) returns
early when `above.y >= 320`. The two guards are IDENTICAL. The
chorus check is a PASS in 1.21.10, and the 26.1.2 ledger's B5-G17
was a misread of `getMaxY()` — it never existed in 26.1.2 either.
The 26.1.2 (and 26.1.1/26.1, which carried the same item) ledgers
need this correction. Lesson for future runs: `getMaxY()` is the
highest BUILDABLE y (319), not the exclusive bound (320) — verify
the accessor, don't assume the constant.

### B5-G18 — Bell `isProperHit` (CLOSED, NEVER EXISTED — confirms the 26.1.2 correction)

Fresh 1.21.10 bytecode, read in full: `BellBlock.isProperHit` is
`dload_3; ldc2_w 0.8123999834060669d; dcmpl; ifle 20; 18: iconst_0;
ireturn; 20: <attachment switch>` — vanilla requires `clickY <=
0.8124`, exactly what the port implements (`redstone.rs:1442`:
`click_y <= f64::from(0.8124f32)`, with the axis + attachment
arms matching). The port was correct all along; the 26.2 run's
`ifle` misread (corrected by the 26.1.2 run) does not recur here.
The bell check is a PASS.

## Oracle defects

Triage is PROVISIONAL (see §0): no measured Batch 0 oracle table
exists for 1.21.10. Each defect below cites the oracle source line
in `mods/horizon/src/test/java/dev/horizonlab/parity/` of the
1.21.10 tree and the javap symbol check against the 1.21.10 jars.
Full `Vanilla*Oracle` names throughout. Batch 0's measured table,
when it lands, should confirm these nine; a measured-table
disagreement supersedes this section.

### B5-O01 — `VanillaCauldronOracle` fails: `Player.drop` 3-arg override uses `net.minecraft.util.Prediction`, absent in 1.21.10 (ORACLE DEFECT, MINOR, REPRODUCES — 1.21.7's O01 cause)

The oracle still overrides `Player.drop` with the 3-arg signature
(`VanillaCauldronOracle.java:118`: `drop(ItemStack, boolean,
net.minecraft.util.Prediction)`). The 1.21.10 jar has NO
`net.minecraft.util.Prediction` (verified: 0 entries in both jars)
and `Player` has only `drop(ItemStack, boolean)`. One-symbol
repair: drop the third parameter. (The 26.1.2 O01 cause — the
`LayeredCauldronBlock.LEVEL` field form — does NOT apply: the
1.21.10 oracle source never reads `LEVEL`. The oracle fails at the
same line 118, for the 1.21.7 cause. See §5.)

### B5-O02 — `VanillaPortalOracle` fails: `Entity.getRelativePortalPosition` does not exist in 1.21.10 (ORACLE DEFECT, MINOR, REPRODUCES — 1.21.7's O02 cause)

The oracle still calls `entity.getRelativePortalPosition(axis,
area)` (`VanillaPortalOracle.java:180`). The 1.21.10 `Entity` class
has no such method (javap-verified fresh: only `portalCooldown` /
`portalProcess` accessors). This is an INDEPENDENT defect in
1.21.10 — unlike 26.1.2, where the method exists and the oracle
failed only via the `VanillaBlockUpdateOracle` cascade. One-symbol
repair: inline the 1.21.10 portal-position computation. (The
cascade also applies here — `VanillaBlockUpdateOracle` imports the
non-existent `gamerules` package — but O02 fails on its own symbol
regardless.)

### B5-O03 — `VanillaPickBlockDataOracle` fails: `TypedEntityData` has no `loadInto` (ORACLE DEFECT, MINOR, REPRODUCES — 1.21.7's O03 cause, refined)

The oracle calls `data.type()` and `data.loadInto(entity, access)`
on `DataComponents.BLOCK_ENTITY_DATA`
(`VanillaPickBlockDataOracle.java:58`). In 1.21.10,
`BLOCK_ENTITY_DATA` is `DataComponentType<TypedEntityData<
BlockEntityType<?>>>` — the `BlockEntityData` component class does
not exist in the 1.21.10 jar at all, and `TypedEntityData` HAS
`type()` but has NO `loadInto` method (javap-verified fresh:
`codec`, `streamCodec`, `of`, `type`, `contains`, `equals`,
`hashCode` only). The oracle fails on `loadInto`.
(`BlockEntityType.onlyOpCanSetNbt()` DOES exist in 1.21.10, so that
half of the call resolves.) Unlike 26.1.2, where this oracle PASSED
(26.1.2 has `BlockEntityData`), it fails in 1.21.10. Repair: use
the 1.21.10 `TypedEntityData` form (read the tag, apply via the
block-entity's own load path).

### B5-O04 — `VanillaReplaceBlockOracle` fails: `Feature.DIRECT_CODEC` does not exist in 1.21.10 (ORACLE DEFECT, MINOR, REPRODUCES — 1.21.7's O04 cause)

The oracle still reads `Feature.DIRECT_CODEC`
(`VanillaReplaceBlockOracle.java:36`). The 1.21.10 `Feature` class
has NO `DIRECT_CODEC` (javap-verified fresh: only the
`configuredCodec()` instance method); `ConfiguredFeature` HAS
`DIRECT_CODEC`. Same defect, same cause as 1.21.7's O04 and
26.1.2's O04. Fix note: use `ConfiguredFeature.DIRECT_CODEC`,
the 26.1 form.

### B5-O05 — `VanillaFallingBlockOracle` fails: `ChunkMap` constructor mismatch at line 30 (ORACLE DEFECT, MINOR, REPRODUCES — 26.1.2's O05 cause)

`NoopChunkMap extends ChunkMap` with a 12-arg super call
(`VanillaFallingBlockOracle.java:30`: 10 nulls + `0` + `false`).
The 1.21.10 `ChunkMap` has ONE constructor, with 13 parameters
(`ServerLevel, LevelStorageAccess, DataFixer,
StructureTemplateManager, Executor, BlockableEventLoop,
LightChunkGetter, ChunkGenerator, ChunkStatusUpdateListener,
Supplier, TicketStorage, int, boolean` — javap-verified fresh).
The super call does not match. (The 1.21.7 O05 cause — the missing
`net.minecraft.util.debug` package — does NOT apply: the package
EXISTS in 1.21.10 (43 entries), `ServerDebugSubscribers` is in
both jars, and `MinecraftServer` has the `debugSubscribers` field
and accessor. The oracle's debug-subscriber use at line 40
resolves; the constructor is the failure. See §5.)

### B5-O06 — `VanillaDispenserOracle` fails: non-existent `gamerules` package AND missing `EntityTypes` (ORACLE DEFECT, MINOR, REPRODUCES — both the 1.21.7 and the 26.1.2 causes)

Two independent failures in one oracle. (a) It imports
`net.minecraft.world.level.gamerules.GameRules`
(`VanillaDispenserOracle.java:27`); the package does not exist in
1.21.10 — `GameRules` lives in `net.minecraft.world.level`
(the 1.21.7 O06 cause). (b) It references `EntityTypes.PLAYER` at
lines 212, 235-236, 560-561 (via the wildcard
`net.minecraft.world.entity.*` import); `EntityTypes` does not
exist in 1.21.10 — registration still lives on `EntityType`
itself, the holder move happened after 1.21.10 (the 26.1.2 O06
cause). Either alone fails the oracle. Repair: re-point the
`GameRules` import and replace `EntityTypes.PLAYER` with the
1.21.10 `EntityType` form.

### B5-O07 — `VanillaSectionMeshOracle` fails: `MaterialBaker` absent from both jars (ORACLE DEFECT, MINOR, REPRODUCES — stronger form than 26.1.2's O07)

`Materials extends MaterialBaker`
(`VanillaSectionMeshOracle.java:84`). In 26.1.2, `MaterialBaker`
was a public interface ("no interface expected here"); in 1.21.10
it is ABSENT from both jars (0 entries, verified fresh in the
server-game and client jars). The oracle fails at the same line
for the same symbol, in the stronger form. (The 1.21.7 O07 cause —
`blaze3d` absent from both classpaths — does NOT apply: the
1.21.10 client jar HAS `com.mojang.blaze3d` (36 platform + 28
vertex entries). See §5.)

### B5-O08 — `VanillaBreakingModelOracle` fails: `BlockModelFeatureRenderer` absent (ORACLE DEFECT, MINOR, REPRODUCES — 26.1.2's O08 cause)

The oracle imports `BlockModelFeatureRenderer`
(`VanillaBreakingModelOracle.java:12`). The class does not exist
in 1.21.10 (absent from the client jar, verified fresh — 0
entries). (Note: the related `BlockFeatureRenderer` DOES exist, at
`net.minecraft.client.renderer.feature.BlockFeatureRenderer` — the
oracle needs the model variant, which does not.) The oracle ALSO
imports `MaterialBaker` (line 15, absent — see B5-O07), a second
independent failure. (The 1.21.7 O07 `blaze3d` cause does not
apply — see B5-O07.)

### B5-O09 — `VanillaFaceSupportOracle` fails: protected `CubeVoxelShape` constructor at line 45 (ORACLE DEFECT, MAJOR, REPRODUCES — 26.1.2's O09 cause)

The oracle constructs `CubeVoxelShape` directly
(`VanillaFaceSupportOracle.java:45`: `new CubeVoxelShape(bits)`).
The 1.21.10 constructor is PROTECTED (`protected
CubeVoxelShape(DiscreteVoxelShape)` — javap-verified fresh).
MAJOR because it blocks 21 driver files in `tools/` plus 7 oracle
callers in the 1.21.10 tree (40 files reference the oracle
overall). (Batch 2 also records it; the coordinator
deduplicates.)

### Oracle non-items

- **1.21.7 B5-O05 (debug package) — cause evaporates; oracle still
  fails on the 26.1.2 cause.** The `net.minecraft.util.debug`
  package EXISTS in 1.21.10 — the 1.21.7 item's premise fails.
  No separate item; the oracle's failure is B5-O05 (ChunkMap).
- **1.21.7 B5-O07 (`blaze3d`) — cause evaporates; both oracles
  still fail on the 26.1.2 causes.** The 1.21.10 client jar HAS
  `blaze3d` — the 1.21.7 item's premise fails (the 1.21.7 Batch 0
  measurement had no named client jar; 1.21.10 does). No separate
  item; the failures are B5-O07 (`MaterialBaker`) + B5-O08
  (`BlockModelFeatureRenderer`).
- **B5-O02 cascade note:** `VanillaBlockUpdateOracle` (Batch 2's
  scope) also fails in 1.21.10 on the `gamerules` import — but
  B5-O02 is independently defective here (see above), unlike
  26.1.2 where it was cascade-only.

**In-scope oracles passing (provisional):**
VanillaBlockSurvivalOracle, VanillaBlockSupportOracle,
VanillaBlockPropertiesOracle, VanillaBlockStateCodecOracle,
VanillaCollisionOracle, VanillaHeightmapOracle — all six are clean
of the suspect imports (`gamerules`, `EntityTypes`,
`BlockEntityTypes`, `Prediction`, `util.debug`, `blaze3d`,
`MaterialBaker`, `BlockModelFeatureRenderer`, `Feature.DIRECT_CODEC`)
and their key symbols (`SupportType`, `Heightmap`,
`mayPlaceOn`/`canSurvive`) verify against the 1.21.10 jars.
VanillaPickBlockDataOracle does NOT pass here (B5-O03) — unlike
26.1.2, where it passed.

## §5 Re-check verdicts

### 1.21.7 re-checks (PRIMARY, 22/22 decided — 12 REPRODUCE, 8 FIXED, 2 NEVER EXISTED)

| 1.21.7 item | 1.21.10 verdict | 1.21.10 item |
|---|---|---|
| B5-G01 farmland | FIXED (dispatch mapped; tick/survive/updateShape/placement arms live and match vanilla; trample verified in full in `block_landing.rs`: server-level + `random < distance - 0.5` + living + mobGriefing/spawn-protection + `width²·height > 0.512`) | — |
| B5-G02 grass/mycelium | FIXED (aliases mapped; tick + updateShape + placement all live and match vanilla `SpreadingSnowyDirtBlock`) | — |
| B5-G03 dirt path | FIXED (alias mapped; all four arms live; schedule delay 1 verified in `DirtPathBlock.updateShape`) | — |
| B5-G04 fungi/roots | REPRODUCES (worse: class mapped now, so the dead-tag arm fires and shadows the Vegetation fallback — survive nowhere) | B5-G04 |
| B5-G05 stems | FIXED (planting hardcoded farmland; fruiting hardcoded dirt/farmland + `link`/`link2` present via the exporter's `fruit`/`attachedStem` fields) | — |
| B5-G06 crops | FIXED (hardcoded farmland + moisture growth speed matches `CropBlock.getGrowthSpeed`) | — |
| B5-G07 mushrooms | FIXED (uses `mushroom_grow_block`, present in 1.21.10; `mayPlaceOn` is `isSolidRender`, matching the port) | — |
| B5-G08 snow layer | REPRODUCES (port reads absent 26.x tags; vanilla uses present `snow_layer_cannot_survive_on`/`snow_layer_can_survive_on`) | B5-G08 |
| B5-G09 seagrass | REPRODUCES (absent `cannot_support_seagrass`; vanilla hardcodes magma exclusion) | B5-G09 |
| B5-G10 kelp | FIXED (hardcoded magma refusal in `growing_plant_can_attach_to`) | — |
| B5-G11 cactus flower | REPRODUCES (absent `support_override_cactus_flower`; vanilla CACTUS\|\|FARMLAND\|\|sturdy-center; cactus/farmland CENTER_UP false per vanilla sturdy data + fresh 1.21.10 shape bytecode) | B5-G11 |
| B5-G12 fletching table | FIXED (no `FletchingTableBlock` in 1.21.10 — chain is `['Block']`; no `FletchingTable` variant in `classes.rs`; zero "fletching" mentions in the level/game code; no menu arm fires — vanilla PASS matches) | — |
| B5-G13 dripstone | REPRODUCES (narrower: dispatch live, tag dead; latent G16 fixed in the dead code) | B5-G13 |
| B5-G14 tree water | SPLIT — single path EVAPORATES, mega path REPRODUCES | B5-G34 (mega) |
| B5-G15 bonemeal validity | REPRODUCES (over-strict direction only: the `tree_can_grow` gate; the over-permissive direction evaporates — no height gates in either vanilla 1.21.10 or the port) | B5-G29 |
| B5-O01 Cauldron | REPRODUCES (3-arg `drop` override; no `Prediction` in 1.21.10) | B5-O01 |
| B5-O02 Portal | REPRODUCES (`getRelativePortalPosition` absent in 1.21.10; independent defect, not just a cascade) | B5-O02 |
| B5-O03 PickBlockData | REPRODUCES (`TypedEntityData` has `type()` but no `loadInto`; no `BlockEntityData` class) | B5-O03 |
| B5-O04 ReplaceBlock | REPRODUCES (`Feature` has no `DIRECT_CODEC`; `ConfiguredFeature` has it) | B5-O04 |
| B5-O05 FallingBlock | NEVER EXISTED (`net.minecraft.util.debug` EXISTS in 1.21.10; oracle still fails on the ChunkMap constructor — see B5-O05) | — |
| B5-O06 Dispenser | REPRODUCES (`gamerules` package absent; `GameRules` in `net.minecraft.world.level`) | B5-O06 |
| B5-O07 SectionMesh/BreakingModel | NEVER EXISTED (`blaze3d` EXISTS in the 1.21.10 client jar; both oracles still fail on `MaterialBaker`/`BlockModelFeatureRenderer` — see B5-O07/O08) | — |

### 26.1.2 re-checks (SECONDARY, 29/29 decided — 22 REPRODUCE, 2 FIXED, 1 WITHDRAWN, 4 NEVER EXISTED)

| 26.1.2 item | 1.21.10 verdict | 1.21.10 item |
|---|---|---|
| B5-G03 dirt path | FIXED (`DirtPathBlock` alias present in `classes.rs`) | — |
| B5-G13 dripstone | REPRODUCES (tag-only cause; dispatch live) | B5-G13 |
| B5-G15 bonemeal ceiling | WITHDRAWN (vanilla 1.21.10 has no bonemeal height gates and no `getMinimumHeight` — the 26.x API the item is premised on does not exist in 1.21.10; the port has no gates either) | — |
| B5-G17 chorus ceiling | NEVER EXISTED (`getMaxY()` = 319 in BOTH jars — the 26.1.2 item misread it as 320; the port's guard is identical to vanilla's) | B5-G17 (closed) |
| B5-G18 bell inverted | NEVER EXISTED (fresh 1.21.10 bytecode: vanilla requires `clickY <= 0.8124`; port matches exactly) | B5-G18 (closed) |
| B5-G19 mud tag | REPRODUCES (all four reads still 'i' spelling; 1.21.10 tag is 'a') | B5-G19 |
| B5-G20 portal frame | FIXED (`portal_frame` hardcodes obsidian, with a comment citing the absent tag) | — |
| B5-G21 motion_blocking_top | REPRODUCES (absent tag; + `feature_world` site) | B5-G21 |
| B5-G22 composter | REPRODUCES (0/1,488 items carry the component; `compost_chance` absent) | B5-G22 |
| B5-G23 ice melt event | REPRODUCES (event line at `hooks.rs:716`) | B5-G23 |
| B5-G24 ice mined | REPRODUCES (`#blocks_motion` absent) | B5-G24 |
| B5-G25 tripwire re-check | REPRODUCES (`else if was` arm at `rails.rs:595-596`) | B5-G25 |
| B5-G26 eyeblossom order | REPRODUCES (`box_scan_order` at `timeline.rs:10-24`) | B5-G26 |
| B5-G27 bed pair check | REPRODUCES (`interact.rs:365-386`) | B5-G27 |
| B5-G28 dragon egg border | REPRODUCES (no border check in `dragon_egg_teleport`) | B5-G28 |
| B5-G29 canGrow gate | REPRODUCES (`bonemeal.rs` Sapling arm + `growth.rs:425-444`) | B5-G29 |
| B5-G30 sweet berry gates | NEVER EXISTED (vanilla 1.21.10 is age-only — the air-above + build-height checks do not exist in 1.21.10; the port's age-only arm matches) | — |
| B5-G31 client bonemeal | NEVER EXISTED as stated (vanilla 1.21.10 returns SUCCESS on the client for `growCrop` too — the premise fails); the INVERSE defect is new | B5-G36 (new) |
| B5-G32 grass RNG | REPRODUCES (`next_float` at `bonemeal.rs:1244/1252`) | B5-G32 |
| B5-G33 TreeGrower | REPRODUCES (weighted form at `growth.rs:145-269`) | B5-G33 |
| B5-G34 mega water | REPRODUCES (`remove_sapling` at `growth.rs:375-382`) | B5-G34 |
| B5-G35 digging events | REPRODUCES (`block_breaking.rs:100-101` + `game.rs:5305/5317`) | B5-G35 |
| B5-O01 Cauldron | REPRODUCES (same line 118, same missing-symbol failure — but the cause is the `Prediction` symbol, not the `LEVEL` form; the 1.21.10 oracle never reads `LEVEL`) | B5-O01 |
| B5-O04 ReplaceBlock | REPRODUCES (line 36; `Feature` has no `DIRECT_CODEC`) | B5-O04 |
| B5-O05 FallingBlock | REPRODUCES (line 30; 12-arg super vs 13-param constructor) | B5-O05 |
| B5-O06 Dispenser | REPRODUCES (line 212; `EntityTypes` absent — plus the `gamerules` import, a second cause) | B5-O06 |
| B5-O07 SectionMesh | REPRODUCES (line 84; `MaterialBaker` ABSENT in 1.21.10, not an interface — stronger form) | B5-O07 |
| B5-O08 BreakingModel | REPRODUCES (line 12; `BlockModelFeatureRenderer` absent) | B5-O08 |
| B5-O09 FaceSupport | REPRODUCES (line 45; `CubeVoxelShape` constructor PROTECTED) | B5-O09 |

Closed items are minted only for CORRECTIONS of the source ledgers
(G17 corrects the 26.1.2/26.1.1/26.1 ledgers; G18 confirms the
26.1.2 correction against 1.21.10). Version-difference verdicts
(G30, G31, O05, O07 from 1.21.7) are verdicts, not items — vanilla
changed between versions; nothing was misread.

## §6 Systemic findings

### 1. The 1.21.10 adaptation fixed dispatch and left the tags (provenance confirmed, not assumed)

The 1.21.10 tree derives from the 1.21.11 template with deliberate
1.21.10 adaptations (`level/` differs from 1.21.11's in exactly 16
files — the B5-scope files). The adaptation's pattern is visible in
the verdicts: every 1.21.7 defect whose root cause was a CLASS NAME
is FIXED (the port agent added the 1.21.10 aliases —
`FarmBlock`, `DirtPathBlock`, `SpreadingSnowyDirtBlock`,
`SnowyDirtBlock`, `FungusBlock`, `RootsBlock`, `RedStoneWireBlock`
— and hardcoded the vanilla forms for stems, crops, kelp and the
portal frame), while every defect whose root cause was a TAG read
survives or worsens (G04, G08, G09, G11, G13, G19, G21, G24, and
the six new G37–G42). The agent fixed what the class census showed
and never ran the tag inventory. The 1.21.11 template tree is the
repair reference for the tag half: each fix note says "hardcode
vanilla 1.21.10's form", and the port's own hardcoded arms (stems,
crops, chorus, kelp) show the pattern already established in-tree.

### 2. The absent-tag root cause is unchanged — and still silent

The 1.21.10 tree reads the same absent tags as the 26.x trees, plus
the whole `supports_*` family 1.21.10 never had: `#speleothems`
(G13), `#blocks_motion_in_heightmap` /
`#blocks_motion_in_heightmap_no_leaves` (G21, incl. the
`feature_world` site), `#convertible_to_mud` (G19, wrong spelling —
the 'a' form exists), `#blocks_motion` (G24),
`#cannot_support_snow_layer` / `#support_override_snow_layer`
(G08), `#cannot_support_seagrass` (G09),
`#support_override_cactus_flower` (G11), `#supports_sugar_cane`
(G37), `#supports_cocoa` (G38), `#supports_azalea` (G39),
`#supports_mangrove_propagule` /
`#supports_hanging_mangrove_propagule` (G40),
`#supports_nether_sprouts` (G41), `#supports_nether_wart` (G42),
plus `#blocks_fluid_flow`, `#washed_away_by_fluids`,
`#blocks_lava_fire_spread`, `#sculk_growth_inhibitors`,
`#douses_campfires`, `#grows_crops`,
`#overrides_mushroom_light_requirement`,
`#causes_suffocation`, `#suppresses_bounce`,
`#prevents_nearby_leaf_decay` (other batches' scope — verified
absent in the 203-tag inventory). The tag-table construction
resolves absent tags to EMPTY sets (`data.rs` `.unwrap_or_default()`),
and `tag()` returns `Option`, so every absent-tag read silently
degrades to "false" — never an error, never a log line. The
leakage audit exists for exactly this pattern; these reads should
be audited against the 1.21.10 tag inventory (203 tags) rather
than the 26.3 one. Note the asymmetry the 1.21.7 tree avoided:
1.21.7 SYNTHESIZED the `supports_*` tags; 1.21.10 does not (no
`synthes*` anywhere in `horizon-prepare`/`horizon-world` level
code) — the synthesis was dropped in the template re-base.

### 3. Two whole-block regressions, not one

In 26.1.2 the composter (G22) was the only whole-block regression.
In 1.21.10 there are seven: the composter plus the six survival
blocks (G37–G42), each of which cannot be placed or survive
anywhere. G04 (fungi/roots) makes eight blocks unplaceable in
total. These are the batch's headline because they are not
edge cases — a player in 1.21.10 cannot plant sugar cane, grow
cocoa, place azalea, plant a propagule, grow nether wart or place
nether sprouts, at all.

### 4. NEW corrections (for the 1.21.10 summary's corrections section)

**B5-G17 (chorus ceiling) never existed — in 1.21.10 or in 26.1.2.**
The 26.1.2 run claimed vanilla proceeds when `above.y <=
getMaxY()` (320); `getMaxY()` is `minY + height - 1` = 319 in both
jars, verified fresh in each. The 26.1.2, 26.1.1 and 26.1 ledgers
carry G17 as a live minor gap and all three need this correction.
**B5-G18 (bell) never existed — confirmed for 1.21.10.** The
26.1.2 run's correction holds against 1.21.10's bytecode too.
Lesson for future runs: when a report's claim turns on a constant
or a branch direction, verify the accessor/branch fresh in the
version's own jar — don't carry the constant forward from another
version's report.

### Cross-referenced gaps (not re-minted)

- **Fire spread ignoring `doFireTick`** — Batch 1's G4, LIVE in
  1.21.10 (it evaporated for 26.1.2 only because 26.1.2 has no
  `doFireTick` rule). Vanilla 1.21.10 HAS `RULE_DOFIRETICK` (60
  rules; `FireBlock.tick` reads it at offsets 14-20) and the
  port's `GameRules` struct has no `do_fire_tick` field at all —
  `fire_tick` (`tick.rs:629`) never consults it. The fire tick
  check gaps in this batch's tally; the ITEM belongs to Batch 1.
- **Sculk inhibitors in `level/sculk.rs`** — reads the absent
  `#sculk_growth_inhibitors` tag. Batch 6 minted the worldgen
  instance and routed the level instance to B4; recorded here only
  as a cross-reference, NOT minted.
- **`update.rs` `ensure_block_entity` re-check** — B4's scope.
- **Campfire douse** (`game/campfire_douse.rs` reads the absent
  `#douses_campfires` tag) — Batch 7's scope.
- **`can_hold_any_fluid` / lava-fire walk** — Batch 1 minted
  (B1-G05, B1-G06). Sponge does NOT use `can_hold_any_fluid`
  (`hooks.rs` `sponge_remove_water` uses `is_water_fluid` /
  `pickup_block` / Liquid checks) and stays a pass.
- **Diode guard** — B2-G07. **Redstone wire unmapped** — B2-G01
  (note: 1.21.10's wire class is `RedStoneWireBlock`, capital S,
  and the port's alias covers it — B2 decides).
- **B3-G22 startup blocker** (`#causes_suffocation` /
  `#suppresses_bounce`) — Batch 3's. Both tags verified absent
  from the 1.21.10 inventory (fresh census).
- **Dripstone cauldron fill** — Batch 1's G6. **Loot tables behind
  `drop_resources`** — Batch 3's. **`TreeFeatureOracle`** — Batch
  6's. **`CushionPlacementOracle`** — entity-decoration scope.
- **`blocks_fluid_flow` reads** — only in render assets/mesher and
  arrows/entities fluids, never in `level/`; B1/render scope.
- **`feature_world` heightmap tags** — folded into G21's evidence
  as an additional site of the same root cause (same absent tag,
  same fix); NOT minted separately.
- **The twelve 26.1 regressions** — owned by Batches 1, 3, 4, 7
  and 9; not re-decided here.
- **Saved-palette legacy form** (`chunk.rs` `from_palette_nbt`):
  the 26.1.2 summary asked 1.21.10's Batch 11 to check whether its
  chunk reader handles vanilla 1.21.10's `Name`/`Properties`
  palettes — save-loading scope, not block behaviour; flagged
  here, not minted.

## §7 Passes by area, pilot leftovers, drivers, evidence

### Passes — ticks (31/37)

Fire form (the `doFireTick` gate is the gap, owned by B1 — the
reschedule `30 + rand(10)`, `can_survive` removal, infiniburn and
rain checks match), cactus, sugar cane (tick form — the PLANTING
is G37), bamboo (sapling `rand_int(3)` + light ≥ 9; stalk stage
check), vine (`spread_vines` rule + `rand_int(4)` + direction
`rand_int(6)`), kelp, seagrass (tick form — the SURVIVAL is G09),
turtle egg, sniffer egg, cauldron (tick), frosted ice (melt +
`Mth.nextInt(20, 40)` reschedule form), concrete powder, sand,
gravel, anvil, dragon egg (falling), chorus plant, nether wart
(age < 3 + `nextInt(10)`), cocoa (age < 2 + `nextInt(5)`), sweet
berry (age < 3 + `nextInt(5)` + light ≥ 9), pumpkin/melon stem,
crop (growth-speed form verified against
`CropBlock.getGrowthSpeed`), farmland (9×2×9 water search, rain,
moisture 7/−1, revert at 0 when above not in
`#maintains_farmland` — all verified), grass/mycelium spread
(`canBeGrass` → base, light ≥ 9, 4 attempts, `SNOWY` = above in
`#snow` — all verified), mushroom spread, snow layer (survival
tick form — the SURVIVAL is G08), ice (freeze tick), bell
(powered tick), dispenser (scheduled), observer (scheduled),
redstone wire (scheduled — see B2-G01 for the wire class), piston
(scheduled), sponge (placement, not tick), bed (no tick),
composter tick (level 7 → 8 + ready sound — the INSERTION is
G22), dirt path revert tick (G03 FIXED — unconditional
`turn_to_base_block`), chorus flower tick (G17 CLOSED — the
ceiling guard matches vanilla exactly). The six gaps are G13
(dripstone), G21 (precipitation), G23 (ice melt event), G26
(eyeblossom), G33/G34 (sapling advance) and fire/`doFireTick`
(B1's item).

### Passes — survival/support (18/28)

Farmland, grass/mycelium, dirt path (survival + revert — G03
FIXED), stems, crops, mushrooms (survive via
`mushroom_grow_block`; placement via `isSolidRender` — both
match), kelp (hardcoded magma refusal), chorus plant (hardcoded
end stone), chorus flower (hardcoded end stone), bell, sapling
(the SAPLING itself — the check gaps on the propagule, G40),
bamboo, vine, sweet berry, turtle egg, sniffer egg, bed (survival
form — the USE is G27), dry vegetation (`dry_vegetation_may_place_on`
tag PRESENT and read), lily pad (water/ice/frosted-ice form
matches), waterlily-adjacent. The ten gaps are G04+G41
(fungi/roots/sprouts), G08 (snow layer), G09 (seagrass), G11
(cactus flower), G13 (dripstone), G37 (sugar cane), G38 (cocoa),
G39 (azalea), G40 (mangrove propagule, carried on the sapling
check), G42 (nether wart).

### Passes — updates (15/16)

Neighbour-change propagation, `updateShape` for every in-scope
block (farmland/path UP + !canSurvive → schedule tick 1, verified
against `FarmBlock.updateShape`/`DirtPathBlock.updateShape`;
snowy UP → `SNOWY` = above in `#snow`, verified against
`SnowyDirtBlock.updateShape`), redstone power updates (see B2),
the `ensure_block_entity` re-check is B4's scope (see
Cross-references). The one gap is G13 (dripstone updateShape arm
dead on the tag).

### Passes — placement (10/11)

Block placement, waterlogging, the `canSurvive` gate at
placement, face-attached placement, the `getStateForPlacement`
forms for every in-scope block (snowy placement, dripstone
placement form — the THICKNESS is G13). The one gap is G13
(dripstone thickness always "tip" on the dead tag). Note:
placement of the G04/G37–G42 blocks fails via the survival gate,
counted in Survival, not here — the placement forms themselves
are fine.

### Passes — interaction (15/20)

Pick-block (data + item), replace-block (placement replace rules),
door/gate/trapdoor use, chest/barrel use, furnace use, crafting
table use, anvil use, enchanting table use, beacon use, brewing
stand use, lectern use, jukebox use, note block use, lever/button
use, pressure plate use, tripwire hook use (the USE is fine — the
re-check is G25), bell use (the USE is fine — and the hit test is
a PASS, see the G18 correction), fletching table use (G12 FIXED —
plain block, PASS, no menu), bed use form (the PAIR CHECK is
G27), dragon egg use form (the BORDER is G28), composter use form
(the ITEM TABLE is G22), mud conversion form (the TAG is G19),
portal use form (the FRAME is G20 — FIXED, obsidian hardcoded).
The five gaps are the bonemeal-growth check (G29/G32/G36), G19,
G22, G27, G28.

### Passes — breaking/removal (11/14)

`drop_resources` form (loot tables are Batch 3's), block-break
events (the 2001 event form matches; the door `!correct_tool`
extra 2001 event is a carried nit, not a defect — see Nits),
ice melt form (the EVENT is G23), tripwire break (the RE-CHECK
is G25). The three gaps are G24 (ice mined), G25 (tripwire
re-check), G35 (digging events).

### Passes — structural (1/2)

The class-chain check PASSES for B5 scope: census 1,166 blocks,
291 distinct class names, 26 unmapped, 1 plain-`Block` fallback
(`minecraft:amethyst_block`), 90 ancestor-recognised — and every
in-scope chain resolves to a live `Class` variant (the 1.21.10
aliases cover `FarmBlock`, `DirtPathBlock`,
`SpreadingSnowyDirtBlock`, `SnowyDirtBlock`, `FungusBlock`,
`RootsBlock`, `RedStoneWireBlock`, `PointedDripstoneBlock`; the 26
unmapped classes — furnaces, heads, glass, weathering copper,
leaves — are all out of B5 scope). The tag-inventory check GAPS:
203 tags; absent (verified): `#speleothems`,
`#nether_portal_frame`, `#blocks_motion_in_heightmap`,
`#blocks_motion_in_heightmap_no_leaves`, `#convertible_to_mud`
('i'), `#blocks_motion`, `#blocks_fluid_flow`,
`#washed_away_by_fluids`, `#blocks_lava_fire_spread`,
`#sculk_growth_inhibitors`, `#douses_campfires`, `#grows_crops`,
`#overrides_mushroom_light_requirement`,
`#cannot_support_snow_layer`, `#support_override_snow_layer`,
`#cannot_support_seagrass`, `#cannot_support_kelp`,
`#support_override_cactus_flower`, ALL `supports_*` tags checked
(`supports_nether_wart`, `supports_crimson/warped_fungus`,
`supports_crimson/warped_roots`, `supports_stem_crops`,
`supports_stem_fruit`, `supports_pumpkin/melon_stem[_fruit]`,
`supports_vegetation`, `supports_cactus`, `supports_sugar_cane`,
`supports_bamboo`, `supports_chorus_flower`,
`supports_chorus_plant`, `supports_mangrove_propagule`,
`supports_hanging_mangrove_propagule`, `supports_cocoa`,
`supports_azalea`, `supports_nether_sprouts`),
`#prevents_nearby_leaf_decay`, `#causes_suffocation`,
`#suppresses_bounce`. Present (verified):
`#convertable_to_mud` ('a'), `#mushroom_grow_block`,
`#snow_layer_cannot_survive_on`, `#snow_layer_can_survive_on`,
`#maintains_farmland`, `#sniffer_egg_hatch_boost`, `#snow`,
`#dirt`, `#nylium`, `#logs`, `#dry_vegetation_may_place_on`,
`#jungle_logs`.

### Nits recorded as passes, not defects (carried from 26.1.2, re-verified where in scope)

- Door `!correct_tool` extra 2001 event (unobservable in practice).
- Sniffer-egg hardcoded `moss_block` (matches the shipped data).
- Turtle-egg/eyeblossom hardcoded overworld timelines (match vanilla
  1.21.10 timeline data).
- Tripwire disarm flags 260 vs vanilla 3 (unobservable).
- Bamboo dead `bamboo_height_above` query (no caller).
- Sleep-height shape-top form (equivalent for all 1.21.10 beds — see
  G27's note).
- Poplar grower (dead code — no `poplar_sapling` in the export;
  folded into G33's evidence).
- Single-path tree flags 818 vs vanilla 260 (unobservable — the
  state written is the same; folded into G34's evidence).
- The `bonemeal.rs` `place_inline_feature` API (out of B5 scope —
  noted only).

### Pilot leftovers diagnosed

None outstanding for Batch 5. The 26.1 pilot's B5 leftovers (the
composter table, the TreeGrower draw form, the ice rules) were all
decided in the 26.1 run and re-verified here — and the 1.21.10 tree
still carries the 26.x regressions on all three (G22, G33/G34,
G23/G24). No check in this batch is left undecided.

### Drivers and oracle status

No driver runs per the brief (static checks only). Oracle status is
PROVISIONAL (no Batch 0 measured table for 1.21.10 — see §0):
in-scope oracles — 6 PASS (provisional, clean imports),
9 FAIL (minted as B5-O01–O09, each with its failing symbol and jar
fact cited). The oracle sources in the 1.21.10 tree carry the 26.3
forms throughout (`Prediction` drop override, `Feature.DIRECT_CODEC`,
`gamerules` import, `EntityTypes` references, `MaterialBaker`
extension) — the 1.21.10 adaptation never reached the oracle
sources.

### Evidence index

Batch evidence dir: `~/workspace/mcir-work/sweep-1.21.10-batches/b05/evidence/`

- `block-chains-1.21.10.json` + `block-class-map.json` +
  `block-class-lambda.json` — class-chain census (1,166 blocks,
  291 distinct class names, 26 unmapped, 1 plain-`Block` fallback,
  90 ancestor-recognised), built from `Blocks.<clinit>` +
  BootstrapMethods.
- `block-tags-1.21.10.json` — tag inventory (203 tags, absent/
  present per tag), built from the client jar's
  `data/minecraft/tags/block/*.json`.
- `javap/` (73 files) — fresh `javap -c -p` dumps from the 1.21.10
  jars: `FarmBlock`, `DirtPathBlock`, `SpreadingSnowyDirtBlock`,
  `SnowyDirtBlock`, `FungusBlock`, `RootsBlock`, `StemBlock`,
  `AttachedStemBlock`, `CropBlock`, `MushroomBlock`,
  `SnowLayerBlock`, `SeagrassBlock`, `KelpBlock`,
  `CactusFlowerBlock`, `CactusBlock`, `PointedDripstoneBlock`,
  `BellBlock`, `TreeGrower`, `SaplingBlock`, `AzaleaBlock`,
  `SweetBerryBushBlock`, `BoneMealItem`, `GrassBlock`,
  `ComposterBlock`, `PotionItem`, `PortalShape`, `Heightmap`
  (`-v` for the bootstrap method), `IceBlock`, `TripWireBlock`,
  `TripWireHookBlock`, `EyeblossomBlock`, `BedBlock`,
  `DragonEggBlock`, `ChorusFlowerBlock`, `LevelEvent`
  (`-constants`), `GameRules` (60 rules, `doFireTick` present),
  `Player`, `Entity`, `LivingEntity`, `ChunkMap`, `Feature`,
  `ConfiguredFeature`, `DataComponents`, `CubeVoxelShape`,
  `BlockTags`, `BlockBehaviour`, `BlockStateProperties`, `Blocks`
  (`-c` clinit + `-v`), `SugarCaneBlock`, `CocoaBlock`,
  `NetherWartBlock`, `MangrovePropaguleBlock`,
  `NetherSproutsBlock`, `VegetationBlock`, `GrowingPlantHeadBlock`,
  `FlowerPotBlock`, `FireBlock`, `BambooStalkBlock`, `VineBlock`,
  `TurtleEggBlock`, `SnifferEggBlock`, `FrostedIceBlock`,
  `DispenserBlock`, `ObserverBlock`, `PistonBaseBlock`,
  `RedStoneWireBlock`, `SpongeBlock`, `Level`, `LevelReader`,
  `ServerLevel` (plus `LevelHeightAccessor.getMaxY` and
  `SupportType$2/$3` read inline, and the 26.1.2 jar's
  `LevelHeightAccessor.getMaxY` for the G17 arbitration).

Key port-side line numbers (1.21.10 tree,
`versions/1.21.10/crates/horizon-world/src/level/` unless noted) —
all verified fresh:

- G04: `survive.rs:399-402` (`support_tag`); `classes.rs`
  (aliases); exporter `VanillaBlockBehaviourExporter.java`
  (reflective field dump).
- G08/G09/G11: `survive.rs` (SnowLayer/Seagrass/CactusFlower
  arms); `data.rs:458` (tag table).
- G13: `speleothem.rs:15`, `falling.rs:55`, `place.rs:1409`,
  `shapes.rs:1186` (tag reads); `tick.rs:257`/`443`,
  `shapes.rs:745`, `place.rs:1156` (live dispatch arms).
- G19: `data.rs:458`, `horizon-client/src/game/potions.rs:710`,
  `horizon-client/src/game/remote.rs:8677`,
  `horizon-client/src/game/dispenser.rs:2251` (all 'i' spelling).
- G20 (FIXED): `portal.rs:43-46` (`portal_frame` hardcodes
  obsidian).
- G21: `weather.rs:41-59` (`motion_blocking_top`);
  `feature_world.rs:350-360` (same-root-cause site).
- G22: `horizon-items/src/items.rs:1049` (component read);
  `game/composter.rs:24-25`; `tick.rs:86` (tick arm — works).
- G23: `hooks.rs:701-717` (`ice_melt`, event line at `:716`).
- G24: `hooks.rs:535-553` (`ice_destroyed_by_player`, tag read at
  `:549`).
- G25: `rails.rs:593-597` (`else if was` arm at `:595-596`).
- G26: `timeline.rs:10-24` (`box_scan_order`), `:69-115`
  (`eyeblossom_tick`).
- G27: `interact.rs:365-386` (pair check).
- G28: `interact.rs:581-600` (`dragon_egg_teleport`, no border check).
- G29: `bonemeal.rs` Sapling arm (`tree_can_grow` gate);
  `growth.rs:425-444` (`tree_can_grow`).
- G32: `bonemeal.rs:1244`/`1252` (`grass_bonemeal_effect`,
  `next_float`).
- G33: `growth.rs:145-269` (weighted lists); dead poplar grower.
- G34: `growth.rs:375-382` (`remove_sapling`, 818), `:384-419`
  (`grow_tree`).
- G35: `game/block_breaking.rs:100-101`
  (`destroy_progress_event`), `game.rs:5305`/`5317` (call sites),
  `game.rs:4312` (handler).
- G36: `bonemeal.rs:216-243` (`bone_meal_succeeds_on_client`).
- G37: `survive.rs:336-360` (SugarCane arm).
- G38: `survive.rs:150` (Cocoa arm).
- G39: `survive.rs:403` (Azalea arm).
- G40: `survive.rs:252-260` (hanging), `:430` (planted).
- G41: `survive.rs:432` (NetherSprouts arm).
- G42: `survive.rs:433` (NetherWart arm).
- G17 (CLOSED): `tick.rs:1441` (`above_pos.y > max_y() - 1` —
  CORRECT); `LevelHeightAccessor.getMaxY` = 319 in both jars.
- G18 (CLOSED): `redstone.rs:1442` (`click_y <= 0.8124f32` —
  CORRECT).
- G01 (FIXED): `block_landing.rs:416-445` (trample rule, matches
  vanilla `fallOn` in full).
- O01–O09: oracle sources in
  `mods/horizon/src/test/java/dev/horizonlab/parity/` (line
  numbers in §4).

## §8 Verdict

**Batch 5 (1.21.10): 128 checks, 101 pass, 27 gap; 37 items (26
behaviour gaps — 10 major, 16 minor — plus 9 oracle defects, 1
major, plus 2 closed items).** The 1.21.10 adaptation fixed the
dispatch half of the 1.21.7 ledger (8 FIXED) and left the tag half
(12 REPRODUCE, one worse than its source) — and the tag half grew:
six NEW major survival defects (G37–G42) where the port reads
`supports_*` tags 1.21.10 never had. Twenty-two of the 26.1.2
run's twenty-nine items reproduce; the seven that don't are two
fixes (G03, G20), one withdrawal (G15 — the 26.x bonemeal-gate API
doesn't exist in 1.21.10) and four never-existed (G17, G18, G30,
G31), two of which correct the source ledgers (G17's `getMaxY`
misread, G18's `ifle` misread — both verified fresh in 1.21.10's
own jar, G17 also in 26.1.2's). The batch is complete: all checks
decided statically, all 22 primary and 29 secondary re-checks
decided, no needs-harness items, nothing stalled. Two things the
parent should know: (1) oracle triage is PROVISIONAL — Batch 0's
measured table has not landed for 1.21.10, and it should confirm
the nine oracle defects (each cites its failing symbol and jar
fact); (2) the port's own tests still assert the wrong behaviour
in at least two places (`growth_tests.rs`: lone dark oak refusal,
client bonemeal success), so a fix wave will need to restore the
correct tests too — and the G17 correction should reach the 26.1.2,
26.1.1 and 26.1 ledgers before anyone "fixes" a chorus ceiling
that was never broken.
