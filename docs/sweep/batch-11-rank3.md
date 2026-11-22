# Batch 11 — Rank-3 template items (247), 1.21.7 parity sweep

- **Batch:** 11 (rank-3 gaps) — checks + triage only, no fixes.
- **Date:** 2026-11-22
- **Scope:** the 247 rank-3 items of the 1.21.11 template ledger (items
  503–749), triaged FOR 1.21.7 against the named 1.21.7 jars and the
  1.21.7 port in `versions/1.21.7/crates/`. Vanilla 1.21.7's own jar/data
  is the spec throughout; the 26.3 base's form was never trusted.
- **Ledger slice:** `pins/1.21.7/sweep/batch-11.json`.

## Tally (247 items)

| Verdict | Count |
|---|---|
| pass | 223 |
| gap (defect attached) | 22 |
| needs-new-harness | 1 |
| needs-in-game-probe | 1 |
| accepted gap (recorded, not an item verdict) | 4 notes (§5) |

Defects found: **23** (`B11-G01`–`B11-G23`), of which 3 major, 20 minor.
Oracle repairs: **1** (`B11-O01`, committed separately as c754e3a2).
The 23 defects include 6 from a systematic class-table sweep (§6.1) that
cut across items, and 3 cross-cutting defects (beds, fungus/roots,
speleothem) with no single owning rank-3 item — those are recorded as
defects, not item verdicts, so the defect count (23) exceeds the gap-item
count (22).

## §1 Defects

### B11-G01 — Beds place foot-only; sleep is unreachable (major)

`Class::AbstractBed = "AbstractBedBlock"` never matches any 1.21.7 chain:
1.21.7 has no `AbstractBedBlock` — beds are `BedBlock` directly (jar +
`block-behaviours.json` chain `[BedBlock, HorizontalDirectionalBlock,
Block]`). `Class::Bed = "BedBlock"` exists (classes.rs:60) but NOTHING
dispatches on it. Every bed arm is dead:

- `place.rs:356` (placement check), `place.rs:1481` `set_placed_by` — the
  ONLY place a `"head"` part is ever set (grep-confirmed) — so a
  player-placed bed is a foot-only single block.
- `interact.rs:365` — the ONLY emitter of `PlayerEffect::Sleep`
  (interact.rs:381) — so clicking ANY bed (including village beds) does
  nothing; `try_sleep` (game.rs:8642) never fires.
- `hooks.rs:459` (creative break), `shapes.rs:124` (neighbour update).

`BlockData::is` (data.rs:217) checks the full chain, so no fallback exists.
Severity major: sleep/spawn is core gameplay. **Template-level too:** the
1.21.11 server jar contains `BedBlock` but NOT `AbstractBedBlock`, so the
mapping is dead in the template as well — the E6 re-base cannot fix this by
propagation; it needs a template fix (or a 1.21.7-direct fix).

### B11-G02 — Redstone wire never updates power (major; cross-ref Batch 2)

`Class::RedstoneWire = "RedstoneWireBlock"` (lowercase s); 1.21.7 names the
class `RedStoneWireBlock` (capital S, jar-verified). Every wire arm is dead
with no property-based fallback: hooks.rs:100/211/380 (neighbour-changed,
notify, on-remove → `wire_update_power`), redstone.rs:65/101/219
(`wire_signal`), interact.rs:263, place.rs:1089. Wire power never
recomputes; the signal arms fall to the `_ => nibble` default. Batch 2
(redstone) owns the full measurement; this is the root cause. The template
FIXED this rename with a dual name — the fix is a re-base propagation
candidate. Also: `survival.rs:43` (horizon-prepare) maps
`("RedstoneWireBlock", "RedstoneWire")` — the lowercase-s name, never
matches 1.21.7's `RedStoneWireBlock`.

### B11-G03 — Farmland never dries/decays; dirt paths never revert (minor)

`Class::Farmland = "FarmlandBlock"`; 1.21.7 names it `FarmBlock`.
`Class::Path = "PathBlock"`; 1.21.7 names it `DirtPathBlock`. Dead arms:
tick.rs:197 (scheduled tick: can_survive → turn to base), tick.rs:350
(random tick → `farmland_random_tick`: moisture drying + decay to dirt),
place.rs:922/929, survive.rs:194/273, shapes.rs:465, tick.rs:202. Farmland
never dries, never decays, never reverts on support loss; dirt paths never
revert. PORT-NOTES already noted "farmland decay never runs" — confirmed
with exact arm locations. Template fixed both renames (dual names).

### B11-G04 — Snowy placement + grass/mycelium spread dead (minor)

`Class::Snowy = "SnowyBlock"`; 1.21.7 has `SnowyDirtBlock`.
`Class::SpreadingSnowy = "SpreadingSnowyBlock"`; 1.21.7 has
`SpreadingSnowyDirtBlock`. Dead arms: place.rs:1152 (snowy placement),
shapes.rs:721 (neighbour update), tick.rs:450 (`spreading_snowy_tick` —
grass/mycelium spread). Item 726. Template fixed both renames.

### B11-G05 — Nether fungi/roots bonemeal + survival fallback (minor)

`Class::NetherFungus = "NetherFungusBlock"`; 1.21.7 has `FungusBlock`.
`Class::NetherRoots = "NetherRootsBlock"`; 1.21.7 has `RootsBlock`. Dead
arms: bonemeal.rs:61 (fungus bonemeal kind), survive.rs:375 (falls back to
`supports_vegetation`). PORT-NOTES known. Template fixed both renames.

### B11-G06 — Dripstone placement/survival/growth dead; second cause of G6 (minor)

`Class::Speleothem = "SpeleothemBlock"`; 1.21.7 has `PointedDripstoneBlock`.
Dead arms: place.rs:1156, survive.rs:307, tick.rs:248 + tick.rs:434
(`speleothem_random_tick` — growth + cauldron fill), shapes.rs:728. This is
a SECOND independent cause of pilot G6 (dripstone never fills cauldrons)
alongside the absent `speleothems` tag. **Template-level too:** the 1.21.11
server jar contains `PointedDripstoneBlock` but NOT `SpeleothemBlock` — the
template did NOT fix this rename.

### B11-G07 — Bonemealing a blocked sapling keeps the meal (item 581, minor)

Vanilla 1.21.7 `SaplingBlock.isValidBonemealTarget` returns `true`
unconditionally (iconst_1; ireturn) — same in 1.21.11. Vanilla
`BoneMealItem.growCrop`: valid → success roll → `stack.shrink(1)`
UNCONDITIONALLY when valid (bytecode 81-83, outside the success branch).
Rust `bonemeal_valid` (Kind::Sapling, bonemeal.rs:331-340) returns FALSE
when the tree can't grow / no grower / out of build height, so `grow_crop`
returns false and the meal is KEPT. Divergence: bonemealing a blocked
sapling consumes the meal + shows particles in vanilla, does nothing in the
port. (The 0.45 success roll matches: bonemeal.rs:469.)

### B11-G08 — Riptide blocked while riding (item 541, minor)

Rust `release_trident` (trident.rs:500) blocks Riptide while riding
(`riptide > 0 && (!in_water_or_rain || riding.is_some())`). Vanilla 1.21.7
`TridentItem.releaseUsing` has NO passenger/vehicle check (offsets 46-62:
only `spinAttack > 0 && !isInWaterOrRain()`); 1.21.11 likewise. 26.3-leak.
Everything else matches (10-tick minimum, throw 2.5/1.0, auto-spin
20/8.0, lift 1.1999999, strength 1.5+0.75*(lvl-1)); the riptide sound is
verified correct via data (`riptide.json` defines the
`minecraft:trident_sound` list).

### B11-G09 — Failed lead tie creates no knot (item 572, minor)

Vanilla 1.21.7 `LeadItem.bindPlayerMobs`: for the FIRST leashable mob in
range, the knot is created FIRST (`LeashFenceKnotEntity.getOrCreateKnot` +
`playPlacementSound`, bytecode 52-63) and only THEN is
`canHaveALeashAttachedTo(knot)` checked — so a tie attempt where NO mob can
attach still creates a knot + placement sound (method returns PASS). Rust
`use_knot` (leads.rs:640-653) checks `can_attach_lead` first and calls
`tie_knot` only `if attached` — no knot, no sound on a failed tie.

### B11-G10 — Creeper defuses when its target dies (item 666, minor)

Vanilla 1.21.7 `SwellGoal` has NO alive/dead check: `canUse` =
`swellDir > 0 || (target != null && distSq < 9)`; `tick` = target==null →
-1; distSq > 49 or no LoS → -1; else +1. 1.21.11 likewise (no
isDeadOrDying). Rust `Swell` (hostile.rs:286-319) adds `t.alive` in
`can_use` AND in `tick`. Observable: in vanilla a creeper mid-fuse keeps
swelling and EXPLODES when its target dies; in the port it defuses (dir=-1)
as soon as the target dies. Present in neither 1.21.7 nor 1.21.11 vanilla.

### B11-G11 — Polar bear cubs never hunt foxes (item 622, minor)

Vanilla 1.21.7 fox goal: `NearestAttackableTargetGoal(this, Fox.class, 10,
true, true, null)` — selector NULL, no age gate (registerGoals offsets
194-214). 1.21.11 identical. Rust `register` (polar_bear.rs:51-56) adds
`.when(|m, _| !m.is_baby())` — cub bears never hunt foxes in the port; in
vanilla they do. (The `isBaby` lambda in registerGoals is the PanicGoal's
damage-tag function — correctly ported as PolarBearPanic.)

### B11-G12 — Three number providers return 0 (item 637, minor, datapack-only)

Vanilla 1.21.7 has 6 number providers: ConstantValue, UniformGenerator,
BinomialDistributionGenerator, EnchantmentLevelProvider, ScoreboardValue,
StorageValue. Both Rust implementations handle only 3
(`minecraft:constant`, `minecraft:uniform`, `minecraft:binomial`) —
`minecraft:enchantment_level`, `minecraft:score`, `minecraft:storage` fall
to `_ => 0` (horizon-world loot.rs `int`/`float` 1329-1370; horizon-items
items.rs `number` 1903-1936). Data check: 0/1251 vanilla 1.21.7 loot tables
use the 3 missing providers — vanilla gameplay unaffected; datapacks using
them get 0.

### B11-G13 — Splash water never douses fire, hurts, extinguishes, or rehydrates (item 699, major)

Vanilla 1.21.7 `AbstractThrownPotion.onHitBlock`: the gate is
`potionContents.is(Potions.WATER)` — direct holder equality, NO tag
(bytecode 55-63). (ThrownPotion does not exist in 1.21.7; the classes are
AbstractThrownPotion, ThrownSplashPotion, ThrownLingeringPotion.) Rust
`potion_in_tag` (splash_water.rs:42-51) reads the `minecraft:douses_fire`
TAG from `potion_tags` — but 1.21.7 has NO potion tags in data (0 files
under tags/potion; `PotionTags` class absent from jar) → the map is empty
→ splash water NEVER douses fire. Same dead gate hits 3 more callers:
`hurts_water_sensitive_entities` (134), `extinguishes_entities` (135),
`rehydrates_axolotls` (197). The douse AREA matches vanilla exactly; only
the gate diverges. Systemic cause = the pilot G-defects' (26.x tag logic
for absent tags).

### B11-G14 — Player data saved where vanilla 1.21.7 won't look (item 617, minor)

Vanilla 1.21.7 LevelResource paths are flat: `playerdata`, `advancements`,
`stats` (and `players` is the OLD data dir). The Rust's native layout is
`<world>/players/data/` (the 26.x form, lan/playerdata.rs:28-30). Mitigated
one way: the filefix migration (datafix/files.rs:287) moves
playerdata/advancements/stats → players/* when opening a vanilla world, so
vanilla→port works; but port-saved worlds put player data where vanilla
1.21.7 won't look (port→vanilla loses player state). Save-interchange
divergence, not gameplay.

### B11-G15 — Recipe-book fuel slot cycles fuels in the wrong order (item 662, minor)

Vanilla 1.21.7 `AnyFuel.resolve`: `fuelValues.fuelItems()` — the keySet of
an insertion-ordered LinkedOpenHashMap populated by the HARDCODED
`FuelValues.vanillaBurnTimes` sequence (LAVA_BUCKET, COAL_BLOCK, BLAZE_ROD,
COAL, CHARCOAL, #LOGS, #BAMBOO_BLOCKS, #PLANKS, ...). The Rust groups fuels
by burn-time value in registry order (recipe_book.rs:566-585,
native_panel_recipes.rs:173-190) — the 26.x DataComponentLookup behaviour
(DataComponentLookup itself is absent from the 1.21.7 jar). First-shown
fuel differs (vanilla: lava bucket). Burn TIMES unaffected — display order
only.

### B11-G16 — Golem stroll roll order + snow-layer tags (item 520, minor)

Two template fixes NOT in the 1.21.7 tree (E6 re-base held): (a) stroll
roll order — 1.21.7 vanilla stores the FIRST roll (fstore_2) and compares
the SECOND to 0.3f (the 1.21.11 order); the Rust draws 0.3 first, 0.7
second (golem.rs:152-155, the 26.1 order). Same distributions, different
per-seed picks. (b) Snow-layer tags — 1.21.7 data has ONLY
`snow_layer_can_survive_on` + `snow_layer_cannot_survive_on` (1.21.11
names); the Rust reads the 26.1 names (`cannot_support_snow_layer`,
`support_override_snow_layer`) in golem.rs:576-579,
block_survival.rs:529-530, survive.rs:295-297 → empty sets → snow layers
survive on ice/packed ice/barrier (should not) and die on honey/soul
sand/mud (should survive).

### B11-G17 — Mushroom bonemeal gate + light exemption (item 533, minor)

(a) Bonemeal: vanilla 1.21.7 `MushroomBlock.isValidBonemealTarget` returns
true unconditionally (iconst_1). Rust bonemeal_valid Kind::Mushroom
(bonemeal.rs:342-356) requires a configured huge-mushroom feature + build
height (26.1 form), and grow_crop gates on it (bonemeal.rs:317). Same class
as B11-G07. (b) Light: Rust survive.rs:266 + block_survival.rs:514 read
`overrides_mushroom_light_requirement` — absent from 1.21.7 data (only
`mushroom_grow_block` exists) → mushrooms never get the light exemption on
mycelium/podzol/nylium. (data.rs:476 HAS the mushroom_grow_block field —
survive.rs reads the wrong one.) SpikeFeature itself is a PASS (1.21.7's
SpikeFeature IS the end-spike placer, = 1.21.11 form).

### B11-G18 — Social screen shows row buttons when chat is disallowed (item 523, minor)

1.21.7 PlayerEntry ctor calls `getChatStatus().isChatAllowed(...)`
(offset 104) + `PlayerSocialManager.isBlocked` (offset 143) — the 1.21.11
gate form (no buttons unless chat allowed + not blocked). The 1.21.7 Rust's
social.rs has NO `chat_allowed` gate (template fix not propagated) → the
port draws hide/show/report buttons for every non-local row whatever the
chat setting.

### B11-G19 — Wandering trader spawns + save layout (item 535, minor)

1.21.7 = 1.21.11 form on BOTH halves: findSpawnPositionNear uses
Heightmap WORLD_SURFACE (offset 64), and PrimaryLevelData carries the
WanderingTraderSpawnDelay key (level.dat storage; WanderingTraderData
absent from the jar). The 1.21.7 Rust uses `motion_top`
(MOTION_BLOCKING_NO_LEAVES) in find_trader_spawn_near (:206) and writes its
own `wandering_trader.dat` (:48). Traders spawn at different spots under
trees; the trader clock is lost across vanilla↔port save interchange.

### B11-G20 — F3 lines: no day count, MiB labels, one-space Mood (item 571, minor)

1.21.7 vanilla's F3 lines are the 1.21.11 forms (string literals in
DebugScreenOverlay): `"Local Difficulty: %.2f // %.2f (Day %d)"` (offset
1606), `"  (Mood %d%%)"` TWO spaces (offset 1907), `"Mem: %2d%%
%03d/%03dMB"` (offset 68). The 1.21.7 Rust has all three 26.1 forms:
local_difficulty_line takes no day param (debug_overlay.rs:406, format has
no Day suffix), memory lines use "MiB" (:446, :533, :538, :540), sound mood
has ONE space before (Mood (:433). Cosmetic. (The DebugEntry system itself
doesn't exist in 1.21.7 — 0 classes — but the Rust's line CONTENT is what
matters, and it diverges on these three.)

### B11-G21 — Chorus plants/flowers can't survive on end stone (item 544, minor)

1.21.7 data has NEITHER `supports_chorus_plant` NOR `supports_chorus_flower`
tag files; both ChorusPlantBlock and ChorusFlowerBlock hardcode END_STONE
(the 1.21.11 form). The 1.21.7 Rust reads the missing tags:
survive.rs:132,137 (plant), survive.rs:102 (flower), block_survival.rs:534
(tag_states → empty set) → chorus plants and flowers can never connect to
or survive on end stone. Same systemic cause as the pilot G-defects; the
template fix (hardcode end stone) was never propagated to 1.21.7.

### B11-G22 — Dialog item tooltip missing on keyboard focus (item 615, minor, known)

1.21.7's ItemDisplayWidget shows its tooltip on `isHoveredOrFocused()`
(offset 104) — the 1.21.11 form. The Rust shows the tooltip on hover only;
dialog focus tracks text inputs only, so item parts are never focusable.
Same known gap as the template (recorded in its PORT-NOTES as a
dialog-focus feature, out of rank-3 scope).

### B11-G23 — Washing dyed leather armour never works (items 558/683/735, minor)

Vanilla 1.21.7's CauldronInteraction table has explicit wash entries for
leather armour (LEATHER_BOOTS/CHESTPLATE/HELMET/LEGGINGS,
LEATHER_HORSE_ARMOR) plus banners and shulker boxes (jar constants). The
Rust handles banners (pop top pattern) and shulker boxes (transmute to
plain) with explicit arms — but dyed leather goes ONLY through the
`removes_dye` gate (cauldron.rs:214), which reads the item tag
`minecraft:cauldron_can_remove_dye` (cauldron.rs:7-12, citing
`ItemTags.CAULDRON_CAN_REMOVE_DYE`) — a tag file that exists in NEITHER
the 1.21.7 NOR the 1.21.11 jar data. The gate is always false, so washing
dyed leather armour/horse armour does nothing in the port. Oracle evidence
(driver case 5835, dyed leather helmet on a water cauldron level 1,
survival): vanilla returns `success`, the cauldron empties, and the helmet
loses its `dyed_color` component; the port passes with no change. Same
systemic cause as the pilot G-defects (26.x tag-driven dispatch — the
comment cites "Dispatcher.get: tags first, then the item map" — for a tag
that doesn't exist in 1.21.7). The rest of the table was verified by
inspection (buckets/bottles/potions/banners/shulker boxes all have explicit
arms matching vanilla); the exhaustive driver comparison is still owed as
confirmation (B11-H02).

## §2 Oracle repair

### B11-O01 — VanillaCauldronOracle drop signature (fixed, c754e3a2)

`VanillaCauldronOracle.FakePlayer.drop` used the 26.x 3-arg
`drop(ItemStack, boolean, Prediction)`; 1.21.7 has `drop(ItemStack,
boolean)` and NO `net.minecraft.util.Prediction` class (javap-verified).
Repaired to the 2-arg override (test-only Java, 1 file, 1 error fixed).
Oracle now COMPILES and RAN: 20,475 cauldron cases produced native.txt
(20,475 lines) under `test_cauldron_parity.py`. `VanillaLavaCauldronItemOracle`
still fails (missing-package gamerules, 5 errors) — NOT repaired; out of
scope for the cauldron driver (test_cauldron_parity.py only compiles
ParityData + VanillaCauldronOracle).

## §3 Needs-new-harness

### B11-H01 — Shipwreck re-adjustment (item 640)

Vanilla 1.21.7 `ShipwreckPieces$ShipwreckPiece.postProcess` (class is in
`.../structures/`, NOT `.../pieces/`): heightmap WORLD_SURFACE_WG (beached)
/ OCEAN_FLOOR_WG; mean height over footprint (single sample if area 0), min
tracked; `adjustPositionHeight(beached ? calculateBeachedPosition(min,
random) : mean)` then super.postProcess. `calculateBeachedPosition` =
`min - sizeY/2 - random.nextInt(3)` — the Rust (simple.rs:355-392) matches
exactly. NO heightAdjusted field in 1.21.7; `adjustPositionHeight` sets
templatePosition.y ABSOLUTELY, so every postProcess re-adjusts (fresh
nextInt(3) for beached). The Rust CACHES the first computed height (`state:
Some(y)` — the 26.x heightAdjusted behaviour). Whether vanilla's
re-adjustment is observable (multi-chunk beached wreck shear) depends on
per-chunk random seeding.

**Harness design:** a worldgen comparison harness — same seed, a beached
shipwreck straddling a chunk border, vanilla 1.21.7 region files (from
vanilla-world-mc or a fresh vanilla run) vs the port's generated region.
PASS = the wreck's blocks match exactly across the border (no shear step).
If the harness shows shear in vanilla but not the port (or vice versa), the
caching is a real divergence; if both agree, the caching is unobservable
and the item closes as a pass.

### Cauldron driver (items 558, 683, 735) — wash gap recorded; full comparison owed

The oracle side is complete (20,475 cases, native.txt written) and the
wash gap is already recorded as B11-G23 with oracle evidence (case 5835).
The exhaustive Rust-fixture comparison (`cauldron_fixture` via
client_probe) was still building at slice time after a disk-quota failure
(first run) and a service-restart SIGTERM (second run, ~25 min). The
resume path persists: `~/workspace/mcir-work/sweep-batch-11-scratch/
resume-cauldron.py` reuses the completed oracle output
(`.local/cauldron-parity/cases.jsonl` + `native.txt`) and runs only the
fixture + compare. The comparison is confirmatory for B11-G23 and may find
further table differences; update this section when it lands.

## §4 Needs-in-game-probe

### Item 519 — F3 chart footer layout

The Rust's `debug_chart_lines` returns `[String; 3]` with `"Debug charts:
{}; {};"` — the 26.1 THREE-line form. 1.21.11 inlines ONE concat (all three
charts on one line). 1.21.7's form could not be settled from the jar
cheaply: the DebugEntry system doesn't exist in 1.21.7 at all (0 classes),
and the footer literal isn't a single string constant in
DebugScreenOverlay. **Probe (owner-initiated only):** open F3 in vanilla
1.21.7 and in the port with the profiler/FPS/network charts toggled, and
compare the "Debug charts:" footer lines (one line vs three, trailing `;`,
lightmap chart presence). Do NOT launch anything from here.

## §5 Accepted gaps / notes (not defects)

- **597 ProvidesTrimMaterial inline form:** vanilla 1.21.7's
  `ProvidesTrimMaterial.material` is `EitherHolder<TrimMaterial>` (Holder
  direct OR ResourceKey string). The Rust reads the string form only
  (enchant.rs:335). All 11 vanilla items with the component use string
  keys — the direct/inline form is datapack-only. Accepted limitation.
- **682 NBT sound_variant on non-wolf mobs:** the Rust's NBT writer emits
  `"sound_variant": "minecraft:classic"` on cows/pigs/chickens/cats
  (entity_nbt.rs:872-880) — a tag vanilla 1.21.7 never writes for them
  (only wolves have sound variants in 1.21.7 data). Ignored on load;
  harmless. The SOUNDS are correct (VariantRegistry::random returns 0 for
  an empty registry → "classic" → plain entity sounds).
- **534 special-foil decal:** the special-foil decal (compasses/clocks,
  0.5x in GUI / 0.75x first person) is not modelled — the GUI sink draws
  the glint over the full quad. Only visible on a command-enchanted
  compass/clock. Pre-existing in both versions, not a version delta.
- **512 separator + per-player resolution:** the Rust ignores the
  `separator` field of `nbt` components (always ", ") and resolves a
  title/tellraw component ONCE for all players (vanilla resolves PER
  PLAYER). Gaps from BOTH versions equally, not deltas.

## §6 Systemic findings

### §6.1 The class table: 16 dead mappings, 9 live-arm renames

Method: parse every `Class::X = "Name" | "AltName"` in
`versions/1.21.7/crates/horizon-world/src/level/classes.rs` (271 mappings);
LIVE iff ANY alternative appears in ≥1 chain of the 1.21.7 export
`block-behaviours.json` (1105 blocks). Cross-checked against the jar (every
dead name also absent from the jar's block classes).

**Group A — RENAMES: 1.21.7 content exists under another name; live arms
dead (9):** Farmland→FarmBlock, Snowy→SnowyDirtBlock,
SpreadingSnowy→SpreadingSnowyDirtBlock, NetherFungus→FungusBlock,
NetherRoots→RootsBlock, Path→DirtPathBlock, RedstoneWire→RedStoneWireBlock,
Speleothem→PointedDripstoneBlock, AbstractBed→(no such class; BedBlock
directly). Defects B11-G01–B11-G06. The template FIXED 7 of the 9 with dual
names (Farmland, Snowy, SpreadingSnowy, NetherFungus, NetherRoots, Path,
RedstoneWire; LilyPad = "WaterlilyBlock" — the only dual-name mapping in
1.21.7's table, fixed in 6e6cb55b). The template did NOT fix Speleothem or
AbstractBed — both are dead in the template too (verified against the
1.21.11 server jar).

**Group B — NO 1.21.7 CONTENT: dead but harmless (7):** CopperChestBlock
(1.21.9+), CopperGolemStatueBlock (1.21.9+), PotentSulfurBlock +
SulfurSpikeBlock (26.2 sulfur), ShelfBlock + ShelfMushroomBlock (1.21.9+
shelves), StrawBedBlock (26.x). No 1.21.7 block can match; arms
unreachable. Hygiene note, not gaps.

This extends the MEMORY.md PORT-NOTES finding ("28 class names in the
1.21.7 export don't map to the port's class table") with exact arm
locations and the template comparison.

### §6.2 The 26.x tag-name pattern keeps recurring

B11-G13 (potion tags), B11-G16b (snow-layer tags), B11-G17b (mushroom tag),
B11-G21 (chorus tags), B11-G23 (cauldron wash tag), B11-G06 (speleothems tag,
second cause) — six more instances of the pilot's systemic cause: the port reads 26.x tag names for
tags that don't exist in 1.21.7 data, and every such lookup silently
degrades to empty/false. The leakage-audit tool (tools/audit_26_3_leakage.py)
is the standing guard; these six are concrete instances it should already
flag (verify: the tag names appear in the audit's baseline).

### §6.3 Template fixes NOT in the 1.21.7 tree (re-base input)

The following template fixes apply to 1.21.7 (jar-verified) but are NOT in
the 1.21.7 tree — the E6 re-base propagation (held until 1.21.7 is
verified) should carry them: the 7 class-table dual names (B11-G02–G05),
the snow-layer tag renames (B11-G16b), the mushroom tag + bonemeal
(B11-G17), the chorus end-stone hardcode (B11-G21), the social gate
(B11-G18), the trader heightmap + storage (B11-G19), the F3 line formats
(B11-G20), the golem roll order (B11-G16a).

### §6.4 Re-base WARNING: item 567 must NOT propagate as-is

The template REMOVED the 1e-5 fluid-current floor for 1.21.11 ("the 26.x
tracker's 1e-5 squared-length floor is gone"). But 1.21.7 vanilla HAS the
floor — the constant 9.999999747378752E-6d appears 4x in 1.21.7's Entity
class (inline in updateFluidHeightAndDoFluidPushing) — and the 1.21.7 Rust
has it too (entity_fluid.rs:87). Propagating the template's floor removal
to 1.21.7 would INTRODUCE a divergence. This unit needs a per-version
check in the re-base, not a blind propagation.

### §6.5 Class-absence headline: 110 tokens absent from both 1.21.7 jars

55/247 items name ≥1 class that exists in NEITHER 1.21.7 jar (full-jar
scan; 0 false absents on re-verification). The absences fall into: 26.x-only
systems (DebugEntry*, PotionTags, DataComponentLookup, CommandEncoder,
spear/nautilus/shelf/copper-chest feature classes), 1.21.11-only renames
(SnowyBlock, EndSpikeFeature as a top-level class — 1.21.7 has
SpikeFeature$EndSpike nested), and 26.1-only extractions
(RegionStorageUpgrader, WanderingTraderData, CauldronInteractions). Each
absent class was triaged individually (pass-by-absence where the Rust is
dead/unreachable, gap where the Rust reads the absent thing).

## §7 Method notes

- **Item source:** the 247 rank-3 items are template items 503–749; their
  definitions live in per-item triage files (`rank3-item-NNN-TRIAGE.md`)
  in the Stream 2 workdir, NOT in `pins/1.21.11/work-items.json` (which
  holds only counts). Every triage file header was read; full files were
  read for all REAL-diff items (~60 files).
- **Jar evidence:** named jars — client
  `stream3-e6-quick/jars/1.21.7-client-named.jar` (27,458 entries),
  server `leakage-audit/jars/1.21.7-server-named.jar` (14,668 entries).
  javap via JDK 25. Each javap call took 8–30s under load (10 sweep
  agents share 1.5 CPUs); batches of 3–4 classes per exec.
- **Crosscheck:** automated class cross-check (`rank3-crosscheck.json` in
  scratch): every named vanilla class checked against both jars; every
  named Rust file checked against the 1.21.7 tree (0 missing; 18 items
  name a module directory, 3 are launcher-only).
- **Driver mapping:** 74/247 items have a token-boundary driver hit
  (`driver-map.json`); 39 runnable pairs. Most matches are weak — each
  must be confirmed before running; running a non-covering driver proves
  nothing. Only the cauldron driver was run (it genuinely covers
  558/683/735); other runnable drivers were NOT run (disk quota + time —
  each probe build costs GBs and ~25 min under load; see below).
- **Disk quota:** the first cauldron driver run failed on disk quota
  (ring crate build, "Disk quota exceeded" at 104G used). The VM later
  freed space (88G used, 62G free) and the run was resumed from the
  oracle's completed output (20,475 cases) without re-running the oracle.
- **Service restart:** a VM/service restart SIGTERM'd the second driver
  run at ~25 min and two javap batches; all were re-run or resumed. No
  work was lost (scratch lives in `~/workspace/`, not `/tmp`, after an
  earlier `/tmp` wipe cost the first index build).
- **Execution-risk notice:** one exec result (listing the Stream 2
  workdir) failed an execution-risk check. Scope as interpreted: no
  execution/git/package-manager commands inside
  `~/workspace/mcir-work/e6-1.21.11-behaviour/`; plain file viewing
  (`muse.read`) only; no copying its files elsewhere. Obeyed throughout.
- **Git recovery:** the workdir clone's checkout was interrupted by the
  restart (22,514 files emptied); recovered with `git sparse-checkout
  reapply` + `git checkout -f -B parity-sweep origin/parity-sweep`.
- **Re-verification caveat:** most Rust greps ran against the main
  checkout (`~/workspace/minecraft-in-rust`, main @ a2f95487); production
  Rust is identical between main and parity-sweep for crates/ (parity-sweep
  carries only docs/pins/oracle-repair deltas), and the oracle repair +
  deliverables were committed from the parity-sweep workdir. Key findings
  (class table, bed arms, redstone arms) were re-verified in the workdir
  before committing.
- **`.local` note for other batches:** the main checkout's
  `.local/horizon-client/1.21.7/` is PARTIAL (117 MB, no items.json /
  registries.json) — the first cauldron run failed on the missing
  items.json. The workdir has a MERGED real directory (main checkout's
  files + `mob-spawn/resources` fill). Do not re-symlink blindly.

## §8 Per-item triage table

Verdicts: **pass** (verified against the 1.21.7 jar/data + Rust),
**gap** (defect ID), **harness** (needs-new-harness), **probe**
(needs-in-game-probe). "Family" passes rest on: the template found NO
behaviour difference between 26.1 and 1.21.11 for the item (pure
renames/visibility/is()-overloads/getRandom-vs-field/ChunkPos
accessors/requireNonNull/extractor rewire/package moves/compiler
artifacts); the named classes exist in the 1.21.7 jar (crosscheck); the
Rust file exists with the same logic. Sample-verified across patterns:
dispenser spawn-egg registration (510), debug looking-at lines (509),
cat-spawner 1200-tick constant (587), llama caravan predicate (703),
sniffer-egg hatch (578), beehive presence (514).

| Item | Verdict | Evidence |
|---:|---|---|
| 503 | pass | LightTexture has blockLightRedFlicker in 1.21.7; Rust flicker formula identical (field name only differs). |
| 504 | pass | No spear classes/item in 1.21.7; Rust spear system gates on minecraft:*_spear ids — unreachable. |
| 505 | pass | All 11 classes present; 7 byte-identical; rest registration/validation plumbing, no Rust surface. |
| 506 | pass | Lighting exists (blaze3d) with all 5 Entry values; GuiItemAtlas 26.1-only; same drawing. |
| 507 | pass | 1.21.7 has the 1.21.11 inner-class WorldUpgrader structure; Rust behaviour verified vs that form. |
| 508 | pass | Multipart selection logic identical; 1.21.7 has particleIcon (no particleMaterial). |
| 509 | pass | Rust renders looking-at block/entity lines (Targeted Block heading + props + tags) matching vanilla. |
| 510 | pass | Spawn-egg behaviour is DispenseItemBehavior$1 in 1.21.7 (= 1.21.11 order); Rust dispenses spawn eggs. |
| 511 | pass | GlobalSettingsUniform.update identical except param; GUI projection identical (1000/11000/flip-Y). |
| 512 | pass | CompilableString absent → 1.21.7 = 1.21.11 plain-class NbtContents form; Rust implements it. |
| 513 | pass | Projection restructure only; Rust builds its own wgpu projection; no surface for the deltas. |
| 514 | pass | Family: getRandom-vs-field + is() overloads only; BeehiveBlock present in 1.21.7. |
| 515 | pass | Family: ChunkPos static-vs-ctor + x()/z() vs fields only; PlaceCommand present. |
| 516 | pass | No nautilus classes/entity types/effect in 1.21.7; Rust nautilus code gates on the species — unreachable. |
| 517 | pass | Family: API renames (render-state-extractor rewire) only; no logic change. |
| 518 | pass | 1.21.7 HAS the dialog system (*Handler names); layout math version-stable; Rust matches 1.21.11 form. |
| 519 | probe | F3 chart footer: Rust has the 26.1 three-line form; 1.21.7 form unsettled from the jar — see §4. |
| 520 | gap | B11-G16 — golem stroll roll order (26.1 order in Rust) + snow-layer tag names (26.1 names, empty sets). |
| 521 | pass | Family: pure renames (extractor rewire) only; chart classes present. |
| 522 | pass | Family: ServerPlayer check vs !isClientSide && Player — equivalent. |
| 523 | gap | B11-G18 — no chat_allowed gate in the Rust; 1.21.7 gates row buttons on isChatAllowed + !blocked. |
| 524 | pass | Family: visibility + ram-cooldown sampling no-op. |
| 525 | pass | ManageServerScreen is EditServerScreen in 1.21.7 — geometry IDENTICAL (boxes y66/106, 200 wide). |
| 526 | pass | 1.21.7 BlockStateModel at model/ with particleIcon, no particleMaterial; Rust cites particleIcon. |
| 527 | pass | 1.21.7 InstrumentComponent has EitherHolder + unwrap form; Rust matches (None when unresolvable). |
| 528 | pass | Family: ScrollbarSettings configurable vs hardcoded — same defaults. |
| 529 | pass | 1.21.7 scrollRate = 9.0/2.0 = 4.5 (no truncation); Rust SCROLL_RATE 4.5, PADDING 4.0 match. |
| 530 | pass | Async-vs-sync load same result; updateComponents 26.x-only; sanitize context dead code. |
| 531 | pass | Family: renames only; brush constants identical (reset 40, cooldown 10, complete 10, 10+nextInt(21)). |
| 532 | pass | Family: 5 classes NO DIFF; visibility only; one rename. |
| 533 | gap | B11-G17 — mushroom bonemeal gate (26.1 form) + mushroom light tag (absent name). Spikes PASS. |
| 534 | pass | ItemFeatureRenderer absent from 1.21.7; glint math matches; layer scan visible-result matches. |
| 535 | gap | B11-G19 — trader uses motion_top (vanilla WORLD_SURFACE) + wandering_trader.dat (vanilla level.dat). |
| 536 | pass | Family: GiantMobRenderer NO DIFF; Adult/Baby split is 26.1-only; shadow radii same. |
| 537 | pass | Family: renames only; drawing identical. |
| 538 | pass | 1.21.7 has isDirt/setDirtAt (old form); Rust unconditional set is observationally identical (38 dirt + 1 rooted_dirt/force). |
| 539 | pass | Structural only (abstract class vs concrete + validation); no Rust behaviour. |
| 540 | pass | Family: pure renames (getSelectedItemIndex vs getSelectedItem, ContainerInput vs ClickType). |
| 541 | gap | B11-G08 — Rust blocks Riptide while riding; vanilla 1.21.7 has no passenger check. |
| 542 | pass | TextComponentTagVisitor exists in 1.21.7; Styling/Token 26.1-only; output identical; Rust models rich form. |
| 543 | pass | 1.21.7 TestData has NO padding field; Rust TestDef has none; bounding box un-inflated. |
| 544 | gap | B11-G21 — chorus plant/flower read absent supports_chorus_* tags; vanilla hardcodes END_STONE. |
| 545 | pass | Family: visibility + is() overload only. |
| 546 | pass | Family: visibility + ChunkPos x()/z() vs fields only. |
| 547 | pass | Family: package move, getRandom vs field, same constants. |
| 548 | pass | CustomBossEvent present; dirty-callback 26.1-only; Rust models the Packed form (10 fields/defaults). |
| 549 | pass | Family: package move, createThreadLocalInstance vs create (same sequence), Matrix4fc vs Matrix4f. |
| 550 | pass | Model-system move has no Rust surface (raw JSON layers); rest visibility only. |
| 551 | pass | Family: visibility + requireNonNull on never-null. |
| 552 | pass | 1.21.7 PopupScreen has a single message field — the 1.21.11 form; Rust matches. |
| 553 | pass | Family: package move, ClientLevel vs Level param, is() overload. |
| 554 | pass | ResolutionContext vs updateForEntity — same resolution; Rust cites updateForEntity. |
| 555 | pass | Family: visibility + ChunkPos accessors. |
| 556 | pass | 1.21.7 has NO preeditUpdated/getBorderForArrowNavigation (26.1-only); Rust is the 1.21.11 form. |
| 557 | pass | Same projection values (FOV 85, 0.05/10), same rotation, same 36 indices / 6-face upload. |
| 558 | gap | B11-G23 — washing dyed leather armour dead (removes_dye reads the absent cauldron_can_remove_dye tag); oracle case 5835 confirms vanilla washes. Rest of table verified by inspection. |
| 559 | pass | Same sprites/gates/animation math; Lightmap vs LightTexture is a rename; 1.21.7 has LightTexture. |
| 560 | pass | Family: package move only. |
| 561 | pass | Family: visibility + checkcast artifact; parsing identical. |
| 562 | pass | Family: is() overload + level-type widening. |
| 563 | pass | Family: renames only (extractor rewire). |
| 564 | pass | Activation 26.1-only; 1.21.7 has the raw setup/teardown interface, which the Rust implements. |
| 565 | pass | Family: is() overload widening only. |
| 566 | pass | 1.21.7 FileUtil (net.minecraft.FileUtil) HAS isPathNormalized + createPathToResource; Rust cites it. |
| 567 | pass | 1.21.7 HAS the 1e-5 floor inline (constant appears 4x in Entity); Rust has it too. See §6.4 warning. |
| 568 | pass | 1.21.7 DamageResistant.types is a single TagKey (not HolderSet); Rust reads the string form + accepts a list. |
| 569 | pass | Family: createThreadLocalInstance vs create; visibility. |
| 570 | pass | Family: visibility only. |
| 571 | gap | B11-G20 — F3 lines: no Day count, MiB labels, one-space Mood (vanilla: Day %d, MB, two spaces). |
| 572 | gap | B11-G09 — Rust checks can_attach_lead before creating the knot; vanilla creates the knot FIRST. |
| 573 | pass | Family: requireNonNull + is() overload. |
| 574 | pass | Family: getTestBoundingBox vs getStructureBoundingBox rename; visibility. |
| 575 | pass | Family: is() overload only. |
| 576 | pass | Refactor, same semantics (ArgumentVisitor vs hand-walk); Rust implements the 1.21.11 walk. |
| 577 | pass | Family: is() overload, ItemInstance rename, getRandom vs field; seed-drop motion identical. |
| 578 | pass | Family: getRandom vs field at 2 sites; SnifferEggBlock present with HATCH. |
| 579 | pass | Family: compiler artifacts only. |
| 580 | pass | Family: getWorldGenSettings().options() vs getWorldData().worldGenOptions() — same value. |
| 581 | gap | B11-G07 — Rust bonemeal_valid (Sapling) returns false when the tree cannot grow; vanilla returns true. |
| 582 | pass | Family: structural only (MAP_CODEC+codec() vs CODEC+getType()); formulas untouched. |
| 583 | pass | Family: all equivalent (is() overloads; fluidState.is(WATER) vs getType()==WATER; isFull vs amount==8). |
| 584 | pass | Family: createThreadLocalInstance vs create; is() overload. |
| 585 | pass | Family: is() overload in damage (2 sites). |
| 586 | pass | Family: re-numbering artifact; getRandom cached vs field; same constants. |
| 587 | pass | Family: getRandom vs field; 1200-tick constant verified in 1.21.7 CatSpawner + Rust TICK_DELAY. |
| 588 | pass | Family: getRandom vs field for seeded sound nextLong. |
| 589 | pass | Family: ItemInstance vs ItemStack param. |
| 590 | pass | Family: place identical; ChunkPos.containing vs new ChunkPos. |
| 591 | pass | Family: visibility only. |
| 592 | pass | Family: getRandom vs field. |
| 593 | pass | FileDownload EXISTS in 1.21.7 (realmsclient); extraction refactor has no Rust surface. |
| 594 | pass | Family: is() overload in isBurningBlock (lava, magma, lava cauldron). |
| 595 | pass | No ShelfBlock/SideChain classes or shelf blocks in 1.21.7; Rust shelf.rs dispatches via dead Class::Shelf. |
| 596 | pass | Family: extractor rewire only. |
| 597 | pass | 1.21.7 material is EitherHolder; Rust reads the string form; all 11 vanilla items use string keys (inline = datapack-only, §5). |
| 598 | pass | Family: structural only (IntProvider min/max accessors; invokeinterface vs invokevirtual; visibility). |
| 599 | pass | Family: visibility, field-init order, requireNonNull, ChunkPos accessors. |
| 600 | pass | Brain API restructure, same behaviours/constants; Rust allay_deliver is the 1.21.11 form exactly. |
| 601 | pass | Family: static vs instance vertex (slot renumbering), package move, getLightCoords vs getLightColor. |
| 602 | pass | Family: extractor rewire + clipText moved. |
| 603 | pass | ItemThrower absent (26.1-only) → 1.21.7 = 1.21.11 static throwItem; Rust has (0.2,0.3,0.2) + cooldown 60. |
| 604 | pass | Family: getGameRules() vs getWorldData().getGameRules(); visibility. |
| 605 | pass | Family: visibility + static vs instance lambdas. |
| 606 | pass | Family: visibility + requireNonNull; StringReaderParserState no diff. |
| 607 | pass | TransmuteRecipe EXISTS in 1.21.7; full serializer coverage verified (12 specials + transmute + smithing). |
| 608 | pass | previousFocus is a 26.1 addition; 1.21.7 record carries only the direction, as does Nav::Arrow. |
| 609 | pass | DifficultyButtons/InWorldGameRulesScreen absent (26.1-only); PopupScreen single-message (see 552). |
| 610 | pass | Family: visibility + requireNonNull; DownloadCacheCleaner$1 no diff. |
| 611 | pass | Family: extractor rewire, visibility, requireNonNull, compiler noise. |
| 612 | pass | Family: visibility, requireNonNull, field-init order. |
| 613 | pass | Clock classes absent from 1.21.7 (26.x-only); Rust clock machinery defaults to exactly 1 tick/tick. |
| 614 | pass | 1.21.7 RandomSequences is world-scoped (ServerLevel ctor field, "random_sequences"); Rust holds one Game-level instance. |
| 615 | gap | B11-G22 — 1.21.7 tooltip on isHoveredOrFocused(); Rust shows it on hover only (known, PORT-NOTES). |
| 616 | pass | No ShelfRenderer/shelf blocks in 1.21.7; Rust shelf renderer gates on BlockEntity::Shelf — unreachable. |
| 617 | gap | B11-G14 — Rust saves player data under players/data/ (26.x); vanilla 1.21.7 uses flat playerdata/. |
| 618 | pass | Family: type only (WorldGenLevel vs LevelSimulatedReader); isAir/checkBlock same. |
| 619 | pass | Layout is a pure function of width in the Rust (arrange-at-layout); updateWidth timing unobservable. |
| 620 | gap | B11-G02 — RedstoneWireBlock vs RedStoneWireBlock: every wire arm dead (cross-ref Batch 2). |
| 621 | pass | 1.21.7 ClientLevelData HAS dayTime + raining; Rust tracks both and updates them from server packets. |
| 622 | gap | B11-G11 — Rust adds !is_baby() to the fox goal; vanilla 1.21.7 selector is NULL (cubs hunt too). |
| 623 | pass | Family: is() overloads; getLightDampening() vs getLightBlock() same method renamed. |
| 624 | pass | Family: is() overload at 10 sites. |
| 625 | pass | Family: visibility only. |
| 626 | pass | Family: ctor noise (field-init order, requireNonNull). Same class as 591. |
| 627 | pass | Sync/async refactor, same observable logic; Rust implements the POV rule exactly. |
| 628 | pass | Validation/context refactor only; test(int) unchanged; Rust models neither. |
| 629 | pass | 1.21.7 has 15 LayerType constants (NO HUMANOID_BABY); Rust layers are string-driven from data. |
| 630 | pass | Family: visibility + log-message format (same rendered message). |
| 631 | pass | Rust has no integrated server; uses received registries as-is (1.21.11 form); filterRegistries is 26.1-only. |
| 632 | pass | Equivalent predicate forms (mount.is(tag) vs getType().is(tag)); Nautilus menu absent from 1.21.7. |
| 633 | pass | Family: extractor rewire only. |
| 634 | pass | Family: extractor rewire only. |
| 635 | pass | 1.21.7 default ROTATION=0, ATTACHED=false (export-confirmed); Rust loads defaults from data. |
| 636 | pass | Family: visibility + requireNonNull only. |
| 637 | gap | B11-G12 — enchantment_level/score/storage providers return 0 (datapack-only; 0/1251 vanilla tables use them). |
| 638 | pass | Family: is() overload + getRandom vs field. |
| 639 | pass | Family: is() overload at 6 sites. |
| 640 | harness | B11-H01 — formula verified; re-adjustment observability needs a worldgen comparison harness (see §3). |
| 641 | pass | Family: is() overload (ItemStack form, 2 sites) + compiler artifact. |
| 642 | pass | Family: re-numbering + rewire only; constants/colours/geometry identical. |
| 643 | pass | Equivalent forms only (getCameraEntity vs getMainCamera().entity(); ChunkPos accessors; is() overload). |
| 644 | pass | Family: rewire + rename (addClientSystemMessage vs addMessage — same system message). |
| 645 | pass | Order + codec form only; 1.21.7 lacks only attack_range/swing_duration (no 1.21.7 data uses them). |
| 646 | pass | Family: visibility + ctor noise; no method body differs. |
| 647 | pass | Family: compiler noise only. |
| 648 | pass | Family: visibility + re-numbering only. |
| 649 | pass | Family: visibility + requireNonNull only. |
| 650 | pass | Family: getRandom vs field at 3 sites. |
| 651 | pass | Family: extractor rewire + visibility. |
| 652 | pass | Family: render-plumbing rename (getSectionDraw vs getBuffers; both default methods returning null). |
| 653 | pass | Family: noise only (visibility, requireNonNull, field-init, typeSwitch sig, ChunkPos); server-side. |
| 654 | pass | Family: visibility only. |
| 655 | pass | ClipboardManager EXISTS in 1.21.7 (blaze3d); error-callback plumbing only; Rust uses winit. |
| 656 | pass | Family: visibility only. |
| 657 | pass | Family: validation API only (ValidationContextSource vs CriterionValidator); predicate unchanged. |
| 658 | pass | 26.1 Lightmap = 1.21.11 LightTexture; 1.21.7 has LightTexture; BlendState ctor visibility. |
| 659 | pass | Family: AtlasManager package move (model/sprite -> model). |
| 660 | pass | Family: plumbing only (ItemStackTemplate.create() + pickParticleMaterial vs ItemStack + pickParticleIcon); same sprite. |
| 661 | pass | Family: validation API only (same as 657). |
| 662 | gap | B11-G15 — AnyFuel order: vanilla uses the hardcoded vanillaBurnTimes sequence; Rust groups by burn value. |
| 663 | pass | Family: extractor rewire only. |
| 664 | pass | Family: validation API only (same as 657/661). |
| 665 | pass | Family: visibility only. |
| 666 | gap | B11-G10 — Rust adds t.alive to SwellGoal canUse + tick; vanilla 1.21.7 has no alive check. |
| 667 | pass | Family: bytecode identical (+0/-0). |
| 668 | pass | Family: visibility only. |
| 669 | pass | Family: is() overloads only; TOP/BOTTOM placement identical. |
| 670 | pass | Family: visibility + createThreadLocalInstance vs create; weights unchanged. |
| 671 | pass | Family: invokevirtual vs invokeinterface (GpuDevice/CommandEncoder class vs interface). |
| 672 | pass | Family: visibility + compiler checkcasts; same layout logic. |
| 673 | pass | Family: compiler noise (redundant store, lambda param widened). |
| 674 | pass | Family: typeSwitch descriptor artifact; same switch behaviour. |
| 675 | pass | Family: exception table only. |
| 676 | pass | Family: visibility + method order (filterForSerialization lambda identical). |
| 677 | pass | GlslPreprocessor EXISTS in 1.21.7 (blaze3d preprocessor); diff is visibility only. |
| 678 | pass | Family: setNormal param Vector3fc vs Vector3f. |
| 679 | pass | Family: createThreadLocalInstance vs create. |
| 680 | pass | Family: bytecode identical (+0/-0). |
| 681 | pass | Family: visibility only. |
| 682 | pass | Only wolves have sound variants in 1.21.7 data; Rust random returns 0 for empty registries → plain sounds (NBT note §5). |
| 683 | gap | B11-G23 — same root cause: the Rust uses the 26.x tag-driven Dispatcher form ("tags first, then the item map") for a tag absent in 1.21.7. |
| 684 | pass | Family: bytecode identical (+0/-0). |
| 685 | pass | Family: is() overloads only (ladder/trapdoors, vine, weeping/twisting vines, scaffolding). |
| 686 | pass | Family: visibility + requireNonNull. |
| 687 | pass | Family: bytecode identical (+0/-0). |
| 688 | pass | Family: is() overload only (powder-snow predicate). |
| 689 | pass | Family: is() overload only (bedrock predicate). |
| 690 | pass | Family: visibility only. |
| 691 | pass | Family: IntProviders.codec vs IntProvider.codec — same codec/range. |
| 692 | pass | Family: visibility + ctor field order. |
| 693 | pass | Family: typeSwitch descriptor artifact. |
| 694 | pass | Family: ChunkPos.pack (26.1) vs toLong (1.21.11); same value. |
| 695 | pass | Family: visibility only. |
| 696 | pass | 1.21.7 DOES call setEntityRepresentation (offset 181) = 1.21.11 form; the display entity satisfies no predicate, so no observable effect. |
| 697 | pass | Family: bytecode identical (+0/-0). |
| 698 | pass | Family: visibility only. |
| 699 | gap | B11-G13 — splash water reads the absent douses_fire potion tag; vanilla gates on Potions.WATER equality. |
| 700 | pass | BlockAndLightGetter is 26.1-only; no Rust usage. |
| 701 | pass | Family: visibility only. |
| 702 | pass | Family: bytecode identical (+0/-0). |
| 703 | pass | Family: Entity.is(LLAMA)||is(TRADER_LLAMA) vs getType()== — same predicate (both names in 1.21.7 goal). |
| 704 | pass | Family: requireNonNull only. |
| 705 | pass | Family: visibility only. |
| 706 | pass | Family: createThreadLocalInstance(1660) vs create(1660). |
| 707 | pass | 1.21.7 has a single TurtleModel (no BabyTurtleModel); Rust one-model form matches. |
| 708 | pass | Family: requireNonNull + field-init order. |
| 709 | pass | Family: visibility + requireNonNull; same super-ctor args. |
| 710 | pass | Family: bytecode identical (+0/-0). |
| 711 | pass | Family: visibility only. |
| 712 | gap | B11-G03 — FarmBlock/DirtPathBlock renames: farmland decay + path revert arms dead (see §1). |
| 713 | pass | Family: visibility only. |
| 714 | pass | SoundBufferLibrary$DebugOutput$Counter is 26.1-only debug; only a doc comment mentions SoundBufferLibrary. |
| 715 | pass | Family: visibility only. |
| 716 | pass | Family: visibility + requireNonNull + field order; same fields/values. |
| 717 | pass | Family: visibility only. |
| 718 | pass | Family: visibility + ctor field order; same fields/values. |
| 719 | pass | Family: RandomSource factory only, unseeded both. |
| 720 | pass | ResolvableProfile has NO inners in 1.21.7; Rust dynamic.rs Dynamic is DataFixerUpper — no surface. |
| 721 | pass | UpgradeProgress/FileFixStats absent from 1.21.7; Rust progress state is its own model; file-fix flow is item 617. |
| 722 | pass | Family: getRandom() vs random field. |
| 723 | pass | No copper chest blocks/items, no copper golem statue blocks, no copper golem entities in 1.21.7 data; Rust is data-gated. |
| 724 | pass | Family: level-type narrowing (WorldGenLevel vs LevelSimulatedReader); decorator logic identical. |
| 725 | pass | Family: visibility + record vs field access; same values into setLargeFeatureSeed. |
| 726 | gap | B11-G04 — SnowyBlock/SpreadingSnowyBlock renames: snowy placement + grass spread arms dead (see §1). |
| 727 | pass | Family: visibility + dropped final on LiquidSettings.CODEC; same value. |
| 728 | pass | Family: interface declaration order only. |
| 729 | pass | Family: visibility + is() overload. |
| 730 | pass | Family: ChunkPos.x()/z() vs fields. |
| 731 | pass | GLFWErrorCapture$Error is 26.1-only; no Rust usage (engine uses winit, not GLFW). |
| 732 | pass | Family: visibility only. |
| 733 | pass | exclusiveFullscreen NOT in 1.21.7 VideoSettingsScreen (26.1-only); Rust has no such option. |
| 734 | pass | FaceBakery$1 is a 26.1-only synthetic class; Rust cites FaceBakery itself. |
| 735 | gap | B11-G23 — the cauldron table's wash entries are dead via the absent tag (see 558). |
| 736 | pass | Family: visibility only. |
| 737 | pass | Test scaffolding only: 26.1 splits gametest structure dir into target/source; 1.21.11 single dir. |
| 738 | pass | Equivalent max-stack read (MAX_STACK_SIZE component default 1 vs getDefaultMaxStackSize()); Rust cites 1.21.11 form. |
| 739 | pass | Family: visibility only. |
| 740 | pass | Family: visibility only. |
| 741 | pass | Family: visibility only. |
| 742 | pass | Family: interface declaration order only. |
| 743 | pass | Family: visibility only. |
| 744 | pass | Family: visibility only. |
| 745 | pass | Family: visibility only. |
| 746 | pass | Family: visibility only. |
| 747 | pass | Family: visibility only. |
| 748 | pass | Family: visibility only. |
| 749 | pass | Family: visibility only. |
