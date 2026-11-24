# Batch 4 — Block entities / containers — 26.1.2 parity sweep

Date: 2026-11-25. Version: 26.1.2. Worktree: `~/workspace/mcir-work/sweep-26.1.2`
(branch `parity-sweep-26.1.2`). Jars: `~/workspace/mcir-work/26.1.x/versions/26.1.2/jars/`
(`server-game.jar` sha1 `83eb1106…`, `client.jar` sha1 `4e618f09…`, both verified;
`server.jar` is a bundler jar, never javap'd).
Oracle table: `docs/sweep/26.1.2/batch-0-oracles.json` (391 oracles; the 26.1.2
table, landed before this batch finished — see §Method).
Prepared data: `versions/26.1.2/.local/horizon-client/26.1.2/` (built by Batch 12).

Checks + triage ONLY. No code was changed, no commit was made, no git command was
run in the worktree. Every vanilla claim below is verified against 26.1.2's OWN jar
or data, never the 26.3 base's form (owner's VANILLA-CHECK rule). Where a claim
rests on a static read rather than a live run, that is stated.

## §0 Execution status

Complete. All 62 primary (26.1.1) re-checks decided, all 42 1.21.7 re-checks
decided, all 95 checks tallied. The two mandatory re-check-first items, B4-G01
(furnaces never light) and B4-G02 (brewing stands never brew), were verified
first and both REPRODUCE as majors with fresh 26.1.2 evidence. No stall: every
jar, class and data file needed was available. One carried item stays UNVERIFIED
(B4-G17, smoker/blast burn-time halving — still masked by G01, no slice item).

The headline result is a null delta: **every 26.1.1 verdict in this batch's scope
reproduces exactly in 26.1.2** — same tally (95 checks, 47 pass, 48 gap), same
62 items with the same statuses and severities. That is the expected outcome
for a delta port (34 files differ between the trees, none in a Batch 4
behaviour site — see §Method), and it was verified fresh rather than assumed:
each item below carries 26.1.2 jar/source/data evidence. Three findings refine
the 26.1.1 record rather than repeating it: the G25 mechanism is zeroing, not
truncation (serde_json `as_i64()` returns `None` for float JSON numbers); the
G22 mechanism is a missing `isExplorationMap` exclusion, not a wrong predicate;
and the 26.1.1 report's "68 trade-set files with float amount" sub-claim does
not reproduce from the jar (387 trade files, one trade each, no `amount` key
anywhere). One 26.1.2 delta change narrows a defect on LAN only: the protocol
layer now drops the synthetic sign-text components at wire encode (G33) — the
local pick/place defect is unchanged.

## Tally (as-tallied, per the 26.2 precedent)

| class | total | pass | gap |
|---|---|---|---|
| Drivers with a passing oracle | 17 | 17 | 0 |
| Drivers oracle-blocked | 20 | 0 | 20 |
| Drivers with no oracle | 20 | 13 | 7 |
| Method-2 checks | 38 | 17 | 21 |
| **Total** | **95** | **47** | **48** |

Behaviour / non-behaviour split (stated explicitly, as the brief requires):

- **Driver-status (non-behaviour) checks: 37** — 17 pass / 20 gap. These count
  whether a driver's oracle compiles; they say nothing about gameplay.
- **Behaviour checks: 58** — 30 pass / 28 gap (20 no-oracle drivers + 38 method-2
  checks).

Items: **62** — 30 behaviour gap, 17 oracle-defect, 1 fixed, 14 closed.
Live majors: 8 (G01, G02, G16, G25, G30 behaviour; O06, O08, O23 oracle).

Comparison with the 26.1.1 run (95 checks, 47 pass, 48 gap; 62 items): identical
tally, identical item set. No check flipped in either direction. The 26.1.1 run's
swing vs 26.1 (the regression finding) is fully inherited: the 26.1.2 tree is a
delta port of the 26.1.1 tree, not a re-port, so it carries the same regressed
code.

## Method

Same as the 26.1.1 run:

1. **Drivers with a passing oracle (17):** the probe compiles and the driver is
   declared pass statically, per the 26.2 precedent (no live run on this VM;
   the client_probe lib test does not compile on the VM for resources, and the
   sweep's live runs are a CI matter).
2. **Drivers oracle-blocked (20):** the driver's oracle fails to compile against
   the 26.1.2 jar; each is a gap, attributed to the oracle defect that blocks it.
   Oracle error counts and first errors are taken from the Batch 0 table, not
   re-measured; each cause was verified fresh with javap against the 26.1.2 jar.
3. **Drivers with no oracle (20):** verdict from reading the driver's assertions
   against the 26.1.2 port code and vanilla bytecode.
4. **Method-2 checks (38):** static verification of an area against the jar —
   javap of the vanilla class, a read of the port code, and (where data matters)
   a count over the prepared 26.1.2 data. No count is asserted without running it.

**Oracle table.** The 26.1.2 Batch 0 table (`docs/sweep/26.1.2/batch-0-oracles.json`,
391 oracles, generated 2026-11-25) landed during this batch and is the authority
used throughout. It is measurement-identical to the 26.1.1 table across ALL 391
oracles: same oracle set, same pass/fail per oracle, same error counts and first
errors wherever they exist (verified by a full key-by-key comparison, not a
sample). The provisional 26.1.1-table triage and the 26.1.2-table triage therefore
agree on every item; nothing was re-decided on landing.

**26.1.1 tree as control.** Per the brief, every file in scope was diffed against
the 26.1.1 tree (`versions/26.1.1/` in the worktree). The whole-tree diff is 34
files (list saved in `b04/evidence/diffs/tree-diff-26.1.1-to-26.1.2.txt`). Every
Batch 4 behaviour file is byte-identical: `providers.rs`, `brewing.rs`,
`menu.rs`, `block_entity_nbt.rs`, `block_entity_item.rs`, `furnace.rs`,
`containers.rs`, `merchant.rs`, `sign_edit.rs`, `composter.rs`, `level/sculk.rs`,
`dispenser.rs`, `spawner.rs`, `trial_spawner.rs`, `lan_stations.rs`,
`recipe_book.rs`, `machines.rs`, `crafter.rs`, `storage.rs`, `items.rs`
(horizon-items), `level_particles.rs`, `game.rs`, `stations.rs`,
`cartography.rs`, `beacon.rs`, `vault.rs`, `native_machines.rs`,
`menu_clicks.rs`, `lan/menu.rs`, `storage.rs` (horizon-core), `signs.rs`,
`chunk/block_entity.rs`, `game/creatures.rs`, `anvil.rs`, `beehive.rs`,
`shulker.rs`, `jukebox.rs`, `chiseled_bookshelf.rs`, `loom.rs`,
`level/redstone.rs`, `crafting.rs`. The oracle sources (`mods/`) and the
drivers (`tools/`) are byte-identical too. The only differing files that touch
Batch 4 scope at all are:

- `horizon-client/src/save.rs` — DataVersion threshold 4295→4997 and palette
  key `Name`/`Properties`→`id`/`properties` (diff saved in
  `evidence/diffs/save.rs.diff`). No Batch 4 line touched: `decode_block_entity`
  still drops beds via `BlockEntity::for_block(block)?`, the brewing-save arm is
  unchanged.
- `horizon-protocol/src/item.rs` + `item/skip.rs` — the synthetic sign-text
  component kinds (`SIGN_TEXT_FRONT`/`SIGN_TEXT_BACK`, -1001/-1002) no longer
  cross the wire (diffs saved). LAN-only narrowing of G33; see §Behaviour
  defects. The constants themselves still exist (`item.rs:129-130`).
- `horizon-world/src/chunk.rs` — palette `from_palette_nbt` legacy-form removal
  (diff saved). Not a Batch 4 behaviour change.

The remaining differing files are version plumbing (`version.rs`, `ids.rs`
comment, `lib.rs` test, `status.json`, `research-lock.json`, docs), the
Checkbox tooltip (`dialog_screen.rs` — Batch 10 scope; verified Checkbox-only),
datafix files, LAN/net/realms/social/title code, and `datapack.rs`/`packs.rs`.
None touches a Batch 4 behaviour site.

**Vanilla delta.** `PORT-NOTES-26.1.2.md` records the 26.1.1→26.1.2 delta as
DataVersion 4788→4790 plus 6 changed classes: `ServerGamePacketListenerImpl.
handleAttack`, `Checkbox` (+ `Builder`), `AbstractReportScreen.createFooter`,
`DetectedVersion.createBuiltIn`, `SharedConstants` version fields. None is a
Batch 4 class; packets, registries, blocks, items and command nodes are
unchanged, and the 1,202 changed data files differ only in the embedded
DataVersion. Three B4 oracles reference `ServerGamePacketListenerImpl`
(Dispenser line 839 subclasses it, Menu lines 12/70, PlaceRecipe) — as a
superclass/import only, not `handleAttack` — and the measured table is
identical anyway, so the delta has no oracle effect in this batch.

**26.1-record corrections (26.1.1 summary.md line 360):** applied. None is
Batch 4 relevant: the section covers B3-G33 visibility and B8-H02 provider
counts.

## Systemic finding: the 26.1.1 regression is inherited wholesale

The 26.1.1 report's systemic finding stands unchanged for 26.1.2, because the
code is byte-identical: the 26.1.2 tree carries 26.3-form code in files where
the 26.1 tree carries corrected forms. The mechanism is one pattern:
**component/tag reads for data that does not exist in 26.1.2**. Verified fresh
against the 26.1.2 jar and prepared data:

- `DataComponents` in the 26.1.2 server-game jar has no `cooking_fuel`,
  `brewing_fuel`, `villager_food`, `compostable`, `mob_visibility`,
  `sign_text_front`, `sign_text_back` or `waxed` (dump saved in
  `evidence/javap/DataComponents.txt`).
- Counted over the prepared 26.1.2 data: **0 of 1,506 items** carry any of those
  eight components.
- The jar has no `#sculk_growth_inhibitors` or `#extendable_maps` tag, and no
  `data/minecraft/recipe/brewing/` files (0).
- Vanilla 26.1.2 uses the 26.1-form sources the 26.1 tree restored:
  `FuelValues.vanillaBurnTimes` (64 `Builder.add` calls, verified), the
  `#brewing_fuel` item tag (exists; contains only `minecraft:blaze_powder`) +
  `PotionBrewing` mixes, `ComposterBlock.COMPOSTABLES` (hardcoded
  Object2FloatMap), hardcoded sculk sensor/shrieker checks.

Consequence, same as 26.1.1: two 1.21.7 majors that 26.1 had FIXED are still
back (G01 furnaces never light, G02 brewing stands never brew), and the 8 new
26.1.1 defects (G30–G37) are all still present. The 26.1.2 port is no better
and no worse than the 26.1.1 port in this batch's scope.

## Behaviour defects (gap items)

### B4-G01 (major, 1.21.7 REPRODUCES — regressed from 26.1 FIXED) — furnaces never light

`Providers::burn_time` (`horizon-client-render/src/providers.rs:108-113`) reads
`i.fuel`, the `minecraft:cooking_fuel` component, and maps a missing component
to 0. Counted over the prepared 26.1.2 data: **0 of 1,506 items** carry
`minecraft:cooking_fuel` (or `brewing_fuel`, `villager_food`, `compostable`,
`mob_visibility`). Every item's burn time is 0; no furnace, smoker or blast
furnace ever lights. `menu.rs:1432` `is_fuel` reads `i.fuel.is_some()`, so
hopper fuel insertion (`machines.rs`) and recipe-book AnyFuel groups (empty)
fail the same way. Vanilla 26.1.2 uses `FuelValues.vanillaBurnTimes` (verified:
`FuelValues` exists at `net.minecraft.world.level.block.entity.FuelValues`, 64
`Builder.add` calls, same as 26.1.1; dump saved in
`evidence/javap/FuelValues.txt`) — the hardcoded table the 26.1 tree restored
and the 26.1.1/26.1.2 trees dropped. RE-CHECK FIRST item: verified fresh, not
carried.

### B4-G02 (major, 1.21.7 REPRODUCES — regressed from 26.1 FIXED) — brewing stands never brew

Two independent breaks, either fatal alone. (1) `Brewing::load`
(`brewing.rs:203-210`) reads data-driven `minecraft:brewing` recipes from
`data/minecraft/recipe/brewing/`; the 26.1.2 jar has **0** such files, so
`recipes` is empty, `find` never matches, and `is_reagent`/`is_bottle` are
false for every stack — the brewing menu slots refuse every input. The tree's
own `stations.rs:1857` test expects `data.brewing.recipes.len() == 279` (the
26.3 number); it would fail against the prepared 26.1.2 data. (2)
`brewing_fuel` (`brewing.rs:423-436`) reads the `brewing_fuel` component —
0 items carry it. Vanilla 26.1.2 uses the `#brewing_fuel` item tag (verified:
the tag EXISTS in the jar, containing only blaze powder) + `PotionBrewing`
mixes (verified: the class EXISTS), `FUEL_USES` 20, 400-tick brew. The
brewing-totals facet (Batch 3's flag, carried from 26.1.1): the totals DEFAULT
to vanilla's constants (`brewing.rs:322-324`: `total_brew_time: 400`,
`total_fuel: 20`, `speed: 1.0`), and the only paths that mutate them require
the absent `brewing_fuel` component — unreachable in 26.1.2. So the render
change is behaviourally neutral here; the live divergence is the fuel/recipe
mechanism above, and the save divergence is G36. No separate item minted.
RE-CHECK FIRST item: verified fresh, not carried.

### B4-G04 (minor, 26.1.1 REPRODUCES) — furnace completes on `>=`

`furnace.rs:128` completes when `cooking_timer >= cooking_total`. Vanilla
`AbstractFurnaceBlockEntity.serverTick` completes only on `==` (bytecode 274
`if_icmpne` — dump saved in `evidence/javap/AbstractFurnaceBlockEntity.txt`).
Same as 26.1.1.

### B4-G05 (minor, 26.1.1 REPRODUCES) — fuel remainder pushed to drops

`furnace.rs:120` pushes the fuel remainder to `drops` when the fuel stack does
not empty. Vanilla `consumeFuel` shrinks the stack, then places the remainder
(e.g. empty bucket) into the fuel slot only when the stack empties after the
shrink (dump saved). Same as 26.1.1.

### B4-G08 (minor, 26.1.1 REPRODUCES) — hopper suction bowl is 0.125..0.875

`machines.rs:973`. Vanilla `Hopper.getSuckAabb` returns `SUCK_AABB` =
`Block.column(16.0, 11.0, 32.0)` → (0, 11/16, 0)–(1, 2, 1) (dump saved in
`evidence/javap/Hopper.txt`). Same as 26.1.1.

### B4-G09 (minor, 26.1.1 REPRODUCES) — crafter eject reaches blocks only

`crafter.rs:149-152` `crafter_eject` uses block-only `Containers::view`.
Vanilla `CrafterBlock.dispenseItem` uses `HopperBlockEntity.getContainerAt` +
`AABB.ofSize` and reaches entity containers (dump saved in
`evidence/javap/CrafterBlock.txt`). Same as 26.1.1.

### B4-G10 (minor, 26.1.1 REPRODUCES) — crafter trigger checks only the local player

`crafter.rs:188` checks `self.player` only. Vanilla iterates ALL
`ServerPlayer`s. Same as 26.1.1.

### B4-G13 (minor, 26.1.1 REPRODUCES — worse than 26.1) — no bed block entity

`containers.rs` `for_block` has no bed arm (0 `bed` matches in the file);
`save.rs` `decode_block_entity` does `BlockEntity::for_block(block)?`, dropping
bed BEs. `block_entity_nbt.rs` `TYPES` also dropped `"minecraft:bed"` (48→47)
and the bed save arm is gone, so beds are now unrepresentable at the
canonicalizer layer too. Vanilla has `BedBlockEntity` (colour field only) and
`BlockEntityType.BED` (dumps saved). Same as 26.1.1.

### B4-G14 (minor, 26.1.1 REPRODUCES) — ender chest opens on LAN guests

`storage.rs:157` `open_ender_chest`. Same as 26.1.1.

### B4-G15 (minor, 26.1.1 REPRODUCES) — no decorated-pot block entity

`containers.rs` `for_block` has no pot arm (0 `decorated_pot` matches);
`container_size` covers only chest/barrel/ender_chest/hopper/dispenser/dropper/
shulker/copper_chest → `None` for the pot. Vanilla `DecoratedPotBlockEntity`
implements `ContainerSingleItem$BlockContainerSingleItem` +
`RandomizableContainer`, and `BlockEntityType.DECORATED_POT` exists (dumps
saved). Same as 26.1.1.

### B4-G16 (major, 26.1.1 REPRODUCES) — cartography table gated on a nonexistent tag

`menu.rs:1290` gates map cloning on `#extendable_maps`. The tag does not exist
in the 26.1.2 jar. Vanilla `CartographyTableMenu` has 0 `ItemTags` references
(dump saved). Same as 26.1.1.

### B4-G18 (minor, 26.1.1 REPRODUCES) — dispenser mud uses the wrong tag spelling

`dispenser.rs:2245` checks `minecraft:convertible_to_mud`. Vanilla
`DispenseItemBehavior$14` uses `BlockTags.CONVERTABLE_TO_MUD`; the jar has
`convertable_to_mud` and no `convertible_to_mud` (dump saved). Same as 26.1.1.

### B4-G20 (minor, 26.1.1 REPRODUCES) — spawner `setEntityData` merge

`game/spawner.rs:464` merges egg data into the spawner config
(`set_entity_data`). Vanilla's `Spawner` interface has only `setEntityId`; 0
`setEntityData` refs in its class bytes (dump saved). Same as 26.1.1.

### B4-G21 (minor, 26.1.1 REPRODUCES) — merchant result slot blocks pickup

`menu.rs:1505` `may_pickup` gates the merchant result slot (checked at
`:2012`, `:2026`, `:2902`, `:3003`). Vanilla `MerchantResultSlot` has NO
`mayPickup` override (methods: mayPlace, remove, onQuickCraft,
checkTakeAchievements, onTake — dump saved). Same as 26.1.1.

### B4-G22 (minor, 26.1.1 REPRODUCES) — exploration-map check omits the vanilla exclusion

Vanilla `MapExtendingRecipe.matches` returns false when
`MapItemSavedData.isExplorationMap()` is true (bytecode 41–50), and only then
checks scale < 4 (dump saved). The port's `MapExtending` match
(`crafting.rs:942-960`) checks only the paper ring and `scale < 4` — it never
checks whether the map is an exploration map, so exploration maps (which
vanilla refuses to extend) extend in the port. REFINED vs the 26.1.1 record:
the defect is a missing exclusion, not a "different predicate" — the port has
no exploration-map check anywhere in the recipe path.

### B4-G23 (minor, 26.1.1 REPRODUCES) — hero discount computed in f32

`merchant.rs:109` `f64::from(0.3f32 + 0.0625f32 * a)`. Vanilla `Villager.
updateSpecialPrices` uses `0.3d + 0.0625d * amplifier` with `Math.floor`
(double) (dump saved, `Villager.txt:761-830`). Same as 26.1.1.

### B4-G24 (minor, 26.1.1 REPRODUCES) — villager levels up immediately

`merchant.rs:398` `level += 1`. Vanilla `rewardTradeXp` sets
`updateMerchantTimer = 40`, `increaseProfessionLevelOnUpdate = true` — the
level-up is deferred to the next merchant update (dump saved). Same as 26.1.1.

### B4-G25 (major, 26.1.1 REPRODUCES) — merchant float parsing

`merchant.rs:675` `int_provider` uses `as_i64()`; `:768` reads singular
`given_item_modifier`. REFINED vs the 26.1.1 record: `merchant.rs` uses
`serde_json::Value`, and `as_i64()` returns `None` for float JSON numbers — so
`unwrap_or(0)` ZEROES every float count, it does not merely truncate it (a
trade wanting 4.0 emeralds wants 0). Re-quantified against the 26.1.2 jar data,
core counts identical to 26.1.1: 387 trade files (one trade each);
218 float `wants.count`, 0 float `gives.count`, 371 float `max_uses`, 266 float
`xp`, 41 plural `given_item_modifiers`, 0 singular. CORRECTION to the 26.1.1
record: its "Trade sets: 68 files, 68/68 with float amount" sub-claim does not
reproduce — there is no `amount` key in any trade file (0 occurrences across
all 387), and the trade_rebalance datapack holds 93 trade files + 13 tag
files, not 68. The defect itself is unaffected: the float counts above are the
quantification that matters.

### B4-G26 (minor, 26.1.1 REPRODUCES) — loot engine reads 26.3 key names

`horizon-items/src/items.rs` reads `["modifier"]` only (lines 1839–1840, 2112,
2253) and `condition()` reads the `type` key only (line 1957). Re-quantified
against the 26.1.2 jar data, identical to 26.1.1: 1,326 loot tables; 983
`functions` occurrences, 0 `modifier`; 1,545 condition objects naming kind via
the `condition` key. Same as 26.1.1.

### B4-G27 (minor, 26.1.1 REPRODUCES) — furnace int times + sign `allow_op_features` in saves

`block_entity_nbt.rs:383` furnace int times + `speed_multiplier` (vanilla:
4× `putShort`, no `speed_multiplier` — dump saved); sign `allow_op_features`
kept in the save (`:458-459`, vanilla `SignBlockEntity.saveAdditional` writes
`front_text`, `back_text`, `is_waxed` only — dump saved). Same as 26.1.1.

### B4-G28 (minor, 26.1.1 REPRODUCES) — pot sherds saved in compound form

`block_entity_nbt.rs:664-672` compound arm. Vanilla `PotDecorations.CODEC` is
`byNameCodec().sizeLimitedListOf(4).xmap(...)` — list form (dump saved). Same
as 26.1.1.

### B4-G29 (minor, 26.1.1 REPRODUCES) — crafter trigger reads the `recipes` key

`crafter.rs:206` `recipe_matches(&c["recipes"], &id)`. Vanilla
`RecipeCraftedTrigger$TriggerInstance` has the singular `recipeId` field, and
the advancement `adventure/crafters_crafting_crafters.json` has
`"recipe_id": "minecraft:crafter"` (both verified). Same as 26.1.1.

### B4-G30 (major, 26.1.1 REPRODUCES) — composter never composts

`composter.rs:24-25` `compostable()` reads `i.compostable`, the
`minecraft:compostable` component — 0 items carry it, so `composter_add`,
`use_composter`, `composter_takes`, `composter_container_insert` and
`villager_insert_item` all refuse every item: the whole system is dead.
Vanilla `ComposterBlock` uses the hardcoded `COMPOSTABLES` Object2FloatMap
(verified: exists) with `addItem` and `nextFloat` chance rolls (dump saved).
Same as 26.1.1.

### B4-G31 (minor, 26.1.1 REPRODUCES) — beacon validates effects and kicks LAN guests

`menu.rs:306` `beacon_required_levels` + `:1203` `beacon_update` validate the
effect pair and return false when invalid; `lan_stations.rs:771` kicks LAN
guests when `beacon_update` returns false ("Player {} tried to set invalid
beacon effects"). Vanilla 26.1.2 has NO validation:
`BeaconBlockEntity` has no `validateEffects`/`getRequiredLevelsFor`, and
`BeaconMenu.updateEffects` is payment check → encode both effects → set data →
remove payment, with no validation step (dumps saved). Same as 26.1.1.

### B4-G32 (minor, 26.1.1 REPRODUCES) — sign click actions never run

`sign_edit.rs:154-175` gates click commands, dialogs and custom actions on
`allow_op_features`, showing "Click actions are disabled" when false (`:183`).
`SignData` (`horizon-client-render/src/chunk/block_entity.rs:509-516`) derives
`Default`, so `allow_op_features` defaults to **false**; it is loaded from the
save tag only (`signs.rs:222-223`) and saved only when true (`:181-182`).
Vanilla 26.1.2 saves never carry the tag, so in practice no sign ever runs a
click action. Vanilla runs them unconditionally: `SignBlock.useWithoutItem`
calls `executeClickCommandsIfPresent` FIRST (bytecode 80), before the waxed
check (bytecode 87), and `SignBlockEntity` has NO `allow_op_features` field
(dumps saved). Same as 26.1.1.

### B4-G33 (minor, 26.1.1 REPRODUCES, narrowed on LAN) — sign pick/place preserves text and wax

`block_entity_item.rs` maps sign text/wax to `minecraft:sign_text_front`,
`minecraft:sign_text_back` and `minecraft:waxed` components on pick/place
(`:192-193, :231-232, :251, :294`). None of those components exists in 26.1.2
— vanilla signs lose their text and wax when picked up. 26.1.2 DELTA FACET:
the protocol layer changed — `horizon-protocol/src/item.rs` no longer treats
the synthetic sign-text kinds (-1001/-1002) as wire-legal, and `item/skip.rs`
dropped their skip arm, so a picked sign's text components are now dropped at
wire encode (diffs saved in `evidence/diffs/protocol-item*.diff`). The LOCAL
pick/place defect is unchanged (`block_entity_item.rs` byte-identical); only a
LAN round-trip now strips the text. Not a fix — the defect reproduces in
single-player exactly as in 26.1.1.

### B4-G34 (minor, 26.1.1 REPRODUCES) — trial spawner op egg merges entity data

`game/spawner.rs:450-451` calls `override_entity_data` when egg data is
present; `mobs/trial_spawner.rs:405` defines it (merges the data into both
trial-spawner configs). Vanilla `TrialSpawnerBlockEntity` has only
`setEntityId`; 0 `overrideEntityData` refs in its class bytes (dump saved).
The trial-spawner analogue of G20. Same as 26.1.1.

### B4-G35 (minor, 26.1.1 REPRODUCES) — dispensed spawn egg on a null spawn is returned unshrunk

`dispenser.rs:1021-1023` returns the stack unshrunk with no event when the
spawn returns null (`return Ok((Some(stack), CLICK))`). Vanilla
`SpawnEggItemBehavior` (`net.minecraft.core.dispenser` class, dump saved)
does `EntityType.spawn(...)` → `pop` → `shrink(1)` →
`gameEvent(ENTITY_PLACE)` → return stack: a null spawn still shrinks the egg
and fires the event. Same as 26.1.1.

### B4-G36 (minor, 26.1.1 REPRODUCES) — brewing-stand save is the 26.3 form

`block_entity_nbt.rs:514-516` and `save.rs` write int `BrewTime` +
`total_brew_time` + int `Fuel` + `total_fuel` + float `speed_multiplier`.
Vanilla `BrewingStandBlockEntity.saveAdditional` writes `BrewTime` short +
`Fuel` byte only; the class's only fields are `brewTime`/`fuel` (dump saved).
Same as 26.1.1.

### B4-G37 (minor, 26.1.1 REPRODUCES) — sculk growth is never inhibited

`level/sculk.rs:620` reads the `minecraft:sculk_growth_inhibitors` tag, which
does not exist in 26.1.2; `sculk_in_tag` (`:103-104`) returns false when the
tag is absent, so the ≥3 inhibitor count never fires and sensors/shriekers
pile up without limit. Vanilla `SculkBlock.canPlaceGrowth` hardcodes
`Blocks.SCULK_SENSOR`/`SCULK_SHRIEKER` and returns false when the count in the
(-4,0,-4)..(4,2,4) box exceeds 2 (dump saved). Cross-reference B6-G07 at
merge (worldgen-side twin); do not merge the two — different code paths,
different effects. Same as 26.1.1.

## 26.1.1 re-check verdicts (62 items)

| id | verdict | note |
|---|---|---|
| G01, G02, G04, G05, G08, G09, G10, G13, G14, G15, G16, G18, G20, G21, G22, G23, G24, G25, G26, G27, G28, G29, G30, G31, G32, G33, G34, G35, G36, G37 | REPRODUCES (30) | §Behaviour defects above |
| O03, O04, O05, O06, O08, O09, O10, O14, O15, O16, O18, O19, O21, O22, O23, O25, O26 | REPRODUCES (17) | §Oracle defects below; identical error counts/first errors as 26.1.1 |
| G19 | FIXED (still) | dispenser wax sound now plays once — see below |
| G03, G06, G07, G11, G12, O01, O02, O07, O11, O12, O13, O17, O20, O24 | CLOSED (still, 14) | premise still fails on 26.1.2 evidence — see below |

**B4-G19 (dispenser wax sound twice) — FIXED, still.** `level_particles.rs`
routes 3003 to `ParticleType::WaxOn` only (particles, no sound) and
`game.rs:4367-4370` routes 3002..=3005 to `block_particle_event`; the catch-all
sound table in `game.rs` has no 3003 entry — so the dispenser's explicit
`audio.play(wax_on)` is the only play. Sound plays exactly once. Both files
byte-identical to 26.1.1.

**B4-G17 (smoker/blast burn-time halving) — UNVERIFIED, still, not FIXED.**
Carried from the 26.1 run via the 26.1.1 report: the 26.1.1 `burn_time` threads
`fast` through the provider eval (`providers.rs:48-77`, `fast_cooking`
condition), so the ignore-`fast` code is gone — but with G01, NO fuel burns at
all, so the halving behaviour is unobservable and undecidable. Arbitration for
the coordinator: restore fuel data (fix G01), then check halving via the fast
context. No slice item (there is no "unverified" status); recorded here only.

## 1.21.7 re-check verdicts

**REPRODUCES (behaviour, 11):** G01, G02 (majors, regressed from 26.1 FIXED),
G04, G05, G08, G09, G10, G13 (reduced form — the bed BE is still missing),
G14, G15, G16. All detailed in §Behaviour defects. (These are the same items
as the 26.1.1 re-checks; not double-minted. The 26.1.1 report's "10" was a
miscount of its own list of 11.)

**WITHDRAWN (behaviour, 5 — vanilla 26.1.2 differs from 1.21.7, each
javap-verified; land as closed items):**

- **G03** — vanilla grows furnace output by the FULL result count
  (`burn()` grows output by `result.getCount()`, bytecode 29–34).
- **G06** — vanilla does NOT reset the cooking timer on recipe-less input (the
  null-recipe check at bytecode 154–156 jumps to 333, past all timer resets).
- **G07** — no `defaultType` on `SpawnEggItem`; an egg whose `entity_data` was
  removed drops.
- **G11** — `spawner_blocks_work` EXISTS and is checked in both spawner paths
  (`GameRules.SPAWNER_BLOCKS_WORK` at `net.minecraft.world.level.gamerules.
  GameRules`; `BaseSpawner.serverTick` calls `ServerLevel.
  isSpawnerBlockEnabled()`; `TrialSpawner.canSpawnInLevel` checks it first,
  then `overridePeacefulAndMobSpawnRule`, then `PEACEFUL` + `SPAWN_MOBS`).
- **G12** — creative enchanting without lapis is allowed
  (`EnchantmentMenu.clickMenuButton`: the lapis `isEmpty`/count check at
  bytecode 56–71 is bypassed by `hasInfiniteMaterials` at 74–78, same as
  26.1.1).

**WITHDRAWN (oracle, 9 — the oracle now passes; land as closed items):** O01,
O02, O07, O11, O12, O13, O17, O20, O24. (26.1.2 Batch 0 table: all pass on
`both`; O17 passes on client+both while the server side fails on the client
package only — expected, not breakage, per the 26.1 precedent.)

**Carried over (oracle, 17):** O03, O04, O05, O06, O08, O09, O10, O14, O15,
O16, O18, O19, O21, O22, O23, O25, O26. §Oracle defects below.

**FIXED in the 26.1.2 tree:** none of the 1.21.7 behaviour items — G01/G02
were fixed in 26.1 and regressed in 26.1.1, and 26.1.2 inherits the regression.
(Note: G01/G02 were also fixed in the 1.21.7 tree itself after its sweep —
furnace fuel table a53b0c58, brewing fuel b4bfc2cd — but the 26.1.2 tree does
not carry the equivalent: 26.1.2 has no fuel component, and its `FuelValues`
form is the hardcoded table, which the port does not read.)

## Oracle defects

Error counts and first errors from the 26.1.2 Batch 0 table (identical to the
26.1.1 table on every oracle); each cause verified fresh with javap against the
26.1.2 jar and each cited oracle line read in the 26.1.2 tree's oracle source.
17 oracle defects, same set as 26.1.1.

- **B4-O03 (minor) — VanillaFurnaceOracle**: cascade on the broken
  VanillaBlockUpdateOracle (Batch 2's item; 32 errors, first `450: cannot find
  symbol`, in the 26.1.2 table). Blocks `test_crafter`, `test_furnace`,
  `test_vault`.
- **B4-O04 (minor) — VanillaBrewingOracle**: 14 errors, first `23: cannot find
  symbol`. Line 23 of the 26.1.2 oracle file is
  `import net.minecraft.world.item.crafting.BrewingInput;` — `BrewingInput`
  is absent from the 26.1.2 jar (0 matches). The oracle is the 26.3 form
  (`BrewingRecipe`, `RecipePropertySet`, `DataComponents.BREWING_FUEL`,
  totalBrewTime/totalFuel/speedMultiplier). Blocks `test_brewing`.
- **B4-O05 (minor) — VanillaBrewingFuelOracle**: 8 errors, first `13: cannot
  find symbol`. Line 13 is
  `import net.minecraft.world.item.component.BrewingFuel;` — absent from the
  jar (0 matches). The oracle is the 26.3 form (`DataComponents.BREWING_FUEL`,
  `getUses`/`getSpeedMultiplier`). Blocks `test_brewing_fuel`.
- **B4-O06 (major) — VanillaDispenserOracle**: 15 errors,
  `212: package EntityTypes does not exist`. `EntityTypes` is absent from the
  jar (0 matches); line 212 uses `EntityTypes.PLAYER`. Blocks `test_dispenser`.
- **B4-O08 (major) — VanillaMenuOracle**: 4 errors, `138: cannot find symbol`
  (`Prediction`). `net.minecraft.util.Prediction` is absent from the jar (0
  matches); line 138 declares a `drop` override taking a `Prediction`.
  Blocks `test_menu_click`. (26.1 dedup: B4-O08→B10-O05 at merge — see
  coordinator notes.)
- **B4-O09 (minor) — VanillaRecipeBookOracle**: cascade on MenuOracle. Blocks
  `test_recipe_book`.
- **B4-O10 (minor) — VanillaPlaceRecipeOracle**: cascade on MenuOracle. Blocks
  `test_place_recipe`.
- **B4-O14 (minor) — VanillaSculkOracle**: 7 errors, `327: cannot find symbol`
  (`datapackWorldRegistries`). `WorldLoader$DataLoadContext` has
  `datapackWorldgen()`/`datapackDimensions()` only — no
  `datapackWorldRegistries()`; line 327 calls it. Blocks `test_sculk`.
- **B4-O15 (minor) — VanillaBlockEntityAnimOracle**: 3 errors; the fake level
  (`World extends Level`, line 102) lacks a `fuelValues()` override.
  `Level.fuelValues()` is abstract in the jar. Blocks `test_block_entity_anim`.
- **B4-O16 (minor) — VanillaBlockEntityRenderOracle**: 9 errors, `UvMapping`
  (line 23 imports `net.minecraft.client.renderer.texture.UvMapping`; the
  class is absent from the client jar). Same defect as B10-O03 — dedup at
  merge, keep B10 canonical (26.1 record). Blocks `test_block_entity_render`.
- **B4-O18 (minor) — VanillaRecipeBookScreenOracle**: 4 errors, renderpearl
  package (line 4 imports `com.mojang.renderpearl.api.pipeline.RenderPipeline`;
  0 renderpearl entries in the client jar). One of five oracles under B10-O05
  — dedup at merge (26.1 record). Blocks `test_recipe_book_render`.
- **B4-O19 (minor) — VanillaEnchantDurabilityOracle**: cascade on
  BlockUpdateOracle. Blocks `test_enchant_durability`.
- **B4-O21 (minor) — VanillaVehicleContainerOracle**: 3 errors,
  `EntitySpawnRequest` (line 20 imports it). The class is absent from the jar
  (0 matches). Blocks `test_vehicle_container`.
- **B4-O22 (minor) — VanillaShulkerSpaceOracle**: 2 errors, `EntityTypes`
  (line 10 imports `net.minecraft.world.entity.EntityTypes`). Blocks
  `test_shulker_space`.
- **B4-O23 (major) — VanillaNaturalSpawnOracle**: 16 errors, `191: method does
  not override or implement a method from a supertype`. Line 191 overrides
  `getNextEntityId()`; `ServerLevel` has 0 `getNextEntityId`. Blocks
  `test_spawner`, `test_trial_spawner`.
- **B4-O25 (minor) — VanillaTradingOracle**: 6 errors,
  `datapackWorldRegistries` (line 254, same cause as O14). Blocks
  `test_trading`. Batch 3 overlap — cross-reference.
- **B4-O26 (minor) — VanillaGhostRecipeOracle**: 1 error, 26.3 ContextMap API
  (line 56: `ContextMap.builder().set(...).buildAndValidate(...)`).
  `net.minecraft.util.context.ContextMap` has no `builder()`/`set`/
  `buildAndValidate` (only `has`/`getOrThrow`/`getOptional`/`getOrDefault`).
  Blocks `test_ghost_recipe`.

No new in-scope oracles appeared in the 26.1.2 table: the oracle set is
identical to 26.1.1's (391 oracles, 0 added, 0 removed). The only unlisted
name-matches are VanillaFoodStorageOracle (Batch 7, passes),
VanillaLoomScreenOracle (Batch 10, renderpearl) and VanillaShulkerBulletOracle
(Batch 3) — all out of scope, as in 26.1.1.

## Oracle defects withdrawn (9)

The oracle now passes in 26.1.2 (26.1.2 Batch 0 table, `both: pass`); each
lands as a closed item. O17 (VanillaContainerScreenOracle) and
RecipeBookViewOracle pass on client+both while the server side fails on the
client package only — expected, not breakage, per the 26.1 precedent.

- O01 Ticker, O02 Storage, O07 Enchanting, O11 SimpleContainer, O12
  ItemStackStorage, O13 PalettedContainer, O17 ContainerScreen, O20
  StoredEnchantmentPatch, O24 Recipe.

## What passed

**Drivers with a passing oracle (17)** — declared pass statically per the
26.2 precedent: `test_block_entity_tickers`, `test_block_entity_storage`,
`test_enchanting`, `test_simple_container`, `test_item_stack_storage`,
`test_paletted_container`, `test_container_screen`,
`test_stored_enchantment_patch`, `test_recipe`, `test_chest_load`,
`test_chest_pair`, `test_container_title`, `test_enchantment_components`,
`test_storage_menu`, `test_recipe_book_view`, `test_container_component_names`,
`test_trade_components`.

**Drivers with no oracle (13 pass of 20)** — verdict from reading the driver's
assertions against the 26.1.2 port code and vanilla bytecode. The drivers
(`tools/`) are byte-identical to 26.1.1's and the port code they exercise is
byte-identical; vanilla is unchanged for every class they touch (none is among
the 6 changed classes):

- `test_crafter_drop_name`, `test_falling_chest`,
  `test_jukebox_saved_clock`, `test_sculk_saved_fields`,
  `test_enchantment_map_validation`, `test_world_pack_enchantment_loader`,
  `test_anvil_turtle_egg`, `test_crafting_priority`, `test_craft_map_take`,
  `test_end_gateway` — spot-checked for fuel/brewing/sign exposure: none.
  The underlying files are 0-diff vs 26.1.1 and vanilla is unchanged.
- `test_sign_literal` — tests sign TEXT via same-level clone, not pick/place
  item components; the clone path copies the BE directly and is unaffected by
  G33. (The 26.1.2 wire change to sign-text components does not touch the
  clone path either.)
- `test_machine_recipe_placement` — re-verified fresh: the driver freezes
  ticks (`tick freeze`, `observation_ticks=0`), so no cooking happens in
  either implementation; the driver tests recipe placement (menu transfer),
  not fuel burn. The coal in the fuel slot is inventory content only.
- `test_recipe_book_variants` — compares server-sent recipe-book PACKET
  entries (add/remove ids); the AnyFuel grouping is client-side UI rendering
  and does not change packet content.

**Method-2 checks (17 pass of 38):** lectern, comparator, anvil, grindstone,
smithing, loom, stonecutter, vault, chiseled bookshelf, jukebox, campfire,
beehive, shulker box, sculk save fields, falling-block BE restore, container
titles/names, enchanting offers/click. Carry argument, stated explicitly: the
port files behind all 17 areas are byte-identical to 26.1.1's (verified file
by file — `stations.rs`, `menu.rs`, `vault.rs`, `campfire.rs`, `beehive.rs`,
`shulker.rs`, `jukebox.rs`, `chiseled_bookshelf.rs`, `loom.rs`,
`stations/anvil.rs`, `level/redstone.rs`, `containers.rs`, `sculk.rs`,
`save.rs` except the DataVersion/palette lines), and vanilla 26.1.2 is
unchanged for every class they exercise (the 6 changed classes are
handleAttack/Checkbox/AbstractReportScreen/DetectedVersion/SharedConstants).
Spot-verified fresh: enchanting click (`EnchantmentMenu.clickMenuButton`
dump — the lapis/creative logic matches the port) and the anvil/vault/campfire
file identities. (Comparator: the `redstone.rs` file is byte-identical — the
`DiodeBlock.neighborChanged` guard is Batch 2's scope and unchanged here.)

## Suspicions checked and refuted

- **Sculk water-gate narrowing (Batch 6 flag, second half) — REFUTED, with a
  cross-batch disagreement (carried from 26.1.1, re-verified).** Vanilla
  `SculkBlock.canPlaceGrowth` requires `isAir() || (is(WATER) &&
  fluidState.is(Fluids.WATER))` (dump saved). B6 reads
  `fluidState.is(Fluids.WATER)` as true for flowing water; it is not.
  `TypedInstance.is(T)` is a type-identity check: a flowing-water state's type
  is `Fluids.FLOWING_WATER`, not `Fluids.WATER`, so the gate is source-only in
  vanilla — and the port's `is_source()` check at `level/sculk.rs:611`
  MATCHES it. B6's water-gate item rests on the misreading; the coordinator
  should arbitrate — the B6 item may need withdrawal. (The inhibitor half of
  B6's flag is confirmed as B4-G37.)
- **Dispenser shears caller-level alive check — REFUTED (observationally
  equivalent).** `ShearsDispenseItemBehavior.tryShearEntity` has NO
  caller-level alive check, but each species' `readyForShearing()` checks
  `isAlive()` (MushroomCow: isAlive + !isBaby; Sheep: isAlive + !isSheared +
  !isBaby; SnowGolem: isAlive + hasPumpkin; Bogged: !isSheared + isAlive).
  The port's caller-level check changes no outcome. (`dispenser.rs`
  byte-identical to 26.1.1.)
- **Sign wax-on sound playing twice — REFUTED.** The 3003 level event plays no
  sound in this tree (particles only); the dispenser's/sign's explicit play is
  the only one. (Same finding as the G19 fix, on the sign side.)
- **Spawn-egg peaceful gate as a regression — REFUTED.** The port's
  `spawn_egg_allowed` gate (`creatures.rs:7511-7520`, file byte-identical)
  looks like a 26.3-form addition, but vanilla 26.1.2 HAS the gate:
  `SpawnEggItem.spawnMob` checks `EntityType.isAllowedInPeaceful()` +
  `Difficulty.PEACEFUL` (dump saved in `evidence/javap/SpawnEggItem.txt`).
  Spawn eggs are Batch 7's scope — flagged to B7 with the jar evidence; NOT
  minted here.

## Checks by driver class

**Drivers with a passing oracle (17 pass, 0 gap):** see §What passed.

**Drivers oracle-blocked (0 pass, 20 gap):** `test_brewing_fuel` (O05),
`test_dispenser` (O06), `test_menu_click` (O08), `test_recipe_book` (O09),
`test_sculk` (O14), `test_block_entity_anim` (O15),
`test_block_entity_render` (O16), `test_recipe_book_render` (O18),
`test_place_recipe` (O10), `test_trading` (O25), `test_ghost_recipe` (O26),
`test_crafter` (O03), `test_furnace` (O03), `test_brewing` (O04),
`test_enchant_durability` (O19), `test_vehicle_container` (O21),
`test_shulker_space` (O22), `test_spawner` (O23), `test_trial_spawner` (O23),
`test_vault` (O03).

**Drivers with no oracle (13 pass, 7 gap):**

- Gap: `test_merchant_gate` (G25), `test_merchant_crafted` (G25),
  `test_loot_mine_pot_components` (G28), `test_pot_sherd_templates` (G28),
  `test_furnace_loaded_total` (G04), `test_pot_hopper` (G15),
  `test_furnace_output_components` (G01 — its corpus smelts with coal fuel,
  re-verified fresh in `furnace_output_components_corpus.py:34,62-64`, which
  never burns in 26.1.2).
- Pass: see §What passed.

**Method-2 checks (17 pass, 21 gap):**

- Pass (17): lectern, comparator, anvil, grindstone, smithing, loom,
  stonecutter, vault, chiseled bookshelf, jukebox, campfire, beehive, shulker
  box, sculk save fields, falling-block BE restore, container titles/names,
  enchanting offers/click.
- Gap (21): decorated-pot use (G28), merchant pricing (G21/G23/G24/G25),
  sign NBT (G27), furnace tick (G01/G04/G17), spawner tick (G20), cartography
  (G16), dispenser/dropper (G18/G35), hopper (G08), ender chest (G14), save
  round-trip BE (G15/G27/G28/G36), BE coverage (G13/G15), crafter
  (G09/G10/G29), container loot unpacking (G26), beacon (G31), brewing recipes
  (G02), brewing tick (G02), trial spawner tick (G34), menu engine
  (G01/G02 — the furnace fuel and brewing-fuel slot arms refuse every item),
  composter (G30), sculk growth inhibition (G37), sign interaction (G32/G33).

## Notes for the coordinator

1. **Null delta — and it was verified, not assumed.** Every 26.1.1 verdict
   reproduces in 26.1.2 with fresh evidence (26.1.2 jar dumps, 26.1.2 prepared
   data counts, 26.1.2 Batch 0 table, 26.1.2 tree reads). The slice therefore
   has the same 62 items (30 gap + 17 oracle-defect + 1 fixed + 14 closed) as
   the 26.1.1 slice, with fresh evidence strings. The behaviour-only
   restatement is merge.py's job; the as-tallied count (95) and the split
   (37 driver-status / 58 behaviour) are both stated in §Tally.
2. **Dedup (carried from the 26.1 record):** B4-O16→B10-O03 (UvMapping),
   B4-O18→B10-O05 (renderpearl), B4-O08→B10-O05, B4-G20→B7-G17,
   B4-G26→B12-G01, B4-G18→B5-G19, B3-G24→B4-G24. B4-G37 cross-references
   B6-G07 (worldgen-side twin) but is a separate item — different code path.
   B4-O25 overlaps Batch 3 (datapackWorldRegistries).
3. **Cross-batch disagreement — B6 water gate.** Unchanged from 26.1.1: B6's
   sculk water-gate item rests on `fluidState.is(Fluids.WATER)` being true
   for flowing water. Javap of `SculkBlock.canPlaceGrowth` + `TypedInstance.is`
   proves it is a type-identity check (flowing water's type is
   `FLOWING_WATER`), so vanilla's gate is source-only and the port matches.
   B6's item may need withdrawal. Evidence:
   `evidence/javap/SculkBlock.txt` (canPlaceGrowth at line 185).
4. **G17 arbitration.** B4-G17 is UNVERIFIED (masked by G01) — no slice item.
   Restore fuel data first, then check halving via the fast context.
5. **26.1.1-record corrections found by this run:**
   - G25: the mechanism is zeroing, not truncation (`serde_json::Value::
     as_i64()` returns `None` for float JSON numbers; `unwrap_or(0)` zeroes
     them). The 26.1.1 title says "truncates"; the title is kept for
     continuity, the evidence now states the precise mechanism.
   - G25: the "68 trade-set files with float amount" sub-claim does not
     reproduce from the jar (387 trade files, one trade each, 0 `amount`
     keys). Core float counts (218/0/371/266/41/0) match exactly.
   - G22: the defect is a missing `isExplorationMap` exclusion, not a "wrong
     predicate".
   - The 26.1.1 report's 1.21.7 re-check count "10 behaviour REPRODUCES" was
     a miscount of its own list of 11; this report says 11.
6. **26.1.2 delta facet — G33 narrowed on LAN only.** The protocol layer now
   drops synthetic sign-text components at wire encode (`item.rs`, `skip.rs`
   diffs saved). Local pick/place still preserves text/wax; only a LAN
   round-trip strips them. Not a fix; G33 reproduces.
7. **Out-of-scope diffs found while diffing — flag, do not mint:**
   - `game/stations.rs` crossbow charged-projectiles tooltip (multiple line
     uses the plain name; 26.1 used the `[name]` display name for both) →
     Batch 7. (File byte-identical to 26.1.1; the flag carries.)
   - `game/commands/items.rs` (970 lines, `/item` command slot machinery) →
     Batch 10 (commands). (Byte-identical.)
   - `horizon-world/src/level/redstone.rs` `DiodeBlock.neighborChanged`
     guard added (26.2 form) → Batch 2. (Byte-identical.)
   - `creatures.rs` `spawn_egg_allowed` peaceful gate (likely the B7-G04
     fix; vanilla 26.1.2 HAS the gate) → Batch 7, with the jar evidence in
     `evidence/javap/SpawnEggItem.txt`. (Byte-identical.)
   - `dialog_screen.rs` Checkbox row-limit tooltip (26.1.2 delta item #6) →
     Batch 10. Verified Checkbox-only; no Batch 4 surface touched.
   - `ServerGamePacketListenerImpl.handleAttack` changed method body
     unported (26.1.2 delta item #4) → Batch 3 (combat). Not a Batch 4
     surface; noted because three B4 oracles reference the listener class
     (superclass/import only — no oracle effect, table identical).
8. **Nothing was run live.** All 95 checks are static (driver-status from
   the 26.1.2 Batch 0 table, no-oracle and method-2 from code + javap + data
   counts). The 17 "passing" drivers are declared pass per the 26.2
   precedent, not by execution. The brewing `stations.rs:1857` test expecting
   279 recipes would fail against the prepared 26.1.2 data — a CI matter, not
   verified here.
9. **Evidence:** `b04/evidence/` — javap dumps in `javap/` (33 files,
   including `FuelValues.txt`, `AbstractFurnaceBlockEntity.txt`,
   `SculkBlock.txt`, `SignBlock.txt`, `Villager.txt`,
   `BrewingStandBlockEntity.txt`, `DataComponents.txt`, `GameRules.txt`,
   `SpawnEggItem.txt`), tree diffs in `diffs/` (whole-tree list + the four
   in-scope file diffs), jar-data extracts in `jardata/` (387 villager-trade
   files, 1,326 loot tables, brewing-fuel tag), jar class scans in `jarscan/`.
10. **Batch 3 flag — `game/stations.rs` brewing-totals change: VERIFIED, no
    new item (carried from 26.1.1).** The change is real (render bars read
    `total_fuel` / `total_brew_time`; tick takes `providers()`), but it is a
    facet of G02 (component fuel + data-driven recipes) and G36 (26.3 save
    form): the totals default to vanilla's constants (400/20/1.0) and the
    mutating paths are unreachable without the absent `brewing_fuel`
    component, so the render change has no independently observable divergence.
    B3's companion note is also confirmed: `test_brewing_parity.py:129` reads
    26.3 resources where the 26.1 tree reads 26.1's — latent driver defect,
    moot while O04 blocks the driver; it will matter when the oracle is fixed
    (the driver's corpus generator reads `data/minecraft/recipe/brewing/*.json`
    from that path, so it would build a 26.3 recipe corpus for a 26.1.2 run).

## Verdict

The 26.1.2 port in Batch 4 scope is the 26.1.1 port, unchanged: 95 checks
(47 pass, 48 gap as-tallied; 58 behaviour, 30 pass / 28 gap), 62 items
(30 behaviour gap, 17 oracle-defect, 1 fixed, 14 closed), 8 live majors.
The two re-check-first majors — furnaces never light (G01) and brewing stands
never brew (G02) — both REPRODUCE with fresh 26.1.2 evidence. The systemic
cause is the same component/tag-read regression the 26.1.1 run found: the tree
reads eight data components and two tags that do not exist in vanilla 26.1.2,
and the 26.1-form sources (FuelValues table, brewing-fuel tag + PotionBrewing,
hardcoded composter/sculk tables) were not restored. Nothing in the 26.1.2
delta (DataVersion bump, handleAttack, Checkbox tooltip) touches this scope.
No new defects were found beyond the inherited set; three 26.1.1-record
refinements (G25 zeroing, G25 trade-file count, G22 missing exclusion) and one
LAN-only narrowing (G33 wire drop) are recorded above for the coordinator.
