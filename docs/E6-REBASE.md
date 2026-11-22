# E6 Re-base Plan — Propagating the 1.21.11 Behaviour Port Down to 1.21.5–1.21.10

**Status:** Triage complete, 2026-11-22. Propagation not started.
**Branch:** `e6-rebase-triage` (doc-only; no version code touched).
**Data:** [`docs/e6-rebase-triage.json`](e6-rebase-triage.json) — one row per shared change × version, with per-file hunk status and target line evidence.

## Why this exists

Every E6 version tree descends from the 26.3 base, and behaviour code flows down the chain unchanged. The 1.21.11 behaviour port (815 items, branch `behaviour/1.21.11`) fixed 815 divergences from vanilla — but those fixes live only in 1.21.11. They never flowed down.

The water bug proved the cost: `canHoldAnyFluid` used the 26.3 tag form (`#washed_away_by_fluids`), which does not exist in any E6 version's data, so water could not spread at all — and the form was byte-identical in 1.21.5, 1.21.6, 1.21.8, 1.21.9 and 1.21.11. The 1.21.11 port touched `fluids.rs` once in 815 items but not `canHoldAnyFluid`, so nothing caught it. The parity-sweep pilot found four more defects from the same pattern (G1/G3/G5/G6): 26.x tag-based logic for tags absent in the target version.

The owner authorised the re-base on 2026-11-22: 26.3 is no longer the base. Each era's template is built first and the era's versions derive from the completed template. This document is the triage + cost for E6: what propagates from 1.21.11, what does not, and what it costs to carry the shared fixes down to 1.21.5–1.21.10.

## What was triaged

**Source range:** `07e3acb9` (1.21.11 mechanical publish) → `2856b179` (behaviour tip, including the D2 SlotDisplay fix). **Main tip at triage:** `8a3f87b6`.

**The diff:** 1,069 commits in range; 245 touch `crates/` (179 real code, 62 comment-only, 4 test-only). Crates-only diff: 601 files, +18,152/−15,545 lines. Excluded from classification per the brief: the 6 regenerated tables, `pins/`, `work-items.json`, PORT-NOTES, docs.

**Classification units:** 249 — one per item/structural commit (168), plus one per file for the 11 rank-2 A-batch commits (81 file-units), since a batch commit spans many unrelated files.

**Method:** each unit classified from its diff + the item TRIAGE file + PORT-NOTES. Where the classification needed a vanilla fact, it was verified against the vanilla jars (javap against the named 1.21.6–1.21.10 jars; string/tag scans of the obfuscated 1.21.5 jar). The 1.21.11 control jar was used once, to confirm the horse-malus form (item 93). Classifications that could not be verified were marked uncertain, not guessed.

**Applicability:** for each shared unit, the hunk old/new signatures were matched against the target tree on main. `applies` = the pre-change code is present; `already-fixed` = the post-change code is present; `differs` = neither form is present; `uncertain` = a mix of applies + already-fixed hunks (partially applied, needs review). Unit status is the worst file status, so the unit table is conservative — the hunk table is the finer picture.

## Headline numbers

| Classification | Units | Share |
|---|---|---|
| **Shared E6 behaviour — propagates** | **204** | 82% |
| 1.21.11-specific — does not propagate | 34 | 14% |
| Uncertain — needs manual triage | 11 | 4% |
| **Total** | **249** | |

### Per-version applicability (shared units, 204 per version)

Unit-level status is the worst file status in the unit; most "differs" units are mostly applicable (see the hunk table).

| Version | Applies cleanly | Already fixed | Differs | Uncertain (partially applied) |
|---|---|---|---|---|
| 1.21.5 | 135 | 0 | 58 | 11 |
| 1.21.6 | 140 | 0 | 48 | 16 |
| 1.21.7 | 131 | 2 | 57 | 14 |
| 1.21.8 | 124 | 0 | 72 | 8 |
| 1.21.9 | 140 | 0 | 48 | 16 |
| 1.21.10 | 142 | 0 | 46 | 16 |

### Per-version applicability (production hunks — the real workload)

| Version | Applies | Already fixed | Differs | Total |
|---|---|---|---|---|
| 1.21.5 | 1,611 (91%) | 33 | 132 (7%) | 1,776 |
| 1.21.6 | 1,656 (93%) | 29 | 92 (5%) | 1,777 |
| 1.21.7 | 1,630 (92%) | 41 | 108 (6%) | 1,779 |
| 1.21.8 | 1,568 (88%) | 32 | 174 (10%) | 1,774 |
| 1.21.9 | 1,662 (94%) | 29 | 86 (5%) | 1,777 |
| 1.21.10 | 1,666 (94%) | 29 | 82 (5%) | 1,777 |
| **All** | **9,793 (92%)** | **193 (2%)** | **674 (6%)** | **10,660** |

Test hunks: 1,196 applies (88%), 163 differs (12%), 7 already-fixed, of 1,366 total.

Per version, 125–143 units fully apply, 55–66 mostly apply (≥50% of their hunks), and only 3–9 units have zero hunks applying. **The propagation is ~92% mechanical at the hunk level, with a ~6% manual tail.**

## What propagates (204 shared units)

The shared set is the E6-common behaviour the 1.21.11 port restored: 26.x-only mechanics removed (age-lock, WorldClock, per-species baby textures, exclusiveFullscreen), 1.21-era forms restored (classic fluid pushing, motion-based suffocation, list-form survival rules instead of absent `supports_*` tags), and vanilla-accurate entity/mob/AI/render behaviour. The full list with reasons is in the data file.

### Scope caveats (read before propagating)

- **Copper golem (items 86, 109, 446):** the golem exists only in 1.21.9/1.21.10. Items 86 and 109 are classified shared but apply to those two versions only — for 1.21.5–1.21.8 they are N/A (feature absent), not "differs". Item 446 (golem statue textures) is uncertain for the same reason; it propagates only to 1.21.9/1.21.10.
- **Item 16 (block-entity renderers):** the copper-chest Christmas arm only engages in 1.21.9+; inert in 1.21.5–1.21.8. Safe to apply everywhere.
- **Item 751 (villager_trades):** uncertain — the hardcoded-listings mechanism is shared, but the 297 + 94 listing tables are 1.21.11 data. Each version needs its own tables transcribed from its jar.
- **Item 93 (foal scale):** uncertain — the 0.5 foal scale is shared, but the Horse/ZombieHorse −1.0 ctor malus is a 1.21.11 addition (verified against the 1.21.11 control jar; no E6 Horse sets it). **Propagating the malus to E6 would be wrong.**
- **Item 155 (lightmap):** uncertain — the lightmap-shader rewrite is shared, but the `EnvironmentAttributes` removals are 1.21.11-specific (that system was added in 1.21.11; the class is absent from all E6 jars).

### Files that do not exist in the targets

19 file-absent instances across the six versions. The substantive ones:

- **`ice_spike.rs` and `random_patch.rs` (A-batch commit 336b4355):** new files created by the 1.21.11 port (the 26.x config-driven `spike`/`random_patch` forms replaced by 1.21.11's hardcoded feature files). Absent in **all six** targets. Propagation means creating the files **and** wiring them into the module tree (and removing/redirecting the 26.x form). Budget ~1 hour each per version.
- **`bossbars_tests.rs` (item 193):** a new test file, absent in all six targets. Create it during propagation; the production change (in `bossbars.rs` + `game.rs`) applies normally.
- **`dialog_screen.rs` (item 87, 1.21.5 only):** the file does not exist in 1.21.5; the 2-line change needs locating in 1.21.5's dialog code.
- **`game/debug_options.rs` (item 11) and `debug_entries.rs` (item 71), 1.21.8 only:** neither file exists in 1.21.8; the debug-options code lives elsewhere or is absent. Needs a look during propagation.

## What does not propagate (34 specific units)

| Group | Count | Units |
|---|---|---|
| Test-only / comment-only / rename-only (no behaviour change) | 24 | Comment-only (12): `a3a429f4` item_bake/block_entity_nbt/generator/block_entity_item/item_sync/entity_data/vanillagen/item/dialog, `657b693f:compiler.rs`, item 27 (11681eb4), item 119 (9be0f745, NO-CHANGE + citation + test). Test-only (9): `657b693f:tests.rs`, `cfaa397f:block_entity_render_tests.rs`, item 26 (d81843b3), item 199 (cd05db2d, pack-format 94 fixtures), item 204 (8471c286), item 10 (f46c3816, Realms version headers), `a3a429f4:foreign_tests.rs`, `3310abc4:state_provider_bounds.rs`, item 83 (9b92d575). Rename-only (3): `657b693f:goals.rs`, item 292 (16a7dde4), item 503 (bbad0c59). |
| 1.21.11-only features | 5 | Item 74 Timeline system (3516fd34 — `net/minecraft/world/timeline/` absent from the 1.21.7/1.21.10 jars), item 207 spears (df7b8caa), item 258 nautilus (fd624aec), item 126 attack_range component (3e192964 — `ATTACK_RANGE` absent from all E6 jars), item 271 spear sway (9b6629a3 — `iron_spear` absent from all six E6 jars). |
| Numbering / registries / protocol | 4 | D2 SlotDisplay write fix (05350814), item 350 SlotDisplay codec renumbering (7e15d4d2), adapter-ladder re-base to the 774 pin (2e2ef225), packet-set rewire (d8479b17). All per-version numbering; each version has its own. |
| Prepare/oracle tooling | 1 | Completion pass (6ca9c81c) — horizon-prepare only, no version behaviour. |

## What needs manual triage first (11 uncertain units)

These are verified splits, not guesses — each mixes shared and specific parts, or the TRIAGE gives no version boundary. **Triage them (with jar checks) before propagation starts; they are the only blocking analysis work.** ~1 agent-day for all 11.

| Unit | Item | The split |
|---|---|---|
| 2e3435dd | 39 | 104-entry `DATA_COMPONENT_TYPES` list is 1.21.11-specific; the framed-needle owner fallback is shared. Needs splitting. |
| 93827f03 | 15 | Four unrelated sub-changes (prediction teleport removal, squid baby dims, end-flash clock, signed-command chat gate); no version boundary in the diff/TRIAGE. |
| a232303d | 20 | Baby-villager type box at getAgeScale 0.5 + finalizeSpawn biome typing; needs jar checks per version. |
| 413accbc | 35 | Drops `createItemStack` strict validation; TRIAGE compares 26.1 vs 1.21.11 only, so whether 1.21.5–1.21.10 validate is unverified. |
| e7e99fb1 | 102 | Furnace timer reset + canBurn/burn room-for-one; TRIAGE compares 26.1 vs 1.21.11 only. |
| d06ce5b7 | 168 | Shared logic mixed with 1.21.11-specific data: debug-entries registry cut 45 → 39, debug-profile cutoff. |
| 0b6cd98e | 751 | Villager-trades mechanism shared; the 297 + 94 listing tables are 1.21.11 data (see scope caveats). |
| cfaa397f:block_entity_render.rs | 446 | Copper-golem textures: propagates only to 1.21.9/1.21.10. |
| 7d8448ac | 93 | Foal 0.5 scale shared; Horse/ZombieHorse −1.0 malus is 1.21.11-only. Do not propagate the malus. |
| 01a11e2d | 98 | `ObjectContents` absent in 1.21.6–1.21.8, present without a fallback field in 1.21.9/1.21.10 — version split. |
| f44d4ec9 | 155 | Lightmap rewrite shared; `EnvironmentAttributes` removals specific. |

## The trees fix (1.21.7-origin shared fix)

The 1.21.7 trees fix (commit 56e86819, merged 8a3f87b6) is a separate shared fix with a 1.21.7 origin, included in this triage as its own row (`trees-56e86819` in the data file). `synthesize_legacy_support_tags` in `versions/1.21.7/crates/horizon-world/src/worldgen/generator.rs` synthesises the 12 missing `supports_*` tags from 1.21.7 equivalents; without it, `would_survive` failed and trees never decorated (port 978 trees vs vanilla 5,363 at the owner's seed).

| Version | Status |
|---|---|
| 1.21.5 | Applies (3/3 production hunks; test hunk needs adapting to the version's test module) |
| 1.21.6 | Applies (same) |
| 1.21.7 | Already fixed |
| 1.21.8 | Applies (same) |
| 1.21.9 | Applies (same) |
| 1.21.10 | Applies (same) |
| 1.21.11 | Not needed (has the tags; synthesis is a no-op where tags exist) |

**Note:** 1.20.5 and 1.21.4 also lack the tags (the `would_survive` predicate is identical there), but they are outside the re-base range — they need the same carry-over with a per-version check, tracked separately.

**Not in this triage:** the water fix (ee7ab396) is carried over separately (`fix/e6-water-carryover`, paused per the owner's 2026-11-22 sequencing word) and 1.21.11 itself carries the same bug — it is not part of the 1.21.11 port's fixes. The 1.21.7 skeleton-burn investigation is also separate.

## Cost estimate

### Model

Propagation is not original porting. The programme rate for original porting is ~92.5 min per changed item (~12 items/day/agent), which includes vanilla research, design, and debugging from scratch. Propagation skips the research and design: the fix is already written and proven in 1.21.11. The work is apply + adapt + verify.

Per version (204 shared units), using the measured hunk mix:

| Work class | Units/version (avg) | Time each | Hours/version |
|---|---|---|---|
| Mechanical apply (all hunks apply) | ~125 | ~5 min | ~10 |
| Hand adaptation (≥1 differs hunk, or partially applied) | ~70 | ~25 min | ~29 |
| Rework (no hunks apply, or file absent) | ~6 | ~60 min | ~6 |
| New files (ice_spike, random_patch) | 2 | ~60 min | ~2 |
| Verification (cargo check + tests + CI proof + fallout) | — | — | ~8 |
| **Per version** | | | **~55 h ≈ 7 agent-days** |

Shared overhead (once, not per version):

| Item | Cost |
|---|---|
| 11 uncertain units — manual triage with jar checks | ~1 day |
| Trees-fix carry-over (applies cleanly; test hunk per version) | ~0.5 day |
| Coordination, per-version branches, merges, VERSIONS.md | ~2 days |
| **Shared overhead** | **~3.5 days** |

### Total

| | Agent-days |
|---|---|
| 6 versions × ~7 days | ~42 |
| Shared overhead | ~3.5 |
| **Total** | **~45 agent-days** |

**Range: 35–60 agent-days.** The estimate is sensitive to three unknowns:

1. **The 674 differs hunks (6%)** — most are probably trivial context drift, but some may hide real divergences where the target's own behaviour port fixed the same area differently (1.21.8's 174 differs hunks / 10% rate is the biggest unknown). If the differs hunks average 15 min instead of 25, the total drops ~7 days; if they average 45 min, it rises ~12 days.
2. **Verification fallout** — the 8h/version verification budget assumes the propagated code compiles and passes tests with minor fixes. A version whose tree has drifted structurally (1.21.8 again) could take longer.
3. **The uncertain units** — the 1-day triage budget assumes the splits resolve cleanly with jar checks. Item 751's per-version villager-trade tables are the largest single unknown (transcribing 297 + 94 listings per version from each jar); if those tables are needed for all six versions, add ~1–2 days.

**In programme terms:** 1,224 unit-applications (204 × 6) at ~45 agent-days is ~22 min per unit-application on average — about 4× cheaper than the original-porting rate, which is the expected ratio for apply-and-verify work versus research-and-write work.

**Wall-clock:** the six versions are independent once the uncertain triage is done, so the work parallelises. Three streams → ~2 weeks; six parallel agents → ~1 week plus merge overhead. The 1.21.11 merge and the uncertain triage are the only serial prerequisites.

## Sequencing

1. **1.21.11 merge (prerequisite).** The source must be on main before propagation starts, so the source diff is stable. Fresh final proof run 44983736839 was dispatched 2026-11-22 09:53 NPT after the D2 fix; merge + VERSIONS.md flip follow when it is green. The sign-boards fix (`fix/1.21.11-sign-boards`, ac8dd9dc) merges after behaviour/1.21.11 lands — it is a rendering feature, not part of this triage.
2. **Uncertain triage (~1 day, prerequisite).** Resolve the 11 uncertain units with jar checks before any propagation branch starts, so the propagation agents work from a complete list.
3. **1.21.10 first.** Closest to 1.21.11 (fewest differs: 82 hunks, 5%; highest applies: 1,666), no behaviour port yet so no local fixes to reconcile, and its own 719-item behaviour port is pending — **the re-base should land before that port starts**, so the port works on rebased code and does not redo work the re-base covers.
4. **1.21.9, then 1.21.6.** Next-lowest differs (86, 92 hunks). 1.21.9 also takes the copper-golem units (items 86, 109, 446).
5. **1.21.7.** 108 differs hunks. The owner's playtest version — extra care, and a laptop verification build (CI client build + pull) after the merge so he can play the rebased tree. The trees fix is already in 1.21.7; the two already-fixed units (336b4355:worldgen.rs, 3310abc4:noise.rs) need no work.
6. **1.21.5.** 132 differs hunks; the oldest E6 version, most drift. Also needs the `dialog_screen.rs` (item 87) location work.
7. **1.21.8 last.** Highest differs (174 hunks, 10%) and the most local divergence to reconcile (526-item behaviour port). Do it after the process is proven on the easier versions.

### Dependencies and ordering notes

- **Water carry-over:** separate work (`fix/e6-water-carryover`, paused per the owner's 2026-11-22 word — resumes after 1.21.7 verification). The re-base does not include it. If the water carry-over lands in 1.21.11 before propagation, the re-base's `fluids.rs` hunks may conflict — sequence the water carry-over merge before propagation starts, or handle the conflict in the first propagation branch.
- **1.21.10 behaviour port (719 items, 0 done):** re-base first, then port on the rebased tree.
- **1.21.4 (548 items, Stream 3 next):** 1.21.4 is outside the re-base range (it is E6 but below 1.21.5). The re-base does not cover it; the trees fix applies to it separately (see above). Stream 3's 1.21.4 port should be aware of the shared fixes — the 204 shared units are a checklist for it.
- **Leakage audit tool (`tools/leakage-audit`, in progress):** the standing guard against 26.3 leakage. Run it over each rebased version as a verification step; it should flag any remaining tag/attribute/gamerule references that do not exist in the target version's data.

## Propagation workflow (recommended)

Per version, on a branch `rebase/<version>` off main (after the 1.21.11 merge):

1. **Scripted apply** of the fully-applying units (the data file gives file + target lines per hunk). Apply in batches of ~20 units, `cargo check` after each batch, commit per batch.
2. **Hand adaptation** of the differs hunks — read the 1.21.11 diff, read the target's code, adapt the fix to the target's form. Where the target's own port already fixed the same area differently, keep the target's form if it is vanilla-correct (verify against the jar) and record it as already-fixed.
3. **New files** — create `ice_spike.rs` and `random_patch.rs`, wire them into the module tree, remove/redirect the 26.x form.
4. **Tests** — apply the test hunks (88% apply); adapt the rest to the version's test modules.
5. **Verify** — `cargo check` + the affected test suites locally, then a CI port-verify proof run per version (the published profile is the evidence, not the top-level summary).
6. **Merge** — merge to main, flip the VERSIONS.md row, record the pin (with `work-items.json`, `harness-subset.txt`, `regen-determinism.json` in the same commit — the pin-placement lesson).

**Do not** propagate the 34 specific units or the specific halves of the 11 uncertain units. **Do not** propagate the Horse/ZombieHorse malus (item 93) or the `EnvironmentAttributes` removals (item 155) to any E6 version.

## Data file schema

`docs/e6-rebase-triage.json`:

- `meta` — provenance (source range, branch, main tip, date, method), classification counts, per-version unit + hunk counts, scope notes.
- `shared` — 204 rows. Each row: `unit_id`, `sha`, `item`, `kind` (`commit` | `abatch-file`), `files` (1.21.11 paths), `reason` (why it is shared), `versions` — per version: `status` (`applies` | `already-fixed` | `differs` | `uncertain`) and per-file `status`, `hunks`, `prod_hunks` / `test_hunks` counts, `tests_need_adapting`, `target_lines` (line numbers in the target file where the hunk matches).
- `trees_fix` — the 1.21.7-origin trees-fix row, same shape.
- `specific` — 34 rows (unit, sha, item, kind, files, reason).
- `uncertain` — 11 rows (same shape, with the split described in `reason`).
