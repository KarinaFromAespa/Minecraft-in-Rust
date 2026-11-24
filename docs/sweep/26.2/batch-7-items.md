# Batch 7 — Items (26.2 parity sweep)

**Status: checks + triage only, no fixes.** Static analysis only (javap +
Rust source + vanilla/prepared data), per the brief: no cargo builds, no
driver runs, no edits under `versions/26.2/`, nothing committed by this
agent. Deliverables live under
`~/workspace/mcir-work/sweep-26.2-scratch/batch-07/`; the coordinator commits
them.

- **Branch / tip checked:** `parity-sweep-26.2` @ `732c800f` (synced to main,
  2026-11-24)
- **Spec:** the named 26.2 server-game jar
  (`~/workspace/mcir-work/26.1-adapters/versions/26.2/jars/server-game.jar`,
  Mojang-mapped) and prepared 26.2 data
  (`versions/26.2/.local/horizon-client/26.2/`). Vanilla 26.2's own jar/data
  is the spec — never the 26.3 base's form (owner's vanilla-check rule).
  `leakage-audit/jars/26.2-server.jar` is a Mojang bundler jar with no game
  classes and was never javap'd; the client game jar
  (`leakage-audit/jars/26.2-client.jar`) supplied datapack/tag listings.
- **Checks: 42 total — 35 pass, 7 gap. Defects: 7 (0 major, 7 minor).**
  Nine 1.21.7 defects were re-checked and withdrawn for 26.2 (see §4); three
  persist (B7-G03, B7-G06, B7-G12); four are new in 26.2 (B7-G13–G16).
- **Oracle repairs: none** (none in this batch — record only).
  **New oracle defects: none** (see §5).

**Coordinator note (2026-11-24):** this batch used the measured Batch 0
table directly — all 18 blocking-oracle claims in §5 verify exactly
against `docs/sweep/26.2/batch-0-oracles.json` (0 mismatches). Tally
arithmetic verified: D1–D8 (8 pass) + B1–B34 (27 pass, 7 gap) = 42
total, 35 pass, 7 gap; the hoe and shovel checks each carry two defects
(B7-G13 + B7-G14). No corrections needed. (One name unverified: no
`VanillaItemCooldownOracle` entry exists in the Batch 0 table, so the
`item_cooldowns` driver's oracle name could not be confirmed from the
table — the driver map is in `b7-26.2-analysis/driver-map-26.2.json`.)
- **Prepare landed during this run:** horizon-prepare for 26.2 finished
  09:11 NPT Nov 24 (2048.7 s, EXIT 0); `block-behaviours.json`,
  `gameplay-facts.json`, `entity-types.json` and `attributes.json` became
  available partway through. This batch's data needs (items.json,
  block-tags.json, jar data) were already present; the late files were used
  only as cross-checks (entity-types.json category data for §3.4).
  sounds.json / lang files remain absent (no asset index 32.json on the VM)
  — expected, not a port defect.

## 1. Item→class map — 1,537 items, 70 classes, 0 unmapped

`block-behaviours.json` holds only BlockItem entries; the other 483 items
had no behaviour-class entry. Resolved the same way as 1.21.7: `javap -v`
BootstrapMethods (228 entries) resolves each `invokedynamic` to a factory
lambda or direct constructor/method ref, and a `javap -c` `<clinit>` walk
assigns one class per item segment (script
`b7-26.2-analysis/parse_items_class_26b.py`, adapted from the 1.21.7 parser).

**Result: all 1,537 items mapped to 70 distinct classes, zero unmapped,
zero mismatches** against `block-behaviours.json`'s class field. The 483
non-block items break down as: 236 plain `Item` (data-driven via
components), 88 SpawnEggItem, 28 DoubleHighBlockItem, 20 BoatItem, 20
StandingAndWallBlockItem, 19 SmithingTemplateItem, 17 BundleItem, 16 each
BannerItem / BedItem / DyeItem, 12 each SignItem / HangingSignItem, 7 each
GameMasterBlockItem / ShovelItem / AxeItem / HoeItem / MobBucketItem, 6
MinecartItem, 3 each BucketItem / EggItem, 2 each PlaceOnWaterBlockItem /
FoodOnAStickItem / ItemFrameItem, 43 singletons (BowItem, CrossbowItem,
TridentItem, FishingRodItem, ShieldItem, MaceItem, PotionItem,
SplashPotionItem, LingeringPotionItem, EnderpearlItem, …). BlockItem: 938.

Key 26.2 facts established from the jar:

- There is **no** `SwordItem`, `PickaxeItem`, `ArmorItem`, or `ElytraItem`
  class — same as 1.21.7. Swords/pickaxes/armour/elytra are plain `Item`
  with tool/weapon/equippable/attribute components. `AxeItem`/`HoeItem`/
  `ShovelItem` exist (strip/till/path `useOn`).
- `milk_bucket` is plain `Item` (consumable + use_remainder). `AIR` is
  `AirItem`. `ender_pearl` is `EnderpearlItem` (one word).
- **Spears are a new 26.2 family: 7 items** (wood/stone/copper/iron/diamond/
  gold/netherite), plain `Item` with `kinetic_weapon`, `piercing_weapon`,
  `attack_range` and `minimum_attack_charge` components (see §2, §3.17).
- **Nautilus armour is a new 26.2 family: 5 items** (iron/golden/diamond/
  netherite/copper), equippable slot `body`, allowed entities
  `#minecraft:can_wear_nautilus_armor` (see §2, §3.18). Copper and
  netherite horse armour are also new (6 horse armours total).
- `items.json` component frequencies (all 1,537): food 44, consumable 43,
  tool 38, weapon 44, equippable 84, max_damage 84, enchantable 77,
  repairable 75, bundle_contents 17, dye 16, banner_patterns 17,
  potion_contents 4, use_cooldown 3 (chorus_fruit 1.0s, ender_pearl 1.0s,
  wind_charge 0.5s), use_remainder 7, attack_range 7, kinetic_weapon 7,
  piercing_weapon 7, minimum_attack_charge 7 (the 7 = spears),
  damage_resistant 17, potion_duration_scale 2, **block_transformer 0**.
- `minecraft:block_transformer`: **0 files in the client jar datapack, no
  `BlockTransformer` class in the server jar** — 26.2 does NOT have the
  registry (it is a 26.3-only mechanism). `#douses_campfires`: 0 files.
  `#entities_can_teleport_to`: 0 files. `data/minecraft/tags/potion/`
  contains ONLY `tradeable.json`; `PotionTags` has ONLY the `TRADEABLE`
  field. The mud tag is `convertable_to_mud` (with an A, Mojang spelling).

26.2-specific parser traps (do not re-debug): `Items.<clinit>` has only
1,177 static fields; 360 items are registered by **30 bulk collection
calls** that must be stripped before segmentation (12
`ColorCollection.registerBlockItems`, 15
`WeatheringCopperCollection.registerItems`, 3
`ColorCollection.registerItems` — `HARNESS`, `DYED_BUNDLE`, `DYE`).
Collection regions end in a putstatic to a Collection field, not an Item
field. Registry ids come from the **field name first**, validated against
items.json — BlockItemIds/Blocks names are block-oriented and wrong for
custom-name items (`WHEAT_CROP`/`WHEAT` → `wheat_seeds`, `TRIPWIRE` →
`string`). Only 3 field names mismatch the registry (same as 1.21.7):
`DRY_SHORT_GRASS`→`short_dry_grass`, `DRY_TALL_GRASS`→`tall_dry_grass`,
`CUT_STANDSTONE_SLAB` (Mojang typo)→`cut_sandstone_slab`. Lambda resolution
must follow nested invokedynamics and `registerBlock`/`registerItem`
delegation; factory method-refs to `Items.registerBlock` resolve to
BlockItem (not the `Item` return type); `registerSpawnEgg` segments have no
invokedynamic → SpawnEggItem. Weathered copper blocks drop the `_block`
suffix: `exposed_copper`, `weathered_copper`, `oxidized_copper` (+ waxed
forms) belong to COPPER_BLOCK.

## 2. Data checks — 8/8 pass

| # | Check | Result |
|---|-------|--------|
| D1 | Food: nutrition, saturation (= nutrition × modifier × 2, float32), can_always_eat — 44/44 | pass |
| D2 | Max damage — 84 verified, 0 mismatches | pass |
| D3 | Max stack — 1,537/1,537 | pass |
| D4 | Attack damage/speed — 35/35 material tools + trident (8.0, −2.9) + mace (5.0, −3.4) via `createAttributes`, + 7 spears | pass |
| D5 | Tool mining speed — 28/28 (pickaxe/axe/shovel/hoe) + swords vs `ToolMaterial.applySwordProperties` (cobweb 15.0, `#sword_instantly_mines` Float.MAX overrideSpeed, `#sword_efficient` 1.5, damagePerBlock 2, canDestroyInCreative false) | pass |
| D6 | Tool-rule tags — 16/16 exist in jar and contents match prepared `block-tags.json` exactly (after expanding `#` references) | pass |
| D7 | Armour attributes — 41/41 (29 humanoid + 6 horse + 1 wolf + 5 nautilus) | pass |
| D8 | Item→class mapping — 1,537/1,537, 70 classes, 0 mismatches (see §1) | pass |

Jar-verified constants recorded for later batches:

- **ToolMaterial** [durability, speed, attackDamageBonus, enchantmentValue]:
  WOOD [59, 2.0, 0.0, 15], STONE [131, 4.0, 1.0, 5], **COPPER [190, 5.0, 1.0,
  13] (new in 26.2)**, IRON [250, 6.0, 2.0, 14], DIAMOND [1561, 8.0, 3.0,
  10], GOLD [32, 12.0, 0.0, 22], NETHERITE [2031, 9.0, 4.0, 15].
- **ArmorMaterials** [durability, defense boots/legs/chest/helmet/body,
  enchantment, toughness, knockback-resistance]: LEATHER [5, 1/2/3/1/3, 15,
  0, 0], **COPPER [11, 1/3/4/2/4, 8, 0, 0] (new)**, CHAINMAIL [15, 1/4/5/2/4,
  12], IRON [15, 2/5/6/2/5, 9], GOLD [7, 1/3/5/2/7, 25], DIAMOND [33,
  3/6/8/3/11, 10, 2.0], TURTLE_SCUTE [25, 2/5/6/2/5, 9], NETHERITE [37,
  3/6/8/3/**19**, 15, 3.0, 0.1] (body defense 19 in 26.2 vs 11 in 1.21.7),
  ARMADILLO_SCUTE [4, 3/6/8/3/11, 10]. Slot bases: helmet 11, chestplate 16,
  leggings 15, boots 13, body 16.
- `Item$Properties` bytecode verified: `humanoidArmor` sets durability +
  attributes + enchantable; `horseArmor`/`wolfArmor`/`nautilusArmor` set NO
  enchantable (horse/nautilus also no durability; all three end
  `stacksTo(1)`).
- **Spear formulas** (javap-verified from `Item$Properties.spear(material,
  9 floats)`, all float32): attack damage = material bonus; attack speed =
  (1.0/arg0) − 4.0; KineticWeapon contact cooldown 10, delay_ticks =
  (int)(arg2×20), damage_multiplier = arg1, dismount
  `ofAttackerSpeed((int)(arg3×20), arg4)`, knockback
  `ofAttackerSpeed((int)(arg5×20), arg6)`, damage
  `ofRelativeSpeed((int)(arg7×20), arg8)`, forward_movement 0.38;
  AttackRange(2.0, 4.5, 2.0, 6.5, 0.125, 0.5); minimum_attack_charge 1.0;
  swing STAB duration (int)(arg0×20). All 35 tick values (7 materials × 5)
  recomputed under float32 arithmetic match items.json exactly — including
  the rounding edge cases (copper delay 0.65×20 → 13, stone/golden 0.7×20 →
  14). Spear args per material, (attack, multiplier, delay, dismount t/s,
  knockback t/s, damage t/s): wood (0.65, 0.7, 0.75, 5.0/14.0, 10.0/5.1,
  15.0/4.6), stone (0.75, 0.82, 0.7, 4.5/13.0, 9.0/5.1, 13.75/4.6), copper
  (0.85, 0.82, 0.65, 4.0/12.0, 8.25/5.1, 12.5/4.6), iron (0.95, 0.95, 0.6,
  2.5/11.0, 6.75/5.1, 11.25/4.6), gold (0.95, 0.7, 0.7, 3.5/13.0, 8.5/5.1,
  13.75/4.6), diamond (1.05, 1.075, 0.5, 3.0/10.0, 6.5/5.1, 10.0/4.6),
  netherite (1.15, 1.2, 0.4, 2.5/9.0, 5.5/5.1, 8.75/4.6). Wooden spear
  sounds are the `spear_wood` variants; all others use `spear`.
- Tool-rule tags (16): incorrect_for_{wooden,stone,copper,iron,gold,
  diamond,netherite}_tool, mineable/{axe,hoe,pickaxe,shovel},
  shears_{extreme,major,minor}_breaking_speed, sword_efficient,
  sword_instantly_mines. Jar tag keys keep subdirectories
  (`minecraft:mineable/pickaxe`). Note: prepared `block-tags.json` values
  are raw JSON strings — `json.loads` before use.

## 3. Behaviour checks — 27 pass, 7 gap

### 3.1 component_patch — pass (B7-G01 withdrawn)

26.2's `PatchedDataComponentMap.set` is javap-verified identical to 1.21.7's:
returns `previous.orElse(prototype)` — null with no prototype, never an
error. The 26.2 tree's `ComponentMap::set`
(`horizon-world/src/component_patch.rs:109-121`) now matches:
`Some(Some(v)) => Ok(Some(v)), _ => Ok(base)` — a dangling removal marker
falls through to `Ok(base)`, no `Err(MissingPrototype)`. The 1.21.7 defect
is fixed in the 26.2 port.

### 3.2 Buckets — pass (B7-G02 withdrawn)

26.2's `BucketItem.emptyContents` **has** the sneak check
(`LivingEntity.isShiftKeyDown`, javap-verified): sneaking + non-air target +
non-null hit result retries beside the clicked block (relative to the face,
null hit result). `MobBucketItem`/`SolidBucketItem` have 0 sneak references,
as in 1.21.7. The port's `bucket_empty` sneak-redirect is therefore correct
for 26.2 — the 1.21.7 finding was a version difference, not a port bug. The
rest of the bucket path (pickup source-only, fill sounds, `createFilledResult`,
evaporation, mob-bucket spawn, cauldron precedence) is the shared code
verified in 1.21.7 and unchanged in shape for 26.2.

### 3.3 Bundles — pass

26.2's `BundleItem` has the same API as 1.21.7's
(`overrideStackedOnOther`, `overrideOtherStackedOnMe`, `use` → dropContent,
`toggleSelectedItem`, `getSelectedItem`, bar methods, `onDestroyed` spill).
The port implements the full behaviour in `horizon-items/src/bundle.rs`
(`Contents::try_insert`, `remove_one`, `room_for`, `weight_fraction`,
`bar`, `store`, `toggle_selected`, `destroyed` spill; weight 64/max_stack,
capacity 1.0) plus the selection wheel (`game/bundles.rs`), drop-on-use
(`game/block_breaking.rs:62` `drop_bundle_content`) and bar rendering
(`game/item_draw.rs` `bundle_fullness`). All 17 bundles (bundle + 16 dyed)
are `BundleItem` class (§1).

### 3.4 Spawn eggs — 2 minor gaps (B7-G03, B7-G06 persist; B7-G04, B7-G05 withdrawn)

Verified matching: useOn spawner (setEntityId + BLOCK_CHANGE + shrink; the
egg's entity data is ignored for the spawner in 26.2 — `setEntityData` is
26.3-only), spawn pos (collision-empty → clicked pos, else beside face),
fluid `use`, consume + stat, entity_data application onto the spawned mob
(`apply_spawn_egg_entity_data`, `TypedEntityData.loadInto` merge),
offspring path (`spawn_egg_offspring`, incl. the WanderingTrader null
offspring), creative no-consume, random yaw.

**B7-G04 withdrawn.** The 26.2 tree HAS the peaceful gate
(`spawn_egg_allowed`, `creatures.rs:7520`): difficulty != 0, or the species
is allowed in peaceful. Jar-verified exactly: `EntityType.canSpawn` is
`isEnabled && (isAllowedInPeaceful || difficulty != PEACEFUL)`, and the 38
types calling `notInPeaceful()` in `EntityTypes.<clinit>` match the Rust's
`allowed_in_peaceful` denied list **38/38, zero differences either way**.
Note the MONSTER category has 45 types but only 38 are notInPeaceful —
piglin, shulker, ender_dragon, camel_husk, sulfur_cube, zombie_horse and
zombie_nautilus are MONSTER-category but allowed in peaceful, and the Rust
correctly allows them. (In 1.21.7 the gate was absent from the port AND
vanilla had no gate in `EntityType.spawn` — the 1.21.7 defect was the
port inventing a block; in 26.2 vanilla HAS the gate in `canSpawn` and the
port implements it faithfully.)

**B7-G05 withdrawn.** 26.2's `SpawnEggItem.useOn` delegates to `spawnMob`,
which posts ENTITY_PLACE at the **spawn** position (javap-verified) — the
Rust posts at the spawn position too. Vanilla changed between 1.21.7
(clicked pos) and 26.2 (spawn pos); the port matches 26.2.

**B7-G03 (minor, persists):** `entity_data` `id` type override ignored.
Vanilla 26.2's `SpawnEggItem.getType` reads the `entity_data` component's
typed `type()` and spawns THAT type; the Rust derives the type from the
item name only (`trim_end_matches("_spawn_egg")`), and
`spawn_egg_entity_data` returns None when the data's id names a different
type. Commands/op-only edge, same as 1.21.7.

**B7-G06 (minor, persists):** no `Mob.playAmbientSound()` on spawn-egg
placement. Vanilla 26.2's `EntityType.spawn` calls it after
`addFreshEntityWithPassengers` (javap-verified, `entitytype.txt:210`); the
Rust's `spawn_mob_spawned` / `use_spawn_egg` play no sound (ambient sounds
only come from the periodic `Mob.baseTick` timer later).

Notes (not numbered): failed spawn returns false (click falls through) vs
vanilla 26.2's FAIL from `spawnMob` — the Rust's fall-through differs from
both 1.21.7's SUCCESS and 26.2's FAIL, but only on a failed spawn (no room);
same observation class as 1.21.7, not filed. AlignPosition Y-offset
(`getYOffset`) not applied — same as 1.21.7, not filed.

### 3.5 Tools useOn (axe/hoe/shovel) — 3 minor gaps (B7-G13, B7-G14, B7-G15; B7-G07 withdrawn)

**B7-G07 withdrawn — the 26.2 port already fixed it.** 26.2 has NO
`minecraft:block_transformer` registry (§1), and the port does not wait for
one: `crates/horizon-client/src/block_transform.rs` hardcodes the 26.2
tables and `Transformers::load` ignores the root path ("26.2 has no
block_transformer data"). Jar-verified maps (from `AxeItem`/`HoeItem`/
`ShovelItem` `<clinit>`): STRIPPABLES 23 entries identical to 1.21.7's
list; TILLABLES 5 (grass_block/dirt_path/dirt→farmland, coarse_dirt→dirt,
rooted_dirt→dirt) with `onlyIfAirAbove` except rooted_dirt's always-true
predicate + hanging-roots drop; FLATTENABLES 6 identical to 1.21.7. Axe
order jar-verified: strip (AXE_STRIP) → scrape
(`WeatheringCopper.getPrevious`, AXE_SCRAPE + level event 3005) → wax-off
(`HoneycombItem.WAX_OFF_BY_BLOCK`, AXE_WAX_OFF + level event 3004).
Copper maps jar-verified: the 15 `WeatheringCopperCollection`s in
`WeatheringCopper`'s NEXT map and `HoneycombItem`'s WAXABLES map match the
Rust's `copper_stages()` 15 families exactly (0 differences either way).
`use_block_transformer` (`game.rs:7927`) does the swap, loot drop, sound,
stat, damage, BLOCK_CHANGE — and `transform_particle`
(`game/block_particles.rs:173`) sends the level events (scrape→3005,
wax_off→3004, wax_on→3003, dowse→1009), matching vanilla. Wired via
`use_block_transformer` (`game.rs:8263`) and prediction (`remote.rs:8830`).

**B7-G08 withdrawn** (campfire douse, §3.5 in 1.21.7 numbering — the douse
check itself passes): the port has `game/campfire_douse.rs` implementing
`ShovelItem.useOn`'s campfire branch directly: `#minecraft:shovels` item
tag, lit campfire/soul_campfire, face≠DOWN, level event 1009, 20 smoke
puffs, damage 1, trigger, BLOCK_CHANGE. Wired at `use_hand.rs:88` (server
path, before block use) and `predict_campfire_douse` at `remote.rs:8648`.
Jar-verified `ShovelItem.useOn`: face==DOWN → PASS gates BOTH branches;
FLATTENABLES + air above + SHOVEL_FLATTEN; campfire branch (instanceof
CampfireBlock + LIT → levelEvent 1009 server-only + `CampfireBlock.dowse` +
LIT=false); then setBlock 11 + BLOCK_CHANGE + hurtAndBreak(1). The
transformer's redundant dowse transform is never reached first because
`use_campfire_douse` runs earlier in dispatch.

**B7-G13 (minor, new):** the shield blocking-intent check in
`block_transform` (`game.rs:7895-7903`) applies to axe, hoe AND shovel
(offhand has `blocks_attacks` + not sneaking → return None), but vanilla's
`playerHasBlockingItemUseIntent` is **axe-only** (javap-verified: present
only in `AxeItem`; requires hand == MAIN_HAND + offhand has BLOCKS_ATTACKS
+ not secondary-use-active; no such check in HoeItem/ShovelItem). The Rust
also does not check the acting hand is the main hand. This was a latent
note in 1.21.7 (unobservable while the transformer was dead); now
observable in 26.2: holding a shield off-hand blocks hoe tilling and
shovel path-making/flattening where vanilla tills/flattens.

**B7-G14 (minor, new):** `use_block_transformer` fires
`item_used_on_block` (`game.rs:7945`) for ALL transforms, and
`use_campfire_douse` fires it too (`campfire_douse.rs:102`) — but vanilla
26.2 fires ITEM_USED_ON_BLOCK only in `AxeItem.useOn` (1 reference,
javap-verified); `HoeItem` and `ShovelItem` have **zero** trigger
references, and `ItemStack.useOn` does not fire it either (it only awards
the ITEM_USED stat on Success). Hoe tilling, shovel flattening and shovel
campfire dousing incorrectly trigger the advancement criterion in the port.

**B7-G15 (minor, new):** `transform_copper_chest_neighbour`
(`game.rs:7980-8019`) posts a BLOCK_CHANGE game event AND the scrape/wax-off
level event (3004/3005) at the NEIGHBOUR half of a double copper chest.
Vanilla 26.2's `AxeItem.useOn` sends the level event and BLOCK_CHANGE only
at the clicked position. The neighbour particle + sculk-visible BLOCK_CHANGE
is 26.3 `BlockTransformer` COPPER_CHEST behaviour leaking into the 26.2
port (the `transform_type == "copper_chest"` branch in
`block_transform.rs:201,309,315`).

Notes (not numbered): `predict_campfire_douse` returns false while
sneaking — prediction-only lag; the server still douses. The transformer
maps tool by item-name suffix (`_axe`/`_hoe`/`_shovel`) — works for all 7
materials each.

### 3.6 Food / consumable lifecycle — pass (B7-G09 withdrawn)

**B7-G09 withdrawn.** 26.2's `LivingEntity.randomTeleport` is
**blocksMotion-grounded**, not tag-grounded (javap-verified): it walks down
while y > minY looking for `BlockState.blocksMotion()`, then `teleportTo` +
`noCollision` + `!containsAnyLiquid`, rolling back to the old position on
failure, entity event 46 on success, PathfinderMob navigation stop. Zero
references to `entities_can_teleport_to` anywhere in the class. The Rust's
`random_teleport` (`game/consume.rs:123`) mirrors it exactly (blocksMotion
walk, `check_position_and_teleport` with noCollision + liquid check; the
pre-move check is equivalent to vanilla's move-then-rollback). The rest of
the lifecycle (FoodData.eat clamps, eat/burp sounds, `canEat`, consume
timing, `should_emit`, use remainder, `clear_all_effects`/`remove_effects`,
`ApplyEffects`, teleport dispatch diameter 16 / 16 attempts) is the shared
code verified in 1.21.7. Chorus fruit data correct (teleport_randomly,
use_cooldown 1.0s applied at finishUsingItem — see §3.15 for the instant-use
contrast).

### 3.7 Shield blocking — pass

Jar shield component in items.json: `blocks_attacks` (block_delay_seconds
0.25, block_sound `item.shield.block`, bypassed_by `#bypasses_shield`,
disabled_sound `item.shield.break`, item_damage {base 1.0, factor 1.0,
threshold 3.0}). The port reads the component (`shield.rs:34`
`blocks_attacks`, `items::BlocksAttacks::of_stack`), applies the item-damage
function (below threshold → 0, else floor(base + factor×blocked),
`shield.rs:148-163`), the disabled sound (`shield.rs:299`), use duration
72000 for blocks_attacks items (`item_use.rs:78`), and the raise/cooldown
path (`raise_shield`). Reduction resolve, delay ticks, disable ticks, axe
disable, Warden/Ravager handling are the shared code verified in 1.21.7.

### 3.8 Bow — pass

`getPowerForTime` = `(f² + 2f)/3` clamped to 1.0, jar-verified identical to
1.21.7; the Rust matches exactly (`game/item_use.rs:240`,
`first_person.rs:114,247`). The rest (0.1 minimum to fire, `use` gate,
arrow selection, shootFromRotation power×3 / inaccuracy 1.0, critical iff
power == 1.0, Infinity/Flame/Power, sound, stat, ammo, damage 1) is the
shared code verified in 1.21.7.

### 3.9 Crossbow — pass

Charge duration `floor(modifyCrossbowChargingTime(1.25) × 20)`
jar-verified; the Rust matches (`game/crossbow.rs:80-83`:
`(1.25 − 0.25×QuickCharge).max(0) × 20` floored). Charging sounds match
(quick_charge_1/2/3, no middle for Quick Charge, loading_end). The rest
(load at 1.0, Multishot 3, rocket 1.6 / arrow 3.15, shot pitches, Piercing,
SHOT_CROSSBOW trigger, stat) is the shared code verified in 1.21.7.

### 3.10 Trident — pass

26.2's `TridentItem.releaseUsing` (boolean-returning) **does** require
`isInWaterOrRain` AND NOT `isPassenger` for riptide > 0, else returns false;
`use` fails if `nextDamageWillBreak` or (riptide and not wet). So the
1.21.7 B11-G08 finding (port blocks Riptide while riding where vanilla
1.21.7 has no check) does not apply to 26.2 — the port is correct for 26.2.
The rest (≥10 ticks release, riptide strength, spin attack, throw 2.5,
loyalty, channeling) is the shared code verified in 1.21.7. Note (not
filed): `in_water_or_rain` checks the feet block only where vanilla checks
the bounding box — same approximation as 1.21.7.

### 3.11 Potions — pass (B7-G10, B7-G11 withdrawn)

**Drink — pass.** Consumable data, 32-tick duration, drink sounds, contents
effects at `potion_duration_scale`, stat + trigger + game event,
glass-bottle remainder — shared code verified in 1.21.7.

**Splash effects — pass.** AABB inflate (4, 2, 4), d² < 16, intensity,
duration truncation, instant harm/heal, undead inversion, level events
2002/2007, axolotl rehydrate +1800 air — shared code verified in 1.21.7.

**B7-G10 withdrawn.** 26.2's `AbstractThrownPotion` (moved to
`net.minecraft.world.entity.projectile.throwableitemprojectile` in 26.2)
gates `onHitAsWater`/`dowseFire` on `PotionContents.is(Potions.WATER)`
directly — **0 PotionTags references** (javap-verified). The Rust's
`game/splash_water.rs` gates on the same direct check
(`stack.potion() == Some("minecraft:water")`); douse positions (hit cell,
cell in front, 4 horizontal neighbours), hurt 1.0 indirect magic,
extinguish, axolotl rehydrate all present with correct constants, and
`dowseFire` matches (fire tag → destroyBlock no drops; lit candle →
extinguish; lit campfire → levelEvent 1009 + dowse). Callers verified:
`potions.rs:1952/1954` call `potion_hit_block` + `on_hit_as_water`
unconditionally with the gate inside. (In 1.21.7 the port gated on potion
tags that did not exist; the 26.2 port gates directly, as vanilla 26.2
does.)

**Lingering cloud — pass.** Radius 3.0, radiusOnUse −0.5, duration 600,
waitTime 10, radiusPerTick −3/600, duration scale 0.25 — shared code
verified in 1.21.7.

**Throw — pass.** Splash/lingering −20 pitch, 0.5 velocity, 1.0 inaccuracy,
sound, stat, consume; exp bottle XP; glass bottle dragon-breath/water fill
— shared code verified in 1.21.7. Observation (not filed): the throw sound
plays at the eye where vanilla plays at the feet — same as 1.21.7.

**B7-G11 withdrawn.** The real 26.2 tag is `minecraft:convertable_to_mud`
(with an A) — `BlockTags.CONVERTABLE_TO_MUD` in `PotionItem.useOn`
(javap-verified), `data/minecraft/tags/block/convertable_to_mud.json`
exists in the jar, and prepared data carries the same key. The Rust uses
`convertable_to_mud` in all four places (`game/dispenser.rs:2241`,
`game/potions.rs:710`, `game/remote.rs:8695`,
`horizon-world/src/level/data.rs:461`). The rest of
`use_water_bottle_on_block` matches vanilla `PotionItem.useOn` (face ≠
DOWN, mud + GENERIC_SPLASH + particles + bottle-empty sound + FLUID_PLACE +
glass-bottle remainder + stat).

### 3.12 Boats — pass except B7-G12 (minor, persists)

Verified matching: POV raycast, Fluid.ANY; boat spawned at the exact hit
location with the player's yaw; noCollision check (block unobstructed +
`boat_entity_collision`); ENTITY_PLACE at the hit; consume + ITEM_USED stat;
chest boats via `with_chest()`.

**B7-G12 (minor, persists):** vanilla 26.2's `BoatItem.use` returns PASS
when the player's eye is inside a pickable entity's inflated bounding box
(entities in the player bbox expanded 5.0 along the view + inflate 1.0;
javap-verified, same as 1.21.7). The Rust's `boat_to_place`
(`game/boats.rs:1505`) only checks the boat's own bbox
(`boat_entity_collision` checks the boat's bbox vs entities, not the eye).
Edge case: standing inside a mob while placing a boat — the Rust places
where vanilla passes.

### 3.13 Armour damage reduction — pass

`CombatRules.getDamageAfterAbsorb` jar-verified for 26.2: divisor
`2 + toughness/4`, protection `clamp(armor − damage/divisor, armor×0.2,
20)`, damage × (1 − protection/25), with the `armor_effectiveness`
enchantment modify (Breach) clamped 0..1. The Rust matches exactly
(`mobs.rs:3835-3847`); the `armor == 0` skip is outcome-equivalent
(effective clamps to 0). Order armor → resistance → protection matches
vanilla; Resistance `(25 − 5×(amp+1))/25`; Protection EPF/25; armour
damaged by the pre-reduction amount; armor value floors the effective
double, toughness converts directly (`mobs/armor.rs:43-65`). Copper horse
armour (new in 26.2) and netherite horse armour use the same body-slot
path as the existing horse armours.

### 3.14 Mace — pass

Attributes verified in D4 (5.0 damage, −3.4 speed via `createAttributes`).
Smash mechanics present in `game/mace.rs`: threshold fall > 1.5 and not
fall-flying, damage bonus tiers + Density, knockback, radius 3.5, fall stop,
`mace_smash` damage source, Wind Burst — shared code verified in 1.21.7.

### 3.15 Ender pearl — pass except B7-G16 (minor, new)

Verified matching: throw velocity 1.5, inaccuracy 1.0, sound vol 0.5 pitch
`0.4/(rand×0.4+0.8)`, stat, consume (`game/creatures.rs:7420-7448`).
Teleport to the pearl's oldPosition, 5.0 damage, 5% endermite,
owner-alive/not-sleeping gate, single teleport sound at the destination —
shared code verified in 1.21.7.

**B7-G16 (minor, new):** thrown ender pearls and wind charges start NO
cooldown in the port, but vanilla 26.2 starts one. `ItemStack.use`
(javap-verified) calls `applyAfterUseComponentSideEffects` for instant-use
items (use duration ≤ 0) on Success — applying both USE_REMAINDER and
USE_COOLDOWN. items.json: ender_pearl 1.0s, wind_charge 0.5s. The port
implements the opposite belief in four places — `game/creatures.rs:7432`
("26.2's ItemStack.use does not apply the `use_cooldown` component, so a
thrown pearl or wind charge starts no cooldown here (26.3 added the
side-effects call to ItemStack.use)"), `game/remote.rs:9327`,
`game/using_item.rs:26`, and the throw path applies no cooldown. The
comment's premise is wrong for 26.2: the side-effects call IS in 26.2's
`ItemStack.use`. (Chorus fruit is unaffected — it is a timed consume, so
its 1.0s cooldown applies at `finishUsingItem`, which the port handles via
`complete_using_item` → `apply_use_cooldown`. No instant-use item has a
use_remainder, so only the cooldown half bites.) Pearls and wind charges
can be spammed with no cooldown in the port; the cooldown gate itself
exists (`use_hand.rs:236`, `remote.rs:9235`) but is never armed for them.

Observations (not filed): endermite skipped in peaceful where vanilla
spawns then despawns (unobservable); pearl portal-cooldown transfer not
implemented — same as 1.21.7.

### 3.16 Fishing rod — pass

`use` — reel-in vs cast; retrieve damage (hooked entity 5, item entity 3,
`game/fishing.rs:544` `retrieve_hook`), FISHING_BOBBER_RETRIEVE vol 1 pitch
`0.4/(rand×0.4+0.8)` at the feet, ITEM_INTERACT_FINISH vibration
(`game/fishing.rs:462-476`); pull 0.1 toward owner; FISHING_ROD_HOOKED
trigger. Cast velocity, Lure/Luck of the Sea, wait/hooked/nibble timers,
open-water check, loot/XP — shared code verified in 1.21.7. Observation
(not filed): `canSeeSky` approximated by sky-light 15 — same as 1.21.7.

### 3.17 Spears — pass (new 26.2 family)

All 7 spears are plain `Item` with data components (§1, §2). The port
implements the family in `spear.rs` (`KineticWeapon::of` hardcodes the
per-material table), `game/hand.rs` (attack_range: min_reach 2.0 /
max_reach 4.5 read from the component), `game/spear.rs` (minimum_attack_charge
1.0), `mobs/spear_use.rs` (mob spear use), and `first_person.rs`
(piercing-weapon swing). The hardcoded table matches the jar's
`Item$Properties.spear` args exactly for all 7 materials (verified in §2:
all 35 tick values under float32 arithmetic), including the wooden spear's
`spear_wood` sound variants and the f32 rounding edges (copper delay 13,
stone/golden 14). Attack speed = (1.0/duration) − 4.0 and attack damage =
material bonus verified in D4.

### 3.18 Nautilus armour — pass (new 26.2 family)

Attributes verified in D7 (5 pieces, body-slot defense from ArmorMaterials;
no enchantable, no durability, stacksTo(1) — `Item$Properties.nautilusArmor`
bytecode). Equippable component in items.json: slot `body`, allowed
entities `#minecraft:can_wear_nautilus_armor`, equip_on_interact true,
equip sound `item.armor.equip_nautilus`, shearing sound
`item.armor.unequip_nautilus`. The port reads `allowed_entities` from the
component (`menu/equippable.rs:45`) and includes nautilus species in the
armour-wearing check (`game/mount_menu.rs:48`). Same body-slot equip path
as horse/wolf armour (verified in 1.21.7).

## 4. Defect list

| ID | Severity | Title |
|----|----------|-------|
| B7-G03 | minor | Spawn egg entity_data id type override ignored (persists from 1.21.7) |
| B7-G06 | minor | No Mob.playAmbientSound() on spawn-egg placement (persists from 1.21.7) |
| B7-G12 | minor | Boat placement missing the eye-inside-entity PASS check (persists from 1.21.7) |
| B7-G13 | minor | Shield blocking-intent check (axe-only in vanilla) also blocks hoe/shovel transforms, and ignores the acting hand |
| B7-G14 | minor | ITEM_USED_ON_BLOCK trigger fired for hoe till, shovel flatten and shovel campfire douse; vanilla fires it only for the axe |
| B7-G15 | minor | Copper-chest scrape/wax-off posts a BLOCK_CHANGE + level event at the neighbour half (26.3 COPPER_CHEST behaviour) |
| B7-G16 | minor | Thrown ender pearls / wind charges start no use_cooldown; vanilla 26.2 applies it in ItemStack.use |

Withdrawn for 26.2 (re-checked against 26.2's jar + 26.2 tree; numbers NOT
reused):

| ID | Why withdrawn |
|----|---------------|
| B7-G01 | Fixed in the 26.2 tree: `ComponentMap::set` returns `Ok(base)` on a dangling removal marker (§3.1) |
| B7-G02 | 26.2's `BucketItem.emptyContents` HAS the sneak check — the port is correct for 26.2 (§3.2) |
| B7-G04 | The 26.2 tree HAS the peaceful gate; its 38-type list matches the jar's `notInPeaceful()` set exactly (§3.4) |
| B7-G05 | 26.2's `spawnMob` posts ENTITY_PLACE at the spawn position, as does the port (§3.4) |
| B7-G07 | 26.2 has no `block_transformer` registry and the port hardcodes the correct tables instead (§3.5) |
| B7-G08 | The port implements the shovel campfire douse directly in `campfire_douse.rs` (§3.5) |
| B7-G09 | 26.2's `randomTeleport` is blocksMotion-grounded, as is the port's (§3.6) |
| B7-G10 | 26.2 gates splash-water on `is(Potions.WATER)` directly, as does the port (§3.11) |
| B7-G11 | The port uses `convertable_to_mud` (with an A) in all four places (§3.11) |

**Systemic note.** The 26.2 story is the inverse of 1.21.7's. In 1.21.7,
four majors shared one root cause: the port carried 26.x mechanisms for
behaviours 1.21.7 implements in hardcoded Java, and the lookups silently
degraded. In 26.2, the port authors already did the version-specific work:
the block_transformer registry is correctly absent and replaced by
hardcoded tables, the campfire douse / splash-water / teleport gates are
implemented in 26.2's own (non-tag) form, and the bucket sneak check and
spawn-egg peaceful gate — which ARE 26.2 behaviours — are implemented. The
seven defects left are all small, local, and minor: three are 1.21.7
leftovers the 26.2 work did not touch (G03, G06, G12), and four are new
26.2 divergences where the port over- or under-applies a 26.2 rule (G13:
axe-only check widened to hoe/shovel; G14: axe-only trigger widened to
hoe/shovel; G15: a 26.3 chest behaviour carried into 26.2; G16: a
wrong "26.2's ItemStack.use does not apply use_cooldown" belief baked into
four comments). No whole item family is non-functional in 26.2.

## 5. Driver inventory — 74 drivers, none run

Driver inventory is DONE (saved to
`b7-26.2-analysis/driver-map-26.2.json`); **no driver was run** — this batch
is static checks only per the brief. Statuses below are MEASURED oracle
status from `docs/sweep/26.2/batch-0-oracles.json` (Batch 0's javac
measurement against the 26.2 jars; "runnable" = every oracle the driver
needs passes on at least one classpath — client is the superset that
matters). 1.21.7 statuses are NOT copied: 24 drivers that were blocked in
1.21.7 are runnable for 26.2.

- **38 runnable** (oracles pass; NOT run): food, food_properties,
  food_storage, player_food, consumable_emission, consume_effect,
  stored_consume, teleport_consume, item_use, item_use_lifecycle,
  item_after_use, use_flag_reaction, inventory_use, enchant_durability,
  stack_mining, creative_mining, blocking_rules, trident, helmet_damage,
  player_equipment, damage_reduction, combat_rules, component_patch,
  enchantment_components, stored_enchantment_patch, trade_components,
  item_stack_storage, item_wire, item_cooldowns, stack_foil,
  pick_block_data, explosion_items, bundle_weight, lava_cauldron_item,
  map_saved_data, hotbar, inventory_tick, enchanting. Note: `stack_foil`
  (VanillaStackModelFlagsOracle) and `hotbar` (VanillaHotbarOracle) pass on
  client+both but fail on server — runnable on the client cp.
- **19 blocked** by 18 distinct failing oracles (measured error class +
  count; all fail on all three classpaths): consumable
  (VanillaConsumableOracle, missing-symbol, 1), item_use_catalog
  (VanillaItemUseCatalogOracle, missing-symbol, 2), item_use_order
  (KnockbackClient, missing-symbol, 4), item_use_start
  (VanillaItemUseStartOracle, constructor-mismatch, 1 — ServerChunkCache),
  use_item_on + mace (VanillaDispenserOracle, no-override, 1),
  use_remainder + item_template (VanillaUseRemainderOracle /
  VanillaItemTemplateOracle, other, 1 — `ItemStackTemplate.fromNonEmptyStack`
  signature), inventory_use_start
  (VanillaInventoryUseStartOracle, constructor-mismatch, 1 —
  ServerChunkCache), interaction_input
  (VanillaInteractionInputOracle, constructor-mismatch, 3 — LocalPlayer),
  item_durability (VanillaItemDurabilityOracle, incompatible-types, 1 —
  Item → ItemStack), component_durability
  (VanillaComponentDurabilityOracle, incompatible-types, 2),
  blocking_item_damage (VanillaBlockingItemDamageOracle, no-override, 2),
  item_blocking (VanillaItemBlockingOracle, no-override, 2), extra_items
  (VanillaExtraItemsOracle, missing-symbol, 2), held_item_stack
  (VanillaHeldItemStackOracle, missing-symbol, 4), held_drop
  (VanillaHeldDropOracle, constructor-mismatch, 4 — LocalPlayer),
  inventory_pickup (VanillaInventoryPickupOracle, missing-symbol, 1),
  player_inventory (VanillaPlayerInventoryOracle, missing-symbol, 1). All
  19 were also blocked in 1.21.7 (same oracle, mostly the same error class).
  No oracle repairs in this batch.
- **17 no-oracle** (mostly live-game e2e drivers needing a built game +
  fixture servers; NOT run): interaction_data, inline_fishing_tool,
  trident_save, pumpkin_equippable_use, builtin_component_delivery,
  effective_component_presence, effective_item_components,
  numeric_damage_component, item_contents, item_numeric_uuid,
  item_health_load, item_entity_max_stack, spawn_egg_fluid, boat_item_hit,
  map_decoration_patch, map_decoration_namespace,
  enchantment_map_validation. Same 17 as 1.21.7. Flags (carried from
  1.21.7): `effective_item_components` and `pick_block_data` docstrings say
  "(Java 26.3)" — possible 26.3-only drivers.

**Oracle defects found: none new.** The 18 blocking oracles are Batch 0's
measured failures, not new findings. **Oracle repairs done: none** (none in
this batch).

## 6. Method notes

- Every vanilla claim is cited to a specific jar method + bytecode, read
  with `javap -c -p` (JDK 25) on the named 26.2 server-game jar. No
  behaviour was assumed from the 26.3 base or from memory — and no 1.21.7
  defect was assumed to persist: each was re-checked against the 26.2 tree's
  actual Rust, and nine were withdrawn (four because the 26.2 tree fixed
  them, five because 26.2's vanilla differs from 1.21.7's).
- 26.2 class moves found during the batch: `AbstractThrownPotion` moved to
  `net.minecraft.world.entity.projectile.throwableitemprojectile`;
  `EntityType` registrations moved to `net.minecraft.world.entity.EntityTypes`
  (with `notInPeaceful()` per type); `SpawnEggItem.getType` reads the
  typed `entity_data` component (no `CustomData.parseEntityType`).
- Rust item data is data-driven from `items.json` at runtime (`ItemInfo` in
  `horizon-items/src/items.rs`); data parity therefore hinges on prepared
  `items.json` fidelity (verified in §2) and loader/behaviour-code
  interpretation (verified per family in §3). Two exceptions are hardcoded
  tables rather than data-driven: the tool transforms
  (`block_transform.rs`, correctly — 26.2 has no registry) and the spear
  kinetic-weapon table (`spear.rs`, values verified exact against the jar).
- Check counting: one check per behaviour pattern, grouped by behaviour
  class per the brief (all tools of one kind are one check pattern, not N
  checks). The 42 checks are D1–D8 (§2) and B1–B34 (§3): the 32 1.21.7
  behaviour checks plus B33 spears and B34 nautilus armour (new 26.2
  families). A check is "gap" if it contains at least one defect; B7-G13
  and B7-G14 each span two gap checks (hoe + shovel).
- Analysis artefacts (scripts, jar dumps, check outputs, driver map) live in
  `~/workspace/mcir-work/b7-26.2-analysis/`. Note: the first scratch dir
  (`/tmp/b7-26.2/`) was lost to a service restart mid-batch (`/tmp` is a
  tmpfs); all §1/§2 results were preserved in the working record and the
  remaining javap dumps were re-run into the workspace analysis dir. No
  deliverable content was lost.
- The 1.21.7 report's observations carried forward and re-confirmed as
  still-true-but-unfiled for 26.2: potion throw sound at the eye (not the
  feet); endermite skipped in peaceful; pearl portal-cooldown transfer not
  implemented; fishing `canSeeSky` approximated by sky-light 15; spawn-egg
  alignPosition Y-offset not applied; failed spawn-egg spawn returns false
  (falls through) vs vanilla FAIL; trident `in_water_or_rain` checks the
  feet block only. New observations for 26.2: `predict_campfire_douse`
  returns false while sneaking (prediction-only lag); the splash path does
  not apply `POTION_DURATION_SCALE` (default 1.0 for splash potions, so
  only commands-modified scales differ — same as 1.21.7).

## 7. What this batch defers

- Running the 38 runnable drivers (the food driver in particular was called
  out in the 1.21.7 brief; Batch 0 repaired its oracle) and the 17
  no-oracle drivers (live-game e2e; need a built game + fixture servers).
- Repairing any of the 18 blocking oracles (most are 1–4 javac errors;
  VanillaDispenserOracle blocks two drivers) and end-to-end driver runs to
  confirm each repair yields a measurement.
- Committing these deliverables (docs/sweep/26.2/batch-7-items.md,
  pins/26.2/sweep/batch-07.json) — the coordinator's job.
- Fixes for the 7 defects are a separate decision on separate branches.

Nothing in this batch touched production code (`crates/`), per the
checks-only brief.
