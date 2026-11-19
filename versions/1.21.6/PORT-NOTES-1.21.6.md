# Port notes: Minecraft 1.21.6

1.21.6 is the fifth adjacent delta port in the **Components era (E6,
1.20.5–1.21.11)**, taken from 1.21.7. It is a cross-protocol delta:
protocol **772 → 771**, world version 4438 → 4435, data pack 81.0 →
80.0 and resource pack 64.0 → 63.0 — but the packet set is
**byte-identical** between the two releases (0 added, 0 removed,
0 renumbered), so there is no packet rewire: the cross-protocol work
is the adapter re-base to 771 alone. The measured delta is **31
ranked work items**. The port is the mechanical layer: the pin is
set, the six generated tables are regenerated against 1.21.6's own
jar, the adapter ladder is re-based to the new protocol, the
verification gates pass, and the remaining work is sized, ranked
and documented. The behaviour code in this tree is still the 26.3
base's, inherited through 1.21.11, 1.21.10, 1.21.9, 1.21.8 and
1.21.7; the port is **in progress**, not parity-verified.

## Provenance

- Base: the programme's completed 1.21.7 tree — the `port/1.21.7`
  branch at 9128f75, which contains the 1.21.7 tree including the
  1.21.8 code changes (packet rewire, adapter re-base, oracle port),
  the jsr305 jar, the both-forms `gen_version_consts.py`, the
  jsr305 classpath fix in `tools/port_regen.py`, and the full
  ported oracle (391 files). No overlay was needed: the branch is
  the completed tree, unlike the bare `port/1.21.8` branch the
  1.21.7 agent had to overlay. Published 1.21.7: main@be65dca7.
- 1.21.6's facts, from its own version JSON and generated tables:
  protocol **771**, world (data) version **4435**, data pack
  format **80.0**, resource pack format **63.0**, Java **21**,
  stable release (2025-06-17).
- Prepare was salvaged from the 2026-11-13 agent's partial work
  (closed on the owner's redirect, nothing published): the
  mappings front-end run was complete and usable (verified jar
  download + generalized mappings front-end remap + datagen):
  **9,483/9,483 classes mapped (100%), 9,424 renamed,
  0 placeholders; 83,775 methods mapped (62,737 renamed), 42,840
  fields mapped (42,403 renamed)**. The diff log was empty (the
  diff never ran), so the diff and report were re-run from the
  salvaged prepared data. Nothing Mojang-derived is committed in
  this tree.
- Diff 1.21.7 → 1.21.6: **31 ranked work items** (rank 1: 3,
  rank 2: 17, rank 3: 10, rank 5: 1). Classes +0 −1, 0 moved,
  **45 changed (73 methods); 29 concern Horizon**.

## The delta: a cross-protocol pin step

Rank 1 is the version pin (`version.rs`), the save-format item
(DataFixers 4438 → 4435: 0 schemas, 0 fixes added, 0 datafix
classes changed) and the version facts (protocol 772 → 771, world
4438 → 4435). Rank 2 is 17 harness reruns. Rank 3 is 10 unproved
items. Rank 5 is the 17 changed or removed vanilla classes Horizon
does not port, in 12 packages.

The visible content change is the removal of the **Lava Chicken**
music disc, a 1.21.7 addition: the registry loses the item
`minecraft:music_disc_lava_chicken` and the sound event
`minecraft:music_disc.lava_chicken`. Every item id above the
disc's slot shifts down by one, which is why the adapter re-base
and the test re-pins below are mechanical but wide. There is no
other registry, block, item or command change of note.

## Tables: 2 regenerated, 4 reproduce

| Generator | Result vs 1.21.7 |
|---|---|
| `version_consts` | regenerated (+8 −8 — the pin: 1.21.6 / 771 / 4435 / 80.0 / 63.0) |
| `packet_ids` | regenerated (+1 −1 — header comment only; the packet set is identical) |
| `entity_tracking` | unchanged |
| `lan_command_order` | unchanged |
| `command_suggestions` | unchanged |
| `vanilla_command_order` | unchanged |

The committed `regen.json` lists all six generators (D1 completeness,
per the gate change at main@15ff93a8): the final full regen ran with
no `--only` (2026-11-19, commit 7aaa91c49b). A fresh regen reproduces
all six committed tables byte-for-byte — see
`regen-determinism.json`, no overrides. (The brief expected the five
SlotDisplay hand-corrected tables to differ from a fresh regen; they
do not — the generators emit the same numbering the hand corrections
carry, as they did for 1.21.9, 1.21.8 and 1.21.7.)

Two traps from the regen, for the next agent:

- The first regen run was SIGTERM'd at ~37 min during
  `vanilla_command_order`'s client_probe build (a VM restart). The
  retry completed in ~13.6 min. Regen is idempotent; a killed run
  is safe to re-run.
- `version_port.py regen --worktree` takes the path as given: a
  **relative** worktree path breaks the vineflower jar lookup in
  `decompile_classes` (the command runs with cwd=root, so the
  relative jar path resolves inside the worktree). Always pass the
  **absolute** worktree path. The determinism regen hit this and
  failed in seconds; the absolute-path retry ran clean.

## Adapter re-base to 771

The adapter ladder (`crates/horizon-protocol/src/adapter/`) is
re-based from the 772 pin to 771 (commits 881498f, a0b30b3 on
`port/1.21.6`; scripts `rebase771.py` and `fix_slot_display771.py`
in the Stream 1 workdir, modelled on the 1.21.8 `rebase772.py`).
The rebase was recovered from diffing the regenerated tables
against the 772 tree — the committed `rebase772.py` is an
incomplete record of the 1.21.8 re-base, so the 771 maps were
re-derived rather than copied:

- INHERIT map: 1.20.2→1.20.3, 1.20.3→1.20.6, 1.20.6→1.21.1,
  1.21.1→1.21.3, 1.21.3→1.21.4, 1.21.4→1.21.5, 1.21.5→PIN,
  1.21.8→PIN; NAME_ALIASES for 1.20.2/1.20.3 (needed by
  `adapter/legacy.rs`).
- Renames: attributes (1.20.2: 14, 1.20.3: 14, 1.20.6: 22,
  1.21.1: 31 — 1.21.2 dropped the prefixes); items
  (grass→short_grass, scute→turtle_scute); block
  (grass→short_grass); entities (boat→oak_boat,
  chest_boat→oak_chest_boat, potion→splash_potion); particle
  (ambient_entity_effect→entity_effect); component
  (fire_resistant→damage_resistant); sound
  (entity.generic.wind_burst→entity.breeze.wind_burst).
- REGISTRY_REASSIGN (1.21.3 creaking_transient→creaking);
  STATE_ALIASES (1.21.3, nine creaking_heart aliases).
- Two DOWN hand-patches the generator cannot produce:
  v1_20_3 ENTITY_TYPE_DOWN insert (18, 113, 1) (breeze_wind_charge
  → wind_charge); v1_21_3 BLOCK_STATE_DOWN (2054, 2051, 866)→867
  + eight heart runs length 1→2.
- SlotDisplay: 8 entries in both 1.21.6 and 1.21.7, same order —
  the 772 fix values apply unchanged.
- EntityDataSerializers are NOT among the 45 changed classes, so
  the `adapter.rs` serializer remaps are unchanged. `adapter.rs`
  hand edits: "pinned 1.21.8" → "pinned 1.21.6" (5 comment sites),
  the V1_21_8/V1_21_6 static comments rewritten, the
  `from_protocol` test updated (772→V1_21_8, 771→Current).

The Lava Chicken removal shifts every item id above the disc's slot
down by one, so the protocol tests needed re-pins: the sentry trim
template 1354→1353 (×4), context item ids >1256 shifted −1, the
angler sherd 1372→1371 (incl. varint bytes 0xdc→0xdb), the blast
furnace station `SlotDisplay::Item(1291)` → `Item(1290)`, and four
pinned slot fixtures in `item/v1_20_3.rs` (crossbow 1274→1273,
goat_horn 1287→1286, stew 1275→1274, bees 1307→1306). The two
lava-chicken disc tests were rewritten from the 1.21.8 side
(pigstep 1267→1266, levelup sound 1180→1179, `up(1256)==None`,
`down(1256)==Some(1257)`).

## Verification

- `cargo check --workspace --all-targets`: clean, 0 errors,
  0 warnings (15 m 21 s at `-j 1`). The first run at `-j 2` was
  OOM-killed at ~43 min on the horizon-client lib rustc — the
  VM's 6 GB, shared with two other streams' concurrent rustc
  processes, cannot hold two heavy codegen jobs at once.
- `cargo test -p horizon-protocol`: **430 pass / 2 fail** (432 total —
  identical to 1.21.7/1.21.8, as expected: the packet set is
  byte-identical across the delta). The 2 failures are the same
  pre-existing sandbox-network pair in untouched code
  (`connection::tests::a_dead_address_does_not_hold_up_the_next`,
  `resolver::dns::tests::asks_each_server_in_turn_and_retries_truncated_replies_over_tcp`),
  out of scope.
- `horizon-prepare` completes end-to-end: exit 0. **No oracle
  compile errors** — the 1.21.7 oracles compile cleanly against
  the 1.21.6 jars with zero fixes (unlike 1.21.8, which needed
  7 files / 27 javac errors ported).

## Harness subset: no local profile (blocked, CI to measure)

No local harness profile was measured. The sweep could not
complete on this VM, and the profile is to be measured by the CI
`port-verify` workflow on GitHub runners (16 GB) after merge.

- Sweep attempt 1: prepare ok, but the sounds step failed writing
  a sounds asset object (disk quota; 432M already in
  `versions/1.21.6/launcher`), and parity died after the first
  harness saving `state.json` — "Disk quota exceeded". Fixed by
  deleting the candidate's `target/` (4.8G debug build; the check
  and tests were already recorded).
- Sweep attempt 2: prepare ok; parity recorded **10 FAILs** in
  `.local/parity-runner/state.json` (incl. `active_effect_tick`,
  `active_effects`, `ageable_integer_load` — all expected for
  unported behaviour). The 4th harness (`allay_cooldown`) builds
  the client (`cargo build --profile fast -p horizon-client --bin
  horizon-client`); the build hit parity's 3600s per-harness
  timeout and was killed. The exec then ended exit 120 at ~69 min
  — no code path in `version_port.py`/`parity.py` produces 120;
  the most likely cause is a runtime turn-boundary kill (a
  follow-up message arrived ~6 min before the exit). Cause
  unconfirmed.
- Four attempts to pre-build the client in `.local/parity-target`
  (`--profile fast`, `-j 1`) all failed on the same crate: the
  `horizon_client` lib crate (opt-level=3, codegen-units=16)
  needs ~3–3.2 GB RSS for its rustc, and the VM's 6 GB shared
  with two other streams' concurrent rustc processes cannot
  reliably hold it. Attempt 1: killed by a VM restart at ~45 min
  (at horizon-client-render). Attempt 2: OOM-killed at ~61 min
  (at the horizon_client lib crate). Attempt 3: reached 12:15 CPU
  on the lib crate, then its poll was aborted and its outcome was
  never recorded. Attempt 4: OOM-killed at ~7.5 min. Attempt 5
  (`taskset -c 0`): OOM-killed at ~16.8 min — affinity caps the
  codegen threads but the ~2.5 GB metadata floor dominates.
- A swapfile did not help: a 4G file was created and `mkswap`
  succeeded, but `swapon` failed "Operation not permitted"
  (container restriction); the file was removed. Existing swap is
  1 GB (`/dev/loop0`, ~426 MB used).
- There is no dev-profile escape: parity's client build is fixed
  to the `fast` profile by the recorded artifacts
  (`tools/parity.py` `build_commands` groups recorded artifacts
  by (profile, selection) and emits `cargo build --locked
  --profile <profile>`; the shared dependency records fix the
  client binary to `fast`). No flag selects dev.

The 10 recorded FAILs are 10 of 658 individual harnesses, not gate
harnesses; they are not presented as a profile. `status.json`
records the harness section as 0/0/0 of 17 with this note, and the
CI run will fill it in.

## What remains, by cause

### Behaviour work from the diff (not started)

The 31 ranked items are the port's work list; **0 of 31 are
closed**. The work is the Lava Chicken removal's behaviour
consequences, the 45 changed classes (29 concern Horizon), and
the 17 harness reruns — unattempted. The behaviour code in this
tree is still the 26.3 base's.

### Harness-oracle era ports

The per-harness oracles carry their era gaps forward from 1.21.7
unchanged — the same oracle failures as 1.21.7 are expected once
the profile is measured (chunk-generation, command-exec,
first-person, food, item-model, datafix, options, resource-pack,
sound-event), but the counts are unmeasured for 1.21.6.

### Full harness profile

The CI `port-verify` workflow on main measures the profile on
GitHub runners. The local sweep was blocked (see above), so CI is
the first profile for this version, not a confirmation of a local
one.

### D6 playtest

Deferred by the owner ("leave the play test for later") — recorded
as deferred, never claimed.

## Cost experiment

Cross-protocol delta with a byte-identical packet set, recorded
for contrast with 1.21.8's cross-protocol delta (~10 h 40 min,
108 packets renumbered) and 1.21.7's same-protocol hotfix
(~4 h 44 min). Times are from session timestamps (VM clock, UTC).

- **Wall clock:** work started 2026-11-19 ~01:32 UTC; end
  2026-11-19 ~09:15 UTC. Total: ~7 h 45 min.
- **Work items closed vs total:** 0 / 31 behaviour items closed.
  The port closed the infrastructure layer only: pin, tables,
  adapter re-base, verification. No behaviour item was attempted.
- **Harness profile:** none measured locally (blocked — see
  above); CI to measure.
- **Sweep minutes:** none productive. The two sweep attempts
  (~2 h) and the five client pre-build attempts (~2 h 30 min)
  produced no profile.

### Where the time actually went (approximate)

| Activity | Time |
|---|---|
| Setup + salvage / diff / report | ~15 min |
| Table regen (run 1 SIGTERM'd at 37 min; run 2 13.6 min) | ~1 h |
| Adapter re-base + protocol-test re-pins | ~2 h |
| Verification (workspace check 15 min at -j1 after a -j2 OOM; protocol tests) | ~1 h |
| Oracle check (horizon-prepare, zero fixes) | ~15 min |
| Harness attempts (2 sweeps + 5 client pre-builds, all blocked) | ~4 h 30 min |
| Determinism regen (first attempt killed by a runtime restart; retry ~29 min) | ~45 min |
| Behaviour-item porting | 0 min |
| Publish (docs, notes, audit, commit) | ~30 min |

The contrast with 1.21.8 (~10 h 40 min) is not the delta — 1.21.6's
delta is smaller (31 items vs 526) and needed no packet rewire —
it is the VM tax under 3-stream load: the horizon-client lib crate
cannot be built in the `fast` profile on 6 GB shared with two
other streams, so ~4 h 30 min went to harness attempts that
produced no profile. The mechanical work itself (salvage, regen,
adapter re-base, verify, determinism) took ~4 h 45 min, as the
cross-protocol shape suggests.

## Status

Pin set (protocol 771, world 4435); 2 of 6 tables regenerated, 4
reproduce byte-for-byte; adapter ladder re-based to 771, packet
set byte-identical (no rewire); workspace compiles (0 errors,
0 warnings); protocol tests 430 pass / 2 fail (the sandbox-network
pair); prepare verified ok end-to-end, oracles compile with zero
fixes. Harness profile: **not measured locally** (client build
OOM-blocked under 3-stream load); CI `port-verify` to measure
after merge. D6 deferred (owner).
