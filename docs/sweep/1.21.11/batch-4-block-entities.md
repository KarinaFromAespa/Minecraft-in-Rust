# Batch 4 — Block entities / containers — 1.21.11 parity sweep

Static-only batch. Checks + triage only; no fixes, no driver runs, no git in the worktree.

## §0 Execution status

- **Jars**: named 1.21.11 jars at `~/workspace/mcir-work/e6-deltas/versions/1.21.11/jars/`, sha1-verified 2026-11-25: `client-named.jar` 29bddf144a2f2907528b25d616770bfea67c4792, `server-game-named.jar` 7f0d85c12aeb57f14687b7ab2ae2e5d825fbf0d2. `server.jar` is a bundler jar and was never javap'd. Java 21, DataVersion 4671, protocol 774.
- **Oracle measurement**: family-scoped, own measurement — no 1.21.11 Batch 0 oracle table has landed (`docs/sweep/1.21.11/` holds only inventory files). 45 files (31 B4 oracles + helpers + second-wave oracles behind the chest/trade/component-names drivers) compiled fixpoint-style (drop failing files, recompile the clean set) against three classpaths (server-game jar + 39 server libs extracted from the bundler's `META-INF/libraries/`, client jar + 107 launcher libs downloaded from the 1.21.11 `version.json` artifact URLs — 0 failed, versions differ from 1.21.10's, so 1.21.10 libs were not reused — both), javac `--release 25`, `-Xmaxerrs 100000`. jsr305 is absent from the 1.21.11 tree; the 1.21.10 tree's `tools/jsr305-3.0.2.jar` was used. Results in `b04/scratch/oracle-measure-1.21.11.json`, logs in `b04/scratch/oracle-logs/big-*.log`, script `b04/scratch/measure-oracles-b4-1.21.11.py`.
- **Data**: no prepared 1.21.11 `.local` exists, so data counts are jar-direct: 7,557 data files extracted from `server-game-named.jar` (`b04/evidence/jardata/`).
- **Diff controls**: Rust source diff 1.21.11 vs 1.21.10 and vs 1.21.7 (read-only) for every B4 file; javap `-p -c` on the named 1.21.11 server-game jar (client jar for render/client-only classes). Evidence in `b04/evidence/javap/` (61 dumps) and `b04/evidence/jardata/` (7,557 extracted data files).
- **Drivers**: no driver was run live. The 9 oracle-passing drivers are declared pass per the 26.2 precedent. The 20 no-oracle drivers were read for what they assert (including the merchant corpora, `merchant_gate_corpus.py` / `merchant_crafted_corpus.py`, and the crafter-drop driver source) and verdicted on the code path each exercises.

## Tally (as-tallied, per the 26.2 precedent)

| | total | pass | gap |
|---|---|---|---|
| **All checks (as-tallied)** | **96** | **42** | **54** |
| Behaviour checks | 59 | 33 | 26 |
| Non-behaviour (driver-status / oracle-blocked) checks | 37 | 9 | 28 |
| Driver-status checks (57 drivers) | 57 | 23 | 34 |
| Method-2 checks | 39 | 19 | 20 |

For comparison, 1.21.10 Batch 4: 95 checks, 41 pass, 54 gap (58 behaviour: 33 pass, 25 gap). 1.21.11 adds one check (a new furnace-XP method-2 check, pass) and flips one driver to pass (test_container_component_names, on VanillaCommandExecOracle now passing) — but loses one behaviour pass (test_crafter_drop_name, on the corrected G26 trace: the 1.21.10 pass verdict was wrong — see "Notes for the coordinator"). Net: +1 check, +1 pass, same gap count. Behaviour is one gap worse than 1.21.10 (26 vs 25) purely on that correction; oracles are one driver better (9 oracle-passing vs 8).

Items: **63** in the slice — 28 behaviour gap items (5 major, 23 minor), 25 oracle-defect items (5 major, 20 minor), 7 fixed, 3 closed. No new items minted; G26 is upgraded minor → major on fresh blast-radius evidence (see below).

## Method

Static-only, mirroring the 1.21.10 Batch 4 template:

1. **Oracle status** from the own family-scoped measurement above (45 files, 3 classpaths). A driver whose oracle fails on the both-cp is oracle-blocked (gap, non-behaviour); a driver whose oracle passes is declared pass per the 26.2 precedent. Client-only oracles (VanillaRecipeBookViewOracle) are judged on client+both, not server.
2. **Behaviour** from javap `-p -c` on the named 1.21.11 server-game jar (client jar for render/client-only classes) + Rust source comparison against the 1.21.11 tree + jar-direct data counts. Every re-check is decided against 1.21.11's OWN jar/data, never the 26.3 base's form (VANILLA-CHECK rule).
3. **No-oracle drivers** read for what they assert, verdicted on the code path each exercises (including corpus reads for the merchant fixtures, the crafter-drop driver's assertions, and the port's own unit tests where they assert the same semantics).
4. **Method-2 checks** (39 = the template's 38 + one new furnace-XP check for the 1.21.11 delta): source-level verification of each B4 area not covered by a driver, fresh against 1.21.11 code/jar — not carried from 1.21.10 where the files differ.

## Systemic finding: the template is the same mixed inheritance as its child

1.21.11 is the E6 era template and 1.21.10 its first child, and the template carries the same mixed inheritance the 1.21.10 sweep found — file by file, the same files are 26.3-form and the same files are genuinely adapted:

- **The 26.3-form component/tag-read pattern survives wholesale** in `providers.rs`, `brewing.rs`, `composter.rs`, `menu.rs`, `block_entity_nbt.rs`, `block_entity_item.rs`, `sign_edit.rs`, `dispenser.rs`, `spawner.rs`, `items.rs` (loot engine). Those files read components/tags that do not exist in 1.21.11 (`cooking_fuel`, `brewing_fuel`, `compostable`, `sign_text_front`/`sign_text_back`, `waxed`, `#extendable_maps`) and get nothing. Consequence: the four 1.21.7 majors that the 1.21.7 tree had fixed are still present — G01 furnaces never light, G02 brewing stands never brew, G16 cartography dead, G30 composter dead — and a fifth major joins them: G26, whose blast radius this run measured for the first time (856 of 1,083 block tables drop nothing at all). The 1.21.7 tree's fixes (the `FUEL_VALUES` table in `items.rs`, the loot engine reading `"functions"`/`"condition"`) were **not carried** into the template.
- **But genuine 1.21.11 adaptations exist where the 1.21.10 tree has none or fewer.** The furnace burns by 1 and resets its timer on a recipe-less input (G03, G06 fixed, with 1.21.11-specific comments in `furnace.rs`); sculk growth inhibition counts sensors/shriekers directly with the comment "no growth-inhibitors tag in 1.21.11" (G37 fixed); `gamerules.rs` is rewritten for the data-driven `game_rule` registry — snake_case `minecraft:<rule>` keys, typed values, `Data.game_rules` with a `{data, DataVersion}` codec — and its table is COMPLETE: 58 RULES entries + `MAX_MINECART_SPEED` = 59, exactly matching vanilla's 59 `GameRule` fields, zero differences either way (G11 fixed with no name map needed, because vanilla 1.21.11 rule names ARE the engine names); the merchant path defers gossip delivery to the villager AI phase (`villager_trade_pending` → `MobEvent::VillagerTrade` → `villager_traded_by`, +2 per trade, matching the corpus exactly) and drains crafted counts into stats (`menu_clicks.rs`, `lan_trading.rs`, `trading.rs:607`); the merchant live path is hardcoded `villager_trades` tables because 1.21.11 has no trade-set data; `containers.rs` `create_experience` now draws the furnace XP fraction from the level random, matching vanilla 1.21.11's `createExperience` (a new method-2 pass); the smithing template list grows 9 → 11 (`nautilus_armor`, `spear`), matching the 1.21.11 jar data; the loom carries 1.21.11 adaptations (thumb-only hover, dropped scrolling reset on click).
- **The oracle picture improved in places vs 1.21.10, for a version-shape reason.** VanillaCommandExecOracle (45 errors in 1.21.10) and VanillaLevelShapeOracle now PASS on all three classpaths; VanillaFurnaceOracle and VanillaRecipeOracle lost their own `unwrapKey()` errors to the oracle sources' `id()` adaptation (Furnace is now a pure cascade); error counts fell sharply for Dispenser 42 → 19, Sculk 51 → 18, Trading 86 → 23, BlockUpdate 63 → 38, MobAi 103 → 38 — because the `gamerules` and `gizmos` packages now EXIST in 1.21.11 (10 and 21 entries respectively). But a NEW wrong adaptation appeared: the 1.21.11 oracle sources renamed `ChunkPos.asLong` → `ChunkPos.pack`, and the jar has no `pack` at all (only `asLong(int,int)` / `asLong(BlockPos)`) — O01 and O23 both fail on it. The remaining failures are the same 26.3-form causes: `EntityTypes`, `BrewingInput`, `BrewingFuel`, `EntitySpawnRequest`, `ContainerInput`, `TradeCost` all absent; `RegistryLayer` has no `WORLD`; `Reference.bindComponents` and `ItemResult.createItemStack` do not exist; `ServerLevel`'s ctor takes 11 args (incl. `RandomSequences`); `DedicatedServer`'s ctor takes 7; `Recipe.assemble` takes `(T, HolderLookup$Provider)`; `ContextMap` has no `builder`. The oracle sources differ from 1.21.10's in 168 files, mostly the `Identifier` rename (1.21.11 HAS `Identifier`; `ResourceLocation` does not exist) and `holder.id()`.

## Behaviour defects (gap items)

### B4-G01 (major, 1.21.10 REPRODUCES — 1.21.7's fix NOT carried) — furnaces never light

Vanilla 1.21.11: `AbstractFurnaceBlockEntity.getBurnDuration` → `FuelValues.burnDuration`; `FuelValues.vanillaBurnTimes` has **64** `Builder.add` calls (1.21.10: 63). `DataComponents` has no `cooking_fuel` (0 hits in the jar). Port: `providers.rs:108-117` `burn_time` reads `i.fuel` (the `cooking_fuel` component), missing → 0; `items.rs:1041` `fuel` comes only from that component. The 1.21.7 tree HAS the `FUEL_VALUES` table (`items.rs:~251`, 62 entries); the 1.21.11 `items.rs` has no `FuelValues`/`BURN_TIMES` table anywhere. Every furnace, blast furnace and smoker burns nothing.

### B4-G02 (major, 1.21.10 REPRODUCES) — brewing stands never brew

Vanilla 1.21.11: brewing is `PotionBrewing` (`isIngredient`/`hasMix`, now at `world/item/alchemy/PotionBrewing`) + the `#brewing_fuel` item tag (exists, blaze powder only); `BrewingStandBlockEntity` uses `FUEL_USES=20` (bipush 20) and a 400-tick brew (sipush 400). The jar has **0** files under `data/minecraft/recipe/brewing/` and `DataComponents` has no `brewing_fuel`. Port (`brewing.rs`, byte-identical to 1.21.10): `load` reads data-driven brewing recipes (`:260-291`) — of which 0 exist — and `brewing_fuel` reads the absent component (`:423`). Nothing ever brews.

### B4-G04 (minor, 1.21.10 REPRODUCES) — furnace completes on `>=`

Vanilla 1.21.11 `AbstractFurnaceBlockEntity.serverTick`: completion only on `==` (bytecode 292 `if_icmpne`). Port `furnace.rs:129`: `>=`. A loaded furnace with `cookingTotalTime=0` completes instantly in the port; vanilla never completes it. (Masked in practice by G01 — the furnace never lights — but the comparison itself is wrong.)

### B4-G05 (minor, 1.21.10 REPRODUCES) — fuel remainder dropped when the fuel stack remains

Vanilla 1.21.11 furnace tick (bytecode 219-246): the fuel stack shrinks; the remainder goes into the slot **only if the stack empties**, else it is lost. Port `furnace.rs:120-121`: drops the remainder when the stack remains. (Datapack-only in practice — no vanilla fuel has a remainder in 1.21.11 — but the behaviour is inverted.)

### B4-G08 (minor, 1.21.10 REPRODUCES) — hopper suction bowl is 0.125..0.875

Vanilla 1.21.11 `SUCK_AABB` = `Block.column(16.0, 11.0, 32.0)` → (0, 11/16, 0)-(1, 2, 1) — now on the `Hopper` interface (moved from `HopperBlockEntity`). Port `machines.rs:972-977`: the entity-catch bowl is 0.125..0.875. (The tick-path `hopper_suck` at `:1139` uses the full SUCK_AABB — the gap is only the entity-catch bowl, so pickup above the hopper misses items vanilla catches.)

### B4-G09 (minor, 1.21.10 REPRODUCES) — crafter eject reaches blocks only

Vanilla 1.21.11 `CrafterBlock.dispenseItem` uses `HopperBlockEntity.getContainerAt` (block + entity containers). Port `crafter.rs:149-152` (byte-identical to 1.21.10): eject uses block-only `Containers::view`. A crafter facing a chest minecart, chest boat or hopper minecart ejects nothing.

### B4-G10 (minor, 1.21.10 REPRODUCES) — crafter trigger checks only the local player

Vanilla 1.21.11 crafter trigger: `getEntitiesOfClass(ServerPlayer, AABB.ofSize 17,17,17)`, triggers `CRAFTER_RECIPE_CRAFTED` per player in the box. Port `crafter.rs:188` (byte-identical to 1.21.10): checks `self.player` only. LAN guests standing at the crafter never get the advancement trigger.

### B4-G13 (minor, 1.21.10 REPRODUCES) — no bed block entity

Vanilla 1.21.11 has `BedBlockEntity` (colour field) and `BlockEntityType.BED`. Port: `containers.rs` `for_block` has no bed arm; `container_size` (`:320`) covers only chest/barrel/ender_chest/hopper/dispenser/dropper/shulker/copper_chest; `block_entity_nbt.rs` `TYPES` (47 entries) has no `"minecraft:bed"`. Renamed beds lose their name; chunk saves carry no bed data.

### B4-G14 (minor, 1.21.10 REPRODUCES) — ender chest opens on LAN guests

Port `storage.rs:157` `open_ender_chest` swaps the **host's** `ender_items` into the shared block entity. A LAN guest opening an ender chest sees (and edits) the host's ender inventory instead of their own.

### B4-G15 (minor, 1.21.10 REPRODUCES) — no decorated-pot block entity

Vanilla 1.21.11 has `DecoratedPotBlockEntity` (implements `RandomizableContainer` + `ContainerSingleItem$BlockContainerSingleItem`). Port: `containers.rs` `for_block` has no decorated-pot arm. Hoppers/droppers cannot see decorated pots; no container view exists.

### B4-G16 (major, 1.21.10 REPRODUCES) — cartography table gated on a nonexistent tag

Vanilla 1.21.11 `CartographyTableMenu`: 0 `ItemTags` references; `setupResultSlot` checks `Items.PAPER` directly. The jar has no `#extendable_maps` tag. Port `menu.rs:1290`: the map-extend path is gated on the `extendable_maps` tag. Paper zoom-out never works.

### B4-G18 (minor, 1.21.10 REPRODUCES) — dispenser mud uses the wrong tag spelling

Vanilla 1.21.11 `DispenseItemBehavior$15` (the mud behaviour) uses `BlockTags.CONVERTABLE_TO_MUD`; the jar tag is `convertable_to_mud` (`convertible_to_mud` does not exist). Port `dispenser.rs:2251`: checks `"minecraft:convertible_to_mud"`. A dispenser with a water bottle never turns dirt into mud.

### B4-G20 (minor, 1.21.10 REPRODUCES) — spawner `setEntityData` merge

Vanilla 1.21.11 `Spawner` has **only** `setEntityId` (+ static helpers); `SpawnerBlockEntity` has no `setEntityData`/`overrideEntityData`. Port `spawner.rs:464` `set_entity_data` (and `mobs/spawner.rs:278,429`) merges egg entity data into the spawner. The port comment at `spawner.rs:446` claims 1.21.11 has `setEntityData` — wrong per the jar. A spawn egg used on a spawner merges NBT vanilla discards.

### B4-G21 (minor, 1.21.10 REPRODUCES) — merchant result slot blocks pickup

Vanilla 1.21.11 `MerchantResultSlot` has **no** `mayPickup` override (mayPlace, remove, onQuickCraft, checkTakeAchievements, onTake only). Port `menu.rs:1505` `may_pickup`: the `MerchantResult` arm (`:1512-1521`) gates pickup on `o.satisfied_by(...)`. Takes vanilla allows (e.g. with the payment already consumed by a prior take in the same click sequence) are blocked.

### B4-G22 (minor, 1.21.10 REPRODUCES) — exploration-map check omits the vanilla exclusion

Vanilla 1.21.11 `MapExtendingRecipe.matches`: `isExplorationMap` check (bytecode 41) first, then scale < 4. Port `crafting.rs:900-917`: ring + scale < 4 only, no exploration-map check. An exploration map can be zoomed out in the port; vanilla refuses.

### B4-G23 (minor, 1.21.10 REPRODUCES) — hero discount computed in f32

Vanilla 1.21.11 `Villager.updateSpecialPrices` hero part: `0.3d + 0.0625d * amplifier` (double, `ldc2_w`) with `Math.floor(double)`. Port `merchant.rs:116`: `f32` widened to f64. At high amplifier the f32 rounding changes the floored discount by 1 emerald. (The reputation part of the same function matches vanilla exactly — f32 multiply + floor, verified against the bytecode.)

### B4-G24 (minor, 1.21.10 REPRODUCES) — villager levels up immediately

Vanilla 1.21.11 `Villager.rewardTradeXp`: adds XP immediately, but the level-up is deferred — `updateMerchantTimer=40`, `increaseProfessionLevelOnUpdate=true` (only when `shouldIncreaseLevel`). Port `merchant.rs:405`: `level += 1` immediately on trade. (The gossip delivery IS correctly deferred — see merchant_gate in "What passed" — but the level-up is not.)

### B4-G26 (major, 1.21.10 REPRODUCES — UPGRADED from minor — 1.21.7's fix NOT carried) — loot engine reads 26.3 key names; most blocks drop nothing

Vanilla 1.21.11 loot tables: 1,324 files; `"functions"` in 361 files (983 occurrences); `"modifier"` in 0 files; condition objects use the string `"condition"` key (1,542 occurrences). Sample `blocks/diamond_ore.json` confirms the format (`"condition": "minecraft:match_tool"`, `"functions": [{"function": "minecraft:apply_bonus"...}]`). Port `items.rs` (the engine serving block drops via `game.rs:3683` `block_loot_at` → `block_drops_by_player` → `LootContext::evaluate`): modifiers read from `["modifier"]` only (`:1870-1871`, `:2143`, `:2284`); `condition()` reads `o.get("type")` only (`:1988`); `modifiers()` reads `f["type"].or(f["function"])` (`:2308`) but is never handed the `"functions"` list. The 1.21.7 tree FIXED this (reads `"functions"`, condition reads `"type"` OR `"condition"` at `:2126-2127`, with the regression test `conditions_read_the_1_21_7_condition_key_as_well_as_type`); the 1.21.11 tree did NOT carry the fix.

The blast radius, measured fresh this run, is far larger than the 1.21.10 report's "no gated loot drop can work":

- **856 of 1,083 block tables (79%) have ALL pools conditioned** — and every condition object evaluates false in the port (kind comes from `"type"`, absent in 1.21.11 data, so `condition()` falls through to `_ => false`). Those blocks drop **nothing at all** when broken. Verified individually: crafter (sole pool conditioned on `survives_explosion`), chest, oak log. The second engine (`horizon-world/src/loot.rs`, serving entity/fishing/trial-spawner/vault loot) has the same defect in mirror form: it reads singular `pool["condition"]` / `entry["condition"]` (`:2028`, `:2080`) and `pool["modifier"]` / `table["modifier"]` (`:2039-2040`), so pool/entry conditions are never evaluated and functions never applied — that loot is ungated instead of absent (e.g. diamond ore would always drop the ore via the `alternatives` first child).
- **Entry-level gating is broken too**: diamond ore's pool is unconditioned, but both `alternatives` children carry `conditions` lists, so `expand()` rejects both and diamond ore drops nothing.
- **Functions never apply**: shulker boxes drop without their contents/name (`copy_components` never runs), Fortune bonuses never run, `set_count` never runs.
- **The port's own loot tests cannot see it**: `items.rs:3248` `loot_table_entries_roll_their_table` uses 26.3-form fixtures (`"modifier":[{"type":"minecraft:set_count",...}]`); no test exercises a 1.21.11-form table.
- **No load-time rescue exists**: `shared_tables` loads the tables verbatim from JSON; horizon-prepare has no loot-table rewriting; the `native_loot_table_load` hook is a no-op mod point.

This corrects the 1.21.10 report's test_crafter_drop_name pass verdict (see "Notes for the coordinator"): the crafter's sole pool is skipped, so the driver fails on all four of its cases (named crafter, plain crafter, chest, shulker). Blocks test_crafter_drop_name (NEW) and test_loot_mine_pot_components (with G28), and the container-loot-unpacking method-2 check.

### B4-G27 (minor, 1.21.10 REPRODUCES) — furnace int times + sign `allow_op_features` in saves

Vanilla 1.21.11 `AbstractFurnaceBlockEntity.saveAdditional`: 4× `putShort`, no `speed_multiplier`. Vanilla `SignBlockEntity.saveAdditional`: `front_text`, `back_text`, `is_waxed` only — no `allow_op_features` field exists on the class. Port `block_entity_nbt.rs`: the furnace arm writes int times + `speed_multiplier` (`:383`); the sign arm keeps `allow_op_features` (`:458-459`). Port-written saves carry fields vanilla never writes (and vanilla-written saves lose nothing the port needs — the port's fields are invented).

### B4-G28 (minor, 1.21.10 REPRODUCES) — pot sherds saved in compound form

Vanilla 1.21.11 `PotDecorations.CODEC` = `byNameCodec().sizeLimitedListOf(4).xmap(...)` — **list** form. Port `block_entity_nbt.rs:663-672`: the pot sherds arm writes the compound form. A port-written decorated pot is not one vanilla 1.21.11 reads (and vice versa).

### B4-G29 (minor, 1.21.10 REPRODUCES) — crafter trigger reads the `recipes` key

Vanilla 1.21.11 `RecipeCraftedTrigger$TriggerInstance` (package `net.minecraft.advancements.criterion` — Mojang's misspelling FIXED in 1.21.11) has the singular `recipeId` field (`ResourceKey<Recipe<?>>`); the advancement `adventure/crafters_crafting_crafters.json` uses `"recipe_id": "minecraft:crafter"`. Port `crafter.rs:206`: `recipe_matches(&c["recipes"])`. The crafter trigger never matches a vanilla advancement criterion.

### B4-G30 (major, 1.21.10 REPRODUCES) — composter never composts

Vanilla 1.21.11: `ComposterBlock.COMPOSTABLES` is a hardcoded `Object2FloatMap`. `DataComponents` has no `compostable`. Port (`composter.rs`, byte-identical to 1.21.10): reads the absent `compostable` component (`:24-25`). Nothing ever composts. (The driver `test_composter_parity.py` belongs to Batch 5's tally per the 26.1.2 precedent — it references VanillaDispenserOracle and is not counted in B4's 57; the behaviour is covered here.)

### B4-G31 (minor, 1.21.10 REPRODUCES) — beacon validates effects and kicks LAN guests

Vanilla 1.21.11: `BeaconBlockEntity` has no `validateEffects`/`getRequiredLevelsFor` (0 hits); `BeaconMenu.updateEffects` checks payment → encodes both → sets data → removes payment, with no validation. Port `menu.rs`: `beacon_required_levels` (`:306`) and `beacon_update` validation (`:1203-1215`, returns false when invalid); `lan_stations.rs:768` kicks with "Player {} tried to set invalid beacon effects". A LAN guest setting effects the port deems invalid is kicked; vanilla accepts any selection with payment.

### B4-G32 (minor, 1.21.10 REPRODUCES) — sign click actions never run

Vanilla 1.21.11 `SignBlock.useWithoutItem`: `executeClickCommandsIfPresent` runs **first** (bytecode 80), before the waxed check (87) — unconditionally. Port `sign_edit.rs:166-213`: click actions are gated on `allow_op_features` (default false), "Click actions are disabled" (`:213`). Clickable signs never run their commands.

### B4-G33 (minor, 1.21.10 REPRODUCES, narrowed on LAN) — sign pick/place preserves text and wax via nonexistent components

Vanilla 1.21.11 `DataComponents` has no `sign_text_front`/`sign_text_back`/`waxed`. Port `block_entity_item.rs:192-193,231-232,251,294`: sign pick/place preserves text and wax via those components. Locally the text/wax survives a pick-and-place that vanilla drops. Narrowed on LAN exactly as in 1.21.10: the protocol layer drops them at wire encode — `horizon-protocol/src/item.rs:1246-1247` `write_patch` filters by `ids::component(kind) >= 0`, and the three components have no 1.21.11 registry id — so LAN guests receive a plain sign item.

### B4-G34 (minor, 1.21.10 REPRODUCES) — trial spawner op egg merges entity data

Vanilla 1.21.11 `TrialSpawnerBlockEntity` has only `setEntityId`. Port `spawner.rs:450` `override_entity_data` (and `mobs/trial_spawner.rs:405,629`) merges egg entity data into the trial spawner. Same defect as G20 on the trial-spawner path.

### B4-G35 (minor, 1.21.10 REPRODUCES) — dispensed spawn egg on a null spawn is returned unshrunk

Vanilla 1.21.11 `DispenseItemBehavior$1` (spawn egg): `getType` null → return stack unchanged; else `EntityType.spawn` (result popped) → `shrink(1)` → `gameEvent(ENTITY_PLACE)` → return stack. Note vanilla **always** shrinks and fires the event once a type exists, even if the spawn itself produces nothing. Port `dispenser.rs:1024-1026`: on a null spawn, returns unshrunk with a CLICK and no event. A dispensed egg that fails to spawn keeps its count and plays no placement event.

### B4-G36 (minor, 1.21.10 REPRODUCES) — brewing-stand save is the 26.3 form

Vanilla 1.21.11 `BrewingStandBlockEntity.saveAdditional`: `BrewTime` short + `Fuel` byte only. Port `block_entity_nbt.rs:509-516`: int `BrewTime` + `total_brew_time` + `speed_multiplier`. Port-written brewing stands carry invented fields.

## 1.21.10 re-check verdicts (PRIMARY, 63 items)

Brief discrepancy, flagged for the coordinator: the brief says "60 items" but lists 59 by count, and omits four items present in the 1.21.10 slice — B4-O08, B4-O16, B4-O18 and B4-G37. All 63 slice items are decided here; the four omitted ones are marked (omitted from brief list).

| Item | Verdict | Note |
|---|---|---|
| B4-G01 | REPRODUCES | major; 1.21.7's FuelValues fix not carried; 64 `Builder.add` calls in 1.21.11 |
| B4-G02 | REPRODUCES | major |
| B4-G03 | FIXED | port grows output by 1 (furnace.rs:~133-138), matching vanilla 1.21.11 burn() (bytecode 84-87) |
| B4-G04 | REPRODUCES | minor |
| B4-G05 | REPRODUCES | minor |
| B4-G06 | FIXED | port resets timer on no recipe (furnace.rs:~169-174), matching vanilla 1.21.11 serverTick (bytecode 341-343) |
| B4-G07 | NEVER EXISTED | vanilla 1.21.11 getType returns null (no defaultType); dispense returns the stack unchanged — the port matches; the 1.21.7 premise "vanilla spawns the default mob" is false for 1.21.11 |
| B4-G08 | REPRODUCES | minor; SUCK_AABB now on the Hopper interface |
| B4-G09 | REPRODUCES | minor |
| B4-G10 | REPRODUCES | minor |
| B4-G11 | FIXED | vanilla 1.21.11 HAS the rule as `spawner_blocks_work` (`GameRules.SPAWNER_BLOCKS_WORK`, checked via `ServerLevel.isSpawnerBlockEnabled()` and in `TrialSpawner.canSpawnInLevel`); the port gates on the same name (game/spawner.rs:121,438, mobs/spawner.rs:472, mobs/trial_spawner.rs:664) — no name map needed in 1.21.11 |
| B4-G12 | WITHDRAWN | vanilla 1.21.11 bypasses the lapis check in creative (hasInfiniteMaterials at bytecode 74-78); the port matches (menu.rs:1146) — the 1.21.7 premise was the 1.21.7 form, not 1.21.11's |
| B4-G13 | REPRODUCES | minor |
| B4-G14 | REPRODUCES | minor |
| B4-G15 | REPRODUCES | minor |
| B4-G16 | REPRODUCES | major |
| B4-G18 | REPRODUCES | minor |
| B4-G19 | FIXED | 3002..=3005 route to particles only (game.rs:4367); 3003 appears only at :8342/:8361 paired with explicit `wax_on_sound` |
| B4-G20 | REPRODUCES | minor; port comment at spawner.rs:446 still wrongly claims setEntityData exists |
| B4-G21 | REPRODUCES | minor |
| B4-G22 | REPRODUCES | minor |
| B4-G23 | REPRODUCES | minor |
| B4-G24 | REPRODUCES | minor |
| B4-G25 | WITHDRAWN | vanilla 1.21.11 has NO trade-set/villager-trade data files (0 exist; trades are hardcoded in `VillagerTrades`); the int_provider zeroing survives only in the data-driven path (`merchant.rs:660` `offers_from_set`), unreachable with vanilla data — the live path is the hardcoded `villager_trades` tables. Dead code, noted in the report, no item. Both merchant drivers pass in 1.21.11 (see "What passed"). |
| B4-G26 | REPRODUCES | **UPGRADED to major** — 856/1,083 block tables drop nothing; test_crafter_drop_name now gaps (corrects the 1.21.10 pass verdict) |
| B4-G27 | REPRODUCES | minor |
| B4-G28 | REPRODUCES | minor |
| B4-G29 | REPRODUCES | minor; Mojang's `critereon` misspelling is FIXED in 1.21.11 (`criterion`) |
| B4-G30 | REPRODUCES | major |
| B4-G31 | REPRODUCES | minor |
| B4-G32 | REPRODUCES | minor |
| B4-G33 | REPRODUCES | minor; LAN-narrowing facet confirmed for 1.21.11 (write_patch drops the components) |
| B4-G34 | REPRODUCES | minor |
| B4-G35 | REPRODUCES | minor |
| B4-G36 | REPRODUCES | minor |
| B4-G37 | FIXED | (omitted from brief list) `sculk_can_place_growth` counts sensors/shriekers directly (false at matched >= 3), box (-4..4, 0..2, -4..4), source-only water gate — matches vanilla `SculkBlock.canPlaceGrowth` exactly; comment "no growth-inhibitors tag in 1.21.11" at sculk.rs:621 |
| B4-O01 | REPRODUCES | VanillaBlockEntityTickerOracle: ServerLevel ctor 10 args passed, 11 required (incl. RandomSequences), :70, 4 errors — plus the NEW bogus `ChunkPos.pack(...)` calls (:111/:119/:198); the jar has only `asLong` |
| B4-O02 | FIXED | VanillaBlockEntityStorageOracle passes in 1.21.11 (all three classpaths) |
| B4-O03 | REPRODUCES | VanillaFurnaceOracle: now a pure cascade on broken VanillaBlockUpdateOracle (round 2) — its own `unwrapKey()` error was fixed by the oracle's `id().identifier()` adaptation; no own errors |
| B4-O04 | REPRODUCES | VanillaBrewingOracle: `BrewingInput` import, :23, 14 errors — identical to 1.21.10 |
| B4-O05 | REPRODUCES | VanillaBrewingFuelOracle: `BrewingFuel` import, :13, 9 errors — identical to 1.21.10 |
| B4-O06 | REPRODUCES | VanillaDispenserOracle: first error differs (`EntityTypes`, :212, 19 errors, was 42) — the gamerules-package first error is GONE because `net.minecraft.world.level.gamerules` EXISTS in 1.21.11 |
| B4-O07 | REPRODUCES | VanillaEnchantingOracle: `RegistryLayer.WORLD` + `DATA_COMPONENT_INITIALIZERS` absent, 4 errors — identical to 1.21.10 |
| B4-O08 | REPRODUCES | (omitted from brief list) VanillaMenuOracle: first error DedicatedServer ctor (:65, 7 args in 1.21.11, 10 passed), 8 errors (was 10); ServerLevel ctor (:80), EntityTypes (:128-129), Prediction drop override (:138), ContainerInput (:377/:384) |
| B4-O09 | REPRODUCES | VanillaRecipeBookOracle: cascade on MenuOracle |
| B4-O10 | REPRODUCES | VanillaPlaceRecipeOracle: cascade on MenuOracle |
| B4-O11 | REPRODUCES | VanillaSimpleContainerOracle: `Reference.bindComponents` absent, 1 error, :25 — identical to 1.21.10 |
| B4-O12 | REPRODUCES | VanillaItemStackStorageOracle: `Reference.bindComponents` absent, 1 error, :20 — identical to 1.21.10 |
| B4-O13 | FIXED | VanillaPalettedContainerOracle passes in 1.21.11 (all three classpaths) |
| B4-O14 | REPRODUCES | VanillaSculkOracle: first error differs (world.clock package, :66, 18 errors, was 51) — the gizmos-package first error is GONE (gizmos EXISTS in 1.21.11, 21 entries); the datapackWorldRegistries cause is present (:327) |
| B4-O15 | REPRODUCES | VanillaBlockEntityAnimOracle: first error differs (no-override, :90, 7 errors, was 21); the fuelValues cause is present (:102) |
| B4-O16 | REPRODUCES | (omitted from brief list) VanillaBlockEntityRenderOracle: first error differs (renderer.state.level package, :21, 30 errors, was 42); the UvMapping cause is present (:23, :155); dedup to B10-O03 for the UvMapping facet |
| B4-O17 | REPRODUCES | VanillaContainerScreenOracle: `ContainerInput` absent (×5), 5 errors — identical to 1.21.10 |
| B4-O18 | REPRODUCES | (omitted from brief list) VanillaRecipeBookScreenOracle: renderpearl import, :4, 31 errors (was 35) — identical cause; dedup to B10-O05 |
| B4-O19 | REPRODUCES | VanillaEnchantDurabilityOracle: cascade on BlockUpdateOracle |
| B4-O20 | REPRODUCES | VanillaStoredEnchantmentPatchOracle: `ItemResult.createItemStack` absent, 1 error, :18 — identical to 1.21.10 (ItemResult is `ItemParser$ItemResult`, a record with item()/components() only) |
| B4-O21 | REPRODUCES | VanillaVehicleContainerOracle: `EntitySpawnRequest` import, :20, 5 errors — identical to 1.21.10 |
| B4-O22 | REPRODUCES | VanillaShulkerSpaceOracle: `EntityTypes` import, :10, 2 errors — identical to 1.21.10 |
| B4-O23 | REPRODUCES | VanillaNaturalSpawnOracle: first error differs (ServerLevel ctor, :141, 30 errors, was 22); also the bogus `pos.pack()` (:273), DedicatedServer/ServerChunkCache/ChunkMap ctors, cubemob package (:537), TagLoader/RegistryDataLoader (:612-619), EntityTypes (:754), spawnMobsForChunkGeneration signature (:826) |
| B4-O24 | REPRODUCES | VanillaRecipeOracle: assemble 2-arg ×4 (:55,:79,:97,:110) + `SmithingTransformRecipe.MAP_CODEC` absent (:85), 5 errors (was 10 — the unwrapKey errors are gone via the `id()` adaptation) |
| B4-O25 | REPRODUCES | VanillaTradingOracle: first error differs (world.clock package, :62, 23 errors, was 86); the datapackWorldRegistries cause is present (:254) |
| B4-O26 | REPRODUCES | VanillaGhostRecipeOracle: `ContextMap.builder()` absent, :56, 1 error — identical to 1.21.10 |
| B4-O27 | REPRODUCES | VanillaTradeComponentsOracle: TagLoader/RegistryLayer.WORLD/RegistryDataLoader/DATA_COMPONENT_INITIALIZERS (:35-39), SetEnchantmentsFunction.MAP_CODEC (:55), TradeCost (:57-58), 7 errors — identical to 1.21.10 |

PRIMARY tally: **53 REPRODUCES, 7 FIXED (G03, G06, G11, G19, G37, O02, O13), 1 NEVER EXISTED (G07), 2 WITHDRAWN (G12, G25).**

## 1.21.7 re-check verdicts (SECONDARY, 42 items)

| Item | 1.21.7 status | 1.21.11 verdict | Note |
|---|---|---|---|
| B4-G01 | gap | REPRODUCES | major; 1.21.7's FuelValues fix not carried |
| B4-G02 | gap | REPRODUCES | major |
| B4-G03 | closed | FIXED (still) | port grows output by 1, matching vanilla 1.21.11 burn() |
| B4-G04 | gap | REPRODUCES | |
| B4-G05 | gap | REPRODUCES | |
| B4-G06 | closed | FIXED (still) | port resets timer on no recipe, matching vanilla 1.21.11 |
| B4-G07 | closed | NEVER EXISTED (still) | |
| B4-G08 | gap | REPRODUCES | |
| B4-G09 | gap | REPRODUCES | |
| B4-G10 | gap | REPRODUCES | |
| B4-G11 | closed | FIXED (still) | vanilla 1.21.11 HAS the rule as `spawner_blocks_work`; the port gates on the same name — no name map needed (see PRIMARY table) |
| B4-G12 | closed | WITHDRAWN (still) | |
| B4-G13 | gap | REPRODUCES | |
| B4-G14 | gap | REPRODUCES | |
| B4-G15 | gap | REPRODUCES | |
| B4-G16 | gap | REPRODUCES | major |
| B4-O01 | closed | REPRODUCES | reopened (see PRIMARY table); now 4 errors incl. the bogus `ChunkPos.pack` calls |
| B4-O02 | closed | FIXED (still) | |
| B4-O03 | oracle-defect | REPRODUCES | now a pure cascade (own error fixed by the `id()` adaptation) |
| B4-O04 | oracle-defect | REPRODUCES | |
| B4-O05 | oracle-defect | REPRODUCES | |
| B4-O06 | oracle-defect | REPRODUCES | major; 19 errors (was 42 in 1.21.10) |
| B4-O07 | closed | REPRODUCES | reopened |
| B4-O08 | oracle-defect | REPRODUCES | major; 8 errors |
| B4-O09 | oracle-defect | REPRODUCES | |
| B4-O10 | oracle-defect | REPRODUCES | |
| B4-O11 | closed | REPRODUCES | reopened |
| B4-O12 | closed | REPRODUCES | reopened |
| B4-O13 | closed | FIXED (still) | |
| B4-O14 | oracle-defect | REPRODUCES | 18 errors (was 51 in 1.21.10) |
| B4-O15 | oracle-defect | REPRODUCES | 7 errors (was 21 in 1.21.10) |
| B4-O16 | oracle-defect | REPRODUCES | 30 errors (was 42 in 1.21.10) |
| B4-O17 | closed | REPRODUCES | reopened; major |
| B4-O18 | oracle-defect | REPRODUCES | 31 errors (was 35 in 1.21.10) |
| B4-O19 | oracle-defect | REPRODUCES | |
| B4-O20 | closed | REPRODUCES | reopened |
| B4-O21 | oracle-defect | REPRODUCES | |
| B4-O22 | oracle-defect | REPRODUCES | |
| B4-O23 | oracle-defect | REPRODUCES | major; 30 errors (was 22 in 1.21.10) |
| B4-O24 | closed | REPRODUCES | reopened; major; 5 errors (was 10 in 1.21.10) |
| B4-O25 | oracle-defect | REPRODUCES | 23 errors (was 86 in 1.21.10) |
| B4-O26 | oracle-defect | REPRODUCES | |

SECONDARY tally: **35 REPRODUCES, 5 FIXED (G03, G06, G11, O02, O13), 1 NEVER EXISTED (G07), 1 WITHDRAWN (G12).** Same tally shape as the 1.21.10 run.

## Oracle defects (25)

Error counts are the both-cp round-1 own errors from the own measurement (`b04/scratch/oracle-measure-1.21.11.json`).

| Item | Oracle | Errors | First error / cause |
|---|---|---|---|
| B4-O01 | VanillaBlockEntityTickerOracle | 4 | :70 ServerLevel ctor (10 passed, 11 required incl. RandomSequences) + :111/:119/:198 bogus `ChunkPos.pack(...)` — the jar has only `asLong`. Blocks test_block_entity_tickers. |
| B4-O03 | VanillaFurnaceOracle | cascade | Pure cascade on broken VanillaBlockUpdateOracle (round 2); no own errors. Blocks test_furnace. |
| B4-O04 | VanillaBrewingOracle | 14 | :23 `BrewingInput` import absent. Blocks test_brewing. |
| B4-O05 | VanillaBrewingFuelOracle | 9 | :13 `BrewingFuel` import absent. Blocks test_brewing_fuel. |
| B4-O06 | VanillaDispenserOracle | 19 | :212 `EntityTypes` — the gamerules-package first error is gone (the package EXISTS in 1.21.11). Blocks test_dispenser and test_crafter. **Major.** |
| B4-O07 | VanillaEnchantingOracle | 4 | :62 `RegistryLayer.WORLD` + :67 `DATA_COMPONENT_INITIALIZERS` absent. Blocks test_enchanting. |
| B4-O08 | VanillaMenuOracle | 8 | :65 DedicatedServer ctor (7 args in 1.21.11, 10 passed); :80 ServerLevel ctor; :128-129 EntityTypes; :138 Prediction drop override; :377/:384 ContainerInput. Blocks test_menu_click. **Major.** |
| B4-O09 | VanillaRecipeBookOracle | cascade | Depends on broken VanillaMenuOracle. Blocks test_recipe_book. |
| B4-O10 | VanillaPlaceRecipeOracle | cascade | Depends on broken VanillaMenuOracle. Blocks test_place_recipe. |
| B4-O11 | VanillaSimpleContainerOracle | 1 | :25 `Reference.bindComponents` absent. Blocks test_simple_container. |
| B4-O12 | VanillaItemStackStorageOracle | 1 | :20 `Reference.bindComponents` absent. Blocks test_item_stack_storage. |
| B4-O14 | VanillaSculkOracle | 18 | :66 `net.minecraft.world.clock` package absent — the gizmos-package first error is gone (gizmos EXISTS in 1.21.11); `datapackWorldRegistries` cause present at :327. Blocks test_sculk. |
| B4-O15 | VanillaBlockEntityAnimOracle | 7 | :90 no-override on both-cp; fuelValues cause present at :102 (`World is not abstract and does not override abstract method fuelValues()`). Blocks test_block_entity_anim. |
| B4-O16 | VanillaBlockEntityRenderOracle | 30 | :21 `net.minecraft.client.renderer.state.level` package absent; UvMapping cause present (:23 import, :155 cast; 0 UvMapping matches in client jar). Dedup to B10-O03 for the UvMapping facet. Blocks test_block_entity_render. |
| B4-O17 | VanillaContainerScreenOracle | 5 | `ContainerInput` absent ×5 (:55,62,67,72,77). Blocks test_container_screen. **Major.** |
| B4-O18 | VanillaRecipeBookScreenOracle | 31 | :4 renderpearl package absent (0 renderpearl entries in client jar). Dedup to B10-O05. Blocks test_recipe_book_render. |
| B4-O19 | VanillaEnchantDurabilityOracle | cascade | Depends on broken VanillaBlockUpdateOracle. Blocks test_enchant_durability. |
| B4-O20 | VanillaStoredEnchantmentPatchOracle | 1 | :18 `ItemResult.createItemStack` absent. Blocks test_stored_enchantment_patch. |
| B4-O21 | VanillaVehicleContainerOracle | 5 | :20 `EntitySpawnRequest` import absent. Blocks test_vehicle_container. |
| B4-O22 | VanillaShulkerSpaceOracle | 2 | :10 `EntityTypes` import absent. Blocks test_shulker_space. |
| B4-O23 | VanillaNaturalSpawnOracle | 30 | :141 ServerLevel ctor; :273 bogus `pos.pack()`; :337 DedicatedServer ctor; :357 ServerChunkCache ctor; :364 ChunkMap ctor; :537 cubemob package; :612-619 TagLoader/RegistryDataLoader/DataComponentInitializers; :754 EntityTypes; :826 spawnMobsForChunkGeneration signature; :865 getFilteredSpawningCategories signature. Blocks test_spawner, test_trial_spawner, test_vault. **Major.** |
| B4-O24 | VanillaRecipeOracle | 5 | :55 assemble 2-arg (×4: :55,:79,:97,:110) + :85 `SmithingTransformRecipe.MAP_CODEC` absent (0 matches in jar). Blocks test_recipe. **Major.** |
| B4-O25 | VanillaTradingOracle | 23 | :62 `world.clock` package absent; datapackWorldRegistries cause at :254; :376 EntityTypes. Blocks test_trading. Batch 3 overlap. |
| B4-O26 | VanillaGhostRecipeOracle | 1 | :56 `ContextMap.builder()` absent. Blocks test_ghost_recipe. |
| B4-O27 | VanillaTradeComponentsOracle | 7 | :35-39 TagLoader/RegistryLayer.WORLD/RegistryDataLoader/DATA_COMPONENT_INITIALIZERS, :55 SetEnchantmentsFunction.MAP_CODEC, :57-58 TradeCost absent. Blocks test_trade_components. Batch 3 overlap. |

Cross-batch oracle notes (no B4 item minted — see "Notes for the coordinator"):
- **VanillaCommandExecOracle** (Batch 9's oracle): **PASSES in 1.21.11, all three classpaths** (failed in 1.21.10 with 45 errors, first `:8` authlib.services package absent). test_container_component_names flips gap → pass.
- **VanillaLevelShapeOracle**: **PASSES in 1.21.11, all three classpaths** (failed in 1.21.10 with 2 errors, `:16` world.attribute package absent).
- **VanillaBlockUpdateOracle** (Batch 2's item): FAILS, 38 errors (was 63), first `:136` ServerLevel ctor, plus the `world.level.blockscan` package absent. B4 cascades on it (O03, O19).
- **VanillaMobAiOracle**: FAILS, 38 errors (was 103), first `:60` world.clock package absent.
- **VanillaDragonContactsOracle** (Batch 3's oracle, a CommandExec dependency): FAILS, 3 errors (was 5), first `:15`.
- **ParityFont** (helper): FAILS on all classpaths, 1 error both, `:43`. **VanillaTextDisplayRenderOracle** (helper): FAILS, 6 errors both, `:27`.

## Oracle defects withdrawn (2)

- **B4-O02** — VanillaBlockEntityStorageOracle passes in 1.21.11 (all three classpaths). test_block_entity_storage stays passing.
- **B4-O13** — VanillaPalettedContainerOracle passes in 1.21.11 (all three classpaths). test_paletted_container stays passing.

Also passing in 1.21.11 (no 1.21.10 item — they passed there too): VanillaChestLoadOracle, VanillaChestPairOracle, VanillaContainerTitleOracle, VanillaEnchantmentComponentsOracle, VanillaFoodStorageOracle, VanillaHeightmapOracle, VanillaRecipeBookViewOracle (client+both; server fails on the client package only, which is expected, not breakage), VanillaStateCodec, VanillaStorageMenuOracle, ParityData, SeedAgent.

## What passed

### Oracle-passing drivers (9)

test_block_entity_storage, test_paletted_container, test_chest_load, test_chest_pair, test_container_title, test_enchantment_components, test_storage_menu, test_recipe_book_view, **test_container_component_names** (flips gap → pass: VanillaCommandExecOracle now passes on all three classpaths; declared pass per the 26.2 precedent — it asserts container CustomName admission + slots {0,26} on disk, a live run would truly verify it). Declared pass per the 26.2 precedent (their oracles pass in the own measurement).

### No-oracle drivers passing (14)

- **test_falling_chest** — `falling_blocks.rs` + `native_falling.rs` byte-identical to 1.21.10; the chest arm exists in `containers.rs` `for_block`, so a landed chest restores its block entity and inventory.
- **test_jukebox_saved_clock** — the saved-clock mechanism is present (`ticks_since_song_started` Long; empty → absent, start → 0, tick → 1), asserted by the port's own `saved_song_clock_tests` integration test.
- **test_sculk_saved_fields** — the NBT arms carry `last_vibration_frequency` + `listener` (sensor), `warning_level` + `listener` (shrieker), `cursors` (catalyst); placement tags survive ticking and reload.
- **test_enchantment_map_validation** — `horizon-items/src/enchant.rs` byte-identical to 1.21.10.
- **test_world_pack_enchantment_loader** — same file, same carry.
- **test_anvil_turtle_egg** — falling-block tick byte-identical to 1.21.10 (see test_falling_chest).
- **test_crafting_priority** — `crafting.rs` carries the first-match sort key ("vanilla's recipe registration order, which first-match…").
- **test_craft_map_take** — `menu.rs:1310` `MapItem.onCraftedPostProcess`, shared by menu takes and system crafting. (G22 is about the recipe match, not the take post-processing.)
- **test_end_gateway** — the NBT arm exists (`block_entity_nbt.rs:438`) with `ExactTeleport` handling, citing `TheEndGatewayBlockEntity.loadAdditional`.
- **test_sign_literal** — the sign NBT arm writes `front_text`/`back_text` (`:461-462`) with a round-trip test for literal messages. (G27/G32/G33 concern invented fields, click gating and components — not the literal strings.)
- **test_machine_recipe_placement** — `Menu::place_recipe` exists (`menu/place.rs:380`).
- **test_recipe_book_variants** — `RecipeBookAdd` with per-display `ClientboundRecipeBookAddPacket.Entry` exists (`horizon-protocol/src/packets/recipes.rs:425-473`).
- **test_merchant_gate** — the corpus (`merchant_gate_corpus.py`) uses explicit NBT offers, not data-driven trade sets, so G25 never applied to it. What it tests — deferred gossip delivery (0 while frozen / NoAI, +2 at the first AI phase, special price −2, immediate XP) — the port implements exactly: `trading.rs:604` `villager_trade_pending` retains the player; `mobs/ai.rs:2091` (inside `Villager.customServerAiStep`) takes it and pushes `MobEvent::VillagerTrade`; `villagers.rs` `villager_traded_by` adds `GossipType::Trading` with `REPUTATION_CHANGE_PER_TRADE = 2`, entity event 14, and a price refresh; `merchant.rs:114` `update_special_prices` matches vanilla's f32 reputation formula exactly (verified against the `Villager.updateSpecialPrices` bytecode). G24 (immediate level-up) does not affect this corpus — it never observes the villager's level, and its few 1-tick pumps never elapse the 40-tick timer.
- **test_merchant_crafted** — same NBT-offers reasoning. The port's own unit test `merchant_takes_record_crafted_counts_once` (`menu.rs:3553`) asserts exactly the fixture's semantics — preview records nothing, payment 3 is rejected (no crafted counts, payment untouched, uses 0), two Pickup takes record (glass, 3) twice — and `crafted_counts` drains into `award_stat("crafted", …)` for host (`menu_clicks.rs:578`) and LAN guests (`lan_trading.rs:322`), with `traded_with_villager` awarded at `trading.rs:607`.

### Method-2 passes (19)

Lectern (books.rs lectern handling + `containers::Lectern`; page-turn sound `game.rs:1043`), comparator (`redstone.rs` `analog_output` → `vanilla_analog_output`, state sources + container fullness), anvil (`anvil.rs` byte-identical to 1.21.10 and 1.21.7), grindstone (stations.rs arms + `grindstone_xp_base`), smithing (stations.rs arms; template list 9 → 11 matches the 1.21.11 jar data), loom (loom.rs screen + pattern list with the 1.21.11 thumb-hover adaptation; `LoomResult` handling), stonecutter (stations.rs arms + `StoneResult` onTake), vault (`vault.rs` byte-identical to both), chiseled bookshelf (byte-identical to both), jukebox (see test_jukebox_saved_clock), campfire (`campfire.rs` byte-identical to 1.21.10), beehive (beehive.rs hive trait + honey level; comparator reads `honey_level`), shulker box (`storage.rs` `is_shulker_box` + `ShulkerBoxSlot.mayPlace` + shulker in `container_size`), sculk save fields (see above), falling-block BE restore (see test_falling_chest), container titles/names (VanillaContainerTitleOracle passes), enchanting offers/click (`menu.rs:1161` `enchant_changed`, `stations.rs:108` `is_enchantable`, bookshelf power offsets, `enchanting_book_opens_with_offers_and_turns_for_a_new_item` test; the click matches vanilla per the G12 withdrawal), sculk growth inhibition (G37 fixed), **furnace XP (NEW)** — `containers.rs` `create_experience` now takes the level random and draws `random.next_float() < fraction`, matching vanilla 1.21.11's `createExperience` (which uses `ServerLevel.random.nextFloat()`; 1.21.10 used `Math.random()`).

### Method-2 gaps (20)

Decorated-pot use (G15), merchant pricing (G23 + G24), sign NBT (G27), furnace tick (G01 + G04 + G05; G03/G06 fixed), spawner tick (G20; G11 fixed), cartography (G16), dispenser/dropper (G18 + G35; G07 never existed), hopper (G08 — entity-catch bowl only), ender chest (G14), save round-trip BE (G27 + G28 + G36), BE coverage (G13 + G15), crafter (G09 + G10 + G29; +G26 drop facet), container loot unpacking (G26), beacon (G31), brewing recipes (G02), brewing tick (G02 + G36), trial spawner tick (G34), menu engine (G21; G12 withdrawn), composter (G30), sign interaction (G32 + G33).

## Suspicions checked and refuted

- **"The spawner gate is broken because the port uses a name vanilla doesn't define"** — refuted. Vanilla 1.21.11 HAS the rule (`GameRules.SPAWNER_BLOCKS_WORK`, registered as `"spawner_blocks_work"`, checked via `ServerLevel.isSpawnerBlockEnabled()` and in `TrialSpawner.canSpawnInLevel`), and the port gates on the same name (game/spawner.rs:121,438, mobs/spawner.rs:472, mobs/trial_spawner.rs:664). G11 is FIXED. (Caveat, not an item: whether the `/gamerule` command surface accepts the vanilla key is Batch 9's territory and was not verified here.)
- **"The wax sound plays twice"** (G19) — refuted again for 1.21.11. 3002..=3005 route to `block_particle_event` (particles only) before the catch-all (game.rs:4367); the wax sound plays once via the explicit `wax_on_sound` / dispenser play (3003 at :8342/:8361 only).
- **"The sculk water gate admits flowing water"** — refuted (as in 1.21.10). Vanilla `SculkBlock.canPlaceGrowth` requires air OR (water block AND `fluidState.is(Fluids.WATER)`); `TypedInstance.is` is a type-identity check, so flowing water (`FLOWING_WATER`) fails it — the gate is source-only, and the port matches.
- **"Merchant float parsing (G25) breaks the merchant drivers"** — refuted for 1.21.11. Both merchant corpora use explicit NBT offers; vanilla 1.21.11 has no trade-set data at all; the port's live path is hardcoded tables. G25 is WITHDRAWN and both drivers pass.
- **"The 1.21.11 oracle sources are the repaired 1.21.10 forms"** — refuted. They are adapted 26.3 forms (168 files differ from 1.21.10's, mostly the `Identifier` rename and `holder.id()`), with one NEW wrong adaptation (`ChunkPos.pack`, which the jar does not have) — and the 26.3-form causes (`EntityTypes`, `ContainerInput`, `TradeCost`, 11-arg ServerLevel ctor) are unchanged.
- **"The crafter drops a plain crafter because its table has no functions"** (the 1.21.10 report's test_crafter_drop_name reasoning) — REFUTED, and the verdict corrected. The crafter's sole pool carries a `survives_explosion` condition under the plural `"conditions"` key; `conditions()` reads the list but `condition()` dispatches on `o.get("type")` only, so the condition evaluates false and the pool is skipped — the crafter drops nothing. See G26 and "Notes for the coordinator".

## Checks by driver class

**Oracle-passing drivers (9 pass):** test_block_entity_storage, test_paletted_container, test_chest_load, test_chest_pair, test_container_title, test_enchantment_components, test_storage_menu, test_recipe_book_view, test_container_component_names (VanillaCommandExecOracle — Batch 9).

**Oracle-blocked drivers (28 gap):** test_block_entity_tickers (O01), test_enchanting (O07), test_simple_container (O11), test_item_stack_storage (O12), test_container_screen (O17), test_stored_enchantment_patch (O20), test_recipe (O24), test_trade_components (O27), test_brewing_fuel (O05), test_dispenser (O06), test_menu_click (O08), test_recipe_book (O09), test_sculk (O14), test_block_entity_anim (O15), test_block_entity_render (O16), test_recipe_book_render (O18), test_place_recipe (O10), test_trading (O25), test_ghost_recipe (O26), test_crafter (O06), test_furnace (O03), test_brewing (O04), test_enchant_durability (O19), test_vehicle_container (O21), test_shulker_space (O22), test_spawner (O23), test_trial_spawner (O23), test_vault (O23).

**No-oracle drivers (14 pass, 6 gap):** pass — test_falling_chest, test_jukebox_saved_clock, test_sculk_saved_fields, test_enchantment_map_validation, test_world_pack_enchantment_loader, test_anvil_turtle_egg, test_crafting_priority, test_craft_map_take, test_end_gateway, test_sign_literal, test_machine_recipe_placement, test_recipe_book_variants, test_merchant_gate, test_merchant_crafted. Gap — test_furnace_loaded_total (G01/G04), test_furnace_output_components (G01), test_loot_mine_pot_components (G28/G26), test_pot_sherd_templates (G28), test_pot_hopper (G15), test_crafter_drop_name (G26 — NEW, corrects the 1.21.10 pass verdict).

**Method-2 (19 pass, 20 gap):** see "What passed" and the method-2 gap list above.

**Not counted:** test_composter_parity.py (references VanillaDispenserOracle; Batch 5's driver per the 26.1.2 precedent — the behaviour is B4-G30, not double-counted here).

## Notes for the coordinator

- **Status-convention deviation (flagged explicitly).** The 1.21.10 slice used status `"closed"` for re-checks fixed in-tree (G03, G06, G11, G37, O02, O13) and `"fixed"` only for G19. The 1.21.11 common brief defines the convention the other way: closed = premise fails (NEVER EXISTED / WITHDRAWN); fixed = re-check verified absent in the tree. This batch follows **common.md** (binding for this run): 7 fixed (G03, G06, G11, G19, G37, O02, O13), 3 closed (G07 never existed, G12 withdrawn, G25 withdrawn). The merge should not be surprised by the difference from the 1.21.10 slice.
- **Brief-list discrepancy.** The b04 brief says "60 items" but lists 59 by count and omits B4-O08, B4-O16, B4-O18 and B4-G37, all present in the 1.21.10 slice. All 63 slice items were decided; the four omitted ones are marked in the PRIMARY table.
- **1.21.10 correction — test_crafter_drop_name and G26 severity.** The 1.21.10 report declares test_crafter_drop_name pass on the reasoning that "a survives_explosion condition the port's `conditions()` reads" lets the crafter drop. That reasoning is wrong: `conditions()` reads the `"conditions"` list, but `condition()` dispatches on `o.get("type")` only, so the condition evaluates false and the pool is skipped — the crafter drops nothing in 1.21.10 too (the 1.21.10 tree's `condition()` is identical, `:1991`). The 1.21.10 B4 tally (95 checks, 41 pass, 54 gap) therefore contains one wrong driver verdict, and its G26 severity (minor) understates a defect whose measured blast radius is 856/1,083 block tables dropping nothing. This batch records the driver as gap and G26 as major; the 1.21.10 ledger should be corrected the same way.
- **CommandExec flip.** VanillaCommandExecOracle passes in 1.21.11 on all three classpaths (45 errors in 1.21.10). test_container_component_names flips gap → pass. Batch 9 owns the oracle; B4's driver table depends on it.
- **Wrong oracle adaptation — `ChunkPos.pack`.** The 1.21.11 oracle sources renamed `ChunkPos.asLong` → `ChunkPos.pack` (168 oracle files differ from 1.21.10's), but the 1.21.11 jar has no `pack` — only `asLong(int,int)` / `asLong(BlockPos)` (javap-verified). O01 and O23 both fail on it. The fix is to revert the rename in the oracle sources.
- **Gamerules table complete (Batch 9 note).** The port's `gamerules.rs` table (58 RULES + `MAX_MINECART_SPEED` = 59) exactly matches vanilla 1.21.11's 59 `GameRule` fields, zero differences either way (counted 2026-11-25; `b04/scratch/port-rules.txt` vs `b04/scratch/vanilla-rule-fields.txt`). Batch 9 owns gamerule enumeration — flagged as a note, not a B4 item.
- **Attribution (carried from 1.21.10).** test_crafter compiles VanillaDispenserOracle (`test_crafter_parity.py:242`), test_vault compiles VanillaNaturalSpawnOracle in vault mode (`test_vault_parity.py:175,193-195`) — NOT VanillaFurnaceOracle. Attributed to O06 and O23 respectively.
- **26.3-only content in the 1.21.11 tree (flagged, not minted):** a `potent_sulfur` block-entity arm in `containers.rs` `for_block` (`:515`); a `sulfur_spike` land sound in the level-event table (`game.rs`); `sulfur_cube_content` handling in the protocol skip table (correctly read-and-dropped as a component 1.21.11 predates). The sulfur skip is correct handling; the BE arm and sound are dead 26.3 leakage worth a look.
- **A wrong port comment:** `spawner.rs:446` claims 1.21.11 has `setEntityData`. It does not (jar-verified; `Spawner` has only `setEntityId`). Folded into G20/G34 evidence.
- **Cross-batch oracle ownership:** VanillaCommandExecOracle (Batch 9) and VanillaDragonContactsOracle (Batch 3) are Batch 3/9's; VanillaBlockUpdateOracle / VanillaLevelShapeOracle / VanillaMobAiOracle (Batch 2/3) are theirs. B4's driver table depends on CommandExec (test_container_component_names) and BlockUpdate (O03, O19 cascades). No 1.21.11 Batch 0 oracle table exists yet — when one lands, it should supersede this batch's family-scoped measurement for the shared oracles (the per-file counts here are the both-cp counts from `b04/scratch/oracle-measure-1.21.11.json`).
- **Dedup:** B4-O16's UvMapping facet dedups to B10-O03 (keep B10 canonical); B4-O18 dedups to B10-O05 (one of five renderpearl oracles). B4-O25/O27 (VanillaTradingOracle / VanillaTradeComponentsOracle) overlap Batch 3's trading territory — cross-reference, do not double-mint.
- **G17 stays UNVERIFIED** (smoker/blast halving masked by G01; no slice item), as in 1.21.10/26.1.2.
- **No prepared 1.21.11 `.local`** exists; all data counts are jar-direct. If a prepared `.local` lands later, the brewing-recipe count (0 data-driven recipes) and the tag/component absences should be re-confirmed against it — though the jar is authoritative for those.

## Verdict

1.21.11 Batch 4: **96 checks, 42 pass, 54 gap** as-tallied (59 behaviour: 33 pass, 26 gap; 37 non-behaviour: 9 pass, 28 gap). **63 items**: 28 behaviour gaps (5 major), 25 oracle defects (5 major), 7 fixed, 3 closed.

Headline: the template carries the same mixed inheritance as its child — the four 1.21.7 majors are all still live (furnaces never light, brewing never brews, cartography dead, composter dead) because the 1.21.7 tree's fixes were not carried into the template, and a fifth major joins them: the loot engine's 26.3 key names mean 856 of 1,083 block tables drop nothing at all, which also corrects the 1.21.10 report's test_crafter_drop_name pass verdict. But the template also carries real 1.21.11 adaptations the child lacks or shares (furnace burn/timer, sculk inhibition, the complete 59/59 gamerule registry rewrite, deferred merchant gossip, merchant crafted stats, level-random furnace XP, smithing templates), so seven items are fixed and both merchant drivers pass. The oracle side improved in places: CommandExec and LevelShape now pass, Furnace lost its own errors, and error counts fell sharply where the gamerules and gizmos packages now exist — but a new wrong adaptation (`ChunkPos.pack`) appeared, and the 26.3-form causes still block 28 drivers.
