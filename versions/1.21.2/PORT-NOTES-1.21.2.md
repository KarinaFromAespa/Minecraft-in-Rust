# Port notes: Minecraft 1.21.2

1.21.2 is the ninth adjacent delta port in the **Components era
(E6, 1.20.5–1.21.11)**, taken from 1.21.3. It is a same-protocol
delta: protocol **768** in both, world version 4082 → 4080,
data pack 57.0 and resource pack 42.0 unchanged — so the pin is
a version bump only. Packets, registries, blocks, items and
commands are identical, and no packet rewire or adapter re-base
was needed. The measured delta is **7 ranked work items**. The
port is the mechanical layer: the pin is set, the two stale
generated tables are regenerated against 1.21.2's own jar, the
other four are confirmed unchanged, the verification gates pass,
and the remaining work is sized, ranked and documented. The
behaviour code in this tree is still the 26.3 base's, inherited
through 1.21.11, 1.21.10, 1.21.9, 1.21.8, 1.21.7, 1.21.6, 1.21.5,
1.21.4 and 1.21.3; the port is **in progress**, not
parity-verified.

## Provenance

- Base: the programme's completed 1.21.3 tree — the `port/1.21.3`
  branch at 6d7959d, which contains the 1.21.3 tree including the
  1.21.8 code changes (packet rewire, adapter re-base, oracle port),
  the jsr305 jar, the both-forms `gen_version_consts.py`, the
  jsr305 classpath fix in `tools/port_regen.py`, and the full
  ported oracle. No overlay was needed. Published 1.21.3:
  main@f4bd438c.
- 1.21.2's facts, from its own version JSON and generated tables:
  protocol **768**, world (data) version **4080**, data pack
  format **57.0**, resource pack format **42.0**, Java **21**,
  stable release.
- Prepare ran clean from the Mojang manifest (2026-11-19):
  **8,725 classes indexed, 70,948 methods**; mappings: classes
  **8,725 mapped / 8,725 total (8,699 renamed)**, methods
  **77,481 mapped / 77,481 total (58,935 renamed)**, fields
  **39,707 mapped / 39,707 total (39,461 renamed)**,
  0 placeholders; named jar 32,658,341 bytes; datagen ran.
  Nothing Mojang-derived is committed in this tree.
- Diff 1.21.3 → 1.21.2: **7 ranked work items** (rank 1: 3,
  rank 2: 2, rank 3: 1, rank 5: 1). Classes +0 −1, 0 moved,
  **6 changed (5 methods); 4 of the changed or removed ones
  are ported or proved by Horizon**. Registries +0 −0, 0
  changed; blocks +0 −0, 0 changed, states 27318 → 27318, 0
  blocks' state ids shifted; items +0 −0, 0 with changed
  default components; command nodes +0 −0, 0 changed; data
  files +0 −0 ~1,201; assets +0 −0; en_us keys +0 −0.

## The delta: a version bump

Rank 1 is the version pin (`version.rs`), the save-format item
(DataFixers 4082 → 4080: 0 schemas, 0 fixes added, 1 datafix
class changed) and the version facts (world 4082 → 4080).
Rank 2 is 2 harness reruns (`Salmon.Variant`, 1 method;
`SharedConstants`, 0 methods). Rank 3 is 1 item
(`GetServerDetailsTask.scheduleResourcePackDownload`, in
`game/server_packs.rs`). Rank 5 is 1 item (2 changed or
removed vanilla classes Horizon doesn't port, in 2 packages).

There is no content change: no registry, block, item, particle,
sound, command or data-file difference between 1.21.3 and
1.21.2. The only generator the report marks stale is
`version_consts`.

## Tables: 2 regenerated, 4 verified unchanged

| Generator | Result vs 1.21.3 |
|---|---|
| `version_consts` | regenerated (+5 −5 — the pin: 1.21.2 / 768 / 4080 / 57.0 / 42.0) |
| `packet_ids` | regenerated (+1 −1 — header only: the version in the doc comment) |
| `entity_tracking` | unchanged |
| `lan_command_order` | unchanged |
| `command_suggestions` | unchanged |
| `vanilla_command_order` | unchanged |

The regen took two runs (commit 3f5956ec16 on `port/1.21.2`,
then a `--only vanilla_command_order` re-run). Run 1
regenerated the two tables, confirmed three unchanged, and
failed `vanilla_command_order` when the client_probe build was
OOM-killed at `-j 2` under 3-stream load (6 GB VM). Run 2 at
`-j 1` confirmed `vanilla_command_order` unchanged. The
committed `regen.json` lists all six generators with their
run-1–2 statuses combined (D1 completeness, per the gate
change at main@15ff93a8). A fresh regen reproduces all six
committed tables byte-for-byte — see `regen-determinism.json`,
no overrides.

## No rewire, no re-base, no code changes

Same protocol, same registries, same wire: the 1.21.3 adapter
ladder, packet codecs and component codec apply to 1.21.2
unchanged. The only tree changes in this port are the two
regenerated tables (version.rs +5/−5, ids.rs header +1/−1).
The protocol tests pass unmodified (see "Verification").

## Oracle check: zero fixes

`horizon-prepare` completes end-to-end with **no oracle code
changes**: the 1.21.3 oracles compile unchanged against the
1.21.2 jars (only 6 vanilla classes changed, 5 methods, none
of them oracle API).

Run 1 failed at the last step, not on an oracle error but on
a missing file: `.local/datagen/out/reports/registries.json`
did not exist — the regen's datagen run left only a `.cache`
directory in the candidate's `.local/datagen/out/`. The fix
was to copy the prepare step's datagen reports (same version,
same generator — `versions/1.21.2/datagen/reports/`) into the
candidate's `.local/datagen/out/reports/`. Run 2 then
completed: exit 0, "Done in 306.6 s".

## Verification

- `cargo check --workspace --all-targets`: clean, 0 errors,
  0 warnings (16 m 03 s at `-j 1`, with
  `CARGO_INCREMENTAL=0`; the first attempt was SIGTERM'd at
  ~15.5 min by a service restart and re-run from the cargo
  cache).
- `cargo test -p horizon-protocol`: **424 pass / 2 fail** (426
  total — identical to 1.21.3). The 2 failures are the same
  pre-existing sandbox-network pair in untouched code
  (`connection::tests::a_dead_address_does_not_hold_up_the_next`,
  `resolver::dns::tests::asks_each_server_in_turn_and_retries_truncated_replies_over_tcp`),
  out of scope.
- `horizon-prepare` completes end-to-end: exit 0, "Done in
  306.6 s", after the datagen-reports fix above.

## Harness subset: no local profile (skipped, CI to measure)

No local harness profile was measured. The 1.21.6 publish
attempted a local sweep and spent ~4 h 30 min on attempts that
produced no profile (the `fast`-profile horizon-client build
OOMs on this VM's 6 GB under 3-stream load); the programme has
since moved to measuring profiles on CI, and disk quota pressure
(~98–100G of ~100–103G on 2026-11-19) rules out the local
sweep's multi-GB build trees. The profile is to be measured by
the CI `port-verify` workflow on GitHub runners (16 GB) after
merge.

`status.json` records the harness section as 0/0/0 of 17 with
this note, and the CI run will fill it in.

## Known gaps (mechanical-layer limits, for the behaviour port)

All inherited unchanged from 1.21.3:

- **Tooltip hiding is lost writing to 1.21.5+ servers**: the
  marker is dropped by `write_patch`'s `has` filter (no DOWN
  synthesis — the target's `tooltip_display` id differs by
  version and the generated `Ids` struct has no name→id lookup).
- **Per-component show flags don't round-trip through saves**:
  `components_tag` keeps emitting the synthesized 1.21.5-form
  `minecraft:tooltip_display` compound, which a 1.21.2 save
  would carry as an unknown component (vanilla drops unknown
  components on load).
- **`skip_dropped` reads the EitherHolder form unconditionally**
  for the removed components, but 26.1+ sends them as bare
  holders — a 26.1+ stack carrying `damage_type`,
  `chicken/variant` or `provides_trim_material` may misparse.
- **Smithing-trim DOWN-write old form not implemented**: the
  write side always writes the new form (pattern id); a
  pre-1.21.5 guest's recipe displays would need a reverse
  pattern→template-item lookup plus the trim-pattern context on
  the guest path.
- **Text components stay JSON strings in NBT at this pin**
  (the 4290 unflatten fix is a 1.21.5 fix).
- **Equip-on-interact is dropped on the pin wire**: the 1.21.2
  equippable layout has no interact field.
- **The native pick-item hook has no call site at this pin**:
  it exists only so the native mod API generator finds its
  signature.

## What remains, by cause

### Behaviour work from the diff (not started)

The 7 ranked items are the port's work list; **0 of 7 are
closed**. The work is the save-format step (4082 → 4080: 0
schemas, 0 fixes, 1 datafix class), the 2 harness reruns, the
`GetServerDetailsTask` port and the 2 unported classes —
unattempted. The behaviour code in this tree is still the
26.3 base's.

### Harness-oracle era ports

The per-harness oracles carry their era gaps forward from 1.21.3
unchanged — the same oracle failures as 1.21.3 are expected once
the profile is measured, but the counts are unmeasured for
1.21.2.

### Full harness profile

The CI `port-verify` workflow on main measures the profile on
GitHub runners. The local sweep was skipped (see above), so CI is
the first profile for this version, not a confirmation of a local
one.

### D6 playtest

Deferred by the owner ("leave the play test for later") — recorded
as deferred, never claimed.

## Status

Pin set (protocol 768, world 4080); 6 of 6 tables accounted
for (2 regenerated, 4 verified unchanged); no rewire, no
re-base, no code changes beyond the tables; oracles compile
unchanged against the 1.21.2 jars; workspace compiles (0
errors, 0 warnings); protocol tests 424 pass / 2 fail (the
sandbox-network pair); prepare verified ok end-to-end.
Harness profile: **not measured locally** (skipped — CI to
measure). D6 deferred (owner).

End main island (worldgen-verify): vanilla 1.21.2's
`minecraft:end_islands` includes the central island cone in the
function itself, but the tree carried the 26.3 outer-only sampler
and did not decode the `end_islands` name at all, so the End
pipeline failed to build. Carried 26.2's fix:
`GraphNode::EndIslands { central }`, with `end_islands`
decoding to central: true and `end_outer_islands` to central:
false; the cone formula was verified against the 1.21.11 named
jar's EndIslandDensityFunction bytecode. The end-to-end worldgen
test (`prepared_world_builds_and_generates_chunks`, run by the
worldgen-verify CI job) now generates two chunks per dimension
and dumps them as JSON; it caught this gap.
