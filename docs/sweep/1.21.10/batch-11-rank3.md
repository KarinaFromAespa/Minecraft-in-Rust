# Batch 11 — Rank-3 triage (211 items), 1.21.10 parity sweep

- **Batch:** 11 (rank-3 triage) — checks + triage only, no fixes.
- **Date:** 2026-11-25
- **Scope:** 1.21.10's OWN rank-3 ledger
  (`versions/1.21.10/rank3-progress.json`: 211 items, all closed, keyed
  by item id — a dict, NOT a list — each {module, verdict, evidence};
  verdicts no-change 151, carry-over 49, ported 11), triaged against
  the named 1.21.10 jars and the 1.21.10 port in
  `versions/1.21.10/crates/` (branch `parity-sweep-1.21.10`).
  Vanilla 1.21.10's own jar/data is the spec throughout; the 26.3
  base's form was never trusted. (The `work-items.json` `note` is
  STALE — this batch trusts rank3-progress.json and the tree, not
  the note.) Each module appears once (211 distinct modules).
- **Ledger slice:** `slice.json` (same shape as the 26.1.2 batch-11 slice).

## Tally (211 items)

| Verdict | Count |
|---|---|
| pass | 208 |
| gap (defect attached) | 3 |
| needs-new-harness | 0 |
| needs-in-game-probe | 0 |
| accepted gap (ledger verdict) | 0 |

As-tallied 211 = 151 no-change + 49 carry-over + 11 ported. The three
gaps are item 592 (a real behaviour defect, B11-G19 — the setSize
feedback sub-port is inverted) and items 532/662 (ledger-evidence
staleness: both claim "0 diff lines" but the 592 stopwatch deletion
removed 52 lines from misc.rs and deleted stopwatches.rs outright;
the behaviour is correct, the ledger entries are stale — §6.7). The
class-table sweep (§6.1) is reported with its own counts
(live/total, jar half only — the chain half is B11-H01), NOT folded
into the tally.

**On the brief's lib-test expectation:** the brief anticipated that
1.21.10's regression tests might not run. That premise does not hold:
crate-tests run 45376855229 (head b3cd002835de44ca3681acfcf68a0c45426d8830)
was fully green — 3 jobs green (resolve, crate tests 1.21.10 (linux),
defect suite 1.21.10), 9 "test result: ok" lines, **6,995 passed /
0 failed** (horizon-client 1285, horizon-client-render 2075,
horizon-world 485, horizon-client-core 84, plus 2513/440/110/3/0).
The regression tests RUN — no ledger item needs a harness to settle
on test grounds. (Batch 0 has NOT landed for 1.21.10 — no
`batch-0-oracles.json`, no prepared-data section, no
`block-behaviours.json` — so the oracle triage here is provisional +
direct javap verification, and the chain-liveness half is B11-H01.)

Defects found: **14** (`B11-G04`–`B11-G07`, `B11-G09`–`B11-G10`,
`B11-G12`–`B11-G19`), of which 2 major, 12 minor. Nine reproduce
1.21.7 defects (G04 dripstone narrowed, G05 splash water, G06 number
providers, G07 fuel in worse form, G12 sapling narrowed, G13
playerdata, G14 golem, G15 dialog tooltip, G16 cauldron wash); two
reproduce 26.1.2 defects (G09 spawn counts, G10 palette reader); three
are NEW for 1.21.10 (G17 F3 footer help suffix, G18 prepare survival
table, G19 setSize feedback inversion — the last is the one ported
item whose verdict fails). All fourteen are cross-cutting (class-table
sweep + 1.21.7/26.1.2-defect cross-checks + the 1.21.11-tree diff +
the chunk.rs palette flag) except G19, which attaches to ledger item
592 — so the defect count (14) exceeds the behaviour-gap count (1).
Oracle defects: **2** (`B11-O01`–`B11-O02`, both minor, both cauldron
oracles — §2). Needs-harness: **2** (B11-H01 chain-liveness half,
B11-H02 cauldron table — §3). Accepted-gap notes carried from the
1.21.7 sweep: **3** (`B11-A01`–`B11-A03`, §5 — not ledger verdicts,
not counted in the 211-item tally). No probe items: the 1.21.7 P01
probe is settled by the 1.21.10 jar (§6.6), and the 26.2 run's B11-H01
(chain-liveness half) is RE-OPENED here as B11-H01 because Batch 0
has not landed for 1.21.10.

**Overlap flags for the coordinator (do not fold — merge notes only):**
four of the fourteen defects are also minted by the batch that owns
the system, with the same root cause and evidence — B11-G05 ↔ B1-G03
(splash water), B11-G04 ↔ B5-G13 (dripstone), B11-G07 ↔ B4-G01
(fuel — the `burn_time` 0 consequence is the furnace half), and
B11-G19 may overlap Batch 9 (worldborder feedback; flagged, not
folded). The bed arm's internal pair-check logic is Batch 5's B5-G27
layer (noted in §6.1, not minted — the dispatch itself is fixed).
The coordinator dedups oracles at merge, as in 26.1.2.

**Primary re-checks — the 1.21.7 sweep's 31 items, each decided:**

| 1.21.7 item | Verdict for 1.21.10 | 1.21.10 item |
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
| B11-G12 number providers | REPRODUCES (datapack-only; 0/1325 vanilla tables) | B11-G06 |
| B11-G13 splash water | REPRODUCES | B11-G05 |
| B11-G14 player data path | REPRODUCES | B11-G13 |
| B11-G15 recipe-book fuel | REPRODUCES (worse form: empty list) | B11-G07 |
| B11-G16 golem stroll + snow tags | REPRODUCES (both halves) | B11-G14 |
| B11-G17 mushroom | FIXED (gate + grow block) | — |
| B11-G18 social chat gate | FIXED (chat_allowed gate + tests) | — |
| B11-G19 wandering trader | FIXED (level.dat + WORLD_SURFACE) | — |
| B11-G20 F3 lines | FIXED (Day, MB, two-space Mood) | — |
| B11-G21 chorus on end stone | FIXED (hardcoded) | — |
| B11-G22 dialog tooltip on focus | REPRODUCES (hover-only) | B11-G15 |
| B11-G23 cauldron leather wash | REPRODUCES (tag gate, no leather arms) | B11-G16 |
| B11-H01 shipwreck re-adjustment | FIXED (Rust re-adjusts like vanilla) | — |
| B11-H02 cauldron table | CARRIES (needs-harness) | B11-H02 |
| B11-O01 VanillaCauldronOracle | REPRODUCES | B11-O01 |
| B11-P01 F3 chart footer | FIXED (settled by jar; help suffix is new B11-G17) | — |

**Secondary re-checks — the 26.1.2 run's 16 items, each decided:**

| 26.1.2 item | Verdict for 1.21.10 | 1.21.10 item |
|---|---|---|
| B11-G01 beds | FIXED | — |
| B11-G02 redstone wire | FIXED | — |
| B11-G03 dirt paths | FIXED | — |
| B11-G04 dripstone | REPRODUCES (narrowed) | B11-G04 |
| B11-G05 splash water | REPRODUCES | B11-G05 |
| B11-G06 number providers | REPRODUCES | B11-G06 |
| B11-G07 recipe-book fuel | REPRODUCES (worse form) | B11-G07 |
| B11-G08 F3 fps casing | FIXED (uppercase) | — |
| B11-G09 F3 spawn-counts | REPRODUCES | B11-G09 |
| B11-G10 palette reader | REPRODUCES | B11-G10 |
| B11-G11 chat_disabled_or_blocked | FIXED (chat gate via chat_allowed; blocked half vacuous without UserApiService) | — |
| B11-O01 VanillaCauldronOracle | REPRODUCES | B11-O01 |
| B11-O02 VanillaLavaCauldronItemOracle | REPRODUCES | B11-O02 |
| B11-A01 trim inline form | CARRIES (accepted-gap) | B11-A01 |
| B11-A02 special-foil decal | CARRIES (accepted-gap) | B11-A02 |
| B11-A03 NBT separator + per-player | CARRIES (accepted-gap) | B11-A03 |

Headline: the 1.21.10 port's 211-item ledger holds up — 208 verdicts
re-verify against the 1.21.10 jar and the current tree, 10 of the 11
ported items re-verify individually, and the regression tests ran
green in CI (6,995 passed / 0 failed). The 1.21.10 port is in
markedly better shape than the 26.1.2 delta was: the class table
carries dual names for every 1.21.10 rename (beds, wire, paths,
snowy, fungus, roots all dispatch live), and 17 of the 31 1.21.7
re-checks are FIXED in this tree. But the delta-era inheritance
failures persist where the port reads 26.x-shaped data that 1.21.10
never had: the chunk palette reader accepts only the post-5006
`id`/`properties` form while vanilla 1.21.10 still writes
`Name`/`Properties` palettes — so a vanilla-written 1.21.10 world
loads as void (B11-G10, major, inherited from the 1.21.11 template,
not a delta regression); splash water reads a potion-tag map that is
empty because 1.21.10 has no potion tags (B11-G05, major); and the
fuel list reads a cooking-fuel component 1.21.10 never defines
(B11-G07, minor, worse form — the list is empty).

## §1 Defects

### B11-G04 — Dripstone placement/growth dead via a tag that does not exist (minor; = 1.21.7 B11-G06 narrowed, = 26.1.2 B11-G04 narrowed; REPRODUCES; overlaps B5-G13)

Vanilla 1.21.10's pointed dripstone is `PointedDripstoneBlock`
directly — there is no `SpeleothemBlock` in either jar (26.2
addition; present in the 26.3 jar), and there is no `speleothems`
tag in the client-jar datapack (626 tag files, 0 matching). The
class-table half is FIXED here: the table maps both names and all
five dripstone arms dispatch `Class::Speleothem |
Class::PointedDripstone` (place.rs:1156, shapes.rs:745,
survive.rs:326, tick.rs:257/443), so dispatch is live via
PointedDripstone. What reproduces is the TAG half: placement
thickness is always "tip" (`speleothem_thickness_for`,
place.rs:1398-1438, tag-gates via `is_with`), and growth is dead
(`speleothem_random_tick`'s start gate is the `speleothem_with`
tag, speleothem.rs:91-94; `speleothem_find_tip` and
`is_free_hanging_stalactite` are tag-gated the same way). Survival
(`is_valid_speleothem_placement`, not tag-gated), the scheduled
tick, and fluid transfer are live. Narrowed from 1.21.7/26.1.2,
where dispatch itself was dead.

### B11-G05 — Splash water never douses fire, hurts, extinguishes, or rehydrates (major; = 1.21.7 B11-G13, = 26.1.2 B11-G05; REPRODUCES; overlaps B1-G03)

Vanilla 1.21.10 gates the splash-water effect on
`potionContents.is(Potions.WATER)` (bytecode offsets 57-60,
javap-verified) — a direct potion check, no tag. The Rust's
`splash_water.rs` `potion_in_tag` (:42-51) reads the potion-tags
map, which is EMPTY: 1.21.10 has no potion tag files at all (0 in
the client-jar datapack) and no `PotionTags` class. Every splash
water bottle therefore fails the gate — no dousing, no fire
extinguish, no rehydration. Same root cause as 1.21.7/26.1.2,
unfixed in this tree.

### B11-G06 — Three number providers return 0 (minor, datapack-only; = 1.21.7 B11-G12, = 26.1.2 B11-G06; REPRODUCES)

The 1.21.10 jar has six number providers
(`constant`, `uniform`, `binomial`, `enchantment_level`, `score`,
`storage` — javap-verified). The Rust implements three: loot.rs
int provider `:1344/:1346` falls through to `_ => 0`, float
provider `:1363` to `_ => 0.` (and also lacks `binomial`), and
items.rs `number()` `:1937-1972` falls through to `_ => 0.0`.
Datapack-only, and weaker than it looks: **0 of the 1,325
vanilla loot tables** in the 1.21.10 client jar use
`enchantment_level`, `score`, or `storage` — no vanilla content
is affected, only datapacks that use those providers.

### B11-G07 — Recipe-book fuel slot shows no fuel; the order it would cycle is wrong (minor; = 1.21.7 B11-G15 in worse form, = 26.1.2 B11-G07; REPRODUCES; overlaps B4-G01)

Vanilla 1.21.10 has NO `cooking_fuel` component — `DataComponents`
has no COOKING_FUEL entry (nor brewing_fuel, compostable,
villager_food, or mob_visibility; javap-verified). Fuel times come
from `AnyFuel.resolve`, which reads `FUEL_VALUES` →
`fuelItems()` (a SequencedSet), with `vanillaBurnTimes` adding
LAVA_BUCKET first. The Rust reads fuel from the item's
`minecraft:cooking_fuel` component (items.rs:1044, field
`fuel: Option<(Value, Value)>` at :202) — which no 1.21.10 item
carries — so BOTH fuel sites group an EMPTY list: recipe_book.rs
:566-585 and native_panel_recipes.rs:173-190 group by the
component value in registry order. Worse than 1.21.7's wrong-order
form: the list is empty AND the order form is wrong. The
native_panel comment even admits it ("The grouping below is the
26.1 form; matching 1.21.10's order needs the full vanillaBurnTimes
sequence (see PORT-NOTES item 662)") — and that cross-reference
dangles: there is no item 662 in PORT-NOTES-1.21.10.md (rank-3 item
662 is stopwatches.rs). The furnace half is the same root cause:
`providers.burn_time` (providers.rs:108-116) reads `item.fuel`
and returns 0 when absent, so furnaces never light — that is
Batch 4's B4-G01 territory; this item owns the recipe-book half.

### B11-G09 — F3 spawn-counts line uses 26.2+ category abbreviations instead of first letters (minor, cosmetic; = 26.1.2 B11-G09; REPRODUCES)

Vanilla 1.21.10's `DebugEntrySpawnCounts` abbreviates each
mob category by `Character.toUpperCase(getName().charAt(0))`
(javap-verified) — first letters. The Rust uses the 26.2+
abbreviation table `["MO", "C", "AM", "AX", "UWC", "WC", "WA",
"MI"]` (debug_overlay.rs:394) and a test pins it (:640: `"SC:
289, MO: 3, C: 10, AM: 1, AX: 0, UWC: 0, WC: 2, WA: 4, MI: 0"`).
Cosmetic; the counts themselves are correct.

### B11-G10 — Vanilla-written worlds load as void: chunk palette reader accepts only the post-5006 id form (major; = 26.1.2 B11-G10; REPRODUCES)

All three links of the 26.1.2 pattern verify for 1.21.10:

1. **Reader.** There is NO `BlockState::from_palette_nbt`
   anywhere in the crates; `BlockState::from_nbt` reads the
   strict `"id"` key only and errors "block state is missing
   id" (chunk.rs:208). A datafix key 5006 renaming Name→id
   exists (fixes_c.rs:159) but NEVER RUNS: `update()` runs
   fixes where `k/10 <= to_version` (datafix/mod.rs:223), key
   = version*10+sub (schema.rs:74), and CURRENT = DATA_VERSION
   = 4556 (version.rs:11), so 5006 > 4556 never runs. The
   datafix test `block_states_take_their_new_field_names`
   (datafix/tests.rs:66-87) documents Name/Properties as
   1.21.10's own form.
2. **Load path.** `upgrade_record` early-returns for DataVersion
   ≥ 4556 (save.rs:2440); `decode_column` maps unresolvable
   palette entries to air (save.rs:1788,
   `registry.saved_state_id(p).map_or(air, ...)`); and
   `saved_state_id` uses `BlockState::from_nbt(tag).ok()`
   (registry.rs:241) — so a `Name`-keyed palette entry fails
   to parse and becomes air.
3. **Reference world.** `~/workspace/vanilla-world-mc/1.21.10/"New
   World"`, seed 6826278913115239597 (level.dat
   WorldGenSettings), DataVersion 4556 — its region palettes
   are `Name`/`Properties` compounds (verified via nbt_probe.py
   on r.0.0.mca).

Consequence is statically derived, not demonstrated in-game: a
vanilla-written 1.21.10 world loads as void. Inherited from the
1.21.11 template (which also has 0 `from_palette_nbt`), NOT a
delta regression — 1.21.7 also has 0; only 26.1.1 has the reader
(6 occurrences).

### B11-G12 — Bonemealing a blocked sapling keeps the meal (minor; = 1.21.7 B11-G07 narrowed; REPRODUCES)

Vanilla 1.21.10's sapling `isValidBonemealTarget` is `iconst_1`
— always true (javap-verified). The Rust's `bonemeal_valid`
gates on grower/live_features/tree_can_grow (growth.rs:443),
which is false only for the lone-mega 2x2 case; `grow_crop`
then returns false and the meal is kept (bonemeal.rs:297-320).
The Rust's own comment says "always true" while the code gates.
Narrowed from 1.21.7: only the lone-mega case diverges.

### B11-G13 — Player data saved where vanilla 1.21.10 won't look (minor; = 1.21.7 B11-G14; REPRODUCES)

Vanilla 1.21.10's `LevelResource` contains the flat
`"playerdata"` path — there is no `"players/data"` in the jar
(javap-verified). The Rust writes player data to
`players/data/` (lan/playerdata.rs:18) — the 26.x layout. A
world the port saves has its player data in a directory vanilla
1.21.10 never reads.

### B11-G14 — Golem stroll roll order + snow-layer tag names (minor; = 1.21.7 B11-G16; REPRODUCES, both halves)

(a) Vanilla 1.21.10's golem `getPosition` draws two floats,
compares the SECOND to 0.3f and the FIRST to 0.7f (store-first
form, javap-verified). The Rust draws 0.3 first, 0.7 second
(golem.rs:152/155) — the roll order is swapped, so the stroll
distribution differs from vanilla's. (b) The Rust reads the
26.x snow tag names (`cannot_support_snow_layer`,
`support_override_snow_layer`) in four places (data.rs:431-432,
survive.rs:314/316, weather.rs:113/116, golem.rs:576/579) — both
absent from the 1.21.10 datapack. The 1.21.10 names
(`snow_layer_can_survive_on`, `snow_layer_cannot_survive_on`)
are PRESENT in the datapack (honey/soul sand/mud survive;
ice/packed ice/barrier do not) but never read.

### B11-G15 — Dialog item tooltip missing on keyboard focus (minor; = 1.21.7 B11-G22; REPRODUCES)

Vanilla 1.21.10's `ItemDisplayWidget` shows its tooltip on
`tooltip && isHoveredOrFocused()` (javap offsets 97/104) —
hover OR keyboard focus. The Rust's dialog item part shows the
tooltip on hover only (dialogs.rs:705: `*show_tooltip &&
in_area(cursor.1) && ui::hovered(...)`); dialog focus tracks
text inputs only, so a keyboard-focused item never shows its
tooltip.

### B11-G16 — Washing dyed leather armour never works (minor; = 1.21.7 B11-G23; REPRODUCES)

Vanilla 1.21.10's `CauldronInteraction` has explicit entries
for LEATHER_BOOTS/LEGGINGS/CHESTPLATE/HELMET,
LEATHER_HORSE_ARMOR, and WOLF_ARMOR (javap-verified); there is
no `cauldron_can_remove_dye` tag in the datapack. The Rust's
cauldron wash routes through `removes_dye`, which reads that
absent tag (cauldron.rs:8, :217) — and unlike banners and
shulker boxes, which DO have explicit arms (cauldron.rs:318-339),
leather has no explicit arm at all. Dyed leather armour can
never be washed.

### B11-G17 — F3 chart footer omits the "For help" suffix (minor, cosmetic; NEW)

Vanilla 1.21.10's `DebugScreenOverlay` footer (javap-verified)
is the one-line chart template `Debug charts: [F3+1] Profiler
\x01; [F3+2] \x01\x01; [F3+3] \x01\x01` (StringConcatFactory
template, `javap -v`) with UPPERCASE chart names `FPS + TPS` /
`FPS` — which the Rust matches exactly
(debug_keys.rs:254-295, :284) — plus a second line that reads
`"To edit: press F3 + F6. For help: press F3 + Q"` (offset 749)
when in a level with chat allowed, else `"To edit: press F3 +
F6"` (offset 763). The Rust's second line is always `"To edit:
press {}"` — the `. For help: press F3 + Q` suffix is missing
in the common case. Cosmetic; the 1.21.7 P01 question (chart
names/layout) is settled FIXED — this is a new, smaller delta.

### B11-G18 — horizon-prepare's survival table binds dead class names (minor; NEW)

`horizon-prepare`'s survival.rs CLASS_RULES maps `"FarmBlock"`
(:31, fixed with a 1.21.10 comment) but still `"PathBlock"`
(:10), `"SpeleothemBlock"` (:21), and `"RedstoneWireBlock"`
(:45) — all three dead names for 1.21.10 (the jar has
DirtPathBlock, PointedDripstoneBlock, RedStoneWireBlock).
`bind_definitions` silently skips unmatched classes, so dirt
paths, redstone wire, and pointed dripstone bind NO survival
rule at prepare time. (The VEGETATION_RULES
NetherFungusBlock/NetherRootsBlock fallback is deliberate —
hardcoded mayPlaceOn, item 757 — not a defect.)

### B11-G19 — /worldborder setSize feedback uses the raw double instead of %.1f (minor; ledger item 592; NEW — possible Batch 9 overlap, flag only)

Item 592's port is half right: the stopwatch removal is verified
(0 stopwatch classes in either jar; stopwatches.rs deleted; 0
hits in commands/tree files) and the worldborder integer time
args are verified (tree_world.rs:333-337 `integer(0)`; javap
IntegerArgumentType.integer(0)). But the setSize feedback
sub-port is inverted: BOTH 1.21.10 and 1.21.11 format `%.1f`
(1.21.10 in lambdas lambda$setSize$16/17/18 via
String.format(Locale.ROOT, "%.1f"); 1.21.11 in setSize itself,
offsets 65-79, passing a pre-formatted String). The Rust's
terrain_cmds.rs:441-445 uses `double_to_string` with the wrong
comment "1.21.10 passes the raw double". The feedback text
therefore prints the unformatted double where vanilla prints
one decimal place.

## §2 Oracle defects

No oracle repairs were made (Batch 0's scope decision:
measurement + triage only; the oracle `.java` files live under
`versions/1.21.10/`). Batch 0 has NOT landed for 1.21.10, so
there is no landed oracle table to triage against — both
defects below were verified directly against the 1.21.10 jars
with javap, and the triage is provisional until Batch 0 lands.

- **B11-O01 — VanillaCauldronOracle uses a 3-arg drop that does
  not exist in 1.21.10 (minor; = 1.21.7 B11-O01, = 26.1.2
  B11-O01; REPRODUCES).** VanillaCauldronOracle.java:118 calls
  the 3-arg `drop(ItemStack, boolean, Prediction)`; but
  `net.minecraft.util.Prediction` is ABSENT from both 1.21.10
  jars, and `Player.drop` is 2-arg. The oracle cannot compile
  against 1.21.10. Repair (not done here): drop the
  `Prediction` parameter, as 1.21.7's B11-O01 did (commit
  c754e3a2 there). This oracle is the driver behind the
  cauldron table comparison (1.21.7's B11-H02, carried here as
  B11-H02); with B11-G16 live, the comparison is confirmatory
  for a real defect — but the oracle remains broken for any
  cauldron driver work until repaired.
- **B11-O02 — VanillaLavaCauldronItemOracle overrides a method
  that does not exist in 1.21.10 (minor; = 26.1.2 B11-O02;
  REPRODUCES).** VanillaLavaCauldronItemOracle.java:28 overrides
  `getNextEntityId()`; that method is absent from both `Level`
  and `ServerLevel` in the 1.21.10 server jar (javap-verified).
  The oracle cannot compile against 1.21.10. Repair (not done
  here): remove the override.

## §3 Needs-new-harness

- **B11-H01 — Class-table chain-liveness half (minor).** The
  jar half of §6.1 ran (265/271 live); the chain half needs
  `block-behaviours.json`, which does not exist for 1.21.10
  anywhere (the worktree's `.local/minecraft/1.21.10` is an
  empty dir; only 1.21.7's exists, at
  `~/workspace/mcir-work/mob-spawn/resources/`). Batch 0 has
  NOT landed for 1.21.10 (b0 dir empty; no
  `batch-0-oracles.json`, no prepared-data section), so the
  chain half cannot run. Recorded, not guessed — same as the
  26.2 run's B11-H01, which the 26.1.2 run had closed. What
  would settle it: Batch 0's prepared data landing, then the
  chain walk over `block-behaviours.json`.
- **B11-H02 — Cauldron interaction table vs Rust (minor;
  = 1.21.7 B11-H02; CARRIES).** The exhaustive driver
  comparison of the cauldron table against the Rust was BLOCKED
  on resources in 1.21.7 (confirmatory there). For 1.21.10 it
  is confirmatory for a LIVE defect (B11-G16: leather wash
  dead) — but it is also blocked on the oracles: B11-O01 and
  B11-O02 are both broken against the 1.21.10 jars, so the
  driver cannot compile until they are repaired. What would
  settle it: oracle repair (Batch 0's scope) + the driver run.

## §4 Needs-in-game-probe

None. The 1.21.7 P01 probe (F3 chart footer layout) is settled
by the 1.21.10 jar (§6.6): the footer template, chart names,
and visibility words are all javap-verified, and the Rust
matches them except for the missing "For help" suffix (B11-G17,
a static finding — no probe needed to confirm a missing string).

## §5 Accepted gaps / notes (not defects)

The 1.21.10 port ledger carries NO accepted-gap verdicts, so
none of these is a ledger item and none is counted in the
211-item tally. All three are notes carried from the 1.21.7
sweep, re-verified against 1.21.10:

- **B11-A01 — ProvidesTrimMaterial inline/direct form [=
  1.21.7 B11-A01, = 26.1.2 B11-A01].** Vanilla 1.21.10's
  `ProvidesTrimMaterial.material` is `EitherHolder<TrimMaterial>`
  (javap-verified) — a Holder direct OR a ResourceKey string.
  The Rust reads the string form only (enchant.rs:335-337,
  `.as_str()`). All 11 vanilla 1.21.10 items with the component
  use string keys (datagen reports/items.json: amethyst shard,
  copper ingot, diamond, emerald, gold ingot, iron ingot, lapis
  lazuli, netherite ingot, quartz, redstone, resin brick — all
  values verified strings) — the direct/inline form is
  datapack-only.
- **B11-A02 — Special-foil decal not modelled [= 1.21.7
  B11-A03, = 26.1.2 B11-A02].** The Rust HAS a `Foil::Special`
  variant ("compasses and clocks, a sheeted decal",
  item_render.rs:149-150) carried through `submit_quads`
  (:453), but both sinks treat every non-None foil identically:
  the GUI sink draws the glint over the full quad
  (item_render/gui.rs:422) and the world sink likewise
  (item_render.rs:1163) — no 0.5x GUI / 0.75x first-person
  decal scaling. Only visible on a command-enchanted
  compass/clock. Pre-existing in both versions, not a version
  delta.
- **B11-A03 — NBT separator ignored + per-player resolution [=
  1.21.7 B11-A04, = 26.1.2 B11-A03].** The Rust's
  `component_arg` (game/commands/text_commands.rs:72-135)
  resolves selectors ONCE for the command source and always
  joins names with `", "` (:131, :241) — the separator field of
  nbt/selector components is ignored and a title/tellraw
  component is not re-resolved per player (vanilla resolves
  per player). Gaps from BOTH versions equally, not version
  deltas.

NOT carried: 1.21.7's B11-A02 (NBT `sound_variant` written on
non-wolf mobs) is FIXED for 1.21.10 — vanilla 1.21.10 has sound
variants for wolves ONLY (only `WolfSoundVariants` exists in
the server jar; Chicken/Cow/Cat/PigSoundVariants are ABSENT),
and the Rust's `sound_variants` (mobs/finalize.rs:2293-2297)
lists exactly Wolf, with the write side gated on
`!sounds.is_empty()` (entity_nbt.rs:888-893) and the read side
resolving to None for non-wolves (:2338-2341). Writing the tag
is correct 1.21.10 behaviour for wolves and impossible for
anything else.

## §6 Systemic findings

### §6.1 The class table: 265/271 jar-live, 6 jar-dead — and the dead names are inert

Method: parse every `Class::X = "Name"` in
`versions/1.21.10/crates/horizon-world/src/level/classes.rs`
(271 mappings, 7 dual-name `| "Alt"`), check each name against
both 1.21.10 jar indexes, and check arms by grep. **265
jar-live, 6 jar-dead**, all 6 present in the 26.3 jar:
AbstractBed (`AbstractBedBlock`), StrawBed, Speleothem
(`SpeleothemBlock`), SulfurSpike, ShelfMushroom, PotentSulfur.
All 7 duals live via their 1.21.10 alt: FarmBlock, SnowyDirtBlock,
SpreadingSnowyDirtBlock, FungusBlock, RootsBlock, DirtPathBlock,
RedStoneWireBlock (each 26.3-name absent from the 1.21.10 jars).

Unlike 26.1.2 (8 dead, 4 with live arms on blocks that exist),
1.21.10's dead names cause NO behaviour defect:

- The two RENAME cases are paired with their live twins in
  every arm: AbstractBed's 5 arms all dispatch
  `Class::AbstractBed | Class::Bed` (hooks.rs:469,
  interact.rs:365, place.rs:356, place.rs:1483, shapes.rs:124);
  Speleothem's 5 arms all dispatch `Class::Speleothem |
  Class::PointedDripstone` (place.rs:1156, shapes.rs:745,
  survive.rs:326, tick.rs:257/443). The arms work via the live
  twin. (Bed arm-logic nuance only: the Rust uses the pair-check
  form; vanilla 1.21.10's `BedBlock.useWithoutItem` redirects
  foot→head requiring only `is(this)` — Batch 5's B5-G27 layer,
  noted here, not minted.)
- The four 26.x-ONLY cases have sole-dispatch arms but no
  blocks to fire on: StrawBed (update.rs:429), SulfurSpike
  (speleothem.rs:113), ShelfMushroom (bonemeal.rs:94 sole;
  place.rs:493, shapes.rs:137, survive.rs:42 paired),
  PotentSulfur (hooks.rs:209, pistons.rs:154, place.rs:1193,
  shapes.rs:836 — all sole). Straw beds, sulfur spikes, shelf
  mushrooms, and potent sulfur do not exist in 1.21.10, so the
  arms are dead code, not defects.

Chain half: NOT run — B11-H01 (no block-behaviours.json; §3).
The jar half alone cannot prove chain-liveness, but the dual
names + live twins make a chain-side surprise unlikely for the
rename cases.

### §6.2 The 26.x tag reads: six absent tags still read

The port reads six tag names that do not exist in 1.21.10's
datapack (626 tag files, checked by name):

| Tag read by the Rust | 1.21.10 status | Effect |
|---|---|---|
| `speleothems` | ABSENT | Dripstone placement/growth dead (B11-G04) |
| `cauldron_can_remove_dye` | ABSENT | Leather wash dead (B11-G16) |
| potion tags (any) | ABSENT (0 files, no PotionTags class) | Splash water dead (B11-G05) |
| `cannot_support_snow_layer` / `support_override_snow_layer` | ABSENT (1.21.10 names present, unread) | Golem/snow behaviour wrong (B11-G14) |
| `sculk_growth_inhibitors` | ABSENT | (No Rust read verified in this batch's scope; noted for Batch 4) |
| `supports_chorus_plant` / `supports_chorus_flower` | ABSENT | No effect — the Rust hardcodes end stone (G21 FIXED) |

Also absent: `overrides_mushroom_light_requirement` (declared
in data.rs:453 but never used — no effect), `causes_suffocation`
/ `suppresses_bounce`. Present and correctly used:
`mushroom_grow_block` (survive.rs:285),
`convertable_to_mud` (note the vanilla spelling with an 'a';
`convertible_to_mud` is ABSENT), `dripstone_replaceable_blocks`,
`snow_layer_can_survive_on` / `snow_layer_cannot_survive_on`
(present but unread — see B11-G14).

### §6.3 The 26.x component reads: five absent components

`DataComponents` in the 1.21.10 server jar has NO COOKING_FUEL,
brewing_fuel, compostable, villager_food, or mob_visibility
(javap-verified). The Rust reads `minecraft:cooking_fuel`
(items.rs:1044) and `minecraft:brewing_fuel` (items.rs:1046)
for every item — both always None on 1.21.10 data, which is
the B11-G07 fuel defect and the B4-G01 furnace consequence.
The same pattern (26.x component read against a version that
never defined it) is the component-side twin of §6.2.

### §6.4 The fluid floor does NOT apply here

The 1.21.7 sweep's §6.4 warned that the 1e-5 fluid-push floor
was a re-base risk. For 1.21.10 it is not: 1.21.10's `Entity`
has the 1e-5 float constant but NOT in
`updateFluidHeightAndDoFluidPushing` — the push gate there is
`length() <= 0`. The Rust removed the floor
(entity_fluid.rs:88), which matches 1.21.10 exactly. No defect.

### §6.5 The evidence-token check: 19 26.3-only names, all cited as absent

Method: extract every CamelCase token from the 211 ledger
evidences (263 distinct), check each against the 1.21.10 jar
indexes (both jars, top-level + inner classes) and the 26.3
client jar. **132 tokens live in the 1.21.10 jars; 19 are
26.3-only** — and EVERY 26.3-only citation is a "removed in
1.21.10" / "rename" / "dead on 1.21.10 data" claim, verified
correct: AbstractNautilus, CamelHusk, Nautilus, Parched,
ZombieNautilus (item 587/588 — 0 Nautilus classes in the
1.21.10 jars); StopwatchCommand, Stopwatches (item 592 — 0
stopwatch classes); SwingAnimation, KineticWeapon, UseEffects
(item 491 — removed hash functions); AnyValue, TypeBase,
ConcreteType (item 549 — internal Java API refactor);
Identifier, IdentifierArgument, IdentifierException (items
480/568/606/675 — renames to ResourceLocation*, verified live
in the 1.21.10 jars); PermissionSet (items 489/502/524 — int
levels in 1.21.10); RenderTypes (item 501 — RenderType in
1.21.10); OutboundPlayer (item 465 — PlayerInfo in 1.21.10).
No evidence claims a 26.3-only class exists in 1.21.10. The
remaining tokens are JDK/library classes (DataOutputStream,
HttpClient, Optional...), Rust-side names, or English words.

### §6.6 The P01 settlement: footer template matches, one suffix missing

Vanilla 1.21.10's `DebugScreenOverlay` builds its chart footer
from the StringConcatFactory template `Debug charts: [F3+1]
Profiler \x01; [F3+2] \x01\x01; [F3+3] \x01\x01`
(`javap -v`, constant #474) with chart names `FPS + TPS` /
`FPS` (UPPERCASE, offsets 608/614) and visibility words
` visible` / ` hidden` (offsets 659/665). The Rust's
`debug_chart_lines` (debug_keys.rs:254-295) reproduces the
template, the names, and the visibility words exactly — the
1.21.7 P01 question is settled FIXED, and the 26.1.2 G08
(lowercase fps) does not reproduce. The one delta is the
second line: vanilla appends `. For help: press F3 + Q` when
in a level with chat allowed (offset 749); the Rust never does
(B11-G17).

### §6.7 Ledger-evidence staleness: the 592 stopwatch deletion outran the ledger

Three ledger entries carry claims the 592 stopwatch deletion
falsified after (or without) updating them: item 532 claims
"Module identical (0 diff lines)" for misc.rs, which lost the
52-line `cmd_stopwatch`; item 662 claims the same for
stopwatches.rs, which was deleted outright; item 641 claims
"comment-only" for registries.rs, which lost the 1-line
`stopwatches` field. The deletion itself is correct and
verified (0 stopwatch classes in the 1.21.10 jars) — the
staleness is bookkeeping, not behaviour. Items 532/662 are
triaged gap on that basis (their no-change verdicts do not
hold against the current tree); item 641 passes (its
carry-over verdict holds; the stale phrase is one line).
Lesson for the coordinator: ledger entries that claim "0 diff
lines" should be re-checked after any multi-file port lands.

## §7 Method notes

- **Jars:** `~/workspace/mcir-work/e6-deltas/versions/1.21.10/jars/client-named.jar`
  (9,969 classes) and `server-game-named.jar` (6,964 classes);
  26.3 client jar `~/workspace/mcir-work/jars-26.3/client-26.3.jar`
  (11,383 classes) for absence checks only; 1.21.11 jars at
  `~/workspace/mcir-work/e6-deltas/versions/1.21.11/jars/` for
  the 592 comparison. Jar indexes + extracted classes + all
  javap outputs are in `b11/jar-classes.json`,
  `b11/jar-classes/`, `b11/javap/`.
- **javap is slow** (~15s per JVM invocation even on extracted
  dirs) — multi-class invocations were batched throughout.
- **No prepared data for 1.21.10:** Batch 0 has not landed
  (b0 dir empty; no `batch-0-oracles.json`, no prepared-data
  section). No `block-behaviours.json` exists for 1.21.10
  anywhere (worktree `.local/minecraft/1.21.10` is an empty
  dir). Data checks used the client-jar datapack directly
  (626 tag files; 1,325 loot-table files) and the e6-deltas
  datagen reports (`datagen/reports/items.json`, 1,488 items).
  Oracle triage is therefore provisional + direct javap
  verification, per the brief.
- **Reference world:** `~/workspace/vanilla-world-mc/1.21.10/"New
  World"`, seed 6826278913115239597 (level.dat WorldGenSettings),
  DataVersion 4556; palettes checked with nbt_probe.py (copied
  from the 26.1.2 batch dir) on r.0.0.mca.
- **CI:** crate-tests run 45376855229 (head b3cd0028), fully
  green, 6,995 passed / 0 failed — verified from the run log,
  not from PORT-NOTES alone.
- **Diff classification:** every ledger module was diffed
  against the 1.21.11 tree in the same worktree
  (`versions/1.21.11/`), classified identical / comment-only /
  code-diff / multi-file / missing-file
  (`b11/diff-classification.json`), and every code-diff and
  multi-file item was individually reviewed against its
  evidence. The 20-item random spot-check (seed 11) agreed
  with the classification in all 20 cases.
- **Environment:** RUSTUP_HOME/CARGO_HOME under
  `~/workspace/tools/`, `~/workspace/tools/jdk-25/bin` on PATH.
  Two exec calls failed transiently mid-session (session
  metadata timeout; service restart) — retries succeeded,
  nothing lost.
## §8 Per-item triage — the 211 rank-3 ledger items

All 211 items in `versions/1.21.10/rank3-progress.json`, triaged against the 1.21.10 jars and the current tree (2026-11-25). The 11 ported items were individually re-verified (javap + Rust grep); the no-change/carry-over items were triaged by a full automated diff classification against the 1.21.11 tree (123 identical, 19 comment-only, 8 code-diff, 1 missing-file for no-change; 15 identical, 18 comment-only, 7 code-diff, 9 multi-file for carry-over) plus a 20-item random spot-check and individual review of every code-diff and multi-file item. The three gaps are item 592 (a real behaviour defect, B11-G19) and items 532/662 (ledger-evidence staleness from the 592 stopwatch deletion — the behaviour is correct).

| Item | Module | Ledger verdict | Triage | Evidence (all verified 2026-11-25) |
|---|---|---|---|---|
| 465 | crates/horizon-client/src/realms/client.rs | no-change | **pass** | Reviewed: 6-line diff is test-only version labels (1.21.11 to 1.21.10, protocol 774 to 773); the getDateOr deviation is documented in the evidence as accepted (wall-clock default non-deterministic). |
| 466 | crates/horizon-client-render/src/post_chain.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("TextureInput ctor/filter plumbing in post_chain: 1.21.11's `PostChain` passes texture inputs differently, but the Rust's post-chain is Horizon's own..."). |
| 467 | crates/horizon-client/src/game/paintings.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("GameRules rename only: `ENTITY_DROPS` (1.21.11) vs `RULE_DOENTITYDROPS` (1.21.10) in the paintings drop check; the Rust reads the rule through its..."). |
| 468 | crates/horizon-client/src/mobs/polar_bear.rs | no-change | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("7 named method(s), all same after rename normalisation: net/minecraft/world/entity/animal/PolarBear#getDefaultDimensions,..."). |
| 469 | crates/horizon-server/src/management.rs | ported | **pass** | Javap: `validateAuthentication` is Bearer-only (startsWith "Bearer", trim, `isValidApiKey` via MessageDigest.isEqual), single denial "Invalid or missing API key", no origin/subprotocol check; SecurityConfig has only secretKey. Rust management.rs:423 strip_prefix, :416 message, :456-461 minecraft-v1 echo. |
| 470 | crates/horizon-client/src/game/trident.rs | no-change | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("All six `ThrownTrident` methods (addAdditionalSaveData, findHitEntity, playerTouch, readAdditionalSaveData, tick, tryPickup) have EMPTY diffs — false..."). |
| 471 | crates/horizon-client/src/game/command_blocks.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("`BaseCommandBlock#performCommand`: the 1.21.10 `isClientSide` gate and the `isCommandBlockEnabled` read from the server (vs the level) are the same..."). |
| 472 | crates/horizon-client/src/mobs/illusioner.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("`Illusioner#aiStep` and `#finalizeSpawn` diffs are EMPTY; `registerGoals` is a package move (`monster/illager/` vs `monster/`), same goals in the..."). |
| 473 | crates/horizon-client/src/game/commands/items.rs | no-change | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("`ItemCommands#getContainer` is a control-flow reorder with the same outcome. `SlotAccess$1/$2/$3#set` are anonymous-class renumbering: the same four..."). |
| 474 | crates/horizon-client/src/block_pattern.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Golem patterns: 1.21.11 wraps the state predicate in `BlockInWorld.hasState`, 1.21.10 passes the raw predicate — the same matching...."). |
| 475 | crates/horizon-client/src/game/dedicated/management.rs | ported | **pass** | Javap: UntypedRule is a String key/value record. Rust dedicated/management.rs parses string values (Boolean.parseBoolean / Integer.parseInt semantics, "For input string" error). |
| 476 | crates/horizon-client/src/realms/download.rs | no-change | **pass** | Spot-checked: byte-identical to 1.21.11. |
| 477 | crates/horizon-client/src/game/test_instances.rs | ported | **pass** | Javap: highlightPos fill -2147418368 (0x8000FF00). Rust HIGHLIGHT_FILL = 0x8000_FF00 (test_instances.rs:32). |
| 478 | crates/horizon-assets/src/bake.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("`FaceBakery#bakeQuad`/`calculateFacing` are restructured in 1.21.11 (inline vertex baking with FaceInfo/bakeVertex; `GeometryUtils.normal` +..."). |
| 479 | crates/horizon-client/src/multiplayer.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("The Rust's server-list hover helpers (`over_right_half`/`over_top_left`/`over_bottom_left`) already use the 1.21.10 inline float regions (16.0/32.0);..."). |
| 480 | crates/horizon-client/src/game/commands/place.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("`IdentifierException` vs `ResourceLocationException` — a rename, same error."). |
| 481 | crates/horizon-client/src/game/item_use.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("`renderArmWithItem`: the `applyItemArmTransform` placement move and the `swingArm` extra float are vanilla render plumbing; the Rust's first-person..."). |
| 482 | crates/horizon-world/src/datafix/world_upgrader.rs | no-change | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("1 named method(s), all same after rename normalisation: net/minecraft/util/worldupdate/WorldUpgrader#work. No behaviour diff 1.21.11->1.21.10."). |
| 483 | crates/horizon-client/src/game/respawn_anchor.rs | no-change | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("`canSetSpawn` already reads the dimension type's `respawnAnchorWorks` — the 1.21.10 form (1.21.11 reads the `RESPAWN_ANCHOR_WORKS` environment..."). |
| 484 | crates/horizon-client/src/spear.rs | no-change | **pass** | Reviewed: 34-line diff is the 1.21.11 spear-value removal (dismount thresholds back to 13/12/11/13/10/9, sway ends at finish_swaying, 1.21.11-only tests removed). Spears do not exist in 1.21.10 (0 spear classes, 0 Kinetic classes in either jar) — the module is dead code; the review method's diff is empty as claimed. |
| 485 | crates/horizon-client/src/shadows.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module. Shadow-size name table includes..."). |
| 486 | crates/horizon-client/src/game/jigsaw_blocks.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("2 named method(s), all same after rename normalisation: net/minecraft/world/level/block/entity/JigsawBlockEntity#generate,..."). |
| 487 | crates/horizon-world/src/worldgen/structure/simple.rs | no-change | **pass** | Spot-checked: differs only in comments (version labels). |
| 488 | crates/horizon-client/src/game/commands/test_locate.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("2 named method(s), all same after rename normalisation: net/minecraft/gametest/framework/TestCommand#createNewStructure,..."). |
| 489 | crates/horizon-client/src/game/command_minecarts.rs | no-change | **pass** | Spot-checked: byte-identical to 1.21.11. |
| 490 | crates/horizon-client/src/item_render/gui.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("2 named method(s), all same after rename normalisation: net/minecraft/client/gui/render/pip/GuiBannerResultRenderer#renderToTexture,..."). |
| 491 | crates/horizon-protocol/src/item/hash/gameplay.rs | carry-over | **pass** | Reviewed: 87-line diff is the 1.21.11-only hash-function removal (swing/kinetic/use_effects) + component-id renumbering, as the evidence states; verified: no swing/kinetic/use_effects in 1.21.10's DataComponents. |
| 492 | crates/horizon-world/src/worldgen/structure/ruined_portal.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("2 named method(s), all same after rename normalisation: net/minecraft/world/level/levelgen/structure/structures/RuinedPortalPiece#makeSettings,..."). |
| 493 | crates/horizon-client-render/src/hud/pack_text.rs | no-change | **pass** | Spot-checked: byte-identical to 1.21.11. |
| 494 | crates/horizon-client-render/src/zip.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("2 named method(s), all same after rename normalisation: net/minecraft/server/packs/FilePackResources#getResource,..."). |
| 495 | crates/horizon-client/src/game/wandering_trader.rs | no-change | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("`WanderingTraderSpawner#spawn` diff is EMPTY; `#tick` is a GameRules rename (`SPAWN_WANDERING_TRADERS` vs `RULE_DO_TRADER_SPAWNING`)."). |
| 496 | crates/horizon-client/src/mobs/phantom.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("f32-vs-double trig noise, sub-rounding."). |
| 497 | crates/horizon-client/src/toasts.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("`getCurrentSongName` vs the music-manager key — the same value."). |
| 498 | crates/horizon-world/src/worldgen/generator.rs | no-change | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("2 named method(s), all same after rename normalisation: net/minecraft/world/level/levelgen/structure/pools/JigsawPlacement#addPieces,..."). |
| 499 | crates/horizon-client-render/src/gfx/pack_atlas/stitch.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); its single named vanilla method is unchanged 1.21.11->1.21.10 (named_changed..."). |
| 500 | crates/horizon-client-render/src/trims.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 501 | crates/horizon-client/src/game/cartography.rs | no-change | **pass** | Spot-checked: differs only in comments (version labels). |
| 502 | crates/horizon-client/src/game/test_environments.rs | no-change | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("GAMEMASTER PermissionSet vs permission level 2 — the same level."). |
| 503 | crates/horizon-client/src/mobs/trader.rs | no-change | **pass** | Spot-checked: byte-identical to 1.21.11. |
| 504 | crates/horizon-client/src/realms/upload.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("`worldClosed()` vs `isWorldClosed()` — a rename."). |
| 505 | crates/horizon-client/src/game/debug_options.rs | ported | **pass** | Javap: getRowWidth returns 310. Rust ROW_WIDTH = 310.0 (debug_options.rs:21). |
| 506 | crates/horizon-client/src/game/initial_spawn.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("GameRules rename: `RESPAWN_RADIUS` vs `RULE_SPAWN_RADIUS`."). |
| 507 | crates/horizon-client/src/mobs/dolphin.rs | carry-over | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Diff vs the 1.21.11 tree is comment-only (version labels); no vanilla method is named in the module. The nautilus vehicle check is dead on 1.21.10..."). |
| 508 | crates/horizon-client/src/mobs/enderman.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Both enderman goals are a GameRules rename (`MOB_GRIEFING` vs `RULE_MOBGRIEFING`)."). |
| 509 | crates/horizon-client/src/game/dispenser.rs | no-change | **pass** | Reviewed: 2-line diff is the `spawn_jockeys(id, false)` to `spawn_jockeys(id)` signature change (1.21.10 has no natural-spawn flag); the review method's diff is empty as claimed. |
| 510 | crates/horizon-client-render/src/gfx/panorama.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Render-pass plumbing (`writeTransform`/`bindSampler`) in the panorama renderer; the Rust's panorama is its own implementation."). |
| 511 | crates/horizon-client/src/item_sync.rs | carry-over | **pass** | Reviewed: 58-line diff is the painting/variant id 89 + sign-text test removal, as the evidence states. |
| 512 | crates/horizon-client/src/game/commands/data.rs | no-change | **pass** | Reviewed: 40-line diff is the already-ported item-11/item-20 anger work, as the evidence states. |
| 513 | crates/horizon-client/src/game/gamemode_switcher.rs | ported | **pass** | Javap: GameModeSwitcherScreen has NO mouseReleased (only init/render/keyPressed/isPauseScreen). Rust `switcher_release`/`release_applies` 0 hits. |
| 514 | crates/horizon-client/src/game/native_client_services.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("`isOverlayVisible` vs `isF3Visible` — a rename, same flag."). |
| 515 | crates/horizon-client/src/mobs/sniffer.rs | no-change | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("1 named method(s), all same after rename normalisation: net/minecraft/client/model/SnifferModel#setupAnim. No behaviour diff 1.21.11->1.21.10."). |
| 516 | crates/horizon-client/src/game/advancements_screen.rs | carry-over | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Diff vs the 1.21.11 tree is comment-only (version labels); no vanilla method is named in the module."). |
| 517 | crates/horizon-client/src/title/flat.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("1 named method(s), all same after rename normalisation: net/minecraft/client/gui/screens/PresetFlatWorldScreen#fromString. No behaviour diff..."). |
| 518 | crates/horizon-world/src/loot/odds.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 519 | crates/horizon-client/src/game/block_editor.rs | carry-over | **pass** | Spot-checked: byte-identical to 1.21.11. |
| 520 | crates/horizon-client/src/game/dialogs.rs | no-change | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("1 named method, missing-new: AbstractStringWidget#setComponentClickHandler is 1.21.11-only (1.21.10's AbstractStringWidget has no click-handler API),..."). |
| 521 | crates/horizon-client/src/game/zombie_villagers.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 522 | crates/horizon-command/src/network.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("1 named method(s), all same after rename normalisation: net/minecraft/commands/synchronization/SuggestionProviders#getProvider. No behaviour diff..."). |
| 523 | crates/horizon-client-core/src/pattern.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 524 | crates/horizon-client/src/game/commands/profiles.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("LEVEL_ADMINS PermissionSet vs permission level 3 — the same level."). |
| 525 | crates/horizon-client/src/game/resource_loader.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 526 | crates/horizon-client/src/game/tutorial.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Instruction reordering in the tutorial, same values in the same order of effect."). |
| 527 | crates/horizon-client-render/src/gfx/native_material.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("1 named method, missing-new: SectionRenderDispatcher$RenderSection#getVisibility is 1.21.11-only. The Rust module (native_material.rs) never..."). |
| 528 | crates/horizon-client/src/game/clone_dimensions_tests.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("1 named method(s), all same after rename normalisation: net/minecraft/commands/arguments/DimensionArgument#getDimension. No behaviour diff..."). |
| 529 | crates/horizon-client/src/mobs/hostile.rs | no-change | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("1 named method(s), all same after rename normalisation: net/minecraft/world/entity/monster/Spider$SpiderAttackGoal#canUse. No behaviour diff..."). |
| 530 | crates/horizon-world/src/level/fluids.rs | no-change | **pass** | Spot-checked: differs only in comments (version labels). |
| 531 | crates/horizon-client-core/src/component.rs | carry-over | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Diff vs the 1.21.11 tree is comment-only (version labels); no vanilla method is named in the module."). |
| 532 | crates/horizon-client/src/game/commands/misc.rs | no-change | **gap** | GAP (ledger evidence, not behaviour): evidence claims "Module identical (0 diff lines)" but the module differs by 52 lines — the `cmd_stopwatch` deletion, part of item 592's port (verified correct: 0 stopwatch classes in the 1.21.10 jars). The no-change verdict does not hold against the current tree; the ledger entry is stale. |
| 533 | crates/horizon-client/src/game/debug_lines.rs | no-change | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("1 named method(s), all same after rename normalisation: net/minecraft/client/gui/components/debug/DebugEntryChunkGeneration#display. No behaviour..."). |
| 534 | crates/horizon-client/src/game/mount_menu.rs | carry-over | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Diff vs the 1.21.11 tree is comment-only (version labels); no vanilla method is named in the module. The nautilus slot checks are dead on 1.21.10..."). |
| 535 | crates/horizon-client/src/game/totem.rs | no-change | **pass** | Spot-checked: byte-identical to 1.21.11. |
| 536 | crates/horizon-client/src/locator_bar.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("1 named method(s), all same after rename normalisation: net/minecraft/client/resources/WaypointStyle#sprite. No behaviour diff 1.21.11->1.21.10."). |
| 537 | crates/horizon-client/src/mobs/creaking.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("1 named method(s), all same after rename normalisation: net/minecraft/client/model/CreakingModel#setupAnim. No behaviour diff 1.21.11->1.21.10."). |
| 538 | crates/horizon-world/src/worldgen/structure/nether_fossil.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("1 named method(s), all same after rename normalisation: net/minecraft/world/level/levelgen/structure/structures/NetherFossilPieces#addPieces. No..."). |
| 539 | crates/horizon-client/src/debug_overlay.rs | carry-over | **pass** | Spot-checked: differs only in comments (version labels). |
| 540 | crates/horizon-client/src/game/crossbow.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("`Stray#getArrow` diff is EMPTY — a false positive."). |
| 541 | crates/horizon-client/src/game/golem_defense_tests.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("`DefendVillageTargetGoal#canUse` diff is EMPTY — a false positive."). |
| 542 | crates/horizon-client/src/game/mace.rs | no-change | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("1 named method, missing-new: MaceItem#getItemDamageSource is 1.21.11's rename of 1.21.10's MaceItem#getDamageSource (javap-verified: canSmashAttack..."). |
| 543 | crates/horizon-client/src/game/stats_screen.rs | carry-over | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Diff vs the 1.21.11 tree is comment-only (version labels); no vanilla method is named in the module. The nautilus_one_cm stat name sits in a format..."). |
| 544 | crates/horizon-client/src/item_render/glint_atlas.rs | no-change | **pass** | Reviewed: 5-line diff is the already-ported item-91 mipmap work, as the evidence states. |
| 545 | crates/horizon-client/src/lan/guest.rs | carry-over | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module. The 1.21.11 behaviour is already..."). |
| 546 | crates/horizon-client/src/title/tests.rs | no-change | **pass** | Reviewed: 14-line diff is test-only version-string substitutions (1.21.11 to 1.21.10) + comment labels. |
| 547 | crates/horizon-server/src/management_methods.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("1 named method, missing-new: MinecraftServerSettingsServiceImpl#getOperatorUserPermissions is 1.21.11-only vanilla service API. The Rust's management..."). |
| 548 | crates/horizon-server/src/query.rs | ported | **pass** | Javap: writeString = DataOutputStream.writeBytes. Rust query.rs:154 + test:390 asserting 0xA7. |
| 549 | widely named: core/component/predicates | carry-over | **pass** | Reviewed (multi-file): AnyValue/TypeBase/ConcreteType are 26.3-only (§6.5); EnchantmentsPredicate is live in the 1.21.10 jars; the Rust models enchantment matching only. |
| 550 | crates/horizon-client-render/src/ban_notice.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("`getBanReasonText` is a `Component.copy()` + `MutableComponent` signature change; the rendered ban text is identical."). |
| 551 | crates/horizon-client-render/src/multiline_edit.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("`hasControlDownWithQuirk` vs `hasControlDown` — the same rename as item 13 (1.21.10's `hasControlDown` IS the quirk form, javap-verified); the file..."). |
| 552 | crates/horizon-client/src/dialog_screen.rs | carry-over | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module. The 1.21.11 behaviour is already..."). |
| 553 | crates/horizon-client/src/game/boat_passengers.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("`BoatItem#use` diff is EMPTY — a false positive."). |
| 554 | crates/horizon-client/src/game/debug_renderers.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 555 | crates/horizon-prepare/src/sounds.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("1 named method(s), all same after rename normalisation: net/minecraft/server/packs/repository/BuiltInPackSource#populatePackList. No behaviour diff..."). |
| 556 | crates/horizon-client-render/src/gfx/lightning.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("1 named method(s), all same after rename normalisation: net/minecraft/client/renderer/entity/LightningBoltRenderer#submit. No behaviour diff..."). |
| 557 | crates/horizon-client-render/src/level_loading.rs | no-change | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("The Rust fires `player_section_ready` when the player's chunk is lit (compiled) — no visibility check — which is 1.21.10's `isSectionCompiled` form..."). |
| 558 | crates/horizon-client/src/game/clone_block_ticks_tests.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("GameRules rename: `MAX_BLOCK_MODIFICATIONS` vs `RULE_COMMAND_MODIFICATION_BLOCK_LIMIT`."). |
| 559 | crates/horizon-client/src/game/spill.rs | no-change | **pass** | Reviewed: 7-line diff is already-ported work (batch-27 offsetRandom doc fix, create_experience random-param removal), as the evidence states. |
| 560 | crates/horizon-client/src/mobs/caravan.rs | carry-over | **pass** | Spot-checked: byte-identical to 1.21.11. |
| 561 | crates/horizon-client/src/title/buffet.rs | carry-over | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module. The 1.21.11 behaviour is already..."). |
| 562 | crates/horizon-world/src/worldgen/structure.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("1 named method(s), all same after rename normalisation: net/minecraft/world/level/block/Rotation#getRandom. No behaviour diff 1.21.11->1.21.10."). |
| 563 | crates/horizon-client/src/debug_chart.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("1 named method(s), all same after rename normalisation: net/minecraft/client/gui/components/debugchart/AbstractDebugChart#getSampleColor. No..."). |
| 564 | crates/horizon-client/src/game/brush.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("`criterion` vs `critereon` package rename."). |
| 565 | crates/horizon-client/src/game/end.rs | no-change | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("`EnderEyeItem#use` diff is EMPTY — a false positive."). |
| 566 | crates/horizon-client/src/game/item_draw.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("1 named method(s), all same after rename normalisation: net/minecraft/client/renderer/entity/ItemEntityRenderer#submit. No behaviour diff..."). |
| 567 | crates/horizon-command/src/vanilla.rs | ported | **pass** | Javap: TimeCommand is the simple set/add/query form with day/noon/night/midnight; no `world/timeline` package in either jar. Rust: `clock_nodes` 0 hits; vanilla.rs:146. |
| 568 | crates/horizon-protocol/src/packets/play/commands.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("`IdentifierArgument` vs `ResourceLocationArgument` — a class rename."). |
| 569 | crates/horizon-world/src/worldgen/ore_feature.rs | no-change | **pass** | Spot-checked: byte-identical to 1.21.11. |
| 570 | crates/horizon-world/src/worldgen/structure/mineshaft.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("`MineShaftCorridor#createChest` diff is EMPTY — a false positive."). |
| 571 | crates/horizon-world/src/worldgen/structure/temple.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("1 named method(s), all same after rename normalisation: net/minecraft/world/level/levelgen/structure/structures/DesertPyramidStructure#afterPlace. No..."). |
| 572 | widely named: client/gui/screens/advancements | carry-over | **pass** | Reviewed (multi-file): advancements/ directory byte-identical between the trees. |
| 573 | widely named: world/entity/projectile/windcharge | carry-over | **pass** | Reviewed (multi-file): wind-charge classes live in `projectile/windcharge/` in the 1.21.10 jars (package move confirmed); projectiles.rs byte-identical. |
| 574 | crates/horizon-client-render/src/gfx/core_lines.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 575 | crates/horizon-client-render/src/inventory_effects.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 576 | crates/horizon-client/src/dialog.rs | carry-over | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module. The 1.21.11 behaviour is already..."). |
| 577 | crates/horizon-server/src/management_schema.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 578 | crates/horizon-world/src/worldgen/structure/ocean_ruin.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 579 | crates/horizon-client/src/advancements/state_ranges.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 580 | crates/horizon-client/src/game/lan_screen.rs | carry-over | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Diff vs the 1.21.11 tree is comment-only (version labels); no vanilla method is named in the module."). |
| 581 | crates/horizon-protocol/src/packets/configuration.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 582 | crates/horizon-shaders/src/custom.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 583 | crates/horizon-world/src/worldgen/structure/jigsaw.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 584 | widely named: client/resources/metadata/texture | carry-over | **pass** | Reviewed (multi-file): TextureMetadataSection is blur/clamp only in the 1.21.10 jar (javap-verified). |
| 585 | crates/horizon-client-core/src/bossbar.rs | carry-over | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Diff vs the 1.21.11 tree is comment-only (version labels); no vanilla method is named in the module."). |
| 586 | crates/horizon-client-render/src/options_file.rs | carry-over | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module. The 1.21.11 behaviour is already..."). |
| 587 | crates/horizon-client/src/mobs/armadillo.rs | carry-over | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Diff vs the 1.21.11 tree is comment-only (version labels); no vanilla method is named in the module. The ZombieNautilus match arm is dead on 1.21.10..."). |
| 588 | crates/horizon-client/src/mobs/nautilus.rs | carry-over | **pass** | Reviewed: 10-line diff is the feed_heal + test removal, as the evidence states; verified: 0 Nautilus classes in the 1.21.10 jars. |
| 589 | crates/horizon-command/src/packrat.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 590 | crates/horizon-protocol/src/hash_ops.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 591 | crates/horizon-world/src/worldgen/structure/mansion.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 592 | widely named: server/commands | ported | **gap** | GAP (behaviour, minor — B11-G19): stopwatch removal verified (0 stopwatch classes in either jar; stopwatches.rs deleted; 0 hits in commands/tree files) and worldborder integer time args verified (tree_world.rs:333-337 `integer(0)`; javap IntegerArgumentType.integer(0)) — BUT the setSize feedback sub-port is WRONG/inverted. Both 1.21.10 and 1.21.11 format `%.1f` (1.21.10 in lambdas lambda$setSize$16/17/18 via String.format(Locale.ROOT, "%.1f"); 1.21.11 in setSize itself, offsets 65-79, passing a pre-formatted String). Rust terrain_cmds.rs:441-445 uses `double_to_string` with the wrong comment "1.21.10 passes the raw double". Possible Batch 9 overlap — flag, do not fold. |
| 593 | crates/horizon-assets/src/item_bake.rs | carry-over | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Diff vs the 1.21.11 tree is comment-only (version labels); no vanilla method is named in the module."). |
| 594 | crates/horizon-client-render/src/gfx/submit.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 595 | crates/horizon-client/src/game/cauldrons.rs | carry-over | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module. The 1.21.11 behaviour is already..."). |
| 596 | crates/horizon-command/src/codec.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 597 | crates/horizon-world/src/level/timeline.rs | carry-over | **pass** | Reviewed: 3-line diff is the turtle_hatch_chance 0.002 fallback, as the evidence states. |
| 598 | crates/horizon-world/src/worldgen/structure/end_city.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 599 | crates/horizon-client-render/src/command_block.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 600 | crates/horizon-client-render/src/jigsaw_block.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 601 | crates/horizon-prepare/src/artifacts.rs | no-change | **pass** | Spot-checked: byte-identical to 1.21.11. |
| 602 | crates/horizon-client-core/src/text.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 603 | crates/horizon-client-input/src/gamepad.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 604 | crates/horizon-client/src/menu/equippable.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 605 | crates/horizon-command/src/tree_extra.rs | ported | **pass** | Javap: FetchProfileCommand literals only fetchprofile/name/id (no entity branch). Rust tree_extra.rs:57-64. |
| 606 | widely named: commands/arguments | carry-over | **pass** | Reviewed (multi-file): `PermissionSource.allowsSelectors()` = `hasPermission(2)` (javap-verified, iconst_2); ResourceLocationArgument/Exception live in the 1.21.10 jars (Identifier renames confirmed). |
| 607 | crates/horizon-bots/src/main.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 608 | crates/horizon-client-render/src/vault.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 609 | crates/horizon-client/src/game/block_items.rs | carry-over | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module. The 1.21.11 behaviour is already..."). |
| 610 | crates/horizon-client/src/game/cat_spawner.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 611 | crates/horizon-client/src/item_render.rs | carry-over | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Diff vs the 1.21.11 tree is comment-only (version labels); no vanilla method is named in the module."). |
| 612 | crates/horizon-client/src/scoreboard_cmd.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 613 | crates/horizon-world/src/worldgen/structure/fortress.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 614 | crates/horizon-world/src/worldgen/structure/piece_limited.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 615 | crates/horizon-world/src/worldgen/structure/saved_templates.rs | carry-over | **pass** | Reviewed: 3-line diff is the shipwreck `state: None` (no settled-height state), as the evidence states; verified against 1.21.10's ShipwreckPiece (addAdditionalSaveData only calls super). |
| 616 | launcher/src/screenshots.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 617 | widely named: client/gui/screens/social | carry-over | **pass** | Reviewed (multi-file): social.rs + social_screen.rs byte-identical between the trees. |
| 618 | crates/horizon-client-core/src/gamerules.rs | ported | **pass** | `minecartMaxSpeed` in both gamerule tables (gamerules.rs:180/216) + test:579-583. Javap: IntegerValue.set just sets (putfield + onChanged, no range check); Rules::set:354 no range enforcement. |
| 619 | crates/horizon-client-render/src/crash.rs | carry-over | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Diff vs the 1.21.11 tree is comment-only (version labels); no vanilla method is named in the module."). |
| 620 | crates/horizon-client-render/src/social.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 621 | crates/horizon-client/src/flatgen.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 622 | crates/horizon-client/src/game/allays.rs | carry-over | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Diff vs the 1.21.11 tree is comment-only (version labels); no vanilla method is named in the module."). |
| 623 | crates/horizon-client/src/game/commands/effects.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 624 | crates/horizon-client/src/game/friends.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 625 | crates/horizon-client/src/game/mob_looting.rs | carry-over | **pass** | Reviewed: 12-line diff is the `approximate_with` unfiltered fold, as the evidence states. |
| 626 | crates/horizon-client/src/game/names.rs | carry-over | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Diff vs the 1.21.11 tree is comment-only (version labels); no vanilla method is named in the module."). |
| 627 | crates/horizon-client/src/game/native_criteria.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 628 | crates/horizon-client/src/game/saving.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 629 | crates/horizon-client/src/game/social.rs | carry-over | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module. The 1.21.11 behaviour is already..."). |
| 630 | crates/horizon-client/src/game/spectator.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 631 | crates/horizon-client/src/lan/waypoints.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 632 | crates/horizon-client/src/legacy_combat.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module. The undead/arthropod name lists..."). |
| 633 | crates/horizon-shaders/src/biomes.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 634 | crates/horizon-world/src/level/feature_world.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 635 | crates/horizon-world/src/level/redstone/experimental.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 636 | crates/horizon-world/src/level/sculk.rs | carry-over | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Diff vs the 1.21.11 tree is comment-only (version labels); no vanilla method is named in the module."). |
| 637 | crates/horizon-world/src/worldgen/feature_catalog.rs | ported | **pass** | Javap: RandomPatchConfiguration HAS a `feature` field. Rust feature_catalog.rs:245-247 restores flower/no_bonemeal_flower/random_patch to the nested-`feature` expansion arm. |
| 638 | crates/horizon-world/src/worldgen/structure/processor.rs | carry-over | **pass** | Spot-checked: byte-identical to 1.21.11. |
| 639 | widely named: client/gui/layouts | carry-over | **pass** | Reviewed (multi-file): screen_layout.rs, hud_layout.rs, split_layout.rs byte-identical between the trees. |
| 640 | widely named: client/gui/screens/dialog | carry-over | **pass** | Reviewed (multi-file): dialog.rs + dialog_screen.rs byte-identical between the trees. |
| 641 | crates/horizon-client-core/src/registries.rs | carry-over | **pass** | Reviewed: 1-line diff is the `stopwatches` registry field removal (item 592 spillover); the carry-over claim (timeline registry stays empty on 1.21.10 data) holds. The evidence's "comment-only" phrase is stale by that one line. |
| 642 | crates/horizon-client-input/src/clipboard.rs | no-change | **pass** | Spot-checked: byte-identical to 1.21.11. |
| 643 | crates/horizon-client-render/src/auth.rs | no-change | **pass** | Spot-checked: byte-identical to 1.21.11. |
| 644 | crates/horizon-client-render/src/core_shader.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 645 | crates/horizon-client-render/src/gfx/end_sky/tests.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 646 | crates/horizon-client-render/src/gfx/frame_gen.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 647 | crates/horizon-client-render/src/pack_palette.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 648 | crates/horizon-client-render/src/streaming.rs | carry-over | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module. The 1.21.11 behaviour is already..."). |
| 649 | crates/horizon-client/src/game/code_of_conduct.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 650 | crates/horizon-client/src/game/commands/items/contents.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 651 | crates/horizon-client/src/game/commands/raid.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 652 | crates/horizon-client/src/game/item_particles.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 653 | crates/horizon-client/src/game/lan_crafting.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 654 | crates/horizon-client/src/game/launch_options.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 655 | crates/horizon-client/src/game/native_client_entities.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 656 | crates/horizon-client/src/game/pickups.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 657 | crates/horizon-client/src/game/quick_actions.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 658 | crates/horizon-client/src/game/resource_load.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 659 | crates/horizon-client/src/game/sculk.rs | carry-over | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module. The 1.21.11 behaviour is already..."). |
| 660 | crates/horizon-client/src/game/window_handler.rs | carry-over | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module. The 1.21.11 behaviour is already..."). |
| 661 | crates/horizon-client/src/mobs/cube.rs | no-change | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module. Stale spear comment fixed..."). |
| 662 | crates/horizon-client/src/stopwatches.rs | no-change | **gap** | GAP (ledger evidence, not behaviour): evidence claims "Module identical (0 diff lines)" but stopwatches.rs is DELETED in the 1.21.10 tree (exists in 1.21.11, 3638 bytes) — removed by item 592's port (verified correct: 0 stopwatch classes in the 1.21.10 jars). The no-change verdict does not hold against the current tree; the ledger entry is stale. |
| 663 | crates/horizon-client/src/title/create_world.rs | carry-over | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module. The 1.21.11 behaviour is already..."). |
| 664 | crates/horizon-port-pack/src/carry.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 665 | crates/horizon-port-pack/src/lithostitched.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 666 | crates/horizon-server-files/src/profiles.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 667 | crates/horizon-world/src/datafix/dynamic.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 668 | crates/horizon-world/src/level/shapes.rs | carry-over | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Diff vs the 1.21.11 tree is comment-only (version labels); no vanilla method is named in the module."). |
| 669 | crates/horizon-world/src/level/update.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 670 | crates/horizon-world/src/worldgen/dripstone.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 671 | crates/horizon-world/src/worldgen/end_spike.rs | carry-over | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Diff vs the 1.21.11 tree is comment-only (version labels); no vanilla method is named in the module."). |
| 672 | crates/horizon-world/src/worldgen/generator_decoration.rs | carry-over | **pass** | Diff vs the 1.21.11 tree is comment-only (version labels); ledger evidence holds ("Diff vs the 1.21.11 tree is comment-only (version labels); no vanilla method is named in the module."). |
| 673 | crates/horizon-world/src/worldgen/structure/monument.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 674 | crates/horizon-world/src/worldgen/structure/start.rs | no-change | **pass** | Module byte-identical to the 1.21.11 tree; ledger evidence holds ("Module identical between the 1.21.11 and 1.21.10 trees (0 diff lines); no vanilla method is named in the module."). |
| 675 | widely named: server/commands/data | carry-over | **pass** | Reviewed (multi-file): DataCommands + StorageDataAccessor live in the 1.21.10 server jar; data_cmd.rs byte-identical; game/commands/data.rs differs only by the item 11/20 anger work (item 512). |

Tally: 211 = 208 pass + 3 gap + 0 needs-harness + 0 needs-probe + 0 accepted-gap (ledger verdicts).
