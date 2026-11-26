# Restructure sketch: commons + thin per-version overlays

**Status: steps 1–4 executed on branch `fix/repo-restructure`
(2026-11-27); cutover (step 5) NOT done — pending the owner's timing
decision. See §8 for the execution record and §9 for the cutover plan.**
Written 2026-11-12 in response to external review (Claude + Freebuff), both of which independently flagged that the repo's
layout contradicts the roadmap's own §4-P2 design. This sketch turns P2 into a
concrete shape. Execution needs the owner's word and must be sequenced between
agent jobs, not mid-flight.

## 1. The problem, measured

The repo holds a full copy of the upstream snapshot in every `versions/<id>/`:

| Version | Tree size | Files | Diff vs its predecessor |
|---|---|---|---|
| 26.3 | 137 MB | 5,914 | — (base) |
| 26.1 | 137 MB | 5,914 | 135 files vs 26.3 |
| 26.1.1 | 137 MB | 5,915 | 9 files vs 26.1 |
| 26.1.2 | 137 MB | 5,915 | 9 files vs 26.1.1 |
| 26.2 | 137 MB | 5,914 | — |
| 1.21.11 | 137 MB | 5,916 | 148 files vs 26.1 |

Six trees = 822 MB working tree, ~35,500 files. At 103 versions that is ~14 GB
of working tree and ~600k files. Disk is not the real cost — git dedupes
identical blobs (the `.git` pack for all six trees is 101 MB) — the real cost
is the one P2 names: **every engine fix has to land N times**, or the trees
diverge into 103 fossils. Nothing can cherry-pick a fix across directory
copies, so commons-sync today is manual copying. The restructure fixes the
maintenance shape, not the disk shape.

**Correction (2026-11-27, measured from `git ls-tree`): the Nov-12
table's per-tree size still holds (137 MB, ~5,916 tracked files), but
there are now 19 trees, so the working tree is ~2.6 GB / ~112,650
files. The measured chain diffs are also larger than the Nov-12
table: 26.3 → 26.1 is 552 files, not 135; 26.3 → 1.21.11 is 691,
not 148. The overlay file counts after extraction (metadata
excluded) are in §8.

## 2. Target shape

```
commons/                  One full copy of the upstream base tree (today's 26.3),
                          version-agnostic: renderer, UI, audio, math, launcher,
                          telemetry, shared tools. The living codebase.
versions/<id>/
  overlay/                Only the files that differ from commons for this
                          version, at their tree-relative paths (generated
                          tables, rewired packets, behaviour deltas, oracle
                          adaptations, tools/ differences).
  research-lock.json      Pin, protocol, Java major, jar SHAs (unchanged).
  PORT-NOTES-<id>.md      Authoritative per-version record (unchanged).
  status.json             Machine-readable status (unchanged).
  README.md               Reframed front doc (unchanged).
tools/assemble.py         Builds a full working tree: commons + overlay chain.
```

**Overlay chains, not flat overlays.** A patch release layers on its parent,
an in-era release layers on its era template:

- `26.1.1` = commons + 26.1 overlay + 26.1.1 overlay (9 files)
- `1.21.10` = commons + 1.21.11 overlay + 1.21.10 overlay

This is the delta-chain method already proven by the 26.1 carries (method b:
start from the completed predecessor, re-apply the measured delta) — the
restructure makes the repo layout match the method instead of fighting it.

**Era lines** stay the living codebases per P2: the era template's overlay is
where era-level work lands (oracle bootstrap, data model), and in-era releases
inherit it. Commons fixes land once and flow down through assembly; era fixes
land on the template overlay.

## 3. How the pipeline keeps working

The pipeline (`tools/version_port.py`, `tools/port_harness.py`, the CI
workflow) operates on a **full tree** with a `.git` at the candidate root.
That does not change. What changes is where the full tree comes from:

- **Local work:** `tools/assemble.py <version> --into <dir>` copies commons,
  applies the overlay chain, and produces a full tree. Agents work in the
  assembled tree exactly as today. When a job finishes, the agent extracts the
  diff back into the overlay (`assemble.py --extract`) instead of committing
  the whole tree.
- **CI (`port-verify`):** the "stage the candidate" step assembles commons +
  overlay instead of copying a published tree. Everything downstream (prepare,
  lab-prepare, harness subset) is unchanged.
- **Verification gate for the migration itself:** every assembled tree must be
  **byte-identical** to today's published tree (`diff -r`, excluding
  `.local/` and generated files). No full copy is deleted until its assembled
  twin verifies.

**Tools promotion.** The per-version `tools/` directories have already
diverged in ways that belong in commons: 1.21.11's `lab.py` remap stage and
signature stripping, 26.1's `port_regen.py` EntityType pre-split fallback.
Migration promotes the shared machinery to commons `tools/` and leaves only
genuine per-version differences as overlay files.

## 4. Migration plan

1. **Build `tools/assemble.py`** (copy + overlay apply + `--extract` diff-back),
   plus a `tools/verify_assembly.py` that assembles all six versions and diffs
   each against the published tree.
2. **Extract the overlays** for the six published versions by diffing each
   tree against the 26.3 base. 26.3 itself becomes `commons/` (moved, not
   copied, so its history follows).
3. **Verify byte-identical assembly** for all six. Any mismatch blocks the cut.
4. **Update `port-verify`** to assemble the candidate; prove it with one
   dispatch against an assembled tree before the cut.
5. **Cut over in one commit:** overlays + commons + workflow change, full
   copies removed. The copies remain in git history, so nothing is lost.
6. **Update agent briefs:** port agents work in assembled trees and extract
   overlays on completion. The standing publish rule ("ship reframed front
   docs, confine pushes to your own `versions/<id>/`") is unchanged in spirit;
   the overlay directory is the new confinement boundary.

## 5. Sequencing constraints

- **Not mid-flight.** The E6 delta-chain agent (1.21.10, then 1.21.9) works on
  full trees and publishes `versions/1.21.10/` in the current shape. The
  restructure runs after it lands, or its in-flight tree is converted during
  migration — decided at execution time, not assumed.
- **Not during a CI run.** The port-verify workflow change lands when no run
  is in flight (a running run uses the workflow as dispatched, so the risk is
  confusion, not breakage — still, land it clean).
- **Cheapest now.** Six trees is the cheapest this ever gets. Every version
  added in the old shape is another tree to convert, and every fix landed in
  N trees is a fix the migration has to reconcile.

## 6. Interaction with the other review fixes

- **Control run (CI feedback).** The 26.3 control profile defines the Phase 0
  gate baseline. After the restructure, the control run assembles `commons/`
  directly — it is the same tree, so the baseline is unaffected by the move.
- **CI-emitted status (CI feedback).** `status.json` is currently maintained
  by hand when a profile is recorded. The end state is the workflow emitting
  it from `harnesses.json`. That change is independent of the restructure and
  can land before or after it; the file's location (`versions/<id>/`) does
  not change.
- **Front-end generalization (CI feedback).** Multi-namespace tables and
  member-level coverage are prerequisites for the 1.12.2 SRG proof, not for
  the restructure. The front-end already lives at repo-root `tools/`, shared
  by all versions — the restructure does not touch it.

## 7. Open questions for the owner

1. **Execute?** This sketch is a proposal. Execution is a half-day job for
   one agent (assemble tooling + extraction + verification + workflow change
   + cutover), best done with no other agent running.
2. **Timing vs the E6 chain?** Restructure after 1.21.10/1.21.9 land, or
   pause the chain and restructure now while the tree count is six?
3. **Freeze policy interaction?** P2's freeze policy (an era line freezes as
   a finished artefact after its soak) is unchanged — a frozen era's overlay
   simply stops receiving commons updates. No decision needed unless the
   owner wants the soak period defined now.

## 8. Execution record — steps 1–4 (2026-11-27, branch `fix/repo-restructure`)

The owner gave the word on 2026-11-27 01:22 NPT. Steps 1–4 are done on
the branch; step 5 (cutover) is deliberately NOT done — the 1.21.4
behaviour port is live, so the proposal's own sequencing constraint
("best done with no other agent running", "not mid-flight") forbids it.

- **Tooling** (`tools/assemble.py`, `tools/verify_assembly.py`,
  `tools/overlay-chains.json`, commit 42101615): `assemble.py` builds a
  full tree from a base + overlay chain + the version's metadata files,
  `--extract` diffs an edited assembled tree back into the overlay and
  the metadata files, and `--extract-all` regenerates every overlay
  from the committed trees via git plumbing. File enumeration uses
  `git ls-files --cached --others --exclude-standard`, so ignored build
  output is never copied while tracked-but-ignored files (e.g.
  `tools/jsr305-3.0.2.jar`) are. `verify_assembly.py` assembles each
  version into scratch and compares (git-mode, git-blob-sha) per path
  against the published tree. The `--extract` round-trip was proven on
  a throwaway repo (edited file → overlay, new file → overlay, deleted
  file → `.deletions`, metadata edit → versions dir, re-assembly
  identical).
- **Chains** (`tools/overlay-chains.json`): chains follow the derivation
  recorded in each tree's PORT-NOTES and are also the minimal chains by
  measured diff. Two corrections to the Nov-12 sketch: 26.1.1 chains
  from commons, not from 26.1 (it was re-ported from 26.3: 156 files vs
  26.3, 444 vs 26.1), and 1.21.11 chains from commons, not from 26.1
  (691 vs 26.3, 959 vs 26.1; the `diff-26.1-1.21.11` directory records
  a sizing measurement, not the derivation). Metadata files
  (PORT-NOTES, README, AGENTS, ROADMAP, research-lock.json, status.json,
  work-items.json, rank\*-progress.json, regen-determinism.json,
  harness-subset.txt, tier.txt, `diff-*/`) stay at `versions/<id>/` and
  are copied into the assembled tree last; they are never in an overlay.
  Deletions are recorded in `overlay/.deletions` (one path per line).
- **Extraction** (commit 4c91e08d): overlays extracted for all 18
  non-base versions — 26.2: 715 files/27 deletions; 26.1: 523/16;
  26.1.1: 147/1; 26.1.2: 25/0; 1.21.11: 678/2; 1.21.10: 587/1; 1.21.9:
  603/4; 1.21.8: 474/14; 1.21.7: 752/4; 1.21.6: 444/13; 1.21.5: 537/11;
  1.21.4: 542/1; 1.21.3: 79/0; 1.21.2: 5/0; 1.21.1: 121/2; 1.21: 5/0;
  1.20.6: 67/0; 1.20.5: 5/0. Total ~7,100 overlay files (~330 MB on
  disk; git dedupes the blobs — the overlay files are the same blobs
  the published trees already carry).
- **Verification**: `tools/verify_assembly.py` — 19/19 versions assemble
  byte-identically against their published trees.
- **port-verify** (commit 00ce804c, workflow v6.9): the workflow
  assembles the candidate with `tools/assemble.py` into
  `.horizon-ports/<v>` before anything that needs the workspace, inits
  git there, and runs the compile gate, test-target check, protocol
  tests, prepare and harness inside the assembled tree; rust-cache keys
  off the assembled lockfile. The version lock is still read from
  `versions/<v>/research-lock.json` in the checkout. Proof run:
  port-verify run 45887925797 on 1.21.7, dispatched 2026-11-27 from
  `fix/repo-restructure` (head 00ce804c) — overall conclusion FAILURE,
  but the failure is NOT the assembly: assemble ✓, stage ✓, compile
  gate ✓, test-target check ✓ (0 errors), protocol tests ✓ (438 pass),
  prepare ✓, and the judge ran against the assembled tree with D2 pass
  and D3 pass. Lab-prepare failed because Mojang republished 1.21.7's
  version JSON (fresh sha1 609e61c2…, in neither the lock's
  manifest_sha1 48c9b004… nor its accepted list [5149e9aa…]);
  `lab.py::import_manifest` refuses unknown manifests. The harness step
  was skipped as a consequence, so D1 is blocked (no prepared data) and
  no harness profile was produced. Verified 2026-11-27: Mojang has
  republished ALL 19 version JSONs since their locks were pinned, so
  this lab-prepare failure would hit any port-verify run on main right
  now, on any version — it is independent of the restructure (the
  assembled tree is byte-identical to the published tree, so the
  known-sha list and lock are identical either way). The fix is a
  separate job: add each fresh manifest sha1 to the corresponding
  lock's `accepted_manifest_sha1` (19 one-line lock updates), after
  confirming each republish is asset-index-only (the check already
  verifies the client/server jar sha1s match the lock). A full-green
  proof run needs that fix first; it was NOT done on this branch (out
  of scope — it touches all 19 published trees).

**Proposal gaps found during execution (not in the Nov-12 sketch):**

- The sketch covers only port-verify, but four more workflows assume
  `versions/<v>` is a full Cargo workspace and would break at cutover:
  `crate-tests.yml` (`cd versions/<v>`, rust-cache `workspaces:
  versions/<v>`), `client-build.yml` (same, plus the binary path
  `versions/<v>/target/release/horizon-client.exe`), `launcher-build.yml`
  (`cd versions/<v>/launcher`, `workspaces: versions/<v>/launcher`),
  and `worldgen-verify.yml` (`cd versions/<v>`, `workspaces:
  versions/<v>`). `leakage-audit.yml` also reads version trees
  (`has_tree(v)`). These need the same assemble-first treatment before
  the cutover, each with its own proof dispatch — they are NOT converted
  on this branch.
- The sketch says "six trees is the cheapest this ever gets". There are
  now 19; every version added in the old shape is another tree to
  convert, and the E6 chain is still adding them (1.21.4 in flight).
- A full worktree no longer fits the VM's disk quota alongside the
  existing ~45 worktrees: the first `git worktree add` failed with
  "Disk quota exceeded" at ~2.6 GB. The branch was built in a sparse
  worktree instead. The cutover commit itself is cheap (git objects are
  deduped), but any agent doing the cutover needs a full checkout or
  must work with plumbing only.

## 9. Cutover plan (step 5 — for the main agent to schedule)

Prerequisites (all must hold before the cutover commit):
1. The port-verify proof run on this branch shows the assembly path
   green (run 45887925797, 1.21.7: assemble, stage, compile, tests,
   prepare green; D2/D3 pass on the assembled tree). Full-green needs
   the Mojang-republish lock updates first (see §8): until the 19
   locks accept the fresh manifest sha1s, lab-prepare fails on main
   too, and no harness profile can be produced from any tree,
   assembled or not.
2. `crate-tests.yml`, `client-build.yml`, `launcher-build.yml`,
   `worldgen-verify.yml` (and the tree-reading part of
   `leakage-audit.yml`) are converted to assemble-first on this branch
   or a follow-up branch, each with a proof dispatch. Without this,
   the first crate-tests/client-build dispatch after the cutover fails
   on every version.
3. No port-verify / crate-tests / client-build run is in flight.
4. No agent is working in a full version tree (see the 1.21.4 plan
   below).

Cutover commit (one commit, on this branch, then merge):
1. `git mv versions/26.3 commons` (moved, not copied, so history
   follows — the sketch's requirement).
2. Delete every other full tree's content except `versions/<id>/overlay/`
   and the metadata files listed in §8 (i.e. `rm -rf` everything under
   `versions/<id>/` except `overlay/`, `research-lock.json`,
   `PORT-NOTES-<id>.md`, `status.json`, `README.md`, `AGENTS.md`,
   `ROADMAP.md`, `work-items.json`, `rank*-progress.json`,
   `regen-determinism.json`, `harness-subset.txt`, `tier.txt`,
   `diff-*/`). `pins/` is untouched.
3. The workflow changes (port-verify v6.9 already on this branch, plus
   the prerequisite conversions) land in the same commit.
4. Verify after the commit, before merging: run
   `tools/verify_assembly.py` against the cutover commit (it assembles
   from `commons/` now — the fallback to `versions/26.3` must NOT
   trigger) and re-dispatch port-verify on one version. If either
   fails, the cutover does not merge; the full trees remain in git
   history either way, so nothing is lost.

**1.21.4 conversion plan.** The `behaviour/1.21.4` branch is live
(worktree `~/workspace/mcir-work/port-1.21.4`) and its tree is
pre-restructure shape. Do NOT convert it mid-flight. Two options, in
order of preference:
- (a) Wait for it to merge to main, then extract its overlay with
  `tools/assemble.py --extract-all` (or a single-version extraction)
  from the merged tree — the same deterministic operation that built
  the other 18 overlays. No agent action needed.
- (b) If the cutover cannot wait: stop the 1.21.4 agent, checkpoint
  and push its branch, rebase it onto the cutover commit, then run
  `tools/assemble.py --extract 1.21.4 --from <its tree>` in its
  workdir (which writes `versions/1.21.4/overlay/` and the metadata
  files), delete the full tree content as in the cutover commit, and
  commit. The agent then resumes in an assembled tree
  (`assemble.py 1.21.4 --into <dir>`) and extracts on completion.
  Its `work-items.json` counts are unaffected — they live in the
  metadata set.
Either way, the agent's workdir must not be touched while it is
running, and the cutover must not land while the agent holds
uncommitted work.
