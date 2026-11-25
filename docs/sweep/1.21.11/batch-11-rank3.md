# Batch 11 — Rank-3 triage (247 items), 1.21.11 parity sweep

- **Batch:** 11 (rank-3 triage) — checks + triage only, no fixes.
- **Date:** 2026-11-25
- **Scope:** 1.21.11's OWN rank-3 ledger
  (`~/workspace/mcir-work/e6-1.21.11-behaviour/rank3-item-*.md`: 247
  triage files, one per item, items 503–749, all closed; 1.21.11 has
  NO rank3-progress.json — the triage files ARE the item source),
  triaged against the named 1.21.11 jars and the 1.21.11 template
  tree in `versions/1.21.11/` (branch `parity-sweep-1.21.11`).
  Vanilla 1.21.11's own jar/data is the spec throughout; the 26.3
  base's form was never trusted. Each module appears once (247
  distinct modules).
- **Ledger slice:** `slice.json` (same shape as the 1.21.10 batch-11 slice).

## Tally (247 items)

| Verdict | Count |
|---|---|
| pass | 243 |
| gap (defect attached) | 4 |
| needs-new-harness | 0 |
| needs-in-game-probe | 0 |
| accepted gap (ledger verdict) | 0 |

As-tallied 247 = 229 no-change + 18 ported (all behaviour checks;
non-behaviour 0 — the class-table sweep §6.1, the palette-reader
check, the lib-test check and the oracle triage are reported with
their own counts, NOT folded into the tally). The four gaps are
items 581 (sapling bonemeal — the triage claims the arm "already
returns true" but it gates on grower/live-features/tree-can-grow),
637 (number providers — the triage frames the 0-fallthrough as
26.1-only providers, but `score`/`storage`/`enchantment_level` ARE
1.21.11 providers), 662 (fuel — the triage claims "the fuel SET is
the same", but the set is EMPTY: no 1.21.11 item carries
`minecraft:cooking_fuel`), and 699 (splash water — the triage claims
"no `PotionTags` usage anywhere in the Rust", but `potion_in_tag`
reads the potion-tags map). All four are triage-file claims falsified
against the 1.21.11 jar/data + the current tree (§6.7).

**On the brief's lib-test expectation:** the brief anticipated that
1.21.11's regression tests might not run, and asked for 1.21.11's own
crate-tests run. There is NO such run: all 103 crate-tests runs ever
dispatched tested 26.1 (48), 1.21.10 (24), 1.21.7 (9 on main + fix
branches), 26.1.1 (5), 26.1.2 (2), or fix branches — none tested
1.21.11. The port's only CI evidence is the port-verify proof run
44989513836 (Nov 22 05:27 UTC, success, profile 5 pass / 18 fail of
23, D1 fail on 21 dead production tags, D2 pass, D3 pass 502/502).
The regression tests named in the triage files DO exist in the
current tree (fog 16, golem 10, social_screen 3, debug_overlay 9,
end_spike 1, growth_tests 21, block_survival 1, the knot test, the
shipwreck test) and passed locally at port time — but their CI
status for 1.21.11 is UNVERIFIED, recorded as B11-H03 (§3), not a
pass and not breakage. (Batch 0 has NOT landed for 1.21.11 — no
`batch-0-oracles.json`, no `block-behaviours.json` — so the oracle
triage here is provisional + direct javap verification, and the
chain-liveness half is B11-H01.)

Defects found: **11** (`B11-G04`–`B11-G07`, `B11-G09`–`B11-G10`,
`B11-G12`–`B11-G15`, `B11-G18`), of which 2 major, 9 minor. Nine
reproduce 1.21.7 defects (G04 dripstone narrowed, G05 splash water,
G06 number providers, G07 fuel in worse form, G12 sapling narrowed,
G13 playerdata, G14 snow tags narrowed, G15 dialog tooltip) or
1.21.10 defects (G09 spawn counts, G10 palette reader, G18 prepare
survival table); none is NEW for 1.21.11. Seven attach to ledger
items (G05→699, G06→637, G07→662, G12→581) or are cross-cutting
(G04, G09, G10, G13, G14, G15, G18) — so the defect count (11)
exceeds the behaviour-gap count (4). Oracle defects: **2**
(`B11-O01`–`B11-O02`, both minor, both cauldron oracles — §2).
Needs-harness: **3** (B11-H01 chain-liveness half, B11-H02 cauldron
table, B11-H03 lib-test CI status — §3). Accepted-gap notes carried
from the 1.21.7 sweep: **3** (`B11-A01`–`B11-A03`, §5 — not ledger
verdicts, not counted in the 247-item tally). No probe items: the
1.21.7 P01 probe is settled by the 1.21.11 jar (§6.6).

**Overlap flags for the coordinator (do not fold — merge notes only):**
three of the eleven defects are also minted by the batch that owns
the system, with the same root cause and evidence — B11-G05 ↔ B1-G03
(splash water), B11-G04 ↔ B5-G13 (dripstone), B11-G07 ↔ B4-G01
(fuel — the `burn_time` 0 consequence is the furnace half). The
snow-tag half of B11-G14 (survive.rs/weather.rs) is Batch 5's
survival territory; flagged, not folded. The coordinator dedups
oracles at merge, as in 26.1.2.

**Correction owed to the 1.21.10 ledger (flag, not a 1.21.11 item):**
the 1.21.10 run's B11-G16 (leather wash dead) is a MISREAD — the
1.21.10 tree's `removes_dye` reads `minecraft:dyeable` (cauldron.rs:8-13),
not `cauldron_can_remove_dye` (which appears only in the :190 doc
comment), and 1.21.10's datapack HAS the `dyeable` tag with the same
6 values. The wash works in 1.21.10 too. The 1.21.7 B11-G23 was
correct (the 1.21.7 tree genuinely reads the absent tag, oracle-confirmed)
— the template fixed it, the 1.21.10 delta inherited the fix, and the
1.21.10 agent misread the doc comment as the code. See §6.6.

**Primary re-checks — the 1.21.10 sweep's 35 items, each decided:**

| 1.21.10 item | Verdict for 1.21.11 | 1.21.11 item |
|---|---|---|
| B11-A01 trim inline form | CARRIES (accepted-gap) | B11-A01 |
| B11-A02 special-foil decal | CARRIES (accepted-gap) | B11-A02 |
| B11-A03 NBT separator + per-player | CARRIES (accepted-gap) | B11-A03 |
| B11-G06 number providers | REPRODUCES | B11-G06 |
| B11-G07 recipe-book fuel | REPRODUCES (worse form: empty list) | B11-G07 |
| B11-G09 F3 spawn-counts | REPRODUCES | B11-G09 |
| B11-G10 palette reader | REPRODUCES | B11-G10 |
| B11-G12 sapling bonemeal | REPRODUCES (narrowed: lone mega case) | B11-G12 |
| B11-G13 player data path | REPRODUCES | B11-G13 |
| B11-G14 golem stroll + snow tags | REPRODUCES (narrowed: snow-tag half only; golem half FIXED) | B11-G14 |
| B11-G15 dialog tooltip on focus | REPRODUCES (hover-only) | B11-G15 |
| B11-G16 cauldron leather wash | FIXED (reads `dyeable`, tag present) | — |
| B11-G17 F3 footer help suffix | WITHDRAWN (vanilla 1.21.11 has no such suffix) | — |
| B11-G18 prepare survival table | REPRODUCES | B11-G18 |
| B11-G19 worldborder setSize feedback | FIXED (`percent_f`, %.1f — the delta introduced the inversion) | — |
| B11-G20 beds | FIXED (dual dispatch live) | — |
| B11-G21 redstone wire | FIXED (dual name live) | — |
| B11-G22 farmland/paths | FIXED (duals live) | — |
| B11-G23 snowy | FIXED (duals live) | — |
| B11-G24 fungus/roots | FIXED (duals live) | — |
| B11-G25 riptide | FIXED (no riding check) | — |
| B11-G26 lead knot | FIXED (knot first) | — |
| B11-G27 creeper defuse | FIXED (no alive check) | — |
| B11-G28 polar bear cubs | FIXED (no fox selector) | — |
| B11-G29 mushroom | FIXED (gate + grow block) | — |
| B11-G30 social chat gate | FIXED (chat_allowed gate + tests) | — |
| B11-G31 wandering trader | FIXED (level.dat + WORLD_SURFACE) | — |
| B11-G32 F3 lines | FIXED (Day, MB, two-space Mood) | — |
| B11-G33 chorus on end stone | FIXED (hardcoded) | — |
| B11-G34 sound_variant on non-wolves | FIXED (Rust gates on Wolf only) | — |
| B11-G35 shipwreck re-adjustment | FIXED (Rust re-adjusts like vanilla) | — |
| B11-G36 F3 chart footer | FIXED (settled by jar) | — |
| B11-G37 chatDisabledOrBlocked | FIXED (chat gate via chat_allowed; blocked half vacuous without UserApiService) | — |
| B11-H01 chain-liveness half | CARRIES (needs-harness) | B11-H01 |
| B11-H02 cauldron table | CARRIES (needs-harness; no longer confirmatory for a live defect) | B11-H02 |

**Secondary re-checks — the 1.21.7 sweep's 31 items, each decided:**

| 1.21.7 item | Verdict for 1.21.11 | 1.21.11 item |
|---|---|---|
| B11-A01 trim inline form | CARRIES (accepted-gap) | B11-A01 |
| B11-A02 NBT sound_variant on non-wolves | FIXED (Rust gates on Wolf only) | — |
| B11-A03 special-foil decal | CARRIES (accepted-gap) | B11-A02 |
| B11-A04 NBT separator + per-player | CARRIES (accepted-gap) | B11-A03 |
| B11-G01 beds | FIXED (dual dispatch live) | — |
| B11-G02 redstone wire | FIXED (dual name live) | — |
| B11-G03 farmland/paths | FIXED (duals live) | — |
| B11-G04 snowy | FIXED (duals live) | — |
| B11-G05 fungus/roots | FIXED (duals live) | — |
| B11-G06 dripstone | REPRODUCES (narrowed: tag gate) | B11-G04 |
| B11-G07 sapling bonemeal | REPRODUCES (narrowed: lone mega case) | B11-G12 |
| B11-G08 riptide | FIXED (no riding check) | — |
| B11-G09 lead knot | FIXED (knot first) | — |
| B11-G10 creeper defuse | FIXED (no alive check) | — |
| B11-G11 polar bear cubs | FIXED (no fox selector) | — |
| B11-G12 number providers | REPRODUCES (datapack-only; 0/1329 vanilla tables) | B11-G06 |
| B11-G13 splash water | REPRODUCES | B11-G05 |
| B11-G14 player data path | REPRODUCES | B11-G13 |
| B11-G15 recipe-book fuel | REPRODUCES (worse form: empty list) | B11-G07 |
| B11-G16 golem stroll + snow tags | REPRODUCES (narrowed: snow-tag half only; golem half FIXED) | B11-G14 |
| B11-G17 mushroom | FIXED (gate + grow block) | — |
| B11-G18 social chat gate | FIXED (chat_allowed gate + tests) | — |
| B11-G19 wandering trader | FIXED (level.dat + WORLD_SURFACE) | — |
| B11-G20 F3 lines | FIXED (Day, MB, two-space Mood) | — |
| B11-G21 chorus on end stone | FIXED (hardcoded) | — |
| B11-G22 dialog tooltip on focus | REPRODUCES (hover-only) | B11-G15 |
| B11-G23 cauldron leather wash | FIXED (reads `dyeable`, tag present) | — |
| B11-H01 shipwreck re-adjustment | FIXED (Rust re-adjusts like vanilla) | — |
| B11-H02 cauldron table | CARRIES (needs-harness) | B11-H02 |
| B11-O01 VanillaCauldronOracle | REPRODUCES (3-arg drop; the 1.21.7 repair never reached this tree) | B11-O01 |
| B11-P01 F3 chart footer | FIXED (settled by jar; no "For help" suffix in 1.21.11) | — |

Headline: the template's 247-item ledger holds up — 243 verdicts
re-verify against the 1.21.11 jar and the current tree, all 18
ported items re-verify individually, and the class table carries dual
names for every 1.21.11 rename (beds, wire, paths, snowy, fungus,
roots all dispatch live). 20 of the 35 1.21.10 re-checks and 18 of
the 31 1.21.7 re-checks are FIXED in this tree — the template fixed
more than its first child kept (the child lost the golem roll-order
fix, the leather-wash fix's reputation if not its code, and inverted
the worldborder feedback). But the template-era inheritance failures
persist where the port reads 26.x-shaped data that 1.21.11 never
had: the chunk palette reader accepts only the post-5006
`id`/`properties` form while vanilla 1.21.11 still writes
`Name`/`Properties` palettes — so a vanilla-written 1.21.11 world
loads as void (B11-G10, major, the template's own defect, inherited
BY 1.21.10 and 26.1.2, not from them); splash water reads a
potion-tag map that is empty because 1.21.11 has no potion tags
(B11-G05, major); and the fuel list reads a cooking-fuel component
1.21.11 never defines (B11-G07, minor, worse form — the list is
empty, and the triage file's "the fuel SET is the same" is false).

## §1 — Defects (11)

### B11-G04 — dripstone gates on `minecraft:speleothems`, which 1.21.11 never defined (minor; overlaps B5-G13)

- **Source:** 1.21.7 B11-G06 (dispatch arm) + 1.21.10 B11-G04, reproduced
  fresh in 1.21.11's tree.
- **Vanilla 1.21.11:** no `minecraft:speleothems` tag file (639 tag
  files; 0 speleothems). Dripstone survival, placement and the
  `speleothem` behaviours gate on `PointedDripstoneBlock`/`Speleothem`
  class checks, not a tag.
- **Rust:** the dispatch half is FIXED — all five Speleothem arms
  dispatch `Class::Speleothem | Class::PointedDripstone`
  (place.rs:1156, shapes.rs:745, survive.rs:326, tick.rs:257/443) and
  `Class::PointedDripstone` is live. The tag half REPRODUCES:
  `minecraft:speleothems` is still read at falling.rs:55,
  place.rs:1409, shapes.rs:1186, speleothem.rs:15/18/90. Every read
  hits an empty tag, so every tag-gated dripstone check fails.
- **Severity:** minor — dripstone placement/survival is otherwise
  live via the dual dispatch; the tag gates are the residue. Overlap
  with B5 (dripstone is Batch 5's system); the B5 item carries the
  merge note.

### B11-G05 — splash water never douses fire (MAJOR; overlaps B1-G03)

- **Source:** 1.21.7 B11-G13 + 1.21.10 B11-G05, reproduced fresh.
  Attached to item 699 (gap).
- **Vanilla 1.21.11:** `AbstractThrownPotion` gates the douse on
  `potionContents.is(Potions.WATER)` (offsets 55–63) — a direct
  potion check, no tag. There is NO `PotionTags` class in either
  1.21.11 jar and 0 potion tag files in the datapack.
- **Rust:** `potion_in_tag` (splash_water.rs:42-51) reads the
  potion-tags map and gates the effect on it. The map is empty
  (nothing populates it — there are no potion tags to populate it
  from), so splash water never douses fire, never wets farmland,
  never cools magma. Item 699's triage claims "no `PotionTags`
  usage anywhere in the Rust (the only `potion_tags` hit is an
  unrelated probe field)" — flatly false; the gate IS the usage.
- **Severity:** major — a whole interaction (water splash dousing)
  is dead, and the ledger file asserted it away. Overlap with B1
  (fluids); the B1 item carries the merge note.

### B11-G06 — `score`/`storage`/`enchantment_level` number providers return 0 (minor; datapack-only)

- **Source:** 1.21.7 B11-G12 + 1.21.10 B11-G06, reproduced fresh.
  Attached to item 637 (gap).
- **Vanilla 1.21.11:** `NumberProviders` has exactly 6 entries
  (CONSTANT, UNIFORM, BINOMIAL, SCORE, STORAGE, ENCHANTMENT_LEVEL)
  — all six are live 1.21.11 providers. (`sum` and
  `environment_attribute` are the 26.1-only ones.)
- **Rust:** `loot.rs:1344/1346` (`_ => 0`) and `:1363` (`_ => 0.`),
  and `horizon-items/items.rs:1934-1972` (`_ => 0.0`), yield 0 for
  any provider that isn't constant/uniform/binomial. Item 637's
  triage says the 0-fallthrough is fine because "the two 26.1-only
  providers can never appear in a 1.21.11 loot table" — true of
  those two, but it misses that three LIVE 1.21.11 providers also
  fall through to 0.
- **Severity:** minor — datapack-only: 0 of the 1,329 vanilla loot
  tables use `enchantment_level`/`score`/`storage`, so no vanilla
  loot is affected; a datapack using them gets 0 counts.

### B11-G07 — recipe-book fuel list is EMPTY, not merely misordered (minor; overlaps B4-G01)

- **Source:** 1.21.7 B11-G15 + 1.21.10 B11-G07, reproduced fresh in a
  worse form. Attached to item 662 (gap).
- **Vanilla 1.21.11:** fuel is a CODE table — `FuelValues.fuelItems()`
  + `vanillaBurnTimes` (server jar), and `SlotDisplay$AnyFuel.resolve`
  streams `FuelValues.fuelItems`. There is no `minecraft:cooking_fuel`
  or `minecraft:brewing_fuel` component: `DataComponents` (105 fields)
  has neither, and 0 of the 1,505 datagen items carry them.
- **Rust:** `items.rs:1040-1042` reads `minecraft:cooking_fuel` for
  the fuel list; `recipe_book.rs:566-585` + `native_panel_recipes.rs:173-190`
  group an EMPTY list; `burn_time` (providers.rs:108-116) returns 0.
  There is no `FuelValues` synthesis anywhere in the Rust. Item 662's
  triage claims "the fuel SET is the same" — false: the set is
  empty. The comment in native_panel_recipes.rs admits the 26.1 form
  and cross-refs PORT-NOTES item 662, which EXISTS in 1.21.11's
  PORT-NOTES (line 12526) — unlike 1.21.10, where it dangled.
- **Severity:** minor — the recipe book shows no fuel; the furnace
  half (burn_time 0) is B4-G01's territory, flagged as overlap.

### B11-G09 — F3 spawn-counts use two-letter abbreviations, vanilla uses initials (minor)

- **Source:** 1.21.10 B11-G09, reproduced fresh.
- **Vanilla 1.21.11:** the spawn-count entries are formed by
  `charAt(0) + toUpperCase` — single initials (M, C, A, X, U, W, M).
- **Rust:** `debug_overlay.rs:394` uses a two-letter abbreviation
  table (`MO`, `C`, `AM`, `AX`, `UWC`, `WC`, `WA`, `MI`). Cosmetic
  only — the counts are right, the labels are wrong.

### B11-G10 — vanilla-written worlds load as void: the palette reader accepts only the post-5006 form (MAJOR)

- **Source:** 1.21.10 B11-G10, reproduced fresh — all three links
  re-verified against the 1.21.11 jar/data/tree. This is the
  template's OWN defect, inherited BY 1.21.10 and 26.1.2, not from
  them.
- **Link 1 — the reader:** `BlockState::from_nbt`
  (horizon-world/src/chunk.rs:208) reads a strict `"id"` key and
  errors "block state is missing id". There is no `from_palette_nbt`
  anywhere in the tree (0 occurrences). The datafix that converts
  `Name`→`id` is key 5006 (fixes_c.rs:155), but `update()` runs
  fixes only where `k/10 <= to_version` (datafix/mod.rs:223) and
  `CURRENT = DATA_VERSION = 4671` (horizon-core/src/version.rs:11)
  — 5006 > 4671, so the conversion never runs for a 1.21.11 world.
- **Link 2 — the load path:** `upgrade_record` early-returns for
  DataVersion ≥ 4671 (horizon-client/src/save.rs:2433);
  `decode_column` maps unresolvable palette entries to air
  (save.rs:1788, `saved_state_id(p).map_or(air, ...)`); `saved_state_id`
  uses `BlockState::from_nbt(tag).ok()` (horizon-world/src/registry.rs:230),
  which fails on every `Name`-form entry.
- **Link 3 — the world:** the reference world
  (`~/workspace/vanilla-world-mc/1.21.11/"New World"`, level.dat
  DataVersion 4671, seed -76134417269081163) writes palettes as
  `Name`/`Properties` compounds (verified with nbt_probe.py on
  r.0.0.mca, 3 chunks sampled) — the form vanilla 1.21.11 itself
  writes. Every palette entry fails the reader; every chunk decodes
  to air.
- **Severity:** major — a vanilla-written 1.21.11 world loads as
  void. Consequence is statically derived from the three verified
  links, not demonstrated in-game (no live load was run).

### B11-G12 — bonemealing a lone mega sapling wastes nothing but does nothing (minor; narrowed)

- **Source:** 1.21.7 B11-G07 + 1.21.10 B11-G12, reproduced fresh,
  narrowed. Attached to item 581 (gap).
- **Vanilla 1.21.11:** `SaplingBlock.isValidBonemealTarget` is
  `iconst_1` — always valid; `performBonemeal` calls `advanceTree`,
  which for a mega tree with no 2x2 partner returns without growing
  (and the meal IS consumed, because validity is unconditional).
- **Rust:** `bonemeal_valid` (bonemeal.rs:331-340) returns false
  when there is no grower, no live features, or the tree can't grow
  — so a lone mega sapling keeps its meal and nothing happens.
  Item 581's triage claims the Sapling arm "already returns true" —
  it returns true only after passing the growability gates, which
  conflates validity with growability.
- **Severity:** minor — narrowed to the lone-mega case (and any
  sapling whose grower has no live features); the common case
  (valid sapling, live features) works.

### B11-G13 — player data saved to `players/data/`, vanilla uses flat `playerdata/` (minor)

- **Source:** 1.21.7 B11-G14 + 1.21.10 B11-G13, reproduced fresh.
  Item 617's triage is honest about this (rank-2 precedent) — the
  defect is cross-cutting, not a ledger gap.
- **Vanilla 1.21.11:** `LevelResource.PLAYER_DATA_DIR` is the flat
  `"playerdata"` (PLAYER_OLD_DATA_DIR is `"players"` — the legacy
  dir, not the live one).
- **Rust:** `lan/playerdata.rs:18` writes `players/data/`.
- **Severity:** minor — a save written by the port is not one
  vanilla 1.21.11 reads player data from (and vice versa); the
  world itself loads.

### B11-G14 — snow-layer survival reads the 26.x tag names (minor; narrowed — golem half FIXED)

- **Source:** 1.21.7 B11-G16 + 1.21.10 B11-G14, reproduced fresh,
  NARROWED.
- **Vanilla 1.21.11:** `SnowLayerBlock.canSurvive` uses
  `SNOW_LAYER_CANNOT_SURVIVE_ON` (offset 14) +
  `SNOW_LAYER_CAN_SURVIVE_ON` (offset 27). The 26.x names
  (`cannot_support_snow_layer`, `support_override_snow_layer`) do
  not exist in 1.21.11 data (0 files).
- **Rust — golem half FIXED:** the golem roll order is the 1.21.11
  form (golem.rs:152-158 draws the 0.7 roll first, item 520
  verified) and golem.rs:579/582 uses the 1.21.11 tag names.
  **Snow-tag half REPRODUCES:** horizon-world still reads the
  absent 26.x names at data.rs:431-432, survive.rs:314/316,
  weather.rs:113/116 — every read hits an empty tag, so snow-layer
  survival falls back to the default branch in those paths.
- **Severity:** minor. Overlap flag: the surviving reads are in
  Batch 5's survival territory.

### B11-G15 — dialog tooltip shows on hover only, vanilla shows it on focus too (minor)

- **Source:** 1.21.7 B11-G22 + 1.21.10 B11-G15, reproduced fresh.
  Item 615's triage is honest about this (known gap, no surface) —
  the defect is cross-cutting, not a ledger gap.
- **Vanilla 1.21.11:** the dialog tooltip renders when the widget
  is hovered OR focused (`ItemDisplayWidget` uses
  `isHoveredOrFocused`, offset 104).
- **Rust:** `dialogs.rs:705` shows the tooltip on hover only.
- **Severity:** minor — keyboard-only dialog navigation loses the
  tooltip.

### B11-G18 — prepare's survival table cites dead class names (minor)

- **Source:** 1.21.10 B11-G18, reproduced fresh.
- **Vanilla 1.21.11:** the classes are `DirtPathBlock`,
  `PointedDripstoneBlock`, `FarmBlock`, `RedStoneWireBlock` — there
  is no `PathBlock`, `SpeleothemBlock` or `RedstoneWireBlock`.
- **Rust:** `horizon-prepare/src/survival.rs` still cites the dead
  names at :10 (`PathBlock`), :21 (`SpeleothemBlock`), :45
  (`RedstoneWireBlock`); :31 (`FarmBlock`) is fixed. The table is
  the prepare-time survival lookup, so the dead entries never match
  and those blocks fall back to the default survival rule.
- **Severity:** minor — prepare-time only, default-rule fallback.

## §2 — Oracle defects (2)

No oracle repairs were made in this batch — the task is triage
only, and the 1.21.11 Batch 0 (oracle table) has not landed, so the
oracle triage here is provisional + direct javap verification
against the 1.21.11 jars. The oracle files live at
`versions/1.21.11/mods/horizon/src/test/java/dev/horizonlab/parity/`.
Both defects are compile failures: any harness run that compiles
these oracles fails before it measures anything.

### B11-O01 — VanillaCauldronOracle calls the 3-arg `Player.drop` with `Prediction` (minor)

- **Source:** 1.21.7 B11-O01 (repaired in 1.21.7's parity-sweep
  branch, commit c754e3a2 — the repair never reached the 1.21.11
  tree) + 1.21.10 B11-O01.
- **Vanilla 1.21.11:** `Player.drop` is 2-arg
  (`drop(ItemStack, boolean)`). There is no `Prediction` class in
  either 1.21.11 jar (verified: NOT FOUND in client-named and
  server-game-named).
- **Oracle:** `VanillaCauldronOracle.java:118` calls the 3-arg drop
  with a `Prediction` argument — does not compile against 1.21.11.

### B11-O02 — VanillaLavaCauldronItemOracle overrides a method that does not exist (minor)

- **Source:** 1.21.10 B11-O02, reproduced fresh.
- **Vanilla 1.21.11:** `getNextEntityId` is absent from both `Level`
  and `ServerLevel` (verified via class bytes — no such method in
  either). `ServerLevel`'s constructor is 11-arg (with
  `RandomSequences` + the `randomSequences` field).
- **Oracle:** `VanillaLavaCauldronItemOracle.java:28` overrides
  `getNextEntityId` — does not compile. The same oracle has a
  SECOND compile failure: its `Lvl` constructor calls the 10-arg
  `super(...)` while `ServerLevel`'s constructor is 11-arg.

## §3 — Needs-new-harness (3)

### B11-H01 — chain-liveness half of the class-table sweep (needs-harness)

The class-table sweep (§6.1) verified the Rust-side half: 265 of
271 mappings jar-live, 6 dead, all duals live. The chain half —
whether every `Class` enum value is REACHED by a prepared
block-behaviours chain — cannot run: there is no
`block-behaviours.json` for 1.21.11 anywhere (only 1.21.7's exists,
at `~/workspace/mcir-work/mob-spawn/resources/`), and Batch 0 has
NOT landed (no `batch-0-oracles.json` in `docs/sweep/1.21.11/`;
`.local/minecraft/1.21.11/` has prepared.json/reports but no
block-behaviours.json). Needs the Batch 0 prepare to produce the
1.21.11 chain, then a re-run of the liveness half.

### B11-H02 — cauldron behaviour table (needs-harness; no longer confirmatory for a live defect)

The cauldron table harness (leather wash, dye removal, lava
cauldron item interactions) is blocked on the oracle repairs
(B11-O01/O02 — the oracles do not compile). CARRIES from 1.21.10
B11-H02 and 1.21.7 B11-H02 — but its status has changed: the wash
itself WORKS in this tree (B11-G16 FIXED, §6.6), so the harness is
no longer confirmatory for a live defect; it is a coverage gap,
not a defect probe.

### B11-H03 — no crate-tests run has ever tested 1.21.11 (needs-harness)

All 103 crate-tests runs ever dispatched tested 26.1 (48), 1.21.10
(24), 1.21.7 (9 on main + fix branches), 26.1.1 (5), 26.1.2 (2), or
fix branches — none tested 1.21.11. The port's only CI evidence is
the port-verify proof run 44989513836 (Nov 22 05:27 UTC, success,
profile 5 pass / 18 fail of 23, D1 fail on 21 dead production tags,
D2 pass, D3 pass 502/502). The regression tests named in the
triage files exist in the current tree and passed locally at port
time (fog 16, golem 10, social_screen 3, debug_overlay 9,
end_spike 1, growth_tests 21, block_survival 1, the knot test at
knot_identity_tests.rs:258, the shipwreck test
`repurposed_shipwreck_needs_no_solid_block_around`). Dispatch a
crate-tests run for 1.21.11 to verify the lib suites run green.
Do NOT read this as breakage — it is an unverified status.

## §4 — Needs-in-game-probe (none)

No probe items. The 1.21.7 P01 footer probe is settled by the
1.21.11 jar: `DebugScreenOverlay`'s footer template is
`"Debug charts: \u0001 Profiler \u0001; \u0001 \u0001\u0001; \u0001 \u0001\u0001"`
with FPS + TPS/FPS and `"To edit: press \u0001"` — zero "For help"
occurrences in 1.21.11 (B11-G17 withdrawn, §6.6). The B11-G10 void
consequence is statically derived from three verified links, not
demonstrated in-game; a live load would confirm it, but the three
links leave no room for the load to succeed.

## §5 — Accepted gaps (3)

Carried from the 1.21.7/1.21.10 sweeps; these are gaps the
1.21.11 port shares with its parent (26.1) — noted, not defects
of this delta. They are NOT ledger verdicts and are NOT counted
in the 247-item tally.

### B11-A01 — trim material inline form read as string only (accepted-gap)

`enchant.rs:335-337` reads `provides_trim_material` via
`.as_str()` only. Vanilla 1.21.11's `ProvidesTrimMaterial.material`
is `EitherHolder<TrimMaterial>` (flag, then holder or key). All 11
datagen items carrying the component use the string form, so no
vanilla item is affected. (The NETWORK adapter half — item 597 —
is handled correctly via the `either_holders_and_tag_resistance`
quirk; this note is the item-data read only.)

### B11-A02 — special-foil decal not drawn (accepted-gap)

`Foil::Special` exists (item_render.rs:149-150) but gui.rs:422 +
item_render.rs:1163 treat all non-None foil identically — the
glint is drawn over the full quad with no special decal. Item
534's triage notes it honestly. Pre-existing in both versions.

### B11-A03 — NBT separator ignored + per-player resolution (accepted-gap)

`text_commands.rs:131/:241` always joins with `", "`; the Rust
resolves a title/tellraw component ONCE for all players while
vanilla resolves PER PLAYER (`@s` in a title resolves to each
recipient). Item 512's triage notes both honestly as "gaps from
BOTH versions equally". A bigger rework, not a delta.

## §6 — Systemic findings

### §6.1 — Class-table sweep (271 mappings; 265 live, 6 dead; chain half blocked — B11-H01)

`crates/horizon-world/src/level/classes.rs` carries 271
class-name mappings. Jar-verified against 1.21.11's own jars:
**265 live, 6 dead**. The 6 dead are `AbstractBed`,
`StrawBed`, `Speleothem`, `SulfurSpike`, `ShelfMushroom`,
`PotentSulfur` — all present in 26.3, all absent from 1.21.11.

- All 7 duals live via their 1.21.11 alt: `FarmBlock`,
  `SnowyDirtBlock`, `SpreadingSnowyDirtBlock`, `FungusBlock`,
  `RootsBlock`, `DirtPathBlock`, `RedStoneWireBlock`.
- All 5 `AbstractBed` arms dispatch
  `Class::AbstractBed | Class::Bed` (hooks.rs:469, interact.rs:365,
  place.rs:356, place.rs:1483, shapes.rs:124) — beds place, break
  and sleep live (B11-G20 FIXED).
- All 5 `Speleothem` arms dispatch
  `Class::Speleothem | Class::PointedDripstone` (place.rs:1156,
  shapes.rs:745, survive.rs:326, tick.rs:257/443) — dripstone
  dispatches live; only the tag gates are dead (B11-G04).
- The four 26.x-only names (`StrawBed`, `SulfurSpike`,
  `ShelfMushroom`, `PotentSulfur`) have sole-dispatch arms but no
  blocks to fire on — dead code, not defects. They are the
  template's residue from the 26.3 base; the leakage-audit tool
  should flag them, and a re-base of the E6 children must not
  carry them down as live names.
- Chain half: BLOCKED (B11-H01) — no `block-behaviours.json` for
  1.21.11, Batch 0 not landed. The 265/271 count is the Rust-side
  half only and is NOT folded into the tally.

### §6.2 — 26.x tag reads that 1.21.11 never defined (six absent tags still read)

Datapack inventory: 639 tag files, 1,329 loot tables, 0 potion tag
files. The Rust still reads six tags that do not exist in 1.21.11
data:

| Tag | Read at | Status |
|---|---|---|
| `minecraft:speleothems` | falling.rs:55, place.rs:1409, shapes.rs:1186, speleothem.rs:15/18/90 | Live defect (B11-G04) |
| `minecraft:cauldron_can_remove_dye` | cauldron.rs:190 (doc comment ONLY) | Harmless — the code reads `dyeable` (see §6.6) |
| potion tags (the whole map) | splash_water.rs:42-51 | Live defect (B11-G05, major) |
| `cannot_support_snow_layer` | data.rs:431-432, survive.rs:314/316, weather.rs:113/116 | Live defect (B11-G14, narrowed) |
| `support_override_snow_layer` | same sites | Live defect (B11-G14, narrowed) |
| `sculk_growth_inhibitors` | (sculk growth gate) | Read, tag absent — no B11 defect attached; flagged for the batch that owns sculk |

Present and correctly used: `snow_layer_can_survive_on`,
`snow_layer_cannot_survive_on`, `mushroom_grow_block`,
`convertable_to_mud` (vanilla spelling — the Rust's
`convertible_to_mud` read is the absent one), `dripstone_replaceable_blocks`.
Also absent (verified, no live read found in this batch's scope):
`supports_chorus_plant`/`supports_chorus_flower`,
`overrides_mushroom_light_requirement`, `causes_suffocation`,
`suppresses_bounce`.

### §6.3 — 26.x component reads that 1.21.11 never defined

`DataComponents` has 105 fields in 1.21.11 and NO
`COOKING_FUEL` / `BREWING_FUEL` / `COMPOSTABLE` / `VILLAGER_FOOD` /
`MOB_VISIBILITY` (verified by javap on the 1.21.11 server jar).
The Rust reads `minecraft:cooking_fuel` (items.rs:1040-1042) and
`minecraft:brewing_fuel` — both return nothing for every 1.21.11
item (0/1,505 datagen items carry them). This is the B11-G07 root
cause and the same pattern as §6.2: the port reads 26.x-shaped
data that 1.21.11 never had. Note for the re-base: 1.21.11 DOES
have `ZOMBIE_NAUTILUS_VARIANT`, `KINETIC_WEAPON`,
`SWING_ANIMATION`, `USE_EFFECTS` — these arrived in 1.21.11 and
are absent in 1.21.10, so the 1.21.10 run's "26.3-only" list for
them does not apply here.

### §6.4 — Re-base warnings (1.21.11 form ≠ 26.3 form, Rust matches 1.21.11 — do NOT propagate the 26.3 form down)

- **Fluid-current floor (item 567):** the 26.x `Tracker` has a
  1e-5 squared-length floor; 1.21.11's inline logic has none, and
  the Rust matches 1.21.11 (entity_fluid.rs:86-90). **1.21.7 vanilla
  HAS the floor** — the floor removal must NOT propagate to 1.21.7
  (already noted in the 1.21.7 sweep's systemic note; restated
  because the template is the re-base source).
- **Golem roll order (item 520):** 1.21.11 stores the 0.7 roll
  first; the template matches. The 1.21.10 delta LOST this fix
  (its golem.rs draws the 0.3 roll first) — a delta regression to
  carry into the 1.21.10 fix list, not a template defect.
- **Stopwatches:** 1.21.11 HAS stopwatches (the 1.21.10 run's gap
  item 592 was a 1.21.10-only deletion). Do not propagate the
  1.21.10 stopwatch removal upward.
- **Nautilus / Spear / CamelHusk / Parched:** all present in
  1.21.11 (21 Nautilus classes, Spear classes, CamelHusk, Parched
  — verified in the 1.21.11 jars). They arrived in 1.21.11 and are
  absent in 1.21.10; the 1.21.10 run's "26.3-only" list for them
  does not apply here. Sulfur: 0 classes in 1.21.11 — 26.3-only,
  correctly cited as such.

### §6.5 — Evidence-token check (762 tokens; 489 live; 62 26.3-only, all correctly cited)

762 distinct CamelCase tokens across the 247 triage files: 489
live in the 1.21.11 jars; 62 are 26.3-only and ALL are correctly
cited as absent/renamed/26.x-side (five initially suspicious
verified OK: `AbstractBedBlock` rename in item 726,
`IntProviders` rename in 691, `MenuTabBar` doc citation in 619,
`PlayerFaceExtractor` rename in 523/633, `SavedDataStorage`
"not" claim in 507); 211 in neither jar (inner classes,
Rust-side names, JDK classes — expected). No token evidence
contradicts a verdict.

Named-class check: 129 named classes extracted (the regex only
matched some header formats); all 17 apparent absences resolved
(inner classes, fields, or correctly-noted absences, e.g.
`IMEPreeditOverlay` marked "26.1-only IME" in item 529).
Module resolution: 228 items resolve to a file under `crates/`;
3 launcher files exist (650 java_launch.rs, 671 screenshots.rs,
731 wardrobe.rs under `launcher/src/`); 14 items name Java
packages, not Rust files (612, 626, 652, 653, 654, 672, 674, 732,
733, 734, 736, 737, 738, 739); 505/561 resolve under
horizon-command.

Spot-check sample (seed 11, 20 items: 534, 561, 562, 593, 632,
633, 637, 639, 649, 650, 662, 670, 677, 681, 719, 723, 725, 739,
740, 742): all 20 agreed with the classification — 18 pass
(named classes jar-verified present, no-op diffs as claimed),
2 gap (637, 662 — already flagged by the defect cross-checks,
confirmed by the sample). Sample agreement 20/20.

### §6.6 — The 1.21.7-defect cross-check + two corrections

The secondary re-check table (above) is the cross-check: 18 of
the 31 1.21.7 items are FIXED in the template, 9 reproduce, 4
carry. The fixed set is the template's real achievement — every
ported item (541 riptide, 572 lead, 666 creeper, 622 polar bear,
533 mushroom, 523 social, 535 trader, 571 F3, 544 chorus, 640
shipwreck, 519 footer, 520 golem) re-verifies, and the dual-name
class table fixes beds, wire, farmland, paths, snowy, fungus and
roots without any ported item at all.

Two corrections the coordinator must carry:

1. **The 1.21.10 ledger's B11-G16 NEVER EXISTED.** The 1.21.10
   agent cited cauldron.rs:190 — a doc comment naming
   `cauldron_can_remove_dye` — and concluded the wash was dead.
   The code at cauldron.rs:8-13 reads `minecraft:dyeable`, the
   tag EXISTS in 1.21.10's datapack with the same 6 values, and
   the wash strips `dyed_color` (cauldron.rs:217-226). The wash
   works in 1.21.10, in 1.21.11, and — per the 1.21.7 sweep — the
   1.21.7 tree genuinely read the absent tag (oracle-confirmed),
   so 1.21.7's B11-G23 was correct and the template fixed it.
   The 1.21.10 ledger needs a NEVER EXISTED correction for
   B11-G16; the 1.21.11 ledger records it as FIXED.
2. **B11-G17 "For help" suffix — withdrawn, not a defect.**
   Vanilla 1.21.11's `DebugScreenOverlay` footer has no "For
   help" string (zero occurrences in the jar); the Rust matches
   1.21.11 exactly (item 519 + javap). The 1.21.10 run's B11-G17
   was a 26.3-form expectation, not a 1.21.11 defect. Recorded
   as a closed item, not a gap.

### §6.7 — Ledger-evidence staleness (4 items)

Four triage files make factual claims that the 1.21.11 jar/data +
current tree falsify. These are the four gap verdicts; each is a
staleness or misread in the ledger, not a code regression:

- **581** (sapling): claims the arm "already returns true" — it
  returns true only after the growability gates.
- **637** (number providers): claims only the two 26.1-only
  providers would yield 0 — misses the three live 1.21.11
  providers that also yield 0.
- **662** (fuel): claims "the fuel SET is the same" — the set is
  empty (0/1,505 items carry the component).
- **699** (splash water): claims "no `PotionTags` usage anywhere
  in the Rust" — `potion_in_tag` IS the usage.

Two near-misses that PASS because the triage is honest: **615**
(dialog tooltip — known gap recorded, no surface claimed) and
**617** (playerdata — documented rank-2 precedent). **558**
(cauldron) passes — its dyeable-tag claim is correct, and it is
the file the 1.21.10 agent should have believed.

## §7 — Method notes

- **Jar discipline:** all javap/bytecode verification ran against
  the NAMED 1.21.11 jars (`client-named.jar` 10,291 classes,
  `server-game-named.jar` 7,163 classes). `server.jar` is a
  bundler and was never javap'd. The 26.3 jar
  (`client-26.3.jar`, 11,383 classes) was used ONLY for
  absence checks (which names exist in 26.3 but not 1.21.11).
- **javap batching:** javap is ~15s per JVM invocation; calls
  were batched (a1/a2/a3, b1/b2/b3, c1/c2/c3) and the output kept
  in `javap/`.
- **Reference world:** read in place at
  `~/workspace/vanilla-world-mc/1.21.11/"New World"`; no world
  files copied or moved; no git run inside the clone.
- **No git anywhere** (coordinator owns all git). No writes to
  /tmp. No sibling agents' files touched. The worktree is
  read-only for this batch.
- **Verification standard:** a ported item passes only when the
  named Rust code exists in the CURRENT tree AND the vanilla form
  is confirmed by javap on the 1.21.11 jar. A no-change item
  passes when its module resolves in the tree, its named classes
  are jar-present (where extractable), and no defect cross-check
  contradicts it. The 20-item spot-check sample (seed 11)
  agreed 20/20.
- **What was NOT verified:** in-game demonstration (no live load
  for B11-G10 — statically derived from three verified links);
  the lib-test CI status (B11-H03 — no run exists); the
  chain-liveness half (B11-H01 — Batch 0 not landed); oracle
  compilation (B11-O01/O02 are javap-derived, not compiled —
  the oracles' own compile would need the Batch 0 harness).

## §8 — Per-item triage table (all 247 items)

Verdict key: **pass** — the ledger verdict re-verifies against the 1.21.11 jar + the current tree; **gap** — the ledger verdict's factual claim is falsified (defect attached). Ledger column: `ported` = the port changed code for this item; `no-change` = the port recorded no change. The 18 ported items were each verified individually against the jar; the 4 gaps are individually evidenced; no-change items carry their module resolution + the triage file's own claim + the verification basis (family-pass = named classes jar-checked where extractable; SPOT-CHECKED = in the seed-11 sample, named classes jar-verified present).

| Item | Module | Ledger | Verdict | Evidence |
|---|---|---|---|---|
| 503 | `fog.rs` | ported | **pass** | PORTED verified: `block_light_red_flicker` in fog.rs:75/84/99-100; 0 `LightmapRenderStateExtractor` citations; jar: `LightTexture.blockLightRedFlicker` exists, extractor absent. |
| 504 | `mobs/spear_use.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/mobs/spear_use.rs); ledger verdict no-change (doc citation only); family-pass (named classes jar-checked where extracted). |
| 505 | `command/codec.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 506 | `item_render/gui.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/item_render/gui.rs); ledger verdict no-change (doc citation only); family-pass (named classes jar-checked where extracted). |
| 507 | `datafix/world_upgrader.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/datafix/world_upgrader.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 508 | `blockstate/choice.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-assets/src/blockstate/choice.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 509 | `game/debug_lines.rs` | ported | **pass** | PORTED verified: `pU` field in debug_chunk_render_line (debug_lines.rs:242); jar: `getStats` is `"pC: %03d, pU: %02d, aB: %02d"`. |
| 510 | `game/dispenser.rs` | ported | **pass** | PORTED verified: spawn-egg arm in `dispense_item` (dispenser.rs:670); no-ENTITY_DATA egg returns the stack unchanged + CLICK (dispense_spawn_egg :1011). |
| 511 | `hud/pack_text.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/hud/pack_text.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 512 | `game/commands/text_commands.rs` | no-change | **pass** | Pass — triage correct: Rust implements the 1.21.11 `NbtContents` form; the separator + per-player gaps are honestly noted as gaps from BOTH versions equally (B11-A03). |
| 513 | `post_chain.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/post_chain.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 514 | `game/beehives.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/beehives.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 515 | `game/commands/place.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/commands/place.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 516 | `mobs/nautilus.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/mobs/nautilus.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 517 | `game/advancements_screen.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/advancements_screen.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 518 | `dialog_screen.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/dialog_screen.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 519 | `game/debug_keys.rs` | ported | **pass** | PORTED verified: `debug_chart_lines` returns `[String; 2]` (debug_keys.rs:254), FPS uppercase (:284); jar: one-line concat template + `FPS + TPS`/`FPS`. |
| 520 | `mobs/golem.rs` | ported | **pass** | PORTED verified: golem draws the 0.7 roll first (golem.rs:152-158); snow tags renamed to `snow_layer_can/cannot_survive_on` (golem.rs:579-582); jar: store-first 0.7 form. |
| 521 | `debug_chart.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/debug_chart.rs); ledger verdict no-change apart from citations; family-pass (named classes jar-checked where extracted). |
| 522 | `game/debug_stick.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/debug_stick.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 523 | `game/social.rs` | ported | **pass** | PORTED verified: `chat_allowed` gate in social.rs:147 + draw/click/buttons gated in social_screen.rs:177/199/343, tests :461/:485; jar: `PlayerEntry` gates on `isChatAllowed` + `isBlocked`. |
| 524 | `mobs/goat.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/mobs/goat.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 525 | `multiplayer.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/multiplayer.rs); ledger verdict no-change apart from citations; family-pass (named classes jar-checked where extracted). |
| 526 | `models/catalog.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/models/catalog.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 527 | `game/using_item.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/using_item.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 528 | `graphics_ui.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/graphics_ui.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 529 | `multiline_edit.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/multiline_edit.rs); ledger verdict no-change apart from citations; family-pass (named classes jar-checked where extracted). |
| 530 | `net.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/net.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 531 | `game/brush.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/brush.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 532 | `realms/client.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/realms/client.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 533 | `generator_decoration.rs` | ported | **pass** | PORTED verified: `Kind::Mushroom` returns true (bonemeal.rs:345-352); `mushroom_grow_block` in survive.rs:285 + block_survival.rs:566; jar: `isValidBonemealTarget` is `iconst_1`, tag `MUSHROOM_GROW_BLOCK`. |
| 534 | `item_render.rs` | no-change | **pass** | Pass — triage correct: layer decision + glint math match; the special-foil decal is honestly noted as a pre-existing gap (B11-A02). |
| 535 | `game/wandering_trader.rs` | ported | **pass** | PORTED verified: `world_surface` (wandering_trader.rs:243), level.dat storage via `load_trader_data` (:57), `wandering_trader.dat` read-only fallback; jar: WORLD_SURFACE at offset 64, `PrimaryLevelData` carries the clock. |
| 536 | `shadows.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/shadows.rs); ledger verdict no-change; family-pass (named classes jar-checked where extracted). |
| 537 | `toasts.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/toasts.rs); ledger verdict no-change apart from citations; family-pass (named classes jar-checked where extracted). |
| 538 | `worldgen/tree/trunk.rs` | ported | **pass** | PORTED verified: `TreeFeature::decode` falls back to `dirt_provider` (tree.rs:540-542); jar: `TrunkPlacer.setDirtAt`/`isDirt` exist, no `below_trunk_provider`. |
| 539 | `gfx/submit.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/gfx/submit.rs); ledger verdict no-change; "`gfx/submit.rs` is a pure native wgpu submit-thread layer (frame `Job`s with a wgpu `CommandEncoder`, ordered ..."; family-pass (named classes jar-checked where extracted). |
| 540 | `game/bundles.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/bundles.rs); ledger verdict no-change; "- The Rust reads the selection from `bundle_selected` directly; it never calls `getSelectedItem(Index)` by nam..."; family-pass (named classes jar-checked where extracted). |
| 541 | `game/item_use.rs` | ported | **pass** | PORTED verified: no passenger check in `release_trident` (trident.rs:501-503, comment notes the 26.1-only check); jar: `releaseUsing` gates only on spinAttack + water/rain. |
| 542 | `snbt.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-core/src/snbt.rs); ledger verdict no-change; "`Pretty` in snbt.rs models the 1.21.11 hardcoded rich form exactly: GOLD numbers, RED suffixes, GREEN strings,..."; family-pass (named classes jar-checked where extracted). |
| 543 | `game/test_instances.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/test_instances.rs); ledger verdict no-change; "- `TestDef` models the 1.21.11 field set (structure, rotation, required, sky_access, ticks, attempts, dimensio..."; family-pass (named classes jar-checked where extracted). |
| 544 | `level/place.rs` | ported | **pass** | PORTED verified: end stone hardcoded (survive.rs:111/143/148, place.rs:1330, shapes.rs, block_survival.rs:607, chorus_plant.rs); jar: `END_STONE` hardcoded, 0 `SUPPORTS_CHORUS`. |
| 545 | `worldgen/structure/mineshaft.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/structure/mineshaft.rs); ledger verdict no-change; "The Rust cites `generateAndAddPiece` by name (shared by both versions); visibility has no Rust surface, and th..."; family-pass (named classes jar-checked where extracted). |
| 546 | `worldgen/structure/stronghold.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/structure/stronghold.rs); ledger verdict no-change; "The Rust cites `generateAndAddPiece` by name (shared by both versions); visibility and the record accessor hav..."; family-pass (named classes jar-checked where extracted). |
| 547 | `vault.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/vault.rs); ledger verdict no-change; "vault.rs cites `DefaultDispenseItemBehavior.spawnItem` and `VaultState` by name, both shared; the random-acces..."; family-pass (named classes jar-checked where extracted). |
| 548 | `bossbar.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-core/src/bossbar.rs); ledger verdict no-change; "bossbar.rs models the `Packed` form with the same 10 fields and defaults, `display_name` with the id hover, an..."; family-pass (named classes jar-checked where extracted). |
| 549 | `gfx/lightning.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/gfx/lightning.rs); ledger verdict no-change; "The only citation is `LightningBoltRenderer.submit`, a name shared by both versions. No test run (no code chan..."; family-pass (named classes jar-checked where extracted). |
| 550 | `packs.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/packs.rs); ledger verdict no-change; "packs.rs cites `BlockStateModelLoader.loadBlockStateDefinitionStack` and `FallbackResourceManager` by name, bo..."; family-pass (named classes jar-checked where extracted). |
| 551 | `game/commands/world_cmds.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/commands/world_cmds.rs); ledger verdict no-change; "world_cmds.rs already cites the 1.21.11 form ("`RideCommand.mount`: `vehicle.getType() == EntityType.PLAYER`")..."; family-pass (named classes jar-checked where extracted). |
| 552 | `game/screen_info.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/screen_info.rs); ledger verdict no-change apart from one citation; "screen_info.rs is the screen-narration module and models no PopupScreen (PopupScreen overrides no narration me..."; family-pass (named classes jar-checked where extracted). |
| 553 | `game/moving_pistons.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/moving_pistons.rs); ledger verdict no-change; "The Rust cites `extractRenderState`, `submit` and `createMovingBlock` — all names shared by both versions. (No..."; family-pass (named classes jar-checked where extracted). |
| 554 | `scoreboard_cmd.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/scoreboard_cmd.rs); ledger verdict no-change; "scoreboard_cmd.rs cites `ComponentArgument.getResolvedComponent` by name (shared); the resolution itself alrea..."; family-pass (named classes jar-checked where extracted). |
| 555 | `worldgen/structure/monument.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/structure/monument.rs); ledger verdict no-change; "The Rust cites only the class names (`OceanMonumentStructure`, `OceanMonumentPieces`), both shared. No test ru..."; family-pass (named classes jar-checked where extracted). |
| 556 | `gamepad.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-input/src/gamepad.rs); ledger verdict no-change; "`focus_step_rect` implements the shared `nextFocusPathInDirection` from the focused rect — the 1.21.11 start p..."; family-pass (named classes jar-checked where extracted). |
| 557 | `gfx/panorama.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/gfx/panorama.rs); ledger verdict no-change; "panorama.rs already encodes the shared values: "CubeMap.render: 85-degree FOV, near/far 0.05/10, rotationX(PI)..."; family-pass (named classes jar-checked where extracted). |
| 558 | `game/cauldrons.rs` | no-change | **pass** | Pass — triage correct: `removes_dye` reads `minecraft:dyeable` (cauldron.rs:8-13), the tag EXISTS in 1.21.11 data with exactly the 6 vanilla items, and the wash strips `dyed_color` (cauldron.rs:217-226). The 1.21.10 run's B11-G16 misread the :190 doc comment — wash works here (see §6.6). |
| 559 | `game/totem.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/totem.rs); ledger verdict no-change; "`item_activation.rs` matches the animation math exactly, and `item_activation_geometry` applies the 1.21.11 ga..."; family-pass (named classes jar-checked where extracted). |
| 560 | `game/fireworks.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/fireworks.rs); ledger verdict no-change; "The Rust cites `FireworkEntityRenderer.submit` by name, which is shared. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 561 | `horizon-command/snbt.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "`IntegerLiteral::create` mirrors the logic exactly (signed/unsigned paths, radix by base, `snbt.parser.expecte..."; SPOT-CHECKED: named vanilla classes jar-verified present. |
| 562 | `level/feature_world.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/level/feature_world.rs); ledger verdict no-change; "The Rust cites `MonsterRoomFeature.place` and BeehiveDecorator's bee placement by name, both shared. No test r..."; SPOT-CHECKED: named vanilla classes jar-verified present. |
| 563 | `game/gamemode_switcher.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/gamemode_switcher.rs); ledger verdict no-change; "The Rust cites only `GameModeSwitcherScreen.mouseReleased` and the class name, both shared. No test run (no co..."; family-pass (named classes jar-checked where extracted). |
| 564 | `game/test_environments.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/test_environments.rs); ledger verdict no-change; "test_environments.rs already documents and implements the 1.21.11 form: "1.21.11: no saved data; `setup` and `..."; family-pass (named classes jar-checked where extracted). |
| 565 | `block_pattern.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/block_pattern.rs); ledger verdict no-change; "The Rust cites `WitherSkullBlock.getOrCreateWitherFull`, `getOrCreateWitherBase` and `checkSpawn` by name, all..."; family-pass (named classes jar-checked where extracted). |
| 566 | `game/native_structures.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/native_structures.rs); ledger verdict no-change; "The Rust already cites `FileUtil.isPathNormalized` — the 1.21.11-only method — and implements it equivalently ..."; family-pass (named classes jar-checked where extracted). |
| 567 | `lan/entity.rs` | no-change | **pass** | Pass — triage correct: `CurrentAccumulator::apply_to` pushes on any nonzero current (entity_fluid.rs:86-90, "the 26.x tracker's 1e-5 squared-length floor is gone"); the 1e-5 constant is absent from 1.21.11's `Entity.class`. Re-base warning: 1.21.7 HAS the floor (§6.4). |
| 568 | `item/hash/gameplay.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-protocol/src/item/hash/gameplay.rs); ledger verdict no-change; "Already handled in the adapter: `Quirks.either_holders_and_tag_resistance` (1.21.11) makes item.rs's `DAMAGE_R..."; family-pass (named classes jar-checked where extracted). |
| 569 | `worldgen/structure/temple.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/structure/temple.rs); ledger verdict no-change; "The Rust cites the shared piece/structure names. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 570 | `block_entities.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/block_entities.rs); ledger verdict no-change; "The Rust cites `RebindableTickingBlockEntityWrapper.rebind` by name. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 571 | `debug_overlay.rs` | ported | **pass** | PORTED verified: Day param (debug_overlay.rs:419), MB labels (:533/:540), two-space Mood (:439), tests pin them; jar: `"(Day %d)"`, `bytesToMegabytes`, `"  (Mood %d%%)"`. |
| 572 | `game/boat_leads.rs` | ported | **pass** | PORTED verified: knot tied + sound before the attach check (leads.rs:646-656); test `a_fence_clicked_with_a_lead_keeps_its_knot_when_nothing_attaches` exists; jar: `getOrCreateKnot` at offsets 58/63 before the check at 69. |
| 573 | `game/command_blocks.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/command_blocks.rs); ledger verdict no-change; "The Rust cites the shared `CommandBlock`/`BaseCommandBlock` names. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 574 | `game/commands/test_runs.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/commands/test_runs.rs); ledger verdict no-change; "The Rust already cites the 1.21.11 name `getStructureBoundingBox` (test_instances.rs). No test run (no code ch..."; family-pass (named classes jar-checked where extracted). |
| 575 | `game/golems.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/golems.rs); ledger verdict no-change; "The Rust cites the shared `trySpawnGolem`/`canSpawnGolem` names. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 576 | `game/lan_chat.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/lan_chat.rs); ledger verdict no-change; "horizon-command's `signable_arguments` already implements the 1.21.11 walk (`child.root != root` filter) and c..."; family-pass (named classes jar-checked where extracted). |
| 577 | `game/pumpkin.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/pumpkin.rs); ledger verdict no-change; "The Rust matches the motion exactly (`vx = 0.05 * dx + r.next_double() * 0.02`, y `0.05`). No test run (no cod..."; family-pass (named classes jar-checked where extracted). |
| 578 | `game/sniffers.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/sniffers.rs); ledger verdict no-change; "The Rust cites the shared `SnifferEggBlock.tick`. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 579 | `title/experiments.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/title/experiments.rs); ledger verdict no-change; "The Rust cites the shared `ExperimentsScreen` names. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 580 | `vanillagen.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/vanillagen.rs); ledger verdict no-change; "The Rust cites `ChunkStatusTasks.full`; the `generateStructures` gate has no Rust surface here. No test run (n..."; family-pass (named classes jar-checked where extracted). |
| 581 | `level/growth_tests.rs` | no-change | **gap** | **gap** — triage claims the Sapling arm "already returns true", but `bonemeal_valid` returns false with no grower / no live features / tree can't grow (bonemeal.rs:331-340); vanilla `isValidBonemealTarget` is `iconst_1` (B11-G12). |
| 582 | `loot/odds.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/loot/odds.rs); ledger verdict no-change; "The Rust cites the shared `ApplyBonusCount`/`MatchTool` names; the formula math has no diff. No test run (no c..."; family-pass (named classes jar-checked where extracted). |
| 583 | `worldgen/cave.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/cave.rs); ledger verdict no-change; "The Rust checks air or (water block and FLUID_FULL), equivalent to both forms. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 584 | `worldgen/structure/nether_fossil.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/structure/nether_fossil.rs); ledger verdict no-change; "The Rust cites the shared structure/piece names. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 585 | `falling_block.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/falling_block.rs); ledger verdict no-change; "The Rust cites the shared `AnvilBlock.damage`. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 586 | `furnace.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/furnace.rs); ledger verdict no-change; "`horizon-world`'s `drop_item_stack_at` matches exactly (floored origin, 0.75 range, split, triangle motion). N..."; family-pass (named classes jar-checked where extracted). |
| 587 | `game/cat_spawner.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/cat_spawner.rs); ledger verdict no-change; "The Rust cites the shared `CatSpawner.tick`. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 588 | `game/commands/raid.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/commands/raid.rs); ledger verdict no-change; "The Rust cites the shared `RaidCommand.playSound`. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 589 | `game/growth.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/growth.rs); ledger verdict no-change; "The Rust cites the shared `ShearsItem.useOn`. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 590 | `game/initial_spawn.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/initial_spawn.rs); ledger verdict no-change; "The Rust cites the shared `BonusChestFeature.place`. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 591 | `game/server_packs.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/server_packs.rs); ledger verdict no-change; "The Rust cites the shared `DownloadedPackSource` names. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 592 | `game/stats.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/stats.rs); ledger verdict no-change; "The Rust cites the shared `PhantomSpawner.tick`. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 593 | `lan/playerdata.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/lan/playerdata.rs); ledger verdict no-change; "playerdata.rs models only the consequence — a Realms download's level.dat has no `Player`, so the owner's play..."; SPOT-CHECKED: named vanilla classes jar-verified present. |
| 594 | `stand_up.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/stand_up.rs); ledger verdict no-change; "The Rust cites the shared `NodeEvaluator.isBurningBlock`. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 595 | `level/shelf.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/level/shelf.rs); ledger verdict no-change; "The Rust cites the shared `SideChainPartBlock.setPart`. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 596 | `game/block_editor.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/block_editor.rs); ledger verdict no-change apart from a citation; "One 26.1-only citation re-pointed per convention: `LABEL`'s doc `Screen.extractRenderState` -> `Screen.render`..."; family-pass (named classes jar-checked where extracted). |
| 597 | `lan/guest.rs` | no-change | **pass** | Pass — triage correct: the `either_holders_and_tag_resistance` adapter quirk exists (adapter/ids.rs:132, adapter.rs:174) and `builtin_component` writes the pinned holder form (guest.rs:979/1356). The string-only trim read is the separate B11-A01 note. |
| 598 | `worldgen/dripstone.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/dripstone.rs); ledger verdict no-change; "The Rust cites the shared feature names and does not cite the renamed accessors. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 599 | `horizon-bots/src/main.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-bots/src/main.rs); ledger verdict no-change; "The Rust cites the shared `ClientChunkCache`. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 600 | `game/allay_pickup_cooldown_tests.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/allay_pickup_cooldown_tests.rs); ledger verdict no-change apart from citations; "`allay_deliver` (game/allays.rs) already implements the 1.21.11 form exactly: criterion for the liked player a..."; family-pass (named classes jar-checked where extracted). |
| 601 | `game/paintings.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/paintings.rs); ledger verdict no-change; "The Rust cites the shared `PaintingRenderer` name and does not cite the renamed light method. No test run (no ..."; family-pass (named classes jar-checked where extracted). |
| 602 | `inventory_effects.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/inventory_effects.rs); ledger verdict no-change; "The Rust cites only the shared `EffectsInInventory`. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 603 | `game/allays.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/allays.rs); ledger verdict no-change; "`allay_deliver` already implements the 1.21.11 form exactly (verified for item 600). No test run (no code chan..."; family-pass (named classes jar-checked where extracted). |
| 604 | `game/dedicated/management.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/dedicated/management.rs); ledger verdict no-change; "The Rust cites only shared class/method names. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 605 | `title/create_world.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/title/create_world.rs); ledger verdict no-change; "The Rust cites only shared names (`CreateWorldScreen`, `SwitchGrid`). No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 606 | `horizon-command/src/packrat.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-command/src/packrat.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 607 | `adapter/translate/v1_21_1/recipes.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-protocol/src/adapter/translate/v1_21_1/recipes.rs); ledger verdict no-change; "This file is the legacy recipe parser for 1.21/1.21.1 servers only (`translate.rs`: it runs only when `version..."; family-pass (named classes jar-checked where extracted). |
| 608 | `game/key_focus.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/key_focus.rs); ledger verdict no-change; "`Nav::Arrow(Button)` carries only the direction, matching the 1.21.11 form; the no-focus case starts from the ..."; family-pass (named classes jar-checked where extracted). |
| 609 | `game/lan_screen.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/lan_screen.rs); ledger verdict no-change; "lan_screen.rs's header already documents this: 1.21.11 has no WorldOptionsScreen/RestrictionsScreen and the in..."; family-pass (named classes jar-checked where extracted). |
| 610 | `server_packs/download.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/server_packs/download.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 611 | `social_screen.rs` | no-change | **pass** | Pass — triage correct: item 523's `chat_allowed` gate is in place at social_screen.rs:177/199/343, as the file states. No staleness. |
| 612 | `commands/arguments` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 613 | `game/chat.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/chat.rs); ledger verdict no-change (deliberate, documented); "The Rust keeps the clock machinery as Horizon's own state, and the port already documents it: `load_world_stat..."; family-pass (named classes jar-checked where extracted). |
| 614 | `game/commands/misc.rs` | ported | **pass** | PORTED verified: per-level `rand_sequences` (`Parked` gains the field, `swap_level` swaps it, game.rs:3139/7599); jar: `ServerLevel.randomSequences` field, 11-arg ctor. |
| 615 | `game/dialogs.rs` | no-change | **pass** | Pass — triage is honest: NO-CHANGE with the known gap recorded (tooltip-on-focus has no surface; dialog focus tracks text inputs only). The gap itself is B11-G15 (cross-cutting). |
| 616 | `game/shelf.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/shelf.rs); ledger verdict no-change; "The Rust reads `facing` from the block state properties at use time (the 1.21.11 form). No test run (no code c..."; family-pass (named classes jar-checked where extracted). |
| 617 | `realms/download.rs` | no-change | **pass** | Pass — triage is honest: NO-CHANGE with the documented rank-2 precedent (engine saves in its own `players/data` layout). The divergence itself is B11-G13 (cross-cutting). |
| 618 | `worldgen/tree/decorator.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/tree/decorator.rs); ledger verdict no-change; "The Rust does not cite the level type. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 619 | `client/gui/components/tabs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/hud_kit/tabs.rs); ledger verdict no-change; "The Rust has no stored tab-bar width: tab layout is a pure function of the current width (`tabs(w)` in create_..."; family-pass (named classes jar-checked where extracted). |
| 620 | `pack_palette.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/pack_palette.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 621 | `game/render_tick.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/render_tick.rs); ledger verdict no-change; "Already handled: the adapter translates 1.21.11's SetTime (game time, day time, ticking) into the pinned clock..."; family-pass (named classes jar-checked where extracted). |
| 622 | `mobs/polar_bear.rs` | ported | **pass** | PORTED verified: fox goal has no selector (polar_bear.rs:54-60, comment notes cubs hunt foxes); jar: fox goal passes `aconst_null` (offsets 194-214). |
| 623 | `level/hooks.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/level/hooks.rs); ledger verdict no-change; "The Rust already cites the 1.21.11 `getLightBlock()` name (tick.rs). No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 624 | `worldgen/iceberg.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/iceberg.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 625 | `worldgen/structure/mansion.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/structure/mansion.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 626 | `client/resources/server` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 627 | `gfx/translucency.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/gfx/translucency.rs); ledger verdict no-change; "The Rust's `update` already implements the POV rule exactly (diagonal views keep their ordering until the POV ..."; family-pass (named classes jar-checked where extracted). |
| 628 | `options_file.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/options_file.rs); ledger verdict no-change; "The Rust's `int_range` (loot/odds.rs) is a pure min/max/distribution computation; it models neither validation..."; family-pass (named classes jar-checked where extracted). |
| 629 | `trims.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/trims.rs); ledger verdict no-change; "Already handled: mobs/render.rs documents (in two places) that "1.21.11 has no `humanoid_baby` layer type" and..."; family-pass (named classes jar-checked where extracted). |
| 630 | `zip.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/zip.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 631 | `game/code_of_conduct.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/code_of_conduct.rs); ledger verdict no-change; "The Rust has no integrated server and no registry filtering — it uses the received registries as-is, the 1.21...."; family-pass (named classes jar-checked where extracted). |
| 632 | `game/mount_menu.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/mount_menu.rs); ledger verdict no-change; "The Rust gates the slots by species, the entity-type form. No test run (no code change)."; SPOT-CHECKED: named vanilla classes jar-verified present. |
| 633 | `game/spectator.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/spectator.rs); ledger verdict no-change; "No test run (no code change)."; SPOT-CHECKED: named vanilla classes jar-verified present. |
| 634 | `game/tutorial.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/tutorial.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 635 | `game/use_hand.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/use_hand.rs); ledger verdict no-change; "Default states come from the prepared 1.21.11 block registry data (`info(block).default`), so the 1.21.11 defa..."; family-pass (named classes jar-checked where extracted). |
| 636 | `horizon-narrator/src/lib.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-narrator/src/lib.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 637 | `port-pack/carry.rs` | no-change | **gap** | **gap** — triage frames the 0-fallthrough as 26.1-only providers, but `score`/`storage`/`enchantment_level` ARE 1.21.11 providers (jar-verified) and yield 0 in loot.rs:1344/1346/1363 + items.rs:1934-1972 (B11-G06, datapack-only: 0/1,329 vanilla tables use them). |
| 638 | `level/redstone/experimental.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/level/redstone/experimental.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 639 | `worldgen/structure/ruined_portal.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/structure/ruined_portal.rs); ledger verdict no-change; "No test run (no code change)."; SPOT-CHECKED: named vanilla classes jar-verified present. |
| 640 | `worldgen/structure/saved_templates.rs` | ported | **pass** | PORTED verified: no `height_adjusted` caching in shipwreck code (simple.rs recomputes every postProcess); jar: `ShipwreckPiece` has only `isBeached`. |
| 641 | `game/chiseled_bookshelf.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/chiseled_bookshelf.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 642 | `game/debug_options.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/debug_options.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 643 | `game/debug_renderers.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/debug_renderers.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 644 | `game/friends.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/friends.rs); ledger verdict no-change; "No citation of the renamed method in the Rust. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 645 | `game/mob_looting.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/mob_looting.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 646 | `server_packs.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/server_packs.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 647 | `title/confirm_experiments.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/title/confirm_experiments.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 648 | `title/flat.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/title/flat.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 649 | `horizon-command/src/tree_extra.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-command/src/tree_extra.rs); ledger verdict no-change; "No test run (no code change)."; SPOT-CHECKED: named vanilla classes jar-verified present. |
| 650 | `launcher/src/java_launch.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "No test run (no code change)."; SPOT-CHECKED: named vanilla classes jar-verified present. |
| 651 | `client/gui/components/toasts` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/toasts.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 652 | `client/renderer/chunk` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "Blaze3D section-mesh plumbing the native wgpu renderer never mirrors (as item 539). No test run (no code chang..."; family-pass (named classes jar-checked where extracted). |
| 653 | `server/network/config` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 654 | `world/level/levelgen/structure/pools` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 655 | `horizon-client-input/src/clipboard.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-input/src/clipboard.rs); ledger verdict no-change; "The Rust's clipboard goes through winit, never GLFW. No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 656 | `atlas_sources.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/atlas_sources.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 657 | `effects.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/effects.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 658 | `screen_effects.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/screen_effects.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 659 | `extras_hud.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/extras_hud.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 660 | `game/item_particles.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/item_particles.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 661 | `game/native_criteria.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/native_criteria.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 662 | `game/native_panel_recipes.rs` | no-change | **gap** | **gap** — triage claims "the fuel SET is the same", but no 1.21.11 item carries `minecraft:cooking_fuel` (0/1,505 in datagen items.json; `DataComponents` has no COOKING_FUEL), so the fuel list is EMPTY, not merely misordered (B11-G07). |
| 663 | `game/saving.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/saving.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 664 | `game/zombie_villagers.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/zombie_villagers.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 665 | `lan/arrivals.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/lan/arrivals.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 666 | `mobs/hostile.rs` | ported | **pass** | PORTED verified: `Swell::can_use`/`tick` have no alive check (hostile.rs:286-319; the `!m.alive()` at :331/:342 is the fuse-tick); jar: `SwellGoal` has no `isDeadOrDying`. |
| 667 | `horizon-protocol/src/frame.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-protocol/src/frame.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 668 | `horizon-server/src/management_methods.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-server/src/management_methods.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 669 | `worldgen/structure.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/structure.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 670 | `worldgen/structure/fortress.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/structure/fortress.rs); ledger verdict no-change; "No test run (no code change)."; SPOT-CHECKED: named vanilla classes jar-verified present. |
| 671 | `launcher/src/screenshots.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 672 | `client/gui/layouts` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 673 | `client/gui/screens/multiplayer` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/multiplayer.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 674 | `server/commands/data` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 675 | `horizon-client-core/src/lang.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-core/src/lang.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 676 | `horizon-client-core/src/text.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-core/src/text.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 677 | `horizon-client-render/src/core_shader.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/core_shader.rs); ledger verdict no-change; "No test run (no code change)."; SPOT-CHECKED: named vanilla classes jar-verified present. |
| 678 | `horizon-client-render/src/gfx/core_lines.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/gfx/core_lines.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 679 | `horizon-client-render/src/gfx/end_sky/tests.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/gfx/end_sky/tests.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 680 | `horizon-client-render/src/gfx/frame_gen.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/gfx/frame_gen.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 681 | `horizon-client-render/src/gfx/pack_atlas/stitch.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/gfx/pack_atlas/stitch.rs); ledger verdict no-change; "No test run (no code change)."; SPOT-CHECKED: named vanilla classes jar-verified present. |
| 682 | `horizon-client-render/src/sound_preview.rs` | no-change | **pass** | Pass — triage correct: `sound_variants` lists exactly Wolf (finalize.rs:2378-2383); only `WolfSoundVariants` exists in the 1.21.11 jars. The 1.21.7 B11-A02 note does not apply (B11-G34 FIXED). |
| 683 | `cauldron.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/cauldron.rs); ledger verdict no-change; "Already handled: cauldron.rs:214 records that 1.21.11 dropped the 26.1 Dispatcher, and item 558 established th..."; family-pass (named classes jar-checked where extracted). |
| 684 | `game/clone_dimensions_tests.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/clone_dimensions_tests.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 685 | `game/combat.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/combat.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 686 | `game/command_minecarts.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/command_minecarts.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 687 | `game/commands/execute.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/commands/execute.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 688 | `game/crossbow.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/crossbow.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 689 | `game/enderman.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/enderman.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 690 | `game/interactions.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/interactions.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 691 | `game/item_entities.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/item_entities.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 692 | `game/lan_crafting.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/lan_crafting.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 693 | `game/launch_options.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/launch_options.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 694 | `game/levels.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/levels.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 695 | `game/mods.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/mods.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 696 | `game/native_client_entities.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/native_client_entities.rs); ledger verdict no-change; "The Rust has no entity-representation plumbing at all (no `setEntityRepresentation` anywhere) and builds the d..."; family-pass (named classes jar-checked where extracted). |
| 697 | `game/pad_menu.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/pad_menu.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 698 | `game/respawn_anchor.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/respawn_anchor.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 699 | `game/splash_water.rs` | no-change | **gap** | **gap** — triage claims "no `PotionTags` usage anywhere in the Rust", but `potion_in_tag` (splash_water.rs:42-51) reads the potion-tags map, which is empty (0 potion tag files, no `PotionTags` class); vanilla gates on `potionContents.is(Potions.WATER)` (B11-G05, major). |
| 700 | `game/trident.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/trident.rs); ledger verdict no-change; "No `BlockAndLightGetter` usage anywhere in the Rust."; family-pass (named classes jar-checked where extracted). |
| 701 | `menu/equippable.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/menu/equippable.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 702 | `mobs/bat.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/mobs/bat.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 703 | `mobs/caravan.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/mobs/caravan.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 704 | `mobs/doors.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/mobs/doors.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 705 | `mobs/farm.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/mobs/farm.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 706 | `mobs/nether.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/mobs/nether.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 707 | `mobs/skin.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/mobs/skin.rs); ledger verdict no-change; "Already handled: body_anim.rs documents that one `TurtleModel` serves both ages in 1.21.11 (26.1 split it into..."; family-pass (named classes jar-checked where extracted). |
| 708 | `mobs/trader.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/mobs/trader.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 709 | `title/buffet.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/title/buffet.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 710 | `horizon-command/src/tests.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-command/src/tests.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 711 | `horizon-command/src/tree_execute.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-command/src/tree_execute.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 712 | `horizon-port-pack/src/oracle.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-port-pack/src/oracle.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 713 | `horizon-prepare/src/artifacts.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-prepare/src/artifacts.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 714 | `horizon-prepare/src/inventory.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-prepare/src/inventory.rs); ledger verdict no-change; "Only a doc comment mentions `SoundBufferLibrary` (audio.rs:374); nothing references the debug counter."; family-pass (named classes jar-checked where extracted). |
| 715 | `horizon-protocol/src/item/skip.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-protocol/src/item/skip.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 716 | `horizon-server-files/src/properties.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-server-files/src/properties.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 717 | `horizon-server-files/src/user_cache.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-server-files/src/user_cache.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 718 | `horizon-server/src/management.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-server/src/management.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 719 | `horizon-server/src/query.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-server/src/query.rs); ledger verdict no-change; "No test run (no code change)."; SPOT-CHECKED: named vanilla classes jar-verified present. |
| 720 | `horizon-world/src/datafix/dynamic.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/datafix/dynamic.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 721 | `horizon-world/src/datafix/progress.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/datafix/progress.rs); ledger verdict no-change; "Already handled: progress.rs:1-5 documents that 1.21.11 has no file-fixing flow and the `FileFixStats` half is..."; family-pass (named classes jar-checked where extracted). |
| 722 | `horizon-world/src/level/orientation.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/level/orientation.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 723 | `horizon-world/src/level/tests.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/level/tests.rs); ledger verdict no-change; "No test run (no code change)."; SPOT-CHECKED: named vanilla classes jar-verified present. |
| 724 | `worldgen/feature_catalog.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/feature_catalog.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 725 | `worldgen/generator.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/generator.rs); ledger verdict no-change; "No test run (no code change)."; SPOT-CHECKED: named vanilla classes jar-verified present. |
| 726 | `worldgen/snow_freeze.rs` | ported | **pass** | PORTED verified: 7 dual names in classes.rs (FarmBlock, SnowyDirtBlock, SpreadingSnowyDirtBlock, FungusBlock, RootsBlock, DirtPathBlock, RedStoneWireBlock); AbstractBed/Speleothem arms paired with live twins (§6.1). |
| 727 | `worldgen/structure/formations.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/structure/formations.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 728 | `worldgen/structure/piece_limited.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/structure/piece_limited.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 729 | `worldgen/structure/simple.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/structure/simple.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 730 | `worldgen/structure/start.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/structure/start.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 731 | `launcher/src/wardrobe.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "No `GLFWErrorCapture` usage anywhere in the Rust (the engine uses winit, not GLFW — as item 655)."; family-pass (named classes jar-checked where extracted). |
| 732 | `client/gui/screens/advancements` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 733 | `client/gui/screens/options` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "No `exclusiveFullscreen` anywhere in the Rust — the engine never had the 26.1-only option."; family-pass (named classes jar-checked where extracted). |
| 734 | `client/renderer/block/model` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "The Rust cites `FaceBakery` itself (present in both versions); nothing references the `$1` anonymous class."; family-pass (named classes jar-checked where extracted). |
| 735 | `core/cauldron` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/cauldron.rs); ledger verdict no-change; "No `CauldronInteractions` usage anywhere in the Rust; cauldron.rs is a complete port of the 1.21.11 table (ite..."; family-pass (named classes jar-checked where extracted). |
| 736 | `core/component/predicates` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 737 | `gametest/framework` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 738 | `recipebook` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "The Rust already cites the 1.21.11 form (`Item.getDefaultMaxStackSize`, native_hooks.rs:10728)."; family-pass (named classes jar-checked where extracted). |
| 739 | `world/entity/vehicle/minecart` | no-change | **pass** | Module resolves in the 1.21.11 tree (launcher/java-package); ledger verdict no-change; "No test run (no code change)."; SPOT-CHECKED: named vanilla classes jar-verified present. |
| 740 | `horizon-client-render/src/gfx/entity_skin.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/gfx/entity_skin.rs); ledger verdict no-change; "No test run (no code change)."; SPOT-CHECKED: named vanilla classes jar-verified present. |
| 741 | `horizon-client-render/src/gfx/gpu_culling.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client-render/src/gfx/gpu_culling.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 742 | `game/packs.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/packs.rs); ledger verdict no-change; "No test run (no code change)."; SPOT-CHECKED: named vanilla classes jar-verified present. |
| 743 | `game/pickups.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/game/pickups.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 744 | `lan/commands.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-client/src/lan/commands.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 745 | `horizon-port-pack/src/upgrade/density.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-port-pack/src/upgrade/density.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 746 | `horizon-port-pack/src/upgrade/settings.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-port-pack/src/upgrade/settings.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 747 | `horizon-world/src/level/ticks.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/level/ticks.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 748 | `worldgen/climate/index.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/climate/index.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
| 749 | `worldgen/structure/ocean_ruin.rs` | no-change | **pass** | Module resolves in the 1.21.11 tree (crates/horizon-world/src/worldgen/structure/ocean_ruin.rs); ledger verdict no-change; "No test run (no code change)."; family-pass (named classes jar-checked where extracted). |
