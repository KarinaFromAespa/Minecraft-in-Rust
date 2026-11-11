# Port notes: Minecraft 26.2

This directory is the 26.2 port produced by the version-port pipeline
(`tools/version_port.py`, see `docs/version-ports.md`), ported backward from
the programme's 26.3 base. It is an honest partial port: the pin and five of
the six generated tables are 26.2's, the test oracles are adapted to 26.2's
API, and the behaviour work the diff ranked is itemised below with its
current state. Nothing in this file is a claim that the port is finished.

## Provenance

- Base: upstream Horizon at commit `962f5949ac1a8f789227614a75bcf360d0fb12a2`
  (pins Minecraft 26.3: protocol 777, data version 5023).
- This version pins Minecraft 26.2: protocol 776, data version 4903,
  resource pack 88.0, data pack 107.1, Java 25.
- Steps run: `prepare 26.3`, `prepare 26.2` (download path),
  `diff 26.3 26.2` (651 ranked work items: rank 1: 7, rank 2: 410,
  rank 3: 177, rank 4: 55, rank 5: 2), `regen 26.2`, oracle adaptations,
  `harnesses 26.2` over the 17-harness port set.
- No Mojang-derived material is committed: no jars, no mappings, no
  decompiled sources, no prepared data. The version store and the
  candidate's `.local` trees are local only.

## Regeneration

Five of the six generators regenerated for 26.2 (upstream's own backward
run of this port regenerated three):

| generator | result |
| --- | --- |
| version_consts | regenerated (version.rs +8 −8) |
| packet_ids | regenerated (ids.rs +134 −142) |
| entity_tracking | regenerated (tracking.rs +16 −21) |
| lan_command_order | regenerated (command_order.rs +45 −302) |
| command_suggestions | regenerated (command_suggestions.rs +11 −22) |
| vanilla_command_order | **failed** — the generator's probe (`client_probe`) does not build yet; see "Protocol" below |

## Adaptations already made

- **Parity oracles → 26.2 API**: `RegistryLayer.WORLD` → `WORLDGEN`,
  `RegistryDataLoader.WORLD_REGISTRIES` → `WORLDGEN_REGISTRIES`,
  `BlockState.FULL_CODEC` → `BlockState.CODEC` (26.2 has the single full codec),
  the 26.2 `CommandSourceStack` constructor (text name, display name, entity)
  and the non-generic `suggestRegistryElements` signature,
  `ServerChunkCache`'s extra `Supplier` parameter, authlib 9.0.75 names
  (`yggdrasil.ServicesKeySet`, `MinecraftSessionService`, `EnderMan`,
  `ServerboundSwingPacket`), `DataLoadContext.datapackWorldgen`,
  `ServerPlayer.teleport(TeleportTransition)`, `Pack.open()` returning one
  `PackResources`, `SoundInstance.resolve`, and the attenuation-distance
  product. Verified per file with javac: VanillaCommandParseOracle,
  VanillaCommandExecOracle, VanillaPackResourcesOracle, VanillaSoundEventOracle,
  VanillaLanAuthOracle, VanillaDragonContactsOracle and ParityData compile
  clean against 26.2; VanillaChunkGenerationOracle is down to the four
  `ChunkStatus.TERRAIN` errors listed below.
- **Chunk generation oracle, second pass**: `RandomState.create` argument
  order and `ChunkGeneratorStructureState.createForNormal` without the
  `ChunkPos` argument; the 26.3-only uncached biome resolver hooks are
  marked rather than faked, and the HORIZON_FLAT variant (whose flat-world
  `RandomState` factory is 26.3-only) now throws instead of silently
  mis-generating.
- **Serverbound arm-swing packet renamed** Punch → Swing in the protocol enum
  and every sender, handler and adapter reference (`SWING`, play serverbound
  63). `horizon-protocol`'s only remaining compile errors against the 26.2
  tables are the three clientbound packets below (8 sites).

## Harness profile

The port set (17 harnesses, 22 runs counting datafix's six shards) was run
in this tree. The pipeline records (diff.json, regen.json, harnesses.json)
are working files and are not part of this tree; their results are
summarised here. Two invocations were needed: the first was stopped during
`datafix`'s sample-world generation (see below), and the remaining
harnesses ran in a second invocation; the classifications were merged with
the pipeline's own classifier. No 26.3 baseline was run locally, so the
profile carries no before/after split.

- **prepare: failed** — the exporter bundle does not compile yet (see
  "horizon-prepare does not complete" below); that is later in the run
  than upstream's blocker, which was VanillaHeightmapOracle.
- **pass: 1** — `food`.
- **oracle (oracle no longer compiles): 1** — `chunk_generation`
  (the four `ChunkStatus.TERRAIN` references).
- **fail: 15** — `block_properties`, `command_exec`, `command_parse`,
  `component_patch`, `datafix`, `entity_components`, `first_person`,
  `hotbar`, `item_model`, `lan`, `light`, `options`, `resource_pack`,
  `save_roundtrip`, `sound_event`.

Per-harness notes:

- `block_properties`: the probe fixture command exits 1 — its
  resources are incomplete while prepare fails.
- `command_exec`: hit the 600 s harness timeout in this environment
  (its oracle compiles clean); no verdict on behaviour.
- `command_parse`: oracle compiles clean, then fails at runtime —
  `Registry minecraft:recipe not found` from the 26.2 reloadable
  registry loading sequence (work item below).
- `component_patch`: oracle compiles; its client-mode comparison
  exits 1.
- `datafix`: the sample-world generator downloads a historical server
  jar per version (38 versions); those downloads hang from this
  environment, so shards 1–2 recorded failed and shards 3–6 never
  started. Counted as fail; the blocker is environmental, on top of
  the real DataFixer work item below.
- `entity_components`: the harness driver itself raises a TypeError
  on 26.2's entity-component data shape.
- `first_person`: oracle rework needed (work item below).
- `hotbar`, `lan`, `options`, `resource_pack`: their Rust probe
  (`client_probe`) cannot build until horizon-protocol compiles
  (the three packets below); their Java oracles compile clean.
- `item_model`: its oracle also fails to compile
  (`CuboidModelElement.shadeDirectionOverride()`, the 26.3 model
  rework) — the classifier records it as fail.
- `light`: the oracle's authlib key fetch
  (api.minecraftservices.com) timed out from this environment.
- `save_roundtrip`: the vanilla login timed out on both attempts
  (server boot under load); no verdict on behaviour.
- `sound_event`: oracle compiles clean; the harness stops on a
  missing prepare-gated resource file.

Upstream's own backward run of this port (their published status)
ended at 1 pass, 4 oracle-compile failures (chunk_generation,
command_exec, command_parse, hotbar) and 12 fails, with prepare
blocked earlier, at VanillaHeightmapOracle. After the adaptations
above, three of those four oracles compile again; the remaining
oracle failure is chunk_generation's terrain re-staging.

## What remains, by cause

### Protocol: three clientbound packets 26.2 does not have

`horizon-protocol` does not compile yet: the regenerated 26.2 table has no
`SWING_ANIMATION`, `POST_EFFECTS` or `ADD_TRANSIENT_BLOCK` ids, and each packet
is woven into client behaviour, so removing one is a design decision, not a
rename:

- `ClientboundSwingAnimationPacket` is 26.3's swing split. In 26.2, swings
  animate from `ClientboundAnimatePacket` (id 2) plus local state; the swing
  kinds and durations the packet carries have no wire form. Consumers:
  `game/hand.rs`, `game/lan.rs`, `game/lan_actions.rs`, `game/remote.rs`,
  `lan/containers.rs`, `net.rs`, the session adapter. This is the diff's
  rank-1 family (with the `minecraft:swing_animation` component split: in 26.2
  items carry `attack_animation` / `interact_animation` instead, and
  `LivingEntity.SwingState` / `SwingDescription` do not exist — the
  first_person and held-item oracles need the same rework).
- `ClientboundPostEffectsPacket`: in 26.2 post effects are client-local; the
  server (LAN host) cannot request chains. Consumers: `game/remote.rs`,
  `lan/hub.rs`. Related: `SharedSuggestionProvider.getAvailablePostEffects`
  does not exist in 26.2 (the chat-screen oracle's delegation was removed and
  marked).
- `ClientboundAddTransientBlockPacket`: transient blocks are 26.3-only.
  Consumers: `game/remote.rs`, `game/falling_damage.rs`, the session adapter
  (whose tests already note the packet is "new in 26.3: not sent").

### horizon-prepare does not complete

The exporter bundle fails to compile against 26.2 in five files. Outputs that
do not need the bundle are produced (registries.json, items.json,
commands.json, block-tags.json, biome parameters, the data generator's
reports, the creative inventory), but every exporter step — starting with
the block-state facts — waits on the bundle:

- `VanillaBlockLandingOracle` (5 errors): 26.3 extracted `Block.bounceOn`
  from `fallOn` and added the shelf mushroom; the oracle's whole bounce axis
  is 26.3-shaped. In 26.2 bouncing lives in `fallOn` /
  `updateEntityMovementAfterFallOn`.
- `VanillaSimpleBlockOracle` (13) and `VanillaPlacementOracle` (8): the
  worldgen feature/placement API restructure (feature codecs,
  `BLOCK_STATE_PROVIDER` registry, `DIRECT_CODEC`, placement modifier
  access), the same family as the audit's density/material oracles.
- `VanillaMovementRegistryOracle` (1): `Block.getFallDistanceReduction()` is
  26.3-only; 26.2 has no per-block fall-reduction value to export.
- `VanillaCuboidModelOracle` (1): `CuboidModelElement.shadeDirectionOverride()`
  is part of 26.3's model/material rework (see item_model below).

### Harness oracles that need a rework (outside the prepare bundle)

- chunk_generation: `ChunkStatus.TERRAIN` / `NoiseBasedChunkGenerator.buildTerrain`
  merge 26.3's NOISE + SURFACE + CARVERS stages; the oracle's terrain phase
  must be re-staged onto 26.2's `createBiomes` / `buildSurface` /
  `applyCarvers` with statuses NOISE → SURFACE → CARVERS (4 remaining javac
  errors, all TERRAIN). The flat-world `RandomState` factory used by the
  HORIZON_FLAT variant is also 26.3-only; that variant now throws instead of
  silently mis-generating.
- first_person: 26.3 extracted hand state into
  `LocalPlayer.firstPersonHandsAndItems` and swing state into
  `LivingEntity.SwingState`; in 26.2 the heights live on `ItemInHandRenderer`
  and swings are the `swinging` / `swingTime` fields.
- item_model: 26.3's model rework (`ResourceManager.Selector`,
  `ItemQuads`, `MaterialBaker`, `BundleContents.asMutable`,
  `VanillaRegistries.createWorldLookup`) — the oracle needs re-derivation
  against 26.2's ModelBakery API.
- knockback client: `ServerboundSwingPacket` has no `INSTANCE` singleton in
  26.2 (it is constructed with the hand), and
  `ServerboundAcceptTeleportationPacket` gains a parameter.
- game_loop: `LocalPlayer` constructor, `MouseHandler.onMove` / `onScroll`
  access, and the input `Type.KEYBOARD` constant all differ.
- tab_list (`tools/test_tab_list_parity.py`, outside the port harness set):
  authlib 9.0.75 has no `MinecraftServicesSessionService` /
  `MinecraftServicesDiscoveryService`; the session service is
  `YggdrasilMinecraftSessionService` with a different construction path
  (29 javac errors).
- The wider audit suite (~90 harnesses) has further 26.3-only families:
  the density-function / material-rule package restructure
  (`levelgen.densityfunction.*`, `levelgen.material.*`, ~50 files),
  `com.mojang.renderpearl` (HUD, loom, pack screens), `world.level.blockscan`
  (block updates, cushion placement), cushions, the 26.3 brewing rework
  (`BrewingFuel`, `BrewingRecipe`, `PotionIngredient`), `world.clock`
  (`ClockInstance`), `net.minecraft.util.Prediction`, and
  `Actor.MoveSimulationType`. Measured with a bulk javac of the whole oracle
  tree against 26.2: 1,136 errors in 156 files before triage; every error is
  attributable to one of the families named in this document.

### Behaviour work from the diff (not started)

The diff's ranked items beyond the packet work: DataFixers for data versions
4996–5016 (26.3's fixers that 26.2 predates), the sign block-entity split
(`sign_text_front` / `sign_text_back` components return in 26.2), cooking fuel
and compostable data components (26.2 has `Level.fuelValues()` instead),
26.3-only content absent in 26.2 (~90 blocks, ~121 items, the shelf mushroom
and cushion among them), and the registry/worldgen shifts the regenerated
tables already encode. `docs/version-ports.md` in this tree describes the
order to take them in.
