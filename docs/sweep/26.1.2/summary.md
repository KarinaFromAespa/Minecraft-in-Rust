# 26.1.2 parity sweep — summary

**Version:** 26.1.2 · **Date:** 2026-11-25 · **Branch:** `parity-sweep-26.1.2`
(synced to main `db11596a`) · **Ledger:** `pins/26.1.2/sweep-items.json`
(merged from the 13 per-batch slices in `pins/26.1.2/sweep/batch-NN.json`).
Per-batch reports live in this directory (`batch-N-*.md`); the universe
inventory is `inventory.md` / `inventory-26.1.2.json`.

Checks + triage only. No fixes were made in any batch; no production code
under `versions/26.1.2/` was touched. Every `fix` field in the merged ledger
is null. Batches ran 2026-11-25 (00:26–02:42 NPT); the merge landed the
same morning.

## Headline

| | Total | Pass | Gap | Accepted-gap |
|---|---:|---:|---:|---:|
| Checks as tallied | 9,855 | 8,260 | 1,595 | 0 |
| Checks, behaviour only | 9,811 | 8,242 | 1,569 | 0 |

The two rows differ by 44 non-behaviour checks three batches count in
their as-tallied tallies, each per a 26.2 precedent the coordinator
accepted: Batch 4 counts 37 driver-status checks (17 pass / 20 gap),
Batch 3 counts 3 driver-code-defect checks (0 pass / 3 gap), Batch 9
counts 3 oracle items + 1 harness blocker as checks (1 pass / 3 gap:
H01 + O01/O02/O03). No other batch counts oracle-status checks. Quote
the behaviour-only row when comparing against other versions' behaviour
tallies.

**Ledger items:** 333 raw across the slices, **302 unique** after dedup
(31 whole-item duplicates folded — see Dedup below). **262 live** (189
gap + 66 oracle-defect + 7 needs-harness), **61 live major** (47
behaviour-gap + 14 oracle-defect by status; the 47 includes the 9 Batch 0
repair groups, all major). Non-live: 9 fixed (26.1.1 defects withdrawn
for 26.1.2), 28 closed (26.1.1 defects re-checked and closed),
3 accepted-gap.

**Comparison with the 26.1.1 sweep** (9,851 checks, 8,254 pass, 1,596
gap, 300 items, 262 live, 60 live major), **the 26.1 sweep** (10,016
checks, 8,502 pass, 1,514 gap, 229 items, 186 live, 50 live major),
**the 26.2 sweep** (10,374 checks, 10,058 pass, 314 gap, 113 items, 20
live major) **and the 1.21.7 sweep** (10,341 checks, 7,730 pass, 2,609
gap, 208 items, 35 major): 26.1.2 ran 4 more checks than 26.1.1 (B3 +2:
the G47 check and the handleAttack check; B10 +1: the Checkbox check;
B11 +1: 7 ledger items vs 6) and found 1 fewer gap (1,595 vs 1,596),
the same live count (262) and 1 more live major (61 vs 60 — the new
B11-G10). On paper 26.1.2 is the dirtiest 26.x sweep
yet, by one major — but the honest framing is inheritance: this run is
a re-verification of the 26.1.1 ledger against a 7-item delta, and the
dirt is 26.1.1's dirt, carried over wholesale. The pass count rose by
6: two checks flipped gap → pass on corrections, not fixes (B5-G18,
B10-G04 — see below), the four new checks split 3 pass / 1 gap, and
B11's sixth check went from needs-harness to pass (H01 withdrawn).
What is genuinely new
here is two delta regressions (B11-G10, B11-G11), one defect the 26.1
and 26.1.1 sweeps both missed (B3-G47), and two 26.1.1 items that never
existed (B5-G18, B10-G04).

## Per-batch tallies

| Batch | Scope | Checks | Pass | Gap | Items |
|---|---|---:|---:|---:|---:|
| 0 | Oracle measurement | 391 | 189 | 202 | 9 |
| 1 | Fluids + falling blocks | 93 | 73 | 20 | 17 |
| 2 | Redstone + pistons | 52 | 45 | 7 | 17 |
| 3 | Entities / mobs | 98 | 63 | 35 | 46 |
| 4 | Block entities + containers | 95 | 47 | 48 | 62 |
| 5 | Block behaviour | 128 | 103 | 25 | 29 |
| 6 | Worldgen / features | 27 | 19 | 8 | 25 |
| 7 | Items | 44 | 28 | 16 | 49 |
| 8 | Particles | 279 | 267 | 12 | 16 |
| 9 | Commands | 24 | 7 | 17 | 26 |
| 10 | Rendering, sound, UI | 24 | 11 | 13 | 17 |
| 11 | Rank-3 triage (7 items) | 7 | 7 | 0 | 16 |
| 12 | Data-driven registries | 8,593 | 7,401 | 1,192 | 4 |
| **Total** | | **9,855** | **8,260** | **1,595** | **333 raw / 302 unique** |

## The story: 26.1.2 is 26.1.1 plus a 7-item delta

Step 0 (provenance, verified before any batch was briefed) established
that the 26.1.2 tree is a **delta port of the 26.1.1 tree**: 34 files
differ, none in the regression sites — `fluids.rs` is byte-identical to
26.1.1's, as are the furnace, brewing, enderman, tool-transform,
campfire and command-tree files. 26.1.2 is NOT a fresh 26.3 re-port and
NOT re-derived from the 26.1 tree: it inherits 26.1.1's re-port defects
wholesale. The vanilla delta is a DataVersion bump (4788 → 4790,
embedded in the 1,202 structure files) + 8 changed class files / 5
top-level classes (Batch 0, by file diff; PORT-NOTES says "6 changed
classes" — substance agrees: no public API change, only two private
`Checkbox` members changed). Packets, registries, blocks, items and
command nodes are all unchanged. The behaviour universe is
byte-identical to 26.1.1's in every count (1,168 blocks, 29,873 states,
157 entities, 1,506 items, 117 particles).

- **The twelve 26.1 regressions all reproduce.** B1-G05 no fluid
  spreading, B3-G14 endermen never teleport, B4-G01 furnaces never
  light, B4-G02 brewing never brews, B7-G07 tool transforms, B7-G08
  campfire douse, and the five command flips B9-G03/G05/G07b/G12/G13
  — each re-verified fresh against 26.1.2's own jar/data, each with
  byte-identical code to 26.1.1's. The 26.1 tree's adaptations are
  still not carried.
- **The delta adds two regressions of its own.** **B11-G10** (major,
  NEW): the 26.1.2 delta removed `BlockState::from_palette_nbt` from
  `chunk.rs`, so the port reads only the strict `id`/`properties`
  palette form while vanilla 26.1.2 still writes `Name`/`Properties`
  palettes (proven in the reference world) — and the load path maps
  unresolvable entries to air, so a vanilla-written 26.1.2 world
  loads as void. The rename is datafix 5006, which never runs at
  CURRENT 4790. CI corroborates: 4 of run 45150986405's 41 failures
  are save/datafix tests asserting the `id` form the upgrade cannot
  produce. Caveat: the void consequence is derived statically from
  the decode chain, not demonstrated by running the game.
  **B11-G11** (minor, NEW): 26.1.2 dropped 26.1.1's
  `chat_disabled_or_blocked` fix + test from `social_screen.rs`;
  vanilla `PlayerEntry` is byte-identical, so the flipped formula
  still applies. Uncalled code today, so no observable behaviour
  change — the formula pin is lost.
- **One defect the 26.1 and 26.1.1 sweeps both missed: B3-G47**
  (minor, NEW) — the AI walk-target value misses the Zero arms and
  the Giant/Mooshroom arms. Both earlier sweeps checked only the
  spawn table.
- **Two 26.1.1 items never existed — corrections, not fixes.**
  **B5-G18** (bell `isProperHit`): the 26.2 run misread the `ifle`
  branch direction in the bytecode, concluding vanilla requires
  `click_y > 0.8124`; the misread propagated through the 26.1 and
  26.1.1 runs. Fresh bytecode re-read + a Vineflower decompilation
  prove vanilla requires `click_y <= 0.8124` — exactly what the port
  implements (`redstone.rs:1442`), including the exact float
  constant. **B10-G04** (seven "missing" BE renderers): the 26.1.1
  run searched only the `Kind` enum in `block_entity_render.rs` and
  missed the `game.rs` geometry path — all seven have port
  equivalents with live call sites (spawner, campfire, moving piston,
  brushable, vault, shelf + ominous-item spawner). Both land CLOSED
  here. Both correct the 26.1 and 26.1.1 ledgers too — the correction
  should reach them before anyone "fixes" a bell or a renderer that
  was never broken. (The 1.21.7 run's B10-G04 is a different tree and
  is NOT corrected by this run.)
- **The 7-item port ledger passes 7/7** (Batch 11). Only #4
  handleAttack and #6 Checkbox tooltip changed behaviour code, and
  both are verified ported with regression tests green in CI run
  45150986405 (horizon-client 1,274 passed / 0 failed). PORT-NOTES
  line 71 ("changed method body remains unported") is stale/wrong —
  the spectator guard IS ported in `lan/hub.rs:4063-4065` and
  `4083-4087`. Doc fix for the parent, not a sweep item.
- **B11-H01 is withdrawn.** The horizon-client lib-test breakage it
  named was fixed by the 26.1.2 port's enabling work (and the
  #4/#6 regression tests ran green in CI). The withdrawal is
  report-only — no slice item — and its loss is B11-G11.
- **The oracle table is byte-identical to 26.1.1's.** Batch 0
  measured all 391 oracle files fresh: zero deviations (identical
  file sets, 0 differing entries). 202 fail javac on all three
  classpaths (51.7%). No oracle was repaired in this sweep.
- **Two startup blockers** (same as 26.1 and 26.1.1). B3-G22: the
  client cannot load its core tables with prepared 26.1.2 data
  (motion requires two absent tags). B6-G06: no 26.1.2 world can
  generate at all (empty carver/feature bundles). Both need fix
  decisions, not just ledger entries — and both have the 26.2 tree as
  the repair template.
- **Live behaviour gaps: 180 G-items (38 major + 142 minor) + the 9
  Batch 0 repair groups (all major).** The minors are single-site
  slips of the same 26.3-form kind: tag spellings, missing food tags,
  inverted comparisons, off-by-ones, stale 26.3 constants in drivers.
- **Live oracle defects: 66 items naming 162 distinct failing files**
  (plus Batch 0's group items covering the other 40 failing files) —
  identical to 26.1.1, as the per-batch O-items re-decided with zero
  status diffs.
- **H01 stays fixed.** The probe compiles, and `command_parse` RAN
  end-to-end again — the only end-to-end driver run in this sweep
  (see Strongest verifications). The build was interrupted at 20
  crates by a service restart and resumed from the warm cache
  (56m56s, slowed by a competing 1.21.4 build on the shared VM).
- **No UNVERIFIED carry-overs except B4-G17.** Every 26.1.1 and
  1.21.7 item was decided against 26.1.2 evidence — except B4-G17
  (smoker/blast burn-time halving), which stays UNVERIFIED in both
  runs, masked by B4-G01; it has no slice item in either run.
  Arbitration: restore fuel data first, then check halving via the
  fast context.
- **Batch 0's prepared-data section is stale.** It was written before
  Batch 12 built and ran horizon-prepare (7m00s build, 614.5s run,
  exit 0). Prepared 26.1.2 data EXISTS in the worktree at
  `versions/26.1.2/.local/` (git-ignored, never committed). This also
  satisfies the B2-H01/H02 prepare precondition.

## Live majors (61)

**Behaviour majors (38):**

- **Startup blockers (2):** **B3-G22** — default suffocation tag-gated
  on absent `#causes_suffocation` (first failure `#suppresses_bounce`);
  the client cannot load its core tables. **B6-G06** — prepared
  carvers/features bundles empty; `GeneratorData::load` fails, no world
  can generate.
- **Worldgen cascade (8):** **B6-G09** aquifers entirely disabled,
  **B6-G10** NoiseRouter is the 26.3 eight-field form, **B6-G11**
  requires `material_rule` (26.1.2 has inline `surface_rule`),
  **B6-G12** spawn-target decode fails, **B6-G13** density decode
  missing cache / shifted-noise arms, **B6-G14** 11 feature types +
  coral mushroom + nether carver lack decode arms (22 configured
  features), **B6-G15** state-provider decode accepts only the 26.3
  short names (110 of 221 configured features fail), **B6-G16**
  `random_offset` placement modifier has no decode arm (70 placed
  features fail).
- **1.21.7 carry-overs (16 — the same sixteen as 26.1.1):** **B1-G01**
  entire dripstone system dead (3 dead causes), **B2-G01** redstone
  wire never updates power, **B2-G02** push-reaction loader dead,
  **B2-G03** obsidian pushable, **B4-G16** cartography paper zoom dead,
  **B6-G01** sculk catalyst/shrieker tail missing, **B6-G03**
  underground water deficit (demonstrated), **B7-G09** chorus fruit
  never teleports, **B9-G02** `/posteffect`/`/unpublish`/`/compute`
  remain in the release tree, **B10-G01** sign boards not rendered,
  **B10-G02** no `sounds.json` (game silent; all 6 audio drivers
  oracle-blocked too), **B10-G03** beds invisible when placed AND bed
  items fail to decode, **B11-G01** beds place foot-only / sleep
  unreachable, **B11-G05** splash water does nothing, **B12-G01**
  items.rs evaluates every loot condition as false and never runs any
  function (1,192 tables), **B12-G02** loot.rs ignores all conditions
  and functions.
- **Regressions from 26.1 closures (6):** **B1-G05** NO fluid
  spreading at all (was CLOSED in 26.1), **B3-G14** endermen never
  teleport (26.1 had cleared it), **B4-G01** furnaces never light and
  **B4-G02** brewing stands never brew (both regressed from 26.1
  FIXED), **B7-G07** axe/hoe/shovel transforms and honeycomb wax-on
  entirely non-functional (26.1 had withdrawn it), **B7-G08** shovel
  campfire douse dead (26.1 had withdrawn it).
- **Reproduces from 26.1's new majors (3):** **B4-G25** merchant float
  parsing (int_provider truncates — mechanism refined this run: it
  ZEROES, see 26.1.1-record corrections), **B5-G20** no nether portal
  can ever be created, **B5-G21** snow never accumulates on land / ice
  only on water.
- **26.1.1's new majors (2):** **B5-G22** composter completely dead
  (reads the `minecraft:compostable` component no 26.1.2 item
  carries), **B8-G10** `/spreadplayers` always fails (shared with the
  26.1 tree, missed by the 26.1 run).
- **New for 26.1.2 (1):** **B11-G10** vanilla-written worlds load as
  void (palette reader removed in the delta).

**Oracle majors (23):**

- **Batch 0 groups (9):** O1 LocalPlayer ctor (20 oracles), O2 latent
  jsr305 javac crash, O3 `fuelValues()` + missing overrides, O4
  `densityfunction` absent (17 oracles), O5 EntityTypes holder move
  (31 oracles), O6 other 26.3-only packages (renderpearl 8,
  material/rule 5, advancements 3, authlib 1, lwjgl.sdl 1), O7 headline (202 fail /
  231 drivers blocked), O8 runtime layer unmeasured (partially
  measured now via `command_parse`), O9 re-ported 26.3-form oracle
  sources (17 files fail new or worse vs 26.1).
- **B2-O01** BlockUpdate (32 errors), **B2-O03** FaceSupport (1 error,
  19 drivers), **B4-O06** Dispenser (EntityTypes, blocks 6 drivers),
  **B4-O23** NaturalSpawn (16 errors).
- **B6-O01–O07** — 39 worldgen oracles in 7 groups (densityfunction
  9, material/rule 5, Feature ctor 3, missing symbols 12, lookup
  overrides 2, no-override 4, other 4). StructureDensity unrepairable
  by design.
- **B10-O03** render/model family (12 oracles, FirstPerson grew to 20
  errors), **B10-O04** sound family (8 own-error failures + 2
  cascades, 11 drivers blocked), **B10-O05** UI family (renderpearl +
  Hud absent).

Cheapest high-leverage repairs: B3-G22's untagged suffocation form
(26.2's `motion.rs` is the template), the B6 prepare folder names
(three lines in `WORLDGEN_BUNDLES`), the B12 loot dual-form reads
(26.2's `items.rs` is the template), the 26.1 tree as the repair
template for the twelve regressions (its furnace/brewing/tool-
transform/command adaptations are the reference — the 26.1.2 port
needs to be re-derived from the 26.1 tree, not re-ported from 26.3),
`ParityFont` (1 error, blocks 8 drivers), B9-O01 Scoreboard (3
errors, passed in 26.2). New this run: **B11-G10** is a one-file
restore (`from_palette_nbt` + its 3 call sites, all present in the
26.1.1 tree) and **B11-G11** is a restore of 26.1.1's
`chat_disabled_or_blocked` fix + test. The probe already compiles, so
`command_parse` is available to verify any command fix.

## Strongest verifications

- **Commands (B9):** the run — 34,542 commands × 3 permission levels
  = 103,626 parse comparisons, 97,315 match, 6,311 mismatches (3,162
  unique inputs; perm 4: 3,162, perm 2: 3,148, perm 0: 1). The corpus
  is md5-identical to the 26.1.1 corpus (`aa28f318…`); the per-perm
  outputs are byte-identical to the 26.1.1 run's by `cmp`. The node
  diff confirms G16 (34 / 29 / 0 inputs; 6 of its 9 label pairs
  confirmed empirically). The recipe-suggestion artifact prediction
  (G11) confirmed exactly (6 mismatches, oracle suggests `["*"]`
  only). The error-text composition verified from the raw jsonl:
  2,815 SlotSource (G12) + 38 codec-shape (G07a) + 3 place + 1
  publish + 1 swing = 2,858 at perm 4; the 22 ok-differs / 44
  cursor-differs / 2,858 error-text-only split reproduced exactly by
  an independent classifier. The run's only raw key is swing's
  (`argument.swing_animation.invalid`, 4 emissions, 2 unique inputs).
  G19/G20/G21 (minted at merge in 26.1.1) re-verified fresh against
  the 26.1.2 run — no merge minting needed this time. The run
  contradicted NO static verdict.
- **Registries (B12):** all 5,309 data-driven files byte-identical to
  the 26.1.2 jar; loot census identical to 26.1.1's (1,545 `condition`
  occurrences, 0 `type`-keyed; 1,326 loot tables, 134 pass / 1,192 gap
  under items.rs). `loot_fill` was NOT run (both loot files
  `cmp`-identical to 26.1.1's, per the brief's byte-identical-code
  condition).
- **Worldgen (B6):** reference-world census (7,074 chunks,
  695,402,496 blocks, 18 biomes, seed 4656297851632877335 confirmed
  from `world_gen_settings.dat`); the construction cascade verified
  link by link against code + prepared data + jar (the coordinator
  confirmed the empty bundles directly: carvers/features/feature_tags
  all 0 entries); snow-on-packed-ice 0 world-wide confirms the G02
  fix.
- **Entities (B3):** the handleAttack spectator early-return check
  PASSES (ported in `lan/hub.rs:4063-4065, 4083-4087` with a test) —
  PORT-NOTES line 71 is stale. G47 is the only new defect: the AI
  walk-target value misses the Zero arms and the Giant/Mooshroom arms.
- **Items (B7):** all 1,506 items mapped, 0 unmapped; all 30 in-scope
  files and all 66 in-scope vanilla classes byte-identical to 26.1.1's.
  Zero status diffs across all 49 items.
- **Rank-3 (B11):** class-table chain-liveness run (263/271 jar-live,
  263/271 chain-live, 0 jar-live-but-chain-dead — identical to
  26.1.1; the 271 denominator confirmed from the `classes!` macro);
  7/7 ledger items pass; G10 verified end-to-end by the coordinator
  (reader removed + save.rs maps unresolvable entries to air +
  reference world uses `Name`/`Properties` palettes with zero strict
  `id` entries).
- **Fluids (B1):** waterlogging census 410/410, fire odds 207/207
  exact; 15 of 17 26.1.1 items reproduce, 2 land closed again.
- **Particles (B8):** `ParticleTypes.<clinit>` parsed fresh from the
  26.1.2 jar (117 registrations, 28 limiter-true, 0 id mismatches vs
  prepared == datagen registries; elder_guardian id 17); all 40
  particle-relevant vanilla classes byte-identical between the jars.
- **Block behaviour (B5):** the G18 correction verified by the
  coordinator against the Vineflower decompile (`!(clickY > 0.8124F)`)
  and the port code (`click_y <= 0.8124f32`, same constant).
- **Rendering (B10):** the G04 correction verified by the coordinator
  (six geometry functions at their exact cited lines, live call sites
  in `game.rs`, all six files byte-identical to 26.1.1's); the
  Checkbox tooltip (#6) verified passing (`CHECKBOX_ROWS = 2` +
  `checkbox_overflows_row_limit`, exact vanilla semantics).
- **Oracle measurement (B0):** 391 files, 202 fail-all (51.7%), 231
  drivers blocked; zero deviations vs the 26.1.1 table; per-batch
  O-items name 162 distinct failing files (156 Vanilla*Oracle + 6
  harness/client files).

## Needs-harness (7)

B2-H01/H02 (redstone signal nibbles, circuit equivalence; the 4
level::tests need a cargo run — the prepare precondition is now
satisfied, Batch 12 having built the prepare), B6-H (paired
port-vs-vanilla chunk comparison at seed 4656297851632877335, STAGE
RULE — blocked until the construction cascade is fixed), B8-H01–H04
(particle driver gaps). B11-H01 is WITHDRAWN this run (the lib-test
breakage was fixed). Only `command_parse` was run end-to-end; Batch
9's 18 other e2e drivers are unrun (command_exec fixture blocker).

## Dedup record

31 whole-item folds (duplicate removed from the merged list, kept in
its slice; canonical item carries a merge note) — the 26.1.1 record,
re-verified at merge time against the 26.1.2 slices (every participant
exists, every pair is live, and the duplicate/canonical evidence was
compared fresh for the splash-water 3-way, the spawner 3-way, the
composter and the dripstone 3-way): B11-G02 → B2-G01, B11-G03 →
B5-G03, B11-G04 → B1-G01, B5-G13 → B1-G01, B1-G03 → B11-G05, B7-G10 →
B11-G05, B7-G11 → B5-G19, B4-G18 → B5-G19, B5-O01 → B1-O02, B11-O01 →
B1-O02, B5-O05 → B1-O03, B11-O02 → B1-O07, B7-O07 → B1-O07, B3-O08 →
B1-O06, B5-O06 → B4-O06, B7-O01 → B4-O06, B7-O02 → B2-O01, B5-O09 →
B2-O03, B7-O09 → B10-O03, B5-O07 → B10-O03, B5-O08 → B10-O03, B4-O16 →
B10-O03, B4-O18 → B10-O05, B4-O08 → B10-O05, B7-O10 → B4-O21, B4-G20 →
B7-G17, B3-G39 → B7-G17 (spawner 3-way), B3-G24 → B4-G24, B4-G26 →
B12-G01, B4-G30 → B5-G22 (composter), B4-G35 → B7-G18 (dispenser
spawn egg). 22 per-oracle member folds inside group items (same set as
26.1.1: B6-O04 Sculk → B4-O14, B6-O06 NaturalSpawn → B4-O23,
B6-O06 FinalizeSpawn → B3-O03, B6-O06 SpawnRules → B3-O03, B6-O01
WorldSpawn → B3-O05, B3-O03 NaturalSpawn → B4-O23, B3-O05
RegisteredEntityFluid → B1-O01, LivingFluid → B1-O05, Trading →
B4-O25, VehicleContainer → B4-O21, PlayerInventory → B7-O27,
KnockbackClient → B7-O11, LocatorBarClient → B9-O03, B3-O03
Arrow/BoatRider/ClientArrow/BlockingItemDamage → B7 singletons,
B3-O07 HelmetDamage → B7-O30, B3-O04 BoatAuthority → B7-O08,
ShulkerSpace → B4-O22, B3-O02 EntitySoundPacket → B10-O04, B10-O03
BlockEntityAnim → B4-O15). Full record in the merged ledger's `dedup`
block, including the Batch 0 group ↔ per-batch overlap map.

Two worker fold suggestions were declined in favour of the 26.1.1
canonicals: Batch 7 suggested B7-G10 → B1-G03 (the splash-water 3-way
canonical is B11-G05, 1.21.7 lineage — both B1-G03 and B7-G10 fold
there) and B7-G17 → B3-G39 (backwards — B3-G39 and B4-G20 fold into
B7-G17).

Deliberately NOT folded (kept as separate items, as in 26.1.1):
B4-O19/B7-O29 EnchantDurability, B4-G37/B6-G07 sculk inhibitors (Batch
4 judged a different code path), the beds facets
(B10-G03/B11-G01/B4-G13/B5-G27 — different facets), B4-O09/O10
RecipeBook/PlaceRecipe cascades.

## Coordinator corrections (during reconciliation)

- **B5-G18 and B10-G04 never existed.** Both corrections were
  verified independently by the coordinator before landing (see
  Strongest verifications). They land CLOSED, not fixed — and they
  correct the 26.1 and 26.1.1 ledgers, which carry both as live
  gap/minor items.
- **B11-G10 verified end-to-end by the coordinator** (reader removed
  + save.rs air-mapping + reference-world palette form — see
  Strongest verifications). The void consequence is stated as
  derived-statically, not demonstrated, in the report and the slice.
- **B9 — no merge minting needed.** G19/G20/G21 (minted at merge in
  26.1.1) were re-verified fresh by the worker against the 26.1.2
  run. The run was verified exactly from the raw artifacts (per-perm
  table, mismatch line counts, node diffs, the 22/44/2,858 split
  reproduced by an independent classifier, corpus md5). One claim
  could NOT be re-checked independently: the worker's byte-identity
  assertion for the jsonl outputs vs the 26.1.1 run (the 26.1.1 jsonl
  files are not in the 26.1.1 batches dir) — accepted on the strength
  of every independently checkable number matching exactly. The
  raw-key count (4 emissions) was already corrected in the worker's
  report.
- **B7 fold directions corrected** (see Dedup record).
- **B3-G39's "missed by the 26.1 sweep" claim is wrong** — Batch 7
  established in 26.1.1 that 26.1 B7-G17 is exactly the regular
  spawner arm; it was NOT missed. Not repeated in this run.
- **Tally:** B4's 37 driver-status checks, B3's 3 driver checks and
  B9's 4 oracle/harness checks were kept as-tallied per their 26.1.1
  precedents, with the behaviour-only restatement computed in
  `merge.py` (44 checks removed — identical to 26.1.1).
- **Batch 0's prepared-data section is stale** (see The story) — the
  summary corrects it.
- **B0/B1/B2/B4/B5/B6/B7/B8/B10/B11/B12:** no other corrections
  needed — all headline claims verified exactly from raw artifacts
  (B0's measured table; B1's censuses; B4's tally table recomputed;
  B5's portal/snow/composter mechanisms; B6's world census and
  cascade; B7's item map; B8's particle ids and tree diff; B10's
  driver table recomputed; B11's chain-liveness recomputed; B12's
  loot census and 26.1.1 code comparison re-verified).

## 26.1.1-record corrections (from this run)

These concern the recorded 26.1.1 ledger on main, not the 26.1.2
ledger. The two NEVER EXISTED items head the list; the rest are
wording, counting and mechanism errors found while re-verifying fresh:

- **B5-G18 (bell `isProperHit`) never existed.** The 26.2 run misread
  the `ifle` branch direction; the misread propagated through the
  26.1 and 26.1.1 runs. Vanilla requires `click_y <= 0.8124` — exactly
  what the port implements. Corrects the 26.1 ledger too.
- **B10-G04 (seven BE renderers) never existed.** The 26.1.1 run
  searched only the `Kind` enum; the `game.rs` geometry equivalents
  exist in all three trees. Corrects the 26.1 ledger too (nuance:
  26.1's `spawner.rs` differs from 26.1.2's by 3 lines — the
  setEntityData arm — but `spawner_geometry` exists in 26.1 at
  `spawner.rs:505`, and the other five files are byte-identical).
- **B4:** G25's mechanism is ZEROING, not truncation (`as_i64()`
  returns None for float JSON, `unwrap_or(0)` zeroes every float
  count); the "68 trade-set files with float amount" sub-claim does
  NOT reproduce (387 trade files, one trade each, 0 `amount` keys);
  G22 omits the `isExplorationMap` exclusion entirely (not a wrong
  predicate); G33 narrowed to LAN only (26.1.2's protocol layer drops
  synthetic sign-text components at wire encode; local pick/place
  still preserves text/wax); the 26.1.1 "10 behaviour REPRODUCES"
  for 1.21.7 was a miscount of its own list of 11.
- **B6:** `minecraft:y` is y_clamped_gradient from_y −4064 / to_y
  4062 (not −64/320); there are 6 `spring_feature` configured
  features, not 7; biome carver lists read `[cave,
  cave_extra_underground, canyon]` in data order.
- **B7:** the §1 item→class table sums to 1,530 over 57 rows, not
  1,506 over 70 classes, and lists classes the jar never constructs
  (swords/spears/armour are plain `Item` with components in 26.x);
  "187 item tag files" is a miscount — 186 .json files (the 187th
  entry is the `enchantable/` subdirectory); `javap_dumps.sh` lacked
  the `PlayerSpawnFinder` / `TeleportRandomlyConsumeEffect` dumps its
  report cites (correct packages:
  `net.minecraft.server.level.PlayerSpawnFinder`,
  `net.minecraft.world.item.consume_effects.TeleportRandomlyConsumeEffect`).
- **B9:** the dedicated list had `kick` in / `transfer` out —
  backwards (`KickCommand.register` at `Commands.<init>` offset 216,
  unconditional/integrated; `TransferCommand.register` at offset 752,
  inside `includeDedicated`; the oracle's INTEGRATED tree export
  confirms). Several 26.1.1 line numbers were off by 1–2 (G12, G18,
  G10); the 26.1.2 report cites corrected numbers.
- **B10:** `Audio::play_on` is NOT "used only by /playsound" — it has
  four game-code callers (portal travel, bubble pop, projectiles,
  ominous spawner warning); the 26.1.1 §5 blocked table dropped the
  `entity_sound_packet` row (the 26.1.2 §6 has all 31 rows); Kind arms
  cover 16 slots, not 15.

## What the 1.21.10 run needs

1. **Check the tree's provenance FIRST.** This run's lesson, again:
   provenance determines everything. 26.1.2 was a delta port of
   26.1.1, so the sweep was a re-verification; 1.21.10 is a
   template-era version — establish whether its tree derives from the
   1.21.11 template, the 26.3 base, or another route, and brief the
   re-check-first list accordingly.
2. **Carry the two NEVER EXISTED corrections — but check fresh.**
   B5-G18 and B10-G04 must NOT be re-minted for 1.21.10 without fresh
   bytecode/geometry evidence against 1.21.10's own jar/tree. The
   1.21.7 tree is different; its B10-G04 is NOT corrected by this
   run, and 1.21.10's bell/renderers need their own verdicts.
3. **Check the palette reader.** The 26.1.2 delta removed
   `from_palette_nbt` from `chunk.rs` (B11-G10); 1.21.10 predates the
   26.x `id`-form change, so vanilla 1.21.10 writes
   `Name`/`Properties` palettes — check whether 1.21.10's chunk.rs
   has the legacy reader, and mint if it doesn't.
4. **The B8 parser gotcha:** the particle limiter boolean must be
   read as the `iconst` AFTER the name `ldc` — a backwards walk picks
   up the previous registration's boolean.
5. **The probe compiles — run `command_parse`.** H01 is fixed for
   26.1.1 and 26.1.2; check 1.21.10's probe the same way before
   briefing Batch 9, then run the driver. Budget for the build (the
   26.1.2 build took 56m56s from a warm cache under a competing
   build; the build is the long pole, not the run). The corpus
   (34,542 commands, md5 `aa28f318…`) and analysis scripts are
   preserved in the 26.1.2 `b09/` dir for the rerun — but 1.21.10's
   command tree differs, so regenerate the corpus from its own tree
   export; do NOT reuse the 26.x corpus.
6. **Establish the holder shape by jar listing.** 1.21.10 predates
   the EntityTypes move (1.21.x shape expected) — verify by jar
   listing before briefing any batch.
7. **Name the tally precedents in the brief.** B4/B3/B9 each needed
   a restatement despite the convention being in `common.md` and
   `merge.py`. Say explicitly: oracle/harness/driver-status checks
   are not tallied, and the restatement is the coordinator's job.
8. **`coordinator_note` is the coordinator's.** Say so in the brief.
9. **Brief oracle names, not short names.** Full `Vanilla*Oracle`
   names everywhere; keep driver/transcript names
   (EntityTranscript, RideTranscript) out of oracle counts.
10. **Keep analysis artifacts under `~/workspace/mcir-work/`, never
    `/tmp`.** B9's corpus, run logs and node-diff scripts are
    preserved in the 26.1.2 `b09/` dir for the rerun.
11. **Verify report completeness before accepting a batch** (read the
    tail; grep for truncation markers). Batch 9's build was
    interrupted mid-flight by a service restart and resumed cleanly
    from its on-disk state — because the worker's deliverables and
    run state were preserved in the batch scratch dir.
12. **The lib-test breakage is fixed in this line.** 1.21.10's Batch
    11 should check whether its regression tests run rather than
    assuming the breakage — the 26.1.2 run's B11-H01 withdrawal shows
    the assumption is now stale.
13. **The NEVER EXISTED corrections should reach the 26.1 and 26.1.1
    ledgers** before anyone "fixes" a bell or a renderer that was
    never broken — a parent action, not a sweep action; flagged here
    so it is not lost.
