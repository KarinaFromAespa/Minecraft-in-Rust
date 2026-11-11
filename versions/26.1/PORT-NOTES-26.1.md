# Port notes: Minecraft 26.1

This directory is a port of the upstream tree (base commit `962f5949ac`, which
pins Minecraft 26.3) to Minecraft 26.1, produced with the tree's own
version-port pipeline (`tools/version_port.py`). The initial port was an
honest partial; this file now describes the state after a completion pass
that closed its four documented blockers. What remains is listed at the end.
Nothing in this file is a claim that the port is finished.

## Version facts

- 26.1: protocol 775, world version 4786, data pack 101.1, resource pack 84.0, Java 25.
- 26.3 (the base): protocol 777, world version 5023, data pack 121.0, resource pack 97.1.
- Diff 26.3 -> 26.1: 3281 classes changed (18452 methods), 261 classes moved,
  1151 removed and 450 added going backward; blocks -118, items -152; command
  nodes +297 -680. The pipeline's report lists 795 work items for this port.

## Regenerated tables (all six)

- `version_consts` -> `crates/horizon-core/src/version.rs`
- `packet_ids` -> `crates/horizon-protocol/src/ids.rs`
- `entity_tracking` -> `crates/horizon-client/src/lan/tracking.rs`
- `lan_command_order` -> `crates/horizon-client/src/lan/command_order.rs`
- `command_suggestions` -> `crates/horizon-client/src/lan/command_suggestions.rs`
- `vanilla_command_order` -> `crates/horizon-command/src/vanilla_order.rs`
  (new in this pass; it needed a building `client_probe` and the ported
  command-parse oracle)

## Completion pass — what closed

### 1. Protocol rewire (the main blocker)

`horizon-protocol` referenced five packets that do not exist in 26.1. Each
was determined from the 26.1 jar, its datagen reports and decompiled codecs,
and the code rewired to how 26.1 actually carries the behaviour:

- `SWING_ANIMATION` (clientbound): 26.1 has no such packet; swings are
  `animate` actions 0 (main hand) and 3 (off hand), with wake-up 2 and the
  critical hits 4 and 5. All send/receive sites (hand handling, LAN hub,
  session recording, remote players) now use `animate`.
- `PUNCH` (serverbound): replaced by 26.1's `swing` packet (a hand VarInt,
  0 main / 1 off), which is now a pinned packet variant with hub and bot
  handling. The off hand crosses too; it was dropped by the old design.
- `SPECTATOR_ACTION`: 26.1's equivalent is `spectate_entity`, whose payload
  is a bare entity-id VarInt (not a UUID — the jar is the authority here).
  It is now a pinned variant; the hub's spectator click sends/honours it,
  and a click on nothing sends no packet, as in 26.1.
- `POST_EFFECTS`: the feature does not exist in 26.1 — neither the packet
  nor a `/posteffect` command. The wire paths (hub sends, remote receive)
  are removed; the local post-chain state stays but nothing drives it from
  the network.
- `ADD_TRANSIENT_BLOCK`: likewise 26.3-only. The packet and its hub/remote
  paths are removed; an explosion that used to also send one now sends the
  block update alone. The local transient-block rendering module stays,
  inert.

Packet-codec corrections the pin exposed (all verified against the jar):

- `explode`: no trailing "sound plays" flag (26.3 added it). Hub-built
  explosions that must stay silent now name the intentionally-empty sound
  event, which is how 26.1 expresses a silent explosion.
- `accept_teleportation`: the id alone (26.3 added position/rotation). The
  hub confirms teleports by id and uses its own record of the target.
- `entity_position_sync`: 26.1's classic record layout (position, delta,
  rotations, on-ground) replaces 26.3's stepped `PositionPath`, which is
  deleted. Move-entity packets likewise use the classic short-delta layout;
  internally tracked steps are folded to their sum at the wire.
- `level_particles`: 26.1's field order (flags, position, spread, the one
  speed, the count as a fixed Int, the particle last) replaces 26.3's. The
  internal per-axis speed/randomization model is kept locally; at the wire
  the speed folds to its dominant component, the fold the down-adapter
  already used.
- `sign_update`: the front-text boolean sits right after the position,
  before the four lines (26.3 moved it to the end as a VarInt).
- `set_player_team`: the parameters are in 26.1's order (colour before the
  prefix/suffix).
- `login`: no `online_mode` field. The client's online-mode knowledge now
  comes from the session's own login exchange (the `Hello` packet's
  authenticate flag), which is where 26.1 carries it.
- `player_action` / `player_command` / `client_command` action numbers were
  audited against the jar and renumbered where 26.1 differs (digging
  abort/stop/drop/release/swap; the destroy-direction and stab actions do
  not exist in 26.1 — stab arrived in 26.2 appended at 7, the direction
  change only in 26.3).
- The two serverbound packets 26.1 has that the base lacked (`swing`,
  `spectate_entity`) are full pinned variants now (ids 63 and 62).
- `pinned_ids.rs` was regenerated for 26.1, and the particle data kinds in
  `entity_data.rs` were re-keyed to 26.1's `particle_type` registry order
  (the geyser particles a later release adds do not exist here).
- `tools/port_regen.py` learned the pre-split fallback the tracking
  generator already had: versions before the registry split decompile
  `EntityType`, not `EntityTypes`.

The whole workspace compiles (`cargo check --workspace`), and
`cargo test -p horizon-protocol` stands at **437 passed / 2 failed**; the
only failures are the two sandbox-network tests in code this port does not
touch. The 32 legacy-adapter failures the completion pass left are closed
(see "What remains" #1, resolved).

### 2. horizon-prepare exporter

The block-state exporter compiles and runs against 26.1, and the prepared
resources exist (`.local/horizon-client/26.1`, ~20k files):

- `VanillaBlockStateBindingOracle`: 26.1's mapper method is package-private
  (called reflectively now), the cuboid element's `shade()` is a boolean
  (not a direction), and the mob exporter uses `EntityType`. The binding
  export produces 1,170 blocks including both fake item frames, and every
  Java exporter lane in prepare succeeds.
- Rust side: `worldgen_bundles` no longer requires datapack resources for
  the three worldgen kinds that are code-only in 26.1 (`material_rule`,
  `material_condition`, `block_state_provider` exist in neither jar); the
  two jars must still agree. Model elements now read 26.1's boolean
  `shade`, which flows into baked quads and the baking fixtures.

### 3. Oracles ported

- `VanillaChunkGenerationOracle`: 26.1 splits terrain generation into
  NOISE/SURFACE/CARVERS steps (`fillFromNoise`/`buildSurface`/`applyCarvers`),
  `RandomState.create` takes its arguments in the other order, and the
  biome source is itself the resolver. A standalone dump run produces a
  complete chunk (307 KB, 384 layers + biome quads).
- `VanillaCommandExecOracle`: `EntityType` constants, the authlib
  `ServicesKeySet` package, `datapackWorldgen()`, the ten-argument server
  constructor, `teleport(TeleportTransition)`. It compiles clean.
- `VanillaBlockPropertiesOracle` (and the eight oracles sharing the
  helper): in 26.1 `BlockState.CODEC` *is* the full codec; the split into a
  compact `CODEC` plus `FULL_CODEC` came later.
- `VanillaCommandParseOracle`: recipes are not a registry in 26.1 — they
  live in the `RecipeManager`. Its `registries` mode runs to completion
  (988 KB, 1,491 learnable recipes); the `tree` mode feeds the regenerated
  `vanilla_command_order` table.
- Further oracle adjustments the prepare bundle required:
  `VanillaBlockLandingOracle`, `VanillaMovementRegistryOracle`,
  `VanillaSimpleBlockOracle`, `VanillaPlacementOracle` (fall/bounce hooks,
  code-only block state providers, levelgen class locations in 26.1).
- The codec split is the subtle one: the base's oracles export block states
  with `BlockState.FULL_CODEC`, the flat {id, properties} form the Rust
  fixtures consume. 26.1 has only the palette codec ({Name, Properties}),
  and substituting it compiles but silently changes every export's shape —
  the block-properties fixture then fails on "block state is missing id".
  A shared `VanillaStateCodec` helper now assembles the flat form from the
  registry and each state's property map (26.1's `StateHolder.getValues()`
  streams `Property.Value`s), and all ten oracles that exported through
  FULL_CODEC use it. The block-properties harness passes end-to-end with
  it (below).

### 4. The 26.2 neighbour

26.2 is the adjacent release and the only other unobfuscated one, so its
adapter tables could be regenerated against the new pin from its prepared
reports: `v26_2.rs` / `v26_1_2.rs` (packet ids) and `v26_2_ids.rs` /
`v26_1_2_ids.rs` (registry ids). Verified from the reports: 26.2 is a
strict superset of 26.1 for blocks, items and entity types (it only adds
the cinnabar/sulfur content and the sulfur cube), so no 26.1 content needs
a stand-in for a 26.2 client. Two payload conversions were added for 26.2
guests: `spectator_action` (an OptionalInt) becomes the pin's
`spectate_entity` (empty target = no packet), and a 26.2 stab
(`player_action` 7) is dropped, the pin having no stab.

## Harness profile

The subset is the same 17 harnesses the tree's docs use for the backward
experiment: block_properties, chunk_generation, command_exec, command_parse,
component_patch, datafix, entity_components, first_person, food, hotbar,
item_model, lan, light, options, resource_pack, save_roundtrip, sound_event.

Measured in this pass:

- **horizon-prepare: ok** (run through the pipeline's `harnesses` step).
  The initial port recorded it FAILED; the exporter ports above are what
  fixed it, and the prepared resources under `.local/horizon-client/26.1`
  are produced.
- The four oracles that recorded "oracle no longer compiles" or a compile
  failure in the initial port (chunk_generation, command_exec,
  block_properties, command_parse) now compile against 26.1, and were
  verified standalone: chunk_generation dumps a complete chunk,
  command_parse's `registries` mode completes (above), the block-state
  binding export runs (1,170 blocks).
- **block_properties: PASS** — re-run end-to-end in this pass (oracle
  export, Rust fixture, comparison), up from fail in the initial port.
- The remaining sixteen harnesses were not re-run to a recorded verdict in
  this pass: a full 17-harness run was killed after 71 minutes on the
  porting machine and a focused five-harness re-run also elapsed its
  window inside the parity stage (each harness compiles the client
  fixtures and boots JVMs; this machine is slow — the single passing
  block_properties re-run alone took 7 minutes). Their initial-port
  verdicts stand except where this pass re-measured: the two
  "oracle no longer compiles" verdicts (chunk_generation, command_exec)
  and block_properties' failure are resolved as described above, and
  prepare is green. A full parity re-run on a faster machine is the
  remaining measurement, not a known code blocker.

## What remains

1. **RESOLVED — cross-version adapters for 1.20.2–1.21.11 re-based to
   the 26.1 pin.** The mappings front-end wiring was lifted from the
   1.21.11 tree into this tree's `tools/version_port.py`,
   `tools/lab.py` and `tools/test_rust_parity.py` (the EntityType
   pre-split fallback in `port_regen.py` kept), all 11 legacy versions
   were prepared through it (1.20.2, 1.20.3, 1.20.6, 1.21.1, 1.21.3,
   1.21.4, 1.21.5, 1.21.6, 1.21.8, 1.21.10, 1.21.11), and the packet-id
   and registry-id tables were regenerated against protocol 775. The
   three versions with no datagen packet report (1.20.2, 1.20.3, 1.20.6)
   had their `packets.json` synthesized from the decompiled
   `ConnectionProtocol`/`GameProtocols` registration order, validated
   exact against 1.21.11's real report; rename arbitration between old
   and pinned packet names came from inverting the published adapter
   tables against the pinned report. The re-base reached the production
   conversions the tables exposed (`item.rs` component constants,
   `item/skip.rs`, the hash dispatches, the `v1_20_3`/`v1_21_4`/`v1_21_5`
   layout code, tooltip-flag lists) and the fixtures, which were
   re-keyed by registry name, with the pinned item-stack hexes
   regenerated through the conversion itself. Two pin facts the
   26.3-keyed tests had baked in: 1.21.11 and 26.1 share identical
   entity and block-entity registries (so no 1.21.11 entity is unknown
   to the pin), and bed still exists as a block entity in 26.1 (id 25;
   it was only removed in 26.3), and the 5002-era sign
   `allow_op_features` datafix does not run at world version 4786.
   `cargo test -p horizon-protocol` is now **437 passed / 2 failed**
   (only the sandbox-network pair, out of scope).
2. **26.1.2** shares protocol 775 with 26.1, so `from_protocol` resolves it
   to the pin; its adapter tables exist (regenerated above) for explicit
   selection, but nothing distinguishes it on the wire.
3. The old `step_26_1` (26.2↔26.1 layout step) was removed: under this pin
   a 26.1 peer is the pin itself, and 26.2's layout deltas are handled by
   `layout_26_2` plus the two conversions listed above.

## CI profile — port-verify v5, run 43062079306 (2026-11-11)

The full 17-harness profile the completion pass deferred to CI is now
measured on GitHub runners, against this tree (candidate commit e27b20a).
The route itself is proven: store prepare, lab-prepare from a staged
launcher directory, protocol baseline, and the harness subset all executed
end to end; `prepare: ok`; protocol compiles clean (5 warnings, 0 errors);
horizon-prepare completed in 30.7 s.

**Profile: 3 pass / 0 oracle-no-longer-compiles / 14 fail.**
Passes: block_properties, food, lan. (The "14 newly failing" count is a
no-baseline classification artifact — no baseline was passed to the run.)
Sampled failure causes, all specific and diagnosable, none a route defect:
chunk_generation and light fail on a missing `minecraft:blocks_motion_in_heightmap`
block tag in the prepared data; command_exec's oracle reflects on an
`ENTITY_COUNTER` field 26.1's jar does not have; datafix reports 19,372
differences over 7 worlds; four harness logs mention timeouts. These are
the next behaviour/oracle work for this tree, not verification gaps.
Artifacts kept at `~/workspace/mcir-ci/26.1v5/`.
