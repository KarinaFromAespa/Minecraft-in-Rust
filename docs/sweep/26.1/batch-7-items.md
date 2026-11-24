# Batch 7 — Items (26.1 parity sweep)

**Tally: 42 behaviour checks — 29 pass, 13 gap.** Oracle triage is separate
(§5): 30 oracle defects (2 major, 28 minor), 36 passing in-scope oracles.
No driver was run; no production code was touched.

Scope: item use, food, tools, armour, buckets, spawn eggs, components —
1,506 items in 26.1 (not 26.2's 1,537). The brief's assumption that spears
and nautilus armour are 26.2-only families was wrong: all 7 spears and all 5
nautilus armours exist in 26.1, verified in both `items.json` and the jar's
`Items` class, so B33/B34 are full checks here, not stubs.

Counting rule: one check per behaviour pattern, grouped by behaviour class
per the brief (all tools of one kind are one check pattern, not N checks).
The 42 checks are D1–D8 (§2) and B1–B34 (§3): the 32 1.21.7 behaviour checks
plus B33 spears and B34 nautilus armour. A check is "gap" if it contains at
least one defect; B7-G13 and B7-G14 each span two gap checks (hoe + shovel),
and B7-G14 spans a third (campfire douse).

## 1. Item→class map — 1,506 items, 70 classes, 0 unmapped

Parser: `evidence/parse_items_class_26_1.py` (adapted from the 1.21.7
parser), run against the named 26.1 client jar's `Items` class
(`javap -v` + `javap -c` dumps in `evidence/items-v.txt`,
`evidence/items-clinit.txt`). **1,506 items → 70 classes, 0 unmapped,
0 mismatches** against prepared `block-behaviours.json`.

- 1,482 `putstatic Item` fields + 24 `WeatheringCopperItems` collection
  items (copper bars/chain/lantern × 8 oxidation/wax stages) via
  `WeatheringCopperItems.create` with an `Items.registerBlock` factory
  (indy #27) → BlockItem. No bulk `ColorCollection` calls in 26.1
  (unlike 26.2).
- `SWEET_BERRIES` resolves to BlockItem
  (`createBlockItemWithCustomItemName`); factory method-refs to
  `Items.registerBlock` resolve to BlockItem, not `Item`.
- Histogram: BlockItem 910, Item 235, SpawnEggItem 87, DoubleHighBlockItem
  28, StandingAndWallBlockItem 20, BoatItem 20, SmithingTemplateItem 19,
  BundleItem 17, DyeItem 16, BedItem 16, BannerItem 16, SignItem 12,
  HangingSignItem 12, GameMasterBlockItem 7, ShovelItem 7, AxeItem 7,
  HoeItem 7, MinecartItem 6, MobBucketItem 6, BucketItem 3, EggItem 3,
  PlaceOnWaterBlockItem 2, FoodOnAStickItem 2, ItemFrameItem 2, +
  43 singletons. No `SwordItem`/`PickaxeItem`/`ArmorItem`/`ElytraItem`
  classes in 26.1 (plain `Item` with components, as in 26.2).
  `milk_bucket` is a plain `Item`; `AIR` is `AirItem`; `ender_pearl` is
  `EnderpearlItem`.
- Spears (7) and nautilus armour (5) DO exist in 26.1 — see §3.17/§3.18.
  Copper horse armour also exists.
- `block_transformer`: 0 items (26.1 has no `block_transformer` data).
- `use_cooldown`: chorus_fruit 1.0s, ender_pearl 1.0s, wind_charge 0.5s.

## 2. Data checks — 8/8 pass

Script: `evidence/data_checks_26_1_v2.py`, output
`evidence/data_checks_26_1_v2.out`. All values below were read from the
26.1 jar (`Foods`, `ToolMaterial`, `ArmorMaterials`, `ArmorMaterial`,
`ArmorType`, `Item$Properties`, `BowItem`, `CrossbowItem`, `TridentItem`,
`MaceItem`, `Tool`, `Weapon`, `BlockTags` dumps in `evidence/`) and
compared against prepared `items.json` / `block-tags.json`.

- **D1 foods — 44/44 pass.** 40 `Foods` fields + 4 mob buckets carrying
  their fish's `Foods` values — jar-verified: `COD_BUCKET` has
  `.component(FOOD, Foods.COD)`. Saturation = f32-stepped
  `nutrition × modifier × 2`.
- **D2 max damage — 84/84 pass.**
- **D3 max stack — 1506/1506 pass.** `durability()` sets
  MAX_STACK_SIZE=1; horse/nautilus armour end `stacksTo(1)`.
- **D4 tool/spear attributes — 42 pairs pass.** Trident (8.0, −2.9), mace
  (5.0, −3.4). Spear kinetic: 7 spears × 8 values match, including the
  delay-tick rounding edges (copper 13, stone/golden 14). Spear args per
  material in the v1 output (e.g. iron (0.95, 0.95, 0.6, 2.5/11.0,
  6.75/5.1, 11.25/4.6)); attack speed = (1.0/duration) − 4.0, damage =
  material bonus.
- **D5 tool components — 35 pass.** Sword rules: cobweb 15.0,
  `#sword_instantly_mines` Float.MAX, `#sword_efficient` 1.5,
  damagePerBlock 2, canDestroyInCreative false. `applyToolProperties`:
  rules [deniesDrops(incorrect tag), minesAndDrops(mineable tag, material
  speed)], defaultMiningSpeed 1.0.
- **D6 tags — 13/13 pass.** The `shears_*_breaking_speed` tags do NOT
  exist in 26.1 (absent from `BlockTags`, both jars' datapacks, and
  prepared `block-tags.json`); they are a 26.2 addition.
- **D7 armour — 41/41 pass** (29 humanoid + 6 horse + 1 wolf +
  5 nautilus).
- **D8 item→class map — pass** (§1).

ToolMaterial [durability, speed, attackDamageBonus, enchantment]: WOOD
[59, 2.0, 0.0, 15], STONE [131, 4.0, 1.0, 5], COPPER [190, 5.0, 1.0, 13],
IRON [250, 6.0, 2.0, 14], DIAMOND [1561, 8.0, 3.0, 10], GOLD [32, 12.0,
0.0, 22], NETHERITE [2031, 9.0, 4.0, 15].

ArmorMaterials [durability, defense boots/legs/chest/helmet/body,
enchantment, toughness, kb-resist]: LEATHER [5, 1/2/3/1/3, 15, 0, 0],
COPPER [11, 1/3/4/2/4, 8, 0, 0], CHAINMAIL [15, 1/4/5/2/4, 12], IRON
[15, 2/5/6/2/5, 9], GOLD [7, 1/3/5/2/7, 25], DIAMOND [33, 3/6/8/3/11,
10, 2.0], TURTLE_SCUTE [25, 2/5/6/2/5, 9], NETHERITE [37, 3/6/8/3/19,
15, 3.0, 0.1], ARMADILLO_SCUTE [4, 3/6/8/3/11, 10]. `makeDefense` order
is (boots, legs, chest, helmet, body). Slot bases: HELMET 11, CHESTPLATE
16, LEGGINGS 15, BOOTS 13, BODY 16. Horse/nautilus: no durability, no
enchantable, stacksTo(1); wolf armour: durability 64 (4×16), no
enchantable.

Tag facts (both jars): `convertable_to_mud` exists (with an A);
`#douses_campfires` absent; `#entities_can_teleport_to` absent;
`tags/potion/` contains only `tradeable.json`; no `block_transformer`
data; `#can_wear_nautilus_armor` exists.

## 3. Behaviour checks — 21 pass, 13 gap

### 3.1 component_patch — gap (B1; B7-G01 reproduces)

Vanilla 26.1 `PatchedDataComponentMap.set` returns
`previous.orElse(prototype)` — null on a dangling removal marker, never an
error (javap-verified, `evidence/PatchedDataComponentMap.txt`). The 26.1
Rust `ComponentMap::set`
(`crates/horizon-world/src/component_patch.rs:106-119`, line 116) returns
`Err(MissingPrototype)` on a dangling removal marker, and the doc comment
above it is still wrong. **26.2 fixed this; 26.1 did not.** Minor.

### 3.2 Buckets — pass (B2, B3, B4; B7-G02 withdrawn)

Vanilla 26.1 `BucketItem.emptyContents` HAS the sneak check
(`isShiftKeyDown`, javap-verified, `evidence/BucketItem.txt`), and the
Rust `bucket_empty` (`horizon-world/src/level/buckets.rs`) matches: air
always takes the fluid, retry beside with a null hit result. The 1.21.7
premise (vanilla had no sneak check) fails for 26.1 — WITHDRAWN. Mob
bucket + cauldron (B4) also pass.

### 3.3 Bundles — pass (B5)

`BundleItem` API matches 26.2; the Rust bar width
`(1 + n*12/d).min(13)` and colours 0xFF5454/0x7087FF match the jar.

### 3.4 Spawn eggs — 2 pass, 4 gap (B6–B11)

- **B6 spawner use — gap (B7-G17, NEW, minor, op-only).** The Rust applies
  an op's egg `entity_data` to spawners
  (`block_entity_item.rs` `spawn_egg_on_spawner` +
  `game/spawner.rs` `use_spawn_egg_on_spawner` via `set_entity_data`);
  the 26.1 jar has NO `Spawner.setEntityData` — `SpawnEggItem.useOn`
  calls only `setEntityId`. 26.3 behaviour leaking into 26.1. (The 26.2
  report's spawner note did not cover the op-data path.)
- **B7 fluid use / consume / offspring / entity_data — pass.** Fluid use
  is SOURCE_ONLY + LiquidBlock, consume works, offspring
  (`spawn_egg_offspring`) exists, entity_data application works.
- **B8 entity_data id override — gap (B7-G03 reproduces, minor).**
  Vanilla `SpawnEggItem.getType` reads the typed `entity_data`
  `type()` and spawns THAT type; the Rust derives the kind from the item
  name (`game/creatures.rs` `use_spawn_egg`,
  `trim_end_matches("_spawn_egg")`) and ignores an id override.
- **B9 peaceful gate — gap (B7-G04 reproduces, minor).** Vanilla 26.1
  `SpawnEggItem.spawnMob` FAILs hostile eggs in peaceful
  (`!isAllowedInPeaceful && difficulty == PEACEFUL`); the 26.1 port has
  NO gate — and the comment at `game/creatures.rs:7513-7515` falsely
  claims "26.1 has no eligibility gate: 26.2 added
  `EntityType.canSpawn`". The direction is reversed vs 1.21.7 (there the
  port had a gate vanilla lacked). 37 `notInPeaceful` types in 26.1 (vs
  38 in 26.2; the difference is `sulfur_cube`): blaze, bogged, breeze,
  cave_spider, creaking, creeper, drowned, elder_guardian, enderman,
  endermite, evoker, ghast, giant, guardian, husk, illusioner,
  magma_cube, parched, phantom, piglin_brute, pillager, ravager,
  silverfish, skeleton, slime, spider, stray, vex, vindicator, warden,
  witch, wither, wither_skeleton, zoglin, zombie, zombie_villager,
  zombified_piglin.
- **B10 ENTITY_PLACE position — pass (B7-G05 withdrawn).** Vanilla 26.1
  posts ENTITY_PLACE at the spawn position; the Rust posts at `p`
  (the spawn pos). The 1.21.7 premise fails for 26.1.
- **B11 ambient sound — gap (B7-G06 reproduces, minor).** No
  `Mob.playAmbientSound()` on spawn-egg placement: the jar's
  `EntityType.spawn` calls it after `addFreshEntityWithPassengers`; the
  Rust `ambient_sound` is only called from the dispenser path
  (`game/dispenser.rs:985`).

### 3.5 Tools useOn (axe/hoe/shovel) — 1 pass, 4 gap (B12–B16)

- **B12 axe — pass (B7-G07 withdrawn).** The port hardcodes the 26.1
  tables (`block_transform.rs:231-234`, "26.1 has no `block_transformer`
  data"); STRIPPABLES 23, TILLABLES 5, FLATTENABLES 6 match the jar;
  copper 15 families × 8 stages verified against the prepared registry
  (copper_block `_block`-suffix-drop quirk: exposed/weathered/oxidized
  copper). Axe order: strip (AXE_STRIP) → scrape (`getPrevious`,
  AXE_SCRAPE + level event 3005) → wax-off (`WAX_OFF_BY_BLOCK`,
  AXE_WAX_OFF + 3004). Till: air-above + face≠DOWN; rooted_dirt
  unconditional + hanging_roots drop (`game.rs:7942-7944`).
- **B13 hoe — gap (B7-G13 + B7-G14 reproduce, minor).**
- **B14 shovel — gap (B7-G13 + B7-G14 reproduce, minor).**
- **B15 campfire douse — gap (B7-G14 reproduces, minor; B7-G08
  withdrawn).** The douse itself is implemented directly in
  `game/campfire_douse.rs` (`#minecraft:shovels`, level event 1009,
  wired `game/use_hand.rs:88`); the jar's `ShovelItem.useOn` gates both
  branches on face≠DOWN and the campfire branch is lit campfire →
  `CampfireBlock.dowse` + 1009. So G08 is withdrawn — only the G14
  trigger defect remains.
- **B16 copper-chest neighbour — gap (B7-G15 reproduces, minor).**
  `transform_copper_chest_neighbour` (`game.rs:7984+`) posts
  BLOCK_CHANGE + particle at the neighbour half; vanilla `AxeItem.useOn`
  sends events only at the clicked pos.

**B7-G13 (minor):** the shield blocking-intent check applies to axe AND
hoe/shovel in the Rust (`game.rs:7890-7898` `block_transform`), with no
main-hand check; vanilla `playerHasBlockingItemUseIntent` is axe-only.

**B7-G14 (minor):** `item_used_on_block` is fired for ALL transforms
(`game.rs:7939`) and campfire douse (`campfire_douse.rs:102`); vanilla
fires ITEM_USED_ON_BLOCK only in `AxeItem.useOn` (0 references in
`HoeItem`/`ShovelItem`, javap-verified).

### 3.6 Food / consumable lifecycle — 1 pass, 1 gap (B17, B18)

- **B17 lifecycle — pass.** Shared code (`finish_consuming` etc.);
  consumable data, durations, sounds, effects verified through the D1
  data checks and the shared-code reads.
- **B18 chorus-fruit teleport — gap (B7-G09 reproduces, MAJOR).** The
  jar's `LivingEntity.randomTeleport` is blocksMotion-grounded; the Rust
  `game/consume.rs:131-139` walks down for
  `#minecraft:entities_can_teleport_to`, a tag ABSENT in 26.1, so the
  loop always runs to min_y and returns false — chorus fruit never
  teleports. Dispatch: `teleport_randomly` → `TeleportConsume.apply` →
  `Teleporter::random_teleport` → `Game::random_teleport`. Note: the
  enderman `random_teleport` (`mobs/enderman.rs:87+`) IS correctly
  blocksMotion-grounded for 26.1 (its comment even says the tag "neither
  exists in 26.1") — only the consume path is unfixed. The same tag is
  also used at `game/commands/world_cmds.rs:2141-2146` — flagged for
  Batch 8 (commands), not this batch's defect.

### 3.7 Shield blocking — pass (B19)

`blocks_attacks` component matches the 26.2 values; `shield.rs:34` reads
the component; use duration 72000 (`item_use.rs:60-65`).

### 3.8 Bow — pass (B20)

Jar `getPowerForTime` = (f²+2f)/3 clamped 1.0; the Rust
(`game/item_use.rs:240`) is identical; 0.1 minimum to fire.

### 3.9 Crossbow — pass (B21)

Jar `getChargeDuration` = floor(modifyCrossbowChargingTime(1.25)×20);
the Rust `charge_ticks` = floor((1.25−0.25×QC).max(0)×20); charging
sounds match including the quick_charge variants.

### 3.10 Trident — pass (B22)

Jar-verified (`evidence/TridentItem.txt`): `releaseUsing` requires ≥10
ticks; riptide>0 requires `isInWaterOrRain` AND NOT `isPassenger`, else
false; `use` fails if `nextDamageWillBreak` or (riptide and not wet).
The Rust matches: `game/trident.rs:397` (use gate),
`trident.rs:501` (releaseUsing wet + `riding.is_some()` gate),
`trident.rs:519` (strength 1.5 + 0.75×(riptide−1)). Note: 1.21.7's
B11-G08 (the port blocking Riptide while riding where vanilla 1.21.7 had
no check) does not apply — 26.1 vanilla HAS the passenger check.

### 3.11 Potions — 2 pass, 2 gap (B23–B26)

- **B23 drink — pass.** `tick_drinking`/`finish_drinking`
  (`game/potions.rs:768/795`), drink sounds, glass-bottle remainder
  (`potions.rs:632/717`), `potion_duration_scale`
  (`potions.rs:816/856/1574`).
- **B24 splash effects + water — gap (B7-G10 reproduces, MAJOR).**
  Vanilla 26.1's `AbstractThrownPotion` has ZERO `PotionTags`
  references — it gates on `Potions.WATER` directly
  (`evidence/AbstractThrownPotion.txt`, lines 74 and 130:
  `PotionContents.is(Potions.WATER)`). The 26.1 `PotionTags` class has
  only TRADEABLE, and the prepared datapack's `tags/potion/` contains
  only `tradeable.json`. But the Rust gates everything on potion tags
  (`game/splash_water.rs:42-50` `potion_in_tag`:
  `minecraft:douses_fire`, `hurts_water_sensitive_entities`,
  `extinguishes_entities`, `rehydrates_axolotls`), which always return
  false — so water splash/lingering bottles do nothing on impact. The
  douse positions, hurt amount (1.0 indirect magic) and extinguish logic
  are otherwise exactly right — only the gate is dead. Axolotl rehydrate
  still works via the separate direct water check in
  `potions.rs:1972` (`stack.potion() == Some("minecraft:water")`).
  Callers verified: `potions.rs:1952/1954` call `potion_hit_block` +
  `affect_entities_around` unconditionally, with the dead gate inside.
- **B25 lingering cloud — pass.** Shared code (cloud at the hit mob's
  feet, else the impact).
- **B26 throw + water bottle on dirt — gap (B7-G11 reproduces, minor).**
  Vanilla 26.1's `PotionItem.useOn` uses `BlockTags.CONVERTABLE_TO_MUD`
  (with an A, `evidence/PotionItem.txt:55`); the Rust uses
  `convertible_to_mud` (with an I) in all four places
  (`game/potions.rs:710`, `game/dispenser.rs:2249`,
  `game/remote.rs:8690`, `horizon-world/src/level/data.rs:459`) — the
  lookup always misses, so a water bottle on dirt never makes mud.
  (The comment at `potions.rs:699` carries the same misspelling.)

### 3.12 Boats — gap (B27; B7-G12 reproduces, minor)

Vanilla 26.1's `BoatItem.use` (`evidence/BoatItem.txt:44-149`) gathers
entities in the player's bbox expanded 5.0 along the view + inflate 1.0,
and returns PASS when the eye is inside any entity's pick-inflated bbox.
The Rust `boat_to_place` (`game/boats.rs:1517-1542`) only checks the
boat's own bbox — a boat is placeable while the player's eye is inside
an entity.

### 3.13 Armour damage reduction — pass (B28)

Jar `CombatRules.getDamageAfterAbsorb`
(`evidence/CombatRules.txt`): divisor 2 + toughness/4, protection
clamp(armor − damage/divisor, armor×0.2, 20), damage ×
(1 − protection/25), `modifyArmorEffectiveness` clamped 0..1. The Rust
(`mobs.rs:3840-3860`) matches exactly, including the hurt-armour-before-
absorb ordering and the Resistance step after.

### 3.14 Mace — pass (B29)

Attributes 5.0 damage / −3.4 speed (D4); `game/mace.rs` present with
FALL_THRESHOLD 1.5 (`mace.rs:20`), Wind Burst handling (`mace.rs:356`,
`385`, `413`).

### 3.15 Ender pearl — pass (B30, B31; B7-G16 FIXED)

- **B30 throw/teleport — pass.** Throw speed 1.5, 0 damage on hit,
  teleport to the pearl's previous position with 5.0 fall damage
  (`game/creatures.rs:5866-5878`).
- **B31 cooldown — pass.** Vanilla 26.1's `ItemStack.use` calls
  `applyAfterUseComponentSideEffects` for instant-use items on Success
  (`evidence/ItemStack.txt:498-543`), which reads USE_COOLDOWN — so
  pearls (1.0s) and wind charges (0.5s) DO get a cooldown in vanilla
  26.1. The 26.1 Rust already applies it: `throw_item`
  (`game/creatures.rs:7370`) calls `apply_use_cooldown` for pearls
  (line ~7430, with a comment citing
  `ItemStack.use's applyAfterUseComponentSideEffects`) and wind charges
  (line ~7381); `apply_use_cooldown` (`game.rs:5187`) reads the
  `use_cooldown` component and starts it. **The 26.2 defect B7-G16 is
  fixed in the 26.1 tree.**

### 3.16 Fishing rod — pass (B32)

`game/fishing.rs` present with `lure_wait` 100–600 ticks, cast/reel,
medium loot tables — shared code, matches the jar's `FishingRodItem`.

### 3.17 Spears — pass (B33)

Full implementation present: `spear.rs` `Kinetic` (charge/damage
conditions), `attack_range` max 4.5 (`spear.rs:52-63`),
`hits_in_reach` (`spear.rs:459`), `game/hand.rs` attack-range gating
(`hand.rs:131-143`), `mobs/spear_use.rs`, `first_person.rs` spear
charging/swing. Data verified in D4: `items.json`
`minecraft:iron_spear` carries `minecraft:attack_range` (max_reach 4.5,
min_reach 2.0, hitbox_margin 0.125) and `minecraft:kinetic_weapon`
with the jar's exact values (damage_multiplier 0.95, delay_ticks 12,
dismount/knockback/damage conditions).

### 3.18 Nautilus armour — pass (B34)

`game/mount_menu.rs` handles nautilus body slots
(`mount_menu.rs:39/48/83-84`, citing
`EntityTypeTags.CAN_WEAR_NAUTILUS_ARMOR`); the tag exists in 26.1's
data. Data verified in D7 (5 pieces, no durability, no enchantable,
stacksTo(1)).

## 4. Defect list

12 live defects: **2 major, 10 minor.** All verified against 26.1's OWN
jar + the 26.1 tree. No UNVERIFIED carry-overs.

| ID | Sev | Check(s) | Title |
|----|-----|----------|-------|
| B7-G01 | minor | B1 | `ComponentMap::set` errors on a dangling removal marker; vanilla returns null |
| B7-G03 | minor | B8 | Spawn-egg `entity_data` id type override ignored |
| B7-G04 | minor | B9 | No peaceful gate for hostile spawn eggs (vanilla 26.1 HAS the gate) |
| B7-G06 | minor | B11 | No `Mob.playAmbientSound()` on spawn-egg placement |
| B7-G09 | **major** | B18 | Chorus fruit never teleports (tag `#entities_can_teleport_to` absent in 26.1) |
| B7-G10 | **major** | B24 | Water splash/lingering bottles do nothing on impact (potion-tag gate dead) |
| B7-G11 | minor | B26 | Water bottle on dirt never makes mud (`convertible` vs `convertable`) |
| B7-G12 | minor | B27 | Boat placeable while the eye is inside an entity |
| B7-G13 | minor | B13, B14 | Shield blocking-intent check (axe-only in vanilla) also blocks hoe/shovel, ignores hand |
| B7-G14 | minor | B13, B14, B15 | ITEM_USED_ON_BLOCK fired for hoe/shovel/campfire; vanilla fires it only for the axe |
| B7-G15 | minor | B16 | Copper-chest scrape/wax-off posts BLOCK_CHANGE + particle at the neighbour half |
| B7-G17 | minor | B6 | Op's spawn-egg `entity_data` applied to spawners (26.3 behaviour; 26.1 has no `Spawner.setEntityData`) |

1.21.7 re-check verdicts (G01–G12, one verdict each, all against 26.1's
jar + tree):
- G01 REPRODUCES (26.2 fixed it; 26.1 did not)
- G02 WITHDRAWN (26.1 `BucketItem` HAS the sneak check)
- G03 REPRODUCES
- G04 REPRODUCES (reversed direction vs 1.21.7: vanilla 26.1 has the
  gate, the port lacks it)
- G05 WITHDRAWN (26.1 posts ENTITY_PLACE at the spawn position)
- G06 REPRODUCES
- G07 WITHDRAWN (hardcoded tables correct for 26.1)
- G08 WITHDRAWN (campfire douse implemented directly)
- G09 REPRODUCES
- G10 REPRODUCES
- G11 REPRODUCES
- G12 REPRODUCES

26.2 re-checks: G13 REPRODUCES, G14 REPRODUCES, G15 REPRODUCES,
G16 FIXED (the 26.1 tree applies `use_cooldown` on throw — §3.15).

## 5. Oracle triage — 30 oracle defects (2 major, 28 minor)

Authority: `docs/sweep/26.1/batch-0-oracles.json` (Batch 0's measured
javac table: 199/391 pass on client, 192 fail). Every failing oracle
below was triaged from that table AND javap-verified against the named
26.1 jars myself — Batch 0's attribution was not copied unverified.
The full failing list (192) was swept, not just driver oracles; the
30 below are the item-family ones. Cross-batch duplicates are expected;
the coordinator dedups at merge.

Root-cause groups (one 26.2-form API gap each, shared by several
oracles):

**A. `EntityTypes` registration move (G4 group) — 26.1 still registers
in `EntityType`.** `net.minecraft.world.entity.EntityTypes` does not
exist in the 26.1 client jar (unzip-verified, 0 matches).
- **B7-O01 VanillaDispenserOracle — major.** missing-package, 15 errors.
  First error `212: package EntityTypes does not exist`
  (`EntityTypes.PLAYER.getDimensions()`). Blocks 2 drivers
  (use_item_on, mace). Also the G4 group representative (Batch 0: 31
  files in the group).
- B7-O08 VanillaBoatAuthorityOracle — minor. missing-symbol, 4 errors.
  First error `14: cannot find symbol` (import
  `net.minecraft.world.entity.EntityTypes`). No Batch 7 driver.
- B7-O09 VanillaItemModelOracle — minor. missing-symbol, 10 errors.
  First error `37: cannot find symbol` (import
  `net.minecraft.world.entity.EntityTypes`). No Batch 7 driver.

**B. `Level.getNextEntityId` — no such method in 26.1** (javap on
`net.minecraft.world.level.Level`: no match). Five oracles override it.
- B7-O03 VanillaArrowOracle — minor. no-override, 1 error (`78`).
  Blocks trident.
- B7-O04 VanillaBoatRiderOracle — minor. no-override, 4 errors (`38`).
  No Batch 7 driver.
- B7-O05 VanillaClientArrowOracle — minor. no-override, 1 error (`38`).
  No Batch 7 driver.
- B7-O06 VanillaExplosionItemsOracle — minor. no-override, 1 error
  (`29`). Blocks explosion_items (jointly with O10).
- B7-O07 VanillaLavaCauldronItemOracle — minor. no-override, 1 error
  (`28`). Blocks lava_cauldron_item.

**C. `VanillaBlockUpdateOracle` — the clock API.** `net.minecraft.world.
clock.ClockInstance` does not exist in the 26.1 client jar
(unzip-verified, 0 matches).
- **B7-O02 VanillaBlockUpdateOracle — major.** missing-symbol, 32
  errors. First error `450: cannot find symbol` (ClockInstance).
  Cascade root for O29/O30; also blocks helmet_damage directly.
  Cross-batch: block-domain (Batch 3); recorded here because it blocks
  two Batch 7 drivers.
- B7-O29 VanillaEnchantDurabilityOracle — minor. cascade (depends on
  broken VanillaBlockUpdateOracle). Blocks enchant_durability.
  (Runnable in 26.2.)
- B7-O30 VanillaHelmetDamageOracle — minor. cascade (depends on broken
  VanillaBlockUpdateOracle). Blocks helmet_damage. (Runnable in 26.2.)

**D. 26.1 passes the broken `Item`, 26.2 passes the `ItemStack` copy.**
`ItemStack.hurtAndBreak(int, ServerLevel, ServerPlayer,
Consumer<Item>)` and `LivingEntity.onEquippedItemBroken(Item,
EquipmentSlot)` in 26.1 (javap-verified).
- B7-O15 VanillaItemDurabilityOracle — minor. incompatible-types,
  1 error (`20: incompatible types: Item cannot be converted to
  ItemStack`). Blocks item_durability.
- B7-O16 VanillaComponentDurabilityOracle — minor. incompatible-types,
  2 errors (`23`, same message). Blocks component_durability.
- B7-O17 VanillaBlockingItemDamageOracle — minor. no-override, 2 errors
  (`21`: overrides `onEquippedItemBroken(ItemStack, EquipmentSlot)`).
  Blocks blocking_item_damage.

**E. `ServerChunkCache` constructor — 26.1 takes 11 args** (…,
ChunkStatusUpdateListener, Supplier<SavedDataStorage>); the oracles
call a 10-arg form (javap-verified on server-game.jar).
- B7-O19 VanillaItemUseStartOracle — minor. constructor-mismatch,
  1 error (`16`). Blocks item_use_start.
- B7-O20 VanillaInventoryUseStartOracle — minor. constructor-mismatch,
  1 error (`19`). Blocks inventory_use_start.

**F. `Gui` constructor — 26.1 takes 1 arg** (`Gui(Minecraft)`); the
oracles call `super(null, null, null)` (javap-verified on client.jar).
- B7-O21 VanillaHeldDropOracle — minor. constructor-mismatch, 6 errors
  (`38`). Blocks held_drop.
- B7-O22 VanillaInteractionInputOracle — minor. constructor-mismatch,
  5 errors (`39`). Blocks interaction_input.

**G. `ItemStackTemplate.fromStack` — absent in 26.1** (only
`fromNonEmptyStack(ItemStack)`, 1-arg, exists; javap-verified).
- B7-O23 VanillaItemTemplateOracle — minor. missing-symbol, 2 errors
  (`25`). Blocks item_template.
- B7-O24 VanillaUseRemainderOracle — minor. missing-symbol, 2 errors
  (`26`). Blocks use_remainder.

**H. `VanillaRegistries.createWorldLookup` — absent in 26.1**
(`createLookup()` instead; javap-verified). `Enchantments.UNBREAKING`
does exist (ResourceKey form), so this is the single error in each.
- B7-O25 VanillaItemUseCatalogOracle — minor. missing-symbol, 2 errors
  (`12`). Blocks item_use_catalog.
- B7-O26 VanillaInventoryPickupOracle — minor. missing-symbol, 1 error
  (`33`). Blocks inventory_pickup.
- B7-O27 VanillaPlayerInventoryOracle — minor. missing-symbol, 1 error
  (`28`). Blocks player_inventory.

**I. Singletons.**
- B7-O10 VanillaVehicleContainerOracle — minor. missing-symbol,
  3 errors. First error `20: cannot find symbol` (import
  `net.minecraft.world.entity.EntitySpawnRequest` — class absent from
  the 26.1 client jar, unzip-verified). Blocks explosion_items
  (jointly with O06). Cross-batch: vehicle/container (entities).
- B7-O11 KnockbackClient — minor. missing-symbol, 6 errors. First
  error `55: cannot find symbol` (import
  `net.minecraft.network.protocol.game.ServerboundPunchPacket` — class
  absent from the 26.1 client jar, unzip-verified). Blocks
  item_use_order.
- B7-O12 VanillaConsumableOracle — minor. missing-symbol, 1 error
  (`76`). The missing symbol is
  `InteractionResult.Success.shouldSwing()` — 26.1's `Success` has
  `swingSource()` instead (javap-verified); the `Consumable`
  constructor and `canConsume`/`startConsuming`/`onConsume` all match.
  Blocks consumable.
- B7-O13 VanillaTeleportConsumeOracle — minor. no-override, 5 errors.
  First error `27`: overrides
  `randomTeleport(double, double, double, boolean, TagKey<Block>)` —
  26.1's `LivingEntity.randomTeleport` takes only 4 args (no tag
  parameter; javap-verified). Same 26.2-form tag parameter behind
  B7-G09. Blocks teleport_consume. (Runnable in 26.2.)
- B7-O14 VanillaItemBlockingOracle — minor. no-override, 2 errors.
  First error `29`: overrides `blockUsingItem(ServerLevel,
  LivingEntity, DamageSource, float, boolean)` — 26.1's
  `LivingEntity.blockUsingItem` takes only (ServerLevel, LivingEntity)
  (javap-verified). Blocks item_blocking.
- B7-O18 VanillaExtraItemsOracle — minor. missing-symbol, 2 errors.
  First error `15`: overrides `drop(ItemStack, boolean, Prediction)` —
  26.1's `Player.drop` takes only (ItemStack, boolean)
  (javap-verified); `handleExtraItemsCreatedOnUse` DOES exist on
  26.1's ServerPlayer. Blocks extra_items.
- B7-O28 VanillaHeldItemStackOracle — minor. missing-symbol, 4 errors.
  First error `30`: overrides `LivingEntity.SwingDescription
  getCurrentSwing()` — `SwingDescription` does not exist anywhere in
  the 26.1 client jar (unzip-verified, 0 matches). Blocks
  held_item_stack.

**Passing in-scope oracles (36, from the Batch 0 table):** the 33
oracles of the runnable drivers — VanillaFoodOracle,
VanillaFoodPropertiesOracle, VanillaFoodStorageOracle,
VanillaPlayerFoodOracle, VanillaConsumableEmissionOracle,
VanillaConsumeEffectOracle, VanillaStoredConsumeOracle,
VanillaItemUseOracle, VanillaItemUseLifecycleOracle,
VanillaItemAfterUseOracle, VanillaUseFlagReactionOracle,
VanillaInventoryUseOracle, VanillaStackMiningOracle,
VanillaCreativeMiningOracle, VanillaBlockingRulesOracle,
VanillaPlayerEquipmentOracle, VanillaDamageReductionOracle,
VanillaCombatRulesOracle, VanillaComponentPatchOracle,
VanillaEnchantmentComponentsOracle, VanillaStoredEnchantmentPatchOracle,
VanillaTradeComponentsOracle, VanillaItemStackStorageOracle,
VanillaItemWireOracle, VanillaItemCooldownsOracle,
VanillaStackFoilOracle, VanillaStackModelFlagsOracle,
VanillaPickBlockDataOracle, VanillaBundleWeightOracle,
VanillaMapSavedDataOracle, VanillaHotbarOracle, VanillaInventoryTickOracle,
VanillaEnchantingOracle — plus three passing item-adjacent oracles with
no Batch 7 driver: VanillaItemParticlesOracle,
VanillaInteractionShapeOracle, VanillaStatusEffectOracle. Notable:
`VanillaItemCooldownsOracle` PASSES in 26.1 (it resolves the 26.2
coordinator's unverified oracle name); `VanillaComponentPatchOracle`
passes even though the Rust has B7-G01 (the oracle measures vanilla,
not the port).

**Out of scope (assessed, not triaged here):** ComponentWireCapture
(advancements predicates), VanillaAllayInteractionStateOracle (mob
interaction), VanillaBlockCollisionScanOracle,
VanillaBlockEntityAnimOracle, VanillaBlockEntityRenderOracle,
VanillaEntityComponentsOracle, VanillaFallingBlockOracle — block/entity
domain, other batches' scope.

## 6. Driver inventory — 74 drivers, none run

Driver inventory is DONE (saved to
`evidence/driver-inventory-26.1.json`, script
`evidence/driver_inventory_26_1.py`); **no driver was run** — this batch
is static checks only per the brief. Statuses below are MEASURED oracle
status from `docs/sweep/26.1/batch-0-oracles.json` (Batch 0's javac
measurement against the 26.1 jars; "runnable" = every oracle the driver
needs passes on the client classpath). Oracle sets were taken from the
26.2 driver map and cross-checked against the 26.1 driver files
themselves: the only mismatch is `interaction_input`, whose 26.1 file
also references the `ParityData` helper (not an oracle).

**26.1: 32 runnable, 25 blocked, 17 no-oracle** (26.2 was 38 / 19 / 17).
Six drivers flipped runnable→blocked vs 26.2: teleport_consume
(VanillaTeleportConsumeOracle, no-override), enchant_durability
(cascade on VanillaBlockUpdateOracle), trident (VanillaArrowOracle,
no-override), explosion_items (VanillaExplosionItemsOracle, no-override
+ VanillaVehicleContainerOracle, missing-symbol), lava_cauldron_item
(VanillaLavaCauldronItemOracle, no-override), helmet_damage
(VanillaBlockUpdateOracle, missing-symbol + VanillaHelmetDamageOracle,
cascade). No driver flipped blocked→runnable.

- **32 runnable** (oracles pass; NOT run): food, food_properties,
  food_storage, player_food, consumable_emission, consume_effect,
  stored_consume, item_use, item_use_lifecycle, item_after_use,
  use_flag_reaction, inventory_use, stack_mining, creative_mining,
  blocking_rules, player_equipment, damage_reduction, combat_rules,
  component_patch, enchantment_components, stored_enchantment_patch,
  trade_components, item_stack_storage, item_wire, item_cooldowns,
  stack_foil, pick_block_data, bundle_weight, map_saved_data, hotbar,
  inventory_tick, enchanting.
- **25 blocked** by 26 distinct failing oracles (measured error class;
  all fail on all three classpaths): consumable
  (VanillaConsumableOracle, missing-symbol, 1), teleport_consume
  (VanillaTeleportConsumeOracle, no-override, 5), item_use_catalog
  (VanillaItemUseCatalogOracle, missing-symbol, 2), item_use_order
  (KnockbackClient, missing-symbol, 6), item_use_start
  (VanillaItemUseStartOracle, constructor-mismatch, 1 —
  ServerChunkCache), use_item_on + mace (VanillaDispenserOracle,
  missing-package, 15 — EntityTypes), use_remainder
  (VanillaUseRemainderOracle, missing-symbol, 2 — fromStack),
  inventory_use_start (VanillaInventoryUseStartOracle,
  constructor-mismatch, 1 — ServerChunkCache), interaction_input
  (VanillaInteractionInputOracle, constructor-mismatch, 5 — Gui),
  item_durability (VanillaItemDurabilityOracle, incompatible-types,
  1 — Item → ItemStack), component_durability
  (VanillaComponentDurabilityOracle, incompatible-types, 2),
  enchant_durability (VanillaEnchantDurabilityOracle, cascade),
  blocking_item_damage (VanillaBlockingItemDamageOracle, no-override,
  2), item_blocking (VanillaItemBlockingOracle, no-override, 2),
  trident (VanillaArrowOracle, no-override, 1 — getNextEntityId),
  helmet_damage (VanillaBlockUpdateOracle, missing-symbol, 32 —
  ClockInstance + VanillaHelmetDamageOracle, cascade), item_template
  (VanillaItemTemplateOracle, missing-symbol, 2 — fromStack),
  extra_items (VanillaExtraItemsOracle, missing-symbol, 2 — drop),
  explosion_items (VanillaExplosionItemsOracle, no-override, 1 —
  getNextEntityId + VanillaVehicleContainerOracle, missing-symbol, 3
  — EntitySpawnRequest), lava_cauldron_item
  (VanillaLavaCauldronItemOracle, no-override, 1 — getNextEntityId),
  held_item_stack (VanillaHeldItemStackOracle, missing-symbol, 4 —
  SwingDescription), held_drop (VanillaHeldDropOracle,
  constructor-mismatch, 6 — Gui), inventory_pickup
  (VanillaInventoryPickupOracle, missing-symbol, 1 —
  createWorldLookup), player_inventory (VanillaPlayerInventoryOracle,
  missing-symbol, 1 — createWorldLookup). No oracle repairs in this
  batch.
- **17 no-oracle** (mostly live-game e2e drivers needing a built game +
  fixture servers; NOT run): interaction_data, inline_fishing_tool,
  trident_save, pumpkin_equippable_use, builtin_component_delivery,
  effective_component_presence, effective_item_components,
  numeric_damage_component, item_contents, item_numeric_uuid,
  item_health_load, item_entity_max_stack, spawn_egg_fluid,
  boat_item_hit, map_decoration_patch, map_decoration_namespace,
  enchantment_map_validation. Same 17 as 26.2 (and 1.21.7).

**Oracle defects found: 30** (§5). **Oracle repairs done: none** (none
in this batch).

## 7. Method notes

- Every vanilla claim is cited to a specific jar method + bytecode, read
  with `javap -c -p` (JDK 25) on the named 26.1 jars (client.jar for
  client classes, server-game.jar for server classes). No behaviour was
  assumed from the 26.3 base or from memory — and no 1.21.7 defect was
  assumed to persist: each was re-checked against the 26.1 tree's actual
  Rust, and four were withdrawn (two because the 26.1 tree fixed them,
  two because 26.1's vanilla differs from 1.21.7's). No 26.2 verdict was
  copied either: G01 and G09/G10/G11 reproduce in 26.1 although 26.2
  fixed/withdrew them, and G16 is fixed in 26.1 although it was live in
  26.2.
- 26.1 class facts found during the batch: `AbstractThrownPotion` is in
  `net.minecraft.world.entity.projectile.throwableitemprojectile` (same
  package move as 26.2); registration is still in `EntityType` /
  `BlockEntityType` (pre-`EntityTypes` move); bed IS a BE type; shulkers
  and banners are uncollapsed; `PotionTags` has only TRADEABLE;
  `LivingEntity.randomTeleport` takes 4 args (no tag parameter);
  `ItemStack.hurtAndBreak` takes `Consumer<Item>`; `Level` has no
  `getNextEntityId`; `Gui` takes 1 constructor arg; `ServerChunkCache`
  takes 11; `VanillaRegistries` has `createLookup`, not
  `createWorldLookup`; `ItemStackTemplate` has only 1-arg
  `fromNonEmptyStack`.
- Rust item data is data-driven from `items.json` at runtime (`ItemInfo`
  in `horizon-items/src/items.rs`); data parity therefore hinges on
  prepared `items.json` fidelity (verified in §2) and loader/behaviour-
  code interpretation (verified per family in §3). Two exceptions are
  hardcoded tables rather than data-driven: the tool transforms
  (`block_transform.rs`, correctly — 26.1 has no registry) and the
  spear kinetic-weapon table (`spear.rs`, values verified exact against
  the jar).
- The 26.2 run's out-of-scope observations were NOT re-verified for
  26.1 and are not carried forward as claims here. One new out-of-scope
  observation from this batch: the `#entities_can_teleport_to` tag
  (absent in 26.1) is also consulted at
  `game/commands/world_cmds.rs:2141-2146` — for Batch 8 (commands).
- Analysis artefacts (scripts, jar dumps, check outputs, driver map,
  oracle failing-lines) live in `~/workspace/mcir-work/sweep-26.1-
  batches/b07/evidence/`.

## 8. What this batch defers

- Running the 32 runnable drivers (the food driver in particular was
  called out in the 1.21.7 brief) and the 17 no-oracle drivers
  (live-game e2e; need a built game + fixture servers).
- Repairing any of the 30 blocking oracles (most are 1–6 javac errors;
  VanillaDispenserOracle blocks two drivers; VanillaBlockUpdateOracle
  is the cascade root for two more) and end-to-end driver runs to
  confirm each repair yields a measurement.
- Committing these deliverables (docs/sweep/26.1/batch-7-items.md,
  pins/26.1/sweep/batch-07.json) — the coordinator's job.
- Fixes for the 12 defects are a separate decision on separate branches.

Nothing in this batch touched production code (`crates/`), per the
checks-only brief.
