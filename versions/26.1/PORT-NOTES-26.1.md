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

## Behaviour port — rank 1 closed (2026-11-23, branch behaviour/26.1)

Rank 1: 7/7. Items 2/3/5/6/7 were closed by the completion pass and are
verified here, not re-ported; items 1 and 4 were ported from the 26.2
parent's closures, every behaviour verified against 26.1's own named
game jars (`26.1.x/versions/26.1/jars/{server-game,client}.jar`). The
26.2→26.1 jardiff (`diff-26.2-26.1.json`, in the triage dir) governs
rank-2 triage. Ledger: `rank1-progress.json`.

1. **Components (+2 −14).** Carried over 26.2's closure (422ffac1, 25
   files) with 26.1 corrections. The 26.1 component registry has 110
   types (26.2: 111); the only difference is `minecraft:sulfur_cube_content`
   (26.2-only), so `DATA_COMPONENT_TYPES` drops to 110 and the tree keeps
   its own `item.rs` form (`SULFUR_CUBE_CONTENT = -1000` marker,
   `SIGN_TEXT_FRONT/BACK = -1001/-1002` markers, `PAINTING_VARIANT = 102`).
   Jar-verified identical 26.2→26.1: `FuelValues`, `ComposterBlock`,
   `PotDecorations`, `HoeItem`, `ShovelItem`, `Item`,
   `Villager.wantsToPickUp`; rename-only: `AxeItem.useOn` (advancements
   trigger package move), `LivingEntity.getVisibilityPercent` and
   `BrewingStandBlockEntity` (`EntityTypes`→`EntityType`,
   `BlockEntityTypes`→`BlockEntityType`). `Villager.FOOD_POINTS` from the
   26.1 `<clinit>`: bread 4, potato/carrot/beetroot 1. The 23 sherds in
   `DecoratedPotPatterns.ITEM_TO_POT_TEXTURE` match the ported table
   exactly. **Copper is a real 26.2→26.1 difference:** 26.1's
   `HoneycombItem.WAXABLES` has 48 pairs and
   `WeatheringCopper.NEXT_BY_BLOCK` 36 pairs, both over 12 families —
   bars, lantern and chain exist as blocks (all 8 variants each) but do
   not weather or wax in 26.1, so `copper_stages()` drops them; scrape
   is unwaxed-only (`PREVIOUS_BY_BLOCK = NEXT.inverse()`, b75f738d).
   `MapItem.update` also differs 26.2→26.1 (early-return restructure)
   but does not touch the `map_color` tint work. Supporting fixes
   carried over: per-state `blocksMotion()` (c54f13b6;
   `BlockStateBase.blocksMotion` is bytecode-identical to 26.2's:
   `isSolid` except cobweb and bamboo sapling — this answers the CI
   profile's missing `blocks_motion_in_heightmap` tag failures),
   the NBT-form `{Name, Properties}` block-state codec in `facts.json`
   (e8b568a1), pack-compat fixtures re-pinned to 26.1 formats 101.1/84.0
   (8c4582c6), and the vibration test re-pinned to particle id 48
   (a86e4c05; 26.1 registry).
2. **vanilla_command_order.** Present: `crates/horizon-command/src/vanilla_order.rs`
   (228 lines, generated header).
3. **Protocol.** `Swing { hand: i32 => VarInt }`, `spectate_entity` at id
   62; no `SWING_ANIMATION`/`Serverbound::Punch`/`SPECTATOR_ACTION`.
4. **Save format.** `CarvingStepRemoveFix` added at key 4057 (verified
   in 26.1's `DataFixers` static init: `sipush 4057` +
   `new CarvingStepRemoveFix`); all 5 fix classes exist in the 26.1 jar.
   `update()` key-filters, so 26.2/26.3-era fixes (4885+) never run at
   `CURRENT = 4786`. Tests updated: `current_data_is_left_alone`
   5023→4786, block-states test rewritten as
   `current_block_states_keep_their_field_names`, new
   `carving_masks_become_the_carving_mask`. Residual: 19 changed datafix
   classes (26.2 had 8); the datafix harness arbitrates old-schema risk.
5. **Facts.** `version.rs`: 26.1 / 775 / 4786 / (84,0) / (101,1) / Java 25.
6. **Report format.** No-change: nothing reads the `definition` field.
7. **Prepare.** CI run 43062079306: `prepare: ok`, 30.7 s.

## Behaviour port — rank 2 in progress (2026-11-23, branch behaviour/26.1)

Rank 2: 101/498 closed (rank 1: 7/7; total 108/795). Triage:
first-pass buckets B 265 / A 231 / D 2; class-level 26.2→26.1 split
(zip CRC) B1c 233 / B2c 32 / A1 138 / A2 93 / D 2. The full-jardiff run
was abandoned (projected ~3 h, one JVM per class); the lazy replacement
is `jardiff_batch.py` (50 classes per javap call, ~50x faster) over the
1,210 byte-differing named classes of the open B1c/A1 items.

Batches closed so far (per-item evidence in `rank2-progress.json`):
- 1+2: 82 no-change closures (A2/B2c, 26.2 counterparts verified
  no-change, covering Rust code identical between the trees).
- 3a: 5 ported carry-overs (70 palette paths, 192 preedit area, 196
  fluid list form + lava-fire walk, 246 melt event, 351 tooltip keys).
- 3b: 2 decoder-arm removals (292 lerp/distance_to_point, 435 ore_vein).
- 3c: bc42387a carried over (282 gradient/y_clamped_gradient, 293 straw
  beds, 296 placement modifiers, 241 SwingState for item 281, open).
- 3d: c03452c8 carried over (221 kill attribution, 449 cushion removal)
  + 302 eyeblossom scan + 135 DimensionOrigin removal.
- 3e: 137 TreeGrower secondary-chance form (+ bonemeal.rs carry-over:
  no canGrow check, nextInt grass rolls; 26.1 inline-feature support
  preserved) and 168 port-pack gating (26.1 series only).
- 3f: 242 loot validation warn-only, 313 state providers (long type
  names, object-only codec, Axis.getRandom; noise.rs kept f32 — the
  26.1 NoiseStack is f32, unlike the 26.2 f64 engine).
- 3g: 379 enderman teleport (blocks-motion scan; tags absent in 26.1).

Real 26.2→26.1 differences found (not carry-overs): copper bars/lantern/
chain do not weather or wax in 26.1 (12 families, not 15);
DensityFunctions bootstrap has weird_scaled_sampler + blend_density and
lacks interval_select; BoneMealItem.useOn falls through to PASS (26.2:
SUCCESS); Window.updateFullscreenIfChanged/changeFullscreenVideoMode
differ (item 96 needs fresh porting); LivingEntity combat/knockback
methods differ (other items).

Deferred: 35 (blending f32→f64 cascade), 96 (window.rs fresh port),
117 (environment-attributes ARGB color system), 125 (blending.rs, same
f64 cascade as 35), 150 (synth.rs f64 noise engine + 6 consumers),
227 (brewing: 26.1 matches 26.2's PotionBrewing shape — no BrewingRecipe
class, no recipe/brewing data, BrewingStandBlockEntity has only
brewTime/fuel — but the mix table differs: 41 addMix + 11 addStartMix +
containers vs 26.2's 65, so the table must be extracted fresh from the
26.1 jar; the tree's data-driven loader and total_fuel/total_brew_time
UI are the 26.3 forms), 288 (EntityPositionSync/InterpolationHandler
subsystem). Item 281 stays open (SwingState aspect carried over; 15 more
world/entity classes to review).

Batch 6 (2026-11-23): 37 no-change closures — remaining B2/A2 items whose
26.2 counterparts were verified no-change, closed by manual diff review
(the title file differs between the trees, but every hunk is outside the
item's named methods: comments, rank-1 fixtures, or another item's port).

Batch 7 (2026-11-23): 3 ported carry-overs, all jar-verified — 129
natural spawning (fresh_dimensions pufferfish x0.5, spawn_cost at the
quart-origin biome, spawn_obstruction doc), 170 station tooltips
(ChargedProjectiles multiple line uses getDisplayName; container_lines
already name-first), 243 cats (CatLieOnBed + #beds, hardcoded can_sit_on,
giveMorningGift teleports from the leash holder via the 4-arg
randomTeleport).

Batch 8 (2026-11-23): 7 no-change closures — B2/A2 items with no 26.2
counterpart: 305 (math.rs identical), 348 (leash-holder code identical;
diff is the `slots` subcommand), 354 (extractSpawnerData region
identical; diff is setEntityData), 448/476/486/488 (0 methods named in
Horizon).

Run-3 test-health fixes (crate-tests 45172126527, head c856d48b: 5 pass /
world + client-lib fail, 0 new failures vs run 2): requires_file_fixing
bound by CURRENT (26.1.1 d16b413a — 4899 postdates 26.1, the unbound
comparison claimed file fixing for every world; the 28 title failures);
Realms test Client-Ref "26.1" / protocol "775" (26.1.1 2a2419cc, jar:
SharedConstants.getProtocolVersion = 775); router test samples at
[4, 4, -32] (the y_clamped_gradient fixture needs a Y in range; 26.1 has
no axis-based `gradient` type — the bootstrap lists y_clamped_gradient
only — so the 26.1.1 test's X-gradient fixture does not apply here).
Still open: `failed_upgrade_keeps_old_data_version` (pre-existing, also
in the 26.1.2 baseline). Run 45174862114 dispatched for the fixed head
f6de46ae.

Batch 9 (2026-11-23): 4 closures — 60 NbtIo.read codegen-only, 57
MerchantMenu.removed codegen-only, 216 FontTexture#add unchanged
(GpuFormat->TextureFormat enum rename in the ctor only), 248 structure
processors ported: RuleTest registry is exactly the six vanilla types in
26.1 (no height_match/not/all_of/any_of; BlockState.CODEC byte-identical
to 26.2), so rule_test.rs takes the 26.2 form (2-arg test, full=true
block-state decode) with call-site updates in processor.rs,
block_feature.rs, ore_feature.rs and the harness scope; BlockRot DIVERGES
from 26.2 — 26.1 tests the TEMPLATE info's state for rottability, so
process_block/process_list thread the template state through (regression
test block_rot_tests_the_template_state, green locally).

Batch 10 (2026-11-23): 4 no-change closures — 251 GuiEntityRenderer
(submission plumbing only; same draw sequence), 254 handleKey (same ops,
reordered; the intended-profile check has no Rust counterpart), 274
copper chest mapping contents identical (8 pairs; construction differs
only), 283 TurtleMoveControl tick/updateSpeed instruction-identical
after normalizing the generic-vs-field receiver access.

Batch 11 (2026-11-23): 7 closures (196/498) — 287/290/300/307
no-change (EntityTypes->EntityType renames; FenceGateBlock.isPathfindable
codegen-only). 287 surfaced a real 26.1 difference for item 165:
SweetBerryBushBlock.isValidBonemealTarget requires age<3 AND air above
AND inside build height in 26.1 (age only in 26.2). 291 ported:
MobCategory has no getDebugAbbreviation in 26.1, so spawn-count
abbreviations are the first letter of getName() (M/C/A/A/U/W/W/M).
308 ported comment-only: no canBeABaby in 26.1 (isBaby is
getAge() < 0 with four overrides; Slime/MagmaCube are not AgeableMobs).
309 ported: ArmadilloBallUp.checkExtraStartConditions is onGround only
(the 26.3 !isInLiquid check dropped), plus the jar-wide baby-dims
mechanism — 26.1 has BABY_DIMENSIONS in only 20 classes vs 36 in 26.2,
every other baby taking the type box scaled by getAgeScale (0.5
default; goat 0.55, armadillo/camel 0.6, dolphin 0.65, happy ghast
0.2375). 29 Baby-table entries corrected (zombie family 0.99 tall,
axolotl 0.5x0.25, squid 0.63 tall, chicken eye 0.28, cow eye 0.665,
fox eye 0.2975, no eye overrides on armadillo/dolphin/happy ghast/
panda); camel sitting baby dims are SITTING_DIMENSIONS x 0.6
(1.02, 0.567, 0.507). Regression test
baby_boxes_are_the_26_1_boxes_not_the_26_2_ones; cow test eye
0.69 -> 0.665.

Batch 12 (2026-11-23): 8 closures (204/498) — 315/317/327/332/336/337
no-change (package moves, renames, pure refactors; Ravager#blockedByItem
identical body, 26.2 only widened the signature; VillagerData.DEFAULT_TYPE
is PLAINS). 334 ported comment-only: the evoker ally check uses the
vex's direct owner in 26.1 (no getRootOwner; the Rust already read
counter2). 338 ported: the peaceful gate moved — 26.2 put it at the top
of SpawnPlacements.checkSpawnRules, 26.1 keeps it in the per-type
predicates, so the top gate is removed and !peaceful restored to
monster_rules, the any-light arms (blaze/breeze/zoglin), pillager and
endermite/silverfish (verified via the SpawnPlacements bootstrap table);
phantom/shulker correctly have no peaceful check. Regression test
peaceful_blocks_monster_rules_but_not_plain_mob_rules.

Note: the leakage-audit 26.2-server.jar is a Mojang BUNDLER jar (no game
classes) — server-side 26.2 jardiffs must use 26.2-client.jar, which
holds all game classes.

Batch 13 (2026-11-23): 8 closures (212/498). 45 ported comment-only:
SulfurCube is a 26.2 addition — 26.1 has no cubemob package, no
SULFUR_CUBE type, Slime still at monster/Slime extending Mob; the Rust
module is unreachable in 26.1 (the type never resolves from 26.1 data)
and kept per the programme convention (1.21.7/1.21.11/26.1.1/26.1.2 all
keep it), with a module-doc note recording the absence. 314/320/324/
339/349/350/355 no-change: getter renames, Guava-to-JDK collection
construction, receiver-access-only, the protocol constant already
pinned at 775, getCenter/atCenterOf, the EntityType rename.

Batch 14 (2026-11-23): 6 closures (218/498). 39 ported (carry-over of
26.2 #34): inhabited time accrues by timeDiff in 26.1 vanilla — the
26.1 tree had the 26.3 +1 form; added World.last_inhabited_update, the
stamp at the top of spawn_natural and += time_diff accrual, with the
26.2 tests. 44 ported: 26.2 added a post-onPlace re-check to
LevelChunk.setBlockState; 26.1 creates the block entity for the new
state unconditionally — the re-check is removed at update.rs
(chunk_set_block_state), commands.rs (place_state) and native_hooks.rs.
212 ported comment-only (26.1's applyInstantenousEffect typo).
36/40/43 no-change: an extracted helper with an identical predicate, a
field-vs-accessor change, byte-identical bake methods.

CI run 45181781216 (head 231549d2): defect suite PASS, 6/7 crate
suites PASS; the only failure was the pre-existing
failed_upgrade_keeps_old_data_version — fixed in b6260577 (test start
4899 -> 4780, below CURRENT 4786; same pattern as 26.1.1's 4786 fix),
verified locally. Run 45185167162 dispatched for head b6260577.

Test health: crate-tests run 45163597005 (rank-1 state): protocol,
items, client-core, client-render PASS; world FAIL (1 pre-existing
datafix failure, `failed_upgrade_keeps_old_data_version` — the other
baseline failure was fixed by item 4); client + client-lib FAIL on 14
lib-test compile errors (26.3-era protocol shapes). The 14 errors were
fixed by carrying over the 26.1.2 tree's test fixes (max_speed f32,
no randomization/online_mode, Animate for SwingAnimation, position for
EntityPositionSync.path, Swing for Punch); run 45168217761 dispatched
to verify. The 34 remaining `check --tests` errors are all in
tests/lan_bot.rs (integration test, not compiled by CI's --lib run;
identical in the 26.1.2 tree) — shared 26.1-line debt, out of scope.
