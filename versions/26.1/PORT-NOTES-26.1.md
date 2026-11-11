# Port notes: Minecraft 26.1

This branch is a port of the upstream tree (base commit `962f5949ac`, which pins
Minecraft 26.3) to Minecraft 26.1, produced with the tree's own version-port
pipeline (`tools/version_port.py`). It is an honest partial port: the version
pin and five of the six generated tables are regenerated for 26.1, the shared
parity oracles are ported to the 26.1 API, and the remaining gaps are listed
below. Nothing in this file is a claim that the port is finished.

## Version facts

- 26.1: protocol 775, world version 4786, data pack 101.1, resource pack 84.0, Java 25.
- 26.3 (the base): protocol 777, world version 5023, data pack 121.0, resource pack 97.1.
- Diff 26.3 -> 26.1: 3281 classes changed (18452 methods), 261 classes moved,
  1151 removed and 450 added going backward; blocks -118, items -152; command
  nodes +297 -680. The pipeline's report lists 795 work items for this port.

## Pipeline steps run

`prepare` (both versions, via the download path), `diff 26.3 26.1`,
`regen 26.1`, `harnesses 26.1` on a subset, `report 26.3 26.1`. The pipeline
records (diff.json, regen.json, harnesses.json, report.md) are working files
and are not part of this tree; their results are summarised here.

## Regenerated tables

- `version_consts` -> `crates/horizon-core/src/version.rs` (+8 -8)
- `packet_ids` -> `crates/horizon-protocol/src/ids.rs` (+135 -143)
- `entity_tracking` -> `crates/horizon-client/src/lan/tracking.rs` (+17 -23)
- `lan_command_order` -> `crates/horizon-client/src/lan/command_order.rs` (+45 -303)
- `command_suggestions` -> `crates/horizon-client/src/lan/command_suggestions.rs` (+11 -22)
- `vanilla_command_order` -> FAILED (blocked by the protocol compile breakage below)

## Ported by hand

- `tools/generate_entity_tracking.py`: fall back to `EntityType.java`. 26.1
  predates the `EntityTypes` registry split — the entity-type registry lives in
  `EntityType` itself (157 `register(...)` entries parsed).
- The parity oracles (`mods/horizon/src/test/java/dev/horizonlab/parity`) are
  ported to the 26.1 API: `RegistryLayer.WORLDGEN` and
  `RegistryDataLoader.WORLDGEN_REGISTRIES` (renamed to `WORLD` /
  `WORLD_REGISTRIES` in a later version), the nine-argument
  `CommandSourceStack` constructor, and the non-generic four-argument
  `suggestRegistryElements` (the generic, filtered form came later).

## Harness profile

The subset is the same 17 harnesses the tree's docs use for the 26.2 backward
experiment: block_properties, chunk_generation, command_exec, command_parse,
component_patch, datafix, entity_components, first_person, food, hotbar,
item_model, lan, light, options, resource_pack, save_roundtrip, sound_event.

- horizon-prepare: FAILED. Its Java exporter does not compile against 26.1
  ("javac failed while exporting block states"), so the prepared resources
  under `.local/horizon-client/26.1` were never produced.
- 16 harnesses recorded a verdict: **1 pass** (food), **2 oracle no longer
  compiles** (chunk_generation, command_exec), **13 fail**.
- datafix: no verdict — the runner was killed during the harness's
  sample-world generation phase on the porting machine.
- No baseline run on the pinned 26.3 tree was made, so no newly-failing split
  is claimed: a harness listed as failing here may or may not have passed on
  the base.

Failure notes, where the logs pin them down:

- chunk_generation (oracle): `RandomState.create` signatures and
  `ChunkStatus` constants differ in 26.1.
- command_exec (oracle): the oracle references `EntityTypes`, which does not
  exist in 26.1.
- block_properties: the oracle compile step failed (javac exit 1).
- command_parse: the oracle compiles and runs, but throws in
  `exportRegistries` — a registry lookup fails under 26.1.
- component_patch: the oracle compiles; its run exits 1.
- The remaining failures recorded no oracle compile errors; the known dominant
  causes are the protocol compile breakage below and the missing prepared
  resources from the failed horizon-prepare.

## Blockers — what remains

1. `horizon-protocol` does not compile against the regenerated 26.1 packet
   ids: the code references five packets that do not exist in 26.1 —
   `SWING_ANIMATION`, `SPECTATOR_ACTION`, `PUNCH`, `POST_EFFECTS`,
   `ADD_TRANSIENT_BLOCK` (16 compile errors). These packets are woven through
   gameplay (packet handlers, the LAN hub, hand/swing handling, the spectator
   camera, post effects, transient blocks), and 26.1 carries the equivalent
   behaviour in other packets (a swing is an entity animation there), so this
   is feature-level rewiring, not a table fix. It also blocks the
   `vanilla_command_order` generator, which needs a building `client_probe`.
2. horizon-prepare's block-state exporter does not compile against 26.1, so no
   prepared client resources exist for this version yet.
3. More oracles need porting: chunk_generation, command_exec, block_properties,
   command_parse (above), and likely others the subset did not reach.
4. The two serverbound packets 26.1 has that the base dropped have ids in the
   regenerated table but no packet variants or handlers yet.

## Commits on this branch (over base 962f5949ac)

- `6b12c3f198` Entity tracking generator: fall back to EntityType.java on
  versions before the EntityTypes registry split
- `54289d3515` Port to Minecraft 26.1: pin it and regenerate (version_port.py regen)
- `419cc4e166` Port the parity oracles to the 26.1 API
- `a9deeb5ba0` Parity oracles: suggestRegistryElements is not generic in 26.1
- `50a22d19a7` Port to Minecraft 26.1: pin it and regenerate (command_suggestions)
- `5a7e083982` Port to Minecraft 26.1: pin it and regenerate (lan_command_order)
