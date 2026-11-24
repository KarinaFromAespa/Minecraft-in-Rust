# Batch 12 — Data-driven registries (26.1)

Checks + triage only, 2026-11-24. Scope: recipes (1,515), loot tables
(1,326), advancements (1,617, triggers only), tags (758), enchantments
(43), status effects (40), damage types (50). Method 4 (PARITY-SWEEP.md
§3): byte-level diff of the port's prepared data against the vanilla
26.1 data files, plus behaviour checks of the loaders — here, where the
*loader* disagrees with the data's form even though the data itself is
faithful.

Vanilla spec: the named 26.1 game jars
(`26.1.x/versions/26.1/jars/{client,server-game}.jar`, Mojang-mapped;
`data/minecraft/*` extracted to the batch scratch dir), `javap -c` on
the client jar's `MobEffects` class for effect colours/categories/
instant flags, the 26.1 datagen reports for the inventory counts, and
`javap -c` on the cooking-recipe classes for the cook-time defaults.
Port data: `versions/26.1/.local/horizon-client/26.1/data/minecraft/`
(prepared data; Batch 0's prepare had landed before this batch started).
Loot behaviour: the existing `loot_fill` example, built once and run
against the real prepared data. NOTE: `sounds.json` is absent from the
prepared tree (needs the separate `horizon-prepare sounds` step against
a launcher asset store the VM lacks) — expected, not a port defect, and
no check in this batch needed it. `en_us.json` IS present
(`assets/minecraft/lang/en_us.json`) and md5-identical to the client
jar's file (`98deafdb5e90e4438db30853494e1aeb`).

**Headline:** the prepared data is faithful — every one of the 5,309
data-driven files is byte-identical to the 26.1 jar — but BOTH loot
engines are written for the 26.3 key forms and 26.1 writes the old
forms, so the 1.21.7 loot defects (B12-G01/B12-G02) reproduce in full
in 26.1. The 26.2 playtest fixes that repaired both engines in the
26.2 tree were never carried to 26.1: the 26.2 tree reads
`["functions"]` + dual-form discriminators everywhere
(`items.rs:1969-1970/2246/2387`, `loot.rs:1386/1507/1534/2040/2042/
2053/2097/2153`, plus the `tag_entry_items` name fallback at
`loot.rs:1042`), while the 26.1 tree reads `["modifier"]` and
`["type"]`/singular `["condition"]` only. Verified by direct
comparison against `versions/26.2/` in the same worktree.

## Tally

| Registry | Data diff (byte-level) | Behaviour check | Result |
|---|---|---|---|
| Recipes (1,515) | 1,515/1,515 identical | Loadability per recipe (parser-arm emulation incl. tag resolution): 1,515 load, 0 dropped; 21/21 recipe types have a parser arm | pass |
| Loot tables (1,326) | 1,326/1,326 identical | items.rs × 1,326 tables: 134 pass / 1,192 gap; loot.rs run (tables already counted under items.rs, not double-counted): conditions/functions ignored throughout | **1,192 gap checks** (B12-G01, B12-G02) |
| Advancements (1,617) | 1,617/1,617 identical | 54 distinct triggers: 53 fired, `impossible` never fires by vanilla design | pass |
| Tags (758) | 758/758 identical | Runtime loaders read the same files; `block-tags.json` 248/248 exact-equal; `supports_*` tags exist natively in 26.1 | pass |
| Enchantments (43) | 43/43 identical | Loader reads all 26.1 fields; all 30 effect component keys wired (data-driven or by enchantment id) | pass |
| Status effects (40) | n/a — code-defined in 26.1, no `mob_effect/` data files in the jar | 40/40 colours, categories, instant flags, protocol order match the jar; `breath_of_the_nautilus` is real in 26.1 | pass |
| Damage types (50) | 50/50 identical | Loader validates all fields; every value in the files is accepted | pass |

Checks total: **8,593** (5,309 data-diff entries + 3,284 behaviour
checks). Pass: **7,401**. Gap: **1,192** — collapsing to **4 defects**,
2 major (B12-G01, B12-G02) and 2 minor (B12-G03, B12-G04, both
datapack-scope, 0 gap checks each). Behaviour subtotal reconciles as
1,515 recipe loads + 21 recipe-type arms + 1,326 items.rs tables + 54
triggers + 248 block-tags + 30 enchantment keys + 40 effects + 50
damage types = 3,284; the loot.rs run covers tables already counted
under items.rs and is not double-counted.

Counting rule: a table counts as a gap check for items.rs when it
carries ≥1 condition (the engine evaluates every object condition as
false, so conditioned pools/entries are dropped) or ≥1 function (the
engine never reads the `functions` arrays, so no function runs).
Tables with neither (134) roll correctly through the engine and pass.
The loot.rs defect is recorded as its own item with 0 additional gap
checks, per the settled no-double-count rule.

No oracle defects, no oracle repairs: this batch is driver-independent
(javap + data diffs + the `loot_fill` example run). No fixes made.

## The systemic pattern

The prepared data is faithful — all 5,309 data-driven files
byte-identical to the 26.1 jar, and `block-tags.json` byte-faithful
too. The 1.21.7 batch's systemic finding (code reading the 26.x form
of formats the version writes differently) DOES carry to 26.1, for the
same reason it carried to 1.21.7: **26.1 writes the old form**. Its
loot JSONs use the `condition` discriminator (1,545 occurrences, 0
`type`), the `function` discriminator (1,404, 0 `type`), plural
`conditions` arrays (1,389), `functions` arrays at entry/pool/table
level (964/10/9 = 983), and the `name` field on tag entries — while
both engines in the 26.1 tree read the 26.3 forms (`type`,
`modifier`, singular `condition`, tag `items`). The `type`/`modifier`
renames are 26.3 changes, not 26.1 ones, and the 26.2 playtest fixes
that taught the engines both forms were never carried to the 26.1
tree. The load path does no normalisation (`loot.rs` `load_dir`,
lines 773-790, parses each file straight into the shared map), so the
raw old-form JSON is exactly what the engines roll. Everything else
in this batch's scope is clean: the specials are already data-driven
(post-E6 form), the effect table legitimately has 40 entries, and the
recipe/advancement/tag/enchantment/damage loaders all read 26.1's
forms correctly.

## Defects

### B12-G01 — MAJOR: items.rs evaluates every loot condition as false and never runs any loot function; survival block-breaking drops nothing from conditioned tables

`horizon-items/src/items.rs`, two key-form defects in `LootContext`:

1. **Conditions.** `condition()` (line 2070) reads the kind ONLY from
   `o.get("type")` (line 2075) — there is no
   `.or_else(|| o.get("condition"))` dual-form read (contrast the
   26.2 tree, `items.rs:2089-2090`). 26.1's data uses the `condition`
   key throughout (1,545 occurrences, 0 `type`-keyed), so every
   object-form condition's kind is `""` and the match falls to
   `_ => false`. `conditions()` (line 2196) DOES consult the plural
   `conditions` array — every element fails. Nested conditions fail
   the same way (`inverted`'s `term`, `any_of`/`all_of`'s `terms`).
2. **Functions.** `evaluate()` applies `pool["modifier"]` and
   `table["modifier"]` (lines 1957-1958), `expand()` pushes
   `e["modifier"]` (line 2230), `nested()` applies
   `entry["modifier"]` (line 2371) — but 26.1's data has ZERO
   `modifier` keys anywhere and puts functions in `functions`
   arrays (983: 964 entry-level, 10 pool-level, 9 table-level).
   `modifiers()` (line 2385) itself HAS the function-discriminator
   dual-form fix (`f["type"].as_str().or(f["function"].as_str())`,
   line 2395) — dead code, because the container key is never read.
   No function ever runs through this engine on 26.1 data.

Impact, counted over all 1,326 tables: **1,192 tables** carry ≥1
condition or ≥1 function — blocks 1,052/1,085 (1,023 with a
condition, 225 with a function), entities 62/108, chests 54/56,
gameplay 6/26, dispensers 3/3, equipment 3/3, spawners 3/5,
shearing 3/22, harvest 2/3, archaeology 1/6, carve 1/1,
charged_creeper 1/6, pots 1/1. Only 134 tables (neither conditions
nor functions) roll correctly.

Survival impact is direct: `Game::block_loot_at`
(`game.rs:5084`) → `block_drops_by_player` (`game.rs:3697/3699`,
`items.rs:1646`) → `block_drops_raw` → `ctx.evaluate(table,
random)` (`items.rs:1755`). Every pool with a condition is skipped
in `evaluate()` and every entry with a condition is rejected in
`expand()`, so a block whose table is fully conditioned drops
nothing — the same symptom the owner confirmed in-game for 1.21.7
("survival block-breaking drops nothing"). 1,023 of 1,085 block
tables carry ≥1 condition. `/loot` routes every table type through
this engine too (`loot_table_for_command`, `items.rs:1772`;
`entity_drops_for_command` / `entity_drops_with_target`,
`items.rs:1877/1889`; `block_drops_by_player`), so command rolls
lose conditions and functions alike.

**Post-fix residual (recorded here, not a separate item):** once the
key forms are fixed, the 26.2-style partial-function-set defect
surfaces. `modifiers()` has 12 arms (`set_count`, `set_damage`,
`apply_bonus`, `enchanted_count_increase`, `copy_state`,
`explosion_decay`, `set_potion`, `enchant_with_levels`,
`enchant_randomly`, `limit_count`, `set_name`, `set_components`;
default `_ => {}`, line 2591) and is missing 7 of the 19 function
kinds 26.1's data uses: `copy_components` (71 tables, all blocks),
`furnace_smelt` (16, entities), `set_enchantments` (4: 3 equipment,
1 chest), `set_stew_effect` (3: 1 archaeology, 2 chests),
`exploration_map` (3, chests), `set_ominous_bottle_amplifier` (3: 2
chests, 1 entity), `set_instrument` (1, chest) — union 101 tables.
`expand()` (line 2211) has no `minecraft:tag` arm (default
`_ => false`), losing the single tag entry
(`entities/creeper.json`, the music-disc entry). Union + tag =
**102 tables** — exactly the 26.2 run's B12-G01 count, the same
defect waiting underneath. (Note: 26.1's `modifiers()` DOES have a
`set_name` arm, which 26.2's lacked; the missing set is otherwise
identical.) The `spill.rs` shield noted for 26.2 (block-entity
components copied by gameplay code for the 71 `copy_components`
tables) does not apply here: with conditions failing, `evaluate()`
returns no stacks for `spill.rs` to attach to.

This item is the canonical loot-engine item for the merge: Batch 4's
B4-G26 (loot keys: 983 `functions`, 0 `modifier`, 1,545 `condition`)
folds into it — all three of B4-G26's numbers are verified exactly
by this batch's census, and the per-table impact counts live here.

### B12-G02 — MAJOR: the loot.rs engine ignores all loot conditions and all loot functions

`horizon-world/src/loot.rs` is written for the 26.3 loot format
throughout, and 26.1 writes the old form at every level:

- `condition()` (line 1368) reads the kind from `c["type"]` only
  (line 1377) — 26.1's 1,545 conditions all use the `condition` key.
- `modify()` (line 1478) consults singular `m["condition"]` only
  (line 1488) — 26.1's 184 function-level condition lists are all
  plural `conditions` arrays.
- `run()` (line 1512) reads the kind from `f["type"]` only (line
  1514) — 26.1's 1,404 functions all use the `function` key; the
  default arm `_ => {}` (line 1927) no-ops anything it does see.
- `table_raw()` (line 2011) checks singular `pool["condition"]`
  (line 2018) and applies `pool["modifier"]` / `table["modifier"]`
  (lines 2029, 2016) — 26.1 has 958 plural pool `conditions`
  arrays and 0 `modifier` keys.
- `expand()` (line 2070) checks singular `entry["condition"]`
  (line 2071) — 26.1's 247 entry-level condition lists are all
  plural, so every entry passes its gate.
- `create()` (line 2121) applies `entry["modifier"]` (line 2123) —
  0 in the data, so no entry function ever runs.
- The `minecraft:tag` arm reads `entry["items"]` (lines 2100,
  2136) — 26.1's tag entry uses the `name` field
  (`minecraft:creeper_drop_music_discs`), so `holder_set` gets
  null and returns an empty list (contrast the 26.2 tree's
  `tag_entry_items`, `loot.rs:1042`, which falls back to `name`).

Net: conditions are never evaluated (every gate passes) AND
functions never run (every modifier is a silent no-op). The engine's
arm coverage is otherwise good — `run()` has 26 arms covering 15 of
the 19 function kinds in the data, missing only the block-only kinds
`apply_bonus`, `copy_components`, `copy_state`, `explosion_decay`
(an engine-scope boundary, not a defect: `loot.rs` never serves
block tables in gameplay) — but the arms are unreachable on 26.1
data because of the key forms.

Demonstrated with the existing `loot_fill` example, built once
(`cargo build -p horizon-world --example loot_fill`, dev profile,
`taskset -c 0`, `-j 1`, 3m 35s, EXIT 0 — the target cache was warm)
and run from `versions/26.1/` as `./target/debug/examples/loot_fill
./.local/horizon-client/26.1 <tables> 1 2 3 4 5` (namespaced table
ids — un-namespaced ids look up nothing, since the tables are keyed
`minecraft:…`; the first run returned empty for that reason alone).
All 7 tables, seeds 1–5 (`evidence/loot_fill.log`):

- `chests/simple_dungeon`: every stack count 1 (`set_count`
  uniform 1–5 ignored; 26.2's run showed counts 1–5).
- `chests/ancient_city`: books stay plain `minecraft:book`
  (`enchant_randomly` ignored — 26.2's run showed enchanted books
  with real enchantments), leggings unenchanted
  (`enchant_with_levels` ignored), every count 1.
- `entities/zombie`: `music_disc_lava_chicken` drops on ALL 5 seeds
  — a drop gated on `killed_by_player` + baby-on-chicken
  (`entity_properties`), unconditionally dropped because no gate is
  evaluated. Iron/carrot/potato (gated on `killed_by_player` + 3.5%
  `random_chance_with_enchanted_bonus`) drop on most seeds.
- `entities/creeper`: gunpowder ×1 only — the music-disc pool's
  skeleton-kill gate is ignored, but the tag entry yields nothing
  (`items` vs `name`), so no disc appears for the wrong reason.
- `gameplay/fishing/fish`: returns fish (no functions on those
  entries; the open-water gate lives on the parent table).
- `blocks/stone` → `minecraft:stone` on all 5 seeds (the silk-touch
  child wins unconditionally; vanilla without silk touch drops
  cobblestone) — exactly the 1.21.7 symptom.
- `blocks/diamond_ore` → `minecraft:diamond_ore` on all 5 seeds
  (vanilla: diamond) — exactly the 1.21.7 symptom.

Affects entity drops, chest/container loot, fishing, trial spawners
and vaults in gameplay — everything `loot.rs` serves. 0 additional
gap checks: these tables are already counted under items.rs
(B12-G01).

### B12-G03 — minor: object-form recipe ingredients are rejected (datapack scope only)

`crafting.rs`, `Ingredient::parse_object` (line 123): a JSON-object
ingredient requires a `neoforge:ingredient_type` or `type` key, and
returns `None` otherwise. Vanilla's own object forms (`{"item": "..."}`,
`{"tag": "..."}`) have neither key, so any recipe using them fails to
parse (shaped: whole recipe dropped; shapeless/cooking: dropped).
No vanilla 26.1 recipe uses object ingredients (2,391 string slots +
84 array slots whose 1,072 elements are all strings; the one apparent
object slot is the firework star's `shapes` map, shape name → string
ingredient, not an ingredient object), so this affects only player
datapacks / mod recipes that use the object form. Same code, same
data property as 1.21.7's B12-G05 and 26.2's B12-G02 — carried
forward, not a vanilla parity defect. 0 gap checks.

### B12-G04 — minor: cooking-time default is a flat 200 for all four cooking types (datapack scope only)

`crafting.rs:1739`: `time: v["cookingtime"].as_u64().unwrap_or(200)`.
Vanilla 26.1's per-type defaults, verified by `javap -c` on the
client jar (the int each recipe class passes to
`AbstractCookingRecipe.cookingMapCodec`): smelting 200, blasting 100,
smoking 100, campfire_cooking 100. A datapack blasting/smoking/
campfire recipe without `cookingtime` would cook at 200 ticks instead
of 100. All 116 vanilla cooking recipes carry explicit `cookingtime`
(smelting 73, blasting 25, smoking 9, campfire 9), so no vanilla
recipe is affected — 0 gap checks. (The 26.2 tree uses per-type
defaults, `crafting.rs:1745-1759`; the 26.1 tree does not.)

## Passes, with the evidence

- **Recipes.** All 1,515 load under a parser-arm emulation (type arms
  + required fields + ingredient/tag resolution against the jar's
  item tags + result codec range): shaped 708, shapeless 322,
  stonecutting 275 (via `stations.rs` `parse_recipe_sources`, line
  805), smelting 73, blasting 25, smoking 9, campfire_cooking 9,
  transmute 33 (including `map_cloning` — map cloning IS a transmute
  in 26.1), smithing_trim 18, smithing_transform 12, crafting_dye 6
  (armour dyeing), crafting_imbue 1 (tipped arrows),
  crafting_decorated_pot 1, and 23 `crafting_special_*` files (16
  banner duplicates, book cloning, firework rocket, firework star,
  firework star fade, map extending, repair item, shield decoration).
  The 1.21.7 defect does not reproduce: 26.1's special files are
  DATA-DRIVEN (they carry `banner`/`result`, `material`/`source`,
  `fuel`/`shell`/`star`, `shapes`, etc.), not bare `{"type",
  "category"}` files, and `Recipes::from_sources`
  (`crafting.rs:1541`) plus `parse_recipe_sources` have an arm for
  every one of the 21 recipe types in the data, each requiring
  exactly the fields its files carry (verified per file, including
  the firework star's `shapes` object and the trim recipes' string
  `pattern` via `pattern_from_nbt`). `custom_type`
  (`crafting.rs:1332`) diverts only non-`minecraft:` types, so no
  vanilla recipe is diverted. All shaped patterns' characters exist
  in their `key` dicts (0 bad). Ingredient slots: 2,391 strings +
  84 arrays, all string elements, handled by `Ingredient::parse`
  (`crafting.rs:98`); 0 object slots (see B12-G03). Result forms:
  `{id}` ×878, `{id, count}` ×601, `{id, components}` ×17, all
  handled by `Template::parse` / `recipe_result_id`; max count 16
  (inside the 1..=99 codec range). All 116 cooking recipes carry
  explicit `cookingtime` (see B12-G04).
- **Advancements (triggers only).** 54 distinct triggers used across
  the 1,617 files (26.1 has `spear_mobs`, like 26.2). 53 are fired
  from gameplay code: direct `self.trigger("minecraft:…")` calls
  (`bee_nest_destroyed` `game/advancements.rs:2044`, `spear_mobs`
  `:2020`, `tick` `:1259`, `bred_animals`, `brewed_potion`,
  `changed_dimension`, `channeled_lightning`, `construct_beacon`,
  `cured_zombie_villager`, `effects_changed`, `enchanted_item`,
  `enter_block`, `fall_after_explosion`, `fishing_rod_hooked`,
  `killed_by_arrow`, `levitation`, `lightning_strike`,
  `player_generates_container_loot`, `recipe_crafted`,
  `slide_down_block`, `target_hit`, `villager_trade`, …), the
  generic `trigger_block` / `trigger_item` / `trigger_player` /
  `trigger_entity` / `trigger_distance` call sites (`placed_block`,
  `item_used_on_block`, `filled_bucket`, `consume_item`,
  `tame_animal`, `summoned_entity`, `nether_travel`,
  `fall_from_height`, `ride_entity_in_lava`, `shot_crossbow`,
  `slept_in_bed`, `started_riding`, `allay_drop_item_on_block`,
  `avoid_vibration`, `hero_of_the_village`, …), `trigger_killed` /
  `trigger_killed_as` (`player_killed_entity`,
  `kill_mob_near_sculk_catalyst`), `trigger_item_entity`
  (`player_interacted_with_entity`, `player_sheared_equipment`,
  `thrown_item_picked_up_by_entity`,
  `thrown_item_picked_up_by_player`), `trigger_item_durability`
  (`item_durability_changed`), `trigger_player_hurt_entity` /
  `trigger_entity_hurt_player`, `check_inventory_trigger`
  (`inventory_changed`, `game/advancements.rs:1150-1215`) and
  `recipe_unlocked` (`game/advancements.rs:944`). The one unfired
  trigger, `minecraft:impossible`, is used only by
  `recipes/root.json` and never fires in vanilla either — pass by
  design. `Tree::by_trigger` indexes triggers from the data files
  themselves (`advancements.rs:62`, built at `:222`), so no trigger
  is dropped at load. Item-predicate evaluation reads the 26.1 form
  (`items` / `count` / `components` / `predicates`,
  `advancements.rs:838-878`). Observation (not a numbered defect):
  `entity_killed_player`'s only firing call site found is the LAN
  guest death path (`game/lan_actions.rs:4707-4728`,
  `guest_killed_player_trigger`); no single-player local-death firing
  call site was found — same observation as the 26.2 run. The only
  advancement using it, `adventure/root.json`, grants via the OR'd
  `player_killed_entity` criterion too, so nothing is unreachable;
  the single-player firing path belongs to the batch that owns
  player-death semantics. (Criterion *condition* semantics beyond
  trigger firing are out of this batch's triggers-only scope.)
- **Tags.** All 758 files byte-identical, across 16 kinds (block 248,
  item 207, worldgen 92, villager_trade 73, entity_type 47,
  damage_type 33, enchantment 22, banner_pattern 11, fluid 6,
  game_event 5, timeline 4, instrument 3, point_of_interest_type 3,
  dialog 2, painting_variant 1, potion 1). `block-tags.json` (248
  keys) is exactly faithful to the jar's block tags — 248/248
  exact-equal (values are JSON-encoded strings; compared parsed),
  no extras, no missing keys. Runtime loaders read the same files
  (`assets.rs:194`, `animate_tick.rs:342`, `motion.rs:103` read
  `block-tags.json`; `load_tag_lists`, `items.rs:519`, is generic
  over kind). The 1.21.7 declared override does NOT apply here:
  26.1's jar ships 30 `supports_*` block tags and 3 `supports_*`
  fluid tags natively, and no `synthesize_legacy_support_tags` code
  exists in the 26.1 tree — nothing is synthesised and nothing is
  overridden.
- **Enchantments.** Loader (`horizon-items/src/enchant.rs:149-178`)
  reads exactly the 26.1 fields: `supported_items`,
  `primary_items` (5 files), `exclusive_set` (18), `max_level`,
  `weight`, `anvil_cost`, `min_cost`/`max_cost` (`base` /
  `per_level_above_first`), `slots`, `effects`. All 30 effect
  component keys used by the 43 files are evaluated somewhere: 9
  via the data-driven `modify_value` path (`enchant.rs:697`)
  (`damage`, `damage_protection` via `potions.rs:1221`,
  `item_damage`, `knockback`, `ammo_use`,
  `armor_effectiveness`, `projectile_piercing`,
  `repair_with_xp`, `smash_damage_per_fallen_block`), 8 via data-driven
  readers in `enchant_effects.rs` / `loot.rs` / `enchant.rs`
  (`attributes` `enchant.rs:744/767`, `post_attack`,
  `post_piercing_attack`, `projectile_spawned`,
  `damage_immunity`, `location_changed`, `equipment_drops`
  `loot.rs:1298`, `prevent_equipment_drop` `loot.rs:959`), 13 by
  enchantment id in gameplay code (`block_experience`: Silk Touch
  XP, `spill.rs:483`; `crossbow_charge_time` +
  `crossbow_charging_sounds`: Quick Charge, `crossbow.rs:81-102`;
  `fishing_luck_bonus` + `fishing_time_reduction`: Luck of the Sea
  / Lure, `fishing.rs:485-486`; `hit_block`: Channeling,
  `trident.rs:447` `channel_rod_hit`, `:1091`, `:1167`;
  `prevent_armor_change`: Binding Curse,
  `armor_stands.rs:1833`, `mob_looting.rs:170`;
  `projectile_count` + `projectile_spread`: Multishot,
  `crossbow.rs:261`; `tick`: Soul Speed particles,
  `enchant_effects.rs:1028` `tick_soul_speed`;
  `trident_return_acceleration`: Loyalty, `trident.rs:274-299`;
  `trident_sound` + `trident_spin_attack_strength`: Riptide,
  `trident.rs:396,499,551`). `fortune.json` has no `effects` —
  correct, fortune is implemented via loot `apply_bonus`. Effect
  *values* (damage numbers, durations) are behaviour depth for
  other batches, not checked here.
- **Status effects.** 40/40 colours, categories, instant flags and
  protocol order match the jar (`javap -c` on
  `MobEffects.<clinit>`, parsed programmatically). The instant set
  is exactly `instant_health`, `instant_damage`, `saturation`
  (`HealOrHarmMobEffect` / `SaturationMobEffect` extend
  `InstantaneousMobEffect`, verified by javap; the other 11
  effect subclasses extend `MobEffect` directly). Protocol ids
  0–39 in `registries.json` `minecraft:mob_effect` match the port
  table's order exactly. B12-G04 does not reproduce:
  `breath_of_the_nautilus` (colour 65518, Beneficial) IS in 26.1
  — last field of the jar's `MobEffects`, protocol id 39, last
  entry of the port's `EFFECTS` (`potions.rs:50-106`).
- **Damage types.** Loader
  (`horizon-world/src/damage_source/resources.rs:127-173`,
  `from_json`) parses `message_id` (required), `exhaustion`
  (required, parsed directly to f32), `scaling` (validated:
  `never` / `when_caused_by_living_non_player` / `always`),
  `effects` (default `hurt`, validated against hurt/thorns/drowning/
  burning/poking/freezing), and `death_message_type` (default
  `default`, validated against default/fall_variants/
  intentional_game_design). Every value in the 50 files is inside
  the accepted lists (scaling: `always` ×4 /
  `when_caused_by_living_non_player` ×46; effects: burning ×7,
  drowning, freezing, poking, thorns, default ×39;
  death_message_type: `fall_variants` ×2,
  `intentional_game_design` ×1, default ×47).

## Declared overrides

None. The 1.21.7 batch's declared override (`supports_*` tag
synthesis) does not exist in the 26.1 tree and is not needed: 26.1
ships the tags natively (see Tags pass above).

## Not reproduced (1.21.7 Batch 12 defects, re-checked against 26.1)

- **B12-G01 (1.21.7, MAJOR)** — items.rs conditions always false:
  REPRODUCED as B12-G01 above (first half), plus the
  `functions`-container defect — the 26.2 tree's fix was never
  carried to 26.1.
- **B12-G02 (1.21.7, MAJOR)** — loot.rs ignores all conditions and
  functions: REPRODUCED as B12-G02 above, demonstrated end to end
  by the `loot_fill` run.
- **B12-G03 (1.21.7, MAJOR)** — 11 of 12 special recipes never load:
  NOT REPRODUCED. 26.1's specials are data-driven and the parser has
  arms for all 21 recipe types; 1,515/1,515 load.
- **B12-G04 (1.21.7, minor)** — extra 26.x effect in the table: NOT
  REPRODUCED. `breath_of_the_nautilus` is a real 26.1 effect
  (protocol id 39); the port's 40-entry table matches the jar
  exactly.
- **B12-G05 (1.21.7, minor)** — object-form ingredients rejected:
  REPRODUCED as B12-G03 above (same code, same 0-object-slots data
  property; datapack scope only).
- **26.2 B12-G01 (minor)** — items.rs partial function set (102
  tables): SUBSUMED by this batch's B12-G01. The arm gaps exist in
  26.1 (7 missing kinds + no `tag` arm, 102-table post-fix
  residual — the same count as 26.2) but are unobservable until the
  key-form fix lands; recorded inside B12-G01's evidence, not as a
  separate item.
- **26.2 B12-G02 (minor)** — object-form ingredients: REPRODUCED as
  B12-G03 above.

## Method notes / limitations

- The data diff is byte-level (`diff -r` over the extracted jar
  `data/` vs the prepared `.local` copy): 0 differing files in all
  six data-driven families. The prepared data is a faithful copy of
  the jar's datapack — the divergences are in the code that reads
  it, which is why this batch's behaviour checks carried the
  findings.
- The loot condition/function census walk recurses into every dict
  and list (including `terms`/`term` combinator arrays) — the 26.2
  coordinator note's correction is built into the census script
  (`evidence/census.py`). Counts cross-check against Batch 4's B4-G26
  exactly: 1,545 `condition`, 983 `functions`, 0 `modifier`.
- `loot_fill` was built and run (`cargo build -p horizon-world
  --example loot_fill`, dev profile, `taskset -c 0`, `-j 1`, 3m 35s,
  EXIT 0; run from `versions/26.1/` as
  `./target/debug/examples/loot_fill ./.local/horizon-client/26.1
  <tables> <seeds>`). Table ids must be namespaced
  (`minecraft:chests/simple_dungeon`) — the tables are keyed with
  namespace, and an un-namespaced id silently returns empty. No
  second probe was built: the items.rs engine is covered by
  arm-by-arm code reads against the per-family data census plus the
  survival call-path trace (`game.rs:5084` →
  `block_drops_by_player` → `evaluate`), and its symptom is the
  same one the owner confirmed in-game for 1.21.7. The items.rs
  symptom was NOT re-confirmed by a live 26.1 playtest (no playtest
  runs in this sweep); the code path and data shape are identical
  to the owner-confirmed 1.21.7 case.
- The recipe loadability count is a parser-arm emulation in Python
  (type arms + required fields + ingredient/tag resolution against
  the jar's item tags + result codec range), not a run of the Rust
  parser; it mirrors `from_sources` / `parse_recipe_sources` /
  `Ingredient::parse` / `Template::parse` field by field, and every
  arm's required fields were checked against the actual files.
  `decode_components` failure modes for the 17 components-bearing
  results were not emulated (it returns defaults, never `None`).
- `registries.json` `minecraft:block_type` (unverified provenance
  per §1.1) was not used for any count or check.
- Loot *number providers* (`{"type": "minecraft:uniform", …}`) and
  pool *entry* types (`minecraft:item`, `alternatives`, …) correctly
  use the `"type"` discriminator in 26.1 — only the condition and
  function discriminators were renamed in 26.3, and 26.1 still uses
  the old keys. The defect is specific to the two keys (plus the
  singular/plural `condition` vs `conditions` change, the
  `modifier` vs `functions` container key, and the tag entry's
  `items` vs `name` field), and 26.1's engines handle none of the
  old forms.
- Recipe *book* display was not checked; the book reads the same
  `sources`, so loaded recipes list. Behaviour of enchantment effect
  *values* and advancement criterion *condition* semantics belong to
  other batches. Whether the local single-player death path fires
  `entity_killed_player` (only the LAN guest path was found) is
  flagged above as an observation for the owning batch.
- The 26.2 tree comparison (`versions/26.2/` in the same worktree)
  was used only to establish that the 26.2 playtest loot fixes were
  never carried to 26.1; all defect claims rest on the 26.1 tree and
  the 26.1 jar/data alone.
- Evidence kept in the batch dir: `evidence/census.py` +
  `census.json`/`census.txt` (full per-family census),
  `evidence/recipe_emulation.py` + `recipe_emulation.txt`,
  `evidence/loot_fill.log` + `loot_fill-build.log`,
  `evidence/mobeffects.txt` (javap dump) +
  `effects_compare.txt` + `effect_superclasses.txt`,
  `evidence/triggers.txt`, `jar-data/` (extracted jar `data/`,
  8,282 files).
