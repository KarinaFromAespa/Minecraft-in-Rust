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
own jar, and no adapter re-base is needed. The behaviour port is
**complete** (2026-11-19, Stream 3): all 27 diff items closed, verified
against the 1.21.10/1.21.9 jars — see "Behaviour port (2026-11-19)"
below. CI proof pending.

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

## Behaviour port (2026-11-19, Stream 3)

All 27 items closed, verified against the 1.21.10/1.21.9 named jars
(byte-level class comparison: 48 changed class files, 35 top-level;
decompiled diff of all 35, 439 lines). The delta is one cross-cutting
change plus three small ones:

- **1.21.10 added a moved-distance boolean to
  `BlockState.entityInside`** (5th argument: the entity moved more
  than ~1 block, or its deflated box intersects the block). 1.21.9
  calls the 4-arg form. 24 of the 25 changed block overrides ignore
  the boolean — signature-only. The one reader is
  `BubbleColumnBlock`, whose above/inside logic runs only when the
  boolean is true in 1.21.10, unconditionally in 1.21.9.
- **`Entity.move`**: 1.21.10 skips the stuck-speed multiplier for
  `MoverType.PISTON`; 1.21.9 applies it to every move.
- **`HangingEntity`/`ItemFrame`**: 1.21.10 added `getPopBox()` (the
  non-map box) for the survives/canCoexist checks; 1.21.9 uses
  `getBoundingBox()` (a block wide with a framed map, 0.75 without).
- **`MinecraftServer`**: 1.21.10 added a `scheduledPacketProcessing`
  step to the tick; 1.21.9 has none — and neither does the Rust port,
  so nothing to remove.

Code changes (7de0f6d, 6 files, +26/−38):

1. `horizon-world/src/entity_move.rs`: the stuck multiplier now
   applies to piston moves too (the `mover != Piston` guard removed).
2. `horizon-client/src/motion.rs` +
   `motion/environment.rs`: the player's bubble-column handling drops
   the `precise` gate (`moved_far`/`far` removed); every column block
   the sweep crosses pushes.
3. `horizon-client/src/projectiles.rs`: `bubble_columns_since` drops
   its `far || deflated.intersects(cell)` gate; every swept column
   pushes. (The mob/item path, `motion/fluids.rs`, was already
   ungated; `first_tick_bubble_column` is ungated in both versions.)
4. `horizon-client/src/entities.rs`: `ItemFrame::bounds()` is now a
   block wide with a framed map, 0.75 without — 1.21.9's
   `getBoundingBox`, used by survives/canCoexist/crosshair. The
   renderer's map expansion in `game/frames.rs` is now redundant and
   removed.

Everything else verifies as no-change: version.rs was already
regenerated by the mechanical publish (values match
`SharedConstants`/`DetectedVersion`: 4554, 773, packs 69/88); zero
datafix classes changed (the server-jar comparison shows only
`DetectedVersion` + `SharedConstants` + the same entity/block/server
classes — no datafix, no schemas); no stale 4556 in code; the
snapshot protocol (274 → 273) is unreferenced; `DetectedVersion` is
version-literal-only and `WaterlilyBlock` delegates to `super`
(rank 5); paintings are unaffected (the base `getPopBox` is the
bounding box). D3 passes locally (rank-1 3/3, rank-2 20/20).

Manifest pin: Mojang republished the 1.21.9 version manifest too
(live sha1 `61826f3f…`, pinned `1b06d291…`; jars byte-identical) —
accepted in `research-lock.json` (7de0f6d), verified with the real
rule (`lab.manifest_accepted`).

Tag audit (D1 sub-check, local run 2026-11-19): **33 dead production
tags** (inventory 539 tags, 16 registries, from the 1.21.9 jar's own
tag files; 6 dead test-only tags are deliberate fixtures). Four fewer
than 1.21.7's 37: 1.21.9's data defines `copper`, `copper_chests`,
`shearable_from_copper_golem` and `wooden_shelves`. The remaining 33
are the same inherited 26.3-isms (including the 7 triaged on 26.2);
not triaged here — the fixes belong at the 1.21.11 level, flowing
down the chain (see 1.21.7's notes). D1 will fail on CI until the
triage lands; locations are in the Stream 3 workdir
(`dead-tags-1.21.9.json`).

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

### Behaviour work from the diff (complete 2026-11-19)

All 27 ranked items closed (Stream 3); see "Behaviour port
(2026-11-19)" above. 4 code changes, the rest verified no-change
against the jars.

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
  ~01:52–01:54 UTC; port work 02:56–03:31 UTC (publish commit
  a1d6494). Active time (excluding the idle gap while 1.21.10
  finished): ~65 min. (Corrected 2026-11-12: the end time was
  estimated 15 minutes late before the commit landed.)
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
2 fail (sandbox-network pair only). Behaviour port complete 2026-11-19
(27/27, Stream 3). Tag audit: 33 dead production tags (inherited
26.3-isms, triage deferred to the 1.21.11 level). Harness profile: not
measured locally; CI to measure. D6 deferred (owner).
