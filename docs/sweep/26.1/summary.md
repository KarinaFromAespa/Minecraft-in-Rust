# 26.1 parity sweep — summary

**Version:** 26.1 · **Date:** 2026-11-24 · **Branch:** `parity-sweep-26.1`
(synced to main `40738825`) · **Ledger:** `pins/26.1/sweep-items.json`
(merged from the 13 per-batch slices in `pins/26.1/sweep/batch-NN.json`).
Per-batch reports live in this directory (`batch-N-*.md`); the universe
inventory is `inventory.md` / `inventory-26.1.json`.

Checks + triage only. No fixes were made in any batch; no production code
under `versions/26.1/` was touched. Every `fix` field in the merged ledger
is null.

## Headline

| | Total | Pass | Gap | Accepted-gap |
|---|---:|---:|---:|---:|
| Checks as tallied | 10,016 | 8,502 | 1,514 | 0 |
| Checks, behaviour only | 9,972 | 8,484 | 1,488 | 0 |

The two rows differ by 44 non-behaviour checks three batches count in
their as-tallied tallies, each per a 26.2 precedent the coordinator
accepted: Batch 4 counts 37 driver-status checks (18 pass / 19 gap),
Batch 3 counts 3 driver-code-defect checks (0 pass / 3 gap), Batch 9
counts 3 oracle items + 1 harness blocker as gap checks (0 pass / 4
gap). No other batch counts oracle-status checks. Quote the
behaviour-only row when comparing against other versions' behaviour
tallies.

**Ledger items:** 229 raw across the slices, **201 unique** after dedup
(28 whole-item duplicates folded — see Dedup below). **186 live** (115
gap + 64 oracle-defect + 7 needs-harness), **50 live major**. Non-live:
2 fixed (1.21.7 defects withdrawn for 26.1), 10 closed (1.21.7 defects
re-checked and closed), 3 accepted-gap.

**Comparison with the 26.2 sweep** (10,374 checks, 10,058 pass, 314 gap,
113 items, 20 live major) **and the 1.21.7 sweep** (10,341 checks, 7,730
pass, 2,609 gap, 208 items, 35 major): 26.1 ran a similar number of
checks but found nearly 5× 26.2's gaps (1,514 vs 314) and 2.5× its live
majors (50 vs 20). It sits between the two runs — far cleaner than
1.21.7, far dirtier than 26.2. The reason is structural, not luck: see
below.

## Per-batch tallies

| Batch | Scope | Checks | Pass | Gap | Items |
|---|---|---:|---:|---:|---:|
| 0 | Oracle measurement | 391 | 199 | 192 | 8 |
| 1 | Fluids + falling blocks | 93 | 78 | 15 | 11 |
| 2 | Redstone + pistons | 52 | 46 | 6 | 14 |
| 3 | Entities / mobs | 80 | 67 | 13 | 23 |
| 4 | Block entities + containers | 92 | 54 | 38 | 38 |
| 5 | Block behaviour | 127 | 111 | 16 | 14 |
| 6 | Worldgen / features | 25 | 18 | 7 | 22 |
| 7 | Items | 42 | 29 | 13 | 42 |
| 8 | Particles | 272 | 266 | 6 | 10 |
| 9 | Commands | 23 | 11 | 12 | 22 |
| 10 | Rendering, sound, UI | 12 | 8 | 4 | 8 |
| 11 | Rank-3 triage (214 items) | 214 | 214 | 0 (+3 accepted) | 13 |
| 12 | Data-driven registries | 8,593 | 7,401 | 1,192 | 4 |
| **Total** | | **10,016** | **8,502** | **1,514** | **229 raw / 201 unique** |

## The story: why 26.1 is this dirty

The 26.2 sweep was clean because 26.2 IS a 26.x version in the modern
shape: its class names, tags, data formats and APIs are the ones the
port was written against, and the 26.2 tree had been adapted version by
version (playtest fixes, loot dual-form, motion untagged form, probe
adaptation). **26.1 is an early 26.x version, and its tree was ported
from 26.3 with almost none of those adaptations carried:**

- **26.1 predates the `EntityTypes`/`BlockEntityTypes` holder move.**
  Registration still lives on `EntityType`/`BlockEntityType` — the
  1.21.x shape. The port's oracles and code carry the 26.2+ holder form:
  Batch 0's new group G4 (31 oracles), Dispenser, BoatAuthority,
  ItemModel, and a dozen more. This run's key discovery, found when the
  26.2 extractor reported 0s for all registration facts until given an
  old-shape fallback.
- **The 26.2 fixes were never carried.** Batch after batch found the
  same pattern: the 26.2 tree has the adapted form, the 26.1 tree still
  carries 26.3's. The loot engines read `type`/`modifier` while 26.1
  data writes `condition`/`function` (26.2's playtest fix); motion
  requires the `#causes_suffocation` / `#suppresses_bounce` tags 26.1
  data lacks (26.2 uses the untagged form, with a documenting comment);
  prepare maps the 26.3 worldgen folder names (`worldgen/carver`,
  `worldgen/feature`) while the 26.1 jar uses `configured_carver` /
  `configured_feature`; the NoiseRouter is the 26.3 eight-field form;
  aquifers read the 26.3 config key; `material_rule` is required where
  26.1 has inline `surface_rule`; the density decoder lacks the cache
  arms; 11 feature types lack decode arms; `/perf` and six dev commands
  are missing; item modifiers, particle `Name` compounds, the selector
  parser and recipe suggestions all carry the 26.2 shape against 26.1
  data.
- **The 1.21.7 systemic causes apply here in muted form.** 26.1 is
  26.x, but early: 16 of the 28 live behaviour majors are 1.21.7
  defects reproducing (dripstone, redstone wire, beds, splash water,
  sculk, loot, chorus fruit, signs, sounds, cartography), where 26.2
  carried only 1. The 26.1 class table carries NO dual names at all
  (26.2's carries Path/RedstoneWire duals) — why four 1.21.7 renames
  reproduce here that 26.2 fixed.
- **Two startup blockers.** B3-G22: the client cannot load its core
  tables with prepared 26.1 data (motion requires two absent tags).
  B6's construction cascade: no 26.1 world can generate at all
  (`GeneratorData::load` fails on the empty carver bundle, then
  `Pipeline::new` fails on the router, density caches, material rule
  and spawn target, in that order). Both need fix decisions, not just
  ledger entries — and both have the 26.2 tree as the repair template.
- **Live behaviour gaps: 115** (87 minor + 28 major). The minors are
  single-site slips of the same 26.3-form kind: tag spellings, missing
  food tags, inverted comparisons, off-by-ones, stale 26.3 constants in
  drivers (35723 state counts, `horizon-client/26.3` paths — a
  sweep-wide pattern: 34 further drivers carry the 35723 hardcode, 28
  further the 26.3 path).
- **Live oracle defects: 64 items naming 155 distinct failing files**
  (plus Batch 0's group items covering the other 37 failing files).
  192 of 391 oracles fail javac on all three classpaths (49.1%, vs
  26.2's 129) — the EntityTypes move alone accounts for 31. No oracle
  was repaired in this sweep (Batch 0 = measure + triage only, per the
  brief's scope limit).
- **No UNVERIFIED carry-overs.** Every 1.21.7 item was decided against
  26.1 evidence. Notably B6-G03 (underground water deficit), carried
  UNVERIFIED in 26.2, is DEMONSTRATED here via the total aquifer
  absence (B6-G09).

## Live majors (50)

**Behaviour majors (28):**

- **Startup blockers (2):** **B3-G22** — default suffocation tag-gated
  on absent `#causes_suffocation` (first failure `#suppresses_bounce`);
  the client cannot load its core tables. **B6-G06** — prepared
  carvers/features bundles empty; `GeneratorData::load` fails, no world
  can generate.
- **Worldgen cascade (6):** **B6-G09** aquifers entirely disabled,
  **B6-G10** NoiseRouter is the 26.3 eight-field form, **B6-G11**
  requires `material_rule` (26.1 has inline `surface_rule`), **B6-G12**
  spawn-target decode fails, **B6-G13** density decode missing cache /
  shifted-noise arms, **B6-G14** 11 feature types + coral mushroom +
  nether carver lack decode arms (22 configured features).
- **1.21.7 carry-overs (16):** **B1-G01** entire dripstone system dead
  (3 dead causes), **B2-G01** redstone wire never updates power,
  **B2-G02** push-reaction loader dead, **B2-G03** obsidian pushable,
  **B4-G16** cartography paper zoom dead, **B6-G01** sculk
  catalyst/shrieker tail missing, **B6-G03** underground water deficit
  (demonstrated), **B7-G09** chorus fruit never teleports, **B9-G02**
  `/posteffect` remains in the release tree (partial), **B10-G01** sign
  boards not rendered, **B10-G02** no `sounds.json` (game silent; worse
  than 26.2 — all 6 audio drivers oracle-blocked too), **B10-G03** beds
  invisible when placed, **B11-G01** beds place foot-only / sleep
  unreachable, **B11-G05** splash water does nothing, **B12-G01**
  items.rs evaluates every loot condition as false and never runs any
  function (1,192 tables), **B12-G02** loot.rs ignores all conditions
  and functions (demonstrated live by `loot_fill`).
- **New (4):** **B4-G25** villager/wandering-trader trading yields NO
  offers at all (26.1 trade data writes whole numbers as floats; port
  reads as_i64), **B5-G20** no nether portal can ever be created,
  **B5-G21** snow never accumulates on land / ice only on water,
  **B9-H01** `client_probe` does not compile (two lines set 26.3-only
  `FrameContext` fields) — blocks the `command_parse` run and any other
  `client_probe`-based driver.

**Oracle majors (22):**

- **Batch 0 groups (8):** O1 LocalPlayer ctor (20 oracles), O2 latent
  jsr305 javac crash, O3 `fuelValues()` + missing overrides, O4
  `densityfunction` absent (17 oracles), O5 EntityTypes holder move
  (31 oracles — new for 26.1), O6 other 26.3-only packages
  (renderpearl 8, material/rule 5, advancements 3, authlib 1), O7
  headline (192 fail / 221 drivers blocked), O8 runtime layer
  unmeasured.
- **B2-O01** BlockUpdate (32 errors), **B2-O03** FaceSupport (1 error,
  19 drivers), **B4-O06** Dispenser (EntityTypes, blocks 6 drivers),
  **B4-O23** NaturalSpawn (16 errors).
- **B6-O01–O07** — 34 worldgen oracles in 7 groups (densityfunction
  10, material/rule 5, Feature ctor 1, missing symbols 8, lookup
  overrides 2, no-override 4, other 4). StructureDensity unrepairable
  by design.
- **B10-O03** render/model family (12 oracles), **B10-O04** sound
  family (SoundEventOracle fails — new vs 26.2), **B10-O05** UI family
  (renderpearl + Hud absent).

Cheapest high-leverage repairs: the H01 two-line probe fix (unblocks
`command_parse` + every `client_probe` driver), B3-G22's untagged
suffocation form (26.2's `motion.rs` is the template), the B6 prepare
folder names (three lines in `WORLDGEN_BUNDLES`), the B12 loot
dual-form reads (26.2's `items.rs` is the template), `ParityFont` (1
error, blocks 8 drivers), B9-O01 Scoreboard (3 errors, passed in 26.2).

## Strongest verifications

- **Registries (B12):** all 5,309 data-driven files byte-identical to
  the 26.1 jar; loot census by recursive walk (1,545 conditions, 1,404
  functions, 1,389 plural-conditions arrays, 983 `functions`
  containers, 0 `modifier` — cross-checks B4-G26 exactly); `loot_fill`
  run live (7 tables × 5 seeds, exit 0; gated lava-chicken disc drops
  on all 5 seeds); effects 40/40 jar match; triggers 53/54 fired.
- **Worldgen (B6):** 715/715 populated registry entries identical,
  1,202/1,202 structure NBTs byte-identical, flat presets 9/9; vanilla
  26.1 world censused (8,138 chunks, 799,997,952 blocks, seed
  −1600712240839185950 confirmed); the construction cascade verified
  link by link against code + prepared data + jar.
- **Entities (B3):** complete class-file scan over all 708 entity
  classes for 14 declarer sets; `EntityType.<clinit>` parsed for all
  157 types; `Attributes.<clinit>` 35/35; 82 spawn-placement
  registrations with 38 predicates resolved via BootstrapMethods.
- **Commands (B9):** corpus preserved (34,542 commands, md5-verified);
  G16's 7 node-name pairs verified from both sides (Rust literals +
  datagen `commands.json`); the RECIPE suggestion special case verified
  via javap. **No run exists** — H01 blocks `command_parse` (contrast
  26.2's 103,743 parses, 0 mismatches).
- **Items (B7):** all 1,506 items mapped to 70 classes, 0 unmapped;
  driver inventory recomputed exactly (74 drivers).
- **Rank-3 (B11):** class-table chain-liveness run for the first time
  in the programme (263/271 jar-live, 263/271 chain-live, 0
  jar-live-but-chain-dead) — **closes 26.2's B11-H01 for 26.1**.
- **Fluids (B1):** waterlogging census 410/410, fire odds 207/207
  exact; `doFireTick` verified independently NOT APPLICABLE (59
  gamerules, no doFireTick).
- **Particles (B8):** particle ids confirmed from prepared
  `registries.json` (elder_guardian 17, flash 42, vibration 48).

## Needs-harness (7)

B2-H01/H02 (redstone signal nibbles, circuit equivalence), B6-H
(paired port-vs-vanilla chunk comparison at seed −1600712240839185950,
STAGE RULE — blocked until the construction cascade is fixed),
B8-H01–H04 (particle driver gaps). Only `loot_fill` was run end-to-end;
Batch 9's 19 e2e drivers are unrun (command_parse blocked by H01, the
rest recorded in B9's `out_of_scope_failures`).

## Dedup record

28 whole-item folds (duplicate removed from the merged list, kept in
its slice; canonical item carries a merge note): B11-G02 → B2-G01,
B11-G03 → B5-G03, B11-G04 → B1-G01, B5-G13 → B1-G01, B1-G03 → B11-G05,
B7-G10 → B11-G05, B7-G11 → B5-G19, B4-G18 → B5-G19, B5-O01 → B1-O02,
B11-O01 → B1-O02, B5-O05 → B1-O03, B11-O02 → B1-O07, B7-O07 → B1-O07,
B3-O08 → B1-O06, B5-O06 → B4-O06, B7-O01 → B4-O06, B7-O02 → B2-O01,
B5-O09 → B2-O03, B7-O09 → B10-O03, B5-O07 → B10-O03, B5-O08 → B10-O03,
B4-O16 → B10-O03, B4-O18 → B10-O05, B4-O08 → B10-O05, B7-O10 → B4-O21,
B4-G20 → B7-G17, B3-G24 → B4-G24, B4-G26 → B12-G01. 21 per-oracle member
folds inside group items (B6-O04 Sculk → B4-O14, B6-O06 NaturalSpawn →
B4-O23, B6-O06 FinalizeSpawn → B3-O03, B6-O01 WorldSpawn → B3-O05,
B3-O03 NaturalSpawn → B4-O23, B3-O05 RegisteredEntityFluid → B1-O01,
LivingFluid → B1-O05, Trading → B4-O25, VehicleContainer → B4-O21,
PlayerInventory → B7-O27, KnockbackClient → B7-O11, LocatorBarClient →
B9-O03, B3-O03 Arrow/BoatRider/ClientArrow/BlockingItemDamage → B7
singletons, B3-O07 HelmetDamage → B7-O30, B3-O04 BoatAuthority →
B7-O08, ShulkerSpace → B4-O22, B3-O02 EntitySoundPacket → B10-O04,
B10-O03 BlockEntityAnim → B4-O15). Full record in the merged ledger's
`dedup` block, including the Batch 0 group ↔ per-batch overlap map.

## Coordinator corrections (during reconciliation)

- **Tally:** B4's 37 driver-status checks, B3's 3 driver checks and
  B9's 4 oracle/harness checks were kept as-tallied per their 26.2
  precedents, with the behaviour-only restatement computed in
  `merge.py` (44 checks removed). The merge script's tally text was
  corrected (it claimed behaviour-only "by construction") and its merge
  notes fixed ("same oracle defect" → "same defect", "later batch" →
  "another batch" — several folds run from an earlier batch into a
  later canonical item).
- **B3:** the agent's "26.2 record needs flags" claim was wrong for
  G05 — 26.2's B3 DID defect G05 (in the 26.2 ledger, as are G06/G09/
  G11). Genuine 26.2 flags: G27/G28 (food table — `food_tag()` verified
  byte-identical 26.1 vs 26.2), G25, G26. G16's clearance cross-noted
  against B4-G25 (loader reads the dirs; amounts still fail to parse).
- **B6:** the agent wrote its own `coordinator_note` (replaced);
  `decode-audit.json`'s missing-feature list omits seagrass/
  weeping_vines (the report's G14 table is authoritative); B6's O-items
  use short oracle names (map 1:1 to `Vanilla*Oracle`).
- **B9:** G16's prose undercounts ("five label pairs, the same five as
  26.2") — the table has 7 pairs and 26.2's G16 had four. H01 verified
  end-to-end (probe byte-identical to 26.3's; 26.2's adapted; E0560 × 2
  in the driver log).
- **B1/B2/B5/B7/B8/B10/B11/B12:** no corrections needed — all headline
  claims verified exactly from raw artifacts (B1's dripstone third
  cause in `block-behaviours.json`; B5's portal/snow/mud mechanisms;
  B7's item map and driver inventory recomputed; B8's particle ids from
  prepared registries; B10's driver table recomputed; B11's
  class-table sweep recomputed; B12's loot census, loot_fill log and
  26.2 code comparison all re-verified).

## 26.2-record corrections (from this run)

These concern the recorded 26.2 ledger on main, not the 26.1 ledger:

- **B3-G27/G28/G25/G26** (zombie/skeleton horse food, furnace minecart
  cap, BreedGoal panic) are shared verbatim with the 26.2 port, which
  passed those sections — the 26.2 sweep missed them.
- **B9-G11:** real vanilla 26.1 DOES suggest recipes (RECIPE special
  case in `CommandSourceStack`, javap-verified); the 26.2 run's G11
  closure missed this. Future recipe-suggestion mismatches in a rerun
  are an oracle artifact, not a port defect.
- **B6:** 26.2's prepare still has the `feature_tags` folder-name gap
  (latent there), and `dripstone_cluster` is missing in both trees
  (both carry the 26.3 name `speleothem_cluster`).

## What the next version's run needs (26.1.1)

1. **Adapt the probe first.** 26.1.1 carries the same unadapted
   `game_loop_probe.rs`; adapt it per version (as 26.2's and 1.21.11's
   were) or Batch 9 blocks again on H01. The two-line fix is the
   cheapest unblock in the programme.
2. **Establish the holder shape FIRST, by jar listing.** This run's key
   discovery (26.1 predates the EntityTypes move) came only when the
   extractor reported 0s. 26.1.1's shape is unknown — check
   `EntityTypes`/`BlockEntityTypes` in the jar before briefing any
   batch, and parameterize the extractor for the old shape if needed.
3. **Expect the 26.1 shape, not the 26.2 shape.** 26.1.1 is 26.1's
   neighbour: the cascade defects (B6-G06/G09/G10/G11/G12/G13/G14),
   the loot engines, the suffocation gate and the probe are all
   re-check-first candidates. Check the tree's provenance first — if
   the 26.1.1 port derives from the 26.1 tree they carry; if from
   26.2's, they may not.
4. **Brief oracle names, not short names.** B6's short oracle names
   needed a mapping pass in the merge; brief batches to use full
   `Vanilla*Oracle` names, and to keep driver/transcript names
   (EntityTranscript, RideTranscript) out of oracle counts.
5. **`coordinator_note` is the coordinator's.** Two batches pre-filled
   it; say so in the brief.
6. **Name the tally precedents in the brief.** B4/B3/B9 each needed a
   restatement despite the convention being in `common.md` and
   `merge.py` — the 26.2 precedents pulled the other way. Say
   explicitly: oracle/harness/driver-status checks are not tallied,
   and the restatement is the coordinator's job.
7. **Budget for VM reboots on the `command_parse` build.** B9 survived
   two reboots from the warm cargo cache but lost ~2h30m wall-time;
   the build is the long pole, not the run. Keep the corpus + scripts
   in the batch scratch dir for the rerun.
8. **Keep analysis artifacts under `~/workspace/mcir-work/`, never
   `/tmp`.** B9's corpus (34,542 commands, md5-verified) and node-diff
   scripts are preserved in `b09/` for the rerun after H01 lands.
9. **Reuse the template:** `docs/sweep/26.1/extract-inventory.py` is
   parameterized (inventory first, from the version's OWN sources);
   Batch 0 lands before the batches and the measured table is the
   oracle authority; verify report completeness before accepting a
   batch (read the tail; grep for truncation markers).
