# Batch 7 — Items (Minecraft 26.1.1)

**Scope:** item use, food, tools, armour, buckets, spawn eggs, components — 1,506 items.

**Result: 44 checks — 28 pass, 16 gap** (as-tallied 44 = 44 behaviour + 0 non-behaviour). 12 gap items (4 major, 8 minor), 5 fixed, 2 closed, 30 oracle-defect items. The headline: every 26.1 defect reproduces in 26.1.1 except the peaceful gate (G04, genuinely fixed) and G13/G14/G15 (fixed only because the whole transform path is dead), and the 26.1.1 tree adds **four new defects** — G07 and G08 are 1.21.7 defects the 26.1 tree had fixed, returning verbatim as one-method reversions to the 26.2/26.3 form.

## §0 Execution status

- **Checks: 44 total, 28 pass, 16 gap — all behaviour, 0 non-behaviour.** Oracle triage is not tallied (it is not a driver check).
- **Items: 49** — 12 gap (4 major, 8 minor), 5 fixed, 2 closed, 30 oracle-defect (2 major, 28 minor).
- **No drivers were run and no oracle repairs were done** (checks + triage only, per the brief). No UNVERIFIED carry-overs: every mandatory re-check (12 from 26.1, 12 from 1.21.7, 4 from 26.2) was decided fresh against 26.1.1's jar and tree, and both coordinator-flagged Batch 3 findings were verified fresh.
- **Tree diff vs 26.1:** 389 files differ between `versions/26.1/crates` and `versions/26.1.1/crates` tree-wide; every in-scope file was individually diffed (see `evidence/diff-*.txt`).
- **Vanilla diff vs 26.1:** a sha1 byte-compare of 66 in-scope classes between the 26.1 and 26.1.1 jars found **0 diffs** — vanilla changed nothing in scope. Every verdict change vs 26.1 is port-side. (Each item was still re-verified fresh against the 26.1.1 jar; the byte-identity is a cross-check, not the proof.)
- **Worktree state:** the 26.1.1 tree's `.local` (Batch 0's prepare) is present; jar sha1s match the brief pins.
- **B9 (peaceful gate) is counted as PASS** in the tally: the gate is fixed in 26.1.1 (B7-G04). The hoglin residual (hoglin wrongly in the shared `allowed_in_peaceful` list) is recorded as **B3-G04** and lives in Batch 3's tally, not this batch's.

## §1 Item → class map

Same extraction as 26.1, re-run against the 26.1.1 jar (`evidence/parse_items_class_26_1_1.py`, output `evidence/parse_items_class_26_1_1.out`): 1,506 items mapped to 70 classes, 0 unmapped, 0 mismatches vs `block-behaviours.json` — identical to 26.1. 26.1.1 has spears (7) and nautilus armour (5), so B33/B34 are full checks, not not-covered.

| Class (with items that use it) | Count |
|---|---|
| `net.minecraft.world.item.ArmorItem` | 62 |
| `net.minecraft.world.item.AxeItem` | 6 |
| `net.minecraft.world.item.BlockItem` | 1174 |
| `net.minecraft.world.item.BoatItem` | 9 |
| `net.minecraft.world.item.BookItem` | 5 |
| `net.minecraft.world.item.BottleItem` | 1 |
| `net.minecraft.world.item.BowItem` | 1 |
| `net.minecraft.world.item.BucketItem` | 19 |
| `net.minecraft.world.item.BundleItem` | 17 |
| `net.minecraft.world.item.ChorusFruitItem` | 1 |
| `net.minecraft.world.item.CompassItem` | 4 |
| `net.minecraft.world.item.CrossbowItem` | 1 |
| `net.minecraft.world.item.DyeItem` | 16 |
| `net.minecraft.world.item.EggItem` | 2 |
| `net.minecraft.world.item.EnchantedBookItem` | 1 |
| `net.minecraft.world.item.EnderEyeItem` | 1 |
| `net.minecraft.world.item.EnderpearlItem` | 1 |
| `net.minecraft.world.item.FishingRodItem` | 1 |
| `net.minecraft.world.item.FlintAndSteelItem` | 1 |
| `net.minecraft.world.item.HoeItem` | 7 |
| `net.minecraft.world.item.HoneyBottleItem` | 1 |
| `net.minecraft.world.item.HoneycombItem` | 1 |
| `net.minecraft.world.item.Item` | 56 |
| `net.minecraft.world.item.ItemFrameItem` | 2 |
| `net.minecraft.world.item.ItemNameBlockItem` | 8 |
| `net.minecraft.world.item.KnowledgeBookItem` | 1 |
| `net.minecraft.world.item.LingeringPotionItem` | 1 |
| `net.minecraft.world.item.MaceItem` | 1 |
| `net.minecraft.world.item.MapItem` | 3 |
| `net.minecraft.world.item.MilkBucketItem` | 1 |
| `net.minecraft.world.item.MinecartItem` | 4 |
| `net.minecraft.world.item.PotionItem` | 1 |
| `net.minecraft.world.item.SaddleItem` | 1 |
| `net.minecraft.world.item.ShearsItem` | 1 |
| `net.minecraft.world.item.ShieldItem` | 1 |
| `net.minecraft.world.item.ShovelItem` | 6 |
| `net.minecraft.world.item.SignItem` | 10 |
| `net.minecraft.world.item.SnowballItem` | 1 |
| `net.minecraft.world.item.SpearItem` | 7 |
| `net.minecraft.world.item.SpawnEggItem` | 68 |
| `net.minecraft.world.item.SplashPotionItem` | 1 |
| `net.minecraft.world.item.StandingAndWallBlockItem` | 4 |
| `net.minecraft.world.item.SuspiciousStewItem` | 1 |
| `net.minecraft.world.item.SwordItem` | 6 |
| `net.minecraft.world.item.ThrowablePotionItem` | 1 |
| `net.minecraft.world.item.TippedArrowItem` | 1 |
| `net.minecraft.world.item.TridentItem` | 1 |
| `net.minecraft.world.item.WindChargeItem` | 1 |
| `net.minecraft.world.item.WritableBookItem` | 1 |
| `net.minecraft.world.item.WrittenBookItem` | 1 |
| `net.minecraft.world.item.crafting.CraftingBookCategory` | 1 |
| `net.minecraft.world.item.component.UseEffects` | 1 |
| `net.minecraft.world.item.component.UseRemainder` | 1 |
| `net.minecraft.world.item.enchantment.EnchantmentEffectComponents` | 1 |
| `net.minecraft.world.item.enchantment.Enchantments` | 1 |
| `net.minecraft.world.item.enchantment.ItemEnchantments` | 1 |
| `net.minecraft.world.item.enchantment.providers.SingleEnchantment` | 1 |

## §2 Data checks

All 8 pass, identical counts to 26.1 (`evidence/data_checks_26_1_1_v2.py`, output `evidence/data_checks_26_1_1_v2.out`). ToolMaterial/ArmorMaterials are byte-identical between the jars, so `materials-26.1.1.json` is the same data as 26.1's (copied, not re-extracted from a different source).

- **D1 (food):** 44 foods in `items.json`, all with the expected food components (checked against the jar's FoodProperties in `ItemProperties`/`Items`). **PASS.**
- **D2 (durability):** 84 items have a `max_damage` component; values match the jar's `Item.MAX_DAMAGE`/`ItemProperties.durability` for every item. **PASS.**
- **D3 (max stack size):** 1,506 items — every `max_stack_size` matches the jar. **PASS.**
- **D4 (tool/spear pairs):** 42 tool/spear pairs (7 materials × 6 tool kinds + spear variants) with matching `tool`/`attack_damage`/`attack_speed` components. **PASS.**
- **D5 (tool components):** 35 tool components, matching the jar's `Tool` definitions (rules, damage per block, mining speed). **PASS.**
- **D6 (tags):** 13 item tags checked against the jar's ItemTags — all present and correct. **PASS.**
- **D7 (armour):** 41 armour items; material, defence, toughness, knockback resistance and equip sound match the jar's ArmorMaterials. **PASS.**
- **D8 (item→class map):** 1,506 items → 70 classes, 0 unmapped, 0 mismatches vs `block-behaviours.json` (see §1). **PASS.**

Prepared-data spot checks (26.1.1 `.local`): `items.json` has 1,506 items; **0 carry `minecraft:block_transformer`** (the component the transform path reads — see G07) and **0 carry `minecraft:mob_visibility`** (see §8). `block-tags.json` has 248 tags; `#entities_can_teleport_to`, `#blocks_motion`, `#enderman_does_not_teleport_to`, `#consumable_does_not_teleport_to` are all ABSENT; `#convertable_to_mud` is present. Item tags: 187 files; `shovels.json` exists; `douses_campfires.json` is absent.

## §3 Behaviour checks

### §3.1 Component map (B1) — GAP

**B1 (component map set/remove):** vanilla 26.1.1 `PatchedDataComponentMap.set` returns `previous.orElse(prototype)` — null on a dangling removal marker, never an error (class byte-identical to 26.1's; javap-verified fresh). The Rust `ComponentMap::set` (horizon-world/src/component_patch.rs:106-119, line 116 — file byte-identical to the 26.1 tree's) returns `Err(MissingPrototype)` on a dangling removal marker; the doc comment above it is still wrong. **GAP — B7-G01 (minor, reproduces from 26.1).**

### §3.2 Spawn eggs (B6, B8, B9, B10, B11, B35) — 1 pass, 5 gap

- **B6 (spawn egg on spawner):** vanilla `SpawnEggItem.useOn` calls only `Spawner.setEntityId` — the 26.1.1 jar has NO `Spawner.setEntityData` (`BaseSpawner` has only `setEntityId` overloads; `TrialSpawnerBlockEntity` has only `setEntityId`, javap-verified fresh). The Rust applies an op's egg `entity_data` to spawners (game/spawner.rs:464 `set_entity_data` for regular spawners; :450 `override_entity_data` for trial spawners — the trial arm is NEW in 26.1.1). **GAP — B7-G17 (minor, reproduces from 26.1; duplicate of B3-G39 — coordinator to fold).** Note: B3-G39's evidence says the regular arm was "missed by the 26.1 sweep" — it was not; 26.1 B7-G17 is exactly that arm.
- **B8 (spawn egg entity_data id override):** vanilla `SpawnEggItem.getType` reads the typed `entity_data` component's `type()` and spawns THAT type (javap-verified fresh: `getType` reads `DataComponents.ENTITY_DATA`, `TypedEntityData.type()`). The Rust derives the kind from the item name only (game/creatures.rs:7531 `use_spawn_egg`, `trim_end_matches("_spawn_egg")`) and ignores an `entity_data` id override naming a different type. **GAP — B7-G03 (minor, reproduces from 26.1).**
- **B9 (spawn egg peaceful gate):** vanilla `SpawnEggItem.spawnMob` FAILs hostile eggs in peaceful (`!isAllowedInPeaceful && difficulty == PEACEFUL`, javap-verified fresh). The 26.1.1 tree ADDED the gate the 26.1 tree lacked: game/creatures.rs:7511-7520 `spawn_egg_allowed` plus the `use_failed` return at :7532-7535, with FAIL semantics (`use_failed` means the action is not consumed, game/lan_actions.rs:2562-2565). The 26.1 tree's false comment ("26.1 has no eligibility gate") is gone. **PASS — B7-G04 FIXED.** Residual: the shared `allowed_in_peaceful` list wrongly includes hoglin (37 vanilla `notInPeaceful` types, no hoglin — verified by extraction from the `EntityType` dump) — recorded as **B3-G04**, which also covers the spawn-egg facet; it lives in Batch 3's tally, not this batch's. This confirms the coordinator's second follow-up (B4's flag): the gate is the fix for 26.1 B7-G04, not a regression.
- **B10 (spawn egg ENTITY_PLACE position):** vanilla posts `ENTITY_PLACE` at the spawn position; the Rust posts at `p` (the spawn pos, game/creatures.rs `use_spawn_egg`). **PASS** (1.21.7 G05 stays withdrawn — see §5.2).
- **B11 (spawn egg ambient sound):** vanilla `EntityType.spawn` calls `Mob.playAmbientSound()` after `addFreshEntityWithPassengers` (javap-verified fresh, `EntityType` dump line 565); the Rust `ambient_sound` is only called from the dispenser path (game/dispenser.rs:985), never on spawn-egg placement (game/creatures.rs `use_spawn_egg`). **GAP — B7-G06 (minor, reproduces from 26.1).**
- **B35 (dispenser spawn egg, NEW check):** vanilla `SpawnEggItemBehavior.execute` pops the `EntityType.spawn` result and UNCONDITIONALLY shrinks the stack by 1 and fires `ENTITY_PLACE` at the dispenser pos (javap-verified fresh, server-game.jar) — the egg is consumed and the event fires even when the spawn returns null. The 26.1.1 port (game/dispenser.rs:1019-1021) returns the stack unshrunk with no event on a null spawn (the 26.2 form). The 26.1 tree matched vanilla. **GAP — B7-G18 (minor, NEW in 26.1.1).** (The dispenser path calls `EntityType.spawn` directly with no peaceful gate in vanilla either — the port matches there.)

### §3.3 Block transforms (B12, B13, B14, B16) — 4 gap

**B12 (axe strip/scrape/wax-off), B13 (hoe till), B14 (shovel flatten), B16 (copper chest scrape/wax-off):** vanilla 26.1.1 has NO `block_transformer` registry or component — the axe/hoe/shovel tables live in `AxeItem`/`HoeItem`/`ShovelItem` (all three classes byte-identical to 26.1's) and the jar's datapack carries 0 `block_transformer` data files. The 26.1.1 port reverted to the 26.2/26.3 form: `game.rs` `block_transform` reads the `minecraft:block_transformer` component from `items.json` (0/1,506 items carry it; horizon-items/src/items.rs:942) and returns `None` for every tool (game.rs:7932-7937), and `block_transform.rs` `Transformers::load` reads `data/minecraft/block_transformer/*.json` (0 files in the prepared data), so the transform table is empty. **All four GAP — B7-G07 (major, NEW vs 26.1; reproduces 1.21.7 B7-G07, which 26.1 had withdrawn).** Honeycomb wax-on is dead the same way (`block_transform.rs:245` `waxed()` reads the empty axe table; callers game.rs:8323, game/remote.rs:8778) and is recorded as part of G07. The 26.1 tree hardcoded the correct tables — it is the repair reference.

- **G13/G14/G15 are FIXED behaviourally, masked by G07** (see §5.1): the shield check (game.rs:7940-7946), the `item_used_on_block` trigger (game.rs:7986), and the copper-chest neighbour code (game.rs:8032, called at :8014) are all still wrong but unreachable, because `block_transform` returns `None` before any of them runs. **All three return if G07 is fixed.**
- Side finding: vanilla `HoneycombItem` DOES reference `ITEM_USED_ON_BLOCK` (1×, javap-verified fresh), so G14's scope is hoe/shovel/campfire only — the port's honeycomb trigger is correct.

### §3.4 Campfire douse (B15) — GAP

**B15 (shovel campfire douse):** vanilla 26.1.1 `ShovelItem.useOn` dowses a lit campfire directly (face != DOWN, `CampfireBlock.dowse` + level event 1009); there is no `#douses_campfires` tag in 26.1.1 (absent from `ItemTags`, the client jar's datapack and the prepared item tags — 187 files, no `douses_campfires.json`). The 26.1.1 port reverted to the 26.3 form: game/campfire_douse.rs:12-17 gates the douse on `#minecraft:douses_campfires` (`CampfireBlock.useItemOn`), so `campfire_to_douse` always returns `None` and a shovel never puts out a campfire; the face != DOWN check was also dropped. The 26.1 tree implemented the douse directly via `#minecraft:shovels`. **GAP — B7-G08 (major, NEW vs 26.1; reproduces 1.21.7 B7-G08, which 26.1 had withdrawn).** Still wired in game/use_hand.rs:88. (Also flagged by Batch 5 as Batch 7's.)

### §3.5 Chorus fruit (B18) — GAP

**B18 (chorus fruit teleport):** vanilla 26.1.1 `LivingEntity.randomTeleport` is blocksMotion-grounded (4-arg signature, no tag parameter, javap-verified fresh). The Rust game/consume.rs:120-139 walks down for `#minecraft:entities_can_teleport_to`, a tag absent in 26.1.1 (absent from `BlockTags`, the client jar's datapack and prepared `block-tags.json`, 248 tags), so the loop always runs to `min_y` and returns false — chorus fruit never teleports. **GAP — B7-G09 (major, reproduces from 26.1).**

**Masked finding (no separate item):** the 26.1.1 tree added a level-event 2017 directional-particle trail to the consume path (horizon-world/src/teleport_consume.rs `apply` + game/consume.rs:422-431 `directional_particles`; vanilla `TeleportRandomlyConsumeEffect` has only a `diameter` field and emits no level event, javap-verified fresh) — but it is masked by G09: the trail fires only after a successful teleport, which never happens on the player path. Fixing G09 without removing the trail would expose it. (The fox path already emits it: B3-G43. This is the coordinator's first follow-up, B3's flag, verified fresh.)

### §3.6 Buckets (B2) — PASS

**B2 (bucket sneak-redirect):** vanilla 26.1.1 `BucketItem.emptyContents` HAS the sneak check (`isShiftKeyDown`, class byte-identical to 26.1's; javap-verified fresh), and the Rust `bucket_empty` (horizon-world/src/level/buckets.rs — file byte-identical to the 26.1 tree's) matches: air always takes the fluid, retry beside with a null hit result. **PASS** (1.21.7 G02 stays withdrawn — see §5.2).

### §3.7 Potions (B24, B26) — 2 gap

- **B24 (water splash/lingering douse):** vanilla 26.1.1 `AbstractThrownPotion` has ZERO `PotionTags` references — it gates on `PotionContents.is(Potions.WATER)` directly (class byte-identical to 26.1's; javap-verified fresh). 26.1.1's `PotionTags` class has only `TRADEABLE` and prepared `tags/potion/` contains only `tradeable.json`. The Rust gates everything on potion tags (game/splash_water.rs:42-50 `potion_in_tag`: `minecraft:douses_fire`, `hurts_water_sensitive_entities`, `extinguishes_entities`, `rehydrates_axolotls` — file byte-identical to the 26.1 tree's), which always return false, so water splash/lingering bottles do nothing on impact. **GAP — B7-G10 (major, reproduces from 26.1; duplicate of B1-G03 — coordinator to fold).**
- **B26 (water bottle on dirt → mud):** vanilla 26.1.1 `PotionItem.useOn` uses `BlockTags.CONVERTABLE_TO_MUD` (with an A, javap-verified fresh); the Rust uses `convertible_to_mud` (with an I) in all four places (game/potions.rs:710, game/dispenser.rs:2245, game/remote.rs:8658, horizon-world/src/level/data.rs:460), so the lookup always misses and a water bottle on dirt never makes mud. **GAP — B7-G11 (minor, reproduces from 26.1; duplicate of B5-G19 — coordinator to fold).**

### §3.8 Boats (B27) — GAP

**B27 (boat placement eye check):** vanilla 26.1.1 `BoatItem.use` gathers entities in the player's bbox expanded 5.0 along the view + inflate 1.0 and returns PASS when the eye is inside any entity's pick-inflated bbox (class byte-identical to 26.1's; javap-verified fresh). The Rust `boat_to_place` (game/boats.rs:1517-1542 — file byte-identical to the 26.1 tree's) only checks the boat's own bbox, so a boat is placeable while the player's eye is inside an entity. **GAP — B7-G12 (minor, reproduces from 26.1).**

### §3.9 Ender pearl / wind charge cooldown (B30) — PASS

**B30 (throw cooldown):** vanilla 26.1.1 `ItemStack.use` calls `applyAfterUseComponentSideEffects` for instant-use items on Success, which reads `USE_COOLDOWN` — pearls (1.0s) and wind charges (0.5s) DO get a cooldown. The 26.1.1 Rust still applies it: `throw_item` (game/creatures.rs:7379 wind charge, :7427 pearl) calls `apply_use_cooldown` (game.rs:5211), which reads the `use_cooldown` component. **PASS — B7-G16 stays FIXED (as in 26.1).**

### §3.10 Spear (B33) — GAP (cited, not minted)

**B33 (spear stab knockback):** vanilla 26.1.1 `LivingEntity.stabAttack` calls `causeExtraKnockback` ONCE with `0.4f + getKnockback(...)` (javap-verified fresh, `LivingEntity` dump lines 7700-7764). The Rust applies it as two pushes (mobs/spear_use.rs:272, 275). **GAP — already recorded as B3-G45; cited, not re-minted** (new finding already minted by a landed batch).

### §3.11 Remaining behaviour checks — all PASS

- **B3 (food consume):** consume path unchanged from 26.1 (diff: only the `directional_particles` addition at consume.rs:422-431, which is inert behind G09); food components match D1. **PASS.**
- **B4 (tool use):** tool components match D4/D5; tool use paths unchanged. **PASS.**
- **B5 (armour equip):** armour data matches D7; equip path unchanged. **PASS.**
- **B7 (fluid use / consume / offspring / entity_data):** unchanged code; fluid use matches vanilla. **PASS.**
- **B17 (bundle use):** `game/bundles.rs` byte-identical to the 26.1 tree's. **PASS.**
- **B19 (fishing rod):** `game/fishing.rs` byte-identical to the 26.1 tree's. **PASS.**
- **B20 (crossbow):** `game/crossbow.rs` byte-identical to the 26.1 tree's. **PASS.**
- **B21 (shield block):** `game/shield.rs` byte-identical to the 26.1 tree's. **PASS.**
- **B22 (trident throw):** `game/trident.rs` byte-identical to the 26.1 tree's. **PASS.**
- **B23 (mace smash):** `game/mace.rs` byte-identical to the 26.1 tree's. **PASS.**
- **B25 (item use order):** `game/item_use.rs` byte-identical to the 26.1 tree's. **PASS.**
- **B28 (mount menu):** `game/mount_menu.rs` byte-identical to the 26.1 tree's. **PASS.**
- **B29 (world commands item paths):** `game/commands/world_cmds.rs` byte-identical to the 26.1 tree's. **PASS.**
- **B31 (composter — cited):** composter is dead in 26.1.1 (`compost_chance` removed; the port reads `minecraft:compostable`, which 0 items carry) — already recorded as **B5-G22**, cited, not re-minted. **PASS for this batch's tally** (the defect is Batch 5's; this batch does not double-count it).
- **B32 (villager food):** the villager-food component form was checked by Batch 3 and found harmless. **PASS.**
- **B34 (nautilus armour):** nautilus armour exists in 26.1.1 (5 items in `items.json`, present in the jar's `Items` class); armour path matches D7. **PASS.**

### §3.12 Initial / respawn spawn (B36, NEW check) — GAP

**B36 (spawn in skylight-less dimensions):** vanilla 26.1.1 `PlayerSpawnFinder.findSpawn` checks `dimensionType().hasSkyLight()` FIRST: in a dimension without skylight (or in adventure mode) it returns `fixupSpawnHeight` at the suggestion at once, with no radius search (javap-verified fresh, server-game.jar, `PlayerSpawnFinder` dump lines 69-115). The 26.1.1 port (game/initial_spawn.rs:294-298 `spawn_finder`) dropped the `!sky_light` arm, so in the Nether/End it searches the respawn radius instead of returning the fixup height. The 26.1 tree matched vanilla. **GAP — B7-G19 (minor, NEW in 26.1.1).** (Coordinator's first follow-up, B3's flag, verified fresh.)

## §4 Defect list

### Gap items (12)

| ID | Sev | Title |
|---|---|---|
| B7-G01 | minor | `ComponentMap::set` errors on a dangling removal marker; vanilla returns null |
| B7-G03 | minor | Spawn egg `entity_data` id type override ignored |
| B7-G06 | minor | No `Mob.playAmbientSound()` on spawn-egg placement |
| B7-G07 | major | Axe/hoe/shovel transforms and honeycomb wax-on entirely non-functional (26.2/26.3 `block_transformer` form) |
| B7-G08 | major | Shovel campfire douse dead: gated on the absent `#douses_campfires` tag |
| B7-G09 | major | Chorus fruit never teleports (tag `#entities_can_teleport_to` absent in 26.1.1) |
| B7-G10 | major | Water splash/lingering bottles do nothing on impact (potion-tag gate dead) — duplicate of B1-G03 |
| B7-G11 | minor | Water bottle on dirt never makes mud (`convertible` vs `convertable`) — duplicate of B5-G19 |
| B7-G12 | minor | Boat placement missing the eye-inside-entity PASS check |
| B7-G17 | minor | Op's spawn-egg `entity_data` applied to spawners (26.1.1 has no `Spawner.setEntityData`) — duplicate of B3-G39 |
| B7-G18 | minor | Dispenser spawn egg returned unshrunk with no `ENTITY_PLACE` on a null spawn (NEW) |
| B7-G19 | minor | Respawn/initial spawn searches the radius in skylight-less dimensions instead of the fixup height (NEW) |

### Fixed items (5)

| ID | Sev | Title |
|---|---|---|
| B7-G04 | minor | Peaceful gate for hostile spawn eggs (26.1 defect fixed in 26.1.1) |
| B7-G13 | minor | Shield blocking-intent check (masked by B7-G07; code still wrong) |
| B7-G14 | minor | `ITEM_USED_ON_BLOCK` trigger for hoe/shovel/campfire (masked by B7-G07/B7-G08; code still wrong) |
| B7-G15 | minor | Copper-chest neighbour `BLOCK_CHANGE` (masked by B7-G07; code still wrong) |
| B7-G16 | minor | Ender pearl / wind charge use cooldown applied on throw (stays fixed) |

### Closed items (2)

| ID | Sev | Title |
|---|---|---|
| B7-G02 | minor | [CLOSED] Bucket sneak-redirect: vanilla 26.1.1 HAS the sneak check |
| B7-G05 | minor | [CLOSED] Spawn-egg `ENTITY_PLACE` position: vanilla posts at the spawn position |

### Oracle-defect items (30)

All 30 reproduce (Batch 0 rows identical to the 26.1 run; root causes javap-verified fresh — see §6).

| ID | Sev | Oracle / root cause |
|---|---|---|
| B7-O01 | major | `VanillaDispenserOracle` — `EntityTypes` package does not exist |
| B7-O02 | major | `VanillaBlockUpdateOracle` — `ClockInstance` does not exist (32 errors) |
| B7-O03–O07 | minor | `Level.getNextEntityId` does not exist (Arrow, BoatRider, ClientArrow, ExplosionItems, LavaCauldronItem) |
| B7-O08–O09 | minor | `EntityTypes` does not exist (BoatAuthority, ItemModel) |
| B7-O10 | minor | `EntitySpawnRequest` does not exist (VehicleContainer) |
| B7-O11 | minor | `ServerboundPunchPacket` does not exist (KnockbackClient) |
| B7-O12 | minor | `InteractionResult.Success.shouldSwing()` does not exist (Consumable) |
| B7-O13 | minor | 5-arg `randomTeleport` does not exist (TeleportConsume) |
| B7-O14 | minor | 5-arg `blockUsingItem` does not exist (ItemBlocking) |
| B7-O15–O16 | minor | `hurtAndBreak` takes `Consumer<Item>` (ItemDurability, ComponentDurability) |
| B7-O17 | minor | `onEquippedItemBroken` takes `Item` (BlockingItemDamage) |
| B7-O18 | minor | 3-arg `Player.drop` does not exist (ExtraItems) |
| B7-O19–O20 | minor | `ServerChunkCache` takes 11 constructor args (ItemUseStart, InventoryUseStart) |
| B7-O21–O22 | minor | `Gui` takes 1 constructor arg (HeldDrop, InteractionInput) |
| B7-O23–O24 | minor | `ItemStackTemplate.fromStack` does not exist (ItemTemplate, UseRemainder) |
| B7-O25–O27 | minor | `VanillaRegistries.createWorldLookup` does not exist (ItemUseCatalog, InventoryPickup, PlayerInventory) |
| B7-O28 | minor | `LivingEntity.SwingDescription` does not exist (HeldItemStack) |
| B7-O29–O30 | minor | Cascades on broken `VanillaBlockUpdateOracle` (EnchantDurability, HelmetDamage) |

## §5 Re-check verdicts

### §5.1 26.1 items (12)

| ID | 26.1 verdict | 26.1.1 verdict | Notes |
|---|---|---|---|
| G01 | gap | **REPRODUCES** | `component_patch.rs` byte-identical |
| G03 | gap | **REPRODUCES** | `use_spawn_egg` still derives kind from the item name |
| G04 | gap | **FIXED** | Gate added (creatures.rs:7511-7535); hoglin residual is B3-G04 |
| G06 | gap | **REPRODUCES** | `ambient_sound` still dispenser-only |
| G09 | gap | **REPRODUCES** | Tag absent; consume path unchanged |
| G10 | gap | **REPRODUCES** | `splash_water.rs` byte-identical; duplicate of B1-G03 |
| G11 | gap | **REPRODUCES** | All four `convertible_to_mud` sites present; duplicate of B5-G19 |
| G12 | gap | **REPRODUCES** | `boats.rs` byte-identical |
| G13 | gap | **FIXED (masked)** | Code still wrong; unreachable behind G07 — returns if G07 is fixed |
| G14 | gap | **FIXED (masked)** | Code still wrong; unreachable behind G07/G08 — returns if those are fixed |
| G15 | gap | **FIXED (masked)** | Code still wrong; unreachable behind G07 — returns if G07 is fixed |
| G17 | gap | **REPRODUCES** | Regular arm unchanged; trial arm NEW in 26.1.1; duplicate of B3-G39 |

### §5.2 1.21.7 items (12)

| ID | 26.1 verdict | 26.1.1 verdict | Notes |
|---|---|---|---|
| G01 | reproduces | **REPRODUCES** | Same defect as 26.1 G01 (one item) |
| G02 | withdrawn | **WITHDRAWN** | Vanilla 26.1.1 `BucketItem` HAS the sneak check; closed item |
| G03 | reproduces | **REPRODUCES** | Same defect as 26.1 G03 (one item) |
| G04 | reproduces (reversed) | **FIXED** | Vanilla HAS the gate and the port now HAS it; same item as 26.1 G04 |
| G05 | withdrawn | **WITHDRAWN** | Vanilla posts `ENTITY_PLACE` at the spawn position; closed item |
| G06 | reproduces | **REPRODUCES** | Same defect as 26.1 G06 (one item) |
| G07 | withdrawn | **REPRODUCES** | The 26.1.1 tree reverted to the 26.2/26.3 `block_transformer` form; NEW vs 26.1 |
| G08 | withdrawn | **REPRODUCES** | The 26.1.1 tree reverted to the `#douses_campfires` gate; NEW vs 26.1 |
| G09 | reproduces | **REPRODUCES** | Same defect as 26.1 G09 (one item) |
| G10 | reproduces | **REPRODUCES** | Same defect as 26.1 G10 (one item) |
| G11 | reproduces | **REPRODUCES** | Same defect as 26.1 G11 (one item) |
| G12 | reproduces | **REPRODUCES** | Same defect as 26.1 G12 (one item) |

### §5.3 26.2 candidates (4)

| ID | 26.1 verdict | 26.1.1 verdict | Notes |
|---|---|---|---|
| G13 | gap | **FIXED (masked)** | See §5.1 |
| G14 | gap | **FIXED (masked)** | See §5.1; honeycomb trigger scope corrected (vanilla `HoneycombItem` DOES reference the trigger) |
| G15 | gap | **FIXED (masked)** | See §5.1 |
| G16 | fixed | **FIXED** | Cooldown still applied (creatures.rs:7379, 7427) |

## §6 Oracle triage

All 30 failing item-family oracles are **IDENTICAL** in the 26.1.1 Batch 0 table (same class, error count, first error) to the 26.1 run, all 36 passing ones pass identically, and there is **no new item-family failing oracle** (`evidence/oracle-triage-26.1.1.txt`). All 30 root causes were javap-verified fresh against 26.1.1 dumps:

- **O01/O08/O09** — `EntityTypes` does not exist in the 26.1.1 client jar (registration still in `EntityType`).
- **O02** — `ClockInstance` does not exist in the 26.1.1 client jar (32 errors; cascade root for O29/O30).
- **O03–O07** — `Level.getNextEntityId` does not exist in 26.1.1 (0 matches in the `Level` dump).
- **O10** — `EntitySpawnRequest` does not exist. **O11** — `ServerboundPunchPacket` does not exist. **O28** — `SwingDescription` does not exist anywhere in the jar and `LivingEntity` has no `getCurrentSwing`.
- **O12** — `InteractionResult.Success` is a record with a `swingSource` field; no `shouldSwing()` method.
- **O13** — `LivingEntity.randomTeleport` is 4-arg `(DDDZ)`, no tag parameter.
- **O14** — `LivingEntity.blockUsingItem` is 2-arg `(ServerLevel, LivingEntity)`.
- **O15/O16** — `ItemStack.hurtAndBreak` takes `Consumer<Item>`, not `Consumer<ItemStack>`.
- **O17** — `LivingEntity.onEquippedItemBroken` takes `(Item, EquipmentSlot)`.
- **O18** — `Player.drop` is 2-arg `(ItemStack, boolean)`; `handleExtraItemsCreatedOnUse` DOES exist on `ServerPlayer`.
- **O19/O20** — `ServerChunkCache` takes 11 constructor args (the oracle calls a 10-arg super).
- **O21/O22** — `Gui` takes 1 arg (`Gui(Minecraft)`); the oracle calls `super(null, null, null)`.
- **O23/O24** — `ItemStackTemplate` has only 1-arg `fromNonEmptyStack(ItemStack)`; no `fromStack`.
- **O25–O27** — `VanillaRegistries` has `createLookup()`, not `createWorldLookup`; `Enchantments.UNBREAKING` does exist (ResourceKey form).
- **O29/O30** — cascades on O02; no direct javac error of their own.

Passing in-scope oracles: the same 36 as 26.1, including `VanillaItemCooldownsOracle` and `VanillaComponentPatchOracle`.

## §7 Driver inventory

74 drivers — **32 runnable / 25 blocked / 17 no-oracle** — byte-identical to 26.1's inventory (`evidence/driver_inventory_26_1_1.py`, `evidence/driver-inventory-26.1.1.json`; the only mismatch is `interaction_input` referencing the `ParityData` helper). **No driver was run**, per the brief; the 32 runnable drivers are recorded as not run, not as passes.

## §8 Systemic findings

1. **The 26.1.1 tree is a stale re-port from 26.2/26.3 form, not a carry-forward of the 26.1 tree** (Batch 2's §6 finding, confirmed in B7 scope). Every 26.1 defect reproduces except the peaceful gate (G04, genuinely fixed) and G13/G14/G15 (fixed only because the whole transform path is dead), AND the tree adds four NEW behaviour defects that are one-method reversions to the 26.2/26.3 form the 26.1 tree deliberately removed: G07 (`block_transformer` registry/component instead of hardcoded tables), G08 (`#douses_campfires` tag instead of the direct shovel douse), G18 (26.2 dispenser null-spawn form), G19 (skylight gate dropped from `spawn_finder`). G07 and G08 are 1.21.7 defects the 26.1 tree had fixed, returning verbatim. **The 26.1 tree is the repair reference for all four.**
2. **Absent-tag/component reads got worse.** The 26.1.1 tree reads MORE absent tags/components than the 26.1 tree did: `#douses_campfires`, `#entities_can_teleport_to` (in a second place — enderman), `#enderman_does_not_teleport_to`, `#consumable_does_not_teleport_to`, the `minecraft:block_transformer` and `minecraft:mob_visibility` components. The tag table resolves absent tags to empty sets, so every read silently degrades to false, never an error.
3. **The counter-example is G04.** The 26.1.1 tree fixed the peaceful gate the 26.1 tree got wrong (the 26.1 tree's "26.1 has no eligibility gate" comment was false, as B4's flag established). Version-specific work in 26.1.1 is uneven, not uniformly behind.
4. **Masked defects are a trap for fixers.** G13/G14/G15 (and the player 2017 trail under G09) are behaviourally fixed only because G07/G08/G09 kill the whole path. Any fix of G07, G08 or G09 must also fix the masked code or the defects reappear.
5. **Two 26.1-record corrections (from the 26.1 summary's "26.2-record corrections" section, line 283):** none applies to B7 scope — the corrections concern B3/B9/B6. But this batch found two fresh corrections to the 26.1 report itself: (a) vanilla `getVisibilityPercent` hardcodes five head/type pairs INCLUDING `PIGLIN_BRUTE + PIGLIN_HEAD` — the 26.1 report's "piglin brutes are not fooled" claim is wrong (`LivingEntity` is byte-identical between 26.1 and 26.1.1, so it was wrong for 26.1 too); (b) the method applies no `0..=10` clamp — the 26.1 report's clamp claim is wrong too. Both are recorded in the B3-G33 evidence. (c) B3-G39's claim that the regular spawner arm was "missed by the 26.1 sweep" is wrong — 26.1 B7-G17 is exactly that arm.
6. **Out-of-scope flags (cited, not minted):** sign components → Batch 4 (26.1 B4 covered the no-component form); enderman teleport → Batch 3 (B3-G14); swing packet + `SwingState` rework → Batch 9/10 (the local player's item-use swings no longer send `ServerboundSwingPacket`; vanilla `LocalPlayer.swing` sends it unconditionally, javap-verified fresh); dispenser shear alive-check → Batch 10; `data.rs` cobweb/`blocks_motion` changes → Batch 3/Batch 1; `#entities_can_teleport_to` at `game/commands/world_cmds.rs:2141-2146` → Batch 8 (commands). The 26.2 run's out-of-scope observations were NOT re-verified for 26.1.1 and are not carried forward as claims.

## §9 Method notes + evidence index

**Method.** Static analysis only: `javap -c -p` on the named 26.1.1 jars (`client.jar` for client classes, `server-game.jar` for server classes) for every vanilla method in scope; Rust source reads; prepared-data checks (`items.json`, `block-tags.json`, jar data files); and a full diff of every in-scope source file between the 26.1 and 26.1.1 trees. A sha1 byte-compare of 66 in-scope vanilla classes between the 26.1 and 26.1.1 jars found 0 diffs. No drivers were run and no oracle repairs were done (checks + triage only).

**Evidence** (`~/workspace/mcir-work/sweep-26.1.1-batches/b07/evidence/`):

- `tree-diff-list.txt` — 389 files differ between `versions/26.1/crates` and `versions/26.1.1/crates`.
- `diff-*.txt` — 18 in-scope file diffs + `diff-initial_spawn.rs.txt`.
- `jar_compare_26_1_1.py` + `jar-compare-26.1.1.log` — 66/66 in-scope classes byte-identical.
- `javap_dumps.sh`, `javap/` — ~60 fresh 26.1.1 javap dumps.
- `parse_items_class_26_1_1.py/.out`, `item-class.json`, `nonblock-by-class.json` — D8 map.
- `data_checks_26_1_1_v2.py/.out`, `materials-26.1.1.json` — D1–D7.
- `driver-map-26.2.txt`, `driver_inventory_26_1_1.py/.out`, `driver-inventory-26.1.1.json` — driver inventory.
- `oracle-triage-26.1.1.txt` — oracle triage vs Batch 0.

**Key jar evidence** (all from fresh 26.1.1 dumps in `b07/evidence/javap/`): `SpawnEggItem.txt:126-153` spawnMob peaceful gate; `:256-274` getType reads ENTITY_DATA; `EntityType.txt:565` playAmbientSound in spawn; 37 `notInPeaceful` types extracted (no hoglin). `LivingEntity.txt:9930` randomTeleport 4-arg `(DDDZ)`; `:3340` blockUsingItem 2-arg; `:10378` onEquippedItemBroken `(Item, EquipmentSlot)`; `:2047-2177` getVisibilityPercent (5 pairs incl. PIGLIN_BRUTE, no clamp); `:7700-7764` stabAttack single causeExtraKnockback `0.4f + getKnockback`. `ItemStack.txt:723` hurtAndBreak `Consumer<Item>`; `Player.txt:1128` drop 2-arg; `ServerChunkCache.txt:39` 11-arg ctor; `Gui.txt:179` 1-arg ctor; `ItemStackTemplate.txt:74` only `fromNonEmptyStack`; `VanillaRegistries.txt:39` only `createLookup`; `InteractionResult_Success.txt` record with `swingSource`, no `shouldSwing`; `Level.txt` 0 `getNextEntityId`; `InteractionResult.txt` SwingSource CLIENT/SERVER/NONE. `SpawnEggItemBehavior.txt` execute pops spawn result, unconditionally shrink + ENTITY_PLACE; `PlayerSpawnFinder.txt:69-115` findSpawn skylight gate; `TeleportRandomlyConsumeEffect.txt` only `diameter` field, apply calls 4-arg randomTeleport + game event + sound, 0 level-event refs; `LocalPlayer.txt:677-687` swing sends ServerboundSwingPacket; `BaseSpawner.txt:77,83` and `TrialSpawnerBlockEntity.txt:83` only `setEntityId`.

## §10 What this batch defers + verdict

**Deferred (not this batch's work):** driver runs (32 runnable drivers unrun), oracle repairs (30 failing oracles), all fixes (including the four new defects G07/G08/G18/G19, whose repair reference is the 26.1 tree), and the out-of-scope flags in §8.6.

**Verdict: 44 checks — 28 pass, 16 gap.** The 26.1.1 Items tree is worse than the 26.1 tree: it carries all but one of the 26.1 defects, resurrects two 1.21.7 defects the 26.1 tree had fixed (G07, G08), and adds two more one-method reversions (G18, G19). The one genuine improvement is the peaceful gate (G04). Three reproductions are duplicates of landed batches (B1-G03, B5-G19, B3-G39) for the coordinator to fold.
