# 26.1.1 parity sweep — summary

**Version:** 26.1.1 · **Date:** 2026-11-25 · **Branch:** `parity-sweep-26.1.1`
(synced to main `e105c215`) · **Ledger:** `pins/26.1.1/sweep-items.json`
(merged from the 13 per-batch slices in `pins/26.1.1/sweep/batch-NN.json`).
Per-batch reports live in this directory (`batch-N-*.md`); the universe
inventory is `inventory.md` / `inventory-26.1.1.json`.

Checks + triage only. No fixes were made in any batch; no production code
under `versions/26.1.1/` was touched. Every `fix` field in the merged ledger
is null. Batches ran 2026-11-24; Batch 9's follow-up run and the merge
landed 2026-11-25 (the sweep paused 21:02–23:46 NPT Nov 24 on the owner's
order, to free VM resources for the fresh-tester fix).

## Headline

| | Total | Pass | Gap | Accepted-gap |
|---|---:|---:|---:|---:|
| Checks as tallied | 9,851 | 8,254 | 1,596 | 0 |
| Checks, behaviour only | 9,807 | 8,236 | 1,570 | 0 |

The two rows differ by 44 non-behaviour checks three batches count in
their as-tallied tallies, each per a 26.1 precedent the coordinator
accepted: Batch 4 counts 37 driver-status checks (17 pass / 20 gap),
Batch 3 counts 3 driver-code-defect checks (0 pass / 3 gap), Batch 9
counts 3 oracle items + 1 harness blocker as checks (1 pass / 3 gap:
H01 + O01/O02/O03). No other batch counts oracle-status checks. Quote
the behaviour-only row when comparing against other versions' behaviour
tallies.

**Ledger items:** 331 raw across the slices, **300 unique** after dedup
(31 whole-item duplicates folded — see Dedup below). **262 live** (188
gap + 66 oracle-defect + 8 needs-harness), **60 live major**. Non-live:
9 fixed (26.1 defects withdrawn for 26.1.1), 26 closed (26.1 defects
re-checked and closed), 3 accepted-gap.

**Comparison with the 26.1 sweep** (10,016 checks, 8,502 pass, 1,514
gap, 229 items, 186 live, 50 live major), **the 26.2 sweep** (10,374
checks, 10,058 pass, 314 gap, 113 items, 20 live major) **and the
1.21.7 sweep** (10,341 checks, 7,730 pass, 2,609 gap, 208 items, 35
major): 26.1.1 ran fewer checks than 26.1 (9,851 vs 10,016 — Batch 11
counted 6 checks against 26.1's 214, 26.1.1's rank-3 ledger being 6
items, not 214; partly offset by Batch 10's 23 vs 12 and Batch 3's 96
vs 80) but found MORE gaps (1,596 vs 1,514), MORE live items (262 vs
186) and MORE live majors than any previous sweep (60 vs 26.1's 50,
1.21.7's 35, 26.2's 20). It is the dirtiest 26.x sweep yet — dirtier
than 26.1, its own neighbour, one patch version away. The reason is
provenance, not luck: see below.

## Per-batch tallies

| Batch | Scope | Checks | Pass | Gap | Items |
|---|---|---:|---:|---:|---:|
| 0 | Oracle measurement | 391 | 189 | 202 | 9 |
| 1 | Fluids + falling blocks | 93 | 73 | 20 | 17 |
| 2 | Redstone + pistons | 52 | 45 | 7 | 17 |
| 3 | Entities / mobs | 96 | 62 | 34 | 45 |
| 4 | Block entities + containers | 95 | 47 | 48 | 62 |
| 5 | Block behaviour | 128 | 102 | 26 | 29 |
| 6 | Worldgen / features | 27 | 19 | 8 | 25 |
| 7 | Items | 44 | 28 | 16 | 49 |
| 8 | Particles | 279 | 267 | 12 | 16 |
| 9 | Commands | 24 | 7 | 17 | 26 |
| 10 | Rendering, sound, UI | 23 | 9 | 14 | 17 |
| 11 | Rank-3 triage (6 items) | 6 | 5 | 0 (+1 needs-harness) | 15 |
| 12 | Data-driven registries | 8,593 | 7,401 | 1,192 | 4 |
| **Total** | | **9,851** | **8,254** | **1,596** | **331 raw / 300 unique** |

## The story: why 26.1.1 is dirtier than 26.1

The 26.1 sweep's story was that 26.1's tree was ported from 26.3 with
almost none of the 26.2 tree's adaptations carried. **26.1.1 is worse:
its tree was re-ported from 26.3 a second time, instead of deriving
from the 26.1 tree — so the 26.1 tree's adaptations were never
carried, and defects 26.1 had closed came back:**

- **Twelve regressions of 26.1 closures.** Six behaviour defects 26.1
  had closed, withdrawn or fixed are live again here: **B1-G05** no
  fluid spreading at all (the worst 1.21.7 defect — closed in 26.1),
  **B3-G14** endermen never teleport (26.1 had cleared it; the 26.1
  tree used `blocksMotion`), **B4-G01** furnaces never light and
  **B4-G02** brewing stands never brew (both regressed from 26.1
  FIXED), **B7-G07** tool transforms and **B7-G08** campfire douse
  (both withdrawn in 26.1). Five 26.1 command closures flip back to
  gaps: **B9-G03** (`/place feature` now `ResourceOrId`), **B9-G05**
  (`/publish` lost its gamemode argument), **B9-G07b** (multi-entity
  item source), **B9-G12** (`SlotSource` back in the trees),
  **B9-G13** (the 62-entry 26.3 argument-type table). And **B9-G02**
  widens from one extra top-level command to three (`posteffect`,
  `unpublish`, `compute`), with **B9-G18** recording three 26.3-era
  subcommands (`/item fill`, `/item override`, `/execute if|unless
  slots`) vanilla 26.1.1 never had.
- **The 26.1 cascade defects reproduce wholesale.** B6-G06 (empty
  carver/feature bundles — no world can generate), B6-G09–G14 (the
  worldgen cascade: aquifers, NoiseRouter, material rule, spawn
  target, density caches, feature decode arms), B3-G22 (suffocation
  gate), the B12 loot engines — plus two NEW worldgen majors
  (**B6-G15** state-provider short names, 110 of 221 configured
  features fail; **B6-G16** `random_offset` has no decode arm, 70
  placed features fail) and a new composter major (**B5-G22**,
  completely dead).
- **The oracle sources were re-ported too.** Batch 0's new group O9:
  17 oracle files fail new or worse vs 26.1. The fail-all count ROSE
  192 → 202 (+10) despite 26.1.1 being the later version — the
  26.1.1 oracle sources carry 26.3-form code the 26.1 sources had
  already been adapted away from.
- **The 1.21.7 systemic causes apply in full.** The same sixteen
  1.21.7 carry-over majors as 26.1 (dripstone, redstone wire, beds,
  splash water, sculk, loot, chorus fruit, signs, sounds,
  cartography) PLUS six more 1.21.7 defects 26.1 had withdrawn and
  26.1.1 re-minted (B1-G05, B3-G14, B4-G01, B4-G02, B7-G07, B7-G08).
  26.1.1 predates the `EntityTypes`/`BlockEntityTypes` holder move
  exactly as 26.1 does (registration on `EntityType`/`BlockEntityType`,
  the 1.21.x shape) — O5's 31 oracles reproduce.
- **Two startup blockers** (same as 26.1). B3-G22: the client cannot
  load its core tables with prepared 26.1.1 data (motion requires two
  absent tags). B6's construction cascade: no 26.1.1 world can
  generate at all. Both need fix decisions, not just ledger entries —
  and both have the 26.2 tree as the repair template.
- **Live behaviour gaps: 169** (132 minor + 37 major). The minors are
  single-site slips of the same 26.3-form kind: tag spellings, missing
  food tags, inverted comparisons, off-by-ones, stale 26.3 constants
  in drivers.
- **Live oracle defects: 66 items naming 162 distinct failing files**
  (plus Batch 0's group items covering the other 40 failing files).
  202 of 391 oracles fail javac on all three classpaths (51.7%, vs
  26.1's 192) — the EntityTypes move alone accounts for 31. No oracle
  was repaired in this sweep (Batch 0 = measure + triage only, per the
  brief's scope limit).
- **The one genuine improvement over 26.1: H01.** The probe compiles
  (26.1.1's `FrameContext` keeps the 26.3 form, so the two probe lines
  are valid initialisers), so `command_parse` RAN end-to-end — the
  first full probe build on this VM and the only end-to-end driver run
  in this sweep (see Strongest verifications).
- **No UNVERIFIED carry-overs.** Every 1.21.7 item was decided against
  26.1.1 evidence. Notably B6-G03 (underground water deficit),
  carried UNVERIFIED in 26.2, is DEMONSTRATED here via the total
  aquifer absence (B6-G09), as in 26.1.

## Live majors (60)

**Behaviour majors (37):**

- **Startup blockers (2):** **B3-G22** — default suffocation tag-gated
  on absent `#causes_suffocation` (first failure `#suppresses_bounce`);
  the client cannot load its core tables. **B6-G06** — prepared
  carvers/features bundles empty; `GeneratorData::load` fails, no world
  can generate.
- **Worldgen cascade (8):** **B6-G09** aquifers entirely disabled,
  **B6-G10** NoiseRouter is the 26.3 eight-field form, **B6-G11**
  requires `material_rule` (26.1.1 has inline `surface_rule`),
  **B6-G12** spawn-target decode fails, **B6-G13** density decode
  missing cache / shifted-noise arms, **B6-G14** 11 feature types +
  coral mushroom + nether carver lack decode arms (22 configured
  features), **B6-G15** state-provider decode accepts only the 26.3
  short names (110 of 221 configured features fail — NEW), **B6-G16**
  `random_offset` placement modifier has no decode arm (70 placed
  features fail — NEW).
- **1.21.7 carry-overs (16 — the same sixteen as 26.1):** **B1-G01**
  entire dripstone system dead (3 dead causes), **B2-G01** redstone
  wire never updates power, **B2-G02** push-reaction loader dead,
  **B2-G03** obsidian pushable, **B4-G16** cartography paper zoom dead,
  **B6-G01** sculk catalyst/shrieker tail missing, **B6-G03**
  underground water deficit (demonstrated), **B7-G09** chorus fruit
  never teleports, **B9-G02** `/posteffect`/`/unpublish`/`/compute`
  remain in the release tree (wider than 26.1), **B10-G01** sign boards
  not rendered, **B10-G02** no `sounds.json` (game silent; all 6 audio
  drivers oracle-blocked too), **B10-G03** beds invisible when placed
  AND bed items now fail to decode (worse than 26.1), **B11-G01** beds
  place foot-only / sleep unreachable, **B11-G05** splash water does
  nothing, **B12-G01** items.rs evaluates every loot condition as
  false and never runs any function (1,192 tables), **B12-G02**
  loot.rs ignores all conditions and functions.
- **Regressions from 26.1 closures (6):** **B1-G05** NO fluid
  spreading at all (was CLOSED in 26.1), **B3-G14** endermen never
  teleport (26.1 had cleared it), **B4-G01** furnaces never light and
  **B4-G02** brewing stands never brew (both regressed from 26.1
  FIXED), **B7-G07** axe/hoe/shovel transforms and honeycomb wax-on
  entirely non-functional (26.1 had withdrawn it), **B7-G08** shovel
  campfire douse dead (26.1 had withdrawn it).
- **Reproduces from 26.1's new majors (3):** **B4-G25** merchant float
  parsing (int_provider truncates), **B5-G20** no nether portal can
  ever be created, **B5-G21** snow never accumulates on land / ice
  only on water.
- **New for 26.1.1 (2):** **B5-G22** composter completely dead (reads
  the `minecraft:compostable` component no 26.1.1 item carries),
  **B8-G10** `/spreadplayers` always fails (shared with the 26.1 tree,
  missed by the 26.1 run).

**Oracle majors (23):**

- **Batch 0 groups (9):** O1 LocalPlayer ctor (20 oracles), O2 latent
  jsr305 javac crash, O3 `fuelValues()` + missing overrides, O4
  `densityfunction` absent (17 oracles), O5 EntityTypes holder move
  (31 oracles), O6 other 26.3-only packages (renderpearl 8,
  material/rule 5, advancements 3, authlib 1, lwjgl.sdl 1), O7 headline (202 fail /
  231 drivers blocked), O8 runtime layer unmeasured (partially
  measured now via `command_parse`), O9 re-ported 26.3-form oracle
  sources (17 files fail new or worse vs 26.1 — NEW for 26.1.1).
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
transform/command adaptations are the reference — the 26.1.1 port
needs to be re-derived from the 26.1 tree, not re-ported from 26.3),
`ParityFont` (1 error, blocks 8 drivers), B9-O01 Scoreboard (3
errors, passed in 26.2). The probe already compiles, so `command_parse`
is available to verify any command fix.

## Strongest verifications

- **Commands (B9):** the run — 34,542 commands × 3 permission levels
  = 103,626 parse comparisons, 97,315 match, 6,311 mismatches (3,162
  unique inputs; perm 4: 3,162, perm 2: 3,148, perm 0: 1). The node
  diff confirms G16 (34 / 29 / 0 inputs; 6 of its 9 label pairs
  confirmed empirically, the other 3 standing on the static proof).
  The recipe-suggestion artifact prediction (G11) confirmed exactly
  (6 mismatches, oracle suggests `["*"]` only). The error-text
  composition verified from the raw jsonl: 2,815 SlotSource (G12) +
  38 codec-shape (G07a) + 3 place + 1 publish + 1 swing = 2,858 at
  perm 4. The run's only raw key is swing's
  (`argument.swing_animation.invalid`, 4 emissions, 2 unique inputs).
  The run minted three new items (B9-G19/G20/G21 — see Coordinator
  corrections) and contradicted NO static verdict.
- **Registries (B12):** all 5,309 data-driven files byte-identical to
  the 26.1.1 jar; loot census by recursive walk (1,545 `condition`
  occurrences, 0 `type`-keyed; 983 `functions` containers, 0
  `modifier` — cross-checks B4-G26 exactly); 1,326 loot tables
  (134 pass / 1,192 gap under items.rs); recipes 1,515, advancements
  1,617. `loot_fill` was NOT run for 26.1.1 (the 26.1 live run covers
  the identical code shape).
- **Worldgen (B6):** reference-world census (8,692 chunks, 12,192,372
  vanilla water blocks, seed 3231603902411641524 confirmed); the
  construction cascade verified link by link against code + prepared
  data + jar; 12 of 15 gaps are 26.1 reproductions, 3 NEW (G15, G16,
  G17).
- **Entities (B3):** complete class-file scan; `EntityType.<clinit>`
  parsed for all 157 types; `Attributes.<clinit>`; 82 spawn-placement
  registrations with 38 predicates resolved via BootstrapMethods.
- **Items (B7):** all 1,506 items mapped to 70 classes, 0 unmapped, 0
  mismatches vs `block-behaviours.json` — identical to 26.1; spears
  (7) and nautilus armour (5) make B33/B34 full checks, not
  not-covered.
- **Rank-3 (B11):** class-table chain-liveness run (263/271 jar-live,
  263/271 chain-live, 0 jar-live-but-chain-dead) — **26.2's B11-H01
  chain-liveness half stays CLOSED for 26.1.1**.
- **Fluids (B1):** waterlogging census 410/410, fire odds 207/207
  exact; total states 29,873 confirmed.
- **Particles (B8):** 28 particle-relevant classes md5-compared
  between the 26.1 and 26.1.1 jars, all byte-identical; registry
  census 117 types, ids contiguous 0–116 (elder_guardian 17, flash
  42, vibration 48); full tree diff against versions/26.1/ for every
  particle file (surfaced B8-G05–G09).
- **Oracle measurement (B0):** 391 files, 202 fail-all (51.7%), 231
  drivers blocked; per-batch O-items name 162 distinct failing files
  (156 Vanilla*Oracle + 6 harness/client files).

## Needs-harness (8)

B2-H01/H02 (redstone signal nibbles, circuit equivalence; the 4
level::tests need a cargo run), B6-H (paired port-vs-vanilla chunk
comparison at seed 3231603902411641524, STAGE RULE — blocked until the
construction cascade is fixed), B8-H01–H04 (particle driver gaps),
B11-H01 (item 5's regression test cannot run until the horizon-client
lib-test breakage is fixed — the id is reused; 26.2's B11-H01
chain-liveness half stays CLOSED). Only `command_parse`
was run end-to-end; Batch 9's 18 other e2e drivers are unrun
(command_exec fixture blocker).

## Dedup record

31 whole-item folds (duplicate removed from the merged list, kept in
its slice; canonical item carries a merge note): B11-G02 → B2-G01,
B11-G03 → B5-G03, B11-G04 → B1-G01, B5-G13 → B1-G01, B1-G03 → B11-G05,
B7-G10 → B11-G05, B7-G11 → B5-G19, B4-G18 → B5-G19, B5-O01 → B1-O02,
B11-O01 → B1-O02, B5-O05 → B1-O03, B11-O02 → B1-O07, B7-O07 → B1-O07,
B3-O08 → B1-O06, B5-O06 → B4-O06, B7-O01 → B4-O06, B7-O02 → B2-O01,
B5-O09 → B2-O03, B7-O09 → B10-O03, B5-O07 → B10-O03, B5-O08 → B10-O03,
B4-O16 → B10-O03, B4-O18 → B10-O05, B4-O08 → B10-O05, B7-O10 → B4-O21,
B4-G20 → B7-G17, B3-G39 → B7-G17 (spawner 3-way — NEW vs 26.1),
B3-G24 → B4-G24, B4-G26 → B12-G01, B4-G30 → B5-G22 (composter — NEW
vs 26.1), B4-G35 → B7-G18 (dispenser spawn egg — NEW vs 26.1). 22
per-oracle member folds inside group items (B6-O04 Sculk → B4-O14,
B6-O06 NaturalSpawn → B4-O23, B6-O06 FinalizeSpawn → B3-O03,
B6-O06 SpawnRules → B3-O03 — NEW vs 26.1, B6-O01 WorldSpawn → B3-O05,
B3-O03 NaturalSpawn → B4-O23, B3-O05 RegisteredEntityFluid → B1-O01,
LivingFluid → B1-O05, Trading → B4-O25, VehicleContainer → B4-O21,
PlayerInventory → B7-O27, KnockbackClient → B7-O11, LocatorBarClient →
B9-O03, B3-O03 Arrow/BoatRider/ClientArrow/BlockingItemDamage → B7
singletons, B3-O07 HelmetDamage → B7-O30, B3-O04 BoatAuthority →
B7-O08, ShulkerSpace → B4-O22, B3-O02 EntitySoundPacket → B10-O04,
B10-O03 BlockEntityAnim → B4-O15). Full record in the merged ledger's
`dedup` block, including the Batch 0 group ↔ per-batch overlap map.

Deliberately NOT folded (kept as separate items, as in 26.1 where
applicable): B4-O19/B7-O29 EnchantDurability (26.1 kept both),
B4-G37/B6-G07 sculk inhibitors (Batch 4 judged a different code path;
26.1 had only B6-G07), the beds facets (B10-G03/B11-G01/B4-G13/B5-G27
— different facets; 26.1 kept them), B4-O09/O10 RecipeBook/PlaceRecipe
cascades (26.1 kept them).

## Coordinator corrections (during reconciliation)

- **B9 — three items minted at merge.** The `command_parse` run's
  §2.5 observations had no static item; the coordinator minted
  **B9-G19** (`/swing` accepts 26.3-era animation + duration
  arguments; 8 mismatches, verified against `tree_world.rs:176-192`
  and vanilla's `tree.json`), **B9-G20** (`/schedule clear` takes a
  single id where vanilla takes a greedy string; 4 mismatches,
  verified against `tree_misc.rs:49` and `tree.json`), **B9-G21**
  (scoreboard criterion suggestions include one criterion vanilla
  26.1.1 lacks; 6 mismatches, verified from the raw jsonl: Rust
  9,133 vs vanilla 9,132, single extra
  `minecraft.custom:minecraft.sleep_in_straw_bed`). All three minor.
  The worker's suggestion to fold G19 into G18 was declined: G18 is
  subcommand-level, G19 is argument-level, and G19 carries the run's
  only raw-key errors. Batch 9's slice went from 23 to 26 items; the
  24-check tally is unchanged (the findings fall under the existing
  command_parse coverage, per Batch 3's convention that one check
  yields one item per defect).
- **B9 — raw-key count corrected.** The worker's report said "only 2
  raw-key emissions in the whole run"; the raw jsonl shows 2 unique
  inputs × 2 perms = **4 emissions**. Corrected in three places
  (§2.5, §2.6, §4 O04). Everything else in §2 verified exactly from
  the raw artifacts (per-perm table, by-command composition,
  SlotSource 2,815 / codec 38 split, node-diff pairs, scoreboard
  diff, recipe artifact, zero oracle crashes).
- **B9 — the 26.1-summary G11 correction was already applied** in the
  batch report (the RECIPE special case; report line 433). The
  follow-up re-verification's remaining work was the run itself.
- **B3-G39's "missed by the 26.1 sweep" claim is wrong.** Batch 7
  established that 26.1 B7-G17 is exactly the regular spawner arm —
  it was NOT missed. (The trial arm IS new in 26.1.1.) Not a
  26.1-record correction; recorded here so the claim is not repeated.
- **Tally:** B4's 37 driver-status checks, B3's 3 driver checks and
  B9's 4 oracle/harness checks were kept as-tallied per their 26.1
  precedents, with the behaviour-only restatement computed in
  `merge.py` (44 checks removed).
- **B0/B1/B2/B4/B5/B6/B7/B8/B10/B11/B12:** no corrections needed — all
  headline claims verified exactly from raw artifacts (B0's measured
  table; B1's censuses; B4's tally table recomputed; B5's
  portal/snow/composter mechanisms; B6's world census and cascade;
  B7's item map; B8's particle ids and tree diff; B10's driver table
  recomputed; B11's chain-liveness recomputed; B12's loot census and
  26.1 code comparison re-verified).

## 26.1-record corrections (from this run)

These concern the recorded 26.1 ledger on main, not the 26.1.1 ledger:

- **B3-G33 (visibility):** vanilla `getVisibilityPercent` hardcodes
  five head/type pairs INCLUDING `PIGLIN_BRUTE + PIGLIN_HEAD` — the
  26.1 report's "piglin brutes are not fooled" claim is wrong
  (`LivingEntity` is byte-identical between 26.1 and 26.1.1, so it was
  wrong for 26.1 too); the method applies no `0..=10` clamp — the
  26.1 report's clamp claim is wrong too. Both recorded in the
  26.1.1 B3-G33 evidence.
- **B8:** B8-H02 undercounts — the providers driver sends 11
  non-26.1 kinds, not 5 (the geyser/noxious sections were missed);
  the oracle-API check "Villager.handleEntityEvent" is misattributed
  — the method is declared on `LivingEntity` (inherited by Villager);
  the check still passes.
- **Missed by the 26.1 sweep** (shared verbatim with the 26.1 tree):
  **B8-G10** `/spreadplayers` always fails (major), **B10-G15** sound
  categories come from the event name, not the call site (minor),
  **B3-G34** trader/villager hurt gating, brain half (minor — the
  trader half is NEW in 26.1.1).

## What the next version's run needs (26.1.2)

1. **Check the tree's provenance FIRST.** This run's lesson is that
   provenance determines everything: 26.1.1 was re-ported from 26.3
   instead of deriving from the 26.1 tree, and twelve 26.1 closures
   came back with it. Before briefing any batch, establish whether
   the 26.1.2 tree derives from the 26.1.1 tree, the 26.1 tree, or a
   fresh 26.3 re-port — and brief the re-check-first list accordingly.
2. **Re-check the twelve regressions first.** B1-G05, B3-G14, B4-G01,
   B4-G02, B7-G07, B7-G08, B9-G03/G05/G07b/G12/G13 — if 26.1.2 is
   another re-port, these are the first places it shows.
3. **Do NOT re-port oracle sources from 26.3.** O9's lesson: 17 files
   fail new or worse vs 26.1 because the 26.1.1 oracle sources were
   re-ported. Carry the 26.1 tree's adapted oracle sources forward.
4. **Establish the holder shape by jar listing.** 26.1.1 predates the
   EntityTypes move like 26.1; 26.1.2's shape is unknown — check
   `EntityTypes`/`BlockEntityTypes` in the jar before briefing any
   batch, and parameterize the extractor for the old shape if needed.
5. **The probe compiles — run `command_parse`.** H01 is fixed for
   26.1.1 (26.1.1's `FrameContext` keeps the 26.3 form); check 26.1.2's
   probe the same way, then run the driver. Budget for the build
   (21m from a warm cache, longer cold; the build is the long pole,
   not the run). The corpus (34,542 commands, md5-verified) and
   analysis scripts are preserved in `b09/` for the rerun.
6. **Name the tally precedents in the brief.** B4/B3/B9 each needed a
   restatement despite the convention being in `common.md` and
   `merge.py` — the 26.1 precedents pulled the other way. Say
   explicitly: oracle/harness/driver-status checks are not tallied,
   and the restatement is the coordinator's job.
7. **`coordinator_note` is the coordinator's.** Say so in the brief.
8. **Brief oracle names, not short names.** B6's short oracle names
   needed a mapping pass in the merge; brief batches to use full
   `Vanilla*Oracle` names, and to keep driver/transcript names
   (EntityTranscript, RideTranscript) out of oracle counts.
9. **Keep analysis artifacts under `~/workspace/mcir-work/`, never
   `/tmp`.** B9's corpus, run logs and node-diff scripts are preserved
   in `b09/` for the rerun.
10. **Verify report completeness before accepting a batch** (read the
    tail; grep for truncation markers). Batch 9's follow-up was
    interrupted mid-flight and resumed cleanly from its on-disk
    deliverables — because the worker's report and slice were complete
    and the run state was preserved in the batch scratch dir.
