# Port notes: Minecraft 26.1.2

This directory is a delta port of the programme's 26.1.1 tree to Minecraft
26.1.2 — the second step of the 26.1 patch chain
(26.1 -> 26.1.1 -> 26.1.2) — produced with the tree's own version-port
pipeline (`tools/version_port.py`: prepare, diff, regen, report). Like
26.1.1, 26.1.2 is a patch release on the same protocol; unlike 26.1.1, its
delta contains one behaviour-bearing change (the server-side attack
handler, below). The port inherits the 26.1 line's state — including, in
this tree, its protocol-crate blocker; see "Inherited state". Nothing in
this file is a claim that the port is finished.

## Version facts

- 26.1.2: protocol 775, world version 4790, data pack 101.1, resource pack 84.0, Java 25.
- 26.1.1 (the parent): protocol 775, world version 4788, data pack 101.1, resource pack 84.0.
- Diff 26.1.1 -> 26.1.2: classes +0 -0, 0 moved, 6 changed (5 methods
  changed, 1 method and 1 field added); packets, registries, blocks
  (29,873 states), items and command nodes all unchanged; data files
  +0 -0 with 1,202 changed — every one a structure file whose bytes
  differ only in the embedded DataVersion. The pipeline's report lists
  7 work items for this port.

## Pipeline steps run

`prepare` (26.1.2 was already in the version store via the download path),
`diff 26.1.1 26.1.2`, `regen 26.1.2` with the 26.1.1 candidate as base,
`report 26.1.1 26.1.2`. The pipeline records (diff.json, regen.json,
report.md) are working files and are not part of this tree; their results
are summarised here.

## Regenerated tables

- `version_consts` -> `crates/horizon-core/src/version.rs` (+5 -5): the
  version id and name, DATA_VERSION 4788 -> 4790, the build time.
- `packet_ids` -> `crates/horizon-protocol/src/ids.rs` (+1 -1): the header
  line only; the protocol is identical to 26.1.1's, no packet changed.
- `entity_tracking` -> regenerated, byte-identical (unchanged).
- `lan_command_order` -> regenerated, byte-identical (unchanged).
- `command_suggestions` -> regenerated, byte-identical (unchanged).
- `vanilla_command_order` -> FAILED, the inherited blocker below.

## The delta itself

- **DataVersion 4788 -> 4790, with no format change.** The report's
  DataFixers item records 0 schemas added, 0 fixes added, 0 datafix
  classes changed: the bump is a version stamp, and the 1,202 changed
  structure files differ only in the DataVersion written into their NBT.
- **`ServerGamePacketListenerImpl.handleAttack` changed** — the
  server-side handler for the attack packet. It is ported in
  `crates/horizon-client/src/game/hand.rs` and is the one changed class
  the combat harnesses prove (report item #4, rank 2: server_player_combat,
  registered_player_combat, damage_immunity, attribute_updates and four
  more). The changed method body is **not** ported in this delta, and
  those harnesses cannot run on this line until the inherited protocol
  blocker clears. This is the delta's one real behaviour item; it is
  flagged here, not silently absorbed.
- **Checkbox row limit.** `Checkbox` gained a `ROWS` field and an
  `overflowsRowLimit` method, `Checkbox$Builder.build()` changed with it,
  and `AbstractReportScreen.createFooter()` changed on the screen that
  renders those checkboxes. The checkbox pair is ported in
  `crates/horizon-client/src/dialog_screen.rs` with no harness over it
  (report item #6, rank 3); the report screen itself is not ported
  (item #7, rank 5). Neither change is ported in this delta.
- `DetectedVersion.createBuiltIn` and the `SharedConstants` version
  fields are version plumbing, fully absorbed by the `version_consts`
  regen.

## Inherited state

This delta descends from the 26.1 candidate as published before that
line's completion pass, through 26.1.1, and its tree inherits that state
(see `versions/26.1.1/PORT-NOTES-26.1.1.md` for the full statement):

- **`horizon-protocol` does not compile** against this line's regenerated
  packet ids: the crate references five packets that exist only in 26.3
  (SWING_ANIMATION, SPECTATOR_ACTION, PUNCH, POST_EFFECTS,
  ADD_TRANSIENT_BLOCK; 16 errors). Confirmed unchanged on 26.1.2: the
  `vanilla_command_order` generator's build fails with the same 16
  errors, which is why that one table is not regenerated here.
- `horizon-prepare`'s block-state exporter does not compile against this
  line, so no prepared resources were produced for 26.1.2, and no harness
  run was made for this patch delta — in particular the combat harnesses
  that cover the `handleAttack` change above cannot be exercised until
  the blocker clears. The 26.1 subset profile stands as this version's
  inherited profile; see `versions/26.1/PORT-NOTES-26.1.md`.

Since this delta chain was ported, the completion pass has landed for
26.1 itself on `main` (the protocol rewire to the pinned 775 forms, all
six tables, the prepare exporter and the oracles). That work was done
against 26.1's tree and is **not** in this directory; carrying it forward
through 26.1.1 to 26.1.2 — whose protocol is identical — is follow-up
work, as are the `handleAttack` and checkbox changes above.
