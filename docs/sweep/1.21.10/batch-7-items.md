# Batch 7 — Items (Minecraft 1.21.10)

**Scope:** item use, food, tools, armour, buckets, spawn eggs, components — 1,488 items.

**Result: 44 checks — 30 pass, 14 gap** (as-tallied 44 = 44 behaviour + 0 non-behaviour). 11 gap items (4 major, 7 minor), 6 fixed, 2 closed, 29 oracle-defect items (2 major, 27 minor), 1 oracle fixed. The headline: the 1.21.10 Items tree is a fresh full port of the 1.21.11 era template, not a 26.x re-port — and it fixed two things the 26.x trees never did (B7-G01 component_patch, B7-O11 KnockbackClient) — but the four re-check-first regressions (G07 tool transforms, G08 campfire douse, G09 chorus fruit, G10 splash water) all REPRODUCE, as do seven more 1.21.7/26.1.2 defects. No new behaviour defect was found in scope. Two 1.21.7 items (G02 bucket sneak, G05 ENTITY_PLACE) are WITHDRAWN because vanilla 1.21.10 changed: the bucket sneak check and the spawn-position ENTITY_PLACE both exist in 1.21.10's jar, where 1.21.7 had neither.

## §0 Execution status

- **Checks: 44 total, 30 pass, 14 gap — all behaviour, 0 non-behaviour.** Oracle triage is not tallied (it is not a driver check).
- **Items: 49** — 11 gap (4 major, 7 minor), 6 fixed, 2 closed, 29 oracle-defect (2 major, 27 minor), 1 oracle fixed (B7-O11).
- **No drivers were run and no oracle repairs were done** (checks + triage only, per the brief). No UNVERIFIED carry-overs: every mandatory re-check (12 from 1.21.7, 49 from 26.1.2) was decided fresh against 1.21.10's jar and tree.
- **Batch 0 oracle table has NOT landed** for 1.21.10 (`docs/sweep/1.21.10/` holds only the inventory + `measure-oracles.py`). Oracle triage is therefore PROVISIONAL from the 26.1.2 measured table (391 rows), with every root cause re-verified two ways: the oracle source in the 1.21.10 tree still references the API, and the API is absent/different in 1.21.10's own jar. The driver inventory (§7) inherits the same provisional status. The one correction the provisional table needed — KnockbackClient fixed in 1.21.10 — was applied and is javap-verified, not assumed.
- **Oracle sources vs 26.1.2:** 20 of the 30 in-scope oracle files are byte-identical to the 26.1.2 tree's; 10 differ only by the 1.21.x naming adaptation (`Identifier` → `ResourceLocation`, `key().identifier()` → `key().location()`, `RegistryLayer.WORLDGEN` → `WORLD`). The root-cause references survive in all 10.
- **Tree vs 26.1.2:** the 1.21.10 tree is a template-derived delta port of 1.21.11 (605 files differ from the template), NOT derived from 26.1/26.1.1/26.1.2 — no byte-identity argument is available or used. Every verdict below rests on fresh 1.21.10 jar + tree evidence.
- **Worktree state:** the 1.21.10 tree's `.local` (prepared data) is present; jar sha1s match the brief pins (client-named `715f5816…`, server-game-named `ba1fc89e…`, client `d3bdf582…`, bundler server `95495a7f…`).
- **B9 (peaceful gate) is counted as PASS** in the tally: vanilla 1.21.10 HAS the gate and the port HAS it (B7-G04 FIXED). The hoglin residual (hoglin wrongly in the port's not-allowed list) is Batch 3's B3-G04 facet and lives in Batch 3's tally, not this batch's (flagged in §8.6; the 1.21.10 Batch 3 report has not landed).

## §1 Item → class map

Same extraction as 1.21.7/26.1.2, re-run against the 1.21.10 jar (`evidence/parse_items_class_1_21_10.py`, output `evidence/parse_items_class_1_21_10.out`): **1,488 items mapped to 70 classes, 0 unmapped, 0 mismatches vs `block-behaviours.json`**. Block items: 1,025; non-block items: 463. No spears and no nautilus armour exist in 1.21.10 (26.x-only items) — swords, tools and armour are plain `Item` with components, as in 1.21.7.

**Parser fix vs the 1.21.7 script (recorded, not a defect):** the 1.21.7 parser's putstatic split only matched `:Lnet/minecraft/world/item/Item;` fields, so the three `WeatheringCopperItems` collection fields (`COPPER_BARS`/`COPPER_CHAIN`/`COPPER_LANTERN`) were absorbed into the following segments — `IRON_CHAIN`'s segment mis-attributed to `copper_bars` and `GLASS_PANE`'s to `copper_chain`, leaving 24 items unmapped and 26 mismatches. The fix splits on both field types and expands the three collections (bars/chain/lantern × 8 oxidation/wax states = 24 items) as `BlockItem`, justified by bootstrap entry 26: the collection factory is `Items.registerBlock`. After the fix: 0 unmapped, 0 mismatches, 0 multi-class segments.

| Class | Count | Class | Count |
|---|---|---|---|
| `net.minecraft.world.item.BlockItem` | 909 | `net.minecraft.world.item.EggItem` | 3 |
| `net.minecraft.world.item.Item` | 222 | `net.minecraft.world.item.PlaceOnWaterBlockItem` | 2 |
| `net.minecraft.world.item.SpawnEggItem` | 83 | `net.minecraft.world.item.FoodOnAStickItem` | 2 |
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

The 46 singletons include `BowItem`, `CrossbowItem`, `TridentItem`, `MaceItem`, `ShieldItem`, `FishingRodItem`, `EnderpearlItem`, `WindChargeItem`, `PotionItem`, `SplashPotionItem`, `LingeringPotionItem`, `HoneycombItem`, `FlintAndSteelItem`, `ShearsItem`, `CompassItem`, `SpyglassItem`, `SolidBucketItem`, `AirItem` (full list in `evidence/parse_items_class_1_21_10.out`). Copper golem spawn egg maps to `SpawnEggItem`; ominous bottle and trial keys map to plain `Item`.

## §2 Data checks

All 8 pass (`evidence/data_checks_1_21_10.py`, output `evidence/data_checks_1_21_10.out`; materials extracted fresh from the 1.21.10 jar into `evidence/materials-1.21.10.json` — 7 tool materials incl. COPPER [190, 5.0, 1.0, 13], 9 armour materials incl. COPPER [11, 1/3/4/2/4, 8]).

- **D1 (food):** 40 foods in `items.json`, all with the expected food components (checked against the jar's `FoodProperties`). **PASS.** Note: the adapted 26.1.2 script initially flagged the 4 fish buckets as missing food components — that is a 26.x script assumption, not a data defect: vanilla 1.21.10's mob-bucket segments carry no `FOOD` component (only `BUCKET_ENTITY_DATA`; `MobBucketItem` has 0 `Foods` references), so the prepared data is correct to omit it.
- **D2 (durability):** 77 items have a `max_damage` component; values match the jar for every item (26.1.2 had 84 — the 7 spears account for the difference). **PASS.**
- **D3 (max stack size):** 1,488 items — every `max_stack_size` matches the jar. **PASS.**
- **D4 (tool pairs):** 35 tool pairs (7 materials × 5 tool kinds) with matching `tool`/`attack_damage`/`attack_speed` components; 0 spears, 0 spear kinetic values (none exist in 1.21.10). **PASS.**
- **D5 (tool components):** 35 tool components, matching the jar's `Tool` definitions (rules, damage per block, mining speed). **PASS.**
- **D6 (tags):** 13 tool-rule tags checked against the jar's datapack — all present in `block-tags.json` with expanded contents matching exactly. **PASS.**
- **D7 (armour):** 35 armour items; material, defence, toughness, knockback resistance, enchant value and durability match the jar's ArmorMaterials. **PASS.**
- **D8 (item→class map):** 1,488 items → 70 classes, 0 unmapped, 0 mismatches vs `block-behaviours.json` (see §1). **PASS.**

Prepared-data spot checks (1.21.10 `.local`): `items.json` has 1,488 items; **0 carry `minecraft:block_transformer`** (the component the transform path reads — see G07), **0 carry `minecraft:mob_visibility`**, and **0 carry `minecraft:compostable`** (see B31). `block-tags.json` has 203 keys; `#entities_can_teleport_to`, `#blocks_motion`, `#enderman_does_not_teleport_to`, `#consumable_does_not_teleport_to` are all ABSENT; `#convertable_to_mud` is present (with an A) and `#convertible_to_mud` is absent. Item tags: **172 files** in prepared data (the jar itself carries 190); `shovels.json` exists; `douses_campfires.json` is absent from both. No `tags/potion/` directory and no `block_transformer/` directory in prepared data; the jar's datapack likewise has 0 `block_transformer` data files and 0 `tags/potion/` files.

## §3 Behaviour checks

### §3.1 Component map (B1) — PASS (FIXED in 1.21.10)

**B1 (component map set/remove):** vanilla 1.21.10 `PatchedDataComponentMap.set` returns `previous.orElse(prototype)` — null on a dangling removal marker, never an error (javap-verified fresh, `PatchedDataComponentMap.txt:131-163`). The 1.21.10 Rust `ComponentMap::set` (horizon-world/src/component_patch.rs:109-122) now returns `Ok(base)` on a dangling removal marker — the port fixed it during the behaviour port, with a doc comment citing 1.21.10's form explicitly ("In 1.21.10 an unsanitized removal with no prototype returns null, not an error"). **PASS — B7-G01 FIXED** (was a gap in 1.21.7 and all 26.x runs).

### §3.2 Spawn eggs (B6, B8, B9, B10, B11, B35) — 2 pass, 4 gap

- **B6 (spawn egg on spawner):** vanilla `SpawnEggItem.useOn` calls only `Spawner.setEntityId` — the 1.21.10 jar has NO `Spawner.setEntityData` (0 matches in the `BaseSpawner` and `TrialSpawnerBlockEntity` dumps; `BaseSpawner` has only `setEntityId`, javap-verified fresh). The Rust applies an op's egg `entity_data` to spawners (game/spawner.rs:464 `set_entity_data` for regular spawners; :450 `override_entity_data` for trial spawners). **GAP — B7-G17 (minor, reproduces from 26.1.2; suspected duplicate of B3-G39 — coordinator to fold; the 1.21.10 Batch 3 report has not landed).**
- **B8 (spawn egg entity_data id override):** vanilla `SpawnEggItem.getType` reads the typed `entity_data` component's `type()` and spawns THAT type (javap-verified fresh: `getType` reads `DataComponents.ENTITY_DATA`, `TypedEntityData.type()`; `SpawnEggItem.txt:283-303`). The Rust derives the kind from the item name only (game/creatures.rs:7461 `use_spawn_egg`, `trim_end_matches("_spawn_egg")`) and ignores an `entity_data` id override naming a different type. **GAP — B7-G03 (minor, reproduces from 1.21.7 and 26.1.2).**
- **B9 (spawn egg peaceful gate):** vanilla 1.21.10 `SpawnEggItem.spawnMob` FAILs hostile eggs in peaceful (`!isAllowedInPeaceful && difficulty == PEACEFUL`, javap-verified fresh, `SpawnEggItem.txt:146-180`) — vanilla changed between 1.21.7 (no gate) and 1.21.10 (gate). The 1.21.10 tree has the gate: game/creatures.rs:7442-7450 `spawn_egg_allowed` plus the `use_failed` return at :7462-7465, with FAIL semantics (`use_failed` means the action is not consumed, game/lan_actions.rs:2536-2539). **PASS — B7-G04 FIXED.** Residual: the shared `allowed_in_peaceful` denylist wrongly includes hoglin (36 vanilla `notInPeaceful` types extracted fresh from the 1.21.10 `EntityType` dump, no hoglin; port denylist at mobs/spawn_rules.rs, `S::Hoglin` in the list — plus a dead `S::Parched` entry, a 26.x-only species) — recorded as **B3-G04**, which also covers the spawn-egg facet; it lives in Batch 3's tally, not this batch's.
- **B10 (spawn egg ENTITY_PLACE position):** vanilla 1.21.10 posts `ENTITY_PLACE` at the spawn position (`spawnMob` posts at its `BlockPos` argument, the computed spawn pos; `SpawnEggItem.txt:182`), and the Rust posts at `p` (the spawn pos, game/creatures.rs:7516). **PASS** (1.21.7 G05 WITHDRAWN — vanilla changed between 1.21.7, which posted at the clicked block, and 1.21.10; see §5.2).
- **B11 (spawn egg ambient sound):** vanilla `EntityType.spawn` calls `Mob.playAmbientSound()` after `addFreshEntityWithPassengers` (javap-verified fresh, `EntityType` dump line 557); the Rust `ambient_sound` is only called from the dispenser path (game/dispenser.rs:980) and the tick-time path (mobs.rs:6492), never on spawn-egg or mob-bucket placement (game/creatures.rs `use_spawn_egg`) — verified by a tree-wide call-site grep. **GAP — B7-G06 (minor, reproduces from 1.21.7 and 26.1.2).**
- **B35 (dispenser spawn egg):** vanilla 1.21.10 has no named `SpawnEggItemBehavior` class — the behaviour is the anonymous `DispenseItemBehavior$1`, whose `execute` pops the `EntityType.spawn` result and UNCONDITIONALLY shrinks the stack by 1 and fires `ENTITY_PLACE` at the dispenser pos (javap-verified fresh, server-game.jar, `DispenseItemBehavior-1.txt:69-116`) — the egg is consumed and the event fires even when the spawn returns null. (Only a null `getType` returns the stack unchanged.) The 1.21.10 port (game/dispenser.rs:1025-1027) returns the stack unshrunk with no event on a null spawn. **GAP — B7-G18 (minor, reproduces from 26.1.2).** (The dispenser path calls `EntityType.spawn` directly with no peaceful gate in vanilla either — the port matches there.)

### §3.3 Block transforms (B12, B13, B14, B16) — 4 gap — RE-CHECK FIRST (B7-G07)

**B12 (axe strip/scrape/wax-off), B13 (hoe till), B14 (shovel flatten), B16 (copper chest scrape/wax-off):** vanilla 1.21.10 has NO `block_transformer` registry or component — the axe/hoe/shovel tables live in `AxeItem`/`HoeItem`/`ShovelItem` (`STRIPPABLES`, `TILLABLES`, `FLATTENABLES`; 0 `BlockTransformer` references in any dump) and the jar's datapack carries 0 `block_transformer` data files. The 1.21.10 port keeps the 26.2/26.3 form: `game.rs` `block_transform` reads the `minecraft:block_transformer` component from `items.json` (0/1,488 items carry it; horizon-items/src/items.rs:976) and returns `None` for every tool (game.rs:7955-7959), and `block_transform.rs` `Transformers::load` reads `data/minecraft/block_transformer/*.json` (0 files in the prepared data), so the transform table is empty. **All four GAP — B7-G07 (major, reproduces from 1.21.7 and 26.1.2; re-checked first).** Honeycomb wax-on is dead the same way (`block_transform.rs:245` `waxed()` reads the empty axe table; callers game.rs:8346, game/remote.rs:8797) and is recorded as part of G07. No horizon-prepare synthesis step exists for the registry.

- **G13/G14/G15 are FIXED behaviourally, masked by G07** (see §5.1): the shield check (game.rs:7960-7967), the `item_used_on_block` trigger (game.rs:8010), and the copper-chest neighbour code (game.rs:8037 call, `transform_copper_chest_neighbour` at :8055) are all still wrong but unreachable, because `block_transform` returns `None` before any of them runs. **All three return if G07 is fixed.**
- Side finding: vanilla `HoneycombItem` DOES reference `ITEM_USED_ON_BLOCK` (1×, javap-verified fresh), so G14's scope is hoe/shovel/campfire only — the port's honeycomb trigger is correct.

### §3.4 Campfire douse (B15) — GAP — RE-CHECK FIRST (B7-G08)

**B15 (shovel campfire douse):** vanilla 1.21.10 `ShovelItem.useOn` dowses a lit campfire directly (face != DOWN, `CampfireBlock.dowse` + level event 1009; `ShovelItem.txt:64-88`); there is no `#douses_campfires` tag in 1.21.10 (absent from `ItemTags`, the client jar's datapack — 190 item tag files — and the prepared item tags — 172 files, no `douses_campfires.json`). The 1.21.10 port keeps the 26.3 form: game/campfire_douse.rs:12-17 gates the douse on `#minecraft:douses_campfires` (`CampfireBlock.useItemOn`), so `campfire_to_douse` always returns `None` and a shovel never puts out a campfire; the face != DOWN check was also dropped. **GAP — B7-G08 (major, reproduces from 1.21.7 and 26.1.2; re-checked first).** Still wired in game/use_hand.rs:88.

### §3.5 Chorus fruit (B18) — GAP

**B18 (chorus fruit teleport):** vanilla 1.21.10 `LivingEntity.randomTeleport` is blocksMotion-grounded (4-arg signature `(DDDZ)`, no tag parameter, javap-verified fresh, `LivingEntity.txt:9299`). The Rust game/consume.rs:120-139 walks down for `#minecraft:entities_can_teleport_to`, a tag absent in 1.21.10 (absent from `BlockTags`, the client jar's datapack and prepared `block-tags.json`, 203 tags), so the loop always runs to `min_y` and returns false — chorus fruit never teleports. **GAP — B7-G09 (major, reproduces from 1.21.7 and 26.1.2).**

**Masked finding (no separate item):** the tree keeps the level-event 2017 directional-particle trail on the consume path (horizon-world/src/teleport_consume.rs `apply` + game/consume.rs:422-431 `directional_particles`; vanilla `TeleportRandomlyConsumeEffect` has only a `diameter` field and emits no level event — 0 level-event references in the fresh dump, `TeleportRandomlyConsumeEffect.txt`) — but it is masked by G09: the trail fires only after a successful teleport, which never happens on the player path. Fixing G09 without removing the trail would expose it. (The fox path already emits it — Batch 3's area.)

### §3.6 Buckets (B2) — PASS

**B2 (bucket sneak-redirect):** vanilla 1.21.10 `BucketItem.emptyContents` HAS the sneak check (`isShiftKeyDown`, javap-verified fresh, `BucketItem.txt:231`) — vanilla changed between 1.21.7 (no sneak check) and 1.21.10 — and the Rust `bucket_empty` (horizon-world/src/level/buckets.rs:103-146) matches: air always takes the fluid, retry beside with a null hit result when sneaking onto a non-air block. **PASS** (1.21.7 G02 WITHDRAWN — see §5.2).

### §3.7 Potions (B24, B26) — 2 gap

- **B24 (water splash/lingering douse):** vanilla 1.21.10 `AbstractThrownPotion` has ZERO `PotionTags` references — it gates on `PotionContents.is(Potions.WATER)` directly (javap-verified fresh; the 1.21.10 jar has no `PotionTags` class at all and no `data/minecraft/tags/potion/`). The Rust gates everything on potion tags (game/splash_water.rs:42-50 `potion_in_tag`: `minecraft:douses_fire`, `hurts_water_sensitive_entities`, `extinguishes_entities`, `rehydrates_axolotls`), which always return false, so water splash/lingering bottles do nothing on impact. **GAP — B7-G10 (major, reproduces from 1.21.7 and 26.1.2; suspected duplicate of B1-G03 — coordinator to fold; the 1.21.10 Batch 1 report has not landed).**
- **B26 (water bottle on dirt → mud):** vanilla 1.21.10 `PotionItem.useOn` uses `BlockTags.CONVERTABLE_TO_MUD` (with an A, javap-verified fresh, `PotionItem.txt:55`); the Rust uses `convertible_to_mud` (with an I) in all four places (game/potions.rs:710, game/dispenser.rs:2251, game/remote.rs:8677, horizon-world/src/level/data.rs:458), so the lookup always misses and a water bottle on dirt never makes mud. **GAP — B7-G11 (minor, reproduces from 1.21.7 and 26.1.2; suspected duplicate of B5-G19 — coordinator to fold; the 1.21.10 Batch 5 report has not landed).**

### §3.8 Boats (B27) — GAP

**B27 (boat placement eye check):** vanilla 1.21.10 `BoatItem.use` gathers entities in the player's bbox expanded 5.0 along the view + inflate 1.0 and returns PASS when the eye is inside any entity's pick-inflated bbox (javap-verified fresh, `BoatItem.txt:47-78`: `getEyePosition`, `AABB.inflate`, `InteractionResult.PASS`). The Rust `boat_to_place` (game/boats.rs:1518-1542) only checks the boat's own bbox, so a boat is placeable while the player's eye is inside an entity. **GAP — B7-G12 (minor, reproduces from 1.21.7 and 26.1.2).**

### §3.9 Ender pearl / wind charge cooldown (B30) — PASS

**B30 (throw cooldown):** vanilla 1.21.10 `ItemStack.use` calls `applyAfterUseComponentSideEffects` for instant-use items on Success, which reads `USE_COOLDOWN` — pearls (1.0s) and wind charges (0.5s) DO get a cooldown (javap-verified fresh, `ItemStack.txt:537-575`; `items.json` carries `use_cooldown` 1.0s on ender_pearl, 0.5s on wind_charge). The 1.21.10 Rust still applies it: `throw_item` (game/creatures.rs:7309 wind charge, :7357 pearl) calls `apply_use_cooldown` (game.rs:5222), which reads the `use_cooldown` component. **PASS — B7-G16 stays FIXED (as in 26.1.2).**

### §3.10 Copper golem spawn egg (B33) — PASS

**B33 (copper golem spawn egg):** the egg exists in `items.json` (with an `entity_data` component) and maps to `SpawnEggItem`; the port's `Species` has `copper_golem` (mobs/species.rs:660), `use_spawn_egg` derives the kind from the item name, and the dispenser path marks copper golems persistent (game/dispenser.rs:1035) — matching vanilla, whose `CopperGolem` constructor calls `setPersistenceRequired()` (`CopperGolem.txt:93`). No copper-golem-specific defect was found; the generic spawn-egg defects (G03 entity_data override, G06 ambient sound) apply to this egg too and are recorded there. **PASS.**

### §3.11 Ominous bottle / trial key (B34) — PASS

**B34 (ominous bottle / trial key):** ominous bottle is plain `Item` with `consumable` + `ominous_bottle_amplifier` components in `items.json`; the port handles drinking it (`game/potions.rs:769-809` → `drink_ominous_bottle`, game/raids.rs:1450). Trial keys and ominous trial keys are plain `Item`s, handled by the vault/spawner code (game/spawner.rs:495-498). No defect found in item scope; vault behaviour itself is Batch 5's area. **PASS.**

### §3.12 Remaining behaviour checks — all PASS

- **B3 (food consume):** consume path verified against the jar's `Consumable`/`FoodData` shape; saturation clamps to the food level (game/consume.rs:63); food data matches D1. **PASS.**
- **B4 (tool use):** tool components match D4/D5; block breaking reads `damage_per_block` (game/block_breaking.rs:9). **PASS.**
- **B5 (armour equip):** armour data matches D7; equippable handling present (`item_sync::equippable`, used at game/armor_stands.rs:283-284). **PASS.**
- **B7 (fluid use / consume / offspring / entity_data):** spawn-egg fluid path in `use_spawn_egg` (game/creatures.rs:7453-7459); offspring path present (`spawn_egg_offspring`, game/creatures.rs:6316); entity_data application onto the spawned mob present (`apply_spawn_egg_entity_data`, game/creatures.rs:7529). **PASS.**
- **B17 (bundle use):** bundle weight uses `Fraction` with the 1/16 bundle-in-bundle weight (horizon-items/src/bundle.rs:75). **PASS.**
- **B19 (fishing rod):** wait 100–600 ticks minus Lure, nibble 20–40 ticks (game/fishing.rs:118, 247). **PASS.**
- **B20 (crossbow):** charge duration `(1.25 − 0.25×QuickCharge)` s, sounds at charge 0.2/0.5 (game/crossbow.rs:67-81). **PASS.**
- **B21 (shield block):** `blocks_attacks` loader defaults match the jar (90° horizontal angle default, horizon-items/src/items.rs:325). **PASS.**
- **B22 (trident throw):** riptide gate (wet + not riding) and spin strength `1.5 + 0.75×(level−1)` (game/trident.rs:396-397, 519). **PASS.**
- **B23 (mace smash):** fall threshold 1.5, damage tiers 4×/12+2×/22+1×, knockback radius 3.5 (game/mace.rs:20-24, 129-133). **PASS.**
- **B25 (item use order):** `use_hand` dispatch present (game/use_hand.rs:8); bow release power `(f² + 2f)/3` clamped to 1.0, 0.1 minimum (game/item_use.rs:235-243) matches `BowItem.getPowerForTime`. **PASS.**
- **B28 (mount menu):** horse inventory menu present (game/mount_menu.rs). Note: the file's doc comment and `Species` still carry the dead 26.x nautilus (`Nautilus`/`ZombieNautilus`, no spawn path in 1.21.10) — a known deferred port item (PORT-NOTES-1.21.10), not a behaviour defect. **PASS.**
- **B29 (world commands item paths):** item paths present in game/commands/world_cmds.rs. **PASS.**
- **B31 (composter — cited):** composter is dead in 1.21.10 (`compostable` read at game/composter.rs:24-25; 0 items carry `minecraft:compostable`) — Batch 5's defect (B5-G22 in the 26.1.2 run), cited, not re-minted. **PASS for this batch's tally** (the defect is Batch 5's; this batch does not double-count it).
- **B32 (villager food):** vanilla 1.21.10's `DataComponents` has no `villager_food` (0 matches in the dump) and 0/1,488 items carry it; no villager-food component read was found in the item path. **PASS.**

### §3.13 Initial / respawn spawn (B36) — GAP

**B36 (spawn in skylight-less dimensions):** vanilla 1.21.10 `PlayerSpawnFinder.findSpawn` checks `dimensionType().hasSkyLight()` FIRST: in a dimension without skylight (or in adventure mode) it returns `fixupSpawnHeight` at the suggestion at once, with no radius search (javap-verified fresh, server-game.jar, `PlayerSpawnFinder.txt:69-83`). The 1.21.10 port (game/initial_spawn.rs:294-298 `spawn_finder`) has the adventure-mode arm but dropped the `!sky_light` arm, so in the Nether/End it searches the respawn radius instead of returning the fixup height. **GAP — B7-G19 (minor, reproduces from 26.1.2).**

## §4 Defect list

### Gap items (11)

| ID | Sev | Title |
|---|---|---|
| B7-G03 | minor | Spawn egg `entity_data` id type override ignored |
| B7-G06 | minor | No `Mob.playAmbientSound()` on spawn-egg placement |
| B7-G07 | major | Axe/hoe/shovel transforms and honeycomb wax-on entirely non-functional (26.2/26.3 `block_transformer` form) |
| B7-G08 | major | Shovel campfire douse dead: gated on the absent `#douses_campfires` tag |
| B7-G09 | major | Chorus fruit never teleports (tag `#entities_can_teleport_to` absent in 1.21.10) |
| B7-G10 | major | Water splash/lingering bottles do nothing on impact (potion-tag gate dead) — suspected duplicate of B1-G03 |
| B7-G11 | minor | Water bottle on dirt never makes mud (`convertible` vs `convertable`) — suspected duplicate of B5-G19 |
| B7-G12 | minor | Boat placement missing the eye-inside-entity PASS check |
| B7-G17 | minor | Op's spawn-egg `entity_data` applied to spawners (1.21.10 has no `Spawner.setEntityData`) — suspected duplicate of B3-G39 |
| B7-G18 | minor | Dispenser spawn egg returned unshrunk with no `ENTITY_PLACE` on a null spawn |
| B7-G19 | minor | Respawn/initial spawn searches the radius in skylight-less dimensions instead of the fixup height |

### Fixed items (6)

| ID | Sev | Title |
|---|---|---|
| B7-G01 | minor | `ComponentMap::set` returns null (as `Ok`) on a dangling removal marker (FIXED in the 1.21.10 port) |
| B7-G04 | minor | Peaceful gate for hostile spawn eggs (vanilla 1.21.10 HAS the gate; the port HAS it) |
| B7-G13 | minor | Shield blocking-intent check (masked by B7-G07; code still wrong) |
| B7-G14 | minor | `ITEM_USED_ON_BLOCK` trigger for hoe/shovel/campfire (masked by B7-G07/B7-G08; code still wrong) |
| B7-G15 | minor | Copper-chest neighbour `BLOCK_CHANGE` (masked by B7-G07; code still wrong) |
| B7-G16 | minor | Ender pearl / wind charge use cooldown applied on throw (stays fixed) |

### Closed items (2)

| ID | Sev | Title |
|---|---|---|
| B7-G02 | minor | [CLOSED] Bucket sneak-redirect: vanilla 1.21.10 HAS the sneak check |
| B7-G05 | minor | [CLOSED] Spawn-egg `ENTITY_PLACE` position: vanilla posts at the spawn position |

### Oracle-defect items (29)

All 29 reproduce (provisional triage from the 26.1.2 Batch 0 table — the 1.21.10 table has not landed — with root causes javap-verified fresh against 1.21.10's jars and the 1.21.10 oracle sources; see §6).

| ID | Sev | Oracle / root cause |
|---|---|---|
| B7-O01 | major | `VanillaDispenserOracle` — `EntityTypes` package does not exist |
| B7-O02 | major | `VanillaBlockUpdateOracle` — `ClockInstance` does not exist (32 errors in 26.1.2) |
| B7-O03–O07 | minor | `Level.getNextEntityId` does not exist (Arrow, BoatRider, ClientArrow, ExplosionItems, LavaCauldronItem) |
| B7-O08–O09 | minor | `EntityTypes` does not exist (BoatAuthority, ItemModel) |
| B7-O10 | minor | `EntitySpawnRequest` does not exist (VehicleContainer) |
| B7-O12 | minor | `InteractionResult.Success.shouldSwing()` does not exist (Consumable) |
| B7-O13 | minor | 5-arg `randomTeleport` does not exist (TeleportConsume) |
| B7-O14 | minor | 5-arg `blockUsingItem` does not exist (ItemBlocking) |
| B7-O15–O16 | minor | `hurtAndBreak` takes `Consumer<Item>` (ItemDurability, ComponentDurability) |
| B7-O17 | minor | `onEquippedItemBroken` takes `Item` (BlockingItemDamage) |
| B7-O18 | minor | 3-arg `Player.drop` does not exist (ExtraItems; `Prediction` also absent) |
| B7-O19–O20 | minor | `ServerChunkCache` takes 11 constructor args (ItemUseStart, InventoryUseStart) |
| B7-O21–O22 | minor | `Gui` takes 1 constructor arg (HeldDrop, InteractionInput) |
| B7-O23–O24 | minor | `ItemStackTemplate.fromStack` does not exist — the class itself is absent (ItemTemplate, UseRemainder) |
| B7-O25–O27 | minor | `VanillaRegistries.createWorldLookup` does not exist (ItemUseCatalog, InventoryPickup, PlayerInventory) |
| B7-O28 | minor | `LivingEntity.SwingDescription` does not exist (HeldItemStack) |
| B7-O29–O30 | minor | Cascades on broken `VanillaBlockUpdateOracle` (EnchantDurability, HelmetDamage) |

### Oracle fixed (1)

| ID | Sev | Title |
|---|---|---|
| B7-O11 | minor | `KnockbackClient` fixed: punch now sends `ServerboundSwingPacket`; all 49 imports exist in the 1.21.10 jar |

## §5 Re-check verdicts

### §5.1 26.1.2 items (49, SECONDARY)

Every item was re-decided fresh against the 1.21.10 jar and tree. The 1.21.10 tree is NOT a delta of 26.1.2, so no verdict carries by identity — each rests on fresh evidence cited in §3/§6.

| ID | 26.1.2 verdict | 1.21.10 verdict | Notes |
|---|---|---|---|
| G01 | gap | **FIXED** | `component_patch.rs:109-122` returns `Ok(base)` on a dangling marker; doc cites 1.21.10 vanilla |
| G02 | closed | **WITHDRAWN** | Vanilla 1.21.10 `BucketItem` HAS the sneak check; closed item |
| G03 | gap | **REPRODUCES** | `use_spawn_egg` still derives kind from the item name |
| G04 | fixed | **FIXED** | Gate present (creatures.rs:7442-7465); hoglin residual is B3-G04 |
| G05 | closed | **WITHDRAWN** | Vanilla posts `ENTITY_PLACE` at the spawn position; closed item |
| G06 | gap | **REPRODUCES** | `ambient_sound` still dispenser/tick-only |
| G07 | gap | **REPRODUCES** | Re-check-first regression; transform table still empty |
| G08 | gap | **REPRODUCES** | Re-check-first regression; `#douses_campfires` gate still dead |
| G09 | gap | **REPRODUCES** | Tag absent; consume path unchanged; 2017 trail still masked |
| G10 | gap | **REPRODUCES** | `splash_water.rs` gates on potion tags; suspected duplicate of B1-G03 |
| G11 | gap | **REPRODUCES** | All four `convertible_to_mud` sites present; suspected duplicate of B5-G19 |
| G12 | gap | **REPRODUCES** | `boats.rs` has no eye check |
| G13 | fixed | **FIXED (masked)** | Code still wrong; unreachable behind G07 — returns if G07 is fixed |
| G14 | fixed | **FIXED (masked)** | Code still wrong; unreachable behind G07/G08 — returns if those are fixed |
| G15 | fixed | **FIXED (masked)** | Code still wrong; unreachable behind G07 — returns if G07 is fixed |
| G16 | fixed | **FIXED** | Cooldown still applied (creatures.rs:7309, 7357) |
| G17 | gap | **REPRODUCES** | Both spawner arms present; suspected duplicate of B3-G39 |
| G18 | gap | **REPRODUCES** | Null spawn still returns the stack unshrunk, no event |
| G19 | gap | **REPRODUCES** | `!sky_light` arm still missing from `spawn_finder` |
| O01–O10 | oracle-defect | **REPRODUCE** | Root causes javap-verified fresh (see §6) |
| O11 | oracle-defect | **FIXED** | KnockbackClient adapted: `ServerboundSwingPacket`, no `ServerboundPunchPacket` ref |
| O12–O30 | oracle-defect | **REPRODUCE** | Root causes javap-verified fresh (see §6) |

### §5.2 1.21.7 items (12, PRIMARY)

| ID | 1.21.7 verdict | 1.21.10 verdict | Notes |
|---|---|---|---|
| G01 | gap | **FIXED** | Port fixed it; same item as 26.1.2 G01 |
| G02 | gap | **WITHDRAWN** | Vanilla 1.21.10 `BucketItem` HAS the sneak check (1.21.7 did not); closed item |
| G03 | gap | **REPRODUCES** | Same defect as 26.1.2 G03 (one item) |
| G04 | gap | **FIXED** | Vanilla 1.21.10 HAS the gate and the port HAS it; same item as 26.1.2 G04 |
| G05 | gap | **WITHDRAWN** | Vanilla 1.21.10 posts `ENTITY_PLACE` at the spawn position (1.21.7 posted at the clicked block); closed item |
| G06 | gap | **REPRODUCES** | Same defect as 26.1.2 G06 (one item) |
| G07 | gap | **REPRODUCES** | Same defect as 26.1.2 G07 (one item); re-check-first regression |
| G08 | gap | **REPRODUCES** | Same defect as 26.1.2 G08 (one item); re-check-first regression |
| G09 | gap | **REPRODUCES** | Same defect as 26.1.2 G09 (one item) |
| G10 | gap | **REPRODUCES** | Same defect as 26.1.2 G10 (one item) |
| G11 | gap | **REPRODUCES** | Same defect as 26.1.2 G11 (one item) |
| G12 | gap | **REPRODUCES** | Same defect as 26.1.2 G12 (one item) |

## §6 Oracle triage

**Provisional** — the 1.21.10 Batch 0 table has not landed. Triage is from the 26.1.2 measured table (error counts/classes below are the 26.1.2 rows), with every root cause re-verified two ways against 1.21.10: the oracle source in the 1.21.10 tree still references the API, and the API is absent/different in 1.21.10's own jar. 20 of the 30 oracle files are byte-identical to 26.1.2's; the 10 that differ carry only the 1.21.x naming adaptation (`Identifier` → `ResourceLocation` etc.) — no root cause was adapted away except O11.

- **O01/O08/O09** — `EntityTypes` does not exist in the 1.21.10 client jar (registration still in `EntityType`; common.md confirms). Sources still reference it (`VanillaDispenserOracle.java:212,235-236`).
- **O02** — `ClockInstance` does not exist in the 1.21.10 client jar (no `world/clock/` package); source still references it (`VanillaBlockUpdateOracle.java:450`). Cascade root for O29/O30.
- **O03–O07** — `Level.getNextEntityId` does not exist in 1.21.10 (0 matches in the `Level` dump); all five sources still `@Override` it.
- **O10** — `EntitySpawnRequest` does not exist; source imports and uses it (`VanillaVehicleContainerOracle.java:20,118`).
- **O11 FIXED** — the 1.21.10 `KnockbackClient.java` has no `ServerboundPunchPacket` reference (the 26.1.2 file has 3); punch sends `ServerboundSwingPacket` (line 144); all 49 `net.minecraft` imports exist in the 1.21.10 client jar.
- **O12** — `InteractionResult.Success` is a record with a `swingSource` field; no `shouldSwing()` method (`InteractionResult_Success.txt:3,81`); source calls `s.shouldSwing()` (`VanillaConsumableOracle.java:76`).
- **O13** — `LivingEntity.randomTeleport` is 4-arg `(DDDZ)`, no tag parameter; source overrides the 5-arg form with a `TagKey<Block>` (`VanillaTeleportConsumeOracle.java:27,38`). Same 26.2-form tag parameter behind B7-G09.
- **O14** — `LivingEntity.blockUsingItem` is 2-arg `(ServerLevel, LivingEntity)`; source overrides the 5-arg form (`VanillaItemBlockingOracle.java:29,38`).
- **O15/O16** — `ItemStack.hurtAndBreak` takes `Consumer<Item>`, not `Consumer<ItemStack>` (`ItemStack.txt:739`); both sources pass the lambda result to `snapshot(ItemStack)`.
- **O17** — `LivingEntity.onEquippedItemBroken` takes `(Item, EquipmentSlot)` (`LivingEntity.txt:9747`); source overrides `(ItemStack, EquipmentSlot)`.
- **O18** — `Player.drop` is 2-arg `(ItemStack, boolean)` (`Player.txt:1157`); source overrides the 3-arg form with `Prediction`, a class that does not exist in either 1.21.10 jar.
- **O19/O20** — `ServerChunkCache` takes 11 constructor args in 1.21.10 (`ServerChunkCache.txt:39`); both sources call a 10-arg super.
- **O21/O22** — `Gui` takes 1 arg (`Gui(Minecraft)`, `Gui.txt:181`); both sources call `super(null, null, null)` from a `Gui` subclass.
- **O23/O24** — `ItemStackTemplate` does not exist at all in 1.21.10 (stronger than 26.1.2, where the class existed without `fromStack`); both sources call `ItemStackTemplate.fromStack`.
- **O25–O27** — `VanillaRegistries` has `createLookup()`, not `createWorldLookup` (`VanillaRegistries.txt:39`); all three sources call `createWorldLookup`.
- **O28** — `SwingDescription` does not exist anywhere in the 1.21.10 client jar and `LivingEntity` has no `getCurrentSwing`; source overrides `getCurrentSwing()` returning `SwingDescription`.
- **O29/O30** — cascades on O02; both sources reference `VanillaBlockUpdateOracle` directly (`VanillaEnchantDurabilityOracle.java:27-33`, `VanillaHelmetDamageOracle.java:33-98`); no direct javac error of their own.

Passing in-scope oracles (per the provisional table, same set as 26.1.2): 36 pass, including `VanillaItemCooldownsOracle` and `VanillaComponentPatchOracle` — plus `KnockbackClient`, newly passing in 1.21.10.

## §7 Driver inventory

74 drivers — **33 runnable / 24 blocked / 17 no-oracle** — PROVISIONAL (derived from the 26.1.2 Batch 0 table + the verified B7-O11 fix; the 1.21.10 table has not landed). All 74 driver files exist in the 1.21.10 tree; the only oracle-set mismatch vs the 26.2 map is the known `interaction_input`/`ParityData` helper reference. The one change vs 26.1.2 (32/25/17) is `item_use_order` flipping to runnable because `KnockbackClient` is fixed in 1.21.10. **No driver was run**, per the brief; the 33 runnable drivers are recorded as not run, not as passes. Full inventory: `evidence/driver-inventory-1.21.10.json`, `evidence/driver_inventory_1_21_10.out`.

## §8 Systemic findings

1. **The 1.21.10 port fixed two things the 26.x trees never did — and kept the rest.** `ComponentMap::set` (G01) and `KnockbackClient` (O11) were both fixed during the 1.21.10 behaviour port, with comments citing 1.21.10's vanilla form. But the four re-check-first regressions (G07, G08, G09, G10) reproduce verbatim, as do G03, G06, G11, G12, G17, G18, G19. The port's fixes were selective, not systemic.
2. **Vanilla changed between 1.21.7 and 1.21.10 in exactly the two places that withdraw 1.21.7 items.** The bucket sneak check and the spawn-egg peaceful gate and spawn-position `ENTITY_PLACE` all exist in 1.21.10's jar where 1.21.7 had none of them. The 1.21.7 sweep's G02/G04/G05 premises fail for 1.21.10 on the jar, not on the port — the port matches vanilla 1.21.10 in all three cases.
3. **Absent-tag/component reads are unchanged in scope.** The 1.21.10 tree reads the same absent tags/components as the 26.x trees did: `#douses_campfires`, `#entities_can_teleport_to` (in four places — chorus fruit at consume.rs:134, enderman at mobs/enderman.rs:117, spreadplayers-style code at game/commands/world_cmds.rs:2100-2105, pet teleport at mobs/tame.rs:1341), `#enderman_does_not_teleport_to`, `#consumable_does_not_teleport_to` (consume.rs:399, fox.rs:439), the `minecraft:block_transformer` and `minecraft:mob_visibility` components. The tag table resolves absent tags to empty sets, so every read silently degrades to false, never an error.
4. **Masked defects are still a trap for fixers.** G13/G14/G15 (and the player 2017 trail under G09) are behaviourally fixed only because G07/G08/G09 kill the whole path. Any fix of G07, G08 or G09 must also fix the masked code or the defects reappear.
5. **1.21.10-record corrections (from this run):** (a) the 1.21.7 item→class parser mis-attributes the `WeatheringCopperItems` collections in 1.21.10 (24 unmapped, 26 mismatches before the fix — see §1); the fix is in this run's script. (b) The 26.1.2 data-check script's mob-bucket food assumption (`.component(FOOD, Foods.X)`) does not hold for 1.21.10 — vanilla 1.21.10 puts no food component on mob buckets, so D1's 4 initial "mismatches" are script artifacts. (c) Vanilla 1.21.10 has no named `SpawnEggItemBehavior` class — the dispenser spawn-egg behaviour is the anonymous `DispenseItemBehavior$1` (the 26.x named class does not exist in 1.21.10).
6. **Out-of-scope flags (cited, not minted):** hoglin in the shared `allowed_in_peaceful` denylist (+ a dead `Parched` entry) → Batch 3 (B3-G04, including the spawn-egg facet); enderman teleport on the absent `#entities_can_teleport_to` (mobs/enderman.rs:117) → Batch 3 (B3-G14); mob-head visibility via the `minecraft:mob_visibility` component (mobs.rs:3168) → Batch 3 (B3-G33); composter → Batch 5 (B5-G22); `#entities_can_teleport_to` at game/commands/world_cmds.rs:2100-2105 → Batch 8 (commands); pet teleport at mobs/tame.rs:1341 → Batch 3; splash-water douse → Batch 1 (B1-G03, suspected duplicate of B7-G10); mud tag → Batch 5 (B5-G19, suspected duplicate of B7-G11); spawner entity_data → Batch 3 (B3-G39, suspected duplicate of B7-G17). The 1.21.10 Batch 1/3/5 reports have not landed, so the three suspected duplicates are flagged for the coordinator to fold, not folded here. The 26.2 run's out-of-scope observations were NOT re-verified for 1.21.10 and are not carried forward as claims.

## §9 Method notes + evidence index

**Method.** Static analysis only: `javap -c -p` on the named 1.21.10 jars (`client-named.jar` for client classes, `server-game-named.jar` for server classes) for every vanilla method in scope; Rust source reads; prepared-data checks (`.local` `items.json`, `block-tags.json`, jar data files); and a diff of every in-scope oracle source between the 26.1.2 and 1.21.10 trees. No drivers were run and no oracle repairs were done (checks + triage only). Jar sha1s verified against the brief pins before use.

**Evidence** (`~/workspace/mcir-work/sweep-1.21.10-batches/b07/evidence/`):

- `javap_dumps.sh`, `javap/` — 66 fresh 1.21.10 javap dumps (incl. `DispenseItemBehavior$1`–`$15` anonymous classes and `WeatheringCopperItems`), plus `client-named-list.txt` / `server-game-named-list.txt` / `client-list.txt` jar listings.
- `parse_items_class_1_21_10.py/.out`, `item-class.json`, `nonblock-by-class.json` — D8 map.
- `extract_materials_1_21_10.py`, `materials-1.21.10.json` — ToolMaterial/ArmorMaterials from the 1.21.10 jar.
- `data_checks_1_21_10.py/.out` — D1–D7.
- `driver_inventory_1_21_10.py/.out`, `driver-inventory-1.21.10.json` — driver inventory (provisional).

**Key jar evidence** (all from fresh 1.21.10 dumps in `b07/evidence/javap/`): `SpawnEggItem.txt:146-180` spawnMob peaceful gate; `:182` ENTITY_PLACE at the spawn pos; `:283-303` getType reads ENTITY_DATA; `EntityType.txt:557` playAmbientSound in spawn; 36 `notInPeaceful` types extracted (no hoglin). `LivingEntity.txt:9299` randomTeleport 4-arg `(DDDZ)`; `:3302` blockUsingItem 2-arg; `:9747` onEquippedItemBroken `(Item, EquipmentSlot)`. `ItemStack.txt:739` hurtAndBreak `Consumer<Item>`; `:537-575` applyAfterUseComponentSideEffects reads USE_COOLDOWN; `Player.txt:1157` drop 2-arg; `ServerChunkCache.txt:39` 11-arg ctor; `Gui.txt:181` 1-arg ctor; `VanillaRegistries.txt:39` only `createLookup`; `InteractionResult_Success.txt:3,81` record with `swingSource`, no `shouldSwing`; `Level.txt` 0 `getNextEntityId`; `DispenseItemBehavior-1.txt:69-116` execute pops spawn result, unconditionally shrink + ENTITY_PLACE; `PlayerSpawnFinder.txt:69-83` findSpawn skylight gate first; `TeleportRandomlyConsumeEffect.txt` only `diameter` field, apply calls 4-arg randomTeleport, 0 level-event refs; `BaseSpawner.txt:77` only `setEntityId` (0 `setEntityData` in any dump); `AxeItem.txt:3` STRIPPABLES, `HoeItem.txt:3` TILLABLES, `ShovelItem.txt:3` FLATTENABLES, 0 `BlockTransformer` references in any of the three; `BucketItem.txt:231` isShiftKeyDown; `ShovelItem.txt:64-88` direct campfire dowse; `AbstractThrownPotion.txt` 0 `PotionTags` refs, `Potions.WATER` at :74 and :130; `PotionItem.txt:55` CONVERTABLE_TO_MUD; `BoatItem.txt:47-78` eye-inside-entity check; `CopperGolem.txt:93` setPersistenceRequired in the constructor. Absent from both 1.21.10 jars: `PotionTags`, `ItemStackTemplate`, `EntityTypes`, `ClockInstance`, `EntitySpawnRequest`, `ServerboundPunchPacket`, `LivingEntity$SwingDescription`, `UseEffects`, `PiercingWeapon`, `KineticWeapon`, `SpearItem`, `SpawnEggItemBehavior`, `Prediction`, `OminousBottleItem`, `TrialKeyItem`.

## §10 What this batch defers + verdict

**Deferred (not this batch's work):** driver runs (33 runnable drivers unrun), oracle repairs (29 failing oracles), all fixes (including the four re-check-first regressions G07/G08/G09/G10), re-verification of the oracle triage against the 1.21.10 Batch 0 table when it lands, and the out-of-scope flags in §8.6.

**Verdict: 44 checks — 30 pass, 14 gap.** The 1.21.10 Items tree is a fresh full port that fixed two defects the 26.x trees never did (B7-G01 component_patch, B7-O11 KnockbackClient) and matches vanilla 1.21.10 where vanilla changed since 1.21.7 (bucket sneak, peaceful gate, ENTITY_PLACE position) — but the four re-check-first regressions (B7-G07 tool transforms, B7-G08 campfire douse, B7-G09 chorus fruit, B7-G10 splash water) all reproduce with fresh 1.21.10 evidence, as do seven more defects. No new behaviour defect was found in scope. Three reproductions are suspected duplicates of other 1.21.10 batches (B1-G03, B5-G19, B3-G39) for the coordinator to fold once those reports land.
