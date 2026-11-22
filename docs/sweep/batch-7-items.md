# Batch 7 — Items (1.21.7 parity sweep)

**Status: DRAFT — uncommitted, awaiting the user's go-ahead.** An execution-risk
notice blocked all repo execution (git, cargo, javac, driver runs) partway
through this batch. Per the Batch 2 precedent no workaround was attempted:
this batch is static analysis only (javap + source + data). Deliverables live
under `~/workspace/mcir-work/sweep-batch-7-deliverables/` until the block is
lifted; nothing was committed to `parity-sweep`.

- **Branch / tip checked:** `parity-sweep` @ `d27d1d33` (2026-11-22)
- **Spec:** the named 1.21.7 server jar
  (`~/workspace/mcir-work/leakage-audit/jars/1.21.7-server-named.jar`) and
  prepared data (`~/workspace/mcir-work/mob-spawn/resources/`). Vanilla
  1.21.7's own jar/data is the spec — never the 26.3 base's form (owner's
  vanilla-check rule).
- **Checks: 40 total — 26 pass, 14 gap. Defects: 12 (4 major, 8 minor).**
- **Oracle repairs: none** (blocked). **New oracle defects: none** (see §5).

## 1. The §1.1 spec gap — resolved

`block-behaviours.json` holds only 965 BlockItem entries; the other 451 items
had no behaviour-class entry. Resolved by extracting the item→Java-class map
from the named jar: `javap -v` BootstrapMethods (306 entries) resolves each
`invokedynamic` to a factory lambda or direct constructor/method ref, and a
`javap -c` `<clinit>` walk assigns one class per item segment
(script `b7-analysis/parse_items_class.py`).

**Result: all 1,416 items mapped to 70 distinct classes, zero unmapped, zero
mismatches** between `block-behaviours.json`'s class field and the jar. The
451 break down as: 214 plain `Item` (data-driven via components), 82
SpawnEggItem, 20 BoatItem, 19 SmithingTemplateItem, 17 BundleItem, 16 DyeItem,
6 each MobBucketItem / MinecartItem / AxeItem / HoeItem / ShovelItem, 3 each
EggItem / BucketItem, 2 each FoodOnAStickItem / ItemFrameItem, 43 singletons
(BowItem, CrossbowItem, TridentItem, FishingRodItem, ShieldItem, MaceItem,
PotionItem, SplashPotionItem, LingeringPotionItem, …).

Key 1.21.7 facts established from the jar:

- There is **no** `SwordItem`, `PickaxeItem`, `ArmorItem`, or `ElytraItem`
  class — swords/pickaxes/armour/elytra are plain `Item` with tool/weapon/
  equippable/attribute components. `AxeItem`/`HoeItem`/`ShovelItem` exist
  (strip/till/path `useOn`).
- `milk_bucket` is plain `Item` (consumable + use_remainder).
- `items.json` components alone do NOT resolve the gap: 277 of the 451 have
  no behaviour component at all (boats, spawn eggs, buckets, minecarts,
  compass, clock, bone_meal, …) — their behaviour lives in the Java class.
- Component frequencies in `items.json` (all 1,416): food 40, consumable 43,
  tool 33, weapon 32, equippable 73, damage/max_damage 68, enchantable 61,
  repairable 59, bundle_contents 17, use_remainder 7, use_cooldown 3
  (chorus_fruit, ender_pearl, wind_charge), potion_contents 4,
  jukebox_playable 21, fireworks 1, instrument 1, glider 1, blocks_attacks 1,
  death_protection 1.

Parser traps (do not re-debug): invokedynamic lines have two spaces after the
comma (`#1479, 0`); the last lambda before `<clinit>` absorbs `<clinit>`
unless `static {};` ends a method body; the `createBlockItemWithCustomItemName`
helper covers 15 items (seeds, redstone, string, carrot, potato, berries, … —
they ARE BlockItems); field names ≠ registry ids (`DRY_SHORT_GRASS` /
`DRY_TALL_GRASS` vs `short_dry_grass` / `tall_dry_grass`; `CUT_STANDSTONE_SLAB`
is a Mojang field-name typo); bootstrap entries 304/305 are
`SmithingTemplateItem.createNetheriteUpgradeTemplate` /
`createArmorTrimTemplate` factory method-refs.

## 2. Data checks — 8/8 pass

| # | Check | Result |
|---|-------|--------|
| D1 | Food: nutrition, saturation (= nutrition × modifier × 2, float32 tolerance), can_always_eat — 40/40 | pass |
| D2 | Max damage — 66 verified + 2 hand-verified (turtle_helmet 275 = 25×11, wolf_armor 64 = 4×16 body), 0 mismatches | pass |
| D3 | Max stack — 1,416/1,416 (durability ⇒ max_stack 1; horseArmor `stacksTo(1)`; explicit `stacksTo` wins; default 64) | pass |
| D4 | Attack damage/speed — 30/30 (material bonus + tool args, float32) | pass |
| D5 | Tool mining speed — rule speeds vs ToolMaterial.speed, 0 mismatches | pass |
| D6 | Tool-rule tags — 14/14 exist in the jar and contents match prepared `block-tags.json` exactly | pass |
| D7 | Armour attributes — 30/30 (defense/toughness/knockback-resistance/enchant value; 28 wearable + 4 horse + wolf body) | pass |
| D8 | Item→class mapping — 1,416/1,416, 70 classes, 0 mismatches (see §1) | pass |

Jar-verified constants recorded for later batches:

- **ToolMaterial** [durability, speed, attackDamageBonus, enchantmentValue]:
  WOOD [59, 2.0, 0.0, 15], STONE [131, 4.0, 1.0, 5], IRON [250, 6.0, 2.0, 14],
  DIAMOND [1561, 8.0, 3.0, 10], GOLD [32, 12.0, 0.0, 22], NETHERITE [2031, 9.0, 4.0, 15].
- **ArmorMaterials** [durability, defense helmet/chestplate/leggings/boots/body,
  enchantment, toughness, knockback-resistance]: LEATHER [5, 1/3/2/1/3, 15, 0, 0],
  CHAINMAIL [15, 2/5/4/1/4, 12], IRON [15, 2/6/5/2/5, 9], GOLD [7, 2/5/3/1/7, 25],
  DIAMOND [33, 3/8/6/3/11, 10, 2.0], TURTLE_SCUTE [25, 2/6/5/2/5],
  NETHERITE [37, 3/8/6/3/11, 15, 3.0, 0.1], ARMADILLO_SCUTE [4, 3/8/6/3/11, 10].
  Slot bases: helmet 11, chestplate 16, leggings 15, boots 13.
- `Foods.<clinit>` uses a Builder plus a `stew(N)` helper (saturation
  modifier 0.6f). `items.json` carries float32 artifacts (e.g. 7.2000003).

## 3. Behaviour checks — 18 pass, 14 gap

### 3.1 component_patch — gap (B7-G01, minor)

Batch 0's end-to-end sample: first divergence at output line 106,152
(vanilla `old none` vs Rust `result error`); the first 106,151 lines match.
Confirmed from Batch 0's saved outputs: Rust has exactly **77 `result error`**
vs vanilla 0; ok counts differ by exactly 77; old-lines differ by exactly 77.
With `^old ` / `^result ` lines stripped, both 527,506-line outputs are
**byte-identical** — the 77 set-return-values are the ONLY divergence;
post-op map state always matches.

Root cause: Rust `ComponentMap::set` (`horizon-world/src/component_patch.rs`,
~106–118) returns `Err(MissingPrototype)` when overwriting a removal marker
for a key with no prototype default. Vanilla `PatchedDataComponentMap.set`
returns `previous.orElse(prototype)` — null when there is no prototype
default, never an error (javap-verified; `get` returns
`Optional.orElse(null)` for a removal marker). The Rust doc comment ("An
unsanitized removal with no prototype causes an error after mutation") is
wrong — vanilla never errors here. First divergence is the special fixture:
`build([[0,None],[1,2],[0,None]])` + `restore()` (unsanitized, creates the
dangling removal marker) + `set(key=0, value=1)` on an empty prototype.
Fix shape (not to implement — checks only): `Some(None) => Ok(base)`.
Severity minor: return-value-only, requires a dangling removal marker.

### 3.2 Buckets — pass except B7-G02 (minor)

Verified matching: empty-bucket pickup (source-only — `level == 0`;
waterlogged via `SimpleWaterloggedBlock extends BucketPickup`; bubble column;
powder snow; barrier-creative), fill sounds (fill/fill_lava/fill_powder_snow),
FLUID_PICKUP, filled_bucket trigger, `createFilledResult` semantics (creative
keep+give, survival shrink/replace/drop), full-bucket place target
(LiquidBlockContainer + water only), emptyContents (canBeReplaced, retry
beside with null hit result, ultra-warm evaporation: FIRE_EXTINGUISH 0.5 vol,
pitch 2.6 + (r1−r2)×0.8, 8 LARGE_SMOKE), placeLiquid into container
(`getSource(false)` ≡ level-8 non-falling), destroyBlock condition
(mayReplace && !liquid, server only), setBlock flags 11 + isSource fallback,
playEmptySound (lava → BUCKET_EMPTY_LAVA else BUCKET_EMPTY, vol/pitch 1,
BLOCKS, + FLUID_PLACE; mob buckets no game event), mob-bucket spawn +
ENTITY_PLACE (at target pos), powder snow place (air only + FLUID_PLACE +
sound), cauldron pickup via `use_on_cauldron` first (matching vanilla
`CauldronBlock.useItemOn` precedence). Adventure-mode check resolved as
matching: vanilla `Player.mayUseItemAt(beside, direction, stack)` tests
`beside.relative(direction.getOpposite())` = clicked pos; Rust tests the
clicked pos.

**B7-G02 (minor):** Rust `bucket_empty` pours beside the clicked block when
sneaking (any non-air target). Vanilla 1.21.7 has NO sneak check anywhere in
the bucket path — zero `isShiftKeyDown` references in BucketItem /
MobBucketItem / SolidBucketItem, `use`, `emptyContents`, or
`SimpleWaterloggedBlock.canPlaceLiquid` (all javap-verified). Likely a 26.x
behaviour leaked into the port. Dispensers unaffected.

### 3.3 Bundles — pass

Verified matching: weight (`BUNDLE_IN_BUNDLE_WEIGHT` = Fraction(1,16) from
`<clinit>`; beehive-with-bees = 1; else 1/maxStackSize), `canItemBeInBundle`
(non-empty + `canFitInsideContainerItems` — shulker-only override in 1.21.7),
`tryInsert` (amount = min(count, getMaxAmountToAdd); `findStackIndex` via
`isStackable` + `isSameItemSameComponents`; merge + insert at front, else
split + insert at front), `getMaxAmountToAdd` ((1−weight)/itemWeight
truncated, max 0), `removeOne` (selected if in bounds else index 0; weight
−= getWeight×count; selection → −1), bar (width = `1 + mulAndTruncate(weight,
12)` min 13; FULL_BAR_COLOR = ARGB(1, 1, 0.33, 0.33) = 0xFF5454; BAR_COLOR =
ARGB(1, 0.44, 0.53, 1) = 0x7087FF; visible iff weight > 0),
`getNumberOfItemsToShow` (size>12→11 else 12; empty = 4 − size%4 if nonzero;
min(size, available−empty)), `toggleSelectedItem`, use (startUsingItem,
duration 200 ticks, drop cadence: first tick, then every 2nd tick once
remaining < 190), `dropContent` (removeOne + playRemoveOneSound + Player.drop
+ playDropContentsSound + stat, only when something removed), `onDestroyed`
spill.

### 3.4 Spawn eggs — 4 minor gaps (B7-G03–G06)

Verified matching: useOn spawner (setEntityId + BLOCK_CHANGE + shrink), spawn
pos (collision-empty → clicked pos, else beside face), fluid `use`
(SOURCE_ONLY ray + LiquidBlock check + mayInteract/mayUseItemAt), consume +
stat on `use`, entity_data application onto spawned mob, offspring path,
creative no-consume, random yaw 0–360. The useOn ITEM_USED stat was checked
and matches: `ItemStack.useOn` awards ITEM_USED when the result is a Success
with `wasItemInteraction()` true, and `ItemContext.DEFAULT = new
ItemContext(true, null)` — so plain `InteractionResult.SUCCESS` (returned by
SpawnEggItem.useOn and AxeItem.useOn) DOES award the stat.

**B7-G03 (minor):** `entity_data` `id` type override ignored. Vanilla
`SpawnEggItem.getType` reads `entity_data`'s `id` via
`CustomData.parseEntityType` and spawns that type; Rust derives the type from
the item name only. Commands/op-only edge.

**B7-G04 (minor):** hostile spawn eggs refused in peaceful. Rust
`spawn_egg_allowed` blocks the use (egg kept, no spawn). Vanilla has no
difficulty gate in `EntityType.spawn`; the egg is consumed, the mob spawns,
and `Mob.checkDespawn` discards it on the next tick. Rust also has tick-time
peaceful removal (`mobs/ai.rs`), so the gate is an extra, divergent block.

**B7-G05 (minor):** ENTITY_PLACE game event posted at the spawn position on
useOn; vanilla posts it at the CLICKED pos (`gameEvent(player, ENTITY_PLACE,
blockPos)`). Fluid `use` path matches.

**B7-G06 (minor):** no `Mob.playAmbientSound()` on spawn-egg/mob-bucket
placement. Vanilla `EntityType.spawn` plays it; the Rust's
`mobs::ai::ambient_sound` exists ("a spawn egg or a mob bucket just placed
`m`") but is only called from the dispenser path and a test — zero callers in
`creatures.rs`.

Notes (not numbered): alignPosition Y-offset (`getYOffset`) not applied — Rust
spawns at block bottom and physics resolves it. Failed spawn returns false
(click falls through) vs vanilla SUCCESS.

### 3.5 Tools useOn (axe/hoe/shovel) — 2 major gaps (B7-G07, B7-G08)

Vanilla maps extracted from the jar: **STRIPPABLES 23 entries** (oak/dark_oak/
pale_oak/acacia/cherry/birch/jungle/spruce/mangrove wood+log, warped/crimson
stem+hyphae, bamboo_block → stripped variants; axis preserved via
`RotatedPillarBlock.AXIS`); **TILLABLES 5 entries** (grass_block→farmland,
dirt_path→farmland, dirt→farmland, coarse_dirt→dirt — all `onlyIfAirAbove`
(face ≠ DOWN + air above); rooted_dirt→dirt + hanging_roots drop —
always-true predicate, NO air requirement); **FLATTENABLES 6 entries**
(grass_block, dirt, podzol, coarse_dirt, mycelium, rooted_dirt → dirt_path;
face ≠ DOWN + air above; sound SHOVEL_FLATTEN). Axe order: strip (AXE_STRIP)
→ WeatheringCopper.getPrevious (AXE_SCRAPE + level event 3005) → wax-off via
`HoneycombItem.WAX_OFF_BY_BLOCK` (AXE_WAX_OFF + level event 3004);
ITEM_USED_ON_BLOCK trigger, setBlock 11, BLOCK_CHANGE with new state,
hurtAndBreak(1). Axe-only `playerHasBlockingItemUseIntent` (main hand +
offhand has BLOCKS_ATTACKS + not secondary-use-active → PASS). Shovel
campfire branch: lit campfire → `CampfireBlock.dowse` + level event 1009 +
LIT=false, independent of the flatten branch (no air requirement).

**B7-G07 (major):** axe strip/scrape/wax-off, hoe till, shovel path-making
entirely non-functional. The port routes all of it through the 26.x
`minecraft:block_transformer` registry, which does not exist in 1.21.7: (a) no
`minecraft:block_transformer` component on any item in prepared `items.json`;
(b) no `data/minecraft/block_transformer/` JSONs in prepared data; (c)
`ItemInfo.block_transformer` comes ONLY from the component (`items.rs:942`),
no synthesis anywhere; (d) `block_transform` returns None without it,
`use_block_transformer` always returns false; (e) no fallback strip/till/path
code exists in the game. The 1.21.7 prepare step never synthesized the
registry from the jar's hardcoded maps.

**B7-G08 (major):** shovel campfire douse dead. Rust routes it through the
26.x `#minecraft:douses_campfires` item tag, which does NOT exist in the
1.21.7 jar (0 files) or prepared item tags — so the filter always fails.
1.21.7's `CampfireBlock.useItemOn` only places food; dousing lives only in
`ShovelItem.useOn`'s campfire branch, which has no implementation.

Latent note for when transformer data lands: the shield blocking-intent check
in `block_transform` applies to hoe/shovel too, but vanilla's
`playerHasBlockingItemUseIntent` is axe-only. Also the transformer data must
encode hoe's rooted-dirt always-true predicate vs onlyIfAirAbove for the
rest, and shovel's face≠DOWN + air-above for flatten but not for campfire.
Not filed separately (unobservable while the transformer is dead).

### 3.6 Food / consumable lifecycle — pass except B7-G09 (major)

Verified matching: `FoodData.eat` (food clamp 0–20; saturation clamp 0..food
level), eat sound (consumable.sound, NEUTRAL, vol 1, pitch `triangle(1.0,
0.4)`), burp (PLAYER_BURP, PLAYERS, vol 0.5, pitch `randomBetween(0.9, 1.0)`,
players only), `canEat` (invulnerable || canAlwaysEat || needsFood — Rust's
`|| self.creative` matches since creative sets invulnerable), consume default
1.6s, ticks = `(seconds*20)` truncated, `should_emit` (elapsed > ticks×0.21875
&& remaining%4==0), use remainder (held when emptied, else inventory/drop;
creative skips), `clear_all_effects` (milk bucket) and `remove_effects`
(honey bottle, poison) parsed + applied, `ApplyEffects` (random drawn even at
probability 1), suspicious stew effects (amplifier 0), teleport effect
dispatch (diameter default 16.0, 16 attempts, target = pos + (rand−0.5)×diameter
per axis), chorus_fruit data correct (teleport_randomly effect, use_cooldown
1.0s, always-eat).

**B7-G09 (major):** chorus fruit never teleports. Rust `Game::random_teleport`
walks down looking for a block in `#minecraft:entities_can_teleport_to` — a
26.x tag that does NOT exist in the 1.21.7 jar (0 files) and is NOT in
prepared `block-tags.json` — so the loop always runs to min_y and returns
false; all 16 attempts fail. Vanilla `LivingEntity.randomTeleport` grounds on
`BlockState.blocksMotion()` (no tag), then checks `noCollision` +
`!containsAnyLiquid`. The avoid tag
`minecraft:consumable_does_not_teleport_to` is likewise absent in 1.21.7
(dead no-op check). **Shared root cause, flagged for Batch 8 / commands:**
the same tag is used by endermen (`mobs/enderman.rs:117`) and
`/spreadplayers`-style code (`game/commands/world_cmds.rs:2142`) — Batch 7
owns only the chorus-fruit instance.

### 3.7 Shield blocking — pass

Verified matching: jar shield component = `BlocksAttacks(0.25s delay, 1.0
disable scale, [DamageReduction(90°, no type filter, base 0.0, factor 1.0)],
ItemDamageFunction(3.0, 1.0, 1.0), bypassed_by #bypasses_shield, SHIELD_BLOCK,
SHIELD_BREAK)`; `items.json` omits damage_reductions/disable_cooldown_scale
but the Rust loader defaults are identical (90°/factor 1.0; scale 1.0).
Reduction resolve (angle > radians(horizontal_angle) → 0; type match;
clamp(base + factor×damage, damage); sum clamped to damage), item damage
(below threshold → 0, else floor(base + factor×blocked)), delay ticks
round(0.25×20)=5, disable ticks round(seconds×scale×20), axe disable 5.0s via
weapon component, Warden flat 5s, Ravager stun/knockback, blockedByItem
knockback 0.5 when not fully blocked, block sound pitch 0.8+rand×0.4, break
handling (onEquippedItemBroken).

### 3.8 Bow — pass

Verified matching: `getPowerForTime` = `(f² + 2f)/3` clamped to 1.0 (jar
bytecode), 0.1 minimum to fire; `use` gate (FAIL without projectile unless
creative); arrow selection (`Player.getProjectile`); shootFromRotation
(power×3, inaccuracy 1.0); base damage 2.0 + Power bonus via the enchantment
pipeline; critical iff power == 1.0; spectral arrow → glowing 200 ticks;
infinity/creative → CREATIVE_ONLY pickup; Flame ignites the arrow; sound
ARROW_SHOOT at the feet, vol 1, pitch `1/(rand×0.4+1.2) + power×0.5`;
ITEM_USED stat; ammo consumption (creative/infinity skip); bow damage 1.

### 3.9 Crossbow — pass

Verified matching: charge duration `(1.25 − 0.25×QuickCharge) × 20` floored
(Quick Charge data jar-verified: −0.25/level on `crossbow_charge_time`);
loading sounds at charge 0.2/0.5 (start/middle; Quick Charge has no middle,
start sounds quick_charge_1/2/3, end loading_end — all jar-verified);
loading_end pitch `1/(rand×0.5+1.0) + 0.2`; load at charge 1.0; Multishot
loads 3 (1 arrow consumed, extras intangible); rocket power 1.6 / 3
durability, arrow power 3.15 / 1 durability; arrows always critical, base
damage 2.0; Multishot fan ±10°; shot pitches (index 0 → 1.0; odd index →
`1/(rand×0.5+1.8) + 0.63`, even → `+ 0.43` — jar-verified
`getShotPitch`/`getRandomShotPitch`); Piercing via enchantment modify of
`projectile_piercing`; Flame ignites all arrows; SHOT_CROSSBOW trigger +
ITEM_USED stat; `use` shoots when charged, else starts charging when ammo
exists.

### 3.10 Trident — pass

Verified matching: `use` fails if `nextDamageWillBreak` or (riptide and not
in water/rain); release requires ≥10 ticks; riptide requires wet + not riding;
riptide strength `1.5 + 0.75×(level−1)` (jar-verified from
`data/minecraft/enchantment/riptide.json`); spin attack 20 ticks / 8.0 damage;
lift 1.1999999 when on ground; throw velocity 2.5, inaccuracy 1.0;
`hurtWithoutBreaking(1)` for both throw and riptide; creative → CREATIVE_ONLY
pickup; thrown damage 8 + Impaling; loyalty pull (y += v.y×0.015×loyalty;
accel 0.05×loyalty; velocity ×0.95); water inertia 0.99, gravity 0.05, shake
7 ticks; channeling (thunderstorm + canSeeSky + lightning on hit); TRIDENT_SOUND
pick-highest-level with TRIDENT_THROW fallback; ITEM_USED stat. Note:
`in_water_or_rain` checks the feet block only where vanilla checks the
bounding box in water — a minor approximation, not filed.

### 3.11 Potions — pass except B7-G10 (major) and B7-G11 (minor)

**Drink — pass.** Consumable data present in `items.json` (drink, no
particles, sound drink; use_remainder glass_bottle); 32-tick duration; drink
sound every 4 ticks when remaining ≤ 25 (pitch 0.9+rand×0.1, vol 0.5);
contents effects at `potion_duration_scale`; ITEM_USED stat + consume_item
trigger + drink game event; glass-bottle remainder with
`createFilledResult` semantics. `potion_duration_scale` fallbacks verified:
tipped_arrow 0.125, lingering_potion 0.25, else 1.0.

**Splash effects — pass.** Distance via potion AABB (0.25³) inflated (4, 2,
4), margin `clamp((age−2)/20, 0, 0.3)`, d² < 16, intensity `1 − √d²/4`;
duration `(intensity × duration + 0.5)` truncated, applied iff > 20 ticks;
instant harm `(intensity × (6 << amp) + 0.5)`, heal `(intensity × (4 << amp)
+ 0.5)`, undead inversion; enderman water hurt → teleport counter; level
events 2002/2007 + sounds 1053/1054; axolotl rehydrate +1800 air (direct
water check). Observation (not filed): the Rust splash path does not apply
`POTION_DURATION_SCALE` — vanilla does, but the default is 1.0 for splash
potions, so only commands-modified scales differ.

**Lingering cloud — pass.** Radius 3.0, radiusOnUse −0.5, duration 600,
waitTime 10, radiusPerTick −3/600, duration scale 0.25; cloud at the mob hit
(feet) else the impact; no cloud for effect-less potions.

**Throw — pass.** Splash/lingering: −20 pitch offset, 0.5 velocity, 1.0
inaccuracy (triangle jitter 0.0172275), eye − 0.1, sound vol 0.5 pitch
`0.4/(rand×0.4+0.8)`, stat, consume. Exp bottle: 0.7 velocity, 3 + rand(5) +
rand(5) XP, level events 2002/1053. Glass bottle: dragon-breath collection
first (radius shrink 0.5, fill sound, FLUID_PICKUP, PLAYER_INTERACTED_WITH_ENTITY),
then water fill (sound at player, FLUID_PICKUP at the water block, water
potion via `with_potion`). Observation (not filed): the throw sound plays at
the eye where vanilla plays at the feet (same class as the #8448 bow fix).

**B7-G10 (major):** water splash/lingering bottles do nothing on impact
except axolotl rehydrate. Vanilla `AbstractThrownPotion.onHit`/`onHitBlock`
gate `onHitAsWater`/`dowseFire` on `PotionContents.is(Potions.WATER)`
directly (javap-verified). The Rust gates all three behaviours on potion
tags — `minecraft:douses_fire` (fire blocks, candles, campfires at the hit
cell, the cell in front, and 4 horizontal neighbours), `minecraft:hurts_water_sensitive_entities`
(1.0 indirect magic), `minecraft:extinguishes_entities` (burning entities) —
read from `tags/potion`. But the 1.21.7 jar has **no `PotionTags` class and no
`data/minecraft/tags/potion/`**, and prepared data has no potion tag
directory, so `potion_in_tag` always returns false. The douse positions, hurt
amount and extinguish logic are otherwise exactly right — only the gate is
dead.

**B7-G11 (minor):** water bottle on dirt never makes mud. The real 1.21.7
tag is `minecraft:convertable_to_mud` (Mojang's spelling, with an A) —
`data/minecraft/tags/block/convertable_to_mud.json` exists (dirt, coarse_dirt,
rooted_dirt) and prepared `block-tags.json` carries the same key. The Rust
looks up `minecraft:convertible_to_mud` (with an I) in three places
(`potions.rs` use, `dispenser.rs`, `remote.rs` prediction) plus a `data.rs`
enum entry, so the lookup always misses. The rest of `use_water_bottle_on_block`
matches vanilla `PotionItem.useOn` (face ≠ DOWN, mud + GENERIC_SPLASH + 5
splash particles + bottle-empty sound + FLUID_PLACE + glass-bottle remainder
+ stat) — it just never runs.

### 3.12 Boats — pass except B7-G12 (minor)

Verified matching: POV raycast, OUTLINE block mode, Fluid.ANY; boat spawned
at the exact hit location with the player's yaw; noCollision check (block
unobstructed + entity collision); ENTITY_PLACE at the hit (= boat position);
consume + ITEM_USED stat; chest boats via `with_chest()`.

**B7-G12 (minor):** vanilla `BoatItem.use` returns PASS when the player's
eye is inside a pickable entity's inflated bounding box (entities in the
player bbox expanded 5.0 along the view + inflate 1.0). The Rust's
`boat_to_place` only checks the boat's own bbox. Edge case: standing inside
a mob while placing a boat — the Rust places where vanilla passes.

### 3.13 Armour damage reduction — pass

Verified matching: `after_armor` = divisor `2 + toughness/4`, protection =
`clamp(armor − damage/divisor, armor×0.2, 20)`, damage × (1 −
protection/25), with the `armor_effectiveness` enchantment modify (Breach)
clamped 0..1 — exactly `CombatRules.getDamageAfterAbsorb`. Order armor →
resistance → protection matches vanilla. Resistance: reduction =
`(amp+1)×5`, damage × `(25 − reduction)/25` — exactly
`LivingEntity.getDamageAfterMagicAbsorb`. Protection: `clamp(0..20)`, damage
× (1 − p/25). Armour is damaged by the pre-reduction amount, as vanilla.
Armor value floors the effective double before converting to float;
toughness converts directly.

### 3.14 Mace — pass

Verified matching: smash threshold fall > 1.5 and not fall-flying; damage
bonus tiers (≤3: 4×fall; ≤8: 12 + 2×(fall−3); else 22 + fall−8) plus Density
via `smash_damage_per_fallen_block` × fall; knockback power
`(3.5 − length) × 0.7 × (fall > 5 ? 2 : 1) × (1 − resistance)`, push x/z from
the normalized vector × power, y += 0.7 flat (jar-verified
`getKnockbackPower` + `lambda$knockback$0`); radius 3.5; fall stop
(velocity.y = 0.01), fall-damage waived from the impact position, extra
fall particles when the target is on the ground; sounds (ground heavy if
fall > 5 else ground; air if the target is airborne) at the player's feet;
`postHurtEnemy` resets fall distance; `mace_smash` damage source; Wind Burst
via `post_attack`.

### 3.15 Ender pearl — pass

Verified matching: throw velocity 1.5, inaccuracy 1.0, sound vol 0.5 pitch
`0.4/(rand×0.4+0.8)`, stat, consume, 1.0s `use_cooldown`; hit deals 0 damage
to entities; teleport to the pearl's **oldPosition** (jar-verified —
`ThrownEnderpearl.onHit` builds the TeleportTransition from `oldPosition()`);
5.0 damage; 5% endermite at the owner's position when monsters spawn; fall
distance + impulse reset; owner-alive/not-sleeping gate
(`isAllowedToTeleportOwner`); teleport sound once at the destination
(jar-verified — the two `playSound` calls in `onHit` are the player-owner and
non-player-owner branches, not two positions). Observation (not filed):
vanilla spawns the endermite in peaceful too (it despawns the next tick);
the Rust skips it — unobservable. The pearl's portal-cooldown transfer is not
implemented (pearl-through-portal edge, not filed).

### 3.16 Fishing rod — pass

Verified matching: `use` — reel-in (retrieve damage, FISHING_BOBBER_RETRIEVE
vol 1, ITEM_INTERACT_FINISH) vs cast (FISHING_BOBBER_THROW vol 0.5, stat on
cast only, ITEM_INTERACT_START); both sounds pitch `0.4/(rand×0.4+0.8)` at
the feet; Lure = level × 5 × 20 ticks, Luck of the Sea = level; hook start
(player pos − sin/cos × 0.3, eye height); cast velocity `v × (0.6/len +
triangle(0.5, 0.0103365))` per component with three independent draws
(jar-verified — the 0.5 centre is vanilla's, not a bug); retrieve damage
(hooked entity 5, item entity 3, bite 1, on ground 2 — ground overrides);
pull 0.1 toward owner; loot luck = rod luck + owner luck attribute; XP 1 +
rand(6); fish_caught stat for `#fishes`; item spawn velocity (dx×0.1,
dy×0.1 + √dist×0.08, dz×0.1); FISHING_ROD_HOOKED trigger; wait 100–600
ticks minus Lure; hooked 20–80 ticks; nibble 20–40 ticks; bite impulse
`−0.4 × nextFloat(0.6, 1.0)`; tease tiers (0.15 + 0.05/0.02/0.01 within
20/40/60 ticks); rain/sky speed (+1 on 25% rain roll, −1 on 50% no-sky
roll); bite splash sound vol 0.25 pitch 1±0.4; open water 5×4×5 (x/z ±2,
y −1..+2, layer-type state machine). Observation (not filed): `canSeeSky`
is approximated by sky-light 15 — the two agree in the common cases (glass,
leaves) and differ only under unusual transparent cover.

## 4. Defect list

| ID | Severity | Title |
|----|----------|-------|
| B7-G01 | minor | ComponentMap.set returns an error where vanilla returns null (dangling removal marker) |
| B7-G02 | minor | Bucket sneak-redirect: the port pours beside the clicked block while sneaking; vanilla 1.21.7 has no sneak check |
| B7-G03 | minor | Spawn egg entity_data id type override ignored |
| B7-G04 | minor | Hostile spawn eggs refused in peaceful instead of spawn-then-despawn + consume |
| B7-G05 | minor | Spawn-egg useOn posts ENTITY_PLACE at the spawn position instead of the clicked block |
| B7-G06 | minor | No Mob.playAmbientSound() on spawn-egg or mob-bucket placement |
| B7-G07 | major | Axe strip/scrape/wax-off, hoe till, shovel path-making entirely non-functional (26.x block_transformer registry, absent in 1.21.7) |
| B7-G08 | major | Shovel campfire douse dead (#douses_campfires tag absent in 1.21.7) |
| B7-G09 | major | Chorus fruit never teleports (#entities_can_teleport_to tag absent in 1.21.7) |
| B7-G10 | major | Water splash/lingering bottles do nothing on impact except axolotl rehydrate (potion tags absent in 1.21.7) |
| B7-G11 | minor | Water bottle on dirt never makes mud (tag spelled convertible_to_mud; 1.21.7 has convertable_to_mud) |
| B7-G12 | minor | Boat placement missing the eye-inside-entity PASS check |

**Systemic note.** Four of the twelve defects (B7-G07, B7-G08, B7-G09,
B7-G10) share the pilot's root cause: the 1.21.7 port carries 26.x mechanisms
(a data registry, item/block tags, potion tags) for behaviours that 1.21.7
implements in hardcoded Java, and the prepare step never synthesized the
26.x-shaped data from the jar — so every such lookup silently degrades to
empty/false and the whole behaviour dies. B7-G11 is the same family one step
removed: a tag name whose spelling changed between 1.21.7 and 26.x
(convertable → convertible). The prepared data is faithful to the jar; the
port code (and the missing synthesis step) is the 26.x carry-over.

Two numbering notes: the provisional G03/G04 slots from working notes were
closed at write time (one was a suspicion resolved as matching, one a
retracted claim about the spawn-egg stat — see §3.4). The pearl sound was
also investigated as a candidate gap and retracted after the jar showed
vanilla plays the teleport sound once, at the destination.

## 5. Driver inventory — 74 drivers, none run

Driver inventory is DONE (saved to `b7-analysis/b7-map.json`,
`b7-map.txt`, `b7-blocked-oracles.json`); **no driver was run**, because the
execution-risk notice blocked all repo execution. The 14 "runnable" drivers
are recorded as **not run — blocked**, not as passes.

- **14 runnable** (oracles pass on their classpath; NOT run): food,
  food_properties, food_storage, consume_effect, stored_consume,
  blocking_rules, damage_reduction, combat_rules, component_patch,
  enchantment_components, stack_foil, map_saved_data, hotbar, inventory_tick.
- **42 blocked** by 43 distinct failing oracles. Small in-family repair
  candidates (1–8 javac errors): VanillaItemUseOracle (1),
  VanillaItemDurabilityOracle (1), VanillaStackMiningOracle (1),
  VanillaCreativeMiningOracle (1), VanillaPlayerEquipmentOracle (1),
  VanillaStoredEnchantmentPatchOracle (1), VanillaItemStackStorageOracle (1),
  VanillaItemCooldownsOracle (1), VanillaItemUseLifecycleOracle (2),
  VanillaComponentDurabilityOracle (2), VanillaBlockingItemDamageOracle (2),
  VanillaInventoryPickupOracle (2), VanillaPlayerInventoryOracle (2),
  VanillaPlayerFoodOracle (3), VanillaConsumableEmissionOracle (3),
  VanillaUseFlagReactionOracle (3), VanillaInventoryUseOracle (3),
  VanillaConsumableOracle (4), VanillaItemWireOracle (4),
  VanillaEnchantingOracle (4), VanillaExplosionItemsOracle (5),
  VanillaVehicleContainerOracle (5), VanillaLavaCauldronItemOracle (5),
  VanillaItemUseStartOracle (6), VanillaItemAfterUseOracle (6),
  VanillaItemBlockingOracle (6), VanillaInventoryUseStartOracle (7),
  VanillaTradeComponentsOracle (7), VanillaBundleWeightOracle (7),
  VanillaItemUseCatalogOracle (8), KnockbackClient (8),
  VanillaUseRemainderOracle (8), VanillaItemTemplateOracle (8),
  VanillaExtraItemsOracle (8). Large: VanillaHeldItemStackOracle (18),
  VanillaTeleportConsumeOracle (21), VanillaArrowOracle (22),
  VanillaHeldDropOracle (40), VanillaDispenserOracle (54),
  VanillaInteractionInputOracle (68), VanillaBlockUpdateOracle (69);
  cascades: VanillaEnchantDurabilityOracle, VanillaHelmetDamageOracle (both
  via BlockUpdate).
- **17 no-oracle** (mostly live-game e2e drivers needing a built game +
  fixture servers; NOT run): interaction_data, inline_fishing_tool,
  trident_save, pumpkin_equippable_use, builtin_component_delivery,
  effective_component_presence, effective_item_components,
  numeric_damage_component, item_contents, item_numeric_uuid,
  item_health_load, item_entity_max_stack, spawn_egg_fluid, boat_item_hit,
  map_decoration_patch, map_decoration_namespace, enchantment_map_validation.
  Flags: `effective_item_components` and `pick_block_data` docstrings say
  "(Java 26.3)" — possible 26.3-only drivers.
- **1 indirect**: pick_block_data (references VanillaPickBlockDataOracle, 1
  error, no direct javac path).

**Oracle defects found: none new.** The 43 blocking oracles are Batch 0's
known-blocked list, not new findings. **Oracle repairs done: none** (blocked).

## 6. Method notes

- Every vanilla claim is cited to a specific jar method + bytecode, read with
  `javap -c -p` on the named jar. No behaviour was assumed from the 26.3 base
  or from memory: three candidate gaps were retracted after jar checks (the
  spawn-egg stat, the crossbow shot-pitch parity, the pearl teleport sound —
  the jar proved the Rust right in each case), and one suspicion (bucket
  adventure-mode check) resolved as matching.
- Rust item data is data-driven from `items.json` at runtime (`ItemInfo` in
  `horizon-items/src/items.rs`); data parity therefore hinges on prepared
  `items.json` fidelity (verified in §2) and loader/behaviour-code
  interpretation (verified per family in §3).
- Item-use dispatch lives in `horizon-client/src/game/` (`use_hand.rs`,
  `item_actions.rs`, `consume.rs`, `using_item.rs`, `potions.rs`,
  `shield.rs`, `crossbow.rs`, `trident.rs`, `fishing.rs`, `melee.rs`,
  `combat.rs`, `mace.rs`, `boats.rs`, `buckets.rs`, `splash_water.rs`);
  `horizon-items/src/` has `items.rs`, `item_data.rs`, `bundle.rs`,
  `cooldowns.rs`, `enchant.rs`.
- Check counting: one check per behaviour pattern, grouped by behaviour class
  per the brief (all tools of one kind are one check pattern, not N checks).
  The 40 checks are D1–D8 (§2) and B1–B32 (§3). A check is "gap" if it
  contains at least one defect; B7-G07 spans three gap checks (axe/hoe/shovel).
- Analysis artefacts (scripts, jar dumps, check outputs) live in
  `~/workspace/mcir-work/b7-analysis/`; the driver map and blocked-oracle
  list are there too. The deliverable drafts are in
  `~/workspace/mcir-work/sweep-batch-7-deliverables/`.

## 7. What the execution-risk block defers

- Running the 14 runnable drivers (food, food_properties, food_storage,
  consume_effect, stored_consume, blocking_rules, damage_reduction,
  combat_rules, component_patch, enchantment_components, stack_foil,
  map_saved_data, hotbar, inventory_tick) — the food driver in particular
  was called out in the brief (Batch 0 repaired its oracle).
- Running the 17 no-oracle drivers (live-game e2e; need a built game +
  fixture servers).
- Repairing any of the 43 blocking oracles (34 are small, 1–8 javac errors)
  and end-to-end driver runs to confirm each repair yields a measurement.
- Committing these deliverables to `parity-sweep` (docs/sweep/batch-7-items.md,
  pins/1.21.7/sweep/batch-07.json) and pushing.

Nothing in this batch touched production code (`crates/`), per the
checks-only brief. Fixes for the 12 defects are a separate decision on
separate branches.
