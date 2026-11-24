# Batch 12 — Data-driven registries (26.2)

Checks + triage only, 2026-11-24. Scope: recipes (1,585), loot tables
(1,355), advancements (1,688, triggers only), tags (794), enchantments
(43), status effects (40), damage types (51). Method 4 (PARITY-SWEEP.md
§3): byte-level diff of the port's prepared data against the vanilla
26.2 data files, plus behaviour checks of the loaders — here, where the
*loader* disagrees with the data's form even though the data itself is
faithful.

Vanilla spec: the named 26.2 server-game jar
(`26.1-adapters/versions/26.2/jars/server-game.jar`, Mojang-mapped;
`data/minecraft/*` extracted to the batch scratch dir), `javap -c` on
its `MobEffects` class for effect colours/categories/instant flags, and
the 26.2 datagen reports for the inventory counts. Port data:
`versions/26.2/.local/horizon-client/26.2/data/minecraft/` (prepared
data; the prepare run finished DURING this batch — 2048.7 s, EXIT 0 —
and the diffs below were re-run against the final prepared tree, still
0 differing). Loot behaviour: the existing `loot_fill` example, built
once and run against the real prepared data. NOTE: `sounds.json` and
lang files are absent from the prepared tree (the VM lacks the official
launcher's asset index 32.json); that is expected, not a port defect,
and no check in this batch needed them.

**Coordinator note (2026-11-24):** spot-verified against the sweep
worktree and the prepared data — the loot dual-form fix is present at
`items.rs:2090`, and `items.rs` has no `copy_components` arm (B12-G01
holds). One census number is corrected: the `condition` discriminator
count is **1,574**, not 1,457 — the agent's JSON walk missed the 117
conditions nested inside `terms`/`term` combinator arrays (102 in
`terms`, 15 in inverted `term`); `function` (1,416) and plural
`conditions` (1,418) recount exactly. No finding changes: the 12 kinds
present in the data are all in the handled list (`all_of`, named in the
report's list, has 0 occurrences in the data). Behaviour subtotal
reconciles as 1,585 recipe loads + 21 recipe-type arms + 1,355 items.rs
tables + 54 triggers + 265 block-tags + 30 enchantment keys + 40 effects
+ 51 damage types = 3,401; the loot.rs 242-table run covers tables
already counted under items.rs and is not double-counted.

## Tally

| Registry | Data diff (byte-level) | Behaviour check | Result |
|---|---|---|---|
| Recipes (1,585) | 1,585/1,585 identical | Loadability per recipe (parser-arm emulation incl. tag resolution): 1,585 load, 0 dropped | pass |
| Loot tables (1,355) | 1,355/1,355 identical | items.rs × 1,355 tables: 1,253 pass / 102 gap; loot.rs × 242 served tables: 242 pass / 0 gap | **102 gap checks** (B12-G01) |
| Advancements (1,688) | 1,688/1,688 identical | 54 distinct triggers: 53 fired, `impossible` never fires by vanilla design | pass |
| Tags (794) | 794/794 identical | Runtime loaders read the same files; `block-tags.json` 265/265 exact-equal; `supports_*` tags exist natively in 26.2 | pass |
| Enchantments (43) | 43/43 identical | Loader reads all 26.2 fields; all 30 effect component keys wired (data-driven or by enchantment id) | pass |
| Status effects (40) | n/a — code-defined in 26.2, no `mob_effect/` data files in the jar | 40/40 colours, categories, instant flags, protocol order match the jar; `breath_of_the_nautilus` is real in 26.2 | pass |
| Damage types (51) | 51/51 identical | Loader validates all fields; every value in the files is accepted | pass |

Checks total: **8,917** (5,516 data-diff entries + 3,401 behaviour
checks). Pass: **8,815**. Gap: **102** — collapsing to **2 defects**,
both minor (B12-G02 contributes 0 gap checks: no vanilla recipe uses
object-form ingredients, same as 1.21.7). The gap count is the
`items.rs` engine's partial function/entry set hitting 102 tables
through `/loot` (and 71 block tables through the engine itself, where
the survival path is covered by dedicated gameplay code — see
B12-G01).

No oracle defects, no oracle repairs: this batch is driver-independent
(javap + data diffs + the `loot_fill` example run). No fixes made.

## The systemic pattern

The prepared data is faithful — every one of the 5,516 data-driven
files is byte-identical to the 26.2 jar, and `block-tags.json` is
byte-faithful too. The 1.21.7 batch's systemic finding (code reading
the 26.x form of formats 1.21.7 writes differently) does NOT carry to
26.2, for a simple reason: **26.2 writes the old form**. Its loot JSONs
use the `condition` discriminator (1,574 occurrences, 0 `type`), the
`function` discriminator (1,416, 0 `type`), and plural `conditions`
arrays (1,418) — the `type` rename is a 26.3 change, not a 26.2 one.
The 26.2 tree's loot fix (accept both key forms) is therefore
load-bearing, not belt-and-braces, and it is verified working end to
end below. Likewise 26.2's special recipes are already data-driven
(post-E6 form), and its effect table legitimately has 40 entries.
What remains in 26.2 is a narrower, older pattern: `items.rs` is a
block-drop engine with a partial loot-function set, reused by `/loot`
for every table type, so command-rolled tables silently lose the
functions and entry types it never implemented.

## Defects

### B12-G01 — minor: items.rs has no `copy_components` arm (and 7 more function kinds + the `tag` entry); `/loot` rolls lose them silently

`horizon-items/src/items.rs`, `LootContext::modifiers()` (line 2401):
the function dispatch (line 2411) covers 11 of the 19 function kinds
26.2's data uses. Missing, with the default arm `_ => {}` (line 2607)
silently no-opping them: `copy_components`, `furnace_smelt`,
`set_enchantments`, `set_stew_effect`, `set_name`, `exploration_map`,
`set_ominous_bottle_amplifier`, `set_instrument`. `expand()` (line
2227) likewise has no `minecraft:tag` arm (default `_ => false`, line
2319). Per-table impact, counted over all 1,355 tables: **102 tables**
carry ≥1 unhandled function/entry — 71 block tables
(`copy_components`), 18 entity tables (`furnace_smelt` 16,
`set_ominous_bottle_amplifier` 1, the `tag` entry in
`entities/creeper.json`), 9 chest tables, 3 equipment tables, 1
archaeology table.

Survival gameplay is largely shielded, which is why this is minor
rather than major:

- Block breaking (the 71 `copy_components` tables: shulker boxes,
  banners, beehives, chests, heads, decorated pots) is covered by
  dedicated gameplay code, not the engine: `game/spill.rs`
  `Pending::record` (line 44) / `attach` (line 100) copies the
  block-entity components into the dropped item by block-entity type.
  All six distinct `include` sets in the data are covered (34×
  `custom_name`-only via the names map, 17× the shulker set via the
  shulker contents map, 16× the banner set via the banner map +
  item_name/rarity/tooltip_display loop, 2× `bees`, 1×
  `pot_decorations`, 1× the skull set). Two components in the shulker
  set, `lock` and `container_loot`, are not copied by `spill.rs` —
  both are unreachable on a shulker box in vanilla survival play
  (lock needs NBT editing; container_loot is never generated on one).
- Entity drops, chest/container loot, fishing, trial spawners and
  vaults go through `loot.rs` in gameplay, and that engine handles
  every kind those tables use (see Passes).

What actually breaks is `/loot`, which routes every table through
`items.rs`: `cmd_loot` (`game/commands/terrain_cmds.rs:585`) uses
`loot_table_for_command` (items.rs:1784) for `loot`/`fish`,
`entity_drops_for_command` / `entity_drops_with_target` (items.rs:1889)
for `kill`, and `block_drops_by_player` for `mine`. Concretely:
`/loot give … loot minecraft:chests/simple_dungeon` drops nothing the
engine lacks arms for in that table (set_count/set_damage work), but
`/loot … loot minecraft:chests/ancient_city` loses `set_name`d items'
names; `/loot kill` on a creeper loses the music-disc `tag` entry;
`/loot kill` on a cow loses `furnace_smelt` (no cooked drops from a
fire kill via the command); `/loot mine` on a shulker box drops it
empty (no `spill.rs` on the command path). The engine's condition
handling is complete (all 12 kinds), so only functions/entries are
affected.

### B12-G02 — minor: object-form recipe ingredients are rejected (datapack scope only)

`crafting.rs`, `Ingredient::parse_object` (line 123): a JSON-object
ingredient requires a `neoforge:ingredient_type` or `type` key, and
returns `None` otherwise. Vanilla's own object forms (`{"item": "..."}`,
`{"tag": "..."}`) have neither key, so any recipe using them fails to
parse (shaped: whole recipe dropped; shapeless/cooking: dropped).
No vanilla 26.2 recipe uses object ingredients (all 3,542 ingredient
slots are strings, plus 84 array slots whose elements are all strings),
so this affects only player datapacks / mod recipes that use the
object form. Same code, same data property as 1.21.7's B12-G05 —
carried forward, not a vanilla parity defect.

## Passes, with the evidence

- **Loot engines — the 1.21.7 format defects do not reproduce.**
  26.2's data uses the pre-26.3 key forms throughout (`condition` /
  `function` discriminators, plural `conditions` arrays — counted
  above), and both engines accept both forms. `items.rs`
  `LootContext::condition()` (line 2082) reads the kind from
  `o.get("type").or_else(|| o.get("condition"))` (line 2090) and
  handles all 12 condition kinds in the data (`survives_explosion`,
  `block_state_property`, `match_tool`, `table_bonus`, `inverted`,
  `any_of`, `all_of`, `random_chance`,
  `random_chance_with_enchanted_bonus`, `killed_by_player`,
  `entity_properties`, `damage_source_properties`,
  `location_check`); `conditions()` (line 2217) consults the plural
  array. `loot.rs` `condition()` (line 1386) reads
  `c["type"].or(c["condition"])`; `modify()` (line 1497) consults
  `m["conditions"]` (line 1507); `run()` (line 1531) reads
  `f["type"].or(f["function"])` (line 1534); `table_raw` (line 2035)
  and `expand` (line 2096) consult pool/entry `conditions` (lines
  2042, 2097). `loot.rs` handles all 19 function kinds except
  `apply_bonus`, `copy_components`, `copy_state`, `explosion_decay`
  — which occur ONLY in block tables, which `loot.rs` never serves
  in gameplay (its callers are entity drops, chest/container loot,
  fishing, trial spawners, vaults, decorated pots). Likewise its
  `condition()` lacks `survives_explosion` / `block_state_property` /
  `match_tool` / `table_bonus` arms, and no table it serves uses
  them (verified per family; the one non-block
  `block_state_property` use, `harvest/sweet_berry_bush`'s age=3
  gate, goes through `items.rs` `interact_loot`, line 1824, which
  has the arm). The single `tag` entry in the data
  (`entities/creeper.json`, `expand: true`, music discs) is handled
  by `loot.rs` `expand()` (line 2124, with tests).
  Ran the existing `loot_fill` example against the real prepared
  data (one build: `cargo build -p horizon-world --example
  loot_fill`, dev profile, `taskset -c 0`, `-j 1`, 15m 51s, EXIT 0;
  run command below): `chests/simple_dungeon` counts vary 1–5
  (`set_count` works; 1.21.7's symptom was every stack count 1),
  `chests/ancient_city` rolls enchanted books with real enchantments
  (`swift_sneak=2`, `piercing=4`, `density=3`, leggings with
  `protection=4` + `mending=1` — `enchant_randomly` /
  `enchant_with_levels` work), `entities/zombie` drops rotten flesh
  under its `random_chance` gate, `entities/creeper` drops gunpowder
  (the disc entry correctly gated off without a skeleton kill),
  `gameplay/fishing/fish` returns fish. `blocks/stone` through
  `loot_fill` returns empty — expected: `loot.rs` is not a block
  engine (its `survives_explosion` arm is absent by design; block
  drops go through `items.rs`), documented here as the engine-scope
  boundary, not a defect. `blocks/diamond_ore` through `loot_fill`
  returns diamond ×1 (silk-touch child correctly fails without a
  silk-touch tool; `apply_bonus`/`explosion_decay` no-op in this
  engine, outcome coincides with vanilla's no-fortune drop).
- **Recipes.** All 1,585 load under a parser-arm emulation (type arms
  + required fields + ingredient resolution against the jar's item
  tags + result forms): shaped 733, shapeless 323, stonecutting 319
  (via `stations.rs` `parse_recipe_sources`, line 805), smelting 73,
  blasting 25, smoking 9, campfire_cooking 9, transmute 33 (including
  `map_cloning` — map cloning IS a transmute in 26.2),
  smithing_trim 18, smithing_transform 12, crafting_dye 6 (armour
  dyeing), crafting_imbue 1 (tipped arrows), crafting_decorated_pot
  1, and 23 `crafting_special_*` files (16 banner duplicates, book
  cloning, firework rocket, firework star, firework star fade, map
  extending, repair item, shield decoration). The 1.21.7 defect does
  not reproduce: 26.2's special files are DATA-DRIVEN (they carry
  `banner`/`result`, `material`/`source`, `fuel`/`shell`/`star`,
  `shapes`, etc.), not bare `{"type", "category"}` files, and
  `Recipes::from_sources` (`crafting.rs:1553`) has an arm for every
  one of the 21 recipe types in the data, each requiring exactly the
  fields its files carry (verified per file, including the firework
  star's `shapes` object and the trim recipes' string `pattern`
  via `pattern_from_nbt`). Cooking uses 26.2's per-type default
  times (smelting 200, others 100) with optional `cookingtime`
  (`crafting.rs:1744-1747`). Ingredient slots: 3,542 strings + 84
  arrays, all string elements, handled by `Ingredient::parse`
  (`crafting.rs:98`); 0 object slots (see B12-G02). Result forms:
  `{id}` ×914, `{id, count}` ×635, `{id, components}` ×17, all
  handled by `Template::parse` / `recipe_result_id`; max count 16
  (inside the 1..=99 codec range).
- **Advancements (triggers only).** 54 distinct triggers used across
  the 1,688 files (26.2 adds `spear_mobs` over 1.21.7's 53). 53 are
  fired from gameplay code: direct `self.trigger("minecraft:…")`
  calls (crafter_recipe_crafted, cured_zombie_villager, tick,
  location, killed_by_arrow, levitation, fall_after_explosion,
  effects_changed, item_durability_changed, using_item,
  player_hurt_entity, entity_hurt_player, construct_beacon,
  enchanted_item, brewed_potion, villager_trade, fishing_rod_hooked,
  enter_block, slide_down_block, target_hit, lightning_strike,
  channeled_lightning, spear_mobs, bee_nest_destroyed,
  recipe_crafted, player_generates_container_loot, bred_animals,
  changed_dimension, …), the generic `trigger_block` / `trigger_item`
  / `trigger_player` / `trigger_entity` / `trigger_distance` call
  sites (placed_block, item_used_on_block, filled_bucket,
  consume_item, tame_animal, summoned_entity, nether_travel,
  fall_from_height, ride_entity_in_lava, shot_crossbow, slept_in_bed,
  started_riding, allay_drop_item_on_block, avoid_vibration,
  hero_of_the_village, …), `trigger_killed` /
  `trigger_killed_as` (`player_killed_entity`, `entity_killed_player`,
  `kill_mob_near_sculk_catalyst`), `trigger_item_entity`
  (`player_interacted_with_entity`, `player_sheared_equipment`,
  `thrown_item_picked_up_by_entity`, `thrown_item_picked_up_by_player`),
  `check_inventory_trigger` (`inventory_changed`) and
  `recipe_unlocked`. The one unfired trigger, `minecraft:impossible`,
  is used only by `recipes/root.json` and never fires in vanilla
  either — pass by design. `Tree::by_trigger` indexes triggers from
  the data files themselves (`advancements.rs:222`), so no trigger is
  dropped at load. Item-predicate evaluation reads the 26.2 form
  (`items` / `count` / `components` / `predicates`,
  `advancements.rs:860-878`). Observation (not a numbered defect):
  `entity_killed_player`'s only firing call site found is the LAN
  guest death path (`game/lan_actions.rs:4727-4750`,
  `guest_killed_player_trigger`); no single-player local-death firing
  call site was found. The only advancement using it,
  `adventure/root.json`, grants via the OR'd `player_killed_entity`
  criterion too, so nothing is unreachable; the single-player firing
  path belongs to the batch that owns player-death semantics.
  (Criterion *condition* semantics beyond trigger firing are out of
  this batch's triggers-only scope.)
- **Tags.** All 794 files byte-identical, across 16 kinds (block 265,
  item 224, worldgen 92, villager_trade 73, entity_type 48,
  damage_type 34, enchantment 22, banner_pattern 11, fluid 6,
  game_event 5, timeline 4, instrument 3, point_of_interest_type 3,
  dialog 2, painting_variant 1, potion 1). `block-tags.json` (265
  keys) is exactly faithful to the jar's block tags — 265/265
  exact-equal, no extras. Runtime loaders read the same files
  (`level/data.rs:739` reads `block-tags.json`; `load_tag_lists`,
  `items.rs:519`, is generic over kind). The 1.21.7 declared
  override does NOT apply here: 26.2's jar ships 33 `supports_*`
  block tags and 3 `supports_*` fluid tags natively, and no
  `synthesize_legacy_support_tags` code exists in the 26.2 tree —
  nothing is synthesised and nothing is overridden.
- **Enchantments.** Loader (`horizon-items/src/enchant.rs:149-178`)
  reads exactly the 26.2 fields: `supported_items`,
  `primary_items` (5 files), `exclusive_set` (18), `max_level`,
  `weight`, `anvil_cost`, `min_cost`/`max_cost` (`base` /
  `per_level_above_first`), `slots`, `effects`. All 30 effect
  component keys used by the 43 files are evaluated somewhere: 9 via
  the data-driven `modify_value` path (`damage`,
  `damage_protection`, `item_damage`, `knockback`, `ammo_use`,
  `armor_effectiveness`, `projectile_piercing`, `repair_with_xp`,
  `smash_damage_per_fallen_block`), 8 via data-driven readers in
  `enchant_effects.rs` / `loot.rs` / `enchant.rs` (`attributes`,
  `post_attack`, `post_piercing_attack`, `projectile_spawned`,
  `damage_immunity`, `location_changed`, `equipment_drops`,
  `prevent_equipment_drop`), 13 by enchantment id in gameplay code
  (`block_experience`: Silk Touch XP, `spill.rs:504`;
  `crossbow_charge_time` + `crossbow_charging_sounds`: Quick Charge,
  `crossbow.rs:81-102`; `fishing_luck_bonus` +
  `fishing_time_reduction`: Luck of the Sea / Lure,
  `fishing.rs:485-486`; `hit_block`: Channeling, `trident.rs:1091,
  1167`; `prevent_armor_change`: Binding Curse,
  `armor_stands.rs:1834`, `mob_looting.rs:170`;
  `projectile_count` + `projectile_spread`: Multishot,
  `crossbow.rs:261`; `tick`: Soul Speed particles,
  `enchant_effects.rs:1028` `tick_soul_speed`;
  `trident_return_acceleration`: Loyalty, `trident.rs:274-299`;
  `trident_sound` + `trident_spin_attack_strength`: Riptide,
  `trident.rs:396,499,551`). `fortune.json` has no `effects` — correct,
  fortune is implemented via loot `apply_bonus`. Effect *values*
  (damage numbers, durations) are behaviour depth for other batches,
  not checked here.
- **Status effects.** 40/40 colours, categories, instant flags and
  protocol order match the jar (`javap -c` on
  `MobEffects.<clinit>`, parsed programmatically). The instant set is
  exactly `instant_health`, `instant_damage`, `saturation`
  (`HealOrHarmMobEffect` / `SaturationMobEffect` extend
  `InstantaneousMobEffect`, verified by javap; the other 37 classes
  extend `MobEffect` directly or via a non-instant subclass).
  Protocol ids 0–39 in `registries.json`
  `minecraft:mob_effect` match the port table's order exactly.
  B12-G04 does not reproduce: `breath_of_the_nautilus` (colour
  65518, Beneficial) IS in 26.2 — last field of the jar's
  `MobEffects`, protocol id 39, last entry of the port's `EFFECTS`
  (`potions.rs:50-106`).
- **Damage types.** Loader
  (`horizon-world/src/damage_source/resources.rs:127-173`,
  `from_json`) parses `message_id` (required), `exhaustion`
  (required, parsed directly to f32), `scaling` (validated:
  `never` / `when_caused_by_living_non_player` / `always`),
  `effects` (default `hurt`, validated against hurt/thorns/drowning/
  burning/poking/freezing), and `death_message_type` (default
  `default`, validated against default/fall_variants/
  intentional_game_design). Every value in the 51 files is inside
  the accepted lists (scaling: `always` ×4 /
  `when_caused_by_living_non_player` ×47; effects: burning ×8,
  drowning, freezing, poking, thorns, default ×39;
  death_message_type: `fall_variants` ×2,
  `intentional_game_design` ×1, default ×48).

## Declared overrides

None. The 1.21.7 batch's declared override (`supports_*` tag
synthesis) does not exist in the 26.2 tree and is not needed: 26.2
ships the tags natively (see Tags pass above).

## Not reproduced (1.21.7 Batch 12 defects, re-checked against 26.2)

- **B12-G01 (1.21.7, MAJOR)** — items.rs conditions always false:
  NOT REPRODUCED. The 26.2 tree has the fix
  (`o.get("type").or_else(|| o.get("condition"))`, items.rs:2090)
  and 26.2's data uses the `condition` key throughout, so the fix is
  load-bearing and verified working.
- **B12-G02 (1.21.7, MAJOR)** — loot.rs ignores all conditions and
  functions: NOT REPRODUCED. All three halves of the fix are present
  (loot.rs:1386, 1507, 1534, 2042, 2097) and `loot_fill` confirms
  conditions gate and functions run on real data.
- **B12-G03 (1.21.7, MAJOR)** — 11 of 12 special recipes never load:
  NOT REPRODUCED. 26.2's specials are data-driven and the parser has
  arms for all 21 recipe types; 1,585/1,585 load.
- **B12-G04 (1.21.7, minor)** — extra 26.x effect in the table: NOT
  REPRODUCED. `breath_of_the_nautilus` is a real 26.2 effect
  (protocol id 39); the port's 40-entry table matches the jar
  exactly.
- **B12-G05 (1.21.7, minor)** — object-form ingredients rejected:
  REPRODUCED as B12-G02 above (same code, same 0-object-slots data
  property; datapack scope only).

## Method notes / limitations

- The data diff is byte-level (`diff -r` over the extracted jar
  `data/` vs the prepared `.local` copy): 0 differing files in all
  six data-driven families, re-run after the prepare finished during
  this batch. The prepared data is a faithful copy of the jar's
  datapack — the divergences are in the code that reads it, which is
  why this batch's behaviour checks carried the findings.
- `loot_fill` was built and run (`cargo build -p horizon-world
  --example loot_fill`, dev profile, `taskset -c 0`, `-j 1`, 15m 51s
  under load, EXIT 0; run from `versions/26.2/` as
  `./target/debug/examples/loot_fill ./.local/horizon-client/26.2
  <tables> <seeds>`). No second probe was built: the items.rs engine
  is covered by arm-by-arm code reads against the per-family data
  census plus the `spill.rs` coverage check, and its symptom (via
  `/loot`) is command-scope.
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
  use the `"type"` discriminator in 26.2 — only the condition and
  function discriminators were renamed in 26.3, and 26.2 still uses
  the old keys. The defect is specific to the two keys (plus the
  singular/plural `condition` vs `conditions` change), and 26.2's
  engines handle both forms.
- Recipe *book* display was not checked; the book reads the same
  `sources`, so loaded recipes list. Behaviour of enchantment effect
  *values* and advancement criterion *condition* semantics belong to
  other batches. Whether the local single-player death path fires
  `entity_killed_player` (only the LAN guest path was found) is
  flagged above as an observation for the owning batch.
- Workdir note: the extracted jar data (`jar-data/`, 8,525 files)
  and the `javap -c` dump (`mobeffects.txt`) were deleted after the
  checks; only the two deliverables remain in the batch scratch dir.
