# Batch 12 — Data-driven registries (1.21.11)

## §0 Execution status

Checks + triage only, 2026-11-25. All checks in scope completed
fresh against 1.21.11's own jar/data and the 1.21.11 tree. No fixes,
no code changes, no commits, no git commands in the worktree.

**Prepare note:** Batch 0's prepare had NOT landed when this batch
started (the `b00/` scratch dir held only the oracle-measurement
files and no `versions/1.21.11/.local/horizon-client/` existed in
the worktree). This batch therefore ran the 1.21.10 Batch 12 recipe
itself: built `horizon-prepare` (release, `taskset -c 0`, `-j 1`,
batch-local target dir, 14m39s, exit 0 — the first build attempt
failed on `zlib-rs` with "Disk quota exceeded"; ~7 GB of
completed-run build output was freed first: the 1.21.10 sweep
worktree's `target/` (3.7 GB) and `.local/` (337 MB) plus the
1.21.10 Batch 0 `target/` (1.8 GB), all git-ignored regenerable
output of a finished sweep, no live agent using them) and ran it
(`./target/release/horizon-prepare --minecraft-dir
versions/1.21.11/.local/minecraft/1.21.11`, the lab-prepared
1.21.11 minecraft dir Batch 0 staged in the worktree). The run
completed cleanly in a single pass (1,373.9 s, exit 0,
`horizon-prepare.json` written). The byte-level diffs below are
against that FINAL prepared tree
(`versions/1.21.11/.local/horizon-client/1.21.11/`).

**`loot_fill` was built AND run for 1.21.11** (dev profile,
`taskset -c 0`, `-j 1`, 11m44s, exit 0; the run itself took
~1.5 s): 7 tables × seeds 1–5 against the real prepared data,
with the `minecraft:`-prefixed table ids the example requires
(bare names return empty — `fill` does no name normalisation,
`loot.rs:1163`; see the method notes). The first invocation
passed the tables as separate arguments and failed with
`ParseIntError` (the example takes one comma-separated table
argument); it was re-run correctly. The run demonstrates
B12-G02 live; see §4.

## §1 Scope

Recipes (1,470), loot tables (1,324), advancements (1,584,
triggers only), tags (625), enchantments (43), status effects
(40), damage types (50). Method 4 (PARITY-SWEEP.md §3):
byte-level diff of the port's prepared data against the vanilla
1.21.11 data files, plus behaviour checks of the loaders — where
the LOADER disagrees with the data's form even though the data
itself is faithful.

Vanilla spec: the named 1.21.11 game jars
(`e6-deltas/versions/1.21.11/jars/{client,server-game}-named.jar`,
sha1-verified against the brief's pins: client
`29bddf144a2f2907528b25d616770bfea67c4792`, server-game
`7f0d85c12aeb57f14687b7ab2ae2e5d825fbf0d2`; `server.jar` is a
bundler jar and was never javap'd), `data/minecraft/*` extracted
to the batch scratch dir (`jar-data/`, 7,556 files), `javap -c`
on the client jar's `MobEffects` class for effect
colours/categories/instant flags, on `RecipeSerializer.<clinit>`
for the cooking-time defaults, and on seven block classes for the
survival tag forms. Port data:
`versions/1.21.11/.local/horizon-client/1.21.11/data/minecraft/`
(this batch's prepare run, see §0).

**Headline:** the prepared data is faithful — every one of the
5,096 data-driven files is byte-identical to the 1.21.11 jar —
but the 1.21.11 tree reads 26.x forms in THREE places, exactly as
its first child 1.21.10 did: both loot engines read 26.3 key forms
against 1.21.11's old-form data (B12-G01/B12-G02, the 1.21.7
defects reproducing in full in the template itself), the survival
code reads eleven `supports_*` tag names that do not exist in
1.21.11 and resolves them all to empty sets, so seven block types
can never survive on valid ground (B12-G09, frogspawn, is the
item that stays in this batch — the other six blocks' reads are
Batch 5's scope in the merged ledger), and the render asset loader
hard-requires three 26.x-only tags (`blocks_motion_in_heightmap`,
`blocks_motion_in_heightmap_no_leaves`, `blocks_fluid_flow`)
that 1.21.11 never shipped, so the client cannot start (B12-G04).
The template carried neither the 26.2 loot fixes nor the 1.21.7
`supports_*` synthesis or motion-tag derivation. Two 1.21.7/26.1.2
defects ARE fixed here, as in 1.21.10: the special recipes
(hardcoded arms for all 12 bare-form specials) and the cooking-time
defaults (per-type 200/100/100/100, javap-verified). One 1.21.10
defect is WITHDRAWN by vanilla itself: `breath_of_the_nautilus`
arrived as a real vanilla effect in 1.21.11 (the nautilus update),
so the port's 40-entry MobEffects table matches the jar exactly
(B12-G06, closed). And `spear_mobs` — 26.x-only in 1.21.10 — is a
live trigger in 1.21.11 (`adventure/spear_many_mobs.json`), fired
from gameplay code.

NOTE: `sounds.json` is absent from the prepared tree (needs the
separate `horizon-prepare sounds` step against a launcher asset
store the VM lacks) — expected, not a port defect, and no check
in this batch needed it. `en_us.json` IS present
(`assets/minecraft/lang/en_us.json`) and md5-identical to the
client jar's file (`1460272285b8f2671c56c538113d1308`).

## §2 Method

- Byte-level `diff -r` of the extracted jar `data/minecraft/` vs
  the prepared `.local` copy, per family (recipes, loot tables,
  advancements, tags, enchantments, damage types) — plus a full
  `data/minecraft/` diff (0 differing files overall).
- Loot census in Python (`evidence/census.py`, adapted from the
  1.21.10 run): a walk that recurses into every dict and list,
  including `terms`/`term` combinator arrays (the 26.2
  coordinator note's correction is built in). Plural-`conditions`
  levels resolved by a level-aware walk that descends into inline
  tables nested in an entry's `value`; cross-checked by an
  independent pools-with-`rolls`-and-`conditions` signature count
  (955, including the 6 nested in `equipment/trial_chamber.json`).
- Recipe parser-arm emulation in Python
  (`evidence/recipe_emulation.py`, adapted from the 1.21.10 run):
  mirrors `from_sources` / `parse_recipe_sources` /
  `Ingredient::parse` / `Template::parse` field by field, with
  ingredient/tag resolution against the jar's item tags (flattened
  recursively, mirroring `load_tag_lists`) and the result codec
  range.
- Advancement trigger census over the 1,584 files + a
  firing-call-site grep for every distinct trigger across the
  1.21.11 crates (`evidence/trigger_coverage.txt`,
  `evidence/trigger_sites.txt`).
- Tag comparison: `block-tags.json` parsed and compared key by
  key against the jar's block tags (values are JSON-encoded
  strings of the whole tag object; compared parsed). Runtime
  tag-read audit: every `supports_*` / motion / fluid-flow tag
  name the tree reads, checked against the jar's tag files, with
  the missing ones traced to their readers and their vanilla
  1.21.11 forms verified by javap on the owning block classes.
- Enchantment loader read + effect-component wiring grep for
  all 30 keys (by key string, then by enchantment id for the 12
  keys wired that way); damage-type loader validation read +
  value census.
- `javap -c` on `MobEffects.<clinit>` (parsed programmatically
  for all 40 effects; the parser handles `ldc_w` past
  constant-pool index 255) + `javap` on all 14 distinct effect
  subclasses for the instant set + protocol-id order vs the
  1.21.11 prepared `registries.json` `minecraft:mob_effect`.
- `javap -c` on `RecipeSerializer.<clinit>` for the per-type
  cook-time defaults (the int each serializer passes to
  `AbstractCookingRecipe$Serializer.<init>`).
- `javap -c` on `SugarCaneBlock`, `CocoaBlock`,
  `MangrovePropaguleBlock`, `FrogspawnBlock`, `AzaleaBlock`,
  `NetherSproutsBlock`, `NetherWartBlock` for their vanilla
  1.21.11 survival forms.
- Live `loot_fill` run (7 tables × seeds 1–5) against the real
  prepared data — see §0.
- Batch 0's oracle table
  (`docs/sweep/1.21.11/batch-0-oracles.json`) had NOT landed when
  this batch finished, so no oracle table was consulted. None
  was needed: this batch is driver-independent (javap + data
  diffs + code reads + the prepare run + the `loot_fill` run).
  No oracle defects, no oracle repairs.

## §3 Checks table

| Registry | Data diff (byte-level) | Behaviour check | Result |
|---|---|---|---|
| Recipes (1,470) | 1,470/1,470 identical | Loadability per recipe (parser-arm emulation incl. tag resolution): 1,470 load, 0 dropped; 22/22 recipe types have a parser arm | pass |
| Loot tables (1,324) | 1,324/1,324 identical | items.rs × 1,324 tables: 134 pass / 1,190 gap; loot.rs live `loot_fill` run (tables already counted under items.rs, not double-counted): conditions/functions ignored throughout | **1,190 gap checks** (B12-G01, B12-G02) |
| Advancements (1,584) | 1,584/1,584 identical | 54 distinct triggers: 53 fired, `impossible` never fires by vanilla design | pass |
| Tags (625) | 625/625 identical | `block-tags.json` 204/204 exact-equal; 11 `supports_*` runtime tag reads resolve empty and 3 motion/fluid-flow tags are hard-required at startup and absent — verified as re-check evidence for B12-G09/B12-G04 (see the counting note below) | pass (204 checks) + 2 major items at 0 gap checks |
| Enchantments (43) | 43/43 identical | Loader reads all 1.21.11 fields; all 30 effect component keys wired (data-driven or by enchantment id) | pass |
| Status effects (40) | n/a — code-defined in 1.21.11, no `mob_effect/` data files in the jar | 40/40 colours, categories, instant flags, protocol order match the jar; `breath_of_the_nautilus` is vanilla in 1.21.11 (B12-G06 withdrawn, 0 gap checks) | pass (40 checks) |
| Damage types (50) | 50/50 identical | Loader validates all fields; every value in the files is accepted | pass |

Checks total: **8,290** (5,096 data-diff entries + 3,194
behaviour checks). Pass: **7,100**. Gap: **1,190** — collapsing
to **5 gap defects** (4 major: B12-G01, B12-G02, B12-G04,
B12-G09; 1 minor: B12-G05), plus 1 closed (B12-G06) and 2
fixed (B12-G07, B12-G08). B12-G04 and B12-G09 are recorded at
0 gap checks each (see the counting note below); all 1,190 gap
checks come from B12-G01.
Behaviour subtotal reconciles as 1,470 recipe loads + 22
recipe-type arms + 1,324 items.rs tables + 54 triggers + 204
block-tags + 30 enchantment keys + 40 effects + 50 damage types
= 3,194; the loot.rs live run covers tables already counted under
items.rs and is not double-counted.

Behaviour/non-behaviour split: as-tallied 8,290 (8,290
behaviour + 0 non-behaviour). Batch 12 is not one of the three
batches that count non-behaviour checks as-tallied.

Counting rules: a table counts as a gap check for items.rs when
it carries ≥1 condition (the engine evaluates every object
condition as false, so conditioned pools/entries are dropped)
or ≥1 function (the engine never reads the `functions` arrays,
so no function runs). Tables with neither (134) roll correctly
through the engine and pass. The loot.rs defect is recorded as
its own item with 0 additional gap checks, per the settled
no-double-count rule.

Counting note (deviation from the 1.21.10 run, per this run's
brief): the 1.21.10 run counted the 11 `supports_*` runtime tag
reads and the 3 motion-tag reads as 14 gap checks. This run's
brief gives an explicit reconciliation formula for the behaviour
subtotal (recipe loads + recipe-type arms + items.rs tables +
triggers + block-tags + enchantment keys + effects + damage
types) that does NOT include tag reads, and the merged 1.21.10
ledger folded the `supports_*` group item out of Batch 12 (only
frogspawn, B12-G09, stays in B12; the rest is Batch 5's scope)
while the motion-tag item overlaps Batch 3's startup-blocker
item. The 11 `supports_*` reads and 3 motion-tag reads were
therefore verified fresh as re-check evidence for B12-G09 and
B12-G04 (both recorded at 0 gap checks, like B12-G05/B12-G06 in
the 1.21.10 run) and are NOT counted as separate checks here,
to avoid double-counting across batches. If the coordinator
prefers the 1.21.10 counting, add 14 gap checks (total 8,304;
gap 1,204).

No oracle defects, no oracle repairs: this batch is
driver-independent (javap + data diffs + code reads + the
prepare run + the `loot_fill` run). No fixes made.

## §4 Defects

### B12-G01 — MAJOR: items.rs evaluates every loot condition as false and never runs any loot function; survival block-breaking drops nothing from conditioned tables

`horizon-items/src/items.rs`, two key-form defects in `LootContext`
(line numbers verified fresh in the 1.21.11 tree — identical to
the 1.21.10 tree's):

1. **Conditions.** `condition()` (line 1983) reads the kind ONLY from
   `o.get("type")` (line 1988) — there is no
   `.or_else(|| o.get("condition"))` dual-form read (contrast the
   26.2 tree, `items.rs:2090`, per the 1.21.10 report). 1.21.11's
   data uses the `condition` key throughout (1,542 occurrences, 0
   `type`-keyed), so every object-form condition's kind is `""`
   and the match falls to `_ => false`. `conditions()` (line 2109)
   DOES consult the plural `conditions` array — every element
   fails. Nested conditions fail the same way (`inverted`'s `term`,
   `any_of`/`all_of`'s `terms`).
2. **Functions.** `evaluate()` applies `pool["modifier"]` and
   `table["modifier"]` (lines 1870-1871), `expand()` pushes
   `e["modifier"]` (line 2143, applied by `choose()`), `nested()`
   applies `entry["modifier"]` (line 2284) — but 1.21.11's data has
   ZERO `modifier` keys anywhere and puts functions in `functions`
   arrays (983: 964 entry-level, 10 pool-level, 9 table-level).
   `modifiers()` (line 2298) itself HAS the function-discriminator
   dual-form fix (`f["type"].as_str().or(f["function"].as_str())`,
   line 2308) — dead code, because the container key is never read.
   No function ever runs through this engine on 1.21.11 data.

Impact, counted over all 1,324 tables: **1,190 tables** carry ≥1
condition or ≥1 function — blocks 1,050/1,083 (1,021 with a
condition, 225 with a function), entities 62/108, chests 54/56,
gameplay 6/26, dispensers 3/3, equipment 3/3, spawners 3/5,
shearing 3/22, harvest 2/3, archaeology 1/6, carve 1/1,
charged_creeper 1/6, pots 1/1, brush 0/1. Only 134 tables (neither
conditions nor functions) roll correctly.

Survival impact is direct: `Game::block_loot_at`
(`game.rs:3683`) → `block_drops_by_player` (`items.rs:1559`) →
`block_drops_raw` (`items.rs:1620`) → `ctx.evaluate(table, random)`
(`items.rs:1657/1671`). Every pool with a condition is skipped in
`evaluate()` and every entry with a condition is rejected in
`expand()`, so a block whose table is fully conditioned drops
nothing — the same symptom the owner confirmed in-game for 1.21.7
("survival block-breaking drops nothing"). 1,021 of 1,083 block
tables carry ≥1 condition. `/loot` routes every table type through
this engine too (`loot_table_for_command`, `items.rs:1685`;
`entity_drops_for_command` / `entity_drops_with_target`,
`items.rs:1790/1802`; `block_drops_by_player`), so command rolls
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
`set_name`, `set_components`; default `_ => {}`, line ~2510) and
is missing 7 of the 19 function kinds 1.21.11's data uses:
`copy_components` (71 tables, all blocks), `furnace_smelt` (16,
entities), `set_enchantments` (4: 3 equipment, 1 chest),
`set_stew_effect` (3: 1 archaeology, 2 chests),
`exploration_map` (3, chests),
`set_ominous_bottle_amplifier` (3: 2 chests, 1 entity),
`set_instrument` (1, chest) — union 101 tables (verified
disjoint fresh from the jar data). `expand()` (line 2124) has no
`minecraft:tag` arm (default `_ => false`, line 2219), losing
the single tag entry (`entities/creeper.json`, the music-disc
entry). Union + tag = **102 tables** — exactly the 26.2 run's
B12-G01 count and the 1.21.10 run's, the same defect waiting
underneath. The 26.2 tree reads `["functions"]` + dual-form
discriminators (per the 1.21.10 report: `items.rs:1969-1970/2090/
2246/2387`, `loot.rs:1042/2126/2164`) — the 26.2 playtest fixes
were never carried to the 1.21.11 template.

### B12-G02 — MAJOR: the loot.rs engine ignores all loot conditions and all loot functions

`horizon-world/src/loot.rs` is written for the 26.3 loot format
throughout, and 1.21.11 writes the old form at every level (line
numbers verified fresh — identical to the 1.21.10 tree's):

- `condition()` (line 1368) reads the kind from `c["type"]` only
  (line 1377) — 1.21.11's 1,542 conditions all use the `condition`
  key.
- `modify()` (line 1478) consults singular `m["condition"]` only
  (line 1488) — 1.21.11's 184 function-level condition lists are
  all plural `conditions` arrays.
- `run()` (line 1512) reads the kind from `f["type"]` only (line
  1514) — 1.21.11's 1,404 functions all use the `function` key;
  the default arm `_ => {}` (line 1937) no-ops anything it does
  see.
- `table_raw()` (line 2021) checks singular `pool["condition"]`
  (line 2028) and applies `pool["modifier"]` / `table["modifier"]`
  (lines 2039, 2026) — 1.21.11 has 955 plural pool `conditions`
  arrays (including 6 nested inside an entry's `value`, in
  `equipment/trial_chamber.json`) and 0 `modifier` keys.
- `expand()` (line 2080) checks singular `entry["condition"]`
  (line 2081) — 1.21.11's 247 entry-level condition lists are all
  plural, so every entry passes its gate.
- `create()` (line 2131) applies `entry["modifier"]` (line 2133)
  — 0 in the data, so no entry function ever runs.
- The `minecraft:tag` arm reads `entry["items"]` (lines 2110,
  2146) — 1.21.11's tag entry uses the `name` field
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
the arms are unreachable on 1.21.11 data because of the key
forms.

Precision note: the file DOES contain one `f["functions"]` read,
at line 1516 — inside the `minecraft:sequence` arm of `run()`,
which iterates a sequence function's nested functions. It
changes nothing: the arm is reached only when `f["type"] ==
"minecraft:sequence"` (line 1514), and 1.21.11's data uses the
`function` key — and `minecraft:sequence` is not among the 19
function kinds in the data at all. The only other `functions`
occurrence in the file (line 2232) is test JSON.

**Demonstrated live** with the existing `loot_fill` example
against the real prepared data (7 tables × seeds 1–5,
`evidence/loot_fill.txt`, 153 lines):

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
  as plain `minecraft:book`, never `enchanted_book`).

Affects entity drops, chest/container loot, fishing, trial
spawners and vaults in gameplay — everything `loot.rs` serves.
0 additional gap checks: these tables are already counted under
items.rs (B12-G01).

### B12-G04 — MAJOR: the asset loader hard-requires three 26.x-only tags that 1.21.11 never shipped; the client cannot start

`horizon-client-render/src/assets.rs`, `Assets::load` (line 186)
reads `block-tags.json` and REQUIRES three tags, erroring if any
is absent:

- `minecraft:blocks_motion_in_heightmap` (:212-216,
  `.ok_or_else(|| invalid("missing motion tag"))?`)
- `minecraft:blocks_motion_in_heightmap_no_leaves` (:217-221,
  `"missing no-leaves motion tag"`)
- `minecraft:blocks_fluid_flow` (:224-229,
  `"missing fluid flow tag"`)

None of the three exists in 1.21.11: the jar's 204 block tags
contain no `blocks_motion*` or `blocks_fluid_flow` file, and
the prepared `block-tags.json` (204/204 exact-equal to the jar)
contains none either. No synthesis or derivation exists
anywhere in the 1.21.11 tree — `TagDefinitions::build`
(`worldgen/tags.rs:149`) does not synthesise them, and no
`horizon-prepare` step writes them. The load therefore fails
with "missing motion tag" before the game can start — the
exact crash the 1.21.7 port hit on its first laptop launch
(Nov 21), fixed there by deriving the sets from block data
instead of reading tags (`versions/1.21.7/.../assets.rs:212-283`).
That fix was never carried to the 1.21.11 template. 0 gap checks
in this run's tally (see the counting note in §3); Batch 3's
B3-G22-equivalent may be the same defect from the startup side —
flagging for the coordinator to fold, not to double-count.

### B12-G05 — minor: object-form recipe ingredients are rejected (datapack scope only)

`crafting.rs`, `Ingredient::parse_object` (line 123): a
JSON-object ingredient requires a `neoforge:ingredient_type` or
`type` key, and returns `None` otherwise. Vanilla's own object
forms (`{"item": "..."}`, `{"tag": "..."}`) have neither key, so
any recipe using them fails to parse (shaped: whole recipe
dropped; shapeless/cooking: dropped). No vanilla 1.21.11 recipe
uses object ingredients (5,076 string slots + 115 array slots
whose 1,151 elements are all strings, counted as occurrences —
shaped keys counted per pattern cell; 0 object slots), so this
affects only player datapacks / mod recipes that use the object
form. Same code, same data property as 1.21.7's B12-G05 and
1.21.10's B12-G05 — carried forward, not a vanilla parity
defect. 0 gap checks.

### B12-G09 — MAJOR: frogspawn can never survive: port reads the absent #supports_frogspawn tag

The 1.21.11 jar ships ZERO `supports_*` block tags and only two
fluid tags (`lava`, `water`). The 1.21.7 tree answered this by
synthesising the tags (`synthesize_legacy_support_tags`, a
declared override in the 1.21.7 sweep). The 1.21.11 template has
NO synthesis code anywhere (grep-verified across the tree), and
the port adapted only SOME survival checks to 1.21.11-native
forms. The frogspawn reads still point at the 26.x name and
resolve to EMPTY sets, in both survival implementations:

- `block_survival.rs`: `frogspawn: tag_states(
  "minecraft:supports_frogspawn")?` (:609, block) and
  `fluid_tag("minecraft:supports_frogspawn")?` (:619, fluid) —
  missing keys yield empty sets silently (`tags.get(name)
  .into_iter().flatten()`, lines 482-497).
- `level/survive.rs` (:477-478): `b.fluid_in(fluid.kind(),
  b.fluid_tags.supports_frogspawn) || b.in_set(self.state(pos),
  &b.tags.supports_frogspawn)` — the level `Tags` struct's
  `resolve()` defaults missing tags to empty (`level/data.rs:
  398-400`, `.unwrap_or_default()`).

Vanilla 1.21.11's form, verified by `javap -c` on the 1.21.11
client-named jar (`FrogspawnBlock.mayPlaceOn`): the fluid below
must be `Fluids.WATER` and the fluid above `Fluids.EMPTY` — no
tag is involved. With both sets empty, frogspawn can never
survive on water in the port. 0 gap checks in this run's tally
(see the counting note in §3).

The other ten `supports_*` reads were verified fresh too and
reproduce the same defect (sugar cane :611-612/:620, cocoa :593,
mangrove propagule :600-601, azalea via `survival.rs:76`, nether
sprouts via `survival.rs:85`, nether wart via `survival.rs:87` —
each block's vanilla 1.21.11 survival form javap-verified, none
using a `supports_*` tag; see the evidence index). They are
Batch 5's scope in the merged ledger (the 1.21.10 run's group
item B12-G03 was folded out of Batch 12), so they are recorded
here as evidence only, not minted as a B12 item.

## §5 Re-check verdicts

### 1.21.10 items (PRIMARY — all decided)

- **B12-G01 (1.21.10, MAJOR)** — items.rs conditions always false:
  **REPRODUCES** as this batch's B12-G01 above (same condition
  key-form defect, plus the `functions`-container defect — the
  26.2 tree's fix was never carried to the template; 1,190-table
  impact in 1.21.11 vs 1,186 in 1.21.10 — the difference is real
  data difference, 1,324 vs 1,320 tables).
- **B12-G02 (1.21.10, MAJOR)** — loot.rs ignores all conditions
  and functions: **REPRODUCES** as this batch's B12-G02 above,
  demonstrated live with the `loot_fill` run (see §4).
- **B12-G04 (1.21.10, MAJOR)** — asset loader hard-requires three
  26.x-only tags: **REPRODUCES** as this batch's B12-G04 above
  (same three hard-required reads at the same lines; no
  derivation anywhere in the template).
- **B12-G05 (1.21.10, minor)** — object-form ingredients
  rejected: **REPRODUCES** as this batch's B12-G05 above (same
  code at `crafting.rs:123`, same 0-object-slots data property;
  datapack scope only).
- **B12-G06 (1.21.10, minor)** — extra 26.x effect in the table:
  **WITHDRAWN** — vanilla 1.21.11 ships `breath_of_the_nautilus`
  as a real effect (40 effects in the jar's `MobEffects`,
  `BREATH_OF_THE_NAUTILUS` at index 39, colour 65518,
  Beneficial; the nautilus arrived in 1.21.11). The port's
  40-entry table matches the jar exactly (0 mismatches on order,
  colour, category, instant set). Recorded as slice item B12-G06
  (status closed).
- **B12-G07 (1.21.10, major/fixed)** — 11 of 12 special recipes
  never load: **FIXED** in the 1.21.11 tree too. `from_sources`
  (`crafting.rs:1557`) has hardcoded arms for all 12 bare-form
  specials (comment at :1589-1594: "1.21.11's specials are
  hardcoded: their JSONs carry only `category` (and `group`)"),
  including the three 1.21.7-era type names the 26.x parser
  lacked (`crafting_special_armordye` :1699,
  `crafting_special_tippedarrow` :1706,
  `crafting_special_mapcloning` :1764) and
  `crafting_decorated_pot` (:1754); the tag-dependent arms
  resolve against the jar's item tags (`#banners` 16,
  `#dyeable` 6, `#book_cloning_target` 1, `#skulls` 7,
  `#decorated_pot_ingredients` 24). The parser-arm emulation
  loads 12/12. Recorded as slice item B12-G07 (status fixed).
- **B12-G08 (1.21.10, minor/fixed)** — flat 200 cooking-time
  default: **FIXED** in the 1.21.11 tree too. `crafting.rs:1772-1779`
  uses per-type defaults (smelting 200, blasting/smoking/campfire
  100), matching vanilla 1.21.11 exactly as verified by `javap
  -c` on `RecipeSerializer.<clinit>` (the int each serializer
  passes to `AbstractCookingRecipe$Serializer.<init>`: smelting
  `sipush 200`, the other three `bipush 100`). All 116 vanilla
  cooking recipes carry explicit `cookingtime` anyway. Recorded
  as slice item B12-G08 (status fixed).
- **B12-G09 (1.21.10, MAJOR)** — frogspawn can never survive:
  **REPRODUCES** as this batch's B12-G09 above (same empty-set
  reads at the same lines; vanilla 1.21.11's water/empty-fluid
  form javap-verified).

### 1.21.7 items (SECONDARY — all decided)

- **B12-G01 (1.21.7, MAJOR)** — items.rs conditions always false:
  **REPRODUCES** as this batch's B12-G01 above (same condition
  key-form defect, plus the `functions`-container defect).
- **B12-G02 (1.21.7, MAJOR)** — loot.rs ignores all conditions
  and functions: **REPRODUCES** as this batch's B12-G02 above,
  demonstrated live with the `loot_fill` run (see §4).
- **B12-G03 (1.21.7, MAJOR)** — 11 of 12 special recipes never
  load: **FIXED** in the 1.21.11 tree (see 1.21.10 B12-G07
  above). Recorded as slice item B12-G07 (status fixed).
- **B12-G04 (1.21.7, minor)** — extra 26.x effect in the table:
  **WITHDRAWN** — vanilla 1.21.11 ships the effect (see 1.21.10
  B12-G06 above). Recorded as slice item B12-G06 (status
  closed).
- **B12-G05 (1.21.7, minor)** — object-form ingredients
  rejected: **REPRODUCES** as this batch's B12-G05 above (same
  code at `crafting.rs:123`, same 0-object-slots data property;
  datapack scope only).

## §6 Systemic findings

The prepared data is faithful — all 5,096 data-driven files
byte-identical to the 1.21.11 jar, and `block-tags.json`
byte-faithful too. Every defect is code reading the 26.x form
of a format 1.21.11 writes differently and failing silently —
the same bug class as the 1.21.7 pilot's 26.x-tag findings
(G1/G3/G5/G6), now in three places at once, and this run proves
the defects live in the TEMPLATE itself, not just in the child:
1.21.10 inherited them from 1.21.11, line numbers identical.

1. **Loot keys.** 1.21.11 writes the old forms (the `condition`
   discriminator ×1,542, the `function` discriminator ×1,404,
   plural `conditions` arrays ×1,386, `functions` arrays ×983,
   the tag entry's `name` field) while both engines read the
   26.3 forms (`type`, `modifier`, singular `condition`, tag
   `items`). The `type`/`modifier` renames are 26.3 changes,
   and the 26.2 playtest fixes that taught the engines both
   forms were never carried to the template.
2. **Survival tags.** 1.21.11 has no `supports_*` tags at all;
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
   never carried, so the 1.21.11 client fails at the same
   startup point 1.21.7 did before its fix.

The port DOES get the 1.21.11 forms right where it was written
for them: the specials are hardcoded to the bare form, the
cooking defaults are per-type, the transmute arm pins the
1.21.11 material bounds (no `material_count` exists in the
data), the spear trigger is wired live (spears are new in
1.21.11), and the recipe/advancement/enchantment/damage loaders
all read 1.21.11's forms correctly. The defects are the places
where 26.3-base code survived the port untouched. Vanilla also
moved under the template in one place this run caught: the
nautilus effect (B12-G06) is genuinely new in 1.21.11, so a
defect in 1.21.10 is correct behaviour here.

## Passes, with the evidence

- **Recipes.** All 1,470 load under a parser-arm emulation
  (type arms + required fields + ingredient/tag resolution
  against the jar's item tags + result codec range): shaped
  706, shapeless 320, stonecutting 254 (via `stations.rs`
  `parse_recipe_sources`, line 805), smelting 73, blasting 25,
  smoking 9, campfire_cooking 9, transmute 32 (bundle/shulker
  dyeing only — map cloning is a special here, not a
  transmute), smithing_trim 18, smithing_transform 12, and 12
  bare-form specials (armour dyeing, banner duplicate, book
  cloning, firework rocket, firework star, firework star fade,
  map cloning, map extending, repair item, shield decoration,
  tipped arrow, decorated pot). The 1.21.7 defect does not
  reproduce: the 1.21.11 parser has hardcoded arms for all 12
  specials and every tag they need resolves. `custom_type`
  (`crafting.rs:1348`) diverts only non-`minecraft:` types, so
  no vanilla recipe is diverted. All shaped patterns'
  characters exist in their `key` dicts (0 bad). Ingredient
  slots: 5,076 string slots + 115 array slots whose 1,151
  elements are all strings, counted as occurrences (shaped keys
  counted per pattern cell); 0 object slots (see B12-G05).
  Result forms: `{count, id}` ×1,264, `{id}` ×160,
  `{components, count, id}` ×16, all handled by
  `Template::parse` / `recipe_result_id`; max count 16 (inside
  the 1..=99 codec range). No transmute file carries
  `material_count`, so the arm's hardcoded 1..1 bounds match
  the data. All 116 cooking recipes carry explicit
  `cookingtime` (see §5, 1.21.10 B12-G08).
- **Advancements (triggers only).** 54 distinct triggers used
  across the 1,584 files — including `spear_mobs`, which IS
  present in 1.21.11 (`adventure/spear_many_mobs.json`, iron
  spear icon; spears arrived in 1.21.11) and IS fired
  (`trigger_spear_mobs`, `game/advancements.rs:2012-2014`,
  called from `spear.rs:225`). The inventory's 58 counts
  trigger TYPES in the registry, of which 54 are used. 53 are
  fired from gameplay code: direct `self.trigger` calls
  (`tick` `game/advancements.rs:1253`, `location` `:1287`,
  `bee_nest_destroyed` `:2038`, `fall_after_explosion`
  `:1466`, `using_item` `:1556`, `slide_down_block` `:1876`,
  `crafter_recipe_crafted` `machines/crafter.rs:199`, …), the
  generic `trigger_block` / `trigger_item` / `trigger_player`
  / `trigger_entity` / `trigger_distance` call sites
  (`placed_block` `game.rs:8323`, `filled_bucket`
  `game.rs:8235`, `consume_item` `consume.rs:53`, `slept_in_bed`
  `game.rs:8861`, `shot_crossbow` `crossbow.rs:401`,
  `summoned_entity` `dragon.rs:4523`, `hero_of_the_village`
  `raids.rs:683`, `avoid_vibration` `sculk.rs:877/958`,
  `allay_drop_item_on_block` `allays.rs:309`, …),
  `trigger_killed` / `trigger_killed_as`
  (`player_killed_entity`, `kill_mob_near_sculk_catalyst`
  `creatures.rs:1731/1861`), `trigger_item_entity`
  (`player_interacted_with_entity`,
  `player_sheared_equipment` `creatures.rs:7144`,
  `thrown_item_picked_up_by_entity` `:1744`,
  `thrown_item_picked_up_by_player` `:1754`),
  `trigger_item_durability` (`item_durability_changed`),
  `trigger_player_hurt_entity` /
  `trigger_entity_hurt_player`, `trigger_bred` (`bred_animals`,
  `creatures.rs:3859`), `trigger_dimension`
  (`changed_dimension`, `server_travel.rs:600/1015`),
  `trigger_loot_table` (`player_generates_container_loot`,
  `brush.rs:202`), `check_inventory_trigger`
  (`inventory_changed`, `:1144`) and `recipe_unlocked`
  (`:938`). Every trigger was confirmed present in firing code
  by a per-trigger grep across the crates
  (`evidence/trigger_coverage.txt`: 0 missing;
  `evidence/trigger_sites.txt` classifies the sites). The one
  unfired trigger, `minecraft:impossible`, is used only by
  `recipes/root.json` and never fires in vanilla either — pass
  by design. `Tree::by_trigger` indexes triggers from the data
  files themselves (`advancements.rs:62`), so no trigger is
  dropped at load. Item-predicate evaluation reads the 1.21.11
  form (`items` / `count` / `components` / `predicates`,
  `advancements.rs:835-840`). Observation (not a numbered
  defect): `entity_killed_player`'s only firing call site
  found is the LAN guest death path
  (`game/lan_actions.rs:4666`, `guest_killed_player_trigger`);
  no single-player local-death firing call site was found —
  same observation as the 26.2, 26.1.1, 26.1.2 and 1.21.10
  runs. The only advancement using it, `adventure/root.json`,
  grants via the OR'd `player_killed_entity` criterion too, so
  nothing is unreachable; the single-player firing path belongs
  to the batch that owns player-death semantics. (Criterion
  *condition* semantics beyond trigger firing are out of this
  batch's triggers-only scope.)
- **Tags.** All 625 files byte-identical, across 14 kinds
  (block 204, item 198, worldgen 91, entity_type 46,
  damage_type 33, enchantment 22, banner_pattern 11, game_event
  5, timeline 4, instrument 3, point_of_interest_type 3, fluid
  2, dialog 2, painting_variant 1). `block-tags.json` (204 keys)
  is exactly faithful to the jar's block tags — 204/204
  exact-equal (values are JSON-encoded strings of the whole tag
  object; compared parsed), no extras, no missing keys. Runtime
  loaders read the same files (`horizon-client-render/src/
  assets.rs:192` reads `block-tags.json`; `load_tag_lists`,
  `items.rs:566`, is generic over kind and flattens nested
  `#tag` references recursively). The 1.21.7 declared override
  does NOT apply here in the synthesis direction: 1.21.11's jar
  ships ZERO `supports_*` tags and NO
  `synthesize_legacy_support_tags` code exists in the 1.21.11
  tree — but the tree still reads the names, which is B12-G09
  (and Batch 5's scope for the other six blocks), and the asset
  loader still demands the motion tags, which is B12-G04. The
  jar DOES ship the 1.21.11-native forms the port adapted to
  elsewhere: `snow_layer_can_survive_on` /
  `snow_layer_cannot_survive_on`, `azalea_grows_on`,
  `dry_vegetation_may_place_on`, `bamboo_plantable_on`,
  `big_dripleaf_placeable`, `small_dripleaf_placeable`,
  `convertable_to_mud` (Mojang's spelling — the level `Tags`
  struct's `convertible_to_mud` field names a tag that does not
  exist, but it has no readers).
- **Enchantments.** Loader (`horizon-items/src/enchant.rs:146-181`)
  reads exactly the 1.21.11 fields: `supported_items`,
  `primary_items` (5 files), `exclusive_set` (18), `max_level`,
  `weight`, `anvil_cost`, `min_cost`/`max_cost` (`base` /
  `per_level_above_first`), `slots`, `effects`. All 30 effect
  component keys used by the 43 files are evaluated somewhere:
  18 via the data-driven `modify_value` path (`enchant.rs:700`)
  and data-driven readers in `enchant.rs` / `loot.rs`
  (`damage`, `damage_protection` via `game/potions.rs:1221`,
  `item_damage`, `knockback`, `ammo_use`,
  `armor_effectiveness`, `projectile_piercing`,
  `repair_with_xp`, `smash_damage_per_fallen_block`,
  `attributes` `enchant.rs:747/770`, `post_attack`,
  `post_piercing_attack` `game/enchant_effects.rs:316`,
  `projectile_spawned`, `damage_immunity`, `location_changed`,
  `equipment_drops` `loot.rs:1298`,
  `prevent_equipment_drop` `loot.rs:959`), 12 by enchantment
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
  Soul Speed particles, `game/enchant_effects.rs:1032`
  `tick_soul_speed`; `trident_return_acceleration`: Loyalty,
  `trident.rs:274-299`; `trident_sound` +
  `trident_spin_attack_strength`: Riptide,
  `trident.rs:396,499-520`). 1.21.11 has 30 effect keys vs
  1.21.10's 29 — `post_piercing_attack` occurs in 1.21.11's
  data (1 file) and is wired data-driven. `fortune.json` has
  no `effects` — correct, fortune is implemented via loot
  `apply_bonus`. Effect *values* (damage numbers, durations)
  are behaviour depth for other batches, not checked here.
- **Status effects.** 40/40 colours, categories, instant flags
  and protocol order match the jar (`javap -c` on
  `MobEffects.<clinit>`, parsed programmatically). The instant
  set is exactly `instant_health`, `instant_damage`,
  `saturation` (`HealOrHarmMobEffect` / `SaturationMobEffect`
  extend `InstantenousMobEffect` — Mojang's spelling —
  verified by javap; the other 12 effect subclasses extend
  `MobEffect` directly). Protocol ids 0–39 in the 1.21.11
  prepared `registries.json` `minecraft:mob_effect` match the
  port table's order exactly. B12-G06 is WITHDRAWN: the
  nautilus effect is vanilla in 1.21.11.
- **Damage types.** Loader
  (`horizon-world/src/damage_source/resources.rs:127-173`,
  `from_json`) parses `message_id` (required), `exhaustion`
  (required, parsed directly to f32), `scaling` (validated:
  `never` / `when_caused_by_living_non_player` / `always`),
  `effects` (default `hurt`, validated against hurt/thorns/drowning/
  burning/poking/freezing), and `death_message_type` (default
  `default`, validated against default/fall_variants/
  intentional_game_design). Every value in the 50 files is
  inside the accepted lists (scaling: `always` ×4 /
  `when_caused_by_living_non_player` ×46; effects: burning ×7,
  drowning, freezing, poking, thorns, default ×39;
  death_message_type: `fall_variants` ×2,
  `intentional_game_design` ×1, default ×47).

## Declared overrides

None. The 1.21.7 batch's declared override (`supports_*` tag
synthesis) does not exist in the 1.21.11 tree and is not
applied: 1.21.11 ships no `supports_*` tags natively, no
synthesis code exists, and the reads resolve empty instead —
which is B12-G09 (and Batch 5's scope for the other six blocks),
a defect, not an override.

## §7 Evidence index

All paths under
`~/workspace/mcir-work/sweep-1.21.11-batches/b12/evidence/`
unless noted. Jar data extracted to `b12/jar-data/`
(`data/minecraft/*` from the 1.21.11 client jar, 7,556 files).

- `data_diff.txt` — byte-level diff of the extracted jar
  `data/minecraft/` vs this batch's prepared tree: 0 differing
  files overall; per-family counts: recipes 1,470, loot 1,324,
  advancements 1,584, tags 625, enchantments 43, damage types
  50 (5,096 data-diff entries).
- `horizon-prepare-build.log` — release build of
  `horizon-prepare`, exit 0, 14m39s (after a first attempt
  failed on disk quota; see §0).
  `horizon-prepare.log` — prepare run against the
  lab-prepared 1.21.11 minecraft dir, exit 0, 1,373.9 s.
- `census.py` / `census.json` / `census.txt` — recursive loot
  census (recurse into `terms`/`term` arrays; level-aware
  plural-`conditions` attribution; pool-signature cross-check):
  1,542 conditions (all `condition`-keyed, 0 `type`-keyed, 12
  kinds), 1,404 functions (all `function`-keyed, 19 kinds),
  0 `modifier` keys, 0 singular condition objects, `functions`
  containers 983 (964 entry / 10 pool / 9 table), plural
  `conditions` 1,386 (pool 955 incl. 6 nested in an entry's
  `value` in `equipment/trial_chamber.json`, entry 247,
  function 184); per-family "either" counts; the single tag
  entry (`entities/creeper.json`,
  `{"type":"minecraft:tag","expand":true,"name":"minecraft:creeper_drop_music_discs"}`);
  the 101-table post-fix residual by missing kind, disjoint
  by construction.
- `loot_fill-build.log` — dev-profile build of the
  `loot_fill` example, exit 0, 11m44s.
  `loot_fill.txt` — the live run, 7 tables × seeds 1–5, 153
  lines: every stack count 1; the gated lava-chicken disc on
  all 5 zombie seeds; `blocks/stone` → `minecraft:stone`,
  `blocks/diamond_ore` → `minecraft:diamond_ore` on all 5
  seeds; `entities/creeper` → gunpowder ×1 only; no
  enchantments anywhere.
- `recipe_emulation.py` / `recipe_emulation.txt` — parser-arm
  emulation: 1,470/1,470 load, 0 dropped; 22 types with arms;
  12/12 specials; ingredient-slot census (5,076 string + 115
  array slots, 1,151 string elements, 0 object slots);
  result-form census; `material_count` absence; explicit
  `cookingtime` on all 116 cooking recipes.
- `javap_RecipeSerializer.txt` — `RecipeSerializer.<clinit>`:
  smelting `sipush 200`, blasting/smoking/campfire
  `bipush 100`.
- `javap_SugarCaneBlock.txt`, `javap_CocoaBlock.txt`,
  `javap_MangrovePropaguleBlock.txt`, `javap_FrogspawnBlock.txt`,
  `javap_AzaleaBlock.txt`, `javap_NetherSproutsBlock.txt`,
  `javap_NetherWartBlock.txt` — the seven survival forms
  (dirt/sand + water; jungle logs; clay/water + mangrove
  leaves; water below / empty above; clay or vegetation;
  nylium / soul soil / vegetation; soul sand only).
- `triggers.txt` — the 54 distinct triggers used across the
  1,584 advancement files (incl. `spear_mobs`, 1 criterion).
  `trigger_coverage.txt` — per-trigger firing-call-site grep
  result (0 missing). `trigger_sites.txt` — the firing sites
  classified by mechanism, incl. the `entity_killed_player`
  observation and the live `trigger_spear_mobs`.
- `tags_block.txt` — `block-tags.json` vs the jar's block
  tags: 204/204 exact-equal.
- `mobeffects.txt` / `effects_parse.py` / `effects_jar.json` /
  `effects_jar.txt` — the 40 effects parsed from
  `MobEffects.<clinit>` (name, colour, category) vs the
  port's 40-entry table: 0 mismatches; `breath_of_the_nautilus`
  is vanilla effect 39 in 1.21.11.
  `effect_superclasses.txt` — javap on all 14 effect
  subclasses: instant exactly `instant_health`,
  `instant_damage`, `saturation`.

Method notes and limitations:

- The `loot_fill` example takes full `minecraft:` table ids;
  bare names return empty (no normalisation in `fill`,
  `loot.rs:1163`) — example usability, not a defect.
- The prepared tree used for the byte diffs was produced by
  THIS batch's prepare run (§0); no candidate-test prepared
  tree exists for 1.21.11, so no cross-check tree was
  available.
- The recipe emulation assumes item defaults exist for all 12
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
  batches rather than minted here (their vanilla 1.21.11
  forms were NOT javap-verified in this batch): the level
  `Tags` struct's `cannot_support_seagrass`
  (`survive.rs:442`), `cannot_support_snow_layer` +
  `support_override_snow_layer` (`survive.rs:314/316`,
  `weather.rs:113/116` — the jar natively ships
  `snow_layer_can_survive_on` /
  `snow_layer_cannot_survive_on`), `support_override_cactus_flower`
  (`survive.rs:406`), `washed_away_by_fluids`
  (`fluids.rs:360`), and `blocks_motion` (`level/hooks.rs:549`,
  the ice-melt check — vanilla 1.21.11 uses motion-blocking
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
(5,096/5,096), and recipes, advancements, enchantments,
damage types and status effects all check out (the two
1.21.7/26.1.2 defects in those areas — special recipes and
cooking defaults — are fixed here, and the nautilus "extra
effect" is withdrawn by vanilla itself). But the 1.21.11
template never carried the 26.2 loot fixes, never carried
1.21.7's `supports_*` synthesis or motion-tag derivation, and
the result is three silent-form defects at once: both loot
engines dead on 1.21.11 data (B12-G01/B12-G02, 1,190 gap checks,
live-demonstrated), frogspawn unable to survive on water with
six more blocks in the same state (B12-G09; the other reads
are Batch 5's scope), and a client that cannot start past asset
load (B12-G04). This run proves the defects live in the
template itself: 1.21.10 inherited them line-for-line.

**8,290 checks; 7,100 pass; 1,190 gap; 8 items (4 major gap,
1 minor gap, 1 closed, 2 fixed).**
