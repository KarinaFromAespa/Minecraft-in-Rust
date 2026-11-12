# Port notes: Minecraft 1.21.7

1.21.7 is the fourth adjacent delta port in the **Components era (E6,
1.20.5–1.21.11)**, taken from 1.21.8. Like 1.21.9, it shares its
protocol with its neighbour: protocol **772 on both sides**, world
version 4440 → 4438, data pack 81.0 and resource pack 64.0 unchanged.
There is no adapter re-base, no packet rewire and no oracle rework —
the measured delta is just **5 ranked work items**, the smallest yet
(smaller than 1.21.9's 27). The port is the mechanical layer: the pin
is set, the six generated tables are regenerated against 1.21.7's own
jar, the verification gates pass, and the remaining work is sized,
ranked and documented. The behaviour code in this tree is still the
26.3 base's, inherited through 1.21.11, 1.21.10, 1.21.9 and 1.21.8;
the port is **in progress**, not parity-verified.

## Provenance

- Base: the programme's published 1.21.8 tree (main@fb20f190, commit
  b884d16d: cross-protocol delta from 1.21.9, 6/6 tables regenerated,
  protocol tests 430 pass / 2 fail), with the measured 1.21.8 →
  1.21.7 delta re-applied (method (b) from the 26.1 carries).
- 1.21.7's facts, from its own version JSON and generated tables:
  protocol **772**, world (data) version **4438**, data pack format
  **81.0**, resource pack format **64.0**, Java **21**, stable release
  (2025-06-30). (The diff report lists the pack formats as
  `None.None` — a diff-tool artifact; the generated `version.rs` and
  the version JSON both give 81.0 / 64.0.)
- Prepare ran through the pipeline (verified jar download + generalized
  mappings front-end remap + datagen, 2026-11-12 ~14:38–14:41 UTC):
  **9,484/9,484 classes mapped (100%), 9,425 renamed,
  0 placeholders; 83,784 methods mapped (62,742 renamed), 42,848
  fields mapped (42,408 renamed)**. Nothing Mojang-derived is
  committed in this tree.
- Diff 1.21.8 → 1.21.7: **5 ranked work items** (rank 1: 3,
  rank 2: 1, rank 5: 1). Classes +0 −1, 0 moved, **6 changed
  (5 methods); 1 concerns Horizon**. No protocol change, no packet
  changes, no registry/block/item/command changes; data files ~1202
  changed, assets ~0, en_us ~0.

## The delta: a hotfix pin step

Rank 1 is the version pin (`version.rs`), the save-format item
(DataFixers 4440 → 4438: 0 schemas, 0 fixes added, 0 datafix classes
changed) and the version facts. Rank 2 is the one changed class that
concerns Horizon: `SharedConstants` (0 methods changed). Rank 5 is
the 6 changed/removed vanilla classes Horizon does not port (blaze3d
opengl 2, blaze3d/vertex 1, net/minecraft 1, client/multiplayer/chat
report 1, blaze3d 1). There is no behaviour delta to port beyond the
pin: the 1.21.8 behaviour code, adapter ladder, packet set and oracle
ports all carry forward untouched.

## Tables: 2 regenerated, 4 reproduce

| Generator | Result vs 1.21.8 |
|---|---|
| `version_consts` | regenerated (+5 −5 — the pin: 1.21.7 / 772 / 4438 / 81.0 / 64.0) |
| `packet_ids` | regenerated (+1 −1 — header comment only; the packet set is identical) |
| `entity_tracking` | unchanged |
| `lan_command_order` | unchanged |
| `command_suggestions` | unchanged |
| `vanilla_command_order` | unchanged |

The committed `regen.json` lists all six generators (D1 completeness,
per the gate change at main@15ff93a8): the final full regen ran with
no `--only` (2026-11-12 15:05–15:10 UTC, commit 9128f753ed). A fresh
regen reproduces all six committed tables byte-for-byte — see
`regen-determinism.json`, no overrides. (The brief expected the five
SlotDisplay hand-corrected tables to differ from a fresh regen; they
do not — the generators emit the same numbering the hand corrections
carry, as they did for 1.21.9 and 1.21.8.)

Two traps from the regen, for the next agent:

- The `port/1.21.8` branch does **not** contain the 1.21.8 agent's
  post-regen code changes (packet rewire, adapter re-base, oracle
  port): they live only in the published tree (and as uncommitted
  files in that agent's workdir). A regen from the bare branch fails
  at `vanilla_command_order` (`ACCEPT_CODE_OF_CONDUCT` missing from
  `ids`). The fix is to overlay the published 1.21.8 tree onto the
  candidate first — committed here as 7d55fec on `port/1.21.7` so the
  next agent starts from the completed tree.
- The branch's `gen_version_consts.py` handles only the (major,
  minor) pack form; 1.21.7's version.json uses single ints like
  1.21.8. The published tree's both-forms script is the one to use.

## Verification

- `cargo check --workspace --all-targets`: clean, 0 errors,
  0 warnings (11 m 53 s).
- `cargo test -p horizon-protocol`: **430 pass / 2 fail** (432 total —
  identical to 1.21.8, as expected for a same-protocol delta). The 2
  failures are the same pre-existing sandbox-network pair in
  untouched code, out of scope.
- `horizon-prepare` completes end-to-end: exit 0. The
  HORIZON_JAVA_25 datafix routing (v6.4) and
  `tools/jsr305-3.0.2.jar` both carry forward from 1.21.8 (verified
  in the candidate, not assumed).

## Harness subset

Measured locally on this VM (run 4, 2026-11-12 18:31–18:55 UTC,
24 m 25 s; prepare: ok). **2 pass / 2 oracle / 13 fail** of 17 — the
gate's classification from `harnesses.json` (no baseline was passed,
so all 15 non-passing are "new"). Identical to 1.21.8's profile,
as expected for a same-protocol delta with no behaviour change.

- **Pass:** `block_properties`, `lan`.
- **Oracle (gate):** `chunk_generation` (8 javac errors),
  `command_exec` (6 listed).
- **Fail:** `command_parse`, `component_patch`, `datafix`,
  `entity_components`, `first_person`, `food`, `hotbar`, `item_model`,
  `light`, `options`, `resource_pack`, `save_roundtrip`,
  `sound_event`.

Seven of the thirteen "fail" entries are really oracle-compile
failures whose javac errors land in separate compile logs the gate's
classifier does not read, so the gate's oracle count (2) understates
the true one (9):

| Harness | True cause | Errors |
|---|---|---|
| `chunk_generation` | oracle compile (`ChunkPos.x/z`, `ChunkStatus.TERRAIN`, `NoiseGeneratorSettings.create`) | 8 |
| `command_exec` | oracle compile (`ContainerInput` missing, etc.; the classifier lists 6, the compile log has 71) | 71 |
| `datafix` | oracle compile (`VanillaDataFixOracle`) | 26 |
| `first_person` | oracle compile | 25 |
| `food` | oracle compile | 9 |
| `item_model` | oracle compile | 49 |
| `options` | oracle compile | 1 |
| `resource_pack` | oracle compile | 1 |
| `sound_event` | oracle compile | 3 |

The remaining six fails are real verdicts or harness/environment
causes:

- `command_parse`: real verdict — 7,546 parse mismatches
  (32,319/32,321 commands match at permission 0).
- `component_patch`: the oracle crashes at runtime.
- `entity_components`: harness-side Python bug (the entity predicate
  is a list in 1.21.8; `TypeError: list indices must be integers`).
- `hotbar`: real verdict — 891/905 cases match.
- `light`: real verdict — "light mismatch against vanilla"
  (missing motion tag).
- `save_roundtrip`: the game exits before opening the control socket
  — no display on this headless VM (the same environmental failure
  as the 26.3 baseline).

Datafix reached its true verdict this time without a re-run:
`HORIZON_JAVA_25` was set from the start (unlike 1.21.8's first
sweep), and the fixture worlds were seeded from the 1.21.8 run's
finished worlds — 6 generated fresh (1.8.9–1.13.2), 35 reused. The
worlds are candidate-independent inputs (generated by old-version
servers), so the reuse is exact. Trap: the finished worlds live in
`worlds/` (1.17+) and `worlds-pre-1.17/`; the `gen/` directories are
work dirs the generator ignores — seeding `gen/` does nothing.

Two sweep lessons for the next agent:

- `tools/interaction_lab.py:cargo_example` hardcodes `cargo run -j3`,
  ignoring `CARGO_BUILD_JOBS`/`FLEET_LOAD_JOBS`. On this VM the
  debug `client_probe` build OOM-killed `horizon-client-render`'s
  rustc at `-j3`. The fix was to build the debug probe manually at
  `-j2` with parity's profile env (`CARGO_PROFILE_DEV_OPT_LEVEL=3`
  — without it the fingerprints differ and the harness rebuilds
  everything), then resume the sweep.
- `tools/parity.py` re-runs **all** selected harnesses on every
  invocation (longest-first by recorded seconds); it does not skip
  recorded ones. A killed sweep repeats the whole set, not just the
  remainder. Three runtime restarts killed runs 1–3 here; run 4 is
  the profile.

## What remains, by cause

### Behaviour work from the diff (not started)

The 5 ranked items are the port's work list; **0 of 5 are closed**.
The work is the pin-adjacent behaviour (the `SharedConstants` change
and the save-format step, both rank 1/2) — small, but unattempted.
The behaviour code in this tree is still the 26.3 base's.

### Harness-oracle era ports

The per-harness oracles carry their era gaps forward from 1.21.8
unchanged (chunk-generation, command-exec, first-person, food,
item-model, datafix, options, resource-pack, sound-event) — the
same 9 oracle failures as 1.21.8, with the same error counts.

### Full harness profile

The CI `port-verify` workflow on main completes the profile on GitHub
runners.

## Cost experiment

Same-protocol hotfix delta, recorded for contrast with 1.21.9's
same-protocol delta (~65 min) and 1.21.8's cross-protocol delta
(~10 h 40 min). Times are from session timestamps (VM clock, UTC).

- **Wall clock:** work started 2026-11-12 ~14:32 UTC; the
  determinism regen finished 19:13 UTC. Total to that point:
  ~4 h 41 min. (Publish commit and final wall clock to be recorded
  after pushing — see the follow-up correction, as for 1.21.8.)
- **Work items closed vs total:** 0 / 5 behaviour items closed. The
  port closed the infrastructure layer only: pin, tables,
  verification. No behaviour item was attempted.
- **Harness profile:** 2 pass / 2 oracle / 13 fail of 17 (gate
  classification; true oracle count 9 — see above).
- **Sweep minutes:** run 4 (the profile) 24 m 25 s. Runs 1–3 were
  killed (two runtime restarts, one deliberate OOM stop) and their
  ~2 h 30 min of partial sweeps was repeated work, not profile.

### Where the time actually went (approximate)

| Activity | Time |
|---|---|
| Setup + prepare / diff / report | ~10 min |
| Table regen (4 attempts — flaky downloads, dangling symlink, branch-vs-tree traps, disk quota — then the full run) | ~28 min |
| Verification (workspace check 12 min, protocol tests) | ~15 min |
| Harness runs (run 4 profile 24 min; runs 1–3 partial ~2 h 30 min; manual debug-probe build ~17 min) | ~3 h 10 min |
| Determinism regen (first attempt killed by disk quota; retry 6 min) | ~15 min |
| Behaviour-item porting | 0 min |
| Publish (docs, notes, audit, commit) | ~30 min |

The contrast with 1.21.9 (~65 min) is not the delta — 1.21.7's delta
is smaller (5 items vs 27) — it is the VM tax: three runtime
restarts, a disk-quota kill, and the `-j3` OOM in the harness's probe
build. The mechanical work itself (prepare, regen, verify) took
under an hour, as the delta size suggests.

## Status

Pin set (protocol 772, world 4438); 2 of 6 tables regenerated, 4
reproduce byte-for-byte; no packet or adapter change (same protocol);
workspace compiles (0 errors, 0 warnings); protocol tests 430 pass /
2 fail (sandbox-network pair only); prepare verified ok end-to-end.
Behaviour port unstarted (0/5). Harness profile: 2 pass / 2 oracle /
13 fail of 17 measured locally (true oracle count 9; the other six
fails are real verdicts or environmental). CI port-verify to confirm.
