# The delta bar: checklist and gate

Companion to `docs/ROADMAP.md` §8.2. Turns D1–D6 into six checks a pipeline can run,
against artifacts the pipeline already produces, so a release is either **delta-verified** or it
isn't — with no third state and no adjective in between.

The gate is `tools/delta_verify.py` at the repo root (pass `--tree` when the
release's tree is elsewhere). Stdlib only, 3.10+.

```sh
python3 tools/delta_verify.py 1.21.10                 # judge recorded evidence
python3 tools/delta_verify.py 1.21.10 --run           # run prepare, cargo, harnesses first
python3 tools/delta_verify.py 1.21.10 --run --download
python3 tools/delta_verify.py 1.21.10 --check D3      # one check, while working
```

Exit status: **0** delta-verified · **1** failed · **3** blocked (evidence missing) · **2** the
pin's own metadata is malformed. **Blocked is not verified.** That distinction is the whole point:
`3` means "not finished", `1` means "wrong", and only `0` may enter `VERSIONS.md`.

---

## In this repository today

The tree lives at `versions/<id>/` and holds both the code and this release's metadata, so the gate
needs no `--pin`:

```sh
python3 tools/delta_verify.py 1.21.11 --tree versions/1.21.11            # judge recorded evidence
python3 tools/delta_verify.py 1.21.11 --tree versions/1.21.11 --check D3 # one check, while working
python3 tools/delta_verify.py --index                                    # VERSIONS.md rows
```

Artifacts land beside the tree: `versions/<id>/{work-items.json, harness-subset.txt,
allowed-failures.txt, regen-determinism.json, playtest.md, profile.json, ci-out/, delta-verdict.json}`.
§2 also describes `pins/<release>/`, which is the redraft's target shape (code in `lines/<era>/`,
metadata in `pins/<release>/`); both layouts work, and `--pin` overrides either.

---

## 1. The one rule

> A release is delta-verified iff **D1–D6 all pass at one commit**, with the evidence recorded in
> its pin directory, and D6 signed by a person other than the one who did the port.

Everything else follows from that sentence. The gate enforces the mechanical parts; §7 lists the
parts only a reviewer can enforce.

---

## 2. Pin layout

Two layouts, and the second is the one to pick: pin metadata beside the version, not a second
per-version tree. The gate resolves `--pin`, then `pins/<release>/`, then `versions/<release>/`,
then `<tree>/pins/<release>/`, then the tree itself.

```
lines/1.21.11/                  one real Rust tree per era line (code only)
versions/1.21.10/               this release's metadata, no crates
  tier.txt                      "T" (template) or "D" (delta) — the index's first scalar
  profile.json                  the published sweep: which harnesses ran, and how they ended
  work-items.json               D3 — closure ledger, hashed against report.json
  harness-subset.txt            D5 — this era's frozen regression subset
  allowed-failures.txt          any check — exemptions, each with cause + owner
  regen-determinism.json        D1 — committed vs freshly regenerated table hashes, + overrides
  playtest.md                   D6 — signed play gate
  lan-join.log                  D6 — evidence the playtest cites
  ci-out/cargo-check.log        D2 — recorded, when the gate runs without --run
  ci-out/cargo-test-protocol.log
  delta-verdict.json            written by the gate; the only thing VERSIONS.md reads
```

`profile.json` is the gate's preferred evidence for D4/D5. It accepts the summary shape the
emitter already produces — `{"harness": {"passing": [...], "failing": [...]}}`, singular or
nested — as well as `{"harnesses": {name: status}}` and `{"results": [{name, passed}]}`, with
`pass`/`passed`/`ok` normalised. The point is that a published profile exists, not that it is
written in one dialect. When a profile carries both a summary and a `harnesses` map, the map
wins and the summary fills gaps — so a harness with a status of its own (`oracle`, `flaky`)
keeps it instead of disappearing into "unmeasured". `parity.py`'s local `state.json` is
accepted only as a fallback, so a
verdict judges evidence that was actually shipped.

A profile stamped with a `commit` that does not match HEAD is **refused**, not downgraded: it is
not evidence about this tree. That is the one cross-file check in the gate, and it exists because
a profile, a playtest and a verdict are three documents that can otherwise disagree quietly.

Things the gate reads from the tree, not the pin: `research-lock.json`,
`crates/horizon-core/src/version.rs`, the `diff-<prev>-<release>/report.json` and `regen.json`
that `version_port.py` writes into its store, `.local/horizon-client/<release>/`, and
`.local/parity-runner/state.json` (`parity.py`'s own record).

---

## 3. D1 — Pin regenerated, prepare green, deterministic

| | |
|---|---|
| **Must hold** | The tree pins this exact release; all six tables regenerated from its own jar; prepare ran end to end; regeneration is byte-stable. |
| **Command** | `python3 tools/version_port.py prepare <v>` (add `--download` if the store is cold); `python3 tools/version_port.py regen <v> --worktree A`, then `--worktree B`. |
| **Artifacts** | `research-lock.json`; `diff-*-<v>/regen.json`; `pins/<v>/regen-determinism.json`; `.local/horizon-client/<v>/`. |
| **Passes when** | `minecraft.version == <v>`; `client_sha1`, `server_sha1`, `java_major` present; `version.rs` says `VERSION_ID == <v>` with non-zero protocol/data; every generator in `regen.json` clean; a fresh regen reproduces the **committed** tables exactly, except tables listed in `overrides` with a cause and an owner; prepared data exists (or prepare exited 0 under `--run`). |
| **Cost** | 1–2 h warm; 10–30 min with a hot store. |

Producing the determinism record (the only artifact nobody generates today). **One side must be
the committed tree** — hashing two fresh regenerations against each other passes two identically
wrong runs, which is precisely how a hand-corrected table goes unnoticed:

```sh
python3 tools/version_port.py regen 1.21.10 --worktree /tmp/regen-fresh
python3 - <<'PY'
import hashlib, json, pathlib
tables = ["crates/horizon-core/src/version.rs", "crates/horizon-protocol/src/ids.rs",
          "crates/horizon-client/src/lan/tracking.rs", "crates/horizon-client/src/lan/command_order.rs",
          "crates/horizon-client/src/lan/command_suggestions.rs", "crates/horizon-command/src/vanilla_order.rs"]
def hashes(root):
    return {t: hashlib.sha256((pathlib.Path(root) / t).read_bytes()).hexdigest() for t in tables}
record = {
    "release": "1.21.10",
    "committed": hashes("versions/1.21.10"),      # the tree as shipped
    "fresh": hashes("/tmp/regen-fresh"),           # one run of the generators
    "overrides": [],                               # each: {table, reason, owner}
}
pathlib.Path("versions/1.21.10/regen-determinism.json").write_text(json.dumps(record, indent=2))
PY
```

Any table where `fresh` differs from `committed` is either a generator bug or a hand correction,
and both need to be visible. A hand correction is allowed — but only as an `overrides` entry with
a reason and an owner: 1.21.11 carries five hand-corrected adapter tables and a `slot_display`
numbering trap (identity for the 8-entry registry, 11→8 for the 26.x versions), where the
generator is wrong for the codec as it stands. Under the old two-fresh-runs recipe all six would
pass silently; under this one each has to be declared, and the declaration is what tells the next
person the generator needs fixing before that override can go.

---

## 4. D2 — Build and protocol tests

| | |
|---|---|
| **Must hold** | `cargo check --workspace --all-targets`: 0 errors, 0 warnings. `cargo test -p horizon-protocol`: green except exemptions. |
| **Command** | `CARGO_BUILD_JOBS=2 cargo check --workspace --all-targets`; `cargo test -p horizon-protocol`. |
| **Artifacts** | `ci-out/cargo-check.log`, `ci-out/cargo-test-protocol.log` (recorded mode), or the gate runs them (`--run`). |
| **Passes when** | No `^error` lines, no `^warning:` lines; every failing test name is covered by an exemption. |
| **Cost** | 2–5 min warm, 10–20 min cold. `CARGO_BUILD_JOBS=2` because the full check OOMs at default jobs on a 6 GB VM. |

Record the exemption with its cause and owner — the gate refuses a bare name:

```
sandbox_network_dns            # cause: CI proxy blocks resolution, owner: karina
sandbox_network_dead_address   # cause: sandbox forbids outbound, owner: karina
```

---

## 5. D3 — Changed-scope behaviour closed or accepted

| | |
|---|---|
| **Must hold** | Every rank-1 item closed; every rank-2 item closed or recorded as an accepted gap with a cause and an owner. Rank-3+ inherit the era template's answer, in writing. |
| **Command** | `python3 tools/version_port.py report <prev> <v>` (produces `diff-<prev>-<v>/report.json`, `.md`); then record closure. |
| **Artifacts** | `diff-*-<v>/report.json` (pipeline) + `pins/<v>/work-items.json` (human/agent). |
| **Passes when** | `work-items.json.generated_from` equals the sha256 of `report.json`; each rank's `total` equals `report.json.ranks`; `closed + accepted == total`; every accepted gap has `cause` **and** `owner`; `rank3plus` says what is inherited. |
| **Cost** | diff 6 s + report seconds; the recording is bookkeeping, but it is the ledger the whole bar rests on. |

`work-items.json` — counts are what the gate checks, so the numbers cannot drift from the diff:

```json
{
  "release": "1.21.10",
  "generated_from": "<sha256 of report.json>",
  "rank1": { "total": 10, "closed": 10, "accepted": [] },
  "rank2": { "total": 492, "closed": 461,
             "accepted": [ { "id": "harness:lan", "cause": "sandbox blocks UDP", "owner": "karina" } ] },
  "rank3plus": "inherited from 1.21.11 (template); 247 open there, none reopened here"
}
```

The `generated_from` hash is the anti-drift device: re-running the diff invalidates the ledger
until it is re-recorded, so a release cannot be "verified" against a report from a different tree.

---

## 6. D4 — Data compatibility, D5 — Era regression subset, D6 — Play gate

### D4

| | |
|---|---|
| **Must hold** | The `datafix` ladder and a save round-trip pass for this release. |
| **Command** | `python3 tools/parity.py datafix save_roundtrip --timeout 1800` |
| **Artifact** | `.local/parity-runner/state.json` |
| **Passes when** | Both records say `"status": "pass"`. |
| **Cost** | 15–40 min; the most valuable check here — 26.1's datafix run reported 19,372 differences over 7 worlds, which is exactly the silent rot this catches. |

### D5

| | |
|---|---|
| **Must hold** | The era's frozen subset (≥5, typically ~40 harnesses) is green, or exempted with cause + owner. |
| **Command** | `python3 tools/parity.py $(cat pins/<v>/harness-subset.txt) --timeout 1800` |
| **Artifacts** | `pins/<v>/harness-subset.txt`, `state.json`. |
| **Passes when** | Every named harness is measured (`pass`) or exempted. Unmeasured is blocked, never pass. |
| **Cost** | ~1.5 min per harness warm (10–60 min); ×8–10 cold. |

Freeze the subset per era and change it only with a dated `#` comment, otherwise a release can be
made "verified" by shrinking its own test set. Start each era's list from upstream's
`parity.py --list` filtered to the subsystems the era's template touched — for E6, the 17-harness
port profile is the seed, not the answer.

### D6

| | |
|---|---|
| **Must hold** | A person played it: world loads, and a vanilla client of that exact version joined a Rust LAN world of that version; evidence attached. |
| **Command** | Manual. Nothing here can be automated, and pretending otherwise is how "works" gets claimed without being seen. |
| **Artifact** | `pins/<v>/playtest.md` + the log/screenshot it cites. |
| **Passes when** | `release:` matches, `commit:` matches HEAD, `tester:` present, ≥6 ticked boxes, every `evidence:` path exists. |
| **Cost** | 30–60 min per release. |

```markdown
# Playtest — 1.21.10

release: 1.21.10
commit: 0123456789abcdef
tester: karina

- [x] world loads from a level.dat saved by 1.21.10
- [x] movement, swimming, elytra
- [x] combat, knockback and shield
- [x] containers, crafting, anvil
- [x] a redstone sample machine ticks like vanilla
- [x] worldgen matches a vanilla seed at spawn
- [x] vanilla 1.21.10 client joins the LAN world
- [x] save round-trip: play, save, reopen

evidence: lan-join.log
```

---

## 7. Rules that keep it honest

1. **D6 is signed, never self-signed.** The gate checks the stamp; a reviewer checks that the
   signer is not the porter. Record the reviewer in the pin where the team is more than one.
2. **An accepted gap needs a cause and an owner.** `allowed-failures.txt` and
   `work-items.json` reject anything else — by design, at exit code 2.
3. **Counts come from the diff.** `generated_from` binds the ledger to one `report.json`. Re-diff
   and the ledger is stale, not reusable.
4. **The subset is frozen.** Growing it is cheap and encouraged; shrinking it is a decision with a
   date and a name attached.
5. **Unmeasured is blocked.** The *local* 1.21.11 run left 8 of 17 harnesses unmeasured — CI
   later measured all 17 (2 pass / 13 fail), and the notes correctly refused to call the
   unmeasured ones anything; the gate does the same mechanically.
6. **One commit.** Verdict records the commit; if HEAD moves, rerun.
7. **Precedence:** `fail` beats `blocked` beats `verified`. A wrong answer outranks a missing one.

---

## 8. Verdict file and the truth chain

Three status artifacts cannot each be authoritative. The chain is one-directional, and each
stage is derived from the one before it:

```
  parity.py run          .local/parity-runner/state.json     raw, local, disposable
        │  (CI uploads it as one completed sweep)
        ▼
  profile.json           per release, committed            the frozen record: who ran, what passed
        │  (the gate reads it for D4/D5)
        ▼
  delta-verdict.json     per release, committed            six checks + scalars, written by the gate
        │  (delta_verify.py --index)
        ▼
  VERSIONS.md rows       generated                           no prose, no adjectives, no hand edits
```

So `status.json` retires, or becomes a regenerated cache with no hand-edited fields. If a number
appears in the index and is not in a verdict, it is wrong by construction — which is the failure
mode that produced "completion pass landed" beside a 3-of-17 profile.

`delta-verdict.json`:

```json
{
  "schema": "delta-verdict/1",
  "release": "1.21.10",
  "commit": "0123456789abcdef",
  "mode": "run",
  "verified": true,
  "state": "verified",
  "checked_at": "2026-11-12T09:14:03Z",
  "scalars": { "tier": "D", "suite_pass": 6, "suite_total": 6, "suite_exempt": 0,
               "items_closed": 495, "items_total": 496, "items_accepted": 1 },
  "index_line": "| 1.21.10 | D | 6/6 | 495/496 | 1 | verified 2026-11-12 |",
  "checks": {
    "D1": { "status": "pass", "notes": ["lock pins 1.21.10 (java 21)", "6 tables identical across two regen runs"], "evidence": ["research-lock.json"] },
    "D2": { "status": "pass", "notes": ["cargo check: 0 errors, 0 warnings", "protocol tests: 438, none failing"], "evidence": [] },
    "D3": { "status": "pass", "notes": ["rank-1: 10 closed, 0 accepted, 10 total", "rank-2: 491 closed, 1 accepted, 492 total"], "evidence": [] },
    "D4": { "status": "pass", "notes": ["datafix, save_roundtrip: pass"], "evidence": [] },
    "D5": { "status": "pass", "notes": ["6/6 subset harnesses pass"], "evidence": [] },
    "D6": { "status": "pass", "notes": ["playtest signed by karina, 8 checks, commit 0123456"], "evidence": ["playtest.md"] }
  }
}
```

`VERSIONS.md` is then not written, it is printed:

```sh
python3 tools/delta_verify.py --index > /tmp/index.md   # from pins/, versions/, lines/
```

```
| Release | Tier | Subset | Items | Accepted | Status |
|---|---|---|---|---|---|
| 1.21.10 | D | 6/6 | 495/496 | 1 | verified 2026-11-12 |
| 1.21.11 | T | 40/40 | 743/743 | 0 | verified 2026-11-09 |
```

A release with no verdict is simply absent — which is the correct display for "not done", and
the reason the command exits 3 when it finds no verdicts at all rather than printing an empty
table that could be mistaken for success.

---

## 9. Running it from CI

The gate is record-and-judge, so CI only has to produce artifacts. Two jobs per release.

**`port-verify` v6.2 already records both D2 logs.** If a line still does not, the steps are these
— and the `set -o pipefail` is not decoration: without it `tee` reports its own status, which is
how an earlier draft of this note quoted an exit code the gate never returned.

```yaml
      - name: Workspace check (D2 evidence)
        continue-on-error: true
        run: |
          set -o pipefail
          cargo check --workspace --all-targets 2>&1 | tee ci-out/cargo-check.log
      - name: Protocol tests (D2 evidence)
        continue-on-error: true
        run: |
          set -o pipefail
          cargo test -p horizon-protocol 2>&1 | tee ci-out/cargo-test-protocol.log
```

With those two steps the 26.1 line will fail D2 on its 48 test-target errors — which is the
honest verdict and the work item, not a reason to weaken the bar.

```yaml
  record:                      # the harness SET is split here, one job per slice
    strategy: { matrix: { slice: [1,2,3,4,5,6,7,8,9,10] } }
    steps:
      - name: This slice's harnesses
        run: |
          python3 tools/parity.py --list > /tmp/all.txt
          python3 - "${{ matrix.slice }}" <<'PY'
          import sys, pathlib
          names = [l.strip() for l in pathlib.Path("/tmp/all.txt").read_text().split()
                   if l.strip() and not l.startswith("/")]
          index = int(sys.argv[1]) - 1
          pathlib.Path(f"/tmp/slice-{sys.argv[1]}.txt").write_text("\n".join(names[index::10]))
          PY
      - run: python3 tools/parity.py $(cat /tmp/slice-${{ matrix.slice }}.txt) --timeout 1800
      - uses: actions/upload-artifact@v4
        with: { name: parity-${{ matrix.slice }}, path: .local/parity-runner/state.json }
  judge:                       # merges the slices into the published profile, then judges
    needs: record
    steps:
      - uses: actions/download-artifact@v4
        with: { pattern: parity-*, path: slices }
      - name: Merge into versions/$V/profile.json
        run: |
          python3 - "$V" <<'PY'
          import json, os, pathlib, sys
          release = sys.argv[1]
          merged = {"release": release, "harnesses": {}, "commit": os.environ.get("GITHUB_SHA", "")}
          for path in pathlib.Path("slices").glob("*/state.json"):
              merged["harnesses"].update(json.loads(path.read_text()).get("harnesses", {}))
          out = pathlib.Path(f"versions/{release}/profile.json")
          out.parent.mkdir(parents=True, exist_ok=True)
          out.write_text(json.dumps(merged, indent=2) + "\n")
          print(f"{out}: {len(merged['harnesses'])} harness records")
          PY
      - run: python3 tools/delta_verify.py "$V"        # exit 3 → not finished, exit 1 → broken
      - uses: actions/upload-artifact@v4
        with: { name: delta-verdict, path: versions/${{ env.V }}/delta-verdict.json }
```

Two different splits exist and neither should be confused with the other: the matrix above
splits the harness *set* across jobs; `PARITY_SHARD=2/6` splits one expensive harness's *cases*
(`datafix` is the one that needs it — six shards in the 1.21.11 profile) across runners that
otherwise share the same list. Use the first to fit the 300-minute job cap, the second to stop a
single harness being the critical path.

`--run` exists for the local loop and for a cold tree; keep CI on recorded mode so the verdict is
a judgement on uploaded evidence rather than on whatever the runner happened to build. The full
suite does not fit one 300-minute job (a warm sweep is ~970 harness-minutes), which is what the
shard matrix is for.

---

## 10. What this changes in practice

Run today against the published trees, the gate reports every release **blocked** — `1.21.11`
returns `D1 blocked (no regen.json in the pin), D2–D5 blocked, D6 blocked`, exit 3. That is the
correct reading of the current state: the tree compiles, prepares and has a complete rewire, and
it has not met the bar, because nobody has recorded the evidence and D3's 815 items are open.

That is also the point. Six checks, one file, one exit code, and a `VERSIONS.md` that can no
longer say "completion pass landed" about a tree that fails 13 of 17 harnesses. The cost of
running it honestly is 30–90 minutes per release on recorded evidence, and the cost of not
running it is the gap that already exists between the port notes and the index.
