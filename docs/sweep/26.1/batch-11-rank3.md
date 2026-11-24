# Batch 11 — Rank-3 triage (214 items), 26.1 parity sweep

- **Batch:** 11 (rank-3 triage) — checks + triage only, no fixes.
- **Date:** 2026-11-24
- **Scope:** 26.1's OWN 214 rank-3 ledger items
  (`versions/26.1/rank3-progress.json`, the `closed` list: triage string
  A1 60 / A2 67 / B1 56 / B2 31; verdicts no-change 179 / ported 35, no
  accepted-gap verdicts in the file), triaged against the named 26.1 jars
  and the 26.1 port in `versions/26.1/crates/` (branch
  `parity-sweep-26.1`). The items are 26.2→26.1 diffs (26.2 is 26.1's
  parent version — item evidence cites "26.2 holds…" vs "26.1 holds…").
  Vanilla 26.1's own jar/data is the spec throughout; the 26.3 base's
  form was never trusted. **Scope difference from the 1.21.7 Batch 11:**
  that batch triaged the 1.21.11 TEMPLATE's 247 rank-3 items FOR 1.21.7;
  like the 26.2 run, this batch verifies 26.1's own port ledger — i.e.
  each "closed" verdict is checked to actually hold in the current tree.
- **Ledger slice:** `slice.json` (same shape as the 26.2 batch-11 slice).

## Tally (214 items)

| Verdict | Count |
|---|---|
| pass | 214 |
| gap (defect attached) | 0 |
| needs-new-harness | 0 |
| needs-in-game-probe | 0 |
| accepted gap (ledger verdict) | 0 |

Defects found: **8** (`B11-G01`–`B11-G08`), of which 3 major, 5 minor.
All eight are cross-cutting (class-table sweep + 1.21.7-defect
cross-checks) with no single owning rank-3 item — they are recorded as
defects, not item verdicts, so the defect count (8) exceeds the gap-item
count (0). Oracle defects: **2** (`B11-O01`–`B11-O02`, both minor, both
cauldron oracles). Accepted-gap notes carried from the 1.21.7 sweep:
**3** (`B11-A01`–`B11-A03`, §5 — not ledger verdicts, not counted in the
214-item tally). No harness or probe items: the 26.2 run's B11-H01
(chain-liveness half) is CLOSED by this run — the chain half ran against
the landed prepare (§6.1) — and the 1.21.7 P01 probe is settled by the
26.1 jar (§6.6).

Headline: the 26.1 port's rank-3 ledger holds up — all 35 "ported"
verdicts re-verify in the current tree and every no-change item checks
out against the 26.1 jar/data. But the class table is in worse shape
than 26.2's: the 26.1 tree carries NO dual names, so four 1.21.7
class-rename defects reproduce here that 26.2 had already fixed (beds,
redstone wire, dirt paths, dripstone), and the splash-water tag gate
that 26.2 fixed to `Potions.WATER` equality is still the dead 26.3 tag
form in 26.1. Of the 23 defects the 1.21.7 Batch 11 found, **seven**
carry over to 26.1 (G01, G02, G03-paths-half, G06, G12, G13, G15) plus
one cosmetic detail from P01 — against only one major carry-over
(beds) in 26.2. The pattern is consistent: 26.1 predates the 26.2
renames (`RedStoneWireBlock`→`RedstoneWireBlock`,
`DirtPathBlock`→`PathBlock`, `PointedDripstoneBlock`→`SpeleothemBlock`
as the abstract parent) that the 26.3-derived class table assumes, and
the 26.1 port never added the dual names 26.2 carries.

## §1 Defects

### B11-G01 — Beds place foot-only; sleep is unreachable (major; = 1.21.7 B11-G01, = 26.2 B11-G01)

`Class::AbstractBed = "AbstractBedBlock"` (classes.rs:59) never matches
any 26.1 chain: 26.1 has no `AbstractBedBlock` — beds are `BedBlock`
directly (jar-verified: `BedBlock extends HorizontalDirectionalBlock`,
implements `EntityBlock`; no `AbstractBedBlock` in either 26.1 jar;
`BlockEntityType.BED` exists and covers all 16 bed blocks). The
prepared chain for `minecraft:white_bed` is
`[BedBlock, HorizontalDirectionalBlock, Block]`. `Class::Bed =
"BedBlock"` (classes.rs:60) exists but NOTHING dispatches on it
(grep-confirmed across all crates). Every bed arm is dead:

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
AbstractBedBlock`, javap-verified), so the mapping is 26.3-correct and
26.1-wrong. Note the irony at interact.rs:366-369: item 584 ported the
arm's LOGIC to the 26.1 form (the comment documents 26.1's
`BedBlock.useWithoutItem` foot→head redirect, "26.3's AbstractBedBlock
added the pair check") while the arm itself still dispatches on the dead
26.3 name — the port fixed the inside of the arm but not the dispatch.
The fix is a 26.1-direct one (dispatch on `Class::Bed`), not a re-base
propagation.

### B11-G02 — Redstone wire never updates power (major; = 1.21.7 B11-G02; 26.2 fixed this with a dual name)

26.1 names the class `RedStoneWireBlock` (capital S, jar-verified in
both jars; the prepared chain for `minecraft:redstone_wire` is
`[RedStoneWireBlock, Block]`). The table maps `RedstoneWireBlock` only
(classes.rs:224) — the 26.3 name, absent from both 26.1 jars — and the
26.1 tree carries NO dual name (26.2's tree maps
`"RedstoneWireBlock" | "RedStoneWireBlock"`). All ten wire arms are
dead: hooks.rs:100/211/380 (neighbour changed / notify / piston move),
interact.rs:263 (`wire_use`, the dot/cross toggle), place.rs:1089
(placement connection state), redstone.rs:65/101 (`wire_signal`
dispatch), redstone.rs:219 (`should_wire_connect_to`), shapes.rs:698
(`update_shape` → `redstone_wire_update_shape`), survive.rs:281
(`canSurvive`). Decisively, `wire_update_power` has exactly THREE
callers — all inside the dead hooks.rs arms (:105, :213, :385) — so a
wire's power is never recomputed after placement: it keeps power 0
forever. The name-based reads (`direct_signal` redstone.rs:205,
`is_signal_source` :36, `wire_power_of` :415) only ever read the stale
power. Same comment-irony as beds: survive.rs:282 cites
`RedStoneWireBlock.canSurvive` (the correct 26.1 name) while the arm
dispatches on the dead 26.3 name. Also dead in prepare:
horizon-prepare's survival.rs:43 maps `("RedstoneWireBlock",
"RedstoneWire")`, which misses 26.1's `RedStoneWireBlock` chain name
(same for `PathBlock` and `SpeleothemBlock` in that table). Severity
major: redstone is core gameplay. **26.3-leak:** `RedstoneWireBlock`
exists in the 26.3 jar.

### B11-G03 — Dirt paths never revert (minor; = the paths half of 1.21.7 B11-G03)

26.1 has `DirtPathBlock` (jar-verified; chain `[DirtPathBlock,
Block]`); the table maps `PathBlock` only (classes.rs:208) — the 26.3
name, absent from both 26.1 jars, no dual name. All four Path arms are
dead: place.rs:929 (placement state), shapes.rs:465 (updateShape —
shared with the live Farmland arm), survive.rs:273 (`canSurvive`: solid
above kills the path), tick.rs:202 (random tick →
`turn_to_base_block`). `turn_to_base_block` has only three callers —
tick.rs:199 (Farmland, live), :202 (Path, dead), :1046 (farmland
moisture, live) — so no name-based path revert exists: a covered or
trampled dirt path never turns back to dirt. The farmland half of
1.21.7's B11-G03 does NOT reproduce: `FarmlandBlock` is live in 26.1
(chain `[FarmlandBlock, Block]`, classes.rs:151) and its arms work.

### B11-G04 — Dripstone placement/survival/growth dead (minor; = 1.21.7 B11-G06)

26.1 has no `SpeleothemBlock` at all (absent from both jars; item 542:
speleothems are a 26.2 addition). Pointed dripstone is
`PointedDripstoneBlock` directly (chain `[PointedDripstoneBlock,
Block]`). The table maps `Speleothem = "SpeleothemBlock"`
(classes.rs:215) and all five Speleothem arms are dead: place.rs:1156
(placement state), shapes.rs:728 (updateShape), survive.rs:307
(`canSurvive`), tick.rs:248 (scheduled tick — destroy/fall),
tick.rs:434 (random tick → `speleothem_random_tick`, growth +
cauldron-fill). There is NO `Class::PointedDripstone` arm at any of
those five dispatch sites — `PointedDripstone` (classes.rs:216, live)
is used only inside speleothem.rs helpers, which are themselves reached
only via the dead Speleothem arms. Second independent cause, as in
1.21.7: the `minecraft:speleothems` TAG is absent from 26.1 data (26.2
has it), so even the tag-gated helpers (`speleothem_tag`,
`speleothem_with`, `speleothem_is_tip` in speleothem.rs:16-31) return
false for every block. In 26.2 the arms were live because
`SpeleothemBlock` is `PointedDripstoneBlock`'s abstract parent there;
in 26.1 neither the class nor the tag exists.

### B11-G05 — Splash water never douses fire, hurts, extinguishes, or rehydrates (major; = 1.21.7 B11-G13; 26.2 fixed this)

Vanilla 26.1 `AbstractThrownPotion.onHitBlock` gates on
`potionContents.is(Potions.WATER)` — direct holder equality, NO tag
(bytecode 55-63, javap-verified on the 26.1 server jar), and
`onHitEntity` routes water through `onHitAsWater` the same way
(bytecode 46-59). The Rust gates all four behaviours on potion TAGS
(game/splash_water.rs: `potion_in_tag`, :42-51; douse :60, hurt/extinguish
:134-135, rehydrate :197), but 26.1 has NO potion tags beyond
`tradeable`: `PotionTags` has ONLY `TRADEABLE` (javap-verified), the
client-jar datapack holds only `tags/potion/tradeable.json`, and the
prepared data's `tags/potion/` likewise — so the `potion_tags` map
(merchant.rs:489, :574) never contains `douses_fire`,
`hurts_water_sensitive_entities`, `extinguishes_entities`, or
`rehydrates_axolotls`, and every gate is always false. The douse area
matches vanilla exactly; only the gate diverges. **26.3-leak:** 26.3's
datapack HAS all four tag files, so the tag gate is 26.3-correct and
26.1-wrong. (26.2's tree was fixed to `Potions.WATER` equality; the
26.1 tree still carries the tag form.) Severity major: splash water is
the fire-dousing mechanic.

### B11-G06 — Three number providers return 0 (minor, datapack-only; = 1.21.7 B11-G12, = 26.2 B11-G02)

Vanilla 26.1 has 6 number providers (`EnchantmentLevelProvider`,
`ScoreboardValue`, `StorageValue` all present in the server jar). Both
Rust implementations handle only 3 (`minecraft:constant`,
`minecraft:uniform`, `minecraft:binomial`) — `minecraft:enchantment_level`,
`minecraft:score`, `minecraft:storage` fall to `_ => 0` (horizon-world
loot.rs `int`/`float` 1329-1346; horizon-items items.rs `number`
2033-2050). Data check: 0/1,326 vanilla 26.1 loot tables use the 3
missing providers — vanilla loot is unaffected. The only
`enchantment_level` uses in 26.1's datapack are the `chance` providers
in `enchantment/soul_speed.json` and `enchantment/thorns.json`, and
those go through the SEPARATE enchant.rs `random_chance` path, which
handles `enchantment_level` explicitly (enchant.rs:874-884:
`level_value(amount, level)`) — verified against vanilla 26.1's
`EnchantmentLevelProvider.getFloat`, which is `amount.calculate(level)`
with NO multiplication by level, exactly what the Rust computes. So the
gap is datapack-only: datapacks using the three providers in LOOT get 0.

### B11-G07 — Recipe-book fuel slot cycles fuels in the wrong order (minor; = 1.21.7 B11-G15, = 26.2 B11-G03)

Vanilla 26.1 `SlotDisplay$AnyFuel.resolve` returns
`FuelValues.fuelItems()` — the keySet of an insertion-ordered
`Object2IntLinkedOpenHashMap` (javap-verified: the Builder constructs
one, `FuelValues$Builder.<init>` offsets 5-12) populated by the
HARDCODED `FuelValues.vanillaBurnTimes` sequence (LAVA_BUCKET 20000,
COAL_BLOCK 16000, …; javap-verified). First fuel shown: lava bucket.
The Rust groups fuels by burn-time value in registry order
(recipe_book.rs:566-585, native_panel_recipes.rs:173-190) — in registry
order the first fuel is oak planks (protocol id 63; coal 924, lava
bucket 1042), so the first-shown fuel is a 300-tick plank, not lava
bucket. Burn TIMES unaffected — display order only. **26.3-form
carry-over:** 26.3's `AnyFuel.resolve` uses
`DataComponentLookup.findAll(COOKING_FUEL)` (javap-verified on the 26.3
client jar) — the grouping-by-component behaviour the Rust implements;
26.1's insertion-ordered FuelValues form is what the Rust should match.

### B11-G08 — F3 chart footer uses lowercase fps (minor, cosmetic; part of 1.21.7 B11-P01, now settled by the jar)

Vanilla 26.1's `DebugScreenOverlay` builds the chart footer as three
list entries (javap-verified): `Debug charts: {Profiler}; {FPS + TPS |
FPS};`, `{Bandwidth + Ping | Ping}; {Lightmap};`, `To edit: press
{key}` — with the chart names `FPS + TPS` / `FPS` in UPPERCASE (string
constants in the class). The Rust's `debug_chart_lines`
(debug_keys.rs:253-295) matches the three-line layout and the
single-player variants exactly, but names the chart `fps` / `fps + tps`
in lowercase (:285-286). Only the fps chart name differs in case
(Profiler, Bandwidth + Ping, Ping, Lightmap all match). Cosmetic. The
layout question the 1.21.7 probe item left open is settled: no in-game
probe is needed — the jar gives the footer outright.

## §2 Oracle defects

No oracle repairs were made (Batch 0's scope decision: measurement +
triage only; the oracle `.java` files live under `versions/26.1/` and
this sweep forbids touching them). Two cauldron oracles in this batch's
scope fail against the 26.1 jars, both triaged from the Batch 0 measured
table (`docs/sweep/26.1/batch-0-oracles.json`) with the missing symbol
javap-verified against the 26.1 jar — not from an import-level check.
Both are the same two oracles the 1.21.7 sweep met (B11-O01 there was
repaired; here they are recorded only).

### B11-O01 — VanillaCauldronOracle fails: `net.minecraft.util.Prediction` absent in 26.1 (minor, oracle-defect)

Batch 0: fails on all three classpaths, class `missing-symbol`,
first error `118: cannot find symbol`, 1 error. Line 118 is the
`FakePlayer.drop(ItemStack, boolean, net.minecraft.util.Prediction)`
override — the 26.3 3-arg drop form. `Prediction` is absent from both
26.1 jars (jar-index verified) and present in the 26.3 client jar
(`net.minecraft.util.Prediction`). 26.1's drop is the 2-arg form, as in
1.21.7 (whose B11-O01 repaired exactly this). Repair (follow-up, not
done here): drop the `Prediction` parameter, as 1.21.7's B11-O01 did
(commit c754e3a2 there). This oracle is the driver behind the cauldron
table comparison (1.21.7's B11-H02); with G23 withdrawn for 26.1 the
comparison is no longer confirmatory for a live defect, but the oracle
remains broken for any future cauldron driver work.

### B11-O02 — VanillaLavaCauldronItemOracle fails: `getNextEntityId` absent in 26.1 (minor, oracle-defect)

Batch 0: fails on all three classpaths, class `no-override`, first
error `28: method does not override or implement a method from a
supertype`, 1 error. Line 28 is `@Override public int
getNextEntityId()` on the `Lvl extends ServerLevel` stand-in.
`getNextEntityId` is declared on NEITHER `Level` nor `ServerLevel` in
the 26.1 server jar (javap `-p` grep on both classes; `registryAccess`
IS present on `ServerLevel`, so the neighbouring override at line 29 is
fine). The method is a 26.2/26.3 addition the oracle assumes. Repair
(follow-up): remove the override (the stand-in's `next` counter is only
used by the oracle itself) or move the id allocation out of the
`ServerLevel` subclass.

In-scope oracles assessed PASSING (from the Batch 0 table, for
auditability): no other oracle in this batch's scope was assessed —
Batch 11 runs no drivers and its only in-scope oracles are the two
cauldron oracles above, both failing. The full failing-oracle list
(192 files) belongs to the batches whose drivers use them; the
coordinator dedups at merge.

## §3 Needs-new-harness

None. Two notes on inherited harness items:

- **26.2's B11-H01 (class-table chain-liveness half) is CLOSED by this
  run.** Batch 0's prepare landed for 26.1, so `block-behaviours.json`
  exists (1,168 blocks) and the chain half ran (§6.1): 263/271 mappings
  chain-live, 0 jar-live-but-chain-dead. The 26.2 H01's data
  precondition is met for 26.1 and the re-run itself is done here, not
  deferred.
- **1.21.7's B11-H02 (exhaustive cauldron-table driver comparison)
  does NOT carry.** Its premise was confirming B11-G23 (the wash gap);
  G23 is withdrawn for 26.1 (§6.6 — the tag is present and the gate is
  live). The comparison would still be confirmatory for the rest of the
  cauldron table, but the driver is blocked on the two oracle defects
  above (B11-O01/O02) and the comparison itself needs a probe build this
  batch does not run. No new harness item is minted: nothing in this
  batch's scope needs a harness to settle.

## §4 Needs-in-game-probe

None. The 1.21.7 batch's probe item (F3 chart footer, P01) does not
carry: 26.1 HAS the `DebugScreenOverlay` chart footer in the jar, and
the Rust's three-line form matches it — the only delta is the lowercase
`fps` name, recorded as B11-G08. No rank-3 item needed a probe to
settle.

## §5 Accepted gaps / notes (not defects)

The 26.1 port ledger carries NO accepted-gap verdicts (unlike 26.2's
items 427/436), so none of these is a ledger item and none is counted
in the 214-item tally. All three are notes carried from the 1.21.7
sweep, re-verified against 26.1:

- **B11-A01 — ProvidesTrimMaterial inline/direct form [= 1.21.7
  B11-A01].** Vanilla 26.1's `ProvidesTrimMaterial.material` is
  `Holder<TrimMaterial>` (javap-verified) — a Holder direct OR a
  ResourceKey string. The Rust reads the string form only
  (enchant.rs:333-335, `.as_str()`). All 11 vanilla 26.1 items with the
  component use string keys (prepared items.json: amethyst, copper,
  diamond, emerald, gold, iron, lapis, netherite, quartz, redstone,
  resin) — the direct/inline form is datapack-only.
- **B11-A02 — Special-foil decal not modelled [= 1.21.7 B11-A03].**
  The Rust HAS a `Foil::Special` variant ("compasses and clocks, a
  sheeted decal", item_render.rs:140-146) carried through `submit_quads`
  (:449), but both sinks treat every non-None foil identically: the GUI
  sink draws the glint over the full quad (gui.rs:421-423) and the world
  sink likewise (item_render.rs:1155-1173) — no 0.5x GUI / 0.75x
  first-person decal scaling. Only visible on a command-enchanted
  compass/clock. Pre-existing in both versions, not a version delta.
- **B11-A03 — NBT separator ignored + per-player resolution [= 1.21.7
  B11-A04].** The Rust's `component_arg` (text_commands.rs:60-135)
  resolves selectors ONCE for the command source and always joins names
  with `", "` — the separator field of nbt/selector components is
  ignored and a title/tellraw component is not re-resolved per player
  (vanilla resolves per player). Gaps from BOTH versions equally, not
  version deltas.

NOT carried: 1.21.7's B11-A02 (NBT `sound_variant` written on
non-wolf mobs) NEVER EXISTED for 26.1 — 26.1 vanilla HAS sound variants
for cows, pigs, chickens and cats (`sound_variant` referenced by
`Chicken`, `Cow`, `Cat`, `Pig` and `Wolf` classes in the 26.1 server
jar), and the Rust's `sound_variants` (finalize.rs:2368-2377) lists
exactly those five species. Writing the tag is correct 26.1 behaviour.

## §6 Systemic findings

### §6.1 The class table: 8 dead mappings, 4 with live arms — and the chain half RUNS

Method: parse every `Class::X = "Name"` in
`versions/26.1/crates/horizon-world/src/level/classes.rs` (271
mappings — the table has NO dual names at all, unlike 26.2's, which
carries `"PathBlock" | "DirtPathBlock"` and `"RedstoneWireBlock" |
"RedStoneWireBlock"`); LIVE iff the name exists as a class (top-level
or nested) in either 26.1 jar (server-game 7,351 classes; client
10,682). Chain half (new for 26.1 — the 26.2 run deferred it to
B11-H01): LIVE iff the name appears in ≥1 block chain in the prepared
`block-behaviours.json` (1,168 blocks, 291 distinct chain names).

Result: **263/271 jar-live, 8 jar-dead; 263/271 chain-live, 0
jar-live-but-chain-dead.** The jar half and the chain half agree
exactly — every jar-live mapping is chain-live, every jar-dead mapping
is chain-dead. The chain half adds no new dead mappings, which settles
the 26.2 H01 question for 26.1 in the negative: no mapping is live in
the jar but dead in the chains.

**Group A — RENAME: 26.1 content exists under another name; live arms
dead (4):**

| Mapping | 26.1 form (jar + chain) | Dead arms | Defect |
|---|---|---|---|
| AbstractBed → `AbstractBedBlock` | `BedBlock` directly | 5 (place ×2, interact, hooks, shapes) | B11-G01 (major) |
| RedstoneWire → `RedstoneWireBlock` | `RedStoneWireBlock` | 10 (hooks ×3, interact, place, redstone ×3, shapes, survive) | B11-G02 (major) |
| Path → `PathBlock` | `DirtPathBlock` | 4 (place, shapes, survive, tick) | B11-G03 (minor) |
| Speleothem → `SpeleothemBlock` | `PointedDripstoneBlock` directly (no abstract parent) | 5 (place, shapes, survive, tick ×2) | B11-G04 (minor) |

All four 26.3 names exist in the 26.3 client jar — the table is
26.3-correct and 26.1-wrong in all four cases. The other five 1.21.7
renames do NOT carry to 26.1 — the 26.x names MATCH the 26.1 jar
(verified each, not assumed): FarmlandBlock ✓ (chain
`[FarmlandBlock, Block]`), SnowyBlock ✓ / SpreadingSnowyBlock ✓
(grass_block chain `[GrassBlock, SpreadingSnowyBlock, SnowyBlock,
Block]`), NetherFungusBlock ✓ / NetherRootsBlock ✓ (crimson_fungus
chain `[NetherFungusBlock, VegetationBlock, Block]`). LilyPad is the
correct `LilyPadBlock` (26.1 has no `WaterlilyBlock`); CopperChestBlock
✓ (chain via `WeatheringCopperChestBlock`), CopperGolemStatueBlock ✓.

**Group B — NO 26.1 CONTENT: dead but harmless (4):** StrawBedBlock
(26.3-only; 1 arm: update.rs:430), SulfurSpikeBlock (26.2-only; 1 arm:
speleothem.rs:110, the max-height-2 branch), ShelfMushroomBlock
(26.3-only; 4 arms: bonemeal.rs:94, place.rs:493, shapes.rs:137,
survive.rs:42), PotentSulfurBlock (26.2-only; 4 arms: hooks.rs:199,
pistons.rs:154, place.rs:1193, shapes.rs:819). All four classes exist
in the 26.3 jar (SulfurSpike/PotentSulfur also in 26.2's) and none has
any block in 26.1 data (0 straw_bed / sulfur_spike / shelf_mushroom /
potent_sulfur entries in block-behaviours.json). Arms unreachable.
Hygiene note, not gaps.

Also verified: horizon-prepare's survival.rs CLASS_RULES carries the
same four 26.3 names (`PathBlock`, `RedstoneWireBlock`,
`SpeleothemBlock`, `ShelfMushroomBlock`) — the prepare-side table
misses 26.1's chain names the same way (see B11-G02).

### §6.2 The 26.x tag-name pattern INVERTS for 26.1 — except the two 26.3-only tags

The 1.21.7 sweep's systemic cause — the port reads 26.x tag names for
tags that don't exist in 1.21.7 data — mostly does not apply to 26.1,
because 26.1 HAS the 26.x tags. Every tag read this batch checked is
live in 26.1's data (client-jar datapack, 775 tag files; tag-file
existence + contents):

- `minecraft:cauldron_can_remove_dye` (item tag: leather helmet /
  chestplate / leggings / boots, leather horse armor, wolf armor) — the
  Rust's wash gate (cauldron.rs:7-12, :214) is live; 1.21.7's B11-G23
  does not apply.
- `supports_chorus_plant` / `supports_chorus_flower` (both: end stone)
  — survive.rs:102/132/137 live; 1.21.7's B11-G21 does not apply.
- `overrides_mushroom_light_requirement` (mycelium, podzol, crimson +
  warped nylium) — survive.rs:266 live; `mushroom_grow_block` is the
  ABSENT one in 26.1 (the reverse of 1.21.7); 1.21.7's B11-G17b does not
  apply.
- `cannot_support_snow_layer` (ice, packed ice, barrier) /
  `support_override_snow_layer` (honey block, soul sand, mud) —
  survive.rs:295/297 live; the 1.21.11 names
  (`snow_layer_can_survive_on` / `snow_layer_cannot_survive_on`) are
  absent in 26.1 (the reverse of 1.21.7); 1.21.7's B11-G16b does not
  apply.
- `convertable_to_mud` (dirt, coarse dirt, rooted dirt) — live (item
  446's 26.1 spelling confirmed).
- `dripstone_replaceable_blocks` (`#base_stone_overworld`) — live
  (item 542's default confirmed).

Correctly-absent tags, correctly handled: `sculk_growth_inhibitors` is
absent from 26.1 data and the Rust checks sculk_sensor/sculk_shrieker
blocks explicitly (item 510, verified). The two EXCEPTIONS — tags the
port reads that 26.1 genuinely lacks — are both 26.3-only and both
produce defects: the potion tags `douses_fire` /
`hurts_water_sensitive_entities` / `extinguishes_entities` /
`rehydrates_axolotls` (26.1's `PotionTags` has ONLY `TRADEABLE`; the
26.3 datapack has all four) — B11-G05; and `speleothems` (26.2+ only)
— the second cause of B11-G04.

### §6.3 26.3-form code that 26.1's jar contradicts

The port carries 26.3 forms in six places where 26.1's jar/data says
otherwise — all six are 26.1-direct fixes, not re-base propagations.
(The 26.3 client jar, 11,383 classes, was used ONLY for these absence
checks and the §6.5 comparison — never as the spec.)

1. The four class-table mappings (§6.1 Group A): `AbstractBedBlock`,
   `RedstoneWireBlock`, `PathBlock`, `SpeleothemBlock` all exist in the
   26.3 jar and in NEITHER 26.1 jar.
2. The AnyFuel grouping (B11-G07): 26.3 resolves via
   `DataComponentLookup.findAll(COOKING_FUEL)` (javap-verified on the
   26.3 jar); 26.1 via the insertion-ordered `FuelValues`.
3. The splash-water tag gate (B11-G05): 26.3's datapack HAS the four
   potion tag files; 26.1's has only `tradeable.json`, and 26.1 vanilla
   gates on `Potions.WATER` equality.

The Group B mappings (StrawBed, SulfurSpike, ShelfMushroom,
PotentSulfur) are 26.2/26.3-only content carried harmlessly. The Mood
spacing is NOT 26.3-form — 26.1's own literal is one-spaced (see §6.6
G20), and the Rust matches it; the two-space form belongs to 26.2/26.3.

### §6.4 Re-base WARNING: 26.1 is 26.x where 1.21.7 is not — do not propagate 1.21.7 forms down

This batch found ELEVEN behaviours where 26.1's vanilla form differs
from 1.21.7's and the Rust already matches 26.1 — a re-base must not
propagate the 1.21.7 forms to 26.1: Riptide passenger check (26.1 HAS
`isPassenger` in `releaseUsing`, offsets 61-66), SwellGoal alive check
(26.1 HAS `isDeadOrDying` in `canUse` + `tick`), polar-bear fox-goal
age gate (26.1's selector is the `!isBaby` lambda, bootstrap #2),
lead-knot discard on failed tie (26.1 discards the fresh knot, offsets
139-152), mushroom bonemeal gate (26.1 requires the huge-mushroom
feature + `4 + foliageRadius` height), sapling bonemeal height check
(26.1: `TreeGrower.getMinimumHeight` + build height, no `canGrow`),
golem stroll roll order (26.1 draws 0.3 first, 0.7 second — the Rust's
order; 1.21.7 stores the first roll and compares the second to 0.3),
social gate (26.1's chat check feeds ONLY the report button; hide/show
are ungated), trader heightmap + storage (26.1 registers
WANDERING_TRADER with MOTION_BLOCKING_NO_LEAVES and stores SavedData
`wandering_trader.dat`, fields `spawn_delay` / `spawn_chance`),
shipwreck `heightAdjusted` caching (26.1 HAS the field), player-data
layout (26.1 IS `players/data/`). The 1e-5 fluid-current floor: present
in vanilla 26.1 (the constant 9.999999747378752E-6 appears in both
float and double form in 26.1's `Entity` constant pool) AND in the Rust
(entity_fluid.rs:87, `length_squared() < 1.0e-5`). No floor-removal to
propagate; the 1.21.7 §6.4 warning (the template removed the floor for
1.21.11, but 1.21.7 HAS it) stands as a general per-version caution.
Two 26.1-vs-26.2 cautions also belong here: 26.1's F3 Mood literal is
ONE-spaced while 26.2's is TWO-spaced (do not propagate 26.2's form
down), and 26.1's sound variants cover cows/pigs/chickens/cats (do not
propagate 1.21.7's A02 note — see §5).

### §6.5 Class-absence headline: 69 items name 26.2-only or 26.3-only classes (57 tokens)

69/214 items name ≥1 class that exists in the 26.2 jars (7,445 +
10,952 classes) or the 26.3 client jar (11,383) but in NEITHER 26.1
jar. 57 distinct tokens, split by the 26.2 jar index into:

- **36 tokens that are 26.2-only** (present in 26.2, absent in 26.1):
  the holder move (EntityTypes, BlockEntityTypes), the friends system
  (PresenceSharing, PresenceHandler, RemoteFriendListUpdateHandler),
  the render rework (GpuFormat, PrimitiveTopology, BlendEquation,
  BlendFactor, BlendOp, StagedVertexBuffer, StagingBuffer,
  SectionTaskDynamicQueue, FeatureRendererType,
  RenderTypeFeatureRenderer, GizmoFeatureRenderer), the GUI moves
  (MenuTabBar, Hud, LocatorBar, GridPos), the command/selector rework
  (TeamColorArgument, ColorCollection, SetOnceOptionState,
  InvertableSetOptionState, GameTestEntityBuilder, GameTestMobBuilder,
  SetDifficulty), the content additions (SulfurCube, PotentSulfurState,
  SpeleothemUtils, WeatheringCopperCollection, AbstractCubeMob,
  MultiplayerScope, UnpublishCommand, LevelExtractor) — plus `Submit`
  (a nested-class fragment) and `MultiplayerOptionsScreen` (26.2-only,
  confirmed in the 26.2 jar index though absent from all three jars
  first indexed). 50 items name ≥1 of these.
- **21 tokens that are 26.3-only** (absent from 26.1 AND 26.2), of
  which 2 are false positives (`Removed` — an English word matching a
  26.3 simple name; `Step` — a fragment of
  `SteppedInterpolationHandler$Step`), leaving 19 real 26.3-only
  classes: the removed systems (CommandResponseTracker,
  ComputeCommand, ContextFloatProvider/IntProvider, SlotSourceArgument,
  SpawnTargetPoint, SteppedInterpolationTracker, NarrationTrigger,
  ArgProvider, LootContextSources, ItemActivation, EmissiveRisingParticle,
  OitStage, TrimOverride, InviteCode, RealmTierConfigurationDto,
  BoxBlockMatcher, Axes), plus the rename `RedstoneWireBlock` (26.3's
  name; 26.1 has RedStoneWireBlock). 21 items name ≥1 of these.

Each absent class was triaged individually: pass-by-removal where the
Rust deleted the surface (items 507, 508, 511, 566, 587, 600, 602, 619,
625, 675, 696, 714), pass-by-absence where the Rust never had the
surface (items 510, 512, 513, 522, 525, 528, 538, 542, 543, 557, 562,
567, 570, 572, 580, 616, 618, 622, 627, 630, 636, 639, 644, 650, 654,
663, 664, 665, 672, 680, 686, 688, 691, 694, 702, 710, 711, 715). A
further ~109 classlike tokens are absent from ALL FOUR jar indexes but
are Rust-side names (ArgType, ResKey, Opt, Wind, BlockRot),
JDK/Brigadier/fastutil classes (ArrayList, AtomicReference,
Long2ObjectMap, Optional, Integer), constants (BANNER, DROWNED, POWER,
RESET), or generic words — 0 real vanilla absences beyond the
26.2-only and 26.3-only sets.

### §6.6 1.21.7 Batch 11 defects re-checked against 26.1

Every 1.21.7 Batch 11 defect was re-checked against 26.1's jar/data/Rust.
Seven carry over (plus one cosmetic detail from P01); the rest are
withdrawn with the deciding evidence.

**APPLIES (minted as 26.1 items):**

- G01 beds — APPLIES (B11-G01 above). Major. 26.1 HAS a bed BE type
  and `BedBlock` — but the arms dispatch on the dead `AbstractBed`.
- G02 redstone wire — APPLIES (B11-G02 above). Major. 26.1 has
  `RedStoneWireBlock`; the table has only the 26.3 name, no dual.
- G03 farmland/paths — APPLIES in part (B11-G03 above): paths dead
  (`DirtPathBlock` vs `PathBlock`); farmland live.
- G06 speleothem — APPLIES (B11-G04 above). 26.1 has neither the
  class nor the tag.
- G12 number providers — APPLIES (B11-G06 above), datapack-only.
- G13 splash water — APPLIES (B11-G05 above). Major. The Rust's tag
  gate is dead in 26.1 (only `tradeable` exists).
- G15 fuel order — APPLIES (B11-G07 above).
- P01 F3 chart footer — SETTLED BY THE JAR, with one cosmetic delta
  (B11-G08 above): layout matches; only the `fps` casing differs.

**Withdrawn (verified not-applicable, with the deciding evidence):**

- G04 snowy — SnowyBlock/SpreadingSnowyBlock live in the 26.1 jar +
  chains (§6.1).
- G05 fungus/roots — NetherFungusBlock/NetherRootsBlock live (§6.1).
- G07 sapling bonemeal — 26.1's gate IS the height check
  (`getMinimumHeight` + build height); Rust matches (bonemeal.rs:304-315).
- G08 riptide riding — 26.1 vanilla HAS the isPassenger check
  (releaseUsing offsets 61-66); Rust blocks while riding
  (trident.rs:501).
- G09 lead knot — 26.1 vanilla discards the fresh knot on a failed
  tie (offsets 139-152); Rust creates the knot only if attached
  (leads.rs:640-653). Same observable outcome.
- G10 creeper defuse — 26.1 vanilla HAS isDeadOrDying in SwellGoal
  canUse + tick; Rust gates on t.alive (hostile.rs:294, :312).
- G11 polar-bear cubs — 26.1's fox-goal selector IS !isBaby
  (bootstrap #2 → lambda$registerGoals$1); Rust matches
  (polar_bear.rs:51-56).
- G14 player-data layout — 26.1 vanilla IS players/data/
  (LevelResource.PLAYER_DATA_DIR = "players/data"); Rust matches.
- G16 golem stroll + snow tags — stroll roll order matches 26.1
  exactly (0.3 first, 0.7 second, javap-verified); snow tags are the
  26.x names, present (§6.2).
- G17 mushroom — 26.1's gate genuinely requires the feature + height
  (`4 + foliageRadius`); Rust matches; light tag present (§6.2).
- G18 social gate — 26.1's chat check feeds ONLY the report button
  (ctor offsets 100-123, 157-214); hide/show are ungated; Rust matches
  (no chat gate on hide/show).
- G19 trader — heightmap MOTION_BLOCKING_NO_LEAVES matches; storage is
  SavedData wandering_trader.dat, matches (§6.4).
- G20 F3 lines — ALL THREE thirds match 26.1: Local Difficulty has no
  Day suffix (`"Local Difficulty: %.2f // %.2f"`, day count is the
  separate `DebugEntryDayCount` entry `"Day #"`, which the Rust renders
  as `"Day #{}"`); memory lines use MiB (`"Mem: %2d%% %03d/%03dMiB"`);
  and the Mood literal is ONE-spaced in 26.1 (`' (Mood %d%%)'`,
  constant-pool verified) — the Rust's one-space form matches 26.1
  exactly. (26.2's literal is two-spaced; the one-space form is a
  26.1-form match, not a defect here.)
- G21 chorus — tags present in 26.1 data (§6.2).
- G22 dialog tooltip — 26.1's ItemDisplayWidget uses isHovered() only
  (offsets 97-107); Rust matches (hover-only).
- G23 cauldron wash — tag present with leather armour (§6.2); Rust
  gate reads it (cauldron.rs:7-12, :214).
- H01 shipwreck — 26.1 HAS heightAdjusted; Rust caching matches.
- H02 cauldron table — premise withdrawn with G23 (see §3); the two
  cauldron oracles are recorded as B11-O01/O02 (§2).
- A01 trim inline form — CARRIES as an accepted note (B11-A01, §5).
- A02 sound_variant — NEVER EXISTED for 26.1 (see §5): 26.1 vanilla
  has sound variants for cows/pigs/chickens/cats.
- A03 special-foil decal — CARRIES as an accepted note (B11-A02, §5).
- A04 separator + per-player resolution — CARRIES as an accepted note
  (B11-A03, §5).

## §7 Method notes

- **Item source:** 26.1's own rank-3 ledger,
  `versions/26.1/rank3-progress.json` (214 items in the `closed` list;
  counts verified: no-change 179 / ported 35; triage string A1 60 / A2
  67 / B1 56 / B2 31 = 214). NOTE: 145 of the 214 items have NO `title`
  field (only `id`, `verdict`, `evidence`) — the §8 table identifies
  those by id + evidence. The retriage behind it
  (`~/workspace/mcir-work/port-26.1-triage/retriage-rank345.json`) was
  available but not needed: triage here rests on the per-item evidence
  + jar/Rust verification. `versions/26.1/work-items.json` (the full
  ledger) was consulted for context only.
- **Jar evidence:** named 26.1 server-game jar
  (`26.1.x/versions/26.1/jars/server-game.jar`, 7,351 classes) + 26.1
  client jar (10,682 classes), both Mojang-mapped. The 26.1 server jar
  in leakage-audit is a Mojang BUNDLER jar and was never javap'd. The
  26.2 jars (`26.1-adapters/versions/26.2/jars/`, server-game 7,445 +
  client 10,952) were indexed ONLY for the §6.5 26.2-only/26.3-only
  split and the §6.2/§6.3 comparisons. The 26.3 client jar
  (`jars-26.3/client-26.3.jar`, 11,383 classes) was used ONLY for the
  §6.5 absence comparison and the §6.3 26.3-form checks — never as the
  spec. javap via JDK 25 (`~/workspace/tools/jdk-25/bin/javap`); each
  call took 4–18s, batched 1–4 classes per exec. Class-existence checks
  used zipfile indexes of the jars (in scratch:
  `server26.1-classes.json`, `client26.1-classes.json`,
  `server26.2-classes.json`, `client26.2-classes.json`,
  `client26.3-classes.json`), not per-class javap. Two string literals
  (Mood spacing, Day #) were verified by parsing the class files'
  constant pools directly, since javap's `// String` rendering makes
  leading-space counts ambiguous.
- **Crosscheck:** automated class cross-check (`rank3-crosscheck-raw.json`
  + `class-absence-split.json` in scratch): every named vanilla class
  checked against both 26.1 jars, both 26.2 jars, and the 26.3 jar (see
  §6.5); every named Rust module checked against the 26.1 tree. 1 named
  module does not exist: 511's `horizon-client-render/src/social.rs`
  was DELETED by its port (expected). 62 named module refs verified
  present.
- **Family-pass discipline:** a pass rests on jar evidence (named
  classes exist in the 26.1 jar, or are correctly identified as
  26.2/26.3-only) + the Rust file existing with the same logic.
  Sample-verified across patterns: all 35 "ported" verdicts were
  individually re-verified in the current tree (greps + line numbers in
  §8), plus spot checks on no-change items with specific jar claims
  (589 shears tool rules in prepared items.json, 595 TickThrottler
  (20,200), 607 context args, 623 SoulParticle, 635 cauldron set, 646
  ResourceArgument, 655 ArgProvider, 663 GpuFormat, 665 waypoint
  system, 672 shouldInvertValue, 674 NarrationTrigger, 680
  BoxBlockMatcher, 686 Axes, 688 OitStage, 691 stepped tracker, 694
  PotentSulfur, 698 Gui ContextualInfo, 699 Allay vibration user, 702
  UnpublishCommand, 704 WoodType POPLAR, 707 BiomeTags, 710
  SpawnTargetPoint, 711 stepped handler). The 1.21.7-defect
  cross-checks (§6.6) are the deep-verification layer: 23 defects +
  H01/H02/P01 + 4 accepted notes re-checked with javap against 26.1's
  jar.
- **Class-table sweep:** `class-table-sweep.json` in scratch (271
  mappings × jar-liveness × chain-liveness). Chains from the prepared
  `block-behaviours.json` (1,168 blocks). The sweep is the first
  chain-liveness run in the programme (26.2 deferred it) and it
  agrees exactly with the jar half.
- **Data checks:** tag files + contents from the 26.1 client-jar
  datapack (775 tag files) and the 26.2/26.3 jars for comparison;
  block/item registries from the 26.1 datagen reports
  (`26.1.x/versions/26.1/datagen/reports/`); loot-table provider usage
  counted over all 1,326 vanilla loot tables + all datapack JSON;
  `en_us.json` VERIFIED PRESENT in the prepared tree
  (`.local/horizon-client/26.1/assets/minecraft/lang/en_us.json`);
  `sounds.json` confirmed ABSENT (genuinely — needs the separate
  `horizon-prepare sounds` step against a launcher asset store the VM
  lacks; not a port defect).
- **Oracle triage:** from the Batch 0 measured table ONLY
  (`docs/sweep/26.1/batch-0-oracles.json`, 199/391 compile); the two
  in-scope cauldron oracles were triaged with the missing symbol
  javap-verified (§2). No oracle was repaired (Batch 0 scope decision).
- **No cargo builds, no driver runs, no git commits.** Static checks
  only (javap + python + grep), per the batch constraints. No file
  under `versions/26.1/` was modified; no git command was run in the
  worktree.
- **Re-verification caveat:** all Rust greps ran against the sweep
  worktree (`~/workspace/mcir-work/sweep-26.1/versions/26.1/`, branch
  parity-sweep-26.1); line numbers are from that tree.

## §8 Per-item triage table

Verdicts: **pass** (verified against the 26.1 jar/data + Rust). No item
verdicts gap/harness/probe/accepted-gap in this batch; the eight
defects are cross-cutting (§1) and the three accepted notes are
1.21.7-sweep carries (§5), not ledger items. "Family" passes rest on:
the port found NO behaviour difference between 26.2 and 26.1 for the
item (pure renames/re-shapes/codecs/bytecode artifacts); the named
classes exist in the 26.1 jar (crosscheck) or are correctly
26.2/26.3-only; the Rust file exists with the same logic.
Sample-verified across patterns (see §7). Items whose ledger entry has
no title are identified by id + evidence (145 of 214 lack a `title`
field in rank3-progress.json).

| Item | Verdict | Evidence |
|---:|---|---|
| 506 | pass | Family: MinecraftServerSettingsServiceImpl: 26.2 holds a NotificationManager and derives the server via a private server() helper (with requireNonNull); 26.1 ... |
| 507 | pass | Ported, verified: headers() sends only Is-Prerelease (realms/client.rs:1069-1081); test asserts X-NetworkProtocolVersion absent (:1721); no InviteCode/RealmTierConfiguration refs. |
| 508 | pass | Ported, verified: ArgType::SlotSource gone (0 refs; only a RangeSlotSource comment, codec.rs:197); slot_id in args.rs:251; /item tree is ResKey. |
| 509 | pass | Ported, verified: Std140 vec3/ivec3 -> (16,16) (post_chain.rs:64); Globals 56-byte layout (:517-533). |
| 510 | pass | Ported, verified: sculk_can_place_growth counts sensor+shrieker explicitly, false at 3 (level/sculk.rs:607-636); sculk_growth_inhibitors absent from 26.1 data. |
| 511 | pass | Ported, verified: social.rs deleted; 0 PresenceSharing/SharePresence refs; friends machinery removed; game/social.rs + game/friends.rs untouched. |
| 512 | pass | Ported, verified: test_runs.rs:212 '26.1 has no dimension on a test'; tests run in the level in place. |
| 513 | pass | Family: EntityType->EntityTypes renames only (26.1 keeps EntityType registration); fortress.rs 0-diff. |
| 514 | pass | Family: StatsScreen differs only by TabNavigationBar -> MenuTabBar (26.2's new tab-bar class); TabManager.setCurrentTab's 2-arg form delegates to the 3-arg ... |
| 515 | pass | Family: BlockRotProcessor.processBlock bodies identical (same random/integrity/HolderSet logic); the differences are API shape (26.1 extends ... |
| 516 | pass | Ported, verified: close_advancements_to_game added (advancements_screen.rs:183) and wired to KeyL (play_keys.rs:189). |
| 517 | pass | Family: GuiItemAtlas/PictureInPictureRenderer differ only by render-submission plumbing (26.1 takes SubmitNodeCollector + BufferSource in; 26.2 creates a ... |
| 518 | pass | Ported, verified: lan_screen.rs has the two 26.1 screens (WorldOptions + ShareToLan); success key commands.publish.started (extra.rs:996). |
| 519 | pass | Family: Batch 72 NO-CHANGE (SystemToast): 26.2 refactored the toast internals (title Component -> titleLines list, TEXT_X_START const, mutable width). ... |
| 520 | pass | Ported, verified: initial_spawn.rs:298 takes the suggestion-only path when game_mode == 2 OR !world.shape.sky_light. |
| 521 | pass | Family: GuiRenderer 832-line diff is the render-submission rework (StagedVertexBuffer); pack_text.rs 0-diff |
| 522 | pass | Family: EntityType->EntityTypes + getCenter->atCenterOf renames only; respawn_anchor.rs 0-diff. |
| 523 | pass | Family: All 7 codec/loot classes structurally identical 26.1/26.2; the codec.rs diff is earlier 26.1 work. |
| 524 | pass | Family: Batch 72 NO-CHANGE (SectionRenderDispatcher): 26.2 refactored the section-compile queue (CompileTaskDynamicQueue -> SectionTaskDynamicQueue, added ... |
| 525 | pass | Family: EntityType->EntityTypes renames only (CHEST_MINECART, CAVE_SPIDER); mineshaft.rs 0-diff. |
| 526 | pass | Family: Batch 72 NO-CHANGE (ItemFeatureRenderer): 26.2 refactored it into the RenderTypeFeatureRenderer/FeatureRendererType framework (Submit inner class + ... |
| 527 | pass | Ported, verified: screen_effects.rs:82-84 docs the spinning effect on GameRenderer (26.3 moved it to LocalPlayer). |
| 528 | pass | Family: package moves only (criterion->triggers, BlockEntityType->BlockEntityTypes); brush.rs 0-diff. |
| 529 | pass | Ported, verified: place_netherrack_or_magma unconditional (ruined_portal.rs:293); checked wrapper netherrack_or_magma (:306) for the spread site. |
| 530 | pass | Family: CommandEncoder GPU-backend rework (backend removed, render-pass checks); render internals, submit.rs 0-diff. |
| 531 | pass | Ported, verified: level_particles.rs has 3003 wax-on (:23-32) + 2002/2007 splash sound (:363-424); no 2019/2020 (:347); 2015 kept as the internal dragon-egg channel. |
| 532 | pass | Ported, verified: rpc_discover assertion is 85 (management.rs:716); heartbeat supervisor stands (statusHeartbeat abstract in both jars). |
| 533 | pass | Ported, verified: give/clear/enchant branch on target count (effects.rs:92/199/240). |
| 534 | pass | Family: RenderSection 300-line diff is the section-compile refactor (RebuildTask->CompileTask); native_material.rs 0-diff |
| 535 | pass | Family: JoinMultiplayerScreen: the Minecraft.setScreen -> Gui.setScreen move (7 sites) + one reorder; LoadingDotsWidget: getY()+getHeight() -> getBottom(); ... |
| 536 | pass | Ported, verified: /place feature is ResKey(configured_feature) (tree_world.rs:65); inline values noted 26.3-only. |
| 537 | pass | Family: MinecraftBanListServiceImpl + MinecraftGameRuleServiceImpl: 26.2 derives the server from a NotificationManager via a private server() helper; 26.1 ... |
| 538 | pass | Ported, verified: no team-color reset literal (tree_scoreboard.rs:173); network.rs maps minecraft:color (:217) with team_color kept as an alias (:214). |
| 539 | pass | Family: package moves only; beehives.rs 0-diff. |
| 540 | pass | Ported, verified: transient_blocks.rs carries the dead-feeder comments + #[allow(dead_code)] (:14-17); no add-transient-block packet in either 26.1 jar. |
| 541 | pass | Family: FillLayerFeature + SpikeFeature 0-diff; RandomizableContainerBlockEntity is the getCenter->atCenterOf rename |
| 542 | pass | Ported, verified: Wind has no max_offset, raw floored offset (dripstone.rs:537-556); replaceable_blocks optional (:603); column_radius (1,60) (:621). |
| 543 | pass | Ported, verified: cmd_publish uses commands.publish.started (extra.rs:993-996); /unpublish removed (0 refs in commands/tree). |
| 544 | pass | Family: CompiledSectionMesh: VertexFormat$IndexType -> IndexType type move only. CompileTask: the section-compile queue refactor (same family as item 524's ... |
| 545 | pass | Family: DeOpCommands: lambda capture Collection -> NameAndId, logic unchanged. BanListEntry: 26.2 added hasExpired() + hashCode(); the Rust models no ... |
| 546 | pass | Family: GhastMoveControl: only the ghast->mob field rename. HappyGhastLookControl: 26.1's local wrapDegrees90 (mod 90, clamp to +-45) moved to ... |
| 547 | pass | Family: IglooPieces + ShipwreckPieces 0-diff; IglooPiece is a branch-offset/local-store reshuffle |
| 548 | pass | Family: DataCommands 15 lines: branch inversion + local renumbering, same logic; command_storage.rs 0-diff |
| 549 | pass | Family: CarvedPumpkinBlock: EntityType->EntityTypes, CriteriaTriggers criterion->triggers, Blocks.COPPER_BLOCK read via the WeatheringCopperCollection in ... |
| 550 | pass | Family: AbstractTextAreaWidget + MultiLineEditBox + MultilineTextField all 0-diff; multiline_edit.rs 0-diff |
| 551 | pass | Family: NbtContents + TitleCommand both 0-diff; text_commands.rs 0-diff |
| 552 | pass | Family: branch inversion + Gui.getChat/Hud + setScreen moves; social.rs 0-diff. |
| 553 | pass | Family: LightCoordsUtil.smoothBlend: compiler reorder only (same sky/block masks 0xFF0000/0xFF, same zero-sample/borrow-centre-sky logic). 26.2 added a ... |
| 554 | pass | Family: DataComponentMatchers pure package move criterion->predicates; bodies identical. |
| 555 | pass | Family: OreFeature 0-diff; ore_feature.rs 0-diff |
| 556 | pass | Family: MonsterRoomFeature + BeehiveDecorator are EntityType/BlockEntityType renames only (same spawner table); feature_world.rs diff is the 26.2 ... |
| 557 | pass | Family: ClientChunkCache 91-line diff is the LevelExtractor move + empty-sections rework; remote_chunks.rs fixed in batch 84 |
| 558 | pass | Ported, verified: test_instances.rs has no dimension field. |
| 559 | pass | Ported, verified: growth_tests.rs:233 lone_dark_oak_sapling_takes_bone_meal_but_grows_no_tree; no poplar refs. |
| 560 | pass | Family: InWorldGameRulesScreen: the Minecraft.setScreen -> Gui.setScreen move (3 sites) + Minecraft.screen -> Gui.screen(). Menu chrome. game_rules_screen.rs ... |
| 561 | pass | Family: OverworldBiomeBuilder 62-line diff is only the Spline Coordinate ctor API shape + offsets, no parameter changes; settings.rs doc fixed 26.2->26.1 |
| 562 | pass | Family: BlendFunction refactored to BlendEquation + BlendFactor/BlendOp in 26.2; GPU plumbing. |
| 563 | pass | Family: leafComponent() is a 26.2 addition the Rust does not model; pad_menu diff is batch-73 LAN work |
| 564 | pass | Family: CuredZombieVillagerTrigger pure package moves (criterion->triggers, EntityPredicate->predicates/entity). |
| 565 | pass | Family: OceanRuinPiece is branch offsets + the DROWNED EntityType rename; ocean_ruin.rs 0-diff |
| 566 | pass | Ported, verified: scoreboard set/enable branch on target count (scoreboard_cmd.rs:532-541, 636-640); 0 CommandResponseTracker refs. |
| 567 | pass | Family: CubeMap: getGameRenderState->gameRenderState, getMainRenderTarget->GameRenderer.mainRenderTarget, VertexFormat$Mode->PrimitiveTopology. ... |
| 568 | pass | Family: ScissorStack: 26.2 made push/pop void and falls back to a screenSize rect on an empty stack (26.1 pushes the rect unclipped); the intersect math is ... |
| 569 | pass | Family: GameModeSwitcherScreen: the Minecraft.setScreen -> Gui.setScreen move (2 sites). Menu chrome. gamemode_switcher.rs 0-diff. |
| 570 | pass | Family: EntityType->EntityTypes only (5 sites); wandering_trader.rs 0-diff. |
| 571 | pass | Family: Batch 63 NO-CHANGE (MatchTool): the only difference is ItemPredicate's package move (advancements.predicates -> advancements.criterion) in 26.1's ... |
| 572 | pass | Family: NautilusAi: EntityType->EntityTypes only. ChargeAttack: 26.2's dealKnockBack threads a mobAttack DamageSource + ATTACK_DAMAGE through the 6-arg ... |
| 573 | pass | Family: TestBlock getSignal->ownSignal extraction (same as 634); test_block.rs 0-diff. |
| 574 | pass | Ported, verified: chart gridlines uppercase '30 FPS'/'60 FPS' (debug_chart.rs:372-373). |
| 575 | pass | Family: AbstractStringWidget handleStyleClick extraction + DialogScreen setScreen move; dialogs.rs 0-diff. |
| 576 | pass | Ported, verified: hand.rs swing sends the packet unconditionally (:199-202); no spectator-crosshair portal branch (:689-691, :720). |
| 577 | pass | Family: QuickPlay setScreen move only; launch_options.rs 0-diff. |
| 578 | pass | Family: InventoryChangeTrigger Slots pure package move; matches() identical. |
| 579 | pass | Family: BlockEntityType->BlockEntityTypes rename only; creaking.rs 0-diff. |
| 580 | pass | Family: Screenshot GPU plumbing (TextureFormat->GpuFormat) + 26.2-only grab(Minecraft,boolean) panoramix path; Rust only manages the screenshots folder. |
| 581 | pass | Family: EntityType->EntityTypes + CriteriaTriggers move only; block_pattern.rs 0-diff. |
| 582 | pass | Family: PumpkinBlock 0-diff; pumpkin.rs 0-diff |
| 583 | pass | Ported, verified: PlayerMenuItem.isEnabled constant true (spectator_menu.rs:24-25); enabled: true at both construction sites (:155, spectator.rs:35). |
| 584 | pass | Ported, verified: bed click redirects foot->head (interact.rs:365-381); dragon-egg teleport has the world-border check (:598). NOTE: the bed arms dispatch on the dead Class::AbstractBed - see B11-G01. |
| 585 | pass | Family: NetherFossilPieces + NetherFossilStructure both 0-diff despite the differing tag; nether_fossil.rs 0-diff |
| 586 | pass | Family: Batch 63 NO-CHANGE (ClientChunkCache$Storage): 26.2 refactored empty-section tracking from 26.1's single loadedEmptySections set to double-buffered ... |
| 587 | pass | Ported, verified: trims.rs is the atlas palette-swap form (trim_palette.png, :3-6); 0 TrimOverride refs. |
| 588 | pass | Family: EntityType->EntityTypes only; cat_spawner.rs 0-diff. |
| 589 | pass | Shears rules verified in prepared items.json: minecraft:tool rules cobweb 15.0, #leaves 15.0, #wool 5.0, vine+glow_lichen 2.0 - data-driven, no code port. |
| 590 | pass | Ported, verified: TradeWithPlayerGoal gates on !hurt_marked (trader.rs:200); Mob.hurt_marked field (mobs.rs:1548, set :3925). |
| 591 | pass | Family: ExperimentsScreen I18n.exists->Language.has + setScreen move; experiments.rs 0-diff. |
| 592 | pass | Family: EntityType->EntityTypes only; monument.rs 0-diff. |
| 593 | pass | Family: EntityType->EntityTypes only; furnace.rs diff is the earlier 26.1 fuel work. |
| 594 | pass | Family: MinecartCommandBlock$MinecartCommandBase 0-diff; command_minecarts.rs 0-diff |
| 595 | pass | TickThrottler(20, 200) hardcoded (dedicated.rs:1277; lan_actions.rs:63); 26.2's threshold<=0 guard is unreachable in 26.1. |
| 596 | pass | Family: EntityType->EntityTypes only; stats.rs 0-diff. |
| 597 | pass | Family: RamTarget knockback API rework (3-arg -> 6-arg with mobAttack source + attack-damage float); strength/x/z computation identical, physics identical — ... |
| 598 | pass | Family: PandaRenderer 0-diff; panda.rs 0-diff |
| 599 | pass | Family: Phantom$PhantomSweepAttackGoal 0-diff despite the differing tag; phantom.rs 0-diff |
| 600 | pass | Ported, verified: 0 NarrationTrigger refs; screen_narrator.rs carried (updateNarratorStatus form). |
| 601 | pass | Family: NowPlayingToast 0-diff; toasts.rs 0-diff |
| 602 | pass | Ported, verified: ARGUMENT_TYPES is 57 entries (commands.rs:26-84) with minecraft:color, no 26.3-only entries; matches the 26.1 datagen registry count (57). |
| 603 | pass | Family: SolidBucketItem 0-diff; buckets.rs 0-diff |
| 604 | pass | Family: SideChainPartBlock 0-diff; shelf.rs 0-diff |
| 605 | pass | Family: IcebergFeature structurally identical; iceberg.rs 0-diff. |
| 606 | pass | Family: TrunkPlacer structurally identical; trunk.rs 0-diff. |
| 607 | pass | ContextFloat/Int/Feature inner args in neither jar (26.3-only); item 602 removed their registry entries. |
| 608 | pass | Family: Batch 63 NO-CHANGE (SpriteContents animated textures): createAnimationState/drawToAtlas differ only in GPU upload plumbing (per-frame texture ... |
| 609 | pass | Family: RangedMatcher pure criterion->predicates package move (14 diff lines, all package names); state_ranges.rs 0-diff |
| 610 | pass | Family: Batch 63 NO-CHANGE (jigsaw/structure/test edit screens): onCancel/onDone differ only by the screen-setter moving from Minecraft.setScreen (26.1) to ... |
| 611 | pass | Family: MultifaceSpreadeableBlock + WaterloggedVegetationPatchFeature both 0-diff |
| 612 | pass | Family: Batch 63 NO-CHANGE (CrashReport/ServerWatchdog/BlockableEventLoop): CrashReport.getDetails is the same sections in a restructured order (same '-- ... |
| 613 | pass | Family: ContentsSlotSource 0-diff; contents.rs docs re-anchored from 26.3-only classes to ContainerComponentManipulator |
| 614 | pass | Family: Batch 63 NO-CHANGE (AbstractFurnaceMenu): fillCraftSlotsStackedContents is logically identical (26.2 only stores the container in a local first). The ... |
| 615 | pass | Family: Batch 63 NO-CHANGE (SectionMesh$SectionDraw): pure type move — blaze3d.IndexType (26.2) vs blaze3d.vertex.VertexFormat$IndexType (26.1) — same logic, ... |
| 616 | pass | Family: Batch 63 NO-CHANGE (LeashFeatureRenderer): 26.2 refactored the render methods (buildGroup/prepare -> renderLeash/renderSolid) and removed the Submit ... |
| 617 | pass | Family: Batch 64 NO-CHANGE (RandomCommand): the only difference is MinMaxBounds' package move (advancements.predicates -> advancements.criterion) in ... |
| 618 | pass | Family: Batch 64 NO-CHANGE (EntitySelectorOptions): the registered option-name sets are identical (26 names each, no diff). The +22/-21/~29 method churn is ... |
| 619 | pass | Ported, verified: 0 teams_allow_damage refs in cube.rs; Slime.dealDamage has no team check in either jar. |
| 620 | pass | Family: Illusioner + IllusionerRenderer both 0-diff |
| 621 | pass | Family: ExperimentalRedstoneWireEvaluator 0-diff |
| 622 | pass | Family: Batch 64 NO-CHANGE (TabNavigationBar): 26.2 refactored the Builder (added x/y/height/tabButtons fields, 5-arg constructor, addTabs varargs replacing ... |
| 623 | pass | SoulParticle in the 26.1 jar; EmissiveRisingParticle is 26.3-only (cross-check). |
| 624 | pass | Family: Batch 64 NO-CHANGE (RepeatedNarrator$Params): 26.2 record-ified the params class (751 -> 1939 bytes, record methods added). Client-only Realms ... |
| 625 | pass | Ported, verified: 0 compute refs in tree_data.rs; no cmd_compute; ComputeCommand absent from both 26.1 jars. |
| 626 | pass | Family: SnifferRenderer 0-diff |
| 627 | pass | Family: Batch 64 NO-CHANGE (ShadowFeatureRenderer): the 26.2 render-feature refactor (buildGroup/prepare -> renderTranslucent, Submit inner class moved to ... |
| 628 | pass | Family: Batch 64 NO-CHANGE (FlatLayerInfo): 26.2 stores the block as a Holder<Block> where 26.1 stores a plain Block; getBlockState() (default state) and ... |
| 629 | pass | Family: ScreenNarrationCollector + 4 inners all 0-diff; narrator lib.rs 0-diff |
| 630 | pass | Family: Batch 64 NO-CHANGE (WoodlandMansionPieces): the large diffs are 26.2 refactors only — Tuple<Integer,Integer> grid positions replaced by a GridPos ... |
| 631 | pass | Family: Batch 64 NO-CHANGE (JigsawBlock.useWithoutItem): 26.2 only reorders the body (casts the block entity to a local before the canUseGameMasterBlocks ... |
| 632 | pass | Family: FilePackResources + SharedZipFileAccess + FileResourcesSupplier all 0-diff; zip.rs 0-diff |
| 633 | pass | Family: Batch 64 NO-CHANGE (dialog screens): WaitingForResponseScreen.onClose differs only by Minecraft.setScreen -> Minecraft.gui.setScreen (same as item ... |
| 634 | pass | Family: Batch 64 NO-CHANGE (RedstoneTorchBlock): 26.2 extracted getSignal's body into an ownSignal helper; the logic is identical (lit && side != UP ? 15 : ... |
| 635 | pass | CauldronInteractions register the same 16+16 set in both jars; Rust wash gate reads the live cauldron_can_remove_dye tag (see G23). |
| 636 | pass | Family: WindCharge/BreezeWindCharge constructors identical except EntityType -> EntityTypes rename; noDeflectTicks = 5 in both. |
| 637 | pass | Family: handleCodeOfConduct differs only by minecraft.screen field vs minecraft.gui.screen() accessor; same CodeOfConductScreen flow. code_of_conduct.rs ... |
| 638 | pass | Family: ReloadableResourceManager 0-diff; packs.rs 0-diff |
| 639 | pass | Family: emitGizmos: 26.2 adds a LevelExtractor.isEntityVisible check + extracts GizmoFeatureRenderer (absent in 26.1); debug-render plumbing, same hitbox ... |
| 640 | pass | Family: TeamSelectionItem.extractIcon: 26.1 reads ChatFormatting.getColor(), 26.2 the TeamColor form (same 16 colors, same RGB extraction); spectator-menu ... |
| 641 | pass | Family: WorldTab: 26.2 extracted createTypeButtonNarration from the inline CycleButton builder; same map-type button + narration. PresetEditor lambda differs ... |
| 642 | pass | Family: BaseCoralPlantTypeBlock + StairBlock both 0-diff; shapes.rs 0-diff |
| 643 | pass | Family: Mirror.mirror(int,int) identical arithmetic (26.2 uses a single return); SlabBlock.isPathfindable identical (LAND false, WATER fluid-is-water, AIR ... |
| 644 | pass | Family: DesertPyramidPiece: 26.2 reads terracotta via Blocks.DYED_TERRACOTTA ColorCollection (35 refs) where 26.1 uses ORANGE_TERRACOTTA (34) + ... |
| 645 | pass | Family: ProjectileDeflection 0-diff |
| 646 | pass | ResourceArgument in the 26.1 jar; registries.rs diff is earlier inline_feature work. |
| 647 | pass | Family: LightningBoltRenderer 0-diff; lightning.rs 0-diff |
| 648 | pass | Family: PackEntry extractContent/handlePackSelection restructured (26.2 checks parent.getSelected() where 26.1 checks the touchscreen option); pack-screen ... |
| 649 | pass | Family: SerializedOptions: only the canonical constructor's access modifier changed (package-private 26.1 -> private 26.2); same record fields and CODEC. ... |
| 650 | pass | Family: EntityType->EntityTypes rename only; raid.rs diff is the earlier trailing-space work. |
| 651 | pass | Family: DropExperienceBlock 0-diff; item_entities.rs 1-line diff is earlier 26.1 powder-snow velocity arg, out of scope |
| 652 | pass | Family: ChunkMap$TrackedEntity.equals compares entity.getId() in both; 26.2 uses the single-return form. lan_entities.rs 0-diff vs 26.2. |
| 653 | pass | Family: LeashKnotRenderer + FenceBlock both 0-diff; leads.rs 0-diff |
| 654 | pass | Family: EntityType->EntityTypes rename only; caravan.rs 0-diff. |
| 655 | pass | ArgProvider in neither jar (26.3-only); 26.1 form is DataCommands$DataProvider, tree_data.rs fixed to it. |
| 656 | pass | Family: FlowerBlock 0-diff; flowers.rs 0-diff |
| 657 | pass | Family: WeatheringCopperChestBlock 0-diff; tests.rs 0-diff |
| 658 | pass | Family: TreeDecorator$Context 0-diff; decorator.rs 0-diff |
| 659 | pass | Family: 26.2 added removeChildren() to FrameLayout/HeaderAndFooterLayout/LinearLayout; absent in 26.1. GUI plumbing, no Rust model of layout children removal. |
| 660 | pass | Family: SocialInteractionsScreen.onClose differs only by Minecraft.setScreen -> Gui.setScreen; same behaviour (restore last screen). Menu chrome. |
| 661 | pass | Family: BanNoticeScreens 0-diff; ban_notice.rs 0-diff |
| 662 | pass | Family: CommandBlockEditScreen 2 lines: 26.2 added getCommandBlock/getPreviousY accessors, bodies identical; menu chrome |
| 663 | pass | GpuFormat is 26.2-only (cross-check); 26.1 uses TextureFormat.RGBA8. |
| 664 | pass | Family: EffectsInInventory.extractEffects differs only by Gui.getMobEffectSprite -> Hud.getMobEffectSprite (method moved class). inventory_effects.rs 0-diff ... |
| 665 | pass | LocatorBar is 26.2-only (cross-check); ClientWaypointManager + WaypointStyle present in the 26.1 jar. |
| 666 | pass | Family: ShelfRenderer 0-diff; shelf.rs 0-diff |
| 667 | pass | Family: TutorialToast + Icons both 0-diff; tutorial.rs 0-diff |
| 668 | pass | Family: EditWorldScreen backup lambdas differ only by getToastManager() -> gui.toastManager() accessor and SystemToast.multiline static -> constructor; same ... |
| 669 | pass | Family: PandaMoveControl: 26.2 genericized it (PandaMoveControl<T extends Panda> extends MoveControl<T>, uses inherited mob field) where 26.1 keeps a ... |
| 670 | pass | Family: SilverfishWakeUpFriendsGoal.tick: structural diff shows only a local-store reshuffle around the InfestedBlock cast; same bounds checks, same ... |
| 671 | pass | Family: Overlay: 26.1 declares abstract isPausing(), 26.2 abstract isPauseScreen() — a pure rename, no default body in either; the pause predicate's contract ... |
| 672 | pass | InvertableSetOptionState is 26.2-only (cross-check); selector.rs:45 cites 26.1's shouldInvertValue. |
| 673 | pass | Family: FluidModel$Unbaked.bake: structural diff shows only invokeinterface -> invokevirtual for MaterialBaker.get (26.2 made MaterialBaker a class); same ... |
| 674 | pass | NarrationTrigger in neither jar (26.3-only). |
| 675 | pass | Ported, verified: METHODS is 85 entries (management_schema.rs:20); upgrade_* appears only in the explanatory comment (:94-95). |
| 676 | pass | Family: RconClient: only the constructor's access modifier changed (package-private 26.1 -> public 26.2); run() identical. rcon.rs 0-diff vs 26.2. |
| 677 | pass | Family: ChorusPlantBlock 0-diff; place.rs 0-diff |
| 678 | pass | Family: Climate$RTree$SubTree 4 lines: both ctors protected->public only; index.rs 0-diff |
| 679 | pass | Family: PaleMossDecorator 0-diff; feature_catalog.rs diff is earlier 26.1 work (lift_config/coral), out of scope |
| 680 | pass | BoxBlockMatcher in neither jar (26.3-only); sculk.rs uses the direct loop (item 510). |
| 681 | pass | Family: SnowyBlock 0-diff; snow_freeze.rs 0-diff |
| 682 | pass | Family: RotatedPillarBlock 0-diff despite the differing tag; tree_blocks.rs 0-diff |
| 683 | pass | Family: DownloadedPackSource$3 differs only by getToastManager() -> gui.toastManager() accessor; same pack-download toast. Menu chrome. |
| 684 | pass | Family: ItemStackWithSlot codec: 26.1 fieldOf('slot').orElse(0), 26.2 ExtraCodecs.optionalAlwaysPresentFieldOf(..., 'slot', 0) — same shape (slot optional, ... |
| 685 | pass | Family: 26.1's MutableComponent has withColor(TextColor) as well as withColor(int); 26.2 dropped the TextColor overload. The Rust's with_color(u32) is the ... |
| 686 | pass | DensityFunction$Axes in neither jar (26.3-only marker); fusion.rs Axes is the OptiFine CTM one. |
| 687 | pass | Family: GraphStorage: 26.2 replaced chunksWaitingForNeighbors (Long2ObjectMap<List<RenderSection>>) with sectionsWaitingForChunkLoads ... |
| 688 | pass | OitStage in neither jar (26.3-only); oit.rs is 26.1-only engine OIT work. |
| 689 | pass | Family: 26.1's Tab interface declares abstract getLayout(); 26.2 dropped it (doLayout only). GUI tab plumbing. tabs.rs 0-diff vs 26.2. |
| 690 | pass | Family: AbstractBannerBlock 0-diff; banners.rs 0-diff |
| 691 | pass | SteppedInterpolationTracker in neither jar (26.3-only); batch 61 deleted the stepped tracker. |
| 692 | pass | Family: PistonHeadRenderer.submit: 26.2's submitMovingBlock gained a 4th int arg (passed 0); same head + base submissions. Render-submission plumbing; ... |
| 693 | pass | Family: SimpleInstance pure criterion->triggers package move (4 lines, incl. ContextAwarePredicate->predicates) |
| 694 | pass | PotentSulfurState is 26.2-only; no PotentSulfur classes in either 26.1 jar; unreachability notes in both potent_sulfur.rs files. |
| 695 | pass | Family: TrialSpawnerState$ParticleEmission lambdas differ only by BlockPos.getCenter() (26.1) vs Vec3.atCenterOf(pos) (26.2) — same center value. ... |
| 696 | pass | Ported, verified: item_activation.rs docs cite GameRenderer.displayItemActivation + ScreenEffectRenderer (:3-5), the 26.2 form. |
| 697 | pass | Family: Waypoint: 26.1 keeps MAX_RANGE = 60000000, which 26.2 dropped; the Rust does not model it. Icon.cloneAndAssignStyle: 26.1 maps Integer ... |
| 698 | pass | Hud$ContextualInfo is the 26.2 name; 26.1 Gui has ContextualInfo (Gui in the 26.1 jar); docs fixed to Gui. |
| 699 | pass | Allay$VibrationUser 0-diff; allay.rs 0-diff. |
| 700 | pass | Family: WorldCreationUiState.findResultFolder: 26.2 added a saves-folder existence check + createDirectory ('Could not create save folder'); 26.1 only ... |
| 701 | pass | Family: DetailsScreen.onClose differs only by Minecraft.setScreen -> Gui.setScreen. Menu chrome; confirm_experiments.rs 0-diff vs 26.2. |
| 702 | pass | UnpublishCommand is in the 26.2 jar, NOT in 26.1; /unpublish already removed (item 543). |
| 703 | pass | Family: SubShape constructor went protected (26.1) -> package-private (26.2); same 7-arg constructor, no logic. face.rs 0-diff vs 26.2. |
| 704 | pass | WoodType in the 26.1 jar; POPLAR absent from WoodType.class constants (26.3-only). |
| 705 | pass | Family: 26.2 added removeChildren() to the Layout interface (same as item 659's three layout classes); absent in 26.1. GUI plumbing; layout.rs 0-diff vs 26.2. |
| 706 | pass | Family: CachedUserNameToIdResolver 0-diff; profiles.rs 0-diff |
| 707 | pass | BiomeTags in the 26.1 jar (0-diff). |
| 708 | pass | Family: WorldUpgrader: 26.2 added setNameFormat to the upgrade thread factory; thread-naming only, same daemon threads. world_upgrader.rs 0-diff vs 26.2. |
| 709 | pass | Family: GrowingPlantBlock 0-diff; fixed RedstoneWireBlock comment to RedStoneWireBlock (both jars use capital S) |
| 710 | pass | SpawnTargetPoint in neither jar (26.3-only); spawn_finder.rs diff is the batch-72 port. |
| 711 | pass | SteppedInterpolationHandler$Step in neither jar (26.3-only). |
| 712 | pass | Family: SystemToastId: 26.2 added FRIEND_SYSTEM_NOTIFICATION; constructors identical (5000ms). No Rust reference to the new id. Menu chrome. |
| 713 | pass | Family: WarningScreen lambda differs only by Minecraft.setScreen -> Gui.setScreen. Menu chrome. |
| 714 | pass | Ported, verified: 0 LevelExtractor refs anywhere; extraction stays on LevelRenderer. |
| 715 | pass | Family: TestCommand$Info in neither jar; TestCommand differs only by the BlockEntityType->BlockEntityTypes rename |
| 716 | pass | Family: ServerPlaceRecipe 0-diff |
| 717 | pass | Family: GameRuleMap$Builder.map went package-private (26.1) -> private (26.2); same field, same set/build. gamerules.rs 0-diff vs 26.2. |
| 718 | pass | Family: SpearUseGoal's four distance constants went package-private (26.1) -> private (26.2); same fields, no method changes. spear_use.rs diff is the ... |
| 719 | pass | Family: RealmsUploadTooLargeException differs only by a field access modifier; upload.rs 0-diff |

— End of Batch 11 report (214 items triaged, 8 defects, 2 oracle defects, 3 accepted notes). —
