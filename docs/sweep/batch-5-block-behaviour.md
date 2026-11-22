# Batch 5 — Block behaviour (1.21.7 parity sweep)

**Version:** 1.21.7 · **Date:** 2026-11-22 · **Branch:** `parity-sweep` (workdir
`~/workspace/mcir-work/sweep-batch-5`, HEAD `a2f95487`) · **Scope:**
`docs/PARITY-SWEEP.md` §4 Batch 5 — block ticks, interaction, survival,
support, updates, breaking; ~30 drivers; 1,105 blocks grouped by Java class
chain (16 colours of one block are one check, not 16).

**Status: DRAFT — not committed.** An execution-risk notice on the setup step
blocks running, building, testing, or issuing git commands inside the repo
without the user's go-ahead, so no driver runs, no cargo tests, no oracle
repairs, and no commit/push happened in this batch. Everything below is static
analysis: `javap -c` on the named 1.21.7 server jar, source comparison against
`versions/1.21.7/crates/`, and checks of the prepared 1.21.7 data
(`~/workspace/mcir-work/mob-spawn/resources/`). The coordinator needs the
user's go-ahead to run the drivers/tests and to commit these deliverables to
`docs/sweep/batch-5-block-behaviour.md` and `pins/1.21.7/sweep/batch-05.json`.

**Workdir caveat — read before trusting the leaf check.** The files on disk in
this workdir appear to predate the 14:09 NPT leaf fix (`5d83cfb8`) and the
14:45 burn fix (`a2f95487`), even though HEAD points at `a2f95487`: the 12:40
supports synthesis (`6e6cb55b`) and the 11:47 burn-list fix (`651b43d3`) ARE
present in the files, but `leaf_distance_at` (`level/shapes.rs:1081`) still
reads the absent `#prevents_nearby_leaf_decay` tag with no `#logs` read, and
no synthesis of that tag exists anywhere in the tree — which the leaf fix is
recorded as changing. The workdir was found in a broken state (index cleared,
everything untracked) after the previous agent was cut off, so its files are
most likely from a checkout made between 12:40 and 14:09. All findings below
are against the files on disk; no finding except the leaf check touches code
changed by the leaf or burn fixes. The coordinator should re-verify the leaf
check against current main on merge.

## Tally

- **Behaviour checks: 86 — 59 pass, 27 gap** (across 15 defect items
  B5-G01–B5-G15, plus 2 cross-referenced gaps owned by other batches).
- **Oracle defects: 7** (B5-O01–B5-O07, for Batch 0 — not counted in the check
  tally). No oracle repairs were possible (blocked).
- **Pilot leftovers: all 4 in Batch 5 scope diagnosed** (see below). No pilot
  failure remains unexplained.

## Systemic picture

Two systemic causes cover 11 of the 15 defects.

**1. Class-name drift (5 defects).** 28 exported 1.21.7 class names do not map
to the port's 26.x-shaped class table (`level/classes.rs`); verified
exhaustively — 255 distinct terminal classes in the export, 28 unmapped (same
count Batch 2 verified). Chain building skips unmapped names
(`level/data.rs:943-955`), so the block resolves to a shorter chain and every
class-dispatched arm for the missing class never runs. In every Batch 5 case
the port logic itself matches vanilla method-for-method — only the dispatch is
dead. Affected here: `FarmBlock` (B5-G01), `SpreadingSnowyDirtBlock` /
`SnowyDirtBlock` (B5-G02), `DirtPathBlock` (B5-G03), `FungusBlock` /
`RootsBlock` (B5-G04), `FletchingTable` (B5-G12). The other 23 unmapped names
are harmless in this batch's scope (amethyst chime, glass fallbacks,
weathering-copper variants that fall back to working ancestors, furnaces →
`AbstractFurnace`, skulls → `Skull`/`WallSkull`, `RedStoneWireBlock` already
Batch 2's G7).

**2. 26.x tag-based logic for tags absent in 1.21.7 (7 defects).** Every such
lookup silently degrades to the empty set (`Tags::resolve` maps a missing tag
to `unwrap_or_default()`), exactly the pattern behind pilot defects
G1/G3/G5/G6. Affected here: `grows_crops` (B5-G06),
`overrides_mushroom_light_requirement` (B5-G07),
`cannot_support_snow_layer` / `support_override_snow_layer` (B5-G08),
`cannot_support_seagrass` (B5-G09), `cannot_support_kelp` (B5-G10),
`support_override_cactus_flower` (B5-G11), `speleothems` (B5-G13). The
`supports_*` synthesis (`6e6cb55b`, `worldgen/tags.rs`) is the proven fix
pattern for this half: chorus, cactus, sugar cane, bamboo, cocoa and
vegetation support all pass because their tags are synthesized; the seven
defects above are the tags the synthesis does not cover.

The remaining three defects are a data gap (no stem support data in the
export, B5-G05), a fluid-handling difference (B5-G14) and an over-strict
validity gate (B5-G15).

## Defects

### B5-G01 — Farmland is fully static: never dries, never reverts, never tramples (MAJOR, NEW)

**Checks failed (4):** farmland random tick; farmland survival; farmland
updateShape; farmland trampling (`fallOn`).

**Cause — a class-name mismatch, not a logic bug.** Vanilla 1.21.7 names the
class `FarmBlock`; the export says so (`minecraft:farmland` →
`['FarmBlock', 'Block']`) but `level/classes.rs` only knows
`"FarmlandBlock"`, so the chain is `[Block]` and every farmland arm is dead:
`farmland_random_tick` (`tick.rs:350`, moisture dry/wet/decay — logic itself
verified exact vs `FarmBlock.randomTick` bytecode), the scheduled-tick arm
(`tick.rs:197`), the survive arm (`survive.rs:194`), the updateShape arm
(`shapes.rs:465`) and the placement arm (`place.rs:922`). Farmland never
dries, never reverts to dirt.

**Trampling is a second, independent gap.** `FarmBlock.fallOn` (jar: calls
`turnToDirt` when an entity falls on it with fall distance > 0.5, random
chance, mobGriefing-gated for mobs) has no implementation anywhere in `level/`
or the game layer — no `fallOn`/`trample` handling exists at all.

**Latent error in the dead survive arm, for the fix agent:** the port uses a
`maintains_farmland` tag, but vanilla `FarmBlock.canSurvive` exempts only a
fence gate or moving piston above (bytecode-verified). A bare class alias
would revive the arm with the wrong survival rule.

### B5-G02 — Grass/mycelium never spread; snowy never updates (MAJOR, NEW)

**Checks failed (2):** spreading snowy random tick; snowy updateShape.

Vanilla 1.21.7 names the classes `SpreadingSnowyDirtBlock` and
`SnowyDirtBlock`; the export says so (grass/mycelium →
`['GrassBlock'|'MyceliumBlock', 'SpreadingSnowyDirtBlock', 'SnowyDirtBlock',
'Block']`; podzol → `['SnowyDirtBlock', 'Block']`) but `level/classes.rs`
only knows `SpreadingSnowy`/`Snowy`, so both names are skipped and the chains
are `[Grass|Mycelium, Block]` / `[Block]`. The `spreading_snowy_tick` arm
(`tick.rs:450` — logic verified faithful vs
`SpreadingSnowyDirtBlock.randomTick`) and the `Snowy` updateShape arm
(`shapes.rs:721` — logic verified vs `SnowyDirtBlock.updateShape`, uses the
`#snow` tag which exists) never run: grass/mycelium never spread to dirt, and
snowy never updates. Explains the snowy half of pilot failure
`snowy_grass_and_chests_pair`.

### B5-G03 — Dirt path never reverts to dirt (MINOR, NEW)

**Checks failed (3):** dirt path revert tick; dirt path survival; dirt path
updateShape.

Vanilla 1.21.7 names the class `DirtPathBlock`; the export says so but
`level/classes.rs` only knows `PathBlock`, so the chain is `[Block]` and the
`Path` arms are dead: survive (`survive.rs:273`), updateShape
(`shapes.rs:465`), tick (`tick.rs` → `turn_to_base_block`) and placement
(`place.rs:929`). Port logic verified exact vs `DirtPathBlock` bytecode
(canSurvive: above not solid or fence gate; updateShape schedules tick 1;
tick unconditionally reverts to dirt) — dead only due to dispatch.

### B5-G04 — Nether fungi/roots cannot survive on nylium or soul soil (MAJOR, NEW)

**Checks failed (2):** fungi/roots survival; fungi/roots planting on nylium.

Vanilla 1.21.7 names the classes `FungusBlock` and `RootsBlock`; the export
says so but `level/classes.rs` only knows `NetherFungusBlock` /
`NetherRootsBlock`, so crimson/warped fungi and roots resolve to `Vegetation`
and survive only on `supports_vegetation` (#dirt + farmland). Vanilla
bytecode: `FungusBlock.mayPlaceOn` = `#NYLIUM || MYCELIUM || SOUL_SOIL ||
super`; `RootsBlock.mayPlaceOn` = `#NYLIUM || SOUL_SOIL || super`. Mycelium
is already in `#dirt`, so the gap is nylium + soul soil: fungi/roots cannot
be placed on, and pop off, their native Nether ground.

**Fix note:** the port's `NetherFungus`/`NetherRoots` `may_place_on` arm
(`survive.rs:375`) reads `data.support_tag`, which is `None` for fungi/roots
(they carry `requiredBlock` instead of `supportBlocks` in the export), so a
mere class-name alias would make survival return false — the fix needs the
hardcoded nylium/soul-soil rule.

### B5-G05 — Pumpkin/melon stems cannot be planted and never fruit (MAJOR, NEW)

**Checks failed (3):** stem growth + fruiting; stem `mayPlaceOn`; stem
planting.

**Zero blocks in the 1.21.7 export carry `stemSupportBlocks` /
`supportBlocks` / `connectsTo` / `fruitSupportBlocks` fields**, so
`support_tag` and `support_tag2` are `None` for every block. The
`Stem`/`AttachedStem` `may_place_on` arm (`survive.rs:375`) therefore returns
false always, and `placement()` (`place.rs:1595+`) refuses any placement that
fails `can_survive` — pumpkin/melon seeds cannot be planted at all. Vanilla
`StemBlock.mayPlaceOn` = `is(FARMLAND)` (bytecode). The `stem_random_tick`
fruit branch (`tick.rs:913`) requires `support_tag2`, so stems never fruit
either; vanilla fruits when the target is air and the block below is FARMLAND
or `#DIRT` (bytecode). The stem age-growth branch itself is correct.

### B5-G06 — Crops grow ~4x too slow (MAJOR, NEW)

**Checks failed (1):** crop growth rate.

Vanilla `CropBlock.getGrowthSpeed` is hardcoded: `is(FARMLAND)` below +
`MOISTURE > 0` gives 3.0 instead of 1.0 (bytecode-verified), so a hydrated
single crop grows ~4x as fast and a hydrated field up to ~9x. The port
(`tick.rs:819-828`) reads tag `grows_crops`, which does NOT exist in 1.21.7
jar data and is NOT synthesized (only `data.rs:462` + `tick.rs:828` reference
it). Growth speed is always 1.0.

### B5-G07 — Mushrooms die in bright light on mycelium/podzol/nylium (MINOR, NEW)

**Checks failed (1):** mushroom light survival.

Vanilla `MushroomBlock.canSurvive`: below in `#MUSHROOM_GROW_BLOCK` returns
true regardless of light, else raw brightness < 13 and `mayPlaceOn`
(bytecode-verified). The port (`survive.rs:266`) checks the absent 26.x tag
`overrides_mushroom_light_requirement` instead of `mushroom_grow_block`,
which EXISTS in the 1.21.7 jar (mycelium, podzol, crimson/warped nylium) and
is in the tags list.

### B5-G08 — Snow layer survival wrong on ice/packed ice/barrier and soul sand (MINOR, NEW)

**Checks failed (1):** snow layer survival.

Vanilla `SnowLayerBlock.canSurvive` (bytecode): below in
`#SNOW_LAYER_CANNOT_SURVIVE_ON` (ice, packed_ice, barrier) → false; in
`#SNOW_LAYER_CAN_SURVIVE_ON` (honey_block, soul_sand, mud) → true; else
`isFaceFull(collision, UP)` or 8-layer snow below. The port
(`survive.rs:295`) uses the absent 26.x tags `cannot_support_snow_layer` /
`support_override_snow_layer` plus a full-face fallback: snow survives on
ice/packed ice/barrier (should pop) and pops on soul sand (should survive;
honey/mud pass via the full-face check). The same absent tags are also used
in `weather.rs:114-117` (snow accumulation) — same fix. The port's
`full_faces` fallback itself matches vanilla's `isFaceFull` predicate
exactly.

### B5-G09 — Seagrass survives on magma blocks (MINOR, NEW)

**Checks failed (1):** seagrass survival.

Vanilla `SeagrassBlock.mayPlaceOn`: face sturdy UP and NOT magma block
(bytecode-verified hardcoded check). The port (`survive.rs:407`) checks the
absent 26.x tag `cannot_support_seagrass`, so the magma exclusion never
fires.

### B5-G10 — Kelp attaches to magma blocks (MINOR, NEW)

**Checks failed (1):** kelp attach.

Vanilla `KelpBlock.canAttachTo`: NOT magma block (bytecode-verified
hardcoded check). The port (`survive.rs:480`,
`growing_plant_can_attach_to`) checks the absent 26.x tag
`cannot_support_kelp`, so the magma exclusion never fires.

### B5-G11 — Cactus flowers cannot survive on cacti or farmland (MINOR, NEW)

**Checks failed (2):** cactus flower survival; cactus flower planting on
cactus.

Vanilla `CactusFlowerBlock.mayPlaceOn`: below is CACTUS || FARMLAND ||
`isFaceSturdy(UP, CENTER)` (bytecode-verified). The port (`survive.rs:383`)
checks the absent 26.x tag `support_override_cactus_flower` OR
face-sturdy-center. Vanilla's own sturdy data (the exporter calls
`state.isFaceSturdy` for every state/direction/type) says cactus CENTER_UP is
false and farmland CENTER_UP is false, so the port's arm returns false on
both: cactus flowers cannot be placed on cacti or farmland, and flowers grown
by `cactus_random_tick` (whose trigger is correct — it checks the cactus's own
`canSurvive`, matching vanilla) pop on the next neighbour update.

### B5-G12 — Fletching table opens a crafting menu (MINOR, NEW)

**Checks failed (1):** fletching table use.

Vanilla `FletchingTableBlock.useWithoutItem` returns
`InteractionResult.PASS` (bytecode: `getstatic PASS; areturn`) — a fletching
table has no menu. The port resolves the fletching table to
`Class::CraftingTable` (chain `[FletchingTable, CraftingTable, Block]` —
`FletchingTable` is unmapped in `classes.rs`) and the `CraftingTable` arm
(`interact.rs:439`) returns OpenMenu + Success.

### B5-G13 — Dripstone never grows and stalactites never fall (MAJOR, NEW)

**Checks failed (1):** dripstone growth/fall tick.

Every speleothem predicate goes through `#minecraft:speleothems`
(`speleothem.rs:15`, `falling.rs:54-58`), a tag absent in 1.21.7 jar data and
not synthesized. Vanilla uses hardcoded `POINTED_DRIPSTONE` checks.
`speleothem_grow`, `dripstone_transfer_fluid` and
`spawn_falling_stalactite` (its while loop never runs) are all dead. The
randomTick growth chance (0.011377778f) is verified correct in the dead code.
The cauldron-fill half is Batch 1's G6 — not duplicated here; this item is
growth + falling only.

### B5-G14 — Tree growth from a waterlogged sapling leaves water (MINOR, NEW)

**Checks failed (1):** sapling removal during tree growth.

Vanilla `TreeGrower.growTree` removes saplings by setting `Blocks.AIR` with
flags 260 (bytecode-verified). The port's `remove_sapling` (`growth.rs`)
sets the fluid's legacy block instead — a waterlogged sapling removed for
tree growth leaves a water block at the trunk base where vanilla leaves air.
The rest of `growTree` matches vanilla exactly (mega-first with 2x2
fall-through, flower check, weighted feature choice; all ten 1.21.7 growers'
feature keys and secondary chances verified against `TreeGrower` bytecode:
oak 0.1, spruce mega 0.5, mangrove 0.85, pale oak mega =
`PALE_OAK_BONEMEAL`).

### B5-G15 — Bonemeal validity stricter than vanilla for saplings/azalea/mushrooms (MINOR, NEW)

**Checks failed (1):** bonemeal validity.

Vanilla `SaplingBlock.isValidBonemealTarget` returns true unconditionally
and `AzaleaBlock`'s likewise; `MushroomBlock`'s returns only whether its
feature exists (bytecode-verified). The port's `bonemeal_valid`
(`bonemeal.rs:331-360`) adds growability gates (live features installed +
`tree_can_grow`, which draws random and refuses a lone dark/pale oak sapling
with no 2x2) and build-height gates. Vanilla consumes bonemeal 45% of the
time on an un-growable sapling with no growth; the port refuses the meal
outright. The 0.45 success roll itself matches vanilla exactly.

## Cross-referenced gaps (not re-minted)

- **Fire spread ignores `doFireTick`** — Batch 1's G4. Vanilla
  `FireBlock.tick` checks `RULE_DOFIRETICK` then
  `RULE_ALLOWFIRETICKAWAYFROMPLAYERS` (bytecode); the port's `fire_tick`
  (`tick.rs:619`) gates on `can_spread_fire_around` and no `doFireTick` exists
  anywhere in the port's gamerules. Recorded as a gap check, cross-referencing
  G4.
- **Redstone wire dot/cross toggle dead** — Batch 2's G7. The wire arm
  (`interact.rs:263`) never runs because `RedStoneWireBlock` is unmapped;
  recorded as a gap check, cross-referencing G7.
- **Dripstone cauldron fill** — Batch 1's G6 (see B5-G13).
- **Snow accumulation** (`weather.rs:114-117`) uses the same absent snow tags
  as B5-G08 — same fix.
- **Leaf decay** — fixed on main by `5d83cfb8`; see the workdir caveat above.
  Recorded as a pass by reference to the merged fix.
- **Owner playtest report "survival block-breaking drops nothing"**
  (2026-11-22): the level-side drop path (`DropResources` event +
  `doTileDrops` gate, `update.rs:445`) is verified intact; the symptom is
  game-layer/loot-side and fixes are underway elsewhere — not duplicated here.

## Oracle defects (for Batch 0)

No oracle repairs were possible (execution-risk block — no compiling). All
seven are static characterisations from the Batch 0 ledger plus jar checks.

### B5-O01 — VanillaCauldronOracle: `Player.drop` override uses a 3-arg signature 1.21.7 lacks (MINOR)

1 error, `cannot find symbol` at line 118. The oracle overrides `Player.drop`
with `(ItemStack, boolean, net.minecraft.util.Prediction)`; the 1.21.7 jar
has only `drop(ItemStack, boolean)` — the `Prediction` parameter is a
later-version addition. One-symbol repair: drop the third parameter. Blocks
`test_cauldron_parity.py`.

### B5-O02 — VanillaPortalOracle: `Entity.getRelativePortalPosition` does not exist in 1.21.7 (MINOR)

1 error, `cannot find symbol` at line 178. The 1.21.7 `Entity` class has no
such method (only portal-cooldown/processor accessors). One-symbol repair:
inline the 1.21.7 portal-position computation. Blocks
`test_nether_portal_parity.py`.

### B5-O03 — VanillaPickBlockDataOracle: `BLOCK_ENTITY_DATA` is `CustomData` in 1.21.7 (MINOR)

1 error, `cannot find symbol` at line 58. In 1.21.7,
`DataComponents.BLOCK_ENTITY_DATA` is `DataComponentType<CustomData>` — the
`BlockEntityData` component class does not exist in the 1.21.7 jar at all
(introduced in a later version), so `data.type()` and
`data.loadInto(entity, access)` do not resolve.
(`BlockEntityType.onlyOpCanSetNbt()` does exist.) Repair: read the
block-entity NBT from `CustomData` and apply it via the 1.21.7 placement
path. Blocks `test_pick_block_data_parity.py`.

### B5-O04 — VanillaReplaceBlockOracle: `Feature.DIRECT_CODEC` does not exist in 1.21.7 (MINOR)

1 error, `cannot find symbol` at line 36. The 1.21.7 `Feature` class has no
`DIRECT_CODEC` (only the `configuredCodec()` instance method). One-symbol
repair: decode via the 1.21.7 codec path. Blocks
`test_replace_block_parity.py`.

### B5-O05 — VanillaFallingBlockOracle: imports the 26.x-only `net.minecraft.util.debug` package (MINOR)

5 errors, first `package net.minecraft.util.debug does not exist` (line 11
import of `ServerDebugSubscribers`). The debug-subscribers API is a 26.x
addition. Repair: remove the debug-subscriber usage. Blocks
`test_falling_block_parity.py`.

### B5-O06 — VanillaDispenserOracle: imports the non-existent `net.minecraft.world.level.gamerules` package (MAJOR)

54 errors, first `package net.minecraft.world.level.gamerules does not
exist` (line 27 import of `GameRules`). In 1.21.7 `GameRules` lives in
`net.minecraft.world.level`; the `gamerules` package is a later-version move.
Same root cause as Batch 2's O1 (`VanillaBlockUpdateOracle`, 69 errors, same
package) — same fix pattern (re-point the import, then resolve any API
deltas). Blocks `test_block_removal_parity.py` (dispenser side effects).

### B5-O07 — VanillaBreakingModelOracle + VanillaSectionMeshOracle: depend on client blaze3d classes (MAJOR)

37 + 99 errors, both first `package com.mojang.blaze3d.platform does not
exist` — client render classes, failing on both the server and client
classpaths in the Batch 0 measurement (the measurement has no named client
jar; only `1.21.7-server-named.jar` exists in `leakage-audit/jars/`). Repair
is a classpath question (run against a real client classpath or stub the
render surface), bigger than the 1-symbol fixes above. Blocks
`test_breaking_model_parity.py`.

## Passes (59 checks)

Grouped by family. "javap" = vanilla method verified in bytecode;
"source" = port source read against the known vanilla behaviour;
"data" = verified against the prepared 1.21.7 data.

**Ticks — random + scheduled (22 pass):** nether wart growth (age<3, 1/10,
javap); bamboo stalk/sapling (stage 0, 1/3, light≥9, height<16, javap);
sugar cane (age 15, height 3, javap); cactus growth + flower trigger
(age 8 flower at 0.25/0.1, age 15 grow, javap — flower *persistence* is
B5-G11); kelp/vines growing-plant tick (age<25, `grow_probability` from the
export, kelp-into-water/vines-into-air, source); chorus flower growth
(age<5, support via synthesized `supports_chorus_flower` = end stone,
javap+data); vine spread (respects the `spread_vines` gamerule, javap);
sapling advance (light≥9 above, 1/7, `advanceTree`, javap) + the full
`TreeGrower` table (all ten growers' keys and chances, javap); mushroom
spread (1/25, max 5 in 9x3x9, 4 offset attempts, javap); turtle egg hatch
(hatch<2 crack + hatch+1, hatch==2 hatch on sand, dusk window 21062..21905
ticks = vanilla's (0.65, 0.69) time-of-day window under the (2d+d₁)/3 blend,
javap); sniffer egg (crack twice, hatch, source); frosted ice melt tick
(1/3 or <4 neighbours, brightness > 11−age−lightBlock, reschedule 20–40,
javap); ice melt (block light > 11−dampening → water, air where water
evaporates — the ultraWarm proxy, equivalent for all vanilla dimensions,
javap); scaffolding fall (distance 7, javap); leaves decay (not persistent
&& distance==7, by reference to `5d83cfb8` — see workdir caveat); coral
death out of water (dead form via `link`, source); frogspawn hatch
(survival check then hatch event, source); budding amethyst (1/5, random
direction, stage advance with facing match, source); cocoa (1/5, age<2,
javap); sweet berry growth (age<3, 1/5, light≥9 above, javap); composter
ready tick (level 7 → 8 + ready sound after 20 ticks, source); nether
portal piglin spawn (rules + difficulty + 1/2000, source).

**Survival/support (10 pass):** vegetation (`#dirt` || farmland — matches
the `supports_vegetation` synthesis exactly, javap+data); chorus
plant/flower support (synthesized end-stone tags, data); lantern
(hanging → centre support above, standing → centre support below, javap);
torch/candle (centre support below, source); wall torch (sturdy attached
face, source); bell (survival arm, source); grindstone (`canSurvive`
always true in vanilla — `iconst_1; ireturn` — port matches, javap);
scaffolding support (distance < 7, source); tripwire hook (source); cactus /
sugar cane / bamboo support (synthesized `supports_cactus` /
`supports_sugar_cane` / `supports_bamboo`, data).

**Updates (6 pass):** door updateShape (matches vanilla exactly, including
`instanceof DoorBlock` rather than same-block and returning the neighbour
state with flipped half, javap); leaves updateShape (distance recompute +
schedule, by reference to `5d83cfb8`); double-plant updateShape (other-half
check + lower-half survival on down-change, source); tripwire updateShape
(source); scaffolding updateShape (source); chorus updateShape (source).

**Placement (3 pass):** door/bed/double-plant placement incl. second halves
(`set_placed_by`, source); chorus placement connections (source);
scaffolding placement exemption (no survival check, matches vanilla,
source). Bed head direction verified (`from_y_rot`: yaw 180 → North, the
vanilla 2D mapping).

**Interaction (11 pass):** cake/candle cake (bites, eat gate, candle drop,
source); composter extract (level 8 only, source); cave vines / sweet berry
harvest (loot + sound + state reset, source); flower pot (give potted item,
source); dragon egg teleport (1000 attempts, ±15/±7, event 2015 with packed
difference, source); bell ring (hit → Success else Pass, javap); bed use
(level arm returns Consume; game layer has bed occupancy machinery
`beds.rs` — the explosion path is covered by the unrun driver
`test_bed_explosion_parity.py`); container/menu arms (furnace/chest/etc.
open menus — Batch 4 owns containers; structure verified); jukebox eject
(level arm Success with record; game layer `use_jukebox` does the eject);
note block / daylight detector (cycle + play/update, source); bonemeal
growth (bamboo 1+nextInt(2) up to 16 with stage check, sapling 0.45 roll,
crop/stem +2..5 with fruit tick at age 7, cocoa +1 — javap for the bamboo
and sapling arms).

**Breaking/removal (7 pass):** drop emission + `doTileDrops` gate
(`update.rs:445`); pre-remove side effects (container slot counts, furnace
XP, campfire/jukebox/lectern/shrieker/creaking-heart/beacon — all match
vanilla, source); spill physics (`Containers.dropItemStack` — frozen-bit
test in `removal.rs`); break XP (coal 0–2, lapis/quartz 2–5,
diamond/emerald 3–7, redstone 1–5, sculk 1, catalyst/sensor/shrieker 5,
spawner 15–43 — all match, source); ice player-destroy → water (below
blocks motion or liquid, silk-touch exemption, nether stays air, source);
turtle egg player-destroy (decrease eggs, source); sponge absorb (BFS depth
6, cap 65 including the sponge node, returns count>1, kelp/seagrass
handling — constants verified against `SpongeBlock` bytecode).

**Also verified, no defect:** wither spawning (game layer `game/wither.rs`,
called after every placement and from dispensers; `#wither_summon_base_blocks`
exists in the jar: soul sand, soul soil); weathering copper (works via the
`WeatheringCopper` interface fallback in `tick.rs:487` despite unmapped
class names); door toggle sync via `set_block` flags 10 → updateShape
(`update.rs:286-322`); the sturdy-bit data itself (the exporter calls
vanilla's `isFaceSturdy` for every state/direction/type — the cactus and
farmland CENTER_UP=false readings behind B5-G11 are vanilla's own answers).

## Pilot leftovers diagnosed

- `crops_grow_and_leaves_decay`: the failure was the leaf-distance assert,
  fixed by `5d83cfb8` (leaves). The wheat half likely passed (400 ticks at
  speed 1.0 → ~15 expected growths, need 7); the `grows_crops` rate defect
  (B5-G06) remains regardless. Needs a test re-run to confirm (blocked).
- `doors_beds_and_plants_place_both_halves`: the failure was the sunflower
  half — before `6e6cb55b`, `supports_vegetation` was empty at runtime, so
  the sunflower (`DoublePlant`/`Vegetation`) failed placement survival.
  Fixed by `6e6cb55b` (2026-11-22 12:40 NPT). Needs a test re-run to confirm
  (blocked).
- `snowy_grass_and_chests_pair`: the snowy half fails due to the dead `Snowy`
  arm (B5-G02, still open). The chest half is Batch 4.
- `a_viewed_copper_chest_does_not_weather` (`tests.rs:538`): uses
  `w.id("copper_chest")` — no copper chest block exists in 1.21.7 (added
  1.21.9). Handed to Batch 4 (block entities/containers).
- `levers_power_wire_through_interaction`, `redstone_lever_wire_and_lamp`,
  `pistons_pop_fragile_blocks_and_refuse_obsidian`: Batch 2's G7/G8/G9 —
  done, not duplicated here.
- `torch_and_door_halves_break_with_support` (`tests.rs:979`) was NOT among
  the pilot failures (presumably passed).

## Drivers and oracle status

No driver was run (blocked). Static assessment of the ~30 Batch 5 drivers
against `docs/sweep/batch-0-oracles.json`:

- **Runnable by javac (oracle passes on both classpaths):**
  `test_block_survival_parity` (BlockSurvival, BlockSupport, BlockProperties,
  BlockStateCodec, Collision, Heightmap), `test_block_support_parity`,
  `test_face_support_parity`, `test_dynamic_support_parity`,
  `test_ground_support_parity`, `test_movement_support_parity`,
  `test_block_collision_scan_parity`, `test_block_landing_parity`,
  `test_block_properties_parity`, `test_block_push_parity`,
  `test_interaction_shape_parity`. These are the first drivers to run once
  execution is unblocked — several of them (`test_block_survival_parity`
  especially) should reproduce B5-G01–B5-G11 directly.
- **Blocked on a failing oracle (see B5-O01–B5-O07):**
  `test_cauldron_parity` (B5-O01), `test_nether_portal_parity` (B5-O02),
  `test_pick_block_data_parity` (B5-O03), `test_replace_block_parity`
  (B5-O04), `test_falling_block_parity` (B5-O05),
  `test_block_removal_parity` (B5-O06, dispenser side),
  `test_breaking_model_parity` (B5-O07). `test_block_update_parity` is
  blocked on `VanillaBlockUpdateOracle` — Batch 2's O1, cross-referenced,
  not duplicated.
- **No oracle reference (live-server / fixture drivers, not runnable by
  javac alone):** `test_bamboo_parity`, `test_block_blob_parity`,
  `test_block_column_parity`, `test_block_pile_parity`,
  `test_chorus_plant_parity`, `test_vines_parity`, `test_blue_ice_parity`,
  `test_snow_freeze_parity`, `test_anvil_turtle_egg_parity`,
  `test_bed_explosion_parity`, `test_clone_block_ticks_parity`,
  `test_chunk_restart_ticks_parity`, `test_falling_air_tick_parity`,
  `test_enderman_placement_parity`, `test_stand_break_loot_parity`,
  `test_loot_mine_location_block_parity`, `test_mob_landing_support_parity`,
  `test_interaction_data_parity`. These need the live-server harness; they
  are not oracle defects.

## Method notes and limitations

- **Verification levels.** 44 vanilla classes were dumped with `javap -c`
  (`/tmp/javap/b5/`, plus `TreeGrower`, `SpongeBlock`, `TreeFeatures`,
  `DataComponents`, `BlockEntityType`, `Feature`, `Entity`, `Player` spot
  checks). Checks marked "javap" compare the port arm against the actual
  1.21.7 bytecode; checks marked "source" compare against the vanilla
  behaviour as documented in the port's own citations and the sweep's
  accumulated knowledge. Where a check rests on source reading alone, the
  doc says so.
- **RNG equivalence, not identity.** The port's `TreeGrower` uses the 26.x
  weighted-list form where 1.21.7 uses a `secondaryChance` float; the
  distributions are identical (oak 9:1, spruce mega 1:1, mangrove 15:85)
  but the random-stream consumption differs (one int draw vs one float
  draw). Not recorded as a defect — random streams diverge anyway.
- **Dead table entries, not defects.** The `GROWERS` table carries a
  `poplar` entry (26.x trees); no 1.21.7 block references it. `pre_remove`
  carries copper-chest and shelf entries; no 1.21.7 block matches them.
- **`grows_crops` vs the pilot.** The pilot's wheat check passed despite
  B5-G06 because 400 ticks at speed 1.0 still yields ~15 expected growths;
  the rate defect only shows in timed comparisons.
- **What was not covered.** Sculk drivers (scope call pending with the
  coordinator — sculk behaviour spans blocks, sensors and the spread
  manager); the game-layer loot-table consumer behind the owner's
  no-drops report (fixes underway elsewhere); the bed explosion path
  (driver unrun); flower-pot / cauldron `useItemOn` filling paths beyond
  the arms read (cauldron interaction is partly Batch 1 fluid scope).
- **Nothing was changed.** No production code, no test code, no oracle
  code was modified. The two deliverables in
  `~/workspace/mcir-work/sweep-batch-5-deliverables/` are uncommitted
  drafts awaiting the user's go-ahead.
