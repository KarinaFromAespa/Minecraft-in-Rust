# 1.21.10 parity sweep — summary

**Version:** 1.21.10 · **Date:** 2026-11-25 · **Branch:** `parity-sweep-1.21.10`
(synced to main `09151952`) · **Ledger:** `pins/1.21.10/sweep-items.json`
(merged from the 13 per-batch slices in `pins/1.21.10/sweep/batch-NN.json`).
Per-batch reports live in this directory (`batch-N-*.md`); the universe
inventory is `inventory.md` / `inventory-1.21.10.json`.

Checks + triage only. No fixes were made in any batch; no production code
under `versions/1.21.10/` was touched. Every `fix` field in the merged ledger
is null. Batches ran 2026-11-25 (03:05–05:45 NPT); the merge landed the
same morning.

## Headline

| | Total | Pass | Gap | Accepted-gap |
|---|---:|---:|---:|---:|
| Checks as tallied | 9,711 | 8,015 | 1,696 | 0 |
| Checks, behaviour only | 9,667 | 8,006 | 1,661 | 0 |

The two rows differ by 44 non-behaviour checks three batches count in
their as-tallied tallies, each per a 26.2 precedent the coordinator
accepted: Batch 4 counts 37 driver-status checks (8 pass / 29 gap),
Batch 3 counts 3 driver-code-defect checks (0 pass / 3 gap), Batch 9
counts 3 oracle items + 1 harness blocker as checks (1 pass / 3 gap:
H01 + O01/O02/O03). No other batch counts oracle-status checks. Quote
the behaviour-only row when comparing against other versions' behaviour
tallies.

**Ledger items:** 402 raw across the slices, **379 unique** after dedup
(24 whole-item duplicates folded, 1 item minted at merge — see Dedup
below). **307 live** (204 gap + 94 oracle-defect + 9 needs-harness),
**75 live major** (57 gap + 18 oracle-defect by status; the 57 includes
the 11 Batch 0 repair groups, all major, and B9-O05). Non-live: 43 fixed
(1.21.7 defects withdrawn for 1.21.10), 26 closed (1.21.7 defects
re-checked and closed), 3 accepted-gap.

**Comparison with the 26.1.2 sweep** (9,855 checks, 8,260 pass, 1,595
gap, 302 items, 262 live, 61 live major), **the 26.1.1 sweep** (9,851
checks, 8,254 pass, 1,596 gap, 300 items, 262 live, 60 live major),
**the 26.1 sweep** (10,016 checks, 8,502 pass, 1,514 gap, 201 items,
186 live, 50 live major), **the 26.2 sweep** (10,374 checks, 10,058
pass, 314 gap, 113 items, 20 live major) **and the 1.21.7 sweep**
(10,341 checks, 7,730 pass, 2,609 gap, 208 items, 35 major): 1.21.10
ran 144 fewer checks than 26.1.2 (B12's registry count is smaller —
8,248 vs 8,593, because 1.21.10 ships fewer data-driven files; B8 ran
274 vs 279) and found 101 more gaps (1,696 vs 1,595), 45 more live
items (307 vs 262) and 14 more live majors (75 vs 61). On paper 1.21.10
is the dirtiest sweep yet — and unlike 26.1.2, the dirt is not
inheritance from one parent ledger: it is three layers at once. The
1.21.7 fixes that were supposed to carry into the E6 template never
did (fluid spreading, the supports_* synthesis, the motion-tag
derivation, the palette reader). The 1.21.11 template's own defects
came down with the port (the palette reader, the ServerLevel ctor
form). And the oracle measurement is the worst of any sweep so far:
only 26.6% of the 391 oracle files compile (26.1.2: 48.3%), with 220
of 391 deviating from 26.1.2's table (85 new-fail, 135 changed-error,
**0 new-pass**). What is genuinely new here is the six supports_*
survival blocks (B5-G37–G42 + B12-G09), the ServerLevel ctor group
(O10, 86 files), the CommandExec oracle (B9-O05, 45 errors), the
ore-vein gap (B6-G18), the client bonemeal inversion (B5-G36), the
Assets::load startup blocker (B12-G04), and four items that never
existed (B5-G17, B5-G18, B2-G11/G12, B10-G04 — see below).

## Per-batch tallies

| Batch | Scope | Checks | Pass | Gap | Items |
|---|---|---:|---:|---:|---:|
| 0 | Oracle measurement | 391 | 104 | 287 | 13 |
| 1 | Fluids + falling blocks | 95 | 77 | 18 | 19 |
| 2 | Redstone + pistons | 53 | 46 | 7 | 21 |
| 3 | Entities / mobs | 97 | 61 | 36 | 59 |
| 4 | Block entities + containers | 95 | 41 | 54 | 63 |
| 5 | Block behaviour | 128 | 102 | 26 | 37 |
| 6 | Worldgen / features | 27 | 18 | 9 | 27 |
| 7 | Items | 44 | 30 | 14 | 49 |
| 8 | Particles | 274 | 262 | 12 | 22 |
| 9 | Commands | 24 | 5 | 19 | 27 |
| 10 | Rendering, sound, UI | 24 | 13 | 11 | 18 |
| 11 | Rank-3 triage (719 items) | 211 | 208 | 3 | 39 |
| 12 | Data-driven registries | 8,248 | 7,048 | 1,200 | 8 |
| **Total** | | **9,711** | **8,015** | **1,696** | **402 raw / 379 unique** |

B5's tally is the coordinator-corrected 102/26 (101/27 as delivered —
see Coordinator corrections). B3/B4/B9 count non-behaviour checks in
their as-tallied tallies (see Headline); their behaviour splits are B3
94 (61/33), B4 58 (33/25), B9 20 (4/16).

## The story: 1.21.10 is the template's first child — and the 1.21.7 fixes didn't carry

Step 0 (provenance, verified before any batch was briefed) established
that 1.21.10 is template-derived from the 1.21.11 template — the first
adjacent delta port in E6, cross-protocol 774→773, 605 files differing
from the template. The behaviour port itself is 719/719, merged
fcb0e7e3 with crate-tests 45376855229 fully green (6,995 tests, 0 fail).
The holder shape is 1.21.x (no EntityTypes/BlockEntityTypes in either
jar), confirmed by jar listing. The jars are the named game jars
(client-named 9,969 classes, server-game-named 6,964 classes, sha1s
verified); the `server.jar` in the same directory is a Mojang bundler
(4 classes) and was never javap'd.

The sweep's central finding is a carry-over failure. The 1.21.7 port
accumulated fixes through November 22–23 — fluid spreading, the
supports_* tag synthesis, the motion-tag derivation, the palette
reader, the ServerLevel ctor repair — and none of them reached the
1.21.11 template or its first child:

- **No fluid spreading at all** (B1-G05, major): reproduces the pilot
  G5, "fixed" in the 1.21.7 ledger; the fix did not carry. The port's
  fluid code still reads 26.x tags that 1.21.10 never shipped.
- **Seven blocks can never survive anywhere** (B5-G37–G42 + B12-G09,
  all major): sugar cane, cocoa, azalea, mangrove propagule (both
  forms), nether sprouts, nether wart, frogspawn. The survival code
  reads eleven supports_* tag names that do not exist in 1.21.10; the
  1.21.7 tree's `synthesize_legacy_support_tags` fix was never
  carried. Each block's vanilla form was javap-verified fresh.
- **The client cannot start** (B12-G04, major): `Assets::load`
  hard-requires three 26.x-only motion tags
  (`blocks_motion_in_heightmap`, `..._no_leaves`, `blocks_fluid_flow`)
  with hard errors; 1.21.10 never shipped them and no synthesis or
  derivation exists in the tree. A second, independent startup
  blocker lives in B3-G22 (major): `MotionResources::load` requires
  the absent `#suppresses_bounce` / `#causes_suffocation` tags.
- **Vanilla-written worlds load as void** (B11-G10, major): no
  `from_palette_nbt` anywhere in the tree; `from_nbt` reads strict
  `"id"` only; datafix 5006 never runs at DataVersion 4556; the
  reference world uses `Name`/`Properties` palettes throughout.
  Inherited from the 1.21.11 template, statically derived (not
  demonstrated in-game).
- **No 1.21.10 world can generate at all** (B6 cascade, 21 major):
  empty carver/feature bundles (G06) → 7 missing density arms (G13)
  → `material_rule` (G11) → `spawn_target` (G12), with aquifers
  (G09) behind them. Plus NEW: ore veins never generate (G18).
- **Both loot engines dead** (B12-G01/G02, both major): items.rs
  makes every condition false (1,186 gap checks); loot.rs ignores all
  conditions and functions (demonstrated live with `loot_fill` — the
  first loot_fill run in any sweep).
- **The gamerule table is wrong** (B9-G01, major): snake_case table
  vs vanilla's 59 camelCase rules, with the fire-pair collapse
  (vanilla's `doFireTick` / `allowFireTicksAwayFromPlayer` bool pair
  vs the engine's single int).
- **Four 26.x-only commands in the release tree** (B9-G02, major):
  `compute`, `posteffect`, `swing`, `unpublish` — vanilla 1.21.10 has
  no `/swing` at all.

The oracle layer is the worst measured in any sweep: 391 files, 104
pass / 287 fail-all on the client classpath (26.6%), 314 drivers
blocked, only 2 of 9 harness oracles passing. The new O10 group
(ServerLevel ctor, 86 files) is 1.21.10-specific: its ctor is 11-arg
with RandomSequences, and the 1.21.7 12-arg fix does NOT carry. The
1.21.11-era package moves are absent in 1.21.10 (gamerules 24, entity
sub-packages 32, world.attribute 28, gizmos 4, permissions 9). The
`VanillaCommandExecOracle` fails with 45 errors (B9-O05, major, NEW),
blocking the linchpin `command_exec` driver on top of its
`effect_view:always` fixture blocker.

It is not all dirt. Four 1.21.7 defects are genuinely fixed here, and
two of them are firsts across the sweep programme:

- **Redstone wire FIXED** (B2-G01): the class-alias mechanism
  dispatches wire correctly — the first E6 version where wire works.
- **Sign boards FIXED** (B10-G01): signs render boards — the first
  version in the sweep where they do.
- **Bubble columns FIXED** (B1-G10) and **doFireTick FIXED**
  (B1-G11): the port honours doFireTick via `gamerules::fire_radius`.
- **Beds place correctly** (B11-G20, fixed — 1.21.7's B11-G01): beds
  place foot+head and sleep works. But beds are INVISIBLE (B10-G03,
  major) — the renderer doesn't draw them. The 1.21.7 beds fix carried;
  the 1.21.7 beds renderer defect did not get fixed.
  35m37s, confirmed independently by B9's driver build, which
  compiles AND links it) — so `command_parse` ran end-to-end, the
  only end-to-end driver run in this sweep besides `loot_fill`.
- **Lib tests RUN**: crate-tests 45376855229, 6,995 passed / 0 failed
  — the 26.1.2 B11-H01 withdrawal is confirmed as the new normal, not
  a one-off.

## Live majors (75)

**Startup / world-loading (5):** B12-G04 (Assets::load motion tags —
client cannot start), B3-G22 (MotionResources tags — suffocation
startup blocker), B11-G10 (vanilla worlds load as void), B12-G01
(items.rs loot engine dead — every condition false), B12-G02
(loot.rs loot engine dead — conditions/functions ignored).

**Worldgen cascade (19 live of 21 — G10 and G15 are fixed):** B6-G01,
B6-G02, B6-G03, B6-G06, B6-G09, B6-G11, B6-G12, B6-G13, B6-G14,
B6-G16, B6-G18 (ore veins never generate) + the eight B6 oracle
groups O01–O08 (densityfunction, material/rule, feature, spawn, and
template-API drift).

**Survival / blocks (12):** B5-G37–G42 (six supports_* blocks),
B12-G09 (frogspawn), B5-G04 (fungi/roots — worse than 1.21.7),
B5-G13 (dripstone tag half dead), B5-G21 (snow/ice), B5-G22
(composter), B1-G01 (dripstone system half dead).

**Fluids (2):** B1-G05 (no fluid spreading at all), B1-G03 (splash
water never douses — canonical of the 3-way fold).

**Redstone / pistons (2):** B2-G02 (push-reaction loader matches no
1.21.10 value — every state loads as PushPull), B2-G03 (is_pushable
lacks vanilla's hardcoded obsidian-family refusal).

**Entities / block entities (8):** B3-G14 (endermen never teleport),
B4-G01 (furnaces never light), B4-G02 (brewing never brews), B4-G16
(cartography), B4-G30 (composter), B7-G07 (tool transforms), B7-G08
(campfire douse), B7-G09 (chorus fruit).

**Commands / UI / sound (6):** B9-G01 (gamerule table), B9-G02
(26.x-only commands), B9-O05 (CommandExec oracle, 45 errors),
B8-G10 (/spreadplayers always fails), B10-G02 (no sounds.json),
B10-G03 (beds invisible).

**Oracle families (11 B0 groups + 10 singletons):** O1 (LocalPlayer
ctor), O2 (jsr305 crash), O3 (fuelValues), O4 (densityfunction), O5
(EntityTypes holder move), O6 (26.3/1.21.11-only packages), O7 (287
fail / 314 drivers blocked), O8 (runtime layer unmeasured), O9
(re-ported 26.3-form sources), O10 (ServerLevel ctor, 86 files),
O12 (ComponentPatch runtime crash); B2-O01 (BlockUpdate, 63 errors),
B2-O02 (RedstoneCatalog, 36 errors), B2-O06 (FaceSupport), B4-O06
(Dispenser), B4-O17 (ContainerScreen), B4-O23 (NaturalSpawn), B4-O24,
B10-O03 (render family, 21 oracles), B10-O04 (sound family, 11
oracles), B10-O05 (UI family, 11 oracles).

Full list in the ledger (`totals.live_major_ids`).

## Strongest verifications

- **Commands (B9):** the run — 32,365 commands × 3 permission levels
  = 97,095 parse comparisons, 90,457 match, 6,638 mismatches across
  3,325 unique inputs (perm 4: 3,325, perm 2: 3,312, perm 0: 1). The
  corpus was regenerated from 1.21.10's own tree export (md5
  `6fa6f661f641dc84f86c099184f1c644`), 494 EXTRA entries, 255 gamerule
  cases. The node diff confirms G16 (157 / 153 / 0 inputs). Perm 0's
  single mismatch is the empty-input root list: Rust's 78 names =
  vanilla's 74 + exactly the four G02 extras. 0 raw-key error
  emissions in the whole run (O04 mitigation confirmed). 14
  oracle-crash rows skipped by compare() (10 `random`, 4 `schedule
  clear` — the latter hide G20's divergence). The run contradicted NO
  static verdict. The first driver launch was killed by a VM reboot
  at ~23:50 UTC; relaunched, the build finished in 29m31s.
- **Registries (B12):** all data-driven files byte-identical to the
  1.21.10 jar where they exist; loot census identical in shape to
  26.1.2's (1,186 gap checks under items.rs). `loot_fill` WAS run —
  G02 demonstrated live, the first loot_fill run in any sweep. This
  batch built horizon-prepare (24m17s) and ran it to completion (exit
  0); the prepared 1.21.10 tree now exists in the worktree's `.local`.
- **Worldgen (B6):** reference-world census (8,363 chunks, 16 biomes,
  seed 6826278913115239597 confirmed from level.dat
  WorldGenSettings); the construction cascade verified link by link
  against code + prepared data + jar.
- **Entities (B3):** the suffocation rule is fixed but the load path
  still requires the absent tags (G22) — the fix and the blocker
  distinguished by direct code read.
- **Items (B7):** all 1,488 items mapped, 0 unmapped; four re-check-
  first regressions reproduce (G07 tool transforms, G08 campfire
  douse, G09 chorus fruit, G10 splash water).
- **Rank-3 (B11):** class-table jar half run (265/271 live, 6
  jar-dead all inert); G10 verified end-to-end by the coordinator
  (reader removed + save.rs maps unresolvable entries to air +
  reference world uses `Name`/`Properties` palettes with zero strict
  `id` entries). Lib tests RUN (crate-tests 45376855229: 6,995 passed
  / 0 failed).
- **Fluids (B1):** the doFireTick fix verified by the coordinator
  against the tree (`gamerules::fire_radius` →
  `fire_spread_radius_around_player` → `can_spread_fire_around`,
  consulted by tick.rs:635 and fluids.rs:748) — B1-G11 stays FIXED
  against B5's contrary claim.
- **Particles (B8):** `ParticleTypes.<clinit>` parsed fresh from the
  1.21.10 jar with the B8 parser gotcha observed (the limiter boolean
  is the `iconst` AFTER the name `ldc`); G10 (/spreadplayers always
  fails) is the batch's only major.
- **Block behaviour (B5):** the G17 correction verified by the
  coordinator against both jars (`getMaxY()` is 319 in 1.21.10's AND
  26.1.2's jar — the chorus-ceiling defect never existed); the G18
  correction confirmed fresh (bell isProperHit — the branch direction
  was misread in 1.21.7).
- **Rendering (B10):** the G04 correction verified by the coordinator
  against the 1.21.7 tree itself (all five geometry functions present
  with live call sites — the correction extends to the 1.21.7 ledger);
  the sign-board fix verified (G01, first version where signs render
  boards).
- **Redstone (B2):** the wire fix verified (class-alias mechanism,
  first E6 version where wire dispatches); the G11/G12 corrections
  verified against 1.21.7's own bytecode (both never existed).
- **Oracle measurement (B0):** 391 files, 287 fail-all (73.4%), 314
  drivers blocked; zero deviations vs the 26.1.2 table in the pass
  direction (0 new-pass); per-batch O-items name 168 distinct files,
  152 genuinely failing.

## Needs-harness (9)

All minor: B2-H01 (tick-by-tick circuit equivalence + piston corpus
driver runs), B2-H02 (the 4 level::tests need a cargo test run), B6-H
(paired port-vs-vanilla chunk comparison — stage rule applies),
B8-H01 (providers driver omits vibration), B8-H02 (providers driver
includes 13 kinds 1.21.10 lacks), B8-H03 (marker driver zoo includes
red_poplar_leaves), B8-H05 (entity driver sends nautilus cases for an
entity 1.21.10 lacks), B11-H01 (class-table chain-liveness half cannot
run — no block-behaviours.json for 1.21.10), B11-H02 (cauldron
interaction table vs Rust — carries from 1.21.7).

## Dedup record

24 whole-item folds (duplicate removed from the merged list, kept in
its slice): splash water 3-way (B7-G10, B11-G05 → B1-G03), mud
(B7-G11 → B5-G19), spawner setEntityData (B7-G17 → B3-G39), dripstone
tag half (B11-G04 → B5-G13), supports_* census (B12-G03 → B5-G37,
the first of the six per-block items), cauldron 3-way (B5-O01,
B11-O01 → B1-O02), FallingBlock (B5-O05 → B1-O03), LavaCauldronItem
3-way (B7-O07, B11-O02 → B1-O07), EntityFluid (B3-O08 → B1-O06),
dispenser 3-way (B5-O06, B7-O01 → B4-O06), BlockUpdate (B7-O02 →
B2-O01), FaceSupport (B5-O09 → B2-O06), VehicleContainer (B7-O10 →
B4-O21), EnchantDurability cascade (B7-O29 → B4-O19), render/UI
singletons → B10 families (B7-O09, B5-O07, B5-O08, B4-O16 → B10-O03;
B4-O18, B4-O08 → B10-O05).

19 oracle member folds (group item stays; the member folds into the
earlier canonical item): see `dedup.oracle_member_folds` in the
ledger. Cascade/dependency mentions are NOT members — BlockUpdate
named in B4-O03/B4-O19/B7-O29/B7-O30/B3-O07 as the cascade source,
FaceSupport named in B2-O07/B3-O07 as the cascade source, and
RegistryFluid named in B1-O04 as a shared-symbol cross-reference get
no fold.

1 item minted at merge: B12-G09 (frogspawn can never survive) from
B12-G03's evidence — the census item folds into the six B5 per-block
items, and frogspawn is its seventh block, covered nowhere else.

Kept separate (cross-referenced, not folded): B1-G01 vs B5-G13
(dripstone — B1 covers the fluid-transfer/cauldron side plus the
blockToGrowOn link cause); B11-G07 vs B4-G01 (furnace fuel — B11 is
the recipe-book fuel list, B4 is the furnace lighting); B12-G04 vs
B3-G22 (startup blockers — different loaders, different tags);
B4-O27 vs B3's trading territory (cross-reference only).

## Coordinator corrections (during reconciliation)

1. **B5 tally: 101/27 → 102/26.** B5 claimed the port ignores
   doFireTick (no `do_fire_tick` field on the GameRules struct). The
   coordinator checked the tree directly: the port honours it via
   `gamerules::fire_radius` → `fire_spread_radius_around_player` →
   `can_spread_fire_around`, consulted by tick.rs:635 and
   fluids.rs:748. **B1 is right, B1-G11 stays FIXED.** B5's
   "(B1-G04, cross-reference)" label is also a mislabel (B1-G04 is the
   eye check; the doFireTick item is B1-G11).
2. **B7-O11: fixed → oracle-defect.** B7 claimed KnockbackClient is
   fixed (no ServerboundPunchPacket ref, all 49 imports exist) and
   flips item_use_order to runnable. B0's table says it FAILS on all
   three classpaths (7 errors, no-override, first error line 362).
   The 26.1.2 cause is gone but a new cause replaces it. The
   item_use_order driver does NOT flip (B7's inventory corrects to 32
   runnable / 25 blocked).
3. **B5-O03: oracle-defect → fixed.** B5 claimed
   VanillaPickBlockDataOracle fails because TypedEntityData has no
   `loadInto`. B0's table says it PASSES on all three — and the
   coordinator's own javap confirmed TypedEntityData HAS both
   `loadInto` overloads in 1.21.10's jars. B5's javap read was wrong.
4. **H01 timeline (B9's report).** B9's report claims "Batch 0's cargo
   check never completed (also killed by the reboot)". Wrong — B0's
   cargo check completed EXIT=0 in 35m37s
   (evidence/client-probe-check2.log) and B0 itself completed 23:19
   UTC, BEFORE the ~23:50 UTC reboot, which killed B9's first driver
   launch, not B0's check. H01 is confirmed twice over (B0's cargo
   check + B9's driver build, which compiles AND links the probe).
5. **B9 brief corrections (worker-verified).** `fetchprofile` and
   `publish` DO exist in vanilla 1.21.10 (the brief said to drop them
   as 26.x-only; their EXTRA cases were kept); vanilla has 59
   camelCase gamerules.

**Oracle reconciliation:** 94 oracle-name checks across the 12 landed
slices found exactly the 2 contradictions above (B7-O11, B5-O03).
Spot-checks agreeing with the B0 table: VanillaCommandExecOracle
fail-all 45 errors; VanillaChunkGenerationOracle fail-all 24;
VanillaFluidBoxOracle fail-all 16; VanillaBlockUpdateOracle fail-all
63; VanillaRedstoneCatalogOracle fail-all 36; VanillaEntityParticles-
Oracle fail-all 18; VanillaComponentPatchOracle PASS all three (B0-O12
is a runtime crash, not javac); VanillaStructureDensityOracle fail-all
4; VanillaEffectParticlesOracle PASS all three. B2-O08/O09
(driver-script `--resources` path defaults) are misclassified as
oracle-defect — noted, NOT reclassified (live minor items either way).

## Ledger corrections owed to other ledgers (parent action)

- **1.21.7 ledger:** B2-G11/G12 NEVER EXISTED (1.21.7's own bytecode
  contradicts both; the 26.x "vanilla changed" framing was wrong);
  B10-G04 NEVER EXISTED extends to 1.21.7 (verified: the 1.21.7 tree
  has all five geometry functions with live call sites). Also the
  diode guard (B2-G07) is a real defect the 1.21.7 sweep missed
  (redstone.rs byte-identical between the trees).
- **26.1.2/26.1.1/26.1 ledgers:** B5-G17 chorus ceiling NEVER EXISTED
  (getMaxY() is 319 in both jars, verified fresh in 26.1.2's jar too).
- From the 26.1.2 run (still owed): B5-G18 bell + B10-G04 BE renderers
  NEVER EXISTED — corrections owed to the 26.1/26.1.1 ledgers
  "whenever those are next touched".

## What the 1.21.11 run needs

1. **PROVENANCE IS THE TEMPLATE ITSELF.** 1.21.11 is the E6 template;
   the re-check-first list is the 1.21.10 ledger (379 items) + the
   1.21.7 ledger (208). Expect the template's own defects to show
   here first: the palette reader (B11-G10 was inherited FROM the
   template — verify fresh whether 1.21.11's chunk.rs has
   `from_palette_nbt`), the ServerLevel ctor form (1.21.10 is 11-arg
   with RandomSequences; 1.21.7's 12-arg fix does not carry to
   1.21.10 — establish 1.21.11's own arity by javap), the supports_*
   synthesis (check whether the 1.21.11 tree has
   `synthesize_legacy_support_tags` — 1.21.10 doesn't), and the
   motion-tag derivation (check whether 1.21.11's `Assets::load`
   hard-requires the motion tags).
2. **THE LOADING STICK.** 1.21.11 carries the identical code to the
   1.21.7 loading-stick defect (fixed in 1.21.7 by 775a25c2, merged
   8123e438 — a pause landing after Login and before the load tracker
   finishes deadlocks three ways). The 1.21.11 sweep should check for
   it; propagation is a separate owner decision.
3. **CARRY THE FOUR NEVER EXISTED CORRECTIONS** (B5-G17 chorus
   ceiling, B5-G18 bell isProperHit, B2-G11/G12, B10-G04 BE
   renderers) but CHECK FRESH against 1.21.11's own jar/tree — do NOT
   re-mint without fresh bytecode/geometry evidence.
4. **B8 parser gotcha:** the particle limiter boolean is the `iconst`
   AFTER the name `ldc` — a backwards walk picks up the previous
   registration's boolean.
5. **Probe:** check whether 1.21.11's client_probe compiles BEFORE
   briefing Batch 9, then run command_parse. REGENERATE the command
   corpus from 1.21.11's own tree export — do NOT reuse the 1.21.10
   corpus (1.21.10's corpus + scripts are preserved in
   sweep-1.21.10-batches/b09/ for reference only). Budget for the
   probe build (30m+ under load; the 1.21.10 driver build took
   29m31s after a reboot killed the first launch).
6. **Establish the holder shape by jar listing** (1.21.x shape
   expected — 1.21.11 predates the EntityTypes move).
7. **Name the tally precedents in every batch brief:** oracle/harness/
   driver-status checks are NOT tallied; the restatement is the
   coordinator's job.
8. **`coordinator_note` is the coordinator's** — say so in the brief.
9. **Brief oracle names, not short names:** full `Vanilla*Oracle`
   names; keep driver/transcript names (EntityTranscript,
   RideTranscript) out of oracle counts.
10. **Keep analysis artifacts under `~/workspace/mcir-work/`**, never
    `/tmp`.
11. **Verify report completeness before accepting a batch** (read the
    tail; grep for truncation markers).
12. **Lib-test breakage:** check whether 1.21.11's regression tests
    RUN rather than assuming breakage (1.21.10's lib tests run green
    — crate-tests 45376855229, 6,995 passed / 0 failed).
13. **The NEVER EXISTED corrections owed to the 1.21.7 / 26.1 /
    26.1.1 / 26.1.2 ledgers are a PARENT action, not yours** — do not
    touch those ledgers.
14. **Disk:** the 1.21.10 run hit the VM quota mid-prep (104G used).
    Free space per the standing rule before briefing (delete completed
    sweeps' `.local` + `target/` after the safety checks); keep the
    1.21.10 worktree's `.local` until Batch 9 has run (the driver
    build resumes from its cached target dir).
