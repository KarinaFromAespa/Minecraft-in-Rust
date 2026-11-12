# Restructure sketch: commons + thin per-version overlays

**Status: proposal, not executed.** Written 2026-11-12 in response to external
review (Claude + Freebuff), both of which independently flagged that the repo's
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
