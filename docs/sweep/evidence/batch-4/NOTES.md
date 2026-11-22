# Batch 4 working notes (draft, uncommitted — execution-risk notice, Batch 2 precedent)
Method: javap -c on 1.21.7-server-named.jar + Rust source + prepared data. NO driver runs, NO cargo, NO git ops in the repo (execution-risk notice).
Jar: ~/workspace/mcir-work/leakage-audit/jars/1.21.7-server-named.jar
Data: ~/workspace/mcir-work/mob-spawn/resources/ (items.json etc.)

## Findings so far
- B4-G01 MAJOR: Furnaces (furnace/blast/smoker) can NEVER light. Item.fuel comes only from
  minecraft:cooking_fuel (horizon-items items.rs:1009-1011); 0/1416 items in 1.21.7 items.json
  carry it; no JSON in the tree defines it; no hardcoded fallback. Vanilla 1.21.7 has no fuel
  component at all (DataComponents javap: 0 fuel refs); burn times are the hardcoded
  FuelValues.vanillaBurnTimes table (lava bucket 20000, coal block 16000, blaze rod 2400,
  coal/charcoal 1600, #logs 300...). Consequences: Providers::burn_time=0 always ->
  Furnace::tick never lights; menu is_fuel false -> fuel slot refuses all fuel (only buckets
  pass via is_bucket); hopper fuel insertion likewise gated (verify). Same systemic cause as
  pilot G5 (26.x component/tag logic for things absent in 1.21.7).
- B4-G02 MAJOR: Brewing stands can NEVER brew. brewing_fuel() (horizon-client-render brewing.rs:423)
  reads minecraft:brewing_fuel component only; absent in 1.21.7. Vanilla uses #brewing_fuel
  item tag (blaze powder) + FUEL_USES=20 (BrewingStandBlockEntity.serverTick bytecode).
  Brew logic itself matches vanilla exactly (400 ticks, fuel-- on start, ingredient-identity
  reset, doBrew gating).
- B4-G03 minor (furnace, datapack-only): burn() grows output by recipe.result.count; vanilla
  burn() grows by exactly 1 when the output slot is non-empty (javap: ItemStack.grow(1)),
  and sets the full result copy only when the slot is empty. canBurn likewise: vanilla allows
  out.count < item max regardless of result count; Rust requires out.count + result.count <= max.
  Unreachable in vanilla data (all cooking recipes yield 1).
- B4-G04 minor (furnace, edge): completion uses >= (furnace.rs tick); vanilla uses == 
  (serverTick bytecode 284-292: if_icmpne skip). With cookingTotalTime=0 (loaded save) vanilla
  never completes; Rust completes in 1 tick. Rust test loaded_totals_survive_until_actual_completion
  pins the >= behaviour (total=0 -> completes tick 1), contradicting vanilla bytecode.
- B4-G05 minor (furnace, datapack-only): fuel remainder DROPPED when fuel stack remains
  (furnace.rs drops.push); vanilla only places the remainder when the fuel stack empties
  (serverTick bytecode 225-246: shrink, if empty set remainder), otherwise it is lost.
  Unreachable in vanilla data (only lava bucket has a remainder, max stack 1).
- B4-G06 minor (furnace, edge): lit furnace + input with NO recipe: vanilla resets cookingTimer
  to 0 (serverTick 341: !canBurn -> timer=0); Rust leaves the timer unchanged (no else on the
  recipe lookup). Reachable only via datapack reload mid-cook / NBT edit (setItem resets on change).
- HOPPER: PASS (pushItemsTick cooldown, tryMoveItems eject-then-suck, tryMoveInItem incl.
  hopper-to-hopper 8-skip cooldown + isOnCustomCooldown>8 guard, suckInItems container-first +
  full-block-above blocking + #does_not_block_hoppers (tag EXISTS in 1.21.7 jar), SUCK_AABB
  (0,11/16,0)-(1,2,1) exact, getRandom entity container pick, entityInside exists in vanilla).
  B4-G08 minor (timing-only): hoppers_catch_items (entityInside path) uses the narrow INSIDE
  bowl (0.125..0.875, 11/16..1) while vanilla entityInside uses the full SUCK_AABB
  (0..1, 11/16..2); the tick path covers the full AABB, so pickup can only be 1-2 ticks late
  for fast items at the edges, never missed.
- DISPENSER/DROPPER: PASS on table + dispatch. Vanilla registry (DispenseItemBehavior.bootStrap):
  60 items + 3 blocks + all spawn eggs (loop) + all shulker colours (loop) + boats + 6 minecarts;
  Rust covers all + equipment fallback via EQUIPPABLE (vanilla getDefaultDispenseMethod).
  Random-slot reservoir sampling matches getRandomSlot. Dropper insert/no-container/fail paths
  match. Empty-dispenser 1001 + BLOCK_ACTIVATE matches. Potion mud conversion matches.
  B4-G07 minor (commands-only corner): spawn egg with REMOVED minecraft:entity_data dispensed:
  vanilla SpawnEggItem.getType falls back to defaultType (spawns the default mob); the port
  drops the egg as an item (dispenser.rs dispense_default_method).
- CRAFTER: PASS on tick (6 ticks, flags 3 clear), dispenseFrom (1050, 6 ticks, flags 2 set,
  onCraftedBySystem, remainders before shrink, shrink-all, setChanged), canPlaceItem +
  smallerStackExist (exact), redstone signal (filled+disabled), dispenseItem one-by-one
  (crafter target / oversized), spawnItem 0.7/6, events 1049+2010 (facing 3D value; Direction
  enum order matches), advancement 17-box.
  B4-G09 minor: crafter_eject never inserts into container ENTITIES (chest/hopper minecart,
  chest boat): it only consults the block-only Containers::view, while vanilla dispenseItem
  uses HopperBlockEntity.getContainerAt (block + entity). Dropper/hopper have the entity
  fallback; crafter lacks it. Result drops as an item instead.
  B4-G10 minor (LAN-only): crafter_recipe_crafted trigger fires only for self.player; vanilla
  triggers for every ServerPlayer in the 17-box. LAN guests in range miss the trigger.

## Spawner (2026-11-22, resumed)
- Spawner defaults match vanilla BaseSpawner (delay 20, min 200, max 800, count 4, max nearby 6, range 16, spawn range 4). load/save: shorts, SpawnPotentials weight int, update_tag excludes SpawnPotentials. delay() re-rolls potential + broadcastEvent(1) — matches vanilla delay() bytecode (WeightedList.getRandom + ifPresent + broadcastEvent 1). weighted_index matches WeightedList.getRandom (nextInt(total), walk subtract, pick < 0). server_tick: near-player (center +0.5, hasNearbyAlivePlayer), delay -1/>0 handling, spawn pos (d-d)*range+0.5 / y+nextInt(3)-1, yaw nextFloat*360, max-nearby count in inflated box, delay after spawn — all match BaseSpawner.serverTick.
- **B4-G11 minor — spawner + trial spawner gated on non-vanilla `spawner_blocks_work` gamerule.** Vanilla 1.21.7 GameRules has NO such rule (javap of GameRules: no RULE_SPAWNER_*); vanilla BaseSpawner.serverTick checks NO gamerule at all, and TrialSpawner.canSpawnInLevel checks only `overridePeacefulAndMobSpawnRule || (difficulty != PEACEFUL && doMobSpawning)` (javap). The port's gamerule table (gamerules.rs:114) includes `spawner_blocks_work` defaulting true, and both `Spawner::server_tick` (via spawner_blocks_work()) and `TrialSpawner::can_spawn_in_level` gate on it. Default play unaffected (defaults true), but the gate + the rule itself are 26.x carry-over; setting it false stops spawners in a way vanilla 1.21.7 cannot express. Also the Rust trial spawner has no `overridePeacefulAndMobSpawnRule` equivalent (vanilla flag, set by TrialSpawnerBlockEntity paths — unexamined corner, not recorded as a defect).
- client_tick spin: 1000/(client_delay+200) % 360 matches vanilla clientTick form; client_event(1) sets client_delay = min_spawn_delay.

## Trial spawner — PASS (state machine verified against TrialSpawnerState.tickAndGetNext bytecode)
- States, transitions, order all match: Inactive (display entity → WaitingForPlayers), WaitingForPlayers (canSpawnInLevel → reset+stay; !hasMobToSpawn → Inactive; detect → Active), Active (countAdditionalPlayers BEFORE tryDetectPlayers; ominous item spawner; finished+all dead → cooldownEndsAt=now+targetCooldown, total=0, next=0 → WaitingForRewardEjection; isReadyToSpawnNextMob → spawnMob), WaitingForRewardEjection (isReadyToOpenShutter 40.0f + open_shutter sound), EjectingReward (isReadyToEjectItems gate FIRST, then empty → close_shutter + Cooldown + clear table; else pick table once, eject, remove iterator().next() — Rust's hash_order().first() + retain matches Set-iteration removal), Cooldown (detect → Active with total=0/next=0; isCooldownFinished → removeOminous + reset → WaitingForPlayers).
- Constants: MAX_MOB_TRACKING_DISTANCE=47 (TrialSpawner class, javap), DELAY_BEFORE_EJECT=40.0f (javap), TIME_BETWEEN_EACH_EJECTION=30 (Rust; vanilla-source value — javap -constants shows it non-constant-folded, value not directly extracted; behaviour 1 reward/30 ticks is the documented vanilla rate), DELAY_BETWEEN_PLAYER_SCANS=20 (javap), TRIAL_OMEN_PER_BAD_OMEN_LEVEL=18000 (javap; Rust TRIAL_OMEN_TICKS_PER_LEVEL, "15 minutes per level").
- PlayerDetector verified: NO_CREATIVE_PLAYERS predicate = player.blockPosition().closerThan(pos, range) && !creative && !spectator (javap lambda$static$0) — the Rust detect() matches exactly (block-pos distance^2 < range^2). Line of sight = block center → player eye. My earlier box-vs-sphere suspicion was WRONG; no divergence.
- countAdditionalPlayers = max(0, detected.size()-1) (javap) — Rust matches.
- Omen: Trial Omen preferred, else last Bad Omen; Bad Omen → Trial Omen 18000/level + sound; apply_ominous discards current mobs with flame particles (3012), resets totals, next spawn = now + ominous ticks_between_spawn.
- TrialConfig defaults match TrialSpawnerConfig.Builder (spawn_range 4, total 6, simultaneous 2, +2/+1 per player, 40 ticks between, cooldown 36000, player range 14).

## Vault — PASS
- Constants match: activation 4.0, deactivation 4.5, MAX_REWARD_PLAYERS 128, UNLOCKING_DELAY 14, INSERT_FAIL_SOUND_BUFFER 15, STATE_DELAY 20. server_tick: display cycle every 20 ticks when Active; state updates gated on state_updating_resumes_at; Inactive/Active update connected players (activation/deactivation range); Unlocking → Ejecting; Ejecting pops one item per state update (20 ticks), progress = inverseLerp, eject sound pitch 0.8+0.4*progress, shutter sounds on transitions.
- tryInsertKey verified against VaultBlockEntity$Server.tryInsertKey bytecode: canEjectReward → isValidToInsert → fail sound → hasRewardedPlayer → reject sound → resolveItemsToEject → empty → return → awardStat ITEM_USED → consume(key count) → unlock → addToRewardedPlayers → updateConnectedPlayers. isValidToInsert = isSameItemSameComponents && count >= key count (javap) — Rust same_item_same_components CORRECT (my item-only suspicion was wrong).
- update_connected_players: PlayerDetector INCLUDING_CREATIVE_PLAYERS (block-pos distance, excludes spectators + rewarded) — Rust matches.
- Caveat (not a defect ID): Rust use_item_on returns the key count taken; the caller consumes. Whether the ITEM_USED stat is awarded by the caller was not verified.

## Anvil — PASS (pending javap confirmation of rename-cap order)
- anvil_result matches AnvilMenu.createResult: material repair (min(damage, max/4) loop, +1/material, repair_item_count_cost), same-item merge (remaining1+remaining2+max*12/100, +2), enchant merge (same level +1 else max, compatibility +1 per conflict, book fee halved max 1, count>1 → 40, all-incompatible → fail), rename (+1, custom-name set/remove), cost = tax+price, rename-only caps at 39, >=40 too expensive unless creative, repair cost doubled+1 unless rename-only. increased_repair_cost = cost*2+1 (capped i32::MAX; vanilla uncapped, unreachable).
- TODO: javap AnvilMenu.createResult to confirm the rename-cap (39) is applied BEFORE the >=40 too-expensive check (Rust order). Vanilla-source memory says cap first; verify.
- anvil_used: 12% damage chance, level events 1029/1030 — matches AnvilBlock.damage.

## Grindstone — PASS
- grindstone_result matches GrindstoneMenu.computeResult: merge durability +5% bonus, non-damageable identical stacks → count 2, curses kept (mergeEnchantsFrom max), remove_non_curses (book → book, repair cost 2^n-1 via repeated increased_repair_cost). XP: min-cost sum of non-curse enchantments, roll ceil(half)+nextInt(half). grindstone_taken spawns orbs + level event 1042.

## Enchanting — PASS except B4-G12
- enchant_offers: costs via shared random seeded xp_seed (3 slots), cost < slot+1 → 0; clue reseeds seed+slot, select_of from #in_enchanting_table, book removes one at random, random clue pick — matches EnchantmentMenu.slotsChanged/getEnchantmentList.
- apply_enchantment: same list, book → enchanted book transmute.
- click_enchant: lapis needed = slot+1, level check (level >= slot+1 AND level >= cost), deducts slot+1 levels (NOT the cost — matches onEnchantmentPerformed), reseeds xp_seed, consumes lapis unless creative, sound pitch nextFloat*0.1+0.9, stat + trigger.
- **B4-G12 minor — creative enchanting without lapis allowed.** Vanilla EnchantmentMenu.clickMenuButton returns false when the lapis slot is EMPTY even in creative (`itemstack.isEmpty()` is outside the `!hasInfiniteMaterials` guard); the port's click_enchant skips the lapis-presence check entirely in creative. Survival behaviour matches.
- NOTE: enchantment_cost_of / select_of (Enchantments impl — cost formula + weighted selection) NOT yet located/read; the file defining them was not found in horizon-client/src (grep returned nothing). TODO: find Enchantments impl (likely horizon-client-render or horizon-world) and verify getEnchantmentCost + selectEnchantment formulas.

## Beacon — PASS
- tick_beacon: 10-block/tick beam scan, dampening>=15 (non-bedrock) clears beam, colour averaging; levels recompute every 80 ticks via #beacon_base_blocks complete layers (tag exists in 1.21.7); effects every 80 ticks when levels>0 + ambient sound; activate/deactivate sounds + CONSTRUCT_BEACON box AABB(x,y,z,x,y-4,z).inflate(10,5,10) verified.
- applyEffects: range levels*10+10, box inflated + up to world height, duration (9+levels*2)*20, amplifier 1 when levels>=4 && primary==secondary, secondary only levels>=4 && != primary, ambient=true. Matches BeaconBlockEntity.applyEffects.

## Brewing stand tick — PASS (logic); fuel broken per B4-G02
- tick_brewing_stand delegates to BrewingStand::tick (already verified vs vanilla serverTick: 400 ticks, fuel-- on start, ingredient-identity reset, doBrew gating, bottle bits).

## Menu engine (menu.rs) — PASS except fuel/brewing extensions of B4-G01/B4-G02
- quick_move routes verified vs vanilla per-menu quickMoveStack: Chest/Hopper/Shulker/Dispenser/Crafter standard; Brewing (fuel→4, reagent→3, bottle→0-2, else shuffle); Loom (result→inv, banner→0, dye→1, pattern→2); Stonecutter (result/in→inv, stonecutting input→0); Grindstone (full→shuffle else →0-2); Cartography (map→0, extra→1); Lectern moves nothing; Beacon (payment slot 0 only when empty && mayPlace && count==1); Enchanting (lapis→1, else ONE item into empty slot 0 via setByPlayer); Anvil/Smithing (ItemCombinerMenu; smithing into inputs only if an empty slot accepts); Merchant (result→inv + remove count + achievements + trade sound); Furnace (result→inv, smeltable→0, fuel→1, else shuffle); Crafting (result→inv, inv→grid 1-10, grid→inv); Inventory (armor/offhand routing).
- **B4-G01 extends: furnace quick-move fuel routing never fires** — menu.rs is_fuel uses `i.fuel.is_some()` (the absent component). Shift-clicking coal into a furnace shuffles inventory instead of filling the fuel slot.
- **B4-G02 extends: brewing quick-move fuel routing never fires** — brewing_fuel() component check; blaze powder can't be shift-clicked into the fuel slot.
- click(): QuickCraft drag (status 0/1/2, type from button>>2, type 2 creative-only, single-slot → Pickup, even/one/max distribution), QuickMove repeat-while-same-item loop, Pickup -999 drop, Swap (0-8 + 40), Clone creative max-stack (+ CartoResult onCraftedBy), Throw (1/all + repeat loop), PickupAll — all match AbstractContainerMenu.doClick. pickup_inner: safe_insert, half-take div_ceil(2), may_place-false merge-back arm, swap overflow → inventory_add → drop — matches.
- may_pickup checked before QuickMove ✓.

## BE coverage (47 types) — from BlockEntity enum + for_block
- Covered as variants: banner, beacon, beehive, bell, blast_furnace/furnace/smoker (Furnace kind), brewing_stand, brushable_block, calibrated_sculk_sensor/sculk_catalyst/sculk_sensor/sculk_shrieker (Sculk), campfire, chest/trapped_chest/barrel/hopper/dispenser/dropper/shulker_box/ender_chest (Chest fallback via container_size: 27/27/27/5/9/9/27/27), chiseled_bookshelf, command_block, conduit, crafter, creaking_heart, daylight_detector, enchanting_table, end_gateway, end_portal, hanging_sign/sign (Sign), jigsaw, jukebox, lectern, mob_spawner, skull, structure_block, test_block, test_instance_block, trial_spawner, vault.
- **B4-G13 minor — no Bed block entity.** for_block returns None for beds (not in container_size). Vanilla beds have BedBlockEntity (colour + custom name). Consequences: renamed beds lose their custom name on placement; `/data get block` on a bed returns nothing; chunk saves carry no bed BEs — explains the world-compare gap (bed blocks 50=50, bed BEs 0 vs 50). Colour itself is safe (per-block in 1.21.7).
- **B4-G14 minor (LAN-only) — ender chest shows the wrong inventory to LAN guests.** open_ender_chest swaps the HOST's ender_items into the shared block BE; close copies back and leaves an empty Chest(27). A guest opening the same chest sees the host's items (while host has it open) or an empty chest — never their own EnderItems. Host single-player behaviour is correct (player-specific inventory, block holds none).
- Decorated pot: NO BlockEntity variant, but the item + sherds persist as a raw saved tag via spill::saved_entity (pot_data/set_pot_data/pot_tag); use_decorated_pot matches vanilla insert/wobble (insert_fail sound + WOBBLE_NEGATIVE, insert + WOBBLE_POSITIVE, consumeAndReturn 1, pitch by fill).
- **B4-G15 minor — hoppers/droppers cannot see decorated pots.** Containers::view requires a BlockEntity in world.block_entities; pots have none, so view returns None. Vanilla HopperBlockEntity.getContainerAt returns the pot (1-slot container). Hopper insert/extract and dropper insert into pots silently fail; comparator reading a pot (same view) also reads 0. Manual insert via use_decorated_pot works.
- Comparator BE: no variant (comparator signal is derived at read time; vanilla ComparatorBlockEntity stores outputSignal — save-format divergence only, no functional check made). Piston: handled by moving_pistons (separate system, not a BE variant).

## Enchanting — full PASS (enchant.rs verified)
- getEnchantmentCost exact: enchantable 0 → 0; power min 15; j = nextInt(8)+1+(power>>1)+nextInt(power+1); slot 0 max(j/3,1), slot 1 j*2/3+1, slot 2 max(j, power*2).
- selectEnchantment exact: cost+1+nextInt(v/4+1)x2, ±15% float span, round, min 1; available_results = highest level in [min_cost, max_cost] per enchantment, book → all in pool else primary only; weighted pick; loop while nextInt(50) <= cost, retain compatible-with-last, cost /= 2.
- compatible() false for identical ids (matches vanilla isCompatibleWith's this != other). can_enchant = supported items. is_primary = supported AND (no primary set OR in primary) — vanilla checks primary-only when present; primary ⊆ supported in vanilla data, so unreachable difference.
- read_levels: levels kept only in 1..=255, partial map kept on entry failure (ItemEnchantments codec). PENDING javap check: is vanilla's codec intRange(1,255) or POSITIVE_INT (no max)? If the latter, 256 kept in vanilla = gap. See javap-enchant-codec.txt.
- cmd_enchant: level > max → error; incompatible → error; level.max(0) applied. (Driver test_enchantment_map_validation is about the map codec, not /enchant.)

## Comparator — PASS (container overlap)
- machines.rs signal(): crafter special-case (filled+disabled count); else sum(count/max_stack)/slots, floor(min(total,1)*14)+(total>0) = Mth.lerpDiscrete. Chiseled bookshelf: lastInteractedSlot+1. Composter excluded (own analog output). Note: Rust uses the item's max stack, vanilla min(container max, item max) — all vanilla block containers are 64, equivalent in vanilla data. B4-G15 corollary: pots read 0 (same view).

## Merchant — PASS
- update_special_prices: reputation floor(rep * multiplier), hero 0.3+0.0625*amp, reduction floor(hero * baseCount) min 1 — matches MerchantOffer.updateSpecialPriceDiff. cost_a_count: base + max(0, floor(base*demand*multiplier)) + special, clamped [1, maxStack]. update_demand: demand + uses - (maxUses - uses). satisfied_by (incl. B-absent requires empty slot). take consumes A then B. merchant_take tries (a,b) then (b,a). notify_trade: uses+1, xp, level-up (new offers + special prices), orb pop 3+nextInt(4) (+5 on level-up).
- MerchantResultSlot.onTake javap: NO onCraftedBy — vanilla awards only TRADED_WITH_VILLAGER + notifyTrade. The port matches (no crafted-stat push in the merchant arm). Earlier B4-G17 suspicion REFUTED — no defect.
- test_merchant_gate: reputation deferred via villager_trade_pending (the villager owns the pending event until its AI step) — matches the driver's premise. PASS (static).
- test_merchant_crafted: result takes + crafted statistics — vanilla onTake awards TRADED_WITH_VILLAGER (port does, in merchant_traded) and no ITEM_CRAFTED. PASS (static).

## Loom / stonecutter / cartography
- Loom: PASS — slotsChanged matches (no inputs → clear; single pattern auto-select; selection preserved by value; max 6 layers → no result).
- Stonecutter: PASS — setupRecipeList on input change, selected -1; setupResultSlot by index; onTake consumes 1 input + take sound.
- Cartography: glass-pane lock + map copy (count 2) match; paper zoom PENDING javap (see B4-G16 below).

## B4-G16 (PENDING javap confirmation) — cartography paper zoom-out never works
- menu.rs carto_changed gates paper zoom on `#extendable_maps` item tag. That tag does NOT exist in the 1.21.7 jar data (unzip -l: no data/minecraft/tags/item/extendable_maps.json; also no brewing_potion_inputs.json). Same systemic cause as B4-G01/G02 (26.x tag absent in 1.21.7). Vanilla 1.21.7 CartographyTableMenu.setupResultSlot (private method — slotsChanged delegates to it) needs javap to confirm what the paper check uses instead (likely MapItem instanceof + scale < 4, no tag). If confirmed: MAJOR (a whole cartography function dead). Also verified present in jar: brewing_fuel (blaze_powder) — confirms B4-G02 fix direction; beacon_base_blocks + does_not_block_hoppers (block tags) — beacon/hopper passes stand.
- Brewing quick-move bottle arm is SAFE despite the missing #brewing_potion_inputs tag: is_bottle uses the prepared `inputs` set (potion/splash/lingering/glass_bottle).

## Signs — PASS
- SignData preserves the original component verbatim while text is unchanged (saved[i]); edited lines become literals in the old root style (matches setMessages). front_text/back_text NBT, color, glowing, waxed, allow_op_features. test_sign_literal: PASS (static). Caveat (not a defect): no filtered_messages support — filtered text is dropped on load.

## Sculk / jukebox / campfire / beehive
- Sculk saved fields: last_vibration_frequency (sensor/calibrated), warning_level (shrieker), listener, cursors (catalyst) — all saved + loaded. test_sculk_saved_fields: PASS (static). Sculk tick structure mirrors vanilla (BE-add order, chunk-ticking gates, idle skip); deep vibration/catalyst/warden mechanics NOT verified — driver oracle-blocked (B4-O14).
- Jukebox: struct has ticks + song_clock_present + playing; clock written only when a song reference exists. test_jukebox_saved_clock: PASS (static).
- Campfire: PASS — place_food (first empty slot, count 1, total = recipe time), cook_tick (+1, done at >= total, result or item itself dropped), cooldown_tick (-2, clamped [0, total] with negative-time guard).
- Beehive: PASS — fire in 6 neighbours → emergency release; server_tick (ticks_in_hive++ then release when > min 600, HoneyDelivered vs BeeReleased by nectar, setChanged, 0.005 hum). MAX_OCCUPANTS 3.

## Falling blocks / containers
- Falling chest: finish_falling_block creates a fresh BE via new_block_entity_for_state, then overlays carried TileEntityData fields and reloads — matches vanilla (BE created + inventory retained). test_falling_chest: PASS (static).
- Anvil on turtle eggs: drop at block.position (where it came to rest, on top of the eggs) — the defect described in the driver docstring (dropped inside the cell, pushed 2.5 blocks) appears fixed. test_anvil_turtle_egg: PASS (static). Scope note: falling-block behaviour, Batch 1-adjacent; counted here (no other batch claimed it).
- Block names: world.block_names map holds CustomName for all BEs except Banner/Sign (own fields); load_block_name + saved_block_entity round-trip it; container titles read it (double chest: right half first). test_container_component_names: PASS (static). test_crafter_drop_name: PASS (static) — name survives via saved tag → loot copy_name path.

## Recipe book / crafting
- Recipe lookup: first-match in vanilla registration order (crafting.rs:1321 sort key). test_crafting_priority: PASS (static).
- PlaceRecipe machinery exists (menu/place.rs: StackedContents-like accounting, try_pick, place_grid). test_machine_recipe_placement: PASS (static, machinery present; packet-level comparison needs a run). test_place_recipe: oracle-blocked (B4-O10).
- test_craft_map_take: PASS (static) — map_crafted verified (post-processing 1 = scale up, 0 = lock, derive new id).
- test_recipe_book_variants: server sends one packet entry per display; client display_ids(id) exists (recipe_book.rs:1170) and sync_client_book/toast iterate per display; transmute material bounds (map_cloning 1..8 displays) in crafting.rs. PASS (static) — server-side packet construction not directly read; the client-side multi-display handling is present.
- test_recipe_book_view: PASS (static) — oracle passes on client+both (server fails on client package); view/filter logic in game/recipe_book.rs.
- test_recipe: oracle-blocked (B4-O24, VanillaRecipeOracle 10 errs, assemble signature).
- test_ghost_recipe: oracle-blocked (B4-O26, VanillaGhostRecipeOracle 16 errs, client recipebook package).
- test_disconnect_crafting: NOT fully verified — remove_guest closes native menus + logs reason; the Game-side guest-left menu/cursor drop path was not located. Verdict: gap? NO — record as UNMEASURED in the report; ledger: count as gap with a note? Decision: include as a check with status gap and evidence "disconnect drop path not located statically; needs a run" — hmm, that inflates gap without a defect. Better: exclude from the tally as unmeasured (needs a live run; no oracle exists) and list it in out_of_scope/unmeasured. DECIDED: exclude, note in report.
- test_hive_default_patch: beehive save elision — NOT verified (save::encode_block_entity prototype elision not read). Same treatment: exclude as unmeasured, note in report.
- test_end_gateway: PASS (static) — exit_portal + exact_teleport wired (game/end.rs), geometry fns (swept/pearl/bottom_center) present.
- test_trade_components: PASS (static) — pricing/cost/payment methods verified (see Merchant).

## Driver triage verdicts (no-oracle)
- PASS (static): test_crafter, test_furnace_output_components, test_jukebox_saved_clock, test_sign_literal, test_merchant_gate, test_merchant_crafted, test_crafter_drop_name, test_falling_chest, test_loot_mine_pot_components, test_pot_sherd_templates, test_recipe_book_variants, test_sculk_saved_fields, test_enchantment_map_validation (pending codec javap), test_world_pack_enchantment_loader, test_anvil_turtle_egg, test_crafting_priority, test_machine_recipe_placement, test_craft_map_take, test_container_component_names, test_end_gateway, test_trade_components.
- GAP: test_pot_hopper (B4-G15), test_furnace_loaded_total (B4-G04 — 'zero' totals complete in 1 tick via >= vs vanilla ==).
- EXCLUDED from tally (other batches' scope, noted in report): test_bonus_chest (worldgen feature), test_decorated_region (worldgen decoration order), test_storage_alias (commands/Batch 10), test_anvil_turtle_egg counted (see above), test_food_storage (items/Batch 7), test_server_player_sculk (player/Batch 3), test_shulker_bullet (entity/Batch 3), test_held_item_stack (items/Batch 7).
- UNMEASURED (excluded from tally, needs a live run, no oracle): test_disconnect_crafting, test_hive_default_patch.

## Driver triage verdicts (oracle-passing — static pass, driver not run)
- test_chest_load, test_chest_pair, test_container_title, test_enchantment_components, test_storage_menu, test_recipe_book_view: PASS (static). (test_food_storage excluded as items-scope.)

## Oracle-blocked drivers → B4-Oxx (one item per oracle, Batch 2 precedent)
- B4-O01 Ticker (8, util.debug) → test_block_entity_tickers
- B4-O02 BEStorage (4) → test_block_entity_storage
- B4-O03 Furnace (1) → test_furnace
- B4-O04 Brewing (14) → test_brewing
- B4-O05 BrewingFuel (8) → test_brewing_fuel
- B4-O06 Dispenser (54, gamerules pkg) → test_dispenser
- B4-O07 Enchanting (4) → test_enchanting
- B4-O08 Menu (9) → test_menu_click (+ cascades)
- B4-O09 RecipeBook (cascade Menu) → test_recipe_book
- B4-O10 PlaceRecipe (cascade Menu) → test_place_recipe
- B4-O11 SimpleContainer (1) → test_simple_container
- B4-O12 ItemStackStorage (1) → test_item_stack_storage
- B4-O13 PalettedContainer (4) → test_paletted_container
- B4-O14 Sculk (60, gizmos) → test_sculk
- B4-O15 BEAnim (45, client model) → test_block_entity_anim
- B4-O16 BERender (99, blaze3d) → test_block_entity_render
- B4-O17 ContainerScreen (54, client) → test_container_screen
- B4-O18 RecipeBookScreen (112, renderpearl) → test_recipe_book_render
- B4-O19 EnchantDurability (cascade VanillaBlockUpdateOracle = Batch 2 O1) → test_enchant_durability
- B4-O20 StoredEnchantmentPatch (1) → test_stored_enchantment_patch
- B4-O21 VehicleContainer (5) → test_vehicle_container
- B4-O22 ShulkerSpace (2) → test_shulker_space
- B4-O23 NaturalSpawn (43, gamerules pkg) → test_spawner, test_trial_spawner, test_vault
- B4-O24 Recipe (10, assemble sig) → test_recipe
- B4-O25 Trading (94, gizmos) → test_trading
- B4-O26 GhostRecipe (16, client recipebook) → test_ghost_recipe

## Final confirmations (2026-11-22, end of session)
- B4-G16 CONFIRMED MAJOR: vanilla CartographyTableMenu.setupResultSlot lambda checks paper with NO tag — `extra.is(PAPER) && !locked && scale < 4` (javap, no ItemTags reference in the whole class). The port's extra `#extendable_maps` gate (tag absent in 1.21.7) kills paper zoom-out entirely. Glass-pane lock + map copy (count 2) match vanilla; cartography quick-move verified against vanilla quickMoveStack bytecode (result 2 → 3..39 reverse with onCraftedBy; MAP_ID → 0; paper/map/glass_pane → 1; shuffle 3..30 ↔ 30..39).
- Anvil order CONFIRMED by bytecode: cost = clamp(tax+price) → set; price<=0 → EMPTY; rename-only && cost>=40 → 39 + onlyRenaming; cost>=40 && !creative → EMPTY; repair base = max(result, addition), doubled+1 unless rename-only. Rust matches exactly.
- Enchantment LEVEL_CODEC = Codec.intRange(1, 255) (javap <clinit>: iconst_1, sipush 255). Rust read_levels (1..=255, partial map kept) matches. test_enchantment_map_validation: PASS.
- MerchantResultSlot.onTake javap: NO onCraftedBy — vanilla awards only TRADED_WITH_VILLAGER + notifyTrade. Port matches. (No B4-G17.)
- Lectern: PASS — #lectern_books EXISTS in 1.21.7 (written + writable book); useItemOn, page buttons, signalPageChange, lectern_signal = floor(page/(pages-1)*14) + hasBook.
- Beacon menu: PASS — beacon_update validation matches (payment present, secondary needs levels>=4, primary tier <4, secondary tier 4 unless same-as-primary).
- Smithing: PASS (static) — first-match over smithing list, optional template/addition matching, transform + trim.
- Chiseled bookshelf: PASS (static) — use place/take, last_slot tracking, comparator signal lastSlot+1; #bookshelf_books tag exists in jar.
- Command blocks: BE data model complete (command, success_count, custom_name, track_output, last_output, last_execution, powered, auto, condition_met); execution semantics NOT verified (commands scope, Batch 10).
- Recipe book variants: PASS (static) — client display_ids + per-display sync/toast; transmute bounds.
- Sculk tick mechanics NOT deep-verified (oracle-blocked); save fields pass.
- test_disconnect_crafting + test_hive_default_patch: UNMEASURED, excluded from tally (need live runs; no oracle; disconnect drop path not located statically, hive save elision not read).

## TALLY (final)
- Drivers: 57 (6 oracle-pass → 6 pass; 23 no-oracle → 21 pass + 2 gap; 28 oracle-blocked → 28 gap).
- Method-2 checks: 34 (21 pass, 13 gap).
- TOTAL: 91 checks, 48 pass, 43 gap. 28 of the 43 gaps are "driver cannot run (oracle broken)", not behaviour defects.
- Defects: 16 G (3 major: G01, G02, G16; 13 minor) + 26 O (10 major, 16 minor) = 42 items.
