# Port notes: Minecraft 26.1.2

This directory is a delta port of the programme's 26.1.1 tree to Minecraft
26.1.2 — the second step of the 26.1 patch chain
(26.1 -> 26.1.1 -> 26.1.2) — produced with the tree's own version-port
pipeline (`tools/version_port.py`: prepare, diff, regen, report). Like
26.1.1, 26.1.2 is a patch release on the same protocol; unlike 26.1.1, its
delta contains one behaviour-bearing change (the server-side attack
handler, below). The delta was first ported from the pre-completion 26.1
line; that line's completion pass has since been carried forward into
this tree (see "Carried-forward state" below), so this directory now
stands at the completed state with 26.1.2's own identity. Nothing in this
file is a claim that the port is finished.

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

The carry-forward ran `regen 26.1.2 --only vanilla_command_order` again on
the completed tree (below).

## Regenerated tables

- `version_consts` -> `crates/horizon-core/src/version.rs` (+5 -5): the
  version id and name, DATA_VERSION 4788 -> 4790, the build time.
- `packet_ids` -> `crates/horizon-protocol/src/ids.rs` (+1 -1): the header
  line only; the protocol is identical to 26.1.1's, no packet changed.
- `entity_tracking` -> regenerated, byte-identical (unchanged).
- `lan_command_order` -> regenerated, byte-identical (unchanged).
- `command_suggestions` -> regenerated, byte-identical (unchanged).
- `vanilla_command_order` -> `crates/horizon-command/src/vanilla_order.rs`:
  blocked in the original delta port (the inherited protocol blocker,
  below); regenerated in the carry-forward. The generator — the ported
  command-parse oracle against this version's own server jar, against the
  now-building `client_probe` — reproduced the table byte-identically, so
  the table in this tree is confirmed generator output for 26.1.2. All
  six tables now exist.

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
  flagged here, not silently absorbed. The flag is unchanged by the
  carry-forward: the completion pass's rework of `hand.rs` (the swing
  packet family) does not touch the server-side attack handler, and the
  changed method body remains unported — now the blocker it was waiting
  on is gone, so porting the method body is follow-up work in its own
  right.
- **Checkbox row limit.** `Checkbox` gained a `ROWS` field and an
  `overflowsRowLimit` method, `Checkbox$Builder.build()` changed with it,
  and `AbstractReportScreen.createFooter()` changed on the screen that
  renders those checkboxes. The checkbox pair is ported in
  `crates/horizon-client/src/dialog_screen.rs` with no harness over it
  (report item #6, rank 3); the report screen itself is not ported
  (item #7, rank 5). Neither change is ported in this delta. Both flags
  are unchanged by the carry-forward.
- `DetectedVersion.createBuiltIn` and the `SharedConstants` version
  fields are version plumbing, fully absorbed by the `version_consts`
  regen.

## Carried-forward state

This delta descends from the 26.1 candidate as published before that
line's completion pass, through 26.1.1, and inherited that state (see
`versions/26.1.1/PORT-NOTES-26.1.1.md` for the full statement). Since
then, the completion pass that landed for 26.1 itself on `main`
(`ed18214`: the protocol rewire to the pinned 775 forms, all six tables,
the prepare exporter and the oracles) has been carried forward into this
tree: the completed 26.1 code was taken as the base and this version's
own delta re-applied on top (research-lock, `version_consts`, the
`packet_ids` header, the front docs, these notes). The three versions
share protocol 775 with identical packets, registries, blocks, items and
commands, and the only file where the completion pass and this delta both
differed from the pre-completion base is `vanilla_order.rs`, settled by
the regeneration above. Measured in this tree after the carry:

- `cargo check --workspace` is clean (the five 26.3-only packets —
  SWING_ANIMATION, SPECTATOR_ACTION, PUNCH, POST_EFFECTS,
  ADD_TRANSIENT_BLOCK — are rewired to the pinned 775 forms exactly as in
  26.1; see `versions/26.1/PORT-NOTES-26.1.md` §1).
- `cargo test -p horizon-protocol` stands at **437 passed / 2 failed**,
  identical to 26.1's and 26.1.1's states after that line's adapter
  re-base (below) — not only the counts, but the same failing tests —
  with every failure accounted for: the 2 sandbox-network tests in code
  this port does not touch.
- The horizon-prepare exporter and the parity oracles are the ported
  (post-completion) ones. No prepared resources were produced and no
  harness run was made for this patch delta in the carry-forward — in
  particular the combat harnesses that cover the `handleAttack` change
  above were not run; the blocker that prevented them (the protocol
  crate not compiling) is now gone, so they are runnable follow-up work.
  26.1's post-completion subset profile (prepare ok; block_properties
  passes; the remaining harnesses not re-run to verdicts there) stands as
  this version's inherited profile; see `versions/26.1/PORT-NOTES-26.1.md`.

## Adapter re-base carried

The 1.20.2–1.21.11 cross-version adapter re-base that landed for 26.1
itself on `main` (`ff7d657`: the legacy packet/registry tables
regenerated against the 775 pin, plus the production re-bases the new
tables exposed in item components and entity data) has been carried
forward into this tree the same way as the completion pass: the current
26.1 tree was taken as the base and this version's own delta re-applied
on top. The assembly was verified two ways — against current 26.1 it
differs only in this version's identity set, and against the previously
published 26.1.2 tree it differs in exactly the re-base files (46, the
re-base commit's list less its 26.1 port notes). Measured in this tree
after the carry: `cargo check --workspace` clean; `cargo test
-p horizon-protocol` 437 passed / 2 failed (the sandbox-network pair —
the same two tests fail in 26.1 and 26.1.1). The 32 legacy adapter
failures recorded in the earlier carry are closed by this one.

Known, out of scope for this carry: `cargo check --workspace
--all-targets` fails in test code only (the library and binary targets
are clean — see the `cargo check --workspace` line above). Measured in
this tree: 42 errors in the `horizon-client-lib` test targets (8 in the
lib test, 34 in the `lan_bot` integration test) — the tests still build
26.3 packet shapes the completion rewire removed (`Punch`,
`SwingAnimation`, `AcceptTeleportation` position fields,
`EntityPositionSync.path`, `Login.online_mode`). The 6 errors in the
`horizon-client` lib test recorded for 26.1.1 (`Login.online_mode` in
`native_client_services`/`native_entity_refs`; the lan-particles
speed/randomization model) apply here identically: the failing test
files are byte-identical between the two trees. These breakages are
latent from the completion pass, which verified the library targets
only; fixing them is test-code work in its own right, not part of this
carry.

What remains for this version is whatever remains for the line (see
`versions/26.1/PORT-NOTES-26.1.md`); the flagged `handleAttack` method
body and checkbox/report-screen changes are closed by the behaviour port
below.

## Behaviour port, 2026-11-23

The 7-item ledger (`work-items.json`, derived from the pipeline report
`report-26.1.2.log`, sha256 `9880cf1f…`) is closed 7/7. Every behaviour
was checked against 26.1.2's own named jars in the version store
(`versions/26.1.2/jars/{server-game,client}.jar`) with `javap`; the jar
named in the brief (`leakage-audit/jars/26.1.2-server.jar`) is the Mojang
bundler jar and holds no game classes. The 26.3-leakage audit
(`tools/audit_26_3_leakage.py 26.1.2 --jar …/server-game.jar --baseline
tools/leakage-baselines.json`) exits 0 — no new leaks; the 23 leaks it
reports are baseline-known pre-existing ones, none in touched code.

- **#1 (rank 1) `vanilla_command_order` generator.** Closed by the
  carry-forward regen (byte-identical, above); the table is in the tree
  (`crates/horizon-command/src/vanilla_order.rs`). No behaviour change.
- **#2 (rank 1) DataFixers 4788 -> 4790.** A version stamp only: the
  report records 0 schemas added, 0 fixes added, 0 datafix classes
  changed, and the NBT spot check agrees — `bastion/blocks/air.nbt`
  decompressed from both jars differs in exactly one byte, the
  DataVersion int `0x12B4` -> `0x12B6`. The Rust side needs nothing:
  `datafix::CURRENT` is `DATA_VERSION` (4790) and the latest registered
  fix is 4773, so no fix runs in (4788, 4790]. No code change.
- **#3 (rank 1) Version facts.** `crates/horizon-core/src/version.rs`
  verified against the jar: 26.1.2, protocol 775, DATA_VERSION 4790,
  resource pack 84.0, data pack 101.1, Java 25, stable. No code change.
- **#4 (rank 2) `ServerGamePacketListenerImpl.handleAttack`.** The
  delta's one real behaviour item. `javap -c` on both server-game jars
  shows the ONLY change is the head guard: 26.1.1
  `if (!hasClientLoaded()) return;` becomes 26.1.2
  `if (!hasClientLoaded() || player.isSpectator()) return;` (+4 bytecode
  lines); the rest of the method is byte-identical (entity lookup,
  `resetLastActionTime`, the world-border check, `isWithinAttackRange`
  at 3.0, the `PIERCING_WEAPON` return, the invalid-entity disconnect,
  `isItemEnabled`, `cannotAttackWithItem`, `player.attack`). The
  report's `hand.rs` attribution is the range helper's home; the
  server-side handler is `guest_attack` in
  `crates/horizon-client/src/game/lan_actions.rs`, and it already
  returned early for spectators — but the hub (`lan/hub.rs`) updated the
  guest's `last_action_tick` for the Attack packet and queued the event
  before that guard ran, so a spectator spamming attacks kept their idle
  timer fresh and reached the anticheat, where vanilla returns before
  `resetLastActionTime`. The port moves the guard to vanilla's position:
  the `acted` match now counts an Attack as an action only for a
  non-spectator, and a new gate arm drops a spectator's Attack before
  the anticheat, the event and the move queue; `guest_attack`'s guard
  stays as the second line. Regression test
  `lan/hub_tests.rs::a_spectators_attack_reaches_nothing_and_resets_no_timer`:
  a spectator's Attack produces no `HostEvent::Packet` and leaves
  `last_action_tick` at 41 while the hub is at tick 100, and the same
  packet from a survival guest still reaches the game and resets the
  timer to 100.
- **#5 (rank 2) `SharedConstants`.** `WORLD_VERSION` 4788 -> 4790 is
  `DATA_VERSION` in `version.rs`; `SNAPSHOT_NETWORK_PROTOCOL_VERSION`
  304 -> 307 is snapshot-only and unused anywhere in the Rust tree. No
  code change.
- **#6 (rank 3) Checkbox row limit.** `javap -c` on both client jars:
  `Checkbox` gains `private static final int ROWS = 2` and
  `overflowsRowLimit(Font)` — `splitLines(message, width,
  Style.EMPTY).size() > 2` — and `Builder.build()` calls `setTooltip`
  only when it overflows (was unconditional). The constructor's
  `setMaxRows(2)` and `getDefaultWidth` (box 17 + spacing 4 + text
  width) are unchanged, and the Rust dialog checkbox layout already
  matches that width. The port adds `CHECKBOX_ROWS` and
  `checkbox_overflows_row_limit(font, label, width)` to
  `crates/horizon-client/src/dialog_screen.rs`, splitting at the
  widget's full width as vanilla does. No live caller yet: only
  `AbstractReportScreen` sets a tooltip on a Checkbox builder in
  vanilla, and the report screen is not ported; dialog checkboxes take
  the full label width, so the helper returns false for them. Regression
  test `dialog_screen::tests::a_checkbox_label_over_two_rows_gets_the_tooltip`:
  one and two rows stay within the limit, three rows (newlines) overflow,
  and three 8-wide sprite objects overflow at width 8 but not at 24.
- **#7 (rank 5) `DetectedVersion.createBuiltIn` +
  `AbstractReportScreen.createFooter`.** `createBuiltIn`'s only change
  is `sipush 4788` -> `sipush 4790` — version plumbing, absorbed by the
  `version_consts` regen. `createFooter` adds
  `.tooltip(Tooltip.create(ATTESTATION_CHECKBOX))` to the attestation
  checkbox builder chain; the report screen is not ported anywhere in
  the Rust tree, so this is documented, not coded.

Enabling work, not a behaviour item: the `horizon-client` lib test
target did not compile (the 6 errors this file already recorded under
"Adapter re-base carried"), which would have kept the two regression
tests above from ever running in CI. Fixed in test code only:
`lan_particles.rs` asserted the 26.3 particle model (`max_speed` as
`[f32; 3]`, a `randomization` field) — 26.1's packet has one speed and
no randomization mode, as the production code in the same file already
says; three `Login` literals in `native_client_services.rs` /
`native_entity_refs.rs` set the removed `online_mode` field. The
`LevelParticles` doc comment in `horizon-protocol` described the same
26.3 fields and is corrected. The 8 `horizon-client-lib` lib-test
errors and the 34 `lan_bot` errors recorded in the same paragraph are
untouched — they are in a suite this port's tests also compile under,
but the tests run in the `horizon-client` suite, so they stay
pre-existing work for their own fix.

