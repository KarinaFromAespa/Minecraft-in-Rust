# Batch 4 — Block entities + containers (26.2 parity sweep)

**Date:** 2026-11-24 · **Branch:** `parity-sweep-26.2` (tip `e30ceac7`) · **Ledger slice:** `batch-04.json` (this batch's scratch dir)
**Status: DRAFT — uncommitted, awaiting the coordinator.** Checks + triage only; no fixes, no edits under `versions/26.2/`, no commits, no cargo builds, no driver runs.

**Coordinator note (2026-11-24):** this batch used the measured Batch 0
table directly — all 10 oracle-defect claims verify exactly against
`docs/sweep/26.2/batch-0-oracles.json`, and all 16 withdrawn oracles pass.
No corrections needed. Tally arithmetic verified (26 + 20 + 11 + 34 = 91;
74 pass; 17 gap). **Cross-batch overlap, dedup at merge time:** B4-O16
(`VanillaBlockEntityRenderOracle`, 7 errors) is the same oracle defect as
B10-O03 (recorded major there, with the UvMapping cause); B4-O18
(`VanillaRecipeBookScreenOracle`, 4 errors, renderpearl) is one of the
five oracles grouped under B10-O05. At merge, fold B4-O16 into B10-O03
and B4-O18 into B10-O05, keeping the B10 items canonical and these as
cross-references — do not double-count them in the merged ledger.

## Tally

| | Count |
|---|---|
| Checks total | 91 |
| Pass | 74 |
| Gap | 17 |
| Behaviour defects (B4-Gxx) | 5 (0 major, 5 minor) |
| Oracle defects (B4-Oxx) | 10 (2 major, 8 minor) |
| 1.21.7 behaviour defects re-checked | 16: 5 fixed in the port, 6 withdrawn, 5 carried over |
| 1.21.7 oracle defects re-checked | 26: 16 withdrawn (oracle now passes), 10 carried over |

11 of the 17 gaps are "driver cannot run (oracle broken)", not behaviour defects. 1 gap is a no-oracle driver whose assertions fail against the code (B4-G15, `test_pot_hopper`). The remaining 5 gaps are method-2 behaviour divergences (B4-G08/G09/G10/G14/G15).

## Method

No driver was run and no code was executed. Every check is `javap -c` on the named 26.2 server-game jar (`~/workspace/mcir-work/26.1-adapters/versions/26.2/jars/server-game.jar`, Mojang-mapped), plus the Rust source in `versions/26.2/crates/` (read-only) and the prepared 26.2 data in `versions/26.2/.local/horizon-client/26.2/`, plus jar-data scans (client jar `leakage-audit/jars/26.2-client.jar`, a game jar) and the 26.2 datagen reports. "Pass" means the static comparison found no divergence; it is not a run result. Oracle statuses are Batch 0's MEASURED table (`docs/sweep/26.2/batch-0-oracles.json`, quoted with pass/fail + error class + error count throughout) — no import-level substitution.

Vanilla 26.2's own jar/data is the spec throughout (owner's vanilla-check rule). Where the port's behaviour matches the 26.3 base but not 26.2, that is recorded as a defect; where 26.2's vanilla differs from 1.21.7's, the 1.21.7 defect is withdrawn, not carried over.

**Prepare landed during this run.** At spawn time the prepare was still running; it finished (2048.7 s, EXIT 0) and all exports the batch needs are present: `items.json`, `registries.json`, `block-behaviours.json`, `gameplay-facts.json`, `entity-types.json`, `attributes.json`, `block-tags.json`, `data/` (tags), `worldgen/`. `sounds.json` and lang files are still absent (the VM lacks the launcher asset index) — expected, not a finding. No check in this batch was blocked on prepared data.

**Diff-based carry-over.** The sweep worktree holds the 1.21.7 tree, so every Batch 4 Rust file was diffed 1.21.7 → 26.2. Files with 0 changed lines (`vault.rs`, `sculk.rs`, `game/beacon.rs`, `game/campfire.rs`, `game/storage.rs`, `game/menu_clicks.rs`, `game/lan_stations.rs`, `game/recipe_book.rs`, `game/saving.rs`) carry the 1.21.7 method-2 pass directly; files with small diffs were re-verified at each hunk against the 26.2 jar (see "What passed"). The rewritten files (`furnace.rs`, `brewing.rs`, `items.rs` fuel table, `menu.rs` fuel/cartography arms) were verified in full against 26.2 bytecode, including two programmatic table comparisons (fuel values, brewing mixes — both exact).

## Systemic finding

The 1.21.7 systemic cause — the port carrying 26.x component/tag logic for lookups that do not exist in vanilla 1.21.7 — is **inverted for 26.2**: the 26.2 port was written against 26.2's own forms and all three 1.21.7 MAJOR defects are fixed here (furnaces light from the `FuelValues` table, brewing stands brew from the `#brewing_fuel` tag, cartography zooms with no tag gate). The port's 26.2-specific corrections are thorough and jar-accurate: the 64-entry fuel table matches `FuelValues.vanillaBurnTimes` entry-for-entry (verified programmatically, including the `#non_flammable_wood` removal and the two 26.2-new entries `#wooden_shelves` 300 and `wooden_spear` 200), and the 63 potion mixes + 2 container mixes match `PotionBrewing.addVanillaMixes` in set and order.

Six of the sixteen 1.21.7 defects are withdrawn because **vanilla itself changed between 1.21.7 and 26.2** — the 1.21.7 defect was real there, but the same code is correct against 26.2: furnace `burn()` now grows by the full result count, a lit furnace with no recipe now keeps its timer, spawn eggs lost the `defaultType` fallback, `spawner_blocks_work` now exists and is checked, creative enchanting no longer requires lapis, and beds have no block-entity type at all. The five remaining defects are one-off logic divergences carried over unchanged from 1.21.7 (hopper bowl, crafter eject/trigger, ender-chest LAN scope, decorated-pot container view).

Oracle breakage is far lighter than 1.21.7's: 16 of the 26 Batch 4 oracles now compile against the 26.2 jars. The 10 that still fail are 1–7 errors each (median 1), dominated by two one-line-class repairs (`VanillaMenuOracle`, 1 missing symbol, blocking 4 drivers; `VanillaDispenserOracle`, 1 no-override, blocking 6 drivers sweep-wide) plus the `fuelValues()` abstract-method gap (Batch 0 G1) and the 26.3-only `renderpearl` package (Batch 0 G4).

## Behaviour defects

All five are minor carry-overs from 1.21.7. There are **no major behaviour defects** in this batch for 26.2.

### Minor

**B4-G08 — Hopper entity-catch uses the narrow bowl, not the full SUCK_AABB.** `hoppers_catch_items` (`horizon-client/src/game/machines.rs:941`, bowl check at 963–974) tests the narrow INSIDE bowl (0.125..0.875, 11/16..1) while vanilla 26.2 `HopperBlockEntity.entityInside` intersects the item's box with `getSuckAabb()` (javap; `Hopper` static init: `Block.column(16, 11, 32)` → 0..1, 11/16..2). The tick path covers the full AABB, so pickup can only be 1–2 ticks late for fast items at the edges, never missed. Unchanged from 1.21.7.

**B4-G09 — Crafter never ejects into container ENTITIES** (chest/hopper minecart, chest boat). `crafter_eject` (`horizon-client/src/game/machines/crafter.rs:144`) consults only the block-only `Containers::view` (`machines.rs:213`, which returns `None` without a block entity at the position), while vanilla 26.2 `CrafterBlockEntity` dispense uses `HopperBlockEntity.getContainerAt` (block container first, then `getEntityContainer`, javap-verified). Hopper and dropper have the entity fallback in the port (`entity_container_at`, `machines.rs:1274`, used at 1055/1179/1536); the crafter lacks it. The result drops as an item instead. Unchanged from 1.21.7.

**B4-G10 — Crafter `recipe_crafted` trigger fires only for the host player** (LAN-only). `crafter_eject`'s trigger block (`machines/crafter.rs:180–205`) checks only `self.player` against the 17-box; vanilla 26.2 `CrafterBlock.dispenseItem` iterates `level.getEntitiesOfClass(ServerPlayer, AABB.ofSize(center, 17, 17, 17))` and triggers every player in range (javap). The 8.5 half-extent strict-intersects geometry matches for the host; only the scope diverges. LAN guests in range miss the trigger. Unchanged from 1.21.7.

**B4-G14 — Ender chest shows the wrong inventory to LAN guests** (LAN-only). `open_ender_chest` (`horizon-client/src/game/storage.rs:157–163`) swaps the *host's* `ender_items` into the shared block BE; close (`storage.rs:217`) copies back and leaves an empty Chest(27). A guest opening the same chest sees the host's items (while the host has it open) or an empty chest — never their own `EnderItems`. Single-player behaviour is correct. Unchanged from 1.21.7 (`storage.rs` has 0 changed lines).

**B4-G15 — Hoppers/droppers cannot see decorated pots, and the port has no pot block entity at all.** `Containers::view` requires a `BlockEntity` in `world.block_entities`; `BlockEntity::for_block` returns `None` for `minecraft:decorated_pot` (no branch in `containers.rs:376–516`, no entry in `container_size`, `containers.rs:320–332`). Vanilla 26.2 HAS the `minecraft:decorated_pot` block-entity type (registry + `DecoratedPotBlockEntity`, which implements `ContainerSingleItem$BlockContainerSingleItem` — a 1-slot container) and `HopperBlockEntity.getContainerAt` returns it. Consequences beyond 1.21.7's: chunk saves carry no pot BEs (the port keeps pot item + sherds in a spill saved tag instead), so a port-written save loses pot contents to vanilla 26.2, and `/data get block` on a pot returns nothing. Hopper insert/extract, dropper insert, and comparator reads of pots all silently fail. Manual insert via `use_decorated_pot` works. In `test_pot_hopper`'s scope (still a gap).

## Withdrawn / fixed 1.21.7 defects (not gaps for 26.2)

### Fixed in the 26.2 port

**B4-G01 — FIXED: furnaces light.** 26.2 has no `minecraft:cooking_fuel` component (0/1,537 items in `items.json` carry it; no fuel-ish entry among the 111 data component types in `registries.json`). Vanilla 26.2 burn times are the hardcoded `FuelValues.vanillaBurnTimes` table, reached via the abstract `Level.fuelValues()`. The port implements the table as `FUEL_VALUES` (`horizon-items/src/items.rs:734–824`, 64 entries) and `Providers::burn_time` (`horizon-client-render/src/providers.rs:112–113`) reads it, halving for smoker/blast furnace (verified: both `SmokerBlockEntity`/`BlastFurnaceBlockEntity.getBurnDuration` return `super/2` in the jar). Programmatic comparison of the parsed `vanillaBurnTimes` bytecode against `FUEL_VALUES`: 64/64 entries, same order, same values (lava bucket 20000, coal block 16000, blaze rod 2400, coal/charcoal 1600, `#logs`/`#planks`/`#wooden_shelves` 300, `#wool_carpets` 67, dried kelp block 4001), plus the `remove(#non_flammable_wood)` step (`items.rs:822`). `menu.rs is_fuel` (`menu.rs:1439–1440`) and hopper fuel insertion (`machines.rs:420`) use the same table. Furnace tick logic verified in full against `AbstractFurnaceBlockEntity.serverTick` (lit decrement, gate, `canBurn`, fuel consume, cook increment, cooldown −2 clamp, LIT flags 3, `setChanged` conditions) — all match.

**B4-G02 — FIXED: brewing stands brew.** 26.2 has no `minecraft:brewing_fuel` component; vanilla 26.2 `BrewingStandBlockEntity.serverTick` and `canPlaceItem` test `ItemTags.BREWING_FUEL` (javap; the tag exists in the 26.2 jar data and the prepared `data/minecraft/tags/item/brewing_fuel.json`). The port's `brewing_fuel()` (`horizon-client-render/src/brewing.rs:723`) reads the tag. The full tick was verified against the jar (refuel 20 uses, 400-tick brew, fuel−− on start, ingredient-identity reset, `doBrew` gating including the remainder-drop path, bottle bits, level event 1035) — all match. The 63 potion mixes + 2 container mixes match `PotionBrewing.addVanillaMixes` exactly (programmatic comparison, set and order, including the `addStartMix` water→mundane / awkward→potion expansion). Brewing quick-move fuel routing works via the tag (`menu.rs:1474`, `2335`).

**B4-G04 — FIXED: furnace completion uses `==`.** `furnace.rs:123` completes on `cooking_timer == cooking_total`, matching vanilla 26.2 `serverTick` (`if_icmpne` at bytecode 274). A loaded total of 0 stalls in both (Rust test `a_loaded_total_of_zero_stalls` pins the stall). `test_furnace_loaded_total` is now a static pass.

**B4-G05 — FIXED: fuel remainder only when the stack empties.** `furnace.rs:108` places the crafting remainder only when `f.count == 0` after the decrement, matching vanilla `consumeFuel` (shrink, then remainder only if empty). Rust tests pin both arms (`the_last_fuel_leaves_its_remainder_in_the_slot`, `a_fuel_still_in_the_stack_loses_its_remainder`).

**B4-G16 — FIXED: cartography paper zoom works.** `menu.rs carto_changed` (`menu.rs:1294–1314`) gates the paper branch on `!locked && scale < 4` only. Vanilla 26.2 `CartographyTableMenu.setupResultSlot` has no `ItemTags` reference anywhere in the class (javap); `#extendable_maps` does not exist in the 26.2 jar data. Glass-pane lock and map copy (count 2) match vanilla, as does cartography quick-move.

### Withdrawn — vanilla 26.2 differs from 1.21.7

**B4-G03 — WITHDRAWN: vanilla 26.2 grows furnace output by the full result count.** 1.21.7's `burn()` grew by 1; 26.2's `burn()` calls `ItemStack.grow(result.getCount())` (javap) and `canBurn` allows `out.count + result.count <= min(containerMax, resultMax)`. The Rust (`furnace.rs:127`, `out.count += recipe.result.count`; `can_burn` at `furnace.rs:84–92`) matches 26.2 exactly.

**B4-G06 — WITHDRAWN: vanilla 26.2 does NOT reset the cooking timer when the input has no recipe.** 1.21.7's `serverTick` reset it; 26.2's jumps from the null-recipe check straight past the reset (`ifnull 333`, `333: goto 369`, javap). The Rust's missing `else` on the recipe lookup (`furnace.rs:78–137`) therefore matches 26.2. (The reset still happens for no-ingredient, `!canBurn`, and unlit-after-fuel-attempt — all three match.)

**B4-G07 — WITHDRAWN: vanilla 26.2 drops a spawn egg whose `entity_data` was removed.** 1.21.7's `SpawnEggItem.getType` fell back to `defaultType`; 26.2's returns `null` when `ENTITY_DATA` is absent (javap — no fallback), and `DispenserBlock.getDefaultDispenseMethod` routes to `SpawnEggItemBehavior` only when the stack `has(ENTITY_DATA)` (javap). The Rust (`dispenser.rs:710–716`) dispenses as a spawn egg unless the `!minecraft:entity_data` removal marker is present (the port's removal convention, `item_data.rs:1369–1373`), and drops otherwise — matching 26.2. (Residual corner, not a defect: an `entity_data` compound with no `id` — the Rust falls back to the item-named mob via `spawn_egg_on_spawner`, `block_entity_item.rs:77–89`; vanilla's behaviour there depends on component decode failure and was not established statically.)

**B4-G11 — WITHDRAWN: `spawner_blocks_work` EXISTS in vanilla 26.2 and is checked in both places.** The rule is in 26.2's game-rule registry (59 rules, `minecraft:spawner_blocks_work` present). Vanilla 26.2 `BaseSpawner.serverTick` returns early unless `isNearPlayer && level.isSpawnerBlockEnabled()` (javap; `ServerLevel.isSpawnerBlockEnabled` reads the rule), and `TrialSpawner.canSpawnInLevel` returns false first when the rule is off, then checks `overridePeacefulAndMobSpawnRule || (difficulty != PEACEFUL && spawn_mobs)` (javap). The port gates both spawner ticking (`game/spawner.rs:121`, `mobs/spawner.rs:451`) and trial-spawner spawning (`trial_spawner.rs:649–651`, which also checks `!peaceful && spawn_mobs`) on the rule — matching 26.2. The 1.21.7 "unexamined corner" (no `overridePeacefulAndMobSpawnRule` equivalent in the Rust) is resolved for 26.2: no class in the 26.2 server jar calls the setter, so the field is always false in vanilla gameplay.

**B4-G12 — WITHDRAWN: vanilla 26.2 ALLOWS creative enchanting without lapis.** 1.21.7's `clickMenuButton` returned false on an empty lapis slot even in creative; 26.2's bytecode is `if ((lapis.isEmpty() || lapis.getCount() < slot+1) && !player.hasInfiniteMaterials()) return false` — the empty check is inside the creative guard (javap, `EnchantmentMenu.clickMenuButton` offsets 56–81). The Rust (`game/stations.rs:1080`, `if lapis < lapis_needed && !self.creative`) matches. Survival behaviour matches (lapis count ≥ slot+1, level ≥ slot+1 and ≥ cost, deducts slot+1 levels).

**B4-G13 — WITHDRAWN by design: 26.2 has NO bed block-entity type.** The 26.2 registry has 49 BE types with `bed` out and `potent_sulfur` in (inventory-verified in `BlockEntityTypes` bytecode); the port's `for_block` correctly returns `None` for beds and has a `PotentSulfur` variant (`containers.rs:514`). No defect.

## Oracle defects

Measured statuses from `docs/sweep/26.2/batch-0-oracles.json` (fail on all three classpaths unless noted). Severities follow the 1.21.7 rule: major for shared dependencies and multi-driver blocks; minor otherwise.

| ID | Oracle | Errors | First error | Blocks |
|---|---|---|---|---|
| B4-O06 | VanillaDispenserOracle | 1 | `360: method does not override or implement a method from a supertype` (no-override) | test_dispenser, test_crafter (+ 4 drivers in other batches: block_removal, composter, mace, use_item_on) |
| B4-O08 | VanillaMenuOracle | 1 | `138: cannot find symbol` (missing-symbol) | test_menu_click + cascades to test_recipe_book, test_place_recipe, test_ghost_recipe |
| B4-O09 | VanillaRecipeBookOracle | cascade | depends on broken: VanillaMenuOracle | test_recipe_book |
| B4-O10 | VanillaPlaceRecipeOracle | cascade | depends on broken: VanillaMenuOracle | test_place_recipe |
| B4-O14 | VanillaSculkOracle | 4 | `327: cannot find symbol` (missing-symbol) | test_sculk |
| B4-O15 | VanillaBlockEntityAnimOracle | 1 | `102: World is not abstract and does not override abstract method fuelValues() in Level` (missing-override; Batch 0 G1) | test_block_entity_anim |
| B4-O16 | VanillaBlockEntityRenderOracle | 7 | `23: cannot find symbol` (missing-symbol) | test_block_entity_render (also blocked by VanillaTextDisplayRenderOracle, 1 error — Batch 10's item, do not double-count) |
| B4-O18 | VanillaRecipeBookScreenOracle | 4 | `4: package com.mojang.renderpearl.api.pipeline does not exist` (missing-package; Batch 0 G4 — 26.3-only API, needs a rewrite against 26.2's screen APIs) | test_recipe_book_render |
| B4-O25 | VanillaTradingOracle | 1 | `254: cannot find symbol` (missing-symbol) | test_trading |
| B4-O26 | VanillaGhostRecipeOracle | 1 | `56: cannot find symbol` (missing-symbol) | test_ghost_recipe (also needs VanillaMenuOracle) |

Severities in the JSON: major for B4-O06 (6 drivers sweep-wide) and B4-O08 (shared dependency, 4 drivers); minor for the rest. The cheapest repairs are the five 1-error oracles (O06, O08, O15, O25, O26); O08 is the most valuable single repair (unblocks 4 drivers, 2 of them cascades that pass for free).

No repairs attempted — oracle `.java` files live in `versions/26.2/mods/horizon/` and the brief forbids touching `versions/26.2/`; Batch 0 for 26.2 is measurement + triage only by coordinator decision.

## Oracle defects withdrawn (16)

These 1.21.7 oracle defects do not exist for 26.2 — the oracle compiles on all classpaths (or, for O17, on the client+both classpaths its driver uses):

B4-O01 VanillaBlockEntityTickerOracle, B4-O02 VanillaBlockEntityStorageOracle, B4-O03 VanillaFurnaceOracle, B4-O04 VanillaBrewingOracle, B4-O05 VanillaBrewingFuelOracle, B4-O07 VanillaEnchantingOracle, B4-O11 VanillaSimpleContainerOracle, B4-O12 VanillaItemStackStorageOracle, B4-O13 VanillaPalettedContainerOracle, B4-O17 VanillaContainerScreenOracle (pass on client+both; server fails on the client package only — expected, not breakage), B4-O19 VanillaEnchantDurabilityOracle (and its dependency VanillaBlockUpdateOracle also passes), B4-O20 VanillaStoredEnchantmentPatchOracle, B4-O21 VanillaVehicleContainerOracle, B4-O22 VanillaShulkerSpaceOracle, B4-O23 VanillaNaturalSpawnOracle, B4-O24 VanillaRecipeOracle.

## What passed (method-2 verifications)

These behaviours were verified against the 26.2 jar by bytecode + source + data and found correct. (Where the corresponding driver is oracle-blocked, the driver check itself is still a gap — the driver cannot run — but the behaviour is verified here.)

- **Furnace** (G01/G03–G06 all fixed or withdrawn): full `serverTick` verified above; `getTotalCookTime` (recipe time or 200) matches `set_input`; wet-sponge bucket fill matches `burn()`; furnace save fields are shorts in both jar (`putShort`) and port (`block_entity_nbt.rs`); smoker/blast-furnace burn-time halving verified.
- **Brewing** (G02 fixed): full tick, mixes, `canPlaceItem` (slot 3 reagent by `PotionBrewing.isIngredient`, slot 4 by tag, bottle slots by the four items + empty-slot rule), fuel-slot and quick-move routing, save fields (`BrewTime` short, `Fuel` byte).
- **Hopper** (except B4-G08): push cooldown, eject-then-suck order, hopper-to-hopper 8-tick skip + `>8` custom-cooldown guard, container-first suck + full-block-above blocking + `#does_not_block_hoppers` (tag exists in 26.2), SUCK_AABB exact (tick path), random entity-container pick, entity fallback. Rust unchanged from 1.21.7 (38-line diff is fuel/brewing only).
- **Dispenser/dropper** (G07 withdrawn): full registry, equipment fallback (vanilla `getDefaultDispenseMethod` order: equippable → `#sulfur_cube_swallowable` → spawn egg with `ENTITY_DATA` → default — the Rust matches, including the 26.2-new sulfur-cube arm), reservoir sampling, dropper insert/fail paths + entity fallback, empty-dispenser 1001 + BLOCK_ACTIVATE, potion mud conversion via the 26.2-spelled `#convertable_to_mud` tag (tag exists in 26.2 data; the 1.21.7 spelling `convertible_to_mud` does not), `spawnItem` constants exact.
- **Crafter** (except B4-G09/G10): 6-tick cycle, `dispenseFrom` events, `canPlaceItem`/`smallerStackExist`, redstone signal, one-by-one dispense, recipe condition now the singular `recipe_id` key (the 26.x plural `recipes` matched every craft — fixed in the 26.2 port).
- **Spawner** (G11 withdrawn): defaults, load/save (shorts, potentials, update tag without potentials), `delay()` re-roll + broadcastEvent(1), spawn position/yaw/max-nearby, `weighted_index` = `WeightedList.getRandom`, client spin, `spawner_blocks_work` gate. Spawn-egg-on-spawner drops entity data in 26.2 (`setEntityData` is 26.3-only) — the port removed the merge path.
- **Trial spawner** (G11 withdrawn): full state machine (Rust unchanged except the `overrideEntityData` removal); `canSpawnInLevel` matches including the `spawn_mobs` rule (26.2 renamed the rule from `do_mob_spawning`); `PlayerDetector` verified (block-position distance² < range² + creative/spectator filter + center→eye line of sight); `overridePeacefulAndMobSpawnRule` is dead in vanilla 26.2 (no caller).
- **Vault**: constants, state machine, `tryInsertKey` with `isSameItemSameComponents` key check, ejection one item per 20 ticks. `vault.rs` has 0 changed lines.
- **Anvil**: full `createResult` (material repair loop, same-item merge +12%, enchantment merge, book halving, count>1 → 40, rename +1, rename-only cap to 39 before the ≥40 too-expensive check, repair-cost doubling). Rust unchanged (`stations.rs` lib diff is test-only).
- **Grindstone**: merge durability +5%, curses kept, book→book, repair cost 2ⁿ−1, XP roll. Rust unchanged.
- **Enchanting** (G12 withdrawn): cost formula (`EnchantmentHelper.getEnchantmentCost` in the 26.2 menu), `selectEnchantment`, clue reseeding, book transmute, lapis/level checks, deducts slot+1 levels, seed re-roll. Rust unchanged.
- **Beacon**: 10-block/tick beam scan, `#beacon_base_blocks` layers (tag exists in 26.2), 80-tick effects, CONSTRUCT_BEACON box, menu payment validation. `game/beacon.rs` has 0 changed lines.
- **Merchant**: special prices verified against 26.2 bytecode — reputation in float (`Mth.floor((float)rep * multiplier)`), Hero of the Village in **double** (`0.3d + 0.0625d * amplifier`, `Math.floor`, min 1) — the port's f64 change matches 26.2 (the 1.21.7 f32 form matched 1.21.7); demand pricing, `satisfied_by`/`take` (including (b,a) fallback), restock demand update, level-up deferred 40 ticks via `updateMerchantTimer`/`increaseProfessionLevelOnUpdate` (verified in `Villager.rewardTradeXp` bytecode; the port's `level_up_pending` + `villager_level_up` matches), XP orbs (3 + nextInt(4), +5 on level-up), `MerchantResultSlot` has no `mayPickup` override in 26.2 (the port's `Role::MerchantResult => true` matches; the 1.21.7 satisfied-by gate was the 26.3 form), no `onCraftedBy` on take.
- **Menu engine** (fuel routing fixed): quick-move routes for all 25 menus — Rust unchanged except the verified fuel/brewing/cartography/merchant arms; click engine (drag, quick-move repeat loop, pickup, swap, clone, throw, pickup-all) matches `AbstractContainerMenu.doClick`; `pickup_inner`/`swap` edge arms match.
- **Loom / stonecutter / smithing**: loom selection + 6-layer cap, stonecutter recipe list + take, smithing first-match + optional ingredient matching. Rust unchanged.
- **Comparator**: `getRedstoneSignalFromContainer` formula (incl. crafter special-case), chiseled-bookshelf last-slot signal, lectern page signal. Rust unchanged (`redstone.rs` diff is comment-only).
- **Lectern**: `#lectern_books` tag exists in 26.2; place/read/page/signal all match. Rust unchanged.
- **Campfire / beehive / jukebox / sculk (save fields) / signs**: cook/cooldown ticks, hive release (600-tick minimum, nectar, hum), jukebox saved clock presence, sculk `last_vibration_frequency`/`warning_level`/listener/cursors round-trip, sign component preservation. Signs: 26.2 has no `allow_op_features` field (javap `SignBlockEntity`: only front/back text) and no `sign_text_front`/`sign_text_back`/`waxed` components (registry check) — the port drops them on pick-up, matching 26.2. `sculk.rs`/`campfire.rs` have 0 changed lines.
- **Storage**: chest load/pair (right-half-first double chest), container titles via the generic `block_names` map, ender-chest player inventory (single-player), shulker-box menu refusal, falling-chest BE creation + `TileEntityData` restore, decorated-pot manual use (insert/wobble/loot unpack; list-form sherds verified against 26.2's list-based `PotDecorations`), recipe lookup first-match in registration order (map-extending recipe excludes exploration maps — verified in `MapExtendingRecipe.matches`, which returns false on `isExplorationMap()` before the scale < 4 check), cooking-time codec defaults (smelting 200, smoking/blasting/campfire 100 — verified per-class in the jar), recipe-book multi-display handling (`SlotDisplay.AnyFuel` resolves via `FuelValues.fuelItems()`; the port groups by burn time), datapack enchantment loading, `/enchant` validation.
- **Block-entity coverage** (except B4-G15): 47 of 49 vanilla BE types have a port variant; `potent_sulfur` (26.2-new) is covered (`containers.rs:514`), as are `shelf`, `copper_golem_statue`, `creaking_heart`, `brushable_block`, `calibrated_sculk_sensor`. Beds correctly have none.

## Suspicions checked and refuted

- Trial-spawner/vault player detection is **not** box-vs-sphere wrong: vanilla `PlayerDetector` uses block-position distance² < range², exactly as the port does.
- Vault key check is **not** item-only: vanilla `isValidToInsert` uses `isSameItemSameComponents`, as the port does.
- Trial-spawner ejection order is **not** wrong: vanilla checks the 30-tick readiness gate *before* the empty-players check, as the port does.
- Merchant takes do **not** award the crafted statistic in vanilla (`MerchantResultSlot.onTake` has no `onCraftedBy`); the port matches.
- The brewing quick-move bottle arm is **not** broken by the missing `#brewing_potion_inputs` tag: `is_bottle` uses the prepared inputs set {potion, splash, lingering, glass bottle}, exactly vanilla 26.2's `PotionSlot` set.
- Anvil rename-cap order matches vanilla bytecode exactly (cap to 39 before the too-expensive check).
- The trial spawner's missing `overridePeacefulAndMobSpawnRule` is **not** a gap: no class in the 26.2 server jar calls the setter, so the field is always false in vanilla gameplay.
- The merchant's deferred level-up is **not** a stall bug: vanilla 26.2 `rewardTradeXp` sets a 40-tick timer and a flag; the level rises in the villager tick, as the port does.
- The furnace's missing timer reset on a recipe-less input is **not** a gap in 26.2: vanilla changed (see B4-G06).

## Checks

### Drivers — oracle passing (26, all pass, static)

`test_block_entity_tickers`, `test_block_entity_storage`, `test_furnace`, `test_brewing`, `test_brewing_fuel`, `test_enchanting`, `test_simple_container`, `test_item_stack_storage`, `test_paletted_container`, `test_container_screen` (client+both pass; server fails on the client package only), `test_enchant_durability`, `test_stored_enchantment_patch`, `test_vehicle_container`, `test_shulker_space`, `test_spawner`, `test_trial_spawner`, `test_vault`, `test_recipe`, `test_chest_load`, `test_chest_pair`, `test_container_title`, `test_enchantment_components`, `test_storage_menu`, `test_recipe_book_view`, `test_container_component_names` (VanillaCommandExecOracle, pass), `test_trade_components` (VanillaTradeComponentsOracle, pass). Each driver's oracle compiles on its path's classpath (Batch 0 table) and the behaviour it exercises was verified statically above. No driver was run — a run is still needed to confirm each yields a measurement (compiling ≠ running).

### Drivers — no oracle (20: 19 pass, 1 gap)

Pass (static): `test_furnace_output_components`, `test_jukebox_saved_clock`, `test_sign_literal`, `test_merchant_gate`, `test_merchant_crafted`, `test_crafter_drop_name`, `test_falling_chest`, `test_loot_mine_pot_components`, `test_pot_sherd_templates`, `test_recipe_book_variants`, `test_sculk_saved_fields`, `test_enchantment_map_validation`, `test_world_pack_enchantment_loader`, `test_anvil_turtle_egg`, `test_crafting_priority`, `test_machine_recipe_placement`, `test_craft_map_take`, `test_end_gateway`, `test_furnace_loaded_total` (G04 fixed — the zero-total case now stalls as vanilla 26.2 does).
Gap: `test_pot_hopper` (B4-G15).

### Drivers — oracle blocked (11, all gap)

`test_dispenser` (O06), `test_menu_click` (O08), `test_recipe_book` (O08+O09), `test_sculk` (O14), `test_block_entity_anim` (O15), `test_block_entity_render` (O16 + Batch 10's TextDisplayRenderOracle), `test_recipe_book_render` (O18), `test_place_recipe` (O08+O10), `test_trading` (O25), `test_ghost_recipe` (O26+O08), `test_crafter` (O06 — the 26.2 driver compiles and runs VanillaDispenserOracle, `test_crafter_parity.py:242–243`; in 1.21.7 it was no-oracle).

### Method-2 checks (34: 29 pass, 5 gap)

Pass: beacon, lectern, comparator, anvil, grindstone, smithing, loom, stonecutter, vault, chiseled bookshelf, jukebox, campfire, beehive, decorated-pot use, shulker box, merchant pricing, sculk save fields, sign NBT, falling-block BE restore, container titles/names, brewing recipes, furnace tick, brewing tick, spawner tick, trial spawner tick, enchanting offers/click, cartography, menu engine, dispenser/dropper.
Gap: hopper (B4-G08), ender chest (B4-G14), save round-trip BE (B4-G15), BE coverage (B4-G15), crafter (B4-G09/G10).

### Excluded from the tally

Other batches' scope: `test_bonus_chest`, `test_decorated_region` (worldgen); `test_storage_alias` (commands/Batch 10); `test_food_storage` (items/Batch 7); `test_server_player_sculk` (player/Batch 3); `test_shulker_bullet` (entity/Batch 3); `test_held_item_stack` (items/Batch 7); `test_finalize_spawn`, `test_natural_spawn` (entity/Batch 3); `test_screen_layout`, `test_text_display_render` (rendering/Batch 10 — VanillaLoomScreenOracle and VanillaTextDisplayRenderOracle are Batch 10's items, B10-O05 and B10-O03 context).
Unmeasured (no oracle exists; needs a live run): `test_disconnect_crafting` (guest disconnect drop path not located statically), `test_hive_default_patch` (beehive save prototype elision not read).

### Not verified

Sculk tick mechanics beyond save fields (oracle-blocked); command-block execution semantics (commands scope — the BE data model is complete); full jukebox song lifecycle (the driver disclaims it); recipe-book *server* packet construction (client side verified); trial-spawner `TIME_BETWEEN_EACH_EJECTION` constant value (30, from vanilla source — javap shows it non-constant-folded, same caveat as 1.21.7); the save_roundtrip harness failure itself (owned by the save harness — only its BE touchpoint, pot BEs via B4-G15, is recorded here). Sign `filtered_messages` are dropped on load (caveat, no defect ID). The comparator BE's saved `output` field has no port equivalent (the port computes comparator output from block state; no steady-state divergence established — a reload-transient corner, not a defect). Moving pistons are tracked in `world.moving_pistons`, not as a BE variant (Batch 2's scope).

## Notes for the coordinator

- There are **no major behaviour defects** in this batch for 26.2 — the three 1.21.7 majors are all fixed in the port, and the five remaining gaps are all minor carry-overs. The cheapest behaviour fixes are B4-G08 (one AABB) and B4-G09 (reuse the hopper's `entity_container_at` fallback in `crafter_eject`).
- The cheapest oracle repairs are the five 1-error oracles (B4-O06 dispenser, B4-O08 menu, B4-O15 anim, B4-O25 trading, B4-O26 ghost recipe). The most valuable single repair is B4-O08 (MenuOracle — unblocks 4 drivers, 2 of them cascades that pass for free), followed by B4-O06 (DispenserOracle — unblocks 6 drivers sweep-wide, 2 of them in this batch).
- B4-O09/O10 share their repair with B4-O08 (cascades); do not schedule them separately. B4-O15 shares its repair pattern with Batch 0 G1 (add a `fuelValues()` override to the fake level — the same one-line fix rescues 4 oracles). B4-O18 needs a rewrite against 26.2's screen APIs (renderpearl is 26.3-only), not an import fix.
- `test_block_entity_render` is additionally blocked by VanillaTextDisplayRenderOracle (1 error) and ParityFont (1 error) — both recorded by Batch 10; do not double-count.
- 11 of the 16 re-checked 1.21.7 behaviour defects were resolved without carry-over (5 fixed in the port, 6 withdrawn because vanilla 26.2 differs from 1.21.7) — the 26.2 port is substantially more faithful in this area than the 1.21.7 port was at sweep time. The withdrawn items are recorded in the report only, not in the JSON slice (which follows the 1.21.7 slice shape: gap + oracle-defect items only).
- Supporting evidence (javap dumps used for the verdicts) is under `evidence/` next to this report in the batch scratch dir.
