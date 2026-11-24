# Batch 11 — Rank-3 triage (6 items), 26.1.1 parity sweep

- **Batch:** 11 (rank-3 triage) — checks + triage only, no fixes.
- **Date:** 2026-11-24
- **Scope:** 26.1.1's OWN 6 port-ledger items
  (`versions/26.1.1/work-items.json`, all closed; ranks 3/1/1/0/1 —
  item 5 rank 3, items 1–3 rank 1, item 4 rank 2, item 6 rank 5),
  triaged against the named 26.1.1 jars and the 26.1.1 port in
  `versions/26.1.1/crates/` (branch `parity-sweep-26.1.1`). 26.1.1 is
  a patch delta on 26.1, so there is NO `rank3-progress.json` and no
  port triage workdir for this version. Vanilla 26.1.1's own jar/data
  is the spec throughout; the 26.3 base's form was never trusted.
  **Scope difference from the 1.21.7 Batch 11:** that batch triaged
  the 1.21.11 TEMPLATE's 247 rank-3 items FOR 1.21.7; like the 26.2
  run, this batch verifies 26.1.1's own port ledger — i.e. each
  "closed" verdict is checked to actually hold in the current tree.
- **Ledger slice:** `slice.json` (same shape as the 26.1 batch-11 slice).

## Tally (6 items)

| Verdict | Count |
|---|---|
| pass | 5 |
| gap (defect attached) | 0 |
| needs-new-harness | 1 |
| needs-in-game-probe | 0 |
| accepted gap (ledger verdict) | 0 |

As-tallied 6 = 1 behaviour-change item (item 5) + 5 plumbing items
(items 1–4, 6); 0 oracle/harness/driver-status checks tallied. The
class-table sweep (§6.1) is reported with its own counts (live/total,
jar + chain), NOT folded into the tally.

Defects found: **9** (`B11-G01`–`B11-G09`), of which 3 major, 6 minor.
Eight reproduce the 26.1 run's defects (G01–G08); one is NEW vs 26.1
(G09, F3 spawn-count abbreviations — the 26.1 tree was fixed, the
26.1.1 tree regressed). All nine are cross-cutting (class-table sweep
+ 1.21.7-defect cross-checks + the 26.1-tree diff) with no single
owning ledger item — they are recorded as defects, not item verdicts,
so the defect count (9) exceeds the gap-item count (0). Oracle
defects: **2** (`B11-O01`–`B11-O02`, both minor, both cauldron
oracles). Needs-harness: **1** (`B11-H01`, item 5's regression test,
which cannot run until the pre-existing horizon-client lib-test
breakage is fixed — per the work-items note; recorded as
needs-harness, not a pass, per the brief). Accepted-gap notes carried
from the 1.21.7 sweep: **3** (`B11-A01`–`B11-A03`, §5 — not ledger
verdicts, not counted in the 6-item tally). No probe items: the
1.21.7 P01 probe is settled by the 26.1.1 jar (§6.6), and the 26.2
run's B11-H01 (chain-liveness half) stays CLOSED — the chain half ran
again here (§6.1).

**Overlap flags for the coordinator (do not fold — merge notes only):**
six of the nine defects are also minted by the batch that owns the
system, with the same root cause and evidence — B11-G02 ↔ B2-G01
(wire), B11-G05 ↔ B1-G03 (splash water), B11-G03 ↔ B5-G03 (paths),
B11-G04 ↔ B5-G13 (dripstone), B11-G01 ↔ B5-G27 (partial: B5 owns the
bed arm's internal pair-check logic; B11 owns the dead dispatch),
B11-G07 ↔ B4-G01 (partial: B4 owns the empty-fuel-list/furnace
consequence; B11 owns the recipe-book order). Both oracle defects are
also minted elsewhere: B11-O01 ↔ B1-O02 + B5-O01, B11-O02 ↔ B1-O07
(the coordinator dedups oracles at merge, as in 26.1).

Headline: the 26.1.1 port's 6-item ledger holds up — the five
plumbing verdicts re-verify against the 26.1.1 jar and the current
tree, and item 5's one-instruction fix is present in the tree (though
its regression test cannot run). But the class table is in exactly
the shape 26.1's was in — 263/271 live, the same 8 dead mappings, no
dual names — and the 26.1.1 tree additionally FAILED to inherit
several of the 26.1 tree's deliberate behaviour fixes: the bed arm
carries the 26.3 pair-check form (26.1's item-584 redirect is gone),
the fuel system reads the 26.3 `cooking_fuel` component 26.1.1
vanilla never defines (26.1's `burn_time` table is gone), and the F3
spawn-counts line uses the 26.2+ abbreviations (26.1's single letters
are gone). Of the 23 defects the 1.21.7 Batch 11 found, **seven**
carry over to 26.1.1 (G01, G02, G03-paths-half, G06, G12, G13, G15)
plus one cosmetic detail from P01 — the same seven as 26.1 — plus one
NEW F3 defect (G09) that 26.1 did not have. The pattern is
consistent: the 26.1.1 tree was cut from a pre-completion 26.1 base
and the completion pass was not fully carried forward (see §6.3).

## §1 Defects

### B11-G01 — Beds place foot-only; sleep is unreachable (major; = 1.21.7 B11-G01, = 26.2 B11-G01, = 26.1 B11-G01; REPRODUCES)

`Class::AbstractBed = "AbstractBedBlock"` (classes.rs:59) never matches
any 26.1.1 chain: 26.1.1 has no `AbstractBedBlock` — beds are `BedBlock`
directly (jar-verified: `BedBlock extends HorizontalDirectionalBlock`,
implements `EntityBlock`; no `AbstractBedBlock` in either 26.1.1 jar;
`BlockEntityType.BED` exists and covers all 16 bed blocks, whose
prepared chains are `[BedBlock, HorizontalDirectionalBlock, Block]`).
`Class::Bed = "BedBlock"` (classes.rs:60) exists but NOTHING dispatches
on it (grep-confirmed across all crates). Every bed arm is dead:

- `place.rs:356` (placement check), `place.rs:1481` `set_placed_by` — the
  ONLY place a `"head"` part is ever set (place.rs:1483, grep-confirmed)
  — so a player-placed bed is a foot-only single block.
- `interact.rs:365` — the ONLY emitter of `PlayerEffect::Sleep`
  (interact.rs:377/380) — so clicking ANY bed (including village beds)
  does nothing; `try_sleep` never fires.
- `hooks.rs:459` (creative break), `shapes.rs:124` (neighbour update).

`BlockData::is` checks the bitset built from `Class::from_java` over the
chain (data.rs:215, :922), so no fallback exists. Severity major:
sleep/spawn is core gameplay. **26.3-leak:** the 26.3 client jar
CONTAINS `AbstractBedBlock` (26.3's `BedBlock extends
AbstractBedBlock`), so the mapping is 26.3-correct and 26.1.1-wrong.
**NEW vs 26.1:** the 26.1 tree's item-584 port of the arm's LOGIC is
absent here — the 26.1.1 tree's interact.rs:365-384 arm is the 26.3
pair-check form (checks the other half is the same bed with the
opposite part, `facing.opposite()` for a head click), not 26.1's
foot→head redirect (26.1 tree: no pair check, Sleep emitted at `pos`).
So even a dispatch fix alone would leave the arm implementing the
wrong form; the fix must restore the 26.1 redirect too. The arm-logic
layer is also recorded as B5-G27 (Batch 5); this item owns the dead
dispatch. The fix is a 26.1-direct one (dispatch on `Class::Bed` +
26.1 redirect), not a re-base propagation.

### B11-G02 — Redstone wire never updates power (major; = 1.21.7 B11-G02; = 26.1 B11-G02; REPRODUCES; overlaps B2-G01)

26.1.1 names the class `RedStoneWireBlock` (capital S, jar-verified in
both jars — the class file is byte-identical to 26.1's; the prepared
chain for `minecraft:redstone_wire` is `[RedStoneWireBlock, Block]`).
The table maps `RedstoneWireBlock` only (classes.rs:224) — the 26.3
name, absent from both 26.1.1 jars — and the 26.1.1 tree carries NO
dual name (26.2's tree maps `"RedstoneWireBlock" |
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
"RedstoneWire")`, which misses 26.1.1's `RedStoneWireBlock` chain name
(same for `PathBlock` :10 and `SpeleothemBlock` :21 in that table).
Severity major: redstone is core gameplay. **26.3-leak:**
`RedstoneWireBlock` exists in the 26.3 jar. **Regression note:** the
26.1 tree's survive.rs:282 comment cited the correct 26.1 name
(`RedStoneWireBlock.canSurvive`); the 26.1.1 tree's comment at the
same line cites the 26.3 name (`RedstoneWireBlock.canSurvive`) — and
interact.rs:492's doc likewise regressed to the 26.3 name. Comment-only,
but it marks the tree's provenance. Batch 2 mints the same defect as
B2-G01 from the redstone side.

### B11-G03 — Dirt paths never revert (minor; = the paths half of 1.21.7 B11-G03; = 26.1 B11-G03; REPRODUCES; overlaps B5-G03)

26.1.1 has `DirtPathBlock` (jar-verified; chain `[DirtPathBlock,
Block]`); the table maps `PathBlock` only (classes.rs:208) — the 26.3
name, absent from both 26.1.1 jars, no dual name. All four Path arms
are dead: place.rs:929 (placement state), shapes.rs:465 (updateShape —
shared with the live Farmland arm), survive.rs:273 (`canSurvive`: solid
above kills the path), tick.rs:202 (random tick →
`turn_to_base_block`). `turn_to_base_block` (tick.rs:501) has only
three callers — tick.rs:199 (Farmland, live), :202 (Path, dead), :1046
(farmland moisture, live) — so no name-based path revert exists: a
covered or trampled dirt path never turns back to dirt. The farmland
half of 1.21.7's B11-G03 does NOT reproduce: `FarmlandBlock` is live
in 26.1.1 (chain `[FarmlandBlock, Block]`, classes.rs:151) and its arms
work. Batch 5 mints the same defect as B5-G03.

### B11-G04 — Dripstone placement/survival/growth dead (minor; = 1.21.7 B11-G06; = 26.1 B11-G04; REPRODUCES; overlaps B5-G13)

26.1.1 has no `SpeleothemBlock` at all (absent from both jars;
speleothems are a 26.2 addition). Pointed dripstone is
`PointedDripstoneBlock` directly (chain `[PointedDripstoneBlock,
Block]`). The table maps `Speleothem = "SpeleothemBlock"`
(classes.rs:215) and all five Speleothem arms are dead: place.rs:1156
(placement state), shapes.rs:728 (updateShape), survive.rs:307
(`canSurvive`), tick.rs:248 (scheduled tick — destroy/fall),
tick.rs:434 (random tick → `speleothem_random_tick`, growth +
cauldron-fill). There is NO `Class::PointedDripstone` arm at any of
those five dispatch sites — `PointedDripstone` (classes.rs:216, live)
is used only inside speleothem.rs helpers (:86, :103, :206), which are
themselves reached only via the dead Speleothem arms. Second
independent cause, as in 1.21.7: the `minecraft:speleothems` TAG is
absent from 26.1.1 data (26.2 has it), so even the tag-gated helpers
(`speleothem_tag`, `speleothem_with`, `speleothem_is_tip` in
speleothem.rs:16-31) return false for every block. In 26.2 the arms
were live because `SpeleothemBlock` is `PointedDripstoneBlock`'s
abstract parent there; in 26.1.1 neither the class nor the tag exists.
Batch 5 mints the same defect as B5-G13 (major there).

### B11-G05 — Splash water never douses fire, hurts, extinguishes, or rehydrates (major; = 1.21.7 B11-G13; = 26.1 B11-G05; REPRODUCES; overlaps B1-G03)

Vanilla 26.1.1 `AbstractThrownPotion.onHitBlock` gates on
`potionContents.is(Potions.WATER)` — direct holder equality, NO tag
(bytecode 55-63, javap-verified on the 26.1.1 server jar), and
`onHitEntity` routes water through `onHitAsWater` the same way. The
Rust gates all four behaviours on potion TAGS (game/splash_water.rs:
`potion_in_tag`, :42-51; douse :60, hurt/extinguish :134-135,
rehydrate :197), but 26.1.1 has NO potion tags beyond `tradeable`:
`PotionTags` has ONLY `TRADEABLE` (javap-verified), the client-jar
datapack holds only `tags/potion/tradeable.json`, and the prepared
data's `tags/potion/` likewise — so the `potion_tags` map
(merchant.rs:489, :574) never contains `douses_fire`,
`hurts_water_sensitive_entities`, `extinguishes_entities`, or
`rehydrates_axolotls`, and every gate is always false. The douse area
matches vanilla exactly; only the gate diverges. **26.3-leak:** 26.3's
datapack HAS all four tag files, so the tag gate is 26.3-correct and
26.1.1-wrong. (26.2's tree was fixed to `Potions.WATER` equality; the
26.1.1 tree still carries the tag form — splash_water.rs is 0-diff vs
the 26.1 tree.) Severity major: splash water is the fire-dousing
mechanic. Batch 1 mints the same defect as B1-G03.

### B11-G06 — Three number providers return 0 (minor, datapack-only; = 1.21.7 B11-G12, = 26.2 B11-G02, = 26.1 B11-G06; REPRODUCES)

Vanilla 26.1.1 has 6 number providers (`EnchantmentLevelProvider`,
`ScoreboardValue`, `StorageValue` all present in the server jar). Both
Rust implementations handle only 3 (`minecraft:constant`,
`minecraft:uniform`, `minecraft:binomial`) — `minecraft:enchantment_level`,
`minecraft:score`, `minecraft:storage` fall to `_ => 0` (horizon-world
loot.rs `int`/`float` 1329-1346+; horizon-items items.rs `number`
1903-1936 — the `number` fn body is byte-identical to the 26.1 tree's,
only the line numbers moved). Data check: 0/1,326 prepared vanilla
26.1.1 loot tables use the 3 missing providers (0/1,331 client-jar
loot-table files likewise) — vanilla loot is unaffected. The only
`enchantment_level` uses in 26.1.1's datapack are the `chance`
providers in `enchantment/soul_speed.json` and
`enchantment/thorns.json`, and those go through the SEPARATE enchant.rs
`random_chance` path, which handles `enchantment_level` explicitly
(enchant.rs:874-884: `level_value(amount, level)`) — verified against
vanilla 26.1.1's `EnchantmentLevelProvider.getFloat`, which is
`amount.calculate(level)` with NO multiplication by level, exactly
what the Rust computes. So the gap is datapack-only: datapacks using
the three providers in LOOT get 0.

### B11-G07 — Recipe-book fuel slot shows no fuel; the order it would cycle is wrong (minor; = 1.21.7 B11-G15, = 26.2 B11-G03, = 26.1 B11-G07; REPRODUCES in a worse form; overlaps B4-G01)

Vanilla 26.1.1 `SlotDisplay$AnyFuel.resolve` returns
`FuelValues.fuelItems()` — the keySet of an insertion-ordered map
(javap-verified on the 26.1.1 client jar: `resolve` reads the
`FUEL_VALUES` context key and streams `fuelItems()`; `fuelItems()`
returns the keySet of the `Object2IntSortedMap` populated by the
HARDCODED `FuelValues.vanillaBurnTimes` sequence, LAVA_BUCKET first at
20000 ticks). The 26.1.1 Rust groups fuels by COOKING-FUEL COMPONENT
value in registry order (recipe_book.rs:566-585,
native_panel_recipes.rs:173-190) — but 26.1.1 vanilla has NO
`minecraft:cooking_fuel` component at all (`DataComponents` in the
26.1.1 jar has no `COOKING_FUEL` field; 26.3's has it), and the
prepared data carries it for 0 of 1,506 items, so the fuel list is
EMPTY and the ghost fuel slot is dropped (`GhostSlots.setSlot` keeps
only slots with something to show). Two layers, both wrong: the
mechanism reads a component that does not exist in this version, and
the order it would produce (component groups in registry order — oak
planks protocol id 36 first, lava bucket 1015) is not vanilla's
insertion-ordered FuelValues sequence (lava bucket first). **Worse
than 26.1:** the 26.1 tree grouped by burn-time VALUE via the
hardcoded `burn_time()` table (items.rs:743) — wrong order, but a
non-empty list; the 26.1.1 tree dropped `burn_time()` entirely and
reads the absent component. The empty-list/furnace consequence is
Batch 4's B4-G01 (major — furnaces never light, since
`providers.burn_time` also reads the absent component and returns 0);
this item owns the recipe-book surface and the order. Fixing B4-G01
by restoring the FuelValues table fixes this item's order too.

### B11-G08 — F3 chart footer uses lowercase fps (minor, cosmetic; part of 1.21.7 B11-P01, settled by the jar; = 26.1 B11-G08; REPRODUCES)

Vanilla 26.1.1's `DebugScreenOverlay` builds the chart footer as three
list entries (javap-verified): `Debug charts: {Profiler}; {FPS + TPS |
FPS};`, `{Bandwidth + Ping | Ping}; {Lightmap};`, `To edit: press
{key}` — with the chart names `FPS + TPS` / `FPS` in UPPERCASE (string
constants in the class; the footer template is `Debug charts: \x01;
\x01;`). The Rust's `debug_chart_lines` (game/debug_keys.rs:253-295)
matches the three-line layout and the single-player variants exactly,
but names the chart `fps` / `fps + tps` in lowercase (:283). Only the
fps chart name differs in case (Profiler, Bandwidth + Ping, Ping,
Lightmap all match). Cosmetic. The layout question the 1.21.7 probe
item left open is settled: no in-game probe is needed — the jar gives
the footer outright.

### B11-G09 — F3 spawn-counts line uses 26.2+ category abbreviations (minor, cosmetic; NEW vs 26.1)

Vanilla 26.1.1's `DebugEntrySpawnCounts` renders each mob category as
the FIRST LETTER of its name, uppercased
(`Character.toUpperCase(MobCategory.getName().charAt(0))`,
javap-verified on the 26.1.1 client jar) — so monster and misc both
read M, ambient and axolotls both A, water creature and water ambient
both W: `SC: 289, M: 3, C: 10, A: 1, A: 0, U: 0, W: 2, W: 4, M: 0`.
26.1.1 has no `MobCategory.getDebugAbbreviation` (a 26.2 addition —
absent from the 26.1.1 server jar). The Rust uses the 26.2+ two/three-
letter abbreviations (debug_overlay.rs:391-392:
`["MO", "C", "AM", "AX", "UWC", "WC", "WA", "MI"]`, rendered at :397),
and its own test pins the wrong form (:640:
`"SC: 289, MO: 3, C: 10, AM: 1, AX: 0, UWC: 0, WC: 2, WA: 4, MI: 0"`).
**NEW vs 26.1:** the 26.1 tree was FIXED here (single-letter
abbreviations with a comment documenting the absence of
`getDebugAbbreviation` in 26.1); the 26.1.1 tree regressed to the
26.2+ form. Cosmetic — the counts themselves are right, only the
labels differ. A re-base must not propagate the 26.2+ abbreviations
down (§6.4).

## §2 Oracle defects

No oracle repairs were made (Batch 0's scope decision: measurement +
triage only; the oracle `.java` files live under `versions/26.1.1/`
and this sweep forbids touching them). Two cauldron oracles in this
batch's scope fail against the 26.1.1 jars, both triaged from the
Batch 0 measured table (`docs/sweep/26.1.1/batch-0-oracles.json`,
189/391 compile) with the missing symbol javap-verified against the
26.1.1 jar — not from an import-level check. Both are the same two
oracles the 1.21.7 sweep met (B11-O01 there was repaired; here they
are recorded only). Both are also minted by Batch 1 (B1-O02, B1-O07)
and B11-O01 additionally by Batch 5 (B5-O01); the coordinator dedups
at merge.

### B11-O01 — VanillaCauldronOracle fails: `net.minecraft.util.Prediction` absent in 26.1.1 (minor, oracle-defect; REPRODUCES 26.1 B11-O01)

Batch 0: fails on all three classpaths, class `missing-symbol`, first
error `118: cannot find symbol`, 1 error. Line 118 is the
`FakePlayer.drop(ItemStack, boolean, net.minecraft.util.Prediction)`
override — the 26.3 3-arg drop form. `Prediction` is absent from both
26.1.1 jars (jar-index verified) and present in the 26.3 client jar
(`net.minecraft.util.Prediction`). 26.1.1's drop is the 2-arg form, as
in 1.21.7 (whose B11-O01 repaired exactly this). Repair (follow-up,
not done here): drop the `Prediction` parameter, as 1.21.7's B11-O01
did (commit c754e3a2 there). This oracle is the driver behind the
cauldron table comparison (1.21.7's B11-H02); with G23 withdrawn for
26.1.1 the comparison is no longer confirmatory for a live defect,
but the oracle remains broken for any future cauldron driver work.

### B11-O02 — VanillaLavaCauldronItemOracle fails: `getNextEntityId` absent in 26.1.1 (minor, oracle-defect; REPRODUCES 26.1 B11-O02)

Batch 0: fails on all three classpaths, class `no-override`, first
error `28: method does not override or implement a method from a
supertype`, 1 error. Line 28 is `@Override public int
getNextEntityId()` on the `Lvl extends ServerLevel` stand-in.
`getNextEntityId` is declared on NEITHER `Level` nor `ServerLevel` in
the 26.1.1 server jar (javap `-p` grep on both classes; `registryAccess`
IS present on `ServerLevel`, so the neighbouring override at line 29 is
fine). The method is a 26.2/26.3 addition the oracle assumes. Repair
(follow-up): remove the override (the stand-in's `next` counter is only
used by the oracle itself) or move the id allocation out of the
`ServerLevel` subclass.

In-scope oracles assessed PASSING (from the Batch 0 table, for
auditability): no other oracle in this batch's scope was assessed —
Batch 11 runs no drivers and its only in-scope oracles are the two
cauldron oracles above, both failing. The full failing-oracle list
(202 files) belongs to the batches whose drivers use them; the
coordinator dedups at merge.

## §3 Needs-new-harness

### B11-H01 — Item 5's regression test cannot run (minor, needs-harness)

The 26.1.1 port's one behaviour change (item 5, the PlayerEntry
`chatDisabledOrBlocked` flip) is implemented in the tree
(social_screen.rs:130-132, `!can_receive_player_messages || blocked`,
with a four-case unit test at :454-462) and verified statically
against the jar (§8, item 5) — but the work-items note records that
the regression test "cannot run until the pre-existing horizon-client
lib-test breakage is fixed". Per the brief this is recorded as a
needs-harness note, NOT a pass: item 5's tally verdict is
needs-harness. What would settle it: the horizon-client lib-test
breakage fixed (a test-health job, not this sweep), then the
`chat_disabled_or_blocked_follows_26_1_1` test run green. This B11-H01
is NOT the 26.2 run's B11-H01 (class-table chain-liveness half) —
that item stays CLOSED (see below); the id is reused for this run's
first harness item.

Two notes on inherited harness items:

- **26.2's B11-H01 (class-table chain-liveness half) stays CLOSED.**
  Batch 0's prepare landed for 26.1.1, so `block-behaviours.json`
  exists (1,168 blocks) and the chain half ran again here (§6.1):
  263/271 mappings chain-live, 0 jar-live-but-chain-dead.
- **1.21.7's B11-H02 (exhaustive cauldron-table driver comparison)
  does NOT carry.** Its premise was confirming B11-G23 (the wash gap);
  G23 is withdrawn for 26.1.1 (§6.6 — the tag is present and the gate
  is live). The 26.1 prepare does not unblock it in any case: the
  driver is blocked on the two oracle defects above (B11-O01/O02),
  and the comparison itself needs a probe build this batch does not
  run. No new harness item is minted for it: nothing in this batch's
  scope needs a harness to settle beyond B11-H01.

## §4 Needs-in-game-probe

None. The 1.21.7 batch's probe item (F3 chart footer, P01) does not
carry: 26.1.1 HAS the `DebugScreenOverlay` chart footer in the jar, and
the Rust's three-line form matches it — the only delta is the lowercase
`fps` name, recorded as B11-G08. No ledger item needed a probe to
settle.

## §5 Accepted gaps / notes (not defects)

The 26.1.1 port ledger carries NO accepted-gap verdicts (unlike 26.2's
items 427/436), so none of these is a ledger item and none is counted
in the 6-item tally. All three are notes carried from the 1.21.7
sweep, re-verified against 26.1.1:

- **B11-A01 — ProvidesTrimMaterial inline/direct form [= 1.21.7
  B11-A01, = 26.1 B11-A01].** Vanilla 26.1.1's
  `ProvidesTrimMaterial.material` is `Holder<TrimMaterial>`
  (javap-verified) — a Holder direct OR a ResourceKey string. The Rust
  reads the string form only (enchant.rs:333-335, `.as_str()`). All 11
  vanilla 26.1.1 items with the component use string keys (prepared
  items.json: amethyst, copper, diamond, emerald, gold, iron, lapis,
  netherite, quartz, redstone, resin) — the direct/inline form is
  datapack-only.
- **B11-A02 — Special-foil decal not modelled [= 1.21.7 B11-A03, =
  26.1 B11-A02].** The Rust HAS a `Foil::Special` variant ("compasses
  and clocks, a sheeted decal", item_render.rs:140-146) carried
  through `submit_quads` (:437), but both sinks treat every non-None
  foil identically: the GUI sink draws the glint over the full quad
  (item_render/gui.rs:421-423) and the world sink likewise
  (item_render.rs:1155-1173) — no 0.5x GUI / 0.75x first-person decal
  scaling. Only visible on a command-enchanted compass/clock.
  Pre-existing in both versions, not a version delta.
- **B11-A03 — NBT separator ignored + per-player resolution [= 1.21.7
  B11-A04, = 26.1 B11-A03].** The Rust's `component_arg`
  (game/commands/text_commands.rs:60-135) resolves selectors ONCE for
  the command source and always joins names with `", "` (:120, :231)
  — the separator field of nbt/selector components is ignored and a
  title/tellraw component is not re-resolved per player (vanilla
  resolves per player). Gaps from BOTH versions equally, not version
  deltas.

NOT carried: 1.21.7's B11-A02 (NBT `sound_variant` written on
non-wolf mobs) NEVER EXISTED for 26.1.1 — 26.1.1 vanilla HAS sound
variants for cows, pigs, chickens and cats (`sound_variant` referenced
by `Chicken`, `Cow`, `Cat`, `Pig` and `Wolf` classes in the 26.1.1
server jar — the classes are byte-identical to 26.1's), and the Rust's
`sound_variants` (mobs/finalize.rs:2368-2377) lists exactly those five
species. Writing the tag is correct 26.1.1 behaviour.

## §6 Systemic findings

### §6.1 The class table: 8 dead mappings, 4 with live arms — and the chain half RUNS again

Method: parse every `Class::X = "Name"` in
`versions/26.1.1/crates/horizon-world/src/level/classes.rs` (271
mappings — the table has NO dual names at all, unlike 26.2's, which
carries `"PathBlock" | "DirtPathBlock"` and `"RedstoneWireBlock" |
"RedStoneWireBlock"`; classes.rs is 0-diff vs the 26.1 tree); LIVE iff
the name exists as a class (top-level or nested) in either 26.1.1 jar
(server-game 7,351 classes; client 10,682 — the same counts as 26.1).
Chain half: LIVE iff the name appears in ≥1 block chain in the prepared
`block-behaviours.json` (1,168 blocks, 291 distinct chain names).

Result: **263/271 jar-live, 8 jar-dead; 263/271 chain-live, 0
jar-live-but-chain-dead.** The jar half and the chain half agree
exactly — every jar-live mapping is chain-live, every jar-dead mapping
is chain-dead. The chain half adds no new dead mappings, which keeps
the 26.2 H01 question settled in the negative for 26.1.1 as well: no
mapping is live in the jar but dead in the chains. The sweep is
identical to 26.1's in every count (the classes.rs file is unchanged
and the 26.1.1 chains match 26.1's).

**Group A — RENAME: 26.1.1 content exists under another name; live arms
dead (4):**

| Mapping | 26.1.1 form (jar + chain) | Dead arms | Defect |
|---|---|---|---|
| AbstractBed → `AbstractBedBlock` | `BedBlock` directly | 5 (place ×2, interact, hooks, shapes) | B11-G01 (major) |
| RedstoneWire → `RedstoneWireBlock` | `RedStoneWireBlock` | 10 (hooks ×3, interact, place, redstone ×3, shapes, survive) | B11-G02 (major) |
| Path → `PathBlock` | `DirtPathBlock` | 4 (place, shapes, survive, tick) | B11-G03 (minor) |
| Speleothem → `SpeleothemBlock` | `PointedDripstoneBlock` directly (no abstract parent) | 5 (place, shapes, survive, tick ×2) | B11-G04 (minor) |

All four 26.3 names exist in the 26.3 client jar — the table is
26.3-correct and 26.1.1-wrong in all four cases. The other five 1.21.7
renames do NOT carry to 26.1.1 — the 26.x names MATCH the 26.1.1 jar
(verified each, not assumed): FarmlandBlock ✓ (chain
`[FarmlandBlock, Block]`), SnowyBlock ✓ / SpreadingSnowyBlock ✓
(grass_block chain `[GrassBlock, SpreadingSnowyBlock, SnowyBlock,
Block]`), NetherFungusBlock ✓ / NetherRootsBlock ✓ (crimson_fungus
chain `[NetherFungusBlock, VegetationBlock, Block]`). LilyPad is the
correct `LilyPadBlock` (26.1.1 has no `WaterlilyBlock`); CopperChestBlock
✓ (chain via `WeatheringCopperChestBlock`), CopperGolemStatueBlock ✓.

**Group B — NO 26.1.1 CONTENT: dead but harmless (4):** StrawBedBlock
(26.3-only; 1 arm: update.rs:430), SulfurSpikeBlock (26.2-only; 1 arm:
speleothem.rs:110, the max-height-2 branch), ShelfMushroomBlock
(26.3-only; 4 arms: bonemeal.rs:94, place.rs:493, shapes.rs:137,
survive.rs:42), PotentSulfurBlock (26.2-only; 4 arms: hooks.rs:199,
pistons.rs:154, place.rs:1193, shapes.rs:819). All four classes exist
in the 26.3 jar (SulfurSpike/PotentSulfur also in 26.2's) and none has
any block in 26.1.1 data (0 straw_bed / sulfur_spike / shelf_mushroom /
potent_sulfur entries in block-behaviours.json). Arms unreachable.
Hygiene note, not gaps.

Also verified: horizon-prepare's survival.rs CLASS_RULES carries the
same four 26.3 names (`PathBlock` :10, `RedstoneWireBlock` :43,
`SpeleothemBlock` :21, `ShelfMushroomBlock` :18) — the prepare-side
table misses 26.1.1's chain names the same way (see B11-G02).

### §6.2 The 26.x tag-name pattern INVERTS for 26.1.1 — except the two 26.3-only tags

The 1.21.7 sweep's systemic cause — the port reads 26.x tag names for
tags that don't exist in 1.21.7 data — mostly does not apply to 26.1.1,
because 26.1.1 HAS the 26.x tags. Every tag read this batch checked is
live in 26.1.1's data (client-jar datapack, 775 tag files — the same
count as 26.1; tag-file existence + contents):

- `minecraft:cauldron_can_remove_dye` (item tag: leather helmet /
  chestplate / leggings / boots, leather horse armor, wolf armor) — the
  Rust's wash gate (horizon-client/src/cauldron.rs:7-12, :214) is live;
  1.21.7's B11-G23 does not apply.
- `supports_chorus_plant` / `supports_chorus_flower` (both: end stone)
  — survive.rs:102/132/137 live; 1.21.7's B11-G21 does not apply.
- `overrides_mushroom_light_requirement` (mycelium, podzol, crimson +
  warped nylium) — survive.rs:266 live; `mushroom_grow_block` is the
  ABSENT one in 26.1.1 (the reverse of 1.21.7); 1.21.7's B11-G17b does
  not apply.
- `cannot_support_snow_layer` (ice, packed ice, barrier) /
  `support_override_snow_layer` (honey block, soul sand, mud) —
  survive.rs:295/297 live; the 1.21.11 names
  (`snow_layer_can_survive_on` / `snow_layer_cannot_survive_on`) are
  absent in 26.1.1 (the reverse of 1.21.7); 1.21.7's B11-G16b does not
  apply.
- `convertable_to_mud` (dirt, coarse dirt, rooted dirt) — live (the
  26.1.1 spelling; note B5-G19 found a DIFFERENT read site — the water
  bottle — using the wrong `convertible_to_mud` spelling; that is Batch
  5's item, not this batch's).
- `dripstone_replaceable_blocks` (`#base_stone_overworld`) — live.

Correctly-absent tags, correctly handled: `sculk_growth_inhibitors` is
absent from 26.1.1 data and the Rust checks sculk_sensor/sculk_shrieker
blocks explicitly (26.1 item 510's form, verified in the 26.1 tree;
the 26.1.1 tree's level/sculk.rs is listed by Batch 4 as regressed to
a tag-based form — Batch 4's scope, flagged here as a cross-ref, not
re-triaged). The two EXCEPTIONS — tags the port reads that 26.1.1
genuinely lacks — are both 26.3-only and both produce defects: the
potion tags `douses_fire` / `hurts_water_sensitive_entities` /
`extinguishes_entities` / `rehydrates_axolotls` (26.1.1's `PotionTags`
has ONLY `TRADEABLE`; the 26.3 datapack has all four) — B11-G05; and
`speleothems` (26.2+ only) — the second cause of B11-G04.

### §6.3 26.3-form code that 26.1.1's jar contradicts — and the 26.1 fixes the tree did NOT inherit

The port carries 26.3 forms in SEVEN places where 26.1.1's jar/data
says otherwise — all seven are 26.1.1-direct fixes, not re-base
propagations. (The 26.3 client jar, 11,383 classes, was used ONLY for
these absence checks and the §6.5 comparison — never as the spec.)

1. The four class-table mappings (§6.1 Group A): `AbstractBedBlock`,
   `RedstoneWireBlock`, `PathBlock`, `SpeleothemBlock` all exist in the
   26.3 jar and in NEITHER 26.1.1 jar.
2. The AnyFuel grouping (B11-G07): 26.3 resolves via
   `DataComponentLookup.findAll(COOKING_FUEL)` (javap-verified on the
   26.3 jar); 26.1.1 via the insertion-ordered `FuelValues` — and the
   26.1.1 tree reads the `cooking_fuel` component, which 26.1.1
   vanilla does not define at all.
3. The splash-water tag gate (B11-G05): 26.3's datapack HAS the four
   potion tag files; 26.1.1's has only `tradeable.json`, and 26.1.1
   vanilla gates on `Potions.WATER` equality.
4. The F3 spawn-counts abbreviations (B11-G09): 26.2+ uses
   `MobCategory.getDebugAbbreviation`; 26.1.1 uses the first letter
   of the category name, uppercased.
5. The bed arm's pair check (part of B11-G01): 26.3's
   `AbstractBedBlock` added the pair check; 26.1.1's `BedBlock`
   redirects a foot click to the head and requires only that the head
   is this bed.

The Group B mappings (StrawBed, SulfurSpike, ShelfMushroom,
PotentSulfur) are 26.2/26.3-only content carried harmlessly. The Mood
spacing is NOT 26.3-form — 26.1.1's own literal is one-spaced (see §6.6
G20), and the Rust matches it; the two-space form belongs to 26.2/26.3.

**The inheritance failure (measured, not assumed).** The 26.1.1 tree
was diffed file-by-file against the 26.1 tree in this batch's scope.
26.1 fixes the 26.1.1 tree does NOT carry: the bed-arm redirect
(interact.rs — 26.1's item-584 port, replaced by the 26.3 pair-check
form), the `RedStoneWireBlock` name in comments/docs (survive.rs:282,
interact.rs:492 — both regressed to the 26.3 spelling), the hardcoded
`burn_time`/`compost_chance`/`villager_food` tables (items.rs, 465
diff lines — replaced by component reads for `cooking_fuel` /
`brewing_fuel` / `compostable` / `villager_food` / `mob_visibility`,
components 26.1.1 vanilla does not define; Batch 4 documents the same
465-line diff), the composter chance form (interact.rs
`composter_add_item` → `composter_add_layer`; Batch 5's B5-G22 owns
the composter consequence), and the single-letter F3 category
abbreviations (debug_overlay.rs). Zero-diff files (26.1 behaviour
carries): classes.rs, splash_water.rs, game/social.rs, loot.rs. The
only 26.1.1-delta addition in scope is the `chat_disabled_or_blocked`
function + test (social_screen.rs, +23 lines). PORT-NOTES-26.1.1.md
says the completion pass "has since been carried forward into this
tree" — the diffs above show that carry-forward was partial at best
for behaviour code; the claim holds for the protocol/tables work, not
for the behaviour fixes. (The same inheritance failure is the
programme-wide systemic finding of the landed Batches 1–7.)

### §6.4 Re-base WARNING: 26.1.1 is 26.x where 1.21.7 is not — do not propagate 1.21.7 forms down

This batch found ELEVEN behaviours where 26.1.1's vanilla form differs
from 1.21.7's and the Rust already matches 26.1.1 — a re-base must not
propagate the 1.21.7 forms to 26.1.1: Riptide passenger check (26.1.1
HAS `isPassenger` in `releaseUsing`, offset 63), SwellGoal alive check
(26.1.1 HAS `isDeadOrDying` in `canUse` + `tick`), polar-bear fox-goal
age gate (26.1.1's selector is the `!isBaby` lambda,
`lambda$registerGoals$1`), lead-knot discard on failed tie (26.1.1
discards the fresh knot, offsets 139-152), mushroom bonemeal gate
(26.1.1 requires the huge-mushroom feature + `4 + foliageRadius`
height), sapling bonemeal height check (26.1.1:
`TreeGrower.getMinimumHeight` + build height, no `canGrow`), golem
stroll roll order (26.1.1 draws 0.3 first, 0.7 second — the Rust's
order; 1.21.7 stores the first roll and compares the second to 0.3),
social gate (26.1.1's chat check feeds ONLY the report button;
hide/show are ungated), trader heightmap + storage (26.1.1 registers
WANDERING_TRADER with MOTION_BLOCKING_NO_LEAVES and stores SavedData
`wandering_trader.dat`, fields `spawn_delay` / `spawn_chance`),
shipwreck `heightAdjusted` caching (26.1.1 HAS the field), player-data
layout (26.1.1 IS `players/data/`). The 1e-5 fluid-current floor:
present in vanilla 26.1.1 (the constant 9.999999747378752E-6 appears in
both float and double form in 26.1.1's `Entity` constant pool — the
class is byte-identical to 26.1's) AND in the Rust
(entity_fluid.rs:87, `length_squared() < 1.0e-5`). No floor-removal to
propagate; the 1.21.7 §6.4 warning (the template removed the floor for
1.21.11, but 1.21.7 HAS it) stands as a general per-version caution.
Three 26.1.1-vs-26.2 cautions also belong here: 26.1.1's F3 Mood
literal is ONE-spaced while 26.2's is TWO-spaced (do not propagate
26.2's form down); 26.1.1's F3 spawn-counts abbreviations are single
letters while 26.2's are two/three letters (B11-G09 — do not propagate
26.2's form down); and 26.1.1 vanilla HAS sound variants for
cows/pigs/chickens/cats (do not propagate 1.21.7's A02 note — see §5).
A fourth, from §6.3: 26.1.1 has NO `cooking_fuel`/`brewing_fuel`/
`compostable`/`villager_food`/`mob_visibility` components — do not
propagate the 26.3 component forms down; the 26.1.1 forms are the
hardcoded tables and `FuelValues`.

### §6.5 Class-absence headline: 0 of the ledger's tokens name 26.2-only or 26.3-only classes

The 6-item ledger is small, so the headline is small. 16 classlike
tokens were extracted from `work-items.json` (titles + note + rank
text) and checked against all four jar indexes (both 26.1.1 jars, both
26.2 jars, the 26.3 client jar):

- **7 real vanilla classes, ALL present in 26.1.1:** `DataFixers`,
  `DataVersion`, `DetectedVersion`, `Item`, `Items`, `PlayerEntry`,
  `SharedConstants`. Each is triaged in §8 (items 1–6). None is a
  26.2-only or 26.3-only class.
- **9 non-class tokens** (English words, file names, Rust names:
  `Derived`, `Generator`, `Horizon`, `Inherits`, `NOTES`, `PORT`,
  `Patch`, `Save`, `Version`) — absent from ALL FOUR jar indexes, 0
  real vanilla absences.

The class-absence story for this batch therefore lives in §6.1, not
in the ledger: the 8 dead class-table mappings (4 renames with live
arms dead, 4 content-absent) are all 26.2/26.3-only names, and the
component story of §6.3 (`COOKING_FUEL` etc. — fields on
`DataComponents`, not classes) is the same pattern at the component
level: the 26.1.1 tree names 26.2/26.3 surfaces 26.1.1 vanilla never
had. (For scale: the 26.1 run's 214-item ledger named 69 items with
26.2/26.3-only classes, 57 tokens; the 26.1.1 ledger's 6 items name
none — the absence lives in the carried-forward code, not in the
delta's ledger.)

### §6.6 1.21.7 Batch 11 defects re-checked against 26.1.1

Every 1.21.7 Batch 11 defect was re-checked against 26.1.1's jar/data/Rust.
Seven carry over (plus one cosmetic detail from P01) — the same seven
as 26.1 — plus one NEW F3 defect (G09) that 26.1 did not have; the
rest are withdrawn with the deciding evidence. The 26.1 run's
"26.2-record corrections" (summary.md line 283) were applied: none of
them touches this batch's scope — B3-G27/G28/G25/G26 are Batch 3's
(horse food, furnace minecart, BreedGoal), B9-G11 is Batch 9's (recipe
suggestions), and B6's `feature_tags`/`dripstone_cluster` notes are
Batch 6's. No correction changed any verdict below.

**APPLIES (minted as 26.1.1 items):**

- G01 beds — APPLIES (B11-G01 above). Major. 26.1.1 HAS a bed BE type
  and `BedBlock` — but the arms dispatch on the dead `AbstractBed`,
  and the arm logic is now the 26.3 pair-check form too.
- G02 redstone wire — APPLIES (B11-G02 above). Major. 26.1.1 has
  `RedStoneWireBlock`; the table has only the 26.3 name, no dual.
- G03 farmland/paths — APPLIES in part (B11-G03 above): paths dead
  (`DirtPathBlock` vs `PathBlock`); farmland live.
- G06 speleothem — APPLIES (B11-G04 above). 26.1.1 has neither the
  class nor the tag.
- G12 number providers — APPLIES (B11-G06 above), datapack-only.
- G13 splash water — APPLIES (B11-G05 above). Major. The Rust's tag
  gate is dead in 26.1.1 (only `tradeable` exists).
- G15 fuel order — APPLIES (B11-G07 above), in a worse form: the
  component-based list is empty, and the order it would cycle is not
  vanilla's FuelValues order.
- P01 F3 chart footer — SETTLED BY THE JAR, with one cosmetic delta
  (B11-G08 above): layout matches; only the `fps` casing differs.
- (NEW) F3 spawn-counts abbreviations — B11-G09 above: the 26.1 tree
  was fixed; the 26.1.1 tree regressed to the 26.2+ form.

**Withdrawn (verified not-applicable, with the deciding evidence):**

- G04 snowy — SnowyBlock/SpreadingSnowyBlock live in the 26.1.1 jar +
  chains (§6.1).
- G05 fungus/roots — NetherFungusBlock/NetherRootsBlock live (§6.1).
- G07 sapling bonemeal — 26.1.1's gate IS the height check
  (`getMinimumHeight` + build height, javap-verified); Rust matches
  (bonemeal.rs:331-340). (Batch 5's B5-G29 records a SEPARATE
  `canGrow`-gate issue for lone dark/pale oak saplings — different
  mechanism, Batch 5's item.)
- G08 riptide riding — 26.1.1 vanilla HAS the isPassenger check
  (releaseUsing offset 63); Rust blocks while riding
  (trident.rs:501).
- G09 lead knot — 26.1.1 vanilla discards the fresh knot on a failed
  tie (offsets 139-152); Rust creates the knot only if attached
  (leads.rs:640-653). Same observable outcome.
- G10 creeper defuse — 26.1.1 vanilla HAS isDeadOrDying in SwellGoal
  canUse + tick; Rust gates on t.alive (hostile.rs:294, :312).
- G11 polar-bear cubs — 26.1.1's fox-goal selector IS !isBaby
  (`lambda$registerGoals$1`); Rust matches (polar_bear.rs:51-56).
- G14 player-data layout — 26.1.1 vanilla IS players/data/
  (LevelResource.PLAYER_DATA_DIR = "players/data", constant-pool
  verified); Rust matches (lan/playerdata.rs:18).
- G16 golem stroll + snow tags — stroll roll order matches 26.1.1
  exactly (0.3 first, 0.7 second — mobs/golem.rs:152/155; vanilla
  class byte-identical to 26.1's, whose order the 26.1 run
  javap-verified); snow tags are the 26.x names, present (§6.2).
- G17 mushroom — 26.1.1's gate genuinely requires the feature + height
  (`4 + foliageRadius`, javap-verified); Rust matches
  (bonemeal.rs:342-356); light tag present (§6.2).
- G18 social gate — 26.1.1's chat check feeds ONLY the report button
  (PlayerEntry ctor offsets 100-123, 157-214); hide/show are ungated;
  Rust matches (no chat gate on hide/show in game/social.rs; the
  report-button formula is item 5, §8). NOTE: 26.1.1 FLIPPED the
  report-button condition vs 26.1 (`!canReceive || isBlocked`) — the
  Rust pins the flipped 26.1.1 form.
- G19 trader — heightmap MOTION_BLOCKING_NO_LEAVES matches
  (WanderingTraderSpawner uses `SpawnPlacements.getHeightmapType`;
  the registration class is byte-identical to 26.1's); storage is
  SavedData wandering_trader.dat with `spawn_delay`/`spawn_chance`,
  matches (wandering_trader.rs:48, :61-62, :74; `WanderingTraderData`
  javap-verified).
- G20 F3 lines — ALL THREE thirds match 26.1.1: Local Difficulty has no
  Day suffix (`"Local Difficulty: %.2f // %.2f"`, day count is the
  separate `DebugEntryDayCount` entry `"Day #"`, which the Rust renders
  as `"Day #{}"` at debug_lines.rs:57); memory lines use MiB
  (`"Mem: %2d%% %03d/%03dMiB"`); and the Mood literal is ONE-spaced in
  26.1.1 (`' (Mood %d%%)'`, constant-pool verified) — the Rust's
  one-space form matches 26.1.1 exactly. (26.2's literal is
  two-spaced; the one-space form is a 26.1.1-form match, not a defect
  here.) The spawn-counts line is a FOURTH F3 element and is defective
  (B11-G09) — it was not part of 1.21.7's G20.
- G21 chorus — tags present in 26.1.1 data (§6.2).
- G22 dialog tooltip — 26.1.1's ItemDisplayWidget shows its tooltip on
  `tooltip && isHovered()` only (offsets 97-107; the `isFocused()`
  call at offset 69 draws the focus OUTLINE, not the tooltip);
  Rust matches (hover-only, game/dialogs.rs:705).
- G23 cauldron wash — tag present with leather armour (§6.2); Rust
  gate reads it (horizon-client/src/cauldron.rs:7-12, :214).
- H01 shipwreck — 26.1.1 HAS heightAdjusted; Rust caching matches
  (worldgen/structure/saved_templates.rs:178, simple.rs:250).
- H02 cauldron table — premise withdrawn with G23 (see §3); the two
  cauldron oracles are recorded as B11-O01/O02 (§2).
- A01 trim inline form — CARRIES as an accepted note (B11-A01, §5).
- A02 sound_variant — NEVER EXISTED for 26.1.1 (see §5): 26.1.1 vanilla
  has sound variants for cows/pigs/chickens/cats.
- A03 special-foil decal — CARRIES as an accepted note (B11-A02, §5).
- A04 separator + per-player resolution — CARRIES as an accepted note
  (B11-A03, §5).

## §7 Method notes

- **Item source:** 26.1.1's own port ledger,
  `versions/26.1.1/work-items.json` (6 items, all closed; ranks
  3/1/1/0/1 — item 5 rank 3, items 1–3 rank 1, item 4 rank 2, item 6
  rank 5). There is NO `rank3-progress.json` and no port triage
  workdir for this version (26.1.1 is a patch delta). The 6 items
  were triaged individually: item 5 re-verified in the current tree
  (javap diff of the vanilla ctor + the Rust function + its test);
  the plumbing items by family-pass discipline (version facts /
  DataFixer / SharedConstants / DetectedVersion claims checked
  against the 26.1.1 jar). `versions/26.1.1/PORT-NOTES-26.1.1.md`
  was consulted and found STALE on item 5 (it says the PlayerEntry
  fix was "flagged here, not ported in this delta" — the tree DOES
  carry the fix, in `crates/horizon-client/src/social_screen.rs`,
  not in `game/social.rs` as work-items.json's files list says).
- **Jar evidence:** named 26.1.1 server-game jar
  (`26.1.x/versions/26.1.1/jars/server-game.jar`, 7,351 classes) +
  26.1.1 client jar (10,682 classes), both Mojang-mapped. The 26.1.1
  server jar in the same dir is a Mojang BUNDLER jar and was never
  javap'd. **Jar diff (the key structural result):** every `.class`
  file was sha1-compared between the 26.1 and 26.1.1 jars — the ONLY
  changed classes are `PlayerEntry` (client only),
  `DetectedVersion`, and `SharedConstants`. Every other class is
  byte-identical, so the 26.1 run's javap evidence carries exactly
  for unchanged classes; the defect-bearing classes were still
  re-verified directly against the 26.1.1 jars (AbstractThrownPotion,
  PotionTags, AnyFuel, FuelValues, DebugScreenOverlay + the four
  DebugEntry classes, ProvidesTrimMaterial, BedBlock,
  BlockEntityType, Level, ServerLevel, TridentItem, LeadItem,
  SwellGoal, PolarBear, MushroomBlock, SaplingBlock, SnowGolem,
  WanderingTraderSpawner, WanderingTraderData, ShipwreckPiece,
  LevelResource, ItemDisplayWidget, MobCategory, DataComponents,
  PlayerEntry 26.1-vs-26.1.1 diff). The 26.2 jars
  (`26.1-adapters/versions/26.2/jars/`, server-game 7,445 + client
  10,952) were indexed ONLY for the §6.5 26.2-only/26.3-only split
  and the §6.2/§6.3 comparisons. The 26.3 client jar
  (`jars-26.3/client-26.3.jar`, 11,383 classes) was used ONLY for the
  §6.5 absence comparison and the §6.3 26.3-form checks — never as
  the spec. javap via JDK 25 (`~/workspace/tools/jdk-25/bin/javap`).
  Class-existence checks used zipfile indexes of the jars (in
  scratch: `client26.1.1-classes.json`, `server26.1.1-classes.json`,
  `client26.2-classes.json`, `server26.2-classes.json`,
  `client26.3-classes.json`), not per-class javap. Five string
  literals (Mood spacing, Day #, Mem, Local Difficulty, players/data)
  were verified by parsing the class files' constant pools directly,
  since javap's `// String` rendering makes leading-space counts
  ambiguous.
- **Tree diff:** the 26.1.1 tree was diffed file-by-file against the
  26.1 tree in this batch's scope (classes.rs, interact.rs,
  survive.rs, splash_water.rs, social_screen.rs, game/social.rs,
  loot.rs, items.rs, recipe_book.rs, debug_keys.rs, debug_overlay.rs)
  — the §6.3 inheritance-failure finding rests on those diffs, not
  on assumption. classes.rs, splash_water.rs, game/social.rs and
  loot.rs are 0-diff; items.rs is 465 diff lines (the component
  regression Batch 4 documents); interact.rs 58, social_screen.rs 23
  (the item-5 addition), recipe_book.rs 19, debug_overlay.rs 10,
  debug_keys.rs 5, survive.rs 2.
- **Class-table sweep:** `class-table-sweep.json` in scratch (271
  mappings × jar-liveness × chain-liveness). Chains from the prepared
  `block-behaviours.json` (1,168 blocks, 291 distinct chain names).
  The chain half ran for the second time in the programme (26.1 was
  the first) and agrees exactly with the jar half again.
- **Data checks:** tag files + contents from the 26.1.1 client-jar
  datapack (775 tag files) and the prepared data; block/item
  registries from the prepared `registries.json` (oak planks
  protocol id 36, coal 897, lava bucket 1015); loot-table provider
  usage counted over all 1,326 prepared vanilla loot tables (0 uses
  of the 3 missing providers; 0/1,331 client-jar loot-table files
  likewise); `enchantment_level` uses found only in
  `enchantment/soul_speed.json` and `enchantment/thorns.json`;
  trim-material items counted in prepared items.json (11, all string
  keys); fuel-component census over prepared items.json (0 of 1,506
  items carry `minecraft:cooking_fuel` — matches Batch 4's count).
- **Oracle triage:** from the Batch 0 measured table ONLY
  (`docs/sweep/26.1.1/batch-0-oracles.json`, 189/391 compile), the
  two in-scope cauldron oracles triaged with the missing symbol
  javap-verified (§2). No oracle was repaired (Batch 0 scope decision).
- **Cross-batch overlap check:** the landed Batch 1–7 reports in
  `docs/sweep/26.1.1/` were grepped for this batch's defects before
  minting, so every overlap is flagged with the owning item id (§1
  header + per-item notes). No item was folded; the coordinator
  dedups at merge.
- **No cargo builds, no driver runs, no git commits.** Static checks
  only (javap + python + grep), per the batch constraints. No file
  under `versions/26.1.1/` was modified; no git command was run in
  the worktree. The horizon-client lib-test breakage behind B11-H01
  was NOT independently re-verified by a build in this batch — it is
  recorded from the work-items note, as the brief instructs.
- **Re-verification caveat:** all Rust greps ran against the sweep
  worktree (`~/workspace/mcir-work/sweep-26.1.1/versions/26.1.1/`,
  branch parity-sweep-26.1.1); line numbers are from that tree.
- **Unproven / stalled:** nothing stalled. Two things are unproven by
  design in this batch: (1) B11-H01 — item 5's regression test needs
  the lib-test breakage fixed first; (2) the golem-stroll roll order
  and the trader heightmap registration were verified on the Rust
  side plus the byte-identity of the vanilla classes to 26.1's (whose
  order/registration the 26.1 run javap-verified) — no fresh javap
  of those two vanilla methods was run against the 26.1.1 jar, since
  the class files are byte-identical to 26.1's and the jar-diff
  proves it.

## §8 Per-item triage table

Verdicts: **pass** (verified against the 26.1.1 jar/data + Rust),
**needs-harness** (item 5 — the fix is in the tree but its regression
test cannot run). No item verdicts gap/probe/accepted-gap in this
batch; the nine defects are cross-cutting (§1) and the three accepted
notes are 1.21.7-sweep carries (§5), not ledger items.

| Item | Verdict | Evidence |
|---:|---|---|
| 1 | pass | Family: `vanilla_command_order` generator table — `crates/horizon-command/src/vanilla_order.rs` exists with the generated `ORDER` table and is byte-identical to the 26.1 tree's (command nodes unchanged 26.1→26.1.1, per the jar-diff: no command class changed). PORT-NOTES records the carry-forward regeneration reproducing the table byte-identically against the 26.1.1 server jar. |
| 2 | pass | Family: DataFixers 4786→4788, 0 schemas / 0 fixes / 0 datafix classes changed — the jar-diff proves NO datafix class changed 26.1→26.1.1 (only PlayerEntry, DetectedVersion, SharedConstants differ); `DataFixers` exists in both 26.1.1 jars. The bump flows through `DATA_VERSION` alone: datafix/mod.rs:40 `CURRENT = crate::DATA_VERSION` (4788, version.rs:11), and save.rs:207-215 gates world upgrades on `DATA_VERSION`. No new fixer needed or added. |
| 3 | pass | Family: version facts — version.rs:5 `VERSION_ID = "26.1.1"`, :9 `PROTOCOL_VERSION = 775`, :11 `DATA_VERSION = 4788`, matching the 26.1.1 jar's `SharedConstants` (`WORLD_VERSION = 4788`, `RELEASE_NETWORK_PROTOCOL_VERSION = 775`, javap `-constants` verified) and the research-lock pins (DataVersion 4788, protocol 775, resource pack 84.0, data pack 101.1). |
| 4 | pass | Family: `SharedConstants` fields changed (WORLD_VERSION, SNAPSHOT_NETWORK_PROTOCOL_VERSION) — jar-verified: `WORLD_VERSION = 4788`, `RELEASE_NETWORK_PROTOCOL_VERSION = 775`, `SNAPSHOT_NETWORK_PROTOCOL_VERSION = 304`. The Rust carries the version facts in version.rs (item 3) and the protocol comment in horizon-protocol/src/lib.rs:92-97 documents 775 as unchanged from 26.1. Absorbed by the `version_consts` regen; nothing else to port. |
| 5 | needs-harness | Individually re-verified: vanilla's `PlayerEntry.<init>` differs 26.1→26.1.1 by EXACTLY one instruction (javap diff: offset 106 `ifne 118` → `ifeq 118`), flipping `chatDisabledOrBlocked` from `canReceive \|\| isBlocked` (26.1's inverted form) to `!canReceive \|\| isBlocked`. The Rust pins the 26.1.1 formula: `chat_disabled_or_blocked` (social_screen.rs:130-132, `!can_receive_player_messages \|\| blocked`) with a four-case unit test (:454-462). NOTE: the fix lives in `crates/horizon-client/src/social_screen.rs`, NOT in `game/social.rs` as work-items.json's files list says, and PORT-NOTES ("flagged here, not ported") is stale — the tree carries the fix. The function is currently uncalled (the report flow is not offered; the row shows the disabled report button vanilla shows when reporting is off). Verdict is needs-harness, not pass, per the brief: the work-items note records the regression test cannot run until the pre-existing horizon-client lib-test breakage is fixed (B11-H01, §3). |
| 6 | pass | Family: `DetectedVersion.createBuiltIn` changed (DataVersion constant only) — jar-verified: `createBuiltIn(String, String, boolean)` pushes `4788` (`sipush 4788`, offset 29); the only change vs 26.1 is the constant. Horizon does not port DetectedVersion: 0 references in the 26.1.1 tree's Rust (grep-confirmed). Nothing to port. |
