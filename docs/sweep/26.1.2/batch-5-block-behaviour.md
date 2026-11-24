# Batch 5 — Block behaviour (26.1.2 parity sweep)

Checks + triage only, against VANILLA 26.1.2's own jar and data (named
server-game jar `~/workspace/mcir-work/26.1.x/versions/26.1.2/jars/server-game.jar`,
sha1 `83eb1106f12f24c9127ce22070d71853157af127`; client game jar
`.../jars/client.jar`, sha1 `4e618f09a0c649dde3fdf829df443ce0b8831e65`;
prepared 26.1.2 data in
`versions/26.1.2/.local/horizon-client/26.1.2/`). Static checks only:
javap on the named jars, Rust source comparison, vanilla/prepared data.
No fixes, no cargo builds, no driver runs, no git commands in the
worktree.

**Headline:** twenty-eight of the 26.1.1 run's twenty-nine items
REPRODUCE in 26.1.2 — the 26.1.2 tree is a delta port of the 26.1.1
tree with `level/` byte-identical, and every in-scope vanilla class is
byte-identical between the two jars, so every 26.1.1 defect in scope
carries over unchanged: the dead composter (B5-G22, MAJOR), the
uncreatable nether portal (B5-G20, MAJOR), snow that never accumulates
(B5-G21, MAJOR), dead dripstone (B5-G13, MAJOR), the bonemeal gates in
both directions (B5-G15, B5-G29–G32), the weighted TreeGrower (B5-G33/
G34), and the rest. **The one exception is the headline finding of
this batch: B5-G18 (bell `isProperHit` "inverted") NEVER EXISTED.**
Re-reading the bytecode fresh and decompiling `BellBlock` with
Vineflower shows vanilla 26.1.2 requires `click_y <= 0.8124` — exactly
what the port implements. The 26.2 run misread the `ifle` branch
direction, and the misread propagated through the 26.1 and 26.1.1 runs
unchallenged; the port was correct all along. G18 lands as a CLOSED
item with the correction recorded, and the bell check flips to PASS.
No NEW 26.1.2 defects: the 7-item port delta touches nothing in B5
scope (its only behaviour items are `handleAttack` and a Checkbox
tooltip).

## §0 Execution status

Checks + triage only. No file under `versions/26.1.2/` was modified,
nothing was committed, no git commands ran in the worktree, no cargo
builds ran. Everything below is static analysis: `javap -c -p` on the
named 26.1.2 jars (43 dumps), a full-class sha1 compare of both jars
between 26.1.1 and 26.1.2, a full tree diff between the 26.1.1 and
26.1.2 trees in this worktree, source comparison against
`versions/26.1.2/crates/`, the prepared 26.1.2 data (Batch 0's
prepare, built by Batch 12, exit 0: `block-behaviours.json`,
`gameplay-facts.json`, `block-tags.json`, `items.json`, configured
features), and the 26.1.2 Batch 0 oracle measurement
(`docs/sweep/26.1.2/batch-0-oracles.json`, 391 oracles, 202 fail-all /
158 pass-all / 31 mixed).

**Tally: 128 behaviour checks — 103 pass, 25 gap** (26.1.1: 128
checks, 102 pass, 26 gap). Behaviour/non-behaviour split: **128
behaviour + 0 non-behaviour**. Batch 5 is not one of the three
as-tallied exception batches; oracle triage is counted separately
below. The single tally change vs 26.1.1 is the bell check flipping
gap → pass on the G18 correction — a correction, not a fix; no 26.1.1
pass flips to gap and no new gap appears.

**Items: 29** — 21 behaviour gaps (4 major, 17 minor) + 7 oracle
defects (1 major, 6 minor) + 1 closed item (B5-G18, NEVER EXISTED).
28 of the 29 reproduce 26.1.1's items; 0 are NEW for 26.1.2.

**26.1.1 re-checks: 29/29 decided — 28 REPRODUCE, 1 NEVER EXISTED
(G18).** No 26.1.1 item was FIXED in the 26.1.2 tree.
**1.21.7 re-checks: 15 behaviour + 7 oracle, all decided** — G03 and
G13 reproduce, G15 reproduces in BOTH directions (B5-G15 + B5-G29),
G14 splits (single path evaporates, mega path reproduces as B5-G34),
G01/G02/G04–G12 evaporate (11); oracle O01 reproduces, O02's cause
evaporates (cascade only), O03 evaporates (PASS), O04 reproduces,
O05/O06/O07's 1.21.7 causes evaporate but the oracles still fail on
the 26.1 causes. Identical to the 26.1.1 run's 1.21.7 verdicts, every
one re-verified fresh against 26.1.2 evidence.

## §1 Scope

Block ticks, interaction, survival/support, updates, breaking/removal,
placement — the general block-behaviour surface: farmland, grass/mycelium
spread, dirt paths, nether fungi/roots, stems/crops, mushrooms, snow
layers, seagrass/kelp, cactus flowers, fletching table, dripstone,
saplings/bonemeal, trees, bells, chorus, portals, pick-block, replace,
plus the composter, ice, tripwire, eyeblossom, beds, dragon eggs and
mining feedback that the tree diff put in scope. Method: javap on the
named 26.1.2 jars + source comparison + prepared data
(`block-behaviours.json` class chains, `block-tags.json`, `items.json`,
configured-feature JSON). No cargo builds, no driver runs.

## §2 Method

Three verification layers, all fresh against 26.1.2 evidence:

1. **Jar identity proof.** A FULL sha1 byte-compare of every class
   entry between the 26.1.1 and 26.1.2 jars — server-game: 7,351
   classes each, exactly 3 changed (`DetectedVersion`,
   `SharedConstants`, `ServerGamePacketListenerImpl` — the documented
   26.1.1→26.1.2 delta, none in Batch 5 scope); client: 10,682
   classes each, exactly 8 changed (the same 3 plus `Checkbox` ×3 and
   `AbstractReportScreen` ×2 — GUI only). Of 142 distinct in-scope
   B5 class names checked, 122 are byte-identical, 0 changed, and 20
   are absent from BOTH jars — 12 meaningful absences
   (`FarmBlock`, `FletchingTableBlock`, `FungusBlock`, `RootsBlock`,
   `SpeleothemBlock`, `SpreadingSnowyDirtBlock`, `SnowyDirtBlock`,
   `RedstoneWireBlock`, `EntityTypes`, `BlockEntityTypes`,
   `net.minecraft.util.Prediction`, `SulfurSpikeBlock`) and 8 names
   that match no class in either jar (check-list spellings, e.g.
   `BambooBlock` is `BambooStalkBlock` in both jars, `GameRules`
   lives in `net.minecraft.world.level.gamerules` in both). Every
   jar-side verification from the 26.1.1 run therefore carries to
   26.1.2 by identity, not by assumption — and every defect-bearing
   method was ALSO re-dumped fresh from the 26.1.2 jar (43 dumps, see
   §7 evidence index), including a Vineflower decompilation of
   `BellBlock` to arbitrate the G18 correction.
2. **Tree diff.** A full `diff -rq` of `versions/26.1.1` vs
   `versions/26.1.2` in this worktree (32 differing files + 5
   only-in entries, `evidence/tree-diff-summary.txt`):
   `crates/horizon-world/src/level/` is byte-IDENTICAL, every
   game-layer file in B5 scope is byte-identical (`composter.rs`,
   `beds.rs`, `block_breaking.rs`, `potions.rs`, `remote.rs`,
   `dispenser.rs`, `campfire_douse.rs`, `motion.rs`, `mobs/ai.rs`,
   `block_landing.rs`, `game/lan/level_event_targets.rs`),
   `horizon-items/src/items.rs` is identical, and the oracle sources
   are identical. The differing files are all out of B5 scope: docs,
   version stamp, protocol sign-text component handling, the
   `chunk.rs` saved-palette legacy-form removal (save loading, noted
   in Cross-references), datafix, LAN/GUI. The 103 pass checks rest
   on (1)+(2): the jar methods are the ones the 26.1.1 run verified,
   and the port code deciding each check is unchanged from the 26.1.1
   tree — with the single exception of the bell check, whose verdict
   changes on the jar side (the G18 correction), not the port side.
3. **Fresh data checks.** The class-chain census (1,168 blocks,
   291 distinct class names, 28 unmapped — identical to 26.1.1), the
   tag inventory (248 tags; absent/present re-verified per tag), the
   stem support fields, the configured-feature JSON nesting (oak/
   spruce/azalea base heights, mushroom foliage radii), and the
   `compostable` component census (0/1,506 items) were all re-run
   against the prepared 26.1.2 data (`evidence/census.py`, log in
   `evidence/`).

**Counting rule:** one check = one vanilla-method/behaviour comparison
(jar + port source, or jar + prepared data). One item may sit behind
several checks (G03, G13) and one check may carry several items (the
bonemeal-growth check carries G15 + G29 + G30 + G31 + G32). Oracle/
driver-path checks are not in the behaviour tally.

**Note on the 26.1.1 summary's "26.1-record corrections" section**
(`sweep-26.1.2-ref/26.1.1/summary.md`, line 360): read and applied.
None of its corrections touches Batch 5 scope — B3-G33 (visibility)
is Batch 3's, the B8 items are Batch 8's, and the missed-by-26.1 items
(B8-G10, B10-G15, B3-G34) are other batches'. No B5 verdict changes
from it. The G18 correction in this report is NEW and belongs in the
26.1.2 run's own corrections section (see §6.4).

## §3 Checks table

| Family | Checks | Pass | Gap |
|---|---:|---:|---:|
| Random/scheduled ticks | 37 | 29 | 8 (B5-G03, B5-G13, B5-G17, B5-G21, B5-G22, B5-G23, B5-G26, B5-G33/G34) |
| Survival/support | 28 | 26 | 2 (B5-G03, B5-G13) |
| Updates | 16 | 14 | 2 (B5-G03, B5-G13) |
| Placement | 11 | 9 | 2 (B5-G03, B5-G13) |
| Interaction | 20 | 14 | 6 (B5-G15/G29/G30/G31/G32, B5-G19, B5-G20, B5-G22, B5-G27, B5-G28) |
| Breaking/removal | 14 | 11 | 3 (B5-G24, B5-G25, B5-G35) |
| Structural (class chains, tags) | 2 | 0 | 2 (class-chain liveness, tag inventory) |
| **Total** | **128** | **103** | **25** |

Behaviour checks only; oracle triage is counted separately below. The
structural gaps are the systemic roots of G03/G13/G19/G20/G21/G22/G24 —
the checks are the census and the inventory themselves, not a re-count
of the defects. G03 and G13 each appear in four families because each is
a distinct code path (tick, survive, updateShape, placement) sharing one
root cause. The bonemeal-growth interaction check carries five items
(G15 + G29–G32); the sapling-advance tick check carries two (G33, G34).
Interaction is 14/6 (26.1.1: 13/7) — the bell check is now a pass on
the G18 correction; every other family is unchanged from 26.1.1.

4 gaps are MAJOR (G13, G20, G21, G22), 17 are MINOR (G03, G15, G17,
G19, G23–G35 except G22). No B5-H (needs-harness) items: prepared
data landed before this batch, so every check in scope was decided
statically.

Oracle defects: 7 (B5-O01, O04, O05, O06, O07, O08, O09) — 6 minor,
1 major (O09, FaceSupport, blocks a driver family). B5-O02 is a cascade
with no independent defect (do not mint); B5-O03 evaporates (PASS all
classpaths in the measured Batch 0 table).

## §4 Defects

### Reproduced 26.1.1 defects (all verified fresh — jar byte-identical, port code byte-identical)

### B5-G03 — Dirt path never reverts: `DirtPathBlock` unmapped in `classes.rs` (MINOR, REPRODUCES)

26.1.2's jar HAS `DirtPathBlock` (no `PathBlock`); the export chain is
`['DirtPathBlock','Block']`. But the 26.1.2 tree's `classes.rs:208`
(byte-identical to 26.1.1's) reads `Path = "PathBlock"` only — the
`"DirtPathBlock"` alias 26.2's tree carries was dropped. The class
resolves to plain `Block`, so every path arm is dead: survive
(`survive.rs:273`), updateShape, scheduled tick (`tick.rs:202`,
unconditional `turn_to_base_block`), placement. Dirt paths never revert
to dirt when the block above becomes solid, and never revert on schedule.
Vanilla 26.1.2 `DirtPathBlock` behaviour itself is unchanged from 26.1.1
(byte-identical class). Fix note: add `"DirtPathBlock"` to the `Path`
alias (the 26.2 form), with the unit test 26.2's tree carries.

### B5-G13 — Dripstone fully dead: no `SpeleothemBlock` class in 26.1.2 AND `#speleothems` tag absent (MAJOR, REPRODUCES)

Two independent causes, either alone sufficient:

(a) **Class dispatch dead.** 26.1.2 has no `SpeleothemBlock` class
(`PointedDripstoneBlock` extends `Block` directly, javap-verified
fresh; export chain `['PointedDripstoneBlock','Block']`). The
`Class::Speleothem` dispatch arms never fire: scheduled tick →
`spawn_falling_stalactite` (`tick.rs:248`), random tick → growth
(`tick.rs:434`), survive (`survive.rs:307`), updateShape
(`shapes.rs:728`), placement (`place.rs:1156`).

(b) **Tag predicates dead.** `#speleothems` is ABSENT from 26.1.2 data
(verified in `block-tags.json` and in the client jar — no
`speleothems.json`), so all tag predicates fail: `speleothem.rs:15`,
`falling.rs:55`, `place.rs:1407`, `shapes.rs:1189`. Vanilla 26.1.2 uses
hardcoded `Blocks.POINTED_DRIPSTONE` checks instead.

Observable: dripstone never grows, stalactites never fall,
`can_survive` defaults true (floating dripstone never pops), placement tip
direction/thickness never computed. The cauldron-fill half remains Batch 1's
G6. Fix note (for the fix agent): 26.1.2 needs the 26.2 pattern inverted —
dispatch on `PointedDripstoneBlock` directly (or add a `PointedDripstone`
class mapping) and replace the `#speleothems` tag reads with a hardcoded
pointed-dripstone check, matching vanilla 26.1.2's own form. While fixing,
also fix the latent B5-G16: vanilla 26.1.2 `PointedDripstoneBlock.canGrow`
requires the fluid two blocks above the stalactite start to be WATER **and a
source** (`isSource()`, javap-verified fresh); the port (`speleothem.rs:104-107`)
checks only `fluid.kind() == FluidKind::Water`. Unobservable while growth is
dead, so not minted separately.

### B5-G15 — Bonemeal validity too permissive at the build ceiling: sapling/azalea/mushroom height gates read the wrong JSON level (MINOR, REPRODUCES)

Vanilla 26.1.2 HAS bonemeal height gates (unlike 1.21.7), re-verified
fresh by javap: `SaplingBlock.isValidBonemealTarget` = ServerLevel AND
`isInsideBuildHeight(pos.above(getMinimumHeight))`;
`AzaleaBlock` = same + 2 and fluid above empty; `MushroomBlock` =
feature exists AND `AbstractHugeMushroomFeature` AND
`HugeMushroomFeatureConfiguration` AND
`isInsideBuildHeight(pos.above(4 + foliageRadius))`.
`getMinimumHeight` = the primary tree's `trunkPlacer.getBaseHeight()`.
The port's `bonemeal_valid` (`bonemeal.rs:326-360`) matches in form, but
`tree_minimum_height` (`growth.rs:446-453`) reads
`json["trunk_placer"]["base_height"]` at the JSON top level, while the
stored feature JSON (verified fresh in
`data/minecraft/worldgen/configured_feature/*.json`) nests it under
`config` — `configured_json` (`worldgen/live.rs:234`) returns the raw
whole feature JSON `{"type":..., "config": {...}}`. So it always returns
0. Vanilla trunk base heights: oak 4, spruce 5, mangrove 2, azalea 4,
birch 5, jungle 4, acacia 5, cherry 7, dark/pale oak 6. The port accepts
bonemeal on saplings/azalea at heights vanilla refuses. Same nesting bug
for mushrooms: the port reads `json["foliage_radius"]` top-level
(`bonemeal.rs:353`) but it lives under `config` (brown = 3, red = 2,
verified fresh in the 26.1.2 configured-feature JSON). The port always
uses 2 — red correct by luck, brown's gate is pos.y+6 instead of
pos.y+7 (one block too permissive). `isBonemealSuccess`
(sapling/azalea 0.45, mushroom/fungus 0.4) matches vanilla exactly.
Note: `worldgen/features.json` in the prepared data is EMPTY (`{}`); the
live features come from `data/minecraft/worldgen/configured_feature/*.json`
via `configured_json`. Fix note: read `config.trunk_placer.base_height`
and `config.foliage_radius` (or make `configured_json` return the `config`
object — its only other caller is the mushroom check). The opposite
direction (too strict) is B5-G29 below — both directions are live at
once in 26.1.2.

### B5-G17 — Chorus flower never ticks at the build ceiling (MINOR, REPRODUCES)

Vanilla 26.1.2 `ChorusFlowerBlock.randomTick` proceeds when
`above.y <= getMaxY()` (320) — bytecode `if_icmple 28`, re-verified
fresh. The port (`tick.rs:1430`, `chorus_flower_random_tick`, file
byte-identical to 26.1.1's) returns early when
`above_pos.y > max_y() - 1`, i.e. above.y >= 320. A flower at pos.y ==
319: vanilla still ages/branches (horizontal targets at y=319 are
in-bounds); the port returns early and the flower stays forever young.
The rest of the chorus logic matches exactly (age < 5, support/pillar
logic, `height < 2 || height <= rand(5 or 4)`, grow-up with
`allNeighborsEmpty` + empty at +2, branch attempts `rand(4) +
pillar?1:0`, events 1033/1034, age-5 death). Note for the fix agent: the
analogous vine up-branch guard (`pos.y < max_y() - 1` vs vanilla
`pos.y < maxY`) is NOT a defect — at y=319 vanilla's up-branch has no
observable effect either (no block can exist at y=320 to attach to).
Only the chorus-flower gate is wrong, because its horizontal branches at
y=319 are observable.

### B5-G19 — Water bottle on dirt never makes mud: port reads `#convertible_to_mud`, 26.1.2's tag is `#convertable_to_mud` (MINOR, REPRODUCES)

The port reads `#convertible_to_mud` (with 'i') in four places:
`data.rs:460` (tag table), `horizon-client/src/game/potions.rs:710`
(water bottle use), `horizon-client/src/game/remote.rs:8658`,
`horizon-client/src/game/dispenser.rs:2245` (dispenser water bottle).
26.1.2's tag is `#convertable_to_mud` (with 'a') — verified three ways:
`BlockTags.CONVERTABLE_TO_MUD` in the jar,
`data/minecraft/tags/block/convertable_to_mud.json` in the client jar
(values dirt, coarse_dirt, rooted_dirt — verified fresh in the prepared
`block-tags.json`), and vanilla `PotionItem.useOn` checks it (bytecode
offset 57, re-verified fresh). The port's tag read returns nothing, so
the mud conversion never fires. 26.2's tree uses the correct 'a'
spelling; the 26.1.2 port wrongly took the 26.3 spelling (the `data.rs`
tag table has only the 'i' spelling — `convertable_to_mud` appears
nowhere in the table). Fix note: rename all four reads to
`convertable_to_mud`.

### B5-G20 — No nether portal can ever be created: `portal_frame` reads the absent `#nether_portal_frame` tag (MAJOR, REPRODUCES)

The port's `portal_frame()` (`portal.rs:40-44`, file byte-identical to
26.1.1's) reads `#nether_portal_frame`, which is ABSENT from 26.1.2 data
(verified in `block-tags.json`; no synthesis anywhere in the tree) →
always false. Vanilla 26.1.2 `PortalShape.FRAME` is hardcoded
`state.is(Blocks.OBSIDIAN)` (`lambda$static$0` in `PortalShape.txt`,
re-verified fresh — note the class lives at
`net.minecraft.world.level.portal.PortalShape`, not `.block.`).
Consequence: `find_portal_shape` (`portal.rs:67`) always returns
width/height 0, `find_empty_portal_shape` (`portal.rs:131`) always
returns None, and `hooks.rs:588-597` `base_fire_on_place` never creates
portal blocks — **no nether portal can ever be created in the 26.1.2
port**. 26.2's tree correctly hardcoded obsidian. Fix note: hardcode
obsidian, matching vanilla 26.1.2 and the 26.2 tree's form.

### B5-G21 — Snow never accumulates on land, ice never forms except on water: `motion_blocking_top` reads the absent `#blocks_motion_in_heightmap` tag (MAJOR, REPRODUCES)

The port's `motion_blocking_top` (`weather.rs:41-59`, file byte-identical
to 26.1.1's) reads `#blocks_motion_in_heightmap`, which is ABSENT from
26.1.2 data (verified in `block-tags.json`; no synthesis anywhere in the
tree) → only fluids count as motion-blocking. Vanilla 26.1.2
`Heightmap.MATERIAL_MOTION_BLOCKING` is a method reference to
`BlockStateBase.blocksMotion()` (bootstrap method 1 in `Heightmap`,
re-verified fresh with `javap -v`); the MOTION_BLOCKING heightmap type
uses it. Consequence: `precipitation_info` computes top = min_y in
fluidless columns → snow never accumulates on land and ice never forms
except on water. The same wrong top also feeds `is_raining_at`
(under-glass check, `weather.rs:127`), lightning strike height
(`lightning.rs:199`), and portal-travel spawn height
(`portal_travel.rs:506`). 26.2's tree correctly used per-state
`blocksMotion()`. **Same-root-cause site in `feature_world.rs:350-360`:**
the 26.1 tree's `blocks_motion()` reads there were replaced with
`#blocks_motion_in_heightmap` / `#blocks_motion_in_heightmap_no_leaves`
tag reads (both tags absent), so feature-placement heightmaps
(OceanFloor, MotionBlocking, MotionBlockingNoLeaves) are broken the
same way; recorded here as an additional site of this item (worldgen
scope — see Cross-references; Batch 6's landed report does not cover
it). Fix note: replace the tag reads with the per-state `blocksMotion()`
value from the prepared data (26.2's form), or synthesise the tag at
prepare time from it.

### B5-G22 — Composter completely dead: the port reads the `minecraft:compostable` component no 26.1.2 item carries (MAJOR, REPRODUCES)

Vanilla 26.1.2 `ComposterBlock` composts by a static chance table:
`COMPOSTABLES` (`Object2FloatMap<ItemLike>`, 115 entries, `bootStrap()`)
and `addItem` (bytecode read fresh): level 0 + chance > 0 adds a layer
with NO roll (offsets 30–39); above level 0 it adds one when the level
random rolls under the chance (`nextDouble() < chance`, offsets 42–57).
The 26.1 tree implemented exactly this (`compost_chance` in
`horizon-items`, verified 115/115 exact in the 26.1 run; level-side
`composter_add_item` with the level-0 rule). The 26.1.2 tree carries
the 26.1.1 tree's 26.3 form:

- `horizon-items/src/items.rs:1015` populates the item's `compostable`
  field ONLY from the `minecraft:compostable` component:
  `compostable: c.get("minecraft:compostable").map(|f| f["layers"].clone())`.
- **Zero of the 1,506 items in the prepared 26.1.2 `items.json` carry
  that component** (census re-run fresh; 26.1.2 vanilla has no
  `compostable` component — 26.3 added it). `compost_chance` does not
  exist anywhere in the 26.1.2 tree (grep: zero hits).
- `game/composter.rs:29-38` therefore returns `None` for every item:
  `use_composter` returns false (no hand composting),
  `composter_container_insert` returns the stack untouched (no hopper
  composting), `villager_insert_item` returns early (no villager
  composting), and `composter_takes` is false for every held item.
- The level side (`level/interact.rs:559-633`) lost `composter_add_item`
  entirely: `composter_insert`/`composter_layer` now take a `layers`
  count directly, so the level-0-always-adds rule and the chance roll
  exist nowhere in the tree.

Observable: **no item can ever be composted in the 26.1.2 port** — the
composter accepts nothing by any path. (The level-7→8 tick itself still
functions if a composter reaches level 7 by command; the insertion path
is what is dead.) Fix note: restore the 26.1 form — the `compost_chance`
table plus `composter_add_item` with the level-0 rule.

### B5-G23 — Ice melt fires a `BLOCK_DESTROY` game event vanilla 26.1.2 never fires (MINOR, REPRODUCES)

Vanilla 26.1.2 `IceBlock.melt` (bytecode read fresh): water-evaporates
→ removeBlock, else setBlockAndUpdate(water) + neighborChanged — ZERO
`gameEvent` references in the whole class. The 26.1 tree matched this,
with a comment noting the event is a 26.3 addition. The 26.1.2 tree's
`ice_melt` (`hooks.rs:691-706`) adds
`self.game_event_state(pos, "minecraft:block_destroy", state)` after
the setBlockAndUpdate. Observable: sculk sensors (and any game-event
listener) trigger on every ice melt in the port, never in vanilla
26.1.2. (Also flagged by Batch 1 as out-of-scope observation (a).)
Fix note: delete the event line, restoring the 26.1 form.

### B5-G24 — Ice mined over solid ground leaves no water: `ice_destroyed_by_player` reads the absent `#blocks_motion` tag (MINOR, REPRODUCES)

Vanilla 26.1.2 `IceBlock.playerDestroy` (bytecode read fresh, offsets
52–86): unless the tool prevents melting or water evaporates, if the
block below `blocksMotion()` OR is liquid, the position becomes water
(`setBlockAndUpdate(meltsInto())`). The 26.1 tree matched this with the
per-state `blocks_motion()` rule (`mod.rs:975-985` in 26.1: solid unless
cobweb or bamboo sapling). The 26.1.2 tree deleted `blocks_motion()`
and `ice_destroyed_by_player` (`hooks.rs:525-541`) now reads
`b.in_set(below, &b.tags.blocks_motion)` — and `#blocks_motion` is
ABSENT from 26.1.2 data (verified in `block-tags.json`; the tag table
resolves absent tags to empty sets, `data.rs:396`
`.unwrap_or_default()`), so the check is always false. Observable:
mined ice over solid ground leaves air in the port where vanilla
26.1.2 leaves water (the liquid-below arm still works). (Also flagged
by Batch 1 as out-of-scope observation (a).) Fix note: restore the
26.1 per-state rule.

### B5-G25 — Tripwire schedules a 1-tick re-check when unpressed that vanilla 26.1.2 never schedules (MINOR, REPRODUCES)

Vanilla 26.1.2 schedules a tripwire re-check tick 10 ONLY while pressed:
`TripWireBlock`'s single `scheduleTick` is `bipush 10` gated on the
pressed flag (offsets 124–141, fresh javap), and `TripWireHookBlock`
has exactly ONE `scheduleTick` in the whole class — the 10-tick
attached-circuit re-check inside `calculateState` (offset 281). There
is no 1-tick unpress re-check anywhere in either class. The 26.1 tree
matched this, with a comment stating the pressed-to-unpressed immediate
re-check is 26.2-only. The 26.1.2 tree's `tripwire_check_pressed`
(`rails.rs:593-597`) adds `else if was { self.schedule_tick(pos, block,
1); }`. Observable: an extra scheduled tick on every press→release
transition (visible via the tick queue; the re-check itself finds the
wire still unpressed and does nothing further in the common case, but
it can catch an entity that entered without triggering `entityInside`
a tick earlier than vanilla would). Fix note: delete the `else if was`
arm, restoring the 26.1 form.

### B5-G26 — Eyeblossom neighbour cascade scans in 26.3 section order instead of `betweenClosed` order (MINOR, REPRODUCES)

Vanilla 26.1.2 `EyeblossomBlock.tryChangingState` iterates the 7×5×7
neighbour box with `BlockPos.betweenClosed` (offset 99, fresh javap) —
x innermost, then y, then z — and each matching neighbour draws its
delay from `nextIntBetweenInclusive(dist*5, dist*10)` in that scan
order. The 26.1 tree matched this with `between_closed` (plus a unit
test pinning the order). The 26.1.2 tree replaced it with
`box_scan_order` (`timeline.rs:9-23`, the 26.3 `findBlocksIn` order:
sections by x, z, y; blocks by y, z, x), used by `eyeblossom_tick`
(`timeline.rs:92`). Vanilla 26.1.2 has no blockscan package at all
(0 jar entries). Observable: the per-neighbour random draws are
assigned in a different order, so the cascade delays differ from
vanilla for the same seed (the open/close windows themselves still
match: `(12600..23401)`). Fix note: restore `between_closed` and its
test, the 26.1 form.

### B5-G27 — Bed use requires the other half: a head click with no foot returns Consume instead of sleeping (MINOR, REPRODUCES)

Vanilla 26.1.2 `BedBlock.useWithoutItem` (bytecode read fresh): client
→ SUCCESS_SERVER; if part != HEAD, redirect to the head and require
only that the head is this bed (else CONSUME); a HEAD click needs no
foot check at all — it proceeds straight to the `BED_RULE` path (sleep
or explode). The 26.1 tree matched this. The 26.1.2 tree's
`Class::AbstractBed` arm (`interact.rs:367-384`) implements the 26.3
pair check: it requires the block at the other half's position to be
this bed with the opposite part, else returns Consume. Observable:
clicking the head of a bed whose foot was destroyed (or replaced)
sleeps in vanilla 26.1.2 but does nothing in the port; the foot-click
path is unaffected (both forms Consume when the head is missing).
Fix note: restore the 26.1 form (foot redirects to head, head needs no
foot check). Note: the 26.1.2 tree's `sleep_height` change in
`game/beds.rs` (shape-top form) is NOT a defect — for every 26.1.2 bed
the shape top is 9/16 = 0.5625 and `pos_to_bed` adds 0.125, giving
exactly vanilla's flat 0.6875 (`LivingEntity.setPosToBed`, javap-verified
fresh); the straw-bed branch is dead (no `straw_bed` in the 26.1.2 export).

### B5-G28 — Dragon egg teleport ignores the world border (MINOR, REPRODUCES)

Vanilla 26.1.2 `DragonEggBlock.teleport` (bytecode read fresh) accepts
a candidate only when it is air, the block below is non-air,
`WorldBorder.isWithinBounds` (offsets 113–120), and inside build
height — 1,000 attempts. The 26.1 tree matched this
(`within_world_border(test)` in the candidate test). The 26.1.2 tree's
`dragon_egg_teleport` (`interact.rs:579-600`) dropped the border check:
candidates need only air + solid below + build height. Observable: a
dragon egg near the world border can teleport outside it in the port;
vanilla 26.1.2 never places it there. Fix note: restore the border
check, the 26.1 form.

### B5-G29 — Bonemeal validity too strict: the 26.3 `canGrow` gate refuses lone dark/pale oak saplings (MINOR, REPRODUCES — 1.21.7 G15's original form)

Vanilla 26.1.2 `SaplingBlock.isValidBonemealTarget` is ONLY the height
check (ServerLevel AND `isInsideBuildHeight(pos.above(getMinimumHeight))`,
javap-verified fresh) — there is no `canGrow` method on 26.1.2's
`TreeGrower` at all. The 26.1 tree matched this, with a comment noting
`canGrow` (the 2×2 requirement for a mega-only grower) is 26.3-only, so
a lone dark oak sapling takes the meal (and consumes it 45% of the
time with no growth). The 26.1.2 tree's `bonemeal_valid`
(`bonemeal.rs:335`) adds `!self.tree_can_grow(g, pos, state)` —
`tree_can_grow` (`growth.rs:425-444`) returns false for a mega-only
grower (dark oak, pale oak — empty `trees` list, non-empty `mega_trees`)
with no 2×2 square. Observable: bonemeal on a lone dark/pale oak
sapling is refused outright in the port; vanilla 26.1.2 accepts it (and
the 0.45 success roll consumes the meal with no tree). The gate also
draws random (`weighted` on both lists) on EVERY sapling/azalea
validity check, shifting the RNG stream where vanilla draws nothing.
(The 26.1.2 tree's own `growth_tests.rs` asserts the refusal —
`lone_dark_oak_sapling_cannot_grow` — where the 26.1 test asserted the
meal is taken.) Fix note: delete the `tree_can_grow` gate, restoring
the 26.1 form. The opposite direction (too permissive at the ceiling)
is the reproduced B5-G15 — both directions are live at once.

### B5-G30 — Sweet berry bonemeal validity dropped the air-above and build-height checks (MINOR, REPRODUCES)

Vanilla 26.1.2 `SweetBerryBushBlock.isValidBonemealTarget` (bytecode
read fresh, offsets 0–50): age < 3 AND air above AND inside build
height. The 26.1 tree kept all three, with a comment noting 26.2
dropped the air and height checks. The 26.1.2 tree's arm
(`bonemeal.rs:406`) is age-only: `Kind::SweetBerry =>
b.table.int(state, p.age) < 3`. Observable: bonemeal is accepted on a
sweet berry bush with a solid block above it, or at the build ceiling,
where vanilla 26.1.2 refuses it. Fix note: restore the 26.1 three-part
gate.

### B5-G31 — Bonemeal succeeds on the client for `growCrop` where vanilla 26.1.2 returns PASS (MINOR, REPRODUCES)

Vanilla 26.1.2 `BoneMealItem.useOn` (bytecode read fresh): a `growCrop`
success returns SUCCESS_SERVER on the server and PASS on the client
(offsets 36–72); only the `growWaterPlant` path returns SUCCESS on both
sides (offset 143). The 26.1 tree matched this — its
`bone_meal_succeeds_on_client` returned true only for the water-plant
path. The 26.1.2 tree's version (`bonemeal.rs:216-243`) returns true
for any valid non-sapling/azalea/mushroom bonemeal target on the client
(plus a mod-block answer path). Observable: on the client, bonemeal
used on a crop (or any valid growCrop target) predicts SUCCESS in the
port where vanilla 26.1.2 passes (the server's packet does the growing;
the client-side success changes the predicted hand swing / use
result). Fix note: restore the 26.1 form (water-plant path only).

### B5-G32 — Grass bonemeal rolls use `nextFloat` where vanilla 26.1.2 uses `nextInt` (MINOR, REPRODUCES)

Vanilla 26.1.2 `GrassBlock.performBonemeal` (bytecode read fresh):
the tall-grass roll is `nextInt(10) == 0` (offsets 169–177) and the
flower-feature roll is `nextInt(8) == 0` (offsets 238–246). The 26.1
tree matched this with `next_int(10)` / `next_int(8)`, with a comment
noting 26.3's form is `nextFloat` and the draws differ. The 26.1.2
tree's `grass_bonemeal_effect` (`bonemeal.rs:1224-1234`) uses
`self.random().next_float() < 0.1` and `< 0.125`. Observable: same
probabilities (10%, 12.5%), but different draw types consume the RNG
stream differently, so the bonemeal outcome (tall grass vs flower
feature vs nothing) and every random outcome after it in the tick
diverge from vanilla for the same seed. Fix note: restore the 26.1
`nextInt` rolls.

### B5-G33 — `TreeGrower` rewritten in the 26.3 weighted-list form: tree selection diverges from vanilla's secondary-chance draws (MINOR, REPRODUCES)

Vanilla 26.1.2 `TreeGrower` (bytecode + fields read fresh) is the
secondary-chance form: 10 growers (oak, spruce, mangrove, azalea,
birch, jungle, acacia, cherry, dark oak, pale oak — NO poplar),
`secondaryChance` float, `megaTree`/`secondaryMegaTree`/`tree`/
`secondaryTree`/`flowers`/`secondaryFlowers` optionals.
`getConfiguredFeature` draws ONE `nextFloat()` unconditionally (even
at chance 0) against `secondaryChance`; `getConfiguredMegaFeature`
draws only when a secondary mega tree exists (spruce only). The 26.1
tree implemented exactly this (with three unit tests pinning the draw
counts). The 26.1.2 tree carries the 26.1.1 tree's 26.3 weighted-list
form (`growth.rs:145-269`): `trees`/`mega_trees`/`flower_trees` weight
lists plus `weighted()` (`growth.rs:249-264`, `next_int_bound(total)`),
a `shortest` field, and a `poplar` grower (dead — no `poplar_sapling`
in the 26.1.2 export). Observable: the probabilities coincide (oak
fancy 10%, mangrove tall 85%, spruce mega 50/50) but the draw type
(`nextInt` vs `nextFloat`) and draw count differ — jungle's mega draw
happens in the port (total 1) where vanilla draws nothing, and every
single-tree grower draws `nextInt(1)` where vanilla draws a float —
so the chosen tree (fancy vs regular oak, tall vs short mangrove,
mega spruce vs mega pine) and the RNG stream after it diverge from
vanilla for the same seed. Fix note: restore the 26.1 secondary-chance
form and its tests.

### B5-G34 — Mega sapling growth leaves water where vanilla 26.1.2 leaves air (MINOR, REPRODUCES — 1.21.7 G14's form, mega path)

Vanilla 26.1.2 `TreeGrower.growTree`, mega path (bytecode read fresh,
offsets 90–288): clears the 2×2 square to `Blocks.AIR` with flags 260
and restores the PASSED sapling state on failure, also with flags 260.
The 26.1 tree matched this. The 26.1.2 tree's `grow_tree`
(`growth.rs:384-419`) removes each sapling via `remove_sapling`
(`growth.rs:375-382`) — the fluid's legacy block with flags 818
(`UPDATE_SKIP_ALL_SIDEEFFECTS | UPDATE_CLIENTS`) — and restores the
collected per-position states on failure. Observable: a 2×2 of
waterlogged saplings grown to a mega tree leaves water blocks at the
trunk base in the port where vanilla 26.1.2 leaves air. (The single
path is unaffected in substance: vanilla clears the sapling to its
fluid's legacy block with flags 260 — water IS vanilla 26.1.2's form
there, verified fresh at offsets 371–388, and the port's legacy block
matches — though the port's 818 flags differ from vanilla's 260: an
extra client notification and a skipped onPlace where vanilla runs it.
Note `UPDATE_NONE` IS 260 in the port's flag table, `mod.rs:164`.)
Fix note: restore the 26.1 mega path (air + flags 260, restore the
passed state) and the single-path flags.

### B5-G35 — Digging broadcasts level events 2019/2020 that vanilla 26.1.2 never sends (MINOR, REPRODUCES)

Vanilla 26.1.2 has NO level events 2019/2020: `LevelEvent`
(`net.minecraft.world.level.block.LevelEvent`, javap `-constants`,
fresh) ends its 20xx range at 2013 (`PARTICLES_SMASH_ATTACK`). The
26.1 tree matched this, with an explicit comment at `game.rs:5266`:
"26.1 sends no level event while a player digs (26.3's 2019/2020)". The
26.1.2 tree carries the 26.1.1 tree's 26.3 form: `destroy_progress_event`
(`game/block_breaking.rs:97-108`, called from `game.rs:5294` and
`:5306`) sends 2020 every fourth mining tick (the hit sound) else 2019
(the crack only); the client handles both (`game.rs:4298-4306`,
`breaking_block_effects` — crack particles on the mined face plus the
hit sound for 2020) and LAN broadcasts them
(`game/lan/level_event_targets.rs:28-29`). Observable: bystanders see
crack particles and hear hit sounds while someone digs in the port;
vanilla 26.1.2 sends no such events (the digger's own client plays its
sounds locally). Fix note: delete the event emission and handling,
restoring the 26.1 form. Cross-ref: Batch 10 (sound/rendering of the
events) — the emission itself is the breaking-family defect recorded
here.

### Closed re-check (NEVER EXISTED — a correction to the 26.1, 26.1.1 and 26.2 records)

### B5-G18 — Bell `isProperHit` height comparison: the port was correct all along (CLOSED, NEVER EXISTED)

The 26.1.1 item claimed vanilla `BellBlock.isProperHit` requires
`click_y > 0.8123999834060669` and the port's `click_y <= 0.8124f32`
(`redstone.rs:1442`) was inverted. That claim is WRONG, and it was
wrong in the 26.1 and 26.2 runs too (the 26.1 report says G18
"REPRODUCES from 26.2" — the misread originated there and propagated
unchallenged). Fresh evidence from the 26.1.2 jar, three ways:

1. **Bytecode.** `isProperHit` (fresh `javap -c`):
   `dload_3; ldc2_w 0.8123999834060669d; dcmpl; ifle 20; 18: iconst_0;
   ireturn; 20: <attachment switch>`. `dcmpl` pushes the sign of
   (clickY − 0.8124); `ifle 20` branches to the switch — the only
   path that can return true — exactly when clickY <= 0.8124. A
   click ABOVE 0.8124 falls through to offset 18 and returns false.
   Vanilla requires clickY **<=** 0.8124, not >.
2. **Decompilation.** Vineflower 1.12.0 on the 26.1.2 class gives the
   source form verbatim:
   `if (clickedDirection.getAxis() != Axis.Y && !(clickY > 0.8124F))`
   — the earlier agents dropped the negation when reading the branch.
3. **Physical sense.** The bell body tops out at y = 13/16 = 0.8125
   (`BELL_SHAPE = Shapes.or(Block.column(6.0, 6.0, 13.0), ...)`); the
   mount column is 13.0–16.0. Clicking at y <= 0.8124 hits the bell
   body; above that hits the mount. Vanilla's gate is the sensible
   one, and the port implements it exactly — including the value:
   `f64::from(0.8124f32)` is exactly the jar's double constant
   0.8123999834060669d (the source literal is a float widened to
   double). The caller passes the within-block Y in both forms
   (vanilla: `hitResult.getLocation().y - blockPos.getY()`; port:
   `hit.y - f64::from(pos.y)`, `interact.rs:424` / `mod.rs:1363`).
   The attachment/axis arms match too (floor: facing axis == hit
   axis; single/double wall: axes differ; ceiling: always), and
   `bell_ring` matches vanilla `attemptToRing`.

`BellBlock` is byte-identical across the 26.1, 26.1.1 and 26.1.2 jars
and the port code is byte-identical across the three trees, so the
item never existed in ANY of the three runs. The bell check is a PASS
in 26.1.2. **Coordinator:** the 26.1 and 26.1.1 ledgers carry G18 as a
live minor gap; both need this correction (see §6.4). No fix is
needed — there is nothing to fix.

## Oracle defects

Triage is from the measured Batch 0 table
(`docs/sweep/26.1.2/batch-0-oracles.json`) — no oracle was re-measured.
Each defect below is `fail-all` on all three classpaths. Full
`Vanilla*Oracle` names throughout. Line numbers are the oracle source
lines in `mods/horizon/src/test/java/dev/horizonlab/parity/` of the
26.1.2 tree, matched against the Batch 0 table's `firstErrorLine`.
Oracle sources are byte-identical to the 26.1.1 tree's.

### B5-O01 — `VanillaCauldronOracle` fails: missing-symbol at line 118 (ORACLE DEFECT, MINOR, REPRODUCES)

Batch 0 table: `FAIL all` (1 error, missing-symbol, line 118) —
identical to the 26.1.1 run. The 26.1.2 jar has no
`LayeredCauldronBlock.LEVEL` field of the form the oracle reads; the
level predicate differs in 26.1.2. Same defect, same cause. (See §7
for the behaviour status: cauldron behaviour is otherwise Batch 1's
fluid surface; the level predicate is what this oracle owns.)

### B5-O04 — `VanillaReplaceBlockOracle` fails: `Feature.DIRECT_CODEC` does not exist in 26.1.2 (ORACLE DEFECT, MINOR, REPRODUCES — 1.21.7's O04 cause)

Batch 0 table: `FAIL all` (1 error, missing-symbol, line 36). The
26.1 run's oracle used `ConfiguredFeature.DIRECT_CODEC` and PASSED.
The 26.1.1/26.1.2 oracle source was re-ported from the 26.2/26.3 form
and now reads `Feature.DIRECT_CODEC` (`VanillaReplaceBlockOracle.java:36`) —
but 26.1.2's `Feature` class has NO `DIRECT_CODEC` (javap-verified
fresh: only `ConfiguredFeature` has it). This is the exact 1.21.7
B5-O04 cause (26.2's oracle had it too). Fix note: use
`ConfiguredFeature.DIRECT_CODEC`, the 26.1 form.

### B5-O05 — `VanillaFallingBlockOracle` fails: constructor mismatch at line 30 (ORACLE DEFECT, MINOR, REPRODUCES)

Batch 0 table: `FAIL all` (1 error, constructor-mismatch, line 30 —
"constructor ChunkMap in class ChunkMap cannot be applied to given
types"). The falling-block entity constructor changed shape in 26.1.2
vs 26.3; the oracle still constructs with the 26.3 form. Same defect,
same cause as the 26.1.1 run (whose first-error line was also 30).

### B5-O06 — `VanillaDispenserOracle` fails: missing-package at line 212 (ORACLE DEFECT, MINOR, REPRODUCES)

Batch 0 table: `FAIL all` (15 errors, missing-package, line 212 —
"package EntityTypes does not exist"). The oracle imports
`EntityTypes` (`net.minecraft.world.entity.EntityTypes`), which does
not exist in 26.1.2 (registration still lives on `EntityType` itself
— the holder move happened after 26.1.2). Same defect, same cause as
the 26.1.1 run (also 15 errors, line 212).

### B5-O07 — `VanillaSectionMeshOracle` fails: `MaterialBaker` is an interface in 26.1.2 (ORACLE DEFECT, MINOR, REPRODUCES)

Batch 0 table: `FAIL all` (7 errors, other, line 84 — "no interface
expected here"). The oracle instantiates `MaterialBaker` as a class;
26.1.2's
`net.minecraft.client.resources.model.sprite.MaterialBaker` is a
PUBLIC INTERFACE (javap-verified fresh, byte-identical to the 26.1.1
class). Same defect, same cause as the 26.1.1 run (also 7 errors,
line 84).

### B5-O08 — `VanillaBreakingModelOracle` fails: `BlockModelFeatureRenderer` absent (ORACLE DEFECT, MINOR, REPRODUCES)

Batch 0 table: `FAIL all` (2 errors, missing-symbol, line 12). The
class `BlockModelFeatureRenderer` does not exist in 26.1.2 (absent
from the client jar, verified fresh — 0 entries; absent in 26.1.1
too). Same defect, same cause as the 26.1.1 run. (Note: the related
`BlockFeatureRenderer` DOES exist, at
`net.minecraft.client.renderer.feature.BlockFeatureRenderer` — the
oracle needs the model variant, which does not.)

### B5-O09 — `VanillaFaceSupportOracle` fails: protected `CubeVoxelShape` constructor at line 45 (ORACLE DEFECT, MAJOR, REPRODUCES)

Batch 0 table: `FAIL all` (1 error, other, line 45 —
"`CubeVoxelShape(DiscreteVoxelShape)` has protected access in
`CubeVoxelShape`"). The oracle constructs `CubeVoxelShape` directly;
26.1.2's constructor is PROTECTED (javap-verified fresh,
byte-identical to 26.1.1's). Same defect, same cause as the 26.1.1
run — MAJOR because it blocks 19 drivers plus 5 cascades. (Batch 2
also records it; the coordinator deduplicates.)

### Oracle non-items

- **B5-O02 (VanillaPortalOracle) — cascade, no independent defect, do
  not mint.** FAIL all via `VanillaBlockUpdateOracle` (Batch 0 table;
  the update oracle itself fails 32 errors, line 450). The 1.21.7 O02
  cause (`getRelativePortalPosition` missing) does NOT apply —
  26.1.2 HAS that method (`Entity.getRelativePortalPosition(Axis,
  FoundRectangle)`, javap-verified fresh); the failure is purely the
  cascade. Same as the 26.1.1 run.
- **B5-O03 (VanillaPickBlockDataOracle) — evaporates, PASS all
  classpaths** in the Batch 0 table. Same as the 26.1.1 run.

**In-scope oracles passing:** VanillaPickBlockDataOracle,
VanillaBlockSurvivalOracle, VanillaBlockSupportOracle,
VanillaBlockPropertiesOracle, VanillaBlockStateCodecOracle,
VanillaCollisionOracle, VanillaHeightmapOracle.

## §5 Re-check verdicts

### 26.1.1 re-checks (29/29 decided — 28 REPRODUCE, 1 NEVER EXISTED)

| 26.1.1 item | 26.1.2 verdict | 26.1.2 item |
|---|---|---|
| B5-G03 dirt path | REPRODUCES (jar byte-identical; `classes.rs` byte-identical) | B5-G03 |
| B5-G13 dripstone | REPRODUCES (both causes; `#speleothems` absent; `PointedDripstoneBlock` extends `Block`) | B5-G13 |
| B5-G15 bonemeal ceiling | REPRODUCES (nesting bug persists; see also B5-G29) | B5-G15 |
| B5-G17 chorus ceiling | REPRODUCES (`tick.rs` byte-identical) | B5-G17 |
| B5-G18 bell inverted | NEVER EXISTED (vanilla requires `<= 0.8124` — the port matches; Vineflower + bytecode) | B5-G18 (closed) |
| B5-G19 mud tag | REPRODUCES (all four reads still 'i' spelling) | B5-G19 |
| B5-G20 portal frame | REPRODUCES (`portal.rs` byte-identical) | B5-G20 |
| B5-G21 motion_blocking_top | REPRODUCES (`weather.rs` byte-identical; + `feature_world` site) | B5-G21 |
| B5-G22 composter | REPRODUCES (0/1,506 items carry the component; `compost_chance` absent) | B5-G22 |
| B5-G23 ice melt event | REPRODUCES (event line at `hooks.rs:706`) | B5-G23 |
| B5-G24 ice mined | REPRODUCES (`#blocks_motion` absent) | B5-G24 |
| B5-G25 tripwire re-check | REPRODUCES (`else if was` arm at `rails.rs:595-596`) | B5-G25 |
| B5-G26 eyeblossom order | REPRODUCES (`box_scan_order` at `timeline.rs:9-23`) | B5-G26 |
| B5-G27 bed pair check | REPRODUCES (`interact.rs:367-384`) | B5-G27 |
| B5-G28 dragon egg border | REPRODUCES (no border check in `dragon_egg_teleport`) | B5-G28 |
| B5-G29 canGrow gate | REPRODUCES (`bonemeal.rs:335` + `growth.rs:425-444`) | B5-G29 |
| B5-G30 sweet berry gates | REPRODUCES (age-only arm at `bonemeal.rs:406`) | B5-G30 |
| B5-G31 client bonemeal | REPRODUCES (`bonemeal.rs:216-243`) | B5-G31 |
| B5-G32 grass RNG | REPRODUCES (`next_float` at `bonemeal.rs:1224-1234`) | B5-G32 |
| B5-G33 TreeGrower | REPRODUCES (weighted form at `growth.rs:145-269`) | B5-G33 |
| B5-G34 mega water | REPRODUCES (`remove_sapling` at `growth.rs:375-382`) | B5-G34 |
| B5-G35 digging events | REPRODUCES (`block_breaking.rs:97-108` + `game.rs:5294/5306`) | B5-G35 |
| B5-O01 Cauldron | REPRODUCES (Batch 0 table, line 118) | B5-O01 |
| B5-O04 ReplaceBlock | REPRODUCES (Batch 0 table, line 36; `Feature` has no `DIRECT_CODEC`) | B5-O04 |
| B5-O05 FallingBlock | REPRODUCES (Batch 0 table, line 30) | B5-O05 |
| B5-O06 Dispenser | REPRODUCES (Batch 0 table, line 212) | B5-O06 |
| B5-O07 SectionMesh | REPRODUCES (Batch 0 table, line 84) | B5-O07 |
| B5-O08 BreakingModel | REPRODUCES (Batch 0 table, line 12) | B5-O08 |
| B5-O09 FaceSupport | REPRODUCES (Batch 0 table, line 45) | B5-O09 |

### 1.21.7 defect re-checks (15 behaviour + 7 oracle — all decided)

| 1.21.7 item | 26.1.2 verdict | Notes |
|---|---|---|
| B5-G01 farmland | EVAPORATES | 26.1.2's farmland uses `FarmlandBlock` (mapped, `classes.rs:151`) + `#maintains_farmland` (present); the 1.21.7 defect was the absent `#convertible_to_mud` tag in the 1.21.7 export — different mechanism. Port matches vanilla 26.1.2. |
| B5-G02 grass/mycelium | EVAPORATES | 26.1.2 chains map via `SpreadingSnowyBlock`/`SnowyBlock` (mapped, `classes.rs:166-167`); grass-spread logic matches vanilla 26.1.2. |
| B5-G03 dirt path | REPRODUCES | Same class-name mismatch (`DirtPathBlock` vs `PathBlock`) → B5-G03. |
| B5-G04 fungi/roots | EVAPORATES | 26.1.2 chains map (`NetherFungusBlock`→`VegetationBlock`, mapped `classes.rs:200/202`); support tags `#supports_crimson_fungus` etc. present and read. |
| B5-G05 stems | EVAPORATES | 26.1.2 stem chains map; `stemSupportBlocks`/`fruitSupportBlocks` fields present in prepared data and read. |
| B5-G06 crops | EVAPORATES | Crop growth/survival matches vanilla 26.1.2 (`CropBlock.getGrowthSpeed` reads `#grows_crops`, present; farmland tag present, moisture logic verified). |
| B5-G07 mushrooms | EVAPORATES | Mushroom spread/survival matches (vanilla `MushroomBlock.canSurvive` reads `#overrides_mushroom_light_requirement`, present); only the bonemeal height gate has the nesting bug (B5-G15, a different check). |
| B5-G08 snow layer | EVAPORATES | `#cannot_support_snow_layer`/`#support_override_snow_layer` present (vanilla reads both, javap-verified); survival matches vanilla 26.1.2. |
| B5-G09 seagrass | EVAPORATES | `#cannot_support_seagrass` present (vanilla reads it); survival matches. |
| B5-G10 kelp | EVAPORATES | `#cannot_support_kelp` present (vanilla reads it); survival matches. |
| B5-G11 cactus flowers | EVAPORATES | `#support_override_cactus_flower` present (vanilla reads it); survival matches. |
| B5-G12 fletching table | EVAPORATES | No `FletchingTableBlock` in 26.1.2 (chain `['Block']`); vanilla 26.1.2's fletching table is a plain block with no special behaviour — port matches. |
| B5-G13 dripstone | REPRODUCES | Same two causes → B5-G13. |
| B5-G14 tree growth water | SPLIT — single path EVAPORATES, mega path REPRODUCES | Vanilla 26.1.2's SINGLE path leaves the fluid's legacy block (offsets 371–388: `getFluidState().createLegacyBlock()` + flags 260 — water is vanilla's form there); the MEGA path sets air. The port leaves water on BOTH → mega-path defect is B5-G34. |
| B5-G15 bonemeal validity | REPRODUCES (both directions) | The 26.1 over-permissive form persists (B5-G15 nesting bug) AND the 1.21.7 over-strict form returns (B5-G29 `canGrow` gate). Both live at once. |

| 1.21.7 oracle | 26.1.2 verdict | Notes |
|---|---|---|
| B5-O01 Cauldron | REPRODUCES | Same cause → B5-O01. |
| B5-O02 Portal | Cause EVAPORATES; cascade remains (no item) | `getRelativePortalPosition` EXISTS in 26.1.2; the oracle still fails via the `VanillaBlockUpdateOracle` cascade. |
| B5-O03 PickBlockData | EVAPORATES | PASS all classpaths in the Batch 0 table. |
| B5-O04 ReplaceBlock | REPRODUCES | The re-port reintroduced the exact 1.21.7 cause (`Feature.DIRECT_CODEC`) → B5-O04. |
| B5-O05 FallingBlock | 1.21.7 cause EVAPORATES; oracle still fails on 26.1 cause | Debug-package cause gone; now constructor mismatch → B5-O05. |
| B5-O06 Dispenser | 1.21.7 cause EVAPORATES; oracle still fails on 26.1 cause | GameRules-package cause gone; now `EntityTypes` missing-package → B5-O06. |
| B5-O07 SectionMesh/BreakingModel | 1.21.7 causes EVAPORATE; both still fail on 26.1 causes | Splits into B5-O07 (`MaterialBaker` interface) + B5-O08 (`BlockModelFeatureRenderer` absent). |

## §6 Systemic findings

### 1. The 26.1.2 tree inherits the 26.1.1 tree's stale re-port wholesale (provenance confirmed, not assumed)

The coordinator's Step 0 provenance claim is verified in B5 scope by
direct measurement: `level/` is byte-identical between the 26.1.1 and
26.1.2 trees, every B5 game-layer file is byte-identical, and every
in-scope vanilla class is byte-identical between the jars. The 26.1.2
run therefore reproduces the 26.1.1 run's findings by construction —
and the 26.1.1 tree's own diagnosis stands: it was re-ported from the
26.2/26.3 form and dropped the 26.1 tree's deliberate 26.1-form fixes
(Batch 2's §6 finding, confirmed in B5 scope in 26.1.1 and unchanged
here). The 26.1 tree remains the repair reference for all twenty-one
behaviour gaps: each fix note says "restore the 26.1 form", and the
26.1 tree's own comments name each distinction.

### 2. The absent-tag root cause is unchanged — and still silent

The 26.1.2 tree reads the same absent tags as the 26.1.1 tree:
`#speleothems` (G13), `#nether_portal_frame` (G20),
`#blocks_motion_in_heightmap` / `#blocks_motion_in_heightmap_no_leaves`
(G21, incl. the `feature_world` site), `#convertible_to_mud` (G19,
wrong spelling — the 'a' form exists), `#blocks_motion` (G24),
`#blocks_fluid_flow` (render/mesher + fluids — B1/render scope),
`#sculk_growth_inhibitors` in `level/sculk.rs:620` (B4 scope, see
Cross-references). The tag-table construction resolves absent tags to
EMPTY sets (`data.rs:396` `.unwrap_or_default()`), and `tag()` returns
`Option` (`mod.rs:941-942`), so every absent-tag read silently
degrades to "false" — never an error, never a log line. The leakage
audit exists for exactly this pattern; these reads should be audited
against the 26.1.2 tag inventory (248 tags) rather than the 26.3 one.

### 3. The composter is the only whole-block regression

Twenty of the twenty-one behaviour gaps are edge-case divergences (one
method, one code path). G22 alone removes an entire block's
interaction: no item can be composted by any path. It is also the only
gap whose root cause is a data-model change (the 26.3
`minecraft:compostable` component) rather than a method-form change —
and the only one whose fix requires restoring a data table
(`compost_chance`, 115 entries) rather than a few lines.

### 4. NEW 26.1.1-record correction (for the 26.1.2 summary's corrections section)

**B5-G18 (bell) never existed — in 26.1.2, 26.1.1, 26.1, or (by the
same misread) 26.2.** The 26.2 run misread the `ifle` branch direction
in `BellBlock.isProperHit`, concluding vanilla requires `click_y >
0.8124`; the 26.1 and 26.1.1 runs carried the claim forward (each
reporting the bytecode as "read fresh in full"). The 26.1.2 run
re-read the bytecode and decompiled the class: vanilla requires
`click_y <= 0.8124`, exactly what every port tree implements. The
26.1 and 26.1.1 ledgers carry G18 as a live minor gap and both need
this correction; the 26.2 record should be re-checked by whoever owns
it (this batch verified only the 26.1.2 jar — `BellBlock` is
byte-identical across 26.1/26.1.1/26.1.2, but 26.2's jar was not
compared here). Lesson for future runs: a `dcmpl` + `ifle` pair
branches on the LESSER side — when a report's claim turns on a branch
direction, decompile the method rather than trusting the javap read
alone.

### Cross-referenced gaps (not re-minted)

- **Sculk inhibitors in `level/sculk.rs:620`** — reads the absent
  `#sculk_growth_inhibitors` tag. Batch 6 minted the worldgen
  instance (B6-G07) and explicitly routed the level instance to B4;
  recorded here only as a cross-reference, NOT minted.
- **`update.rs` `ensure_block_entity` re-check** — the 26.1.1 report
  assigned it to B4; the 26.1.2 diff confirms the re-check is still
  present. B4's scope.
- **Campfire douse** (`game/campfire_douse.rs` reads the absent
  `#douses_campfires` tag) — Batch 7's scope (its brief lists the
  1.21.7 G08 shovel campfire douse).
- **`can_hold_any_fluid` / lava-fire walk** — Batch 1 minted (B1-G05,
  B1-G06). Sponge does NOT use `can_hold_any_fluid` (see §7) and
  stays a pass.
- **Diode guard** — B2-G07. **Redstone wire unmapped** — B2-G01.
  **PushReaction old names** — B2-G02.
- **B3-G22 startup blocker** (`#causes_suffocation` /
  `#suppresses_bounce`) — Batch 3's. Both tags verified absent from
  the 26.1.2 inventory (fresh census).
- **Fire spread ignoring doFireTick (Batch 1's G4) — EVAPORATES for
  26.1.2, as it did for 26.1.1.** There is NO `doFireTick` gamerule
  in vanilla 26.1.2 (59 `GameRule`s; the only fire-related ones are
  `FIRE_DAMAGE` and `FIRE_SPREAD_RADIUS_AROUND_PLAYER`; the only
  mention of doFireTick in the jar is `GameRuleRegistryFix`).
  Flagged for Batch 1's re-examination.
- **Dripstone cauldron fill** — Batch 1's G6. **Loot tables behind
  `drop_resources`** — Batch 3's. **`TreeFeatureOracle`** — Batch 6's.
  **`CushionPlacementOracle`** — entity-decoration scope.
- **`blocks_fluid_flow` reads** — only in render assets/mesher and
  arrows/entities fluids, never in `level/`; B1/render scope, not B5.
- **`feature_world` heightmap tags** — folded into G21's evidence as
  an additional site of the same root cause (same absent tag, same
  fix); NOT minted separately. Batch 6's landed report does not cover
  it and Batch 1 flagged it as worldgen scope.
- **The twelve 26.1 regressions** (B1-G05, B3-G14, B4-G01, B4-G02,
  B7-G07, B7-G08, B9-G03/G05/G07b/G12/G13) — owned by Batches 1, 3,
  4, 7 and 9; not re-decided here. Their code is byte-identical
  between the 26.1.1 and 26.1.2 trees (coordinator Step 0).
- **`chunk.rs` saved-palette legacy-form removal** (26.1.2 tree
  change, observed in the tree diff): the 26.1.1 tree's
  `from_palette_nbt` accepted the `Name`/`Properties` palette form
  26.1.1 saves write; the 26.1.2 tree reads only the modern
  `id`/`properties` form. Save-loading scope, not block behaviour —
  flagged here, not minted. Whoever owns save migration should
  confirm 26.1.1 saves still load.
- **Port-delta items #4 (`handleAttack`) and #6 (Checkbox tooltip)**
  — the delta's only behaviour changes; attack handling and GUI,
  both outside B5 scope. The `handleAttack` jar change is a head
  guard (`hasClientLoaded` → the 26.1.2 form, per
  PORT-NOTES-26.1.2.md); no B5 verdict touches it.

## §7 Passes by area, pilot leftovers, drivers, evidence

### Passes — ticks (29/37)

Fire, cactus, sugar cane, bamboo, vine, kelp, seagrass, turtle egg,
sniffer egg, cauldron (tick), frosted ice, concrete powder, sand,
gravel, anvil, dragon egg (falling), chorus plant, nether wart, cocoa,
sweet berry, pumpkin/melon stem, crop, farmland (tick), grass/mycelium
spread, mushroom spread, snow layer (survival tick), ice (freeze tick),
obsidian (nether portal frame tick — N/A, see G20), bell (powered
tick), dispenser (scheduled), observer (scheduled), redstone wire
(scheduled — see B2-G01 for the wire class), piston (scheduled — see
B2), sponge (placement, not tick), bed (no tick), composter tick
(level-7→8 by command — insertion is G22). All pass because the jar
methods are byte-identical to 26.1.1's and the deciding port code
(`tick.rs`, `survive.rs`, `shapes.rs`, `classes.rs`) is byte-identical
to the 26.1.1 tree's, with the census confirming the class chains and
tags the checks depend on. Specific fresh verifications: `IceBlock`
melt has zero `gameEvent` refs (G23 is the port adding one); the
turtle-egg trample code is present at `motion.rs:1620`,
`mobs/ai.rs:857-860`, `armor_stands.rs:1153-1185` (`block_landing.rs`
byte-identical); `sponge_remove_water`
(`hooks.rs:618`) uses `is_water_fluid`/`pickup_block`/Liquid checks,
NOT `can_hold_any_fluid`, so the sponge pass stands despite B1-G05.

### Passes — survival/support (26/28)

All 26.1.1 passes stand: farmland, grass/mycelium, dirt path (survival
form — the REVERT is G03, the survive predicate itself is fine),
nether fungi/roots, stems, crops, mushrooms, snow layer, seagrass,
kelp, cactus flower, chorus plant, chorus flower, bell, sapling,
azalea, bamboo, sugar cane, vine, cocoa, sweet berry, nether wart,
turtle egg, sniffer egg, dripstone-adjacent (see G13), bed (survival
form — the USE is G27). The support tags the checks depend on are all
present in the 26.1.2 tag inventory (verified fresh): `#maintains_farmland`,
`#grows_crops`, `#supports_nether_wart`, `#supports_crimson/warped_fungus`,
`#supports_crimson/warped_roots`, `#supports_stem_crops`,
`#supports_stem_fruit`, `#supports_pumpkin/melon_stem`,
`#supports_pumpkin/melon_stem_fruit`,
`#supports_vegetation/cactus/sugar_cane/bamboo/chorus_flower/chorus_plant/
hanging_mangrove_propagule/mangrove_propagule`, `#cannot_support_snow_layer`,
`#support_override_snow_layer`, `#cannot_support_seagrass`,
`#cannot_support_kelp`, `#support_override_cactus_flower`,
`#overrides_mushroom_light_requirement`, `#prevents_nearby_leaf_decay`,
`#sniffer_egg_hatch_boost`. The two gaps are G03 and G13 (both
survival-form consequences of the same root causes).

### Passes — updates (14/16)

All 26.1.1 passes stand: neighbour-change propagation, `updateShape`
for every in-scope block, redstone power updates (see B2), the
`ensure_block_entity` re-check is B4's scope (see Cross-references).
The two gaps are G03 and G13 (updateShape arms dead for the same
reasons). `update.rs` is byte-identical to the 26.1.1 tree's.

### Passes — placement (9/11)

All 26.1.1 passes stand: block placement, waterlogging, the
`canSurvive` gate at placement, face-attached placement, the
`getStateForPlacement` forms for every in-scope block. The two gaps
are G03 and G13 (placement arms dead for the same reasons). `place.rs`
is byte-identical to the 26.1.1 tree's.

### Passes — interaction (14/20)

Pick-block (data + item), replace-block (placement replace rules),
door/gate/trapdoor use, chest/barrel use, furnace use, crafting table
use, anvil use, enchanting table use, beacon use, brewing stand use,
lectern use, jukebox use, note block use, lever/button use, pressure
plate use, tripwire hook use (the USE is fine — the re-check is G25),
bell use (the USE is fine — and the hit test is now a PASS, see the
G18 correction), bed use form (the PAIR CHECK is G27), dragon egg use
form (the BORDER is G28), composter use form (the ITEM TABLE is G22),
mud conversion form (the TAG is G19), portal use form (the FRAME is
G20). The six gaps are the bonemeal-growth check (G15/G29–G32), G19,
G20, G22, G27, G28.

### Passes — breaking/removal (11/14)

All 26.1.1 passes stand except the three gaps: `drop_resources` form
(loot tables are Batch 3's), block-break events (the 2001 event form
matches; the door `!correct_tool` extra 2001 event is a carried nit,
not a defect — see Nits), `destroy_progress` is G35, ice mined is
G24, tripwire break is G25's family. `block_breaking.rs` and
`block_landing.rs` are byte-identical to the 26.1.1 tree's.

### Passes — structural (0/2)

Both structural checks gap, as in 26.1.1 — and the tag inventory is
unchanged (the port reads the same absent tags; see §6.2).
Class-chain census: 1,168 blocks, 29,873 states, 291 distinct class
names, 28 unmapped (same 28 as 26.1.1, including `DirtPathBlock`,
`RedStoneWireBlock`, `AmethystBlock`); 3 blocks fall back to plain
`Block` despite a non-`Block` chain (dirt_path, redstone_wire,
amethyst_block); 87 blocks with an unrecognised most-derived class but
a recognised ancestor (same as 26.1.1). Tag inventory: 248 tags;
absent (verified): `#speleothems`, `#nether_portal_frame`,
`#blocks_motion_in_heightmap`, `#blocks_motion_in_heightmap_no_leaves`,
`#convertible_to_mud` ('i'), `#blocks_motion`, `#blocks_fluid_flow`,
`#washed_away_by_fluids`, `#blocks_lava_fire_spread`,
`#sculk_growth_inhibitors`, `#douses_campfires`,
`#mushroom_grow_block`, `#snow_layer_cannot_survive_on`,
`#snow_layer_can_survive_on`, `#causes_suffocation`,
`#suppresses_bounce`.

### Nits recorded as passes, not defects (carried from 26.1.1, re-verified)

- Door `!correct_tool` extra 2001 event (unobservable in practice).
- Sniffer-egg hardcoded `moss_block` (matches the shipped data).
- Turtle-egg/eyeblossom hardcoded overworld timelines (match vanilla
  26.1.2 timeline data).
- Tripwire disarm flags 260 vs vanilla 3 (unobservable).
- Bamboo dead `bamboo_height_above` query (no caller).
- Sleep-height shape-top form (equivalent for all 26.1.2 beds — see
  G27's note).
- Poplar grower (dead code — no `poplar_sapling` in the export).
- The `bonemeal.rs` `place_inline_feature` API (out of B5 scope —
  noted only).

### Pilot leftovers diagnosed

None outstanding for Batch 5. The 26.1 pilot's B5 leftovers (the
composter table, the TreeGrower draw form, the ice rules) were all
decided in the 26.1 run and re-verified here — and the 26.1.2 tree
still carries the 26.1.1 regressions on all three (G22, G33/G34,
G23/G24). No check in this batch is left undecided.

### Drivers and oracle status

No driver runs per the brief (static checks only). Oracle status is
from the Batch 0 measured table (`docs/sweep/26.1.2/batch-0-oracles.json`,
391 oracles, 202 fail-all / 158 pass-all / 31 mixed):
in-scope oracles — 7 PASS all, 7 FAIL all (minted as B5-O01/O04–O09),
1 cascade (B5-O02, no item), 1 evaporated (B5-O03). The oracle sources
in the 26.1.2 tree are byte-identical to the 26.1.1 tree's.

### Evidence index

Batch evidence dir: `~/workspace/mcir-work/sweep-26.1.2-batches/b05/evidence/`

- `jar-compare.py` + `jar-compare.log` + `absent-both.txt` — FULL
  sha1 compare of every class entry between the 26.1.1 and 26.1.2
  jars (server-game: 3 changed of 7,351; client: 8 changed of
  10,682; none in B5 scope); 142 distinct in-scope B5 class names
  checked (122 identical, 0 changed, 20 absent-both).
- `tree-diff-summary.txt` — the full 26.1.1-vs-26.1.2 tree diff
  (32 differing files + 5 only-in entries); `level/` byte-identical,
  all B5 game-layer files identical, oracle sources identical.
- `census.py` + `census.log` — class-chain census (1,168 blocks, 28
  unmapped, 3 plain-`Block` fallbacks, 87 ancestor-recognised), tag
  inventory (248 tags, absent/present per tag), stem support fields,
  configured-feature JSON nesting, `compostable` component census
  (0/1,506 items), `worldgen/features.json` empty.
- `javap/` (43 files) — fresh `javap -c -p` dumps from the 26.1.2
  jars: `IceBlock`, `BedBlock`, `DragonEggBlock`, `EyeblossomBlock`,
  `TreeGrower`, `SaplingBlock`, `AzaleaBlock`, `MushroomBlock`,
  `SweetBerryBushBlock`, `BoneMealItem`, `GrassBlock`, `ComposterBlock`,
  `TripWireHookBlock`, `TripWireBlock`, `PortalShape`, `Heightmap`
  (`-v` for the bootstrap method), `PotionItem`, `ChorusFlowerBlock`,
  `BellBlock`, `Player`, `ChunkMap`, `MaterialBaker`, `Feature`,
  `ConfiguredFeature`, `LevelEvent` (`-constants`), `LivingEntity`,
  `PointedDripstoneBlock`, `DirtPathBlock`, `GameRules` (59 rules, no
  `doFireTick`), `BlockTags`, `CubeVoxelShape`, `FarmlandBlock`,
  `SpreadingSnowyBlock`, `SnowyBlock`, `StemBlock`, `CropBlock`,
  `SnowLayerBlock`, `SeagrassBlock`, `KelpBlock`, `CactusFlowerBlock`,
  `NetherFungusBlock`, `NetherRootsBlock`, `BambooStalkBlock`.
- `decompile/bellout/BellBlock.java` — Vineflower 1.12.0
  decompilation of vanilla 26.1.2's `BellBlock` (the G18
  arbitration: `isProperHit` requires `!(clickY > 0.8124F)`).

Key port-side line numbers (26.1.2 tree,
`versions/26.1.2/crates/horizon-world/src/level/` unless noted) —
all verified fresh; identical to 26.1.1's:

- G03: `classes.rs:208` (`Path = "PathBlock"` only); `tick.rs:202`.
- G13: `speleothem.rs:15`, `falling.rs:55`, `place.rs:1407`,
  `shapes.rs:1189` (tag reads); `tick.rs:248`/`434`, `survive.rs:307`,
  `shapes.rs:728`, `place.rs:1156` (dead dispatch arms);
  `speleothem.rs:104-107` (latent G16 water-source check).
- G15: `bonemeal.rs:326-360` (`bonemeal_valid`), `:353`
  (`foliage_radius` top-level); `growth.rs:446-453`
  (`tree_minimum_height` top-level); `worldgen/live.rs:234`
  (`configured_json` returns the whole feature JSON).
- G17: `tick.rs:1430` (`above_pos.y > max_y() - 1`).
- G18 (CLOSED): `redstone.rs:1442` (`click_y <= 0.8124f32` — CORRECT);
  callers `interact.rs:424`, `mod.rs:1363` (within-block Y).
- G19: `data.rs:460`, `horizon-client/src/game/potions.rs:710`,
  `horizon-client/src/game/remote.rs:8658`,
  `horizon-client/src/game/dispenser.rs:2245` (all 'i' spelling).
- G20: `portal.rs:40-44` (`portal_frame`), `:67`, `:131`;
  `hooks.rs:588-597` (`base_fire_on_place`).
- G21: `weather.rs:41-59` (`motion_blocking_top`), `:127`
  (`is_raining_at`); `feature_world.rs:350-360` (same-root-cause
  site); `lightning.rs:199`, `portal_travel.rs:506` (consumers).
- G22: `horizon-items/src/items.rs:1015` (component read);
  `game/composter.rs:29-38` (all paths return `None`);
  `level/interact.rs:559-633` (no `composter_add_item`).
- G23: `hooks.rs:691-706` (`ice_melt`, event line at `:706`).
- G24: `hooks.rs:525-541` (`ice_destroyed_by_player`, tag read at
  `:539`); `data.rs:396` (absent tags → empty sets);
  `mod.rs:941-942` (`tag()` returns `Option`).
- G25: `rails.rs:593-597` (`else if was` arm at `:595-596`).
- G26: `timeline.rs:9-23` (`box_scan_order`), `:92`
  (`eyeblossom_tick`).
- G27: `interact.rs:367-384` (pair check); `game/beds.rs`
  (`sleep_height` shape-top form — NOT a defect).
- G28: `interact.rs:579-600` (`dragon_egg_teleport`, no border check).
- G29: `bonemeal.rs:335` (`tree_can_grow` gate); `growth.rs:425-444`
  (`tree_can_grow`); `growth_tests.rs:234`
  (`lone_dark_oak_sapling_cannot_grow`).
- G30: `bonemeal.rs:406` (age-only sweet berry arm).
- G31: `bonemeal.rs:216-243` (`bone_meal_succeeds_on_client`).
- G32: `bonemeal.rs:1224-1234` (`grass_bonemeal_effect`, `next_float`).
- G33: `growth.rs:145-269` (weighted lists), `:249-264`
  (`weighted()`); `:232-240` (dead poplar grower).
- G34: `growth.rs:375-382` (`remove_sapling`, 818), `:384-419`
  (`grow_tree`); `mod.rs:164` (`UPDATE_NONE = 260`).
- G35: `game/block_breaking.rs:97-108` (`destroy_progress_event`),
  `game.rs:5294`/`5306` (call sites), `game.rs:4298-4306`
  (`breaking_block_effects`), `game/lan/level_event_targets.rs:28-29`.

## §8 Verdict

**Batch 5 (26.1.2): 128 checks, 103 pass, 25 gap; 29 items (21
behaviour gaps — 4 major, 17 minor — plus 7 oracle defects, 1 major,
plus 1 closed item).** Twenty-eight of the 26.1.1 run's twenty-nine
items reproduce unchanged — the 26.1.2 tree is a delta port with
`level/` byte-identical and every in-scope vanilla class
byte-identical, so the 26.1.1 defects carry over wholesale and the
7-item port delta adds nothing in B5 scope. The one change is a
correction, not a fix: B5-G18 (bell `isProperHit`) NEVER EXISTED —
the 26.2 run misread the `ifle` branch direction, the misread
propagated through the 26.1 and 26.1.1 runs, and the port was correct
all along. The 26.1 and 26.1.1 ledgers need that correction applied.

The batch is complete: all checks decided statically, all 26.1.1 and
1.21.7 re-checks decided, no needs-harness items, nothing stalled.
The one thing the parent should know: the 26.1.2 tree's own tests
still assert the wrong behaviour in at least two places
(`growth_tests.rs`: lone dark oak refusal, client bonemeal success),
so a fix wave that restores the 26.1 form will need to restore the
26.1 tests too — and the G18 correction should reach the 26.1 and
26.1.1 ledgers before anyone "fixes" a bell that was never broken.
