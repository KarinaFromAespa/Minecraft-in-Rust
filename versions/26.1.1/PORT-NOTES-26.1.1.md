# Port notes: Minecraft 26.1.1

This directory is a delta port of the programme's 26.1 tree to Minecraft
26.1.1, produced with the tree's own version-port pipeline
(`tools/version_port.py`: prepare, diff, regen, report). 26.1.1 is a patch
release on the same protocol as 26.1, and the delta is correspondingly
small: a DataVersion bump and three changed classes. The port inherits the
26.1 line's state — including, in this tree, its protocol-crate blocker;
see "Inherited state" below. Nothing in this file is a claim that the port
is finished.

## Version facts

- 26.1.1: protocol 775, world version 4788, data pack 101.1, resource pack 84.0, Java 25.
- 26.1 (the parent): protocol 775, world version 4786, data pack 101.1, resource pack 84.0.
- Diff 26.1 -> 26.1.1: classes +0 -0, 0 moved, 3 changed (2 methods, 2
  fields); packets, registries, blocks (29,873 states), items and command
  nodes all unchanged; data files +0 -0 with 1,202 changed — every one a
  structure file whose bytes differ only in the embedded DataVersion. The
  pipeline's report lists 6 work items for this port.

## Pipeline steps run

`prepare 26.1.1` (via the download path), `diff 26.1 26.1.1`,
`regen 26.1.1` with the 26.1 candidate as base, `report 26.1 26.1.1`. The
pipeline records (diff.json, regen.json, report.md) are working files and
are not part of this tree; their results are summarised here.

## Regenerated tables

- `version_consts` -> `crates/horizon-core/src/version.rs` (+5 -5): the
  version id and name, DATA_VERSION 4786 -> 4788, the build time.
- `packet_ids` -> `crates/horizon-protocol/src/ids.rs` (+1 -1): the header
  line only; the protocol is identical to 26.1's, no packet changed.
- `entity_tracking` -> regenerated, byte-identical (unchanged).
- `lan_command_order` -> regenerated, byte-identical (unchanged).
- `command_suggestions` -> regenerated, byte-identical (unchanged).
- `vanilla_command_order` -> FAILED, the inherited blocker below.

## The delta itself

- **DataVersion 4786 -> 4788, with no format change.** The report's
  DataFixers item records 0 schemas added, 0 fixes added, 0 datafix classes
  changed: the bump is a version stamp, and the 1,202 changed structure
  files differ only in the DataVersion written into their NBT.
- **`PlayerEntry.<init>` changed** (social interactions screen). The port
  in `crates/horizon-client/src/game/social.rs` still mirrors 26.1's
  version of that constructor: one method, ranked 3 by the report (ported
  code, no harness covers it). Flagged here, not ported in this delta.
- `DetectedVersion.createBuiltIn` and the `SharedConstants` version fields
  are version plumbing, fully absorbed by the `version_consts` regen.

## Inherited state

This delta was ported from the 26.1 candidate as published before that
line's completion pass, and its tree inherits that state:

- **`horizon-protocol` does not compile** against this line's regenerated
  packet ids: the crate references five packets that exist only in 26.3
  (SWING_ANIMATION, SPECTATOR_ACTION, PUNCH, POST_EFFECTS,
  ADD_TRANSIENT_BLOCK; 16 errors). Confirmed unchanged on 26.1.1: the
  `vanilla_command_order` generator's build fails with the same 16 errors
  (e.g. `PUNCH` and `SPECTATOR_ACTION` not found in `ids`), which is why
  that one table is not regenerated here.
- `horizon-prepare`'s block-state exporter does not compile against this
  line, so no prepared resources were produced for 26.1.1, and no harness
  run was made for this patch delta: it changes no protocol, registry,
  block, item or command surface. The 26.1 subset profile (17 harnesses:
  1 pass, 2 oracle-no-longer-compiles, 13 fail, 1 no verdict) stands as
  this version's inherited profile; see `versions/26.1/PORT-NOTES-26.1.md`.

Since this delta was ported, the completion pass has landed for 26.1
itself on `main` (the protocol rewire to the pinned 775 forms, all six
tables, the prepare exporter and the oracles). That work was done against
26.1's tree and is **not** in this directory; carrying it forward to
26.1.1 (whose protocol is identical) is follow-up work, as is the
`PlayerEntry` method above.
