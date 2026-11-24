# Batch 12 — Data-driven registries (1.21.10)

## §0 Execution status

Checks + triage only, 2026-11-25. All checks in scope completed
fresh against 1.21.10's own jar/data and the 1.21.10 tree. No fixes,
no code changes, no commits, no git commands in the worktree.

**Prepare note:** Batch 0's prepare had NOT landed when this batch
started (the `b0/` scratch dir was empty and no
`versions/1.21.10/.local/` existed in the worktree). This batch
therefore ran the 26.1.2 Batch 0 recipe itself: built
`horizon-prepare` (release, `taskset -c 0`, `-j 1`, 24m17s, exit 0)
and ran it (`./target/release/horizon-prepare --minecraft-dir
~/workspace/mcir-work/e6-deltas/versions/1.21.10/game`, the
lab-prepared 1.21.10 minecraft dir with jars + libraries). The
first run was killed by a runtime restart mid-exporter-compile
(it had already copied assets/data and written the worldgen
bundle); the re-run completed cleanly (1,828.9 s, exit 0,
`horizon-prepare.json` written). The byte-level diffs below are
against that FINAL prepared tree
(`versions/1.21.10/.local/horizon-client/1.21.10/`). A pre-existing
prepared tree from the port candidate
(`~/workspace/mcir-work/candidate-test-1.21.10/.local/`) was diffed
first as a cross-check and is also byte-identical to the jar (0
differing files), but the candidate's code differs from the sweep
worktree in the key loot/recipe files, so all behaviour claims rest
on the sweep worktree's tree and this batch's own prepare run.

**`loot_fill` was built AND run for 1.21.10** (dev profile,
`taskset -c 0`, `-j 1`, 28m29s under heavy shared-cell contention,
exit 0; the run itself took under a second): 7 tables × seeds 1–5
against the real prepared data, with the `minecraft:`-prefixed
table ids the example requires (bare names return empty — `fill`
does no name normalisation, `loot.rs:1163`; see the method notes).
The run demonstrates B12-G02 live; see §4.

## §1 Scope

Recipes (1,461), loot tables (1,320), advancements (1,574,
triggers only), tags (612), enchantments (42), status effects
(39), damage types (49). Method 4 (PARITY-SWEEP.md §3):
byte-level diff of the port's prepared data against the vanilla
1.21.10 data files, plus behaviour checks of the loaders — where
the LOADER disagrees with the data's form even though the data
itself is faithful.

Vanilla spec: the named 1.21.10 game jars
(`e6-deltas/versions/1.21.10/jars/{client,server-game}-named.jar`,
sha1-verified against the brief's pins: client
`715f58165acfaf8b61ab921064f08c202b41bd0c`, server-game
`ba1fc89ead95b8ebdb8727f84e2bd487533c8b7f`; `server.jar` is a
bundler jar and was never javap'd), `data/minecraft/*` extracted
to the batch scratch dir (`jar-data/`, 7,512 files), `javap -c`
on the client jar's `MobEffects` class for effect
colours/categories/instant flags, on `RecipeSerializer.<clinit>`
for the cooking-time defaults, and on seven block classes for the
survival tag forms. Port data:
`versions/1.21.10/.local/horizon-client/1.21.10/data/minecraft/`
(this batch's prepare run, see §0).

**Headline:** the prepared data is faithful — every one of the
5,058 data-driven files is byte-identical to the 1.21.10 jar —
but the 1.21.10 tree reads 26.x forms in THREE places, not just
the loot engines: both loot engines read 26.3 key forms against
1.21.10's old-form data (B12-G01/B12-G02, the 1.21.7 defects
reproducing in full), the survival code reads eleven `supports_*`
tag names that do not exist in 1.21.10 and resolves them all to
empty sets, so seven block types can never survive on valid
ground (B12-G03), and the render asset loader hard-requires
three 26.x-only tags (`blocks_motion_in_heightmap`,
`blocks_motion_in_heightmap_no_leaves`, `blocks_fluid_flow`)
that 1.21.10 never shipped, so the client cannot start (B12-G04).
The 1.21.7 tree fixed the motion-tag half by deriving the sets
from block data; the 1.21.10 tree never carried that fix, and it
carried neither the 26.2 loot fixes nor the 1.21.7 `supports_*`
synthesis. Two 1.21.7/26.1.2 defects ARE fixed here: the special
recipes (hardcoded arms for all 12 bare-form specials) and the
cooking-time defaults (per-type 200/100/100/100, javap-verified).

NOTE: `sounds.json` is absent from the prepared tree (needs the
separate `horizon-prepare sounds` step against a launcher asset
store the VM lacks) — expected, not a port defect, and no check
in this batch needed it. `en_us.json` IS present
(`assets/minecraft/lang/en_us.json`) and md5-identical to the
client jar's file (`d166df536bf7f617599990f8437ba18c`).

## §2 Method

- Byte-level `diff -r` of the extracted jar `data/minecraft/` vs
  the prepared `.local` copy, per family (recipes, loot tables,
  advancements, tags, enchantments, damage types) — plus a full
  `data/minecraft/` diff (0 differing files overall).
- Loot census in Python (`evidence/census.py`): a walk that
  recurses into every dict and list, including `terms`/`term`
  combinator arrays (the 26.2 coordinator note's correction is
  built in). Plural-`conditions` levels resolved by a
  level-aware walk that descends into inline tables nested in an
  entry's `value`; cross-checked by an independent
  pools-with-`rolls`-and-`conditions` signature count (952,
  including the 6 nested in `equipment/trial_chamber.json`).
- Recipe parser-arm emulation in Python
  (`evidence/recipe_emulation.py`): mirrors `from_sources` /
  `parse_recipe_sources` / `Ingredient::parse` /
  `Template::parse` field by field, with ingredient/tag
  resolution against the jar's item tags (flattened recursively,
  mirroring `load_tag_lists`) and the result codec range.
- Advancement trigger census over the 1,574 files + a
  firing-call-site grep for every distinct trigger across the
  1.21.10 crates (`evidence/trigger_coverage.txt`,
  `evidence/trigger_sites.txt`).
- Tag comparison: `block-tags.json` parsed and compared key by
  key against the jar's block tags (values are JSON-encoded
  strings of the whole tag object; compared parsed). Runtime
  tag-read audit: every `supports_*` / motion / fluid-flow tag
  name the tree reads, checked against the jar's tag files, with
  the missing ones traced to their readers and their vanilla
  1.21.10 forms verified by javap on the owning block classes.
- Enchantment loader read + effect-component wiring grep for
  all 29 keys (by key string, then by enchantment id for the 12
  keys wired that way); damage-type loader validation read +
  value census.
- `javap -c` on `MobEffects.<clinit>` (parsed programmatically
  for all 39 effects; the parser handles `ldc_w` past
  constant-pool index 255) + `javap` on all 14 distinct effect
  subclasses for the instant set + protocol-id order vs the
  1.21.10 prepared `registries.json` `minecraft:mob_effect`.
- `javap -c` on `RecipeSerializer.<clinit>` for the per-type
  cook-time defaults (the int each serializer passes to
  `AbstractCookingRecipe$Serializer.<init>`).
- `javap -c` on `SugarCaneBlock`, `CocoaBlock`,
  `MangrovePropaguleBlock`, `FrogspawnBlock`, `AzaleaBlock`,
  `NetherSproutsBlock`, `NetherWartBlock` for their vanilla
  1.21.10 survival forms.
- Live `loot_fill` run (7 tables × seeds 1–5) against the real
  prepared data — see §0.
- Batch 0's oracle table
  (`docs/sweep/1.21.10/batch-0-oracles.json`) had NOT landed when
  this batch finished, so no oracle table was consulted. None
  was needed: this batch is driver-independent (javap + data
  diffs + code reads + the prepare run + the `loot_fill` run).
  No oracle defects, no oracle repairs.

## §3 Checks table

| Registry | Data diff (byte-level) | Behaviour check | Result |
|---|---|---|---|
| Recipes (1,461) | 1,461/1,461 identical | Loadability per recipe (parser-arm emulation incl. tag resolution): 1,461 load, 0 dropped; 22/22 recipe types have a parser arm | pass |
| Loot tables (1,320) | 1,320/1,320 identical | items.rs × 1,320 tables: 134 pass / 1,186 gap; loot.rs live `loot_fill` run (tables already counted under items.rs, not double-counted): conditions/functions ignored throughout | **1,186 gap checks** (B12-G01, B12-G02) |
| Advancements (1,574) | 1,574/1,574 identical | 53 distinct triggers: 52 fired, `impossible` never fires by vanilla design | pass |
| Tags (612) | 612/612 identical | `block-tags.json` 203/203 exact-equal; 11 `supports_*` runtime tag reads resolve empty (gap); 3 motion/fluid-flow tags hard-required at startup and absent (gap) | **14 gap checks** (B12-G03, B12-G04) |
| Enchantments (42) | 42/42 identical | Loader reads all 1.21.10 fields; all 29 effect component keys wired (data-driven or by enchantment id) | pass |
| Status effects (39) | n/a — code-defined in 1.21.10, no `mob_effect/` data files in the jar | 39/39 colours, categories, instant flags, protocol order match the jar; the port table carries one extra 26.x-only effect (B12-G06, 0 gap checks) | pass (39 checks) + 1 minor item |
| Damage types (49) | 49/49 identical | Loader validates all fields; every value in the files is accepted | pass |

Checks total: **8,248** (5,058 data-diff entries + 3,190
behaviour checks). Pass: **7,048**. Gap: **1,200** — collapsing
to **6 defects**, 4 major (B12-G01–G04) and 2 minor (B12-G05,
B12-G06, both 0 gap checks: B12-G05 is datapack-scope, B12-G06
is an extra table entry appended after the 39 vanilla ids).
Behaviour subtotal reconciles as 1,461 recipe loads + 22
recipe-type arms + 1,320 items.rs tables + 53 triggers + 203
block-tags + 11 supports_* reads + 3 motion-tag reads + 29
enchantment keys + 39 effects + 49 damage types = 3,190; the
loot.rs live run covers tables already counted under items.rs
and is not double-counted.

Behaviour/non-behaviour split: as-tallied 8,248 (8,248
behaviour + 0 non-behaviour). Batch 12 is not one of the three
batches that count non-behaviour checks as-tallied.

Counting rules: a table counts as a gap check for items.rs when
it carries ≥1 condition (the engine evaluates every object
condition as false, so conditioned pools/entries are dropped)
or ≥1 function (the engine never reads the `functions` arrays,
so no function runs). Tables with neither (134) roll correctly
through the engine and pass. The loot.rs defect is recorded as
its own item with 0 additional gap checks, per the settled
no-double-count rule. A runtime tag read counts as a gap check
when the tag name it reads does not exist in 1.21.10's data and
no synthesis supplies it (11 `supports_*` names, 3
motion/fluid-flow names); each is a distinct load the tree
performs against the 1.21.10 datapack.

No oracle defects, no oracle repairs: this batch is
driver-independent (javap + data diffs + code reads + the
prepare run + the `loot_fill` run). No fixes made.

## §4 Defects

### B12-G01 — MAJOR: items.rs evaluates every loot condition as false and never runs any loot function; survival block-breaking drops nothing from conditioned tables

`horizon-items/src/items.rs`, two key-form defects in `LootContext`
(line numbers verified fresh in the 1.21.10 tree):

1. **Conditions.** `condition()` (line 1986) reads the kind ONLY from
   `o.get("type")` (line 1991) — there is no
   `.or_else(|| o.get("condition"))` dual-form read (contrast the
   26.2 tree, `items.rs:2090`). 1.21.10's data uses the `condition`
   key throughout (1,536 occurrences, 0 `type`-keyed), so every
   object-form condition's kind is `""` and the match falls to
   `_ => false`. `conditions()` (line 2112) DOES consult the plural
   `conditions` array — every element fails. Nested conditions fail
   the same way (`inverted`'s `term`, `any_of`/`all_of`'s `terms`).
2. **Functions.** `evaluate()` applies `pool["modifier"]` and
   `table["modifier"]` (lines 1873-1874), `expand()` pushes
   `e["modifier"]` (line 2146, applied by `choose()`), `nested()`
   applies `entry["modifier"]` (line 2287) — but 1.21.10's data has
   ZERO `modifier` keys anywhere and puts functions in `functions`
   arrays (950: 931 entry-level, 10 pool-level, 9 table-level).
   `modifiers()` (line 2301) itself HAS the function-discriminator
   dual-form fix (`f["type"].as_str().or(f["function"].as_str())`,
   line 2311) — dead code, because the container key is never read.
   No function ever runs through this engine on 1.21.10 data.

Impact, counted over all 1,320 tables: **1,186 tables** carry ≥1
condition or ≥1 function — blocks 1,050/1,083 (1,021 with a
condition, 225 with a function), entities 58/104, chests 54/56,
gameplay 6/26, dispensers 3/3, equipment 3/3, spawners 3/5,
shearing 3/22, harvest 2/3, archaeology 1/6, carve 1/1,
charged_creeper 1/6, pots 1/1, brush 0/1. Only 134 tables (neither
conditions nor functions) roll correctly.

Survival impact is direct: `Game::block_loot_at`
(`game.rs:3694`) → `block_drops_by_player` (`items.rs:1562`) →
`block_drops_raw` (`items.rs:1623`) → `ctx.evaluate(table, random)`
(`items.rs:1660/1674`). Every pool with a condition is skipped in
`evaluate()` and every entry with a condition is rejected in
`expand()`, so a block whose table is fully conditioned drops
nothing — the same symptom the owner confirmed in-game for 1.21.7
("survival block-breaking drops nothing"). 1,021 of 1,083 block
tables carry ≥1 condition. `/loot` routes every table type through
this engine too (`loot_table_for_command`, `items.rs:1688`;
`entity_drops_for_command` / `entity_drops_with_target`,
`items.rs:1793/1805`; `block_drops_by_player`), so command rolls
lose conditions and functions alike. The load path does no
normalisation (`loot.rs` `load_dir`, lines 773-790, parses each
file straight into the shared map), so the raw old-form JSON is
exactly what the engines roll.

**Post-fix residual (recorded here, not a separate item):** once
the key forms are fixed, the 26.2-style partial-function-set
defect surfaces. `modifiers()` has 12 arms (`set_count`,
`set_damage`, `apply_bonus`, `enchanted_count_increase`,
`copy_state`, `explosion_decay`, `set_potion`,
`enchant_with_levels`, `enchant_randomly`, `limit_count`,
`set_name`, `set_components`; default `_ => {}`, line 2507) and
is missing 7 of the 19 function kinds 1.21.10's data uses:
`copy_components` (71 tables, all blocks), `furnace_smelt` (16,
entities), `set_enchantments` (4: 3 equipment, 1 chest),
`set_stew_effect` (3: 1 archaeology, 2 chests),
`exploration_map` (3, chests),
`set_ominous_bottle_amplifier` (3: 2 chests, 1 entity),
`set_instrument` (1, chest) — union 101 tables (verified
disjoint fresh from the jar data). `expand()` (line 2127) has no
`minecraft:tag` arm (default `_ => false`, line 2219), losing
the single tag entry (`entities/creeper.json`, the music-disc
entry). Union + tag = **102 tables** — exactly the 26.2 run's
B12-G01 count, the same defect waiting underneath. The 26.2
tree reads `["functions"]` + dual-form discriminators
(`items.rs:1969-1970/2090/2246/2387`, `loot.rs:1042/2126/2164`)
— the 26.2 playtest fixes were never carried to 1.21.10.

### B12-G02 — MAJOR: the loot.rs engine ignores all loot conditions and all loot functions

`horizon-world/src/loot.rs` is written for the 26.3 loot format
throughout, and 1.21.10 writes the old form at every level (line
numbers verified fresh):

- `condition()` (line 1368) reads the kind from `c["type"]` only
  (line 1377) — 1.21.10's 1,536 conditions all use the `condition`
  key.
- `modify()` (line 1478) consults singular `m["condition"]` only
  (line 1488) — 1.21.10's 184 function-level condition lists are
  all plural `conditions` arrays.
- `run()` (line 1512) reads the kind from `f["type"]` only (line
  1514) — 1.21.10's 1,362 functions all use the `function` key;
  the default arm `_ => {}` (line 1937) no-ops anything it does
  see.
- `table_raw()` (line 2021) checks singular `pool["condition"]`
  (line 2028) and applies `pool["modifier"]` / `table["modifier"]`
  (lines 2039, 2026) — 1.21.10 has 952 plural pool `conditions`
  arrays (including 6 nested inside an entry's `value`, in
  `equipment/trial_chamber.json`) and 0 `modifier` keys.
- `expand()` (line 2080) checks singular `entry["condition"]`
  (line 2081) — 1.21.10's 245 entry-level condition lists are all
  plural, so every entry passes its gate.
- `create()` (line 2131) applies `entry["modifier"]` (line 2133)
  — 0 in the data, so no entry function ever runs.
- The `minecraft:tag` arm reads `entry["items"]` (lines 2110,
  2146) — 1.21.10's tag entry uses the `name` field
  (`minecraft:creeper_drop_music_discs`), so `holder_set` gets
  null and returns an empty list (contrast the 26.2 tree's
  `tag_entry_items`, `loot.rs:1042`, which falls back to `name`).

Net: conditions are never evaluated (every gate passes) AND
functions never run (every modifier is a silent no-op). The
engine's arm coverage is otherwise good — `run()` has 26 arms
covering 15 of the 19 function kinds in the data, missing only
the block-only kinds `apply_bonus`, `copy_components`,
`copy_state`, `explosion_decay` (an engine-scope boundary, not a
defect: `loot.rs` never serves block tables in gameplay) — but
the arms are unreachable on 1.21.10 data because of the key
forms.

Precision note: the file DOES contain one `f["functions"]` read,
at line 1516 — inside the `minecraft:sequence` arm of `run()`,
which iterates a sequence function's nested functions. It
changes nothing: the arm is reached only when `f["type"] ==
"minecraft:sequence"` (line 1514), and 1.21.10's data uses the
`function` key — and `minecraft:sequence` is not among the 19
function kinds in the data at all. The only other `functions`
occurrence in the file (line 3248) is test JSON.

**Demonstrated live** with the existing `loot_fill` example
against the real prepared data (7 tables × seeds 1–5,
`evidence/loot_fill.txt`, 148 lines):

- Every stack count is 1 — `set_count` never runs (dungeon
  bone/redstone/coal, rotten flesh, string, gunpowder all ×1).
- The gated `music_disc_lava_chicken` dropped on ALL 5 zombie
  seeds — the `entity_properties` + `random_chance` gate on the
  disc pool is never evaluated, so the entry always passes.
- `blocks/stone` → `minecraft:stone` on all 5 seeds (vanilla:
  cobblestone without silk touch — the silk-touch alternative
  child wins unconditionally because the `match_tool` condition
  is never evaluated).
- `blocks/diamond_ore` → `minecraft:diamond_ore` on all 5 seeds
  (vanilla: diamond).
- `entities/creeper` → gunpowder ×1 only on all 5 seeds — the
  tag entry (music discs via `#creeper_drop_music_discs`) never
  expands, and `set_count` never applies (gunpowder should be
  0–2).
- No enchantments anywhere — `enchant_randomly` /
  `enchant_with_levels` never run (ancient-city books come out
  unenchanted).

Affects entity drops, chest/container loot, fishing, trial
spawners and vaults in gameplay — everything `loot.rs` serves.
0 additional gap checks: these tables are already counted under
items.rs (B12-G01).

### B12-G03 — MAJOR: survival code reads eleven `supports_*` tag names that do not exist in 1.21.10; all resolve empty, so seven block types can never survive on valid ground

The 1.21.10 jar ships ZERO `supports_*` block tags and only two
fluid tags (`lava`, `water`). The 1.21.7 tree answered this by
synthesising the tags (`synthesize_legacy_support_tags`, a
declared override in the 1.21.7 sweep). The 1.21.10 tree has NO
synthesis code anywhere, and the port adapted only SOME survival
checks to 1.21.10-native forms (dirt/sand tags,
`bamboo_plantable_on`, `big_dripleaf_placeable`, hardcoded
farmland/end stone/magma — see the comments in
`block_survival.rs`). Eleven tag reads still point at 26.x names
and resolve to EMPTY sets, in BOTH survival implementations:

- `block_survival.rs` `tag_states`/`fluid_tag` (missing keys
  yield empty sets silently — `tags.get(name).into_iter().flatten()`,
  lines 482-497): `supports_cocoa` (:593),
  `supports_mangrove_propagule` (:600),
  `supports_hanging_mangrove_propagule` (:601),
  `supports_frogspawn` (:609, block + fluid :619),
  `supports_sugar_cane` (:611),
  `supports_sugar_cane_adjacently` (:612, block + fluid :620).
- `horizon-prepare/src/survival.rs` export table, flowing into
  `SurvivalDefinitions` as `TagBelow` rules resolved through the
  same empty-yielding lookup (`bound_tags`, :573-576):
  `supports_azalea` (:76), `supports_nether_sprouts` (:85),
  `supports_nether_wart` (:87).
- `level/survive.rs` (the server-level implementation) reads the
  same empty sets from the level `Tags` struct, whose
  `resolve()` defaults missing tags to empty
  (`level/data.rs:399-401`, `.unwrap_or_default()`): cocoa
  (:150), hanging mangrove (:256), sugar cane (:341, :348-349),
  azalea (:403), mangrove propagule (:430), nether sprouts
  (:432), nether wart (:433), frogspawn (:477-478).

Every affected block's vanilla 1.21.10 form was verified by
`javap -c` on the 1.21.10 client-named jar — none of them uses a
`supports_*` tag:

| Block | Vanilla 1.21.10 survival form (javap) | Port reads | Result |
|---|---|---|---|
| Sugar cane | below is `#dirt`/`#sand` + water adjacent (`SugarCaneBlock.canSurvive`: `BlockTags.DIRT`/`SAND`, `FluidTags.WATER`) | empty `supports_sugar_cane` set; `Rule::SugarCane` returns false unless the neighbour is in it (`block_survival.rs:1104`) | can never survive on dirt/sand |
| Cocoa | behind is `#jungle_logs` (`CocoaBlock.canSurvive`: `BlockTags.JUNGLE_LOGS`) | empty `supports_cocoa` set | can never survive on jungle logs |
| Mangrove propagule | `SaplingBlock.mayPlaceOn` or clay; hanging: mangrove leaves above (`MangrovePropaguleBlock`: `Blocks.CLAY`, `Blocks.MANGROVE_LEAVES`) | empty `supports_mangrove_propagule` / `supports_hanging_mangrove_propagule` sets | can never survive either way |
| Frogspawn | fluid below is water, fluid above empty (`FrogspawnBlock.mayPlaceOn`: `Fluids.WATER`/`Fluids.EMPTY`) | empty `supports_frogspawn` fluid mask + block set | can never survive on water |
| Azalea | clay or `VegetationBlock.mayPlaceOn` (dirt tag/farmland) | empty `supports_azalea` set | can never survive |
| Nether sprouts | `#nylium`, soul soil, or `VegetationBlock.mayPlaceOn` | empty `supports_nether_sprouts` set | can never survive |
| Nether wart | soul sand only (`Blocks.SOUL_SAND`) | empty `supports_nether_wart` set | can never survive |

The remaining `supports_*` fields in the level `Tags` struct
(`supports_cactus`, `supports_wither_rose`, `supports_lily_pad`,
`supports_chorus_plant`, plus the `supports_lily_pad` fluid
field) have NO gameplay readers — dead fields, harmless, not
counted. 11 gap checks: the 11 unique tag names actually read
(9 block + 2 fluid).

### B12-G04 — MAJOR: the asset loader hard-requires three 26.x-only tags that 1.21.10 never shipped; the client cannot start

`horizon-client-render/src/assets.rs`, `Assets::load` (line 186)
reads `block-tags.json` and REQUIRES three tags, erroring if any
is absent:

- `minecraft:blocks_motion_in_heightmap` (:212-216,
  `.ok_or_else(|| invalid("missing motion tag"))?`)
- `minecraft:blocks_motion_in_heightmap_no_leaves` (:217-221,
  `"missing no-leaves motion tag"`)
- `minecraft:blocks_fluid_flow` (:224-229,
  `"missing fluid flow tag"`)

None of the three exists in 1.21.10: the jar's 203 block tags
contain no `blocks_motion*` or `blocks_fluid_flow` file, and
the prepared `block-tags.json` (203/203 exact-equal to the jar)
contains none either. No synthesis or derivation exists
anywhere in the 1.21.10 tree — `TagDefinitions::build`
(`worldgen/tags.rs:149`) does not synthesise them, and no
`horizon-prepare` step writes them. The load therefore fails
with "missing motion tag" before the game can start — the
exact crash the 1.21.7 port hit on its first laptop launch
(Nov 21), fixed there by deriving the sets from block data
instead of reading tags (`versions/1.21.7/.../assets.rs:212-283`:
"1.21.7 has no `blocks_motion*`/`blocks_fluid_flow` tags (26.3
added...)", `blocks_fluid_flow: blocks_motion.into()`). That
fix was never carried to the 1.21.10 tree. 3 gap checks: the
three hard-required tag reads. (Batch 3's B3-G22-equivalent may
be the same defect from the startup side — flagging for the
coordinator to fold, not to double-count.)

### B12-G05 — minor: object-form recipe ingredients are rejected (datapack scope only)

`crafting.rs`, `Ingredient::parse_object` (line 123): a
JSON-object ingredient requires a `neoforge:ingredient_type` or
`type` key, and returns `None` otherwise. Vanilla's own object
forms (`{"item": "..."}`, `{"tag": "..."}`) have neither key, so
any recipe using them fails to parse (shaped: whole recipe
dropped; shapeless/cooking: dropped). No vanilla 1.21.10 recipe
uses object ingredients (5,049 string slots + 115 array slots
whose 1,139 elements are all strings, counted as occurrences —
shaped keys counted per pattern cell; 0 object slots), so this
affects only player datapacks / mod recipes that use the object
form. Same code, same data property as 1.21.7's B12-G05 and
26.1.2's B12-G03 — carried forward, not a vanilla parity
defect. 0 gap checks.

### B12-G06 — minor: the port's MobEffects table carries a 26.x-only effect (breath_of_the_nautilus)

`horizon-client-core/src/potions.rs` `EFFECTS` has 40 entries
vs vanilla 1.21.10's 39; the extra is
`breath_of_the_nautilus` (colour 65518, Beneficial), absent
from the jar's `MobEffects` class and from the prepared
`registries.json` `minecraft:mob_effect` (39 entries, protocol
ids 0–38). Appended last, so the 39 vanilla protocol ids are
unaffected (order verified vs registries.json protocol ids
0–38, 0 mismatches). All 39 vanilla colours, categories and
instant flags verified against `javap -c` on
`MobEffects.<clinit>` (instant exactly: `instant_health`,
`instant_damage`, `saturation` —
`HealOrHarmMobEffect`/`SaturationMobEffect` extend
`InstantenousMobEffect`, Mojang's spelling; the other 12
effect subclasses extend `MobEffect` directly). 0 gap checks.

## §5 Re-check verdicts

### 1.21.7 items (PRIMARY — all decided)

- **B12-G01 (1.21.7, MAJOR)** — items.rs conditions always false:
  **REPRODUCES** as this batch's B12-G01 above (same condition
  key-form defect, plus the `functions`-container defect — the
  26.2 tree's fix was never carried to 1.21.10).
- **B12-G02 (1.21.7, MAJOR)** — loot.rs ignores all conditions
  and functions: **REPRODUCES** as this batch's B12-G02 above,
  demonstrated live with the `loot_fill` run (see §4).
- **B12-G03 (1.21.7, MAJOR)** — 11 of 12 special recipes never
  load: **FIXED** in the 1.21.10 tree. `from_sources`
  (`crafting.rs:1557`) has hardcoded arms for all 12 bare-form
  specials (comment at :1589-1594: "1.21.10's specials are
  hardcoded: their JSONs carry only `category` (and `group`)"),
  including the three 1.21.7-era type names the 26.x parser
  lacked (`crafting_special_armordye` :1699,
  `crafting_special_tippedarrow` :1706,
  `crafting_special_mapcloning` :1764); the tag-dependent arms
  resolve against the jar's item tags (`#banners` 16,
  `#dyeable` 6, `#book_cloning_target` 1, `#skulls` 7,
  `#decorated_pot_ingredients` 24). The parser-arm emulation
  loads 12/12. Recorded as slice item B12-G07 (status fixed).
- **B12-G04 (1.21.7, minor)** — extra 26.x effect in the table:
  **REPRODUCES** as this batch's B12-G06 above (same extra
  entry, same colour, appended last; the 39 vanilla entries
  match the jar exactly).
- **B12-G05 (1.21.7, minor)** — object-form ingredients
  rejected: **REPRODUCES** as this batch's B12-G05 above (same
  code at `crafting.rs:123`, same 0-object-slots data property;
  datapack scope only).

### 26.1.2 items (SECONDARY — all decided)

- **B12-G01 (26.1.2, major)** — items.rs conditions always
  false, no functions run: **REPRODUCES** as this batch's
  B12-G01 above (same two key-form defects; 1,186-table impact
  in 1.21.10 vs 1,192 in 26.1.2 — the difference is real data
  difference, 1,320 vs 1,326 tables).
- **B12-G02 (26.1.2, major)** — loot.rs ignores all conditions
  and functions: **REPRODUCES** as this batch's B12-G02 above,
  demonstrated live with the `loot_fill` run (the 26.1.2 run
  had only the static trace).
- **B12-G03 (26.1.2, minor)** — object-form ingredients
  rejected: **REPRODUCES** as this batch's B12-G05 above.
- **B12-G04 (26.1.2, minor)** — flat 200 cooking-time default:
  **FIXED** in the 1.21.10 tree. `crafting.rs:1772-1779` uses
  per-type defaults (smelting 200, blasting/smoking/campfire
  100), matching vanilla 1.21.10 exactly as verified by `javap
  -c` on `RecipeSerializer.<clinit>` (the int each serializer
  passes to `AbstractCookingRecipe$Serializer.<init>`: smelting
  `sipush 200`, the other three `bipush 100`). All 116 vanilla
  cooking recipes carry explicit `cookingtime` anyway. Recorded
  as slice item B12-G08 (status fixed).
- **26.2 B12-G01 (minor)** — items.rs partial function set (102
  tables): SUBSUMED by this batch's B12-G01. The arm gaps exist
  in 1.21.10 (7 missing kinds + no `tag` arm, 102-table
  post-fix residual — the same count as 26.2, union verified
  disjoint fresh) but are unobservable until the key-form fix
  lands; recorded inside B12-G01's evidence, not as a separate
  item.

## §6 Systemic findings

The prepared data is faithful — all 5,058 data-driven files
byte-identical to the 1.21.10 jar, and `block-tags.json`
byte-faithful too. Every defect is code reading the 26.x form
of a format 1.21.10 writes differently and failing silently —
the same bug class as the 1.21.7 pilot's 26.x-tag findings
(G1/G3/G5/G6), now in three places at once:

1. **Loot keys.** 1.21.10 writes the old forms (the `condition`
   discriminator ×1,536, the `function` discriminator ×1,362,
   plural `conditions` arrays ×1,381, `functions` arrays ×950,
   the tag entry's `name` field) while both engines read the
   26.3 forms (`type`, `modifier`, singular `condition`, tag
   `items`). The `type`/`modifier` renames are 26.3 changes,
   and the 26.2 playtest fixes that taught the engines both
   forms were never carried to the 1.21.10 tree.
2. **Survival tags.** 1.21.10 has no `supports_*` tags at all;
   vanilla's survival checks use native tags (`#dirt`, `#sand`,
   `#jungle_logs`, `#nylium`) and hardcoded blocks (clay, soul
   sand, soul soil, mangrove leaves, water). The port adapted
   most checks to those native forms but left eleven reads
   pointing at 26.x names, and every one of them resolves to an
   empty set instead of failing loudly — the tag lookups
   (`tag_states`, `Tags::resolve`) default missing tags to
   empty, so the defect is silent until a player plants sugar
   cane.
3. **Motion/fluid-flow tags.** The asset loader demands three
   26.x-only tags with hard errors; 1.21.7's derivation fix was
   never carried, so the 1.21.10 client fails at the same
   startup point 1.21.7 did before its fix.

The port DOES get the 1.21.10 forms right where it was written
for them: the specials are hardcoded to the bare form, the
cooking defaults are per-type, the transmute arm pins the
1.21.10 material bounds (no `material_count` exists in the
data), and the recipe/advancement/enchantment/damage loaders
all read 1.21.10's forms correctly. The defects are the places
where 26.3-base code survived the port untouched.

## Passes, with the evidence

- **Recipes.** All 1,461 load under a parser-arm emulation
  (type arms + required fields + ingredient/tag resolution
  against the jar's item tags + result codec range): shaped
  700, shapeless 320, stonecutting 254 (via `stations.rs`
  `parse_recipe_sources`, line 805), smelting 73, blasting 25,
  smoking 9, campfire_cooking 9, transmute 32 (bundle/shulker
  dyeing only — map cloning is a special here, not a
  transmute), smithing_trim 18, smithing_transform 9, and 12
  bare-form specials (armour dyeing, banner duplicate, book
  cloning, firework rocket, firework star, firework star fade,
  map cloning, map extending, repair item, shield decoration,
  tipped arrow, decorated pot). The 1.21.7 defect does not
  reproduce: the 1.21.10 parser has hardcoded arms for all 12
  specials and every tag they need resolves. `custom_type`
  (`crafting.rs:1348`) diverts only non-`minecraft:` types, so
  no vanilla recipe is diverted. All shaped patterns'
  characters exist in their `key` dicts (0 bad). Ingredient
  slots: 5,049 string slots + 115 array slots whose 1,139
  elements are all strings, counted as occurrences (shaped keys
  counted per pattern cell); 0 object slots (see B12-G05).
  Result forms: `{count, id}` ×1,258, `{id}` ×157,
  `{components, count, id}` ×16, all handled by
  `Template::parse` / `recipe_result_id`; max count 16 (inside
  the 1..=99 codec range). No transmute file carries
  `material_count`, so the arm's hardcoded 1..1 bounds match
  the data. All 116 cooking recipes carry explicit
  `cookingtime` (see §5, 26.1.2 B12-G04).
- **Advancements (triggers only).** 53 distinct triggers used
  across the 1,574 files (no `spear_mobs` — 26.x-only, absent
  from 1.21.10's data as the brief stated; the inventory's 57
  counts trigger TYPES in the registry, of which 53 are used).
  52 are fired from gameplay code: direct `self.trigger`
  calls (`tick` `game/advancements.rs:1253`, `location`
  `:1287`, `bee_nest_destroyed` `:2036`, `fall_after_explosion`
  `:1465`, `using_item` `:1556`, `slide_down_block` `:1875`,
  `crafter_recipe_crafted` `machines/crafter.rs:198`, …), the
  generic `trigger_block` / `trigger_item` / `trigger_player`
  / `trigger_entity` / `trigger_distance` call sites
  (`placed_block` `game.rs:8334`, `filled_bucket`
  `game.rs:8246`, `consume_item` `consume.rs:53`, `slept_in_bed`
  `game.rs:8872`, `shot_crossbow` `crossbow.rs:401`,
  `summoned_entity` `dragon.rs:4523`, `hero_of_the_village`
  `raids.rs:659`, `avoid_vibration` `sculk.rs:877/958`,
  `allay_drop_item_on_block` `allays.rs:309`, …),
  `trigger_killed` / `trigger_killed_as`
  (`player_killed_entity`, `kill_mob_near_sculk_catalyst`
  `creatures.rs:1731/1861`), `trigger_item_entity`
  (`player_interacted_with_entity`,
  `player_sheared_equipment` `creatures.rs:7145`,
  `thrown_item_picked_up_by_entity` `:1741`,
  `thrown_item_picked_up_by_player` `:1754`),
  `trigger_item_durability` (`item_durability_changed`),
  `trigger_player_hurt_entity` /
  `trigger_entity_hurt_player`, `check_inventory_trigger`
  (`inventory_changed`, `:1144`) and `recipe_unlocked`
  (`:938`). Every trigger was confirmed present in firing code
  by a per-trigger grep across the crates
  (`evidence/trigger_coverage.txt`: 0 missing;
  `evidence/trigger_sites.txt` classifies the sites). The one
  unfired trigger, `minecraft:impossible`, is used only by
  `recipes/root.json` and never fires in vanilla either — pass
  by design. `Tree::by_trigger` indexes triggers from the data
  files themselves (`advancements.rs:62`), so no trigger is
  dropped at load. Item-predicate evaluation reads the 1.21.10
  form (`items` / `count` / `components` / `predicates`,
  `advancements.rs:824-840`). Observation (not a numbered
  defect): `entity_killed_player`'s only firing call site
  found is the LAN guest death path
  (`game/lan_actions.rs:4628`, `guest_killed_player_trigger`);
  no single-player local-death firing call site was found —
  same observation as the 26.2, 26.1.1 and 26.1.2 runs. The
  only advancement using it, `adventure/root.json`, grants via
  the OR'd `player_killed_entity` criterion too, so nothing is
  unreachable; the single-player firing path belongs to the
  batch that owns player-death semantics. Dead 26.x leftover
  (not a defect): `trigger_spear_mobs`
  (`game/advancements.rs:2012-2014`, called from
  `spear.rs:225`) fires a trigger no 1.21.10 advancement uses
  and no 1.21.10 item can reach. (Criterion *condition*
  semantics beyond trigger firing are out of this batch's
  triggers-only scope.)
- **Tags.** All 612 files byte-identical, across 13 kinds
  (block 203, item 190, worldgen 95, entity_type 42,
  damage_type 33, enchantment 22, banner_pattern 11, game_event
  5, instrument 3, point_of_interest_type 3, fluid 2, dialog 2,
  painting_variant 1). `block-tags.json` (203 keys) is exactly
  faithful to the jar's block tags — 203/203 exact-equal
  (values are JSON-encoded strings of the whole tag object;
  compared parsed), no extras, no missing keys. Runtime loaders
  read the same files (`horizon-client-render/src/assets.rs:192`
  reads `block-tags.json`; `load_tag_lists`, `items.rs:569`, is
  generic over kind and flattens nested `#tag` references
  recursively). The 1.21.7 declared override does NOT apply
  here in the synthesis direction: 1.21.10's jar ships ZERO
  `supports_*` tags and NO `synthesize_legacy_support_tags`
  code exists in the 1.21.10 tree — but the tree still reads
  the names, which is B12-G03, and the asset loader still
  demands the motion tags, which is B12-G04. The jar DOES ship
  the 1.21.10-native forms the port adapted to elsewhere:
  `snow_layer_can_survive_on` / `snow_layer_cannot_survive_on`,
  `azalea_grows_on`, `dry_vegetation_may_place_on`,
  `bamboo_plantable_on`, `big_dripleaf_placeable`,
  `small_dripleaf_placeable`, `convertable_to_mud` (Mojang's
  spelling — the level `Tags` struct's `convertible_to_mud`
  field names a tag that does not exist, but it has no
  readers).
- **Enchantments.** Loader (`horizon-items/src/enchant.rs:146-181`)
  reads exactly the 1.21.10 fields: `supported_items`,
  `primary_items` (5 files), `exclusive_set` (18), `max_level`,
  `weight`, `anvil_cost`, `min_cost`/`max_cost` (`base` /
  `per_level_above_first`), `slots`, `effects`. All 29 effect
  component keys used by the 42 files are evaluated somewhere:
  16 via the data-driven `modify_value` path (`enchant.rs:699`)
  and data-driven readers in `enchant.rs` / `loot.rs`
  (`damage`, `damage_protection` via `game/potions.rs:1221`,
  `item_damage`, `knockback`, `ammo_use`,
  `armor_effectiveness`, `projectile_piercing`,
  `repair_with_xp`, `smash_damage_per_fallen_block`,
  `attributes` `enchant.rs:747/763`, `post_attack`,
  `projectile_spawned`, `damage_immunity`, `location_changed`,
  `equipment_drops` `loot.rs:1298`,
  `prevent_equipment_drop` `loot.rs:959`), 13 by enchantment
  id in gameplay code (`block_experience`: Silk Touch XP,
  `spill.rs:483`; `crossbow_charge_time` +
  `crossbow_charging_sounds`: Quick Charge,
  `crossbow.rs:81-102`; `fishing_luck_bonus` +
  `fishing_time_reduction`: Luck of the Sea / Lure,
  `fishing.rs:485-486`; `hit_block`: Channeling,
  `trident.rs:443-446` `channel_rod_hit`;
  `prevent_armor_change`: Binding Curse,
  `armor_stands.rs:1841`; `projectile_count` +
  `projectile_spread`: Multishot, `crossbow.rs:261`; `tick`:
  Soul Speed particles, `game/enchant_effects.rs:1027`
  `tick_soul_speed`; `trident_return_acceleration`: Loyalty,
  `trident.rs:274-299`; `trident_sound` +
  `trident_spin_attack_strength`: Riptide,
  `trident.rs:396,499-520`). 1.21.10 has 29 effect keys vs
  26.1.2's 30 — `post_piercing_attack` does not occur in
  1.21.10's data. `fortune.json` has no `effects` — correct,
  fortune is implemented via loot `apply_bonus`. Effect
  *values* (damage numbers, durations) are behaviour depth for
  other batches, not checked here.
- **Status effects.** 39/39 colours, categories, instant flags
  and protocol order match the jar (`javap -c` on
  `MobEffects.<clinit>`, parsed programmatically). The instant
  set is exactly `instant_health`, `instant_damage`,
  `saturation` (`HealOrHarmMobEffect` / `SaturationMobEffect`
  extend `InstantenousMobEffect` — Mojang's spelling —
  verified by javap; the other 12 effect subclasses extend
  `MobEffect` directly). Protocol ids 0–38 in the 1.21.10
  prepared `registries.json` `minecraft:mob_effect` match the
  port table's order exactly. B12-G06 (the extra nautilus
  entry) is the only blemish.
- **Damage types.** Loader
  (`horizon-world/src/damage_source/resources.rs:127-173`,
  `from_json`) parses `message_id` (required), `exhaustion`
  (required, parsed directly to f32), `scaling` (validated:
  `never` / `when_caused_by_living_non_player` / `always`),
  `effects` (default `hurt`, validated against hurt/thorns/drowning/
  burning/poking/freezing), and `death_message_type` (default
  `default`, validated against default/fall_variants/
  intentional_game_design). Every value in the 49 files is
  inside the accepted lists (scaling: `always` ×4 /
  `when_caused_by_living_non_player` ×45; effects: burning ×7,
  drowning, freezing, poking, thorns, default ×38;
  death_message_type: `fall_variants` ×2,
  `intentional_game_design` ×1, default ×46).

## Declared overrides

None. The 1.21.7 batch's declared override (`supports_*` tag
synthesis) does not exist in the 1.21.10 tree and is not
applied: 1.21.10 ships no `supports_*` tags natively, no
synthesis code exists, and the reads resolve empty instead —
which is B12-G03, a defect, not an override.

## §7 Evidence index

All paths under
`~/workspace/mcir-work/sweep-1.21.10-batches/b12/evidence/`
unless noted. Jar data extracted to `b12/jar-data/`
(`data/minecraft/*` from the 1.21.10 client jar, 7,512 files).

- `data_diff.txt` — byte-level diff of the extracted jar
  `data/minecraft/` vs this batch's prepared tree: 0 differing
  files overall; per-family counts: recipes 1,461, loot 1,320,
  advancements 1,574, tags 612, enchantments 42, damage types
  49 (5,058 data-diff entries). `data_diff_candidate.txt` —
  same diff vs the candidate-test prepared tree, also 0.
- `horizon-prepare-build.log` — release build of
  `horizon-prepare`, exit 0, 24m17s.
  `horizon-prepare.log` — prepare run against the
  lab-prepared 1.21.10 minecraft dir, exit 0, 1,828.9 s.
- `census.py` / `census.json` / `census.txt` — recursive loot
  census (recurse into `terms`/`term` arrays; level-aware
  plural-`conditions` attribution; pool-signature cross-check):
  1,536 conditions (all `condition`-keyed, 0 `type`-keyed, 12
  kinds), 1,362 functions (all `function`-keyed, 19 kinds),
  0 `modifier` keys, 0 singular condition objects, `functions`
  containers 950 (931 entry / 10 pool / 9 table), plural
  `conditions` 1,381 (pool 952 incl. 6 nested in an entry's
  `value` in `equipment/trial_chamber.json`, entry 245,
  function 184); per-family "either" counts; the single tag
  entry (`entities/creeper.json`,
  `{"type":"minecraft:tag","expand":true,"name":"minecraft:creeper_drop_music_discs"}`);
  the 101-table post-fix residual by missing kind, disjoint
  by construction.
- `loot_fill-build.log` — dev-profile build of the
  `loot_fill` example, exit 0, 28m29s.
  `loot_fill.txt` — the live run, 7 tables × seeds 1–5, 148
  lines: every stack count 1; the gated lava-chicken disc on
  all 5 zombie seeds; `blocks/stone` → `minecraft:stone`,
  `blocks/diamond_ore` → `minecraft:diamond_ore` on all 5
  seeds; `entities/creeper` → gunpowder ×1 only; no
  enchantments anywhere.
- `recipe_emulation.py` / `recipe_emulation.txt` — parser-arm
  emulation: 1,461/1,461 load, 0 dropped; 22 types with arms;
  12/12 specials; ingredient-slot census (5,049 string + 115
  array slots, 1,139 string elements, 0 object slots);
  result-form census; `material_count` absence; explicit
  `cookingtime` on all 116 cooking recipes.
- `javap_RecipeSerializer.txt` — `RecipeSerializer.<clinit>`:
  smelting `sipush 200`, blasting/smoking/campfire
  `bipush 100`.
- `javap_SmeltingRecipe.txt`, `javap_BlastingRecipe.txt`,
  `javap_SmokingRecipe.txt`, `javap_CampfireCookingRecipe.txt`
  — the four cooking serializer classes (supporting the
  default-time read).
- `triggers.txt` — the 53 distinct triggers used across the
  1,574 advancement files.
  `trigger_coverage.txt` — per-trigger firing-call-site grep
  result (0 missing). `trigger_sites.txt` — the firing sites
  classified by mechanism, incl. the `entity_killed_player`
  observation and the dead `trigger_spear_mobs` leftover.
- `tags_block.txt` — `block-tags.json` vs the jar's block
  tags: 203/203 exact-equal.
- `mobeffects.txt` / `effects_parse.py` / `effects_jar.json` /
  `effects_jar.txt` — the 39 effects parsed from
  `MobEffects.<clinit>` (name, colour, category) vs the
  port's 40-entry table: 0 mismatches on the 39 vanilla
  entries; the extra `breath_of_the_nautilus`.
  `effect_superclasses.txt` — javap on all 14 effect
  subclasses: instant exactly `instant_health`,
  `instant_damage`, `saturation`.

Method notes and limitations:

- The `loot_fill` example takes full `minecraft:` table ids;
  bare names return empty (no normalisation in `fill`,
  `loot.rs:1163`) — example usability, not a defect. The
  first (bare-name) run returned 0 fills and was re-run
  correctly.
- The prepared tree used for the byte diffs was produced by
  THIS batch's prepare run (§0); the candidate-test tree
  (`candidate-test-1.21.10/.local/`) is also byte-identical
  to the jar, but its code differs from the worktree in the
  key loot/recipe files, so it was used only as a provisional
  cross-check, never as the evidence base.
- The recipe emulation assumes item defaults exist for all 9
  smithing_transform results (all vanilla items, and the
  loader is generic over the prepared item table); a missing
  default would make `parse_recipe_sources` fail that recipe
  — no such failure was observed, and the assumption is
  stated here rather than silently carried.
- The `entity_killed_player` observation (§6 Passes) and the
  single-player death firing path belong to the batch that
  owns player-death semantics; this batch records only that
  the trigger's sole firing call site found is the LAN guest
  path.
- Other nonexistent-tag reads, flagged for their owning
  batches rather than minted here (their vanilla 1.21.10
  forms were NOT javap-verified in this batch): the level
  `Tags` struct's `cannot_support_seagrass`
  (`survive.rs:442`), `cannot_support_snow_layer` +
  `support_override_snow_layer` (`survive.rs:314/316`,
  `weather.rs:113/116` — the jar natively ships
  `snow_layer_can_survive_on` /
  `snow_layer_cannot_survive_on`), `support_override_cactus_flower`
  (`survive.rs:406`), `washed_away_by_fluids`
  (`fluids.rs:360`), and `blocks_motion` (`level/hooks.rs:549`,
  the ice-melt check — vanilla 1.21.10 uses motion-blocking
  code checks, not a tag). The level `Tags` struct has 73
  fields; 24 of them name tags absent from the jar's block
  tags (the 9 counted `supports_*` fields among them). Dead
  fields with no gameplay readers (`convertible_to_mud` —
  the jar spells it `convertable_to_mud`;
  `enables_bubble_column_*`;
  `overrides_mushroom_light_requirement`;
  `supports_cactus` / `supports_wither_rose` /
  `supports_lily_pad` / `supports_chorus_plant` /
  `supports_bamboo`) are harmless and uncounted.
- B12-G04 may overlap Batch 3's startup-blocker item
  (B3-G22-equivalent) — flagged for the coordinator to fold,
  not to double-count.
- The two 26.1.1 items corrected after that sweep (B5-G18
  bell, B10-G04 BE renderers) are out of this batch's scope
  and were not re-minted.

## §8 Verdict

**FAIL on loot + tags.** The prepared data is byte-faithful
(5,058/5,058), and recipes, advancements, enchantments,
damage types and status effects all check out (the two
1.21.7/26.1.2 defects in those areas — special recipes and
cooking defaults — are fixed here). But the 1.21.10 tree
never carried the 26.2 loot fixes, never carried 1.21.7's
`supports_*` synthesis or motion-tag derivation, and the
result is three silent-form defects at once: both loot engines
dead on 1.21.10 data (B12-G01/B12-G02, 1,186 gap checks,
live-demonstrated), seven block types unable to survive on
valid ground (B12-G03, 11 gap checks), and a client that
cannot start past asset load (B12-G04, 3 gap checks).

**8,248 checks; 7,048 pass; 1,200 gap; 8 items (4 major gap,
2 minor gap, 2 fixed).**
