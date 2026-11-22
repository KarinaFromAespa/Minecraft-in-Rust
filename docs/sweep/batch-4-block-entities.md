# Batch 4 — Block entities + containers (1.21.7 parity sweep)

**Date:** 2026-11-22 · **Branch:** `parity-sweep` (tip `d27d1d33`) · **Ledger slice:** `pins/1.21.7/sweep/batch-04.json`
**Status: DRAFT — uncommitted, awaiting the user's go-ahead.** An execution-risk notice blocked all repo execution, git commands, cargo builds/tests and driver runs, so this batch ran the prescribed fallback (Batch 2 precedent): static analysis only, deliverables saved as drafts under `~/workspace/mcir-work/sweep-batch-4-deliverables/`.

## Tally

| | Count |
|---|---|
| Checks total | 91 |
| Pass | 48 |
| Gap | 43 |
| Behaviour defects (B4-Gxx) | 16 (3 major, 13 minor) |
| Oracle defects (B4-Oxx) | 26 (12 major, 14 minor) |
| Oracle repairs done | 0 (blocked by the notice) |

28 of the 43 gaps are "driver cannot run (oracle broken)", not behaviour defects. The remaining 15 gaps are behaviour divergences found by method-2 analysis (javap vs code) plus 2 no-oracle drivers whose assertions fail against the code (B4-G04, B4-G15).

## Method

No driver was run and no code was executed. Every check is `javap -c` on the named 1.21.7 server jar (`~/workspace/mcir-work/leakage-audit/jars/1.21.7-server-named.jar`), plus the Rust source in `versions/1.21.7/crates/` and the prepared data in `~/workspace/mcir-work/mob-spawn/resources/`, plus jar-data scans (`unzip -l` for tag files). "Pass" means the static comparison found no divergence; it is not a run result. Oracle statuses are Batch 0's measurement (`docs/sweep/batch-0-oracles.json`).

Vanilla 1.21.7's own jar/data is the spec throughout (owner's vanilla-check rule). Where the port's behaviour matches the 26.3 base but not 1.21.7, that is recorded as a defect.

## Systemic finding

Three of the sixteen behaviour defects (B4-G01, B4-G02, B4-G16) share the pilot's systemic cause: the 1.21.7 port carries 26.x component/tag logic for lookups that do not exist in vanilla 1.21.7, and every such lookup silently degrades — the fuel component is absent from all 1,416 items, the brewing-fuel component is absent where vanilla uses the `#brewing_fuel` tag, and `#extendable_maps` is absent where vanilla checks no tag at all. B4-G11 is the same pattern in the gamerule table (a 26.x-only rule gating spawner ticking). The prepared data is faithful to the jar — the port code, not the data, is the 26.x carry-over. The other thirteen defects are one-off logic divergences (ordering, rounding, missing fallbacks, LAN-only scoping).

Oracle breakage is systemic too: 26 of the batch's oracles fail, mostly on 26.x-only packages (`gamerules`, `gizmos`, `client`/`renderpearl`/`blaze3d`) or 26.x API shapes — the oracle suite was written against 26.3 and needs a 1.21.7 retarget, not per-oracle fixes alone. The four client-jar oracles (anim, render, container screen, recipe-book screen, ghost recipe) additionally need the 1.21.7 *client* jar, which the harness does not currently wire in.

## Behaviour defects

### Major

**B4-G01 — Furnaces (furnace / blast furnace / smoker) can NEVER light.**
`Item.fuel` comes only from the `minecraft:cooking_fuel` component (`horizon-items/src/items.rs:1009-1011`); 0/1,416 items in 1.21.7 `items.json` carry it and no JSON in the tree defines it. Vanilla 1.21.7 has no fuel component at all (`DataComponents` javap: 0 fuel refs); burn times are the hardcoded `FuelValues.vanillaBurnTimes` table (lava bucket 20000, coal block 16000, blaze rod 2400, coal/charcoal 1600, `#logs` 300, …). Consequences: `Providers::burn_time` is 0 for every item, so `Furnace::tick` never lights; `menu.rs is_fuel` is false, so the fuel slot refuses all fuel (only buckets pass via `is_bucket`); hopper fuel insertion and furnace quick-move fuel routing are likewise dead. The furnace tick logic itself otherwise matches `AbstractFurnaceBlockEntity.serverTick` exactly (lit decrement, gate, `canBurn`, fuel consume, cook increment, cooldown −2 clamp, LIT flags 3, `setChanged` conditions).

**B4-G02 — Brewing stands can NEVER brew.**
`brewing_fuel()` (`horizon-client-render/src/brewing.rs:423`) reads only the `minecraft:brewing_fuel` component, absent in 1.21.7. Vanilla uses the `#brewing_fuel` item tag (blaze powder — the tag *exists* in the 1.21.7 jar data) plus `FUEL_USES=20` (`BrewingStandBlockEntity.serverTick` bytecode). The brew logic itself matches vanilla exactly (400 ticks, fuel−− on start, ingredient-identity reset, `doBrew` gating, bottle bits). Also breaks brewing quick-move fuel routing (blaze powder cannot be shift-clicked into the fuel slot).

**B4-G16 — Cartography paper zoom-out never works.**
`menu.rs carto_changed` gates paper zoom on the `minecraft:extendable_maps` item tag, which does not exist in the 1.21.7 jar data. Vanilla 1.21.7 `CartographyTableMenu.setupResultSlot` checks paper with no tag at all — `extra.is(PAPER) && !locked && scale < 4` (javap; no `ItemTags` reference anywhere in the class). Glass-pane lock and map copy (count 2) match vanilla, as does cartography quick-move (verified against vanilla `quickMoveStack` bytecode).

### Minor

**B4-G03 — Furnace `burn()` grows output by the full result count; vanilla grows by 1** (datapack-only). Vanilla grows by exactly 1 when the output slot is non-empty (`ItemStack.grow(1)`), full result copy only when empty; `canBurn` likewise allows `out.count < max` regardless of result count. Unreachable in vanilla data (all cooking recipes yield 1).

**B4-G04 — Furnace completion uses `>=` where vanilla uses `==`.** With `cookingTotalTime=0` (loaded save) vanilla never completes; Rust completes in 1 tick. A Rust test pins the `>=` behaviour, contradicting vanilla bytecode. In `test_furnace_loaded_total`'s scope ("zero" totals).

**B4-G05 — Furnace fuel remainder DROPPED when the fuel stack remains** (datapack-only). Vanilla only places the remainder when the fuel stack empties. Unreachable in vanilla data (only lava bucket has a remainder, max stack 1).

**B4-G06 — Lit furnace + input with NO recipe keeps its cooking timer; vanilla resets it to 0.** Reachable only via datapack reload mid-cook / NBT edit.

**B4-G07 — Dispensed spawn egg with removed `minecraft:entity_data` drops as an item; vanilla spawns the default mob.** Vanilla `SpawnEggItem.getType` falls back to `defaultType`. Commands-only corner.

**B4-G08 — Hopper entity-catch uses the narrow bowl, not the full SUCK_AABB.** Pickup can only be 1–2 ticks late for fast items at the edges, never missed (the tick path covers the full AABB).

**B4-G09 — Crafter never ejects into container ENTITIES** (chest/hopper minecart, chest boat). `crafter_eject` consults only the block-only `Containers::view`; vanilla uses `HopperBlockEntity.getContainerAt` (block + entity). The result drops as an item instead.

**B4-G10 — Crafter `recipe_crafted` trigger fires only for the host player** (LAN-only). Vanilla triggers for every `ServerPlayer` in the 17-box.

**B4-G11 — Spawner + trial spawner ticking gated on non-vanilla `spawner_blocks_work` gamerule.** Vanilla 1.21.7 has no such rule; `BaseSpawner.serverTick` checks no gamerule at all and `TrialSpawner.canSpawnInLevel` checks only `overridePeacefulAndMobSpawnRule || (difficulty != PEACEFUL && doMobSpawning)`. The port defaults the rule to true, so default play is unaffected, but the gate and the rule are 26.x carry-over. (The Rust trial spawner also has no `overridePeacefulAndMobSpawnRule` equivalent — unexamined corner, not recorded as a defect.)

**B4-G12 — Creative enchanting without lapis allowed.** Vanilla `EnchantmentMenu.clickMenuButton` returns false when the lapis slot is empty even in creative. Survival behaviour matches.

**B4-G13 — No Bed block entity.** `BlockEntity::for_block` returns `None` for beds. Renamed beds lose their custom name on placement, `/data get block` on a bed returns nothing, and chunk saves carry no bed BEs — this explains the world-compare gap (bed blocks 50=50, bed BEs 0 vs 50). Colour itself is safe (per-block in 1.21.7).

**B4-G14 — Ender chest shows the wrong inventory to LAN guests** (LAN-only). `open_ender_chest` swaps the *host's* `ender_items` into the shared block BE; a guest sees the host's items (while the host has it open) or an empty chest — never their own `EnderItems`. Single-player behaviour is correct.

**B4-G15 — Hoppers/droppers cannot see decorated pots.** `Containers::view` requires a `BlockEntity`; pots have none (item + sherds live in a spill saved tag). Vanilla returns the pot as a 1-slot container. Hopper insert/extract, dropper insert, and comparator reads of pots all silently fail. Manual insert via `use_decorated_pot` works. In `test_pot_hopper`'s scope.

## Oracle defects

One item per broken oracle (Batch 2 precedent). No repairs attempted — repair requires editing repo Java + compiling to verify, both prohibited by the notice.

| ID | Oracle | Errors | First error | Blocks |
|---|---|---|---|---|
| B4-O01 | VanillaBlockEntityTickerOracle | 8 | `net.minecraft.util.debug` missing | test_block_entity_tickers |
| B4-O02 | VanillaBlockEntityStorageOracle | 4 | cannot find symbol | test_block_entity_storage |
| B4-O03 | VanillaFurnaceOracle | 1 | cannot find symbol | test_furnace |
| B4-O04 | VanillaBrewingOracle | 14 | cannot find symbol | test_brewing |
| B4-O05 | VanillaBrewingFuelOracle | 8 | cannot find symbol | test_brewing_fuel |
| B4-O06 | VanillaDispenserOracle | 54 | `world.level.gamerules` missing | test_dispenser |
| B4-O07 | VanillaEnchantingOracle | 4 | cannot find symbol | test_enchanting |
| B4-O08 | VanillaMenuOracle | 9 | cannot find symbol | test_menu_click (+ cascades) |
| B4-O09 | VanillaRecipeBookOracle | cascade | depends on VanillaMenuOracle | test_recipe_book |
| B4-O10 | VanillaPlaceRecipeOracle | cascade | depends on VanillaMenuOracle | test_place_recipe |
| B4-O11 | VanillaSimpleContainerOracle | 1 | cannot find symbol | test_simple_container |
| B4-O12 | VanillaItemStackStorageOracle | 1 | cannot find symbol | test_item_stack_storage |
| B4-O13 | VanillaPalettedContainerOracle | 4 | cannot find symbol | test_paletted_container |
| B4-O14 | VanillaSculkOracle | 60 | `net.minecraft.gizmos` missing | test_sculk |
| B4-O15 | VanillaBlockEntityAnimOracle | 45 | client model package missing | test_block_entity_anim |
| B4-O16 | VanillaBlockEntityRenderOracle | 99 | blaze3d missing | test_block_entity_render |
| B4-O17 | VanillaContainerScreenOracle | 54 | `net.minecraft.client` missing | test_container_screen |
| B4-O18 | VanillaRecipeBookScreenOracle | 112 | renderpearl missing | test_recipe_book_render |
| B4-O19 | VanillaEnchantDurabilityOracle | cascade | depends on VanillaBlockUpdateOracle (Batch 2 O1 — do not double-count the repair) | test_enchant_durability |
| B4-O20 | VanillaStoredEnchantmentPatchOracle | 1 | cannot find symbol | test_stored_enchantment_patch |
| B4-O21 | VanillaVehicleContainerOracle | 5 | cannot find symbol | test_vehicle_container |
| B4-O22 | VanillaShulkerSpaceOracle | 2 | cannot find symbol | test_shulker_space |
| B4-O23 | VanillaNaturalSpawnOracle | 43 | `world.level.gamerules` missing | test_spawner, test_trial_spawner, test_vault |
| B4-O24 | VanillaRecipeOracle | 10 | `Recipe.assemble` signature | test_recipe |
| B4-O25 | VanillaTradingOracle | 94 | `net.minecraft.gizmos` missing | test_trading |
| B4-O26 | VanillaGhostRecipeOracle | 16 | client recipebook package missing | test_ghost_recipe |

Severities in the JSON: major for ≥10 errors, shared dependencies (O08), and multi-driver blocks (O23); minor otherwise. The 1-error oracles (O03, O11, O12, O20) are the cheapest repairs in the batch.

## What passed (method-2 verifications)

These behaviours were verified against the 1.21.7 jar by bytecode + source and found correct. (Where the corresponding driver is oracle-blocked, the driver check itself is still a gap — the driver cannot run — but the behaviour is verified here.)

- **Hopper** (except B4-G08): push cooldown, eject-then-suck order, hopper-to-hopper 8-tick skip + `>8` custom-cooldown guard, container-first suck + full-block-above blocking + `#does_not_block_hoppers` (tag exists in 1.21.7), SUCK_AABB exact, random entity-container pick.
- **Dispenser/dropper** (except B4-G07): full 60-item + 3-block + spawn-egg + shulker + boat + minecart registry, equipment fallback, reservoir sampling, dropper insert/fail paths, empty-dispenser 1001 + BLOCK_ACTIVATE, potion mud conversion, `spawnItem` constants exact.
- **Crafter** (except B4-G09/G10): 6-tick cycle, `dispenseFrom` events, `canPlaceItem`/`smallerStackExist`, redstone signal, one-by-one dispense, advancement 17-box.
- **Spawner** (except B4-G11): defaults, load/save (shorts, potentials, update tag without potentials), `delay()` re-roll + broadcastEvent(1), spawn position/yaw/max-nearby, `weighted_index` = `WeightedList.getRandom`, client spin.
- **Trial spawner** (except B4-G11): full state machine verified against `TrialSpawnerState.tickAndGetNext` bytecode (all six states, transition order, `countAdditionalPlayers` = max(0, detected−1), 40-tick shutter delay, eject-every-30-ticks with the readiness gate checked first, omen conversion 18000 ticks/level). `PlayerDetector` verified: block-position distance² < range² + creative/spectator filter + center→eye line of sight. MAX_MOB_TRACKING_DISTANCE = 47.
- **Vault**: constants (4.0/4.5/128/14/15/20), state machine, `tryInsertKey` verified against bytecode including `isSameItemSameComponents` key check, ejection one item per 20 ticks.
- **Anvil**: full `createResult` verified by bytecode — material repair loop, same-item merge (+12%), enchantment merge (same level +1, compatibility fees, book halving, count>1 → 40), rename (+1), cost order (rename-only cap to 39 *before* the ≥40 too-expensive check), repair-cost doubling.
- **Grindstone**: merge durability +5%, curses kept, book→book, repair cost 2ⁿ−1, XP = min-cost sum rolled as ceil(half)+nextInt(half).
- **Enchanting** (except B4-G12): cost formula, `selectEnchantment` (level modifier, ±15% span, weighted pick, halving loop), clue reseeding, book transmute, lapis/level checks, deducts slot+1 levels (not the cost), seed re-roll. `LEVEL_CODEC = intRange(1, 255)` confirmed by bytecode.
- **Beacon**: 10-block/tick beam scan, `#beacon_base_blocks` layers (tag exists), 80-tick effects (range levels×10+10, duration (9+levels×2)×20, amplifier rules), CONSTRUCT_BEACON box, menu payment validation.
- **Merchant**: special prices (reputation + Hero of the Village), demand pricing, `satisfied_by`/`take` (including (b,a) fallback), restock demand update, level-up/XP orbs, reputation deferred to the villager's AI step (`villager_trade_pending`). `MerchantResultSlot.onTake` javap: no `onCraftedBy` — the port matches.
- **Menu engine** (except fuel routing via B4-G01/G02): quick-move routes for all 25 menus verified against vanilla `quickMoveStack` implementations; click engine (drag, quick-move repeat loop, pickup, swap, clone, throw, pickup-all) matches `AbstractContainerMenu.doClick`; `pickup_inner`/`swap` edge arms match.
- **Loom / stonecutter / smithing**: loom selection + 6-layer cap, stonecutter recipe list + take, smithing first-match + optional ingredient matching.
- **Comparator**: `getRedstoneSignalFromContainer` formula (incl. crafter special-case), chiseled-bookshelf last-slot signal, lectern page signal.
- **Lectern**: `#lectern_books` tag exists (written + writable book); place/read/page/signal all match.
- **Campfire / beehive / jukebox / sculk (save fields) / signs**: cook/cooldown ticks, hive release (600-tick minimum, nectar, hum), jukebox saved clock presence, sculk `last_vibration_frequency`/`warning_level`/listener/cursors round-trip, sign component preservation (original component kept verbatim while text is unchanged).
- **Storage**: chest load/pair (right-half-first double chest), container titles via the generic `block_names` map, ender-chest player inventory (single-player), shulker-box menu refusal, falling-chest BE creation + `TileEntityData` restore, decorated-pot manual use (insert/wobble/loot unpack), recipe lookup first-match in registration order, recipe-book multi-display handling, datapack enchantment loading, `/enchant` validation.

## Suspicions checked and refuted

- Trial-spawner/vault player detection is **not** box-vs-sphere wrong: vanilla `PlayerDetector` uses block-position distance² < range², exactly as the port does.
- Vault key check is **not** item-only: vanilla `isValidToInsert` uses `isSameItemSameComponents`, as the port does.
- Trial-spawner ejection order is **not** wrong: vanilla checks the 30-tick readiness gate *before* the empty-players check, as the port does.
- Merchant takes do **not** award the crafted statistic in vanilla (`MerchantResultSlot.onTake` has no `onCraftedBy`); the port matches.
- The brewing quick-move bottle arm is **not** broken by the missing `#brewing_potion_inputs` tag: `is_bottle` uses the prepared inputs set.
- Anvil rename-cap order matches vanilla bytecode exactly (cap to 39 before the too-expensive check).

## Checks

### Drivers — oracle passing (6, all pass, static)

`test_chest_load`, `test_chest_pair`, `test_container_title`, `test_enchantment_components`, `test_storage_menu`, `test_recipe_book_view` (client+both pass; server fails on the client package).

### Drivers — no oracle (23: 21 pass, 2 gap)

Pass (static): `test_crafter`, `test_furnace_output_components`, `test_jukebox_saved_clock`, `test_sign_literal`, `test_merchant_gate`, `test_merchant_crafted`, `test_crafter_drop_name`, `test_falling_chest`, `test_loot_mine_pot_components`, `test_pot_sherd_templates`, `test_recipe_book_variants`, `test_sculk_saved_fields`, `test_enchantment_map_validation`, `test_world_pack_enchantment_loader`, `test_anvil_turtle_egg`, `test_crafting_priority`, `test_machine_recipe_placement`, `test_craft_map_take`, `test_container_component_names`, `test_end_gateway`, `test_trade_components`.
Gap: `test_pot_hopper` (B4-G15), `test_furnace_loaded_total` (B4-G04).

### Drivers — oracle blocked (28, all gap)

`test_block_entity_tickers`, `test_block_entity_storage`, `test_furnace`, `test_brewing`, `test_brewing_fuel`, `test_dispenser`, `test_enchanting`, `test_menu_click`, `test_recipe_book`, `test_simple_container`, `test_item_stack_storage`, `test_paletted_container`, `test_sculk`, `test_block_entity_anim`, `test_block_entity_render`, `test_container_screen`, `test_recipe_book_render`, `test_enchant_durability`, `test_stored_enchantment_patch`, `test_vehicle_container`, `test_shulker_space`, `test_spawner`, `test_trial_spawner`, `test_vault`, `test_place_recipe`, `test_recipe`, `test_trading`, `test_ghost_recipe`.

### Method-2 checks (34: 21 pass, 13 gap)

Pass: beacon, lectern, comparator, anvil, grindstone, smithing, loom, stonecutter, vault, chiseled bookshelf, jukebox, campfire, beehive, decorated-pot use, shulker box, merchant pricing, sculk save fields, sign NBT, falling-block BE restore, container titles/names, brewing recipes.
Gap: hopper (B4-G08), ender chest (B4-G14), save round-trip BE (B4-G13), BE coverage (B4-G13/G15), cartography (B4-G16), brewing tick (B4-G02), furnace tick (B4-G01/G03–G06), spawner tick (B4-G11), trial spawner tick (B4-G11), crafter (B4-G09/G10), dispenser/dropper (B4-G07), menu engine (B4-G01/G02), enchanting offers/click (B4-G12).

### Excluded from the tally

Other batches' scope: `test_bonus_chest`, `test_decorated_region` (worldgen); `test_storage_alias` (commands/Batch 10); `test_food_storage` (items/Batch 7); `test_server_player_sculk` (player/Batch 3); `test_shulker_bullet` (entity/Batch 3); `test_held_item_stack` (items/Batch 7).
Unmeasured (no oracle exists; needs a live run): `test_disconnect_crafting` (guest disconnect drop path not located statically), `test_hive_default_patch` (beehive save prototype elision not read).

### Not verified

Sculk tick mechanics beyond save fields (oracle-blocked); command-block execution semantics (commands scope — the BE data model is complete); full jukebox song lifecycle (the driver disclaims it); recipe-book *server* packet construction (client side verified); trial-spawner `TIME_BETWEEN_EACH_EJECTION` constant value (30, from vanilla source — javap shows it non-constant-folded); the save_roundtrip harness failure itself (owned by the save harness — only its BE touchpoint, B4-G13, is recorded here). Sign `filtered_messages` are dropped on load (caveat, no defect ID).

## Notes for the coordinator

- The three MAJOR defects (B4-G01 furnaces, B4-G02 brewing, B4-G16 cartography zoom) are all one-line-class fixes at the data-lookup layer (restore the vanilla burn-times table / the `#brewing_fuel` tag / drop the `#extendable_maps` gate) — but fixes are a separate decision; nothing was changed here.
- The cheapest oracle repairs are the four 1-error oracles (B4-O03 furnace, B4-O11 simple container, B4-O12 item-stack storage, B4-O20 stored-enchantment patch). The most valuable single repair is B4-O08 (MenuOracle — unblocks three drivers) followed by B4-O23 (NaturalSpawnOracle — unblocks three drivers).
- B4-O19 shares its repair with Batch 2's O1 (`VanillaBlockUpdateOracle`); do not schedule it twice.
- The four client-jar oracles (B4-O15, B4-O16, B4-O17, B4-O18, plus B4-O26) need the 1.21.7 client jar wired into the harness; they cannot be repaired against the server jar alone.
- Deliverables are drafts under `~/workspace/mcir-work/sweep-batch-4-deliverables/` (`batch-04.json`, this file, `NOTES.md`, `javap-*.txt` evidence). They need committing to `parity-sweep` (`docs/sweep/batch-4-block-entities.md`, `pins/1.21.7/sweep/batch-04.json`) once the user gives the go-ahead. `pins/1.21.7/sweep-items.json` was not touched (coordinator merges slices).
