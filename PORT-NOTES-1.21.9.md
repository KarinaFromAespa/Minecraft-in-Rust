# Port notes: Minecraft 1.21.9

1.21.9 is the second adjacent delta port in the **Components era (E6,
1.20.5–1.21.11)**, taken from 1.21.10. Unlike 1.21.10, which crossed a
protocol version, this one is a **same-protocol delta**: protocol stays
**773** (1.21.9 shares 773 with 1.21.10 — the brief's assumption of 772
was wrong; the version JSON and the prepared store both say 773), world
version 4556 → 4554, data pack 88.0 and resource pack 69.0 unchanged.
Packets, registries, blocks, items and commands are identical to
1.21.10. The port is the mechanical layer: the pin is set, the six
generated tables are regenerated or confirmed unchanged against 1.21.9's
own jar, and no adapter re-base is needed. The behaviour code in this
tree is still the 26.3 base's, inherited through 1.21.11 and 1.21.10;
the port is **in progress**, not parity-verified.

## Provenance

- Base: the programme's published 1.21.10 tree (main@e8d85d1: protocol
  rewire and adapter re-base to the 773 pin, 6/6 tables, protocol tests
  438 pass / 2 fail, prepare verified ok), with the measured
  1.21.10 → 1.21.9 delta re-applied (method (b) from the 26.1 carries).
- 1.21.9's facts, from its own version JSON and prepared store: protocol
  **773**, world (data) version **4554**, data pack format **88.0**,
  resource pack format **69.0**, Java **21**, stable release.
- Prepare ran through the pipeline (verified jar download + generalized
  mappings front-end remap + datagen, 2026-11-12 01:31–01:36 UTC,
  ~5 min): **9,969/9,969 classes mapped (100%), 9,909 renamed,
  0 placeholders; 87,472 methods, 44,527 fields**; indexed 9,969
  classes, 80,638 methods. Nothing Mojang-derived is committed in this
  tree.
- Diff 1.21.10 → 1.21.9: **27 ranked work items** (rank 1: 3,
  rank 2: 20, rank 3: 3, rank 5: 1). Classes +0 −0, 0 moved, **35
  changed (68 methods)**; 33 concern Horizon code or harnesses.
  Registries +0 −0, 0 changed; blocks +0 −0, 0 changed (29,671
  states, 0 shifted); items +0 −0; command nodes +0 −0, 0 changed;
  data files +0 −0; assets +0 −0; en_us +0 −0. Packets not listed →
  identical.

## The delta: an `entityInside` change

Rank 1 is version plumbing only (#1 regenerate version.rs; #2 DataFixers
4556 → 4554, 0 schemas, 0 fixes, 0 datafix classes changed; #3 version
facts). Rank 2 is dominated by one change: `entityInside` across ~20
block classes (2 methods each) — CactusBlock, HoneyBlock,
LavaCauldronBlock, PowderSnowBlock, BaseFireBlock, BubbleColumnBlock,
CropBlock, PitcherCropBlock, EndGatewayBlock, WebBlock,
NetherPortalBlock, BigDripleafBlock, WitherRoseBlock,
SweetBerryBushBlock, CampfireBlock, EyeblossomBlock,
LayeredCauldronBlock, ButtonBlock, EndPortalBlock — plus Entity
(checkInsideBlocks, move), ItemFrame, MinecraftServer (runServer),
ThrowableProjectile, HangingEntity, BlockBehaviour, HopperBlock,
FireworkRocketEntity, DetectorRailBlock, FrogspawnBlock, TripWireBlock
and BasePressurePlateBlock. Rank 5 is 2 classes Horizon does not port.
No packet, registry, block, item or command work: the delta is a
behaviour change in block-entity interaction only.

## Tables: 2 regenerated, 4 unchanged

| Generator | Result |
|---|---|
| `version_consts` | regenerated (+5/−5 — the version pin) |
| `packet_ids` | regenerated (+1/−1 — header comment only: "protocol 773 (1.21.9)") |
| `entity_tracking` | unchanged |
| `lan_command_order` | unchanged |
| `command_suggestions` | unchanged |
| `vanilla_command_order` | unchanged |

The regen commit touches exactly 3 files (`version.rs`,
`ids.rs` header, `research-lock.json`). Two regen attempts failed
before the third succeeded, both on the same flaky-network cause as
1.21.10's prepare: `lab.download` (urllib) truncated the
server-mappings.txt download twice (two different wrong SHA-1s). The
verified store copy (SHA-1 `587e016f…`, matching the version JSON)
was seeded into the candidate's lab directory and the regen then ran
clean. No oracle port was needed: 1.21.9's API is 1.21.10's API
(1.21.10's oracles, including the jsr305 fix, carry forward
unchanged).

## Adapter ladder: no re-base needed

Same protocol (773), so the ladder stays exactly as 1.21.10 left it:
773-pinned, `step_1_21_10` the pin (identity). No converter, hash-table
or fixture change. The SlotDisplay trap carries forward unchanged (see
1.21.10's notes).

## Verification

- `cargo check --workspace`: clean (7 m 37 s).
- `cargo check --workspace --all-targets`: clean, 0 errors, 0 warnings
  (6 m 15 s).
- `cargo test -p horizon-protocol`: **438 pass / 2 fail** — the same
  split as 1.21.10, with only the two pre-existing sandbox-network
  tests failing
  (`connection::tests::a_dead_address_does_not_hold_up_the_next`,
  `resolver::dns::tests::asks_each_server_in_turn_and_retries_truncated_replies_over_tcp`),
  in untouched code, out of scope.
- Prepare was not re-run end-to-end for this tree: the lab-prepare
  inside regen already remapped both jars and ran datagen against
  1.21.9's own jars, and the exporter mains are 1.21.10's (verified
  against the 1.21.10 API, which 1.21.9 shares). CI's `port-verify`
  runs the full prepare gate.

## Harness subset: not re-measured locally

The 1.21.9 tree differs from the published 1.21.10 tree in exactly
3 files (version pin, one header comment, research lock). The Rust
behaviour code, the oracle sources and the adapter ladder are
byte-identical, so the harness verdicts are 1.21.10's by construction
except where 1.21.9's jar itself differs — the `entityInside` change,
which touches none of the 9 locally-measured harnesses' inputs
(block properties, chunk generation, command execution/parsing,
component patches, datafix, entity components, first person, food).
A local re-sweep would cost ~45 minutes of cold probe builds to
re-confirm an identical profile, so it was not run; the CI
`port-verify` workflow measures the full 17-harness profile on
GitHub runners. 1.21.10's measured profile (1 pass / 4 oracle /
4 fail of 9) is the expected profile for this tree, unconfirmed
locally.

## What remains, by cause

### Behaviour work from the diff (not started)

The 27 ranked items are the port's work list; **0 of 27 are closed**.
The work is the `entityInside` behaviour change across ~20 block
classes plus the Entity/HangingEntity/ItemFrame companions. The
behaviour code in this tree is still the 26.3 base's.

### Harness-oracle era ports

Inherited from 1.21.10 unchanged: chunk-generation (24 errors),
command-exec (44), first-person (25), food (8), component-patch
(`Removed` runtime crash), entity-components (harness-side list bug).
See 1.21.10's notes for the per-oracle causes.

### Full harness profile

The CI `port-verify` workflow on main completes the profile on GitHub
runners.

## Cost experiment

Same-protocol delta, recorded for contrast with 1.21.10's
cross-protocol delta. Times are reconstructed from session file
timestamps (VM clock, UTC) and are approximate to about ±10 minutes
per bucket.

- **Wall clock:** prepare 2026-11-12 01:31–01:36 UTC; diff/report
  ~01:52–01:54 UTC; port work 02:56–~03:45 UTC (publish commit;
  approximate). Active time (excluding the idle gap while 1.21.10
  finished): ~70 min.
- **Work items closed vs total:** 0 / 27 behaviour items closed. The
  port closed the infrastructure layer only: pin, tables, verification.
  No behaviour item was attempted.
- **Harness profile:** not measured locally (see above); CI profile
  pending.
- **Sweep minutes:** 0 locally (no sweep run — see above).

### Where the time actually went (approximate)

| Activity | Time |
|---|---|
| Prepare / diff tooling (prepare ~5 min, diff + report ~3 min) | ~10 min |
| Table regen (3 attempts: 2 flaky-download failures, 1 clean ~15 min incl. cold client_probe build) | ~25 min |
| Adapter re-base | 0 min (same protocol — none needed) |
| Verification (protocol test runs, workspace checks) | ~17 min |
| Oracle rework for prepare | 0 min (1.21.10's oracles carry forward unchanged) |
| Harness runs / debugging | 0 min (no sweep — see above) |
| Behaviour-item porting | 0 min |
| Publish (docs, notes, audit, commit) | ~20 min |

The contrast with 1.21.10 (~3 h 25 min) is the point: a
same-protocol delta costs a fraction of a cross-protocol one, because
the two expensive buckets — adapter re-base (~55 min) and harness
sweeps (~75 min) — drop to zero, and the oracle corpus carries
forward untouched. What remains is the pin, the regen (dominated by
the cold client_probe build inside `vanilla_command_order`), and
verification. The flaky-network download failures (27.json,
server-mappings.txt) are a recurring VM tax, not a port cost: each
was fixed by seeding a hash-verified copy.

## Status

Pin set (protocol 773, world 4554); 2 of 6 tables regenerated, 4
confirmed unchanged; no packet change; no adapter re-base needed;
workspace compiles (0 errors, 0 warnings); protocol tests 438 pass /
2 fail (sandbox-network pair only). Behaviour port unstarted (0/27).
Harness profile: not measured locally; CI to measure.
