# Batch 11 — Rank-3 triage (7 items), 26.1.2 parity sweep

- **Batch:** 11 (rank-3 triage) — checks + triage only, no fixes.
- **Date:** 2026-11-25
- **Scope:** 26.1.2's OWN 7 port-ledger items
  (`versions/26.1.2/work-items.json`, all closed; ranks 3/2/1/0/1 —
  items 1–3 rank 1, items 4–5 rank 2, item 6 rank 3, item 7 rank 5),
  triaged against the named 26.1.2 jars and the 26.1.2 port in
  `versions/26.1.2/crates/` (branch `parity-sweep-26.1.2`). 26.1.2 is
  a patch delta on 26.1.1, so there is NO `rank3-progress.json` and no
  port triage workdir for this version. Vanilla 26.1.2's own jar/data
  is the spec throughout; the 26.3 base's form was never trusted.
  **Scope difference from the 1.21.7 Batch 11:** that batch triaged
  the 1.21.11 TEMPLATE's 247 rank-3 items FOR 1.21.7; like the 26.2
  and 26.1.1 runs, this batch verifies 26.1.2's own port ledger —
  i.e. each "closed" verdict is checked to actually hold in the
  current tree.
- **Ledger slice:** `slice.json` (same shape as the 26.1.1 batch-11 slice).

## Tally (7 items)

| Verdict | Count |
|---|---|
| pass | 7 |
| gap (defect attached) | 0 |
| needs-new-harness | 0 |
| needs-in-game-probe | 0 |
| accepted gap (ledger verdict) | 0 |

As-tallied 7 = 2 behaviour-change items (items 4, 6) + 5 plumbing
items (items 1–3, 5, 7); 0 oracle/harness/driver-status checks
tallied. The class-table sweep (§6.1) is reported with its own counts
(live/total, jar + chain), NOT folded into the tally.

**On the brief's needs-harness expectation:** the brief anticipated
that items 4's and 6's regression tests could not run (the
horizon-client lib-test breakage). That premise does not hold for
26.1.2: the behaviour port's enabling work fixed the lib-test
compile breakage in test code, and crate-tests run 45150986405
(head 81311fcc) ran BOTH regression tests green — verified from the
run log, not from PORT-NOTES alone:
`dialog_screen::tests::a_checkbox_label_over_two_rows_gets_the_tooltip
... ok` and
`lan::hub::tests::a_spectators_attack_reaches_nothing_and_resets_no_timer
... ok`, with horizon-client 1274 passed / 0 failed. The run's 41
failures (horizon-client-lib 36, horizon-world 2, horizon-client-render
3) are in untouched code. Items 4 and 6 are therefore **pass**, not
needs-harness. (Three of those 41 failures are the save-upgrade tests
that corroborate B11-G10 — see §1.)

Defects found: **11** (`B11-G01`–`B11-G11`), of which 4 major, 7 minor.
Nine reproduce the 26.1.1 run's defects (G01–G09); two are NEW vs
26.1.1 (G10, the chunk-palette reader — a regression introduced by
this delta; G11, the dropped 26.1.1 item-5 fix). All eleven are
cross-cutting (class-table sweep + 1.21.7-defect cross-checks + the
26.1.1-tree diff + the chunk.rs flag) with no single owning ledger
item — they are recorded as defects, not item verdicts, so the defect
count (11) exceeds the gap-item count (0). Oracle defects: **2**
(`B11-O01`–`B11-O02`, both minor, both cauldron oracles). Needs-harness:
**0** — the 26.1.1 run's B11-H01 is WITHDRAWN (its premise changed;
see §3), and no ledger item needs a harness to settle. Accepted-gap
notes carried from the 1.21.7 sweep: **3** (`B11-A01`–`B11-A03`, §5 —
not ledger verdicts, not counted in the 7-item tally). No probe items:
the 1.21.7 P01 probe is settled by the 26.1.2 jar (§6.6), and the 26.2
run's B11-H01 (chain-liveness half) stays CLOSED — the chain half ran
again here (§6.1).

**Overlap flags for the coordinator (do not fold — merge notes only):**
six of the eleven defects are also minted by the batch that owns the
system, with the same root cause and evidence — B11-G02 ↔ B2-G01
(wire), B11-G05 ↔ B1-G03 (splash water), B11-G03 ↔ B5-G03 (paths),
B11-G04 ↔ B5-G13 (dripstone), B11-G01 ↔ B5-G27 (partial: B5 owns the
bed arm's internal pair-check logic; B11 owns the dead dispatch) and
↔ B10-G03 (bed rendering, a different surface), B11-G07 ↔ B4-G01
(partial: B4 owns the empty-fuel-list/furnace consequence; B11 owns
the recipe-book order). Both oracle defects are also minted elsewhere:
B11-O01 ↔ B1-O02 + B5-O01, B11-O02 ↔ B1-O07 (the coordinator dedups
oracles at merge, as in 26.1.1). The palette defect (B11-G10) was
flagged but explicitly NOT minted by Batches 5 and 6 ("flagged here,
not minted"; "not minted") — B11 mints it. The chat-helper removal
(B11-G11) was noted but not minted by Batch 10. The sculk-inhibitor
tag read (§6.2) is Batch 4's B4-G37, cross-referenced here only.

**Primary re-checks — the 26.1.1 run's 15 items, each decided:**

| 26.1.1 item | Verdict for 26.1.2 | 26.1.2 item |
|---|---|---|
| B11-G01 beds | REPRODUCES | B11-G01 |
| B11-G02 redstone wire | REPRODUCES | B11-G02 |
| B11-G03 dirt paths | REPRODUCES | B11-G03 |
| B11-G04 dripstone | REPRODUCES | B11-G04 |
| B11-G05 splash water | REPRODUCES | B11-G05 |
| B11-G06 number providers | REPRODUCES | B11-G06 |
| B11-G07 recipe-book fuel | REPRODUCES | B11-G07 |
| B11-G08 F3 fps casing | REPRODUCES | B11-G08 |
| B11-G09 F3 spawn-counts | REPRODUCES | B11-G09 |
| B11-O01 VanillaCauldronOracle | REPRODUCES | B11-O01 |
| B11-O02 VanillaLavaCauldronItemOracle | REPRODUCES | B11-O02 |
| B11-H01 item-5 regression test | WITHDRAWN (premise changed — fix + test removed; loss is B11-G11) | — |
| B11-A01 trim inline form | CARRIES (accepted-gap) | B11-A01 |
| B11-A02 special-foil decal | CARRIES (accepted-gap) | B11-A02 |
| B11-A03 NBT separator + per-player | CARRIES (accepted-gap) | B11-A03 |

Headline: the 26.1.2 port's 7-item ledger holds up — all seven
verdicts re-verify against the 26.1.2 jar and the current tree, and
both behaviour items' regression tests ran green in CI. But the delta
itself introduced a NEW save-loading regression: the chunk palette
reader now accepts only the post-5006 `id`/`properties` form, while
vanilla 26.1.2 still writes `Name`/`Properties` palettes — so a
vanilla-written 26.1.2 world loads as void (B11-G10, major). The delta
also dropped 26.1.1's one behaviour fix (B11-G11, minor, uncalled
code). The class table is in exactly the shape 26.1.1's was in —
263/271 live, the same 8 dead mappings, no dual names — and the
26.1.1 tree's inheritance failures (bed pair-check form, component
reads for `cooking_fuel` etc., 26.2+ F3 abbreviations) all carry
forward unchanged: every defect-bearing Rust file in this batch's
scope is byte-identical to 26.1.1's. Of the 23 defects the 1.21.7
Batch 11 found, **seven** carry over to 26.1.2 (G01, G02, G03-paths-half,
G06, G12, G13, G15) plus one cosmetic detail from P01 — the same seven
as 26.1 and 26.1.1.

## §1 Defects

### B11-G01 — Beds place foot-only; sleep is unreachable (major; = 1.21.7 B11-G01, = 26.2 B11-G01, = 26.1 B11-G01, = 26.1.1 B11-G01; REPRODUCES)

`Class::AbstractBed = "AbstractBedBlock"` (classes.rs:59) never matches
any 26.1.2 chain: 26.1.2 has no `AbstractBedBlock` — beds are `BedBlock`
directly (jar-verified: `BedBlock extends HorizontalDirectionalBlock`,
implements `EntityBlock`; no `AbstractBedBlock` in either 26.1.2 jar;
`BlockEntityType.BED` exists and covers all 16 bed blocks, whose
prepared chains are `[BedBlock, HorizontalDirectionalBlock, Block]`).
`Class::Bed = "BedBlock"` (classes.rs:60) exists but NOTHING dispatches
on it (grep-confirmed across all crates — 0 uses outside classes.rs).
Every bed arm is dead:

- `place.rs:356` (placement check), `place.rs:1481` `set_placed_by` — the
  ONLY place a `"head"` part is ever set (place.rs:1483, grep-confirmed)
  — so a player-placed bed is a foot-only single block.
- `interact.rs:365` — the ONLY emitter of `PlayerEffect::Sleep`
  (interact.rs:381) — so clicking ANY bed (including village beds)
  does nothing; `try_sleep` never fires.
- `hooks.rs:459` (creative break), `shapes.rs:124` (neighbour update).

`BlockData::is` checks the bitset built from `Class::from_java` over the
chain (data.rs:215, :922), so no fallback exists. Severity major:
sleep/spawn is core gameplay. **26.3-leak:** the 26.3 client jar
CONTAINS `AbstractBedBlock` (26.3's `BedBlock extends
AbstractBedBlock`), so the mapping is 26.3-correct and 26.1.2-wrong.
**Still missing vs 26.1:** the 26.1 tree's item-584 arm-logic port is
absent here too — the 26.1.2 tree's interact.rs:365-384 arm is the 26.3
pair-check form (checks the other half is the same bed with the
opposite part, `facing.opposite()` for a head click), not 26.1's
foot→head redirect, so even a dispatch fix alone would leave the arm
implementing the wrong form; the fix must restore the 26.1 redirect
too. The arm-logic layer is also recorded as B5-G27 (Batch 5); this
item owns the dead dispatch. (Line-number correction vs the 26.1.1
report: the Sleep emitter is interact.rs:381, not :377/380 — the file
is byte-identical to 26.1.1's.)

### B11-G02 — Redstone wire never updates power (major; = 1.21.7 B11-G02; = 26.1 B11-G02; = 26.1.1 B11-G02; REPRODUCES; overlaps B2-G01)

26.1.2 names the class `RedStoneWireBlock` (capital S, jar-verified in
both jars — the class file is byte-identical to 26.1.1's and 26.1's;
the prepared chain for `minecraft:redstone_wire` is `[RedStoneWireBlock,
Block]`). The table maps `RedstoneWireBlock` only (classes.rs:224) —
the 26.3 name, absent from both 26.1.2 jars — and the 26.1.2 tree
carries NO dual name (26.2's tree maps `"RedstoneWireBlock" |
"RedStoneWireBlock"`). All ten wire arms are dead: hooks.rs:100/211/380
(neighbour changed / notify / piston move), interact.rs:263 (`wire_use`,
the dot/cross toggle), place.rs:1089 (placement connection state),
redstone.rs:65/101 (`wire_signal` dispatch), redstone.rs:219
(`should_wire_connect_to`), shapes.rs:698 (`update_shape` →
`redstone_wire_update_shape`), survive.rs:281 (`canSurvive`).
Decisively, `wire_update_power` has exactly THREE callers — all inside
the dead hooks.rs arms (:105, :213, :385) — so a wire's power is never
recomputed after placement: it keeps power 0 forever. The name-based
reads (`direct_signal` redstone.rs:205, `is_signal_source` :36,
`wire_power_of` :415) only ever read the stale power. Also dead in
prepare: horizon-prepare's survival.rs:43 maps `("RedstoneWireBlock",
"RedstoneWire")`, which misses 26.1.2's `RedStoneWireBlock` chain name
(same for `PathBlock` :10 and `SpeleothemBlock` :21 in that table).
Severity major: redstone is core gameplay. **26.3-leak:**
`RedstoneWireBlock` exists in the 26.3 jar. The 26.1 tree's
survive.rs:282 comment cited the correct 26.1 name
(`RedStoneWireBlock.canSurvive`); the 26.1.2 tree's comment at the
same line cites the 26.3 name (`RedstoneWireBlock.canSurvive`) — and
interact.rs:492's doc likewise carries the 26.3 name. Comment-only,
but it marks the tree's provenance. Batch 2 mints the same defect as
B2-G01 from the redstone side.

### B11-G03 — Dirt paths never revert (minor; = the paths half of 1.21.7 B11-G03; = 26.1 B11-G03; = 26.1.1 B11-G03; REPRODUCES; overlaps B5-G03)

26.1.2 has `DirtPathBlock` (jar-verified; chain `[DirtPathBlock,
Block]`); the table maps `PathBlock` only (classes.rs:208) — the 26.3
name, absent from both 26.1.2 jars, no dual name. All four Path arms
are dead: place.rs:929 (placement state), shapes.rs:465 (updateShape —
shared with the live Farmland arm), survive.rs:273 (`canSurvive`: solid
above kills the path), tick.rs:202 (random tick →
`turn_to_base_block`). `turn_to_base_block` (tick.rs:501) has only
three callers — tick.rs:199 (Farmland, live), :202 (Path, dead), :1046
(farmland moisture, live) — so no name-based path revert exists: a
covered or trampled dirt path never turns back to dirt. The farmland
half of 1.21.7's B11-G03 does NOT reproduce: `FarmlandBlock` is live
in 26.1.2 (chain `[FarmlandBlock, Block]`, classes.rs:151) and its arms
work. Batch 5 mints the same defect as B5-G03.

### B11-G04 — Dripstone placement/survival/growth dead (minor; = 1.21.7 B11-G06; = 26.1 B11-G04; = 26.1.1 B11-G04; REPRODUCES; overlaps B5-G13)

26.1.2 has no `SpeleothemBlock` at all (absent from both jars;
speleothems are a 26.2 addition — present in both 26.2 jars). Pointed
dripstone is `PointedDripstoneBlock` directly (chain
`[PointedDripstoneBlock, Block]`). The table maps `Speleothem =
"SpeleothemBlock"` (classes.rs:215) and all five Speleothem arms are
dead: place.rs:1156 (placement state), shapes.rs:728 (updateShape),
survive.rs:307 (`canSurvive`), tick.rs:248 (scheduled tick —
destroy/fall), tick.rs:434 (random tick → `speleothem_random_tick`,
growth + cauldron-fill). There is NO `Class::PointedDripstone` arm at
any of those five dispatch sites — `PointedDripstone` (classes.rs:216,
live) is used only inside speleothem.rs helpers (:86, :103, :206),
which are themselves reached only via the dead Speleothem arms. Second
independent cause, as in 1.21.7: the `minecraft:speleothems` TAG is
absent from 26.1.2 data (26.2 and 26.3 have it), so even the tag-gated
helpers (`speleothem_tag`, `speleothem_with`, `speleothem_is_tip` in
speleothem.rs:14-31) return false for every block. In 26.2 the arms
were live because `SpeleothemBlock` is `PointedDripstoneBlock`'s
abstract parent there; in 26.1.2 neither the class nor the tag exists.
Batch 5 mints the same defect as B5-G13 (major there).

### B11-G05 — Splash water never douses fire, hurts, extinguishes, or rehydrates (major; = 1.21.7 B11-G13; = 26.1 B11-G05; = 26.1.1 B11-G05; REPRODUCES; overlaps B1-G03)

Vanilla 26.1.2 `AbstractThrownPotion.onHitBlock` gates on
`potionContents.is(Potions.WATER)` — direct holder equality, NO tag
(bytecode 55-63, javap-verified on the 26.1.2 server jar;
`net.minecraft.world.entity.projectile.throwableitemprojectile.AbstractThrownPotion`),
and `onHitEntity` routes water through `onHitAsWater` the same way.
The Rust gates all four behaviours on potion TAGS (game/splash_water.rs:
`potion_in_tag`, :42-51; douse :60, hurt/extinguish :134-135,
rehydrate :197), but 26.1.2 has NO potion tags beyond `tradeable`:
`PotionTags` has ONLY `TRADEABLE` (javap-verified), the client-jar
datapack holds only `tags/potion/tradeable.json`, and the prepared
data's `tags/potion/` likewise — so the `potion_tags` map
(merchant.rs:464, :574) never contains `douses_fire`,
`hurts_water_sensitive_entities`, `extinguishes_entities`, or
`rehydrates_axolotls`, and every gate is always false. The douse area
matches vanilla exactly; only the gate diverges. **26.3-leak:** 26.3's
datapack HAS all four tag files (verified in the 26.3 client jar), so
the tag gate is 26.3-correct and 26.1.2-wrong. (26.2's tree was fixed
to `Potions.WATER` equality; the 26.1.2 tree still carries the tag form
— splash_water.rs is 0-diff vs the 26.1.1 tree.) Severity major:
splash water is the fire-dousing mechanic. Batch 1 mints the same
defect as B1-G03.

### B11-G06 — Three number providers return 0 (minor, datapack-only; = 1.21.7 B11-G12, = 26.2 B11-G02, = 26.1 B11-G06, = 26.1.1 B11-G06; REPRODUCES)

Vanilla 26.1.2 has 6 number providers (`EnchantmentLevelProvider`,
`ScoreboardValue`, `StorageValue` all present in the server jar). Both
Rust implementations handle only 3 (`minecraft:constant`,
`minecraft:uniform`, `minecraft:binomial`) — `minecraft:enchantment_level`,
`minecraft:score`, `minecraft:storage` fall to `_ => 0` (horizon-world
loot.rs `int`/`float` :1344/:1346 and :1363/:1365; horizon-items
items.rs `number` 1903-1936, `_ => 0.0` at :1930/:1932 — the `number`
fn body is byte-identical to the 26.1.1 tree's). Data check: 0/1,326
prepared vanilla 26.1.2 loot tables use the 3 missing providers (0/1,331
client-jar loot-table files likewise) — vanilla loot is unaffected.
The only `enchantment_level` uses in 26.1.2's datapack are the `chance`
providers in `enchantment/soul_speed.json` and
`enchantment/thorns.json`, and those go through the SEPARATE enchant.rs
`random_chance` path, which handles `enchantment_level` explicitly
(enchant.rs:881: `level_value(amount, level)`) — verified against
vanilla 26.1.2's `EnchantmentLevelProvider.getFloat`, which is
`amount.calculate(level)` with NO multiplication by level, exactly
what the Rust computes. So the gap is datapack-only: datapacks using
the three providers in LOOT get 0.

### B11-G07 — Recipe-book fuel slot shows no fuel; the order it would cycle is wrong (minor; = 1.21.7 B11-G15, = 26.2 B11-G03, = 26.1 B11-G07, = 26.1.1 B11-G07; REPRODUCES in a worse form; overlaps B4-G01)

Vanilla 26.1.2 `SlotDisplay$AnyFuel.resolve` returns
`FuelValues.fuelItems()` — the keySet of an insertion-ordered map
(javap-verified on the 26.1.2 client jar: `resolve` reads the
`FUEL_VALUES` context key and streams `fuelItems()`; `fuelItems()`
returns the keySet of the `Object2IntSortedMap` populated by the
HARDCODED `FuelValues.vanillaBurnTimes` sequence, LAVA_BUCKET first at
20000 ticks). The 26.1.2 Rust groups fuels by COOKING-FUEL COMPONENT
value in registry order (recipe_book.rs:566-585,
native_panel_recipes.rs:168-190, via `item.fuel` read from the
`minecraft:cooking_fuel` component at items.rs:1010) — but 26.1.2
vanilla has NO `minecraft:cooking_fuel` component at all
(`DataComponents` in the 26.1.2 jar has no `COOKING_FUEL` field; 26.3's
has it, and 26.3's `AnyFuel.resolve` uses
`DataComponentLookup.findAll(COOKING_FUEL)` — javap-verified on the
26.3 jar), and the prepared data carries it for 0 of 1,506 items, so
the fuel list is EMPTY and the ghost fuel slot is dropped
(`GhostSlots.setSlot` keeps only slots with something to show). Two
layers, both wrong: the mechanism reads a component that does not
exist in this version, and the order it would produce (component
groups in registry order — oak planks protocol id 36 first, lava
bucket 1015) is not vanilla's insertion-ordered FuelValues sequence
(lava bucket first). **Worse than 26.1:** the 26.1 tree grouped by
burn-time VALUE via the hardcoded `burn_time()` table (items.rs:743)
— wrong order, but a non-empty list; the 26.1.1 tree dropped
`burn_time()` entirely and reads the absent component, and the 26.1.2
tree is identical. The empty-list/furnace consequence is Batch 4's
B4-G01 (major — furnaces never light, since `providers.burn_time`
also reads the absent component and returns 0); this item owns the
recipe-book surface and the order. Fixing B4-G01 by restoring the
FuelValues table fixes this item's order too.

### B11-G08 — F3 chart footer uses lowercase fps (minor, cosmetic; part of 1.21.7 B11-P01, settled by the jar; = 26.1 B11-G08, = 26.1.1 B11-G08; REPRODUCES)

Vanilla 26.1.2's `DebugScreenOverlay` builds the chart footer as three
list entries (javap-verified) with the chart names `FPS + TPS` / `FPS`
in UPPERCASE (string constants in the class at offsets 620/626; the
footer template is `Debug charts: \x01; \x01;`). The Rust's
`debug_chart_lines` (game/debug_keys.rs:253-295) matches the three-line
layout and the single-player variants exactly, but names the chart
`fps` / `fps + tps` in lowercase (:283). Only the fps chart name
differs in case (Profiler, Bandwidth + Ping, Ping, Lightmap all
match). Cosmetic. The layout question the 1.21.7 probe item left open
is settled: no in-game probe is needed — the jar gives the footer
outright.

### B11-G09 — F3 spawn-counts line uses 26.2+ category abbreviations (minor, cosmetic; NEW vs 26.1, carried from 26.1.1)

Vanilla 26.1.2's `DebugEntrySpawnCounts` renders each mob category as
the FIRST LETTER of its name, uppercased
(`Character.toUpperCase(MobCategory.getName().charAt(0))`,
javap-verified on the 26.1.2 client jar) — so monster and misc both
read M, ambient and axolotls both A, water creature and water ambient
both W: `SC: 289, M: 3, C: 10, A: 1, A: 0, U: 0, W: 2, W: 4, M: 0`.
26.1.2 has no `MobCategory.getDebugAbbreviation` (a 26.2 addition —
absent from the 26.1.2 server jar, present in the 26.2 server jar
with its `debugAbbreviation` field). The Rust uses the 26.2+
two/three-letter abbreviations (debug_overlay.rs:392:
`["MO", "C", "AM", "AX", "UWC", "WC", "WA", "MI"]`, rendered at :402),
and its own test pins the wrong form (:640:
`"SC: 289, MO: 3, C: 10, AM: 1, AX: 0, UWC: 0, WC: 2, WA: 4, MI: 0"`).
Cosmetic — the counts themselves are right, only the labels differ. A
re-base must not propagate the 26.2+ abbreviations down (§6.4).

### B11-G10 — Vanilla-written worlds load as void: the chunk palette reader accepts only the post-5006 `id` form (major, save-loading; NEW vs 26.1.1)

**Scope: save-loading only.** The 26.1.2 delta REMOVED
`BlockState::from_palette_nbt` from `crates/horizon-world/src/chunk.rs`
(0 hits in 26.1.2's chunk.rs vs 6 in 26.1.1's) and replaced all three
call sites with the strict `BlockState::from_nbt`, which requires the
`id`/`properties` form (chunk.rs:208-213: a compound without `id`
returns `Err("block state is missing id")`; bare strings still decode).
The three call sites: `ChunkData::block` (:341), `ChunkData::set_block`
(:359), and the section decoder `BlockSection::from_nbt` (:442, which
maps over the whole palette with `?`). The 26.1.1 tree's
`from_palette_nbt` accepted the `Name`/`Properties` form for saved
palettes, with a comment recording that "26.1.1 saves still write
palettes as `Name`/`Properties` (the rename to `id` is a later fix, key
5006)" — and its test `a_palette_entry_reads_its_old_name_and_properties`
pinned the equivalence. Both are gone in 26.1.2.

Vanilla 26.1.2 still writes `Name`/`Properties` palettes — verified in
the reference world (`vanilla-world-mc` "26.1.2", seed 4656297851632877335
confirmed from `world_gen_settings.dat`, DataVersion 4790): every
sampled palette entry is a `Name`/`Properties` compound (e.g.
`{'Name': 'minecraft:deepslate', 'Properties': {'axis': 'y'}}`). The
rename to `id` is datafix key 5006 (fixes_c.rs:155-159: renames `Name`→
`id`, `Properties`→`properties` for every `Ty::BlockState`) — a LATER
version's fix: it only runs for upgrades to DataVersion ≥ 5006, and
this port's `CURRENT` is 4790 (`update` runs fixes with
`k/10 <= to_version`, datafix/mod.rs:223), so it NEVER runs here. A
vanilla 26.1.2 world at DataVersion 4790 runs NO datafix at all on
load (`upgrade_record` returns early when the record's version is ≥
`DATA_VERSION`, save.rs:2452). The 26.3 reference world, by contrast,
writes `id`/`properties` compounds (and bare strings) — the strict
reader is 26.3-correct and 26.1.2-wrong. **26.3-leak**, adopted a
version early.

Consequence, traced through the load path: `SaveDir::load`
(save.rs:2880) → `decode_with` → `decode_column` (save.rs:1756), which
resolves each palette entry via `registry.saved_state_id`
(registry.rs:230-241: `BlockState::from_nbt(tag).ok()`) and maps an
unresolvable entry to AIR (`map_or(air, |id| id.0)`, save.rs:1799).
Every palette entry in a vanilla-written 26.1.2 world fails `from_nbt`,
so **the whole world decodes as air** — the game loads a void, with no
error surfaced. The LOD path drops the same entries
(save.rs:5146, `BlockState::from_nbt(entry).ok()?`); `ChunkData::block`
/ `set_block` (used by save/gateway.rs:192 and horizon-world's data
tool) fail outright with the `io::Error`; the `world_fixture` probe
fails at `BlockSection::from_nbt`. Port-written saves are unaffected
(the port's own `to_nbt` writes `id`, chunk.rs:192-207), so this hits
exactly the worlds a tester brings from vanilla — including the
reference world itself.

CI corroboration (run 45150986405, head 81311fcc): the tree's own
tests assert the new form where the upgrade cannot produce it, and
they FAIL — `datafix::tests::block_states_take_their_new_field_names`
(upgrades a chunk at DataVersion 4903, above CURRENT, so no upgrade
runs at all and both assertions fail) and
`save::tests::older_records_of_a_current_world_upgrade_as_they_are_read`
(asserts `palette[0].get("id")` after upgrading a 1.20.1 chunk whose
palette no in-range fix converts), plus the neighbouring save-upgrade
tests `a_chunk_from_before_1_18_is_brought_to_the_current_layout` and
`old_chunks_are_found_in_the_saved_records_and_old_poi_upgrade`.
Flagged but explicitly not minted by Batches 5 and 6; minted here.

### B11-G11 — 26.1.1's PlayerEntry `chatDisabledOrBlocked` fix was dropped in the 26.1.2 tree (minor; NEW vs 26.1.1)

**Scope: a lost port item with no observable behaviour change today.**
The 26.1.1 tree carried the item-5 fix — `chat_disabled_or_blocked`
(social_screen.rs:130-132, `!can_receive_player_messages || blocked`)
with its four-case regression test
(`chat_disabled_or_blocked_follows_26_1_1`, :458-462), pinning the
flipped 26.1.1 formula for `PlayerEntry.<init>`'s `chatDisabledOrBlocked`.
The 26.1.2 tree REMOVED both the function and the test (0 references
to `chat_disabled_or_blocked` anywhere in the 26.1.2 crates; the
social_screen.rs diff is exactly those two deletions). Vanilla 26.1.2
still has the flipped form: `PlayerEntry` is byte-identical between
the 26.1.1 and 26.1.2 client jars (javap diff empty — the class is not
among the 6 changed). The function was uncalled in 26.1.1 (the report
flow is not offered; the row shows the disabled report button vanilla
shows when reporting is off), so nothing observable changes — but the
port no longer pins the formula, and the report flow, when it lands,
would need it re-added (and would have no test guarding the 26.1
inverted form). Noted but not minted by Batch 10 ("lost an uncalled
helper, no behaviour change"). This supersedes the 26.1.1 run's B11-H01
(see §3).

## §2 Oracle defects

No oracle repairs were made (Batch 0's scope decision: measurement +
triage only; the oracle `.java` files live under `versions/26.1.2/`
and this sweep forbids touching them). Two cauldron oracles in this
batch's scope fail against the 26.1.2 jars, both triaged from the
Batch 0 measured table (`docs/sweep/26.1.2/batch-0-oracles.json`,
189/391 compile — zero deviations vs the 26.1.1 table) with the
missing symbol javap-verified against the 26.1.2 jar — not from an
import-level check. Both are the same two oracles the 1.21.7 sweep met
(B11-O01 there was repaired; here they are recorded only). Both are
also minted by Batch 1 (B1-O02, B1-O07) and B11-O01 additionally by
Batch 5 (B5-O01); the coordinator dedups at merge.

### B11-O01 — VanillaCauldronOracle fails: `net.minecraft.util.Prediction` absent in 26.1.2 (minor, oracle-defect; REPRODUCES 26.1.1 B11-O01)

Batch 0: fails on all three classpaths, class `missing-symbol`, first
error `118: cannot find symbol`, 1 error. Line 118 is the
`FakePlayer.drop(ItemStack, boolean, net.minecraft.util.Prediction)`
override — the 26.3 3-arg drop form. `Prediction` is absent from both
26.1.2 jars (jar-index verified) and present in the 26.3 client jar
(`net.minecraft.util.Prediction`). 26.1.2's drop is the 2-arg form, as
in 1.21.7 (whose B11-O01 repaired exactly this). Repair (follow-up,
not done here): drop the `Prediction` parameter, as 1.21.7's B11-O01
did (commit c754e3a2 there). This oracle is the driver behind the
cauldron table comparison (1.21.7's B11-H02); with G23 withdrawn for
26.1.2 the comparison is no longer confirmatory for a live defect,
but the oracle remains broken for any future cauldron driver work.

### B11-O02 — VanillaLavaCauldronItemOracle fails: `getNextEntityId` absent in 26.1.2 (minor, oracle-defect; REPRODUCES 26.1.1 B11-O02)

Batch 0: fails on all three classpaths, class `no-override`, first
error `28: method does not override or implement a method from a
supertype`, 1 error. Line 28 is `@Override public int
getNextEntityId()` on the `Lvl extends ServerLevel` stand-in.
`getNextEntityId` is declared on NEITHER `Level` nor `ServerLevel` in
the 26.1.2 server jar (javap `-p` grep on both classes; the oracle
fails with exactly 1 error, so the neighbouring `registryAccess()`
override at line 29 is fine). The method is a 26.2/26.3 addition the
oracle assumes. Repair (follow-up): remove the override (the stand-in's
`next` counter is only used by the oracle itself) or move the id
allocation out of the `ServerLevel` subclass.

In-scope oracles assessed PASSING (from the Batch 0 table, for
auditability): no other oracle in this batch's scope was assessed —
Batch 11 runs no drivers and its only in-scope oracles are the two
cauldron oracles above, both failing. The full failing-oracle list
(202 files) belongs to the batches whose drivers use them; the
coordinator dedups at merge.

## §3 Needs-new-harness

None. Two notes on inherited harness items:

- **26.1.1's B11-H01 (item 5's regression test cannot run) is
  WITHDRAWN for 26.1.2 — its premise changed, not its blocker.** The
  26.1.1 item was "the fix is in the tree but its regression test
  cannot run until the pre-existing horizon-client lib-test breakage
  is fixed". In 26.1.2 BOTH halves of that premise are gone: the
  lib-test breakage was fixed (the behaviour port's enabling work;
  crate-tests run 45150986405 compiled and ran the suites), AND the
  fix plus its test were removed from the tree (B11-G11). There is no
  test left to run and no harness that would settle anything — the
  loss itself is recorded as B11-G11. No new harness item is minted.
- **26.2's B11-H01 (class-table chain-liveness half) stays CLOSED.**
  Batch 0's prepare landed for 26.1.2, so `block-behaviours.json`
  exists (1,168 blocks) and the chain half ran again here (§6.1):
  263/271 mappings chain-live, 0 jar-live-but-chain-dead.
- **1.21.7's B11-H02 (exhaustive cauldron-table driver comparison)
  does NOT carry.** Its premise was confirming B11-G23 (the wash gap);
  G23 is withdrawn for 26.1.2 (§6.6 — the tag is present and the gate
  is live). The 26.1 prepare does not unblock it in any case: the
  driver is blocked on the two oracle defects above (B11-O01/O02),
  and the comparison itself needs a probe build this batch does not
  run. No new harness item is minted for it: nothing in this batch's
  scope needs a harness to settle.

## §4 Needs-in-game-probe

None. The 1.21.7 batch's probe item (F3 chart footer, P01) does not
carry: 26.1.2 HAS the `DebugScreenOverlay` chart footer in the jar, and
the Rust's three-line form matches it — the only delta is the lowercase
`fps` name, recorded as B11-G08. No ledger item needed a probe to
settle.

## §5 Accepted gaps / notes (not defects)

The 26.1.2 port ledger carries NO accepted-gap verdicts (unlike 26.2's
items 427/436), so none of these is a ledger item and none is counted
in the 7-item tally. All three are notes carried from the 1.21.7
sweep, re-verified against 26.1.2:

- **B11-A01 — ProvidesTrimMaterial inline/direct form [= 1.21.7
  B11-A01, = 26.1 B11-A01, = 26.1.1 B11-A01].** Vanilla 26.1.2's
  `ProvidesTrimMaterial.material` is `Holder<TrimMaterial>`
  (javap-verified) — a Holder direct OR a ResourceKey string. The Rust
  reads the string form only (enchant.rs:333-335, `.as_str()`). All 11
  vanilla 26.1.2 items with the component use string keys (prepared
  items.json: amethyst shard, copper ingot, diamond, emerald, gold
  ingot, iron ingot, lapis lazuli, netherite ingot, quartz, redstone,
  resin brick — all values verified strings) — the direct/inline form
  is datapack-only.
- **B11-A02 — Special-foil decal not modelled [= 1.21.7 B11-A03, =
  26.1 B11-A02, = 26.1.1 B11-A02].** The Rust HAS a `Foil::Special`
  variant ("compasses and clocks, a sheeted decal", item_render.rs:145-146)
  carried through `submit_quads` (:437), but both sinks treat every
  non-None foil identically: the GUI sink draws the glint over the
  full quad (item_render/gui.rs:421-423) and the world sink likewise
  (item_render.rs:1155-1173) — no 0.5x GUI / 0.75x first-person decal
  scaling. Only visible on a command-enchanted compass/clock.
  Pre-existing in both versions, not a version delta.
- **B11-A03 — NBT separator ignored + per-player resolution [= 1.21.7
  B11-A04, = 26.1 B11-A03, = 26.1.1 B11-A03].** The Rust's
  `component_arg` (game/commands/text_commands.rs:60-135) resolves
  selectors ONCE for the command source and always joins names with
  `", "` (:120, :231) — the separator field of nbt/selector components
  is ignored and a title/tellraw component is not re-resolved per
  player (vanilla resolves per player). Gaps from BOTH versions
  equally, not version deltas.

NOT carried: 1.21.7's B11-A02 (NBT `sound_variant` written on
non-wolf mobs) NEVER EXISTED for 26.1.2 — 26.1.2 vanilla HAS sound
variants for cows, pigs, chickens and cats (the `ChickenSoundVariants`,
`CowSoundVariants`, `CatSoundVariants`, `PigSoundVariants` and
`WolfSoundVariants` registry classes all exist in the 26.1.2 server
jar), and the Rust's `sound_variants` (mobs/finalize.rs:2368-2377)
lists exactly those five species. Writing the tag is correct 26.1.2
behaviour.

## §6 Systemic findings

### §6.1 The class table: 8 dead mappings, 4 with live arms — and the chain half RUNS again

Method: parse every `Class::X = "Name"` in
`versions/26.1.2/crates/horizon-world/src/level/classes.rs` (271
mappings — the table has NO dual names at all, unlike 26.2's, which
carries `"PathBlock" | "DirtPathBlock"` and `"RedstoneWireBlock" |
"RedStoneWireBlock"`; classes.rs is 0-diff vs the 26.1.1 tree); LIVE iff
the name exists as a class (top-level or nested) in either 26.1.2 jar
(server-game 7,351 classes; client 10,682 — the same counts as 26.1.1
and 26.1). Chain half: LIVE iff the name appears in ≥1 block chain in
the prepared `block-behaviours.json` (1,168 blocks, 291 distinct chain
names).

Result: **263/271 jar-live, 8 jar-dead; 263/271 chain-live, 0
jar-live-but-chain-dead.** The jar half and the chain half agree
exactly — every jar-live mapping is chain-live, every jar-dead mapping
is chain-dead. The chain half adds no new dead mappings, which keeps
the 26.2 H01 question settled in the negative for 26.1.2 as well: no
mapping is live in the jar but dead in the chains. The sweep is
identical to 26.1.1's in every count (the classes.rs file is unchanged
and the 26.1.2 chains match 26.1.1's).

**Group A — RENAME: 26.1.2 content exists under another name; live arms
dead (4):**

| Mapping | 26.1.2 form (jar + chain) | Dead arms | Defect |
|---|---|---|---|
| AbstractBed → `AbstractBedBlock` | `BedBlock` directly | 5 (place ×2, interact, hooks, shapes) | B11-G01 (major) |
| RedstoneWire → `RedstoneWireBlock` | `RedStoneWireBlock` | 10 (hooks ×3, interact, place, redstone ×3, shapes, survive) | B11-G02 (major) |
| Path → `PathBlock` | `DirtPathBlock` | 4 (place, shapes, survive, tick) | B11-G03 (minor) |
| Speleothem → `SpeleothemBlock` | `PointedDripstoneBlock` directly (no abstract parent) | 5 (place, shapes, survive, tick ×2) | B11-G04 (minor) |

All four 26.3 names exist in the 26.3 client jar — the table is
26.3-correct and 26.1.2-wrong in all four cases. The other five 1.21.7
renames do NOT carry to 26.1.2 — the 26.x names MATCH the 26.1.2 jar
(verified each, not assumed): FarmlandBlock ✓ (chain
`[FarmlandBlock, Block]`), SnowyBlock ✓ / SpreadingSnowyBlock ✓
(grass_block chain `[GrassBlock, SpreadingSnowyBlock, SnowyBlock,
Block]`), NetherFungusBlock ✓ / NetherRootsBlock ✓ (crimson_fungus
chain `[NetherFungusBlock, VegetationBlock, Block]`). LilyPad is the
correct `LilyPadBlock` (26.1.2 has no `WaterlilyBlock`); CopperChestBlock
✓ (chain via `WeatheringCopperChestBlock`), CopperGolemStatueBlock ✓.

**Group B — NO 26.1.2 CONTENT: dead but harmless (4):** StrawBedBlock
(26.3-only; 1 arm: update.rs:430), SulfurSpikeBlock (26.2-only; 1 arm:
speleothem.rs:110, the max-height-2 branch), ShelfMushroomBlock
(26.3-only; 4 arms: bonemeal.rs:94, place.rs:493, shapes.rs:137,
survive.rs:42), PotentSulfurBlock (26.2-only; 4 arms: hooks.rs:199,
pistons.rs:154, place.rs:1193, shapes.rs:819). All four classes exist
in the 26.3 jar (SulfurSpike/PotentSulfur also in 26.2's) and none has
any block in 26.1.2 data (0 straw_bed / sulfur_spike / shelf_mushroom /
potent_sulfur entries in block-behaviours.json). Arms unreachable.
Hygiene note, not gaps.

Also verified: horizon-prepare's survival.rs CLASS_RULES carries the
same four 26.3 names (`PathBlock` :10, `RedstoneWireBlock` :43,
`SpeleothemBlock` :21, `ShelfMushroomBlock` :18) — the prepare-side
table misses 26.1.2's chain names the same way (see B11-G02).

### §6.2 The 26.x tag-name pattern INVERTS for 26.1.2 — except the two 26.3-only tags

The 1.21.7 sweep's systemic cause — the port reads 26.x tag names for
tags that don't exist in 1.21.7 data — mostly does not apply to 26.1.2,
because 26.1.2 HAS the 26.x tags. Every tag read this batch checked is
live in 26.1.2's data (client-jar datapack, 775 tag files — the same
count as 26.1.1; tag-file existence + contents):

- `minecraft:cauldron_can_remove_dye` (item tag: leather helmet /
  chestplate / leggings / boots, leather horse armor, wolf armor) — the
  Rust's wash gate (horizon-client/src/cauldron.rs:7-12, :214) is live;
  1.21.7's B11-G23 does not apply.
- `supports_chorus_plant` / `supports_chorus_flower` (both: end stone)
  — survive.rs:102/132/137 live; 1.21.7's B11-G21 does not apply.
- `overrides_mushroom_light_requirement` (mycelium, podzol, crimson +
  warped nylium) — survive.rs:266 live; `mushroom_grow_block` is the
  ABSENT one in 26.1.2 (the reverse of 1.21.7); 1.21.7's B11-G17b does
  not apply.
- `cannot_support_snow_layer` (ice, packed ice, barrier) /
  `support_override_snow_layer` (honey block, soul sand, mud) —
  survive.rs:295/297 live; the 1.21.11 names
  (`snow_layer_can_survive_on` / `snow_layer_cannot_survive_on`) are
  absent in 26.1.2 (the reverse of 1.21.7); 1.21.7's B11-G16b does not
  apply.
- `convertable_to_mud` (dirt, coarse dirt, rooted dirt) — live (the
  26.1.2 spelling; note B5-G19 found a DIFFERENT read site — the water
  bottle — using the wrong `convertible_to_mud` spelling; that is Batch
  5's item, not this batch's).
- `dripstone_replaceable_blocks` (`#base_stone_overworld`) — live.

Correctly-absent tags, correctly handled: `sculk_growth_inhibitors` is
absent from 26.1.2 data and the Rust checks sculk_sensor/sculk_shrieker
blocks explicitly in 26.1's form (26.1 item 510) — but the 26.1.2 tree's
level/sculk.rs:620 reads the TAG (`minecraft:sculk_growth_inhibitors`),
so the inhibitor check is always false. That is Batch 4's B4-G37
(sculk growth never inhibited), flagged here as a cross-ref, not
re-triaged. The two EXCEPTIONS — tags the port reads that 26.1.2
genuinely lacks — are both 26.3-only and both produce defects: the
potion tags `douses_fire` / `hurts_water_sensitive_entities` /
`extinguishes_entities` / `rehydrates_axolotls` (26.1.2's `PotionTags`
has ONLY `TRADEABLE`; the 26.3 datapack has all four) — B11-G05; and
`speleothems` (26.2+ only) — the second cause of B11-G04.

### §6.3 26.3-form code that 26.1.2's jar contradicts — and the 26.1 fixes the tree did NOT inherit

The port carries 26.3 forms in EIGHT places where 26.1.2's jar/data
says otherwise — all eight are 26.1.2-direct fixes, not re-base
propagations. (The 26.3 client jar, 11,383 classes, was used ONLY for
these absence checks and the §6.5 comparison — never as the spec.)

1. The four class-table mappings (§6.1 Group A): `AbstractBedBlock`,
   `RedstoneWireBlock`, `PathBlock`, `SpeleothemBlock` all exist in the
   26.3 jar and in NEITHER 26.1.2 jar.
2. The AnyFuel grouping (B11-G07): 26.3 resolves via
   `DataComponentLookup.findAll(COOKING_FUEL)` (javap-verified on the
   26.3 jar); 26.1.2 via the insertion-ordered `FuelValues` — and the
   26.1.2 tree reads the `cooking_fuel` component, which 26.1.2
   vanilla does not define at all.
3. The splash-water tag gate (B11-G05): 26.3's datapack HAS the four
   potion tag files; 26.1.2's has only `tradeable.json`, and 26.1.2
   vanilla gates on `Potions.WATER` equality.
4. The F3 spawn-counts abbreviations (B11-G09): 26.2+ uses
   `MobCategory.getDebugAbbreviation`; 26.1.2 uses the first letter
   of the category name, uppercased.
5. The bed arm's pair check (part of B11-G01): 26.3's
   `AbstractBedBlock` added the pair check; 26.1.2's `BedBlock`
   redirects a foot click to the head and requires only that the head
   is this bed.
6. **NEW vs 26.1.1 — the palette reader (B11-G10):** 26.3 writes
   `id`/`properties` palette compounds (verified in the 26.3 reference
   world); 26.1.2 writes `Name`/`Properties` (verified in the 26.1.2
   reference world), and the rename is datafix 5006, past this
   version. The 26.1.2 tree reads only the 26.3 form.

The Group B mappings (StrawBed, SulfurSpike, ShelfMushroom,
PotentSulfur) are 26.2/26.3-only content carried harmlessly. The Mood
spacing is NOT 26.3-form — 26.1.2's own literal is one-spaced (see §6.6
G20), and the Rust matches it; the two-space form belongs to 26.2/26.3.

**The inheritance failure (measured, not assumed).** The 26.1.2 tree
was diffed file-by-file against the 26.1.1 tree in this batch's scope.
26.1 fixes the 26.1.2 tree does NOT carry (all inherited unchanged
from 26.1.1): the bed-arm redirect (interact.rs — 26.1's item-584
port, replaced by the 26.3 pair-check form), the `RedStoneWireBlock`
name in comments/docs (survive.rs:282, interact.rs:492 — both carry
the 26.3 spelling), the hardcoded `burn_time`/`compost_chance`/
`villager_food` tables (items.rs — component reads for `cooking_fuel`
/ `brewing_fuel` / `compostable` / `villager_food` / `mob_visibility`,
components 26.1.2 vanilla does not define; Batch 4 documents the same
diff), the composter chance form (interact.rs
`composter_add_item` → `composter_add_layer`; Batch 5's B5-G22 owns
the composter consequence), and the single-letter F3 category
abbreviations (debug_overlay.rs). Zero-diff files (26.1.1 behaviour
carries): classes.rs, splash_water.rs, game/social.rs, loot.rs,
items.rs, recipe_book.rs, debug_keys.rs, debug_overlay.rs,
interact.rs, survive.rs, place.rs, hooks.rs, shapes.rs, tick.rs,
redstone.rs, speleothem.rs, enchant.rs. The only in-scope changes
26.1.1→26.1.2 are the delta's own: the `handleAttack` gate + test
(hub.rs, hub_tests.rs), the checkbox helper + test (dialog_screen.rs),
the palette-reader removal (chunk.rs), the `record_is_old` threshold
+ test (save.rs), the datafix key/test changes (fixes_c.rs, files.rs,
tests.rs), and the REMOVAL of the 26.1.1 item-5 fix
(social_screen.rs). PORT-NOTES-26.1.2.md says the completion pass "has
since been carried forward into this tree" — the diffs above show that
carry-forward was partial at best for behaviour code, exactly as in
26.1.1; the claim holds for the protocol/tables work, not for the
behaviour fixes. (The same inheritance failure is the programme-wide
systemic finding of the landed Batches 1–7.)

### §6.4 Re-base WARNING: 26.1.2 is 26.x where 1.21.7 is not — do not propagate 1.21.7 forms down

This batch found ELEVEN behaviours where 26.1.2's vanilla form differs
from 1.21.7's and the Rust already matches 26.1.2 — a re-base must not
propagate the 1.21.7 forms to 26.1.2: Riptide passenger check (26.1.2
HAS `isPassenger` in `releaseUsing`, offset 63), SwellGoal alive check
(26.1.2 HAS `isDeadOrDying` in `canUse` + `tick`), polar-bear fox-goal
age gate (26.1.2's selector is the `!isBaby` lambda), lead-knot discard
on failed tie (26.1.2 discards the fresh knot, offsets 147-149),
mushroom bonemeal gate (26.1.2 requires the huge-mushroom feature +
`4 + foliageRadius` height), sapling bonemeal height check (26.1.2:
`TreeGrower.getMinimumHeight` + build height, no `canGrow`), golem
stroll roll order (26.1.2 draws 0.3 first, 0.7 second — the Rust's
order; 1.21.7 stores the first roll and compares the second to 0.3),
social gate (26.1.2's chat check feeds ONLY the report button;
hide/show are ungated), trader heightmap + storage (26.1.2 registers
WANDERING_TRADER with MOTION_BLOCKING_NO_LEAVES and stores SavedData
`wandering_trader.dat`, fields `spawn_delay` / `spawn_chance`),
shipwreck `heightAdjusted` caching (26.1.2 HAS the field), player-data
layout (26.1.2 IS `players/data/`). The 1e-5 fluid-current floor:
present in vanilla 26.1.2 (the constant 9.999999747378752E-6 appears
9 times in 26.1.2's `Entity` constant pool — the class is byte-identical
to 26.1.1's) AND in the Rust (entity_fluid.rs:87,
`length_squared() < 1.0e-5`). No floor-removal to propagate; the 1.21.7
§6.4 warning (the template removed the floor for 1.21.11, but 1.21.7
HAS it) stands as a general per-version caution. Three 26.1.2-vs-26.2
cautions also belong here: 26.1.2's F3 Mood literal is ONE-spaced
while 26.2's is TWO-spaced (do not propagate 26.2's form down);
26.1.2's F3 spawn-counts abbreviations are single letters while 26.2's
are two/three letters (B11-G09 — do not propagate 26.2's form down);
and 26.1.2 vanilla HAS sound variants for cows/pigs/chickens/cats (do
not propagate 1.21.7's A02 note — see §5). A fourth, from §6.3:
26.1.2 has NO `cooking_fuel`/`brewing_fuel`/`compostable`/
`villager_food`/`mob_visibility` components — do not propagate the
26.3 component forms down; the 26.1.2 forms are the hardcoded tables
and `FuelValues`. A fifth, NEW: 26.1.2 writes `Name`/`Properties`
chunk palettes — do not propagate the post-5006 `id` reader down
(B11-G10); the 26.1.2 reader must accept the old form, as 26.1.1's
`from_palette_nbt` did.

### §6.5 Class-absence headline: 0 of the ledger's tokens name 26.2-only or 26.3-only classes

The 7-item ledger is small, so the headline is small. 10 capitalized
tokens were extracted from `work-items.json` (note + rank3plus text)
and checked against all five jar indexes (both 26.1.2 jars, both 26.2
jars, the 26.3 client jar):

- **6 real vanilla classes, ALL present in 26.1.2:**
  `AbstractReportScreen` (client only), `Checkbox` (client only),
  `DataFixers`, `DetectedVersion`, `ServerGamePacketListenerImpl`,
  `SharedConstants` (both jars). Each is triaged in §8 (items 1–7).
  None is a 26.2-only or 26.3-only class.
- **4 non-class tokens** (English words: `Behaviour`, `Everything`,
  `Patch`, `The`) — absent from ALL FIVE jar indexes, 0 real vanilla
  absences.

The class-absence story for this batch therefore lives in §6.1, not
in the ledger: the 8 dead class-table mappings (4 renames with live
arms dead, 4 content-absent) are all 26.2/26.3-only names, and the
component story of §6.3 (`COOKING_FUEL` etc. — fields on
`DataComponents`, not classes) is the same pattern at the component
level: the 26.1.2 tree names 26.2/26.3 surfaces 26.1.2 vanilla never
had. (For scale: the 26.1 run's 214-item ledger named 69 items with
26.2/26.3-only classes, 57 tokens; the 26.1.2 ledger's 7 items name
none — the absence lives in the carried-forward code, not in the
delta's ledger.)

### §6.6 1.21.7 Batch 11 defects re-checked against 26.1.2

Every 1.21.7 Batch 11 defect was re-checked against 26.1.2's jar/data/Rust.
Seven carry over (plus one cosmetic detail from P01) — the same seven
as 26.1 and 26.1.1 — plus the carried F3 defect (G09) that 26.1 did
not have; the rest are withdrawn with the deciding evidence. The
26.1.1 run's "26.1-record corrections" (summary.md line 360) were
applied: none of them touches this batch's scope — B3-G33/G34 are
Batch 3's (visibility, trader/villager hurt gating), B8's notes are
Batch 8's (providers driver, Villager.handleEntityEvent), B10-G15 is
Batch 10's (sound categories). No correction changed any verdict below.

**APPLIES (minted as 26.1.2 items):**

- G01 beds — APPLIES (B11-G01 above). Major. 26.1.2 HAS a bed BE type
  and `BedBlock` — but the arms dispatch on the dead `AbstractBed`,
  and the arm logic is now the 26.3 pair-check form too.
- G02 redstone wire — APPLIES (B11-G02 above). Major. 26.1.2 has
  `RedStoneWireBlock`; the table has only the 26.3 name, no dual.
- G03 farmland/paths — APPLIES in part (B11-G03 above): paths dead
  (`DirtPathBlock` vs `PathBlock`); farmland live.
- G06 speleothem — APPLIES (B11-G04 above). 26.1.2 has neither the
  class nor the tag.
- G12 number providers — APPLIES (B11-G06 above), datapack-only.
- G13 splash water — APPLIES (B11-G05 above). Major. The Rust's tag
  gate is dead in 26.1.2 (only `tradeable` exists).
- G15 fuel order — APPLIES (B11-G07 above), in a worse form: the
  component-based list is empty, and the order it would cycle is not
  vanilla's FuelValues order.
- P01 F3 chart footer — SETTLED BY THE JAR, with one cosmetic delta
  (B11-G08 above): layout matches; only the `fps` casing differs.
- (Carried from 26.1.1) F3 spawn-counts abbreviations — B11-G09 above:
  the 26.1 tree was fixed; the 26.1.1 tree regressed to the 26.2+ form
  and 26.1.2 inherits it.

**Withdrawn (verified not-applicable, with the deciding evidence):**

- G04 snowy — SnowyBlock/SpreadingSnowyBlock live in the 26.1.2 jar +
  chains (§6.1).
- G05 fungus/roots — NetherFungusBlock/NetherRootsBlock live (§6.1).
- G07 sapling bonemeal — 26.1.2's gate IS the height check
  (`getMinimumHeight` + build height, javap-verified); Rust matches
  (bonemeal.rs:331-340). (Batch 5's B5-G29 records a SEPARATE
  `canGrow`-gate issue for lone dark/pale oak saplings — different
  mechanism, Batch 5's item.)
- G08 riptide riding — 26.1.2 vanilla HAS the isPassenger check
  (releaseUsing offset 63); Rust blocks while riding
  (trident.rs:501).
- G09 lead knot — 26.1.2 vanilla discards the fresh knot on a failed
  tie (offsets 147-149); Rust creates the knot only if attached
  (leads.rs:640-653). Same observable outcome.
- G10 creeper defuse — 26.1.2 vanilla HAS isDeadOrDying in SwellGoal
  canUse + tick; Rust gates on t.alive (hostile.rs:294, :312).
- G11 polar-bear cubs — 26.1.2's fox-goal selector IS !isBaby
  (lambda); Rust matches (polar_bear.rs:51-56).
- G14 player-data layout — 26.1.2 vanilla IS players/data/
  (LevelResource.PLAYER_DATA_DIR = "players/data", constant-pool
  verified); Rust matches (lan/playerdata.rs:18).
- G16 golem stroll + snow tags — stroll roll order matches 26.1.2
  exactly (0.3 first, 0.7 second — mobs/golem.rs:152/155; the vanilla
  goal class is byte-identical to 26.1.1's, whose order the 26.1 run
  javap-verified via the 26.1.1 chain); snow tags are the 26.x names,
  present (§6.2).
- G17 mushroom — 26.1.2's gate genuinely requires the feature + height
  (`4 + foliageRadius`, javap-verified); Rust matches
  (bonemeal.rs:342-356); light tag present (§6.2).
- G18 social gate — 26.1.2's chat check feeds ONLY the report button
  (PlayerEntry ctor, byte-identical to 26.1.1's); hide/show are
  ungated; Rust matches (no chat gate on hide/show in game/social.rs;
  the report-button formula was 26.1.1's item 5 — DROPPED in this tree,
  see B11-G11). NOTE: 26.1.2 keeps the 26.1.1 flipped form
  (`!canReceive || isBlocked`).
- G19 trader — heightmap MOTION_BLOCKING_NO_LEAVES matches
  (WanderingTraderSpawner uses `SpawnPlacements.getHeightmapType`;
  the registration is javap-verified in the 26.1.2 jar); storage is
  SavedData wandering_trader.dat with `spawn_delay`/`spawn_chance`,
  matches (wandering_trader.rs:48, :61-62, :74; `WanderingTraderData`
  javap-verified).
- G20 F3 lines — ALL THREE thirds match 26.1.2: Local Difficulty has no
  Day suffix (`"Local Difficulty: %.2f // %.2f"`, day count is the
  separate `DebugEntryDayCount` entry `"Day #"`, which the Rust renders
  as `"Day #{}"` at debug_lines.rs:57); memory lines use MiB
  (`"Mem: %2d%% %03d/%03dMiB"`); and the Mood literal is ONE-spaced in
  26.1.2 (`' (Mood %d%%)'`, constant-pool verified) — the Rust's
  one-space form matches 26.1.2 exactly. (26.2's literal is
  two-spaced; the one-space form is a 26.1.2-form match, not a defect
  here.) The spawn-counts line is a FOURTH F3 element and is defective
  (B11-G09) — it was not part of 1.21.7's G20.
- G21 chorus — tags present in 26.1.2 data (§6.2).
- G22 dialog tooltip — 26.1.2's ItemDisplayWidget shows its tooltip on
  `tooltip && isHovered()` only (offsets 97-107; the `isFocused()`
  call at offset 69 draws the focus OUTLINE, not the tooltip);
  Rust matches (hover-only, game/dialogs.rs:705).
- G23 cauldron wash — tag present with leather armour (§6.2); Rust
  gate reads it (horizon-client/src/cauldron.rs:7-12, :214).
- H01 shipwreck — 26.1.2 HAS heightAdjusted; Rust caching matches
  (worldgen/structure/saved_templates.rs:178, simple.rs:250).
- H02 cauldron table — premise withdrawn with G23 (see §3); the two
  cauldron oracles are recorded as B11-O01/O02 (§2).
- A01 trim inline form — CARRIES as an accepted note (B11-A01, §5).
- A02 sound_variant — NEVER EXISTED for 26.1.2 (see §5): 26.1.2 vanilla
  has sound variants for cows/pigs/chickens/cats.
- A03 special-foil decal — CARRIES as an accepted note (B11-A02, §5).
- A04 separator + per-player resolution — CARRIES as an accepted note
  (B11-A03, §5).

## §7 Method notes

- **Jar-diff first.** Every 26.1.2 verification in this report starts
  from the jar-diff 26.1.1 → 26.1.2 (sha1 per class, both jars):
  server-game 7,351 classes, client 10,682 classes, +0 −0 both sides,
  and only 8 client class files changed (5 top-level classes:
  `DetectedVersion`, `SharedConstants`, `Checkbox` + `Checkbox$Builder`
  + `Checkbox$OnValueChange`, `AbstractReportScreen` +
  `AbstractReportScreen$DiscardReportWarningScreen`,
  `ServerGamePacketListenerImpl`) and 3 server class files
  (`DetectedVersion`, `SharedConstants`, `ServerGamePacketListenerImpl`).
  The two nested classes (`Checkbox$OnValueChange`,
  `AbstractReportScreen$DiscardReportWarningScreen`) changed only via
  constant-pool references — their javap diffs are empty — so the
  pipeline's "6 changed classes" count (PORT-NOTES-26.1.2.md) is
  consistent with the measured 8 changed class files. Every other
  class is byte-identical to 26.1.1's, so 26.1.1's javap evidence
  carries for unchanged classes — but every defect-bearing class was
  still re-verified fresh against the 26.1.2 jars, and the report
  states where a verdict rests on byte-identity rather than a fresh
  javap (golem stroll order, trader heightmap registration — see
  below).
- **javap verification.** `javap -c` (JDK 25, `~/workspace/tools/jdk-25/bin/javap`)
  against the named 26.1.2 jars for every vanilla method in scope:
  handleAttack's guard, `Checkbox.build` / `overflowsRowLimit`,
  `DetectedVersion.createBuiltIn`, `AbstractReportScreen.createFooter`,
  `AbstractThrownPotion.onHitBlock`/`onHitEntity`, `AnyFuel.resolve` /
  `FuelValues.fuelItems`, `DebugScreenOverlay` footer,
  `DebugEntrySpawnCounts`, the 1.21.7 re-checks (riptide releaseUsing,
  SwellGoal, lead bindPlayerMobs, mushroom/sapling gates, WANDERING_TRADER
  registration, PlayerEntry ctor), and `DataComponents` field listings
  (no COOKING_FUEL in 26.1.2; COOKING_FUEL present in 26.3). 5 string
  literals verified by constant-pool parsing (Mood spacing, Day #,
  Mem MiB, Local Difficulty, players/data). Class-existence checks used
  zipfile indexes (scratch `*-classes.json`), not per-class javap.
  The 26.2 jars and the 26.3 client jar were used ONLY for
  absence/comparison checks (getDebugAbbreviation, SpeleothemBlock,
  potion tags, `id` palettes, 26.3-form code) — never as the spec.
- **Class-table sweep.** `class-table-sweep.json` (scratch): 271
  mappings × jar-liveness × chain-liveness, with the chain half run
  against the prepared `block-behaviours.json` (1,168 blocks, 291
  distinct chain names) — the 26.2 run deferred this half because
  prepare hadn't landed; it has landed for 26.1.2 (Batch 12), so the
  sweep is complete and 26.2's B11-H01 stays closed.
- **Tree diff.** The 26.1.1 vs 26.1.2 tree diff was run file-by-file
  in this batch's scope: every defect-bearing Rust file is
  byte-identical to 26.1.1's (classes.rs, interact.rs, survive.rs,
  splash_water.rs, loot.rs, items.rs, recipe_book.rs, debug_keys.rs,
  debug_overlay.rs, place.rs, hooks.rs, shapes.rs, tick.rs,
  redstone.rs, speleothem.rs, enchant.rs, game/social.rs,
  text_commands.rs, item_render.rs, cauldron.rs, registry.rs) — so
  all line numbers carry from 26.1.1, and the two NEW 26.1.2 defects
  (B11-G10, B11-G11) rest on measured diffs, not on inference.
- **Data checks.** 26.1.2 client-jar datapack (775 tag files) + prepared
  data: 1,326 loot tables counted (0 using the 3 missing providers),
  0/1,506 items with a cooking_fuel component (and 0/1,506 each for
  brewing_fuel, compostable, villager_food), 11 trim items all string
  keys, protocol ids oak planks 36 / coal 897 / lava bucket 1015.
- **Reference-world check.** A minimal NBT reader (`nbt_probe.py`,
  scratch) over the `vanilla-world-mc` 26.1.2 / 26.1.1 / 26.3 region
  files (read in place; no git command was run in that clone):
  26.1.2 and 26.1.1 worlds write `Name`/`Properties` palettes;
  the 26.3 world writes `id`/`properties` (and bare strings). The
  26.1.2 world's seed (4656297851632877335) was confirmed from
  `data/minecraft/world_gen_settings.dat`, DataVersion 4790.
- **Oracle triage from the Batch 0 measured table ONLY** (189/391
  compile, zero deviations vs 26.1.1) — no re-measurement. Missing
  symbols (`Prediction`, `getNextEntityId`) javap-verified against
  the 26.1.2 jars. No oracle repaired.
- **Overlap check.** The landed Batch 1–8, 10 and 12 reports in
  `docs/sweep/26.1.2/` were grepped before minting (B1-G03, B2-G01,
  B4-G01, B4-G37, B5-G03/G13/G19/G27, B10-G03); B5/B6's palette flags
  and B10's chat-helper note were confirmed as flagged-not-minted
  before B11 minted them.
- **CI log check.** `gh run view 45150986405 --log-failed` (repo
  KarinaFromAespa/Minecraft-in-Rust, head 81311fcc): both regression
  tests green, horizon-client 1274/0, and the four save/datafix
  failures corroborating B11-G10. No cargo builds, no driver runs, no
  git commits. Static checks only (javap + python + grep).
- **Unproven by design:** (1) the golem-stroll roll order and the
  trader heightmap registration rest on the Rust side + the
  byte-identity of the vanilla classes to 26.1.1's (the 26.1 run
  javap-verified the order and registration against the 26.1.1 jar;
  the WANDERING_TRADER registration itself WAS javap-verified fresh
  against the 26.1.2 jar — only the stroll order rests on
  byte-identity alone); (2) the palette defect's runtime consequence
  (world loads as void) is derived statically from the decode chain,
  not demonstrated by running the game — the save path maps
  unresolvable entries to air silently, so the void is the derived
  outcome, not a demonstrated one. Both are stated here, not hidden.

## §8 Per-item triage — the 7 port-ledger items

| Item | Title | Rank | Verdict | Evidence (all verified 2026-11-25) |
|---|---|---|---|---|
| 1 | vanilla_command_order generator | 1 | **pass** | `crates/horizon-command/src/vanilla_order.rs` exists, `pub const ORDER` at line 6; byte-identical to 26.1.1's. Jar-diff: no command class changed 26.1.1→26.1.2, so the order carries. |
| 2 | DataFixers 4788→4790 | 1 | **pass** | Jar-diff: NO datafix class changed. `datafix/mod.rs:40` `CURRENT = crate::DATA_VERSION` (4790); `save.rs:207-215` gates world upgrades on `DATA_VERSION`; latest registered fix 4773, no fix runs in (4788, 4790]. (The datafix TEST failures in run 45150986405 are a B11-G10 consequence, not an item-2 failure — the upgrade code is correct; the tests assert a post-5006 form.) |
| 3 | version facts | 1 | **pass** | `version.rs`: VERSION_ID `26.1.2` (:5), PROTOCOL_VERSION 775 (:9), DATA_VERSION 4790 (:11), RESOURCE_PACK_FORMAT (84,0), DATA_PACK_FORMAT (101,1), JAVA_VERSION 25, STABLE true — all match the jar's `SharedConstants` (javap `-constants`: WORLD_VERSION 4790, RELEASE_NETWORK_PROTOCOL_VERSION 775, SNAPSHOT 307, RESOURCE_PACK_FORMAT 84.0, DATA_PACK_FORMAT 101.1). |
| 4 | handleAttack | 2 | **pass** | Javap diff on `ServerGamePacketListenerImpl`: the ONLY change is the head guard — 26.1.1 `if (!hasClientLoaded()) return;` → 26.1.2 `if (!hasClientLoaded() \|\| player.isSpectator()) return;` (offsets 13–29; `resetLastActionTime` at offset 51 comes AFTER the guard). Rust: `lan/hub.rs:4063-4065` (`S::Attack(_) => !self.guests[i].spectator()` in the `acted` match), `:4083-4087` (gate arm drops spectator Attack before anticheat/event/move queue), `game/lan_actions.rs:3516-3518` (`guest_attack` spectator guard as second line). Regression test `lan/hub_tests.rs:931` `a_spectators_attack_reaches_nothing_and_resets_no_timer` (spectator Attack → 0 HostEvent::Packet, last_action_tick stays 41 at tick 100; survival guest still reaches game) — CI-verified green in run 45150986405. |
| 5 | SharedConstants fields | 2 | **pass** | WORLD_VERSION 4790 → DATA_VERSION in version.rs. SNAPSHOT_NETWORK_PROTOCOL_VERSION 304→307 is snapshot-only, 0 references in the Rust tree (grep-confirmed). |
| 6 | Checkbox row-limit tooltip | 3 | **pass** | Jar: `Checkbox` gains `private static final int ROWS = 2` and `overflowsRowLimit(Font)` = `splitLines(message, width, Style.EMPTY).size() > 2` (javap-verified); `Builder.build()` diff shows `setTooltip` now conditional on `overflowsRowLimit(font)` (ifeq skips it). Rust: `dialog_screen.rs:42` `CHECKBOX_ROWS = 2`, `:48-49` `checkbox_overflows_row_limit` (`font.split_lines(label, width).len() > CHECKBOX_ROWS`), test `:519` `a_checkbox_label_over_two_rows_gets_the_tooltip` (1–2 rows in limit, 3 rows overflow, sprite wrap at width 8 vs 24) — CI-verified green in run 45150986405. No live caller yet (only AbstractReportScreen sets a tooltip on a Checkbox builder in vanilla; the report screen is not ported — item 7). |
| 7 | DetectedVersion + report screen | 5 | **pass** | `createBuiltIn`'s only change is `sipush 4788` → `sipush 4790` (offset 10, javap-verified); 0 `DetectedVersion` references in the Rust tree (grep-confirmed). `createFooter` adds `.tooltip(Tooltip.create(ATTESTATION_CHECKBOX))` to the attestation checkbox builder chain (offsets 31–37, javap-verified); 0 `AbstractReportScreen`/`ReportScreen` references in the Rust tree — the report screen is not ported, as documented. |
