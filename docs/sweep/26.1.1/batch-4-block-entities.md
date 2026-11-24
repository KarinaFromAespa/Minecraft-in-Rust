# Batch 4 — Block entities / containers — 26.1.1 parity sweep

Date: 2026-11-24. Version: 26.1.1. Worktree: `~/workspace/mcir-work/sweep-26.1.1`
(branch `parity-sweep-26.1.1`). Jars: `~/workspace/mcir-work/26.1.x/versions/26.1.1/jars/`
(`server-game.jar` + `client.jar`; `server.jar` is a bundler jar, never javap'd).
Oracle table: `docs/sweep/26.1.1/batch-0-oracles.json` (391 oracles, 189 pass / 202 fail-all).
Prepared data: `versions/26.1.1/.local/horizon-client/26.1.1/`.

Checks + triage ONLY. No code was changed, no commit was made, no git command was
run in the worktree. Every vanilla claim below is verified against 26.1.1's OWN jar
or data, never the 26.3 base's form (owner's VANILLA-CHECK rule). Where a claim
rests on a static read rather than a live run, that is stated.

## Tally (as-tallied, per the 26.2 precedent)

| class | total | pass | gap |
|---|---|---|---|
| Drivers with a passing oracle | 17 | 17 | 0 |
| Drivers oracle-blocked | 20 | 0 | 20 |
| Drivers with no oracle | 20 | 13 | 7 |
| Method-2 checks | 38 | 17 | 21 |
| **Total** | **95** | **47** | **48** |

Behaviour / non-behaviour split (stated explicitly, as the brief requires):

- **Driver-status (non-behaviour) checks: 37** — 17 pass / 20 gap. These count
  whether a driver's oracle compiles; they say nothing about gameplay.
- **Behaviour checks: 58** — 30 pass / 28 gap (20 no-oracle drivers + 38 method-2
  checks).

Items: **62** — 30 behaviour gap, 17 oracle-defect, 1 fixed, 14 closed.
Live majors: 8 (G01, G02, G16, G25, G30 behaviour; O06, O08, O23 oracle).

Comparison with the 26.1 run (92 checks, 54 pass, 38 gap; 38 items): the 26.1.1
run has 3 more checks (3 new method-2 areas: composter, sculk growth inhibition,
sign interaction), 7 fewer passes, 10 more gaps. The swing is almost entirely
the regression finding below: 5 method-2 passes flipped to gap (beacon, brewing
recipes, brewing tick, trial spawner tick, menu engine), 1 no-oracle pass flipped
to gap (`test_furnace_output_components`, on G01), and 1 oracle-passing driver
moved to blocked (`test_brewing_fuel`, on the newly-failing O05).

## Method

Same as the 26.1 run:

1. **Drivers with a passing oracle (17):** the probe compiles and the driver is
   declared pass statically, per the 26.2 precedent (no live run on this VM;
   the client_probe lib test does not compile on the VM for resources, and the
   sweep's live runs are a CI matter).
2. **Drivers oracle-blocked (20):** the driver's oracle fails to compile against
   the 26.1.1 jar; each is a gap, attributed to the oracle defect that blocks it.
   Oracle error counts and first errors are taken from the Batch 0 table, not
   re-measured; each cause was verified fresh with javap against the 26.1.1 jar.
3. **Drivers with no oracle (20):** verdict from reading the driver's assertions
   against the 26.1.1 port code and vanilla bytecode. Where a driver's outcome
   depends on a regressed system (fuel, brewing), it was re-read fresh; the
   other 10 no-oracle passes were spot-checked for fuel/brewing/sign exposure.
4. **Method-2 checks (38):** static verification of an area against the jar —
   javap of the vanilla class, a read of the port code, and (where data matters)
   a count over the prepared 26.1.1 data. No count is asserted without running it.

**26.1 tree as control.** Per the coordinator follow-up, every file in scope was
diffed against the 26.1 tree (`versions/26.1/` in the worktree). The diffs are
saved in `b04/evidence/diffs/` (23 files, incl. `game-stations.rs.diff` added
for the Batch 3 brewing-totals flag). The 26.1.1 tree's own
`PORT-NOTES-26.1.1.md` records that vanilla 26.1.1 == vanilla 26.1 for every
Batch 4 class: the 26.1→26.1.1 delta is DataVersion 4786→4788 plus 3 changed
classes (`PlayerEntry.<init>`, `DetectedVersion.createBuiltIn`, `SharedConstants`
version fields); packets, registries, blocks, items and command nodes are
unchanged. None of the 6 work items touches Batch 4 scope. So every behavioural
difference between the two runs is a PORT-side regression, not a vanilla change.

**26.2-record corrections (26.1 summary.md line 283):** applied. None is Batch 4
relevant: the section covers B3 zombie/skeleton-horse food, the furnace-minecart
cap and the BreedGoal panic, B9-G11 recipe suggestions, and B6 feature_tags /
dripstone. The 26.1 summary's dedup record is carried forward: B4-G18→B5-G19,
B4-O16→B10-O03, B4-O18→B10-O05, B4-O08→B10-O05, B4-G20→B7-G17, B3-G24→B4-G24,
B4-G26→B12-G01.

## Systemic finding: the 26.1.1 tree regressed the 26.1 fixes

The 26.1.1 tree carries 26.3-form code in files where the 26.1 tree carries
corrected forms — the same pattern Batch 1 found in fluids and Batch 2 found in
level behaviour. The 26.1 fixes were NOT inherited. The regression is
concentrated in one mechanism: **component/tag reads for data that does not exist
in 26.1.1**. Vanilla 26.1.1 has no `cooking_fuel`, `brewing_fuel`,
`villager_food`, `compostable`, `mob_visibility`, `sign_text_front`,
`sign_text_back` or `waxed` components (verified: `DataComponents` in the
26.1.1 jar has none of them), and no `#sculk_growth_inhibitors` or
`#extendable_maps` tags. Where the 26.1 tree replaced those reads with the
26.1-form source (hardcoded tables, tag checks, `FuelValues`), the 26.1.1 tree
reads the absent component/tag and gets nothing.

Regressed files (diffs in `b04/evidence/diffs/`):

- `horizon-items/src/items.rs` (465 changed lines) — lost the hardcoded
  `burn_time` / `compost_chance` / `villager_food` tables and
  `BREWING_FUEL_TAG`; now reads the `cooking_fuel` / `brewing_fuel` /
  `compostable` / `villager_food` / `mob_visibility` components.
- `horizon-client-render/src/brewing.rs` (694 lines) — now data-driven brewing
  recipes + component fuel.
- `providers.rs` — `burn_time` reads the `cooking_fuel` component.
- `menu.rs` — beacon validation added; fuel/brewing arms component-based.
- `block_entity_nbt.rs` — bed type/arm dropped; brewing-save 26.3 form.
- `block_entity_item.rs` — sign text/wax components.
- `sign_edit.rs` — `allow_op_features` gate.
- `composter.rs` — component-based.
- `level/sculk.rs` — tag-based inhibitors.
- `dispenser.rs` — spawn-egg null spawn; shears alive check.
- `game/spawner.rs` + `mobs/trial_spawner.rs` — `override_entity_data`.
- `lan_stations.rs` — LAN beacon kick.
- `save.rs` — brewing save 26.3 form.
- `recipe_book.rs` — AnyFuel component-based.
- `machines.rs` — fuel insertion component-based.

Zero-diff files (26.1 behaviour carries): `merchant.rs`, `mobs/spawner.rs`,
`game/machines/crafter.rs`, `game/storage.rs`, `game/beacon.rs`,
`game/campfire.rs`, `game/cartography.rs`, `game/menu_clicks.rs`,
`game/native_machines.rs`, `game/recipe_book.rs`, `game/trial_spawner.rs`,
`game/vault.rs`, `lan/menu.rs`, `horizon-core/src/storage.rs`.

Consequence: two 1.21.7 majors that 26.1 had FIXED are back (G01 furnaces never
light, G02 brewing stands never brew), and 8 new defects (G30–G37) appear in
systems that were correct in 26.1. The 26.1.1 port is strictly worse than the
26.1 port in this batch's scope.

## Behaviour defects (gap items)

### B4-G01 (major, 1.21.7 REPRODUCES — regressed from 26.1 FIXED) — furnaces never light

`Providers::burn_time` (`horizon-client-render/src/providers.rs`) reads
`i.fuel`, the `minecraft:cooking_fuel` component. Counted over the prepared
26.1.1 data: **0 of 1,506 items** carry `minecraft:cooking_fuel` (or
`minecraft:brewing_fuel`, `minecraft:villager_food`, `minecraft:compostable`,
`minecraft:mob_visibility`). Every item's burn time is 0; no furnace, smoker or
blast furnace ever lights. `menu.rs` `is_fuel` reads `i.fuel.is_some()`, so
hopper fuel insertion (`machines.rs`) and recipe-book AnyFuel groups (empty)
fail the same way. Vanilla 26.1.1 uses `FuelValues.vanillaBurnTimes` (verified:
`FuelValues` exists, 64 `Builder.add` calls, same as 26.1) — the hardcoded
table the 26.1 tree restored and the 26.1.1 tree dropped.

### B4-G02 (major, 1.21.7 REPRODUCES — regressed from 26.1 FIXED) — brewing stands never brew

Two independent breaks, either fatal alone. (1) `Brewing::load`
(`brewing.rs`) reads data-driven `minecraft:brewing` recipes; the 26.1.1 jar
has **0** `data/minecraft/recipe/brewing/` files (the only brewing file is
`brewing_stand.json`, a crafting recipe), so `recipes` is empty, `find` never
matches, and `is_reagent`/`is_bottle` are false for every stack — the brewing
menu slots refuse every input. The tree's own `stations.rs` test expects
`data.brewing.recipes.len() == 279` (the 26.3 number); it would fail against
the prepared 26.1.1 data. (2) `brewing_fuel` reads the `brewing_fuel`
component — 0 items carry it. Vanilla 26.1.1 uses the `#brewing_fuel` item tag
(verified: the tag EXISTS in the jar) + `PotionBrewing` mixes (verified: the
class EXISTS), `FUEL_USES` 20, 400-tick brew.

Batch 3's flagged `game/stations.rs` brewing-totals change (diff saved in
`evidence/diffs/game-stations.rs.diff`) is a facet of this defect and of G36,
not a separate one: the brewing-menu fuel/progress bars now read
`total_fuel`/`total_brew_time` instead of the constants 20/400, and the tick
call takes `providers()` for the component-fuel lookup. Verified fresh: the
totals DEFAULT to vanilla's constants (`brewing.rs:322-324`: `total_brew_time:
400`, `total_fuel: 20`, `speed: 1.0`), and the only paths that mutate them
require the absent `brewing_fuel` component (`brewing.rs:345-349, 368-371`) —
unreachable in 26.1.1. So the render change is behaviourally neutral here; the
live divergence is the fuel/recipe mechanism above, and the save divergence is
G36. No separate item minted. (B3's companion note is also confirmed:
`test_brewing_parity.py:129` reads `lab.LOCAL / 'horizon-client' / '26.3'`
where the 26.1 tree reads `'26.1'` — moot while O04 blocks the driver; see
coordinator note 8.)

### B4-G04 (minor, 26.1 REPRODUCES) — furnace completes on `>=`

`furnace.rs:128` completes when `cooking >= total`. Vanilla
`AbstractFurnaceBlockEntity.serverTick` completes only on `==` (bytecode 274
`if_icmpne` — dump saved in `evidence/javap/AbstractFurnaceBlockEntity.txt`).
Same as 26.1.

### B4-G05 (minor, 26.1 REPRODUCES) — fuel remainder pushed to drops

`furnace.rs:120`. Vanilla `consumeFuel` places the remainder (e.g. empty
bucket) only when the fuel stack empties after the shrink. Same as 26.1.

### B4-G08 (minor, 26.1 REPRODUCES) — hopper suction bowl is 0.125..0.875

`machines.rs:973`. Vanilla `Hopper.getSuckAabb` is `Block.column(16.0, 11.0,
32.0)` → (0, 11/16, 0)–(1, 2, 1). Same as 26.1.

### B4-G09 (minor, 26.1 REPRODUCES) — crafter eject reaches entities

`crafter.rs:149-152` `crafter_eject` uses block-only `Containers::view`.
Vanilla `CrafterBlock.dispenseItem` uses `HopperBlockEntity.getContainerAt` +
`AABB.ofSize` and reaches entity containers (dump saved in
`evidence/javap/CrafterBlock.txt`). Same as 26.1.

### B4-G10 (minor, 26.1 REPRODUCES) — crafter trigger checks only the local player

`crafter.rs:188`. Vanilla iterates ALL `ServerPlayer`s. Same as 26.1.

### B4-G13 (minor, 26.1 REPRODUCES — worse) — no bed block entity

`containers.rs` `for_block` has no bed arm; `save.rs:1022`
`decode_block_entity` does `BlockEntity::for_block(block)?`, dropping bed BEs.
WORSE than 26.1: `block_entity_nbt.rs` `TYPES` also dropped `"minecraft:bed"`
(48→47) and the bed save arm is gone, so beds are now unrepresentable at the
canonicalizer layer too. Vanilla has `BedBlockEntity` (colour field only) and
`BlockEntityType.BED`.

### B4-G14 (minor, 26.1 REPRODUCES) — ender chest opens on LAN guests

`storage.rs:157` `open_ender_chest`. Same as 26.1.

### B4-G15 (minor, 26.1 REPRODUCES) — no decorated-pot block entity

`containers.rs` `for_block` has no pot arm; `container_size` covers only
chest/barrel/ender_chest/hopper/dispenser/dropper/shulker/copper_chest → `None`
for the pot. Vanilla `DecoratedPotBlockEntity` implements
`ContainerSingleItem$BlockContainerSingleItem` + `RandomizableContainer`, and
`BlockEntityType.DECORATED_POT` exists. Same as 26.1.

### B4-G16 (major, 26.1 REPRODUCES) — cartography table gated on a nonexistent tag

`menu.rs:1290` gates map cloning on `#extendable_maps`. The tag does not exist
in the 26.1.1 jar. Vanilla `CartographyTableMenu` has 0 `ItemTags` references.
Same as 26.1.

### B4-G18 (minor, 26.1 REPRODUCES) — dispenser mud uses the wrong tag spelling

`dispenser.rs:2245` checks `minecraft:convertible_to_mud`. Vanilla
`DispenseItemBehavior$14` uses `BlockTags.CONVERTABLE_TO_MUD`; the jar has
`convertable_to_mud` and no `convertible_to_mud`. Same as 26.1.

### B4-G20 (minor, 26.1 REPRODUCES) — spawner `setEntityData` merge

`game/spawner.rs` merges egg data into the spawner config. Vanilla's `Spawner`
interface has only `setEntityId`; 0 `setEntityData` refs in its class bytes.
Same as 26.1.

### B4-G21 (minor, 26.1 REPRODUCES) — merchant result slot blocks pickup

`menu.rs:1512-1517` `may_pickup` gate on the merchant result slot. Vanilla
`MerchantResultSlot` has NO `mayPickup` override (methods: mayPlace, remove,
onQuickCraft, checkTakeAchievements, onTake). Same as 26.1.

### B4-G22 (minor, 26.1 REPRODUCES) — exploration-map check uses the wrong predicate

Vanilla `MapExtendingRecipe.matches` calls `MapItemSavedData.isExplorationMap()`.
Same as 26.1.

### B4-G23 (minor, 26.1 REPRODUCES) — hero discount computed in f32

`merchant.rs:109` `f64::from(0.3f32 + 0.0625f32 * a)`. Vanilla `Villager`
`updateSpecialPrices` uses `0.3d + 0.0625d * amplifier` with `Math.floor`
(double). Same as 26.1.

### B4-G24 (minor, 26.1 REPRODUCES) — villager levels up immediately

`merchant.rs:398` `level += 1`. Vanilla `rewardTradeXp` sets
`updateMerchantTimer = 40`, `increaseProfessionLevelOnUpdate = true` — the
level-up is deferred to the next merchant update. Same as 26.1.

### B4-G25 (major, 26.1 REPRODUCES) — merchant float parsing

`merchant.rs:675` `int_provider` uses `as_i64()`; `:768` reads singular
`given_item_modifier`. Re-quantified against the 26.1.1 jar data, identical to
26.1: 387 trades in `data/minecraft/villager_trade/**/*.json`; 218 float
`wants.count`, 0 float `gives.count`, 371 float `max_uses`, 266 float `xp`, 41
plural `given_item_modifiers`, 0 singular. Trade sets: 68 files, 68/68 with
float `amount`. Same as 26.1.

### B4-G26 (minor, 26.1 REPRODUCES) — loot engine reads 26.3 key names

`horizon-items/src/items.rs` reads `["modifier"]` only (lines 1839–1840, 2112,
2253) and `condition()` reads the `type` key only (line 1957). Re-quantified
against the 26.1.1 jar data, identical to 26.1: 1,326 loot tables; 983
`functions` occurrences, 0 `modifier`; 1,545 condition objects naming kind via
the `condition` key. Same as 26.1.

### B4-G27 (minor, 26.1 REPRODUCES) — furnace int times + sign `allow_op_features` in saves

`block_entity_nbt.rs:383` furnace int times + `speed_multiplier` (vanilla:
4× `putShort`, no `speed_multiplier`); sign `allow_op_features` kept in the
save (vanilla `SignBlockEntity.saveAdditional` writes `front_text`,
`back_text`, `is_waxed` only). Same as 26.1.

### B4-G28 (minor, 26.1 REPRODUCES) — pot sherds saved in compound form

`block_entity_nbt.rs:664-672` compound arm. Vanilla `PotDecorations.CODEC` is
`byNameCodec().sizeLimitedListOf(4).xmap(...)` — list form. Same as 26.1.

### B4-G29 (minor, 26.1 REPRODUCES) — crafter trigger reads the `recipes` key

`crafter.rs:206` `c["recipes"]`. Vanilla
`RecipeCraftedTrigger$TriggerInstance` has the singular `recipeId` field, and
the advancement `adventure/crafters_crafting_crafters.json` has
`"recipe_id": "minecraft:crafter"`. Same as 26.1.

### B4-G30 (major, NEW in 26.1.1) — composter never composts

`composter.rs` `compostable()` reads `i.compostable`, the
`minecraft:compostable` component — 0 items carry it, so `composter_add`,
`use_composter`, `composter_takes`, `composter_container_insert` and
`villager_insert_item` all refuse every item: the whole system is dead.
Vanilla `ComposterBlock` uses the hardcoded `COMPOSTABLES` Object2FloatMap
(verified: exists) with `addItem` (not `addLayer`) and `nextFloat` chance
rolls. The 26.1 tree carried the hardcoded table; the 26.1.1 tree regressed to
the component form. Flagged by Batch 2; verified fresh.

### B4-G31 (minor, NEW in 26.1.1) — beacon validates effects and kicks LAN guests

`menu.rs:306` `beacon_required_levels` + `:1203` `beacon_update` validate the
effect pair and return false when invalid; `lan_stations.rs` kicks LAN guests
on invalid beacon effects. Vanilla 26.1.1 has NO validation:
`BeaconBlockEntity` has no `validateEffects`/`getRequiredLevelsFor`, and
`BeaconMenu.updateEffects` is payment check → encode both effects → set data →
remove payment, with no validation step. The 26.1 tree had no validation; the
26.1.1 tree added the 26.2 form.

### B4-G32 (minor, NEW in 26.1.1) — sign click actions never run

`sign_edit.rs:154-175` gates click commands, dialogs and custom actions on
`allow_op_features`, showing "Click actions are disabled" when false (`:183`).
`SignData` (`horizon-client-render/src/chunk/block_entity.rs:509-516`) derives
`Default`, so `allow_op_features` defaults to **false**; it is loaded from the
save tag only (`signs.rs:222-223`) and saved only when true (`:181-182`).
Vanilla 26.1.1 saves never carry the tag, so in practice no sign ever runs a
click action. Vanilla runs them unconditionally: `SignBlock.useWithoutItem`
calls `executeClickCommandsIfPresent` FIRST, before the waxed check
(bytecode dump saved in `evidence/javap-SignBlock-useWithoutItem.txt`), and
`SignBlockEntity` has NO `allow_op_features` field at all. The 26.1 tree did
not gate click actions.

### B4-G33 (minor, NEW in 26.1.1) — sign pick/place preserves text and wax

`block_entity_item.rs` maps sign text/wax to `minecraft:sign_text_front`,
`minecraft:sign_text_back` and `minecraft:waxed` components on pick/place.
None of those components exists in 26.1.1 — vanilla signs lose their text and
wax when picked up. The 26.1 tree did not carry the components.

### B4-G34 (minor, NEW in 26.1.1) — trial spawner op egg merges entity data

`game/spawner.rs:450-451` calls `override_entity_data` when egg data is
present; `mobs/trial_spawner.rs:405` defines it (merges the data into both
trial-spawner configs). Vanilla `TrialSpawnerBlockEntity` has only
`setEntityId`; 0 `overrideEntityData` refs in its class bytes. The trial-
spawner analogue of G20. The 26.1 tree had no merge.

### B4-G35 (minor, NEW in 26.1.1) — dispensed spawn egg on a null spawn is returned unshrunk

`dispenser.rs` returns the stack unshrunk with no event when the spawn returns
null. Vanilla `SpawnEggItemBehavior` (`net.minecraft.core.dispenser` class,
dump saved in `evidence/javap/SpawnEggItemBehavior.txt`) does
`EntityType.spawn(...)` → `pop` → `shrink(1)` → `gameEvent(ENTITY_PLACE)` →
return stack: a null spawn still shrinks the egg and fires the event. The 26.1
tree matched vanilla.

### B4-G36 (minor, NEW in 26.1.1) — brewing-stand save is the 26.3 form

`block_entity_nbt.rs:516` and `save.rs` write int `BrewTime` +
`total_brew_time` + int `Fuel` + `total_fuel` + float `speed_multiplier`.
Vanilla `BrewingStandBlockEntity.saveAdditional` writes `BrewTime` short +
`Fuel` byte only; the class's only fields are `brewTime`/`fuel` (dump saved in
`evidence/javap/BrewingStandBlockEntity.txt`). The 26.1 port matched vanilla;
this is a 26.1.1 regression. Minted separately from G27 (which covers the
furnace times + sign tag) because the brewing save is a distinct regression
with its own code path.

### B4-G37 (minor, NEW in 26.1.1) — sculk growth is never inhibited

`level/sculk.rs:620` reads the `minecraft:sculk_growth_inhibitors` tag, which
does not exist in 26.1.1; `sculk_in_tag` (`:103-104`) returns false when the
tag is absent, so the ≥3 inhibitor count never fires and sensors/shriekers
pile up without limit. Vanilla `SculkBlock.canPlaceGrowth` hardcodes
`Blocks.SCULK_SENSOR`/`SCULK_SHRIEKER` and returns false when the count in the
box exceeds 2 (dump saved in `evidence/javap-SculkBlock-canPlaceGrowth.txt`).
The 26.1 tree checked sensor/shrieker directly. Flagged by Batch 6 (their
B6-G07 is the worldgen-side twin in `worldgen/sculk.rs`; this item is the live-
gameplay side). Cross-reference B6 at merge; do not merge the two — different
code paths, different effects (a wrongly-passing live gate also consumes RNG
and returns charge−50 instead of the decay path).

## 26.1 re-check verdicts (38 items)

| id | verdict | note |
|---|---|---|
| G04, G05, G08, G09, G10, G13, G14, G15, G16, G18, G20, G21, G22, G23, G24, G25, G26, G27, G28, G29 | REPRODUCES (20) | §Behaviour defects above |
| O03, O06, O08, O09, O10, O14, O15, O16, O18, O19, O21, O22, O23, O25, O26 | REPRODUCES (15) | §Oracle defects below; identical error counts/first errors as 26.1 |
| O04 | REPRODUCES, changed form | cascade → direct missing-symbol, 14 errors |
| G17 | UNVERIFIED | masked by G01 (see below) |
| G19 | FIXED | dispenser wax sound now plays once |

**B4-G17 (smoker/blast burn-time halving) — UNVERIFIED, not FIXED.** The 26.1
defect was "Providers::burn_time ignores `fast`". The 26.1.1 `burn_time` does
thread `fast` through the provider eval, so the ignore-`fast` code is gone —
but with G01, NO fuel burns at all, so the halving behaviour is unobservable
and undecidable: the halving is data-determined by the provider value, and no
26.1.1 item has a provider value. Arbitration for the coordinator: restore
fuel data (fix G01), then check halving via the fast context. No slice item
(there is no "unverified" status); recorded here only.

**B4-G19 (dispenser wax sound twice) — FIXED.** The 26.1.1 tree removed the
3003 wax-on sound from `block_particle_event` in `level_particles.rs` (the
26.1 tree played it via `play_unshared`; the comment says 26.3 moved the
sound to the item code). `process_level_events` routes 3002..=3005 to
`block_particle_event` (particles only) and the catch-all sound table in
`game.rs` has no 3003 entry — so the dispenser's explicit
`audio.play(wax_on)` is now the only play. Sound plays exactly once.

## 1.21.7 re-check verdicts

**REPRODUCES (behaviour, 10):** G01, G02 (majors, regressed from 26.1 FIXED),
G04, G05, G08, G09, G10, G13 (reduced form — the bed BE is still missing),
G14, G15, G16. All detailed in §Behaviour defects. (G04–G16 are the same items
as the 26.1 re-checks; not double-minted.)

**WITHDRAWN (behaviour, 5 — vanilla 26.1.1 differs from 1.21.7, each
javap-verified; land as closed items):**

- **G03** — vanilla grows furnace output by the FULL result count
  (`burn()` grows output by `result.getCount()`).
- **G06** — vanilla does NOT reset the cooking timer on recipe-less input (the
  null-recipe check jumps past any timer reset).
- **G07** — no `defaultType` on `SpawnEggItem`; an egg whose `entity_data` was
  removed drops.
- **G11** — `spawner_blocks_work` EXISTS and is checked in both spawner paths
  (`GameRules.SPAWNER_BLOCKS_WORK`; `BaseSpawner.serverTick` calls
  `ServerLevel.isSpawnerBlockEnabled()`; `TrialSpawner.canSpawnInLevel` checks
  it first, then `overridePeacefulAndMobSpawnRule`, then `PEACEFUL` +
  `SPAWN_MOBS`).
- **G12** — creative enchanting without lapis is allowed
  (`EnchantmentMenu.clickMenuButton`: the lapis `isEmpty` check sits inside the
  creative guard, same as 26.1).

**WITHDRAWN (oracle, 9 — the oracle now passes; land as closed items):** O01,
O02, O07, O11, O12, O13, O17, O20, O24. (26.1 had 10 withdrawn / 16 carried;
O05 is newly failing in 26.1.1.)

**Carried over (oracle, 17):** O03, O04, O05, O06, O08, O09, O10, O14, O15,
O16, O18, O19, O21, O22, O23, O25, O26. §Oracle defects below.

**FIXED in the 26.1.1 tree:** none of the 1.21.7 behaviour items — G01/G02
were fixed in 26.1 and regressed here.

## Oracle defects

Error counts and first errors from the Batch 0 table; each cause verified
fresh with javap against the 26.1.1 jar. 17 oracle defects (26.1 had 16;
O05 is new for 26.1.1).

- **B4-O03 (minor) — VanillaFurnaceOracle**: 32 errors, cascade on the broken
  VanillaBlockUpdateOracle (Batch 2's item). Blocks `test_crafter`,
  `test_furnace`, `test_vault`.
- **B4-O04 (minor) — VanillaBrewingOracle**: 14 errors, first `23: cannot find
  symbol`. Line 23 of the 26.1.1 oracle file is
  `import net.minecraft.world.item.crafting.BrewingInput;` — `BrewingInput`
  is absent from the 26.1.1 jar. CHANGED SHAPE vs 26.1 (was a cascade on
  BlockUpdateOracle): the 26.1.1 oracle is the 26.3 form (`BrewingRecipe`,
  `RecipePropertySet`, `DataComponents.BREWING_FUEL`,
  totalBrewTime/totalFuel/speedMultiplier); 26.1's was retargeted to
  `PotionBrewing`. Diff saved in `evidence/oracle-brewing-26.1-vs-26.1.1.diff`.
  Blocks `test_brewing`.
- **B4-O05 (minor) — VanillaBrewingFuelOracle**: 8 errors, first `13: cannot
  find symbol`. Line 13 is
  `import net.minecraft.world.item.component.BrewingFuel;` — absent from the
  jar. NEW for 26.1.1 (passed in 26.1): the 26.1.1 oracle is the 26.3 form
  (`DataComponents.BREWING_FUEL`, `getUses`/`getSpeedMultiplier`); 26.1's was
  tag-based. Diff saved in `evidence/oracle-brewingfuel-26.1-vs-26.1.1.diff`.
  Blocks `test_brewing_fuel`.
- **B4-O06 (major) — VanillaDispenserOracle**: 15 errors,
  `212: package EntityTypes does not exist`. `EntityTypes` is absent from the
  jar. Blocks `test_dispenser`.
- **B4-O08 (major) — VanillaMenuOracle**: 4 errors, `138: cannot find symbol`
  (`Prediction`). `net.minecraft.util.Prediction` is absent from the jar.
  Blocks `test_menu_click`. (26.1 dedup: B4-O08→B10-O05 at merge — see
  coordinator notes.)
- **B4-O09 (minor) — VanillaRecipeBookOracle**: cascade on MenuOracle. Blocks
  `test_recipe_book`.
- **B4-O10 (minor) — VanillaPlaceRecipeOracle**: cascade on MenuOracle. Blocks
  `test_place_recipe`.
- **B4-O14 (minor) — VanillaSculkOracle**: 7 errors, `327: cannot find symbol`
  (`datapackWorldRegistries`). `WorldLoader$DataLoadContext` has
  `datapackWorldgen()`/`datapackDimensions()` only — no
  `datapackWorldRegistries()`. Blocks `test_sculk`.
- **B4-O15 (minor) — VanillaBlockEntityAnimOracle**: 3 errors; the fake level
  lacks a `fuelValues()` override. `Level.fuelValues()` is abstract in the jar.
  Blocks `test_block_entity_anim`.
- **B4-O16 (minor) — VanillaBlockEntityRenderOracle**: 9 errors, `UvMapping`.
  Same defect as B10-O03 — dedup at merge, keep B10 canonical (26.1 record).
  Blocks `test_block_entity_render`.
- **B4-O18 (minor) — VanillaRecipeBookScreenOracle**: 4 errors, renderpearl
  package. One of five oracles under B10-O05 — dedup at merge (26.1 record).
  Blocks `test_recipe_book_render`.
- **B4-O19 (minor) — VanillaEnchantDurabilityOracle**: cascade on
  BlockUpdateOracle. Blocks `test_enchant_durability`.
- **B4-O21 (minor) — VanillaVehicleContainerOracle**: 3 errors,
  `EntitySpawnRequest`. The class is absent from the jar. Blocks
  `test_vehicle_container`.
- **B4-O22 (minor) — VanillaShulkerSpaceOracle**: 2 errors, `EntityTypes`.
  Blocks `test_shulker_space`.
- **B4-O23 (major) — VanillaNaturalSpawnOracle**: 16 errors,
  `getNextEntityId()` override. `ServerLevel` has 0 `getNextEntityId`.
  Blocks `test_spawner`, `test_trial_spawner`.
- **B4-O25 (minor) — VanillaTradingOracle**: 6 errors,
  `datapackWorldRegistries` (same cause as O14). Blocks `test_trading`.
  Batch 3 overlap — cross-reference.
- **B4-O26 (minor) — VanillaGhostRecipeOracle**: 1 error, 26.3 ContextMap API.
  `net.minecraft.util.context.ContextMap` has no `builder()`/`set`/
  `buildAndValidate`. Blocks `test_ghost_recipe`.

No new in-scope oracles appeared in the 26.1.1 table: the only unlisted
name-matches are VanillaFoodStorageOracle (Batch 7, passes),
VanillaLoomScreenOracle (Batch 10, renderpearl) and VanillaShulkerBulletOracle
(Batch 3) — all out of scope, as in 26.1.

## Oracle defects withdrawn (9)

The oracle now passes in 26.1.1; each lands as a closed item. O17
(VanillaContainerScreenOracle) and RecipeBookViewOracle pass on client+both
while the server side fails on the client package only — expected, not
breakage, per the 26.1 precedent.

- O01 Ticker, O02 Storage, O07 Enchanting, O11 SimpleContainer, O12
  ItemStackStorage, O13 PalettedContainer, O17 ContainerScreen, O20
  StoredEnchantmentPatch, O24 Recipe.

## What passed

**Drivers with a passing oracle (17)** — declared pass statically per the
26.2 precedent: `test_block_entity_tickers`, `test_block_entity_storage`,
`test_enchanting`, `test_simple_container`, `test_item_stack_storage`,
`test_paletted_container`, `test_container_screen`,
`test_stored_enchantment_patch`, `test_recipe`, `test_chest_load`,
`test_chest_pair`, `test_container_title`, `test_enchantment_components`,
`test_storage_menu`, `test_recipe_book_view`, `test_container_component_names`,
`test_trade_components`.

**Drivers with no oracle (13 pass of 20)** — verdict from reading the driver's
assertions against the 26.1.1 port code and vanilla bytecode:

- `test_crafter_drop_name`, `test_falling_chest`,
  `test_jukebox_saved_clock`, `test_sculk_saved_fields`,
  `test_enchantment_map_validation`, `test_world_pack_enchantment_loader`,
  `test_anvil_turtle_egg`, `test_crafting_priority`, `test_craft_map_take`,
  `test_end_gateway` — spot-checked for fuel/brewing/sign exposure: none.
  The underlying files are 0-diff vs 26.1 and vanilla is unchanged.
- `test_sign_literal` — tests sign TEXT via same-level clone, not pick/place
  item components; the clone path copies the BE directly and is unaffected by
  G33.
- `test_machine_recipe_placement` — the furnace scenes run with ticks FROZEN,
  so no cooking happens in either implementation; the driver tests recipe
  placement (menu transfer), not fuel burn. The coal in the fuel slot is
  inventory content only.
- `test_recipe_book_variants` — compares server-sent recipe-book PACKET
  entries (add/remove ids); the AnyFuel grouping is client-side UI rendering
  and does not change packet content.

**Method-2 checks (17 pass of 38):** lectern, comparator, anvil, grindstone,
smithing, loom, stonecutter, vault, chiseled bookshelf, jukebox, campfire,
beehive, shulker box, sculk save fields, falling-block BE restore, container
titles/names, enchanting offers/click. (Comparator: the `redstone.rs` diff
touches only the `DiodeBlock.neighborChanged` guard — Batch 2's scope — not
the comparator signal formula.)

## Suspicions checked and refuted

- **Sculk water-gate narrowing (Batch 6 flag, second half) — REFUTED, with a
  cross-batch disagreement.** Vanilla `SculkBlock.canPlaceGrowth` requires
  `isAir() || (is(WATER) && fluidState.is(Fluids.WATER))`. B6 reads
  `fluidState.is(Fluids.WATER)` as true for flowing water; it is not.
  `TypedInstance.is(T)` (the method `FluidState.is` resolves to — dumps saved
  in `evidence/javap-TypedInstance-is.txt`) is a type-identity check:
  `typeHolder().value() == arg`. A flowing-water state's type is
  `Fluids.FLOWING_WATER`, not `Fluids.WATER`, so the gate is source-only in
  vanilla — and the port's `is_source()` check at `level/sculk.rs:611`
  MATCHES it. B6's water-gate item (the section just above B6-G06 in
  `docs/sweep/26.1.1/batch-6-worldgen.md`) rests on the misreading; the
  coordinator should arbitrate — the B6 item may need withdrawal. (The
  inhibitor half of B6's flag is confirmed as B4-G37.)
- **Dispenser shears caller-level alive check — REFUTED (observationally
  equivalent).** `ShearsDispenseItemBehavior.tryShearEntity` has NO
  caller-level alive check, but each species' `readyForShearing()` checks
  `isAlive()` (MushroomCow: isAlive + !isBaby; Sheep: isAlive + !isSheared +
  !isBaby; SnowGolem: isAlive + hasPumpkin; Bogged: !isSheared + isAlive).
  The port's caller-level check changes no outcome.
- **Sign wax-on sound playing twice — REFUTED.** The 3003 level event plays no
  sound in this tree (particles only); the dispenser's/sign's explicit play is
  the only one. (Same finding as the G19 fix, on the sign side.)
- **Spawn-egg peaceful gate as a regression — REFUTED.** The port's
  `spawn_egg_allowed` gate (`creatures.rs:7511-7520`) looks like a 26.3-form
  addition, but vanilla 26.1.1 HAS the gate: `SpawnEggItem.spawnMob` checks
  `EntityType.isAllowedInPeaceful()` + `Difficulty.PEACEFUL` (dump saved in
  `evidence/javap/SpawnEggItem.txt`). This is likely the FIX for 26.1 B7-G04,
  not a regression. Spawn eggs are Batch 7's scope — flagged to B7 with the
  jar evidence; NOT minted here. (The 26.1 `creatures.rs` comment claiming
  "26.1 has no eligibility gate" was false — the 26.1 B7 report caught it.)

## Checks by driver class

**Drivers with a passing oracle (17 pass, 0 gap):** see §What passed.

**Drivers oracle-blocked (0 pass, 20 gap):** `test_brewing_fuel` (O05),
`test_dispenser` (O06), `test_menu_click` (O08), `test_recipe_book` (O09),
`test_sculk` (O14), `test_block_entity_anim` (O15),
`test_block_entity_render` (O16), `test_recipe_book_render` (O18),
`test_place_recipe` (O10), `test_trading` (O25), `test_ghost_recipe` (O26),
`test_crafter` (O03), `test_furnace` (O03), `test_brewing` (O04),
`test_enchant_durability` (O19), `test_vehicle_container` (O21),
`test_shulker_space` (O22), `test_spawner` (O23), `test_trial_spawner` (O23),
`test_vault` (O03).

**Drivers with no oracle (13 pass, 7 gap):**

- Gap: `test_merchant_gate` (G25), `test_merchant_crafted` (G25),
  `test_loot_mine_pot_components` (G28), `test_pot_sherd_templates` (G28),
  `test_furnace_loaded_total` (G04), `test_pot_hopper` (G15),
  `test_furnace_output_components` (G01 — FLIPPED from pass in 26.1: its
  corpus smelts with coal fuel, which never burns in 26.1.1).
- Pass: see §What passed.

**Method-2 checks (17 pass, 21 gap):**

- Pass (17): lectern, comparator, anvil, grindstone, smithing, loom,
  stonecutter, vault, chiseled bookshelf, jukebox, campfire, beehive, shulker
  box, sculk save fields, falling-block BE restore, container titles/names,
  enchanting offers/click.
- Gap (21): decorated-pot use (G28), merchant pricing (G21/G23/G24/G25),
  sign NBT (G27), furnace tick (G01/G04/G17), spawner tick (G20), cartography
  (G16), dispenser/dropper (G18/G35), hopper (G08), ender chest (G14), save
  round-trip BE (G15/G27/G28/G36), BE coverage (G13/G15), crafter
  (G09/G10/G29), container loot unpacking (G26), beacon (G31 — flipped from
  pass), brewing recipes (G02 — flipped), brewing tick (G02 — flipped),
  trial spawner tick (G34 — flipped), menu engine (G01/G02 — flipped: the
  furnace fuel and brewing-fuel slot arms refuse every item), composter (G30
  — NEW check), sculk growth inhibition (G37 — NEW check), sign interaction
  (G32/G33 — NEW check).

## Notes for the coordinator

1. **Slice shape change (deliberate).** Per `briefs/common.md`, WITHDRAWN
   re-checks land as closed items and FIXED re-checks as fixed items in
   `slice.json` — the 26.1 run recorded withdrawn items in the report only.
   This slice therefore has 62 items (30 gap + 17 oracle-defect + 1 fixed +
   14 closed), not 47. The behaviour-only restatement is merge.py's job; the
   as-tallied count (95) and the split (37 driver-status / 58 behaviour) are
   both stated in §Tally.
2. **Dedup (carried from the 26.1 record):** B4-O16→B10-O03 (UvMapping),
   B4-O18→B10-O05 (renderpearl), B4-O08→B10-O05, B4-G20→B7-G17,
   B4-G26→B12-G01, B4-G18→B5-G19, B3-G24→B4-G24. B4-G37 cross-references
   B6-G07 (worldgen-side twin) but is a separate item — different code path.
   B4-O25 overlaps Batch 3 (datapackWorldRegistries).
3. **Cross-batch disagreement — B6 water gate.** B6's sculk water-gate item
   rests on `fluidState.is(Fluids.WATER)` being true for flowing water.
   Javap of `TypedInstance.is` proves it is a type-identity check (flowing
   water's type is `FLOWING_WATER`), so vanilla's gate is source-only and
   the port matches. B6's item may need withdrawal; B4's half (live
   `level/sculk.rs:611`) is refuted, not minted. Evidence:
   `evidence/javap-TypedInstance-is.txt`, `evidence/javap-SculkBlock-canPlaceGrowth.txt`.
4. **G17 arbitration.** B4-G17 is UNVERIFIED (masked by G01) — no slice item.
   Restore fuel data first, then check halving via the fast context.
5. **Out-of-scope diffs found while diffing — flag, do not mint:**
   - `game/stations.rs` crossbow charged-projectiles tooltip (multiple line
     uses the plain name; 26.1 used the `[name]` display name for both) →
     Batch 7.
   - `game/commands/items.rs` (970 lines, `/item` command slot machinery) →
     Batch 10 (commands).
   - `horizon-world/src/level/redstone.rs` `DiodeBlock.neighborChanged`
     guard added (26.2 form) → Batch 2.
   - `creatures.rs` `spawn_egg_allowed` peaceful gate (likely the B7-G04
     fix; vanilla 26.1.1 HAS the gate) → Batch 7, with the jar evidence in
     `evidence/javap/SpawnEggItem.txt`.
6. **Nothing was run live.** All 95 checks are static (driver-status from
   the Batch 0 table, no-oracle and method-2 from code + javap + data
   counts). The 17 "passing" drivers are declared pass per the 26.2
   precedent, not by execution. The brewing `stations.rs` test expecting 279
   recipes would fail against the prepared 26.1.1 data — a CI matter, not
   verified here.
7. **Evidence:** `b04/evidence/` — 23 diffs in `diffs/` (incl.
   `game-stations.rs.diff`, added for the Batch 3 brewing-totals flag), oracle
   diffs (`oracle-brewing-*.diff`, `oracle-brewingfuel-*.diff`), javap dumps
   (`javap/` + `javap-SculkBlock-canPlaceGrowth.txt`,
   `javap-SignBlock-useWithoutItem.txt`, `javap-SignBlockEntity-click.txt`,
   `javap-TypedInstance-is.txt`, `javap-FluidState.txt`).
8. **Batch 3 flag — `game/stations.rs` brewing-totals change: VERIFIED, no
   new item.** The change is real (render bars read `total_fuel` /
   `total_brew_time`; tick takes `providers()`), but it is a facet of G02
   (component fuel + data-driven recipes) and G36 (26.3 save form): the
   totals default to vanilla's constants (400/20/1.0) and the mutating paths
   are unreachable without the absent `brewing_fuel` component, so the
   render change has no independently observable divergence. B3's companion
   note is also confirmed: `test_brewing_parity.py:129` reads 26.3 resources
   where the 26.1 tree reads 26.1's — latent driver defect, moot while O04
   blocks the driver; it will matter when the oracle is fixed (the driver's
   corpus generator reads `data/minecraft/recipe/brewing/*.json` from that
   path, so it would build a 26.3 recipe corpus for a 26.1.1 run).
