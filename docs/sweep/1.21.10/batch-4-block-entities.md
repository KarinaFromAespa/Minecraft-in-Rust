# Batch 4 — Block entities / containers — 1.21.10 parity sweep

Static-only batch. Checks + triage only; no fixes, no driver runs, no git in the worktree.

## §0 Execution status

- **Jars**: named 1.21.10 jars at `~/workspace/mcir-work/e6-deltas/versions/1.21.10/jars/`, sha1-verified 2026-11-25: `client-named.jar` 715f58165acfaf8b61ab921064f08c202b41bd0c, `server-game-named.jar` ba1fc89ead95b8ebdb8727f84e2bd487533c8b7f. `server.jar` is a bundler jar and was never javap'd. Java 21, DataVersion 4556, protocol 773.
- **Oracle measurement**: family-scoped, own measurement — no 1.21.10 Batch 0 oracle table has landed (`docs/sweep/1.21.10/` holds only the inventory + merge scripts). 45 files (31 B4 oracles + 8 helpers + 6 second-wave oracles behind the chest/trade/component-names drivers) compiled fixpoint-style (drop failing files, recompile the clean set) against three classpaths (server-game jar + 44 server libs, client jar + 70 launcher libs, both), javac `--release 25`, `-Xmaxerrs 100000`. Results in `b04/scratch/oracle-measure-1.21.10.json`, logs in `b04/scratch/oracle-logs/big-*.log`.
  - Two measurement attempts were discarded as invalid and are not used: (1) a per-file compile whose `-sourcepath` pointed at the package dir instead of the source root (could not resolve same-package helpers); (2) a big-compile run without `-Xmaxerrs`, whose round-1 error lists were truncated at javac's default 100-error cap, misclassifying own-errors as cascades. The run reported here sets `-Xmaxerrs 100000`.
  - Two background jobs were interrupted by a runtime restart mid-session (one oracle run, one javap batch); both were recovered from saved on-disk output and re-run to completion. The ServerLevel-ctor error first seen in the invalid attempt's Ticker log was re-confirmed in the valid measurement.
- **Data**: no prepared 1.21.10 `.local` exists, so data counts are jar-direct: 7,512 data files extracted from `server-game-named.jar` (`b04/evidence/jardata/`).
- **Diff controls**: Rust source diff 1.21.10 vs 26.1.2 and vs 1.21.7 (read-only) for every B4 file; javap `-p -c` on the named 1.21.10 server-game jar (client jar for render/client-only classes). Evidence in `b04/evidence/javap/` (45 dumps) and `b04/evidence/jardata/` (7,512 extracted data files).
- **Drivers**: no driver was run live. The 8 oracle-passing drivers are declared pass per the 26.2 precedent. The 20 no-oracle drivers were read for what they assert (including the merchant corpora, `merchant_gate_corpus.py` / `merchant_crafted_corpus.py`) and verdicted on the code path each exercises.

## Tally (as-tallied, per the 26.2 precedent)

| | total | pass | gap |
|---|---|---|---|
| **All checks (as-tallied)** | **95** | **41** | **54** |
| Behaviour checks | 58 | 33 | 25 |
| Non-behaviour (driver-status / oracle-blocked) checks | 37 | 8 | 29 |
| Driver-status checks (57 drivers) | 57 | 23 | 34 |
| Method-2 checks | 38 | 18 | 20 |

For comparison, 26.1.2 Batch 4: 95 checks, 47 pass, 48 gap (58 behaviour: 30 pass, 28 gap). 1.21.10 is **better on behaviour** (33 pass vs 30: the two merchant drivers and sculk growth inhibition flip to pass) and **worse on oracles** (8 oracle-passing drivers vs 17: nine oracles that passed in 26.1.2 fail in 1.21.10 — see the systemic finding).

Items: **63** in the slice — 28 behaviour gap items (4 major, 24 minor), 25 oracle-defect items (5 major, 20 minor), 1 fixed, 9 closed. No new behaviour items minted; one new oracle item (B4-O27, VanillaTradeComponentsOracle).

## Method

Static-only, mirroring the 26.1.2 Batch 4 template:

1. **Oracle status** from the own family-scoped measurement above (45 files, 3 classpaths). A driver whose oracle fails on the both-cp is oracle-blocked (gap, non-behaviour); a driver whose oracle passes is declared pass per the 26.2 precedent. Client-only oracles (VanillaRecipeBookViewOracle) are judged on client+both, not server.
2. **Behaviour** from javap `-p -c` on the named 1.21.10 server-game jar (client jar for render/client-only classes) + Rust source comparison against the 1.21.10 tree + jar-direct data counts. Every re-check is decided against 1.21.10's OWN jar/data, never the 26.3 base's form (VANILLA-CHECK rule).
3. **No-oracle drivers** read for what they assert, verdicted on the code path each exercises (including corpus reads for the merchant fixtures and the port's own unit tests where they assert the same semantics).
4. **Method-2 checks** (38, mirroring the template): source-level verification of each B4 area not covered by a driver, fresh against 1.21.10 code/jar — not carried from 26.1.2 where the files differ.

## Systemic finding: mixed inheritance — 26.3-form reads survive, but real 1.21.10 adaptations exist

The 1.21.10 tree is **not** a clean copy of the 26.1.2 regression, and it is not a clean 1.21.10 port either. It is both at once, file by file:

- **The 26.3-form component/tag-read pattern survives wholesale** in `providers.rs`, `brewing.rs`, `composter.rs`, `menu.rs`, `block_entity_nbt.rs`, `block_entity_item.rs`, `sign_edit.rs`, `dispenser.rs`, `spawner.rs`, `items.rs` (loot engine). Those files read components/tags that do not exist in 1.21.10 (`cooking_fuel`, `brewing_fuel`, `compostable`, `sign_text_front`/`sign_text_back`, `waxed`, `#extendable_maps`) and get nothing. Consequence: the four 1.21.7 majors that the 1.21.7 tree had fixed are still present — G01 furnaces never light, G02 brewing stands never brew, G16 cartography dead, G30 composter dead. The 1.21.7 tree's fixes (the `FUEL_VALUES` table in `items.rs`, the loot engine reading `"functions"`/`"condition"`) were **not carried** into the 1.21.10 tree.
- **But genuine 1.21.10 adaptations exist where the 26.1.2 tree has none.** The furnace burns by 1 and resets its timer on a recipe-less input (G03, G06 fixed, with 1.21.10-specific comments in `furnace.rs`); sculk growth inhibition counts sensors/shriekers directly with the comment "there is no growth-inhibitors tag in 1.21.10" (G37 fixed); `gamerules.rs` carries an explicit 1.21.10 key map (`spawner_blocks_work` ↔ `spawnerBlocksEnabled`, round-tripped through `apply_compound`/`rules_compound`) (G11 fixed); the merchant path defers gossip delivery to the villager AI phase (`villager_trade_pending` → `MobEvent::VillagerTrade` → `villager_traded_by`, +2 per trade, matching the corpus exactly) and drains crafted counts into stats (`menu_clicks.rs`, `lan_trading.rs`, `trading.rs:607`); the merchant live path is hardcoded `villager_trades` tables (1,313 lines) because 1.21.10 has no trade-set data. Consequence: both merchant drivers pass in 1.21.10 where they gapped in 26.1.2, and G25 (merchant float parsing) is withdrawn — its data-driven path is unreachable with vanilla 1.21.10 data.
- **The oracle picture is worse than 26.1.2's, for a version-shape reason.** Nine oracles that passed in 26.1.2 fail in 1.21.10 (O01, O07, O11, O12, O17, O20, O24, O27, plus VanillaCommandExecOracle). The 1.21.10 oracle sources are the unrepaired 26.3 forms, and 1.21.10's API surface is further from 26.3 than 26.1.2's: `ChunkPos` has public fields, not record accessors (`x()`/`z()` absent); `RecipeHolder` has no `unwrapKey()`; `Recipe.assemble` takes `(CraftingInput, Provider)`; `DedicatedServer`'s ctor takes 7 args; `RegistryLayer` has no `WORLD`; `Reference.bindComponents` and `ItemResult.createItemStack` do not exist; `ServerLevel`'s ctor takes 11 args (incl. `RandomSequences`); `ContainerInput` and `TradeCost` do not exist. The 26.1.2 oracle repairs were never applied to the 1.21.10 oracle sources.

## Behaviour defects (gap items)

### B4-G01 (major, 1.21.7 REPRODUCES — 1.21.7's fix NOT carried) — furnaces never light

Vanilla 1.21.10: `AbstractFurnaceBlockEntity.getBurnDuration` → `FuelValues.burnDuration`; `FuelValues.vanillaBurnTimes` has 63 `Builder.add` calls. `DataComponents` has no `cooking_fuel` (0 hits in the jar). Port: `providers.rs:108-113` `burn_time` reads `i.fuel` (the `cooking_fuel` component), missing → 0; `items.rs:1043-1047` `fuel` comes only from that component. The 1.21.7 tree HAS the `FUEL_VALUES` table (`items.rs:~251`, 62 entries); the 1.21.10 `items.rs` has no `FuelValues`/`BURN_TIMES`/1600. Every furnace, blast furnace and smoker burns nothing.

### B4-G02 (major, 1.21.7 REPRODUCES) — brewing stands never brew

Vanilla 1.21.10: brewing is `PotionBrewing` (addMix/isIngredient/hasMix) + the `#brewing_fuel` item tag (exists, blaze powder only); `BrewingStandBlockEntity` uses `FUEL_USES=20` (bipush 20) and a 400-tick brew (sipush 400). The jar has **0** files under `data/minecraft/recipe/brewing/` and `DataComponents` has no `brewing_fuel`. Port (`brewing.rs`, byte-identical to 26.1.2): `load` reads data-driven brewing recipes (`:203-210`) — of which 0 exist — and `brewing_fuel` reads the absent component (`:423-436`). Nothing ever brews.

### B4-G04 (minor, 1.21.7 REPRODUCES) — furnace completes on `>=`

Vanilla 1.21.10 `AbstractFurnaceBlockEntity.serverTick`: completion only on `==` (bytecode 292 `if_icmpne`). Port `furnace.rs:~145`: `>=`. A loaded furnace with `cookingTotalTime=0` completes instantly in the port; vanilla never completes it. (Masked in practice by G01 — the furnace never lights — but the comparison itself is wrong.)

### B4-G05 (minor, 1.21.7 REPRODUCES) — fuel remainder dropped when the fuel stack remains

Vanilla 1.21.10 furnace tick (bytecode 219-246): the fuel stack shrinks; the remainder goes into the slot **only if the stack empties**, else it is lost. Port `furnace.rs:~120-125`: drops the remainder when the stack remains. (Datapack-only in practice — no vanilla fuel has a remainder in 1.21.10 — but the behaviour is inverted.)

### B4-G08 (minor, 1.21.7 REPRODUCES) — hopper suction bowl is 0.125..0.875

Vanilla 1.21.10 `HopperBlockEntity.SUCK_AABB` = `Block.column(16.0, 11.0, 32.0)` → (0, 11/16, 0)-(1, 2, 1). Port `machines.rs:972-977`: the entity-catch bowl is 0.125..0.875. (The tick-path `hopper_suck` at `:1159-1177` uses the full SUCK_AABB — the gap is only the entity-catch bowl, so pickup above the hopper misses items vanilla catches.)

### B4-G09 (minor, 1.21.7 REPRODUCES) — crafter eject reaches blocks only

Vanilla 1.21.10 `CrafterBlock.dispenseItem` uses `HopperBlockEntity.getContainerAt` (block + entity containers). Port `crafter.rs:149-152` (byte-identical to 26.1.2): eject uses block-only `Containers::view`. A crafter facing a chest minecart, chest boat or hopper minecart ejects nothing.

### B4-G10 (minor, 1.21.7 REPRODUCES) — crafter trigger checks only the local player

Vanilla 1.21.10 crafter trigger: `getEntitiesOfClass(ServerPlayer, AABB.ofSize 17,17,17)`, triggers `CRAFTER_RECIPE_CRAFTED` per player in the box. Port `crafter.rs:188` (byte-identical to 26.1.2): checks `self.player` only. LAN guests standing at the crafter never get the advancement trigger.

### B4-G13 (minor, 1.21.7 REPRODUCES) — no bed block entity

Vanilla 1.21.10 has `BedBlockEntity` (colour field) and `BlockEntityType.BED`. Port: `containers.rs` `for_block` (`:376-518`) has no bed arm; `container_size` (`:320`) covers only chest/barrel/ender_chest/hopper/dispenser/dropper/shulker/copper_chest; `block_entity_nbt.rs` `TYPES` (47 entries) has no `"minecraft:bed"`; `save.rs:1010` `decode_block_entity` does `BlockEntity::for_block(block)?`, dropping beds. Renamed beds lose their name; chunk saves carry no bed data.

### B4-G14 (minor, 1.21.7 REPRODUCES) — ender chest opens on LAN guests

Port `storage.rs:157` `open_ender_chest` swaps the **host's** `ender_items` into the shared block entity. A LAN guest opening an ender chest sees (and edits) the host's ender inventory instead of their own.

### B4-G15 (minor, 1.21.7 REPRODUCES) — no decorated-pot block entity

Vanilla 1.21.10 has `DecoratedPotBlockEntity` (implements `ContainerSingleItem$BlockContainerSingleItem` + `RandomizableContainer`). Port: `containers.rs` `for_block` has no decorated-pot arm. Hoppers/droppers cannot see decorated pots; no container view exists.

### B4-G16 (major, 1.21.7 REPRODUCES) — cartography table gated on a nonexistent tag

Vanilla 1.21.10 `CartographyTableMenu`: 0 `ItemTags` references; `setupResultSlot` checks `Items.PAPER` directly. The jar has no `#extendable_maps` tag. Port `menu.rs:1290`: the map-extend path is gated on the `extendable_maps` tag. Paper zoom-out never works.

### B4-G18 (minor, 1.21.7 REPRODUCES) — dispenser mud uses the wrong tag spelling

Vanilla 1.21.10 `DispenseItemBehavior$15` (the mud behaviour; `$14` in 26.1.2) uses `BlockTags.CONVERTABLE_TO_MUD`; the jar tag is `convertable_to_mud` (`convertible_to_mud` does not exist). Port `dispenser.rs:2251`: checks `"minecraft:convertible_to_mud"`. A dispenser with a water bottle never turns dirt into mud.

### B4-G20 (minor, 26.1.2 REPRODUCES) — spawner `setEntityData` merge

Vanilla 1.21.10 `Spawner` has **only** `setEntityId`; `SpawnerBlockEntity` has no `setEntityData`/`overrideEntityData`. Port `spawner.rs:464` `set_entity_data` (and `mobs/spawner.rs:278,429`) merges egg entity data into the spawner. The port comment at `spawner.rs:446` claims 1.21.10 has `setEntityData` — wrong per the jar. A spawn egg used on a spawner merges NBT vanilla discards.

### B4-G21 (minor, 26.1.2 REPRODUCES) — merchant result slot blocks pickup

Vanilla 1.21.10 `MerchantResultSlot` has **no** `mayPickup` override (mayPlace, remove, onQuickCraft, checkTakeAchievements, onTake only). Port `menu.rs:1505` `may_pickup`: the `MerchantResult` arm gates pickup on `o.satisfied_by(...)` (checked at `:2012/:2026/:2902/:3003`). Takes vanilla allows (e.g. with the payment already consumed by a prior take in the same click sequence) are blocked.

### B4-G22 (minor, 26.1.2 REPRODUCES) — exploration-map check omits the vanilla exclusion

Vanilla 1.21.10 `MapExtendingRecipe.matches`: `isExplorationMap` check (bytecode 41-48) first, then scale < 4. Port `crafting.rs:900-917`: ring + scale < 4 only, no exploration-map check. An exploration map can be zoomed out in the port; vanilla refuses.

### B4-G23 (minor, 26.1.2 REPRODUCES) — hero discount computed in f32

Vanilla 1.21.10 `Villager.updateSpecialPrices` hero part: `0.3d + 0.0625d * amplifier` (double, `ldc2_w`) with `Math.floor(double)`. Port `merchant.rs:116`: `f32::from(0.3f32 + 0.0625f32 * a as f32)` widened to f64. At high amplifier the f32 rounding changes the floored discount by 1 emerald. (The reputation part of the same function matches vanilla exactly — f32 multiply + floor, verified against the bytecode.)

### B4-G24 (minor, 26.1.2 REPRODUCES) — villager levels up immediately

Vanilla 1.21.10 `Villager.rewardTradeXp`: adds XP immediately, but the level-up is deferred — `updateMerchantTimer=40`, `increaseProfessionLevelOnUpdate=true` (only when `shouldIncreaseLevel`). Port `merchant.rs:405`: `level += 1` immediately on trade. (The gossip delivery IS correctly deferred — see merchant_gate in "What passed" — but the level-up is not.)

### B4-G26 (minor, 26.1.2 REPRODUCES — 1.21.7's fix NOT carried) — loot engine reads 26.3 key names

Vanilla 1.21.10 loot tables: 1,320 files; `"functions"` lists in 950; `"modifier"` in 0; condition objects use the string `"condition"` key (1,536 objects). Sample `blocks/diamond_ore.json` confirms the format (`"condition": "minecraft:match_tool"`, `"functions": [{"function": "minecraft:apply_bonus"...}]`). Port `items.rs`: modifiers read from `["modifier"]` only (`:1873-1874`, `:2146`, `:2287`); `condition()` reads `o.get("type")` only (`:1991`); `modifiers()` reads `f["type"].or(f["function"])` (`:2311`) but is never handed the `"functions"` list. The 1.21.7 tree FIXED this (reads `"functions"`, condition reads `"type"` OR `"condition"` at `:2126-2127`); the 1.21.10 tree did NOT carry the fix. No gated loot drop in 1.21.10 can work (diamond ore with Fortune, silk-touch paths, entity drops with conditions).

### B4-G27 (minor, 26.1.2 REPRODUCES) — furnace int times + sign `allow_op_features` in saves

Vanilla 1.21.10 `AbstractFurnaceBlockEntity.saveAdditional`: 4× `putShort`, no `speed_multiplier`. Vanilla `SignBlockEntity.saveAdditional`: `front_text`, `back_text`, `is_waxed` only — no `allow_op_features` field exists on the class. Port `block_entity_nbt.rs`: the furnace arm writes int times + `speed_multiplier` (`:383`); the sign arm keeps `allow_op_features` (`:458-459`). Port-written saves carry fields vanilla never writes (and vanilla-written saves lose nothing the port needs — the port's fields are invented).

### B4-G28 (minor, 26.1.2 REPRODUCES) — pot sherds saved in compound form

Vanilla 1.21.10 `PotDecorations.CODEC` = `byNameCodec().sizeLimitedListOf(4).xmap(...)` — **list** form. Port `block_entity_nbt.rs:663-672`: the pot sherds arm writes the compound form. A port-written decorated pot is not one vanilla 1.21.10 reads (and vice versa).

### B4-G29 (minor, 26.1.2 REPRODUCES) — crafter trigger reads the `recipes` key

Vanilla 1.21.10 `RecipeCraftedTrigger$TriggerInstance` (package `net.minecraft.advancements.critereon` — Mojang's misspelling) has the singular `recipeId` field; the advancement `adventure/crafters_crafting_crafters.json` uses `"recipe_id": "minecraft:crafter"`. Port `crafter.rs:206`: `recipe_matches(&c["recipes"])`. The crafter trigger never matches a vanilla advancement criterion.

### B4-G30 (major, 26.1.2 REPRODUCES) — composter never composts

Vanilla 1.21.10: `ComposterBlock.COMPOSTABLES` is a hardcoded `Object2FloatMap`. `DataComponents` has no `compostable`. Port (`composter.rs`, byte-identical to 26.1.2): reads the absent `compostable` component. Nothing ever composts. (The driver `test_composter_parity.py` belongs to Batch 5's tally per the 26.1.2 precedent — it references VanillaDispenserOracle and is not counted in B4's 57; the behaviour is covered here.)

### B4-G31 (minor, 26.1.2 REPRODUCES) — beacon validates effects and kicks LAN guests

Vanilla 1.21.10: `BeaconBlockEntity` has no `validateEffects`/`getRequiredLevelsFor`; `BeaconMenu.updateEffects` checks payment → encodes both → sets data → removes payment, with no validation. Port `menu.rs`: `beacon_required_levels` (`:306`) and `beacon_update` validation (`:1203-1215`, returns false when invalid); `lan_stations.rs:768` kicks with "Player {} tried to set invalid beacon effects". A LAN guest setting effects the port deems invalid is kicked; vanilla accepts any selection with payment.

### B4-G32 (minor, 26.1.2 REPRODUCES) — sign click actions never run

Vanilla 1.21.10 `SignBlock.useWithoutItem`: `executeClickCommandsIfPresent` runs **first** (bytecode 80), before the waxed check (87) — unconditionally. Port `sign_edit.rs:166-213`: click actions are gated on `allow_op_features` (default false), "Click actions are disabled" (`:213`). Clickable signs never run their commands.

### B4-G33 (minor, 26.1.2 REPRODUCES, narrowed on LAN) — sign pick/place preserves text and wax via nonexistent components

Vanilla 1.21.10 `DataComponents` has no `sign_text_front`/`sign_text_back`/`waxed`. Port `block_entity_item.rs:192-193,231-232,251,294`: sign pick/place preserves text and wax via those components. Locally the text/wax survives a pick-and-place that vanilla drops. Narrowed on LAN exactly as in 26.1.2: the protocol layer drops them at wire encode — `horizon-protocol/src/item.rs:1306-1316` `write_patch` filters by `ids::component(kind) >= 0`, and the three components have no 1.21.10 registry id — so LAN guests receive a plain sign item.

### B4-G34 (minor, 26.1.2 REPRODUCES) — trial spawner op egg merges entity data

Vanilla 1.21.10 `TrialSpawnerBlockEntity` has only `setEntityId`. Port `spawner.rs:450` `override_entity_data` (and `game/trial_spawner.rs:405,629`, `mobs/trial_spawner.rs`) merges egg entity data into the trial spawner. Same defect as G20 on the trial-spawner path.

### B4-G35 (minor, 26.1.2 REPRODUCES) — dispensed spawn egg on a null spawn is returned unshrunk

Vanilla 1.21.10 `DispenseItemBehavior$1` (spawn egg): `getType` null → return stack unchanged; else `EntityType.spawn` (result popped) → `shrink(1)` → `gameEvent(ENTITY_PLACE)` → return stack. Note vanilla **always** shrinks and fires the event once a type exists, even if the spawn itself produces nothing. Port `dispenser.rs:1011-1045`: on a null spawn, returns unshrunk with a CLICK and no event (`:1024-1026`). A dispensed egg that fails to spawn keeps its count and plays no placement event.

### B4-G36 (minor, 26.1.2 REPRODUCES) — brewing-stand save is the 26.3 form

Vanilla 1.21.10 `BrewingStandBlockEntity.saveAdditional`: `BrewTime` short + `Fuel` byte only. Port `block_entity_nbt.rs:509-516`: int `BrewTime` + `total_brew_time` + `speed_multiplier`. Port-written brewing stands carry invented fields.

## 1.21.7 re-check verdicts (PRIMARY, 42 items)

| Item | Verdict | Note |
|---|---|---|
| B4-G01 | REPRODUCES | major; 1.21.7's FuelValues fix not carried |
| B4-G02 | REPRODUCES | major |
| B4-G03 | FIXED | port grows output by 1, matching vanilla 1.21.10 burn() |
| B4-G04 | REPRODUCES | minor |
| B4-G05 | REPRODUCES | minor |
| B4-G06 | FIXED | port resets timer on no recipe, matching vanilla 1.21.10 |
| B4-G07 | NEVER EXISTED | vanilla 1.21.10 getType returns null (no defaultType); dispense returns the stack unchanged — the port matches; the 1.21.7 premise "vanilla spawns the default mob" is false for 1.21.10 |
| B4-G08 | REPRODUCES | minor |
| B4-G09 | REPRODUCES | minor |
| B4-G10 | REPRODUCES | minor |
| B4-G11 | FIXED | vanilla 1.21.10 HAS the rule as `spawnerBlocksEnabled` (RULE_SPAWNER_BLOCKS_ENABLED); the port maps it explicitly in gamerules.rs (vanilla_key/from_vanilla_key, round-tripped on load/save) |
| B4-G12 | WITHDRAWN | vanilla 1.21.10 bypasses the lapis check in creative (hasInfiniteMaterials at bytecode 74-78); the port matches — the 1.21.7 premise was the 1.21.7 form, not 1.21.10's |
| B4-G13 | REPRODUCES | minor |
| B4-G14 | REPRODUCES | minor |
| B4-G15 | REPRODUCES | minor |
| B4-G16 | REPRODUCES | major |
| B4-O01 | REPRODUCES | VanillaBlockEntityTickerOracle: ServerLevel ctor 10 args passed, 11 required (incl. RandomSequences) — reopened; passed in 26.1.2 |
| B4-O02 | FIXED | VanillaBlockEntityStorageOracle passes in 1.21.10 |
| B4-O03 | REPRODUCES | VanillaFurnaceOracle: own error (`RecipeHolder.unwrapKey()` absent, :58, 1 error) + cascade on broken VanillaBlockUpdateOracle |
| B4-O04 | REPRODUCES | VanillaBrewingOracle: `BrewingInput` import, :23, 14 errors — identical to 26.1.2 |
| B4-O05 | REPRODUCES | VanillaBrewingFuelOracle: `BrewingFuel` import, :13, 9 errors — identical to 26.1.2 |
| B4-O06 | REPRODUCES | VanillaDispenserOracle: first error differs (gamerules package import, :27) but the EntityTypes cause is present (:212, 42 errors) |
| B4-O07 | REPRODUCES | VanillaEnchantingOracle: `RegistryLayer.WORLD` + `DATA_COMPONENT_INITIALIZERS` absent, 4 errors — reopened; passed in 26.1.2 |
| B4-O08 | REPRODUCES | VanillaMenuOracle: first error differs (DedicatedServer ctor, :65) but the Prediction cause is present (:138, 10 errors) |
| B4-O09 | REPRODUCES | VanillaRecipeBookOracle: cascade on MenuOracle |
| B4-O10 | REPRODUCES | VanillaPlaceRecipeOracle: cascade on MenuOracle |
| B4-O11 | REPRODUCES | VanillaSimpleContainerOracle: `Reference.bindComponents` absent, 1 error — reopened; passed in 26.1.2 |
| B4-O12 | REPRODUCES | VanillaItemStackStorageOracle: `Reference.bindComponents` absent, 1 error — reopened; passed in 26.1.2 |
| B4-O13 | FIXED | VanillaPalettedContainerOracle passes in 1.21.10 |
| B4-O14 | REPRODUCES | VanillaSculkOracle: first error differs (gizmos import, :34) but the datapackWorldRegistries cause is present (:327, 51 errors) |
| B4-O15 | REPRODUCES | VanillaBlockEntityAnimOracle: first error differs (book-model package, :9) but the fuelValues cause is present (:102, 21 errors) |
| B4-O16 | REPRODUCES | VanillaBlockEntityRenderOracle: first error differs (banner-model package, :14) but the UvMapping cause is present (:23, 42 errors); dedup to B10-O03 for the UvMapping facet |
| B4-O17 | REPRODUCES | VanillaContainerScreenOracle: `ContainerInput` absent (×5), 5 errors — reopened; passed in 26.1.2 |
| B4-O18 | REPRODUCES | VanillaRecipeBookScreenOracle: renderpearl import, :4, 35 errors — identical to 26.1.2; dedup to B10-O05 |
| B4-O19 | REPRODUCES | VanillaEnchantDurabilityOracle: cascade on BlockUpdateOracle |
| B4-O20 | REPRODUCES | VanillaStoredEnchantmentPatchOracle: `ItemResult.createItemStack` absent, 1 error — reopened; passed in 26.1.2 |
| B4-O21 | REPRODUCES | VanillaVehicleContainerOracle: `EntitySpawnRequest` import, :20, 5 errors — identical to 26.1.2 |
| B4-O22 | REPRODUCES | VanillaShulkerSpaceOracle: `EntityTypes` import, :10, 2 errors — identical to 26.1.2 |
| B4-O23 | REPRODUCES | VanillaNaturalSpawnOracle: first error differs (`ChunkPos.x()` accessor absent, :274; 22 errors incl. overrides of absent methods, TagLoader/RegistryDataLoader, EntityTypes) — the 26.1.2 getNextEntityId cause is NOT in this oracle source (no such override); the oracle still fails |
| B4-O24 | REPRODUCES | VanillaRecipeOracle: `assemble(CraftingInput, Provider)` needs 2 args + `RecipeHolder.unwrapKey()` absent, 10 errors — reopened; passed in 26.1.2 |
| B4-O25 | REPRODUCES | VanillaTradingOracle: first error differs (gizmos import, :30) but the datapackWorldRegistries cause is present (:254, 86 errors) |
| B4-O26 | REPRODUCES | VanillaGhostRecipeOracle: `ContextMap.builder()` absent, :56, 1 error — identical to 26.1.2 |

PRIMARY tally: **35 REPRODUCES, 5 FIXED (G03, G06, G11, O02, O13), 1 NEVER EXISTED (G07), 1 WITHDRAWN (G12).**

## 26.1.2 re-check verdicts (SECONDARY, 62 items)

| Item | 26.1.2 status | 1.21.10 verdict | Note |
|---|---|---|---|
| B4-G01 | gap | REPRODUCES | major |
| B4-G02 | gap | REPRODUCES | major |
| B4-G03 | closed | FIXED (still) | |
| B4-G04 | gap | REPRODUCES | |
| B4-G05 | gap | REPRODUCES | |
| B4-G06 | closed | FIXED (still) | |
| B4-G07 | closed | NEVER EXISTED (still) | |
| B4-G08 | gap | REPRODUCES | |
| B4-G09 | gap | REPRODUCES | |
| B4-G10 | gap | REPRODUCES | |
| B4-G11 | closed | FIXED (still) | via the gamerules.rs name map (see PRIMARY table) |
| B4-G12 | closed | WITHDRAWN (still) | |
| B4-G13 | gap | REPRODUCES | |
| B4-G14 | gap | REPRODUCES | |
| B4-G15 | gap | REPRODUCES | |
| B4-G16 | gap | REPRODUCES | major |
| B4-G18 | gap | REPRODUCES | |
| B4-G19 | fixed | FIXED (still) | 3003 routes to particles only; catch-all sound table (game.rs:4471-4505) has no 3003 entry; wax sound plays once via the explicit play |
| B4-G20 | gap | REPRODUCES | |
| B4-G21 | gap | REPRODUCES | |
| B4-G22 | gap | REPRODUCES | |
| B4-G23 | gap | REPRODUCES | |
| B4-G24 | gap | REPRODUCES | |
| B4-G25 | gap (major) | WITHDRAWN | vanilla 1.21.10 has NO trade-set/villager-trade data files (0 exist; trades are hardcoded in `VillagerTrades`); the int_provider zeroing survives only in the data-driven path (`merchant.rs:660` `offers_from_set`), unreachable with vanilla data — the live path is the hardcoded `villager_trades` tables. Dead code, noted in the report, no item. The two merchant drivers pass in 1.21.10 (see "What passed"). |
| B4-G26 | gap | REPRODUCES | |
| B4-G27 | gap | REPRODUCES | |
| B4-G28 | gap | REPRODUCES | |
| B4-G29 | gap | REPRODUCES | |
| B4-G30 | gap | REPRODUCES | major |
| B4-G31 | gap | REPRODUCES | |
| B4-G32 | gap | REPRODUCES | |
| B4-G33 | gap | REPRODUCES | LAN-narrowing facet confirmed for 1.21.10 (write_patch drops the components) |
| B4-G34 | gap | REPRODUCES | |
| B4-G35 | gap | REPRODUCES | |
| B4-G36 | gap | REPRODUCES | |
| B4-G37 | gap | FIXED | `sculk_can_place_growth` counts sensors/shriekers directly (false at matched >= 3), box (-4..4, 0..2, -4..4), source-only water gate — matches vanilla `SculkBlock.canPlaceGrowth` exactly |
| B4-O01 | closed | REPRODUCES | reopened (see PRIMARY table) |
| B4-O02 | closed | FIXED (still) | |
| B4-O03 | oracle-defect | REPRODUCES | |
| B4-O04 | oracle-defect | REPRODUCES | |
| B4-O05 | oracle-defect | REPRODUCES | |
| B4-O06 | oracle-defect | REPRODUCES | major |
| B4-O07 | closed | REPRODUCES | reopened |
| B4-O08 | oracle-defect | REPRODUCES | major |
| B4-O09 | oracle-defect | REPRODUCES | |
| B4-O10 | oracle-defect | REPRODUCES | |
| B4-O11 | closed | REPRODUCES | reopened |
| B4-O12 | closed | REPRODUCES | reopened |
| B4-O13 | closed | FIXED (still) | |
| B4-O14 | oracle-defect | REPRODUCES | |
| B4-O15 | oracle-defect | REPRODUCES | |
| B4-O16 | oracle-defect | REPRODUCES | |
| B4-O17 | closed | REPRODUCES | reopened; major |
| B4-O18 | oracle-defect | REPRODUCES | |
| B4-O19 | oracle-defect | REPRODUCES | |
| B4-O20 | closed | REPRODUCES | reopened |
| B4-O21 | oracle-defect | REPRODUCES | |
| B4-O22 | oracle-defect | REPRODUCES | |
| B4-O23 | oracle-defect | REPRODUCES | major |
| B4-O24 | closed | REPRODUCES | reopened; major |
| B4-O25 | oracle-defect | REPRODUCES | |
| B4-O26 | oracle-defect | REPRODUCES | |

SECONDARY tally: **52 REPRODUCES, 7 FIXED (G03, G06, G11, G19, G37, O02, O13), 1 NEVER EXISTED (G07), 2 WITHDRAWN (G12, G25).**

G17 (smoker/blast halving, masked by G01) stays UNVERIFIED and carries no slice item, as in 26.1.2 — `providers.rs` threads `fast` through the `fast_cooking` condition (byte-identical to 26.1.2), but with no furnace ever lighting the halving cannot be observed.

## Oracle defects

25 oracle-defect items. Error counts are the both-classpath counts from the own measurement (`b04/scratch/oracle-measure-1.21.10.json`); "first error" is the both-cp first error.

| Item | Oracle | Errors (both) | First error / cause |
|---|---|---|---|
| B4-O01 | VanillaBlockEntityTickerOracle | 1 | :70 ServerLevel ctor — 10 args passed, 11 required (incl. RandomSequences). REOPENED (passed in 26.1.2). Blocks test_block_entity_tickers. |
| B4-O03 | VanillaFurnaceOracle | 1 | :58 `RecipeHolder.unwrapKey()` absent + cascade on broken VanillaBlockUpdateOracle. Blocks test_furnace. |
| B4-O04 | VanillaBrewingOracle | 14 | :23 `import BrewingInput` — class absent from the jar. Blocks test_brewing. |
| B4-O05 | VanillaBrewingFuelOracle | 9 | :13 `import BrewingFuel` — class absent from the jar. Blocks test_brewing_fuel. |
| B4-O06 | VanillaDispenserOracle | 42 | :27 gamerules package import (first); EntityTypes cause at :212. Blocks test_dispenser, test_crafter. **Major.** |
| B4-O07 | VanillaEnchantingOracle | 4 | :62 `RegistryLayer.WORLD` absent; :67 `DATA_COMPONENT_INITIALIZERS` absent. REOPENED. Blocks test_enchanting. |
| B4-O08 | VanillaMenuOracle | 10 | :65 DedicatedServer ctor (7 args in 1.21.10, 10 passed); Prediction cause at :138; EntityTypes, ContainerInput, gamerules pkg. Blocks test_menu_click. **Major.** |
| B4-O09 | VanillaRecipeBookOracle | cascade | Depends on broken VanillaMenuOracle. Blocks test_recipe_book. |
| B4-O10 | VanillaPlaceRecipeOracle | cascade | Depends on broken VanillaMenuOracle. Blocks test_place_recipe. |
| B4-O11 | VanillaSimpleContainerOracle | 1 | :25 `Reference.bindComponents(DataComponentMap)` absent. REOPENED. Blocks test_simple_container. |
| B4-O12 | VanillaItemStackStorageOracle | 1 | :20 `Reference.bindComponents(DataComponentMap)` absent. REOPENED. Blocks test_item_stack_storage. |
| B4-O14 | VanillaSculkOracle | 51 | :34 gizmos import (first); datapackWorldRegistries cause at :327. Blocks test_sculk. |
| B4-O15 | VanillaBlockEntityAnimOracle | 21 | :9 book-model package import (first); fuelValues cause at :102. Blocks test_block_entity_anim. |
| B4-O16 | VanillaBlockEntityRenderOracle | 42 | :14 banner-model package import (first); UvMapping cause at :23. Blocks test_block_entity_render. Dedup to B10-O03 for the UvMapping facet. |
| B4-O17 | VanillaContainerScreenOracle | 5 | :55 `ContainerInput` absent (×5). REOPENED. Blocks test_container_screen. **Major.** |
| B4-O18 | VanillaRecipeBookScreenOracle | 35 | :4 renderpearl import. Blocks test_recipe_book_render. Dedup to B10-O05. |
| B4-O19 | VanillaEnchantDurabilityOracle | cascade | Depends on broken VanillaBlockUpdateOracle. Blocks test_enchant_durability. |
| B4-O20 | VanillaStoredEnchantmentPatchOracle | 1 | :18 `ItemResult.createItemStack(int)` absent. REOPENED. Blocks test_stored_enchantment_patch. |
| B4-O21 | VanillaVehicleContainerOracle | 5 | :20 `import EntitySpawnRequest` — class absent from the jar. Blocks test_vehicle_container. |
| B4-O22 | VanillaShulkerSpaceOracle | 2 | :10 `import EntityTypes` — class absent from the jar. Blocks test_shulker_space. |
| B4-O23 | VanillaNaturalSpawnOracle | 22 | :274 `ChunkPos.x()` accessor absent (1.21.10 ChunkPos has fields); also overrides of absent methods (canSpawnEntitiesInChunk, getAllStructuresAt, startsForStructure, getBiome), TagLoader/RegistryDataLoader, EntityTypes, spawnMobsForChunkGeneration signature. Blocks test_spawner, test_trial_spawner, test_vault. **Major.** |
| B4-O24 | VanillaRecipeOracle | 10 | :55 `assemble(CraftingInput, Provider)` needs 2 args; `RecipeHolder.unwrapKey()` absent; `SmithingTransformRecipe.MAP_CODEC` absent. REOPENED. Blocks test_recipe. **Major.** |
| B4-O25 | VanillaTradingOracle | 86 | :30 gizmos import (first); datapackWorldRegistries cause at :254. Blocks test_trading. |
| B4-O26 | VanillaGhostRecipeOracle | 1 | :56 `ContextMap.builder()` absent. Blocks test_ghost_recipe. |
| B4-O27 | VanillaTradeComponentsOracle | 7 | **NEW.** :35 `TagLoader.buildUpdatedLookups` / `RegistryLayer.WORLD`; :55 `SetEnchantmentsFunction.MAP_CODEC` absent; :57-58 `TradeCost` (a 26.1-only class) absent. Blocks test_trade_components. |

Cross-batch oracle notes (no B4 item minted — see "Notes for the coordinator"):
- **VanillaCommandExecOracle** (Batch 9's oracle): FAILS in 1.21.10, 45 errors, first `:8` `com.mojang.authlib.services` package absent (plus the villager package move, `datapackWorldRegistries`, `ContainerInput`, `EntityTypes`, and 26.x server ctors). Blocks test_container_component_names.
- **VanillaBlockUpdateOracle** (Batch 2's item): FAILS, 63 errors, first `:16` gamerules package import. B4 cascades on it (O03, O19).
- **VanillaLevelShapeOracle**: FAILS, 2 errors, `:16` `world.attribute` package absent.
- **VanillaMobAiOracle**: FAILS, 103 errors, `:31` gizmos package absent.
- **VanillaDragonContactsOracle** (Batch 3's oracle, a CommandExec dependency): FAILS, 5 errors — `EnderDragonPart` / `Enderman` imports (1.21.10 has `EnderMan`, capital M).
- **ParityFont** (helper): FAILS on all classpaths (server: blaze3d.font package, 41 errors; client+both: `VanillaPackResources.asResourceManager()` absent, 2 errors). Contributes to O16.

## Oracle defects withdrawn (2)

- **B4-O02** — VanillaBlockEntityStorageOracle passes in 1.21.10 (all three classpaths). test_block_entity_storage stays passing.
- **B4-O13** — VanillaPalettedContainerOracle passes in 1.21.10 (all three classpaths). test_paletted_container stays passing.

Also passing in 1.21.10 (no 26.1.2 item — they passed there too): VanillaChestLoadOracle, VanillaChestPairOracle, VanillaContainerTitleOracle, VanillaEnchantmentComponentsOracle, VanillaFoodStorageOracle, VanillaHeightmapOracle, VanillaRecipeBookViewOracle (client+both; server fails on the client package only, which is expected, not breakage), VanillaStateCodec, VanillaStorageMenuOracle, ParityData, SeedAgent.

## What passed

### Oracle-passing drivers (8)

test_block_entity_storage, test_paletted_container, test_chest_load, test_chest_pair, test_container_title, test_enchantment_components, test_storage_menu, test_recipe_book_view. Declared pass per the 26.2 precedent (their oracles pass in the own measurement).

### No-oracle drivers passing (15)

- **test_crafter_drop_name** — the 1.21.10 crafter loot table (`blocks/crafter.json`) has no `"functions"` — a plain item entry with a `survives_explosion` condition the port's `conditions()` reads — so the port drops a plain crafter exactly as vanilla does. G26 does not touch this table.
- **test_falling_chest** — `falling_blocks.rs` + `native_falling.rs` byte-identical to 26.1.2; the chest arm exists in `containers.rs` `for_block`, so a landed chest restores its block entity and inventory.
- **test_jukebox_saved_clock** — the saved-clock mechanism is present (`ticks_since_song_started` Long; empty → absent, start → 0, tick → 1), asserted by the port's own `saved_song_clock_tests` integration test.
- **test_sculk_saved_fields** — the NBT arms carry `last_vibration_frequency` + `listener` (sensor), `warning_level` + `listener` (shrieker), `cursors` (catalyst); placement tags survive ticking and reload.
- **test_enchantment_map_validation** — `horizon-items/src/enchant.rs` byte-identical to 26.1.2.
- **test_world_pack_enchantment_loader** — same file, same carry.
- **test_anvil_turtle_egg** — falling-block tick byte-identical to 26.1.2 (see test_falling_chest).
- **test_crafting_priority** — `crafting.rs` carries the first-match sort key ("vanilla's recipe registration order, which first-match…").
- **test_craft_map_take** — `menu.rs:1310` `MapItem.onCraftedPostProcess`, shared by menu takes and system crafting. (G22 is about the recipe match, not the take post-processing.)
- **test_end_gateway** — the NBT arm exists (`block_entity_nbt.rs:438`) with `ExactTeleport` handling, citing `TheEndGatewayBlockEntity.loadAdditional`.
- **test_sign_literal** — the sign NBT arm writes `front_text`/`back_text` (`:461-462`) with a round-trip test for literal messages. (G27/G32/G33 concern invented fields, click gating and components — not the literal strings.)
- **test_machine_recipe_placement** — `Menu::place_recipe` exists (`menu/place.rs:380`).
- **test_recipe_book_variants** — `RecipeBookAdd` with per-display `ClientboundRecipeBookAddPacket.Entry` exists (`horizon-protocol/src/packets/recipes.rs:425-473`).
- **test_merchant_gate** — **flips to pass in 1.21.10** (gapped in 26.1.2 on G25). The corpus (`merchant_gate_corpus.py`) uses explicit NBT offers, not data-driven trade sets, so G25 never applied to it. What it tests — deferred gossip delivery (0 while frozen / NoAI, +2 at the first AI phase, special price −2, immediate XP) — the port implements exactly: `trading.rs:604` `villager_trade_pending` retains the player; `mobs/ai.rs:2091` (inside `Villager.customServerAiStep`) takes it and pushes `MobEvent::VillagerTrade`; `villagers.rs` `villager_traded_by` adds `GossipType::Trading` with `REPUTATION_CHANGE_PER_TRADE = 2`, entity event 14, and a price refresh; `merchant.rs:114` `update_special_prices` matches vanilla's f32 reputation formula exactly (verified against the `Villager.updateSpecialPrices` bytecode). G24 (immediate level-up) does not affect this corpus — it never observes the villager's level, and its few 1-tick pumps never elapse the 40-tick timer.
- **test_merchant_crafted** — **flips to pass in 1.21.10** (gapped in 26.1.2 on G25). Same NBT-offers reasoning. The port's own unit test `merchant_takes_record_crafted_counts_once` (`menu.rs:3553`) asserts exactly the fixture's semantics — preview records nothing, payment 3 is rejected (no crafted counts, payment untouched, uses 0), two Pickup takes record (glass, 3) twice — and `crafted_counts` drains into `award_stat("crafted", …)` for host (`menu_clicks.rs:578`) and LAN guests (`lan_trading.rs:322`), with `traded_with_villager` awarded at `trading.rs:607`.

### Method-2 passes (18)

Lectern (books.rs lectern handling + `containers::Lectern`; page-turn sound `game.rs:1043`), comparator (`redstone.rs` `analog_output` → `vanilla_analog_output`, state sources + container fullness), anvil (`anvil.rs` byte-identical to 26.1.2 and 1.21.7), grindstone (stations.rs arms + `grindstone_xp_base`), smithing (stations.rs arms), loom (loom.rs screen + pattern list; `LoomResult` handling), stonecutter (stations.rs arms + `StoneResult` onTake), vault (`vault.rs` byte-identical to both), chiseled bookshelf (byte-identical to both), jukebox (see test_jukebox_saved_clock), campfire (`campfire.rs` byte-identical to 26.1.2), beehive (beehive.rs hive trait + honey level; comparator reads `honey_level`), shulker box (`storage.rs` `is_shulker_box` + `ShulkerBoxSlot.mayPlace` + shulker in `container_size`), sculk save fields (see above), falling-block BE restore (see test_falling_chest), container titles/names (VanillaContainerTitleOracle passes), enchanting offers/click (`menu.rs:1161` `enchant_changed`, `stations.rs:108` `is_enchantable`, bookshelf power offsets, `enchanting_book_opens_with_offers_and_turns_for_a_new_item` test; the click matches vanilla per the G12 withdrawal), **sculk growth inhibition** (G37 fixed — flips from gap in 26.1.2 to pass here).

### Method-2 gaps (20)

Decorated-pot use (G15), merchant pricing (G23 + G24), sign NBT (G27), furnace tick (G01 + G04 + G05; G03/G06 fixed), spawner tick (G20; G11 fixed), cartography (G16), dispenser/dropper (G18 + G35; G07 never existed), hopper (G08 — entity-catch bowl only), ender chest (G14), save round-trip BE (G27 + G28 + G36), BE coverage (G13 + G15), crafter (G09 + G10 + G29), container loot unpacking (G26), beacon (G31), brewing recipes (G02), brewing tick (G02 + G36), trial spawner tick (G34), menu engine (G21; G12 withdrawn), composter (G30), sign interaction (G32 + G33).

## Suspicions checked and refuted

- **"The spawner gate is broken because the port uses a name vanilla doesn't define"** — refuted. Vanilla 1.21.10 HAS the rule (`spawnerBlocksEnabled`, `RULE_SPAWNER_BLOCKS_ENABLED`, checked in both `BaseSpawner.serverTick` and `TrialSpawner.canSpawnInLevel`), and the port maps it explicitly (`gamerules.rs` `vanilla_key`/`from_vanilla_key`, round-tripped through `apply_compound`/`rules_compound`). G11 is FIXED. (Caveat, not an item: whether the `/gamerule` command surface accepts the vanilla key is Batch 9's territory and was not verified here.)
- **"The wax sound plays twice"** (G19) — refuted again for 1.21.10. 3002..=3005 route to `block_particle_event` (particles only) before the catch-all; the catch-all sound table (`game.rs:4471-4505`) has no 3003 entry; the wax sound plays once via the explicit `wax_on_sound` / dispenser play.
- **"The sculk water gate admits flowing water"** — refuted (as in 26.1.2). Vanilla `SculkBlock.canPlaceGrowth` requires air OR (water block AND `fluidState.is(Fluids.WATER)`); `TypedInstance.is` is a type-identity check, so flowing water (`FLOWING_WATER`) fails it — the gate is source-only, and the port matches.
- **"Merchant float parsing (G25) breaks the merchant drivers"** — refuted for 1.21.10. Both merchant corpora use explicit NBT offers; vanilla 1.21.10 has no trade-set data at all; the port's live path is hardcoded tables. G25 is WITHDRAWN and both drivers pass.
- **"The 1.21.10 oracle sources are the repaired 26.1.2 forms"** — refuted. They are the unrepaired 26.3 forms (gamerules/gizmos/EntityTypes/ContainerInput imports, 10-arg ctors), and 1.21.10's API is further from 26.3 than 26.1.2's — nine oracles regress from pass to fail.
- **"The first-attempt Ticker ServerLevel-ctor error was an artifact of the broken measurement"** — refuted. The valid measurement confirms it: 1 error, `:70`, 10 args passed vs 11 required.

## Checks by driver class

**Oracle-passing drivers (8 pass):** test_block_entity_storage, test_paletted_container, test_chest_load, test_chest_pair, test_container_title, test_enchantment_components, test_storage_menu, test_recipe_book_view.

**Oracle-blocked drivers (29 gap):** test_block_entity_tickers (O01), test_enchanting (O07), test_simple_container (O11), test_item_stack_storage (O12), test_container_screen (O17), test_stored_enchantment_patch (O20), test_recipe (O24), test_trade_components (O27), test_container_component_names (VanillaCommandExecOracle — Batch 9), test_brewing_fuel (O05), test_dispenser (O06), test_menu_click (O08), test_recipe_book (O09), test_sculk (O14), test_block_entity_anim (O15), test_block_entity_render (O16), test_recipe_book_render (O18), test_place_recipe (O10), test_trading (O25), test_ghost_recipe (O26), test_crafter (O06), test_furnace (O03), test_brewing (O04), test_enchant_durability (O19), test_vehicle_container (O21), test_shulker_space (O22), test_spawner (O23), test_trial_spawner (O23), test_vault (O23).

**No-oracle drivers (15 pass, 5 gap):** pass — test_crafter_drop_name, test_falling_chest, test_jukebox_saved_clock, test_sculk_saved_fields, test_enchantment_map_validation, test_world_pack_enchantment_loader, test_anvil_turtle_egg, test_crafting_priority, test_craft_map_take, test_end_gateway, test_sign_literal, test_machine_recipe_placement, test_recipe_book_variants, test_merchant_gate, test_merchant_crafted. Gap — test_furnace_loaded_total (G01/G04), test_furnace_output_components (G01), test_loot_mine_pot_components (G28/G26), test_pot_sherd_templates (G28), test_pot_hopper (G15).

**Method-2 (18 pass, 20 gap):** see "What passed" and the method-2 gap list above.

**Not counted:** test_composter_parity.py (references VanillaDispenserOracle; Batch 5's driver per the 26.1.2 precedent — the behaviour is B4-G30, not double-counted here).

## Notes for the coordinator

- **26.1.2 attribution correction.** The 26.1.2 Batch 4 report and slice attribute test_crafter and test_vault to O03 (VanillaFurnaceOracle). The drivers compile different oracles: test_crafter compiles VanillaDispenserOracle (`test_crafter_parity.py:242`), test_vault compiles VanillaNaturalSpawnOracle in vault mode (`test_vault_parity.py:175,193-195`) — in both the 26.1.2 and the 1.21.10 trees. This report attributes them to O06 and O23 respectively. Gap counts are unaffected (both drivers are blocked either way); the 26.1.2 slice's O03 evidence line ("Blocks test_crafter, test_furnace, test_vault") should read "Blocks test_furnace" only.
- **26.3-only content in the 1.21.10 tree (flagged, not minted):** a `potent_sulfur` block-entity arm in `containers.rs` `for_block` (sulfur is 26.3-only); a `sulfur_spike` land sound in the level-event table (`game.rs:1052`); `sulfur_cube_content` handling in the protocol skip table (`item.rs:1053-1055` — correctly read-and-dropped as a component 1.21.10 predates). The sulfur skip is correct handling; the BE arm and sound are dead 26.3 leakage worth a look.
- **A wrong port comment:** `spawner.rs:446` claims 1.21.10 has `setEntityData`. It does not (jar-verified; `Spawner` has only `setEntityId`). Folded into G20/G34 evidence.
- **Cross-batch oracle ownership:** VanillaCommandExecOracle (Batch 9) and VanillaDragonContactsOracle (Batch 3) fail in 1.21.10; VanillaBlockUpdateOracle / VanillaLevelShapeOracle / VanillaMobAiOracle (Batch 2/3) fail in 1.21.10. B4's driver table depends on CommandExec (test_container_component_names) and BlockUpdate (O03, O19 cascades). No 1.21.10 Batch 0 oracle table exists yet — when one lands, it should supersede this batch's family-scoped measurement for the shared oracles (the per-file counts here are the both-cp counts from `b04/scratch/oracle-measure-1.21.10.json`).
- **Dedup:** B4-O16's UvMapping facet dedups to B10-O03 (keep B10 canonical); B4-O18 dedups to B10-O05 (one of five renderpearl oracles). B4-O27 (VanillaTradeComponentsOracle) overlaps Batch 3's trading territory — cross-reference, do not double-mint.
- **G17 stays UNVERIFIED** (smoker/blast halving masked by G01; no slice item), as in 26.1.2.
- **No prepared 1.21.10 `.local`** exists; all data counts are jar-direct. If a prepared `.local` lands later, the brewing-recipe count (0 data-driven recipes) and the tag/component absences should be re-confirmed against it — though the jar is authoritative for those.

## Verdict

1.21.10 Batch 4: **95 checks, 41 pass, 54 gap** as-tallied (58 behaviour: 33 pass, 25 gap; 37 non-behaviour: 8 pass, 29 gap). **63 items**: 28 behaviour gaps (4 major), 25 oracle defects (5 major), 1 fixed, 9 closed.

Headline: the four 1.21.7 majors are all still live (furnaces never light, brewing never brews, cartography dead, composter dead) because the 1.21.7 tree's fixes were not carried into the 1.21.10 tree — but the 1.21.10 tree also carries real adaptations the 26.1.2 tree lacks (furnace burn/timer, sculk inhibition, gamerule name mapping, deferred merchant gossip, merchant crafted stats), so five behaviour items are fixed and both merchant drivers pass. The oracle side regressed hard: nine oracles that passed in 26.1.2 fail in 1.21.10, because the oracle sources are unrepaired 26.3 forms and 1.21.10's API surface is the furthest yet from 26.3.
