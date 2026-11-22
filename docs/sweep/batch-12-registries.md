# Batch 12 — Data-driven registries (1.21.7)

Checks + triage only, 2026-11-22. Scope: recipes (1,407), loot tables
(1,246), advancements (1,520, triggers only), tags (589), enchantments
(42), status effects (39), damage types (49). Method 4 (PARITY-SWEEP.md
§3): byte-level diff of the port's prepared data against the vanilla
1.21.7 data files, plus behaviour spot-checks where a diff disagrees —
here, where the *loader* disagrees with the data's form even though the
data itself is faithful.

Vanilla spec: `1.21.7-server-named.jar` (`data/minecraft/*`), extracted
to the batch scratch dir; port data:
`versions/1.21.7/.local/horizon-client/1.21.7/data/minecraft/` (and the
identical `mob-spawn/resources/` copy). Effect colours/categories:
`javap -c` on the named jar's `MobEffects` class. Loot behaviour: the
existing `loot_fill` example, run against the real prepared data.

## Tally

| Registry | Data diff (byte-level) | Behaviour check | Result |
|---|---|---|---|
| Recipes (1,407) | 1,407/1,407 identical | Loadability per recipe: 1,396 load, 11 specials dropped | **11 gap** (B12-G03) |
| Loot tables (1,246) | 1,246/1,246 identical | Two engines × 1,246 tables: items.rs 240 pass / 1,006 gap; loot.rs 131 pass / 1,115 gap | **2,121 gap checks** (B12-G01, B12-G02) |
| Advancements (1,520) | 1,520/1,520 identical | 53 distinct triggers: all fired except `impossible` (never fires by vanilla design) | pass |
| Tags (589) | 589/589 identical | Runtime loaders read the same files; supports_* synthesis is a declared override | pass + 1 declared override |
| Enchantments (42) | 42/42 identical | Loader reads all 1.21.7 fields; all 29 effect components wired (data-driven or by enchantment id) | pass |
| Status effects (39) | n/a — code-defined in 1.21.7, no `mob_effect/` data files in the jar | 39/39 colours, categories, instant flags, protocol order match the jar; port table carries 1 extra 26.x entry | **1 gap** (B12-G04) |
| Damage types (49) | 49/49 identical | Loader validates all fields; every value in the files is accepted | pass |

Checks total: **8,936** (4,853 data-diff entries + 4,083 behaviour
checks). Pass: **6,803**. Gap: **2,133** — collapsing to **5 defects**,
all grouped by cause below. The gap count is large because the loot
format defect hits per-table in two engines; it is two defects, not two
thousand.

No oracle defects, no oracle repairs: this batch is driver-independent
(javap + data diffs + the `loot_fill` example run). No fixes made.

## The systemic pattern

The prepared data is faithful — every one of the 4,853 data-driven
files is byte-identical to the 1.21.7 jar, and `block-tags.json` is
byte-faithful too. Every defect in this batch is the *code* reading the
26.x form of a data format that 1.21.7 writes differently, and failing
silently: the loot engines, the recipe parser, and the effect table all
carry 26.x assumptions. This is the same bug class as the pilot's
26.x-tag findings (G1/G3/G5/G6), one level down: not a missing tag but a
renamed key or a renamed recipe type. The 26.2 playtest already fixed
the loot half of it in the 26.2 tree; the fix was never carried to
1.21.7 (and was only half-carried to `items.rs` — functions yes,
conditions no).

## Defects

### B12-G01 — MAJOR: block drops evaluate every loot condition as false; survival block-breaking drops nothing

`horizon-items/src/items.rs`, `LootContext::condition()` (line ~1952):
the condition kind is read ONLY from `o.get("type")`. Vanilla 1.21.7
loot conditions use the `"condition"` key
(`{"condition": "minecraft:survives_explosion"}`), so the kind is `""`
and the match falls to `_ => false`. The plural `"conditions"` array IS
consulted (`conditions()`, line ~2078) — every element just fails.

Impact: **961 of 1,023 block tables** carry ≥1 condition (733 with
`survives_explosion`, 252 `block_state_property`, 203 `match_tool`
occurrences across the corpus), so breaking those blocks in survival
drops nothing. Path verified end to end: player break →
`Game::block_loot_at` → `Items::block_drops_by_player` →
`block_drops_raw` → `LootContext::evaluate` → `conditions` →
`condition`; no fallback, no normalisation anywhere in the load path
(`shared_tables` stores raw JSON). The same engine serves `/loot`,
`block_drops*`, and `interact_loot`, so **1,006 of 1,246 tables**
(all with conditions) misbehave through it.

Owner-confirmed in-game (2026-11-22 playtest: "survival
block-breaking drops nothing"). The 26.2 tree has the fix
(`o.get("type").or_else(|| o.get("condition"))` with the comment "26.2
names a condition's kind `condition`; 26.3 renamed it `type`");
1.21.7 lacks it. Note the asymmetry: `items.rs`'s *function* dispatch
(line 2277) already has the `.or(f["function"])` fix, so loot functions
(set_count, apply_bonus, explosion_decay) DO run here — only
conditions are broken.

### B12-G02 — MAJOR: the loot.rs engine ignores all loot conditions and all loot functions

`horizon-world/src/loot.rs` is written for the 26.x loot format in
three places, and 1.21.7's format differs in all three:

- Pools/entries/modifiers are consulted via the singular
  `"condition"` key (`table_raw` line 2018, `expand` line 2071,
  `modify` line 1488). 1.21.7 uses the plural `"conditions"` array,
  so the lookup is null and **no condition is ever evaluated** — every
  entry passes unconditionally.
- `condition()` (line 1377) reads the kind from `c["type"]`; 1.21.7
  uses `"condition"`. (Moot while the callers never pass a condition,
  but wrong on its own.)
- `run()` (line 1514) reads the function kind from `f["type"]`;
  1.21.7 uses `"function"`. The default arm is `_ => {}` — **every
  loot function is a silent no-op**.

This engine serves entity drops (`roll_entity`), chest/container loot
(`fill`/`roll`), fishing, trial spawners and vaults. Ran the existing
`loot_fill` example against the real prepared data
(`mob-spawn/resources`, seeds 1–5):

- `minecraft:blocks/stone` → `minecraft:stone` (the silk-touch child
  wins unconditionally; vanilla without silk touch drops cobblestone).
- `minecraft:blocks/diamond_ore` → `minecraft:diamond_ore` (vanilla:
  diamond).
- `minecraft:chests/simple_dungeon` → every stack count 1 (`set_count`
  ignored; vanilla counts vary 1–8+).

Corpus impact: **1,006 tables** carry conditions (never consulted),
**327 tables** carry functions (never run); 1,115 tables carry at
least one of the two. Concretely: no fortune (`apply_bonus`, 32
occurrences), no looting (`enchanted_count_increase`, 72), no
explosion decay (147), no `killed_by_player` gating (21), no
`random_chance` gating (19), no silk-touch alternatives anywhere.
The 26.2 tree has all three halves of the fix (lines 1386, 1507,
1534, 2042, 2097); 1.21.7 has none of them.

### B12-G03 — MAJOR: 11 of 12 special recipes never load

`horizon-client/src/crafting.rs`, `Recipes::from_sources`. Two
sub-causes, one root: the parser knows only the 26.x special-recipe
form.

(a) **8 handled specials are dropped because 1.21.7's files are
bare.** In 1.21.7 every special recipe file is just
`{"type": "minecraft:crafting_special_*", "category": "misc"}` — the
ingredients live in vanilla Java code. The parser requires the
ingredients from the JSON (`ing("banner")`, `ing("shell")`,
`v["shapes"]`, `result()`, …), so with the fields absent the parse
returns `None` and the recipe is silently dropped. Affected: banner
duplicate, shield decoration, book cloning, firework rocket, firework
star, firework star fade, map extending, decorated pot. Only
`repairitem` loads (it needs no fields). Verified the bare form is
also 1.21.6's and 1.21.10's (named jars) — the data-driven special
form is post-E6, so this likely hits every E6 version, not just
1.21.7.

(b) **3 specials have no parser arm at all** — the parser knows only
their 26.x type names: `crafting_special_armordye` (parser has
`crafting_dye`), `crafting_special_tippedarrow` (parser has
`crafting_imbue`), `crafting_special_mapcloning` (26.x does map
cloning as a transmute; 1.21.7's 32 transmute recipes are bundle /
shulker-box dyeing only). No other construction site for these
`Special` variants exists — the parser is the only source, so the
behaviours (armour dyeing, tipped arrows, map cloning) are
unreachable.

Everything else loads: shaped (673), shapeless (295), cooking
(smelting 72 / blasting 24 / smoking 9 / campfire 9), transmute (32),
stonecutting (254, via `stations.rs`), smithing transform (9) /
trim (18, via `stations.rs`), repairitem (1). All 3,239 ingredient
slots in vanilla data are plain strings, so the parser's string-only
ingredient handling is sufficient for vanilla (see B12-G05 for the
datapack-only gap). Result forms (`{"id", "count"}`, with optional
`components`) are all handled by `recipe_result_id` /
`Template::parse`.

### B12-G04 — minor: the port's MobEffects table carries a 26.x-only effect

`horizon-client-core/src/potions.rs`, `EFFECTS`: 40 entries vs
vanilla 1.21.7's 39. The extra is `breath_of_the_nautilus` (colour
65518, Beneficial), which does not exist in 1.21.7 (no
`minecraft:breath_of_the_nautilus` in the jar's registry dump or
`MobEffects` class). It is appended last, so the 39 vanilla protocol
ids are unaffected (order verified against
`registries.json` `minecraft:mob_effect` protocol ids 0–38).

Everything else in the table verified against the jar with
`javap -c` on `MobEffects.<clinit>`: all 39 colours, all 39
categories (B/H/N), and the instant flags — exactly
`instant_health`, `instant_damage`, `saturation` are instant
(`HealOrHarmMobEffect` / `SaturationMobEffect` extend
`InstantenousMobEffect`; the other 36 do not).

### B12-G05 — minor: object-form recipe ingredients are rejected (datapack scope only)

`crafting.rs`, `Ingredient::parse_object`: a JSON-object ingredient
requires a `neoforge:ingredient_type` or `type` key, and returns
`None` otherwise. Vanilla's own object forms (`{"item": "..."}`,
`{"tag": "..."}`) have neither key, so any recipe using them fails to
parse (shaped: whole recipe dropped; shapeless/cooking: dropped).
No vanilla 1.21.7 recipe uses object ingredients (all 3,239 slots are
strings), so this affects only player datapacks / mod recipes that
use the object form. Recorded as a gap, not a vanilla parity defect.

## Passes, with the evidence

- **Advancements (triggers only).** 53 distinct triggers used across
  the 1,520 files. 52 are fired from gameplay code: 29 via direct
  `self.trigger("minecraft:…")` calls, 16 via the generic
  `trigger_block` / `trigger_item` / `trigger_player` /
  `trigger_entity` / `trigger_distance` call sites, plus
  `inventory_changed` (`check_inventory_trigger`), `recipe_unlocked`
  (`recipe_unlocked`), `player_killed_entity` (`trigger_killed_as`),
  `entity_killed_player`, `kill_mob_near_sculk_catalyst`,
  `player_interacted_with_entity`, `player_sheared_equipment`,
  `thrown_item_picked_up_by_*` (verified call sites). The one
  unfired trigger, `minecraft:impossible`, is used only by
  `recipes/root.json` and never fires in vanilla either — pass by
  design. `Tree::by_trigger` indexes triggers from the data files
  themselves, so no trigger is dropped at load. Item-predicate
  evaluation reads the 1.21.7 form (`items` / `count` / `components`
  / `predicates`). (Criterion *condition* semantics beyond the
  trigger firing are out of this batch's triggers-only scope.)
- **Tags.** All 589 files byte-identical, across 13 kinds (block 194,
  item 179, worldgen 95, entity_type 39, damage_type 33, enchantment
  22, banner_pattern 11, game_event 5, instrument 3,
  point_of_interest_type 3, fluid 2, dialog 2, painting_variant 1).
  `block-tags.json` (194 keys) is byte-faithful to the jar's block
  tags, values included. Runtime loaders read the same files
  (`level/data.rs` reads `block-tags.json` for blocks,
  `data/minecraft/tags/fluid/` for fluids; `load_tag_lists` is
  generic over kind). The pilot's adjacent observation — "prepared
  .local renames four 1.21.7 tool tags to their 26.x `mineable/*`
  names" — does **not reproduce**: the jar itself ships
  `tags/block/mineable/{axe,hoe,pickaxe,shovel}.json`, the prepared
  copies are byte-identical, and no rename code exists in the
  1.21.7 tree (the only `mineable` references are test fixtures and
  mod-pack tooling). Tool correctness in-game comes from the
  `minecraft:tool` component rules in `items.json`, not from tags.
- **Enchantments.** Loader (`loot.rs` `LootData::load`,
  `horizon-items/src/enchant.rs`) reads exactly the 1.21.7 fields:
  `supported_items`, `primary_items`, `exclusive_set`, `max_level`,
  `weight`, `min_cost`/`max_cost` (`base` /
  `per_level_above_first`), `slots`, `effects`. All 29 effect
  component keys used by the 42 files are evaluated somewhere: 17
  via the data-driven effects path, 12 by enchantment id in gameplay
  code (Lure / Luck of the Sea in `fishing.rs`, Quick Charge /
  Multishot in `crossbow.rs`, Loyalty / Riptide / Channeling in
  `trident.rs`, Binding in `armor_stands.rs` / `mob_looting.rs`,
  Silk Touch XP in `spill.rs`). Effect *values* (damage numbers,
  durations) are behaviour depth for other batches, not checked here.
- **Damage types.** Loader
  (`horizon-world/src/damage_source/resources.rs`) parses
  `message_id` (required), `exhaustion` (required), `scaling`
  (validated), `effects` (default `hurt`, validated), and
  `death_message_type` (default `default`, validated). Every value
  in the 49 files is inside the accepted lists (scaling:
  `always` ×4 / `when_caused_by_living_non_player` ×45; effects:
  burning ×7, drowning, freezing, poking, thorns; death_message_type:
  `fall_variants` ×2, `intentional_game_design` ×1).
- **Status effects.** See B12-G04: the 39 vanilla entries match the
  jar exactly (colour, category, instant, protocol order).

## Declared override

- **supports_* tag synthesis** (19 block + 3 fluid tags). The port
  synthesises the 1.21.11-era `supports_*` tags 1.21.7 lacks, from
  jar-verified 1.21.7 equivalents, in
  `horizon-world/src/worldgen/tags.rs`
  (`synthesize_legacy_support_tags`, applied on the worldgen and the
  runtime tag-loading paths). The jar has zero `supports_*` tags, so
  nothing is overridden — the synthesis only fills absent tags
  (`entry().or_insert`). `supports_frogspawn` is deliberately
  block-empty (vanilla 1.21.7 is fluid-only). Record: fix 6e6cb55b
  (merged 2026-11-22), jar-verified. This is a deliberate, recorded
  divergence, not a defect. No D1 overrides list for registries
  exists in `pins/1.21.7/` (checked `delta-verdict.json`,
  `profile.json`, `sweep-items.json`); this synthesis is recorded in
  the fix commit and PORT-NOTES, cited here.

## Method notes / limitations

- The data diff is byte-level (`diff -r` over the extracted jar
  `data/` vs the prepared `.local` copy): 0 differing files in all
  six data-driven families. The prepared data is a faithful copy of
  the jar's datapack — the divergences are all in the code that reads
  it, which is why this batch's behaviour spot-checks carried the
  findings.
- `loot_fill` was built and run (`cargo build -p horizon-world
  --example loot_fill`, dev profile, `-j 1`, ~18.5 min under load;
  run against `mob-spawn/resources`). No second probe was built for
  the items.rs engine: the owner already confirmed its symptom
  in-game, and the code path is unambiguous.
- `registries.json` `minecraft:block_type` (255 entries, unverified
  provenance per §1.1) was not used for any count or check.
- Loot *number providers* (`{"type": "minecraft:uniform", …}`) and
  pool *entry* types (`minecraft:item`, `alternatives`, …) correctly
  use the `"type"` discriminator in 1.21.7 — only the condition and
  function discriminators were renamed in 26.x. The defect is
  specific to those two keys (plus the singular/plural `condition`
  vs `conditions` change).
- Recipe *book* display of the 11 dropped specials was not checked;
  the book reads the same `sources`, so they likely list but do not
  craft. Behaviour of enchantment effect *values* and advancement
  criterion *condition semantics* belong to other batches.
- Workdir note: the reused workdir had 20,718 zero-byte files (all
  outside this batch's sparse scope — other versions' tools/crates,
  truncated by the runtime restart that cut off the previous agent;
  `versions/1.21.7`, `docs`, `pins`, `tools` were intact). Restored
  with `git restore .` before this batch's work; no previous agent
  work product was found to keep (no batch-12 files existed).
