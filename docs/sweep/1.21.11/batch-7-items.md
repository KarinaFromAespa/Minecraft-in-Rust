# Batch 7 — Items (Minecraft 1.21.11)

**Scope:** item use, food, tools, armour, buckets, spawn eggs, components — 1,505 items.

**Result: 45 checks — 31 pass, 14 gap** (as-tallied 45 = 45 behaviour + 0 non-behaviour). 11 gap items (4 major, 7 minor), 6 fixed, 2 closed, 39 oracle-defect items (2 major, 37 minor), 1 oracle fixed. The headline: the 1.21.11 template fixed one thing its first child never did (B7-O09 `VanillaItemModelOracle` — substantively adapted and now passing) — but every behaviour defect from 1.21.10 reproduces verbatim, including the four re-check-first regressions (G07 tool transforms, G08 campfire douse, G09 chorus fruit, G10 splash water). No new behaviour defect was found in scope. Ten oracle defects (O31–O40) are NEW to this run: they failed in the 1.21.10 measured table too but no 1.21.10 batch recorded them, because the batches triaged provisionally from the 26.1.2 table where they passed. The 1.21.10 run's B7-O11 "FIXED" verdict for `KnockbackClient` was a provisional-triage error — the measured 1.21.10 table already shows it failing, and it fails in 1.21.11 with 7 errors (the punch cause is gone; new causes replace it).

## §0 Execution status

- **Checks: 45 total, 31 pass, 14 gap — all behaviour, 0 non-behaviour.** Oracle triage is not tallied (it is not a driver check). One check is new in this run: B37 (spear kinetic/piercing, 1.21.11-only content) — PASS.
- **Items: 59** — 11 gap (4 major, 7 minor), 6 fixed, 2 closed, 39 oracle-defect (2 major, 37 minor), 1 oracle fixed (B7-O09).
- **No drivers were run and no oracle repairs were done** (checks + triage only, per the brief). No UNVERIFIED carry-overs: every mandatory re-check (40 from 1.21.10, 12 from 1.21.7) was decided fresh against 1.21.11's jar and tree — and so were the 9 items the brief's mandatory primary list omitted (G10, G11, G17, O01, O02, O07, O09, O10, O29): G10/G11 are covered by the secondary 1.21.7 items; G17/O01/O02/O07/O10/O29 were triaged as in-scope per the brief's oracle-scope section and the template's B6 check. All 49 1.21.10 items are decided in §5.1.
- **Batch 0 oracle table has NOT landed** for 1.21.11 (`docs/sweep/1.21.11/` holds only the inventory + `extract-inventory.py`; b00 still working). Oracle triage is therefore **MEASURED by this batch itself**, not provisional: a full javac fixpoint measurement over all 391 oracle files on three classpaths (`evidence/measure_oracles_1_21_11.py`, output `evidence/oracle-measure/results.json`): **both 127/391 pass, client 127/391 pass, server 102/391 pass.** Every in-scope root cause was then javap-verified against 1.21.11's own jars and the 1.21.11 oracle sources. The driver inventory (§7) inherits the measured status.
- **The 1.21.10 batch's provisional single-cause citations do NOT transfer.** The 1.21.10 report cited one root cause per oracle from the 26.1.2 table; the measured 1.21.11 picture is multi-cause for many oracles (notably the widespread 10-arg `ServerLevel` super vs vanilla's 11-arg ctor, which rides along with the headline cause in O03, O06, O07, O12, O17, O38, O39, O40). Several 1.21.10-table first-errors also do not transfer at all: `world/level/gamerules/`, `world/entity/projectile/arrow/`, `world/entity/vehicle/boat/`, `world/entity/animal/fox/` packages all EXIST in 1.21.11's jars.
- **Oracle sources vs 1.21.10:** 20 of the 30 in-scope oracle files are byte-identical to the 1.21.10 tree's; 10 differ, 9 only by the naming adaptation in reverse (1.21.10 uses `ResourceLocation`/`location()`, 1.21.11 uses `Identifier`/`identifier()`); the 10th is O09's substantive adaptation (§6).
- **Tree vs 1.21.10:** the 1.21.11 tree IS the E6 template — 1.21.10 derives from it, not the reverse. No verdict carries by identity in either direction; every verdict below rests on fresh 1.21.11 jar + tree evidence.
- **No horizon-prepare output exists for 1.21.11 anywhere** (no `.local/horizon-client/1.21.11/items.json` or `block-tags.json`; the sweep worktree `.local` holds only `minecraft/1.21.11/` staging + `measure-oracles/`). Data checks therefore run against Mojang's datagen report `~/workspace/mcir-work/e6-1.21.11-behaviour/versions/1.21.11/.local/datagen-1.21.11/reports/items.json` (1,505 items) as the component spec. The prepare step itself — whether horizon-prepare synthesises the same data for 1.21.11 — is UNVERIFIED by this batch and is stated as a gap in §2/§10, not assumed.
- **Worktree state:** jar sha1s match the brief pins (client-named `29bddf14…`, server-game-named `7f0d85c1…`, client `ba2df812…`, bundler server `64bb6d76…`). Java 21, DataVersion 4671, Protocol 774.
- **B9 (peaceful gate) is counted as PASS** in the tally: vanilla 1.21.11 HAS the gate and the port HAS it (B7-G04 FIXED). The hoglin residual (hoglin wrongly in the port's not-allowed list) is Batch 3's B3-G04 facet and lives in Batch 3's tally, not this batch's (flagged in §8.6; the 1.21.11 Batch 3 report has not landed).

## §1 Item → class map

Same extraction as 1.21.10, re-run against the 1.21.11 jar (`evidence/parse_items_class_1_21_11.py`, output `evidence/parse_items_class_1_21_11.out`): **1,505 items mapped to 70 classes, 0 unmapped, 0 multi-class segments**. Block items: 1,022; non-block items: 483. No `block-behaviours.json` comparison was possible for 1.21.11 (no prepared data — see §0); the block set for the split came from the datagen `blocks.json` (1,166 blocks).

**Parser fix vs the 1.21.10 script (recorded, not a defect):** in 1.21.11 spawn eggs go through `Items.registerSpawnEgg` (bootstrap factory #1, constructs `SpawnEggItem`); the segment shows no indy, so the 1.21.10 parser's factory-based mapping misses them. The 1.21.11 parser maps `registerSpawnEgg` segments to `SpawnEggItem`. After the fix: 0 unmapped, 0 multi-class segments.

| Class | Count | Class | Count |
|---|---|---|---|
| `net.minecraft.world.item.BlockItem` | 909 | `net.minecraft.world.item.EggItem` | 3 |
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

The 46 singletons include `BowItem`, `CrossbowItem`, `TridentItem`, `MaceItem`, `ShieldItem`, `FishingRodItem`, `EnderpearlItem`, `WindChargeItem`, `PotionItem`, `SplashPotionItem`, `LingeringPotionItem`, `HoneycombItem`, `FlintAndSteelItem`, `ShearsItem`, `CompassItem`, `SpyglassItem`, `SolidBucketItem`, `AirItem` (full list in `evidence/parse_items_class_1_21_11.out`). Copper golem spawn egg maps to `SpawnEggItem`; ominous bottle and trial keys map to plain `Item`.

**The +17 items vs 1.21.10, identified exactly** (diff of the 1.21.11 datagen report vs the 1.21.10 prepared `items.json`; 1.21.10-only: none): 7 spears, 5 nautilus armours, 4 spawn eggs (`camel_husk`, `nautilus`, `parched`, `zombie_nautilus`), `netherite_horse_armor`. There is no `SpearItem` class in 1.21.11 — spears and nautilus armour map to plain `Item` with components. The nautilus species are REAL in 1.21.11 (`DefaultAttributes` has NAUTILUS, ZOMBIE_NAUTILUS, PARCHED, CAMEL_HUSK); the template ports them (`mobs/nautilus.rs`, spawning finalize at `mobs/spawning.rs:282`) and carries spear code (`game/spear.rs`, kinetic/piercing handling in `game/hand.rs:827`).

## §2 Data checks

All 8 pass (`evidence/data_checks_1_21_11.py`, output `evidence/data_checks_1_21_11.out`; materials extracted fresh from the 1.21.11 jar into `evidence/materials-1.21.11.json` — 7 tool materials incl. COPPER [190, 5.0, 1.0, 13], 9 armour materials incl. COPPER [11, 1/3/4/2/4, 8] — identical to 1.21.10). **Caveat:** the spec side is Mojang's datagen report, not horizon-prepare output (none exists for 1.21.11 — see §0). These checks prove the datagen spec is internally consistent with the jar; they do NOT prove the port's prepare step reproduces it.

- **D1 (food):** 44 foods, 0 mismatches (40 + 4 fish buckets — **vanilla 1.21.11 ADDED food components to mob buckets** vs 1.21.10, where the segments carried none; `MobBucketItem` has 0 `Foods.` refs — the components come from `Items.<clinit>`). **PASS.**
- **D2 (durability):** 84 items with durability (77 + 7 spears), 0 mismatches. **PASS.**
- **D3 (max stack size):** 1,505 items, 0 mismatches. **PASS.**
- **D4 (tool pairs):** 42 tool/spear pairs (35 + 7), 0 mismatches; spear kinetic 7 spears × 8 values, 0 mismatches. **PASS.**
- **D5 (tool components):** 35 tool components, 0 mismatches. **PASS.**
- **D6 (tags):** 13 tool-rule tags present in the jar datapack, 0 mismatches (jar half only — no prepared `block-tags.json` to compare against). **PASS.** **Script bug found and fixed:** the 1.21.10 script's `expand()` double-prefixed `"minecraft:"` on `#minecraft:...` refs, making both sides equally wrong in the comparison. Expanded sizes are recorded in the `.out`; `incorrect_for_diamond_tool` and `incorrect_for_netherite_tool` expand to 0 — vanilla-correct, identical in the 1.21.10 jar.
- **D7 (armour):** 41 armour pieces (35 + 5 nautilus + 1 netherite horse armour), 0 mismatches. **PASS.**
- **D8 (item→class map):** 1,505 items → 70 classes, 0 unmapped, 0 multi-class segments (see §1). **PASS.**

Datagen spot checks (1.21.11 report): **0 items carry `minecraft:block_transformer`** (the component the transform path reads — see G07), **0 carry `minecraft:compostable`**, **0 carry `minecraft:mob_visibility`**, **0 carry `minecraft:villager_food`** — and none of those four components exists in 1.21.11's `DataComponents` (105 fields) at all. Ender pearl `use_cooldown` 1.0s, wind charge 0.5s; ominous bottle has `consumable` + `ominous_bottle_amplifier`; copper golem spawn egg has `entity_data` id `minecraft:copper_golem`. Jar datapack: 198 item tag files, 204 block tag files; no `douses_campfires.json`, no `tags/potion/`; `convertable_to_mud.json` present (with an A), `convertible_to_mud` absent. `BlockTags`: `CONVERTABLE_TO_MUD` present; `ENTITIES_CAN_TELEPORT_TO`, `ENDERMAN_DOES_NOT_TELEPORT_TO`, `CONSUMABLE_DOES_NOT_TELEPORT_TO`, `BLOCKS_MOTION` absent.

## §3 Behaviour checks

### §3.1 Component map (B1) — PASS (FIXED in the template)

**B1 (component map set/remove):** vanilla 1.21.11 `PatchedDataComponentMap.set` returns `previous.orElse(prototype)` — null on a dangling removal marker, never an error (javap-verified fresh, `PatchedDataComponentMap.txt:131-163`). The template's `ComponentMap::set` (horizon-world/src/component_patch.rs:109-122) returns `Ok(base)` on a dangling removal marker — fixed during the behaviour port, with a doc comment citing 1.21.11's form explicitly. **PASS — B7-G01 FIXED** (was a gap in 1.21.7 and all 26.x runs).

### §3.2 Spawn eggs (B6, B8, B9, B10, B11, B35) — 2 pass, 4 gap

- **B6 (spawn egg on spawner):** vanilla `SpawnEggItem.useOn` calls only `Spawner.setEntityId` — the 1.21.11 jar has NO `Spawner.setEntityData` (0 matches in the `BaseSpawner` and `TrialSpawnerBlockEntity` dumps; `BaseSpawner` has only `setEntityId`, javap-verified fresh, `BaseSpawner.txt:77`). The template applies an op's egg `entity_data` to spawners (game/spawner.rs:464 `set_entity_data` for regular spawners; :450 `override_entity_data` for trial spawners). **GAP — B7-G17 (minor, reproduces from 1.21.10/26.1.2; suspected duplicate of B3-G39 — coordinator to fold; the 1.21.11 Batch 3 report has not landed).**
- **B8 (spawn egg entity_data id override):** vanilla `SpawnEggItem.getType` reads the typed `entity_data` component's `type()` and spawns THAT type (javap-verified fresh: `getType` reads `DataComponents.ENTITY_DATA`, `TypedEntityData.type()`; `SpawnEggItem.txt:282-303`). The template derives the kind from the item name only (game/creatures.rs:7460 `use_spawn_egg`, `trim_end_matches("_spawn_egg")`) and `spawn_egg_entity_data` returns `None` when the `id` differs (game/block_entity_item.rs:104-106). **GAP — B7-G03 (minor, reproduces from 1.21.7, 1.21.10 and 26.1.2).**
- **B9 (spawn egg peaceful gate):** vanilla 1.21.11 `SpawnEggItem.spawnMob` FAILs hostile eggs in peaceful (`!isAllowedInPeaceful && difficulty == PEACEFUL`, javap-verified fresh, `SpawnEggItem.txt:156-160`). The template has the gate: game/creatures.rs:7441-7448 plus the `use_failed` return at :7462-7465. **PASS — B7-G04 FIXED.** Residual: the shared `allowed_in_peaceful` denylist wrongly includes hoglin (37 vanilla `notInPeaceful` types extracted fresh from the 1.21.11 `EntityType` dump — 36 + `parched`, new in 1.21.11; no hoglin; port denylist at mobs/spawn_rules.rs:134-175 has 38 = vanilla's 37 + Hoglin) — recorded as **B3-G04**, which also covers the spawn-egg facet; it lives in Batch 3's tally, not this batch's. Note the 1.21.10 report's dead-`Parched` observation does NOT apply here: parched is a real 1.21.11 species and correctly included.
- **B10 (spawn egg ENTITY_PLACE position):** vanilla 1.21.11 posts `ENTITY_PLACE` at the spawn position (`spawnMob` posts at its `BlockPos` argument, the computed spawn pos; `SpawnEggItem.txt:181`), and the template posts at `p` (the spawn pos, game/creatures.rs:7516). **PASS** (1.21.7 G05 WITHDRAWN — vanilla changed between 1.21.7, which posted at the clicked block, and 1.21.10/1.21.11; see §5.2).
- **B11 (spawn egg ambient sound):** vanilla `EntityType.spawn` calls `Mob.playAmbientSound()` after `addFreshEntityWithPassengers` (javap-verified fresh, `EntityType` dump lines 557/565); the template's `ambient_sound` is only called from the dispenser path (game/dispenser.rs:980) and the tick-time path (mobs.rs:6499), never on spawn-egg or mob-bucket placement (game/creatures.rs `use_spawn_egg`) — verified by a tree-wide call-site grep. **GAP — B7-G06 (minor, reproduces from 1.21.7, 1.21.10 and 26.1.2).**
- **B35 (dispenser spawn egg):** vanilla 1.21.11 has no named `SpawnEggItemBehavior` class — the behaviour is the anonymous `DispenseItemBehavior$1`, whose `execute` pops the `EntityType.spawn` result and UNCONDITIONALLY shrinks the stack by 1 and fires `ENTITY_PLACE` at the dispenser pos (javap-verified fresh, server-game.jar, `DispenseItemBehavior-1.txt:69` pop, `:95-97` shrink, `:100-112` event) — the egg is consumed and the event fires even when the spawn returns null. (Only a null `getType` returns the stack unchanged.) The template (game/dispenser.rs:1025-1027) returns the stack unshrunk with no event on a null spawn. **GAP — B7-G18 (minor, reproduces from 1.21.10/26.1.2).** (The dispenser path calls `EntityType.spawn` directly with no peaceful gate in vanilla either — the port matches there.)

### §3.3 Block transforms (B12, B13, B14, B16) — 4 gap — RE-CHECK FIRST (B7-G07)

**B12 (axe strip/scrape/wax-off), B13 (hoe till), B14 (shovel flatten), B16 (copper chest scrape/wax-off):** vanilla 1.21.11 has NO `block_transformer` registry or component — the axe/hoe/shovel tables live in `AxeItem`/`HoeItem`/`ShovelItem` (`STRIPPABLES`, `TILLABLES`, `FLATTENABLES`; 0 `BlockTransformer` references in any dump) and the jar's datapack carries 0 `block_transformer` data files. The template keeps the 26.2/26.3 form: `game.rs` `block_transform` reads the `minecraft:block_transformer` component (game.rs:7919, lookup at :7948; horizon-items/src/items.rs:973) and returns `None` for every tool, and `block_transform.rs` `Transformers::load` reads `data/minecraft/block_transformer/*.json` (:185; 0 files in vanilla data), so the transform table is empty. **All four GAP — B7-G07 (major, reproduces from 1.21.7, 1.21.10 and 26.1.2; re-checked first).** Honeycomb wax-on is dead the same way (`block_transform.rs:245` `waxed()` reads the empty axe table) and is recorded as part of G07. No horizon-prepare synthesis step exists for the registry.

- **G13/G14/G15 are FIXED behaviourally, masked by G07** (see §5.1): the shield check (game.rs:7952-7958 — applies to any tool, no axe-only/main-hand check), the `item_used_on_block` trigger (game.rs:7999), and the copper-chest neighbour code (game.rs:8026 call, `transform_copper_chest_neighbour` at :8044) are all still wrong but unreachable, because `block_transform` returns `None` before any of them runs. **All three return if G07 is fixed.**
- Side finding: vanilla `HoneycombItem` DOES reference `ITEM_USED_ON_BLOCK` (1×, javap-verified fresh), so G14's scope is hoe/shovel/campfire only — the port's honeycomb trigger is correct. Vanilla fires the trigger in `AxeItem` (`:52`) and `HoneycombItem` (`:118`) only — 0 refs in `HoeItem`/`ShovelItem`.

### §3.4 Campfire douse (B15) — GAP — RE-CHECK FIRST (B7-G08)

**B15 (shovel campfire douse):** vanilla 1.21.11 `ShovelItem.useOn` dowses a lit campfire directly (face != DOWN, `CampfireBlock.dowse` + level event 1009; `ShovelItem.txt:64-88`); there is no `#douses_campfires` tag in 1.21.11 (absent from `ItemTags`, the client jar's datapack — 198 item tag files — and the datagen report). The template keeps the 26.3 form: game/campfire_douse.rs:12-17 gates the douse on `#minecraft:douses_campfires` (`CampfireBlock.useItemOn`), so `campfire_to_douse` always returns `None` and a shovel never puts out a campfire; the face != DOWN check was also dropped. **GAP — B7-G08 (major, reproduces from 1.21.7, 1.21.10 and 26.1.2; re-checked first).** Still wired in game/use_hand.rs:88.

### §3.5 Chorus fruit (B18) — GAP

**B18 (chorus fruit teleport):** vanilla 1.21.11 `LivingEntity.randomTeleport` is blocksMotion-grounded (4-arg signature `(DDDZ)`, no tag parameter, javap-verified fresh, `LivingEntity.txt:9792`). The template game/consume.rs:134 walks down for `#minecraft:entities_can_teleport_to`, a tag absent in 1.21.11 (absent from `BlockTags`, the client jar's datapack and the datagen report), so the loop always runs to `min_y` and returns false — chorus fruit never teleports. **GAP — B7-G09 (major, reproduces from 1.21.7, 1.21.10 and 26.1.2).**

**Masked finding (no separate item):** the tree keeps the level-event 2017 directional-particle trail on the consume path (horizon-world/src/teleport_consume.rs `apply` :51-52, :109-116 + game/consume.rs:422-429 `directional_particles`; vanilla `TeleportRandomlyConsumeEffect` has only a `diameter` field and emits no level event — 0 level-event references in the fresh dump, calls 4-arg `randomTeleport` at `:106`) — but it is masked by G09: the trail fires only after a successful teleport, which never happens on the player path. Fixing G09 without removing the trail would expose it. (The fox path already emits it — Batch 3's area.)

### §3.6 Buckets (B2) — PASS

**B2 (bucket sneak-redirect):** vanilla 1.21.11 `BucketItem.emptyContents` HAS the sneak check (`isShiftKeyDown`, javap-verified fresh, `BucketItem.txt:231`) — vanilla changed between 1.21.7 (no sneak check) and 1.21.10/1.21.11 — and the template's `bucket_empty` (horizon-world/src/level/buckets.rs:103-146) matches: air always takes the fluid, retry beside with a null hit result when sneaking onto a non-air block. **PASS** (1.21.7 G02 WITHDRAWN — see §5.2).

### §3.7 Potions (B24, B26) — 2 gap

- **B24 (water splash/lingering douse):** vanilla 1.21.11 `AbstractThrownPotion` has ZERO `PotionTags` references — it gates on `PotionContents.is(Potions.WATER)` directly (javap-verified fresh; the 1.21.11 jar has no `PotionTags` class at all and no `data/minecraft/tags/potion/`). The template gates all four effects on potion tags (game/splash_water.rs:42-50 `potion_in_tag`: douses :60, hurts :134, extinguishes :135, **rehydrates :197**), which always return false, so water splash/lingering bottles do nothing on impact. **GAP — B7-G10 (major, reproduces from 1.21.7, 1.21.10 and 26.1.2; suspected duplicate of B1-G03 — coordinator to fold; the 1.21.11 Batch 1 report has not landed).** Note: unlike the 1.21.7 item's title ("except axolotl rehydrate"), the 1.21.11 port gates the rehydrate effect on tags too — the qualifier does not hold here.
- **B26 (water bottle on dirt → mud):** vanilla 1.21.11 `PotionItem.useOn` uses `BlockTags.CONVERTABLE_TO_MUD` (with an A, javap-verified fresh, `PotionItem.txt:55`); the template uses `convertible_to_mud` (with an I) in all four places (game/potions.rs:710, game/dispenser.rs:2251, game/remote.rs:8676, horizon-world/src/level/data.rs:458), so the lookup always misses and a water bottle on dirt never makes mud. **GAP — B7-G11 (minor, reproduces from 1.21.7, 1.21.10 and 26.1.2; suspected duplicate of B5-G19 — coordinator to fold; the 1.21.11 Batch 5 report has not landed).**

### §3.8 Boats (B27) — GAP

**B27 (boat placement eye check):** vanilla 1.21.11 `BoatItem.use` gathers entities in the player's bbox expanded 5.0 along the view + inflate 1.0 and returns PASS when the eye is inside any entity's pick-inflated bbox (javap-verified fresh, `BoatItem.txt:47-78`: `getEyePosition`, `AABB.inflate`, `InteractionResult.PASS`). The template `boat_to_place` (game/boats.rs:1518-1542) + `boat_entity_collision` (game/boat_passengers.rs:206) check only the boat's own bbox, so a boat is placeable while the player's eye is inside an entity. **GAP — B7-G12 (minor, reproduces from 1.21.7, 1.21.10 and 26.1.2).**

### §3.9 Ender pearl / wind charge cooldown (B30) — PASS

**B30 (throw cooldown):** vanilla 1.21.11 `ItemStack.use` calls `applyAfterUseComponentSideEffects` for instant-use items on Success, which reads `USE_COOLDOWN` — pearls (1.0s) and wind charges (0.5s) DO get a cooldown (javap-verified fresh, `ItemStack.txt:537-575`; the datagen report carries `use_cooldown` 1.0s on ender_pearl, 0.5s on wind_charge). The template still applies it: `throw_item` (game/creatures.rs:7308 wind charge, :7356 pearl) calls `apply_use_cooldown` (game.rs:5211), which reads the `use_cooldown` component. **PASS — B7-G16 stays FIXED (as in 1.21.10/26.1.2).**

### §3.10 Copper golem spawn egg (B33) — PASS

**B33 (copper golem spawn egg):** the egg exists in the datagen report (with an `entity_data` component) and maps to `SpawnEggItem`; the port's `Species` has `copper_golem` (mobs/species.rs:660), `use_spawn_egg` derives the kind from the item name, and the dispenser path marks copper golems persistent (game/dispenser.rs:1033) — matching vanilla, whose `CopperGolem` constructor calls `setPersistenceRequired()` (`CopperGolem.txt:93`). No copper-golem-specific defect was found; the generic spawn-egg defects (G03 entity_data override, G06 ambient sound) apply to this egg too and are recorded there. **PASS.**

### §3.11 Ominous bottle / trial key (B34) — PASS

**B34 (ominous bottle / trial key):** ominous bottle is plain `Item` with `consumable` + `ominous_bottle_amplifier` components in the datagen report; the port handles drinking it (`game/potions.rs:769-809` → `drink_ominous_bottle`, game/raids.rs:1475). Trial keys and ominous trial keys are plain `Item`s, handled by the vault/spawner code (game/spawner.rs:495-498). No defect found in item scope; vault behaviour itself is Batch 5's area. **PASS.**

### §3.12 Remaining behaviour checks — all PASS

- **B3 (food consume):** consume path verified against the jar's `Consumable`/`FoodData` shape; saturation clamps to the food level (game/consume.rs:63); food data matches D1. **PASS.**
- **B4 (tool use):** tool components match D4/D5; block breaking reads `damage_per_block` (game/block_breaking.rs:9). **PASS.**
- **B5 (armour equip):** armour data matches D7; equippable handling present (`item_sync::equippable`, used at game/armor_stands.rs:283-284). **PASS.**
- **B7 (fluid use / consume / offspring / entity_data):** spawn-egg fluid path in `use_spawn_egg` (game/creatures.rs:7453-7459); offspring path present (`spawn_egg_offspring`, game/creatures.rs:7386); entity_data application onto the spawned mob present (`apply_spawn_egg_entity_data`, game/creatures.rs:7532). **PASS.**
- **B17 (bundle use):** bundle weight uses `Fraction` with the 1/16 bundle-in-bundle weight (horizon-items/src/bundle.rs:75). **PASS.**
- **B19 (fishing rod):** wait 100–600 ticks minus Lure, nibble 20–40 ticks (game/fishing.rs:118, 247). **PASS.**
- **B20 (crossbow):** charge duration `(1.25 − 0.25×QuickCharge)` s, sounds at charge 0.2/0.5 (game/crossbow.rs:67-81). **PASS.**
- **B21 (shield block):** `blocks_attacks` loader defaults match the jar (90° horizontal angle default, horizon-items/src/items.rs:325). **PASS.**
- **B22 (trident throw):** riptide gate (wet + not riding) and spin strength `1.5 + 0.75×(level−1)` (game/trident.rs:519-520). **PASS.**
- **B23 (mace smash):** fall threshold 1.5, knockback radius 3.5 (game/mace.rs:20-24). **PASS.**
- **B25 (item use order):** `use_hand` dispatch present (game/use_hand.rs:8); bow release power `(f² + 2f)/3` clamped to 1.0, 0.1 minimum (game/item_use.rs:235-243) matches `BowItem.getPowerForTime`. **PASS.**
- **B28 (mount menu):** horse inventory menu present (game/mount_menu.rs). Note: unlike 1.21.10, the nautilus here are REAL — `NautilusInventoryMenu` exists and nautilus armour slots are handled; the 1.21.10 report's dead-nautilus note does not apply to 1.21.11. **PASS.**
- **B29 (world commands item paths):** item paths present in game/commands/world_cmds.rs (48 fns). **PASS.**
- **B31 (composter — cited):** composter is dead in 1.21.11 (`compostable` read at game/composter.rs:24-25; 0 items carry `minecraft:compostable`, and the component does not exist in 1.21.11's `DataComponents` at all) — Batch 5's defect (B5-G22 in the 26.1.2 run), cited, not re-minted. Vanilla composts from a hardcoded `COMPOSTABLES` Object2FloatMap (`ComposterBlock.txt:13`), not a component. **PASS for this batch's tally** (the defect is Batch 5's; this batch does not double-count it).
- **B32 (villager food):** vanilla 1.21.11's `DataComponents` has no `villager_food` (0 matches in the dump) and 0/1,505 items carry it; the tree reads it via horizon-items/src/items.rs:224 / game/villager.rs:408 but it never fires. **PASS.**

### §3.13 Initial / respawn spawn (B36) — GAP

**B36 (spawn in skylight-less dimensions):** vanilla 1.21.11 `PlayerSpawnFinder.findSpawn` checks `dimensionType().hasSkyLight()` FIRST: in a dimension without skylight (or in adventure mode) it returns `fixupSpawnHeight` at the suggestion at once, with no radius search (javap-verified fresh, server-game.jar, `PlayerSpawnFinder.txt:69-83`). The template (game/initial_spawn.rs:294-298 `spawn_finder`) has the adventure-mode arm but dropped the `!sky_light` arm (no `sky_light`/`has_sky` anywhere in the file), so in the Nether/End it searches the respawn radius instead of returning the fixup height. **GAP — B7-G19 (minor, reproduces from 1.21.10/26.1.2).**

### §3.14 Spear (B37) — PASS — NEW check (1.21.11-only content)

**B37 (spear kinetic/piercing):** vanilla 1.21.11 spears are plain `Item` with `kinetic_weapon`/`piercing_weapon` components (`world/item/component/{KineticWeapon,PiercingWeapon,UseEffects}` exist in the jar; no `SpearItem` class). The template's `spear_tick` (game/spear.rs:151-200) mirrors vanilla `KineticWeapon.damageEntities`: delay gate, relative speed `max(0)`, the three condition tests, `1.0 + floor(relative × damage_multiplier)` at :195, stabbed-recency via `contact_cooldown_ticks`, broadcast + trigger feedback. **PASS.** (Data side: D4 spear kinetic 7 spears × 8 values, 0 mismatches.)

## §4 Defect list

### Gap items (11)

| ID | Sev | Title |
|---|---|---|
| B7-G03 | minor | Spawn egg `entity_data` id type override ignored |
| B7-G06 | minor | No `Mob.playAmbientSound()` on spawn-egg placement |
| B7-G07 | major | Axe/hoe/shovel transforms and honeycomb wax-on entirely non-functional (26.2/26.3 `block_transformer` form) |
| B7-G08 | major | Shovel campfire douse dead: gated on the absent `#douses_campfires` tag |
| B7-G09 | major | Chorus fruit never teleports (tag `#entities_can_teleport_to` absent in 1.21.11) |
| B7-G10 | major | Water splash/lingering bottles do nothing on impact (potion-tag gate dead, incl. rehydrate) — suspected duplicate of B1-G03 |
| B7-G11 | minor | Water bottle on dirt never makes mud (`convertible` vs `convertable`) — suspected duplicate of B5-G19 |
| B7-G12 | minor | Boat placement missing the eye-inside-entity PASS check |
| B7-G17 | minor | Op's spawn-egg `entity_data` applied to spawners (1.21.11 has no `Spawner.setEntityData`) — suspected duplicate of B3-G39 |
| B7-G18 | minor | Dispenser spawn egg returned unshrunk with no `ENTITY_PLACE` on a null spawn |
| B7-G19 | minor | Respawn/initial spawn searches the radius in skylight-less dimensions instead of the fixup height |

### Fixed items (6)

| ID | Sev | Title |
|---|---|---|
| B7-G01 | minor | `ComponentMap::set` returns null (as `Ok`) on a dangling removal marker (FIXED in the 1.21.11 port) |
| B7-G04 | minor | Peaceful gate for hostile spawn eggs (vanilla 1.21.11 HAS the gate; the port HAS it) |
| B7-G13 | minor | Shield blocking-intent check (masked by B7-G07; code still wrong) |
| B7-G14 | minor | `ITEM_USED_ON_BLOCK` trigger for hoe/shovel/campfire (masked by B7-G07/B7-G08; code still wrong) |
| B7-G15 | minor | Copper-chest neighbour `BLOCK_CHANGE` (masked by B7-G07; code still wrong) |
| B7-G16 | minor | Ender pearl / wind charge use cooldown applied on throw (stays fixed) |

### Closed items (2)

| ID | Sev | Title |
|---|---|---|
| B7-G02 | minor | [CLOSED] Bucket sneak-redirect: vanilla 1.21.11 HAS the sneak check |
| B7-G05 | minor | [CLOSED] Spawn-egg `ENTITY_PLACE` position: vanilla posts at the spawn position |

### Oracle-defect items (39)

29 reproduce from 1.21.10 (O01–O08, O10–O30; measured triage, root causes javap-verified fresh against 1.21.11's jars and the 1.21.11 oracle sources — see §6) + 10 NEW in this run (O31–O40; they failed in the 1.21.10 measured table too but no 1.21.10 batch recorded them).

| ID | Sev | Oracle / root cause |
|---|---|---|
| B7-O01 | major | `VanillaDispenserOracle` — `EntityTypes` package does not exist (19 errors) |
| B7-O02 | major | `VanillaBlockUpdateOracle` — `ClockInstance` does not exist (38 errors); cascade root for O29 |
| B7-O03 | minor | `VanillaArrowOracle` — `Level.getNextEntityId` + `is()` overrides + 10-arg `ServerLevel` super (5 errors) |
| B7-O04 | minor | `VanillaBoatRiderOracle` — 10-arg `ServerLevel` super + `getNextEntityId` + `LocalPlayer` ctor (5 errors) |
| B7-O05 | minor | `VanillaClientArrowOracle` — `getNextEntityId` override (1 error) |
| B7-O06 | minor | `VanillaExplosionItemsOracle` — 10-arg `ServerLevel` super + `getNextEntityId` (2 errors) |
| B7-O07 | minor | `VanillaLavaCauldronItemOracle` — 10-arg `ServerLevel` super + `getNextEntityId` (2 errors) |
| B7-O08 | minor | `VanillaBoatAuthorityOracle` — `EntityTypes` does not exist (5 errors) |
| B7-O10 | minor | `VanillaVehicleContainerOracle` — `EntitySpawnRequest` does not exist (5 errors) |
| B7-O11 | minor | `KnockbackClient` — NEW causes: `canUseCommandBlocks` override + `EntityTypes.PLAYER` + `ServerboundAcceptTeleportationPacket` 6-arg ctor (7 errors; 0 `ServerboundPunchPacket` refs) |
| B7-O12 | minor | `VanillaConsumableOracle` — 10-arg `ServerLevel` super + `Success.shouldSwing()` does not exist (5 errors) |
| B7-O13 | minor | `VanillaTeleportConsumeOracle` — 5-arg `randomTeleport` does not exist (10 errors) |
| B7-O14 | minor | `VanillaItemBlockingOracle` — 10-arg `ServerLevel` super + 5-arg `blockUsingItem` does not exist (4 errors) |
| B7-O15–O16 | minor | `hurtAndBreak` takes `Consumer<Item>` (ItemDurability 1 error, ComponentDurability 2 errors) |
| B7-O17 | minor | `VanillaBlockingItemDamageOracle` — 10-arg `ServerLevel` super + `onEquippedItemBroken` takes `Item` (3 errors) |
| B7-O18 | minor | `VanillaExtraItemsOracle` — 3-arg `Player.drop` does not exist; `Prediction` + `ItemStackTemplate` also absent (8 errors) |
| B7-O19–O20 | minor | `ServerChunkCache` takes 11 constructor args (ItemUseStart 2 errors, InventoryUseStart 3 errors; both also 10-arg `ServerLevel` super) |
| B7-O21–O22 | minor | `Gui` takes 1 constructor arg (HeldDrop 8 errors, InteractionInput 6 errors) |
| B7-O23–O24 | minor | `ItemStackTemplate.fromStack` does not exist — the class itself is absent (ItemTemplate 8 errors, UseRemainder 8 errors) |
| B7-O25–O27 | minor | `VanillaRegistries.createWorldLookup` does not exist (ItemUseCatalog 5 errors, InventoryPickup 2 errors, PlayerInventory 2 errors) |
| B7-O28 | minor | `VanillaHeldItemStackOracle` — `LivingEntity.SwingDescription` does not exist (5 errors) |
| B7-O29 | minor | `VanillaEnchantDurabilityOracle` — cascade on broken `VanillaBlockUpdateOracle` |
| B7-O30 | minor | `VanillaHelmetDamageOracle` — 10-arg `ServerLevel` super (1 OWN error, not a pure cascade; also refs `VanillaBlockUpdateOracle`) |
| B7-O31 | minor | **NEW** `VanillaBundleWeightOracle` — `ItemStackTemplate` + `bindComponents` absent; `BundleContents.weight()` is a `Fraction` field (7 errors) |
| B7-O32 | minor | **NEW** `VanillaCreativeMiningOracle` — `Holder.Reference.bindComponents` does not exist (1 error) |
| B7-O33 | minor | **NEW** `VanillaItemAfterUseOracle` — `ItemStackTemplate` + `bindComponents` absent (6 errors) |
| B7-O34 | minor | **NEW** `VanillaItemCooldownsOracle` — `Holder.Reference.bindComponents` does not exist (1 error) |
| B7-O35 | minor | **NEW** `VanillaItemWireOracle` — `DataComponents.ADDITIONAL_TRADE_COST` absent + `JukeboxPlayable` takes `EitherHolder<JukeboxSong>` (2 errors) |
| B7-O36 | minor | **NEW** `VanillaPlayerEquipmentOracle` — `Holder.Reference.bindComponents` does not exist (1 error) |
| B7-O37 | minor | **NEW** `VanillaStackMiningOracle` — `ItemParser.parse` returns `ItemResult` (no `createItemStack`); 1.21.11 form is `ItemInput.createItemStack(int, boolean)` (1 error) |
| B7-O38 | minor | **NEW** `VanillaConsumableEmissionOracle` — 10-arg `ServerLevel` super + `ItemParticleOption.getItem()` returns `ItemStack` (no `item()`/`count()`) + `bindComponents` absent (4 errors) |
| B7-O39 | minor | **NEW** `VanillaInventoryUseOracle` — 10-arg `ServerLevel` super + `bindComponents` absent (2 errors) |
| B7-O40 | minor | **NEW** `VanillaUseFlagReactionOracle` — 10-arg `ServerLevel` super + `bindComponents` absent (2 errors) |

### Oracle fixed (1)

| ID | Sev | Title |
|---|---|---|
| B7-O09 | minor | `VanillaItemModelOracle` FIXED: adapted to 1.21.11's model API (`client/renderer/block/model/{BakedQuad,BlockModel,ItemModelGenerator,ItemTransform}`); passes on the client and both classpaths; all 30 imports exist in the 1.21.11 client jar |

## §5 Re-check verdicts

### §5.1 1.21.10 items (49, PRIMARY)

Every item was re-decided fresh against the 1.21.11 jar and tree. The 1.21.11 tree IS the template 1.21.10 derives from, so no verdict carries by identity — each rests on fresh evidence cited in §3/§6. The brief's mandatory primary list named 40 of these 49; the 9 it omitted (G10, G11, G17, O01, O02, O07, O09, O10, O29) are marked † and were decided as in-scope anyway.

| ID | 1.21.10 verdict | 1.21.11 verdict | Notes |
|---|---|---|---|
| G01 | fixed | **FIXED** | `component_patch.rs:109-122` returns `Ok(base)` on a dangling marker; doc cites 1.21.11 vanilla |
| G02 | closed | **WITHDRAWN** | Vanilla 1.21.11 `BucketItem` HAS the sneak check; closed item |
| G03 | gap | **REPRODUCES** | `use_spawn_egg` still derives kind from the item name |
| G04 | fixed | **FIXED** | Gate present (creatures.rs:7441-7465); hoglin residual is B3-G04 |
| G05 | closed | **WITHDRAWN** | Vanilla posts `ENTITY_PLACE` at the spawn position; closed item |
| G06 | gap | **REPRODUCES** | `ambient_sound` still dispenser/tick-only |
| G07 | gap | **REPRODUCES** | Re-check-first regression; transform table still empty |
| G08 | gap | **REPRODUCES** | Re-check-first regression; `#douses_campfires` gate still dead |
| G09 | gap | **REPRODUCES** | Tag absent; consume path unchanged; 2017 trail still masked |
| G10 † | gap | **REPRODUCES** | `splash_water.rs` gates on potion tags incl. rehydrate; suspected duplicate of B1-G03 |
| G11 † | gap | **REPRODUCES** | All four `convertible_to_mud` sites present; suspected duplicate of B5-G19 |
| G12 | gap | **REPRODUCES** | `boats.rs` has no eye check |
| G13 | fixed | **FIXED (masked)** | Code still wrong; unreachable behind G07 — returns if G07 is fixed |
| G14 | fixed | **FIXED (masked)** | Code still wrong; unreachable behind G07/G08 — returns if those are fixed |
| G15 | fixed | **FIXED (masked)** | Code still wrong; unreachable behind G07 — returns if G07 is fixed |
| G16 | fixed | **FIXED** | Cooldown still applied (creatures.rs:7308, 7356) |
| G17 † | gap | **REPRODUCES** | Both spawner arms present; suspected duplicate of B3-G39 |
| G18 | gap | **REPRODUCES** | Null spawn still returns the stack unshrunk, no event |
| G19 | gap | **REPRODUCES** | `!sky_light` arm still missing from `spawn_finder` |
| O01 † | oracle-defect | **REPRODUCES** | `EntityTypes` still referenced; 19 errors measured |
| O02 † | oracle-defect | **REPRODUCES** | `ClockInstance` still referenced; 38 errors measured; cascade root |
| O03–O06 | oracle-defect | **REPRODUCE** | Root causes javap-verified fresh; multi-cause in 1.21.11 (see §6) |
| O07 † | oracle-defect | **REPRODUCES** | 10-arg `ServerLevel` super + `getNextEntityId`; 2 errors measured |
| O08 | oracle-defect | **REPRODUCES** | `EntityTypes` still referenced; 5 errors measured |
| O09 † | oracle-defect | **FIXED** | Template adapted it to 1.21.11's model API; passes on client + both |
| O10 † | oracle-defect | **REPRODUCES** | `EntitySpawnRequest` still referenced; 5 errors measured |
| O11 | oracle-defect (1.21.10 report said "fixed" — provisional error, see §6) | **REPRODUCES** | Punch cause gone; 7 NEW errors (see §6) |
| O12–O28 | oracle-defect | **REPRODUCE** | Root causes javap-verified fresh (see §6) |
| O29 † | oracle-defect | **REPRODUCES** | Cascade on O02 (measured, not assumed) |
| O30 | oracle-defect | **REPRODUCES** | 1 OWN error (10-arg `ServerLevel` super), not a pure cascade |

### §5.2 1.21.7 items (12, SECONDARY)

| ID | 1.21.7 verdict | 1.21.11 verdict | Notes |
|---|---|---|---|
| G01 | gap | **FIXED** | Port fixed it; same item as 1.21.10 G01 |
| G02 | gap | **WITHDRAWN** | Vanilla 1.21.11 `BucketItem` HAS the sneak check (1.21.7 did not); closed item |
| G03 | gap | **REPRODUCES** | Same defect as 1.21.10 G03 (one item) |
| G04 | gap | **FIXED** | Vanilla 1.21.11 HAS the gate and the port HAS it; same item as 1.21.10 G04 |
| G05 | gap | **WITHDRAWN** | Vanilla 1.21.11 posts `ENTITY_PLACE` at the spawn position (1.21.7 posted at the clicked block); closed item |
| G06 | gap | **REPRODUCES** | Same defect as 1.21.10 G06 (one item) |
| G07 | gap | **REPRODUCES** | Same defect as 1.21.10 G07 (one item); re-check-first regression |
| G08 | gap | **REPRODUCES** | Same defect as 1.21.10 G08 (one item); re-check-first regression |
| G09 | gap | **REPRODUCES** | Same defect as 1.21.10 G09 (one item) |
| G10 | gap | **REPRODUCES** | Same defect as 1.21.10 G10 (one item); rehydrate also tag-gated in 1.21.11 |
| G11 | gap | **REPRODUCES** | Same defect as 1.21.10 G11 (one item) |
| G12 | gap | **REPRODUCES** | Same defect as 1.21.10 G12 (one item) |

## §6 Oracle triage

**MEASURED** — the 1.21.11 Batch 0 table has not landed, so this batch ran its own javac fixpoint measurement over all 391 oracle files on three classpaths (`evidence/measure_oracles_1_21_11.py`, `evidence/oracle-measure/results.json`; classpaths use the brief-pinned named jars + libraries from `.local/minecraft/1.21.11/libraries/` (39 server + 60 client jars) + jsr305 from the behaviour worktree, read-only; javac `--release 21`, JDK 25): **both 127/391 pass, client 127/391 pass, server 102/391 pass.** Client-class error classes: missing-package 39, missing-symbol 103, no-override 13, constructor-mismatch 82, incompatible-types 4, other 7, cascade 15, missing-override 1. Of the 30 in-scope oracles: **29 fail, 1 passes (O09)** — on the client classpath, the same 29 fail and O09 passes; on the server classpath all 30 fail (O09's server failure is client-package imports only — 136 errors, all `net.minecraft.client.*`/`blaze3d` — expected for a client oracle). Every root cause below was re-verified two ways against 1.21.11: the oracle source in the 1.21.11 tree still references the API, and the API is absent/different in 1.21.11's own jar. 20 of the 30 oracle files are byte-identical to 1.21.10's; 9 differ only by the naming adaptation in reverse (`ResourceLocation` → `Identifier` etc.); O09 is the substantive adaptation.

- **O01/O08** — `EntityTypes` does not exist in the 1.21.11 client jar (registration still in `EntityType`). Sources still reference it (`VanillaDispenserOracle.java:212,235-236,560`; `VanillaBoatAuthorityOracle.java:14,53,96`).
- **O02** — `ClockInstance` does not exist in the 1.21.11 client jar (no `world/clock/` package); source still references it (`VanillaBlockUpdateOracle.java:450`). Cascade root for O29.
- **O03–O07** — `Level.getNextEntityId` does not exist in 1.21.11 (0 matches in the `Level` dump); all five sources still `@Override` it — but in the measured picture this is no longer the whole story: O03 also overrides `is()` and calls a 10-arg `ServerLevel` super (:75), O04 also calls a 10-arg `ServerLevel` super (:36) and a wrong `LocalPlayer` ctor (:55), O06/O07 also call a 10-arg `ServerLevel` super (:28/:27).
- **O09 FIXED** — the 1.21.11 `VanillaItemModelOracle.java` was adapted: it imports `client/renderer/block/model/{BakedQuad,BlockModel,ItemModelGenerator,ItemTransform}` (all exist in the jar), dropped the `client.resources.model.geometry/cuboid/sprite` imports and the `EntityTypes` import; all 30 imports verified present in the client jar. Passes on client and both classpaths.
- **O10** — `EntitySpawnRequest` does not exist; source imports and uses it (`VanillaVehicleContainerOracle.java:20,118`).
- **O11 REPRODUCES with NEW causes** — the 1.21.10 report's "FIXED" was a provisional-triage error: it checked only that `ServerboundPunchPacket` was gone and the imports existed. The file is byte-identical to 1.21.10's, and the measured 1.21.10 table already shows it failing (no-override). In 1.21.11 it fails with 7 errors: `canUseCommandBlocks` override (:362), `EntityTypes.PLAYER` (:383), cannot-find-symbol (:394/:396/:397), `ServerboundAcceptTeleportationPacket` 6-arg ctor (:436). 0 `ServerboundPunchPacket` refs — punch sends `ServerboundSwingPacket` (:144/:308), so the original cause is gone and new causes replace it, exactly as the brief framed it.
- **O12** — `InteractionResult.Success` is a record with a `swingSource` field; no `shouldSwing()` method; source calls `s.shouldSwing()` (`VanillaConsumableOracle.java:76`) — plus a 10-arg `ServerLevel` super (:69).
- **O13** — `LivingEntity.randomTeleport` is 4-arg `(DDDZ)`, no tag parameter; source overrides the 5-arg form with a `TagKey<Block>` (`VanillaTeleportConsumeOracle.java:27,30,38,41`). Same 26.2-form tag parameter behind B7-G09.
- **O14** — `LivingEntity.blockUsingItem` is 2-arg `(ServerLevel, LivingEntity)`; source overrides the 5-arg form (`VanillaItemBlockingOracle.java:29,38`) — plus a 10-arg `ServerLevel` super (:21).
- **O15/O16** — `ItemStack.hurtAndBreak` takes `Consumer<Item>`, not `Consumer<ItemStack>` (`ItemStack.txt:741`); both sources pass the lambda result to `snapshot(ItemStack)`.
- **O17** — `LivingEntity.onEquippedItemBroken` takes `(Item, EquipmentSlot)` (`LivingEntity.txt:10240`); source overrides `(ItemStack, EquipmentSlot)` — plus a 10-arg `ServerLevel` super (:17).
- **O18** — `Player.drop` is 2-arg `(ItemStack, boolean)` (`Player.txt:1161`); source overrides the 3-arg form with `Prediction`, a class that does not exist in either 1.21.11 jar; `ItemStackTemplate` also absent (:12/:13).
- **O19/O20** — `ServerChunkCache` takes 11 constructor args in 1.21.11 (`ServerChunkCache.txt:39`); both sources call a 10-arg super — and both also call a 10-arg `ServerLevel` super (:17/:19).
- **O21/O22** — `Gui` takes 1 arg (`Gui(Minecraft)`, `Gui.txt:181`); both sources call `super(null, null, null)` from a `Gui` subclass (:38/:39).
- **O23/O24** — `ItemStackTemplate` does not exist at all in 1.21.11; both sources call `ItemStackTemplate.fromStack` (:25/:15).
- **O25–O27** — `VanillaRegistries` has `createLookup()`, not `createWorldLookup` (`VanillaRegistries.txt:39`); all three sources call `createWorldLookup`. (O25 also has an `Instrument` ctor error at :25.)
- **O28** — `SwingDescription` does not exist anywhere in the 1.21.11 client jar and `LivingEntity` has no `getCurrentSwing`; source overrides `getCurrentSwing()` returning `SwingDescription` (:30/:31) — plus a wrong `LocalPlayer` ctor (:23). Vanilla `LocalPlayer` ctor is 7-arg (`Minecraft, ClientLevel, ClientPacketListener, StatsCounter, ClientRecipeBook, Input, boolean`).
- **O29** — cascade on O02 (measured: the fixpoint drops it after O02; source references `VanillaBlockUpdateOracle` at :27-33).
- **O30** — NOT a pure cascade: it carries 1 OWN error, the 10-arg `ServerLevel` super (:34), though it also references `VanillaBlockUpdateOracle` (:33/:37/:97). The 1.21.10 report's "cascades on O02" grouping for O30 was provisional; the measured picture corrects it.
- **O31–O40 (NEW)** — all ten failed in the 1.21.10 measured table too but were recorded by no 1.21.10 batch (the batches triaged provisionally from the 26.1.2 table, where they passed). Root causes, all javap-verified: `Holder.Reference.bindComponents(DataComponentMap)` does not exist in 1.21.11 (javap of `Holder$Reference` shows no such method — 25 oracle files carry the error tree-wide, 33 occurrences; the 1.21.10 run recorded per-oracle items for this cause, e.g. B4-O11/B4-O12, so per-oracle items are the right pattern here too) — O31 (:8), O32 (:36), O33 (:23/:26/:27), O34 (:24), O36 (:21), O38 (:35), O39 (:35), O40 (:36). O31 additionally uses the absent `ItemStackTemplate` (:13) and `BundleContents.weight()` as a `DataResult` (in 1.21.11 `weight` is a `Fraction` field and the ctor takes `List<ItemStack>`). O35: `DataComponents.ADDITIONAL_TRADE_COST` absent from the 105-field `DataComponents`, and `JukeboxPlayable` takes `EitherHolder<JukeboxSong>` (:83). O37: `ItemParser.parse` returns `ItemParser$ItemResult` (no `createItemStack`); the 1.21.11 form is `ItemInput.createItemStack(int, boolean)`. O38 additionally: `ItemParticleOption.getItem()` returns `ItemStack` in 1.21.11, so `t.item()`/`t.count()` (:33) fail. O38/O39/O40 also carry the 10-arg `ServerLevel` super.
- **NOT minted (Batch 0 group territory):** `VanillaItemUseLifecycleOracle` and `VanillaPlayerFoodOracle` fail with the 10-arg `ServerLevel` super as their ONLY error — they belong to Batch 0's ServerLevel-ctor group item (85 files carry that error in 1.21.11; the 1.21.10 run's Batch 0 grouped it as O10, 86 oracles). The coordinator should confirm Batch 0 re-mints the group for 1.21.11. Five more failing family oracles are owned by other batches' 1.21.10 items (`VanillaEnchantingOracle` B4-O07, `VanillaItemStackStorageOracle` B4-O12, `VanillaStoredEnchantmentPatchOracle` B4-O20, `VanillaTradeComponentsOracle` B3-O05/B4-O27, plus `VanillaCombatRulesOracle`/`VanillaConsumeEffectOracle`/`VanillaDamageReductionOracle`/`VanillaFoodPropertiesOracle`/`VanillaStoredConsumeOracle` in Batch 0's O10 group) and are cited in §7, not re-minted here.

Passing family oracles (measured, client cp): 12 of the 59 distinct oracles the 74 drivers reference pass — `VanillaBlockingRulesOracle`, `VanillaComponentPatchOracle`, `VanillaEnchantmentComponentsOracle`, `VanillaFoodOracle`, `VanillaFoodStorageOracle`, `VanillaHotbarOracle`, `VanillaInventoryTickOracle`, `VanillaItemUseOracle`, `VanillaMapSavedDataOracle`, `VanillaPickBlockDataOracle`, `VanillaStackFoilOracle`, `VanillaStackModelFlagsOracle` — plus `VanillaItemModelOracle` (O09), which passes but is referenced by no Batch 7 driver.

## §7 Driver inventory

74 drivers — **11 runnable / 46 blocked / 17 no-oracle** — MEASURED (from the 1.21.11 client-column table; `evidence/driver-inventory-1.21.11.json`, `evidence/driver_inventory_1_21_11.out`). All 74 driver files exist in the 1.21.11 tree; the only oracle-set mismatch vs the 26.2 map is the known `interaction_input`/`ParityData` helper reference. Runnable: `food`, `food_storage`, `item_use`, `blocking_rules`, `component_patch`, `enchantment_components`, `stack_foil`, `pick_block_data`, `map_saved_data`, `hotbar`, `inventory_tick`. 47 distinct oracles block the 46 blocked drivers; **none of the 47 passed in the 1.21.10 measured table** — the 1.21.10 run's 33-runnable figure was a provisional-table artifact (its measured table shows the same failures), not a 1.21.11 regression. **No driver was run**, per the brief; the 11 runnable drivers are recorded as not run, not as passes.

## §8 Systemic findings

1. **The template fixed one oracle and kept every behaviour defect.** `VanillaItemModelOracle` (O09) was substantively adapted for 1.21.11's model API and passes. But all 11 behaviour gap items from 1.21.10 reproduce verbatim in the template — including the four re-check-first regressions (G07, G08, G09, G10). The template's fixes were selective, not systemic; 1.21.10 inherited the defects from the template, not the other way round.
2. **The 1.21.10 run's O11 "FIXED" was wrong, and the measured table proves it.** `KnockbackClient` is byte-identical between 1.21.10 and 1.21.11, fails in both measured tables (no-override), and fails in 1.21.11 with 7 errors. The 1.21.10 verdict rested on an import check + the absent punch packet, never a compile. Lesson for the coordinator: provisional oracle verdicts need a compile, not an import grep.
3. **The provisional-triage gap is bigger than O11.** Ten oracles (O31–O40) failed in the 1.21.10 measured table but were recorded by no 1.21.10 batch, because every batch triaged from the 26.1.2 table where they passed. Measured triage — as done here — is what closes that gap. The 1.21.11 Batch 0 table, when it lands, should be cross-checked against this batch's measurement (`evidence/oracle-measure/results.json`) rather than assumed to match.
4. **Vanilla changed between 1.21.7 and 1.21.10/1.21.11 in exactly the places that withdraw 1.21.7 items.** The bucket sneak check, the spawn-egg peaceful gate and the spawn-position `ENTITY_PLACE` all exist in 1.21.11's jar where 1.21.7 had none of them. The 1.21.7 sweep's G02/G04/G05 premises fail for 1.21.11 on the jar, not on the port — the port matches vanilla 1.21.11 in all three cases. Vanilla ALSO changed between 1.21.10 and 1.21.11 in item scope: mob buckets gained food components, parched joined the peaceful denylist (37 types), and spears/nautilus content arrived (B37, D4/D7).
5. **Absent-tag/component reads are unchanged in scope.** The template reads the same absent tags/components as the 1.21.10 tree did: `#douses_campfires`, `#entities_can_teleport_to` (in four places — chorus fruit at consume.rs:134, enderman at mobs/enderman.rs:117, spreadplayers-style code at game/commands/world_cmds.rs:2100-2105, pet teleport at mobs/tame.rs:1341), `#enderman_does_not_teleport_to`, `#consumable_does_not_teleport_to` (consume.rs:399, fox.rs:439), the `minecraft:block_transformer`, `minecraft:mob_visibility`, `minecraft:compostable` and `minecraft:villager_food` components. The tag table resolves absent tags to empty sets, so every read silently degrades to false, never an error.
6. **Masked defects are still a trap for fixers.** G13/G14/G15 (and the player 2017 trail under G09) are behaviourally fixed only because G07/G08/G09 kill the whole path. Any fix of G07, G08 or G09 must also fix the masked code or the defects reappear.
7. **1.21.11-record corrections (from this run):** (a) the 1.21.10 data-check script's `expand()` double-prefixes `"minecraft:"` on `#minecraft:...` refs — both sides equally wrong in D6 (see §2); the fix is in this run's script. (b) The 1.21.10 item→class parser misses spawn eggs in 1.21.11 (`registerSpawnEgg` has no indy factory) — the fix is in this run's script. (c) Vanilla 1.21.11 has no named `SpawnEggItemBehavior` class — the dispenser spawn-egg behaviour is the anonymous `DispenseItemBehavior$1` (the 26.x named class does not exist in 1.21.11). (d) The 1.21.7 G10 title's "except axolotl rehydrate" qualifier does not hold in 1.21.11 — the port gates rehydrate on tags too. (e) The 1.21.10 report's dead-`Parched` denylist note does not apply to 1.21.11 — parched is a real species here and correctly included; hoglin is the only wrong denylist entry.
8. **Out-of-scope flags (cited, not minted):** hoglin in the shared `allowed_in_peaceful` denylist → Batch 3 (B3-G04, including the spawn-egg facet); enderman teleport on the absent `#entities_can_teleport_to` (mobs/enderman.rs:117) → Batch 3 (B3-G14); mob-head visibility via the `minecraft:mob_visibility` component (mobs.rs:3173) → Batch 3 (B3-G33); composter → Batch 5 (B5-G22); `#entities_can_teleport_to` at game/commands/world_cmds.rs:2100-2105 → Batch 8 (commands); pet teleport at mobs/tame.rs:1341 → Batch 3; splash-water douse → Batch 1 (B1-G03, suspected duplicate of B7-G10); mud tag → Batch 5 (B5-G19, suspected duplicate of B7-G11); spawner entity_data → Batch 3 (B3-G39, suspected duplicate of B7-G17); the ServerLevel-ctor group (85 files) and the `bindComponents` group (25 files) → Batch 0. The 1.21.11 Batch 1/3/5 reports have not landed, so the three suspected duplicates are flagged for the coordinator to fold, not folded here. The four NEVER EXISTED corrections owed to other ledgers (B5-G17, B5-G18, B2-G11/G12, B10-G04) are outside Batch 7 scope; none was re-minted.

## §9 Method notes + evidence index

**Method.** Static analysis only: `javap -c -p` on the named 1.21.11 jars (`client-named.jar` for client classes, `server-game-named.jar` for server classes) for every vanilla method in scope; Rust source reads; data checks against Mojang's datagen report (no horizon-prepare output exists for 1.21.11); a diff of every in-scope oracle source between the 1.21.10 and 1.21.11 trees; and a full javac fixpoint measurement of all 391 oracle files on three classpaths. No drivers were run and no oracle repairs were done (checks + triage only). Jar sha1s verified against the brief pins before use.

**Evidence** (`~/workspace/mcir-work/sweep-1.21.11-batches/b07/evidence/`):

- `javap_dumps.sh`, `javap/` — 73 fresh 1.21.11 javap dumps (incl. `DispenseItemBehavior$1`–`$15` anonymous classes, `WeatheringCopperItems`, `ComposterBlock`), plus `client-named-list.txt` / `server-game-named-list.txt` / `client-list.txt` jar listings.
- `parse_items_class_1_21_11.py/.out`, `item-class.json`, `nonblock-by-class.json` — D8 map.
- `extract_materials_1_21_11.py`, `materials-1.21.11.json` — ToolMaterial/ArmorMaterials from the 1.21.11 jar.
- `data_checks_1_21_11.py/.out` — D1–D7.
- `measure_oracles_1_21_11.py`, `oracle-measure/results.json` + `big-compile-{both,server,client}.log` — the measured oracle table (127/391 both, 127/391 client, 102/391 server).
- `driver_inventory_1_21_11.py/.out`, `driver-inventory-1.21.11.json` — driver inventory (measured).

**Key jar evidence** (all from fresh 1.21.11 dumps in `b07/evidence/javap/`): `SpawnEggItem.txt:156-160` spawnMob peaceful gate; `:181` ENTITY_PLACE at the spawn pos; `:282-303` getType reads ENTITY_DATA; `EntityType.txt:557/565` playAmbientSound in spawn; 37 `notInPeaceful` types extracted (36 + parched; no hoglin). `LivingEntity.txt:9792` randomTeleport 4-arg `(DDDZ)`; `:3331` blockUsingItem 2-arg; `:10240` onEquippedItemBroken `(Item, EquipmentSlot)`. `ItemStack.txt:741` hurtAndBreak `Consumer<Item>`; `:537-575` applyAfterUseComponentSideEffects reads USE_COOLDOWN; `Player.txt:1161` drop 2-arg; `ServerLevel` ctor 11-arg ending `RandomSequences`; `ServerChunkCache.txt:39` 11-arg ctor; `Gui.txt:181` 1-arg ctor; `VanillaRegistries.txt:39` only `createLookup`; `InteractionResult_Success` record with `swingSource`, no `shouldSwing`; `Level.txt` 0 `getNextEntityId`; `DispenseItemBehavior-1.txt:69-112` execute pops spawn result, unconditionally shrink + ENTITY_PLACE; `PlayerSpawnFinder.txt:69-83` findSpawn skylight gate first; `TeleportRandomlyConsumeEffect.txt:106` apply calls 4-arg randomTeleport, only `diameter` field, 0 level-event refs; `BaseSpawner.txt:77` only `setEntityId` (0 `setEntityData` in any dump); `AxeItem` STRIPPABLES, `HoeItem` TILLABLES, `ShovelItem` FLATTENABLES, 0 `BlockTransformer` references in any of the three; `BucketItem.txt:231` isShiftKeyDown; `ShovelItem.txt:64-88` direct campfire dowse; `AbstractThrownPotion.txt` 0 `PotionTags` refs, `Potions.WATER` at :74 and :130; `PotionItem.txt:55` CONVERTABLE_TO_MUD; `BoatItem.txt:47-78` eye-inside-entity check; `CopperGolem.txt:93` setPersistenceRequired in the constructor; `ComposterBlock.txt:13` hardcoded `COMPOSTABLES` map; `Holder$Reference` no `bindComponents`; `ItemParticleOption.getItem()` returns `ItemStack`; `BundleContents` ctor takes `List<ItemStack>`, `weight` is a `Fraction` field; `ItemParser.parse` returns `ItemParser$ItemResult`; `ItemInput.createItemStack(int, boolean)`; `DataComponents` 105 fields, no `ADDITIONAL_TRADE_COST`/`VILLAGER_FOOD`/`BLOCK_TRANSFORMER`/`MOB_VISIBILITY`/`COMPOSTABLE`. Absent from both 1.21.11 jars: `PotionTags`, `ItemStackTemplate`, `EntityTypes`, `ClockInstance`, `EntitySpawnRequest`, `ServerboundPunchPacket`, `LivingEntity$SwingDescription`, `SpearItem`, `SpawnEggItemBehavior`, `Prediction`, `OminousBottleItem`, `TrialKeyItem`. Present (new/moved vs 1.21.10): `world/item/component/{KineticWeapon,PiercingWeapon,UseEffects}`; packages `world/level/gamerules/`, `world/entity/projectile/arrow/`, `world/entity/vehicle/boat/`, `world/entity/animal/fox/`, `world/entity/projectile/throwableitemprojectile/`; `Identifier` exists, `ResourceLocation` does not.

## §10 What this batch defers + verdict

**Deferred (not this batch's work):** driver runs (11 runnable drivers unrun), oracle repairs (39 failing oracles), all fixes (including the four re-check-first regressions G07/G08/G09/G10), verification of the horizon-prepare step for 1.21.11 (no prepared data exists; the data checks ran against Mojang's datagen report), cross-checking this batch's measured oracle table against the 1.21.11 Batch 0 table when it lands, and the out-of-scope flags in §8.8.

**Verdict: 45 checks — 31 pass, 14 gap.** The 1.21.11 template fixed one oracle the 1.21.10 run never had passing (B7-O09 `VanillaItemModelOracle`) and matches vanilla 1.21.11 where vanilla changed since 1.21.7 (bucket sneak, peaceful gate, ENTITY_PLACE position) — but every behaviour defect from 1.21.10 reproduces with fresh 1.21.11 evidence, including the four re-check-first regressions (B7-G07 tool transforms, B7-G08 campfire douse, B7-G09 chorus fruit, B7-G10 splash water). No new behaviour defect was found in scope. Ten oracle defects (B7-O31–O40) are new to this run's ledger, and the 1.21.10 run's B7-O11 "FIXED" is corrected to REPRODUCES (new causes). Three reproductions are suspected duplicates of other 1.21.11 batches (B1-G03, B5-G19, B3-G39) for the coordinator to fold once those reports land.
