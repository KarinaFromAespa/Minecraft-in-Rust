# Port notes: Minecraft 1.21.11

1.21.11 is the template version of the **Components era (E6, 1.20.5–1.21.11)**
and the first version in the programme whose jar is **obfuscated**: Mojang
stopped shipping unobfuscated builds before this era, and 1.21.11 is
specified through its official mapping tables instead. The era's gate was
that the pipeline run on such a release at all — **prepare, end-to-end,
through the mappings front-end**. That gate is met, and this tree is the
result: the pin is set, three of the six generated tables are regenerated
from 1.21.11's own jar, the distance to the neighbouring port is sized and
ranked, and the remaining work is itemised below by cause. The behaviour
code in this tree is still the 26.3 base's; the port is **in progress**,
not parity-verified.

## Provenance

- Base: the programme's 26.3 tree (upstream horizon-rust snapshot), pinned
  by `research-lock.json` to **1.21.11** by the pipeline's regen commit
  (version, manifest and jar hashes, Java major 21).
- 1.21.11's facts, from its own version JSON and prepare run: protocol
  **774**, world (data) version **4671**, data pack format **94.1**,
  resource pack format **75.0**, Java **21**, stable release.
- The specification sources were 1.21.11's client and server jars,
  deobfuscated with Mojang's official mapping tables by the programme's
  mappings front-end (`tools/mapping-frontend/` at the repository root).
  Nothing Mojang-derived is committed in this tree.

## The era gate: the front-end wired into prepare

Before this port, `prepare` refused obfuscated releases outright ("This
workflow targets unobfuscated releases. Older releases need a remapping
stage."). The wiring that removes that refusal is in this tree's tools and
is the reusable asset for every remaining obfuscated version:

- `tools/version_port.py` — `prepare` no longer refuses a release that
  ships mapping tables. After the verified download it runs the front-end
  on the client jar (the named jar becomes the prepared client artifact)
  and remaps the server game jar through the front-end's driver with the
  server table, so the store's inventory, class index and reports all read
  named classes.
- `tools/lab.py` — `prepare` verifies Mojang's checksums at import as
  before, then a remapping stage renames the imported client and server
  game jars in place. Every downstream stage (inventory, decompile,
  exporters, oracles) reads named classes with no further changes. The
  stage records its outputs — and the verified mapping tables they came
  from — in `remapped.json`, and `tools/test_rust_parity.py`'s artifact
  authentication accepts exactly those recorded digests alongside Mojang's
  checksums, so a remapped jar remains authenticated end-to-end.
- The launcher tree keeps presenting Mojang's original jar (its checksum
  is verified against the manifest); the named jars live in the prepared
  store only.

Prepare's own numbers for 1.21.11, reproduced by both prepare paths (the
version-port store and the lab prepare inside the regen candidate):

- Client mappings: **10,291 of 10,291 classes mapped (100%), 10,229
  renamed**; the 62 identity mappings are `com/mojang/blaze3d` library
  classes the table leaves alone. Server table: 7,163 classes, 7,102
  renamed.
- Class index built from the named jar: **10,291 classes, 82,622 methods**
  (the lab inventory counts 89,557 methods), with no obfuscated class
  names anywhere in it.
- Server datagen ran on the unmodified bundler jar (its entry points keep
  their names) and produced the full report set: registries, blocks,
  items, packets, commands and the rest.

The same wiring is what unblocks regenerating the cross-version adapter
tables for 1.20.2–1.21.11 against their own jars (the residual failure
class recorded in the 26.1 port notes): their jars can now be prepared.

## The distance, sized: diff 26.1 → 1.21.11

The pipeline's diff against the nearest published port (26.1) produced
**815 ranked work items** (rank 1: 10, rank 2: 492, rank 3: 247, rank 4:
64, rank 5: 2). The headline shifts:

- Protocol **775 → 774**. Play packets: clientbound 141 → 139 (+0, −2,
  100 renumbered), serverbound 69 → 66 (+0, −3, 65 renumbered).
- Block states 29,873 → 29,671; blocks +0 −2 with 51 changed; items show
  −1,506 by registry id (id-space churn across the components break), with
  **0 items' default components changed** per the diff's component
  comparison — sizing the break honestly: the wire and id layers moved
  far more than the component defaults did.
- 20 registries changed (largest: `sound_event` −64 entries; item and
  block id spaces shifted by ~1,275 and ~1,008 respectively); command
  tree +6 −29 nodes; data files +100 −826; asset files +93 −248;
  `en_us` keys −119.
- Classes +213 −604 (102 moved), 4,154 changed classes covering 14,306
  methods; 3,310 of the changed/removed methods were already ported or
  proved by the base code.

## Regeneration

Against the 26.3 base pin, regen produced the pin commit plus:

| Generator | Result |
|---|---|
| `version_consts` | regenerated (`crates/horizon-core/src/version.rs`, +9 −9) |
| `packet_ids` | regenerated (`crates/horizon-protocol/src/ids.rs`, +179 −197) |
| `entity_tracking` | regenerated (`crates/horizon-client/src/lan/tracking.rs`, +17 −23) |
| `lan_command_order` | failed — parity-oracle compile (below) |
| `command_suggestions` | failed — parity-oracle compile (below) |
| `vanilla_command_order` | failed — parity-oracle compile (below) |

`entity_tracking` needed the port's first era adaptation: 1.21.11 predates
the `EntityTypes` registry split, so the decompile step takes `EntityType`
(the class that held the registry) and the generator reads either source —
the same fix the 26.1 port made, lifted into this tree's
`tools/port_regen.py` and `tools/generate_entity_tracking.py`.

## What remains, by cause

### The parity oracles do not compile against 1.21.11's registry bootstrap

All three remaining generators drive Java oracles
(`VanillaCommandParseOracle`, `ParityData`) that bootstrap the vanilla
registries the way 26.3 does. They now authenticate the (remapped)
artifacts and pass the era's renames — `RegistryLayer.WORLD` →
`WORLDGEN`, `RegistryDataLoader.WORLD_REGISTRIES` → `WORLDGEN_REGISTRIES`
(both applied in this tree) — and stop at the semantic differences:

- `RegistryDataLoader.load(...)` has a different shape in 1.21.11: the
  networked-registry-data parameter the 26.x bootstrap passes does not
  exist in this version's API.
- `BuiltInRegistries.DATA_COMPONENT_INITIALIZERS` and the
  `net.minecraft.core.component.DataComponentInitializers` class do not
  exist in 1.21.11 (verified against its named jar): component defaults
  are initialised by a different mechanism here, and the oracles'
  bootstrap has to be reworked for it, not renamed into it.
- The structure-template loader package
  (`world.level.levelgen.structure.templatesystem.loader`) moved.
- `CommandSourceStack`'s constructor and the
  `suggestRegistryElements` signature differ (the 26.x permission rework
  postdates this version).

Until the oracles compile, the three command-table generators cannot run,
and the parity harness suite has not been run for this version.

### Behaviour work from the diff (not started)

The 815 ranked items are the port's work list. Largest first: the play
packet renumbering and the two/three removed packets per direction, the
registry id-space shifts (item, block, sound), and the 4,154 changed
classes. The behaviour code in this tree is still the 26.3 base's —
`horizon-protocol` carries the regenerated 1.21.11 packet-id table but not
yet the protocol rewire this version needs, the same feature-level step
the 26.1 and 26.2 notes record at their publish points — so the workspace
is not expected to compile as a 1.21.11 client yet.

### horizon-prepare and the exporter bundle (not attempted)

The `horizon-prepare` exporter bundle was not run against the named jar in
this pass; the 26.1/26.2 notes show it is its own adaptation (exporter
sources compile against the version's APIs). It is the natural next step
after the oracle bootstrap rework, since both turn on the same registry
APIs.

## Status

Era opened and prepare-proven; pin + 3 of 6 tables regenerated; diff sized
at 815 ranked items; oracle bootstrap rework is the named blocker for the
rest. Status in [VERSIONS.md](../../VERSIONS.md): **in progress**.
