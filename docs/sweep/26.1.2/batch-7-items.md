# Batch 7 — Items (Minecraft 26.1.2)

**Scope:** item use, food, tools, armour, buckets, spawn eggs, components — 1,506 items.

**Result: 44 checks — 28 pass, 16 gap** (as-tallied 44 = 44 behaviour + 0 non-behaviour). 12 gap items (4 major, 8 minor), 5 fixed, 2 closed, 30 oracle-defect items. The headline: the 26.1.2 Items tree is the 26.1.1 tree — every in-scope file byte-identical, every in-scope vanilla class byte-identical, the Batch 0 oracle table row-identical — so **all 49 of the 26.1.1 items re-decide the same way**, with fresh 26.1.2 evidence for each. No new behaviour defect was found in scope. The two re-check-first regressions (G07 tool transforms, G08 campfire douse) both REPRODUCE.

## §0 Execution status

- **Checks: 44 total, 28 pass, 16 gap — all behaviour, 0 non-behaviour.** Oracle triage is not tallied (it is not a driver check).
- **Items: 49** — 12 gap (4 major, 8 minor), 5 fixed, 2 closed, 30 oracle-defect (2 major, 28 minor).
- **No drivers were run and no oracle repairs were done** (checks + triage only, per the brief). No UNVERIFIED carry-overs: every mandatory re-check (49 from 26.1.1, 12 from 1.21.7, 4 from 26.2) was decided fresh against 26.1.2's jar and tree.
- **Tree diff vs 26.1.1:** 24 files differ between `versions/26.1.1/crates` and `versions/26.1.2/crates` tree-wide (see `evidence/tree-diff-list.txt`); **all 30 in-scope files are byte-identical** (see `evidence/in-scope-diff.txt`). The only near-scope deltas are out of scope: the protocol sign-text removal (Batch 4's B4-G33 facet), the `handleAttack` spectator gate in `lan/hub.rs` (Batch 3/9), the Checkbox tooltip in `dialog_screen.rs` (Batch 10), and a test-only painting/variant id change (102→109) in `item_sync.rs`.
- **Vanilla diff vs 26.1.1:** a sha1 byte-compare of 66 in-scope classes between the 26.1.1 and 26.1.2 jars found **0 diffs** — vanilla changed nothing in scope. Every verdict is still re-verified fresh against the 26.1.2 jar; the byte-identity is a cross-check, not the proof.
- **Oracle table vs 26.1.1:** the landed 26.1.2 Batch 0 table is **row-identical** to the 26.1.1 table — 0 of 391 rows differ, 202 failing in both.
- **Worktree state:** the 26.1.2 tree's `.local` (Batch 12's prepare, exit 0) is present; jar sha1s match the brief pins (client `4e618f09…`, server-game `83eb1106…`).
- **B9 (peaceful gate) is counted as PASS** in the tally: the gate stays fixed in 26.1.2 (B7-G04). The hoglin residual (hoglin wrongly in the shared `allowed_in_peaceful` list) is recorded as **B3-G04** and lives in Batch 3's tally, not this batch's (landed 26.1.2 Batch 3 report: B3-G04 REPRODUCES).

## §1 Item → class map

Same extraction as 26.1.1, re-run against the 26.1.2 jar (`evidence/parse_items_class_26_1_2.py`, output `evidence/parse_items_class_26_1_2.out`): **1,506 items mapped to 70 classes, 0 unmapped, 0 mismatches vs `block-behaviours.json`** — the parse output is byte-identical to the 26.1.1 run's. 26.1.2 has spears (7) and nautilus armour (5), so B33/B34 are full checks, not not-covered.

**Correction to the 26.1.1 report's §1 table (coordinator note for the 26.1.1 record):** that table does not match its own evidence. It has 57 rows summing to **1,530**, not 1,506 over 70 classes, and lists classes the jar never constructs for those items — swords, spears and armour are plain `Item` with `Properties.sword/spear/humanoidArmor` components in 26.x (the extraction maps `copper_sword`, `iron_spear`, `copper_nautilus_armor` to `Item`), and seven rows name non-item classes (`CraftingBookCategory`, `UseEffects`, `UseRemainder`, `EnchantmentEffectComponents`, `Enchantments`, `ItemEnchantments`, `SingleEnchantment`). The true histogram (this run's extraction, identical for 26.1.1 and 26.1.2) is:

| Class | Count | Class | Count |
|---|---|---|---|
| `net.minecraft.world.item.BlockItem` | 910 | `net.minecraft.world.item.EggItem` | 3 |
| `net.minecraft.world.item.Item` | 235 | `net.minecraft.world.item.PlaceOnWaterBlockItem` | 2 |
| `net.minecraft.world.item.SpawnEggItem` | 87 | `net.minecraft.world.item.FoodOnAStickItem` | 2 |
| `net.minecraft.world.item.DoubleHighBlockItem` | 28 | `net.minecraft.world.item.ItemFrameItem` | 2 |
| `net.minecraft.world.item.StandingAndWallBlockItem` | 20 | 46 further classes at 1 each | 46 |
| `net.minecraft.world.item.BoatItem` | 20 | | |
| `net.minecraft.world.item.SmithingTemplateItem` | 19 | | |
| `net.minecraft.world.item.BundleItem` | 17 | | |
| `net.minecraft.world.item.DyeItem` | 16 | | |
| `net.minecraft.world.item.BedItem` | 16 | | |
| `net.minecraft.world.item.BannerItem` | 16 | | |
| `net.minecraft.world.item.SignItem` | 12 | | |
| `net.minecraft.world.item.HangingSignItem` | 12 | | |
| `net.minecraft.world.item.GameMasterBlockItem` | 7 | | |
| `net.minecraft.world.item.ShovelItem` | 7 | | |
| `net.minecraft.world.item.AxeItem` | 7 | | |
| `net.minecraft.world.item.HoeItem` | 7 | | |
| `net.minecraft.world.item.MinecartItem` | 6 | | |
| `net.minecraft.world.item.MobBucketItem` | 6 | | |
| `net.minecraft.world.item.BucketItem` | 3 | | |

The 46 singletons include `BowItem`, `CrossbowItem`, `TridentItem`, `MaceItem`, `ShieldItem`, `FishingRodItem`, `EnderpearlItem`, `WindChargeItem`, `PotionItem`, `SplashPotionItem`, `LingeringPotionItem`, `HoneycombItem`, `FlintAndSteelItem`, `ShearsItem`, `CompassItem`, `SpyglassItem` (full list in `evidence/parse_items_class_26_1_2.out`). BlockItem is 910 in the final map (the raw histogram prints 886 before the 24 WeatheringCopperItems collection items — copper bars/chain/lantern × 8 oxidation/wax states — are added as `BlockItem`). Block items: 1,026; non-block items: 480.

## §2 Data checks

All 8 pass, identical counts to 26.1.1 (`evidence/data_checks_26_1_2_v2.py`, output `evidence/data_checks_26_1_2_v2.out` — byte-identical to the 26.1.1 run's output). ToolMaterial/ArmorMaterials are byte-identical between the jars, so `materials-26.1.2.json` is the same data as 26.1.1's (copied, not re-extracted from a different source).

- **D1 (food):** 44 foods in `items.json`, all with the expected food components (checked against the jar's FoodProperties in `ItemProperties`/`Items`). **PASS.**
- **D2 (durability):** 84 items have a `max_damage` component; values match the jar's `Item.MAX_DAMAGE`/`ItemProperties.durability` for every item. **PASS.**
- **D3 (max stack size):** 1,506 items — every `max_stack_size` matches the jar. **PASS.**
- **D4 (tool/spear pairs):** 42 tool/spear pairs (7 materials × 6 tool kinds + spear variants) with matching `tool`/`attack_damage`/`attack_speed` components; 7 spears × 8 kinetic values also match. **PASS.**
- **D5 (tool components):** 35 tool components, matching the jar's `Tool` definitions (rules, damage per block, mining speed). **PASS.**
- **D6 (tags):** 13 item tags checked against the jar's ItemTags — all present and correct. **PASS.**
- **D7 (armour):** 41 armour items; material, defence, toughness, knockback resistance and equip sound match the jar's ArmorMaterials. **PASS.**
- **D8 (item→class map):** 1,506 items → 70 classes, 0 unmapped, 0 mismatches vs `block-behaviours.json` (see §1). **PASS.**

Prepared-data spot checks (26.1.2 `.local`): `items.json` has 1,506 items; **0 carry `minecraft:block_transformer`** (the component the transform path reads — see G07), **0 carry `minecraft:mob_visibility`**, and **0 carry `minecraft:compostable`** (see B31). `block-tags.json` has 248 tags; `#entities_can_teleport_to`, `#blocks_motion`, `#enderman_does_not_teleport_to`, `#consumable_does_not_teleport_to` are all ABSENT; `#convertable_to_mud` is present. Item tags: **186 files** (the 26.1.1 report's "187" was a miscount — both versions' prepared data have 186; the jar itself carries 203 in both); `shovels.json` exists; `douses_campfires.json` is absent. `tags/potion/` contains only `tradeable.json`; `block_transformer` data files: 0.

## §3 Behaviour checks

### §3.1 Component map (B1) — GAP

**B1 (component map set/remove):** vanilla 26.1.2 `PatchedDataComponentMap.set` returns `previous.orElse(prototype)` — null on a dangling removal marker, never an error (class byte-identical to 26.1.1's; javap-verified fresh). The Rust `ComponentMap::set` (horizon-world/src/component_patch.rs:106-119, line 116 — file byte-identical to the 26.1.1 tree's) returns `Err(MissingPrototype)` on a dangling removal marker; the doc comment above it is still wrong. **GAP — B7-G01 (minor, reproduces from 26.1.1).**

### §3.2 Spawn eggs (B6, B8, B9, B10, B11, B35) — 1 pass, 5 gap

- **B6 (spawn egg on spawner):** vanilla `SpawnEggItem.useOn` calls only `Spawner.setEntityId` — the 26.1.2 jar has NO `Spawner.setEntityData` (0 matches in the `BaseSpawner`, server `BaseSpawner` and `TrialSpawnerBlockEntity` dumps; `BaseSpawner` has only `setEntityId`, javap-verified fresh). The Rust applies an op's egg `entity_data` to spawners (game/spawner.rs:464 `set_entity_data` for regular spawners; :450 `override_entity_data` for trial spawners). **GAP — B7-G17 (minor, reproduces from 26.1.1; duplicate of B3-G39 — coordinator to fold; landed 26.1.2 Batch 3: B3-G39 REPRODUCES).**
- **B8 (spawn egg entity_data id override):** vanilla `SpawnEggItem.getType` reads the typed `entity_data` component's `type()` and spawns THAT type (javap-verified fresh: `getType` reads `DataComponents.ENTITY_DATA`, `TypedEntityData.type()`; `SpawnEggItem.txt:256-274`). The Rust derives the kind from the item name only (game/creatures.rs:7531 `use_spawn_egg`, `trim_end_matches("_spawn_egg")`) and ignores an `entity_data` id override naming a different type. **GAP — B7-G03 (minor, reproduces from 26.1.1).**
- **B9 (spawn egg peaceful gate):** vanilla `SpawnEggItem.spawnMob` FAILs hostile eggs in peaceful (`!isAllowedInPeaceful && difficulty == PEACEFUL`, javap-verified fresh, `SpawnEggItem.txt:126-153`). The 26.1.2 tree keeps the gate: game/creatures.rs:7511-7520 `spawn_egg_allowed` plus the `use_failed` return at :7532-7535, with FAIL semantics (`use_failed` means the action is not consumed, game/lan_actions.rs:2562-2565). **PASS — B7-G04 stays FIXED.** Residual: the shared `allowed_in_peaceful` list wrongly includes hoglin (37 vanilla `notInPeaceful` types extracted fresh from the 26.1.2 `EntityType` dump, no hoglin; port list at mobs/spawn_rules.rs:134, hoglin at :152) — recorded as **B3-G04**, which also covers the spawn-egg facet; it lives in Batch 3's tally, not this batch's (landed 26.1.2 Batch 3: B3-G04 REPRODUCES).
- **B10 (spawn egg ENTITY_PLACE position):** vanilla posts `ENTITY_PLACE` at the spawn position; the Rust posts at `p` (the spawn pos, game/creatures.rs `use_spawn_egg`). **PASS** (1.21.7 G05 stays withdrawn — see §5.2).
- **B11 (spawn egg ambient sound):** vanilla `EntityType.spawn` calls `Mob.playAmbientSound()` after `addFreshEntityWithPassengers` (javap-verified fresh, `EntityType` dump line 565); the Rust `ambient_sound` is only called from the dispenser path (game/dispenser.rs:985), never on spawn-egg placement (game/creatures.rs `use_spawn_egg`) — verified by a tree-wide call-site grep. **GAP — B7-G06 (minor, reproduces from 26.1.1).**
- **B35 (dispenser spawn egg):** vanilla `SpawnEggItemBehavior.execute` pops the `EntityType.spawn` result and UNCONDITIONALLY shrinks the stack by 1 and fires `ENTITY_PLACE` at the dispenser pos (javap-verified fresh, server-game.jar, `SpawnEggItemBehavior.txt:42-63`) — the egg is consumed and the event fires even when the spawn returns null. The 26.1.2 port (game/dispenser.rs:1019-1021) returns the stack unshrunk with no event on a null spawn (the 26.2 form). **GAP — B7-G18 (minor, reproduces from 26.1.1).** (The dispenser path calls `EntityType.spawn` directly with no peaceful gate in vanilla either — the port matches there.)

### §3.3 Block transforms (B12, B13, B14, B16) — 4 gap — RE-CHECK FIRST (B7-G07)

**B12 (axe strip/scrape/wax-off), B13 (hoe till), B14 (shovel flatten), B16 (copper chest scrape/wax-off):** vanilla 26.1.2 has NO `block_transformer` registry or component — the axe/hoe/shovel tables live in `AxeItem`/`HoeItem`/`ShovelItem` (`STRIPPABLES`, `TILLABLES`, `FLATTENABLES`; all three classes byte-identical to 26.1.1's, 0 `BlockTransformer` references in any dump) and the jar's datapack carries 0 `block_transformer` data files. The 26.1.2 port keeps the 26.2/26.3 form: `game.rs` `block_transform` reads the `minecraft:block_transformer` component from `items.json` (0/1,506 items carry it; horizon-items/src/items.rs:942) and returns `None` for every tool (game.rs:7932-7937), and `block_transform.rs` `Transformers::load` reads `data/minecraft/block_transformer/*.json` (0 files in the prepared data), so the transform table is empty. **All four GAP — B7-G07 (major, reproduces from 26.1.1; one of the twelve 26.1 regressions, re-checked first).** Honeycomb wax-on is dead the same way (`block_transform.rs:245` `waxed()` reads the empty axe table; callers game.rs:8323, game/remote.rs:8778) and is recorded as part of G07. The 26.1 tree hardcoded the correct tables — it is the repair reference.

- **G13/G14/G15 are FIXED behaviourally, masked by G07** (see §5.1): the shield check (game.rs:7940-7946), the `item_used_on_block` trigger (game.rs:7986), and the copper-chest neighbour code (game.rs:8032, called at :8014) are all still wrong but unreachable, because `block_transform` returns `None` before any of them runs. **All three return if G07 is fixed.**
- Side finding: vanilla `HoneycombItem` DOES reference `ITEM_USED_ON_BLOCK` (1×, javap-verified fresh), so G14's scope is hoe/shovel/campfire only — the port's honeycomb trigger is correct.

### §3.4 Campfire douse (B15) — GAP — RE-CHECK FIRST (B7-G08)

**B15 (shovel campfire douse):** vanilla 26.1.2 `ShovelItem.useOn` dowses a lit campfire directly (face != DOWN, `CampfireBlock.dowse` + level event 1009; `ShovelItem.txt:64-88`); there is no `#douses_campfires` tag in 26.1.2 (absent from `ItemTags`, the client jar's datapack and the prepared item tags — 186 files, no `douses_campfires.json`). The 26.1.2 port keeps the 26.3 form: game/campfire_douse.rs:12-17 gates the douse on `#minecraft:douses_campfires` (`CampfireBlock.useItemOn`), so `campfire_to_douse` always returns `None` and a shovel never puts out a campfire; the face != DOWN check was also dropped. **GAP — B7-G08 (major, reproduces from 26.1.1; one of the twelve 26.1 regressions, re-checked first).** Still wired in game/use_hand.rs:88. The 26.1 tree implemented the douse directly via `#minecraft:shovels`.

### §3.5 Chorus fruit (B18) — GAP

**B18 (chorus fruit teleport):** vanilla 26.1.2 `LivingEntity.randomTeleport` is blocksMotion-grounded (4-arg signature, no tag parameter, javap-verified fresh, `LivingEntity.txt:9930`). The Rust game/consume.rs:120-139 walks down for `#minecraft:entities_can_teleport_to`, a tag absent in 26.1.2 (absent from `BlockTags`, the client jar's datapack and prepared `block-tags.json`, 248 tags), so the loop always runs to `min_y` and returns false — chorus fruit never teleports. **GAP — B7-G09 (major, reproduces from 26.1.1).**

**Masked finding (no separate item):** the tree keeps the level-event 2017 directional-particle trail on the consume path (horizon-world/src/teleport_consume.rs `apply` + game/consume.rs:422-431 `directional_particles`; vanilla `TeleportRandomlyConsumeEffect` has only a `diameter` field and emits no level event — 0 level-event references in the fresh dump, `TeleportRandomlyConsumeEffect.txt`) — but it is masked by G09: the trail fires only after a successful teleport, which never happens on the player path. Fixing G09 without removing the trail would expose it. (The fox path already emits it: B3-G43, landed 26.1.2 Batch 3 REPRODUCES.)

### §3.6 Buckets (B2) — PASS

**B2 (bucket sneak-redirect):** vanilla 26.1.2 `BucketItem.emptyContents` HAS the sneak check (`isShiftKeyDown`, class byte-identical to 26.1.1's; javap-verified fresh, `BucketItem.txt:231`), and the Rust `bucket_empty` (horizon-world/src/level/buckets.rs — file byte-identical to the 26.1.1 tree's) matches: air always takes the fluid, retry beside with a null hit result. **PASS** (1.21.7 G02 stays withdrawn — see §5.2).

### §3.7 Potions (B24, B26) — 2 gap

- **B24 (water splash/lingering douse):** vanilla 26.1.2 `AbstractThrownPotion` has ZERO `PotionTags` references — it gates on `PotionContents.is(Potions.WATER)` directly (class byte-identical to 26.1.1's; javap-verified fresh). 26.1.2's `PotionTags` class has only `TRADEABLE` and prepared `tags/potion/` contains only `tradeable.json`. The Rust gates everything on potion tags (game/splash_water.rs:42-50 `potion_in_tag`: `minecraft:douses_fire`, `hurts_water_sensitive_entities`, `extinguishes_entities`, `rehydrates_axolotls` — file byte-identical to the 26.1.1 tree's), which always return false, so water splash/lingering bottles do nothing on impact. **GAP — B7-G10 (major, reproduces from 26.1.1; duplicate of B1-G03 — coordinator to fold; landed 26.1.2 Batch 1: B1-G03 REPRODUCES).**
- **B26 (water bottle on dirt → mud):** vanilla 26.1.2 `PotionItem.useOn` uses `BlockTags.CONVERTABLE_TO_MUD` (with an A, javap-verified fresh, `PotionItem.txt:55`); the Rust uses `convertible_to_mud` (with an I) in all four places (game/potions.rs:710, game/dispenser.rs:2245, game/remote.rs:8658, horizon-world/src/level/data.rs:460), so the lookup always misses and a water bottle on dirt never makes mud. **GAP — B7-G11 (minor, reproduces from 26.1.1; duplicate of B5-G19 — coordinator to fold; landed 26.1.2 Batch 5: B5-G19 REPRODUCES).**

### §3.8 Boats (B27) — GAP

**B27 (boat placement eye check):** vanilla 26.1.2 `BoatItem.use` gathers entities in the player's bbox expanded 5.0 along the view + inflate 1.0 and returns PASS when the eye is inside any entity's pick-inflated bbox (class byte-identical to 26.1.1's; javap-verified fresh, `BoatItem.txt:47-72`). The Rust `boat_to_place` (game/boats.rs:1517-1542 — file byte-identical to the 26.1.1 tree's) only checks the boat's own bbox, so a boat is placeable while the player's eye is inside an entity. **GAP — B7-G12 (minor, reproduces from 26.1.1).**

### §3.9 Ender pearl / wind charge cooldown (B30) — PASS

**B30 (throw cooldown):** vanilla 26.1.2 `ItemStack.use` calls `applyAfterUseComponentSideEffects` for instant-use items on Success, which reads `USE_COOLDOWN` — pearls (1.0s) and wind charges (0.5s) DO get a cooldown (javap-verified fresh, `ItemStack.txt:532-570`). The 26.1.2 Rust still applies it: `throw_item` (game/creatures.rs:7379 wind charge, :7427 pearl) calls `apply_use_cooldown` (game.rs:5211), which reads the `use_cooldown` component. **PASS — B7-G16 stays FIXED (as in 26.1.1).**

### §3.10 Spear (B33) — GAP (cited, not minted)

**B33 (spear stab knockback):** vanilla 26.1.2 `LivingEntity.stabAttack` calls `causeExtraKnockback` ONCE with `0.4f + getKnockback(...)` (javap-verified fresh, `LivingEntity` dump lines 7700-7764). The Rust applies it as two pushes (mobs/spear_use.rs:272, 275). **GAP — already recorded as B3-G45; cited, not re-minted** (landed 26.1.2 Batch 3: B3-G45 REPRODUCES).

### §3.11 Remaining behaviour checks — all PASS

- **B3 (food consume):** consume path unchanged from 26.1.1 (byte-identical file); food components match D1. **PASS.**
- **B4 (tool use):** tool components match D4/D5; tool use paths unchanged. **PASS.**
- **B5 (armour equip):** armour data matches D7; equip path unchanged. **PASS.**
- **B7 (fluid use / consume / offspring / entity_data):** unchanged code; fluid use matches vanilla. **PASS.**
- **B17 (bundle use):** `game/bundles.rs` byte-identical to the 26.1.1 tree's. **PASS.**
- **B19 (fishing rod):** `game/fishing.rs` byte-identical to the 26.1.1 tree's. **PASS.**
- **B20 (crossbow):** `game/crossbow.rs` byte-identical to the 26.1.1 tree's. **PASS.**
- **B21 (shield block):** `game/shield.rs` byte-identical to the 26.1.1 tree's. **PASS.**
- **B22 (trident throw):** `game/trident.rs` byte-identical to the 26.1.1 tree's. **PASS.**
- **B23 (mace smash):** `game/mace.rs` byte-identical to the 26.1.1 tree's. **PASS.**
- **B25 (item use order):** `game/item_use.rs` byte-identical to the 26.1.1 tree's. **PASS.**
- **B28 (mount menu):** `game/mount_menu.rs` byte-identical to the 26.1.1 tree's. **PASS.**
- **B29 (world commands item paths):** `game/commands/world_cmds.rs` byte-identical to the 26.1.1 tree's. **PASS.**
- **B31 (composter — cited):** composter is dead in 26.1.2 (`compost_chance` removed; the port reads `minecraft:compostable`, which 0 items carry) — already recorded as **B5-G22**, cited, not re-minted (landed 26.1.2 Batch 5: B5-G22 REPRODUCES). **PASS for this batch's tally** (the defect is Batch 5's; this batch does not double-count it).
- **B32 (villager food):** the villager-food component form was checked by Batch 3 and found harmless in 26.1.1; no mobs/villager file differs in 26.1.2, and Batch 4's landed 26.1.2 report confirms vanilla 26.1.2's `DataComponents` has no `villager_food` and 0/1,506 items carry it. **PASS.**
- **B34 (nautilus armour):** nautilus armour exists in 26.1.2 (5 items in `items.json`, present in the jar's `Items` class); armour path matches D7. **PASS.**

### §3.12 Initial / respawn spawn (B36) — GAP

**B36 (spawn in skylight-less dimensions):** vanilla 26.1.2 `PlayerSpawnFinder.findSpawn` checks `dimensionType().hasSkyLight()` FIRST: in a dimension without skylight (or in adventure mode) it returns `fixupSpawnHeight` at the suggestion at once, with no radius search (javap-verified fresh, server-game.jar, `PlayerSpawnFinder.txt`, `net.minecraft.server.level.PlayerSpawnFinder`). The 26.1.2 port (game/initial_spawn.rs:294-298 `spawn_finder`) dropped the `!sky_light` arm, so in the Nether/End it searches the respawn radius instead of returning the fixup height. **GAP — B7-G19 (minor, reproduces from 26.1.1).**

## §4 Defect list

### Gap items (12)

| ID | Sev | Title |
|---|---|---|
| B7-G01 | minor | `ComponentMap::set` errors on a dangling removal marker; vanilla returns null |
| B7-G03 | minor | Spawn egg `entity_data` id type override ignored |
| B7-G06 | minor | No `Mob.playAmbientSound()` on spawn-egg placement |
| B7-G07 | major | Axe/hoe/shovel transforms and honeycomb wax-on entirely non-functional (26.2/26.3 `block_transformer` form) |
| B7-G08 | major | Shovel campfire douse dead: gated on the absent `#douses_campfires` tag |
| B7-G09 | major | Chorus fruit never teleports (tag `#entities_can_teleport_to` absent in 26.1.2) |
| B7-G10 | major | Water splash/lingering bottles do nothing on impact (potion-tag gate dead) — duplicate of B1-G03 |
| B7-G11 | minor | Water bottle on dirt never makes mud (`convertible` vs `convertable`) — duplicate of B5-G19 |
| B7-G12 | minor | Boat placement missing the eye-inside-entity PASS check |
| B7-G17 | minor | Op's spawn-egg `entity_data` applied to spawners (26.1.2 has no `Spawner.setEntityData`) — duplicate of B3-G39 |
| B7-G18 | minor | Dispenser spawn egg returned unshrunk with no `ENTITY_PLACE` on a null spawn |
| B7-G19 | minor | Respawn/initial spawn searches the radius in skylight-less dimensions instead of the fixup height |

### Fixed items (5)

| ID | Sev | Title |
|---|---|---|
| B7-G04 | minor | Peaceful gate for hostile spawn eggs (26.1 defect fixed in 26.1.1; stays fixed) |
| B7-G13 | minor | Shield blocking-intent check (masked by B7-G07; code still wrong) |
| B7-G14 | minor | `ITEM_USED_ON_BLOCK` trigger for hoe/shovel/campfire (masked by B7-G07/B7-G08; code still wrong) |
| B7-G15 | minor | Copper-chest neighbour `BLOCK_CHANGE` (masked by B7-G07; code still wrong) |
| B7-G16 | minor | Ender pearl / wind charge use cooldown applied on throw (stays fixed) |

### Closed items (2)

| ID | Sev | Title |
|---|---|---|
| B7-G02 | minor | [CLOSED] Bucket sneak-redirect: vanilla 26.1.2 HAS the sneak check |
| B7-G05 | minor | [CLOSED] Spawn-egg `ENTITY_PLACE` position: vanilla posts at the spawn position |

### Oracle-defect items (30)

All 30 reproduce (26.1.2 Batch 0 rows identical to the 26.1.1 run; root causes javap-verified fresh — see §6).

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

### §5.1 26.1.1 items (49, PRIMARY)

Every item was re-decided fresh against the 26.1.2 jar and tree. All 30 in-scope files and all 66 in-scope vanilla classes are byte-identical to 26.1.1's, so every verdict repeats — with fresh 26.1.2 evidence cited in §3/§6, not copied verdicts.

| ID | 26.1.1 verdict | 26.1.2 verdict | Notes |
|---|---|---|---|
| G01 | gap | **REPRODUCES** | `component_patch.rs` byte-identical; vanilla `set` returns null |
| G02 | closed | **WITHDRAWN** | Vanilla 26.1.2 `BucketItem` HAS the sneak check; closed item |
| G03 | gap | **REPRODUCES** | `use_spawn_egg` still derives kind from the item name |
| G04 | fixed | **FIXED** | Gate still present (creatures.rs:7511-7535); hoglin residual is B3-G04 |
| G05 | closed | **WITHDRAWN** | Vanilla posts `ENTITY_PLACE` at the spawn position; closed item |
| G06 | gap | **REPRODUCES** | `ambient_sound` still dispenser-only |
| G07 | gap | **REPRODUCES** | Re-check-first regression; transform table still empty |
| G08 | gap | **REPRODUCES** | Re-check-first regression; `#douses_campfires` gate still dead |
| G09 | gap | **REPRODUCES** | Tag absent; consume path unchanged; 2017 trail still masked |
| G10 | gap | **REPRODUCES** | `splash_water.rs` byte-identical; duplicate of B1-G03 |
| G11 | gap | **REPRODUCES** | All four `convertible_to_mud` sites present; duplicate of B5-G19 |
| G12 | gap | **REPRODUCES** | `boats.rs` byte-identical |
| G13 | fixed | **FIXED (masked)** | Code still wrong; unreachable behind G07 — returns if G07 is fixed |
| G14 | fixed | **FIXED (masked)** | Code still wrong; unreachable behind G07/G08 — returns if those are fixed |
| G15 | fixed | **FIXED (masked)** | Code still wrong; unreachable behind G07 — returns if G07 is fixed |
| G16 | fixed | **FIXED** | Cooldown still applied (creatures.rs:7379, 7427) |
| G17 | gap | **REPRODUCES** | Both spawner arms present; duplicate of B3-G39 |
| G18 | gap | **REPRODUCES** | Null spawn still returns the stack unshrunk, no event |
| G19 | gap | **REPRODUCES** | `!sky_light` arm still missing from `spawn_finder` |
| O01–O30 | oracle-defect | **REPRODUCE** | Batch 0 rows identical; root causes javap-verified fresh (see §6) |

### §5.2 1.21.7 items (12)

| ID | 26.1.1 verdict | 26.1.2 verdict | Notes |
|---|---|---|---|
| G01 | reproduces | **REPRODUCES** | Same defect as 26.1.1 G01 (one item) |
| G02 | withdrawn | **WITHDRAWN** | Vanilla 26.1.2 `BucketItem` HAS the sneak check; closed item |
| G03 | reproduces | **REPRODUCES** | Same defect as 26.1.1 G03 (one item) |
| G04 | fixed | **FIXED** | Vanilla HAS the gate and the port still HAS it; same item as 26.1.1 G04 |
| G05 | withdrawn | **WITHDRAWN** | Vanilla posts `ENTITY_PLACE` at the spawn position; closed item |
| G06 | reproduces | **REPRODUCES** | Same defect as 26.1.1 G06 (one item) |
| G07 | reproduces | **REPRODUCES** | Same defect as 26.1.1 G07 (one item); re-check-first regression |
| G08 | reproduces | **REPRODUCES** | Same defect as 26.1.1 G08 (one item); re-check-first regression |
| G09 | reproduces | **REPRODUCES** | Same defect as 26.1.1 G09 (one item) |
| G10 | reproduces | **REPRODUCES** | Same defect as 26.1.1 G10 (one item) |
| G11 | reproduces | **REPRODUCES** | Same defect as 26.1.1 G11 (one item) |
| G12 | reproduces | **REPRODUCES** | Same defect as 26.1.1 G12 (one item) |

### §5.3 26.2 candidates (4)

| ID | 26.1.1 verdict | 26.1.2 verdict | Notes |
|---|---|---|---|
| G13 | fixed (masked) | **FIXED (masked)** | See §5.1 |
| G14 | fixed (masked) | **FIXED (masked)** | See §5.1; honeycomb trigger scope confirmed (vanilla `HoneycombItem` DOES reference the trigger) |
| G15 | fixed (masked) | **FIXED (masked)** | See §5.1 |
| G16 | fixed | **FIXED** | Cooldown still applied (creatures.rs:7379, 7427) |

## §6 Oracle triage

All 30 failing item-family oracles are **IDENTICAL** in the 26.1.2 Batch 0 table (same class, error count, first error) to the 26.1.1 run — the whole table is row-identical (0/391 rows differ) — all 36 passing ones pass identically, and there is **no new item-family failing oracle** (`evidence/oracle-triage-26.1.2.txt`). All 30 root causes were javap-verified fresh against 26.1.2 dumps:

- **O01/O08/O09** — `EntityTypes` does not exist in the 26.1.2 client jar (registration still in `EntityType`).
- **O02** — `ClockInstance` does not exist in the 26.1.2 client jar (32 errors; cascade root for O29/O30).
- **O03–O07** — `Level.getNextEntityId` does not exist in 26.1.2 (0 matches in the `Level` dump).
- **O10** — `EntitySpawnRequest` does not exist. **O11** — `ServerboundPunchPacket` does not exist. **O28** — `SwingDescription` does not exist anywhere in the jar and `LivingEntity` has no `getCurrentSwing`.
- **O12** — `InteractionResult.Success` is a record with a `swingSource` field; no `shouldSwing()` method (`InteractionResult_Success.txt:3,81`).
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

Passing in-scope oracles: the same 36 as 26.1.1, including `VanillaItemCooldownsOracle` and `VanillaComponentPatchOracle`.

## §7 Driver inventory

74 drivers — **32 runnable / 25 blocked / 17 no-oracle** — byte-identical to 26.1.1's inventory (`evidence/driver_inventory_26_1_2.py`, `evidence/driver-inventory-26.1.2.json`; the only mismatch is `interaction_input` referencing the `ParityData` helper). **No driver was run**, per the brief; the 32 runnable drivers are recorded as not run, not as passes.

## §8 Systemic findings

1. **26.1.2 is a true delta port of 26.1.1 in B7 scope — nothing changed.** All 30 in-scope files byte-identical, all 66 in-scope vanilla classes byte-identical, the Batch 0 table row-identical, the data checks byte-identical in output. The 26.1.1 tree's stale-re-port defects (G07, G08, G18, G19 — one-method reversions to the 26.2/26.3 form) carry forward verbatim, and the 26.1 tree remains the repair reference for all four.
2. **Absent-tag/component reads are unchanged in scope.** The 26.1.2 tree reads the same absent tags/components as 26.1.1 did: `#douses_campfires`, `#entities_can_teleport_to` (in two places — chorus fruit and enderman), `#enderman_does_not_teleport_to`, `#consumable_does_not_teleport_to`, the `minecraft:block_transformer` and `minecraft:mob_visibility` components. The tag table resolves absent tags to empty sets, so every read silently degrades to false, never an error.
3. **Masked defects are still a trap for fixers.** G13/G14/G15 (and the player 2017 trail under G09) are behaviourally fixed only because G07/G08/G09 kill the whole path. Any fix of G07, G08 or G09 must also fix the masked code or the defects reappear.
4. **26.1.1-record corrections (from this run):** (a) the 26.1.1 report's §1 item→class table sums to 1,530 over 57 rows and lists classes the jar never constructs (see §1 — the true extraction is in its own evidence `.out`, byte-identical to this run's); (b) the 26.1.1 report's "187 item tag files" is a miscount — both versions' prepared data have 186 (the jar carries 203 in both); (c) the 26.1.1 `javap_dumps.sh` lacked the `PlayerSpawnFinder` and `TeleportRandomlyConsumeEffect` dumps its report cites — the correct packages are `net.minecraft.server.level.PlayerSpawnFinder` and `net.minecraft.world.item.consume_effects.TeleportRandomlyConsumeEffect`; this run's script dumps both.
5. **26.1.2 deltas adjacent to scope (all out of scope, noted for the coordinator):** (a) the protocol layer dropped the synthetic sign-text component kinds (`horizon-protocol/src/item.rs`, `item/skip.rs`, `horizon-client/src/item_sync.rs`) — Batch 4's landed 26.1.2 report covers this as the LAN-narrowing facet of B4-G33; the local pick/place defect in `block_entity_item.rs` (byte-identical) is unchanged. (b) The `#4 handleAttack` delta (spectator attack gate, `lan/hub.rs:4063,4083`) is LAN attack handling, not item behaviour — Batch 3/9's scope. (c) The `#6 Checkbox tooltip` delta (`dialog_screen.rs`) is Batch 10's scope. (d) `item_sync.rs` changed a test-only painting/variant registry id (102→109); no behaviour change.
6. **Out-of-scope flags (cited, not minted):** sign components → Batch 4 (B4-G33, landed 26.1.2, REPRODUCES with the LAN narrowing noted above); enderman teleport → Batch 3 (B3-G14); spear knockback → Batch 3 (B3-G45, landed REPRODUCES); mob-head visibility → Batch 3 (B3-G33, landed REPRODUCES — vanilla `getVisibilityPercent` hardcodes five head/type pairs INCLUDING `PIGLIN_BRUTE + PIGLIN_HEAD` with no clamp, re-verified fresh at `LivingEntity.txt:2047-2177`); hoglin in peaceful → Batch 3 (B3-G04, landed REPRODUCES); composter → Batch 5 (B5-G22, landed REPRODUCES); swing packet + `SwingState` rework → Batch 9/10 (the local player's item-use swings no longer send `ServerboundSwingPacket`; vanilla `LocalPlayer.swing` sends it unconditionally, javap-verified fresh at `LocalPlayer.txt:677-687`); dispenser shear alive-check → Batch 10; `data.rs` cobweb/`blocks_motion` changes → Batch 3/Batch 1; `#entities_can_teleport_to` at `game/commands/world_cmds.rs:2141-2146` → Batch 8 (commands). The 26.2 run's out-of-scope observations were NOT re-verified for 26.1.2 and are not carried forward as claims.

## §9 Method notes + evidence index

**Method.** Static analysis only: `javap -c -p` on the named 26.1.2 jars (`client.jar` for client classes, `server-game.jar` for server classes) for every vanilla method in scope; Rust source reads; prepared-data checks (`items.json`, `block-tags.json`, jar data files); and a full diff of every in-scope source file between the 26.1.1 and 26.1.2 trees. A sha1 byte-compare of 66 in-scope vanilla classes between the 26.1.1 and 26.1.2 jars found 0 diffs. No drivers were run and no oracle repairs were done (checks + triage only).

**Evidence** (`~/workspace/mcir-work/sweep-26.1.2-batches/b07/evidence/`):

- `tree-diff-list.txt` — 24 files differ between `versions/26.1.1/crates` and `versions/26.1.2/crates`.
- `in-scope-diff.txt` — all 30 in-scope files byte-identical.
- `jar_compare_26_1_2.py` + `jar-compare-26.1.2.log` — 66/66 in-scope classes byte-identical; absence + datapack-tag checks.
- `javap_dumps.sh`, `javap/` — 61 fresh 26.1.2 javap dumps (incl. `PlayerSpawnFinder` and `TeleportRandomlyConsumeEffect`, which the 26.1.1 script lacked).
- `parse_items_class_26_1_2.py/.out`, `item-class.json`, `nonblock-by-class.json` — D8 map.
- `data_checks_26_1_2_v2.py/.out`, `materials-26.1.2.json` — D1–D7.
- `driver-map-26.2.txt`, `driver_inventory_26_1_2.py/.out`, `driver-inventory-26.1.2.json` — driver inventory.
- `oracle-triage-26.1.2.txt` — oracle triage vs Batch 0 (0/391 rows differ from 26.1.1).

**Key jar evidence** (all from fresh 26.1.2 dumps in `b07/evidence/javap/`): `SpawnEggItem.txt:126-153` spawnMob peaceful gate; `:256-274` getType reads ENTITY_DATA; `EntityType.txt:565` playAmbientSound in spawn; 37 `notInPeaceful` types extracted (no hoglin). `LivingEntity.txt:9930` randomTeleport 4-arg `(DDDZ)`; `:3340` blockUsingItem 2-arg; `:10378` onEquippedItemBroken `(Item, EquipmentSlot)`; `:2047-2177` getVisibilityPercent (5 pairs incl. PIGLIN_BRUTE, no clamp); `:7700-7764` stabAttack single causeExtraKnockback `0.4f + getKnockback`. `ItemStack.txt:723` hurtAndBreak `Consumer<Item>`; `:532-570` applyAfterUseComponentSideEffects reads USE_COOLDOWN; `Player.txt:1128` drop 2-arg; `ServerChunkCache.txt:39` 11-arg ctor; `Gui.txt:179` 1-arg ctor; `ItemStackTemplate.txt:74` only `fromNonEmptyStack`; `VanillaRegistries.txt:39` only `createLookup`; `InteractionResult_Success.txt:3,81` record with `swingSource`, no `shouldSwing`; `Level.txt` 0 `getNextEntityId`; `InteractionResult.txt` SwingSource CLIENT/SERVER/NONE. `SpawnEggItemBehavior.txt:42-63` execute pops spawn result, unconditionally shrink + ENTITY_PLACE; `PlayerSpawnFinder.txt` findSpawn skylight gate first; `TeleportRandomlyConsumeEffect.txt` only `diameter` field, apply calls 4-arg randomTeleport + game event + sound, 0 level-event refs; `LocalPlayer.txt:677-687` swing sends ServerboundSwingPacket; `BaseSpawner.txt:77` and `TrialSpawnerBlockEntity.txt:83` only `setEntityId` (0 `setEntityData` in any dump); `AxeItem.txt:3` STRIPPABLES, `HoeItem.txt:3` TILLABLES, `ShovelItem.txt:3` FLATTENABLES, 0 `BlockTransformer` references in any of the three; `BucketItem.txt:231` isShiftKeyDown; `ShovelItem.txt:64-88` direct campfire dowse; `AbstractThrownPotion.txt` 0 `PotionTags` refs, `Potions.WATER` at :74; `PotionItem.txt:55` CONVERTABLE_TO_MUD; `BoatItem.txt:47-72` eye-inside-entity check; `PotionTags.txt:3` only TRADEABLE.

## §10 What this batch defers + verdict

**Deferred (not this batch's work):** driver runs (32 runnable drivers unrun), oracle repairs (30 failing oracles), all fixes (including the four re-port defects G07/G08/G18/G19, whose repair reference is the 26.1 tree), and the out-of-scope flags in §8.6.

**Verdict: 44 checks — 28 pass, 16 gap.** The 26.1.2 Items tree is the 26.1.1 tree in every file and class this batch touches: all 12 gap items reproduce with fresh 26.1.2 evidence, the 5 fixed and 2 closed items re-decide the same way, all 30 oracle defects reproduce, and no new behaviour defect was found in scope. The two re-check-first regressions (B7-G07 tool transforms, B7-G08 campfire douse) both reproduce. Three reproductions are duplicates of landed batches (B1-G03, B5-G19, B3-G39) for the coordinator to fold.
