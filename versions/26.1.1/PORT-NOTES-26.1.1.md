# Port notes: Minecraft 26.1.1

This directory is a delta port of the programme's 26.1 tree to Minecraft
26.1.1, produced with the tree's own version-port pipeline
(`tools/version_port.py`: prepare, diff, regen, report). 26.1.1 is a patch
release on the same protocol as 26.1, and the delta is correspondingly
small: a DataVersion bump and three changed classes. The delta was first
ported from the pre-completion 26.1 tree; the 26.1 line's completion pass
has since been carried forward into this tree (see "Carried-forward state"
below), so this directory now stands at the completed state with 26.1.1's
own identity. Nothing in this file is a claim that the port is finished.

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

The carry-forward ran `regen 26.1.1 --only vanilla_command_order` again on
the completed tree (below).

## Regenerated tables

- `version_consts` -> `crates/horizon-core/src/version.rs` (+5 -5): the
  version id and name, DATA_VERSION 4786 -> 4788, the build time.
- `packet_ids` -> `crates/horizon-protocol/src/ids.rs` (+1 -1): the header
  line only; the protocol is identical to 26.1's, no packet changed.
- `entity_tracking` -> regenerated, byte-identical (unchanged).
- `lan_command_order` -> regenerated, byte-identical (unchanged).
- `command_suggestions` -> regenerated, byte-identical (unchanged).
- `vanilla_command_order` -> `crates/horizon-command/src/vanilla_order.rs`:
  blocked in the original delta port (the inherited protocol blocker,
  below); regenerated in the carry-forward. The generator — the ported
  command-parse oracle against this version's own server jar, against the
  now-building `client_probe` — reproduced the table byte-identically, so
  the table in this tree is confirmed generator output for 26.1.1. All
  six tables now exist.

## The delta itself

- **DataVersion 4786 -> 4788, with no format change.** The report's
  DataFixers item records 0 schemas added, 0 fixes added, 0 datafix classes
  changed: the bump is a version stamp, and the 1,202 changed structure
  files differ only in the DataVersion written into their NBT.
- **`PlayerEntry.<init>` changed** (social interactions screen). The port
  in `crates/horizon-client/src/game/social.rs` still mirrors 26.1's
  version of that constructor: one method, ranked 3 by the report (ported
  code, no harness covers it). Flagged here, not ported in this delta —
  the flag is unchanged by the carry-forward.
- `DetectedVersion.createBuiltIn` and the `SharedConstants` version fields
  are version plumbing, fully absorbed by the `version_consts` regen.

## Carried-forward state

This delta was originally ported from the 26.1 candidate as published
before that line's completion pass, and inherited its blockers. Since
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
  identical to 26.1's state after that line's adapter re-base (below),
  with every failure accounted for: the 2 sandbox-network tests in code
  this port does not touch.
- The horizon-prepare exporter and the parity oracles are the ported
  (post-completion) ones. No prepared resources were produced and no
  harness run was made for this patch delta in the carry-forward: it
  changes no protocol, registry, block, item or command surface. 26.1's
  post-completion subset profile (prepare ok; block_properties passes;
  the remaining harnesses not re-run to verdicts there) stands as this
  version's inherited profile; see `versions/26.1/PORT-NOTES-26.1.md`.

## Adapter re-base carried

The 1.20.2–1.21.11 cross-version adapter re-base that landed for 26.1
itself on `main` (`ff7d657`: the legacy packet/registry tables
regenerated against the 775 pin, plus the production re-bases the new
tables exposed in item components and entity data) has been carried
forward into this tree the same way as the completion pass: the current
26.1 tree was taken as the base and this version's own delta re-applied
on top. The assembly was verified two ways — against current 26.1 it
differs only in this version's identity set, and against the previously
published 26.1.1 tree it differs in exactly the re-base files (46, the
re-base commit's list less its 26.1 port notes). Measured in this tree
after the carry: `cargo check --workspace` clean; `cargo test
-p horizon-protocol` 437 passed / 2 failed (the sandbox-network pair).
The 32 legacy adapter failures recorded in the earlier carry are closed
by this one.

Known, out of scope for this carry: `cargo check --workspace
--all-targets` fails in test code only (the library and binary targets
are clean — see the `cargo check --workspace` line above). Measured in
this tree: 42 errors in the `horizon-client-lib` test targets (8 in the
lib test, 34 in the `lan_bot` integration test) — the tests still build
26.3 packet shapes the completion rewire removed (`Punch`,
`SwingAnimation`, `AcceptTeleportation` position fields,
`EntityPositionSync.path`, `Login.online_mode`) — plus 6 errors in the
`horizon-client` lib test (`Login.online_mode` in
`native_client_services`/`native_entity_refs`; the lan-particles
speed/randomization model). These breakages are latent from the
completion pass, which verified the library targets only; fixing them
is test-code work in its own right, not part of this carry.

What remains for this version is the flagged `PlayerEntry` method, plus
whatever remains for the line (see `versions/26.1/PORT-NOTES-26.1.md`).

## Behaviour port (2026-11-23): all 6 items closed

No `work-items.json` existed for this version; the list was derived from
the pipeline report (`report.json`, sha256 `3ddd211c…`, in
`versions/26.1.1/work-items.json`) and every item verified against
26.1.1's own jars (named `server-game.jar` / `client.jar` from the
prepare; the leakage-audit server jars are obfuscated and cannot be used
for `javap` — they answer "class not found" for named classes).

1. **`vanilla_command_order` regen failure (rank 1).** Closed with no
   code change. The carry-forward had already regenerated the table
   (PORT-NOTES above); verified here by direct comparison:
   `crates/horizon-command/src/vanilla_order.rs` is byte-identical to
   26.1's, and the 26.1 -> 26.1.1 diff records 0 changed command nodes,
   so the same command tree yields the same order.
2. **DataFixers 4786 -> 4788 (rank 1).** Closed with no code change: the
   bump is a version stamp, as the report records (0 schemas, 0 fixes,
   0 datafix classes changed). The tree keys everything off
   `DATA_VERSION` (`datafix::CURRENT = crate::DATA_VERSION`,
   `horizon-world` re-exports `horizon-core`'s 4788), and no fix in the
   tree is keyed 4787 or 4788. Regression test
   `patch_delta_26_1_1_adds_no_fixes` (datafix/tests.rs): a 26.1 chunk
   upgrades with only its DataVersion stamp changed.
3. **Version facts 4786 -> 4788 (rank 1).** Closed with no code change:
   the `version_consts` regen already set `DATA_VERSION = 4788` and
   `VERSION_ID = "26.1.1"`. Grep over the tree finds no hardcoded 4786
   in production code (only a seed-hash constant and two launcher test
   fixtures holding old options text) and no stale `"26.1"` version
   string (only the 26.1 adapter's name and a polytone range test).
   Covered by the same datafix test, which asserts `CURRENT == 4788`
   and `VERSION_ID == "26.1.1"`.
4. **`SharedConstants` fields (rank 2).** Closed with no code change.
   javap on the named jars: `WORLD_VERSION` 4786 -> 4788 and
   `SNAPSHOT_NETWORK_PROTOCOL_VERSION` 303 -> 304. `getProtocolVersion`
   returns the constant 775 in both versions (it never reads the
   snapshot field), so the tree's `PROTOCOL_VERSION = 775` is already
   right; the snapshot constant is mirrored nowhere in the tree, in
   26.1 or 26.1.1. Regression test `version_facts_match_26_1_1`
   (horizon-protocol lib.rs). The 8 harnesses the report names were not
   re-run: they exercise blocks, particles, LAN auth and the like, none
   of which reads a version constant, and no prepared resources exist
   for this patch delta.
5. **`PlayerEntry.<init>` (rank 3).** Ported. The whole class differs by
   exactly one bytecode instruction: at constructor offset 106, 26.1
   has `ifne` and 26.1.1 has `ifeq`. The branch computes the boolean
   passed through the report button's onPress into
   `ReportPlayerScreen`'s `chatDisabledOrBlocked` field, which disables
   that screen's Report Chat button. 26.1 computed
   `canReceivePlayerMessages || isBlocked(uuid)` — reporting disabled
   exactly when messages could arrive; 26.1.1 computes
   `!canReceivePlayerMessages || isBlocked(uuid)`, matching the field
   name. This is a vanilla bug fix, not a feature. The tree does not
   offer the report flow at all (the row's report button is the
   disabled one vanilla shows when reporting is off, and the Blocked
   tab lists no one — same in 26.1 and 26.3), so the fix has no
   observable effect here; the formula is ported as
   `social_screen::chat_disabled_or_blocked` with the test
   `chat_disabled_or_blocked_follows_26_1_1` pinning the full truth
   table, so the report flow — when it lands — uses the 26.1.1 form.
6. **`DetectedVersion.createBuiltIn` (rank 5).** Closed with no code
   change. javap on the named jars: the only difference is the
   DataVersion literal, `sipush 4786` -> `sipush 4788`. Horizon does
   not port `DetectedVersion` (zero references in the tree); the
   constant is absorbed by the `version_consts` regen.

26.3-leakage audit (`tools/audit_26_3_leakage.py 26.1.1`, against the
26.1.1 server jar): 23 leaks, identical to the 26.1.1 baseline — 0 new,
none in any file this port touched. The baseline leaks are line-level
issues shared with 26.1, out of scope for this patch delta.

CI run 45141444453 (head bf68e269) confirmed the port's two runnable
tests pass (protocol version facts, datafix no-fixes) and exposed
pre-existing stale-version breakage the mechanical pass had left behind.
All of it was fixed on this branch, since it is the only way the port's
regression tests run at all:

- The `horizon-client` lib test target did not compile (6 errors): three
  `Login` literals still set `online_mode` (26.1's packet has no such
  field), and two `LevelParticles` assertions compared the scalar
  `max_speed` against a vec3 plus one read the removed `randomization`
  field (26.1's packet has one speed and no randomization mode).
  Fixed in `game/lan_particles.rs`, `game/native_client_services.rs`,
  `game/native_entity_refs.rs` (test code only).
- The `horizon-client-lib` lib test target did not compile (8 errors):
  `SwingAnimation`/`Punch` (26.3 packets) replaced by 26.1's
  `Animate`/`Swing` in `net.rs` and `lan/hub_tests.rs` (the hub's own
  code already routes `S::Swing`), `EntityPositionSync.path` replaced by
  the 26.1 field `position` in `lan/entity.rs`, and one more stray
  `online_mode` literal in `studio/replay.rs`.
- Five tests failed on stale 26.3-era fixtures: three world-upgrade tests
  used a 4899 world (from the future here; now 4786, the 26.1 version).
  That exposed a real defect: `requires_file_fixing` compared against
  the table's last entry (4899), so it claimed file fixing for every
  26.1 world even though no file fix applies between 4786 and 4788. It
  now checks for a fix in `(from, CURRENT]`, matching the `applicable`
  filter beside it;
  `block_states_take_their_new_field_names` pinned the `Name`->`id`
  palette rename, a fix keyed 5006 that postdates this version — it now
  asserts the 26.1.1 truth, that a 26.1 chunk keeps `Name`/`Properties`;
  and three pack tests declared 26.3's formats (resource 97/98, data
  121) — now 26.1.1's (resource 84.0, data 101.1).
- Found while fixing the palette test: `BlockState::from_nbt` read only
  the new `id`/`properties` form, so a real 26.1.1 save (which still
  writes `Name`/`Properties`) failed chunk loading with "block state is
  missing id". It now falls back to the old form, same class as the
  1.21.7 save-decoder fix; `to_nbt` still writes the new form only, so
  a port-written save is not one vanilla 26.1.1 reads (known gap,
  shared with 1.21.7).

The 34 `lan_bot` integration-test errors are untouched: crate-tests does
not run integration tests, and adapting that bot to 775 is a job of its
own, not part of this patch delta.
