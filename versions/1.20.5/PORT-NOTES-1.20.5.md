# Port notes: Minecraft 1.20.5

1.20.5 is the thirteenth adjacent delta port in the **Components era
(E6, 1.20.5–1.21.11)**, taken from 1.20.6 — and the floor of the
era: E5 (1.20.4 down) needs its own template decision. It is a
same-protocol delta: protocol **766** in both, world version 3839
→ 3837, data pack 41.0 and resource pack 32.0 unchanged — so the
pin is a version bump only. Packets, registries, blocks, items and
commands are identical, and no packet rewire or adapter re-base
was needed. The measured delta is **12 ranked work items**. The
port is the mechanical layer: the pin is set, the two stale
generated tables are regenerated against 1.20.5's own jar, the
other four are confirmed unchanged, the verification gates pass,
and the remaining work is sized, ranked and documented. The
behaviour code in this tree is still the 26.3 base's, inherited
through 1.21.11, 1.21.10, 1.21.9, 1.21.8, 1.21.7, 1.21.6, 1.21.5,
1.21.4, 1.21.3, 1.21.2, 1.21.1, 1.21 and 1.20.6; the port is **in
progress**, not parity-verified.

## Provenance

- Base: the programme's completed 1.20.6 tree — the `port/1.20.6`
  branch at 8312f6a4, which contains the 1.20.6 tree including the
  1.21.8 code changes (packet rewire, adapter re-base, oracle
  port), the 1.21.1 rewire, re-pin and oracle port, the 1.21
  version bump, the 1.20.6 cross-protocol rewire, re-pin and
  oracle port, the jsr305 jar, the both-forms
  `gen_version_consts.py`, the jsr305 classpath fix in
  `tools/port_regen.py`, and the full ported oracle. No overlay
  was needed. Published 1.20.6: main@d571f888.
- 1.20.5's facts, from its own version JSON and generated tables:
  protocol **766**, world (data) version **3837**, data pack
  format **41.0**, resource pack format **32.0**, Java **21**,
  stable release, released 2024-04-23.
- Prepare ran clean from the Mojang manifest (2026-11-20):
  mappings: classes **8,205 mapped / 8,205 total (8,178
  renamed)**, methods **72,178 mapped / 72,178 total (55,030
  renamed)**, fields **37,285 mapped / 37,285 total (37,055
  renamed)**, 0 placeholders; datagen indexed 8,205 classes /
  66,538 methods. Nothing Mojang-derived is committed in this
  tree.
- 1.20.5's datagen has no packet report provider, as at 1.20.6:
  `packets.json` was produced by `PacketReport1205.java` (the
  1.20.6 program, renamed — it reads each ProtocolInfo codec's
  IdDispatchCodec.byId), and is byte-identical to 1.20.6's.
- Diff 1.20.6 → 1.20.5: **12 ranked work items** (rank 1: 3,
  rank 2: 7, rank 3: 1, rank 5: 1). Classes +0 −1, 0 moved,
  **12 changed (12 methods); 9 of the changed or removed ones
  are ported or proved by Horizon**. Registries +0 −0, 0
  changed; blocks +0 −0, 0 changed, states 26684 → 26684, 0
  blocks' state ids shifted; items +0 −0, 0 with changed
  default components; command nodes +0 −0, 0 changed; data
  files +0 −0 ~1,190; assets +0 −0; en_us keys +0 −0.

## The delta: a version bump

Rank 1 is the version pin (`version.rs`), the save-format item
(DataFixers 3839 → 3837: 0 schemas, 0 fixes added, 2 datafix
classes changed) and the version facts (world 3839 → 3837).
Rank 2 is 7 harness reruns (`block_items.rs`, `remote.rs`,
`save.rs`, `item/hash/raw.rs`, `block_entity_anim.rs`,
`chunk/block_entity.rs`, widely named `net/minecraft`).
Rank 3 is 1 item (`command_block.rs`). Rank 5 is 1 item (2
changed or removed vanilla classes Horizon doesn't port, in 2
packages).

There is no content change: no registry, block, item, particle,
sound, command or data-file difference between 1.20.6 and
1.20.5. The only generator the report marks stale is
`version_consts`.

## Tables: 2 regenerated, 4 verified unchanged

| Generator | Result vs 1.20.6 |
|---|---|
| `version_consts` | regenerated (+5 −5 — the pin: 1.20.5 / 766 / 3837 / 41.0 / 32.0) |
| `packet_ids` | regenerated (+1 −1 — header only: the version in the doc comment) |
| `entity_tracking` | unchanged |
| `lan_command_order` | unchanged |
| `command_suggestions` | unchanged |
| `vanilla_command_order` | unchanged |

The regen took three runs (commit 399bf6cf on `port/1.20.5`).
Run 1 was cut off by a service restart during the
`vanilla_command_order` client_probe build. Run 2 resumed
from the cargo cache and was OOM-killed at the probe link
under 3-stream load. Run 3 retried only
`vanilla_command_order` and confirmed it unchanged. A fresh
regen reproduces all six committed tables byte-for-byte —
see `regen-determinism.json`, no overrides.

Note for the next port: a `--only` regen run overwrites
`diff-OLD-NEW/regen.json` with just the retried generator.
The 1.20.6 publish shipped that partial file by mistake and
it had to be repaired on main (360717fe); the 1.20.5 file was
rebuilt from the run logs before publishing, with all six
generators and the stats verified against the regen commit.

## No rewire, no re-base, no code changes

Same protocol, same registries, same wire: the 1.20.6 adapter
ladder, packet codecs and component codec apply to 1.20.5
unchanged. The only tree changes in this port are the two
regenerated tables (version.rs +5/−5, ids.rs header +1/−1).
The protocol tests pass unmodified (see "Verification").

## Oracle check: zero fixes

`horizon-prepare compile-exporters` passes both sides with
**no oracle code changes**: the 1.20.6 oracles compile
unchanged against the 1.20.5 jars (only 12 vanilla classes
changed, 12 methods, none of them oracle API). Client 22.2 s,
server 32.8 s.

## Verification

- `cargo check --workspace --all-targets`: clean, 0 errors,
  0 warnings (at `-j 2`).
- `cargo test -p horizon-protocol`: **420 pass / 2 fail** (422
  total — identical to 1.20.6). The 2 failures are the same
  pre-existing sandbox-network pair in untouched code
  (`connection::tests::a_dead_address_does_not_hold_up_the_next`,
  `resolver::dns::tests::asks_each_server_in_turn_and_retries_truncated_replies_over_tcp`),
  out of scope.
- `horizon-prepare compile-exporters`: both sides pass (see
  "Oracle check").

## Harness subset: no local profile (skipped, CI to measure)

No local harness profile was measured. The 1.21.6 publish
attempted a local sweep and spent ~4 h 30 min on attempts that
produced no profile (the `fast`-profile horizon-client build
OOMs on this VM's 6 GB under 3-stream load); the programme has
since moved to measuring profiles on CI, and disk quota pressure
(~97G of ~100–103G on 2026-11-20) rules out the local
sweep's multi-GB build trees. The profile is to be measured by
the CI `port-verify` workflow on GitHub runners (16 GB) after
merge.

`status.json` records the harness section as 0/0/0 of 17 with
this note, and the CI run will fill it in.

## Known gaps (mechanical-layer limits, for the behaviour port)

All inherited unchanged from 1.20.6:

- **Arbitrary modifier ids and food use remainders are lossy
  through the pin's wire by construction**: the write derives
  a stable UUID from the id and the read falls back to
  `minecraft:<uuid>` (vanilla's own 1.21 datafix loses the
  name the same way); the pin's food has no remainder field.
- **Tooltip hiding is lost writing to 1.21.5+ servers**: the
  marker is dropped by `write_patch`'s `has` filter (no DOWN
  synthesis — the target's `tooltip_display` id differs by
  version and the generated `Ids` struct has no name→id lookup).
- **Per-component show flags don't round-trip through saves**:
  `components_tag` keeps emitting the synthesized 1.21.5-form
  `minecraft:tooltip_display` compound, which a 1.20.5 save
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
- **Entity-data PARTICLE path**: an unmapped particle still
  hits the −1/mod-sentinel collision (only LevelParticles
  drops).
- **1.21.2+ damage_resistant is dropped** rather than
  converting to the pin's fire_resistant unit.
- **1.21.2+ consume effects are LOST** when a newer server's
  food is expanded to the pin form (the older_layout FOOD arm
  uses defaults); instrument durability/description are dropped
  by the Up conversion; a 1.21.11 tool with creative=false
  cannot hash to its 1.21.11 value at this pin; predicates with
  non-empty matchers always resync (the pin cannot hash
  matchers it dropped).
- **Recipe-book sync gap** (1.21.1 single `recipe` packet vs
  split packets), bundle-local selection, tick-end-per-server-tick
  — documented behaviour gaps.
- **Blockstate binding overlap**: vanilla 1.20.5 assigns the
  missing model to a state claimed by two variants; the Rust
  `bind.rs` keeps the newer variant (1.21.4+ semantics). The
  oracle reports vanilla; the harness will flag the gap.
- **Cuboid light emission**: 1.20.5's parser ignores
  `light_emission` (no such field); the Rust parser rejects
  out-of-range values (1.21.2+ validation). The oracle reports
  light 0 for every element.

## What remains, by cause

### Behaviour work from the diff (not started)

The 12 ranked items are the port's work list; **0 of 12 are
closed**. The work is the save-format step (3839 → 3837: 0
schemas, 0 fixes, 2 datafix classes), the 7 harness reruns,
the 1 rank-3 item and the 2 unported classes — unattempted.
The behaviour code in this tree is still the 26.3 base's.

### Harness-oracle era ports

The per-harness oracles carry their era gaps forward from 1.20.6
unchanged — the same oracle failures as 1.20.6 are expected once
the profile is measured, but the counts are unmeasured for
1.20.5.

### Full harness profile

The CI `port-verify` workflow on main measures the profile on
GitHub runners. The local sweep was skipped (see above), so CI is
the first profile for this version, not a confirmation of a local
one.

### D6 playtest

Deferred by the owner ("leave the play test for later") — recorded
as deferred, never claimed.

## Status

Pin set (protocol 766, world 3837); 6 of 6 tables accounted
for (2 regenerated, 4 verified unchanged); no rewire, no
re-base, no code changes beyond the tables; oracles compile
unchanged against the 1.20.5 jars; workspace compiles (0
errors, 0 warnings); protocol tests 420 pass / 2 fail (the
sandbox-network pair); exporters compile on both sides.
Harness profile: **not measured locally** (skipped — CI to
measure). D6 deferred (owner). **E6 chain complete at the
floor**: 1.20.5 is the last E6 release; E5 needs its own
template decision.

End main island (worldgen-verify): vanilla 1.20.5's
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
