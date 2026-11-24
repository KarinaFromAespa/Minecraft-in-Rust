# 26.2 parity sweep — summary

**Version:** 26.2 · **Date:** 2026-11-24 · **Branch:** `parity-sweep-26.2`
(synced to main `732c800f`) · **Ledger:** `pins/26.2/sweep-items.json`
(merged from the 13 per-batch slices in `pins/26.2/sweep/batch-NN.json`).
Per-batch reports live in this directory (`batch-N-*.md`); the universe
inventory is `inventory.md` / `inventory-26.2.json`.

Checks + triage only. No fixes were made in any batch; no production code
under `versions/26.2/` was touched. Every `fix` field in the merged ledger
is null.

## Headline

| | Total | Pass | Gap | Accepted-gap |
|---|---:|---:|---:|---:|
| Checks as tallied | 10,374 | 10,058 | 314 | 2 |
| Checks, behaviour only | 10,320 | 10,030 | 288 | 2 |

The two rows differ by Batch 6's 54 oracle-status checks (28 pass / 26
fail). Batch 6 counts oracle status in its tally, exactly as the 1.21.7
Batch 6 did (69 = 22 behaviour + 47 oracle); the other twelve batches do
not. Quote the behaviour-only row when comparing against other versions'
behaviour tallies.

**Ledger items:** 119 raw across the slices, **113 unique** after dedup
(6 whole-item duplicates folded — see Dedup below). **93 live** (62 gap +
22 oracle-defect + 9 needs-harness), **20 live major**. Non-live: 3 fixed
(1.21.7 defects withdrawn for 26.2), 15 closed (1.21.7 defects re-checked
and closed), 2 accepted-gap.

**Comparison with the 1.21.7 sweep** (10,341 checks, 7,730 pass, 2,609
gap, 208 items, 68 major): 26.2 ran a similar number of checks but found
an order of magnitude fewer gaps (314 vs 2,609) and a third as many majors
(25 vs 68, 20 live vs 68). The reason is structural, not luck: see below.

## Per-batch tallies

| Batch | Scope | Checks | Pass | Gap | Items |
|---|---|---:|---:|---:|---:|
| 0 | Oracle measurement | 391 | 262 | 129 | 8 |
| 1 | Fluids + falling blocks | 110 | 108 | 2 | 6 |
| 2 | Redstone + pistons | 45 | 42 | 3 | 9 |
| 3 | Entities / mobs | 78 | 72 | 6 | 11 |
| 4 | Block entities + containers | 91 | 74 | 17 | 15 |
| 5 | Block behaviour | 123 | 119 | 4 | 8 |
| 6 | Worldgen / features | 79 | 51 | 28 | 12 |
| 7 | Items | 42 | 35 | 7 | 7 |
| 8 | Particles | 283 | 278 | 5 | 8 |
| 9 | Commands | 21 | 16 | 5 | 20 |
| 10 | Rendering, sound, UI | 17 | 11 | 6 | 6 |
| 11 | Rank-3 triage (177 items) | 177 | 175 | 0 (+2 accepted) | 7 |
| 12 | Data-driven registries | 8,917 | 8,815 | 102 | 2 |
| **Total** | | **10,374** | **10,058** | **314** | **119 raw / 113 unique** |

## The story: why 26.2 is this clean

The 1.21.7 sweep's gaps were dominated by two systemic causes: (1) the
port carries 26.x class names and tag-based logic for tags that do not
exist in 1.21.7, so each silently degraded; (2) the 1.21.7 tree predated
several fixes. **Both evaporate for 26.2 because 26.2 IS a 26.x version:**
its class names, tags, data formats and APIs are the ones the port was
written against. Batch after batch reports the same pattern — the
1.21.7 defect either never existed in 26.2 (vanilla 26.2 uses the 26.x
form), was already fixed in the 26.2 tree, or was a genuine
1.21.7↔26.2 vanilla difference. What remains is small, specific, and
mostly minor:

- **Live behaviour gaps: 42** (39 minor + 3 major). The minors are
  single-site slips: a missing `isSource()` check, off-by-ones at the
  build ceiling, an inverted comparison, a wrong JSON level, a missing
  food tag, stale 26.3 constants in drivers (35723 state counts,
  `horizon-client/26.3` paths — a sweep-wide pattern: 25+ further
  drivers carry the 35723 hardcode, ~20+ further the 26.3 path).
- **Live oracle defects: 22 items naming 84 distinct failing oracles**
  (plus Batch 0's group items covering the other 45 failing oracles).
  Every oracle failure is 26.3-form code against 26.2's API — missing
  packages (`densityfunction`, `renderpearl`, `material/rule`),
  constructor/signature drift (`LocalPlayer`, `ChunkMap`, `Feature`,
  `onEquippedItemBroken`), missing symbols, missing overrides
  (`fuelValues()`). No oracle was repaired in this sweep (Batch 0 =
  measure + triage only, per the brief's scope limit).
- **The 1.21.7 majors are almost all gone.** Of the 1.21.7 sweep's
  majors re-checked here, only **B11-G01 (beds place foot-only; sleep
  unreachable)** carries over as a demonstrated 26.2 defect — it is a
  template-level `AbstractBed`/`BedBlock` dispatch failure, same as in
  1.21.7. B6-G03 (underground water deficit) is carried over
  **unverified** — see below.

## Live majors (20)

**Behaviour majors (3):**

- **B11-G01** — beds place foot-only; sleep unreachable (`AbstractBed`
  dead in 26.2; all bed arms dispatch on it). Template-level, same as
  1.21.7. Demonstrated.
- **B10-G02** — no `sounds.json` in the prepared 26.2 resources: the
  game is silent; 6 audio drivers data-gated. Needs the `horizon-prepare
  sounds` step (not a code defect).
- **B6-G03** — underground water deficit (−1.04% in 1.21.7), carried
  over **UNVERIFIED**. The full aquifer audit found NO divergence
  against the 26.2 jar and the aquifer code is effectively identical
  between the trees. No 26.2 evidence exists; B6-H (paired comparison)
  arbitrates. Do not present as a demonstrated 26.2 defect.

**Oracle majors (17):**

- **Batch 0 groups (7):** O1 LocalPlayer ctor (17 oracles), O2 latent
  jsr305 javac crash, O3 `fuelValues()` + missing overrides, O4
  `densityfunction` absent (9 oracles), O5 other 26.3-only packages
  (renderpearl 7, material/rule 3, authlib 1, lwjgl/sdl 1), O6 headline
  (129 fail / 147 drivers blocked), O8 runtime layer unmeasured.
- **B4-O06** Dispenser (1e, blocks 6 drivers sweep-wide), **B4-O08**
  Menu (1e, blocks 4 drivers).
- **B6-O01–O06** — 26 worldgen oracles in 6 groups (densityfunction 8,
  material/rule 3, Feature ctor 1, missing symbols 9, lookup overrides
  2, other 3). StructureDensity unrepairable by design.
- **B10-O03** BlockEntityRender needs `UvMapping` (26.3-only),
  **B10-O05** UI/screen oracle family needs `renderpearl` (6 drivers
  blocked).

Cheapest high-leverage repairs: `ParityFont` (1 error, blocks 8 drivers),
the five 1-error oracles in Batch 4's scope, B4-O08 (Menu), B9-O02
(TeamRules, 2 errors, fully scoped).

## Strongest verifications

- **Commands (B9):** `command_parse` run end-to-end — **0 mismatches in
  103,743 parses** (34,581 commands × perm 4/2/0), parse success, error
  text, cursor and suggestions all exact. The run's only finding is
  B9-G16 (14 node-label differences, cosmetic). All 13 1.21.7 command
  defects closed.
- **Registries (B12):** all 5,516 data-driven files byte-identical to
  the 26.2 jar; `block-tags.json` 265/265 exact; the 1.21.7 systemic
  loot finding does NOT carry (26.2 writes the old `condition`/`function`
  form; the tree's dual-form loot fix verified working end-to-end).
- **Worldgen (B6):** 952/952 registry entries identical, 1,212/1,212
  structure NBTs byte-identical, vanilla 26.2 world censused (6,018
  chunks, seed confirmed).
- **Entities (B3):** 40/40 attributes, 2,619 attribute values, 83
  placement registers vs 158 entities, 0 mismatches; complete declarer
  sets established by class-file scan over all 716 entity classes.
- **Items (B7):** all 1,537 items mapped to 70 classes, 0 unmapped.

## Needs-harness (9)

B2-H01/H02/H03 (redstone signal nibbles, circuit equivalence, 4
level::tests), B6-H (paired port-vs-vanilla chunk comparison at seed
−1514548643533799951, STAGE RULE), B8-H01–H04 (particle driver gaps:
vibration omitted, 26.3-only poplars included, `zombie_nautilus` /
`camel_husk` omitted), B11-H01 (class-table chain-liveness re-run —
data precondition now met by prepare). B2-H01/H02's data precondition
is also met by prepare; the runs remain follow-up. Only `command_parse`
was run end-to-end; 19 Batch 9 e2e drivers (command_exec linchpin + 17
no-oracle + scoreboard) are unrun, recorded in B9's
`out_of_scope_failures`.

## Dedup record

Whole-item folds (duplicate removed from the merged list, kept in its
slice; canonical item carries a merge note): B4-O16 → B10-O03, B4-O18 →
B10-O05, B5-O01 → B1-O02, B5-O05 → B1-O03, B5-O06 → B4-O06, B9-O03 →
B3-O04. Per-oracle member folds inside group items: B3-O04 loses
RegisteredEntityFluid → B1-O01, LivingFluid → B1-O05, Trading → B4-O25;
B6-O04 loses WorldSpawn → B3-O04, Sculk → B4-O14; B6-O06 loses
FinalizeSpawn → B3-O03. Full record in the merged ledger's `dedup`
block, including the Batch 0 group ↔ per-batch overlap map.

## Coordinator corrections (during reconciliation)

- **B10:** agent triaged oracles by import check only and its report
  ended mid-sentence with a pasted truncation marker. Reinstated B10-O02
  (ParityFont) and B10-O04 (sound oracle family), corrected the driver
  table, completed §4–§7.
- **B1:** agent recorded no oracle defects; added B1-O01–B1-O05 from the
  measured table (all javap-verified 26.3-only API).
- **B5:** draft carried a literal truncation marker at line 561 (the
  agent's repair claim was wrong); reconstructed the lost
  Pilot-leftovers section. Corrected "`GameRules` has 86 rules" to 59
  (javap count; matches the inventory).
- **B8:** added B8-O01 (MarkerParticles, LocalPlayer ctor) from the
  measured table; corrected the driver table and tally.
- **B12:** agent's `condition`-discriminator census undercounted
  (missed `terms`/`term` nesting); recount 1,457 → 1,574.
- **B9:** verified the lang correction — prepared `en_us.json` IS
  present and md5-identical to the client jar's file (8,123 keys). The
  "lang files absent" statements in the Batch 0/3/5/10 notes are wrong;
  only `sounds.json` is genuinely absent (B10-G02 unaffected).
- **B3/B6/B9:** no corrections needed — all oracle claims verified
  exactly (B3: 0 mismatches across 80 checks; B6: 0 across the full
  oracle + driver triage; B9: command_parse run re-verified from its own
  artifacts, including an independent node-diff reproducing B9-G16).

## What the next version's run needs

1. **Reuse the template:** `docs/sweep/26.2/extract-inventory.py` is
   parameterized (inventory first, from the version's OWN sources);
   `docs/sweep/measure-oracles.py` + the Batch 0 method (fixpoint
   big-compiles, sourcepath=False, jsr305 on the client classpath).
   Batch briefs should name the version's own jars explicitly — the
   26.2 run's jar map (named server-game jar vs bundler server jar vs
   game client jar) differs per version and is the first thing to
   establish.
2. **Batch 0 lands before the batches, and the measured table is the
   oracle authority.** Every batch that ran before Batch 0 landed
   needed coordinator repair (B1, B8, B10); every batch briefed on the
   measured table verified clean (B3, B4, B5, B6, B7, B9, B12). Never
   let a batch substitute an import-level static check.
3. **Verify report completeness before accepting a batch.** Two agents
   delivered truncated reports (B10 mid-sentence, B5 with a literal
   truncation marker) while claiming completeness. Read the tail; grep
   for truncation markers.
4. **Settle the tally convention up front.** Batch 6 counts oracle
   status in its tally (per its 1.21.7 template); the other batches do
   not. Either make all batches count oracle status or none — and say
   which in the brief.
5. **Per-batch oracle triage should sweep the full failing list, not
   just in-scope drivers.** 45 of the 129 failing oracles appear only
   in the Batch 0 group items, including VanillaProtocolClient (32
   errors, 8 drivers blocked) and VanillaModelBakingOracle.
6. **Run `command_parse` the same way.** It is the model driver run:
   full corpus from the oracle's own tree export, 3 permission levels,
   node comparison done separately (the driver's `compare()` strips
   nodes — B9-G16 was invisible to it). Budget a ~35 min fast-profile
   lib build at `-j 1` under `taskset`; it survives SIGTERM/reboot from
   the warm cargo cache.
7. **Keep analysis artifacts under `~/workspace/mcir-work/`, never
   `/tmp`.** `/tmp` was wiped mid-run twice (B1, B5, B7 dumps lost;
   findings survived only in run records). The B9 command-parse
   report.json is preserved in the batch scratch dir ahead of `.local`
   cleanup.
8. **Prepare first, then batches.** The 26.2 prepare (34 min) unblocked
   chain-liveness (B5), the item→class map (B7), entity data (B3),
   command registries (B9) and the B12 data diffs. Batches spawned
   before it landed had to be re-notified or re-scoped.
9. **Expect the same shape for 26.x versions, not for 1.21.x.** The
   26.2 run's cleanliness comes from 26.2 being 26.x. A 1.21.x version
   will look like the 1.21.7 run (tag/class drift dominates); an E6
   version needs its own template-first treatment, not the 26.2
   numbers.
