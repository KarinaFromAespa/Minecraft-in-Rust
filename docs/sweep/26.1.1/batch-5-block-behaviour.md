# Batch 5 — Block behaviour (26.1.1 parity sweep)

Checks + triage only, against VANILLA 26.1.1's own jar and data (named
server-game jar `~/workspace/mcir-work/26.1.x/versions/26.1.1/jars/server-game.jar`,
sha1 `63e2f8ca1ccbcfda2f72ad0e8278f975d5a6a011`; client game jar
`.../jars/client.jar`; prepared 26.1.1 data in
`versions/26.1.1/.local/horizon-client/26.1.1/`). Static checks only: javap
on the named jars, Rust source comparison, vanilla/prepared data. No fixes,
no cargo builds, no driver runs, no git commands in the worktree.

**Headline:** all eight 26.1 behaviour defects and all six 26.1 oracle
defects REPRODUCE in 26.1.1 — and the 26.1.1 tree adds fourteen NEW
behaviour defects and one NEW oracle defect on top, all of the same
stale-carry class Batch 2 proved for redstone: the 26.1.1 tree was
re-ported from the 26.2/26.3 form and dropped the 26.1 tree's deliberate
26.1-form fixes. The worst new defect is a dead composter (B5-G22, MAJOR):
the port reads a `minecraft:compostable` component no 26.1.1 item carries,
so nothing can ever be composted — by hand, hopper, or villager. The rest
are one-per-method reversions: ice melt gains a 26.3 game event, mined ice
loses its water, tripwire gains a 1-tick re-check, eyeblossom scans in
section order, bed use gains a pair check, dragon eggs ignore the world
border, bonemeal gains the 26.3 `canGrow` gate while losing the sweet-berry
gates and the client PASS, grass bonemeal rolls the wrong RNG, TreeGrower
is rewritten in weighted-list form with a water-leaving mega path, and
digging broadcasts level events 2019/2020 that vanilla 26.1.1 never sends.
Four majors in total (G13, G20, G21, G22), eighteen minors, plus seven
oracle defects (one major, O09).

## §0 Execution status

Checks + triage only. No file under `versions/26.1.1/` was modified,
nothing was committed, no git commands ran in the worktree, no cargo
builds ran. Everything below is static analysis: `javap -c -p` on the
named 26.1.1 jars, source comparison against `versions/26.1.1/crates/`,
a full `level/` diff against the 26.1 tree in the same worktree, the
prepared 26.1.1 data (Batch 0's prepare: `block-behaviours.json`,
`gameplay-facts.json`, `block-tags.json`, `items.json`, configured
features), and the 26.1.1 Batch 0 oracle measurement
(`docs/sweep/26.1.1/batch-0-oracles.json`, 391 oracles, 189 pass / 202
fail-all).

**Tally: 128 behaviour checks — 102 pass, 26 gap** (26.1: 127 checks,
111 pass, 16 gap). Behaviour/non-behaviour split: **128 behaviour + 0
non-behaviour**. Batch 5 is not one of the three as-tallied exception
batches; oracle triage is counted separately below. The +1 check vs
26.1 is the destroy-progress-events check (B5-G35), a behaviour surface
the 26.1 tree did not have at all. Nine 26.1 passes flip to gaps (ice
melt, sapling advance, eyeblossom, composter tick, composter use, dragon
egg, bed use, ice mined, tripwire); no 26.1 gap closes.

**Items: 29** — 22 behaviour (4 major, 18 minor) + 7 oracle defects
(1 major, 6 minor). 8 behaviour + 6 oracle items reproduce 26.1's;
14 behaviour + 1 oracle item are NEW for 26.1.1.

**26.1 re-checks: 14/14 decided, all REPRODUCE.** No 26.1 item was
FIXED in the 26.1.1 tree. **1.21.7 re-checks: 15 behaviour + 7 oracle,
all decided** — G03 and G13 reproduce, G15 reproduces in BOTH directions
(the 26.1 permissive form persists AND the 1.21.7 over-strict form
returns), G14 splits (single path evaporates, mega path reproduces),
G01/G02/G04–G12 evaporate (11); oracle O01 reproduces, O02's cause
evaporates (cascade only), O03 evaporates (PASS), O04 reproduces (the
re-port reintroduced its exact 1.21.7 cause), O05/O06/O07's 1.21.7 causes
evaporate but the oracles still fail on the 26.1 causes.

## §1 Scope

Block ticks, interaction, survival/support, updates, breaking/removal,
placement — the general block-behaviour surface: farmland, grass/mycelium
spread, dirt paths, nether fungi/roots, stems/crops, mushrooms, snow
layers, seagrass/kelp, cactus flowers, fletching table, dripstone,
saplings/bonemeal, trees, bells, chorus, portals, pick-block, replace,
plus the composter, ice, tripwire, eyeblossom, beds, dragon eggs and
mining feedback that the tree diff put in scope. Method: javap on the
named 26.1.1 jars + source comparison + prepared data
(`block-behaviours.json` class chains, `block-tags.json`, `items.json`,
configured-feature JSON). No cargo builds, no driver runs.

## §2 Method

Three verification layers, all fresh against 26.1.1 evidence:

1. **Jar identity proof.** 94 in-scope classes were byte-compared
   (sha1 of the class file) between the 26.1 and 26.1.1 server-game
   jars — all 94 IDENTICAL, including `TreeGrower`, `IceBlock`,
   `BedBlock`, `DragonEggBlock`, `EyeblossomBlock`, `BoneMealItem`,
   `GrassBlock`, `ComposterBlock`, `TripWireHookBlock`, `DirtPathBlock`,
   `PointedDripstoneBlock`, `PortalShape`, `Heightmap`, `PotionItem`,
   `ChorusFlowerBlock`, `BellBlock`, `SweetBerryBushBlock`,
   `SaplingBlock`, `AzaleaBlock`, `MushroomBlock`, `RedStoneWireBlock`,
   `ChunkMap`, `Player`, `Feature`, `ConfiguredFeature` — and the
   absences too (`SpeleothemBlock`, `FletchingTableBlock`, `FarmBlock`,
   `SulfurSpikeBlock`, `RedstoneWireBlock`, `EntityTypes`,
   `net.minecraft.util.Prediction` are absent from BOTH jars). Four
   client-jar classes were compared the same way (`MaterialBaker`,
   `BlockModelFeatureRenderer` absent in both, `BlockFeatureRenderer`,
   `CubeVoxelShape`). This is consistent with the documented
   26.1→26.1.1 delta (3 changed classes: `PlayerEntry.<init>`,
   `DetectedVersion.createBuiltIn`, `SharedConstants` version fields —
   none in Batch 5 scope). Every jar-side verification from the 26.1
   run therefore carries to 26.1.1 by identity, not by assumption —
   and every defect-bearing method was ALSO re-dumped fresh from the
   26.1.1 jar (see §7 evidence index).
2. **Tree diff.** A full diff of `crates/horizon-world/src/level/`
   between the 26.1 and 26.1.1 trees in this worktree (15 files differ,
   1,098 lines; `evidence/level-diff-26.1-vs-26.1.1.txt`). In-scope
   files `classes.rs`, `tick.rs`, `survive.rs` (comment only),
   `shapes.rs`, `place.rs`, `portal.rs`, `weather.rs`, `speleothem.rs`,
   `falling.rs`, `tests.rs` are byte-identical (or comment-only);
   `bonemeal.rs`, `growth.rs`, `hooks.rs`, `interact.rs`, `rails.rs`,
   `timeline.rs`, `data.rs`, `mod.rs`, `feature_world.rs`, `fluids.rs`,
   `sculk.rs`, `update.rs`, `redstone.rs` differ. The game-layer files
   the 26.1 report cites were diffed individually (`game/composter.rs`,
   `game/beds.rs`, `game/potions.rs`, `game/dispenser.rs`,
   `game/remote.rs`, `game/block_breaking.rs`, `game/armor_stands.rs`,
   `mobs/ai.rs`, `motion.rs`, `block_landing.rs` — the last byte-identical).
   The 102 pass checks rest on (1)+(2): the jar methods are the ones
   the 26.1 run verified, and the port code deciding each check is
   unchanged from the 26.1 tree except where this report says otherwise.
3. **Fresh data checks.** The class-chain census (1,168 blocks,
   291 distinct class names, 28 unmapped — identical to 26.1), the tag
   inventory (248 tags; absent/present re-verified per tag), the stem
   support fields, the configured-feature JSON nesting (oak/spruce/
   mushrooms), and the `compostable` component census (0/1,506 items)
   were all re-run against the prepared 26.1.1 data
   (`evidence/census.py`, log in `evidence/`).

**Counting rule:** one check = one vanilla-method/behaviour comparison
(jar + port source, or jar + prepared data). One item may sit behind
several checks (G03, G13) and one check may carry several items (the
bonemeal-growth check carries G15 + G29 + G30 + G31 + G32). Oracle/
driver-path checks are not in the behaviour tally.

**Note on the 26.1 summary's "26.2-record corrections" section**
(`sweep-26.1.1-ref/26.1/summary.md`, line 283): read and applied. None
of its three corrections touches Batch 5 scope — B3-G27/G28/G25/G26
(horse food, furnace minecart, BreedGoal) is Batch 3's, B9-G11 (recipe
suggestions) is Batch 9's, and the B6 note (feature_tags folder,
`dripstone_cluster` naming) is Batch 6's. No B5 verdict changes.

## §3 Checks table

| Family | Checks | Pass | Gap |
|---|---:|---:|---:|
| Random/scheduled ticks | 37 | 29 | 8 (B5-G03, B5-G13, B5-G17, B5-G21, B5-G22, B5-G23, B5-G26, B5-G33/G34) |
| Survival/support | 28 | 26 | 2 (B5-G03, B5-G13) |
| Updates | 16 | 14 | 2 (B5-G03, B5-G13) |
| Placement | 11 | 9 | 2 (B5-G03, B5-G13) |
| Interaction | 20 | 13 | 7 (B5-G15/G29/G30/G31/G32, B5-G18, B5-G19, B5-G20, B5-G22, B5-G27, B5-G28) |
| Breaking/removal | 14 | 11 | 3 (B5-G24, B5-G25, B5-G35) |
| Structural (class chains, tags) | 2 | 0 | 2 (class-chain liveness, tag inventory) |
| **Total** | **128** | **102** | **26** |

Behaviour checks only; oracle triage is counted separately below. The
structural gaps are the systemic roots of G03/G13/G19/G20/G21/G22/G24 —
the checks are the census and the inventory themselves, not a re-count
of the defects. G03 and G13 each appear in four families because each is
a distinct code path (tick, survive, updateShape, placement) sharing one
root cause. The bonemeal-growth interaction check carries five items
(G15 + G29–G32); the sapling-advance tick check carries two (G33, G34).
Breaking has 14 checks (26.1 had 13): the destroy-progress-events check
is new, a behaviour surface the 26.1 tree did not have.

4 gaps are MAJOR (G13, G20, G21, G22), 18 are MINOR (G03, G15, G17, G18,
G19, G23–G35 except G22). No B5-H (needs-harness) items: prepared data
landed before this batch, so every check in scope was decided statically.

Oracle defects: 7 (B5-O01, O04, O05, O06, O07, O08, O09) — 6 minor,
1 major (O09, FaceSupport, blocks a driver family). B5-O02 is a cascade
with no independent defect (do not mint); B5-O03 evaporates (PASS all
classpaths in the measured Batch 0 table).

## §4 Defects

### Reproduced 26.1 defects (all verified fresh — jar byte-identical, port code byte-identical unless noted)

### B5-G03 — Dirt path never reverts: `DirtPathBlock` unmapped in `classes.rs` (MINOR, REPRODUCES)

26.1.1's jar HAS `DirtPathBlock` (no `PathBlock`); the export chain is
`['DirtPathBlock','Block']`. But the 26.1.1 tree's `classes.rs:208`
(byte-identical to 26.1's) reads `Path = "PathBlock"` only — the
`"DirtPathBlock"` alias 26.2's tree carries was dropped. The class
resolves to plain `Block`, so every path arm is dead: survive
(`survive.rs:273`), updateShape, scheduled tick (`tick.rs:202`,
unconditional `turn_to_base_block`), placement. Dirt paths never revert
to dirt when the block above becomes solid, and never revert on schedule.
Vanilla 26.1.1 `DirtPathBlock` behaviour itself is unchanged from 26.1
(byte-identical class). Fix note: add `"DirtPathBlock"` to the `Path`
alias (the 26.2 form), with the unit test 26.2's tree carries.

### B5-G13 — Dripstone fully dead: no `SpeleothemBlock` class in 26.1.1 AND `#speleothems` tag absent (MAJOR, REPRODUCES)

Two independent causes, either alone sufficient:

(a) **Class dispatch dead.** 26.1.1 has no `SpeleothemBlock` class
(`PointedDripstoneBlock` extends `Block` directly, javap-verified;
export chain `['PointedDripstoneBlock','Block']`). The `Class::Speleothem`
dispatch arms never fire: scheduled tick → `spawn_falling_stalactite`
(`tick.rs:248`), random tick → growth (`tick.rs:434`), survive
(`survive.rs:307`), updateShape (`shapes.rs:728`), placement
(`place.rs:1156`).

(b) **Tag predicates dead.** `#speleothems` is ABSENT from 26.1.1 data
(verified in `block-tags.json` and in the client jar — no
`speleothems.json`), so all tag predicates fail: `speleothem.rs:15`,
`falling.rs:55`, `place.rs:1407`, `shapes.rs:1189`. Vanilla 26.1.1 uses
hardcoded `Blocks.POINTED_DRIPSTONE` checks instead.

Observable: dripstone never grows, stalactites never fall,
`can_survive` defaults true (floating dripstone never pops), placement tip
direction/thickness never computed. The cauldron-fill half remains Batch 1's
G6. Fix note (for the fix agent): 26.1.1 needs the 26.2 pattern inverted —
dispatch on `PointedDripstoneBlock` directly (or add a `PointedDripstone`
class mapping) and replace the `#speleothems` tag reads with a hardcoded
pointed-dripstone check, matching vanilla 26.1.1's own form. While fixing,
also fix the latent B5-G16: vanilla 26.1.1 `PointedDripstoneBlock.canGrow`
requires the fluid two blocks above the stalactite start to be WATER **and a
source** (`isSource()`); the port (`speleothem.rs:104-107`) checks only
`fluid.kind() == FluidKind::Water`. Unobservable while growth is dead, so not
minted separately.

### B5-G15 — Bonemeal validity too permissive at the build ceiling: sapling/azalea/mushroom height gates read the wrong JSON level (MINOR, REPRODUCES)

Vanilla 26.1.1 HAS bonemeal height gates (unlike 1.21.7), re-verified
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
(`bonemeal.rs:353`) but it lives under `config` (brown = 3, red = 2).
The port always uses 2 — red correct by luck, brown's gate is pos.y+6
instead of pos.y+7 (one block too permissive). `isBonemealSuccess`
(sapling/azalea 0.45, mushroom/fungus 0.4) matches vanilla exactly.
Note: `worldgen/features.json` in the prepared data is EMPTY (`{}`); the
live features come from `data/minecraft/worldgen/configured_feature/*.json`
via `configured_json`. Fix note: read `config.trunk_placer.base_height`
and `config.foliage_radius` (or make `configured_json` return the `config`
object — its only other caller is the mushroom check). The opposite
direction (too strict) is the NEW item B5-G29 below — both directions
are live at once in 26.1.1.

### B5-G17 — Chorus flower never ticks at the build ceiling (MINOR, REPRODUCES)

Vanilla 26.1.1 `ChorusFlowerBlock.randomTick` proceeds when
`above.y <= getMaxY()` (320) — bytecode `if_icmple 28`, re-verified
fresh. The port (`tick.rs:1430`, `chorus_flower_random_tick`, file
byte-identical to 26.1's) returns early when
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

### B5-G18 — Bell `isProperHit` height comparison inverted (MINOR, REPRODUCES)

Vanilla 26.1.1 `BellBlock.isProperHit` (bytecode read fresh in full)
requires `click_y > 0.8123999834060669` — hit the UPPER part of the bell
— plus a non-Y face plus attachment/axis rules (floor: facing axis ==
hit axis; single/double wall: axes differ; ceiling: always). The port
(`redstone.rs:1442`, `bell_hit`, unchanged by the tree diff — the
`redstone.rs` diff touches only the diode guard, B2-G07) requires
`click_y <= 0.8124f32` — inverted. Hits at the top ring ring in vanilla
but not in the port; hits at the bottom ring ring in the port but not in
vanilla. The attachment/axis arms match; `bell_ring` (block event 1 +
sound volume 2.0 pitch 1.0 + BLOCK_CHANGE) matches vanilla
`attemptToRing` (null direction → facing fallback; the port passes
facing directly — same result). Bell `neighborChanged` (powered toggle
+ ring), placement, survive and updateShape all match.

### B5-G19 — Water bottle on dirt never makes mud: port reads `#convertible_to_mud`, 26.1.1's tag is `#convertable_to_mud` (MINOR, REPRODUCES)

The port reads `#convertible_to_mud` (with 'i') in four places:
`data.rs:460` (tag table), `horizon-client/src/game/potions.rs:710`
(water bottle use), `horizon-client/src/game/remote.rs:8658`,
`horizon-client/src/game/dispenser.rs:2245` (dispenser water bottle).
26.1.1's tag is `#convertable_to_mud` (with 'a') — verified three ways:
`BlockTags.CONVERTABLE_TO_MUD` in the jar,
`data/minecraft/tags/block/convertable_to_mud.json` in the client jar
(values dirt, coarse_dirt, rooted_dirt), and vanilla `PotionItem.useOn`
checks it (bytecode offset 57, re-verified fresh). The port's tag read
returns nothing, so the mud conversion never fires. 26.2's tree uses the
correct 'a' spelling; the 26.1.1 port wrongly took the 26.3 spelling
(the `data.rs` tag table has only the 'i' spelling — `convertable_to_mud`
appears nowhere in the table). Fix note: rename all four reads to
`convertable_to_mud`.

### B5-G20 — No nether portal can ever be created: `portal_frame` reads the absent `#nether_portal_frame` tag (MAJOR, REPRODUCES)

The port's `portal_frame()` (`portal.rs:40-44`, file byte-identical to
26.1's) reads `#nether_portal_frame`, which is ABSENT from 26.1.1 data
(verified in `block-tags.json`; no synthesis anywhere in the tree) →
always false. Vanilla 26.1.1 `PortalShape.FRAME` is hardcoded
`state.is(Blocks.OBSIDIAN)` (`lambda$static$0` in `PortalShape.txt`,
re-verified fresh — note the class lives at
`net.minecraft.world.level.portal.PortalShape`, not `.block.`).
Consequence: `find_portal_shape` (`portal.rs:67`) always returns
width/height 0, `find_empty_portal_shape` (`portal.rs:131`) always
returns None, and `hooks.rs:588-597` `base_fire_on_place` never creates
portal blocks — **no nether portal can ever be created in the 26.1.1
port**. 26.2's tree correctly hardcoded obsidian. Fix note: hardcode
obsidian, matching vanilla 26.1.1 and the 26.2 tree's form.

### B5-G21 — Snow never accumulates on land, ice never forms except on water: `motion_blocking_top` reads the absent `#blocks_motion_in_heightmap` tag (MAJOR, REPRODUCES — extended)

The port's `motion_blocking_top` (`weather.rs:41-59`, file byte-identical
to 26.1's) reads `#blocks_motion_in_heightmap`, which is ABSENT from
26.1.1 data (verified in `block-tags.json`; no synthesis anywhere in the
tree) → only fluids count as motion-blocking. Vanilla 26.1.1
`Heightmap.MATERIAL_MOTION_BLOCKING` is a method reference to
`BlockStateBase.blocksMotion()` (bootstrap method 1 in `Heightmap`,
re-verified fresh with `javap -v`); the MOTION_BLOCKING heightmap type
uses it. Consequence: `precipitation_info` computes top = min_y in
fluidless columns → snow never accumulates on land and ice never forms
except on water. The same wrong top also feeds `is_raining_at`
(under-glass check, `weather.rs:127`), lightning strike height
(`lightning.rs:199`), and portal-travel spawn height
(`portal_travel.rs:506`). 26.2's tree correctly used per-state
`blocksMotion()`. **NEW site in 26.1.1 (same root cause):**
`feature_world.rs:350-360` also regressed — the 26.1 tree's
`blocks_motion()` reads there were replaced with
`#blocks_motion_in_heightmap` / `#blocks_motion_in_heightmap_no_leaves`
tag reads (both tags absent), so feature-placement heightmaps
(OceanFloor, MotionBlocking, MotionBlockingNoLeaves) are broken the
same way; recorded here as an additional site of this item (worldgen
scope — see Cross-references; Batch 6's landed report does not cover
it). Fix note: replace the tag reads with the per-state `blocksMotion()`
value from the prepared data (26.2's form), or synthesise the tag at
prepare time from it.

### NEW defects in 26.1.1 (stale-carry regressions — the 26.1 tree was correct, the 26.1.1 tree reverted to the 26.2/26.3 form)

### B5-G22 — Composter completely dead: the port reads the `minecraft:compostable` component no 26.1.1 item carries (MAJOR, NEW)

Vanilla 26.1.1 `ComposterBlock` composts by a static chance table:
`COMPOSTABLES` (`Object2FloatMap<ItemLike>`, 115 entries, `bootStrap()`)
and `addItem` (bytecode read fresh): level 0 + chance > 0 adds a layer
with NO roll (offsets 30–39); above level 0 it adds one when the level
random rolls under the chance (`nextDouble() < chance`, offsets 42–57).
The 26.1 tree implemented exactly this (`compost_chance` in
`horizon-items`, verified 115/115 exact in the 26.1 run; level-side
`composter_add_item` with the level-0 rule). The 26.1.1 tree replaced
the whole mechanism with the 26.3 form:

- `horizon-items/src/items.rs:1015` populates the item's `compostable`
  field ONLY from the `minecraft:compostable` component:
  `compostable: c.get("minecraft:compostable").map(|f| f["layers"].clone())`.
- **Zero of the 1,506 items in the prepared 26.1.1 `items.json` carry
  that component** (census re-run fresh; 26.1.1 vanilla has no
  `compostable` component — 26.3 added it). `compost_chance` does not
  exist anywhere in the 26.1.1 tree (grep: zero hits).
- `game/composter.rs:29-38` therefore returns `None` for every item:
  `use_composter` returns false (no hand composting),
  `composter_container_insert` returns the stack untouched (no hopper
  composting), `villager_insert_item` returns early (no villager
  composting), and `composter_takes` is false for every held item.
- The level side (`level/interact.rs:559-633`) lost `composter_add_item`
  entirely: `composter_insert`/`composter_layer` now take a `layers`
  count directly, so the level-0-always-adds rule and the chance roll
  exist nowhere in the tree.

Observable: **no item can ever be composted in the 26.1.1 port** — the
composter accepts nothing by any path. (The level-7→8 tick itself still
functions if a composter reaches level 7 by command; the insertion path
is what is dead.) Fix note: restore the 26.1 form — the `compost_chance`
table plus `composter_add_item` with the level-0 rule.

### B5-G23 — Ice melt fires a `BLOCK_DESTROY` game event vanilla 26.1.1 never fires (MINOR, NEW)

Vanilla 26.1.1 `IceBlock.melt` (bytecode read fresh): water-evaporates
→ removeBlock, else setBlockAndUpdate(water) + neighborChanged — ZERO
`gameEvent` references in the whole class. The 26.1 tree matched this,
with a comment noting the event is a 26.3 addition. The 26.1.1 tree's
`ice_melt` (`hooks.rs:691-706`) adds
`self.game_event_state(pos, "minecraft:block_destroy", state)` after
the setBlockAndUpdate. Observable: sculk sensors (and any game-event
listener) trigger on every ice melt in the port, never in vanilla
26.1.1. (Also flagged by Batch 1 as out-of-scope observation (a).)
Fix note: delete the event line, restoring the 26.1 form.

### B5-G24 — Ice mined over solid ground leaves no water: `ice_destroyed_by_player` reads the absent `#blocks_motion` tag (MINOR, NEW)

Vanilla 26.1.1 `IceBlock.playerDestroy` (bytecode read fresh, offsets
52–86): unless the tool prevents melting or water evaporates, if the
block below `blocksMotion()` OR is liquid, the position becomes water
(`setBlockAndUpdate(meltsInto())`). The 26.1 tree matched this with the
per-state `blocks_motion()` rule (`mod.rs:975-985` in 26.1: solid unless
cobweb or bamboo sapling). The 26.1.1 tree deleted `blocks_motion()`
and `ice_destroyed_by_player` (`hooks.rs:525-541`) now reads
`b.in_set(below, &b.tags.blocks_motion)` — and `#blocks_motion` is
ABSENT from 26.1.1 data (verified in `block-tags.json`; the tag table
resolves absent tags to empty sets, `data.rs:396`
`.unwrap_or_default()`), so the check is always false. Observable:
mined ice over solid ground leaves air in the port where vanilla
26.1.1 leaves water (the liquid-below arm still works). (Also flagged
by Batch 1 as out-of-scope observation (a).) Fix note: restore the
26.1 per-state rule.

### B5-G25 — Tripwire schedules a 1-tick re-check when unpressed that vanilla 26.1.1 never schedules (MINOR, NEW)

Vanilla 26.1.1 schedules a tripwire re-check tick 10 ONLY while pressed:
`TripWireBlock`'s single `scheduleTick` is `bipush 10` gated on the
pressed flag (offsets 124–141, fresh javap), and `TripWireHookBlock`
has exactly ONE `scheduleTick` in the whole class — the 10-tick
attached-circuit re-check inside `calculateState` (offset 281). There
is no 1-tick unpress re-check anywhere in either class. The 26.1 tree
matched this, with a comment stating the pressed-to-unpressed immediate
re-check is 26.2-only. The 26.1.1 tree's `tripwire_check_pressed`
(`rails.rs:593-597`) adds `else if was { self.schedule_tick(pos, block,
1); }`. Observable: an extra scheduled tick on every press→release
transition (visible via the tick queue; the re-check itself finds the
wire still unpressed and does nothing further in the common case, but
it can catch an entity that entered without triggering `entityInside`
a tick earlier than vanilla would). Fix note: delete the `else if was`
arm, restoring the 26.1 form.

### B5-G26 — Eyeblossom neighbour cascade scans in 26.3 section order instead of `betweenClosed` order (MINOR, NEW)

Vanilla 26.1.1 `EyeblossomBlock.tryChangingState` iterates the 7×5×7
neighbour box with `BlockPos.betweenClosed` (offset 99, fresh javap) —
x innermost, then y, then z — and each matching neighbour draws its
delay from `nextIntBetweenInclusive(dist*5, dist*10)` in that scan
order. The 26.1 tree matched this with `between_closed` (plus a unit
test pinning the order). The 26.1.1 tree replaced it with
`box_scan_order` (`timeline.rs:9-23`, the 26.3 `findBlocksIn` order:
sections by x, z, y; blocks by y, z, x), used by `eyeblossom_tick`
(`timeline.rs:92`). Vanilla 26.1.1 has no blockscan package at all
(0 jar entries). Observable: the per-neighbour random draws are
assigned in a different order, so the cascade delays differ from
vanilla for the same seed (the open/close windows themselves still
match: `(12600..23401)`). Fix note: restore `between_closed` and its
test, the 26.1 form.

### B5-G27 — Bed use requires the other half: a head click with no foot returns Consume instead of sleeping (MINOR, NEW)

Vanilla 26.1.1 `BedBlock.useWithoutItem` (bytecode read fresh): client
→ SUCCESS_SERVER; if part != HEAD, redirect to the head and require
only that the head is this bed (else CONSUME); a HEAD click needs no
foot check at all — it proceeds straight to the `BED_RULE` path (sleep
or explode). The 26.1 tree matched this. The 26.1.1 tree's
`Class::AbstractBed` arm (`interact.rs:367-384`) implements the 26.3
pair check: it requires the block at the other half's position to be
this bed with the opposite part, else returns Consume. Observable:
clicking the head of a bed whose foot was destroyed (or replaced)
sleeps in vanilla 26.1.1 but does nothing in the port; the foot-click
path is unaffected (both forms Consume when the head is missing).
Fix note: restore the 26.1 form (foot redirects to head, head needs no
foot check). Note: the 26.1.1 tree's `sleep_height` change in
`game/beds.rs` (shape-top form) is NOT a defect — for every 26.1.1 bed
the shape top is 9/16 = 0.5625 and `pos_to_bed` adds 0.125, giving
exactly vanilla's flat 0.6875 (`LivingEntity.setPosToBed`, javap-verified);
the straw-bed branch is dead (no `straw_bed` in the 26.1.1 export).

### B5-G28 — Dragon egg teleport ignores the world border (MINOR, NEW)

Vanilla 26.1.1 `DragonEggBlock.teleport` (bytecode read fresh) accepts
a candidate only when it is air, the block below is non-air,
`WorldBorder.isWithinBounds` (offsets 113–120), and inside build
height — 1,000 attempts. The 26.1 tree matched this
(`within_world_border(test)` in the candidate test). The 26.1.1 tree's
`dragon_egg_teleport` (`interact.rs:579-600`) dropped the border check:
candidates need only air + solid below + build height. Observable: a
dragon egg near the world border can teleport outside it in the port;
vanilla 26.1.1 never places it there. Fix note: restore the border
check, the 26.1 form.

### B5-G29 — Bonemeal validity too strict: the 26.3 `canGrow` gate refuses lone dark/pale oak saplings (MINOR, NEW — 1.21.7 G15's original form returns)

Vanilla 26.1.1 `SaplingBlock.isValidBonemealTarget` is ONLY the height
check (ServerLevel AND `isInsideBuildHeight(pos.above(getMinimumHeight))`,
javap-verified fresh) — there is no `canGrow` method on 26.1.1's
`TreeGrower` at all. The 26.1 tree matched this, with a comment noting
`canGrow` (the 2×2 requirement for a mega-only grower) is 26.3-only, so
a lone dark oak sapling takes the meal (and consumes it 45% of the
time with no growth). The 26.1.1 tree's `bonemeal_valid`
(`bonemeal.rs:335`) adds `!self.tree_can_grow(g, pos, state)` —
`tree_can_grow` (`growth.rs:425-444`) returns false for a mega-only
grower (dark oak, pale oak — empty `trees` list, non-empty `mega_trees`)
with no 2×2 square. Observable: bonemeal on a lone dark/pale oak
sapling is refused outright in the port; vanilla 26.1.1 accepts it (and
the 0.45 success roll consumes the meal with no tree). The gate also
draws random (`weighted` on both lists) on EVERY sapling/azalea
validity check, shifting the RNG stream where vanilla draws nothing.
(This is the 1.21.7 B5-G15 over-strict form returning; the 26.1.1 tree's
own `growth_tests.rs` now asserts the refusal —
`lone_dark_oak_sapling_cannot_grow` — where the 26.1 test asserted the
meal is taken.) Fix note: delete the `tree_can_grow` gate, restoring
the 26.1 form. The opposite direction (too permissive at the ceiling)
is the reproduced B5-G15 — both directions are live at once.

### B5-G30 — Sweet berry bonemeal validity dropped the air-above and build-height checks (MINOR, NEW)

Vanilla 26.1.1 `SweetBerryBushBlock.isValidBonemealTarget` (bytecode
read fresh, offsets 0–50): age < 3 AND air above AND inside build
height. The 26.1 tree kept all three, with a comment noting 26.2
dropped the air and height checks. The 26.1.1 tree's arm
(`bonemeal.rs:406`) is age-only: `Kind::SweetBerry =>
b.table.int(state, p.age) < 3`. Observable: bonemeal is accepted on a
sweet berry bush with a solid block above it, or at the build ceiling,
where vanilla 26.1.1 refuses it. Fix note: restore the 26.1 three-part
gate.

### B5-G31 — Bonemeal succeeds on the client for `growCrop` where vanilla 26.1.1 returns PASS (MINOR, NEW)

Vanilla 26.1.1 `BoneMealItem.useOn` (bytecode read fresh): a `growCrop`
success returns SUCCESS_SERVER on the server and PASS on the client
(offsets 36–72); only the `growWaterPlant` path returns SUCCESS on both
sides (offset 143). The 26.1 tree matched this — its
`bone_meal_succeeds_on_client` returned true only for the water-plant
path. The 26.1.1 tree's version (`bonemeal.rs:216-243`) returns true
for any valid non-sapling/azalea/mushroom bonemeal target on the client
(plus a mod-block answer path). Observable: on the client, bonemeal
used on a crop (or any valid growCrop target) predicts SUCCESS in the
port where vanilla 26.1.1 passes (the server's packet does the growing;
the client-side success changes the predicted hand swing / use
result). Fix note: restore the 26.1 form (water-plant path only).

### B5-G32 — Grass bonemeal rolls use `nextFloat` where vanilla 26.1.1 uses `nextInt` (MINOR, NEW)

Vanilla 26.1.1 `GrassBlock.performBonemeal` (bytecode read fresh):
the tall-grass roll is `nextInt(10) == 0` (offsets 169–177) and the
flower-feature roll is `nextInt(8) == 0` (offsets 238–246). The 26.1
tree matched this with `next_int(10)` / `next_int(8)`, with a comment
noting 26.3's form is `nextFloat` and the draws differ. The 26.1.1
tree's `grass_bonemeal_effect` (`bonemeal.rs:1224-1234`) uses
`self.random().next_float() < 0.1` and `< 0.125`. Observable: same
probabilities (10%, 12.5%), but different draw types consume the RNG
stream differently, so the bonemeal outcome (tall grass vs flower
feature vs nothing) and every random outcome after it in the tick
diverge from vanilla for the same seed. Fix note: restore the 26.1
`nextInt` rolls.

### B5-G33 — `TreeGrower` rewritten in the 26.3 weighted-list form: tree selection diverges from vanilla's secondary-chance draws (MINOR, NEW)

Vanilla 26.1.1 `TreeGrower` (bytecode + fields read fresh) is the
secondary-chance form: 10 growers (oak, spruce, mangrove, azalea,
birch, jungle, acacia, cherry, dark oak, pale oak — NO poplar),
`secondaryChance` float, `megaTree`/`secondaryMegaTree`/`tree`/
`secondaryTree`/`flowers`/`secondaryFlowers` optionals.
`getConfiguredFeature` draws ONE `nextFloat()` unconditionally (even
at chance 0) against `secondaryChance`; `getConfiguredMegaFeature`
draws only when a secondary mega tree exists (spruce only). The 26.1
tree implemented exactly this (with three unit tests pinning the draw
counts). The 26.1.1 tree replaced it with the 26.3 weighted-list form
(`growth.rs:145-269`): `trees`/`mega_trees`/`flower_trees` weight lists
plus `weighted()` (`growth.rs:249-264`, `next_int_bound(total)`), a
`shortest` field, and a `poplar` grower (dead — no `poplar_sapling` in
the 26.1.1 export). Observable: the probabilities coincide (oak
fancy 10%, mangrove tall 85%, spruce mega 50/50) but the draw type
(`nextInt` vs `nextFloat`) and draw count differ — jungle's mega draw
happens in the port (total 1) where vanilla draws nothing, and every
single-tree grower draws `nextInt(1)` where vanilla draws a float —
so the chosen tree (fancy vs regular oak, tall vs short mangrove,
mega spruce vs mega pine) and the RNG stream after it diverge from
vanilla for the same seed. Fix note: restore the 26.1 secondary-chance
form and its tests.

### B5-G34 — Mega sapling growth leaves water where vanilla 26.1.1 leaves air (MINOR, NEW — 1.21.7 G14's form returns for the mega path)

Vanilla 26.1.1 `TreeGrower.growTree`, mega path (bytecode read fresh,
offsets 90–288): clears the 2×2 square to `Blocks.AIR` with flags 260
and restores the PASSED sapling state on failure, also with flags 260.
The 26.1 tree matched this. The 26.1.1 tree's `grow_tree`
(`growth.rs:384-419`) removes each sapling via `remove_sapling`
(`growth.rs:375-382`) — the fluid's legacy block with flags 818
(`UPDATE_SKIP_ALL_SIDEEFFECTS | UPDATE_CLIENTS`) — and restores the
collected per-position states on failure. Observable: a 2×2 of
waterlogged saplings grown to a mega tree leaves water blocks at the
trunk base in the port where vanilla 26.1.1 leaves air. (The single
path is unaffected in substance: vanilla clears the sapling to its
fluid's legacy block with flags 260 — water IS vanilla 26.1.1's form
there, and the port's legacy block matches — though the port's 818
flags differ from vanilla's 260: an extra client notification and a
skipped onPlace where vanilla runs it. Note `UPDATE_NONE` IS 260 in the
port's flag table, `mod.rs:164`.) Fix note: restore the 26.1 mega path
(air + flags 260, restore the passed state) and the single-path flags.

### B5-G35 — Digging broadcasts level events 2019/2020 that vanilla 26.1.1 never sends (MINOR, NEW)

Vanilla 26.1.1 has NO level events 2019/2020: `LevelEvent`
(`net.minecraft.world.level.block.LevelEvent`, javap `-constants`,
fresh) ends its 20xx range at 2013 (`PARTICLES_SMASH_ATTACK`). The
26.1 tree matched this, with an explicit comment at `game.rs:5266`:
"26.1 sends no level event while a player digs (26.3's 2019/2020)". The
26.1.1 tree re-added the 26.3 form: `destroy_progress_event`
(`game/block_breaking.rs:97-108`, called from `game.rs:5294` and
`:5306`) sends 2020 every fourth mining tick (the hit sound) else 2019
(the crack only); the client handles both (`game.rs:4298-4306`,
`breaking_block_effects` — crack particles on the mined face plus the
hit sound for 2020) and LAN broadcasts them
(`game/lan/level_event_targets.rs:28-29`). Observable: bystanders see
crack particles and hear hit sounds while someone digs in the port;
vanilla 26.1.1 sends no such events (the digger's own client plays its
sounds locally). Fix note: delete the event emission and handling,
restoring the 26.1 form. Cross-ref: Batch 10 (sound/rendering of the
events) — the emission itself is the breaking-family defect recorded
here.

## Oracle defects

Triage is from the measured Batch 0 table
(`docs/sweep/26.1.1/batch-0-oracles.json`) — no oracle was re-measured.
Each defect below is `fail-all` on all three classpaths. Full
`Vanilla*Oracle` names throughout. Line numbers are the oracle source
lines in `mods/horizon/src/test/java/dev/horizonlab/parity/` of the
26.1.1 tree, matched against the Batch 0 table's `firstErrorLine`.
Oracle sources are byte-identical to the 26.1 tree's except
`VanillaReplaceBlockOracle` (see B5-O04).

### B5-O01 — `VanillaCauldronOracle` fails: missing-symbol at line 118 (ORACLE DEFECT, MINOR, REPRODUCES)

Batch 0 table: `FAIL all` (1 error, missing-symbol, line 118) —
identical to the 26.1 run. The 26.1.1 jar has no
`LayeredCauldronBlock.LEVEL` field of the form the oracle reads; the
level predicate differs in 26.1.1. Same defect, same cause. (See §7
for the behaviour status: cauldron behaviour is otherwise Batch 1's
fluid surface; the level predicate is what this oracle owns.)

### B5-O04 — `VanillaReplaceBlockOracle` fails: `Feature.DIRECT_CODEC` does not exist in 26.1.1 (ORACLE DEFECT, MINOR, NEW — 1.21.7's O04 cause returns)

Batch 0 table: `FAIL all` (1 error, missing-symbol, line 36). The
26.1 run's oracle used `ConfiguredFeature.DIRECT_CODEC` and PASSED.
The 26.1.1 oracle source was re-ported from the 26.2/26.3 form and now
reads `Feature.DIRECT_CODEC` (`VanillaReplaceBlockOracle.java:36`) —
but 26.1.1's `Feature` class has NO `DIRECT_CODEC` (javap-verified
fresh: only `ConfiguredFeature` has it). This is the exact 1.21.7
B5-O04 cause returning (26.2's oracle had it too). Fix note: use
`ConfiguredFeature.DIRECT_CODEC`, the 26.1 form.

### B5-O05 — `VanillaFallingBlockOracle` fails: constructor mismatch at line 30 (ORACLE DEFECT, MINOR, REPRODUCES)

Batch 0 table: `FAIL all` (1 error, constructor-mismatch, line 30).
The falling-block entity constructor changed shape in 26.1.1 vs 26.3;
the oracle still constructs with the 26.3 form. Same defect, same
cause as the 26.1 run (whose first-error line was also 30).

### B5-O06 — `VanillaDispenserOracle` fails: missing-package at line 212 (ORACLE DEFECT, MINOR, REPRODUCES)

Batch 0 table: `FAIL all` (15 errors, missing-package, line 212).
The oracle imports `EntityTypes` (`net.minecraft.world.entity.EntityTypes`),
which does not exist in 26.1.1 (registration still lives on
`EntityType` itself — the holder move happened after 26.1.1). Same
defect, same cause as the 26.1 run (also 15 errors, line 212).

### B5-O07 — `VanillaSectionMeshOracle` fails: `MaterialBaker` is an interface in 26.1.1 (ORACLE DEFECT, MINOR, REPRODUCES)

Batch 0 table: `FAIL all` (7 errors, other, line 84). The oracle
instantiates `MaterialBaker` as a class; 26.1.1's
`net.minecraft.client.resources.model.sprite.MaterialBaker` is a
PUBLIC INTERFACE (javap-verified fresh, byte-identical to the 26.1
class). Same defect, same cause as the 26.1 run (also 7 errors,
line 84).

### B5-O08 — `VanillaBreakingModelOracle` fails: `BlockModelFeatureRenderer` absent (ORACLE DEFECT, MINOR, REPRODUCES)

Batch 0 table: `FAIL all` (2 errors, missing-symbol, line 12). The
class `BlockModelFeatureRenderer` does not exist in 26.1.1 (absent
from the client jar, verified fresh; absent in 26.1 too). Same defect,
same cause as the 26.1 run.

### B5-O09 — `VanillaFaceSupportOracle` fails: protected `CubeVoxelShape` constructor at line 45 (ORACLE DEFECT, MAJOR, REPRODUCES)

Batch 0 table: `FAIL all` (1 error, other, line 45). The oracle
constructs `CubeVoxelShape` directly; 26.1.1's constructor is
PROTECTED (javap-verified fresh, byte-identical to 26.1's). Same
defect, same cause as the 26.1 run — MAJOR because it blocks 19
drivers plus 5 cascades. (Batch 2 also records it; the coordinator
deduplicates.)

### Oracle non-items

- **B5-O02 (VanillaPortalOracle) — cascade, no independent defect, do
  not mint.** FAIL all via `VanillaBlockUpdateOracle` (Batch 0 table).
  The 1.21.7 O02 cause (`getRelativePortalPosition` missing) does NOT
  apply — 26.1.1 HAS that method; the failure is purely the cascade.
  Same as the 26.1 run.
- **B5-O03 (VanillaPickBlockDataOracle) — evaporates, PASS all
  classpaths** in the Batch 0 table. Same as the 26.1 run.

**In-scope oracles passing:** VanillaPickBlockDataOracle,
VanillaBlockSurvivalOracle, VanillaBlockSupportOracle,
VanillaBlockPropertiesOracle, VanillaBlockStateCodecOracle,
VanillaCollisionOracle, VanillaHeightmapOracle.

## §5 Re-check verdicts

### 26.1 re-checks (14/14 decided — all REPRODUCE)

| 26.1 item | 26.1.1 verdict | 26.1.1 item |
|---|---|---|
| B5-G03 dirt path | REPRODUCES (jar byte-identical; `classes.rs` byte-identical) | B5-G03 |
| B5-G13 dripstone | REPRODUCES (both causes; `#speleothems` absent; `PointedDripstoneBlock` extends `Block`) | B5-G13 |
| B5-G15 bonemeal ceiling | REPRODUCES (nesting bug persists; see also new B5-G29) | B5-G15 |
| B5-G17 chorus ceiling | REPRODUCES (`tick.rs` byte-identical) | B5-G17 |
| B5-G18 bell inverted | REPRODUCES (`redstone.rs` bell arm unchanged) | B5-G18 |
| B5-G19 mud tag | REPRODUCES (all four reads still 'i' spelling) | B5-G19 |
| B5-G20 portal frame | REPRODUCES (`portal.rs` byte-identical) | B5-G20 |
| B5-G21 motion_blocking_top | REPRODUCES (`weather.rs` byte-identical; + `feature_world` site) | B5-G21 |
| B5-O01 Cauldron | REPRODUCES (Batch 0 table, line 118) | B5-O01 |
| B5-O05 FallingBlock | REPRODUCES (Batch 0 table, line 30) | B5-O05 |
| B5-O06 Dispenser | REPRODUCES (Batch 0 table, line 212) | B5-O06 |
| B5-O07 SectionMesh | REPRODUCES (Batch 0 table, line 84) | B5-O07 |
| B5-O08 BreakingModel | REPRODUCES (Batch 0 table, line 12) | B5-O08 |
| B5-O09 FaceSupport | REPRODUCES (Batch 0 table, line 45) | B5-O09 |

### 1.21.7 defect re-checks (15 behaviour + 7 oracle — all decided)

| 1.21.7 item | 26.1.1 verdict | Notes |
|---|---|---|
| B5-G01 farmland | EVAPORATES | 26.1.1's farmland uses `FarmlandBlock` + `#maintains_farmland` (present); the 1.21.7 defect was the absent `#convertible_to_mud` tag in the 1.21.7 export — different mechanism. Port matches vanilla 26.1.1. |
| B5-G02 grass/mycelium | EVAPORATES | 26.1.1 chains map via `SpreadingSnowyBlock`/`SnowyBlock`; grass-spread logic matches vanilla 26.1.1. |
| B5-G03 dirt path | REPRODUCES | Same class-name mismatch (`DirtPathBlock` vs `PathBlock`) → B5-G03. |
| B5-G04 fungi/roots | EVAPORATES | 26.1.1 chains map (`NetherFungusBlock`→`VegetationBlock`); support tags `#supports_crimson_fungus` etc. present and read. |
| B5-G05 stems | EVAPORATES | 26.1.1 stem chains map; `stemSupportBlocks`/`fruitSupportBlocks` fields present in prepared data and read. |
| B5-G06 crops | EVAPORATES | Crop growth/survival matches vanilla 26.1.1 (farmland tag present, moisture logic verified). |
| B5-G07 mushrooms | EVAPORATES | Mushroom spread/survival matches; only the bonemeal height gate has the nesting bug (B5-G15, a different check). |
| B5-G08 snow layer | EVAPORATES | `#cannot_support_snow_layer`/`#support_override_snow_layer` present; survival matches vanilla 26.1.1. |
| B5-G09 seagrass | EVAPORATES | `#cannot_support_seagrass` present; survival matches. |
| B5-G10 kelp | EVAPORATES | `#cannot_support_kelp` present; survival matches. |
| B5-G11 cactus flowers | EVAPORATES | `#support_override_cactus_flower` present; survival matches. |
| B5-G12 fletching table | EVAPORATES | No `FletchingTableBlock` in 26.1.1 (chain `['Block']`); vanilla 26.1.1's fletching table is a plain block with no special behaviour — port matches. |
| B5-G13 dripstone | REPRODUCES | Same two causes → B5-G13. |
| B5-G14 tree growth water | SPLIT — single path EVAPORATES, mega path REPRODUCES | Vanilla 26.1.1's SINGLE path leaves the fluid's legacy block (water is vanilla's form there); the MEGA path sets air. The port leaves water on BOTH → mega-path defect is the new B5-G34. |
| B5-G15 bonemeal validity | REPRODUCES (both directions) | The 26.1 over-permissive form persists (B5-G15 nesting bug) AND the 1.21.7 over-strict form returns (B5-G29 `canGrow` gate). Both live at once. |

| 1.21.7 oracle | 26.1.1 verdict | Notes |
|---|---|---|
| B5-O01 Cauldron | REPRODUCES | Same cause → B5-O01. |
| B5-O02 Portal | Cause EVAPORATES; cascade remains (no item) | `getRelativePortalPosition` EXISTS in 26.1.1; the oracle still fails via the `VanillaBlockUpdateOracle` cascade. |
| B5-O03 PickBlockData | EVAPORATES | PASS all classpaths in the Batch 0 table. |
| B5-O04 ReplaceBlock | REPRODUCES | The re-port reintroduced the exact 1.21.7 cause (`Feature.DIRECT_CODEC`) → new B5-O04. |
| B5-O05 FallingBlock | 1.21.7 cause EVAPORATES; oracle still fails on 26.1 cause | Debug-package cause gone; now constructor mismatch → B5-O05. |
| B5-O06 Dispenser | 1.21.7 cause EVAPORATES; oracle still fails on 26.1 cause | GameRules-package cause gone; now `EntityTypes` missing-package → B5-O06. |
| B5-O07 SectionMesh/BreakingModel | 1.21.7 causes EVAPORATE; both still fail on 26.1 causes | Splits into B5-O07 (`MaterialBaker` interface) + B5-O08 (`BlockModelFeatureRenderer` absent). |

## §6 Systemic findings

### 1. The 26.1.1 tree is a stale re-port from 26.2/26.3 form, not a carry-forward of the 26.1 tree (B2's finding, confirmed in B5 scope)

Batch 2's §6 systemic finding — the 26.1.1 tree did NOT inherit the
26.1 tree's `level/` behaviour fixes, contradicting
PORT-NOTES-26.1.1.md's carry-forward claim — reproduces in every B5
file that differs. The 26.1 tree's deliberate 26.1-form fixes were
reverted to the 26.2/26.3 form one method at a time: ice melt gained
the 26.3 game event (G23), ice mining lost the per-state rule (G24),
tripwire gained the 26.2 re-check (G25), eyeblossom gained the 26.3
scan order (G26), beds gained the 26.3 pair check (G27), dragon eggs
lost the border check (G28), bonemeal gained the 26.3 `canGrow` gate
(G29) and lost the 26.1 sweet-berry gates (G30) and client PASS (G31),
grass bonemeal gained the 26.3 `nextFloat` rolls (G32), TreeGrower was
rewritten in the 26.3 weighted form (G33/G34), digging gained the 26.3
level events (G35), and the composter was converted to the 26.3
component form (G22). The 26.1 tree's own comments name each of these
as a deliberate 26.1-vs-26.3 distinction — the 26.1.1 tree deleted the
comments along with the behaviour. And the 26.1.1 tree's own
`growth_tests.rs` now ASSERTS the wrong behaviour (lone dark oak
refusal, client bonemeal success) where the 26.1 tests asserted the
26.1 form. The 26.1 tree is the repair reference for all fourteen:
each new item's fix note says "restore the 26.1 form".

### 2. The absent-tag root cause got WORSE, not better

The 26.1.1 tree reads MORE absent tags than the 26.1 tree did:
`#blocks_motion` (ice, G24), `#blocks_motion_in_heightmap` and
`#blocks_motion_in_heightmap_no_leaves` (feature_world, folded into
G21), `#blocks_fluid_flow` (render/mesher + fluids — B1/render scope),
`#sculk_growth_inhibitors` in `level/sculk.rs:620` (B4 scope, see
Cross-references). The tag-table construction resolves absent tags to
EMPTY sets (`data.rs:396` `.unwrap_or_default()`), and `tag()` returns
`Option` (`mod.rs:941-942`), so every absent-tag read silently
degrades to "false" — never an error, never a log line. The leakage
audit exists for exactly this pattern; these reads should be audited
against the 26.1.1 tag inventory (248 tags) rather than the 26.3 one.

### 3. The composter is the only whole-block regression

Fourteen of the fifteen new defects are edge-case divergences (one
method, one code path). G22 alone removes an entire block's
interaction: no item can be composted by any path. It is also the only
new defect whose root cause is a data-model change (the 26.3
`minecraft:compostable` component) rather than a method-form change —
and the only one whose fix requires restoring a data table
(`compost_chance`, 115 entries) rather than a few lines.

### Cross-referenced gaps (not re-minted)

- **Sculk inhibitors in `level/sculk.rs:620`** — reads the absent
  `#sculk_growth_inhibitors` tag. Batch 6 minted the worldgen
  instance (B6-G07) and explicitly routed the level instance to B4;
  recorded here only as a cross-reference, NOT minted.
- **`update.rs` `ensure_block_entity` re-check** — the 26.1 report
  assigned it to B4; the 26.1.1 diff confirms the re-check is still
  present. B4's scope.
- **Campfire douse** (`game/campfire_douse.rs` reads the absent
  `#douses_campfires` tag) — Batch 7's scope (its brief lists the
  1.21.7 G08 shovel campfire douse).
- **`can_hold_any_fluid` / lava-fire walk** — Batch 1 minted (B1-G05,
  B1-G06). Sponge does NOT use `can_hold_any_fluid` (see §7) and
  stays a pass.
- **Diode guard** — B2-G07 (the `redstone.rs` diff). **Redstone wire
  unmapped** — B2-G01. **PushReaction old names** — B2-G02.
- **B3-G22 startup blocker** (`#causes_suffocation` /
  `#suppresses_bounce`) — Batch 3's.
- **Fire spread ignoring doFireTick (Batch 1's G4) — EVAPORATES for
  26.1.1.** There is NO `doFireTick` gamerule in vanilla 26.1.1
  (59 `GameRule`s; the only fire-related ones are `FIRE_DAMAGE` and
  `FIRE_SPREAD_RADIUS_AROUND_PLAYER`; the only mention of doFireTick
  in the jar is `GameRuleRegistryFix`). Flagged for Batch 1's
  re-examination.
- **Dripstone cauldron fill** — Batch 1's G6. **Loot tables behind
  `drop_resources`** — Batch 3's. **`TreeFeatureOracle`** — Batch 6's.
  **`CushionPlacementOracle`** — entity-decoration scope.
- **`blocks_fluid_flow` reads** — only in render assets/mesher and
  arrows/entities fluids, never in `level/`; B1/render scope, not B5.
- **`feature_world` heightmap tags** — folded into G21's evidence as
  an additional site of the same root cause (same absent tag, same
  fix); NOT minted separately. Batch 6's landed report does not cover
  it and Batch 1 flagged it as worldgen scope.

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
methods are byte-identical to 26.1's and the deciding port code
(`tick.rs`, `survive.rs`, `shapes.rs`, `classes.rs`) is byte-identical
to the 26.1 tree's, with the census confirming the class chains and
tags the checks depend on. Specific fresh verifications: `IceBlock`
melt has zero `gameEvent` refs (G23 is the port adding one); the
turtle-egg trample code is present at `motion.rs:1620`,
`mobs/ai.rs:857-860`, `armor_stands.rs:1153-1185` (`block_landing.rs`
byte-identical, `motion.rs`/`mobs/ai.rs` diffs comment-only plus one
`hurt_marked` removal unrelated to trample); `sponge_remove_water`
(`hooks.rs:618`) uses `is_water_fluid`/`pickup_block`/Liquid checks,
NOT `can_hold_any_fluid`, so the sponge pass stands despite B1-G05.

### Passes — survival/support (26/28)

All 26.1 passes stand: farmland, grass/mycelium, dirt path (survival
form — the REVERT is G03, the survive predicate itself is fine),
nether fungi/roots, stems, crops, mushrooms, snow layer, seagrass,
kelp, cactus flower, chorus plant, chorus flower, bell, sapling,
azalea, bamboo, sugar cane, vine, cocoa, sweet berry, nether wart,
turtle egg, sniffer egg, dripstone-adjacent (see G13), bed (survival
form — the USE is G27). The support tags the checks depend on are all
present in the 26.1.1 tag inventory (verified fresh): `#maintains_farmland`,
`#grows_crops`, `#supports_nether_wart`, `#supports_crimson/warped_fungus`,
`#supports_crimson/warped_roots`, `#supports_stem_crops/fruit`,
`#supports_vegetation/cactus/sugar_cane/bamboo/chorus_flower/chorus_plant/
hanging_mangrove_propagule/mangrove_propagule`, `#cannot_support_snow_layer`,
`#support_override_snow_layer`, `#cannot_support_seagrass`,
`#cannot_support_kelp`, `#support_override_cactus_flower`,
`#overrides_mushroom_light_requirement`, `#prevents_nearby_leaf_decay`,
`#sniffer_egg_hatch_boost`. The two gaps are G03 and G13 (both
survival-form consequences of the same root causes).

### Passes — updates (14/16)

All 26.1 passes stand: neighbour-change propagation, `updateShape`
for every in-scope block, redstone power updates (see B2), the
`ensure_block_entity` re-check is B4's scope (see Cross-references).
The two gaps are G03 and G13 (updateShape arms dead for the same
reasons). `update.rs` differs from the 26.1 tree only in the
`ensure_block_entity` region and comments — no B5 verdict changes.

### Passes — placement (9/11)

All 26.1 passes stand: block placement, waterlogging, the
`canSurvive` gate at placement, face-attached placement, the
`getStateForPlacement` forms for every in-scope block. The two gaps
are G03 and G13 (placement arms dead for the same reasons). `place.rs`
is byte-identical to the 26.1 tree's.

### Passes — interaction (13/20)

Pick-block (data + item), replace-block (placement replace rules),
door/gate/trapdoor use, chest/barrel use, furnace use, crafting table
use, anvil use, enchanting table use, beacon use, brewing stand use,
lectern use, jukebox use, note block use, lever/button use, pressure
plate use, tripwire hook use (the USE is fine — the re-check is G25),
bell use (the USE is fine — the hit test is G18), bed use form (the
PAIR CHECK is G27), dragon egg use form (the BORDER is G28),
composter use form (the ITEM TABLE is G22), mud conversion form (the
TAG is G19), portal use form (the FRAME is G20). The seven gaps are
the bonemeal-growth check (G15/G29–G32), G18, G19, G20, G22, G27, G28.

### Passes — breaking/removal (11/14)

All 26.1 passes stand except the three gaps: `drop_resources` form
(loot tables are Batch 3's), block-break events (the 2001 event form
matches; the door `!correct_tool` extra 2001 event is a carried nit,
not a defect — see Nits), `destroy_progress` is the NEW G35, ice
mined is G24, tripwire break is G25's family. The `block_breaking.rs`
diff (destroy-progress events) is the only breaking-family change;
`block_landing.rs` is byte-identical.

### Passes — structural (0/2)

Both structural checks gap, as in 26.1 — and the tag inventory is
worse in 26.1.1 (the port now reads more absent tags; see §6.2).
Class-chain census: 1,168 blocks, 29,873 states, 291 distinct class
names, 28 unmapped (same 28 as 26.1, including `DirtPathBlock`,
`RedStoneWireBlock`, `AmethystBlock`); 3 blocks fall back to plain
`Block` despite a non-`Block` chain (dirt_path, redstone_wire,
amethyst_block); 87 blocks with an unrecognised most-derived class but
a recognised ancestor (same as 26.1). Tag inventory: 248 tags;
absent (verified): `#speleothems`, `#nether_portal_frame`,
`#blocks_motion_in_heightmap`, `#blocks_motion_in_heightmap_no_leaves`,
`#convertible_to_mud` ('i'), `#blocks_motion`, `#blocks_fluid_flow`,
`#washed_away_by_fluids`, `#blocks_lava_fire_spread`,
`#sculk_growth_inhibitors`, `#douses_campfires`,
`#mushroom_grow_block`, `#snow_layer_cannot_survive_on`,
`#snow_layer_can_survive_on`.

### Nits recorded as passes, not defects (carried from 26.1, re-verified)

- Door `!correct_tool` extra 2001 event (unobservable in practice).
- Sniffer-egg hardcoded `moss_block` (matches the shipped data).
- Turtle-egg/eyeblossom hardcoded overworld timelines (match vanilla
  26.1.1 timeline data).
- Tripwire disarm flags 260 vs vanilla 3 (unobservable).
- Bamboo dead `bamboo_height_above` query (no caller).
- Sleep-height shape-top form (equivalent for all 26.1.1 beds — see
  G27's note).
- Poplar grower (dead code — no `poplar_sapling` in the export).
- The 26.1.1 tree's `bonemeal.rs` adds `place_inline_feature` (new API,
  out of B5 scope — noted only).

### Pilot leftovers diagnosed

None outstanding for Batch 5. The 26.1 pilot's B5 leftovers (the
composter table, the TreeGrower draw form, the ice rules) were all
decided in the 26.1 run and re-verified here — and the 26.1.1 tree
REGRESSED on all three (G22, G33/G34, G23/G24). No check in this batch
is left undecided.

### Drivers and oracle status

No driver runs per the brief (static checks only). Oracle status is
from the Batch 0 measured table (`docs/sweep/26.1.1/batch-0-oracles.json`):
in-scope oracles — 7 PASS all, 7 FAIL all (minted as B5-O01/O04–O09),
1 cascade (B5-O02, no item), 1 evaporated (B5-O03). The oracle sources
in the 26.1.1 tree are byte-identical to the 26.1 tree's except
`VanillaReplaceBlockOracle` (the `Feature.DIRECT_CODEC` re-port, B5-O04).

### Evidence index

Batch evidence dir: `~/workspace/mcir-work/sweep-26.1.1-batches/b05/evidence/`

- `jar-compare.py` + `jar-compare.log` — 94 in-scope server-game classes
  byte-identical between the 26.1 and 26.1.1 jars (0 diffs), 4 client-jar
  classes likewise; includes the absences (`SpeleothemBlock`,
  `FletchingTableBlock`, `FarmBlock`, `SulfurSpikeBlock`,
  `RedstoneWireBlock`, `EntityTypes`, `Prediction`, `BlockModelFeatureRenderer`).
- `level-diff-26.1-vs-26.1.1.txt` (1,098 lines) — the full `level/` diff;
  15 files differ, the rest byte-identical.
- `census.py` + `census.log` — class-chain census (1,168 blocks, 28
  unmapped, 3 plain-`Block` fallbacks), tag inventory (248 tags,
  absent/present per tag), stem support fields, configured-feature JSON
  nesting, `compostable` component census (0/1,506 items).
- `javap/` (27 files) — fresh `javap -c -p` dumps from the 26.1.1 jars:
  `IceBlock`, `BedBlock`, `DragonEggBlock`, `EyeblossomBlock`,
  `TreeGrower`, `SaplingBlock`, `AzaleaBlock`, `MushroomBlock`,
  `SweetBerryBushBlock`, `BoneMealItem`, `GrassBlock`, `ComposterBlock`,
  `TripWireHookBlock`, `TripWireBlock`, `PortalShape`, `Heightmap`
  (`-v` for the bootstrap method), `PotionItem`, `ChorusFlowerBlock`,
  `BellBlock`, `Player`, `ChunkMap`, `MaterialBaker`, `Feature`,
  `ConfiguredFeature`, `LevelEvent` (`-constants`), `LivingEntity`,
  `PointedDripstoneBlock`, `DirtPathBlock`, `GameRules` (59 rules, no
  `doFireTick`).

Key port-side line numbers (26.1.1 tree,
`versions/26.1.1/crates/horizon-world/src/level/` unless noted):

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
- G18: `redstone.rs:1442` (`click_y <= 0.8124f32`).
- G19: `data.rs:460`, `horizon-client/src/game/potions.rs:710`,
  `horizon-client/src/game/remote.rs:8658`,
  `horizon-client/src/game/dispenser.rs:2245` (all 'i' spelling).
- G20: `portal.rs:40-44` (`portal_frame`), `:67`, `:131`;
  `hooks.rs:588-597` (`base_fire_on_place`).
- G21: `weather.rs:41-59` (`motion_blocking_top`), `:127`
  (`is_raining_at`); `feature_world.rs:350-360` (new site);
  `lightning.rs:199`, `portal_travel.rs:506` (consumers).
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
  (`tree_can_grow`); `growth_tests.rs`
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

**Batch 5 (26.1.1): 128 checks, 102 pass, 26 gap; 29 items (22
behaviour — 4 major, 18 minor — plus 7 oracle defects, 1 major).**
Every 26.1 defect reproduces; the 26.1.1 tree adds fourteen new
behaviour defects and one new oracle defect, all stale-carry
regressions from the 26.2/26.3 form. The 26.1 tree is the repair
reference for all fifteen new items — each fix note says "restore the
26.1 form", and the 26.1 tree's own comments name each distinction.

The batch is complete: all checks decided statically, all 26.1 and
1.21.7 re-checks decided, no needs-harness items, nothing stalled.
The one thing the parent should know: the 26.1.1 tree's own tests now
assert the wrong behaviour in at least two places (`growth_tests.rs`:
lone dark oak refusal, client bonemeal success), so a fix wave that
restores the 26.1 form will need to restore the 26.1 tests too.
